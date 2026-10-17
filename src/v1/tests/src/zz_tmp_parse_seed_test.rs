#![allow(clippy::all, dead_code, unused_imports)]
use crate::helpers::parse_source_named;
use std::rc::Rc;

fn report(label: &str, source: &str) {
    let r = parse_source_named("snippet.dag", source);
    match &*r.error.clone() {
        Some(e) => {
            let msg = v1_compiler::v1_std_core::diagnostic_to_message(e.diagnostic.clone());
            println!("PARSE_FAIL\t{label}\t{msg}");
        }
        std::option::Option::None => println!("PARSE_OK\t{label}"),
    }
}

#[test]
fn zz_tmp_parse_seed() {
    let whole = std::fs::read_to_string("src/v1/04_infer.dag").expect("read seed infer");
    report("whole-04_infer", &whole);

    let all = std::fs::read_to_string("src/v1/04_infer.dag").unwrap();
    let start = all
        .find("fn record_lit_name_declares_union")
        .expect("helper");
    let end = all[start..]
        .find("\n}\n")
        .map(|e| start + e + 3)
        .unwrap_or(all.len());
    let helper = all[start..end].to_string();
    println!("HELPER_TEXT_START");
    for (i, l) in helper.lines().enumerate() {
        println!("{:02}: {}", i, l);
    }
    println!("HELPER_TEXT_END");
    let prelude = "module zz.probe\ntype Node { a: Int }\ntype InferScope { a: Int }\ntype TypeEnv { a: Int }\ntype NodeResolveResult { resolved: Node }\nfn lookup_type_by_name(env: TypeEnv, name: String) -> Node? { none }\nfn expand_type_for_field_access(n: Node, env: TypeEnv, module_name: String) -> NodeResolveResult { NodeResolveResult { resolved: Node { a: 1 } } }\n";
    // the helper alone, with stubs for its dependencies
    report("helper-alone", &format!("{prelude}{helper}\n"));

    // variants to bisect the construct
    let v1 = "fn zz_f(type_name: String, scope: InferScope) -> Node? {\n  match lookup_type_by_name(env: scope.type_env, name: type_name) {\n    Present { value: decl } =>\n      let x = if decl.connective == NoConnective {\n        decl\n      } else { decl }\n      if x.connective == Disj { Present { value: x } } else { none }\n    Absent => none\n  }\n}\n";
    report("v1-let-then-if", v1);
    let v2 = "fn zz_f(type_name: String, scope: InferScope) -> Node? {\n  match lookup_type_by_name(env: scope.type_env, name: type_name) {\n    Present { value: decl } =>\n      if decl.connective == Disj { Present { value: decl } } else { none }\n    Absent => none\n  }\n}\n";
    report("v2-simple-arm", v2);
    let v3 = "fn zz_f(type_name: String, scope: InferScope) -> Node? {\n  match lookup_type_by_name(env: scope.type_env, name: type_name) {\n    Present { value: decl } =>\n      let to_expand = if decl.connective == NoConnective && decl.children |> count == 0 && decl.inferred != none {\n        match decl.inferred {\n          Present { value: Resolved { node: target } } => target\n          _ => decl\n        }\n      } else { decl }\n      if to_expand.connective == Disj { Present { value: to_expand } } else { none }\n    Absent => none\n  }\n}\n";
    report("v3-nested-match-in-let-if", v3);
    let v4 = "fn zz_f(type_name: String, scope: InferScope) -> Node? {\n  match lookup_type_by_name(env: scope.type_env, name: type_name) {\n    Present { value: decl } =>\n      let a = decl\n      let expanded_result = expand_type_for_field_access(n: a, env: scope.type_env, module_name: scope.module_name)\n      let expanded = expanded_result.resolved\n      if expanded.connective == Disj { Present { value: expanded } } else { none }\n    Absent => none\n  }\n}\n";
    report("v4-lets-then-if", v4);
    let v5 = "fn zz_f(type_name: String, scope: InferScope) -> Node? {\n  match lookup_type_by_name(env: scope.type_env, name: type_name) {\n    Present { value: decl } =>\n      if decl.connective == Disj {\n        Present { value: decl }\n      } else if decl.connective == NoConnective {\n        let expanded_result = expand_type_for_field_access(n: decl, env: scope.type_env, module_name: scope.module_name)\n        let expanded = expanded_result.resolved\n        if expanded.connective == Disj { Present { value: expanded } } else { none }\n      } else { none }\n    Absent => none\n  }\n}\n";
    report("v5-if-else-if-with-lets", v5);
}
