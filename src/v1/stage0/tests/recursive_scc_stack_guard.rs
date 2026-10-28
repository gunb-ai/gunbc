//! Every member of a recursive strongly connected component gets the bounded-stack guard, not only
//! the self-recursive ones (gunbc dashboard node://adhoc-9743d9e6-426; RFM
//! gunbc.recurring_failure_mode.a_stack_guard_placed_on_self_recursion_only_aborts_a_mutual_scc).
//!
//! A recursive SCC has no interior and no exterior: a member that only calls INTO the cycle runs out
//! of stack exactly as surely as the member that closes it. `a_even` calls `a_odd` and `a_odd` calls
//! `a_even`; neither calls itself, so the emitter's old per-function self-call test answered "not
//! recursive" about both and emitted them unguarded. Their recursive call is not in tail position
//! (each concatenates onto the callee's result), so no loop lowering applies and every level is a
//! real frame.
//!
//! Two tests, two distinct observables, so a repair that only moves one of them cannot pass:
//!   * `..._guarded_and_no_one_else_is` reads the EMITTED TEXT -- both members of the pair carry
//!     `stacker::maybe_grow`, the self-recursive control still does, and the non-recursive control
//!     still does not (the verdict is membership-precise, not a blanket).
//!   * `deep_mutual_recursion_survives_a_small_stack` ignores the text and EXECUTES the compiled
//!     specimen on a deliberately small 1 MiB thread stack, where the unguarded pair dies of stack
//!     overflow instead of answering.
//! Pre-repair both fail; post-repair both pass.
//!
//! Run with cargo test -p v1-compiler --test recursive_scc_stack_guard. Required CI does not execute
//! Rust integration tests (gunbc.rung_drop v2_native_route_off_the_merge_path), so this is
//! on-demand evidence; the RFM row names the required-path control that would climb it a rung.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::rc::Rc;

use v1_compiler::v1_compiler_artifact::RenderTarget;
use v1_compiler::v1_compiler_compile::{compile_sources, SourceFile};
use v1_compiler::v1_std_core::diagnostic_to_message;

// `acc` is threaded and concatenated on the way OUT of every call, so the recursive call is not in
// tail position: neither member can be lowered to a loop, and each level must keep a frame.
const SOURCE: &str = r#"module guard_probe

fn a_even(n: Int, acc: String) -> String {
  if n == 0 { acc } else { concat(a_odd(n - 1, acc), "a") }
}

fn a_odd(n: Int, acc: String) -> String {
  if n == 0 { acc } else { concat(a_even(n - 1, acc), "b") }
}

fn self_depth(n: Int, acc: String) -> String {
  if n == 0 { acc } else { concat(self_depth(n - 1, acc), "s") }
}

fn plain(n: Int) -> Int {
  n + 1
}
"#;

// Deep enough that an unguarded 1 MiB stack cannot hold it, shallow enough that a guarded one grows
// through it: the pre-repair specimen aborted at 400,000 on the main thread and answered guarded.
const DEPTH: i64 = 60_000;

struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn emit_probe() -> String {
    let sources = vec![Rc::new(SourceFile {
        path: "guard_probe.dag".into(),
        content: SOURCE.into(),
    })];
    let result = compile_sources(Rc::new(sources.iter().cloned().collect()), RenderTarget::Rust);
    let modules: Vec<_> = result
        .files
        .iter()
        .filter(|file| file.path.ends_with("guard_probe.rs"))
        .collect();
    assert_eq!(
        modules.len(),
        1,
        "emission did not produce the specimen: {:?}",
        (
            result
                .files
                .iter()
                .map(|f| f.path.clone())
                .collect::<Vec<_>>(),
            result
                .diagnostics
                .iter()
                .map(|d| diagnostic_to_message(d.diagnostic.clone()))
                .take(8)
                .collect::<Vec<_>>()
        )
    );
    modules[0].content.clone()
}

fn fn_text<'a>(emitted: &'a str, name: &str) -> &'a str {
    let at = emitted
        .find(&format!("fn {name}("))
        .unwrap_or_else(|| panic!("no fn {name} in emitted module"));
    let rest = &emitted[at..];
    let end = rest[1..]
        .find("\nfn ")
        .or_else(|| rest[1..].find("\npub fn "))
        .map(|e| e + 1)
        .unwrap_or(rest.len());
    &rest[..end]
}

// Cargo leaves one rlib per build of the crate in deps/, and nothing names which one this test
// executable was linked against, so more than one is a refusal and not a pick.
fn unique_rlib(deps: &Path, name: &str) -> PathBuf {
    let prefix = format!("lib{name}-");
    let matches: Vec<_> = std::fs::read_dir(deps)
        .expect("read test dependency directory")
        .map(|entry| entry.expect("read dependency entry").path())
        .filter(|path| {
            let filename = path.file_name().unwrap().to_string_lossy();
            filename.starts_with(&prefix) && filename.ends_with(".rlib")
        })
        .collect();
    assert_eq!(
        matches.len(),
        1,
        "ambiguous or absent {name} artifact: {matches:?}; remove the stale rlibs (cargo clean -p v1-compiler) and rerun"
    );
    matches[0].clone()
}

fn compile_and_run(emitted: &str, body: &str) -> std::process::Output {
    let root = std::env::temp_dir().join(format!(
        "gunbc-scc-guard-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    std::fs::create_dir(&root).expect("create isolated specimen directory");
    let scratch = Scratch(root);
    std::fs::write(scratch.0.join("guard_probe.rs"), emitted).expect("retain emitted bytes");
    std::fs::write(scratch.0.join("main.rs"), body).expect("write harness around emitted module");
    let executable = std::env::current_exe().expect("locate test executable");
    let deps = executable.parent().expect("test dependency directory");
    let built = Command::new("rustc")
        .arg("--edition=2021")
        .arg(scratch.0.join("main.rs"))
        .arg("-L")
        .arg(format!("dependency={}", deps.display()))
        .arg("--extern")
        .arg(format!(
            "v1_compiler={}",
            unique_rlib(deps, "v1_compiler").display()
        ))
        .arg("--extern")
        .arg(format!("im={}", unique_rlib(deps, "im").display()))
        .arg("--extern")
        .arg(format!("serde={}", unique_rlib(deps, "serde").display()))
        .arg("--extern")
        .arg(format!(
            "stacker={}",
            unique_rlib(deps, "stacker").display()
        ))
        .arg("-o")
        .arg(scratch.0.join("specimen"))
        .output()
        .expect("invoke rustc on emitted specimen");
    assert!(
        built.status.success(),
        "emitted Rust refused: {}",
        String::from_utf8_lossy(&built.stderr)
    );
    Command::new(scratch.0.join("specimen"))
        .output()
        .expect("execute compiled specimen")
}

fn small_stack_probe(call: &str) -> String {
    format!(
        r#"pub use v1_compiler::*;
mod guard_probe;
fn main() {{
    use guard_probe::*;
    let child = std::thread::Builder::new()
        .stack_size(1 << 20)
        .spawn(|| {call})
        .expect("spawn small-stack thread");
    let out = child.join().expect("recursion survived its own depth");
    assert_eq!(out.chars().count(), {DEPTH}usize, "recursion answered wrongly");
}}
"#
    )
}

#[test]
fn every_member_of_a_recursive_component_is_guarded_and_no_one_else_is() {
    let emitted = emit_probe();
    // The mutual pair is ONE component: both members guarded or neither. Pre-repair both were
    // unguarded, because each member's own body names only its neighbour.
    for name in ["a_even", "a_odd", "self_depth"] {
        assert!(
            fn_text(&emitted, name).contains("maybe_grow"),
            "{name} is a member of a recursive component and must carry the bounded-stack guard\n{}",
            fn_text(&emitted, name)
        );
    }
    // Non-recursive control: a function in no recursive component gets no guard. Without this the
    // guard set could pass by guarding everything.
    assert!(
        !fn_text(&emitted, "plain").contains("maybe_grow"),
        "plain is in no recursive component and must not be guarded\n{}",
        fn_text(&emitted, "plain")
    );
}

#[test]
fn deep_mutual_recursion_survives_a_small_stack() {
    let emitted = emit_probe();
    let run = compile_and_run(&emitted, &small_stack_probe(&format!("a_even({DEPTH}, String::new())")));
    assert!(
        run.status.success(),
        "deep mutual recursion overflowed the stack (unguarded SCC member): status={}, stderr={}",
        run.status,
        String::from_utf8_lossy(&run.stderr)
    );
    // The self-recursive control is the pre-existing case and must not regress.
    let self_run = compile_and_run(
        &emitted,
        &small_stack_probe(&format!("self_depth({DEPTH}, String::new())")),
    );
    assert!(
        self_run.status.success(),
        "self recursion regressed: status={}, stderr={}",
        self_run.status,
        String::from_utf8_lossy(&self_run.stderr)
    );
}
