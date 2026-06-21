# Nightly `--ignored` CI lane (ROADMAP §1)

*Design-first. Surfaced for sign-off before editing the load-bearing CI-gen machinery
(`gunbc.ci_spec`, `gunbc.ci_workflow`, the ci.yml drift gate). Owner: proud-deer-709 under
§1 / quick-ant-298.*

## The pain (denominated cost, DESIGN §6)

62 rust tests in `v1-compiler-tests` are `#[ignore]`'d. ~24 are **correct but expensive**
(build stage0 binary, run full compiles, hit live network). They are **never run by any
cadence** — so a regression in expensive territory is invisible until someone runs them by
hand. The deliverable is a *destination*: a scheduled lane that runs the expensive set so
"every reasoned-ignore has a lane," upgrading the per-PR completeness story (#5427).

## The category split (the heart of the ask)

An `#[ignore]` reason answers *why excused*, and the reasons fall into two disjoint buckets
the lane MUST keep apart (inventory taken on main, `grep '#[ignore'` over `src/v1/tests/src`):

**RUN-NIGHTLY (expensive-but-correct):** `Expensive: …` (build/compile/cargo/disk, 8),
`Requires building stage0 binary (~2 min)` (4), `wet-only: …` / `calls real Anthropic API`
(2, network), the `ci_`-tagged set (5), `heavy test` / `run with: … --ignored` (≈4),
`Boundary …` (2). ≈24 total.

**STAY RED-TRACKED (not run in a green lane):** known hangs under triage
(`78s/40s/119s — hanging … PERF track`, 3), blocked-on-a-fix
(`emitter seed … until #5325 co-land`, `Rc sharing bridge regressed … needs full
bootstrap-closure`), disabled-pending-rewrite (`complexity analysis disabled for memory`,
`CX gate bypassed … CX-5 analyzer rewrite`, ≈17), unimplemented-feature
(`requires full structural algebra authority`, `stage0 does not yet validate …`),
and known-broken receipts (`receipt: … 779 typecheck diags`). ≈28 total.

Running a red-tracked test in the nightly lane makes the lane perpetually red → a useless
gate (DESIGN §5: a gate that is always red protects nothing; the noise *is* the fail-open).
So **category, not `--ignored`, is the selector**: `cargo test -- --ignored` runs *all*
ignored tests indiscriminately and is wrong here.

## §3 tension: the reason is the authority, but cargo can't filter on it

`#[ignore = "<reason>"]` is the single authority for "why excused" (established by #5427,
whose completeness lens makes a reasonless ignore RED). But **libtest cannot filter tests by
ignore-reason** — `cargo test <substr> -- --ignored` filters by *test name* only. So the
run-set must be reachable by name, while the category lives in the reason. That fork is the
design decision to lock before any edit:

- **Option A — name-prefix realization (cheap, ships now).** Adopt the existing `ci_`
  convention as *the* nightly selector: a test is nightly-run IFF its name starts with the
  agreed prefix (e.g. `ci_` / `nightly_`). The nightly lane runs `cargo test -p
  v1-compiler-tests <prefix> -- --ignored`. The prefix is a *second* representation of
  "expensive," so it is a §3 parallel-authority UNLESS bound: extend #5427's completeness
  lens with a **categorization tooth** — for every `#[ignore = "<reason>"]`, assert
  `reason ∈ run-nightly-vocab ⟺ name has the prefix`. The lens makes "expensive but no
  lane" and "red-tracked but in the lane" both *unwritable*. This keeps the reason as the
  authority and derives the binding by lens, not by a hand-maintained list.

- **Option B — derived roster (purer §3, heavier).** A `.dag`-modeled selector reflects the
  test sources, partitions by a closed category vocabulary parsed from the reason, and emits
  the explicit `cargo test <name1> <name2> … -- --ignored` argv as the single authority. No
  name convention. Costs source-reflection at gen time and a richer category grammar in the
  reason string; more than §1's stabilization window needs.

**Recommendation: Option A** — concrete-before-abstract (§6), ships the destination now,
and the lens binding closes the parallel-authority hole. B is the dissolution target once
fn-body/source reflection is cheap (ties to the §4 testgen reflection work).

This requires a closed category vocabulary in the reason prefix
(`expensive:` / `wet:` / `blocked-#NNNN:` / `hang:` / `cx:` / `unimplemented:`) so the lens
can decide membership structurally rather than by ad-hoc substring. Normalizing the current
free-text reasons to that vocabulary is a prerequisite — and it overlaps #5427's reason work,
so it must be coordinated with fierce-hawk-540, not forked.

## Where the lane lives (CI-gen, load-bearing)

The substrate is ready: `WorkflowTrigger::Schedule { cron: CronSchedule }` exists
(`extdeps/github/actions.dag`), `CronSchedule` + `render_cron_schedule` exist
(`extdeps/cron/schedule_model.dag`), and the YAML projection + drift gate are generic over a
`Workflow` value (`gunbc.ci_yaml_emit`, `tools/ci_yaml_gate.dag`).

- **Option 1 — `Schedule` trigger on the existing `ci.yml`.** One file, but the single job
  runs the per-PR floor (cost-bounded subset); making it category-aware needs a
  `github.event_name == 'schedule'` branch inside the run step → conditional sprawl, and the
  floor scheduler takes no category arg today.
- **Option 2 — separate `nightly.yml` (recommended).** A second authored `Workflow` value
  with its own `Schedule` trigger and a run step that invokes the nightly cargo selector
  (Option A's `cargo test <prefix> -- --ignored`), plus the expensive `.dag` witnesses if any
  route through it. Emitted by a second `expected_nightly_yml()` and guarded by a second drift
  gate row (mirror of `ci_yaml_gate`). §3-clean: each workflow is a distinct authority; the
  nightly gate set is explicitly *not* the per-PR floor.

**Recommendation: Option 2.** Model a `nightly_ci_spec` (or extend `CiSpec` with the lane's
selector + schedule) and a `nightly_workflow`, mirroring the existing emit+drift+parse gate
trio. Touches `gunbc.ci_spec` / `gunbc.ci_workflow` / a new drift gate — all load-bearing,
hence this design-first surface.

## Sequencing / dependency

- **#5427 (fierce-hawk-540) is upstream and NOT yet merged.** It establishes the
  `#[ignore = "<reason>"]` single authority and the completeness lens this lane's
  categorization tooth extends. Building the categorization lens or normalizing reasons before
  #5427 lands would fork its lens and reason work. → **co-sequence with fierce-hawk-540**:
  either land after #5427 merges, or coordinate the shared reason-vocab + lens so we extend,
  not duplicate.
- #5431 (per-test cost measurement) is **on main** — the sequencing gate named in ROADMAP §1
  is satisfied.

## Proposed increments (each lands green-by-execution, DESIGN §5)

1. (coordinated w/ #5427) Closed category vocabulary in the ignore-reason prefix; normalize
   the 62 reasons. Discriminating: the categorization lens goes RED if a reason is
   uncategorized or prefix↔category disagree.
2. `nightly_workflow` + `expected_nightly_yml()` + nightly drift/parse gate; commit
   `.github/workflows/nightly.yml` as the byte projection. Discriminating: drift red-receipt
   (clean matches, perturb drifts), mirroring `ci_yaml_gate`.
3. The nightly run step invokes the category selector and is proven by *running* the expensive
   subset (not a grep): a discriminating input where the lane goes red on a real expensive
   regression and green on a clean tree.

## Open decisions for sign-off

1. Option A (name-prefix + lens binding) vs B (derived roster)? — recommend A.
2. Option 2 (separate `nightly.yml`) vs 1 (`Schedule` on ci.yml)? — recommend 2.
3. Co-sequence with #5427 now (extend its lens/reason work) vs wait for its merge?
4. Schedule cadence + runner: nightly (`0 <h> * * *`) on the same fleet runner as ci.yml?
