//! Type registry for named type DAGs.
//!
//! The registry stores named type DAGs that can be referenced by `TypeId` in ports.
//! This enables type composition, sharing, and lookup during validation.
//!
//! # Example
//!
//! ```text
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
use crate::type_op::{BaseType, Coercion, Predicate, TypeOp, WrapperKind};
use crate::types::{Cardinality, TypeId};
use serde::{Deserialize, Serialize};
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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TypeRegistry {
    /// Map from type name to type DAG.
    types: HashMap<TypeId, Dag<TypeOp>>,
    /// Explicit coercion edges keyed by source type.
    coercion_edges: HashMap<TypeId, Vec<CoercionEdge>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

    /// Register primitive types (String, Bool, Int, Float, Bytes, Unit, Json, Secret).
    pub fn register_primitives(&mut self) {
        self.register("String", type_lib::string());
        self.register("Bool", type_lib::bool());
        self.register("Int", type_lib::int());
        self.register("Float", type_lib::float());
        self.register("Bytes", type_lib::bytes());
        self.register("Unit", type_lib::unit());
        self.register("Json", type_lib::json());
        self.register("Secret", type_lib::secret());
    }

    /// Register common refined/core types used across the repo.
    ///
    /// These are structural refinements over primitives (e.g., Url is a refined String).
    pub fn register_core_types(&mut self) {
        self.register("NonEmptyString", type_lib::non_empty_string());
        self.register("NonEmptyStr", type_lib::non_empty_string());
        self.register("SecretName", type_lib::non_empty_string());
        self.register("LanguageId", type_lib::non_empty_string());
        self.register("Url", type_lib::url());
        self.register("FilePath", type_lib::file_path());
        self.register("Path", type_lib::file_path());
        self.register("Email", type_lib::email());
        self.register("PositiveInt", type_lib::positive_int());
        self.register("NonNegativeInt", type_lib::non_negative_int());
        self.register("GitRef", type_lib::non_empty_string());
        self.register(
            "ProjectId",
            type_lib::refined(
                "String",
                vec![
                    Predicate::NonEmpty,
                    Predicate::Matches("^[a-z][a-z0-9-]{4,28}[a-z0-9]$".to_string()),
                ],
            ),
        );
        self.register(
            "ServiceAccountEmail",
            type_lib::refined(
                "String",
                vec![Predicate::Matches(
                    "^[a-z][a-z0-9-]*@[a-z0-9-]+\\.iam\\.gserviceaccount\\.com$".to_string(),
                )],
            ),
        );

        // Content-encoded file path types (set-theoretic type system).
        self.register("TextFilePath", type_lib::text_file_path());
        self.register("BinaryFilePath", type_lib::binary_file_path());

        // ContentEncoding variants as a coproduct type.
        self.register(
            "ContentEncoding",
            type_lib::coproduct(
                "ContentEncoding",
                vec![
                    ("Unknown", "String"),
                    ("Text", "String"),
                    ("UTF8", "String"),
                    ("ASCII", "String"),
                    ("Latin1", "String"),
                    ("Binary", "Bytes"),
                ],
            ),
        );
        self.register(
            "WarningPolicy",
            type_lib::coproduct(
                "WarningPolicy",
                vec![("DenyAll", "String"), ("Default", "String")],
            ),
        );
        self.register(
            "CloudRuntime",
            type_lib::coproduct(
                "CloudRuntime",
                vec![
                    ("GitHubActions", "String"),
                    ("Metadata", "String"),
                    ("LocalDev", "String"),
                ],
            ),
        );
        self.register(
            "AuthScheme",
            type_lib::coproduct(
                "AuthScheme",
                vec![
                    ("Bearer", "String"),
                    ("Header", "String"),
                    ("Basic", "String"),
                ],
            ),
        );
        self.register(
            "FermiDepth",
            type_lib::coproduct(
                "FermiDepth",
                vec![
                    ("Xs", "String"),
                    ("S", "String"),
                    ("M", "String"),
                    ("L", "String"),
                    ("Xl", "String"),
                ],
            ),
        );
        self.register(
            "TransportClass",
            type_lib::coproduct(
                "TransportClass",
                vec![
                    ("LocalDirect", "String"),
                    ("ShellLocal", "String"),
                    ("FileBoundary", "String"),
                    ("RestNetwork", "String"),
                    ("InterfaceStub", "String"),
                    ("Unknown", "String"),
                ],
            ),
        );
        self.register(
            "TestClass",
            type_lib::coproduct(
                "TestClass",
                vec![
                    ("Unit", "String"),
                    ("Hermetic", "String"),
                    ("Integration", "String"),
                ],
            ),
        );
        self.register(
            "DisplayWidth",
            type_lib::coproduct(
                "DisplayWidth",
                vec![
                    ("ZeroWidth", "String"),
                    ("Narrow", "String"),
                    ("Wide", "String"),
                ],
            ),
        );
        self.register(
            "SemanticColor",
            type_lib::coproduct(
                "SemanticColor",
                vec![
                    ("Default", "String"),
                    ("Success", "String"),
                    ("Warning", "String"),
                    ("Error", "String"),
                    ("Info", "String"),
                    ("Dim", "String"),
                    ("Active", "String"),
                    ("Accent", "String"),
                ],
            ),
        );
        self.register(
            "Tier",
            type_lib::coproduct(
                "Tier",
                vec![
                    ("Emoji", "String"),
                    ("Unicode", "String"),
                    ("Ascii", "String"),
                ],
            ),
        );
        self.register(
            "SymbolId",
            type_lib::coproduct(
                "SymbolId",
                vec![
                    ("NodePending", "String"),
                    ("NodeRunning", "String"),
                    ("NodeCompleted", "String"),
                    ("NodeFailed", "String"),
                    ("NodeSkipped", "String"),
                    ("NodeIntercepted", "String"),
                    ("EdgeIdle", "String"),
                    ("EdgeFlowing", "String"),
                    ("EdgeDone", "String"),
                    ("EdgeDead", "String"),
                    ("DagNotStarted", "String"),
                    ("DagRunning", "String"),
                    ("DagCompleted", "String"),
                    ("DagFailed", "String"),
                    ("BoundaryMarker", "String"),
                    ("Spinner0", "String"),
                    ("Spinner1", "String"),
                    ("Spinner2", "String"),
                    ("Spinner3", "String"),
                    ("Spinner4", "String"),
                    ("Spinner5", "String"),
                    ("Spinner6", "String"),
                    ("Spinner7", "String"),
                    ("Spinner8", "String"),
                    ("Spinner9", "String"),
                    ("Success", "String"),
                    ("Failure", "String"),
                    ("Warning", "String"),
                    ("Info", "String"),
                    ("DataList", "String"),
                    ("DataMap", "String"),
                    ("DataSecret", "String"),
                    ("DataUrl", "String"),
                    ("DataTimer", "String"),
                    ("ConnectorHorizontal", "String"),
                    ("ConnectorVertical", "String"),
                    ("ConnectorTeeDown", "String"),
                    ("ConnectorTeeUp", "String"),
                    ("ConnectorCornerBottomLeft", "String"),
                    ("ConnectorCornerTopLeft", "String"),
                ],
            ),
        );

        // Domain types for transport/infrastructure.
        self.register("TransportRequest", type_lib::identity("TransportRequest"));
        self.register("TransportResponse", type_lib::identity("TransportResponse"));
        self.register("Credential", type_lib::identity("Credential"));
        self.register("FilesystemHandle", type_lib::identity("FilesystemHandle"));
        self.register("NetworkHandle", type_lib::identity("NetworkHandle"));
        self.register(
            "CliResult",
            type_lib::product(
                "CliResult",
                vec![
                    ("stdout", "String"),
                    ("stderr", "String"),
                    ("exit_code", "Int"),
                ],
            ),
        );
        self.register("ToolHandle", type_lib::identity("ToolHandle"));
        self.register("Platform", type_lib::identity("Platform"));
        self.register("Timestamp", type_lib::identity("Timestamp"));
        self.register("Record", type_lib::identity("Record"));

        // Transport response subtypes (json-backed).
        self.register("FileResponse", type_lib::identity("FileResponse"));
        self.register("ShellResponse", type_lib::identity("ShellResponse"));
        self.register("RestResponse", type_lib::identity("RestResponse"));
        self.register("HttpResponse", type_lib::identity("HttpResponse"));

        // GCP/OIDC identity types (string-backed refinements).
        self.register("OidcAudience", type_lib::non_empty_string());
        self.register("WifAudience", type_lib::non_empty_string());
        self.register("GcpProjectId", type_lib::non_empty_string());
        self.register("GcpSecretId", type_lib::non_empty_string());
        self.register("GcpSecretVersion", type_lib::non_empty_string());
        self.register("GcpServiceAccountEmail", type_lib::non_empty_string());
        self.register("GcpSubjectToken", type_lib::non_empty_string());
        self.register("OidcSubjectToken", type_lib::non_empty_string());

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

        // Coercion edges: TextFilePath → FilePath → String
        self.register_coercion_edge("TextFilePath", "FilePath");
        self.register_coercion_edge("BinaryFilePath", "FilePath");
        self.register_coercion_edge("FilePath", "String");
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

    /// Merge another registry into this one without overwriting existing types.
    ///
    /// Core types take precedence: only types not already present are inserted.
    pub fn merge(&mut self, other: &TypeRegistry) {
        for (k, v) in &other.types {
            self.types.entry(k.clone()).or_insert_with(|| v.clone());
        }
    }

    /// Resolve a type DAG, honoring wrapper expressions like `Optional<T>`.
    ///
    /// Returns `None` if the type is not registered and no wrapper expression is present.
    pub fn resolve_type(&self, type_id: &TypeId) -> Option<Dag<TypeOp>> {
        self.resolve_type_checked(type_id).unwrap_or_default()
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
    /// 4. Coercion path: there exists a widening path from A to B in the registry
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
            // If types aren't registered, check if there's a coercion path.
            return self.coercion_path(from, to).is_some();
        };

        let from_contract = TypeContract::from_type_dag(&from_dag);
        let to_contract = TypeContract::from_type_dag(&to_dag);

        if from_contract
            .can_safely_coerce_to_with(&to_contract, |from, to| self.base_type_upcasts_to(from, to))
            .is_ok()
        {
            return true;
        }

        // Fall back to coercion path check — branded/nominal types may have
        // implicit predicate inheritance that the contract comparison misses.
        self.coercion_path(from, to).is_some()
    }

    /// Check structural + strict semantic-carrier compatibility.
    ///
    /// This is stricter than [`Self::is_compatible`]:
    /// - structural compatibility must hold
    /// - semantic carrier kinds must be compatible (no semantic→structural fallback)
    /// - unknown semantic carriers are rejected (fail-closed)
    pub fn is_compatible_strict_semantic(&self, from: &TypeId, to: &TypeId) -> bool {
        self.is_compatible(from, to) && crate::types::semantic_carrier_compatible(from, to)
    }

    /// Check whether `from` is a structural refinement of `to`.
    ///
    /// A refinement can safely coerce to its base type (widening).
    fn is_refinement_of(&self, from: &TypeId, to: &TypeId) -> bool {
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
        let mut has_structural_parent = false;

        // Explicit registry edges via TypeOp::Transform(Coercion).
        if let Some(edges) = self.coercion_edges.get(current) {
            neighbors.extend(edges.iter().filter_map(|edge| match &edge.transform {
                TypeOp::Transform(_) => Some(edge.to.clone()),
                _ => None,
            }));
        }

        // Structural ancestry from type DAGs / generic expressions.
        if let Some(parent) = self.expression_parent_type_id(current) {
            has_structural_parent = true;
            neighbors.push(parent);
        }

        // Json is the widening top type once no stronger ancestry edge remains.
        if current.0 != "Json" && !has_structural_parent {
            neighbors.push(TypeId::from("Json"));
        }

        neighbors
    }

    fn expression_parent_type_id(&self, type_id: &TypeId) -> Option<TypeId> {
        let expr = parse_type_expr(&type_id.0).ok()?;
        let parent_expr = self.expression_parent_expr(&expr)?;
        Some(TypeId(render_type_expr(&parent_expr)))
    }

    fn expression_parent_expr(&self, expr: &TypeExpr) -> Option<TypeExpr> {
        match expr {
            TypeExpr::Named(name) => self.named_parent(name).map(TypeExpr::Named),
            TypeExpr::Wrapper(kind, inner) => self
                .expression_parent_expr(inner)
                .map(|parent| TypeExpr::Wrapper(kind.clone(), Box::new(parent))),
            TypeExpr::Map(key, value) => self
                .expression_parent_expr(value)
                .map(|parent| TypeExpr::Map(key.clone(), Box::new(parent))),
        }
    }

    fn named_parent(&self, name: &str) -> Option<String> {
        let dag = self.get_by_name(name)?;
        let parent = crate::contract::base_type(dag)?;
        if parent == name {
            return None;
        }
        self.get_by_name(&parent)?;
        Some(parent)
    }

    pub(crate) fn base_type_upcasts_to(&self, from: &str, to: &str) -> bool {
        self.coercion_path(&TypeId::from(from), &TypeId::from(to))
            .is_some()
    }

    /// Determine the runtime `ValueBacking` for a type using registry knowledge.
    ///
    /// This replaces the free function `value_backing_for_type_id()` by using
    /// the registry's type DAGs and coercion paths instead of the hardcoded
    /// `PortType` enum.
    pub fn value_backing(&self, type_id: &TypeId) -> crate::types::ValueBacking {
        use crate::types::{
            optional_inner_type_id, parse_map_type_id, parse_unary_generic_type_id, ValueBacking,
        };

        let raw = &type_id.0;

        // Credential is a structured map payload at runtime.
        if raw == "Credential" {
            return ValueBacking::Map;
        }

        // Parametric containers.
        if parse_map_type_id(raw).is_some() {
            return ValueBacking::Map;
        }
        if parse_unary_generic_type_id(raw, "Set").is_some() {
            return ValueBacking::Set;
        }
        if parse_unary_generic_type_id(raw, "List").is_some() {
            return ValueBacking::List;
        }
        if let Some(inner) = optional_inner_type_id(raw) {
            return self.value_backing(&TypeId::from(inner));
        }

        // Primitives (direct match).
        match raw.as_str() {
            "String" => return ValueBacking::String,
            "Bool" => return ValueBacking::Bool,
            "Int" => return ValueBacking::Int,
            "Float" => return ValueBacking::Float,
            "Bytes" => return ValueBacking::Bytes,
            "Json" => return ValueBacking::Json,
            "Unit" => return ValueBacking::Unit,
            "Secret" => return ValueBacking::Secret,
            _ => {}
        }

        // Registry-driven: find the nearest primitive ancestor via coercion path.
        static PRIMITIVE_BACKINGS: &[(&str, ValueBacking)] = &[
            ("String", ValueBacking::String),
            ("Int", ValueBacking::Int),
            ("Float", ValueBacking::Float),
            ("Bool", ValueBacking::Bool),
            ("Bytes", ValueBacking::Bytes),
            ("Secret", ValueBacking::Secret),
        ];
        for &(prim, backing) in PRIMITIVE_BACKINGS {
            if self.coercion_path(type_id, &TypeId::from(prim)).is_some() {
                return backing;
            }
        }

        // Identity types (no coercion path to primitives): use semantic carrier
        // classification to determine backing.
        use crate::types::{semantic_carrier_kind_for_type_id, SemanticCarrierKind};
        match semantic_carrier_kind_for_type_id(raw) {
            SemanticCarrierKind::Structural => {} // fall through to suffix check
            SemanticCarrierKind::Platform => return ValueBacking::String,
            SemanticCarrierKind::Timestamp => return ValueBacking::Int,
            SemanticCarrierKind::TransportRequest
            | SemanticCarrierKind::TransportResponse
            | SemanticCarrierKind::FilesystemHandle
            | SemanticCarrierKind::NetworkHandle
            | SemanticCarrierKind::ToolHandle => return ValueBacking::Json,
            SemanticCarrierKind::Credential => return ValueBacking::Map,
            SemanticCarrierKind::Secret => return ValueBacking::Secret,
            SemanticCarrierKind::UnknownSemantic => return ValueBacking::Json,
        }

        // Legacy suffix-based aliases.
        if raw.ends_with("List") {
            return ValueBacking::List;
        }
        if raw.ends_with("Set") {
            return ValueBacking::Set;
        }

        // Fallback: Json accepts anything.
        eprintln!("warning: unknown type '{}' defaulting to ValueBacking::Json", type_id.0);
        ValueBacking::Json
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
        assert!(registry.contains(&TypeId::from("Float")));
        assert!(registry.contains(&TypeId::from("Bytes")));
        assert!(registry.contains(&TypeId::from("Unit")));
        assert!(registry.contains(&TypeId::from("Json")));
        assert!(registry.contains(&TypeId::from("Secret")));
        assert_eq!(registry.len(), 8);
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
    fn test_type_compatibility_strict_semantic_rejects_semantic_to_any() {
        let registry = TypeRegistry::with_core_types();
        // Structural compatibility allows Any as target.
        assert!(registry.is_compatible(&TypeId::from("Credential"), &TypeId::from("Any")));
        // Strict semantic mode rejects semantic -> structural fallback.
        assert!(!registry
            .is_compatible_strict_semantic(&TypeId::from("Credential"), &TypeId::from("Any")));
        // Same semantic type remains allowed.
        assert!(registry.is_compatible_strict_semantic(
            &TypeId::from("TransportResponse"),
            &TypeId::from("TransportResponse"),
        ));
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
    fn test_coercion_path_supports_multi_step_widening_chain() {
        let mut registry = TypeRegistry::with_primitives();
        registry.register("Url", type_lib::url());
        registry.register(
            "NonEmptyUrl",
            type_lib::refined("Url", vec![crate::type_op::Predicate::NonEmpty]),
        );

        let path = registry
            .coercion_path(&TypeId::from("NonEmptyUrl"), &TypeId::from("Json"))
            .expect("multi-step widening path should be discoverable");
        assert_eq!(
            path,
            vec![
                TypeId::from("NonEmptyUrl"),
                TypeId::from("Url"),
                TypeId::from("String"),
                TypeId::from("Json")
            ]
        );
    }

    #[test]
    fn test_coercion_path_supports_list_covariance() {
        let mut registry = TypeRegistry::with_primitives();
        registry.register("Url", type_lib::url());

        let path = registry
            .coercion_path(&TypeId::from("List<Url>"), &TypeId::from("List<String>"))
            .expect("List<Url> should widen to List<String>");
        assert_eq!(
            path,
            vec![TypeId::from("List<Url>"), TypeId::from("List<String>")]
        );
    }

    #[test]
    fn test_coercion_path_supports_map_value_covariance() {
        let mut registry = TypeRegistry::with_primitives();
        registry.register("Url", type_lib::url());

        let path = registry
            .coercion_path(
                &TypeId::from("Map<String,Url>"),
                &TypeId::from("Map<String,String>"),
            )
            .expect("Map<String, Url> should widen to Map<String, String>");
        assert_eq!(
            path,
            vec![
                TypeId::from("Map<String,Url>"),
                TypeId::from("Map<String,String>")
            ]
        );
    }

    #[test]
    fn test_coercion_path_optional_unwrap_requires_explicit_transform() {
        let mut registry = TypeRegistry::with_primitives();
        let optional_string = TypeId::from("Optional<String>");
        let string = TypeId::from("String");

        assert!(
            registry.coercion_path(&optional_string, &string).is_none(),
            "Optional<String> -> String is narrowing and must require explicit transform"
        );

        registry.register_coercion_edge("Optional<String>", "String");
        let path = registry
            .coercion_path(&optional_string, &string)
            .expect("explicit optional unwrap transform should be discoverable");
        assert_eq!(path, vec![optional_string, string]);
    }

    #[test]
    fn test_coercion_path_cross_provider_secret_payloads_widen_to_string_only() {
        let mut registry = TypeRegistry::with_primitives();
        registry.register(
            "GcpSecretPayload",
            type_lib::refined(
                "String",
                vec![crate::type_op::Predicate::Matches("^gcp:.*$".to_string())],
            ),
        );
        registry.register(
            "AwsSecretValue",
            type_lib::refined(
                "String",
                vec![crate::type_op::Predicate::Matches("^aws:.*$".to_string())],
            ),
        );

        assert_eq!(
            registry.coercion_path(&TypeId::from("GcpSecretPayload"), &TypeId::from("String")),
            Some(vec![
                TypeId::from("GcpSecretPayload"),
                TypeId::from("String")
            ])
        );
        assert_eq!(
            registry.coercion_path(&TypeId::from("AwsSecretValue"), &TypeId::from("String")),
            Some(vec![TypeId::from("AwsSecretValue"), TypeId::from("String")])
        );
        assert!(
            registry
                .coercion_path(
                    &TypeId::from("GcpSecretPayload"),
                    &TypeId::from("AwsSecretValue")
                )
                .is_none(),
            "provider payload types should not coerce directly to each other"
        );
    }

    #[test]
    fn test_coercion_dag_walk_cross_provider_secret_payloads_are_isolated() {
        let mut registry = TypeRegistry::with_primitives();
        registry.register(
            "GcpSecretPayload",
            type_lib::refined(
                "String",
                vec![crate::type_op::Predicate::Matches("^gcp:.*$".to_string())],
            ),
        );
        registry.register(
            "AwsSecretValue",
            type_lib::refined(
                "String",
                vec![crate::type_op::Predicate::Matches("^aws:.*$".to_string())],
            ),
        );

        // DAG-walk widening paths should each terminate at String.
        let gcp_walk = registry
            .coercion_path(&TypeId::from("GcpSecretPayload"), &TypeId::from("String"))
            .expect("gcp payload should widen to String");
        assert_eq!(
            gcp_walk,
            vec![TypeId::from("GcpSecretPayload"), TypeId::from("String")]
        );
        let aws_walk = registry
            .coercion_path(&TypeId::from("AwsSecretValue"), &TypeId::from("String"))
            .expect("aws value should widen to String");
        assert_eq!(
            aws_walk,
            vec![TypeId::from("AwsSecretValue"), TypeId::from("String")]
        );

        // Cross-provider coercion must remain impossible in both directions.
        assert!(registry
            .coercion_path(
                &TypeId::from("GcpSecretPayload"),
                &TypeId::from("AwsSecretValue")
            )
            .is_none());
        assert!(registry
            .coercion_path(
                &TypeId::from("AwsSecretValue"),
                &TypeId::from("GcpSecretPayload")
            )
            .is_none());
    }

    #[test]
    fn test_coercion_path_cross_provider_tokens_widen_to_credential_base_only() {
        let mut registry = TypeRegistry::with_primitives();
        registry.register(
            "Credential",
            type_lib::refined("String", vec![crate::type_op::Predicate::NonEmpty]),
        );
        registry.register(
            "GcpAccessToken",
            type_lib::refined(
                "Credential",
                vec![crate::type_op::Predicate::Matches(
                    "^ya29\\..+$".to_string(),
                )],
            ),
        );
        registry.register(
            "AwsSessionToken",
            type_lib::refined(
                "Credential",
                vec![crate::type_op::Predicate::Matches(
                    "^ASIA[0-9A-Z]+$".to_string(),
                )],
            ),
        );

        assert_eq!(
            registry.coercion_path(&TypeId::from("GcpAccessToken"), &TypeId::from("Credential")),
            Some(vec![
                TypeId::from("GcpAccessToken"),
                TypeId::from("Credential")
            ])
        );
        assert_eq!(
            registry.coercion_path(
                &TypeId::from("AwsSessionToken"),
                &TypeId::from("Credential")
            ),
            Some(vec![
                TypeId::from("AwsSessionToken"),
                TypeId::from("Credential")
            ])
        );
        assert!(
            registry
                .coercion_path(
                    &TypeId::from("AwsSessionToken"),
                    &TypeId::from("GcpAccessToken")
                )
                .is_none(),
            "provider token types should not coerce directly to each other"
        );
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
    fn test_text_file_path_coercion_chain() {
        let registry = TypeRegistry::with_core_types();

        // TextFilePath → FilePath is safe (widening via coercion edge)
        assert!(registry.is_compatible(&TypeId::from("TextFilePath"), &TypeId::from("FilePath")));

        // FilePath → TextFilePath is NOT safe (narrowing)
        assert!(!registry.is_compatible(&TypeId::from("FilePath"), &TypeId::from("TextFilePath")));

        // TextFilePath → String is safe (multi-step widening)
        let path = registry
            .coercion_path(&TypeId::from("TextFilePath"), &TypeId::from("String"))
            .expect("TextFilePath should widen to String");
        assert_eq!(
            path,
            vec![
                TypeId::from("TextFilePath"),
                TypeId::from("FilePath"),
                TypeId::from("String"),
            ]
        );

        // BinaryFilePath → FilePath is safe
        assert!(registry.is_compatible(&TypeId::from("BinaryFilePath"), &TypeId::from("FilePath")));

        // BinaryFilePath → TextFilePath is NOT safe (different brands)
        assert!(!registry.is_compatible(
            &TypeId::from("BinaryFilePath"),
            &TypeId::from("TextFilePath")
        ));
    }

    #[test]
    fn test_domain_types_registered() {
        let registry = TypeRegistry::with_core_types();

        assert!(registry.contains(&TypeId::from("TextFilePath")));
        assert!(registry.contains(&TypeId::from("BinaryFilePath")));
        assert!(registry.contains(&TypeId::from("ContentEncoding")));
        assert!(registry.contains(&TypeId::from("NonEmptyStr")));
        assert!(registry.contains(&TypeId::from("LanguageId")));
        assert!(registry.contains(&TypeId::from("GitRef")));
        assert!(registry.contains(&TypeId::from("ProjectId")));
        assert!(registry.contains(&TypeId::from("ServiceAccountEmail")));
        assert!(registry.contains(&TypeId::from("WarningPolicy")));
        assert!(registry.contains(&TypeId::from("CloudRuntime")));
        assert!(registry.contains(&TypeId::from("AuthScheme")));
        assert!(registry.contains(&TypeId::from("FermiDepth")));
        assert!(registry.contains(&TypeId::from("TransportClass")));
        assert!(registry.contains(&TypeId::from("TestClass")));
        assert!(registry.contains(&TypeId::from("DisplayWidth")));
        assert!(registry.contains(&TypeId::from("SemanticColor")));
        assert!(registry.contains(&TypeId::from("Tier")));
        assert!(registry.contains(&TypeId::from("SymbolId")));
        assert!(registry.contains(&TypeId::from("TransportRequest")));
        assert!(registry.contains(&TypeId::from("TransportResponse")));
        assert!(registry.contains(&TypeId::from("Credential")));
        assert!(registry.contains(&TypeId::from("FilesystemHandle")));
        assert!(registry.contains(&TypeId::from("NetworkHandle")));
        assert!(registry.contains(&TypeId::from("CliResult")));
        assert!(registry.contains(&TypeId::from("ToolHandle")));
        assert!(registry.contains(&TypeId::from("Platform")));
        assert!(registry.contains(&TypeId::from("Timestamp")));
        assert!(registry.contains(&TypeId::from("Record")));
        assert!(registry.contains(&TypeId::from("FileResponse")));
        assert!(registry.contains(&TypeId::from("ShellResponse")));
        assert!(registry.contains(&TypeId::from("RestResponse")));
        assert!(registry.contains(&TypeId::from("HttpResponse")));
        assert!(registry.contains(&TypeId::from("OidcAudience")));
        assert!(registry.contains(&TypeId::from("WifAudience")));
        assert!(registry.contains(&TypeId::from("GcpProjectId")));
        assert!(registry.contains(&TypeId::from("GcpSecretId")));
        assert!(registry.contains(&TypeId::from("GcpSecretVersion")));
        assert!(registry.contains(&TypeId::from("GcpServiceAccountEmail")));
        assert!(registry.contains(&TypeId::from("GcpSubjectToken")));
        assert!(registry.contains(&TypeId::from("OidcSubjectToken")));
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

    #[test]
    fn test_value_backing_matches_free_function() {
        use crate::types::value_backing_for_type_id;

        let registry = TypeRegistry::with_core_types();

        let cases = [
            // Primitives
            "String",
            "Bool",
            "Int",
            "Float",
            "Bytes",
            "Json",
            "Secret",
            // Domain types (string-backed)
            "FilePath",
            "Url",
            "Email",
            "NonEmptyString",
            "Platform",
            "GcpProjectId",
            "GcpSecretId",
            "OidcAudience",
            // Domain types (other backings)
            "Credential",
            "Timestamp",
            // Parametric
            "List<String>",
            "Set<String>",
            "Map<String,Int>",
            "Optional<String>",
            "Optional<Int>",
            // Legacy aliases
            "StringList",
            "UrlList",
        ];

        for type_name in &cases {
            let expected = value_backing_for_type_id(type_name);
            let actual = registry.value_backing(&TypeId::from(*type_name));
            assert_eq!(
                expected, actual,
                "value_backing mismatch for '{}': expected {:?}, got {:?}",
                type_name, expected, actual
            );
        }
    }

    #[test]
    fn test_value_backing_regression_credential() {
        use crate::types::ValueBacking;
        let r = TypeRegistry::with_core_types();
        assert_eq!(
            r.value_backing(&TypeId::from("Credential")),
            ValueBacking::Map
        );
    }

    #[test]
    fn test_value_backing_regression_tool_handle() {
        use crate::types::ValueBacking;
        let r = TypeRegistry::with_core_types();
        assert_eq!(
            r.value_backing(&TypeId::from("ToolHandle")),
            ValueBacking::Json
        );
    }

    #[test]
    fn test_value_backing_regression_platform() {
        use crate::types::ValueBacking;
        let r = TypeRegistry::with_core_types();
        assert_eq!(
            r.value_backing(&TypeId::from("Platform")),
            ValueBacking::String
        );
    }

    #[test]
    fn test_value_backing_regression_timestamp() {
        use crate::types::ValueBacking;
        let r = TypeRegistry::with_core_types();
        assert_eq!(
            r.value_backing(&TypeId::from("Timestamp")),
            ValueBacking::Int
        );
    }

    #[test]
    fn test_value_backing_regression_parametric_types() {
        use crate::types::ValueBacking;
        let r = TypeRegistry::with_core_types();
        assert_eq!(
            r.value_backing(&TypeId::from("List<String>")),
            ValueBacking::List
        );
        assert_eq!(
            r.value_backing(&TypeId::from("Set<Int>")),
            ValueBacking::Set
        );
        assert_eq!(
            r.value_backing(&TypeId::from("Map<String,Bool>")),
            ValueBacking::Map
        );
        assert_eq!(
            r.value_backing(&TypeId::from("Optional<Float>")),
            ValueBacking::Float
        );
        assert_eq!(
            r.value_backing(&TypeId::from("Optional<Credential>")),
            ValueBacking::Map
        );
    }

    #[test]
    fn test_value_backing_regression_transport_types() {
        use crate::types::ValueBacking;
        let r = TypeRegistry::with_core_types();
        assert_eq!(
            r.value_backing(&TypeId::from("TransportRequest")),
            ValueBacking::Json
        );
        assert_eq!(
            r.value_backing(&TypeId::from("TransportResponse")),
            ValueBacking::Json
        );
        assert_eq!(
            r.value_backing(&TypeId::from("FilesystemHandle")),
            ValueBacking::Json
        );
        assert_eq!(
            r.value_backing(&TypeId::from("NetworkHandle")),
            ValueBacking::Json
        );
    }

    #[test]
    fn test_value_backing_regression_coercion_types() {
        use crate::types::ValueBacking;
        let r = TypeRegistry::with_core_types();
        // FilePath coerces to String via identity chain
        assert_eq!(
            r.value_backing(&TypeId::from("FilePath")),
            ValueBacking::String
        );
        assert_eq!(r.value_backing(&TypeId::from("Url")), ValueBacking::String);
        assert_eq!(
            r.value_backing(&TypeId::from("Email")),
            ValueBacking::String
        );
        assert_eq!(
            r.value_backing(&TypeId::from("NonEmptyString")),
            ValueBacking::String
        );
    }

    #[test]
    fn test_value_backing_regression_secret() {
        use crate::types::ValueBacking;
        let r = TypeRegistry::with_core_types();
        assert_eq!(
            r.value_backing(&TypeId::from("Secret")),
            ValueBacking::Secret
        );
    }

    #[test]
    fn test_value_backing_regression_unknown_type() {
        use crate::types::ValueBacking;
        let r = TypeRegistry::with_core_types();
        // Unknown types fall back to Json
        assert_eq!(
            r.value_backing(&TypeId::from("CompletelyUnknownType")),
            ValueBacking::Json
        );
    }
}
