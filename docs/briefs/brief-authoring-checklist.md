# Brief-authoring authority-audit checklist

> **Mandatory pre-author discipline** for any Director / manager
> brief that proposes substrate carriers, claims an existing
> design-doc recommendation, cites file:line authority, or scopes
> a consumer against a producer brief. Captures the discipline
> derived from #836's review cycle (8 substantive reframes, all
> from the same `feedback_audit_adjacent_authority_first` /
> `feedback_verify_thesis_claims` failure mode).

## When to apply

**Apply this checklist BEFORE drafting Slice / Acceptance / STOP-AND-ESCALATE for:**

- Substrate-producer worker briefs (lands new substrate carrier).
- Consumer-migration worker briefs (consumes producer's signal).
- Implementation worker briefs that cite a design-doc recommendation.
- Per-class sub-lane briefs that depend on prior brief decisions.
- Any brief that includes file:line citations as authority.

**Skip when:** the brief is a tracking doc, a status-only report, a closed-as-redundant routing doc (where the audit receipt itself IS the deliverable), or a fresh-from-scratch design proposal with no prior authority claims.

## The five-question audit (mandatory)

Before authoring any of the brief sections above, answer **all five** in writing. The answers go in the PR body that lands the brief; if they reveal the brief's premise is wrong, the brief reframes accordingly **before** authoring.

### 1. Does the substrate this brief assumes exist already?

**Grep:**
- `src/v3/std/` + `src/v3/spec/` for the proposed carrier shape (field names, type names, variants).
- `src/v3/compiler/src/dag.rs` and `infer.rs` for the proposed Rust mirror.

**If found:** the brief is consumer-migration, not producer landing. Reframe before authoring Slice. Cite the existing authority's file:line.

**Examples from #836's reframes:**
- Parametric-algebra-for-Dimensions: `Declaration.phantom_params` already at `dag.rs:186`; `phantom_unit_mismatch` already at `infer.rs:1057`. Closed as no-op.

### 2. Does an existing brief already cover this scope?

**Grep `docs/briefs/`** for similar scope (cardinality, fold, etc.).

**If found:** route to existing brief; this one becomes a closed-as-redundant routing doc, not a second authority. Per `INVARIANTS.md` P2 (single authority).

**Examples from #836:**
- cardinality-for-int-lit subset: `t-substrate-cardinality-int-lit-worker.md` already exists with re-scoped post-`wise-pike-578` decisions. Closed as redundant.

### 3. Does the design-doc §Director-actionable recommendation match the brief's premise?

**For any worker brief that consumes a design-doc:** read the design-doc's §Director-actionable / §Q-recommendation / §"Path forward" section **in full**. Do NOT assume the recommendation matches your initial framing.

**If the recommendation differs:** reframe Slice + STOP-AND-ESCALATE to match. Surface the design-doc's specific reqs as numbered Slice items.

**Examples from #836:**
- Nested-optional: design-doc verifies v3 substrate is past the cardinality bridge; recommendation is ungated constructor invariant, not gated implementation. Brief was reframed.
- Unhandled-diagnostic: design-doc §4 explicitly recommends totality-by-omission; predicate-entailment is M+ scope reopening DB-11. Brief was reframed.
- Unenumerated-effects: design-doc §Q6 enumerates 8 specific reqs; brief had elided 3. Brief was reframed.

### 4. Are the file:line citations live at HEAD?

**For every `file.rs:N` citation:** grep the file at HEAD; verify N still names the cited construct.

**If line numbers drift:** update before authoring. Cite by symbol grep when feasible (e.g., search-anchor) instead of fragile line ranges.

**Examples from #836:**
- `dag.rs:395-398` (Cardinality) → `:408-411` (variant moved; Arrow now occupies prior range).
- `src/v3/std/types.dag` → `dsl/std/types.dag` (5 briefs had wrong path).

### 5. Does the carrier shape actually dissolve the cited bridge?

**For substrate-producer briefs:** read the call-site code that consumes the bridge being dissolved. Confirm the proposed carrier answers the right question.

**Specifically for span-suffix / file-equality bridges:** read the inline doc comment + the helper function the bridge gates. The bridge often skips on a different fact than the brief assumes.

**Examples from #836:**
- B4.2 fold_step_formal: brief assumed bridge skips on step-formal binding. Worker pre-flight audit revealed bridge actually skips on accumulator/element type eligibility. Carrier addressed wrong question. Brief reframed.

## Citation discipline — cross-doc references

**Cross-doc references in briefs MUST use section anchors or
direct rule-text quotes. Bare line-number references
(`file.md:NNN`) are PROHIBITED** — they drift silently on any
doc edit above the cited line, going stale without surfacing
in review.

**Stable forms** (use either):

1. **Section anchor**: e.g.,
   `docs/modeling-discipline.md#4-coproduct-dissolution` —
   GitHub auto-generates anchors from headings; survives any
   line-number drift below.
2. **Rule-text quote**: cite the rule directly. E.g., for the
   coproduct-classification rule: per `docs/modeling-discipline.md`
   Practice 4 ("Coproduct dissolution"): *"Any new Rust enum with
   N ≥ 2 variants must have a checkpoint comment naming its
   classification (🟢/🟡/🔴), with a ledger entry if GREEN or a
   named trigger if YELLOW."* Self-contained; survives even doc
   restructuring.

If the rule has no stable anchor available, surface to the doc
author to add one before citing.

**Applies to all** cross-doc citations across briefs:
`r3-program-plan.md`, `r3-structure.md`, `INVARIANTS.md`,
`MODELING.md`, `modeling-discipline.md`, design-doc references,
etc.

**Specific to Practice 4 / coproduct classification**: cite
`docs/modeling-discipline.md#4-coproduct-dissolution` (Practice 4)
plus the "What to check" rule text inline. Do NOT cite "Step 4
of the type-introduction checklist" — no such checklist exists.

**Failure modes**:
- Codex review on PR #1782 (2026-05-06) flagged fabricated
  "Step 4 of the type-introduction checklist" wording.
- Director redirect on the paydown form (gunbc#828
  #issuecomment-4385393971 follow-on) flagged that the bare
  line-number replacement (`:131`) is the same drift class.
- Corpus sweeps at `2dc92f34f` (initial Step-4 paydown) +
  `b4cae5299` (anchor/quote conversion + checklist rewrite)
  closed both vectors.

## Audit receipt format

The PR body that lands the brief must include a short audit receipt:

```markdown
## Authority audit receipt (brief-authoring-checklist.md compliance)

1. **Substrate exists?** [grep result + answer]
2. **Existing brief?** [grep result + answer]
3. **Design-doc recommendation matches?** [§reference + verdict]
4. **Citations live?** [verified at sha XXX]
5. **Carrier dissolves the bridge?** [call-site + helper-doc cite + verdict]
```

If any answer reveals a mismatch, the brief reframes; the receipt records the reframe path.

## Cross-references

- `feedback_audit_adjacent_authority_first` — the parent discipline.
- `feedback_verify_thesis_claims` — the failure mode this checklist prevents.
- `feedback_design_before_implement` — pre-flight audit prevents propagation of contested shapes.
- `feedback_parallel_representation_debt` — what the audit prevents.
- `INVARIANTS.md` P2 (single authority) + P5 (paired dispatch).

## Provenance

This checklist captures the discipline lesson from #836's review cycle (8 substantive reframes across the R2 spin-up Wave 1-4 brief authoring), per openai-pro meta-verdict 2026-04-26 SHIP_WITH_DEBT recommendation. The 8 reframes — nested-optional gating, unhandled-diagnostic path-default, unenumerated-effects elision, parametric-algebra producer, cardinality-for-int-lit producer, nominal-opaque 7th-connective, int-lit consumer scope mismatch, B4.2 carrier-shape misdiagnosis — were all caught reactively by reviewers (7) or worker pre-flight (1). This checklist makes the discipline mechanical and pre-flight rather than reactive.
