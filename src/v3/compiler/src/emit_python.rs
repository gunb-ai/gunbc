use std::collections::HashMap;

use crate::dag::{
    ArrowBody, AtomPayload, Behavior, BindNode, BranchNode, BranchPattern, DeclarationId, Field,
    FieldValue, LiteralBits, Path, PortId, TemplateArgument, TransformNode, TransformTarget,
    TypeConnective,
};
use crate::operators::OperatorKind;
use crate::variant_payload::{
    variant_payload_shape, VariantPayloadBinding, VariantPayloadFieldAccessRuleBinding,
    VariantPayloadShape,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EmitPythonMode {
    Program,
    Module,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PythonCallableStrategy {
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
    binary_op: String,
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
struct PythonIndexes {
    types: HashMap<DeclarationId, String>,
    type_instantiations: HashMap<DeclarationId, String>,
    operators: HashMap<(DeclarationId, DeclarationId), String>,
    callables: HashMap<DeclarationId, PythonCallableStrategy>,
    syntax: PythonSyntax,
    target: PythonTarget,
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
        let type_meta = named_decl(dag, "PythonTypeRealization")?;
        let type_instantiation_meta = named_decl(dag, "PythonTypeInstantiationRealization")?;
        let operator_meta = named_decl(dag, "PythonOperatorRealization")?;
        let callable_meta = named_decl(dag, "PythonCallableRealization")?;
        let mut types = HashMap::new();
        let mut type_instantiations = HashMap::new();
        let mut operators = HashMap::new();
        let mut callables = HashMap::new();

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
                || meta_tag == callable_meta;
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
            if meta_tag == type_meta {
                let target = require_field_decl_ref(fields, "target", decl.id)?;
                let carrier = require_field_string(fields, "carrier", decl.id)?;
                if types.insert(target, carrier).is_some() {
                    return Err(EmitPythonError::DuplicateRealization {
                        declaration: decl.id,
                        detail: "two PythonTypeRealization data items target the same declaration",
                    });
                }
            } else if meta_tag == type_instantiation_meta {
                let target = require_field_decl_ref(fields, "target", decl.id)?;
                let carrier = require_field_string(fields, "carrier", decl.id)?;
                if type_instantiations.insert(target, carrier).is_some() {
                    return Err(EmitPythonError::DuplicateRealization {
                        declaration: decl.id,
                        detail: "two PythonTypeInstantiationRealization data items target the same declaration",
                    });
                }
            } else if meta_tag == operator_meta {
                let target = require_field_decl_ref(fields, "target", decl.id)?;
                let op = require_field_decl_ref(fields, "op", decl.id)?;
                let carrier = require_field_string(fields, "carrier", decl.id)?;
                if operators.insert((target, op), carrier).is_some() {
                    return Err(EmitPythonError::DuplicateRealization {
                        declaration: decl.id,
                        detail:
                            "two PythonOperatorRealization data items share the same (target, op) pair",
                    });
                }
            } else if meta_tag == callable_meta {
                let target = require_field_decl_ref(fields, "target", decl.id)?;
                let strategy = parse_callable_strategy(dag, fields, decl.id)?;
                if callables.insert(target, strategy).is_some() {
                    return Err(EmitPythonError::DuplicateRealization {
                        declaration: decl.id,
                        detail:
                            "two PythonCallableRealization data items target the same callable declaration",
                    });
                }
            }
        }

        let expressions = structural_fields_for_named(dag, "python_expressions")?;
        let collections = structural_fields_for_named(dag, "python_collections")?;
        let type_apps = structural_fields_for_named(dag, "python_type_applications")?;
        let target_fields = structural_fields_for_named(dag, "python_target")?;

        let syntax = PythonSyntax {
            binary_op: require_field_string(
                expressions,
                "binary_op",
                named_decl(dag, "python_expressions")?,
            )?,
            field_access: require_field_string(
                expressions,
                "field_access",
                named_decl(dag, "python_expressions")?,
            )?,
            function_call: require_field_string(
                expressions,
                "function_call",
                named_decl(dag, "python_expressions")?,
            )?,
            closure: require_field_string(
                expressions,
                "closure",
                named_decl(dag, "python_expressions")?,
            )?,
            empty_list: require_field_string(
                collections,
                "empty_list",
                named_decl(dag, "python_collections")?,
            )?,
            list_literal: require_field_string(
                collections,
                "list_literal",
                named_decl(dag, "python_collections")?,
            )?,
            cons: require_field_string(
                collections,
                "cons",
                named_decl(dag, "python_collections")?,
            )?,
            concat: require_field_string(
                collections,
                "concat",
                named_decl(dag, "python_collections")?,
            )?,
            length: require_field_string(
                collections,
                "length",
                named_decl(dag, "python_collections")?,
            )?,
            is_empty: require_field_string(
                collections,
                "is_empty",
                named_decl(dag, "python_collections")?,
            )?,
            fold: require_field_string(
                collections,
                "fold",
                named_decl(dag, "python_collections")?,
            )?,
            map: require_field_string(collections, "map", named_decl(dag, "python_collections")?)?,
            filter: require_field_string(
                collections,
                "filter",
                named_decl(dag, "python_collections")?,
            )?,
            contains: require_field_string(
                collections,
                "contains",
                named_decl(dag, "python_collections")?,
            )?,
            optional: require_field_string(
                type_apps,
                "optional",
                named_decl(dag, "python_type_applications")?,
            )?,
        };

        let target_decl = named_decl(dag, "python_target")?;
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
            syntax,
            target,
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

pub fn emit_python(dag: &Dag) -> Result<String, EmitPythonError> {
    emit_python_with_mode(dag, EmitPythonMode::Program)
}

pub fn emit_python_module(dag: &Dag) -> Result<String, EmitPythonError> {
    emit_python_with_mode(dag, EmitPythonMode::Module)
}

fn emit_python_with_mode(dag: &Dag, mode: EmitPythonMode) -> Result<String, EmitPythonError> {
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
    let top_level_binds: Vec<&BindNode> = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
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

    for decl in type_decls {
        sections.push(ctx.render_type_declaration(decl)?);
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
            TransformTarget::FieldProject { field_label, .. } => {
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
        // Logical operators are Bool-monomorphic and do not dispatch
        // through a Bool algebra today — render the Python keyword
        // form directly. `&&` / `||` in source become `and` / `or`.
        if let OperatorKind::Logical(logical_op) = op {
            let symbol = match logical_op {
                crate::operators::LogicalOp::And => "and",
                crate::operators::LogicalOp::Or => "or",
            };
            let lhs = self.render_port(t.inputs[0], locals)?;
            let rhs = self.render_port(t.inputs[1], locals)?;
            return Ok(render_named_template(
                &self.indexes.syntax.binary_op,
                &[("lhs", &lhs), ("op", symbol), ("rhs", &rhs)],
            ));
        }
        let operand_type = primitive_type_id_for_port(self.dag, t.inputs[0])?;
        let op_decl = algebra_field_for_operator(self.dag, operand_type, op)?;
        let carrier = self.indexes.operators.get(&(operand_type, op_decl)).ok_or(
            EmitPythonError::MissingOperatorRealization {
                target: operand_type,
                op: op_decl,
            },
        )?;
        let lhs = self.render_port(t.inputs[0], locals)?;
        let rhs = self.render_port(t.inputs[1], locals)?;
        Ok(render_named_template(
            &self.indexes.syntax.binary_op,
            &[("lhs", &lhs), ("op", carrier), ("rhs", &rhs)],
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
        if let Some(rendered) = self.render_list_branch(branch, locals)? {
            return Ok(rendered);
        }
        self.render_general_match(branch, locals)
    }

    fn render_list_branch(
        &self,
        branch: &BranchNode,
        locals: &RenderLocals,
    ) -> Result<Option<String>, EmitPythonError> {
        let scrutinee_type_id = primitive_type_id_for_port(self.dag, branch.input)?;
        let Some(disj_id) = walk_to_disj(self.dag, scrutinee_type_id) else {
            return Ok(None);
        };
        if !self.dag.list_template().is_some_and(|list| list == disj_id) {
            return Ok(None);
        }
        let TypeConnective::Disj { variants } = &self.dag.declaration(disj_id).connective else {
            return Ok(None);
        };
        let empty_variant = variants.iter().find(|v| v.label == "Empty").map(|v| v.ty);
        let cons_variant = variants.iter().find(|v| v.label == "Cons").map(|v| v.ty);
        let (Some(empty_variant), Some(cons_variant)) = (empty_variant, cons_variant) else {
            return Ok(None);
        };
        let Some(empty_path) = find_resolved_branch_path(branch, empty_variant) else {
            return Ok(None);
        };
        let Some(cons_path) = find_resolved_branch_path(branch, cons_variant) else {
            return Ok(None);
        };
        let scrutinee = self.render_port(branch.input, locals)?;
        let empty_body = self.render_path_body(empty_path, locals)?;
        let mut cons_locals = locals.clone();
        if let Some(binding) = &cons_path.binding {
            cons_locals.payload_bindings.insert(
                binding.payload_port,
                VariantPayloadBinding::Direct(
                    "types.SimpleNamespace(head=__match[0], tail=__match[1:])".to_string(),
                ),
            );
        }
        let cons_body = self.render_port(cons_path.output, &cons_locals)?;
        Ok(Some(format!(
            "(lambda __match: ({empty_body} if len(__match) == 0 else {cons_body}))({scrutinee})"
        )))
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
            let variant_name = variant_name_for_decl(self.dag, disj_id, variant_id)?;
            return Ok(if variant_name == "None" {
                "__match is None".to_string()
            } else {
                "__match is not None".to_string()
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
        let Some(shape) = variant_payload_shape(self.dag, variant_id) else {
            return Ok(Some(VariantPayloadBinding::Direct("__match".to_string())));
        };
        Ok(match shape {
            VariantPayloadShape::Empty => None,
            VariantPayloadShape::PositionalSingle => {
                Some(VariantPayloadBinding::Direct("__match._0".to_string()))
            }
            VariantPayloadShape::NamedFields(field_labels) => {
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
        _loop_node: &crate::dag::LoopNode,
        _locals: &RenderLocals,
    ) -> Result<String, EmitPythonError> {
        // emit_python does not yet model `Behavior::Loop`. Earlier
        // code rendered just the loop body's result port, silently
        // dropping iteration semantics — a Loop became its first
        // iteration's expression. Fail-closed instead so callers see
        // the unsupported case directly.
        Err(EmitPythonError::Unsupported(
            "emit_python does not yet support Behavior::Loop; iteration construct must be expressed via fold/map/filter callables for now"
                .to_string(),
        ))
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
        strategy: PythonCallableStrategy,
        arguments: &[TemplateArgument],
        inputs: &[PortId],
        locals: &RenderLocals,
    ) -> Result<String, EmitPythonError> {
        match strategy {
            PythonCallableStrategy::Empty => Ok(self.indexes.syntax.empty_list.clone()),
            PythonCallableStrategy::Singleton => {
                let value = self.render_port(inputs[0], locals)?;
                Ok(render_named_template(
                    &self.indexes.syntax.list_literal,
                    &[("elements", &value)],
                ))
            }
            PythonCallableStrategy::Cons => {
                let head = self.render_port(inputs[0], locals)?;
                let tail = self.render_port(inputs[1], locals)?;
                Ok(render_named_template(
                    &self.indexes.syntax.cons,
                    &[("head", &head), ("tail", &tail)],
                ))
            }
            PythonCallableStrategy::Concat => {
                let left = self.render_port(inputs[0], locals)?;
                let right = self.render_port(inputs[1], locals)?;
                Ok(render_named_template(
                    &self.indexes.syntax.concat,
                    &[("left", &left), ("right", &right)],
                ))
            }
            PythonCallableStrategy::Length => {
                let recv = self.render_port(inputs[0], locals)?;
                Ok(render_named_template(
                    &self.indexes.syntax.length,
                    &[("recv", &recv)],
                ))
            }
            PythonCallableStrategy::IsEmpty => {
                let recv = self.render_port(inputs[0], locals)?;
                Ok(render_named_template(
                    &self.indexes.syntax.is_empty,
                    &[("recv", &recv)],
                ))
            }
            PythonCallableStrategy::Fold => {
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
            PythonCallableStrategy::Map => {
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
            PythonCallableStrategy::Filter => {
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
            PythonCallableStrategy::Contains => {
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
        if children.len() == 1 && children[0].label == "_0" {
            let arg = self.render_port(inputs[0], locals)?;
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
        let bind = self
            .dag
            .node(*bind_id)
            .as_bind()
            .expect("UserDefined arrow body must point at a Bind");
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
        let bind = self
            .dag
            .node(*bind_id)
            .as_bind()
            .expect("UserDefined arrow body must point at a Bind");
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
            TypeConnective::Cardinality {
                element,
                bound: crate::dag::CardinalityBound::AtMostOne,
            } => {
                let inner = self.python_type_name_for_decl_at_depth(*element, depth + 1)?;
                Ok(render_named_template(
                    &self.indexes.syntax.optional,
                    &[("element", &inner)],
                ))
            }
            TypeConnective::Atom(AtomPayload::ResolvedIdentifier(next)) => {
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
        LiteralBits::Int(n) => n.to_string(),
        LiteralBits::Bool(true) => "True".to_string(),
        LiteralBits::Bool(false) => "False".to_string(),
        LiteralBits::String(s) => format!("{:?}", s),
    }
}

fn parse_callable_strategy(
    dag: &Dag,
    fields: &[(String, FieldValue)],
    declaration: DeclarationId,
) -> Result<PythonCallableStrategy, EmitPythonError> {
    let (constructor, payload) = variant_field(fields, "strategy", declaration)?;
    if !payload.is_empty() {
        return Err(EmitPythonError::MalformedSpec {
            declaration,
            detail: "PythonCallableStrategy variants must not carry payload",
        });
    }
    let variants = [
        ("ListEmpty", PythonCallableStrategy::Empty),
        ("ListSingleton", PythonCallableStrategy::Singleton),
        ("ListCons", PythonCallableStrategy::Cons),
        ("ListConcat", PythonCallableStrategy::Concat),
        ("ListLength", PythonCallableStrategy::Length),
        ("ListIsEmpty", PythonCallableStrategy::IsEmpty),
        ("ListFold", PythonCallableStrategy::Fold),
        ("ListMap", PythonCallableStrategy::Map),
        ("ListFilter", PythonCallableStrategy::Filter),
        ("ListContains", PythonCallableStrategy::Contains),
    ];
    for (name, strategy) in variants {
        if constructor == named_variant_id(dag, "PythonCallableStrategy", name)? {
            return Ok(strategy);
        }
    }
    Err(EmitPythonError::MalformedSpec {
        declaration,
        detail: "unsupported PythonCallableStrategy variant",
    })
}

fn named_decl(dag: &Dag, name: &'static str) -> Result<DeclarationId, EmitPythonError> {
    dag.declaration_by_name(name)
        .map(|decl| decl.id)
        .ok_or(EmitPythonError::MissingMeta(name))
}

fn structural_fields_for_named<'a>(
    dag: &'a Dag,
    name: &'static str,
) -> Result<&'a [(String, FieldValue)], EmitPythonError> {
    let decl = dag
        .declaration_by_name(name)
        .ok_or(EmitPythonError::MissingSpec(name))?;
    structural_fields(decl).ok_or(EmitPythonError::MalformedSpec {
        declaration: decl.id,
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
    let variants = [
        ("ValueOnly", MemoryModelBinding::ValueOnly),
        ("GarbageCollected", MemoryModelBinding::GarbageCollected),
        ("RefCounted", MemoryModelBinding::RefCounted),
        ("OwnershipBased", MemoryModelBinding::OwnershipBased),
    ];
    for (label, binding) in variants {
        let variant_id = named_variant_id(dag, "MemoryModel", label)?;
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
    let variants = [
        ("LexicalScoping", ScopeModelBinding::LexicalScoping),
        ("DynamicScoping", ScopeModelBinding::DynamicScoping),
    ];
    for (label, binding) in variants {
        let variant_id = named_variant_id(dag, "ScopeModel", label)?;
        if constructor == variant_id {
            return Ok(binding);
        }
    }
    Err(EmitPythonError::MalformedSpec {
        declaration,
        detail: "TargetExecutionModel.scope must be LexicalScoping/DynamicScoping",
    })
}

fn named_variant_id(
    dag: &Dag,
    parent_name: &str,
    variant_label: &str,
) -> Result<DeclarationId, EmitPythonError> {
    let parent = dag
        .declaration_by_name(parent_name)
        .ok_or(EmitPythonError::MissingMeta("variant parent"))?;
    let TypeConnective::Disj { variants } = &parent.connective else {
        return Err(EmitPythonError::Unsupported(format!(
            "{parent_name} is not a disjunction"
        )));
    };
    variants
        .iter()
        .find(|variant| variant.label == variant_label)
        .map(|variant| variant.ty)
        .ok_or(EmitPythonError::MissingMeta("variant"))
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

fn is_bootstrap_file(file: &str) -> bool {
    file.starts_with("dsl/std/")
        || file.starts_with("src/v3/std/")
        || file.starts_with("src/v3/spec/")
}

fn primitive_type_id_for_port(dag: &Dag, port: PortId) -> Result<DeclarationId, EmitPythonError> {
    let ts = dag
        .port(port)
        .value_type()
        .ok_or(EmitPythonError::UntypedPort(port))?;
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
    Err(EmitPythonError::Unsupported(
        "port type walk exceeded depth 32".to_string(),
    ))
}

fn walk_to_disj(dag: &Dag, start: DeclarationId) -> Option<DeclarationId> {
    let mut current = start;
    for _ in 0..32 {
        match &dag.declaration(current).connective {
            TypeConnective::Disj { .. } => return Some(current),
            TypeConnective::Cardinality {
                bound: crate::dag::CardinalityBound::AtMostOne,
                ..
            } => return dag.optional_match_disj(current),
            TypeConnective::Instantiation { template, .. } => current = *template,
            TypeConnective::Atom(AtomPayload::ResolvedIdentifier(next)) => current = *next,
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
    if let Some(algebra_conj_id) = walk_to_algebra_conj(dag, operand_type_id) {
        let field_label = op.algebra_field_name();
        let children = match &dag.declaration(algebra_conj_id).connective {
            TypeConnective::Conj { children } => children,
            _ => unreachable!("walk_to_algebra_conj returned a non-Conj"),
        };
        if let Some(field) = children.iter().find(|f| f.label == field_label) {
            return Ok(field.ty);
        }
    }
    canonical_operator_field(dag, op)
}

/// Walk a declaration through aliases / instantiations until it
/// reaches a Conj (the algebra declaration). Returns the Conj's id.
/// Mirrors emit_rust::walk_to_algebra_conj.
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

fn canonical_operator_field(dag: &Dag, op: OperatorKind) -> Result<DeclarationId, EmitPythonError> {
    let field_label = op.algebra_field_name();
    let ordered_ring = dag.declaration_by_name("OrderedRing").ok_or_else(|| {
        EmitPythonError::Unsupported(
            "bootstrap is missing canonical OrderedRing declaration".to_string(),
        )
    })?;
    let TypeConnective::Conj { children } = &ordered_ring.connective else {
        return Err(EmitPythonError::Unsupported(
            "OrderedRing did not lower to a Conj".to_string(),
        ));
    };
    children
        .iter()
        .find(|field| field.label == field_label)
        .map(|field| field.ty)
        .ok_or_else(|| {
            EmitPythonError::Unsupported(format!(
                "OrderedRing has no canonical field labeled {field_label}"
            ))
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
