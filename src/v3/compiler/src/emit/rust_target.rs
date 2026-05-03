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
//   - rust.dag carries typed `DeclarationRef` field references via
//     identifier and dotted-path values resolved at lower time.
//   - emit_rust.rs builds typed indexes keyed by
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
//   - Narrow staged-std list helpers used by Prereq 4 fixtures:
//     `empty`, `singleton`, `cons`, `fold`, `map`, `filter`
//
// Out of scope (follow-up work, tracked in DOWNSTREAM_REQUIREMENTS):
//   - User-defined functions (Bind with non-empty params)
//   - Loops
//   - General TransformTarget::Callable dispatch
//   - Record / enum construction
//
// Template placeholders the substitution engine recognizes (see
// src/v3/spec/rust.dag for the authoritative list):
//   {name}  Bind name           {cond}  branch condition
//   {type}  Rust type name      {then}  then-arm body
//   {value} bind value expr     {else}  else-arm body
//   {body}  list of lets        {quote} literal double-quote
//   {final} final bind's name

use std::collections::{HashMap, HashSet};

use super::{
    algebra_field_for_operator_shared, dag_needs_div_error_prelude,
    div_prelude_reserved_name_collision, parse_pattern_strategy, primitive_type_id_for_port_shared,
    walk_to_disj, EmitMode, PatternStrategyBinding, SharedEmitLookupError, SourceFilteringBinding,
    VariantPayloadBinding, VariantPayloadFieldAccessRuleBinding,
};
use crate::dag::{
    ArrowBody, AtomPayload, Behavior, BranchNode, BranchPattern, Dag, DeclarationId, Field,
    FieldValue, LiteralBits, Path, PortId, TemplateArgument, TransformNode, TransformTarget,
    TypeConnective, ValueBody, ValueNode,
};
use crate::operators::OperatorKind;
use crate::variant_payload::{
    variant_payload_shape, VariantPayloadShape, VariantPayloadShapeLookup,
};

/// Errors the Rust emitter surfaces when the DAG reaches a shape it
/// cannot render under the PR-B scope. Each variant names a specific
/// structural cause — no catch-all `Unknown` — so consumers can
/// classify the failure against `src/v3/spec/rust.dag`'s
/// coverage.
///
/// **Dissolution receipt — 🟢 TERMINAL.** Eleven variants, each
/// classifying a structurally distinct failure mode at a different
/// boundary in the emitter pipeline. The variants partition into
/// four categories:
///
///   1. **Realization-table gaps** (`MissingTypeRealization`,
///      `MissingOperatorRealization`, `MissingBehaviorRealization`):
///      the declaration the DAG references has no matching
///      realization in `src/v3/spec/rust.dag`. Each payload is a
///      typed `DeclarationId` (no string keys) so the caller can
///      pinpoint which declaration is uncovered.
///
///   2. **Spec consistency failures** (`MissingRealizationMeta`,
///      `MalformedRealization`, `DuplicateRealization`): the
///      per-target spec file (rust.dag) is internally inconsistent
///      — missing meta-types, malformed realization records, or
///      duplicate keys. These are spec-side bugs that
///      `RealizationIndexes::build` fail-closes on at index
///      construction time. The pre-unwind shape silently dropped
///      malformed entries and silently overwrote duplicate keys;
///      the explicit variants are the fail-closed counterpart.
///
///   3. **Substrate-side bugs** (`MissingSubstrateMarker`,
///      `UntypedPort`, `UnresolvedBranchPattern`): the substrate
///      handed the emitter a state inference should have driven
///      to a terminal form. Reaching any of these is a bug in
///      bootstrap, infer, or lowering — not a target-language
///      coverage issue.
///
///   4. **Out-of-scope DAG shapes** (`UnsupportedBehavior`,
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
    /// A required substrate marker is absent from `src/v3/spec/v3_l1.dag`
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
    /// A Branch arm resolved to a variant declaration that no longer
    /// has a Disj parent. Emit must not fabricate a plausible target
    /// identifier when the substrate cannot identify the parent enum.
    VariantParentNotFound { variant_id: DeclarationId },
    /// A Branch's scrutinee resolved to a Disj that isn't the v3
    /// `Classical` (Bool) sum — PR-B only emits boolean branches.
    /// Carries the scrutinee's resolved variant ids so callers can
    /// inspect what the substrate handed them.
    NonBooleanBranch { variant_ids: Vec<DeclarationId> },
    /// A required realization meta-type is missing from the cached
    /// substrate. Bootstrap should populate all realization categories
    /// from `src/v3/spec/rust.dag`; reaching this arm means the
    /// spec file failed to load or the meta-type wasn't declared.
    MissingRealizationMeta(RealizationCategory),
    /// The target-language syntax bundle is absent from rust.dag.
    MissingTargetSyntax(&'static str),
    /// The target-language syntax bundle or one of its referenced
    /// syntax declarations has the wrong structural shape.
    MalformedTargetSyntax {
        declaration: DeclarationId,
        detail: &'static str,
    },
    /// A data item tagged with a realization meta-type carried a
    /// wrong-shaped value body or a missing required field. The
    /// `lower_record_to_structural` inhabitance check should have
    /// caught this at lower time; reaching it at index-build time
    /// is a fail-open leak that the index builder fail-closes on.
    MalformedRealization {
        declaration: DeclarationId,
        detail: &'static str,
    },
    /// Two realizations in the loaded spec set share the same key
    /// (e.g. two `data rust_*: TypeRealization` items both
    /// targeting `Int`). Single Authority requires the spec to be
    /// unambiguous; collisions are spec bugs.
    DuplicateRealization {
        declaration: DeclarationId,
        detail: &'static str,
    },
    /// A user-defined callable's Arrow shape doesn't match the
    /// structural invariants the ownership analysis depends on —
    /// e.g., the body isn't a `Bind`, or `Bind.params` has fewer
    /// ports than the Arrow declares inputs. The IR should be
    /// well-formed before emit runs; reaching this arm is an
    /// upstream bug, not a spec issue.
    MalformedUserDefinedCallable {
        declaration: DeclarationId,
        detail: &'static str,
    },
}

/// Typed tag identifying which realization category a
/// `MissingRealizationMeta` error refers to. Three variants, one
/// per meta-type declared in the per-target spec file. Replaces
/// what would otherwise be a string label for the missing role.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RealizationCategory {
    /// `TypeRealization` — primitive type → target type name.
    Type,
    /// `TypeInstantiationRealization` — generic template
    /// declaration → target instantiated carrier syntax.
    TypeInstantiation,
    /// `OperatorRealization` — (operand type, algebra field) →
    /// target operator symbol.
    Operator,
    /// `BehaviorRealization` — substrate marker → target template.
    Behavior,
    /// `CallableRealization` — callable declaration → Rust render
    /// strategy.
    Callable,
    /// `PatternRealization` — structural sum declaration →
    /// carrier-specific pattern lowering facts.
    Pattern,
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
#[derive(Debug, Clone, PartialEq, Eq)]
enum RustFieldAccessBinding {
    DirectField(String),
    AccessorMethod(String),
}

#[derive(Debug, Clone)]
struct FieldBindingBinding {
    access: RustFieldAccessBinding,
    borrowed_read: bool,
}

#[derive(Debug, Clone)]
struct TypeRealizationBinding {
    carrier: String,
    is_copy: bool,
    fields: HashMap<String, FieldBindingBinding>,
}

#[derive(Debug, Clone)]
struct TypeInstantiationBinding {
    carrier: String,
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RustCallableStrategyBinding {
    ListEmpty,
    ListSingleton,
    ListCons,
    ListConcat,
    ListLength,
    ListIsEmpty,
    ListFold,
    ListMap,
    ListFilter,
    ListContains,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParameterDispositionBinding {
    Borrowed,
    Consumed,
}

impl ParameterDispositionBinding {
    fn merge(self, other: Self) -> Self {
        match (self, other) {
            (Self::Consumed, _) | (_, Self::Consumed) => Self::Consumed,
            _ => Self::Borrowed,
        }
    }
}

#[derive(Debug, Clone)]
struct PatternRealizationBinding {
    empty_variant: DeclarationId,
    cons_variant: DeclarationId,
    scrutinee: String,
    empty_pattern: String,
    cons_pattern: String,
    head_expr: String,
    tail_expr: String,
}

#[derive(Debug, Clone)]
struct StatementSyntaxBinding {
    let_binding: String,
}

#[derive(Debug, Clone)]
struct ExpressionSyntaxBinding {
    field_access: String,
    function_call: String,
    closure: String,
}

#[derive(Debug, Clone)]
struct ControlFlowSyntaxBinding {
    if_else: String,
}

#[derive(Debug, Clone)]
struct LiteralSyntaxBinding {
    true_keyword: String,
    false_keyword: String,
    string_delimiter: String,
}

#[derive(Debug, Clone)]
struct ModuleSyntaxBinding {
    path_separator: String,
}

#[derive(Debug, Clone)]
struct FunctionSyntaxBinding {
    definition: String,
    definition_exported: String,
    param_with_type: String,
    param_separator: String,
}

#[derive(Debug, Clone)]
struct TypeApplicationSyntaxBinding {
    optional: String,
}

#[derive(Debug, Clone)]
struct TypeDefinitionSyntaxBinding {
    struct_def: String,
    struct_field: String,
    enum_def: String,
    enum_unit_variant: String,
    enum_data_variant: String,
}

#[derive(Debug, Clone)]
struct RecordDeriveTemplateBundleBinding {
    struct_def_no_debug: String,
    enum_def_no_debug: String,
}

#[derive(Debug, Clone)]
struct PatternMatchSyntaxBinding {
    match_expr: String,
    match_arm: String,
    variant_pattern: String,
    variant_pattern_positional: String,
    variant_pattern_empty: String,
    field_binding: String,
    field_binding_separator: String,
    // Wildcard pattern (`_`). Declared by every target spec but
    // currently unused by the Rust emitter: multi-field struct-variant
    // patterns now alias every field for override routing, and
    // zero-binding variant patterns flow through
    // `variant_pattern_empty`. Kept on the binding so future emit
    // paths (other target languages, partial-match forms) can consume
    // it without another spec round-trip.
    #[allow(dead_code)]
    wildcard: String,
}

#[derive(Debug, Clone)]
struct CollectionOpsBinding {
    concat: String,
    length: String,
    is_empty: String,
    fold: String,
    map: String,
    filter: String,
    contains: String,
    empty_list: String,
    list_literal: String,
    cons: String,
}

#[derive(Debug, Clone)]
struct ValueConstructionSyntaxBinding {
    struct_literal: String,
    struct_field_init: String,
    struct_field_separator: String,
    variant_named_construction: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReadStrategyBinding {
    Borrow,
    PassByValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConstructStrategyBinding {
    CopyOrClone,
    PassByValue,
}

#[derive(Debug, Clone)]
struct RenderingModelBinding {
    read: ReadStrategyBinding,
    construct: ConstructStrategyBinding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SourceMutabilityBinding {
    Immutable,
    Mutable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SourcePurityBinding {
    Pure,
    Effectful,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SourceStructureBinding {
    ExplicitDag,
    Arbitrary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SourceIterationBinding {
    Bounded,
    Unbounded,
}

#[derive(Debug, Clone)]
struct ComputationModelBinding {
    mutability: SourceMutabilityBinding,
    purity: SourcePurityBinding,
    structure: SourceStructureBinding,
    iteration: SourceIterationBinding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MemoryModelBinding {
    ValueOnly,
    GarbageCollected,
    RefCounted,
    OwnershipBased,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScopeModelBinding {
    LexicalScoping,
    DynamicScoping,
}

#[derive(Debug, Clone)]
struct TargetExecutionModelBinding {
    memory: MemoryModelBinding,
    scope: ScopeModelBinding,
}

#[derive(Debug, Clone)]
struct RustLanguageSyntax {
    statements: StatementSyntaxBinding,
    expressions: ExpressionSyntaxBinding,
    control_flow: ControlFlowSyntaxBinding,
    literals: LiteralSyntaxBinding,
    modules: ModuleSyntaxBinding,
    functions: FunctionSyntaxBinding,
    type_applications: TypeApplicationSyntaxBinding,
    type_definitions: TypeDefinitionSyntaxBinding,
    /// From `rust_language.record_derive_templates` (`LanguageSpec` in
    /// `emit_model.dag`); Rust data is `rust_record_derive_templates` in
    /// `rust.dag` — `RecordDeriveTemplateBundle` (target-neutral; Rust consumes).
    record_derive_no_debug: RecordDeriveTemplateBundleBinding,
    patterns: PatternMatchSyntaxBinding,
    collection_ops: CollectionOpsBinding,
    values: ValueConstructionSyntaxBinding,
}

/// Typed read of `data rust_clean_emission: CleanEmissionContract`
/// from `src/v3/spec/rust.dag` — the portion this pilot consumes
/// (E-5 / Lane 1 Stage 1c PR 1). Other contract rules land here as
/// their consumers wire in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CleanEmissionContractBinding {
    pattern_bindings: PatternBindingRuleBinding,
    variant_payload_field_access: VariantPayloadFieldAccessRuleBinding,
}

/// Rust-valid slice of `std.clean_emission.PatternBindingRule`.
/// Parsed in `CleanEmissionContractBinding::build`, which rejects
/// target-invalid constructors instead of letting the renderer
/// normalize them later.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PatternBindingRuleBinding {
    EmitBindingAlways,
    EmitUnderscoreWhenUnused,
}

struct RealizationIndexes {
    /// `target_decl_id → carrier + field bindings`. Built from
    /// `data rust_*: TypeRealization` items in rust.dag.
    types: HashMap<DeclarationId, TypeRealizationBinding>,
    /// `generic_template_decl → instantiated carrier syntax`.
    /// Built from `data rust_*: TypeInstantiationRealization`
    /// items in rust.dag. Used when rendering generic carriers
    /// such as `List<T> -> Vec<T>` without template-name string
    /// dispatch in Rust.
    instantiations: HashMap<DeclarationId, TypeInstantiationBinding>,
    /// `(operand_type_decl, algebra_field_decl) → carrier`. Built
    /// from `data rust_*: OperatorRealization` items in rust.dag.
    /// Used when emitting a `Transform { target: Operator(_), .. }`.
    operators: HashMap<(DeclarationId, DeclarationId), String>,
    /// `behavior_marker_decl → carrier`. Built from `data rust_*:
    /// BehaviorRealization` items in rust.dag. Used when emitting
    /// the substrate behaviors (let / if-else / main wrapper). The
    /// key declaration ids come from `src/v3/spec/v3_l1.dag` markers
    /// cached in `Dag::substrate_markers` — every dispatch site
    /// reads those typed handles instead of looking up by name.
    behaviors: HashMap<DeclarationId, String>,
    /// `callable_decl → render strategy`. Built from `data rust_*:
    /// CallableRealization` items in rust.dag. Used when emitting
    /// callable transforms without name-keyed builtin dispatch.
    callables: HashMap<DeclarationId, RustCallableStrategyBinding>,
    /// `callable_decl → per-runtime-input ownership disposition`.
    /// External callables read this directly from rust.dag; user-
    /// defined callables derive it from their body DAG via a
    /// monotone body walk.
    callable_dispositions: HashMap<DeclarationId, Vec<ParameterDispositionBinding>>,
    /// `structural_sum_decl → carrier pattern lowering facts`.
    /// Built from `data rust_*: PatternRealization` items in
    /// rust.dag. Used when a structural match lowers against a
    /// realized container carrier rather than a native Rust enum.
    patterns: HashMap<DeclarationId, PatternRealizationBinding>,
    /// DB-14: `accessor_decl → realization_decl` for the active
    /// Rust target. Built from `data *: SubstrateAccessorBinding`
    /// records filtered by `language == rust_language`. Used by
    /// `render_substrate_accessor` on every Callable Transform: if
    /// the callable's target is in this map, render the
    /// realization's `carrier` template; otherwise fall through to
    /// normal callable dispatch. Replaces the earlier design that
    /// upgraded accessor Arrow bodies to `ExternalRealization` at
    /// bootstrap, which silently dropped target selection (review
    /// round 1b.3 root cause).
    substrate_accessors: HashMap<DeclarationId, DeclarationId>,
    /// DB-14 coverage set: every accessor declaration referenced by
    /// **any** `SubstrateAccessorBinding` record, across all target
    /// languages. `render_substrate_accessor` uses this to
    /// distinguish "callable target is not a substrate accessor at
    /// all" (fall through to normal dispatch) from "declared
    /// substrate accessor, but no binding for the active target"
    /// (fail-closed `EmitError::MissingSubstrateAccessorRealization`).
    /// Post review round 1b.3: a missing active-target realization
    /// must be a hard emit error, not a silent fall-through that
    /// would emit `func(args)` for a function Rust doesn't have.
    substrate_accessor_universe: HashSet<DeclarationId>,
    /// The Rust target-language syntax bundle loaded from
    /// `data rust_language: LanguageSpec`.
    syntax: RustLanguageSyntax,
    /// The Rust target-language ownership rendering model loaded
    /// from `data rust_rendering: RenderingModel`.
    rendering: RenderingModelBinding,
    /// The source-side `.dag` computation model loaded from
    /// `data dag_model: ComputationModel`.
    computation: ComputationModelBinding,
    /// The target-side Rust execution model loaded from
    /// `data rust_execution_model: TargetExecutionModel`.
    execution: TargetExecutionModelBinding,
    /// Source exclusion policy loaded from
    /// `data rust_source_filtering: ShapeATargetSourceFiltering`.
    source_filtering: SourceFilteringBinding,
    /// The Rust clean-emission contract loaded from
    /// `data rust_clean_emission: CleanEmissionContract` (E-5 /
    /// Lane 1 Stage 1c). Rule variants dispatch inside the emitter
    /// to shape emitted code so it passes `rustc -D warnings` by
    /// construction.
    clean_emission: CleanEmissionContractBinding,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum InputSlot {
    Positional(usize),
    BranchInput,
    LoopSource,
    LoopInit,
    LoopBoundCount,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct InputUseKey {
    consumer: crate::dag::NodeId,
    slot: InputSlot,
}

#[derive(Debug, Clone, Default)]
struct InputUseFacts {
    /// Per-edge total ordering across all input slots in the DAG.
    /// Includes every consumer/slot — Borrowed and Consumed alike.
    edge_order: HashMap<InputUseKey, usize>,
    /// Per-port, the order of the LAST edge that touches the port —
    /// regardless of whether that edge is a Borrow or a Consume.
    /// "Safe to move at edge E" requires E's order equal this value:
    /// any later use (borrow or consume) makes the move unsafe.
    last_use_order_by_port: HashMap<PortId, usize>,
}

impl RealizationIndexes {
    /// Build the typed indexes from rust.dag's data
    /// declarations. **Fail-closed at every step.** Returns
    /// `Err(EmitError)` if:
    ///
    ///   - The realization meta-type cache is unpopulated (a
    ///     bootstrap failure earlier in the pipeline; rust.dag
    ///     didn't load).
    ///   - A declaration tagged with one of the realization
    ///     meta-types is missing a required field (target / op /
    ///     carrier) or carries a wrong-shaped field value. This
    ///     is a spec-file consistency error that the lower-time
    ///     inhabitance check would normally surface — reaching
    ///     this path here means the inhabitance check let
    ///     something through and we want a loud failure, not a
    ///     silent skip.
    ///   - Two realizations share the same key (e.g. two
    ///     `data rust_*: TypeRealization` items both targeting
    ///     `Int`). Single Authority requires the spec to be
    ///     unambiguous; collisions are spec bugs.
    ///
    /// The pre-unwind silent-skip + silent-overwrite shape was
    /// flagged in PR #445 review as fail-open behavior at the
    /// realization boundary. This function is the explicit
    /// fail-closed counterpart.
    fn build(dag: &Dag) -> Result<Self, EmitError> {
        // Read the cached realization meta-type handles. These
        // are populated at bootstrap end via
        // `populate_primitive_cache` from `src/v3/spec/rust.dag`'s
        // top-level `type TypeRealization { ... }` declarations.
        // Reading them via the typed accessor keeps emit_rust
        // free of any name-string lookup: the meta-type identity
        // is a `DeclarationId` from the moment the index builds.
        let type_meta = dag
            .type_realization_meta()
            .ok_or(EmitError::MissingRealizationMeta(RealizationCategory::Type))?;
        let type_instantiation_meta =
            dag.type_instantiation_realization_meta()
                .ok_or(EmitError::MissingRealizationMeta(
                    RealizationCategory::TypeInstantiation,
                ))?;
        let op_meta = dag
            .operator_realization_meta()
            .ok_or(EmitError::MissingRealizationMeta(
                RealizationCategory::Operator,
            ))?;
        let behavior_meta =
            dag.behavior_realization_meta()
                .ok_or(EmitError::MissingRealizationMeta(
                    RealizationCategory::Behavior,
                ))?;
        let callable_meta =
            dag.callable_realization_meta()
                .ok_or(EmitError::MissingRealizationMeta(
                    RealizationCategory::Callable,
                ))?;
        let pattern_meta =
            dag.pattern_realization_meta()
                .ok_or(EmitError::MissingRealizationMeta(
                    RealizationCategory::Pattern,
                ))?;

        let mut types: HashMap<DeclarationId, TypeRealizationBinding> = HashMap::new();
        let mut instantiations: HashMap<DeclarationId, TypeInstantiationBinding> = HashMap::new();
        let mut operators: HashMap<(DeclarationId, DeclarationId), String> = HashMap::new();
        let mut behaviors: HashMap<DeclarationId, String> = HashMap::new();
        let mut callables: HashMap<DeclarationId, RustCallableStrategyBinding> = HashMap::new();
        let mut external_callable_dispositions: HashMap<
            DeclarationId,
            Vec<ParameterDispositionBinding>,
        > = HashMap::new();
        let mut patterns: HashMap<DeclarationId, PatternRealizationBinding> = HashMap::new();

        // Cache the Rust language-spec declaration id once; shared
        // realizations reference it via `language: DeclarationRef`
        // and this emitter picks up only entries that match.
        // Replaces the previous TargetLanguage enum roster with a
        // declaration-identity compare (INVARIANTS.md E-6: target
        // ownership is carried by a typed edge, not a compiler-side
        // variant list).
        let rust_language_id = dag
            .rust_language_spec()
            .ok_or(EmitError::MissingTargetSyntax("rust_language"))?;

        for decl in dag.declarations() {
            let Some(meta_tag) = decl.meta_tag else {
                continue;
            };
            // Determine which realization category this declaration
            // belongs to (if any). Comparing typed handles, no name
            // matching.
            let category = if meta_tag == type_meta {
                RealizationCategory::Type
            } else if meta_tag == type_instantiation_meta {
                RealizationCategory::TypeInstantiation
            } else if meta_tag == op_meta {
                RealizationCategory::Operator
            } else if meta_tag == behavior_meta {
                RealizationCategory::Behavior
            } else if meta_tag == callable_meta {
                RealizationCategory::Callable
            } else if meta_tag == pattern_meta {
                RealizationCategory::Pattern
            } else {
                continue;
            };
            let Some(ValueBody::Structural { fields }) = &decl.value_body else {
                // A data item tagged with a realization meta-type
                // must have a Structural value_body. If it's
                // Unparsed, the inhabitance check let a malformed
                // spec entry through — fail-closed so the spec
                // inconsistency surfaces loudly.
                return Err(EmitError::MalformedRealization {
                    declaration: decl.id,
                    detail:
                        "realization data item has no Structural value_body — bootstrap inhabitance check missed a malformed spec entry",
                });
            };

            // Skip realizations declared for other shared targets
            // (e.g. Go) by comparing the typed `language` field to
            // this emitter's cached language-spec declaration id.
            // Replaces the previous TargetLanguage enum roster; adding
            // a new shared target is now a pure spec-file change.
            let language_ref = require_field_decl_ref(fields, "language", decl.id)?;
            if language_ref != rust_language_id {
                continue;
            }

            // Required for every category. Missing → fail-closed
            // (the inhabitance check would normally surface a
            // diagnostic at lower time; reaching this point means
            // the spec violates its own meta-type and we should
            // refuse to silently skip it).
            let target = require_field_decl_ref(fields, "target", decl.id)?;

            match category {
                RealizationCategory::Type => {
                    let carrier = require_field_string(fields, "carrier", decl.id)?;
                    let field_bindings = require_field_bindings(dag, fields, decl.id)?;
                    if types
                        .insert(
                            target,
                            TypeRealizationBinding {
                                carrier,
                                is_copy: require_field_bool(fields, "is_copy", decl.id)?,
                                fields: field_bindings,
                            },
                        )
                        .is_some()
                    {
                        return Err(EmitError::DuplicateRealization {
                            declaration: decl.id,
                            detail:
                                "two TypeRealization data items target the same primitive — single authority requires unique targets",
                        });
                    }
                }
                RealizationCategory::TypeInstantiation => {
                    let carrier = require_field_string(fields, "carrier", decl.id)?;
                    if instantiations
                        .insert(target, TypeInstantiationBinding { carrier })
                        .is_some()
                    {
                        return Err(EmitError::DuplicateRealization {
                            declaration: decl.id,
                            detail:
                                "two TypeInstantiationRealization data items target the same generic declaration — single authority requires unique targets",
                        });
                    }
                }
                RealizationCategory::Operator => {
                    let carrier = require_field_string(fields, "carrier", decl.id)?;
                    if !carrier.contains("{lhs}") || !carrier.contains("{rhs}") {
                        return Err(EmitError::MalformedRealization {
                            declaration: decl.id,
                            detail: "OperatorRealization carrier must be a full-expression template containing {lhs} and {rhs}",
                        });
                    }
                    let op = require_field_decl_ref(fields, "op", decl.id)?;
                    if operators.insert((target, op), carrier).is_some() {
                        return Err(EmitError::DuplicateRealization {
                            declaration: decl.id,
                            detail:
                                "two OperatorRealization data items share the same (target, op) pair — single authority requires unique keys",
                        });
                    }
                }
                RealizationCategory::Behavior => {
                    let carrier = require_field_string(fields, "carrier", decl.id)?;
                    if behaviors.insert(target, carrier).is_some() {
                        return Err(EmitError::DuplicateRealization {
                            declaration: decl.id,
                            detail:
                            "two BehaviorRealization data items target the same substrate marker — single authority requires unique targets",
                        });
                    }
                }
                RealizationCategory::Callable => {
                    let strategy = require_callable_strategy(dag, fields, decl.id)?;
                    let expected_arity = match &dag.declaration(target).connective {
                        TypeConnective::Arrow { inputs, .. } => inputs.len(),
                        _ => {
                            return Err(EmitError::MalformedRealization {
                                declaration: decl.id,
                                detail: "CallableRealization target must be an Arrow declaration",
                            })
                        }
                    };
                    let dispositions =
                        require_parameter_dispositions(dag, fields, decl.id, expected_arity)?;
                    if callables.insert(target, strategy).is_some() {
                        return Err(EmitError::DuplicateRealization {
                            declaration: decl.id,
                            detail:
                                "two CallableRealization data items target the same callable declaration — single authority requires unique targets",
                        });
                    }
                    if external_callable_dispositions
                        .insert(target, dispositions)
                        .is_some()
                    {
                        return Err(EmitError::DuplicateRealization {
                            declaration: decl.id,
                            detail:
                                "two CallableRealization data items target the same callable declaration's parameter dispositions — single authority requires unique targets",
                        });
                    }
                }
                RealizationCategory::Pattern => {
                    let binding = require_pattern_realization(dag, fields, decl.id)?;
                    validate_pattern_roles(dag, target, &binding, decl.id)?;
                    if patterns.insert(target, binding).is_some() {
                        return Err(EmitError::DuplicateRealization {
                            declaration: decl.id,
                            detail:
                                "two PatternRealization data items target the same structural sum declaration — single authority requires unique targets",
                        });
                    }
                }
            }
        }

        let syntax = RustLanguageSyntax::build(dag)?;
        let rendering = RenderingModelBinding::build(dag)?;
        let computation = ComputationModelBinding::build(dag)?;
        let execution = TargetExecutionModelBinding::build(dag)?;
        let source_filtering = SourceFilteringBinding::build(
            dag,
            dag.rust_source_filtering_spec()
                .ok_or(EmitError::MissingTargetSyntax("rust_source_filtering"))?,
        )?;
        let clean_emission = CleanEmissionContractBinding::build(dag)?;
        let callable_dispositions =
            derive_callable_dispositions(dag, &external_callable_dispositions)?;
        let (substrate_accessors, substrate_accessor_universe) =
            build_substrate_accessor_index(dag, rust_language_id)?;

        Ok(Self {
            types,
            instantiations,
            operators,
            behaviors,
            callables,
            callable_dispositions,
            patterns,
            syntax,
            rendering,
            computation,
            execution,
            source_filtering,
            clean_emission,
            substrate_accessors,
            substrate_accessor_universe,
        })
    }
}

impl CleanEmissionContractBinding {
    /// Parse the portion of `data rust_clean_emission:
    /// CleanEmissionContract` this emitter consumes. Currently only
    /// `pattern_bindings` and `variant_payload_field_access`
    /// dispatch; other rules parse when their consumers land.
    fn build(dag: &Dag) -> Result<Self, EmitError> {
        let declaration = dag
            .rust_clean_emission_spec()
            .ok_or(EmitError::MissingTargetSyntax("rust_clean_emission"))?;
        let fields = structural_fields_for_decl(dag, declaration)?;
        let pattern_bindings_value = fields
            .iter()
            .find(|(label, _)| label == "pattern_bindings")
            .map(|(_, value)| value)
            .ok_or(EmitError::MalformedTargetSyntax {
                declaration,
                detail: "rust_clean_emission is missing required `pattern_bindings` field",
            })?;
        let pattern_bindings =
            parse_pattern_binding_rule(dag, pattern_bindings_value, declaration)?;
        let variant_payload_field_access_value = fields
            .iter()
            .find(|(label, _)| label == "variant_payload_field_access")
            .map(|(_, value)| value)
            .ok_or(EmitError::MalformedTargetSyntax {
                declaration,
                detail:
                    "rust_clean_emission is missing required `variant_payload_field_access` field",
            })?;
        let variant_payload_field_access = parse_variant_payload_field_access_rule(
            dag,
            variant_payload_field_access_value,
            declaration,
        )?;
        Ok(Self {
            pattern_bindings,
            variant_payload_field_access,
        })
    }
}

fn parse_pattern_binding_rule(
    dag: &Dag,
    value: &FieldValue,
    declaration: DeclarationId,
) -> Result<PatternBindingRuleBinding, EmitError> {
    let FieldValue::Variant {
        constructor,
        payload,
    } = value
    else {
        return Err(EmitError::MalformedTargetSyntax {
            declaration,
            detail: "rust_clean_emission.pattern_bindings must be a PatternBindingRule variant",
        });
    };
    if !payload.is_empty() {
        return Err(EmitError::MalformedTargetSyntax {
            declaration,
            detail: "PatternBindingRule variants must not carry payload fields",
        });
    }
    let variants = dag.pattern_binding_rule_variants();
    let emit_always = variants
        .emit_always
        .ok_or(EmitError::MalformedTargetSyntax {
            declaration,
            detail: "PatternBindingRule.EmitBindingAlways declaration was not found",
        })?;
    let emit_underscore = variants
        .emit_underscore
        .ok_or(EmitError::MalformedTargetSyntax {
            declaration,
            detail: "PatternBindingRule.EmitUnderscoreWhenUnused declaration was not found",
        })?;
    let emit_prefixed = variants
        .emit_prefixed
        .ok_or(EmitError::MalformedTargetSyntax {
            declaration,
            detail: "PatternBindingRule.EmitPrefixedUnderscoreWhenUnused declaration was not found",
        })?;
    let not_applicable = variants
        .not_applicable
        .ok_or(EmitError::MalformedTargetSyntax {
            declaration,
            detail: "PatternBindingRule.NotApplicablePatternBinding declaration was not found",
        })?;
    if *constructor == emit_always {
        Ok(PatternBindingRuleBinding::EmitBindingAlways)
    } else if *constructor == emit_underscore {
        Ok(PatternBindingRuleBinding::EmitUnderscoreWhenUnused)
    } else if *constructor == emit_prefixed {
        Err(EmitError::MalformedTargetSyntax {
            declaration,
            detail: "rust_clean_emission.pattern_bindings cannot use PatternBindingRule.EmitPrefixedUnderscoreWhenUnused; Rust only supports EmitBindingAlways or EmitUnderscoreWhenUnused",
        })
    } else if *constructor == not_applicable {
        Err(EmitError::MalformedTargetSyntax {
            declaration,
            detail: "rust_clean_emission.pattern_bindings cannot use PatternBindingRule.NotApplicablePatternBinding; Rust only supports EmitBindingAlways or EmitUnderscoreWhenUnused",
        })
    } else {
        Err(EmitError::MalformedTargetSyntax {
            declaration,
            detail: "rust_clean_emission.pattern_bindings constructor is not a known PatternBindingRule variant",
        })
    }
}

fn parse_variant_payload_field_access_rule(
    dag: &Dag,
    value: &FieldValue,
    declaration: DeclarationId,
) -> Result<VariantPayloadFieldAccessRuleBinding, EmitError> {
    let FieldValue::Variant {
        constructor,
        payload,
    } = value
    else {
        return Err(EmitError::MalformedTargetSyntax {
            declaration,
            detail:
                "rust_clean_emission.variant_payload_field_access must be a VariantPayloadFieldAccessRule variant",
        });
    };
    if !payload.is_empty() {
        return Err(EmitError::MalformedTargetSyntax {
            declaration,
            detail: "VariantPayloadFieldAccessRule variants must not carry payload fields",
        });
    }
    let variants = dag.variant_payload_field_access_rule_variants();
    let access_from_payload_binding =
        variants
            .access_from_payload_binding
            .ok_or(EmitError::MalformedTargetSyntax {
            declaration,
            detail:
                "VariantPayloadFieldAccessRule.AccessFromPayloadBinding declaration was not found",
        })?;
    let override_named_fields_at_binding_site = variants
        .override_named_fields_at_binding_site
        .ok_or(EmitError::MalformedTargetSyntax {
            declaration,
            detail: "VariantPayloadFieldAccessRule.OverrideNamedFieldsAtBindingSite declaration was not found",
        })?;
    if *constructor == override_named_fields_at_binding_site {
        Ok(VariantPayloadFieldAccessRuleBinding::OverrideNamedFieldsAtBindingSite)
    } else if *constructor == access_from_payload_binding {
        Err(EmitError::MalformedTargetSyntax {
            declaration,
            detail: "rust_clean_emission.variant_payload_field_access cannot use VariantPayloadFieldAccessRule.AccessFromPayloadBinding; Rust requires OverrideNamedFieldsAtBindingSite for named payloads",
        })
    } else {
        Err(EmitError::MalformedTargetSyntax {
            declaration,
            detail: "rust_clean_emission.variant_payload_field_access constructor is not a known VariantPayloadFieldAccessRule variant",
        })
    }
}

/// DB-14: build `(accessor_decl → realization_decl, universe)` for
/// the active target language.
///
/// The map is the per-language lookup used by
/// `render_substrate_accessor` when a callable target IS bound for
/// the active target. The universe is every accessor referenced by
/// any `SubstrateAccessorBinding` across all target languages — it
/// lets the emitter distinguish "callable isn't a substrate
/// accessor at all" (fall through) from "declared substrate
/// accessor, but no binding for this target" (fail closed).
///
/// Single-authority enforcement: duplicate `(accessor × language)`
/// pairs fail closed with `EmitError::DuplicateRealization`.
fn build_substrate_accessor_index(
    dag: &Dag,
    target_language_id: DeclarationId,
) -> Result<
    (
        HashMap<DeclarationId, DeclarationId>,
        HashSet<DeclarationId>,
    ),
    EmitError,
> {
    let mut index: HashMap<DeclarationId, DeclarationId> = HashMap::new();
    let mut universe: HashSet<DeclarationId> = HashSet::new();
    let Some(binding_meta_id) = dag.substrate_accessor_binding_meta() else {
        // No substrate accessor binding type — pre-DB-14 bootstrap
        // or a minimal fixture that didn't load substrate.dag. Empty
        // universe means render_substrate_accessor falls through on
        // every callable, matching pre-DB-14 behavior.
        return Ok((index, universe));
    };
    for decl in dag.declarations() {
        if decl.meta_tag != Some(binding_meta_id) {
            continue;
        }
        let Some(ValueBody::Structural { fields }) = &decl.value_body else {
            return Err(EmitError::MalformedRealization {
                declaration: decl.id,
                detail:
                    "SubstrateAccessorBinding data item has no Structural value_body — bootstrap inhabitance check missed a malformed spec entry",
            });
        };
        let accessor = require_field_decl_ref(fields, "accessor", decl.id)?;
        universe.insert(accessor);
        let language = require_field_decl_ref(fields, "language", decl.id)?;
        if language != target_language_id {
            continue;
        }
        let realization = require_field_decl_ref(fields, "realization", decl.id)?;
        if index.insert(accessor, realization).is_some() {
            return Err(EmitError::DuplicateRealization {
                declaration: decl.id,
                detail:
                    "two SubstrateAccessorBinding data items target the same accessor × language pair — single authority requires a unique realization per target language",
            });
        }
    }
    Ok((index, universe))
}

impl RenderingModelBinding {
    fn build(dag: &Dag) -> Result<Self, EmitError> {
        let rendering_decl = dag
            .rust_rendering_spec()
            .ok_or(EmitError::MissingTargetSyntax("rust_rendering"))?;
        let fields = structural_fields_for_decl(dag, rendering_decl)?;
        Ok(Self {
            read: require_read_strategy(dag, fields, "read", rendering_decl)?,
            construct: require_construct_strategy(dag, fields, "construct", rendering_decl)?,
        })
    }
}

impl ComputationModelBinding {
    fn build(dag: &Dag) -> Result<Self, EmitError> {
        let declaration = dag
            .computation_model_spec()
            .ok_or(EmitError::MissingTargetSyntax("dag_model"))?;
        let fields = structural_fields_for_decl(dag, declaration)?;
        Ok(Self {
            mutability: require_source_mutability(dag, fields, declaration)?,
            purity: require_source_purity(dag, fields, declaration)?,
            structure: require_source_structure(dag, fields, declaration)?,
            iteration: require_source_iteration(dag, fields, declaration)?,
        })
    }

    fn is_canonical_dag(&self) -> bool {
        self.mutability == SourceMutabilityBinding::Immutable
            && self.purity == SourcePurityBinding::Pure
            && self.structure == SourceStructureBinding::ExplicitDag
            && self.iteration == SourceIterationBinding::Bounded
    }
}

impl TargetExecutionModelBinding {
    fn build(dag: &Dag) -> Result<Self, EmitError> {
        let declaration = dag
            .rust_execution_model_spec()
            .ok_or(EmitError::MissingTargetSyntax("rust_execution_model"))?;
        let fields = structural_fields_for_decl(dag, declaration)?;
        Ok(Self {
            memory: require_memory_model(dag, fields, declaration)?,
            scope: require_scope_model(dag, fields, declaration)?,
        })
    }

    fn is_ownership_based(&self) -> bool {
        self.memory == MemoryModelBinding::OwnershipBased
    }

    fn is_lexically_scoped(&self) -> bool {
        self.scope == ScopeModelBinding::LexicalScoping
    }
}

impl InputUseFacts {
    fn build(dag: &Dag, _indexes: &RealizationIndexes) -> Self {
        let mut facts = Self::default();
        let mut order = 0usize;

        // Record every edge — Borrowed and Consumed — so the
        // last-use check can see borrows that come after consumes
        // (a later borrow makes a move unsafe even though it isn't
        // itself a consume).
        let mut record = |port: PortId, key: InputUseKey, order: &mut usize| {
            facts.edge_order.insert(key, *order);
            facts.last_use_order_by_port.insert(port, *order);
            *order += 1;
        };

        for node in dag.nodes() {
            match node {
                Behavior::Transform(transform) => {
                    for (slot, &port) in transform.inputs.iter().enumerate() {
                        let key = InputUseKey {
                            consumer: transform.id,
                            slot: InputSlot::Positional(slot),
                        };
                        record(port, key, &mut order);
                    }
                }
                Behavior::Branch(branch) => {
                    let key = InputUseKey {
                        consumer: branch.id,
                        slot: InputSlot::BranchInput,
                    };
                    record(branch.input, key, &mut order);
                }
                Behavior::Loop(loop_node) => {
                    for (slot, port) in [
                        (InputSlot::LoopSource, Some(loop_node.source)),
                        (InputSlot::LoopInit, Some(loop_node.init)),
                        (InputSlot::LoopBoundCount, loop_node.bound.count_port()),
                    ] {
                        let Some(port) = port else {
                            continue;
                        };
                        let key = InputUseKey {
                            consumer: loop_node.id,
                            slot,
                        };
                        record(port, key, &mut order);
                    }
                }
                Behavior::Value(_) | Behavior::Bind(_) => {}
            }
        }

        facts
    }
}

impl RustLanguageSyntax {
    fn build(dag: &Dag) -> Result<Self, EmitError> {
        let language_decl = dag
            .rust_language_spec()
            .ok_or(EmitError::MissingTargetSyntax("rust_language"))?;
        let fields = structural_fields_for_decl(dag, language_decl)?;
        Ok(Self {
            statements: parse_statement_syntax(
                dag,
                require_field_decl_ref(fields, "statements", language_decl)?,
            )?,
            expressions: parse_expression_syntax(
                dag,
                require_field_decl_ref(fields, "expressions", language_decl)?,
            )?,
            control_flow: parse_control_flow_syntax(
                dag,
                require_field_decl_ref(fields, "control_flow", language_decl)?,
            )?,
            literals: parse_literal_syntax(
                dag,
                require_field_decl_ref(fields, "literals", language_decl)?,
            )?,
            modules: parse_module_syntax(
                dag,
                require_field_decl_ref(fields, "modules", language_decl)?,
            )?,
            functions: parse_function_syntax(
                dag,
                require_field_decl_ref(fields, "functions", language_decl)?,
            )?,
            type_applications: parse_type_application_syntax(
                dag,
                require_field_decl_ref(fields, "type_applications", language_decl)?,
            )?,
            type_definitions: parse_type_definition_syntax(
                dag,
                require_field_decl_ref(fields, "type_definitions", language_decl)?,
            )?,
            record_derive_no_debug: parse_record_derive_template_bundle(
                dag,
                require_field_decl_ref(fields, "record_derive_templates", language_decl)?,
            )?,
            patterns: parse_pattern_match_syntax(
                dag,
                require_field_decl_ref(fields, "patterns", language_decl)?,
            )?,
            collection_ops: parse_collection_ops(
                dag,
                require_field_decl_ref(fields, "collection_ops", language_decl)?,
            )?,
            values: parse_value_construction_syntax(
                dag,
                require_field_decl_ref(fields, "values", language_decl)?,
            )?,
        })
    }
}

fn structural_fields_for_decl(
    dag: &Dag,
    declaration: DeclarationId,
) -> Result<&[(String, FieldValue)], EmitError> {
    let Some(ValueBody::Structural { fields }) = &dag.declaration(declaration).value_body else {
        return Err(EmitError::MalformedTargetSyntax {
            declaration,
            detail: "syntax declaration must carry a structural value body",
        });
    };
    Ok(fields)
}

fn syntax_field_string(
    fields: &[(String, FieldValue)],
    label: &str,
    declaration: DeclarationId,
) -> Result<String, EmitError> {
    require_field_string(fields, label, declaration).map(|s| s.replace("%Q", "\""))
}

fn parse_statement_syntax(
    dag: &Dag,
    declaration: DeclarationId,
) -> Result<StatementSyntaxBinding, EmitError> {
    let fields = structural_fields_for_decl(dag, declaration)?;
    Ok(StatementSyntaxBinding {
        let_binding: syntax_field_string(fields, "let_binding", declaration)?,
    })
}

fn parse_expression_syntax(
    dag: &Dag,
    declaration: DeclarationId,
) -> Result<ExpressionSyntaxBinding, EmitError> {
    let fields = structural_fields_for_decl(dag, declaration)?;
    Ok(ExpressionSyntaxBinding {
        field_access: syntax_field_string(fields, "field_access", declaration)?,
        function_call: syntax_field_string(fields, "function_call", declaration)?,
        closure: syntax_field_string(fields, "closure", declaration)?,
    })
}

fn parse_control_flow_syntax(
    dag: &Dag,
    declaration: DeclarationId,
) -> Result<ControlFlowSyntaxBinding, EmitError> {
    let fields = structural_fields_for_decl(dag, declaration)?;
    Ok(ControlFlowSyntaxBinding {
        if_else: syntax_field_string(fields, "if_else", declaration)?,
    })
}

fn parse_literal_syntax(
    dag: &Dag,
    declaration: DeclarationId,
) -> Result<LiteralSyntaxBinding, EmitError> {
    let fields = structural_fields_for_decl(dag, declaration)?;
    Ok(LiteralSyntaxBinding {
        true_keyword: syntax_field_string(fields, "true_keyword", declaration)?,
        false_keyword: syntax_field_string(fields, "false_keyword", declaration)?,
        string_delimiter: syntax_field_string(fields, "string_delimiter", declaration)?,
    })
}

fn parse_module_syntax(
    dag: &Dag,
    declaration: DeclarationId,
) -> Result<ModuleSyntaxBinding, EmitError> {
    let fields = structural_fields_for_decl(dag, declaration)?;
    Ok(ModuleSyntaxBinding {
        path_separator: syntax_field_string(fields, "path_separator", declaration)?,
    })
}

fn parse_function_syntax(
    dag: &Dag,
    declaration: DeclarationId,
) -> Result<FunctionSyntaxBinding, EmitError> {
    let fields = structural_fields_for_decl(dag, declaration)?;
    Ok(FunctionSyntaxBinding {
        definition: syntax_field_string(fields, "definition", declaration)?,
        definition_exported: syntax_field_string(fields, "definition_exported", declaration)?,
        param_with_type: syntax_field_string(fields, "param_with_type", declaration)?,
        param_separator: syntax_field_string(fields, "param_separator", declaration)?,
    })
}

fn parse_type_application_syntax(
    dag: &Dag,
    declaration: DeclarationId,
) -> Result<TypeApplicationSyntaxBinding, EmitError> {
    let fields = structural_fields_for_decl(dag, declaration)?;
    Ok(TypeApplicationSyntaxBinding {
        optional: syntax_field_string(fields, "optional", declaration)?,
    })
}

fn parse_type_definition_syntax(
    dag: &Dag,
    declaration: DeclarationId,
) -> Result<TypeDefinitionSyntaxBinding, EmitError> {
    let fields = structural_fields_for_decl(dag, declaration)?;
    Ok(TypeDefinitionSyntaxBinding {
        struct_def: syntax_field_string(fields, "struct_def", declaration)?,
        struct_field: syntax_field_string(fields, "struct_field", declaration)?,
        enum_def: syntax_field_string(fields, "enum_def", declaration)?,
        enum_unit_variant: syntax_field_string(fields, "enum_unit_variant", declaration)?,
        enum_data_variant: syntax_field_string(fields, "enum_data_variant", declaration)?,
    })
}

fn parse_record_derive_template_bundle(
    dag: &Dag,
    declaration: DeclarationId,
) -> Result<RecordDeriveTemplateBundleBinding, EmitError> {
    let fields = structural_fields_for_decl(dag, declaration)?;
    Ok(RecordDeriveTemplateBundleBinding {
        struct_def_no_debug: syntax_field_string(fields, "struct_def_no_debug", declaration)?,
        enum_def_no_debug: syntax_field_string(fields, "enum_def_no_debug", declaration)?,
    })
}

fn parse_pattern_match_syntax(
    dag: &Dag,
    declaration: DeclarationId,
) -> Result<PatternMatchSyntaxBinding, EmitError> {
    let fields = structural_fields_for_decl(dag, declaration)?;
    Ok(PatternMatchSyntaxBinding {
        match_expr: syntax_field_string(fields, "match_expr", declaration)?,
        match_arm: syntax_field_string(fields, "match_arm", declaration)?,
        variant_pattern: syntax_field_string(fields, "variant_pattern", declaration)?,
        variant_pattern_positional: syntax_field_string(
            fields,
            "variant_pattern_positional",
            declaration,
        )?,
        variant_pattern_empty: syntax_field_string(fields, "variant_pattern_empty", declaration)?,
        field_binding: syntax_field_string(fields, "field_binding", declaration)?,
        field_binding_separator: syntax_field_string(
            fields,
            "field_binding_separator",
            declaration,
        )?,
        wildcard: syntax_field_string(fields, "wildcard", declaration)?,
    })
}

fn parse_collection_ops(
    dag: &Dag,
    declaration: DeclarationId,
) -> Result<CollectionOpsBinding, EmitError> {
    let fields = structural_fields_for_decl(dag, declaration)?;
    let fold_contract = require_field_decl_ref(fields, "fold_contract", declaration)?;
    let fold = method_contract_single_emit_template_string(dag, fold_contract)?;
    Ok(CollectionOpsBinding {
        concat: syntax_field_string(fields, "concat", declaration)?,
        length: syntax_field_string(fields, "length", declaration)?,
        is_empty: syntax_field_string(fields, "is_empty", declaration)?,
        fold,
        map: syntax_field_string(fields, "map", declaration)?,
        filter: syntax_field_string(fields, "filter", declaration)?,
        contains: syntax_field_string(fields, "contains", declaration)?,
        empty_list: syntax_field_string(fields, "empty_list", declaration)?,
        list_literal: syntax_field_string(fields, "list_literal", declaration)?,
        cons: syntax_field_string(fields, "cons", declaration)?,
    })
}

/// `MethodTemplateContract.emit_template` as a `SingleTemplate` string — the
/// shape collection fold emission supports today (higher-order split is not
/// wired through this path yet).
fn method_contract_single_emit_template_string(
    dag: &Dag,
    contract_decl: DeclarationId,
) -> Result<String, EmitError> {
    let fields = structural_fields_for_decl(dag, contract_decl)?;
    let emit_value = fields
        .iter()
        .find(|(label, _)| label == "emit_template")
        .map(|(_, v)| v)
        .ok_or(EmitError::MalformedTargetSyntax {
            declaration: contract_decl,
            detail: "MethodTemplateContract missing emit_template field",
        })?;
    let FieldValue::Variant {
        constructor,
        ref payload,
    } = emit_value
    else {
        return Err(EmitError::MalformedTargetSyntax {
            declaration: contract_decl,
            detail: "MethodTemplateContract.emit_template must be a sum variant",
        });
    };
    let ctor = dag.declaration(*constructor);
    let Some(ctor_name) = ctor.name.as_deref() else {
        return Err(EmitError::MalformedTargetSyntax {
            declaration: contract_decl,
            detail: "MethodTemplateContract.emit_template variant has no name",
        });
    };
    if ctor_name != "SingleTemplate" {
        return Err(EmitError::MalformedTargetSyntax {
            declaration: contract_decl,
            detail: "collection fold contract must use MethodEmitTemplate.SingleTemplate today",
        });
    }
    let [FieldValue::Literal(LiteralBits::String(template))] = payload.as_slice() else {
        return Err(EmitError::MalformedTargetSyntax {
            declaration: contract_decl,
            detail: "SingleTemplate must carry exactly one string template payload",
        });
    };
    Ok(template.replace("%Q", "\""))
}

fn parse_value_construction_syntax(
    dag: &Dag,
    declaration: DeclarationId,
) -> Result<ValueConstructionSyntaxBinding, EmitError> {
    let fields = structural_fields_for_decl(dag, declaration)?;
    Ok(ValueConstructionSyntaxBinding {
        struct_literal: syntax_field_string(fields, "struct_literal", declaration)?,
        struct_field_init: syntax_field_string(fields, "struct_field_init", declaration)?,
        struct_field_separator: syntax_field_string(fields, "struct_field_separator", declaration)?,
        variant_named_construction: syntax_field_string(
            fields,
            "variant_named_construction",
            declaration,
        )?,
    })
}

fn render_named_template(template: &str, bindings: &[(&str, &str)]) -> String {
    let bindings: HashMap<&str, &str> = bindings.iter().copied().collect();
    let chars: Vec<char> = template.chars().collect();
    let mut rendered = String::with_capacity(template.len());
    let mut i = 0;

    while i < chars.len() {
        match chars[i] {
            '{' => {
                if i + 1 < chars.len() && chars[i + 1] == '{' {
                    rendered.push('{');
                    i += 2;
                    continue;
                }

                let mut j = i + 1;
                while j < chars.len() && chars[j] != '}' {
                    j += 1;
                }
                if j < chars.len() {
                    let key: String = chars[i + 1..j].iter().collect();
                    if is_template_placeholder_key(&key) {
                        if let Some(value) = bindings.get(key.as_str()) {
                            rendered.push_str(value);
                        } else {
                            rendered.push('{');
                            rendered.push_str(&key);
                            rendered.push('}');
                        }
                        i = j + 1;
                        continue;
                    }
                }

                rendered.push('{');
                i += 1;
            }
            '}' => {
                if i + 1 < chars.len() && chars[i + 1] == '}' {
                    rendered.push('}');
                    i += 2;
                } else {
                    rendered.push('}');
                    i += 1;
                }
            }
            ch => {
                rendered.push(ch);
                i += 1;
            }
        }
    }

    rendered
}

/// Local name for a destructured field of a multi-field struct-variant
/// payload. Used by both the pattern emitter and the arm-body renderer
/// so the aliased destructure and the payload-binding field routing
/// stay in lockstep. The leading `__` avoids colliding with user
/// identifiers and silences unused-binding warnings on fields the arm
/// body never references.
fn destructured_field_alias(binding_name: &str, field_label: &str) -> String {
    format!("__{binding_name}_{field_label}")
}

fn find_resolved_branch_path(branch: &BranchNode, variant_id: DeclarationId) -> Option<&Path> {
    branch.paths.iter().find(|path| match path.pattern {
        BranchPattern::ResolvedVariant(id) => id == variant_id,
        BranchPattern::UnresolvedVariant { .. } => false,
    })
}

fn is_template_placeholder_key(key: &str) -> bool {
    let mut chars = key.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first == '_' || first.is_ascii_alphabetic()) {
        return false;
    }
    chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}

fn join_rendered(values: &[String], separator: &str) -> String {
    values.join(separator)
}

/// Required-field lookup: returns `Err` if the field is absent or
/// not a `FieldValue::Reference`. Used at realization-index build
/// time to fail-closed on spec entries that should have been
/// rejected by `lower_record_to_structural`'s inhabitance check.
fn require_field_decl_ref(
    fields: &[(String, FieldValue)],
    label: &str,
    declaration: DeclarationId,
) -> Result<DeclarationId, EmitError> {
    fields
        .iter()
        .find(|(l, _)| l == label)
        .and_then(|(_, v)| match v {
            FieldValue::Reference(id) => Some(*id),
            _ => None,
        })
        .ok_or(EmitError::MalformedRealization {
            declaration,
            detail:
                "realization data item is missing a required Reference field or has wrong shape — see lower_record_to_structural inhabitance check",
        })
}

/// Required-field lookup: returns `Err` if the field is absent or
/// not a `FieldValue::Literal(LiteralBits::String)`. Same
/// fail-closed semantics as `require_field_decl_ref`.
fn require_field_string(
    fields: &[(String, FieldValue)],
    label: &str,
    declaration: DeclarationId,
) -> Result<String, EmitError> {
    fields
        .iter()
        .find(|(l, _)| l == label)
        .and_then(|(_, v)| match v {
            FieldValue::Literal(LiteralBits::String(s)) => Some(s.clone()),
            _ => None,
        })
        .ok_or(EmitError::MalformedRealization {
            declaration,
            detail:
                "realization data item is missing a required String field or has wrong shape — see lower_record_to_structural inhabitance check",
        })
}

fn require_field_bool(
    fields: &[(String, FieldValue)],
    label: &str,
    declaration: DeclarationId,
) -> Result<bool, EmitError> {
    fields
        .iter()
        .find(|(l, _)| l == label)
        .and_then(|(_, v)| match v {
            FieldValue::Literal(LiteralBits::Bool(b)) => Some(*b),
            _ => None,
        })
        .ok_or(EmitError::MalformedRealization {
            declaration,
            detail:
                "realization data item is missing a required Bool field or has wrong shape — see lower_record_to_structural inhabitance check",
        })
}

fn require_field_bindings(
    dag: &Dag,
    fields: &[(String, FieldValue)],
    declaration: DeclarationId,
) -> Result<HashMap<String, FieldBindingBinding>, EmitError> {
    let value = fields
        .iter()
        .find(|(label, _)| label == "fields")
        .map(|(_, value)| value)
        .ok_or(EmitError::MalformedRealization {
            declaration,
            detail: "TypeRealization is missing required `fields` list",
        })?;
    let FieldValue::List(entries) = value else {
        return Err(EmitError::MalformedRealization {
            declaration,
            detail: "TypeRealization.fields must be a structural list",
        });
    };
    let mut bindings = HashMap::new();
    for entry in entries {
        let FieldValue::Record(fields) = entry else {
            return Err(EmitError::MalformedRealization {
                declaration,
                detail: "FieldBinding entries must be structural records",
            });
        };
        let dag_name = fields
            .iter()
            .find(|(label, _)| label == "dag_name")
            .and_then(|(_, value)| match value {
                FieldValue::Literal(LiteralBits::String(name)) => Some(name.clone()),
                _ => None,
            })
            .ok_or(EmitError::MalformedRealization {
                declaration,
                detail: "FieldBinding.dag_name must be a String literal",
            })?;
        let rust_access = fields
            .iter()
            .find(|(label, _)| label == "access")
            .ok_or(EmitError::MalformedRealization {
                declaration,
                detail: "FieldBinding.access is required",
            })
            .and_then(|(_, value)| parse_rust_field_access(dag, value, declaration))?;
        let borrowed_read = fields
            .iter()
            .find(|(label, _)| label == "borrowed_read")
            .and_then(|(_, value)| match value {
                FieldValue::Literal(LiteralBits::Bool(value)) => Some(*value),
                _ => None,
            })
            .unwrap_or(false);
        if bindings
            .insert(
                dag_name,
                FieldBindingBinding {
                    access: rust_access,
                    borrowed_read,
                },
            )
            .is_some()
        {
            return Err(EmitError::DuplicateRealization {
                declaration,
                detail: "TypeRealization.fields contains duplicate dag_name entries",
            });
        }
    }
    Ok(bindings)
}

fn parse_rust_field_access(
    dag: &Dag,
    value: &FieldValue,
    declaration: DeclarationId,
) -> Result<RustFieldAccessBinding, EmitError> {
    let FieldValue::Variant {
        constructor,
        payload,
    } = value
    else {
        return Err(EmitError::MalformedRealization {
            declaration,
            detail: "FieldBinding.access must be a FieldAccess variant",
        });
    };
    if payload.len() != 1 {
        return Err(EmitError::MalformedRealization {
            declaration,
            detail: "FieldAccess variants must carry exactly one String payload",
        });
    }
    let name = match &payload[0] {
        FieldValue::Literal(LiteralBits::String(name)) => name.clone(),
        _ => {
            return Err(EmitError::MalformedRealization {
                declaration,
                detail: "FieldAccess payload must be a String literal",
            });
        }
    };
    let direct_field = named_variant_id(dag, "FieldAccess", "DirectField").ok_or(
        EmitError::MalformedRealization {
            declaration,
            detail: "FieldAccess.DirectField declaration was not found",
        },
    )?;
    let accessor_method = named_variant_id(dag, "FieldAccess", "AccessorMethod").ok_or(
        EmitError::MalformedRealization {
            declaration,
            detail: "FieldAccess.AccessorMethod declaration was not found",
        },
    )?;
    if *constructor == direct_field {
        Ok(RustFieldAccessBinding::DirectField(name))
    } else if *constructor == accessor_method {
        Ok(RustFieldAccessBinding::AccessorMethod(name))
    } else {
        Err(EmitError::MalformedRealization {
            declaration,
            detail: "RustFieldAccess constructor must be DirectField or AccessorMethod",
        })
    }
}

fn require_callable_strategy(
    dag: &Dag,
    fields: &[(String, FieldValue)],
    declaration: DeclarationId,
) -> Result<RustCallableStrategyBinding, EmitError> {
    let value = fields
        .iter()
        .find(|(label, _)| label == "strategy")
        .map(|(_, value)| value)
        .ok_or(EmitError::MalformedRealization {
            declaration,
            detail: "CallableRealization is missing required `strategy` field",
        })?;
    let FieldValue::Variant {
        constructor,
        payload,
    } = value
    else {
        return Err(EmitError::MalformedRealization {
            declaration,
            detail: "CallableRealization.strategy must be a CallableStrategy variant",
        });
    };
    if !payload.is_empty() {
        return Err(EmitError::MalformedRealization {
            declaration,
            detail: "CallableStrategy variants must not carry payload fields",
        });
    }
    let variants = dag.callable_strategy_variants();
    let strategies = [
        (variants.list_empty, RustCallableStrategyBinding::ListEmpty),
        (
            variants.list_singleton,
            RustCallableStrategyBinding::ListSingleton,
        ),
        (variants.list_cons, RustCallableStrategyBinding::ListCons),
        (
            variants.list_concat,
            RustCallableStrategyBinding::ListConcat,
        ),
        (
            variants.list_length,
            RustCallableStrategyBinding::ListLength,
        ),
        (
            variants.list_is_empty,
            RustCallableStrategyBinding::ListIsEmpty,
        ),
        (variants.list_fold, RustCallableStrategyBinding::ListFold),
        (variants.list_map, RustCallableStrategyBinding::ListMap),
        (
            variants.list_filter,
            RustCallableStrategyBinding::ListFilter,
        ),
        (
            variants.list_contains,
            RustCallableStrategyBinding::ListContains,
        ),
    ];
    for (variant_id, binding) in strategies {
        let Some(variant_id) = variant_id else {
            return Err(EmitError::MalformedRealization {
                declaration,
                detail: "CallableStrategy variant declaration was not found",
            });
        };
        if *constructor == variant_id {
            return Ok(binding);
        }
    }
    Err(EmitError::MalformedRealization {
        declaration,
        detail:
            "RustCallableStrategy constructor must be ListEmpty/ListSingleton/ListCons/ListConcat/ListLength/ListIsEmpty/ListFold/ListMap/ListFilter/ListContains",
    })
}

/// Parse and validate the `parameters` list on a CallableRealization.
/// Each entry is a `CallableParameter` record with `slot: Int` and
/// `disposition: ParameterDisposition`. Returns a Vec indexed by slot
/// (length == `expected_arity`). Validation makes arity and order
/// drift unrepresentable: any missing slot, duplicate slot, or
/// out-of-range slot fails closed at build time instead of silently
/// defaulting to Borrowed at use time.
fn require_parameter_dispositions(
    dag: &Dag,
    fields: &[(String, FieldValue)],
    declaration: DeclarationId,
    expected_arity: usize,
) -> Result<Vec<ParameterDispositionBinding>, EmitError> {
    let value = fields
        .iter()
        .find(|(label, _)| label == "parameters")
        .map(|(_, value)| value)
        .ok_or(EmitError::MalformedRealization {
            declaration,
            detail: "CallableRealization is missing required `parameters` field",
        })?;
    let FieldValue::List(entries) = value else {
        return Err(EmitError::MalformedRealization {
            declaration,
            detail: "CallableRealization.parameters must be a structural list",
        });
    };
    if entries.len() != expected_arity {
        return Err(EmitError::MalformedRealization {
            declaration,
            detail: "CallableRealization.parameters length does not match the callable's Arrow input arity",
        });
    }
    let mut filled: Vec<Option<ParameterDispositionBinding>> = vec![None; expected_arity];
    for entry in entries {
        let (slot, disposition) = parse_callable_parameter(dag, entry, declaration)?;
        if slot >= expected_arity {
            return Err(EmitError::MalformedRealization {
                declaration,
                detail: "CallableParameter.slot is out of range for the callable's declared arity",
            });
        }
        if filled[slot].is_some() {
            return Err(EmitError::MalformedRealization {
                declaration,
                detail: "CallableParameter.slot is duplicated within parameters list",
            });
        }
        filled[slot] = Some(disposition);
    }
    filled
        .into_iter()
        .map(|opt| {
            opt.ok_or(EmitError::MalformedRealization {
                declaration,
                detail: "CallableRealization.parameters does not cover every slot in [0, arity)",
            })
        })
        .collect()
}

fn parse_callable_parameter(
    dag: &Dag,
    value: &FieldValue,
    declaration: DeclarationId,
) -> Result<(usize, ParameterDispositionBinding), EmitError> {
    let FieldValue::Record(fields) = value else {
        return Err(EmitError::MalformedRealization {
            declaration,
            detail: "CallableRealization.parameters entries must be CallableParameter records",
        });
    };
    let slot = fields
        .iter()
        .find(|(label, _)| label == "slot")
        .map(|(_, value)| value)
        .ok_or(EmitError::MalformedRealization {
            declaration,
            detail: "CallableParameter is missing required `slot` field",
        })?;
    let FieldValue::Literal(LiteralBits::Int(slot_int)) = slot else {
        return Err(EmitError::MalformedRealization {
            declaration,
            detail: "CallableParameter.slot must be an Int literal",
        });
    };
    if *slot_int < 0 {
        return Err(EmitError::MalformedRealization {
            declaration,
            detail: "CallableParameter.slot must be non-negative",
        });
    }
    let disposition_value = fields
        .iter()
        .find(|(label, _)| label == "disposition")
        .map(|(_, value)| value)
        .ok_or(EmitError::MalformedRealization {
            declaration,
            detail: "CallableParameter is missing required `disposition` field",
        })?;
    let disposition = parse_parameter_disposition(dag, disposition_value, declaration)?;
    Ok((*slot_int as usize, disposition))
}

fn parse_parameter_disposition(
    dag: &Dag,
    value: &FieldValue,
    declaration: DeclarationId,
) -> Result<ParameterDispositionBinding, EmitError> {
    let FieldValue::Variant {
        constructor,
        payload,
    } = value
    else {
        return Err(EmitError::MalformedRealization {
            declaration,
            detail: "CallableParameter.disposition must be a ParameterDisposition variant",
        });
    };
    if !payload.is_empty() {
        return Err(EmitError::MalformedRealization {
            declaration,
            detail: "ParameterDisposition variants must not carry payload fields",
        });
    }
    let borrowed = named_variant_id(dag, "ParameterDisposition", "Borrowed").ok_or(
        EmitError::MalformedRealization {
            declaration,
            detail: "ParameterDisposition.Borrowed declaration was not found",
        },
    )?;
    let consumed = named_variant_id(dag, "ParameterDisposition", "Consumed").ok_or(
        EmitError::MalformedRealization {
            declaration,
            detail: "ParameterDisposition.Consumed declaration was not found",
        },
    )?;
    if *constructor == borrowed {
        Ok(ParameterDispositionBinding::Borrowed)
    } else if *constructor == consumed {
        Ok(ParameterDispositionBinding::Consumed)
    } else {
        Err(EmitError::MalformedRealization {
            declaration,
            detail: "ParameterDisposition constructor must be Borrowed or Consumed",
        })
    }
}

/// Structural boundary check for `PatternRealization.empty_variant` /
/// `cons_variant`. These fields carry typed `DeclarationRef`s, but
/// `DeclarationRef` alone cannot express the shape constraint "must be
/// a variant of `target`'s Disj" — so the spec grammar admits
/// `empty_variant: Int` even though it is nonsensical.
///
/// Reject-at-boundary: after parsing a realization, walk `target` to
/// its Disj and verify both role refs match one of the variants' `ty`,
/// and that they are distinct. A malformed spec surfaces loudly via
/// `MalformedRealization` instead of silently producing emitter output
/// that never finds the named branch arm at render time.
fn validate_pattern_roles(
    dag: &Dag,
    target: DeclarationId,
    binding: &PatternRealizationBinding,
    declaration: DeclarationId,
) -> Result<(), EmitError> {
    let disj_id = walk_to_disj(dag, target).ok_or(EmitError::MalformedRealization {
        declaration,
        detail:
            "PatternRealization.target must resolve to a Disj — empty_variant / cons_variant have no target to range over otherwise",
    })?;
    let TypeConnective::Disj { variants } = &dag.declaration(disj_id).connective else {
        return Err(EmitError::MalformedRealization {
            declaration,
            detail: "walk_to_disj returned a non-Disj declaration (internal invariant violation)",
        });
    };
    let has_empty = variants.iter().any(|v| v.ty == binding.empty_variant);
    if !has_empty {
        return Err(EmitError::MalformedRealization {
            declaration,
            detail:
                "PatternRealization.empty_variant must be a variant of `target` — structural boundary check rejects unrelated DeclarationRefs",
        });
    }
    let has_cons = variants.iter().any(|v| v.ty == binding.cons_variant);
    if !has_cons {
        return Err(EmitError::MalformedRealization {
            declaration,
            detail:
                "PatternRealization.cons_variant must be a variant of `target` — structural boundary check rejects unrelated DeclarationRefs",
        });
    }
    if binding.empty_variant == binding.cons_variant {
        return Err(EmitError::MalformedRealization {
            declaration,
            detail:
                "PatternRealization.empty_variant and cons_variant must be distinct variants of `target`",
        });
    }
    Ok(())
}

fn require_pattern_realization(
    dag: &Dag,
    fields: &[(String, FieldValue)],
    declaration: DeclarationId,
) -> Result<PatternRealizationBinding, EmitError> {
    let value = fields
        .iter()
        .find(|(label, _)| label == "strategy")
        .map(|(_, value)| value)
        .ok_or(EmitError::MalformedRealization {
            declaration,
            detail: "PatternRealization is missing required `strategy` field",
        })?;
    match parse_pattern_strategy(dag, value) {
        Ok(PatternStrategyBinding::VectorList) => {}
        Err(detail) => {
            return Err(EmitError::MalformedRealization {
                declaration,
                detail,
            });
        }
    }
    Ok(PatternRealizationBinding {
        empty_variant: require_field_decl_ref(fields, "empty_variant", declaration)?,
        cons_variant: require_field_decl_ref(fields, "cons_variant", declaration)?,
        scrutinee: require_field_string(fields, "scrutinee", declaration)?,
        empty_pattern: require_field_string(fields, "empty_pattern", declaration)?,
        cons_pattern: require_field_string(fields, "cons_pattern", declaration)?,
        head_expr: require_field_string(fields, "head_expr", declaration)?,
        tail_expr: require_field_string(fields, "tail_expr", declaration)?,
    })
}

fn require_read_strategy(
    dag: &Dag,
    fields: &[(String, FieldValue)],
    label: &str,
    declaration: DeclarationId,
) -> Result<ReadStrategyBinding, EmitError> {
    let value = fields
        .iter()
        .find(|(field_label, _)| field_label == label)
        .map(|(_, value)| value)
        .ok_or(EmitError::MalformedTargetSyntax {
            declaration,
            detail: "RenderingModel is missing a required field",
        })?;
    let FieldValue::Variant {
        constructor,
        payload,
    } = value
    else {
        return Err(EmitError::MalformedTargetSyntax {
            declaration,
            detail: "RenderingModel.read must be a ReadStrategy variant",
        });
    };
    if !payload.is_empty() {
        return Err(EmitError::MalformedTargetSyntax {
            declaration,
            detail: "ReadStrategy variants must not carry payload fields",
        });
    }
    let borrow_variant = named_variant_id(dag, "ReadStrategy", "Borrow").ok_or(
        EmitError::MalformedTargetSyntax {
            declaration,
            detail: "ReadStrategy.Borrow declaration was not found",
        },
    )?;
    let pass_variant = named_variant_id(dag, "ReadStrategy", "PassByValue").ok_or(
        EmitError::MalformedTargetSyntax {
            declaration,
            detail: "ReadStrategy.PassByValue declaration was not found",
        },
    )?;
    if *constructor == borrow_variant {
        Ok(ReadStrategyBinding::Borrow)
    } else if *constructor == pass_variant {
        Ok(ReadStrategyBinding::PassByValue)
    } else {
        Err(EmitError::MalformedTargetSyntax {
            declaration,
            detail: "RenderingModel.read must be Borrow or PassByValue",
        })
    }
}

fn require_construct_strategy(
    dag: &Dag,
    fields: &[(String, FieldValue)],
    label: &str,
    declaration: DeclarationId,
) -> Result<ConstructStrategyBinding, EmitError> {
    let value = fields
        .iter()
        .find(|(field_label, _)| field_label == label)
        .map(|(_, value)| value)
        .ok_or(EmitError::MalformedTargetSyntax {
            declaration,
            detail: "RenderingModel is missing a required field",
        })?;
    let FieldValue::Variant {
        constructor,
        payload,
    } = value
    else {
        return Err(EmitError::MalformedTargetSyntax {
            declaration,
            detail: "RenderingModel.construct must be a ConstructStrategy variant",
        });
    };
    if !payload.is_empty() {
        return Err(EmitError::MalformedTargetSyntax {
            declaration,
            detail: "ConstructStrategy variants must not carry payload fields",
        });
    }
    let copy_or_clone = named_variant_id(dag, "ConstructStrategy", "CopyOrClone").ok_or(
        EmitError::MalformedTargetSyntax {
            declaration,
            detail: "ConstructStrategy.CopyOrClone declaration was not found",
        },
    )?;
    let pass_variant = named_variant_id(dag, "ConstructStrategy", "PassByValue").ok_or(
        EmitError::MalformedTargetSyntax {
            declaration,
            detail: "ConstructStrategy.PassByValue declaration was not found",
        },
    )?;
    if *constructor == copy_or_clone {
        Ok(ConstructStrategyBinding::CopyOrClone)
    } else if *constructor == pass_variant {
        Ok(ConstructStrategyBinding::PassByValue)
    } else {
        Err(EmitError::MalformedTargetSyntax {
            declaration,
            detail: "RenderingModel.construct must be CopyOrClone or PassByValue",
        })
    }
}

fn require_source_mutability(
    dag: &Dag,
    fields: &[(String, FieldValue)],
    declaration: DeclarationId,
) -> Result<SourceMutabilityBinding, EmitError> {
    let value = require_unit_variant_field(fields, "mutability", declaration)?;
    let immutable = named_variant_id(dag, "Mutability", "Immutable").ok_or(
        EmitError::MalformedTargetSyntax {
            declaration,
            detail: "Mutability.Immutable declaration was not found",
        },
    )?;
    let mutable =
        named_variant_id(dag, "Mutability", "Mutable").ok_or(EmitError::MalformedTargetSyntax {
            declaration,
            detail: "Mutability.Mutable declaration was not found",
        })?;
    if value == immutable {
        Ok(SourceMutabilityBinding::Immutable)
    } else if value == mutable {
        Ok(SourceMutabilityBinding::Mutable)
    } else {
        Err(EmitError::MalformedTargetSyntax {
            declaration,
            detail: "ComputationModel.mutability must be Immutable or Mutable",
        })
    }
}

fn require_source_purity(
    dag: &Dag,
    fields: &[(String, FieldValue)],
    declaration: DeclarationId,
) -> Result<SourcePurityBinding, EmitError> {
    let value = require_unit_variant_field(fields, "purity", declaration)?;
    let pure = named_variant_id(dag, "Purity", "Pure").ok_or(EmitError::MalformedTargetSyntax {
        declaration,
        detail: "Purity.Pure declaration was not found",
    })?;
    let effectful =
        named_variant_id(dag, "Purity", "Effectful").ok_or(EmitError::MalformedTargetSyntax {
            declaration,
            detail: "Purity.Effectful declaration was not found",
        })?;
    if value == pure {
        Ok(SourcePurityBinding::Pure)
    } else if value == effectful {
        Ok(SourcePurityBinding::Effectful)
    } else {
        Err(EmitError::MalformedTargetSyntax {
            declaration,
            detail: "ComputationModel.purity must be Pure or Effectful",
        })
    }
}

fn require_source_structure(
    dag: &Dag,
    fields: &[(String, FieldValue)],
    declaration: DeclarationId,
) -> Result<SourceStructureBinding, EmitError> {
    let value = require_unit_variant_field(fields, "structure", declaration)?;
    let explicit = named_variant_id(dag, "Structure", "ExplicitDAG").ok_or(
        EmitError::MalformedTargetSyntax {
            declaration,
            detail: "Structure.ExplicitDAG declaration was not found",
        },
    )?;
    let arbitrary = named_variant_id(dag, "Structure", "Arbitrary").ok_or(
        EmitError::MalformedTargetSyntax {
            declaration,
            detail: "Structure.Arbitrary declaration was not found",
        },
    )?;
    if value == explicit {
        Ok(SourceStructureBinding::ExplicitDag)
    } else if value == arbitrary {
        Ok(SourceStructureBinding::Arbitrary)
    } else {
        Err(EmitError::MalformedTargetSyntax {
            declaration,
            detail: "ComputationModel.structure must be ExplicitDAG or Arbitrary",
        })
    }
}

fn require_source_iteration(
    dag: &Dag,
    fields: &[(String, FieldValue)],
    declaration: DeclarationId,
) -> Result<SourceIterationBinding, EmitError> {
    let value = require_unit_variant_field(fields, "iteration", declaration)?;
    let bounded =
        named_variant_id(dag, "Iteration", "Bounded").ok_or(EmitError::MalformedTargetSyntax {
            declaration,
            detail: "Iteration.Bounded declaration was not found",
        })?;
    let unbounded = named_variant_id(dag, "Iteration", "Unbounded").ok_or(
        EmitError::MalformedTargetSyntax {
            declaration,
            detail: "Iteration.Unbounded declaration was not found",
        },
    )?;
    if value == bounded {
        Ok(SourceIterationBinding::Bounded)
    } else if value == unbounded {
        Ok(SourceIterationBinding::Unbounded)
    } else {
        Err(EmitError::MalformedTargetSyntax {
            declaration,
            detail: "ComputationModel.iteration must be Bounded or Unbounded",
        })
    }
}

fn require_memory_model(
    dag: &Dag,
    fields: &[(String, FieldValue)],
    declaration: DeclarationId,
) -> Result<MemoryModelBinding, EmitError> {
    let value = require_unit_variant_field(fields, "memory", declaration)?;
    let variants = [
        (
            "ValueOnly",
            MemoryModelBinding::ValueOnly,
            "MemoryModel.ValueOnly declaration was not found",
        ),
        (
            "GarbageCollected",
            MemoryModelBinding::GarbageCollected,
            "MemoryModel.GarbageCollected declaration was not found",
        ),
        (
            "RefCounted",
            MemoryModelBinding::RefCounted,
            "MemoryModel.RefCounted declaration was not found",
        ),
        (
            "OwnershipBased",
            MemoryModelBinding::OwnershipBased,
            "MemoryModel.OwnershipBased declaration was not found",
        ),
    ];
    for (variant_name, binding, detail) in variants {
        let variant = named_variant_id(dag, "MemoryModel", variant_name).ok_or(
            EmitError::MalformedTargetSyntax {
                declaration,
                detail,
            },
        )?;
        if value == variant {
            return Ok(binding);
        }
    }
    Err(EmitError::MalformedTargetSyntax {
        declaration,
        detail:
            "TargetExecutionModel.memory must be ValueOnly/GarbageCollected/RefCounted/OwnershipBased",
    })
}

fn require_scope_model(
    dag: &Dag,
    fields: &[(String, FieldValue)],
    declaration: DeclarationId,
) -> Result<ScopeModelBinding, EmitError> {
    let value = require_unit_variant_field(fields, "scope", declaration)?;
    let variants = [
        (
            "LexicalScoping",
            ScopeModelBinding::LexicalScoping,
            "ScopeModel.LexicalScoping declaration was not found",
        ),
        (
            "DynamicScoping",
            ScopeModelBinding::DynamicScoping,
            "ScopeModel.DynamicScoping declaration was not found",
        ),
    ];
    for (variant_name, binding, detail) in variants {
        let variant = named_variant_id(dag, "ScopeModel", variant_name).ok_or(
            EmitError::MalformedTargetSyntax {
                declaration,
                detail,
            },
        )?;
        if value == variant {
            return Ok(binding);
        }
    }
    Err(EmitError::MalformedTargetSyntax {
        declaration,
        detail: "TargetExecutionModel.scope must be LexicalScoping or DynamicScoping",
    })
}

fn require_unit_variant_field(
    fields: &[(String, FieldValue)],
    label: &str,
    declaration: DeclarationId,
) -> Result<DeclarationId, EmitError> {
    let value = fields
        .iter()
        .find(|(field_label, _)| field_label == label)
        .map(|(_, value)| value)
        .ok_or(EmitError::MalformedTargetSyntax {
            declaration,
            detail: "target-model declaration is missing a required field",
        })?;
    let FieldValue::Variant {
        constructor,
        payload,
    } = value
    else {
        return Err(EmitError::MalformedTargetSyntax {
            declaration,
            detail: "target-model field must be a unit variant",
        });
    };
    if !payload.is_empty() {
        return Err(EmitError::MalformedTargetSyntax {
            declaration,
            detail: "target-model variants must not carry payload fields",
        });
    }
    Ok(*constructor)
}

fn named_variant_id(dag: &Dag, parent_name: &str, variant_label: &str) -> Option<DeclarationId> {
    let parent = dag.declaration_by_name(parent_name)?;
    let TypeConnective::Disj { variants } = &parent.connective else {
        return None;
    };
    variants
        .iter()
        .find(|variant| variant.label == variant_label)
        .map(|variant| variant.ty)
}

fn derive_callable_dispositions(
    dag: &Dag,
    external: &HashMap<DeclarationId, Vec<ParameterDispositionBinding>>,
) -> Result<HashMap<DeclarationId, Vec<ParameterDispositionBinding>>, EmitError> {
    let mut dispositions = external.clone();
    let user_defined: Vec<DeclarationId> = dag
        .declarations()
        .iter()
        .filter_map(|decl| match &decl.connective {
            TypeConnective::Arrow {
                body: ArrowBody::UserDefined(_),
                ..
            } => Some(decl.id),
            _ => None,
        })
        .collect();

    for declaration in &user_defined {
        let TypeConnective::Arrow { inputs, .. } = &dag.declaration(*declaration).connective else {
            continue;
        };
        dispositions
            .entry(*declaration)
            .or_insert_with(|| vec![ParameterDispositionBinding::Borrowed; inputs.len()]);
    }

    loop {
        let mut changed = false;
        for declaration in &user_defined {
            let next = analyze_user_defined_callable(dag, *declaration, &dispositions)?;
            let entry = dispositions
                .entry(*declaration)
                .or_insert_with(|| next.clone());
            let merged: Vec<_> = entry
                .iter()
                .copied()
                .zip(next.iter().copied())
                .map(|(current, observed)| current.merge(observed))
                .collect();
            if *entry != merged {
                *entry = merged;
                changed = true;
            }
        }
        if !changed {
            return Ok(dispositions);
        }
    }
}

fn analyze_user_defined_callable(
    dag: &Dag,
    declaration: DeclarationId,
    dispositions: &HashMap<DeclarationId, Vec<ParameterDispositionBinding>>,
) -> Result<Vec<ParameterDispositionBinding>, EmitError> {
    let TypeConnective::Arrow { inputs, body, .. } = &dag.declaration(declaration).connective
    else {
        return Err(EmitError::MalformedUserDefinedCallable {
            declaration,
            detail: "callable declaration is not an Arrow",
        });
    };
    let ArrowBody::UserDefined(bind_id) = body else {
        return Err(EmitError::MalformedUserDefinedCallable {
            declaration,
            detail: "user-defined callable does not have a UserDefined Arrow body",
        });
    };
    let Some(bind) = (*bind_id).bind_opt(dag) else {
        return Err(EmitError::MalformedUserDefinedCallable {
            declaration,
            detail: "user-defined callable body does not point to a Bind node",
        });
    };
    if bind.params.len() < inputs.len() {
        return Err(EmitError::MalformedUserDefinedCallable {
            declaration,
            detail: "Bind.params has fewer ports than the Arrow declares inputs",
        });
    }

    let runtime_params = &bind.params[bind.params.len() - inputs.len()..];
    let runtime_index: HashMap<PortId, usize> = runtime_params
        .iter()
        .copied()
        .enumerate()
        .map(|(idx, port)| (port, idx))
        .collect();
    let mut observed = vec![ParameterDispositionBinding::Borrowed; inputs.len()];
    let mut queue = vec![bind.value];
    if let Some(&index) = runtime_index.get(&bind.value) {
        observed[index] = observed[index].merge(ParameterDispositionBinding::Consumed);
    }
    let mut expanded = HashSet::new();

    while let Some(port) = queue.pop() {
        if !expanded.insert(port) {
            continue;
        }
        let Some(producer) = dag.port(port).produced_by else {
            continue;
        };
        match dag.node(producer) {
            Behavior::Value(_) => {}
            Behavior::Transform(t) => {
                for (slot, input) in t.inputs.iter().copied().enumerate() {
                    let disposition = transform_input_disposition(dag, t, slot, dispositions);
                    if let Some(&index) = runtime_index.get(&input) {
                        observed[index] = observed[index].merge(disposition);
                    }
                    queue.push(input);
                }
            }
            Behavior::Branch(b) => {
                if let Some(&index) = runtime_index.get(&b.input) {
                    observed[index] = observed[index].merge(ParameterDispositionBinding::Borrowed);
                }
                queue.push(b.input);
                for path in &b.paths {
                    if let Some(&index) = runtime_index.get(&path.output) {
                        observed[index] =
                            observed[index].merge(ParameterDispositionBinding::Consumed);
                    }
                    queue.push(path.output);
                }
            }
            Behavior::Loop(l) => {
                for input in [Some(l.source), Some(l.init), l.bound.count_port()] {
                    let Some(input) = input else {
                        continue;
                    };
                    if let Some(&index) = runtime_index.get(&input) {
                        observed[index] =
                            observed[index].merge(ParameterDispositionBinding::Borrowed);
                    }
                    queue.push(input);
                }
                let body_port = super::behavior_result_port(dag.node(l.body));
                if let Some(&index) = runtime_index.get(&body_port) {
                    observed[index] = observed[index].merge(ParameterDispositionBinding::Consumed);
                }
                queue.push(body_port);
            }
            Behavior::Bind(b) => {
                // Bind is a naming node (thesis: let-bindings give a
                // name to a value, they do not consume it). The
                // disposition of the bound value is determined by the
                // downstream consumer of the bound name, not by the
                // Bind itself. Walk through transparently.
                queue.push(b.value);
            }
        }
    }

    Ok(observed)
}

fn transform_input_disposition(
    dag: &Dag,
    transform: &TransformNode,
    slot: usize,
    dispositions: &HashMap<DeclarationId, Vec<ParameterDispositionBinding>>,
) -> ParameterDispositionBinding {
    match &transform.target {
        TransformTarget::Operator(_) | TransformTarget::FieldProject { .. } => {
            ParameterDispositionBinding::Borrowed
        }
        TransformTarget::Callable(target) => {
            callable_input_disposition_for_target(dag, *target, slot, dispositions)
        }
    }
}

fn callable_input_disposition_for_target(
    dag: &Dag,
    target: DeclarationId,
    slot: usize,
    dispositions: &HashMap<DeclarationId, Vec<ParameterDispositionBinding>>,
) -> ParameterDispositionBinding {
    let (template, _) = callable_template(target, dag);
    let decl = dag.declaration(template);
    match &decl.connective {
        TypeConnective::Arrow { inputs, .. } => dispositions
            .get(&template)
            .and_then(|values| values.get(slot).copied())
            .unwrap_or({
                if slot < inputs.len() {
                    ParameterDispositionBinding::Borrowed
                } else {
                    ParameterDispositionBinding::Consumed
                }
            }),
        TypeConnective::Conj { children } => {
            if slot < children.len() {
                ParameterDispositionBinding::Consumed
            } else {
                ParameterDispositionBinding::Borrowed
            }
        }
        TypeConnective::Disj { variants } => {
            let Some(variant_decl) = variants.first().map(|variant| variant.ty) else {
                return ParameterDispositionBinding::Borrowed;
            };
            match &dag.declaration(variant_decl).connective {
                TypeConnective::Conj { children } if slot < children.len() => {
                    ParameterDispositionBinding::Consumed
                }
                _ => ParameterDispositionBinding::Borrowed,
            }
        }
        _ => ParameterDispositionBinding::Borrowed,
    }
}

pub(crate) type EmitRustMode = EmitMode;

/// Top-level value `Bind` nodes that participate in Rust program-mode emission, in `Dag::nodes`
/// order. **Single selector** for `emit_rust_with_mode` and W1 (`last_emit_rust_program_top_level_value_bind_name`).
fn program_mode_top_level_value_binds<'a>(
    dag: &'a Dag,
    indexes: &RealizationIndexes,
) -> Vec<&'a crate::dag::BindNode> {
    dag.nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .filter(|bind| !indexes.source_filtering.excludes(&bind.span.file))
        .filter(|b| b.params.is_empty())
        .collect()
}

pub(crate) fn emit_rust_with_mode(dag: &Dag, mode: EmitRustMode) -> Result<String, EmitError> {
    let indexes = RealizationIndexes::build(dag)?;
    if !indexes.execution.is_lexically_scoped() {
        return Err(EmitError::UnsupportedBehavior(
            "emit_rust requires rust_execution_model.scope = LexicalScoping".to_string(),
        ));
    }
    let input_use_facts = InputUseFacts::build(dag, &indexes);

    // Resolve the substrate markers we need ONCE up front. Each
    // marker is a typed `DeclarationId` cached at bootstrap end
    // from `src/v3/spec/v3_l1.dag`; if any is missing, the file
    // failed to load and emit can't proceed. Rendering downstream
    // uses the bound handles, never a name string.
    let main_marker = dag
        .main_marker()
        .ok_or(EmitError::MissingSubstrateMarker(SubstrateMarkerRole::Main))?;
    let type_decls: Vec<_> = dag
        .declarations()
        .iter()
        .filter(|decl| !indexes.source_filtering.excludes(&decl.span.file))
        .filter(|decl| decl.name.is_some())
        // `type Result<ok, err> = ...` in `error_primitives.dag` is
        // type-checking authority; Rust
        // materializes it as `::core::result::Result<…>`. Generic `Result` is not
        // emitted as a Rust `enum` (and would collide with the prelude if it were).
        .filter(|decl| !super::substrate_result_type_decl_suppressed_for_emit(dag, decl))
        .filter(|decl| {
            matches!(
                decl.connective,
                TypeConnective::Conj { .. } | TypeConnective::Disj { .. }
            )
        })
        .collect();
    let function_decls: Vec<_> = dag
        .declarations()
        .iter()
        .filter(|decl| !indexes.source_filtering.excludes(&decl.span.file))
        .filter(|decl| decl.name.is_some())
        .filter(|decl| {
            !decl
                .name
                .as_deref()
                .is_some_and(|name| name.starts_with("__anon_lambda_"))
        })
        .filter(|decl| {
            matches!(
                decl.connective,
                TypeConnective::Arrow {
                    body: ArrowBody::UserDefined(_),
                    ..
                }
            )
        })
        .collect();
    let top_level_binds = program_mode_top_level_value_binds(dag, &indexes);

    if mode == EmitRustMode::Program && top_level_binds.is_empty() {
        return Err(EmitError::UnsupportedBehavior(
            "emit_rust requires at least one top-level value Bind".to_string(),
        ));
    }
    if mode == EmitRustMode::Module && !top_level_binds.is_empty() {
        return Err(EmitError::UnsupportedBehavior(
            "emit_rust module mode does not support top-level value Binds".to_string(),
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
    let mut bound_names: HashMap<PortId, LocalBinding> = HashMap::new();
    for bind in &top_level_binds {
        bound_names.insert(bind.value, LocalBinding::Owned(bind.name.clone()));
    }

    let ctx = Ctx {
        dag,
        indexes: &indexes,
        bound_names: &bound_names,
        input_use_facts: &input_use_facts,
        mode,
    };

    let rendered_types: Vec<String> = type_decls
        .iter()
        .map(|decl| ctx.render_type_declaration(decl))
        .collect::<Result<Vec<_>, _>>()?;
    let rendered_functions: Vec<String> = function_decls
        .iter()
        .map(|decl| ctx.render_function_declaration(decl))
        .collect::<Result<Vec<_>, _>>()?;
    let needs_int_div_prelude =
        dag_needs_div_error_prelude(dag, &type_decls, &top_level_binds, &function_decls);
    if let (true, Some(name)) = (
        needs_int_div_prelude,
        div_prelude_reserved_name_collision(
            type_decls.iter(),
            function_decls.iter(),
            top_level_binds.iter(),
            "__v3_int_div",
        ),
    ) {
        return Err(EmitError::UnsupportedBehavior(format!(
            "Rust checked-division prelude would collide with user-defined `{name}`"
        )));
    }

    // `DivError` and other `dsl/std` types are excluded from `type_decls` (see
    // `rust_source_filtering`); the division helper must still compile, so the
    // v3 error enum + `__v3_int_div` are emitted as a small prelude. Names
    // align with `std.error_primitives.DivError` for `Result<…, DivError>` / `rust_int_div`.
    //
    // Dissolution trigger (M1 scaffold): delete when `dsl/std/error_primitives` emits through
    // the normal filtered-type path (no separate string prelude).
    const RUST_V3_INT_OP_PRELUDE: &str = r#"#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DivError { DivideByZero, Overflow }
pub fn __v3_int_div(l: i64, r: i64) -> ::core::result::Result<i64, DivError> {
    if r == 0 {
        return ::core::result::Result::Err(DivError::DivideByZero);
    }
    if l == i64::MIN && r == -1 {
        return ::core::result::Result::Err(DivError::Overflow);
    }
    ::core::result::Result::Ok(l / r)
}
"#;

    let type_defs = join_rendered(&rendered_types, " ");
    let function_defs = join_rendered(&rendered_functions, " ");
    let mut sections: Vec<String> = Vec::new();
    if needs_int_div_prelude {
        sections.push(RUST_V3_INT_OP_PRELUDE.to_string());
    }
    if !type_defs.is_empty() {
        sections.push(type_defs);
    }
    if !function_defs.is_empty() {
        sections.push(function_defs);
    }
    if mode == EmitRustMode::Program {
        let mut rendered_binds: Vec<String> = Vec::with_capacity(top_level_binds.len());
        for bind in &top_level_binds {
            let ty_name = ctx.rust_type_name_for_port(bind.value)?;
            let value_expr = ctx.render_top_level_value(bind.value)?;
            let rendered = render_named_template(
                &indexes.syntax.statements.let_binding,
                &[
                    ("name", &bind.name),
                    ("type", &ty_name),
                    ("value", &value_expr),
                ],
            );
            rendered_binds.push(rendered);
        }

        let body_joined = join_rendered(&rendered_binds, " ");
        let final_bind = top_level_binds.last().expect("guarded above");
        let final_bind_name = final_bind.name.clone();
        let final_display_expr = if ctx.port_is_substrate_result(final_bind.value)? {
            // `std.error_primitives.Result` lowers to Rust's core Result, which
            // does not implement Display. Keep the substrate Main template
            // unchanged and pass it a displayable String.
            //
            // Dissolution trigger (M1 scaffold): delete this Debug wrapper when
            // Result has a target-owned display realization consumed by Main emission.
            format!("format!(\"{{:?}}\", {final_bind_name})")
        } else {
            final_bind_name
        };

        let main_template =
            indexes
                .behaviors
                .get(&main_marker)
                .ok_or(EmitError::MissingBehaviorRealization {
                    marker: main_marker,
                })?;
        let main_program = render_named_template(
            main_template,
            &[
                ("body", &body_joined),
                ("final", &final_display_expr),
                ("quote", &indexes.syntax.literals.string_delimiter),
            ],
        );
        sections.push(main_program);
    }
    Ok(join_rendered(&sections, " "))
}

/// Name of the last top-level **value** `Bind` that `emit_rust` program-mode `main` prints — uses
/// `program_mode_top_level_value_binds` (same vector construction as `emit_rust_with_mode`). W1
/// `rust_emit_output` consults this so the runner cannot drift from emission.
pub(crate) fn last_emit_rust_program_top_level_value_bind_name(
    dag: &Dag,
) -> Result<Option<String>, EmitError> {
    let indexes = RealizationIndexes::build(dag)?;
    Ok(program_mode_top_level_value_binds(dag, &indexes)
        .last()
        .map(|b| b.name.clone()))
}

/// Bundled emission context. Carries the typed indexes, substrate
/// marker handles, and bound-name index through the recursive
/// render walk. Replaces the pre-unwind multi-arg threading where
/// every helper took `dag, index, bound_names, ...` separately.
///
struct Ctx<'a> {
    dag: &'a Dag,
    indexes: &'a RealizationIndexes,
    bound_names: &'a HashMap<PortId, LocalBinding>,
    input_use_facts: &'a InputUseFacts,
    mode: EmitRustMode,
}

#[derive(Debug, Clone)]
enum LocalBinding {
    Owned(String),
    Borrowed(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RenderMode {
    BorrowedRead,
    CopyRead,
    OwnedConstruct,
    OwnedConstructLastUse,
}

#[derive(Debug, Clone, Default)]
struct RenderLocals {
    names: HashMap<PortId, LocalBinding>,
    payload_bindings: HashMap<PortId, VariantPayloadBinding<LocalBinding>>,
}

/// Policy for lowering anonymous `TypeConnective::Arrow` (`fn(..)->_` types)
/// to Rust text. `impl Trait` spellings and `Rc<dyn Fn…>` are only legal in
/// explicit, position-specific emit paths (blocking api-review on #676).
///
/// **Dissolution trigger:** when position-specific Arrow carrier spellings are
/// modeled as `rust.dag` realization rows (or equivalent single-authority data),
/// delete this enum and route both `impl Fn + Clone` and `Rc<dyn Fn…>` through
/// those rows instead of emitter-local policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArrowRustEmitPolicy {
    /// Reserved for fail-closed paths in future context-free seams. Not
    /// selected by any current public entry point — all wired call sites pass
    /// `StorageRcDynFn` (or route to `impl Fn + Clone` for user-fn params).
    // This variant is never *constructed*; the `NoBody` + `match` arm is live if it were.
    #[allow(dead_code)]
    RejectFirstClassFn,
    /// `std::rc::Rc<dyn Fn…>` — struct fields, collection instantiations,
    /// top-level `let` annotations, inferred return slots, and nested
    /// callable types inside `impl Fn` parameter lists.
    StorageRcDynFn,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
enum InputConsumer<'a> {
    Transform(&'a TransformNode),
    Branch(&'a BranchNode),
    Loop(&'a crate::dag::LoopNode),
}

impl<'a> Ctx<'a> {
    fn elide_explicit_borrow(&self, expr: &str) -> String {
        expr.strip_prefix('&').unwrap_or(expr).to_string()
    }

    fn borrowed_list_literal(&self, expr: &str) -> Option<String> {
        if expr == self.indexes.syntax.collection_ops.empty_list {
            return Some("&[]".to_string());
        }
        expr.strip_prefix("vec![")
            .and_then(|tail| tail.strip_suffix(']'))
            .map(|elements| format!("&[{elements}]"))
    }

    fn render_borrowed_expr(&self, port: PortId, expr: String) -> Result<String, EmitError> {
        match self.read_strategy() {
            ReadStrategyBinding::Borrow => {
                if self.port_is_list(port)? {
                    if let Some(slice) = self.borrowed_list_literal(&expr) {
                        return Ok(slice);
                    }
                }
                Ok(format!("&({expr})"))
            }
            ReadStrategyBinding::PassByValue => Ok(expr),
        }
    }

    fn render_collection_receiver(
        &self,
        consumer: InputConsumer<'_>,
        slot: InputSlot,
        locals: &RenderLocals,
    ) -> Result<String, EmitError> {
        let recv = self.render_input_use(consumer, slot, locals)?;
        Ok(self.elide_explicit_borrow(&recv))
    }

    fn read_strategy(&self) -> ReadStrategyBinding {
        self.indexes.rendering.read
    }

    fn construct_strategy(&self) -> ConstructStrategyBinding {
        self.indexes.rendering.construct
    }

    fn callable_param_dispositions(
        &self,
        declaration: DeclarationId,
        input_count: usize,
    ) -> Vec<ParameterDispositionBinding> {
        self.indexes
            .callable_dispositions
            .get(&declaration)
            .cloned()
            .unwrap_or_else(|| vec![ParameterDispositionBinding::Borrowed; input_count])
    }

    fn ownership_edge_rendering_enabled(&self) -> bool {
        self.indexes.computation.is_canonical_dag() && self.indexes.execution.is_ownership_based()
    }

    fn render_binding(
        &self,
        port: PortId,
        binding: &LocalBinding,
        mode: RenderMode,
    ) -> Result<String, EmitError> {
        match mode {
            RenderMode::BorrowedRead => match self.read_strategy() {
                ReadStrategyBinding::Borrow => match binding {
                    LocalBinding::Owned(name) => Ok(format!("&{name}")),
                    LocalBinding::Borrowed(expr) => Ok(expr.clone()),
                },
                ReadStrategyBinding::PassByValue => match binding {
                    LocalBinding::Owned(name) => {
                        if self.port_is_copy(port)? {
                            Ok(name.clone())
                        } else {
                            Ok(format!("({name}).clone()"))
                        }
                    }
                    LocalBinding::Borrowed(expr) => self.construct_from_borrowed_expr(port, expr),
                },
            },
            RenderMode::CopyRead => match binding {
                LocalBinding::Owned(name) => Ok(name.clone()),
                LocalBinding::Borrowed(expr) => Ok(format!("(*({expr}))")),
            },
            RenderMode::OwnedConstruct | RenderMode::OwnedConstructLastUse => match binding {
                LocalBinding::Owned(name) => match self.construct_strategy() {
                    ConstructStrategyBinding::CopyOrClone => {
                        if mode == RenderMode::OwnedConstructLastUse || self.port_is_copy(port)? {
                            Ok(name.clone())
                        } else {
                            Ok(format!("({name}).clone()"))
                        }
                    }
                    ConstructStrategyBinding::PassByValue => {
                        if mode == RenderMode::OwnedConstructLastUse || self.port_is_copy(port)? {
                            Ok(name.clone())
                        } else {
                            Err(EmitError::UnsupportedBehavior(
                                    "rust_rendering.construct = PassByValue is not yet supported for non-Copy owned bindings"
                                        .to_string(),
                                ))
                        }
                    }
                },
                LocalBinding::Borrowed(expr) => self.construct_from_borrowed_expr(port, expr),
            },
        }
    }

    fn construct_from_borrowed_expr(&self, port: PortId, expr: &str) -> Result<String, EmitError> {
        match self.construct_strategy() {
            ConstructStrategyBinding::CopyOrClone => {
                if self.port_is_copy(port)? {
                    Ok(format!("(*({expr}))"))
                } else if self.port_is_list(port)? {
                    Ok(format!("({expr}).to_vec()"))
                } else {
                    Ok(format!("({expr}).clone()"))
                }
            }
            ConstructStrategyBinding::PassByValue => {
                if self.port_is_copy(port)? {
                    Ok(format!("(*({expr}))"))
                } else {
                    Err(EmitError::UnsupportedBehavior(
                        "rust_rendering.construct = PassByValue is not yet supported for borrowed non-Copy values"
                            .to_string(),
                    ))
                }
            }
        }
    }

    fn render_port(
        &self,
        port: PortId,
        locals: &RenderLocals,
        mode: RenderMode,
    ) -> Result<String, EmitError> {
        if let Some(binding) = locals.names.get(&port) {
            return self.render_binding(port, binding, mode);
        }
        if let Some(binding) = locals
            .payload_bindings
            .get(&port)
            .and_then(VariantPayloadBinding::direct)
        {
            return self.render_binding(port, binding, mode);
        }
        if let Some(binding) = self.bound_names.get(&port) {
            return self.render_binding(port, binding, mode);
        }
        self.dispatch_producer(port, locals, mode)
    }

    fn render_input_use(
        &self,
        consumer: InputConsumer<'_>,
        slot: InputSlot,
        locals: &RenderLocals,
    ) -> Result<String, EmitError> {
        let port = self.input_port(consumer, slot)?;
        if !self.ownership_edge_rendering_enabled() {
            return self.render_port(port, locals, RenderMode::BorrowedRead);
        }
        match self.input_disposition(consumer, slot) {
            ParameterDispositionBinding::Borrowed => {
                self.render_port(port, locals, RenderMode::BorrowedRead)
            }
            ParameterDispositionBinding::Consumed => {
                // Conservative: always render as OwnedConstruct (clone) for
                // Consumed disposition. The OwnedConstructLastUse optimization
                // (skip the clone if this is the last use) requires the
                // last-use ordering to match the *rendered* code's evaluation
                // order, but rendered templates can reorder evaluation
                // relative to dag.nodes() iteration order — e.g.,
                // `cons(entry_for(acc, _), acc)` renders as
                // `let mut __list = acc; __list.insert(0, entry_for(&acc, _))`,
                // which moves `acc` (slot 1) before borrowing it (slot 0).
                // Until is_last_use accounts for template-induced reordering,
                // skipping the clone is unsound. The is_last_use facts are
                // still computed and tracked (B3) so the lookup is correct;
                // we just don't act on it here yet.
                let _ = self.is_last_use(port, consumer, slot);
                self.render_port(port, locals, RenderMode::OwnedConstruct)
            }
        }
    }

    fn render_copy_input_use(
        &self,
        consumer: InputConsumer<'_>,
        slot: InputSlot,
        locals: &RenderLocals,
    ) -> Result<String, EmitError> {
        let port = self.input_port(consumer, slot)?;
        self.render_port(port, locals, RenderMode::CopyRead)
    }

    fn input_port(
        &self,
        consumer: InputConsumer<'_>,
        slot: InputSlot,
    ) -> Result<PortId, EmitError> {
        match (consumer, slot) {
            (InputConsumer::Transform(transform), InputSlot::Positional(slot)) => {
                transform.inputs.get(slot).copied().ok_or_else(|| {
                    EmitError::UnsupportedBehavior(format!(
                        "transform input slot {slot} is out of bounds"
                    ))
                })
            }
            (InputConsumer::Branch(branch), InputSlot::BranchInput) => Ok(branch.input),
            (InputConsumer::Loop(loop_node), InputSlot::LoopSource) => Ok(loop_node.source),
            (InputConsumer::Loop(loop_node), InputSlot::LoopInit) => Ok(loop_node.init),
            (InputConsumer::Loop(loop_node), InputSlot::LoopBoundCount) => {
                loop_node.bound.count_port().ok_or_else(|| {
                    EmitError::UnsupportedBehavior(
                        "loop does not carry a cardinality-bound input".to_string(),
                    )
                })
            }
            _ => Err(EmitError::UnsupportedBehavior(
                "input-slot kind does not match the selected consumer".to_string(),
            )),
        }
    }

    fn input_disposition(
        &self,
        consumer: InputConsumer<'_>,
        slot: InputSlot,
    ) -> ParameterDispositionBinding {
        match consumer {
            InputConsumer::Transform(transform) => match slot {
                InputSlot::Positional(index) => transform_input_disposition(
                    self.dag,
                    transform,
                    index,
                    &self.indexes.callable_dispositions,
                ),
                _ => ParameterDispositionBinding::Borrowed,
            },
            InputConsumer::Branch(_) | InputConsumer::Loop(_) => {
                ParameterDispositionBinding::Borrowed
            }
        }
    }

    /// True iff the edge identified by `(consumer, slot)` is the
    /// last edge that touches `port` in the total ordering — meaning
    /// no later borrow or consume references this port. Only when
    /// this holds is it safe to render a Consumed edge as a move
    /// (`OwnedConstructLastUse`); otherwise we must clone.
    fn is_last_use(&self, port: PortId, consumer: InputConsumer<'_>, slot: InputSlot) -> bool {
        let key = match consumer {
            InputConsumer::Transform(transform) => InputUseKey {
                consumer: transform.id,
                slot,
            },
            InputConsumer::Branch(branch) => InputUseKey {
                consumer: branch.id,
                slot,
            },
            InputConsumer::Loop(loop_node) => InputUseKey {
                consumer: loop_node.id,
                slot,
            },
        };
        self.input_use_facts
            .edge_order
            .get(&key)
            .zip(self.input_use_facts.last_use_order_by_port.get(&port))
            .is_some_and(|(edge, last)| edge == last)
    }

    /// Render the value for a top-level let binding. Bypasses
    /// `bound_names` for `port` itself (otherwise every let would
    /// render as `let x: i64 = x;`); recursive sub-walks still use
    /// `render_port` and DO consult `bound_names`.
    fn render_top_level_value(&self, port: PortId) -> Result<String, EmitError> {
        self.dispatch_producer(port, &RenderLocals::default(), RenderMode::OwnedConstruct)
    }

    fn dispatch_producer(
        &self,
        port: PortId,
        locals: &RenderLocals,
        mode: RenderMode,
    ) -> Result<String, EmitError> {
        let Some(node_id) = self.dag.port(port).produced_by else {
            return Err(EmitError::UnsupportedBehavior(
                "render reached a port with no producer (parameter?)".to_string(),
            ));
        };
        match self.dag.node(node_id) {
            Behavior::Value(v) => match mode {
                RenderMode::BorrowedRead => {
                    self.render_borrowed_expr(port, render_value(v, &self.indexes.syntax.literals))
                }
                RenderMode::CopyRead
                | RenderMode::OwnedConstruct
                | RenderMode::OwnedConstructLastUse => {
                    Ok(render_value(v, &self.indexes.syntax.literals))
                }
            },
            Behavior::Transform(t) => self.render_transform(t, locals, mode),
            Behavior::Branch(b) => {
                let expr = self.render_branch(b, locals)?;
                match mode {
                    RenderMode::BorrowedRead => self.render_borrowed_expr(port, expr),
                    RenderMode::CopyRead
                    | RenderMode::OwnedConstruct
                    | RenderMode::OwnedConstructLastUse => Ok(expr),
                }
            }
            Behavior::Loop(l) => {
                let expr = self.render_loop(l, locals)?;
                match mode {
                    RenderMode::BorrowedRead => self.render_borrowed_expr(port, expr),
                    RenderMode::CopyRead
                    | RenderMode::OwnedConstruct
                    | RenderMode::OwnedConstructLastUse => Ok(expr),
                }
            }
            Behavior::Bind(b) => {
                self.render_binding(port, &LocalBinding::Owned(b.name.clone()), mode)
            }
        }
    }

    fn render_transform(
        &self,
        t: &TransformNode,
        locals: &RenderLocals,
        mode: RenderMode,
    ) -> Result<String, EmitError> {
        match &t.target {
            TransformTarget::Operator(op) => {
                let expr = self.render_operator(t, *op, locals)?;
                self.adjust_owned_expr(t.output, expr, mode)
            }
            TransformTarget::FieldProject {
                field_label,
                field_child,
            } => self.render_field_project(t, field_label, locals, *field_child, mode),
            TransformTarget::Callable(target) => {
                let expr = self.render_callable_transform(t, *target, locals)?;
                self.adjust_owned_expr(t.output, expr, mode)
            }
        }
    }

    fn adjust_owned_expr(
        &self,
        port: PortId,
        expr: String,
        mode: RenderMode,
    ) -> Result<String, EmitError> {
        match mode {
            RenderMode::BorrowedRead => self.render_borrowed_expr(port, expr),
            RenderMode::CopyRead => {
                if self.port_is_copy(port)? {
                    Ok(expr)
                } else {
                    Ok(format!("({expr}).clone()"))
                }
            }
            RenderMode::OwnedConstruct | RenderMode::OwnedConstructLastUse => Ok(expr),
        }
    }

    fn render_field_project(
        &self,
        t: &TransformNode,
        field_label: &str,
        locals: &RenderLocals,
        field_child: Option<DeclarationId>,
        mode: RenderMode,
    ) -> Result<String, EmitError> {
        if field_child.is_none() {
            return Err(EmitError::UnsupportedBehavior(format!(
                "field projection .{field_label} is missing its resolved field child carrier; emit_rust expects post-infer FieldProject targets"
            )));
        }
        if t.inputs.len() != 1 {
            return Err(EmitError::UnsupportedBehavior(format!(
                "field projection .{field_label} arity {} is not supported; expected exactly one parent input",
                t.inputs.len()
            )));
        }
        if let Some(binding) = locals
            .payload_bindings
            .get(&t.inputs[0])
            .and_then(|binding| binding.field(field_label))
        {
            return self.render_binding(t.output, binding, mode);
        }
        let parent_expr = self.render_input_use(
            InputConsumer::Transform(t),
            InputSlot::Positional(0),
            locals,
        )?;
        let parent_access = self.elide_explicit_borrow(&parent_expr);
        let parent_type_id = primitive_type_id_for_port(self.dag, t.inputs[0])?;
        if let Some(type_binding) = self.indexes.types.get(&parent_type_id) {
            let binding = type_binding
                .fields
                .get(field_label)
                .ok_or_else(|| {
                    EmitError::UnsupportedBehavior(format!(
                        "field projection .{field_label} has no FieldBinding entry on the parent TypeRealization"
                    ))
                })?;
            let access_expr = match &binding.access {
                RustFieldAccessBinding::DirectField(name) => render_named_template(
                    &self.indexes.syntax.expressions.field_access,
                    &[("object", &parent_access), ("field", name)],
                ),
                RustFieldAccessBinding::AccessorMethod(name) => format!(
                    "{}()",
                    render_named_template(
                        &self.indexes.syntax.expressions.field_access,
                        &[("object", &parent_access), ("field", name)],
                    )
                ),
            };
            return match mode {
                RenderMode::BorrowedRead => match self.read_strategy() {
                    ReadStrategyBinding::Borrow => {
                        if binding.borrowed_read {
                            Ok(access_expr)
                        } else {
                            self.render_borrowed_expr(t.output, access_expr)
                        }
                    }
                    ReadStrategyBinding::PassByValue => {
                        if binding.borrowed_read {
                            self.construct_from_borrowed_expr(t.output, &access_expr)
                        } else {
                            Ok(access_expr)
                        }
                    }
                },
                RenderMode::CopyRead => Ok(access_expr),
                RenderMode::OwnedConstruct | RenderMode::OwnedConstructLastUse => {
                    if binding.borrowed_read {
                        self.construct_from_borrowed_expr(t.output, &access_expr)
                    } else if self.port_is_copy(t.output)? {
                        Ok(access_expr)
                    } else if self.port_is_list(t.output)? {
                        Ok(format!("({access_expr}).to_vec()"))
                    } else {
                        Ok(format!("({access_expr}).clone()"))
                    }
                }
            };
        }
        let Some(conj_id) = walk_to_conj(self.dag, parent_type_id) else {
            return Err(EmitError::MissingTypeRealization {
                target: parent_type_id,
            });
        };
        if !matches!(
            self.dag.declaration(conj_id).connective,
            TypeConnective::Conj { .. }
        ) {
            return Err(EmitError::MissingTypeRealization {
                target: parent_type_id,
            });
        }
        let access_expr = render_named_template(
            &self.indexes.syntax.expressions.field_access,
            &[("object", &parent_access), ("field", field_label)],
        );
        match mode {
            RenderMode::BorrowedRead => self.render_borrowed_expr(t.output, access_expr),
            RenderMode::CopyRead => Ok(access_expr),
            RenderMode::OwnedConstruct | RenderMode::OwnedConstructLastUse => {
                if self.port_is_copy(t.output)? {
                    Ok(access_expr)
                } else {
                    Ok(format!("({access_expr}).clone()"))
                }
            }
        }
    }

    fn render_operator(
        &self,
        t: &TransformNode,
        op: OperatorKind,
        locals: &RenderLocals,
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
        let carrier = super::operator_carrier_realization(
            &self.indexes.operators,
            self.dag,
            operand_type_id,
            op_decl_id,
        )
        .ok_or(EmitError::MissingOperatorRealization {
            target: operand_type_id,
            op: op_decl_id,
        })?
        .clone();
        let lhs = self.render_copy_input_use(
            InputConsumer::Transform(t),
            InputSlot::Positional(0),
            locals,
        )?;
        let rhs = self.render_copy_input_use(
            InputConsumer::Transform(t),
            InputSlot::Positional(1),
            locals,
        )?;
        Ok(render_named_template(
            &carrier,
            &[("lhs", &lhs), ("rhs", &rhs)],
        ))
    }

    fn render_branch(&self, b: &BranchNode, locals: &RenderLocals) -> Result<String, EmitError> {
        if self.branch_scrutinee_is_bool(b)? {
            let (then_path, else_path) = self.split_bool_paths(b)?;
            let cond = self.render_copy_input_use(
                InputConsumer::Branch(b),
                InputSlot::BranchInput,
                locals,
            )?;
            let then_expr = self.render_path_body(then_path, locals)?;
            let else_expr = self.render_path_body(else_path, locals)?;
            return Ok(render_named_template(
                &self.indexes.syntax.control_flow.if_else,
                &[("cond", &cond), ("then", &then_expr), ("else", &else_expr)],
            ));
        }
        if let Some(rendered) = self.render_realized_pattern_branch(b, locals)? {
            return Ok(rendered);
        }

        let expr =
            self.render_input_use(InputConsumer::Branch(b), InputSlot::BranchInput, locals)?;
        let arms = b
            .paths
            .iter()
            .map(|path| {
                let pattern = self.render_branch_pattern(b, path)?;
                let body = self.render_path_body(path, locals)?;
                Ok(render_named_template(
                    &self.indexes.syntax.patterns.match_arm,
                    &[("pattern", &pattern), ("body", &body)],
                ))
            })
            .collect::<Result<Vec<_>, EmitError>>()?;
        let arms_joined = join_rendered(&arms, " ");
        Ok(render_named_template(
            &self.indexes.syntax.patterns.match_expr,
            &[("expr", &expr), ("arms", &arms_joined)],
        ))
    }

    fn render_realized_pattern_branch(
        &self,
        branch: &BranchNode,
        locals: &RenderLocals,
    ) -> Result<Option<String>, EmitError> {
        let scrutinee_type_id = primitive_type_id_for_port(self.dag, branch.input)?;
        let Some(disj_id) = walk_to_disj(self.dag, scrutinee_type_id) else {
            return Ok(None);
        };
        let Some(binding) = self.indexes.patterns.get(&disj_id) else {
            return Ok(None);
        };
        self.render_vector_list_pattern_branch(branch, disj_id, binding, locals)
            .map(Some)
    }

    fn render_vector_list_pattern_branch(
        &self,
        branch: &BranchNode,
        _disj_id: DeclarationId,
        binding: &PatternRealizationBinding,
        locals: &RenderLocals,
    ) -> Result<String, EmitError> {
        // Role identity is a spec fact: `PatternRealization.empty_variant` /
        // `cons_variant` are typed `DeclarationRef`s to the sum's variants
        // (`List.Empty` / `List.Cons` in `rust.dag`). The emitter reads them
        // — no label-matching on variant strings.
        let empty_path = find_resolved_branch_path(branch, binding.empty_variant).ok_or_else(
            || {
                EmitError::UnsupportedBehavior(
                    "vector-list pattern realization requires a branch arm for the declared empty_variant"
                        .to_string(),
                )
            },
        )?;
        let cons_path = find_resolved_branch_path(branch, binding.cons_variant).ok_or_else(
            || {
                EmitError::UnsupportedBehavior(
                    "vector-list pattern realization requires a branch arm for the declared cons_variant"
                        .to_string(),
                )
            },
        )?;
        let scrutinee = self.render_input_use(
            InputConsumer::Branch(branch),
            InputSlot::BranchInput,
            locals,
        )?;
        let realized_scrutinee = render_named_template(&binding.scrutinee, &[("expr", &scrutinee)]);
        let empty_body = self.render_path_body(empty_path, locals)?;

        let head_name = "__list_head";
        let tail_name = "__list_tail";
        let cons_pattern = render_named_template(
            &binding.cons_pattern,
            &[("head", head_name), ("tail", tail_name)],
        );
        let mut cons_locals = locals.clone();
        if let Some(payload) = &cons_path.binding {
            let mut fields = HashMap::new();
            fields.insert(
                "head".to_string(),
                LocalBinding::Borrowed(render_named_template(
                    &binding.head_expr,
                    &[("head", head_name)],
                )),
            );
            fields.insert(
                "tail".to_string(),
                LocalBinding::Borrowed(render_named_template(
                    &binding.tail_expr,
                    &[("tail", tail_name)],
                )),
            );
            cons_locals
                .payload_bindings
                .insert(payload.payload_port, VariantPayloadBinding::Fields(fields));
        }
        let cons_body = self.render_port(
            cons_path.output,
            &cons_locals,
            RenderMode::OwnedConstructLastUse,
        )?;

        let arms = vec![
            render_named_template(
                &self.indexes.syntax.patterns.match_arm,
                &[("pattern", &binding.empty_pattern), ("body", &empty_body)],
            ),
            render_named_template(
                &self.indexes.syntax.patterns.match_arm,
                &[("pattern", &cons_pattern), ("body", &cons_body)],
            ),
        ];
        Ok(render_named_template(
            &self.indexes.syntax.patterns.match_expr,
            &[
                ("expr", &realized_scrutinee),
                ("arms", &join_rendered(&arms, " ")),
            ],
        ))
    }

    fn render_path_body(&self, path: &Path, locals: &RenderLocals) -> Result<String, EmitError> {
        let mut arm_locals = locals.clone();
        if let Some(binding) = &path.binding {
            if let Some(payload_binding) = self.render_variant_payload_binding(path, binding)? {
                arm_locals
                    .payload_bindings
                    .insert(binding.payload_port, payload_binding);
            }
        }
        self.render_port(path.output, &arm_locals, RenderMode::OwnedConstructLastUse)
    }

    fn render_branch_pattern(&self, branch: &BranchNode, path: &Path) -> Result<String, EmitError> {
        let scrutinee_type_id = primitive_type_id_for_port(self.dag, branch.input)?;
        let disj_id = walk_to_disj(self.dag, scrutinee_type_id).ok_or_else(|| {
            EmitError::UnsupportedBehavior(format!(
                "branch scrutinee type at {scrutinee_type_id:?} does not walk to a Disj"
            ))
        })?;
        let resolved_id = match &path.pattern {
            BranchPattern::ResolvedVariant(id) => *id,
            BranchPattern::UnresolvedVariant { name, .. } => {
                return Err(EmitError::UnresolvedBranchPattern {
                    variant_name: name.clone(),
                });
            }
        };

        if self.branch_scrutinee_is_bool(branch)? {
            return self.render_bool_pattern(disj_id, resolved_id);
        }

        let variant_name = variant_name_for_decl(self.dag, disj_id, resolved_id)?;
        let is_optional_match = is_optional_match_disj(self.dag, disj_id);
        let qualified_name = if is_optional_match {
            variant_name.clone()
        } else {
            let enum_name = named_disj_enum_name_for_rust_match_emit(self.dag, disj_id)
                .ok_or_else(|| {
                    EmitError::UnsupportedBehavior(
                        "match on anonymous sum declarations is not yet supported in Rust emission"
                            .to_string(),
                    )
                })?;
            self.qualified_name(&enum_name, &variant_name)
        };
        let Some(binding) = &path.binding else {
            return Ok(render_named_template(
                &self.indexes.syntax.patterns.variant_pattern_empty,
                &[("name", &qualified_name)],
            ));
        };
        let payload_shape = match variant_payload_shape(self.dag, &resolved_id) {
            VariantPayloadShapeLookup::DeclarationMissing => {
                return Err(EmitError::UnsupportedBehavior(format!(
                    "matched variant `{variant_name}` references an absent declaration"
                )));
            }
            VariantPayloadShapeLookup::NotPayloadProduct => {
                return Err(EmitError::UnsupportedBehavior(format!(
                    "matched variant `{variant_name}` does not lower to a payload product"
                )));
            }
            VariantPayloadShapeLookup::Found { _0: shape } => shape,
        };
        let rendered_binding = self.render_payload_binding_name(path, binding);
        if let Some(rendered) = self.render_multi_field_variant_pattern(
            &qualified_name,
            binding,
            &rendered_binding,
            &payload_shape,
        )? {
            return Ok(rendered);
        }
        if let Some(rendered) = self.render_single_field_variant_pattern(
            disj_id,
            is_optional_match,
            &qualified_name,
            &rendered_binding,
            &payload_shape,
        ) {
            return Ok(rendered);
        }
        match payload_shape {
            VariantPayloadShape::Empty => Ok(render_named_template(
                &self.indexes.syntax.patterns.variant_pattern_empty,
                &[("name", &qualified_name)],
            )),
            VariantPayloadShape::PositionalSingle | VariantPayloadShape::NamedFields { .. } => {
                unreachable!("single-field patterns return above")
            }
        }
    }

    fn render_multi_field_variant_pattern(
        &self,
        qualified_name: &str,
        binding: &crate::dag::PayloadBinding,
        rendered_binding: &str,
        payload_shape: &VariantPayloadShape,
    ) -> Result<Option<String>, EmitError> {
        // Substrate still models `Cardinality` as `{ element, bound }`, but the
        // hand-authored Rust enum is `Cardinality(CardinalityPayload)` (tuple).
        if qualified_name == "TypeConnective::Cardinality" {
            return Ok(Some(render_named_template(
                &self.indexes.syntax.patterns.variant_pattern_positional,
                &[("name", qualified_name), ("binding", rendered_binding)],
            )));
        }
        if !matches!(
            payload_shape,
            VariantPayloadShape::NamedFields { _0: ref fields } if fields.len() > 1
        ) {
            return Ok(None);
        }
        let VariantPayloadShape::NamedFields { _0: field_labels } = payload_shape else {
            unreachable!("guarded above")
        };
        let wildcard = self.indexes.syntax.patterns.wildcard.clone();
        let payload_unused = rendered_binding == wildcard;
        let field_bindings = field_labels
            .iter()
            .map(|child| {
                let binding_text = if payload_unused {
                    wildcard.clone()
                } else {
                    destructured_field_alias(&binding.binding_name, child)
                };
                Ok(render_named_template(
                    &self.indexes.syntax.patterns.field_binding,
                    &[("field", child), ("binding", &binding_text)],
                ))
            })
            .collect::<Result<Vec<_>, EmitError>>()?;
        Ok(Some(render_named_template(
            &self.indexes.syntax.patterns.variant_pattern,
            &[
                ("name", qualified_name),
                (
                    "bindings",
                    &join_rendered(
                        &field_bindings,
                        &self.indexes.syntax.patterns.field_binding_separator,
                    ),
                ),
            ],
        )))
    }

    fn render_single_field_variant_pattern(
        &self,
        disj_id: DeclarationId,
        is_optional_match: bool,
        qualified_name: &str,
        rendered_binding: &str,
        payload_shape: &VariantPayloadShape,
    ) -> Option<String> {
        let single_field_label = match payload_shape {
            VariantPayloadShape::PositionalSingle => None,
            VariantPayloadShape::NamedFields { _0: field_labels } if field_labels.len() == 1 => {
                Some(field_labels[0].as_str())
            }
            VariantPayloadShape::NamedFields { .. } | VariantPayloadShape::Empty => return None,
        };
        if single_field_label.is_none()
            && (self.indexes.types.contains_key(&disj_id) || is_optional_match)
        {
            return Some(render_named_template(
                &self.indexes.syntax.patterns.variant_pattern_positional,
                &[("name", qualified_name), ("binding", rendered_binding)],
            ));
        }
        let field_name = single_field_label.unwrap_or("_0");
        // `v3.std.lookup::Lookup::Hit` is a Rust tuple variant (`Hit(T)` in
        // `dag_lookup_generated`); other single-`_0` sum arms stay struct-style.
        // Dissolution trigger: today `Conj` children for a sum arm do not record
        // "tuple vs struct payload" for Rust; emit would otherwise use struct
        // patterns for every `_0` field. When positionality is a fact on the
        // `Declaration` / `Disj` surface (or spec-driven), drop this name-match
        // and read it structurally. Paired with `render_variant_constructor`'s
        // `Lookup`/`Hit` constructor branch.
        let is_lookup_hit = field_name == "_0"
            && matches!(
                qualified_name.split("::").collect::<Vec<_>>().as_slice(),
                [a, b] if *a == "Lookup" && *b == "Hit"
            );
        if is_lookup_hit {
            return Some(render_named_template(
                &self.indexes.syntax.patterns.variant_pattern_positional,
                &[("name", qualified_name), ("binding", rendered_binding)],
            ));
        }
        let bindings = render_named_template(
            &self.indexes.syntax.patterns.field_binding,
            &[("field", field_name), ("binding", rendered_binding)],
        );
        Some(render_named_template(
            &self.indexes.syntax.patterns.variant_pattern,
            &[("name", qualified_name), ("bindings", &bindings)],
        ))
    }

    /// E-5 / Lane 1 Stage 1c: render the arm's payload binding name
    /// per `rust_clean_emission.pattern_bindings`. When the rule is
    /// `EmitUnderscoreWhenUnused` and the arm body does not consume
    /// `binding.payload_port`, emit `_` so `rustc -D warnings` does
    /// not fire `unused_variables`. Rust-invalid contract variants
    /// are rejected while building `CleanEmissionContractBinding`,
    /// so the renderer only sees Rust-valid states.
    fn render_payload_binding_name(
        &self,
        path: &Path,
        binding: &crate::dag::PayloadBinding,
    ) -> String {
        match self.indexes.clean_emission.pattern_bindings {
            PatternBindingRuleBinding::EmitUnderscoreWhenUnused
                if !self.port_is_consumed_from(path.output, binding.payload_port) =>
            {
                self.indexes.syntax.patterns.wildcard.clone()
            }
            PatternBindingRuleBinding::EmitBindingAlways
            | PatternBindingRuleBinding::EmitUnderscoreWhenUnused => binding.binding_name.clone(),
        }
    }

    fn render_variant_payload_binding(
        &self,
        path: &Path,
        binding: &crate::dag::PayloadBinding,
    ) -> Result<Option<VariantPayloadBinding<LocalBinding>>, EmitError> {
        let BranchPattern::ResolvedVariant(variant_id) = &path.pattern else {
            return Ok(None);
        };
        let shape = match variant_payload_shape(self.dag, variant_id) {
            VariantPayloadShapeLookup::DeclarationMissing => {
                return Err(EmitError::UnsupportedBehavior(
                    "variant payload binding references an absent declaration".to_string(),
                ));
            }
            VariantPayloadShapeLookup::NotPayloadProduct => return Ok(None),
            VariantPayloadShapeLookup::Found { _0: shape } => shape,
        };
        let payload_binding_name = self.render_payload_binding_name(path, binding);
        let wildcard = self.indexes.syntax.patterns.wildcard.clone();
        Ok(match shape {
            VariantPayloadShape::Empty => None,
            VariantPayloadShape::PositionalSingle => Some(VariantPayloadBinding::Direct(
                LocalBinding::Borrowed(payload_binding_name),
            )),
            VariantPayloadShape::NamedFields { _0: field_labels } => {
                match self.indexes.clean_emission.variant_payload_field_access {
                    VariantPayloadFieldAccessRuleBinding::AccessFromPayloadBinding => Some(
                        VariantPayloadBinding::Direct(LocalBinding::Borrowed(payload_binding_name)),
                    ),
                    VariantPayloadFieldAccessRuleBinding::OverrideNamedFieldsAtBindingSite => {
                        let multiple_fields = field_labels.len() > 1;
                        let fields = field_labels
                            .into_iter()
                            .map(|field_label| {
                                let local_name = if payload_binding_name == wildcard {
                                    wildcard.clone()
                                } else if multiple_fields {
                                    destructured_field_alias(&binding.binding_name, &field_label)
                                } else {
                                    payload_binding_name.clone()
                                };
                                (field_label, LocalBinding::Borrowed(local_name))
                            })
                            .collect();
                        Some(VariantPayloadBinding::Fields(fields))
                    }
                }
            }
        })
    }

    /// Thin wrapper; rationale lives on `crate::emit::port_is_consumed_from`.
    fn port_is_consumed_from(&self, root: PortId, target: PortId) -> bool {
        super::port_is_consumed_from(self.dag, root, target)
    }

    fn render_bool_pattern(
        &self,
        disj_id: DeclarationId,
        variant_id: DeclarationId,
    ) -> Result<String, EmitError> {
        let TypeConnective::Disj { variants } = &self.dag.declaration(disj_id).connective else {
            unreachable!("walk_to_disj returned non-Disj")
        };
        let Some((idx, _)) = variants
            .iter()
            .enumerate()
            .find(|(_, variant)| variant.ty == variant_id)
        else {
            return Err(EmitError::UnsupportedBehavior(format!(
                "bool branch variant {variant_id:?} was not found on its parent disjunction"
            )));
        };
        match idx {
            0 => Ok(self.indexes.syntax.literals.true_keyword.clone()),
            1 => Ok(self.indexes.syntax.literals.false_keyword.clone()),
            _ => Err(EmitError::NonBooleanBranch {
                variant_ids: variants.iter().map(|variant| variant.ty).collect(),
            }),
        }
    }

    fn branch_scrutinee_is_bool(&self, branch: &BranchNode) -> Result<bool, EmitError> {
        let scrutinee_type_id = primitive_type_id_for_port(self.dag, branch.input)?;
        let Some(bool_shape) = self.dag.bool_shape() else {
            return Err(EmitError::MissingTypeRealization {
                target: scrutinee_type_id,
            });
        };
        let Some(scrutinee_disj) = walk_to_disj(self.dag, scrutinee_type_id) else {
            return Ok(false);
        };
        let Some(bool_disj) = walk_to_disj(self.dag, bool_shape.declaration) else {
            return Ok(false);
        };
        Ok(scrutinee_disj == bool_disj)
    }

    /// v3.std.lookup / v3.std.algebra: `miss_*_lookup` / `hit_*_lookup`
    /// are thin monomorphized `Lookup<T>` constructors (one pair per
    /// element type — `Int`, `SymbolicCost`, …). Emit as `Lookup::Miss`
    /// / `Lookup::Hit(...)` so generated lens code does not call
    /// out-of-scope shims. Runs before [`Self::render_realized_callable`]
    /// so a registered callable strategy does not pre-empt enum lowering.
    fn lookup_monomorphized_constructor_emit(
        &self,
        t: &TransformNode,
        template: DeclarationId,
        locals: &RenderLocals,
    ) -> Result<Option<String>, EmitError> {
        let Some(name) = self.dag.declaration(template).name.as_deref() else {
            return Ok(None);
        };
        let is_miss = name == "miss_int_lookup"
            || name == "miss_symbolic_cost_lookup"
            || name == "miss_declaration_id_lookup";
        let is_hit = name == "hit_int_lookup"
            || name == "hit_symbolic_cost_lookup"
            || name == "hit_declaration_id_lookup";
        if is_miss {
            if !t.inputs.is_empty() {
                return Err(EmitError::UnsupportedBehavior(format!(
                    "{name}() expects zero arguments"
                )));
            }
            return Ok(Some("Lookup::Miss".to_string()));
        }
        if is_hit {
            if t.inputs.len() != 1 {
                return Err(EmitError::UnsupportedBehavior(format!(
                    "{name}(v) expected one argument, got {}",
                    t.inputs.len()
                )));
            }
            let arg = self.elide_explicit_borrow(&self.render_input_use(
                InputConsumer::Transform(t),
                InputSlot::Positional(0),
                locals,
            )?);
            let out = if arg.starts_with('(') {
                format!("Lookup::Hit{arg}")
            } else {
                format!("Lookup::Hit({arg})")
            };
            return Ok(Some(out));
        }
        Ok(None)
    }

    fn render_callable_transform(
        &self,
        t: &TransformNode,
        target: DeclarationId,
        locals: &RenderLocals,
    ) -> Result<String, EmitError> {
        let (template, arguments) = callable_template(target, self.dag);
        if let Some(rendered) = self.lookup_monomorphized_constructor_emit(t, template, locals)? {
            return Ok(rendered);
        }
        if let Some(rendered) = self.render_substrate_accessor(t, template, locals)? {
            return Ok(rendered);
        }
        if let Some(strategy) = self.indexes.callables.get(&template) {
            return self.render_realized_callable(t, template, *strategy, &arguments, locals);
        }
        self.render_general_callable(t, template, locals)
    }

    /// DB-14 substrate-accessor dispatch. If the callable's target
    /// decl is in `substrate_accessors` (a binding for the active
    /// Rust target exists), render the realization's `carrier`
    /// template via positional `{p0}`, `{p1}` substitution with the
    /// Transform's input expressions. Otherwise return `None` and
    /// let `render_callable_transform` fall through to the standard
    /// dispatch.
    ///
    /// Design: the accessor's Arrow body stays `Unparsed` at bootstrap (the
    /// `{ host <name> }` stub) because the accessor → realization mapping is
    /// TARGET-specific. The per-target resolution happens here, at emission
    /// time, against this emitter's `substrate_accessors` index — which was
    /// built from `SubstrateAccessorBinding` records filtered by
    /// `language == rust_language`. See `build_substrate_accessor_index`.
    fn render_substrate_accessor(
        &self,
        t: &TransformNode,
        template: DeclarationId,
        locals: &RenderLocals,
    ) -> Result<Option<String>, EmitError> {
        let Some(realization_id) = self.indexes.substrate_accessors.get(&template).copied() else {
            // If the template IS a declared substrate accessor (in
            // the universe of all `SubstrateAccessorBinding`
            // records across languages) but has no binding for the
            // active target, fail closed rather than fall through
            // to generic callable rendering — that would emit
            // `func(args)` for a function the target doesn't
            // provide. Post review round 1b.4: "no binding for this
            // target" on a declared accessor is not a benign miss.
            if self.indexes.substrate_accessor_universe.contains(&template) {
                let accessor_name = self
                    .dag
                    .declaration(template)
                    .name
                    .clone()
                    .unwrap_or_else(|| format!("declaration#{}", template.raw()));
                return Err(EmitError::UnsupportedBehavior(format!(
                    "substrate accessor `{accessor_name}` has no SubstrateAccessorBinding for the active Rust target (`rust_language`); add `data <name>_binding_rust: SubstrateAccessorBinding = {{ accessor: {accessor_name}, realization: <target-realization>, language: rust_language }}` in `src/v3/spec/rust.dag`"
                )));
            }
            return Ok(None);
        };
        let realization = self.dag.declaration(realization_id);
        let Some(ValueBody::Structural { fields }) = &realization.value_body else {
            return Err(EmitError::UnsupportedBehavior(format!(
                "substrate accessor realization for `{}` lacks a structural value body",
                self.dag
                    .declaration(template)
                    .name
                    .as_deref()
                    .unwrap_or("<anonymous>")
            )));
        };
        let carrier = fields
            .iter()
            .find_map(|(label, value)| match (label.as_str(), value) {
                ("carrier", FieldValue::Literal(LiteralBits::String(s))) => Some(s.clone()),
                _ => None,
            })
            .ok_or_else(|| {
                EmitError::UnsupportedBehavior(format!(
                    "substrate accessor realization `{}` is missing required String field `carrier`",
                    realization.name.as_deref().unwrap_or("<anonymous>")
                ))
            })?;
        let rendered_inputs = t
            .inputs
            .iter()
            .enumerate()
            .map(|(slot, _)| {
                self.render_input_use(
                    InputConsumer::Transform(t),
                    InputSlot::Positional(slot),
                    locals,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let placeholders: Vec<String> = (0..rendered_inputs.len())
            .map(|i| format!("p{i}"))
            .collect();
        let bindings: Vec<(&str, &str)> = placeholders
            .iter()
            .zip(rendered_inputs.iter())
            .map(|(p, expr)| (p.as_str(), expr.as_str()))
            .collect();
        Ok(Some(render_named_template(&carrier, &bindings)))
    }

    fn render_realized_callable(
        &self,
        consumer: &TransformNode,
        template: DeclarationId,
        strategy: RustCallableStrategyBinding,
        arguments: &[TemplateArgument],
        locals: &RenderLocals,
    ) -> Result<String, EmitError> {
        match strategy {
            RustCallableStrategyBinding::ListEmpty => {
                Ok(self.indexes.syntax.collection_ops.empty_list.clone())
            }
            RustCallableStrategyBinding::ListSingleton => {
                if consumer.inputs.len() != 1 {
                    return Err(EmitError::UnsupportedBehavior(format!(
                        "singleton arity {} is not supported; expected one runtime input",
                        consumer.inputs.len()
                    )));
                }
                let value = self.render_input_use(
                    InputConsumer::Transform(consumer),
                    InputSlot::Positional(0),
                    locals,
                )?;
                Ok(render_named_template(
                    &self.indexes.syntax.collection_ops.list_literal,
                    &[("elements", &value)],
                ))
            }
            RustCallableStrategyBinding::ListCons => {
                if consumer.inputs.len() != 2 {
                    return Err(EmitError::UnsupportedBehavior(format!(
                        "cons arity {} is not supported; expected two runtime inputs",
                        consumer.inputs.len()
                    )));
                }
                let head = self.render_input_use(
                    InputConsumer::Transform(consumer),
                    InputSlot::Positional(0),
                    locals,
                )?;
                let tail = self.render_input_use(
                    InputConsumer::Transform(consumer),
                    InputSlot::Positional(1),
                    locals,
                )?;
                Ok(render_named_template(
                    &self.indexes.syntax.collection_ops.cons,
                    &[("head", &head), ("tail", &tail)],
                ))
            }
            RustCallableStrategyBinding::ListConcat => {
                if consumer.inputs.len() != 2 {
                    return Err(EmitError::UnsupportedBehavior(format!(
                        "concat runtime arity {} is not supported; expected [left, right]",
                        consumer.inputs.len()
                    )));
                }
                let left = self.render_input_use(
                    InputConsumer::Transform(consumer),
                    InputSlot::Positional(0),
                    locals,
                )?;
                let right = self.render_input_use(
                    InputConsumer::Transform(consumer),
                    InputSlot::Positional(1),
                    locals,
                )?;
                Ok(render_named_template(
                    &self.indexes.syntax.collection_ops.concat,
                    &[("left", &left), ("right", &right)],
                ))
            }
            RustCallableStrategyBinding::ListLength => {
                if consumer.inputs.len() != 1 {
                    return Err(EmitError::UnsupportedBehavior(format!(
                        "length runtime arity {} is not supported; expected [list]",
                        consumer.inputs.len()
                    )));
                }
                let recv = self.render_collection_receiver(
                    InputConsumer::Transform(consumer),
                    InputSlot::Positional(0),
                    locals,
                )?;
                Ok(render_named_template(
                    &self.indexes.syntax.collection_ops.length,
                    &[("recv", &recv)],
                ))
            }
            RustCallableStrategyBinding::ListIsEmpty => {
                if consumer.inputs.len() != 1 {
                    return Err(EmitError::UnsupportedBehavior(format!(
                        "is_empty runtime arity {} is not supported; expected [list]",
                        consumer.inputs.len()
                    )));
                }
                let recv = self.render_collection_receiver(
                    InputConsumer::Transform(consumer),
                    InputSlot::Positional(0),
                    locals,
                )?;
                Ok(render_named_template(
                    &self.indexes.syntax.collection_ops.is_empty,
                    &[("recv", &recv)],
                ))
            }
            RustCallableStrategyBinding::ListFold => {
                if consumer.inputs.len() != 2 {
                    return Err(EmitError::UnsupportedBehavior(format!(
                        "fold runtime arity {} is not supported; expected [list, init]",
                        consumer.inputs.len()
                    )));
                }
                let fn_decl = bound_callable_argument(self.dag, template, arguments, 2)?;
                let acc = "__fold_acc".to_string();
                let item = "__fold_item".to_string();
                let body = self.render_closure(
                    fn_decl,
                    &[
                        (acc.clone(), LocalBinding::Owned(acc.clone())),
                        (item.clone(), LocalBinding::Borrowed(item.clone())),
                    ],
                    locals,
                )?;
                let list = self.render_collection_receiver(
                    InputConsumer::Transform(consumer),
                    InputSlot::Positional(0),
                    locals,
                )?;
                let init = self.render_input_use(
                    InputConsumer::Transform(consumer),
                    InputSlot::Positional(1),
                    locals,
                )?;
                Ok(render_named_template(
                    &self.indexes.syntax.collection_ops.fold,
                    &[("recv", &list), ("init", &init), ("body", &body)],
                ))
            }
            RustCallableStrategyBinding::ListMap => {
                if consumer.inputs.len() != 1 {
                    return Err(EmitError::UnsupportedBehavior(format!(
                        "map runtime arity {} is not supported; expected [list]",
                        consumer.inputs.len()
                    )));
                }
                let fn_decl = bound_callable_argument(self.dag, template, arguments, 1)?;
                let item = "__map_item".to_string();
                let body = self.render_closure(
                    fn_decl,
                    &[(item.clone(), LocalBinding::Borrowed(item.clone()))],
                    locals,
                )?;
                let list = self.render_collection_receiver(
                    InputConsumer::Transform(consumer),
                    InputSlot::Positional(0),
                    locals,
                )?;
                Ok(render_named_template(
                    &self.indexes.syntax.collection_ops.map,
                    &[("recv", &list), ("body", &body)],
                ))
            }
            RustCallableStrategyBinding::ListFilter => {
                if consumer.inputs.len() != 1 {
                    return Err(EmitError::UnsupportedBehavior(format!(
                        "filter runtime arity {} is not supported; expected [list]",
                        consumer.inputs.len()
                    )));
                }
                let fn_decl = bound_callable_argument(self.dag, template, arguments, 1)?;
                let item = "__filter_item".to_string();
                let predicate = self.render_callable_body(
                    fn_decl,
                    &[(item.clone(), LocalBinding::Borrowed(item.clone()))],
                    locals,
                )?;
                let list = self.render_collection_receiver(
                    InputConsumer::Transform(consumer),
                    InputSlot::Positional(0),
                    locals,
                )?;
                let item_push = self.render_list_item_construct_expr(consumer.inputs[0], &item)?;
                Ok(render_named_template(
                    &self.indexes.syntax.collection_ops.filter,
                    &[
                        ("recv", &list),
                        ("item", &item),
                        ("predicate", &predicate),
                        ("item_push", &item_push),
                    ],
                ))
            }
            RustCallableStrategyBinding::ListContains => {
                if consumer.inputs.len() != 2 {
                    return Err(EmitError::UnsupportedBehavior(format!(
                        "contains runtime arity {} is not supported; expected [list, item]",
                        consumer.inputs.len()
                    )));
                }
                let list = self.render_collection_receiver(
                    InputConsumer::Transform(consumer),
                    InputSlot::Positional(0),
                    locals,
                )?;
                let item = self.render_input_use(
                    InputConsumer::Transform(consumer),
                    InputSlot::Positional(1),
                    locals,
                )?;
                Ok(render_named_template(
                    &self.indexes.syntax.collection_ops.contains,
                    &[("recv", &list), ("item", &item)],
                ))
            }
        }
    }

    fn render_general_callable(
        &self,
        consumer: &TransformNode,
        template: DeclarationId,
        locals: &RenderLocals,
    ) -> Result<String, EmitError> {
        if let Some(rendered) = self.render_variant_constructor(consumer, template, locals)? {
            return Ok(rendered);
        }
        if let Some(rendered) = self.render_record_constructor(consumer, template, locals)? {
            return Ok(rendered);
        }
        let func =
            self.dag
                .declaration(template)
                .name
                .clone()
                .ok_or(EmitError::UnsupportedBehavior(
                    "callable target is anonymous and cannot be rendered as a direct Rust call"
                        .to_string(),
                ))?;
        let args = consumer
            .inputs
            .iter()
            .enumerate()
            .map(|(slot, _)| {
                self.render_input_use(
                    InputConsumer::Transform(consumer),
                    InputSlot::Positional(slot),
                    locals,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let joined = join_rendered(&args, ", ");
        Ok(render_named_template(
            &self.indexes.syntax.expressions.function_call,
            &[("func", &func), ("args", &joined)],
        ))
    }

    fn render_record_constructor(
        &self,
        consumer: &TransformNode,
        template: DeclarationId,
        locals: &RenderLocals,
    ) -> Result<Option<String>, EmitError> {
        let decl = self.dag.declaration(template);
        let Some(type_name) = &decl.name else {
            return Ok(None);
        };
        let TypeConnective::Conj { children } = &decl.connective else {
            return Ok(None);
        };
        if children.len() != consumer.inputs.len() {
            return Err(EmitError::UnsupportedBehavior(format!(
                "record constructor `{type_name}` expected {} field input(s), got {}",
                children.len(),
                consumer.inputs.len()
            )));
        }
        let fields = children
            .iter()
            .enumerate()
            .map(|(slot, field)| {
                let value = self.render_input_use(
                    InputConsumer::Transform(consumer),
                    InputSlot::Positional(slot),
                    locals,
                )?;
                Ok(render_named_template(
                    &self.indexes.syntax.values.struct_field_init,
                    &[("name", &field.label), ("value", &value)],
                ))
            })
            .collect::<Result<Vec<_>, EmitError>>()?;
        let joined = join_rendered(&fields, &self.indexes.syntax.values.struct_field_separator);
        Ok(Some(render_named_template(
            &self.indexes.syntax.values.struct_literal,
            &[("type", type_name), ("fields", &joined)],
        )))
    }

    fn render_variant_constructor(
        &self,
        consumer: &TransformNode,
        template: DeclarationId,
        locals: &RenderLocals,
    ) -> Result<Option<String>, EmitError> {
        let Some((enum_name, variant_name)) = variant_parent_info(self.dag, template) else {
            return Ok(None);
        };
        let qualified_name = self.qualified_name(&enum_name, &variant_name);
        let TypeConnective::Conj { children } = &self.dag.declaration(template).connective else {
            return Ok(None);
        };
        if children.len() != consumer.inputs.len() {
            return Err(EmitError::UnsupportedBehavior(format!(
                "variant constructor `{qualified_name}` expected {} payload field(s), got {}",
                children.len(),
                consumer.inputs.len()
            )));
        }
        if children.is_empty() {
            return Ok(Some(qualified_name));
        }
        // Dissolution: same "tuple `Hit` for `v3.std.lookup` only" bridge as
        // `render_single_field_variant_pattern` (pattern side); see long comment
        // there. Until variant payload positionality is DAG-carried, keep narrow.
        if children.len() == 1
            && children[0].label == "_0"
            && enum_name == "Lookup"
            && variant_name == "Hit"
        {
            let value = self.elide_explicit_borrow(&self.render_input_use(
                InputConsumer::Transform(consumer),
                InputSlot::Positional(0),
                locals,
            )?);
            // `elide` may leave a parenthesized value (`(0)`) for literals; do not
            // add a second paren layer (`((0))`).
            let out = if value.starts_with('(') {
                format!("{qualified_name}{value}")
            } else {
                format!("{qualified_name}({value})")
            };
            return Ok(Some(out));
        }
        let fields = children
            .iter()
            .enumerate()
            .map(|(slot, field)| {
                let value = self.render_input_use(
                    InputConsumer::Transform(consumer),
                    InputSlot::Positional(slot),
                    locals,
                )?;
                Ok(render_named_template(
                    &self.indexes.syntax.values.struct_field_init,
                    &[("name", &field.label), ("value", &value)],
                ))
            })
            .collect::<Result<Vec<_>, EmitError>>()?;
        let joined = join_rendered(&fields, &self.indexes.syntax.values.struct_field_separator);
        Ok(Some(render_named_template(
            &self.indexes.syntax.values.variant_named_construction,
            &[("variant", &qualified_name), ("fields", &joined)],
        )))
    }

    fn qualified_name(&self, left: &str, right: &str) -> String {
        format!(
            "{left}{}{right}",
            self.indexes.syntax.modules.path_separator
        )
    }

    fn render_callable_body(
        &self,
        callable_decl: DeclarationId,
        param_bindings: &[(String, LocalBinding)],
        outer_locals: &RenderLocals,
    ) -> Result<String, EmitError> {
        let TypeConnective::Arrow { inputs, body, .. } =
            &self.dag.declaration(callable_decl).connective
        else {
            return Err(EmitError::UnsupportedBehavior(
                "callable template binding did not resolve to an Arrow declaration".to_string(),
            ));
        };
        let ArrowBody::UserDefined(bind_id) = body else {
            return Err(EmitError::UnsupportedBehavior(
                "external or unparsed callable bodies are not yet supported in staged std.list emission"
                    .to_string(),
            ));
        };
        let bind = (*bind_id).bind(self.dag);
        if inputs.len() != param_bindings.len() {
            return Err(EmitError::UnsupportedBehavior(
                "callable parameter count does not match the requested Rust closure parameters"
                    .to_string(),
            ));
        }
        if bind.params.len() < inputs.len() {
            return Err(EmitError::UnsupportedBehavior(
                "callable bind parameter count does not match Arrow inputs".to_string(),
            ));
        }
        let capture_count = bind.params.len() - inputs.len();
        let mut locals = RenderLocals::default();
        for capture in bind.params.iter().copied().take(capture_count) {
            let value = self.render_port(capture, outer_locals, RenderMode::BorrowedRead)?;
            locals.names.insert(capture, LocalBinding::Borrowed(value));
        }
        for (port, (_, binding)) in bind
            .params
            .iter()
            .copied()
            .skip(capture_count)
            .zip(param_bindings.iter())
        {
            locals.names.insert(port, binding.clone());
        }
        self.render_port(bind.value, &locals, RenderMode::OwnedConstructLastUse)
    }

    fn render_closure(
        &self,
        callable_decl: DeclarationId,
        param_bindings: &[(String, LocalBinding)],
        outer_locals: &RenderLocals,
    ) -> Result<String, EmitError> {
        let body = self.render_callable_body(callable_decl, param_bindings, outer_locals)?;
        let joined = join_rendered(
            &param_bindings
                .iter()
                .map(|(name, _)| name.clone())
                .collect::<Vec<_>>(),
            ", ",
        );
        Ok(render_named_template(
            &self.indexes.syntax.expressions.closure,
            &[("params", &joined), ("body", &body)],
        ))
    }

    fn render_loop(
        &self,
        l: &crate::dag::LoopNode,
        locals: &RenderLocals,
    ) -> Result<String, EmitError> {
        let body_port = super::behavior_result_port(self.dag.node(l.body));
        self.render_port(body_port, locals, RenderMode::OwnedConstructLastUse)
    }

    fn render_function_declaration(
        &self,
        declaration: &crate::dag::Declaration,
    ) -> Result<String, EmitError> {
        let Some(name) = &declaration.name else {
            return Err(EmitError::UnsupportedBehavior(
                "anonymous Arrow declarations cannot be emitted as top-level Rust functions"
                    .to_string(),
            ));
        };
        let TypeConnective::Arrow {
            inputs,
            output,
            body,
        } = &declaration.connective
        else {
            return Err(EmitError::UnsupportedBehavior(
                "render_function_declaration expected an Arrow declaration".to_string(),
            ));
        };
        if !declaration.type_params.is_empty() {
            return Err(EmitError::UnsupportedBehavior(
                "generic user-defined functions are not yet supported by emit_rust".to_string(),
            ));
        }
        let ArrowBody::UserDefined(bind_id) = body else {
            return Err(EmitError::UnsupportedBehavior(
                "external or unparsed Arrow bodies are not yet supported in function emission"
                    .to_string(),
            ));
        };
        let bind = (*bind_id).bind(self.dag);
        let param_dispositions = self.callable_param_dispositions(declaration.id, inputs.len());
        let mut locals = RenderLocals::default();
        let mut output_callable_walk = HashSet::new();
        // C-8 / #676: there is no separate "return-only" / `AppliedTypeArguments` walk. The
        // return slot uses the same `decl_includes_first_class_arrow_data` as struct derives /
        // storage — including `Instantiation` (args **and** non-`List` **template**), so
        // first-class `fn` living only "under" a template head still sets this flag. That
        // unifies with `rust_type_name_for_user_function_parameter` (compose `Rc` when the
        // return carries callable data anywhere).
        let return_includes_first_class_arrow =
            self.decl_includes_first_class_arrow_data(*output, &mut output_callable_walk);
        let params = bind
            .params
            .iter()
            .zip(param_dispositions.iter())
            .enumerate()
            .map(|(idx, (port, disposition))| {
                let param_name = format!("p{idx}");
                let ty_decl = self
                    .dag
                    .port(*port)
                    .value_type()
                    .ok_or(EmitError::UntypedPort(*port))?
                    .declaration;
                let callable_param_ty = self.type_declaration_peels_to_arrow(ty_decl);
                let ty = self.rust_type_name_for_user_function_parameter(
                    *port,
                    *disposition,
                    return_includes_first_class_arrow,
                )?;
                match disposition {
                    ParameterDispositionBinding::Borrowed
                        if self.read_strategy() == ReadStrategyBinding::Borrow
                            && !callable_param_ty =>
                    {
                        locals
                            .names
                            .insert(*port, LocalBinding::Borrowed(param_name.clone()));
                    }
                    _ => {
                        locals
                            .names
                            .insert(*port, LocalBinding::Owned(param_name.clone()));
                    }
                }
                Ok(render_named_template(
                    &self.indexes.syntax.functions.param_with_type,
                    &[("name", &param_name), ("type", &ty)],
                ))
            })
            .collect::<Result<Vec<_>, EmitError>>()?;
        if bind.params.len() != inputs.len() {
            return Err(EmitError::UnsupportedBehavior(
                "function bind parameter count does not match Arrow inputs".to_string(),
            ));
        }
        let params_joined = join_rendered(&params, &self.indexes.syntax.functions.param_separator);
        let ret = self
            .indexes
            .types
            .get(output)
            .map(|binding| binding.carrier.clone())
            .or_else(|| self.rust_type_name_for_port(bind.value).ok())
            .ok_or(EmitError::MissingTypeRealization { target: *output })?;
        let body = self.render_port(bind.value, &locals, RenderMode::OwnedConstructLastUse)?;
        let rendered = render_named_template(
            match self.mode {
                EmitRustMode::Program => &self.indexes.syntax.functions.definition,
                EmitRustMode::Module => &self.indexes.syntax.functions.definition_exported,
            },
            &[
                ("name", name),
                ("params", &params_joined),
                ("ret", &ret),
                ("body", &body),
            ],
        );
        Ok(rendered)
    }

    fn render_type_declaration(
        &self,
        declaration: &crate::dag::Declaration,
    ) -> Result<String, EmitError> {
        let Some(name) = &declaration.name else {
            return Err(EmitError::UnsupportedBehavior(
                "anonymous type declarations cannot be emitted".to_string(),
            ));
        };
        if !declaration.type_params.is_empty() {
            return Err(EmitError::UnsupportedBehavior(format!(
                "generic type declaration `{name}` is not yet supported by emit_rust"
            )));
        }
        match &declaration.connective {
            TypeConnective::Conj { children } => {
                // `Rc<dyn Fn…>` fields are not `Debug`. `rust_type_defs.struct_def` still
                // carries `#[derive(Clone, Debug)]` for ordinary records — when any field
                // transitively holds first-class `fn` data, use `rust_record_derive_templates`
                // instead (`struct_def_no_debug`: clone-only). See `decl_includes_first_class_arrow_data`
                // and `emit_callable_field_types_use_rc_dyn_fn_storage` (INVARIANTS.md C-8).
                let omit_debug = children.iter().any(|field| {
                    let mut visited = HashSet::new();
                    self.decl_includes_first_class_arrow_data(field.ty, &mut visited)
                });
                let fields = children
                    .iter()
                    .map(|field| self.render_struct_field(field))
                    .collect::<Result<Vec<_>, _>>()?;
                let fields_joined = join_rendered(&fields, " ");
                let template = if omit_debug {
                    &self
                        .indexes
                        .syntax
                        .record_derive_no_debug
                        .struct_def_no_debug
                } else {
                    &self.indexes.syntax.type_definitions.struct_def
                };
                Ok(render_named_template(
                    template,
                    &[("name", name), ("fields", &fields_joined)],
                ))
            }
            TypeConnective::Disj { variants } => {
                // Same `Debug` omission as records when variant payloads carry `Rc<dyn Fn…>`.
                let omit_debug = variants.iter().any(|variant| {
                    let mut visited = HashSet::new();
                    self.decl_includes_first_class_arrow_data(variant.ty, &mut visited)
                });
                let rendered_variants = variants
                    .iter()
                    .map(|variant| self.render_enum_variant(variant))
                    .collect::<Result<Vec<_>, _>>()?;
                let variants_joined = join_rendered(&rendered_variants, " ");
                let template = if omit_debug {
                    &self.indexes.syntax.record_derive_no_debug.enum_def_no_debug
                } else {
                    &self.indexes.syntax.type_definitions.enum_def
                };
                Ok(render_named_template(
                    template,
                    &[("name", name), ("variants", &variants_joined)],
                ))
            }
            _ => Err(EmitError::UnsupportedBehavior(format!(
                "type declaration `{name}` does not lower to a record or sum shape"
            ))),
        }
    }

    fn render_struct_field(&self, field: &Field) -> Result<String, EmitError> {
        // Record fields are **not** `impl Trait` positions. First-class `fn`
        // types use the storage carrier (`Rc<dyn Fn…>`) under
        // `ArrowRustEmitPolicy::StorageRcDynFn` — never `rust_arrow_as_parameter_impl_fn_clone`
        // (PR #676 inline review / INVARIANTS.md C-8 fail-closed vs plausible Rust).
        let ty = self.rust_type_name_for_decl_storage(field.ty)?;
        Ok(render_named_template(
            &self.indexes.syntax.type_definitions.struct_field,
            &[("name", &field.label), ("type", &ty)],
        ))
    }

    fn render_enum_variant(&self, variant: &Field) -> Result<String, EmitError> {
        let variant_decl = self.dag.declaration(variant.ty);
        let TypeConnective::Conj { children } = &variant_decl.connective else {
            return Err(EmitError::UnsupportedBehavior(format!(
                "enum variant `{}` does not lower to a product declaration",
                variant.label
            )));
        };
        if children.is_empty() {
            return Ok(render_named_template(
                &self.indexes.syntax.type_definitions.enum_unit_variant,
                &[("name", &variant.label)],
            ));
        }
        let fields = children
            .iter()
            .map(|field| {
                let rendered = self.render_struct_field(field)?;
                Ok(rendered.replacen("pub ", "", 1))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let fields_joined = join_rendered(&fields, " ");
        Ok(render_named_template(
            &self.indexes.syntax.type_definitions.enum_data_variant,
            &[("name", &variant.label), ("fields", &fields_joined)],
        ))
    }

    /// Sort a Branch's paths into (then, else) for if/else emission.
    /// Walks the scrutinee's port type to its Disj children, finds
    /// the True/False variants (resolved against `Classical` / Bool)
    /// by structural position, and matches each path's
    /// ResolvedVariant declaration id against them. Zero name
    /// strings — the True/False distinction comes from the
    /// scrutinee's Disj order, which is itself a fact of std/logic.dag.
    fn split_bool_paths<'p>(&self, b: &'p BranchNode) -> Result<(&'p Path, &'p Path), EmitError> {
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

    /// Rust type for a **user `fn` item parameter** only (`render_function_declaration`).
    /// Record / enum payload field types never call this helper: they use
    /// `render_struct_field` → `rust_type_name_for_decl_storage` (`Rc<dyn Fn…>`
    /// for first-class `fn` data; INVARIANTS.md C-8 / PR #676 inline review).
    ///
    /// Callable-shaped parameters use `impl Fn(...) -> T + Clone` (v2 / PR #650)
    /// **unless** the function's declared return type carries first-class `fn`
    /// anywhere — then callable parameters use `std::rc::Rc<dyn Fn…>` so
    /// pass-through composes with the return type (rustc `E0308` otherwise).
    /// Other parameters use `rust_type_name_for_port` /
    /// `rust_borrowed_type_name_for_port`.
    fn rust_type_name_for_user_function_parameter(
        &self,
        port: PortId,
        disposition: ParameterDispositionBinding,
        return_includes_first_class_arrow: bool,
    ) -> Result<String, EmitError> {
        let ty_decl = self
            .dag
            .port(port)
            .value_type()
            .ok_or(EmitError::UntypedPort(port))?
            .declaration;
        if self.type_declaration_peels_to_arrow(ty_decl) {
            if return_includes_first_class_arrow {
                return self.rust_type_name_for_decl_storage(ty_decl);
            }
            return self.rust_arrow_as_parameter_impl_fn_clone(ty_decl);
        }
        match disposition {
            ParameterDispositionBinding::Borrowed
                if self.read_strategy() == ReadStrategyBinding::Borrow =>
            {
                self.rust_borrowed_type_name_for_port(port)
            }
            _ => self.rust_type_name_for_port(port),
        }
    }

    /// `impl Fn(...) -> R + Clone` for a first-class `fn` type used **only**
    /// in direct user-function parameter position (this is **not** the
    /// context-free `rust_type_name_for_decl_with_policy` renderer — record
    /// fields / `Vec<…>` / etc. never call here). Nested parameter/return types
    /// use `rust_type_name_for_decl_storage` (`Rc<dyn Fn…>`). See
    /// `ArrowRustEmitPolicy` and `src/v3/spec/rust.dag` (first-class callable note).
    /// Future: lift the two spellings into `rust.dag` `TypeRealization`-style rows
    /// for full THESIS “declared target realization” parity (today: policy + spec comment).
    fn rust_arrow_as_parameter_impl_fn_clone(
        &self,
        declaration: DeclarationId,
    ) -> Result<String, EmitError> {
        let Some((inputs, output)) = self.peel_resolved_chain_to_first_class_arrow(declaration)
        else {
            return Err(EmitError::UnsupportedBehavior(
                "rust_arrow_as_parameter_impl_fn_clone: declaration does not peel to Arrow"
                    .to_string(),
            ));
        };
        let param_types = inputs
            .iter()
            .map(|i| self.rust_type_name_for_decl_storage(*i))
            .collect::<Result<Vec<_>, _>>()?;
        let param_str = param_types.join(", ");
        let ret_str = self.rust_type_name_for_decl_storage(output)?;
        Ok(format!("impl Fn({param_str}) -> {ret_str} + Clone"))
    }

    /// Rust type spelling for a port in **storage-class** positions: resolves the
    /// port's value type to a declaration id, then `rust_type_name_for_decl_storage`
    /// (including the `types` realization index when populated). Not used for user
    /// `fn` parameters — those use `rust_type_name_for_user_function_parameter` first.
    fn rust_type_name_for_port(&self, port: PortId) -> Result<String, EmitError> {
        let ty = self
            .dag
            .port(port)
            .value_type()
            .ok_or(EmitError::UntypedPort(port))?;
        self.rust_type_name_for_decl_storage(ty.declaration)
    }

    /// Rust type spelling for a port consumed under **borrow** read strategy
    /// when the port is **not** a first-class callable (`fn` data) type.
    /// Callable-shaped user `fn` parameters are routed in
    /// `rust_type_name_for_user_function_parameter` **before** this helper is
    /// consulted (`impl Fn + Clone` or `Rc<dyn Fn…>` per return composition).
    /// Non-callable ports delegate to `rust_borrowed_type_name_for_decl` (`&T` /
    /// `&[U]` only).
    fn rust_borrowed_type_name_for_port(&self, port: PortId) -> Result<String, EmitError> {
        let ty = self
            .dag
            .port(port)
            .value_type()
            .ok_or(EmitError::UntypedPort(port))?;
        self.rust_borrowed_type_name_for_decl(ty.declaration)
    }

    fn port_is_substrate_result(&self, port: PortId) -> Result<bool, EmitError> {
        let ty = self
            .dag
            .port(port)
            .value_type()
            .ok_or(EmitError::UntypedPort(port))?;
        let mut visited = HashSet::new();
        Ok(self.decl_is_substrate_result_rec(ty.declaration, &mut visited))
    }

    fn decl_is_substrate_result_rec(
        &self,
        declaration: DeclarationId,
        visited: &mut HashSet<DeclarationId>,
    ) -> bool {
        if !visited.insert(declaration) {
            return false;
        }
        let decl = self.dag.declaration(declaration);
        match &decl.connective {
            TypeConnective::Instantiation { template, .. } => {
                super::substrate_result_type_decl_suppressed_for_emit(
                    self.dag,
                    self.dag.declaration(*template),
                ) || self.decl_is_substrate_result_rec(*template, visited)
            }
            TypeConnective::Atom(AtomPayload::ResolvedByStructure(next))
            | TypeConnective::Atom(AtomPayload::ResolvedByName(next)) => {
                self.decl_is_substrate_result_rec(*next, visited)
            }
            _ => super::substrate_result_type_decl_suppressed_for_emit(self.dag, decl),
        }
    }

    /// Peel `ResolvedBy{Structure,Name}` atoms, and **zero-arity
    /// `Instantiation` aliases** (`type G = F` lower as `Instantiate(F, []);`
    /// not `ResolvedByName`) until a first-class `TypeConnective::Arrow` with
    /// `ArrowBody::NoBody` is found.
    /// `None` on cycle, on `ArrowBody::UserDefined` / other heads, or when no
    /// such arrow is reachable.
    fn peel_resolved_chain_to_first_class_arrow(
        &self,
        mut declaration: DeclarationId,
    ) -> Option<(Vec<DeclarationId>, DeclarationId)> {
        let mut visited = HashSet::new();
        loop {
            if !visited.insert(declaration) {
                return None;
            }
            let decl = self.dag.declaration(declaration);
            match &decl.connective {
                TypeConnective::Arrow {
                    inputs,
                    output,
                    body: ArrowBody::NoBody,
                } => {
                    return Some((inputs.clone(), *output));
                }
                TypeConnective::Arrow { .. } => return None,
                TypeConnective::Atom(AtomPayload::ResolvedByStructure(next))
                | TypeConnective::Atom(AtomPayload::ResolvedByName(next)) => {
                    declaration = *next;
                }
                TypeConnective::Instantiation {
                    template,
                    arguments,
                } if arguments.is_empty() => {
                    declaration = *template;
                }
                _ => return None,
            }
        }
    }

    /// True when `declaration` (after peeling `ResolvedBy*` and zero-arity
    /// `Instantiation` type aliases) is a first-class function type (`fn(...) -> _` in surface),
    /// i.e. a `TypeConnective::Arrow` with `ArrowBody::NoBody` (data `fn`, not a
    /// user `fn` item's `ArrowBody::UserDefined`).
    fn type_declaration_peels_to_arrow(&self, declaration: DeclarationId) -> bool {
        self.peel_resolved_chain_to_first_class_arrow(declaration)
            .is_some()
    }

    /// Bootstrap `Int` / `Bool` / `String` type roots. Their expanded substrate
    /// (rings, algebras) contains `TypeConnective::Arrow` for operations, not
    /// first-class `fn` data — we must not confuse that with a user `fn` when
    /// classifying a **return** type; see `decl_includes_first_class_arrow_data`.
    ///
    /// **Extension / dissolve:** a new std primitive with the same
    /// algebra-`Arrow` pattern (e.g. `Float`, `Bytes`) must extend
    /// `Dag::first_class_fn_walk_bootstrap_prune_type_shapes` in the same PR, or
    /// replace with a `Dag`-driven "numeric / algebra" classification.
    fn declaration_is_bootstrap_int_bool_string(&self, declaration: DeclarationId) -> bool {
        self.dag
            .first_class_fn_walk_bootstrap_prune_type_shapes()
            .iter()
            .any(|s| s.declaration == declaration)
    }

    /// True when `declaration` (including nested record / sum / instantiations)
    /// carries first-class `fn` (`TypeConnective::Arrow` + `ArrowBody::NoBody`)
    /// anywhere, so (a) storage positions use `Rc<dyn Fn…>` and (b) `Debug`
    /// is omitted for user `struct` / `enum` derives. Used for return-vs-param
    /// callable **carrier** selection (user `fn` params) and for derive
    /// templates.
    ///
    /// For `Instantiation`, we walk type **arguments** and, when the head is
    /// not the `List` template, the **template** (so a user `G<T>` can carry
    /// callable data on `G` without it appearing in `T`). The `List` head and
    /// the `PartialFunction` head (surface `Map<K, V>`) are skipped: the first
    /// is always element-shaped in `T`; the second is a `Conj` of algebra
    /// `Arrow`+`NoBody` *operations* (lookup, insert, …) that must not be
    /// taken for user first-class `fn` *values* (C-8; #676).
    /// The primitive check above **short-circuits** `-> Int` (etc.) so we do
    /// not follow those type roots into ring/algebra noise.
    fn decl_includes_first_class_arrow_data(
        &self,
        declaration: DeclarationId,
        visited: &mut HashSet<DeclarationId>,
    ) -> bool {
        if !visited.insert(declaration) {
            return false;
        }
        if self.declaration_is_bootstrap_int_bool_string(declaration) {
            return false;
        }
        if self.type_declaration_peels_to_arrow(declaration) {
            return true;
        }
        let decl = self.dag.declaration(declaration);
        match &decl.connective {
            TypeConnective::Conj { children } => children
                .iter()
                .any(|field| self.decl_includes_first_class_arrow_data(field.ty, visited)),
            TypeConnective::Disj { variants } => variants
                .iter()
                .any(|variant| self.decl_includes_first_class_arrow_data(variant.ty, visited)),
            TypeConnective::Instantiation {
                template,
                arguments,
            } => {
                let from_args = arguments
                    .iter()
                    .any(|arg| self.decl_includes_first_class_arrow_data(arg.value, visited));
                from_args
                    || (!self.is_list_template(*template)
                        && !self.is_partial_function_template(*template)
                        && self.decl_includes_first_class_arrow_data(*template, visited))
            }
            TypeConnective::Cardinality(p) => {
                self.decl_includes_first_class_arrow_data(p.element(), visited)
            }
            TypeConnective::Atom(AtomPayload::ResolvedByStructure(next))
            | TypeConnective::Atom(AtomPayload::ResolvedByName(next)) => {
                self.decl_includes_first_class_arrow_data(*next, visited)
            }
            _ => false,
        }
    }

    /// `&T` / `&[U]` spellings for borrowed emission. **Does not** handle peeled
    /// callable `Arrow` types — callers must use
    /// `rust_type_name_for_user_function_parameter` for user `fn` parameters
    /// (which selects `impl Fn + Clone` vs `Rc<dyn Fn…>` before borrow paths).
    fn rust_borrowed_type_name_for_decl(
        &self,
        declaration: DeclarationId,
    ) -> Result<String, EmitError> {
        let decl = self.dag.declaration(declaration);
        match &decl.connective {
            TypeConnective::Instantiation {
                template,
                arguments,
            } => {
                if self.is_list_template(*template) {
                    let [element] = arguments.as_slice() else {
                        return Err(EmitError::UnsupportedBehavior(
                            "borrowed List carrier expects exactly one type argument".to_string(),
                        ));
                    };
                    let element_name = self.rust_type_name_for_decl_storage(element.value)?;
                    Ok(format!("&[{element_name}]"))
                } else {
                    Ok(format!(
                        "&{}",
                        self.rust_type_name_for_decl_storage(declaration)?
                    ))
                }
            }
            _ => Ok(format!(
                "&{}",
                self.rust_type_name_for_decl_storage(declaration)?
            )),
        }
    }

    /// Vetted storage positions (`Rc<dyn Fn…>`) for anonymous `fn` types.
    fn rust_type_name_for_decl_storage(
        &self,
        declaration: DeclarationId,
    ) -> Result<String, EmitError> {
        self.rust_type_name_for_decl_with_policy(
            declaration,
            0,
            ArrowRustEmitPolicy::StorageRcDynFn,
        )
    }

    fn rust_type_name_for_decl_with_policy(
        &self,
        declaration: DeclarationId,
        depth: usize,
        arrow_policy: ArrowRustEmitPolicy,
    ) -> Result<String, EmitError> {
        if depth >= 32 {
            return Err(EmitError::UnsupportedBehavior(
                "type-name rendering exceeded depth 32 — likely a cycle".to_string(),
            ));
        }
        if let Some(binding) = self.indexes.types.get(&declaration) {
            return Ok(binding.carrier.clone());
        }
        let decl = self.dag.declaration(declaration);
        if let Some(name) = &decl.name {
            // Named user `struct` / `sum` (record or enum) are the Rust name authority
            // for a field like `cb: Callback` when `Callback` is its own `type` item,
            // even if that record transitively contains first-class `fn` storage — do
            // not run the alias-only `decl_includes_first_class_arrow_data` fast-path
            // below (codex #676: `Wrapper { cb: Callback }` with `Callback { f: fn… }`).
            if matches!(
                &decl.connective,
                TypeConnective::Conj { .. } | TypeConnective::Disj { .. }
            ) {
                return Ok(name.clone());
            }
            // `type F = fn(...) -> _` is a **named** declaration whose connective is
            // `Arrow` + `NoBody`. The Rust layer does not emit a `type F = …` typedef
            // for first-class `fn` data, so returning `F` would be plausible but
            // invalid (P3 fail-closed; #676). Fall through to the `Arrow` arm for
            // `Rc<dyn Fn…>` or `UnsupportedBehavior` per `arrow_policy`.
            let is_named_first_class_fn_alias = matches!(
                &decl.connective,
                TypeConnective::Arrow {
                    body: ArrowBody::NoBody,
                    ..
                }
            );
            // A **chain** `type G = F` with `F = fn..` is lowered as
            // `Instantiation(F, [])` (and sometimes `ResolvedByName`); a bare
            // `G` is invalid in Rust like a bare `F` (#676, P3, Codex).
            // When peeling finds first-class `fn` under the alias, skip this
            // return and use the `Instantiation` / `Arrow` path below.
            //
            // `type L = List<fn..>` (or `Map<.., fn..>`, etc.) does **not** peel
            // to a top-level `Arrow` — but `list` / `value` args still carry
            // first-class `fn` data. Emitting the bare `L` would be an undefined
            // Rust name (C-8; #676) — `decl_includes_first_class_arrow_data` matches
            // the same carrier rules as `decl_includes` on struct fields.
            if !is_named_first_class_fn_alias
                && self
                    .peel_resolved_chain_to_first_class_arrow(declaration)
                    .is_none()
            {
                let mut first_class_fn_walk = std::collections::HashSet::new();
                if !self.decl_includes_first_class_arrow_data(declaration, &mut first_class_fn_walk)
                {
                    return Ok(name.clone());
                }
            }
        }
        match &decl.connective {
            // `type G = F` → `Instantiate(F, [])` (see `type_to_connective`); alias is
            // not a `List`/`Map` template realization.
            TypeConnective::Instantiation {
                template,
                arguments,
            } if arguments.is_empty() => {
                self.rust_type_name_for_decl_with_policy(*template, depth + 1, arrow_policy)
            }
            TypeConnective::Instantiation {
                template,
                arguments,
            } => self.render_instantiated_type(*template, arguments, depth + 1, arrow_policy),
            TypeConnective::Cardinality(p)
                if p.bound() == crate::dag::CardinalityBound::AtMostOne =>
            {
                let inner =
                    self.rust_type_name_for_decl_with_policy(p.element(), depth + 1, arrow_policy)?;
                Ok(render_named_template(
                    &self.indexes.syntax.type_applications.optional,
                    &[("element", &inner)],
                ))
            }
            // Named user `Conj` / `Disj`: a field may hold first-class `fn` data, so
            // `decl_includes_first_class_arrow_data` can block the **alias** fast
            // path above — this connective is still a real `pub struct` / `enum` we
            // emit, and the bare name is a valid forward reference in other
            // parameter / return positions (PR #676 follow-up, claude review).
            TypeConnective::Conj { .. } | TypeConnective::Disj { .. } => {
                if let Some(n) = &decl.name {
                    Ok(n.clone())
                } else {
                    Err(EmitError::MissingTypeRealization { target: declaration })
                }
            }
            // First-class anonymous `fn` (`ArrowBody::NoBody` only): `impl Fn + Clone` is **not**
            // produced here — only `StorageRcDynFn` → `Rc<dyn Fn…>` or
            // `RejectFirstClassFn` → `UnsupportedBehavior` (INVARIANTS.md C-8).
            // `Arrow` with `UserDefined` / other bodies is a user `fn` item shape, not
            // first-class `fn` data — fail closed if it reaches storage rendering (P3).
            TypeConnective::Arrow {
                inputs,
                output,
                body: ArrowBody::NoBody,
            } => match arrow_policy {
                ArrowRustEmitPolicy::RejectFirstClassFn => Err(EmitError::UnsupportedBehavior(
                    "emit_rust: first-class function type (`fn(...) -> _` / TypeConnective::Arrow) \
                     in an unsupported Rust type-name context; supported carriers are: user `fn` \
                     parameters via `rust_type_name_for_user_function_parameter` (`impl Fn + \
                     Clone`, or `std::rc::Rc<dyn Fn…>` when the return type carries a first-class \
                     `fn` so param/ret compose), and struct fields / collection instantiations / \
                     top-level `let` / inferred return slots via `rust_type_name_for_decl_storage` \
                     (`std::rc::Rc<dyn Fn…>`). See `src/v3/spec/rust.dag` header on first-class callable surfaces."
                        .to_string(),
                )),
                ArrowRustEmitPolicy::StorageRcDynFn => {
                    let param_types = inputs
                        .iter()
                        .map(|i| {
                            self.rust_type_name_for_decl_with_policy(
                                *i,
                                depth + 1,
                                ArrowRustEmitPolicy::StorageRcDynFn,
                            )
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    let param_str = param_types.join(", ");
                    let ret_str = self.rust_type_name_for_decl_with_policy(
                        *output,
                        depth + 1,
                        ArrowRustEmitPolicy::StorageRcDynFn,
                    )?;
                    Ok(format!("std::rc::Rc<dyn Fn({param_str}) -> {ret_str}>"))
                }
            },
            TypeConnective::Arrow { .. } => Err(EmitError::UnsupportedBehavior(
                "emit_rust: TypeConnective::Arrow reached storage type-name rendering with a \
                 body other than `ArrowBody::NoBody`; only first-class anonymous `fn` data may \
                 use the `std::rc::Rc<dyn Fn…>` carrier here. User-defined function arrows must \
                 not be lowered through `rust_type_name_for_decl_storage`."
                    .to_string(),
            )),
            TypeConnective::Atom(AtomPayload::ResolvedByStructure(next))
            | TypeConnective::Atom(AtomPayload::ResolvedByName(next)) => {
                self.rust_type_name_for_decl_with_policy(*next, depth + 1, arrow_policy)
            }
            _ => Err(EmitError::MissingTypeRealization {
                target: declaration,
            }),
        }
    }

    fn render_instantiated_type(
        &self,
        template: DeclarationId,
        arguments: &[TemplateArgument],
        depth: usize,
        arrow_policy: ArrowRustEmitPolicy,
    ) -> Result<String, EmitError> {
        let Some(binding) = self.indexes.instantiations.get(&template) else {
            return Err(EmitError::MissingTypeRealization { target: template });
        };
        match arguments {
            [element] => {
                let element_name = self.rust_type_name_for_decl_with_policy(
                    element.value,
                    depth + 1,
                    arrow_policy,
                )?;
                Ok(render_named_template(
                    &binding.carrier,
                    &[("element", &element_name)],
                ))
            }
            [key, value] => {
                let key_name =
                    self.rust_type_name_for_decl_with_policy(key.value, depth + 1, arrow_policy)?;
                let value_name = self.rust_type_name_for_decl_with_policy(
                    value.value,
                    depth + 1,
                    arrow_policy,
                )?;
                Ok(render_named_template(
                    &binding.carrier,
                    &[("key", &key_name), ("value", &value_name)],
                ))
            }
            _ => Err(EmitError::UnsupportedBehavior(format!(
                "instantiated type carrier for declaration {:?} only supports arities 1 and 2 at PR scope",
                template
            ))),
        }
    }

    fn port_is_copy(&self, port: PortId) -> Result<bool, EmitError> {
        let ty = self
            .dag
            .port(port)
            .value_type()
            .ok_or(EmitError::UntypedPort(port))?;
        let mut visited = HashSet::new();
        self.decl_is_copy_rec(ty.declaration, &mut visited)
    }

    fn decl_is_copy(&self, declaration: DeclarationId) -> Result<bool, EmitError> {
        let mut visited = HashSet::new();
        self.decl_is_copy_rec(declaration, &mut visited)
    }

    fn decl_is_copy_rec(
        &self,
        declaration: DeclarationId,
        visited: &mut HashSet<DeclarationId>,
    ) -> Result<bool, EmitError> {
        if !visited.insert(declaration) {
            // Self-referential cycle. A recursive type holds itself,
            // which requires heap indirection on every target we emit —
            // it is never Copy.
            return Ok(false);
        }
        let decl = self.dag.declaration(declaration);
        if let Some(binding) = self.indexes.types.get(&declaration) {
            return Ok(binding.is_copy);
        }
        match &decl.connective {
            TypeConnective::Atom(AtomPayload::ResolvedByStructure(next))
            | TypeConnective::Atom(AtomPayload::ResolvedByName(next)) => {
                self.decl_is_copy_rec(*next, visited)
            }
            // For user-defined Conj/Disj/Instantiation/Cardinality
            // types without an explicit TypeRealization, conservatively
            // return false. The emitter derives `#[derive(Clone, Debug)]`
            // (not Copy) on user-defined types, so recursing through the
            // algebra and reporting "all leaves are Copy → composite is
            // Copy" mismatches the emitted Rust and produces use-after-
            // move errors. The cycle guard above still applies if a
            // future user-defined type recursively contains itself.
            TypeConnective::Conj { .. }
            | TypeConnective::Disj { .. }
            | TypeConnective::Instantiation { .. }
            | TypeConnective::Cardinality(_)
            | TypeConnective::Arrow { .. } => Ok(false),
            _ => Ok(false),
        }
    }

    fn port_is_list(&self, port: PortId) -> Result<bool, EmitError> {
        let ty = self
            .dag
            .port(port)
            .value_type()
            .ok_or(EmitError::UntypedPort(port))?;
        self.decl_is_list(ty.declaration)
    }

    fn decl_is_list(&self, declaration: DeclarationId) -> Result<bool, EmitError> {
        let decl = self.dag.declaration(declaration);
        match &decl.connective {
            TypeConnective::Atom(AtomPayload::ResolvedByStructure(next))
            | TypeConnective::Atom(AtomPayload::ResolvedByName(next)) => self.decl_is_list(*next),
            TypeConnective::Instantiation { template, .. } => Ok(self.is_list_template(*template)),
            _ => Ok(self.is_list_template(declaration)),
        }
    }

    fn is_list_template(&self, declaration: DeclarationId) -> bool {
        self.dag
            .list_template()
            .is_some_and(|list| list == declaration)
    }

    /// `Map<K, V>` instantiates the std `PartialFunction` record; its fields are
    /// not user first-class `fn` as in `peel_resolved_chain_to_first_class_arrow`.
    fn is_partial_function_template(&self, declaration: DeclarationId) -> bool {
        self.dag
            .partial_function_template()
            .is_some_and(|pfun| pfun == declaration)
    }

    fn render_list_item_construct_expr(
        &self,
        list_port: PortId,
        item_name: &str,
    ) -> Result<String, EmitError> {
        let ty = self
            .dag
            .port(list_port)
            .value_type()
            .ok_or(EmitError::UntypedPort(list_port))?;
        let TypeConnective::Instantiation {
            template,
            arguments,
        } = &self.dag.declaration(ty.declaration).connective
        else {
            return Err(EmitError::UnsupportedBehavior(
                "list construct rendering expected an instantiated List type".to_string(),
            ));
        };
        if !self.is_list_template(*template) {
            return Err(EmitError::UnsupportedBehavior(
                "list construct rendering expected the List template".to_string(),
            ));
        }
        let [element] = arguments.as_slice() else {
            return Err(EmitError::UnsupportedBehavior(
                "List instantiation should carry exactly one element argument".to_string(),
            ));
        };
        if self.decl_is_copy(element.value)? {
            Ok(format!("(*({item_name}))"))
        } else if self.decl_is_list(element.value)? {
            Ok(format!("({item_name}).to_vec()"))
        } else {
            Ok(format!("({item_name}).clone()"))
        }
    }
}

/// Anonymous specialized `Disj` nodes from `lower::specialize_decl_for_lowering`
/// set `specialization_parent = Some(template_disj_id)` so Rust emit can recover
/// the template enum name without cloning `Declaration::name` onto a second
/// declaration id (P2 / single-authority metadata for `Dag::declaration_by_name`).
///
/// Returns the template's `Declaration::name` once a named `Disj` is reached.
fn named_disj_enum_name_for_rust_match_emit(
    dag: &Dag,
    mut disj_id: DeclarationId,
) -> Option<String> {
    // Chains are one hop in practice (`specialize_decl_for_lowering`); 32 matches
    // `specialize_decl_for_lowering`'s depth bound so a bug cannot spin forever.
    for _ in 0..32 {
        let decl = dag.declaration(disj_id);
        let TypeConnective::Disj { .. } = &decl.connective else {
            return None;
        };
        if let Some(name) = decl.name.clone() {
            return Some(name);
        }
        disj_id = decl.specialization_parent?;
    }
    None
}

fn variant_name_for_decl(
    dag: &Dag,
    disj_id: DeclarationId,
    variant_id: DeclarationId,
) -> Result<String, EmitError> {
    let TypeConnective::Disj { variants } = &dag.declaration(disj_id).connective else {
        unreachable!("variant_name_for_decl requires a Disj parent")
    };
    variants
        .iter()
        .find(|variant| variant.ty == variant_id)
        .map(|variant| variant.label.clone())
        .ok_or_else(|| {
            EmitError::UnsupportedBehavior(format!(
                "variant {variant_id:?} was not found under parent disjunction {disj_id:?}"
            ))
        })
}

fn variant_parent_info(dag: &Dag, variant_id: DeclarationId) -> Option<(String, String)> {
    dag.declarations().iter().find_map(|decl| {
        let enum_name = decl.name.as_ref()?;
        let TypeConnective::Disj { variants } = &decl.connective else {
            return None;
        };
        variants
            .iter()
            .find(|variant| variant.ty == variant_id)
            .map(|variant| (enum_name.clone(), variant.label.clone()))
    })
}

fn callable_template(target: DeclarationId, dag: &Dag) -> (DeclarationId, Vec<TemplateArgument>) {
    match &dag.declaration(target).connective {
        TypeConnective::Instantiation {
            template,
            arguments,
        } => (*template, arguments.clone()),
        _ => (target, Vec::new()),
    }
}

fn bound_callable_argument(
    dag: &Dag,
    template: DeclarationId,
    arguments: &[TemplateArgument],
    input_index: usize,
) -> Result<DeclarationId, EmitError> {
    let TypeConnective::Arrow { inputs, .. } = &dag.declaration(template).connective else {
        return Err(EmitError::UnsupportedBehavior(
            "realized callable template did not resolve to an Arrow declaration".to_string(),
        ));
    };
    let Some(param_decl) = inputs.get(input_index).copied() else {
        return Err(EmitError::UnsupportedBehavior(format!(
            "realized callable slot {} is missing from the template declaration",
            input_index
        )));
    };
    arguments
        .iter()
        .find(|arg| arg.parameter == param_decl)
        .map(|arg| arg.value)
        .ok_or_else(|| {
            EmitError::UnsupportedBehavior(
                "realized callable argument did not bind through template instantiation"
                    .to_string(),
            )
        })
}

fn rust_string_literal_body(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_ascii() && !c.is_control() => out.push(c),
            c => {
                use std::fmt::Write;
                let _ = write!(&mut out, "\\u{{{:X}}}", c as u32);
            }
        }
    }
    out
}

fn render_value(v: &ValueNode, literals: &LiteralSyntaxBinding) -> String {
    match &v.data {
        LiteralBits::Int(n) => n.to_string(),
        LiteralBits::Bool(true) => literals.true_keyword.clone(),
        LiteralBits::Bool(false) => literals.false_keyword.clone(),
        LiteralBits::String(s) => format!(
            "String::from({}{}{})",
            literals.string_delimiter,
            rust_string_literal_body(s),
            literals.string_delimiter
        ),
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
    primitive_type_id_for_port_shared(dag, port).map_err(|err| match err {
        SharedEmitLookupError::UntypedPort(port) => EmitError::UntypedPort(port),
        SharedEmitLookupError::Unsupported(detail) => EmitError::UnsupportedBehavior(detail),
    })
}

fn walk_to_conj(dag: &Dag, start: DeclarationId) -> Option<DeclarationId> {
    let mut current = start;
    for _ in 0..32 {
        let decl = dag.declaration(current);
        match &decl.connective {
            TypeConnective::Conj { .. } => return Some(current),
            TypeConnective::Instantiation { template, .. } => current = *template,
            TypeConnective::Atom(AtomPayload::ResolvedByStructure(next))
            | TypeConnective::Atom(AtomPayload::ResolvedByName(next)) => current = *next,
            _ => return None,
        }
    }
    None
}

fn is_optional_match_disj(dag: &Dag, disj_id: DeclarationId) -> bool {
    dag.declarations()
        .iter()
        .filter_map(|decl| dag.optional_match_disj(decl.id))
        .any(|optional_disj| optional_disj == disj_id)
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
/// the name lives once in the generated operators authority
/// (`operators_generated.rs`, re-exported via `crate::operators`)
/// and the resolved declaration id is what flows downstream.
/// The emitter doesn't repeat the comparison; it asks this helper
/// for the field id and uses it as a typed index key.
fn algebra_field_for_operator(
    dag: &Dag,
    operand_type_id: DeclarationId,
    op: OperatorKind,
) -> Result<DeclarationId, EmitError> {
    algebra_field_for_operator_shared(dag, operand_type_id, op).map_err(|err| match err {
        SharedEmitLookupError::UntypedPort(port) => EmitError::UntypedPort(port),
        SharedEmitLookupError::Unsupported(detail) => EmitError::UnsupportedBehavior(detail),
    })
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::compile_to_dag;
    use crate::diagnostics::SourceSpan;

    #[test]
    fn first_class_fn_walk_does_not_confuse_map_partial_function_with_user_fn() {
        let dag = compile_to_dag("let _x: Int = 0\n", "t.v3").expect("compiles");
        let map = dag
            .declaration_by_name("Map")
            .expect("Map from std should load in bootstrap")
            .id;
        let indexes = RealizationIndexes::build(&dag).expect("indexes build");
        let input_use_facts = InputUseFacts::build(&dag, &indexes);
        let bound_names: HashMap<PortId, LocalBinding> = HashMap::new();
        let ctx = Ctx {
            dag: &dag,
            indexes: &indexes,
            bound_names: &bound_names,
            input_use_facts: &input_use_facts,
            mode: EmitRustMode::Module,
        };
        let mut visited = HashSet::new();
        assert!(
            !ctx.decl_includes_first_class_arrow_data(map, &mut visited),
            "`Map<K,V>`'s `PartialFunction` head carries `Arrow`+`NoBody` for algebra operations, \
not user `fn` data; must not set return-carrier / Rc on callable params (PR #676)"
        );
    }

    #[test]
    fn render_field_project_reads_borrowed_nodes_without_cloning() {
        let mut dag = Dag::new();
        let parent_port = dag.alloc_port(None);
        let dag_type = dag
            .dag_type_decl()
            .expect("Dag type realization target exists");
        let dag_nodes_type = match &dag.declaration(dag_type).connective {
            TypeConnective::Conj { children } => {
                children
                    .iter()
                    .find(|field| field.label == "nodes")
                    .expect("Dag.nodes field")
                    .ty
            }
            other => panic!("Dag must be a Conj, got {other:?}"),
        };
        dag.set_port_type(parent_port, crate::types::TypeShape::new(dag_type));
        let node_id = dag.alloc_node_id();
        let output = dag.alloc_port(Some(node_id));
        dag.push_node(Behavior::Transform(TransformNode {
            id: node_id,
            target: TransformTarget::FieldProject {
                field_label: "nodes".to_string(),
                field_child: Some(dag_nodes_type),
            },
            inputs: vec![parent_port],
            output,
            span: SourceSpan::new("<test>", 0, 0),
        }));
        dag.set_port_type(output, crate::types::TypeShape::new(dag_nodes_type));

        let indexes = RealizationIndexes::build(&dag).expect("indexes build");
        let input_use_facts = InputUseFacts::build(&dag, &indexes);
        let mut bound_names = HashMap::new();
        bound_names.insert(parent_port, LocalBinding::Owned("parent".to_string()));
        let ctx = Ctx {
            dag: &dag,
            indexes: &indexes,
            bound_names: &bound_names,
            input_use_facts: &input_use_facts,
            mode: EmitRustMode::Program,
        };

        let rendered = match dag.node(node_id) {
            Behavior::Transform(t) => ctx
                .render_transform(t, &RenderLocals::default(), RenderMode::BorrowedRead)
                .expect("field project renders"),
            other => panic!("expected Transform node, got {other:?}"),
        };
        assert_eq!(rendered, "(parent).nodes()");
    }

    #[test]
    fn render_field_project_constructs_owned_list_from_borrowed_nodes() {
        let mut dag = Dag::new();
        let parent_port = dag.alloc_port(None);
        let dag_type = dag
            .dag_type_decl()
            .expect("Dag type realization target exists");
        let dag_nodes_type = match &dag.declaration(dag_type).connective {
            TypeConnective::Conj { children } => {
                children
                    .iter()
                    .find(|field| field.label == "nodes")
                    .expect("Dag.nodes field")
                    .ty
            }
            other => panic!("Dag must be a Conj, got {other:?}"),
        };
        dag.set_port_type(parent_port, crate::types::TypeShape::new(dag_type));

        let mut test_node_ids = Vec::new();
        for _ in 0..2 {
            let node_id = dag.alloc_node_id();
            let output = dag.alloc_port(Some(node_id));
            dag.push_node(Behavior::Transform(TransformNode {
                id: node_id,
                target: TransformTarget::FieldProject {
                    field_label: "nodes".to_string(),
                    field_child: Some(dag_nodes_type),
                },
                inputs: vec![parent_port],
                output,
                span: SourceSpan::new("<test>", 0, 0),
            }));
            dag.set_port_type(output, crate::types::TypeShape::new(dag_nodes_type));
            test_node_ids.push(node_id);
        }

        // Query by the specific node we just pushed — earlier Transform
        // nodes in `dag.nodes()` belong to bootstrap-loaded std modules
        // and have no `parent` binding in `bound_names`, which renders
        // as the empty-list fallback rather than the expected projection.
        let first_transform = match dag.node(test_node_ids[0]) {
            Behavior::Transform(t) => t,
            other => panic!("pushed transform went missing, got {other:?}"),
        };
        let indexes = RealizationIndexes::build(&dag).expect("indexes build");
        let input_use_facts = InputUseFacts::build(&dag, &indexes);
        let mut bound_names = HashMap::new();
        bound_names.insert(parent_port, LocalBinding::Owned("parent".to_string()));
        let ctx = Ctx {
            dag: &dag,
            indexes: &indexes,
            bound_names: &bound_names,
            input_use_facts: &input_use_facts,
            mode: EmitRustMode::Program,
        };

        let rendered = ctx
            .render_transform(
                first_transform,
                &RenderLocals::default(),
                RenderMode::OwnedConstruct,
            )
            .expect("field project renders");
        assert_eq!(rendered, "((parent).nodes()).to_vec()");
    }

    #[test]
    fn render_fold_iterates_named_list_input_by_borrow() {
        let dag = compile_to_dag(
            "let total: Int = fold(singleton(1), 0, |acc, x| acc + x)",
            "test.v3",
        )
        .expect("compiles");
        let fold_template = dag.std_list_fold_decl().expect("fold decl");
        let fold_transform = dag
            .nodes()
            .iter()
            .find_map(|node| match node {
                Behavior::Transform(t) => match &t.target {
                    TransformTarget::Callable(target) => {
                        let (template, _) = callable_template(*target, &dag);
                        (template == fold_template).then_some(t)
                    }
                    _ => None,
                },
                _ => None,
            })
            .expect("fold transform");

        let indexes = RealizationIndexes::build(&dag).expect("indexes build");
        let input_use_facts = InputUseFacts::build(&dag, &indexes);
        let mut bound_names = HashMap::new();
        bound_names.insert(
            fold_transform.inputs[0],
            LocalBinding::Owned("xs".to_string()),
        );
        let ctx = Ctx {
            dag: &dag,
            indexes: &indexes,
            bound_names: &bound_names,
            input_use_facts: &input_use_facts,
            mode: EmitRustMode::Program,
        };

        let rendered = ctx
            .render_transform(
                fold_transform,
                &RenderLocals::default(),
                RenderMode::OwnedConstruct,
            )
            .expect("fold renders");
        assert!(
            rendered.contains("(xs).iter().fold("),
            "expected named list inputs to be iterated by borrow, got: {rendered}"
        );
    }

    #[test]
    fn rendering_model_read_strategy_controls_function_parameter_shape() {
        let mut dag = compile_to_dag(
            "type Sign = Plus | Minus
fn classify(s: Sign) -> Int = match s { Plus => 0, Minus => 1 }",
            "test.v3",
        )
        .expect("compiles");
        let rendering_decl = dag.rust_rendering_spec().expect("rust_rendering cached");
        let pass_by_value = named_variant_id(&dag, "ReadStrategy", "PassByValue")
            .expect("ReadStrategy.PassByValue exists");
        let copy_or_clone = named_variant_id(&dag, "ConstructStrategy", "CopyOrClone")
            .expect("ConstructStrategy.CopyOrClone exists");
        dag.declaration_mut(rendering_decl).value_body = Some(ValueBody::Structural {
            fields: vec![
                (
                    "read".to_string(),
                    FieldValue::Variant {
                        constructor: pass_by_value,
                        payload: Vec::new(),
                    },
                ),
                (
                    "construct".to_string(),
                    FieldValue::Variant {
                        constructor: copy_or_clone,
                        payload: Vec::new(),
                    },
                ),
            ],
        });

        let rendered = emit_rust_with_mode(&dag, EmitRustMode::Module).expect("emits");
        assert!(
            rendered.contains("fn classify(p0: Sign) -> i64 {"),
            "expected PassByValue read strategy to render owned function params, got: {rendered}"
        );
    }

    #[test]
    fn module_function_visibility_comes_from_rust_function_syntax() {
        let mut dag =
            compile_to_dag("fn classify(s: Int) -> Int = s", "test.v3").expect("compiles");
        let functions_decl = dag
            .rust_functions_syntax_decl()
            .expect("rust_functions declaration");
        dag.declaration_mut(functions_decl).value_body = Some(ValueBody::Structural {
            fields: vec![
                (
                    "definition".to_string(),
                    FieldValue::Literal(LiteralBits::String(
                        "fn {name}({params}) -> {ret} { {body} }".to_string(),
                    )),
                ),
                (
                    "definition_exported".to_string(),
                    FieldValue::Literal(LiteralBits::String(
                        "pub(crate) fn {name}({params}) -> {ret} { {body} }".to_string(),
                    )),
                ),
                (
                    "definition_void".to_string(),
                    FieldValue::Literal(LiteralBits::String(
                        "fn {name}({params}) { {body} }".to_string(),
                    )),
                ),
                (
                    "param_with_type".to_string(),
                    FieldValue::Literal(LiteralBits::String("{name}: {type}".to_string())),
                ),
                (
                    "param_separator".to_string(),
                    FieldValue::Literal(LiteralBits::String(", ".to_string())),
                ),
            ],
        });

        let rendered = emit_rust_with_mode(&dag, EmitRustMode::Module).expect("emits");
        assert!(
            rendered.contains("pub(crate) fn classify("),
            "expected module-mode function visibility to come from rust_functions.definition_exported, got: {rendered}"
        );
        assert!(
            !rendered.contains("pub fn classify("),
            "expected handwritten `pub ` prefix logic to be gone, got: {rendered}"
        );
    }

    #[test]
    fn clean_emission_rejects_rust_invalid_pattern_binding_variants() {
        let assert_rejected =
            |pick: fn(&crate::dag::PatternBindingRuleVariants) -> Option<DeclarationId>,
             expected_detail: &'static str| {
                let mut dag = compile_to_dag(
                    "type Sign = Plus | Minus
fn classify(s: Sign) -> Int = match s { Plus => 0, Minus => 1 }",
                    "test.v3",
                )
                .expect("compiles");
                let clean_decl = dag
                    .rust_clean_emission_spec()
                    .expect("rust_clean_emission cached");
                let invalid_ctor = pick(dag.pattern_binding_rule_variants())
                    .expect("PatternBindingRule variant cached");
                dag.declaration_mut(clean_decl).value_body = Some(ValueBody::Structural {
                    fields: vec![(
                        "pattern_bindings".to_string(),
                        FieldValue::Variant {
                            constructor: invalid_ctor,
                            payload: Vec::new(),
                        },
                    )],
                });

                let err = emit_rust_with_mode(&dag, EmitRustMode::Module)
                    .expect_err("Rust-invalid pattern binding rule must fail closed");
                assert!(matches!(
                    err,
                    EmitError::MalformedTargetSyntax {
                        declaration,
                        detail,
                    } if declaration == clean_decl && detail == expected_detail
                ));
            };

        assert_rejected(
            |v| v.emit_prefixed,
            "rust_clean_emission.pattern_bindings cannot use PatternBindingRule.EmitPrefixedUnderscoreWhenUnused; Rust only supports EmitBindingAlways or EmitUnderscoreWhenUnused",
        );
        assert_rejected(
            |v| v.not_applicable,
            "rust_clean_emission.pattern_bindings cannot use PatternBindingRule.NotApplicablePatternBinding; Rust only supports EmitBindingAlways or EmitUnderscoreWhenUnused",
        );
    }

    #[test]
    fn callable_disposition_derives_direct_return_as_consumed() {
        let dag = compile_to_dag("fn id(x: Int) -> Int = x", "direct_return.v3").expect("compiles");
        let indexes = RealizationIndexes::build(&dag).expect("indexes build");
        let id_decl = dag.declaration_by_name("id").expect("id decl").id;
        assert_eq!(
            indexes.callable_dispositions.get(&id_decl),
            Some(&vec![ParameterDispositionBinding::Consumed]),
        );
    }

    #[test]
    fn callable_disposition_keeps_match_scrutinee_borrowed() {
        let dag = compile_to_dag(
            "fn head_or_zero(list: List<Int>) -> Int = match list { Empty => 0, Cons(payload) => payload.head }",
            "match_payload.v3",
        )
        .expect("compiles");
        let indexes = RealizationIndexes::build(&dag).expect("indexes build");
        let decl = dag
            .declaration_by_name("head_or_zero")
            .expect("head_or_zero decl")
            .id;
        assert_eq!(
            indexes.callable_dispositions.get(&decl),
            Some(&vec![ParameterDispositionBinding::Borrowed]),
        );
    }

    #[test]
    fn callable_disposition_keeps_nested_lambda_capture_borrowed() {
        let dag = compile_to_dag(
            "fn apply_to_three(f: fn(Int) -> Int) -> Int = f(3)
fn use_callback(base: Int) -> Int = apply_to_three(|x| base + x)",
            "nested_lambda.v3",
        )
        .expect("compiles");
        let indexes = RealizationIndexes::build(&dag).expect("indexes build");
        let decl = dag
            .declaration_by_name("use_callback")
            .expect("use_callback decl")
            .id;
        assert_eq!(
            indexes.callable_dispositions.get(&decl),
            Some(&vec![ParameterDispositionBinding::Borrowed]),
        );
    }

    /// B1 — `require_parameter_dispositions` is fail-closed against
    /// arity, slot duplication, and out-of-range slots, so a spec
    /// CallableRealization can't silently drift from the callable's
    /// declared Arrow input arity.
    #[test]
    fn parameter_dispositions_reject_arity_drift_and_slot_collisions() {
        let dag = compile_to_dag("fn id(x: Int) -> Int = x", "arity_drift.v3").expect("compiles");
        let bogus_decl = dag.declaration_by_name("id").expect("id decl").id;
        let borrowed =
            named_variant_id(&dag, "ParameterDisposition", "Borrowed").expect("Borrowed");
        let consumed =
            named_variant_id(&dag, "ParameterDisposition", "Consumed").expect("Consumed");
        let entry = |slot: i64, ctor: DeclarationId| {
            FieldValue::Record(vec![
                (
                    "slot".to_string(),
                    FieldValue::Literal(LiteralBits::Int(slot)),
                ),
                (
                    "disposition".to_string(),
                    FieldValue::Variant {
                        constructor: ctor,
                        payload: vec![],
                    },
                ),
            ])
        };
        let bind =
            |entries: Vec<FieldValue>| vec![("parameters".to_string(), FieldValue::List(entries))];

        // Arity too low: 1 entry expected, 0 supplied.
        let fields = bind(vec![]);
        assert!(matches!(
            require_parameter_dispositions(&dag, &fields, bogus_decl, 1),
            Err(EmitError::MalformedRealization { .. }),
        ));

        // Arity too high: 1 entry expected, 2 supplied.
        let fields = bind(vec![entry(0, borrowed), entry(1, borrowed)]);
        assert!(matches!(
            require_parameter_dispositions(&dag, &fields, bogus_decl, 1),
            Err(EmitError::MalformedRealization { .. }),
        ));

        // Slot duplication: both entries claim slot 0.
        let fields = bind(vec![entry(0, borrowed), entry(0, consumed)]);
        assert!(matches!(
            require_parameter_dispositions(&dag, &fields, bogus_decl, 2),
            Err(EmitError::MalformedRealization { .. }),
        ));

        // Out-of-range slot: arity is 1 but entry claims slot 5.
        let fields = bind(vec![entry(5, borrowed)]);
        assert!(matches!(
            require_parameter_dispositions(&dag, &fields, bogus_decl, 1),
            Err(EmitError::MalformedRealization { .. }),
        ));

        // Negative slot: rejected before the bound check.
        let fields = bind(vec![entry(-1, borrowed)]);
        assert!(matches!(
            require_parameter_dispositions(&dag, &fields, bogus_decl, 1),
            Err(EmitError::MalformedRealization { .. }),
        ));

        // Well-formed: each slot in [0, arity) exactly once. Returns a
        // Vec of length `expected_arity`, indexed by slot.
        let fields = bind(vec![entry(1, borrowed), entry(0, consumed)]);
        let result = require_parameter_dispositions(&dag, &fields, bogus_decl, 2)
            .expect("well-formed parameters parse");
        assert_eq!(
            result,
            vec![
                ParameterDispositionBinding::Consumed,
                ParameterDispositionBinding::Borrowed,
            ],
        );
    }

    /// B11 (post-refactor) — shared realizations are owned via a
    /// typed `language: DeclarationRef` pointing at the target's
    /// language-spec declaration, NOT a TargetLanguage enum variant.
    /// Each emitter compares the typed reference to its cached
    /// language-spec id at index-build time. A realization whose
    /// surface name is `rust_*` but whose `language` refers to
    /// `go_language` is owned by Go.
    #[test]
    fn target_language_is_typed_reference_to_language_spec() {
        let dag = compile_to_dag("fn id(x: Int) -> Int = x", "lang_ref.v3").expect("compiles");
        let rust_language_id = dag
            .rust_language_spec()
            .expect("rust_language cached after bootstrap");
        let go_language_id = dag
            .go_language_spec()
            .expect("go_language cached after bootstrap");
        // Rust and Go have distinct language-spec declaration ids, so
        // comparing a realization's `language` field to the cached id
        // partitions entries cleanly. This is the structural signal
        // that replaced the TargetLanguage enum roster.
        assert_ne!(rust_language_id, go_language_id);
    }

    /// Boundary check for `PatternRealization.empty_variant` /
    /// `cons_variant`. The substrate types these as `DeclarationRef`,
    /// which is unconstrained — nothing in the grammar rejects
    /// `empty_variant: Int` under `target: List`. `validate_pattern_roles`
    /// enforces "must be a variant of `target`" at parse time. This test
    /// pins the rejection so later refactors can't silently drop it.
    #[test]
    fn validate_pattern_roles_rejects_non_variant_empty_ref() {
        let dag = Dag::new();
        let bool_id = dag
            .bool_shape()
            .expect("Bool is a Disj in bootstrap std")
            .declaration;
        let int_id = dag.int_shape().expect("Int in bootstrap std").declaration;
        // Pick a real Bool variant for `cons_variant` so only
        // `empty_variant` is the illegal pointer.
        let bool_variant_ty = match &dag.declaration(bool_id).connective {
            TypeConnective::Disj { variants } => variants[0].ty,
            other => panic!("Bool should be a Disj, got {other:?}"),
        };
        let binding = PatternRealizationBinding {
            empty_variant: int_id,
            cons_variant: bool_variant_ty,
            scrutinee: "{expr}".into(),
            empty_pattern: "[]".into(),
            cons_pattern: "[{head}, {tail} @ ..]".into(),
            head_expr: "{head}".into(),
            tail_expr: "{tail}".into(),
        };
        let result = validate_pattern_roles(&dag, bool_id, &binding, bool_id);
        match result {
            Err(EmitError::MalformedRealization { detail, .. }) => {
                assert!(
                    detail.contains("empty_variant must be a variant"),
                    "unexpected detail: {detail}"
                );
            }
            other => {
                panic!("expected MalformedRealization for non-variant empty_variant, got {other:?}")
            }
        }
    }

    /// Mirror of the above for `cons_variant`. Distinct test so a future
    /// regression on only one of the two role checks is caught narrowly.
    #[test]
    fn validate_pattern_roles_rejects_non_variant_cons_ref() {
        let dag = Dag::new();
        let bool_id = dag
            .bool_shape()
            .expect("Bool is a Disj in bootstrap std")
            .declaration;
        let int_id = dag.int_shape().expect("Int in bootstrap std").declaration;
        let bool_variant_ty = match &dag.declaration(bool_id).connective {
            TypeConnective::Disj { variants } => variants[0].ty,
            other => panic!("Bool should be a Disj, got {other:?}"),
        };
        let binding = PatternRealizationBinding {
            empty_variant: bool_variant_ty,
            cons_variant: int_id,
            scrutinee: "{expr}".into(),
            empty_pattern: "[]".into(),
            cons_pattern: "[{head}, {tail} @ ..]".into(),
            head_expr: "{head}".into(),
            tail_expr: "{tail}".into(),
        };
        let result = validate_pattern_roles(&dag, bool_id, &binding, bool_id);
        match result {
            Err(EmitError::MalformedRealization { detail, .. }) => {
                assert!(
                    detail.contains("cons_variant must be a variant"),
                    "unexpected detail: {detail}"
                );
            }
            other => {
                panic!("expected MalformedRealization for non-variant cons_variant, got {other:?}")
            }
        }
    }

    /// Distinct-variants check: even if both refs are valid variants of
    /// `target`, they must not be the same one — the branch shape needs
    /// two distinct arms. Pins the third clause of `validate_pattern_roles`.
    #[test]
    fn validate_pattern_roles_rejects_aliased_role_refs() {
        let dag = Dag::new();
        let bool_id = dag
            .bool_shape()
            .expect("Bool is a Disj in bootstrap std")
            .declaration;
        let bool_variant_ty = match &dag.declaration(bool_id).connective {
            TypeConnective::Disj { variants } => variants[0].ty,
            other => panic!("Bool should be a Disj, got {other:?}"),
        };
        let binding = PatternRealizationBinding {
            empty_variant: bool_variant_ty,
            cons_variant: bool_variant_ty,
            scrutinee: "{expr}".into(),
            empty_pattern: "[]".into(),
            cons_pattern: "[{head}, {tail} @ ..]".into(),
            head_expr: "{head}".into(),
            tail_expr: "{tail}".into(),
        };
        let result = validate_pattern_roles(&dag, bool_id, &binding, bool_id);
        match result {
            Err(EmitError::MalformedRealization { detail, .. }) => {
                assert!(
                    detail.contains("must be distinct"),
                    "unexpected detail: {detail}"
                );
            }
            other => panic!("expected MalformedRealization for aliased role refs, got {other:?}"),
        }
    }

    #[test]
    fn named_disj_enum_name_for_rust_match_emit_follows_specialization_parent_to_named_template() {
        use crate::dag::Declaration;

        let mut dag = Dag::new();
        let template_id = dag
            .declarations()
            .iter()
            .find(|d| {
                matches!(&d.connective, TypeConnective::Disj { .. })
                    && d.name.as_deref() == Some("Classical")
            })
            .expect("bootstrap `Classical` sum")
            .id;

        let anon_id = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: anon_id,
            name: None,
            connective: TypeConnective::Disj {
                variants: Vec::new(),
            },
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: Some(template_id),
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("<test>", 0, 0),
        });

        assert_eq!(
            named_disj_enum_name_for_rust_match_emit(&dag, anon_id).as_deref(),
            Some("Classical")
        );
        assert_eq!(
            named_disj_enum_name_for_rust_match_emit(&dag, template_id).as_deref(),
            Some("Classical")
        );
    }
}
