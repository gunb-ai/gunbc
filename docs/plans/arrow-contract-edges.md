# Arrow contract edges (XL-2 PR2a)

Parent design: [service-interface-and-uses-carriers.md](service-interface-and-uses-carriers.md) (#12851). Ruling: the side chat, 2026-10-01, option A, admitted with no operator sign-off needed. The ruling forbids four things, none of which this change does: a new connective or behaviour, an annotation bag, unknown labels, and a second carrier for one fact.

## What lands

An Arrow may carry three optional **contract edges**. The first two (effect claims and execution mode) landed in PR2a. The third (resource requirements) landed with D13 step (a); see the section on it below.

| Edge | Target shape | Vocabulary home | Admitted labels |
| --- | --- | --- | --- |
| `^arrow_effect_claims_edge` | NONEMPTY Conj of self-named childless Atoms, no label twice (no claims has one form: the absent edge) | `std.effects` `EffectClaim` (`ReadonlyClaim`, `IdempotentClaim`), new here | `readonly`, `idempotent` |
| `^arrow_execution_mode_claim_edge` | one childless Atom | `std.execution_mode` `ExecutionMode` (existing) | `hermetic` (only `Hermetic` is claimable) |
| `^arrow_resource_requirements_edge` | one of THREE declared forms: a NONEMPTY Conj, one Named edge per member (label = spelling, target = type reference); the childless Atom `^requirements_declared_none` (`requires none`); the childless Atom `^requirements_opaque` (`requires opaque`). An ABSENT edge is an undeclared operation, read as Undecided (see the reversal below) | the open set of `resource` declarations, checked at resolution (`v2.std.symbol_index` `symbol_index_declares_resource_at`) | any declared resource, each at most once by resolved declaration |

`hermetic` is kept off the effect claims: it says where the Arrow may run, not what its effect does. The surface `OperationModifier` stays one syntax vocabulary (`v2.extdeps.languages.dag` `dag_grammar_op_modifier_expr`), and lowering (PR2c) projects each arm to its home.

**Three layers, one fact each:**
- **Structure: `v2.std.node`.**
  - `arrow_named_edge_is_contract`.
  - `arrow_named_edge_is_non_binder`, the one predicate every count of an Arrow's binders reads, here and in `v2.std.type_binder` `arrow_named_labels_conform`, which no longer restates an allowlist.
  - `arrow_signature_edges_conform`: at most one of each contract edge, and never a binder. This module names no claim or mode.
- **Vocabulary: the homes.** `std.effects` `EffectClaim` and `std.execution_mode` `ExecutionMode`.
- **Spelling and check: `v2.std.arrow_contract`.**
  - `effect_claim_label` and `execution_mode_claims_label` are exhaustive matches over the home types. They are the only place the surface spellings appear, so a variant added to a home must be decided there.
  - `arrow_contract_conforms` checks each contract edge's target against the homes. `v2.std.type_binder` `type_binder_node_conforms` runs it on every Arrow.

**Declared residue (🟡).** A label is admitted by comparing it against each claimable variant's projection (`effect_claim_label_is_admitted`, `execution_mode_claim_label_is_admitted`). That is a hand list of the variants, because the language cannot fold over a closed coproduct's constructors. No decoded variant is produced, because nothing consumes one: lowering (PR2c) projects variants to labels, never the reverse. A variant missing from the list is refused, never admitted, so the residue fails closed. **Bound:** two claims and one claimable mode. **Owner:** XL-2. **Trigger:** a language capability to enumerate a closed coproduct's constructors; the list is then derived from the projections and deleted. For the same reason, the edge target is a label checked against the home rather than a value of the home type: a Node edge carries Symbols, and making a wrong claim unwritable would need typed node targets. That is a substrate capability beyond this ruling, which forbids a new connective without operator sign-off.

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
| `v2.std.type_binder` `node_inferred_subtree_nodes` | walked as inferable | skip every `arrow_named_edge_is_contract` edge. **Landed with D13 step (a)** (the first producer) |
| `v2.compiler.resolve` `resolve_arrow_node_in` | resolved under the body scope | effect claims and execution mode: carried unwalked. Resource requirements: walked in the Arrow's type scope (no value params), each member must name a declared resource, at most once (`resolve_arrow_resource_requirements`). **Landed with D13 step (a)** |
| `v2.compiler.infer` `infer_gather_fold_step` / `infer_arrow_signature_order_edge` | inferred as a value child | skip every contract edge. **Landed with D13 step (a)** |
| `v2.compiler.infer` `infer_product_child_evidence_edges` (the Arrow's derived TYPE) | its type enters the derived type | every contract edge rides as metadata. **Landed with D13 step (a)** |
| `v2.compiler.translate` `translate_grounding_derived_gate_subtree` | requires grounding for every subtree node | exempt (metadata). **Landed with PR2c-i**: the walk skips a contract edge under an Arrow (`translate_edge_is_arrow_contract`, reading `arrow_named_edge_is_contract`), so a claim payload is never required to carry grounding. Discriminator: `v2.test.claim.translate_underived_refusal` `translate_gates_a_positional_payload_but_not_a_contract_payload_holds` |
| `v2.std.node` content hash (`canonicalize_arrow_labeled`) | hashes every edge, named edges sorted by label | in the hash, order-independent (tested here) |

Readers that ignore a named edge safely: the domain, codomain and body readers in resolve, infer, eval, translate and `v2.std.compilers.target_model`; `v2.std.node_query`; `v2.std.decl_index`; `v2.compiler.symbol_index_fill`; `v2.compiler.emit_produced`; `v2.compiler.compile`; `v2.std.node` `cost_edge_role`; `v2.lens.cost.copied_port_derivation`.

## Standing: DESIGN §3c, per edge family

The two families have different producers and different consumers, so each has its own standing.

### Effect claims and execution mode (PR2a, produced by PR2c-i): consumer a declared frontier

- **Consumed by execution:** the conformance wall (`v2.std.node` `arrow_signature_edges_conform`, reached by every `well_formed` check of an Arrow) and `v2.test.claim.arrow_contract_edge_conformance`. The pipeline reader arms (resolve carries both unwalked; infer and `type_binder` skip them) landed with D13 step (a), the first producer of any contract edge.
- **Produced by execution (PR2c-i):** the operation-modifier lowering (`v2.compiler.body_lowering_fold` `body_lower_operation_modifier`) emits both edges from `readonly`, `idempotent` and `hermetic`, and refuses a repeated modifier at the second (`v2.test.claim.normalize.operation_modifier`). `translate_grounding_derived_gate_subtree` carries them as metadata in the same change.
- **Consumer of the claims themselves, still a declared frontier:** no stage yet reads an effect claim or the execution mode to decide anything (retry admission, caching, hermetic placement). Trigger: the first such reader lands and consumes the edge off the operation Arrow; until then the conformance wall and the reader arms are the consumers.
- **Why ahead of the consumer:** the 2026-10-01 ruling asked for the substrate change to be reviewable on its own diff.

### Resource requirements (D13 step a): producer landed, demand consumer a declared frontier

- **Produced and consumed by execution in step (a):** `body_lower_operation_requires` emits the edge from an operation's `requires` clause. `v2.compiler.resolve` `resolve_arrow_resource_requirements` reads it on every resolved operation Arrow and refuses a member that is not a declared resource or that repeats one. Controls: `v2.test.claim.normalize.operation_requires_edge`. That is a conformance consumer: it establishes that the requirement names real resources. It does not use the requirement.
- **Production consumer of the requirement itself, a named later change:** D13 step (b), the DependencyDemand derivation. A function's demand is the union of three parts: its direct resource operations; for each service operation it calls, that callee Arrow's `^arrow_resource_requirements_edge`; and its resolved callees' demand. That is how a `net: Network` requirement becomes derived instead of authored, which is the restoration trigger of `gunbc.rung_drop` `network_requirement_unrepresented_after_uses_cut`.
- **Transition trigger, at executable grain:** step (b) lands, and its derivation reads this edge off the callee operation's Arrow into the calling function's derived demand. Executed evidence is a control in which a function calling an operation that `requires Network` derives demand naming `std.resources.Network`, and that demand disappears when the clause is removed. Reading the edge only for conformance, or adding Network to demand by spelling, does not satisfy the trigger.
- **If step (b) is abandoned:** the `requires` grammar member, `body_lower_operation_requires`, the `^arrow_resource_requirements_edge` label and its at-most-one count, its target check in `v2.std.arrow_contract`, and `resolve_arrow_resource_requirements` with its two refusal reasons are deleted. The resource declaration's index mark (`symbol_index_declares_resource_at`, from #12898) then has no reader left, so it is deleted with them, along with its `resource_declarations` carrier.

## Not in this change: input defaults

Per the ruling, a default belongs to the binder and not to its type. One binder representation (a required type, an optional single default, closed labels, one reader and builder) serves fn parameters, service io fields and record fields. That changes the existing name-to-type binder shape, so it is a replacement migration (DESIGN §3) and is PR2b: its producers and readers are measured first, then all move together.

## Third edge: resource requirements (D13 step a)

`^arrow_resource_requirements_edge` (`v2.std.node` `arrow_resource_requirements_label`) is the operation's declared `requires R, ..` (`v2.extdeps.languages.dag` `dag_grammar_op_requires_expr`), lowered by `v2.compiler.body_lowering_fold` `body_lower_operation_requires`. Its target is a nonempty Conj with one Named edge per member: the label is the member's spelling, and the target is its type reference. Unlike the two claim edges, it names DECLARATIONS from an open set, so `v2.compiler.resolve` walks it and checks each member against `v2.std.symbol_index` `symbol_index_declares_resource_at`. It is the first producer of any contract edge, so the pipeline reader arms above landed with it, for all three edges. Controls: `v2.test.claim.normalize.operation_requires_edge`.

### Reversal: an absent requirements edge is UNDECLARED, not "none" (D13 ruling B)

**This reverses #12911**, where "no requirements has one form, the absent edge". D13 step (b) derives a function's demand from the operations it calls. Under the old reading, every operation nobody audited, and every one added later, would silently read as requiring nothing. That is the fail-open default DESIGN section 5 forbids, so the lane owner ruled it out (option B). Every operation now declares one of three forms, all on this one edge, with no second carrier:

- `requires R, ..`: the named resources, checked at resolution as above.
- `requires none`: **no requirement in the audited vocabulary.** Today that vocabulary is `Network` only (the D13 Network audit, `docs/plans/d13-network-audit.md`). It is not a claim about Filesystem, Clock or Entropy: their demand derives from direct capability calls and is never authored. When the audited vocabulary grows, the meaning of `none` grows with it, and the rows are re-audited.
- `requires opaque`: the operation's demand is decided by runtime values (a generic exec, `shell.exec Run`, `systemd-run`, a runtime-program runner), so a caller's derived demand is **Undecided**, a located refusal at the uses-wall, never empty.

An operation with no clause carries no edge. Step (b) reads that as **Undecided**, exactly like `opaque`, so silence can never mint "none". The grammar (`dag_grammar_op_requires_expr`) makes `none` and `opaque` whole clauses: a member after either refuses. The v1 seed carries the same three forms (`v1.compiler.parse` `operation_requires_declaration`: `RequiresUndeclared | RequiresNone | RequiresOpaque | RequiresResources`). Controls: `v2.test.claim.normalize.operation_requires_edge` and `test.claim.v1_operation_requires_parse_witness_test`.
