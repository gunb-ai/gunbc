# Upcoming Work — Post-R2 (dependency-tracked)

> **Status:** DRAFT for operator alignment. Authored 2026-06-06, immediately after R2 landed.
> **Placement note:** operator asked for this "in ctrl/". The ctrl repo's git is CI-runner-owned and not writable from the gunbc environment, and this session has no `-ctrl` worktree — so it's drafted here in gunbc `docs/planning/`. Relocate to the ctrl repo (or a ctrl worktree) on request.
> **Purpose:** the single dependency-tracked plan for the post-R2 phase, so we parallelize without missing anything and consciously gate load-bearing work. Nothing below is dispatched yet — this is the alignment pass.

## Milestone context

R2 ("gunbc emits correct Rust end-to-end") **landed and is proven**, not merely merged:
- Discriminating cert: `emit(add)` == exactly `fn add(x: i32, y: i32) -> i32 { x + y }` (whole-node match, 51/51 emit/translate).
- Suite-delta=0 **proven by member-by-member set comparison BEFORE the ready-flip** (branch 1417p/39f == origin/main 1417p/39f; NEW-failures set empty). `merged ≠ proven` was honored: the gate cleared on the set diff, not the merge.
- Landed PRs: #4462 (R2), #4470 (pilot dissolution), #4473 (grounding_typescript additive pilot), #4474 (census deletes), plus #4469 (smoke-roster fence), #4471 (StructuralEqualsClaim predicate + 2 corpus rows re-homed).

The design/measurement phase wrapped; all prior worker/manager sessions (eager-moth, fierce-otter, jolly-heron, …) auto-archived when their nodes closed. **Execution of the work below = fresh lanes seeded from the captured designs** (in this manager's memory). Sessions are cheap; the plans are durable.

---

## 1. Strategic frame — two largely-independent thrusts

| Thrust | Goal | Lane | Paced by |
|---|---|---|---|
| **A — Enforcement (the AIM)** | Make gunbc *enforce* type/nominal distinctness (today `compile` is vacuous on call-args). | infer stage | the (a)/(b) scoping result + Stage-1 reconcile |
| **B — Get off frozen v3** | Dissolve frozen v3 Rust smokes → native v4 `.dag` claim-runs. | interpreter / roster | substrate dissolutions (consumer-driven) |

They run **in parallel** (different files/stages, no conflict). **Source-authority straddles both** (perf half + opacity half) — see PD-5 and B5/A4.

---

## 2. PRE-DISPATCH GATES — close or *consciously wave* before dispatching

These are the "close before dispatch" items. Status legend: ✅ done · ⛳ open (must close/wave) · 🔀 other lane (track, not ours).

| # | Gate | Why it matters | Status | To close |
|---|---|---|---|---|
| PD-1 | **Reconcile Stage-1 (`brand→fresh-nominal-type`) vs enforcement-B** | Two mechanisms plausibly aimed at the *same* hole (make brands distinct by construction vs reject the mismatch at the call). Uncoordinated → two divergent brand-enforcement answers. | ⛳ open | Reconcile the two lanes' models **before** GO on B. Stage-1 is visible as an active "Stage 1 SCOPING (design-first)" lane on main CI (not in this subtree). |
| PD-2 | **"Bounded" must be the (a)/(b) scoping's *result*, not an assertion** | The capability finding was that infer typechecks **no** call args — which *could* mean the type isn't even propagated to the application rule yet (not bounded). | ⛳ open | Tie "bounded" to the infer-arg-propagation measurement (the (a)/(b) scoping quiet-tern was sizing). Size by measurement, not assertion. |
| PD-3 | **Brand-twin witness = reject + accept-control + run-path + suite-delta=0** | Reject-UserId-for-AccountId is *half* the test. B must also **accept** UserId-for-UserId (else it over-rejects and tanks the suite), be measured through **`run`** (compile is the vacuous path that fooled the original answer), and prove suite-delta=0. | ⛳ spec'd | Enforce the full gate at implement-time (see A3). |
| PD-4 | **Confirm SG-2 is deferred *because* raw generalizes** (not dropped in handoff) | The call was to gate SG-2 on a consumer. | ✅ done | Consumer-check verdict: raw path generalizes for all near post-`add` constructs; the only wrapper consumer (compositional/generic type positions, e.g. `Rc<FooBar<X,Y>>`) is NOT on the near roadmap → SG-2 correctly deferred, consumer-triggered. |
| PD-5 | **Source-authority is NOT just "1 roster row, defer"** | It loses two things bigger than the row: the **perf half** (the O(file_len) recurrence fixed 7×; **#4451 sits tracked-RED waiting on it**) and the **opacity half** (`CharOffset ≠ ByteOffset` is a brand-twin → same nominal-distinctness foundation as enforcement). | ⛳ open (decision) | Defer may still be right — but make the call *knowing* #4451 stays red and the opacity half belongs to **Thrust A**, not an orphan roster item. See A4 + B5. |
| PD-6 | **Witness-must-run sweep** (witnesses measuring type-facts via `compile` = false greens) | A false green is worse than a red; flagged non-deferrable. | ✅ done | Full-surface sweep: **0 false greens**. The 78 executable Bool-risk declarations all run via `--claim-run` or are RED/ERROR/non-Bool; the compile-zero-diagnostic pattern lives only in M1/`compile_host_runner` infra + comments, not the claim corpus. |
| PD-7 | **sleek-stag's EqualsClaim-B + Transform-eval arm** | The fixes that turn 2 emit corpus-rows from "harness rejects" into honest verdicts. | 🔀 other lane | Track in sleek-stag's lane (B8). Confirm it stays on the board. |
| PD-8 | **#4469 landed with a self-explaining failure message** | A fence must carry its "why" in the failure (signpost, not blank wall). | ✅ done | #4469 MERGED. Verified: `sg0_census_test.rs::v4_parse_surface_smoke_roster_is_closed_to_growth` (ceiling 18) fails with a self-explaining message — states the rule, cites ctrl#1467 §6.4, names the fix (".dag TestClaim witness under src/v4/test/claim/ run through v2, not a new v3 smoke"), and dumps the offending roster. |
| PD-9 | **the-39 triage** (env / census-drift / harness-panic classes) | Owed; shouldn't vanish. | ✅ done | Triaged by execution: 4 classes, ~12 real residue (well under 39). A=ENV(14,not real), B=census/doc-drift(9→deleted via #4474), C=perf-harness(4,repair), D=real(12; only emit one = `l5` unused-parens lint, off-keystone). The run caught the `l5` env+emit tangle. |
| PD-10 | **R2 suite-delta cleared by set-comparison before the flip** | `merged ≠ proven`. | ✅ done | Proven member-by-member before re-flip (see Milestone). The ready-flip had been a dashboard false-ready; it was re-drafted and only re-flipped after the proof. |

**Net:** PD-4, PD-6, PD-8, PD-9, PD-10 ✅ closed. **PD-1, PD-2, PD-5 are the real open gates before Thrust A dispatches.** PD-3 is spec'd (enforce at implement). PD-7 is tracked (other lane).

---

## 3. Thrust A — Enforcement (work items)

| ID | Item | Deps | Gate | Risk | Notes |
|---|---|---|---|---|---|
| **A1** | Reconcile Stage-1 (`brand→fresh-nominal`) ↔ enforcement-B model | — | — (design) | low | PD-1. Decide whether the two are one mechanism or complementary; converge before B. |
| **A2** | (a)/(b) scoping: measure whether infer propagates the formal-param type to the application rule | — | — (measure) | low | PD-2. Output sizes A3 and *defines* "bounded". |
| **A3** | Enforcement-B implement-on-branch (`call_arg_type_check_diags` in `04_infer.dag`, `node_type_compatible`) | A1, A2 | **operator GO** (load-bearing infer) | **bounded IF brand-twin passes; larger if not** | Full gate PD-3: brand-twin **reject** (UserId/AccountId) + **accept-control** (UserId/UserId) + measured via **`run`** + suite-delta=0. If `compatible` canonicalizes brands to base type → false fix → escalate (brand-aware relation / A+migration). Scope: direct-call `ExprCall` arm only; method-calls a later tranche. |
| **A4** | Opacity half of source-authority (`CharOffset ≠ ByteOffset` as a brand-twin) | A3 (shares the relation) | operator GO | medium | PD-5. Belongs to this thrust, not the roster. Consumer-triggered: pull when a span/position fact needs the distinctness. |

---

## 4. Thrust B — Get off frozen v3 (work items)

| ID | Item | Deps | Gate | Risk | Notes |
|---|---|---|---|---|---|
| **B0** | Mechanical follow-ons (ready now) | #4462/#4470 merged ✅ | none | low | (a) roster **row-one** promotion (add-emit cert = first promoted-green named test); (b) **lookup re-run on main** (re-measure grounding_typescript/go now that #4462's narrow lookup fix is on main — expect still-residual, confirm). |
| **B1** | **List=FreeMonoid** dissolution (Option-B chokepoint **+ detection test**) | — | operator GO (interpreter substrate) | medium | **Top value** — unblocks the CDV crashes + gap-4. Designed. Detection test = red if any list op bypasses `expect_list` (convention→named-red bridge); construction-tier (unify-rep) deferred on perf. |
| **B2** | **LOOKUP** dissolution (Option-C dual-dispatch, record-form first / native fallback) | — | operator GO | medium-**high**, least-crisp | Unblocks ≥2 grounding rows. **Verify-at-implement** (construction sites unpinned). D (deprecate native `Value::Map`) = eventual construction-tier fix. Pair with detection. |
| **B3** | Gap-1 Generator-arm + Gap-2 Rc-arm (add the missing `match` arms) | — | operator GO | small | 1 row each, distinct (the "1-fix-for-3" hypothesis was **falsified** — three distinct constructors, not one Record-dot-access cause). |
| **B4** | positional-Conj (builders emit `fold_list` by construction → positional-Conj unbuildable) | — | operator GO | **lowest** (contained, construction-tier) | Consumer = the **emit** round-8 recurrence, *not* the roster (low roster value). Different file (06_translate/rust.dag) → parallel-safe with B1. |
| **B5** | source-authority **perf half** (char→byte offset table) | — | operator GO | large | Unblocks **#4451 (tracked-RED)**; the O(file_len) substring recurrence (fixed 7× per-site). Straddles with A4's opacity half. PD-5 decision applies. |
| **B6** | Wave-A mass-production (mutation-witness → claim-run → delete frozen smoke) | B1/B2/B3 (per the roster-blocker map's blast-radius ranking) | — | medium | Substrate-paced: each row needs its blocker dissolved. **Use ONE consolidated runner, not a script-per-row** (the per-site-patch anti-pattern at the harness level). Each row: discriminating mutation-witness BEFORE deleting its frozen smoke. |
| **B7** | 18-file / 129-test smoke→witness migration | predicate landed (#4471 StructuralEqualsClaim) ✅; runtime-needing rows dep B1/B2 | — | low (mechanical) | See §5. Per-file fan-out, mostly unblocked. |
| **B8** | sleek-stag EqualsClaim-B + Transform-eval arm → 2 emit corpus rows honest | — | — | — | 🔀 other lane (sleek-stag). PD-7. |

---

## 5. The 18-file / 129-test smoke→witness migration (B7) — detail

Source: sleek-stag-849 V3-tests audit. The `v4_*_dag_smoke_test.rs` set is census-fenced by #4469 (ceiling pins exactly 18, shrink-only). **#4471 did not move any of these** — it landed the *predicate* (`StructuralEqualsClaim`) the migration needs + re-homed 2 corpus rows. All 18 smoke files still present; **0 migrated so far.**

| #tests | file | runtime-exec needed? |
|---|---|---|
| 26 | v4_lens_testgen_dag_smoke_test.rs | some (Generator → B3/B1) |
| 21 | v4_std_target_realization_dag_smoke_test.rs | mostly structural |
| 14 | v4_extdeps_react_dag_smoke_test.rs | structural (19-arm roster) |
| 13 | v4_std_grounding_dag_smoke_test.rs | lookup-touching → B2 |
| 10 | v4_std_model_core_dag_smoke_test.rs | structural |
| 7 | v4_lens_registry_dag_smoke_test.rs | some |
| 6 | v4_lens_application_dag_smoke_test.rs | structural |
| 5 | v4_workflow_release / v4_extdeps_file_system | structural |
| 4 | v4_std_text / v4_lens_affected_set | structural / contains→? |
| 3 | v4_lens_edit_locus / v4_extdeps_coordination | structural |
| 2 | v4_lens_identical_variant_payload / v4_lens_idempotency_claim / v4_compiler_parse_table | structural |
| 1 | v4_extdeps_formatters_black / v4_bin_main | structural |

**How to move each (mostly unblocked):** each smoke asserts two things —
1. *"the model file parses/compiles"* → becomes a **dependency, not an assertion**: a v4 witness that `import`s the model forces v2 to compile it (fail-closed on diagnostics). No predicate needed.
2. *"declares type X with these arms/fields"* → a **`StructuralEqualsClaim`** (landed #4471) vs an expected receipt node, or a reflection witness folding the compiled `Node`. This was the blocker class — now expressible.

Per file: write the v4 witness(es) under `src/v4/test/claim/`, delete the `.rs`, the census ceiling ratchets down automatically. **Mechanical, parallelizable, per-file lane** for the parse/structural majority. Only assertions needing *runtime execution* of a v4 program wait on B1/B2 (List=FreeMonoid / Transform-eval / lookup).

**Suggested kickoff:** prove the smoke→witness pattern end-to-end through v2 on two exemplars (`v4_std_text`, 4 tests; `v4_extdeps_react`, the 19-arm roster), then fan out.

**Separately — 12 other `v4_*` integration files (not `_dag_smoke`):** `v4_emit_host_harness`, `v4_compiler_emit_translate_smoke`, grounding/self-host/p9-cost, etc. Different category — some genuinely full-compile, some are Rust harnesses that *legitimately stay Rust* (they test the runner/emit host, not a model's shape). **Not all migration targets**; not census-fenced. Triage before assuming they move.

---

## 6. Dependency graph (parallelization view)

```
THRUST A (enforcement)            THRUST B (get off frozen v3)
  A1 reconcile Stage-1 ─┐           B0 mechanical (row-one, lookup re-run)  [ready now]
  A2 (a)/(b) scoping  ──┤           B1 List=FreeMonoid ─┐
                        ▼                                ├─► B6 Wave-A mass-production
  A3 enforcement-B  (GO) ◄── PD-1,2,3                   │      (per roster-blocker map,
       │                          B2 LOOKUP  ───────────┤       ONE consolidated runner)
       ▼                          B3 Generator/Rc arms ─┘
  A4 opacity half  ◄── shares relation
                                  B4 positional-Conj (emit) ── parallel, independent
                                  B5 source-authority perf (char→byte) ── unblocks #4451-RED
                                  B7 smoke→witness migration (fan-out; runtime rows dep B1/B2)
                                  B8 sleek-stag EqualsClaim-B + Transform-eval [other lane]
```

**Parallel-safe now (no cross-deps):** B0, B1, B3, B4, B7-structural-majority — plus A1/A2 (design/measure). **Gated on operator GO:** A3, B1, B2, B3, B4, B5 (all load-bearing). **Serial-ish:** A1+A2 → A3; B1/B2/B3 → B6.

---

## 7. Open questions for the operator

1. **Enforcement-B — GO?** (after PD-1 reconcile + PD-2 scoping). It's the AIM and load-bearing infer.
2. **Dissolutions — batch-authorize the value-ranked order** (List=FreeMonoid → LOOKUP → Generator/Rc → positional-Conj; source-authority decided per PD-5) **and let me drive, or gate one-at-a-time with a check-in each?**
3. **Parallelism** — confirmed YES; run Thrust A ∥ Thrust B. How many B-dissolutions concurrently?
4. **PD-5 / source-authority** — defer (accepting #4451 stays red + opacity deferred), or pull it in now given it straddles both thrusts and unblocks a tracked-red?
5. **Placement** — relocate this doc into the ctrl repo / a ctrl worktree, or keep in gunbc `docs/planning/`?
