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

use v1_compiler::cli_run::namespace_baseline::{
    blob_id_at, environment_load_refusal_text, evaluate_environment_in, kernel_set_serves_both,
    load_parse_environment_at, materialize_revision_paths,
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
        Err(e) => panic!(
            "decode refused on the live tree: {}",
            environment_load_refusal_text(&e)
        ),
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

    // The whole `dag/` tree, so the closure resolves through the repository's own index -- acquired
    // through the loader's own route rather than a hand-shell pipeline beside it.
    materialize_revision_paths(&root, "HEAD", &scratch, &["dag"]).unwrap_or_else(|e| {
        panic!(
            "materializing the scratch corpus failed: {}",
            environment_load_refusal_text(&e)
        )
    });

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

    // The committed blob exists -- the premise of the whole probe. The repository is named
    // explicitly rather than reached through the process cwd: `workspace_root()` memoizes the cwd
    // on first use and the other tests in this binary run concurrently, so a chdir here would
    // have made their colour depend on thread interleaving.
    let committed_id = blob_id_at(&scratch, &baseline, ENVIRONMENT_MODULE_PATH);
    assert!(
        matches!(committed_id, Ok(Some(_))),
        "the scratch revision has no environment blob, so the probe is not set up"
    );

    let loaded = match load_parse_environment_at(&scratch, &baseline) {
        Ok(env) => env,
        Err(e) => panic!(
            "loader refused on the scratch revision: {}",
            environment_load_refusal_text(&e)
        ),
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

const KERNEL_TYPES_PATH: &str = "dag/std/types.dag";

/// THE KERNEL GUARD DECIDES AT THE GRAIN OF THE NAME SET, NOT THE FILE'S BYTES.
///
/// Three revisions of one scratch repository, each compared against the live-tree baseline:
///
/// - a function-body and annotation edit to `dag/std/types.dag` changes the blob but cannot change
///   the kernel names, so the guard must answer `true`. The earlier whole-blob guard answered `false`
///   here and refused every such change repo-wide -- this is the discriminating RED for that shape;
/// - a revision that adds a kernel name must answer `false`: the single head map cannot speak for it;
/// - a revision whose `types.dag` does not declare `kernel_type_set` must REFUSE rather than
///   substitute this binary's set.
#[test]
fn the_kernel_guard_compares_names_not_bytes() {
    let root = workspace_root();
    let scratch =
        std::env::temp_dir().join(format!("gunbc-kernel-set-probe-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);
    std::fs::create_dir_all(&scratch).expect("create scratch repo");
    materialize_revision_paths(&root, "HEAD", &scratch, &["dag"]).unwrap_or_else(|e| {
        panic!(
            "materializing the scratch corpus failed: {}",
            environment_load_refusal_text(&e)
        )
    });
    git(&scratch, &["init", "--quiet"]);
    git(&scratch, &["config", "user.email", "probe@example.invalid"]);
    git(&scratch, &["config", "user.name", "probe"]);
    git(&scratch, &["add", "-A"]);
    git(&scratch, &["commit", "--quiet", "-m", "baseline"]);
    let baseline = git(&scratch, &["rev-parse", "HEAD"]);

    let types_path = scratch.join(KERNEL_TYPES_PATH);
    let original = std::fs::read_to_string(&types_path).expect("read scratch types.dag");
    let commit_variant = |label: &str, text: &str| -> String {
        std::fs::write(&types_path, text).expect("write types.dag variant");
        git(&scratch, &["commit", "--quiet", "-am", label]);
        let id = git(&scratch, &["rev-parse", "HEAD"]);
        git(
            &scratch,
            &["checkout", "--quiet", &baseline, "--", KERNEL_TYPES_PATH],
        );
        git(
            &scratch,
            &["commit", "--quiet", "--allow-empty", "-am", "restore"],
        );
        id
    };

    // Body-only: the kernel set is untouched, the bytes are not.
    let body_edit = original.replacen("    Absent => false\n", "    Absent => (1 == 2)\n", 1);
    assert_ne!(
        original, body_edit,
        "the probe could not find is_kernel_type's body to edit"
    );
    let body_rev = commit_variant("body-only edit", &body_edit);
    assert_ne!(
        blob_id_at(&scratch, &baseline, KERNEL_TYPES_PATH).ok(),
        blob_id_at(&scratch, &body_rev, KERNEL_TYPES_PATH).ok(),
        "the body edit did not change the blob, so the probe does not exercise the name-set arm"
    );
    match kernel_set_serves_both(&scratch, &body_rev, &baseline) {
        Ok(true) => {}
        other => panic!(
            "a function-body edit to {KERNEL_TYPES_PATH} was judged to change the kernel set: {:?}",
            other.map_err(|e| environment_load_refusal_text(&e))
        ),
    }

    // A kernel name added: the base's set is not this binary's.
    let widened = original.replacen("\"Bytes\": true", "\"Bytes\": true, \"ZzProbe\": true", 1);
    assert_ne!(
        original, widened,
        "the probe could not find the kernel set to widen"
    );
    let widened_rev = commit_variant("kernel name added", &widened);
    match kernel_set_serves_both(&scratch, &widened_rev, &baseline) {
        Ok(false) => {}
        other => panic!(
            "a base revision declaring an extra kernel name was judged served by this binary's set: {:?}",
            other.map_err(|e| environment_load_refusal_text(&e))
        ),
    }

    // The set is not declared at all: refuse, never substitute.
    let renamed = original.replacen("data kernel_type_set:", "data zz_kernel_type_set:", 1);
    assert_ne!(
        original, renamed,
        "the probe could not find the kernel set declaration"
    );
    let unreadable_rev = commit_variant(
        "kernel set undeclared",
        &renamed.replace("map_get(kernel_type_set,", "map_get(zz_kernel_type_set,"),
    );
    assert!(
        kernel_set_serves_both(&scratch, &unreadable_rev, &baseline).is_err(),
        "a base revision with no kernel_type_set declaration was answered rather than refused"
    );

    let _ = std::fs::remove_dir_all(&scratch);
}
