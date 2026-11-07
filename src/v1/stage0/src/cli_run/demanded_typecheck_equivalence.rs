//! A typecheck that infers bodies only for the DEMANDED declarations must give every demanded
//! declaration exactly the typed item and function signature a full typecheck gives it, and must
//! refuse nothing the full typecheck does not. Run on a real closure from the repository: the full
//! chain and the demanded chain each thread their own module index, as `cli_run` does.
use crate::v1_compiler_infer::{self as infer, DeclarationDemand};
use crate::v1_std_core::{authored_name_at, build_newline_index, empty_intern_table, module_items};
use std::collections::HashSet;
use std::rc::Rc;

fn closure_of(entry: &str) -> Vec<Rc<crate::v1_compiler_compile::SourceFile>> {
    let root = std::env::current_dir()
        .unwrap()
        .ancestors()
        .find(|d| d.join("dag").exists() && d.join("Cargo.toml").exists())
        .unwrap()
        .to_path_buf();
    std::env::set_current_dir(&root).unwrap();
    let seed = Rc::new(crate::v1_compiler_compile::SourceFile {
        path: entry.to_string(),
        content: std::fs::read_to_string(root.join(entry)).unwrap(),
    });
    let mut v =
        super::entry_resolve::resolve_seeded_compile_closure(vec![seed], &["dag".to_string()])
            .unwrap();
    v.sort_by(|a, b| a.path.cmp(&b.path));
    v
}

#[test]
fn demanded_set_types_every_demanded_declaration_as_a_full_typecheck_does() {
    std::thread::Builder::new()
        .stack_size(256 << 20)
        .spawn(body)
        .unwrap()
        .join()
        .unwrap();
}

fn body() {
    let sources = closure_of("dag/std/pareto.dag");
    let mut intern = empty_intern_table();
    let mut parsed = Vec::new();
    let mut si = crate::v1_rt::rc_empty_map::<String, Rc<crate::v1_std_core::NewlineIndex>>();
    for s in &sources {
        let idx = build_newline_index(s.path.clone(), s.content.clone());
        si = crate::v1_rt::rc_map_insert(si, s.path.clone(), idx.clone());
        let toks = crate::v1_compiler_tokenize::tokenize(
            s.content.clone(),
            s.path.clone(),
            crate::extdeps_languages_dag_syntax::dag_parse_environment(),
        );
        let p = crate::v1_compiler_parse::parse_with_table(
            toks,
            crate::v1_rt::rc_map_insert(
                crate::v1_rt::rc_empty_map::<String, Rc<crate::v1_std_core::NewlineIndex>>(),
                idx.file.clone(),
                idx.clone(),
            ),
            intern.clone(),
        );
        intern = p.intern_table.clone();
        parsed.push(
            p.result
                .module
                .clone()
                .unwrap_or_else(|| panic!("parse {}", s.path)),
        );
    }
    let graph = crate::v1_compiler_resolve::resolve_modules(Rc::new(parsed.into()), si.clone());
    let entry_module = graph.modules.last().unwrap().clone();
    let seeds: Vec<String> = module_items(entry_module.module.clone())
        .iter()
        .filter(|i| i.body.is_some() && !i.params.is_empty())
        .map(|i| authored_name_at(si.clone(), i.clone()))
        .take(1)
        .collect();
    assert!(!seeds.is_empty(), "entry module has no function to demand");
    let names =
        infer::demanded_declaration_names(graph.modules.clone(), Rc::new(seeds.into()), si.clone());
    let demand = Rc::new(DeclarationDemand::OnlyDeclarations {
        names: names.clone(),
    });

    let mut full = (
        crate::v1_rt::rc_empty_map(),
        crate::v1_rt::rc_empty_map::<String, Rc<infer::VariantExportSurface>>(),
        crate::v1_rt::rc_empty_map::<String, bool>(),
    );
    let mut part = full.clone();
    let (mut skipped, mut compared) = (0usize, 0usize);
    for m in graph.modules.iter() {
        let f = infer::typecheck_module(
            m.clone(),
            full.0.clone(),
            full.1.clone(),
            si.clone(),
            intern.clone(),
            crate::v1_compiler_infer_env::empty_symbol_index(),
            crate::v1_rt::rc_empty_map(),
            full.2.clone(),
        );
        let p = infer::typecheck_module_demanded(
            m.clone(),
            part.0.clone(),
            part.1.clone(),
            si.clone(),
            intern.clone(),
            crate::v1_compiler_infer_env::empty_symbol_index(),
            crate::v1_rt::rc_empty_map(),
            part.2.clone(),
            demand.clone(),
        );
        assert_eq!(f.typed.items.len(), p.typed.items.len());
        for (fi, pi) in f.typed.items.iter().zip(p.typed.items.iter()) {
            if infer::declaration_demanded(demand.clone(), fi.clone(), si.clone()) {
                assert_eq!(
                    fi,
                    pi,
                    "demanded declaration {} typed differently",
                    authored_name_at(si.clone(), fi.clone())
                );
                compared += 1;
            } else {
                assert!(pi.body.is_none());
                skipped += 1;
            }
        }
        let fe: HashSet<String> = f.diagnostics.iter().map(|d| format!("{:?}", d)).collect();
        for d in p.diagnostics.iter() {
            assert!(
                fe.contains(&format!("{:?}", d)),
                "demanded typecheck raised a diagnostic the full typecheck did not: {:?}",
                d
            );
        }
        for (st, r) in [(&mut full, &f), (&mut part, &p)] {
            let t = r.typed.clone();
            let path = authored_name_at(si.clone(), t.module.clone());
            let vs = infer::build_variant_export_surface(t.clone(), st.1.clone(), si.clone());
            st.1 = crate::v1_rt::rc_map_insert(st.1.clone(), path.clone(), vs);
            st.2 = infer::equality_table_with_module(st.2.clone(), t.clone(), si.clone());
            st.0 = crate::v1_rt::rc_map_insert(st.0.clone(), path, t);
        }
    }
    assert!(
        skipped > 0,
        "vacuous: nothing was skipped ({compared} compared)"
    );
    assert!(compared > 0);
    eprintln!("demanded typecheck: {compared} items identical, {skipped} bodies skipped");
}
