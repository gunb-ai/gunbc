//! Type registry for named type DAGs.
//!
//! The registry stores named type DAGs that can be referenced by `TypeId` in ports.
//! This enables type composition, sharing, and lookup during validation.
//!
//! # Example
//!
//! ```ignore
//! use gunbc_ir::{TypeRegistry, type_lib};
//!
//! let mut registry = TypeRegistry::new();
//!
//! // Register common types
//! registry.register("String", type_lib::string());
//! registry.register("Url", type_lib::url());
//! registry.register("OptionalUrl", type_lib::optional_url());
//!
//! // Look up types by name
//! let url_type = registry.get("Url").unwrap();
//! ```

use crate::contract::{self, TypeContract};
use crate::dag::Dag;
use crate::type_lib;
use crate::type_op::{BaseType, Coercion, TypeOp, WrapperKind};
use crate::types::{Cardinality, TypeId};
use std::collections::{HashMap, VecDeque};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
enum TypeExpr {
    Named(String),
    Wrapper(WrapperKind, Box<TypeExpr>),
    Map(Box<TypeExpr>, Box<TypeExpr>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResolveMode {
    Root,
    InWrapper,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeExprError {
    expr: String,
    message: String,
}

impl TypeExprError {
    fn new(expr: &str, message: impl Into<String>) -> Self {
        Self {
            expr: expr.to_string(),
            message: message.into(),
        }
    }
}

impl fmt::Display for TypeExprError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "invalid type expression '{}': {}",
            self.expr, self.message
        )
    }
}

impl std::error::Error for TypeExprError {}

fn parse_wrapper_kind(name: &str) -> Option<WrapperKind> {
    match name {
        "Optional" | "Option" => Some(WrapperKind::Optional),
        "List" => Some(WrapperKind::List),
        "NonEmptyList" => Some(WrapperKind::NonEmptyList),
        "Set" => Some(WrapperKind::Set),
        "NonEmptySet" => Some(WrapperKind::NonEmptySet),
        _ => None,
    }
}

fn split_top_level_generic(expr: &str) -> Result<Option<(&str, &str)>, TypeExprError> {
    let mut depth = 0usize;
    let mut start = None;
    let mut end = None;
    let mut saw_lt = false;

    for (idx, ch) in expr.char_indices() {
        match ch {
            '<' => {
                if depth == 0 {
                    start = Some(idx);
                    saw_lt = true;
                }
                depth += 1;
            }
            '>' => {
                if depth == 0 {
                    return Err(TypeExprError::new(expr, "unexpected '>'"));
                }
                depth -= 1;
                if depth == 0 {
                    end = Some(idx);
                    break;
                }
            }
            _ => {}
        }
    }

    if !saw_lt {
        return Ok(None);
    }

    if depth != 0 {
        return Err(TypeExprError::new(expr, "unbalanced '<'"));
    }

    let (start, end) = match (start, end) {
        (Some(start), Some(end)) => (start, end),
        _ => return Err(TypeExprError::new(expr, "missing generic arguments")),
    };
    if !expr[end + 1..].trim().is_empty() {
        return Err(TypeExprError::new(
            expr,
            "trailing characters after generic",
        ));
    }

    let name = expr[..start].trim();
    let inner = expr[start + 1..end].trim();
    if name.is_empty() || inner.is_empty() {
        return Err(TypeExprError::new(
            expr,
            "generic name or arguments are empty",
        ));
    }

    Ok(Some((name, inner)))
}

fn split_top_level_args(inner: &str) -> Vec<&str> {
    let mut args = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;

    for (idx, ch) in inner.char_indices() {
        match ch {
            '<' => depth += 1,
            '>' => {
                depth = depth.saturating_sub(1);
            }
            ',' if depth == 0 => {
                args.push(inner[start..idx].trim());
                start = idx + 1;
            }
            _ => {}
        }
    }

    args.push(inner[start..].trim());
    args
}

fn render_type_expr(expr: &TypeExpr) -> String {
    match expr {
        TypeExpr::Named(name) => name.clone(),
        TypeExpr::Wrapper(kind, inner) => {
            let wrapper = match kind {
                WrapperKind::Optional => "Optional",
                WrapperKind::List => "List",
                WrapperKind::NonEmptyList => "NonEmptyList",
                WrapperKind::Set => "Set",
                WrapperKind::NonEmptySet => "NonEmptySet",
                WrapperKind::Map => "Map",
            };
            format!("{wrapper}<{}>", render_type_expr(inner))
        }
        TypeExpr::Map(key, value) => {
            format!("Map<{},{}>", render_type_expr(key), render_type_expr(value))
        }
    }
}

fn map_key_is_string(expr: &TypeExpr) -> bool {
    matches!(expr, TypeExpr::Named(name) if name == "String")
}

fn parse_type_expr(raw: &str) -> Result<TypeExpr, TypeExprError> {
    let expr = raw.trim();
    if expr.is_empty() {
        return Err(TypeExprError::new(raw, "empty type expression"));
    }

    if let Some((name, inner)) = split_top_level_generic(expr)? {
        if let Some(kind) = parse_wrapper_kind(name) {
            let args = split_top_level_args(inner);
            if args.len() != 1 || args.iter().any(|arg| arg.is_empty()) {
                return Err(TypeExprError::new(
                    expr,
                    "wrapper expects a single type argument",
                ));
            }
            let inner_expr = parse_type_expr(args[0])?;
            return Ok(TypeExpr::Wrapper(kind, Box::new(inner_expr)));
        }
        if name == "Map" {
            let args = split_top_level_args(inner);
            if args.len() != 2 || args.iter().any(|arg| arg.is_empty()) {
                return Err(TypeExprError::new(
                    expr,
                    "Map expects exactly two type arguments",
                ));
            }
            let key = parse_type_expr(args[0])?;
            let value = parse_type_expr(args[1])?;
            if !map_key_is_string(&key) {
                return Err(TypeExprError::new(expr, "Map key type must be String"));
            }
            return Ok(TypeExpr::Map(Box::new(key), Box::new(value)));
        }
        return Ok(TypeExpr::Named(expr.to_string()));
    }

    if expr.contains('<') || expr.contains('>') {
        return Err(TypeExprError::new(expr, "unbalanced generic brackets"));
    }
    if expr.contains(',') {
        return Err(TypeExprError::new(expr, "unexpected ','"));
    }

    Ok(TypeExpr::Named(expr.to_string()))
}

/// Registry for named type DAGs.
///
/// Types are stored as `Dag<TypeOp>` and can be looked up by `TypeId`.
#[derive(Debug, Clone, Default)]
pub struct TypeRegistry {
    /// Map from type name to type DAG.
    types: HashMap<TypeId, Dag<TypeOp>>,
    /// Explicit coercion edges keyed by source type.
    coercion_edges: HashMap<TypeId, Vec<CoercionEdge>>,
}

#[derive(Debug, Clone)]
struct CoercionEdge {
    to: TypeId,
    transform: TypeOp,
}

/// Suggested explicit transformation strategy for an unsafe coercion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoercionStrategy {
    /// Run the target type's validation/transform DAG explicitly.
    ValidateTo(TypeId),
}

impl fmt::Display for CoercionStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CoercionStrategy::ValidateTo(target) => {
                write!(f, "validate to '{}'", target.0)
            }
        }
    }
}

impl TypeRegistry {
    /// Create a new empty type registry.
    pub fn new() -> Self {
        Self {
            types: HashMap::new(),
            coercion_edges: HashMap::new(),
        }
    }

    /// Create a type registry with common primitive types pre-registered.
    pub fn with_primitives() -> Self {
        let mut registry = Self::new();
        registry.register_primitives();
        registry
    }

    /// Register primitive types (String, Bool, Int, Unit, Json).
    pub fn register_primitives(&mut self) {
        self.register("String", type_lib::string());
        self.register("Bool", type_lib::bool());
        self.register("Int", type_lib::int());
        self.register("Unit", type_lib::unit());
        self.register("Json", type_lib::json());
    }

    /// Register common refined/core types used across the repo.
    ///
    /// These are structural refinements over primitives (e.g., Url is a refined String).
    pub fn register_core_types(&mut self) {
        self.register("NonEmptyString", type_lib::non_empty_string());
        self.register("Url", type_lib::url());
        self.register("FilePath", type_lib::file_path());
        self.register("Path", type_lib::file_path());
        self.register("Email", type_lib::email());
        self.register("PositiveInt", type_lib::positive_int());
        self.register("NonNegativeInt", type_lib::non_negative_int());

        // Container aliases (cardinality encoded in the type DAG).
        self.register("OptionalString", type_lib::optional(type_lib::string()));
        self.register("OptionalInt", type_lib::optional(type_lib::int()));
        self.register("OptionalBool", type_lib::optional(type_lib::bool()));
        self.register("OptionalJson", type_lib::optional(type_lib::json()));
        self.register("StringList", type_lib::list(type_lib::string()));
        self.register(
            "NonEmptyStringList",
            type_lib::non_empty_list(type_lib::string()),
        );
        self.register("IntList", type_lib::list(type_lib::int()));
        self.register("BoolList", type_lib::list(type_lib::bool()));
        self.register("JsonList", type_lib::list(type_lib::json()));

        // Legacy/container aliases (cardinality encoded in the type DAG).
        self.register("OptionalUrl", type_lib::optional_url());
        self.register("UrlList", type_lib::url_list());
        self.register("FilePathList", type_lib::file_path_list());
        self.register("NonEmptyFilePathList", type_lib::non_empty_file_path_list());
    }

    /// Create a type registry with primitives + common refined/core types.
    pub fn with_core_types() -> Self {
        let mut registry = Self::with_primitives();
        registry.register_core_types();
        registry
    }

    /// Register a type DAG with a name.
    pub fn register(&mut self, name: impl Into<TypeId>, type_dag: Dag<TypeOp>) {
        self.types.insert(name.into(), type_dag);
    }

    /// Register an explicit coercion edge between named types.
    ///
    /// This records a `TypeOp::Transform(Coercion)` edge in the registry-level
    /// coercion graph so discovery can find paths that are not implied by base
    /// ancestry alone.
    pub fn register_coercion_edge(&mut self, from: impl Into<TypeId>, to: impl Into<TypeId>) {
        let from = from.into();
        let to = to.into();
        let edge = CoercionEdge {
            to: to.clone(),
            transform: TypeOp::Transform(Coercion::new(
                BaseType::named(from.0.clone()),
                BaseType::named(to.0.clone()),
            )),
        };
        self.coercion_edges.entry(from).or_default().push(edge);
    }

    /// Resolve a type DAG, honoring wrapper expressions like `Optional<T>`.
    ///
    /// Returns `None` if the type is not registered and no wrapper expression is present.
    pub fn resolve_type(&self, type_id: &TypeId) -> Option<Dag<TypeOp>> {
        self.resolve_type_checked(type_id).ok().flatten()
    }

    /// Resolve a type DAG, returning a diagnostic if the expression is invalid.
    ///
    /// Returns `Ok(None)` when the type is syntactically valid but not registered.
    pub fn resolve_type_checked(
        &self,
        type_id: &TypeId,
    ) -> Result<Option<Dag<TypeOp>>, TypeExprError> {
        let expr = parse_type_expr(&type_id.0)?;
        Ok(self.resolve_expr(&expr, ResolveMode::Root))
    }

    /// Validate that a type expression is syntactically well-formed.
    pub fn validate_type_expr(&self, type_id: &TypeId) -> Result<(), TypeExprError> {
        parse_type_expr(&type_id.0).map(|_| ())
    }

    fn resolve_expr(&self, expr: &TypeExpr, mode: ResolveMode) -> Option<Dag<TypeOp>> {
        match expr {
            TypeExpr::Named(name) => {
                if let Some(dag) = self.get_by_name(name) {
                    return Some(dag.clone());
                }
                match mode {
                    ResolveMode::Root => None,
                    ResolveMode::InWrapper => Some(type_lib::identity(name)),
                }
            }
            TypeExpr::Wrapper(kind, inner) => {
                let inner_dag = self.resolve_expr(inner, ResolveMode::InWrapper)?;
                let dag = match kind {
                    WrapperKind::Optional => type_lib::optional(inner_dag),
                    WrapperKind::List => type_lib::list(inner_dag),
                    WrapperKind::NonEmptyList => type_lib::non_empty_list(inner_dag),
                    WrapperKind::Set => type_lib::set(inner_dag),
                    WrapperKind::NonEmptySet => type_lib::non_empty_set(inner_dag),
                    WrapperKind::Map => type_lib::map(inner_dag),
                };
                Some(dag)
            }
            TypeExpr::Map(key, value) => {
                let _ = self.resolve_expr(key, ResolveMode::InWrapper)?;
                let value_dag = self.resolve_expr(value, ResolveMode::InWrapper)?;
                let name = render_type_expr(expr);
                if let Some(dag) = self.get_by_name(&name) {
                    return Some(dag.clone());
                }
                Some(type_lib::map(value_dag))
            }
        }
    }

    /// Get a type DAG by name.
    pub fn get(&self, name: &TypeId) -> Option<&Dag<TypeOp>> {
        self.types.get(name)
    }

    /// Get a type DAG by string name.
    pub fn get_by_name(&self, name: &str) -> Option<&Dag<TypeOp>> {
        self.types.get(&TypeId::from(name))
    }

    /// Check if a type is registered.
    pub fn contains(&self, name: &TypeId) -> bool {
        self.types.contains_key(name)
    }

    /// Get all registered type names.
    pub fn type_names(&self) -> impl Iterator<Item = &TypeId> {
        self.types.keys()
    }

    /// Get the number of registered types.
    pub fn len(&self) -> usize {
        self.types.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
    }

    /// Infer cardinality from a registered type or wrapper expression.
    ///
    /// Returns `None` if the type is not registered or does not encode cardinality.
    pub fn infer_cardinality(&self, type_id: &TypeId) -> Option<Cardinality> {
        let dag = self.resolve_type(type_id)?;
        contract::wrapper_kind(&dag)?;
        Some(type_lib::infer_cardinality(&dag))
    }

    /// Get the base type name from a registered type or wrapper expression.
    ///
    /// Returns `None` if the type is not registered.
    pub fn base_type_name(&self, type_id: &TypeId) -> Option<String> {
        self.resolve_type(type_id)
            .and_then(|dag| TypeContract::from_type_dag(&dag).base_type)
    }

    /// Check if type A is compatible with type B.
    ///
    /// Compatibility is determined by:
    /// 1. Same type name (exact match)
    /// 2. Target is "Any" (accepts anything)
    /// 3. Structural refinement: A's contract entails B's contract
    pub fn is_compatible(&self, from: &TypeId, to: &TypeId) -> bool {
        // Same type is always compatible.
        if from == to {
            return true;
        }

        // Target Any accepts anything.
        if to.0 == "Any" {
            return true;
        }

        // Source Any does not entail specific targets.
        if from.0 == "Any" {
            return false;
        }

        // Look up both types; if not registered, fall back to name equality (handled above).
        let (Some(from_dag), Some(to_dag)) = (self.resolve_type(from), self.resolve_type(to))
        else {
            return false;
        };

        let from_contract = TypeContract::from_type_dag(&from_dag);
        let to_contract = TypeContract::from_type_dag(&to_dag);

        from_contract
            .can_safely_coerce_to_with(&to_contract, |from, to| self.base_type_upcasts_to(from, to))
            .is_ok()
    }

    /// Check whether `from` is a structural refinement of `to`.
    ///
    /// A refinement can safely coerce to its base type (widening).
    pub fn is_refinement_of(&self, from: &TypeId, to: &TypeId) -> bool {
        self.is_compatible(from, to)
    }

    /// Suggest an explicit coercion strategy when a safe upcast is not possible.
    pub fn coercion_strategy(&self, from: &TypeId, to: &TypeId) -> Option<CoercionStrategy> {
        if from == to {
            return None;
        }

        if self.is_compatible(from, to) {
            return None;
        }

        if self.is_refinement_of(to, from) {
            return Some(CoercionStrategy::ValidateTo(to.clone()));
        }

        None
    }

    /// Discover a widening coercion path between two named types.
    ///
    /// Returns the shortest known upcast chain as type IDs, including source
    /// and target, when `from` can safely widen into `to`.
    pub fn coercion_path(&self, from: &TypeId, to: &TypeId) -> Option<Vec<TypeId>> {
        let mut queue: VecDeque<Vec<TypeId>> = VecDeque::new();
        let mut visited = std::collections::HashSet::new();
        queue.push_back(vec![from.clone()]);

        while let Some(path) = queue.pop_front() {
            let current = path.last().cloned()?;
            if current == *to {
                return Some(path);
            }
            if !visited.insert(current.clone()) {
                continue;
            }

            for next in self.coercion_neighbors(&current) {
                if path.iter().any(|step| step == &next) {
                    continue;
                }
                let mut next_path = path.clone();
                next_path.push(next);
                queue.push_back(next_path);
            }
        }

        None
    }

    fn coercion_neighbors(&self, current: &TypeId) -> Vec<TypeId> {
        let mut neighbors = Vec::new();

        // Explicit registry edges via TypeOp::Transform(Coercion).
        if let Some(edges) = self.coercion_edges.get(current) {
            neighbors.extend(edges.iter().filter_map(|edge| match &edge.transform {
                TypeOp::Transform(_) => Some(edge.to.clone()),
                _ => None,
            }));
        }

        // Json is the widening top type.
        if current.0 != "Json" {
            neighbors.push(TypeId::from("Json"));
        }

        // Structural ancestry from base type DAG.
        if let Some(dag) = self.get_by_name(&current.0) {
            if let Some(parent) = crate::contract::base_type(dag) {
                if parent != current.0 {
                    neighbors.push(TypeId(parent));
                }
            }
        }

        neighbors
    }

    pub(crate) fn base_type_upcasts_to(&self, from: &str, to: &str) -> bool {
        self.coercion_path(&TypeId::from(from), &TypeId::from(to))
            .is_some()
    }
}

/// Error when type lookup fails.
#[derive(Debug, Clone)]
pub struct TypeNotFoundError {
    pub type_id: TypeId,
}

impl std::fmt::Display for TypeNotFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "type not found: {}", self.type_id)
    }
}

impl std::error::Error for TypeNotFoundError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_registry() {
        let registry = TypeRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_register_and_get() {
        let mut registry = TypeRegistry::new();
        registry.register("String", type_lib::string());

        assert!(registry.contains(&TypeId::from("String")));
        assert!(registry.get(&TypeId::from("String")).is_some());
        assert!(registry.get_by_name("String").is_some());
        assert!(registry.get_by_name("Unknown").is_none());
    }

    #[test]
    fn test_with_primitives() {
        let registry = TypeRegistry::with_primitives();

        assert!(registry.contains(&TypeId::from("String")));
        assert!(registry.contains(&TypeId::from("Bool")));
        assert!(registry.contains(&TypeId::from("Int")));
        assert!(registry.contains(&TypeId::from("Unit")));
        assert!(registry.contains(&TypeId::from("Json")));
        assert_eq!(registry.len(), 5);
    }

    #[test]
    fn test_cardinality_inference() {
        let mut registry = TypeRegistry::with_primitives();
        registry.register("MaybeValue", type_lib::optional(type_lib::string()));
        registry.register("ValueCollection", type_lib::list(type_lib::string()));

        assert_eq!(registry.infer_cardinality(&TypeId::from("String")), None);
        assert_eq!(
            registry.infer_cardinality(&TypeId::from("MaybeValue")),
            Some(Cardinality::ZERO_OR_ONE)
        );
        assert_eq!(
            registry.infer_cardinality(&TypeId::from("ValueCollection")),
            Some(Cardinality::ZERO_OR_MORE)
        );
    }

    #[test]
    fn test_type_compatibility() {
        let mut registry = TypeRegistry::with_primitives();
        registry.register("Url", type_lib::url());
        registry.register("NonEmptyString", type_lib::non_empty_string());
        registry.register(
            "CustomUrl",
            type_lib::refined("Url", vec![crate::type_op::Predicate::NonEmpty]),
        );

        // Same type is compatible
        assert!(registry.is_compatible(&TypeId::from("String"), &TypeId::from("String")));

        // Refined types are compatible with their base (Url -> String).
        assert!(registry.is_compatible(&TypeId::from("Url"), &TypeId::from("String")));

        // Base types are NOT compatible with refined types (String -> Url).
        assert!(!registry.is_compatible(&TypeId::from("String"), &TypeId::from("Url")));

        // NonEmptyString is a refinement of String.
        assert!(registry.is_compatible(&TypeId::from("NonEmptyString"), &TypeId::from("String")));
        assert!(!registry.is_compatible(&TypeId::from("String"), &TypeId::from("NonEmptyString")));

        // Primitive upcasts to Json are safe.
        assert!(registry.is_compatible(&TypeId::from("Int"), &TypeId::from("Json")));
        assert!(registry.is_compatible(&TypeId::from("Bool"), &TypeId::from("Json")));
        assert!(registry.is_compatible(&TypeId::from("String"), &TypeId::from("Json")));
        assert!(!registry.is_compatible(&TypeId::from("Json"), &TypeId::from("Int")));

        // Registry-driven refinement: CustomUrl (refines Url) upcasts to String via Url.
        assert!(registry.is_compatible(&TypeId::from("CustomUrl"), &TypeId::from("String")));
        assert!(!registry.is_compatible(&TypeId::from("String"), &TypeId::from("CustomUrl")));
    }

    #[test]
    fn test_coercion_strategy_for_refinement() {
        let mut registry = TypeRegistry::with_primitives();
        registry.register("Url", type_lib::url());

        let strategy = registry.coercion_strategy(&TypeId::from("String"), &TypeId::from("Url"));
        assert_eq!(
            strategy,
            Some(CoercionStrategy::ValidateTo(TypeId::from("Url")))
        );

        // Safe upcast returns no strategy.
        let strategy = registry.coercion_strategy(&TypeId::from("Url"), &TypeId::from("String"));
        assert!(strategy.is_none());
    }

    #[test]
    fn test_coercion_path_finds_refinement_upcast_chain() {
        let mut registry = TypeRegistry::with_primitives();
        registry.register("Url", type_lib::url());
        let path = registry
            .coercion_path(&TypeId::from("Url"), &TypeId::from("String"))
            .expect("Url should widen to String");
        assert_eq!(path, vec![TypeId::from("Url"), TypeId::from("String")]);
    }

    #[test]
    fn test_coercion_path_finds_json_top_widening() {
        let registry = TypeRegistry::with_primitives();
        let int_path = registry
            .coercion_path(&TypeId::from("Int"), &TypeId::from("Json"))
            .expect("Int should widen to Json");
        assert_eq!(int_path, vec![TypeId::from("Int"), TypeId::from("Json")]);

        let string_path = registry
            .coercion_path(&TypeId::from("String"), &TypeId::from("Json"))
            .expect("String should widen to Json");
        assert_eq!(
            string_path,
            vec![TypeId::from("String"), TypeId::from("Json")]
        );
    }

    #[test]
    fn test_coercion_path_rejects_narrowing_chain() {
        let mut registry = TypeRegistry::with_primitives();
        registry.register("Url", type_lib::url());
        assert!(
            registry
                .coercion_path(&TypeId::from("String"), &TypeId::from("Url"))
                .is_none(),
            "String -> Url is narrowing and must not be discovered as safe path"
        );
    }

    #[test]
    fn test_explicit_coercion_edge_registers_transform_and_is_discoverable() {
        let mut registry = TypeRegistry::with_primitives();
        registry.register_coercion_edge("String", "Url");

        let path = registry
            .coercion_path(&TypeId::from("String"), &TypeId::from("Url"))
            .expect("explicit String->Url transform edge should be discoverable");
        assert_eq!(path, vec![TypeId::from("String"), TypeId::from("Url")]);

        let edges = registry
            .coercion_edges
            .get(&TypeId::from("String"))
            .expect("coercion edge should be stored");
        assert!(edges.iter().any(|edge| {
            edge.to == TypeId::from("Url") && matches!(edge.transform, TypeOp::Transform(_))
        }));
    }

    #[test]
    fn test_base_type_name() {
        let mut registry = TypeRegistry::with_primitives();
        registry.register("Url", type_lib::url());

        assert_eq!(
            registry.base_type_name(&TypeId::from("String")),
            Some("String".to_string())
        );
        assert_eq!(
            registry.base_type_name(&TypeId::from("Url")),
            Some("String".to_string())
        );
        assert_eq!(registry.base_type_name(&TypeId::from("Unknown")), None);
    }

    #[test]
    fn test_type_expression_resolution() {
        let registry = TypeRegistry::with_primitives();

        let optional_string = TypeId::from("Optional<String>");
        assert_eq!(
            registry.infer_cardinality(&optional_string),
            Some(Cardinality::ZERO_OR_ONE)
        );
        assert_eq!(
            registry.base_type_name(&optional_string),
            Some("String".to_string())
        );

        let optional_transport = TypeId::from("Optional<TransportResponse>");
        assert_eq!(
            registry.infer_cardinality(&optional_transport),
            Some(Cardinality::ZERO_OR_ONE)
        );
        assert_eq!(
            registry.base_type_name(&optional_transport),
            Some("TransportResponse".to_string())
        );

        let map_type = TypeId::from("Map<String,Int>");
        assert_eq!(registry.base_type_name(&map_type), Some("Int".to_string()));
        assert_eq!(
            registry.infer_cardinality(&map_type),
            Some(Cardinality::ONE)
        );

        let nested = TypeId::from("Optional<Map<String, Optional<Int>>>");
        assert_eq!(registry.base_type_name(&nested), Some("Int".to_string()));
        assert_eq!(
            registry.infer_cardinality(&nested),
            Some(Cardinality::ZERO_OR_ONE)
        );
    }

    #[test]
    fn test_invalid_type_expression_diagnostics() {
        let registry = TypeRegistry::with_primitives();

        assert!(registry
            .validate_type_expr(&TypeId::from("Optional<String"))
            .is_err());
        assert!(registry
            .validate_type_expr(&TypeId::from("Map<String>"))
            .is_err());
        assert!(registry
            .validate_type_expr(&TypeId::from("List<String,Int>"))
            .is_err());
        assert!(registry
            .validate_type_expr(&TypeId::from("Map<,Int>"))
            .is_err());
        assert!(registry
            .validate_type_expr(&TypeId::from("Map<Int,String>"))
            .is_err());
    }
}
