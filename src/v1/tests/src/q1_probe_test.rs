use std::rc::Rc;
use v1_compiler::v1_compiler_compile::compile_to_resolved;
use v1_compiler::v1_compiler_infer_sigs::lookup_resolved_sig;
use crate::helpers::{resolve_imports_transitively_with_source_roots, workspace_root};

#[test]
fn q1_which_declaration_does_typecheck_bind() {
    let ws = workspace_root();
    let roots = [ws.join("src/v2"), ws.join("dag")];
    let src = std::fs::read_to_string(ws.join("dag/test/claim/sigprobe/planted_test.dag")).unwrap();
    let sources = resolve_imports_transitively_with_source_roots("planted_test.dag", &src, &roots);
    let resolved = compile_to_resolved(Rc::new(sources.into()));
    let graph = resolved.graph.as_ref().expect("graph");
    let planted = graph.modules.iter()
        .find(|m| m.func_env.name.contains("planted")).expect("planted module");
    eprintln!("PLANTED ENV = {}", planted.func_env.name);
    eprintln!("PARENTS = {:?}", planted.func_env.parents.iter().map(|p| p.name.clone()).collect::<Vec<_>>());
    match lookup_resolved_sig(planted.func_env.clone(), "twin_sig".to_string()) {
        Some(sig) => eprintln!("RESOLVED twin_sig -> {} PARAM(S)  [twin_p=1, twin_q=3]", sig.params.len()),
        None => eprintln!("RESOLVED twin_sig -> None"),
    }
}
