# Nightly residue-test CI lane (ROADMAP §1) — Pop-B-only scope

> **RE-SCOPED 2026-06-21 (operator via quick-ant-298, "opt-level won" trigger).** The original
> framing wanted a nightly lane for the *expensive* in-process compile set (Pop-A, ~22 tests)
> with a category-selector. **That is now MOOT:** `[profile.test.package] opt-level=3`
> (warm-ram-537, sccache-amortized: run-1 1m33s → run-2 0.09s; 377 pipeline tests collapse
> 28–118s → ~0.4s each) **restores Pop-A to per-PR.** So the expensive category is gone from
> the problem and **no expensive-category selector is built** (the old work-item title's central
> ask is dropped). What remains is the small, irreducible residue: **Pop-B** — the genuinely
> *out-of-process* tests (live network + cargo-subprocess on emitted output) that cannot run
> in-process and so cannot be restored to per-PR by a compiler-speed fix.

*Design-first, decisions locked by §1 owner (quick-ant-298). The load-bearing CI-gen
(`nightly.yml` via `gunbc ci`/`ci_spec`) is **escalated to the owner for review before it
lands** — that guardrail stands regardless of scope.*

Owner: proud-deer-709 under §1 / quick-ant-298.

## The pain (denominated cost, DESIGN §6) — now Pop-B only

Pop-B is the set of `#[ignore]`'d tests whose cost is **not** the in-process seed compiler
(that was Pop-A, now per-PR) but a genuine host effect: a live network call or a `cargo`
subprocess over emitted output. They are **never run by any cadence**, so a regression in that
territory — a broken HTTP record→replay, a real-API contract drift, emitted Rust that stops
compiling — is invisible until someone runs them by hand. The deliverable is a *destination*: a
scheduled lane that runs the genuine residue so "every reasoned-ignore has a lane."

**Do NOT schedule what a fix removes (§6 trap).** Two upstream drains shrink the residue first,
and a lane for anything they cover would be the schedule-away-defers-cost trap:
- **opt-level** (landed) — drains every *in-process* test (the transitive-resolve/disk-read
  set: `pipeline.rs:8273/8291/10333`, `full_dsl_compiles:21`) back to per-PR.
- **build-once** (warm-ram-537, #5450) — drains the *stage0-build-bound* set by consuming the
  floor's prebuilt release bin instead of `cargo build`-ing it (`bootstrap.rs:307/331/374/659/578`,
  `pipeline.rs:8310`).

## The residue (what genuinely earns the lane)

**CORE — live-network, irreducible (locked, in the lane regardless):**
- `interp_recorded_fixture_test.rs:1103` — wet-only live `jsonplaceholder` record→replay.
- `pipeline.rs:10510` `anthropic_live_e2e` — builds bin + calls the **real Anthropic API**.

These hit external services; they cannot be hermeticized away (the *replay* is hermetic and
runs per-PR, but the *record* leg that proves the fixture still matches upstream is genuinely
wet). They are the §1 honest-periodic residue — run on a cadence, never on the per-PR floor.

**EMITTED-CARGO subprocess — in the lane, but WITH a named dissolution trigger** (confirmed by
warm-ram-537, #5450 is answer (b): it swaps the stage0 *build* only; the `cargo check`/`cargo
test` over the **emitted crate** is a *separate* subprocess it does NOT drain):
- `bootstrap.rs:483` `bootstrap_stage0_to_stage1` (emit Rust + `cargo check` on emitted crate),
  `579` `bootstrap_fixed_point` (full compile + `cargo check`/`test`), `976`
  `bootstrap_l4_structural` (compile `weather.dag` in-process + write emitted + `cargo test`),
  `pipeline.rs:8135` `v2_trivial_import_emits_rust_that_cargo_checks`, `8173`
  `review_dag_compiles_to_rust`.

> **Dissolution trigger (§6 — a lane is the residue fallback, not a parking lot for a fixable
> defect):** these 5 are scheduled *only because* they currently shell to real `rustc` via
> `cargo`. If their intent is "emitted Rust is **structurally well-formed**," an in-process
> structural check can replace the subprocess and **drain them to per-PR** (the cause-table
> records this as a candidate second drain). They leave this lane the moment such a check exists.
> They stay only if the intent is genuinely "emitted Rust **compiles under real rustc**" (then
> truly periodic). Not my fix to build; flagged to warm-ram-537.

**EXISTING `ci_`-tagged ratchets — in the lane (genuine periodic):**
- `bootstrap.rs:863/906/923/940/956` — `ci_full_dsl`, `ci_diagnostic_ratchet`,
  `ci_performance_ratchet`, `ci_freshness`, `ci_fixed_point`. Confirmed by warm-ram-537:
  untouched by #5450 (they already used `find_or_build_stage0()` pre-Pop-B), they run the full
  CI self-compile pipeline (fixed-point, diagnostic-count, perf, freshness) — the classic
  "needs the richer nightly environment," genuine honest-periodic.

**NOT in the lane:**
- Red-tracked / unimplemented (`pipeline.rs:1532` "stage0 does not yet validate …") — stay
  plain `#[ignore]`, never enter a green lane (§5 perpetually-red guard).
- Diagnostic dumps (`pipeline.rs:12084` `dump_complexity_report --nocapture`) — not a pass/fail
  test; keep plain.

## Mechanism (LOCKED): feature-gated conditional ignore — single structural authority

Unchanged from the parked design, and *better-fit* at the reduced scope: the residue is small
and stable, so one machine-readable authority in the source is exactly right. Reject a
name-prefix-plus-sync-lens (a §3 fork) and a source-reflection roster (heavier than needed).

- **Nightly-runnable** (the residue): `#[cfg_attr(not(feature = "ci_nightly"), ignore = "<Ns> <why>")]`
- **Red-tracked / never** tests: plain unconditional `#[ignore = "<reason>"]`

Behaviour falls out by construction:

| build | conditional (`cfg_attr`) | plain `#[ignore]` |
|---|---|---|
| per-PR (no feature) | `ignore` applies → **skipped** | skipped |
| nightly (`--features ci_nightly`) | `ignore` removed → **RUNS** | **still skipped** |

So "runs in nightly" is exactly the `cfg` condition — one authority, in the source, visible and
reviewable; the reason-string reverts to pure prose. The §5 perpetually-red worry is dissolved:
red-tracked tests keep plain `#[ignore]`, so they never enter the nightly run.

## Lane placement (LOCKED): separate `nightly.yml`

A second authored `Workflow` value with its **own `Schedule` trigger and its own drift gate**,
NOT a `Schedule` bolted onto `ci.yml`. Substrate is ready:
`WorkflowTrigger::Schedule { cron: CronSchedule }` (`extdeps/github/actions.dag`), `CronSchedule`
+ `render_cron_schedule` (`extdeps/cron/schedule_model.dag`), the generic YAML projection +
drift/parse gate (`gunbc.ci_yaml_emit`, `tools/ci_yaml_gate.dag`). The nightly run step invokes
`cargo test -p v1-compiler-tests --features ci_nightly` (+ the release build it needs, and the
network secret for the wet leg). §3-clean: each workflow is a distinct authority; the nightly
gate set is explicitly *not* the per-PR floor.

## The confirmed CONVERT set (12 tests, final)

`#[cfg_attr(not(feature = "ci_nightly"), ignore = "<prose>")]` over exactly:
- **live-network (2):** `interp_recorded_fixture_test.rs:1103`, `pipeline.rs:10510`.
- **emitted-cargo (5, w/ dissolution trigger):** `bootstrap.rs:483/579/976`, `pipeline.rs:8135/8173`.
- **`ci_` ratchets (5):** `bootstrap.rs:863/906/923/940/956`.

Everything else keeps plain `#[ignore]` (red-tracked) or is already per-PR (#5450 / opt-level
drained): NOT converted.

## Sequencing (LOCKED): residue confirmed → build

1. ~~Confirm the residue with warm-ram-537~~ **DONE** (#5450 = answer (b); set above is final).
2. **`expensive: → cfg_attr` conversion** over exactly the 12 confirmed tests, deriving the set
   ONCE so `cfg_attr` becomes the ongoing single authority (the reason-prefix stops being
   load-bearing for selection after conversion — not a §3 fork).
3. **Add the `ci_nightly` feature** to the test crate `Cargo.toml`.
4. **`nightly_workflow`** value + `expected_nightly_yml()` + commit `.github/workflows/nightly.yml`.
5. **Nightly drift/parse gate** (mirror of `ci_yaml_gate`: clean matches, perturb drifts).
6. **Recognize `cfg_attr(…, ignore = "…")`** in the completeness lens (a conditional-ignore still
   counts as reasoned; a reasonless one still goes RED).

Steps 2–6 touch the load-bearing CI-gen — **escalate the concrete diff to quick-ant-298 before
landing** (next section). Coordinate step 1's input with #5450's merge so the drained tests are
not in the floor AND not in the lane (no double-coverage, no gap).

## Escalation (guardrail stands)

The concrete CI-gen diff (`nightly_workflow` + `ci_spec` touch points) is **escalated to
quick-ant-298 for review before it lands** — `gunbc ci`/`ci_spec` is load-bearing (DESIGN names
the CI floor machinery; the §1 owner pre-approves the machinery diff).

## Machinery (BUILT on-branch 2026-06-21, escalated for review-before-land)

The harness landed; the cfg_attr per-test markings are **held for the post-#5450 baseline** (so
the conversion excludes #5450-drained tests and avoids `bootstrap.rs` merge-skew). Built as the
§2-horizontal generalization (operator-signed-off seam shape):

- **`gunbc.ci_workflow_shape`** (NEW) — `repo_ci_workflow(name, triggers, run_step)`, the ONE
  workflow-construction authority (shared prelude/concurrency/env/permissions/runner/timeout;
  job id = name). `ci_workflow` and `nightly_workflow` are two ROWS (two calls). The prior
  inlined prelude moved here verbatim — copying it into a second value would fork "the repo CI
  runner prelude" into two authorities (§3); this module makes that unwritable.
- **`gunbc.ci_yaml_emit`** — `expected_workflow_yml(workflow)` the ONE emit authority;
  `expected_ci_yml` / `expected_nightly_yml` two wrapper rows (projection already polymorphic).
- **`tools.ci_yaml_gate`** — ONE gate concept `run_workflow_yaml_gate_body(path, expected,
  label)`; CiYamlGate + nightly are two INSTANCES (both per-PR so nightly.yml drift is caught
  every PR). `run_ci_yaml_gate` unchanged for `tools.ci_gates`.
- **`gunbc.workflow_yaml_project`** — Schedule arm cron wiring (`render_cron_schedule`),
  completing the latent `YamlNull`-drop scaffold (required regardless).
- **`gunbc.nightly_workflow`** (NEW) — the nightly row (Schedule `0 7 * * *` + `workflow_dispatch`,
  `--features ci_nightly` run step). **`gunbc.ci_spec`** — nightly run policy. **Cargo.toml** —
  `[features] ci_nightly = []` (declared; markings held). **`nightly.yml`** — generated byte-output.
- **`nightly_yaml_serializer_witness_test.dag`** (NEW) — floor-enrolled NightlyYamlGate witness.

**Proofs by execution** (escalation packet): (1) ci.yml byte-identical post cron-fix +
prelude-factoring (`CiYamlGate main = ExitSuccess`; `git diff HEAD ci.yml` empty; existing
`ci_yaml_serializer_keystone_holds` still true); (2) cron renders, discriminating (pre-fix
`YamlNull` → witness RED + bare `schedule:` 3102B; post-fix → `- cron: 0 7 * * *` 3124B, GREEN);
(3) NightlyYamlGate floor-enrolled, green on committed / red on perturbed, nightly.yml byte-exact.

## Proof obligation (DESIGN §5 — green-by-execution, not grep)

The nightly workflow must be demonstrated to **actually run** the residue set and **skip** the
red-tracked set — a discriminating input where the lane goes red on a real residue regression
(a broken record→replay / an emitted-crate that stops compiling) and green on a clean tree.
Drift gate proven by red-receipt (clean matches, perturb drifts), mirroring `ci_yaml_gate`.
