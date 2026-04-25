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
// is a DeclarationId into the Declaration table; ArrowBody::UserDefined holds a NodeId
// back into the computation substrate. There is no name-based dispatch at the substrate
// layer — operators like `+` resolve to the `add` field of an inhabited algebra
// declaration during inference (via M1_DESIGN §8.9), not at parse time.
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

use crate::diagnostics::{Diagnostic, DiagnosticTable, SourceSpan};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PortId(u32);

impl PortId {
    pub fn raw(self) -> u32 {
        self.0
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

/// A type-system declaration. The unit of the type substrate.
///
/// Every named declaration (primitive, algebra, user type, type alias) lives in
/// `Dag.declarations` under a stable DeclarationId. Anonymous declarations (the
/// inner types of Cardinality bounds, Arrow inputs, etc.) also live here; only the
/// `name` field distinguishes them.
///
/// `type_params`, `meta_tag`, `specialization_parent`, `inhabits`, and
/// `value_body` are separate edges with distinct semantics:
/// - `type_params`: the canonical carrier for generic parameters declared on
///   `type Foo<T, U> { ... }` / sum / alias items. Each entry is a
///   DeclarationId whose connective is `Atom(TypeParam(name))`. Keeping type
///   params off the connective axis means `Conj.children` stays pure record
///   fields and `Disj.variants` stays pure sum alternatives — type params no
///   longer share a slot with either. Empty for most declarations.
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
    pub span: SourceSpan,
}

/// Value-body shape for `data foo: T = { body }` declarations. Two
/// variants at M1(3) PR-B-unwind:
///
/// - **`Unparsed(SourceSpan)`** — the parser could not lower the
///   body (it isn't a record literal shape the M1(3) parser
///   recognizes). The body's source span is preserved so M2+
///   parser extensions (list literals, nested records, variant
///   constructors) can reach in later. User-range declarations
///   carrying `Unparsed` are rejected by
///   `reject_user_unparsed_scaffolds`; bootstrap-range declarations
///   tolerate it so std/*.dag files whose data bodies still use
///   unsupported shapes continue to load.
///
/// - **`Structural { fields }`** — the body parsed as a record
///   literal and lowering ran inhabitance checking against the
///   declared type. Each field is a `(String, FieldValue)` pair
///   where `FieldValue` is either a scalar literal or a typed
///   declaration reference (the unwind shape — PR-B's initial
///   payload was `Vec<(String, LiteralBits)>` and it forced
///   downstream consumers like `emit_rust.rs` to dispatch on
///   string keys, regenerating the name-bridge pattern that
///   M1(2.7) had eliminated at the inference layer).
///
/// **Dissolution ledger** — mixed-lifecycle coproduct. `Unparsed`
/// is the bounded scaffold (named dissolution trigger: M2+ parser
/// extensions close class-5 gap #3); `Structural` is the
/// structurally-grounded form. When the M2+ parser catches up to
/// nested records / list literals / map literals, those non-record
/// shapes currently landing in `Unparsed` move to `Structural`
/// (probably via an extended `FieldValue` payload — see below),
/// and `Unparsed` is removed via a reverse substrate-extension PR.
///
/// 4-pattern check on `Structural`:
/// - Pattern 1 (fact placement): fails. The inline `(label,
///   FieldValue)` list is a data-item-specific record-construction
///   fact with no natural home on the other substrate edges.
/// - Pattern 2 (variant-is-data): fails. `Structural`'s payload is
///   structurally distinct from `Unparsed`'s source span.
/// - Pattern 3 (algebraic form): fails. The two variants represent
///   two parser-boundary states (structurally lowered vs
///   scaffolded), not two points in a single algebra.
/// - Pattern 4 (dimensional): fails. No shared coordinate space.
///
/// Verdict: `Structural` is terminal-at-current-scope, with the
/// `FieldValue` enum carrying the literal-vs-reference distinction
/// internally. Future extensions (port-carried values, nested
/// records) grow `FieldValue`, not `ValueBody`. Bounded by the
/// Scaffold Boundaries invariant in `INVARIANTS.md`.
#[derive(Debug, Clone)]
pub enum ValueBody {
    /// The body exists in source at the given span but is not yet
    /// lowered to a value sub-DAG. The body's shape (record / map /
    /// list / variant literal) awaits M2+ parser extension.
    Unparsed(SourceSpan),
    /// The body parsed as a record literal and was inhabitance-
    /// checked against the declared type. Each field holds a
    /// recursively structural `FieldValue`; the label matches a
    /// field on the type's Conj children.
    Structural { fields: Vec<(String, FieldValue)> },
    /// Scalar-valued data declaration: `data answer: Int = 42`.
    /// Carries `LiteralBits` directly (Int / Bool / String) —
    /// NOT a full `FieldValue`. This is deliberate:
    ///
    /// - `FieldValue::Record { .. }` at the top level is already
    ///   representable as `ValueBody::Structural { fields }`;
    ///   allowing `ValueBody::Scalar(FieldValue::Record(..))` would
    ///   make illegal/overlapping states representable (two distinct
    ///   encodings of the same top-level record body). Rejected.
    /// - `FieldValue::Reference`, `List`, `Variant` as top-level
    ///   data bodies are out of scope for DB-10's acceptance
    ///   (scalar + structural record only). When those shapes
    ///   become parseable at the top level, grow `ValueBody` with
    ///   a new variant — do not widen `Scalar` to swallow them.
    ///
    /// DB-10 (Lane 3 Stage 3a.2) — `compiler.dag` needs compile-time
    /// scalar constants; previously the parser rejected non-
    /// `{`-shaped RHS, so scalar `data` declarations could not exist.
    Scalar(LiteralBits),
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
///   `List(Vec<...>)`, and `Variant { .. }` inhabit different
///   structural spaces.
/// - Pattern 3 (algebraic form): fails. The five variants are not
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
    /// Structural sum constructor with positional payload fields.
    /// The exact variant child declaration is preserved explicitly
    /// so downstream consumers can recover variant identity without
    /// string bridges.
    Variant {
        constructor: DeclarationId,
        payload: Vec<FieldValue>,
    },
}

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
    Cardinality {
        element: DeclarationId,
        bound: CardinalityBound,
    },
    /// Specialization of a parameterized template with concrete template arguments.
    /// For pure aliases like `Int = OrderedRing<Word64>`, Int's connective IS
    /// Instantiation directly — inhabitance collapses into this form. See
    /// M1_DESIGN.md §Q0, §Q1.
    Instantiation {
        template: DeclarationId,
        arguments: Vec<TemplateArgument>,
    },
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

// Terminal literal/cardinality/template/port-state mirrors are generated
// from `std/substrate.dag`; keep the substrate authority there, not here.
include!("dag_scalar_generated.rs");

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
///      `ArrowBody::UserDefined(bind_id)` before the Dag is frozen
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
///   files where the body contains match/pipe/lambda/ etc. **`pipeline.dag`
///   per-stage fns (case 2a)** parse as `FnExternalBody` → `Unparsed`, then
///   bootstrap rewrites those Arrow bodies to `ExternalRealization` before
///   inference — so `Unparsed` does not persist for those stages in a
///   bootstrapped DAG. **`fn compile` (case 2c)** has no `PipelineStageBinding`:
///   **`Unparsed` persists** on its Arrow body. Pipeline ordering authority is
///   the declaration order of the `PipelineStageBinding` records in the Dag —
///   `ordered_pipeline_stages` reads that structural order directly at runtime.
///   The `compile` body is retained as a second surface-level expression of
///   the same ordering and `ordered_pipeline_stages` fail-closes on any drift
///   between the two (`reconcile_with_compile_body`) so the bindings remain
///   the single runtime authority without silently diverging from the
///   orchestrator surface. The `compile` body is the human-readable pipeline
///   contract that the binding records satisfy — a reader sees the pipeline
///   in one glance as `{ parse; lower; infer; ... }` rather than
///   reconstructing it from a binding table. The bindings are the runtime
///   authority; the body is the surface the bindings commit to; the
///   fail-closed reconcile keeps that contract honest (P3) without letting
///   the body become a second runtime source (P2). This is **bridge shape**,
///   not terminal: two authored carriers kept consistent by reconcile. PR #637
///   narrowed the prior body-span-as-authority shape; the bridge itself
///   remains scheduled debt (see `docs/history/roadmap-scheduled-deletions.md`
///   case 2c) until derivation collapses the two carriers to a single
///   authored source — e.g., regen emits the `compile` body from binding
///   declaration order. The signature still flows forward through the
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
/// (structural binding order is), but two authored carriers still require
/// `reconcile_with_compile_body` to stay consistent, so 2c remains scheduled
/// debt with its own dissolution trigger (derivation, not the M2 parser
/// milestone). User-range
/// `Unparsed` stays gated (R14).
#[derive(Debug, Clone)]
pub enum ArrowBody {
    /// User-defined function. NodeId is the root of a sub-DAG of L1 behavior
    /// nodes in `Dag.nodes`. Inference walks the sub-DAG and checks the body
    /// against the declared inputs/output.
    UserDefined(NodeId),
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

pub fn evidence_rank(evidence: DescentEvidence) -> i64 {
    match evidence {
        DescentEvidence::Strict => 2,
        DescentEvidence::NonIncreasing => 1,
        DescentEvidence::DescentUnknown => 0,
    }
}

pub fn merge_evidence(a: DescentEvidence, b: DescentEvidence) -> DescentEvidence {
    match a {
        DescentEvidence::Strict => match b {
            DescentEvidence::Strict => DescentEvidence::Strict,
            DescentEvidence::NonIncreasing => DescentEvidence::NonIncreasing,
            DescentEvidence::DescentUnknown => DescentEvidence::DescentUnknown,
        },
        DescentEvidence::NonIncreasing => match b {
            DescentEvidence::Strict => DescentEvidence::NonIncreasing,
            DescentEvidence::NonIncreasing => DescentEvidence::NonIncreasing,
            DescentEvidence::DescentUnknown => DescentEvidence::DescentUnknown,
        },
        DescentEvidence::DescentUnknown => DescentEvidence::DescentUnknown,
    }
}

pub fn join_evidence(a: DescentEvidence, b: DescentEvidence) -> DescentEvidence {
    match a {
        DescentEvidence::DescentUnknown => b,
        DescentEvidence::NonIncreasing => match b {
            DescentEvidence::Strict => DescentEvidence::Strict,
            DescentEvidence::NonIncreasing => DescentEvidence::NonIncreasing,
            DescentEvidence::DescentUnknown => DescentEvidence::NonIncreasing,
        },
        DescentEvidence::Strict => DescentEvidence::Strict,
    }
}

/// Legacy E-T helper name retained for carrier API parity.
///
/// Fail-closed behavior means no unary helper may fabricate `Strict` from
/// weaker evidence; strict promotion requires a separate structural witness.
///
/// P5 bridge: identifier suggests promotion; this mirror is identity on the
/// three `DescentEvidence` variants today (same fail-closed contract as
/// `std.termination`). Dissolution: rename to e.g. `evidence_passthrough_preserving_strict`
/// and/or remove `v2.compiler.complexity` call sites when parser progress threads
/// `Strict` at the witness site.
pub fn promote_to_strict(evidence: DescentEvidence) -> DescentEvidence {
    evidence
}

pub fn optional_evidence_meet(
    a: Option<DescentEvidence>,
    b: Option<DescentEvidence>,
) -> Option<DescentEvidence> {
    match a {
        None => b,
        Some(va) => match b {
            None => a,
            Some(vb) => Some(merge_evidence(va, vb)),
        },
    }
}

pub fn map_evidence_merge_at(
    mut base: HashMap<String, DescentEvidence>,
    key: String,
    new_val: DescentEvidence,
) -> HashMap<String, DescentEvidence> {
    let merged = match base.get(&key).copied() {
        Some(existing) => merge_evidence(existing, new_val),
        None => new_val,
    };
    base.insert(key, merged);
    base
}

// Continuation: Rust execution mirror for `src/v3/std/computation.dag` (Lane E-C).
// Same staging contract as the termination mirror above (`ArrowBody::Unparsed` std bodies).
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
/// structure. It currently proves arithmetic evidence only for direct self-call
/// arguments. Other callable edges are still represented, but their argument
/// positions fail closed to [`SubValueRelation::SubValueUnknown`] until the
/// producer can prove stronger mutual-recursive or cross-call facts.
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
    dag.node_opt(bind_id)?.as_bind()
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

    SubValueRelation::SubValueUnknown
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
    match dag.resolve_producer_opt(&port)? {
        Behavior::Value(value) => match &value.data {
            LiteralBits::Int(n) => Some(*n),
            _ => None,
        },
        _ => None,
    }
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

/// 🟡 SCAFFOLD — `AlgebraProfile` coproduct (`docs/modeling-discipline.md` §4).
///
/// Closed seven-variant mirror of `dsl/std/algebra.dag` `kernel_algebra_profile` while the
/// table is still `ArrowBody::Unparsed`. **Named trigger:** evaluated std bodies / read the
/// table from `.dag` (see [`kernel_algebra_profile`] below). **Ledger:** P2 ratchet
/// `m2_substrate_inhabitance_test::v3_kernel_algebra_profile_mirror_matches_v2_stage0_authority`.
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
/// `v2_compiler::std_algebra::kernel_algebra_profile` is regenerated from that
/// block; this match is a transitional Rust mirror while bootstrap still carries
/// the table as [`ArrowBody::Unparsed`].
///
/// **P2 drift ratchet:** `m2_substrate_inhabitance_test::v3_kernel_algebra_profile_mirror_matches_v2_stage0_authority`
/// compares this map entry-for-entry to the stage0 table.
///
/// **Dissolution:** when `kernel_algebra_profile` lowers to evaluated `.dag` (same
/// std-body staging trigger as the termination-lattice scaffold above), delete
/// this mirror and read the evaluated map instead.
pub fn kernel_algebra_profile(type_name: &str) -> Option<AlgebraProfile> {
    match type_name {
        "Int" => Some(AlgebraProfile::OrderedRingProfile),
        "Float" => Some(AlgebraProfile::ApproximateFieldProfile),
        "Bool" => Some(AlgebraProfile::BooleanAlgebraProfile),
        "String" => Some(AlgebraProfile::FreeMonoidScalarProfile),
        "List" => Some(AlgebraProfile::FreeMonoidCollectionProfile),
        "Set" => Some(AlgebraProfile::BooleanAlgebraCollectionProfile),
        "Map" => Some(AlgebraProfile::PartialFunctionProfile),
        _ => None,
    }
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
///   the declaration's Arrow body; FieldProject dispatches via
///   the input port's resolved Conj + field label; Operator
///   dispatches via the operand type's algebra walk.
/// - Pattern 2 (variant-is-data): fails. Callable carries a
///   DeclarationId; FieldProject carries a field label plus an
///   optional post-infer child declaration carrier;
///   Operator carries an OperatorKind.
/// - Pattern 3 (algebraic form): fails.
/// - Pattern 4 (dimensional): fails.
///
/// Verdict: mixed lifecycle. `Callable` and `FieldProject` are
/// terminal;
/// `Operator` is 🟡 scaffold with an explicit M2+ dissolution
/// trigger (surface grammar adoption of direct algebra field
/// access or a parse-time desugaring pass).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TransformTarget {
    /// A user function or resolved declaration. Inference walks the
    /// referenced declaration's `Arrow` connective via `resolve_arrow`.
    Callable(DeclarationId),
    /// Structural projection on a Conj-typed parent value. The
    /// input port is the single authority for the parent type;
    /// inference walks that input through any Instantiation /
    /// ResolvedIdentifier edges, looks up `field_label` on the
    /// reached Conj, and resolves the output through the same
    /// substitution context. `field_child` is the post-infer phase
    /// carrier for the resolved projected child declaration, so
    /// downstream consumers can read typed child identity without
    /// repeating the label lookup. No synthesized accessor
    /// declaration.
    FieldProject {
        field_label: String,
        field_child: Option<DeclarationId>,
    },
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

#[derive(Debug, Clone)]
pub struct BranchNode {
    pub id: NodeId,
    pub input: PortId,
    pub paths: Vec<Path>,
    pub output: PortId,
    pub span: SourceSpan,
}

impl BranchNode {
    pub fn result_port(&self) -> PortId {
        self.output
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
}

#[derive(Debug, Default, Clone)]
pub(crate) struct SubstrateMarkers {
    /// `dsl/std/v3_l1.dag` `Value` marker. Targets values
    /// (literals) in target language realizations.
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
    /// Sidecar structural facts for mutually-recursive SCCs.
    clusters: Vec<Cluster>,
    /// Synthetic match carriers for anonymous `T?` cardinalities. Used when
    /// inference needs stable `Some` / `None` variant identities without
    /// promoting optionals into named top-level declarations.
    optional_match_disjs: HashMap<DeclarationId, DeclarationId>,
}

static BOOTSTRAPPED_DAG: LazyLock<Dag> = LazyLock::new(|| {
    let mut dag = bootstrap_generated::bootstrapped_fixture_dag();
    dag.populate_primitive_cache();
    dag
});

// Generated bootstrap snapshots are performance caches over the checked-in
// `.dag` authorities, not independent authorities. `regen_bootstrap` is the
// sole writer and the PB-1 equivalence tests ratchet generated == runtime.
static BOOTSTRAPPED_STD_FIXTURE_DAG: LazyLock<Dag> = LazyLock::new(|| {
    let mut dag = bootstrap_std_generated::bootstrapped_std_fixture_dag();
    dag.populate_primitive_cache();
    dag
});

static BOOTSTRAPPED_DAG_WITHOUT_PARSE_SURFACE_FIXTURE: LazyLock<Dag> = LazyLock::new(|| {
    let mut dag =
        bootstrap_generated_without_parse_surface::bootstrapped_fixture_without_parse_surface_dag();
    dag.populate_primitive_cache();
    dag
});

impl Dag {
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
            clusters: Vec::new(),
            optional_match_disjs: HashMap::new(),
        }
    }

    pub fn new() -> Self {
        (*BOOTSTRAPPED_DAG).clone()
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

    /// Typed accessor for the v3_l1 `Value` marker. Same bootstrap-
    /// failure semantics as `bind_marker`.
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
    /// `resolve_producer(d, port_id) -> Behavior?`. DB-5 locks this
    /// as recursive Bind-chain resolution: follow `produced_by` to
    /// the producing Behavior, and if that Behavior is a `Bind`,
    /// recurse on `bind.value` until a non-Bind producer (Value /
    /// Transform / Branch / Loop) is reached. Every current lens
    /// wrote this chain inline; centralizing it here is the DB-14
    /// substrate-primitive refactor.
    ///
    /// `None` covers the miss modes (missing port, port has no
    /// producer, produced_by references a missing node) — all
    /// structurally equivalent to "no non-Bind producer found" at
    /// this substrate boundary. Richer lens-local enums layer on top.
    pub fn resolve_producer_opt(&self, port_id: &PortId) -> Option<&Behavior> {
        // Bounded by the total number of nodes: every Bind hop
        // consumes one node in the walk, and the Dag is finite.
        let bound = self.nodes.len();
        let mut current_port = *port_id;
        for _ in 0..=bound {
            let port = self.port_opt(&current_port)?;
            let producer_id = port.produced_by?;
            let behavior = self.node_opt(&producer_id)?;
            match behavior {
                Behavior::Bind(bind) => {
                    current_port = bind.value;
                    continue;
                }
                _ => return Some(behavior),
            }
        }
        // Cycle in the Bind chain — malformed substrate. Surface as
        // miss rather than infinite loop.
        None
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
        if let Some(p) = self.ports.get_mut(&port) {
            p.state = PortState::Unresolved;
        }
        self.diagnostics.insert(port, diagnostic);
    }

    /// Attach a diagnostic to the Dag without a pre-existing port anchor.
    /// Allocates a detached phantom port as the diagnostic carrier so the
    /// existing fail-closed biconditional still holds. Used by
    /// bootstrap / lowering for failures that don't have a natural
    /// PortId (unresolved declarations, tokenize/parse errors on
    /// bootstrap fixtures, duplicate top-level declarations, etc.).
    /// `compile_to_dag` surfaces these through `Err(CompileError::Semantic)`.
    pub(crate) fn attach_diagnostic(&mut self, diagnostic: Diagnostic) {
        let port = self.alloc_port(None);
        self.mark_unresolved(port, diagnostic);
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
        // declaration from `dsl/std/v3_l1.dag` by its declared
        // name and stores the typed handle. The lookup happens
        // once at bootstrap end; downstream consumers
        // (`lower_record_to_structural`, `emit_rust`) read the
        // typed handle via `bind_marker()` / `branch_marker()` /
        // etc. without any runtime name strings.
        self.substrate_markers.value = self.declaration_by_name("Value").map(|d| d.id);
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
        self.emit_anchors.substrate_accessor_binding = self
            .declaration_by_name("SubstrateAccessorBinding")
            .map(|d| d.id);
        self.emit_anchors.dag_type = self.declaration_by_name("Dag").map(|d| d.id);
        self.emit_anchors.std_list_fold = self.declaration_by_name("fold").map(|d| d.id);
        self.emit_anchors.rust_functions = self.declaration_by_name("rust_functions").map(|d| d.id);

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
            meta_tag: Some(binding_meta),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(binding_fields(rust_language, go_clean_emission)),
            refinement: None,
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
}
