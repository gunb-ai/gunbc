# Review-surface subject map (declaration grain)

**Status: planning worksheet under review. No review-system implementation edits follow until the
rows are agreed — this document and its temporary doc-graph registration are the only changes.**
Deliberately not a `.dag` roster — landing a permanent registry with no consumer is the error this map
exists to catch, and is already in the tree (finding 1).

**Dissolve-on:** delete this file when every row has either been deleted, consumed by a named live
edge, or transferred to a typed roadmap obligation with an owner and acceptance.

Module grain decides nothing: `gunbc.review_verdict` alone spans external-artifact ingestion, prose
findings, a workflow recommendation, and merge-readiness tallying. Every cell below was established by
search against the current tree, not by reading a module's description of itself.

## Liveness grain

The phrase "real producer/consumer" caused the inaccurate rows in the first revision. Four levels,
worksheet-only vocabulary:

| Level | Meaning |
| --- | --- |
| `DeclaredOnly` | The declaration exists. Nothing constructs it anywhere, including tests. |
| `FixtureExercised` | Constructed and executed by its own tests/fixtures only. |
| `SemanticEdge` | Another domain function consumes it in the call graph — **which does not mean anything runs**. A dormant edge is still `SemanticEdge`. |
| `LiveObserved` | Observed executing on a scheduled or production path. |

"Could run", "is called by another domain function", and "has been observed executing" are three
different facts. Conflating them is what produced the previous revision's `MergeReadinessTally` row.

**Known weakness:** `FixtureExercised` and `SemanticEdge` are cumulative rather than exclusive — a
declaration can have a call-graph consumer *and* execute only in fixtures. The liveness cell records
the **highest** level reached and appends the execution qualifier (e.g. "`SemanticEdge`, fixture-only
execution").

---

## The map

| Declaration or group | Semantic role | Producer mechanism | Non-test semantic consumer | Liveness | Disposition |
| --- | --- | --- | --- | --- | --- |
| `std.claim_evidence.*` | Generic claim / evidence / provenance / readiness | Several domain producers | **7 non-test importing modules**: `std.citation`, `gunbc.repository_census_observation`, `gunbc.os_install_claim_evidence`, `gunbc.provider_readiness_claim_evidence`, `gunbc.claim_evidence_probe_rule`, `gunbc.source_integration_claim_evidence`, `gunbc.source_integration_landing_spine` (+5 test files: 4 witnesses, 1 acceptance fixture) | `SemanticEdge` (broad) | **Keep — the substrate.** Corrected from "8 production modules"; the eighth path was the module itself, and "non-test consumer" is the safer claim than "production" until live execution is established. |
| `v2.lens.grounding.*` | Mechanical grounding-candidate extraction | Live tree traversal | **None.** No non-test caller of `candidates`, `candidates_in`, or `worklist`. Enrolled in `v2.lens.registry` (`lens_registry_v0_grounding`) and `lens_module_gate` — enrollment establishes the lens is *known*, not that its findings are consumed. Contract is `AuditOnly` / `NoConsumerWitness` | `FixtureExercised` | **Repair candidate population and identity FIRST, then extend.** Downgraded from "the one real mechanical producer". See blocker below. |
| `v2.lens.grounding_ledger.*` | Copied subject + verdict + provenance | Its own constructor | **None** — no module references `GroundingVerdict` or `LedgerEntry` | `FixtureExercised` | **Delete / fold.** Finding 1. |
| anemia eval corpus (17 rows) | Historical labelled evaluation data | Authored JSON | **None** — only reference was the unused `eval_corpus_path` constant in `gunbc.tools.grounding_confirm`, and that module was **DELETED 2026-08-22** as census residue, so the corpus now has no in-tree reference at all | `DeclaredOnly` (as a consumed artifact) | **Keep as evaluation data.** Stale `ctrl` claim retired in #7896. |
| `gunbc.econ.llm_attempt_receipt.LlmAttemptReceipt` | Execution / economic observation | **None** | Derivations **inside its own module** + its witness; no external consumer | `FixtureExercised` | **Reuse, insufficient alone.** Corrected: previous revision implied external economic consumers. |
| `gunbc.review_verdict.ReviewArtifact` / `ReviewVerdictReport` / `ParsedReport` / `ReviewVerdicts.Parse` | External-review ingestion **contract** | **None in tree.** GitHub/dashboard objects are upstream sources, not constructors of these carriers | None | `DeclaredOnly` | **Preserve separately.** Corrected: previous revision named GitHub/dashboard as "the producer", conflating an upstream source object with an in-tree capture adapter. No capture adapter, parser realization, report producer, or caller exists. |
| `extdeps.github.pulls.PullReview` | Upstream review observation | REST projection | extdeps consumers | `SemanticEdge` | **Insufficient for reviewer identity.** Carries `id, body, state, commit_id, html_url` — **no `user`**, recorded as a typed coverage gap in its own `structural_coverage_gap_github_pull_review_response_residual`. So no honest `ReviewArtifact.reviewer`, no distinct-reviewer approval tally, no observed byline. This is the prerequisite for reviving #7539's distinct-reviewer logic. |
| `gunbc.review_verdict.MergeReadinessTally` | Workflow recommendation input | **None outside fixtures** | `gunbc.code_change_workflow.evaluate_record_merge_ready` reads it; but `decide_code_change_transition` is referenced only by that module and its witness, and there is **no in-tree call to `CodeChangeWorkflow.DecideTransition`** | `SemanticEdge`, **dormant** | **Do not build the producer yet.** Corrected: previously called a "live gate". It is a dormant admission kernel missing both producer and driver. |
| ↳ `MergeReadinessTally` count carriers (`approval_count`, `request_changes_count`, `blocking_finding_count`) | Count fields | Caller-authored arithmetic | Consumer asks only whether request-change and blocking counts are `> 0` | — | **Second state-space defect.** All three are `Int`, so a **negative** value behaves like zero and permits the transition; `approval_count` is not read at all. Use `Nat`/domain count carriers, or derive counts from retained populations — the review-composition receipt should retain the exact review/finding populations and derive counts rather than accept caller-authored arithmetic as primary authority. Zero-approval **and negative-count** RED controls before activation. |
| ↳ `MergeReadinessTally.approval_count` | Approval-count field | Authored `2` in 5 witness fixtures | **Never read.** `code_change_workflow` checks only `head_sha`, `request_changes_count`, `blocking_finding_count`, `mergeable_clean` | — | **Defect (dormant).** A zero-approval tally would still permit `ChangeMergeReady`. `source_artifact_ids`, `repo`, `pr_number` likewise unconsumed. No RED plants this. The two-approval rule becomes a real typed obligation only in the composition slice. |
| `gunbc.pr_digests.MergeReadinessVerdict` (**module DELETED 2026-08-22** as census residue — it was unreachable and uncalled, so the "second required aggregate" this row weighs no longer has a carrier; the open question it poses is unaffected and now has one fewer premise) (`Ready` \| `NotReady{first_reason, more_reasons}`) + `JudgeMergeReadiness` | Second required aggregate | **None** | `code_change_workflow` refuses a missing digest independently of a missing tally | `SemanticEdge`, dormant | **Answer before building either producer:** are `MergeReadinessVerdict` and `MergeReadinessTally` complementary projections of one review-composition receipt, or two independently writable authorities over overlapping PR/review facts? Building the tally producer first calcifies the split. |
| `gunbc.merge_admission`, `gunbc.merge_admission_subject`, `gunbc.merge_admission_produce`, `gunbc.merge_lifecycle`; `tools.merge_admission_capture`, `tools.merge_admission_capture_transport`, `tools.merge_admission_current_context`, `tools.merge_admission_walk` | **Existing CI/check-freshness admission authority** — binds tested head, base tree, gate-roster identity, check conclusion | CI floor (pre-walk capture, floor stamping, current-target refresh) | CI floor scheduling | `LiveObserved` — the CI run **executed** the pre-walk typed capture subprocess and produced its receipt, and executed the merge-admission and merge-lifecycle witnesses; not merely scheduled | **Missing from the previous revision entirely.** Not another review parser. Review work must not re-encode base-tree freshness, check coverage, or gate-roster identity in its own tally; conversely this authority must not learn about anemia verdicts, review prose, or model vendors. |
| `gunbc.code_change_workflow` | Orchestration join (?) | Authored | Its witness | `SemanticEdge`, dormant | **Unresolved relationship row.** Is it the future join over review authorization × merge admission, a prototype to dissolve, or is another roadmap belt the correct composition point? Unanswered ⇒ "keep the tally and construct it" is premature. |
| `gunbc.review_verdict.ReviewVerdict` | Workflow **recommendation**, not a criterion verdict | `demo_spine_green` / `demo_spine_unknown` fixtures construct `Approve`, `ApproveWithComments`, `RequestChanges`; external-artifact parser realization **absent** | `idea_pr_spine.review_to_reduce` | `SemanticEdge`, fixture-only execution (`workflow_recursive_witness_test` calls and settles both demo spines) | **Rename/rehome to `ReviewRecommendation` when the external-review producer lands** — not an immediate independent rename against an inert path. |
| `Reviewer` / `LlmReviewProvider` / `ReviewSource` / `ReviewProvider` | Who supplied a review | — | `idea_pr_spine`, `reviewer_source_witness_test` | `FixtureExercised` | **Not "four names for one thing"** (previous revision overstated). `Reviewer` models the observed actor/byline; `LlmReviewProvider` is a concrete-product subfield of an agent reviewer; `ReviewSource` is a lossy projection *from* `Reviewer`, not a second identity; `ReviewProvider` is a competing simpler identity used only by the demo spine. Disposition: preserve an **open-world observed byline** for imported artifacts, keep source as a projection, delete `ReviewProvider` with the demo spine unless a real consumer earns it, and keep concrete-product identity out of generic judgment semantics. Do not add `Qwen`/`Spark`/`Local`. |
| `gunbc.tools.review`, `gunbc.tools.review_codex` | Vendor-specific execution + artifact posting | Anthropic / Codex | GitHub reviews (out-of-tree) | Effect-capable; **cadence unproven** — only `review_cycle` references are self-authored command strings | **Producer implementations.** They post prose `PullReview`s, not typed criterion evidence. The Spark path should produce an evidence link first and derive any posted prose from it. |
| `gunbc.digest_render` | PR-review / readiness **presentation** prototype | Authored constructors/projections, **no caller** | **None.** `gunbc.plans.format_model_reconciliation` only *names* it in prose inside an `li(text: ...)` backlog item — it does not import it, call `project_pull_request_digest` or either renderer, or consume `PullRequestDigestView` | `DeclaredOnly` | **Presentation prototype: delete, consume, or derive from the eventual review-composition receipt.** Corrected — a prose mention is not a consumer edge; counting one is the same stale-citation class this worksheet exists to detect. |
| `gunbc.workflow.types` review subset (`ReviewDimension`, `ReviewConcern`, `DimensionReviewOutput`, `DesignFinding`, `DesignReviewOutput`) | Parallel review vocabulary | None | None | `DeclaredOnly` | **Delete, rehome, or justify.** Note: the file carries a much wider dormant workflow schema; only the review subset is in scope here. `system_prompt_preamble` does not belong on a criterion carrier — a prompt is one judge realization's derived input. |
| `gunbc.idea_pr_spine.Review { subject, reviewer, verdict }` | Workflow composition | `demo_spine_green` / `demo_spine_unknown` only | `review_to_reduce` → `ReduceVerdict` | `FixtureExercised`, **demo-only/inert** | **Recut after the map.** The provider that executed one judgment is not a property of a workflow stage. |

---

## Findings

**1. A false registry exemption.** `v2.lens.registry.completeness` exempts `v2.lens.grounding_ledger`
from distinct lens enrollment because it is "vocabulary/support types consumed by `v2.lens.grounding`
(`GroundingVerdict`, `LedgerEntry`)". `v2.lens.grounding` references neither symbol. The exemption is
justified by an edge that does not exist, and the module it protects has no consumer. Repair by
deleting the module so the row disappears; any surviving row must name an exact live consumer.

**2. Gaps run in both directions, and none is a live failure.** `grounding_ledger` has a producer and
no consumer. `MergeReadinessTally` has a dormant semantic consumer and no producer. `ReviewVerdictReport`
has neither. **Correction to the previous revision:** none of this shows a live merge process accepting
bad data — `code_change_workflow`'s kernel has no in-tree driver, so the repository does not establish
that today's merge decision passes through it at all. Consequential semantically, not operationally.

**3a. The exclusive proposition space is NOT established.** The constraint section below describes
grounding as choosing among `Reference{authority}` / `ReferenceUri` / `ReferencePath` / `Role` /
`StayString` / `GenuineProse`. That population comes from the inert `grounding_ledger`, and nothing
establishes that its arms are mutually exclusive or complete: `ReferenceUri` and `ReferencePath` look
like particular *reference targets*, while `GenuineProse` looks like a *reason for retaining a leaf
string* — different axes, with no construction proving they cannot overlap. **Do not carry that
coproduct forward merely because the ledger is being deleted.** A likely minimal outer partition is
`GroundTo { authority: DeclarationRef } | RetainLeaf { reason: GroundingRetentionReason }`, where URI,
path, role, prose, open registry, parse input and cited identifier may turn out to be exact authorities
or retention reasons rather than peer top-level verdicts — a hypothesis for the DFS, not a ruling to
encode. The proposition space is defined by GROUNDING-DECIDE-0, not by the population repair.

**3b. Grounding cannot yet feed an exclusive-candidate review.** `ConceptByName` can carry multiple
`QualifiedConcept`s under one name, but `first_coincident_target` folds to the **first** non-self match
and `GroundingWorklistEntry` stores a single `coincides_with` **string**. The test corpus has no
same-name ambiguity control. That is a silent-pick defect exactly where the proposed design assumes a
complete finite candidate population — and the worklist's `enclosing`/`field`/`qualified_name`/
`coincides_with`/`target_kind`/`target_structure` are presentation strings standing where exact
`DeclarationRef` identities are needed.

**4. `std.claim_evidence` is settled substrate** — 7 non-test importing modules.

---

## Constraints on the recut (recorded so worksheet and build cannot drift)

**Grounding is exclusive-candidate, not conjunctive-readiness.** `ClaimReadinessPolicy` is a
conjunction; grounding asks which *one* proposition holds. **The proposition population itself is open
— see finding 3a; the ledger's six arms are not established as disjoint or complete and must not be
carried forward by default.** One `Claim` per candidate proposition through the generic
layer, then a grounding-specific exclusive-resolution fold: exactly one supported and unchallenged
resolves; zero supported is unresolved, never clean; two supported is conflict, never first-wins;
support-and-challenge on one proposition is conflict; admission failure is refusal, not missing.
"Send to a Spark" is routing policy derived from `GroundingUnresolved`, not an epistemic state.

**A receipt proves a model ran, not what it answered.** `LlmAttemptReceipt` carries no returned output.
Model evidence must additionally bind raw-output identity, decoder identity, and selected candidate,
with admission checking candidate-set membership, resolved-not-requested model, substrate equality,
exactly-once decode, and refusal on malformed output.

**"Cleared" stays criterion-specific.** A resolved `Reference{authority}` confirms anemia; a resolved
`StayString` may clear it. Decision-existence and clearance must never share a function.

**Final admission is a product, not a merge.**

```
review authorization receipt  ×  CI/check-freshness admission receipt  ×  operator/policy authorization
                                    ↓
                            final merge authorization
```

---

## Sequence

1. Correct and merge this worksheet.
2. **GROUNDING-POP-0 — population only.** Exact field-grain subject → all coincident declaration
   candidates → exact `DeclarationRef` identities → deduplicate only identical identities → explicit
   zero/one/many population. Acceptance: two same-named concepts in different modules both survive
   enumeration; source order cannot change the population; the current first-match algorithm **fails**
   the ambiguity control; the subject is not independently writable `enclosing`/`field`/qualified-name
   strings; candidate display structure remains a projection, not identity; empty, unique and ambiguous
   populations stay distinguishable; and **source order cannot change the population's canonical identity
   or emitted ordering** — one mathematical candidate set must not be representable by several
   order-sensitive lists, which is the HOST-0 list-identity defect in its general form. **Explicitly
   not in it:** `std.claim_evidence` aliases, the final
   `GroundingVerdict`, the anemia corpus, any LLM call, any tally, any admission authority.
3. **GROUNDING-DECIDE-0** — disjoint criterion propositions (see 3a) + structural evidence + exclusive
   resolution. This slice defines the proposition space.
4. **REVIEW-SHADOW-0** — roadmap presentation of typed results. No merge authority.
5. **SPARK-JUDGE-0** — judgment evidence, with the bindings above; malformed output / invented candidates / stale
   substrate / missing or duplicate receipts refuse.
6. **External-review capture**, in parallel: enrich `PullReview` with reviewer identity, capture typed
   `ReviewArtifact`, preserve stale/foreign/unreadable states, produce one subject-bound receipt.
7. **REVIEW-COMPOSE-0** — decide the relationship among native criteria, external recommendations,
   `MergeReadinessVerdict`, `MergeReadinessTally`, and approval policy. The two-approval rule becomes a
   typed obligation here, with a zero-approval RED.
8. **REVIEW-ADMIT-0** — join the review-authorization receipt with the existing
   `merge_admission` receipt. Only then wire the roadmap merge-ready transition.
