// AN UNREADABLE CLAIM SOURCE IS NO OBSERVATION, AND THE ROUTE NAMES THE FILE.
//
// `gunbc test //test/claim/x:c` used to read a header-less claim file as "the pattern selected no
// discovered claim" (the index silently dropped it) and a header-bearing unparseable one as exit 1
// (`module index refused`, process exit). Both are refusals located at the file; `run_claim_route`
// returns them as `Err`, which `target_invocation_host` terminates `SubjectUnreached` (exit 2).
//
// THE CONTROL IS THE POINT OF THE THIRD TEST: the same scratch root with a healthy claim must
// still select and HOLD it, so a refusal that fires on everything cannot pass the two REDs.
// The scratch root is added beside the real witness roots because discovery resolves the floor's
// own authority module from them.

use std::path::PathBuf;

fn scratch_claim_root(name: &str, file: &str, body: &str) -> String {
    // Under the workspace root: the resolve normalizes every source path to a workspace-relative
    // key and refuses a root outside it, so a system temp dir cannot host the healthy control.
    let dir = v1_compiler::cli_run::workspace_root()
        .join("target")
        .join(format!("gunbc_claim_route_{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    let claim_dir: PathBuf = dir.join("test").join("claim");
    std::fs::create_dir_all(&claim_dir).expect("scratch claim dir");
    std::fs::write(claim_dir.join(file), body).expect("scratch claim source");
    dir.to_string_lossy().into_owned()
}

fn route(scratch: String) -> Result<usize, String> {
    // Entry resolution reads workspace-relative paths against the process cwd, which the `gunbc`
    // binary stands at the workspace root and a cargo test stands at the crate. Every test here
    // sets the same directory, so the shared process-global is written with one value.
    std::env::set_current_dir(v1_compiler::cli_run::workspace_root()).expect("enter workspace");
    let mut roots = vec![scratch];
    roots.extend(v1_compiler::cli_run::witness_layer_roots());
    v1_compiler::cli_run::run_claim_route(
        &roots,
        &|module| module == "test.claim.zz_scratch",
        &|_, _| true,
    )
    .map(|members| {
        members
            .iter()
            .filter(|m| matches!(m.outcome, v1_compiler::cli_run::ClaimOutcome::Pass))
            .count()
    })
}

#[test]
fn a_headerless_claim_source_refuses_naming_the_file() {
    let root = scratch_claim_root(
        "headerless",
        "zz_scratch.dag",
        "test fn zz_claim() -> Bool {\n  true\n}\n",
    );
    let err = route(root).expect_err("a header-less claim source must refuse, not select nothing");
    assert!(err.contains("zz_scratch.dag"), "must name the file: {err}");
    assert!(
        err.contains("declares no module"),
        "must name the cause: {err}"
    );
}

#[test]
fn an_unparseable_claim_source_refuses_naming_the_file() {
    let root = scratch_claim_root(
        "unparseable",
        "zz_scratch.dag",
        "module test.claim.zz_scratch\n\ntest fn zz_claim() -> Bool {\n  true &&\n",
    );
    let err = route(root).expect_err("an unparseable claim source must refuse");
    assert!(err.contains("zz_scratch.dag"), "must name the file: {err}");
    assert!(err.contains("does not parse"), "must name the cause: {err}");
}

#[test]
fn a_healthy_claim_source_is_selected_and_holds() {
    let root = scratch_claim_root(
        "healthy",
        "zz_scratch.dag",
        "module test.claim.zz_scratch\n\ntest fn zz_claim() -> Bool {\n  true\n}\n",
    );
    assert_eq!(route(root).expect("a healthy claim must not refuse"), 1);
}

// THE RUST RENDERER IS PINNED TO THE SAME LITERALS AS THE `.dag` CARRIER. The `.dag` witness
// `a_claim_source_refusal_names_the_file_and_its_cause` asserts these exact strings for
// `gunbc.target_invocation` `claim_source_refusal_rendered`; if either renderer's wording or cause
// set moves alone, one of the two reds.
#[test]
fn the_rust_renderer_matches_the_dag_carrier_for_every_cause() {
    use v1_compiler::cli_run::{claim_source_refusal_rendered as r, ClaimSourceRefusalCause as C};
    assert_eq!(
        r(
            "dag/test/claim/x.dag",
            &C::Unparseable {
                diagnostic: "d".into()
            }
        ),
        "claim source refused: dag/test/claim/x.dag: the file does not parse: d"
    );
    assert_eq!(
        r("dag/test/claim/y.dag", &C::ModuleHeaderAbsent),
        "claim source refused: dag/test/claim/y.dag: a .dag under the claim root declares no module, so no claim label can name it (paths follow names)"
    );
    assert_eq!(
        r("p", &C::Unreadable { reason: "r".into() }),
        "claim source refused: p: unreadable: r"
    );
    assert_eq!(
        r("p", &C::ModuleCollision { other: "q".into() }),
        "claim source refused: p: declares a module also declared by q; one module, one authority (DESIGN section 3), so de-fork or rename one side"
    );
    assert_eq!(
        r("p", &C::DirectoryNamedDag),
        "claim source refused: p: a directory named like a .dag source; a source path is a file"
    );
    assert_eq!(
        r("p", &C::RootUnreadable { reason: "r".into() }),
        "claim source refused: p: unreadable source root: r"
    );
}

#[test]
fn a_directory_named_like_a_dag_source_refuses_naming_the_path() {
    let root = scratch_claim_root("dirdag", "ok.dag", "module test.claim.zz_ok\n");
    let dir = std::path::Path::new(&root).join("test/claim/zz_dir.dag");
    std::fs::create_dir_all(&dir).expect("directory named .dag");
    let err = route(root).expect_err("a directory named x.dag must refuse, not read as green");
    assert!(err.contains("zz_dir.dag"), "must name the path: {err}");
    assert!(
        err.contains("a directory named like a .dag source"),
        "{err}"
    );
}

// A dangling symlink named like a source cannot be read by ANY uid, root included (a mode-000 file
// can, because root bypasses permissions), so this RED discriminates wherever it runs and never
// takes a skip branch.
#[test]
fn an_unreadable_claim_source_refuses_naming_the_file() {
    let root = scratch_claim_root("dangling", "ok.dag", "module test.claim.zz_ok\n");
    let link = std::path::Path::new(&root).join("test/claim/zz_dangling.dag");
    std::os::unix::fs::symlink("/nonexistent/zz_target.dag", &link).expect("dangling symlink");
    let err = route(root).expect_err("an unreadable claim source must refuse, not panic");
    assert!(err.contains("zz_dangling.dag"), "must name the file: {err}");
}

#[test]
fn two_claim_sources_declaring_one_module_refuse_naming_both_paths() {
    let root = scratch_claim_root("collision", "a.dag", "module test.claim.zz_dup\n");
    let second = std::path::Path::new(&root).join("test/claim/b.dag");
    std::fs::write(&second, "module test.claim.zz_dup\n").expect("second declaring file");
    let err = route(root).expect_err("a module-path collision must refuse, not panic");
    assert!(
        err.contains("a.dag") && err.contains("b.dag"),
        "must name both paths: {err}"
    );
    assert!(err.contains("also declared by"), "{err}");
}
