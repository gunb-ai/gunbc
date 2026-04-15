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
// **Convention support.** Parameters whose name starts with `_`
// are conventionally unused (Rust style); the lens skips them
// when `ignore_underscore_prefix` is set.
//
// **Pure reader, zero substrate changes.** The lens reads
// `Dag` + `UnusedParametersConfig` and returns
// `Vec<UnusedParameter>`. No mutation, no side tables, no
// caches. Same template as `lens_depth`, `lens_provenance`,
// `lens_cost`.

use std::collections::HashSet;

use crate::dag::{Behavior, BindNode, Dag, DeclarationId, NodeId, PortId};
use crate::diagnostics::SourceSpan;

/// Configuration for the unused-parameters lens.
///
/// **Dissolution receipt — 🟢 TERMINAL.** Two scalar fields with
/// distinct concerns: `scope` is the optional restriction set,
/// `ignore_underscore_prefix` is a convention toggle. Each is a
/// configuration knob that downstream applications set
/// independently. Pattern 1 (fact placement) fails because the
/// fields encode different aspects (which functions vs which
/// parameters); Pattern 2 (variant-is-data) is N/A for a struct;
/// Pattern 3 (algebraic form) fails for the same reason; Pattern
/// 4 (dimensional) fails. Verdict: terminal at the lens-library-
/// initial-three scope. Future lenses with similar configs grow
/// independently.
#[derive(Debug, Clone, Default)]
pub struct UnusedParametersConfig {
    /// Restrict the scan to specific function declarations. If
    /// empty, scan every function-shaped Bind in the Dag. The
    /// declaration-id form (rather than name strings) keeps the
    /// scope description in the substrate's identity space —
    /// callers walk the Dag to pick which declarations to scan
    /// rather than naming them by string.
    pub scope: Vec<DeclarationId>,
    /// When true, parameters whose name starts with `_` are
    /// skipped. Rust convention: `_unused`, `_ignored`, etc.
    pub ignore_underscore_prefix: bool,
}

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
    /// Dag (or the subset selected by `config.scope`). Returns
    /// one `UnusedParameter` per parameter port that the
    /// function body never reads.
    pub fn query(&self, config: &UnusedParametersConfig) -> Vec<UnusedParameter> {
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
            // Optional scope filter: if a non-empty scope is set,
            // only check binds whose own underlying declaration
            // matches. v3's BindNode doesn't carry a back-pointer
            // to its declaration at this scope, so the scope
            // filter currently checks against an empty list (no-op
            // when scope is empty). When the substrate grows a
            // bind→declaration edge, this filter activates.
            if !config.scope.is_empty() {
                // No-op until BindNode → DeclarationId is plumbed.
                // Documented as future work; for now the scope is
                // effectively "all functions in the Dag."
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
/// - Loop: `l.source` + `l.init`. The body sub-DAG via
///   `l.body` is its own NodeId; this lens does not recurse
///   into nested function bodies — each function gets its own
///   lens invocation via `query`.
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
            }
            Behavior::Bind(b) => {
                queue.push(b.value);
            }
        }
    }

    referenced
}
