//! Discriminating witness: `first` on `List<ContentHash>` must return an optional
//! element (`CardOptional`), matching `List<String>`. A bare branded element lets
//! `match ... { Present {..} Absent => ... }` bind the wrong shape silently.

use std::fs;

use crate::helpers::workspace_root;
use v1_compiler::cli_run::{make_eval_context, resolve_entry_graph, run_claim, ClaimOutcome};
use v1_compiler::v1_interpreter::ExecutionMode;

fn run_fixture(files: &[(&str, &str)], entry: &str, function: &str) -> ClaimOutcome {
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let seq = SEQ.fetch_add(1, Ordering::SeqCst);
    let dir = workspace_root()
        .join("target")
        .join(format!("list-first-branded-{seq}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("fixture dir");
    for (name, src) in files {
        fs::write(dir.join(name), src).unwrap_or_else(|e| panic!("write {name}: {e}"));
    }
    let roots = vec![
        dir.to_string_lossy().into_owned(),
        workspace_root().join("dag").to_string_lossy().into_owned(),
        workspace_root()
            .join("src/v2")
            .to_string_lossy()
            .into_owned(),
    ];
    let entry_path = dir.join(entry).to_string_lossy().into_owned();
    let (graph, si) = resolve_entry_graph(&roots, &entry_path).expect("fixture resolve");
    let ctx = make_eval_context(&graph, si, ExecutionMode::Wet);
    let outcome = run_claim(&ctx, function);
    let _ = fs::remove_dir_all(&dir);
    outcome
}

const PROBE: &str = r#"module probe.first_branded
import std.types { ContentHash, List, String }
import std.logic { Bool }
import v2.std.collection { Present, Absent }

fn hash_list() -> List<ContentHash> {
  ["a" as ContentHash]
}

fn empty_hash_list() -> List<ContentHash> {
  []
}

fn string_list() -> List<String> {
  ["hello"]
}

fn empty_string_list() -> List<String> {
  []
}

fn first_hash_method(xs: List<ContentHash>) -> ContentHash? {
  xs.first()
}

fn first_string_method(xs: List<String>) -> String? {
  xs.first()
}

fn first_hash_pipe(xs: List<ContentHash>) -> ContentHash? {
  xs |> first
}

fn first_string_pipe(xs: List<String>) -> String? {
  xs |> first
}

fn nonempty_hash_method() -> Bool {
  match hash_list().first() {
    Present { value: h } => h == ("a" as ContentHash)
    Absent => false
  }
}

fn empty_hash_method() -> Bool {
  match empty_hash_list().first() {
    Present { value: _ } => false
    Absent => true
  }
}

fn nonempty_string_method() -> Bool {
  match string_list().first() {
    Present { value: s } => s == "hello"
    Absent => false
  }
}

fn empty_string_method() -> Bool {
  match empty_string_list().first() {
    Present { value: _ } => false
    Absent => true
  }
}

fn nonempty_hash_pipe() -> Bool {
  match hash_list() |> first {
    Present { value: h } => h == ("a" as ContentHash)
    Absent => false
  }
}

fn empty_hash_pipe() -> Bool {
  match empty_hash_list() |> first {
    Present { value: _ } => false
    Absent => true
  }
}

fn nonempty_string_pipe() -> Bool {
  match string_list() |> first {
    Present { value: s } => s == "hello"
    Absent => false
  }
}

fn empty_string_pipe() -> Bool {
  match empty_string_list() |> first {
    Present { value: _ } => false
    Absent => true
  }
}

test fn probe_nonempty_hash_method() -> Bool { nonempty_hash_method() }
test fn probe_empty_hash_method() -> Bool { empty_hash_method() }
test fn probe_nonempty_string_method() -> Bool { nonempty_string_method() }
test fn probe_empty_string_method() -> Bool { empty_string_method() }
test fn probe_nonempty_hash_pipe() -> Bool { nonempty_hash_pipe() }
test fn probe_empty_hash_pipe() -> Bool { empty_hash_pipe() }
test fn probe_nonempty_string_pipe() -> Bool { nonempty_string_pipe() }
test fn probe_empty_string_pipe() -> Bool { empty_string_pipe() }
"#;

#[test]
fn list_first_content_hash_method_nonempty() {
    let outcome = run_fixture(
        &[("probe.dag", PROBE)],
        "probe.dag",
        "probe_nonempty_hash_method",
    );
    assert!(
        matches!(outcome, ClaimOutcome::Pass),
        "List<ContentHash>.first() nonempty method: {outcome:?}"
    );
}

#[test]
fn list_first_content_hash_method_empty() {
    let outcome = run_fixture(
        &[("probe.dag", PROBE)],
        "probe.dag",
        "probe_empty_hash_method",
    );
    assert!(
        matches!(outcome, ClaimOutcome::Pass),
        "List<ContentHash>.first() empty method: {outcome:?}"
    );
}

#[test]
fn list_first_string_method_nonempty() {
    let outcome = run_fixture(
        &[("probe.dag", PROBE)],
        "probe.dag",
        "probe_nonempty_string_method",
    );
    assert!(
        matches!(outcome, ClaimOutcome::Pass),
        "List<String>.first() nonempty method control: {outcome:?}"
    );
}

#[test]
fn list_first_string_method_empty() {
    let outcome = run_fixture(
        &[("probe.dag", PROBE)],
        "probe.dag",
        "probe_empty_string_method",
    );
    assert!(
        matches!(outcome, ClaimOutcome::Pass),
        "List<String>.first() empty method control: {outcome:?}"
    );
}

#[test]
fn list_first_content_hash_pipe_nonempty() {
    let outcome = run_fixture(
        &[("probe.dag", PROBE)],
        "probe.dag",
        "probe_nonempty_hash_pipe",
    );
    assert!(
        matches!(outcome, ClaimOutcome::Pass),
        "List<ContentHash> |> first nonempty pipe: {outcome:?}"
    );
}

#[test]
fn list_first_content_hash_pipe_empty() {
    let outcome = run_fixture(
        &[("probe.dag", PROBE)],
        "probe.dag",
        "probe_empty_hash_pipe",
    );
    assert!(
        matches!(outcome, ClaimOutcome::Pass),
        "List<ContentHash> |> first empty pipe: {outcome:?}"
    );
}

#[test]
fn list_first_string_pipe_nonempty() {
    let outcome = run_fixture(
        &[("probe.dag", PROBE)],
        "probe.dag",
        "probe_nonempty_string_pipe",
    );
    assert!(
        matches!(outcome, ClaimOutcome::Pass),
        "List<String> |> first nonempty pipe control: {outcome:?}"
    );
}

#[test]
fn list_first_string_pipe_empty() {
    let outcome = run_fixture(
        &[("probe.dag", PROBE)],
        "probe.dag",
        "probe_empty_string_pipe",
    );
    assert!(
        matches!(outcome, ClaimOutcome::Pass),
        "List<String> |> first empty pipe control: {outcome:?}"
    );
}
