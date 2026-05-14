# R3 Grounding Manager Brief

**Status:** ACTIVE — post–R2-close **Gap 13** continuation for the **11 T-Ground** sub-lanes (operator §4 sub-item 6 ratification, 2026-05-13; PR #3013 + #3038 + bundled-5-asks Ask 2–3). Spawned as the fifth R3 program manager lane to restore **named** execution authority over residuals that had partially dispersed onto overlapping substrate work (per [`docs/r3-actual-close-plan.md`](../r3-actual-close-plan.md) §Gap 13).

## 2026-05-14 hold-pattern update (dashboard + Mgr-tier deliverables)

**Worker dispatch:** gated until the dashboard backend redeploys **`dashboard-ops work-items create --shape`** ([PR #3041](https://github.com/gunb-ai/gunbc/pull/3041) merged on `main`); until then the bound work-node stays **leaf-default** and returns HTTP 400 on child create. **Do not** run `dashboard-ops replan` to “fix” leaf shape — Director reports **auto-archive** risk (same failure mode as wise-stag-555 / neat-heron-793).

**Mgr-tier lane execution during hold** (per Director routing, 2026-05-14): scope-discrimination canvas + this brief + **Director-ratified** closure-ledger refresh for `docs/r2-closure-ledger.md` §T-Ground rows — substantive program work that does not require worker spawn.

**Close-plan anchor:** [`docs/r3-actual-close-plan.md`](../r3-actual-close-plan.md) §Gap 13 (emit.rs survivor, `emit_model.dag` scaffold, `coercion.dag` transitional consumer references, §1.8 architectural-shape decision, PB-0 coupling). This manager does **not** substitute for PB / Debt-Paydown on census ratchets; it **sequences** T-Ground obligations and publishes **handshake** signals where Substrate and Grounding overlap.

## Orient before reading

- **R3 structure / continuation:** [`docs/r3-structure.md`](../r3-structure.md) + Gap 13 narrative in [`docs/r3-actual-close-plan.md`](../r3-actual-close-plan.md).
- **R2 historical program + 11-lane decomposition:** [`docs/briefs/r2-grounding-manager.md`](r2-grounding-manager.md) (design locks, cadence PR-F…J, per-lane tables).
- **No-engine emission discipline:** [`docs/design-emission-model.md`](../design-emission-model.md) — coercion is structural projection / emission fold; no parallel hand-Rust “coercion engine” authority.
- **Scope discrimination (dispatch prerequisite):** [`docs/audit/r3-grounding-mgr-scope-discrimination-canvas-2026-05-14.md`](../audit/r3-grounding-mgr-scope-discrimination-canvas-2026-05-14.md) — classifies each sub-lane as Grounding-owned vs substrate-handshake-heavy before workers are spawned.
- **Substrate-fact introduction:** [`INVARIANTS.md`](../../INVARIANTS.md) §P1 — applies to overlapping LanguageSpec / Coercion-Fold / L6 substrate slices; Grounding does not bypass §P1 by editing substrate carriers without Substrate coordination.

## Closure ledger — signal / authority protocol

Per **`feedback_substrate_principle_audit`** + **`feedback_dissolution_authority_not_file_presence`** discipline (carried from R3 Evaluator manager pattern, PR #3053 lineage):

1. **R2 Release Manager** (closure ledger doc + inbox) remains the **single row owner** for `docs/r2-closure-ledger.md` tables.
2. **Lane-close** for a T-Ground sub-lane is still **structural gate firing** — the demo IS the gate (`docs/r2-structure.md` structural-acceptance discipline).
3. **Row transitions** enter the ledger through:
   - **Signal + ack:** Grounding Mgr sends lane-close or partial-landing signal; Release Manager applies the row update; or
   - **Director-ratified batch refresh:** hold-pattern doc PRs (such as this line of work) may refresh `Last signal` / `Notes` against HEAD **without** claiming `green` until the named gate actually fires.

Grounding Mgr **does not** silently edit ledger `Status` to `green` based on narrative proximity alone.

## Owned program scope (11 T-Ground sub-lanes)

Canonical row set: `docs/r2-closure-ledger.md` §Grounding Manager — T-Ground (`Identifier` column). At R3 spin-up the **critical path shape** from the R2 brief remains: `Pilot → Rust → LanguageSpec → Coercion-Fold → Tests → Dissolve` with Python/Go fill-queue and parallel substrate-completion lanes (Lifetime-Analyzer, Diagnostic, CrossTarget-Meta) per [`r2-grounding-manager.md`](r2-grounding-manager.md).

| Sub-lane | R3 posture (summary) |
|---|---|
| T-Ground-Pilot | **Green** — receipt #765; maintenance only. |
| T-Ground-Rust / Python / Go | **In-flight** — per-target primitive authority toward Shape-A; consume cadence locks PR-F/G/H. |
| T-Ground-LanguageSpec | **In-flight** — registry + Phase 2 remainder; **handshake** with Substrate on realization / `cost_target_realization` class carriers (#2229-class landings). |
| T-Ground-Coercion-Fold | **In-flight** — fold body + dissolution sequencing; **handshake** with Substrate on R3-tier substrate slices (#1980, #2279-class landings). |
| T-Ground-Lifetime-Analyzer | **In-flight** — R2 (a–c) landed; R3 advanced obligations per emission-model open calls unless Director re-homes. |
| T-Ground-Diagnostic | **Not-started** until first implementation PR (ledger convention) — Layer-1 kinds remain Substrate-owned (Q6.5). |
| T-Ground-CrossTarget-Meta | **In-flight** — L6 completeness + ledger key ratchet; **handshake** with Substrate on emission-path substrate (#2103-class landings). |
| T-Ground-Tests | **Not-started** — gated Q4 + Coercion-Fold body. |
| T-Ground-Dissolve | **Not-started** — after Coercion-Fold carries load; coordinate PB ratchet on emit.rs **without** owning PB program. |

## Cross-program dependencies

**Produces:** target primitive tables; LanguageSpec rows toward language-agnostic structural gate; coercion-fold emission story; L6 meta completeness evidence; dissolution sequencing inputs.

**Consumes:**

- **R3 Substrate Mgr** — shared parents (Interval / ValueBody-list+sum / lens-primitive machinery), carrier fields introduced under §P1, L6 walker substrate.
- **R3 Verification Mgr** — L4/L5 harnesses consume Shape-A grounding; does not own primitive declaration work.
- **R3 Evaluator Mgr** — `.dag`-authored emitter / witness surfaces that must meet “no parallel coercion engine” end state.
- **R3 Debt-Paydown Mgr** — `EXPECTED_HAND_AUTHORED_NON_TEST` emit/coercion rows may wait on T-Ground gate proximity; Grounding publishes **readiness** after honest gate checks.

## Autonomous dispatch authority (post–dashboard unblock)

- Authors / refreshes T-Ground worker briefs per [`docs/r2-structure.md`](../r2-structure.md) P5 dispatch discipline (dissolution trigger + ROADMAP debt stance on scaffolds).
- Dispatches workers **only** when `work-items create --shape` is live **and** scope canvas + ledger Notes identify no dual-authority collision with in-flight Substrate PRs.
- Resolves T-Ground-internal refinements; escalates substrate-shape questions to Director with §P1 receipt or Substrate queue ping.

## Reporting cadence

- **Lane-close signals** → R2 Release Manager (closure ledger) with merged PR + gate name.
- **Cross-program handshakes** (Substrate / Verification / Evaluator / Debt-Paydown) → cross-manager queue + **Gap-tier** references (not session-id anchoring per Director discipline).
- **Blockers + scope changes** → Director (`#828` thread family when already bound).
- **Dashboard unblock** → one parent message when `--shape` is confirmed in production: worker dispatch resumes.

## Acceptance — `.dag` gates

Authoritative gate strings and lane-owned naming remain per [`r2-grounding-manager.md`](r2-grounding-manager.md) §"Acceptance — `.dag` gates" until ROADMAP alignment pass renames cells. R3 adds **no parallel acceptance artifact** — each row’s **Gate** column is still the structural demo target.

## Sub-briefs (R2-authored baseline)

R2 manager + worker briefs listed in [`r2-grounding-manager.md`](r2-grounding-manager.md) §"Sub-briefs (authored / pending)" remain the **authoring baseline**. R3 Grounding Mgr spawns **delta briefs** only when a new slice needs an explicit dissolution trigger (P5) beyond those documents.

## Working state (2026-05-14)

- **Delivered in hold-pattern PR:** scope canvas + this brief + ledger `Last signal`/`Notes` refresh for T-Ground-Rust, LanguageSpec, Coercion-Fold, CrossTarget-Meta (HEAD slices #2272, #2229, #1980, #2279, #2103 per Gap 13 audit).
- **Next after dashboard unblock:** spawn workers against post-canvas brief titles; drive remaining `in-flight` rows to gate-fired `green` with Release Manager signal protocol.

## Cross-refs

- Scope canvas: [`docs/audit/r3-grounding-mgr-scope-discrimination-canvas-2026-05-14.md`](../audit/r3-grounding-mgr-scope-discrimination-canvas-2026-05-14.md)
- Closure ledger rows: [`docs/r2-closure-ledger.md`](../r2-closure-ledger.md) §Grounding Manager — T-Ground
- R3 Verification manager shape reference: [`docs/briefs/r3-verification-manager.md`](r3-verification-manager.md)
- Parallel hold-pattern brief: R3 Evaluator Manager ([PR #3053](https://github.com/gunb-ai/gunbc/pull/3053) lineage on `main`)
