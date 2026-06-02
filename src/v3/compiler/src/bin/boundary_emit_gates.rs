// `eprintln!` is on the project's disallowed-macros list (library code must
// use structured error returns). Binary entrypoints may opt in per the lint
// note; this bin is a host-shim whose only behavior is exit-code + stderr
// for human debugging when a gate fails.
#![allow(clippy::disallowed_macros)]

//! Boundary emit — `ExecuteCommand` logical child for `.dag` `TestClaim`
//! wrappers (`tests/dag/boundary_emit_gates.template.dag`). Subcommands wrap
//! `v3_compiler::boundary_emit_gates::check_*` so hand-Rust `#[test]` harnesses
//! and `.dag`-driven gates share one implementation.

use std::process::ExitCode;

use v3_compiler::boundary_emit_gates;

fn usage() -> ! {
    eprintln!(
        "usage: boundary_emit_gates <subcommand>\n\
         subcommands: m2-destructure-alias | m2-arm-aliased-ref | m2-rustfmt-valid | \
         python-checked-division-roundtrip"
    );
    std::process::exit(2);
}

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let sub = args.next().unwrap_or_else(|| usage());
    if args.next().is_some() {
        usage();
    }

    let result = match sub.as_str() {
        "m2-destructure-alias" => boundary_emit_gates::check_m2_multi_field_struct_variant_destructure_alias(),
        "m2-arm-aliased-ref" => boundary_emit_gates::check_m2_multi_field_struct_variant_arm_aliased_ref(),
        "m2-rustfmt-valid" => boundary_emit_gates::check_m2_multi_field_struct_variant_rustfmt_valid(),
        "python-checked-division-roundtrip" => boundary_emit_gates::check_python_checked_division_roundtrips(),
        _ => usage(),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(detail) => {
            eprintln!("boundary_emit_gates {sub}: {detail}");
            ExitCode::FAILURE
        }
    }
}
