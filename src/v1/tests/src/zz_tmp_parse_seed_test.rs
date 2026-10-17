#![allow(clippy::all, dead_code, unused_imports)]
// TEMPORARY parse bisector (deleted before merge): the seed grammar rejected my helper twice via CI
// heal (`parse-failed at src/v1/04_infer.dag:10774:27: expected expression, found Eq`), and CI is a
// 7-minute round trip. This prints, in one cargo run, whether the whole seed file parses and whether
// the helper parses on its own. Unbound names are a resolve concern, not a parse one, so nothing is
// stubbed. `include_str!` embeds the seed at COMPILE time so the runner's working directory cannot
// make the measurement silently unavailable (the first version read it from disk and panicked before
// printing anything).
use crate::helpers::parse_source_named;

const SEED: &str = include_str!("../../04_infer.dag");

fn report(label: &str, source: &str) {
    let r = parse_source_named("snippet.dag", source);
    match r.error.clone() {
        Some(e) => {
            let msg = v1_compiler::v1_std_core::diagnostic_to_message(e.diagnostic.clone());
            println!("PARSE_FAIL\t{label}\t{msg}");
        }
        std::option::Option::None => println!("PARSE_OK\t{label}"),
    }
}

fn extract(name: &str) -> String {
    let start = SEED
        .find(&format!("fn {name}"))
        .unwrap_or_else(|| panic!("helper {name} present in the seed"));
    let end = SEED[start..]
        .find("\n}\n")
        .map(|e| start + e + 3)
        .unwrap_or(SEED.len());
    SEED[start..end].to_string()
}

#[test]
fn zz_tmp_parse_seed() {
    report("whole-04_infer", SEED);
    let helper = extract("record_lit_name_declares_union");
    println!("HELPER_A_START\n{helper}\nHELPER_A_END");
    report("helper-A", &helper);
    let helper_b = extract("record_lit_expansion_declares_union");
    println!("HELPER_B_START\n{helper_b}\nHELPER_B_END");
    report("helper-B", &helper_b);
}
