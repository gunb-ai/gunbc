//! CI provider implementations.
//!
//! Each provider implements the `CiProvider` trait with platform-specific
//! formatting for workflow commands.

mod github;
mod gitlab;
mod plain;

pub use github::GitHubActionsProvider;
pub use gitlab::GitLabCiProvider;
pub use plain::PlainTextProvider;
