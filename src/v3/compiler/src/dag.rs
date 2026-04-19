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

use std::collections::HashMap;
use std::sync::LazyLock;

use crate::diagnostics::{Diagnostic, DiagnosticTable, SourceSpan};
use crate::types::TypeShape;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(u32);

impl NodeId {
    fn index(self) -> usize {
        self.0 as usize
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
/// `type_params`, `meta_tag`, `inhabits`, and `value_body` are separate edges
/// with distinct semantics:
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

/// Dissolution ledger — **AtomPayload**:
///
/// 🟢 **Terminal at M1(2.6).** Four variants covering the four
/// user-input kinds at the type-atom level:
///
///   - `Literal(LiteralBits)` — a literal bit pattern carried at
///     the type level (e.g., the `3` in `Cardinality::Exact(3)`).
///   - `UnresolvedIdentifier(String)` — a name reference that has
///     not yet been resolved against the declaration table.
///     Produced during lowering and eliminated at
///     `resolve_pending_identifiers` time.
///   - `ResolvedIdentifier(DeclarationId)` — a name reference that
///     has been resolved. Structurally distinct from the unresolved
///     form (no `Option` field hiding a phase coproduct).
///   - `TypeParam(String)` — a type parameter declaration slot.
///     Shared across references inside a parameterized declaration.
///
/// 4-pattern check:
/// - Pattern 1 (fact placement): fails. Each variant has distinct
///   downstream consumers (literal constant folding, identifier
///   resolution via `declaration_by_name`, SubstStack lookup).
/// - Pattern 2 (variant-is-data): fails. Different payload types
///   per variant.
/// - Pattern 3 (algebraic form): fails. Each variant is a terminal
///   fact from a distinct user-input boundary.
/// - Pattern 4 (dimensional): fails.
///
/// Pre/post-sweep phase is now **structural**. Before M1(2.6)
/// review round 7 the shape was
/// `Identifier { name: String, resolved: Option<DeclarationId> }`
/// which hid a phase coproduct inside the Option. The split into
/// `UnresolvedIdentifier` and `ResolvedIdentifier` makes that
/// phase distinction visible to the type system: pattern matches
/// for unresolved stubs and resolved references are on separate
/// variants, not on `Some`/`None`.
///
/// Verdict: terminal. Future extensions (Span-backed metadata
/// atoms for diagnostic-only uses, Char / Float literals) go
/// through §8.10's substrate-extension audit.
#[derive(Debug, Clone)]
pub enum AtomPayload {
    /// A literal bit pattern carried at the type level.
    /// Computation-side literals live in ValueNode.data, not here.
    Literal(LiteralBits),
    /// An unresolved identifier. Produced during lowering when a
    /// name reference can't be resolved against the current symbol
    /// table (forward references, pending cross-file imports, the
    /// bootstrap's dangling refs to types in un-loaded std/ modules).
    /// `resolve_pending_identifiers` either converts to
    /// `ResolvedIdentifier` or emits a fail-closed diagnostic.
    UnresolvedIdentifier(String),
    /// A resolved identifier. Produced by
    /// `resolve_pending_identifiers` or directly by lowering when
    /// a name is already in the symbol table. Carries the typed
    /// edge to the referent declaration.
    ResolvedIdentifier(DeclarationId),
    /// A type parameter declaration slot. Declared at the top of a
    /// parameterized declaration (via `Declaration.type_params`);
    /// referenced from inside the body by ResolvedIdentifier atoms
    /// that resolve to this slot's DeclarationId. A single TypeParam
    /// Atom is shared across all references to it — the substrate
    /// is a DAG of declarations, not a tree.
    TypeParam(String),
}

/// Terminal literal payload. 3 variants, each corresponds to a distinct
/// user-input boundary (integer literal, boolean literal, string literal
/// from source text). Pattern-check is trivial: fact placement, variant-
/// is-data, algebraic form, and dimensional all fail on disjoint payload
/// types. Any future `Float`/`Char` additions go through §8.10's audit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LiteralBits {
    Int(i64),
    Bool(bool),
    String(String),
}

/// Dissolution ledger: CardinalityBound is a 3-variant coproduct that
/// encodes the "how many?" dimension of field/element repetition.
/// Required/Optional/List distinctions that v2 carried in separate
/// attributes collapse into this one axis.
///
/// 4-pattern check:
/// - Pattern 1 (fact placement): fails. Callers dispatch on bound.
/// - Pattern 2 (variant-is-data): partial. `Exact(n)` differs from
///   `AtMostOne`/`Unbounded` structurally (carries a u32), but
///   `AtMostOne` is distinct from `Exact(1)` because it admits the
///   zero case.
/// - Pattern 3 (algebraic form): passes. The three variants cover the
///   `{n}` / `[0..1]` / `[0..∞)` range algebra.
/// - Pattern 4 (dimensional): fails. No orthogonal coordinate space.
///
/// Verdict: terminal at M1(2.5). Adding `AtLeast(n)` / `Range(lo, hi)`
/// would be a variant extension subject to §8.10's audit; neither is
/// motivated by current std/ or shell.dag consumers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CardinalityBound {
    /// Exact count. Required = Exact(1); fixed-size arrays = Exact(n); argv
    /// literals = Exact(3).
    Exact(u32),
    /// Zero or one. Distinct from Unbounded so that `T?` and `List<T>` are
    /// structurally unrepresentable as the same thing.
    AtMostOne,
    /// Zero or more. `List<T>` and friends.
    Unbounded,
}

#[derive(Debug, Clone)]
pub struct TemplateArgument {
    /// The template parameter being bound. References a TypeParam Atom declared
    /// as a child of the template.
    pub parameter: DeclarationId,
    /// The concrete declaration the parameter binds to. This is a type
    /// declaration for ordinary generics and may also be a callable
    /// declaration for higher-order-function instantiation.
    pub value: DeclarationId,
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
///      `ArrowBody::UserDefined(bind_id)` before the Dag is frozen
///      — including on error paths (R13 fix at `lower.rs:2293`). A
///      named `Arrow(Pending)` surviving into the final Dag is
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
///   at `lower.rs:931` (`type_to_connective`), `lower.rs:872`
///   (anonymous nested Arrow), and `infer.rs:1893`
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
///   distinction exists to make the substrate predicate "named
///   `Arrow(Pending)` = R13-class regression" structurally exact
///   rather than a `name`-based proxy — see the
///   `lens_structural_resolution` ledger entry for the proxy
///   dissolution this enables.
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
///   **`Unparsed` persists**; `pipeline_compile_order_stage_names` reads its
///   **body span** as pipeline ordering authority — **terminal for bootstrap
///   ordering**, not a host bridge (contrast 2a) and not parse-lag debt
///   (contrast case 1). **Dissolution for 2c:** a future substrate change that
///   records stage order structurally and supersedes span extraction (then this
///   path retires). The signature flows forward through the declaration table
///   so callers can type-check against it; the body source span is preserved so
///   M2+ parser extensions can reach in for case 1, or so pipeline authority can
///   parse ordering for `compile`.
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
/// `compile` (DB-16 case 2c)** is different — **persistent bootstrap-range
/// ordering authority** until structural pipeline order supersedes span
/// extraction; it does **not** share case 1’s “wait for M2 parser” story. User-range
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
    ///    A named `Arrow(Pending)` surviving into the final Dag is
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
    /// match `Arrow(Pending)` as the structural fact for "executable-fn
    /// body patching missed a path" without depending on `decl.name` as
    /// a proxy for producer provenance.
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

/// Three-state port type. Illegal combinations of "has a type" and "has a
/// diagnostic" are unrepresentable by type:
///
///   - `Uninferred`: port exists but inference has not run on it yet.
///     Transitional state during DAG construction and fixpoint iteration. The
///     post-infer sweep drives every port to Resolved or Unresolved before
///     compile_to_dag returns.
///   - `Resolved(TypeShape)`: inference (or lowering from a declaration) has
///     committed to a type.
///   - `Unresolved`: inference or lowering detected a failure and called
///     `Dag::mark_unresolved`. A diagnostic exists in the DiagnosticTable
///     keyed by this port's id.
///
/// Biconditional (checked by the invariant audit test):
///   state == Unresolved  iff  diagnostics.contains(port.id())
///
/// **Dissolution receipt: TERMINAL.** PortState is substrate, not an annotation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PortState {
    Uninferred,
    Resolved(TypeShape),
    Unresolved,
}

/// A Port carries a typed value forward in time.
///
/// `state` has a single authoritative location: the Port struct stored in
/// Dag.ports. There are no stale copies — behaviors hold PortId references, not
/// embedded Ports.
#[derive(Debug, Clone)]
pub struct Port {
    id: PortId,
    pub(super) state: PortState,
    pub produced_by: Option<NodeId>,
}

impl Port {
    pub fn id(&self) -> PortId {
        self.id
    }

    pub fn state(&self) -> &PortState {
        &self.state
    }

    pub fn state_value(&self) -> PortState {
        self.state.clone()
    }

    /// Backward-compat accessor: returns `Some(&TypeShape)` for Resolved ports,
    /// `None` for Uninferred or Unresolved. Prefer `state()` when you need to
    /// distinguish the three cases.
    pub fn value_type(&self) -> Option<&TypeShape> {
        match &self.state {
            PortState::Resolved(ty) => Some(ty),
            PortState::Uninferred | PortState::Unresolved => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ValueNode {
    pub id: NodeId,
    pub data: LiteralBits,
    pub output: PortId,
    pub span: SourceSpan,
    /// Lane 2 Stage 2b: idempotency projection for this node. **Native Rust only**
    /// — not part of the reflected `Behavior` surface in `substrate.dag`, so `.dag`
    /// lenses cannot read it until a workflow fact is reflected + realized.
    /// Populated by lowering or [`Dag::try_register_lane2_workflow_effect`];
    /// [`crate::workflow_idempotency::analyze_workflow`] reads it from the graph.
    pub(crate) lane2_workflow: Option<Box<WorkflowEffect>>,
}

impl ValueNode {
    pub fn result_port(&self) -> PortId {
        self.output
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
    /// type, comparison returns Bool. No declaration is allocated.
    Operator(crate::operators::OperatorKind),
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

/// Per-arm pattern on a `Path`. Encodes which variant of the
/// scrutinee's Disj this arm handles. Unifies `if`/`else` with
/// `match` — both lower to `Branch` with one `Path` per arm, and
/// the discriminator lives on the pattern instead of on positional
/// convention.
///
/// **Phase coproduct — M1(2.8).** Lowering emits
/// `UnresolvedVariant { name, span }`; inference walks the
/// scrutinee's Disj children, matches the arm's variant name
/// scoped against that Disj, and mutates the Path's pattern
/// in-place to `ResolvedVariant(DeclarationId)`. The resolved
/// form is the stable post-infer shape; the unresolved form is
/// the transient lowering-output shape. Same pattern the substrate
/// uses for `AtomPayload::Unresolved/ResolvedIdentifier`.
///
/// 4-pattern check:
/// - Pattern 1 (fact placement): fails. Variant identity is a
///   per-arm fact that downstream code (exhaustiveness checks,
///   emission) must read per-Path.
/// - Pattern 2 (variant-is-data): fails. Unresolved carries a
///   name + span; Resolved carries a DeclarationId.
/// - Pattern 3 (algebraic form): fails. The two variants are
///   two phases of the same fact, not an algebra.
/// - Pattern 4 (dimensional): fails.
///
/// Verdict: terminal at M1(2.8). Future pattern extensions
/// (wildcard, record destructure, nested patterns) go through
/// §8.10's substrate-extension audit.
#[derive(Debug, Clone)]
pub enum BranchPattern {
    /// Arm pattern as written in surface syntax. Populated by
    /// lowering; the name is the variant identifier the user wrote
    /// (or `"True"` / `"False"` for an `if`/`else`'s two branches).
    /// Must be resolved by the end of inference.
    UnresolvedVariant { name: String, span: SourceSpan },
    /// Arm pattern resolved to a variant declaration. The
    /// `DeclarationId` points at the anonymous variant child of
    /// the scrutinee's Disj.
    ResolvedVariant(DeclarationId),
}

#[derive(Debug, Clone)]
pub struct PayloadBinding {
    /// Authored arm-local name from the surface pattern
    /// (`Some(payload)` -> `"payload"`). Lowering consumes it to
    /// extend the arm-local scope. It remains on the substrate as
    /// carry-forward for readable downstream rendering.
    pub binding_name: String,
    /// Port carrying the variant payload value for this arm.
    /// Lowering allocates the port so the binding can exist in
    /// arm-local scope immediately; inference later validates the
    /// matched variant shape and populates the payload type.
    pub payload_port: PortId,
}

#[derive(Debug, Clone)]
pub struct Path {
    pub body: NodeId,
    pub output: PortId,
    /// Which variant of the scrutinee's Disj this path handles.
    /// Discriminator for both `if`/`else` (on Bool) and `match`
    /// (on any Disj). See `BranchPattern`.
    pub pattern: BranchPattern,
    /// Optional payload extraction for this arm. Present for
    /// `Variant(binding)` surface patterns; absent for bare-variant
    /// arms and `if`/`else`.
    pub binding: Option<PayloadBinding>,
}

impl Path {
    pub fn result_port(&self) -> PortId {
        self.output
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ParamRef {
    member: NodeId,
    slot: usize,
}

impl ParamRef {
    pub fn member_of(self) -> NodeId {
        self.member
    }

    pub fn slot_of(self) -> usize {
        self.slot
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TransformRef(NodeId);

impl TransformRef {
    pub fn node_id(self) -> NodeId {
        self.0
    }
}

/// 🟢 **TERMINAL.** Bool-typed branch predicate port — Track 9 parallel to
/// [`ParamRef`] / [`TransformRef`]. The only Rust constructor is
/// [`Dag::branch_arm_of`], which checks the port resolves to `Bool`. The
/// substrate field shape matches `src/v3/std/effects.dag`; direct `.dag`
/// construction gains the same authority in the Lane 3c cycle (ROADMAP Track 9 debt).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BoolPortRef {
    port: PortId,
}

impl BoolPortRef {
    pub fn port_id(self) -> PortId {
        self.port
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NonEmptyList<T> {
    pub first: T,
    pub rest: Vec<T>,
}

impl<T> NonEmptyList<T> {
    pub fn from_vec(values: Vec<T>) -> Option<Self> {
        let mut iter = values.into_iter();
        let first = iter.next()?;
        Some(Self {
            first,
            rest: iter.collect(),
        })
    }

    pub fn to_vec(&self) -> Vec<T>
    where
        T: Clone,
    {
        std::iter::once(self.first.clone())
            .chain(self.rest.iter().cloned())
            .collect()
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        std::iter::once(&self.first).chain(self.rest.iter())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NonSingletonList<T> {
    pub first: T,
    pub second: T,
    pub rest: Vec<T>,
}

impl<T> NonSingletonList<T> {
    pub fn from_vec(values: Vec<T>) -> Option<Self> {
        let mut iter = values.into_iter();
        let first = iter.next()?;
        let second = iter.next()?;
        Some(Self {
            first,
            second,
            rest: iter.collect(),
        })
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        std::iter::once(&self.first)
            .chain(std::iter::once(&self.second))
            .chain(self.rest.iter())
    }
}

// ── std.effects mirror (DB-18 / Lane 2 Stage 2b) ───────────────────
//
// Structural carriers aligned with `src/v3/std/effects.dag` — the
// compiler-side authority for `compose_effects`, `WorkflowEffect`, and
// `BranchArm` until the self-hosted pipeline consumes the `.dag` forms
// directly.
//
// Each coproduct / boundary carrier below carries its own 🟢/🟡 dissolution
// stamp (modeling-discipline principle 4); do not rely on this banner alone.
// 🔴 does not appear in this block — there is no intentionally-wrong deferred
// carrier here; unsupported control flow is modeled via explicit sums, not
// silent placeholders.

/// 🟢 **TERMINAL.** HTTP verb literals — 1:1 with `std.effects` `HttpMethod`;
/// naming authority is `effects.dag`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HttpMethodScalar {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Options,
}

/// 🟢 **TERMINAL.** Where a stable idempotency key comes from — mirrors
/// `KeySource` in `effects.dag`; no parallel spelling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeySource {
    PathParam { param: String },
    InputField { field: String },
    CompositeKey { fields: Vec<String> },
}

/// 🟢 **TERMINAL.** Why a create-shaped op is classified breaking — mirrors
/// `CreateCause` in `effects.dag`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreateCause {
    PostAlways,
    KeylessFallback { method: HttpMethodScalar },
}

/// 🟢 **TERMINAL.** Idempotent-side effect shapes — mirrors `IdempotentShape`
/// in `effects.dag`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdempotentShape {
    ReadEffect,
    UpsertEffect { key_source: KeySource },
    DeleteEffect { key_source: KeySource },
}

/// 🟢 **TERMINAL.** Breaking-side effect shapes — mirrors `BreakingShape` in
/// `effects.dag`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BreakingShape {
    CreateEffect { cause: CreateCause },
    AppendEffect,
}

/// 🟢 **TERMINAL.** Classified per-op shape — sum of idempotent vs breaking
/// carriers; mirrors `EffectShape` in `effects.dag`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectShape {
    IsIdempotent(IdempotentShape),
    IsBreaking(BreakingShape),
}

/// 🟢 **TERMINAL.** Named operation plus classified shape — mirrors the
/// `OperationEffect` record in `effects.dag`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationEffect {
    pub operation_name: String,
    pub shape: EffectShape,
}

/// 🟢 **TERMINAL.** First breaking witness in a composition chain — mirrors
/// `BreakingOperation` in `effects.dag`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BreakingOperation {
    pub operation_name: String,
    pub shape: BreakingShape,
}

/// 🟢 **TERMINAL.** Result of linear `compose_effects` — mirrors
/// `CompositionVerdict` in `effects.dag`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompositionVerdict {
    IdempotentComposition,
    BrokenBy { first_breaker: BreakingOperation },
}

/// 🟢 **TERMINAL.** Branch arm with a [`BoolPortRef`] witnessed as Bool by
/// [`Dag::branch_arm_of`] — the sole constructor for valid arms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BranchArm {
    condition: BoolPortRef,
    body: Box<WorkflowEffect>,
}

/// 🟡 **SCAFFOLD.** Four-variant workflow sum aligned with `effects.dag`;
/// Stage 2b analyzes `LinearEffect` only — non-linear variants surface
/// `IdempotencyUnsupported` until branch-wise algebra lands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkflowEffect {
    LinearEffect {
        ops: NonEmptyList<OperationEffect>,
    },
    BranchEffect {
        arms: NonSingletonList<BranchArm>,
    },
    LoopEffect {
        body: Box<WorkflowEffect>,
    },
    ParallelEffect {
        branches: NonSingletonList<Box<WorkflowEffect>>,
    },
}

impl BranchArm {
    pub fn branch_predicate(&self) -> BoolPortRef {
        self.condition
    }

    pub fn body(&self) -> &WorkflowEffect {
        &self.body
    }
}

/// 🟢 **TERMINAL.** Explicit unsupported payload — names variant + stage +
/// reason; not a silent `Option` alongside a verdict.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdempotencyUnsupportedDetail {
    pub variant_name: String,
    pub downstream_stage: String,
    pub reason: String,
}

/// 🟢 **TERMINAL.** Stage 2b lens report sum — success path vs explicit
/// unsupported; mirrors `WorkflowIdempotencyReport` in `effects.dag`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkflowIdempotencyReport {
    WorkflowCompositionVerdict(CompositionVerdict),
    IdempotencyUnsupported(IdempotencyUnsupportedDetail),
}

// ── end std.effects mirror (DB-18) ───────────────────────────────────
// Cluster / loop-bound carriers below are Track 9 mutual-recursion
// witnesses — not part of the Lane 2 Stage 2b effects algebra.

/// 🟢 **TERMINAL.** Single cluster member's descent parameter — typed
/// `ParamRef` witness (see `docs/design-mutual-recursion-lowering.md`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemberDescent {
    pub param: ParamRef,
}

/// 🟢 **TERMINAL.** One intra-cluster `Transform` call edge inside the SCC.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntraClusterCall {
    pub transform: TransformRef,
}

/// 🟢 **TERMINAL.** Typed index over authoritative member/call topology for
/// `LoopBound::Descent` — not a parallel copy of the Dag call graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cluster {
    pub members: NonSingletonList<MemberDescent>,
    pub intra_cluster_calls: NonEmptyList<IntraClusterCall>,
}

/// 🟢 TERMINAL. `LoopBound` records the irreducible witness that makes
/// a `Behavior::Loop` honest at the substrate layer: either an
/// explicit runtime count port or a proved mutual-recursion cluster.
/// Collapsing the variants would either fabricate a count fact or
/// erase a structural termination proof. See
/// `docs/design-mutual-recursion-lowering.md` for the full receipt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopBound {
    Cardinality { count: PortId },
    Descent { cluster: ClusterId },
}

impl LoopBound {
    pub fn count_port(&self) -> Option<PortId> {
        match self {
            Self::Cardinality { count } => Some(*count),
            Self::Descent { .. } => None,
        }
    }
}

/// 🟢 TERMINAL at Stage 2d scope. Rust mirror of the
/// `SymbolicCost` coproduct declared in `src/v3/std/algebra.dag`.
/// The .dag declaration is the authority — this mirror exists
/// because `emit_rust_module` filters declarations from
/// `src/v3/std/` as bootstrap-resident, so the generated
/// `lens_cost_symbolic_generated.rs` references `SymbolicCost`
/// by name without re-declaring it. Keeping the Rust shape
/// adjacent to the other substrate carriers (`Behavior`,
/// `LoopBound`, `Cluster`) matches the existing pattern used for
/// every other .dag type the generated lenses consume.
///
/// The `SizeVariable` name field the DB-7 design doc sketches is
/// deliberately not mirrored at this stage: the MVP render path
/// pins structural equality on `source_port` alone (two
/// `LinearCost` terms collapse to `PolynomialCost(var, 2)` when
/// their ports match), and pulling a user-facing name through
/// would require an InternTable lookup the lens doesn't yet run.
/// Name-rendering lands when a concrete display consumer pins
/// the missing piece — tracked in DB-7 Open Question §1's
/// size-variable normalization.
///
/// Fields mirror the .dag variants exactly (anonymous tuple
/// payloads render as `{ _0: ... }`, record payloads render as
/// `{ field: ... }`); kept in sync by hand for now and tracked as
/// Lane 1e scheduled-deletion work when substrate emission grows
/// a cross-file type-mirror pass.
///
/// See `docs/design-symbolic-cost-algebra.md` (DB-7) for the
/// dissolution receipt and variant rationale.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolicCost {
    ConstantCost { _0: i64 },
    LinearCost { _0: SizeVariable },
    PolynomialCost { var: SizeVariable, degree: i64 },
    ProductCost { _0: Vec<SymbolicCost> },
    SumCost { _0: Vec<SymbolicCost> },
    LogCost { _0: SizeVariable },
    UnknownCost { _0: String },
}

/// 🟢 TERMINAL at Stage 2d scope. Rust mirror of `SizeVariable`
/// from `src/v3/std/algebra.dag`. Structural equality on
/// `source_port` is how two `LinearCost` terms collapse into
/// `PolynomialCost(var, 2)` through
/// `std.algebra::combine_binary_product` — the nested-fold
/// fingerprint DB-7's `all_pairs` acceptance fixture exercises.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SizeVariable {
    pub source_port: PortId,
}

// Composition functions mirroring `src/v3/std/algebra.dag`.
// `emit_rust_module` filters declarations under `src/v3/std/` as
// bootstrap-resident, so the functions there never emit their own
// Rust projection; `lens_cost_symbolic_generated.rs` references
// `sequential`, `iterate`, `max_path` by name and resolves them
// through `use crate::dag::*`. Kept in sync with the .dag source
// by hand for now, same cadence as the `SymbolicCost` variant
// mirror above.
//
// Algorithmic equivalence receipts:
//   - `sequential` / `iterate` follow DB-7 §"Composition
//     operations": sequential is a normalized sum, iterate is a
//     normalized product.
//   - `normalize` implements the Sum/Product reductions DB-7
//     §"Dominance / normalization" specifies: drop `ConstantCost(0)`
//     out of sums, collapse singleton wrappers, fold two identical
//     `LinearCost` terms into `PolynomialCost(var, 2)`.
//   - `dominates` implements the dominance partial order:
//     Unknown dominates everything (safest over-approximation),
//     Constant dominated by every other variant, Polynomial degrees
//     strictly-order, Linear≡Polynomial(v, 1), Sum/Product use
//     dominant-child summary.

pub fn sequential(a: SymbolicCost, b: SymbolicCost) -> SymbolicCost {
    normalize(SymbolicCost::SumCost { _0: vec![a, b] })
}

pub fn iterate(bound: SymbolicCost, body: SymbolicCost) -> SymbolicCost {
    normalize(SymbolicCost::ProductCost {
        _0: vec![bound, body],
    })
}

pub fn max_path(paths: &[SymbolicCost]) -> SymbolicCost {
    // Three-way step: candidate-wins / acc-wins / keep-both. Fixes
    // PR #537 review (Facts Flow Forward violation): pairing paths
    // via a two-way `if dominates(c, acc) then c else acc` silently
    // dropped whichever path was ordered later when the two were
    // incomparable (e.g. `Linear(n)` vs `Linear(m)` with distinct
    // size variables). When neither dominates, preserve both via
    // `sequential`, which normalizes through `drop_dominated`;
    // `Big-O(f + g) = Big-O(max(f, g))` makes the sum asymptotically
    // identical to the max.
    paths
        .iter()
        .fold(SymbolicCost::ConstantCost { _0: 0 }, |acc, candidate| {
            if dominates(candidate, &acc) {
                candidate.clone()
            } else if dominates(&acc, candidate) {
                acc
            } else {
                sequential(acc, candidate.clone())
            }
        })
}

pub fn normalize(cost: SymbolicCost) -> SymbolicCost {
    match cost {
        SymbolicCost::SumCost { _0: terms } => reduce_sum(drop_zero_terms(terms)),
        SymbolicCost::ProductCost { _0: terms } => reduce_product(drop_zero_terms(terms)),
        other => other,
    }
}

fn drop_zero_terms(terms: Vec<SymbolicCost>) -> Vec<SymbolicCost> {
    terms
        .into_iter()
        .filter(|t| !matches!(t, SymbolicCost::ConstantCost { _0: 0 }))
        .collect()
}

fn reduce_sum(mut terms: Vec<SymbolicCost>) -> SymbolicCost {
    terms = drop_dominated_in_sum(terms);
    match terms.len() {
        0 => SymbolicCost::ConstantCost { _0: 0 },
        1 => terms.into_iter().next().unwrap(),
        _ => SymbolicCost::SumCost { _0: terms },
    }
}

fn reduce_product(terms: Vec<SymbolicCost>) -> SymbolicCost {
    match terms.len() {
        0 => SymbolicCost::ConstantCost { _0: 0 },
        1 => terms.into_iter().next().unwrap(),
        2 => {
            let mut iter = terms.into_iter();
            let a = iter.next().unwrap();
            let b = iter.next().unwrap();
            combine_binary_product(a, b)
        }
        _ => SymbolicCost::ProductCost { _0: terms },
    }
}

fn combine_binary_product(a: SymbolicCost, b: SymbolicCost) -> SymbolicCost {
    if let (SymbolicCost::LinearCost { _0: va }, SymbolicCost::LinearCost { _0: vb }) = (&a, &b) {
        if va == vb {
            return SymbolicCost::PolynomialCost {
                var: va.clone(),
                degree: 2,
            };
        }
    }
    SymbolicCost::ProductCost { _0: vec![a, b] }
}

fn drop_dominated_in_sum(terms: Vec<SymbolicCost>) -> Vec<SymbolicCost> {
    let mut keep: Vec<SymbolicCost> = Vec::with_capacity(terms.len());
    for term in terms {
        let term_dominated = keep.iter().any(|k| dominates(k, &term));
        if term_dominated {
            continue;
        }
        keep.retain(|k| !dominates(&term, k));
        keep.push(term);
    }
    keep
}

pub fn dominates(a: &SymbolicCost, b: &SymbolicCost) -> bool {
    match a {
        SymbolicCost::UnknownCost { .. } => true,
        SymbolicCost::ConstantCost { .. } => matches!(b, SymbolicCost::ConstantCost { .. }),
        SymbolicCost::LinearCost { _0: va } => match b {
            SymbolicCost::ConstantCost { .. } | SymbolicCost::LogCost { .. } => true,
            SymbolicCost::LinearCost { _0: vb } => va == vb,
            SymbolicCost::PolynomialCost { var, degree } => va == var && *degree <= 1,
            _ => false,
        },
        SymbolicCost::PolynomialCost {
            var: va,
            degree: ka,
        } => match b {
            SymbolicCost::ConstantCost { .. } | SymbolicCost::LogCost { .. } => true,
            SymbolicCost::LinearCost { _0: vb } => va == vb && *ka >= 1,
            SymbolicCost::PolynomialCost {
                var: vb,
                degree: kb,
            } => va == vb && *ka >= *kb,
            _ => false,
        },
        SymbolicCost::LogCost { _0: va } => match b {
            SymbolicCost::ConstantCost { .. } => true,
            SymbolicCost::LogCost { _0: vb } => va == vb,
            _ => false,
        },
        // Composite dominance via "dominant child summary" (DB-7
        // §Dominance). Both `Sum([A, B])` and `Product([A, B])` bound
        // each child from below — sum and product of non-negative
        // terms are ≥ any single term — so the composite dominates
        // `b` iff *any* child does. Short-circuit evaluation keeps
        // this O(n) worst-case over the (post-normalize) ≤ few-term
        // lists. Fixes PR #537 review (codex): prior code hardcoded
        // `LinearCost(_) => True` / `LogCost(_) => True` without
        // inspecting terms, so e.g. `Product([Log(n)])` incorrectly
        // dominated `Linear(n)`.
        SymbolicCost::ProductCost { _0: terms } | SymbolicCost::SumCost { _0: terms } => {
            terms.iter().any(|child| dominates(child, b))
        }
    }
}

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
    /// Lane 2 Stage 2b: idempotency projection for this bind. Same contract as
    /// [`ValueNode::lane2_workflow`] (native Rust field; see that comment for the
    /// substrate-reflection deferral).
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

    pub fn span(&self) -> &SourceSpan {
        match self {
            Behavior::Value(v) => &v.span,
            Behavior::Transform(t) => &t.span,
            Behavior::Branch(b) => &b.span,
            Behavior::Loop(l) => &l.span,
            Behavior::Bind(b) => &b.span,
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
    /// `rust_clean_emission` CleanEmissionContract declaration
    /// loaded from `src/v3/spec/rust.dag`. Lane 1 Stage 1c / E-5:
    /// the emitter dispatches on this contract's rule fields to
    /// shape emitted code so it passes `rustc -D warnings` by
    /// construction. Go / Python cache analogues land when their
    /// respective pilots do.
    pub rust_clean_emission: Option<DeclarationId>,
    /// `rust_execution_model` declaration loaded from
    /// `src/v3/spec/rust.dag`. Used by emitters to gate the
    /// ownership stage on the target memory model.
    pub rust_execution_model: Option<DeclarationId>,
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
    /// `go_clean_emission` CleanEmissionContract declaration loaded
    /// from `src/v3/spec/go.dag`. Lane 1 Stage 1c PR 2 / E-5: the
    /// Go emitter dispatches on this contract's rule fields so
    /// emitted Go compiles under `gofmt -l` + the Go compiler's
    /// own unused-local check by construction.
    pub go_clean_emission: Option<DeclarationId>,
    /// `python_clean_emission` CleanEmissionContract declaration
    /// loaded from `src/v3/spec/python.dag`. Lane 1 Stage 1c PR 3 /
    /// E-5: the Python emitter dispatches on this contract's
    /// `pattern_bindings` field. Python's `NotApplicablePatternBinding`
    /// selects the substitute-at-render-time path — the binding
    /// identifier is never emitted at the pattern site, so
    /// py_compile never flags an unused binding.
    pub python_clean_emission: Option<DeclarationId>,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct StdlibTypeCache {
    /// `std.list.List` template declaration. Resolved once at
    /// bootstrap end so downstream consumers compare typed
    /// declaration ids instead of reconstructing stdlib identity
    /// through `declaration_by_name("List")`.
    pub list: Option<DeclarationId>,
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
    /// Sidecar structural facts for mutually-recursive SCCs.
    clusters: Vec<Cluster>,
    /// Synthetic match carriers for anonymous `T?` cardinalities. Used when
    /// inference needs stable `Some` / `None` variant identities without
    /// promoting optionals into named top-level declarations.
    optional_match_disjs: HashMap<DeclarationId, DeclarationId>,
}

static BOOTSTRAPPED_DAG: LazyLock<Dag> = LazyLock::new(|| {
    let mut dag = Dag::empty();
    crate::bootstrap::bootstrap(&mut dag);
    dag
});

impl Dag {
    fn empty() -> Self {
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
            pattern_binding_rule_variants: PatternBindingRuleVariants::default(),
            variant_payload_field_access_rule_variants:
                VariantPayloadFieldAccessRuleVariants::default(),
            verifier_output_policy_variants: VerifierOutputPolicyVariants::default(),
            clusters: Vec::new(),
            optional_match_disjs: HashMap::new(),
        }
    }

    pub fn new() -> Self {
        (*BOOTSTRAPPED_DAG).clone()
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
        self.target_syntax.rust_clean_emission
    }

    /// Typed accessor for the Rust target execution model
    /// declaration loaded from `src/v3/spec/rust.dag`.
    pub fn rust_execution_model_spec(&self) -> Option<DeclarationId> {
        self.target_syntax.rust_execution_model
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

    /// Typed accessor for the Go `CleanEmissionContract`
    /// declaration loaded from `src/v3/spec/go.dag` (E-5 / Lane 1
    /// Stage 1c PR 2). Mirrors `rust_clean_emission_spec`; emitter
    /// parses the structural fields and dispatches on the rule
    /// variants.
    pub fn go_clean_emission_spec(&self) -> Option<DeclarationId> {
        self.target_syntax.go_clean_emission
    }

    /// Typed accessor for the Python `CleanEmissionContract`
    /// declaration loaded from `src/v3/spec/python.dag` (E-5 / Lane
    /// 1 Stage 1c PR 3). Mirrors `rust_clean_emission_spec` and
    /// `go_clean_emission_spec`; emitter parses the structural
    /// fields and dispatches on the rule variants.
    pub fn python_clean_emission_spec(&self) -> Option<DeclarationId> {
        self.target_syntax.python_clean_emission
    }

    /// Typed accessor for the cached `std.list.List` template.
    pub fn list_template(&self) -> Option<DeclarationId> {
        self.stdlib_types.list
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

    pub fn cluster(&self, id: ClusterId) -> &Cluster {
        &self.clusters[id.index()]
    }

    /// **🟡 Scaffold hook (API is intentional, substrate is not).** Attaches a
    /// [`WorkflowEffect`] on **native** [`Behavior`] nodes at `root` (`Value` or
    /// `Bind` only). This does **not** populate a reflected substrate field —
    /// `.dag` lens walkers cannot see `lane2_workflow`. Not a type-system proof
    /// that `root` is “the” workflow root; tests and lowering use it under the
    /// ROADMAP “Reflection boundary” contract until the fact is reflected.
    /// Returns `false` if `root` is missing or not `Value`/`Bind`. Downstream
    /// lowering should populate the same fields so
    /// [`crate::workflow_idempotency::analyze_workflow`] reads one graph-local
    /// store (not a parallel side table).
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

    pub fn lane2_workflow_effect_at(&self, root: NodeId) -> Option<&WorkflowEffect> {
        match self.node_opt(&root)? {
            Behavior::Value(v) => v.lane2_workflow.as_deref(),
            Behavior::Bind(b) => b.lane2_workflow.as_deref(),
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

    fn declaration_name_preference_rank(file: &str) -> usize {
        if file.starts_with("src/v3/") {
            2
        } else if file.starts_with("dsl/") {
            0
        } else {
            1
        }
    }

    /// Find a top-level declaration by name. Prefer v3 declarations
    /// over legacy `dsl/` duplicates; otherwise keep the earliest
    /// declaration at the same precedence level.
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
        self.target_syntax.rust_clean_emission = self
            .declaration_by_name("rust_clean_emission")
            .map(|d| d.id);
        self.target_syntax.rust_execution_model = self
            .declaration_by_name("rust_execution_model")
            .map(|d| d.id);
        self.target_syntax.dag_model = self.declaration_by_name("dag_model").map(|d| d.id);
        self.target_syntax.go_language = self.declaration_by_name("go_language").map(|d| d.id);
        self.target_syntax.go_execution_model =
            self.declaration_by_name("go_execution_model").map(|d| d.id);
        self.target_syntax.go_clean_emission =
            self.declaration_by_name("go_clean_emission").map(|d| d.id);
        self.target_syntax.python_clean_emission = self
            .declaration_by_name("python_clean_emission")
            .map(|d| d.id);
        self.stdlib_types.list = self.declaration_by_name("List").map(|d| d.id);

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
    }

    pub fn param_of(&self, member: NodeId, slot: usize) -> Option<ParamRef> {
        let bind = self.node(member).as_bind()?;
        bind.params.get(slot)?;
        Some(ParamRef { member, slot })
    }

    /// Construct a [`BranchArm`] only when `port` is resolved to the `Bool`
    /// primitive, packaging the port as a [`BoolPortRef`] (Track 9
    /// parity with [`Dag::param_of`] / [`Dag::as_transform_ref`]).
    pub fn branch_arm_of(&self, port: PortId, body: WorkflowEffect) -> Option<BranchArm> {
        let bool_ty = self.bool_shape()?;
        let p = self.port_opt(&port)?;
        let ty = p.value_type()?;
        if *ty != bool_ty {
            return None;
        }
        Some(BranchArm {
            condition: BoolPortRef { port },
            body: Box::new(body),
        })
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
