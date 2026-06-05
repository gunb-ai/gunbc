# CI Floor: parallelism + bidirectional affected gate + sccache re-enable (2026-06-05)

> **Status:** Phase 1 (sccache) implemented in this PR. Phases 2–3 specced here for
> operator/CI-Manager sign-off (each carries a branch-protection and/or modeled-detector
> dependency that forbids a unilateral land).
> **Governing contract:** `docs/planning/v4-incremental-bootstrap-ci-perf-rr-l-worksheet-2026-06-02.md`
> (RR-L). "Orchestration may sequence existing authorities but must not co-author parse,
> infer, emit, or source facts. CI budget is not a verifier. Performance caches must be
> observationally invisible, fail closed."
> **Authority layers touched:** `src/v4/workflow/ci.dag` (model authority) →
> `dsl/gunbc/ci_github_actions_workflow.dag` (carrier byte-mirror + `Source-SHA256` pin) →
> `.github/workflows/ci.yml` (transport). Enforced by
> `gunbc_ci_github_actions_workflow_matches_ci_yml_structure` and
> `gunbc_ci_github_actions_workflow_pins_ci_yml_source_checksum`
> (`src/v3/compiler/tests/integration/t_ci_workflow_as_data_demo_test.rs`).

## 1. Evidence — measured profile of commit `60826a53` (#4437), self-hosted arm64

Re-profiled by re-running the same commit twice (cold gunbc-bin cache, then warm). `ci_floor`
is the entire critical path; `fmt` (24s), `affected` (27s), `infra_isolation` (4s), and the
`ci` aggregator (4s) run in parallel and never extend wall-clock.

### `ci_floor` total

| Run | gunbc-bin cache | `ci_floor` total |
|---|---|---|
| Cold (#4437 first run, 26993455876 attempt 1) | miss → rebuilds v2 | **1033s (~17.2 min)** |
| Warm (#4437 re-run, 26993455876 attempt 2)    | hit → build skipped | **1012s (~16.9 min)** |
| Prior main baseline (26992806266, warm)       | hit                 | 1229s (~20.5 min)  |

### `ci_floor` step breakdown

| Step | Cold | Warm | Depends on | sccache today |
|---|---|---|---|---|
| Build v2 compiler (`-p v2-compiler --release`) | 76s | — (cache hit) | — | **OFF** (`RUSTC_WRAPPER=`) |
| v2 stage0 freshness receipt (`regen_stage0 --verify`) | 318s | 393s | v2-compiler built | **OFF** (`RUSTC_WRAPPER=`) |
| M1 v4 rust emit probe (`v4-m1-rust-emit-probe.sh`) | 520s | 507s | `target/release/gunbc` binary only | **N/A** — no rustc; runs the built `gunbc` to *emit* `src/v4` |
| v2 DAG emit parity (`cargo test … dag_emit_from_resolved…`) | 72s | 74s | v2-compiler-tests built | **ON** (inherits probe env) |

### Headline findings

1. **Cold vs warm caches barely matters (1033s vs 1012s, ~2%).** The gunbc-bin cache only
   gates the 76s build step. The two dominant steps run with sccache disabled / not-applicable
   and depend on the `target/` cache + the v2 compiler's own runtime, not gunbc-bin warmth. The
   run-to-run variance on stage0 freshness alone (318→393→636s across the three runs) is *larger*
   than the entire gunbc-bin saving.
2. **The emit probe is not a rustc workload** — it runs the pre-built `gunbc` binary to emit Rust
   from `src/v4`. Its ~510s is pure v2-emit runtime (the known O(n²) `find_resource_module` emit
   cost), which sccache can never touch. The real lever there is the emitter, not caching.
3. **The two heavy steps are independent** (different corpora, both read-only verification) and
   are serialized only by living in one job.
4. **Latent fingerprint mismatch:** the build step is `RUSTC_WRAPPER=` (off) while the DAG-parity
   `cargo test` inherits `RUSTC_WRAPPER=sccache` (on). Different wrapper ⇒ different cargo
   fingerprint ⇒ DAG-parity re-compiles `v2-compiler` from scratch instead of reusing the build
   step's artifacts. This plausibly explains DAG-parity at 72–74s on #4437 vs 8s on the prior
   baseline.

## 2. Phase 1 — sccache re-enable (this PR)

**Change:** remove the two undocumented `RUSTC_WRAPPER=` overrides in `ci_floor` (Build v2
compiler + stage0 freshness receipt) so the *whole* job uniformly honors the probe-decided
wrapper.

**Why this is safe and correct:**
- The `RUSTC_WRAPPER=` overrides are **undocumented and vestigial.** Traced: added to the build
  step in T-38B (`1b8ef4dc21`, one-line commit, no rationale), copy-pasted onto the stage0 step
  in #4414 (`7d357d1458`). They predate the *resilient* sccache wiring — the runner-provided
  lazy-socket sccache (ctrl#1419) and the in-workflow health-probe that degrades to a cold build
  instead of hard-failing only landed 2026-06-01 → 06-04 (#4248, #4433). The original reason
  (sccache hard-failing the bootstrap build, "failed to read response header") is now handled by
  the probe.
- RR-L compliance: sccache is a compilation cache — **observationally invisible** (keyed on
  preprocessed input + flags) and **fail-closed** (the probe degrades to cold on any
  unreachability). It sequences existing authority; it co-authors no parse/infer/emit/source fact.
- **Fixes** the §1.4 fingerprint mismatch by making build + stage0 + DAG-parity share one wrapper.
- **Reversible** (a CI perf knob) and **no branch-protection impact** (job name `ci_floor`
  unchanged, still reports `success`).

**Not applicable elsewhere:** the M1 emit probe invokes no rustc (sccache cannot help); its cost
is emitter runtime (see Phase 4 note).

**Cascade:** `.github/workflows/ci.yml` (2 `run:` strings) → carrier `run:` strings (lines 309,
319) → recompute and re-pin `Source-SHA256` → green
`gunbc_ci_github_actions_workflow_{matches_ci_yml_structure,pins_ci_yml_source_checksum}` + fmt.

**Expected effect:** cold build < 76s when sccache deps are warm; DAG-parity returns toward ~8s
by reusing the build artifacts. Warm-cache PRs (build skipped) are unaffected on that step.

## 3. Phase 2 — bidirectional, fail-closed affected gate for stage0 freshness (spec)

**Property guarded (both directions must stay covered):**
1. A `.dag`/regen-logic/Cargo-model input changed but the committed generated `.rs` was not
   regenerated (stale outputs).
2. A *generated* `.rs` was hand-edited (cementing Rust into the seed — INVARIANTS/SELF_HOSTING).

`regen_stage0 --verify` catches both because it regenerates the whole crate from scratch and
byte-compares; it trusts the committed files in neither direction.

**Skip-safe predicate (run unless provably disjoint, fail-closed):**

```
RUN stage0 freshness  IF  changed_files ∩ (
      STAGE0_DAG_INPUTS              // .dag sources feeding compile_stage0
    ∪ REGEN_LOGIC                    // regen_stage0.rs + emit/cargo-model code it calls
    ∪ CARGO_CRATE_BOUNDARY_MODEL     // the .dag #4437 moved authority into
    ∪ GENERATED_STAGE0_FILES         // registry-enumerated committed outputs
  ) ≠ ∅
ELSE skip
```

- `GENERATED_STAGE0_FILES` is the **registry-enumerated** output set
  (`assert_output_set_matches_registry`, `GENERATED_STAGE0_FILES.len()` in `regen_stage0.rs`) —
  exact, not heuristic. This term covers direction (2).
- **Blunt-but-safe v1:** "any path under `src/v2/stage0/**` OR any `STAGE0_DAG_INPUT` touched →
  run." Conservatively over-runs (e.g. a hand-maintained support file) but cannot miss either
  direction. Tighten to the exact registry set only if over-run cost is measured to matter.
- **Fail-closed default:** any unclassified path (added/renamed generated file, new `.dag` input)
  falls on the **run** side, never skip.

**Right-way implementation (NOT a bash `git diff` in ci.yml — that is the forbidden "immediate
local patch"):** extend the modeled affected detector. The `affected` job already runs
`detect-ci-affected-components` and emits `v2`/`v3`/`v4` outputs; `src/v4/workflow/ci.dag` already
has `ci_select_ci_jobs_from_affected_set`. Add a `stage0_freshness` (or reuse `v2`) signal there,
defined by the union set above, and gate the step on it.

**Open dependency / why not landed here:** affected-set *gating* of the §11.7.1 floor is an
operator-policy decision tied to the kill-criterion instrumentation
(`emit-affected-set-ci-receipt`). Needs CI-Manager sign-off. If gated at **step** level inside the
existing `ci_floor` job, there is **no branch-protection impact** (job still succeeds), so Phase 2
can ship independently of Phase 3 once the modeled signal lands.

## 4. Phase 3 — split `ci_floor` into parallel jobs (spec)

The `CiJob` model already supports this: `type CiJob { … needs: List<Symbol> … }`
(`src/v4/workflow/ci.dag:276`). RR-L explicitly permits orchestration that sequences existing
authorities. This is **not** blocked by the emit-parallelism Rc issue — that concerns
*intra-emit, per-module* parallelism (the emit model is `Rc`, `!Send+!Sync`); splitting GHA jobs
touches none of that ownership model.

**Proposed 2-way split** (parity is too small for its own job):

| Job | Steps | ~time |
|---|---|---|
| `ci_floor_stage0` | build v2 → stage0 freshness → DAG parity | ~520s |
| `ci_floor_emit`   | restore gunbc-bin cache → M1 emit probe | ~560s |

Critical path: `max(520, 560) ≈ 560s (~9.3 min)`, down from ~980–1060s (**~7–8 min saved**) —
*iff* the fleet has spare concurrent-runner capacity (self-hosted srv1/srv2, pids-cgroup
pressure; if it just queues, no wall-clock win).

**Gotchas (why operator-coordinated, not a drive-by):**
1. **Skipped ≠ success in the aggregator.** The `ci` job does `always()` + `result != 'success' →
   fail`. An affected-gate-skipped job reports `result=skipped`, which the current logic fails
   closed on. The aggregator must distinguish "skipped by affected gate" (accept) from "skipped
   due to upstream failure" (reject). Highest-risk item.
2. **Branch protection** references job IDs by name. `ci_floor` → `ci_floor_stage0` +
   `ci_floor_emit` changes the required-check set. **The operator must update branch protection**,
   or the new jobs aren't enforced and the old `ci_floor` check blocks forever. A PR that performs
   this rename *cannot go green* until branch protection is updated — so it must be landed in
   lockstep with the operator action, never speculatively.
3. **Carrier byte-mirror** must mirror the new job/step structure model → carrier → re-pin SHA,
   ASCII-only in run-strings (lexer mojibake trap, `dag_string_lexer_non_ascii_mojibake`).
4. **Cold-cache build duplication:** both jobs cold-build the compiler in parallel (~76s each,
   wasteful but concurrent). A shared `ci_floor_build` job uploading the binary artifact removes
   it but only cleanly helps the emit job. Start without it; the cache amortizes across runs.

**Composition:** Phase 3 is the natural home for the per-job affected gates from Phase 2 (gating
whole jobs is cleaner than gating a step inside a monolith), but it inherits gotcha (1).

## 5. The real long-pole (Phase 4 note, out of scope here)

Even fully parallelized and affected-gated, the floor is bounded by the **~510s emit probe** and
**~400s stage0 runtime**, both v2-compiler runtime (the O(n²) `find_resource_module` emit cost;
see `v2_emit_on2_find_resource_module`). That helps *every* PR and is the highest-leverage lever —
but it is compiler-authority work under RR-L's "do not co-author emit" constraint (it must be an
equivalence-receipted refactor of the existing emitter), not a CI-orchestration change. Tracked
separately.

## 6. Sequencing

1. **Phase 1 (this PR):** sccache re-enable. Mergeable now; no branch-protection impact.
2. **Phase 2:** modeled `stage0_freshness` affected signal + step-level gate. Mergeable
   independently once the signal lands + CI-Manager sign-off; no branch-protection impact at step
   level.
3. **Phase 3:** parallel job split + aggregator skipped-vs-success fix. **Requires operator
   branch-protection update in lockstep.** Land last.
4. **Phase 4:** emit O(n²) refactor (separate, compiler-authority, equivalence-receipted).
