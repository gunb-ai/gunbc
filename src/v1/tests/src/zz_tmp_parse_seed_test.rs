#![allow(clippy::all, dead_code, unused_imports)]
// TEMPORARY parse bisector (deleted before merge): the seed grammar rejected my helper twice via CI
// heal (`parse-failed at src/v1/04_infer.dag:10774:29: expected expression, found Eq`), and CI is a
// 7-minute round trip. This prints, in one cargo run, whether the whole seed file parses and which
// of the arm spellings the parser accepts. `include_str!` embeds the seed at COMPILE time so the
// runner's working directory cannot make the measurement silently unavailable (the first version read
// it from disk and panicked before printing anything).
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

const PRELUDE: &str = "module zz.probe\ntype Node { connective: Int, children: List<Int>, inferred: Int }\ntype InferScope { type_env: Int, module_name: String }\nenum Connective { Disj, NoConnective }\ntype TypeEnv { a: Int }\ntype NodeResolveResult { resolved: Node }\nfn lookup_type_by_name(env: Int, name: String) -> Node? { none }\nfn expand_type_for_field_access(n: Node, env: Int, module_name: String) -> NodeResolveResult { NodeResolveResult { resolved: Node { connective: 1, children: [], inferred: 0 } } }\n";

#[test]
fn zz_tmp_parse_seed() {
    report("whole-04_infer", SEED);

    let start = SEED
        .find("fn record_lit_name_declares_union")
        .expect("helper present");
    let end = SEED[start..]
        .find("\n}\n")
        .map(|e| start + e + 3)
        .unwrap_or(SEED.len());
    let helper = SEED[start..end].to_string();
    println!("HELPER_START");
    println!("{helper}");
    println!("HELPER_END");
    report("helper-with-stubs", &format!("{PRELUDE}{helper}"));

    // A: the arm body on the SAME line as `=>` (the shape used throughout the seed's own code)
    report(
        "A-same-line-arm",
        &format!(
            "{PRELUDE}fn zz_a(type_name: String, scope: InferScope) -> Node? {{\n  match lookup_type_by_name(env: scope.type_env, name: type_name) {{\n    Absent => none\n    Present {{ value: decl }} => if decl.connective == Disj {{\n      Present {{ value: decl }}\n    }} else if decl.connective == NoConnective {{\n      let expanded_result = expand_type_for_field_access(n: decl, env: scope.type_env, module_name: scope.module_name)\n      let expanded = expanded_result.resolved\n      if expanded.connective == Disj {{ Present {{ value: expanded }} }} else {{ none }}\n    }} else {{ none }}\n  }}\n}}\n"
        ),
    );

    // B: the arm body on the line AFTER `=>`, an if-expression (what CI refused)
    report(
        "B-next-line-if",
        &format!(
            "{PRELUDE}fn zz_b(type_name: String, scope: InferScope) -> Node? {{\n  match lookup_type_by_name(env: scope.type_env, name: type_name) {{\n    Present {{ value: decl }} =>\n      if decl.connective == Disj {{\n        Present {{ value: decl }}\n      }} else {{ none }}\n    Absent => none\n  }}\n}}\n"
        ),
    );

    // C: the arm body on the line after `=>`, with `let`s then an if (the block-body shape)
    report(
        "C-next-line-lets",
        &format!(
            "{PRELUDE}fn zz_c(type_name: String, scope: InferScope) -> Node? {{\n  match lookup_type_by_name(env: scope.type_env, name: type_name) {{\n    Present {{ value: decl }} =>\n      let expanded_result = expand_type_for_field_access(n: decl, env: scope.type_env, module_name: scope.module_name)\n      let expanded = expanded_result.resolved\n      if expanded.connective == Disj {{ Present {{ value: expanded }} }} else {{ none }}\n    Absent => none\n  }}\n}}\n"
        ),
    );
}
