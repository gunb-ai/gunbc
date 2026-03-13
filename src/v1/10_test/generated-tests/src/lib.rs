//! Workspace-generated test harness.
//!
//! The crate itself is stable and tracked. Auto-generated test modules live
//! under `src/generated/` and are discovered by `build.rs`.

#[cfg(test)]
mod generated {
    include!(concat!(env!("OUT_DIR"), "/generated_tests_mods.rs"));
}
