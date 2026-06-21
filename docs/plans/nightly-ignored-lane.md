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

**GRAY ZONE — cargo-subprocess on *emitted* output (pending warm-ram #5450 confirmation):**
- `pipeline.rs:8135` (temp project + `cargo check`), `8173` (disk + temp + `cargo check`),
  `bootstrap.rs:483` (build + full compile + `cargo check`), `976` (compiles `.dag` + builds
  emitted crate + `cargo test`).

`#5450`'s build-once gives these a prebuilt *stage0* bin, but the `cargo check`/`cargo test`
that verifies the **emitted crate** compiles is a *separate* subprocess. **Open question to
warm-ram-537:** does #5450 also hermeticize that emitted-output check to an in-process
structural check (→ drains, NOT in this lane), or only swap the stage0 build (→ the
emitted-cargo subprocess remains a periodic build-smoke → in this lane)? Finalize this set on
warm-ram's answer; do not commit the candidate list until then.

**EXISTING `ci_`-tagged set (pending warm-ram confirmation):**
- `bootstrap.rs:863/906/923/940/956` — `ci_full_dsl`, `ci_diagnostic_ratchet`,
  `ci_performance_ratchet`, `ci_freshness`, `ci_fixed_point`. These are the pre-existing
  nightly-intent ratchets. In the lane *iff* they are genuine periodic ratchets and not
  build-bound work #5450 already drains.

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

## Sequencing (LOCKED): residue confirmation gates the candidate set

1. **Confirm the residue with warm-ram-537** (#5450): finalize the gray-zone + `ci_` set above.
   Until then the CONVERT list is just the two live-network core tests.
2. **`expensive: → cfg_attr` conversion**, but only over the *confirmed residue* (live-network
   core + un-drained gray-zone), deriving the set ONCE so `cfg_attr` becomes the ongoing single
   authority.
3. **Add the `ci_nightly` feature** to the test crate `Cargo.toml`.
4. **`nightly_workflow`** value + `expected_nightly_yml()` + commit `.github/workflows/nightly.yml`.
5. **Nightly drift/parse gate** (mirror of `ci_yaml_gate`: clean matches, perturb drifts).
6. **Recognize `cfg_attr(…, ignore = "…")`** in the completeness lens (a conditional-ignore still
   counts as reasoned; a reasonless one still goes RED).

## Escalation (guardrail stands)

The concrete CI-gen diff (`nightly_workflow` + `ci_spec` touch points) is **escalated to
quick-ant-298 for review before it lands** — `gunbc ci`/`ci_spec` is load-bearing (DESIGN names
the CI floor machinery; the §1 owner pre-approves the machinery diff).

## Proof obligation (DESIGN §5 — green-by-execution, not grep)

The nightly workflow must be demonstrated to **actually run** the residue set and **skip** the
red-tracked set — a discriminating input where the lane goes red on a real residue regression
(a broken record→replay / an emitted-crate that stops compiling) and green on a clean tree.
Drift gate proven by red-receipt (clean matches, perturb drifts), mirroring `ci_yaml_gate`.
