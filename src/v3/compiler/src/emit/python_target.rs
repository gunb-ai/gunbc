use std::collections::HashMap;

use super::{
    algebra_field_for_operator_shared,
    collection_ops_method_contract::require_method_template_contract_dag_method,
    dag_needs_div_error_prelude, div_prelude_reserved_name_collision,
    method_emit_template_variant_label, optional_match_variant_roles, parse_pattern_strategy,
    primitive_type_id_for_port_shared, walk_to_disj, EmitMode, PatternStrategyBinding,
    SharedEmitLookupError, SourceFilteringBinding, VariantPayloadBinding,
    VariantPayloadFieldAccessRuleBinding,
};
use crate::dag::{
    ArrowBody, AtomPayload, Behavior, BindNode, BranchNode, BranchPattern, DeclarationId, Field,
    FieldValue, LiteralBits, Path, PortId, TemplateArgument, TransformNode, TransformTarget,
    TypeConnective,
};
use crate::operators::OperatorKind;
use crate::variant_payload::{
    variant_payload_shape, VariantPayloadShape, VariantPayloadShapeLookup,
};
use crate::Dag;

#[derive(Debug, Clone)]
pub enum EmitPythonError {
    MissingMeta(&'static str),
    MissingSpec(&'static str),
    MissingTypeRealization {
        target: DeclarationId,
    },
    MissingOperatorRealization {
        target: DeclarationId,
        op: DeclarationId,
    },
    MalformedSpec {
        declaration: DeclarationId,
        detail: &'static str,
    },
    UntypedPort(PortId),
    Unsupported(String),
    UnresolvedBranchPattern {
        variant_name: String,
    },
    /// Two realization data items in the loaded spec set targeted the
    /// same key (type, (operand_type, op), or callable). The Python
    /// loader used to silently overwrite the first entry when this
    /// happened. Now fail-closed so spec collisions surface.
    DuplicateRealization {
        declaration: DeclarationId,
        detail: &'static str,
    },
}

pub(crate) type EmitPythonMode = EmitMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CallableStrategyBinding {
    Empty,
    Singleton,
    Cons,
    Concat,
    Length,
    IsEmpty,
    Fold,
    Map,
    Filter,
    Contains,
}

#[derive(Debug, Clone)]
struct PythonSyntax {
    field_access: String,
    function_call: String,
    closure: String,
    empty_list: String,
    list_literal: String,
    cons: String,
    concat: String,
    length: String,
    is_empty: String,
    fold: String,
    map: String,
    filter: String,
    contains: String,
    optional: String,
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
struct PythonTarget {
    memory: MemoryModelBinding,
    scope: ScopeModelBinding,
}

/// Typed read of `data python_clean_emission: CleanEmissionContract`
/// from `src/v3/spec/python.dag` — the portion this pilot consumes
/// (E-5 / Lane 1 Stage 1c PR 3). Other contract rules land here as
/// their consumers wire in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CleanEmissionContractBinding {
    pattern_bindings: PatternBindingRuleBinding,
    variant_payload_field_access: VariantPayloadFieldAccessRuleBinding,
}

/// Python-valid slice of `std.clean_emission.PatternBindingRule`.
/// Parsed in `CleanEmissionContractBinding::build`, which rejects
/// target-invalid constructors instead of letting the renderer
/// normalize them later. emit_python does not emit Python's native
/// `match` statement — it substitutes an extraction expression
/// (`__match._0` / `__match`) at every payload-binding port
/// reference inside the rendered arm body, so no binding identifier
/// is ever written at a pattern site. `NotApplicablePatternBinding`
/// is the only valid rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PatternBindingRuleBinding {
    NotApplicable,
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
struct PythonIndexes {
    types: HashMap<DeclarationId, String>,
    type_instantiations: HashMap<DeclarationId, String>,
    operators: HashMap<(DeclarationId, DeclarationId), String>,
    callables: HashMap<DeclarationId, CallableStrategyBinding>,
    patterns: HashMap<DeclarationId, PatternRealizationBinding>,
    syntax: PythonSyntax,
    target: PythonTarget,
    /// Source exclusion policy loaded from
    /// `data python_source_filtering: ShapeATargetSourceFiltering`.
    source_filtering: SourceFilteringBinding,
    /// The Python clean-emission contract loaded from `data
    /// python_clean_emission: CleanEmissionContract` (E-5 / Lane 1
    /// Stage 1c PR 3). Rule variants dispatch inside the emitter so
    /// emitted Python passes `python3 -m py_compile` by
    /// construction. For the current pilot only `pattern_bindings`
    /// is dispatched on; other fields are authored-but-unread until
    /// Lane 1d/1e consolidation.
    clean_emission: CleanEmissionContractBinding,
}

#[derive(Debug, Clone, Default)]
struct RenderLocals {
    names: HashMap<PortId, String>,
    payload_bindings: HashMap<PortId, VariantPayloadBinding<String>>,
}

struct Ctx<'a> {
    dag: &'a Dag,
    indexes: &'a PythonIndexes,
    bound_names: &'a HashMap<PortId, String>,
}

impl PythonIndexes {
    fn build(dag: &Dag) -> Result<Self, EmitPythonError> {
        let type_meta = dag
            .type_realization_meta()
            .ok_or(EmitPythonError::MissingMeta("TypeRealization"))?;
        let type_instantiation_meta = dag
            .type_instantiation_realization_meta()
            .ok_or(EmitPythonError::MissingMeta("TypeInstantiationRealization"))?;
        let operator_meta = dag
            .operator_realization_meta()
            .ok_or(EmitPythonError::MissingMeta("OperatorRealization"))?;
        let callable_meta = dag
            .callable_realization_meta()
            .ok_or(EmitPythonError::MissingMeta("CallableRealization"))?;
        let pattern_meta = dag
            .pattern_realization_meta()
            .ok_or(EmitPythonError::MissingMeta("PatternRealization"))?;
        let python_language = dag
            .python_language_spec()
            .ok_or(EmitPythonError::MissingSpec("python_language"))?;
        let mut types = HashMap::new();
        let mut type_instantiations = HashMap::new();
        let mut operators = HashMap::new();
        let mut callables = HashMap::new();
        let mut patterns = HashMap::new();

        for decl in dag.declarations() {
            let Some(meta_tag) = decl.meta_tag else {
                continue;
            };
            // Only act on declarations tagged with one of the four
            // realization meta types. Skip silently when the meta-tag
            // is for a different category — that's structural, not a
            // spec inconsistency.
            let is_realization_meta = meta_tag == type_meta
                || meta_tag == type_instantiation_meta
                || meta_tag == operator_meta
                || meta_tag == callable_meta
                || meta_tag == pattern_meta;
            if !is_realization_meta {
                continue;
            }
            // A data item tagged with a realization meta-type MUST
            // have a Structural value_body. If it's Unparsed, the
            // inhabitance check let a malformed spec entry through —
            // fail-closed so the spec inconsistency surfaces loudly
            // (matches emit_rust + emit_go behavior).
            let Some(fields) = structural_fields(decl) else {
                return Err(EmitPythonError::MalformedSpec {
                    declaration: decl.id,
                    detail:
                        "realization data item has no Structural value_body — bootstrap inhabitance check missed a malformed spec entry",
                });
            };
            let language = require_field_decl_ref(fields, "language", decl.id)?;
            if language != python_language {
                continue;
            }
            if meta_tag == type_meta {
                let target = require_field_decl_ref(fields, "target", decl.id)?;
                let carrier = require_field_string(fields, "carrier", decl.id)?;
                if types.insert(target, carrier).is_some() {
                    return Err(EmitPythonError::DuplicateRealization {
                        declaration: decl.id,
                        detail: "two TypeRealization data items target the same declaration",
                    });
                }
            } else if meta_tag == type_instantiation_meta {
                let target = require_field_decl_ref(fields, "target", decl.id)?;
                let carrier = require_field_string(fields, "carrier", decl.id)?;
                if type_instantiations.insert(target, carrier).is_some() {
                    return Err(EmitPythonError::DuplicateRealization {
                        declaration: decl.id,
                        detail: "two TypeInstantiationRealization data items target the same declaration",
                    });
                }
            } else if meta_tag == operator_meta {
                let target = require_field_decl_ref(fields, "target", decl.id)?;
                let op = require_field_decl_ref(fields, "op", decl.id)?;
                let carrier = require_field_string(fields, "carrier", decl.id)?;
                if !carrier.contains("{lhs}") || !carrier.contains("{rhs}") {
                    return Err(EmitPythonError::MalformedSpec {
                        declaration: decl.id,
                        detail: "OperatorRealization carrier must be a full-expression template containing {lhs} and {rhs}",
                    });
                }
                if operators.insert((target, op), carrier).is_some() {
                    return Err(EmitPythonError::DuplicateRealization {
                        declaration: decl.id,
                        detail:
                            "two OperatorRealization data items share the same (target, op) pair",
                    });
                }
            } else if meta_tag == callable_meta {
                let target = require_field_decl_ref(fields, "target", decl.id)?;
                let strategy = parse_callable_strategy(dag, fields, decl.id)?;
                if callables.insert(target, strategy).is_some() {
                    return Err(EmitPythonError::DuplicateRealization {
                        declaration: decl.id,
                        detail:
                            "two CallableRealization data items target the same callable declaration",
                    });
                }
            } else {
                let target = require_field_decl_ref(fields, "target", decl.id)?;
                let binding = parse_pattern_realization(dag, fields, decl.id)?;
                validate_pattern_roles(dag, target, &binding, decl.id)?;
                if patterns.insert(target, binding).is_some() {
                    return Err(EmitPythonError::DuplicateRealization {
                        declaration: decl.id,
                        detail:
                            "two PatternRealization data items target the same structural sum declaration",
                    });
                }
            }
        }

        let language_fields = structural_fields_for_decl(dag, python_language)?;
        let expressions = require_field_decl_ref(language_fields, "expressions", python_language)?;
        let collections =
            require_field_decl_ref(language_fields, "collection_ops", python_language)?;
        let type_apps =
            require_field_decl_ref(language_fields, "type_applications", python_language)?;
        let target_decl = dag
            .python_target_spec()
            .ok_or(EmitPythonError::MissingSpec("python_target"))?;
        let target_fields = structural_fields_for_decl(dag, target_decl)?;
        let source_filtering = SourceFilteringBinding::build(
            dag,
            dag.python_source_filtering_spec()
                .ok_or(EmitPythonError::MissingSpec("python_source_filtering"))?,
        )
        .map_err(|err| match err {
            super::rust_target::EmitError::MalformedTargetSyntax {
                declaration,
                detail,
            } => EmitPythonError::MalformedSpec {
                declaration,
                detail,
            },
            other => EmitPythonError::Unsupported(format!("{other:?}")),
        })?;

        let syntax = PythonSyntax {
            field_access: require_field_string(
                structural_fields_for_decl(dag, expressions)?,
                "field_access",
                expressions,
            )?,
            function_call: require_field_string(
                structural_fields_for_decl(dag, expressions)?,
                "function_call",
                expressions,
            )?,
            closure: require_field_string(
                structural_fields_for_decl(dag, expressions)?,
                "closure",
                expressions,
            )?,
            empty_list: require_field_string(
                structural_fields_for_decl(dag, collections)?,
                "empty_list",
                collections,
            )?,
            list_literal: require_field_string(
                structural_fields_for_decl(dag, collections)?,
                "list_literal",
                collections,
            )?,
            cons: require_field_string(
                structural_fields_for_decl(dag, collections)?,
                "cons",
                collections,
            )?,
            concat: {
                let cfields = structural_fields_for_decl(dag, collections)?;
                let concat_method_decl =
                    dag.concat_method_decl()
                        .ok_or(EmitPythonError::MalformedSpec {
                            declaration: collections,
                            detail: "internal: concat_method missing from std.methods registry",
                        })?;
                let id = require_field_decl_ref(cfields, "concat_contract", collections)?;
                require_method_template_contract_dag_method(
                    dag,
                    id,
                    "concat_contract",
                    concat_method_decl,
                )
                .map_err(|detail| EmitPythonError::MalformedSpec {
                    declaration: id,
                    detail,
                })?;
                method_contract_single_emit_template_string(dag, id)?
            },
            length: {
                let cfields = structural_fields_for_decl(dag, collections)?;
                let length_method_decl =
                    dag.length_method_decl()
                        .ok_or(EmitPythonError::MalformedSpec {
                            declaration: collections,
                            detail: "internal: length_method missing from std.methods registry",
                        })?;
                let id = require_field_decl_ref(cfields, "length_contract", collections)?;
                require_method_template_contract_dag_method(
                    dag,
                    id,
                    "length_contract",
                    length_method_decl,
                )
                .map_err(|detail| EmitPythonError::MalformedSpec {
                    declaration: id,
                    detail,
                })?;
                method_contract_single_emit_template_string(dag, id)?
            },
            is_empty: {
                let cfields = structural_fields_for_decl(dag, collections)?;
                let is_empty_method_decl =
                    dag.is_empty_method_decl()
                        .ok_or(EmitPythonError::MalformedSpec {
                            declaration: collections,
                            detail: "internal: is_empty_method missing from std.methods registry",
                        })?;
                let id = require_field_decl_ref(cfields, "is_empty_contract", collections)?;
                require_method_template_contract_dag_method(
                    dag,
                    id,
                    "is_empty_contract",
                    is_empty_method_decl,
                )
                .map_err(|detail| EmitPythonError::MalformedSpec {
                    declaration: id,
                    detail,
                })?;
                method_contract_single_emit_template_string(dag, id)?
            },
            fold: {
                let cfields = structural_fields_for_decl(dag, collections)?;
                let fold_method_decl =
                    dag.fold_method_decl()
                        .ok_or(EmitPythonError::MalformedSpec {
                            declaration: collections,
                            detail: "internal: fold_method missing from std.methods registry",
                        })?;
                let fold_contract = require_field_decl_ref(cfields, "fold_contract", collections)?;
                require_method_template_contract_dag_method(
                    dag,
                    fold_contract,
                    "fold_contract",
                    fold_method_decl,
                )
                .map_err(|detail| EmitPythonError::MalformedSpec {
                    declaration: fold_contract,
                    detail,
                })?;
                method_contract_single_emit_template_string(dag, fold_contract)?
            },
            map: {
                let cfields = structural_fields_for_decl(dag, collections)?;
                let map_method_decl =
                    dag.map_method_decl()
                        .ok_or(EmitPythonError::MalformedSpec {
                            declaration: collections,
                            detail: "internal: map_method missing from std.methods registry",
                        })?;
                let map_contract = require_field_decl_ref(cfields, "map_contract", collections)?;
                require_method_template_contract_dag_method(
                    dag,
                    map_contract,
                    "map_contract",
                    map_method_decl,
                )
                .map_err(|detail| EmitPythonError::MalformedSpec {
                    declaration: map_contract,
                    detail,
                })?;
                method_contract_single_emit_template_string(dag, map_contract)?
            },
            filter: require_field_string(
                structural_fields_for_decl(dag, collections)?,
                "filter",
                collections,
            )?,
            contains: require_field_string(
                structural_fields_for_decl(dag, collections)?,
                "contains",
                collections,
            )?,
            optional: require_field_string(
                structural_fields_for_decl(dag, type_apps)?,
                "optional",
                type_apps,
            )?,
        };

        let target = PythonTarget {
            memory: require_memory_model(dag, target_fields, target_decl)?,
            scope: require_scope_model(dag, target_fields, target_decl)?,
        };

        let clean_emission = CleanEmissionContractBinding::build(dag)?;

        Ok(Self {
            types,
            type_instantiations,
            operators,
            callables,
            patterns,
            syntax,
            target,
            source_filtering,
            clean_emission,
        })
    }
}

impl CleanEmissionContractBinding {
    /// Parse the portion of `data python_clean_emission:
    /// CleanEmissionContract` this pilot consumes. Mirrors
    /// emit_rust / emit_go's `CleanEmissionContractBinding::build`.
    fn build(dag: &Dag) -> Result<Self, EmitPythonError> {
        let declaration = dag
            .python_clean_emission_spec()
            .ok_or(EmitPythonError::MissingSpec("python_clean_emission"))?;
        let decl = dag.declaration(declaration);
        let fields = structural_fields(decl).ok_or(EmitPythonError::MalformedSpec {
            declaration,
            detail: "python_clean_emission must be a structural data item",
        })?;
        let pattern_bindings_value = fields
            .iter()
            .find(|(label, _)| label == "pattern_bindings")
            .map(|(_, value)| value)
            .ok_or(EmitPythonError::MalformedSpec {
                declaration,
                detail: "python_clean_emission is missing required `pattern_bindings` field",
            })?;
        let pattern_bindings =
            parse_pattern_binding_rule(dag, pattern_bindings_value, declaration)?;
        let variant_payload_field_access_value = fields
            .iter()
            .find(|(label, _)| label == "variant_payload_field_access")
            .map(|(_, value)| value)
            .ok_or(EmitPythonError::MalformedSpec {
                declaration,
                detail:
                    "python_clean_emission is missing required `variant_payload_field_access` field",
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

/// Parse `python_clean_emission.pattern_bindings` via the typed
/// `PatternBindingRuleVariants` cache on `Dag` (Lane 1 Stage 1c
/// PR 2.5). NOT `named_variant_id` — the typed cache is the single
/// authority that lets emit_rust / emit_go / emit_python share the
/// same resolution path without each reconstructing the same fact.
///
/// Rejects every variant except `NotApplicablePatternBinding`: the
/// other rules (`EmitBindingAlways` / `EmitUnderscoreWhenUnused` /
/// `EmitPrefixedUnderscoreWhenUnused`) describe pattern-site
/// binding elisions that emit_python has no pattern site to apply
/// them to — the emitter substitutes an extraction expression at
/// every port reference in the rendered arm body, so the binding
/// identifier never appears in source by construction.
fn parse_pattern_binding_rule(
    dag: &Dag,
    value: &FieldValue,
    declaration: DeclarationId,
) -> Result<PatternBindingRuleBinding, EmitPythonError> {
    let FieldValue::Variant {
        constructor,
        payload,
    } = value
    else {
        return Err(EmitPythonError::MalformedSpec {
            declaration,
            detail: "python_clean_emission.pattern_bindings must be a PatternBindingRule variant",
        });
    };
    if !payload.is_empty() {
        return Err(EmitPythonError::MalformedSpec {
            declaration,
            detail: "PatternBindingRule variants must not carry payload fields",
        });
    }
    let variants = dag.pattern_binding_rule_variants();
    let emit_always = variants.emit_always.ok_or(EmitPythonError::MalformedSpec {
        declaration,
        detail: "PatternBindingRule.EmitBindingAlways declaration was not found",
    })?;
    let emit_underscore = variants
        .emit_underscore
        .ok_or(EmitPythonError::MalformedSpec {
            declaration,
            detail: "PatternBindingRule.EmitUnderscoreWhenUnused declaration was not found",
        })?;
    let emit_prefixed = variants
        .emit_prefixed
        .ok_or(EmitPythonError::MalformedSpec {
            declaration,
            detail: "PatternBindingRule.EmitPrefixedUnderscoreWhenUnused declaration was not found",
        })?;
    let not_applicable = variants
        .not_applicable
        .ok_or(EmitPythonError::MalformedSpec {
            declaration,
            detail: "PatternBindingRule.NotApplicablePatternBinding declaration was not found",
        })?;
    if *constructor == not_applicable {
        Ok(PatternBindingRuleBinding::NotApplicable)
    } else if *constructor == emit_always {
        Err(EmitPythonError::MalformedSpec {
            declaration,
            detail: "python_clean_emission.pattern_bindings cannot use PatternBindingRule.EmitBindingAlways; Python only supports NotApplicablePatternBinding",
        })
    } else if *constructor == emit_underscore {
        Err(EmitPythonError::MalformedSpec {
            declaration,
            detail: "python_clean_emission.pattern_bindings cannot use PatternBindingRule.EmitUnderscoreWhenUnused; Python only supports NotApplicablePatternBinding",
        })
    } else if *constructor == emit_prefixed {
        Err(EmitPythonError::MalformedSpec {
            declaration,
            detail: "python_clean_emission.pattern_bindings cannot use PatternBindingRule.EmitPrefixedUnderscoreWhenUnused; Python only supports NotApplicablePatternBinding",
        })
    } else {
        Err(EmitPythonError::MalformedSpec {
            declaration,
            detail:
                "python_clean_emission.pattern_bindings constructor is not a known PatternBindingRule variant",
        })
    }
}

fn parse_variant_payload_field_access_rule(
    dag: &Dag,
    value: &FieldValue,
    declaration: DeclarationId,
) -> Result<VariantPayloadFieldAccessRuleBinding, EmitPythonError> {
    let FieldValue::Variant {
        constructor,
        payload,
    } = value
    else {
        return Err(EmitPythonError::MalformedSpec {
            declaration,
            detail:
                "python_clean_emission.variant_payload_field_access must be a VariantPayloadFieldAccessRule variant",
        });
    };
    if !payload.is_empty() {
        return Err(EmitPythonError::MalformedSpec {
            declaration,
            detail: "VariantPayloadFieldAccessRule variants must not carry payload fields",
        });
    }
    let variants = dag.variant_payload_field_access_rule_variants();
    let access_from_payload_binding =
        variants
            .access_from_payload_binding
            .ok_or(EmitPythonError::MalformedSpec {
            declaration,
            detail:
                "VariantPayloadFieldAccessRule.AccessFromPayloadBinding declaration was not found",
        })?;
    let override_named_fields_at_binding_site = variants
        .override_named_fields_at_binding_site
        .ok_or(EmitPythonError::MalformedSpec {
            declaration,
            detail: "VariantPayloadFieldAccessRule.OverrideNamedFieldsAtBindingSite declaration was not found",
        })?;
    if *constructor == access_from_payload_binding {
        Ok(VariantPayloadFieldAccessRuleBinding::AccessFromPayloadBinding)
    } else if *constructor == override_named_fields_at_binding_site {
        Err(EmitPythonError::MalformedSpec {
            declaration,
            detail: "python_clean_emission.variant_payload_field_access cannot use VariantPayloadFieldAccessRule.OverrideNamedFieldsAtBindingSite; Python requires AccessFromPayloadBinding for native match carriers",
        })
    } else {
        Err(EmitPythonError::MalformedSpec {
            declaration,
            detail: "python_clean_emission.variant_payload_field_access constructor is not a known VariantPayloadFieldAccessRule variant",
        })
    }
}

pub(crate) fn emit_python_with_mode(
    dag: &Dag,
    mode: EmitPythonMode,
) -> Result<String, EmitPythonError> {
    let indexes = PythonIndexes::build(dag)?;
    if indexes.target.memory != MemoryModelBinding::GarbageCollected {
        return Err(EmitPythonError::Unsupported(format!(
            "emit_python requires python_target.memory = GarbageCollected, found {:?}",
            indexes.target.memory
        )));
    }
    if indexes.target.scope != ScopeModelBinding::LexicalScoping {
        return Err(EmitPythonError::Unsupported(format!(
            "emit_python requires python_target.scope = LexicalScoping, found {:?}",
            indexes.target.scope
        )));
    }
    let type_decls: Vec<_> = dag
        .declarations()
        .iter()
        .filter(|decl| !indexes.source_filtering.excludes(&decl.span.file))
        .filter(|decl| decl.name.is_some())
        .filter(|decl| !super::substrate_result_type_decl_suppressed_for_emit(dag, decl))
        .filter(|decl| !super::substrate_div_error_type_decl_suppressed_for_emit(dag, decl))
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
    let top_level_binds: Vec<&BindNode> = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .filter(|bind| !indexes.source_filtering.excludes(&bind.span.file))
        .filter(|bind| bind.params.is_empty())
        .collect();

    if mode == EmitPythonMode::Program && top_level_binds.is_empty() {
        return Err(EmitPythonError::Unsupported(
            "emit_python requires at least one top-level value Bind".to_string(),
        ));
    }
    if mode == EmitPythonMode::Module && !top_level_binds.is_empty() {
        return Err(EmitPythonError::Unsupported(
            "emit_python module mode does not support top-level value Binds".to_string(),
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
    };

    let mut sections = vec![
        "from __future__ import annotations".to_string(),
        "from dataclasses import dataclass".to_string(),
        "import enum".to_string(),
        "import types".to_string(),
        "import typing".to_string(),
        format!(
            "# ownership skipped: {:?} / {:?}",
            indexes.target.memory, indexes.target.scope
        ),
        "__T = typing.TypeVar(\"__T\")".to_string(),
        "__U = typing.TypeVar(\"__U\")".to_string(),
        "def __v3_fold(items: list[typing.Any], init: typing.Any, fn: typing.Callable[[typing.Any, typing.Any], typing.Any]) -> typing.Any:\n    acc = init\n    for item in items:\n        acc = fn(acc, item)\n    return acc".to_string(),
        "def __v3_unreachable(label: str) -> typing.NoReturn:\n    raise ValueError(label)".to_string(),
    ];

    let needs_int_div_prelude =
        dag_needs_div_error_prelude(dag, &type_decls, &top_level_binds, &function_decls);
    if let (true, Some(name)) = (
        needs_int_div_prelude,
        div_prelude_reserved_name_collision(
            dag,
            type_decls.iter(),
            function_decls.iter(),
            top_level_binds.iter(),
            "__v3_idiv",
        ),
    ) {
        return Err(EmitPythonError::Unsupported(format!(
            "Python checked-division prelude would collide with user-defined `{name}`"
        )));
    }
    for decl in type_decls {
        sections.push(ctx.render_type_declaration(decl)?);
    }
    if needs_int_div_prelude {
        // `std.error_primitives` is filtered out of `type_decls`; v3 `DivError` + checked `/` are
        // prelude-only (names align with `python.dag` / `python_int_div` carrier for `__v3_idiv`).
        //
        // Dissolution trigger (M1 scaffold): delete when `dsl/std/error_primitives` emits through
        // the normal type-decl path (no separate prelude strings).
        //
        // M2: gate on emitted `__v3_idiv(...)` (or equivalent) so division-free programs skip
        // this block.
        sections.push(
            "class DivError(enum.IntEnum):\n    DivideByZero = 0\n    Overflow = 1".to_string(),
        );
        sections.push(
            "def __v3_idiv(a: int, b: int) -> typing.Union[typing.Tuple[typing.Literal['Ok'], int], typing.Tuple[typing.Literal['Err'], DivError]]:\n    if b == 0:\n        return ('Err', DivError.DivideByZero)\n    if a == -2 ** 63 and b == -1:\n        return ('Err', DivError.Overflow)\n    q, r = divmod(a, b)\n    w = q + (1 if r != 0 and (a < 0) != (b < 0) else 0)\n    return ('Ok', w)".to_string(),
        );
    }
    for decl in function_decls {
        sections.push(ctx.render_function_declaration(decl)?);
    }
    if mode == EmitPythonMode::Program {
        let mut assignments = Vec::new();
        for bind in &top_level_binds {
            assignments.push(format!(
                "{} = {}",
                bind.name,
                ctx.render_top_level_value(bind.value)?
            ));
        }
        let final_name = &top_level_binds.last().expect("guarded above").name;
        let mut body = assignments.join("\n");
        body.push_str(&format!("\nprint({final_name})"));
        sections.push(format!(
            "if __name__ == \"__main__\":\n{}",
            indent(&body, 1)
        ));
    }

    Ok(sections.join("\n\n"))
}

impl<'a> Ctx<'a> {
    fn render_top_level_value(&self, port: PortId) -> Result<String, EmitPythonError> {
        self.dispatch_producer(port, &RenderLocals::default())
    }

    fn render_port(&self, port: PortId, locals: &RenderLocals) -> Result<String, EmitPythonError> {
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

    fn dispatch_producer(
        &self,
        port: PortId,
        locals: &RenderLocals,
    ) -> Result<String, EmitPythonError> {
        let Some(node_id) = self.dag.port(port).produced_by else {
            return Err(EmitPythonError::Unsupported(
                "render reached a port with no producer".to_string(),
            ));
        };
        match self.dag.node(node_id) {
            Behavior::Value(v) => Ok(render_value(v)),
            Behavior::Transform(t) => self.render_transform(t, locals),
            Behavior::Branch(b) => self.render_branch(b, locals),
            Behavior::Loop(l) => self.render_loop(l, locals),
            Behavior::Bind(b) => Ok(b.name.clone()),
        }
    }

    fn render_transform(
        &self,
        t: &TransformNode,
        locals: &RenderLocals,
    ) -> Result<String, EmitPythonError> {
        match &t.target {
            TransformTarget::Operator(op) => self.render_operator(t, *op, locals),
            TransformTarget::UnresolvedFieldProject { field_label } => {
                return Err(EmitPythonError::Unsupported(format!(
                    "field projection .{field_label} is unresolved; emit_python expects post-infer FieldProject targets"
                )));
            }
            TransformTarget::ResolvedFieldProject { field_label } => {
                if let Some(binding) = locals
                    .payload_bindings
                    .get(&t.inputs[0])
                    .and_then(|binding| binding.field(field_label))
                {
                    return Ok(binding.clone());
                }
                let object = self.render_port(t.inputs[0], locals)?;
                Ok(render_named_template(
                    &self.indexes.syntax.field_access,
                    &[("object", &object), ("field", field_label)],
                ))
            }
            TransformTarget::Callable(target) => self.render_callable_transform(t, *target, locals),
        }
    }

    fn render_operator(
        &self,
        t: &TransformNode,
        op: OperatorKind,
        locals: &RenderLocals,
    ) -> Result<String, EmitPythonError> {
        if t.inputs.len() != 2 {
            return Err(EmitPythonError::Unsupported(format!(
                "python emitter supports only binary operators, got arity {}",
                t.inputs.len()
            )));
        }
        let operand_type = primitive_type_id_for_port(self.dag, t.inputs[0])?;
        let op_decl = algebra_field_for_operator(self.dag, operand_type, op)?;
        let carrier = super::operator_carrier_realization(
            &self.indexes.operators,
            self.dag,
            operand_type,
            op_decl,
        )
        .ok_or(EmitPythonError::MissingOperatorRealization {
            target: operand_type,
            op: op_decl,
        })?;
        let lhs = self.render_port(t.inputs[0], locals)?;
        let rhs = self.render_port(t.inputs[1], locals)?;
        Ok(render_named_template(
            carrier,
            &[("lhs", &lhs), ("rhs", &rhs)],
        ))
    }

    fn render_branch(
        &self,
        branch: &BranchNode,
        locals: &RenderLocals,
    ) -> Result<String, EmitPythonError> {
        if self.branch_scrutinee_is_bool(branch)? {
            let (then_path, else_path) = self.split_bool_paths(branch)?;
            let cond = self.render_port(branch.input, locals)?;
            let then_expr = self.render_path_body(then_path, locals)?;
            let else_expr = self.render_path_body(else_path, locals)?;
            return Ok(format!("({then_expr} if {cond} else {else_expr})"));
        }
        if let Some(rendered) = self.render_realized_pattern_branch(branch, locals)? {
            return Ok(rendered);
        }
        self.render_general_match(branch, locals)
    }

    fn render_realized_pattern_branch(
        &self,
        branch: &BranchNode,
        locals: &RenderLocals,
    ) -> Result<Option<String>, EmitPythonError> {
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
    ) -> Result<String, EmitPythonError> {
        let empty_path = find_resolved_branch_path(branch, binding.empty_variant).ok_or_else(
            || {
                EmitPythonError::Unsupported(
                    "vector-list pattern realization requires a branch arm for the declared empty_variant"
                        .to_string(),
                )
            },
        )?;
        let cons_path = find_resolved_branch_path(branch, binding.cons_variant).ok_or_else(
            || {
                EmitPythonError::Unsupported(
                    "vector-list pattern realization requires a branch arm for the declared cons_variant"
                        .to_string(),
                )
            },
        )?;
        let scrutinee = self.render_port(branch.input, locals)?;
        let empty_body = self.render_path_body(empty_path, locals)?;
        let realized_scrutinee = render_named_template(&binding.scrutinee, &[("expr", &scrutinee)]);
        let empty_predicate = render_named_template(&binding.empty_pattern, &[("expr", "__match")]);
        let mut cons_locals = locals.clone();
        if let Some(payload_binding) = &cons_path.binding {
            let cons_expr = render_named_template(&binding.cons_pattern, &[("expr", "__match")]);
            let head_expr = render_named_template(&binding.head_expr, &[("list", &cons_expr)]);
            let tail_expr = render_named_template(&binding.tail_expr, &[("list", &cons_expr)]);
            cons_locals.payload_bindings.insert(
                payload_binding.payload_port,
                VariantPayloadBinding::Direct(format!(
                    "types.SimpleNamespace(head={head_expr}, tail={tail_expr})"
                )),
            );
        }
        let cons_body = self.render_port(cons_path.output, &cons_locals)?;
        Ok(format!(
            "(lambda __match: ({empty_body} if {empty_predicate} else {cons_body}))({realized_scrutinee})"
        ))
    }

    fn render_general_match(
        &self,
        branch: &BranchNode,
        locals: &RenderLocals,
    ) -> Result<String, EmitPythonError> {
        let scrutinee = self.render_port(branch.input, locals)?;
        let scrutinee_type_id = primitive_type_id_for_port(self.dag, branch.input)?;
        let disj_id = walk_to_disj(self.dag, scrutinee_type_id).ok_or_else(|| {
            EmitPythonError::Unsupported(format!(
                "branch scrutinee {:?} does not walk to a disjunction",
                scrutinee_type_id
            ))
        })?;
        let is_optional = is_optional_match_disj(self.dag, disj_id);
        let mut rendered = "__v3_unreachable(\"non-exhaustive match\")".to_string();
        for path in branch.paths.iter().rev() {
            let cond = self.render_branch_condition(disj_id, is_optional, path)?;
            let body = self.render_branch_body_expr(is_optional, path, locals)?;
            rendered = format!("({body} if {cond} else {rendered})");
        }
        Ok(format!("(lambda __match: {rendered})({scrutinee})"))
    }

    fn render_branch_condition(
        &self,
        disj_id: DeclarationId,
        is_optional: bool,
        path: &Path,
    ) -> Result<String, EmitPythonError> {
        let variant_id = resolved_pattern_id(path)?;
        if is_optional {
            let (none_variant, some_variant) = optional_match_variant_roles(self.dag, disj_id)
                .map_err(|detail| EmitPythonError::Unsupported(detail.to_string()))?;
            return Ok(if variant_id == none_variant {
                "__match is None".to_string()
            } else if variant_id == some_variant {
                "__match is not None".to_string()
            } else {
                return Err(EmitPythonError::Unsupported(
                    "optional match arm resolved to neither optional role".to_string(),
                ));
            });
        }
        let variant_name = runtime_variant_name_for_decl(self.dag, disj_id, variant_id)?;
        Ok(format!("isinstance(__match, {variant_name})"))
    }

    /// E-5 / Lane 1 Stage 1c PR 3: dispatch on
    /// `python_clean_emission.pattern_bindings`.
    /// `NotApplicablePatternBinding` selects the substitute-at-
    /// render-time path: for every payload-binding port we map it
    /// to an extraction expression (`__match._0` / `__match`), so
    /// the binding's identifier never appears in emitted Python at
    /// a pattern site. If the arm body does not reference the
    /// port, the substitution is never invoked and no dead
    /// identifier leaks into the source — py_compile stays silent
    /// by construction. Python-invalid contract variants are
    /// rejected while building `CleanEmissionContractBinding`, so
    /// the renderer only sees Python-valid states.
    fn render_branch_body_expr(
        &self,
        is_optional: bool,
        path: &Path,
        locals: &RenderLocals,
    ) -> Result<String, EmitPythonError> {
        let mut arm_locals = locals.clone();
        if let Some(binding) = &path.binding {
            match self.indexes.clean_emission.pattern_bindings {
                PatternBindingRuleBinding::NotApplicable => {
                    if let Some(payload_binding) =
                        self.render_variant_payload_binding(is_optional, path)?
                    {
                        arm_locals
                            .payload_bindings
                            .insert(binding.payload_port, payload_binding);
                    }
                }
            }
        }
        self.render_port(path.output, &arm_locals)
    }

    fn render_variant_payload_binding(
        &self,
        is_optional: bool,
        path: &Path,
    ) -> Result<Option<VariantPayloadBinding<String>>, EmitPythonError> {
        if is_optional {
            return Ok(Some(VariantPayloadBinding::Direct("__match".to_string())));
        }
        let variant_id = resolved_pattern_id(path)?;
        let shape = match variant_payload_shape(self.dag, &variant_id) {
            VariantPayloadShapeLookup::DeclarationMissing => {
                return Err(EmitPythonError::Unsupported(
                    "variant payload references an absent declaration".to_string(),
                ));
            }
            VariantPayloadShapeLookup::NotPayloadProduct => {
                return Ok(Some(VariantPayloadBinding::Direct("__match".to_string())));
            }
            VariantPayloadShapeLookup::Found { _0: shape } => shape,
        };
        Ok(match shape {
            VariantPayloadShape::Empty => None,
            VariantPayloadShape::PositionalSingle => {
                Some(VariantPayloadBinding::Direct("__match._0".to_string()))
            }
            VariantPayloadShape::NamedFields { _0: field_labels } => {
                match self.indexes.clean_emission.variant_payload_field_access {
                    VariantPayloadFieldAccessRuleBinding::AccessFromPayloadBinding => {
                        Some(VariantPayloadBinding::Direct("__match".to_string()))
                    }
                    VariantPayloadFieldAccessRuleBinding::OverrideNamedFieldsAtBindingSite => {
                        let fields = field_labels
                            .into_iter()
                            .map(|field_label| {
                                (field_label.clone(), format!("__match.{field_label}"))
                            })
                            .collect();
                        Some(VariantPayloadBinding::Fields(fields))
                    }
                }
            }
        })
    }

    fn render_path_body(
        &self,
        path: &Path,
        locals: &RenderLocals,
    ) -> Result<String, EmitPythonError> {
        self.render_port(path.output, locals)
    }

    fn render_loop(
        &self,
        loop_node: &crate::dag::LoopNode,
        locals: &RenderLocals,
    ) -> Result<String, EmitPythonError> {
        // Behavior::Loop has exactly two construction sites in lower.rs,
        // both for recursive user functions:
        //   1. Cardinality bound (lower.rs ~3631): single recursive fn with
        //      descent-provable termination; bound.count = first param port.
        //   2. Descent bound (lower.rs ~382): mutual-recursion cluster.
        //
        // In both cases `loop_node.body` is the root node of the function's
        // body DAG, which already contains recursive self-calls to the same
        // or mutually-recursive functions. Rendering the body node's result
        // port preserves those calls. Python supports recursion natively, so
        // the emitted expression is semantically correct without any iteration
        // scaffolding. The `source`/`init`/`bound` fields encode the
        // termination *proof*, not operational iteration state.
        //
        // Collection folds (fold/map/filter) route through callable
        // realizations (__v3_fold etc.) and never reach Behavior::Loop.
        // If a future IR change adds Loop for non-recursive collection
        // iteration, this site must be updated to emit explicit iteration.
        let body_port = super::behavior_result_port(self.dag.node(loop_node.body));
        self.render_port(body_port, locals)
    }

    fn render_callable_transform(
        &self,
        t: &TransformNode,
        target: DeclarationId,
        locals: &RenderLocals,
    ) -> Result<String, EmitPythonError> {
        let (template, arguments) = callable_template(target, self.dag);
        if let Some(strategy) = self.indexes.callables.get(&template) {
            return self
                .render_realized_callable(template, *strategy, &arguments, &t.inputs, locals);
        }
        self.render_general_callable(template, &t.inputs, locals)
    }

    fn render_realized_callable(
        &self,
        template: DeclarationId,
        strategy: CallableStrategyBinding,
        arguments: &[TemplateArgument],
        inputs: &[PortId],
        locals: &RenderLocals,
    ) -> Result<String, EmitPythonError> {
        match strategy {
            CallableStrategyBinding::Empty => Ok(self.indexes.syntax.empty_list.clone()),
            CallableStrategyBinding::Singleton => {
                let value = self.render_port(inputs[0], locals)?;
                Ok(render_named_template(
                    &self.indexes.syntax.list_literal,
                    &[("elements", &value)],
                ))
            }
            CallableStrategyBinding::Cons => {
                let head = self.render_port(inputs[0], locals)?;
                let tail = self.render_port(inputs[1], locals)?;
                Ok(render_named_template(
                    &self.indexes.syntax.cons,
                    &[("head", &head), ("tail", &tail)],
                ))
            }
            CallableStrategyBinding::Concat => {
                let left = self.render_port(inputs[0], locals)?;
                let right = self.render_port(inputs[1], locals)?;
                Ok(render_named_template(
                    &self.indexes.syntax.concat,
                    &[("left", &left), ("right", &right)],
                ))
            }
            CallableStrategyBinding::Length => {
                let recv = self.render_port(inputs[0], locals)?;
                Ok(render_named_template(
                    &self.indexes.syntax.length,
                    &[("recv", &recv)],
                ))
            }
            CallableStrategyBinding::IsEmpty => {
                let recv = self.render_port(inputs[0], locals)?;
                Ok(render_named_template(
                    &self.indexes.syntax.is_empty,
                    &[("recv", &recv)],
                ))
            }
            CallableStrategyBinding::Fold => {
                let fn_decl = bound_callable_argument(self.dag, template, arguments, 2)?;
                let acc = "__fold_acc".to_string();
                let item = "__fold_item".to_string();
                let body = self.render_closure(
                    fn_decl,
                    &[(acc.clone(), acc.clone()), (item.clone(), item.clone())],
                    locals,
                )?;
                let recv = self.render_port(inputs[0], locals)?;
                let init = self.render_port(inputs[1], locals)?;
                Ok(render_named_template(
                    &self.indexes.syntax.fold,
                    &[("recv", &recv), ("init", &init), ("body", &body)],
                ))
            }
            CallableStrategyBinding::Map => {
                let fn_decl = bound_callable_argument(self.dag, template, arguments, 1)?;
                let item = "__map_item".to_string();
                let body =
                    self.render_callable_body(fn_decl, &[(item.clone(), item.clone())], locals)?;
                let recv = self.render_port(inputs[0], locals)?;
                Ok(render_named_template(
                    &self.indexes.syntax.map,
                    &[("recv", &recv), ("item", &item), ("body", &body)],
                ))
            }
            CallableStrategyBinding::Filter => {
                let fn_decl = bound_callable_argument(self.dag, template, arguments, 1)?;
                let item = "__filter_item".to_string();
                let predicate =
                    self.render_callable_body(fn_decl, &[(item.clone(), item.clone())], locals)?;
                let recv = self.render_port(inputs[0], locals)?;
                Ok(render_named_template(
                    &self.indexes.syntax.filter,
                    &[
                        ("recv", &recv),
                        ("item", &item),
                        ("item_push", &item),
                        ("predicate", &predicate),
                    ],
                ))
            }
            CallableStrategyBinding::Contains => {
                let recv = self.render_port(inputs[0], locals)?;
                let item = self.render_port(inputs[1], locals)?;
                Ok(render_named_template(
                    &self.indexes.syntax.contains,
                    &[("recv", &recv), ("item", &item)],
                ))
            }
        }
    }

    fn render_general_callable(
        &self,
        template: DeclarationId,
        inputs: &[PortId],
        locals: &RenderLocals,
    ) -> Result<String, EmitPythonError> {
        if let Some(rendered) = self.render_variant_constructor(template, inputs, locals)? {
            return Ok(rendered);
        }
        if let Some(rendered) = self.render_record_constructor(template, inputs, locals)? {
            return Ok(rendered);
        }
        let func = self.dag.declaration(template).name.clone().ok_or_else(|| {
            EmitPythonError::Unsupported(
                "callable target is anonymous and cannot be rendered as a direct Python call"
                    .to_string(),
            )
        })?;
        let args = inputs
            .iter()
            .map(|port| self.render_port(*port, locals))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(render_named_template(
            &self.indexes.syntax.function_call,
            &[("func", &func), ("args", &args.join(", "))],
        ))
    }

    fn render_record_constructor(
        &self,
        template: DeclarationId,
        inputs: &[PortId],
        locals: &RenderLocals,
    ) -> Result<Option<String>, EmitPythonError> {
        let decl = self.dag.declaration(template);
        let Some(type_name) = &decl.name else {
            return Ok(None);
        };
        let TypeConnective::Conj { children } = &decl.connective else {
            return Ok(None);
        };
        let mut fields = Vec::new();
        for (field, input) in children.iter().zip(inputs.iter()) {
            fields.push(format!(
                "{}={}",
                field.label,
                self.render_port(*input, locals)?
            ));
        }
        Ok(Some(format!("{type_name}({})", fields.join(", "))))
    }

    fn render_variant_constructor(
        &self,
        template: DeclarationId,
        inputs: &[PortId],
        locals: &RenderLocals,
    ) -> Result<Option<String>, EmitPythonError> {
        let Some((enum_name, variant_name)) = variant_parent_info(self.dag, template) else {
            return Ok(None);
        };
        let runtime_name = python_variant_class_name(&enum_name, &variant_name);
        let TypeConnective::Conj { children } = &self.dag.declaration(template).connective else {
            return Ok(None);
        };
        if children.is_empty() {
            return Ok(Some(format!("{runtime_name}()")));
        }
        let payload_shape = match variant_payload_shape(self.dag, &template) {
            VariantPayloadShapeLookup::DeclarationMissing => {
                return Err(EmitPythonError::Unsupported(
                    "variant constructor references an absent declaration".to_string(),
                ));
            }
            VariantPayloadShapeLookup::NotPayloadProduct => return Ok(None),
            VariantPayloadShapeLookup::Found { _0: shape } => shape,
        };
        if matches!(payload_shape, VariantPayloadShape::PositionalSingle) {
            let [input] = inputs else {
                return Err(EmitPythonError::Unsupported(
                    "positional-single variant construction expects exactly one input".to_string(),
                ));
            };
            let arg = self.render_port(*input, locals)?;
            return Ok(Some(format!("{runtime_name}({arg})")));
        }
        let mut fields = Vec::new();
        for (field, input) in children.iter().zip(inputs.iter()) {
            fields.push(format!(
                "{}={}",
                field.label,
                self.render_port(*input, locals)?
            ));
        }
        Ok(Some(format!("{runtime_name}({})", fields.join(", "))))
    }

    fn render_callable_body(
        &self,
        callable_decl: DeclarationId,
        param_bindings: &[(String, String)],
        outer_locals: &RenderLocals,
    ) -> Result<String, EmitPythonError> {
        let TypeConnective::Arrow { inputs, body, .. } =
            &self.dag.declaration(callable_decl).connective
        else {
            return Err(EmitPythonError::Unsupported(
                "callable template binding did not resolve to an Arrow".to_string(),
            ));
        };
        let ArrowBody::UserDefined(bind_id) = body else {
            return Err(EmitPythonError::Unsupported(
                "python emitter only supports user-defined callable bodies".to_string(),
            ));
        };
        let bind = (*bind_id).bind(self.dag);
        if bind.params.len() < inputs.len() {
            return Err(EmitPythonError::Unsupported(
                "callable bind parameter count does not match arrow inputs".to_string(),
            ));
        }
        let capture_count = bind.params.len() - inputs.len();
        let mut locals = RenderLocals::default();
        for capture in bind.params.iter().copied().take(capture_count) {
            locals
                .names
                .insert(capture, self.render_port(capture, outer_locals)?);
        }
        for (port, (_, name)) in bind
            .params
            .iter()
            .copied()
            .skip(capture_count)
            .zip(param_bindings.iter())
        {
            locals.names.insert(port, name.clone());
        }
        self.render_port(bind.value, &locals)
    }

    fn render_closure(
        &self,
        callable_decl: DeclarationId,
        param_bindings: &[(String, String)],
        outer_locals: &RenderLocals,
    ) -> Result<String, EmitPythonError> {
        let body = self.render_callable_body(callable_decl, param_bindings, outer_locals)?;
        let params = param_bindings
            .iter()
            .map(|(name, _)| name.clone())
            .collect::<Vec<_>>()
            .join(", ");
        Ok(render_named_template(
            &self.indexes.syntax.closure,
            &[("params", &params), ("body", &body)],
        ))
    }

    fn render_function_declaration(
        &self,
        declaration: &crate::dag::Declaration,
    ) -> Result<String, EmitPythonError> {
        let name = declaration.name.clone().ok_or_else(|| {
            EmitPythonError::Unsupported(
                "anonymous arrows cannot be emitted as Python functions".to_string(),
            )
        })?;
        let TypeConnective::Arrow {
            inputs,
            output,
            body,
        } = &declaration.connective
        else {
            return Err(EmitPythonError::Unsupported(
                "render_function_declaration expected Arrow".to_string(),
            ));
        };
        if !declaration.type_params.is_empty() {
            return Err(EmitPythonError::Unsupported(format!(
                "generic function `{name}` is not yet supported by emit_python"
            )));
        }
        let ArrowBody::UserDefined(bind_id) = body else {
            return Err(EmitPythonError::Unsupported(
                "emit_python only supports user-defined function bodies".to_string(),
            ));
        };
        let bind = (*bind_id).bind(self.dag);
        let mut locals = RenderLocals::default();
        let mut params = Vec::new();
        for (idx, port) in bind.params.iter().enumerate() {
            let param_name = format!("p{idx}");
            locals.names.insert(*port, param_name.clone());
            let ty = self.python_type_name_for_port(*port)?;
            params.push(format!("{param_name}: {ty}"));
        }
        if bind.params.len() != inputs.len() {
            return Err(EmitPythonError::Unsupported(
                "function bind parameter count does not match arrow inputs".to_string(),
            ));
        }
        let ret = self.python_type_name_for_decl(*output)?;
        let body_expr = self.render_port(bind.value, &locals)?;
        Ok(format!(
            "def {name}({}) -> {ret}:\n    return {body_expr}",
            params.join(", ")
        ))
    }

    fn render_type_declaration(
        &self,
        declaration: &crate::dag::Declaration,
    ) -> Result<String, EmitPythonError> {
        let name = declaration.name.clone().ok_or_else(|| {
            EmitPythonError::Unsupported(
                "anonymous type declarations cannot be emitted".to_string(),
            )
        })?;
        match &declaration.connective {
            TypeConnective::Conj { children } => {
                if children.is_empty() {
                    return Ok(format!("@dataclass\nclass {name}:\n    pass"));
                }
                let mut lines = vec!["@dataclass".to_string(), format!("class {name}:")];
                for child in children {
                    lines.push(format!(
                        "    {}: {}",
                        child.label,
                        self.python_type_name_for_decl(child.ty)?
                    ));
                }
                Ok(lines.join("\n"))
            }
            TypeConnective::Disj { variants } => {
                let mut blocks = vec![format!("class {name}:\n    pass")];
                for variant in variants {
                    blocks.push(self.render_enum_variant(name.as_str(), variant)?);
                }
                Ok(blocks.join("\n\n"))
            }
            _ => Err(EmitPythonError::Unsupported(format!(
                "type declaration `{name}` does not lower to a record or sum shape"
            ))),
        }
    }

    fn render_enum_variant(
        &self,
        enum_name: &str,
        variant: &Field,
    ) -> Result<String, EmitPythonError> {
        let variant_decl = self.dag.declaration(variant.ty);
        let TypeConnective::Conj { children } = &variant_decl.connective else {
            return Err(EmitPythonError::Unsupported(format!(
                "enum variant `{}` does not lower to a product declaration",
                variant.label
            )));
        };
        let runtime_name = python_variant_class_name(enum_name, &variant.label);
        let mut lines = vec![
            "@dataclass".to_string(),
            format!("class {runtime_name}({enum_name}):"),
        ];
        if children.is_empty() {
            lines.push("    pass".to_string());
            return Ok(lines.join("\n"));
        }
        for child in children {
            lines.push(format!(
                "    {}: {}",
                child.label,
                self.python_type_name_for_decl(child.ty)?
            ));
        }
        Ok(lines.join("\n"))
    }

    fn python_type_name_for_port(&self, port: PortId) -> Result<String, EmitPythonError> {
        let ty = self
            .dag
            .port(port)
            .value_type()
            .ok_or(EmitPythonError::UntypedPort(port))?;
        self.python_type_name_for_decl(ty.declaration)
    }

    fn python_type_name_for_decl(
        &self,
        declaration: DeclarationId,
    ) -> Result<String, EmitPythonError> {
        self.python_type_name_for_decl_at_depth(declaration, 0)
    }

    fn python_type_name_for_decl_at_depth(
        &self,
        declaration: DeclarationId,
        depth: usize,
    ) -> Result<String, EmitPythonError> {
        if depth >= 32 {
            return Err(EmitPythonError::Unsupported(
                "type-name rendering exceeded depth 32".to_string(),
            ));
        }
        if let Some(binding) = self.indexes.types.get(&declaration) {
            return Ok(binding.clone());
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
                let carrier = self
                    .indexes
                    .type_instantiations
                    .get(template)
                    .ok_or(EmitPythonError::MissingTypeRealization { target: *template })?;
                match arguments.as_slice() {
                    [element] => {
                        let element_name =
                            self.python_type_name_for_decl_at_depth(element.value, depth + 1)?;
                        Ok(render_named_template(
                            carrier,
                            &[("element", &element_name)],
                        ))
                    }
                    [key, value] => {
                        let key_name =
                            self.python_type_name_for_decl_at_depth(key.value, depth + 1)?;
                        let value_name =
                            self.python_type_name_for_decl_at_depth(value.value, depth + 1)?;
                        Ok(render_named_template(
                            carrier,
                            &[("key", &key_name), ("value", &value_name)],
                        ))
                    }
                    _ => Err(EmitPythonError::Unsupported(
                        "python type instantiation supports arities 1 and 2".to_string(),
                    )),
                }
            }
            TypeConnective::Cardinality(p)
                if p.bound() == crate::dag::CardinalityBound::AtMostOne =>
            {
                let inner = self.python_type_name_for_decl_at_depth(p.element(), depth + 1)?;
                Ok(render_named_template(
                    &self.indexes.syntax.optional,
                    &[("element", &inner)],
                ))
            }
            TypeConnective::Atom(AtomPayload::ResolvedByStructure(next))
            | TypeConnective::Atom(AtomPayload::ResolvedByName(next)) => {
                self.python_type_name_for_decl_at_depth(*next, depth + 1)
            }
            _ => Err(EmitPythonError::MissingTypeRealization {
                target: declaration,
            }),
        }
    }

    fn branch_scrutinee_is_bool(&self, branch: &BranchNode) -> Result<bool, EmitPythonError> {
        let scrutinee_type_id = primitive_type_id_for_port(self.dag, branch.input)?;
        let Some(bool_shape) = self.dag.bool_shape() else {
            return Ok(false);
        };
        Ok(walk_to_disj(self.dag, scrutinee_type_id)
            .zip(walk_to_disj(self.dag, bool_shape.declaration))
            .is_some_and(|(left, right)| left == right))
    }

    fn split_bool_paths<'p>(
        &self,
        branch: &'p BranchNode,
    ) -> Result<(&'p Path, &'p Path), EmitPythonError> {
        let scrutinee_type_id = primitive_type_id_for_port(self.dag, branch.input)?;
        let disj_id = walk_to_disj(self.dag, scrutinee_type_id).ok_or_else(|| {
            EmitPythonError::Unsupported(
                "bool branch scrutinee does not walk to a Disj".to_string(),
            )
        })?;
        let TypeConnective::Disj { variants } = &self.dag.declaration(disj_id).connective else {
            unreachable!("walk_to_disj returned non-Disj")
        };
        if variants.len() != 2 {
            return Err(EmitPythonError::Unsupported(
                "bool branch must have exactly two variants".to_string(),
            ));
        }
        let true_variant = variants[0].ty;
        let false_variant = variants[1].ty;
        let then_path = find_resolved_branch_path(branch, true_variant)
            .ok_or_else(|| EmitPythonError::Unsupported("missing True branch arm".to_string()))?;
        let else_path = find_resolved_branch_path(branch, false_variant)
            .ok_or_else(|| EmitPythonError::Unsupported("missing False branch arm".to_string()))?;
        Ok((then_path, else_path))
    }
}

fn render_value(v: &crate::dag::ValueNode) -> String {
    match &v.data {
        LiteralBits::Int(decimal) => decimal.clone(),
        LiteralBits::Bool(true) => "True".to_string(),
        LiteralBits::Bool(false) => "False".to_string(),
        LiteralBits::String(s) => format!("{:?}", s),
    }
}

fn parse_callable_strategy(
    dag: &Dag,
    fields: &[(String, FieldValue)],
    declaration: DeclarationId,
) -> Result<CallableStrategyBinding, EmitPythonError> {
    let (constructor, payload) = variant_field(fields, "strategy", declaration)?;
    if !payload.is_empty() {
        return Err(EmitPythonError::MalformedSpec {
            declaration,
            detail: "CallableStrategy variants must not carry payload",
        });
    }
    let variants = dag.callable_strategy_variants();
    let strategies = [
        (variants.list_empty, CallableStrategyBinding::Empty),
        (variants.list_singleton, CallableStrategyBinding::Singleton),
        (variants.list_cons, CallableStrategyBinding::Cons),
        (variants.list_concat, CallableStrategyBinding::Concat),
        (variants.list_length, CallableStrategyBinding::Length),
        (variants.list_is_empty, CallableStrategyBinding::IsEmpty),
        (variants.list_fold, CallableStrategyBinding::Fold),
        (variants.list_map, CallableStrategyBinding::Map),
        (variants.list_filter, CallableStrategyBinding::Filter),
        (variants.list_contains, CallableStrategyBinding::Contains),
    ];
    for (variant_id, strategy) in strategies {
        let Some(variant_id) = variant_id else {
            return Err(EmitPythonError::MalformedSpec {
                declaration,
                detail: "CallableStrategy variant declaration was not found",
            });
        };
        if constructor == variant_id {
            return Ok(strategy);
        }
    }
    Err(EmitPythonError::MalformedSpec {
        declaration,
        detail: "unsupported CallableStrategy variant",
    })
}

/// Structural boundary check for `PatternRealization.empty_variant` /
/// `cons_variant`. Mirrors `rust_target::validate_pattern_roles` — see
/// that comment for rationale. Rejects spec data where the typed
/// `DeclarationRef`s point outside `target`'s variants.
fn validate_pattern_roles(
    dag: &Dag,
    target: DeclarationId,
    binding: &PatternRealizationBinding,
    declaration: DeclarationId,
) -> Result<(), EmitPythonError> {
    let disj_id = walk_to_disj(dag, target).ok_or(EmitPythonError::MalformedSpec {
        declaration,
        detail:
            "PatternRealization.target must resolve to a Disj — empty_variant / cons_variant have no target to range over otherwise",
    })?;
    let TypeConnective::Disj { variants } = &dag.declaration(disj_id).connective else {
        return Err(EmitPythonError::MalformedSpec {
            declaration,
            detail: "walk_to_disj returned a non-Disj declaration (internal invariant violation)",
        });
    };
    if !variants.iter().any(|v| v.ty == binding.empty_variant) {
        return Err(EmitPythonError::MalformedSpec {
            declaration,
            detail:
                "PatternRealization.empty_variant must be a variant of `target` — structural boundary check rejects unrelated DeclarationRefs",
        });
    }
    if !variants.iter().any(|v| v.ty == binding.cons_variant) {
        return Err(EmitPythonError::MalformedSpec {
            declaration,
            detail:
                "PatternRealization.cons_variant must be a variant of `target` — structural boundary check rejects unrelated DeclarationRefs",
        });
    }
    if binding.empty_variant == binding.cons_variant {
        return Err(EmitPythonError::MalformedSpec {
            declaration,
            detail:
                "PatternRealization.empty_variant and cons_variant must be distinct variants of `target`",
        });
    }
    Ok(())
}

fn parse_pattern_realization(
    dag: &Dag,
    fields: &[(String, FieldValue)],
    declaration: DeclarationId,
) -> Result<PatternRealizationBinding, EmitPythonError> {
    let strategy = fields
        .iter()
        .find(|(label, _)| label == "strategy")
        .map(|(_, value)| value)
        .ok_or(EmitPythonError::MalformedSpec {
            declaration,
            detail: "PatternRealization is missing required `strategy` field",
        })?;
    match parse_pattern_strategy(dag, strategy) {
        Ok(PatternStrategyBinding::VectorList) => {}
        Err(detail) => {
            return Err(EmitPythonError::MalformedSpec {
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

fn method_contract_single_emit_template_string(
    dag: &Dag,
    contract_decl: DeclarationId,
) -> Result<String, EmitPythonError> {
    let fields = structural_fields_for_decl(dag, contract_decl)?;
    let emit_value = fields
        .iter()
        .find(|(label, _)| label == "emit_template")
        .map(|(_, v)| v)
        .ok_or(EmitPythonError::MalformedSpec {
            declaration: contract_decl,
            detail: "MethodTemplateContract missing emit_template field",
        })?;
    let FieldValue::Variant {
        constructor,
        ref payload,
    } = emit_value
    else {
        return Err(EmitPythonError::MalformedSpec {
            declaration: contract_decl,
            detail: "MethodTemplateContract.emit_template must be a sum variant",
        });
    };
    let ctor_name = method_emit_template_variant_label(dag, *constructor)
        .ok_or(EmitPythonError::MalformedSpec {
        declaration: contract_decl,
        detail:
            "MethodTemplateContract.emit_template variant not found under MethodEmitTemplate disj",
    })?;
    if ctor_name != "SingleTemplate" {
        return Err(EmitPythonError::MalformedSpec {
            declaration: contract_decl,
            detail: "CollectionOps MethodTemplateContract must use MethodEmitTemplate.SingleTemplate today",
        });
    }
    let [FieldValue::Literal(LiteralBits::String(template))] = payload.as_slice() else {
        return Err(EmitPythonError::MalformedSpec {
            declaration: contract_decl,
            detail: "SingleTemplate must carry exactly one string template payload",
        });
    };
    Ok(template.clone())
}

fn structural_fields_for_decl(
    dag: &Dag,
    declaration: DeclarationId,
) -> Result<&[(String, FieldValue)], EmitPythonError> {
    structural_fields(dag.declaration(declaration)).ok_or(EmitPythonError::MalformedSpec {
        declaration,
        detail: "named Python spec entry must be a structural data item",
    })
}

fn structural_fields(decl: &crate::dag::Declaration) -> Option<&[(String, FieldValue)]> {
    match &decl.value_body {
        Some(crate::dag::ValueBody::Structural { fields }) => Some(fields),
        _ => None,
    }
}

fn require_field_decl_ref(
    fields: &[(String, FieldValue)],
    name: &str,
    declaration: DeclarationId,
) -> Result<DeclarationId, EmitPythonError> {
    fields
        .iter()
        .find(|(label, _)| label == name)
        .and_then(|(_, value)| match value {
            FieldValue::Reference(target) => Some(*target),
            _ => None,
        })
        .ok_or(EmitPythonError::MalformedSpec {
            declaration,
            detail: "required declaration-ref field missing or malformed",
        })
}

fn require_field_string(
    fields: &[(String, FieldValue)],
    name: &str,
    declaration: DeclarationId,
) -> Result<String, EmitPythonError> {
    fields
        .iter()
        .find(|(label, _)| label == name)
        .and_then(|(_, value)| match value {
            FieldValue::Literal(LiteralBits::String(value)) => Some(value.clone()),
            _ => None,
        })
        .ok_or(EmitPythonError::MalformedSpec {
            declaration,
            detail: "required string field missing or malformed",
        })
}

fn variant_field<'a>(
    fields: &'a [(String, FieldValue)],
    name: &str,
    declaration: DeclarationId,
) -> Result<(DeclarationId, &'a [FieldValue]), EmitPythonError> {
    let value = fields
        .iter()
        .find(|(label, _)| label == name)
        .map(|(_, value)| value)
        .ok_or(EmitPythonError::MalformedSpec {
            declaration,
            detail: "required variant field missing",
        })?;
    let FieldValue::Variant {
        constructor,
        payload,
    } = value
    else {
        return Err(EmitPythonError::MalformedSpec {
            declaration,
            detail: "required field must be a variant",
        });
    };
    Ok((*constructor, payload))
}

/// Parse the `memory` field of a TargetExecutionModel into a typed
/// MemoryModelBinding. Mirrors emit_go::require_memory_model — keeps
/// the closed sum closed instead of demoting it to a string.
fn require_memory_model(
    dag: &Dag,
    fields: &[(String, FieldValue)],
    declaration: DeclarationId,
) -> Result<MemoryModelBinding, EmitPythonError> {
    let (constructor, payload) = variant_field(fields, "memory", declaration)?;
    if !payload.is_empty() {
        return Err(EmitPythonError::MalformedSpec {
            declaration,
            detail: "MemoryModel variants must not carry payload fields",
        });
    }
    let variants = dag.emit_model_variants();
    let memory_variants = [
        (
            variants.memory_model.value_only,
            MemoryModelBinding::ValueOnly,
        ),
        (
            variants.memory_model.garbage_collected,
            MemoryModelBinding::GarbageCollected,
        ),
        (
            variants.memory_model.ref_counted,
            MemoryModelBinding::RefCounted,
        ),
        (
            variants.memory_model.ownership_based,
            MemoryModelBinding::OwnershipBased,
        ),
    ];
    for (variant_id, binding) in memory_variants {
        let variant_id = variant_id.ok_or(EmitPythonError::MissingMeta("MemoryModel variant"))?;
        if constructor == variant_id {
            return Ok(binding);
        }
    }
    Err(EmitPythonError::MalformedSpec {
        declaration,
        detail:
            "TargetExecutionModel.memory must be ValueOnly/GarbageCollected/RefCounted/OwnershipBased",
    })
}

/// Parse the `scope` field of a TargetExecutionModel. Mirrors
/// emit_go::require_scope_model.
fn require_scope_model(
    dag: &Dag,
    fields: &[(String, FieldValue)],
    declaration: DeclarationId,
) -> Result<ScopeModelBinding, EmitPythonError> {
    let (constructor, payload) = variant_field(fields, "scope", declaration)?;
    if !payload.is_empty() {
        return Err(EmitPythonError::MalformedSpec {
            declaration,
            detail: "ScopeModel variants must not carry payload fields",
        });
    }
    let variants = dag.emit_model_variants();
    let scope_variants = [
        (
            variants.scope_model.lexical_scoping,
            ScopeModelBinding::LexicalScoping,
        ),
        (
            variants.scope_model.dynamic_scoping,
            ScopeModelBinding::DynamicScoping,
        ),
    ];
    for (variant_id, binding) in scope_variants {
        let variant_id = variant_id.ok_or(EmitPythonError::MissingMeta("ScopeModel variant"))?;
        if constructor == variant_id {
            return Ok(binding);
        }
    }
    Err(EmitPythonError::MalformedSpec {
        declaration,
        detail: "TargetExecutionModel.scope must be LexicalScoping/DynamicScoping",
    })
}

fn render_named_template(template: &str, bindings: &[(&str, &str)]) -> String {
    let mut rendered = template.to_string();
    for (name, value) in bindings {
        rendered = rendered.replace(&format!("{{{name}}}"), value);
    }
    rendered
}

fn indent(source: &str, level: usize) -> String {
    let prefix = "    ".repeat(level);
    source
        .lines()
        .map(|line| {
            if line.is_empty() {
                line.to_string()
            } else {
                format!("{prefix}{line}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn primitive_type_id_for_port(dag: &Dag, port: PortId) -> Result<DeclarationId, EmitPythonError> {
    primitive_type_id_for_port_shared(dag, port).map_err(|err| match err {
        SharedEmitLookupError::UntypedPort(port) => EmitPythonError::UntypedPort(port),
        SharedEmitLookupError::Unsupported(detail) => {
            // Preserve the pre-consolidation Python diagnostic wording while the
            // target still owns its public error strings.
            EmitPythonError::Unsupported(detail.replace(" — likely a cycle", ""))
        }
    })
}

fn is_optional_match_disj(dag: &Dag, disj_id: DeclarationId) -> bool {
    dag.declarations()
        .iter()
        .filter_map(|decl| dag.optional_match_disj(decl.id))
        .any(|optional_disj| optional_disj == disj_id)
}

/// Resolve the algebra-field declaration id for a given operand
/// type and `OperatorKind`. Walks the operand type's instantiation
/// chain to its algebra Conj (e.g. Int → ... → OrderedRing instance),
/// then finds the field whose label matches the operator's algebra
/// field name. Returns the field's child declaration id, which the
/// python.dag `op: OrderedRing.add` reference also resolves to via
/// the dotted-path lowering. Mirrors emit_rust::algebra_field_for_operator.
fn algebra_field_for_operator(
    dag: &Dag,
    operand_type_id: DeclarationId,
    op: OperatorKind,
) -> Result<DeclarationId, EmitPythonError> {
    algebra_field_for_operator_shared(dag, operand_type_id, op).map_err(|err| match err {
        SharedEmitLookupError::UntypedPort(port) => EmitPythonError::UntypedPort(port),
        // Preserve the pre-consolidation Python diagnostic wording while the
        // target still owns its public error strings.
        SharedEmitLookupError::Unsupported(detail) => EmitPythonError::Unsupported(
            detail
                .replace("the canonical `OrderedRing`", "canonical OrderedRing")
                .replace(
                    "`OrderedRing` does not lower to a Conj declaration",
                    "OrderedRing did not lower to a Conj",
                )
                .replace(
                    "`OrderedRing` has no canonical field labeled",
                    "OrderedRing has no canonical field labeled",
                ),
        ),
    })
}

fn variant_name_for_decl(
    dag: &Dag,
    disj_id: DeclarationId,
    variant_id: DeclarationId,
) -> Result<String, EmitPythonError> {
    let TypeConnective::Disj { variants } = &dag.declaration(disj_id).connective else {
        unreachable!("variant_name_for_decl requires a disjunction")
    };
    variants
        .iter()
        .find(|variant| variant.ty == variant_id)
        .map(|variant| variant.label.clone())
        .ok_or_else(|| {
            EmitPythonError::Unsupported(
                "variant id not found under parent disjunction".to_string(),
            )
        })
}

fn runtime_variant_name_for_decl(
    dag: &Dag,
    disj_id: DeclarationId,
    variant_id: DeclarationId,
) -> Result<String, EmitPythonError> {
    let variant_name = variant_name_for_decl(dag, disj_id, variant_id)?;
    Ok(dag
        .declaration(disj_id)
        .name
        .as_deref()
        .map(|enum_name| python_variant_class_name(enum_name, &variant_name))
        .unwrap_or(variant_name))
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

fn python_variant_class_name(enum_name: &str, variant_name: &str) -> String {
    format!("{enum_name}_{variant_name}")
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
) -> Result<DeclarationId, EmitPythonError> {
    let TypeConnective::Arrow { inputs, .. } = &dag.declaration(template).connective else {
        return Err(EmitPythonError::Unsupported(
            "realized callable template did not resolve to an Arrow".to_string(),
        ));
    };
    let Some(param_decl) = inputs.get(input_index).copied() else {
        return Err(EmitPythonError::Unsupported(
            "realized callable slot is missing from the template declaration".to_string(),
        ));
    };
    arguments
        .iter()
        .find(|arg| arg.parameter == param_decl)
        .map(|arg| arg.value)
        .ok_or_else(|| {
            EmitPythonError::Unsupported(
                "callable argument did not bind through template instantiation".to_string(),
            )
        })
}

fn find_resolved_branch_path(branch: &BranchNode, variant_id: DeclarationId) -> Option<&Path> {
    branch.paths.iter().find(|path| match &path.pattern {
        BranchPattern::ResolvedVariant(id) => *id == variant_id,
        BranchPattern::UnresolvedVariant { .. } => false,
    })
}

fn resolved_pattern_id(path: &Path) -> Result<DeclarationId, EmitPythonError> {
    match &path.pattern {
        BranchPattern::ResolvedVariant(id) => Ok(*id),
        BranchPattern::UnresolvedVariant { name, .. } => {
            Err(EmitPythonError::UnresolvedBranchPattern {
                variant_name: name.clone(),
            })
        }
    }
}
