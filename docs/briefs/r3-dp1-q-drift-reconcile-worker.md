# DP1 — Q-Drift-Reconcile worker brief `(S, R3-standing)`

**Dispatch:** `docs/r3-design-schedule-2026-05-06.md` §6 **DP1** + `docs/r3-program-plan.md` §10.1 **Q7** + §533 drift normalization (Grounding Mgr poke-hole 2026-05-06 finding 5).  
**Goal:** one reconciliation PR that retires **ledger↔authority drift** for three named strands — no new substrate behavior unless a strand exposes an execution gap (then STOP-and-escalate per `r3-debt-paydown-program-coordination.md`).

## Read first

- `docs/debt/r3-debt-paydown-ledger-2026-05-02.md` — catalog rows to reconcile (especially rows touching **`declaration_by_name`**, **CollectionOps/StringOps/MapOps**, transitional fence).
- `ROADMAP.md` — **§Tracked debts** row for `declaration_by_name` (RETIRED **PR #1638**).
- `docs/r3-program-plan.md` §533–536 — drift item definitions (same three strands this brief bundles).
- `docs/briefs/collectionops-algebra-reframe.md` — **partial** Phase-1 work on CollectionOps; DP1 **does not** redo that program — it **aligns ledger text** with ROADMAP + plan for the duplicate-surfaces row and cites this brief where Partial → Open/Partial is adjusted honestly.

## Scope — three strands in one PR

### Stratum A — `declaration_by_name` ROADMAP↔ledger

- **Facts:** ROADMAP records emit-site retirement (**PR #1638**); ledger row still **Open** (`declaration_by_name(...) emit pattern`).
- **Work:** flip ledger row to **Retired** (cite **#1638**, ratchet name `emit_production_code_has_no_declaration_by_name_calls` if useful). No code unless reviewer proves drift in ratchet vs ROADMAP — then file substrate-gap note; do not “fix” without lane owner.

### Stratum B — **#1499** transitional fence ledger gap

- **Facts:** ROADMAP transitional-fence narrative moved; ledger missing aligned row or status.
- **Work:** single-row reconciliation — either add ledger paragraph pointing at ROADMAP anchor + merge receipt, or mark **N/A** with grep-proof that no predicate-passing row remains. Cite **#1499** in PR debt-receipt section.

### Stratum C — CollectionOps / StringOps / MapOps ledger refresh

- **Facts:** ledger row **Partial (fold)** with open **concat/length/map** + StringOps/MapOps; design continuation in `collectionops-algebra-reframe.md`.
- **Work:** update ledger row text so **status**, **remaining fields**, and **pointer to collectionops brief** match ROADMAP **CollectionOps/StringOps/MapOps** duplicate-surfaces row + plan §533 wording — **documentation reconciliation**, not the algebra migration itself.

## Non-scope

- DP2 (**#1807** / SG-0 CI gate), DP3 tripwire, DP5 (**#1566**) — separate dispatches.
- Tier-2 ROADMAP narrative edits beyond the three strands unless a citation chain requires a one-line pointer fix in the same PR.

## Acceptance

- [ ] Exactly **one** merged PR whose **Debt receipt** names each stratum **(1) Debt paid** with path + anchor.
- [ ] `docs/debt/r3-debt-paydown-ledger-2026-05-02.md` rows consistent with ROADMAP + plan for strata A–C.
- [ ] No open **internal contradiction** between ROADMAP “RETIRED #1638” and ledger for `declaration_by_name`.
- [ ] `cargo fmt` / `cargo clippy` unchanged expectation for docs-only PR (if diff touches only `docs/`).

## Dispatch trigger

**Now** (Q7 RATIFIED-by-default). **Worker pin:** any idle Debt-Paydown worker session; **Mgr coordination:** quiet-otter-416.

## Blocking conditions

None for authoring — execution waits merge approval only. Cross-lane substrate work **does not** gate this slice (ledger honesty).
