//! Workflow planner admission validation (WF2).

use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

use gunbc_ir::{derive_resource_accesses, Dag, NodeBody, ResourceAccessError, ResourceId};

use crate::errors::WorkflowAdmissionError;
use crate::process_registry::{ClaimId, ProcessUnitRegistry, UnitClaim};
use crate::schema::{WorkflowOp, WorkflowSpec, WorkflowUnit};

/// Validate workflow admission with fail-closed semantics.
///
/// Returns all detected admission errors in one pass.
pub fn validate_workflow_admission(
    spec: &WorkflowSpec,
    registry: &ProcessUnitRegistry,
) -> Result<(), Vec<WorkflowAdmissionError>> {
    let mut errors = Vec::new();
    errors.extend(validate_required_claims(spec, registry));
    errors.extend(validate_effectful_claim_declarations(spec, registry));
    errors.extend(validate_conflicting_claims(spec));

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Validate fail-closed effectful claim declarations.
///
/// Nodes with write/exclusive required claims must declare matching resource
/// claim ports so admission derives from graph structure instead of side tables.
pub fn validate_effectful_claim_declarations(
    spec: &WorkflowSpec,
    registry: &ProcessUnitRegistry,
) -> Vec<WorkflowAdmissionError> {
    let declared_claims = match derive_declared_claims(spec) {
        Ok(claims) => claims,
        Err(claim_errors) => return claim_errors,
    };

    let mut errors = Vec::new();
    for node in &spec.dag.nodes {
        let NodeBody::Opaque(WorkflowUnit { op }) = &node.body else {
            continue;
        };
        let WorkflowOp::InvokeProcessUnit(process_unit) = op else {
            continue;
        };
        let Some(process_spec) = registry.get(process_unit) else {
            continue;
        };

        let effectful_required = process_spec
            .required_claims
            .iter()
            .filter(|claim| claim.access_mode != gunbc_ir::AccessMode::Read)
            .collect::<Vec<_>>();
        if effectful_required.is_empty() {
            continue;
        }

        let node_claims = declared_claims.get(&node.id).cloned().unwrap_or_default();
        let missing_claim_ports = effectful_required
            .into_iter()
            .filter(|required| {
                !node_claims.iter().any(|declared| {
                    declared.claim_id == required.claim_id
                        && declared.access_mode == required.access_mode
                })
            })
            .map(|required| required.claim_id.clone())
            .collect::<Vec<_>>();
        if !missing_claim_ports.is_empty() {
            errors.push(WorkflowAdmissionError::UndeclaredEffectfulIo {
                node_id: node.id.clone(),
                process_unit: process_unit.clone(),
                missing_claim_ports,
            });
        }
    }

    errors
}

/// Validate that invoke units declare all required claims.
pub fn validate_required_claims(
    spec: &WorkflowSpec,
    registry: &ProcessUnitRegistry,
) -> Vec<WorkflowAdmissionError> {
    let declared_claims = match derive_declared_claims(spec) {
        Ok(claims) => claims,
        Err(claim_errors) => return claim_errors,
    };

    let mut errors = Vec::new();
    for node in &spec.dag.nodes {
        let NodeBody::Opaque(WorkflowUnit { op }) = &node.body else {
            continue;
        };
        let WorkflowOp::InvokeProcessUnit(process_unit) = op else {
            continue;
        };

        let Some(process_spec) = registry.get(process_unit) else {
            errors.push(WorkflowAdmissionError::UnknownProcessUnit {
                node_id: node.id.clone(),
                process_unit: process_unit.clone(),
            });
            continue;
        };

        let node_claims = declared_claims.get(&node.id).cloned().unwrap_or_default();
        let missing = process_spec
            .required_claims
            .iter()
            .filter(|required| {
                !node_claims.iter().any(|declared| {
                    declared.claim_id == required.claim_id
                        && declared.access_mode == required.access_mode
                })
            })
            .cloned()
            .collect::<Vec<_>>();

        if !missing.is_empty() {
            errors.push(WorkflowAdmissionError::MissingRequiredClaims {
                node_id: node.id.clone(),
                process_unit: process_unit.clone(),
                missing_claims: missing,
            });
        }
    }

    errors
}

/// Validate that unordered conflicting claims are rejected.
pub fn validate_conflicting_claims(spec: &WorkflowSpec) -> Vec<WorkflowAdmissionError> {
    let mut errors = Vec::new();

    let accesses = match derive_resource_accesses(&spec.dag) {
        Ok(accesses) => accesses,
        Err(resource_errors) => {
            errors.extend(
                resource_errors
                    .into_iter()
                    .map(workflow_resource_access_error),
            );
            return errors;
        }
    };

    let ordered_pairs = compute_ordered_pairs(&spec.dag);
    for idx in 0..accesses.len() {
        for jdx in (idx + 1)..accesses.len() {
            let left = &accesses[idx];
            let right = &accesses[jdx];

            if left.node_id == right.node_id {
                continue;
            }
            if !resource_ids_conflict(&left.resource_id, &right.resource_id) {
                continue;
            }
            if !left.mode.conflicts_with(&right.mode) {
                continue;
            }

            let left_before_right =
                ordered_pairs.contains(&(left.node_id.clone(), right.node_id.clone()));
            let right_before_left =
                ordered_pairs.contains(&(right.node_id.clone(), left.node_id.clone()));
            if left_before_right || right_before_left {
                continue;
            }

            errors.push(WorkflowAdmissionError::ConflictingClaims {
                left_node: left.node_id.clone(),
                right_node: right.node_id.clone(),
                left_claim: ClaimId::new(left.resource_id.0.clone()),
                right_claim: ClaimId::new(right.resource_id.0.clone()),
                left_mode: left.mode,
                right_mode: right.mode,
            });
        }
    }

    errors
}

fn derive_declared_claims(
    spec: &WorkflowSpec,
) -> Result<BTreeMap<gunbc_ir::NodeId, Vec<UnitClaim>>, Vec<WorkflowAdmissionError>> {
    let accesses = derive_resource_accesses(&spec.dag).map_err(|resource_errors| {
        resource_errors
            .into_iter()
            .map(workflow_resource_access_error)
            .collect::<Vec<_>>()
    })?;

    let mut claims_by_node: BTreeMap<gunbc_ir::NodeId, Vec<UnitClaim>> = BTreeMap::new();
    for access in accesses {
        claims_by_node
            .entry(access.node_id)
            .or_default()
            .push(UnitClaim::new(access.resource_id.0, access.mode));
    }
    for claims in claims_by_node.values_mut() {
        claims.sort_by(|left, right| {
            left.claim_id
                .cmp(&right.claim_id)
                .then(mode_rank(left.access_mode).cmp(&mode_rank(right.access_mode)))
        });
        claims.dedup();
    }
    Ok(claims_by_node)
}

fn workflow_resource_access_error(error: ResourceAccessError) -> WorkflowAdmissionError {
    WorkflowAdmissionError::ResourceAccessMetadataInvalid { error }
}

fn mode_rank(mode: gunbc_ir::AccessMode) -> u8 {
    match mode {
        gunbc_ir::AccessMode::Read => 0,
        gunbc_ir::AccessMode::Write => 1,
        gunbc_ir::AccessMode::Exclusive => 2,
    }
}

fn resource_ids_conflict(left: &ResourceId, right: &ResourceId) -> bool {
    // Resource conflict semantics:
    // - exact same resource ID conflicts,
    // - coarse file domain claim (`file`) conflicts with any qualified file
    //   claim (`file:<path>`),
    // - qualified file claims only conflict with the same qualified path,
    // - non-file domains only conflict on exact ID match.
    if left == right {
        return true;
    }

    enum ResourceScope<'a> {
        FileRoot,
        FileScoped(&'a str),
        Other(&'a str),
    }

    fn classify(resource_id: &ResourceId) -> ResourceScope<'_> {
        if resource_id.0 == "file" {
            ResourceScope::FileRoot
        } else if let Some(rest) = resource_id.0.strip_prefix("file:") {
            ResourceScope::FileScoped(rest)
        } else {
            ResourceScope::Other(resource_id.0.as_str())
        }
    }

    match (classify(left), classify(right)) {
        (ResourceScope::FileRoot, ResourceScope::FileRoot) => true,
        (ResourceScope::FileRoot, ResourceScope::FileScoped(_))
        | (ResourceScope::FileScoped(_), ResourceScope::FileRoot) => true,
        (ResourceScope::FileScoped(left_path), ResourceScope::FileScoped(right_path)) => {
            left_path == right_path
        }
        (ResourceScope::Other(left_id), ResourceScope::Other(right_id)) => left_id == right_id,
        _ => false,
    }
}

fn compute_ordered_pairs(dag: &Dag<WorkflowUnit>) -> HashSet<(gunbc_ir::NodeId, gunbc_ir::NodeId)> {
    let mut adjacency: HashMap<gunbc_ir::NodeId, Vec<gunbc_ir::NodeId>> = HashMap::new();
    for node in &dag.nodes {
        adjacency.entry(node.id.clone()).or_default();
    }
    for edge in &dag.edges {
        adjacency
            .entry(edge.from_node.clone())
            .or_default()
            .push(edge.to_node.clone());
    }

    let mut ordered = HashSet::new();
    for start in adjacency.keys().cloned().collect::<Vec<_>>() {
        let mut queue = VecDeque::new();
        let mut seen = HashSet::new();
        queue.push_back(start.clone());
        while let Some(current) = queue.pop_front() {
            if !seen.insert(current.clone()) {
                continue;
            }
            if current != start {
                ordered.insert((start.clone(), current.clone()));
            }
            if let Some(children) = adjacency.get(&current) {
                for child in children {
                    queue.push_back(child.clone());
                }
            }
        }
    }

    ordered
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::{AccessMode, Edge, Node, Port, ResourceAccessError};

    use crate::process_registry::{ProcessUnitRef, ProcessUnitSpec};
    use crate::schema::{
        required_input_contract, required_output_contract, WorkflowId, WorkflowSpec,
    };

    fn invoke_node(
        id: &str,
        process_id: &str,
        unit_id: &str,
        claims: &[(&str, AccessMode)],
    ) -> Node<WorkflowUnit> {
        let mut inputs = required_input_contract();
        for (claim, mode) in claims {
            inputs.push(Port::resource(*claim, "ResourceHandle", *mode));
        }
        Node::opaque(
            id,
            inputs,
            required_output_contract(),
            WorkflowUnit::new(WorkflowOp::InvokeProcessUnit(ProcessUnitRef::new(
                process_id, unit_id,
            ))),
        )
    }

    fn registry_for_two_nodes(
        required_a: Vec<UnitClaim>,
        required_b: Vec<UnitClaim>,
    ) -> ProcessUnitRegistry {
        let mut registry = ProcessUnitRegistry::new();
        registry.register(ProcessUnitSpec::new(
            ProcessUnitRef::new("wf", "a"),
            1,
            required_a,
        ));
        registry.register(ProcessUnitSpec::new(
            ProcessUnitRef::new("wf", "b"),
            1,
            required_b,
        ));
        registry
    }

    #[test]
    fn read_read_claims_are_admitted_in_parallel() {
        let mut dag = Dag::new();
        dag.add_node(invoke_node(
            "wf.a",
            "wf",
            "a",
            &[("file:workspace", AccessMode::Read)],
        ));
        dag.add_node(invoke_node(
            "wf.b",
            "wf",
            "b",
            &[("file:workspace", AccessMode::Read)],
        ));

        let spec = WorkflowSpec::new(WorkflowId::new("wf"), dag, 1);
        let registry = registry_for_two_nodes(
            vec![UnitClaim::read("file:workspace")],
            vec![UnitClaim::read("file:workspace")],
        );

        assert!(
            validate_workflow_admission(&spec, &registry).is_ok(),
            "read/read claims should be admitted"
        );
    }

    #[test]
    fn write_write_claims_are_rejected_when_unordered() {
        let mut dag = Dag::new();
        dag.add_node(invoke_node(
            "wf.a",
            "wf",
            "a",
            &[("file:workspace", AccessMode::Write)],
        ));
        dag.add_node(invoke_node(
            "wf.b",
            "wf",
            "b",
            &[("file:workspace", AccessMode::Write)],
        ));

        let spec = WorkflowSpec::new(WorkflowId::new("wf"), dag, 1);
        let registry = registry_for_two_nodes(
            vec![UnitClaim::write("file:workspace")],
            vec![UnitClaim::write("file:workspace")],
        );
        let errors =
            validate_workflow_admission(&spec, &registry).expect_err("write/write should fail");

        let conflict = errors
            .iter()
            .find_map(|error| match error {
                WorkflowAdmissionError::ConflictingClaims {
                    left_node,
                    right_node,
                    left_claim,
                    right_claim,
                    ..
                } => Some((
                    left_node.0.clone(),
                    right_node.0.clone(),
                    left_claim.0.clone(),
                    right_claim.0.clone(),
                )),
                _ => None,
            })
            .expect("conflicting claims error should be present");
        assert!(conflict.0 == "wf.a" || conflict.0 == "wf.b");
        assert!(conflict.1 == "wf.a" || conflict.1 == "wf.b");
        assert_eq!(conflict.2, "file:workspace");
        assert_eq!(conflict.3, "file:workspace");
    }

    #[test]
    fn missing_required_claims_fail_closed() {
        let mut dag = Dag::new();
        dag.add_node(invoke_node("wf.a", "wf", "a", &[]));
        dag.add_node(invoke_node(
            "wf.b",
            "wf",
            "b",
            &[("file:workspace", AccessMode::Read)],
        ));
        dag.add_edge(Edge::control("wf.a", "commit", "wf.b", "after"));

        let spec = WorkflowSpec::new(WorkflowId::new("wf"), dag, 1);
        let registry = registry_for_two_nodes(
            vec![UnitClaim::write("file:workspace")],
            vec![UnitClaim::read("file:workspace")],
        );
        let errors =
            validate_workflow_admission(&spec, &registry).expect_err("missing claims should fail");
        assert!(errors.iter().any(|error| matches!(
            error,
            WorkflowAdmissionError::MissingRequiredClaims { node_id, .. } if node_id.0 == "wf.a"
        )));
    }

    #[test]
    fn effectful_claim_without_declared_resource_port_fails_closed() {
        let mut dag = Dag::new();
        dag.add_node(invoke_node("wf.a", "wf", "a", &[]));
        let spec = WorkflowSpec::new(WorkflowId::new("wf"), dag, 1);
        let registry = registry_for_two_nodes(
            vec![UnitClaim::write("file:workspace")],
            vec![UnitClaim::read("file:workspace")],
        );
        let errors = validate_effectful_claim_declarations(&spec, &registry);
        assert!(errors.iter().any(|error| matches!(
            error,
            WorkflowAdmissionError::UndeclaredEffectfulIo { node_id, .. } if node_id.0 == "wf.a"
        )));
    }

    #[test]
    fn missing_resource_id_surfaces_exact_metadata_error() {
        let mut inputs = required_input_contract();
        let mut db_input = Port::new("db_conn", "DbHandle");
        db_input.resource_access = Some(AccessMode::Write);
        inputs.push(db_input);

        let mut dag = Dag::new();
        dag.add_node(Node::opaque(
            "wf.a",
            inputs,
            required_output_contract(),
            WorkflowUnit::new(WorkflowOp::InvokeProcessUnit(ProcessUnitRef::new(
                "wf", "a",
            ))),
        ));

        let spec = WorkflowSpec::new(WorkflowId::new("wf"), dag, 1);
        let registry = registry_for_two_nodes(vec![UnitClaim::write("db")], vec![]);
        let errors = validate_workflow_admission(&spec, &registry)
            .expect_err("missing resource_id must fail");

        assert!(errors.iter().any(|error| matches!(
            error,
            WorkflowAdmissionError::ResourceAccessMetadataInvalid {
                error: ResourceAccessError::MissingResourceId { node_id, port_name },
            } if node_id.0 == "wf.a" && port_name == "db_conn"
        )));
        assert!(
            errors
                .iter()
                .all(|error| !error.to_string().contains("<missing resource_id")),
            "workflow admission should not fabricate placeholder claim ids: {errors:?}"
        );
    }

    #[test]
    fn file_root_claim_conflicts_with_file_scoped_claims() {
        assert!(resource_ids_conflict(
            &ResourceId::new("file"),
            &ResourceId::new("file:workspace")
        ));
        assert!(resource_ids_conflict(
            &ResourceId::new("file:workspace"),
            &ResourceId::new("file")
        ));
    }

    #[test]
    fn different_file_scoped_claims_do_not_conflict() {
        assert!(!resource_ids_conflict(
            &ResourceId::new("file:workspace"),
            &ResourceId::new("file:tmp")
        ));
    }

    #[test]
    fn identical_scoped_claims_conflict() {
        assert!(resource_ids_conflict(
            &ResourceId::new("file:workspace"),
            &ResourceId::new("file:workspace")
        ));
    }

    #[test]
    fn non_file_domains_only_conflict_on_exact_match() {
        assert!(!resource_ids_conflict(
            &ResourceId::new("tool:cargo"),
            &ResourceId::new("tool:rustc")
        ));
        assert!(resource_ids_conflict(
            &ResourceId::new("tool:cargo"),
            &ResourceId::new("tool:cargo")
        ));
    }

    #[test]
    fn missing_resource_id_metadata_is_reported_without_fabricated_claim_id() {
        let mut inputs = required_input_contract();
        let mut claim = Port::scalar("db_conn", "ResourceHandle");
        claim.resource_access = Some(AccessMode::Write);
        inputs.push(claim);

        let mut dag = Dag::new();
        dag.add_node(Node::opaque(
            "wf.a",
            inputs,
            required_output_contract(),
            WorkflowUnit::new(WorkflowOp::InvokeProcessUnit(ProcessUnitRef::new(
                "wf", "a",
            ))),
        ));

        let spec = WorkflowSpec::new(WorkflowId::new("wf"), dag, 1);
        let errors = validate_conflicting_claims(&spec);
        assert_eq!(errors.len(), 1);
        match &errors[0] {
            WorkflowAdmissionError::ResourceAccessMetadataInvalid {
                error: ResourceAccessError::MissingResourceId { node_id, port_name },
            } => {
                assert_eq!(node_id.0, "wf.a");
                assert_eq!(port_name, "db_conn");
                assert_eq!(
                    errors[0].to_string(),
                    "resource input 'db_conn' on node 'wf.a' has resource_access but no resource_id"
                );
            }
            other => {
                panic!("expected ResourceAccessMetadataInvalid(MissingResourceId), got {other:?}")
            }
        }
    }

    #[test]
    fn admission_fails_fast_on_missing_resource_id_without_cascade() {
        // A port with resource_access but no resource_id should cause
        // validate_workflow_admission to report only the metadata failure,
        // not cascade into MissingRequiredClaims or UndeclaredEffectfulIo.
        let mut inputs = required_input_contract();
        let mut claim = Port::scalar("db_conn", "ResourceHandle");
        claim.resource_access = Some(AccessMode::Write);
        inputs.push(claim);

        let mut dag = Dag::new();
        dag.add_node(Node::opaque(
            "wf.a",
            inputs,
            required_output_contract(),
            WorkflowUnit::new(WorkflowOp::InvokeProcessUnit(ProcessUnitRef::new(
                "wf", "a",
            ))),
        ));

        let spec = WorkflowSpec::new(WorkflowId::new("wf"), dag, 1);
        let registry = {
            let mut r = ProcessUnitRegistry::new();
            r.register(ProcessUnitSpec::new(
                ProcessUnitRef::new("wf", "a"),
                1,
                vec![UnitClaim::write("db_conn")],
            ));
            r
        };

        let errors = validate_workflow_admission(&spec, &registry)
            .expect_err("admission should fail for missing resource_id");

        // Only metadata errors — no MissingRequiredClaims or UndeclaredEffectfulIo noise.
        for error in &errors {
            assert!(
                matches!(
                    error,
                    WorkflowAdmissionError::ResourceAccessMetadataInvalid { .. }
                ),
                "expected only resource metadata errors, got: {error:?}"
            );
        }
        assert!(errors.iter().any(|e| matches!(
            e,
            WorkflowAdmissionError::ResourceAccessMetadataInvalid {
                error: ResourceAccessError::MissingResourceId { node_id, .. },
            } if node_id.0 == "wf.a"
        )));
    }
}
