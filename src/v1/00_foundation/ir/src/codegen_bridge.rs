//! Codegen bridge: Convert the-gunbai's IR to gunbc's SourceFile.
//!
//! the-gunbai generates code through a pipeline:
//!
//! ```text
//! Understanding → Block → gunbai IR (Module/StructDef/FieldDef/TypeExpr) → LanguageBackend → text
//! ```
//!
//! This module provides conversion from JSON-serialized gunbai IR to gunbc's
//! `code_ir::SourceFile`, enabling both repos to share the same rendering layer.
//!
//! # Mapping
//!
//! | gunbai IR | gunbc code_ir |
//! |---|---|
//! | `Module` | `SourceFile` |
//! | `Module.imports` | `SourceFile.items[Item::Use]` |
//! | `Item::Struct(StructDef)` | `Item::Struct(StructDef)` |
//! | `Item::Enum(EnumDef)` | `Item::Enum(EnumDef)` |
//! | `Item::Function(FunctionDef)` | `Item::Fn(FnDef)` |
//! | `FieldDef.name + TypeExpr` | `(field_name, type_string, is_pub)` |
//!
//! # Rendering Pipeline (F2.2)
//!
//! ```text
//! BridgeModule → to_source_file() → SourceFile → CodeRenderer<M> → text
//!                                                 ├── RustCodeRenderer     → .rs
//!                                                 ├── PythonCodeRenderer   → .py
//!                                                 └── TypeScriptCodeRenderer → .ts
//! ```
//!
//! Use `render_with()` for one-step conversion through any `CodeRenderer`.
//!
//! # Usage
//!
//! ```text
//! use gunbc_ir::codegen_bridge::{BridgeModule, BridgeField, BridgeStruct};
//!
//! let module = BridgeModule {
//!     name: "git_io".to_string(),
//!     structs: vec![BridgeStruct {
//!         name: "GitSpec".to_string(),
//!         fields: vec![BridgeField::new("repo_url", "String")],
//!         derives: vec!["Debug".into(), "Clone".into()],
//!     }],
//!     ..Default::default()
//! };
//! let source_file = module.to_source_file();
//! ```

use crate::code_ir::{Expr, FnDef, Import, Item, SourceFile, Stmt, StructDef};
use crate::render_ir::{CodeRenderer, TextMedium};
use crate::types::{normalize_optional_type_id, optional_inner_type_id};

/// A module from the-gunbai's codegen IR, ready for conversion to SourceFile.
///
/// This is a simplified bridge type. For full fidelity, serialize the-gunbai's
/// `Module` to JSON and deserialize as `BridgeModule`.
#[derive(Debug, Clone, Default)]
pub struct BridgeModule {
    /// Module name (e.g., "git_io").
    pub name: String,
    /// Doc comment for the module.
    pub doc: Option<String>,
    /// Import paths (e.g., `("serde", ["Serialize", "Deserialize"])`).
    pub imports: Vec<(String, Vec<String>)>,
    /// Struct definitions.
    pub structs: Vec<BridgeStruct>,
    /// Enum definitions.
    pub enums: Vec<BridgeEnum>,
    /// Function definitions.
    pub functions: Vec<BridgeFunction>,
}

/// A struct definition from the-gunbai's codegen IR.
#[derive(Debug, Clone)]
pub struct BridgeStruct {
    /// Struct name (PascalCase).
    pub name: String,
    /// Doc comment.
    pub doc: Option<String>,
    /// Fields.
    pub fields: Vec<BridgeField>,
    /// Derive macros (e.g., `["Debug", "Clone", "Serialize"]`).
    pub derives: Vec<String>,
}

/// A field from the-gunbai's codegen IR.
#[derive(Debug, Clone)]
pub struct BridgeField {
    /// Field name (will use language-appropriate casing).
    pub name: String,
    /// Abstract type name (e.g., "String", "Int", "List<String>").
    pub type_name: String,
    /// Whether this field is optional.
    pub optional: bool,
    /// Doc comment.
    pub doc: Option<String>,
}

impl BridgeField {
    /// Create a required field.
    pub fn new(name: impl Into<String>, type_name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            type_name: type_name.into(),
            optional: false,
            doc: None,
        }
    }

    /// Create an optional field.
    pub fn optional(name: impl Into<String>, type_name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            type_name: type_name.into(),
            optional: true,
            doc: None,
        }
    }
}

/// An enum definition from the-gunbai's codegen IR.
#[derive(Debug, Clone)]
pub struct BridgeEnum {
    /// Enum name (PascalCase).
    pub name: String,
    /// Doc comment.
    pub doc: Option<String>,
    /// Variant names.
    pub variants: Vec<String>,
    /// Derive macros.
    pub derives: Vec<String>,
}

/// A function definition from the-gunbai's codegen IR.
#[derive(Debug, Clone)]
pub struct BridgeFunction {
    /// Function name.
    pub name: String,
    /// Doc comment.
    pub doc: Option<String>,
    /// Parameters: `(name, type_name)`.
    pub params: Vec<(String, String)>,
    /// Return type (None = void/unit).
    pub return_type: Option<String>,
    /// Raw body code (escape hatch for v0).
    pub body: Option<String>,
}

impl BridgeModule {
    /// Convert to a gunbc `SourceFile`.
    pub fn to_source_file(&self) -> SourceFile {
        let mut items = Vec::new();

        // Imports
        for (path, import_items) in &self.imports {
            items.push(Item::Use(Import {
                path: path.split("::").map(String::from).collect(),
                items: import_items.clone(),
            }));
        }

        // Structs
        for s in &self.structs {
            let fields: Vec<(String, String, bool)> = s
                .fields
                .iter()
                .map(|f| (f.name.clone(), bridge_field_type_name(f), true))
                .collect();
            items.push(Item::Struct(StructDef {
                name: s.name.clone(),
                is_pub: true,
                derives: s.derives.clone(),
                fields,
                doc: s
                    .doc
                    .as_deref()
                    .map(|d| vec![d.to_string()])
                    .unwrap_or_default(),
            }));
        }

        // Enums
        for e in &self.enums {
            items.push(Item::Enum(crate::code_ir::EnumDef {
                name: e.name.clone(),
                is_pub: true,
                derives: e.derives.clone(),
                variants: e.variants.clone(),
                doc: e
                    .doc
                    .as_deref()
                    .map(|d| vec![d.to_string()])
                    .unwrap_or_default(),
            }));
        }

        // Functions
        for f in &self.functions {
            let params: Vec<(String, String)> = f
                .params
                .iter()
                .map(|(name, ty)| (name.clone(), ty.clone()))
                .collect();
            let body: Vec<Stmt> = if let Some(raw) = &f.body {
                vec![Stmt::Expr(Expr::RawCode(raw.clone()))]
            } else {
                vec![]
            };
            items.push(Item::Fn(FnDef {
                name: f.name.clone(),
                is_pub: true,
                params,
                return_type: f.return_type.clone(),
                body,
                doc: f
                    .doc
                    .as_deref()
                    .map(|d| vec![d.to_string()])
                    .unwrap_or_default(),
                attributes: vec![],
            }));
        }

        SourceFile {
            doc: self
                .doc
                .as_deref()
                .map(|d| vec![d.to_string()])
                .unwrap_or_default(),
            items,
        }
    }

    /// Convert to `SourceFile` and render through a `CodeRenderer`.
    ///
    /// This is the F2.2 pipeline: `BridgeModule → SourceFile → CodeRenderer → text`.
    ///
    /// ```text
    /// use gunbc_codegen::testgen::render_rust::RustCodeRenderer;
    /// use gunbc_ir::render_ir::PlainText;
    ///
    /// let renderer = RustCodeRenderer::new(PlainText);
    /// let output: String = module.render_with(&renderer);
    /// ```
    pub fn render_with<M: TextMedium>(&self, renderer: &impl CodeRenderer<M>) -> M::Output {
        let sf = self.to_source_file();
        renderer.render_source_file(&sf)
    }
}

fn bridge_field_type_name(field: &BridgeField) -> String {
    match (field.optional, optional_inner_type_id(&field.type_name)) {
        (true, Some(_)) | (false, Some(_)) => normalize_optional_type_id(&field.type_name),
        (true, None) => format!("Optional<{}>", field.type_name),
        (false, None) => field.type_name.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::code_ir::{Assert, Expr, Import, Item, SourceFile, Stmt, TestFile};
    use crate::language::NamingCase;
    use crate::render_ir::{CodeRenderer, PlainText};
    use crate::symbols::{Tier, STANDARD};

    struct NamingCaseRenderer {
        medium: PlainText,
        type_case: NamingCase,
        field_case: NamingCase,
    }

    impl NamingCaseRenderer {
        fn new(type_case: NamingCase, field_case: NamingCase) -> Self {
            Self {
                medium: PlainText {
                    tier: Tier::Ascii,
                    symbol_set: &STANDARD,
                },
                type_case,
                field_case,
            }
        }
    }

    impl CodeRenderer<PlainText> for NamingCaseRenderer {
        fn medium(&self) -> &PlainText {
            &self.medium
        }

        fn render_value(&self, _expr: &crate::ValueExpr) -> String {
            String::new()
        }

        fn render_file(&self, _file: &TestFile) -> String {
            String::new()
        }

        fn render_source_file(&self, file: &SourceFile) -> String {
            let mut rendered = Vec::new();
            for item in &file.items {
                if let Item::Struct(def) = item {
                    let type_name = self.type_case.apply(&def.name);
                    let fields = def
                        .fields
                        .iter()
                        .map(|(name, _, _)| self.field_case.apply(name))
                        .collect::<Vec<_>>()
                        .join(", ");
                    rendered.push(format!("struct {type_name} {{{fields}}}"));
                }
            }
            rendered.join("\n")
        }

        fn render_expr(&self, _expr: &Expr) -> String {
            String::new()
        }

        fn render_stmt(&self, _stmt: &Stmt, _indent: usize) -> String {
            String::new()
        }

        fn render_assert(&self, _assert: &Assert, _indent: usize) -> String {
            String::new()
        }

        fn render_import(&self, _import: &Import) -> String {
            String::new()
        }

        fn render_item(&self, _item: &Item, _indent: usize) -> String {
            String::new()
        }
    }

    #[test]
    fn empty_module() {
        let module = BridgeModule {
            name: "empty".into(),
            ..Default::default()
        };
        let sf = module.to_source_file();
        assert!(sf.doc.is_empty());
        assert!(sf.items.is_empty());
    }

    #[test]
    fn module_with_struct() {
        let module = BridgeModule {
            name: "git_io".into(),
            structs: vec![BridgeStruct {
                name: "GitSpec".into(),
                doc: Some("Git repository specification.".into()),
                fields: vec![
                    BridgeField::new("repo_url", "String"),
                    BridgeField::optional("branch", "String"),
                ],
                derives: vec!["Debug".into(), "Clone".into()],
            }],
            ..Default::default()
        };
        let sf = module.to_source_file();
        assert_eq!(sf.items.len(), 1);
        match &sf.items[0] {
            Item::Struct(s) => {
                assert_eq!(s.name, "GitSpec");
                assert_eq!(s.fields.len(), 2);
                assert_eq!(s.fields[0], ("repo_url".into(), "String".into(), true));
                assert_eq!(
                    s.fields[1],
                    ("branch".into(), "Optional<String>".into(), true)
                );
                assert_eq!(s.derives, vec!["Debug", "Clone"]);
            }
            _ => panic!("expected struct"),
        }
    }

    #[test]
    fn module_with_import_and_function() {
        let module = BridgeModule {
            name: "handlers".into(),
            imports: vec![("serde".into(), vec!["Serialize".into()])],
            functions: vec![BridgeFunction {
                name: "handle_request".into(),
                doc: None,
                params: vec![("input".into(), "String".into())],
                return_type: Some("String".into()),
                body: Some("input.to_uppercase()".into()),
            }],
            ..Default::default()
        };
        let sf = module.to_source_file();
        assert_eq!(sf.items.len(), 2); // import + function
    }

    #[test]
    fn module_with_enum() {
        let module = BridgeModule {
            name: "types".into(),
            enums: vec![BridgeEnum {
                name: "Status".into(),
                doc: None,
                variants: vec!["Active".into(), "Inactive".into()],
                derives: vec!["Debug".into()],
            }],
            ..Default::default()
        };
        let sf = module.to_source_file();
        match &sf.items[0] {
            Item::Enum(e) => {
                assert_eq!(e.name, "Status");
                assert_eq!(e.variants, vec!["Active", "Inactive"]);
            }
            _ => panic!("expected enum"),
        }
    }

    #[test]
    fn round_trip_type_mapping() {
        let field = BridgeField::new("items", "List<String>");
        assert_eq!(field.type_name, "List<String>");

        let opt_field = BridgeField::optional("tag", "String");
        assert!(opt_field.optional);
    }

    #[test]
    fn optional_field_does_not_double_wrap_optional_type_names() {
        let module = BridgeModule {
            name: "git_io".into(),
            structs: vec![BridgeStruct {
                name: "GitSpec".into(),
                doc: None,
                fields: vec![
                    BridgeField::optional("branch", "Optional<String>"),
                    BridgeField {
                        name: "tag".into(),
                        type_name: "String?".into(),
                        optional: false,
                        doc: None,
                    },
                ],
                derives: vec![],
            }],
            ..Default::default()
        };

        let sf = module.to_source_file();
        let Item::Struct(def) = &sf.items[0] else {
            panic!("expected struct");
        };

        assert_eq!(def.fields[0].1, "Optional<String>");
        assert_eq!(def.fields[1].1, "Optional<String>");
    }

    #[test]
    fn bridge_preserves_names_and_renderer_applies_casing() {
        let module = BridgeModule {
            name: "git_io_module".into(),
            structs: vec![BridgeStruct {
                name: "git_repo_spec".into(),
                doc: None,
                fields: vec![
                    BridgeField::new("repo_url", "String"),
                    BridgeField::new("created_at_unix", "Int"),
                ],
                derives: vec![],
            }],
            ..Default::default()
        };

        let sf = module.to_source_file();
        let Item::Struct(def) = &sf.items[0] else {
            panic!("expected struct");
        };
        assert_eq!(def.name, "git_repo_spec");
        assert_eq!(def.fields[0].0, "repo_url");
        assert_eq!(def.fields[1].0, "created_at_unix");

        let rust_renderer = NamingCaseRenderer::new(NamingCase::PascalCase, NamingCase::SnakeCase);
        let ts_renderer = NamingCaseRenderer::new(NamingCase::PascalCase, NamingCase::CamelCase);

        let rust_rendered = module.render_with(&rust_renderer);
        let ts_rendered = module.render_with(&ts_renderer);

        assert!(rust_rendered.contains("struct GitRepoSpec {repo_url, created_at_unix}"));
        assert!(ts_rendered.contains("struct GitRepoSpec {repoUrl, createdAtUnix}"));
    }
}
