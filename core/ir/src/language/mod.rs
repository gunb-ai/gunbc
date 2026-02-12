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
//! ```ignore
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

// Re-exports - SubDag builders
pub use categories::{build_config_format_subdag, build_turing_complete_subdag};
pub use languages::{
    build_css_subdag, build_gitignore_subdag, build_html_subdag, build_makefile_subdag,
    build_markdown_subdag, build_rust_subdag, build_toml_subdag, build_yaml_subdag,
};
pub use patterns::{build_glob_subdag, build_regex_subdag, build_variable_syntax_subdag};
pub use traits::{
    build_comment_prefix_subdag, build_naming_conventions_subdag, build_type_system_mapping_subdag,
};

// Re-exports - Static configurations
pub use languages::{
    rust_type, CssConfig, GitignoreConfig, HtmlConfig, MakeTarget, MakefileConfig, MarkdownConfig,
    RustConfig, TomlConfig, YamlConfig, CSS, DEFAULT_GITIGNORE_FILENAME, DEFAULT_MAKEFILE_FILENAME,
    GITIGNORE, HTML, MAKEFILE, MARKDOWN, RUST, TOML, YAML,
};

// Re-exports - Rendering functions
pub use languages::{
    css_comment, html_comment, markdown_comment, render_code_block, render_html_document,
    toml_comment, yaml_comment,
};

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
    for segment in name.split(['_', '-', '/', '.']) {
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
/// └── Language SubDags: rust, gitignore, makefile, yaml, toml
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
    dag.add_node(build_gitignore_subdag());
    dag.add_node(build_makefile_subdag());
    dag.add_node(build_html_subdag());
    dag.add_node(build_css_subdag());
    dag.add_node(build_markdown_subdag());
    dag.add_node(build_yaml_subdag());
    dag.add_node(build_toml_subdag());

    dag
}

/// Detect language from filename using the Languages DAG.
///
/// This replaces the ad-hoc `detect_language()` function in lib/markdown.
pub fn detect_language_from_file(filename: &str) -> Option<&'static str> {
    // Static mapping derived from language SubDag configurations
    // In the future, this could query the actual Languages DAG
    let mappings: &[(&[&str], &str)] = &[
        (&[".rs"], "rust"),
        (&[".py"], "python"),
        (&[".js"], "javascript"),
        (&[".ts", ".tsx"], "typescript"),
        (&[".go"], "go"),
        (&[".md"], "markdown"),
        (&[".toml"], "toml"),
        (&[".json"], "json"),
        (&[".yaml", ".yml"], "yaml"),
        (&[".sh", ".bash"], "bash"),
        (&[".c", ".h"], "c"),
        (&[".cpp", ".hpp", ".cc"], "cpp"),
        (&[".java"], "java"),
        (&[".rb"], "ruby"),
        (&[".html"], "html"),
        (&[".css"], "css"),
        (&[".sql"], "sql"),
    ];

    for (extensions, lang_id) in mappings {
        if extensions.iter().any(|ext| filename.ends_with(ext)) {
            return Some(lang_id);
        }
    }

    None
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
        assert_eq!(detect_language_from_file("bar.py"), Some("python"));
        assert_eq!(detect_language_from_file("baz.ts"), Some("typescript"));
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
        assert!(dag.get_node(&"gitignore".into()).is_some());
        assert!(dag.get_node(&"makefile".into()).is_some());
    }
}
