//! Languages DAG: Fractal composition of language and format characteristics.
//!
//! This module provides a unified DAG for language/format definitions using the
//! fractal DAG pattern. Each language (Rust, Python) and format (Makefile, gitignore)
//! is a SubDag node that composes trait SubDags (TypeSystemMapping, NamingConventions).
//!
//! # Fractal DAG Pattern
//!
//! Every language/format/pattern is a `Node::subdag(...)` with explicit I/O contracts:
//!
//! ```text
//! Languages DAG
//! ├── Pattern SubDags (foundations)
//! │   ├── Regex         - Pattern matching
//! │   ├── GlobPatterns  - File matching (composes Regex)
//! │   └── VariableSyntax - Template expansion
//! ├── Trait SubDags (composable characteristics)
//! │   ├── TypeSystemMapping  - Abstract → concrete types
//! │   ├── NamingConventions  - Case conversion
//! │   └── CommentPrefix      - Comment syntax
//! ├── Category SubDags
//! │   ├── TuringComplete - Programming languages
//! │   └── ConfigFormat   - Configuration files
//! └── Language SubDags
//!     ├── Rust      (composes TuringComplete)
//!     ├── Gitignore (composes ConfigFormat, GlobPatterns, Regex)
//!     └── Makefile  (composes ConfigFormat, VariableSyntax)
//! ```
//!
//! # Example
//!
//! ```text
//! use gunbc_ir::language::{build_languages_dag, LanguageOp};
//!
//! let languages = build_languages_dag();
//! let rust_node = languages.get_node(&"rust".into()).unwrap();
//! ```

pub mod categories;
pub mod languages;
pub mod patterns;
pub mod traits;

use crate::dag::Dag;
use std::path::Path;

// Re-exports - SubDag builders
pub use categories::{build_config_format_subdag, build_turing_complete_subdag};
pub use languages::{
    build_css_subdag, build_gitignore_subdag, build_html_subdag, build_makefile_subdag,
    build_markdown_subdag, build_python_subdag, build_rust_subdag, build_toml_subdag,
    build_typescript_subdag, build_yaml_subdag,
};
pub use patterns::{build_glob_subdag, build_regex_subdag, build_variable_syntax_subdag};
pub use traits::{
    build_add_comment_node, build_comment_prefix_subdag, build_naming_conventions_subdag,
    build_type_system_mapping_subdag,
    convert_for_language, map_type, naming_for_language, optional_wrapper, LanguageNaming,
    TypeMapping,
};

// Re-exports - Static configurations
pub use languages::{
    rust_type, CssConfig, GitignoreConfig, HtmlConfig, MakeTarget, MakefileConfig, MarkdownConfig,
    PythonConfig, RustConfig, TomlConfig, TypeScriptConfig, YamlConfig, CSS,
    DEFAULT_GITIGNORE_FILENAME, DEFAULT_MAKEFILE_FILENAME, GITIGNORE, HTML, MAKEFILE, MARKDOWN,
    PYTHON, PYTHON_NAMING, PYTHON_TYPES, RUST, RUST_NAMING, RUST_TYPES, TOML, TYPESCRIPT,
    TYPESCRIPT_NAMING, TYPESCRIPT_TYPES, YAML,
};

// Re-exports - Rendering functions
pub use languages::{
    css_comment, html_comment, markdown_comment, render_code_block, render_html_document,
    toml_comment, yaml_comment,
};
use traits::comment::CommentSyntax;

/// Operations within the Languages DAG.
///
/// These operations define the behavior of nodes within language/format SubDags.
#[derive(Debug, Clone)]
pub enum LanguageOp {
    // === Pattern Operations ===
    /// Validate and match regex patterns
    RegexValidate,
    /// Match regex against text
    RegexMatch,
    /// Match glob patterns against file list
    GlobMatch,
    /// Expand variable syntax (e.g., $(VAR), ${VAR})
    ExpandVariables,

    // === Trait Operations ===
    /// Map abstract type to language-specific type
    MapType,
    /// Convert naming case (snake_case, PascalCase, etc.)
    ConvertCase,
    /// Add comment prefix to content
    AddComment,

    // === Category Markers ===
    /// TuringComplete category configuration
    TuringCompleteConfig,
    /// ConfigFormat category configuration
    ConfigFormatConfig,

    // === Language-Specific ===
    /// Rust language configuration
    RustConfig,
    /// Rust type mapping
    RustTypeMap,
    /// Gitignore format configuration
    GitignoreConfig,
    /// Gitignore pattern rendering
    GitignoreRender,
    /// Makefile format configuration
    MakefileConfig,
    /// Makefile target rendering
    MakefileRender,
    /// YAML format configuration
    YamlConfig,
    /// TOML format configuration
    TomlConfig,
    /// HTML format configuration
    HtmlConfig,
    /// HTML document rendering
    HtmlRender,
    /// CSS format configuration
    CssConfig,
    /// Markdown format configuration
    MarkdownConfig,
    /// Markdown code block rendering
    MarkdownRenderCodeBlock,
    /// Python language configuration
    PythonConfig,
    /// TypeScript language configuration
    TypeScriptConfig,
}

/// Naming case conventions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NamingCase {
    /// lowercase_with_underscores
    SnakeCase,
    /// UPPERCASE_WITH_UNDERSCORES
    ScreamingSnakeCase,
    /// PascalCase (UpperCamelCase)
    PascalCase,
    /// camelCase (lowerCamelCase)
    CamelCase,
    /// lowercase (no separator)
    Lowercase,
    /// kebab-case (for file names)
    KebabCase,
}

impl NamingCase {
    /// Convert a name to this case convention.
    pub fn apply(&self, name: &str) -> String {
        let words = split_into_words(name);

        match self {
            NamingCase::SnakeCase => words.join("_").to_lowercase(),
            NamingCase::ScreamingSnakeCase => words.join("_").to_uppercase(),
            NamingCase::PascalCase => words
                .iter()
                .map(|w| {
                    let mut chars = w.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(c) => c
                            .to_uppercase()
                            .chain(chars.flat_map(|c| c.to_lowercase()))
                            .collect(),
                    }
                })
                .collect(),
            NamingCase::CamelCase => {
                let mut result = String::new();
                for (i, word) in words.iter().enumerate() {
                    if i == 0 {
                        result.push_str(&word.to_lowercase());
                    } else {
                        let mut chars = word.chars();
                        if let Some(c) = chars.next() {
                            result.push(c.to_ascii_uppercase());
                            result.extend(chars.flat_map(|c| c.to_lowercase()));
                        }
                    }
                }
                result
            }
            NamingCase::Lowercase => words.join("").to_lowercase(),
            NamingCase::KebabCase => words.join("-").to_lowercase(),
        }
    }
}

/// Split a name into words by separators and case boundaries.
fn split_into_words(name: &str) -> Vec<String> {
    let mut words = Vec::new();
    for segment in name.split(['_', '-', '/', '.', ':']) {
        if segment.is_empty() {
            continue;
        }
        let mut current = String::new();
        let mut prev_lower = false;
        for ch in segment.chars() {
            if ch.is_uppercase() && !current.is_empty() && prev_lower {
                words.push(current);
                current = String::new();
            }
            current.push(ch);
            prev_lower = ch.is_lowercase();
        }
        if !current.is_empty() {
            words.push(current);
        }
    }
    words
}

/// Build the Languages fractal DAG.
///
/// This creates the parent DAG containing all language, format, and pattern SubDags.
/// The DAG follows the fractal pattern where SubDags compose other SubDags.
///
/// # Structure
///
/// ```text
/// Languages (Dag<LanguageOp>)
/// ├── Pattern SubDags: regex, glob, variable_syntax
/// ├── Trait SubDags: type_system, naming, comment_prefix
/// ├── Category SubDags: turing_complete, config_format
/// └── Language SubDags: rust, python, typescript, gitignore, makefile, yaml, toml
/// ```
pub fn build_languages_dag() -> Dag<LanguageOp> {
    let mut dag = Dag::new();

    // Pattern SubDags (foundations - other SubDags compose these)
    dag.add_node(build_regex_subdag());
    dag.add_node(build_glob_subdag());
    dag.add_node(build_variable_syntax_subdag());

    // Trait SubDags (composable characteristics)
    dag.add_node(build_type_system_mapping_subdag());
    dag.add_node(build_naming_conventions_subdag());
    dag.add_node(build_comment_prefix_subdag());

    // Category SubDags
    dag.add_node(build_turing_complete_subdag());
    dag.add_node(build_config_format_subdag());

    // Language/Format SubDags
    dag.add_node(build_rust_subdag());
    dag.add_node(build_python_subdag());
    dag.add_node(build_typescript_subdag());
    dag.add_node(build_gitignore_subdag());
    dag.add_node(build_makefile_subdag());
    dag.add_node(build_html_subdag());
    dag.add_node(build_css_subdag());
    dag.add_node(build_markdown_subdag());
    dag.add_node(build_yaml_subdag());
    dag.add_node(build_toml_subdag());

    dag
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct LanguageMetadata {
    pub id: &'static str,
    pub file_extensions: &'static [&'static str],
    pub file_patterns: &'static [&'static str],
    pub comment_syntax: Option<CommentSyntax>,
    pub type_mapping: Option<&'static TypeMapping>,
    pub naming: Option<&'static LanguageNaming>,
}

const LANGUAGE_METADATA_REGISTRY: &[LanguageMetadata] = &[
    LanguageMetadata {
        id: RUST.id,
        file_extensions: RUST.file_extensions,
        file_patterns: &[],
        comment_syntax: Some(CommentSyntax {
            line_prefix: Some(RUST.comment_prefix),
            block_start: Some(RUST.block_comment_open),
            block_end: Some(RUST.block_comment_close),
            doc_prefix: Some(RUST.doc_comment_prefix),
        }),
        type_mapping: Some(&RUST_TYPES),
        naming: Some(&RUST_NAMING),
    },
    LanguageMetadata {
        id: PYTHON.id,
        file_extensions: PYTHON.file_extensions,
        file_patterns: &[],
        comment_syntax: Some(CommentSyntax {
            line_prefix: Some(PYTHON.comment_prefix),
            block_start: None,
            block_end: None,
            doc_prefix: None,
        }),
        type_mapping: Some(&PYTHON_TYPES),
        naming: Some(&PYTHON_NAMING),
    },
    LanguageMetadata {
        id: TYPESCRIPT.id,
        file_extensions: TYPESCRIPT.file_extensions,
        file_patterns: &[],
        comment_syntax: Some(CommentSyntax {
            line_prefix: Some(TYPESCRIPT.comment_prefix),
            block_start: Some(TYPESCRIPT.block_comment_open),
            block_end: Some(TYPESCRIPT.block_comment_close),
            doc_prefix: None,
        }),
        type_mapping: Some(&TYPESCRIPT_TYPES),
        naming: Some(&TYPESCRIPT_NAMING),
    },
    LanguageMetadata {
        id: GITIGNORE.id,
        file_extensions: &[],
        file_patterns: GITIGNORE.file_patterns,
        comment_syntax: Some(CommentSyntax {
            line_prefix: Some(GITIGNORE.comment_prefix),
            block_start: None,
            block_end: None,
            doc_prefix: None,
        }),
        type_mapping: None,
        naming: None,
    },
    LanguageMetadata {
        id: MAKEFILE.id,
        file_extensions: &[],
        file_patterns: MAKEFILE.file_patterns,
        comment_syntax: Some(CommentSyntax {
            line_prefix: Some(MAKEFILE.comment_prefix),
            block_start: None,
            block_end: None,
            doc_prefix: None,
        }),
        type_mapping: None,
        naming: None,
    },
    LanguageMetadata {
        id: HTML.id,
        file_extensions: HTML.file_extensions,
        file_patterns: &[],
        comment_syntax: Some(CommentSyntax {
            line_prefix: None,
            block_start: Some(HTML.comment_open),
            block_end: Some(HTML.comment_close),
            doc_prefix: None,
        }),
        type_mapping: None,
        naming: None,
    },
    LanguageMetadata {
        id: CSS.id,
        file_extensions: CSS.file_extensions,
        file_patterns: &[],
        comment_syntax: Some(CommentSyntax {
            line_prefix: None,
            block_start: Some(CSS.comment_open),
            block_end: Some(CSS.comment_close),
            doc_prefix: None,
        }),
        type_mapping: None,
        naming: None,
    },
    LanguageMetadata {
        id: MARKDOWN.id,
        file_extensions: MARKDOWN.file_extensions,
        file_patterns: &[],
        comment_syntax: Some(CommentSyntax {
            line_prefix: None,
            block_start: Some(MARKDOWN.comment_open),
            block_end: Some(MARKDOWN.comment_close),
            doc_prefix: None,
        }),
        type_mapping: None,
        naming: None,
    },
    LanguageMetadata {
        id: YAML.id,
        file_extensions: YAML.file_extensions,
        file_patterns: &[],
        comment_syntax: Some(CommentSyntax {
            line_prefix: Some(YAML.comment_prefix),
            block_start: None,
            block_end: None,
            doc_prefix: None,
        }),
        type_mapping: None,
        naming: None,
    },
    LanguageMetadata {
        id: TOML.id,
        file_extensions: TOML.file_extensions,
        file_patterns: &[],
        comment_syntax: Some(CommentSyntax {
            line_prefix: Some(TOML.comment_prefix),
            block_start: None,
            block_end: None,
            doc_prefix: None,
        }),
        type_mapping: None,
        naming: None,
    },
];

pub(crate) fn language_metadata_for(id: &str) -> Option<&'static LanguageMetadata> {
    let resolved = match id {
        "javascript" => "typescript",
        other => other,
    };
    LANGUAGE_METADATA_REGISTRY
        .iter()
        .find(|metadata| metadata.id == resolved)
}

fn filename_matches_pattern(filename: &str, pattern: &str) -> bool {
    match pattern.strip_prefix('*') {
        Some(suffix) => filename.ends_with(suffix),
        None => filename == pattern,
    }
}

fn language_metadata_for_file(filename: &str) -> Option<&'static LanguageMetadata> {
    let basename = Path::new(filename)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(filename);

    LANGUAGE_METADATA_REGISTRY.iter().find(|metadata| {
        metadata
            .file_extensions
            .iter()
            .any(|ext| filename.ends_with(ext))
            || metadata
                .file_patterns
                .iter()
                .any(|pattern| filename_matches_pattern(basename, pattern))
    })
}

/// Detect language from filename using the Languages DAG.
///
/// This replaces the ad-hoc `detect_language()` function in lib/markdown.
pub fn detect_language_from_file(filename: &str) -> Option<&'static str> {
    language_metadata_for_file(filename).map(|metadata| metadata.id)
}

/// Get markdown code fence identifier for a file.
///
/// Returns the language ID for use in markdown fenced code blocks.
/// Returns empty string if language is not recognized.
pub fn markdown_language_id(filename: &str) -> &'static str {
    detect_language_from_file(filename).unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_naming_case_snake() {
        assert_eq!(
            NamingCase::SnakeCase.apply("myFunctionName"),
            "my_function_name"
        );
        assert_eq!(
            NamingCase::SnakeCase.apply("my_function_name"),
            "my_function_name"
        );
        assert_eq!(NamingCase::SnakeCase.apply("MyClassName"), "my_class_name");
        assert_eq!(
            NamingCase::SnakeCase.apply("tools_bootstrap::render_makefile"),
            "tools_bootstrap_render_makefile"
        );
    }

    #[test]
    fn test_naming_case_pascal() {
        assert_eq!(
            NamingCase::PascalCase.apply("my_function_name"),
            "MyFunctionName"
        );
        assert_eq!(NamingCase::PascalCase.apply("my-component"), "MyComponent");
    }

    #[test]
    fn test_naming_case_camel() {
        assert_eq!(
            NamingCase::CamelCase.apply("my_function_name"),
            "myFunctionName"
        );
        assert_eq!(NamingCase::CamelCase.apply("my-component"), "myComponent");
    }

    #[test]
    fn test_naming_case_screaming() {
        assert_eq!(
            NamingCase::ScreamingSnakeCase.apply("max_value"),
            "MAX_VALUE"
        );
    }

    #[test]
    fn test_detect_language_from_file() {
        assert_eq!(detect_language_from_file("foo.rs"), Some("rust"));
        assert_eq!(detect_language_from_file("config/bar.yml"), Some("yaml"));
        assert_eq!(
            detect_language_from_file("docs/readme.markdown"),
            Some("markdown")
        );
        assert_eq!(detect_language_from_file("Makefile"), Some("makefile"));
        assert_eq!(
            detect_language_from_file("build/tools.mk"),
            Some("makefile")
        );
        assert_eq!(detect_language_from_file(".gitignore"), Some("gitignore"));
        assert_eq!(detect_language_from_file("unknown.xyz"), None);
    }

    #[test]
    fn test_markdown_language_id() {
        assert_eq!(markdown_language_id("foo.rs"), "rust");
        assert_eq!(markdown_language_id("unknown.xyz"), "");
    }

    #[test]
    fn test_build_languages_dag() {
        let dag = build_languages_dag();

        // Should have all the SubDag nodes
        assert!(dag.get_node(&"regex".into()).is_some());
        assert!(dag.get_node(&"glob".into()).is_some());
        assert!(dag.get_node(&"variable_syntax".into()).is_some());
        assert!(dag.get_node(&"type_system".into()).is_some());
        assert!(dag.get_node(&"naming".into()).is_some());
        assert!(dag.get_node(&"comment_prefix".into()).is_some());
        assert!(dag.get_node(&"turing_complete".into()).is_some());
        assert!(dag.get_node(&"config_format".into()).is_some());
        assert!(dag.get_node(&"rust".into()).is_some());
        assert!(dag.get_node(&"python".into()).is_some());
        assert!(dag.get_node(&"typescript".into()).is_some());
        assert!(dag.get_node(&"gitignore".into()).is_some());
        assert!(dag.get_node(&"makefile".into()).is_some());
    }
}
