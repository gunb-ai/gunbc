//! Gitignore rendering — delegates to gunbc_codegen::makegen::gitignore.

pub use gunbc_codegen::makegen::gitignore::{
    derive_categories, render_gitignore, render_gitignore_content, GitignoreRenderer,
};
