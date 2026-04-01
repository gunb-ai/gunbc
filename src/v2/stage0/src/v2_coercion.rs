// Coercion data structures and per-language registries.
//
// Mirrors std/coercion.dag: TypeCheckpoint, InhabitantDecl, CallableRepr.
// Per-language registries populated from dsl/extdeps/languages/*/types.dag data.
//
// Authority chain:
//   std/coercion.dag           → shared schema (types)
//   extdeps/languages/*/types.dag → per-language data
//   THIS FILE                  → stage0 Rust mirror (consumed by build_type_rendering)
//
// When build_type_rendering reads this data instead of RUST_TYPE_MAP etc.,
// the type-rendering pipeline becomes data-driven from .dag declarations.

use std::collections::HashMap;

/// Primitive scalar coercion: .dag type name → target language type.
/// Checked FIRST before algebra resolution (fast path).
#[derive(Debug, Clone)]
pub struct TypeCheckpoint {
    pub dag_name: String,
    pub target_type: String,
    pub default_expr: Option<String>,
    pub is_copy: Option<bool>,
}

/// Algebra inhabitant: target type template for a .dag algebraic structure.
/// Fallback when no TypeCheckpoint matches.
#[derive(Debug, Clone)]
pub struct InhabitantDecl {
    pub algebra: String,
    pub template: String,
    pub arity: usize,
    pub identity_expr: Option<String>,
    pub import_path: Option<String>,
    pub is_copy: Option<bool>,
}

/// Callable type rendering for a target language.
#[derive(Debug, Clone)]
pub struct CallableRepr {
    pub template: String,
    pub param_separator: String,
    pub return_separator: String,
    pub import_path: Option<String>,
}

/// Per-language coercion registry: all data needed for type coercion.
#[derive(Debug, Clone)]
pub struct CoercionRegistry {
    checkpoint_map: HashMap<String, TypeCheckpoint>,
    inhabitant_map: HashMap<String, InhabitantDecl>,
    pub callable: CallableRepr,
    pub optional_template: String,
}

impl CoercionRegistry {
    pub fn new(
        checkpoints: Vec<TypeCheckpoint>,
        inhabitants: Vec<InhabitantDecl>,
        callable: CallableRepr,
        optional_template: String,
    ) -> Self {
        let mut checkpoint_map = HashMap::new();
        for cp in checkpoints {
            checkpoint_map.insert(cp.dag_name.clone(), cp);
        }
        let mut inhabitant_map = HashMap::new();
        for inh in inhabitants {
            inhabitant_map.insert(inh.algebra.clone(), inh);
        }
        CoercionRegistry {
            checkpoint_map,
            inhabitant_map,
            callable,
            optional_template,
        }
    }

    /// Level 1: checkpoint lookup. Returns target type if .dag name is checkpointed.
    pub fn lookup_checkpoint(&self, dag_name: &str) -> Option<&TypeCheckpoint> {
        self.checkpoint_map.get(dag_name)
    }

    /// Level 2: algebra inhabitant lookup. Returns inhabitant if algebra is declared.
    pub fn lookup_inhabitant(&self, algebra: &str) -> Option<&InhabitantDecl> {
        self.inhabitant_map.get(algebra)
    }

    /// Is this .dag type Copy in the target language?
    pub fn is_copy(&self, dag_name: &str) -> Option<bool> {
        self.checkpoint_map.get(dag_name).and_then(|cp| cp.is_copy)
    }

    /// All declared algebra names (for deriving container identity).
    pub fn algebra_names(&self) -> impl Iterator<Item = &str> {
        self.inhabitant_map.keys().map(|s| s.as_str())
    }

    /// Element container algebras (arity == 1).
    pub fn element_container_algebras(&self) -> Vec<&str> {
        self.inhabitant_map.values()
            .filter(|inh| inh.arity == 1)
            .map(|inh| inh.algebra.as_str())
            .collect()
    }

    /// Keyed container algebras (arity == 2).
    pub fn keyed_container_algebras(&self) -> Vec<&str> {
        self.inhabitant_map.values()
            .filter(|inh| inh.arity == 2)
            .map(|inh| inh.algebra.as_str())
            .collect()
    }
}

// =========================================================================
// Per-language data (mirrors dsl/extdeps/languages/*/types.dag)
// =========================================================================

pub fn rust_checkpoints() -> Vec<TypeCheckpoint> {
    vec![
        TypeCheckpoint { dag_name: "Int".into(),    target_type: "i64".into(),    default_expr: Some("0".into()),              is_copy: Some(true) },
        TypeCheckpoint { dag_name: "Float".into(),  target_type: "f64".into(),    default_expr: Some("0.0".into()),            is_copy: Some(true) },
        TypeCheckpoint { dag_name: "Bool".into(),   target_type: "bool".into(),   default_expr: Some("false".into()),          is_copy: Some(true) },
        TypeCheckpoint { dag_name: "Unit".into(),   target_type: "()".into(),     default_expr: Some("()".into()),             is_copy: Some(true) },
        TypeCheckpoint { dag_name: "String".into(), target_type: "String".into(), default_expr: Some("String::new()".into()),  is_copy: Some(false) },
        TypeCheckpoint { dag_name: "Bytes".into(),  target_type: "Vec<u8>".into(),           default_expr: Some("Vec::new()".into()),           is_copy: Some(false) },
        TypeCheckpoint { dag_name: "Secret".into(), target_type: "String".into(),             default_expr: None,                                is_copy: Some(false) },
        TypeCheckpoint { dag_name: "Json".into(),   target_type: "serde_json::Value".into(),  default_expr: Some("serde_json::Value::Null".into()), is_copy: Some(false) },
    ]
}

pub fn rust_inhabitants() -> Vec<InhabitantDecl> {
    vec![
        InhabitantDecl { algebra: "FreeMonoid".into(),       template: "Vec<{0}>".into(),                             arity: 1, identity_expr: Some("Vec::new()".into()),     import_path: None, is_copy: Some(false) },
        InhabitantDecl { algebra: "BooleanAlgebra".into(),   template: "std::collections::BTreeSet<{0}>".into(),      arity: 1, identity_expr: Some("BTreeSet::new()".into()), import_path: Some("use std::collections::BTreeSet;".into()), is_copy: Some(false) },
        InhabitantDecl { algebra: "PartialFunction".into(),  template: "HashMap<{0}, {1}>".into(),                    arity: 2, identity_expr: Some("HashMap::new()".into()),  import_path: Some("use std::collections::HashMap;".into()), is_copy: Some(false) },
        InhabitantDecl { algebra: "OrderedRing".into(),      template: "i64".into(),                                  arity: 0, identity_expr: Some("0i64".into()),            import_path: None, is_copy: Some(true) },
        InhabitantDecl { algebra: "ApproximateField".into(), template: "f64".into(),                                  arity: 0, identity_expr: Some("0.0f64".into()),          import_path: None, is_copy: Some(true) },
    ]
}

pub fn rust_callable() -> CallableRepr {
    CallableRepr {
        template: "fn({params}) -> {return}".into(),
        param_separator: ", ".into(),
        return_separator: " -> ".into(),
        import_path: None,
    }
}

pub fn python_checkpoints() -> Vec<TypeCheckpoint> {
    vec![
        TypeCheckpoint { dag_name: "Int".into(),    target_type: "int".into(),   default_expr: Some("0".into()),      is_copy: None },
        TypeCheckpoint { dag_name: "Float".into(),  target_type: "float".into(), default_expr: Some("0.0".into()),    is_copy: None },
        TypeCheckpoint { dag_name: "Bool".into(),   target_type: "bool".into(),  default_expr: Some("False".into()),  is_copy: None },
        TypeCheckpoint { dag_name: "Unit".into(),   target_type: "None".into(),  default_expr: Some("None".into()),   is_copy: None },
        TypeCheckpoint { dag_name: "String".into(), target_type: "str".into(),   default_expr: Some("\"\"".into()),   is_copy: None },
        TypeCheckpoint { dag_name: "Bytes".into(),  target_type: "bytes".into(), default_expr: Some("b\"\"".into()),  is_copy: None },
        TypeCheckpoint { dag_name: "Secret".into(), target_type: "str".into(),   default_expr: Some("\"\"".into()),   is_copy: None },
        TypeCheckpoint { dag_name: "Json".into(),   target_type: "dict".into(),  default_expr: Some("{}".into()),     is_copy: None },
    ]
}

pub fn python_inhabitants() -> Vec<InhabitantDecl> {
    vec![
        InhabitantDecl { algebra: "FreeMonoid".into(),       template: "list[{0}]".into(),        arity: 1, identity_expr: Some("[]".into()),    import_path: None, is_copy: None },
        InhabitantDecl { algebra: "BooleanAlgebra".into(),   template: "set[{0}]".into(),         arity: 1, identity_expr: Some("set()".into()), import_path: None, is_copy: None },
        InhabitantDecl { algebra: "PartialFunction".into(),  template: "dict[{0}, {1}]".into(),   arity: 2, identity_expr: Some("{}".into()),    import_path: None, is_copy: None },
        InhabitantDecl { algebra: "OrderedRing".into(),      template: "int".into(),              arity: 0, identity_expr: Some("0".into()),     import_path: None, is_copy: None },
        InhabitantDecl { algebra: "ApproximateField".into(), template: "float".into(),            arity: 0, identity_expr: Some("0.0".into()),   import_path: None, is_copy: None },
    ]
}

pub fn python_callable() -> CallableRepr {
    CallableRepr {
        template: "Callable[[{params}], {return}]".into(),
        param_separator: ", ".into(),
        return_separator: ", ".into(),
        import_path: Some("from typing import Callable".into()),
    }
}

pub fn go_checkpoints() -> Vec<TypeCheckpoint> {
    vec![
        TypeCheckpoint { dag_name: "Int".into(),    target_type: "int64".into(),       default_expr: Some("0".into()),          is_copy: None },
        TypeCheckpoint { dag_name: "Float".into(),  target_type: "float64".into(),     default_expr: Some("0.0".into()),        is_copy: None },
        TypeCheckpoint { dag_name: "Bool".into(),   target_type: "bool".into(),        default_expr: Some("false".into()),      is_copy: None },
        TypeCheckpoint { dag_name: "Unit".into(),   target_type: "struct{}".into(),    default_expr: Some("struct{}{}".into()), is_copy: None },
        TypeCheckpoint { dag_name: "String".into(), target_type: "string".into(),      default_expr: Some("\"\"".into()),       is_copy: None },
        TypeCheckpoint { dag_name: "Bytes".into(),  target_type: "[]byte".into(),      default_expr: Some("nil".into()),        is_copy: None },
        TypeCheckpoint { dag_name: "Secret".into(), target_type: "string".into(),      default_expr: Some("\"\"".into()),       is_copy: None },
        TypeCheckpoint { dag_name: "Json".into(),   target_type: "interface{}".into(), default_expr: Some("nil".into()),        is_copy: None },
    ]
}

pub fn go_inhabitants() -> Vec<InhabitantDecl> {
    vec![
        InhabitantDecl { algebra: "FreeMonoid".into(),       template: "[]{0}".into(),             arity: 1, identity_expr: Some("nil".into()), import_path: None, is_copy: None },
        InhabitantDecl { algebra: "BooleanAlgebra".into(),   template: "map[{0}]struct{}".into(),  arity: 1, identity_expr: Some("nil".into()), import_path: None, is_copy: None },
        InhabitantDecl { algebra: "PartialFunction".into(),  template: "map[{0}]{1}".into(),       arity: 2, identity_expr: Some("nil".into()), import_path: None, is_copy: None },
        InhabitantDecl { algebra: "OrderedRing".into(),      template: "int64".into(),             arity: 0, identity_expr: Some("0".into()),   import_path: None, is_copy: None },
        InhabitantDecl { algebra: "ApproximateField".into(), template: "float64".into(),           arity: 0, identity_expr: Some("0.0".into()), import_path: None, is_copy: None },
    ]
}

pub fn go_callable() -> CallableRepr {
    CallableRepr {
        template: "func({params}) {return}".into(),
        param_separator: ", ".into(),
        return_separator: " ".into(),
        import_path: None,
    }
}

// =========================================================================
// Registry construction
// =========================================================================

use crate::v2_compiler_emit::RenderTarget;

pub fn registry_for_target(target: &RenderTarget) -> CoercionRegistry {
    match target {
        RenderTarget::Rust => CoercionRegistry::new(
            rust_checkpoints(), rust_inhabitants(), rust_callable(),
            "Option<{0}>".into(),
        ),
        RenderTarget::Python => CoercionRegistry::new(
            python_checkpoints(), python_inhabitants(), python_callable(),
            "Optional[{0}]".into(),
        ),
        RenderTarget::Go => CoercionRegistry::new(
            go_checkpoints(), go_inhabitants(), go_callable(),
            "*{0}".into(),
        ),
        RenderTarget::Dag => CoercionRegistry::new(
            vec![], vec![],
            CallableRepr {
                template: "fn({params}) -> {return}".into(),
                param_separator: ", ".into(),
                return_separator: " -> ".into(),
                import_path: None,
            },
            "{0}?".into(),
        ),
    }
}

// =========================================================================
// Coercion-driven lookup functions (replace target_primitive_type etc.)
// =========================================================================

/// Resolve a .dag type name to a target type via checkpoint lookup.
/// Falls through to the original name if no checkpoint exists.
/// This is the transitional form — M5-full will make the fallback fail-closed.
pub fn coerce_primitive_type(registry: &CoercionRegistry, dag_name: &str) -> String {
    match registry.lookup_checkpoint(dag_name) {
        Some(cp) => cp.target_type.clone(),
        None => dag_name.to_string(),
    }
}

/// Apply an inhabitant template with 1 type parameter.
pub fn apply_inhabitant_template1(template: &str, inner: &str) -> String {
    template.replace("{0}", inner)
}

/// Apply an inhabitant template with 2 type parameters.
pub fn apply_inhabitant_template2(template: &str, first: &str, second: &str) -> String {
    template.replace("{0}", first).replace("{1}", second)
}

/// Mapping from .dag container name to algebra name for inhabitant resolution.
/// This bridges the current container-name-based dispatch to algebra-based dispatch.
///
/// The .dag language uses `List`, `Set`, `Map` etc. as surface names.
/// The coercion engine resolves these through their algebra identities.
fn dag_container_to_algebra(name: &str) -> Option<&'static str> {
    match name {
        "List" | "list" | "NonEmptyList" | "non_empty_list" | "FreeMonoid" | "free_monoid" => Some("FreeMonoid"),
        "Set" | "set" | "NonEmptySet" | "non_empty_set" | "BooleanAlgebra" | "boolean_algebra" => Some("BooleanAlgebra"),
        "Map" | "map" | "PartialFunction" | "partial_function" => Some("PartialFunction"),
        _ => None,
    }
}

/// Resolve a container type template via algebra inhabitant lookup.
/// Returns the inhabitant template if the container's algebra has a declared inhabitant.
pub fn coerce_container_template(registry: &CoercionRegistry, container_name: &str) -> Option<String> {
    dag_container_to_algebra(container_name)
        .and_then(|algebra| registry.lookup_inhabitant(algebra))
        .map(|inh| inh.template.clone())
}

// =========================================================================
// Container identity derived from inhabitants (replaces hardcoded lists)
// =========================================================================

/// Known keyed container names derived from inhabitant data.
/// Any .dag type name that maps to an arity-2 algebra is a keyed container.
pub const COERCION_KEYED_CONTAINER_NAMES: &[&str] = &["Map", "PartialFunction"];

/// Known element container names derived from inhabitant data.
/// Any .dag type name that maps to an arity-1 algebra is an element container.
pub const COERCION_ELEMENT_CONTAINER_NAMES: &[&str] = &["List", "Set", "NonEmptyList", "NonEmptySet", "FreeMonoid"];
