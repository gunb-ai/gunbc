//! R1 closeout item TWO, claim B: the roster<->handler-arm bijection stays exhaustive-by-
//! construction over the REAL `gunbc.v1_interpreter_primitive_surface` roster and the REAL
//! `v1_interpreter.rs` handler-arm list -- not a mirrored shape in a disconnected fixture crate.
//!
//! `interpreter_dispatch_bijection_compile_red.rs` proves rustc refuses a non-exhaustive match
//! and an unresolvable macro invocation in the abstract (claim A). That is necessary but not
//! sufficient: a fixture that only reproduces the *shape* stays green even if the real production
//! macro in `v1_interpreter.rs` regains a wildcard catch-all -- exactly the state `origin/main`
//! is in today for the bridge-family sites, and exactly the DESIGN.md §5 tell ("a check
//! satisfiable by editing the declaration while the realization still lies"). This file tests the
//! real thing: it perturbs a clone of the actual roster (or the actual hand-authored arm list),
//! regenerates `v1_interpreter_dispatch_generated.rs` via the real `gunbc run ... main_wet`
//! pipeline, and `cargo build`s the real `v1-compiler` crate.
//!
//! EXPENSIVE by construction (clone + real regen + real full-crate `cargo build`, twice) --
//! `#[ignore]`d per the `route_a_emit_fresh_cargo_green_test.rs` precedent, and per the explicit
//! ruling that a cargo subprocess must never sit in the per-PR CI discovery lane.
//!
//! ENROLLMENT: NOT the falsifier cadence -- executes only by hand today
//! (`cargo test --release -p v1-compiler --test interpreter_dispatch_bijection_real_roster_red --
//! --ignored`). The falsifier cadence's enrollment axis (`gunbc.commit_workflow`
//! `CommitWitnessClaim`/`CommitSpecGate`, `bare_deenrollment_wall_note`) models only .dag-sourced
//! witnesses; it has no member kind for a hand-authored Rust `#[test]`. This is a declared,
//! named gap, not a silent one -- see `gunbc.commit_workflow`
//! `rust_level_evidence_enrollment_axis_gap_note` for the full accounting and dissolution
//! trigger. Do not read this test's presence in the tree as "enrolled on the falsifier cadence".

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn workspace_root() -> PathBuf {
    // CARGO_MANIFEST_DIR for this crate is <repo>/src/v1/stage0.
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("workspace root")
}

fn released_gunbc_bin(ws: &Path) -> PathBuf {
    let bin = ws.join("target/release/gunbc");
    assert!(
        bin.exists(),
        "expected a built release gunbc binary at {}; run `cargo build --release -p v1-compiler --bin gunbc` first",
        bin.display()
    );
    bin
}

/// A local `git clone` (no `target/`, no history rewrite) so the perturbation never touches the
/// real worktree. Distinct temp dirs per direction so the two REDs cannot interfere.
fn clone_workspace(ws: &Path, tag: &str) -> PathBuf {
    let dest = std::env::temp_dir().join(format!(
        "gunbc_dispatch_bijection_real_{tag}_{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dest);
    let out = Command::new("git")
        .args(["clone", "--local", "--no-hardlinks"])
        .arg(ws)
        .arg(&dest)
        .output()
        .expect("git clone");
    assert!(
        out.status.success(),
        "git clone of workspace failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    dest
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn write(path: &Path, content: &str) {
    std::fs::write(path, content).unwrap_or_else(|e| panic!("write {}: {e}", path.display()));
}

/// Runs the real regen entrypoint (`dag/tools/generated_artifact_gate.dag` `main_wet`) inside
/// `clone_root`, using the already-built `gunbc` binary from the real workspace (the binary's own
/// behavior is unaffected by the roster perturbation -- it only interprets `.dag` source at
/// `--source-root`, which points at the clone's mutated copy).
fn regen_generated_artifacts(gunbc_bin: &Path, clone_root: &Path) -> Output {
    Command::new(gunbc_bin)
        .current_dir(clone_root)
        .args([
            "run",
            "--source-root",
            "dag",
            "--source-root",
            "src/v2",
            "--entry",
            "dag/tools/generated_artifact_gate.dag",
            "--function",
            "main_wet",
        ])
        .output()
        .expect("gunbc run main_wet")
}

fn cargo_build_v1_compiler(clone_root: &Path) -> Output {
    Command::new("cargo")
        .current_dir(clone_root)
        .args(["build", "-p", "v1-compiler", "--lib"])
        .env("RUSTC_WRAPPER", "")
        .output()
        .expect("cargo build -p v1-compiler")
}

const ROSTER_REL: &str = "dag/gunbc/v1_interpreter_primitive_surface.dag";
const INTERPRETER_REL: &str = "src/v1/stage0/src/v1_interpreter.rs";

/// Direction 1: add a roster row at `eval_builtin_inner` with a fresh identity/spelling that has
/// no corresponding hand-authored `arm` entry in `v1_interpreter.rs`. Regeneration mints a new
/// `EvalBuiltinArm` variant and a new `eval_builtin_inner_arm!` macro rule for it (both derived
/// from the roster), but the hand-authored `match arm { ... }` in `v1_interpreter.rs` -- which
/// item ONE made exhaustive by construction, no wildcard -- was never told about it. That is
/// exactly the invalid state item ONE's wall exists to make unwritable: real construction wall,
/// real regression, real `cargo build` failure.
#[test]
#[ignore = "Expensive: git clone + real gunbc regen + real `cargo build -p v1-compiler` (falsifier cadence, not per-PR CI)"]
fn w_red_real_roster_row_with_no_real_handler_arm_refuses_compile() {
    let ws = workspace_root();
    let gunbc_bin = released_gunbc_bin(&ws);
    let clone = clone_workspace(&ws, "missing_handler");

    let roster_path = clone.join(ROSTER_REL);
    let roster_src = read(&roster_path);
    let marker =
        "fn v1_interpreter_authored_roster_arms() -> List<InterpreterPrimitiveDispatchArm> {\n  [";
    assert!(
        roster_src.contains(marker),
        "roster function shape changed; update this fixture's insertion point"
    );
    let injected_row = concat!(
        "\n    InterpreterPrimitiveDispatchArm {\n",
        "      arm: InterpreterPrimitiveArmId { identity: \"free_call.dispatch_bijection_missing_handler_probe\" },\n",
        "      form: FreeCall,\n",
        "      authored_spelling: \"dispatch_bijection_missing_handler_probe\",\n",
        "      realization_module: \"v1_interpreter\",\n",
        "      dispatch_symbol: \"eval_builtin_inner\",\n",
        "      dispatch_emit_site: EvalBuiltinInnerSite,\n",
        "      enumeration: AuthoredInRoster,\n",
        "    },",
    );
    let mutated = roster_src.replacen(marker, &format!("{marker}{injected_row}"), 1);
    write(&roster_path, &mutated);

    let regen = regen_generated_artifacts(&gunbc_bin, &clone);
    assert!(
        regen.status.success(),
        "regen (main_wet) must itself succeed -- the mismatch is a Rust-side compile defect, \
         not a `.dag`-level refusal; stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&regen.stdout),
        String::from_utf8_lossy(&regen.stderr)
    );

    let build = cargo_build_v1_compiler(&clone);
    let stderr = String::from_utf8_lossy(&build.stderr);
    assert!(
        !build.status.success(),
        "deliberate-red: a real roster row with no real handler arm must refuse cargo build; \
         regen stdout was:\n{}",
        String::from_utf8_lossy(&regen.stdout)
    );
    assert!(
        stderr.contains("E0004") || stderr.contains("non-exhaustive"),
        "must refuse specifically on non-exhaustive-match, not some other defect; stderr:\n{stderr}"
    );
    assert!(
        stderr.contains("FreeCallDispatchBijectionMissingHandlerProbe"),
        "the refusal should name the exact generated variant the missing handler leaves \
         unmatched, or this is refusing on an unrelated defect; stderr:\n{stderr}"
    );

    let _ = std::fs::remove_dir_all(&clone);
}

/// Direction 2 (mirror defect): add a hand-authored `arm "..." { "..." } => ...,` entry directly
/// into `v1_interpreter.rs`'s `eval_builtin_inner` arm list, with NO corresponding roster row.
/// Regeneration leaves the generated `eval_builtin_inner_arm!` macro without a rule for that
/// identity (the roster never named it), so the macro invocation the orphan arm compiles down to
/// fails to expand -- an orphan handler nobody's roster row grounds.
#[test]
#[ignore = "Expensive: git clone + real gunbc regen + real `cargo build -p v1-compiler` (falsifier cadence, not per-PR CI)"]
fn w_red_real_handler_arm_with_no_real_roster_row_refuses_compile() {
    let ws = workspace_root();
    let gunbc_bin = released_gunbc_bin(&ws);
    let clone = clone_workspace(&ws, "orphan_handler");

    let interpreter_path = clone.join(INTERPRETER_REL);
    let interpreter_src = read(&interpreter_path);
    let marker = "arm \"free_call.parse_stage0_cargo_manifest_bins\" { \"parse_stage0_cargo_manifest_bins\" }";
    assert!(
        interpreter_src.contains(marker),
        "eval_builtin_inner arm-list shape changed; update this fixture's insertion point"
    );
    let orphan_arm = "arm \"free_call.dispatch_bijection_orphan_probe\" { \"dispatch_bijection_orphan_probe\" } => Ok(Some(Value::Bool(true))),\n            ";
    let mutated = interpreter_src.replacen(marker, &format!("{orphan_arm}{marker}"), 1);
    write(&interpreter_path, &mutated);

    // Roster is untouched: regen reproduces today's committed generated file byte-for-byte, so
    // the orphan identity has no macro rule to expand against.
    let regen = regen_generated_artifacts(&gunbc_bin, &clone);
    assert!(
        regen.status.success(),
        "regen (main_wet) must itself succeed -- roster is unperturbed on this direction; \
         stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&regen.stdout),
        String::from_utf8_lossy(&regen.stderr)
    );

    let build = cargo_build_v1_compiler(&clone);
    let stderr = String::from_utf8_lossy(&build.stderr);
    assert!(
        !build.status.success(),
        "deliberate-red: a real handler arm with no real roster row must refuse cargo build"
    );
    assert!(
        stderr.contains("dispatch_bijection_orphan_probe"),
        "must refuse specifically on the orphan macro-invocation token, not some other defect; \
         stderr:\n{stderr}"
    );
    assert!(
        stderr.contains("no rules expected") || stderr.contains("unexpected token") || stderr.contains("no rule expected"),
        "must refuse specifically on macro-invocation failure, not some other defect; stderr:\n{stderr}"
    );

    let _ = std::fs::remove_dir_all(&clone);
}
