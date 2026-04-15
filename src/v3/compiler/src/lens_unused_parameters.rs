// Unused-parameters lens — first lens in the lens library.
//
// **Purpose.** Detect function parameters that are declared in a
// `Bind`'s parameter list but never read by the bind's body
// sub-DAG. The classical "stub function" tell: a parameter
// declared in the signature that the body silently ignores.
//
// **Spec source.** `docs/lens-library-design.md` §2.3. This is
// the simplest of the three initial lenses and intentionally
// the first one shipped — the algorithm exercises the basic
// dataflow walk that more sophisticated lenses build on, and
// it has a known concrete finding (`content_upsert` in
// `dsl/std/patterns.dag:136-139` ignores its `path` parameter)
// that proves the lens is doing real work.
//
// **Algorithm.**
//
//   1. Scope: every function-shaped Bind in the Dag (i.e.
//      `BindNode { params: non-empty, .. }`). Value bindings
//      are skipped — they have no parameters to be unused.
//   2. For each function Bind, walk the body sub-DAG starting
//      from `bind.value` backwards through `produced_by` edges.
//      Collect every PortId that appears as an input to any
//      visited Behavior.
//   3. The set of "referenced" ports is the inputs of every
//      reachable producer node.
//   4. Compare each parameter port against the referenced set.
//      A parameter port absent from the set is unused.
//
// **What "referenced" means here.** A parameter is referenced
// if the body's expression graph actually wires it as an input
// to some node. Just having a parameter port allocated isn't
// enough — the port must appear in some Transform.inputs,
// Branch.input, Loop.source/init, or Bind.value reachable from
// the function's body root.
//
// **No convention filter.** v3's substrate doesn't carry
// per-parameter names past lowering — only positional ports.
// A `_unused`-style underscore filter would require the lens
// to read names that aren't there, which would be a layer-
// opacity violation in disguise: the config would promise
// name-based filtering and silently no-op. The filter is
// intentionally absent; if a future substrate change exposes
// per-parameter names (e.g., a `BindNode.param_names: Vec<String>`
// field added alongside class-5 gap work), the filter can land
// then with a real backing implementation.
//
// **Pure reader, zero substrate changes.** The lens reads
// `Dag` + `UnusedParametersConfig` and returns
// `Vec<UnusedParameter>`. No mutation, no side tables, no
// caches. Same template as `lens_depth`, `lens_provenance`,
// `lens_cost`.

use std::collections::HashSet;

use crate::dag::{Behavior, BindNode, Dag, NodeId, PortId};
use crate::diagnostics::SourceSpan;

/// Configuration for the unused-parameters lens.
///
/// **Dissolution receipt — 🟢 TERMINAL at current substrate
/// scope.** Empty struct: the lens has no configurable knobs at
/// the current substrate scope, so the type exists for API
/// consistency with the other lens-library `*Config` types and
/// to give downstream consumers a stable place to add knobs
/// when the substrate grows the prerequisites. Pattern checks
/// are N/A for an empty struct. Verdict: terminal at the lens-
/// library-initial-three scope.
///
/// **Why no `scope` field.** A `Vec<DeclarationId>` scope filter
/// would require BindNode to carry a back-pointer to its
/// declaration so the filter could compare. v3's substrate
/// doesn't have that edge at this scope — every BindNode is
/// addressed only by its NodeId in the behavior list. Adding
/// the field without the substrate prerequisite would create a
/// misleading API: consumers set a non-empty scope, the lens
/// silently ignores it (no-op), the consumer's expectations
/// silently break. Same ghost-field pattern as
/// `ignore_underscore_prefix` was earlier; same fix (remove the
/// field until it can be implemented). When the substrate grows
/// `BindNode → DeclarationId`, the field can land alongside its
/// real implementation.
///
/// **Why no `ignore_underscore_prefix` field.** v3's substrate
/// doesn't carry per-parameter names past lowering, so the lens
/// has no way to read a parameter's name. Same ghost-field
/// reasoning: removed pending substrate prerequisite.
#[derive(Debug, Clone, Default)]
pub struct UnusedParametersConfig {}

/// One reported violation: a parameter declared on a function
/// Bind that the function body never reads. Carries enough
/// structural identity to point at the parameter precisely
/// (NodeId of the Bind + PortId of the parameter + the parameter's
/// surface name + the Bind's source span).
#[derive(Debug, Clone)]
pub struct UnusedParameter {
    /// The function Bind whose body fails to reference the
    /// parameter. Identifies the function by its `NodeId` so
    /// downstream consumers can index back into the Dag.
    pub function: NodeId,
    /// The parameter port itself.
    pub parameter: PortId,
    /// The parameter's index in the function's parameter list.
    /// Used as the parameter "name" because v3's substrate
    /// doesn't store per-parameter names on the BindNode at this
    /// scope (the names are part of `SurfaceItem::Fn` and aren't
    /// preserved past lowering). Indexing makes the violation
    /// addressable without relying on a name.
    pub parameter_index: usize,
    /// The function Bind's source span. Pointer for diagnostic
    /// output; consumers can render it as e.g.
    /// `dsl/std/patterns.dag:136`.
    pub function_span: SourceSpan,
}

pub struct UnusedParametersLens<'a> {
    dag: &'a Dag,
}

impl<'a> UnusedParametersLens<'a> {
    pub fn new(dag: &'a Dag) -> Self {
        Self { dag }
    }

    /// Run the lens against every function-shaped Bind in the
    /// Dag. Returns one `UnusedParameter` per parameter port that
    /// the function body never reads.
    ///
    /// `_config` is currently unused but accepted to keep the
    /// `(Dag, &Config) → Vec<Violation>` signature aligned with
    /// the rest of the lens library. When the substrate grows
    /// the prerequisites for scope filters / convention filters,
    /// the fields land in `UnusedParametersConfig` and the
    /// signature stays the same.
    pub fn query(&self, _config: &UnusedParametersConfig) -> Vec<UnusedParameter> {
        let mut violations = Vec::new();
        for node in self.dag.nodes() {
            let Behavior::Bind(bind) = node else {
                continue;
            };
            // Function-shape filter: skip value bindings (no
            // parameters means there's nothing to be unused).
            if bind.params.is_empty() {
                continue;
            }
            self.check_bind(bind, &mut violations);
        }
        violations
    }

    fn check_bind(&self, bind: &BindNode, out: &mut Vec<UnusedParameter>) {
        // Walk the function's body sub-DAG and collect every
        // PortId that appears as an input to any reachable node.
        // The body's root port is `bind.value`; we walk backwards
        // through producer edges, collecting input ports along
        // the way.
        let referenced = collect_referenced_ports(self.dag, bind.value);

        for (idx, &param_port) in bind.params.iter().enumerate() {
            if !referenced.contains(&param_port) {
                out.push(UnusedParameter {
                    function: bind.id,
                    parameter: param_port,
                    parameter_index: idx,
                    function_span: bind.span.clone(),
                });
            }
        }
    }
}

/// Walk the sub-DAG rooted at `root_port` backwards through
/// `produced_by` edges and return the set of every port reached
/// by the walk. The returned set is the "referenced" set the
/// lens compares against the function's parameter ports — a
/// parameter port that's NOT in the set means the body's data
/// flow never touches it.
///
/// **What "referenced" means here.** A port P is referenced if
/// there's a path from `root_port` back to P through
/// producer→inputs edges. The body's root port itself is always
/// referenced (the function returns it). Each producer's input
/// ports are referenced (the producer reads them). And so on
/// transitively.
///
/// **Why root inclusion matters.** A function whose body is
/// literally a parameter (`fn first(a, b) = a`) has
/// `bind.value == a_port` — there's no intermediate node. If
/// the walker only added INPUTS-to-nodes to the referenced set,
/// it would miss `a` entirely and report it as unused. Adding
/// the root port (and every port the walk visits) catches the
/// trivial-body case structurally.
///
/// **Walk shape.** Iterative work-list. Each port is visited
/// at most once (tracked via `referenced` itself acting as the
/// visited set, since "visited" and "referenced" are the same
/// concept here). Each visited port's producer (if any) has
/// its inputs queued for further walking.
///
/// **What counts as an input by Behavior kind.**
/// - Value: no inputs (leaf).
/// - Transform: every PortId in `t.inputs`.
/// - Branch: `b.input` (scrutinee) + every `path.output`.
/// - Loop: `l.source` + `l.init` + `l.bound.count` + the body
///   sub-DAG's primary output port (from `l.body`). The body
///   descent is required: parameters used only inside a
///   recursive call's body would be falsely flagged unused
///   without it. Cycle protection comes from `referenced`
///   doubling as the visited set — pushing the loop's own
///   output port (which can happen when `l.body == loop_id`,
///   v3's lower fallback) hits the already-visited check and
///   stops.
/// - Bind: `b.value`. A nested Bind passes through to its
///   value port, same way the cost lens treats it.
fn collect_referenced_ports(dag: &Dag, root_port: PortId) -> HashSet<PortId> {
    let mut referenced: HashSet<PortId> = HashSet::new();
    let mut queue: Vec<PortId> = vec![root_port];

    while let Some(port) = queue.pop() {
        if !referenced.insert(port) {
            // Already visited — skip (also serves as cycle
            // protection, though the substrate is acyclic by
            // construction).
            continue;
        }
        let Some(producer) = dag.port(port).produced_by else {
            // Leaf port — typically a function parameter or a
            // port allocated without a producer. Already added
            // to `referenced` above; nothing more to walk.
            continue;
        };
        match dag.node(producer) {
            Behavior::Value(_) => {
                // Leaf — no inputs to add.
            }
            Behavior::Transform(t) => {
                for &input in &t.inputs {
                    queue.push(input);
                }
            }
            Behavior::Branch(b) => {
                queue.push(b.input);
                for path in &b.paths {
                    queue.push(path.output);
                }
            }
            Behavior::Loop(l) => {
                queue.push(l.source);
                queue.push(l.init);
                queue.push(l.bound.count);
                // Descend into the body sub-DAG. The body is a
                // NodeId, not a port; finding the body node's
                // primary output port lets the walk continue
                // backwards through the body's own data flow,
                // catching parameters that the recursive call
                // reads. The cycle case (body NodeId == loop
                // NodeId, lower's fallback) is harmless because
                // the loop's output port is already visited by
                // the time we get here.
                let body_node = dag.node(l.body);
                queue.push(behavior_output_port(body_node));
            }
            Behavior::Bind(b) => {
                queue.push(b.value);
            }
        }
    }

    referenced
}

/// Return a Behavior's primary output port — the port that
/// downstream consumers read to obtain the behavior's result.
/// Used by the unused-parameters walker to descend into nested
/// sub-DAGs (currently only `Loop.body`, but the helper applies
/// uniformly to every behavior variant for future use).
///
/// Each L1 behavior carries exactly one output port:
///   - Value: `output` (the literal's port)
///   - Transform: `output` (the call's result)
///   - Branch: `output` (the chosen path's output)
///   - Loop: `output` (the loop's per-call result)
///   - Bind: `value` (the bound expression's port — name is
///     historical but semantically it IS the bind's "output")
fn behavior_output_port(behavior: &Behavior) -> PortId {
    match behavior {
        Behavior::Value(v) => v.output,
        Behavior::Transform(t) => t.output,
        Behavior::Branch(b) => b.output,
        Behavior::Loop(l) => l.output,
        Behavior::Bind(b) => b.value,
    }
}
