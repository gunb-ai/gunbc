//! Trait SubDags: Composable characteristics for languages and formats.
//!
//! These SubDags provide reusable behavior:
//! - [`TypeSystemMapping`]: Map abstract types to language-specific types
//! - [`NamingConventions`]: Convert between naming cases
//! - [`CommentPrefix`]: Add comment syntax to content

mod type_system;
mod naming;
pub mod comment;

pub use type_system::build_type_system_mapping_subdag;
pub use naming::build_naming_conventions_subdag;
pub use comment::build_comment_prefix_subdag;
