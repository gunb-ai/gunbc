# R3 — PB-Runtime test-predicate dissolution hook (PB share of `test_runner.rs` predicate-language growth freeze)

**Status:** PROPOSAL — docs-only PB-side packet. **Pairs with** Evaluator Manager's bespoke-arm freeze on `src/v3/compiler/src/test_runner.rs::run_claim`. This brief defines **where** new `TestPredicate` evaluation lands so freeze-blocked authoring has somewhere to dissolve to; it does **not** define the freeze mechanism itself (Evaluator's lane).

**Authority parent (R3 ledger row):**
[`docs/debt/r3-debt-paydown-ledger-2026-05-02.md`](../debt/r3-debt-paydown-ledger-2026-05-02.md) row 83 — `test_runner.rs predicate-language growth` (Owner: R3 Evaluator + PB. Retirement shape: *Freeze bespoke arms unless paired with evaluator/PB-runtime dissolution hook*).

**Authority cross-reads (in-tree on `main`):**
- [`docs/briefs/r2-evaluator-test-runner-authority-ratchet.md`](r2-evaluator-test-runner-authority-ratchet.md) §3 (ratchet recommendation), §2.1 / §2.8 (PB-runtime split, structural-predicates row).
- [`docs/briefs/r2-pr-b-2-runner-extension-bundle.md`](r2-pr-b-2-runner-extension-bundle.md) "Runner authority discipline" + per-workstream dissolution table.
- [`docs/design-pb-runtime-interpreter.md`](../design-pb-runtime-interpreter.md) §3 (PB-Runtime IS), §6 (anti-bridge invariants), §7.1 (PB-Runtime equivalence fixture).
- `src/v3/compiler/src/test_runner.rs::run_claim` and `src/v3/std/verification.dag::TestPredicate` (read for grounding only — this brief does not modify either).

**Implementation request:** none. Docs-only.

---

## 1. Problem statement

`test_runner.rs::run_claim` is today a `match` over `TestPredicate` variants, each arm a bespoke Rust evaluator. Every new predicate adds a parallel test-truth authority alongside the eventual PB-Runtime / R2-Evaluator runtime. Evaluator Manager's ratchet (cross-read §3) freezes new bespoke arms unless a dissolution hook is named; without a PB-side landing point, the freeze either (a) blocks legitimate predicate work indefinitely, or (b) gets bypassed by reviewers who lack a structurally-clean alternative to point at.

This packet names the **landing point**. It is the PB-side counterpart to Evaluator's freeze: the place a new predicate's evaluation **structurally lives** instead of becoming a `match` arm.

## 2. Convergence model (what "PB-Runtime hook" means here)

Per `design-pb-runtime-interpreter.md` §2, PB-Runtime ≡ R2-Evaluator's runtime model expressed as a `.dag` program. They are not parallel runtimes. The "hook" therefore is **not** a new runtime, **not** a new predicate language, and **not** a Rust-side dispatch on a sentinel field. The hook is the structural rule:

> **A new `TestPredicate` variant is evaluable iff PB-Runtime's `evaluate` (the `.dag` program declared per `design-pb-runtime-interpreter.md` §3) can compose a value-domain answer for it from PB-Runtime's existing primitive evaluation rules.**

This is exactly the convergence direction the design lock already names (§3.1: "Anything beyond the 5 primitives is a downstream lens, not the interpreter"). The dissolution hook is not new vocabulary; it is **gating discipline** on where new predicate authoring lands.

Concretely, this means a new predicate's evaluation either:

- **(a) Folds over PB-Runtime evaluation outputs** — the predicate is a `Lens<C>`-shaped structural fold over `Value` results PB-Runtime already produces (per `design-pb-runtime-interpreter.md` §3.5 "reflection vs evaluation"; §1 "consumed by lenses *via the Evaluator*"). The runner arm then dispatches to the lens, not to a bespoke evaluator. *No new runtime mechanism is added; the predicate becomes a consumer of an existing one.*
- **(b) Folds over reflected (static) program structure** — the predicate is a structural projection per `design-reflection-completeness.md`, not an execution. The runner arm dispatches to the reflection lens. *Static, no PB-Runtime invocation needed; still no bespoke evaluator.*
- **(c) Routes to substrate-fact-introduction (P1)** — neither (a) nor (b) applies because the predicate semantics requires a primitive PB-Runtime cannot express via its 5-primitive base. This is a substrate carrier change, not a runner arm; it follows `INVARIANTS.md` §P1 and lands in `verification.dag` / substrate before any runner code is written. *The hook for case (c) is "STOP and route", not "add an arm".*

Cases (a) + (b) are the dissolution path. Case (c) is the explicit STOP+PING. There is no fourth case; specifically, "open a new bespoke `match` arm pending future cleanup" is **not** a case under the freeze.

<a id="pb-runtime-dissolution-hook-qualification"></a>

## PB-Runtime Dissolution Hook Qualification

> **Stable citeable anchor.** Evaluator Manager's freeze template/docs cite this section via the explicit HTML anchor `#pb-runtime-dissolution-hook-qualification` immediately above the heading (independent of GitHub's auto-generated slug). The anchor id, heading text, and the four-criterion / disqualifier shape below are stable; renames require Evaluator + PB Manager coordination.

A future PR adding or extending a `test_runner.rs::run_claim` predicate arm or producer path **qualifies as PB-runtime-routed** (and is therefore freeze-allowed) iff **all** of the following are cited in the PR description and present in-tree at PR landing:

### Q1 — Convergence-case classification

The PR cites which of the three cases in §2 the predicate falls into:

- **(a) Value-domain fold over PB-Runtime evaluation result.** The arm dispatches to a `Lens<C>`-shaped fold that consumes a `Value` produced by PB-Runtime's `evaluate` (per `docs/design-pb-runtime-interpreter.md` §3). The arm is a thin adapter; it does not contain bespoke evaluation logic.
- **(b) Reflection-only structural projection.** The arm dispatches to a structural query per `docs/design-reflection-completeness.md`. No PB-Runtime invocation; no execution. Static fold over reflected program structure.
- **(c) P1-routed substrate gap.** Neither (a) nor (b) applies because the predicate semantics requires a primitive PB-Runtime cannot express via its 5-primitive base (`docs/design-pb-runtime-interpreter.md` §3.1 — `Node` / `Conj` / `Disj` / `Cardinality` / `Bit`). **(c)-classified PRs do not land runner code in the same PR**; they land a P1 substrate-fact-introduction escalation row first.

PRs that cannot place the predicate in (a), (b), or (c) are not freeze-allowed. There is no "(d) bespoke pending cleanup".

### Q2 — Dissolution declaration reference

For (a) and (b), the PR cites a `.dag` declaration ref by symbol/path that is the runner arm's dispatch target — either:

- **Existing on `main`** at PR open time (preferred), or
- **Landing in the same PR** as the arm.

The runner arm's body is the citation site (e.g., `// dissolves through: <decl ref>` with the cited symbol). Bare prose pointers, ROADMAP rows, or cross-brief mentions do **not** satisfy Q2 — the citation is to a structural declaration, not to documentation.

For (c), the citation is the P1 escalation row (issue/PR/ledger row authored by Substrate Manager); runner code is deferred to after the P1 disposition lands and the row converts to (a)/(b).

### Q3 — Convergence claim (case (a) only)

For (a)-classified arms, a `TestClaim` of the shape locked in `docs/design-pb-runtime-interpreter.md` §7.1 (PB-Runtime equivalence fixture: `DifferentialEquals` between PB-Runtime `evaluate` and R2-Evaluator) covers the predicate's semantics. The claim either:

- **Exists on `main`** for an equivalent program shape consumed by the predicate, or
- **Lands in the same PR** as the arm (per-row authoring is parallelizable; the dispatch is the §3.1 table row's worker).

The claim's existence is what makes the arm's evaluation structurally accountable; Q3 closure is the per-row dissolution receipt.

### Q4 — §3.1 dissolution-table row

The §3.1 per-predicate dissolution table (PB-owned; see §3) has a **row for the variant** with columns populated:

- Variant constructor name (per `src/v3/std/verification.dag::TestPredicate`).
- Convergence case (a/b/c) matching Q1.
- Dissolution declaration ref matching Q2 (or P1 row for (c)).
- Status today (bespoke / partially-dissolved / fully-dissolved / P1-pending).

A missing or unfilled row means the predicate is not freeze-allowed regardless of Q1/Q2/Q3 claims in the PR description. The table is the **single source of truth**; Evaluator's freeze mechanism reads it.

### Disqualifiers (explicit non-criteria)

The following do **not** qualify as PB-runtime dissolution hooks even if cited:

- A new `TestPredicate` variant introduced by the PR. Substrate change → P1 → Substrate Manager. Not in scope of any runner PR.
- A new `Value` variant in PB-Runtime. Anti-bridge invariant #2 (`docs/design-pb-runtime-interpreter.md` §6) → P1.
- A new `TestPredicate` discipline-marker field (e.g., `pb_runtime_evaluable: Bit`). Substrate change → P1.
- A new producer identity in `test_runner.rs` (per Evaluator's freeze constraint). Producer dispatch lands only via the W1/W3 paths already locked in `docs/briefs/r2-pr-b-2-runner-extension-bundle.md`.
- A new substrate observation/channel carrier. Routes to P1 per `r2-pr-b-2-runner-extension-bundle.md` W3 "Structural observation authority".
- "Convention-only" runner observation (regex over stdout, ad-hoc parsing, env-variable signaling). Forbidden per `r2-pr-b-2-runner-extension-bundle.md` runner-authority discipline.
- A bespoke `match` arm whose dispatch target is itself in the same arm (self-citation). Q2 requires a structural declaration outside the arm body.

### Reviewer rule (one-line summary)

A `test_runner.rs` predicate-arm or producer-path PR is **PB-runtime-routed** iff Q1+Q2+Q4 hold (and Q3 for (a)). Anything else is **uncited bespoke runner growth** and the freeze blocks it. If a reviewer or worker concludes the predicate cannot satisfy Q1–Q4 without one of the disqualifiers, **STOP+PING** to PB Manager + Evaluator Manager rather than smuggling a carrier.

---

## 3. Hook shape (PB-owned landing point)

The PB-side artifact is a **convergence-discipline contract**, not a new piece of vocabulary. Its three elements:

### 3.1 Per-predicate dissolution table (single authority)

A single table keyed by `TestPredicate` variant naming, for each variant:

| Column | Content |
|---|---|
| Variant | `TestPredicate` constructor name (per `src/v3/std/verification.dag`). |
| Convergence case | (a) value-domain fold over PB-Runtime result, (b) reflection-only fold, or (c) P1-routed (substrate gap). |
| Dissolution declaration ref | If (a)/(b): the lens / structural query the runner dispatches through, declared in `.dag`. If (c): the P1 escalation row. |
| Status today | Bespoke runner arm / partially-dissolved / fully-dissolved / P1-pending. |

This table is the **single authority** for "is this predicate's evaluation structurally accounted for"; it consumes (does not duplicate) the per-workstream dissolution table at `r2-pr-b-2-runner-extension-bundle.md` "Runner authority discipline" and the inventory at `r2-evaluator-test-runner-authority-ratchet.md` §2.

**Where it lives:** appended to `docs/briefs/r2-evaluator-test-runner-authority-ratchet.md` as §6 (or as a sibling `r3-test-predicate-dissolution-table.md` cited from there). PB Manager owns the table; Evaluator Manager consumes it as the freeze allow-list. **Owner pick is a follow-up coordination, not locked here** — see §7.

The table is **the** authority; if a row is missing or marked (c) without a P1 row, the predicate is not freeze-allowed. There is no ad-hoc reviewer override.

### 3.2 PB-Runtime convergence test-claim discipline

For each predicate marked (a), the dissolution receipt is a `TestClaim` of the same shape as `design-pb-runtime-interpreter.md` §7.1's PB-Runtime equivalence fixture: PB-Runtime evaluates the predicate's program through its `.dag` `evaluate`, R2-Evaluator (Rust) evaluates the same program, and a `DifferentialEquals` claim asserts result equality. The claim's existence is the *proof* that the predicate is structurally evaluable; the runner arm becomes a thin adapter (or retires entirely once PR-B.1's eager evaluator is the runner's evaluation backend).

This is **the same anti-bridge invariant #1** from the design lock: "the `.dag` declaration of PB-Runtime IS the spec the Rust crate's tests verify against". The hook does not invent a new convergence claim; it consumes the one already declared.

### 3.3 Reviewer rule (the freeze's allow-condition, PB-side)

A PR adding or modifying a `TestPredicate` evaluation lands iff it satisfies the **PB-Runtime Dissolution Hook Qualification** section above (Q1–Q4 + non-disqualifiers).

This is the PB-side allow-condition for Evaluator's freeze. Evaluator's freeze mechanism (docs/template enforcement, per Evaluator coordination) cites the qualification section by name and checks the §3.1 table; PB owns the qualification text + the table. If docs/template enforcement turns out to be insufficient at the freeze layer, Evaluator STOP+PINGs rather than smuggling a carrier — the qualification stays the contract on PB's side.

## 4. Sequence (sequential vs parallelizable)

| Step | Owner | Sequence |
|---|---|---|
| §3.1 table seeded with current `TestPredicate` variants (consume existing inventories — no new analysis) | PB Manager (this brief's follow-on) | After this brief lands |
| Evaluator freeze mechanism wired against §3.1 table | Evaluator Manager | After §3.1 table lands; can author in parallel against an empty placeholder, but freeze goes live only when table is non-empty + reviewed |
| Per-variant dissolution-ref authoring (predicate-by-predicate; §2 cases (a)/(b)) | Evaluator + PB workers, dispatched per row | Parallelizable across rows once §3.1 lands |
| §3.2 PB-Runtime equivalence claim per (a) variant | Evaluator (claim authoring) + PB (`.dag` evaluate bodies) | After PR-B.1 eager evaluator (already merged per `r2-pr-b-2-runner-extension-bundle.md` parent designs) + per-variant dissolution refs |
| Bespoke arm retirement (per row) | Evaluator | After §3.2 claim is green for the row |
| (c)-routed P1 escalations | Substrate Manager | Triggered per row; not on this brief's critical path |

The whole hook is docs + table; nothing in this packet requires a substrate or runner edit before landing.

## 5. Acceptance criteria (this brief)

- ✅ The PB-side landing point is named structurally (§2 convergence cases) without inventing new vocabulary.
- ✅ The hook artifact (§3.1 table) is identified, with location, owner, and authority relationship to existing inventories.
- ✅ §3.2 cites the existing PB-Runtime equivalence fixture rather than authoring a new convergence claim shape.
- ✅ The PB-vs-Evaluator boundary is explicit (§7).
- ✅ STOP+PING conditions are enumerated (§6) and cover every shape change this brief must not silently make.
- ✅ Sequence (§4) separates sequential dependencies from parallelizable per-row work.

## 6. STOP+PING boundary (binding)

This brief does **not** propose, and any worker concluding any of the following is required must STOP and re-dispatch through the appropriate authority before drafting code or amending this brief:

- **A new `TestPredicate` variant.** Substrate change. Routes to Substrate Manager via `INVARIANTS.md` §P1 (same routing as `Distributivity` in `r2-pr-b-2-runner-extension-bundle.md`).
- **A new `TestPredicate` discipline-marker field** (e.g., `pb_runtime_evaluable: Bit` on the variant). Substrate change. Same routing. The convergence case in §3.1 is a *table column*, not a runtime-typed field.
- **A new `Value` variant** in PB-Runtime. Anti-bridge invariant #2 of `design-pb-runtime-interpreter.md` §6 — escalates to Substrate Manager via P1.
- **A new bespoke runner arm landed before the §3.1 row exists.** That is exactly what the freeze blocks; the hook is the allow-condition, not a bypass.
- **A parallel PB-Runtime evaluation surface that diverges from R2-Evaluator's semantics.** Anti-bridge invariant #1; convergence is non-optional.
- **A second test-predicate language in `.dag`** (e.g., a "pb_runtime_predicate" sub-vocabulary distinct from `TestPredicate`). PB-Runtime is `.dag`; predicates remain `TestPredicate` inhabitants per `verification.dag`. §3.1's case-tagging is metadata about *where evaluation lives*, not a parallel predicate type.

If freeze semantics turn out to require any of the above, the dispatch back to Director/Evaluator is: **STOP + name the missing substrate primitive**, do not invent a convergence shape locally.

## 7. PB vs Evaluator ownership split

| Owns | PB Manager | Evaluator Manager |
|---|---|---|
| §3.1 dissolution table (location, schema, per-row authoring) | ✅ | — |
| Per-variant convergence-case classification (a/b/c) | ✅ (with Evaluator review) | review |
| §3.2 PB-Runtime `.dag` `evaluate` bodies for each (a) row | ✅ (consumed from `design-pb-runtime-interpreter.md` §3) | review |
| §3.2 R2-Evaluator (Rust) side of the `DifferentialEquals` claim | — | ✅ |
| Freeze mechanism implementation (CI / `match`-arm count ratchet / equivalent) | — | ✅ |
| Reviewer rule enforcement at PR time | shared | shared |
| (c)-routed P1 escalation authoring | shared (per row) | shared (per row) |
| New `TestPredicate` variants | — | — (Substrate via P1) |
| Bespoke arm retirement PRs (per row, after §3.2 green) | — | ✅ |

PB does **not** author Evaluator's freeze mechanism. Evaluator does **not** invent PB-Runtime semantics. Both consume `design-pb-runtime-interpreter.md` as the seam.

The §3.1 table location (sibling brief vs §6 of `r2-evaluator-test-runner-authority-ratchet.md`) and exact ownership of cross-row coordination is a follow-up Director call when the seeding worker dispatches; this brief locks the *contract*, not the file path.

## 8. R3 debt receipt (row 83)

| Field | Value |
|---|---|
| Row | `test_runner.rs predicate-language growth` (ledger row 83) |
| Disposition | **Debt found + routed (partial — docs-only landing).** |
| Receipt PR | This brief landing (PB-side hook contract). |
| Remaining work | (1) §3.1 table seeded with existing variants (per-row dispatch). (2) Evaluator freeze mechanism wired to the table. (3) Per-variant dissolution refs. (4) §3.2 convergence claims. (5) Bespoke arm retirements per-row. Steps (1) + (2) close the row's *retirement-shape clause* ("Freeze bespoke arms unless paired with evaluator/PB-runtime dissolution hook"). Steps (3)–(5) are the long-tail implementation that the freeze gates. |
| Sibling rows touched | None paid by this PR. Adjacent rows that converge on the same hook (row 75 `test_runner.rs filename / sentinel bridges`; row 92 `Emitter as_bind().expect()` parked behind Substrate #1548) are noted but not closed here. |
| Velocity tripwire (`INVARIANTS.md` P5(c)) | This brief introduces zero new tracked-debt rows; no introduction:dissolution ratio impact. |

## 9. Related

- `docs/debt/r3-debt-paydown-ledger-2026-05-02.md` row 83 + cross-references to row 75 and row 92.
- `docs/briefs/r2-evaluator-test-runner-authority-ratchet.md` (audit + ratchet recommendation).
- `docs/briefs/r2-pr-b-2-runner-extension-bundle.md` (per-workstream dissolution table; runner-extension discipline).
- `docs/design-pb-runtime-interpreter.md` §3 / §6 / §7.1 (PB-Runtime IS, anti-bridge invariants, equivalence fixture).
- `docs/design-reflection-completeness.md` §3 / §6 (reflection-vs-evaluation distinction; consumed by §2 case (b)).
- `INVARIANTS.md` §P1 (substrate-fact-introduction; consumed by §2 case (c) and §6 STOP+PING boundaries).
