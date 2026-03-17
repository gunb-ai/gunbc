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

use crate::contract;
use crate::dag::Dag;
use crate::type_lib;
use crate::type_op::{Predicate, TypeOp, WrapperKind};
use crate::types::{Cardinality, TypeId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
}

impl TypeRegistry {
    /// Create a new empty type registry.
    pub fn new() -> Self {
        Self {
            types: HashMap::new(),
        }
    }

    /// Create a type registry with kernel type placeholders pre-registered.
    ///
    /// Registers Identity DAG placeholders for all primitive type names.
    /// These are sufficient for tests and contexts that don't need full
    /// structural definitions from the DSL.
    pub fn with_primitives() -> Self {
        let mut registry = Self::new();
        registry.register_kernel_types();
        registry
    }

    /// Register the minimal kernel types needed to bootstrap DSL typecheck.
    ///
    /// Kernel types are Identity DAG placeholders that the compiler needs
    /// before any `.dag` files are processed. The DSL files reference these
    /// names in type positions (field types, alias bases, refinement targets).
    /// After typecheck, `merge_dsl_types()` overwrites the placeholders with
    /// structural definitions from the compiled `.dag` files.
    pub fn register_kernel_types(&mut self) {
        self.register("Unit", type_lib::unit());
        self.register("Json", type_lib::json());
        self.register("Any", type_lib::identity("Any"));
        self.register("Record", type_lib::identity("Record"));

        // Structural kernel types — these serve as bootstrap definitions that
        // the DSL merge can override with richer structural DAGs. Bool and
        // Secret are defined here because no DSL file defines them.
        self.register(
            "Bool",
            type_lib::coproduct_resolved(
                "Bool",
                vec![("True", type_lib::unit()), ("False", type_lib::unit())],
            ),
        );
        self.register("Bytes", type_lib::list(type_lib::identity("Byte")));
        self.register(
            "Secret",
            type_lib::branded("Secret", type_lib::identity("String")),
        );

        // String: structural Product matching string_type.dag.
        self.register_product(
            "String",
            vec![("bytes", "Bytes"), ("encoding", "ContentEncoding")],
        );

        // Int, Float: structural kernel definitions matching the DSL files
        // (integer.dag, float.dag). These produce Platform shapes via
        // Width/Signed/Domain predicates, which the emit layer handles.
        self.register(
            "Int",
            type_lib::refined(
                "Int",
                vec![
                    Predicate::Width(64),
                    Predicate::Signed(None),
                    Predicate::Arithmetic,
                ],
            ),
        );
        self.register(
            "Float",
            type_lib::refined(
                "Float",
                vec![
                    Predicate::Width(64),
                    Predicate::Domain("ieee754_binary64".to_string()),
                    Predicate::Arithmetic,
                ],
            ),
        );
    }

    /// Merge types from a DSL typecheck pass into this registry.
    ///
    /// Types resolved from `.dag` files are merged in, overriding any
    /// matching kernel/primitive registrations. This enables the two-phase
    /// bootstrap: kernel types first, then DSL-defined structural types.
    pub fn merge_dsl_types(&mut self, other: &TypeRegistry) {
        for (type_id, type_dag) in &other.types {
            self.types.insert(type_id.clone(), type_dag.clone());
        }
    }

    /// Register types needed by Rust infrastructure code.
    ///
    /// Many of these also have `.dag` definitions (the source of truth).
    /// The Rust registrations are kept because unit tests call
    /// `with_core_types()` without compiling DSL modules. As DSL-only
    /// compilation contexts expand, these can be removed.
    ///
    /// Types that are ONLY in `.dag` (no Rust reference): deleted.
    /// Types in `.dag` AND referenced by Rust tests/infra: kept here.
    pub fn register_core_types(&mut self) {
        // Refined primitives — referenced by Rust infra.
        self.register("NonEmptyString", type_lib::non_empty_string());
        self.register("SecretName", type_lib::non_empty_string());
        self.register("Url", type_lib::url());
        self.register("FilePath", type_lib::file_path());
        self.register("Path", type_lib::file_path());
        self.register("Email", type_lib::email());
        self.register("PositiveInt", type_lib::positive_int());
        self.register("NonNegativeInt", type_lib::non_negative_int());
        self.register("GitRef", type_lib::non_empty_string());

        // Content-encoded file paths — referenced by type_registry tests.
        self.register("TextFilePath", type_lib::text_file_path());
        self.register("BinaryFilePath", type_lib::binary_file_path());

        // Platform — referenced by testgen/codegen.
        self.register_coproduct(
            "Platform",
            vec![("Linux", "Unit"), ("Macos", "Unit"), ("Windows", "Unit")],
        );

        // Transport infrastructure (no .dag definition yet).
        self.register_product(
            "TransportRequest",
            vec![
                ("method", "String"),
                ("url", "String"),
                ("headers", "Json"),
                ("body", "String"),
            ],
        );
        self.register_product(
            "TransportResponse",
            vec![("status", "Int"), ("headers", "Json"), ("body", "String")],
        );
        self.register_product(
            "FileResponse",
            vec![
                ("path", "String"),
                ("success", "Bool"),
                ("content", "String"),
            ],
        );
        self.register_product(
            "ShellResponse",
            vec![
                ("exit_code", "Int"),
                ("stdout", "String"),
                ("stderr", "String"),
            ],
        );
        self.register_product(
            "RestResponse",
            vec![("status", "Int"), ("headers", "Json"), ("body", "Json")],
        );
        self.register_product(
            "HttpResponse",
            vec![("status", "Int"), ("headers", "Json"), ("body", "String")],
        );
        self.register_product(
            "CliResult",
            vec![
                ("stdout", "String"),
                ("stderr", "String"),
                ("exit_code", "Int"),
            ],
        );

        // Resource/handle types (no .dag definition yet).
        self.register(
            "Credential",
            type_lib::branded("Credential", type_lib::string()),
        );
        self.register(
            "FilesystemHandle",
            type_lib::branded("FilesystemHandle", type_lib::file_path()),
        );
        self.register(
            "NetworkHandle",
            type_lib::branded("NetworkHandle", type_lib::unit()),
        );
        self.register(
            "ToolHandle",
            type_lib::branded("ToolHandle", type_lib::string()),
        );

        // Identity/opaque types.
        self.register("Record", type_lib::identity("Record"));

        // GCP/OIDC identity types (no .dag definition yet).
        self.register("OidcAudience", type_lib::non_empty_string());
        self.register("WifAudience", type_lib::non_empty_string());
        self.register("GcpProjectId", type_lib::non_empty_string());
        self.register("GcpSecretId", type_lib::non_empty_string());
        self.register("GcpSecretVersion", type_lib::non_empty_string());
        self.register("GcpServiceAccountEmail", type_lib::non_empty_string());
        self.register("GcpSubjectToken", type_lib::non_empty_string());
        self.register("OidcSubjectToken", type_lib::non_empty_string());
    }

    /// Create a type registry with primitives + common refined/core types.
    pub fn with_core_types() -> Self {
        let mut registry = Self::with_primitives();
        registry.register_core_types();
        registry
    }

    /// Create a type registry with all default types, ready for DSL merge.
    ///
    /// Registers kernel types and core types. After construction,
    /// call [`merge_dsl_types`] to override with structural definitions from
    /// compiled `.dag` files.
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry.register_kernel_types();
        registry.register_core_types();
        registry
    }

    /// Register a type DAG with a name.
    pub fn register(&mut self, name: impl Into<TypeId>, type_dag: Dag<TypeOp>) {
        self.types.insert(name.into(), type_dag);
    }

    /// Register a product type with field types resolved through the registry.
    ///
    /// For each field, if the field's type name is already registered, the
    /// resolved type DAG is embedded directly. Otherwise, falls back to an
    /// identity wrapper. This enables structural recursion through record
    /// boundaries.
    pub fn register_product(&mut self, name: &str, fields: Vec<(&str, &str)>) {
        let resolved: Vec<(&str, Dag<TypeOp>)> = fields
            .into_iter()
            .map(|(field_name, type_name)| {
                let type_id = TypeId::from(type_name);
                let dag = self
                    .types
                    .get(&type_id)
                    .cloned()
                    .unwrap_or_else(|| type_lib::identity(type_name));
                (field_name, dag)
            })
            .collect();
        self.register(name, type_lib::product_resolved(name, resolved));
    }

    /// Register a product type, returning unresolved field type names instead of
    /// silently falling back to identity wrappers.
    pub fn register_product_checked(
        &mut self,
        name: &str,
        fields: Vec<(&str, &str)>,
    ) -> Result<(), Vec<String>> {
        let mut unresolved = Vec::new();
        let resolved: Vec<(&str, Dag<TypeOp>)> = fields
            .into_iter()
            .map(|(field_name, type_name)| {
                let type_id = TypeId::from(type_name);
                let dag = self.types.get(&type_id).cloned().unwrap_or_else(|| {
                    unresolved.push(format!("{name}.{field_name}: {type_name}"));
                    type_lib::identity(type_name)
                });
                (field_name, dag)
            })
            .collect();
        self.register(name, type_lib::product_resolved(name, resolved));
        if unresolved.is_empty() {
            Ok(())
        } else {
            Err(unresolved)
        }
    }

    /// Register a coproduct type with variant types resolved through the registry.
    ///
    /// For each variant, if the variant's type name is already registered, the
    /// resolved type DAG is embedded directly. Otherwise, falls back to an
    /// identity wrapper.
    pub fn register_coproduct(&mut self, name: &str, variants: Vec<(&str, &str)>) {
        let resolved: Vec<(&str, Dag<TypeOp>)> = variants
            .into_iter()
            .map(|(variant_name, type_name)| {
                let type_id = TypeId::from(type_name);
                let dag = self
                    .types
                    .get(&type_id)
                    .cloned()
                    .unwrap_or_else(|| type_lib::identity(type_name));
                (variant_name, dag)
            })
            .collect();
        self.register(name, type_lib::coproduct_resolved(name, resolved));
    }

    /// Register a coproduct type, returning unresolved variant type names instead of
    /// silently falling back to identity wrappers.
    pub fn register_coproduct_checked(
        &mut self,
        name: &str,
        variants: Vec<(&str, &str)>,
    ) -> Result<(), Vec<String>> {
        let mut unresolved = Vec::new();
        let resolved: Vec<(&str, Dag<TypeOp>)> = variants
            .into_iter()
            .map(|(variant_name, type_name)| {
                let type_id = TypeId::from(type_name);
                let dag = self.types.get(&type_id).cloned().unwrap_or_else(|| {
                    unresolved.push(format!("{name}::{variant_name}: {type_name}"));
                    type_lib::identity(type_name)
                });
                (variant_name, dag)
            })
            .collect();
        self.register(name, type_lib::coproduct_resolved(name, resolved));
        if unresolved.is_empty() {
            Ok(())
        } else {
            Err(unresolved)
        }
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
    /// Tries direct name lookup first so that inline record types (containing
    /// commas/braces) are reachable without going through `parse_type_expr`.
    pub fn resolve_type_checked(
        &self,
        type_id: &TypeId,
    ) -> Result<Option<Dag<TypeOp>>, TypeExprError> {
        // Direct lookup first — handles names that parse_type_expr rejects
        // (e.g. inline record types like `{key: String, value: String}`).
        if let Some(dag) = self.get_by_name(&type_id.0) {
            return Ok(Some(dag.clone()));
        }
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
                    WrapperKind::Map => type_lib::map(type_lib::string(), inner_dag),
                };
                Some(dag)
            }
            TypeExpr::Map(key, value) => {
                let key_dag = self.resolve_expr(key, ResolveMode::InWrapper)?;
                let value_dag = self.resolve_expr(value, ResolveMode::InWrapper)?;
                let name = render_type_expr(expr);
                if let Some(dag) = self.get_by_name(&name) {
                    return Some(dag.clone());
                }
                Some(type_lib::map(key_dag, value_dag))
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

    /// Return all registered type names.
    pub fn type_names(&self) -> Vec<&TypeId> {
        self.types.keys().collect()
    }

    /// Audit identity types: returns TypeIds whose registered DAG is a pure
    /// identity (single Identity node, no Validate/Product/Coproduct/Brand/Wrap).
    ///
    /// A pure identity DAG is a diagnostic signal — it means the type has no
    /// structural definition and will be emitted as Opaque. The goal is to
    /// ratchet this count downward over time.
    pub fn audit_identity_types(&self) -> Vec<TypeId> {
        use crate::node::NodeBody;
        let mut identities = Vec::new();
        for (type_id, dag) in &self.types {
            let is_pure_identity = dag.nodes.len() == 1
                && matches!(dag.nodes[0].body, NodeBody::Opaque(TypeOp::Identity));
            if is_pure_identity {
                identities.push(type_id.clone());
            }
        }
        identities.sort_by(|a, b| a.0.cmp(&b.0));
        identities
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
    /// Recurses through containers, brands, and SubDags to find the innermost
    /// base type name.
    ///
    /// Returns `None` if the type is not registered.
    pub fn base_type_name(&self, type_id: &TypeId) -> Option<String> {
        let dag = self.resolve_type(type_id)?;
        base_type_recursive(&dag)
    }

    /// Check if type A is compatible with type B.
    ///
    /// Compatibility is determined by structural DAG analysis:
    /// 1. Same type name (exact match)
    /// 2. Target is "Any" (accepts anything)
    /// 3. Structural shape compatibility (DAG-derived, no nominal fallbacks)
    /// 4. Predicate entailment: source predicates cover target predicates
    pub fn is_compatible(&self, from: &TypeId, to: &TypeId) -> bool {
        // Same type is always compatible.
        if from == to {
            return true;
        }

        // Target Any accepts anything.
        if to.0 == "Any" {
            return true;
        }

        // Source Any: inferred types that couldn't be resolved
        // (e.g. fold's generic return). Compatible with any target until
        // the type system supports generics.
        if from.0 == "Any" {
            return true;
        }

        // Look up both types; if not registered, fall back to Json top.
        let (Some(from_dag), Some(to_dag)) = (self.resolve_type(from), self.resolve_type(to))
        else {
            // Unregistered types: only compatible if target is Json (top type).
            return to.0 == "Json";
        };

        // Structural shape compatibility: walk the type DAGs directly.
        let Ok(from_shape) = crate::type_shape::type_shape(&from_dag) else {
            return to.0 == "Json";
        };
        let Ok(to_shape) = crate::type_shape::type_shape(&to_dag) else {
            return to.0 == "Json";
        };
        if structural_shapes_compatible(&from_shape, &to_shape) {
            return true;
        }

        // Container wrapping: scalar → List/Optional/Set of compatible type.
        // This enables fan-in edges (scalar output → list input).
        // Uses element-level type name comparison (not structural_shapes_compatible)
        // to handle Opaque types correctly.
        if let crate::type_shape::TypeShape::Container(container) = &to_shape {
            let inner = match container {
                crate::type_shape::ContainerShape::List(inner)
                | crate::type_shape::ContainerShape::Optional(inner)
                | crate::type_shape::ContainerShape::Set(inner) => inner.as_ref(),
                crate::type_shape::ContainerShape::Map(_, _) => return false,
            };
            // Name-based wrapping: extract declared type names from any
            // shape variant (Opaque, named Product, named Coproduct) and
            // compare. This handles structural types (e.g., Product("String"))
            // being wrapped into containers built with identity elements
            // (e.g., List(Opaque("String"))).
            let from_name = shape_declared_name(&from_shape);
            let inner_name = shape_declared_name(inner);
            if let (Some(a), Some(b)) = (from_name, inner_name) {
                if a == b {
                    return true;
                }
            }
            if structural_shapes_compatible(&from_shape, inner) {
                return true;
            }
        }

        // Predicate entailment: check if source predicates cover target.
        let from_preds = contract::predicates(&from_dag);
        let to_preds = contract::predicates(&to_dag);
        let from_base = contract::base_type(&from_dag);
        let to_base = contract::base_type(&to_dag);

        // Same base type (or target is Json top) + predicates entailed → compatible.
        let base_ok = match (&from_base, &to_base) {
            (Some(a), Some(b)) => {
                a == b
                    || b == "Json"
                    // Refinement chain: from's base type is itself a refinement of to's base.
                    // Guard against infinite recursion: only recurse if at least one base
                    // differs from the original type names.
                    || (!(a.as_str() == from.0.as_str() && b.as_str() == to.0.as_str())
                        && self.is_compatible(&TypeId::from(a.as_str()), &TypeId::from(b.as_str())))
            }
            (None, None) => true,
            _ => false,
        };

        // If both types are Brands with different names, they're incompatible
        // regardless of base types or predicates. Brand enforces nominal distinctness.
        let Ok(from_shape) = crate::type_shape::type_shape(&from_dag) else {
            return base_ok;
        };
        let Ok(to_shape) = crate::type_shape::type_shape(&to_dag) else {
            return base_ok;
        };
        if let (
            crate::type_shape::TypeShape::Brand(fn_, _),
            crate::type_shape::TypeShape::Brand(tn, _),
        ) = (&from_shape, &to_shape)
        {
            if fn_ != tn {
                return false;
            }
        }

        if base_ok
            && to_preds
                .iter()
                .all(|tp| from_preds.iter().any(|sp| sp.entails(tp)))
        {
            return true;
        }

        false
    }

    /// Strict semantic carrier compatibility.
    ///
    /// Uses registry-based classification for both types. Structural types
    /// are mutually compatible; non-structural carrier kinds must match exactly.
    pub fn is_type_compatible(&self, from: &TypeId, to: &TypeId) -> bool {
        use crate::types::SemanticCarrierKind as Kind;
        let from_kind = self.semantic_carrier_kind(from);
        let to_kind = self.semantic_carrier_kind(to);
        match (from_kind, to_kind) {
            (Kind::Structural, Kind::Structural) => true,
            (lhs, rhs) => lhs == rhs,
        }
    }

    /// Check structural + strict semantic-carrier compatibility.
    ///
    /// This is stricter than [`Self::is_compatible`]:
    /// - structural compatibility must hold
    /// - semantic carrier kinds must be compatible (no semantic→structural fallback)
    /// - unknown semantic carriers are rejected (fail-closed)
    pub fn is_compatible_strict_semantic(&self, from: &TypeId, to: &TypeId) -> bool {
        self.is_compatible(from, to) && self.is_type_compatible(from, to)
    }

    /// Determine the runtime `ValueBacking` for a type using registry knowledge.
    ///
    /// This replaces the free function `value_backing_for_type_id()` by using
    /// the registry's type DAGs and coercion paths instead of the hardcoded
    /// `PortType` enum.
    pub fn value_backing(&self, type_id: &TypeId) -> Result<crate::types::ValueBacking, String> {
        use crate::types::{
            optional_inner_type_id, parse_map_type_id, parse_unary_generic_type_id, ValueBacking,
        };

        let raw = &type_id.0;

        // Credential is a structured map payload at runtime.
        if raw == "Credential" {
            return Ok(ValueBacking::Map);
        }

        // Parametric containers.
        if parse_map_type_id(raw).is_some() {
            return Ok(ValueBacking::Map);
        }
        if parse_unary_generic_type_id(raw, "Set").is_some() {
            return Ok(ValueBacking::Set);
        }
        if parse_unary_generic_type_id(raw, "List").is_some() {
            return Ok(ValueBacking::List);
        }
        if let Some(inner) = optional_inner_type_id(raw) {
            return self.value_backing(&TypeId::from(inner));
        }

        // Primitives (direct match on well-known type names).
        match raw.as_str() {
            "String" => return Ok(ValueBacking::String),
            "Bool" => return Ok(ValueBacking::Bool),
            "Int" => return Ok(ValueBacking::Int),
            "Float" => return Ok(ValueBacking::Float),
            "Bytes" => return Ok(ValueBacking::Bytes),
            "Json" => return Ok(ValueBacking::Json),
            "Unit" => return Ok(ValueBacking::Unit),
            "Secret" => return Ok(ValueBacking::Secret),
            _ => {}
        }

        // Registry-driven: find the nearest primitive ancestor via structural compatibility.
        static PRIMITIVE_BACKINGS: &[(&str, ValueBacking)] = &[
            ("String", ValueBacking::String),
            ("Int", ValueBacking::Int),
            ("Float", ValueBacking::Float),
            ("Bool", ValueBacking::Bool),
            ("Bytes", ValueBacking::Bytes),
            ("Secret", ValueBacking::Secret),
        ];
        for &(prim, backing) in PRIMITIVE_BACKINGS {
            if self.is_compatible(type_id, &TypeId::from(prim)) {
                return Ok(backing);
            }
        }

        // Identity types (no coercion path to primitives): use semantic carrier
        // classification to determine backing.
        use crate::types::SemanticCarrierKind;
        match self.semantic_carrier_kind(type_id) {
            SemanticCarrierKind::Structural | SemanticCarrierKind::UnknownSemantic => {} // fall through to suffix/error path
            SemanticCarrierKind::Platform => return Ok(ValueBacking::String),
            SemanticCarrierKind::Timestamp => return Ok(ValueBacking::Int),
            SemanticCarrierKind::TransportRequest
            | SemanticCarrierKind::TransportResponse
            | SemanticCarrierKind::FilesystemHandle
            | SemanticCarrierKind::NetworkHandle
            | SemanticCarrierKind::ToolHandle => return Ok(ValueBacking::Json),
            SemanticCarrierKind::Credential => return Ok(ValueBacking::Map),
            SemanticCarrierKind::Secret => return Ok(ValueBacking::Secret),
        }

        // S23/S35: Structural classification from the resolved type shape.
        // This reads the outer composition instead of scanning descendant
        // nodes, so wrapper/container aliases keep their own backing.
        if let Some(dag) = self.resolve_type(type_id) {
            if let Ok(shape) = crate::type_shape::type_shape(&dag) {
                if let Some(backing) = value_backing_from_shape(&shape) {
                    return Ok(backing);
                }
            }
        }

        // Legacy suffix-based aliases.
        if raw.ends_with("List") {
            return Ok(ValueBacking::List);
        }
        if raw.ends_with("Set") {
            return Ok(ValueBacking::Set);
        }

        // Fallback: unknown type — return an error.
        Err(format!(
            "unknown type '{}' has no known ValueBacking",
            type_id.0
        ))
    }

    /// Classify a type's semantic carrier kind using registry knowledge.
    ///
    /// Checks the type name first (handles branded types like Secret that
    /// structurally wrap String but carry semantic meaning), then falls back
    /// to structural DAG inspection so container wrappers keep branded inner
    /// semantic carriers visible.
    pub fn semantic_carrier_kind(&self, type_id: &TypeId) -> crate::types::SemanticCarrierKind {
        // Check the outer type name first — branded types like Secret
        // have semantic meaning that shouldn't be lost by resolving to
        // their structural base (String).
        let outer_kind = crate::types::semantic_carrier_kind_for_type_name(&type_id.0);
        if outer_kind != crate::types::SemanticCarrierKind::UnknownSemantic {
            return outer_kind;
        }

        if let Some(dag) = self.resolve_type(type_id) {
            if let Ok(shape) = crate::type_shape::type_shape(&dag) {
                return semantic_carrier_kind_from_shape(&shape);
            }
            return crate::types::SemanticCarrierKind::Structural;
        }
        outer_kind
    }

    /// Classify a type's semantic carrier class using registry knowledge.
    pub fn semantic_carrier_class(&self, type_id: &TypeId) -> crate::types::SemanticCarrierClass {
        match self.semantic_carrier_kind(type_id) {
            crate::types::SemanticCarrierKind::Structural => {
                crate::types::SemanticCarrierClass::StructuralGeneratable
            }
            _ => crate::types::SemanticCarrierClass::SemanticCarrier,
        }
    }

    /// Classify placeholder seed policy using registry knowledge.
    pub fn seed_placeholder_policy(&self, type_id: &TypeId) -> crate::types::SeedPlaceholderPolicy {
        match self.semantic_carrier_class(type_id) {
            crate::types::SemanticCarrierClass::StructuralGeneratable => {
                crate::types::SeedPlaceholderPolicy::Generated
            }
            crate::types::SemanticCarrierClass::SemanticCarrier => {
                crate::types::SeedPlaceholderPolicy::ExplicitSeedRequired
            }
        }
    }
}

fn value_backing_from_shape(
    shape: &crate::type_shape::TypeShape,
) -> Option<crate::types::ValueBacking> {
    use crate::type_shape::{ContainerShape, TypeShape};
    use crate::types::ValueBacking;

    match shape {
        TypeShape::Coproduct(..) => Some(ValueBacking::String),
        TypeShape::Product(..) => Some(ValueBacking::Map),
        TypeShape::Brand(_, inner) => value_backing_from_shape(inner),
        TypeShape::Container(ContainerShape::Optional(inner)) => value_backing_from_shape(inner),
        TypeShape::Container(ContainerShape::List(_)) => Some(ValueBacking::List),
        TypeShape::Container(ContainerShape::Set(_)) => Some(ValueBacking::Set),
        TypeShape::Container(ContainerShape::Map(..)) => Some(ValueBacking::Map),
        TypeShape::Platform(_) | TypeShape::Opaque(_) => None,
    }
}

/// Recursively extract the innermost base type from a type DAG.
///
/// For containers, prefers semantically meaningful SubDags by name:
/// `value_type` (Map), `element_type` (List/Set), `inner_type` (Optional/Brand).
fn base_type_recursive(dag: &Dag<TypeOp>) -> Option<String> {
    if let Some(base) = contract::base_type(dag) {
        return Some(base);
    }
    // Prefer named SubDags that carry the semantic element type.
    for preferred in &["value_type", "element_type", "inner_type"] {
        for node in &dag.nodes {
            if node.id.0 == *preferred {
                if let crate::node::NodeBody::SubDag(inner, _) = &node.body {
                    if let Some(base) = base_type_recursive(inner) {
                        return Some(base);
                    }
                }
            }
        }
    }
    for node in &dag.nodes {
        if let crate::node::NodeBody::SubDag(inner, _) = &node.body {
            if let Some(base) = base_type_recursive(inner) {
                return Some(base);
            }
        }
    }
    None
}

/// Extract the declared type name from any shape variant.
///
/// Handles `Opaque(n)`, `Product(Some(n), _)`, `Coproduct(Some(n), _)`, and `Brand(n, _)`.
fn shape_declared_name(shape: &crate::type_shape::TypeShape) -> Option<&str> {
    match shape {
        crate::type_shape::TypeShape::Opaque(n) => Some(n.as_str()),
        crate::type_shape::TypeShape::Product(Some(n), _) => Some(n.as_str()),
        crate::type_shape::TypeShape::Coproduct(Some(n), _) => Some(n.as_str()),
        crate::type_shape::TypeShape::Brand(n, _) => Some(n.as_str()),
        _ => None,
    }
}

fn semantic_carrier_kind_from_shape(
    shape: &crate::type_shape::TypeShape,
) -> crate::types::SemanticCarrierKind {
    use crate::type_shape::{ContainerShape, TypeShape};
    use crate::types::SemanticCarrierKind;

    if let Some(name) = shape_declared_name(shape) {
        let kind = crate::types::semantic_carrier_kind_for_type_name(name);
        if kind != SemanticCarrierKind::UnknownSemantic {
            return kind;
        }
    }

    match shape {
        TypeShape::Brand(_, inner)
        | TypeShape::Container(ContainerShape::Optional(inner))
        | TypeShape::Container(ContainerShape::List(inner))
        | TypeShape::Container(ContainerShape::Set(inner)) => {
            semantic_carrier_kind_from_shape(inner)
        }
        TypeShape::Container(ContainerShape::Map(_, value)) => {
            semantic_carrier_kind_from_shape(value)
        }
        TypeShape::Platform(_)
        | TypeShape::Coproduct(..)
        | TypeShape::Product(..)
        | TypeShape::Opaque(..) => SemanticCarrierKind::Structural,
    }
}

/// Structural shape compatibility check.
///
/// Two type shapes are compatible if the source shape's values can safely
/// inhabit the target shape. This is the structural (DAG-derived) alternative
/// to nominal type equality.
/// - Opaque: same name only
fn structural_shapes_compatible(
    from: &crate::type_shape::TypeShape,
    to: &crate::type_shape::TypeShape,
) -> bool {
    use crate::type_shape::{ContainerShape, TypeShape};

    match (from, to) {
        // Opaque types: defer to predicate entailment check in is_compatible.
        // Same-name Opaque types may have different predicates (e.g., String vs Url).
        (TypeShape::Opaque(_), _) | (_, TypeShape::Opaque(_)) => false,

        // Identical structural shapes.
        (a, b) if a == b => true,

        // Platform: source must satisfy target's constraints.
        (TypeShape::Platform(from_props), TypeShape::Platform(to_props)) => {
            // If target requires specific width, source must match.
            if let Some(tw) = to_props.width {
                if from_props.width != Some(tw) {
                    return false;
                }
            }
            // If target requires specific signedness, source must match.
            if let Some(ts) = to_props.signed {
                if from_props.signed != Some(ts) {
                    return false;
                }
            }
            // If target requires specific domain, source must match.
            if let Some(td) = &to_props.domain {
                if from_props.domain.as_ref() != Some(td) {
                    return false;
                }
            }
            true
        }

        // Container covariance: List<A> → List<B> if A → B.
        (TypeShape::Container(from_c), TypeShape::Container(to_c)) => match (from_c, to_c) {
            (ContainerShape::List(a), ContainerShape::List(b))
            | (ContainerShape::Optional(a), ContainerShape::Optional(b))
            | (ContainerShape::Set(a), ContainerShape::Set(b)) => {
                structural_shapes_compatible(a, b)
            }
            (ContainerShape::Map(ak, av), ContainerShape::Map(bk, bv)) => {
                structural_shapes_compatible(ak, bk) && structural_shapes_compatible(av, bv)
            }
            _ => false,
        },

        // Coproduct subset: A|B coerces to A|B|C.
        (TypeShape::Coproduct(_, from_variants), TypeShape::Coproduct(_, to_variants)) => {
            from_variants.iter().all(|(name, shape)| {
                to_variants
                    .iter()
                    .any(|(tn, ts)| tn == name && structural_shapes_compatible(shape, ts))
            })
        }

        // Product: same fields, each field compatible.
        (TypeShape::Product(_, from_fields), TypeShape::Product(_, to_fields)) => {
            from_fields.len() == to_fields.len()
                && from_fields
                    .iter()
                    .zip(to_fields.iter())
                    .all(|((fn_, fs), (tn, ts))| fn_ == tn && structural_shapes_compatible(fs, ts))
        }

        // Brand: a branded source can coerce to its unwrapped structural
        // target, but not vice versa (brands enforce nominal distinctness).
        // Two brands with different names are incompatible.
        (TypeShape::Brand(from_name, from_inner), TypeShape::Brand(to_name, to_inner)) => {
            from_name == to_name && structural_shapes_compatible(from_inner, to_inner)
        }
        // Branded → non-branded: strip brand and compare.
        (TypeShape::Brand(_, inner), other) => structural_shapes_compatible(inner, other),

        _ => false,
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
        assert!(registry.contains(&TypeId::from("Any")));
        assert!(registry.contains(&TypeId::from("Record")));
        assert_eq!(registry.len(), 10);
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
    fn test_text_file_path_compatibility() {
        let registry = TypeRegistry::with_core_types();

        // TextFilePath → String is safe (refinement widening)
        assert!(registry.is_compatible(&TypeId::from("TextFilePath"), &TypeId::from("String")));

        // FilePath → TextFilePath is NOT safe (narrowing)
        assert!(!registry.is_compatible(&TypeId::from("FilePath"), &TypeId::from("TextFilePath")));

        // BinaryFilePath → TextFilePath is NOT safe (different brands)
        assert!(!registry.is_compatible(
            &TypeId::from("BinaryFilePath"),
            &TypeId::from("TextFilePath")
        ));
    }

    #[test]
    fn test_domain_types_registered() {
        let registry = TypeRegistry::with_core_types();

        // Types still registered in Rust (infrastructure / no .dag yet).
        assert!(registry.contains(&TypeId::from("TextFilePath")));
        assert!(registry.contains(&TypeId::from("BinaryFilePath")));
        assert!(registry.contains(&TypeId::from("GitRef")));
        assert!(registry.contains(&TypeId::from("Platform")));
        assert!(registry.contains(&TypeId::from("TransportRequest")));
        assert!(registry.contains(&TypeId::from("TransportResponse")));

        // These types are now defined in .dag files and only available
        // after DSL compilation (via merge_dsl_types). They are NOT in
        // with_core_types() anymore:
        // ContentEncoding, NonEmptyStr, LanguageId, ProjectId,
        // ServiceAccountEmail, WarningPolicy, CloudRuntime, AuthScheme,
        // FermiDepth, TransportClass, TestClass, DisplayWidth,
        // SemanticColor, Tier, SymbolId
        assert!(registry.contains(&TypeId::from("Credential")));
        assert!(registry.contains(&TypeId::from("FilesystemHandle")));
        assert!(registry.contains(&TypeId::from("NetworkHandle")));
        assert!(registry.contains(&TypeId::from("CliResult")));
        assert!(registry.contains(&TypeId::from("ToolHandle")));
        // Timestamp now defined in dsl/std/types.dag
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
    }

    #[test]
    fn test_value_backing_matches_free_function() {
        use crate::types::{value_backing_for_type_id, ValueBacking};

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
            "List<Int>",
            "List<Bool>",
        ];

        for type_name in &cases {
            let expected = value_backing_for_type_id(type_name).unwrap_or(ValueBacking::Json);
            let actual = registry
                .value_backing(&TypeId::from(*type_name))
                .unwrap_or(ValueBacking::Json);
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
            r.value_backing(&TypeId::from("Credential")).unwrap(),
            ValueBacking::Map
        );
    }

    #[test]
    fn test_value_backing_regression_tool_handle() {
        use crate::types::ValueBacking;
        let r = TypeRegistry::with_core_types();
        // ToolHandle is branded("ToolHandle", String) — structurally a String.
        assert_eq!(
            r.value_backing(&TypeId::from("ToolHandle")).unwrap(),
            ValueBacking::String
        );
    }

    #[test]
    fn test_value_backing_regression_platform() {
        use crate::types::ValueBacking;
        let r = TypeRegistry::with_core_types();
        assert_eq!(
            r.value_backing(&TypeId::from("Platform")).unwrap(),
            ValueBacking::String
        );
    }

    #[test]
    fn test_value_backing_regression_timestamp() {
        use crate::types::ValueBacking;
        let r = TypeRegistry::with_core_types();
        assert_eq!(
            r.value_backing(&TypeId::from("Timestamp")).unwrap(),
            ValueBacking::Int
        );
    }

    #[test]
    fn test_value_backing_regression_parametric_types() {
        use crate::types::ValueBacking;
        let r = TypeRegistry::with_core_types();
        assert_eq!(
            r.value_backing(&TypeId::from("List<String>")).unwrap(),
            ValueBacking::List
        );
        assert_eq!(
            r.value_backing(&TypeId::from("Set<Int>")).unwrap(),
            ValueBacking::Set
        );
        assert_eq!(
            r.value_backing(&TypeId::from("Map<String,Bool>")).unwrap(),
            ValueBacking::Map
        );
        assert_eq!(
            r.value_backing(&TypeId::from("Optional<Float>")).unwrap(),
            ValueBacking::Float
        );
        assert_eq!(
            r.value_backing(&TypeId::from("Optional<Credential>"))
                .unwrap(),
            ValueBacking::Map
        );
    }

    #[test]
    fn test_value_backing_regression_container_alias_uses_outer_shape() {
        use crate::types::ValueBacking;

        let mut registry = TypeRegistry::with_core_types();
        registry.register(
            "PayloadList",
            crate::type_lib::list(crate::type_lib::product(
                "Payload",
                vec![("value", "String")],
            )),
        );

        assert_eq!(
            registry
                .value_backing(&TypeId::from("PayloadList"))
                .unwrap(),
            ValueBacking::List
        );
    }

    #[test]
    fn test_value_backing_regression_transport_types() {
        use crate::types::ValueBacking;
        let r = TypeRegistry::with_core_types();
        // TransportRequest/Response are Products — no primitive match,
        // fall through to semantic carrier → Json.
        assert_eq!(
            r.value_backing(&TypeId::from("TransportRequest")).unwrap(),
            ValueBacking::Json
        );
        assert_eq!(
            r.value_backing(&TypeId::from("TransportResponse")).unwrap(),
            ValueBacking::Json
        );
        // FilesystemHandle is branded(FilePath) which is refined(String) —
        // structurally compatible with String.
        assert_eq!(
            r.value_backing(&TypeId::from("FilesystemHandle")).unwrap(),
            ValueBacking::String
        );
        // NetworkHandle is unit().
        assert_eq!(
            r.value_backing(&TypeId::from("NetworkHandle")).unwrap(),
            ValueBacking::Json
        );
    }

    #[test]
    fn test_value_backing_regression_coercion_types() {
        use crate::types::ValueBacking;
        let r = TypeRegistry::with_core_types();
        // FilePath coerces to String via identity chain
        assert_eq!(
            r.value_backing(&TypeId::from("FilePath")).unwrap(),
            ValueBacking::String
        );
        assert_eq!(
            r.value_backing(&TypeId::from("Url")).unwrap(),
            ValueBacking::String
        );
        assert_eq!(
            r.value_backing(&TypeId::from("Email")).unwrap(),
            ValueBacking::String
        );
        assert_eq!(
            r.value_backing(&TypeId::from("NonEmptyString")).unwrap(),
            ValueBacking::String
        );
    }

    #[test]
    fn test_value_backing_regression_secret() {
        use crate::types::ValueBacking;
        let r = TypeRegistry::with_core_types();
        assert_eq!(
            r.value_backing(&TypeId::from("Secret")).unwrap(),
            ValueBacking::Secret
        );
    }

    #[test]
    fn test_value_backing_regression_unknown_type() {
        let r = TypeRegistry::with_core_types();
        // Unknown types return an error.
        assert!(r
            .value_backing(&TypeId::from("CompletelyUnknownType"))
            .is_err());
    }

    #[test]
    fn test_with_defaults_contains_kernel_and_primitives() {
        let registry = TypeRegistry::with_defaults();
        // Kernel types
        assert!(registry.contains(&TypeId::from("Unit")));
        assert!(registry.contains(&TypeId::from("Json")));
        assert!(registry.contains(&TypeId::from("Any")));
        assert!(registry.contains(&TypeId::from("Record")));
        // Primitives
        assert!(registry.contains(&TypeId::from("String")));
        assert!(registry.contains(&TypeId::from("Bool")));
        assert!(registry.contains(&TypeId::from("Int")));
        assert!(registry.contains(&TypeId::from("Float")));
        // Core types
        assert!(registry.contains(&TypeId::from("Url")));
        assert!(registry.contains(&TypeId::from("FilePath")));
    }

    #[test]
    fn test_merge_dsl_types_overrides() {
        let mut registry = TypeRegistry::with_defaults();
        let mut dsl_registry = TypeRegistry::new();
        // Register a structural Int64 type from DSL
        dsl_registry.register(
            "Int64",
            type_lib::refined(
                "Int",
                vec![
                    Predicate::Width(64),
                    Predicate::Signed(None),
                    Predicate::Arithmetic,
                ],
            ),
        );
        registry.merge_dsl_types(&dsl_registry);
        assert!(registry.contains(&TypeId::from("Int64")));
        let dag = registry.resolve_type(&TypeId::from("Int64")).unwrap();
        // Verify the resolved DAG has Width(64) and Signed predicates
        use crate::node::NodeBody;
        use crate::type_op::TypeOp;
        let has_width_64 = dag.nodes.iter().any(|n| {
            matches!(
                &n.body,
                NodeBody::Opaque(TypeOp::Validate(Predicate::Width(64)))
            )
        });
        assert!(has_width_64, "Int64 should have Width(64) predicate");
    }

    #[test]
    fn test_semantic_carrier_kind_on_registry() {
        use crate::types::SemanticCarrierKind;
        let registry = TypeRegistry::with_core_types();
        assert_eq!(
            registry.semantic_carrier_kind(&TypeId::from("String")),
            SemanticCarrierKind::Structural
        );
        assert_eq!(
            registry.semantic_carrier_kind(&TypeId::from("Secret")),
            SemanticCarrierKind::Secret
        );
        assert_eq!(
            registry.semantic_carrier_kind(&TypeId::from("TransportRequest")),
            SemanticCarrierKind::TransportRequest
        );
    }

    #[test]
    fn test_semantic_carrier_kind_resolves_core_container_wrappers() {
        use crate::types::SemanticCarrierKind;
        let registry = TypeRegistry::with_core_types();
        assert_eq!(
            registry.semantic_carrier_kind(&TypeId::from("Optional<FilesystemHandle>")),
            SemanticCarrierKind::FilesystemHandle
        );
    }

    #[test]
    fn test_semantic_carrier_kind_resolves_registered_wrapper_aliases() {
        use crate::types::SemanticCarrierKind;
        let mut registry = TypeRegistry::with_core_types();
        registry.register(
            "MaybeFilesystemHandle",
            type_lib::optional(type_lib::branded("FilesystemHandle", type_lib::file_path())),
        );
        assert_eq!(
            registry.semantic_carrier_kind(&TypeId::from("MaybeFilesystemHandle")),
            SemanticCarrierKind::FilesystemHandle
        );
    }

    #[test]
    fn test_audit_identity_types_detects_pure_identity() {
        let mut registry = TypeRegistry::new();
        registry.register("Foo", type_lib::identity("Foo"));
        registry.register("Url", type_lib::url()); // refined, not identity
        registry.register("Bool", type_lib::bool()); // identity

        let identities = registry.audit_identity_types();
        assert!(identities.contains(&TypeId::from("Foo")));
        assert!(identities.contains(&TypeId::from("Bool")));
        assert!(
            !identities.contains(&TypeId::from("Url")),
            "refined types should not be counted as identity"
        );
    }

    #[test]
    fn test_register_product_checked_reports_unresolved() {
        let mut registry = TypeRegistry::with_primitives();
        let result = registry.register_product_checked(
            "TestRecord",
            vec![("name", "String"), ("age", "UnknownType")],
        );
        assert!(result.is_err());
        let unresolved = result.unwrap_err();
        assert_eq!(unresolved.len(), 1);
        assert!(unresolved[0].contains("UnknownType"));
    }

    #[test]
    fn test_register_product_checked_ok_when_all_resolved() {
        let mut registry = TypeRegistry::with_primitives();
        let result = registry
            .register_product_checked("TestRecord", vec![("name", "String"), ("flag", "Bool")]);
        assert!(result.is_ok());
    }

    #[test]
    fn ratchet_identity_types_in_core_registry() {
        let registry = TypeRegistry::with_core_types();
        let identities = registry.audit_identity_types();
        let allowed: std::collections::BTreeSet<&str> =
            ["Any", "Json", "Record", "Unit"].into_iter().collect();
        let actual: std::collections::BTreeSet<&str> =
            identities.iter().map(|t| t.0.as_str()).collect();
        let unexpected: Vec<_> = actual.difference(&allowed).collect();
        assert!(
            unexpected.is_empty(),
            "unexpected identity types (add structural DAG or update allowlist): {:?}",
            unexpected
        );
    }

    #[test]
    fn test_register_coproduct_checked_reports_unresolved() {
        let mut registry = TypeRegistry::with_primitives();
        let result = registry
            .register_coproduct_checked("TestEnum", vec![("A", "String"), ("B", "MissingType")]);
        assert!(result.is_err());
        let unresolved = result.unwrap_err();
        assert_eq!(unresolved.len(), 1);
        assert!(unresolved[0].contains("MissingType"));
    }
}
