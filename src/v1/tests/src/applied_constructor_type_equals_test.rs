use crate::helpers::{compile_dag, compile_multi, diagnostic_messages};
use v1_compiler::v1_std_core::is_interpreter_blocking_diagnostic;

fn blocking_of(result: &v1_compiler::v1_compiler_compile::PipelineResult) -> Vec<String> {
    result
        .diagnostics
        .iter()
        .filter(|d| is_interpreter_blocking_diagnostic(d.diagnostic.clone()))
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .collect()
}

fn has_rebind(msgs: &[String]) -> bool {
    msgs.iter().any(|m| {
        m.contains("type variable") && m.contains("bound as") && m.contains("later argument")
    })
}

#[test]
fn same_applied_box_int_is_accepted() {
    let result = compile_dag(
        "module locate.boxeq\ntype Box<T>\nfn same_arm<A>(a: A, b: A) -> Bool { true }\nfn probe(x: Box<Int>, y: Box<Int>) -> Bool { same_arm(a: x, b: y) }\n",
    );
    let msgs = blocking_of(&result);
    assert!(
        !has_rebind(&msgs),
        "two Box<Int> must compare equal: {msgs:?}"
    );
}

#[test]
fn applied_box_int_vs_bool_is_refused() {
    let result = compile_dag(
        "module locate.boxne\ntype Box<T>\nfn same_arm<A>(a: A, b: A) -> Bool { true }\nfn probe(x: Box<Int>, y: Box<Bool>) -> Bool { same_arm(a: x, b: y) }\n",
    );
    let msgs = blocking_of(&result);
    assert!(
        has_rebind(&msgs),
        "Box<Int> vs Box<Bool> must refuse: {msgs:?} all={:?}",
        diagnostic_messages(&result)
    );
}

#[test]
fn list_int_vs_list_string_is_refused() {
    let result = compile_dag(
        "module locate.listne\nimport std.types { List }\nfn same_arm<A>(a: A, b: A) -> Bool { true }\nfn probe(x: List<Int>, y: List<String>) -> Bool { same_arm(a: x, b: y) }\n",
    );
    let msgs = blocking_of(&result);
    assert!(
        has_rebind(&msgs),
        "List<Int> vs List<String> must refuse: {msgs:?} all={:?}",
        diagnostic_messages(&result)
    );
}

#[test]
fn list_int_equals_freemonoid_int() {
    let result = compile_dag(
        "module locate.listalias\nimport std.types { List }\nimport std.algebra { FreeMonoid }\nfn same_arm<A>(a: A, b: A) -> Bool { true }\nfn probe(x: List<Int>, y: FreeMonoid<Int>) -> Bool { same_arm(a: x, b: y) }\n",
    );
    let msgs = blocking_of(&result);
    assert!(!has_rebind(&msgs), "List<Int> is FreeMonoid<Int>: {msgs:?}");
}

fn applied_head(
    decl: std::rc::Rc<v1_compiler::std_decl_ref::DeclarationRef>,
) -> std::rc::Rc<v1_compiler::v1_std_core::Node> {
    applied_named(
        decl.decl_name.clone(),
        decl,
        v1_compiler::v1_std_core::int_type(),
    )
}

fn applied_named(
    name: String,
    decl: std::rc::Rc<v1_compiler::std_decl_ref::DeclarationRef>,
    elem: std::rc::Rc<v1_compiler::v1_std_core::Node>,
) -> std::rc::Rc<v1_compiler::v1_std_core::Node> {
    let mut n = (*v1_compiler::v1_std_core::unit_type()).clone();
    n.connective = v1_compiler::v1_std_core::Connective::NoConnective;
    n.name = name.clone();
    n.ident_span = Some(v1_compiler::v1_std_core::kernel_span(name));
    n.declaration = Some(decl);
    n.children = std::rc::Rc::new(im::vector![elem]);
    std::rc::Rc::new(n)
}

fn empty_env(
    module_path: &str,
    str_bindings: im::HashMap<String, std::rc::Rc<v1_compiler::v1_compiler_infer_env::TypeBinding>>,
    parents: im::Vector<std::rc::Rc<v1_compiler::v1_compiler_infer_env::TypeEnv>>,
) -> std::rc::Rc<v1_compiler::v1_compiler_infer_env::TypeEnv> {
    std::rc::Rc::new(v1_compiler::v1_compiler_infer_env::TypeEnv {
        bindings: v1_compiler::v1_rt::rc_empty_map(),
        str_bindings: std::rc::Rc::new(str_bindings),
        ancestry_str_bindings: v1_compiler::v1_rt::rc_empty_map(),
        parents: std::rc::Rc::new(parents),
        recursive_types: std::rc::Rc::new(im::vector![]),
        recursive_type_set: v1_compiler::v1_rt::rc_empty_map(),
        inductive_fields: v1_compiler::v1_rt::rc_empty_map(),
        source_indices: v1_compiler::v1_rt::rc_empty_map(),
        intern_table: v1_compiler::v1_std_core::empty_intern_table(),
        source_visible_names: v1_compiler::v1_rt::rc_empty_map(),
        authored_import_names: v1_compiler::v1_rt::rc_empty_map(),
        symbol_index: v1_compiler::v1_compiler_infer_env::empty_symbol_index(),
        module_path: module_path.to_string(),
        unit_variant_index: v1_compiler::v1_rt::rc_empty_map(),
        unit_variant_index_observed: false,
    })
}

#[test]
fn type_heads_agree_uses_alias_declaration_not_list_spelling() {
    let fm_decl = v1_compiler::std_decl_ref::decl_ref("std.algebra".into(), "FreeMonoid".into());
    let std_list_decl = v1_compiler::std_decl_ref::decl_ref("std.types".into(), "List".into());
    let local_list_decl =
        v1_compiler::std_decl_ref::decl_ref("locate.locallist".into(), "List".into());
    let mut fm_rhs = (*v1_compiler::v1_std_core::unit_type()).clone();
    fm_rhs.declaration = Some(fm_decl.clone());
    fm_rhs.name = "FreeMonoid".into();
    let list_binding = std::rc::Rc::new(v1_compiler::v1_compiler_infer_env::TypeBinding {
        name: "List".into(),
        resolved: v1_compiler::v1_std_core::unit_type(),
        provenance: std::rc::Rc::new(v1_compiler::std_induction::SubValueRelation::SubValueUnknown),
        alias_rhs: Some(std::rc::Rc::new(fm_rhs)),
    });
    let mut std_bindings = im::HashMap::new();
    std_bindings.insert("List".into(), list_binding);
    let std_types = empty_env("std.types", std_bindings, im::vector![]);
    let use_env = empty_env(
        "locate.locallist",
        im::HashMap::new(),
        im::vector![std_types],
    );
    let text = std::rc::Rc::new(
        v1_compiler::v1_compiler_infer_types::TextJudgment::TextJudgedIn { env: use_env },
    );
    let local = applied_head(local_list_decl);
    let std_list = applied_head(std_list_decl);
    let fm = applied_head(fm_decl);
    assert!(
        v1_compiler::v1_compiler_infer_types::type_heads_agree(
            std_list.clone(),
            fm.clone(),
            text.clone()
        ),
        "std.types.List unfolds to FreeMonoid"
    );
    assert!(
        !v1_compiler::v1_compiler_infer_types::type_heads_agree(local, std_list, text),
        "module-local List must not agree with std.types.List by last-segment carrier"
    );
}

#[test]
fn list_int_node_equals_freemonoid_int() {
    let fm_decl = v1_compiler::std_decl_ref::decl_ref("std.algebra".into(), "FreeMonoid".into());
    let std_list_decl = v1_compiler::std_decl_ref::decl_ref("std.types".into(), "List".into());
    let mut fm_rhs = (*v1_compiler::v1_std_core::unit_type()).clone();
    fm_rhs.declaration = Some(fm_decl.clone());
    fm_rhs.name = "FreeMonoid".into();
    let list_binding = std::rc::Rc::new(v1_compiler::v1_compiler_infer_env::TypeBinding {
        name: "List".into(),
        resolved: v1_compiler::v1_std_core::unit_type(),
        provenance: std::rc::Rc::new(v1_compiler::std_induction::SubValueRelation::SubValueUnknown),
        alias_rhs: Some(std::rc::Rc::new(fm_rhs)),
    });
    let mut std_bindings = im::HashMap::new();
    std_bindings.insert("List".into(), list_binding);
    let std_types = empty_env("std.types", std_bindings, im::vector![]);
    let text = std::rc::Rc::new(
        v1_compiler::v1_compiler_infer_types::TextJudgment::TextJudgedIn {
            env: empty_env("use", im::HashMap::new(), im::vector![std_types]),
        },
    );
    let list = applied_named(
        "List".into(),
        std_list_decl,
        v1_compiler::v1_std_core::int_type(),
    );
    let fm = applied_named(
        "FreeMonoid".into(),
        fm_decl,
        v1_compiler::v1_std_core::int_type(),
    );
    let indices = v1_compiler::v1_rt::rc_empty_map();
    assert!(
        v1_compiler::v1_compiler_infer_types::node_is_element_collection(
            list.clone(),
            indices.clone()
        ),
        "List is the alias spelling; both-container must fail against FreeMonoid"
    );
    assert!(
        !v1_compiler::v1_compiler_infer_types::node_is_element_collection(
            fm.clone(),
            indices.clone()
        ),
        "FreeMonoid is not a container alias spelling"
    );
    assert!(
        v1_compiler::v1_compiler_infer_types::node_type_equals(
            list,
            fm,
            indices,
            text
        ),
        "List<Int> equals FreeMonoid<Int> through the applied arm after containers; extra seed exclusions would skip that arm and fail here"
    );
}

#[test]
fn bare_box_vs_qualified_other_decl_is_refused() {
    let result = compile_multi(&[
        ("q/boxmod.dag", "module q.boxmod\ntype Box<T>\n"),
        (
            "locate/bareq.dag",
            "module locate.bareq\nimport q.boxmod\ntype Box<T>\nfn same_arm<A>(a: A, b: A) -> Bool { true }\nfn probe(x: Box<Int>, y: q.boxmod.Box<Int>) -> Bool { same_arm(a: x, b: y) }\n",
        ),
    ]);
    let msgs = blocking_of(&result);
    assert!(
        has_rebind(&msgs)
            || msgs.iter().any(|m| m.contains("UnresolvedDecl")
                || m.contains("TypeMismatch")
                || m.contains("type mismatch")),
        "bare Box<Int> vs q.boxmod.Box<Int> must not equal by last segment: blocking={msgs:?} all={:?}",
        diagnostic_messages(&result)
    );
}

#[test]
fn two_decls_same_leaf_applied_compare_unequal() {
    let result = compile_multi(&[
        ("p/boxmod.dag", "module p.boxmod\ntype Box<T>\n"),
        ("q/boxmod.dag", "module q.boxmod\ntype Box<T>\n"),
        (
            "locate/hom.dag",
            "module locate.hom\nimport p.boxmod\nimport q.boxmod\nfn same_arm<A>(a: A, b: A) -> Bool { true }\nfn probe(x: p.boxmod.Box<Int>, y: q.boxmod.Box<Int>) -> Bool { same_arm(a: x, b: y) }\n",
        ),
    ]);
    let msgs = blocking_of(&result);
    assert!(
        has_rebind(&msgs)
            || msgs.iter().any(|m| m.contains("TypeMismatch") || m.contains("type mismatch")),
        "p.boxmod.Box<Int> vs q.boxmod.Box<Int> must not compare equal by leaf name: blocking={msgs:?} all={:?}",
        diagnostic_messages(&result)
    );
}
