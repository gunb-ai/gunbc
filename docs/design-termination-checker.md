# Design: Termination Checker (fuel-elimination, lane C1)

> **Status: DESIGN — map, not territory** (INVARIANTS.md "Map vs territory"). Nothing in this
> doc is landed behavior; no code enters the active tree from it without the consumer named in
> §7 (E-10). Part of P4 (Decidability) — this is the design that makes the THESIS CX gate
> ("every recursive function terminates with a proven bound") a checked structural fact instead
> of a threaded runtime budget.
>
> Governing principle (from `dsl/std/termination.dag` header, restated by INVARIANTS P1 worked
> example): **the analyzer is a CHECKER of termination proofs, not a DISCOVERER.** Candidate
> proofs are constructed from declared structural facts; the checker validates them against the
> model; no valid proof → reject (fail-closed), never assume-terminates.

## 1. Problem

v2 has no compile-time termination checker. Recursive clusters in the compiler thread an
explicit budget instead: a `*_bounded` wrapper computes a measure, a `*_go` worker threads
`remaining: Int`, and a `remaining <= 0` guard rejects at runtime with a typed `Outcome`.
This is fail-closed (good) and the budgets are structurally derived, not fixed constants
(`06_translate.dag:8` — `translate_serialize_measure` = `node_subtree_count` + translation-rules
subtree; no `fuel: 1024`), but it is still an **operational substitute for a proof**:

- The bound is checked by *running out*, not by *descending*. A genuinely non-terminating
  recursion and a too-small budget are indistinguishable at the failure site.
- Every recursive cluster pays a threading tax: one extra parameter on every function in the
  cluster, one guard per entry, one measure computation per wrapper. The triads are the
  single largest accidental-complexity pattern in `06_translate`.
- The emitted stage0 Rust mirrors the threading (`fuel: i64` in
  `src/v1/stage0/src/v1_compiler_emit*.rs`) — the pattern propagates through self-host.

**Measured (2026-06-09, main):** `06_translate.dag` is 4,490 lines with **20 `remaining <= 0`
guards and 40 `remaining: Int` parameters** — all budget-threading in the v2 compiler tree is
in this one file (the earlier census's "33 triads" has drifted slightly; the localization
claim still holds). `fuel`-named threading otherwise appears only in `std/target_model.dag`,
`01_tokenize.dag`, `02_parse.dag` (small, same pattern) and the emitted v2 stage0 mirrors.

Three sub-problems, increasing hardness:

- **(a) Structural-fold recursion** — functions that recurse along `Node` children (directly or
  via `fold_node`). Terminating by construction (subtree strictly smaller); the checker's job
  is to *recognize and validate* that the recursion actually routes through a child-descent,
  then let the triad dissolve.
- **(b) Mutual recursion — the serialize SCC.** `serialize_type_expr_generic_apply_bounded →
  serialize_type_expr_emitted_bounded → serialize_type_expr_{args,separated,record_fields}_go →
  serialize_type_expr_emitted_bounded` (`06_translate.dag:2501-2910`) is a genuine
  strongly-connected cluster. Needs a per-SCC lexicographic proof, not a per-function one.
- **(c) Infer fixpoints** — constraint-solving iteration in `04_infer` is not structural
  descent at all; its termination argument is lattice-height descent (Kleene). Different
  proof shape; see §4.4.

## 2. What already exists (M9 DFS — do not re-invent)

Per MODELING.md M9 the concept DAG was DFS'd before proposing anything new. Findings:

| Concept | Where it lives today | State |
|---|---|---|
| `DescentEvidence = Strict \| NonIncreasing \| DescentUnknown` | `src/v2/std/cardinality.dag:115` | landed (v2) |
| `RankingDimension { measured: Symbol }` | `src/v2/std/cardinality.dag:119` | landed, **degenerate** (single Symbol; cannot say *what kind* of measure or *which parameter*) |
| `TerminationProof { non_increasing: List<RankingDimension>, strict: RankingDimension }` | `src/v2/std/cardinality.dag:122` | landed, **single-level** (exactly one strict dimension; cannot express the lexicographic proofs SCCs need) |
| `termination_proof_witness_for_node` (node-shape fold: `Loop`/`Cardinality` ⇒ proof required, fail-closed) | `src/v2/std/cardinality.dag:283` | landed; **consumed by `04_infer`** (`infer_descent_witness_for_node` → `InferredFacts.descent`, `04_infer.dag:301`) — this is the working consumption pattern to extend |
| `LoopBound { measure, termination: Witness<TerminationProof> }` | `src/v2/std/cardinality.dag:130` | landed; the `Loop` primitive already carries a proof slot |
| Full proof theory: 5-variant `RankingDimension`, `DescentSource`, `ProofEdge`, lexicographic semantics, `BoundedLattice<DescentEvidence>` inhabitance | `dsl/std/termination.dag` (v2-era) | **port source** — rich, externally grounded (Floyd 1967, Lee/Jones/Ben-Amram 2001, Dershowitz/Manna 1979) |
| The checker algorithm: `is_valid_proof(proof, edges)` = per-edge lexicographic check, then **the non-descending subgraph must be acyclic** (Kosaraju 2-pass DFS; self-loops count; evidence-length mismatch ⇒ non-descending, fail-closed) | `dsl/std/graph.dag:141-197` | **port source** — already written, decidable, polynomial |
| Dependency/graph machinery in v2: `DependencyView`, `dependency_lens` fold, `ready_set`, `topological_layers` | `src/v2/std/dependency.dag` | landed — the **M9 ancestor for the call graph**; SCC attaches here, not in a new sibling module |

The conclusion of the DFS: **almost nothing new needs inventing.** The design is (i) two
carrier upgrades in `v2.std.cardinality`, (ii) a port of the v2 proof theory + checker onto
the v2 dependency substrate, (iii) proof *constructors* that read declared structural facts,
and (iv) a dissolution ratchet for the triads.

## 3. Substrate-fact introduction procedure (MODELING.md, cited per worked-example tracking)

- **Step 1 (DAG-ancestor):** ran. `TerminationProof` and `DescentEvidence` already exist in
  `v2.std.cardinality` — they are upgraded in place, not declared as siblings. The call graph
  is not a new concept: it is a `DependencyView` consumer (`v2.std.dependency`), and SCC
  computation is a derived operation over it. `ProofEdge` attaches to the existing
  caller/callee dependency concept; no new module is coined for it.
- **Step 2 (coproduct-vs-coordinate):** ran. `RankingDimension`'s five kinds (TreeSize,
  ListLength, ArithmeticValue, TokenPosition, SetCardinality) are **alternatives** — one
  dimension measures one kind of thing — so the sum type is the correct shape (same verdict
  as the v2 model). `DescentEvidence`'s three values are alternatives per call edge — sum
  correct. `TerminationProof.dimensions` is an **ordered list** (lexicographic priority), not
  coordinates and not a sum.
- **Step 3 (primitive-vs-lens-extensible):** ran. Descent evidence is **substrate-declared**,
  not lens-extensible: P4's bounded-forward-execution premise depends on it, and every
  target's recursion has one of these shapes. (A *user-facing* termination lens may later
  read the same facts; the facts themselves are substrate.)

## 4. Design

### 4.1 Carrier upgrades in `v2.std.cardinality` (land atomically — P5, no bridge)

Two shapes are upgraded **in place**, all consumers migrated in the same change:

1. `RankingDimension { measured: Symbol }` → the five-variant coproduct from
   `dsl/std/termination.dag:188`, with the parameter binding carried as `Symbol` (v2's
   existing choice; the v2 file's "should become a structural param reference when the
   language has one" note carries forward unchanged).
2. `TerminationProof { non_increasing, strict }` → `TerminationProof { dimensions:
   List<RankingDimension> }` (lexicographic form). The current single-level shape is
   expressible inside the lexicographic one (`[strict]` with invariant dimensions appended),
   so `structural_node_size_termination_proof()` migrates mechanically.
3. New carrier (port, not invention): `ProofEdge { caller: QualifiedName, callee:
   QualifiedName, evidence: List<DescentEvidence> }` — evidence vector ordered to match
   `proof.dimensions`. v2 used `String`; v2 uses the landed `QualifiedName`
   (`v2.std.qualified_name`) since that is what a declared function identity is.

Known consumers to migrate in the same PR: `structural_node_size_termination_proof`,
`multiplicity_termination_witness`, `termination_proof_witness_for_node` (all in
`cardinality.dag`); `infer_descent_witness_for_node` + `InferredFacts.descent`
(`04_infer.dag`); `LoopBound.termination`. The `DescentEvidence` lattice inhabitance
(`BoundedLattice<DescentEvidence>`, meet/join from `dsl/std/termination.dag:91-124`) ports
alongside — it is the single authority for merging evidence across branches.

### 4.2 Sub-problem (a): the structural-fold checker

**Claim checked:** a recursive call whose decreasing argument is a structural child of the
caller's argument (a `fold_node` step, or a direct child-accessor projection) carries
`TreeSize`-`Strict` evidence **by construction** — `Node` is acyclic (P4 bounded forward
execution), so subtree < tree on a well-founded order.

Mechanism: a proof **constructor** (not discoverer — it reads declared facts) recognizes the
two blessed descent shapes and emits the candidate proof:

- recursion via `fold_node` / `NodeFold` algebra → `TerminationProof { dimensions:
  [TreeSize(param)] }`, every self-edge `[Strict]`;
- recursion via a declared child projection (the `DescentSource::ChildAccessor` fact —
  grounded in the substrate's child-edge model, not in a name heuristic).

Everything else in class (a) — `skip(1)` list recursion, `n - k` arithmetic descent — maps to
the corresponding `DescentSource` (ListShrink / ArithmeticSubtractDescent with the existing
`PositiveDescentAmount` witnesses and the shared 256 materialization cap, INVARIANTS
"E-I numeric input boundary"). Each `DescentSource` is grounded in a structural property
(`|skip(n,l)| = max(0,|l|-n)` from List algebra; OrderedRing subtraction), so constructing
the candidate is fact-reading, not pattern-guessing (P1).

**Payoff:** for every function the constructor covers, the `remaining` parameter, the guard,
and the wrapper's budget computation delete. This is the bulk of the 20 triads.

### 4.3 Sub-problem (b): the SCC checker (the hard residual)

Pipeline, all pieces named:

1. **Call-graph extraction** — a consumer of `v2.std.dependency`'s `DependencyView` fold,
   restricted to call-shaped edges (Transform `FunctionRef` targets), keyed by
   `QualifiedName`. This is module-bounded data (the serialize cluster is one module), so the
   graph is finite by construction.
2. **SCC condensation** — port Kosaraju from `dsl/std/graph.dag:143` (2-pass DFS with
   visited-set; each pass visits each node once; descent dimension for the checker's own
   recursion is `SetCardinality(visited-complement)` — the checker passes itself, see §5).
   Attaches next to `ready_set` / `topological_layers` in `v2.std.dependency` — same module,
   same fold idiom.
3. **Per-SCC proof obligation** — every SCC containing a cycle (multi-node, or any self-loop)
   requires a `TerminationProof` + a `ProofEdge` for every intra-SCC call edge. Missing proof
   ⇒ `Violates` with the existing `cardinality_descent_not_proven` diagnostic. No proof
   search, no timeout.
4. **Validation** — port `is_valid_proof` (`dsl/std/graph.dag:185`) verbatim in spirit:
   an edge is *descending* iff its evidence vector is lexicographically descending
   (first non-`NonIncreasing` entry is `Strict`; `DescentUnknown` anywhere before a `Strict`
   ⇒ not descending); evidence-length ≠ dimension-count ⇒ non-descending (fail-closed);
   then **the subgraph of non-descending edges must be acyclic**. Acyclic ⇒ every cycle
   contains a descending edge ⇒ no infinite descent on a well-founded lexicographic order ⇒
   termination. Polynomial: one lexicographic scan per edge + one SCC pass.

**Prerequisite — call-graph producer (AUDITED 2026-06-09: does not exist; must be built).**
The extraction step above was audited against the live tree and is **not readable today** —
three layers deep, in increasing severity:

1. **No production `BindsTo` facts.** The classifier only recognizes edges literally labeled
   `^dependency_binds_to_edge` (`dependency.dag:126`), and the only writers of that edge in
   the entire tree are **four `lens_structural_resolution` test fixtures** — zero pipeline
   writers. This is exactly `dependency.dag`'s own staged-classifier marker ("dissolve-on:
   T-9 resolve writes BindsTo substrate facts inline"), confirmed unbuilt.
2. **Resolution materializes spelling, not reference.** `resolve_atom`
   (`03_resolve.dag:280`) rewrites a use-site atom to a *canonicalized-spelling* atom; the
   use→def relation survives only as "two atoms share a Symbol." Recovering call edges by
   re-joining atoms to declarations on symbol equality would rebuild the
   **spelling-as-identity** channel that the BRAND lane (#4579/#4581 `binding_id`) exists to
   dissolve — a workaround this design refuses.
3. **Call sites are not in the v2 representation at all.** No stage produces
   `ComputationNode` trees (`02_parse` builds none; `03_normalize`/`05_eval` only match
   them), and THESIS's "Transform holds a FunctionRef to an Arrow declaration" has **no
   landed `FunctionRef` carrier** in `std/node.dag`. The serialize cluster's `.dag` functions
   are executed by the v2 interpreter from source; their call structure has no v2 substrate
   representation today.

**The fix (and it should be built — the operator concurs):** one write at one site. When
resolve binds a use to a def, materialize the relation as a substrate fact **on the
`binding_id` channel that #4581 is building right now** — T-9's "resolve writes BindsTo
inline" and BRAND's authority-direct stamping are the *same write at the same seam*, so T-9
should land as a **rider on #4581**, not a separate lane. One producer then serves three
existing consumers at once: the dependency classifier (its own dissolution marker), the
`structural_resolution` lens (currently fixture-fed), and this checker's call graph. Call
edges specifically arrive when function bodies land as `ComputationNode` trees carrying
def-references on the same channel — sized separately as **COMPREP**
(`design-computation-representation.md`; the checker's call graph queues behind its wave 1)
— until then, the §7 slice is **gated on the producer**,
and the only interim alternative (extracting the cluster's call graph v2-side, where the v2
complexity analyzer already models caller/callee `ProofEdge`s) is explicitly second-choice:
it builds the slice's input on the frozen tree instead of the substrate the checker is for.

**Worked target — the serialize cluster.** Proof: `dimensions = [TreeSize(node)]`. Every
edge in the cluster passes a structural child of its input (`generic_apply` recurses on
`split.head`/argument nodes; `args_go`/`separated_go`/`record_fields_go` recurse on
`split.tail` elements and child nodes; `emitted_bounded` receives a strict subtree from every
caller). Where a list-walk edge is only `NonIncreasing` on TreeSize, the proof extends
lexicographically to `[TreeSize(node), ListLength(items)]` — exactly the
`emit_block_stmts` example already worked in `dsl/std/termination.dag:354-369`. Constructing
this proof and validating it against the *real* extracted call graph is the minimal slice
(§7); if the cluster's actual edges defeat the candidate proof, **that is the checker working**
— the budget threading stays until the cluster is reshaped, and the failure is a located
diagnostic naming the offending edge, not a vibe.

### 4.4 Sub-problem (c): infer fixpoints — shape named, commitment deferred

Constraint-solving iteration terminates by a different argument: a **monotone function on a
finite-height lattice** reaches a fixed point in ≤ height steps (Kleene). The proof carrier
shape is therefore not `DescentSource` but `LatticeDescent { lattice: <inhabitance ref>,
height_bound: <measure> }` — the ranking dimension is "distance below top," strictly
decreasing on every non-fixed iteration.

Deliberately **not designed further here**: no census of actual fixpoint sites in `04_infer`
exists yet, and designing carriers ahead of a measured consumer is the E-10 trap. Front door:
census the `04_infer` iteration sites (count, lattice, monotonicity evidence), then extend
this design with the lattice-descent dimension as a sixth `RankingDimension` variant **only
if the census shows real sites** (Step 1/Step 2 re-run at that point).

### 4.5 Where proofs attach and flow

No new channel: proofs ride the existing path. The constructor + checker run at infer time;
the per-declaration verdict lands in `InferredFacts.descent : Witness<TerminationProof>`
(already there, `04_infer.dag:80`); `Loop` nodes keep `LoopBound.termination` (already
there). The substrate target for per-SCC proofs is the declaration's Arrow node — the proof
is a fact *about* declared functions, carried with the inferred facts for the module, not a
new side table. (P1 "Design Commitments Must Name The Substrate Target": named — no
substrate extension required; C1-class stop signal not in play.)

## 5. Decidability of the checker itself

The checker is `.dag` code and must satisfy its own discipline:

- Lexicographic edge scan: fold over a finite evidence list (`ListLength` descent).
- Kosaraju: two DFS passes with a visited set over a finite, declared node set —
  `SetCardinality` descent on the unvisited complement (the work-list pattern already named
  in `dsl/std/termination.dag:371`).
- Non-descending-subgraph acyclicity: same SCC machinery.
- No search, no widening, no timeout anywhere. Every reject is a located typed diagnostic
  (`cardinality_descent_not_proven`, or a new `cardinality_descent_edge_unproven` naming the
  failing `ProofEdge` — reason symbols owned by the consumer per the SPI-3 note at
  `04_infer.dag:300`).

External authority (unchanged from the v2 model): well-founded relations (Zermelo 1904),
ranking functions (Turing 1949, Floyd 1967), size-change termination (Lee, Jones, Ben-Amram
2001), lexicographic orderings (Dershowitz, Manna 1979), Kleene fixed-point theorem for §4.4.

## 6. Dissolution ratchet (P5)

The fuel triads are the scaffold; the checker landing is the dissolution trigger.

- **Ratchet metric:** count of `remaining: Int` parameters (today **40**) and `remaining <= 0`
  guards (today **20**) in `src/v2/compiler/06_translate.dag` → monotonically to **0**. Each
  dissolution PR deletes a cluster's wrapper + guard + threading and cites the validated
  proof (the `TestClaim` row) as the receipt.
- **Order:** class (a) singles first (mechanical once the constructor lands), then the
  serialize SCC (the §4.3 worked target), then the small `01_tokenize` / `02_parse` /
  `target_model` sites, then (c) pending its census.
- **Forbidden:** new `remaining`/fuel threading in `compiler/` once the checker is landed —
  new recursion either passes the checker or carries an explicitly gated scaffold marker with
  this lane as its bind target.
- **Consequence, not a separate task:** the `fuel: i64` mirrors in emitted stage0 Rust
  dissolve when their `.dag` sources do. **Constraint honored: no Rust-side fuel constant is
  ever introduced** — the fix is upstream, in the source of truth (Root-Cause Depth).

## 7. Consumer and minimal slice (E-10 / seesaw discipline)

- **Consumer (exists today):** `04_infer`'s `InferredFacts.descent` — already a real consumer
  of `Witness<TerminationProof>`; it starts consuming *checked SCC* witnesses instead of
  node-shape-only witnesses. Second consumer: the first dissolved triad (its deletion breaks
  if the checker is wrong — the strongest consumer form).
- **Minimal slice** (exercises the committed shape's risk — SCC + lexicographic — not a toy;
  **step 2 is gated on the call-graph producer per the §4.3 audit** — the T-9-rider-on-#4581
  write must land first, or the slice's input doesn't exist):
  1. carrier upgrades (§4.1) + `is_valid_proof`/Kosaraju port onto `v2.std.dependency`;
  2. extract the real `serialize_type_expr_*` call graph; author its candidate proof;
  3. two `TestClaim`s under `src/v2/test/claim/termination/`:
     **green** — the real cluster's proof validates by execution (`--claim-run`);
     **red-when-wrong** — the discriminating case: same edges with one strict edge's evidence
     degraded to `NonIncreasing` (and a second case: evidence-length mismatch) ⇒ `Violates`.
     Per the reviewer's three questions, the PR shows both runs.
- Everything past the slice (constructor breadth, triad-by-triad dissolution, (c)) is
  consumer-triggered roadmap work, not part of the slice.

## 8. Open questions — escalate, don't improvise

- **Q-T1 — proof authorship surface.** Slice scope is **derived-only** proofs (constructors
  reading structural facts) on compiler-internal clusters. A user-facing annotation surface
  ("here is my ranking") is real but separately gated — it adds a language surface and
  belongs with the audience-duality opt-in depth story, not here.
- **Q-T2 — where the checker runs. RESOLVED (operator 2026-06-09), with a sharpening the
  ruling exposed.** Confirmed: the gate lives in infer. But the operator's challenge —
  "if it's expensive to relocate, that's a design issue" — is correct under cost-of-change,
  so the design makes relocation cheap **by construction**: the checker itself is a pure
  substrate function (carriers + `is_valid_proof` port in `v2.std.{cardinality,dependency}`,
  per §4.1/§4.3) with no infer dependency; what lives in infer is only the **gating wire** —
  the call that routes the checker's `Witness<TerminationProof>` into
  `InferredFacts.descent` and lets `Violates` block admission. Relocating the gate (or
  adding a lens-shaped second reader) is rewiring one consumer of a substrate function,
  not moving the checker. Placement of *computation*: substrate. Placement of *gating
  decision*: infer, confirmed.
- **Q-T3 — single proof per SCC vs per function.** Per-SCC (the v2 model's choice; matches
  Lee-Jones-Ben-Amram). A per-function view falls out by projection.
- **Q-T4 — checker self-application.** The ported checker's own recursion (DFS, work-list)
  must validate under itself before its first dissolution PR merges — the receipt that
  "checker, not discoverer" closed over its own implementation.

## 9. Non-goals

- No new connective, behavior, or substrate primitive (C1-class stop signal not triggered).
- No heuristic discovery, no proof search, no timeouts (P4: decidable, not heuristic).
- No cost/complexity derivation changes in this lane — cost composition from proofs
  (`dsl/std/termination.dag:396-420`) is downstream and already designed; it consumes this
  lane's output unchanged.
