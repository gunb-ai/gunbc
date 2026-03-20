// Integration coverage for STS lowering via the canonical
// `dsl/extdeps/cloud/gcp/sts.dag` fixture discovered from the real `dsl/` tree.

use daglang_lower::{lower_typed_project_for_modules_with_entry, LoweredOp, ServiceOperationSpec};
use daglang_resolve::ModuleGraph;
use daglang_typecheck::{typecheck_owned_module_graph, TypedProject};
use gunbc_exec::Executable;
use gunbc_ir::transport::{HttpMethod, TransportRequest};
use gunbc_ir::Dag;
use gunbc_ir::{SecretString, Value};
use gunbc_resolve::service_ops::GenericPrepareOp;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

const STS_HARNESS_MODULE: &str = "tests.sts_transport_fixture_harness";
const STS_HARNESS_PATH: &str = "tests/sts_transport_fixture_harness.dag";
const STS_HARNESS_SOURCE: &str = r#"module tests.sts_transport_fixture_harness
import extdeps.cloud.gcp.sts { gcp.STS }

func run(subject_token: Secret, audience: NonEmptyStr) -> { access_token: Secret, expires_in: Int } {
  token = gcp.STS.Exchange(subject_token: subject_token, audience: audience)
  return { access_token: token.access_token, expires_in: token.expires_in }
}"#;

static STS_FIXTURE_ROOT_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn dsl_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../../dsl")
}

#[allow(clippy::disallowed_methods)] // Non-hermetic integration fixture: writes a temp harness and discovers the real repo DSL tree.
fn typed_project_from_discovered_fixture() -> TypedProject<'static> {
    let id = STS_FIXTURE_ROOT_COUNTER.fetch_add(1, Ordering::Relaxed);
    let fixture_root = std::env::temp_dir().join(format!(
        "daglang_lower_sts_transport_{}_{}",
        std::process::id(),
        id
    ));
    let harness_path = fixture_root.join(STS_HARNESS_PATH);
    if let Some(parent) = harness_path.parent() {
        std::fs::create_dir_all(parent)
            .unwrap_or_else(|error| panic!("failed to create {}: {error}", parent.display()));
    }
    std::fs::write(&harness_path, STS_HARNESS_SOURCE)
        .unwrap_or_else(|error| panic!("failed to write {}: {error}", harness_path.display()));

    let graph = ModuleGraph::discover(&[fixture_root.clone(), dsl_root()])
        .expect("canonical STS fixture should discover via real module resolution");
    let target_index = graph
        .modules
        .iter()
        .position(|module| module.module_path.as_dotted() == STS_HARNESS_MODULE)
        .unwrap_or_else(|| panic!("target module {STS_HARNESS_MODULE} should exist"));
    let mut reachable = HashSet::new();
    let mut queue = VecDeque::from([target_index]);

    while let Some(module_index) = queue.pop_front() {
        if !reachable.insert(module_index) {
            continue;
        }
        queue.extend(graph.modules[module_index].dependencies.iter().copied());
    }

    let kept_indices = graph
        .modules
        .iter()
        .enumerate()
        .filter_map(|(index, _)| reachable.contains(&index).then_some(index))
        .collect::<Vec<_>>();
    let remapped_indices = kept_indices
        .iter()
        .enumerate()
        .map(|(new_index, old_index)| (*old_index, new_index))
        .collect::<HashMap<_, _>>();
    let modules = kept_indices
        .into_iter()
        .map(|old_index| {
            let mut module = graph.modules[old_index].clone();
            module.dependencies = module
                .dependencies
                .iter()
                .filter_map(|dep| remapped_indices.get(dep).copied())
                .collect();
            module
        })
        .collect();

    let graph = ModuleGraph { modules };
    let typed = typecheck_owned_module_graph(graph)
        .expect("canonical STS fixture should typecheck via discovered imports");

    std::fs::remove_dir_all(&fixture_root).unwrap_or_else(|error| {
        panic!(
            "failed to remove temporary fixture root {}: {error}",
            fixture_root.display()
        )
    });

    typed
}

fn lower_target_module(typed: &TypedProject<'_>, target_module: &str) -> Dag<LoweredOp> {
    let module_lookup: HashMap<String, usize> = typed
        .modules()
        .enumerate()
        .map(|(index, module)| (module.module_path.as_dotted(), index))
        .collect();
    let target_index = typed
        .modules()
        .position(|module| module.module_path.as_dotted() == target_module)
        .unwrap_or_else(|| panic!("target module {target_module} should exist"));
    let mut scope = HashSet::new();
    let mut visited = HashSet::new();
    let mut queue = VecDeque::from([target_index]);

    while let Some(module_index) = queue.pop_front() {
        if !visited.insert(module_index) {
            continue;
        }
        let Some(module) = typed.module(module_index) else {
            continue;
        };
        scope.insert(module.module_path.as_dotted());
        for import in module.imports() {
            if let Some(import_index) = module_lookup.get(&import.as_dotted()) {
                queue.push_back(*import_index);
            }
        }
    }

    lower_typed_project_for_modules_with_entry(typed, &scope, None, Some(target_module))
        .expect("lowering should succeed")
}

#[test]
fn canonical_sts_fixture_emits_expected_oauth_urns_in_prepared_request() {
    let typed = typed_project_from_discovered_fixture();
    let dag = lower_target_module(&typed, STS_HARNESS_MODULE);
    let prepare_node = dag
        .nodes
        .iter()
        .find(|node| node.id.0.contains("prepare_transport") && node.id.0.contains("Exchange"))
        .expect("prepare transport node for STS.Exchange should exist");

    let metadata = match &prepare_node.body {
        gunbc_ir::node::NodeBody::Opaque(op) => op
            .service_call_metadata()
            .expect("service metadata should be preserved"),
        _ => panic!("expected opaque lowered node"),
    };

    let spec = metadata.spec.as_ref().expect("spec should be present");
    let ServiceOperationSpec::Rest(rest_spec) = spec else {
        panic!("expected REST spec");
    };
    let op = GenericPrepareOp {
        spec: ServiceOperationSpec::Rest(rest_spec.clone()),
    };

    let mut inputs = HashMap::new();
    inputs.insert(
        "subject_token".to_string(),
        Value::Secret(SecretString::new("tok123")),
    );
    inputs.insert(
        "audience".to_string(),
        Value::Str(
            "projects/123/locations/global/workloadIdentityPools/pool/providers/provider"
                .to_string(),
        ),
    );

    let outputs = op
        .execute(inputs)
        .expect("prepare op should build a request");
    match outputs.get("request") {
        Some(Value::Request(TransportRequest::Rest(request))) => {
            assert_eq!(request.method, HttpMethod::Post);
            assert_eq!(request.url, "https://sts.googleapis.com/v1/token");
            let body = request
                .body
                .as_ref()
                .expect("POST request should have a body");
            assert_eq!(
                body["grant_type"],
                "urn:ietf:params:oauth:grant-type:token-exchange"
            );
            assert_eq!(body["subject_token"], "tok123");
            assert_eq!(
                body["subject_token_type"],
                "urn:ietf:params:oauth:token-type:jwt"
            );
            assert_eq!(
                body["audience"],
                "projects/123/locations/global/workloadIdentityPools/pool/providers/provider"
            );
            assert_eq!(
                body["requested_token_type"],
                "urn:ietf:params:oauth:token-type:access_token"
            );
        }
        other => panic!("expected REST request, got {other:?}"),
    }
}
