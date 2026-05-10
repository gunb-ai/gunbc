// The v3 substrate: five L1 behaviors, Ports, a Declaration table, and the Dag container.
//
// Two coordinated substrates share one container:
//
//   - Computation substrate: five L1 behaviors (Value, Transform, Branch, Loop, Bind)
//     wiring PortIds into a DAG of runtime operations. M0-terminal.
//   - Type substrate: a Declaration table indexed by DeclarationId, each declaration
//     carrying one of six TypeConnective variants (Atom, Conj, Disj, Arrow, Cardinality,
//     Instantiation). M1(2.5)-terminal until the convergence note in M1_DESIGN.md §1
//     reopens substrate extension.
//
// The two substrates reference each other by typed ID, not by name. Transform.target
// is a DeclarationId into the Declaration table; ArrowBody::UserDefined holds a
// BindNodeId witness back into the computation substrate. There is no name-based
// dispatch at the substrate layer — operators like `+` resolve to the `add` field of
// an inhabited algebra declaration during inference (via M1_DESIGN §8.9), not at
// parse time.
//
// Dissolution receipt — M0.3 (UPDATED at M1(2.5)):
//
//   Checkpoint C2 RESOLVED by deletion and renewed. TransformRule no longer exists;
//   Transform is Apply(DeclarationId): one shape that covers operators, primitives,
//   and user functions uniformly. The target is a typed edge into the Declaration
//   table. C2 is a NEGATIVE guardrail: a new Transform variant triggers the C2 stop
//   signal (feedback_checkpoint_dissolution_default, feedback_coproduct_dissolution).
//
//   Define (function definition) is a BIND whose params: Vec<PortId> is non-empty.
//   The body is a sub-DAG whose return port is the Bind's value. No TransformRule
//   variant.
//
// Dissolution receipt — M1(2.5):
//
//   The primitive substrate parallel-representation debt flagged in M0.3
//   (LiteralValue, FunctionRef { name: String }, primitive_signature lookup table,
//   Dag.signatures HashMap) dissolves here. Primitives are now Declarations in the
//   Declaration table, reached by DeclarationId from Transform.target. Operators
//   dispatch via inhabitance walks, not string matching. See M1_DESIGN.md §2, §3,
//   §5, §8.9.
//
// Structural refactor — M0.6 (preserved):
//
//   Port.state is a three-state PortState enum, not Option<TypeShape>. The biconditional
//     state == Unresolved  iff  diagnostics.contains(port.id())
//   holds by construction. Uninferred is the transitional state; the post-infer sweep
//   drives every port to Resolved or Unresolved before compile_to_dag returns.
//
//   SourceSpan lives on every Behavior and every Declaration structurally. Spans flow
//   forward through lowering; no side tables, no reconstruction.

use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

use crate::bootstrap::BOOTSTRAP_FIXTURE_PATH_KEYS;
use crate::diagnostics::{
    BootstrapAuthorityKey, Diagnostic, DiagnosticAttribution, DiagnosticTable, SourceSpan,
};
use crate::types::TypeShape;

mod bootstrap_std_generated {
    #![allow(unused_mut)]

    use super::*;

    include!("bootstrap_std_generated.rs");
}

mod bootstrap_generated {
    #![allow(unused_mut)]

    use super::*;

    include!("bootstrap_generated.rs");
}

mod bootstrap_generated_without_parse_surface {
    #![allow(unused_mut)]

    use super::*;

    include!("bootstrap_generated_without_parse_surface.rs");
}

mod builder;
mod effects;
mod ports;

pub use effects::{
    BranchArm, BreakingShape, CompositionVerdict, CreateCause, EffectShape, HttpMethodScalar,
    IdempotencyUnsupportedDetail, IdempotentShape, KeySource, OperationEffect,
    ParallelismUnsupportedDetail, ParallelismUnsupportedKind, WorkflowEffect,
    WorkflowIdempotencyReport, WorkflowParallelismReport,
};
pub use ports::{
    BoolPortRef, ElementRef, NonEmptyList, NonSingletonList, ParamRef, Port, TransformRef,
};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(u32);

impl NodeId {
    fn index(self) -> usize {
        self.0 as usize
    }

    pub(crate) fn raw(self) -> u32 {
        self.0
    }
}

/// Typed witness that a `NodeId` identifies a [`BindNode`].
///
/// `ArrowBody::UserDefined` means "the declaration body is this bind." Keeping the
/// witness here prevents every emitter/lens/inference consumer from revalidating the
/// same raw `NodeId -> BindNode` invariant with `as_bind().expect(...)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BindNodeId(NodeId);

impl BindNodeId {
    fn new_unchecked(id: NodeId) -> Self {
        Self(id)
    }

    pub(crate) fn from_bind_node(dag: &Dag, id: NodeId) -> Option<Self> {
        dag.node_opt(&id).and_then(Behavior::as_bind)?;
        Some(Self::new_unchecked(id))
    }

    pub fn node_id(self) -> NodeId {
        self.0
    }

    pub fn bind_opt(self, dag: &Dag) -> Option<&BindNode> {
        dag.node_opt(&self.0)?.as_bind()
    }

    pub fn bind(self, dag: &Dag) -> &BindNode {
        self.bind_opt(dag).expect("BindNodeId must point at a Bind")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PortId(u32);

impl PortId {
    pub fn raw(self) -> u32 {
        self.0
    }
}

#[cfg(test)]
impl PortId {
    pub(crate) const fn test_raw(raw: u32) -> Self {
        Self(raw)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DeclarationId(u32);

impl DeclarationId {
    fn index(self) -> usize {
        self.0 as usize
    }

    pub fn raw(self) -> u32 {
        self.0
    }
}

#[cfg(test)]
impl DeclarationId {
    pub(crate) const fn test_raw(raw: u32) -> Self {
        Self(raw)
    }
}

#[cfg(feature = "declaration-id-raw-test-witness")]
impl DeclarationId {
    /// Builds an opaque id without validating substrate indexing — **dependent test witness only**
    /// (feature **`declaration-id-raw-test-witness`**; row-count gates that never read ids).
    #[doc(hidden)]
    pub const fn declaration_id_raw_for_testing(raw: u32) -> Self {
        Self(raw)
    }
}

/// Structural reference to a lens-application section.
///
/// Authority: `src/v3/std/lens_application.dag` (`SectionRef`). Emitted lens code
/// (`lens_cost_symbolic`) references this `dag` mirror so cost-basis subjects align with
/// `EnforcedApplication.section` / design doc §2 (`DeclarationScope` / `NodeScope`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SectionRef {
    DeclarationScope {
        declaration: DeclarationId,
    },
    NodeScope {
        declaration: DeclarationId,
        node: NodeId,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClusterId(u32);

impl ClusterId {
    fn index(self) -> usize {
        self.0 as usize
    }

    pub fn raw(self) -> u32 {
        self.0
    }
}

/// `std.error_primitives` declares canonical `Result<ok, err> = Ok { value: ok } | Err { value: err }`.
/// Emitters lower it to a target-native `Result` / `struct { Ok; Err }` carrier and must
/// not also emit a second substrate `type Result`.
///
/// Suppression keys off the **resolved structural fingerprint** of that declaration
/// (name + type-parameter identities + `Ok`/`Err` payload wiring), not `span.file`
/// suffixes — so unrelated modules named `errors.dag` cannot collide, and renaming
/// the std file alone does not silently retarget suppression.
///
/// **Policy:** the fingerprint is **global**, not std-scoped: any other declaration
/// named `Result` that matches this exact shape is also suppressed (intentional — the
/// substrate owns one canonical `Result<ok, err>` carrier; a user-defined twin with
/// the same fingerprint would not emit as a separate `type Result`).
pub(crate) fn substrate_result_type_decl_suppressed_for_emit(
    dag: &Dag,
    decl: &Declaration,
) -> bool {
    if decl.name.as_deref() != Some("Result") {
        return false;
    }
    let [ok_param, err_param] = match decl.type_params.as_slice() {
        [a, b] => [*a, *b],
        _ => return false,
    };
    let ok_decl = dag.declaration(ok_param);
    let err_decl = dag.declaration(err_param);
    let ok_param_ok = matches!(
        &ok_decl.connective,
        TypeConnective::Atom(AtomPayload::TypeParam(name)) if name == "ok"
    );
    let err_param_ok = matches!(
        &err_decl.connective,
        TypeConnective::Atom(AtomPayload::TypeParam(name)) if name == "err"
    );
    if !ok_param_ok || !err_param_ok {
        return false;
    }
    let TypeConnective::Disj { variants } = &decl.connective else {
        return false;
    };
    if variants.len() != 2 {
        return false;
    }
    let Some(ok_field) = variants.iter().find(|v| v.label == "Ok") else {
        return false;
    };
    let Some(err_field) = variants.iter().find(|v| v.label == "Err") else {
        return false;
    };
    substrate_result_variant_payload_is_value_of(dag, ok_field.ty, ok_param)
        && substrate_result_variant_payload_is_value_of(dag, err_field.ty, err_param)
}

fn substrate_result_variant_payload_is_value_of(
    dag: &Dag,
    payload_ty: DeclarationId,
    type_param: DeclarationId,
) -> bool {
    let payload = dag.declaration(payload_ty);
    let TypeConnective::Conj { children } = &payload.connective else {
        return false;
    };
    children.len() == 1 && children[0].label == "value" && children[0].ty == type_param
}

/// `std.error_primitives` declares canonical `DivError = DivideByZero | Overflow`.
/// Like `Result`, emit suppression keys off the resolved structural fingerprint,
/// not the declaration source path. A declaration with this exact global shape is
/// the substrate-owned integer-division error carrier and is materialized by the
/// target division prelude when a program actually needs it.
pub(crate) fn substrate_div_error_type_decl_suppressed_for_emit(
    dag: &Dag,
    decl: &Declaration,
) -> bool {
    if decl.name.as_deref() != Some("DivError") || !decl.type_params.is_empty() {
        return false;
    }
    matches!(&decl.connective, TypeConnective::Disj { variants } if {
        variants.len() == 2
            && variants.iter().any(|variant| {
                variant.label == "DivideByZero"
                    && substrate_div_error_variant_payload_is_unit(dag, variant.ty)
            })
            && variants.iter().any(|variant| {
                variant.label == "Overflow"
                    && substrate_div_error_variant_payload_is_unit(dag, variant.ty)
            })
    })
}

fn substrate_div_error_variant_payload_is_unit(dag: &Dag, payload_ty: DeclarationId) -> bool {
    matches!(
        &dag.declaration(payload_ty).connective,
        TypeConnective::Conj { children } if children.is_empty()
    )
}

/// A type-system declaration. The unit of the type substrate.
///
/// Every named declaration (primitive, algebra, user type, type alias) lives in
/// `Dag.declarations` under a stable DeclarationId. Anonymous declarations (the
/// inner types of Cardinality bounds, Arrow inputs, etc.) also live here; only the
/// `name` field distinguishes them.
///
/// `type_params`, `phantom_params`, `meta_tag`, `specialization_parent`, `inhabits`, and
/// `value_body` are separate edges with distinct semantics:
/// - `type_params`: the canonical carrier for generic parameters declared on
///   `type Foo<T, U> { ... }` / sum / alias items. Each entry is a
///   DeclarationId whose connective is `Atom(TypeParam(name))`. Keeping type
///   params off the connective axis means `Conj.children` stays pure record
///   fields and `Disj.variants` stays pure sum alternatives — type params no
///   longer share a slot with either. Empty for most declarations.
/// - `phantom_params`: the subset of `type_params` that must be preserved for
///   type checking but do not correspond to runtime fields. Each entry also
///   names the algebra that governs closure for that phantom value. The initial
///   R2 Dimensions consumer needs only abelian-group closure, carried as a
///   typed edge to the substrate algebra declaration: matching phantom values
///   compose, mismatched values fail closed as a unit mismatch.
/// - `meta_tag`: "this Conj's shape is constrained by the linked meta-type
///   declaration." Used for value construction (records, services,
///   transports) per M1_DESIGN.md §Q0. Empty across the M1(2.5) bootstrap
///   set except for the §6.5 realization stub.
/// - `specialization_parent`: optional lowering-only back-pointer from a
///   **materialized** declaration produced by `lower::specialize_decl_for_lowering`
///   to the declaration id that was specialized (the immediate template before
///   substitution). Today only anonymous specialized `Disj` sums set this so
///   Rust emit can walk to the named template without cloning `Declaration::name`
///   (P2 / `Dag::declaration_by_name`). `None` for every other declaration.
/// - `inhabits`: "this declaration additionally satisfies the linked
///   algebra's laws." Used for secondary algebra inhabitance on declarations
///   with their own primary structure. Empty across M1(2.5).
/// - `value_body`: "this declaration is a data value of the declared type,
///   not a type alias." Distinguishes `data foo: Int = {...}` from
///   `type foo = Int` at the substrate level. `None` for type
///   declarations; `Some(ValueBody::Unparsed(span))` for data items whose
///   body source is preserved but not yet lowered to a value sub-DAG.
///   See M1(2.7) QW2 resolution in DOWNSTREAM_REQUIREMENTS.md.
#[derive(Debug, Clone)]
pub struct Declaration {
    pub id: DeclarationId,
    pub name: Option<String>,
    pub connective: TypeConnective,
    pub type_params: Vec<DeclarationId>,
    pub phantom_params: Vec<PhantomParameter>,
    pub meta_tag: Option<DeclarationId>,
    pub specialization_parent: Option<DeclarationId>,
    pub inhabits: Option<DeclarationId>,
    pub value_body: Option<ValueBody>,
    /// DB-11 (3a.3): optional refinement predicate. `None` for
    /// ordinary declarations; `Some(pred_id)` for refined parameter
    /// types where `pred_id` points at a predicate `Declaration`
    /// whose connective is `Arrow { inputs: [base], output: Bool,
    /// body: UserDefined(bind) }`. The `Bind`'s `params[0]` is the
    /// refinement's parameter slot; walking from `Bind.value` through
    /// the predicate's node sub-DAG reaches the Bool output. Two
    /// refinements are structurally equal iff their predicate
    /// expression DAGs walk equal via
    /// `infer::predicates_structurally_equal` — no interning, no SMT
    /// entailment.
    ///
    /// **Consumers.** `infer::decide_transform` consults
    /// `refinement` on the callee's parameter type declaration via
    /// `check_refinement_discharge` after structural equivalence
    /// passes; argument-side ports carry the refined declaration id
    /// through `declaration_to_port_shape` and `signature_type_shape`
    /// (which stops at refinement carriers so the alias walk doesn't
    /// strip the edge). `lower::narrow_scope_for_predicate` creates
    /// refined declarations for arm-local narrowing when an `if`
    /// cond is a single-parameter predicate.
    pub refinement: Option<DeclarationId>,
    /// Nominal-opacity carrier (T-Substrate nominal-opaque-for-Secret
    /// subset, carrier-only staging). When `Some`, the listed
    /// `permitted_accessors` are the intended sealed-accessor boundary
    /// for generic structural walks. The fail-closed walker consumer +
    /// std `Secret` marking + carry-forward through specialization are
    /// the named follow-up enforcement work; this field is staging
    /// surface only and must either gain a real walker consumer or be
    /// removed before T-Modeling Secret<T> graduation can dispatch.
    pub nominal_opacity: Option<NominalOpacity>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhantomParameter {
    pub parameter: DeclarationId,
    pub algebra: DeclarationId,
}

/// Sealed-accessor carrier (carrier-only staging). Lists the
/// `DeclarationId`s intended as the only permitted descent path into
/// a nominal-opaque declaration's interior. The fail-closed walker
/// consumer is the named follow-up enforcement work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NominalOpacity {
    pub permitted_accessors: Vec<DeclarationId>,
}

// Value-body shape for `data foo: T = ...` declarations.
//
// Dissolution ledger: `Unparsed` is the bounded scaffold (named dissolution
// trigger: M2+ parser extensions close class-5 gap #3); structural variants
// carry lowered record/scalar/list/map data bodies. Keep the enum itself in the
// generated include below so `src/v3/std/substrate.dag` remains the carrier
// authority. Map payloads are wrapped in `FieldMap` on the Rust side to
// preserve duplicate-key rejection.
include!("dag_value_body_generated.rs");

/// Ordered string-keyed structural map entries with duplicate keys rejected at
/// construction. The ordered storage is deliberate: `.dag` data maps preserve
/// authored order for deterministic bootstrap/regeneration output, while the
/// constructor enforces map key uniqueness once at the substrate boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldMap {
    entries: Vec<(String, FieldValue)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuplicateFieldMapKey {
    pub key: String,
}

impl FieldMap {
    pub fn from_entries(entries: Vec<(String, FieldValue)>) -> Result<Self, DuplicateFieldMapKey> {
        let mut seen = HashSet::new();
        for (key, _) in &entries {
            if !seen.insert(key.clone()) {
                return Err(DuplicateFieldMapKey { key: key.clone() });
            }
        }
        Ok(Self { entries })
    }

    pub fn entries(&self) -> &[(String, FieldValue)] {
        &self.entries
    }

    pub fn into_entries(self) -> Vec<(String, FieldValue)> {
        self.entries
    }
}

/// Per-field value payload inside a `ValueBody::Structural`.
/// Scalar literals, declaration references, nested records, nested
/// lists, and structural sum constructors each carry distinct
/// payload shapes, so the discriminant remains load-bearing.
///
/// **Why both variants exist.** PR-B's initial payload was
/// `LiteralBits` only, which forced `src/v3/spec/rust.dag`
/// to encode declaration identities as strings (`target_name:
/// "Int"`) — and that pushed string-dispatch back into
/// `emit_rust.rs`, undoing the M1(2.7) cleanup at the inference
/// layer one layer down. The unwind adds the `Reference` variant
/// so target spec files write `target: Int` (typed edge to the
/// `Int` declaration in std/integer.dag) and downstream consumers
/// read a `DeclarationId` directly.
///
/// Lowering recognizes when a record-literal field's declared type
/// walks to the `DeclarationRef` sentinel marker (declared in
/// `src/v3/spec/v3_l1.dag`) and accepts identifier / dotted-path
/// expressions as field values, resolving them to declaration ids.
/// For non-`DeclarationRef` field types lowering requires a literal.
///
/// 4-pattern check:
/// - Pattern 1 (fact placement): fails. Each variant has a
///   distinct payload type; the discriminant is load-bearing for
///   downstream readers (the realization index, the cost lens, etc).
/// - Pattern 2 (variant-is-data): fails. `Literal(LiteralBits)`,
///   `Reference(DeclarationId)`, `Record(Vec<...>)`,
///   `List(Vec<...>)`, `Map(FieldMap)`, and `Variant { .. }` inhabit different
///   structural spaces.
/// - Pattern 3 (algebraic form): fails. The six variants are not
///   points in one algebra; they are the minimal carrier set needed
///   for nested structural data bodies.
/// - Pattern 4 (dimensional): fails.
///
/// Verdict: terminal at the current structural-data layer. Any new
/// `FieldValue` variant still requires its own 4-pattern receipt at
/// extension time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldValue {
    /// A scalar primitive value: Int, Bool, or String. Validated
    /// at lowering time against the declared field type via the
    /// primitive cache.
    Literal(LiteralBits),
    /// A typed reference to another declaration. The
    /// `DeclarationId` was resolved at lowering time from a
    /// `SurfaceExpr::Var` or `SurfaceExpr::Path`. Used by
    /// per-target language spec files (e.g. rust.dag) to point at
    /// realization targets without going through string keys.
    Reference(DeclarationId),
    /// Nested structural record value. Used by staged spec files
    /// whose structural data bodies contain record-valued fields.
    Record(Vec<(String, FieldValue)>),
    /// Structural list value. Used by staged spec files whose
    /// structural data bodies contain list-valued fields.
    List(Vec<FieldValue>),
    /// Structural string-keyed map value. Used by staged spec files whose
    /// structural data bodies contain `Map<String, _>` fields. The carrier is
    /// validated so duplicate keys are not representable after construction.
    Map(FieldMap),
    /// Structural sum constructor with positional payload fields.
    /// The exact variant child declaration is preserved explicitly
    /// so downstream consumers can recover variant identity without
    /// string bridges.
    Variant {
        constructor: DeclarationId,
        payload: Vec<FieldValue>,
    },
}

// Terminal literal/cardinality/template/port-state mirrors are generated
// from `std/substrate.dag`; keep the substrate authority there, not here.
include!("dag_scalar_generated.rs");

/// Construct [`LiteralBits::Int`] from a host `i64` (tests, staging, internal seeds).
#[inline]
pub fn literal_bits_int(i: i64) -> LiteralBits {
    LiteralBits::Int(i.to_string())
}

/// Parse a signed decimal [`LiteralBits::Int`] payload as `i64` (evaluator / emit paths).
#[inline]
pub fn literal_decimal_i64(s: &str) -> Option<i64> {
    s.parse().ok()
}

/// Parse a nonnegative decimal [`LiteralBits::Int`] payload as `usize` (emit slot indices, …).
#[inline]
pub fn literal_decimal_usize(s: &str) -> Option<usize> {
    if s.starts_with('-') {
        return None;
    }
    s.parse().ok()
}

impl CardinalityBound {
    pub fn interval(self) -> Interval<u32> {
        match self {
            Self::Exact(value) => Interval::BoundedInterval {
                lower: value,
                width: IntervalWidth::ZeroWidth,
            },
            Self::AtMostOne => Interval::BoundedInterval {
                lower: 0,
                width: IntervalWidth::PositiveWidth(PositiveIntervalWidth::OneUnit),
            },
            Self::Unbounded => Interval::Unbounded,
        }
    }
}

mod cardinality_payload;
pub use cardinality_payload::CardinalityPayload;

/// The six-variant type substrate. Terminal at M1(2.5).
///
/// Dissolution ledger (4-pattern check per THESIS §"Structural decompression"):
/// - **Pattern 1 (fact placement)**: fails. Each variant carries a
///   structurally distinct shape (Atom is a payload enum, Conj holds a
///   labeled product, Disj a labeled coproduct, Arrow a typed function,
///   Cardinality a repetition, Instantiation a template reference).
///   Scattering them into per-variant side tables duplicates dispatch.
/// - **Pattern 2 (variant-is-data)**: fails. Different variants have
///   different payload types; a unified representation would be a tagged
///   union (which is what we already have).
/// - **Pattern 3 (algebraic form)**: passes. The six variants map to the
///   six category-theoretic forms of a type algebra (intro + AND + OR +
///   function + repetition + parametric specialization). The substrate
///   matches the thesis's type-substrate shape exactly.
/// - **Pattern 4 (dimensional)**: fails. No shared coordinate space
///   underlies all six variants.
///
/// Verdict: terminal at M1(2.5). Extension requires the C1-class stop
/// signal in M1_DESIGN.md §8.10 — all four patterns must be re-run
/// before adding a 7th variant. See also the related §8.11 Pending
/// elimination ratchet that dissolves `ArrowBody::Pending` (not a
/// TypeConnective variant) by M3.
#[derive(Debug, Clone)]
pub enum TypeConnective {
    /// Irreducible leaf. See AtomPayload.
    Atom(AtomPayload),
    /// Labeled product — logical AND. All children present together.
    Conj { children: Vec<Field> },
    /// Labeled coproduct — logical OR. Exactly one variant active.
    Disj { variants: Vec<Field> },
    /// Function type — directional flow from inputs to an output. `body` covers
    /// user sub-DAGs, extdeps-declared realizations, and the Pending bootstrap
    /// state. See M1_DESIGN.md §Q7.
    Arrow {
        inputs: Vec<DeclarationId>,
        output: DeclarationId,
        body: ArrowBody,
    },
    /// Repetition over an element type with a bound. Unifies v2's Required/Optional
    /// with list cardinality.
    Cardinality(CardinalityPayload),
    /// Specialization of a parameterized template with concrete template arguments.
    /// For pure aliases like `Int = OrderedRing<Word64>`, Int's connective IS
    /// Instantiation directly — inhabitance collapses into this form. See
    /// M1_DESIGN.md §Q0, §Q1.
    Instantiation {
        template: DeclarationId,
        arguments: Vec<TemplateArgument>,
    },
}

/// Maximum alias / resolution hops when peeling before cardinality idempotence.
const CARDINALITY_IDEMPOTENCE_PEEL_DEPTH: usize = 64;

/// Peel `ResolvedBy*` atoms and zero-argument `Instantiation` aliases (surface
/// `type Alias = Target` lowering) to the denoted declaration. Stops at the
/// first non-transparent connective or at [`CARDINALITY_IDEMPOTENCE_PEEL_DEPTH`].
fn peel_alias_for_cardinality_idempotence(
    dag: &Dag,
    current: DeclarationId,
    depth: usize,
) -> DeclarationId {
    if depth >= CARDINALITY_IDEMPOTENCE_PEEL_DEPTH {
        return current;
    }
    match &dag.declaration(current).connective {
        TypeConnective::Atom(AtomPayload::ResolvedByStructure(next))
        | TypeConnective::Atom(AtomPayload::ResolvedByName(next)) => {
            peel_alias_for_cardinality_idempotence(dag, *next, depth + 1)
        }
        TypeConnective::Instantiation {
            template,
            arguments,
        } if arguments.is_empty() => {
            peel_alias_for_cardinality_idempotence(dag, *template, depth + 1)
        }
        _ => current,
    }
}

/// `AtMostOne ∧ AtMostOne` idempotence: nested optional uses the inner declaration.
///
/// Single rule authority for T-ImpossibleBugs nested-optional flatten. Applies
/// after peeling zero-arg instantiation / resolved-atom indirection so
/// `type Opt = Int?; type Alias = Opt; … Alias?` collapses like `Opt?`.
pub(crate) fn cardinality_idempotent_target(
    dag: &Dag,
    element: DeclarationId,
    bound: CardinalityBound,
) -> Option<DeclarationId> {
    if bound != CardinalityBound::AtMostOne {
        return None;
    }
    let subject = peel_alias_for_cardinality_idempotence(dag, element, 0);
    match &dag.declaration(subject).connective {
        TypeConnective::Cardinality(p) if p.bound() == CardinalityBound::AtMostOne => Some(subject),
        _ => None,
    }
}

/// `TypeConnective::Cardinality` for contexts that do not allocate a declaration
/// (e.g. type-alias connective). Reuses the same
/// [`cardinality_idempotent_target`] / nested-`AtMostOne` rule as
/// [`Dag::alloc_cardinality_decl`]: if `element` is already
/// `Cardinality(AtMostOne, …)` with matching `bound`, the existing connective is
/// returned unchanged instead of minting `Cardinality(AtMostOne, that_decl)`.
pub(crate) fn type_connective_cardinality(
    dag: &Dag,
    element: DeclarationId,
    bound: CardinalityBound,
) -> TypeConnective {
    if let Some(keep) = cardinality_idempotent_target(dag, element, bound) {
        return dag.declaration(keep).connective.clone();
    }
    TypeConnective::Cardinality(CardinalityPayload::new_unchecked_bypassing_idempotence(
        element, bound,
    ))
}

#[derive(Debug, Clone)]
pub struct Field {
    pub label: String,
    pub ty: DeclarationId,
}

impl AtomPayload {
    pub fn resolved_id(&self) -> Option<DeclarationId> {
        match self {
            Self::ResolvedByStructure(id) | Self::ResolvedByName(id) => Some(*id),
            _ => None,
        }
    }
}

/// Dissolution ledger (per M1_DESIGN.md §Q7 "ArrowBody dissolution ledger"):
/// ArrowBody is a **mixed-lifecycle coproduct**. Terminal shape is 2
/// variants (`UserDefined`, `ExternalRealization`); the two
/// scaffold variants (`Pending`, `Unparsed`) dissolve via separate
/// ratchets with distinct triggers.
///
/// 4-pattern check on the terminal pair (UserDefined, ExternalRealization):
/// - Pattern 1 (fact placement): fails. Both are Arrow-level facts with
///   different structural targets (NodeId vs DeclarationId).
/// - Pattern 2 (variant-is-data): fails. Different payload types.
/// - Pattern 3 (algebraic form): partial. Both are "realization
///   reference" but with structurally different reference types;
///   collapsing would require a sum over NodeId/DeclarationId — a
///   worse coproduct.
/// - Pattern 4 (dimensional): fails. No shared coordinate space.
///
/// Scaffold variants with their dissolution triggers:
///
/// - **`Pending`** — "body to come, transient." Two production sites
///   write Pending today, both transient:
///
///   1. **Executable-fn seeding (declarations).** `seed_function_signature`
///      writes `Pending` for `fn foo(x) -> T = body` declarations as
///      the initial substrate state. `lower_fn_item` is responsible
///      for patching every such declaration to
///      `ArrowBody::UserDefined(BindNodeId::from_bind_node(dag, bind_id))` before the Dag is frozen
///      — including on error paths (R13 fix in `lower.rs`). A
///      final `Arrow(Pending)` surviving into the Dag is
///      structurally equivalent to "body lowering missed a path,"
///      which is exactly what `lens_structural_resolution` detects.
///
///   2. **Operator fallback bridge (transient `ResolvedArrow`).**
///      `infer::resolve_operator_arrow` falls back to a synthetic
///      `(T, T) -> T` / `(T, T) -> Bool` signature with `body:
///      Pending` when the structural algebra walk can't find an
///      algebra Conj (Bool, collection-level algebras — class-5
///      gaps #1 and #2 in DOWNSTREAM_REQUIREMENTS.md). This shape
///      lives in inference-only `ResolvedArrow` values, never in
///      `Dag.declarations`, so the lens cannot see it. Dissolves
///      when those class-5 gaps close.
///
///   **History.** Earlier rounds wrote Pending at four additional
///   sites (anonymous nested Arrow type expressions, type-alias
///   targets, data-item type annotations, variant constructor
///   synthesis), all of which represented "no body by construction"
///   rather than "body to come." Those sites migrated to `NoBody`
///   in the (a)/broader-migration work — see the per-site comments
///   at `lower.rs` (`type_to_connective`, anonymous nested Arrow synthesis),
///   and `infer.rs:1893`
///   (`resolve_direct_target_signature`).
///
/// - **`NoBody`** — terminal "no body by construction." Used wherever
///   an Arrow signature exists but the declaration carries no
///   executable body and never will. Production sites:
///
///   1. **Type aliases / data items** with Arrow targets
///      (`type Callback = fn(Int) -> Int`, `data x: fn(Int) -> Int = ...`)
///      via `lower_type_alias` / `lower_data_item` →
///      `type_to_connective`.
///
///   2. **Anonymous nested Arrow declarations** synthesized inside
///      larger type expressions (`fn handler(cb: fn(Int) -> Int)`,
///      bootstrap algebra arrows like `add: fn(T, T) -> T`) via
///      `type_to_declaration_id`'s Arrow arm.
///
///   3. **Variant constructor signatures** synthesized by
///      `infer::resolve_direct_target_signature` for `Variant(payload)`
///      direct-construction calls (transient `ResolvedArrow`, never
///      stored in `Dag.declarations`).
///
///   `decide_transform` treats `NoBody` identically to `Pending`
///   (signature inhabitance, body-walking skipped). The variant
///   distinction exists to make the substrate predicate
///   "`Arrow(Pending)` in the final Dag = R13-class regression"
///   structurally exact, with no `name`-based proxy in the lens.
///
///   No dissolution trigger — terminal at the substrate level.
///
/// - **`Unparsed`** — surface-grammar lag (**case 1**) and **case 2c** the
///   `pipeline.dag` **`compile`** orchestrator (ordering authority). Used at
///   M1(2.7) for block-bodied `fn foo(x) -> T { body }` declarations in std/
///   files where the body contains match/pipe/lambda/ etc. **Parser note
///   (Prereq-2 / #1248):** authority `.dag` sources still surface these as
///   `SurfaceItem::FnExternalBody` (brace-skip) so bootstrap snapshots stay
///   stable; user `.v3` modules may surface `SurfaceItem::Fn` for single-
///   expression brace bodies instead. **`pipeline.dag`
///   per-stage fns (case 2a)** parse as `FnExternalBody` → `Unparsed`, then
///   bootstrap rewrites those Arrow bodies to `ExternalRealization` before
///   inference — so `Unparsed` does not persist for those stages in a
///   bootstrapped DAG. **`fn compile` (case 2c)** has no `PipelineStageBinding`:
///   **`Unparsed` persists** on its Arrow body. Pipeline **runtime** ordering
///   authority is the declaration order of the `PipelineStageBinding` records
///   in the Dag — `ordered_pipeline_stages` reads that structural order only.
///   The human-readable `compile` body remains a second **authored** surface;
///   fail-closed drift detection between it and the bindings is **suspended**
///   (PR #1171 disposition, 2026-04-29): the lowered Dag does not carry an
///   ordered stage list inside `compile`, and neither compile-time embed nor
///   runtime source-file read satisfies R3 `bridge_include_str_side_channels_retired`
///   for that check — see `pipeline_authority::ordered_pipeline_stages`. Until
///   derivation, review/regen discipline keeps the two carriers aligned. PR #637
///   narrowed the prior body-span-as-authority shape; scheduled debt remains
///   (see `docs/history/roadmap-scheduled-deletions.md` case 2c). The signature
///   still flows forward through the
///   declaration table so callers can type-check against it, and the body
///   source span is preserved so M2+ parser extensions can reach in for case 1.
///   **User-range boundary:** `reject_user_unparsed_scaffolds` in
///   `src/v3/compiler/src/lower.rs` fails-closed any user-range
///   declaration carrying this variant (R14 + M1(2.8) Scaffold
///   Boundaries invariant). Bootstrap-range declarations stay
///   tolerated. **Case 1** dissolution: M2 surface-grammar extension — when
///   every relevant std/ block body becomes parseable, case-1 `Unparsed` is
///   removed via a reverse substrate-extension PR. **Case 2c** is not waiting on
///   that grammar milestone.
///
/// Verdict: terminal form is 3 variants (`UserDefined`,
/// `ExternalRealization`, `NoBody`). The 5-variant shape is a
/// transition state: `Pending` and **case-1** `Unparsed` are scaffolds with
/// named dissolution (M3 / M2 grammar). **`Unparsed` on `pipeline.dag`'s
/// `compile` (DB-16 case 2c)** is different — a **bootstrap-range carrier
/// in bridge shape**: body span is no longer the ordering authority
/// (structural binding order is), but two authored carriers still exist until
/// derivation; runtime reconcile is suspended pending a structural witness, so
/// 2c remains scheduled debt with its own dissolution trigger (derivation, not the M2 parser
/// milestone). User-range
/// `Unparsed` stays gated (R14).
#[derive(Debug, Clone)]
pub enum ArrowBody {
    /// User-defined function. BindNodeId is the root bind of a sub-DAG of L1
    /// behavior nodes in `Dag.nodes`. Inference walks the sub-DAG and checks
    /// the body against the declared inputs/output.
    UserDefined(BindNodeId),
    /// Primitive whose realization is declared in an extdeps language spec.
    /// DeclarationId points at the realization declaration via a typed edge;
    /// inference verifies signature compatibility.
    ExternalRealization(DeclarationId),
    /// Transient "body to come." Signature type-checks via inhabitance;
    /// body-walking is skipped. Two production sites write `Pending`:
    ///
    /// 1. `seed_function_signature` for `fn foo(x) -> T = body`
    ///    declarations — the initial substrate state before
    ///    `lower_fn_item` patches the body into `UserDefined(bind_id)`.
    ///    A final `Arrow(Pending)` surviving into the Dag is
    ///    structurally a missed body-patching path: the R13-class
    ///    regression `lens_structural_resolution` watches for.
    /// 2. `infer::resolve_operator_arrow` for transient `ResolvedArrow`
    ///    fallback signatures (class-5 gap; never stored in
    ///    `Dag.declarations`, so the lens cannot see them).
    ///
    /// All "no body by construction" sites that earlier wrote `Pending`
    /// (anonymous nested Arrows, type aliases, data items, variant
    /// constructor synthesis) now write `NoBody` instead.
    Pending,
    /// Terminal "no body by construction." The Arrow signature exists but
    /// the declaration carries no executable body and never will. Used at
    /// every "Arrow-as-data" production site:
    ///
    /// - `lower_type_alias` / `lower_data_item` → `type_to_connective`
    ///   for named type aliases (`type Callback = fn(Int) -> Int`) and
    ///   data items.
    /// - `type_to_declaration_id` for anonymous nested Arrow
    ///   declarations inside larger type expressions (parameter types,
    ///   field types, bootstrap algebra arrows).
    /// - `infer::resolve_direct_target_signature` for variant
    ///   constructor `ResolvedArrow` synthesis (transient).
    ///
    /// `decide_transform` treats `NoBody` identically to `Pending` at
    /// dispatch time (signature inhabitance, body-walking skipped). The
    /// variant distinction exists so `lens_structural_resolution` can
    /// match `Arrow(Pending)` directly as the structural fact for
    /// "executable-fn body patching missed a path."
    NoBody,
    /// Surface-grammar scaffold. The arrow's signature is resolved and
    /// callers can type-check against it, but the body source is not
    /// yet parseable under the M1(2.7) surface grammar. Used by
    /// block-bodied `fn` declarations in std/ files whose bodies
    /// contain match/pipe/lambda/etc. The `SourceSpan` points at the
    /// unparsed body range so M2+ can complete the lowering.
    /// Dissolves when the surface grammar adopts those forms.
    Unparsed(SourceSpan),
}

// Port, port-reference carriers, and the non-trivial-arity list helpers
// live in `dag/ports.rs`. See re-exports at the top of this module.

// 🟡 SCAFFOLD — Rust execution mirror for `src/v3/std/termination.dag`.
//
// The `.dag` declarations are the carrier authority, but std block bodies
// still lower as `ArrowBody::Unparsed`, so the lattice helpers below are the
// temporary executable bridge. Dissolution trigger: when std block bodies lower
// and can be evaluated from `.dag`, replace these helper bodies with calls into
// the evaluated `.dag` authority or remove them with the first real consumer.
// `m2_substrate_inhabitance_test` pins the carrier shape, body-span staging
// contract, and current Rust mirror behavior until that trigger fires.

/// 🟢 TERMINAL at termination-proof scope.
///
/// Rust mirror of the durable evidence lattice declared in
/// `src/v3/std/termination.dag`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DescentEvidence {
    Strict,
    NonIncreasing,
    DescentUnknown,
}

/// 🟡 SCAFFOLD.
///
/// Ranking dimensions are durable, but the `String` parameter bridge dissolves
/// once function parameters can be referenced structurally.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RankingDimension {
    TreeSize { param: String },
    ListLength { param: String },
    ArithmeticValue { param: String },
    TokenPosition { param: String },
    SetCardinality { param: String },
}

/// 🟢 TERMINAL at descent-witness scope.
///
/// Structural positive amount used so zero/negative shrink witnesses are not
/// representable in proof carriers.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PositiveDescentAmount {
    OneStep,
    AdditionalStep {
        previous: Box<PositiveDescentAmount>,
    },
}

/// Integer ≥ 2 — proportional-divisor witnesses for divide-and-conquer descent.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ProportionalDivisor {
    DivideByTwo,
    StrictlyLarger { inner: Box<ProportionalDivisor> },
}

pub fn positive_descent_count(steps: &PositiveDescentAmount) -> i64 {
    match steps {
        PositiveDescentAmount::OneStep => 1,
        PositiveDescentAmount::AdditionalStep { previous } => {
            1 + positive_descent_count(previous.as_ref())
        }
    }
}

pub fn proportional_divisor_to_int(d: &ProportionalDivisor) -> i64 {
    match d {
        ProportionalDivisor::DivideByTwo => 2,
        ProportionalDivisor::StrictlyLarger { inner } => {
            1 + proportional_divisor_to_int(inner.as_ref())
        }
    }
}

/// Maximum Peano links materialized from a single `i64` literal (M9 / P4).
///
/// Must stay numerically aligned with `dsl/std/termination.dag`
/// `peano_literal_materialization_cap()` (P2 single authority). Larger requests fail closed with
/// [`None`] instead of deep recursive materialization.
pub const MAX_PEANO_MATERIALIZATION: i64 = 256;

/// Builds a Peano witness with **iterative** construction (no deep recursion).
/// Returns [`None`] when `k` is out of range or exceeds [`MAX_PEANO_MATERIALIZATION`].
pub fn positive_amount_from_i64(k: i64) -> Option<PositiveDescentAmount> {
    if !(1..=MAX_PEANO_MATERIALIZATION).contains(&k) {
        return None;
    }
    let mut cur = PositiveDescentAmount::OneStep;
    for _ in 1..k {
        cur = PositiveDescentAmount::AdditionalStep {
            previous: Box::new(cur),
        };
    }
    Some(cur)
}

/// Iterative construction; `k` must be ≥ 2 and ≤ [`MAX_PEANO_MATERIALIZATION`].
pub fn proportional_divisor_from_i64(k: i64) -> Option<ProportionalDivisor> {
    if !(2..=MAX_PEANO_MATERIALIZATION).contains(&k) {
        return None;
    }
    let mut cur = ProportionalDivisor::DivideByTwo;
    for _ in 2..k {
        cur = ProportionalDivisor::StrictlyLarger {
            inner: Box::new(cur),
        };
    }
    Some(cur)
}

/// 🟡 SCAFFOLD.
///
/// The witness taxonomy is durable for E-T; String payloads dissolve when
/// accessor, operation, witness, and element references become first-class
/// substrate values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DescentSource {
    ChildAccessor { accessor: String },
    ListShrink { amount: PositiveDescentAmount },
    ArithmeticSubtractDescent { steps: PositiveDescentAmount },
    ArithmeticDivideDescent { divisor: ProportionalDivisor },
    ParserAdvance { witness: String },
    SetRemoval { element: String },
    FoldIteration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminationProof {
    pub dimensions: Vec<RankingDimension>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofEdge {
    pub caller: String,
    pub callee: String,
    pub evidence: Vec<DescentEvidence>,
}

// Continuation: Rust execution mirror for `src/v3/std/computation.dag` (Lane E-C).
// The termination lattice mirror has dissolved; computation still waits on evaluated std bodies.
// `m2_substrate_inhabitance_test::{computation_*}` pins carrier shape + lowering helpers.

/// 🟡 SCAFFOLD — `SizeBound` coproduct (`docs/modeling-discipline.md` §4).
///
/// Authority: `src/v3/std/computation.dag`. Variant taxonomy is durable; `param: String` and
/// other bootstrap bridges dissolve when size parameters become first-class substrate refs.
/// **Named trigger:** evaluated `std.computation` std block bodies (same dissolution wave as
/// the termination lattice mirror). **Ledger:** parity ratchet
/// `m2_substrate_inhabitance_test::computation_size_bound_helpers_match_dag_authority`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SizeBound {
    CollectionSize { param: String },
    ParserStreamSize { witness: String },
    WorklistDrainSize { element: String },
    TreeSize { param: String },
    ArithmeticParam { param: String },
    ExplicitCountZero,
    ExplicitCountPositive { steps: PositiveDescentAmount },
    Forever,
}

pub fn tree_size_bound(param: String) -> SizeBound {
    SizeBound::TreeSize { param }
}

/// 🟡 SCAFFOLD — `CallPattern` coproduct (`docs/modeling-discipline.md` §4).
///
/// Authority: `src/v3/std/computation.dag`. Peano shrink payloads are proof-grade (terminal
/// at witness shape); `String` slots on `CallPattern` forward into `SizeBound.param` via
/// `lower_call_pattern` (no fabricated size labels). Dissolves with structural parameter refs
/// (E-P). **Named trigger:** same as [`SizeBound`]. **Ledger:**
/// `m2_substrate_inhabitance_test::computation_lowering_rust_mirror_matches_dag_authority`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallPattern {
    ChildAccessorCall {
        accessor: String,
    },
    CollectionShrinkCall {
        amount: PositiveDescentAmount,
        collection: String,
    },
    ArithmeticSubtractCall {
        steps: PositiveDescentAmount,
        ring_param: String,
    },
    ArithmeticDivideCall {
        divisor: ProportionalDivisor,
        ring_param: String,
    },
    ParserAdvanceCall {
        witness: String,
    },
    WorklistDrainCall {
        element: String,
    },
    FoldBodyCall {
        outer_collection: String,
    },
    SameArgumentCall,
}

/// 🟢 TERMINAL — `ShrinkFactor` coproduct (`docs/modeling-discipline.md` §4).
///
/// Authority: `src/v3/std/computation.dag`. Only unit / Peano constant / Peano proportional
/// shrink — illegal rates stay unrepresentable at the carrier. **Ledger:** exercised through
/// the same `m2_substrate_inhabitance_test` computation rows as [`CallPattern`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ShrinkFactor {
    UnitShrink,
    ConstantShrink { steps: PositiveDescentAmount },
    ProportionalShrink { divisor: ProportionalDivisor },
}

/// 🟢 TERMINAL at E-I inductive-shape scope.
///
/// Runtime mirror of `std.induction::RecursionShape`: the closed partition of
/// recursive field wrappers in the .dag language. This mirrors the `.dag`
/// classification directly; adding a new recursive container type requires a
/// new authoritative `.dag` variant and matching mirror/test update.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RecursionShape {
    DirectRecursion,
    ListRecursion,
    OptionalRecursion,
    SetRecursion,
    MapValueRecursion,
}

/// Runtime mirror of `std.induction::InductiveField` for E-P provenance.
///
/// Keep this aligned with `src/v3/std/induction.dag`:
/// - `type RecursionShape`
/// - `type InductiveField`
/// - `type SubValueRelation`
///
/// 🟡 SCAFFOLD. String identity matches the current `.dag` carrier and dissolves
/// when reflected declaration/field references replace the string bridge.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InductiveField {
    pub type_name: String,
    pub variant_name: String,
    pub field_name: String,
    pub shape: RecursionShape,
    pub element_type: String,
}

/// Runtime mirror of `std.induction::SubValueRelation` for E-P provenance.
///
/// 🟡 SCAFFOLD. The `.dag` type remains the authority; this mirror exists so the
/// native DAG lens can expose per-call evidence while std block bodies still
/// lower as `ArrowBody::Unparsed`. Dissolution trigger: generated/reflected
/// lens execution can construct `std.induction::SubValueRelation` directly.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SubValueRelation {
    StrictSubValue {
        field: InductiveField,
        factor: ShrinkFactor,
    },
    IteratedSubValue {
        field: InductiveField,
    },
    ArithmeticDescent {
        param: String,
        factor: ShrinkFactor,
    },
    PreservedValue,
    SubValueUnknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallDescentEvidence {
    pub call: NodeId,
    pub caller: String,
    pub callee: String,
    pub evidence: Vec<SubValueRelation>,
}

/// 🟢 TERMINAL — `IterationPrimitive` coproduct (`docs/modeling-discipline.md` §4).
///
/// Closed `{Fold, Descend, Repeat}` behavioral alphabet (MODELING.md M9). Authority:
/// `src/v3/std/computation.dag`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IterationPrimitive {
    Fold,
    Descend,
    Repeat,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweringTarget {
    pub primitive: IterationPrimitive,
    pub bound: SizeBound,
    pub evidence: DescentEvidence,
    pub factor: Option<ShrinkFactor>,
}

pub fn lower_call_pattern(pattern: CallPattern) -> LoweringTarget {
    match pattern {
        CallPattern::ChildAccessorCall { accessor } => LoweringTarget {
            primitive: IterationPrimitive::Descend,
            bound: SizeBound::TreeSize { param: accessor },
            evidence: DescentEvidence::Strict,
            factor: None,
        },
        CallPattern::CollectionShrinkCall { amount, collection } => LoweringTarget {
            primitive: IterationPrimitive::Fold,
            bound: SizeBound::CollectionSize { param: collection },
            evidence: DescentEvidence::Strict,
            factor: Some(ShrinkFactor::ConstantShrink { steps: amount }),
        },
        CallPattern::ArithmeticSubtractCall { steps, ring_param } => LoweringTarget {
            primitive: IterationPrimitive::Repeat,
            bound: SizeBound::ArithmeticParam { param: ring_param },
            evidence: DescentEvidence::Strict,
            factor: Some(ShrinkFactor::ConstantShrink { steps }),
        },
        CallPattern::ArithmeticDivideCall {
            divisor,
            ring_param,
        } => LoweringTarget {
            primitive: IterationPrimitive::Repeat,
            bound: SizeBound::ArithmeticParam { param: ring_param },
            evidence: DescentEvidence::Strict,
            factor: Some(ShrinkFactor::ProportionalShrink { divisor }),
        },
        CallPattern::ParserAdvanceCall { witness } => LoweringTarget {
            primitive: IterationPrimitive::Fold,
            bound: SizeBound::ParserStreamSize { witness },
            evidence: DescentEvidence::Strict,
            factor: None,
        },
        CallPattern::WorklistDrainCall { element } => LoweringTarget {
            primitive: IterationPrimitive::Fold,
            bound: SizeBound::WorklistDrainSize { element },
            evidence: DescentEvidence::Strict,
            factor: None,
        },
        CallPattern::FoldBodyCall { outer_collection } => LoweringTarget {
            primitive: IterationPrimitive::Fold,
            bound: SizeBound::CollectionSize {
                param: outer_collection,
            },
            evidence: DescentEvidence::NonIncreasing,
            factor: None,
        },
        CallPattern::SameArgumentCall => LoweringTarget {
            primitive: IterationPrimitive::Repeat,
            bound: SizeBound::Forever,
            evidence: DescentEvidence::NonIncreasing,
            factor: None,
        },
    }
}

/// E-P per-call descent evidence side table.
///
/// This is option P-c from `docs/design-substrate-carrier-port-program.md`: keep
/// `TransformNode` minimal and derive a named side table from lowered call
/// structure. The producer emits one row per live callable transform and one
/// [`SubValueRelation`] per argument: [`SubValueRelation::ArithmeticDescent`],
/// structural [`SubValueRelation::StrictSubValue`] for match payload /
/// field-projection descents, [`SubValueRelation::PreservedValue`], and
/// fail-closed unknowns for edges whose descent cannot be proven from the
/// lowered graph.
pub fn per_call_descent_evidence(dag: &Dag) -> Vec<CallDescentEvidence> {
    let mut entries = Vec::new();

    for caller_decl in dag.declarations() {
        let Some(caller) = declaration_body_bind(dag, caller_decl) else {
            continue;
        };
        let caller_template = callable_target_template_for_provenance(dag, caller_decl.id);
        let owned_transforms = bind_body_transform_ids(dag, caller);

        for transform in dag.nodes().iter().filter_map(Behavior::as_transform) {
            if !owned_transforms.contains(&transform.id) {
                continue;
            }
            let TransformTarget::Callable(target_decl) = transform.target else {
                continue;
            };
            let callee_template = callable_target_template_for_provenance(dag, target_decl);
            let evidence = match (caller_template, callee_template) {
                (
                    CallableProvenance::Resolved(caller_template),
                    CallableProvenance::Resolved(callee_template),
                ) if caller_template == callee_template => transform
                    .inputs
                    .iter()
                    .enumerate()
                    .map(|(idx, arg)| classify_call_argument(dag, caller, idx, *arg))
                    .collect(),
                (CallableProvenance::Resolved(_), CallableProvenance::Resolved(_)) => {
                    vec![SubValueRelation::SubValueUnknown; transform.inputs.len()]
                }
                (CallableProvenance::Resolved(_), CallableProvenance::Unresolved)
                | (CallableProvenance::Unresolved, _) => {
                    vec![SubValueRelation::SubValueUnknown; transform.inputs.len()]
                }
            };
            entries.push(CallDescentEvidence {
                call: transform.id,
                caller: caller.name.clone(),
                callee: callee_name_for_provenance(dag, target_decl, callee_template),
                evidence,
            });
        }
    }

    entries
}

/// E-P typed CallPattern query over the per-call evidence authority.
///
/// This is the query surface named by the cost/complexity design docs. It
/// deliberately projects from [`per_call_descent_evidence`] instead of walking
/// call nodes independently, so producer broadening does not create a second
/// callable-edge authority. This first bounded broadening slice adds the
/// locally provable `PreservedValue -> SameArgumentCall` projection for
/// self-calls that pass their argument through unchanged. Existing
/// `SubValueRelation -> CallPattern` projections from `std.induction` remain
/// preserved here; multi-argument composition and lowered/lens consumers
/// remain separate E-P gates.
pub fn per_call_pattern_at(dag: &Dag, call_site: NodeId) -> Option<CallPattern> {
    let entry = per_call_descent_evidence(dag)
        .into_iter()
        .find(|entry| entry.call == call_site)?;
    call_pattern_from_relations(&entry.evidence)
}

fn call_pattern_from_relations(relations: &[SubValueRelation]) -> Option<CallPattern> {
    if let Some(pattern) = relations
        .iter()
        .filter(|relation| !matches!(relation, SubValueRelation::PreservedValue))
        .find_map(sub_value_relation_to_call_pattern)
    {
        return Some(pattern);
    }

    if relations
        .iter()
        .any(|relation| matches!(relation, SubValueRelation::SubValueUnknown))
    {
        return None;
    }

    relations
        .iter()
        .find_map(sub_value_relation_to_call_pattern)
}

pub fn sub_value_relation_to_call_pattern(relation: &SubValueRelation) -> Option<CallPattern> {
    match relation {
        SubValueRelation::ArithmeticDescent { param, factor } => match factor {
            ShrinkFactor::ConstantShrink { steps } => Some(CallPattern::ArithmeticSubtractCall {
                steps: steps.clone(),
                ring_param: param.clone(),
            }),
            ShrinkFactor::ProportionalShrink { divisor } => {
                Some(CallPattern::ArithmeticDivideCall {
                    divisor: divisor.clone(),
                    ring_param: param.clone(),
                })
            }
            ShrinkFactor::UnitShrink => Some(CallPattern::ArithmeticSubtractCall {
                steps: PositiveDescentAmount::OneStep,
                ring_param: param.clone(),
            }),
        },
        SubValueRelation::StrictSubValue { field, .. }
        | SubValueRelation::IteratedSubValue { field } => Some(CallPattern::ChildAccessorCall {
            accessor: field.field_name.clone(),
        }),
        SubValueRelation::PreservedValue => Some(CallPattern::SameArgumentCall),
        SubValueRelation::SubValueUnknown => None,
    }
}

fn declaration_body_bind<'a>(dag: &'a Dag, decl: &Declaration) -> Option<&'a BindNode> {
    // Use the lowered structural authority: function declarations point at
    // their owning body bind through `ArrowBody::UserDefined`.
    let TypeConnective::Arrow {
        body: ArrowBody::UserDefined(bind_id),
        ..
    } = &decl.connective
    else {
        return None;
    };
    Some((*bind_id).bind(dag))
}

fn bind_body_transform_ids(dag: &Dag, bind: &BindNode) -> HashSet<NodeId> {
    let mut transforms = HashSet::new();
    let mut visited_ports = HashSet::new();
    let mut visited_nodes = HashSet::new();
    collect_body_port(
        dag,
        bind.value,
        &mut visited_ports,
        &mut visited_nodes,
        &mut transforms,
    );
    transforms
}

fn collect_body_port(
    dag: &Dag,
    port: PortId,
    visited_ports: &mut HashSet<PortId>,
    visited_nodes: &mut HashSet<NodeId>,
    transforms: &mut HashSet<NodeId>,
) {
    if !visited_ports.insert(port) {
        return;
    }
    let Some(producer) = dag.port_opt(&port).and_then(|p| p.produced_by) else {
        return;
    };
    collect_body_node(dag, producer, visited_ports, visited_nodes, transforms);
}

fn collect_body_node(
    dag: &Dag,
    node: NodeId,
    visited_ports: &mut HashSet<PortId>,
    visited_nodes: &mut HashSet<NodeId>,
    transforms: &mut HashSet<NodeId>,
) {
    if !visited_nodes.insert(node) {
        return;
    }
    let Some(behavior) = dag.node_opt(&node) else {
        return;
    };
    match behavior {
        Behavior::Value(_) => {}
        Behavior::Transform(transform) => {
            transforms.insert(transform.id);
            for input in &transform.inputs {
                collect_body_port(dag, *input, visited_ports, visited_nodes, transforms);
            }
        }
        Behavior::Branch(branch) => {
            collect_body_port(dag, branch.input, visited_ports, visited_nodes, transforms);
            for path in &branch.paths {
                collect_body_node(dag, path.body, visited_ports, visited_nodes, transforms);
                collect_body_port(dag, path.output, visited_ports, visited_nodes, transforms);
            }
        }
        Behavior::Loop(loop_node) => {
            collect_body_port(
                dag,
                loop_node.source,
                visited_ports,
                visited_nodes,
                transforms,
            );
            collect_body_port(
                dag,
                loop_node.init,
                visited_ports,
                visited_nodes,
                transforms,
            );
            if let Some(count) = loop_node.bound.count_port() {
                collect_body_port(dag, count, visited_ports, visited_nodes, transforms);
            }
            collect_body_node(
                dag,
                loop_node.body,
                visited_ports,
                visited_nodes,
                transforms,
            );
        }
        Behavior::Bind(inner) => {
            // Local value binds are part of the current body graph. Function binds
            // own a separate `ArrowBody::UserDefined` body and are scanned through
            // their declaration, not through an enclosing span/body walk.
            if inner.params.is_empty() {
                collect_body_port(dag, inner.value, visited_ports, visited_nodes, transforms);
            }
        }
    }
}

/// 🟢 TERMINAL private proof-state.
///
/// Local E-P producer state: either callable template provenance was resolved
/// to a declaration, or it was not. This is not a substrate carrier; it keeps
/// the bounded instantiation peel fail-closed before emitting side-table facts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CallableProvenance {
    Resolved(DeclarationId),
    Unresolved,
}

const CALLABLE_PROVENANCE_TEMPLATE_DEPTH_LIMIT: usize = 16;

fn callable_target_template_for_provenance(
    dag: &Dag,
    mut decl: DeclarationId,
) -> CallableProvenance {
    // Bounded peel over materialized instantiations. Hitting the cap means the
    // producer cannot prove self-vs-non-self provenance, so callers emit
    // `SubValueUnknown` rather than silently dropping the call edge.
    for _ in 0..CALLABLE_PROVENANCE_TEMPLATE_DEPTH_LIMIT {
        match &dag.declaration(decl).connective {
            TypeConnective::Instantiation { template, .. } => decl = *template,
            _ => return CallableProvenance::Resolved(decl),
        }
    }
    CallableProvenance::Unresolved
}

fn callee_name_for_provenance(
    dag: &Dag,
    target: DeclarationId,
    provenance: CallableProvenance,
) -> String {
    let label = match provenance {
        CallableProvenance::Resolved(template) => template,
        CallableProvenance::Unresolved => target,
    };
    dag.declaration(label)
        .name
        .clone()
        .unwrap_or_else(|| format!("decl#{}", label.raw()))
}

fn classify_call_argument(
    dag: &Dag,
    caller: &BindNode,
    idx: usize,
    arg: PortId,
) -> SubValueRelation {
    let Some(param) = caller.params.get(idx).copied() else {
        return SubValueRelation::SubValueUnknown;
    };

    if arg == param {
        return SubValueRelation::PreservedValue;
    }

    if let Some(relation) = arithmetic_descent_relation(dag, idx, param, arg) {
        return relation;
    }

    if let Some(relation) = match_payload_descent_relation(dag, param, arg) {
        return relation;
    }

    if let Some(relation) = match_payload_field_projection_descent_relation(dag, param, arg) {
        return relation;
    }

    SubValueRelation::SubValueUnknown
}

/// Phase-1 broadening: classify recursive-call arguments that name a match-arm
/// payload binding whose scrutinee is the caller's parameter directly. The
/// surface shape this catches is the canonical positional cons-tail recursion:
///
/// ```ignore
/// type List = Nil | Cons(List)
/// fn length(xs: List) -> Int = match xs { Cons(tail) => length(tail), Nil => 0 }
/// ```
///
/// The `tail` argument port equals the `Path.binding.payload_port` of a path
/// whose enclosing `BranchNode.input` is the parameter port. Field-projection
/// patterns (`Cons { tail }`) and transitive scrutinee tracing (scrutinee
/// reached through identity-ish nodes) are deferred to later slices — same
/// fail-closed discipline as the existing arithmetic slice (left-operand only,
/// integer literal only) keeps producer broadening incrementally provable.
///
/// **Substrate-fact discipline.** `InductiveField` is consumer-facing
/// substrate provenance (cost / complexity lenses project this through
/// `sub_value_relation_to_call_pattern → CallPattern::ChildAccessorCall`).
/// The producer therefore derives every field from authoritative DAG state:
/// - `type_name` / `variant_name` from the parent-Disj lookup of the variant
///   declaration the post-infer `resolve_branch_patterns` pass installed on
///   the path (`crate::infer::resolve_branch_patterns` — `BranchPattern::ResolvedVariant`).
/// - `field_name` from the variant `Conj`'s sole `_0` positional field
///   label, **not** from the user-chosen pattern binding name. The binding
///   name is a user-scope identifier (`Cons(tail) => length(tail)` and
///   `Cons(t) => length(t)` are structurally identical descents); using it
///   would publish an unstable accessor as substrate provenance.
/// - `element_type` from the resolved name of the variant's payload type.
///
/// When any of those facts is unavailable — `UnresolvedVariant`, parent-Disj
/// lookup miss, multi-field variant Conj on a positional pattern, anonymous
/// payload type — the producer fails closed (`SubValueUnknown` upstream).
fn match_payload_descent_relation(
    dag: &Dag,
    param: PortId,
    arg: PortId,
) -> Option<SubValueRelation> {
    for branch in dag.nodes().iter().filter_map(Behavior::as_branch) {
        if !scrutinee_traces_to_param(dag, branch.input, param) {
            continue;
        }
        for path in &branch.paths {
            let Some(binding) = &path.binding else {
                continue;
            };
            if binding.payload_port != arg {
                continue;
            }
            let facts = variant_structural_facts(dag, &path.pattern)?;
            // Positional payload (`VariantWith`): variant Conj has exactly
            // one `_0`-style field. Multi-field variants are
            // `VariantFields` lowering territory (record-payload slice) and
            // are outside this helper's positional-payload contract.
            let [(field_name, payload_ty)] = facts.payload_fields.as_slice() else {
                return None;
            };
            let element_type = named_type_name(dag, *payload_ty)?;
            return Some(SubValueRelation::StrictSubValue {
                field: InductiveField {
                    type_name: facts.type_name,
                    variant_name: facts.variant_name,
                    field_name: field_name.clone(),
                    shape: RecursionShape::DirectRecursion,
                    element_type,
                },
                factor: ShrinkFactor::UnitShrink,
            });
        }
    }
    None
}

/// Phase-1 broadening Slice 2: classify recursive-call arguments produced by
/// a single `FieldProject` transform whose input is the payload port of a
/// match arm whose scrutinee is the parameter directly. This is the
/// record-payload sibling of [`match_payload_descent_relation`]'s
/// positional-payload case.
///
/// Surface shape:
///
/// ```ignore
/// type EpRec = EpLeaf | EpNode { left: EpRec }
/// fn ep_depth(t: EpRec) -> Int =
///   match t { EpNode { left: l } => ep_depth(l), EpLeaf => 0 }
/// ```
///
/// `lower.rs` lowers `EpNode { left: l }` by allocating a payload binding
/// and synthesizing a `FieldProject { field_label: "left", .. }` transform
/// whose input is the payload port (see `lower_field_projection_from_port`).
/// The recursive call's argument is therefore the projection's output port,
/// not the payload port itself — Slice 1's direct-equality check misses it.
///
/// Same fail-closed discipline + structural-fact derivation as Slice 1.
/// `field_name` here is the `FieldProject.field_label` — already structural,
/// established by the lowering, NOT a user-pattern-binding name. The label
/// must also appear in the variant's `Conj` children (validation; if not, the
/// projection isn't on this variant's payload). Cross-slice invariants
/// preserved: no new `CallPattern` variant, no `TransformNode` widening,
/// fail-closed discipline.
fn match_payload_field_projection_descent_relation(
    dag: &Dag,
    param: PortId,
    arg: PortId,
) -> Option<SubValueRelation> {
    let Behavior::Transform(transform) = dag.resolve_producer_opt(&arg)? else {
        return None;
    };
    if transform.inputs.len() != 1 {
        return None;
    }
    let projected_input = transform.inputs[0];

    for branch in dag.nodes().iter().filter_map(Behavior::as_branch) {
        if !scrutinee_traces_to_param(dag, branch.input, param) {
            continue;
        }
        for path in &branch.paths {
            let Some(binding) = &path.binding else {
                continue;
            };
            if binding.payload_port != projected_input {
                continue;
            }
            let facts = variant_structural_facts(dag, &path.pattern)?;
            let (field_name, payload_ty) = match &transform.target {
                TransformTarget::UnresolvedFieldProject { field_label } => facts
                    .payload_fields
                    .iter()
                    .find(|(label, _)| label == field_label)
                    .map(|(label, ty)| (label.clone(), *ty))?,
                TransformTarget::ResolvedFieldProject { field_label } => facts
                    .payload_fields
                    .iter()
                    .find(|(label, _)| label == field_label)
                    .map(|(label, ty)| (label.clone(), *ty))?,
                _ => return None,
            };
            let element_type = named_type_name(dag, payload_ty)?;
            return Some(SubValueRelation::StrictSubValue {
                field: InductiveField {
                    type_name: facts.type_name,
                    variant_name: facts.variant_name,
                    field_name,
                    shape: RecursionShape::DirectRecursion,
                    element_type,
                },
                factor: ShrinkFactor::UnitShrink,
            });
        }
    }
    None
}

/// Phase-1 broadening Slice 3: bounded transitive trace from a match
/// scrutinee back to the caller's parameter port through nested-match
/// payload-binding chains.
///
/// Surface shape this enables (sibling of Slices 1 + 2; the recursive call
/// is two structural peels away from the parameter):
///
/// ```ignore
/// type EpListN = EpNilN | EpConsN(EpListN)
/// fn ep_count2(xs: EpListN) -> Int =
///   match xs {
///     EpConsN(t1) => match t1 {
///       EpConsN(t2) => ep_count2(t2),  // arg traces param via nested match
///       EpNilN => 0
///     },
///     EpNilN => 0
///   }
/// ```
///
/// The inner `Branch.input` is `t1` — a payload binding from the outer
/// match — not the parameter. Slices 1 / 2's direct
/// `branch.input == param` check rejects this; the tracer walks one or more
/// payload-binding levels to confirm structural descent before classifying.
///
/// The trace is bounded by a fixed depth (`SCRUTINEE_TRACE_DEPTH_LIMIT`)
/// for the same reason `callable_target_template_for_provenance` bounds its
/// instantiation peel: hitting the cap means the producer can no longer
/// prove the structural relation, so the caller fails closed
/// (`SubValueUnknown` upstream). `StrictSubValue` is sound at any positive
/// depth — every level peels one constructor — so the helper does not
/// distinguish depth in its boolean answer.
const SCRUTINEE_TRACE_DEPTH_LIMIT: usize = 16;

fn scrutinee_traces_to_param(dag: &Dag, scrutinee: PortId, param: PortId) -> bool {
    let mut current = scrutinee;
    for _ in 0..SCRUTINEE_TRACE_DEPTH_LIMIT {
        if current == param {
            return true;
        }
        // Walk one FieldProject indirection: record-payload variant patterns
        // (`EpNodeN { left: a }`) bind the user-scope name to the OUTPUT
        // port of a synthesized FieldProject whose input is the match arm's
        // payload port (see `lower_field_projection_from_port`). When such
        // a binding is itself an inner-match scrutinee, the tracer must
        // peel the projection before climbing payload-binding chains.
        if let Some(proj_input) = field_project_input_for_port(dag, current) {
            current = proj_input;
            continue;
        }
        if let Some(parent_input) = enclosing_branch_input_for_payload(dag, current) {
            current = parent_input;
            continue;
        }
        return false;
    }
    false
}

/// If `port` is the output of a single-input `FieldProject` transform,
/// return the projected-from input port. Used by [`scrutinee_traces_to_param`]
/// to walk past variant-record-pattern FieldProject indirections.
fn field_project_input_for_port(dag: &Dag, port: PortId) -> Option<PortId> {
    let Behavior::Transform(transform) = dag.resolve_producer_opt(&port)? else {
        return None;
    };
    let (TransformTarget::UnresolvedFieldProject { .. }
    | TransformTarget::ResolvedFieldProject { .. }) = &transform.target
    else {
        return None;
    };
    if transform.inputs.len() != 1 {
        return None;
    }
    Some(transform.inputs[0])
}

/// If `port` is the payload binding port of some `Path`, return the
/// enclosing `BranchNode.input`. Used by [`scrutinee_traces_to_param`] to
/// climb one nested-match level. `None` indicates the chain bottoms out
/// (port isn't a payload binding) — the tracer's fail-closed signal.
fn enclosing_branch_input_for_payload(dag: &Dag, port: PortId) -> Option<PortId> {
    dag.nodes()
        .iter()
        .filter_map(Behavior::as_branch)
        .find_map(|b| {
            b.paths.iter().find_map(|p| {
                let binding = p.binding.as_ref()?;
                if binding.payload_port == port {
                    Some(b.input)
                } else {
                    None
                }
            })
        })
}

/// Authoritative variant-structural facts derived from a resolved variant
/// declaration. Returns `None` for `UnresolvedVariant` and for `ResolvedVariant`
/// whose parent `Disj` cannot be located by scanning declarations — same
/// fail-closed discipline as the rest of the per-call producer.
struct VariantStructuralFacts {
    type_name: String,
    variant_name: String,
    payload_fields: Vec<(String, DeclarationId)>,
}

fn variant_structural_facts(dag: &Dag, pattern: &BranchPattern) -> Option<VariantStructuralFacts> {
    let variant_decl_id = match pattern {
        BranchPattern::ResolvedVariant(id) => *id,
        BranchPattern::UnresolvedVariant { .. } => return None,
    };
    let (parent_decl, variants) = dag.declarations().iter().find_map(|decl| {
        if let TypeConnective::Disj { variants } = &decl.connective {
            if variants.iter().any(|f| f.ty == variant_decl_id) {
                return Some((decl, variants));
            }
        }
        None
    })?;
    let type_name = parent_decl.name.clone()?;
    let variant_name = variants
        .iter()
        .find(|f| f.ty == variant_decl_id)
        .map(|f| f.label.clone())?;
    let payload_fields = match &dag.declaration(variant_decl_id).connective {
        TypeConnective::Conj { children } => children
            .iter()
            .map(|f| (f.label.clone(), f.ty))
            .collect::<Vec<_>>(),
        _ => return None,
    };
    Some(VariantStructuralFacts {
        type_name,
        variant_name,
        payload_fields,
    })
}

/// Walk a declaration through `Instantiation` / structural-alias edges to a
/// named declaration. Returns `None` when the chain bottoms out without a name
/// (anonymous template fields, unresolved atoms, depth-limit hit).
fn named_type_name(dag: &Dag, mut current: DeclarationId) -> Option<String> {
    for _ in 0..CALLABLE_PROVENANCE_TEMPLATE_DEPTH_LIMIT {
        let decl = dag.declaration(current);
        if let Some(name) = &decl.name {
            return Some(name.clone());
        }
        match &decl.connective {
            TypeConnective::Instantiation { template, .. } => current = *template,
            _ => return None,
        }
    }
    None
}

fn arithmetic_descent_relation(
    dag: &Dag,
    idx: usize,
    param: PortId,
    arg: PortId,
) -> Option<SubValueRelation> {
    let Behavior::Transform(transform) = dag.resolve_producer_opt(&arg)? else {
        return None;
    };
    let TransformTarget::Operator(OperatorKind::Arithmetic(op)) = transform.target else {
        return None;
    };
    // First E-P slice recognizes the same left-operand convention as the v3
    // recursive termination gate: `param - k` and `param / k`, not `k - param`.
    if transform.inputs.len() != 2 || transform.inputs[0] != param {
        return None;
    }
    let literal = literal_int_at(dag, transform.inputs[1])?;
    let factor = match op {
        ArithmeticOp::Sub => ShrinkFactor::ConstantShrink {
            steps: positive_amount_from_i64(literal)?,
        },
        ArithmeticOp::Div => ShrinkFactor::ProportionalShrink {
            divisor: proportional_divisor_from_i64(literal)?,
        },
        ArithmeticOp::Add | ArithmeticOp::Mul => return None,
    };
    Some(SubValueRelation::ArithmeticDescent {
        param: ordinal_param_label(idx),
        factor,
    })
}

fn literal_int_at(dag: &Dag, port: PortId) -> Option<i64> {
    let s = match dag.resolve_producer_opt(&port)? {
        Behavior::Value(value) => match &value.data {
            LiteralBits::Int(s) => s.as_str(),
            _ => return None,
        },
        _ => return None,
    };
    s.parse().ok()
}

/// 🟡 SCAFFOLD. `BindNode` currently carries parameter ports but not parameter
/// names, so the side table uses stable ordinal labels. Dissolves when E-P can
/// read reflected parameter names or when `SubValueRelation::ArithmeticDescent`
/// carries a structural `ParamRef`.
fn ordinal_param_label(idx: usize) -> String {
    format!("param_{idx}")
}

pub fn size_bound_param(bound: &SizeBound) -> Option<&str> {
    match bound {
        SizeBound::TreeSize { param } => Some(param.as_str()),
        SizeBound::CollectionSize { param } => Some(param.as_str()),
        SizeBound::ParserStreamSize { witness } => Some(witness.as_str()),
        SizeBound::WorklistDrainSize { element } => Some(element.as_str()),
        SizeBound::ArithmeticParam { param } => Some(param.as_str()),
        SizeBound::ExplicitCountZero
        | SizeBound::ExplicitCountPositive { .. }
        | SizeBound::Forever => None,
    }
}

pub fn is_constant_bound(bound: &SizeBound) -> bool {
    matches!(
        bound,
        SizeBound::ExplicitCountZero | SizeBound::ExplicitCountPositive { .. } | SizeBound::Forever
    )
}

/// Signed `Int` top iterate count (`i64::MAX`) for [`SizeBound::Forever`] / `repeat(max_int)`.
pub fn forever_iteration_bound() -> i64 {
    i64::MAX
}

/// `None` when `bound` is not constant (`ExplicitCount*` / `Forever` only).
pub fn constant_bound_value(bound: &SizeBound) -> Option<i64> {
    match bound {
        SizeBound::ExplicitCountZero => Some(0),
        SizeBound::ExplicitCountPositive { steps } => Some(positive_descent_count(steps)),
        SizeBound::Forever => Some(forever_iteration_bound()),
        _ => None,
    }
}

/// 🟢 TERMINAL — `IterationDimension` coproduct (`docs/modeling-discipline.md` §4).
///
/// Three-way projection from kernel algebra profiles onto iteration regimes. Authority:
/// `src/v3/std/computation.dag`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IterationDimension {
    TreeDescent,
    CollectionFold,
    ArithmeticRepeat,
}

/// 🟢 TERMINAL — `AlgebraProfile` coproduct (`docs/modeling-discipline.md` §4).
///
/// Closed seven-variant mirror of `dsl/std/algebra.dag` `AlgebraProfile`.
/// The profile table itself is read from the lowered `kernel_algebra_profile`
/// `ValueBody::Map`, not from a hand-maintained Rust lookup table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AlgebraProfile {
    OrderedRingProfile,
    ApproximateFieldProfile,
    BooleanAlgebraProfile,
    BooleanAlgebraCollectionProfile,
    FreeMonoidScalarProfile,
    FreeMonoidCollectionProfile,
    PartialFunctionProfile,
}

pub fn algebra_profile_to_dimension(profile: AlgebraProfile) -> Option<IterationDimension> {
    match profile {
        AlgebraProfile::FreeMonoidCollectionProfile
        | AlgebraProfile::FreeMonoidScalarProfile
        | AlgebraProfile::BooleanAlgebraCollectionProfile
        | AlgebraProfile::PartialFunctionProfile => Some(IterationDimension::CollectionFold),
        AlgebraProfile::OrderedRingProfile | AlgebraProfile::ApproximateFieldProfile => {
            Some(IterationDimension::ArithmeticRepeat)
        }
        AlgebraProfile::BooleanAlgebraProfile => None,
    }
}

pub fn type_iteration_dimension(type_name: &str) -> Option<IterationDimension> {
    if type_name == "Node" {
        return Some(IterationDimension::TreeDescent);
    }

    kernel_algebra_profile(type_name).and_then(algebra_profile_to_dimension)
}

/// Kernel type name → iteration algebra profile (`Int`, `List`, …).
///
/// Semantic authority is `dsl/std/algebra.dag` (`data kernel_algebra_profile`).
/// Runtime v3 reads the lowered [`ValueBody::Map`] from the bootstrapped DAG.
pub fn kernel_algebra_profile(type_name: &str) -> Option<AlgebraProfile> {
    BOOTSTRAPPED_DAG.kernel_algebra_profile(type_name)
}
#[derive(Debug, Clone)]
pub struct ValueNode {
    pub id: NodeId,
    pub data: LiteralBits,
    pub output: PortId,
    pub span: SourceSpan,
    /// Lane 2 Stage 2b: idempotency projection for this node.
    ///
    /// **Single authority:** the workflow fact lives only in this field (and the
    /// analogous field on [`BindNode`]). There is no separate `Dag`-level map.
    /// Reflected `ValueNode` / `BindNode` in `src/v3/std/substrate.dag` and
    /// [`Dag::lane2_workflow_effect_at`] are read-only projections of the same
    /// storage. The sole mutating constructor for tests/staging is
    /// [`Dag::try_register_lane2_workflow_effect`]; lowering fills this field when
    /// it exists. [`crate::workflow_idempotency::analyze_workflow`] reads through
    /// [`Dag::lane2_workflow_effect_at`].
    pub(crate) lane2_workflow: Option<Box<WorkflowEffect>>,
}

impl ValueNode {
    pub fn result_port(&self) -> PortId {
        self.output
    }

    /// Reflected substrate optional `WorkflowEffect?`: unboxes staged storage so
    /// Rust realization does not surface `Option<Box<WorkflowEffect>>` at the
    /// reflection boundary (`rust.dag` uses `AccessorMethod("lane2_workflow")`).
    pub fn lane2_workflow(&self) -> Option<&WorkflowEffect> {
        self.lane2_workflow.as_deref()
    }
}

/// Dispatch target of a `TransformNode`. Structural coproduct that
/// replaces the old single `target: DeclarationId` field.
///
/// **🟡 Mixed-lifecycle coproduct — M1(2.7).** `Callable` is
/// terminal (the long-term shape for all user function calls and
/// resolved named declarations). `Operator` is a **scaffold**
/// with a named dissolution trigger: the M2+ parser / desugarer
/// replaces `SurfaceExpr::Operator` with direct algebra-field
/// `Call`s, and this variant disappears back into
/// `Callable(DeclarationId)`. Terminal form is 1 variant.
///
/// **Dissolution receipt — Q3 + R9 operator dispatch.** Before
/// M1(2.7) the target was always a `DeclarationId` and primitive
/// operators were represented by an anonymous declaration whose
/// connective was `Atom(UnresolvedIdentifier("+"))`. Infer.rs then
/// string-matched the identifier payload to decide operator vs
/// callable. The string was the discriminator for a phase/job
/// coproduct hiding inside `UnresolvedIdentifier`. M1(2.7) split
/// the discriminator onto the `TransformTarget` variant itself,
/// and the operator walk in `infer::resolve_operator_arrow` reads
/// the operator's signature from the actual
/// `std/algebra.dag` field (e.g., `OrderedRing.add`) — the Rust
/// `OperatorKind` enum is only a lookup key, not a parallel
/// authority.
///
/// 4-pattern check on (Callable, FieldProject, Operator):
/// - Pattern 1 (fact placement): fails. Callable dispatches via
///   the declaration's Arrow body; unresolved FieldProject dispatches
///   via the input port's resolved Conj + field label; Operator
///   dispatches via the operand type's algebra walk.
/// - Pattern 2 (variant-is-data): fails. Callable carries a
///   DeclarationId; resolved FieldProject carries the projected field
///   label resolved against the parent Conj; Operator carries an
///   OperatorKind.
/// - Pattern 3 (algebraic form): fails.
/// - Pattern 4 (dimensional): fails.
///
/// Verdict: mixed lifecycle. `Callable` and `ResolvedFieldProject`
/// are terminal; `UnresolvedFieldProject` is pre-infer only.
/// `Operator` is 🟡 scaffold with an explicit M2+ dissolution
/// trigger (surface grammar adoption of direct algebra field
/// access or a parse-time desugaring pass).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TransformTarget {
    /// A user function or resolved declaration. Inference walks the
    /// referenced declaration's `Arrow` connective via `resolve_arrow`.
    Callable(DeclarationId),
    /// Pre-infer structural projection on a Conj-typed parent value.
    /// Inference is the only authority allowed to turn this label into
    /// a projected child declaration.
    UnresolvedFieldProject { field_label: String },
    /// Post-infer structural projection. The field label is the single
    /// authority; inference and emit resolve it against the parent Conj
    /// so duplicate-typed fields cannot collapse to the same child id.
    ResolvedFieldProject { field_label: String },
    /// A primitive binary operator. Inference dispatches on the
    /// `OperatorKind` variant directly: arithmetic returns the operand
    /// type, comparison returns Bool, and logical is Bool-monomorphic.
    /// No declaration is allocated.
    Operator(OperatorKind),
}

#[derive(Debug, Clone)]
pub struct TransformNode {
    pub id: NodeId,
    /// The dispatch target of this transform. Either a user function /
    /// resolved declaration (`Callable(DeclarationId)`) or a primitive
    /// operator (`Operator(OperatorKind)`). Discriminated structurally,
    /// not by a string payload. See `TransformTarget`.
    pub target: TransformTarget,
    pub inputs: Vec<PortId>,
    pub output: PortId,
    pub span: SourceSpan,
}

impl TransformNode {
    pub fn result_port(&self) -> PortId {
        self.output
    }
}

/// B4.3 — authority at lowering: user `match` Branch nodes, not `if` on Bool.
/// See [`BranchEmitParticipation`] and `INVARIANTS.md` P2 (no `span.file` as contract).
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum BranchEmitParticipation {
    UserMatch,
}

#[derive(Debug, Clone)]
pub struct BranchNode {
    pub id: NodeId,
    pub input: PortId,
    pub paths: Vec<Path>,
    pub output: PortId,
    pub span: SourceSpan,
    /// B4.3: `Some(UserMatch)` iff lowered from a surface `match` (not `if`).
    pub(crate) emit_participation: Option<BranchEmitParticipation>,
}

impl BranchNode {
    pub fn result_port(&self) -> PortId {
        self.output
    }

    pub fn emit_participation(&self) -> Option<BranchEmitParticipation> {
        self.emit_participation
    }
}

// Branch-pattern/path mirrors are generated from `std/substrate.dag`.
// The host keeps only the `Path.output` field rename plus impl behavior below.
include!("dag_branch_generated.rs");

impl Path {
    pub fn result_port(&self) -> PortId {
        self.output
    }
}

// ParamRef / TransformRef / ElementRef / BoolPortRef /
// NonEmptyList / NonSingletonList live in `dag/ports.rs`. See
// re-exports at the top of this module.

// The `std.effects` mirror (DB-18 / Lane 2 Stage 2b) lives in
// `dag/effects.rs`. See re-exports at the top of this module.

// ── end std.effects mirror (DB-18) ───────────────────────────────────
// Cluster / loop-bound carriers below are Track 9 mutual-recursion
// witnesses — not part of the Lane 2 Stage 2b effects algebra.

// Cluster/loop-bound mirrors are generated from `std/substrate.dag`;
// keep only host helper behavior in Rust.
include!("dag_cluster_generated.rs");

impl LoopBound {
    pub fn count_port(&self) -> Option<PortId> {
        match self {
            Self::Cardinality { count } => Some(*count),
            Self::Descent { .. } => None,
        }
    }
}

include!("dag_lookup_generated.rs");

include!("dag_cost_generated.rs");

#[derive(Debug, Clone)]
pub struct LoopNode {
    pub id: NodeId,
    pub source: PortId,
    pub init: PortId,
    pub body: NodeId,
    pub bound: LoopBound,
    pub output: PortId,
    pub span: SourceSpan,
}

impl LoopNode {
    pub fn result_port(&self) -> PortId {
        self.output
    }
}

/// B4.3 — authority at lowering: user `fn` and lambda `Arrow` body binds; not
/// refinement predicates, `let`, or builder-only Binds. See
/// `primitive_type_id_for_port_shared` and `INVARIANTS.md` P2.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum BindEmitParticipation {
    UserCallable,
}

#[derive(Debug, Clone)]
pub struct BindNode {
    pub id: NodeId,
    pub name: String,
    pub value: PortId,
    /// Parameter ports for function bindings. Empty for value bindings. A
    /// non-empty `params` is the structural distinction between a function
    /// definition and a value definition — no Optional marker, no coproduct,
    /// no type tag. See the C2 dissolution receipt at the top of this file.
    pub params: Vec<PortId>,
    pub span: SourceSpan,
    /// Lane 2 Stage 2b: idempotency projection for this bind — same authority as
    /// [`ValueNode::lane2_workflow`] (see that comment: one native field, reflected
    /// substrate + `lane2_workflow_effect_at`, writers via
    /// [`Dag::try_register_lane2_workflow_effect`] or lowering).
    pub(crate) lane2_workflow: Option<Box<WorkflowEffect>>,
    /// B4.3: set for user `fn` / lambda bodies that participate in named-type-alias
    /// emission; `None` for refinement, `let`, and synthetic Binds. Same visibility
    /// contract as [`Self::lane2_workflow`]: crate-private storage; reads go through
    /// [`Self::emit_participation`].
    pub(crate) emit_participation: Option<BindEmitParticipation>,
}

impl BindNode {
    /// Structural alias for `self.value`. `std/substrate.dag` names this
    /// field `result_port` across all behavior variants; the Rust struct
    /// kept the historical name `value` for BindNode only. `.dag`-generated
    /// lenses read `bind.result_port` (see `lenses/complexity.dag` Bind
    /// branch); hand-written Rust reads `bind.value` (see
    /// `tests/m2_lens_cost_migration_test.rs`). This method is the single
    /// point of agreement between the two.
    pub fn result_port(&self) -> PortId {
        self.value
    }

    /// Reflected substrate optional `WorkflowEffect?` — same contract as
    /// [`ValueNode::lane2_workflow`].
    pub fn lane2_workflow(&self) -> Option<&WorkflowEffect> {
        self.lane2_workflow.as_deref()
    }

    /// B4.3 emit authority: `Some(UserCallable)` iff this bind participates in the
    /// named-type-alias emission path. Reflected in `src/v3/std/substrate.dag`;
    /// writers are lowering / inference / `dag::builder` only.
    pub fn emit_participation(&self) -> Option<BindEmitParticipation> {
        self.emit_participation
    }
}

// Checkpoint C1.
//
// Five L1 behaviors from docs/v3-spec.md lines 22-218. The thesis claim is that
// these are terminal — the irreducible decomposition of computation. M0
// validated the claim; M1(2.5) did not add a 6th variant.
//
// Dissolution-patterns check (all four attempted at M0.1, preserved at M1(2.5)):
//
//   - Pattern 1 (fact placement): fails. Every consumer dispatches on behavior
//     first — cost adds for sequence, multiplies for Loop, maxes for Branch.
//   - Pattern 2 (variant-is-data): fails. Value has `data`, Transform has
//     `inputs + target`, Branch has `paths`, Loop has `bound + source + init +
//     body`, Bind has `value + params`. Structurally different, not one shape
//     with a tag.
//   - Pattern 3 (algebraic-form): confirms terminality.
//       Value    = Terminal intro
//       Transform= morphism application (Apply(DeclarationId))
//       Branch   = Coproduct elim
//       Loop     = well-founded recursion
//       Bind     = let-abstraction (with params for function defs)
//   - Pattern 4 (dimensional): fails. No common M-dim coordinate space generates
//     all five.
//
// STOP SIGNAL: wanting a 6th variant. Pause and escalate rather than silently
// extending.
#[derive(Debug, Clone)]
pub enum Behavior {
    Value(ValueNode),
    Transform(TransformNode),
    Branch(BranchNode),
    Loop(LoopNode),
    Bind(BindNode),
}

/// Workflow-root identification — Rust mirror of
/// [`crate::dag::WorkflowRoot`]'s declaration in
/// `src/v3/std/substrate.dag`.
///
/// 🟡 SCAFFOLD coproduct (mirroring the .dag receipt). The three arms
/// partition every legitimate `Dag` exactly once:
///
///   - `SingleRoot(p)` — α (last topological `Bind`) selected `p`.
///     Emitted whenever the Dag contains at least one `Bind`; multiple
///     Binds are NOT ambiguous under α — linear `d.nodes` picks
///     exactly one last element by definition.
///   - `NoRoot` — zero `Bind` behaviors in `d.nodes`. Lens fold short-
///     circuits to `DimensionFail`; runtime evaluation rejects.
///   - `AmbiguousRoot { candidates }` — reserved for the future
///     enumerate-all-eligible-entries rule that R2-Evaluator's
///     `evaluate(program, entry, args)` consumes for multi-entry
///     programs (per Items 4+5 / #1176 §3.2). The α / γ "last X Bind"
///     rules cannot populate this arm; today's α implementation never
///     emits it. Carried as `NonSingletonList<PortId>` to make the
///     pre-disambiguation 1-candidate case structurally
///     unrepresentable — `AmbiguousRoot` requires ≥2 candidates by
///     construction.
///
/// Dissolution: γ refinement and the enumerate-all rule both reuse
/// this same partition behind the `workflow_root_port` accessor; no
/// carrier change required when those rules wire.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkflowRoot {
    SingleRoot(PortId),
    NoRoot,
    AmbiguousRoot {
        candidates: NonSingletonList<PortId>,
    },
}

impl Behavior {
    pub fn id(&self) -> NodeId {
        match self {
            Behavior::Value(v) => v.id,
            Behavior::Transform(t) => t.id,
            Behavior::Branch(b) => b.id,
            Behavior::Loop(l) => l.id,
            Behavior::Bind(b) => b.id,
        }
    }

    pub fn as_value(&self) -> Option<&ValueNode> {
        if let Behavior::Value(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_transform(&self) -> Option<&TransformNode> {
        if let Behavior::Transform(t) = self {
            Some(t)
        } else {
            None
        }
    }

    pub fn as_branch(&self) -> Option<&BranchNode> {
        if let Behavior::Branch(b) = self {
            Some(b)
        } else {
            None
        }
    }

    pub fn as_loop(&self) -> Option<&LoopNode> {
        if let Behavior::Loop(l) = self {
            Some(l)
        } else {
            None
        }
    }

    pub fn as_bind(&self) -> Option<&BindNode> {
        if let Behavior::Bind(b) = self {
            Some(b)
        } else {
            None
        }
    }
}

/// Substrate-level pointers to declarations that the compiler asks
/// about by role rather than by name. Populated at the end of
/// `bootstrap()` by `Dag::populate_primitive_cache` after every std/
/// file has been lowered and cross-file stubs have been resolved.
/// Before that point every field is `None`.
///
/// **Dissolution receipt.** The cache exists to remove per-call
/// `declaration_by_name("Int")` / `"Bool"` / `"String"` lookups from
/// the hot path. It is NOT a parallel authority — the
/// `DeclarationId`s it stores are already in `Dag.declarations`, and
/// the cache is a typed index over the same table. Adding a new role
/// to this cache is a C1-class stop signal: if the compiler wants to
/// ask "which declaration is the canonical X?", that question
/// belongs as a structural edge on the declaration, not as a role
/// slot here. The three roles cached today (the user-facing
/// primitives `Int`/`Bool`/`String`) are the substrate's built-in
/// roles and cannot dissolve into declaration-level edges because
/// they answer "which declaration is Int?" — a question about
/// *identity*, not *relationship*.
///
/// **Round-10 correction.** Earlier revisions included a
/// `realization_meta` field pointing at a cached `Realization`
/// declaration. Production bootstrap doesn't load a `Realization`
/// declaration (realization facts live in `dsl/extdeps/languages/*`
/// per the thesis, not in the M1(2.7) std/ set), so the cache was
/// always `None` and the downstream `is_realization_shape` check
/// always failed. The check now validates structural shape (Conj +
/// `meta_tag.is_some()`) directly — no cache needed.
#[derive(Debug, Default, Clone)]
pub(crate) struct PrimitiveCache {
    pub int: Option<TypeShape>,
    pub bool: Option<TypeShape>,
    pub string: Option<TypeShape>,
}

/// Substrate-marker handles populated at bootstrap end. Each field
/// resolves a marker declaration from `src/v3/spec/v3_l1.dag` to its
/// `DeclarationId`. The handles are the typed dispatch keys that
/// `lower_record_to_structural` and `emit_rust` use to recognize
/// behavior templates and the `DeclarationRef` sentinel meta-type
/// without round-tripping through string names.
///
/// **Why this is not a name-bridge regression.** PrimitiveCache's
/// dissolution ledger says "adding a new role to this cache is a
/// C1-class stop signal." SubstrateMarkers is a separate cache for
/// a different category of role: not "which declaration is the
/// canonical X" (which the primitive cache answers for user-facing
/// types) but "which declaration is the structural marker for v3's
/// L1 behavior X" — a substrate-internal question that consumers
/// (lower, emit) need to answer at every dispatch site. The cache
/// turns those dispatch sites into typed `decl_id == cache.bind`
/// comparisons instead of `decl.name.as_deref() == Some("Bind")`
/// string matches. The name lookup happens once at bootstrap end
/// from the standard module path; every consumer downstream reads
/// the typed handle.
///
/// **Why one cache per category.** Mixing "user primitive" and
/// "substrate marker" roles in a single cache would conflate two
/// distinct stability classes. Primitives change with v3's user-
/// facing type system; substrate markers change with v3's L1
/// behavior set. Splitting the caches keeps each one's invariants
/// independent.
#[derive(Debug, Default, Clone)]
pub(crate) struct RealizationMetaCache {
    /// `TypeRealization` meta-type. Declared in
    /// `src/v3/spec/rust.dag` (and any future per-target spec
    /// file) as the meta-tag for `data rust_*: TypeRealization
    /// = { ... }` items. Cached typed handle so that consumers
    /// like `emit_rust::RealizationIndexes::build` filter by
    /// `meta_tag == Some(this)` without going through
    /// `declaration_by_name("TypeRealization")` at hot-path
    /// time.
    pub type_realization: Option<DeclarationId>,
    /// `OperatorRealization` meta-type. Same role as `type_realization`
    /// for operator realizations.
    pub operator_realization: Option<DeclarationId>,
    /// `BehaviorRealization` meta-type. Same role for behavior
    /// template realizations (Bind/Branch/Main).
    pub behavior_realization: Option<DeclarationId>,
    /// `CallableRealization` meta-type. Same role for callable
    /// render strategies (currently staged std.list helpers).
    pub callable_realization: Option<DeclarationId>,
    /// `TypeInstantiationRealization` meta-type. Same role for
    /// generic template carriers such as `List<T> -> Vec<T>`.
    pub type_instantiation_realization: Option<DeclarationId>,
    /// `PatternRealization` meta-type. Same role for carrier-specific
    /// pattern lowering facts (currently staged `List<T> -> Vec<T>`
    /// destructuring).
    pub pattern_realization: Option<DeclarationId>,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct TargetSyntaxCache {
    /// `rust_language` syntax bundle declaration loaded from
    /// `src/v3/spec/rust.dag`. This is the target-language
    /// authority the Rust emitter reads for expression/control-flow/
    /// function/type/value syntax templates.
    pub rust_language: Option<DeclarationId>,
    /// `rust_rendering` ownership-model declaration loaded from
    /// `src/v3/spec/rust.dag`. This is the target-language
    /// authority the Rust emitter reads for borrow-vs-construct
    /// rendering policy at use sites.
    pub rust_rendering: Option<DeclarationId>,
    /// `rust_execution_model` declaration loaded from
    /// `src/v3/spec/rust.dag`. Used by emitters to gate the
    /// ownership stage on the target memory model.
    pub rust_execution_model: Option<DeclarationId>,
    /// `rust_source_filtering` declaration loaded from
    /// `src/v3/spec/rust.dag`.
    pub rust_source_filtering: Option<DeclarationId>,
    /// `rust_execution_requirement` declaration loaded from
    /// `src/v3/spec/rust.dag`.
    pub rust_execution_requirement: Option<DeclarationId>,
    /// `dag_model` declaration loaded from
    /// `src/v3/std/computation_model.dag`. The source-side
    /// computation-model fact the emitter reads alongside the
    /// target execution model.
    pub dag_model: Option<DeclarationId>,
    /// `go_language` syntax bundle declaration loaded from
    /// `src/v3/spec/go.dag`.
    pub go_language: Option<DeclarationId>,
    /// `go_execution_model` declaration loaded from
    /// `src/v3/spec/go.dag`.
    pub go_execution_model: Option<DeclarationId>,
    /// `go_source_filtering` declaration loaded from
    /// `src/v3/spec/go.dag`.
    pub go_source_filtering: Option<DeclarationId>,
    /// `go_execution_requirement` declaration loaded from
    /// `src/v3/spec/go.dag`.
    pub go_execution_requirement: Option<DeclarationId>,
    /// `python_language` syntax bundle declaration loaded from
    /// `src/v3/spec/python.dag`.
    pub python_language: Option<DeclarationId>,
    /// `python_target` execution-model declaration loaded from
    /// `src/v3/spec/python.dag`.
    pub python_target: Option<DeclarationId>,
    /// `python_source_filtering` declaration loaded from
    /// `src/v3/spec/python.dag`.
    pub python_source_filtering: Option<DeclarationId>,
    /// `python_execution_requirement` declaration loaded from
    /// `src/v3/spec/python.dag`.
    pub python_execution_requirement: Option<DeclarationId>,
    /// Shared target-authority bindings scanned from
    /// `TargetCleanEmissionBinding` data items. This is the single
    /// cached bridge from `LanguageSpec` to `CleanEmissionContract`;
    /// adding a new target extends the spec surface, not compiler
    /// branches.
    pub clean_emission_by_language: HashMap<DeclarationId, DeclarationId>,
}

/// Shared target-authority bundle tying one `LanguageSpec`
/// declaration to the `CleanEmissionContract` that governs how code
/// for that language must render. This keeps the pairing as a typed
/// carrier instead of asking downstream consumers to reconstruct it
/// from parallel per-target caches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TargetSyntaxBundle {
    pub language_spec: DeclarationId,
    pub clean_emission_spec: DeclarationId,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct StdlibTypeCache {
    /// `std.list.List` template declaration. Resolved once at
    /// bootstrap end so downstream consumers compare typed
    /// declaration ids instead of reconstructing stdlib identity
    /// through `declaration_by_name("List")`.
    pub list: Option<DeclarationId>,
    /// The `Map<K, V> = PartialFunction<…>` template head (`PartialFunction` in
    /// `dsl/std`). The underlying record carries algebra `Arrow`+`NoBody` fields
    /// for operations, not first-class `fn` **values** — emit must not recurse
    /// the template in `decl_includes_first_class_arrow_data` the way it does
    /// for a user `G<T>` (PR #676).
    pub partial_function: Option<DeclarationId>,
}

/// Substrate declarations emitters used to resolve via
/// `declaration_by_name("...")` at call time. Populated once at
/// bootstrap end in [`Dag::populate_primitive_cache`]; downstream code
/// reads typed [`DeclarationId`] handles (ROADMAP P3 — name-keyed emit
/// lookups).
#[derive(Debug, Default, Clone)]
pub(crate) struct EmitAnchorCache {
    /// `OrderedRing` algebra Conj — canonical fallback for operator fields.
    pub ordered_ring: Option<DeclarationId>,
    /// `AbelianGroup` algebra Conj — canonical authority for phantom-unit closure.
    pub abelian_group: Option<DeclarationId>,
    /// `SubstrateAccessorBinding` meta-type for substrate accessor data items.
    pub substrate_accessor_binding: Option<DeclarationId>,
    /// `Dag` graph type (`src/v3/std/substrate.dag`).
    pub dag_type: Option<DeclarationId>,
    /// `fold` from `src/v3/std/list.dag` (list catamorphism).
    pub std_list_fold: Option<DeclarationId>,
    /// `rust_functions` syntax record — resolved to the v3 spec copy in
    /// `src/v3/spec/rust.dag` (bootstrap name resolution prefers `src/v3/`
    /// over the duplicate carrier in `dsl/std/languages.dag`).
    pub rust_functions: Option<DeclarationId>,
    /// `MethodEmitTemplate` coproduct parent used to map a method-template
    /// constructor id back to its variant label during target emission.
    pub method_emit_template: Option<DeclarationId>,
    /// `MethodTemplateContract` meta-type used by fold-method contract checks.
    pub method_template_contract: Option<DeclarationId>,
    /// `std.list.concat_method` method declaration.
    pub concat_method: Option<DeclarationId>,
    /// `std.list.length_method` method declaration.
    pub length_method: Option<DeclarationId>,
    /// `std.list.fold_method` method declaration.
    pub fold_method: Option<DeclarationId>,
    /// `std.list.is_empty_method` method declaration.
    pub is_empty_method: Option<DeclarationId>,
    /// `std.list.filter_method` method declaration.
    pub filter_method: Option<DeclarationId>,
    /// `std.list.flat_map_method` method declaration.
    pub flat_map_method: Option<DeclarationId>,
    /// `std.list.any_method` method declaration.
    pub any_method: Option<DeclarationId>,
    /// `std.list.all_method` method declaration.
    pub all_method: Option<DeclarationId>,
    /// `std.list.map_method` method declaration.
    pub map_method: Option<DeclarationId>,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct PatternStrategyVariants {
    pub vector_list: Option<DeclarationId>,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct FieldAccessVariants {
    pub direct_field: Option<DeclarationId>,
    pub accessor_method: Option<DeclarationId>,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct ParameterDispositionVariants {
    pub borrowed: Option<DeclarationId>,
    pub consumed: Option<DeclarationId>,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct MemoryModelVariants {
    pub value_only: Option<DeclarationId>,
    pub garbage_collected: Option<DeclarationId>,
    pub ref_counted: Option<DeclarationId>,
    pub ownership_based: Option<DeclarationId>,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct ScopeModelVariants {
    pub lexical_scoping: Option<DeclarationId>,
    pub dynamic_scoping: Option<DeclarationId>,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct ReadStrategyVariants {
    pub borrow: Option<DeclarationId>,
    pub pass_by_value: Option<DeclarationId>,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct ConstructStrategyVariants {
    pub copy_or_clone: Option<DeclarationId>,
    pub pass_by_value: Option<DeclarationId>,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct MutabilityVariants {
    pub immutable: Option<DeclarationId>,
    pub mutable: Option<DeclarationId>,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct PurityVariants {
    pub pure: Option<DeclarationId>,
    pub effectful: Option<DeclarationId>,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct StructureVariants {
    pub explicit_dag: Option<DeclarationId>,
    pub arbitrary: Option<DeclarationId>,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct IterationVariants {
    pub bounded: Option<DeclarationId>,
    pub unbounded: Option<DeclarationId>,
}

/// Cached variant DeclarationIds for fixed emit-model coproducts.
/// Populated once at bootstrap end so emitters dispatch on typed
/// constructors instead of resolving parent/variant names at parse time.
#[derive(Debug, Default, Clone)]
pub(crate) struct EmitModelVariants {
    pub pattern_strategy: PatternStrategyVariants,
    pub field_access: FieldAccessVariants,
    pub parameter_disposition: ParameterDispositionVariants,
    pub memory_model: MemoryModelVariants,
    pub scope_model: ScopeModelVariants,
    pub read_strategy: ReadStrategyVariants,
    pub construct_strategy: ConstructStrategyVariants,
    pub mutability: MutabilityVariants,
    pub purity: PurityVariants,
    pub structure: StructureVariants,
    pub iteration: IterationVariants,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct SubstrateMarkers {
    /// `src/v3/spec/v3_l1.dag` `ValueBehavior` marker (L1 Value-shaped
    /// behavior). Targets literals in target language realizations.
    /// Bare `Value` is reserved for PB-Runtime union types, not this marker.
    pub value: Option<DeclarationId>,
    /// `Transform` marker. Targets generic transform-shaped
    /// emissions (currently unused — operators dispatch via the
    /// algebra field walk).
    pub transform: Option<DeclarationId>,
    /// `Branch` marker. Targets the per-target if/else template.
    pub branch: Option<DeclarationId>,
    /// `Loop` marker. Targets per-target loop emission templates
    /// (M2+ once recursion lowers to Loop).
    pub r#loop: Option<DeclarationId>,
    /// `Bind` marker. Targets the per-target let-statement
    /// template.
    pub bind: Option<DeclarationId>,
    /// `Main` marker. Targets the per-target program-entry-point
    /// wrapper template.
    pub main: Option<DeclarationId>,
    /// `DeclarationRef` sentinel meta-type. When a record-literal
    /// field's declared type walks to this declaration, the
    /// lowerer accepts identifier / dotted-path expressions as the
    /// field value and emits `FieldValue::Reference(decl_id)`
    /// instead of `FieldValue::Literal(...)`. This is the
    /// structural escape hatch that lets target spec files write
    /// typed declaration references inside data bodies.
    pub declaration_ref: Option<DeclarationId>,
}

/// Typed handles for the four `PatternBindingRule` variants declared
/// in `src/v3/std/clean_emission.dag`. Populated at bootstrap end.
///
/// Every per-target emitter (`emit_rust`, `emit_go`, future
/// `emit_python`) parses the `pattern_bindings` field of its
/// `CleanEmissionContract` by comparing the field's constructor id
/// against these handles. Caching once at bootstrap end is the
/// single bridge from the variant names in `clean_emission.dag` to
/// declaration ids; every consumer downstream dispatches on the
/// typed id. Without this cache, each emitter would re-resolve the
/// same four names on every contract parse — the exact anti-pattern
/// `feedback_substrate_principle_audit` Q5 calls out ("multiple
/// call sites independently reconstructing the same fact").
#[derive(Debug, Default, Clone)]
pub(crate) struct PatternBindingRuleVariants {
    /// `EmitBindingAlways` — the emitter always writes the binding,
    /// even when the body does not consume it. Used by targets
    /// whose native compilers do not warn on unused pattern
    /// bindings.
    pub emit_always: Option<DeclarationId>,
    /// `EmitUnderscoreWhenUnused` — the emitter replaces the
    /// binding name with `_` (Rust) or elides the binding
    /// statement (Go) when the body does not consume it.
    pub emit_underscore: Option<DeclarationId>,
    /// `EmitPrefixedUnderscoreWhenUnused` — the emitter keeps the
    /// binding but prefixes its name with `_` (Python-style).
    pub emit_prefixed: Option<DeclarationId>,
    /// `NotApplicablePatternBinding` — targets without structural
    /// pattern matching (no variant applicable).
    pub not_applicable: Option<DeclarationId>,
}

/// Cached `VariantPayloadFieldAccessRule` variant DeclarationIds
/// resolved from `src/v3/std/clean_emission.dag`. Populated at
/// bootstrap end and consumed by emitters when parsing
/// `CleanEmissionContract.variant_payload_field_access`.
///
/// Like `PatternBindingRuleVariants`, this central cache is the sole
/// bridge from `clean_emission.dag`'s variant labels to typed
/// `DeclarationId`s. That keeps the "how do target specs classify
/// variant-payload field access?" fact in one place instead of letting
/// three emitters re-resolve the same names independently (Q5
/// construction authority).
#[derive(Debug, Default, Clone)]
pub(crate) struct VariantPayloadFieldAccessRuleVariants {
    /// `AccessFromPayloadBinding` — the bound payload expression is a
    /// whole carrier whose fields can be projected directly.
    pub access_from_payload_binding: Option<DeclarationId>,
    /// `OverrideNamedFieldsAtBindingSite` — named payload fields must
    /// be broken into per-field bindings at the match site.
    pub override_named_fields_at_binding_site: Option<DeclarationId>,
}

/// Cached `VerifierOutputPolicy` variant DeclarationIds resolved
/// from `src/v3/std/clean_emission.dag`. Populated at bootstrap
/// end alongside `PatternBindingRuleVariants`. The
/// `post_emit_verifier` harness dispatches on the cached typed
/// ids when parsing `CleanEmissionContract.post_emit_verifier
/// .output_policy`. Lives here for the same §Layer opacity /
/// §Semantic authority reasons as `PatternBindingRuleVariants`:
/// compiler-side consumers must compare constructor
/// `DeclarationId`s, not variant label strings.
#[derive(Debug, Default, Clone)]
pub(crate) struct VerifierOutputPolicyVariants {
    /// `IgnoreVerifierOutput` — the verdict hinges entirely on
    /// `expected_exit_code`; stdout / stderr are informational.
    pub ignore_output: Option<DeclarationId>,
    /// `RequireEmptyStdout` — the declared verifier (e.g. `gofmt
    /// -l`) reports dirty files on stdout while exiting 0, so
    /// empty stdout is the load-bearing signal.
    pub require_empty_stdout: Option<DeclarationId>,
    /// `RequireEmptyStderr` — exit code and stdout are
    /// informational; stderr carries the verdict.
    pub require_empty_stderr: Option<DeclarationId>,
    /// `RequireEmptyStdoutAndStderr` — both streams must be empty
    /// in addition to the expected exit code.
    pub require_empty_stdout_and_stderr: Option<DeclarationId>,
}

/// Cached `CallableStrategy` variant DeclarationIds resolved from
/// `src/v3/std/emit_model.dag`. Populated at bootstrap end and
/// consumed by the shared Go emitter plus the Rust/Python target
/// emitters when parsing `CallableRealization.strategy`.
///
/// This mirrors `PatternBindingRuleVariants`: one bootstrap-time
/// name walk through the Disj, then downstream consumers compare
/// typed `DeclarationId`s only. Without the cache, each emitter
/// re-resolves the same ten variant labels independently at parse
/// time, recreating the "multiple call sites reconstruct the same
/// fact" bridge.
#[derive(Debug, Default, Clone)]
pub(crate) struct CallableStrategyVariants {
    pub list_empty: Option<DeclarationId>,
    pub list_singleton: Option<DeclarationId>,
    pub list_cons: Option<DeclarationId>,
    pub list_concat: Option<DeclarationId>,
    pub list_length: Option<DeclarationId>,
    pub list_is_empty: Option<DeclarationId>,
    pub list_fold: Option<DeclarationId>,
    pub list_map: Option<DeclarationId>,
    pub list_filter: Option<DeclarationId>,
    pub list_contains: Option<DeclarationId>,
}

#[derive(Debug, Clone)]
pub struct Dag {
    /// Behaviors in construction order. NodeId(k) lives at `nodes[k]`; a forward
    /// walk visits dependencies before dependents (load-bearing for inference).
    nodes: Vec<Behavior>,
    /// Declarations in allocation order. DeclarationId(k) lives at
    /// `declarations[k]` by the same invariant.
    declarations: Vec<Declaration>,
    ports: HashMap<PortId, Port>,
    diagnostics: DiagnosticTable,
    next_node_id: u32,
    next_declaration_id: u32,
    next_port_id: u32,
    /// Cached typed pointers to declarations the compiler asks about by
    /// role. Populated at the end of `bootstrap()`; empty before then.
    primitives: PrimitiveCache,
    /// Cached substrate marker DeclarationIds resolved from
    /// `dsl/std/v3_l1.dag`. Populated alongside the primitive
    /// cache at bootstrap end. See `SubstrateMarkers` for the
    /// rationale behind splitting markers from primitives.
    substrate_markers: SubstrateMarkers,
    /// Cached realization meta-type DeclarationIds resolved
    /// from `src/v3/spec/rust.dag` (and other per-target spec
    /// files). Populated at bootstrap end. Lets downstream
    /// consumers like `emit_rust::RealizationIndexes::build`
    /// filter by typed meta-tag handle instead of doing a name
    /// lookup at hot-path time.
    realization_metas: RealizationMetaCache,
    /// Cached target-language syntax bundle declarations.
    target_syntax: TargetSyntaxCache,
    /// Cached stdlib type-template declarations.
    stdlib_types: StdlibTypeCache,
    /// Cached emit-time substrate anchors (operator fallback, accessors,
    /// std list fold, Rust function-syntax bundle).
    emit_anchors: EmitAnchorCache,
    /// Cached `PatternBindingRule` variant DeclarationIds resolved
    /// from `src/v3/std/clean_emission.dag`. Populated at bootstrap
    /// end alongside `SubstrateMarkers` / `RealizationMetaCache`.
    /// Every per-target emitter dispatches on the cached typed ids
    /// when parsing its `CleanEmissionContract.pattern_bindings`
    /// field — see `PatternBindingRuleVariants` for why one
    /// central cache is the right shape instead of per-emitter
    /// name lookups.
    pattern_binding_rule_variants: PatternBindingRuleVariants,
    /// Cached `VariantPayloadFieldAccessRule` variant DeclarationIds
    /// resolved from `src/v3/std/clean_emission.dag`. Every emitter
    /// reads these typed ids when parsing
    /// `CleanEmissionContract.variant_payload_field_access`.
    variant_payload_field_access_rule_variants: VariantPayloadFieldAccessRuleVariants,
    /// Cached `VerifierOutputPolicy` variant DeclarationIds
    /// resolved from `src/v3/std/clean_emission.dag`. The
    /// `post_emit_verifier` harness dispatches on the cached typed
    /// ids when parsing `CleanEmissionContract.post_emit_verifier
    /// .output_policy` — same §Layer opacity shape as
    /// `pattern_binding_rule_variants`.
    verifier_output_policy_variants: VerifierOutputPolicyVariants,
    /// Cached `CallableStrategy` variant DeclarationIds resolved
    /// from `src/v3/std/emit_model.dag`. Consumed by the shared Go
    /// emitter plus Rust/Python target emitters when parsing
    /// `CallableRealization.strategy`.
    callable_strategy_variants: CallableStrategyVariants,
    /// Cached fixed emit-model coproduct variant DeclarationIds used by
    /// emit target parsers.
    emit_model_variants: EmitModelVariants,
    /// Sidecar structural facts for mutually-recursive SCCs.
    clusters: Vec<Cluster>,
    /// Synthetic match carriers for anonymous `T?` cardinalities. Used when
    /// inference needs stable `Some` / `None` variant identities without
    /// promoting optionals into named top-level declarations.
    optional_match_disjs: HashMap<DeclarationId, DeclarationId>,
    /// Exclusive lower bound on [`DeclarationId::raw`] for declarations
    /// appended **after** embedded bootstrap fixture construction for this
    /// `Dag`. Bootstrap rows use `raw() <` this value; runtime / user-phase
    /// allocations use `raw() >=` this value (structural; do not infer from
    /// `span.file`). `0` for [`Dag::empty`]; stamped when serving
    /// [`Dag::new`] and sibling bootstrap snapshots.
    declaration_append_begin_after_bootstrap: u32,
}

/// Which committed `bootstrap_*_generated` snapshot shape [`Dag::finalize_runtime_bootstrap_from_generated_snapshot`]
/// is finalizing — drives extdeps-only asserts and pilot validation.
#[derive(Debug, Clone, Copy)]
pub(crate) enum RuntimeBootstrapFixtureKind {
    /// `bootstrap_generated` / `bootstrap_generated_without_parse_surface`.
    FullExtdepsPipelineSnapshot,
    /// `bootstrap_std_generated` (std-only; no extdeps fixture-key assert / pilot walk).
    StdOnlySnapshot,
}

static BOOTSTRAPPED_DAG: LazyLock<Dag> = LazyLock::new(|| {
    let mut dag = bootstrap_generated::bootstrapped_fixture_dag();
    dag.finalize_runtime_bootstrap_from_generated_snapshot(
        RuntimeBootstrapFixtureKind::FullExtdepsPipelineSnapshot,
    );
    dag
});

// Generated bootstrap snapshots are performance caches over the checked-in
// `.dag` authorities, not independent authorities. `regen_bootstrap` is the
// sole writer and the PB-1 equivalence tests ratchet generated == runtime.
static BOOTSTRAPPED_STD_FIXTURE_DAG: LazyLock<Dag> = LazyLock::new(|| {
    let mut dag = bootstrap_std_generated::bootstrapped_std_fixture_dag();
    dag.finalize_runtime_bootstrap_from_generated_snapshot(
        RuntimeBootstrapFixtureKind::StdOnlySnapshot,
    );
    dag
});

static BOOTSTRAPPED_DAG_WITHOUT_PARSE_SURFACE_FIXTURE: LazyLock<Dag> = LazyLock::new(|| {
    let mut dag =
        bootstrap_generated_without_parse_surface::bootstrapped_fixture_without_parse_surface_dag();
    dag.finalize_runtime_bootstrap_from_generated_snapshot(
        RuntimeBootstrapFixtureKind::FullExtdepsPipelineSnapshot,
    );
    dag
});

/// Typed result of a Bind-chain producer walk. Discriminates
/// **legitimate absence** (parameter port with no producer) from
/// **malformed-substrate** states (missing port, missing node, cyclic
/// Bind chain). INVARIANTS P3 fail-closed: consumers must not collapse
/// the malformed variants into the legitimate-absence path — see
/// `Dag::resolve_producer_opt` for the compat wrapper that does
/// collapse, and `lens_apply.rs::eligibility_walk_port` for the
/// canonical typed consumer.
pub enum ProducerLookup<'a> {
    /// Legitimate: the port has no `produced_by` link (e.g., a
    /// parameter port bound by the interpreter at evaluation time).
    NoProducer,
    /// A non-Bind producer reached after walking through any number of
    /// Bind hops.
    Found(&'a Behavior),
    /// Malformed substrate: the `PortId` does not resolve in the DAG.
    MissingPort { port: PortId },
    /// Malformed substrate: a `produced_by` link references a `NodeId`
    /// that is not in the DAG.
    MissingNode { producer: NodeId },
    /// Malformed substrate: a cycle was detected while walking the
    /// Bind chain — the walk would not terminate.
    BindCycle { detected_at: NodeId },
}

impl Dag {
    // Only `bootstrap_regen_fresh` constructs an empty Dag for regen; omitting
    // that module without `bootstrap-regen-fresh` would otherwise trip `dead_code`.
    // If a **default-feature** caller needs `empty()`, do not grow `allow(dead_code)` —
    // revisit the PB-1-e regen-vs-runtime authority split instead.
    #[cfg_attr(not(feature = "bootstrap-regen-fresh"), allow(dead_code))]
    pub(crate) fn empty() -> Self {
        Self {
            nodes: Vec::new(),
            declarations: Vec::new(),
            ports: HashMap::new(),
            diagnostics: DiagnosticTable::new(),
            next_node_id: 0,
            next_declaration_id: 0,
            next_port_id: 0,
            primitives: PrimitiveCache::default(),
            substrate_markers: SubstrateMarkers::default(),
            realization_metas: RealizationMetaCache::default(),
            target_syntax: TargetSyntaxCache::default(),
            stdlib_types: StdlibTypeCache::default(),
            emit_anchors: EmitAnchorCache::default(),
            pattern_binding_rule_variants: PatternBindingRuleVariants::default(),
            variant_payload_field_access_rule_variants:
                VariantPayloadFieldAccessRuleVariants::default(),
            verifier_output_policy_variants: VerifierOutputPolicyVariants::default(),
            callable_strategy_variants: CallableStrategyVariants::default(),
            emit_model_variants: EmitModelVariants::default(),
            clusters: Vec::new(),
            optional_match_disjs: HashMap::new(),
            declaration_append_begin_after_bootstrap: 0,
        }
    }

    pub fn new() -> Self {
        (*BOOTSTRAPPED_DAG).clone()
    }

    /// Declaration graph with **no** embedded bootstrap fixture.
    ///
    /// Cross-crate **tests** only (e.g. E-6 witnesses that compare bootstrap-full vs empty graphs).
    /// Production paths must use [`Dag::new`].
    ///
    /// Gated behind feature `empty-substrate-for-tests` so normal library builds do not expose a
    /// public constructor for substrate-invalid empty graphs ([`Dag::new`] remains the production entry).
    #[cfg(feature = "empty-substrate-for-tests")]
    pub fn new_empty_for_testing() -> Self {
        Self::empty()
    }

    /// First [`DeclarationId::raw`] reserved for declarations appended after
    /// embedded bootstrap construction. Values strictly below this bound index
    /// bootstrap fixture rows; values at or above were allocated afterward
    /// (user compile / lowering), without consulting `span.file`.
    pub fn post_bootstrap_declaration_append_begin(&self) -> u32 {
        self.declaration_append_begin_after_bootstrap
    }

    /// `true` when `id` was allocated after bootstrap for this `Dag` (same
    /// predicate as `id.raw() >= self.post_bootstrap_declaration_append_begin()`).
    pub fn is_runtime_appended_declaration(&self, id: DeclarationId) -> bool {
        id.raw() >= self.declaration_append_begin_after_bootstrap
    }

    /// After embedding additional non-user fixtures (e.g. `complexity.dag` for T-LAS),
    /// re-stamp the append boundary so those declarations are treated like bootstrap for
    /// strict identifier / scaffold sweeps.
    pub(crate) fn seal_prepended_authority_fixture_range(&mut self) {
        self.stamp_declaration_append_begin_after_bootstrap();
    }

    fn stamp_declaration_append_begin_after_bootstrap(&mut self) {
        self.declaration_append_begin_after_bootstrap = self.next_declaration_id;
    }

    /// Invariant steps for every `Dag` materialized from a committed
    /// `bootstrap_*_generated` snapshot before it is cached or cloned for
    /// [`Dag::new`]. Centralizes bootstrap finalization so the append boundary
    /// is not tied ad hoc to each `LazyLock` closure.
    pub(crate) fn finalize_runtime_bootstrap_from_generated_snapshot(
        &mut self,
        kind: RuntimeBootstrapFixtureKind,
    ) {
        self.assert_user_defined_arrow_bodies_point_at_binds();
        if matches!(
            kind,
            RuntimeBootstrapFixtureKind::FullExtdepsPipelineSnapshot
        ) {
            assert_bootstrap_fixture_paths_match_regen_keys(self);
        }
        self.populate_primitive_cache();
        if matches!(
            kind,
            RuntimeBootstrapFixtureKind::FullExtdepsPipelineSnapshot
        ) {
            crate::int_literal_ranges::validate_rust_pilot_integer_primitives(self);
        }
        self.stamp_declaration_append_begin_after_bootstrap();
    }

    fn assert_user_defined_arrow_bodies_point_at_binds(&self) {
        for declaration in &self.declarations {
            let TypeConnective::Arrow {
                body: ArrowBody::UserDefined(bind_id),
                ..
            } = &declaration.connective
            else {
                continue;
            };
            let node_id = bind_id.node_id();
            if !matches!(self.node_opt(&node_id), Some(Behavior::Bind(_))) {
                let name = declaration.name.as_deref().unwrap_or("<anonymous>");
                panic!(
                    "generated bootstrap invariant violation: declaration {name:?} ({:?}) has \
                     ArrowBody::UserDefined({node_id:?}), but that NodeId does not point at \
                     Behavior::Bind",
                    declaration.id
                );
            }
        }
    }

    /// Clone of the bootstrapped Dag used by [`crate::compile_parse_surface_std_authority_dag`]:
    /// every fixture except `src/v3/std/parse_surface.dag`, so that file can be
    /// parsed and lowered again without duplicate top-level names.
    pub(crate) fn new_without_parse_surface_staged_fixture_bootstrap() -> Self {
        (*BOOTSTRAPPED_DAG_WITHOUT_PARSE_SURFACE_FIXTURE).clone()
    }

    pub(crate) fn std_fixture_bootstrap_snapshot() -> Self {
        (*BOOTSTRAPPED_STD_FIXTURE_DAG).clone()
    }

    /// Typed accessor for the cached `Int` primitive `TypeShape`. `None`
    /// only when bootstrap failed to load `dsl/std/integer.dag` — a
    /// diagnostic is already on `Dag.diagnostics` in that case and the
    /// compile fails through the ordinary channel.
    pub fn int_shape(&self) -> Option<TypeShape> {
        self.primitives.int
    }

    /// Typed accessor for the cached `Bool` primitive `TypeShape`. Same
    /// bootstrap-failure semantics as `int_shape`.
    pub fn bool_shape(&self) -> Option<TypeShape> {
        self.primitives.bool
    }

    /// Runtime/substrate Bool reification authority for branch dispatch.
    ///
    /// Bool remains a scalar [`LiteralBits::Bool`] for ordinary literal data,
    /// comparisons, and spec fields. When a Bool value is used as a Branch
    /// scrutinee, however, the runtime must compare against the same
    /// declaration-id identity that inference resolved for `True` / `False`
    /// patterns. This helper is the transitional single lookup point for that
    /// reification.
    ///
    /// Retirement trigger: delete this helper once Bool runtime production is
    /// uniformly represented as `VariantValue { tag: True/False, .. }` at the
    /// producer boundary instead of at branch consumers.
    pub fn bool_runtime_variant_id(&self, value: bool) -> Option<DeclarationId> {
        let bool_decl = self.declaration_by_name("Bool")?;
        let TypeConnective::Disj { variants } = &bool_decl.connective else {
            return None;
        };
        let label = if value { "True" } else { "False" };
        variants
            .iter()
            .find(|variant| variant.label == label)
            .map(|variant| variant.ty)
    }

    /// Typed accessor for the cached `String` primitive `TypeShape`. Same
    /// bootstrap-failure semantics as `int_shape`.
    pub fn string_shape(&self) -> Option<TypeShape> {
        self.primitives.string
    }

    /// The std **scalar** `TypeShape` roots used to prune algebra / ring
    /// `TypeConnective::Arrow` from the first-class-`fn` *data* walk in
    /// `emit::rust_target` (`decl_includes_first_class_arrow_data` and related).
    /// **Tied to** `Dag`’s `int` / `bool` / `string` fields in `primitives` — when a
    /// new bootstrap primitive (e.g. `Float`) gains a `float_shape` accessor, extend
    /// this to `[TypeShape; 4]` (or equivalent) in the same change as the emit
    /// predicate so the allowlist cannot drift silently (PR #676, Opus review).
    pub fn first_class_fn_walk_bootstrap_prune_type_shapes(&self) -> [TypeShape; 3] {
        [
            self.int_shape()
                .expect("bootstrap `Int` (dsl/std) required for this helper"),
            self.bool_shape()
                .expect("bootstrap `Bool` (dsl/std) required for this helper"),
            self.string_shape()
                .expect("bootstrap `String` (dsl/std) required for this helper"),
        ]
    }

    /// Typed accessor for the v3_l1 `Bind` marker. `None` only when
    /// bootstrap failed to load `dsl/std/v3_l1.dag`. Used by emit
    /// passes to look up the per-target Bind realization without a
    /// name string.
    pub fn bind_marker(&self) -> Option<DeclarationId> {
        self.substrate_markers.bind
    }

    /// Typed accessor for the v3_l1 `Branch` marker. Same bootstrap-
    /// failure semantics as `bind_marker`.
    pub fn branch_marker(&self) -> Option<DeclarationId> {
        self.substrate_markers.branch
    }

    /// Typed accessor for the v3_l1 `Loop` marker. Same bootstrap-
    /// failure semantics as `bind_marker`.
    pub fn loop_marker(&self) -> Option<DeclarationId> {
        self.substrate_markers.r#loop
    }

    /// Typed accessor for the v3_l1 `Transform` marker. Same
    /// bootstrap-failure semantics as `bind_marker`.
    pub fn transform_marker(&self) -> Option<DeclarationId> {
        self.substrate_markers.transform
    }

    /// Typed accessor for the v3_l1 `ValueBehavior` marker. Same bootstrap-
    /// failure semantics as `bind_marker`. (Rust name `value_marker` is
    /// stable; the underlying declaration is `ValueBehavior`, not bare `Value`.)
    pub fn value_marker(&self) -> Option<DeclarationId> {
        self.substrate_markers.value
    }

    /// Typed accessor for the v3_l1 `Main` marker. Same bootstrap-
    /// failure semantics as `bind_marker`.
    pub fn main_marker(&self) -> Option<DeclarationId> {
        self.substrate_markers.main
    }

    /// Typed accessor for the `DeclarationRef` sentinel meta-type
    /// declared in `src/v3/spec/v3_l1.dag`. Used by
    /// `lower_record_to_structural` to recognize record-literal
    /// fields whose declared type means "any declaration reference"
    /// (and therefore accept identifier / dotted-path expressions
    /// as field values instead of requiring scalar literals).
    pub fn declaration_ref_marker(&self) -> Option<DeclarationId> {
        self.substrate_markers.declaration_ref
    }

    /// Typed accessor for the `TypeRealization` meta-type declared
    /// in `src/v3/spec/rust.dag`. `None` only when bootstrap
    /// failed to load the file. Used by
    /// `emit_rust::RealizationIndexes::build` to filter declarations
    /// by `meta_tag == Some(this)` without name lookup.
    pub fn type_realization_meta(&self) -> Option<DeclarationId> {
        self.realization_metas.type_realization
    }

    /// Typed accessor for the `OperatorRealization` meta-type.
    /// Same bootstrap-failure semantics as `type_realization_meta`.
    pub fn operator_realization_meta(&self) -> Option<DeclarationId> {
        self.realization_metas.operator_realization
    }

    /// Typed accessor for the `BehaviorRealization` meta-type.
    /// Same bootstrap-failure semantics as `type_realization_meta`.
    pub fn behavior_realization_meta(&self) -> Option<DeclarationId> {
        self.realization_metas.behavior_realization
    }

    /// Typed accessor for the `CallableRealization` meta-type.
    /// Same bootstrap-failure semantics as `type_realization_meta`.
    pub fn callable_realization_meta(&self) -> Option<DeclarationId> {
        self.realization_metas.callable_realization
    }

    /// Typed accessor for the `TypeInstantiationRealization`
    /// meta-type. Same bootstrap-failure semantics as
    /// `type_realization_meta`.
    pub fn type_instantiation_realization_meta(&self) -> Option<DeclarationId> {
        self.realization_metas.type_instantiation_realization
    }

    /// Same bootstrap-failure semantics as `type_realization_meta`.
    pub fn pattern_realization_meta(&self) -> Option<DeclarationId> {
        self.realization_metas.pattern_realization
    }

    /// Typed accessor for the Rust target-language syntax bundle
    /// declared in `src/v3/spec/rust.dag`.
    pub fn rust_language_spec(&self) -> Option<DeclarationId> {
        self.target_syntax.rust_language
    }

    /// Typed accessor for the Rust target-language ownership model
    /// declared in `src/v3/spec/rust.dag`.
    pub fn rust_rendering_spec(&self) -> Option<DeclarationId> {
        self.target_syntax.rust_rendering
    }

    /// Typed accessor for the Rust `CleanEmissionContract`
    /// declaration loaded from `src/v3/spec/rust.dag` (E-5). Callers
    /// parse the structural fields via
    /// `structural_fields_for_decl` and dispatch on the rule
    /// variants. `None` before bootstrap completes or when the spec
    /// file has been altered so the data item is missing — the
    /// latter is a spec-file drift and surfaces at emit time as
    /// `EmitError::MissingTargetSyntax`.
    pub fn rust_clean_emission_spec(&self) -> Option<DeclarationId> {
        self.rust_target_syntax_bundle()
            .map(|bundle| bundle.clean_emission_spec)
    }

    /// Typed accessor for the Rust target authority bundle pairing
    /// `rust_language` with its `CleanEmissionContract`.
    pub fn rust_target_syntax_bundle(&self) -> Option<TargetSyntaxBundle> {
        let language_spec = self.target_syntax.rust_language?;
        Some(TargetSyntaxBundle {
            language_spec,
            clean_emission_spec: *self
                .target_syntax
                .clean_emission_by_language
                .get(&language_spec)?,
        })
    }

    /// Typed accessor for the Rust target execution model
    /// declaration loaded from `src/v3/spec/rust.dag`.
    pub fn rust_execution_model_spec(&self) -> Option<DeclarationId> {
        self.target_syntax.rust_execution_model
    }

    pub fn rust_source_filtering_spec(&self) -> Option<DeclarationId> {
        self.target_syntax.rust_source_filtering
    }

    pub fn rust_execution_requirement_spec(&self) -> Option<DeclarationId> {
        self.target_syntax.rust_execution_requirement
    }

    /// Typed accessor for the source computation-model declaration
    /// declared in `src/v3/std/computation_model.dag`.
    pub fn computation_model_spec(&self) -> Option<DeclarationId> {
        self.target_syntax.dag_model
    }

    /// Typed accessor for the Go target-language syntax bundle
    /// declared in `src/v3/spec/go.dag`.
    pub fn go_language_spec(&self) -> Option<DeclarationId> {
        self.target_syntax.go_language
    }

    /// Typed accessor for the Go target execution model
    /// declaration loaded from `src/v3/spec/go.dag`.
    pub fn go_execution_model_spec(&self) -> Option<DeclarationId> {
        self.target_syntax.go_execution_model
    }

    pub fn go_source_filtering_spec(&self) -> Option<DeclarationId> {
        self.target_syntax.go_source_filtering
    }

    pub fn go_execution_requirement_spec(&self) -> Option<DeclarationId> {
        self.target_syntax.go_execution_requirement
    }

    /// Typed accessor for the Go `CleanEmissionContract`
    /// declaration loaded from `src/v3/spec/go.dag` (E-5 / Lane 1
    /// Stage 1c PR 2). Mirrors `rust_clean_emission_spec`; emitter
    /// parses the structural fields and dispatches on the rule
    /// variants.
    pub fn go_clean_emission_spec(&self) -> Option<DeclarationId> {
        self.go_target_syntax_bundle()
            .map(|bundle| bundle.clean_emission_spec)
    }

    /// Typed accessor for the Go target authority bundle pairing
    /// `go_language` with its `CleanEmissionContract`.
    pub fn go_target_syntax_bundle(&self) -> Option<TargetSyntaxBundle> {
        let language_spec = self.target_syntax.go_language?;
        Some(TargetSyntaxBundle {
            language_spec,
            clean_emission_spec: *self
                .target_syntax
                .clean_emission_by_language
                .get(&language_spec)?,
        })
    }

    /// Typed accessor for the Python target-language syntax bundle
    /// declared in `src/v3/spec/python.dag`.
    pub fn python_language_spec(&self) -> Option<DeclarationId> {
        self.target_syntax.python_language
    }

    /// Typed accessor for the Python target execution model
    /// declaration loaded from `src/v3/spec/python.dag`.
    pub fn python_target_spec(&self) -> Option<DeclarationId> {
        self.target_syntax.python_target
    }

    pub fn python_source_filtering_spec(&self) -> Option<DeclarationId> {
        self.target_syntax.python_source_filtering
    }

    pub fn python_execution_requirement_spec(&self) -> Option<DeclarationId> {
        self.target_syntax.python_execution_requirement
    }

    /// Typed accessor for the Python `CleanEmissionContract`
    /// declaration loaded from `src/v3/spec/python.dag` (E-5 / Lane
    /// 1 Stage 1c PR 3). Mirrors `rust_clean_emission_spec` and
    /// `go_clean_emission_spec`; emitter parses the structural
    /// fields and dispatches on the rule variants.
    pub fn python_clean_emission_spec(&self) -> Option<DeclarationId> {
        self.python_target_syntax_bundle()
            .map(|bundle| bundle.clean_emission_spec)
    }

    /// Typed accessor for the Python target authority bundle pairing
    /// `python_language` with its `CleanEmissionContract`.
    pub fn python_target_syntax_bundle(&self) -> Option<TargetSyntaxBundle> {
        let language_spec = self.target_syntax.python_language?;
        Some(TargetSyntaxBundle {
            language_spec,
            clean_emission_spec: *self
                .target_syntax
                .clean_emission_by_language
                .get(&language_spec)?,
        })
    }

    /// Shared target-authority lookup keyed by the target's
    /// `LanguageSpec` declaration id. Consumers that already traffic
    /// in the shared emit-model language surface should resolve the
    /// language first, then ask for the corresponding target bundle
    /// through this accessor rather than reconstructing the
    /// `LanguageSpec -> CleanEmissionContract` pairing themselves.
    pub fn target_syntax_bundle_for_language(
        &self,
        language_spec: DeclarationId,
    ) -> Option<TargetSyntaxBundle> {
        Some(TargetSyntaxBundle {
            language_spec,
            clean_emission_spec: *self
                .target_syntax
                .clean_emission_by_language
                .get(&language_spec)?,
        })
    }

    /// Shared clean-emission lookup keyed by the target's
    /// `LanguageSpec` declaration id. This is the
    /// `TargetSyntaxBundle` projection for consumers that only need
    /// the clean-emission side of the pair.
    pub fn clean_emission_spec_for_language(
        &self,
        language_spec: DeclarationId,
    ) -> Option<DeclarationId> {
        self.target_syntax_bundle_for_language(language_spec)
            .map(|bundle| bundle.clean_emission_spec)
    }

    /// Typed accessor for the cached `std.list.List` template.
    pub fn list_template(&self) -> Option<DeclarationId> {
        self.stdlib_types.list
    }

    /// The `Map` type constructor’s underlying `PartialFunction` record template
    /// (`Map<K, V> = Inst(PartialFunction, [K, V])` in `dsl/std`). See
    /// [`Dag::list_template`].
    pub fn partial_function_template(&self) -> Option<DeclarationId> {
        self.stdlib_types.partial_function
    }

    /// Typed accessor for the canonical `OrderedRing` algebra declaration.
    /// Used by emitters for operator-field fallback without per-call name lookup.
    pub fn ordered_ring_decl(&self) -> Option<DeclarationId> {
        self.emit_anchors.ordered_ring
    }

    /// Typed accessor for the canonical `AbelianGroup` algebra declaration.
    pub fn abelian_group_decl(&self) -> Option<DeclarationId> {
        self.emit_anchors.abelian_group
    }

    /// Meta-type declaration id for `SubstrateAccessorBinding` data items.
    pub fn substrate_accessor_binding_meta(&self) -> Option<DeclarationId> {
        self.emit_anchors.substrate_accessor_binding
    }

    /// The substrate `Dag` graph type declaration id.
    pub fn dag_type_decl(&self) -> Option<DeclarationId> {
        self.emit_anchors.dag_type
    }

    /// `std.list.fold` — list catamorphism callable declaration.
    pub fn std_list_fold_decl(&self) -> Option<DeclarationId> {
        self.emit_anchors.std_list_fold
    }

    /// `rust_functions` syntax record from `src/v3/spec/rust.dag`.
    pub fn rust_functions_syntax_decl(&self) -> Option<DeclarationId> {
        self.emit_anchors.rust_functions
    }

    /// `MethodEmitTemplate` coproduct parent.
    pub(crate) fn method_emit_template_decl(&self) -> Option<DeclarationId> {
        self.emit_anchors.method_emit_template
    }

    /// `MethodTemplateContract` meta-type.
    pub(crate) fn method_template_contract_decl(&self) -> Option<DeclarationId> {
        self.emit_anchors.method_template_contract
    }

    /// `fold_method` method declaration.
    pub(crate) fn fold_method_decl(&self) -> Option<DeclarationId> {
        self.emit_anchors.fold_method
    }

    /// `concat_method` method declaration.
    pub(crate) fn concat_method_decl(&self) -> Option<DeclarationId> {
        self.emit_anchors.concat_method
    }

    /// `length_method` method declaration.
    pub(crate) fn length_method_decl(&self) -> Option<DeclarationId> {
        self.emit_anchors.length_method
    }

    /// `is_empty_method` method declaration.
    pub(crate) fn is_empty_method_decl(&self) -> Option<DeclarationId> {
        self.emit_anchors.is_empty_method
    }

    /// `filter_method` method declaration.
    pub(crate) fn filter_method_decl(&self) -> Option<DeclarationId> {
        self.emit_anchors.filter_method
    }

    /// `flat_map_method` method declaration.
    pub(crate) fn flat_map_method_decl(&self) -> Option<DeclarationId> {
        self.emit_anchors.flat_map_method
    }

    /// `any_method` method declaration.
    pub(crate) fn any_method_decl(&self) -> Option<DeclarationId> {
        self.emit_anchors.any_method
    }

    /// `all_method` method declaration.
    pub(crate) fn all_method_decl(&self) -> Option<DeclarationId> {
        self.emit_anchors.all_method
    }

    /// `map_method` method declaration.
    pub(crate) fn map_method_decl(&self) -> Option<DeclarationId> {
        self.emit_anchors.map_method
    }

    /// Typed accessor for the cached `PatternBindingRule` variant
    /// handles resolved from `src/v3/std/clean_emission.dag` at
    /// bootstrap end. Consumed by per-target emitters when
    /// dispatching on their `CleanEmissionContract.pattern_bindings`
    /// field — see `PatternBindingRuleVariants` for the rationale.
    pub(crate) fn pattern_binding_rule_variants(&self) -> &PatternBindingRuleVariants {
        &self.pattern_binding_rule_variants
    }

    /// Typed accessor for the cached `VariantPayloadFieldAccessRule`
    /// variant handles resolved from
    /// `src/v3/std/clean_emission.dag`. Consumed by per-target
    /// emitters when parsing
    /// `CleanEmissionContract.variant_payload_field_access`.
    pub(crate) fn variant_payload_field_access_rule_variants(
        &self,
    ) -> &VariantPayloadFieldAccessRuleVariants {
        &self.variant_payload_field_access_rule_variants
    }

    /// Typed accessor for the cached `VerifierOutputPolicy` variant
    /// handles resolved from `src/v3/std/clean_emission.dag` at
    /// bootstrap end. Consumed by the shared `post_emit_verifier`
    /// harness when parsing
    /// `CleanEmissionContract.post_emit_verifier.output_policy`.
    pub(crate) fn verifier_output_policy_variants(&self) -> &VerifierOutputPolicyVariants {
        &self.verifier_output_policy_variants
    }

    /// Typed accessor for the cached `CallableStrategy` variant
    /// handles resolved from `src/v3/std/emit_model.dag` at
    /// bootstrap end. Consumed by emiters when parsing
    /// `CallableRealization.strategy`.
    pub(crate) fn callable_strategy_variants(&self) -> &CallableStrategyVariants {
        &self.callable_strategy_variants
    }

    /// Typed accessor for fixed emit-model coproduct variant handles.
    pub(crate) fn emit_model_variants(&self) -> &EmitModelVariants {
        &self.emit_model_variants
    }

    pub fn nodes(&self) -> &[Behavior] {
        &self.nodes
    }

    pub fn nodes_owned(&self) -> Vec<Behavior> {
        self.nodes.clone()
    }

    pub fn declarations(&self) -> &[Declaration] {
        &self.declarations
    }

    pub fn declarations_owned(&self) -> Vec<Declaration> {
        self.declarations.clone()
    }

    pub fn ports(&self) -> Vec<Port> {
        let mut ports: Vec<Port> = self.ports.values().cloned().collect();
        ports.sort_by_key(|port| port.id().raw());
        ports
    }

    pub fn clusters(&self) -> &[Cluster] {
        &self.clusters
    }

    pub(crate) fn optional_match_disjs(&self) -> &HashMap<DeclarationId, DeclarationId> {
        &self.optional_match_disjs
    }

    pub fn cluster(&self, id: ClusterId) -> &Cluster {
        &self.clusters[id.index()]
    }

    /// Attaches a [`WorkflowEffect`] on [`Behavior`] nodes at `root` (`Value` or
    /// `Bind` only). Writes the same `lane2_workflow` field reflected in
    /// `substrate.dag` for `ValueNode` / `BindNode`. Returns `false` if `root` is
    /// missing or not `Value`/`Bind`.
    pub fn try_register_lane2_workflow_effect(
        &mut self,
        root: NodeId,
        workflow: WorkflowEffect,
    ) -> bool {
        let Some(behavior) = self.nodes.get_mut(root.index()) else {
            return false;
        };
        match behavior {
            Behavior::Value(v) => {
                v.lane2_workflow = Some(Box::new(workflow));
                true
            }
            Behavior::Bind(b) => {
                b.lane2_workflow = Some(Box::new(workflow));
                true
            }
            Behavior::Transform(_) | Behavior::Branch(_) | Behavior::Loop(_) => false,
        }
    }

    /// Takes `&NodeId` (not by value) so emitted Rust lens code and substrate
    /// accessor carriers agree with `node_opt` / `port_opt` — transform inputs
    /// render as borrows at the explicit boundary.
    pub fn lane2_workflow_effect_at(&self, root: &NodeId) -> Option<&WorkflowEffect> {
        match self.node_opt(root)? {
            Behavior::Value(v) => v.lane2_workflow(),
            Behavior::Bind(b) => b.lane2_workflow(),
            Behavior::Transform(_) | Behavior::Branch(_) | Behavior::Loop(_) => None,
        }
    }

    /// Workflow-root accessor (Director-locked α implementation per
    /// `docs/design-lens-fold-prerequisites.md` §"Prereq-3a"). Walks
    /// `d.nodes` (which is topologically ordered) backwards and returns:
    ///
    /// - `WorkflowRoot::SingleRoot(p)` for the last `Behavior::Bind`'s
    ///   `result_port`, when at least one `Bind` exists.
    /// - `WorkflowRoot::NoRoot` when zero `Bind` behaviors are present
    ///   (lens fold short-circuits to `DimensionFail`; runtime evaluation
    ///   rejects).
    /// - `WorkflowRoot::AmbiguousRoot { .. }` is intentionally never
    ///   emitted by this α implementation — the linear `d.nodes` order
    ///   cannot tie. The variant is reserved at the type level for the
    ///   future enumerate-all-eligible-entries rule that R2-Evaluator's
    ///   `evaluate(program, entry, args)` consumes.
    pub fn workflow_root_port(&self) -> WorkflowRoot {
        for behavior in self.nodes.iter().rev() {
            if let Behavior::Bind(b) = behavior {
                return WorkflowRoot::SingleRoot(b.result_port());
            }
        }
        WorkflowRoot::NoRoot
    }

    /// Node id of the last [`Behavior::Bind`] in this DAG (same reverse scan as [`Self::workflow_root_port`]).
    ///
    /// [`Dag::try_register_lane2_workflow_effect`] accepts [`Behavior::Value`] and [`Behavior::Bind`]
    /// only. The workflow root **port**'s `produced_by` may be a [`Behavior::Loop`] or
    /// [`Behavior::Transform`] (e.g. `std.list.fold` lowering); staging `lane2_workflow` must target
    /// this **Bind shell** so registration cannot silently no-op while the sequential indicator still
    /// reads `0`.
    pub fn workflow_lane2_subject(&self) -> Option<NodeId> {
        for behavior in self.nodes.iter().rev() {
            if let Behavior::Bind(_) = behavior {
                return Some(behavior.id());
            }
        }
        None
    }

    pub fn optional_match_disj(&self, cardinality_decl_id: DeclarationId) -> Option<DeclarationId> {
        self.optional_match_disjs.get(&cardinality_decl_id).copied()
    }

    pub fn set_optional_match_disj(
        &mut self,
        cardinality_decl_id: DeclarationId,
        disj_decl_id: DeclarationId,
    ) {
        self.optional_match_disjs
            .insert(cardinality_decl_id, disj_decl_id);
    }

    /// O(1) lookup by NodeId. Relies on the dense-sequential allocation invariant.
    pub fn node(&self, id: NodeId) -> &Behavior {
        let node = &self.nodes[id.index()];
        debug_assert_eq!(
            node.id(),
            id,
            "Dag::node: NodeId desync — topological invariant broken"
        );
        node
    }

    /// Option-returning variant for the .dag substrate accessor
    /// `node(d, id) -> Behavior?`. Same pattern as `port_opt`.
    pub fn node_opt(&self, id: &NodeId) -> Option<&Behavior> {
        self.nodes.get(id.index())
    }

    /// Producer walk for the .dag substrate accessor
    /// Typed Bind-chain producer walk. See `ProducerLookup` for the
    /// four discriminated outcomes; each malformed variant is a
    /// substrate-integrity violation that consumers MUST handle
    /// explicitly (do not collapse into `NoProducer`).
    pub fn resolve_producer_lookup(&self, port_id: &PortId) -> ProducerLookup<'_> {
        let bound = self.nodes.len();
        let mut current_port = *port_id;
        let mut last_producer: Option<NodeId> = None;
        for _ in 0..=bound {
            let Some(port) = self.port_opt(&current_port) else {
                return ProducerLookup::MissingPort { port: current_port };
            };
            let Some(producer_id) = port.produced_by else {
                return ProducerLookup::NoProducer;
            };
            let Some(behavior) = self.node_opt(&producer_id) else {
                return ProducerLookup::MissingNode {
                    producer: producer_id,
                };
            };
            last_producer = Some(producer_id);
            match behavior {
                Behavior::Bind(bind) => {
                    current_port = bind.value;
                    continue;
                }
                _ => return ProducerLookup::Found(behavior),
            }
        }
        // Walk exceeded `nodes.len()` hops — only possible on a cyclic
        // Bind chain. Report the last Bind node visited.
        ProducerLookup::BindCycle {
            detected_at: last_producer.expect("walk visited at least one Bind"),
        }
    }

    /// Compat wrapper over `resolve_producer_lookup`: maps the typed
    /// result to `Option<&Behavior>`, **collapsing** all malformed
    /// states (`MissingPort`, `MissingNode`, `BindCycle`) into `None`
    /// alongside legitimate `NoProducer`. Only use when malformed-
    /// substrate equivalence to absence is the intended semantics
    /// (e.g., callers that already wrap `None` in a fail-closed
    /// diagnostic via `ok_or`); prefer `resolve_producer_lookup` when
    /// the consumer must distinguish the five variants.
    pub fn resolve_producer_opt(&self, port_id: &PortId) -> Option<&Behavior> {
        match self.resolve_producer_lookup(port_id) {
            ProducerLookup::Found(b) => Some(b),
            ProducerLookup::NoProducer
            | ProducerLookup::MissingPort { .. }
            | ProducerLookup::MissingNode { .. }
            | ProducerLookup::BindCycle { .. } => None,
        }
    }

    /// O(1) lookup by DeclarationId. Same dense-sequential invariant as nodes.
    pub fn declaration(&self, id: DeclarationId) -> &Declaration {
        let decl = &self.declarations[id.index()];
        debug_assert_eq!(
            decl.id, id,
            "Dag::declaration: DeclarationId desync — allocation invariant broken"
        );
        decl
    }

    /// Option-returning variant for the .dag substrate accessor
    /// `declaration_by_id(d, id) -> Declaration?`. Same pattern as `node_opt`
    /// / `port_opt`: permissive at the reflected-substrate boundary; C-8
    /// fail-closed is enforced at the lens consumer that treats `None` as a
    /// substrate-integrity violation (valid ids can't legitimately miss).
    pub fn declaration_opt(&self, id: &DeclarationId) -> Option<&Declaration> {
        self.declarations.get(id.index())
    }

    /// **🟡 Scaffold — v3 migration preference rank.** Ranks declarations by
    /// source-tree location so that same-named declarations in `src/v3/` win
    /// over `dsl/` duplicates during the bootstrap window. This rule is a
    /// ratified-parallel-authority pattern (P2 Boundary Discipline), kept
    /// as a scaffold rather than deleted because the duplicates it resolves
    /// carry v3-only content (`v3.std.substrate` imports, `ElementRef` /
    /// `PortId` references, partitioned `EffectShape`, substrate-coupled
    /// diagnostic surface) that cannot yet be hosted under `dsl/std/` — v2
    /// CI still recursively ingests `dsl/` and cannot parse v3 grammar.
    ///
    /// **Dissolution trigger.** When every module currently duplicated
    /// between `dsl/std/` and `src/v3/std/` (or `src/v3/spec/`) has
    /// converged to a single canonical home, delete this preference rule
    /// and the mirrored policy in `lower.rs::collect_symbols`. After
    /// convergence there must be no rank-based duplicate-authority bridge:
    /// lookup either resolves against the single surviving authority or
    /// fails closed on multiple matches. Convergence checklist tracked in
    /// ROADMAP.md "Post-merge debt" under the file-preference scaffold row:
    ///   - `module std.effects` (`dsl/std/effects.dag` ↔ `src/v3/std/effects.dag`)
    ///   - `module std.verification` (`dsl/std/verification.dag` ↔ `src/v3/std/verification.dag`)
    ///   - embedded `http_path` mirror inside `src/v3/std/effects.dag:118-260`
    ///   - `module std.computation` (`dsl/std/computation.dag` ↔ `src/v3/std/computation.dag`)
    ///   - `module std.induction` (`dsl/std/induction.dag` ↔ `src/v3/std/induction.dag`)
    ///   - `module std.termination` (`dsl/std/termination.dag` ↔ `src/v3/std/termination.dag`)
    fn declaration_name_preference_rank(file: &str) -> usize {
        if file.starts_with("src/v3/") {
            2
        } else if file.starts_with("dsl/") {
            0
        } else {
            1
        }
    }

    /// Find a top-level declaration by name. During the duplicate-authority
    /// bootstrap window this applies the temporary preference scaffold:
    /// `src/v3/` declarations outrank legacy `dsl/` duplicates; equal-rank
    /// ties remain deterministic by scan order but are not a semantic rule.
    ///
    /// **The preference bias is a v3-migration scaffold**, not a durable
    /// lookup policy — see `declaration_name_preference_rank` for the
    /// dissolution trigger. Once duplicate `std.effects` /
    /// `std.verification` authorities converge to a single home this
    /// function should drop the rank-based bridge and require
    /// single-authority lookup, failing closed on multiple matches.
    ///
    /// **Dissolution trigger.** Once duplicate authorities converge,
    /// delete the preference scaffold and require single-authority
    /// lookup; multiple surviving matches become a fail-closed error.
    ///
    /// **This scan only finds declarations that carry a surface-
    /// visible name** — TypeParams, sum variants, and realization
    /// scaffolds are allocated with `name: None` and are intentionally
    /// unreachable here. Referring to a type parameter, variant
    /// constructor, or realization instance by name outside its
    /// parent's body is a compile error, not silent mis-resolution.
    pub fn declaration_by_name(&self, name: &str) -> Option<&Declaration> {
        self.declarations
            .iter()
            .filter(|d| d.name.as_deref() == Some(name))
            .max_by_key(|decl| {
                (
                    Self::declaration_name_preference_rank(&decl.span.file),
                    std::cmp::Reverse(decl.id.raw()),
                )
            })
    }

    /// Typed accessor for the `rust_pilot_primitives` data declaration
    /// from `dsl/extdeps/languages/rust/primitives.dag` (path authorized by
    /// B4.4 `bootstrap_fixture_authority` and the regen host's
    /// `BOOTSTRAP_FIXTURE_PATH_KEYS` filter over `EXTDEPS_FILES`). Returns the
    /// top-level `List<RustPrimitive>` declaration whose *type* the
    /// target-grounding engine walks structurally (`RustPrimitive =
    /// IntegerPrimitive | NonIntegerPrimitive {target_name, algebra,
    /// carrier, is_copy[, overflow]}`).
    ///
    /// **Path 2 satisfaction.** The returned declaration's `value_body`
    /// is `ValueBody::List(_)`, so both the sum type shape and the
    /// 10-element pilot enumeration are structurally walkable. Map-shaped
    /// bootstrap data such as `kernel_algebra_profile` now lowers through
    /// `ValueBody::Map`; the remaining debt is retiring Rust mirrors that
    /// still read those maps through hand-authored accessors.
    ///
    /// Returns `None` only when bootstrap failed to load
    /// `rust/primitives.dag`, in which case a diagnostic is already on
    /// `Dag.diagnostics`.
    pub fn rust_pilot_primitives(&self) -> Option<&Declaration> {
        self.declaration_by_name("rust_pilot_primitives")
    }

    /// Typed accessor for `data kernel_algebra_profile` in
    /// `dsl/std/algebra.dag`.
    ///
    /// This is the v3-side substrate authority for kernel algebra enrichment:
    /// the map is lowered as [`ValueBody::Map`], keys are kernel type names,
    /// and values are zero-payload `AlgebraProfile` variants. Returning `None`
    /// means either the key is absent or the lowered declaration is malformed;
    /// callers treat both as "no kernel algebra profile for this type".
    pub fn kernel_algebra_profile(&self, type_name: &str) -> Option<AlgebraProfile> {
        let decl = self.declaration_by_name("kernel_algebra_profile")?;
        let ValueBody::Map(entries) = decl.value_body.as_ref()? else {
            return None;
        };
        let (_, value) = entries.entries().iter().find(|(key, _)| key == type_name)?;
        self.algebra_profile_from_field_value(value)
    }

    fn algebra_profile_from_field_value(&self, value: &FieldValue) -> Option<AlgebraProfile> {
        let FieldValue::Variant {
            constructor,
            payload,
        } = value
        else {
            return None;
        };
        if !payload.is_empty() {
            return None;
        }

        let profile_decl = self.declaration_by_name("AlgebraProfile")?;
        let TypeConnective::Disj { variants } = &profile_decl.connective else {
            return None;
        };
        let label = variants
            .iter()
            .find(|variant| variant.ty == *constructor)?
            .label
            .as_str();
        match label {
            "OrderedRingProfile" => Some(AlgebraProfile::OrderedRingProfile),
            "ApproximateFieldProfile" => Some(AlgebraProfile::ApproximateFieldProfile),
            "BooleanAlgebraProfile" => Some(AlgebraProfile::BooleanAlgebraProfile),
            "BooleanAlgebraCollectionProfile" => {
                Some(AlgebraProfile::BooleanAlgebraCollectionProfile)
            }
            "FreeMonoidScalarProfile" => Some(AlgebraProfile::FreeMonoidScalarProfile),
            "FreeMonoidCollectionProfile" => Some(AlgebraProfile::FreeMonoidCollectionProfile),
            "PartialFunctionProfile" => Some(AlgebraProfile::PartialFunctionProfile),
            _ => None,
        }
    }

    /// Virtual paths from the B4.4 extdeps-bootstrap fixture carrier
    /// (`bootstrap_fixture_authority` in
    /// `src/v3/std/extdeps_bootstrap_fixtures.dag`), in the order fields
    /// appear on the lowered `ValueBody::Structural` body.
    ///
    /// Returns `None` if the declaration is missing, has no structural body, or
    /// any fixture slot fails shape checks. Returns `Some` even when the product
    /// has zero fields (degenerate); callers that require a non-empty set should
    /// assert separately.
    ///
    /// Used to keep the regen host's `BOOTSTRAP_FIXTURE_PATH_KEYS` filter
    /// aligned with the substrate declaration (compared in this module's bootstrap
    /// `LazyLock` initializers).
    pub fn bootstrap_fixture_virtual_paths(&self) -> Option<Vec<String>> {
        let decl = self.declaration_by_name("bootstrap_fixture_authority")?;
        let body = decl.value_body.as_ref()?;
        let ValueBody::Structural { fields } = body else {
            return None;
        };
        let mut out = Vec::with_capacity(fields.len());
        for (_slot, fv) in fields {
            out.push(extdeps_fixture_entry_virtual_path(fv)?);
        }
        Some(out)
    }

    /// DB-10 (3a.2): read the compile-time value body attached to a
    /// declaration. Returns `None` for declarations without a
    /// value body (type aliases, bare type declarations, function
    /// declarations); returns `Some(&ValueBody)` for every `data`
    /// declaration — including `ValueBody::Unparsed` scaffolds
    /// whose body shape wasn't recognizable. Consumers that need
    /// an inhabitance-checked body must match on the variant
    /// directly and handle `Unparsed` explicitly; this accessor
    /// does not filter scaffolds.
    ///
    /// Used by dotted-path lowering (to inline record-field values
    /// at use sites) and by any future emission-time consumer that
    /// wants to render a declared constant directly.
    pub fn data_value_at(&self, id: DeclarationId) -> Option<&ValueBody> {
        self.declaration(id).value_body.as_ref()
    }

    pub fn port(&self, id: PortId) -> &Port {
        self.ports.get(&id).expect("PortId not in dag")
    }

    /// Option-returning variant for the .dag substrate accessor
    /// `port(d, id) -> DagPort?`. DB-14 wires this to the generated
    /// Rust via a carrier template; callers that need fail-closed
    /// carriers use this instead of `port`.
    pub fn port_opt(&self, id: &PortId) -> Option<&Port> {
        self.ports.get(id)
    }

    pub fn all_ports(&self) -> impl Iterator<Item = &Port> {
        self.ports.values()
    }

    pub fn diagnostics(&self) -> &DiagnosticTable {
        &self.diagnostics
    }

    // --- crate-private mutators: construction + inference ---

    pub(crate) fn alloc_node_id(&mut self) -> NodeId {
        let id = NodeId(self.next_node_id);
        self.next_node_id += 1;
        id
    }

    pub(crate) fn alloc_declaration_id(&mut self) -> DeclarationId {
        let id = DeclarationId(self.next_declaration_id);
        self.next_declaration_id += 1;
        id
    }

    pub(crate) fn alloc_port(&mut self, produced_by: Option<NodeId>) -> PortId {
        let id = PortId(self.next_port_id);
        self.next_port_id += 1;
        self.ports.insert(
            id,
            Port {
                id,
                state: PortState::Uninferred,
                produced_by,
            },
        );
        id
    }

    pub(crate) fn push_node(&mut self, behavior: Behavior) {
        debug_assert_eq!(
            behavior.id().index(),
            self.nodes.len(),
            "push_node out of sequence — topological invariant broken"
        );
        self.nodes.push(behavior);
    }

    pub(crate) fn push_cluster(&mut self, cluster: Cluster) -> ClusterId {
        let id = ClusterId(self.clusters.len() as u32);
        self.clusters.push(cluster);
        id
    }

    /// Mutable access to the computation-graph node vector. Scoped to
    /// the post-infer pattern resolution pass that rewrites
    /// `BranchPattern::UnresolvedVariant` entries into
    /// `ResolvedVariant(DeclarationId)` — a localized structural
    /// update, not a general mutation channel.
    pub(crate) fn nodes_mut(&mut self) -> &mut [Behavior] {
        &mut self.nodes
    }

    pub(crate) fn push_declaration(&mut self, declaration: Declaration) {
        debug_assert_eq!(
            declaration.id.index(),
            self.declarations.len(),
            "push_declaration out of sequence — allocation invariant broken"
        );
        self.declarations.push(declaration);
    }

    /// Mutable access to a declaration, scoped to lowering's second pass where
    /// Identifier atoms get their `resolved: Some(_)` slot filled in. No
    /// external mutator; resolution is an internal phase of lowering.
    pub(crate) fn declaration_mut(&mut self, id: DeclarationId) -> &mut Declaration {
        &mut self.declarations[id.index()]
    }

    pub(crate) fn patch_bind_value(&mut self, bind_id: NodeId, value: PortId) {
        let Some(Behavior::Bind(bind_node)) = self.nodes.get_mut(bind_id.index()) else {
            return;
        };
        bind_node.value = value;
    }

    pub(crate) fn set_port_producer(&mut self, port: PortId, produced_by: Option<NodeId>) {
        let Some(existing) = self.ports.get_mut(&port) else {
            return;
        };
        existing.produced_by = produced_by;
    }

    /// Transition a port from Uninferred to Resolved. Idempotent when the new
    /// type matches the existing Resolved type. Skips Unresolved ports — once
    /// Unresolved, stays so (otherwise the biconditional breaks).
    pub(crate) fn set_port_type(&mut self, id: PortId, ty: TypeShape) {
        if let Some(port) = self.ports.get_mut(&id) {
            if matches!(port.state, PortState::Unresolved) {
                return;
            }
            port.state = PortState::Resolved(ty);
        }
    }

    /// Enforced public API for transitioning a port to Unresolved. Atomically:
    /// marks the port AND records the diagnostic. There is NO other way to
    /// construct an Unresolved port state.
    ///
    /// Biconditional invariant, held by construction:
    ///   port.state == Unresolved  iff  diagnostics.contains(port_id)
    pub(crate) fn mark_unresolved(&mut self, port: PortId, diagnostic: Diagnostic) {
        self.mark_unresolved_with_attribution(
            port,
            diagnostic,
            DiagnosticAttribution::Unattributed,
        );
    }

    /// `mark_unresolved` carrying an explicit
    /// [`DiagnosticAttribution`]. The diagnostic is recorded against
    /// the same fail-closed biconditional as ordinary
    /// `mark_unresolved`; the attribution rides alongside on
    /// [`DiagnosticTable`] so consumers can dispatch bootstrap-vs-user
    /// origins without scanning `SourceSpan.file`.
    pub(crate) fn mark_unresolved_with_attribution(
        &mut self,
        port: PortId,
        diagnostic: Diagnostic,
        attribution: DiagnosticAttribution,
    ) {
        if let Some(p) = self.ports.get_mut(&port) {
            p.state = PortState::Unresolved;
        }
        self.diagnostics.insert(port, diagnostic, attribution);
    }

    /// Attach a diagnostic to the Dag without a pre-existing port anchor.
    /// Allocates a detached phantom port as the diagnostic carrier so the
    /// existing fail-closed biconditional still holds. Used by
    /// bootstrap / lowering for failures that don't have a natural
    /// PortId (unresolved declarations, tokenize/parse errors on
    /// bootstrap fixtures, duplicate top-level declarations, etc.).
    /// `compile_to_dag` surfaces these through `Err(CompileError::Semantic)`.
    ///
    /// Records [`DiagnosticAttribution::Unattributed`]. Bootstrap
    /// loaders attaching tokenize/parse/fixture failures against a
    /// substrate `bootstrap_authority` row should call
    /// [`Self::attach_bootstrap_diagnostic`] instead so verification
    /// consumers get a structural witness rather than a path string.
    pub(crate) fn attach_diagnostic(&mut self, diagnostic: Diagnostic) {
        let port = self.alloc_port(None);
        self.mark_unresolved(port, diagnostic);
    }

    /// Sibling of [`Self::attach_diagnostic`] for diagnostics raised
    /// while loading or patching a substrate `bootstrap_authority` row.
    /// Allocates the same detached phantom port (no fabricated producer
    /// node) but records
    /// [`DiagnosticAttribution::BootstrapAuthority(key)`] so consumers
    /// can recover bootstrap origin via witness identity instead of a
    /// `SourceSpan.file` string compare.
    pub(crate) fn attach_bootstrap_diagnostic(
        &mut self,
        authority: BootstrapAuthorityKey,
        diagnostic: Diagnostic,
    ) {
        let port = self.alloc_port(None);
        self.mark_unresolved_with_attribution(
            port,
            diagnostic,
            DiagnosticAttribution::BootstrapAuthority(authority),
        );
    }

    /// Populate `primitives` by reading the declaration table for the
    /// three canonical primitive roles. Called once at the end of
    /// `bootstrap()` after all std/ modules are loaded and cross-file
    /// references are resolved. Any role not found stays `None` —
    /// the bootstrap failure is already on the diagnostic table and
    /// downstream consumers surface the missing role through the
    /// ordinary channel.
    pub(crate) fn populate_primitive_cache(&mut self) {
        self.primitives.int = self
            .declaration_by_name("Int")
            .map(|d| TypeShape::new(d.id));
        self.primitives.bool = self
            .declaration_by_name("Bool")
            .map(|d| TypeShape::new(d.id));
        self.primitives.string = self
            .declaration_by_name("String")
            .map(|d| TypeShape::new(d.id));

        // Substrate marker resolution. Pulls each marker
        // declaration from `src/v3/spec/v3_l1.dag` by its declared
        // name and stores the typed handle. The lookup happens
        // once at bootstrap end; downstream consumers
        // (`lower_record_to_structural`, `emit_rust`) read the
        // typed handle via `bind_marker()` / `branch_marker()` /
        // etc. without any runtime name strings.
        self.substrate_markers.value = self.declaration_by_name("ValueBehavior").map(|d| d.id);
        self.substrate_markers.transform = self.declaration_by_name("Transform").map(|d| d.id);
        self.substrate_markers.branch = self.declaration_by_name("Branch").map(|d| d.id);
        self.substrate_markers.r#loop = self.declaration_by_name("Loop").map(|d| d.id);
        self.substrate_markers.bind = self.declaration_by_name("Bind").map(|d| d.id);
        self.substrate_markers.main = self.declaration_by_name("Main").map(|d| d.id);
        self.substrate_markers.declaration_ref =
            self.declaration_by_name("DeclarationRef").map(|d| d.id);

        // Realization meta-type resolution. Pulls each realization
        // category meta-type from `src/v3/spec/rust.dag` (and any
        // future per-target spec file) by its declared name and
        // stores the typed handle. Same one-time-at-bootstrap-end
        // pattern as substrate markers above. The lookup is the
        // single bridge from name space to id space; every
        // downstream consumer reads the typed accessor.
        self.realization_metas.type_realization =
            self.declaration_by_name("TypeRealization").map(|d| d.id);
        self.realization_metas.operator_realization = self
            .declaration_by_name("OperatorRealization")
            .map(|d| d.id);
        self.realization_metas.behavior_realization = self
            .declaration_by_name("BehaviorRealization")
            .map(|d| d.id);
        self.realization_metas.callable_realization = self
            .declaration_by_name("CallableRealization")
            .map(|d| d.id);
        self.realization_metas.type_instantiation_realization = self
            .declaration_by_name("TypeInstantiationRealization")
            .map(|d| d.id);
        self.realization_metas.pattern_realization =
            self.declaration_by_name("PatternRealization").map(|d| d.id);
        self.target_syntax.rust_language = self.declaration_by_name("rust_language").map(|d| d.id);
        self.target_syntax.rust_rendering =
            self.declaration_by_name("rust_rendering").map(|d| d.id);
        self.target_syntax.rust_execution_model = self
            .declaration_by_name("rust_execution_model")
            .map(|d| d.id);
        self.target_syntax.rust_source_filtering = self
            .declaration_by_name("rust_source_filtering")
            .map(|d| d.id);
        self.target_syntax.rust_execution_requirement = self
            .declaration_by_name("rust_execution_requirement")
            .map(|d| d.id);
        self.target_syntax.dag_model = self.declaration_by_name("dag_model").map(|d| d.id);
        self.target_syntax.go_language = self.declaration_by_name("go_language").map(|d| d.id);
        self.target_syntax.go_execution_model =
            self.declaration_by_name("go_execution_model").map(|d| d.id);
        self.target_syntax.go_source_filtering = self
            .declaration_by_name("go_source_filtering")
            .map(|d| d.id);
        self.target_syntax.go_execution_requirement = self
            .declaration_by_name("go_execution_requirement")
            .map(|d| d.id);
        self.target_syntax.python_language =
            self.declaration_by_name("python_language").map(|d| d.id);
        self.target_syntax.python_target = self.declaration_by_name("python_target").map(|d| d.id);
        self.target_syntax.python_source_filtering = self
            .declaration_by_name("python_source_filtering")
            .map(|d| d.id);
        self.target_syntax.python_execution_requirement = self
            .declaration_by_name("python_execution_requirement")
            .map(|d| d.id);
        self.populate_target_clean_emission_bindings();
        self.stdlib_types.list = self.declaration_by_name("List").map(|d| d.id);
        self.stdlib_types.partial_function =
            self.declaration_by_name("PartialFunction").map(|d| d.id);

        self.emit_anchors.ordered_ring = self.declaration_by_name("OrderedRing").map(|d| d.id);
        self.emit_anchors.abelian_group = self.declaration_by_name("AbelianGroup").map(|d| d.id);
        self.emit_anchors.substrate_accessor_binding = self
            .declaration_by_name("SubstrateAccessorBinding")
            .map(|d| d.id);
        self.emit_anchors.dag_type = self.declaration_by_name("Dag").map(|d| d.id);
        self.emit_anchors.std_list_fold = self.declaration_by_name("fold").map(|d| d.id);
        self.emit_anchors.rust_functions = self.declaration_by_name("rust_functions").map(|d| d.id);
        self.emit_anchors.method_emit_template =
            self.declaration_by_name("MethodEmitTemplate").map(|d| d.id);
        self.emit_anchors.method_template_contract = self
            .declaration_by_name("MethodTemplateContract")
            .map(|d| d.id);
        self.emit_anchors.concat_method = self.declaration_by_name("concat_method").map(|d| d.id);
        self.emit_anchors.length_method = self.declaration_by_name("length_method").map(|d| d.id);
        self.emit_anchors.fold_method = self.declaration_by_name("fold_method").map(|d| d.id);
        self.emit_anchors.is_empty_method =
            self.declaration_by_name("is_empty_method").map(|d| d.id);
        self.emit_anchors.filter_method = self.declaration_by_name("filter_method").map(|d| d.id);
        self.emit_anchors.flat_map_method =
            self.declaration_by_name("flat_map_method").map(|d| d.id);
        self.emit_anchors.any_method = self.declaration_by_name("any_method").map(|d| d.id);
        self.emit_anchors.all_method = self.declaration_by_name("all_method").map(|d| d.id);
        self.emit_anchors.map_method = self.declaration_by_name("map_method").map(|d| d.id);

        // `PatternBindingRule` variant resolution. Walks the
        // `std/clean_emission.dag` declaration's `Disj` variants
        // once at bootstrap end and caches the typed id per label.
        // Consumers in `emit_rust` / `emit_go` / (future)
        // `emit_python` read these typed handles instead of each
        // calling `named_variant_id` four times per contract parse.
        let mut pattern_binding_variants = PatternBindingRuleVariants::default();
        if let Some(parent) = self.declaration_by_name("PatternBindingRule") {
            if let TypeConnective::Disj { variants } = &parent.connective {
                for variant in variants {
                    match variant.label.as_str() {
                        "EmitBindingAlways" => {
                            pattern_binding_variants.emit_always = Some(variant.ty);
                        }
                        "EmitUnderscoreWhenUnused" => {
                            pattern_binding_variants.emit_underscore = Some(variant.ty);
                        }
                        "EmitPrefixedUnderscoreWhenUnused" => {
                            pattern_binding_variants.emit_prefixed = Some(variant.ty);
                        }
                        "NotApplicablePatternBinding" => {
                            pattern_binding_variants.not_applicable = Some(variant.ty);
                        }
                        _ => {}
                    }
                }
            }
        }
        self.pattern_binding_rule_variants = pattern_binding_variants;

        let mut variant_payload_field_access_variants =
            VariantPayloadFieldAccessRuleVariants::default();
        if let Some(parent) = self.declaration_by_name("VariantPayloadFieldAccessRule") {
            if let TypeConnective::Disj { variants } = &parent.connective {
                for variant in variants {
                    match variant.label.as_str() {
                        "AccessFromPayloadBinding" => {
                            variant_payload_field_access_variants.access_from_payload_binding =
                                Some(variant.ty);
                        }
                        "OverrideNamedFieldsAtBindingSite" => {
                            variant_payload_field_access_variants
                                .override_named_fields_at_binding_site = Some(variant.ty);
                        }
                        _ => {}
                    }
                }
            }
        }
        self.variant_payload_field_access_rule_variants = variant_payload_field_access_variants;

        // `VerifierOutputPolicy` variant resolution. Same shape as
        // the pattern-binding cache above: one walk of the Disj's
        // variants at bootstrap end, then every downstream consumer
        // (`post_emit_verifier::parse_output_policy` today; Lane 1e
        // generic walker tomorrow) dispatches on cached typed ids.
        // Resolving by variant.label at parse time would reintroduce
        // the same name-bridge pattern PR 2.5 removed from the
        // PatternBindingRule path.
        let mut verifier_policy_variants = VerifierOutputPolicyVariants::default();
        if let Some(parent) = self.declaration_by_name("VerifierOutputPolicy") {
            if let TypeConnective::Disj { variants } = &parent.connective {
                for variant in variants {
                    match variant.label.as_str() {
                        "IgnoreVerifierOutput" => {
                            verifier_policy_variants.ignore_output = Some(variant.ty);
                        }
                        "RequireEmptyStdout" => {
                            verifier_policy_variants.require_empty_stdout = Some(variant.ty);
                        }
                        "RequireEmptyStderr" => {
                            verifier_policy_variants.require_empty_stderr = Some(variant.ty);
                        }
                        "RequireEmptyStdoutAndStderr" => {
                            verifier_policy_variants.require_empty_stdout_and_stderr =
                                Some(variant.ty);
                        }
                        _ => {}
                    }
                }
            }
        }
        self.verifier_output_policy_variants = verifier_policy_variants;

        let mut callable_strategy_variants = CallableStrategyVariants::default();
        if let Some(parent) = self.declaration_by_name("CallableStrategy") {
            if let TypeConnective::Disj { variants } = &parent.connective {
                for variant in variants {
                    match variant.label.as_str() {
                        "ListEmpty" => {
                            callable_strategy_variants.list_empty = Some(variant.ty);
                        }
                        "ListSingleton" => {
                            callable_strategy_variants.list_singleton = Some(variant.ty);
                        }
                        "ListCons" => {
                            callable_strategy_variants.list_cons = Some(variant.ty);
                        }
                        "ListConcat" => {
                            callable_strategy_variants.list_concat = Some(variant.ty);
                        }
                        "ListLength" => {
                            callable_strategy_variants.list_length = Some(variant.ty);
                        }
                        "ListIsEmpty" => {
                            callable_strategy_variants.list_is_empty = Some(variant.ty);
                        }
                        "ListFold" => {
                            callable_strategy_variants.list_fold = Some(variant.ty);
                        }
                        "ListMap" => {
                            callable_strategy_variants.list_map = Some(variant.ty);
                        }
                        "ListFilter" => {
                            callable_strategy_variants.list_filter = Some(variant.ty);
                        }
                        "ListContains" => {
                            callable_strategy_variants.list_contains = Some(variant.ty);
                        }
                        _ => {}
                    }
                }
            }
        }
        self.callable_strategy_variants = callable_strategy_variants;

        let mut emit_model_variants = EmitModelVariants::default();
        if let Some(parent) = self.declaration_by_name("PatternStrategy") {
            if let TypeConnective::Disj { variants } = &parent.connective {
                for variant in variants {
                    if variant.label == "VectorList" {
                        emit_model_variants.pattern_strategy.vector_list = Some(variant.ty);
                    }
                }
            }
        }
        if let Some(parent) = self.declaration_by_name("FieldAccess") {
            if let TypeConnective::Disj { variants } = &parent.connective {
                for variant in variants {
                    match variant.label.as_str() {
                        "DirectField" => {
                            emit_model_variants.field_access.direct_field = Some(variant.ty);
                        }
                        "AccessorMethod" => {
                            emit_model_variants.field_access.accessor_method = Some(variant.ty);
                        }
                        _ => {}
                    }
                }
            }
        }
        if let Some(parent) = self.declaration_by_name("ParameterDisposition") {
            if let TypeConnective::Disj { variants } = &parent.connective {
                for variant in variants {
                    match variant.label.as_str() {
                        "Borrowed" => {
                            emit_model_variants.parameter_disposition.borrowed = Some(variant.ty);
                        }
                        "Consumed" => {
                            emit_model_variants.parameter_disposition.consumed = Some(variant.ty);
                        }
                        _ => {}
                    }
                }
            }
        }
        if let Some(parent) = self.declaration_by_name("MemoryModel") {
            if let TypeConnective::Disj { variants } = &parent.connective {
                for variant in variants {
                    match variant.label.as_str() {
                        "ValueOnly" => {
                            emit_model_variants.memory_model.value_only = Some(variant.ty);
                        }
                        "GarbageCollected" => {
                            emit_model_variants.memory_model.garbage_collected = Some(variant.ty);
                        }
                        "RefCounted" => {
                            emit_model_variants.memory_model.ref_counted = Some(variant.ty);
                        }
                        "OwnershipBased" => {
                            emit_model_variants.memory_model.ownership_based = Some(variant.ty);
                        }
                        _ => {}
                    }
                }
            }
        }
        if let Some(parent) = self.declaration_by_name("ScopeModel") {
            if let TypeConnective::Disj { variants } = &parent.connective {
                for variant in variants {
                    match variant.label.as_str() {
                        "LexicalScoping" => {
                            emit_model_variants.scope_model.lexical_scoping = Some(variant.ty);
                        }
                        "DynamicScoping" => {
                            emit_model_variants.scope_model.dynamic_scoping = Some(variant.ty);
                        }
                        _ => {}
                    }
                }
            }
        }
        if let Some(parent) = self.declaration_by_name("ReadStrategy") {
            if let TypeConnective::Disj { variants } = &parent.connective {
                for variant in variants {
                    match variant.label.as_str() {
                        "Borrow" => {
                            emit_model_variants.read_strategy.borrow = Some(variant.ty);
                        }
                        "PassByValue" => {
                            emit_model_variants.read_strategy.pass_by_value = Some(variant.ty);
                        }
                        _ => {}
                    }
                }
            }
        }
        if let Some(parent) = self.declaration_by_name("ConstructStrategy") {
            if let TypeConnective::Disj { variants } = &parent.connective {
                for variant in variants {
                    match variant.label.as_str() {
                        "CopyOrClone" => {
                            emit_model_variants.construct_strategy.copy_or_clone = Some(variant.ty);
                        }
                        "PassByValue" => {
                            emit_model_variants.construct_strategy.pass_by_value = Some(variant.ty);
                        }
                        _ => {}
                    }
                }
            }
        }
        if let Some(parent) = self.declaration_by_name("Mutability") {
            if let TypeConnective::Disj { variants } = &parent.connective {
                for variant in variants {
                    match variant.label.as_str() {
                        "Immutable" => {
                            emit_model_variants.mutability.immutable = Some(variant.ty);
                        }
                        "Mutable" => {
                            emit_model_variants.mutability.mutable = Some(variant.ty);
                        }
                        _ => {}
                    }
                }
            }
        }
        if let Some(parent) = self.declaration_by_name("Purity") {
            if let TypeConnective::Disj { variants } = &parent.connective {
                for variant in variants {
                    match variant.label.as_str() {
                        "Pure" => {
                            emit_model_variants.purity.pure = Some(variant.ty);
                        }
                        "Effectful" => {
                            emit_model_variants.purity.effectful = Some(variant.ty);
                        }
                        _ => {}
                    }
                }
            }
        }
        if let Some(parent) = self.declaration_by_name("Structure") {
            if let TypeConnective::Disj { variants } = &parent.connective {
                for variant in variants {
                    match variant.label.as_str() {
                        "ExplicitDAG" => {
                            emit_model_variants.structure.explicit_dag = Some(variant.ty);
                        }
                        "Arbitrary" => {
                            emit_model_variants.structure.arbitrary = Some(variant.ty);
                        }
                        _ => {}
                    }
                }
            }
        }
        if let Some(parent) = self.declaration_by_name("Iteration") {
            if let TypeConnective::Disj { variants } = &parent.connective {
                for variant in variants {
                    match variant.label.as_str() {
                        "Bounded" => {
                            emit_model_variants.iteration.bounded = Some(variant.ty);
                        }
                        "Unbounded" => {
                            emit_model_variants.iteration.unbounded = Some(variant.ty);
                        }
                        _ => {}
                    }
                }
            }
        }
        self.emit_model_variants = emit_model_variants;
    }

    fn populate_target_clean_emission_bindings(&mut self) {
        self.target_syntax.clean_emission_by_language.clear();
        let Some(binding_meta) = self
            .declaration_by_name("TargetCleanEmissionBinding")
            .map(|d| d.id)
        else {
            return;
        };
        let language_spec_meta = self.declaration_by_name("LanguageSpec").map(|d| d.id);
        let clean_emission_meta = self
            .declaration_by_name("CleanEmissionContract")
            .map(|d| d.id);
        let mut clean_emission_by_language = HashMap::new();
        let mut duplicate_languages = HashSet::new();
        let mut diagnostics = Vec::new();

        for declaration in &self.declarations {
            if declaration.meta_tag != Some(binding_meta) {
                continue;
            }
            let Some(ValueBody::Structural { fields }) = declaration.value_body.as_ref() else {
                diagnostics.push(malformed_target_clean_emission_binding(
                    declaration,
                    "must carry a structural value_body",
                ));
                continue;
            };
            let Some(language) = binding_reference_field(fields, "language") else {
                diagnostics.push(malformed_target_clean_emission_binding(
                    declaration,
                    "is missing `language: DeclarationRef`",
                ));
                continue;
            };
            let Some(clean_emission) = binding_reference_field(fields, "clean_emission") else {
                diagnostics.push(malformed_target_clean_emission_binding(
                    declaration,
                    "is missing `clean_emission: DeclarationRef`",
                ));
                continue;
            };
            if language_spec_meta
                .is_some_and(|meta| self.declaration(language).meta_tag != Some(meta))
            {
                diagnostics.push(malformed_target_clean_emission_binding(
                    declaration,
                    "`language` must reference a LanguageSpec declaration",
                ));
                continue;
            }
            if clean_emission_meta
                .is_some_and(|meta| self.declaration(clean_emission).meta_tag != Some(meta))
            {
                diagnostics.push(malformed_target_clean_emission_binding(
                    declaration,
                    "`clean_emission` must reference a CleanEmissionContract declaration",
                ));
                continue;
            }
            if duplicate_languages.contains(&language) {
                continue;
            }
            if clean_emission_by_language
                .insert(language, clean_emission)
                .is_some()
            {
                clean_emission_by_language.remove(&language);
                duplicate_languages.insert(language);
                diagnostics.push(duplicate_target_clean_emission_binding(
                    self,
                    declaration,
                    language,
                ));
            }
        }

        self.target_syntax.clean_emission_by_language = clean_emission_by_language;
        for diagnostic in diagnostics {
            self.attach_diagnostic(diagnostic);
        }
    }

    pub fn param_of(&self, member: NodeId, slot: usize) -> Option<ParamRef> {
        let bind = self.node(member).as_bind()?;
        bind.params.get(slot)?;
        Some(ParamRef { member, slot })
    }

    /// Sole Bool-validating constructor: returns a [`BoolPortRef`] when `port`
    /// resolves to the `Bool` primitive (Track 9 parity with [`Dag::param_of`]
    /// / [`Dag::as_transform_ref`]). Build [`BranchArm`] with [`BranchArm::new`].
    pub fn bool_port_of(&self, port: PortId) -> Option<BoolPortRef> {
        let bool_ty = self.bool_shape()?;
        let p = self.port_opt(&port)?;
        let ty = p.value_type()?;
        if *ty != bool_ty {
            return None;
        }
        Some(BoolPortRef { port })
    }

    /// Fail-closed wrapper for data-declaration lowering: on success returns the
    /// same witness as [`Dag::bool_port_of`]; on failure records
    /// [`Diagnostic::BranchConditionNotBool`] (C-8) and returns `None`.
    pub fn bool_port_for_branch_condition_or_diagnose(
        &mut self,
        port: PortId,
        condition_span: SourceSpan,
    ) -> Option<BoolPortRef> {
        if let Some(r) = self.bool_port_of(port) {
            return Some(r);
        }
        let actual_type = self.port_opt(&port).and_then(|p| p.value_type().cloned());
        self.attach_diagnostic(Diagnostic::BranchConditionNotBool {
            port,
            actual_type,
            span: condition_span,
            fixes: Vec::new(),
        });
        None
    }

    /// Convenience wrapper over [`Dag::bool_port_of`] + [`BranchArm::new`].
    pub fn branch_arm_of(&self, port: PortId, body: WorkflowEffect) -> Option<BranchArm> {
        let condition = self.bool_port_of(port)?;
        Some(BranchArm::new(condition, body))
    }

    pub fn as_transform_ref(&self, node: NodeId) -> Option<TransformRef> {
        self.node(node).as_transform()?;
        Some(TransformRef(node))
    }
}

impl Default for Dag {
    fn default() -> Self {
        Self::new()
    }
}

/// PB-1-e B4.4: the substrate `bootstrap_fixture_authority` carrier and
/// [`BOOTSTRAP_FIXTURE_PATH_KEYS`](crate::bootstrap::BOOTSTRAP_FIXTURE_PATH_KEYS)
/// must list the same virtual paths in the same order — the const is the regen
/// filter; the `.dag` declaration is runtime authority.
fn assert_bootstrap_fixture_paths_match_regen_keys(dag: &Dag) {
    let Some(paths) = dag.bootstrap_fixture_virtual_paths() else {
        panic!(
            "bootstrap snapshot must expose `bootstrap_fixture_authority` as a \
             structural `ValueBody` with well-formed `virtual_path` fields on each fixture \
             slot (missing declaration, non-structural body, or malformed fixture records). \
             Regenerate via `regen_bootstrap` after editing \
             `src/v3/std/extdeps_bootstrap_fixtures.dag`."
        );
    };
    if !paths
        .iter()
        .map(String::as_str)
        .eq(BOOTSTRAP_FIXTURE_PATH_KEYS.iter().copied())
    {
        panic!(
            "`BOOTSTRAP_FIXTURE_PATH_KEYS` must match `bootstrap_fixture_authority` \
             virtual_path fields in order.\n  substrate: {paths:?}\n  regen keys: {BOOTSTRAP_FIXTURE_PATH_KEYS:?}"
        );
    }
}

fn extdeps_fixture_entry_virtual_path(fv: &FieldValue) -> Option<String> {
    let FieldValue::Record(fields) = fv else {
        return None;
    };
    let (_, vp) = fields.iter().find(|(l, _)| l == "virtual_path")?;
    match vp {
        FieldValue::Literal(LiteralBits::String(s)) => Some(s.clone()),
        _ => None,
    }
}

fn binding_reference_field(fields: &[(String, FieldValue)], label: &str) -> Option<DeclarationId> {
    fields.iter().find_map(|(field_label, value)| {
        if field_label != label {
            return None;
        }
        match value {
            FieldValue::Reference(id) => Some(*id),
            _ => None,
        }
    })
}

fn malformed_target_clean_emission_binding(declaration: &Declaration, detail: &str) -> Diagnostic {
    Diagnostic::ResolveError {
        name: format!(
            "TargetCleanEmissionBinding `{}` {detail}",
            declaration
                .name
                .as_deref()
                .unwrap_or("<anonymous target clean emission binding>")
        ),
        span: declaration.span.clone(),
        fixes: Vec::new(),
    }
}

fn duplicate_target_clean_emission_binding(
    dag: &Dag,
    declaration: &Declaration,
    language: DeclarationId,
) -> Diagnostic {
    let language_name = dag
        .declaration(language)
        .name
        .as_deref()
        .unwrap_or("<anonymous language>");
    Diagnostic::ResolveError {
        name: format!(
            "TargetCleanEmissionBinding `{}` duplicates the clean-emission authority for language `{language_name}`",
            declaration
                .name
                .as_deref()
                .unwrap_or("<anonymous target clean emission binding>")
        ),
        span: declaration.span.clone(),
        fixes: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn call_pattern_from_relations_fails_closed_for_mixed_unknown_and_preserved_evidence() {
        assert_eq!(
            call_pattern_from_relations(&[
                SubValueRelation::PreservedValue,
                SubValueRelation::SubValueUnknown
            ]),
            None,
            "mixed unknown + preserved evidence must not fabricate SameArgumentCall"
        );
    }

    #[test]
    fn workflow_root_zero_bind_returns_no_root() {
        // V3 surface syntax always lowers each top-level decl to a
        // Bind, so the zero-Bind case is structurally unreachable from
        // `compile_to_dag` fixtures. The α path's `NoRoot` arm is
        // defensive-only at the substrate boundary; this unit test
        // exercises it via the crate-private `Dag::empty()` constructor.
        let dag = Dag::empty();
        let root = dag.workflow_root_port();
        assert_eq!(
            root,
            WorkflowRoot::NoRoot,
            "Dag with no Bind behaviors must fail closed with NoRoot"
        );
    }

    fn binding_fields(language: DeclarationId, clean_emission: DeclarationId) -> ValueBody {
        ValueBody::Structural {
            fields: vec![
                ("language".to_string(), FieldValue::Reference(language)),
                (
                    "clean_emission".to_string(),
                    FieldValue::Reference(clean_emission),
                ),
            ],
        }
    }

    /// Ratchet: [`Dag::populate_primitive_cache`] must resolve every
    /// [`EmitAnchorCache`] role on the bootstrapped std/spec surface. If a
    /// substrate name moves or a fixture omits a declaration, accessors
    /// flip to `None` and emit paths lose typed anchors (ROADMAP P3).
    #[test]
    fn post_bootstrap_declaration_append_begin_is_zero_on_empty() {
        let dag = Dag::empty();
        assert_eq!(dag.post_bootstrap_declaration_append_begin(), 0);
    }

    #[test]
    fn dag_new_bootstrap_declaration_ids_are_below_append_begin() {
        let dag = Dag::new();
        let begin = dag.post_bootstrap_declaration_append_begin();
        assert_eq!(begin as usize, dag.declarations().len());
        for d in dag.declarations() {
            assert!(
                d.id.raw() < begin,
                "expected bootstrap id {:?} < {}",
                d.id,
                begin
            );
            assert!(
                !dag.is_runtime_appended_declaration(d.id),
                "bootstrap decl {:?} must not classify as runtime-appended",
                d.id
            );
        }
    }

    #[test]
    fn runtime_appended_declaration_is_detected() {
        let mut dag = Dag::new();
        let begin = dag.post_bootstrap_declaration_append_begin();
        let binding_meta = dag
            .declaration_by_name("TargetCleanEmissionBinding")
            .expect("binding meta")
            .id;
        let rust_language = dag.rust_language_spec().expect("rust language");
        let new_id = dag.alloc_declaration_id();
        assert_eq!(new_id.raw(), begin);
        dag.push_declaration(Declaration {
            id: new_id,
            name: Some("append_boundary_probe_decl".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(binding_meta)),
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: Some(binding_meta),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(binding_fields(
                rust_language,
                dag.go_clean_emission_spec().expect("go clean emission"),
            )),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("append_boundary_probe.v3", 0, 1),
        });
        assert!(new_id.raw() >= begin);
        assert!(dag.is_runtime_appended_declaration(new_id));
    }

    #[test]
    fn emit_anchor_cache_populated_after_bootstrap() {
        let dag = Dag::new();
        assert!(
            dag.ordered_ring_decl().is_some(),
            "OrderedRing algebra anchor"
        );
        assert!(
            dag.substrate_accessor_binding_meta().is_some(),
            "SubstrateAccessorBinding meta anchor"
        );
        assert!(dag.dag_type_decl().is_some(), "Dag graph type anchor");
        assert!(dag.std_list_fold_decl().is_some(), "std.list fold anchor");
        assert!(
            dag.rust_functions_syntax_decl().is_some(),
            "rust_functions syntax anchor"
        );
        assert!(
            dag.method_emit_template_decl().is_some(),
            "MethodEmitTemplate anchor"
        );
        assert!(
            dag.method_template_contract_decl().is_some(),
            "MethodTemplateContract anchor"
        );
        assert!(dag.fold_method_decl().is_some(), "fold_method anchor");
        assert!(dag.concat_method_decl().is_some(), "concat_method anchor");
        assert!(dag.length_method_decl().is_some(), "length_method anchor");
        assert!(
            dag.is_empty_method_decl().is_some(),
            "is_empty_method anchor"
        );
    }

    #[test]
    fn callable_strategy_variants_populated_after_bootstrap() {
        let dag = Dag::new();
        let variants = dag.callable_strategy_variants();
        assert!(variants.list_empty.is_some(), "CallableStrategy.ListEmpty");
        assert!(
            variants.list_singleton.is_some(),
            "CallableStrategy.ListSingleton"
        );
        assert!(variants.list_cons.is_some(), "CallableStrategy.ListCons");
        assert!(
            variants.list_concat.is_some(),
            "CallableStrategy.ListConcat"
        );
        assert!(
            variants.list_length.is_some(),
            "CallableStrategy.ListLength"
        );
        assert!(
            variants.list_is_empty.is_some(),
            "CallableStrategy.ListIsEmpty"
        );
        assert!(variants.list_fold.is_some(), "CallableStrategy.ListFold");
        assert!(variants.list_map.is_some(), "CallableStrategy.ListMap");
        assert!(
            variants.list_filter.is_some(),
            "CallableStrategy.ListFilter"
        );
        assert!(
            variants.list_contains.is_some(),
            "CallableStrategy.ListContains"
        );
    }

    #[test]
    fn emit_model_variants_populated_after_bootstrap() {
        let dag = Dag::new();
        let variants = dag.emit_model_variants();
        assert!(
            variants.pattern_strategy.vector_list.is_some(),
            "PatternStrategy.VectorList"
        );
        assert!(
            variants.field_access.direct_field.is_some(),
            "FieldAccess.DirectField"
        );
        assert!(
            variants.field_access.accessor_method.is_some(),
            "FieldAccess.AccessorMethod"
        );
        assert!(
            variants.parameter_disposition.borrowed.is_some(),
            "ParameterDisposition.Borrowed"
        );
        assert!(
            variants.parameter_disposition.consumed.is_some(),
            "ParameterDisposition.Consumed"
        );
        assert!(
            variants.memory_model.value_only.is_some(),
            "MemoryModel.ValueOnly"
        );
        assert!(
            variants.memory_model.garbage_collected.is_some(),
            "MemoryModel.GarbageCollected"
        );
        assert!(
            variants.memory_model.ref_counted.is_some(),
            "MemoryModel.RefCounted"
        );
        assert!(
            variants.memory_model.ownership_based.is_some(),
            "MemoryModel.OwnershipBased"
        );
        assert!(
            variants.scope_model.lexical_scoping.is_some(),
            "ScopeModel.LexicalScoping"
        );
        assert!(
            variants.scope_model.dynamic_scoping.is_some(),
            "ScopeModel.DynamicScoping"
        );
        assert!(
            variants.read_strategy.borrow.is_some(),
            "ReadStrategy.Borrow"
        );
        assert!(
            variants.read_strategy.pass_by_value.is_some(),
            "ReadStrategy.PassByValue"
        );
        assert!(
            variants.construct_strategy.copy_or_clone.is_some(),
            "ConstructStrategy.CopyOrClone"
        );
        assert!(
            variants.construct_strategy.pass_by_value.is_some(),
            "ConstructStrategy.PassByValue"
        );
        assert!(
            variants.mutability.immutable.is_some(),
            "Mutability.Immutable"
        );
        assert!(variants.mutability.mutable.is_some(), "Mutability.Mutable");
        assert!(variants.purity.pure.is_some(), "Purity.Pure");
        assert!(variants.purity.effectful.is_some(), "Purity.Effectful");
        assert!(
            variants.structure.explicit_dag.is_some(),
            "Structure.ExplicitDAG"
        );
        assert!(
            variants.structure.arbitrary.is_some(),
            "Structure.Arbitrary"
        );
        assert!(variants.iteration.bounded.is_some(), "Iteration.Bounded");
        assert!(
            variants.iteration.unbounded.is_some(),
            "Iteration.Unbounded"
        );
    }

    #[test]
    fn bridge_mark_bootstrap_secret_nominal_opacity_retired() {
        fn assert_secret_marked(dag: Dag, label: &str) {
            let secret = dag
                .declaration_by_name("Secret")
                .unwrap_or_else(|| panic!("{label} must include std Secret"));
            assert!(
                secret.nominal_opacity.is_some(),
                "{label} must carry Secret nominal opacity in the generated snapshot"
            );
        }

        assert_secret_marked(
            bootstrap_std_generated::bootstrapped_std_fixture_dag(),
            "std bootstrap snapshot",
        );
        assert_secret_marked(
            bootstrap_generated::bootstrapped_fixture_dag(),
            "full bootstrap snapshot",
        );
        assert_secret_marked(
            bootstrap_generated_without_parse_surface::bootstrapped_fixture_without_parse_surface_dag(
            ),
            "bootstrap-without-parse-surface snapshot",
        );
    }

    #[test]
    fn bootstrap_secret_is_nominal_opaque() {
        let dag = Dag::new();
        let secret = dag
            .declaration_by_name("Secret")
            .expect("bootstrap fixture must include std Secret");

        assert!(
            secret.nominal_opacity.is_some(),
            "Secret must carry nominal opacity from std source authority"
        );
    }

    #[test]
    #[should_panic(expected = "generated bootstrap invariant violation")]
    fn generated_bootstrap_rejects_user_defined_body_that_is_not_a_bind() {
        let mut dag = bootstrap_generated::bootstrapped_fixture_dag();
        let non_bind = dag
            .nodes
            .iter()
            .find_map(|behavior| {
                if matches!(behavior, Behavior::Bind(_)) {
                    None
                } else {
                    Some(behavior.id())
                }
            })
            .expect("generated bootstrap fixture should include a non-Bind behavior");
        let declaration = dag
            .declarations
            .iter_mut()
            .find(|declaration| {
                matches!(
                    &declaration.connective,
                    TypeConnective::Arrow {
                        body: ArrowBody::UserDefined(_),
                        ..
                    }
                )
            })
            .expect("generated bootstrap fixture should include a user-defined arrow body");
        let TypeConnective::Arrow { body, .. } = &mut declaration.connective else {
            unreachable!("declaration was selected by arrow body shape")
        };
        *body = ArrowBody::UserDefined(BindNodeId::new_unchecked(non_bind));

        dag.finalize_runtime_bootstrap_from_generated_snapshot(
            RuntimeBootstrapFixtureKind::FullExtdepsPipelineSnapshot,
        );
    }

    #[test]
    fn malformed_target_clean_emission_binding_fails_closed() {
        let mut dag = Dag::new();
        let binding = dag
            .declaration_by_name("rust_clean_emission_binding")
            .expect("rust binding exists")
            .id;
        let rust_language = dag.rust_language_spec().expect("rust language");
        dag.declaration_mut(binding).value_body =
            Some(binding_fields(rust_language, rust_language));

        dag.populate_primitive_cache();

        assert!(
            dag.target_syntax_bundle_for_language(rust_language)
                .is_none(),
            "malformed binding should not populate a target authority bundle"
        );
        assert!(
            dag.rust_clean_emission_spec().is_none(),
            "target-specific clean-emission accessor must also fail closed on malformed bindings"
        );
        assert!(
            dag.diagnostics().iter().any(|(_, diagnostic)| matches!(
                diagnostic,
                Diagnostic::ResolveError { name, .. }
                    if name.contains("rust_clean_emission_binding")
                        && name.contains("CleanEmissionContract")
            )),
            "expected malformed TargetCleanEmissionBinding diagnostic, got {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        );
    }

    #[test]
    fn duplicate_target_clean_emission_binding_fails_closed() {
        let mut dag = Dag::new();
        let binding_meta = dag
            .declaration_by_name("TargetCleanEmissionBinding")
            .expect("binding meta exists")
            .id;
        let rust_language = dag.rust_language_spec().expect("rust language");
        let go_clean_emission = dag.go_clean_emission_spec().expect("go clean emission");
        let duplicate = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: duplicate,
            name: Some("duplicate_rust_clean_emission_binding".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(binding_meta)),
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: Some(binding_meta),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(binding_fields(rust_language, go_clean_emission)),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("duplicate_binding_test.v3", 0, 1),
        });

        dag.populate_primitive_cache();

        assert!(
            dag.target_syntax_bundle_for_language(rust_language)
                .is_none(),
            "duplicate language bindings should remove the ambiguous authority"
        );
        assert!(
            dag.rust_clean_emission_spec().is_none(),
            "target-specific clean-emission accessor must project the same ambiguous authority as None"
        );
        assert!(
            dag.diagnostics().iter().any(|(_, diagnostic)| matches!(
                diagnostic,
                Diagnostic::ResolveError { name, .. }
                    if name.contains("duplicate_rust_clean_emission_binding")
                        && name.contains("duplicates the clean-emission authority")
            )),
            "expected duplicate TargetCleanEmissionBinding diagnostic, got {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        );
    }

    #[test]
    fn cardinality_idempotent_target_peels_empty_instantiation_alias() {
        let mut dag = Dag::new();
        dag.populate_primitive_cache();
        let int_decl = dag.int_shape().expect("bootstrap Int").declaration;
        let opt_decl = dag.alloc_cardinality_decl(
            int_decl,
            CardinalityBound::AtMostOne,
            SourceSpan::new("cardinality_alias_peel_test", 0, 0),
        );
        let alias_decl = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: alias_decl,
            name: Some("Alias".to_string()),
            connective: TypeConnective::Instantiation {
                template: opt_decl,
                arguments: Vec::new(),
            },
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("cardinality_alias_peel_test", 0, 0),
        });

        assert_eq!(
            cardinality_idempotent_target(&dag, alias_decl, CardinalityBound::AtMostOne),
            Some(opt_decl),
            "Alias = Opt (Instantiation with empty args) should peel to Opt before idempotence"
        );
    }

    #[test]
    fn cardinality_idempotent_target_peels_chained_instantiation_aliases() {
        let mut dag = Dag::new();
        dag.populate_primitive_cache();
        let int_decl = dag.int_shape().expect("bootstrap Int").declaration;
        let int_opt_decl = dag.alloc_cardinality_decl(
            int_decl,
            CardinalityBound::AtMostOne,
            SourceSpan::new("cardinality_chain_peel_test", 0, 0),
        );
        // type Alias = Int?   →  empty-arg alias to the `Int?` declaration
        let alias_decl = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: alias_decl,
            name: Some("Alias".to_string()),
            connective: TypeConnective::Instantiation {
                template: int_opt_decl,
                arguments: Vec::new(),
            },
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("cardinality_chain_peel_test", 0, 0),
        });
        // type Wrap = Alias  →  second empty-arg alias
        let wrap_decl = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: wrap_decl,
            name: Some("Wrap".to_string()),
            connective: TypeConnective::Instantiation {
                template: alias_decl,
                arguments: Vec::new(),
            },
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("cardinality_chain_peel_test", 0, 0),
        });

        assert_eq!(
            cardinality_idempotent_target(&dag, wrap_decl, CardinalityBound::AtMostOne),
            Some(int_opt_decl),
            "Wrap = Alias; Alias = Int? should peel through both Instantiations to the \
             canonical `Int?` decl (recursive alias chain, not one step)"
        );
    }

    #[test]
    fn cardinality_idempotence_does_not_collapse_non_at_most_one_inner_bound() {
        let mut dag = Dag::new();
        dag.populate_primitive_cache();
        let int_decl = dag.int_shape().expect("bootstrap Int").declaration;
        let exact_two = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: exact_two,
            name: None,
            connective: TypeConnective::Cardinality(
                CardinalityPayload::new_unchecked_bypassing_idempotence(
                    int_decl,
                    CardinalityBound::Exact(2),
                ),
            ),
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("cardinality_exact_spoof_test", 0, 0),
        });

        assert_eq!(
            cardinality_idempotent_target(&dag, exact_two, CardinalityBound::AtMostOne),
            None,
            "AtMostOne over Exact(2) is optional fixed-cardinality, not nested optional"
        );

        let optional_exact_two = dag.alloc_cardinality_decl(
            exact_two,
            CardinalityBound::AtMostOne,
            SourceSpan::new("cardinality_exact_spoof_test", 0, 0),
        );
        let TypeConnective::Cardinality(payload) = &dag.declaration(optional_exact_two).connective
        else {
            panic!("optional Exact(2) should remain a Cardinality declaration");
        };
        assert_eq!(payload.element(), exact_two);
        assert_eq!(payload.bound(), CardinalityBound::AtMostOne);
    }

    /// Cementing test for issue #2463: `resolve_producer_lookup` must
    /// discriminate the five variants (`NoProducer`, `Found`,
    /// `MissingPort`, `MissingNode`, `BindCycle`) so consumers can
    /// fail-closed on malformed substrate per INVARIANTS P3.
    /// `resolve_producer_opt` (compat wrapper) collapses the four
    /// miss shapes (legitimate `NoProducer` plus three malformed) into
    /// `None`; the typed surface preserves the distinction.
    #[test]
    fn resolve_producer_lookup_discriminates_malformed_substrate() {
        let dag = Dag::new();
        // MissingPort: a `PortId` past the allocated range cannot
        // resolve. The compat wrapper hides this as `None`; the typed
        // surface surfaces it as `MissingPort`.
        let bogus = PortId::test_raw(u32::MAX);
        assert!(
            matches!(
                dag.resolve_producer_lookup(&bogus),
                ProducerLookup::MissingPort { .. }
            ),
            "out-of-range PortId must surface as MissingPort, not collapse to NoProducer"
        );
        assert!(
            dag.resolve_producer_opt(&bogus).is_none(),
            "compat wrapper collapses MissingPort to None"
        );
    }
}
