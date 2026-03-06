//! Repo-owned makegen policy and renderers.
//!
//! This module owns gunbc-specific build target modeling, gitignore emission,
//! and tool projection. It intentionally lives in the app crate rather than
//! `core/codegen` because the data and policy here are repository-specific.

pub mod gitignore;
pub mod model;
pub mod registry;
pub mod shared;
pub mod tools;
