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
//! ```ignore
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
                .map(|f| {
                    let ty = if f.optional {
                        format!("Option<{}>", f.type_name)
                    } else {
                        f.type_name.clone()
                    };
                    (f.name.clone(), ty, true)
                })
                .collect();
            items.push(Item::Struct(StructDef {
                name: s.name.clone(),
                is_pub: true,
                derives: s.derives.clone(),
                fields,
                doc: s.doc.as_deref().map(|d| vec![d.to_string()]).unwrap_or_default(),
            }));
        }

        // Enums
        for e in &self.enums {
            items.push(Item::Enum(crate::code_ir::EnumDef {
                name: e.name.clone(),
                is_pub: true,
                derives: e.derives.clone(),
                variants: e.variants.clone(),
                doc: e.doc.as_deref().map(|d| vec![d.to_string()]).unwrap_or_default(),
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
                doc: f.doc.as_deref().map(|d| vec![d.to_string()]).unwrap_or_default(),
                attributes: vec![],
            }));
        }

        SourceFile {
            doc: self.doc.as_deref().map(|d| vec![d.to_string()]).unwrap_or_default(),
            items,
        }
    }

    /// Convert to `SourceFile` and render through a `CodeRenderer`.
    ///
    /// This is the F2.2 pipeline: `BridgeModule → SourceFile → CodeRenderer → text`.
    ///
    /// ```ignore
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

#[cfg(test)]
mod tests {
    use super::*;

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
                assert_eq!(s.fields[1], ("branch".into(), "Option<String>".into(), true));
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
}
