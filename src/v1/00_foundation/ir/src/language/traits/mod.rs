//! Trait SubDags: Composable characteristics for languages and formats.
//!
//! These SubDags provide reusable behavior:
//! - [`TypeSystemMapping`]: Map abstract types to language-specific types
//! - [`NamingConventions`]: Convert between naming cases
//! - [`CommentPrefix`]: Add comment syntax to content

pub mod comment;
pub mod naming;
pub mod type_system;

pub use comment::{build_add_comment_node, build_comment_prefix_subdag};
pub use naming::{
    build_naming_conventions_subdag, convert_for_language, naming_for_language, LanguageNaming,
};
pub use type_system::{build_type_system_mapping_subdag, map_type, optional_wrapper, TypeMapping};
