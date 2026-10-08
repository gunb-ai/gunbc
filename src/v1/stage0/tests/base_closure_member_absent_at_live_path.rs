//! THE FLOOR'S BASE-ENVIRONMENT LOAD, ASKED FOR A BASE THAT LACKS A PATH THE LIVE CLOSURE NAMES.
//!
//! The discriminating control at the EXISTING entry point (`load_parse_environment_at`, which
//! `environment_agreement` calls for the base). Only APIs present before and after the fix are used,
//! so this file compiles and runs against both.
//!
//! The base commit is the live tree with one environment-closure member moved to another path, its
//! module header unchanged. The base's own closure is therefore complete -- it just lives at a path
//! the live tree does not use -- which is exactly gunbc#13378's shape seen from the base: the live
//! (head) closure names a file the base does not have.
//!
//! - Taking the closure from the live tree archives the live path at the base and refuses at
//!   `git archive` (`pathspec ... did not match any files`).
//! - Reading the closure from the base finds the member by its module header and loads.

use std::path::Path;
use std::process::Command;

use v1_compiler::cli_run::namespace_baseline::{
    environment_load_refusal_text, load_parse_environment_at, materialize_revision_paths,
};
use v1_compiler::cli_run::workspace_root;

/// A member of the live environment closure (`dag/extdeps/languages/dag/syntax.dag` imports it).
const MOVED_FROM: &str = "dag/extdeps/uri.dag";
const MOVED_TO: &str = "dag/extdeps/zz_moved_uri.dag";

fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("run git");
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

#[test]
fn the_base_environment_loads_when_a_live_closure_path_is_absent_at_the_base() {
    let scratch = std::env::temp_dir().join(format!(
        "gunbc-base-closure-live-path-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&scratch);
    std::fs::create_dir_all(&scratch).expect("create scratch repo");
    materialize_revision_paths(&workspace_root(), "HEAD", &scratch, &["dag"]).unwrap_or_else(|e| {
        panic!(
            "materializing the scratch corpus failed: {}",
            environment_load_refusal_text(&e)
        )
    });
    assert!(
        scratch.join(MOVED_FROM).exists(),
        "{MOVED_FROM} is not in the live tree, so the probe is not set up"
    );
    std::fs::rename(scratch.join(MOVED_FROM), scratch.join(MOVED_TO)).expect("move member");
    git(&scratch, &["init", "--quiet"]);
    git(&scratch, &["config", "user.email", "probe@example.invalid"]);
    git(&scratch, &["config", "user.name", "probe"]);
    git(&scratch, &["add", "-A"]);
    git(
        &scratch,
        &[
            "commit",
            "--quiet",
            "-m",
            "base: closure member at another path",
        ],
    );
    let base = git(&scratch, &["rev-parse", "HEAD"]);

    let outcome = load_parse_environment_at(&scratch, &base);
    let _ = std::fs::remove_dir_all(&scratch);
    if let Err(e) = outcome {
        panic!(
            "the base environment did not load, so the base's closure was not read from the base: {}",
            environment_load_refusal_text(&e)
        );
    }
}

/// A PR THAT DELETES A MODULE FROM THE CLOSURE: the base still carries a member the live (head)
/// closure no longer has. Taking the closure from the live tree materializes the base without it and
/// the base's own environment module fails to resolve; reading it from the base loads.
#[test]
fn the_base_environment_loads_a_member_the_live_closure_no_longer_has() {
    const ENVIRONMENT_MODULE_PATH: &str = "dag/extdeps/languages/dag/syntax.dag";
    const MEMBER_PATH: &str = "dag/zz_closure_probe/deleted_member.dag";
    const MEMBER_IMPORT: &str =
        "import zz_closure_probe.deleted_member { zz_closure_probe_deleted_marker }\n";
    let scratch = std::env::temp_dir().join(format!(
        "gunbc-base-closure-deleted-member-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&scratch);
    std::fs::create_dir_all(&scratch).expect("create scratch repo");
    materialize_revision_paths(&workspace_root(), "HEAD", &scratch, &["dag"]).unwrap_or_else(|e| {
        panic!(
            "materializing the scratch corpus failed: {}",
            environment_load_refusal_text(&e)
        )
    });
    let member = scratch.join(MEMBER_PATH);
    std::fs::create_dir_all(member.parent().unwrap()).expect("create member dir");
    std::fs::write(
        &member,
        "module zz_closure_probe.deleted_member\n\ndata zz_closure_probe_deleted_marker: Int = 1\n",
    )
    .expect("write member");
    let env = scratch.join(ENVIRONMENT_MODULE_PATH);
    let source = std::fs::read_to_string(&env).expect("read environment module");
    let (header, rest) = source
        .split_once('\n')
        .expect("environment module has a header line");
    std::fs::write(&env, format!("{header}\n\n{MEMBER_IMPORT}{rest}"))
        .expect("write environment module");
    git(&scratch, &["init", "--quiet"]);
    git(&scratch, &["config", "user.email", "probe@example.invalid"]);
    git(&scratch, &["config", "user.name", "probe"]);
    git(&scratch, &["add", "-A"]);
    git(
        &scratch,
        &[
            "commit",
            "--quiet",
            "-m",
            "base: carries a member the head deleted",
        ],
    );
    let base = git(&scratch, &["rev-parse", "HEAD"]);

    let outcome = load_parse_environment_at(&scratch, &base);
    let _ = std::fs::remove_dir_all(&scratch);
    if let Err(e) = outcome {
        panic!(
            "the base environment did not load, so the base's own closure member was not read: {}",
            environment_load_refusal_text(&e)
        );
    }
}
