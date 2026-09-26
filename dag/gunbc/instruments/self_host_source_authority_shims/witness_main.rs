use std::rc::Rc;

use v1_compiled::v2_compiler_source_authority as emitted;
use v1_compiled::v2_compiler_source_authority::SourceRootCoverage;

// NO SEED ORACLE, ON PURPOSE. This driver used to compare the prose constant
// source_authority_module_note against a hand copy in v1_compiler, which said nothing about what
// source_authority does. Its expected verdicts are read off the authority instead:
// src/v2/compiler/source_authority.dag carries two completeness questions over one
// SourceRootCoverage -- source_root_coverage_is_complete (the INLINE carrier) and
// source_root_ref_transport_coverage_admits (the CLOSURE-REF carrier) -- and they differ on exactly
// one variant, SourceRootManifestElided, which the ref carrier admits and the inline carrier does
// not. The expected table below is that .dag's two match expressions, row for row.

fn rows() -> Vec<(&'static str, SourceRootCoverage, bool, bool)> {
    vec![
        ("complete", SourceRootCoverage::SourceRootCoverageComplete, true, true),
        (
            "manifest_elided",
            SourceRootCoverage::SourceRootManifestElided {
                read_count: 65,
                cap: 64,
            },
            false,
            true,
        ),
        (
            "rows_missing",
            SourceRootCoverage::SourceRootRowsMissing {
                read_count: 3,
                produced_row_count: 2,
            },
            false,
            false,
        ),
        ("manifest_absent", SourceRootCoverage::SourceRootManifestAbsent, false, false),
    ]
}

// The planted fault expects the inline carrier to admit the elided manifest -- the one cell where
// the carriers disagree -- so the fault run goes red only if emitted source_authority really keeps
// the two questions apart.
fn main() {
    let inject_fault = std::env::args().any(|a| a == "--inject-fault");
    let mut all_pass = true;
    for (label, coverage, inline_expected, ref_expected) in rows() {
        let inline_expected = inline_expected || (inject_fault && label == "manifest_elided");
        let coverage = Rc::new(coverage);
        let inline = emitted::source_root_coverage_is_complete(coverage.clone());
        let by_ref = emitted::source_root_ref_transport_coverage_admits(coverage);
        let ok = inline == inline_expected && by_ref == ref_expected;
        println!("{label} inline={inline} ref={by_ref} ok={ok}");
        all_pass &= ok;
    }
    if all_pass {
        println!("SELF_HOST_SOURCE_AUTHORITY_BEHAVIORAL_RECEIPT: PASS");
        std::process::exit(0);
    }
    println!("SELF_HOST_SOURCE_AUTHORITY_BEHAVIORAL_RECEIPT: FAIL");
    std::process::exit(1);
}
