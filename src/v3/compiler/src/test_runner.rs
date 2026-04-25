use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crate::dag::{
    Behavior, Dag, Declaration, DeclarationId, FieldValue, LiteralBits, Path, PortId, PortState,
    TypeConnective, ValueBody,
};
use crate::diagnostics::Diagnostic;
use crate::lens_apply::{
    apply_lens_declaration, field_value_from_value_body, int_associativity_holds_all_triples,
    reflect_program_dag_nodes_in_file, ASSOCIATIVITY_WITNESS_TRIPLES,
};
use crate::lens_cost::{cost_of, CostLookup};
use crate::{compile_to_dag, CompileError};

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

/// Bind whose `value` port receives structural `cost_of` for `LensOutputEquals` / `DifferentialEquals`.
///
/// Today the runner keys this on `TestClaim.file_name` until `DeclarationRef` can name the bind
/// directly (M1(2.8) — same story as `r1_lens_output_input_from_program`). **Process ratchet:** do
/// not extend this `match` without a linked issue toward that dissolution (api-review #764).
fn cost_bind_for_claim_file(file_name: &str) -> Option<&'static str> {
    match file_name {
        "r1_merge_sort_pair.v3" => Some("merge_sort_out"),
        "r1_lane_e_differential_witness.v3" => Some("lane_e_diff_out"),
        "fixture_compiler_nerd_canonical_complexity.v3" => Some("complexity_demo_out"),
        "fixture_compiler_nerd_canonical_parallelism.v3" => Some("total"),
        _ => None,
    }
}

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
/// **`Associativity` — bounded operational witness (T-LensAPI D3), not substrate law proof:**
/// uses [`int_associativity_holds_all_triples`](crate::lens_apply::int_associativity_holds_all_triples)
/// over [`ASSOCIATIVITY_WITNESS_TRIPLES`](crate::lens_apply::ASSOCIATIVITY_WITNESS_TRIPLES) so a
/// single lucky `(a,b,c)` cannot certify a false law. This path does **not** consume quantified
/// associativity facts declared on `OrderedRing` / semigroup
/// carriers in `std.algebra` (those are not yet first-class runner inputs). Treating `Pass` here
/// as full algebraic law evidence would be weaker than a substrate-backed law check — the R1
/// gate is intentionally a **regression harness** that the witness lens behaves associatively on
/// the full witness set, not a proof for all `Int`. **Dissolution:** wire `AlgebraicLaw` to declared law
/// metadata / witnesses on disk and reserve sample-only checks to explicit testgen predicates, or
/// return [`ClaimResult::NotYetImplemented`] until that substrate surface exists.
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
    if law_label != "Associativity" {
        return Err(AlgebraicLawProgramError::UnsupportedLaw { law_label });
    }
    if !law_payload.is_empty() {
        return Err(AlgebraicLawProgramError::MalformedPayload(
            "Associativity should be payload-free".to_string(),
        ));
    }
    let lens_name = declaration_ref_name(fixture_dag, lens_ref)?;
    let Some(target) = program_dag.declaration_by_name(&lens_name) else {
        return Ok(false);
    };
    int_associativity_holds_all_triples(program_dag, target.id, ASSOCIATIVITY_WITNESS_TRIPLES)
        .map_err(|e| AlgebraicLawProgramError::MalformedPayload(format!("lens apply error: {e:?}")))
}

/// Compile-time ratchet (PR #741 / codex P1): `Associativity` must not regress to checking one
/// lucky `(a, b, c)` triple — the gate is a correctness signal only when the witness set has
/// material breadth (see `lens_apply::ASSOCIATIVITY_WITNESS_TRIPLES`).
const _: () = assert!(ASSOCIATIVITY_WITNESS_TRIPLES.len() > 1);

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
fn shell_interpreter_for_c_flag(
    args: &[String],
    c_flag_index: usize,
    leading_hint: Option<&str>,
) -> Option<&str> {
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

/// On Linux, wrap the logical `command` + `args` in a **user + PID namespace** (util-linux
/// `unshare(1)` with `-c` = map current user, `-f` fork, `-p` new PID namespace) so the first exec’d
/// process in the new namespace is PID 1 (init rôle for that namespace): when it exits, the
/// kernel tears down the contained subtree, closing the “direct child matched exit, grandchildren
/// still run” host escape **for this path** (Codex/PR #792: P3/P4 on the `ExecuteCommand`
/// boundary). Other Unix and Windows: no unprivileged one-shot equivalent; wall bound + pgrp
/// signal on timeout + the `sh -c` `&` heuristic only—documented, not a full process-tree
/// guarantee.
/// If `unshare(1)` **spawn** returns `Err(…)` for **any** reason, *or* a **started** `unshare(1)`
/// process prints util-linux `unshare:` lines on the captured **wrapper** stderr (namespace setup
/// before `exec` to the logical `command` — e.g. `Operation not permitted` / `EPERM` / missing
/// util-linux), the runner **falls back** to [`build_execute_command_process`] (direct `Child`);
/// see [`unshare_sandbox_broken_relaunch_with_direct`] for post-start retry. Wall+null stdio+pgrp
/// still hold on the direct path; the PID-namespace **init**-style subtree teardown is skipped.
///
/// The unshare path runs `unshare … -- sh -c` `UNSHARE_LOGICAL_BOOTSTRAP_SH` (on Linux: see
/// that constant in this file), with `command`+`args` as argv (POSIX `$0`/`$@` — not
/// shell-interpolated), so the **logical** process is `exec`’d with `stderr` → `/dev/null` while
/// the parent’s piped `stderr` remains for util-linux and bootstrap output only (codex PR #792: split
/// setup authority from the claim).
//
// **Not the same stream:** The logical child does **not** use the `Stdio::piped` read end for its
// stderr. `UNSHARE_LOGICAL_BOOTSTRAP_SH` runs `exec 2>/dev/null` **before** `exec` of
// `command`+`args`, so the final process replaces the bootstrap with inherited fd2=`/dev/null` — not
// the wrapper pipe. Only util-linux / pre-reexec `sh` may write to that pipe. If the logical
// process still shared the pipe, an exit-only claim that prints heavily to `stderr` would fill
// the pipe and stall; **receipt:** Linux unit
// `unshare_path_drains_piped_stderr_so_huge_logical_stderr_does_not_stall` (large `>&2` loop) passes
// in CI (PR #792, inline review 2026-04-25).
#[cfg(target_os = "linux")]
const UNSHARE_LOGICAL_BOOTSTRAP_SH: &str = "exec 2>/dev/null; exec \"$0\" \"$@\"";

#[cfg(target_os = "linux")]
fn build_execute_command_unshare(command: &str, args: &[String]) -> Command {
    let mut c = Command::new("unshare");
    c.args([
        "-c",
        "-f",
        "-p",
        "--",
        "sh",
        "-c",
        UNSHARE_LOGICAL_BOOTSTRAP_SH,
    ])
    .arg(command)
    .args(args)
    .stdin(Stdio::null())
    .stdout(Stdio::null())
    // stderr: capture for util-linux + **pre-`exec`** child setup lines only; the **logical**
    // process is re-exec’d with `stderr` → `/dev/null` (see
    // [`UNSHARE_LOGICAL_BOOTSTRAP_SH`]) so exit-code-only claims are not entangled with logical
    // `stderr` volume, pipe fill, or heuristics. [`child_wait_for_execute_command`] still
    // **drains** this pipe during `try_wait` (O_NONBLOCK) in case a util-linux or bootstrap
    // `sh` path is unexpectedly chatty.
    .stderr(Stdio::piped());
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

/// After `unshare(1)` exits, scan its stderr. If util-linux reported namespace setup failure (a
/// common path is: process started, then `unshare(2)` / clone fails, **before** `exec` to the
/// logical `command`), the exit code is **not** the program’s. **Retry once** with a direct
/// [`build_execute_command_process`] (same as the Linux `unshare(1)` **spawn** fallback) so
/// P3 is satisfied without turning every `ExecuteCommand("true", …, 0)` into `Fail` on restricted
/// Linux CI. If we cannot read stderr, retry direct (conservative: avoid conflating ambiguous
/// unshare exit with `expect_exit_code`).
///
/// util-linux prefixes *all* wrapper diagnostics with `unshare:` — including messages that do not
/// contain the word `failed` (e.g. `unshare: Operation not permitted`). Treat any such line in the
/// first 20 lines as a setup error so we don’t conflate a permission/setup failure with the
/// logical program’s exit (PR #792).
///
/// **C-5 / P3:** This scan is only applied when
/// [`unshare_post_start_stderr_may_authorize_relaunch`] is true. The logical `command` (after the
/// Linux `UNSHARE_LOGICAL_BOOTSTRAP_SH` hop) has `stderr` to `/dev/null`, so a matching host exit
/// from the logical `command` is not confusable with a post-hoc `unshare:` line from a chatty
/// *user* program on the same capture (that source of false authority is gone; util-linux
/// heuristics remain the concern).
///
/// **P3 (empty buffer):** A second run when merged wrapper stderr is **empty** after an exit
/// **mismatch** is **enabled** in `#[cfg(test)]` builds of this crate, or when
/// `GUNBC_EXECUTE_COMMAND_UNSHARE_EMPTY_STDERR_RELAUNCH` is truthy (`1` or `true`); `cargo
/// build` with that env can match `cargo test` for this *one* bounded retry. Without both, fail-closed:
/// the first unshare(1) exit is authority. (Second-run-on-empty can still flip non-idempotent
/// work; keep test claims to idempotent or narrow commands — PR #792.) `unshare:`-pattern, read
/// errors, `take` failure, and non-zero host-confirmation are unchanged; the latter is
/// independent of this buffer case.
///
/// **P5 (dissolution — shared target):** Retire (1) the empty-stderr test/env relaunch and
/// (2) the **non-zero exit** direct-`Child` “host confirmation” re-exec in
/// `evaluate_execute_command_host_outcome` when unshare(1) / namespace setup can be **typed** as
/// “setup did not reach logical `exec`” (or as a distinct setup failure) **without** a second
/// `Child` — e.g. namespace or util-linux state on a **separate** fd, `pidfd`/ns inspection, or a
/// different sandbox primitive — or when the **hosted** Linux pool no longer hits the PID-1 /
/// empty-fd / spurious-exit quirk (kernel/cap/namespace policy that makes the first `wait` match
/// the direct `exec` result). Both branches are bounded policy today; one retirement hook.
#[cfg(target_os = "linux")]
const UNSHARE_STDERR_SCAN_CAP: u64 = 8 * 1024;

/// Set to `1` (or `true`, case-insensitive) so the unshare post-start **empty-wrapper-stderr** retry
/// (exit mismatch) runs in `cargo build` as well as `cargo test` — T-PB-B, explicit data over
/// build-shape-only gating. See [`unshare_sandbox_broken_relaunch_with_direct`].
#[cfg(target_os = "linux")]
const GUNBC_EXECUTE_COMMAND_UNSHARE_EMPTY_STDERR_RELAUNCH: &str =
    "GUNBC_EXECUTE_COMMAND_UNSHARE_EMPTY_STDERR_RELAUNCH";

/// Second direct `Child` in [`evaluate_execute_command_host_outcome`] (single retry each, P2(d) seam).
#[cfg(target_os = "linux")]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum UnshareDirectRerun {
    /// util-linux / wrapper: piped `stderr` read error, or `unshare:` in merged capture.
    PostStartFallback,
    /// exit mismatch, merged empty, `#[cfg(test)]` **or** this env; see
    /// [`GUNBC_EXECUTE_COMMAND_UNSHARE_EMPTY_STDERR_RELAUNCH`].
    ExitMismatchEmptyWrapperStderr,
    /// non-zero `expect` + code match: one host-confirmation `Child` (unshare PID-1 / namespace
    /// quirk). **Non-idempotent** workloads can differ across the two runs — the highest-risk of the
    /// three re-execs; same P5 retirement as post-start + empty-stderr (Claude e99b53e7).
    NonzeroHostConfirm,
}

#[cfg(target_os = "linux")]
fn unshare_reexec_after_spawn_error_label(reason: UnshareDirectRerun) -> &'static str {
    match reason {
        UnshareDirectRerun::PostStartFallback
        | UnshareDirectRerun::ExitMismatchEmptyWrapperStderr => "unshare(1) post-start fallback",
        UnshareDirectRerun::NonzeroHostConfirm => "unshare(1) host-exit confirmation",
    }
}

/// [`cfg!(test)`](https://doc.rust-lang.org/std/macro.cfg.html) **or** truthy
/// `GUNBC_EXECUTE_COMMAND_UNSHARE_EMPTY_STDERR_RELAUNCH` (see
/// [`GUNBC_EXECUTE_COMMAND_UNSHARE_EMPTY_STDERR_RELAUNCH`]) authorizes the empty-stderr
/// second run on the unshare(1) path; default release builds are fail-closed without the env.
#[cfg(target_os = "linux")]
fn unshare_empty_stderr_relaunch_authorized() -> bool {
    cfg!(test)
        || std::env::var(GUNBC_EXECUTE_COMMAND_UNSHARE_EMPTY_STDERR_RELAUNCH)
            .ok()
            .is_some_and(|s| {
                let t = s.trim();
                t == "1" || t.eq_ignore_ascii_case("true")
            })
}

/// `unshare(1)` (and the thin `sh` bootstrap) may still write to the **parent** pipe before the
/// logical `command` is `exec`’d with `stderr` off the pipe. Only when the observed **exit code
/// already disagrees** with the claim may we treat the capture as a possible *wrapper* setup error
/// and scan for `unshare:` to authorize a single direct-`Child` re-run.
#[cfg(target_os = "linux")]
fn unshare_post_start_stderr_may_authorize_relaunch(
    from_unshare: bool,
    status: &std::process::ExitStatus,
    expect_exit_code: i64,
) -> bool {
    from_unshare && status.code().map(i64::from) != Some(expect_exit_code)
}

/// Returns true if captured stderr from the `unshare(1)` / wrapper side of the process looks like
/// util-linux’s own error output (as opposed to logical-program noise — with
/// `UNSHARE_LOGICAL_BOOTSTRAP_SH` the **logical** process does not use this pipe, but heuristics
/// must stay prefix-based).
#[cfg(target_os = "linux")]
fn unshare_stderr_indicates_sandbox_setup_failure(stderr_text: &str) -> bool {
    for line in stderr_text.lines().take(20) {
        if line.trim().starts_with("unshare:") {
            return true;
        }
    }
    false
}

/// Merge bytes captured while waiting (nonblocking pipe drain) with a post-`try_wait` read, using
/// a single `UNSHARE_STDERR_SCAN_CAP` budget (codex PR #792: `unshare:` in the pre-exit window must
/// not be lost before the setup-failure scan).
#[cfg(target_os = "linux")]
fn unshare_merge_stderr_for_setup_scan(pre_wait: &[u8], post_read: &str) -> String {
    let pre_len = (pre_wait.len() as u64).min(UNSHARE_STDERR_SCAN_CAP) as usize;
    let pre = &pre_wait[..pre_len];
    let mut s = String::new();
    s.push_str(&String::from_utf8_lossy(pre));
    s.push_str(post_read);
    s
}

/// Merges the wait-time drain with a blocking read of the rest of the wrapper’s piped `stderr` (8
/// KiB total cap). Clears `child.stderr` if present. `Err(())` = no read handle, read error, or empty
/// handle; callers treat that conservatively (e.g. relaunch / confirm).
#[cfg(target_os = "linux")]
fn unshare_merged_wrapper_stderr_read(
    child: &mut std::process::Child,
    pre_wait: &[u8],
) -> Result<String, ()> {
    use std::io::Read;
    let Some(h) = child.stderr.take() else {
        return Err(());
    };
    let pre_len = (pre_wait.len() as u64).min(UNSHARE_STDERR_SCAN_CAP) as usize;
    let take_remain = UNSHARE_STDERR_SCAN_CAP.saturating_sub(pre_len as u64);
    let mut buf = String::new();
    if h.take(take_remain).read_to_string(&mut buf).is_err() {
        return Err(());
    }
    Ok(unshare_merge_stderr_for_setup_scan(pre_wait, &buf))
}

#[cfg(target_os = "linux")]
fn unshare_sandbox_broken_relaunch_with_direct(
    from_unshare: bool,
    child: &mut std::process::Child,
    pre_wait_drain: &[u8],
) -> Option<UnshareDirectRerun> {
    if !from_unshare {
        return None;
    }
    let combined = match unshare_merged_wrapper_stderr_read(child, pre_wait_drain) {
        Ok(s) => s,
        // Cannot read wrapper stderr: unshare(1) exit may still be a setup artifact — retry direct.
        Err(()) => {
            return Some(UnshareDirectRerun::PostStartFallback);
        }
    };
    if unshare_stderr_indicates_sandbox_setup_failure(&combined) {
        return Some(UnshareDirectRerun::PostStartFallback);
    }
    // TODO(dissolution, P5): see module doc **P5 (shared target)** — same retirement as non-zero
    // host-confirmation; drop `GUNBC_EXECUTE_COMMAND_UNSHARE_EMPTY_STDERR_RELAUNCH` + `cfg!(test)` when
    // that lands.
    if combined.trim().is_empty() && unshare_empty_stderr_relaunch_authorized() {
        // Piped stderr, read ok, no `unshare:`, empty merge: second run in test or with env.
        return Some(UnshareDirectRerun::ExitMismatchEmptyWrapperStderr);
    }
    None
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

/// Linux + `unshare(1)`: the parent holds a read end to a **piped** stderr for the **wrapper** side
/// (util-linux, thin `sh` bootstrap, and any output **before** the final `exec` to the logical
/// `command`). The logical process is re-`exec`’d with `stderr` → `/dev/null` (see
/// [`UNSHARE_LOGICAL_BOOTSTRAP_SH`]); that pipe does **not** carry the logical process’s
/// `stderr` after the hop. If we only `try_wait` and never read, the **wrapper** subtree can
/// still block when the ~64KiB default pipe buffer fills; we only fail at wall timeout.
/// [`linux_drain_piped_child_stderr_nonblocking_once`] keeps a bounded **prefix** (up to
/// `UNSHARE_STDERR_SCAN_CAP` bytes) for the `unshare:` post-wait scan, then discards the rest
/// in that round to avoid a filling pipe; bytes after the cap are not preserved for the scan
/// (same budget as a single pre-fix `read_to_string` take).
#[cfg(target_os = "linux")]
fn linux_piped_child_stderr_set_nonblock(
    fd: std::os::fd::RawFd,
    nonblock: bool,
) -> std::io::Result<()> {
    // Same pattern as `std` net/uds: F_GETFL / F_SETFL with O_NONBLOCK.
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    if flags < 0 {
        return Err(std::io::Error::last_os_error());
    }
    let new = if nonblock {
        flags | libc::O_NONBLOCK
    } else {
        flags & !libc::O_NONBLOCK
    };
    if unsafe { libc::fcntl(fd, libc::F_SETFL, new) } < 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}

/// Nonblocking drain of `ChildStderr`: prevents pipe stalls (PR #792). Appends into `capture` only
/// while `capture.len() < cap`; any further bytes in this round are discarded so the read end does
/// not back up. (Same authority bytes must be merged into the post-`try_wait` scan — codex: do not
/// `capture` pre-exit and drop before [`unshare_sandbox_broken_relaunch_with_direct`].)
#[cfg(target_os = "linux")]
fn linux_drain_piped_child_stderr_nonblocking_once(
    stderr: &mut std::process::ChildStderr,
    capture: &mut Vec<u8>,
    cap: usize,
) {
    use std::io::ErrorKind;
    use std::io::Read;
    let mut buf = [0u8; 8192];
    loop {
        let n = match stderr.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) if e.kind() == ErrorKind::WouldBlock => break,
            Err(e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(_) => break,
        };
        for &b in buf.iter().take(n) {
            if capture.len() < cap {
                capture.push(b);
            }
        }
    }
}

#[cfg(target_os = "linux")]
struct LinuxPipedChildStderrNonblockGuard {
    fd: std::os::fd::RawFd,
}

#[cfg(target_os = "linux")]
impl LinuxPipedChildStderrNonblockGuard {
    /// When `must_drain` and the `Child` has a piped stderr, set O_NONBLOCK for the read end. On
    /// drop, restore **blocking** so the post-wait `read_to_string` / `take()` path keeps its
    /// previous contract.
    fn try_new(
        must_drain: bool,
        child: &std::process::Child,
    ) -> Option<LinuxPipedChildStderrNonblockGuard> {
        if !must_drain {
            return None;
        }
        use std::os::unix::io::AsRawFd;
        let fd = child.stderr.as_ref()?.as_raw_fd();
        if linux_piped_child_stderr_set_nonblock(fd, true).is_err() {
            return None;
        }
        Some(LinuxPipedChildStderrNonblockGuard { fd })
    }
}

#[cfg(target_os = "linux")]
impl Drop for LinuxPipedChildStderrNonblockGuard {
    fn drop(&mut self) {
        let _ = linux_piped_child_stderr_set_nonblock(self.fd, false);
    }
}

/// On Linux when `must_drain_piped_child_stderr` is true (`unshare(1)` path), the wait loop
/// nonblocking-drains the piped child stderr so it cannot fill and stall the child (PR #792), and
/// (same cap as post-wait read) appends a bounded prefix to `unshare_stderr_drain` for the
/// [`unshare_stderr_indicates_sandbox_setup_failure`] scan (codex: do not drain away `unshare:`
/// lines that arrive before exit).
fn child_wait_for_execute_command(
    child: &mut std::process::Child,
    wall_time: Duration,
    #[cfg_attr(not(target_os = "linux"), allow(unused_variables))]
    must_drain_piped_child_stderr: bool,
    #[cfg(target_os = "linux")] unshare_stderr_drain: &mut Vec<u8>,
) -> Result<std::process::ExitStatus, ClaimResult> {
    #[cfg(target_os = "linux")]
    let _stderr_nonblock =
        LinuxPipedChildStderrNonblockGuard::try_new(must_drain_piped_child_stderr, child);
    // If we could not set nonblock, do not `read` in blocking mode (would block on an idle pipe).
    #[cfg(target_os = "linux")]
    let can_drain_nonblocking = _stderr_nonblock.is_some();

    let wall_label = format!("{:.2}", wall_time.as_secs_f64());
    let deadline = Instant::now() + wall_time;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Ok(status),
            Ok(None) => {
                #[cfg(target_os = "linux")]
                {
                    if must_drain_piped_child_stderr
                        && can_drain_nonblocking
                        && child.stderr.is_some()
                    {
                        if let Some(s) = child.stderr.as_mut() {
                            linux_drain_piped_child_stderr_nonblocking_once(
                                s,
                                unshare_stderr_drain,
                                UNSHARE_STDERR_SCAN_CAP as usize,
                            );
                        }
                    }
                }
                if Instant::now() >= deadline {
                    kill_process_group_on_timeout(child);
                    let _ = child.wait();
                    return Err(ClaimResult::Fail(format!(
                        "ExecuteCommand: process exceeded {wall_label}s wall-clock limit (timeout — process group / child killed, fail-closed)"
                    )));
                }
                std::thread::sleep(EXECUTE_COMMAND_WAIT_POLL);
            }
            Err(err) => {
                return Err(ClaimResult::Fail(format!(
                    "ExecuteCommand: wait on child failed: {err}"
                )));
            }
        }
    }
}

/// Spawns a host process and checks exit status. Used by the Rust `TestRunner` and the M1.5
/// harness (single canonical path per PB-Runtime brief). Core logic is
/// [`evaluate_execute_command_host_outcome`] ([`ExecuteCommandHostOutcome`]); this function and
/// [`evaluate_execute_command_exit_code_with_wall_time`] map to [`ClaimResult`] at the reporting
/// edge. [`evaluate_execute_command_m1_5`] classifies the typed outcome (C-5: no `Fail` string-prefix
/// probing for exit mismatch; codex PR #792).
///
/// - **No stdout/stderr capture** — `stdin`/`stdout`/`stderr` are the null device so malicious or
///   chatty children cannot exhaust memory; only the exit code is read (P3/P4: bounded, fail-closed
///   outcomes). This path does **not** use [`std::process::Command::output`]; it uses
///   [`std::process::Command::spawn`] and a wall-bounded `try_wait` loop (`child_wait_for_execute_command` in
///   this file).
/// - **Wall clock** — [`EXECUTE_COMMAND_WALL_TIMEOUT`]; on exceed, the process group is signalled
///   (Unix) and the result is a typed failure (not a hang).
/// - **Linux: user+PID namespace (when `unshare(1)` can be spawned)** — the usual path wraps in
///   `unshare(1) -c -f -p` plus a small POSIX `sh` bootstrap that `exec(2)`s
///   the user `command`+`args` with **logical** `stderr` to `/dev/null` so the parent’s piped
///   `stderr` is for util-linux and bootstrap `sh` only, not a shared authority with exit-code
///   semantics (PR #792 codex). On any `unshare(1)` **spawn** `Err(…)` the runner falls back to a
///   direct `Child` (util-linux not on `PATH`, `EPERM` at `execve`, etc.); after a **start**,
///   (only when the observed exit does **not** already match the claim) wrapper `stderr` may be
///   scanned for namespace **setup** failure, then the runner **re-runs once** with a direct
///   [`Command`] (see [`unshare_sandbox_broken_relaunch_with_direct`]) so setup failure never
///   masquerades as a matching `expect_exit_code` and portable claims still `Pass` in restricted
///   CI. The `try_wait` loop also **drains** the pipe in nonblocking mode in case a wrapper path is
///   unexpectedly chatty: a **bounded** prefix of bytes is **retained** for the same
///   `unshare_stderr_indicates_sandbox_setup_failure` scan, then overflow in that read round
///   is discarded to avoid a filling pipe. A matching exit on the **logical**
///   child is not re-run for a `unshare:`-shaped line in this capture (C-5, given logical stderr is
///   not on the pipe). Wall+stdio+pgrp still apply. Empty wrapper stderr after an exit **mismatch**
///   is retried (direct) in `#[cfg(test)]` **or** with `GUNBC_EXECUTE_COMMAND_UNSHARE_EMPTY_STDERR_RELAUNCH` —
///   see [`unshare_sandbox_broken_relaunch_with_direct`].
/// - **Heuristic on `&` in `sh`/`bash`/… `-c` scripts (all hosts, including `sh -ec` / `sh -lc`)** — a
///   bare shell background `&`
///   (after eliding `&&` and a few `>&` / `&>`-style token spellings) is still rejected: cheap extra
///   catch for the obvious `sh` escape. Not a full `sh` parser. Non-Linux: no init-style subtree
///   guarantee; Director-level full sandbox is a separate policy track.
///
/// Fail messages distinguish spawn error, policy reject, timeout, missing exit code (signal), and
/// exit-code mismatch.
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

/// String form for the exit-mismatch **reporting** edge; **not** used for M1.5 or other semantic
/// classification (C-5, codex PR #792) — that uses [`ExecuteCommandHostOutcome::Mismatch`].
pub const EXECUTE_COMMAND_EXIT_CODE_MISMATCH_MSG_PREFIX: &str =
    "ExecuteCommand exit code mismatch: expected ";

/// [`evaluate_execute_command_host_outcome`]: single authority **before** [`ClaimResult`]
/// rendering — M1.5 and other consumers classify exit mismatch here as data, not
/// `Fail(String)`-prefix probes (P2 [Host-process (a)](/INVARIANTS.md#p2-host-process-boundary), C-5,
/// DB-1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecuteCommandHostOutcome {
    /// Host exit code equaled the claim.
    Matched,
    /// Host exit code was observed and did not match the claim; M1.5 maps this alone to
    /// propositional “false” for the exit predicate.
    Mismatch { expected: i64, actual: i64 },
    /// All other `ClaimResult` needs (always `Fail` or `NotYetImplemented` in practice) —
    /// timeout, policy, signal, spawn error, etc.
    ///
    /// **TODO(dissolution, T-PB-B, C-5):** if call sites need to *branch* on host-failure *kind* without
    /// string authority, expand this into a dedicated `enum` (e.g. timeout / spawn / policy) and
    /// reserve [`ClaimResult`] for the final reported edge only — the current `Other` is a **partial**
    /// carrier (PR #792; api-review 994fa40d). Until then, do **not** add consumers that
    /// pattern-match on `Other(Fail(_))` text; use [`evaluate_execute_command_m1_5`] (or this full
    /// `enum` without string probes) for classification (api-review 837d0e59). Temptation to
    /// string-probe `Other` in new code is a future C-5 / DB-1 foot-gun (e99b53e7).
    Other(ClaimResult),
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
            ExecuteCommandHostOutcome::Other(c) => c,
        }
    }
}

/// M1.5 and other **boolean** predicate reads: only these outcomes map to propositional
/// true/false; all other results are `Err(ClaimResult)` (not “`false`” for the claim).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecuteCommandM1_5Proposition {
    /// Observed exit satisfied `expect_exit_code`.
    Satisfied,
    /// The process completed with a host exit code that did not match the claim.
    UnsatisfiedExitMismatch,
}

/// Distinguish “exit ≠ expect” from timeout, spawn error, `&` policy, signal, etc. `Err` is the
/// full untyped `ClaimResult` (use [`TestRunner`] for strings); do not map `Err` to
/// propositional `false` (P3/DB-1; codex PR #792). This is the supported boolean exit predicate
/// path while [`ExecuteCommandHostOutcome::Other`] remains a **partial** carrier (837d0e59).
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
        ExecuteCommandHostOutcome::Other(c) => Err(c),
    }
}

/// Core host run: **typed** outcome. Map to [`ClaimResult`] with
/// [`ExecuteCommandHostOutcome::into_claim_result`] for [`TestRunner`], or match directly from
/// [`evaluate_execute_command_m1_5`].
///
/// **P2(d) (implicit re-execution):** On Linux, the `loop` may spawn a **second** `Child` once
/// for (1) unshare post-start setup failure, (2) non-zero-`expect_exit_code` with a matching
/// unshare(1) exit (always one direct-`Child` host confirmation; fail-closed: empty / no-`unshare:`
/// merge is not proof of logical `exec`), or (3) empty piped wrapper stderr on exit **mismatch**
/// with `unshare_empty_stderr_relaunch_authorized`. The authority is
/// `UnshareDirectRerun`; all single-retry, documented in [`unshare_sandbox_broken_relaunch_with_direct`]
/// and the **P3 (empty buffer)** / **P5 (dissolution — shared target)** block. The P5 “typed setup
/// carrier” still out-of-tree dissolves the string/`Child` heuristics together.
pub fn evaluate_execute_command_host_outcome(
    command: &str,
    args: &[String],
    expect_exit_code: i64,
    wall_time: Duration,
) -> ExecuteCommandHostOutcome {
    if let Some(r) = reject_unbounded_shell_background(command, args) {
        return ExecuteCommandHostOutcome::Other(r);
    }
    #[allow(unused_mut)]
    let (mut child, mut from_unshare) = {
        #[cfg(target_os = "linux")]
        {
            // Any unshare(1) spawn failure (not only EPERM) falls back: missing util-linux, wrong
            // `PATH`, container policy, etc. must not block the logical `command` on the direct
            // path (PR #792: PB-Runtime gap vs restricted Linux hosts).
            match build_execute_command_unshare(command, args).spawn() {
                Ok(c) => (c, true),
                Err(e_unshare) => match build_execute_command_process(command, args).spawn() {
                    Ok(c) => (c, false),
                    Err(e2) => {
                        return ExecuteCommandHostOutcome::Other(ClaimResult::Fail(format!(
                            "ExecuteCommand: could not run `{command}`: `unshare(1) -c -f -p` \
                             wrapper failed to spawn: {e_unshare}; direct spawn also failed: {e2} \
                             (P3/P4 — util-linux/namespace or host binary path)"
                        )));
                    }
                },
            }
        }
        #[cfg(not(target_os = "linux"))]
        {
            match build_execute_command_process(command, args).spawn() {
                Ok(c) => (c, false),
                Err(err) => {
                    return ExecuteCommandHostOutcome::Other(ClaimResult::Fail(format!(
                        "ExecuteCommand spawn error ({command}): {err}"
                    )));
                }
            }
        }
    };
    // `continue` (Linux only) may re-run the logical command without `unshare(1)` if stderr
    // shows namespace *setup* failed (not a logical exit).
    #[cfg(target_os = "linux")]
    let mut unshare_stderr_drain: Vec<u8> = Vec::new();
    #[cfg_attr(
        not(target_os = "linux"),
        allow(clippy::never_loop) // the only `continue` is under `#[cfg(target_os = "linux")]`
    )]
    let status = loop {
        // Fresh capture for this `Child` (each `continue` spins up a new process; stale bytes would
        // poison the unshare: merge — codex PR #792).
        #[cfg(target_os = "linux")]
        unshare_stderr_drain.clear();
        let s = match child_wait_for_execute_command(
            &mut child,
            wall_time,
            from_unshare,
            #[cfg(target_os = "linux")]
            &mut unshare_stderr_drain,
        ) {
            Ok(s) => s,
            Err(e) => return ExecuteCommandHostOutcome::Other(e),
        };
        #[cfg(target_os = "linux")]
        {
            if unshare_post_start_stderr_may_authorize_relaunch(from_unshare, &s, expect_exit_code)
            {
                if let Some(reason) = unshare_sandbox_broken_relaunch_with_direct(
                    from_unshare,
                    &mut child,
                    &unshare_stderr_drain,
                ) {
                    child = match build_execute_command_process(command, args).spawn() {
                        Ok(c) => c,
                        Err(e2) => {
                            return ExecuteCommandHostOutcome::Other(ClaimResult::Fail(format!(
                                "ExecuteCommand spawn error ({command}) after {}: {e2}",
                                unshare_reexec_after_spawn_error_label(reason)
                            )));
                        }
                    };
                    from_unshare = false;
                    continue;
                }
            }
            if from_unshare {
                let is_nonzero_match = s
                    .code()
                    .map(i64::from)
                    .is_some_and(|c| c == expect_exit_code)
                    && expect_exit_code != 0;
                if is_nonzero_match {
                    // Non-zero + exit already matches: the unshare(1) path can (rarely) conflate
                    // PID-1 or namespace semantics with a **direct** `Child` (see module docs).
                    // **Always** run one direct-`Child` “host confirmation” re-exec. A merged wrapper
                    // `stderr` with no `unshare:` line (including **empty**) is *not* positive evidence
                    // that `exec` reached the logical child — treating it as a skip was **fail-open**
                    // (api-review codex 946a9918, PR #792). Merge `Err(())` (read / handle loss) is also
                    // not authority for a different conclusion; we still re-run for confirmation.
                    // **P5:** same retirement as post-start + empty-stderr — `UnshareDirectRerun` module doc.
                    let _merged =
                        unshare_merged_wrapper_stderr_read(&mut child, &unshare_stderr_drain);
                    let reason = UnshareDirectRerun::NonzeroHostConfirm;
                    child = match build_execute_command_process(command, args).spawn() {
                        Ok(c) => c,
                        Err(e2) => {
                            return ExecuteCommandHostOutcome::Other(ClaimResult::Fail(format!(
                                "ExecuteCommand spawn error ({command}) after {}: {e2}",
                                unshare_reexec_after_spawn_error_label(reason)
                            )));
                        }
                    };
                    from_unshare = false;
                    continue;
                } else {
                    // P3/C-5: same fd as child stderr after exec; do not leave a piped `stderr` on
                    // paths that skip the merge+decision above.
                    let _ = child.stderr.take();
                }
            }
        }
        break s;
    };
    #[cfg(not(target_os = "linux"))]
    let _ = from_unshare;
    let Some(actual) = status.code().map(i64::from) else {
        return ExecuteCommandHostOutcome::Other(ClaimResult::Fail(
            "ExecuteCommand: child terminated by signal (no host exit code)".to_string(),
        ));
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
    pub source: String,
    pub file_name: String,
    pub predicate: FieldValue,
    pub requires: Vec<FieldValue>,
}

pub struct TestRunner<'a> {
    dag: &'a Dag,
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
        let result = if !claim.requires.is_empty() {
            ClaimResult::Fail(format!(
                "TestClaim `{}` declares {} resource requirement(s), but the Rust runner cannot materialize `requires` yet",
                claim.claim_name,
                claim.requires.len()
            ))
        } else {
            match self.variant_value(&claim.predicate) {
                Some((label, payload)) => match label.as_str() {
                    "Compiles" => self.eval_compiles(claim),
                    "FailsWithDiagnostic" => self.eval_fails_with_diagnostic(claim, &payload),
                    "OutputEquals" => self.eval_output_equals(claim, &payload),
                    "PortHasState" => self.eval_port_has_state(claim, &payload),
                    "CostBounded" => self.eval_cost_bounded(claim, &payload),
                    "LensOutputEquals" => self.eval_lens_output_equals(claim, &payload),
                    "DifferentialEquals" => self.eval_differential_equals(claim, &payload),
                    "AlgebraicLaw" => self.eval_algebraic_law(claim, &payload),
                    "ExecuteCommand" => self.eval_execute_command(claim, &payload),
                    "MockBackedInvariant" => {
                        let inner = self.eval_mock_backed_invariant(claim, &payload);
                        if claim.requires.is_empty() {
                            match inner {
                                ClaimResult::Pass => ClaimResult::NotYetImplemented(
                                    "MockBackedInvariant: `TestClaim.requires` is empty — DB-15 mock \
                                     obligations attach only on `requires` as `ResourceReference` edges; \
                                     hermetic subject/invariant application succeeded but is not a mock-backed \
                                     receipt until at least one obligation is declared (M1(2.8): list bodies \
                                     in fixture `TestClaim` data are not expressible yet)."
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
                },
                None => ClaimResult::Fail("predicate is not a structural variant".to_string()),
            }
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

        // R1 gate sentinel: `Dag` inputs are not yet expressible as structural `data` bodies in the
        // fixture DSL; `r1_lens_output_input_from_program` names a typed placeholder while the
        // runner reflects `Dag.nodes` from `TestClaim.source` / `file_name`.
        // **Dissolution trigger (ROADMAP / INVARIANTS P2):** replace string matching on this name
        // with a structural `TestClaim` / `std.verification` coproduct arm (reflection input vs
        // literal body) so runners do not key behavior on declaration spellings.
        const PROGRAM_INPUT_SENTINEL: &str = "r1_lens_output_input_from_program";

        if input_decl.value_body.is_none() {
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
        // `TestClaim` metadata) without fixture-local stub bodies — same upstream fix as retiring
        // `PROGRAM_INPUT_SENTINEL` string dispatch above.
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
            if input_decl.name.as_deref() != Some(PROGRAM_INPUT_SENTINEL) {
                return ClaimResult::Fail(format!(
                    "LensOutputEquals(cost_of): input_ref must be `{PROGRAM_INPUT_SENTINEL}` sentinel, got `{input_name}`"
                ));
            }
            let Some(cost_bind) = cost_bind_for_claim_file(&claim.file_name) else {
                return ClaimResult::Fail(format!(
                    "LensOutputEquals(cost_of): no structural-cost bind mapping for file `{}`",
                    claim.file_name
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

        let input_field = if input_decl.name.as_deref() == Some(PROGRAM_INPUT_SENTINEL) {
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
                        "LensOutputEquals: input_ref `{input_name}` has no value body (use `{PROGRAM_INPUT_SENTINEL}` sentinel when the input `Dag` is only available via `TestClaim.source`)"
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

    fn resolve_declaration_ref_id(
        &self,
        value: &FieldValue,
        field_label: &str,
    ) -> Result<DeclarationId, String> {
        match value {
            FieldValue::Reference(id) => Ok(*id),
            FieldValue::Record(fields) if fields.is_empty() => Err(format!(
                "LensOutputEquals `{field_label}`: DeclarationRef is the empty record literal {{}} — use an identifier \
                 so lowering emits FieldValue::Reference(DeclarationId), not an empty record",
            )),
            other => Err(format!(
                "LensOutputEquals `{field_label}`: expected FieldValue::Reference(DeclarationId) \
                 for a DeclarationRef edge, got {other:?}"
            )),
        }
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

        const PROGRAM_INPUT_SENTINEL: &str = "r1_lens_output_input_from_program";
        if input_decl.name.as_deref() != Some(PROGRAM_INPUT_SENTINEL) {
            return ClaimResult::Fail(format!(
                "DifferentialEquals: input_ref must be `{PROGRAM_INPUT_SENTINEL}` sentinel, got `{input_name}`"
            ));
        }

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

        let Some(cost_bind) = cost_bind_for_claim_file(&claim.file_name) else {
            return ClaimResult::Fail(format!(
                "DifferentialEquals: no structural-cost bind mapping for file `{}`",
                claim.file_name
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

        let pairing_ok = (subject_lineage.as_str() == "v3_program_cost"
            && oracle_lineage.as_str() == "v2_oracle_cost")
            || (subject_lineage.as_str() == "v2_oracle_cost"
                && oracle_lineage.as_str() == "v3_program_cost");
        if !pairing_ok {
            return ClaimResult::NotYetImplemented(format!(
                "DifferentialEquals(cost): only the (v3_program_cost, v2_oracle_cost) lineage pairing is implemented; got ({subject_lineage}, {oracle_lineage})"
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
        // Only `Associativity` is wired via D3 multi-triple operational witness (see
        // `eval_algebraic_law_for_claim_program` — not substrate law-fact evaluation).
        // Other `AlgebraicLawKind` variants are `NotYetImplemented` (runner cannot evaluate yet),
        // not `Fail` (claim false).
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
        if law_label != "Associativity" {
            return ClaimResult::NotYetImplemented(format!(
                "AlgebraicLaw::{law_label} is not wired in the Rust runner yet"
            ));
        }
        if !law_payload.is_empty() {
            return ClaimResult::Fail("Associativity should be payload-free".to_string());
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
                "AlgebraicLaw Associativity: operational witness failed (must pass all {} fixed \
                 Int triples in lens_apply::ASSOCIATIVITY_WITNESS_TRIPLES; D1 apply — not a \
                 substrate declared-law check; see eval_algebraic_law_for_claim_program)",
                ASSOCIATIVITY_WITNESS_TRIPLES.len()
            )),
            Err(AlgebraicLawProgramError::MalformedPayload(message)) => ClaimResult::Fail(message),
            Err(AlgebraicLawProgramError::UnsupportedLaw { law_label }) => unreachable!(
                "eval_algebraic_law gated on Associativity; helper cannot return UnsupportedLaw({law_label:?})"
            ),
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
        ValueBody::Unparsed(_) | ValueBody::Scalar(_) => None,
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

fn diagnostic_kind(diagnostic: &Diagnostic) -> &'static str {
    match diagnostic {
        Diagnostic::TokenizerError { .. } => "TokenizerError",
        Diagnostic::ParseError { .. } => "ParseError",
        Diagnostic::TypeMismatch { .. } => "TypeMismatch",
        Diagnostic::ArityMismatch { .. } => "ArityMismatch",
        Diagnostic::ResolveError { .. } => "ResolveError",
        Diagnostic::BranchConditionNotBool { .. } => "BranchConditionNotBool",
    }
}

fn diagnostic_matches_reference(
    diagnostic: &Diagnostic,
    reference: &(String, DiagnosticDetailFilter),
) -> bool {
    diagnostic_kind(diagnostic) == reference.0
        && match &reference.1 {
            DiagnosticDetailFilter::Any => true,
            DiagnosticDetailFilter::Contains(text) => diagnostic.message().contains(text),
        }
}

fn render_value_body(dag: &Dag, value: &ValueBody) -> String {
    match value {
        ValueBody::Scalar(bits) => render_literal(bits),
        ValueBody::Structural { fields } => render_record(dag, fields),
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
        assert!(!shell_dash_c_may_start_background_after_eliding_artifacts(
            "true && true"
        ));
        assert!(!shell_dash_c_may_start_background_after_eliding_artifacts(
            "true 2>&1"
        ));
        assert!(!shell_dash_c_may_start_background_after_eliding_artifacts(
            "cmd 3>&4"
        ));
    }

    #[test]
    fn elision_still_fails_on_shell_background() {
        assert!(shell_dash_c_may_start_background_after_eliding_artifacts(
            "sleep 600 &"
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

    /// P2(c): a missing `command` can return **127** on the unshare shell path with stderr nulled
    /// and no `unshare:` in the merge. Direct `std::process::Command` typically **fails to spawn**;
    /// a claim that expects 127 must not `Match` the unrun-only result (PR #792 codex).
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
        let ExecuteCommandHostOutcome::Other(ClaimResult::Fail(msg)) = r else {
            panic!("expected spawn/execute fail for missing command, not {r:?}");
        };
        assert!(
            msg.contains("ExecuteCommand")
                && (msg.contains("spawn") || msg.contains("error") || msg.contains("No such file")),
            "expected spawn-fail phrasing, got: {msg}"
        );
    }

    /// Linux: the unshare bootstrap re-`exec`s the user `command` with logical `stderr` to
    /// `/dev/null` (and `child_wait` still drains the **wrapper** pipe for util-linux/bootstrap only).
    /// If logical `>&2` were still the piped handle, the loop below would fill the pipe and stall
    /// until the wall cap — this test passing is the receipt that they are **not** the same
    /// authority (PR #792; c.f. `UNSHARE_LOGICAL_BOOTSTRAP_SH` module doc).
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
            "i=0; while [ $i -lt 1 ]; do i=$((i+1)); done >&2; exit 0"
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

// Linux: util-linux unshare(1) stderr heuristics for post-start direct retry (see PR #792).
#[cfg(all(test, target_os = "linux"))]
mod unshare_stderr_scan_tests {
    use super::unshare_merge_stderr_for_setup_scan;
    use super::unshare_stderr_indicates_sandbox_setup_failure;

    /// Regression: if `unshare:…` arrived only in the pre-exit (wait-loop) drain, it must not be
    /// lost before the setup scan (codex PR #792, blocking review).
    #[test]
    fn pre_wait_drain_merged_with_empty_post_still_triggers() {
        let c = unshare_merge_stderr_for_setup_scan(b"unshare: Operation not permitted\n", "");
        assert!(unshare_stderr_indicates_sandbox_setup_failure(&c));
    }

    #[test]
    fn any_unshare_prefix_triggers_not_only_failed_substring() {
        assert!(unshare_stderr_indicates_sandbox_setup_failure(
            "unshare: Operation not permitted\n"
        ));
    }

    #[test]
    fn classic_unshare_failed_message_still_triggers() {
        assert!(unshare_stderr_indicates_sandbox_setup_failure(
            "unshare: unshare failed: some syscall\n"
        ));
    }

    #[test]
    fn empty_or_unrelated_stderr_no_trigger() {
        assert!(!unshare_stderr_indicates_sandbox_setup_failure(""));
        assert!(!unshare_stderr_indicates_sandbox_setup_failure(
            "hello from program\n"
        ));
    }
}

// P3/C-5: do not use stderr as setup authority when the exit code already matches the claim.
#[cfg(all(test, target_os = "linux"))]
mod unshare_post_start_stderr_relaunch_authority_tests {
    use super::unshare_post_start_stderr_may_authorize_relaunch;
    use std::process::Command;

    #[test]
    fn matching_exit_does_not_authorize_stderr_relaunch() {
        let s = Command::new("true").status().expect("true");
        assert!(!unshare_post_start_stderr_may_authorize_relaunch(
            true, &s, 0
        ));
    }

    #[test]
    fn exit_mismatch_authorizes() {
        let s = Command::new("sh")
            .args(["-c", "exit 1"])
            .status()
            .expect("sh");
        assert!(unshare_post_start_stderr_may_authorize_relaunch(
            true, &s, 0
        ));
    }

    #[test]
    fn not_unshare_path_does_not_authorize() {
        let s = Command::new("sh")
            .args(["-c", "exit 1"])
            .status()
            .expect("sh");
        assert!(!unshare_post_start_stderr_may_authorize_relaunch(
            false, &s, 0
        ));
    }
}
