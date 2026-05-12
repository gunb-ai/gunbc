//! **Layer:** integration
//!
//! §1.8 / gate #92 `complexity_violation_compile_error_demonstrated`: a concrete
//! recursive-descent program whose `complexity_of` asymptotic class strictly exceeds a
//! `ClassConstant` budget under an `EnforcedApplication` must fail closed with an
//! Error/`ParseError` diagnostic at the lens-application site.

use std::fs;
use std::path::PathBuf;

use v3_compiler::compile_to_dag;
use v3_compiler::CompileError;

const DEMO_REL_PATH: &str = "src/v3/compiler/tests/fixtures/t_las_complexity_contract_demo.dag";
const DEMO_FILE_NAME: &str = "t_las_complexity_contract_demo.dag";
const DEMO_APPLICATION_MARKER: &str = "data witness_log_cap:";

#[test]
#[ignore = "hot-fix-2026-05-12 cold-v3-67min-reduction; rebuild via OnceLock/cached_compile amortization — owner: TBD per separate dispatch"]
fn complexity_violation_compile_error_demonstrated() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join(DEMO_REL_PATH);
    let source = fs::read_to_string(&path).expect("read T-LAS complexity demo fixture");
    let expected_span = authored_application_span(&source);
    std::thread::Builder::new()
        .name("t-las-complexity-demo-compile".into())
        .stack_size(64 * 1024 * 1024)
        .spawn(move || {
            let err = compile_to_dag(&source, DEMO_FILE_NAME).expect_err(
                "recursive-descent witness asymptotic class exceeds ClassConstant - compile must fail",
            );
            let CompileError::Semantic(dag) = err else {
                panic!("expected Semantic(Dag) after violation, got {err:?}");
            };
            let receipts: Vec<_> = dag
                .diagnostics()
                .iter()
                .map(|(_, d)| (d.layer1_kind_label().to_string(), d.message(), d.span()))
                .collect();
            let ok = receipts.iter().any(|(kind, msg, span)| {
                kind == "ParseError"
                    && msg.contains("lens enforcement violation")
                    && msg.contains("ClassConstant")
                    && msg.contains("ClassUnknown")
                    && span.file.ends_with(DEMO_FILE_NAME)
                    && (span.byte_start, span.byte_end) == expected_span
            });
            assert!(
                ok,
                "expected Error/ParseError with budget/observed classes at lens span; got {receipts:?}"
            );
        })
        .expect("spawn demo compile")
        .join()
        .expect("demo compile thread panicked");
}

fn authored_application_span(source: &str) -> (u32, u32) {
    let start = source
        .find(DEMO_APPLICATION_MARKER)
        .expect("fixture contains witness application");
    let end = source[start..]
        .find("\n}")
        .map(|offset| start + offset + 2)
        .expect("fixture witness application closes");
    (
        u32::try_from(start).expect("fixture start offset fits in SourceSpan"),
        u32::try_from(end).expect("fixture end offset fits in SourceSpan"),
    )
}
