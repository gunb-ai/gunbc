# R2 Release — B6 file-preference rank checklist completion `(XS, trivial)`

> **R2 Release Manager dispatch.** Per [`docs/briefs/debt-paydown-synthesis-2026-04-25.md` §"Tier 2"](debt-paydown-synthesis-2026-04-25.md) item 6 + [`docs/r2-structure.md` §"R2 Release Manager"](../r2-structure.md). XS-trivial; one-line ROADMAP / source fix. Pre-spawn authoring per inbox #828 PM portion.

## Read first

- **[`docs/briefs/debt-paydown-synthesis-2026-04-25.md` §"Tier 2" item 6 (lines 315-318)](debt-paydown-synthesis-2026-04-25.md)** — parent scope. Quoted: *"`src/v3/std` vs `dsl/std` checklist undercount fix (PR #809 entry). One-line fix to `dag.rs:2735-2764` + mirror in `lower.rs`. Or surface as 'explain why these three modules are exempt.' Belongs to whoever owns the file-preference rank scaffold; trivial dispatch."*
- **`src/v3/compiler/src/dag.rs:2735-2764`** — `declaration_name_preference_rank` function. The location of the checklist undercount.
- **`src/v3/compiler/src/lower.rs:1451-1452, 1546-1547`** — mirror call sites consuming the rank.
- **PR #809** — original entry naming the three modules (`computation` / `induction` / `termination`) as missing from the file-preference-rank checklist.
- **[`MODELING.md`](../../MODELING.md)** + **[`INVARIANTS.md`](../../INVARIANTS.md)** + **[`CODING.md`](../../CODING.md)**.

## Frame — two paths, pick by audit

The synthesis doc gives two valid resolutions:

1. **Add the three missing modules** to the file-preference-rank checklist if they belong there. Direct one-line fix.
2. **Document the exemption** — if `computation` / `induction` / `termination` modules are intentionally exempt, surface the reason structurally (comment / doc) so the undercount is visibly intentional rather than missed.

The brief is audit-first (per `feedback_audit_adjacent_authority_first`): determine which path applies by reading `dag.rs:2735-2764` + the synthesis doc + the three modules' surface posture before editing.

## Two consumer-side requirements

1. **Audit the rank function.** Read `dag.rs:2735-2764`. Determine: (a) what the rank scaffold is checking; (b) what the three named modules' role is in the file-preference rank semantics; (c) whether they belong in the checklist or are intentionally exempt. Cite the rank-scaffold's existing dissolution trigger from the synthesis doc.

2. **Land the resolution:**
   - **If they belong:** add the three module names to the checklist; verify mirror in `lower.rs` if relevant; gate verification clean.
   - **If exempt:** add a one-line comment at `dag.rs:2735-2764` naming why the three modules are exempt (e.g., "computation/induction/termination are runtime-mirror modules that don't participate in user-surface preference resolution"). Document in the source, not just in the brief.

## Slice — audit + one-line fix

Single PR. Either `+3 lines` (additions) or `+1 line` (comment + cite). Gate verification + DB-8 fixed-point convergence.

## Acceptance

- [ ] Audit captured in PR description (one paragraph: which path applies + why).
- [ ] Resolution landed: 3-line addition OR 1-line comment cite.
- [ ] `cargo test --workspace --exclude v2-compiler-tests` clean.
- [ ] DB-8 fixed-point converges bit-identically.
- [ ] Synthesis-doc Tier 2 §6 row updated to RESOLVED.

## STOP-AND-ESCALATE

Per [`docs/escalation-paths.md`](../escalation-paths.md):

- **Audit reveals the rank function semantics are unclear** (e.g., the three modules' role is genuinely ambiguous — neither "belongs" nor "exempt" cleanly applies) → STOP. Surface for design clarification rather than guess.
- **The "fix" turns out to require substrate work** (e.g., the rank scaffold is part of a larger structural pattern that needs dissolution rather than line-edit) → STOP. Escalate per `feedback_construction_over_ratchets` — don't ratchet on a scaffold the synthesis doc already names as a future dissolution candidate.

## Cross-refs

- Parent: [`docs/briefs/debt-paydown-synthesis-2026-04-25.md` §"Tier 2" item 6](debt-paydown-synthesis-2026-04-25.md).
- Source: `src/v3/compiler/src/dag.rs:2735-2764` (rank function); `src/v3/compiler/src/lower.rs:1451-1452, 1546-1547` (consumers).
- R2 Release Manager scope: [`docs/r2-structure.md` §"R2 Release Manager"](../r2-structure.md) B-wave Tier 2.
- Originating issue: PR #809.
- Discipline anchor: `feedback_audit_adjacent_authority_first` + `feedback_construction_over_ratchets`.
- Escalation discipline: [`docs/escalation-paths.md`](../escalation-paths.md).
