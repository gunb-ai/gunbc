//! Trait SubDags: Composable characteristics for languages and formats.
//!
//! These SubDags provide reusable behavior:
//! - [`TypeSystemMapping`]: Map abstract types to language-specific types
//! - [`NamingConventions`]: Convert between naming cases
//! - [`CommentPrefix`]: Add comment syntax to content

pub mod comment;
mod naming;
mod type_system;

pub use comment::build_comment_prefix_subdag;
pub use naming::build_naming_conventions_subdag;
pub use type_system::build_type_system_mapping_subdag;
