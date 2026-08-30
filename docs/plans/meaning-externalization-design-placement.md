# Meaning, externalization, and quality-floor placement in DESIGN

**Status:** analysis only. This document recommends a supervised authority edit; it does **not** edit `gunbc.design_document`, `DESIGN.md`, `gunbc.recurring_failure_mode`, or `docs/design-ledgers.md`.

**Reviewed public revision:** `gunb-ai/gunbc@b60fc7dd61e5bae6b8c2e493e87815a116d4e7a5`; re-baselined (ROOT-0) onto `main@826824d22e964058d872e5e02a8c1718d1128ac7` — the three intervening commits touch floor execution, regen affected-set bounds, and memory-per-vCPU market material, none of which is a DESIGN authority, ledger, projection route, or §3/§4b/§5 citation surface, so the census and evidence base below stand unchanged.

**Private source study:** `gunb-ai/gunbc-private` PR #25, `strategy/pricing-model-study@880facecf017767a2e80b3800823694e86536fa8`: `dag/strategy/pricing_decomposition.dag` (`strategy.pricing_decomposition`), `dag/test/claim/pricing_decomposition_witness_test.dag` (`test.claim.pricing_decomposition_witness`), and `docs/strategy/2026-08-30-pricing-model-study.md`.

**Scope boundary:** this study locates the consequences, enumerates the exact authority/projection change set, inventories current section citations, and identifies representative downstream enforcement surfaces. The operator's corpus-wide violation audit is explicitly out of scope.

## Recommendation

| Concept | Recommended home | Exact insertion anchor | Ledger consequence |
|---|---|---|---|
| Accuracy of meaning | §3, single authority | In `gunbc.design_document.section_3_blocks`, immediately after the opening nicknaming paragraph and before “Two corollaries.” | Add `gunbc.recurring_failure_mode.meaning_fork`. |
| Quality floor and named degradation | §4b, guarantee-ladder honesty | In `gunbc.design_document.section_4b_blocks`, immediately after the four meta-obligations and before “Every newly discovered error class…” | No third ledger class yet; concrete failures classify under `meaning_fork`, `externalized_degradation`, or rung inflation according to mechanism. |
| Externalization | §5, fail-closed | In `gunbc.design_document.section_5_blocks`, immediately after the absorbing-fallback paragraph and before “A corollary on the refusal itself…” | Add `gunbc.recurring_failure_mode.externalized_degradation`. |

Do not add a second normative externalization paragraph to §2. §2 supplies the premise—present convenience can defer cost onto a later fixer—and the §5 paragraph should name the boundary case where that fixer or burden-bearer is outside the firm. Stating the full rule in both places would make the document repeat itself rather than reason serially.

## Authority and projection trace

`DESIGN.md` is not source. The source path is:

1. `gunbc.design_document.design_document` composes the ordered blocks returned by `gunbc.design_document.design_blocks`.
2. `gunbc.design_document.expected_design_md` serializes that document.
3. `gunbc.generated_artifact_emit.artifact_generate` selects `expected_design_md` for `gunbc.generated_artifact.DesignArtifact`.
4. `tools.generated_artifact_gate.main_wet` writes the generated artifact.
5. `gunbc.ci_spec.ci_heal_author_commit_regen_command` names the canonical transaction:

```text
gunbc run --source-root dag --source-root src/v2 --entry dag/gunbc/instruments/generated_artifact_gate.dag --function main_wet
```

The same generated-artifact transaction projects `gunbc.recurring_failure_mode.recurring_failure_mode_roster` through `gunbc.design_ledgers.expected_design_ledgers_md` into `docs/design-ledgers.md`. `gunbc.design_document.failure_modes_blocks` maps the same roster into the final `DESIGN.md` failure-mode index.

The authority and pipeline are therefore explicit. No stop/escalation condition is present.

## Placement analysis

### Accuracy of meaning is a §3 consequence

§3 currently names **nicknaming**: two names for one meaning. The missing dual is one name carrying two meanings. Both are violations at the semantic authority layer:

- Nicknaming duplicates a semantic fact under multiple names.
- A meaning fork collapses multiple material semantic contracts under one name.

This applies beyond declaration identifiers. Product names, service names, tiers, statuses, units, and terms in every layer are carriers from which users and downstream programs infer obligations. Within one declared effective version or epoch, the same visible name cannot sometimes mean full-strength delivery and sometimes mean quantized delivery when that difference changes the quality floor, refusal behavior, billing consequence, or remedy.

§2 explains why the shortcut is expensive: it buys present convenience by pushing ambiguity, distrust, and correction work downstream. §5 explains why a silent fork can become wrongness. Neither is the concept's home. The root question is still “how many meanings does this name authorize?”, so §3 should state it once as the dual of nicknaming.

The public corpus already contains a direct specimen: `gunbc.provider_interface_binding.ProviderInterfaceBinding` records that “provider” is a homonym for the controlled coding-agent runtime and the inference backend. Today that distinction is an annotation and search instruction, not a structural one-name/one-contract guarantee.

### Externalization belongs once in §5, derived from §2

Externalization begins with §2's cost-deferral shape but becomes a named fail-open trap only at an organizational boundary. The firm was paid, trusted, or assigned to absorb a risk or cost; an adverse state arrives; the firm silently makes a counterparty bear it while keeping the apparent name, price, or contract unchanged.

That is a §5 failure because the missing information is load-bearing and the counterparty receives a plausible success surface. Surprise repricing, quiet service thinning, variable compensation that transfers demand risk, stretched seller terms, a low-disclosure liquidation, neighbor-borne operating cost, and unexplained future-maintainer burden differ in counterparty but share the same mechanism.

The two honest arms are exhaustive at this grain:

1. **Absorb and reserve:** keep the promised surface and hold the resources needed to carry its risk.
2. **Name and price the transfer:** expose the changed burden as its own contract or product, with a distinct name, terms, and price.

The recommendation is not “§2 and §5.” It is one §5 paragraph whose first sentence explicitly derives the trap from §2. A second §2 statement would create two prose authorities for the same rule.

### The quality floor is §4b applied to service delivery

§4b asks the reported rung to equal the rung established by executed evidence. At a service boundary, an opaque dimension can be legitimate: the provider may own memory rank, microarchitecture, supplier choice, or another implementation detail. But opacity establishes no guarantee by itself.

The quality floor supplies the declared subject grain and falsifier. A named billing, remedy, or refusal consequence supplies the contract effect. Together they distinguish:

- **opaque above a falsifiable floor**, where the dimension is the provider's implementation concern;
- **materially different delivery**, which must become a separately named and priced contract; and
- **sub-floor delivery**, which must refuse or discharge the promised remedy.

This is not another §5 trap paragraph. §5 forbids silence; §4b decides what evidence makes the service claim real. “A claim with no consequence is marketing” is rung honesty: an inert sentence, type, or metric establishes no contract rung.

Three states must stay distinct, because their dispositions differ and conflating them lets a provider rename an incident after the fact:

1. **Same contract, delivery below its floor** — a breach; the disposition is the contract's own typed consequence (remedy, refusal, quarantine, disqualification), never a rename.
2. **A materially different delivery with its own name, price, terms, and floor** — a different contract subject; the disposition is ordinary admission as a separate product. It is *not* a degradation of the premium subject and *not* a §4b(3) rung drop, because a stable lower tier may be permanent and owes no restoration trigger toward the premium product.
3. **A temporary loss of the system's ability to verify or enforce the same subject's guarantee** — that, and only that, is a §4b(3) declared drop, with previous rung, temporary rung, reason, bounded population, and restoration trigger. A drop in the *ability to verify* a promise never authorizes silently delivering below it; it means the commercial path may need to refuse until the capability is restored.

The separate-product arm remains coupled to §3: once the contract has materially changed, the old name cannot continue to answer for both contracts.

## Recurring-failure-mode decision

### Add `meaning_fork`

This is not merely `state_space_conflation`. A state-space conflation makes a carrier unable to represent distinct states. A meaning fork can exist even when the underlying states are perfectly modeled: the defect is that one external or cross-layer name continues to denote more than one material contract.

Recognition rule: hold the visible name and declared effective version constant, vary the hidden state, and compare the material contract. If obligations, floor, refusal, billing consequence, or remedy change without a distinct variant name, the name has forked.

There are already independent public/private specimens: the public “provider” homonym and the private premium-name/full-strength-versus-quantized delivery case. That is enough recurrence to justify a roster class rather than a one-off note.

### Add `externalized_degradation`

This is not `absorbing_fallback`. An absorbing fallback is a computation's failure arm that widens instead of refusing. Externalized degradation may involve no algorithmic fallback and no widening; it is the preservation of an apparently unchanged contract while the marginal risk or cost moves to a counterparty.

Recognition rule: identify the risk receipt or assigned responsibility; trigger the adverse state; follow who bears the marginal burden. If the counterparty's burden rises while the apparent name, price, contract, refusal, and remedy remain unchanged, the risk was silently re-exported.

### Do not add `quality_floor_absent` now

“The opaque dimension lacks a falsifiable floor and consequence” is a review test, not yet a distinct recurring mechanism. Depending on the construction, the resulting defect is a meaning fork, externalized degradation, rung inflation, fabricated output, or another existing class. Minting a third row now would classify by missing artifact rather than by failure mechanism.

Both proposed rows should initially keep `RecurringFailureMode.evidence` empty. That field is intentionally unread and unenforced today; populating it would be citation decoration. The supervised edit should follow the current authority's own rule and leave evidence empty until the cited-symbol consumer widens to this carrier.

Append the rows after `gunbc.recurring_failure_mode.identity_absent_graph_traversal`, in this order:

1. `meaning_fork`
2. `externalized_degradation`

The roster is intentionally source-ordered, not sorted.

## Exact artifact change census

### Authored authority changes

Only these source authorities should change in the supervised edit:

- `dag/gunbc/design_document.dag`
  - `gunbc.design_document.section_3_blocks`: one paragraph after the opening nicknaming paragraph.
  - `gunbc.design_document.section_4b_blocks`: one paragraph after the four meta-obligations.
  - `gunbc.design_document.section_5_blocks`: one paragraph after the absorbing-fallback paragraph.
- `dag/gunbc/recurring_failure_mode.dag`
  - new value `gunbc.recurring_failure_mode.meaning_fork`;
  - new value `gunbc.recurring_failure_mode.externalized_degradation`;
  - append both values to `gunbc.recurring_failure_mode.recurring_failure_mode_roster`.

No direct edit is needed to `gunbc.design_document.failure_modes_blocks`: it already projects the roster identities.

### Generated projection changes

Regeneration should change exactly:

- `DESIGN.md`
  - one new §3 paragraph;
  - one new §4b paragraph;
  - one new §5 paragraph;
  - two new identities in the final recurring-failure-mode index.
- `docs/design-ledgers.md`
  - one full `meaning_fork` entry;
  - one full `externalized_degradation` entry.

### Explicit non-changes

No source edit should be made to:

- `gunbc.design_ledgers`;
- `gunbc.generated_artifact`;
- `gunbc.generated_artifact_emit`;
- `tools.generated_artifact_gate`;
- `gunbc.ci_spec`;
- any historical plan merely because it cites §3, §4b, or §5.

The section identifiers and their prior claims remain stable. Existing citations acquire the added consequences but do not become stale or require mechanical rewriting.

## Current citation review surface

This is a literal section-anchor census at the reviewed public revision. It includes `§3`, `§4b`, and `§5` references plus the repository's observed `DESIGN section …` / `DESIGN …` spellings. The spelled-out aliases add no new §3 or §4b artifacts; `DESIGN 5` adds `docs/plans/deploy-convergence-observed-side.md` to the §5 set. These are review surfaces, not proposed edits.

### Documents citing §3 (56)

- `docs/plans/budget-tree.md`
- `docs/plans/compile-clean-forcecheck.md`
- `docs/plans/content-hash-family-grounding.md`
- `docs/plans/seed-honesty-discharge-design.md`
- `docs/plans/shell-intent-emit-realization-design.md`
- `docs/plans/effect-namespace-grants.md`
- `docs/plans/cli-run-reconcile-defork.md`
- `docs/plans/v2-self-hosting.md`
- `docs/plans/git-plumbing-extdeps-authority-design.md`
- `docs/plans/gunbc-served-dashboard-design.md`
- `docs/plans/realization-measurement-loop.md`
- `docs/design-ledgers.md`
- `docs/plans/roadmap-spawner.md`
- `docs/plans/ci-minutes-product-design.md`
- `docs/plans/dag-v2-defork-audit.md`
- `docs/plans/layering-imports-reference-repoint-design.md`
- `docs/plans/ci-humming.md`
- `docs/plans/resource-aware-scheduler.md`
- `docs/plans/floor-cut-replacement-plan.md`
- `docs/plans/fabric-concept-reconciliation.md`
- `docs/plans/dispatch-maintain-cc.md`
- `docs/plans/discrete-cost-derivation.md`
- `docs/plans/witness-execution-closure.md`
- `docs/plans/namespace-resolution-design.md`
- `docs/plans/machine-shape-orthogonal-scheduling.md`
- `docs/plans/dag-native-scm-design.md`
- `docs/plans/type-env-single-authority-design.md`
- `docs/plans/unconsumed-module-census.md`
- `docs/plans/fabric-recut-program.md`
- `docs/plans/dag-scm-design.md`
- `docs/plans/dissolution-census-a-ci-layer-roots.md`
- `docs/plans/parsed-body-projection-increment-spec.md`
- `docs/plans/namespace-cut-replacement-plan.md`
- `docs/plans/space-lens-minimal-project.md`
- `docs/runbooks/bmc-assimilator-wif-setup.md`
- `docs/plans/generated-file-conflict-policy.md`
- `docs/plans/floor-semantic-artifact-design.md`
- `docs/plans/v1-run-stability-throughline.md`
- `docs/plans/membership-diff-reconcile-spine-design.md`
- `docs/plans/cli-invocation-emission-design.md`
- `docs/plans/rc-ownership-wrap-decision-design.md`
- `docs/plans/ci-floor-child-spawn-attribution.md`
- `docs/plans/deploy-convergence-observed-side.md`
- `docs/plans/host-network-attachment-converge-design.md`
- `docs/plans/unconsumed-module-residue-disposition.md`
- `docs/plans/emission-admission-stage-aware-pipeline-design.md`
- `docs/plans/t5b-closure-bearing-serde-debug-decision-2026-08-21.md`
- `docs/plans/floor-expected-red-shrink-monotonicity-design.md`
- `docs/probes/leading_minus_continuation_silently_truncates_2026-08-23.md`
- `docs/plans/self-host-cargo-refusal-root-partition.md`
- `docs/plans/shell-dag-census-0a-projection-blocker.md`
- `docs/plans/floor-time-namespace-walk-regression-diagnosis.md`
- `docs/plans/namespace-unique-on-chain-operational-plan.md`
- `docs/plans/shell-to-dag-residual-census-and-arc-completion.md`
- `docs/plans/import-strip-witness-discovery-cascade-diagnosis.md`
- `docs/plans/keying-relation-design.md`

### Documents citing §4b (19)

- `docs/design-ledgers.md`
- `docs/plans/ci-minutes-product-design.md`
- `docs/plans/witness-execution-closure.md`
- `docs/plans/fabric-recut-program.md`
- `docs/plans/unconsumed-module-census.md`
- `docs/plans/runner-service-capacity-convergence.md`
- `docs/plans/dag-native-scm-design.md`
- `docs/plans/floor-cut-replacement-plan.md`
- `docs/plans/floor-semantic-artifact-design.md`
- `docs/plans/discrete-cost-derivation.md`
- `docs/plans/cli-invocation-emission-design.md`
- `docs/plans/namespace-cut-replacement-plan.md`
- `docs/plans/rc-ownership-wrap-decision-design.md`
- `docs/probes/leading_minus_continuation_silently_truncates_2026-08-23.md`
- `docs/plans/membership-diff-reconcile-spine-design.md`
- `docs/plans/unconsumed-module-residue-disposition.md`
- `docs/plans/self-host-cargo-refusal-root-partition.md`
- `docs/plans/emission-admission-stage-aware-pipeline-design.md`
- `docs/plans/shell-to-dag-residual-census-and-arc-completion.md`

### Documents citing §5 (59)

- `docs/plans/ci-humming.md`
- `docs/plans/budget-tree.md`
- `docs/plans/compile-clean-forcecheck.md`
- `docs/plans/type-env-single-authority-design.md`
- `docs/plans/s2-v2-self-emit-brief.md`
- `docs/plans/s2-v2-self-emit-direction.md`
- `docs/plans/resource-aware-scheduler.md`
- `docs/plans/seed-honesty-discharge-design.md`
- `docs/plans/cli-run-reconcile-defork.md`
- `docs/plans/ci-floor-child-spawn-attribution.md`
- `docs/plans/dag-v2-defork-audit.md`
- `docs/design-ledgers.md`
- `docs/plans/roadmap-spawner.md`
- `docs/plans/space-lens-minimal-project.md`
- `docs/plans/membership-diff-reconcile-spine-design.md`
- `docs/plans/post-engine-pr-roadmap.md`
- `docs/plans/v2-self-hosting.md`
- `docs/plans/effect-namespace-grants.md`
- `docs/plans/realization-measurement-loop.md`
- `docs/probes/leading_minus_continuation_silently_truncates_2026-08-23.md`
- `docs/plans/dispatch-maintain-cc.md`
- `docs/plans/cli-run-hollowing-plan.md`
- `docs/plans/witness-execution-closure.md`
- `docs/plans/dag-scm-design.md`
- `docs/plans/floor-semantic-artifact-design.md`
- `docs/plans/namespace-resolution-design.md`
- `docs/plans/repo-stability-2026-08.md`
- `docs/plans/ci-minutes-product-design.md`
- `docs/plans/discrete-cost-derivation.md`
- `docs/plans/v1-run-stability-throughline.md`
- `docs/plans/keying-relation-design.md`
- `docs/plans/dag-native-scm-design.md`
- `docs/plans/fabric-recut-program.md`
- `docs/plans/parsed-body-projection-increment-spec.md`
- `docs/plans/floor-shared-fill-ledger.md`
- `docs/plans/five-minute-ci-gate-design.md`
- `docs/plans/floor-expected-red-shrink-monotonicity-design.md`
- `docs/plans/shell-intent-emit-realization-design.md`
- `docs/plans/unconsumed-module-census.md`
- `docs/plans/roadmap-workspace-remodel-plan.md`
- `docs/plans/fabric-concept-reconciliation.md`
- `docs/plans/parse-grammar-choice-overlap-residue-finding.md`
- `docs/plans/cli-invocation-emission-design.md`
- `docs/plans/machine-shape-orthogonal-scheduling.md`
- `docs/plans/rc-ownership-wrap-decision-design.md`
- `docs/plans/git-plumbing-extdeps-authority-design.md`
- `docs/plans/host-network-attachment-converge-design.md`
- `docs/plans/generated-file-conflict-policy.md`
- `docs/plans/layering-imports-reference-repoint-design.md`
- `docs/plans/emission-admission-stage-aware-pipeline-design.md`
- `docs/plans/floor-time-namespace-walk-regression-diagnosis.md`
- `docs/plans/import-strip-witness-discovery-cascade-diagnosis.md`
- `docs/plans/unconsumed-module-residue-disposition.md`
- `docs/plans/shell-dag-census-0a-projection-blocker.md`
- `docs/plans/self-host-cargo-refusal-root-partition.md`
- `docs/plans/namespace-flip-last-28-root-a-two-std-defork.md`
- `docs/plans/shell-to-dag-residual-census-and-arc-completion.md`
- `docs/plans/t5b-closure-bearing-serde-debug-decision-2026-08-21.md`
- `docs/plans/deploy-convergence-observed-side.md`

## Downstream implications

The DESIGN additions do not themselves implement a wall. They make the following constructions legitimate and reviewable.

### Candidate lenses and walls

- **Meaning-fork lens.** Over a declared name carrier and effective version/epoch, group every reachable material contract by visible name. Refuse when one name reaches more than one set of obligations, floor, billing consequence, refusal behavior, or remedy. Where the product identity can carry the material contract structurally, construction is preferred and the lens covers only residual external names.
- **Quality-floor contract lens.** Every opaque service dimension must resolve to a falsifiable floor and a named consequence. A metric with no billing, remedy, or refusal consumer is reported as marketing, not contract evidence.
- **Named-degradation wall.** A state whose material contract differs from the base product cannot project the base product's identity. Admission requires a distinct product identity, price/terms authority, and applicable floor.
- **Externalization trace lens.** Join a modeled risk assumption or responsibility receipt to the adverse-state disposition and its burdened counterparty. Refuse an unchanged-contract arm when the marginal burden leaves the named risk holder without a separately named transfer contract.
- **Contract-rung witness discipline.** Each wall needs a positive control and a discriminating red on the real acceptance path. A type name, prose rule, inert lens, or unconsumed remedy row establishes no rung.

### Existing public constructions placed under the higher bar

This is a representative implication census, not the operator's carved-out corpus-wide violation audit.

- `gunbc.provider_interface_binding.ProviderInterfaceBinding` already documents one word, “provider,” denoting two subjects. A meaning-fork lens would turn that annotation-level warning into a check over declared subject/name carriers.
- `product.fabric.isolation.IsolationGuarantee`, `IsolationProfile`, `tenant_workload_isolation_requirement`, and `unsatisfied_isolation_guarantees` already state service dimensions and missing guarantees at identity grain. The higher bar asks the customer-facing product contract to bind those guarantees to an admission refusal or remedy rather than merely carrying vocabulary.
- `product.fabric.supply.SupplierOffer`, `OfferFungibility`, `IsolationNotProvided`, `DemandOfferAffordability`, and `offer_fungibility_for` already refuse several unmet service conditions. These are natural consumers for a named product contract and quality-floor relation; an offer cannot be advertised under a class whose material guarantees it cannot satisfy.
- `product.fabric.supply.OfferQuote` already distinguishes an explicit zero price from absence of pricing, preventing an unpriced burden from being silently treated as free. Externalization makes that same distinction a general boundary rule.
- `gunbc.output_policy.ExpectedOutcome`, `StreamDisposition`, and `effect_stream_disposition` already separate declared expectation from plausible silence. They are not commercial carriers, but they demonstrate the fail-closed shape a service remedy must have.
- `docs/plans/ci-minutes-product-design.md`, especially “ExecutionClass is the stable product seam” and “What a SKU promises,” becomes the nearest public product-design consumer. Its `gunbai-8c32g-short` example and proposed `ExecutionClass` contract must resolve one visible class name to one material requirement/floor/remedy contract. Its entitlement-versus-variance-bound decision cannot remain one name with two simultaneous meanings.

## Private executing evidence and its current rung

`strategy.pricing_decomposition` already carries three useful constructions:

- `QualityDimension`, `quality_dimensions`, and `floor_met`;
- `RiskInterface`, `risk_interfaces`, and the customer/employee/firm partition;
- `PricingLayer`, `pricing_layers`, `product_alpha_permille`, and `below_threshold_needs_declaration`.

`test.claim.pricing_decomposition_witness.pricing_decomposition_witness_main` executes the three-way quality split, both sides of `floor_met`, all three risk-interface sides, the three pricing layers, alpha arithmetic, and the declared-subsidy wall. Those are executing examples for the quality-floor and risk-interface consequences.

The evidence boundary must remain honest:

- `strategy.pricing_decomposition.accuracy_of_meaning_ruling` is a `NonEmptyStr` prose value.
- `pricing_decomposition_witness_main` does not import or evaluate it.
- The private witness therefore does **not** establish a generic meaning-fork wall.
- The risk-interface roster establishes typed coverage of three counterparties, but no generic predicate yet proves that every adverse disposition either remains with the firm or becomes a separately named transfer contract.
- `floor_met` is an executable threshold predicate, but a production admission/billing/refund path consuming it is a further obligation.

The private PR is thus executing evidence for the proposed service-floor shape and selected risk/pricing declarations, not proof that the generalized DESIGN consequences are already enforced.

## Operator-supervised application transaction

1. Edit only `gunbc.design_document.section_3_blocks`, `section_4b_blocks`, and `section_5_blocks` using the source copy at the end of this plan.
2. Add the two proposed `RecurringFailureMode` values and append them to `recurring_failure_mode_roster`.
3. Run `gunbc.ci_spec.ci_heal_author_commit_regen_command`, which invokes `tools.generated_artifact_gate.main_wet`.
4. Require the generated diff to be limited to `DESIGN.md` and `docs/design-ledgers.md`, with the paragraph and roster changes enumerated above.
5. Verify that `gunbc.design_document.design_drifted` and the design-ledger generated-artifact comparison are green.
6. Review the generated prose in serial context, not as isolated inserts: §3 must introduce the semantic dual; §4b must apply rung honesty; §5 must derive the organizational-boundary trap from §2 without restating §2.
7. Leave the corpus-wide violation audit to the operator as ruled.

## §6 terminal-architecture consumption test

This plan passes §6's independent reviewer test because it has a named terminal consumer: the operator-supervised authority edit. The **NOT APPLIED** source copy below is intended to be consumed substantially unchanged into `gunbc.design_document`; the ledger drafts immediately preceding it are intended to be consumed into `gunbc.recurring_failure_mode`. This document is not a parallel authority and cannot generate either projection. After consumption, the `.dag` authorities remain the only live source. The plan's own terminal disposition — retention as the historical placement record, deletion, or registration under a `HandAuthoredDocBind` row — is an operator decision made at approval time, not a fact this document can settle about itself; no bind row is proposed here, and the earlier attempt to self-register one (#9767) was correctly rejected as an authority change outside the one-file boundary.

## Proposed recurring-failure-mode source copy — NOT APPLIED

```dag
data meaning_fork: RecurringFailureMode = RecurringFailureMode {
  identity: "meaning_fork" as NonEmptyStr,
  authored: "**meaning fork** (one name carries materially different semantic contracts under hidden state. The subject is scoped: the key is the naming surface, the visible name, and the declared effective version or epoch — the same spelling in two explicitly distinct scopes or across a declared version transition is legitimate reuse, not a fork. Recognition rule: hold that key constant and vary the hidden state; if obligations, quality floor, refusal behavior, billing consequence, or remedy change without a distinct variant name, the name has forked. This is the one-name/two-meanings dual of nicknaming.)",
  evidence: [],
}

data externalized_degradation: RecurringFailureMode = RecurringFailureMode {
  identity: "externalized_degradation" as NonEmptyStr,
  authored: "**externalized degradation** (an actor paid, trusted, or assigned to absorb a risk or cost silently shifts it to a counterparty while preserving the apparent name, price, or contract. Recognition rule: identify the risk receipt or assigned responsibility, trigger the adverse state, and follow who bears the marginal burden; if the counterparty's burden rises while the firm neither absorbs it through its reserve, refusal, or remedy path nor exposes a separately named and priced transfer, the risk was re-exported. The honest arms are absorb-and-reserve or separately name-and-price the transfer.)",
  evidence: [],
}
```

Proposed roster tail — **NOT APPLIED**:

```dag
  identity_absent_graph_traversal,
  meaning_fork,
  externalized_degradation,
]
```

## Exact proposed `gunbc.design_document` insertion text — NOT APPLIED

Insert in `gunbc.design_document.section_3_blocks` immediately after the opening nicknaming paragraph:

```dag
    p(text: "Single authority applies to meaning, not only to code symbols. Nicknaming gives one meaning two names; its dual, a **meaning fork**, gives one name two materially different meanings. Product names, service names, tier names, status names, and terms in every layer are semantic carriers: within one naming surface and one declared effective version or epoch, holding the name constant may not silently change the obligations, quality floor, refusal behavior, billing consequence, or remedy — while the same spelling in explicitly distinct scopes, or across a declared version transition, is legitimate reuse. A materially different contract needs a materially different name."),
```

Insert in `gunbc.design_document.section_4b_blocks` immediately after the four meta-obligations and before “Every newly discovered error class…”:

```dag
    p(text: "At a service boundary, rung honesty has a commercial consequence: a dimension may remain opaque only above a falsifiable quality floor with a named consequence. A claim that can change billing, trigger a remedy, or require refusal is a contract; without such a consequence it is marketing and establishes no rung. Delivery below the floor must refuse or discharge the remedy; a materially different delivery may be admitted only as its own named and priced product — a different contract subject, not a declared drop of the premium one; and a temporary loss of the ability to verify or enforce a floor is a §4b(3) declared drop on that same subject, which may force the commercial path to refuse and never authorizes silent below-floor delivery. Deviation is allowed; silence is refused."),
```

Insert in `gunbc.design_document.section_5_blocks` immediately after the absorbing-fallback paragraph:

```dag
    p(text: "When §2's later fixer is outside the firm, deferred cost becomes **externalization**. A risk intermediary fails open across the organizational boundary when it quietly re-exports a risk or degradation that a counterparty paid it, trusted it, or assigned it to absorb, or pushes an unpriced cost onto users, employees, sellers, liquidation buyers, neighbors, or future maintainers. There are only two honest arms: absorb the risk and reserve for it, or expose the transfer as a separately named and priced contract; a materially degraded service is therefore its own product. Keeping the old name, price, or contract while the counterparty bears the changed burden is **externalized degradation**."),
```
