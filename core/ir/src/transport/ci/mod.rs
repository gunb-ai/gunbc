//! CI provider abstraction for workflow commands.
//!
//! This module provides a unified interface for emitting CI-specific output
//! (log groups, annotations, outputs) across different CI providers like
//! GitHub Actions and GitLab CI.
//!
//! # Architecture
//!
//! ```text
//! WorkflowCommand (what to emit)
//!         │
//!         ▼
//! CiProvider::format() (how to emit)
//!         │
//!    ┌────┼────────────┐
//!    ▼    ▼            ▼
//! GitHub  GitLab    PlainText
//! Actions    CI     (local dev)
//! ```
//!
//! # Design Principle
//!
//! Following the `Renderable` pattern from the codebase:
//! - **Shared types**: `WorkflowCommand` defines *what* to emit
//! - **Provider-specific**: Each provider's `format()` implements *how*
//! - **Graceful degradation**: Unsupported commands get plain text fallback
//!
//! # Example
//!
//! ```text
//! use gunbc_ir::transport::ci::{detect_provider, WorkflowCommand};
//! use std::collections::HashMap;
//!
//! let env: HashMap<String, String> = std::env::vars().collect();
//! let provider = detect_provider(&env);
//!
//! // Start a collapsible group
//! println!("{}", provider.format(&WorkflowCommand::group_start("build")));
//!
//! // ... do work ...
//!
//! // End the group
//! println!("{}", provider.format(&WorkflowCommand::group_end("build")));
//!
//! // Emit an error annotation
//! println!("{}", provider.format(&WorkflowCommand::error("Test failed")));
//! ```
//!
//! # Capability Comparison
//!
//! | Feature | GitHub Actions | GitLab CI |
//! |---------|---------------|-----------|
//! | Collapsible sections | `::group::` | `section_start` escape |
//! | Inline annotations | `::error::` | Plain text (colored) |
//! | Job summaries | `$GITHUB_STEP_SUMMARY` | Not supported |
//! | Secrets masking | `::add-mask::` | CI variable masking |

pub mod command;
pub mod provider;
pub mod providers;
pub mod render;
pub mod runner;

// Re-exports for convenience
pub use command::{AnnotationLevel, FileLocation, WorkflowCommand};
pub use provider::{detect_provider, detect_provider_strict, is_ci, CiProvider};
pub use providers::{GitHubActionsProvider, GitLabCiProvider, PlainTextProvider};
pub use render::{
    dag_to_shared_steps, yaml_block, CacheConfig, CheckoutConfig, CiRenderer, RenderConfig,
    SharedStep,
};
pub use runner::{
    all_gitlab_runners, gitlab_saas_linux_large, gitlab_saas_linux_medium, gitlab_saas_linux_small,
    GitLabRunner, Runner,
};
