# Placement study: accuracy of meaning, externalization, and the quality floor

**Status: analysis and source copy only — NOT APPLIED.** The operator ruled three concepts DESIGN-worthy on 2026-08-30, developed in gunbc-private (`strategy.pricing_decomposition` and its study memo). This document argues where each lands in the serially-reasoned DESIGN.md, enumerates exactly which artifacts change, and ends with the drafted insertion text. It does **not** edit `gunbc.design_document`, `gunbc.recurring_failure_mode`, or any projection: the edit is a root-impacting authority change performed under operator supervision, starting from the copy below. The corpus-wide violation audit is likewise carved out to the operator.

**§6 reviewer test, answered up front:** this artifact is the supervised edit's source copy — it is consumed by the terminal architecture as the exact text and section coordinates of that edit, and it survives afterward as the placement-reasoning record for the three paragraphs and two ledger rows it seats. Its dissolution condition is the bind row registered beside it: the plan retires when the supervised edit lands its content on the authorities named in §C below.

## A. The authorities and the projection pipeline, as read

- `DESIGN.md` is a projection of `gunbc.design_document`: `design_blocks` concatenates one block-producing function per section (`section_1_blocks` … `section_7_blocks`, `failure_modes_blocks`, `building_checks_blocks`), `expected_design_md` serializes it, and `design_drifted` is the byte comparison. The failure-mode index in the final section is `map(recurring_failure_mode_roster, …)` over `gunbc.recurring_failure_mode` — it grows by construction when the roster grows; no index edit exists.
- `docs/design-ledgers.md` is the peer projection `gunbc.design_ledgers` (`expected_design_ledgers_md`), rendering each roster row's `authored` field in full, in roster order (order is the content-preservation oracle; rows append at the end).
- The committed files are adjudicated against these authorities by the generated-artifact phase: `gunbc.generated_artifact` locates `DesignArtifact` at `./DESIGN.md`, and `gunbc.generated_artifact_boundary_seed_growth` records why the comparison leaf is required-floor-enrolled. Regeneration is the wet generated-artifact actuator (the route `gunbc.generated_artifact_emit` serves), never a hand edit.
- So the supervised edit touches exactly two authorities — `gunbc.design_document` (three paragraphs) and `gunbc.recurring_failure_mode` (two rows) — and regenerates two projections. `gunbc.design_ledgers` and `gunbc.rung_drop` are untouched.

## B. Placement analysis

DESIGN.md's charter is serial: each section is a consequence of the ones before it, never a restatement. Each concept therefore gets **one home**, chosen by which section's existing law it is a consequence of.

### B1. Accuracy of meaning → §3, one paragraph (AGREE with the candidate mapping)

§3's recurring violation is nicknaming — *two names for one concept*. The meaning fork is its exact dual — *one name for two concepts* — and both violate the same law read in opposite directions: single authority is the demand that the name↔meaning relation be a bijection. A fork fails the function direction (one name, two referents); a nickname fails the injective direction (two names, one referent). §3 already carries half the generalization ("keep its real names," "model what the API actually returns"); what is new is (i) naming the dual explicitly and (ii) extending the law's scope to every layer that names — including product names, where the reader resolving the ambiguity is the customer.

Why not §2: the fork's *cost* is §2's deferred-cost sentence, but its *law* is about authority of names, which §3 owns. Why not a standalone section: it is a one-paragraph consequence of §3's first paragraph, exactly the grain of §3's existing standing rules. The two honest arms (falsifiable opacity; named degradation) are pointed at, not stated here — they land in §4b and §5 respectively, keeping each fact in one place.

### B2. Externalization → §5 named trap, single home (AGREE it is §5; REFUTE stating it in §2 as well)

The brief asks whether externalization is a §2 consequence stated once, a §5 trap, or both. **Both is refuted by the document's own charter**: §2 already contains "redundant work defers cost … shoves a problem onto a later fixer," and re-stating it with the fixer outside the firm would be a restatement, which the serial structure forbids. What is genuinely *new* is not the cost claim but the refusal discipline — and refusal disciplines are §5's genre.

Structurally, externalization **is the absorbing fallback with a counterparty as the absorber**. §5's third trap names the arm that widens instead of refusing when a mechanism cannot compute its precise answer; externalization is the arm that, when an obligation cannot be met at its price, pushes the unpriced residue onto a party outside the boundary — a risk the counterparty paid us to absorb quietly re-exported to them. It fails open in §5's own two ways: the degradation is silent (no typed, located diagnostic), and the detecting party is precisely the one with no line to stop. The two honest arms (absorb-and-reserve; name-and-price the degradation as its own product) are the boundary-crossing form of §5's existing rule that a failure arm must refuse, never widen. Placement: a paragraph immediately after the absorbing-fallback paragraph, before the no-escape-hatches corollary, with a one-clause lineage back to §2's later-fixer sentence (a citation, not a restatement).

### B3. The quality floor → §4b, one paragraph (AGREE with the rung-drop mapping, homed in §4b rather than §5)

The floor's content is measured against §4b's vocabulary, clause by clause: a quality claim with a billing/refusal consequence is a contract, without one it is marketing — that is §4b's rung-honesty obligation (a claim's rung equals what executed evidence establishes; a consequence-free claim is the "permanently green decoration … worse than absent because it will be cited as coverage"). Opaque dimensions admissible only over a falsifiable floor — that is the falsifiability demand ("is the check's RED authorable"): the floor is the belief "this dimension does not reach the delivered product" made refutable, with the refund as the executing consequence. And named degradation is §4b(3) verbatim, applied to service delivery: a delivery below the promised rung is admissible only declared, bounded, and priced as its own named product — deviation allowed, silence about deviation refused.

Why §4b and not §5: §5 gets the *refused arm* (externalization); §4b gets the *measure* that decides when the opaque arm is honest. Splitting them this way keeps each a consequence of its own section — the floor is how a promise's rung is established, the externalization trap is what firing past it silently is. Placement: one paragraph after the four meta-obligations, before the error-class filing paragraph, since it generalizes obligations 1 and 3 to a new subject grain (counterparty-facing promises) rather than adding a fifth.

### B4. Ledger classes: both belong (AGREE), with recognition rules

The roster's charter is "instances of §3–§5, kept for pattern-matching," one row per class with a recognition rule. Both qualify:

- **`meaning_fork`** — instance of §3. Distinct from `hollow_alias` (an alias that is minimal but ungrounded) and the complement of nicknaming. It also *subsumes the pattern* behind two standing rows without replacing them: `diagnostic_name_mechanism_silent` is this fork observed from the consumer side (one diagnostic name, several producing predicates), and the "witnessed" ambiguity named in the private audit (green vs rostered-red) is the same shape. Those rows keep their own recognition rules; the new row names the naming-law they instantiate.
- **`externalized_degradation`** — instance of §5, sibling of `absorbing_fallback` distinguished by *who absorbs*: the fallback widens inside the system (cost lands on the corpus/budget); externalization widens across a boundary (cost lands on a counterparty with no refusal channel). The distinction is load-bearing because the detection channels differ — a widened rerun shows up in the budget, an externalized degradation shows up in someone else's ledger, which is why it needs its own recognition rule.

Per `gunbc.recurring_failure_mode`'s own standing note, the `evidence` field stays empty (nothing resolves or renders it today); receipts live inside `authored`. The honest receipt situation: the executing evidence today is **private** — `strategy.pricing_decomposition` and its witness (`dag/test/claim/pricing_decomposition_witness_test.dag` in gunbc-private) — and the public-corpus specimen census is carved out to the operator. The rows below state this explicitly rather than fabricating public receipts; this is the `v1_maintenance_standing` pattern — vocabulary consumed by review diligence, rung stated honestly, no gate claimed.

## C. Literature census — exactly what changes

| Artifact | Change |
|---|---|
| `gunbc.design_document` `section_3_blocks` | +1 paragraph (P1 below), inserted after the opening nicknaming paragraph, before the "Two corollaries" paragraph |
| `gunbc.design_document` `section_4b_blocks` | +1 paragraph (P2 below), inserted after the four-meta-obligations list, before the "Every newly discovered error class…" paragraph |
| `gunbc.design_document` `section_5_blocks` | +1 paragraph (P3 below), inserted after the absorbing-fallback paragraph, before the no-escape-hatches corollary |
| `gunbc.recurring_failure_mode` | +2 rows (R1, R2 below), appended at the end of the module and of `recurring_failure_mode_roster` (roster is source-ordered; appending preserves the projection oracle) |
| `DESIGN.md` (projection) | regenerated: three new paragraphs; the failure-mode index grows by two bullets automatically via the roster map |
| `docs/design-ledgers.md` (projection) | regenerated: two new list items in the failure-mode section |
| `gunbc.design_ledgers`, `gunbc.rung_drop` | **unchanged** — the quality floor is not a rung drop and no drop is declared or retired |
| Other citers | none require edits. `docs/plans/replacement-migration-doctrine.md` and `docs/plans/scaffold-admission-doctrine.md` cite §3/§5 by role, not by paragraph position, and both survive. In gunbc-private, `strategy.pricing_decomposition` `accuracy_of_meaning_ruling` cites "the design doctrine's one-name-one-meaning law"; after the edit that citation gains a named paragraph and two ledger identities to point at — a follow-up tightening in the private repo, not a prerequisite |

Insertion positions are stated relative to named neighboring paragraphs inside the named functions (per §3's cite-the-symbol rule); the functions are small literal lists, so the positions are unambiguous.

## D. Downstream implications (analysis, not implementation)

**What becomes enforceable.** The meaning-fork row makes a class of lens derivable: a *meaning-fork lens* over name-carrying surfaces — any carrier where one name's delivered referent varies with an undisclosed condition. The adjacent existing instrument is `gunbc.bare_name_fork_lens` (bare-name identity forks); the new class is its semantic dual and would reuse the same Node-tree read. Nearer-term and cheaper: the row gives review a named rule for the recurring "one diagnostic, two mechanisms" and "one status word, two dispositions" findings that today get filed under three different classes. The externalization row makes the counterparty question a standard review probe on every failure/shortfall arm ("who bears the residue, and is the bearing typed, priced, refusable?") — the same three-question shape §5 already gives reviewers for line-stops.

**What comes under the higher bar.** Any public construction that carries a counterparty-facing claim without an executing consequence: support-policy and served-surface rows (the §3 example `gunbc.served_surface_browser_support` is the named join of policy and evidence — the floor paragraph is the bar such joins must meet), and any future pricing/product surface in the public repo. Claims about the *compiler's* own guarantees are already governed (§4b rung honesty); the floor paragraph extends the same bar to delivered-service claims so a "quality" sentence with no refund/refusal arm is reviewable as decoration rather than as prose.

**What the private repo already carries as executing evidence.** `strategy.pricing_decomposition` holds the typed constructions — `accuracy_of_meaning_ruling`, `quality_dimensions` (`GuaranteedFloor`/`OpaqueOurs`/`DisclosedPolicy`, one witness per kind), `floor_met`, the closed invoice-constructor sum with seven refused constructors, `risk_interfaces`, `moral_hazard_walls` — exercised by `pricing_decomposition_witness_test`. The five-gap externalization audit (resale condition representation, durability remedy, isolation floor, home-hosting obligation gate, KnownRed staleness) is the specimen backlog the `externalized_degradation` row will accrete receipts from as those lanes land. Honesty boundary, stated so the rows do not inflate: private witnesses are not enrollable evidence in the public repo — no public gate executes them — so both rows land at *review-diligence vocabulary*, exactly like `v1_maintenance_standing`, and say so.

---

## E. Proposed insertion text — DRAFTED, NOT APPLIED

The operator's supervised edit starts from this copy. Each block is the exact `p(text: …)` argument or roster row.

### P1 — `section_3_blocks`, after the nicknaming paragraph

> **The nickname's dual is the meaning fork — one name answering for two concepts** — and the same law forbids both, because single authority is the demand that the name↔meaning relation be a bijection, read in both directions. The fork prices like every fork: it pays the author's present convenience and defers the cost to whoever must resolve the ambiguity later — and where the name is a product name, that later fixer is the customer, paying in distrust and errors. So the law binds every layer that names, the commercial one included: a product name that simultaneously means full-strength delivery and a quietly thinned one is the nicknaming violation with the reader outside the firm. Two honest arms exist and only two, each stated where it lands: a dimension may stay hidden only where its irrelevance is falsifiable (the quality floor, §4b), or the different delivery gets its own name and price (the refused third arm is §5's externalization trap).

### P2 — `section_4b_blocks`, after the meta-obligations list

> The ladder does not stop at the compiler's edge: a promise delivered to a counterparty is an error class like any other, and its rung is set by the same test. A quality claim is **contracted** only when its floor is measured and its breach carries a typed consequence the promiser pays — a refund, a refusal, a named remedy; a claim with no executing consequence is the permanently-green decoration above, sold outward — marketing wearing a contract's name. Above a falsifiable floor, opaque dimensions are admissible and deliberately ours to manage — the floor is the belief "this dimension does not reach the delivered product" made refutable, with the remedy as its executing evidence. And a delivery that must sit below the promised rung follows meta-obligation 3 exactly: declared, bounded, and priced as its own named product, never a quiet mode of the premium name — deviation is allowed, silence about deviation is not.

### P3 — `section_5_blocks`, after the absorbing-fallback paragraph

> The same arm exists across the organization's boundary, and it is the fourth named trap: **externalization — the absorbing fallback whose absorber is a counterparty.** §2 already prices redundant work as cost shoved onto a later fixer; externalization is that sentence with the fixer outside the firm and unable to refuse — a risk a counterparty paid us to absorb, quietly re-exported to them; an unpriced cost pushed onto users, employees, suppliers, liquidation buyers, neighbors, or future maintainers. It fails open twice in this section's own terms: the degradation is silent, so the wrong thing passes with no typed, located diagnostic; and the party positioned to detect it is precisely the one with no line to stop. Two honest arms exist and only two: absorb the risk and hold the reserve that absorption costs, or name and price the degradation as its own product, so the counterparty can see it and decline it. The review tell is the boundary-crossing form of the one above: for every failure or shortfall arm, ask who bears the residue when the obligation cannot be met at its price — if the bearer is outside the boundary and the bearing appears nowhere as a typed, priced, refusable line, the arm is externalizing.

### R1 — `gunbc.recurring_failure_mode`, appended row

```
data meaning_fork: RecurringFailureMode = RecurringFailureMode {
  identity: "meaning_fork" as NonEmptyStr,
  authored: "meaning fork (the nicknaming violation's dual — one name answering for two concepts, where nicknaming is two names for one; §3 forbids both because single authority is the demand that name↔meaning be a bijection. Sharpest where the name crosses a boundary the reader cannot see behind: a product name meaning both full-strength and quantized-under-load, a diagnostic name raised by two unrelated predicates (`diagnostic_name_mechanism_silent` is this fork observed from the consumer side), a status word like \"witnessed\" meaning both green and rostered-red-with-an-excuse. Recognition rule: for a name in any carrier, ask whether its delivered referent is a function of the declared inputs alone; if the referent varies with an undisclosed condition — load, tier, time, caller — the name is forked, and the repair is either a floor that makes the hidden condition falsifiably irrelevant (§4b) or a second name with its own price (§5's named-degradation arm), never a footnote. Receipts: operator ruling 2026-08-30, modeled and witnessed in gunbc-private `strategy.pricing_decomposition` `accuracy_of_meaning_ruling`; public-corpus specimens pending the operator's carved-out census — until that census lands, this row is review-diligence vocabulary, and it says so rather than citing a gate that does not execute it.)",
  evidence: [],
}
```

### R2 — `gunbc.recurring_failure_mode`, appended row

```
data externalized_degradation: RecurringFailureMode = RecurringFailureMode {
  identity: "externalized_degradation" as NonEmptyStr,
  authored: "externalized degradation (the absorbing fallback with a counterparty as the absorber — when a mechanism cannot meet its obligation at its price, the unpriced residue is pushed onto a party outside the diff's accountability boundary: users absorbing silent quality thinning under a premium name, suppliers absorbing stretched terms, a later maintainer absorbing deferred debt. Distinguished from `absorbing_fallback` by who absorbs, and the distinction is load-bearing because the detection channels differ: a widened rerun surfaces in our own budget, an externalized degradation surfaces in someone else's ledger — the arm greens locally because the diagnostic channel belongs to the harmed party, who has no line to stop. Recognition rule: for every failure or shortfall arm, name who bears the residue; if the bearer is outside the firm or the module's declared boundary and the bearing appears nowhere as a typed, priced, refusable line item — a refund, a disclosure, a named cheaper product — the arm is externalizing. The two honest arms are absorb-and-reserve or name-and-price; a dissolution trigger denominated in the counterparty's patience is neither. Receipts: operator ruling 2026-08-30, modeled and witnessed in gunbc-private `strategy.pricing_decomposition` `risk_interfaces`, plus that repo's five-gap externalization audit as the specimen backlog; public-corpus specimens pending the operator's carved-out census — until then this row is review-diligence vocabulary, stated honestly.)",
  evidence: [],
}
```

### Not proposed, and why

- **No §2 edit** — the later-fixer sentence already stands; a second statement would be the restatement the preamble forbids.
- **No rung-drop row** — nothing is lowered; `gunbc.rung_drop` is untouched.
- **No new lens in this edit** — §6: construction and vocabulary first; a meaning-fork lens is a follow-up priced by displaced cost once the rows give it a class to enforce, and an inert lens landed beside its vocabulary would be the lie §6 names.
- **No public citation of private witnesses as evidence** — the rows name the private carriers as provenance and state their own rung as review diligence; claiming more would be rung inflation (§4b(1)).
