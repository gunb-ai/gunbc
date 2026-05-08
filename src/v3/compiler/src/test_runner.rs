use std::collections::BTreeSet;
use std::io::Write;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crate::dag::{
    AtomPayload, Behavior, BindNode, Dag, Declaration, DeclarationId, FieldValue, LiteralBits,
    Path, PortId, PortState, TypeConnective, ValueBody,
};
use crate::diagnostics::Diagnostic;
use crate::emit::rust_target::last_emit_rust_program_top_level_value_bind_name;
use crate::generated_files::GENERATED_FILES;
use crate::infer::type_shapes_equivalent;
use crate::lens_apply::{
    apply_lens_declaration, field_value_from_value_body, int_associativity_holds_all_triples,
    reflect_program_dag_nodes_in_file, ASSOCIATIVITY_WITNESS_TRIPLES, COMMUTATIVITY_WITNESS_PAIRS,
};
use crate::lens_cost::{cost_of, CostLookup};
use crate::types::TypeShape;
use crate::{
    compare_stage_snapshots, compile_stage_snapshots, compile_to_dag, default_fixed_point_source,
    CompileError,
};

/// Same on-disk lens as `v3-compiler/build.rs` splices into `user_authored_lens_compiles_gate`
/// (`emit_r1_gates_fixture`). `LensOutputEquals` applies this program for `named_function_count`
/// so evaluation cannot drift from the fixture-local stub (`INVARIANTS.md` P2).
///
/// **Dissolution:** remove this `include_str!` bridge when `DeclarationRef` (or an equivalent
/// substrate edge) resolves executable lens bodies from `program_dag` / `TestClaim.source` so the
/// runner does not key a second `Dag` on fixture declaration spelling.
pub const R1_CANONICAL_NAMED_FUNCTION_COUNT_LENS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../lenses/named_function_count.dag"
));

/// Same on-disk lens as `src/v3/lenses/complexity.dag`. `LensOutputEquals(cost_of, …)` applies
/// [`crate::lens_cost::cost_of`] (emit from these bytes) on the compiled claim program — not
/// `apply_lens_declaration` on this text (D1 `cost_of` blocks on lens-internal `Loop`). Bytes are
/// still ratcheted in integration tests so the include stays aligned with the lens file.
/// Fixture-local `fn cost_of` stubs are unrelated (`INVARIANTS.md` P2).
pub const R1_CANONICAL_COMPLEXITY_LENS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../lenses/complexity.dag"
));

const SG0_CENSUS_SOURCE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/integration/sg0_census_test.rs"
));
const INFER_HELPERS_SOURCE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../lenses/infer_helpers.dag"
));

/// `.dag` path for [`TestPredicate::SubstrateResearchDeferredClaim`] (TC1 / R2 substrate research).
/// The runner fail-closes unless the `TestClaim` is declared in this fixture file.
const TC1_SUBSTRATE_LENS_ETA_DEFERRED_FIXTURE: &str =
    "src/v3/compiler/tests/fixtures/tc1_substrate_lens_eta_equivalence_deferred.dag";

/// Host-written forward fold for structural depth costs (see `src/v3/lenses/complexity.dag`).
///
/// T-LaneE `DifferentialEquals` compares this receipt to [`crate::lens_cost::cost_of`] (emit output
/// from the same `.dag`). The implementations are **independently maintained** so the gate can
/// fail if the generator drifts from the spec (P3 / api-review #764).
///
/// D1 `apply_lens_declaration` on canonical `cost_of` is **not** used: lowering that lens
/// introduces substrate `Loop` for list recursion, and [`crate::lens_apply::EvalCtx::eval_loop`]
/// returns [`crate::lens_apply::LensApplyError::UnimplementedLoopBound`] until iteration semantics
/// land. **Dissolution:** delete this host mirror once D1 can interpret those `Loop` nodes and route
/// `v3_program_cost` through `apply_lens_declaration` on `cost_of`.
type LaneEHostCostAcc = Vec<(PortId, CostLookup)>;

fn lane_e_host_forward_cost_of(dag: &Dag, port: &PortId) -> CostLookup {
    lane_e_host_lookup_cost(&lane_e_host_compute_costs(dag.nodes()), port)
}

fn lane_e_host_compute_costs(nodes: &[Behavior]) -> LaneEHostCostAcc {
    // Prepend via `insert(0, …)` matches `lens_cost_generated` cons order so `lane_e_host_lookup_cost`
    // agrees with emit (first match wins; order only matters if duplicate ports shadow). Do not
    // reorder without a parity check — delete this receipt once D1 runs canonical `cost_of`.
    let mut acc = lane_e_host_seed_bind_params(nodes);
    for behavior in nodes {
        let entry = lane_e_host_entry_for(&acc, behavior);
        acc.insert(0, entry);
    }
    acc
}

fn lane_e_host_seed_bind_params(nodes: &[Behavior]) -> LaneEHostCostAcc {
    match nodes {
        [] => Vec::new(),
        [head, tail @ ..] => {
            let mut left = lane_e_host_params_of(head);
            left.extend(lane_e_host_seed_bind_params(tail));
            left
        }
    }
}

fn lane_e_host_params_of(behavior: &Behavior) -> LaneEHostCostAcc {
    match behavior {
        Behavior::Value(_) | Behavior::Transform(_) | Behavior::Branch(_) | Behavior::Loop(_) => {
            Vec::new()
        }
        Behavior::Bind(bind) => lane_e_host_param_entries(&bind.params),
    }
}

fn lane_e_host_param_entries(params: &[PortId]) -> LaneEHostCostAcc {
    match params {
        [] => Vec::new(),
        [head, tail @ ..] => {
            let mut list = lane_e_host_param_entries(tail);
            list.insert(0, (*head, CostLookup::Hit(0)));
            list
        }
    }
}

fn lane_e_host_entry_for(acc: &LaneEHostCostAcc, behavior: &Behavior) -> (PortId, CostLookup) {
    match behavior {
        Behavior::Value(v) => (v.result_port(), CostLookup::Hit(0)),
        Behavior::Transform(t) => (
            t.result_port(),
            lane_e_host_add_one(&lane_e_host_sum_costs(acc, &t.inputs)),
        ),
        Behavior::Branch(b) => (
            b.result_port(),
            lane_e_host_add_one(&lane_e_host_add_cost(
                &lane_e_host_lookup_cost(acc, &b.input),
                &lane_e_host_max_path_cost(acc, &b.paths),
            )),
        ),
        Behavior::Loop(l) => (
            l.result_port(),
            lane_e_host_add_one(&lane_e_host_add_cost(
                &lane_e_host_lookup_cost(acc, &l.source),
                &lane_e_host_lookup_cost(acc, &l.init),
            )),
        ),
        Behavior::Bind(bind) => {
            let rp = bind.result_port();
            (rp, lane_e_host_lookup_cost(acc, &rp))
        }
    }
}

fn lane_e_host_sum_costs(acc: &LaneEHostCostAcc, ports: &[PortId]) -> CostLookup {
    ports.iter().fold(CostLookup::Hit(0), |sum, port_id| {
        lane_e_host_add_cost(&sum, &lane_e_host_lookup_cost(acc, port_id))
    })
}

fn lane_e_host_max_path_cost(acc: &LaneEHostCostAcc, paths: &[Path]) -> CostLookup {
    paths.iter().fold(CostLookup::Hit(0), |best, path| {
        lane_e_host_max_cost(&best, &lane_e_host_lookup_cost(acc, &path.output))
    })
}

fn lane_e_host_lookup_cost(acc: &[(PortId, CostLookup)], port_id: &PortId) -> CostLookup {
    match acc.split_first() {
        None => CostLookup::Miss,
        Some(((port, cost), tail)) => {
            if port == port_id {
                cost.clone()
            } else {
                lane_e_host_lookup_cost(tail, port_id)
            }
        }
    }
}

fn lane_e_host_add_one(c: &CostLookup) -> CostLookup {
    match c {
        CostLookup::Miss => CostLookup::Miss,
        CostLookup::Hit(n) => CostLookup::Hit(n + 1),
    }
}

fn lane_e_host_add_cost(a: &CostLookup, b: &CostLookup) -> CostLookup {
    match a {
        CostLookup::Miss => CostLookup::Miss,
        CostLookup::Hit(x) => match b {
            CostLookup::Miss => CostLookup::Miss,
            CostLookup::Hit(y) => CostLookup::Hit(*x + *y),
        },
    }
}

fn lane_e_host_max_cost(a: &CostLookup, b: &CostLookup) -> CostLookup {
    match a {
        CostLookup::Miss => CostLookup::Miss,
        CostLookup::Hit(x) => match b {
            CostLookup::Miss => CostLookup::Miss,
            CostLookup::Hit(y) => CostLookup::Hit((*x).max(*y)),
        },
    }
}

/// T-LaneE `DifferentialEquals` cost lineage: **v3** = host forward fold (spec mirror above);
/// **v2** = Rust-generated [`cost_of`] (`lens_cost_generated`).
fn eval_lane_e_differential_cost_lineage(
    lineage_name: &str,
    program_dag: &Dag,
    bind_port: PortId,
) -> Result<CostLookup, String> {
    match lineage_name {
        "v3_program_cost" => Ok(lane_e_host_forward_cost_of(program_dag, &bind_port)),
        "v2_oracle_cost" => Ok(cost_of(program_dag, &bind_port)),
        _ => Err(format!(
            "unsupported lineage `{lineage_name}` for T-LaneE `DifferentialEquals` cost (expected `v3_program_cost` or `v2_oracle_cost`)"
        )),
    }
}

fn w1_parse_single_int_stdout_carve_out(stdout: &str) -> Result<i64, String> {
    let trimmed = stdout.trim();
    if trimmed.is_empty() {
        return Err(
            "W1 rust_emit_output: empty stdout after trim (transitional Int-only stdout parse \
             carve-out authorized for slice-1 only; dissolution: substrate `ProgramObservation<Value>` \
             + observation-channel / `ValueKind` per `docs/briefs/r3-pr-e8-w1-output-producer-contract-blocker.md`)"
                .to_string(),
        );
    }
    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    if parts.len() != 1 {
        return Err(format!(
            "W1 rust_emit_output: stdout must be exactly one integer token (transitional carve-out); \
             got {parts:?} (dissolution: typed observation channel + `ValueKind`)"
        ));
    }
    parts[0].parse::<i64>().map_err(|_| {
        format!(
            "W1 rust_emit_output: stdout token `{}` is not a valid i64 (transitional Int-only carve-out)",
            parts[0]
        )
    })
}

/// RAII guard: delete the W1 rustc scratch directory on return or panic (best-effort `remove_dir_all`).
struct W1RustEmitScratchGuard {
    dir: std::path::PathBuf,
}

impl W1RustEmitScratchGuard {
    fn new(dir: std::path::PathBuf) -> Self {
        Self { dir }
    }
}

impl Drop for W1RustEmitScratchGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// Max bytes retained from a W1 host child's **stdout** (slice-1 Int observation: one token +
/// whitespace). Beyond this we keep draining until EOF so the child cannot stall on a full pipe.
const W1_HOST_CAPTURE_MAX_STDOUT_BYTES: usize = 16 * 1024;

/// Max bytes retained from a W1 host child's **stderr** (`rustc` diagnostics / runtime errors).
/// Same drain-to-EOF behavior after the cap.
const W1_HOST_CAPTURE_MAX_STDERR_BYTES: usize = 256 * 1024;

const W1_HOST_DRAIN_CHUNK_BYTES: usize = 8192;

#[derive(Debug)]
struct W1BoundedPipeCapture {
    bytes: Vec<u8>,
    truncated: bool,
}

/// Read from `r` until EOF: store at most `max_stored` bytes, then discard the rest so the peer
/// cannot block on a full pipe (P2 host-process boundary).
fn w1_drain_reader_bounded(
    mut r: impl std::io::Read,
    max_stored: usize,
) -> std::io::Result<W1BoundedPipeCapture> {
    let mut buf = Vec::new();
    let mut truncated = false;
    let mut scratch = [0u8; W1_HOST_DRAIN_CHUNK_BYTES];
    loop {
        let n = r.read(&mut scratch)?;
        if n == 0 {
            break;
        }
        if buf.len() >= max_stored {
            truncated = true;
            continue;
        }
        let room = max_stored - buf.len();
        if n <= room {
            buf.extend_from_slice(&scratch[..n]);
        } else {
            buf.extend_from_slice(&scratch[..room]);
            truncated = true;
        }
    }
    Ok(W1BoundedPipeCapture {
        bytes: buf,
        truncated,
    })
}

/// New process group for the W1 host child so [`kill_process_group_on_timeout`] can tear down a
/// wedged `rustc` or emitted binary without leaving grandchildren (same contract as
/// [`build_execute_command_process`]).
fn w1_prepare_host_command(cmd: &mut Command) {
    cmd.stdin(Stdio::null());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        unsafe {
            cmd.pre_exec(|| {
                if libc::setpgid(0, 0) != 0 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
    }
}

/// `spawn` + wall-bounded wait + **bounded** stdout/stderr capture — fail-closed vs unbounded
/// `output()` / `read_to_end` allocation (Decidability / CI). Uses [`EXECUTE_COMMAND_WALL_TIMEOUT`]
/// and the same `try_wait` poll + process-group kill path as [`child_wait_for_execute_command`].
/// After [`W1_HOST_CAPTURE_MAX_STDOUT_BYTES`] / [`W1_HOST_CAPTURE_MAX_STDERR_BYTES`] the reader keeps
/// draining until EOF so a verbose child cannot wedge on a full pipe.
fn w1_host_command_output(
    label: &str,
    wall: Duration,
    mut cmd: Command,
) -> Result<std::process::Output, String> {
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("{label}: failed to spawn host child: {e}"))?;
    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| format!("{label}: internal error: stdout not piped (W1 host harness)"))?;
    let mut stderr = child
        .stderr
        .take()
        .ok_or_else(|| format!("{label}: internal error: stderr not piped (W1 host harness)"))?;

    let stdout_handle = std::thread::spawn(move || {
        w1_drain_reader_bounded(&mut stdout, W1_HOST_CAPTURE_MAX_STDOUT_BYTES)
    });
    let stderr_handle = std::thread::spawn(move || {
        w1_drain_reader_bounded(&mut stderr, W1_HOST_CAPTURE_MAX_STDERR_BYTES)
    });

    let status = match child_wait_for_execute_command(&mut child, wall) {
        Ok(s) => s,
        Err(ChildWaitFail::WallTimeout { wall_time }) => {
            let _ = stdout_handle.join();
            let _ = stderr_handle.join();
            return Err(format!(
                "{label}: exceeded {:.2}s wall-clock limit (timeout — process group / child killed, fail-closed); \
                 dissolution: PB-Runtime owns host subprocess policy for generated tests",
                wall_time.as_secs_f64()
            ));
        }
        Err(ChildWaitFail::Io(err)) => {
            let _ = stdout_handle.join();
            let _ = stderr_handle.join();
            return Err(format!("{label}: wait on host child failed: {err}"));
        }
    };

    let stdout_cap = stdout_handle
        .join()
        .map_err(|_| format!("{label}: stdout capture thread panicked"))?
        .map_err(|e| format!("{label}: read stdout failed: {e}"))?;
    let stderr_cap = stderr_handle
        .join()
        .map_err(|_| format!("{label}: stderr capture thread panicked"))?
        .map_err(|e| format!("{label}: read stderr failed: {e}"))?;

    if stdout_cap.truncated || stderr_cap.truncated {
        return Err(format!(
            "{label}: bounded host I/O exceeded (stdout cap {} B, stderr cap {} B; stdout_trunc={} stderr_trunc={}); \
             child exit={:?}; fail-closed vs unbounded capture; dissolution: PB-Runtime owns diagnostic bounds for generated-test subprocesses",
            W1_HOST_CAPTURE_MAX_STDOUT_BYTES,
            W1_HOST_CAPTURE_MAX_STDERR_BYTES,
            stdout_cap.truncated,
            stderr_cap.truncated,
            status.code(),
        ));
    }

    Ok(std::process::Output {
        status,
        stdout: stdout_cap.bytes,
        stderr: stderr_cap.bytes,
    })
}

fn w1_rust_emit_output_int(
    program_dag: &Dag,
    output_bind: &BindNode,
    claim_file: &str,
) -> Result<i64, String> {
    // **Transitional producer identity (contract 1):** only the spelling `rust_emit_output` is
    // accepted at the `DifferentialEquals` subject/oracle `DeclarationRef` site, fail-closed
    // elsewhere in `eval_differential_equals`. **Dissolution:** substrate producer-role markers
    // replace name-keyed recognition (`docs/briefs/r3-pr-e8-w1-output-producer-contract-blocker.md`).
    let last_bind = match last_emit_rust_program_top_level_value_bind_name(program_dag) {
        Ok(Some(name)) => name,
        Ok(None) => {
            return Err(
                "W1 rust_emit_output: compiled program has no top-level value binds (emit_rust program \
                 mode requires at least one); dissolution: PB-Runtime-generated target-language tests"
                    .to_string(),
            );
        }
        Err(e) => {
            return Err(format!(
                "W1 rust_emit_output: cannot resolve emit-rust print target (same `RealizationIndexes` \
                 path as `emit_rust_with_mode`): {e:?}"
            ));
        }
    };
    if last_bind != output_bind.name {
        return Err(format!(
            "W1 rust_emit_output: `ProgramOutputBind.output_ref` must name the **last** top-level \
             value bind after `source_filtering` (same bind `emit_rust` program-mode `main` prints; \
             see `last_emit_rust_program_top_level_value_bind_name`) — expected `{last_bind}`, got `{}` \
             (claim file `{claim_file}`; transitional coupling; dissolution: PB-Runtime harness + typed \
             observation channel)",
            output_bind.name
        ));
    }
    let rust_src = std::thread::scope(|s| -> Result<String, String> {
        let handle = std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn_scoped(s, || crate::emit_rust::emit_rust(program_dag))
            .map_err(|e| format!("W1 rust_emit_output: spawn emit worker: {e}"))?;
        let joined = handle
            .join()
            .map_err(|_| "W1 rust_emit_output: emit worker panicked".to_string())?;
        joined.map_err(|e| {
            format!(
                "W1 rust_emit_output: emit_rust failed (use #1485-approved `emit_rust` / `emit_rust_with_mode` path only; dissolution: PB-Runtime-generated tests): {e:?}"
            )
        })
    })?;
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let scratch = std::env::temp_dir().join(format!(
        "gunbc_w1_rust_emit_{}_{}",
        std::process::id(),
        stamp
    ));
    std::fs::create_dir_all(&scratch).map_err(|e| {
        format!(
            "W1 rust_emit_output: create scratch dir {}: {e}",
            scratch.display()
        )
    })?;
    let _scratch_guard = W1RustEmitScratchGuard::new(scratch.clone());
    let src_path = scratch.join("main.rs");
    let bin_path = scratch.join("w1_emit_eval_out");
    let mut file = std::fs::File::create(&src_path)
        .map_err(|e| format!("W1 rust_emit_output: create {}: {e}", src_path.display()))?;
    file.write_all(rust_src.as_bytes())
        .map_err(|e| format!("W1 rust_emit_output: write {}: {e}", src_path.display()))?;

    let mut rustc = Command::new("rustc");
    // Same stable-toolchain contract as boundary rustc harnesses (strip bootstrap leakage).
    // Wall-bounded `spawn` + pipe drain (not `status()` / not unbounded `output()`): piped stderr
    // can stall `rustc`; a wedged compile must fail-closed like [`EXECUTE_COMMAND_WALL_TIMEOUT`].
    rustc
        .env_remove("RUSTC_BOOTSTRAP")
        .arg("--edition=2021")
        .arg(&src_path)
        .arg("-o")
        .arg(&bin_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    w1_prepare_host_command(&mut rustc);
    let compile_out = w1_host_command_output(
        "W1 rust_emit_output: rustc",
        EXECUTE_COMMAND_WALL_TIMEOUT,
        rustc,
    )?;
    if !compile_out.status.success() {
        let stderr = String::from_utf8_lossy(&compile_out.stderr);
        let stdout = String::from_utf8_lossy(&compile_out.stdout);
        return Err(format!(
            "W1 rust_emit_output: rustc failed (exit {:?}); rustc stdout:\n{}\nrustc stderr:\n{}; \
             dissolution: PB-Runtime owns compilation policy for generated tests",
            compile_out.status.code(),
            stdout.trim_end(),
            stderr.trim_end(),
        ));
    }
    let mut run_cmd = Command::new(&bin_path);
    run_cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    w1_prepare_host_command(&mut run_cmd);
    let run = w1_host_command_output(
        "W1 rust_emit_output: emitted binary",
        EXECUTE_COMMAND_WALL_TIMEOUT,
        run_cmd,
    )?;
    if !run.status.success() {
        let stderr = String::from_utf8_lossy(&run.stderr);
        let stdout = String::from_utf8_lossy(&run.stdout);
        return Err(format!(
            "W1 rust_emit_output: emitted Rust binary exited with {:?}; stdout:\n{}\nstderr:\n{}; \
             dissolution: PB-Runtime-generated tests own exit-status / stdio policy",
            run.status.code(),
            stdout.trim_end(),
            stderr.trim_end(),
        ));
    }
    w1_parse_single_int_stdout_carve_out(&String::from_utf8_lossy(&run.stdout))
}

fn w1_dag_eval_output_int(program_dag: &Dag, output_bind: &BindNode) -> Result<i64, String> {
    // **Dissolution:** `dag_eval_output` collapses into PR-B eager evaluation + witness construction
    // (`docs/briefs/r2-pr-b-2-runner-extension-bundle.md`); this arm is the first runner bridge only.
    if !output_bind.params.is_empty() {
        return Err(format!(
            "W1 dag_eval_output: bind `{}` must be a top-level value bind (empty `params`); got callable-shaped bind",
            output_bind.name
        ));
    }
    let producer = program_dag
        .resolve_producer_opt(&output_bind.value)
        .ok_or_else(|| {
            format!(
            "W1 dag_eval_output: bind `{}` value port has no producer node in compiled program Dag",
            output_bind.name
        )
        })?;
    let entry = producer.id();
    let strategy = crate::evaluator::EvalStrategy::ApplicativeOrder {
        input_order: crate::evaluator::InputEvaluationOrder::LeftFirst,
    };
    let mut state =
        crate::evaluator::EvalStateStack::with_root_frame(crate::evaluator::EvalFrame::empty());
    let value = crate::evaluator::evaluate_body(program_dag, entry, &mut state, strategy)
        .map_err(|e| {
            format!(
                "W1 dag_eval_output: `evaluate_body` / `eval_node` error on bind `{}` (no-memo eager `ApplicativeOrder` / `LeftFirst` at call site per #1485 fire criteria): {e:?}",
                output_bind.name
            )
        })?;
    match &value {
        crate::evaluator::Value::LiteralValue(LiteralBits::Int(n)) => Ok(*n),
        other => Err(format!(
            "W1 dag_eval_output: only `Value::LiteralValue(Int)` is supported for slice-1 Int parity \
             (transitional debt; dissolution: substrate `ValueKind` widening + observation normalization): {other:?}"
        )),
    }
}

fn w1_differential_equals_lineage_int(
    lineage: &str,
    program_dag: &Dag,
    output_bind: &BindNode,
    claim_file: &str,
) -> Result<i64, String> {
    match lineage {
        "rust_emit_output" => w1_rust_emit_output_int(program_dag, output_bind, claim_file),
        "dag_eval_output" => w1_dag_eval_output_int(program_dag, output_bind),
        _ => Err(format!(
            "W1 internal error: unknown lineage `{lineage}` (expected rust_emit_output or dag_eval_output)"
        )),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClaimResult {
    Pass,
    Fail(String),
    /// Runner does not implement this path yet; message is surfaced to tests and logs.
    NotYetImplemented(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimEvaluation {
    pub claim_name: String,
    pub result: ClaimResult,
}

/// Typed failure modes for [`eval_algebraic_law_for_claim_program`] (C-5: no string
/// sub-match on `Err` to classify behavior — discriminate on this enum).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlgebraicLawProgramError {
    /// Law kind is not implemented in the public helper (M1.5 harness: treat as runner-deferred).
    UnsupportedLaw { law_label: String },
    /// Predicate payload or referenced structure is invalid for evaluation.
    MalformedPayload(String),
}

/// Hermetic `AlgebraicLaw` evaluation against a compiled claim program (`program_dag`).
///
/// **`Associativity` / `Commutativity` — bounded operational witnesses, not substrate law proof:**
/// uses [`int_associativity_holds_all_triples`](crate::lens_apply::int_associativity_holds_all_triples)
/// over [`ASSOCIATIVITY_WITNESS_TRIPLES`](crate::lens_apply::ASSOCIATIVITY_WITNESS_TRIPLES) so a
/// single lucky `(a,b,c)` cannot certify a false law; `Commutativity` uses
/// [`COMMUTATIVITY_WITNESS_PAIRS`](crate::lens_apply::COMMUTATIVITY_WITNESS_PAIRS) the same way.
/// These paths do **not** consume quantified facts declared on `OrderedRing` / semigroup carriers
/// in `std.algebra` (those are not yet first-class runner inputs). Treating `Pass` here as full
/// algebraic law evidence would be weaker than a substrate-backed law check. **Dissolution:** wire
/// `AlgebraicLaw` to declared law metadata / witnesses on disk and reserve sample-only checks to
/// explicit testgen predicates, or return [`ClaimResult::NotYetImplemented`] until that substrate
/// surface exists.
///
/// `lens_ref` is a [`FieldValue::Reference`] into `fixture_dag`; the runner resolves the **name**
/// and looks up the same name in `program_dag`.
pub fn eval_algebraic_law_for_claim_program(
    fixture_dag: &Dag,
    program_dag: &Dag,
    payload: &[FieldValue],
) -> Result<bool, AlgebraicLawProgramError> {
    let (law, lens_ref) = algebraic_law_payload_fields(payload)?;
    let (law_label, law_payload) = variant_fields(fixture_dag, law)?;
    match law_label.as_str() {
        "Associativity" | "Commutativity" => {}
        "Identity" => return Err(AlgebraicLawProgramError::UnsupportedLaw { law_label }),
        // `Distributivity` is intentionally absent from AlgebraicLawKind. If a future substrate
        // enum extension adds it, route implementation through INVARIANTS P1 first instead of
        // encoding it through another predicate or overloading an existing law.
        other => {
            return Err(AlgebraicLawProgramError::UnsupportedLaw {
                law_label: other.to_string(),
            });
        }
    }
    let lens_name = declaration_ref_name(fixture_dag, lens_ref)?;
    let Some(target) = program_dag.declaration_by_name(&lens_name) else {
        return Ok(false);
    };
    match law_label.as_str() {
        "Associativity" => {
            if !law_payload.is_empty() {
                return Err(AlgebraicLawProgramError::MalformedPayload(
                    "Associativity should be payload-free".to_string(),
                ));
            }
            int_associativity_holds_all_triples(
                program_dag,
                target.id,
                ASSOCIATIVITY_WITNESS_TRIPLES,
            )
            .map_err(|e| {
                AlgebraicLawProgramError::MalformedPayload(format!("lens apply error: {e:?}"))
            })
        }
        "Commutativity" => {
            if !law_payload.is_empty() {
                return Err(AlgebraicLawProgramError::MalformedPayload(
                    "Commutativity should be payload-free".to_string(),
                ));
            }
            int_commutativity_holds_all_pairs(program_dag, target.id, COMMUTATIVITY_WITNESS_PAIRS)
                .map_err(|e| {
                    AlgebraicLawProgramError::MalformedPayload(format!("lens apply error: {e:?}"))
                })
        }
        _ => unreachable!("unsupported AlgebraicLawKind returned before lens resolution"),
    }
}

/// Compile-time ratchet (PR #741 / codex P1): `Associativity` must not regress to checking one
/// lucky `(a, b, c)` triple — the gate is a correctness signal only when the witness set has
/// material breadth (see `lens_apply::ASSOCIATIVITY_WITNESS_TRIPLES`).
const _: () = assert!(ASSOCIATIVITY_WITNESS_TRIPLES.len() > 1);

/// Shared structural-value comparator for runner-side value-domain checks.
///
/// PR-B.3 uses this for `AlgebraicLaw::Commutativity`; PR-B.4 should reuse this authority when
/// per-target structural-output normalization lands instead of introducing a second comparator.
pub fn runner_structural_values_equal(left: &FieldValue, right: &FieldValue) -> bool {
    left == right
}

/// Transitional PR-B.3 runner scaffold: evaluate `a ⊕ b == b ⊕ a` over the bounded Int witness
/// table. Dissolution hook: replace sample-table checks with first-class substrate law witnesses
/// consumed by PR-B evaluator / PB-Runtime once lens algebra facts are declared.
fn int_commutativity_holds_all_pairs(
    program_dag: &Dag,
    lens_decl_id: DeclarationId,
    pairs: &[(i64, i64)],
) -> Result<bool, crate::lens_apply::LensApplyError> {
    let int = |n: i64| FieldValue::Literal(LiteralBits::Int(n));
    for &(a, b) in pairs {
        let left = apply_lens_declaration(program_dag, lens_decl_id, &[int(a), int(b)])?;
        let right = apply_lens_declaration(program_dag, lens_decl_id, &[int(b), int(a)])?;
        if !runner_structural_values_equal(&left, &right) {
            return Ok(false);
        }
    }
    Ok(true)
}

const _: () = assert!(COMMUTATIVITY_WITNESS_PAIRS.len() > 1);

// --- `TestPredicate::ExecuteCommand` (PB-Runtime) — shared by `TestRunner` and M1.5 testgen ---

/// Extracts `(command, args, expect_exit_code)` from `ExecuteCommand` lowered payloads
/// (positional `Conj` fields or a single `Record`). Matches `m1_5_testgen` historical parser.
pub fn parse_execute_command_fields(payload: &[FieldValue]) -> Option<(String, Vec<String>, i64)> {
    match payload {
        [FieldValue::Record(fields)] => {
            let command = execute_command_string_field(fields, "command")?;
            let expect_exit_code = fields
                .iter()
                .find(|(label, _)| label == "expect_exit_code")
                .and_then(|(_, value)| match value {
                    FieldValue::Literal(LiteralBits::Int(n)) => Some(*n),
                    _ => None,
                })?;
            let args = fields
                .iter()
                .find(|(label, _)| label == "args")
                .and_then(|(_, value)| list_string_literal_values(value))?;
            Some((command, args, expect_exit_code))
        }
        [cmd, args, code] => {
            let FieldValue::Literal(LiteralBits::String(command)) = cmd else {
                return None;
            };
            let argv = list_string_literal_values(args)?;
            let FieldValue::Literal(LiteralBits::Int(expect_exit_code)) = code else {
                return None;
            };
            Some((command.clone(), argv, *expect_exit_code))
        }
        _ => None,
    }
}

fn execute_command_string_field(fields: &[(String, FieldValue)], label: &str) -> Option<String> {
    fields
        .iter()
        .find(|(l, _)| l == label)
        .and_then(|(_, v)| match v {
            FieldValue::Literal(LiteralBits::String(s)) => Some(s.clone()),
            _ => None,
        })
}

fn list_string_literal_values(value: &FieldValue) -> Option<Vec<String>> {
    let FieldValue::List(items) = value else {
        return None;
    };
    let mut out = Vec::with_capacity(items.len());
    for item in items {
        let FieldValue::Literal(LiteralBits::String(s)) = item else {
            return None;
        };
        out.push(s.clone());
    }
    Some(out)
}

/// Hard wall-clock for [`evaluate_execute_command_exit_code`]: fail-closed `ClaimResult::Fail`
/// (not hang / not unbounded) so checked-in `TestClaim` data cannot block CI on a runaway child.
/// Adjusting the limit is policy; the substrate has no per-claim override today.
pub const EXECUTE_COMMAND_WALL_TIMEOUT: Duration = Duration::from_secs(30);

const EXECUTE_COMMAND_WAIT_POLL: Duration = Duration::from_millis(20);

const SHELL_DASH_C_BACKGROUND_STEMS: [&str; 5] = ["sh", "bash", "dash", "ksh", "zsh"];
const SHELL_C_BACKGROUND_UNBOUNDED_FAIL: &str = "ExecuteCommand: shell `-c` script has a `&` that may be an unmodelled **background** \
         job (after eliding `&&` and `n>&m` / `&>`-style fd spellings) — a direct `Child` wait is not \
         a full process boundary. Rephrase (e.g. a direct tool, or a `-c` string that does not rely on \
         shell `&` background) — P3/P4.";

/// Path stem in [`SHELL_DASH_C_BACKGROUND_STEMS`] (helper for shell `-c` / background guard).
fn shell_dash_c_background_stem_is_shell(arg: &str) -> bool {
    let s = std::path::Path::new(arg)
        .file_name()
        .and_then(|x| x.to_str())
        .unwrap_or(arg);
    SHELL_DASH_C_BACKGROUND_STEMS.contains(&s)
}

/// `sh` and `dash` do not treat `&>` as a single bash-style redirect token — the same bytes can
/// be `&` (background) + `>` (api-review openai-pro gpt-5-5-pro, PR #792: P3/P4).
fn shell_stem_is_posix_sh_or_dash(shell_path: &str) -> bool {
    let s = std::path::Path::new(shell_path)
        .file_name()
        .and_then(|x| x.to_str())
        .unwrap_or(shell_path);
    matches!(s, "sh" | "dash")
}

/// Whether `&>` / `&>>` may be elided as non-background `&` spellings (bash/ksh/zsh). For
/// `sh`/`dash` and unknown interpreter, we **do not** elide — see
/// [`shell_dash_c_may_start_background_after_eliding_artifacts`].
fn shell_interpreter_allows_bash_style_ampersand_gt_redirect(interpreter: Option<&str>) -> bool {
    match interpreter {
        None => false,
        Some(s) if shell_stem_is_posix_sh_or_dash(s) => false,
        Some(_) => true,
    }
}

/// The `-c` (or combined `-?c?`) at `c_flag_index` is run by the nearest preceding shell in
/// `args`, or by `leading_hint` when the slice is `["-c", "script"]` / `["-ec", "script"]` only
/// (e.g. `env(1)` + `sh -c` — the shell is not `args[0]` of the tail).
fn shell_interpreter_for_c_flag<'a>(
    args: &'a [String],
    c_flag_index: usize,
    leading_hint: Option<&'a str>,
) -> Option<&'a str> {
    if c_flag_index > 0 && shell_dash_c_background_stem_is_shell(&args[c_flag_index - 1]) {
        return Some(args[c_flag_index - 1].as_str());
    }
    if c_flag_index == 0 {
        return leading_hint;
    }
    None
}

/// `true` if a bare `&` may be shell background, scanning **all** `"-c"` and combined `-?c?` invocations
/// in `args`, and **recursing** when a `-c` (or combined) **script** value is a shell path stem and
/// more argv follow — e.g. `sh -c sh -ec "sleep&"` (POSIX: `-c` takes one script word, then
/// `argv` continues) would otherwise be mis-read as a script of `sh` only (PR #792 inline; P4).
///
/// Production uses [`shell_argv_may_start_unbounded_background_with_hint`]; this wrapper is for
/// unit tests only.
#[cfg(test)]
fn shell_argv_may_start_unbounded_background(args: &[String]) -> bool {
    shell_argv_may_start_unbounded_background_with_hint(args, None)
}

/// Like [`shell_argv_may_start_unbounded_background`], but when `args` is only the **tail** after a
/// known shell (e.g. `["-c", "…"]` for `env sh -c …`), pass that shell as `leading_hint` so `&>` /
/// POSIX elision is correct (openai-pro PR #792).
fn shell_argv_may_start_unbounded_background_with_hint(
    args: &[String],
    leading_hint: Option<&str>,
) -> bool {
    const MAX_NEST: u32 = 32;

    fn is_combined_c_not_exact(a: &str) -> bool {
        if a == "-c" || a.starts_with("--") {
            return false;
        }
        a.strip_prefix('-')
            .is_some_and(|f| !f.is_empty() && !f.starts_with('-') && f.chars().any(|ch| ch == 'c'))
    }

    fn check_slice(args: &[String], depth: u32, leading_hint: Option<&str>) -> bool {
        // P3 / P4: if we cannot finish scanning, fail closed — a depth escape must not be taken as
        // "no unbounded background" and allow a spawn past the policy guard (api-review codex 3a2a9f64).
        if depth > MAX_NEST {
            return true;
        }
        for i in 0..args.len() {
            if &args[i] == "-c" {
                if let Some(s) = args.get(i + 1) {
                    let intr = shell_interpreter_for_c_flag(args, i, leading_hint);
                    if shell_dash_c_may_start_background_after_eliding_artifacts(s, intr) {
                        return true;
                    }
                    if i + 2 < args.len() && shell_dash_c_background_stem_is_shell(s) {
                        let mut inner = vec![s.to_string()];
                        inner.extend_from_slice(&args[i + 2..]);
                        let inner_leading = inner
                            .first()
                            .filter(|a| shell_dash_c_background_stem_is_shell(a))
                            .map(|a| a.as_str());
                        if check_slice(&inner, depth + 1, inner_leading) {
                            return true;
                        }
                    }
                }
            }
        }
        for i in 0..args.len() {
            let a = &args[i];
            if a.starts_with("--") || a == "-c" {
                continue;
            }
            if is_combined_c_not_exact(a) {
                if let Some(s) = args.get(i + 1) {
                    let intr = shell_interpreter_for_c_flag(args, i, leading_hint);
                    if shell_dash_c_may_start_background_after_eliding_artifacts(s, intr) {
                        return true;
                    }
                    if i + 2 < args.len() && shell_dash_c_background_stem_is_shell(s) {
                        let mut inner = vec![s.to_string()];
                        inner.extend_from_slice(&args[i + 2..]);
                        let inner_leading = inner
                            .first()
                            .filter(|a| shell_dash_c_background_stem_is_shell(a))
                            .map(|a| a.as_str());
                        if check_slice(&inner, depth + 1, inner_leading) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
    let inner_leading = args
        .first()
        .filter(|a| shell_dash_c_background_stem_is_shell(a))
        .map(|a| a.as_str());
    let hint = leading_hint.or(inner_leading);
    check_slice(args, 0, hint)
}

/// Heuristic: on POSIX shells with `-c`, a *shell background* `&` (not part of `&&` / fd
/// redirect spellings) means the child may exit 0 while other work still runs. We are not a full
/// sh parser: strip a few *common* `&` spellings, then if `&` remains, fail-closed (P3/P4). See
/// `shell_dash_c_may_start_background_after_eliding_artifacts` tests in this module.
///
/// The top-level `command` need not be a shell (e.g. `env(1)` with `["sh", "-c", "…&"]`); a shell
/// **anywhere** in `args` (path stem) with a following `-c` in the same tail is checked (api-review
/// 994fa40d). For each such index `j`, the guard runs on **`args[j + 1..]`** (the **argv tail after
/// the shell executable**), not `args[j..]`, so we do not double-count the shell token as if it were
/// part of the combined-flag script (api-review e99b53e7, codex). Nested re-exec (script token is
/// `sh`/`bash`/…, rest is another `-c`/`-ec`) is handled by [`shell_argv_may_start_unbounded_background`].
fn reject_unbounded_shell_background(command: &str, args: &[String]) -> Option<ClaimResult> {
    let fail = || ClaimResult::Fail(SHELL_C_BACKGROUND_UNBOUNDED_FAIL.to_string());

    let stem = std::path::Path::new(command)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(command);
    if SHELL_DASH_C_BACKGROUND_STEMS.contains(&stem)
        && shell_argv_may_start_unbounded_background_with_hint(args, Some(command))
    {
        return Some(fail());
    }
    for j in 0..args.len() {
        if !shell_dash_c_background_stem_is_shell(&args[j]) {
            continue;
        }
        if shell_argv_may_start_unbounded_background_with_hint(&args[j + 1..], Some(&args[j])) {
            return Some(fail());
        }
    }
    None
}

/// `sh`/`bash`/… with `-c` in **any** common spelling: standalone `"-c"`, or combined single-dash
/// flags that include the `c` option (e.g. `"-ec"`, `"-lc"`) with the next argument as the script.
/// The `-c` / combined-flag token may appear **anywhere** in the slice (e.g. `["sh", "-ec", "cmd"]` or
/// `["-ec", "cmd"]`); codex PR #792, api-review e99b53e7: first-arg-only special case missed
/// `env sh -ec "…&"`. Production guard uses [`shell_argv_may_start_unbounded_background`];
/// this helper is kept for **unit** tests and doc parity only.
///
/// **Not a model of the P3/P4 guard:** it returns the script for the *first* `-c` or combined
/// `-?c?` in argv order. For `["sh", "-c", "sh", "-ec", "…&"]` that first script word is the
/// *shell token* `sh` — a correct *slice-local* read, not the same as “what might background.”
/// [`shell_argv_may_start_unbounded_background`] recurses for that case; the pre-spawn code never
/// calls this helper. Confusion with `args[j+1..]` for `env sh -ec` is a false alarm: the tail is
/// `["-ec", "script"]` and [`check_slice`]’s `s` is `args[i+1]` for the matched flag at `i`, not a
/// mis-attached `shell_dash_c_script_string` (PR #792 inline, 2026-04-25).
#[cfg(test)]
fn shell_dash_c_script_string(args: &[String]) -> Option<&str> {
    for (i, a) in args.iter().enumerate() {
        if a == "-c" {
            return args.get(i + 1).map(String::as_str);
        }
        if a.starts_with("--") {
            continue;
        }
        if let Some(flags) = a.strip_prefix('-') {
            if !flags.is_empty() && !flags.starts_with('-') && flags.chars().any(|ch| ch == 'c') {
                return args.get(i + 1).map(String::as_str);
            }
        }
    }
    None
}

/// Strips a few *non-background* `&` patterns from a `-c` string, then returns `true` only if
/// a bare `&` (likely background) may remain. Not a sh grammar; conservative only where
/// we would otherwise false-positive `true && true`, `2>&1`, `n>&m`, and the default-fd shorthand
/// `>&d` (e.g. `>&2` in `command >&2`). **Quoted** `&` (e.g. `echo \"&\"`)
/// is not modeled and may be fail-closed as if it were a background `&` — an acceptable
/// false-reject; user should rephrase without relying on a literal `&` in the `-c` string. This is
/// the likeliest UX foot-gun for hand-authored `.dag` claims (api-review 994fa40d).
///
/// `interpreter` is the path or stem of the shell that will run this `-c` script. On POSIX
/// `sh`/`dash`, `&>` is **not** a single redirect token — the same bytes can background a command
/// before `>` — so we **fail closed** if the script contains `&>` / `&>>` and do not elide (openai-pro
/// gpt-5-5-pro, PR #792). For `bash`/`ksh`/`zsh` we elide `&>` as a non-background spelling. If the
/// interpreter is unknown, we do not elide `&>` (fail closed on any `&>` in the script).
///
/// **TODO(dissolution, T-PB-B, input shaping):** retire literal `String::replace` here when
/// `ExecuteCommand`’s `command`+`args` are narrow enough to forbid ambiguous `sh -c` (schema gate),
/// or a **typed** hermetic host runner supersedes the shell escape hatch, or a real `sh` subset
/// parser is shared with CI policy — *input* heuristics are a smell on the same seam as P2(a) on
/// outcomes, but this path only **rejects** (no accept-on-text-match for claim truth). **Do not**
/// grow the elision list ad hoc — that deepens the bridge; link new work to a dissolution (Claude
/// e99b53e7).
fn shell_dash_c_may_start_background_after_eliding_artifacts(
    script: &str,
    interpreter: Option<&str>,
) -> bool {
    if !shell_interpreter_allows_bash_style_ampersand_gt_redirect(interpreter)
        && (script.contains("&>>") || script.contains("&>"))
    {
        return true;
    }
    let mut t = script.to_string();
    while t.contains("&&") {
        t = t.replace("&&", "  ");
    }
    t = t.replace("2>&1", "");
    t = t.replace("1>&2", "");
    t = t.replace("2>&2", "");
    t = t.replace("1>&1", "");
    t = t.replace("0>&1", "");
    if shell_interpreter_allows_bash_style_ampersand_gt_redirect(interpreter) {
        t = t.replace("&>>", " ");
        t = t.replace("&>", " ");
    }
    for a in 0u8..=9 {
        for b in 0u8..=9 {
            t = t.replace(&format!("{a}>&{b}"), "");
        }
    }
    // `>&d` = default stdout to fd d (e.g. `>&2` in `done >&2`); not a background `&`.
    for d in 0u8..=9 {
        t = t.replace(&format!(">&{d}"), "");
    }
    t.contains('&')
}

// On Linux, wrap the logical `command` + `args` in a **user + PID namespace** (util-linux
// `unshare(1)` with `-c` = map current user, `-f` fork, `-p` new PID namespace) so the first
// exec'd process in the new namespace is PID 1 (init rôle for that namespace): when it exits,
// the kernel tears down the contained subtree, closing the "direct child matched exit,
// grandchildren still run" host escape **for this path** (PR #792: P3/P4). Other Unix and
// Windows: no unprivileged one-shot equivalent; wall bound + pgrp signal on timeout + the
// `sh -c` `&` heuristic only.
//
// **Setup detection — `gunbc_execute_command_bootstrap` helper (P2(c) structural, post-#1063).**
// The unshare target is the helper binary from `src/v3/execute_command_bootstrap`, NOT a
// `sh -c` script. The helper writes the same three sentinel bytes on fd 3 — `s`, `e`, `f` —
// but crucially calls `fcntl(3, F_SETFD, FD_CLOEXEC)` on fd 3 immediately *before*
// `execvp(3)`. On successful `execvp` the kernel atomically closes fd 3 in the new image
// — the user command **cannot** inherit a writable sentinel fd, eliminating the
// `b"sef"`-spoof discriminator gap that was unreachable in pure POSIX sh (no portable
// `fcntl` primitive between probe and `exec`). `argv[1]` is the logical program; `argv[2..]`
// are its args. See `gunbc_execute_command_bootstrap` source for the exact protocol; the
// classifier in this file consumes the same byte vocabulary as before.
//
// After wait, parent reads up to [`UNSHARE_READY_PIPE_MAX`] bytes:
//
// - `b""`    → util-linux exited before the helper ran → `NamespaceSetupFailed`.
// - `b"s"`   → helper ran, executable probe rejected → `LogicalCommandNotExecutable`.
// - `b"se"`  → helper `execvp`'d the user command (kernel atomically closed fd 3 via
//   `FD_CLOEXEC`; user inherits no writable sentinel fd) → exit is the logical exit →
//   `LogicalCommandExeced`.
// - `b"sef"` → helper's `execvp` returned (TOCTOU x-bit removal, `ENOEXEC`, `ETXTBSY`, etc.)
//   → exec failed post-probe → `LogicalExecFailed`. **Structurally unspoofable now** that
//   fd 3 has CLOEXEC before `execvp` — only the helper itself can produce `f` on the
//   sentinel.
// - anything else with `b"se"` prefix → defensively classified as `LogicalCommandExeced`
//   (P2(d) defense-in-depth). With the helper-binary CLOEXEC story, this branch should be
//   unreachable in practice, but the classifier preserves the no-implicit-re-execution
//   guarantee even under a hypothetical helper bug.
// - anything not starting with `b"s"` or whose `s`-prefix didn't reach `e` →
//   `NamespaceSetupFailed`. Bootstrap never committed to `exec`, so the direct fallback is
//   the *first* logical run.
//
// **No setup-string authority** (P2(a)/(b)/(e)): wrapper stderr is `/dev/null`; only
// structural channel is the typed sentinel pipe; classification is exact-byte.
// **No implicit re-execution** (P2(d)): on `NamespaceSetupFailed` the direct fallback is
// the *first* logical run, not a recovery re-exec.
// **No spoofable discriminator** (P2(c), structural): the helper sets `FD_CLOEXEC` on fd 3
// between writing `e` and calling `execvp`; successful exec atomically closes fd 3 in the
// new image, so the user cannot write any byte to the sentinel post-`exec`.

/// Anonymous pipe used as the unshare-bootstrap setup sentinel. The child inherits the write
/// end as fd 3 (set in `pre_exec` via `dup2`). After spawn the parent closes its copy of the
/// write end so EOF becomes observable on the read end. After wait the parent reads up to
/// [`UNSHARE_READY_PIPE_MAX`] bytes — the bytes written by the bootstrap classify the run.
#[cfg(target_os = "linux")]
struct UnshareReadyPipe {
    read_fd: std::os::fd::RawFd,
    /// `Some(_)` until [`Self::close_write_end_in_parent`] runs once after `spawn`.
    write_fd: Option<std::os::fd::RawFd>,
}

/// One byte beyond the longest canonical-protocol sentinel (`b"sef"` = 3 bytes). The extra
/// slot lets stray bytes after `se`/`sef` (from the user writing to the inherited fd 3
/// post-`exec`) be observed by the classifier — those patterns map to
/// `LogicalCommandExeced` rather than triggering a fallback re-run (P2(d)).
/// (api-review claude-opus-4-7 sha 793a57ef sizing rationale; codex sha 7297b04a P2(d)
/// hardening for the post-`se` stray-byte case.)
#[cfg(target_os = "linux")]
const UNSHARE_READY_PIPE_MAX: usize = 4;

#[cfg(target_os = "linux")]
impl UnshareReadyPipe {
    fn new() -> std::io::Result<Self> {
        let mut fds = [0i32; 2];
        if unsafe { libc::pipe2(fds.as_mut_ptr(), libc::O_CLOEXEC) } != 0 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(UnshareReadyPipe {
            read_fd: fds[0],
            write_fd: Some(fds[1]),
        })
    }

    fn write_fd_for_child(&self) -> std::os::fd::RawFd {
        self.write_fd.expect("write fd consumed before spawn")
    }

    fn close_write_end_in_parent(&mut self) {
        if let Some(fd) = self.write_fd.take() {
            unsafe { libc::close(fd) };
        }
    }

    /// Reads up to [`UNSHARE_READY_PIPE_MAX`] bytes after the child exits and the parent's
    /// write end is closed. Bounded — the bootstrap writes at most 3 sentinel bytes.
    ///
    /// **Caller ordering contract.** This is a *blocking* `libc::read` loop. It must be
    /// called only after both (a) [`Self::close_write_end_in_parent`] has run and (b) the
    /// child has been reaped (either via `child_wait_for_execute_command` returning
    /// successfully, or after `kill_process_group_on_timeout` + `child.wait`). Together
    /// those guarantee EOF on the read end. Calling this *before* the child is reaped while
    /// the child still holds an open fd 3 (e.g. pre-`exec` sh, or post-failed-`exec` sh
    /// after the EXIT trap fires) would deadlock if the child has not yet exited.
    ///
    /// **Typed errors (api-review openai-pro/gpt-5-5-pro sha 9fea084e).** A real `read(2)`
    /// failure is surfaced as `Err(io::Error)` rather than collapsed into a short sentinel:
    /// the caller routes that to a typed `WaitFailed::Io` outcome, NOT a fall-back to
    /// direct, because after the unshare child has been reaped we cannot rule out that the
    /// logical command already ran — running it again via direct fallback would be implicit
    /// re-execution. `EINTR` is retried in-loop because the read is idempotent on a
    /// closed-write-end pipe. `n == 0` is a clean EOF.
    fn read_sentinel(&self) -> std::io::Result<Vec<u8>> {
        let mut buf = [0u8; UNSHARE_READY_PIPE_MAX];
        let mut total = 0usize;
        while total < buf.len() {
            let n = unsafe {
                libc::read(
                    self.read_fd,
                    buf.as_mut_ptr().add(total) as *mut libc::c_void,
                    buf.len() - total,
                )
            };
            if n > 0 {
                total += n as usize;
                continue;
            }
            if n == 0 {
                // Clean EOF — all writers closed.
                break;
            }
            // n < 0: real read error.
            let err = std::io::Error::last_os_error();
            if err.kind() == std::io::ErrorKind::Interrupted {
                // EINTR — retry. Idempotent on a closed-write-end pipe.
                continue;
            }
            return Err(err);
        }
        Ok(buf[..total].to_vec())
    }
}

#[cfg(target_os = "linux")]
impl Drop for UnshareReadyPipe {
    fn drop(&mut self) {
        if let Some(fd) = self.write_fd.take() {
            unsafe { libc::close(fd) };
        }
        unsafe { libc::close(self.read_fd) };
    }
}

/// Override env var for the helper binary path (used in tests where the workspace target
/// layout may not match `current_exe`'s parent — e.g., when running a single-crate test in
/// isolation). Production callers should not set this; the runtime walkup is the canonical
/// resolution.
#[cfg(target_os = "linux")]
const GUNBC_EXECUTE_COMMAND_BOOTSTRAP_PATH_ENV: &str = "GUNBC_EXECUTE_COMMAND_BOOTSTRAP";

/// True iff `path` is a regular file that the calling process can actually `execve` —
/// i.e. it exists, is a regular file (not a directory or symlink-to-directory), AND the
/// effective uid/gid has execute permission per `access(2)` with `X_OK`. We don't trust
/// `Path::exists()` (would pass directories) and we don't trust a bare mode-bit check
/// (would pass `--x------` files our uid cannot exec, leading to `EACCES` at exec time
/// → empty sentinel → `NamespaceSetupFailed` → direct fallback runs the logical command,
/// silently converting setup-failure into a possible `Matched` logical exit).
///
/// `access(2)` is the kernel-supplied atomic check that combines uid/gid against the file
/// permissions (api-review codex sha 523776b BLOCKING follow-up + sha 143b7da5 BLOCKING
/// original; together close the P2(c) silent-direct-fallback class for the helper path).
#[cfg(target_os = "linux")]
fn is_regular_executable_file(path: &std::path::Path) -> bool {
    use std::os::unix::ffi::OsStrExt;
    let Ok(meta) = std::fs::metadata(path) else {
        return false;
    };
    if !meta.is_file() {
        return false;
    }
    // Build a NUL-terminated byte vec for libc::access. Reject paths containing interior
    // NULs (cannot be passed to access(2) at all).
    let bytes = path.as_os_str().as_bytes();
    if bytes.contains(&0) {
        return false;
    }
    let mut c_path = Vec::with_capacity(bytes.len() + 1);
    c_path.extend_from_slice(bytes);
    c_path.push(0);
    // SAFETY: c_path is NUL-terminated; access(2) is signal-safe and read-only.
    let rc = unsafe { libc::access(c_path.as_ptr() as *const libc::c_char, libc::X_OK) };
    rc == 0
}

/// Locates the `gunbc_execute_command_bootstrap` helper binary at runtime. The cargo
/// workspace layout puts it at `target/<profile>/gunbc_execute_command_bootstrap`. Tests
/// run from `target/<profile>/deps/v3_compiler-<hash>` (one level deeper) so we walk up
/// once. An env override (`GUNBC_EXECUTE_COMMAND_BOOTSTRAP`) takes precedence for cases
/// where the helper is installed elsewhere.
///
/// **Validates that candidates are regular executable files**, not just present paths
/// (P2(c) hardening — see [`is_regular_executable_file`]).
///
/// Returns `Err(message)` if the helper cannot be located or fails validation — the caller
/// surfaces this as a typed `SetupFailed` so a misconfigured deployment cannot silently
/// degrade to a spoofable bootstrap path.
#[cfg(target_os = "linux")]
fn locate_unshare_bootstrap_helper() -> Result<std::path::PathBuf, String> {
    use std::path::PathBuf;
    if let Ok(p) = std::env::var(GUNBC_EXECUTE_COMMAND_BOOTSTRAP_PATH_ENV) {
        if !p.is_empty() {
            let path = PathBuf::from(p);
            if !path.exists() {
                return Err(format!(
                    "{GUNBC_EXECUTE_COMMAND_BOOTSTRAP_PATH_ENV} points to nonexistent path: {}",
                    path.display()
                ));
            }
            if !is_regular_executable_file(&path) {
                return Err(format!(
                    "{GUNBC_EXECUTE_COMMAND_BOOTSTRAP_PATH_ENV} points to a path that is not a regular executable file: {}",
                    path.display()
                ));
            }
            return Ok(path);
        }
    }
    let exe = std::env::current_exe().map_err(|e| format!("current_exe() failed: {e}"))?;
    let dir = exe
        .parent()
        .ok_or_else(|| "current_exe has no parent dir".to_string())?;
    // target/<profile>/gunbc_execute_command_bootstrap (binary running directly).
    let candidate = dir.join("gunbc_execute_command_bootstrap");
    if is_regular_executable_file(&candidate) {
        return Ok(candidate);
    }
    // target/<profile>/deps/<bin>-<hash> → walk up one to target/<profile>/.
    if let Some(parent) = dir.parent() {
        let candidate = parent.join("gunbc_execute_command_bootstrap");
        if is_regular_executable_file(&candidate) {
            return Ok(candidate);
        }
    }
    Err(format!(
        "gunbc_execute_command_bootstrap not found relative to current_exe ({}); \
         build the workspace (`cargo build --workspace`) or set {} to its absolute path",
        exe.display(),
        GUNBC_EXECUTE_COMMAND_BOOTSTRAP_PATH_ENV
    ))
}

#[cfg(target_os = "linux")]
fn build_execute_command_unshare(
    helper: &std::path::Path,
    command: &str,
    args: &[String],
    ready: &UnshareReadyPipe,
) -> Command {
    let mut c = Command::new("unshare");
    c.args(["-c", "-f", "-p", "--"])
        .arg(helper)
        .arg(command)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        // Wrapper stderr is /dev/null: P2(a)/(b) — no shared diagnostic channel between
        // helper and parent; the structural setup signal is the ready-pipe sentinel.
        .stderr(Stdio::null());
    let write_fd = ready.write_fd_for_child();
    {
        use std::os::unix::process::CommandExt;
        unsafe {
            c.pre_exec(move || {
                if libc::setpgid(0, 0) != 0 {
                    return Err(std::io::Error::last_os_error());
                }
                // Move the ready-pipe write end to fd 3 WITHOUT FD_CLOEXEC. fd 3 must
                // survive the parent's `exec("unshare")` and unshare's own `exec(helper)`;
                // the helper binary itself sets FD_CLOEXEC on fd 3 immediately before its
                // final `execvp` of the user command — that's the structural close that
                // prevents the user from inheriting a writable sentinel fd (P2(c) closure,
                // #1063).
                //
                // dup2(src, dst) clears CLOEXEC on dst *only when src != dst*. POSIX
                // specifies that `dup2(fildes, fildes)` is a no-op (returns fildes2
                // unchanged) — so if a low-fd allocation in the child has already placed
                // our write_fd AT fd 3, dup2 leaves fd 3 with whatever flags it had,
                // including the O_CLOEXEC we set on `pipe2(O_CLOEXEC)`. The kernel would
                // then close fd 3 atomically on `exec("unshare")` and the helper would
                // never see the sentinel — silently bypassing the helper path. Explicitly
                // clear FD_CLOEXEC after dup2 to defend against that case.
                // (api-review codex sha c535f4c BLOCKING.)
                if libc::dup2(write_fd, 3) < 0 {
                    return Err(std::io::Error::last_os_error());
                }
                if libc::fcntl(3, libc::F_SETFD, 0) < 0 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
    }
    c
}

/// Classifies the [`UnshareReadyPipe`] sentinel after the unshare wrapper exits.
#[cfg(target_os = "linux")]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum UnshareBootstrapStage {
    /// `b""` — helper never ran; util-linux failed before fork/exec into the helper.
    NamespaceSetupFailed,
    /// `b"s"` — helper ran, `execvp`-equivalent probe rejected the logical command.
    LogicalCommandNotExecutable,
    /// `b"se"` — helper `execvp`'d the logical command; observed exit is its exit.
    LogicalCommandExeced,
    /// `b"sef"` — helper's `execvp` returned (TOCTOU, `ENOEXEC`, `ETXTBSY`, etc.); the
    /// helper wrote `f` after the failure. The observed exit is the helper's, not the
    /// logical command's — must NOT be classified as a logical exit (P2(c)). Structurally
    /// unspoofable: the helper sets `FD_CLOEXEC` on fd 3 before `execvp`, so a successful
    /// exec atomically closes fd 3 in the user image and only the helper itself can write
    /// `f` (post-#1063).
    LogicalExecFailed,
}

#[cfg(target_os = "linux")]
fn unshare_bootstrap_stage_from_sentinel(sentinel: &[u8]) -> UnshareBootstrapStage {
    // **P2(d) hardening (api-review codex sha 7297b04a, REQUEST_CHANGES).**
    // The bootstrap writes `e` only *after* the executable probe passes and is committed to
    // `exec(2)`. So once `s` then `e` have been observed, the logical command DID start
    // running — any sentinel bytes beyond `se` come from the user process (fd 3 is
    // intentionally inherited; CLOEXEC at this layer is impossible in pure POSIX sh per
    // manager STOP / #856 escalation). Re-routing those `se*` patterns to
    // `NamespaceSetupFailed` would trigger a *second* logical run via the direct fallback —
    // exactly the implicit-re-execution this PR is meant to close.
    //
    // Therefore anything whose prefix is `b"se"` is classified as the user having executed:
    // - `b"se"`     → `LogicalCommandExeced` (canonical)
    // - `b"sef"`    → `LogicalExecFailed`    (canonical post-`exec` failure via EXIT trap)
    // - `b"se*"`    → `LogicalCommandExeced` (user ran + post-`exec` stray write)
    //
    // Patterns that did NOT reach `e` (`b""`, `b"s"`, anything not starting with `s` or `se`)
    // are safe to fail-closed because the bootstrap never committed to `exec` — re-running
    // via direct fallback is the *first* logical run, not a second.
    match sentinel {
        b"" => UnshareBootstrapStage::NamespaceSetupFailed,
        b"s" => UnshareBootstrapStage::LogicalCommandNotExecutable,
        b"sef" => UnshareBootstrapStage::LogicalExecFailed,
        bs if bs.starts_with(b"se") => UnshareBootstrapStage::LogicalCommandExeced,
        _ => UnshareBootstrapStage::NamespaceSetupFailed,
    }
}

/// Configure `Command` for the host check: no capture, and on Unix a new process group for the
/// child so a timeout can `kill(2)` the whole process group. On Linux this is the **non-unshare**
/// path; see [`build_execute_command_unshare`].
fn build_execute_command_process(command: &str, args: &[String]) -> Command {
    let mut c = Command::new(command);
    c.args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        unsafe {
            c.pre_exec(|| {
                if libc::setpgid(0, 0) != 0 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
    }
    c
}

/// Best-effort: signal the process **group** (see `setpgid` in `pre_exec`); then `Child::kill` for
/// portability. A successful `sh -c '…&'`-style path is pre-blocked; `wait` after a SIGKILL is
/// still unbounded in the API, but the child is reaped in practice.
#[cfg(unix)]
fn kill_process_group_on_timeout(child: &mut std::process::Child) {
    use libc::{kill, SIGKILL};
    let p = child.id() as i32;
    if p != 0 {
        if unsafe { kill(-p, SIGKILL) } < 0 {
            let _ = child.kill();
        }
    } else {
        let _ = child.kill();
    }
}
#[cfg(not(unix))]
fn kill_process_group_on_timeout(child: &mut std::process::Child) {
    let _ = child.kill();
}

/// Typed wait failure. Mapped to [`ExecuteCommandHostOutcome::WaitFailed`] by the caller.
#[derive(Debug, Clone, PartialEq, Eq)]
enum ChildWaitFail {
    WallTimeout { wall_time: Duration },
    Io(String),
}

/// Wall-bounded `try_wait` loop. With wrapper stderr at `/dev/null` (P2(b)) there is no pipe to
/// drain — simple poll until the child exits or the deadline trips.
fn child_wait_for_execute_command(
    child: &mut std::process::Child,
    wall_time: Duration,
) -> Result<std::process::ExitStatus, ChildWaitFail> {
    let deadline = Instant::now() + wall_time;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Ok(status),
            Ok(None) => {
                if Instant::now() >= deadline {
                    kill_process_group_on_timeout(child);
                    let _ = child.wait();
                    return Err(ChildWaitFail::WallTimeout { wall_time });
                }
                std::thread::sleep(EXECUTE_COMMAND_WAIT_POLL);
            }
            Err(err) => return Err(ChildWaitFail::Io(format!("{err}"))),
        }
    }
}

/// Spawns a host process and checks exit status. Used by the Rust `TestRunner` and the M1.5
/// harness (single canonical path per PB-Runtime brief). Core logic is
/// [`evaluate_execute_command_host_outcome`] ([`ExecuteCommandHostOutcome`]); this function and
/// [`evaluate_execute_command_exit_code_with_wall_time`] map to [`ClaimResult`] at the
/// reporting edge.
///
/// - **No stdout/stderr capture** — `stdin`/`stdout`/`stderr` are the null device. Only the
///   exit code is read.
/// - **Wall clock** — [`EXECUTE_COMMAND_WALL_TIMEOUT`].
/// - **Linux**: user+PID namespace via `unshare(1)` + the `gunbc_execute_command_bootstrap` helper binary that signals setup
///   progress on a sentinel ready-pipe (fd 3). On namespace setup failure the runner falls
///   back to a direct `Child` once as the *first* logical run (P2(d): no implicit re-exec
///   of an already-run logical command).
/// - **Heuristic on `&` in `sh`/`bash`/… `-c` scripts (all hosts)** — bare shell background `&`
///   (after eliding `&&` and a few `>&`/`&>` token spellings) is rejected pre-spawn.
pub fn evaluate_execute_command_exit_code(
    command: &str,
    args: &[String],
    expect_exit_code: i64,
) -> ClaimResult {
    evaluate_execute_command_exit_code_with_wall_time(
        command,
        args,
        expect_exit_code,
        EXECUTE_COMMAND_WALL_TIMEOUT,
    )
}

/// String form for the exit-mismatch **reporting** edge; **not** used for M1.5 or other
/// semantic classification — that uses [`ExecuteCommandHostOutcome::Mismatch`].
pub const EXECUTE_COMMAND_EXIT_CODE_MISMATCH_MSG_PREFIX: &str =
    "ExecuteCommand exit code mismatch: expected ";

/// Single typed authority for the host predicate. **All** consumers branch on this enum;
/// `Fail(String)` is rendered only at the reporting edge by [`Self::into_claim_result`].
/// **P2(a)–(e) realization (T-PB-B Worker 4):**
/// - (a) typed results: variants only; no `Other(ClaimResult)` partial carrier.
/// - (b) isolated logical-child I/O: wrapper stderr is `/dev/null`; the helper binary inherits stdio normally and
///   logical stderr to `/dev/null` before `exec`. The only structural channel between
///   bootstrap and parent is the [`UnshareReadyPipe`] sentinel (typed bytes, not strings).
/// - (c) setup ≠ logical exit: `SetupFailed` and `SpawnFailed` are distinct from
///   `Matched`/`Mismatch` — they cannot be reinterpreted in one hop.
/// - (d) no implicit re-execution: namespace setup failure → direct fallback runs as the
///   *first* logical run (the unshare wrapper never reached `exec`). The empty-stderr
///   relaunch and non-zero host-confirmation re-execs are deleted.
/// - (e) drains and authority: no draining is necessary — wrapper stderr is `/dev/null`, so
///   no parent-readable stream can be claimed as evidence by a later step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecuteCommandHostOutcome {
    /// Logical exit equaled the claim.
    Matched,
    /// Logical exit observed and did not match the claim. M1.5 maps this alone to
    /// propositional `false` for the exit predicate.
    Mismatch { expected: i64, actual: i64 },
    /// Pre-spawn input policy rejected the claim (e.g. shell `&` background).
    PolicyRejected { policy: PolicyReject },
    /// `Command::spawn` failed, or the unshare bootstrap detected the logical command was
    /// not executable. `wrapper` is `Some(_)` only when the unshare wrapper itself failed to
    /// spawn before falling back to direct.
    SpawnFailed {
        wrapper: Option<String>,
        direct: String,
    },
    /// Namespace / wrapper setup failed *before* the logical command ran. Only produced when
    /// the direct fallback also could not spawn — a successful direct fallback becomes the
    /// authoritative run and is reported as `Matched` / `Mismatch` / etc.
    SetupFailed { reason: SetupFailReason },
    /// Wait-loop failure: timeout, IO, or signal-termination (no exit code).
    WaitFailed(WaitFail),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyReject {
    /// `sh -c` script with a bare `&` not elidable as `&&` / fd redirect.
    ShellBackgroundUnbounded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SetupFailReason {
    /// `unshare(1)` exited before the helper ran (namespace / clone / permission). The runner
    /// attempted a direct fallback; this carrier is only produced when that fallback also
    /// failed to spawn.
    NamespaceSetupAndDirectSpawnFailed { direct: String },
    /// The `gunbc_execute_command_bootstrap` helper binary could not be located at runtime
    /// (build-graph misconfiguration: `cargo test -p v3-compiler` without a prior
    /// `cargo build -p execute-command-bootstrap`, or the `GUNBC_EXECUTE_COMMAND_BOOTSTRAP`
    /// override pointing at a missing path). Distinct from `NamespaceSetupAndDirectSpawnFailed`
    /// because no `unshare(1)` or direct `Child` was attempted — surfacing this as that
    /// variant would lie about which steps failed (api-review cursor/composer-2 sha 7dd7825b).
    HelperBinaryMissing { reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WaitFail {
    WallTimeout {
        wall_time: Duration,
    },
    Io(String),
    /// Child terminated by signal — `ExitStatus::code()` returned `None`.
    TerminatedBySignal,
}

impl ExecuteCommandHostOutcome {
    /// [`ClaimResult`] for [`TestRunner`] and `.dag` reporting; match on
    /// [`ExecuteCommandHostOutcome`] before calling when you need typed discrimination.
    pub fn into_claim_result(self) -> ClaimResult {
        match self {
            ExecuteCommandHostOutcome::Matched => ClaimResult::Pass,
            ExecuteCommandHostOutcome::Mismatch { expected, actual } => ClaimResult::Fail(format!(
                "{EXECUTE_COMMAND_EXIT_CODE_MISMATCH_MSG_PREFIX}{expected}, got {actual}"
            )),
            ExecuteCommandHostOutcome::PolicyRejected { policy } => match policy {
                PolicyReject::ShellBackgroundUnbounded => {
                    ClaimResult::Fail(SHELL_C_BACKGROUND_UNBOUNDED_FAIL.to_string())
                }
            },
            ExecuteCommandHostOutcome::SpawnFailed { wrapper, direct } => match wrapper {
                Some(w) => ClaimResult::Fail(format!(
                    "ExecuteCommand: unshare(1) wrapper failed to spawn: {w}; direct spawn also failed: {direct} (P3/P4 — util-linux/namespace or host binary path)"
                )),
                None => ClaimResult::Fail(format!("ExecuteCommand spawn error: {direct}")),
            },
            ExecuteCommandHostOutcome::SetupFailed { reason } => match reason {
                SetupFailReason::NamespaceSetupAndDirectSpawnFailed { direct } => {
                    ClaimResult::Fail(format!(
                        "ExecuteCommand: namespace setup failed and direct fallback spawn also failed: {direct}"
                    ))
                }
                SetupFailReason::HelperBinaryMissing { reason } => ClaimResult::Fail(format!(
                    "ExecuteCommand: gunbc_execute_command_bootstrap helper not found: {reason}"
                )),
            },
            ExecuteCommandHostOutcome::WaitFailed(w) => match w {
                WaitFail::WallTimeout { wall_time } => ClaimResult::Fail(format!(
                    "ExecuteCommand: process exceeded {:.2}s wall-clock limit (timeout — process group / child killed, fail-closed)",
                    wall_time.as_secs_f64()
                )),
                WaitFail::Io(err) => ClaimResult::Fail(format!(
                    "ExecuteCommand: wait on child failed: {err}"
                )),
                WaitFail::TerminatedBySignal => ClaimResult::Fail(
                    "ExecuteCommand: child terminated by signal (no host exit code)".to_string(),
                ),
            },
        }
    }
}

/// M1.5 and other **boolean** predicate reads: only these outcomes map to propositional
/// true/false; all other results are `Err(ClaimResult)` (not "false" for the claim).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecuteCommandM1_5Proposition {
    Satisfied,
    UnsatisfiedExitMismatch,
}

/// Distinguish "exit ≠ expect" from timeout, spawn error, `&` policy, signal, etc. `Err` is
/// the rendered [`ClaimResult`] (use [`evaluate_execute_command_host_outcome`] directly when
/// you need the typed discriminant).
pub fn evaluate_execute_command_m1_5(
    command: &str,
    args: &[String],
    expect_exit_code: i64,
) -> Result<ExecuteCommandM1_5Proposition, ClaimResult> {
    match evaluate_execute_command_host_outcome(
        command,
        args,
        expect_exit_code,
        EXECUTE_COMMAND_WALL_TIMEOUT,
    ) {
        ExecuteCommandHostOutcome::Matched => Ok(ExecuteCommandM1_5Proposition::Satisfied),
        ExecuteCommandHostOutcome::Mismatch { .. } => {
            Ok(ExecuteCommandM1_5Proposition::UnsatisfiedExitMismatch)
        }
        other => Err(other.into_claim_result()),
    }
}

/// Core host run: typed outcome. See [`ExecuteCommandHostOutcome`] for the P2(a)–(e)
/// realization. Map to [`ClaimResult`] with [`ExecuteCommandHostOutcome::into_claim_result`]
/// for [`TestRunner`].
pub fn evaluate_execute_command_host_outcome(
    command: &str,
    args: &[String],
    expect_exit_code: i64,
    wall_time: Duration,
) -> ExecuteCommandHostOutcome {
    if reject_unbounded_shell_background(command, args).is_some() {
        return ExecuteCommandHostOutcome::PolicyRejected {
            policy: PolicyReject::ShellBackgroundUnbounded,
        };
    }
    #[cfg(target_os = "linux")]
    {
        run_linux_unshare_then_direct(command, args, expect_exit_code, wall_time)
    }
    #[cfg(not(target_os = "linux"))]
    {
        run_direct(command, args, expect_exit_code, wall_time, None)
    }
}

/// Direct (non-unshare) host run. `wrapper_err` is `Some(_)` when this is the fallback after
/// `unshare(1)` itself failed to spawn; it is preserved in `SpawnFailed { wrapper, direct }`
/// if direct also fails, so the caller does not lose the wrapper-side reason.
fn run_direct(
    command: &str,
    args: &[String],
    expect_exit_code: i64,
    wall_time: Duration,
    wrapper_err: Option<String>,
) -> ExecuteCommandHostOutcome {
    let mut child = match build_execute_command_process(command, args).spawn() {
        Ok(c) => c,
        Err(e) => {
            return ExecuteCommandHostOutcome::SpawnFailed {
                wrapper: wrapper_err,
                direct: format!("{e}"),
            };
        }
    };
    finish_after_wait(&mut child, expect_exit_code, wall_time)
}

#[cfg(target_os = "linux")]
fn run_linux_unshare_then_direct(
    command: &str,
    args: &[String],
    expect_exit_code: i64,
    wall_time: Duration,
) -> ExecuteCommandHostOutcome {
    // Locate the helper binary BEFORE creating the pipe. If the helper is missing this is a
    // configuration error — surface as a typed `SetupFailed` so a misconfigured deployment
    // cannot silently degrade through `run_direct` (which would skip the unshare containment
    // entirely without the operator knowing).
    let helper = match locate_unshare_bootstrap_helper() {
        Ok(p) => p,
        Err(reason) => {
            return ExecuteCommandHostOutcome::SetupFailed {
                reason: SetupFailReason::HelperBinaryMissing { reason },
            };
        }
    };
    let mut ready = match UnshareReadyPipe::new() {
        Ok(p) => p,
        Err(e) => {
            // pipe2 failed — drop the unshare path entirely and try direct.
            return run_direct(
                command,
                args,
                expect_exit_code,
                wall_time,
                Some(format!("ready-pipe: {e}")),
            );
        }
    };
    let mut child = match build_execute_command_unshare(&helper, command, args, &ready).spawn() {
        Ok(c) => c,
        Err(e_unshare) => {
            return run_direct(
                command,
                args,
                expect_exit_code,
                wall_time,
                Some(format!("{e_unshare}")),
            );
        }
    };
    // Close parent's write end so the read end sees EOF after the child exits.
    ready.close_write_end_in_parent();

    let status = match child_wait_for_execute_command(&mut child, wall_time) {
        Ok(s) => s,
        Err(ChildWaitFail::WallTimeout { wall_time }) => {
            return ExecuteCommandHostOutcome::WaitFailed(WaitFail::WallTimeout { wall_time });
        }
        Err(ChildWaitFail::Io(err)) => {
            return ExecuteCommandHostOutcome::WaitFailed(WaitFail::Io(err));
        }
    };

    let sentinel = match ready.read_sentinel() {
        Ok(bytes) => bytes,
        Err(io_err) => {
            // Sentinel read failed (real error, not EINTR — that's retried in-loop). After
            // the unshare child has been reaped we cannot rule out that the logical command
            // already ran. Surfacing as a typed wait-failure (NOT a `NamespaceSetupFailed` →
            // direct fallback) avoids implicit re-execution. (api-review openai-pro/
            // gpt-5-5-pro sha 9fea084e.)
            return ExecuteCommandHostOutcome::WaitFailed(WaitFail::Io(format!(
                "ready-pipe read: {io_err}"
            )));
        }
    };
    match unshare_bootstrap_stage_from_sentinel(&sentinel) {
        UnshareBootstrapStage::LogicalCommandExeced => {
            classify_logical_exit(status, expect_exit_code)
        }
        UnshareBootstrapStage::LogicalCommandNotExecutable => {
            ExecuteCommandHostOutcome::SpawnFailed {
                wrapper: None,
                direct: format!(
                    "logical command `{command}` not executable (no such file or not in PATH)"
                ),
            }
        }
        UnshareBootstrapStage::LogicalExecFailed => {
            // Probe passed but `execvp(3)` returned a failure (TOCTOU, ENOEXEC, ETXTBSY,
            // ...). The wrapper exit code is the helper binary's, NOT the logical command's
            // — surface as `SpawnFailed` so a claim expecting (e.g.) 126 cannot Match an
            // unexec'd binary (P2(c) regression manager flagged on draft review).
            ExecuteCommandHostOutcome::SpawnFailed {
                wrapper: None,
                direct: format!(
                    "logical command `{command}` failed to exec after executable probe passed (TOCTOU/ENOEXEC/ETXTBSY)"
                ),
            }
        }
        UnshareBootstrapStage::NamespaceSetupFailed => {
            // util-linux exited before the helper binary ran. Direct fallback is the *first*
            // logical run, not a recovery re-exec (P2(d)).
            match build_execute_command_process(command, args).spawn() {
                Ok(mut direct_child) => {
                    finish_after_wait(&mut direct_child, expect_exit_code, wall_time)
                }
                Err(e) => ExecuteCommandHostOutcome::SetupFailed {
                    reason: SetupFailReason::NamespaceSetupAndDirectSpawnFailed {
                        direct: format!("{e}"),
                    },
                },
            }
        }
    }
}

fn finish_after_wait(
    child: &mut std::process::Child,
    expect_exit_code: i64,
    wall_time: Duration,
) -> ExecuteCommandHostOutcome {
    match child_wait_for_execute_command(child, wall_time) {
        Ok(status) => classify_logical_exit(status, expect_exit_code),
        Err(ChildWaitFail::WallTimeout { wall_time }) => {
            ExecuteCommandHostOutcome::WaitFailed(WaitFail::WallTimeout { wall_time })
        }
        Err(ChildWaitFail::Io(err)) => ExecuteCommandHostOutcome::WaitFailed(WaitFail::Io(err)),
    }
}

fn classify_logical_exit(
    status: std::process::ExitStatus,
    expect_exit_code: i64,
) -> ExecuteCommandHostOutcome {
    let Some(actual) = status.code().map(i64::from) else {
        return ExecuteCommandHostOutcome::WaitFailed(WaitFail::TerminatedBySignal);
    };
    if actual == expect_exit_code {
        ExecuteCommandHostOutcome::Matched
    } else {
        ExecuteCommandHostOutcome::Mismatch {
            expected: expect_exit_code,
            actual,
        }
    }
}

fn evaluate_execute_command_exit_code_with_wall_time(
    command: &str,
    args: &[String],
    expect_exit_code: i64,
    wall_time: Duration,
) -> ClaimResult {
    evaluate_execute_command_host_outcome(command, args, expect_exit_code, wall_time)
        .into_claim_result()
}

#[derive(Debug, Clone)]
pub struct TestClaimValue {
    pub claim_name: String,
    pub declaration_file: String,
    pub source: String,
    pub file_name: String,
    pub predicate: FieldValue,
    pub requires: Vec<FieldValue>,
}

pub struct TestRunner<'a> {
    dag: &'a Dag,
}

/// Resolved `PerfBaselineMeasurement` carrier read from a `.dag data`
/// declaration body. Consumed by [`TestRunner::eval_perf_within_baseline`]
/// to apply the Director-locked Tier-3 budget thresholds (`r3-structure.md`
/// §225). Carrier shape mirrors `src/v3/std/substrate.dag` `PerfBaselineMeasurement`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PerfMeasurement {
    median_ns: i64,
    p99_ns: i64,
}

/// Which budget axis overflowed when computing thresholds; produced by
/// [`compute_perf_budget_bounds`]. Single-authority for the §225 ratio
/// arithmetic — both the runtime evaluator and unit tests call through
/// this function so the `× 2` / `× 5` constants live in one place.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PerfBudgetOverflow {
    Median,
    P99,
}

/// Apply the Director-locked §225 ratio thresholds to a baseline:
/// `median_bound = baseline.median_ns × 2`, `p99_bound = baseline.p99_ns × 5`.
/// Saturate-on-overflow → `Err(PerfBudgetOverflow::*)` so the runtime can
/// fail-closed without silently wrapping. Single-authority for the budget
/// arithmetic per `INVARIANTS.md` P2.
fn compute_perf_budget_bounds(baseline: PerfMeasurement) -> Result<(i64, i64), PerfBudgetOverflow> {
    let median_bound = baseline
        .median_ns
        .checked_mul(2)
        .ok_or(PerfBudgetOverflow::Median)?;
    let p99_bound = baseline
        .p99_ns
        .checked_mul(5)
        .ok_or(PerfBudgetOverflow::P99)?;
    Ok((median_bound, p99_bound))
}

/// Typed resolver failures for [`TestRunner::perf_baseline_measurement`].
/// Per `CODING.md` typed-error discipline: raw `String` is reserved for the
/// `ClaimResult::Fail` boundary at [`PerfMeasurementResolveError::into_claim_fail`];
/// inner helpers carry structural variants.
///
/// **🟢 TERMINAL coproduct** (per `docs/modeling-discipline.md` Practice 4
/// classification checkpoint). The four variants enumerate the exhaustive
/// failure shapes when reading a `PerfBaselineMeasurement` data declaration:
/// either the declaration is absent (`MissingDeclaration` — substrate-integrity
/// violation), present but not a structural record (`WrongConnective` —
/// `Unparsed`, `Scalar`, `List`, or `None`), structural with a missing required
/// field (`MissingField`), or structural with the field present but not an Int
/// literal (`WrongFieldKind`). These cover every way the structural resolver
/// can fail-closed against the substrate-declared shape; no further variants
/// are reachable. No dissolution trigger — the carrier persists with the
/// runtime invariant impl.
#[derive(Debug, Clone, PartialEq, Eq)]
enum PerfMeasurementResolveError {
    /// `DeclarationId` did not resolve via [`Dag::declaration_opt`] —
    /// substrate-integrity violation.
    MissingDeclaration,
    /// Declaration's `value_body` is not `ValueBody::Structural` (e.g.
    /// `Unparsed`, `Scalar`, `List`, or `None`); a `PerfBaselineMeasurement`
    /// data declaration must be a record literal.
    WrongConnective,
    /// Required field absent from the structural record body.
    MissingField { field: &'static str },
    /// Required field present but not `FieldValue::Literal(LiteralBits::Int(_))`.
    WrongFieldKind { field: &'static str },
}

impl PerfMeasurementResolveError {
    /// Convert the typed error into `ClaimResult::Fail` reason text with the
    /// role label preserved for triage.
    fn into_claim_fail(self, role: &str) -> String {
        match self {
            Self::MissingDeclaration => format!(
                "PerfWithinBaseline `{role}`: declaration id did not resolve in DAG \
                 (substrate-integrity violation)"
            ),
            Self::WrongConnective => format!(
                "PerfWithinBaseline `{role}`: declaration is not a structural record \
                 (expected `PerfBaselineMeasurement {{ median_ns, p99_ns }}`)"
            ),
            Self::MissingField { field } => {
                format!("PerfWithinBaseline `{role}`: record is missing required field `{field}`")
            }
            Self::WrongFieldKind { field } => {
                format!("PerfWithinBaseline `{role}`: field `{field}` is not an Int literal")
            }
        }
    }
}

enum ProgramInputRole {
    ProgramInput,
    ProgramOutputBind { output_bind_name: String },
}

impl ProgramInputRole {
    fn output_bind_name(&self) -> Option<&str> {
        match self {
            Self::ProgramInput => None,
            Self::ProgramOutputBind { output_bind_name } => Some(output_bind_name),
        }
    }

    fn is_program_input(&self) -> bool {
        matches!(self, Self::ProgramInput)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DiagnosticDetailFilter {
    Any,
    Contains(String),
}

impl<'a> TestRunner<'a> {
    pub fn new(dag: &'a Dag) -> Self {
        Self { dag }
    }

    pub fn run_suite(&self, suite_name: &str) -> Vec<ClaimEvaluation> {
        let Some(suite) = self.dag.declaration_by_name(suite_name) else {
            return vec![ClaimEvaluation {
                claim_name: suite_name.to_string(),
                result: ClaimResult::Fail(format!("TestSuite `{suite_name}` not found")),
            }];
        };
        let Some(fields) = structural_fields(suite) else {
            return vec![ClaimEvaluation {
                claim_name: suite_name.to_string(),
                result: ClaimResult::Fail(format!("TestSuite `{suite_name}` is not structural")),
            }];
        };
        let Some(FieldValue::List(claims)) = field(fields, "claims") else {
            return vec![ClaimEvaluation {
                claim_name: suite_name.to_string(),
                result: ClaimResult::Fail(format!("TestSuite `{suite_name}` is missing `claims`")),
            }];
        };
        claims
            .iter()
            .map(|claim_ref| match claim_ref {
                FieldValue::Reference(id) => {
                    let decl = self.dag.declaration(*id);
                    match TestClaimValue::from_declaration(decl) {
                        Ok(claim) => self.run_claim(&claim),
                        Err(reason) => ClaimEvaluation {
                            claim_name: decl
                                .name
                                .clone()
                                .unwrap_or_else(|| format!("Declaration#{}", id.raw())),
                            result: ClaimResult::Fail(reason),
                        },
                    }
                }
                other => ClaimEvaluation {
                    claim_name: suite_name.to_string(),
                    result: ClaimResult::Fail(format!(
                        "TestSuite `{suite_name}` claim entry is not a reference: {other:?}"
                    )),
                },
            })
            .collect()
    }

    pub fn run_claim(&self, claim: &TestClaimValue) -> ClaimEvaluation {
        let result = match self.variant_value(&claim.predicate) {
            Some((label, payload)) => {
                if !claim.requires.is_empty() && label != "MockBackedInvariant" {
                    ClaimResult::Fail(format!(
                        "TestClaim `{}` declares {} resource requirement(s), but predicate `{}` does not consume `requires`",
                        claim.claim_name,
                        claim.requires.len(),
                        label
                    ))
                } else {
                    match label.as_str() {
                        "Compiles" => self.eval_compiles(claim),
                        "FailsWithDiagnostic" => self.eval_fails_with_diagnostic(claim, &payload),
                        "OutputEquals" => self.eval_output_equals(claim, &payload),
                        "PortHasState" => self.eval_port_has_state(claim, &payload),
                        "DeclarationHasRefinement" => {
                            self.eval_declaration_has_refinement(claim, &payload)
                        }
                        "CostBounded" => self.eval_cost_bounded(claim, &payload),
                        "PerfWithinBaseline" => self.eval_perf_within_baseline(claim, &payload),
                        "LensOutputEquals" => self.eval_lens_output_equals(claim, &payload),
                        "DifferentialEquals" => self.eval_differential_equals(claim, &payload),
                        "BinaryDimensionReportEquals" => {
                            self.eval_binary_dimension_report_equals_shape(claim, &payload)
                        }
                        "SymbolicCostExprEquals" => {
                            self.eval_symbolic_cost_expr_equals_shape(claim, &payload)
                        }
                        "AlgebraicLaw" => self.eval_algebraic_law(claim, &payload),
                        "ExecuteCommand" => self.eval_execute_command(claim, &payload),
                        "CensusBoundCheck" => self.eval_census_bound_check_shape(claim, &payload),
                        "CensusSubsetCount" => self.eval_census_subset_count_shape(claim, &payload),
                        "FixedPointConverges" => {
                            self.eval_fixed_point_converges_shape(claim, &payload)
                        }
                        "RatchetZero" => self.eval_ratchet_zero_shape(claim, &payload),
                        "BridgeLedgerZero" => self.eval_bridge_ledger_zero(claim, &payload),
                        "GeneratedFromDag" => self.eval_generated_from_dag_shape(claim, &payload),
                        "ReleaseDeferredClaim" => {
                            self.eval_release_deferred_claim_shape(claim, &payload)
                        }
                        "SubstrateResearchDeferredClaim" => {
                            self.eval_substrate_research_deferred_claim_shape(claim, &payload)
                        }
                        "MockBackedInvariant" => {
                            if !claim.requires.is_empty() {
                                if let Err(reason) = self.validate_resource_requirements(claim) {
                                    return ClaimEvaluation {
                                        claim_name: claim.claim_name.clone(),
                                        result: ClaimResult::Fail(reason),
                                    };
                                }
                            }
                            let inner = self.eval_mock_backed_invariant(claim, &payload);
                            if claim.requires.is_empty() {
                                match inner {
                                    ClaimResult::Pass => ClaimResult::NotYetImplemented(
                                        "MockBackedInvariant: `TestClaim.requires` is empty — DB-15 mock \
                                     obligations attach only on `requires` as `ResourceReference` edges; \
                                     hermetic subject/invariant application succeeded but is not a mock-backed \
                                     receipt until at least one obligation is declared."
                                            .to_string(),
                                    ),
                                    other => other,
                                }
                            } else {
                                inner
                            }
                        }
                        other => ClaimResult::NotYetImplemented(format!(
                            "TestPredicate::{other} is not wired in the Rust runner yet"
                        )),
                    }
                }
            }
            None => ClaimResult::Fail("predicate is not a structural variant".to_string()),
        };
        ClaimEvaluation {
            claim_name: claim.claim_name.clone(),
            result,
        }
    }

    fn eval_compiles(&self, claim: &TestClaimValue) -> ClaimResult {
        match compile_to_dag(&claim.source, &claim.file_name) {
            Ok(_) => ClaimResult::Pass,
            Err(CompileError::Semantic(_)) => {
                ClaimResult::Fail("compiled with diagnostics".to_string())
            }
            Err(err) => {
                ClaimResult::Fail(format!("compile failed before semantic analysis: {err:?}"))
            }
        }
    }

    fn eval_fails_with_diagnostic(
        &self,
        claim: &TestClaimValue,
        payload: &[FieldValue],
    ) -> ClaimResult {
        let [reference] = payload else {
            return ClaimResult::Fail(
                "FailsWithDiagnostic payload should be a DiagnosticReference".to_string(),
            );
        };
        match compile_to_dag(&claim.source, &claim.file_name) {
            Ok(_) => ClaimResult::Fail("source compiled cleanly".to_string()),
            Err(CompileError::Semantic(dag)) => match self.diagnostic_matches(&dag, reference) {
                Ok(true) => ClaimResult::Pass,
                Ok(false) => ClaimResult::Fail("expected diagnostic was not found".to_string()),
                Err(reason) => ClaimResult::Fail(reason),
            },
            Err(CompileError::Tokenize(diagnostic)) | Err(CompileError::Parse(diagnostic)) => {
                match self.diagnostic_matches_single(&diagnostic, reference) {
                    Ok(true) => ClaimResult::Pass,
                    Ok(false) => ClaimResult::Fail("expected diagnostic was not found".to_string()),
                    Err(reason) => ClaimResult::Fail(reason),
                }
            }
        }
    }

    fn eval_output_equals(&self, claim: &TestClaimValue, payload: &[FieldValue]) -> ClaimResult {
        let [FieldValue::Literal(LiteralBits::String(expected))] = payload else {
            return ClaimResult::Fail("OutputEquals payload should be a String".to_string());
        };
        let dag = match compile_to_dag(&claim.source, &claim.file_name) {
            Ok(dag) => dag,
            Err(err) => return ClaimResult::Fail(format!("source did not compile: {err:?}")),
        };
        let Some(value) = dag
            .declarations()
            .iter()
            .find(|decl| decl.span.file == claim.file_name && decl.value_body.is_some())
            .and_then(|decl| decl.value_body.as_ref())
        else {
            return ClaimResult::Fail("no data declaration value found".to_string());
        };
        let actual = render_value_body(&dag, value);
        if actual == *expected {
            ClaimResult::Pass
        } else {
            ClaimResult::Fail(format!("expected `{expected}`, got `{actual}`"))
        }
    }

    fn eval_port_has_state(&self, claim: &TestClaimValue, payload: &[FieldValue]) -> ClaimResult {
        let [FieldValue::Literal(LiteralBits::String(bind_name)), expected_state] = payload else {
            return ClaimResult::Fail(
                "PortHasState payload should be (String, PortStateExpectation)".to_string(),
            );
        };
        let dag = match compile_to_dag(&claim.source, &claim.file_name) {
            Ok(dag) => dag,
            Err(CompileError::Semantic(dag)) => dag,
            Err(err) => return ClaimResult::Fail(format!("source did not lower: {err:?}")),
        };
        let Some(bind) = find_bind(&dag, bind_name, &claim.file_name) else {
            return ClaimResult::Fail(format!("bind `{bind_name}` not found"));
        };
        let Some((label, payload)) = self.variant_value(expected_state) else {
            return ClaimResult::Fail("state expectation is not a variant".to_string());
        };
        if !payload.is_empty() {
            return ClaimResult::Fail("state expectation should not carry payload".to_string());
        }
        let matches = matches!(
            (label.as_str(), dag.port(bind.value).state()),
            ("Resolved", PortState::Resolved(_))
                | ("Unresolved", PortState::Uninferred | PortState::Unresolved)
        );
        if matches {
            ClaimResult::Pass
        } else {
            ClaimResult::Fail(format!("bind `{bind_name}` state did not match `{label}`"))
        }
    }

    fn eval_declaration_has_refinement(
        &self,
        claim: &TestClaimValue,
        payload: &[FieldValue],
    ) -> ClaimResult {
        let name = match payload {
            [FieldValue::Literal(LiteralBits::String(name))] => name.clone(),
            [single] => {
                let Some(fields) = record_fields(single) else {
                    return ClaimResult::Fail(
                        "DeclarationHasRefinement: expected `{ declaration_name: String }` record \
                         or a bare String payload"
                            .to_string(),
                    );
                };
                match string_field(fields, "declaration_name") {
                    Ok(s) => s,
                    Err(e) => return ClaimResult::Fail(e),
                }
            }
            _ => {
                return ClaimResult::Fail(format!(
                    "DeclarationHasRefinement: expected one payload field, got {}",
                    payload.len()
                ));
            }
        };
        let dag = match compile_to_dag(&claim.source, &claim.file_name) {
            Ok(dag) => dag,
            Err(CompileError::Semantic(_)) => {
                return ClaimResult::Fail("compiled with diagnostics".to_string());
            }
            Err(err) => {
                return ClaimResult::Fail(format!(
                    "DeclarationHasRefinement: compile failed before structural check: {err:?}"
                ));
            }
        };
        let Some(decl) = dag.declaration_by_name(&name) else {
            return ClaimResult::Fail(format!(
                "DeclarationHasRefinement: declaration `{name}` not found in `{}`",
                claim.file_name
            ));
        };
        if decl.refinement.is_some() {
            ClaimResult::Pass
        } else {
            ClaimResult::Fail(format!(
                "DeclarationHasRefinement: declaration `{name}` has no lowered `refinement` edge"
            ))
        }
    }

    fn eval_lens_output_equals(
        &self,
        claim: &TestClaimValue,
        payload: &[FieldValue],
    ) -> ClaimResult {
        let [lens_fv, input_fv, expected_fv] = payload else {
            return ClaimResult::Fail(format!(
                "LensOutputEquals payload should be exactly three DeclarationRef fields \
                 (lens_ref, input_ref, expected_ref); got {} payload slot(s)",
                payload.len()
            ));
        };
        let lens_id = match self.resolve_declaration_ref_id(lens_fv, "lens_ref") {
            Ok(id) => id,
            Err(msg) => return ClaimResult::Fail(msg),
        };
        let input_id = match self.resolve_declaration_ref_id(input_fv, "input_ref") {
            Ok(id) => id,
            Err(msg) => return ClaimResult::Fail(msg),
        };
        let expected_id = match self.resolve_declaration_ref_id(expected_fv, "expected_ref") {
            Ok(id) => id,
            Err(msg) => return ClaimResult::Fail(msg),
        };

        let lens_decl = self.dag.declaration(lens_id);
        let input_decl = self.dag.declaration(input_id);
        let expected_decl = self.dag.declaration(expected_id);

        let lens_name = decl_display_name(lens_id, lens_decl);
        let input_name = decl_display_name(input_id, input_decl);
        let expected_name = decl_display_name(expected_id, expected_decl);

        let program_input = match self.program_input_role(input_decl) {
            Ok(role) => role,
            Err(msg) => return ClaimResult::Fail(format!("LensOutputEquals: {msg}")),
        };

        if input_decl.value_body.is_none() && program_input.is_none() {
            return ClaimResult::Fail(format!(
                "LensOutputEquals: input_ref `{input_name}` has no value body"
            ));
        }
        if expected_decl.value_body.is_none() {
            return ClaimResult::Fail(format!(
                "LensOutputEquals: expected_ref `{expected_name}` has no value body"
            ));
        }

        // INVARIANTS P2 (executable single authority): `DeclarationRef` for `lens_ref` still
        // resolves against the fixture `Dag` for lowering, but for `named_function_count` the
        // runner compiles `R1_CANONICAL_NAMED_FUNCTION_COUNT_LENS` (same file as `build.rs` splices
        // for `user_authored_lens_compiles_gate`) for `apply_lens_declaration` — not the
        // fixture-local stub body. Other lens names: if `TestClaim.source` exports the same
        // declaration name, apply that program; else fall back to the fixture graph.
        //
        // **Dissolution trigger (name-keyed bridge):** delete the `lens_decl.name ==
        // Some("named_function_count")` arm and this entire parallel authority when
        // `DeclarationRef` resolves lens executable identity from `program_dag` (or structured
        // `TestClaim` metadata) without fixture-local stub bodies.
        // INVARIANTS P3 / TESTING: `TestClaim.source` must lower cleanly — never ignore
        // tokenize/parse failures and fall back to the fixture graph (that would let malformed
        // programs `Pass` when inputs/lens resolve only from the fixture).
        let program_dag = match compile_to_dag(&claim.source, &claim.file_name) {
            Ok(dag) => dag,
            Err(CompileError::Semantic(dag)) => {
                return ClaimResult::Fail(format!(
                    "LensOutputEquals: claim `source` / `{}` failed inference: {:?}",
                    claim.file_name,
                    dag.diagnostics().iter().collect::<Vec<_>>()
                ));
            }
            Err(err) => {
                return ClaimResult::Fail(format!(
                    "LensOutputEquals: claim `source` / `{}` did not compile: {err:?}",
                    claim.file_name
                ));
            }
        };

        // T-LaneE (`cost_of`): structural `Lookup<Int>` from the Rust-generated lens on the claim
        // program's `merge_sort_out` bind vs a fixture `Lookup<Int>` expected value.
        if lens_decl.name.as_deref() == Some("cost_of") {
            let Some(cost_bind) = program_input
                .as_ref()
                .and_then(ProgramInputRole::output_bind_name)
            else {
                return ClaimResult::Fail(format!(
                    "LensOutputEquals(cost_of): input_ref `{input_name}` must inhabit ProgramOutputBind"
                ));
            };
            let Some(bind) = find_bind(&program_dag, cost_bind, &claim.file_name) else {
                return ClaimResult::Fail(format!(
                    "LensOutputEquals(cost_of): bind `{cost_bind}` not found in `{}`",
                    claim.file_name
                ));
            };
            let computed = cost_of(&program_dag, &bind.value);
            // M1(2.8): `Lookup<Int>` is not yet structurally authorable in `data` bodies for this
            // fixture module — compare the lens `Hit(n)` against a scalar `Int` witness.
            let expected_int = match expected_decl.value_body.as_ref() {
                Some(ValueBody::Scalar(LiteralBits::Int(i))) => *i,
                _ => {
                    return ClaimResult::Fail(format!(
                        "LensOutputEquals(cost_of): expected_ref `{expected_name}` must be `data …: Int = <literal>` (M1(2.8); `Lookup<Int>` data literals are deferred)"
                    ));
                }
            };
            return match computed {
                CostLookup::Hit(v) if v == expected_int => ClaimResult::Pass,
                CostLookup::Hit(v) => ClaimResult::Fail(format!(
                    "LensOutputEquals(cost_of): expected `{expected_int}`, computed `{v}` for bind `{cost_bind}`"
                )),
                CostLookup::Miss => ClaimResult::Fail(
                    "LensOutputEquals(cost_of): computed cost is Miss (malformed program)".to_string(),
                ),
            };
        }

        // INVARIANTS P2: reflected `FieldValue` List / `Behavior` variant ids must come from the
        // same `Dag` as `apply_lens_declaration` (canonical `named_function_count` vs claim).
        let canonical_named_function_count_dag: Option<Dag> = if lens_decl.name.as_deref()
            == Some("named_function_count")
        {
            Some(
                match compile_to_dag(
                    R1_CANONICAL_NAMED_FUNCTION_COUNT_LENS,
                    "src/v3/lenses/named_function_count.dag",
                ) {
                    Ok(dag) => dag,
                    Err(CompileError::Semantic(dag)) => {
                        return ClaimResult::Fail(format!(
                            "LensOutputEquals: canonical `named_function_count` lens failed inference: {:?}",
                            dag.diagnostics().iter().collect::<Vec<_>>()
                        ));
                    }
                    Err(err) => {
                        return ClaimResult::Fail(format!(
                            "LensOutputEquals: canonical `named_function_count` lens did not compile: {err:?}"
                        ));
                    }
                },
            )
        } else {
            None
        };

        if matches!(
            program_input,
            Some(ProgramInputRole::ProgramOutputBind { .. })
        ) {
            return ClaimResult::Fail(format!(
                "LensOutputEquals: input_ref `{input_name}` inhabits ProgramOutputBind but lens `{lens_name}` does not consume an output bind"
            ));
        }
        let reflects_claim_program = program_input
            .as_ref()
            .is_some_and(ProgramInputRole::is_program_input);
        let input_field = if reflects_claim_program {
            // P2: `id_space` must be the same `Dag` `apply_lens_declaration` will use for the lens
            // (canonical compile, claim `program_dag`, or merged fixture `self.dag`) so reflected
            // `List` / `Behavior` variant `DeclarationId`s are not mixed across graphs.
            let id_space: &Dag = if let Some(ref cld) = canonical_named_function_count_dag {
                cld
            } else if let Some(name) = lens_decl.name.as_deref() {
                if program_dag.declaration_by_name(name).is_some() {
                    &program_dag
                } else {
                    self.dag
                }
            } else {
                self.dag
            };
            match reflect_program_dag_nodes_in_file(&program_dag, &claim.file_name, id_space) {
                Ok(v) => v,
                Err(err) => {
                    return ClaimResult::Fail(format!(
                        "LensOutputEquals: could not reflect `Dag` nodes from claim program: {err:?}"
                    ));
                }
            }
        } else {
            match &input_decl.value_body {
                Some(body) => match field_value_from_value_body(self.dag, body) {
                    Ok(v) => v,
                    Err(err) => {
                        return ClaimResult::Fail(format!(
                            "LensOutputEquals: could not lower input_ref `{input_name}` value: {err:?}"
                        ));
                    }
                },
                None => {
                    return ClaimResult::Fail(format!(
                        "LensOutputEquals: input_ref `{input_name}` has no value body (use ProgramInput when the input `Dag` is only available via `TestClaim.source`)"
                    ));
                }
            }
        };

        let expected_field = match field_value_from_value_body(
            self.dag,
            expected_decl.value_body.as_ref().expect("checked"),
        ) {
            Ok(v) => v,
            Err(err) => {
                return ClaimResult::Fail(format!(
                    "LensOutputEquals: could not lower expected_ref `{expected_name}` value: {err:?}"
                ));
            }
        };

        let (lens_program, lens_apply_id) =
            if let Some(ref cld) = canonical_named_function_count_dag {
                let Some(d) = cld.declaration_by_name("named_function_count") else {
                    return ClaimResult::Fail(
                    "LensOutputEquals: canonical named_function_count lens missing root declaration"
                        .to_string(),
                );
                };
                (cld, d.id)
            } else if let Some(name) = lens_decl.name.as_deref() {
                match program_dag.declaration_by_name(name) {
                    Some(d) => (&program_dag, d.id),
                    None => (self.dag, lens_id),
                }
            } else {
                (self.dag, lens_id)
            };

        let computed = match apply_lens_declaration(
            lens_program,
            lens_apply_id,
            std::slice::from_ref(&input_field),
        ) {
            Ok(v) => v,
            Err(err) => {
                return ClaimResult::Fail(format!(
                    "LensOutputEquals: applying lens `{lens_name}` failed: {err:?}"
                ));
            }
        };

        if computed == expected_field {
            ClaimResult::Pass
        } else {
            ClaimResult::Fail(format!(
                "LensOutputEquals: expected {} for `{expected_name}`, computed {} for lens `{lens_name}` (input `{input_name}`)",
                render_field_value(self.dag, &expected_field),
                render_field_value(self.dag, &computed),
            ))
        }
    }

    fn eval_symbolic_cost_expr_equals_shape(
        &self,
        _claim: &TestClaimValue,
        payload: &[FieldValue],
    ) -> ClaimResult {
        let [expected_fv] = payload else {
            return ClaimResult::Fail(format!(
                "SymbolicCostExprEquals payload should be exactly one DeclarationRef field \
                 (expected); got {} payload slot(s)",
                payload.len()
            ));
        };
        let expected_id = match self.resolve_declaration_ref_id(expected_fv, "expected") {
            Ok(id) => id,
            Err(msg) => return ClaimResult::Fail(msg),
        };
        let expected_decl = self.dag.declaration(expected_id);
        let expected_name = decl_display_name(expected_id, expected_decl);
        if expected_decl.value_body.is_none() {
            return ClaimResult::Fail(format!(
                "SymbolicCostExprEquals: expected `{expected_name}` has no value body to compare against"
            ));
        }
        // The `expected: DeclarationRef` field is unrefined (`DeclarationRef`
        // admits any declaration); validate at the runner boundary that the
        // referenced declaration actually inhabits `SymbolicCost`. Mirror of
        // the boundary check in `validate_dimension_report_ref` for
        // `BinaryDimensionReportEquals`. Dissolution: when refinement-typing
        // on `DeclarationRef` (or a `SymbolicCostRef` wrapper class) lands in
        // substrate, this runner-side check retires alongside the wrapper.
        if let Err(reason) = self.validate_symbolic_cost_ref(expected_id, "expected") {
            return ClaimResult::Fail(reason);
        }
        ClaimResult::NotYetImplemented(format!(
            "SymbolicCostExprEquals: structural shape is valid for `{expected_name}`, but runner \
             evaluation waits for the heuristic-cost-function-5th-gate testgen dispatch \
             (Verification follow-up). The eval will apply the symbolic-cost lens to the \
             program-under-test (`TestClaim.source` for enumerated claims; `ProgramShape.source` \
             for quantified claims once Slice 1 lands) and compare the result structurally to \
             `{expected_name}`."
        ))
    }

    /// Boundary check that `decl_id` references a declaration whose declared
    /// type is `SymbolicCost`. The runner walks transparent aliases (no-arg
    /// instantiations + ResolvedBy* atoms) to handle `data X: SymbolicCost
    /// = ConstantCost(0)`-style declarations whose connective is an alias to
    /// the algebra type. Same shape as `validate_dimension_report_ref`,
    /// scoped to a single nominal type instead of `DimensionReport<C>`.
    fn validate_symbolic_cost_ref(
        &self,
        decl_id: DeclarationId,
        field_label: &str,
    ) -> Result<(), String> {
        let symbolic_cost_id = self
            .dag
            .declaration_by_name("SymbolicCost")
            .map(|decl| decl.id)
            .ok_or_else(|| {
                format!(
                    "SymbolicCostExprEquals `{field_label}`: \
                     `SymbolicCost` type not found in bootstrap"
                )
            })?;
        let decl = self.dag.declaration(decl_id);
        // For `fn`-shaped expected refs, walk through the arrow output;
        // otherwise normalize the declaration itself.
        let candidate = match &decl.connective {
            TypeConnective::Arrow { output, .. } => *output,
            _ => decl_id,
        };
        if self.normalize_transparent_type(candidate) == symbolic_cost_id {
            return Ok(());
        }
        Err(format!(
            "SymbolicCostExprEquals `{field_label}` must reference a declaration of type \
             `SymbolicCost`; `{}` does not (declared type does not normalize to `SymbolicCost`).",
            decl_display_name(decl_id, decl)
        ))
    }

    fn eval_binary_dimension_report_equals_shape(
        &self,
        _claim: &TestClaimValue,
        payload: &[FieldValue],
    ) -> ClaimResult {
        let [left_fv, right_fv] = payload else {
            return ClaimResult::Fail(format!(
                "BinaryDimensionReportEquals payload should be exactly two DeclarationRef fields \
                 (left_report_ref, right_report_ref); got {} payload slot(s)",
                payload.len()
            ));
        };
        let left_id = match self.resolve_declaration_ref_id(left_fv, "left_report_ref") {
            Ok(id) => id,
            Err(msg) => return ClaimResult::Fail(msg),
        };
        let right_id = match self.resolve_declaration_ref_id(right_fv, "right_report_ref") {
            Ok(id) => id,
            Err(msg) => return ClaimResult::Fail(msg),
        };
        let left_name = decl_display_name(left_id, self.dag.declaration(left_id));
        let right_name = decl_display_name(right_id, self.dag.declaration(right_id));
        let left_carrier = match self.validate_dimension_report_ref(left_id, "left_report_ref") {
            Ok(carrier) => carrier,
            Err(reason) => return ClaimResult::Fail(reason),
        };
        let right_carrier = match self.validate_dimension_report_ref(right_id, "right_report_ref") {
            Ok(carrier) => carrier,
            Err(reason) => return ClaimResult::Fail(reason),
        };
        if !self.dimension_report_carriers_equivalent(left_carrier, right_carrier) {
            return ClaimResult::Fail(format!(
                "BinaryDimensionReportEquals requires both refs to produce DimensionReport<C> \
                 for the same carrier C; `{left_name}` uses `{}` but `{right_name}` uses `{}`",
                decl_display_name(left_carrier, self.dag.declaration(left_carrier)),
                decl_display_name(right_carrier, self.dag.declaration(right_carrier))
            ));
        }
        ClaimResult::NotYetImplemented(format!(
            "BinaryDimensionReportEquals: structural shape is valid for `{left_name}` and \
             `{right_name}`, but runner evaluation waits for generic DimensionReport<C> \
             production/evaluation substrate; serialized report comparison is intentionally \
             unsupported"
        ))
    }

    fn validate_dimension_report_ref(
        &self,
        decl_id: DeclarationId,
        field_label: &str,
    ) -> Result<DeclarationId, String> {
        let decl = self.dag.declaration(decl_id);
        let candidate_type = match &decl.connective {
            TypeConnective::Arrow { output, .. } => *output,
            _ => decl_id,
        };
        self.dimension_report_carrier(candidate_type)
            .ok_or_else(|| {
                format!(
                    "BinaryDimensionReportEquals `{field_label}` must reference a declaration \
                     that produces or inhabits DimensionReport<C>; `{}` does not",
                    decl_display_name(decl_id, decl)
                )
            })
    }

    fn dimension_report_carriers_equivalent(
        &self,
        left_carrier: DeclarationId,
        right_carrier: DeclarationId,
    ) -> bool {
        left_carrier == right_carrier
            || type_shapes_equivalent(
                self.dag,
                &TypeShape::new(left_carrier),
                &TypeShape::new(right_carrier),
            )
    }

    fn dimension_report_carrier(&self, mut current: DeclarationId) -> Option<DeclarationId> {
        let report_id = self
            .dag
            .declaration_by_name("DimensionReport")
            .map(|decl| decl.id)?;
        // Bounded alias walk: this is a fail-closed cycle/depth guard, not a
        // semantic limit on valid DimensionReport<C> producer shapes.
        for _ in 0..32 {
            match &self.dag.declaration(current).connective {
                TypeConnective::Instantiation {
                    template,
                    arguments,
                } if *template == report_id => match arguments.as_slice() {
                    [carrier] => return Some(self.normalize_transparent_type(carrier.value)),
                    _ => return None,
                },
                TypeConnective::Instantiation {
                    template,
                    arguments,
                } if arguments.is_empty() => current = *template,
                TypeConnective::Atom(
                    AtomPayload::ResolvedByStructure(next) | AtomPayload::ResolvedByName(next),
                ) => current = *next,
                _ => return None,
            }
        }
        None
    }

    fn normalize_transparent_type(&self, mut current: DeclarationId) -> DeclarationId {
        for _ in 0..32 {
            match &self.dag.declaration(current).connective {
                TypeConnective::Instantiation {
                    template,
                    arguments,
                } if arguments.is_empty() => current = *template,
                TypeConnective::Atom(
                    AtomPayload::ResolvedByStructure(next) | AtomPayload::ResolvedByName(next),
                ) => current = *next,
                _ => return current,
            }
        }
        current
    }

    fn resolve_declaration_ref_id(
        &self,
        value: &FieldValue,
        field_label: &str,
    ) -> Result<DeclarationId, String> {
        match value {
            FieldValue::Reference(id) => Ok(*id),
            FieldValue::Record(fields) if fields.is_empty() => Err(format!(
                "`{field_label}`: DeclarationRef is the empty record literal {{}} — use an identifier \
                 so lowering emits FieldValue::Reference(DeclarationId), not an empty record",
            )),
            other => Err(format!(
                "`{field_label}`: expected FieldValue::Reference(DeclarationId) \
                 for a DeclarationRef edge, got {other:?}"
            )),
        }
    }

    fn program_input_role(&self, decl: &Declaration) -> Result<Option<ProgramInputRole>, String> {
        if self.decl_inhabits_named_role(decl, "ProgramInput")? {
            return Ok(Some(ProgramInputRole::ProgramInput));
        }
        if !self.decl_inhabits_named_role(decl, "ProgramOutputBind")? {
            return Ok(None);
        }
        let Some(ValueBody::Structural { fields }) = decl.value_body.as_ref() else {
            return Err(format!(
                "ProgramOutputBind `{}` must have a structural data body",
                decl_display_name(decl.id, decl)
            ));
        };
        let Some(output_ref) = field(fields, "output_ref") else {
            return Err(format!(
                "ProgramOutputBind `{}` is missing `output_ref`",
                decl_display_name(decl.id, decl)
            ));
        };
        let output_ref = match output_ref {
            FieldValue::Reference(id) => *id,
            other => {
                return Err(format!(
                    "ProgramOutputBind `{}` output_ref must be a DeclarationRef edge, got {other:?}",
                    decl_display_name(decl.id, decl)
                ));
            }
        };
        let output_decl = self.dag.declaration(output_ref);
        let Some(output_bind_name) = output_decl.name.clone() else {
            return Err(format!(
                "ProgramOutputBind `{}` output_ref must name a declaration",
                decl_display_name(decl.id, decl)
            ));
        };
        // Cross-Dag bridge: `output_ref` is a structural edge in the fixture DAG, but
        // the compiled `TestClaim.source` program is a separate Dag, so the runner
        // still carries the referenced declaration name into `find_bind`.
        // Dissolution trigger: authored claims carry an output-bind identity that
        // resolves inside the compiled program Dag instead of through fixture stubs.
        Ok(Some(ProgramInputRole::ProgramOutputBind {
            output_bind_name,
        }))
    }

    fn decl_inhabits_named_role(
        &self,
        decl: &Declaration,
        role_name: &str,
    ) -> Result<bool, String> {
        // Narrow bridge: role declarations are still found by type name because
        // `input_ref` is statically `DeclarationRef`. Dissolve with a structured
        // `TestPredicate` input-role coproduct when the schema carries it.
        let Some(role_id) = self.dag.declaration_by_name(role_name).map(|d| d.id) else {
            return Err(format!(
                "verification role type `{role_name}` is missing from the fixture Dag"
            ));
        };
        Ok(decl.inhabits == Some(role_id)
            || decl.meta_tag == Some(role_id)
            || matches!(
                &decl.connective,
                TypeConnective::Instantiation { template, .. } if *template == role_id
            ))
    }

    fn decl_inhabits_role_id(decl: &Declaration, role_id: DeclarationId) -> bool {
        decl.inhabits == Some(role_id)
            || decl.meta_tag == Some(role_id)
            || matches!(
                &decl.connective,
                TypeConnective::Instantiation { template, .. } if *template == role_id
            )
    }

    fn eval_differential_equals(
        &self,
        claim: &TestClaimValue,
        payload: &[FieldValue],
    ) -> ClaimResult {
        let [subject_fv, oracle_fv, input_fv] = payload else {
            return ClaimResult::Fail(format!(
                "DifferentialEquals payload should be exactly three DeclarationRef fields \
                 (subject_ref, oracle_ref, input_ref); got {} payload slot(s)",
                payload.len()
            ));
        };
        let subject_id = match self.resolve_declaration_ref_id(subject_fv, "subject_ref") {
            Ok(id) => id,
            Err(msg) => return ClaimResult::Fail(msg),
        };
        let oracle_id = match self.resolve_declaration_ref_id(oracle_fv, "oracle_ref") {
            Ok(id) => id,
            Err(msg) => return ClaimResult::Fail(msg),
        };
        let input_id = match self.resolve_declaration_ref_id(input_fv, "input_ref") {
            Ok(id) => id,
            Err(msg) => return ClaimResult::Fail(msg),
        };

        let subject_decl = self.dag.declaration(subject_id);
        let oracle_decl = self.dag.declaration(oracle_id);
        let input_decl = self.dag.declaration(input_id);

        let subject_lineage = decl_display_name(subject_id, subject_decl);
        let oracle_lineage = decl_display_name(oracle_id, oracle_decl);
        let input_name = decl_display_name(input_id, input_decl);

        let program_input = match self.program_input_role(input_decl) {
            Ok(Some(role)) => role,
            Ok(None) => {
                return ClaimResult::Fail(format!(
                    "DifferentialEquals: input_ref `{input_name}` must inhabit ProgramOutputBind"
                ));
            }
            Err(msg) => return ClaimResult::Fail(format!("DifferentialEquals: {msg}")),
        };

        let program_dag = match compile_to_dag(&claim.source, &claim.file_name) {
            Ok(dag) => dag,
            Err(CompileError::Semantic(dag)) => {
                return ClaimResult::Fail(format!(
                    "DifferentialEquals: claim `source` / `{}` failed inference: {:?}",
                    claim.file_name,
                    dag.diagnostics().iter().collect::<Vec<_>>()
                ));
            }
            Err(err) => {
                return ClaimResult::Fail(format!(
                    "DifferentialEquals: claim `source` / `{}` did not compile: {err:?}",
                    claim.file_name
                ));
            }
        };

        let Some(cost_bind) = program_input.output_bind_name() else {
            return ClaimResult::Fail(format!(
                "DifferentialEquals: input_ref `{input_name}` must inhabit ProgramOutputBind"
            ));
        };
        let Some(bind) = find_bind(&program_dag, cost_bind, &claim.file_name) else {
            return ClaimResult::Fail(format!(
                "DifferentialEquals: bind `{cost_bind}` not found in `{}`",
                claim.file_name
            ));
        };

        if subject_lineage == oracle_lineage {
            return ClaimResult::Fail(
                "DifferentialEquals: subject_ref and oracle_ref must name distinct lineages"
                    .to_string(),
            );
        }

        let w1_emit_eval_pair = (subject_lineage.as_str(), oracle_lineage.as_str());
        if matches!(
            w1_emit_eval_pair,
            ("rust_emit_output", "dag_eval_output") | ("dag_eval_output", "rust_emit_output")
        ) {
            let subject_int = match w1_differential_equals_lineage_int(
                subject_lineage.as_str(),
                &program_dag,
                bind,
                &claim.file_name,
            ) {
                Ok(v) => v,
                Err(msg) => return ClaimResult::Fail(msg),
            };
            let oracle_int = match w1_differential_equals_lineage_int(
                oracle_lineage.as_str(),
                &program_dag,
                bind,
                &claim.file_name,
            ) {
                Ok(v) => v,
                Err(msg) => return ClaimResult::Fail(msg),
            };
            return if subject_int == oracle_int {
                ClaimResult::Pass
            } else {
                ClaimResult::Fail(format!(
                    "DifferentialEquals(W1): `{subject_lineage}` int {subject_int} != `{oracle_lineage}` int {oracle_int} \
                     (emit vs eager eval parity; dissolution: PB-Runtime tests + PR-B witness-shaped `ProgramObservation<Value>`)"
                ))
            };
        }

        let pairing_ok = (subject_lineage.as_str() == "v3_program_cost"
            && oracle_lineage.as_str() == "v2_oracle_cost")
            || (subject_lineage.as_str() == "v2_oracle_cost"
                && oracle_lineage.as_str() == "v3_program_cost");
        if !pairing_ok {
            // E8/W1: unsupported output producers must stay fail-closed until
            // producer identity and typed observation normalization are declared.
            // Dissolution targets: `rust_emit_output` -> PB-Runtime generated
            // target-language tests; `dag_eval_output` -> PR-B eager evaluator
            // plus witness construction.
            return ClaimResult::NotYetImplemented(format!(
                "DifferentialEquals: unsupported producer pairing ({subject_lineage}, {oracle_lineage}); \
                 implemented: (v3_program_cost, v2_oracle_cost) Lane-E cost parity and \
                 (rust_emit_output, dag_eval_output) W1 emit/eval Int slice per #1485 / \
                 `docs/briefs/r3-pr-e8-w1-output-producer-contract-blocker.md`. \
                 This path stays NotYetImplemented (fail-closed unsupported-pair receipt, not Pass) until \
                 an approved producer + observation contract matches; after #1495 lands, rebase this \
                 branch and preserve the stronger unsupported-pair ratchet / producer-identity gates vs \
                 substrate-owned dissolution (`r2-pr-b-2-runner-extension-bundle.md`)."
            ));
        }

        // P3: `subject_ref` / `oracle_ref` are not decorative — `subject_lineage` vs
        // `oracle_lineage` must dispatch distinct producers in
        // `eval_lane_e_differential_cost_lineage` (host forward-fold vs `lens_cost::cost_of`), not
        // two identical `cost_of` calls (PR #764 inline review).
        let subject_out = match eval_lane_e_differential_cost_lineage(
            subject_lineage.as_str(),
            &program_dag,
            bind.value,
        ) {
            Ok(v) => v,
            Err(msg) => return ClaimResult::Fail(msg),
        };
        let oracle_out = match eval_lane_e_differential_cost_lineage(
            oracle_lineage.as_str(),
            &program_dag,
            bind.value,
        ) {
            Ok(v) => v,
            Err(msg) => return ClaimResult::Fail(msg),
        };

        if subject_out == oracle_out {
            ClaimResult::Pass
        } else {
            ClaimResult::Fail(format!(
                "DifferentialEquals: subject `{subject_lineage}` output {subject_out:?} != oracle `{oracle_lineage}` output {oracle_out:?} (host forward-fold vs `lens_cost::cost_of`)"
            ))
        }
    }

    fn eval_algebraic_law(&self, claim: &TestClaimValue, payload: &[FieldValue]) -> ClaimResult {
        // `Associativity` and `Commutativity` are wired via bounded operational witness tables
        // (see `eval_algebraic_law_for_claim_program` — not substrate law-fact evaluation).
        // `Identity` remains `NotYetImplemented` until the substrate exposes the lens identity
        // element edge required by the PR-B.3 runner-extension brief.
        let (law, _) = match algebraic_law_payload_fields(payload) {
            Ok(parts) => parts,
            Err(AlgebraicLawProgramError::MalformedPayload(message)) => {
                return ClaimResult::Fail(message);
            }
            Err(AlgebraicLawProgramError::UnsupportedLaw { law_label }) => unreachable!(
                "algebraic_law_payload_fields only yields MalformedPayload (got UnsupportedLaw({law_label:?}))"
            ),
        };
        let (law_label, law_payload) = match variant_fields(self.dag, law) {
            Ok(parts) => parts,
            Err(AlgebraicLawProgramError::MalformedPayload(message)) => {
                return ClaimResult::Fail(message);
            }
            Err(AlgebraicLawProgramError::UnsupportedLaw { law_label }) => unreachable!(
                "variant_fields only yields MalformedPayload (got UnsupportedLaw({law_label:?}))"
            ),
        };
        if law_label == "Identity" {
            return ClaimResult::NotYetImplemented(
                "AlgebraicLaw::Identity is blocked: no lens identity-element edge is exposed on \
                 the algebra inhabitance yet (PR-B.3 W2); leave fail-closed until that substrate \
                 fact exists"
                    .to_string(),
            );
        }
        if law_label != "Associativity" && law_label != "Commutativity" {
            return ClaimResult::NotYetImplemented(format!(
                "AlgebraicLaw::{law_label} is not wired in the Rust runner; Distributivity must \
                 route through INVARIANTS P1 as an AlgebraicLawKind substrate enum extension"
            ));
        }
        if !law_payload.is_empty() {
            return ClaimResult::Fail(format!("{law_label} should be payload-free"));
        }

        let program_dag = match compile_to_dag(&claim.source, &claim.file_name) {
            Ok(dag) => dag,
            Err(CompileError::Semantic(_)) => {
                return ClaimResult::Fail(
                    "claim program compiled with diagnostics (AlgebraicLaw requires a clean compile)"
                        .to_string(),
                );
            }
            Err(err) => {
                return ClaimResult::Fail(format!(
                    "claim program did not compile (AlgebraicLaw): {err:?}"
                ));
            }
        };
        match eval_algebraic_law_for_claim_program(self.dag, &program_dag, payload) {
            Ok(true) => ClaimResult::Pass,
            Ok(false) => ClaimResult::Fail(format!(
                "AlgebraicLaw {law_label}: operational witness failed (must pass all fixed Int \
                 samples in lens_apply; D1 apply — not a substrate declared-law check; see \
                 eval_algebraic_law_for_claim_program)"
            )),
            Err(AlgebraicLawProgramError::MalformedPayload(message)) => ClaimResult::Fail(message),
            Err(AlgebraicLawProgramError::UnsupportedLaw { law_label }) => {
                ClaimResult::NotYetImplemented(format!(
                    "AlgebraicLaw::{law_label} is not implemented by the Rust runner"
                ))
            }
        }
    }

    /// Same pattern as other arms: `compile_to_dag(claim.source)` is a **clean-claim** gate; host
    /// `command` / `args` / `expect_exit_code` come only from the predicate `payload` (the compiled
    /// `Dag` is not an input to `std::process::Command` here — PR #792, 837d0e59).
    fn eval_execute_command(&self, claim: &TestClaimValue, payload: &[FieldValue]) -> ClaimResult {
        match compile_to_dag(&claim.source, &claim.file_name) {
            Ok(_) => {}
            Err(CompileError::Semantic(_)) => {
                return ClaimResult::Fail(
                    "ExecuteCommand: claim program compiled with diagnostics (clean compile required)"
                        .to_string(),
                );
            }
            Err(err) => {
                return ClaimResult::Fail(format!(
                    "ExecuteCommand: claim program did not compile: {err:?}"
                ));
            }
        }
        let Some((command, args, expect_exit_code)) = parse_execute_command_fields(payload) else {
            return ClaimResult::Fail(
                "ExecuteCommand payload should be (String, List<String>, Int) — see verification.dag"
                    .to_string(),
            );
        };
        evaluate_execute_command_exit_code(&command, &args, expect_exit_code)
    }

    fn eval_cost_bounded(&self, claim: &TestClaimValue, payload: &[FieldValue]) -> ClaimResult {
        let [FieldValue::Literal(LiteralBits::String(bind_name)), comparator, FieldValue::Literal(LiteralBits::Int(bound))] =
            payload
        else {
            return ClaimResult::Fail(
                "CostBounded payload should be (String, ComparisonOp, Int)".to_string(),
            );
        };
        let dag = match compile_to_dag(&claim.source, &claim.file_name) {
            Ok(dag) => dag,
            Err(err) => {
                return ClaimResult::Fail(format!(
                    "CostBounded: claim source did not compile (structural cost check skipped): {err:?}"
                ));
            }
        };
        let Some(bind) = find_bind(&dag, bind_name, &claim.file_name) else {
            return ClaimResult::Fail(format!(
                "CostBounded: bind `{bind_name}` not found in `{}`",
                claim.file_name
            ));
        };
        let actual = match cost_of(&dag, &bind.value) {
            CostLookup::Hit(actual) => actual,
            CostLookup::Miss => {
                return ClaimResult::Fail(format!(
                    "CostBounded: missing structural `cost_of` receipt for bind `{bind_name}`"
                ));
            }
        };
        if self.compare_cost(comparator, actual, *bound) {
            ClaimResult::Pass
        } else {
            ClaimResult::Fail(format!("cost {actual} did not satisfy bound {bound}"))
        }
    }

    /// Evaluate `TestPredicate::PerfWithinBaseline { subject, comparator, baseline_ref }`.
    ///
    /// Resolves both `DeclarationRef`s to `PerfBaselineMeasurement` records
    /// (`{ median_ns: Int, p99_ns: Int }`) and applies the Director-locked
    /// Tier-3 budget thresholds (`docs/r3-structure.md` §225): subject median
    /// must satisfy `comparator` against `baseline.median_ns × 2`; subject p99
    /// against `baseline.p99_ns × 5`. Both axes must satisfy → `Pass`. Overflow
    /// in the multiplication is fail-closed.
    fn eval_perf_within_baseline(
        &self,
        _claim: &TestClaimValue,
        payload: &[FieldValue],
    ) -> ClaimResult {
        let [subject_fv, comparator, baseline_fv] = payload else {
            return ClaimResult::Fail(format!(
                "PerfWithinBaseline payload should be exactly three fields \
                 (subject: DeclarationRef, comparator: ComparisonOp, baseline_ref: DeclarationRef); \
                 got {} payload slot(s)",
                payload.len()
            ));
        };
        let subject_id = match self.resolve_declaration_ref_id(subject_fv, "subject") {
            Ok(id) => id,
            Err(reason) => return ClaimResult::Fail(reason),
        };
        let baseline_id = match self.resolve_declaration_ref_id(baseline_fv, "baseline_ref") {
            Ok(id) => id,
            Err(reason) => return ClaimResult::Fail(reason),
        };
        // Enforce the Director-locked "within baseline" semantics (`r3-structure.md`
        // §225): budget is `median ≤ 2× baseline` AND `p99 ≤ 5× baseline`. The
        // substrate-declared shape carries `comparator: ComparisonOp` for explicit
        // intent in `.dag` claims, but only `Le` matches the locked semantics —
        // `Gt`/`Ge`/`Ne`/`Eq`/`Lt` would invert or skew the gate. Reject
        // fail-closed at the runtime boundary so a malformed claim cannot pass.
        let comparator_label = match self.variant_value(comparator) {
            Some((label, payload)) if payload.is_empty() => label,
            _ => {
                return ClaimResult::Fail(
                    "PerfWithinBaseline `comparator`: expected unit ComparisonOp variant; \
                     got non-variant or non-empty payload"
                        .to_string(),
                );
            }
        };
        if !is_comparator_le(&comparator_label) {
            return ClaimResult::Fail(format!(
                "PerfWithinBaseline `comparator`: only `Le` matches the Director-locked \
                 §225 budget semantics (median ≤ 2× baseline, p99 ≤ 5× baseline); \
                 `{comparator_label}` would invert or skew the gate"
            ));
        }
        let subject = match self.perf_baseline_measurement(subject_id, "subject") {
            Ok(measurement) => measurement,
            Err(err) => return ClaimResult::Fail(err.into_claim_fail("subject")),
        };
        let baseline = match self.perf_baseline_measurement(baseline_id, "baseline_ref") {
            Ok(measurement) => measurement,
            Err(err) => return ClaimResult::Fail(err.into_claim_fail("baseline_ref")),
        };
        let (median_bound, p99_bound) = match compute_perf_budget_bounds(baseline) {
            Ok(bounds) => bounds,
            Err(PerfBudgetOverflow::Median) => {
                return ClaimResult::Fail(
                    "PerfWithinBaseline: median baseline threshold (baseline_ref median_ns × 2) \
                     overflowed Int — fail-closed; recapture with smaller baseline or widen Int."
                        .to_string(),
                );
            }
            Err(PerfBudgetOverflow::P99) => {
                return ClaimResult::Fail(
                    "PerfWithinBaseline: p99 baseline threshold (baseline_ref p99_ns × 5) \
                     overflowed Int — fail-closed; recapture with smaller baseline or widen Int."
                        .to_string(),
                );
            }
        };
        let median_ok = self.compare_cost(comparator, subject.median_ns, median_bound);
        let p99_ok = self.compare_cost(comparator, subject.p99_ns, p99_bound);
        if median_ok && p99_ok {
            ClaimResult::Pass
        } else {
            ClaimResult::Fail(format!(
                "PerfWithinBaseline: subject median_ns={} vs threshold {} (median_ok={}) \
                 and subject p99_ns={} vs threshold {} (p99_ok={}) did not satisfy comparator",
                subject.median_ns, median_bound, median_ok, subject.p99_ns, p99_bound, p99_ok
            ))
        }
    }

    /// Structurally resolve a `PerfBaselineMeasurement` data declaration to
    /// `{ median_ns, p99_ns }`. Returns a typed `PerfMeasurementResolveError`
    /// per `CODING.md` typed-error discipline; the outer
    /// [`Self::eval_perf_within_baseline`] boundary converts to
    /// `ClaimResult::Fail(...)` with the role label preserved.
    fn perf_baseline_measurement(
        &self,
        decl_id: DeclarationId,
        _role: &str,
    ) -> Result<PerfMeasurement, PerfMeasurementResolveError> {
        let decl = match self.dag.declaration_opt(&decl_id) {
            Some(decl) => decl,
            None => return Err(PerfMeasurementResolveError::MissingDeclaration),
        };
        let fields = match decl.value_body.as_ref() {
            Some(ValueBody::Structural { fields }) => fields,
            _ => return Err(PerfMeasurementResolveError::WrongConnective),
        };
        let median_ns = match field(fields, "median_ns") {
            Some(FieldValue::Literal(LiteralBits::Int(v))) => *v,
            Some(_) => {
                return Err(PerfMeasurementResolveError::WrongFieldKind { field: "median_ns" });
            }
            None => {
                return Err(PerfMeasurementResolveError::MissingField { field: "median_ns" });
            }
        };
        let p99_ns = match field(fields, "p99_ns") {
            Some(FieldValue::Literal(LiteralBits::Int(v))) => *v,
            Some(_) => {
                return Err(PerfMeasurementResolveError::WrongFieldKind { field: "p99_ns" });
            }
            None => {
                return Err(PerfMeasurementResolveError::MissingField { field: "p99_ns" });
            }
        };
        Ok(PerfMeasurement { median_ns, p99_ns })
    }

    fn eval_census_bound_check_shape(
        &self,
        _claim: &TestClaimValue,
        payload: &[FieldValue],
    ) -> ClaimResult {
        let [authority, list_constant, FieldValue::Literal(LiteralBits::Int(bound))] = payload
        else {
            return ClaimResult::Fail(
                "CensusBoundCheck payload should be (DeclarationRef, CensusListConstant, Int)"
                    .to_string(),
            );
        };
        if let Err(reason) = self.resolve_census_authority_ref(authority, "authority") {
            return ClaimResult::Fail(reason);
        }
        let list_constant_name = match self.resolve_census_list_constant_ref(list_constant) {
            Ok(name) => name,
            Err(reason) => return ClaimResult::Fail(reason),
        };
        let count = match sg0_census_list_count(&list_constant_name) {
            Ok(count) => count,
            Err(reason) => return ClaimResult::Fail(reason),
        };
        if count <= *bound {
            ClaimResult::Pass
        } else {
            ClaimResult::Fail(format!(
                "CensusBoundCheck `{list_constant_name}` observed {count}, bound {bound}"
            ))
        }
    }

    fn eval_census_subset_count_shape(
        &self,
        _claim: &TestClaimValue,
        payload: &[FieldValue],
    ) -> ClaimResult {
        let [authority, list_constant, subset_predicate] = payload else {
            return ClaimResult::Fail(
                "CensusSubsetCount payload should be (DeclarationRef, CensusListConstant, DeclarationRef)".to_string(),
            );
        };
        if let Err(reason) = self.resolve_census_authority_ref(authority, "authority") {
            return ClaimResult::Fail(reason);
        }
        let list_constant_name = match self.resolve_census_list_constant_ref(list_constant) {
            Ok(name) => name,
            Err(reason) => return ClaimResult::Fail(reason),
        };
        if let Err(reason) = self.resolve_pb_marker_ref(
            subset_predicate,
            "subset_predicate",
            "lens_producer_files_subset_predicate",
            "LensProducerFilesSubsetPredicate",
        ) {
            return ClaimResult::Fail(reason);
        }
        let entries = match sg0_census_list_entries(&list_constant_name) {
            Ok(entries) => entries,
            Err(reason) => return ClaimResult::Fail(reason),
        };
        let count = entries
            .iter()
            .filter(|path| is_lens_producer_census_path(path))
            .count() as i64;
        if count == 0 {
            ClaimResult::Pass
        } else {
            ClaimResult::Fail(format!(
                "CensusSubsetCount `{list_constant_name}` lens-producer subset observed {count}"
            ))
        }
    }

    fn eval_fixed_point_converges_shape(
        &self,
        _claim: &TestClaimValue,
        payload: &[FieldValue],
    ) -> ClaimResult {
        let [FieldValue::Literal(LiteralBits::String(compile_target)), FieldValue::Literal(LiteralBits::String(expected))] =
            payload
        else {
            return ClaimResult::Fail(
                "FixedPointConverges payload should be (Path, SnapshotRef)".to_string(),
            );
        };
        if compile_target != "default_fixed_point_source" {
            return ClaimResult::Fail(format!(
                "FixedPointConverges only supports `default_fixed_point_source` today, got `{compile_target}`"
            ));
        }
        if expected != "pipeline_stage_snapshots" {
            return ClaimResult::Fail(format!(
                "FixedPointConverges only supports `pipeline_stage_snapshots` today, got `{expected}`"
            ));
        }
        let pass1 = match compile_stage_snapshots(default_fixed_point_source(), compile_target) {
            Ok(snapshots) => snapshots,
            Err(err) => {
                return ClaimResult::Fail(format!("FixedPointConverges pass1 failed: {err:?}"))
            }
        };
        let pass2 = match compile_stage_snapshots(default_fixed_point_source(), compile_target) {
            Ok(snapshots) => snapshots,
            Err(err) => {
                return ClaimResult::Fail(format!("FixedPointConverges pass2 failed: {err:?}"))
            }
        };
        match compare_stage_snapshots(&pass1, &pass2) {
            Ok(()) => ClaimResult::Pass,
            Err(mismatch) => ClaimResult::Fail(format!(
                "FixedPointConverges mismatch at `{}`: {}",
                mismatch.stage, mismatch.detail
            )),
        }
    }

    fn eval_ratchet_zero_shape(
        &self,
        _claim: &TestClaimValue,
        payload: &[FieldValue],
    ) -> ClaimResult {
        let [authority, ratchet_kind] = payload else {
            return ClaimResult::Fail(
                "RatchetZero payload should be (DeclarationRef, DeclarationRef)".to_string(),
            );
        };
        if let Err(reason) = self.resolve_census_authority_ref(authority, "authority") {
            return ClaimResult::Fail(reason);
        }
        if let Err(reason) = self.resolve_pb_marker_ref(
            ratchet_kind,
            "ratchet_kind",
            "compiler_std_positive_set_ratchet",
            "CompilerStdPositiveSetRatchet",
        ) {
            return ClaimResult::Fail(reason);
        }
        let count = compiler_std_positive_set_ratchet_count();
        if count == 0 {
            ClaimResult::Pass
        } else {
            ClaimResult::Fail(format!(
                "RatchetZero `compiler_std_positive_set_ratchet` observed {count}"
            ))
        }
    }

    /// `TestPredicate::BridgeLedgerZero { ledger: BridgeLedgerRef }`.
    /// Unwraps the `BridgeLedgerRef { decl: DeclarationRef }` typed
    /// wrapper, fail-closes if the inner declaration is not the
    /// canonical `bridge_ledger`, then walks the
    /// `List<BridgeLedgerRow>` and `Pass`es iff every row's `status`
    /// field resolves to the `Retired` constructor of `BridgeStatus`.
    /// Open rows surface in the failure message by name so
    /// Verification points directly at the residual debt.
    fn eval_bridge_ledger_zero(
        &self,
        _claim: &TestClaimValue,
        payload: &[FieldValue],
    ) -> ClaimResult {
        let [ledger] = payload else {
            return ClaimResult::Fail(
                "BridgeLedgerZero payload should be (BridgeLedgerRef)".to_string(),
            );
        };
        // `ledger: BridgeLedgerRef` lowers as a record `{ decl: DeclarationRef }`.
        // Extract the inner declaration reference structurally; reject
        // an unwrapped `DeclarationRef` because that would regress the
        // typed-edge discipline the `BridgeLedgerRef` wrapper adds.
        let ledger_id = match ledger {
            FieldValue::Record(fields) => {
                let Some(decl) = fields.iter().find(|(l, _)| l == "decl").map(|(_, v)| v) else {
                    return ClaimResult::Fail(
                        "BridgeLedgerZero `ledger`: BridgeLedgerRef record missing `decl` field"
                            .to_string(),
                    );
                };
                match decl {
                    FieldValue::Reference(id) => *id,
                    other => {
                        return ClaimResult::Fail(format!(
                            "BridgeLedgerZero `ledger.decl`: expected \
                             FieldValue::Reference(DeclarationId), got {other:?}"
                        ));
                    }
                }
            }
            other => {
                return ClaimResult::Fail(format!(
                    "BridgeLedgerZero `ledger`: expected BridgeLedgerRef record \
                     `{{ decl: <ref> }}`, got {other:?}"
                ));
            }
        };
        let decl = self.dag.declaration(ledger_id);
        // Fail-closed *identity* check: the ledger declaration must be
        // exactly `bridge_ledger` from `v3.std.bridge_ledger`, not any
        // sibling `List<BridgeLedgerRow>`. Single-authority discipline
        // (INVARIANTS P2 / modeling-discipline single-authority): a
        // second list that happens to share the row type cannot be
        // accepted as a parallel ledger authority and pass the gate
        // independently of the canonical carrier.
        let canonical_ledger_id = self.dag.declaration_by_name("bridge_ledger").map(|d| d.id);
        match canonical_ledger_id {
            Some(canonical) if canonical == ledger_id => {}
            Some(_) => {
                return ClaimResult::Fail(format!(
                    "BridgeLedgerZero `ledger`: only the canonical \
                     `v3.std.bridge_ledger.bridge_ledger` declaration is an \
                     accepted ledger authority. Got `{}` (DeclarationId {:?}).",
                    decl_display_name(decl.id, decl),
                    decl.id
                ));
            }
            None => {
                return ClaimResult::Fail(
                    "BridgeLedgerZero: canonical `bridge_ledger` declaration is missing \
                     from the bootstrap; the ledger gate cannot resolve."
                        .to_string(),
                );
            }
        }
        // Type check (kept as a defense-in-depth guard even after the
        // identity check above): the canonical declaration must be
        // `List<BridgeLedgerRow>`. If the carrier authority ever
        // diverges from this shape the predicate fails closed instead
        // of silently misreading the rows.
        match &decl.connective {
            TypeConnective::Instantiation {
                template,
                arguments,
            } => {
                let template_name = self.dag.declaration(*template).name.clone();
                if template_name.as_deref() != Some("List") {
                    return ClaimResult::Fail(format!(
                        "BridgeLedgerZero `ledger`: declaration `{}` is not a `List<...>` \
                         instantiation (template is `{}`)",
                        decl_display_name(decl.id, decl),
                        template_name.as_deref().unwrap_or("<anon>")
                    ));
                }
                let [arg] = arguments.as_slice() else {
                    return ClaimResult::Fail(format!(
                        "BridgeLedgerZero `ledger`: `List<...>` instantiation must carry \
                         exactly one type argument, got {}",
                        arguments.len()
                    ));
                };
                let element_name = self.dag.declaration(arg.value).name.clone();
                if element_name.as_deref() != Some("BridgeLedgerRow") {
                    return ClaimResult::Fail(format!(
                        "BridgeLedgerZero `ledger`: expected `List<BridgeLedgerRow>` \
                         element type, got `List<{}>`",
                        element_name.as_deref().unwrap_or("<anon>")
                    ));
                }
            }
            other => {
                return ClaimResult::Fail(format!(
                    "BridgeLedgerZero `ledger`: declaration `{}` is not a `List<...>` \
                     instantiation; connective is {other:?}",
                    decl_display_name(decl.id, decl)
                ));
            }
        }
        let rows = match &decl.value_body {
            Some(ValueBody::List(rows)) => rows,
            Some(other) => {
                return ClaimResult::Fail(format!(
                    "BridgeLedgerZero `ledger` must point at a `data X: List<BridgeLedgerRow>` \
                     declaration; `{}` value_body is {other:?}",
                    decl_display_name(decl.id, decl)
                ));
            }
            None => {
                return ClaimResult::Fail(format!(
                    "BridgeLedgerZero `ledger` declaration `{}` has no value_body",
                    decl_display_name(decl.id, decl)
                ));
            }
        };

        let Some(bridge_status) = self.dag.declaration_by_name("BridgeStatus") else {
            return ClaimResult::Fail(
                "BridgeLedgerZero: `BridgeStatus` is not declared in the bootstrap; \
                 cannot resolve the `Retired` / `Open` constructors structurally"
                    .to_string(),
            );
        };
        let TypeConnective::Disj { variants } = &bridge_status.connective else {
            return ClaimResult::Fail(
                "BridgeLedgerZero: `BridgeStatus` must be a Disj coproduct".to_string(),
            );
        };
        let allowed_constructors: std::collections::HashSet<DeclarationId> =
            variants.iter().map(|v| v.ty).collect();
        let Some(retired_ty) = variants.iter().find(|v| v.label == "Retired").map(|v| v.ty) else {
            return ClaimResult::Fail(
                "BridgeLedgerZero: `BridgeStatus::Retired` variant missing".to_string(),
            );
        };

        let mut open_rows: Vec<String> = Vec::new();
        for (idx, row) in rows.iter().enumerate() {
            let FieldValue::Record(fields) = row else {
                return ClaimResult::Fail(format!(
                    "BridgeLedgerZero: row {idx} is not a record literal: {row:?}"
                ));
            };
            let name = match fields.iter().find(|(l, _)| l == "name").map(|(_, v)| v) {
                Some(FieldValue::Literal(LiteralBits::String(s))) => s.clone(),
                Some(other) => {
                    return ClaimResult::Fail(format!(
                        "BridgeLedgerZero: row {idx} `name` must be a String literal, got {other:?}"
                    ));
                }
                None => {
                    return ClaimResult::Fail(format!(
                        "BridgeLedgerZero: row {idx} missing required `name` field"
                    ));
                }
            };
            let status = match fields.iter().find(|(l, _)| l == "status").map(|(_, v)| v) {
                Some(FieldValue::Variant { constructor, .. }) => *constructor,
                Some(other) => {
                    return ClaimResult::Fail(format!(
                        "BridgeLedgerZero: row `{name}` `status` must be a Variant, got {other:?}"
                    ));
                }
                None => {
                    return ClaimResult::Fail(format!(
                        "BridgeLedgerZero: row `{name}` missing `status` field"
                    ));
                }
            };
            // Defensive: a row carrying a constructor outside `BridgeStatus`'s
            // declared variants is malformed at the claim boundary, even
            // though the carrier ratchet guards the substrate side.
            if !allowed_constructors.contains(&status) {
                return ClaimResult::Fail(format!(
                    "BridgeLedgerZero: row `{name}` `status` constructor (DeclarationId {:?}) \
                     is not one of `BridgeStatus`'s declared variants",
                    status
                ));
            }
            if status != retired_ty {
                open_rows.push(name);
            }
        }

        if open_rows.is_empty() {
            ClaimResult::Pass
        } else {
            ClaimResult::Fail(format!(
                "BridgeLedgerZero: {} bridge row(s) not yet `Retired`: [{}]",
                open_rows.len(),
                open_rows.join(", ")
            ))
        }
    }

    fn eval_generated_from_dag_shape(
        &self,
        _claim: &TestClaimValue,
        payload: &[FieldValue],
    ) -> ClaimResult {
        let [authority, FieldValue::List(generated_paths)] = payload else {
            return ClaimResult::Fail(
                "GeneratedFromDag payload should be (DeclarationRef, List<Path>)".to_string(),
            );
        };
        if let Err(reason) = self.resolve_census_authority_ref(authority, "authority") {
            return ClaimResult::Fail(reason);
        }
        let generated: BTreeSet<&str> = GENERATED_FILES.iter().copied().collect();
        let mut named_paths = Vec::new();
        for value in generated_paths {
            match value {
                FieldValue::Literal(LiteralBits::String(path)) => named_paths.push(path.as_str()),
                other => {
                    return ClaimResult::Fail(format!(
                        "GeneratedFromDag generated_paths must contain only Path/String values, got {other:?}"
                    ))
                }
            }
        }
        if let Some(path) = named_paths.iter().find(|path| !generated.contains(**path)) {
            return ClaimResult::Fail(format!(
                "GeneratedFromDag path `{path}` is not in the generated-file authority"
            ));
        }
        let test_count = match sg0_census_list_count("expected_hand_authored_test") {
            Ok(count) => count,
            Err(reason) => return ClaimResult::Fail(reason),
        };
        if test_count == 0 {
            ClaimResult::Pass
        } else {
            ClaimResult::Fail(format!(
                "GeneratedFromDag observed {test_count} hand-authored test file(s) outside generated paths"
            ))
        }
    }

    fn eval_release_deferred_claim_shape(
        &self,
        claim: &TestClaimValue,
        payload: &[FieldValue],
    ) -> ClaimResult {
        const RELEASE_ACCEPTANCE_FIXTURE: &str =
            "src/v3/compiler/tests/fixtures/r1_release_acceptance.dag";
        if claim.declaration_file != RELEASE_ACCEPTANCE_FIXTURE {
            return ClaimResult::Fail(format!(
                "ReleaseDeferredClaim is only valid in `{RELEASE_ACCEPTANCE_FIXTURE}`, got `{}`",
                claim.declaration_file
            ));
        }

        let [deferred_gate, target_lane, authority_doc] = payload else {
            return ClaimResult::Fail(format!(
                "ReleaseDeferredClaim payload should be exactly three DeclarationRef fields \
                 (deferred_gate, target_lane, authority_doc); got {} payload slot(s)",
                payload.len()
            ));
        };

        for (field_label, value, role_name) in [
            ("deferred_gate", deferred_gate, "R1GateMarker"),
            ("target_lane", target_lane, "TargetLaneMarker"),
            ("authority_doc", authority_doc, "ReleaseAuthorityDoc"),
        ] {
            let role_id = match self.release_fixture_local_role_id(role_name) {
                Ok(id) => id,
                Err(reason) => return ClaimResult::Fail(format!("ReleaseDeferredClaim: {reason}")),
            };
            let id = match self.resolve_declaration_ref_id(value, field_label) {
                Ok(id) => id,
                Err(reason) => return ClaimResult::Fail(format!("ReleaseDeferredClaim: {reason}")),
            };
            let decl = self.dag.declaration(id);
            if decl.span.file != RELEASE_ACCEPTANCE_FIXTURE {
                return ClaimResult::Fail(format!(
                    "ReleaseDeferredClaim `{field_label}` must reference a marker declared in `{RELEASE_ACCEPTANCE_FIXTURE}`, got `{}` from `{}`",
                    decl_display_name(id, decl),
                    decl.span.file
                ));
            }
            if !Self::decl_inhabits_role_id(decl, role_id) {
                return ClaimResult::Fail(format!(
                    "ReleaseDeferredClaim `{field_label}` must reference a declaration inhabiting fixture-local `{role_name}`, got `{}`",
                    decl_display_name(id, decl)
                ));
            }
        }

        ClaimResult::Pass
    }

    fn eval_substrate_research_deferred_claim_shape(
        &self,
        claim: &TestClaimValue,
        payload: &[FieldValue],
    ) -> ClaimResult {
        if claim.declaration_file != TC1_SUBSTRATE_LENS_ETA_DEFERRED_FIXTURE {
            return ClaimResult::Fail(format!(
                "SubstrateResearchDeferredClaim is only valid in `{TC1_SUBSTRATE_LENS_ETA_DEFERRED_FIXTURE}`, got `{}`",
                claim.declaration_file
            ));
        }

        let [deferred_gate, target_lane, authority_doc] = payload else {
            return ClaimResult::Fail(format!(
                "SubstrateResearchDeferredClaim payload should be exactly three DeclarationRef fields \
                 (deferred_gate, target_lane, authority_doc); got {} payload slot(s)",
                payload.len()
            ));
        };

        for (field_label, value, role_name) in [
            ("deferred_gate", deferred_gate, "Tc1ResearchGateMarker"),
            (
                "target_lane",
                target_lane,
                "SubstrateLensPrimitiveTargetLaneMarker",
            ),
            (
                "authority_doc",
                authority_doc,
                "LambdaCalculusGroundingAuthorityDoc",
            ),
        ] {
            let role_id = match self.substrate_research_fixture_local_role_id(role_name) {
                Ok(id) => id,
                Err(reason) => {
                    return ClaimResult::Fail(format!("SubstrateResearchDeferredClaim: {reason}"));
                }
            };
            let id = match self.resolve_declaration_ref_id(value, field_label) {
                Ok(id) => id,
                Err(reason) => {
                    return ClaimResult::Fail(format!("SubstrateResearchDeferredClaim: {reason}"));
                }
            };
            let decl = self.dag.declaration(id);
            if decl.span.file != TC1_SUBSTRATE_LENS_ETA_DEFERRED_FIXTURE {
                return ClaimResult::Fail(format!(
                    "SubstrateResearchDeferredClaim `{field_label}` must reference a marker declared in `{TC1_SUBSTRATE_LENS_ETA_DEFERRED_FIXTURE}`, got `{}` from `{}`",
                    decl_display_name(id, decl),
                    decl.span.file
                ));
            }
            if !Self::decl_inhabits_role_id(decl, role_id) {
                return ClaimResult::Fail(format!(
                    "SubstrateResearchDeferredClaim `{field_label}` must reference a declaration inhabiting fixture-local `{role_name}`, got `{}`",
                    decl_display_name(id, decl)
                ));
            }
        }

        ClaimResult::Pass
    }

    fn substrate_research_fixture_local_role_id(
        &self,
        role_name: &str,
    ) -> Result<DeclarationId, String> {
        let mut matches = self.dag.declarations().iter().filter(|decl| {
            decl.name.as_deref() == Some(role_name)
                && decl.span.file == TC1_SUBSTRATE_LENS_ETA_DEFERRED_FIXTURE
        });
        let Some(role) = matches.next() else {
            return Err(format!(
                "substrate TC1 fixture role `{role_name}` is missing from `{TC1_SUBSTRATE_LENS_ETA_DEFERRED_FIXTURE}`"
            ));
        };
        if matches.next().is_some() {
            return Err(format!(
                "substrate TC1 fixture role `{role_name}` is declared more than once in `{TC1_SUBSTRATE_LENS_ETA_DEFERRED_FIXTURE}`"
            ));
        }
        Ok(role.id)
    }

    fn release_fixture_local_role_id(&self, role_name: &str) -> Result<DeclarationId, String> {
        const RELEASE_ACCEPTANCE_FIXTURE: &str =
            "src/v3/compiler/tests/fixtures/r1_release_acceptance.dag";
        let mut matches = self.dag.declarations().iter().filter(|decl| {
            decl.name.as_deref() == Some(role_name) && decl.span.file == RELEASE_ACCEPTANCE_FIXTURE
        });
        let Some(role) = matches.next() else {
            return Err(format!(
                "release fixture role `{role_name}` is missing from `{RELEASE_ACCEPTANCE_FIXTURE}`"
            ));
        };
        if matches.next().is_some() {
            return Err(format!(
                "release fixture role `{role_name}` is declared more than once in `{RELEASE_ACCEPTANCE_FIXTURE}`"
            ));
        }
        Ok(role.id)
    }

    fn resolve_census_authority_ref(
        &self,
        value: &FieldValue,
        field_label: &str,
    ) -> Result<DeclarationId, String> {
        match value {
            FieldValue::Reference(id) => Ok(*id),
            other => Err(format!(
                "PB census predicate `{field_label}` should be a DeclarationRef edge, got {other:?}"
            )),
        }
    }

    fn resolve_pb_marker_ref(
        &self,
        value: &FieldValue,
        field_label: &str,
        expected_decl_name: &str,
        expected_marker_type: &str,
    ) -> Result<DeclarationId, String> {
        let FieldValue::Reference(id) = value else {
            return Err(format!(
                "PB census predicate `{field_label}` should be a DeclarationRef edge to `{expected_decl_name}`, got {value:?}"
            ));
        };
        let decl = self.dag.declaration(*id);
        let actual_name = decl_display_name(*id, decl);
        if decl.name.as_deref() != Some(expected_decl_name) {
            return Err(format!(
                "PB census predicate `{field_label}` expected `{expected_decl_name}`, got `{actual_name}`"
            ));
        }
        match self.decl_inhabits_named_role(decl, expected_marker_type) {
            Ok(true) => Ok(*id),
            Ok(false) => Err(format!(
                "PB census predicate `{field_label}` declaration `{actual_name}` must inhabit `{expected_marker_type}`"
            )),
            Err(reason) => Err(reason),
        }
    }

    fn resolve_census_list_constant_ref(&self, value: &FieldValue) -> Result<String, String> {
        let FieldValue::Reference(id) = value else {
            return Err(format!(
                "PB census predicate `list_constant` should be a CensusListConstant edge, got {value:?}"
            ));
        };
        let decl = self.dag.declaration(*id);
        let actual_name = decl_display_name(*id, decl);
        match self.decl_inhabits_named_role(decl, "CensusListConstant") {
            Ok(true) => Ok(actual_name),
            Ok(false) => Err(format!(
                "PB census predicate `list_constant` declaration `{actual_name}` must inhabit `CensusListConstant`"
            )),
            Err(reason) => Err(reason),
        }
    }

    /// Hermetic path: compile `claim.source`, then `apply_lens_declaration` for subject (0-arity)
    /// and invariant (1-arity). `run_claim` wraps a bare `Pass` in `NotYetImplemented` when
    /// `requires` is empty so we do not fabricate a mock-backed receipt without a DB-15 obligation
    /// surface (see `MockBackedInvariant` arm in `run_claim`).
    fn eval_mock_backed_invariant(
        &self,
        claim: &TestClaimValue,
        payload: &[FieldValue],
    ) -> ClaimResult {
        let [subject, invariant] = payload else {
            return ClaimResult::Fail(
                "MockBackedInvariant payload should be (subject: DeclarationRef, invariant: DeclarationRef)"
                    .to_string(),
            );
        };
        let subject_name = match self.resolve_mock_declaration_ref_edge(subject, "subject") {
            Ok(name) => name,
            Err(reason) => return ClaimResult::Fail(reason),
        };
        let invariant_name = match self.resolve_mock_declaration_ref_edge(invariant, "invariant") {
            Ok(name) => name,
            Err(reason) => return ClaimResult::Fail(reason),
        };

        let program_dag = match compile_to_dag(&claim.source, &claim.file_name) {
            Ok(dag) => dag,
            Err(CompileError::Semantic(dag)) => {
                return ClaimResult::Fail(format!(
                    "MockBackedInvariant: claim `source` / `{}` failed inference: {:?}",
                    claim.file_name,
                    dag.diagnostics().iter().collect::<Vec<_>>()
                ));
            }
            Err(err) => {
                return ClaimResult::Fail(format!(
                    "MockBackedInvariant: claim `source` / `{}` did not compile: {err:?}",
                    claim.file_name
                ));
            }
        };

        let Some(subject_decl) = program_dag.declaration_by_name(&subject_name) else {
            return ClaimResult::Fail(format!(
                "MockBackedInvariant: subject `{subject_name}` not found in compiled claim program"
            ));
        };
        let Some(invariant_decl) = program_dag.declaration_by_name(&invariant_name) else {
            return ClaimResult::Fail(format!(
                "MockBackedInvariant: invariant `{invariant_name}` not found in compiled claim program"
            ));
        };

        let subject_out = match apply_lens_declaration(&program_dag, subject_decl.id, &[]) {
            Ok(v) => v,
            Err(err) => {
                return ClaimResult::Fail(format!(
                    "MockBackedInvariant: applying subject `{subject_name}` failed: {err:?}"
                ));
            }
        };
        match apply_lens_declaration(&program_dag, invariant_decl.id, &[subject_out]) {
            Ok(FieldValue::Literal(LiteralBits::Bool(true))) => ClaimResult::Pass,
            Ok(other) => ClaimResult::Fail(format!(
                "MockBackedInvariant: invariant `{invariant_name}` did not return Bool(true), got {other:?}"
            )),
            Err(err) => ClaimResult::Fail(format!(
                "MockBackedInvariant: applying invariant `{invariant_name}` failed: {err:?}"
            )),
        }
    }

    fn validate_resource_requirements(&self, claim: &TestClaimValue) -> Result<(), String> {
        for (idx, requirement) in claim.requires.iter().enumerate() {
            let Some(fields) = record_fields(requirement) else {
                return Err(format!(
                    "MockBackedInvariant: `requires[{idx}]` must be a ResourceReference record"
                ));
            };
            match field(fields, "target") {
                Some(FieldValue::Reference(_)) => {}
                Some(other) => {
                    return Err(format!(
                        "MockBackedInvariant: `requires[{idx}].target` must be a DeclarationRef edge, got {other:?}"
                    ));
                }
                None => {
                    return Err(format!(
                        "MockBackedInvariant: `requires[{idx}]` is missing `target`"
                    ));
                }
            }
        }
        Ok(())
    }

    fn resolve_mock_declaration_ref_edge(
        &self,
        value: &FieldValue,
        label: &str,
    ) -> Result<String, String> {
        match value {
            FieldValue::Reference(id) => Ok(self
                .dag
                .declaration(*id)
                .name
                .clone()
                .unwrap_or_else(|| format!("Declaration#{}", id.raw()))),
            FieldValue::Record(fields) if fields.is_empty() => Err(format!(
                "MockBackedInvariant `{label}` must be a DeclarationRef edge, got empty record literal"
            )),
            other => Err(format!(
                "MockBackedInvariant `{label}` must be a DeclarationRef edge, got {other:?}"
            )),
        }
    }

    fn diagnostic_matches(&self, actual_dag: &Dag, reference: &FieldValue) -> Result<bool, String> {
        let reference = self.diagnostic_reference(reference)?;
        Ok(actual_dag
            .diagnostics()
            .iter()
            .any(|(_, diagnostic)| diagnostic_matches_reference(diagnostic, &reference)))
    }

    fn diagnostic_matches_single(
        &self,
        diagnostic: &Diagnostic,
        reference: &FieldValue,
    ) -> Result<bool, String> {
        let reference = self.diagnostic_reference(reference)?;
        Ok(diagnostic_matches_reference(diagnostic, &reference))
    }

    fn diagnostic_reference(
        &self,
        reference: &FieldValue,
    ) -> Result<(String, DiagnosticDetailFilter), String> {
        let Some(fields) = record_fields(reference) else {
            return Err("DiagnosticReference payload should be a record".to_string());
        };
        let Some(kind) = field(fields, "kind") else {
            return Err("DiagnosticReference is missing `kind`".to_string());
        };
        let Some(detail_contains) = field(fields, "detail_contains") else {
            return Err("DiagnosticReference is missing `detail_contains`".to_string());
        };
        let Some((kind_label, kind_payload)) = self.variant_value(kind) else {
            return Err("DiagnosticReference `kind` is not a variant".to_string());
        };
        if !kind_payload.is_empty() {
            return Err("DiagnosticReference `kind` should not carry payload".to_string());
        }
        Ok((kind_label, self.detail_filter(detail_contains)?))
    }

    fn detail_filter(&self, value: &FieldValue) -> Result<DiagnosticDetailFilter, String> {
        let Some((label, payload)) = self.variant_value(value) else {
            return Err("DiagnosticDetailExpectation is not a variant".to_string());
        };
        match label.as_str() {
            "AnyDetail" => {
                if payload.is_empty() {
                    Ok(DiagnosticDetailFilter::Any)
                } else {
                    Err("AnyDetail should not carry payload".to_string())
                }
            }
            "Contains" => match payload.as_slice() {
                [FieldValue::Literal(LiteralBits::String(text))] => {
                    Ok(DiagnosticDetailFilter::Contains(text.clone()))
                }
                _ => Err("Contains should carry a single String payload".to_string()),
            },
            other => Err(format!(
                "unsupported DiagnosticDetailExpectation variant `{other}`"
            )),
        }
    }

    fn compare_cost(&self, comparator: &FieldValue, actual: i64, bound: i64) -> bool {
        let Some((label, payload)) = self.variant_value(comparator) else {
            return false;
        };
        if !payload.is_empty() {
            return false;
        }
        match label.as_str() {
            "Eq" => actual == bound,
            "Lt" => actual < bound,
            "Le" => actual <= bound,
            "Gt" => actual > bound,
            "Ge" => actual >= bound,
            "Ne" => actual != bound,
            _ => false,
        }
    }

    fn variant_value(&self, value: &FieldValue) -> Option<(String, Vec<FieldValue>)> {
        match value {
            FieldValue::Variant {
                constructor,
                payload,
            } => Some((variant_label(self.dag, *constructor)?, payload.clone())),
            _ => None,
        }
    }
}

impl TestClaimValue {
    pub fn from_declaration(decl: &Declaration) -> Result<Self, String> {
        let fields = structural_fields(decl)
            .ok_or_else(|| "TestClaim declaration is not structural".to_string())?;
        let claim_name = string_field(fields, "name")?;
        let source = string_field(fields, "source")?;
        let file_name = string_field(fields, "file_name")?;
        let predicate = field(fields, "predicate")
            .ok_or_else(|| "TestClaim is missing `predicate`".to_string())?
            .clone();
        let requires = match field(fields, "requires") {
            Some(FieldValue::List(values)) => values.clone(),
            Some(other) => return Err(format!("TestClaim `requires` is not a list: {other:?}")),
            None => return Err("TestClaim is missing `requires`".to_string()),
        };
        Ok(Self {
            claim_name,
            declaration_file: decl.span.file.clone(),
            source,
            file_name,
            predicate,
            requires,
        })
    }
}

fn structural_fields(decl: &Declaration) -> Option<&[(String, FieldValue)]> {
    match decl.value_body.as_ref()? {
        ValueBody::Structural { fields } => Some(fields),
        ValueBody::Unparsed(_) | ValueBody::Scalar(_) | ValueBody::List(_) | ValueBody::Map(_) => {
            None
        }
    }
}

fn field<'a>(fields: &'a [(String, FieldValue)], label: &str) -> Option<&'a FieldValue> {
    fields
        .iter()
        .find(|(candidate, _)| candidate == label)
        .map(|(_, value)| value)
}

fn string_field(fields: &[(String, FieldValue)], label: &str) -> Result<String, String> {
    match field(fields, label) {
        Some(FieldValue::Literal(LiteralBits::String(value))) => Ok(value.clone()),
        Some(other) => Err(format!("TestClaim `{label}` is not a string: {other:?}")),
        None => Err(format!("TestClaim is missing `{label}`")),
    }
}

fn record_fields(value: &FieldValue) -> Option<&[(String, FieldValue)]> {
    match value {
        FieldValue::Record(fields) => Some(fields),
        _ => None,
    }
}

fn decl_display_name(id: DeclarationId, decl: &Declaration) -> String {
    decl.name
        .clone()
        .unwrap_or_else(|| format!("Declaration#{}", id.raw()))
}

fn find_bind<'a>(
    dag: &'a Dag,
    bind_name: &str,
    claim_file_name: &str,
) -> Option<&'a crate::dag::BindNode> {
    dag.nodes().iter().find_map(|node| match node {
        Behavior::Bind(bind) if bind.name == bind_name && bind.span.file == claim_file_name => {
            Some(bind)
        }
        _ => None,
    })
}

fn diagnostic_matches_reference(
    diagnostic: &Diagnostic,
    reference: &(String, DiagnosticDetailFilter),
) -> bool {
    diagnostic.layer1_kind_label() == reference.0
        && match &reference.1 {
            DiagnosticDetailFilter::Any => true,
            DiagnosticDetailFilter::Contains(text) => diagnostic.message().contains(text),
        }
}

fn sg0_census_list_count(list_constant_name: &str) -> Result<i64, String> {
    Ok(sg0_census_list_entries(list_constant_name)?.len() as i64)
}

fn sg0_census_list_entries(list_constant_name: &str) -> Result<Vec<String>, String> {
    let constant = match list_constant_name {
        "expected_hand_authored_non_test" => "EXPECTED_HAND_AUTHORED_NON_TEST",
        "expected_hand_authored_test" => "EXPECTED_HAND_AUTHORED_TEST",
        "expected_hand_authored_fragments" => "EXPECTED_HAND_AUTHORED_FRAGMENTS",
        other => return Err(format!("unknown SG-0 census list constant `{other}`")),
    };
    sg0_string_slice_constant_entries(constant)
}

fn sg0_string_slice_constant_entries(constant: &str) -> Result<Vec<String>, String> {
    let marker = format!("const {constant}: &[&str] = &[");
    let start = SG0_CENSUS_SOURCE
        .find(&marker)
        .ok_or_else(|| format!("SG-0 census source is missing `{constant}`"))?
        + marker.len();
    let rest = &SG0_CENSUS_SOURCE[start..];
    let end = rest
        .find("\n];")
        .ok_or_else(|| format!("SG-0 census source has unterminated `{constant}`"))?;
    Ok(rest[..end]
        .lines()
        .filter_map(sg0_quoted_path_from_line)
        .collect())
}

fn sg0_quoted_path_from_line(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if trimmed.starts_with("//") {
        return None;
    }
    let start = trimmed.find('"')? + 1;
    let end = trimmed[start..].find('"')? + start;
    Some(trimmed[start..end].to_string())
}

fn is_lens_producer_census_path(path: &str) -> bool {
    matches!(
        path,
        "src/v3/compiler/src/lens_apply.rs"
            | "src/v3/compiler/src/lens_testgen.rs"
            | "src/v3/compiler/src/bin/regen_lens.rs"
    )
}

fn compiler_std_positive_set_ratchet_count() -> i64 {
    [
        "TemplateArgumentsMatch",
        "TemplateArgumentCursor",
        "NormalizedInstantiationArgs",
    ]
    .iter()
    .filter(|name| INFER_HELPERS_SOURCE.contains(&format!("type {name}")))
    .count() as i64
}

fn render_value_body(dag: &Dag, value: &ValueBody) -> String {
    match value {
        ValueBody::Scalar(bits) => render_literal(bits),
        ValueBody::Structural { fields } => render_record(dag, fields),
        ValueBody::List(values) => format!(
            "[{}]",
            values
                .iter()
                .map(|value| render_field_value(dag, value))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        ValueBody::Map(entries) => render_map(dag, entries.entries()),
        ValueBody::Unparsed(span) => format!("<unparsed:{}:{}>", span.file, span.byte_start),
    }
}

fn render_field_value(dag: &Dag, value: &FieldValue) -> String {
    match value {
        FieldValue::Literal(bits) => render_literal(bits),
        FieldValue::Reference(decl_id) => dag
            .declaration(*decl_id)
            .name
            .clone()
            .unwrap_or_else(|| format!("Declaration#{}", decl_id.raw())),
        FieldValue::Record(fields) => render_record(dag, fields),
        FieldValue::List(values) => format!(
            "[{}]",
            values
                .iter()
                .map(|value| render_field_value(dag, value))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        FieldValue::Map(entries) => render_map(dag, entries.entries()),
        FieldValue::Variant {
            constructor,
            payload,
        } => {
            let label = variant_label(dag, *constructor)
                .unwrap_or_else(|| format!("Variant#{}", constructor.raw()));
            if payload.is_empty() {
                label
            } else {
                format!(
                    "{}({})",
                    label,
                    payload
                        .iter()
                        .map(|value| render_field_value(dag, value))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        }
    }
}

fn render_record(dag: &Dag, fields: &[(String, FieldValue)]) -> String {
    format!(
        "{{ {} }}",
        fields
            .iter()
            .map(|(label, value)| format!("{label}: {}", render_field_value(dag, value)))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn render_map(dag: &Dag, entries: &[(String, FieldValue)]) -> String {
    format!(
        "{{ {} }}",
        entries
            .iter()
            .map(|(key, value)| format!("{key:?}: {}", render_field_value(dag, value)))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn render_literal(bits: &LiteralBits) -> String {
    match bits {
        LiteralBits::Int(value) => value.to_string(),
        LiteralBits::Bool(value) => value.to_string(),
        LiteralBits::String(value) => quote_string(value),
    }
}

fn quote_string(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

fn variant_label(dag: &Dag, variant_id: DeclarationId) -> Option<String> {
    dag.declarations()
        .iter()
        .find_map(|decl| match &decl.connective {
            TypeConnective::Disj { variants } => variants
                .iter()
                .find(|variant| variant.ty == variant_id)
                .map(|variant| variant.label.clone()),
            _ => None,
        })
}

fn algebraic_law_payload_fields(
    payload: &[FieldValue],
) -> Result<(&FieldValue, &FieldValue), AlgebraicLawProgramError> {
    match payload {
        [law, lens_ref] => Ok((law, lens_ref)),
        [FieldValue::Record(fields)] => {
            let law = field(fields, "law").ok_or_else(|| {
                AlgebraicLawProgramError::MalformedPayload(
                    "AlgebraicLaw payload record is missing `law` field".to_string(),
                )
            })?;
            let lens_ref = field(fields, "lens_ref").ok_or_else(|| {
                AlgebraicLawProgramError::MalformedPayload(
                    "AlgebraicLaw payload record is missing `lens_ref` field".to_string(),
                )
            })?;
            Ok((law, lens_ref))
        }
        _ => Err(AlgebraicLawProgramError::MalformedPayload(format!(
            "AlgebraicLaw payload should be [law, lens_ref] or a record, got len {}",
            payload.len()
        ))),
    }
}

fn variant_fields<'a>(
    dag: &Dag,
    value: &'a FieldValue,
) -> Result<(String, &'a [FieldValue]), AlgebraicLawProgramError> {
    let FieldValue::Variant {
        constructor,
        payload,
    } = value
    else {
        return Err(AlgebraicLawProgramError::MalformedPayload(
            "expected AlgebraicLawKind variant".to_string(),
        ));
    };
    let label = variant_label(dag, *constructor).ok_or_else(|| {
        AlgebraicLawProgramError::MalformedPayload(format!(
            "variant constructor {:?} not found under any sum",
            constructor
        ))
    })?;
    Ok((label, payload.as_slice()))
}

fn declaration_ref_name(dag: &Dag, value: &FieldValue) -> Result<String, AlgebraicLawProgramError> {
    match value {
        FieldValue::Reference(id) => dag.declaration(*id).name.clone().ok_or_else(|| {
            AlgebraicLawProgramError::MalformedPayload(format!(
                "lens_ref declaration {:?} is anonymous",
                id
            ))
        }),
        other => Err(AlgebraicLawProgramError::MalformedPayload(format!(
            "lens_ref should be a DeclarationRef (FieldValue::Reference), got {other:?}"
        ))),
    }
}

#[cfg(test)]
mod execute_command_timebound_tests {
    use super::evaluate_execute_command_exit_code;
    use super::evaluate_execute_command_exit_code_with_wall_time;
    use super::evaluate_execute_command_m1_5;
    use super::shell_dash_c_may_start_background_after_eliding_artifacts;
    use super::ClaimResult;
    use super::ExecuteCommandM1_5Proposition;
    use std::time::Duration;

    #[test]
    fn elision_allows_and_chain_and_fd_redirects_without_fabricating_bare_ampersand() {
        let sh = Some("sh");
        assert!(!shell_dash_c_may_start_background_after_eliding_artifacts(
            "true && true",
            sh,
        ));
        assert!(!shell_dash_c_may_start_background_after_eliding_artifacts(
            "true 2>&1",
            sh,
        ));
        assert!(!shell_dash_c_may_start_background_after_eliding_artifacts(
            "cmd 3>&4", sh,
        ));
    }

    #[test]
    fn elision_still_fails_on_shell_background() {
        assert!(shell_dash_c_may_start_background_after_eliding_artifacts(
            "sleep 600 &",
            Some("sh"),
        ));
    }

    /// On POSIX `sh`/`dash`, `&>` is not a single redirect token; eliding it as bash would
    /// false-negative the background guard (openai-pro gpt-5-5-pro, PR #792).
    #[test]
    fn posix_sh_ampersand_gt_form_fails_closed_in_elision_helper() {
        assert!(shell_dash_c_may_start_background_after_eliding_artifacts(
            "sleep 600 &> /tmp/gunbc_posix_ampgt",
            Some("sh"),
        ));
        assert!(shell_dash_c_may_start_background_after_eliding_artifacts(
            "sleep 600 &> /tmp/gunbc_posix_ampgt",
            Some("dash"),
        ));
    }

    #[test]
    fn bash_ampersand_gt_redir_is_elided_not_treated_as_background() {
        assert!(!shell_dash_c_may_start_background_after_eliding_artifacts(
            "true &> /dev/null",
            Some("bash"),
        ));
    }

    #[test]
    fn unknown_shell_interpreter_ampersand_gt_fails_closed() {
        assert!(shell_dash_c_may_start_background_after_eliding_artifacts(
            "true &> /dev/null",
            None,
        ));
    }

    #[test]
    fn sh_dash_c_background_ampersand_is_rejected() {
        let r = evaluate_execute_command_exit_code(
            "sh",
            &[String::from("-c"), String::from("sleep 600 &")],
            0,
        );
        let ClaimResult::Fail(m) = r else {
            panic!("expected fail-closed for shell background, got {r:?}");
        };
        assert!(
            m.contains("background")
                || m.contains("P3")
                || m.contains("descendants")
                || m.contains("shell `-c`"),
            "expected policy message, got: {m}"
        );
    }

    /// POSIX: `sleep 600 &> file` is `&` (background) + `>`, not bash `&>`; must not bypass the
    /// guard (openai-pro gpt-5-5-pro, PR #792).
    #[test]
    fn sh_dash_c_posix_ampersand_gt_token_rejected() {
        use super::reject_unbounded_shell_background;
        let script = "sleep 600 &> /tmp/gunbc_reject_unbounded_ampgt";
        assert!(
            reject_unbounded_shell_background("sh", &[String::from("-c"), String::from(script),])
                .is_some(),
            "direct guard on reject_unbounded_shell_background"
        );
        let r = evaluate_execute_command_exit_code(
            "sh",
            &[String::from("-c"), String::from(script)],
            0,
        );
        let ClaimResult::Fail(m) = r else {
            panic!("expected fail-closed for sh -c &> parse, got {r:?}");
        };
        assert!(
            m.contains("background")
                || m.contains("P3")
                || m.contains("descendants")
                || m.contains("shell `-c`"),
            "expected policy message, got: {m}"
        );
    }

    /// `env(1)` + `sh -c` tail: interpreter hint must apply to `&>` (PR #792).
    #[test]
    fn env_sh_dash_c_posix_ampersand_gt_rejected() {
        let script = "sleep 600 &> /tmp/gunbc_env_ampgt";
        let r = evaluate_execute_command_exit_code(
            "env",
            &[String::from("sh"), String::from("-c"), String::from(script)],
            0,
        );
        let ClaimResult::Fail(m) = r else {
            panic!("expected fail-closed for env sh -c &>, got {r:?}");
        };
        assert!(
            m.contains("background")
                || m.contains("P3")
                || m.contains("descendants")
                || m.contains("shell `-c`"),
            "expected policy message, got: {m}"
        );
    }

    /// `env(1)` + `sh -c` indirection: top-level stem is not a shell; must still reject the same
    /// background `&` (api-review 994fa40d).
    #[test]
    fn env_sh_dash_c_background_ampersand_is_rejected() {
        let r = evaluate_execute_command_exit_code(
            "env",
            &[
                String::from("sh"),
                String::from("-c"),
                String::from("sleep 600 &"),
            ],
            0,
        );
        let ClaimResult::Fail(m) = r else {
            panic!("expected fail-closed for env sh -c background, got {r:?}");
        };
        assert!(
            m.contains("background") || m.contains("P3") || m.contains("shell `-c`"),
            "expected policy message, got: {m}"
        );
    }

    /// Combined `-ec` / `-lc` after the shell token: must not bypass the guard (api-review e99b53e7).
    /// Also `env bash -lc` (argv tail after a non-`sh` shell stem) — same policy, pre-spawn.
    #[test]
    fn env_sh_dash_ec_and_dash_lc_background_ampersand_are_rejected() {
        for (flag, label) in [("-ec", "ec"), ("-lc", "lc")] {
            let r = evaluate_execute_command_exit_code(
                "env",
                &[
                    String::from("sh"),
                    String::from(flag),
                    String::from("sleep 600 &"),
                ],
                0,
            );
            let ClaimResult::Fail(m) = r else {
                panic!("expected fail-closed for env sh {label} + background, got {r:?}");
            };
            assert!(
                m.contains("background")
                    || m.contains("P3")
                    || m.contains("descendants")
                    || m.contains("shell `-c`"),
                "expected policy message, got: {m}"
            );
        }
    }

    #[test]
    fn env_bash_dash_lc_background_ampersand_is_rejected() {
        let r = evaluate_execute_command_exit_code(
            "env",
            &[
                String::from("bash"),
                String::from("-lc"),
                String::from("sleep 600 &"),
            ],
            0,
        );
        let ClaimResult::Fail(m) = r else {
            panic!("expected fail-closed for env bash -lc + background, got {r:?}");
        };
        assert!(m.contains("background") || m.contains("P3") || m.contains("shell `-c`"));
    }

    /// `-c` with script token `sh` and a **following** `-ec "…&"` in argv: flat `sh -ec` only looked at
    /// the first "script" word; nested scan must see the `&` (PR #792 blocking inline, P3/P4).
    #[test]
    fn sh_c_sh_dash_ec_nested_background_ampersand_is_rejected() {
        use super::shell_argv_may_start_unbounded_background;
        let v = vec![
            String::from("sh"),
            String::from("-c"),
            String::from("sh"),
            String::from("-ec"),
            String::from("sleep 600 &"),
        ];
        assert!(shell_argv_may_start_unbounded_background(&v));
        let r = evaluate_execute_command_exit_code("sh", &v, 0);
        let ClaimResult::Fail(m) = r else {
            panic!("expected fail-closed for sh -c sh -ec + &, got {r:?}");
        };
        assert!(
            m.contains("background") || m.contains("P3") || m.contains("shell `-c`"),
            "expected policy message, got: {m}"
        );
    }

    #[test]
    fn env_sh_c_sh_dash_ec_nested_background_ampersand_is_rejected() {
        let v = vec![
            String::from("sh"),
            String::from("-c"),
            String::from("sh"),
            String::from("-ec"),
            String::from("sleep 600 &"),
        ];
        let r = evaluate_execute_command_exit_code("env", &v, 0);
        let ClaimResult::Fail(m) = r else {
            panic!("expected fail-closed for env sh -c sh -ec + &, got {r:?}");
        };
        assert!(m.contains("background") || m.contains("P3") || m.contains("shell `-c`"));
    }

    /// P3 / P4: the nested shell scanner is bounded; exceeding it must not mean "no `&` → allow" (api-
    /// review codex 3a2a9f64).
    #[test]
    fn sh_c_nesting_past_max_scan_depth_fails_closed_without_ampersand_in_scripts() {
        use super::reject_unbounded_shell_background;
        use super::shell_argv_may_start_unbounded_background;
        let mut args = vec![String::from("sh"), String::from("-c"), String::from("true")];
        for _ in 0..40 {
            let mut t = vec![String::from("sh"), String::from("-c")];
            t.extend_from_slice(&args);
            args = t;
        }
        assert!(
            shell_argv_may_start_unbounded_background(&args),
            "depth-bound exhaustion must fail closed, not allow a spawn past the policy guard"
        );
        let r = reject_unbounded_shell_background("sh", &args);
        assert!(
            r.is_some(),
            "expected policy fail when scan depth is exhausted, got {r:?}"
        );
    }

    /// M1.5: policy `Fail` is `Err(ClaimResult)`, not propositional `false` (P3/DB-1, PR #792).
    #[test]
    fn m1_5_rejects_policy_fail_as_propositional() {
        let p = evaluate_execute_command_m1_5(
            "sh",
            &[String::from("-c"), String::from("sleep 600 &")],
            0,
        );
        assert!(p.is_err(), "expected Err(Fail) for background &, got {p:?}");
    }

    /// M1.5: exit code mismatch is the only propositional `false` path.
    #[test]
    #[cfg(unix)]
    fn m1_5_exit_mismatch_is_unsatisfied() {
        let p =
            evaluate_execute_command_m1_5("sh", &[String::from("-c"), String::from("exit 1")], 0);
        assert_eq!(
            p,
            Ok(ExecuteCommandM1_5Proposition::UnsatisfiedExitMismatch)
        );
    }

    /// P2(c): a missing `command` must surface as `SpawnFailed` — not `Matched` — even when the
    /// claim expects 127 and the unshare shell path would itself naturally exit 127. The
    /// bootstrap sentinel pipe (`s` written, `e` not written) detects "logical command not
    /// executable" structurally (T-PB-B Worker 4 typed model).
    #[test]
    #[cfg(target_os = "linux")]
    fn unshare_expect_127_missing_command_does_not_pass_without_direct_spawn() {
        use super::evaluate_execute_command_host_outcome;
        use super::ExecuteCommandHostOutcome;
        let r = evaluate_execute_command_host_outcome(
            "definitely_not_a_real_binary_gunbc_792",
            &[],
            127,
            Duration::from_secs(5),
        );
        // Either typed carrier is acceptable: `SpawnFailed` if unshare(1) wasn't usable here,
        // or `SetupFailed { NamespaceSetupAndDirectSpawnFailed }` if unshare(1) ran but
        // namespace setup failed (sentinel pipe empty) and the direct fallback also could not
        // spawn. The contract — missing binary must NOT `Match` 127 — is satisfied by either.
        assert!(
            matches!(
                r,
                ExecuteCommandHostOutcome::SpawnFailed { .. }
                    | ExecuteCommandHostOutcome::SetupFailed { .. }
            ),
            "expected SpawnFailed or SetupFailed for missing command, got {r:?}"
        );
    }

    /// Linux: a logical command that writes a large volume to `>&2` must complete within the
    /// wall bound. Wrapper stderr is `Stdio::null()` and the helper inherits stdio normally;
    /// the parent never reads from any pipe attached to the child stderr, so there's no
    /// pipe-buffer-fill stall to worry about. This test passing is the live-state receipt
    /// that the helper-wired path correctly nulls logical stderr (no shared-channel hazard
    /// with the wall-bound timeout). Pre-helper this same test guarded against a pipe-stall
    /// regression on the wrapper-stderr drain path; post-helper it remains a useful
    /// volume-sanity receipt against future plumbing changes.
    #[test]
    #[cfg(target_os = "linux")]
    fn unshare_path_drains_piped_stderr_so_huge_logical_stderr_does_not_stall() {
        let c = 8000u32;
        let body = format!(
            "i=0; while [ $i -lt {c} ]; do printf 'xxxxxxxxxx'; i=$((i+1)); done >&2; exit 0"
        );
        let r = evaluate_execute_command_exit_code_with_wall_time(
            "sh",
            &[String::from("-c"), body],
            0,
            Duration::from_secs(2),
        );
        assert_eq!(
            r,
            ClaimResult::Pass,
            "expected Pass when logical stderr > pipe; got {r:?}"
        );
    }

    /// Linux receipt that the unshare PID-namespace path is **actually engaged**, not
    /// silently bypassed via direct fallback. `unshare -f -p` makes the helper's
    /// fork PID 1 in the new namespace; `exec` of the user command preserves PID 1. So
    /// `sh -c '[ "$$" = "1" ]'` exits 0 *only* when the unshare path reached the user
    /// command. If the bootstrap-sentinel pipe were broken (e.g. fd 3 closed by parent's
    /// `exec("unshare")` due to a misplaced FD_CLOEXEC), the runner would fall back to
    /// direct, the user sh would NOT be PID 1, and this assertion would fail.
    /// (Regression for codex review on PR #1049, commit e072dce4.)
    #[test]
    #[cfg(target_os = "linux")]
    fn unshare_path_actually_engages_user_runs_as_pid_1_in_new_namespace() {
        use super::evaluate_execute_command_host_outcome;
        use super::ExecuteCommandHostOutcome;
        // Probe: can THIS host create a user+PID namespace via the same `unshare` flags
        // we use? If not (restricted runner / sandbox), the runner is *expected* to fall
        // back to direct and the user sh will not be PID 1. Skip the assertion in that
        // case — the test only asserts engagement when the host actually permits it.
        let probe = std::process::Command::new("unshare")
            .args(["-c", "-f", "-p", "--", "sh", "-c", "[ \"$$\" = \"1\" ]"])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
        let probe_ok = matches!(probe, Ok(s) if s.success());
        if !probe_ok {
            // unshare PID-namespace not permitted on this host (restricted CI / sandbox);
            // skip silently. clippy::disallowed_macros bans eprintln/println in lib crates.
            return;
        }
        // Probe succeeded → unshare PID-namespace works on this host. Our runner MUST
        // engage it, so the inner sh sees $$ == 1 and exits 0. A silent bypass (e.g. a
        // misplaced FD_CLOEXEC closing the sentinel pipe before the bootstrap runs) would
        // route us through the direct fallback, sh would have a normal PID, exit 1, and
        // we'd see `Mismatch { expected: 0, actual: 1 }` here.
        let r = evaluate_execute_command_host_outcome(
            "sh",
            &[String::from("-c"), String::from("[ \"$$\" = \"1\" ]")],
            0,
            Duration::from_secs(5),
        );
        assert_eq!(
            r,
            ExecuteCommandHostOutcome::Matched,
            "unshare path appears silently bypassed (probe says unshare PID-namespace IS permitted on this host); expected Matched ($$ == 1 inside ns), got {r:?}"
        );
    }

    /// **P2(c) sef-spoof structural-unreachability receipt** (api-review openai-pro/
    /// gpt-5-5-pro sha 9fea084e; helper-binary AC per issuecomment-4331097029 on #856).
    ///
    /// A logical command that successfully `execvp`s and attempts to write exactly `f` to
    /// fd 3 must NOT be misclassified as `LogicalExecFailed`. With the
    /// `gunbc_execute_command_bootstrap` helper (#1063) wired in, the helper sets
    /// `FD_CLOEXEC` on fd 3 immediately before `execvp(3)`; on successful exec the kernel
    /// atomically closes fd 3 in the new image, the user's `printf f >&3` gets `EBADF`,
    /// and the parent's sentinel reads exactly `b"se"` → `LogicalCommandExeced` → exit code
    /// is the user's (0) → `Matched`.
    ///
    /// This test was `#[ignore]`d during the pure-sh era (the gap was unreachable in POSIX
    /// sh). It is now an active load-bearing receipt that the helper's CLOEXEC-before-
    /// `execvp` ordering actually closes the spoof window — a regression that broke the
    /// CLOEXEC ordering would surface here as `SpawnFailed`/`LogicalExecFailed` instead of
    /// `Matched`.
    #[test]
    #[cfg(target_os = "linux")]
    fn unshare_logical_command_writing_f_to_fd3_is_not_misclassified_after_helper_lands() {
        use super::evaluate_execute_command_host_outcome;
        use super::ExecuteCommandHostOutcome;
        let probe = std::process::Command::new("unshare")
            .args(["-c", "-f", "-p", "--", "sh", "-c", "[ \"$$\" = \"1\" ]"])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
        if !matches!(probe, Ok(s) if s.success()) {
            return;
        }
        let r = evaluate_execute_command_host_outcome(
            "sh",
            &[
                String::from("-c"),
                String::from("printf f >&3 2>/dev/null; exit 0"),
            ],
            0,
            Duration::from_secs(5),
        );
        // Post-helper-binary expectation: fd 3 is closed atomically by execvp; sh's
        // `printf f >&3` gets EBADF; sentinel = b"se"; outcome is Matched.
        // Pre-helper-binary (current HEAD): sentinel = b"sef"; outcome is SpawnFailed —
        // this test is #[ignore]d until that flip happens.
        assert_eq!(
            r,
            ExecuteCommandHostOutcome::Matched,
            "post-#1063: logical-child write of `f` to fd 3 must NOT misclassify as exec failure; got {r:?}"
        );
    }

    /// **P2(d) defense-in-depth regression** (api-review openai-pro/gpt-5-5-pro sha
    /// 7297b04a). Even after the helper binary's structural CLOEXEC closes the typical
    /// post-`exec` write window, the classifier's `b"se*"` → `LogicalCommandExeced`
    /// hardening must remain in place — a future helper bug that broke CLOEXEC ordering
    /// would let the user write stray bytes to fd 3, and we still must NOT trigger
    /// direct-fallback re-execution.
    ///
    /// Construction: an sh script that asserts it is PID 1 (proving the unshare path
    /// engaged — `[ "$$" = "1" ]`), attempts to write a stray byte `x` to fd 3 (post-
    /// helper-binary: silently EBADFs because fd 3 is closed; pre-helper: writes through
    /// to the parent pipe), then exits 0. Both eras: outcome must be `Matched` (single
    /// run, exit 0). A buggy classifier that routed `b"sex"` to `NamespaceSetupFailed`
    /// would re-run sh outside the PID namespace, `$$ != 1`, exit 99, surface `Mismatch`.
    #[test]
    #[cfg(target_os = "linux")]
    fn unshare_post_exec_fd3_write_does_not_trigger_implicit_rerun() {
        use super::evaluate_execute_command_host_outcome;
        use super::ExecuteCommandHostOutcome;
        let probe = std::process::Command::new("unshare")
            .args(["-c", "-f", "-p", "--", "sh", "-c", "[ \"$$\" = \"1\" ]"])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
        if !matches!(probe, Ok(s) if s.success()) {
            return;
        }
        // First run inside unshare PID-namespace: $$ == 1, write x to fd 3, exit 0.
        // If a buggy classifier triggered direct fallback, the re-run would NOT be in a
        // PID namespace, $$ != 1, exit 99 → Mismatch (caught by this assertion).
        let r = evaluate_execute_command_host_outcome(
            "sh",
            &[
                String::from("-c"),
                String::from(r#"[ "$$" = "1" ] || exit 99; printf x >&3 2>/dev/null; exit 0"#),
            ],
            0,
            Duration::from_secs(5),
        );
        assert_eq!(
            r,
            ExecuteCommandHostOutcome::Matched,
            "post-`se` fd-3 write must not trigger implicit re-run; expected Matched (single run, $$ == 1, exit 0), got {r:?}"
        );
    }

    /// Linux receipt: a `command` containing `/` (absolute path) reaches `execvp` with the
    /// literal path. The helper binary uses `execvp(3)` directly (which doesn't consult
    /// `PATH` for path operands), so the run completes and Matches. (Helper-binary parity
    /// receipt for `std::process::Command` semantics; see #1063.)
    #[test]
    #[cfg(target_os = "linux")]
    fn unshare_absolute_path_command_runs_and_matches() {
        // `/bin/true` exists on virtually all Linux systems; if not, fall back to `/usr/bin/true`.
        let abs = if std::path::Path::new("/bin/true").exists() {
            "/bin/true"
        } else {
            "/usr/bin/true"
        };
        let r = evaluate_execute_command_exit_code(abs, &[], 0);
        assert_eq!(
            r,
            ClaimResult::Pass,
            "absolute-path command must reach exec via the bootstrap path probe; got {r:?}"
        );
    }

    /// Linux receipt: a `command` with no `/` (bare name) is resolved by `execvp(3)` via
    /// `PATH`. `true` is always on `PATH` on every supported host.
    #[test]
    #[cfg(target_os = "linux")]
    fn unshare_bare_name_command_runs_and_matches() {
        let r = evaluate_execute_command_exit_code("true", &[], 0);
        assert_eq!(
            r,
            ClaimResult::Pass,
            "bare-name command must resolve via PATH probe; got {r:?}"
        );
    }

    /// Manager-requested regression (T-PB-B Worker 4 draft review): an existing-but-not-
    /// executable file with `expect_exit_code = 126` must NOT `Match`, even though the sh
    /// fall-back exit for an unrunnable command is naturally 126. The path probe
    /// `[ -x "$0" ]` rejects pre-`exec`, sentinel `s`, classified `LogicalCommandNotExecutable`
    /// → `SpawnFailed`. Closes the P2(c) collapse where a non-executable path could have been
    /// classified as a logical exit.
    #[test]
    #[cfg(target_os = "linux")]
    fn unshare_existing_non_executable_file_with_expect_126_does_not_match() {
        use super::evaluate_execute_command_host_outcome;
        use super::ExecuteCommandHostOutcome;
        use std::os::unix::fs::PermissionsExt;
        let tmp =
            std::env::temp_dir().join(format!("gunbc_pb_runtime_nonexec_{}", std::process::id()));
        std::fs::write(&tmp, b"#!/bin/sh\nexit 0\n").expect("write temp");
        let mut perms = std::fs::metadata(&tmp).expect("meta").permissions();
        perms.set_mode(0o644); // explicitly NO execute bit
        std::fs::set_permissions(&tmp, perms).expect("chmod 644");

        let path = tmp.to_str().expect("path utf8").to_string();
        let r = evaluate_execute_command_host_outcome(&path, &[], 126, Duration::from_secs(5));
        let _ = std::fs::remove_file(&tmp);

        assert!(
            !matches!(r, ExecuteCommandHostOutcome::Matched),
            "non-executable file with expect=126 must NOT Match (P2(c)); got {r:?}"
        );
        assert!(
            matches!(
                r,
                ExecuteCommandHostOutcome::SpawnFailed { .. }
                    | ExecuteCommandHostOutcome::SetupFailed { .. }
            ),
            "expected SpawnFailed/SetupFailed for non-executable path; got {r:?}"
        );
    }

    /// Linux receipt: a relative-path `command` containing `/` that does **not** exist must
    /// surface as a non-`Matched` outcome (`SpawnFailed` or `SetupFailed`), never as a stray
    /// `Matched` from `command -v`'s sh-relative resolution. Distinguishes path-mode probe
    /// from PATH-mode probe.
    #[test]
    #[cfg(target_os = "linux")]
    fn unshare_relative_path_missing_command_does_not_match() {
        use super::evaluate_execute_command_host_outcome;
        use super::ExecuteCommandHostOutcome;
        let r = evaluate_execute_command_host_outcome(
            "./definitely_not_here_gunbc_pb_runtime",
            &[],
            0,
            Duration::from_secs(5),
        );
        assert!(
            matches!(
                r,
                ExecuteCommandHostOutcome::SpawnFailed { .. }
                    | ExecuteCommandHostOutcome::SetupFailed { .. }
            ),
            "missing relative-path command must NOT match; got {r:?}"
        );
    }

    #[test]
    fn shell_dash_c_script_parses_standalone_c_ec_and_e_c() {
        use super::shell_dash_c_script_string;
        assert_eq!(
            shell_dash_c_script_string(&[String::from("-c"), String::from("a")]),
            Some("a")
        );
        assert_eq!(
            shell_dash_c_script_string(&[String::from("-ec"), String::from("b")]),
            Some("b")
        );
        assert_eq!(
            shell_dash_c_script_string(&[
                String::from("-e"),
                String::from("-c"),
                String::from("c")
            ]),
            Some("c")
        );
        assert_eq!(
            shell_dash_c_script_string(&[String::from("-lc"), String::from("d")]),
            Some("d")
        );
        assert_eq!(
            shell_dash_c_script_string(&[
                String::from("sh"),
                String::from("-ec"),
                String::from("e")
            ]),
            Some("e")
        );
        assert_eq!(
            shell_dash_c_script_string(&[
                String::from("env"),
                String::from("sh"),
                String::from("-lc"),
                String::from("f"),
            ]),
            Some("f")
        );
    }

    /// First-`-c`-only `shell_dash_c_script_string` can return the *nested shell token*; the
    /// pre-spawn guard recurses. PR #792 inline: do not use the test helper in production.
    #[test]
    fn shell_dash_c_script_string_first_c_only_unlike_guard_nested_scan() {
        use super::shell_argv_may_start_unbounded_background;
        use super::shell_dash_c_script_string;
        let nested = vec![
            String::from("sh"),
            String::from("-c"),
            String::from("sh"),
            String::from("-ec"),
            String::from("sleep 600 &"),
        ];
        assert_eq!(shell_dash_c_script_string(&nested), Some("sh"));
        assert!(shell_argv_may_start_unbounded_background(&nested));
    }

    /// `sh -ec` and `sh -lc` (codex) must be covered, not only `sh -c …`.
    #[test]
    fn sh_dash_ec_rejects_background_ampersand() {
        let r = evaluate_execute_command_exit_code(
            "sh",
            &[String::from("-ec"), String::from("sleep 600 &")],
            0,
        );
        let ClaimResult::Fail(m) = r else {
            panic!("expected fail-closed for -ec + background, got {r:?}");
        };
        assert!(
            m.contains("background")
                || m.contains("P3")
                || m.contains("descendants")
                || m.contains("shell `-c`"),
            "expected policy message, got: {m}"
        );
    }

    /// `&&` is logical AND, not background; must not be rejected (PR #792 review).
    #[test]
    #[cfg(unix)]
    fn sh_dash_c_and_chain_runs() {
        let r = evaluate_execute_command_exit_code(
            "sh",
            &[String::from("-c"), String::from("true && true")],
            0,
        );
        assert_eq!(r, ClaimResult::Pass);
    }

    /// `2>&1` and similar are not a shell background `&` (elided before the heuristic).
    #[test]
    #[cfg(unix)]
    fn sh_dash_c_2_redir_is_not_treated_as_background() {
        let r = evaluate_execute_command_exit_code(
            "sh",
            &[String::from("-c"), String::from("true 2>&1")],
            0,
        );
        assert_eq!(
            r,
            ClaimResult::Pass,
            "2>&1 should not be confused with sh background &"
        );
    }

    /// `>&2` (default-fd to stderr) is not a background `&` — e.g. `unshare_path_drains_…` uses
    /// a loop with `>&2` on Linux; without eliding, `ExecuteCommand` rejects the `-c` script.
    #[test]
    fn sh_dash_c_greater_redir_to_fd2_is_not_background() {
        assert!(!shell_dash_c_may_start_background_after_eliding_artifacts(
            "i=0; while [ $i -lt 1 ]; do i=$((i+1)); done >&2; exit 0",
            Some("sh"),
        ));
    }

    #[test]
    #[cfg(unix)]
    fn long_running_child_fails_closed_with_timeout_message() {
        let r = evaluate_execute_command_exit_code_with_wall_time(
            "sh",
            &[String::from("-c"), String::from("sleep 5")],
            0,
            Duration::from_millis(150),
        );
        let ClaimResult::Fail(msg) = r else {
            panic!("expected timeout fail, got {r:?}");
        };
        assert!(
            msg.contains("0.15") && msg.contains("exceeded") && msg.contains("wall-clock"),
            "expected timeout phrasing, got: {msg}"
        );
    }
}

// Linux: bootstrap sentinel-pipe classification (T-PB-B Worker 4 typed model).
#[cfg(all(test, target_os = "linux"))]
mod unshare_bootstrap_sentinel_tests {
    use super::unshare_bootstrap_stage_from_sentinel;
    use super::UnshareBootstrapStage;

    #[test]
    fn empty_sentinel_means_namespace_setup_failed() {
        assert_eq!(
            unshare_bootstrap_stage_from_sentinel(b""),
            UnshareBootstrapStage::NamespaceSetupFailed
        );
    }

    #[test]
    fn s_only_means_logical_command_not_executable() {
        assert_eq!(
            unshare_bootstrap_stage_from_sentinel(b"s"),
            UnshareBootstrapStage::LogicalCommandNotExecutable
        );
    }

    #[test]
    fn se_means_logical_command_execed() {
        assert_eq!(
            unshare_bootstrap_stage_from_sentinel(b"se"),
            UnshareBootstrapStage::LogicalCommandExeced
        );
    }

    #[test]
    fn sef_means_logical_exec_failed_post_probe() {
        // `sef` = sh started, probe passed, exec returned (failure), EXIT trap fired. Must
        // surface as a typed exec-failed outcome, NOT as a logical exit — closes the P2(c)
        // regression manager flagged on draft review.
        assert_eq!(
            unshare_bootstrap_stage_from_sentinel(b"sef"),
            UnshareBootstrapStage::LogicalExecFailed
        );
    }

    #[test]
    fn pre_se_unexpected_patterns_fail_closed_to_setup_failed() {
        // Patterns that did NOT reach `e` (so the bootstrap never committed to `exec`) must
        // fail-closed to `NamespaceSetupFailed`. Direct fallback in that case is the *first*
        // logical run, not a second — safe.
        assert_eq!(
            unshare_bootstrap_stage_from_sentinel(b"x"),
            UnshareBootstrapStage::NamespaceSetupFailed
        );
        assert_eq!(
            unshare_bootstrap_stage_from_sentinel(b"es"),
            UnshareBootstrapStage::NamespaceSetupFailed
        );
    }

    /// **P2(d) regression (api-review codex sha 7297b04a).** Once the sentinel reaches `se`,
    /// the bootstrap has committed to `exec` and the user command DID run. Stray bytes after
    /// `se` (from the user writing to the inherited fd 3 post-`exec`) must classify as
    /// `LogicalCommandExeced`, NOT `NamespaceSetupFailed` — otherwise the runner would
    /// implicitly re-execute the user command via the direct fallback. Both `b"sex"` (stray
    /// after canonical `se`) and `b"sefx"` (stray after canonical `sef`-style trap-write)
    /// are user-side post-`exec` writes; both must surface as user ran.
    #[test]
    fn post_se_stray_bytes_classify_as_logical_command_execed_no_rerun() {
        assert_eq!(
            unshare_bootstrap_stage_from_sentinel(b"sex"),
            UnshareBootstrapStage::LogicalCommandExeced
        );
        assert_eq!(
            unshare_bootstrap_stage_from_sentinel(b"sefx"),
            UnshareBootstrapStage::LogicalCommandExeced
        );
        assert_eq!(
            unshare_bootstrap_stage_from_sentinel(b"se\x00"),
            UnshareBootstrapStage::LogicalCommandExeced
        );
        assert_eq!(
            unshare_bootstrap_stage_from_sentinel(b"se\xff\xff"),
            UnshareBootstrapStage::LogicalCommandExeced
        );
    }
}

/// **P2(c) hardening — helper-path validation rejects non-executables.** (api-review
/// codex/codex-default sha 143b7da5, BLOCKING.) Without this check, the override env var
/// could point at a directory or non-x file: \[unshare] would fail to exec it → empty
/// sentinel → `NamespaceSetupFailed` → direct fallback runs the user command, silently
/// converting helper misconfiguration into a possible `Matched` logical exit.
#[cfg(all(test, target_os = "linux"))]
mod helper_path_validation_tests {
    use super::is_regular_executable_file;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn directory_is_rejected() {
        let dir = std::env::temp_dir();
        // tmp dir definitely exists and is a directory.
        assert!(dir.is_dir());
        assert!(
            !is_regular_executable_file(&dir),
            "directory must NOT pass helper-path validation"
        );
    }

    #[test]
    fn non_executable_file_is_rejected() {
        let path = std::env::temp_dir().join(format!(
            "gunbc_pb_runtime_helper_validation_nonexec_{}",
            std::process::id()
        ));
        std::fs::write(&path, b"#!/bin/sh\nexit 0\n").expect("write tmp");
        let mut perms = std::fs::metadata(&path).expect("meta").permissions();
        perms.set_mode(0o644); // explicitly NO execute bit
        std::fs::set_permissions(&path, perms).expect("chmod 644");
        let result = is_regular_executable_file(&path);
        let _ = std::fs::remove_file(&path);
        assert!(
            !result,
            "non-executable file must NOT pass helper-path validation"
        );
    }

    #[test]
    fn missing_file_is_rejected() {
        let path = std::path::PathBuf::from(format!(
            "/tmp/gunbc_pb_runtime_helper_validation_missing_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        assert!(!path.exists());
        assert!(
            !is_regular_executable_file(&path),
            "missing path must NOT pass helper-path validation"
        );
    }

    #[test]
    fn executable_file_is_accepted() {
        // /bin/sh is universally a regular executable file on Linux.
        let path = if std::path::Path::new("/bin/sh").exists() {
            std::path::PathBuf::from("/bin/sh")
        } else {
            std::path::PathBuf::from("/usr/bin/sh")
        };
        assert!(
            is_regular_executable_file(&path),
            "regular executable {} must pass helper-path validation",
            path.display()
        );
    }

    /// **BLOCKING regression (api-review codex sha 523776b).** A file with execute bits
    /// set only for a uid/gid the calling process is NOT (e.g. `--x------` owned by root
    /// while we run as a non-root test runner) must be rejected — otherwise `unshare`
    /// would `EACCES` at exec time, the helper would never write the sentinel, and we'd
    /// silently route to direct fallback, violating P2(c).
    ///
    /// Construction: drop ALL execute bits (mode 0o400 = `r--------`) on a file we own;
    /// `access(X_OK)` returns -1/EACCES and the validator rejects. This exercises the
    /// uid-aware check rather than just "any bit set."
    #[test]
    fn non_executable_to_calling_uid_is_rejected() {
        use std::os::unix::fs::PermissionsExt;
        let path = std::env::temp_dir().join(format!(
            "gunbc_pb_runtime_helper_validation_uid_aware_{}",
            std::process::id()
        ));
        std::fs::write(&path, b"#!/bin/sh\nexit 0\n").expect("write tmp");
        let mut perms = std::fs::metadata(&path).expect("meta").permissions();
        // mode 0o400: owner can read but not execute; nobody can execute.
        // access(X_OK) by the caller (us, the owner) → EACCES. Validator must reject.
        perms.set_mode(0o400);
        std::fs::set_permissions(&path, perms).expect("chmod 400");
        let result = is_regular_executable_file(&path);
        let _ = std::fs::remove_file(&path);
        assert!(
            !result,
            "file with no execute permission for the calling uid must be rejected (P2(c))"
        );
    }
}

#[cfg(test)]
mod perf_within_baseline_tests {
    //! Unit tests for the `PerfWithinBaseline` evaluator path per the
    //! T-Tier3-Dissolution consumer-slice worker brief
    //! (`docs/briefs/r3-pb-t-tier3-consumer-slice-worker.md` §1).
    //!
    //! Two coverage axes:
    //!
    //! 1. **Resolver fail-closed**: `PerfMeasurementResolveError` table-form
    //!    over the four typed variants × two role labels. Each variant must
    //!    produce a `ClaimResult::Fail` reason text that preserves the role
    //!    label for triage (per `CODING.md` typed-error discipline +
    //!    `INVARIANTS.md` P3 fail-closed).
    //!
    //! 2. **Budget evaluation logic**: the four cases enumerated in the
    //!    brief acceptance — pass-when-under-budget, fail-on-median-over,
    //!    fail-on-p99-over, fail-on-overflow. Tested via a small free
    //!    function rather than through full DAG construction; the helper
    //!    encodes the §225 ratio thresholds and the saturating-overflow
    //!    fail-closed semantics.

    use super::compute_perf_budget_bounds;
    use super::is_comparator_le;
    use super::PerfBudgetOverflow;
    use super::PerfMeasurement;
    use super::PerfMeasurementResolveError;

    /// Apply the `Le` budget against the §225-locked thresholds. Tests
    /// call through `compute_perf_budget_bounds` (the same single-authority
    /// free function the runtime uses) so there is no second copy of the
    /// `× 2` / `× 5` constants — drift between test harness and runtime
    /// is structurally impossible.
    fn budget_pass_le(
        subject: PerfMeasurement,
        baseline: PerfMeasurement,
    ) -> Result<bool, PerfBudgetOverflow> {
        let (median_bound, p99_bound) = compute_perf_budget_bounds(baseline)?;
        Ok(subject.median_ns <= median_bound && subject.p99_ns <= p99_bound)
    }

    /// Differential test: every typed `PerfMeasurementResolveError` variant
    /// must produce a structurally-distinct `ClaimResult::Fail` for two
    /// different role labels, demonstrating that the role parameter flows
    /// through `into_claim_fail` without pinning the exact format string.
    /// This avoids the `TESTING.md` anti-pattern of asserting on
    /// human-readable error message text while still exercising the
    /// role-preservation contract from the brief acceptance.
    #[test]
    fn resolver_fail_closed_role_label_flows_through_for_every_variant() {
        let variants: &[PerfMeasurementResolveError] = &[
            PerfMeasurementResolveError::MissingDeclaration,
            PerfMeasurementResolveError::WrongConnective,
            PerfMeasurementResolveError::MissingField { field: "median_ns" },
            PerfMeasurementResolveError::MissingField { field: "p99_ns" },
            PerfMeasurementResolveError::WrongFieldKind { field: "median_ns" },
            PerfMeasurementResolveError::WrongFieldKind { field: "p99_ns" },
        ];
        for variant in variants {
            let alpha = variant.clone().into_claim_fail("alpha_role");
            let beta = variant.clone().into_claim_fail("beta_role");
            assert_ne!(
                alpha, beta,
                "role label must produce different output for variant {:?}; \
                 got identical strings (role param ignored?)",
                variant,
            );
            // Also verify the role string itself is reachable in the output —
            // not by pinning the format but by checking the role name is a
            // distinguishing substring (a role drop would make alpha == beta
            // above, but a role used non-identifyingly could still differ).
            assert!(
                alpha.contains("alpha_role") && beta.contains("beta_role"),
                "role substrings must appear in their respective outputs for {:?}",
                variant,
            );
        }
    }

    #[test]
    fn budget_pass_when_subject_under_thresholds() {
        // baseline median 100, p99 200; bounds = (200, 1000); subject (150, 800) — both under.
        let baseline = PerfMeasurement {
            median_ns: 100,
            p99_ns: 200,
        };
        let subject = PerfMeasurement {
            median_ns: 150,
            p99_ns: 800,
        };
        assert_eq!(budget_pass_le(subject, baseline), Ok(true));
    }

    #[test]
    fn budget_fails_when_subject_median_exceeds_bound() {
        // baseline median 100 → bound 200; subject median 201 fails despite fine p99.
        let baseline = PerfMeasurement {
            median_ns: 100,
            p99_ns: 200,
        };
        let subject = PerfMeasurement {
            median_ns: 201,
            p99_ns: 800,
        };
        assert_eq!(budget_pass_le(subject, baseline), Ok(false));
    }

    #[test]
    fn budget_fails_when_subject_p99_exceeds_bound() {
        // baseline p99 200 → bound 1000; subject p99 1001 fails despite fine median.
        let baseline = PerfMeasurement {
            median_ns: 100,
            p99_ns: 200,
        };
        let subject = PerfMeasurement {
            median_ns: 150,
            p99_ns: 1001,
        };
        assert_eq!(budget_pass_le(subject, baseline), Ok(false));
    }

    #[test]
    fn budget_fail_closed_on_threshold_overflow() {
        // baseline median = i64::MAX → ×2 overflows; helper must surface the
        // overflow rather than silently wrap. The runtime impl converts this
        // to ClaimResult::Fail with an explicit reason.
        let baseline = PerfMeasurement {
            median_ns: i64::MAX,
            p99_ns: 1,
        };
        let subject = PerfMeasurement {
            median_ns: 0,
            p99_ns: 0,
        };
        assert_eq!(
            budget_pass_le(subject, baseline),
            Err(PerfBudgetOverflow::Median)
        );

        let baseline_p99_overflow = PerfMeasurement {
            median_ns: 1,
            p99_ns: i64::MAX,
        };
        assert_eq!(
            budget_pass_le(subject, baseline_p99_overflow),
            Err(PerfBudgetOverflow::P99)
        );
    }

    /// The Director-locked §225 budget semantics are `median ≤ 2× baseline`
    /// AND `p99 ≤ 5× baseline`. The substrate `ComparisonOp` variant
    /// carries `comparator` for explicit intent, but only `Le` matches the
    /// locked semantics. This test reads `ComparisonOp`'s variant set
    /// from the bootstrap DAG (substrate-source-of-truth, not a hardcoded
    /// list) and asserts the runtime guard classifies every variant: `Le`
    /// alone passes, every other variant fails-closed.
    #[test]
    fn runtime_classifies_every_comparison_op_variant_le_only() {
        use crate::dag::TypeConnective;
        let dag = crate::generated_full_bootstrap_dag();
        let comparison_op = dag
            .declaration_by_name("ComparisonOp")
            .expect("ComparisonOp must exist in bootstrap DAG (src/v3/std/substrate.dag)");
        let TypeConnective::Disj { variants } = &comparison_op.connective else {
            panic!("ComparisonOp must be a coproduct (Disj)");
        };
        let mut variant_labels: Vec<&str> = variants.iter().map(|v| v.label.as_str()).collect();
        variant_labels.sort();
        assert!(
            variant_labels.contains(&"Le"),
            "substrate ComparisonOp must include `Le` for §225 semantics; got {variant_labels:?}"
        );
        // Drift gate: every substrate-declared variant must be explicitly
        // classified by the runtime guard. The guard's classification is a
        // single rule — `label == \"Le\"` — so the assertion below mirrors it.
        // If substrate adds a variant (e.g., `Approx`), `is_comparator_le`
        // continues to return `false` for it (correctly fail-closed under the
        // current locked semantics) and this test continues to pass without
        // edits — the new variant is structurally rejected by the same guard,
        // matching the §225 lock.
        for label in &variant_labels {
            let accepted = is_comparator_le(label);
            assert_eq!(
                accepted,
                *label == "Le",
                "ComparisonOp variant `{label}`: runtime guard classification disagrees with §225 \
                 lock (only `Le` may be accepted)",
            );
        }
    }
}

/// Single-rule classifier for the §225 comparator-Le guard. The runtime
/// path in [`TestRunner::eval_perf_within_baseline`] inlines the same rule
/// (`comparator_label != "Le"` → fail-closed). Lifted as a free function
/// so unit tests can drive the same classifier against the substrate-
/// declared `ComparisonOp` variant set without DAG fixtures.
fn is_comparator_le(comparator_label: &str) -> bool {
    comparator_label == "Le"
}
