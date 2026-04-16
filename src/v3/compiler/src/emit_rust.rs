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

use std::collections::HashMap;

use crate::dag::{
    ArrowBody, AtomPayload, Behavior, BranchNode, BranchPattern, Dag, DeclarationId, Field,
    FieldValue, LiteralBits, Path, PortId, TemplateArgument, TransformNode, TransformTarget,
    TypeConnective, ValueBody, ValueNode,
};
use crate::operators::OperatorKind;

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
}

/// Typed tag identifying which realization category a
/// `MissingRealizationMeta` error refers to. Three variants, one
/// per meta-type declared in the per-target spec file. Replaces
/// what would otherwise be a string label for the missing role.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RealizationCategory {
    /// `TypeRealization` — primitive type → target type name.
    Type,
    /// `OperatorRealization` — (operand type, algebra field) →
    /// target operator symbol.
    Operator,
    /// `BehaviorRealization` — substrate marker → target template.
    Behavior,
    /// `CallableRealization` — callable declaration → Rust render
    /// strategy.
    Callable,
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
struct TypeRealizationBinding {
    carrier: String,
    fields: HashMap<String, RustFieldAccessBinding>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RustCallableStrategyBinding {
    ListEmpty,
    ListSingleton,
    ListCons,
    ListFold,
    ListMap,
    ListFilter,
}

#[derive(Debug, Clone)]
struct StatementSyntaxBinding {
    let_binding: String,
    let_binding_inferred: String,
}

#[derive(Debug, Clone)]
struct ExpressionSyntaxBinding {
    binary_op: String,
    field_access: String,
    function_call: String,
    closure: String,
}

#[derive(Debug, Clone)]
struct ForEachSyntaxBinding {
    prefix: String,
    separator: String,
}

#[derive(Debug, Clone)]
struct ControlFlowSyntaxBinding {
    if_else: String,
    match_arm: String,
    for_each_syntax: ForEachSyntaxBinding,
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
    definition_void: String,
    param_with_type: String,
    param_separator: String,
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
struct PatternMatchSyntaxBinding {
    match_expr: String,
    match_arm: String,
    variant_pattern: String,
    variant_pattern_positional: String,
    variant_pattern_empty: String,
    field_binding: String,
    field_binding_separator: String,
    wildcard: String,
}

#[derive(Debug, Clone)]
struct CollectionOpsBinding {
    fold: String,
    map: String,
    filter: String,
    empty_list: String,
    list_literal: String,
    cons: String,
}

#[derive(Debug, Clone)]
struct ValueConstructionSyntaxBinding {
    struct_literal: String,
    struct_field_init: String,
    struct_field_separator: String,
    variant_construction: String,
    variant_named_construction: String,
}

#[derive(Debug, Clone)]
struct RustLanguageSyntax {
    statements: StatementSyntaxBinding,
    expressions: ExpressionSyntaxBinding,
    control_flow: ControlFlowSyntaxBinding,
    literals: LiteralSyntaxBinding,
    modules: ModuleSyntaxBinding,
    functions: FunctionSyntaxBinding,
    type_definitions: TypeDefinitionSyntaxBinding,
    patterns: PatternMatchSyntaxBinding,
    collection_ops: CollectionOpsBinding,
    values: ValueConstructionSyntaxBinding,
}

struct RealizationIndexes {
    /// `target_decl_id → carrier + field bindings`. Built from
    /// `data rust_*: TypeRealization` items in rust.dag.
    types: HashMap<DeclarationId, TypeRealizationBinding>,
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
    /// `callable_decl → render strategy`. Built from `data rust_*:
    /// CallableRealization` items in rust.dag. Used when emitting
    /// callable transforms without name-keyed builtin dispatch.
    callables: HashMap<DeclarationId, RustCallableStrategyBinding>,
    /// The Rust target-language syntax bundle loaded from
    /// `data rust_language: LanguageSpec`.
    syntax: RustLanguageSyntax,
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
        let op_meta = dag
            .operator_realization_meta()
            .ok_or(EmitError::MissingRealizationMeta(
                RealizationCategory::Operator,
            ))?;
        let behavior_meta = dag
            .behavior_realization_meta()
            .ok_or(EmitError::MissingRealizationMeta(
                RealizationCategory::Behavior,
            ))?;
        let callable_meta = dag
            .callable_realization_meta()
            .ok_or(EmitError::MissingRealizationMeta(
                RealizationCategory::Callable,
            ))?;

        let mut types: HashMap<DeclarationId, TypeRealizationBinding> = HashMap::new();
        let mut operators: HashMap<(DeclarationId, DeclarationId), String> = HashMap::new();
        let mut behaviors: HashMap<DeclarationId, String> = HashMap::new();
        let mut callables: HashMap<DeclarationId, RustCallableStrategyBinding> = HashMap::new();

        for decl in dag.declarations() {
            let Some(meta_tag) = decl.meta_tag else {
                continue;
            };
            // Determine which realization category this declaration
            // belongs to (if any). Comparing typed handles, no name
            // matching.
            let category = if meta_tag == type_meta {
                RealizationCategory::Type
            } else if meta_tag == op_meta {
                RealizationCategory::Operator
            } else if meta_tag == behavior_meta {
                RealizationCategory::Behavior
            } else if meta_tag == callable_meta {
                RealizationCategory::Callable
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
                RealizationCategory::Operator => {
                    let carrier = require_field_string(fields, "carrier", decl.id)?;
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
                    let strategy = require_callable_strategy(fields, decl.id)?;
                    if callables.insert(target, strategy).is_some() {
                        return Err(EmitError::DuplicateRealization {
                            declaration: decl.id,
                            detail:
                                "two CallableRealization data items target the same callable declaration — single authority requires unique targets",
                        });
                    }
                }
            }
        }

        let syntax = RustLanguageSyntax::build(dag)?;

        Ok(Self {
            types,
            operators,
            behaviors,
            callables,
            syntax,
        })
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
            type_definitions: parse_type_definition_syntax(
                dag,
                require_field_decl_ref(fields, "type_definitions", language_decl)?,
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

fn structural_fields_for_decl<'a>(
    dag: &'a Dag,
    declaration: DeclarationId,
) -> Result<&'a [(String, FieldValue)], EmitError> {
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

fn syntax_field_record<'a>(
    fields: &'a [(String, FieldValue)],
    label: &str,
    declaration: DeclarationId,
) -> Result<&'a [(String, FieldValue)], EmitError> {
    fields
        .iter()
        .find(|(name, _)| name == label)
        .and_then(|(_, value)| match value {
            FieldValue::Record(fields) => Some(fields.as_slice()),
            _ => None,
        })
        .ok_or(EmitError::MalformedTargetSyntax {
            declaration,
            detail: "syntax field must be a structural record",
        })
}

fn parse_statement_syntax(
    dag: &Dag,
    declaration: DeclarationId,
) -> Result<StatementSyntaxBinding, EmitError> {
    let fields = structural_fields_for_decl(dag, declaration)?;
    Ok(StatementSyntaxBinding {
        let_binding: syntax_field_string(fields, "let_binding", declaration)?,
        let_binding_inferred: syntax_field_string(fields, "let_binding_inferred", declaration)?,
    })
}

fn parse_expression_syntax(
    dag: &Dag,
    declaration: DeclarationId,
) -> Result<ExpressionSyntaxBinding, EmitError> {
    let fields = structural_fields_for_decl(dag, declaration)?;
    Ok(ExpressionSyntaxBinding {
        binary_op: syntax_field_string(fields, "binary_op", declaration)?,
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
    let for_each = syntax_field_record(fields, "for_each_syntax", declaration)?;
    Ok(ControlFlowSyntaxBinding {
        if_else: syntax_field_string(fields, "if_else", declaration)?,
        match_arm: syntax_field_string(fields, "match_arm", declaration)?,
        for_each_syntax: ForEachSyntaxBinding {
            prefix: syntax_field_string(for_each, "prefix", declaration)?,
            separator: syntax_field_string(for_each, "separator", declaration)?,
        },
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
        definition_void: syntax_field_string(fields, "definition_void", declaration)?,
        param_with_type: syntax_field_string(fields, "param_with_type", declaration)?,
        param_separator: syntax_field_string(fields, "param_separator", declaration)?,
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
    Ok(CollectionOpsBinding {
        fold: syntax_field_string(fields, "fold", declaration)?,
        map: syntax_field_string(fields, "map", declaration)?,
        filter: syntax_field_string(fields, "filter", declaration)?,
        empty_list: syntax_field_string(fields, "empty_list", declaration)?,
        list_literal: syntax_field_string(fields, "list_literal", declaration)?,
        cons: syntax_field_string(fields, "cons", declaration)?,
    })
}

fn parse_value_construction_syntax(
    dag: &Dag,
    declaration: DeclarationId,
) -> Result<ValueConstructionSyntaxBinding, EmitError> {
    let fields = structural_fields_for_decl(dag, declaration)?;
    Ok(ValueConstructionSyntaxBinding {
        struct_literal: syntax_field_string(fields, "struct_literal", declaration)?,
        struct_field_init: syntax_field_string(fields, "struct_field_init", declaration)?,
        struct_field_separator: syntax_field_string(
            fields,
            "struct_field_separator",
            declaration,
        )?,
        variant_construction: syntax_field_string(fields, "variant_construction", declaration)?,
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

fn require_field_bindings(
    dag: &Dag,
    fields: &[(String, FieldValue)],
    declaration: DeclarationId,
) -> Result<HashMap<String, RustFieldAccessBinding>, EmitError> {
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
            .find(|(label, _)| label == "rust_access")
            .ok_or(EmitError::MalformedRealization {
                declaration,
                detail: "FieldBinding.rust_access is required",
            })
            .and_then(|(_, value)| parse_rust_field_access(dag, value, declaration))?;
        if bindings.insert(dag_name, rust_access).is_some() {
            return Err(EmitError::DuplicateRealization {
                declaration,
                detail: "TypeRealization.fields contains duplicate dag_name entries",
            });
        }
    }
    Ok(bindings)
}

fn parse_rust_field_access(
    _dag: &Dag,
    value: &FieldValue,
    declaration: DeclarationId,
) -> Result<RustFieldAccessBinding, EmitError> {
    let FieldValue::Variant {
        constructor_name,
        payload,
    } = value
    else {
        return Err(EmitError::MalformedRealization {
            declaration,
            detail: "FieldBinding.rust_access must be a RustFieldAccess variant",
        });
    };
    if payload.len() != 1 {
        return Err(EmitError::MalformedRealization {
            declaration,
            detail: "RustFieldAccess variants must carry exactly one String payload",
        });
    }
    let name = match &payload[0] {
        FieldValue::Literal(LiteralBits::String(name)) => name.clone(),
        _ => {
            return Err(EmitError::MalformedRealization {
                declaration,
                detail: "RustFieldAccess payload must be a String literal",
            });
        }
    };
    match constructor_name.as_str() {
        "DirectField" => Ok(RustFieldAccessBinding::DirectField(name)),
        "AccessorMethod" => Ok(RustFieldAccessBinding::AccessorMethod(name)),
        _ => Err(EmitError::MalformedRealization {
            declaration,
            detail: "RustFieldAccess constructor must be DirectField or AccessorMethod",
        }),
    }
}

fn require_callable_strategy(
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
        constructor_name,
        payload,
    } = value
    else {
        return Err(EmitError::MalformedRealization {
            declaration,
            detail: "CallableRealization.strategy must be a RustCallableStrategy variant",
        });
    };
    if !payload.is_empty() {
        return Err(EmitError::MalformedRealization {
            declaration,
            detail: "RustCallableStrategy variants must not carry payload fields",
        });
    }
    match constructor_name.as_str() {
        "ListEmpty" => Ok(RustCallableStrategyBinding::ListEmpty),
        "ListSingleton" => Ok(RustCallableStrategyBinding::ListSingleton),
        "ListCons" => Ok(RustCallableStrategyBinding::ListCons),
        "ListFold" => Ok(RustCallableStrategyBinding::ListFold),
        "ListMap" => Ok(RustCallableStrategyBinding::ListMap),
        "ListFilter" => Ok(RustCallableStrategyBinding::ListFilter),
        _ => Err(EmitError::MalformedRealization {
            declaration,
            detail:
                "RustCallableStrategy constructor must be ListEmpty/ListSingleton/ListCons/ListFold/ListMap/ListFilter",
        }),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EmitRustMode {
    Program,
    Module,
}

fn emit_rust_with_mode(dag: &Dag, mode: EmitRustMode) -> Result<String, EmitError> {
    let indexes = RealizationIndexes::build(dag)?;

    // Resolve the substrate markers we need ONCE up front. Each
    // marker is a typed `DeclarationId` cached at bootstrap end
    // from `dsl/std/v3_l1.dag`; if any is missing, the file
    // failed to load and emit can't proceed. Rendering downstream
    // uses the bound handles, never a name string.
    let main_marker = dag
        .main_marker()
        .ok_or(EmitError::MissingSubstrateMarker(SubstrateMarkerRole::Main))?;
    let type_decls: Vec<_> = dag
        .declarations()
        .iter()
        .filter(|decl| !is_bootstrap_file(&decl.span.file))
        .filter(|decl| decl.name.is_some())
        .filter(|decl| matches!(
            decl.connective,
            TypeConnective::Conj { .. } | TypeConnective::Disj { .. }
        ))
        .collect();
    let function_decls: Vec<_> = dag
        .declarations()
        .iter()
        .filter(|decl| !is_bootstrap_file(&decl.span.file))
        .filter(|decl| decl.name.is_some())
        .filter(|decl| !decl.name.as_deref().is_some_and(|name| name.starts_with("__anon_lambda_")))
        .filter(|decl| matches!(
            decl.connective,
            TypeConnective::Arrow {
                body: ArrowBody::UserDefined(_),
                ..
            }
        ))
        .collect();
    let top_level_binds: Vec<&crate::dag::BindNode> = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .filter(|b| b.params.is_empty())
        .collect();

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
    let mut bound_names: HashMap<PortId, String> = HashMap::new();
    for bind in &top_level_binds {
        bound_names.insert(bind.value, bind.name.clone());
    }

    let ctx = Ctx {
        dag,
        indexes: &indexes,
        bound_names: &bound_names,
    };

    let rendered_types: Vec<String> = type_decls
        .iter()
        .map(|decl| ctx.render_type_declaration(decl))
        .collect::<Result<Vec<_>, _>>()?;
    let rendered_functions: Vec<String> = function_decls
        .iter()
        .map(|decl| ctx.render_function_declaration(decl))
        .collect::<Result<Vec<_>, _>>()?;

    let type_defs = join_rendered(&rendered_types, " ");
    let function_defs = join_rendered(&rendered_functions, " ");
    let mut sections = Vec::new();
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
                &[("name", &bind.name), ("type", &ty_name), ("value", &value_expr)],
            );
            rendered_binds.push(rendered);
        }

        let body_joined = join_rendered(&rendered_binds, " ");
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
        let main_program = render_named_template(
            main_template,
            &[
                ("body", &body_joined),
                ("final", &final_bind_name),
                ("quote", &indexes.syntax.literals.string_delimiter),
            ],
        );
        sections.push(main_program);
    }
    Ok(join_rendered(&sections, " "))
}

pub fn emit_rust(dag: &Dag) -> Result<String, EmitError> {
    emit_rust_with_mode(dag, EmitRustMode::Program)
}

pub fn emit_rust_module(dag: &Dag) -> Result<String, EmitError> {
    emit_rust_with_mode(dag, EmitRustMode::Module)
}

/// Bundled emission context. Carries the typed indexes, substrate
/// marker handles, and bound-name index through the recursive
/// render walk. Replaces the pre-unwind multi-arg threading where
/// every helper took `dag, index, bound_names, ...` separately.
///
struct Ctx<'a> {
    dag: &'a Dag,
    indexes: &'a RealizationIndexes,
    bound_names: &'a HashMap<PortId, String>,
}

impl<'a> Ctx<'a> {
    fn render_port_with_locals(
        &self,
        port: PortId,
        locals: Option<&HashMap<PortId, String>>,
    ) -> Result<String, EmitError> {
        if let Some(locals) = locals {
            if let Some(name) = locals.get(&port) {
                return Ok(name.clone());
            }
        }
        if let Some(name) = self.bound_names.get(&port) {
            return Ok(name.clone());
        }
        self.dispatch_producer_with_locals(port, locals)
    }

    /// Render the value for a top-level let binding. Bypasses
    /// `bound_names` for `port` itself (otherwise every let would
    /// render as `let x: i64 = x;`); recursive sub-walks still use
    /// `render_port` and DO consult `bound_names`.
    fn render_top_level_value(&self, port: PortId) -> Result<String, EmitError> {
        self.dispatch_producer_with_locals(port, None)
    }

    fn dispatch_producer_with_locals(
        &self,
        port: PortId,
        locals: Option<&HashMap<PortId, String>>,
    ) -> Result<String, EmitError> {
        let Some(node_id) = self.dag.port(port).produced_by else {
            return Err(EmitError::UnsupportedBehavior(
                "render reached a port with no producer (parameter?)".to_string(),
            ));
        };
        match self.dag.node(node_id) {
            Behavior::Value(v) => Ok(render_value(v, &self.indexes.syntax.literals)),
            Behavior::Transform(t) => self.render_transform(t, locals),
            Behavior::Branch(b) => self.render_branch(b, locals),
            Behavior::Loop(l) => self.render_loop(l, locals),
            Behavior::Bind(b) => Ok(b.name.clone()),
        }
    }

    fn render_transform(
        &self,
        t: &TransformNode,
        locals: Option<&HashMap<PortId, String>>,
    ) -> Result<String, EmitError> {
        match &t.target {
            TransformTarget::Operator(op) => self.render_operator(t, *op, locals),
            TransformTarget::FieldProject {
                field_label,
                field_child,
            } => {
                self.render_field_project(t, field_label, locals, *field_child)
            }
            TransformTarget::Callable(target) => {
                self.render_callable_transform(t, *target, locals)
            }
        }
    }

    fn render_field_project(
        &self,
        t: &TransformNode,
        field_label: &str,
        locals: Option<&HashMap<PortId, String>>,
        field_child: Option<DeclarationId>,
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
        let parent_expr = self.render_port_with_locals(t.inputs[0], locals)?;
        let parent_type_id = primitive_type_id_for_port(self.dag, t.inputs[0])?;
        if let Some(type_binding) = self.indexes.types.get(&parent_type_id) {
            let access = type_binding
                .fields
                .get(field_label)
                .ok_or_else(|| {
                    EmitError::UnsupportedBehavior(format!(
                        "field projection .{field_label} has no FieldBinding entry on the parent TypeRealization"
                    ))
                })?;
            return match access {
                RustFieldAccessBinding::DirectField(name) => Ok(render_named_template(
                    &self.indexes.syntax.expressions.field_access,
                    &[("object", &parent_expr), ("field", name)],
                )),
                RustFieldAccessBinding::AccessorMethod(name) => Ok(format!(
                    "{}()",
                    render_named_template(
                        &self.indexes.syntax.expressions.field_access,
                        &[("object", &parent_expr), ("field", name)],
                    )
                )),
            };
        }
        let Some(conj_id) = walk_to_conj(self.dag, parent_type_id) else {
            return Err(EmitError::MissingTypeRealization {
                target: parent_type_id,
            });
        };
        if !matches!(self.dag.declaration(conj_id).connective, TypeConnective::Conj { .. }) {
            return Err(EmitError::MissingTypeRealization {
                target: parent_type_id,
            });
        }
        Ok(render_named_template(
            &self.indexes.syntax.expressions.field_access,
            &[("object", &parent_expr), ("field", field_label)],
        ))
    }

    fn render_operator(
        &self,
        t: &TransformNode,
        op: OperatorKind,
        locals: Option<&HashMap<PortId, String>>,
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
        let lhs = self.render_port_with_locals(t.inputs[0], locals)?;
        let rhs = self.render_port_with_locals(t.inputs[1], locals)?;
        Ok(render_named_template(
            &self.indexes.syntax.expressions.binary_op,
            &[("lhs", &lhs), ("op", &carrier), ("rhs", &rhs)],
        ))
    }

    fn render_branch(
        &self,
        b: &BranchNode,
        locals: Option<&HashMap<PortId, String>>,
    ) -> Result<String, EmitError> {
        if self.branch_scrutinee_is_bool(b)? {
            let (then_path, else_path) = self.split_bool_paths(b)?;
            let cond = self.render_port_with_locals(b.input, locals)?;
            let then_expr = self.render_path_body(then_path, locals)?;
            let else_expr = self.render_path_body(else_path, locals)?;
            return Ok(render_named_template(
                &self.indexes.syntax.control_flow.if_else,
                &[("cond", &cond), ("then", &then_expr), ("else", &else_expr)],
            ));
        }

        let expr = self.render_port_with_locals(b.input, locals)?;
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

    fn render_path_body(
        &self,
        path: &Path,
        locals: Option<&HashMap<PortId, String>>,
    ) -> Result<String, EmitError> {
        let mut arm_locals = locals.cloned().unwrap_or_default();
        if let Some(binding) = &path.binding {
            arm_locals.insert(binding.payload_port, binding.binding_name.clone());
        }
        if arm_locals.is_empty() {
            self.render_port_with_locals(path.output, None)
        } else {
            self.render_port_with_locals(path.output, Some(&arm_locals))
        }
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
        let enum_name = self
            .dag
            .declaration(disj_id)
            .name
            .clone()
            .ok_or(EmitError::UnsupportedBehavior(
                "match on anonymous sum declarations is not yet supported in Rust emission"
                    .to_string(),
            ))?;
        let qualified_name = self.qualified_name(&enum_name, &variant_name);
        let Some(binding) = &path.binding else {
            return Ok(render_named_template(
                &self.indexes.syntax.patterns.variant_pattern_empty,
                &[("name", &qualified_name)],
            ));
        };
        let TypeConnective::Conj { children } = &self.dag.declaration(resolved_id).connective else {
            return Err(EmitError::UnsupportedBehavior(format!(
                "matched variant `{variant_name}` does not lower to a payload product"
            )));
        };
        if children.len() != 1 {
            return Err(EmitError::UnsupportedBehavior(format!(
                "matched variant `{variant_name}` must carry exactly one payload field for direct binding"
            )));
        }
        if children[0].label == "_0" && self.indexes.types.contains_key(&disj_id) {
            return Ok(render_named_template(
                &self.indexes.syntax.patterns.variant_pattern_positional,
                &[("name", &qualified_name), ("binding", &binding.binding_name)],
            ));
        }
        let bindings = render_named_template(
            &self.indexes.syntax.patterns.field_binding,
            &[("field", &children[0].label), ("binding", &binding.binding_name)],
        );
        Ok(render_named_template(
            &self.indexes.syntax.patterns.variant_pattern,
            &[("name", &qualified_name), ("bindings", &bindings)],
        ))
    }

    fn render_bool_pattern(
        &self,
        disj_id: DeclarationId,
        variant_id: DeclarationId,
    ) -> Result<String, EmitError> {
        let TypeConnective::Disj { variants } = &self.dag.declaration(disj_id).connective else {
            unreachable!("walk_to_disj returned non-Disj")
        };
        let Some((idx, _)) = variants.iter().enumerate().find(|(_, variant)| variant.ty == variant_id) else {
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

    fn render_callable_transform(
        &self,
        t: &TransformNode,
        target: DeclarationId,
        locals: Option<&HashMap<PortId, String>>,
    ) -> Result<String, EmitError> {
        let (template, arguments) = callable_template(target, self.dag);
        if let Some(strategy) = self.indexes.callables.get(&template) {
            return self.render_realized_callable(
                template,
                *strategy,
                &arguments,
                &t.inputs,
                locals,
            );
        }
        self.render_general_callable(template, &t.inputs, locals)
    }

    fn render_realized_callable(
        &self,
        template: DeclarationId,
        strategy: RustCallableStrategyBinding,
        arguments: &[TemplateArgument],
        inputs: &[PortId],
        locals: Option<&HashMap<PortId, String>>,
    ) -> Result<String, EmitError> {
        match strategy {
            RustCallableStrategyBinding::ListEmpty => {
                Ok(self.indexes.syntax.collection_ops.empty_list.clone())
            }
            RustCallableStrategyBinding::ListSingleton => {
                if inputs.len() != 1 {
                    return Err(EmitError::UnsupportedBehavior(format!(
                        "singleton arity {} is not supported; expected one runtime input",
                        inputs.len()
                    )));
                }
                let value = self.render_port_with_locals(inputs[0], locals)?;
                Ok(render_named_template(
                    &self.indexes.syntax.collection_ops.list_literal,
                    &[("elements", &value)],
                ))
            }
            RustCallableStrategyBinding::ListCons => {
                if inputs.len() != 2 {
                    return Err(EmitError::UnsupportedBehavior(format!(
                        "cons arity {} is not supported; expected two runtime inputs",
                        inputs.len()
                    )));
                }
                let head = self.render_port_with_locals(inputs[0], locals)?;
                let tail = self.render_port_with_locals(inputs[1], locals)?;
                Ok(render_named_template(
                    &self.indexes.syntax.collection_ops.cons,
                    &[("head", &head), ("tail", &tail)],
                ))
            }
            RustCallableStrategyBinding::ListFold => {
                if inputs.len() != 2 {
                    return Err(EmitError::UnsupportedBehavior(format!(
                        "fold runtime arity {} is not supported; expected [list, init]",
                        inputs.len()
                    )));
                }
                let fn_decl = bound_callable_argument(self.dag, template, arguments, 2)?;
                let acc = "__fold_acc".to_string();
                let item = "__fold_item".to_string();
                let body = self.render_closure(fn_decl, &[acc.clone(), item.clone()])?;
                let list = self.render_port_with_locals(inputs[0], locals)?;
                let init = self.render_port_with_locals(inputs[1], locals)?;
                Ok(render_named_template(
                    &self.indexes.syntax.collection_ops.fold,
                    &[("recv", &list), ("init", &init), ("body", &body)],
                ))
            }
            RustCallableStrategyBinding::ListMap => {
                if inputs.len() != 1 {
                    return Err(EmitError::UnsupportedBehavior(format!(
                        "map runtime arity {} is not supported; expected [list]",
                        inputs.len()
                    )));
                }
                let fn_decl = bound_callable_argument(self.dag, template, arguments, 1)?;
                let item = "__map_item".to_string();
                let body = self.render_closure(fn_decl, std::slice::from_ref(&item))?;
                let list = self.render_port_with_locals(inputs[0], locals)?;
                Ok(render_named_template(
                    &self.indexes.syntax.collection_ops.map,
                    &[("recv", &list), ("body", &body)],
                ))
            }
            RustCallableStrategyBinding::ListFilter => {
                if inputs.len() != 1 {
                    return Err(EmitError::UnsupportedBehavior(format!(
                        "filter runtime arity {} is not supported; expected [list]",
                        inputs.len()
                    )));
                }
                let fn_decl = bound_callable_argument(self.dag, template, arguments, 1)?;
                let item = "__filter_item".to_string();
                let predicate = self.render_callable_body(fn_decl, std::slice::from_ref(&item))?;
                let list = self.render_port_with_locals(inputs[0], locals)?;
                Ok(render_named_template(
                    &self.indexes.syntax.collection_ops.filter,
                    &[("recv", &list), ("item", &item), ("predicate", &predicate)],
                ))
            }
        }
    }

    fn render_general_callable(
        &self,
        template: DeclarationId,
        inputs: &[PortId],
        locals: Option<&HashMap<PortId, String>>,
    ) -> Result<String, EmitError> {
        if let Some(rendered) = self.render_variant_constructor(template, inputs, locals)? {
            return Ok(rendered);
        }
        if let Some(rendered) = self.render_record_constructor(template, inputs, locals)? {
            return Ok(rendered);
        }
        let func = self
            .dag
            .declaration(template)
            .name
            .clone()
            .ok_or(EmitError::UnsupportedBehavior(
                "callable target is anonymous and cannot be rendered as a direct Rust call"
                    .to_string(),
            ))?;
        let args = inputs
            .iter()
            .map(|port| self.render_port_with_locals(*port, locals))
            .collect::<Result<Vec<_>, _>>()?;
        let joined = join_rendered(&args, ", ");
        Ok(render_named_template(
            &self.indexes.syntax.expressions.function_call,
            &[("func", &func), ("args", &joined)],
        ))
    }

    fn render_record_constructor(
        &self,
        template: DeclarationId,
        inputs: &[PortId],
        locals: Option<&HashMap<PortId, String>>,
    ) -> Result<Option<String>, EmitError> {
        let decl = self.dag.declaration(template);
        let Some(type_name) = &decl.name else {
            return Ok(None);
        };
        let TypeConnective::Conj { children } = &decl.connective else {
            return Ok(None);
        };
        if children.len() != inputs.len() {
            return Err(EmitError::UnsupportedBehavior(format!(
                "record constructor `{type_name}` expected {} field input(s), got {}",
                children.len(),
                inputs.len()
            )));
        }
        let fields = children
            .iter()
            .zip(inputs.iter())
            .map(|(field, input)| {
                let value = self.render_port_with_locals(*input, locals)?;
                Ok(render_named_template(
                    &self.indexes.syntax.values.struct_field_init,
                    &[("name", &field.label), ("value", &value)],
                ))
            })
            .collect::<Result<Vec<_>, EmitError>>()?;
        let joined = join_rendered(
            &fields,
            &self.indexes.syntax.values.struct_field_separator,
        );
        Ok(Some(render_named_template(
            &self.indexes.syntax.values.struct_literal,
            &[("type", type_name), ("fields", &joined)],
        )))
    }

    fn render_variant_constructor(
        &self,
        template: DeclarationId,
        inputs: &[PortId],
        locals: Option<&HashMap<PortId, String>>,
    ) -> Result<Option<String>, EmitError> {
        let Some((enum_name, variant_name)) = variant_parent_info(self.dag, template) else {
            return Ok(None);
        };
        let qualified_name = self.qualified_name(&enum_name, &variant_name);
        let TypeConnective::Conj { children } = &self.dag.declaration(template).connective else {
            return Ok(None);
        };
        if children.len() != inputs.len() {
            return Err(EmitError::UnsupportedBehavior(format!(
                "variant constructor `{qualified_name}` expected {} payload field(s), got {}",
                children.len(),
                inputs.len()
            )));
        }
        if children.is_empty() {
            return Ok(Some(qualified_name));
        }
        let fields = children
            .iter()
            .zip(inputs.iter())
            .map(|(field, input)| {
                let value = self.render_port_with_locals(*input, locals)?;
                Ok(render_named_template(
                    &self.indexes.syntax.values.struct_field_init,
                    &[("name", &field.label), ("value", &value)],
                ))
            })
            .collect::<Result<Vec<_>, EmitError>>()?;
        let joined = join_rendered(
            &fields,
            &self.indexes.syntax.values.struct_field_separator,
        );
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
        param_names: &[String],
    ) -> Result<String, EmitError> {
        let TypeConnective::Arrow { inputs, body, .. } = &self.dag.declaration(callable_decl).connective else {
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
        let bind = self
            .dag
            .node(*bind_id)
            .as_bind()
            .expect("UserDefined arrow body must point at a Bind");
        if bind.params.len() != inputs.len() || bind.params.len() != param_names.len() {
            return Err(EmitError::UnsupportedBehavior(
                "capturing callables are not yet supported in staged std.list emission"
                    .to_string(),
            ));
        }
        let locals: HashMap<PortId, String> = bind
            .params
            .iter()
            .copied()
            .zip(param_names.iter().cloned())
            .collect();
        self.render_port_with_locals(bind.value, Some(&locals))
    }

    fn render_closure(
        &self,
        callable_decl: DeclarationId,
        param_names: &[String],
    ) -> Result<String, EmitError> {
        let body = self.render_callable_body(callable_decl, param_names)?;
        let joined = join_rendered(param_names, ", ");
        Ok(render_named_template(
            &self.indexes.syntax.expressions.closure,
            &[("params", &joined), ("body", &body)],
        ))
    }

    fn render_loop(
        &self,
        l: &crate::dag::LoopNode,
        locals: Option<&HashMap<PortId, String>>,
    ) -> Result<String, EmitError> {
        let body_port = behavior_result_port(self.dag.node(l.body));
        self.render_port_with_locals(body_port, locals)
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
        let TypeConnective::Arrow { inputs, output, body } = &declaration.connective else {
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
        let bind = self
            .dag
            .node(*bind_id)
            .as_bind()
            .expect("UserDefined arrow body must point at a Bind");
        let mut locals = HashMap::new();
        let params = bind
            .params
            .iter()
            .enumerate()
            .map(|(idx, port)| {
                let param_name = format!("p{idx}");
                locals.insert(*port, param_name.clone());
                let ty = self.rust_type_name_for_port(*port)?;
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
            .or_else(|| {
                self.rust_type_name_for_port(bind.value).ok()
            })
            .ok_or(EmitError::MissingTypeRealization { target: *output })?;
        let body = self.render_port_with_locals(bind.value, Some(&locals))?;
        Ok(render_named_template(
            &self.indexes.syntax.functions.definition,
            &[("name", name), ("params", &params_joined), ("ret", &ret), ("body", &body)],
        ))
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
                let fields = children
                    .iter()
                    .map(|field| self.render_struct_field(field))
                    .collect::<Result<Vec<_>, _>>()?;
                let fields_joined = join_rendered(&fields, " ");
                Ok(render_named_template(
                    &self.indexes.syntax.type_definitions.struct_def,
                    &[("name", name), ("fields", &fields_joined)],
                ))
            }
            TypeConnective::Disj { variants } => {
                let rendered_variants = variants
                    .iter()
                    .map(|variant| self.render_enum_variant(variant))
                    .collect::<Result<Vec<_>, _>>()?;
                let variants_joined = join_rendered(&rendered_variants, " ");
                Ok(render_named_template(
                    &self.indexes.syntax.type_definitions.enum_def,
                    &[("name", name), ("variants", &variants_joined)],
                ))
            }
            _ => Err(EmitError::UnsupportedBehavior(format!(
                "type declaration `{name}` does not lower to a record or sum shape"
            ))),
        }
    }

    fn render_struct_field(&self, field: &Field) -> Result<String, EmitError> {
        let ty = self.rust_type_name_for_decl(field.ty)?;
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
            .map(|field| self.render_struct_field(field))
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
        self.rust_type_name_for_decl(primitive_id)
    }

    fn rust_type_name_for_decl(&self, declaration: DeclarationId) -> Result<String, EmitError> {
        self.indexes
            .types
            .get(&declaration)
            .map(|binding| binding.carrier.clone())
            .or_else(|| self.dag.declaration(declaration).name.clone())
            .ok_or(EmitError::MissingTypeRealization {
                target: declaration,
            })
    }
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

fn render_value(v: &ValueNode, literals: &LiteralSyntaxBinding) -> String {
    match &v.data {
        LiteralBits::Int(n) => n.to_string(),
        LiteralBits::Bool(true) => literals.true_keyword.clone(),
        LiteralBits::Bool(false) => literals.false_keyword.clone(),
        LiteralBits::String(s) => format!(
            "{}{}{}",
            literals.string_delimiter,
            s.replace('\\', "\\\\").replace('"', "\\\""),
            literals.string_delimiter
        ),
    }
}

fn behavior_result_port(behavior: &Behavior) -> PortId {
    match behavior {
        Behavior::Value(v) => v.result_port(),
        Behavior::Transform(t) => t.result_port(),
        Behavior::Branch(b) => b.result_port(),
        Behavior::Loop(l) => l.result_port(),
        Behavior::Bind(b) => b.result_port(),
    }
}

fn is_bootstrap_file(file: &str) -> bool {
    file.starts_with("dsl/std/") || file.starts_with("src/v3/std/") || file.starts_with("src/v3/spec/")
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

fn walk_to_conj(dag: &Dag, start: DeclarationId) -> Option<DeclarationId> {
    let mut current = start;
    for _ in 0..32 {
        let decl = dag.declaration(current);
        match &decl.connective {
            TypeConnective::Conj { .. } => return Some(current),
            TypeConnective::Instantiation { template, .. } => current = *template,
            TypeConnective::Atom(AtomPayload::ResolvedIdentifier(next)) => current = *next,
            _ => return None,
        }
    }
    None
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::SourceSpan;
    use crate::compile_to_dag;

    #[test]
    fn render_field_project_emits_parent_dot_field() {
        let mut dag = Dag::new();
        let parent_port = dag.alloc_port(None);
        let dag_type = dag
            .declaration_by_name("Dag")
            .expect("Dag type realization target exists")
            .id;
        let dag_nodes_type = match &dag.declaration(dag_type).connective {
            TypeConnective::Conj { children } => children
                .iter()
                .find(|field| field.label == "nodes")
                .expect("Dag.nodes field")
                .ty,
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

        let indexes = RealizationIndexes::build(&dag).expect("indexes build");
        let mut bound_names = HashMap::new();
        bound_names.insert(parent_port, "parent".to_string());
        let ctx = Ctx {
            dag: &dag,
            indexes: &indexes,
            bound_names: &bound_names,
        };

        let rendered = match dag.node(node_id) {
            Behavior::Transform(t) => ctx
                .render_transform(t, None)
                .expect("field project renders"),
            other => panic!("expected Transform node, got {other:?}"),
        };
        assert_eq!(rendered, "parent.nodes()");
    }

    #[test]
    fn render_fold_clones_named_list_input_before_iteration() {
        let dag = compile_to_dag(
            "let total: Int = fold(singleton(1), 0, |acc, x| acc + x)",
            "test.v3",
        )
        .expect("compiles");
        let fold_template = dag.declaration_by_name("fold").expect("fold decl").id;
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
        let mut bound_names = HashMap::new();
        bound_names.insert(fold_transform.inputs[0], "xs".to_string());
        let ctx = Ctx {
            dag: &dag,
            indexes: &indexes,
            bound_names: &bound_names,
        };

        let rendered = ctx
            .render_transform(fold_transform, None)
            .expect("fold renders");
        assert!(
            rendered.contains("(xs).clone().into_iter().fold("),
            "expected named list inputs to be cloned before iteration, got: {rendered}"
        );
    }
}
