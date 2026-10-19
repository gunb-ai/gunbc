use std::collections::HashMap;
use std::rc::Rc;

use v1_compiler::std_decl_ref::decl_ref;
use v1_compiler::v1_compiler_compile::{front_end_sources, SourceFile};
use v1_compiler::v1_compiler_infer::typecheck;
use v1_compiler::v1_compiler_infer_env::alias_rhs_of_declaration;
use v1_compiler::v1_std_core::NewlineIndex;

// An alias whose right-hand side spells its own name (`type String`, an opaque kernel leaf) names no
// declaration: resolving the spelling in the alias's own module must not turn the leaf into a
// declaration of that module (DESIGN 4, text crossings). A declared-target alias (`type Hash = Base`)
// keeps its head.
fn alias_rhs_declaration(
    modules: &[(&str, &str)],
    module: &str,
    name: &str,
) -> Option<(String, String)> {
    let sources: Vec<Rc<SourceFile>> = modules
        .iter()
        .map(|(path, content)| {
            Rc::new(SourceFile {
                path: path.to_string(),
                content: content.to_string(),
            })
        })
        .collect();
    let frontend = front_end_sources(Rc::new(sources.into()));
    let graph = frontend.graph.clone().expect("fixture parses");
    let mut by_file: HashMap<String, Rc<NewlineIndex>> = HashMap::new();
    for index in frontend.newline_indices.iter() {
        by_file.insert(index.file.clone(), index.clone());
    }
    let typed = typecheck(
        graph,
        Rc::new(by_file.into()),
        frontend.intern_table.clone(),
    );
    let env = typed.modules.front().expect("a module").type_env.clone();
    alias_rhs_of_declaration(decl_ref(module.to_string(), name.to_string()), env)
        .and_then(|rhs| rhs.declaration.clone())
        .map(|d| (d.module_path.clone(), d.decl_name.clone()))
}

const KERNEL_LEAF: (&str, &str) = ("kn.dag", "module kn\ntype String\n");
const OPAQUE_NON_KERNEL: (&str, &str) = ("sy.dag", "module sy\ntype Symbol\n");
const DECLARED: (&str, &str) = (
    "dc.dag",
    "module dc\ntype Base { x: Int }\ntype Hash = Base\n",
);

#[test]
fn opaque_kernel_leaf_alias_stays_declaration_less() {
    assert_eq!(
        alias_rhs_declaration(&[KERNEL_LEAF, DECLARED], "kn", "String"),
        None,
        "`type String` must not resolve to a declaration of its own module"
    );
}

#[test]
fn declared_target_alias_keeps_its_head() {
    assert_eq!(
        alias_rhs_declaration(&[KERNEL_LEAF, DECLARED], "dc", "Hash"),
        Some(("dc".to_string(), "Base".to_string()))
    );
}

#[test]
fn opaque_declaration_is_not_an_alias_of_itself_whatever_its_spelling() {
    assert_eq!(
        alias_rhs_declaration(&[OPAQUE_NON_KERNEL, DECLARED], "sy", "Symbol"),
        None,
        "an opaque declaration has no alias right-hand side, kernel-named or not"
    );
}
