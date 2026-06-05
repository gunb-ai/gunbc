use v3_compiler::compile_to_dag;
use v3_compiler::CompileError;

#[test]
fn probe_brand_cross_and_raw() {
    let f = "dsl/std/integer.dag";
    let cross = "type BrandA = Int where brand(\"A\")\n\
type BrandB = Int where brand(\"B\")\n\
fn take_a(x: BrandA) -> Int = x\n\
data b: BrandB = 1\n\
fn cross() -> Int = take_a(b)\n";
    match compile_to_dag(cross, f) {
        Ok(_) => eprintln!("cross-brand: ACCEPTS"),
        Err(CompileError::Semantic(dag)) => {
            eprintln!("cross-brand: REJECTS");
            for (_, d) in dag.diagnostics().iter() { eprintln!("  {d:?}"); }
        }
        Err(e) => panic!("cross: {e:?}"),
    }
    let raw = "type BrandA = Int where brand(\"A\")\n\
fn take_a(x: BrandA) -> Int = x\n\
fn cross() -> Int = take_a(1)\n";
    match compile_to_dag(raw, f) {
        Ok(_) => eprintln!("raw-int: ACCEPTS"),
        Err(CompileError::Semantic(dag)) => {
            eprintln!("raw-int: REJECTS");
            for (_, d) in dag.diagnostics().iter() { eprintln!("  {d:?}"); }
        }
        Err(e) => panic!("raw: {e:?}"),
    }
}
