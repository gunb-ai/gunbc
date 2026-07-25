//! De-fork non-regression witness for the §7.2 FieldOfFractions model grounding.
//!
//! Unlike GroupCompletion (#7197), FieldOfFractions had no existing §3 fork at the time the
//! model landed — this witness guards against a FUTURE fork appearing (a second independent
//! `type FieldOfFractions` declaration anywhere in the corpus, which would duplicate the
//! single authority at dag/std/algebra.dag and desynchronize from it). Reads corpus SHAPE
//! (a per-file top-level `type FieldOfFractions` declaration head), not contents.

fn workspace_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("workspace root")
}

fn collect_dag_files(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    let mut entries: Vec<_> = match std::fs::read_dir(dir) {
        Ok(e) => e.filter_map(|x| x.ok()).collect(),
        Err(_) => return,
    };
    entries.sort_by_key(|e| e.file_name());
    for e in entries {
        let p = e.path();
        if p.is_dir() {
            collect_dag_files(&p, out);
        } else if p.extension().map(|x| x == "dag").unwrap_or(false) {
            out.push(p);
        }
    }
}

fn declares_field_of_fractions(content: &str) -> bool {
    content.lines().any(|line| {
        let t = line.trim_start();
        t.starts_with("type FieldOfFractions") || t.starts_with("type FieldOfFractions<")
    })
}

#[test]
fn field_of_fractions_has_exactly_one_declaration_corpuswide() {
    let ws = workspace_root();
    let mut files = Vec::new();
    for dir in ["dag", "src/v2"] {
        collect_dag_files(&ws.join(dir), &mut files);
    }
    assert!(!files.is_empty(), "no .dag sources found under dag/ or src/v2/");

    let declaring: Vec<String> = files
        .into_iter()
        .filter_map(|p| {
            let content = std::fs::read_to_string(&p).ok()?;
            if declares_field_of_fractions(&content) {
                Some(p.strip_prefix(&ws).unwrap_or(&p).to_string_lossy().into_owned())
            } else {
                None
            }
        })
        .collect();

    assert_eq!(
        declaring,
        vec!["dag/std/algebra.dag".to_string()],
        "FieldOfFractions must have exactly one declaration at its single authority \
         (dag/std/algebra.dag) — a second declaration anywhere else in the corpus is a §3 \
         fork; found: {declaring:?}"
    );
}

// Red control: the scanner itself must actually detect a declaration when one exists,
// not just report zero unconditionally.
#[test]
fn declares_field_of_fractions_detects_a_real_declaration() {
    assert!(declares_field_of_fractions(
        "module std.algebra\n\ntype FieldOfFractions<R> {\n  num: R\n  denom: R\n}\n"
    ));
    assert!(!declares_field_of_fractions(
        "module std.algebra\n\ntype GroupCompletion<M> {\n  pos: M\n  neg: M\n}\n"
    ));
}
