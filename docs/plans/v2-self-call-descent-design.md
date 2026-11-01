# v2 self-call descent: design note (read-only; nothing here is implemented)

Revision read: origin/main 345d6077fc4a for v2 and dag/std; origin/calm-descent-step1 @ 162d06cc839a for the v1 rule. Method: `git grep` and `git show` over those revisions. Scope: src/v2, dag/std, dag/extdeps and the v1 files named below. Not executed; every claim marked UNMEASURED needs the fixture in section 3.

## 0. What is there

- `FnItemDescentFlags { is_self_recursive, has_non_tail_self_call }` (src/v2/std/node_minimal.dag) is a payload of `SurfaceDeclPayload.SdpFnItem`. A `git grep` for `SdpFnItem|FnItemDescentFlags|is_self_recursive` over src/v2 and dag/std finds no other reader or writer; the only other hit is the manual claim v2.test.claim.manual.node_minimal_representation. So the carrier is DESIGN section 3c dangling, as calm-ibex-725 found. It is a field of the minimal representation, not of `v2.std.node.Node`.
- v2 already has a termination authority for repetition, and it is Loop-shaped: `v2.std.cardinality` `behavior_multiplicity` answers `Loop => RequiresTerminationProof` and every other behavior `Bounded`; `loop_multiplicity_with` folds `measure_dimension_rows` over the measure registry (`measure_descent_fact_registry`) with `lexicographic_multiplicity`, whose `Absent` first-strict-dimension case is `RequiresTerminationProof`. The evidence type is `std.termination` `DescentEvidence` (`Strict | NonIncreasing | DescentUnknown`), the same one v1 uses.
- v2 produces a Loop only from the `loop` keyword and `fold` (`v2.std.compilers.body_lowering` `lower_loop`; v2.compiler.body_lowering_fold `body_lower_try_loop_keyword_from_captured`, fold_lowering). I found no lowering in src/v2 that turns a call to the enclosing fn into a Loop (grep for self_call, tail_call, recurs in body_lowering_fold, eval, infer). So a self-recursive fn is, at best, an ordinary application (`ComputationNode { behavior: Transform }`), whose multiplicity is `Bounded` (UNMEASURED: confirm in section 3).
- v1's rule (calm-descent-step1): `std.induction` `call_descent_evidence(relations: List<SubValueRelation>) -> DescentEvidence` (any Strict relation among a call's arguments is Strict, else NonIncreasing, else DescentUnknown), read by `v1.compiler.complexity` `derive_edge_evidence` and by `v1.compiler.infer` `self_call_without_descent_diags_in` (advisory class `SelfCallWithoutDescent`).
- Side finding: `std.graph` `build_call_graph_from_proof_edges` filters edges with `e.caller != e.callee`, so a self-edge never enters that call graph; a census over it cannot see self-recursion.

## 1. Where v2 derives self-recursion

Recommendation: delete `FnItemDescentFlags` (and the `descent` slot of `SdpFnItem`) and DERIVE the fact; do not give anything a writer.

- A stored flag is a second representation of a fact already in the graph (DESIGN section 3). A self-call is a node in fn F's body that is an application whose head resolves to F's own declaration: `v2.std.compilers.body_lowering` `application_head_read` yields the head, and `Edge.declaration` carries the resolved declaration identity. "Self" is decidable from the Node tree by identity, not by spelling (DESIGN section 3: declaration identity, never leaf name).
- Owner: one derivation beside `termination_proof_witness_for_node` in the checker's descent path (`v2.compiler.infer` `infer_descent_witness_for_node` already routes every node through `v2.std.cardinality`), so the obligation attaches where Loop's already does: a node that repeats owes a `TerminationProof`.
- Mutual recursion: a self-edge test misses it. The sound unit is "F reaches F through the call relation", so the derivation must be over strongly connected components of the call relation, not only direct self-calls. v1 names mutual recursion as a residual (`a_self_recursive_call_without_a_descending_argument_is_accepted`). Whether v2 admits that residual or does the SCC is a decision for the operator/M3; I recommend the SCC form because the direct-only form ships a known silent gap.
- `has_non_tail_self_call` goes with the flag: tail-ness is a property of the lowering choice below, not an input to the obligation.

## 2. Lowering, and consuming `call_descent_evidence`

Lowering: today nothing lowers a self-recursive fn to Loop (UNMEASURED). Two honest options: (a) the fn stays an application and the self-call owes the obligation directly (`RequiresTerminationProof` unless a descending argument is proven); (b) lower to Loop with the dag loop measure, which reuses `loop_multiplicity_with` unchanged. Option (a) is what the section 4 sentence "recursion as sugar over Loop" would make equivalent, but (b) is a lowering migration, with its own fail-closed rule (no Strict dimension => `RequiresTerminationProof`). Do (a) first: it makes the obligation exist without choosing a lowering.

Consuming the shared rule: YES at the rule, NO at the input. `call_descent_evidence` takes `List<SubValueRelation>`; a v2 checker must PRODUCE those relations per call (which argument is a strict sub-value of which parameter). v1 produces them in `annotate_descent_evidence`; v2 has no producer. So v2 reuses `std.induction` `call_descent_evidence` and `sub_value_to_evidence` (single authority, dag/std wins v2/dag forks per `v2.std.cardinality`'s own note) and owes only a relation producer over v2 Nodes. Do not write a v2 copy of the combine. Open question: v2 cannot yet import std.induction (only `v2.std.cardinality` imports `std.termination` today); the cross-tree import needs both namespaces in `gunbc.source_layout` (`cross_tree_import_admitted`), which section 3 of DESIGN says is admitted; confirm by an ingest, not by reading.

Interaction with v1: `std.induction` is imported by the v1 seed; changing it re-touches the seed mirrors (`std_induction.rs`).

## 3. RED, controls, and the one fixture

RED (v2 ingest of authored source): `fn spin(n: Int) -> Int { spin(n: n) }`. Expected today: ACCEPTED (UNMEASURED). After the obligation: refused with a typed located diagnostic whose reason names the self-call and the evidence (`DescentUnknown`).
Controls (all must stay accepted): (1) `spin(n: n - 1)` guarded by a base case, arithmetic Strict via `ArithmeticDescent`; (2) a structural fold over a list tail (`StrictSubValue`); (3) a non-recursive fn (no obligation); (4) a call to a DIFFERENT fn of the same name in another module (declaration identity, not spelling); (5) a mutual pair `a -> b -> a` with no descent, once the SCC form is chosen. Mutations: break the identity key (compare by name) and control (4) must fail; replace the combine with "any argument NonIncreasing is enough" and the RED must fail to refuse.
The ONE ingest fixture, for the serial srv1 order: compile the RED source natively and read, for the fn `spin`, whether its body contains a `ComputationNode { behavior: Loop }` and what `loop_multiplicity_with`/`behavior_multiplicity` answers. This settles "does a self-recursive fn lower to Loop"; my reading says no. Shared file: src/v2/compiler/infer.dag is also calm-ibex-725's (v2 Nat-Nat); edits are serialized with that session and this note touches nothing there.

## 4. Decisions needed (not mine)

1. Direct self-call only, or SCC. 2. (a) obligation on the application first, or (b) Loop lowering now. 3. Cross-tree import of std.induction into v2, or relocate the combine into a module both trees already import.
