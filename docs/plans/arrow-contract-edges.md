# Arrow contract edges (XL-2 PR2a)

Parent design: [service-interface-and-uses-carriers.md](service-interface-and-uses-carriers.md) (#12851). Ruling: the side chat, 2026-10-01, option A, admitted with no operator sign-off needed. The ruling forbids four things, none of which this change does: a new connective or behaviour, an annotation bag, unknown labels, and a second carrier for one fact.

## What lands

An Arrow may carry two optional **contract edges**:

| Edge | Target shape | Vocabulary home | Admitted labels |
| --- | --- | --- | --- |
| `^arrow_effect_claims_edge` | NONEMPTY Conj of self-named childless Atoms, no label twice (no claims has one form: the absent edge) | `std.effects` `EffectClaim` (`ReadonlyClaim`, `IdempotentClaim`), new here | `readonly`, `idempotent` |
| `^arrow_execution_mode_claim_edge` | one childless Atom | `std.execution_mode` `ExecutionMode` (existing) | `hermetic` (only `Hermetic` is claimable) |

`hermetic` is kept off the effect claims: it says where the Arrow may run, not what its effect does. The surface `OperationModifier` stays one syntax vocabulary (`v2.extdeps.languages.dag` `dag_grammar_op_modifier_expr`), and lowering (PR2c) projects each arm to its home.

**Three layers, one fact each:**
- **Structure: `v2.std.node`.**
  - `arrow_named_edge_is_contract`.
  - `arrow_named_edge_is_non_binder`, the one predicate every count of an Arrow's binders reads, here and in `v2.std.type_binder` `arrow_named_labels_conform`, which no longer restates an allowlist.
  - `arrow_signature_edges_conform`: at most one of each contract edge, and never a binder. This module names no claim or mode.
- **Vocabulary: the homes.** `std.effects` `EffectClaim` and `std.execution_mode` `ExecutionMode`.
- **Spelling and check: `v2.std.arrow_contract`.**
  - `effect_claim_label` and `execution_mode_claim_label` are exhaustive projections from the home types. They are the only place the surface spellings appear, so a variant added to a home must be decided there.
  - `arrow_contract_conforms` checks each contract edge's target against the homes. `v2.std.type_binder` `type_binder_node_conforms` runs it on every Arrow.

**Declared residue (🟡).** Reading a label back to its variant (`effect_claim_of_label`, `execution_mode_of_claim_label`) compares it against each variant's projection. That is a hand list of the variants, because the language cannot fold over a closed coproduct's constructors. A variant missing there is refused, never admitted, so the residue fails closed. **Bound:** two claims, three modes. **Owner:** XL-2. **Trigger:** a language capability to enumerate a closed coproduct's constructors; the inverse is then derived from the projection and deleted. For the same reason, the edge target is a label checked against the home rather than a value of the home type: a Node edge carries Symbols, and making a wrong claim unwritable would need typed node targets. That is a substrate capability beyond this ruling, which forbids a new connective without operator sign-off.

Properties required by the ruling, and where each is established:
- **Contract edges are not binders.** `arrow_named_edge_is_non_binder`. Test: `admitted_contract_edges_conform_and_are_not_counted_as_a_binder`.
- **Duplicates and unknown labels refuse.** Tests: `a_malformed_effect_claim_refuses` and `a_malformed_execution_mode_claim_refuses`. The reds include an empty claims edge (a second identity for "no claims"), `hermetic` among the effect claims, and `wet`, a real `ExecutionMode` that is not claimable. Every test checks both walls, the structural one and `v2.std.type_binder` `type_binder_node_conforms`, so a positive control goes red if `arrow_named_labels_conform` omits either label.
- **Canonical identity is order-independent, and a claim is part of identity.** Test: `contract_edge_identity_is_order_independent`, over `v2.std.node` `content_hash`.
- **A claim-free Arrow keeps its identity.** It carries neither edge, so nothing about it changes.

All tests are in `v2.test.claim.arrow_contract_edge_conformance`.

## Readers that must not consume the edges

Audit of every Arrow reader outside `src/v2/test`. "Metadata" means: handled exactly as the declared-order edge is.

| Reader | Today with an unknown named edge | Owed |
| --- | --- | --- |
| `v2.std.node` `arrow_signature_edges_conform` | counted as the binder | **this change** |
| `v2.std.type_binder` `arrow_named_labels_conform` | refuses | **this change**: reads `arrow_named_edge_is_non_binder` |
| `v2.std.type_binder` `type_binder_node_conforms` | — | **this change**: runs `v2.std.arrow_contract` `arrow_contract_conforms` |
| `v2.std.type_binder` `node_inferred_subtree_nodes` | walked as inferable | skip (metadata). Owed by PR2c: no producer emits the edges before then |
| `v2.compiler.resolve` `resolve_arrow_node_in` | resolved under the body scope | carry unwalked (metadata). PR2c |
| `v2.compiler.infer` `infer_gather_fold_step` / `infer_arrow_signature_order_edge` | inferred as a value child | skip predicate. PR2c |
| `v2.compiler.infer` `infer_product_child_evidence_edges` (the Arrow's derived TYPE) | its type enters the derived type | rides as metadata. PR2c |
| `v2.compiler.translate` `translate_grounding_derived_gate_subtree` | requires grounding for every subtree node | exempt (metadata). PR2c |
| `v2.std.node` content hash (`canonicalize_arrow_labeled`) | hashes every edge, named edges sorted by label | in the hash, order-independent (tested here) |

Readers that ignore a named edge safely: the domain, codomain and body readers in resolve, infer, eval, translate and `v2.std.compilers.target_model`; `v2.std.node_query`; `v2.std.decl_index`; `v2.compiler.symbol_index_fill`; `v2.compiler.emit_produced`; `v2.compiler.compile`; `v2.std.node` `cost_edge_role`; `v2.lens.cost.copied_port_derivation`.

## Standing: a DESIGN §3c declared frontier

- **Consumed by execution in this change:** the conformance wall (`v2.std.node` `arrow_signature_edges_conform`, reached by every `well_formed` check of an Arrow) and `v2.test.claim.arrow_contract_edge_conformance`.
- **Production consumer, a named later change:** PR2c. The operation-modifier lowering (`v2.compiler.body_lowering_fold` `body_lower_operation`) emits both edges, and the pipeline readers above gain their metadata arms in the same change.
- **Transition trigger:** PR2c lands. If PR2c is abandoned, these edges and their conformance are deleted, not left standing.
- **Why ahead of the consumer:** the 2026-10-01 ruling asked for the substrate change to be reviewable on its own diff.

**Why the pipeline readers move in PR2c rather than here.** Until PR2c, no producer emits a contract edge, so those readers have no input to be wrong about. Moving them here would give them branches that no test can reach. PR2c lands the producer and the reader arms together, with a discriminating red on the production route.

## Not in this change: input defaults

Per the ruling, a default belongs to the binder and not to its type. One binder representation (a required type, an optional single default, closed labels, one reader and builder) serves fn parameters, service io fields and record fields. That changes the existing name-to-type binder shape, so it is a replacement migration (DESIGN §3) and is PR2b: its producers and readers are measured first, then all move together.
