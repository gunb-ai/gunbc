# Repo stability plan — 2026-08-28 (living document)

Working plan iterated with the operator. Subject: get the repository back to a
stable place and keep it there. Maintained on branch
`claude/repo-stability-priorities-frtsy4` until adopted.

## Operator intent (recorded 2026-08-28)

- CI required checks should take **~minutes at most**, not 40, not 100.
- The desired mechanism is **minimal invocation** — affected-set / build-system-style
  caching of test results between runs ("only run what changed").
- **Witness rule:** every witness is a short, one-pass transformation contract.
  Target budget per witness: **<5ms, eventually <1ms guaranteed**. Long
  corpus-walk / integration / performance witnesses are POLICY VIOLATIONS —
  the current 44 BUDGET-REFUSED rows are considered erroring/failing now,
  not "budget too small".
- The recurring pattern to break: things reach "good enough" (e.g. 40-min CI)
  and then spiral (100 min) because nothing prices the derivative. v1 /
  reconcile / CI root work keeps being avoided; it is admitted under the
  2026-08-20 purpose ruling (in support of v2 self-host; fixing issues is fine).

## Measured state (2026-08-28, main @ e910a39)

- `witnesses` required check: **red on every recent main run** with an
  IDENTICAL set of 47 unexpected witness failures across runs (zero flake).
  Latest measured: `planned=12949 executed=12949 passed=12615 known_red_held=33
  failed=47 stale_quarantine=1 interrupted_before_verdict=44
  completed_over_cost_requirement=2` (run 33145062452).
- Wall: ~85–100 min/run (was ~30 min on 2026-08-22 at routed=10439).
  `compile.reconcile` in strict-preparation: 22 min over 4187 modules
  (15 min over 3892 on 2026-08-24 → superlinear; ~92% attributed to
  `typecheck_with_census_extra`).
- Roster growth: routed 10439 → 12949 in 6 days (+24%).
- Queue: 26 main-push runs queued at once (main pushes never coalesce);
  operator: not a priority right now.
- srv1 belt/daily workspace: DOWN since 2026-08-20. `/opt/gunbc/gunbc` is
  ~664 commits behind the admitted fleet revision; belt ticks exit 0 and
  refuse to spawn (diagnosed in #9559). Convergence chain landed
  (#9506 → #9555 → #9563 → #9576) but `fleet-converge.yml` is
  workflow_dispatch-only and the apply has not been run.

## Priorities

### P1 — CI to ~minutes: minimal invocation + short witnesses

The two whole-corpus terms (fold ~30 min, reconcile 22 min) cannot reach
minutes by optimization; only by not redoing unchanged work. Plan:

1. **Hoist corpus walks out of witnesses into run-once indexes**
   (`declaration_index` pattern: the required parse sweep derives the facts
   once; witnesses read the index in ms). Dominant violator families among
   the 44 BUDGET-REFUSED rows: `grammar_coverage_*` (65–69s),
   `enforcement_live_*` (52–60s), `cost_coverage_*` (54–62s),
   `vocabulary_containment` (~30s), plus assorted 5s rows.
   This fixes ~40 of 44 policy violations AND removes minutes of cold cost.
2. **Closure-digest verdict store (the affected-set mechanism, done the
   cache way).** Key: (witness identity, closure content digest, binary
   identity). Hit ⇒ stored verdict replayed as this run's verdict; miss ⇒
   execute, fail-closed. No git-diff observation exists in the mechanism —
   this is identity-derived, not diff-derived, which is what distinguishes
   it from the deleted affected-set machinery and its absorbing failure
   arms. Hosted in the v1 seed now (admitted: it gates all v2 work); the
   durable artifacts (digest definition via `std.content_hash`, store
   schema) survive the v2 cutover. Same treatment per-module for reconcile.
3. **Cold backstop cadence:** scheduled whole-corpus uncached run;
   cached-vs-cold divergence is a typed line-stop (the doc's own purity
   oracle). The cache is never trusted, only checked.
4. **Witness budget wall:** once violators are hoisted, enforce the short-
   witness rule as a hard refusal (existing budget machinery already
   refuses; today its refusals are ignored). Ratchet the ceiling down
   toward the 5ms/1ms target as the population complies.
5. **Derivative gate:** required run holds its own cost against a declared
   policy budget (a §5-legal oracle) so regrowth stops the line the day it
   lands, not six weeks later.
6. stage0 cargo build: sccache / runner image — ordinary work, separate.

First artifact (measurement, before building the store): compute closure
digests over the roster on the current tree, replay the last ~10 main
commits, report would-have-been hit rate + residual per-push cost.

### P2 — Main back to green: the 47 standing reds

Identical across runs; grouped by module (see run 33145062452):
doc_reachability (x6, v1+v2 copies), sole_constructor audit probes (x6),
lens_module_gate (x3), quarantine_probe_disposition (x3), ci_budget_tree
(x3), cost_coverage (x3), compiler_closure ingest/emit (x4), guarantee
probes (x5), and singletons. Plus 1 STALE-QUARANTINE
(`duplicate_definition_binding_probe` passes; remove from
`v2.workflow.floor_expected_red`). Triage by shared root, not row-by-row;
several sit in the same corpus-scan families as P1.1 and get cheap after
the hoist. Dispositions must be honest: fix, or expected-red with typed
reason + trigger — never quarantine-to-green.

### P3 — srv1 / roadmap daily workspace

Run the fleet-converge plan → apply with the landed convergence chain;
verify by the belt's own evidence (next tick spawns; dispatch_preflight
refused_axis_count 5 → 0; fresh provider-events capture). Then consider a
scheduled convergence so a 664-commit drift can never silently accumulate
again (drift was invisible for 8+ days because the belt exits 0).

### P4 — Repo cleanup pass ("pristine")

Top-level census (2026-08-28) and dispositions to decide per item:

| Path | Size | Standing | Proposed disposition |
|---|---|---|---|
| `dag/` (2962 .dag) | 40M | live corpus | keep; audit `dag/examples/` (6 demo dirs — likely stale/scaffold) and `dag/config/` (1 file) |
| `src/v1` | 17M | frozen seed + 13M `stage0/` mirror | keep (seed); no growth |
| `src/v2` | 14M (1346 .dag) | active | keep; `experimental/` dir needs a disposition |
| `docs/plans/` | **20M, 250 files** | mostly stale/pre-cut | classify each: live / completed / superseded / bankrupt; delete-first for the dead (bulk of the 20M) |
| `dag/gunbc/plans/` | 66 files | dual home with docs/plans | decide the ONE home for plans; migrate or delete |
| `docs/briefs/` | 8 files, all 2026-08-24 | one-day lane briefs | likely bankrupt with the measurement corpus; delete or fold into carriers |
| `docs/probes/` | 1 file | recreated after the 2026-08-24 bankruptcy deleted the dir whole | violates the bankruptcy ruling's spirit; model or delete |
| `docs/runbooks/` | 5 files | hand-authored ops docs | §6 out-of-band-actuation tells; keep only with dissolution obligations |
| `docs/extdeps/lotes_azifa072/` | 428K | vendor pad-array data (CSV/SVG) | verify cited by a `.dag` authority; else relocate/delete |
| `tools/*.txt,*.tsv` (5 files, ~1.1M) | receipts/manifests at repo root of `tools/` | transcribed measurement output — the class the bankruptcy deleted; delete or model (`m1c_bulk0_residual.tsv` modified TODAY — find the writer first) |
| `fixtures/`, `test/fixture_roots` | 196K/20K | fixture carriers | keep; confirm all reachable from witnesses |
| `artifacts/bmc/` | 24K | committed artifacts? | verify generated-artifact registry covers or delete |
| `provider-runtime/codex/` | 24K | ? | classify: live realization or residue |
| `DESIGN.md` / `ROADMAP.md` | 160K/58K | generated projections | verified by `generated-artifact` phase; keep |

Rule for the pass: every deletion goes through the delete-first census
(what refuses tells us what was load-bearing); anything kept must name its
consumer; transcribed measurement outputs are debt regardless of accuracy.

### P5 — Plans content refresh

After P4 shrinks the population: rewrite the surviving live plans
(floor-cut, namespace-cut, guarantee-recovery gap analysis,
replacement-migration doctrine) against the current tree, so plan prose
stops contaminating sessions with dead premises.

## Open operator decisions

1. Ruling wanted: identity-derived verdict caching (P1.2) is NOT the
   deleted diff-derived affected-set machinery — confirm it may be built,
   with the cold cadence as its honesty backstop.
2. Where do plans live: `docs/plans/` vs `dag/gunbc/plans/` (one home).
3. P3 apply touches a live host — dispatch when ready.
4. Main-push queue coalescing: deferred by operator (not a priority).

## Log

- 2026-08-28: created; measured state recorded; priorities agreed in session.
- 2026-08-28: P4 first cut — deleted 21 orphan docs (15 docs/plans, 5 docs/briefs,
  1 more) after a full reference census. Remaining candidate classes recorded in
  session discussion: 27 docs referenced only by other docs (islands to cut whole),
  ~200 docs/plans referenced from dag/src carriers, tools/ receipts (consumer:
  srv1_residue_rehearsal path strings only), docs/probes single file (referenced by
  lotes_azifa072.dag), runbooks (referenced by roadmap/actuate carriers).
- 2026-08-28: P4 second cut — deleted tools/ receipt dumps (5 files, operator:
  derived data is never committed; only .dag derivation code), 33 more stale
  off-topic plan docs (closure-safe: refs only from snapshot strings/each other),
  and src/v2/experimental/ (empty quarantine, failed concept per operator).
  DEFERRED: 112 old off-topic plans cited by live carriers (roadmap_authority
  ~25, dag plan carriers, cli_run.rs, lens modules) — need carrier-row edits;
  dag/examples/ (witness-consumed: declared_type_inhabitance, cost_estimate_float,
  bootstrap_witness.rs); artifacts/bmc (bmc_fan_program carriers consume);
  provider-runtime/codex (codex witnesses consume).
- 2026-08-28: P4 third cut — deleted docs/plans/receipts (13M, 73 files) and
  docs/plans/measurements (456K) whole: persisted measurement dumps, operator-ruled
  garbage (derived data is never committed; only the derivation code is). Every
  in-tree mention verified to be prose (comments, note strings, srv1 snapshot rows,
  a roadmap handback sentence) — no executing consumer reads any of these paths.
  Those prose pointers now reference history, same status as any receipt citation.
- 2026-08-28: P4 fourth cut — deleted the 112 deferred stale plan docs; flipped
  48 plan carriers to PlanIsAuthorityOnly (registry row is what made each deleted
  .md expected on disk — the a295e17415 precedent); stripped 45 comment-channel
  pointers to deleted docs (13 files). DEFERRED to P5: string-field citations in
  roadmap_authority.dag (~46 lines) and stale plan-carrier bodies — editing those
  requires regenerating ROADMAP.md alongside. FOUND & FIXED IN PASSING: main @
  e910a39 (#9612) does not build — duplicate shared-artifact wall-fill definitions
  re-added over #9609 (E0428 x3, cli_run.rs); deduped on this branch; main needs
  the same fix (its CI runs were still queued when found). Parse sweep advisory
  count unchanged vs main (164). NEXT CANDIDATE ROUND: delete the 48 stale
  authority-only plan carriers themselves (imports/rosters/doc_graph edits), and
  the docs/plans doc-to-doc islands.

## Finding — per-invocation whole-tree tax (2026-08-28, handoff to operator)

Every `gunbc run` executed with cwd INSIDE a git checkout pays ~60–99s before
the entry's own compile begins; the identical invocation outside a checkout, or
against a replica tree, costs <0.5s. Measured matrix (same binary, entry
`dag/std/abi.dag`, warm):

- full roots `dag + src/v2` (4187 modules): 81–99s
- `--source-root dag` only (2962): 63s
- byte-identical replica of `dag/` in a non-repo dir (2962): 0.13s
- cwd outside the repo, absolute source-root INTO the repo: 0.08s
- every partial-tree combination tried (up to 4103 modules): <0.5s

Cost is entry-INDEPENDENT: the widest closure (`src/v2/compiler/00_compile`)
costs 110s vs the smallest entry's 81s against the same roots.

MECHANISM (established by a git-shim count and gdb stack samples, not by code
read alone): exactly one git subprocess runs — `rev-parse --show-toplevel` in
`cli_run.rs resolve_process_workspace_root` — and its SUCCESS is a switch.
Inside a checkout, `resolve_entry_graph` routes through
`try_process_shared_index_for_pool → new_multi_entry_index_shell →
build_module_graph_facts_live_uncached`, which runs
`reference_resolution_facts` with a tolerant parse over EVERY module, plus
`build_module_path_index_uncached` (a second whole-tree parse). The CI floor's
own `[floor-phase] whole-tree-graph-facts wall_ms=44579` is the same pass at
the runner. Outside a checkout that pool path is skipped and the run is
correct anyway (the probe entry still compiled and reached evaluation).

HOT LEAVES (from the samples): `v1_std_core::build_newline_index` builds each
file's line index by `im::Vector::push_back` from_iter (Arc::make_mut churn
per element — plain Vec shape); `v1_std_core::intern` clones Strings inside
`pre_intern_tokens`.

NOT ESTABLISHED here: why the pool needs whole-tree facts for a single-entry
run; whether the floor's 22-min `compile.reconcile`
(`typecheck_with_census_extra`) shares any of this or is a separate subject;
whether the cwd-outside behavior differs semantically for entries that need
workspace-relative facts (the probe did not).

REPRO: shim `git` onto PATH logging argv (1 call observed); `gdb -p <pid>
-batch -ex "thread apply all bt"` during the slow window; replica trees under
a non-repo directory for the A/B.
