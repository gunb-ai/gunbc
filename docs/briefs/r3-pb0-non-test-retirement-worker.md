---
status: Mgr-authored worker brief (R3 Debt-Paydown Mgr → worker dispatch)
authority_parent: R3 Debt-Paydown Mgr (`zesty-boar-261`)
authoring_date: 2026-05-13
director_dispatch: msg_3f8442f6-c46a-448d-8c55-2d0e0d945a32
pm_endorsement: msg_55c5f2fb
program_anchor: PR #3013 R3-actual-close **Gap 1** — PB-0 `EXPECTED_HAND_AUTHORED_NON_TEST` retirement cluster
---

# PB-0 — SG-0 NON_TEST hand-Rust retirement worker brief

**Dispatch.** Debt-Paydown Mgr standing program: sustained **per-PR** retirement of paths listed in `EXPECTED_HAND_AUTHORED_NON_TEST` inside `src/v3/compiler/tests/integration/sg0_census_test.rs` (const opens ~L237). **Target cadence:** retire **5–10** distinct census paths **per merged PR** (Director directive). **Ratchet:** `feedback_ratchet_only_down` — the `wc -l` / unique-path count for this const slice **must strictly decrease** on the PR that touches the census; **never** add a `NON_TEST` entry without Director-tier substrate-debt justification (substrate fix lands elsewhere; census expansion here is almost always STOP-and-escalate).

**Closure alignment.** §1.8 gate **#8** `sg0_non_test_zero` (`docs/r3-program-plan.md`) — T-PB-A ratchet to **0** for `EXPECTED_HAND_AUTHORED_NON_TEST` ∪ `EXPECTED_HAND_AUTHORED_FRAGMENTS` per `docs/design-pure-bootstrap-zero.md` + `docs/r3-structure.md` §Acceptance. This brief owns **NON_TEST** slice only unless the same PR also carries an explicit, separately justified fragments row (out of scope by default).

## Read first

- [`docs/audit/r3-sg0-trajectory-tracker.md`](../audit/r3-sg0-trajectory-tracker.md) §2 (count procedure) + §4.1 (intro-source decomposition — **NON_TEST** churn vs TEST class).
- [`ROADMAP.md`](../../ROADMAP.md) T-PB-A row — lens-producer priority + SG-0 census authority.
- [`INVARIANTS.md`](../../INVARIANTS.md) Dispatch-Discipline **(b)** — census line removals carry **same-PR** dissolution / retirement receipt (no orphan drops).
- [`TESTING.md`](../../TESTING.md) Band-C / gate **#87** context — distinguish **transient cementing wiring** (retires when #87 closure completes) from **compiler-authority** retirement (preferred for this brief’s “highest-leverage” bucket when available).
- PR **#3013** Gap 1 framing + PM adversarial audit **msg_93d61169** (use **live** §2 count at your branch tip — do not assume a specific integer head-count without running the procedure).

## Retirement source priority (Director-ordered)

1. **Highest leverage — T-LP-Retirement / Cluster F-α adjacent**  
   Compiler-authority retirements in the shape of recent **`lens_apply.rs`** + **`workflow_parallelism.rs`** census dissolutions: consumers, call-site wiring, walker ports toward `.dag` / substrate authority. Aligns with gate **#5** + Cluster **F-α** program.

2. **Medium leverage — T-Tier3 mirror dissolution (§1.8 #2 `tier3_computation_mirror_dissolved`)**  
   `std.computation` mirror surface called out in program-plan notes (e.g. `lower_call_pattern`, `size_bound_param`, `is_constant_bound`, `constant_bound_value`, `algebra_profile_to_dimension`). Evaluator-gated for full PASSING; **consumer wiring / mirror retirement** that removes `NON_TEST` paths is still in-scope pre-work for this cluster.

3. **Lower leverage — cementing / test-discipline files**  
   `cementing_dispatch.rs`, `r3_gate_87_cementing_regen_runner_suites.rs`, etc. Prefer **after** buckets (1)–(2) when the same cycle cannot find authority retirements — structurally different debt class (transient until gate **#87** completes).

## Per-PR acceptance checklist

1. **Count proof:** before/after `EXPECTED_HAND_AUTHORED_NON_TEST` unique `"src/v3/..."` lines via tracker §2 `awk`+`grep` recipe; **after < before** by ≥ **5** (stretch **10** when safely batchable without violating atomic-migration / P5).
2. **`cargo test -p v3-compiler --test integration sg0_v3_hand_authored_census`** (or full workspace per `TESTING.md` if you touched cross-crate surfaces) — **green**.
3. **SG-0 PR-window** (`ROADMAP.md` SG-0 discipline): if this PR edits `sg0_census_test.rs`, PR description carries **`SG-0 hand-path delta: …`** signed net path change + pairing **(a)/(b)/(c)** when net-add would otherwise trip policy.
4. **No new compiler authority** smuggled as “wiring” without substrate receipt — if retirement forces a **new** substrate-shape question, **STOP** → escalate to Debt-Paydown Mgr → Director-tier canvas if needed.

## STOP / escalate

- Census **add** required for correctness → **do not** land in this brief’s PR; route substrate fix + Director justification first.
- Substrate-shape blocker discovered mid-retirement → pause worker; Mgr escalates to Director canvas authoring per `feedback_substrate_shape_belongs_in_mgr_canvas`.
- Scope creep into **`EXPECTED_HAND_AUTHORED_TEST`** bulk port (Cluster M / gate **#84**) → **out of scope** for this dispatch; hand off in dashboard / separate work item.

## Deliverable

One squash-ready PR: **5–10** fewer `NON_TEST` paths, tests green, SG-0 discipline satisfied, brief section “Retired this cycle” listing paths + one-line dissolution receipt each (mirror moved to `.dag`, file deleted, authority merged, etc.).
