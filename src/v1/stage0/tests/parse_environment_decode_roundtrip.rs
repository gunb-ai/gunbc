//! THE ENVIRONMENT LOADER, EXERCISED SO THAT AN IGNORED REVISION ARGUMENT CANNOT PASS.
//!
//! Two separate claims, because an earlier version of this file conflated them and could not have
//! caught the defect it was written to rule out.
//!
//! 1. DECODE. `evaluate_environment_in` over the live tree must reproduce `dag_parse_environment()`
//!    -- the environment compiled into this binary from the same declarations. An independent
//!    referent: the two agree only if evaluation, wire encoding and deserialization all preserved
//!    the environment.
//!
//! 2. REVISION SELECTION. `load_parse_environment_at` must read the bytes GIT holds, not the
//!    worktree's. The previous test called `evaluate_environment_in(workspace_root(), "HEAD")` and
//!    would have stayed green with the revision reader deleted outright, because the revision never
//!    selected anything. The discriminating case here is a revision whose environment DIFFERS from
//!    the worktree's, which is only authorable against a scratch repository -- so that is what the
//!    second test builds.

use std::path::Path;
use std::process::Command;

use v1_compiler::cli_run::namespace_wave_admission::{
    blob_id_at, evaluate_environment_in, load_parse_environment_at,
};
use v1_compiler::cli_run::workspace_root;
use v1_compiler::extdeps_languages_dag_syntax::dag_parse_environment;

const ENVIRONMENT_MODULE_PATH: &str = "dag/extdeps/languages/dag/syntax.dag";

/// The environment decoded from live-tree source equals the one compiled into this binary.
///
/// Whole-value equality rather than a field probe: every item form, every operator row with its
/// binding powers, every keyword literal and both keyword rosters must survive, because the loader
/// hands the WHOLE environment to the tokenizer and parser.
#[test]
fn the_decoder_reproduces_the_compiled_in_environment() {
    let root = workspace_root();
    let decoded = match evaluate_environment_in(&root, "live-tree") {
        Ok(env) => env,
        Err(e) => panic!("decode refused on the live tree: {e}"),
    };
    assert_eq!(
        *decoded,
        *dag_parse_environment(),
        "environment decoded from source diverged from the compiled-in one"
    );
}

/// The oracle is not vacuous.
///
/// Without this, a decode producing an empty environment would pass the equality test on a day the
/// oracle was also empty. It pins the control itself -- the half a positive control usually leaves
/// unstated.
#[test]
fn the_compiled_in_environment_is_not_empty() {
    let compiled = dag_parse_environment();
    assert!(
        !compiled.syntax_spec.item_forms.is_empty(),
        "oracle has no item forms, so equality against it would prove nothing"
    );
    assert!(
        !compiled.syntax_spec.operators.is_empty(),
        "oracle has no operators, so equality against it would prove nothing"
    );
    assert!(
        !compiled.non_name_keywords.is_empty(),
        "oracle has no non-name keywords, so equality against it would prove nothing"
    );
}

fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap_or_else(|e| panic!("git {args:?} failed to start: {e}"));
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// THE DISCRIMINATING CASE: a committed revision whose grammar differs from the worktree's.
///
/// A scratch repository is built holding the live tree's environment closure, committed, and then
/// the worktree's copy is EDITED to rename one keyword without committing. The loader is then asked
/// for the committed revision.
///
/// What this rules out, and nothing else does: a loader that reads the worktree would return the
/// EDITED environment and fail this test, and a loader that ignored its revision argument entirely
/// would do the same. The assertion is therefore about provenance, not about shape.
#[test]
fn the_loader_reads_the_revision_not_the_worktree() {
    let root = workspace_root();
    let scratch =
        std::env::temp_dir().join(format!("gunbc-env-revision-probe-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);
    std::fs::create_dir_all(&scratch).expect("create scratch repo");

    // The whole `dag/` tree, so the closure resolves through the repository's own index.
    let status = Command::new("sh")
        .arg("-c")
        .arg(format!(
            "cd {src} && git archive --format=tar HEAD dag | tar -xf - -C {dst}",
            src = root.display(),
            dst = scratch.display()
        ))
        .status()
        .expect("materialize dag tree into scratch");
    assert!(status.success(), "materializing the scratch corpus failed");

    git(&scratch, &["init", "--quiet"]);
    git(&scratch, &["config", "user.email", "probe@example.invalid"]);
    git(&scratch, &["config", "user.name", "probe"]);
    git(&scratch, &["add", "-A"]);
    git(
        &scratch,
        &["commit", "--quiet", "-m", "baseline environment"],
    );
    let baseline = git(&scratch, &["rev-parse", "HEAD"]);

    // Now diverge the WORKTREE only: rename the `fn` item form's keyword. Uncommitted, so the
    // committed revision still carries the original spelling.
    let env_path = scratch.join(ENVIRONMENT_MODULE_PATH);
    let original = std::fs::read_to_string(&env_path).expect("read scratch environment module");
    let edited = original.replacen("keyword: \"fn\"", "keyword: \"zzfnzz\"", 1);
    assert_ne!(
        original, edited,
        "the probe could not find the `fn` keyword row to diverge, so it would prove nothing"
    );
    std::fs::write(&env_path, &edited).expect("write diverged worktree copy");

    // The committed blob and the worktree differ -- the premise of the whole probe.
    let committed_id = {
        let prev = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(&scratch).expect("chdir scratch");
        let id = blob_id_at(&baseline, ENVIRONMENT_MODULE_PATH);
        std::env::set_current_dir(prev).expect("restore cwd");
        id
    };
    assert!(
        matches!(committed_id, Ok(Some(_))),
        "the scratch revision has no environment blob, so the probe is not set up"
    );

    let loaded = {
        let prev = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(&scratch).expect("chdir scratch");
        let got = load_parse_environment_at(&baseline);
        std::env::set_current_dir(prev).expect("restore cwd");
        got
    };
    let loaded = match loaded {
        Ok(env) => env,
        Err(e) => panic!("loader refused on the scratch revision: {e}"),
    };

    let keywords: Vec<String> = loaded
        .syntax_spec
        .item_forms
        .iter()
        .map(|f| f.keyword.clone())
        .collect();
    assert!(
        keywords.iter().any(|k| k == "fn"),
        "the loader did not return the COMMITTED environment: `fn` is absent, so it read the \
         diverged worktree instead of revision {baseline}. keywords: {keywords:?}"
    );
    assert!(
        !keywords.iter().any(|k| k == "zzfnzz"),
        "the loader returned the WORKTREE environment: the uncommitted `zzfnzz` spelling reached \
         it, so the revision argument selected nothing. keywords: {keywords:?}"
    );

    let _ = std::fs::remove_dir_all(&scratch);
}
