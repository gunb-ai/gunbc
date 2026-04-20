//! Shared emit entrypoint and Stage 1e scaffolding.
//!
//! **Invariant D-1 (determinism, DB-8):** for fixed inputs `(dag, target)`,
//! successive calls to [`emit`] / [`emit_module`] must produce **byte-identical**
//! text. Mechanical ratchet: `tests/determinism_test.rs` (5× re-emit per matrix
//! row). Violations include unstable map/set iteration, timestamps or `file!()` /
//! `line!()` in emitted source, and path-dependent emission. Single authority for
//! this claim is the emit pipeline plus those tests (`feedback_substrate_principle_audit` Q5).
//!
//! `emit.rs` is the single dispatch surface for all targets. Each
//! `*_target.rs` sibling still contains one target-monolithic
//! implementation body; the behavior-by-behavior lifts planned in α
//! §10 Stages 1e.2–1e.4 will move logic out of those files into
//! generic walker helpers here. Until that dissolution lands,
//! target-private carriers stay inside their target module and no
//! cross-target code should read them.

pub(crate) mod python_target;
pub(crate) mod rust_target;

use std::collections::{HashMap, HashSet};

use self::python_target::EmitPythonError;
use self::rust_target::{EmitError, RealizationCategory, SubstrateMarkerRole};
use crate::dag::{
    ArrowBody, AtomPayload, Behavior, BranchNode, BranchPattern, CardinalityBound, DeclarationId,
    Field, FieldValue, LiteralBits, Path, PortId, TemplateArgument, TransformNode, TransformTarget,
    TypeConnective, ValueBody,
};
use crate::operators::OperatorKind;
use crate::Dag;

/// Shared emitter-side classification for a resolved variant payload.
/// Distinguishes the two payload forms that affect field projection
/// lowering:
///
/// - positional single-field payloads (`Variant(T)`) bind directly to
///   the carried value
/// - named payload fields (`Variant { x: T, ... }`) require either a
///   whole-payload carrier expression or per-field overrides,
///   depending on the target's spec rule
#[derive(Debug, Clone, PartialEq, Eq)]
enum VariantPayloadShape {
    Empty,
    PositionalSingle,
    NamedFields(Vec<String>),
}

/// Shared emitter-side mirror of
/// `std.clean_emission.VariantPayloadFieldAccessRule`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VariantPayloadFieldAccessRuleBinding {
    AccessFromPayloadBinding,
    OverrideNamedFieldsAtBindingSite,
}

/// Per-payload-port rendering authority used by the emitters.
/// `Direct` means the payload port itself renders to one expression;
/// `Fields` means the whole payload value is not renderable directly,
/// so downstream field projections must be answered by the provided
/// per-field bindings.
#[derive(Debug, Clone)]
enum VariantPayloadBinding<T> {
    Direct(T),
    Fields(HashMap<String, T>),
}

impl<T> VariantPayloadBinding<T> {
    fn direct(&self) -> Option<&T> {
        match self {
            Self::Direct(value) => Some(value),
            Self::Fields(_) => None,
        }
    }

    fn field(&self, label: &str) -> Option<&T> {
        match self {
            Self::Direct(_) => None,
            Self::Fields(fields) => fields.get(label),
        }
    }
}

fn variant_payload_shape(dag: &Dag, variant_id: DeclarationId) -> Option<VariantPayloadShape> {
    let TypeConnective::Conj { children } = &dag.declaration(variant_id).connective else {
        return None;
    };
    match children.as_slice() {
        [] => Some(VariantPayloadShape::Empty),
        [field] if field.label == "_0" => Some(VariantPayloadShape::PositionalSingle),
        fields => Some(VariantPayloadShape::NamedFields(
            fields.iter().map(|field| field.label.clone()).collect(),
        )),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum GoFieldAccessBinding {
    DirectField(String),
    AccessorMethod(String),
}

#[derive(Debug, Clone)]
struct FieldBindingBinding {
    access: GoFieldAccessBinding,
}

#[derive(Debug, Clone)]
struct TypeRealizationBinding {
    carrier: String,
    fields: HashMap<String, FieldBindingBinding>,
}

#[derive(Debug, Clone)]
struct TypeInstantiationBinding {
    carrier: String,
}

/// Mirrors emit_rust::ParameterDispositionBinding. Go's GC rendering
/// doesn't make borrow/move decisions, so the emitter doesn't act on
/// the value. The field is still required on every CallableRealization
/// (single shared schema with rust.dag), so we parse and validate it
/// fail-closed — a malformed disposition vector is a spec bug, even
/// when this target ignores the result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParameterDispositionBinding {
    Borrowed,
    Consumed,
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CallableStrategyBinding {
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
    binary_op: String,
    field_access: String,
    function_call: String,
}

#[derive(Debug, Clone)]
struct FunctionSyntaxBinding {
    definition: String,
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
}

#[derive(Debug, Clone)]
struct ValueConstructionSyntaxBinding {
    struct_literal: String,
    struct_field_init: String,
    struct_field_separator: String,
    variant_named_construction: String,
}

#[derive(Debug, Clone)]
struct LiteralSyntaxBinding {
    true_keyword: String,
    false_keyword: String,
    string_delimiter: String,
}

#[derive(Debug, Clone)]
struct GoLanguageSyntax {
    statements: StatementSyntaxBinding,
    expressions: ExpressionSyntaxBinding,
    functions: FunctionSyntaxBinding,
    type_applications: TypeApplicationSyntaxBinding,
    type_definitions: TypeDefinitionSyntaxBinding,
    values: ValueConstructionSyntaxBinding,
    literals: LiteralSyntaxBinding,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TargetExecutionModelBinding {
    memory: MemoryModelBinding,
    scope: ScopeModelBinding,
}

/// Typed read of `data go_clean_emission: CleanEmissionContract`
/// from `src/v3/spec/go.dag` — the portion this pilot consumes (E-5
/// / Lane 1 Stage 1c PR 2). Other contract rules land here as their
/// consumers wire in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CleanEmissionContractBinding {
    pattern_bindings: PatternBindingRuleBinding,
    variant_payload_field_access: VariantPayloadFieldAccessRuleBinding,
}

/// Go-valid slice of `std.clean_emission.PatternBindingRule`. Parsed
/// in `CleanEmissionContractBinding::build`, which rejects
/// target-invalid constructors instead of letting the renderer
/// normalize them later. Go has no pattern-level underscore
/// convention (blank identifier is statement-level), so
/// `EmitPrefixedUnderscoreWhenUnused` is a Python-shaped choice
/// the Go emitter refuses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PatternBindingRuleBinding {
    EmitBindingAlways,
    EmitUnderscoreWhenUnused,
}

struct RealizationIndexes {
    types: HashMap<DeclarationId, TypeRealizationBinding>,
    instantiations: HashMap<DeclarationId, TypeInstantiationBinding>,
    operators: HashMap<(DeclarationId, DeclarationId), String>,
    behaviors: HashMap<DeclarationId, String>,
    callables: HashMap<DeclarationId, CallableStrategyBinding>,
    /// Parsed and validated, but not consulted for Go emission (GC
    /// targets don't render borrow/move). Holding the field keeps the
    /// shared CallableRealization schema honest: every entry in a
    /// CallableRealization must have a well-formed dispositions vector,
    /// even when this target ignores the result.
    #[allow(dead_code)]
    callable_dispositions: HashMap<DeclarationId, Vec<ParameterDispositionBinding>>,
    patterns: HashMap<DeclarationId, PatternRealizationBinding>,
    syntax: GoLanguageSyntax,
    execution_model: TargetExecutionModelBinding,
    /// The Go clean-emission contract loaded from `data
    /// go_clean_emission: CleanEmissionContract` (E-5 / Lane 1 Stage
    /// 1c PR 2). Rule variants dispatch inside the emitter to shape
    /// emitted code so `gofmt -l` stays empty and Go's
    /// unused-local compile error never fires.
    clean_emission: CleanEmissionContractBinding,
}

impl RealizationIndexes {
    fn build(dag: &Dag) -> Result<Self, EmitError> {
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

        let mut types = HashMap::new();
        let mut instantiations = HashMap::new();
        let mut operators = HashMap::new();
        let mut behaviors = HashMap::new();
        let mut callables = HashMap::new();
        let mut callable_dispositions: HashMap<DeclarationId, Vec<ParameterDispositionBinding>> =
            HashMap::new();
        let mut patterns = HashMap::new();

        // Cache the Go language-spec declaration id once; shared
        // realizations reference it via `language: DeclarationRef`
        // and this emitter picks up only entries that match.
        // Replaces the previous TargetLanguage enum roster with a
        // declaration-identity compare (INVARIANTS.md E-6).
        let go_language_id = dag
            .go_language_spec()
            .ok_or(EmitError::MissingTargetSyntax("go_language"))?;

        for decl in dag.declarations() {
            let Some(meta_tag) = decl.meta_tag else {
                continue;
            };
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
                return Err(EmitError::MalformedRealization {
                    declaration: decl.id,
                    detail:
                        "realization data item has no Structural value_body — bootstrap inhabitance check missed a malformed spec entry",
                });
            };
            // Skip realizations declared for other shared targets
            // (e.g. Rust) by comparing the typed `language` field to
            // this emitter's cached language-spec declaration id.
            // Replaces the previous TargetLanguage enum roster; adding
            // a new shared target is now a pure spec-file change.
            let language_ref = require_field_decl_ref(fields, "language", decl.id)?;
            if language_ref != go_language_id {
                continue;
            }
            let target = require_field_decl_ref(fields, "target", decl.id)?;
            match category {
                RealizationCategory::Type => {
                    let carrier = require_field_string(fields, "carrier", decl.id)?;
                    if types
                        .insert(
                            target,
                            TypeRealizationBinding {
                                carrier,
                                fields: require_field_bindings(dag, fields, decl.id)?,
                            },
                        )
                        .is_some()
                    {
                        return Err(EmitError::DuplicateRealization {
                            declaration: decl.id,
                            detail: "two TypeRealization data items target the same declaration",
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
                                "two TypeInstantiationRealization data items target the same declaration",
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
                                "two OperatorRealization data items share the same (target, op) pair",
                        });
                    }
                }
                RealizationCategory::Behavior => {
                    let carrier = require_field_string(fields, "carrier", decl.id)?;
                    if behaviors.insert(target, carrier).is_some() {
                        return Err(EmitError::DuplicateRealization {
                            declaration: decl.id,
                            detail:
                                "two BehaviorRealization data items target the same substrate marker",
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
                                "two CallableRealization data items target the same callable declaration",
                        });
                    }
                    if callable_dispositions.insert(target, dispositions).is_some() {
                        return Err(EmitError::DuplicateRealization {
                            declaration: decl.id,
                            detail:
                                "two CallableRealization data items target the same callable declaration's parameter dispositions",
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
                                "two PatternRealization data items target the same structural sum declaration",
                        });
                    }
                }
            }
        }

        let execution_model = TargetExecutionModelBinding::build(
            dag,
            dag.go_execution_model_spec()
                .ok_or(EmitError::MissingTargetSyntax("go_execution_model"))?,
        )?;
        if execution_model.scope != ScopeModelBinding::LexicalScoping {
            return Err(EmitError::UnsupportedBehavior(
                "emit_go requires lexical scoping targets".to_string(),
            ));
        }
        let syntax = GoLanguageSyntax::build(dag)?;
        let clean_emission = CleanEmissionContractBinding::build(dag)?;

        Ok(Self {
            types,
            instantiations,
            operators,
            behaviors,
            callables,
            callable_dispositions,
            patterns,
            syntax,
            execution_model,
            clean_emission,
        })
    }
}

impl CleanEmissionContractBinding {
    /// Parse the portion of `data go_clean_emission:
    /// CleanEmissionContract` this emitter consumes. Currently only
    /// `pattern_bindings` and `variant_payload_field_access`
    /// dispatch; other rules parse when their consumers land.
    fn build(dag: &Dag) -> Result<Self, EmitError> {
        let declaration = dag
            .go_clean_emission_spec()
            .ok_or(EmitError::MissingTargetSyntax("go_clean_emission"))?;
        let fields = structural_fields_for_decl(dag, declaration)?;
        let pattern_bindings_value = fields
            .iter()
            .find(|(label, _)| label == "pattern_bindings")
            .map(|(_, value)| value)
            .ok_or(EmitError::MalformedTargetSyntax {
                declaration,
                detail: "go_clean_emission is missing required `pattern_bindings` field",
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
                    "go_clean_emission is missing required `variant_payload_field_access` field",
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
            detail: "go_clean_emission.pattern_bindings must be a PatternBindingRule variant",
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
            detail: "go_clean_emission.pattern_bindings cannot use PatternBindingRule.EmitPrefixedUnderscoreWhenUnused; Go only supports EmitBindingAlways or EmitUnderscoreWhenUnused",
        })
    } else if *constructor == not_applicable {
        Err(EmitError::MalformedTargetSyntax {
            declaration,
            detail: "go_clean_emission.pattern_bindings cannot use PatternBindingRule.NotApplicablePatternBinding; Go only supports EmitBindingAlways or EmitUnderscoreWhenUnused",
        })
    } else {
        Err(EmitError::MalformedTargetSyntax {
            declaration,
            detail: "go_clean_emission.pattern_bindings constructor is not a known PatternBindingRule variant",
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
                "go_clean_emission.variant_payload_field_access must be a VariantPayloadFieldAccessRule variant",
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
    if *constructor == access_from_payload_binding {
        Ok(VariantPayloadFieldAccessRuleBinding::AccessFromPayloadBinding)
    } else if *constructor == override_named_fields_at_binding_site {
        Err(EmitError::MalformedTargetSyntax {
            declaration,
            detail: "go_clean_emission.variant_payload_field_access cannot use VariantPayloadFieldAccessRule.OverrideNamedFieldsAtBindingSite; Go requires AccessFromPayloadBinding for native sum payloads",
        })
    } else {
        Err(EmitError::MalformedTargetSyntax {
            declaration,
            detail: "go_clean_emission.variant_payload_field_access constructor is not a known VariantPayloadFieldAccessRule variant",
        })
    }
}

impl GoLanguageSyntax {
    fn build(dag: &Dag) -> Result<Self, EmitError> {
        let language_decl = dag
            .go_language_spec()
            .ok_or(EmitError::MissingTargetSyntax("go_language"))?;
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
            values: parse_value_construction_syntax(
                dag,
                require_field_decl_ref(fields, "values", language_decl)?,
            )?,
            literals: parse_literal_syntax(
                dag,
                require_field_decl_ref(fields, "literals", language_decl)?,
            )?,
        })
    }
}

impl TargetExecutionModelBinding {
    fn build(dag: &Dag, declaration: DeclarationId) -> Result<Self, EmitError> {
        let fields = structural_fields_for_decl(dag, declaration)?;
        Ok(Self {
            memory: require_memory_model(dag, fields, declaration)?,
            scope: require_scope_model(dag, fields, declaration)?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmitTarget {
    Go,
    Rust,
    Python,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmitMode {
    Program,
    Module,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmittedSource {
    pub text: String,
    pub target: EmitTarget,
    pub mode: EmitMode,
}

#[derive(Debug, Clone)]
pub enum EmitDispatchError {
    Core(EmitError),
    Python(EmitPythonError),
}

impl From<EmitError> for EmitDispatchError {
    fn from(value: EmitError) -> Self {
        Self::Core(value)
    }
}

impl From<EmitPythonError> for EmitDispatchError {
    fn from(value: EmitPythonError) -> Self {
        Self::Python(value)
    }
}

pub fn emit(dag: &Dag, target: EmitTarget) -> Result<EmittedSource, EmitDispatchError> {
    emit_with_mode(dag, target, EmitMode::Program)
}

pub fn emit_module(dag: &Dag, target: EmitTarget) -> Result<EmittedSource, EmitDispatchError> {
    emit_with_mode(dag, target, EmitMode::Module)
}

fn emit_with_mode(
    dag: &Dag,
    target: EmitTarget,
    mode: EmitMode,
) -> Result<EmittedSource, EmitDispatchError> {
    let text = match target {
        EmitTarget::Go => emit_go_with_mode(dag, mode).map_err(EmitDispatchError::Core)?,
        EmitTarget::Rust => {
            rust_target::emit_rust_with_mode(dag, mode).map_err(EmitDispatchError::Core)?
        }
        EmitTarget::Python => {
            python_target::emit_python_with_mode(dag, mode).map_err(EmitDispatchError::Python)?
        }
    };
    Ok(EmittedSource { text, target, mode })
}

fn emit_go_with_mode(dag: &Dag, mode: EmitMode) -> Result<String, EmitError> {
    let indexes = RealizationIndexes::build(dag)?;
    if indexes.execution_model.memory == MemoryModelBinding::OwnershipBased {
        return Err(EmitError::UnsupportedBehavior(
            "emit_go requires a non-ownership execution model".to_string(),
        ));
    }
    let main_marker = dag
        .main_marker()
        .ok_or(EmitError::MissingSubstrateMarker(SubstrateMarkerRole::Main))?;
    let type_decls: Vec<_> = dag
        .declarations()
        .iter()
        .filter(|decl| !is_bootstrap_file(&decl.span.file))
        .filter(|decl| decl.name.is_some())
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
        .filter(|decl| !is_bootstrap_file(&decl.span.file))
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
    let top_level_binds: Vec<_> = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .filter(|bind| bind.params.is_empty())
        .collect();

    if mode == EmitMode::Program && top_level_binds.is_empty() {
        return Err(EmitError::UnsupportedBehavior(
            "emit_go requires at least one top-level value Bind".to_string(),
        ));
    }
    if mode == EmitMode::Module && !top_level_binds.is_empty() {
        return Err(EmitError::UnsupportedBehavior(
            "emit_go module mode does not support top-level value Binds".to_string(),
        ));
    }

    let mut bound_names = HashMap::new();
    for bind in &top_level_binds {
        bound_names.insert(bind.value, bind.name.clone());
    }
    let ctx = Ctx {
        dag,
        indexes: &indexes,
        bound_names: &bound_names,
        mode,
    };

    let mut sections = vec![if mode == EmitMode::Program {
        "package main".to_string()
    } else {
        "package emitted".to_string()
    }];
    if mode == EmitMode::Program {
        sections.push("import \"fmt\"".to_string());
    }

    let rendered_types = type_decls
        .iter()
        .map(|decl| ctx.render_type_declaration(decl))
        .collect::<Result<Vec<_>, _>>()?;
    if !rendered_types.is_empty() {
        sections.push(rendered_types.join("\n\n"));
    }

    let rendered_functions = function_decls
        .iter()
        .map(|decl| ctx.render_function_declaration(decl))
        .collect::<Result<Vec<_>, _>>()?;
    if !rendered_functions.is_empty() {
        sections.push(rendered_functions.join("\n\n"));
    }

    if mode == EmitMode::Program {
        let body = top_level_binds
            .iter()
            .map(|bind| {
                let ty = ctx.go_type_name_for_port(bind.value)?;
                let value = ctx.render_top_level_value(bind.value)?;
                Ok(render_named_template(
                    &ctx.indexes.syntax.statements.let_binding,
                    &[("name", &bind.name), ("type", &ty), ("value", &value)],
                ))
            })
            .collect::<Result<Vec<_>, EmitError>>()?
            .join(";\n");
        let final_bind_name = &top_level_binds.last().expect("guarded").name;
        let main_template =
            indexes
                .behaviors
                .get(&main_marker)
                .ok_or(EmitError::MissingBehaviorRealization {
                    marker: main_marker,
                })?;
        sections.push(render_named_template(
            main_template,
            &[("body", &body), ("final", final_bind_name)],
        ));
    }

    Ok(sections.join("\n\n"))
}

struct Ctx<'a> {
    dag: &'a Dag,
    indexes: &'a RealizationIndexes,
    bound_names: &'a HashMap<PortId, String>,
    mode: EmitMode,
}

#[derive(Debug, Clone, Default)]
struct RenderLocals {
    names: HashMap<PortId, String>,
    payload_bindings: HashMap<PortId, VariantPayloadBinding<String>>,
}

impl<'a> Ctx<'a> {
    fn render_port(&self, port: PortId, locals: &RenderLocals) -> Result<String, EmitError> {
        if let Some(name) = locals.names.get(&port) {
            return Ok(name.clone());
        }
        if let Some(name) = locals
            .payload_bindings
            .get(&port)
            .and_then(VariantPayloadBinding::direct)
        {
            return Ok(name.clone());
        }
        if let Some(name) = self.bound_names.get(&port) {
            return Ok(name.clone());
        }
        self.dispatch_producer(port, locals)
    }

    fn render_top_level_value(&self, port: PortId) -> Result<String, EmitError> {
        self.dispatch_producer(port, &RenderLocals::default())
    }

    fn dispatch_producer(&self, port: PortId, locals: &RenderLocals) -> Result<String, EmitError> {
        let Some(node_id) = self.dag.port(port).produced_by else {
            return Err(EmitError::UnsupportedBehavior(
                "render reached a port with no producer".to_string(),
            ));
        };
        match self.dag.node(node_id) {
            Behavior::Value(v) => Ok(render_value(v, &self.indexes.syntax.literals)),
            Behavior::Transform(t) => self.render_transform(t, locals),
            Behavior::Branch(b) => self.render_branch(b, locals),
            Behavior::Loop(_) => {
                // emit_go does not yet model `Behavior::Loop`. Earlier
                // code rendered just the loop body's result port, which
                // silently dropped the iteration semantics — a Loop
                // over a list became its first iteration's expression.
                // Fail-closed instead so callers see the unsupported
                // case directly.
                Err(EmitError::UnsupportedBehavior(
                    "emit_go does not yet support Behavior::Loop; iteration construct must be expressed via fold/map/filter callables for now"
                        .to_string(),
                ))
            }
            Behavior::Bind(bind) => Ok(bind.name.clone()),
        }
    }

    fn render_transform(
        &self,
        t: &TransformNode,
        locals: &RenderLocals,
    ) -> Result<String, EmitError> {
        match &t.target {
            TransformTarget::Operator(op) => self.render_operator(t, *op, locals),
            TransformTarget::FieldProject {
                field_label,
                field_child: _,
            } => self.render_field_project(t, field_label, locals),
            TransformTarget::Callable(target) => {
                let (template, arguments) = callable_template(*target, self.dag);
                if let Some(strategy) = self.indexes.callables.get(&template) {
                    return self.render_realized_callable(
                        t.output, template, *strategy, &arguments, &t.inputs, locals,
                    );
                }
                self.render_general_callable(template, &t.inputs, locals)
            }
        }
    }

    fn render_field_project(
        &self,
        t: &TransformNode,
        field_label: &str,
        locals: &RenderLocals,
    ) -> Result<String, EmitError> {
        if t.inputs.len() != 1 {
            return Err(EmitError::UnsupportedBehavior(format!(
                "field projection .{field_label} expected one input"
            )));
        }
        if let Some(binding) = locals
            .payload_bindings
            .get(&t.inputs[0])
            .and_then(|binding| binding.field(field_label))
        {
            return Ok(binding.clone());
        }
        let parent = self.render_port(t.inputs[0], locals)?;
        let parent_type = primitive_type_id_for_port(self.dag, t.inputs[0])?;
        if let Some(type_binding) = self.indexes.types.get(&parent_type) {
            if let Some(binding) = type_binding.fields.get(field_label) {
                return Ok(match &binding.access {
                    GoFieldAccessBinding::DirectField(name) => render_named_template(
                        &self.indexes.syntax.expressions.field_access,
                        &[("object", &parent), ("field", name)],
                    ),
                    GoFieldAccessBinding::AccessorMethod(name) => format!(
                        "{}()",
                        render_named_template(
                            &self.indexes.syntax.expressions.field_access,
                            &[("object", &parent), ("field", name)],
                        )
                    ),
                });
            }
        }
        Ok(render_named_template(
            &self.indexes.syntax.expressions.field_access,
            &[("object", &parent), ("field", field_label)],
        ))
    }

    fn render_operator(
        &self,
        t: &TransformNode,
        op: OperatorKind,
        locals: &RenderLocals,
    ) -> Result<String, EmitError> {
        if t.inputs.len() != 2 {
            return Err(EmitError::UnsupportedBehavior(format!(
                "operator {:?} arity {} is not supported",
                op,
                t.inputs.len()
            )));
        }
        // Logical operators are Bool-monomorphic and do not dispatch
        // through a Bool algebra today — render the symbol directly.
        // Go uses `&&` / `||`; same as the source surface.
        if let OperatorKind::Logical(logical_op) = op {
            let symbol = match logical_op {
                crate::dag::LogicalOp::And => "&&",
                crate::dag::LogicalOp::Or => "||",
            };
            let lhs = self.render_port(t.inputs[0], locals)?;
            let rhs = self.render_port(t.inputs[1], locals)?;
            return Ok(render_named_template(
                &self.indexes.syntax.expressions.binary_op,
                &[("lhs", &lhs), ("op", symbol), ("rhs", &rhs)],
            ));
        }
        let operand_type = primitive_type_id_for_port(self.dag, t.inputs[0])?;
        let op_decl = algebra_field_for_operator(self.dag, operand_type, op)?;
        let carrier = self.indexes.operators.get(&(operand_type, op_decl)).ok_or(
            EmitError::MissingOperatorRealization {
                target: operand_type,
                op: op_decl,
            },
        )?;
        let lhs = self.render_port(t.inputs[0], locals)?;
        let rhs = self.render_port(t.inputs[1], locals)?;
        Ok(render_named_template(
            &self.indexes.syntax.expressions.binary_op,
            &[("lhs", &lhs), ("op", carrier), ("rhs", &rhs)],
        ))
    }

    fn render_branch(&self, b: &BranchNode, locals: &RenderLocals) -> Result<String, EmitError> {
        if self.branch_scrutinee_is_bool(b)? {
            return self.render_bool_branch(b, locals);
        }
        if let Some(rendered) = self.render_realized_pattern_branch(b, locals)? {
            return Ok(rendered);
        }
        let scrutinee_type = primitive_type_id_for_port(self.dag, b.input)?;
        if matches!(
            self.dag.declaration(scrutinee_type).connective,
            TypeConnective::Cardinality {
                bound: CardinalityBound::AtMostOne,
                ..
            }
        ) {
            return self.render_optional_branch(b, locals);
        }
        self.render_sum_branch(b, locals)
    }

    fn render_bool_branch(
        &self,
        branch: &BranchNode,
        locals: &RenderLocals,
    ) -> Result<String, EmitError> {
        let (then_path, else_path) = self.split_bool_paths(branch)?;
        let cond = self.render_port(branch.input, locals)?;
        let then_expr = self.render_path_body(then_path, locals)?;
        let else_expr = self.render_path_body(else_path, locals)?;
        let ret = self.go_type_name_for_port(branch.output)?;
        Ok(format!(
            "func() {ret} {{ if {cond} {{ return {then_expr} }}; return {else_expr} }}()"
        ))
    }

    fn render_optional_branch(
        &self,
        branch: &BranchNode,
        locals: &RenderLocals,
    ) -> Result<String, EmitError> {
        let scrutinee = self.render_port(branch.input, locals)?;
        let scrutinee_type = primitive_type_id_for_port(self.dag, branch.input)?;
        let disj_id = walk_to_disj(self.dag, scrutinee_type).ok_or_else(|| {
            EmitError::UnsupportedBehavior("optional branch must walk to a Disj".to_string())
        })?;
        let TypeConnective::Disj { variants } = &self.dag.declaration(disj_id).connective else {
            unreachable!("walk_to_disj returned non-Disj")
        };
        let none_variant = variants
            .iter()
            .find(|variant| variant.label == "None")
            .map(|variant| variant.ty)
            .ok_or_else(|| {
                EmitError::UnsupportedBehavior("optional branch requires None".to_string())
            })?;
        let some_variant = variants
            .iter()
            .find(|variant| variant.label == "Some")
            .map(|variant| variant.ty)
            .ok_or_else(|| {
                EmitError::UnsupportedBehavior("optional branch requires Some".to_string())
            })?;
        let none_path = find_resolved_branch_path(branch, none_variant).ok_or_else(|| {
            EmitError::UnsupportedBehavior("optional branch missing None arm".to_string())
        })?;
        let some_path = find_resolved_branch_path(branch, some_variant).ok_or_else(|| {
            EmitError::UnsupportedBehavior("optional branch missing Some arm".to_string())
        })?;
        let ret = self.go_type_name_for_port(branch.output)?;
        let none_expr = self.render_path_body(none_path, locals)?;
        let mut some_locals = locals.clone();
        if let Some(binding) = &some_path.binding {
            some_locals
                .names
                .insert(binding.payload_port, format!("*({scrutinee})"));
        }
        let some_expr = self.render_port(some_path.output, &some_locals)?;
        Ok(format!(
            "func() {ret} {{ if ({scrutinee}) == nil {{ return {none_expr} }}; return {some_expr} }}()"
        ))
    }

    fn render_sum_branch(
        &self,
        branch: &BranchNode,
        locals: &RenderLocals,
    ) -> Result<String, EmitError> {
        let scrutinee = self.render_port(branch.input, locals)?;
        let ret = self.go_type_name_for_port(branch.output)?;
        let mut arms = Vec::new();
        // E-5 / Lane 1 Stage 1c PR 2: track whether any arm actually
        // consumes the payload binding. If zero arms do, the type-switch
        // header drops `v :=` — otherwise Go would flag `v` as
        // `declared and not used`. Structural: driven by port liveness,
        // not by scanning the rendered arm text.
        let mut any_arm_uses_v = false;
        for path in &branch.paths {
            let variant_id = match &path.pattern {
                BranchPattern::ResolvedVariant(id) => *id,
                BranchPattern::UnresolvedVariant { name, .. } => {
                    return Err(EmitError::UnresolvedBranchPattern {
                        variant_name: name.clone(),
                    });
                }
            };
            let variant_name = variant_parent_info(self.dag, variant_id)
                .map(|(_, variant_name)| variant_name)
                .unwrap_or_else(|| {
                    self.dag
                        .declaration(variant_id)
                        .name
                        .clone()
                        .unwrap_or_else(|| "UnknownVariant".to_string())
                });
            let mut arm_locals = locals.clone();
            if let Some(binding) = &path.binding {
                let elide = matches!(
                    self.indexes.clean_emission.pattern_bindings,
                    PatternBindingRuleBinding::EmitUnderscoreWhenUnused
                ) && !self.port_is_consumed_from(path.output, binding.payload_port);
                if !elide {
                    if let Some(payload_binding) =
                        self.variant_payload_binding_for_variant(variant_id, "v")?
                    {
                        arm_locals
                            .payload_bindings
                            .insert(binding.payload_port, payload_binding);
                        any_arm_uses_v = true;
                    }
                }
            }
            let body = self.render_port(path.output, &arm_locals)?;
            arms.push(format!("case {variant_name}: return {body}"));
        }
        let switch_header = if any_arm_uses_v {
            format!("switch v := any({scrutinee}).(type)")
        } else {
            format!("switch any({scrutinee}).(type)")
        };
        Ok(format!(
            "func() {ret} {{ {switch_header} {{ {} default: panic(\"non-exhaustive match\") }} }}()",
            arms.join(" ")
        ))
    }

    /// Structural port-liveness walk. Returns true if `target` appears
    /// as any port reachable from `root` via producer→input edges.
    /// Mirrors `emit_rust::Ctx::port_is_consumed_from` — the fact is
    /// structural, so the check is structural too (no textual scan of
    /// rendered arm bodies). Ports with no producer are leaves; the
    /// walk hits them and either returns true (hit the target) or
    /// skips (unrelated parameter port).
    fn port_is_consumed_from(&self, root: PortId, target: PortId) -> bool {
        if root == target {
            return true;
        }
        let mut visited: HashSet<PortId> = HashSet::new();
        let mut queue: Vec<PortId> = vec![root];
        while let Some(port) = queue.pop() {
            if !visited.insert(port) {
                continue;
            }
            if port == target {
                return true;
            }
            let Some(producer) = self.dag.port(port).produced_by else {
                continue;
            };
            match self.dag.node(producer) {
                Behavior::Value(_) => {}
                Behavior::Transform(t) => {
                    for input in t.inputs.iter().copied() {
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
                    if let Some(count) = l.bound.count_port() {
                        queue.push(count);
                    }
                    queue.push(go_behavior_result_port(self.dag.node(l.body)));
                }
                Behavior::Bind(b) => {
                    queue.push(b.value);
                }
            }
        }
        false
    }

    fn render_realized_pattern_branch(
        &self,
        branch: &BranchNode,
        locals: &RenderLocals,
    ) -> Result<Option<String>, EmitError> {
        let scrutinee_type = primitive_type_id_for_port(self.dag, branch.input)?;
        let Some(disj_id) = walk_to_disj(self.dag, scrutinee_type) else {
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
        let ret = self.go_type_name_for_port(branch.output)?;
        let scrutinee = self.render_port(branch.input, locals)?;
        let empty_body = self.render_path_body(empty_path, locals)?;
        let list_name = "__list";
        let realized_scrutinee = render_named_template(&binding.scrutinee, &[("expr", &scrutinee)]);
        // Empty/cons branches dispatch on the typed PatternRealization
        // fields (`empty_pattern`, `cons_pattern`) rather than a
        // hardcoded `len(__list) == 0` check. Go's empty_pattern
        // template rendered with `expr=__list` becomes the if
        // condition; cons_pattern (a passthrough on Go) becomes the
        // list expression head/tail are extracted from.
        let empty_predicate = render_named_template(&binding.empty_pattern, &[("expr", list_name)]);
        let cons_expr = render_named_template(&binding.cons_pattern, &[("expr", list_name)]);
        let head_expr = render_named_template(&binding.head_expr, &[("list", &cons_expr)]);
        let tail_expr = render_named_template(&binding.tail_expr, &[("list", &cons_expr)]);
        let mut cons_locals = locals.clone();
        if let Some(payload) = &cons_path.binding {
            let mut fields = HashMap::new();
            fields.insert("head".to_string(), head_expr);
            fields.insert("tail".to_string(), tail_expr);
            cons_locals
                .payload_bindings
                .insert(payload.payload_port, VariantPayloadBinding::Fields(fields));
        }
        let cons_body = self.render_port(cons_path.output, &cons_locals)?;
        Ok(format!(
            "func() {ret} {{ {list_name} := {realized_scrutinee}; if {empty_predicate} {{ return {empty_body} }}; return {cons_body} }}()"
        ))
    }

    fn render_path_body(&self, path: &Path, locals: &RenderLocals) -> Result<String, EmitError> {
        self.render_port(path.output, locals)
    }

    fn render_realized_callable(
        &self,
        output_port: PortId,
        template: DeclarationId,
        strategy: CallableStrategyBinding,
        arguments: &[TemplateArgument],
        inputs: &[PortId],
        locals: &RenderLocals,
    ) -> Result<String, EmitError> {
        match strategy {
            CallableStrategyBinding::ListEmpty => Ok("nil".to_string()),
            CallableStrategyBinding::ListSingleton => {
                let [input] = inputs else {
                    return Err(EmitError::UnsupportedBehavior(
                        "singleton expects one input".to_string(),
                    ));
                };
                let value = self.render_port(*input, locals)?;
                let element = self.list_element_type_name_for_list_port(output_port)?;
                Ok(format!("[]{element}{{{value}}}"))
            }
            CallableStrategyBinding::ListCons => {
                let [head_port, tail_port] = inputs else {
                    return Err(EmitError::UnsupportedBehavior(
                        "cons expects [head, tail]".to_string(),
                    ));
                };
                let head = self.render_port(*head_port, locals)?;
                let tail = self.render_port(*tail_port, locals)?;
                let element = self.list_element_type_name_for_list_port(*tail_port)?;
                Ok(format!("append([]{element}{{{head}}}, {tail}...)"))
            }
            CallableStrategyBinding::ListConcat => {
                let [left_port, right_port] = inputs else {
                    return Err(EmitError::UnsupportedBehavior(
                        "concat expects [left, right]".to_string(),
                    ));
                };
                let left = self.render_port(*left_port, locals)?;
                let right = self.render_port(*right_port, locals)?;
                Ok(format!("append({left}, {right}...)"))
            }
            CallableStrategyBinding::ListLength => {
                let [list_port] = inputs else {
                    return Err(EmitError::UnsupportedBehavior(
                        "length expects [list]".to_string(),
                    ));
                };
                let list = self.render_port(*list_port, locals)?;
                Ok(format!("int64(len({list}))"))
            }
            CallableStrategyBinding::ListIsEmpty => {
                let [list_port] = inputs else {
                    return Err(EmitError::UnsupportedBehavior(
                        "is_empty expects [list]".to_string(),
                    ));
                };
                let list = self.render_port(*list_port, locals)?;
                Ok(format!("len({list}) == 0"))
            }
            CallableStrategyBinding::ListFold => {
                let [list_port, init_port] = inputs else {
                    return Err(EmitError::UnsupportedBehavior(
                        "fold expects [list, init]".to_string(),
                    ));
                };
                let fn_decl = bound_callable_argument(self.dag, template, arguments, 2)?;
                let list = self.render_port(*list_port, locals)?;
                let init = self.render_port(*init_port, locals)?;
                let ret = self.go_type_name_for_port(output_port)?;
                let acc_name = "__foldAcc";
                let item_name = "__foldItem";
                let body = self.render_callable_body(
                    fn_decl,
                    &[
                        (acc_name.to_string(), acc_name.to_string()),
                        (item_name.to_string(), item_name.to_string()),
                    ],
                    locals,
                )?;
                Ok(format!(
                    "func() {ret} {{ {acc_name} := {init}; for _, {item_name} := range {list} {{ _ = {item_name}; {acc_name} = {body} }}; return {acc_name} }}()"
                ))
            }
            CallableStrategyBinding::ListMap => {
                let [list_port] = inputs else {
                    return Err(EmitError::UnsupportedBehavior(
                        "map expects [list]".to_string(),
                    ));
                };
                let fn_decl = bound_callable_argument(self.dag, template, arguments, 1)?;
                let list = self.render_port(*list_port, locals)?;
                let ret = self.go_type_name_for_port(output_port)?;
                let item_name = "__mapItem";
                let body = self.render_callable_body(
                    fn_decl,
                    &[(item_name.to_string(), item_name.to_string())],
                    locals,
                )?;
                Ok(format!(
                    "func() {ret} {{ __out := make({ret}, 0, len({list})); for _, {item_name} := range {list} {{ _ = {item_name}; __out = append(__out, {body}) }}; return __out }}()"
                ))
            }
            CallableStrategyBinding::ListFilter => {
                let [list_port] = inputs else {
                    return Err(EmitError::UnsupportedBehavior(
                        "filter expects [list]".to_string(),
                    ));
                };
                let fn_decl = bound_callable_argument(self.dag, template, arguments, 1)?;
                let list = self.render_port(*list_port, locals)?;
                let ret = self.go_type_name_for_port(output_port)?;
                let item_name = "__filterItem";
                let predicate = self.render_callable_body(
                    fn_decl,
                    &[(item_name.to_string(), item_name.to_string())],
                    locals,
                )?;
                Ok(format!(
                    "func() {ret} {{ __out := make({ret}, 0); for _, {item_name} := range {list} {{ _ = {item_name}; if {predicate} {{ __out = append(__out, {item_name}) }} }}; return __out }}()"
                ))
            }
            CallableStrategyBinding::ListContains => {
                let [list_port, item_port] = inputs else {
                    return Err(EmitError::UnsupportedBehavior(
                        "contains expects [list, item]".to_string(),
                    ));
                };
                let list = self.render_port(*list_port, locals)?;
                let item = self.render_port(*item_port, locals)?;
                Ok(format!(
                    "func() bool {{ for _, __item := range {list} {{ if __item == {item} {{ return true }} }}; return false }}()"
                ))
            }
        }
    }

    fn render_general_callable(
        &self,
        template: DeclarationId,
        inputs: &[PortId],
        locals: &RenderLocals,
    ) -> Result<String, EmitError> {
        if let Some(rendered) = self.render_variant_constructor(template, inputs, locals)? {
            return Ok(rendered);
        }
        if let Some(rendered) = self.render_record_constructor(template, inputs, locals)? {
            return Ok(rendered);
        }
        let func =
            self.dag
                .declaration(template)
                .name
                .clone()
                .ok_or(EmitError::UnsupportedBehavior(
                    "callable target is anonymous and cannot be rendered as a direct Go call"
                        .to_string(),
                ))?;
        let args = inputs
            .iter()
            .map(|port| self.render_port(*port, locals))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(render_named_template(
            &self.indexes.syntax.expressions.function_call,
            &[("func", &func), ("args", &args.join(", "))],
        ))
    }

    fn render_record_constructor(
        &self,
        template: DeclarationId,
        inputs: &[PortId],
        locals: &RenderLocals,
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
                "record constructor `{type_name}` expected {} inputs, got {}",
                children.len(),
                inputs.len()
            )));
        }
        let fields = children
            .iter()
            .zip(inputs.iter())
            .map(|(field, input)| {
                let value = self.render_port(*input, locals)?;
                Ok(render_named_template(
                    &self.indexes.syntax.values.struct_field_init,
                    &[("name", &field.label), ("value", &value)],
                ))
            })
            .collect::<Result<Vec<_>, EmitError>>()?;
        Ok(Some(render_named_template(
            &self.indexes.syntax.values.struct_literal,
            &[
                ("type", type_name),
                (
                    "fields",
                    &fields.join(&self.indexes.syntax.values.struct_field_separator),
                ),
            ],
        )))
    }

    fn render_variant_constructor(
        &self,
        template: DeclarationId,
        inputs: &[PortId],
        locals: &RenderLocals,
    ) -> Result<Option<String>, EmitError> {
        let Some((_parent_name, variant_name)) = variant_parent_info(self.dag, template) else {
            return Ok(None);
        };
        let TypeConnective::Conj { children } = &self.dag.declaration(template).connective else {
            return Ok(None);
        };
        if children.len() != inputs.len() {
            return Err(EmitError::UnsupportedBehavior(format!(
                "variant constructor `{variant_name}` expected {} payload field(s), got {}",
                children.len(),
                inputs.len()
            )));
        }
        if children.is_empty() {
            return Ok(Some(format!("{variant_name}{{}}")));
        }
        let fields = children
            .iter()
            .zip(inputs.iter())
            .map(|(field, input)| {
                let value = self.render_port(*input, locals)?;
                Ok(render_named_template(
                    &self.indexes.syntax.values.struct_field_init,
                    &[("name", &field.label), ("value", &value)],
                ))
            })
            .collect::<Result<Vec<_>, EmitError>>()?;
        Ok(Some(render_named_template(
            &self.indexes.syntax.values.variant_named_construction,
            &[
                ("variant", &variant_name),
                (
                    "fields",
                    &fields.join(&self.indexes.syntax.values.struct_field_separator),
                ),
            ],
        )))
    }

    fn render_callable_body(
        &self,
        callable_decl: DeclarationId,
        param_bindings: &[(String, String)],
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
                "external or unparsed callable bodies are not yet supported".to_string(),
            ));
        };
        let bind = self
            .dag
            .node(*bind_id)
            .as_bind()
            .expect("UserDefined arrow body must point at a Bind");
        if inputs.len() != param_bindings.len() {
            return Err(EmitError::UnsupportedBehavior(
                "callable parameter count does not match requested bindings".to_string(),
            ));
        }
        let capture_count = bind.params.len() - inputs.len();
        let mut locals = RenderLocals::default();
        for capture in bind.params.iter().copied().take(capture_count) {
            locals
                .names
                .insert(capture, self.render_port(capture, outer_locals)?);
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
        self.render_port(bind.value, &locals)
    }

    fn render_function_declaration(
        &self,
        declaration: &crate::dag::Declaration,
    ) -> Result<String, EmitError> {
        let Some(name) = &declaration.name else {
            return Err(EmitError::UnsupportedBehavior(
                "anonymous Arrow declarations cannot be emitted".to_string(),
            ));
        };
        let TypeConnective::Arrow { inputs, body, .. } = &declaration.connective else {
            return Err(EmitError::UnsupportedBehavior(
                "render_function_declaration expected an Arrow".to_string(),
            ));
        };
        let ArrowBody::UserDefined(bind_id) = body else {
            return Err(EmitError::UnsupportedBehavior(
                "external Arrow bodies are not yet supported in function emission".to_string(),
            ));
        };
        let bind = self
            .dag
            .node(*bind_id)
            .as_bind()
            .expect("UserDefined arrow body must point at a Bind");
        let mut locals = RenderLocals::default();
        let params = bind
            .params
            .iter()
            .enumerate()
            .map(|(idx, port)| {
                let param_name = format!("p{idx}");
                locals.names.insert(*port, param_name.clone());
                let ty = self.go_type_name_for_port(*port)?;
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
        let ret = self.go_type_name_for_port(bind.value)?;
        let body = self.render_port(bind.value, &locals)?;
        let rendered = render_named_template(
            &self.indexes.syntax.functions.definition,
            &[
                ("name", name),
                (
                    "params",
                    &params.join(&self.indexes.syntax.functions.param_separator),
                ),
                ("ret", &ret),
                ("body", &body),
            ],
        );
        let _ = self.mode;
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
        match &declaration.connective {
            TypeConnective::Conj { children } => {
                let fields = children
                    .iter()
                    .map(|field| self.render_struct_field(field))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(render_named_template(
                    &self.indexes.syntax.type_definitions.struct_def,
                    &[("name", name), ("fields", &fields.join("; "))],
                ))
            }
            TypeConnective::Disj { variants } => {
                let mut parts = vec![format!("type {name} interface {{ is{name}() }}")];
                for variant in variants {
                    let variant_decl = self.dag.declaration(variant.ty);
                    let TypeConnective::Conj { children } = &variant_decl.connective else {
                        return Err(EmitError::UnsupportedBehavior(format!(
                            "enum variant `{}` does not lower to a product declaration",
                            variant.label
                        )));
                    };
                    if children.is_empty() {
                        parts.push(format!("type {} struct{{}}", variant.label));
                    } else {
                        let fields = children
                            .iter()
                            .map(|field| self.render_struct_field(field))
                            .collect::<Result<Vec<_>, _>>()?;
                        parts.push(format!(
                            "type {} struct {{ {} }}",
                            variant.label,
                            fields.join("; ")
                        ));
                    }
                    parts.push(format!("func ({}) is{name}() {{}}", variant.label));
                }
                Ok(parts.join("\n"))
            }
            _ => Err(EmitError::UnsupportedBehavior(format!(
                "type declaration `{name}` does not lower to a record or sum shape"
            ))),
        }
    }

    fn render_struct_field(&self, field: &Field) -> Result<String, EmitError> {
        let ty = self.go_type_name_for_decl(field.ty)?;
        Ok(render_named_template(
            &self.indexes.syntax.type_definitions.struct_field,
            &[("name", &field.label), ("type", &ty)],
        ))
    }

    fn branch_scrutinee_is_bool(&self, branch: &BranchNode) -> Result<bool, EmitError> {
        let scrutinee_type = primitive_type_id_for_port(self.dag, branch.input)?;
        let Some(bool_shape) = self.dag.bool_shape() else {
            return Err(EmitError::MissingTypeRealization {
                target: scrutinee_type,
            });
        };
        let Some(scrutinee_disj) = walk_to_disj(self.dag, scrutinee_type) else {
            return Ok(false);
        };
        let Some(bool_disj) = walk_to_disj(self.dag, bool_shape.declaration) else {
            return Ok(false);
        };
        Ok(scrutinee_disj == bool_disj)
    }

    fn split_bool_paths<'p>(
        &self,
        branch: &'p BranchNode,
    ) -> Result<(&'p Path, &'p Path), EmitError> {
        let scrutinee_type = primitive_type_id_for_port(self.dag, branch.input)?;
        let disj_id = walk_to_disj(self.dag, scrutinee_type).ok_or_else(|| {
            EmitError::UnsupportedBehavior(
                "branch scrutinee type does not walk to a Disj".to_string(),
            )
        })?;
        let variants = match &self.dag.declaration(disj_id).connective {
            TypeConnective::Disj { variants } => variants,
            _ => unreachable!("walk_to_disj returned non-Disj"),
        };
        if variants.len() != 2 {
            return Err(EmitError::NonBooleanBranch {
                variant_ids: variants.iter().map(|variant| variant.ty).collect(),
            });
        }
        let true_variant = variants[0].ty;
        let false_variant = variants[1].ty;
        let mut then_path = None;
        let mut else_path = None;
        for path in &branch.paths {
            let resolved = match &path.pattern {
                BranchPattern::ResolvedVariant(id) => *id,
                BranchPattern::UnresolvedVariant { name, .. } => {
                    return Err(EmitError::UnresolvedBranchPattern {
                        variant_name: name.clone(),
                    });
                }
            };
            if resolved == true_variant {
                then_path = Some(path);
            } else if resolved == false_variant {
                else_path = Some(path);
            }
        }
        match (then_path, else_path) {
            (Some(t), Some(e)) => Ok((t, e)),
            _ => Err(EmitError::UnsupportedBehavior(
                "if/else branch must have both True and False arms".to_string(),
            )),
        }
    }

    fn go_type_name_for_port(&self, port: PortId) -> Result<String, EmitError> {
        let ty = self
            .dag
            .port(port)
            .value_type()
            .ok_or(EmitError::UntypedPort(port))?;
        self.go_type_name_for_decl(ty.declaration)
    }

    fn go_type_name_for_decl(&self, declaration: DeclarationId) -> Result<String, EmitError> {
        self.go_type_name_for_decl_at_depth(declaration, 0)
    }

    fn go_type_name_for_decl_at_depth(
        &self,
        declaration: DeclarationId,
        depth: usize,
    ) -> Result<String, EmitError> {
        if depth >= 32 {
            return Err(EmitError::UnsupportedBehavior(
                "type-name rendering exceeded depth 32".to_string(),
            ));
        }
        if let Some(binding) = self.indexes.types.get(&declaration) {
            return Ok(binding.carrier.clone());
        }
        let decl = self.dag.declaration(declaration);
        if let Some(name) = &decl.name {
            return Ok(name.clone());
        }
        match &decl.connective {
            TypeConnective::Instantiation {
                template,
                arguments,
            } => {
                let Some(binding) = self.indexes.instantiations.get(template) else {
                    return Err(EmitError::MissingTypeRealization { target: *template });
                };
                match arguments.as_slice() {
                    [element] => {
                        let element_name =
                            self.go_type_name_for_decl_at_depth(element.value, depth + 1)?;
                        Ok(render_named_template(
                            &binding.carrier,
                            &[("element", &element_name)],
                        ))
                    }
                    [key, value] => {
                        let key_name = self.go_type_name_for_decl_at_depth(key.value, depth + 1)?;
                        let value_name =
                            self.go_type_name_for_decl_at_depth(value.value, depth + 1)?;
                        Ok(render_named_template(
                            &binding.carrier,
                            &[("key", &key_name), ("value", &value_name)],
                        ))
                    }
                    _ => Err(EmitError::UnsupportedBehavior(
                        "instantiated type carrier only supports arities 1 and 2".to_string(),
                    )),
                }
            }
            TypeConnective::Cardinality {
                element,
                bound: CardinalityBound::AtMostOne,
            } => {
                let inner = self.go_type_name_for_decl_at_depth(*element, depth + 1)?;
                Ok(render_named_template(
                    &self.indexes.syntax.type_applications.optional,
                    &[("element", &inner)],
                ))
            }
            TypeConnective::Atom(AtomPayload::ResolvedByStructure(next))
            | TypeConnective::Atom(AtomPayload::ResolvedByName(next)) => {
                self.go_type_name_for_decl_at_depth(*next, depth + 1)
            }
            _ => Err(EmitError::MissingTypeRealization {
                target: declaration,
            }),
        }
    }

    fn list_element_type_name_for_list_port(&self, port: PortId) -> Result<String, EmitError> {
        let ty = self
            .dag
            .port(port)
            .value_type()
            .ok_or(EmitError::UntypedPort(port))?;
        let TypeConnective::Instantiation {
            template,
            arguments,
        } = &self.dag.declaration(ty.declaration).connective
        else {
            return Err(EmitError::UnsupportedBehavior(
                "list type rendering expected instantiated List".to_string(),
            ));
        };
        if !self
            .dag
            .list_template()
            .is_some_and(|list| list == *template)
        {
            return Err(EmitError::UnsupportedBehavior(
                "list type rendering expected List template".to_string(),
            ));
        }
        let [element] = arguments.as_slice() else {
            return Err(EmitError::UnsupportedBehavior(
                "List instantiation must carry one type argument".to_string(),
            ));
        };
        self.go_type_name_for_decl(element.value)
    }

    fn variant_payload_binding_for_variant(
        &self,
        variant_id: DeclarationId,
        binding_expr: &str,
    ) -> Result<Option<VariantPayloadBinding<String>>, EmitError> {
        let Some(shape) = variant_payload_shape(self.dag, variant_id) else {
            return Err(EmitError::UnsupportedBehavior(
                "variant payload expected a product declaration".to_string(),
            ));
        };
        Ok(match shape {
            VariantPayloadShape::Empty => None,
            VariantPayloadShape::PositionalSingle => {
                Some(VariantPayloadBinding::Direct(format!("{binding_expr}._0")))
            }
            VariantPayloadShape::NamedFields(field_labels) => {
                match self.indexes.clean_emission.variant_payload_field_access {
                    VariantPayloadFieldAccessRuleBinding::AccessFromPayloadBinding => {
                        Some(VariantPayloadBinding::Direct(binding_expr.to_string()))
                    }
                    VariantPayloadFieldAccessRuleBinding::OverrideNamedFieldsAtBindingSite => {
                        let fields = field_labels
                            .into_iter()
                            .map(|field_label| {
                                (field_label.clone(), format!("{binding_expr}.{field_label}"))
                            })
                            .collect();
                        Some(VariantPayloadBinding::Fields(fields))
                    }
                }
            }
        })
    }
}

fn go_behavior_result_port(behavior: &Behavior) -> PortId {
    match behavior {
        Behavior::Value(v) => v.result_port(),
        Behavior::Transform(t) => t.result_port(),
        Behavior::Branch(b) => b.result_port(),
        Behavior::Loop(l) => l.result_port(),
        Behavior::Bind(b) => b.result_port(),
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
        binary_op: syntax_field_string(fields, "binary_op", declaration)?,
        field_access: syntax_field_string(fields, "field_access", declaration)?,
        function_call: syntax_field_string(fields, "function_call", declaration)?,
    })
}

fn parse_function_syntax(
    dag: &Dag,
    declaration: DeclarationId,
) -> Result<FunctionSyntaxBinding, EmitError> {
    let fields = structural_fields_for_decl(dag, declaration)?;
    Ok(FunctionSyntaxBinding {
        definition: syntax_field_string(fields, "definition", declaration)?,
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
        struct_field_separator: syntax_field_string(fields, "struct_field_separator", declaration)?,
        variant_named_construction: syntax_field_string(
            fields,
            "variant_named_construction",
            declaration,
        )?,
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

fn require_field_decl_ref(
    fields: &[(String, FieldValue)],
    label: &str,
    declaration: DeclarationId,
) -> Result<DeclarationId, EmitError> {
    fields
        .iter()
        .find(|(field_label, _)| field_label == label)
        .and_then(|(_, value)| match value {
            FieldValue::Reference(id) => Some(*id),
            _ => None,
        })
        .ok_or(EmitError::MalformedRealization {
            declaration,
            detail:
                "realization data item is missing a required Reference field or has wrong shape — see lower_record_to_structural inhabitance check",
        })
}

fn require_field_string(
    fields: &[(String, FieldValue)],
    label: &str,
    declaration: DeclarationId,
) -> Result<String, EmitError> {
    fields
        .iter()
        .find(|(field_label, _)| field_label == label)
        .and_then(|(_, value)| match value {
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
        let access = fields
            .iter()
            .find(|(label, _)| label == "access")
            .ok_or(EmitError::MalformedRealization {
                declaration,
                detail: "FieldBinding.access is required",
            })
            .and_then(|(_, value)| parse_field_access(dag, value, declaration))?;
        if bindings
            .insert(dag_name, FieldBindingBinding { access })
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

fn parse_field_access(
    dag: &Dag,
    value: &FieldValue,
    declaration: DeclarationId,
) -> Result<GoFieldAccessBinding, EmitError> {
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
        Ok(GoFieldAccessBinding::DirectField(name))
    } else if *constructor == accessor_method {
        Ok(GoFieldAccessBinding::AccessorMethod(name))
    } else {
        Err(EmitError::MalformedRealization {
            declaration,
            detail: "FieldAccess constructor must be DirectField or AccessorMethod",
        })
    }
}

fn require_callable_strategy(
    dag: &Dag,
    fields: &[(String, FieldValue)],
    declaration: DeclarationId,
) -> Result<CallableStrategyBinding, EmitError> {
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
    let strategies = [
        ("ListEmpty", CallableStrategyBinding::ListEmpty),
        ("ListSingleton", CallableStrategyBinding::ListSingleton),
        ("ListCons", CallableStrategyBinding::ListCons),
        ("ListConcat", CallableStrategyBinding::ListConcat),
        ("ListLength", CallableStrategyBinding::ListLength),
        ("ListIsEmpty", CallableStrategyBinding::ListIsEmpty),
        ("ListFold", CallableStrategyBinding::ListFold),
        ("ListMap", CallableStrategyBinding::ListMap),
        ("ListFilter", CallableStrategyBinding::ListFilter),
        ("ListContains", CallableStrategyBinding::ListContains),
    ];
    for (label, binding) in strategies {
        let Some(variant_id) = named_variant_id(dag, "CallableStrategy", label) else {
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
            "CallableStrategy constructor must be ListEmpty/ListSingleton/ListCons/ListConcat/ListLength/ListIsEmpty/ListFold/ListMap/ListFilter/ListContains",
    })
}

/// Parse and validate the `parameters` list on a CallableRealization.
/// Mirrors emit_rust::require_parameter_dispositions exactly. Go's GC
/// rendering doesn't act on the result, but the shared schema requires
/// the field and validation makes arity/order drift unrepresentable.
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
/// `cons_variant`. Mirrors the same check in `rust_target` and
/// `python_target` — the typed `DeclarationRef` alone cannot express
/// "must be a variant of `target`", so reject-at-boundary on parse.
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
    if !variants.iter().any(|v| v.ty == binding.empty_variant) {
        return Err(EmitError::MalformedRealization {
            declaration,
            detail:
                "PatternRealization.empty_variant must be a variant of `target` — structural boundary check rejects unrelated DeclarationRefs",
        });
    }
    if !variants.iter().any(|v| v.ty == binding.cons_variant) {
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
    let FieldValue::Variant {
        constructor,
        payload,
    } = value
    else {
        return Err(EmitError::MalformedRealization {
            declaration,
            detail: "PatternRealization.strategy must be a PatternStrategy variant",
        });
    };
    if !payload.is_empty() {
        return Err(EmitError::MalformedRealization {
            declaration,
            detail: "PatternStrategy variants must not carry payload fields",
        });
    }
    let vector_list = named_variant_id(dag, "PatternStrategy", "VectorList").ok_or(
        EmitError::MalformedRealization {
            declaration,
            detail: "PatternStrategy.VectorList declaration was not found",
        },
    )?;
    if *constructor != vector_list {
        return Err(EmitError::MalformedRealization {
            declaration,
            detail: "PatternStrategy constructor must be VectorList",
        });
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

fn require_memory_model(
    dag: &Dag,
    fields: &[(String, FieldValue)],
    declaration: DeclarationId,
) -> Result<MemoryModelBinding, EmitError> {
    let value = fields
        .iter()
        .find(|(label, _)| label == "memory")
        .map(|(_, value)| value)
        .ok_or(EmitError::MalformedTargetSyntax {
            declaration,
            detail: "TargetExecutionModel is missing required `memory` field",
        })?;
    let FieldValue::Variant {
        constructor,
        payload,
    } = value
    else {
        return Err(EmitError::MalformedTargetSyntax {
            declaration,
            detail: "TargetExecutionModel.memory must be a MemoryModel variant",
        });
    };
    if !payload.is_empty() {
        return Err(EmitError::MalformedTargetSyntax {
            declaration,
            detail: "MemoryModel variants must not carry payload fields",
        });
    }
    let variants = [
        ("ValueOnly", MemoryModelBinding::ValueOnly),
        ("GarbageCollected", MemoryModelBinding::GarbageCollected),
        ("RefCounted", MemoryModelBinding::RefCounted),
        ("OwnershipBased", MemoryModelBinding::OwnershipBased),
    ];
    for (label, binding) in variants {
        let Some(variant_id) = named_variant_id(dag, "MemoryModel", label) else {
            return Err(EmitError::MalformedTargetSyntax {
                declaration,
                detail: "MemoryModel variant declaration was not found",
            });
        };
        if *constructor == variant_id {
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
    let value = fields
        .iter()
        .find(|(label, _)| label == "scope")
        .map(|(_, value)| value)
        .ok_or(EmitError::MalformedTargetSyntax {
            declaration,
            detail: "TargetExecutionModel is missing required `scope` field",
        })?;
    let FieldValue::Variant {
        constructor,
        payload,
    } = value
    else {
        return Err(EmitError::MalformedTargetSyntax {
            declaration,
            detail: "TargetExecutionModel.scope must be a ScopeModel variant",
        });
    };
    if !payload.is_empty() {
        return Err(EmitError::MalformedTargetSyntax {
            declaration,
            detail: "ScopeModel variants must not carry payload fields",
        });
    }
    let variants = [
        ("LexicalScoping", ScopeModelBinding::LexicalScoping),
        ("DynamicScoping", ScopeModelBinding::DynamicScoping),
    ];
    for (label, binding) in variants {
        let Some(variant_id) = named_variant_id(dag, "ScopeModel", label) else {
            return Err(EmitError::MalformedTargetSyntax {
                declaration,
                detail: "ScopeModel variant declaration was not found",
            });
        };
        if *constructor == variant_id {
            return Ok(binding);
        }
    }
    Err(EmitError::MalformedTargetSyntax {
        declaration,
        detail: "TargetExecutionModel.scope must be LexicalScoping or DynamicScoping",
    })
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

fn render_named_template(template: &str, bindings: &[(&str, &str)]) -> String {
    let bindings: HashMap<&str, &str> = bindings.iter().copied().collect();
    let chars: Vec<char> = template.chars().collect();
    let mut rendered = String::with_capacity(template.len());
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '{' => {
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

fn find_resolved_branch_path(branch: &BranchNode, variant_id: DeclarationId) -> Option<&Path> {
    branch.paths.iter().find(|path| match path.pattern {
        BranchPattern::ResolvedVariant(id) => id == variant_id,
        BranchPattern::UnresolvedVariant { .. } => false,
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

fn render_value(v: &crate::dag::ValueNode, literals: &LiteralSyntaxBinding) -> String {
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

fn is_bootstrap_file(file: &str) -> bool {
    file.starts_with("dsl/std/")
        || file.starts_with("src/v3/std/")
        || file.starts_with("src/v3/spec/")
        || file.starts_with("src/v3/compiler/")
}

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
            TypeConnective::Atom(AtomPayload::ResolvedByStructure(next))
            | TypeConnective::Atom(AtomPayload::ResolvedByName(next)) => current = *next,
            _ => return Ok(current),
        }
    }
    Err(EmitError::UnsupportedBehavior(
        "port type walk exceeded depth 32 — likely a cycle".to_string(),
    ))
}

fn walk_to_disj(dag: &Dag, start: DeclarationId) -> Option<DeclarationId> {
    let mut current = start;
    for _ in 0..32 {
        match &dag.declaration(current).connective {
            TypeConnective::Disj { .. } => return Some(current),
            TypeConnective::Cardinality {
                bound: CardinalityBound::AtMostOne,
                ..
            } => return dag.optional_match_disj(current),
            TypeConnective::Instantiation { template, .. } => current = *template,
            TypeConnective::Atom(AtomPayload::ResolvedByStructure(next))
            | TypeConnective::Atom(AtomPayload::ResolvedByName(next)) => current = *next,
            _ => return None,
        }
    }
    None
}

fn algebra_field_for_operator(
    dag: &Dag,
    operand_type_id: DeclarationId,
    op: OperatorKind,
) -> Result<DeclarationId, EmitError> {
    let Some(algebra_conj_id) = walk_to_algebra_conj(dag, operand_type_id) else {
        return canonical_operator_field(dag, op);
    };
    let field_label = crate::operators::algebra_field_name(op);
    let children = match &dag.declaration(algebra_conj_id).connective {
        TypeConnective::Conj { children } => children,
        _ => unreachable!("walk_to_algebra_conj returned a non-Conj"),
    };
    if let Some(field) = children.iter().find(|field| field.label == field_label) {
        return Ok(field.ty);
    }
    canonical_operator_field(dag, op)
}

fn walk_to_algebra_conj(dag: &Dag, start: DeclarationId) -> Option<DeclarationId> {
    let mut current = start;
    for _ in 0..32 {
        match &dag.declaration(current).connective {
            TypeConnective::Conj { .. } => return Some(current),
            TypeConnective::Instantiation { template, .. } => current = *template,
            TypeConnective::Atom(AtomPayload::ResolvedByStructure(next))
            | TypeConnective::Atom(AtomPayload::ResolvedByName(next)) => current = *next,
            _ => return None,
        }
    }
    None
}

fn canonical_operator_field(dag: &Dag, op: OperatorKind) -> Result<DeclarationId, EmitError> {
    let ordered_ring = dag.declaration_by_name("OrderedRing").ok_or_else(|| {
        EmitError::UnsupportedBehavior(
            "bootstrap is missing the canonical `OrderedRing` declaration".to_string(),
        )
    })?;
    let TypeConnective::Conj { children } = &ordered_ring.connective else {
        return Err(EmitError::UnsupportedBehavior(
            "`OrderedRing` does not lower to a Conj declaration".to_string(),
        ));
    };
    let field_label = crate::operators::algebra_field_name(op);
    children
        .iter()
        .find(|field| field.label == field_label)
        .map(|field| field.ty)
        .ok_or_else(|| {
            EmitError::UnsupportedBehavior(format!(
                "`OrderedRing` has no canonical field labeled {field_label}"
            ))
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile_to_dag;

    // DELETED: go_gc_targets_skip_rendering_model_loading
    //
    // This test deliberately corrupted `go_rendering` and asserted
    // emission succeeded — codifying "declared target fact is
    // non-authoritative" as a unit-tested property. That directly
    // violates INVARIANTS.md E-6 (no target-spec field without a
    // same-PR consumer). Removed so the go_rendering authority is
    // allowed to become load-bearing; if a future test needs to
    // assert "emission doesn't rely on X for GC targets," it should
    // do so by driving emission on a properly-populated spec, not
    // by corrupting a field and asserting we ignore it.

    #[test]
    fn go_struct_fields_render_with_separators() {
        let dag =
            compile_to_dag("type Pair { left: Int right: Int }", "pair.v3").expect("compiles");
        let rendered = emit_module(&dag, EmitTarget::Go)
            .expect("go emitter should render struct")
            .text;
        assert!(
            rendered.contains("type Pair struct { left int64; right int64 }"),
            "got: {rendered}"
        );
    }

    #[test]
    fn go_program_emission_excludes_internal_pipeline_authority() {
        let dag = compile_to_dag(
            "fn double(x: Int) -> Int = x + x\nlet result: Int = double(20)\n",
            "program.v3",
        )
        .expect("compiles");
        let rendered = emit(&dag, EmitTarget::Go)
            .expect("go emitter should render program")
            .text;
        assert!(!rendered.contains("parse_realization"), "got: {rendered}");
        assert!(!rendered.contains("DeclarationRef"), "got: {rendered}");
    }

    #[test]
    fn go_fold_marks_loop_item_used_inside_the_loop() {
        let dag = compile_to_dag(
            "let total: Int = fold(cons(1, singleton(2)), 0, |acc, x| acc + x)",
            "fold_item_scope.v3",
        )
        .expect("compiles");
        let rendered = emit(&dag, EmitTarget::Go)
            .expect("go emitter should render fold")
            .text;
        assert!(
            rendered.contains("for _, __foldItem := range"),
            "got: {rendered}"
        );
        assert!(
            rendered.contains("{ _ = __foldItem; __foldAcc = "),
            "got: {rendered}"
        );
        assert!(
            !rendered.contains("_ = (int64)(__foldItem)"),
            "got: {rendered}"
        );
    }

    #[test]
    fn go_map_and_filter_mark_unused_loop_items_used_inside_the_loop() {
        let dag = compile_to_dag(
            "let total: Int = length(filter(map(cons(1, singleton(2)), |x| x + 1), |y| y == 2))",
            "map_filter_loop_item.v3",
        )
        .expect("compiles");
        let rendered = emit(&dag, EmitTarget::Go)
            .expect("go emitter should render map/filter")
            .text;
        assert!(
            rendered.contains("for _, __mapItem := range") && rendered.contains("_ = __mapItem;"),
            "got: {rendered}"
        );
        assert!(
            rendered.contains("for _, __filterItem := range")
                && rendered.contains("_ = __filterItem;"),
            "got: {rendered}"
        );
    }

    #[test]
    fn go_multi_field_variant_payload_binding_uses_the_variant_value() {
        // Originally exercised via a recursive `count`, which lowered
        // to `Behavior::Loop` and was silently emitted (the loop-as-
        // body-result collapse). emit_go now fail-closes on Loop, so
        // we use a non-recursive body that still exercises Cons(payload)
        // payload-binding rendering.
        let dag = compile_to_dag(
            "type IntList = Empty | Cons { head: Int, tail: IntList }\nfn head_or_zero(list: IntList) -> Int = match list { Empty => 0, Cons(payload) => payload.head }\n",
            "variant_payload_binding.v3",
        )
        .expect("compiles");
        let rendered = emit_module(&dag, EmitTarget::Go)
            .expect("go emitter should render match")
            .text;
        assert!(
            rendered.contains("case Cons: return (v).head"),
            "got: {rendered}"
        );
        assert!(!rendered.contains("payload := v;"), "got: {rendered}");
    }

    #[test]
    fn go_named_single_field_variant_payload_binding_uses_the_variant_value() {
        let dag = compile_to_dag(
            "type Point { x: Int y: Int }\ntype Wrapped = Wrap { inner: Point } | Empty\nfn unwrap_or_zero(w: Wrapped) -> Int = match w { Wrap(payload) => payload.inner.x, Empty => 0 }\n",
            "variant_payload_named_single.v3",
        )
        .expect("compiles");
        let rendered = emit_module(&dag, EmitTarget::Go)
            .expect("go emitter should render match")
            .text;
        assert!(
            rendered.contains("case Wrap: return ((v).inner).x"),
            "got: {rendered}"
        );
        assert!(!rendered.contains("payload := v.inner;"), "got: {rendered}");
    }
}
