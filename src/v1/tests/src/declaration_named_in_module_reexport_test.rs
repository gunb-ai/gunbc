//! `v1.compiler.infer_env` `declaration_named_in_module` names the DECLARING module of an imported
//! spelling, never the module the import happens to name.
//!
//! THE DEFECT. Its import arm minted `decl_ref(module_path: <import source>, ...)` without asking
//! whether that module declares the name, so a type reached through a re-exporting module (std.bit
//! `Byte` through v2.std.machine, v2.compiler.inferred_tree `InferredTree` through
//! v2.compiler.infer, v2.std.node `Node` through v2.compiler.target_serialize) was keyed on a module
//! that declares nothing, and `alias_identity_in_judgment` read one declaration as two. The arm now
//! asks `bare_occurrence_import_terminal`, the one walk of the named-import chain to the declaring
//! module (gunbc#15004).
//!
//! WHY A CARGO TEST AND NOT A CLAIM. The subject is v1-internal resolver state. A `dag/test/claim`
//! module cannot import `v1.compiler.*` (src/v1 is not a discovery root; measured: every claim of
//! such a witness failed `module 'v1.std.core' not found`), and `src/v1/tests/claim` has no
//! executing consumer (`gunbc.ci.ci_layer_roots` `v1_claim_scoped_witness_batch_deleted_note`).
//! This test is off the required path, inside `gunbc.rung_drop` `v1_required_lanes_withdrawn`. The
//! required-path discriminator is Charter 1's refusal of a name-only product-head agreement
//! (v1.compiler.infer `nominal_product_heads_agree`), after which a re-export mis-key reds the
//! self-host gate.
//!
//! THE INPUTS ARE SUPPLIED AT THIS ONE INTERFACE: a `SymbolIndex` carrying only the two fields the
//! arm reads, `module_named_imports` (the import chain) and `entries` (the qualified declarations).
//! The real producer of that shape is the corpus compile, whose whole-corpus probe at gunbc#15004
//! took the four re-export identity splits from 4 to 0 with the distinct-declaration set unchanged.
//!
//! THE ORACLE IS THE RULE, not an implementation's output: an imported spelling names the module
//! that declares it; an import chain is walked to that module; a chain that ends where no
//! declaration is indexed answers none, never a reference to a module that declares nothing.

use std::rc::Rc;

use im::HashMap;
use v1_compiler::std_decl_ref::{decl_ref, DeclarationRef};
use v1_compiler::v1_compiler_infer_env::{
    declaration_named_in_module, empty_symbol_index, empty_type_env, SymbolIndex, TypeEnv,
};
use v1_compiler::v1_std_core::divergent_type;

/// `imports` rows are (importer, name, source); `declared` are qualified `module.Name` entries.
fn env_of(imports: &[(&str, &str, &str)], declared: &[&str]) -> Rc<TypeEnv> {
    let mut named: HashMap<String, Rc<HashMap<String, String>>> = HashMap::new();
    for (importer, name, source) in imports {
        let mut row = named
            .get(*importer)
            .map(|m| (**m).clone())
            .unwrap_or_default();
        row.insert(name.to_string(), source.to_string());
        named.insert(importer.to_string(), Rc::new(row));
    }
    let entries = declared
        .iter()
        .map(|q| (q.to_string(), divergent_type()))
        .collect::<HashMap<_, _>>();
    let index = SymbolIndex {
        entries: Rc::new(entries),
        module_named_imports: Rc::new(named),
        ..(*empty_symbol_index()).clone()
    };
    Rc::new(TypeEnv {
        symbol_index: Rc::new(index),
        ..(*empty_type_env()).clone()
    })
}

fn named(env: &Rc<TypeEnv>, name: &str, module: &str) -> Option<Rc<DeclarationRef>> {
    declaration_named_in_module(name.to_string(), module.to_string(), env.clone())
}

fn r(module: &str, name: &str) -> Option<Rc<DeclarationRef>> {
    Some(decl_ref(module.to_string(), name.to_string()))
}

/// RED on the one-hop resolver (it answered probe.reexport::Byte).
#[test]
fn a_type_reached_through_a_reexport_names_its_declaring_module() {
    let env = env_of(
        &[
            ("probe.user", "Byte", "probe.reexport"),
            ("probe.reexport", "Byte", "probe.declaring"),
        ],
        &["probe.declaring.Byte"],
    );
    assert_eq!(
        named(&env, "Byte", "probe.user"),
        r("probe.declaring", "Byte")
    );
}

/// RED on the one-hop resolver, and the chain is walked rather than stepped once more.
#[test]
fn a_two_hop_reexport_chain_names_its_declaring_module() {
    let env = env_of(
        &[
            ("probe.user", "Byte", "probe.outer"),
            ("probe.outer", "Byte", "probe.inner"),
            ("probe.inner", "Byte", "probe.declaring"),
        ],
        &["probe.declaring.Byte"],
    );
    assert_eq!(
        named(&env, "Byte", "probe.user"),
        r("probe.declaring", "Byte")
    );
}

/// THE HOMONYM GUARD: two modules each declare Thing, one reached directly and one through a
/// re-export. Each answer names its own declaring module and the two stay distinct, so the chase
/// never collapses one spelling onto one declaration.
#[test]
fn a_distinct_same_named_pair_keeps_two_declarations() {
    let env = env_of(
        &[
            ("probe.user_a", "Thing", "probe.declares_a"),
            ("probe.user_b", "Thing", "probe.reexport_b"),
            ("probe.reexport_b", "Thing", "probe.declares_b"),
        ],
        &["probe.declares_a.Thing", "probe.declares_b.Thing"],
    );
    let a = named(&env, "Thing", "probe.user_a");
    let b = named(&env, "Thing", "probe.user_b");
    assert_eq!(a, r("probe.declares_a", "Thing"));
    assert_eq!(b, r("probe.declares_b", "Thing"));
    assert_ne!(a, b);
}

/// RED on the one-hop resolver (it answered probe.outside::Gadget). probe.outside has no indexed
/// declaration or import -- the shape of a module outside the compiled source roots -- so nothing
/// confirms a declaration there and the answer is none.
#[test]
fn a_declaring_module_outside_the_index_answers_none() {
    let env = env_of(&[("probe.user", "Gadget", "probe.outside")], &[]);
    assert_eq!(named(&env, "Gadget", "probe.user"), None);
}

/// POSITIVE CONTROL, green on both resolvers: a direct import of the declaring module, and a module
/// declaring the spelling itself, each name that module.
#[test]
fn a_direct_import_and_a_local_declaration_name_their_module() {
    let env = env_of(
        &[("probe.user", "Thing", "probe.declares_a")],
        &["probe.declares_a.Thing", "probe.local.Widget"],
    );
    assert_eq!(
        named(&env, "Thing", "probe.user"),
        r("probe.declares_a", "Thing")
    );
    assert_eq!(
        named(&env, "Widget", "probe.local"),
        r("probe.local", "Widget")
    );
}
