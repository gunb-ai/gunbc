# R2 Substrate Manager Brief

**Status:** PROPOSAL (per [`docs/r2-structure.md`](../r2-structure.md), LIVE 2026-04-26 via PR #827). Spawns on R1 close per Transition mechanics step 4. NEW manager — no migration source.

## Orient before reading

- **R2 structure authority:** [`docs/r2-structure.md`](../r2-structure.md). Names this manager one of 6 standing R2 managers; **largest single concentration of R2 work**.
- **Program scope sources:**
  - T-Substrate: [`docs/r2-structure.md` §"Goals" item 3](../r2-structure.md) (4 substrate prereq sub-lanes).
  - B4 Identity-Carrier Substrate Pass program: [`docs/briefs/b4-identity-carrier-substrate-pass.md`](b4-identity-carrier-substrate-pass.md).
- **Cross-program producer:** all four T-Substrate sub-lanes produce carriers consumed by Modeling Manager (3 sub-lanes) + Grounding Manager (Engine sharpened-(b) consumes ValueBody-list/sum).
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

| Sub-lane | Size | Status (at brief authoring) | Carrier shape |
|---|---|---|---|
| T-Substrate cardinality-for-int-lit | M | NOT YET AUTHORED | cardinality refinement narrowing |
| T-Substrate nominal-opaque-for-Secret | M | NOT YET AUTHORED | nominal-type construction restriction |
| T-Substrate parametric-algebra-for-Dimensions | M | NOT YET AUTHORED | parametric algebra attachment (DB-18 territory; tag mismatch with db-history flagged) |
| T-Substrate ValueBody-list/sum + std.unicode | L | DISPATCHED (worker brief #790) | top-level list/sum literal lowering + bootstrap/load-set |
| B4.1 DeclarationRef consumer migration | M | DRAFTED (with §0.2 BLOCKING outstanding — codex finding on PR #819) | existing carrier consumer migration |
| B4.2 fold-shape carrier | S | NOT YET AUTHORED | template-formal edge identification |
| B4.3 emit-helper carrier | S | NOT YET AUTHORED | typed role marker on Bind/Branch nodes |
| B4.4 extdeps-fixture-set carrier | S | NOT YET AUTHORED | typed extdeps-bootstrap-set declaration |
| B4.5–B4.12 Phase 2 site dissolutions | S each | NOT YET AUTHORED (skeletons only — full content waits for Phase 1 carrier landing) | mechanical consumer migration per site |

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
- B4.1 DeclarationRef consumer migration ([`b4-1-declarationref-consumer-migration-worker.md`](b4-1-declarationref-consumer-migration-worker.md)) — **with outstanding BLOCKING** (codex §0.2 scope gap on PR #819; resolution pending)
- T-Substrate ValueBody-list/sum (PR #790 worker brief)

Pending — Director-authored per coordination on inbox #828:
- T-Substrate sub-lane scoping briefs (3): cardinality-for-int-lit, nominal-opaque-for-Secret, parametric-algebra-for-Dimensions
- B4.2, B4.3, B4.4 Phase 1 carriers
- B4.5–B4.12 Phase 2 site dissolution skeletons

## Working state (fill on spawn)

Sub-lane / Phase status table refreshes here as work lands. Pre-spawn placeholder.

## Cross-refs

- Parent: `docs/r2-structure.md` §"Substrate Manager"
- Program brief: `docs/briefs/b4-identity-carrier-substrate-pass.md`
- Synthesis source: `docs/briefs/debt-paydown-synthesis-2026-04-25.md` §0
- Substrate design: `docs/design-substrate-carrier-port-program.md`
- Adjacent: `docs/briefs/b4-1-declarationref-consumer-migration-worker.md` (with BLOCKING)
