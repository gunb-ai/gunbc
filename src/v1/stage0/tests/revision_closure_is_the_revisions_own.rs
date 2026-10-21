//! A REVISION'S ENVIRONMENT CLOSURE IS READ FROM THAT REVISION.
//!
//! The floor's base reconstruction used to take the parse-environment closure (and the kernel-types
//! closure) from the LIVE tree and archive those paths at the base. Two defects followed, one per
//! direction of difference:
//!
//! 1. A head that ADDS a closure member asked the base for a file it never had, and refused at
//!    `git archive` (gunbc#13378's floor: `pathspec 'dag/std/unicode/scalar.dag' did not match`).
//! 2. A base whose closure has a member the head DROPPED was materialized without it.
//!
//! Each control below builds a scratch repository whose two commits differ on exactly one closure
//! member, and asks each commit for its own closure. A loader that derived the closure from any
//! single tree would answer both commits with the same set, so the two-sided assertions are what
//! discriminate; the load assertions show the environment is evaluated from that closure.

use std::path::{Path, PathBuf};
use std::process::Command;

use v1_compiler::cli_run::namespace_baseline::{
    environment_load_refusal_text, load_parse_environment_at, materialize_revision_paths,
    revision_closure_paths, EnvironmentLoadRefusal,
};
use v1_compiler::cli_run::workspace_root;

const ENVIRONMENT_MODULE_PATH: &str = "dag/extdeps/languages/dag/syntax.dag";
const MEMBER_PATH: &str = "dag/zz_closure_probe/member.dag";
const MEMBER_SOURCE: &str =
    "module zz_closure_probe.member\n\ndata zz_closure_probe_marker: Int = 1\n";
const MEMBER_IMPORT: &str = "import zz_closure_probe.member { zz_closure_probe_marker }\n";

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

/// A scratch repository holding the live `dag/` tree as its first commit.
fn scratch_repo(tag: &str) -> PathBuf {
    let scratch = std::env::temp_dir().join(format!(
        "gunbc-revision-closure-{tag}-{}",
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
    git(&scratch, &["init", "--quiet"]);
    git(&scratch, &["config", "user.email", "probe@example.invalid"]);
    git(&scratch, &["config", "user.name", "probe"]);
    scratch
}

fn commit(scratch: &Path, message: &str) -> String {
    git(scratch, &["add", "-A"]);
    git(scratch, &["commit", "--quiet", "-m", message]);
    git(scratch, &["rev-parse", "HEAD"])
}

/// Add the probe member and make the environment module import it.
fn add_member(scratch: &Path) {
    let member = scratch.join(MEMBER_PATH);
    std::fs::create_dir_all(member.parent().unwrap()).expect("create member dir");
    std::fs::write(&member, MEMBER_SOURCE).expect("write member");
    let env = scratch.join(ENVIRONMENT_MODULE_PATH);
    let source = std::fs::read_to_string(&env).expect("read environment module");
    let (header, rest) = source
        .split_once('\n')
        .expect("environment module has a header line");
    std::fs::write(&env, format!("{header}\n\n{MEMBER_IMPORT}{rest}"))
        .expect("write environment module");
}

/// Remove the probe member and its import.
fn drop_member(scratch: &Path) {
    std::fs::remove_file(scratch.join(MEMBER_PATH)).expect("remove member");
    let env = scratch.join(ENVIRONMENT_MODULE_PATH);
    let source = std::fs::read_to_string(&env).expect("read environment module");
    std::fs::write(&env, source.replacen(MEMBER_IMPORT, "", 1)).expect("write environment module");
}

fn closure(scratch: &Path, revision: &str) -> std::collections::BTreeSet<String> {
    revision_closure_paths(scratch, revision, ENVIRONMENT_MODULE_PATH).unwrap_or_else(|e| {
        panic!(
            "the closure walk refused at {revision}: {}",
            environment_load_refusal_text(&e)
        )
    })
}

fn assert_loads(scratch: &Path, revision: &str, which: &str) {
    if let Err(e) = load_parse_environment_at(scratch, revision) {
        panic!(
            "the {which} revision's environment did not load: {}",
            environment_load_refusal_text(&e)
        );
    }
}

/// RED BEFORE THE FIX: the head adds a closure member the base does not have. The base is asked for
/// its own closure, which lacks the member, and its environment loads; the head's closure carries it.
#[test]
fn a_member_the_head_adds_is_not_demanded_of_the_base() {
    let scratch = scratch_repo("head-adds");
    let base = commit(&scratch, "base");
    add_member(&scratch);
    let head = commit(&scratch, "head adds a closure member");

    let base_closure = closure(&scratch, &base);
    let head_closure = closure(&scratch, &head);
    assert!(
        !base_closure.contains(MEMBER_PATH),
        "the base's closure names {MEMBER_PATH}, a file the base does not contain"
    );
    assert!(
        head_closure.contains(MEMBER_PATH),
        "the head's closure omits {MEMBER_PATH}, which its environment module imports"
    );
    assert_loads(&scratch, &base, "base");
    assert_loads(&scratch, &head, "head");
    let _ = std::fs::remove_dir_all(&scratch);
}

/// A base whose closure carries a member the head dropped: the base's closure keeps it and its
/// environment loads from it.
#[test]
fn a_member_only_the_base_has_is_read_from_the_base() {
    let scratch = scratch_repo("base-only");
    add_member(&scratch);
    let base = commit(&scratch, "base carries a closure member");
    drop_member(&scratch);
    let head = commit(&scratch, "head drops it");

    assert!(
        closure(&scratch, &base).contains(MEMBER_PATH),
        "the base's closure omits {MEMBER_PATH}, which the base's environment module imports"
    );
    assert!(
        !closure(&scratch, &head).contains(MEMBER_PATH),
        "the head's closure names {MEMBER_PATH}, which the head no longer has"
    );
    assert_loads(&scratch, &base, "base");
    let _ = std::fs::remove_dir_all(&scratch);
}

/// An import no file at the revision declares refuses as data: the missing module and its importer.
#[test]
fn an_import_no_file_declares_refuses_with_the_missing_module() {
    let scratch = scratch_repo("missing");
    commit(&scratch, "base");
    add_member(&scratch);
    std::fs::remove_file(scratch.join(MEMBER_PATH)).expect("remove member, keep its import");
    let broken = commit(&scratch, "environment imports a module no file declares");

    match load_parse_environment_at(&scratch, &broken) {
        Err(EnvironmentLoadRefusal::ClosureMemberMissing {
            revision,
            module,
            imported_by,
        }) => {
            assert_eq!(revision, broken);
            assert_eq!(module, "zz_closure_probe.member");
            assert_eq!(imported_by, ENVIRONMENT_MODULE_PATH);
        }
        Err(other) => panic!(
            "expected ClosureMemberMissing, got: {}",
            environment_load_refusal_text(&other)
        ),
        Ok(_) => panic!("the loader answered an environment whose closure is not formable"),
    }
    let _ = std::fs::remove_dir_all(&scratch);
}
