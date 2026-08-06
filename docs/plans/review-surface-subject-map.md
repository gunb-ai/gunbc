# Review-surface subject map (declaration grain)

**Status: planning worksheet, reviewed before any recut. No code edits follow from this document
until its rows are agreed.** It is deliberately not a `.dag` roster: landing a permanent registry with
no consumer is the error this map exists to prevent, and it is the error already sitting in the tree
at `v2.lens.registry.completeness` (row 3 below).

Purpose: before consolidating anything onto `std.claim_evidence`, establish which existing review
declarations are evidence producers, external-artifact ingestion, native typed criteria, review
composition, recommendation, workflow admission, or presentation — and which have a **real** producer
and consumer rather than a claimed one. Module grain is too coarse: `gunbc.review_verdict` alone spans
four of those roles.

Every producer/consumer cell below was established by search against the current tree, not by reading
a module's description of itself. Where a claim in the tree disagrees with the tree, the row says so.

---

## The map

| Declaration or group | Semantic role | Real producer | Real consumer | Status |
| --- | --- | --- | --- | --- |
| `std.claim_evidence.*` — `Claim<S,T,P,Scope,Bound>`, `RecordedFact<F>`, `EvidenceLink`, `EvidenceProvenance`, `EvidenceFreshness<C>`, `EvidenceFidelity<Boundary>`, `EvidenceDirection`, `EvidenceInference`, `ClaimRequirementReadiness`, `ClaimReadinessReceipt` | Generic claim / evidence / provenance / readiness | Several domain producers | **8 production modules**: `gunbc.source_integration_landing_spine`, `gunbc.source_integration_claim_evidence`, `gunbc.os_install_claim_evidence`, `gunbc.repository_census_observation`, `gunbc.claim_evidence_probe_rule`, `std.citation`, + 5 witness entries | **Keep — the substrate.** Broader adoption than "the landing spine and others"; this is settled authority, not a candidate. |
| `v2.lens.grounding.*` | Mechanical grounding-candidate extraction (normalization, coincidence match, target-kind enrichment, exclusions) | Live tree traversal | `src/v2/test/claim/grounding_lens_test.dag`; enrolled in `v2.lens.registry` as `lens_registry_v0_grounding` and in `lens_module_gate` | **Extend to the review vertical.** This is the one real mechanical producer in the review area. |
| `v2.lens.grounding_ledger.*` — `LedgerEntry`, `DecidedBy`, `PendingJudge`, `residue_forced_verdict`, `structural_ledger_entry` | Copied subject + verdict + provenance | Its own constructor | **None.** No module references `GroundingVerdict`, `LedgerEntry`, or any ledger symbol; only importer is its own test | **Delete / fold.** See "the false exemption" below. |
| anemia eval corpus (`src/v2/lens/testdata/anemia_confirm_eval_corpus.json`, 17 rows: 10 positive / 7 §5-negative, 12 deterministic / 5 haiku) | Historical labelled evaluation data | Authored JSON | **None.** Only code reference is the unused `eval_corpus_path` constant in `gunbc.tools.grounding_confirm` | **Keep as evaluation data.** Stale `ctrl` consumer claim retired in #7896. |
| `gunbc.econ.llm_attempt_receipt.LlmAttemptReceipt` | Execution / economic observation | **No production path yet** | Economic derivations (`derive_decode_throughput`, `derive_accepted_goodput`, `derive_escalation_fraction`) + its witness | **Reuse, insufficient alone.** Records that a model *ran*, never *what it answered* — see "receipt is not an answer". |
| `gunbc.review_verdict` — `ReviewArtifact`, `ReviewFinding`, `ReviewVerdictReport`, `ReviewVerdictParseRequest`, `ReportExtractionState`, `ParseDeferredState` | External artifact ingestion + prose findings | GitHub / dashboard artifacts (out-of-tree) | **`ReviewVerdictReport` is declared but never constructed anywhere in the tree** | **Preserve separately.** Ingestion of observed prose is a genuinely different concept from a derived typed judgment; do not merge. Producer gap is real. |
| `gunbc.review_verdict.MergeReadinessTally` | Workflow recommendation input | **Missing.** Constructed only in `code_change_workflow_witness_test` fixtures | **Real**: `gunbc.code_change_workflow` matches on it to refuse request-changes / blocking findings / unclean mergeability | **Keep.** The one review declaration with a genuine production consumer and no production producer — the inverse of the ledger. |
| `gunbc.review_verdict.ReviewVerdict` (`Approve` \| `ApproveWithComments` \| `RequestChanges` \| `NeedsHumanTriage` \| `NoVerdict`) | Workflow **recommendation**, not a criterion verdict | Parsed from artifacts | `idea_pr_spine`, tally path | **Rename to `ReviewRecommendation`** so `GroundingVerdict`, `VacuityEvidence` and an approval recommendation stop competing for "verdict". |
| `gunbc.review_verdict.LlmReviewProvider` / `ReviewProvider` / `ReviewSource` / `Reviewer` | Who supplied a review | — | `idea_pr_spine`, `reviewer_source_witness_test` | **Four overlapping ways to name one thing**, and `LlmReviewProvider` enumerates concrete products (`OpenaiPro`\|`Codex`\|`Claude`\|`Cursor`). Retain observed byline for *ingested* artifacts; must not gain `Qwen`/`Spark`/`Local`. |
| `gunbc.tools.review`, `gunbc.tools.review_codex` | Vendor-specific execution + artifact posting | Anthropic / Codex | GitHub reviews (out-of-tree) | **Treat as producer implementations.** They post prose, not typed criterion evidence — the Spark path should produce an evidence link first and derive any posted prose from it. |
| `gunbc.tools.grounding_confirm` | Prototype LLM confirm experiment | Hard-coded 6-row list; hardwired Haiku; `starts_with` grading | None | **Replace in the JUDGE vertical.** Declares `eval_corpus_path` and never reads it. |
| `gunbc.workflow.types` — `ReviewDimension`, `ReviewConcern`, `DimensionReviewOutput`, `DesignFinding`, `DesignReviewOutput` | Parallel review vocabulary | **None** | **None** — symbols appear only in that file | **Delete, rehome, or justify.** `ReviewDimension` may survive as optional UI grouping; `system_prompt_preamble` does not belong on a criterion carrier. |
| `gunbc.idea_pr_spine.Review { subject, reviewer, verdict }` | Workflow composition | Authored process | `review_to_reduce` → `ReduceVerdict` | **Recut after the map.** Couples a workflow stage to a provider identity; the provider that executed one judgment is not a property of the stage. |

---

## Three findings the map produced

**1. The false exemption.** `v2.lens.registry.completeness` exempts `v2.lens.grounding_ledger` from
distinct lens enrollment on the stated ground that it is "vocabulary/support types consumed by
`v2.lens.grounding` (`GroundingVerdict`, `LedgerEntry`)". `v2.lens.grounding` references neither symbol.
The exemption is therefore justified by an edge that does not exist, and the module it protects has no
consumer at all — the coverage-by-illusion tier the `lenses-must-be-live` standing intent targets. The
correct repair is deletion of the module, at which point the roster row disappears rather than being
reworded; if any row survives, its reason must name an exact live consumer.

**2. Producer/consumer gaps run in both directions.** `grounding_ledger` has a producer and no
consumer. `MergeReadinessTally` has a consumer (`code_change_workflow`, which really does gate on it)
and no producer outside test fixtures. `ReviewVerdictReport` has neither. These are three different
defects and were previously lumped as "the review path is partial".

**3. `std.claim_evidence` is more adopted than assumed** — eight production modules, not one. That
strengthens it as the substrate and weakens any argument for a parallel kernel (the argument I lost).

---

## Two constraints on the recut, recorded here so the map and the build agree

**Grounding is exclusive-candidate, not conjunctive-readiness.** `ClaimReadinessPolicy` is a
conjunction — readiness means *every* required claim is satisfied. Grounding asks which *one* of
`Reference{authority}` / `ReferenceUri` / `ReferencePath` / `Role` / `StayString` / `GenuineProse`
holds. So the composition is one `Claim` per candidate proposition, admitted through the generic
layer, then a **grounding-specific exclusive-resolution fold**: exactly one supported and unchallenged
resolves; zero supported is unresolved, never clean; two supported is conflict, never first-wins;
support-and-challenge on one proposition is conflict; admission failure is refusal, not missing.
"Send to a Spark" is routing policy derived from `GroundingUnresolved`, not an epistemic state.

**A receipt proves a model ran, not what it answered.** `LlmAttemptReceipt` carries requested and
resolved model, artifact, runtime, machine opportunity, prompt/context policy, substrate, usage,
timing, energy and disposition — and no returned output. Model evidence must additionally bind raw
output identity, decoder identity, and selected candidate, with admission checking that the selection
is in the mechanically supplied set, that resolved (not requested) model and substrate match the
reviewed substrate, that the output decodes exactly once, and that malformed output refuses.

**"Cleared" stays criterion-specific.** A resolved `Reference{authority}` *confirms* anemia; a
resolved `StayString` may clear it. Decision-existence and clearance must never share a function — a
judge that confirmed every violation would show total decision coverage and zero clearances.
