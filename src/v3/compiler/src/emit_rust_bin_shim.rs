//! PB-1 — Rust `main.rs` shell for generated `[[bin]]` shims (Item 5).
//!
//! **SG-0 / bounded-seed receipt:** hand-authored path is enumerated in
//! `EXPECTED_HAND_AUTHORED_NON_TEST` in `sg0_census_test.rs` (not
//! `GENERATED_FILES` / `build.rs` output). Narrow host helper for §4.2 shell
//! text only; see `docs/invariants/bounded-substrate-seed.md` (no new
//! substrate primitive — mirrors existing `std.process.ProcessExit`).
//!
//! Authority: `docs/design-pb-runtime-interpreter.md` §4.2 (`main` →
//! `ProcessExit` → `ExitCode`). This module owns **parameterized Rust text**
//! only: it does not resolve `BinShim.entry` `DeclarationRef` targets (row-#1
//! / host bridge gap — STOP there) and does not wire `cargo`/build.rs.

/// Formats the `main.rs` wrapper for a `.dag`-authored bin shim.
///
/// `source_dag_path` is emitted verbatim in the `AUTO-GENERATED` header (one
/// line). `description` is split into `//` comment lines. `entry_fn_qname` is
/// the Rust-qualified symbol for the `.dag` entry function returning
/// [`crate::process_exit::ProcessExit`].
pub fn format_bin_shim_main_rs(
    source_dag_path: &str,
    description: &str,
    entry_fn_qname: &str,
) -> String {
    let mut out = String::new();
    out.push_str("// AUTO-GENERATED from ");
    out.push_str(source_dag_path);
    out.push_str(" — DO NOT EDIT.\n//\n");
    if description.is_empty() {
        out.push_str("//\n");
    } else {
        for line in description.lines() {
            out.push_str("// ");
            out.push_str(line);
            out.push('\n');
        }
    }
    out.push_str(
        r#"
use std::io::Write;
use std::process::ExitCode;

use v3_compiler::dag::Dag;
use v3_compiler::process_exit::ProcessExit;

fn main() -> ExitCode {
    match "#,
    );
    out.push_str(entry_fn_qname);
    out.push_str(
        r#"(&Dag::new()) {
        ProcessExit::ExitSuccess => ExitCode::SUCCESS,
        ProcessExit::ExitFailure { code, reason } => {
            let _ = writeln!(std::io::stderr(), "{reason}");
            // Fail-closed: `ExitFailure` must not map to a successful host exit.
            // Non-positive or out-of-range codes remap into 1..=255 (see dsl/std/process.dag).
            ExitCode::from((code.max(1).min(255)) as u8)
        }
    }
}
"#,
    );
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_bin_shim_main_rs_includes_header_entry_and_process_exit() {
        let out = format_bin_shim_main_rs(
            "dsl/std/runtime/bin_shims/regen_lens.dag",
            "Unified lens-regen driver.",
            "std::runtime::bin_shims::regen_lens::regen_lens_main",
        );
        assert!(out.contains(
            "// AUTO-GENERATED from dsl/std/runtime/bin_shims/regen_lens.dag — DO NOT EDIT."
        ));
        assert!(out.contains("// Unified lens-regen driver."));
        assert!(out.contains("use v3_compiler::process_exit::ProcessExit;"));
        assert!(out.contains("code.max(1).min(255)"));
        assert!(!out.contains("code.clamp(0, 255)"));
        assert!(out.contains("std::runtime::bin_shims::regen_lens::regen_lens_main(&Dag::new())"));
    }

    #[test]
    fn format_bin_shim_main_rs_multiline_description() {
        let out = format_bin_shim_main_rs("path/to/shim.dag", "line1\nline2", "m::f");
        assert!(out.contains("// line1\n// line2\n"));
    }
}
