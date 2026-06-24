use crate::helpers::compile_dag_named;
use v1_compiler::v1_compiler_artifact::RenderTarget;

#[test]
fn probe_measure_emit() {
    let src = concat!(
        "module probe.fixture\n",
        "import std.measure { Measure, ByteSize, Gibibyte, time_measure, measure_add }\n",
        "import std.nat { Nat }\n\n",
        "fn mk(n: Nat) -> ByteSize {\n  byte_size(n)\n}\n"
    );
    let r = compile_dag_named("src/v1/probe.dag", src, RenderTarget::Rust);
    let emitted: String = r
        .files
        .iter()
        .map(|f| f.content.clone())
        .collect::<Vec<_>>()
        .join("\n");
    for line in emitted.lines() {
        if line.contains("Measure")
            || line.contains("struct Time")
            || line.contains("struct Length")
            || line.contains("ByteSize")
            || line.contains("time_measure")
        {
            println!("EMIT: {line}");
        }
    }
    println!("---DIAGS---");
    for d in &r.diagnostics {
        println!("{}", v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()));
    }
}
