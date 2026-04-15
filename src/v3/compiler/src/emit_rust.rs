// M1(3) PR-B — Rust emitter.
//
// The first v3 downstream consumer. Reads the DAG structurally plus
// the Realization facts declared in `dsl/extdeps/languages/rust.dag`
// and produces Rust source. No Rust-side knowledge of Rust syntax
// lives in this file — every per-op string (operator symbol, type
// name, template shape) is a data fact looked up via the realization
// index. Adding a second target language (`go.dag`, `python.dag`)
// is another spec file plus a 40-line `emit_go` / `emit_python` that
// reuses the same walk.
//
// Scope at PR-B:
//   - Value literals (Int, Bool, String)
//   - Arithmetic + comparison operators on Int
//   - if/else branches (as Rust if-expressions)
//   - Top-level value Binds (as let statements)
//   - Outer `fn main() { ... println!("{}", x) }` wrapper
//
// Out of scope (follow-up work, tracked in DOWNSTREAM_REQUIREMENTS):
//   - User-defined functions (Bind with non-empty params)
//   - Loops
//   - TransformTarget::Callable dispatch
//   - Record / enum construction
//
// Template placeholders (substituted by `render_template`):
//   %N  Bind name               %C  branch condition
//   %T  Rust type name          %H  then-arm body
//   %V  bind value expression   %E  else-arm body
//   %B  list of let statements  %Q  literal double-quote
//   %X  final bind's name
//
// See the header of `dsl/extdeps/languages/rust.dag` for the
// authoritative placeholder docs and the carrier strings each
// template reads.

use std::collections::HashMap;

use crate::dag::{
    Behavior, BranchNode, BranchPattern, Dag, LiteralBits, Path, PortId, TransformNode,
    TransformTarget, TypeConnective, ValueBody, ValueNode,
};
use crate::operators::OperatorKind;

/// Errors the Rust emitter surfaces when the DAG reaches a shape it
/// cannot render under the PR-B scope. Each variant names a specific
/// structural cause — no catch-all `Unknown` — so consumers can
/// classify the failure against `dsl/extdeps/languages/rust.dag`'s
/// coverage.
#[derive(Debug, Clone)]
pub enum EmitError {
    /// No Realization was declared in rust.dag for this
    /// `(target_name, op_name)` pair. Add a `data rust_*: Realization`
    /// entry with the missing carrier to close the gap.
    MissingRealization {
        target_name: String,
        op_name: String,
    },
    /// A port has no resolved TypeShape, so its Rust type name can't
    /// be looked up. Inference should have driven every port to
    /// Resolved before emit runs; reaching this arm is a bug.
    UntypedPort(PortId),
    /// The DAG carries a behavior variant PR-B doesn't render yet
    /// (Loop, user-function Bind, TransformTarget::Callable, etc.).
    UnsupportedBehavior(String),
    /// A Branch arm's pattern stayed `UnresolvedVariant` past
    /// inference — either inference didn't run or the scrutinee's
    /// Disj has no matching variant. Emission needs the resolved
    /// label to pick between `%H` and `%E`.
    UnresolvedBranchPattern {
        variant_name: String,
    },
    /// An `if`/`else` branch's scrutinee resolved to a variant
    /// declaration that isn't labeled `True` or `False` in the
    /// parent Disj. PR-B only emits boolean branches.
    UnknownBranchArm { label: String },
}

/// Key for the realization index: (target_name, op_name). Use empty
/// `op_name` for type-only mappings and structural templates.
type RealizationKey = (String, String);

/// Pre-built index from `(target_name, op_name)` to the `carrier`
/// string declared in rust.dag. Built once per `emit_rust` call from
/// the declarations with `meta_tag == Some(Realization)` and
/// `value_body == Some(Structural { .. })`. Everything else in the
/// emitter reads through this table.
struct RealizationIndex {
    table: HashMap<RealizationKey, String>,
}

impl RealizationIndex {
    fn build(dag: &Dag) -> Self {
        let realization_id = dag
            .declaration_by_name("Realization")
            .map(|d| d.id);
        let mut table = HashMap::new();
        if let Some(realization_id) = realization_id {
            for decl in dag.declarations() {
                if decl.meta_tag != Some(realization_id) {
                    continue;
                }
                let Some(ValueBody::Structural { fields }) = &decl.value_body else {
                    continue;
                };
                let target_name = structural_string_field(fields, "target_name");
                let op_name = structural_string_field(fields, "op_name");
                let carrier = structural_string_field(fields, "carrier");
                if let (Some(t), Some(o), Some(c)) = (target_name, op_name, carrier) {
                    table.insert((t, o), c);
                }
            }
        }
        Self { table }
    }

    fn lookup(&self, target_name: &str, op_name: &str) -> Option<&str> {
        self.table
            .get(&(target_name.to_string(), op_name.to_string()))
            .map(|s| s.as_str())
    }
}

fn structural_string_field(
    fields: &[(String, LiteralBits)],
    label: &str,
) -> Option<String> {
    fields.iter().find(|(l, _)| l == label).and_then(|(_, bits)| {
        if let LiteralBits::String(s) = bits {
            Some(s.clone())
        } else {
            None
        }
    })
}

pub fn emit_rust(dag: &Dag) -> Result<String, EmitError> {
    let index = RealizationIndex::build(dag);

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

    let mut rendered_binds: Vec<String> = Vec::with_capacity(top_level_binds.len());
    for bind in &top_level_binds {
        let ty_name = rust_type_name_for_port(dag, &index, bind.value)?;
        let value_expr = render_top_level_value(dag, &index, &bound_names, bind.value)?;
        let let_template = index
            .lookup("Bind", "")
            .ok_or_else(|| EmitError::MissingRealization {
                target_name: "Bind".to_string(),
                op_name: String::new(),
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

    let main_template = index
        .lookup("Main", "")
        .ok_or_else(|| EmitError::MissingRealization {
            target_name: "Main".to_string(),
            op_name: String::new(),
        })?;
    let program = main_template
        .replace("%B", &body_joined)
        .replace("%X", &final_bind_name)
        .replace("%Q", "\"");
    Ok(program)
}

/// Render the sub-expression rooted at `port` into a Rust
/// expression string. Recursive in Port → Behavior → input Ports,
/// bottoming out at Value leaves.
///
/// First checks `bound_names` — if the port already corresponds to a
/// top-level let binding, it renders as the name (e.g. `b` reuses
/// `a` by name, not by re-inlining `a`'s value expression). The
/// top-level emit loop bypasses this lookup for the very port it's
/// rendering via `render_top_level_value`.
fn render_port(
    dag: &Dag,
    index: &RealizationIndex,
    bound_names: &HashMap<PortId, String>,
    port: PortId,
) -> Result<String, EmitError> {
    if let Some(name) = bound_names.get(&port) {
        return Ok(name.clone());
    }
    dispatch_producer(dag, index, bound_names, port)
}

/// Render the value for a top-level let binding. Bypasses the
/// `bound_names` lookup for `port` itself (which would short-circuit
/// to the name and produce `let x: i64 = x;`) but recursive sub-walks
/// still go through `render_port` and DO use `bound_names`.
fn render_top_level_value(
    dag: &Dag,
    index: &RealizationIndex,
    bound_names: &HashMap<PortId, String>,
    port: PortId,
) -> Result<String, EmitError> {
    dispatch_producer(dag, index, bound_names, port)
}

fn dispatch_producer(
    dag: &Dag,
    index: &RealizationIndex,
    bound_names: &HashMap<PortId, String>,
    port: PortId,
) -> Result<String, EmitError> {
    let produced_by = dag.port(port).produced_by;
    let Some(node_id) = produced_by else {
        return Err(EmitError::UnsupportedBehavior(
            "dispatch_producer reached a port with no producer (parameter?)".to_string(),
        ));
    };
    match dag.node(node_id) {
        Behavior::Value(v) => Ok(render_value(v)),
        Behavior::Transform(t) => render_transform(dag, index, bound_names, t),
        Behavior::Branch(b) => render_branch(dag, index, bound_names, b),
        Behavior::Loop(_) => Err(EmitError::UnsupportedBehavior(
            "Loop behavior is not yet supported by emit_rust".to_string(),
        )),
        Behavior::Bind(b) => Ok(b.name.clone()),
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

fn render_transform(
    dag: &Dag,
    index: &RealizationIndex,
    bound_names: &HashMap<PortId, String>,
    t: &TransformNode,
) -> Result<String, EmitError> {
    match t.target {
        TransformTarget::Operator(op) => render_operator(dag, index, bound_names, t, op),
        TransformTarget::Callable(_) => Err(EmitError::UnsupportedBehavior(
            "TransformTarget::Callable (user function call) is not yet supported by emit_rust"
                .to_string(),
        )),
    }
}

fn render_operator(
    dag: &Dag,
    index: &RealizationIndex,
    bound_names: &HashMap<PortId, String>,
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
    let operand_type_name = declared_type_name_for_port(dag, t.inputs[0])?;
    let op_field = op.algebra_field_name().to_string();
    let carrier = index
        .lookup(&operand_type_name, &op_field)
        .ok_or_else(|| EmitError::MissingRealization {
            target_name: operand_type_name.clone(),
            op_name: op_field.clone(),
        })?
        .to_string();
    let lhs = render_port(dag, index, bound_names, t.inputs[0])?;
    let rhs = render_port(dag, index, bound_names, t.inputs[1])?;
    Ok(format!("({} {} {})", lhs, carrier, rhs))
}

fn render_branch(
    dag: &Dag,
    index: &RealizationIndex,
    bound_names: &HashMap<PortId, String>,
    b: &BranchNode,
) -> Result<String, EmitError> {
    let (then_path, else_path) = split_bool_paths(dag, b)?;
    let cond = render_port(dag, index, bound_names, b.input)?;
    let then_expr = render_port(dag, index, bound_names, then_path.output)?;
    let else_expr = render_port(dag, index, bound_names, else_path.output)?;
    let template = index
        .lookup("Branch", "")
        .ok_or_else(|| EmitError::MissingRealization {
            target_name: "Branch".to_string(),
            op_name: String::new(),
        })?;
    Ok(template
        .replace("%C", &cond)
        .replace("%H", &then_expr)
        .replace("%E", &else_expr))
}

/// Sort a Branch's paths into (then, else) for `if`/`else` emission.
/// Requires each path's pattern to be a `ResolvedVariant` whose parent
/// Disj labels it `True` or `False` — PR-B only emits boolean
/// branches.
fn split_bool_paths<'a>(
    dag: &'a Dag,
    b: &'a BranchNode,
) -> Result<(&'a Path, &'a Path), EmitError> {
    let mut then_path: Option<&Path> = None;
    let mut else_path: Option<&Path> = None;
    for path in &b.paths {
        let label = branch_pattern_label(dag, &path.pattern)?;
        match label.as_str() {
            "True" => then_path = Some(path),
            "False" => else_path = Some(path),
            other => {
                return Err(EmitError::UnknownBranchArm {
                    label: other.to_string(),
                })
            }
        }
    }
    match (then_path, else_path) {
        (Some(t), Some(e)) => Ok((t, e)),
        _ => Err(EmitError::UnsupportedBehavior(
            "if/else branch must have both True and False arms".to_string(),
        )),
    }
}

/// Look up a ResolvedVariant's label by walking the scrutinee-side
/// Disj and finding the matching variant declaration id. The
/// substrate stores variant declarations anonymously and keeps the
/// label on the parent Disj's `Field.label`; emission walks that
/// edge backward.
fn branch_pattern_label(
    dag: &Dag,
    pattern: &BranchPattern,
) -> Result<String, EmitError> {
    let variant_id = match pattern {
        BranchPattern::ResolvedVariant(id) => *id,
        BranchPattern::UnresolvedVariant { name, .. } => {
            return Err(EmitError::UnresolvedBranchPattern {
                variant_name: name.clone(),
            });
        }
    };
    for decl in dag.declarations() {
        if let TypeConnective::Disj { variants } = &decl.connective {
            if let Some(field) = variants.iter().find(|f| f.ty == variant_id) {
                return Ok(field.label.clone());
            }
        }
    }
    Err(EmitError::UnsupportedBehavior(format!(
        "variant declaration {variant_id:?} has no parent Disj — cannot resolve label"
    )))
}

/// Read a port's Rust type name via the Realization index.
///
/// Walks the port's TypeShape → declaration → declaration name →
/// `find_realization(name, "")`. Fails fail-closed if the port is
/// untyped or the realization is missing.
fn rust_type_name_for_port(
    dag: &Dag,
    index: &RealizationIndex,
    port: PortId,
) -> Result<String, EmitError> {
    let declared = declared_type_name_for_port(dag, port)?;
    let carrier = index
        .lookup(&declared, "")
        .ok_or(EmitError::MissingRealization {
            target_name: declared.clone(),
            op_name: String::new(),
        })?;
    Ok(carrier.to_string())
}

/// Read a port's declared primitive name (e.g., "Int", "Bool",
/// "String"). Walks alias declarations if the port's TypeShape is an
/// Instantiation or ResolvedIdentifier.
fn declared_type_name_for_port(dag: &Dag, port: PortId) -> Result<String, EmitError> {
    let ts = dag
        .port(port)
        .value_type()
        .ok_or(EmitError::UntypedPort(port))?;
    let mut current = ts.declaration;
    for _ in 0..32 {
        let decl = dag.declaration(current);
        if let Some(name) = &decl.name {
            return Ok(name.clone());
        }
        match &decl.connective {
            TypeConnective::Instantiation { template, .. } => current = *template,
            TypeConnective::Atom(crate::dag::AtomPayload::ResolvedIdentifier(next)) => {
                current = *next
            }
            _ => {
                return Err(EmitError::UnsupportedBehavior(format!(
                    "port type walks to anonymous declaration {current:?} — no name to look up in rust.dag"
                )));
            }
        }
    }
    Err(EmitError::UnsupportedBehavior(
        "port type walk exceeded depth 32 — likely a cycle".to_string(),
    ))
}

