// Lane scratch (never lands; cited receipt tool): executed justification for the digest
// census' SKIPPED 1, and the v1-HOST-route half of the data-row layout parity.
// 1) The skipped file, tokenized through the HOST tokenizer (the same lexer the host
//    parse route feeds): every token printed; prefix-operator shapes (ShMinus/ShBang)
//    counted. Zero prefix tokens => no prefix-operator site under ANY binding power.
// 2) parse_dag_content verdict + located message (it renders to stderr itself).
// 3) The two parity layouts (joined vs continuation data row) through the same host route.
// The v2-route half is the scratch claim file run in the same lane.
use v1_compiler::extdeps_languages_dag_syntax::dag_parse_environment;
use v1_compiler::module_path_index::parsed_dag_file::parse_dag_content;
use v1_compiler::v1_compiler_tokenize::tokenize;

fn census(path: &str) {
    let content = std::fs::read_to_string(path).expect("fixture readable");
    println!("=== TOKEN CENSUS {path} ({} bytes) ===", content.len());
    let tokens = tokenize(content.clone(), path.to_string(), dag_parse_environment());
    let mut prefix = 0usize;
    for t in tokens.iter() {
        let shape = format!("{:?}", t.shape);
        if shape == "ShMinus" || shape == "ShBang" {
            prefix += 1;
        }
        println!("token {:?} {:?}", t.text, t.shape);
    }
    println!("PREFIX-TOKEN-COUNT {path}: {prefix}");
    match parse_dag_content(&content, path) {
        Some(p) => println!("HOST-PARSE {path}: OK items={}", p.items.len()),
        None => println!("HOST-PARSE {path}: REFUSED (located message above)"),
    }
}

fn parity(label: &str, content: &str) {
    let name = format!("probe_parity_{label}.dag");
    match parse_dag_content(content, &name) {
        Some(p) => println!("HOST-PARITY {label}: OK items={}", p.items.len()),
        None => println!("HOST-PARITY {label}: REFUSED (located message above)"),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let fixture = args
        .get(0)
        .map(|s| s.as_str())
        .unwrap_or("src/v1/stage0/tests/fixtures/fact_cardinality_split_brace.dag");
    census(fixture);
    println!("=== PARITY via HOST route ===");
    parity(
        "joined",
        "module probe.parity.joined\n\ndata parity_joined: Int = 42\n",
    );
    parity(
        "continuation",
        "module probe.parity.cont\n\ndata parity_contin: Int =\n  42\n",
    );
}
