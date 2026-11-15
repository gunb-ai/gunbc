# Witness-claim execution restoration — v2-bootstrap claim-step measurement

**Goal:** measure what re-instantiating witness-claim execution on CI would cost on
`v2-bootstrap`'s runner class, list which claims fail today, and record the proposed
step with its time bound. No merge; the CI time budget is closed to growth without
operator sign-off (`gunbc.design_ruling::ci_job_budgets_closed_to_growth`,
`gunbc.design_ruling::job_roster_closed_to_growth`).

**Evidence rule applied throughout:** every claim result is marked **executed** (with the
binary/run provenance) or **parsed/unobserved**. Nothing here is asserted from parsing
alone.

**Context.** gunbc CI has executed **no** witness claim since 2026-09-19. PR #11742
(`85c1356300c4`) deleted the required-witnesses-floor job from the generated
`witnesses.yml` to save ~100 self-hosted runner-min per push. The last executing run was
[34797864081](https://github.com/gunb-ai/gunbc/actions/runs/34797864081) (PR #11335,
2026-09-14): 63 `[local-repo-wet]` lines. `witnesses.yml` is **generated** by
`gunbc.witness.compiler_gate_workflow` (`expected_compiler_gate_yml`; regen
`instruments.generated_artifact_gate main_wet`, "do not hand-edit") — a claim step must be
added to the **model** and regenerated, not hand-authored. Today the workflow has one job,
`v2-bootstrap` (`[self-hosted, linux, arm64]`, parse + typecheck + instrument rows); a
current `main` push logs **zero** floor markers.

---

## Current policy — no CI claim tests until the native frontier clears (2026-11-15)

**Decided 2026-11-15 on escalation msg_b70677a1 ("No CI tests until native frontier clears", by
default on timeout). No stopgap route.** No CI claim execution is authorized: not the native
instrument row proposed below, and **not** a v1-interpreter `claim_batch`/floor stopgap. The
required-witnesses-floor job stays deleted; no CI job runs witness claims today. This is why a
green `v2-bootstrap` is not claim execution (see the header) — CI parses and typechecks, it does
not execute claims.

**Restoration trigger**, named per `gunbc.design_argument::guarantee_ladder` meta-obligation 3 (the
trigger names the capability restored): **native claim row green on a required job** — `gunbc.rung_drop`
`v1_required_lanes_withdrawn`'s own full restoration trigger (1a), the native binary executing the
**complete** `local_repo_wet_schedule` claims on a required job with their host effects bounded. A
partial native row "judges part of the population … retires nothing" (the drop's own words), so a
step running only some members does not discharge this policy. Until that fires: measure, model,
keep the model honest (PR #16512) — but add no CI claim execution.

---

## Route honesty (the load-bearing caveat)

There are **two** execution routes and their numbers are **not** interchangeable:

- **v1-interpreter route** — `claim_batch --wet`. This is the wet-floor population
  measurement (§Measuring the population). Per D1 this route is **forbidden as the
  restoration route**; it remains valid as a measurement of the floor population and as the
  number behind any v1-stopgap option. The "21 min floor / +25 min cap" figures are
  **v1-route** numbers.
- **native route** — `v2.workflow.v2_native_route`, driven by
  `claim_executor --v2-native-route` on the `required-v2-native` job (#11003-declined).
  This is the **D1-compliant** restoration route; its real wall is in §Native-route
  measuring run. **The compliant proposal is the native route, not the +25 min v1 figure.**

## Lane premise (task (c))

`v2-bootstrap` already checks out and builds the native tree (compose + `cargo build` +
`gunbc test //gunbc/instruments:*` rows), so the claim step **can be a step on this lane**
— it reuses a tree that is already checked out and built. It **cannot** be a
`claim_executor --required-ci` flag: that path is a stub (10,227 identities / 0 claims)
because `run_required_floor` was deleted with the floor job (`cli_run.rs:24277`). The
model still rosters build+floor via
`gunbc.witness.witness_floor_lanes::witness_floor_lane_roster`, and
`dag/test/claim/required_lane_claim_agreement_witness` reads the **model**, not the
materialized YAML, so it stays green against jobs that no longer exist — see §Model/YAML
honesty.

## What a red discriminates (task (b))

From `required_floor.dag` / `v1_interpreter.rs` / `cli_run.rs`: the budget is deterministic
eval-steps (`claim_eval_step_budget_for_identity()`, compared once vs completed work); the
only **armed** deadline is the **8000 ms wall hang-guard**
(`required_floor_claim_wall_safety_limit_ms()`). The harness records `cpu_ms` vs `wall_ms`
("CPU vs wall is spin vs stall, different remedies"), so a red separates:

- **code-gated (spin):** a deterministic false/type verdict on thread CPU — fails anywhere,
  needs a code fix.
- **host/environment-gated (stall):** wall time on opaque host ops; the 8 s guard does **not**
  fire (measured `guard=0` on every hang) — needs a killable process boundary or host gating.

---

## Measuring the population — v1-interpreter route (executed)

Instrument: canonical `claim_batch --wet --source-root dag --source-root src/v2 --entry <e>
--functions <csv>` (`local_repo_wet_terminal.dag:187`), one killable process per entry,
`CLAIM_IN_FLIGHT_WALL_CAP_NANOS=8e9` armed, NCONC=2 under an 11 GiB launch gate.

- **Binary (executed, aarch64, CI-built):** `claim_batch` from CI release-bins
  [43662346030](https://github.com/gunb-ai/gunbc/actions/runs/43662346030) (sha `b4b7520a`),
  run in-session on the arm64 fleet-class host.
- **Population:** 106 entries / 601 claims, wall 14,667 s. **99 executed PASS, 13 executed
  FAIL, 72 entries HANG-OPAQUE** (host/environment-gated); remainder UNOBSERVED (host-hang
  starved).
- **Corpus prep dominates:** `load_source_corpus_tokens` + `build_index` ≈ **185 s / process**;
  single-proc RSS 8–12 GiB; **peak 18.8 GiB** at NCONC=2 (4-wide OOM-killed an entry).
  Per-claim compute is small (historical [35462055101](https://github.com/gunb-ai/gunbc/actions/runs/35462055101):
  4,185 claims in 53 s).
- **Per-claim timing (executed):** over all 99 PASS claims, per-claim `wall=` max **3937 ms**,
  p99 1 ms, median 15 ms. So a **10 s per-claim killable boundary** clears the slowest legit
  claim with margin and cuts host hangs (which run to minutes); the model's 8 s guard is
  well-tuned.
- **Caps:** Scenario A (host excluded) **~21 min**; Scenario B (host in, bounded) **57 min @
  30 s / 93 min @ 60 s**. Host exclusion is therefore mandatory.

### Failing-claim list (v1 route)

**Code-gated FAIL — executed, deterministic (the fixable, load-bearing reds):**
1. `blackjack_shuffle.wet_entropy_seeded_shuffle_is_standard_deck_permutation_by_real_execution`
   — `dag/examples/blackjack/shuffle.dag:57:28`: `no arithmetic carrier: add(Int, UInt8)`
   (`acc:Int*256 + octet:UInt8`). **EXECUTED (FAIL).** Introduced by commit `a1def7ae578f`
   (2026-10-12, #13203) — **after** the last claim-executing run (2026-09-14), so a
   claim-coverable type error shipped unchecked. The strongest argument for restoration.
2. `record_emission_candidate_set_witness.multi_field_ambiguous_literal_refuses_with_located_diagnostic`
   — "census observed 0 where the claim's comparison did not hold; the observation ran and
   disagreed, which is a finding, not a missing run." **EXECUTED (FAIL)** — a genuine red.

**Environment-gated FAIL — 11, cause confirmed (not code):** `claim_batch_parse_gate_witness`,
`claim_batch_unobserved_exit_witness`, `claim_batch_exit_two_unobserved_witness`,
`claim_batch_outcomes_out_witness`, `self_host_logic_seed_unavailable_witness`. These
`mktemp` into `<root>/target/` and spawn `<root>/target/release/claim_batch`; in the
CI-binary sandbox `target/` is absent → `mktemp: No such file or directory` (verified in the
per-entry logs). Creating `target/release/` + symlinking `claim_batch` removes the failure and
the claims proceed to real execution → **false reds**; they are the intended CI contract
tests and pass on a runner with a populated `target/`. The genuine code-gated list is the 2 above.

**Host / environment-gated (stall) — 72 entries, EXECUTED-under-boundary:** all
`extdeps.shell` / real-git / network / sensor (e.g. `fan_observe_local_wet`,
`roadmap_belt_*`, `scm_repository_save*`, `fabric_*`, `host_*`, `approval_*`,
`materialization_store_local`, `durable_cas_fabric_storage`, …). The 8 s guard never fires →
need killable boundary or host gating.

**Latent red outside the population (executed, logged, no node yet):**
`required_lane_claim_agreement_witness.w_no_projected_ledger_denies_a_lane_the_roster_requires`
is genuinely red — runtime type error `join expects a list, got Variant`. Not binary skew: the
executing binary is `b4b7520a` (3 commits behind HEAD); those 3 touch only `fleet-converge.yml`
+ fleet/roadmap `.dag`, none of the wall's deps → reddens identically on its own commit. Never
seen because CI runs no claims.

---

## Native-route measuring run (executed) — the compliant route, known-red frontier debt

This is the number the operator decision is about. It is **build-dominated**, and it does
**not** reach adjudication today.

- **Run (executed, fresh, arm64 self-hosted fleet):** PR #16527 (do-not-merge scaffold, now
  closed) [43714332518](https://github.com/gunb-ai/gunbc/actions/runs/43714332518), head
  `ae9e250bbf31`. Toolchain provisioned exactly as `v2-bootstrap` does (isolated
  `HOME`/`CARGO_HOME`/`RUSTUP_HOME`, `setup-rust-toolchain`, pinned default, cached). Route:
  `claim_executor --v2-native-route --source-root dag --source-root src/v2`. `RUSTFLAGS=''`,
  `CARGO_BUILD_JOBS=6`, per-claim `CLAIM_IN_FLIGHT_WALL_CAP_NANOS=8e9`; **no**
  `GUNBC_BIND_MEMORY_CGROUP_BYTES` (the runner's cgroup tree is not writable:
  `CgroupBindDecision::Refused{CgroupTreeNotWritable}` — with binding requested the route was
  VACUOUS, 0.0 s "success", so it was removed for a real wall). Peak RSS via a process-tree
  `VmRSS` sampler.

**Recorded boundary, 2026-11-15 (executed):**

| Phase | Wall | Note |
|---|---|---|
| Seed build `cargo build --release -p v1-compiler` | **296.7 s (4.9 min)**, exit 0 | |
| Emission (270 files, closure `9ab7ea07..`) | 0 → 745 s = **12.4 min** | **dominates**, not the cargo build |
| Emitted-compiler `cargo build` | 745 → ~976 s = **3.9 min** | |
| Discriminating red (red=5225) | 976 → 1083 s = **1.8 min** | |
| Adjudication → **ABORTED** | 1084 → 1224 s = **2.3 min** | `ROUTE_EXIT=1` |
| **Route total to abort** | **1225.2 s (20.4 min)** | |
| Peak RSS (process-tree VmRSS) | **6.0 GiB** (6,311,313,408 B) @ 744 s | during emitted build |
| Driver rows | **0** | |

- **The native lane is build-dominated: ~18–20 min/push to the frontier abort; emission
  (12.4 min) dominates, not the cargo build.**
- **Adjudication cannot complete today.** The route aborts at a single cause:
  `module std.effect_grant at dag:std/effect_grant.dag did not front-end:
  body_lowering_reason_paren_group_unread → V2-NATIVE REFUSAL cause=NativeRunFailed`.
  The emitted-compiler `cargo build` succeeds; the refusal fires when the emitted compiler is
  *run* to adjudicate. Root cause: `body_lower_paren_group_lowered`
  (`src/v2/compiler/body_lowering_fold.dag:10498`, raise at :10506) refuses when a paren-group
  operand is neither lowered-behind-shells nor an operand-ref; `effect_grant.dag:296-297`
  places a group as the LHS of an infix (`(verb_eq(..) && position_under(..)) ||
  parent_covers_grant(..)`).
- **This is rostered frontier debt, not a defect to fold away.** `body_lowering_reason_paren_group_unread`
  has a `CauseOwnership` row in `src/v2/workflow/compile_door_cause_ownership.dag` (lane
  `SharedSelfHostCriticalPath`): #12173 **deliberately** introduced the refusal in place of a
  left-element narrowing (`a && b → a` would be wrong; see recurring failure mode
  `lowering_accessor_collapses_a_sequence_operand`). It zeroes when inner value shapes lower
  (function values #12210, `as` casts #12315). Editing the fold to swallow it re-breaks #12173;
  rewriting the source to dodge it is out of scope.
- **Adjudication completion is a multi-PR compiler program, not a small PR.** The native route is
  by-design red across the whole rostered body-lowering frontier: **64 `CauseOwnership` rows**,
  ~30 of them `body_lowering_*` causes on the critical path (e.g. `statement_precedes_without_binding`
  across 71 files). So there is **no "adjudication-complete" wall to measure** until that frontier
  zeroes.

**Restoring claim execution via the compliant native route therefore requires:** (1) the
rostered body-lowering frontier to zero (multi-PR compiler program, owned elsewhere), then
(2) a claim step on `v2-bootstrap`. Claim compute itself is small (~1–2 min estimated from the
v1 population: 4,185 claims = 53 s) once adjudication is unblocked; the ~12.4 min emission is
the standing cost and is paid whether or not claims execute.

---

## Model/YAML honesty — PR #16512

Additive row `w_every_rostered_lane_appears_as_an_emitted_workflow_job` on
`required_lane_claim_agreement_witness` asserts every rostered lane's `job_id` is in the
**generated** job set of `gunbc.witness.compiler_gate_workflow` (not a file read —
`a_claim_reads_yaml_directly_from_the_source_root` rules out the file read). It is **red
today** (the floored jobs are gone from the materialized workflow) = the truth, and is enrolled
in `floor_expected_red` with the chunk `..._required_lane_materialized_workflow_red`,
dissolution trigger = the native claim row landing on a required job (`rung_drop
v1_required_lanes_withdrawn`). **It restores nothing.** The **executed** evidence for the row is
the `claim_batch` binary run
[43662346030](https://github.com/gunb-ai/gunbc/actions/runs/43662346030): the 3 controls PASS
(CONTROL B still reads both lanes) and the new row returns a clean `false` over 6.02 M eval
steps. **CI green is a different claim:** a green `v2-bootstrap` run shows only that the model
change parses and typechecks and the `floor_expected_red` enrolment is well-formed — it does
**not** mean the row executed, because nothing in CI executes claims (see the header).

## Proposed step (ON HOLD — see Current policy above; not authorized to merge or add)

Per the current policy (msg_b70677a1, 2026-11-15), this step is **not authorized** until the
restoration trigger fires (native claim row green on a required job); the operator's ruling is no CI
claim tests until the native frontier clears, with no stopgap. Recorded here as the design the
restoration will follow when it is unblocked, not as a live proposal.

Add a **native instrument row** (D1 shape; not a `claim_executor --required-*` flag) to
`v2-bootstrap` that runs the **deterministic, non-host-gated** required-floor population on the
already-built tree, each claim behind a **10 s killable boundary** plus the model's 8 s guard,
with net-only / host claims excluded (`CLAIM_NET_ONLY_SITES` / `CLAIM_SITE_GATED`). Once the
frontier zeroes, the claim wall is ~1–2 min on top of the build-dominated route. **No standing
CI-time growth without the operator's explicit sign-off** (`ci_job_budgets_closed_to_growth`).
