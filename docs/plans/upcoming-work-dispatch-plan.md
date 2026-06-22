# Upcoming-work dispatch plan

**Status: DISPATCH PAUSED** — no new work-items / workers spawn until **operator + bright-stag-194 + neat-dove-397** agree on this doc. Existing in-flight work continues; expansions (new slices, fan-out children) are gated on agreement.

Purpose: one reviewable place for the three of us to converge on *what's running, what's next, who owns it, and how each milestone is verified by execution* — before resuming dispatch. neat-dove reviews adversarially; operator decides; bright-stag executes.

---

## A. In-flight now — POINT-IN-TIME SNAPSHOT (hand-authored-scratch, drifts)

> ⚠️ **This table is a snapshot, not authority.** Live owner/status/PR truth lives in the work-item DAG (`dashboard-ops work-items mine`) — derive it, don't trust this. Per DESIGN §6 (no second representation) — the same rule the ROADMAP-authority lane enforces — **this doc owns only the DURABLE plan** (Sections B–D: sequencing, milestones, verification bar, decisions), NOT the live status columns. If this snapshot and `dashboard-ops` disagree, `dashboard-ops` wins. (neat-dove caught my last map drifting: it missed the parser-wall and listed an inherited lane under its old owner.)

| Lane | Owner (as of snapshot) | Artifact | State |
|---|---|---|---|
| Self-host Root B (value-grounding) | bright-deer-111 | #5552 | keystone READY; slice-1 design only (fan-out HELD) |
| Self-host tail + TS | jolly-cat-29 | #5551→tail | type-alias dropped; pivoting to E0252/closure-Debug |
| ROADMAP-as-authority | merry-deer-299 | #5535 | MERGEABLE, awaiting warm-lark HOLD clearance |
| CI widen + nextest | fierce-hawk-540 | #5427 | merge-ready (CLEAN/green/0-RC) |
| CI compile-jobs | sleek-cat-446 | #5546 | held DRAFT fail-closed until #5427 |
| CI cross-host placement | proud-tern-439 | #5559 | held DRAFT fail-closed until #5427 |
| CI lead / measurement | quick-ant-298 | — | coordinating triad |
| BMC onboarding (NEW) | neat-boar-71 | — | model + read-only validation (operator-requested) |
| Emission: .gitignore target | witty-eagle-750 | #5549 | running |
| extdeps anchor-drain | lively-wren / calm-crane / deep-cat | — | running |
| Roadmap-ruling brief (docs) | bright-stag-194 | #5550 | review-ready |

---

## B. The lanes — next tasks, milestone, verification, wave-transition

**Verification standard (applies to every "verify by execution" below):** a milestone counts only with a **discriminating witness** — an input that goes RED if the grounding is wrong — plus a **non-growing `#[ignore]` roster**. A raw count ("errors drop 48%") is NOT acceptable: it's gameable by suppression or relocation (precedent: the `0331b526ee` fail-open that hid 8 real inference deficits behind a passing scan). Each scoreboard move cites its red-witness. (bright-deer's keystone already meets this — `generic_alias_coproduct_instantiation_test.dag`, red-on-revert.)

### Lane 1 — Self-host → delete `src/v1` (THE critical path)
**Why it leads:** terminal goal = emitted compiler builds green, reproduces, covers v1 host effects → delete ~154k Rust lines. Everything else is stability *around* this.

**Owner split (single-owner-per-file to avoid 05_emit_rust collisions):**
- **bright-deer (Root B — value-grounding, ~91% of the dominant E0308 errors):**
  1. #5552 keystone (generic-alias instantiation) → merge
  2. slice-1: source def-unification (coproduct = authority, dsl record-surface derived, `String`/`List`/`QualifiedName` aliases) — after #5552
  3. slice-2: FreeMonoid **value**-grounding (~8,263 errors / 66% of E0308) — #5428 construction template (Empty→`vec![]`, Cons→push, String-literal→grounded carrier)
  4. slice-3: Rc/Box **wrap-discipline** (~3,181 / 25%) — one authority computes the wrap-name at declare AND construct sites (jolly-cat's diag: fork is construct-vs-declare name disagreement, not a missing authority)
  5. dissolve 🟡 markers (`qualified_name`/`catalog`); self_gen8 3 ignores flip to grounded-emission
- **jolly-cat (clean tail + TS, fenced out of 05_emit_rust value-grounding):**
  6. E0252 import-dedup (~444) — build-now PR
  7. closure-Debug (~408) — build-now PR
  8. Lane C: TypeScript emit to first-class (beyond `add`)

**Milestone — cargo-green is a CHECKPOINT, the merkle fixed-point is the GATE** (neat-dove's structural correction): cargo-errors→0 proves the emitted Rust *compiles*, NOT that it's *correct* (the "compiles but miscompiles" §5 trap). The terminal gate is the **bit-identical self-emit fixed point** (`self_host.dag` content-hash merkle), and per the self-host model **Stage C** (run the pipeline over its own source = candidate generation) is the real blocker — **not** "scoreboard→0 then delete v1."
- checkpoint: HostNative cargo-error count `15,342 → 0` (per slice, each move citing its red-witness — see Verification standard below)
- **GATE: real fixed point** `content_hash` stage1==stage2 (Stage C) → `regen_stage0 --verify` in CI → seed-honesty (DDC) → **TERMINAL: `src/v1` deleted**
**Open ownership (DECISION C6):** who owns **Stage C / the fixed-point check**? (operator-pending; previously merry-crab-687's comparison substrate). cargo-green has owners (bright-deer/jolly-cat); the merkle gate does NOT yet.
**Wave transition:** cargo-green checkpoint → Stage C / fixed-point wave opens. v1-coupled `coercion`/`node` renames unlock only at v1-delete.

### Lane 2 — ROADMAP-as-authority
**Next:** #5535 merges (warm-lark HOLD clearance) → ROADMAP.md becomes generated from `roadmap_authority.dag`.
**Milestone (verify):** drift gate green; a *direct* ROADMAP.md edit fails CI.
**Wave transition:** post-merge, my #5560 roadmap content re-homes into `roadmap_authority.dag` (first demonstration of the new edit workflow). All future roadmap edits go via the .dag.

### Lane 3 — CI floor: full-coverage + fast + fabric placement
**Next (all gate on #5427 merging — the single unblock):**
- #5427 merge → splits rust gate into its own job (~6m full-coverage run-all via nextest)
- then proud-tern #5559: cross-host placement (srv1/srv2 by measured ResourceEnvelope) — **model + drift-gate only; live-fleet apply FENCED for operator**
- then sleek-cat #5546: compile-jobs (CARGO_BUILD_JOBS from envelope)
- quick-ant: §5 diagnosability (gate swallows nextest test-names)

**ONE measurement / THREE consumers** (placement #5559, compile-jobs #5546, within-run width re-check) all gate on #5427 merge + quick-ant's cgroup-peak measurement. The §5 bug it fixes: the existing `[measurement]` line reports `claim_executor` **self-RSS (~11.7GB) as the whole-run peak**, but the true cgroup peak is ~3.4× higher (~39.9GB) because child rustc/sccache PIDs are excluded — keying any divisor on the self-RSS reproduces the OOM. Now structural (sccache-unaccounted → `PlanUnsound`). Both consumers held fail-closed until the real cgroup peak exists post-#5427.

**⚠ BLOCKER (DECISION C7 — operator/ctrl):** the modeling-coherence gate is **head-independently broken on the ctrl side** (`ctrl/plans/lib/reducible.dag` can't resolve `std.reducible` — ctrl harness source-roots exclude gunbc `dsl/std`; + a `plans.coherence` cycle). quick-ant verified it's NOT a gunbc diff (gunbc `std.reducible` is clean; `plans.*`/`coherence.*` are external ctrl). It reds untouched gunbc PRs and, since the coherence gate is merge-blocking but not a GitHub check, **may block #5427's manual merge** — the keystone for this whole triad. Needs an operator/ctrl harness fix (source-root + cycle) or confirmation of how merges proceed past it.
**Milestone (verify):** PR CI ≤ ~6–22m at full coverage, no OOM, both hosts utilized; divisor keyed on the true cgroup peak.
**Wave transition:** #5427 merge cascades all three forward at once.

### Lane 4 — BMC / fabric lifecycle (NEW, operator-requested)
**Next (neat-boar, staged, read-only first):**
- model the 4-phase lifecycle as `.dag` over Redfish (login → rotate creds → OS install → setup)
- validate read-only against live BMC `192.168.1.192` (login + inventory) **before any write**
- then gate the write steps (cred-rotate, OS-install) behind operator confirmation
**Milestone (verify):** new Altra host onboarded factory-default → fabric-ready, with `.dag` consuming the whole lifecycle; read-path green by execution before writes.
**Open:** default factory BMC creds for this board (check `bmc_factory_credential_witness`; else operator provides).
**Wave transition:** "setup/join fabric" hands off to Lane 3's G2 runner-deployment.

### Lane 5 — Emission / medium expansion (the moat demo)
**Next:** witty-eagle .gitignore as 3rd emit target (#5549) → `omni_*` demos (openapi backend, sql ddl, doc-drift-lock).
**Milestone (verify):** 3+ distinct emit targets from one node tree; emit each, diff vs hand-authored.

### Lane 6 — extdeps hygiene (supporting)
**Next:** lively-wren / calm-crane / deep-cat drain the external-authority anchor allowlist to zero (99 modules anchored, ~5 PRs up).
**Lane-closing dependency (DECISION C8 — operator):** the allowlist can't reach empty until the operator dispositions **28 mis-homed modules** (std-vs-extdeps): widen the lens for File-self-anchor vs. strict-external. Until ruled, this lane cannot close.
**Milestone (verify):** allowlist empty; the live-clean-tree lens green on a fresh extdeps module.

### Lane 3b — CI inline-shell de-fork (NEW candidate, transport-fusion debt) — *needs agreement*
**The fork (neat-dove):** `RunStep.run` is raw concat'd bash across ~26 sites in `ci_workflow.dag`. #5427 modeled a clean `cargo.Build.Nextest` op but **bypasses the model** for the cargo build (`ci_release_build_script` hand-writes it) = a model↔realization fork *in one file*. Plus hardcoded `CARGO_BUILD_JOBS=1/2` (sleek-cat's derive target), a pinned nextest version, and a bash `uname` arch-case (we already model `TargetArchitecture`).
**Root fix:** `RunStep` carries modeled effects, not a `String`; revive the inline-shell reducibility lens (WIP, not on main). This is the existing transport-fusion-debt family ([containment guard](docs/plans/emission-ingestion-inverse.md) #5445/#5453), finally rooted.
**Decision (C9):** dispatch as its own lane, or fold into Lane 3 / Lane 7? Sequence after #5427 + #5546 (they touch the same file).

### Lane 7 — Ergonomics (✦) + fail-closed walls (§0) — *bright-stag owns/enforces*
**Next:** inert-abstraction lens (keystone — flags defined+self-tested+zero-consumer carriers); non-fold-residue audit (`_=>` over closed coproducts); generic-inference keystone (DONE #5552, the fold-reachability root).
**Milestone (verify):** new non-fold residue can't merge (the wall pairs with §0).

### Parked / post-stability (not dispatched this wave)
§3 complexity budget-gate (gated on §5 fn-body reflection) · §4 testgen anemia lens (likely advisory) · §6 idea-machine language axis · §7 react/html. Listed so the plan is complete; none dispatch until the stability band (Lanes 1–4) is through.

---

## C. Decisions needed for agreement (operator + neat-dove)

1. **Lane granularity** — is Lane 1's single-owner-of-05_emit_rust (bright-deer) right, or split value-grounding across workers (risks same-file collision)?
2. **Dispatch resume order** — which lanes resume first when the pause lifts? (proposed: finish merges #5427/#5535/#5552/#5550 → Lane 1 value-grounding slices → BMC writes → CI fabric placement.)
3. **BMC continue-or-hold** — neat-boar is operator-requested; continue the read-only modeling during the pause, or hold it too?
4. **Fleet-apply fence** — confirm cross-host placement + BMC OS-install + cred-rotate all stage as reviewable diffs with live-apply held for operator.
5. **Capacity** — ~11 concurrent sessions; is that the right width, or consolidate lanes under fewer managers?
6. **Stage C / fixed-point owner** — cargo-green has owners; the merkle self-emit gate (the real terminal) does not. Assign one (operator-pending; was merry-crab-687).
7. **Coherence-gate breakage** — the ctrl-side modeling-coherence gate is head-independently broken and may block #5427's manual merge. Operator/ctrl harness fix (source-root + cycle), or confirm how merges proceed past it. *(Time-sensitive — gates the keystone.)*
8. **28 mis-homed modules** (std-vs-extdeps) — needed to close the extdeps anchor lane. Widen-lens-for-self-anchor vs strict-external.
9. **CI inline-shell de-fork** (Lane 3b) — dispatch as its own lane, fold into Lane 3/7, or defer?

## D. What "agreement" gates
Until all three sign off: **no new work-items, no fan-out children, no new workers.** In-flight work (Section A) continues; load-bearing edits and destructive/live-fleet steps stay fenced regardless.
