# Nightly expensive-test CI lane (ROADMAP §1)

*Design-first, decisions locked by §1 owner (quick-ant-298, 2026-06-21). Implementation of
the load-bearing CI-gen (`nightly.yml` via `gunbc ci`/`ci_spec`) is HELD until #5427 merges;
the concrete machinery diff is escalated to the owner for review before it lands.*

Owner: proud-deer-709 under §1 / quick-ant-298.

## The pain (denominated cost, DESIGN §6)

62 rust tests in `v1-compiler-tests` are `#[ignore]`'d. ~22 are **correct but
expensive/environmental** (build stage0 binary, run full compiles, write temp projects, hit
live network). They are **never run by any cadence** — so a regression in that territory is
invisible until someone runs them by hand. The deliverable is a *destination*: a scheduled
lane that runs the expensive set so "every reasoned-ignore has a lane," upgrading the per-PR
completeness story (#5427).

## Naming note (don't get held to the literal flag)

ROADMAP §1 calls this the "`--ignored` lane," but **literal `cargo test -- --ignored` is the
wrong mechanism**: it runs *all* ignored tests indiscriminately, including the red-tracked
ones (known hangs, blocked-on-fix, disabled-pending-rewrite) → the lane would be perpetually
red, a §5 fail-open (a gate that is always red protects nothing). The name is incidental; the
mechanism is a **feature-gated conditional ignore** (below). The lane is the *nightly
expensive-test lane*, not a literal `--ignored` run.

## The category split (why a category selector is mandatory)

An `#[ignore]` reason answers *why excused*, and the reasons fall into two disjoint buckets
the lane MUST keep apart (inventory on main, `grep '#[ignore'` over `src/v1/tests/src`):

**RUN-NIGHTLY (expensive / needs-richer-env, ≈22):** `Expensive: …` (build/compile/cargo/
disk), `Requires building stage0 binary (~2 min)`, `wet-only` / `calls real Anthropic API`
(network), the `ci_`-tagged set, `heavy test`, `Boundary …` (temp project + cargo check).
These are not strictly *cost* — several are "needs the richer environment nightly provides."
The marker is **"runs nightly," regardless of why.**

**STAY RED-TRACKED (not run in a green lane, ≈28):** known hangs under triage
(`78s/40s/119s — hanging … PERF track`), blocked-on-a-fix (`emitter seed … until #5325`,
`Rc sharing bridge regressed … needs full bootstrap-closure`), disabled-pending-rewrite
(`complexity analysis disabled for memory`, `CX gate bypassed … CX-5 analyzer rewrite`),
unimplemented (`requires full structural algebra authority`, `stage0 does not yet validate …`),
known-broken receipts (`receipt: … 779 typecheck diags`), and perf-skips
(`Stream D: parser uses List<Token> … O(n)`).

## Mechanism (LOCKED): feature-gated conditional ignore — single structural authority

Reject a name-prefix-plus-sync-lens (a §3 fork: name and reason both encode category, kept in
sync by a *validation* lens — the exact anti-pattern §1 exists to kill). Reject a
source-reflection roster (right instinct, derive-don't-duplicate, but heavier than needed).
**Adopt conditional ignore**, which is libtest-native, has ONE machine-readable authority, and
needs zero roster / zero sync-lens / zero source reflection:

- **Nightly-runnable** tests:
  ```rust
  #[cfg_attr(not(feature = "ci_nightly"), ignore = "expensive: <Ns> <why>")]
  ```
- **Red-tracked / environmental-never** tests: plain unconditional
  ```rust
  #[ignore = "<reason>"]
  ```

Behaviour falls out by construction:

| build | conditional (`cfg_attr`) test | plain `#[ignore]` test |
|---|---|---|
| per-PR (`#5427`, no feature) | `ignore` applies → **skipped** (cheap-only) | skipped |
| nightly (`--features ci_nightly`) | `ignore` removed → **RUNS** (+ all always-cheap) | **still skipped** |

So "runs in nightly" is exactly the `cfg` condition — one authority, in the source, visible
and reviewable. The reason-string reverts to **pure prose** (the `Ns` / the why); it is no
longer load-bearing for selection. The §5 perpetually-red worry is *dissolved*: red-tracked
tests keep plain `#[ignore]`, so they never enter the nightly run regardless of the feature.

This generalizes better than a cost-only prefix: `Requires-stage0` / `wet-only` aren't cost,
they're environment — and the `cfg_attr` marker says "runs nightly" regardless of *why*,
which is precisely the intent.

## Lane placement (LOCKED): separate `nightly.yml`

A second authored `Workflow` value with its **own `Schedule` trigger and its own drift gate**,
NOT a `Schedule` bolted onto `ci.yml`. The substrate is ready:
`WorkflowTrigger::Schedule { cron: CronSchedule }` (`extdeps/github/actions.dag`),
`CronSchedule` + `render_cron_schedule` (`extdeps/cron/schedule_model.dag`), and the generic
YAML projection + drift/parse gate (`gunbc.ci_yaml_emit`, `tools/ci_yaml_gate.dag`). The
nightly run step invokes `cargo test -p v1-compiler-tests --features ci_nightly` (+ the
release build it needs). §3-clean: each workflow is a distinct authority; the nightly gate set
is explicitly *not* the per-PR floor.

## Sequencing (LOCKED): clean-sequence AFTER #5427 — do NOT couple

#5427 (fierce-hawk-540) is ~1h from ready and owns the `#[ignore = "<reason>"]` reason
single-authority + the completeness lens. **Do not slow it / do not co-author its lens.** Let
it land with plain `#[ignore = "expensive: …"]` / `"failing: …"` / `"environmental: …"`
prefixes (already its scheme). After #5427 merges, THIS PR does, all in its own scope:

1. **One-time `expensive: → cfg_attr` conversion.** Derive the set ONCE from #5427's
   `expensive:`/environmental reason-category. This derivation *establishes* the `cfg_attr` as
   the ongoing single authority — not a fork, because the reason-prefix stops being
   load-bearing for selection after the conversion.
2. **Add the `ci_nightly` feature** to the test crate `Cargo.toml`.
3. **`nightly_workflow`** value + `expected_nightly_yml()` + commit `.github/workflows/
   nightly.yml` as the byte projection.
4. **Nightly drift/parse gate** (mirror of `ci_yaml_gate`: clean matches, perturb drifts).
5. **Extend #5427's completeness lens** to also recognize the `cfg_attr(…, ignore = "…")`
   form (so a conditional-ignore still counts as reasoned, and a reasonless one still goes
   RED).

Coordination with fierce-hawk is therefore **minimal** — just rely on its consistent reason
prefixes as the conversion input. No shared lens to co-author, no co-sequencing thrash.

(#5431, the per-test-cost measurement keystone named as the ROADMAP §1 sequencing gate, is
already on main — that gate is satisfied.)

## Pre-staged conversion candidate list (provisional — finalize against #5427's reasons)

CONVERT to `cfg_attr(not(feature="ci_nightly"), ignore=…)` (run nightly):
- `interp_recorded_fixture_test.rs:1103` — wet-only jsonplaceholder record→replay
- `bootstrap.rs` 307/331/374/659 — Requires building stage0 binary
- `bootstrap.rs` 483/578/976 — Expensive: build + full compile (+ emitted-crate cargo test)
- `bootstrap.rs` 864 `ci_full_dsl`, 907 `ci_diagnostic_ratchet`, 924 `ci_performance_ratchet`,
  941 `ci_freshness`, 957 `ci_fixed_point` — the existing `ci_`-tagged nightly set
- `pipeline.rs` 21 `full_dsl_compiles`, 3205 (heavy), 8043/8135 (Boundary temp+cargo check),
  8173/8273/8291/8310/10333 (Expensive disk/transitive/cargo build), 10334
  `anthropic_dag_compiles_to_rust`, 10511 `anthropic_live_e2e` (live API)

JUDGMENT-NEEDED (likely KEEP plain — diagnostic dumps, not pass/fail assertions):
- `pipeline.rs:12084` `dump_complexity_report` — `--nocapture` report dump, not a green/red test.

KEEP plain `#[ignore]` (red-tracked / disabled / unimplemented): the ≈28 hangs / `#5325` /
`Rc`-regressed / `CX`-disabled / `requires structural algebra` / `779-diags` / `Stream D` set.

(The release of #5427 will rewrite many `#[ignore] // comment` into `#[ignore = "reason"]`;
the final CONVERT set is whatever carries an `expensive:`/environmental reason prefix in
#5427's normalized form. Re-derive at conversion time.)

## Proof obligation (DESIGN §5 — green-by-execution, not grep)

The nightly workflow must be demonstrated to **actually run** the expensive+cheap set green
and **skip** the red-tracked set — a discriminating input where the lane goes red on a real
expensive regression and green on a clean tree. Drift gate proven by red-receipt (clean
matches, perturb drifts), mirroring `ci_yaml_gate`.
