#![allow(clippy::all, dead_code, unused_imports)]
// TEMPORARY parse bisector (deleted before merge): the seed grammar rejected my helper three times via
// CI heal (`parse-failed at src/v1/04_infer.dag:10774:27`), and CI is a 7-minute round trip. This
// splices the helper OUT of the real seed text and puts candidate spellings back IN, so every variant
// is parsed in the context that actually fails -- parsing a snippet alone tests a different grammar
// (it lacks the `module` head and the surrounding items). Offsets are printed because the parser's
// message carries no position. `include_str!` embeds the seed at COMPILE time so the runner's working
// directory cannot make the measurement silently unavailable.
use crate::helpers::parse_source_named;

const SEED: &str = include_str!("../../04_infer.dag");

fn line_col(source: &str, byte: i64) -> String {
    if byte < 0 || (byte as usize) > source.len() {
        return format!("?({byte})");
    }
    let b = &source.as_bytes()[..byte as usize];
    let line = 1 + b.iter().filter(|c| **c == b'\n').count();
    let last = b.iter().rposition(|c| *c == b'\n');
    let col = match last {
        Some(p) => b.len() - p,
        None => b.len() + 1,
    };
    format!("{line}:{col}")
}

fn report(label: &str, source: &str) {
    let r = parse_source_named("snippet.dag", source);
    match r.error.clone() {
        Some(e) => {
            let msg = v1_compiler::v1_std_core::diagnostic_to_message(e.diagnostic.clone());
            let span = v1_compiler::v1_std_core::diagnostic_to_span(e.diagnostic.clone());
            let at = line_col(source, span.start);
            let end = line_col(source, span.end);
            let ctx_start = (span.start as usize).saturating_sub(90);
            let ctx = &source[ctx_start.min(source.len())..(span.end as usize).min(source.len())]
                .replace('\n', "\\n");
            println!("PARSE_FAIL\t{label}\t{msg}\tat {at}..{end}\tctx=[{ctx}]");
        }
        std::option::Option::None => println!("PARSE_OK\t{label}"),
    }
}

fn splice(candidate: &str) -> String {
    let start = SEED
        .find("// DOES THIS NAME CARRY A RECORD CONSTRUCTOR?")
        .expect("marker present");
    let end = SEED[start..]
        .find("fn type_has_sole_constructor")
        .map(|e| start + e)
        .expect("next function present");
    format!("{}{}\n{}", &SEED[..start], candidate, &SEED[end..])
}

#[test]
fn zz_tmp_parse_seed() {
    println!("PROBE_VERSION=splice-v2");
    report("whole-as-is", SEED);
    let stripped = splice("");
    report("helper-removed", &stripped);

    let base = "module zz.probe\n";
    let _ = base;

    // V1: match, arms on the `=>` line, else-branch a CALL (no block with lets)
    report(
        "V1-call-in-else",
        &splice(
            "fn record_lit_name_declares_union(type_name: String, scope: InferScope) -> Node? {
  match lookup_type_by_name(env: scope.type_env, name: type_name) {
    Absent => none
    Present { value: decl } => if decl.connective == Disj { Present { value: decl } } else { record_lit_expansion_declares_union(decl: decl, scope: scope) }
  }
}

fn record_lit_expansion_declares_union(decl: Node, scope: InferScope) -> Node? {
  let expanded_result = expand_type_for_field_access(n: decl, env: scope.type_env, module_name: scope.module_name)
  let expanded = expanded_result.resolved
  if expanded.connective == Disj { Present { value: expanded } } else { none }
}
",
        ),
    );

    // V2: no match at all: two lets, an early `return none`, then an if
    report(
        "V2-lets-and-return",
        &splice(
            "fn record_lit_name_declares_union(type_name: String, scope: InferScope) -> Node? {
  let decl_opt = lookup_type_by_name(env: scope.type_env, name: type_name)
  let decl = match decl_opt {
    Present { value: d } => d
    Absent => return none
  }
  if decl.connective == Disj { return Present { value: decl } }
  let expanded_result = expand_type_for_field_access(n: decl, env: scope.type_env, module_name: scope.module_name)
  let expanded = expanded_result.resolved
  if expanded.connective == Disj { Present { value: expanded } } else { none }
}
",
        ),
    );

    // V3: match whose arms are indented BLOCK bodies (the shape at 04_infer.dag:317, lets allowed)
    report(
        "V3-indented-block-arms",
        &splice(
            "fn record_lit_name_declares_union(type_name: String, scope: InferScope) -> Node? {
  match lookup_type_by_name(env: scope.type_env, name: type_name) {
    Absent => none
    Present { value: decl } =>
      if decl.connective == Disj { return Present { value: decl } }
      let expanded_result = expand_type_for_field_access(n: decl, env: scope.type_env, module_name: scope.module_name)
      let expanded = expanded_result.resolved
      if expanded.connective == Disj { Present { value: expanded } } else { none }
  }
}
",
        ),
    );

    // V4: same as V3 but the arms in the seed's other order (Present first), which is what I shipped
    report(
        "V4-present-first",
        &splice(
            "fn record_lit_name_declares_union(type_name: String, scope: InferScope) -> Node? {
  match lookup_type_by_name(env: scope.type_env, name: type_name) {
    Present { value: decl } =>
      if decl.connective == Disj { return Present { value: decl } }
      let expanded_result = expand_type_for_field_access(n: decl, env: scope.type_env, module_name: scope.module_name)
      let expanded = expanded_result.resolved
      if expanded.connective == Disj { Present { value: expanded } } else { none }
    Absent => none
  }
}
",
        ),
    );
}
