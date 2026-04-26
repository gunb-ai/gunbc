# R2 Substrate Manager Brief

**Status:** PROPOSAL (per [`docs/r2-structure.md`](../r2-structure.md), LIVE 2026-04-26 via PR #827). Spawns on R1 close per Transition mechanics step 4. NEW manager — no migration source.

## Orient before reading

- **R2 structure authority:** [`docs/r2-structure.md`](../r2-structure.md). Names this manager one of 6 standing R2 managers; **largest single concentration of R2 work**.
- **Program scope sources:**
  - T-Substrate: [`docs/r2-structure.md` §"Goals" item 3](../r2-structure.md) (4 substrate prereq sub-lanes).
  - B4 Identity-Carrier Substrate Pass program: [`docs/briefs/b4-identity-carrier-substrate-pass.md`](b4-identity-carrier-substrate-pass.md).
- **Cross-program producer/readiness owner:** T-Substrate sub-lanes either produce carriers or validate existing substrate readiness for Modeling Manager (3 sub-lanes) + Grounding Manager (Engine sharpened-(b) consumes ValueBody-list/sum). The Dimensions lane is already substrate-ready by audit, so it is a readiness signal rather than new carrier work.
- **Watch condition:** if Substrate becomes the new bottleneck (workers idle >7 days waiting for Substrate-authored briefs), split B4 into a dedicated standing **B4 Identity-Carrier Manager** per `docs/r2-structure.md:88` watch trigger. R2 Release Manager surfaces this signal via velocity-tripwire reporting.

## Program scope

**Two coupled sub-programs:**

### A. T-Substrate (4 sub-lanes — Goal 3 substrate prereqs)

Each sub-lane is **scoped to its paired R2 consumer set**, not full substrate-capability close.

1. **Cardinality-substrate subset for int-literal magnitude** — enough cardinality modeling to let `IntLit` carry magnitude that narrows to target int algebra at reconciliation. Consumer: Modeling Manager's int-lit item.
2. **Nominal-opaque substrate subset for `Secret<T>`** — enough nominal-type modeling to carry construction-restriction (`where only X may construct`). Consumer: Modeling Manager's `Secret<T>` item.
3. **Parametric-algebra-attachment subset for `Dimension<Carrier>`** — enough substrate to inhabit `Dimension<Unit>` in an abelian group algebra (compile error on unit-mismatch). Consumer: Modeling Manager's Dimensions item.
4. **Top-level `ValueBody` list/sum + `std.unicode` bootstrap subset** — enough Class 5 Gap 3 substrate for `data ascii_scan_order: List<CharClass> = [...]` to lower structurally. **Two named consumers**: Modeling Manager's tokenizer charclass phase-2 + Grounding Manager's Engine sharpened-(b) full pilot enumeration.

### B. B4 Identity-Carrier Substrate Pass program (12 sub-briefs)

From [`docs/briefs/b4-identity-carrier-substrate-pass.md`](b4-identity-carrier-substrate-pass.md). Treats the §0 class from `docs/briefs/debt-paydown-synthesis-2026-04-25.md` as one M-scope substrate program, not 8 item-by-item paydowns.

**Phase 1 carriers (parallel after audits):**
- B4.1 `DeclarationRef` consumer migration (existing carrier at `src/v3/spec/v3_l1.dag:69`; consumer migration only, no landing)
- B4.2 fold-shape carrier (NEW)
- B4.3 emit-helper carrier (NEW)
- B4.4 extdeps-fixture-set carrier (NEW)

**Phase 2 site dissolutions (mechanical; dispatched as Phase 1 carriers land):**
- B4.5–B4.12: 8 sites (consumer migration of new substrate)

**Phase 3:** discipline ratchet (one-time, after Phase 1) — reviewer-discipline addition for new sentinel-string sites.

## Owned deliverables (through R2 close)

| Sub-lane | Size | Current status | Carrier shape |
|---|---|---|---|
| T-Substrate cardinality-for-int-lit | M | BRIEF LANDED (`t-substrate-cardinality-int-lit-worker.md`, PR #806 merged 2026-04-25); duplicate R2 routing doc closed as redundant (`r2-substrate-cardinality-for-int-lit-subset.md`). | range facts + reconciliation narrowing; Int128/Word128 carrier deferred to sibling sub-lane |
| T-Substrate nominal-opaque-for-Secret | M | BRIEF AUTHORED (`r2-substrate-nominal-opaque-for-secret-subset.md`; PR #836 merged 2026-04-26) | nominal-type construction/access restriction |
| T-Substrate parametric-algebra-for-Dimensions | M | CLOSED BY AUDIT (`r2-substrate-parametric-algebra-for-dimensions-subset.md`; PR #836 merged 2026-04-26); substrate already exists, consumer dispatchable. | existing `Declaration.phantom_params` + `phantom_unit_mismatch` carrier |
| T-Substrate ValueBody-list/sum + std.unicode | L | BRIEF LANDED (`t-substrate-valuebody-list-worker.md`, PR #790 merged 2026-04-25) | top-level list/sum literal lowering + bootstrap/load-set |
| B4.1 DeclarationRef consumer migration | M | LANDED (PR #826 merged 2026-04-26; first consumer migration complete after prior B4.1/B4.1a brief landing). | existing carrier consumer migration |
| B4.2 fold-shape carrier | S | BRIEF AUTHORED (`b4-2-structural-fold-shape-carrier-worker.md`; PR #836 merged 2026-04-26) | structural fold-eligibility query/carrier decision |
| B4.3 emit-helper carrier | S | LANDED (PR #824 merged 2026-04-26) | typed role marker on Bind/Branch nodes |
| B4.4 extdeps-fixture-set carrier | S | LANDED (PR #825 merged 2026-04-26) | typed extdeps-bootstrap-set declaration |
| B4.5–B4.12 Phase 2 site dissolutions | S each | QUEUE AUTHORED (`b4-phase-2-site-dissolution-queue.md`); implementation briefs dispatch as Phase 1 carriers land. | mechanical consumer migration per site |

## Cross-program dependencies

**Produces (4 carrier-readiness signals):**
- Cardinality-for-int-lit → Modeling Manager (int-lit)
- Nominal-opaque-for-Secret → Modeling Manager (Secret<T>)
- Parametric-algebra-for-Dimensions → Modeling Manager (Dimensions)
- ValueBody-list/sum + std.unicode → Modeling Manager (charclass phase-2) + Grounding Manager (Engine sharpened-(b))

**Consumes:** none (Substrate is the substrate).

**Adjacent territory:**
- B4's §0.7 file-preference rank carrier touches Pure Bootstrap territory. Coordinate with Pure Bootstrap Manager.
- `kernel_algebra_profile` mirror dissolution is map-shaped (not list/sum) — tracked as future T-Substrate sub-lane requiring `ValueBody::Map` substrate work; PB Manager consumes when it lands.

## Pre-spawn vs post-spawn authority

- **Pre-spawn (now, before R1 close):** Director + PM coordinate on brief authoring per inbox #828 split. PM authors the manager skeleton (this file); Director authors any worker-level briefs not yet existing per the manager's "Pending" sub-briefs list. Both stop authoring once R2 spawns.
- **Post-spawn (R2 promotion onward):** Manager owns all worker-brief authoring autonomously per "Autonomous dispatch authority" below. Director's role narrows to cross-program conflict resolution + scope-change escalation.

## Autonomous dispatch authority

- Authors all T-Substrate + B4 sub-briefs without Director.
- Dispatches workers against all sub-briefs.
- Resolves Substrate-internal scope refinements; escalates blockers and scope changes to Director.
- Per `docs/r2-structure.md` P5 dispatch-discipline: every Substrate worker brief that introduces a scaffold names its dissolution trigger + adjacent ROADMAP debt row + contributes-or-defers stance.

## Reporting cadence

- Sub-lane / Phase close → R2 Release Manager (closure ledger).
- Cross-program signals (carrier-readiness for 4 carriers) → cross-manager queue (consuming managers ack and dispatch consumer-migration work).
- Blockers + scope changes → Director.

## Sub-briefs (authored / pending)

Authored:
- B4 program brief (this document is the orchestrator; B4 program brief is in [`b4-identity-carrier-substrate-pass.md`](b4-identity-carrier-substrate-pass.md))
- B4.1 DeclarationRef consumer migration ([`b4-1-declarationref-consumer-migration-worker.md`](b4-1-declarationref-consumer-migration-worker.md)) — brief landed PR #819; first-consumer migration landed PR #826.
- B4.2 structural fold-shape (`b4-2-structural-fold-shape-carrier-worker.md`) — authored PR #836; not yet landed as implementation.
- B4.3 structural emit-helper carrier (`b4-3-structural-emit-helper-carrier-worker.md`) — landed PR #824.
- B4.4 structural extdeps-fixture-set carrier (`b4-4-structural-extdeps-fixture-set-carrier-worker.md`) — landed PR #825.
- T-Substrate cardinality-for-int-lit (`t-substrate-cardinality-int-lit-worker.md`) — landed PR #806; `r2-substrate-cardinality-for-int-lit-subset.md` is a redundant routing/audit receipt, not a dispatch target.
- T-Substrate nominal-opaque-for-Secret (`r2-substrate-nominal-opaque-for-secret-subset.md`) — authored PR #836.
- T-Substrate parametric-algebra-for-Dimensions (`r2-substrate-parametric-algebra-for-dimensions-subset.md`) — closed by audit; producer work not needed.
- T-Substrate ValueBody-list/sum (`t-substrate-valuebody-list-worker.md`) — brief landed PR #790.

Pending — post-spawn manager-authored autonomously per "Pre-spawn vs post-spawn authority" subsection above:
- B4.2 implementation dispatch (brief exists; implementation not yet landed).
- B4.5–B4.12 Phase 2 implementation briefs that become live as Phase 1 carriers land; queue skeleton exists in `b4-phase-2-site-dissolution-queue.md`.
- Future T-Substrate sibling lanes explicitly excluded from the four R2 prereqs, including Int128/Word128 carrier widening and `ValueBody::Map`, when/if R2 scope admits them.

## Working state (fill on spawn)

Spawn refresh, 2026-04-26:

- T-Substrate: cardinality-for-int-lit producer landed (#806); nominal-opaque producer brief exists; parametric-algebra producer closed by audit because `phantom_params` already exists; ValueBody-list/sum brief landed (#790).
- B4 Phase 1: B4.1, B4.3, B4.4 landed; B4.2 brief exists and remains the immediate unlanded Phase 1 implementation lane.
- B4 Phase 2: queue exists; dispatch follows Phase 1 carrier disposition.

## Cross-refs

- Parent: `docs/r2-structure.md` §"Substrate Manager"
- Program brief: `docs/briefs/b4-identity-carrier-substrate-pass.md`
- Synthesis source: `docs/briefs/debt-paydown-synthesis-2026-04-25.md` §0
- Substrate design: `docs/design-substrate-carrier-port-program.md`
- Adjacent: `docs/briefs/b4-1-declarationref-consumer-migration-worker.md` (landed PR #819; first-consumer migration tracked at PR #826)
