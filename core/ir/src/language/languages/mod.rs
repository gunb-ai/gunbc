//! Language SubDags: Concrete language and format definitions.
//!
//! Each language/format is a SubDag that composes category and trait SubDags:
//! - [`Rust`]: Programming language (composes TuringComplete)
//! - [`Gitignore`]: File format (composes ConfigFormat, GlobPatterns, Regex)
//! - [`Makefile`]: Build format (composes ConfigFormat, VariableSyntax)
//! - [`Html`]: Markup language (composes ConfigFormat)
//! - [`Css`]: Stylesheet language (composes ConfigFormat)
//! - [`Markdown`]: Document format (composes ConfigFormat)
//! - [`Yaml`]: Configuration format (composes ConfigFormat)
//! - [`Toml`]: Configuration format (composes ConfigFormat)

mod css;
mod gitignore;
mod html;
mod makefile;
mod markdown;
mod rust;
mod toml;
mod yaml;

pub use css::{build_css_subdag, css_comment, CssConfig, CSS};
pub use gitignore::{
    build_gitignore_subdag, GitignoreConfig, DEFAULT_GITIGNORE_FILENAME, GITIGNORE,
};
pub use html::{build_html_subdag, html_comment, render_html_document, HtmlConfig, HTML};
pub use makefile::{
    build_makefile_subdag, MakefileConfig, MakeTarget, DEFAULT_MAKEFILE_FILENAME, MAKEFILE,
};
pub use markdown::{
    build_markdown_subdag, markdown_comment, render_code_block, MarkdownConfig, MARKDOWN,
};
pub use rust::{build_rust_subdag, rust_type, RustConfig, RUST};
pub use toml::{build_toml_subdag, toml_comment, TomlConfig, TOML};
pub use yaml::{build_yaml_subdag, yaml_comment, YamlConfig, YAML};
