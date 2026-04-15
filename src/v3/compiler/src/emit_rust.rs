// M1(3) PR-B-unwind — Rust emitter with typed declaration dispatch.
//
// **What changed from the initial PR-B cut.** The original
// `emit_rust.rs` built a `HashMap<(String, String), String>` from
// rust.dag's name-style string fields and dispatched via
// `index.lookup(...)` calls keyed on canonical primitive names.
// Every dispatch site embedded a Rust string literal naming a
// substrate concept. That pattern was the M1(2.7) name-bridge
// regression the
// review loop spent fourteen rounds eliminating from the inference
// layer, just at the emit layer. The unwind reshapes both ends:
//
//   - rust.dag carries typed `Declaration` field references via
//     identifier and dotted-path values resolved at lower time.
//   - emit_rust.rs builds three typed indexes keyed by
//     `DeclarationId` and tuples thereof. Lookups read declaration
//     ids straight off ports / nodes / substrate markers; zero
//     name strings cross the substrate/emitter boundary.
//
// The end-to-end success criterion is unchanged:
//   compile_to_dag("let x: Int = 1 + 2") → emit_rust → rustc → "3"
//
// Scope at PR-B (unchanged):
//   - Value literals (Int, Bool, String)
//   - Arithmetic + comparison operators on Int
//   - if/else branches (as Rust if-expressions)
//   - Top-level value Binds (as let statements)
//   - Outer fn main wrapper
//
// Out of scope (follow-up work, tracked in DOWNSTREAM_REQUIREMENTS):
//   - User-defined functions (Bind with non-empty params)
//   - Loops
//   - TransformTarget::Callable dispatch
//   - Record / enum construction
//
// Template placeholders the substitution engine recognizes (see
// dsl/extdeps/languages/rust.dag for the authoritative list):
//   %N  Bind name               %C  branch condition
//   %T  Rust type name          %H  then-arm body
//   %V  bind value expression   %E  else-arm body
//   %B  list of let statements  %Q  literal double-quote
//   %X  final bind's name

use std::collections::HashMap;

use crate::dag::{
    AtomPayload, Behavior, BranchNode, BranchPattern, Dag, DeclarationId, Field, FieldValue,
    LiteralBits, Path, PortId, TransformNode, TransformTarget, TypeConnective, ValueBody,
    ValueNode,
};
use crate::operators::OperatorKind;

/// Errors the Rust emitter surfaces when the DAG reaches a shape it
/// cannot render under the PR-B scope. Each variant names a specific
/// structural cause — no catch-all `Unknown` — so consumers can
/// classify the failure against `dsl/extdeps/languages/rust.dag`'s
/// coverage.
///
/// **Dissolution receipt — 🟢 TERMINAL.** Eight variants, each
/// classifying a structurally distinct failure mode at a different
/// boundary in the emitter pipeline. The variants partition into
/// three categories:
///
///   1. **Realization-table gaps** (`MissingTypeRealization`,
///      `MissingOperatorRealization`, `MissingBehaviorRealization`):
///      the declaration the DAG references has no matching
///      realization in `dsl/extdeps/languages/rust.dag`. Each
///      payload is a typed `DeclarationId` (no string keys) so the
///      caller can pinpoint which declaration is uncovered.
///
///   2. **Substrate-side bugs** (`MissingSubstrateMarker`,
///      `UntypedPort`, `UnresolvedBranchPattern`): the substrate
///      handed the emitter a state inference should have driven
///      to a terminal form. Reaching any of these is a bug in
///      bootstrap, infer, or lowering — not a target-language
///      coverage issue.
///
///   3. **Out-of-scope DAG shapes** (`UnsupportedBehavior`,
///      `NonBooleanBranch`): the DAG carries a structurally valid
///      shape that PR-B's emit scope doesn't cover yet (Loop,
///      Callable, non-Bool branches). Each is a follow-up boundary,
///      not a substrate gap.
///
/// 4-pattern check:
/// - **Pattern 1 (fact placement)**: fails. Each variant has a
///   structurally distinct payload: typed `DeclarationId` values
///   for realization-table gaps, a typed `SubstrateMarkerRole`
///   tag for marker absence, a `PortId` for untyped ports, a
///   string-named variant for unresolved branch patterns. Each
///   payload type lives at a different boundary.
/// - **Pattern 2 (variant-is-data)**: fails. Different payload
///   types per variant; no unified record shape.
/// - **Pattern 3 (algebraic form)**: fails. The eight variants do
///   not factor into a smaller algebra — the three categories
///   above are descriptive groupings, not algebraic dimensions.
/// - **Pattern 4 (dimensional)**: fails. No shared coordinate
///   space across the eight failure modes.
///
/// Verdict: **🟢 TERMINAL** at PR-B-unwind scope. Future emit
/// extensions (Callable dispatch, Loop emission, multi-target
/// emission shared across emit_rust/emit_go/emit_python) may add
/// new variants, each with its own substrate-extension audit per
/// `M1_DESIGN.md` §8.10. The three categories above are stable;
/// new variants slot into the appropriate one.
///
/// **`UnsupportedBehavior(String)` payload note.** The string
/// payload is a human-readable description of which shape was
/// hit, not a dispatch key. Callers do not match on the string;
/// they match on the variant tag and treat the string as
/// diagnostic detail. The 🟢 verdict above accounts for it as
/// "category 3 — out-of-scope shape" rather than as a string-
/// dispatch axis.
#[derive(Debug, Clone)]
pub enum EmitError {
    /// No `TypeRealization` was declared in rust.dag for the given
    /// type declaration. Add a `data rust_*: TypeRealization` entry
    /// targeting this declaration to close the gap.
    MissingTypeRealization { target: DeclarationId },
    /// No `OperatorRealization` was declared in rust.dag for the
    /// given (operand_type, algebra_field) pair.
    MissingOperatorRealization {
        target: DeclarationId,
        op: DeclarationId,
    },
    /// No `BehaviorRealization` was declared in rust.dag for the
    /// given substrate marker (Bind / Branch / Main).
    MissingBehaviorRealization { marker: DeclarationId },
    /// A required substrate marker is absent from `dsl/std/v3_l1.dag`
    /// — bootstrap couldn't populate the typed handle and the
    /// emitter has nothing to dispatch on. The variant identifies
    /// which marker by enum tag (not by string), keeping the error
    /// specific to substrate, not target-language, problems.
    MissingSubstrateMarker(SubstrateMarkerRole),
    /// A port has no resolved `TypeShape`, so its primitive
    /// declaration can't be looked up in the type realization
    /// index. Inference should have driven every port to Resolved
    /// before emit runs; reaching this arm is a bug.
    UntypedPort(PortId),
    /// The DAG carries a behavior variant PR-B doesn't render yet
    /// (Loop, user-function Bind, TransformTarget::Callable, etc.).
    UnsupportedBehavior(String),
    /// A Branch arm's pattern stayed `UnresolvedVariant` past
    /// inference — either inference didn't run or the scrutinee's
    /// Disj has no matching variant.
    UnresolvedBranchPattern { variant_name: String },
    /// A Branch's scrutinee resolved to a Disj that isn't the v3
    /// `Classical` (Bool) sum — PR-B only emits boolean branches.
    /// Carries the scrutinee's resolved variant ids so callers can
    /// inspect what the substrate handed them.
    NonBooleanBranch { variant_ids: Vec<DeclarationId> },
}

/// Typed tag identifying which substrate marker is missing in a
/// `MissingSubstrateMarker` error. Replaces the earlier `role:
/// &'static str` payload so that no name string crosses the
/// substrate/emitter boundary even in error reporting. Display
/// formatting is the consumer's job; this tag is dispatch data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubstrateMarkerRole {
    Bind,
    Branch,
    Main,
}

/// Three typed realization indexes built once per `emit_rust` call
/// from rust.dag's data declarations. Each index keyed by the
/// `DeclarationId`s that the substrate already carries — no name
/// strings.
struct RealizationIndexes {
    /// `target_decl_id → carrier`. Built from `data rust_*:
    /// TypeRealization` items in rust.dag. Used when emitting
    /// `let x: <carrier> = ...` to resolve a port's type.
    types: HashMap<DeclarationId, String>,
    /// `(operand_type_decl, algebra_field_decl) → carrier`. Built
    /// from `data rust_*: OperatorRealization` items in rust.dag.
    /// Used when emitting a `Transform { target: Operator(_), .. }`.
    operators: HashMap<(DeclarationId, DeclarationId), String>,
    /// `behavior_marker_decl → carrier`. Built from `data rust_*:
    /// BehaviorRealization` items in rust.dag. Used when emitting
    /// the substrate behaviors (let / if-else / main wrapper). The
    /// key declaration ids come from `dsl/std/v3_l1.dag` markers
    /// cached in `Dag::substrate_markers` — every dispatch site
    /// reads those typed handles instead of looking up by name.
    behaviors: HashMap<DeclarationId, String>,
}

impl RealizationIndexes {
    fn build(dag: &Dag) -> Self {
        let type_meta = dag.declaration_by_name("TypeRealization").map(|d| d.id);
        let op_meta = dag.declaration_by_name("OperatorRealization").map(|d| d.id);
        let behavior_meta = dag.declaration_by_name("BehaviorRealization").map(|d| d.id);

        let mut types: HashMap<DeclarationId, String> = HashMap::new();
        let mut operators: HashMap<(DeclarationId, DeclarationId), String> = HashMap::new();
        let mut behaviors: HashMap<DeclarationId, String> = HashMap::new();

        for decl in dag.declarations() {
            let Some(meta_tag) = decl.meta_tag else {
                continue;
            };
            let Some(ValueBody::Structural { fields }) = &decl.value_body else {
                continue;
            };

            // Common field readers — pull the typed values via
            // the structural-field accessors. A missing or wrong-
            // shaped field silently skips the declaration; the
            // bootstrap's inhabitance check would have surfaced a
            // diagnostic earlier in that case, so we treat it as
            // already-handled here.
            let target = field_decl_ref(fields, "target");
            let carrier = field_string(fields, "carrier");

            if Some(meta_tag) == type_meta {
                if let (Some(t), Some(c)) = (target, carrier) {
                    types.insert(t, c);
                }
            } else if Some(meta_tag) == op_meta {
                let op = field_decl_ref(fields, "op");
                if let (Some(t), Some(o), Some(c)) = (target, op, carrier) {
                    operators.insert((t, o), c);
                }
            } else if Some(meta_tag) == behavior_meta {
                if let (Some(t), Some(c)) = (target, carrier) {
                    behaviors.insert(t, c);
                }
            }
        }

        Self {
            types,
            operators,
            behaviors,
        }
    }
}

/// Pull a typed declaration reference out of a structural field
/// list by label. Used for `target` and `op` fields whose declared
/// type was the `Declaration` sentinel and which therefore lower to
/// `FieldValue::Reference`.
fn field_decl_ref(
    fields: &[(String, FieldValue)],
    label: &str,
) -> Option<DeclarationId> {
    fields.iter().find(|(l, _)| l == label).and_then(|(_, v)| {
        if let FieldValue::Reference(id) = v {
            Some(*id)
        } else {
            None
        }
    })
}

/// Pull a string literal out of a structural field list by label.
/// Used for `carrier` field whose declared type was `String` and
/// which therefore lowers to `FieldValue::Literal(LiteralBits::String)`.
fn field_string(fields: &[(String, FieldValue)], label: &str) -> Option<String> {
    fields.iter().find(|(l, _)| l == label).and_then(|(_, v)| {
        if let FieldValue::Literal(LiteralBits::String(s)) = v {
            Some(s.clone())
        } else {
            None
        }
    })
}

pub fn emit_rust(dag: &Dag) -> Result<String, EmitError> {
    let indexes = RealizationIndexes::build(dag);

    // Resolve the substrate markers we need ONCE up front. Each
    // marker is a typed `DeclarationId` cached at bootstrap end
    // from `dsl/std/v3_l1.dag`; if any is missing, the file
    // failed to load and emit can't proceed. Rendering downstream
    // uses the bound handles, never a name string.
    let bind_marker = dag
        .bind_marker()
        .ok_or(EmitError::MissingSubstrateMarker(SubstrateMarkerRole::Bind))?;
    let branch_marker = dag
        .branch_marker()
        .ok_or(EmitError::MissingSubstrateMarker(SubstrateMarkerRole::Branch))?;
    let main_marker = dag
        .main_marker()
        .ok_or(EmitError::MissingSubstrateMarker(SubstrateMarkerRole::Main))?;

    let top_level_binds: Vec<&crate::dag::BindNode> = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .filter(|b| b.params.is_empty())
        .collect();

    if top_level_binds.is_empty() {
        return Err(EmitError::UnsupportedBehavior(
            "emit_rust requires at least one top-level value Bind".to_string(),
        ));
    }

    // Build the port→bind-name index. When `render_port` recurses
    // into a sub-expression and lands on a port that an earlier
    // top-level Bind already named, it uses the name instead of
    // re-rendering the sub-DAG. This is the structural difference
    // between "the value" and "the named binding pointing at the
    // value" — the substrate stores both pieces and the emitter
    // chooses based on whether the consumer crossed a Bind boundary
    // upstream. Top-level value rendering uses
    // `render_top_level_value` which intentionally bypasses the
    // index for its own bind's value (otherwise every let statement
    // would render as `let x: i64 = x;`).
    let mut bound_names: HashMap<PortId, String> = HashMap::new();
    for bind in &top_level_binds {
        bound_names.insert(bind.value, bind.name.clone());
    }

    let ctx = Ctx {
        dag,
        indexes: &indexes,
        branch_marker,
        bound_names: &bound_names,
    };

    let mut rendered_binds: Vec<String> = Vec::with_capacity(top_level_binds.len());
    for bind in &top_level_binds {
        let ty_name = ctx.rust_type_name_for_port(bind.value)?;
        let value_expr = ctx.render_top_level_value(bind.value)?;
        let let_template = indexes
            .behaviors
            .get(&bind_marker)
            .ok_or(EmitError::MissingBehaviorRealization {
                marker: bind_marker,
            })?;
        let rendered = let_template
            .replace("%N", &bind.name)
            .replace("%T", &ty_name)
            .replace("%V", &value_expr);
        rendered_binds.push(rendered);
    }

    let body_joined = rendered_binds.join(" ");
    let final_bind_name = top_level_binds
        .last()
        .expect("guarded above")
        .name
        .clone();

    let main_template = indexes
        .behaviors
        .get(&main_marker)
        .ok_or(EmitError::MissingBehaviorRealization {
            marker: main_marker,
        })?;
    let program = main_template
        .replace("%B", &body_joined)
        .replace("%X", &final_bind_name)
        .replace("%Q", "\"");
    Ok(program)
}

/// Bundled emission context. Carries the typed indexes, substrate
/// marker handles, and bound-name index through the recursive
/// render walk. Replaces the pre-unwind multi-arg threading where
/// every helper took `dag, index, bound_names, ...` separately.
///
/// `branch_marker` is the only marker the recursive walk reads
/// (Bind and Main are looked up at the top-level emit loop, not
/// recursively). Keeping the others off the struct trims the
/// borrow.
struct Ctx<'a> {
    dag: &'a Dag,
    indexes: &'a RealizationIndexes,
    branch_marker: DeclarationId,
    bound_names: &'a HashMap<PortId, String>,
}

impl<'a> Ctx<'a> {
    /// Render the sub-expression rooted at `port` into a Rust
    /// expression string. First checks `bound_names` — if the port
    /// already corresponds to a top-level let binding, it renders
    /// as the name (e.g. `b` reuses `a` by name, not by re-inlining
    /// `a`'s value expression).
    fn render_port(&self, port: PortId) -> Result<String, EmitError> {
        if let Some(name) = self.bound_names.get(&port) {
            return Ok(name.clone());
        }
        self.dispatch_producer(port)
    }

    /// Render the value for a top-level let binding. Bypasses
    /// `bound_names` for `port` itself (otherwise every let would
    /// render as `let x: i64 = x;`); recursive sub-walks still use
    /// `render_port` and DO consult `bound_names`.
    fn render_top_level_value(&self, port: PortId) -> Result<String, EmitError> {
        self.dispatch_producer(port)
    }

    fn dispatch_producer(&self, port: PortId) -> Result<String, EmitError> {
        let Some(node_id) = self.dag.port(port).produced_by else {
            return Err(EmitError::UnsupportedBehavior(
                "render reached a port with no producer (parameter?)".to_string(),
            ));
        };
        match self.dag.node(node_id) {
            Behavior::Value(v) => Ok(render_value(v)),
            Behavior::Transform(t) => self.render_transform(t),
            Behavior::Branch(b) => self.render_branch(b),
            Behavior::Loop(_) => Err(EmitError::UnsupportedBehavior(
                "Loop behavior is not yet supported by emit_rust".to_string(),
            )),
            Behavior::Bind(b) => Ok(b.name.clone()),
        }
    }

    fn render_transform(&self, t: &TransformNode) -> Result<String, EmitError> {
        match t.target {
            TransformTarget::Operator(op) => self.render_operator(t, op),
            TransformTarget::Callable(_) => Err(EmitError::UnsupportedBehavior(
                "TransformTarget::Callable (user function call) is not yet supported by emit_rust"
                    .to_string(),
            )),
        }
    }

    fn render_operator(
        &self,
        t: &TransformNode,
        op: OperatorKind,
    ) -> Result<String, EmitError> {
        if t.inputs.len() != 2 {
            return Err(EmitError::UnsupportedBehavior(format!(
                "operator {:?} arity {} is not supported; only binary operators",
                op,
                t.inputs.len()
            )));
        }
        // Resolve the operand type's declaration id by walking the
        // input port's TypeShape through aliases / instantiations.
        let operand_type_id = primitive_type_id_for_port(self.dag, t.inputs[0])?;
        // Resolve the algebra field's declaration id by walking
        // the operand type's algebra chain. The OperatorKind-to-
        // field-name lookup inside the helper is the SAME bridge
        // that infer.rs already uses to dispatch operator
        // signatures (see `infer::resolve_operator_arrow`); both
        // sides agree because they read the same algebra field
        // from the substrate.
        let op_decl_id = algebra_field_for_operator(self.dag, operand_type_id, op)?;
        let carrier =
            self.indexes
                .operators
                .get(&(operand_type_id, op_decl_id))
                .ok_or(EmitError::MissingOperatorRealization {
                    target: operand_type_id,
                    op: op_decl_id,
                })?
                .clone();
        let lhs = self.render_port(t.inputs[0])?;
        let rhs = self.render_port(t.inputs[1])?;
        Ok(format!("({} {} {})", lhs, carrier, rhs))
    }

    fn render_branch(&self, b: &BranchNode) -> Result<String, EmitError> {
        let (then_path, else_path) = self.split_bool_paths(b)?;
        let cond = self.render_port(b.input)?;
        let then_expr = self.render_port(then_path.output)?;
        let else_expr = self.render_port(else_path.output)?;
        let template = self
            .indexes
            .behaviors
            .get(&self.branch_marker)
            .ok_or(EmitError::MissingBehaviorRealization {
                marker: self.branch_marker,
            })?;
        Ok(template
            .replace("%C", &cond)
            .replace("%H", &then_expr)
            .replace("%E", &else_expr))
    }

    /// Sort a Branch's paths into (then, else) for if/else emission.
    /// Walks the scrutinee's port type to its Disj children, finds
    /// the True/False variants (resolved against `Classical` / Bool)
    /// by structural position, and matches each path's
    /// ResolvedVariant declaration id against them. Zero name
    /// strings — the True/False distinction comes from the
    /// scrutinee's Disj order, which is itself a fact of std/logic.dag.
    fn split_bool_paths<'p>(
        &self,
        b: &'p BranchNode,
    ) -> Result<(&'p Path, &'p Path), EmitError> {
        // The scrutinee's type tells us which Disj we're branching
        // on. For `if cond then ... else ...`, that's `Classical`
        // (Bool) and its variants are the True/False markers.
        let scrutinee_type_id = primitive_type_id_for_port(self.dag, b.input)?;
        let disj_id = walk_to_disj(self.dag, scrutinee_type_id).ok_or_else(|| {
            EmitError::UnsupportedBehavior(format!(
                "branch scrutinee type at {scrutinee_type_id:?} does not walk to a Disj"
            ))
        })?;
        let variants: Vec<&Field> = match &self.dag.declaration(disj_id).connective {
            TypeConnective::Disj { variants } => variants.iter().collect(),
            _ => unreachable!("walk_to_disj returned a non-Disj"),
        };
        if variants.len() != 2 {
            return Err(EmitError::NonBooleanBranch {
                variant_ids: variants.iter().map(|v| v.ty).collect(),
            });
        }
        // Convention: in std/logic.dag's Classical declaration,
        // the first variant is True and the second is False
        // (`type Classical = True | False`). The emitter uses
        // structural position, not the variant labels — same way
        // infer.rs reads patterns post-resolution.
        let true_variant_id = variants[0].ty;
        let false_variant_id = variants[1].ty;

        let mut then_path: Option<&Path> = None;
        let mut else_path: Option<&Path> = None;
        for path in &b.paths {
            let resolved_id = match &path.pattern {
                BranchPattern::ResolvedVariant(id) => *id,
                BranchPattern::UnresolvedVariant { name, .. } => {
                    return Err(EmitError::UnresolvedBranchPattern {
                        variant_name: name.clone(),
                    });
                }
            };
            if resolved_id == true_variant_id {
                then_path = Some(path);
            } else if resolved_id == false_variant_id {
                else_path = Some(path);
            } else {
                return Err(EmitError::NonBooleanBranch {
                    variant_ids: vec![resolved_id],
                });
            }
        }
        match (then_path, else_path) {
            (Some(t), Some(e)) => Ok((t, e)),
            _ => Err(EmitError::UnsupportedBehavior(
                "if/else branch must have both True and False arms".to_string(),
            )),
        }
    }

    /// Read a port's Rust type name via the `types` realization
    /// index. Walks the port's `TypeShape` through aliases /
    /// instantiations to a primitive declaration id, then looks
    /// up that id in the index. Zero name strings.
    fn rust_type_name_for_port(&self, port: PortId) -> Result<String, EmitError> {
        let primitive_id = primitive_type_id_for_port(self.dag, port)?;
        self.indexes
            .types
            .get(&primitive_id)
            .cloned()
            .ok_or(EmitError::MissingTypeRealization {
                target: primitive_id,
            })
    }
}

fn render_value(v: &ValueNode) -> String {
    match &v.data {
        LiteralBits::Int(n) => n.to_string(),
        LiteralBits::Bool(true) => "true".to_string(),
        LiteralBits::Bool(false) => "false".to_string(),
        LiteralBits::String(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
    }
}

/// Walk a port's resolved TypeShape declaration through anonymous
/// aliases (`Atom(ResolvedIdentifier)`) and instantiations
/// (`TypeConnective::Instantiation`) until it lands on the first
/// **named** declaration. Returns that declaration's id.
///
/// **Why named-declaration stop.** The realization indexes are
/// keyed by the canonical declaration ids of the named primitives
/// declared in std/ (`Int`, `Bool`, `String`, etc.). When a port's
/// `TypeShape` points at an anonymous wrapper (e.g. an
/// `Instantiation { template: Int, .. }` allocated by
/// `type_to_declaration_id` for compound types), the walk steps
/// through the wrapper to the named declaration the realization
/// references. When the port's TypeShape is a named alias like
/// `type CommitSha = String`, the walk stops at `CommitSha` —
/// callers see the alias's id directly. If the realization index
/// has no entry for the alias, the lookup fails with
/// `MissingTypeRealization` carrying the alias id, which is the
/// honest signal: the realization spec needs to declare the alias
/// (or M2+ adds an alias-walking dispatch via meta_tag chains).
///
/// At PR-B scope the walk depth is bounded to 32 to catch any
/// runaway cycles; the std/ types we actually consume bottom out
/// in 1–2 hops.
fn primitive_type_id_for_port(dag: &Dag, port: PortId) -> Result<DeclarationId, EmitError> {
    let ts = dag
        .port(port)
        .value_type()
        .ok_or(EmitError::UntypedPort(port))?;
    let mut current = ts.declaration;
    for _ in 0..32 {
        let decl = dag.declaration(current);
        if decl.name.is_some() {
            return Ok(current);
        }
        match &decl.connective {
            TypeConnective::Instantiation { template, .. } => current = *template,
            TypeConnective::Atom(AtomPayload::ResolvedIdentifier(next)) => current = *next,
            _ => return Ok(current),
        }
    }
    Err(EmitError::UnsupportedBehavior(
        "port type walk exceeded depth 32 — likely a cycle".to_string(),
    ))
}

/// Walk a declaration through aliases / instantiations to a `Disj`.
/// Returns the Disj declaration's id, or None if the chain bottoms
/// out without hitting a Disj. Mirrors `walk_to_conj_decl` in
/// `lower.rs` for symmetry.
fn walk_to_disj(dag: &Dag, start: DeclarationId) -> Option<DeclarationId> {
    let mut current = start;
    for _ in 0..32 {
        match &dag.declaration(current).connective {
            TypeConnective::Disj { .. } => return Some(current),
            TypeConnective::Instantiation { template, .. } => current = *template,
            TypeConnective::Atom(AtomPayload::ResolvedIdentifier(next)) => current = *next,
            _ => return None,
        }
    }
    None
}

/// Resolve the algebra-field declaration id for a given operand
/// type and `OperatorKind`. Walks the operand type's instantiation
/// chain to the algebra Conj (e.g. OrderedRing for Int), then finds
/// the field whose label matches the operator's algebra field name.
/// Returns the field's child declaration id, which the rust.dag
/// `op: OrderedRing.add` reference also resolves to via the
/// dotted-path lowering.
///
/// **Why this is acceptable as a thin bridge.** The
/// `OperatorKind::algebra_field_name()` lookup is the substrate's
/// existing operator → field mapping (already used by
/// `infer::resolve_operator_arrow`). It IS a name comparison, but
/// the name lives ONCE in `operators.rs` (tightly coupled to the
/// `OperatorKind` enum) and the resolved declaration id is what
/// flows downstream. The emitter doesn't repeat the comparison;
/// it asks this helper for the field id and uses it as a typed
/// index key.
fn algebra_field_for_operator(
    dag: &Dag,
    operand_type_id: DeclarationId,
    op: OperatorKind,
) -> Result<DeclarationId, EmitError> {
    // Walk the operand type to its algebra Conj. The same walk is
    // used by infer.rs's resolve_operator_arrow.
    let algebra_conj_id = walk_to_algebra_conj(dag, operand_type_id).ok_or_else(|| {
        EmitError::UnsupportedBehavior(format!(
            "operand type {operand_type_id:?} does not walk to an algebra Conj"
        ))
    })?;
    let field_label = op.algebra_field_name();
    let children = match &dag.declaration(algebra_conj_id).connective {
        TypeConnective::Conj { children } => children,
        _ => unreachable!("walk_to_algebra_conj returned a non-Conj"),
    };
    children
        .iter()
        .find(|f| f.label == field_label)
        .map(|f| f.ty)
        .ok_or_else(|| {
            EmitError::UnsupportedBehavior(format!(
                "algebra Conj {algebra_conj_id:?} has no field labeled {field_label}"
            ))
        })
}

/// Walk a declaration through aliases / instantiations until it
/// reaches a Conj (the algebra declaration). Returns the Conj's id.
fn walk_to_algebra_conj(dag: &Dag, start: DeclarationId) -> Option<DeclarationId> {
    let mut current = start;
    for _ in 0..32 {
        match &dag.declaration(current).connective {
            TypeConnective::Conj { .. } => return Some(current),
            TypeConnective::Instantiation { template, .. } => current = *template,
            TypeConnective::Atom(AtomPayload::ResolvedIdentifier(next)) => current = *next,
            _ => return None,
        }
    }
    None
}
