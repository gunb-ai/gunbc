// SCRATCH probe -- measures the cost curve of string-literal escape processing.
// Not a receipt; deleted before the PR lands.
use std::time::Instant;
use v1_compiler::v1_compiler_tokenize::tokenize;

fn source_with(repeats: usize) -> String {
    let unit = "é\\n—ü\\x41🟡ß";
    let mut body = String::new();
    for _ in 0..repeats {
        body.push_str(unit);
    }
    format!("data x: String = \"{}\"", body)
}

#[test]
fn probe_cost_curve() {
    for repeats in [250usize, 500, 1000, 2000, 4000] {
        let src = source_with(repeats);
        let chars = src.chars().count();
        let t0 = Instant::now();
        let toks = tokenize(src, "probe.dag".to_string());
        let dt = t0.elapsed();
        println!(
            "repeats={:>5} literal_chars={:>7} tokens={:>3} elapsed={:?}",
            repeats,
            chars,
            toks.len(),
            dt
        );
    }
}
