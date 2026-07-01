# The axiom + syllogism lens — the argument as the fourth reachability substrate

> **SCOPE-ONLY design.** This doc partitions syllogism-enforcement into the [expressibility-frontier](expressibility-frontier.md)'s three regions (① wall · ② lens-residue · ③ undecidable-review) and designs how A1–A3 plus the §1–§7 consequence chain are modeled in `.dag`, with **DESIGN.md itself as the first target** (the §7 recursion). It builds nothing — it exists for the operator nod before any carrier or witness lands. DESIGN refs: §1 (axioms A1–A3; displaced cost), §3 (single authority — turned on the argument), §4 (acyclic substrate — *the acyclicity test turned on the argument itself*), §5 (fail-closed; the "never" trap; construction over validation), §6 (lens as residue), §7 (recursion — this document as the first target). Closes DESIGN open thread #1.

## 1. The discipline (the lens schema, filled)

The open thread states the rule: *every claim is a consequence-chain back to an axiom — no orphan and no cycle.* DESIGN is reasoned **serially**: §1 fixes the axioms; each later section is a *consequence* of the ones before (or an independent peer), never a restatement. That serial structure is not prose convention — it is the §4 acyclicity test (intersubjective agreement holds only *across* time, so each claim must stay stable under it: a re-interrogable consequence-chain, never a cycle that could quietly redefine itself). The lens turns that test **on the argument itself**.

Filling the [expressibility-frontier](expressibility-frontier.md) §1.1 intake schema:

> ⟨ **preferred form** · **deviation witness** · **which §1-cost the gap displaces** · **where on the frontier it is enforceable** · **the drag-to-preferred fix** ⟩

- **Preferred form** — the argument is a **rooted DAG**: a finite set of claims; the only roots are the axioms A1–A3; every non-axiom claim names the premises it follows from; the *follows-from* relation is acyclic.
- **Deviation witness** — a claim that (a) reaches no axiom (**orphan** — a smuggled premise dressed as a conclusion), or (b) participates in a **cycle** (a claim that derives itself — the §4 violation), or (c) is treated as a root but is **not in {A1,A2,A3}** (a fourth axiom snuck in without being assumed).
- **Which §1-cost** — the **complexity** axis (time-to-change). An un-grounded claim is the document's own anemic modeling: a consumer (a later section, a lens, a reader, *the recursion in §7*) that builds on a claim with no derivation pays the de-fork later, exactly as code does when it builds on a nickname (§3). The argument is a substrate fact (§7); an orphan in it is the same defect class as an orphan carrier.
- **Frontier region** — §3 below. The **shape** (rooted-DAG-ness) is decidable → ①/②; the **content** (is each step a *valid* entailment?) is undecidable → ③.
- **Drag-to-preferred fix** — model the argument as `.dag` nodes with explicit `because:` edges, run the reachability + acyclicity walk over it, and fail closed on any orphan/cycle/fake-axiom.

This is **not a new lens family.** It is the [inert-layer-lens](inert-layer-lens.md) §8 rule — *every declared node in a graph must be reachable from a root, or rostered, or deleted* — applied to a **fourth substrate**: the **argument** of a reasoned document. The table from that doc gains one row:

| substrate | nodes | edges | roots | orphan = | cycle = |
| --- | --- | --- | --- | --- | --- |
| code | declared concepts | reference / `BindsTo` | run entries | unreachable carrier | (n/a) |
| docs | `docs/**/*.md` | markdown / `bind:` links | `ROADMAP` · `DESIGN` · runbook index | orphan plan doc | (n/a) |
| lenses | `v2.lens.*` | module imports | discovered witnesses | inert lens (#5433) | (n/a) |
| **argument** | **claims (A1–A3 + §1–§7)** | **`because:` (follows-from)** | **the axiom set {A1,A2,A3}** | **un-grounded claim** | **self-deriving claim** |

The argument substrate is the only one of the four where a **cycle** is also a violation — because it is the only one whose edge is *logical consequence* (§4: a cycle there is a claim quietly redefining itself), where the other three carry *dependency*, which cycles are merely undesirable, not incoherent.

## 2. The model — A1–A3 and the §1–§7 chain in `.dag`

Two separable facts, kept in their own layers (§3):

**(a) The framework — `std/`.** The *shape* "axiom · claim · derivation, forming a rooted acyclic argument" is a **universal framework** (classical logic / syllogism), so it homes in `std/` beside its peers, and it **reuses, does not fork**:

- `std/logic.dag` already declares `Classical = True | False` and notes it *"carries syllogistic structure"* — the truth-value authority.
- `std/graph.dag` already declares the directed-graph algebra with **cycle detection** (`graph_has_multi_node_scc`), DFS finish-order, and forward/reverse adjacency — the exact primitives the acyclicity + reachability checks need.
- `std/induction.dag` already grounds well-founded / acyclic structure (initial algebras, size-change).

The new carrier is thin — it *names* the argument over the existing graph algebra, it does not re-implement a graph:

```
module std.argument            // homes beside logic / graph / induction

type ClaimId = String          // a stable §-anchor handle, e.g. "A1", "s1.minimal-safe-efficient"

// An axiom is assumed, not derived: the closed root set. {A1, A2, A3} and nothing else.
type Axiom { id: ClaimId, statement: String, anchor: String }

// A derived claim names the premises it follows from. `because` non-empty ⟺ non-axiom.
type Claim { id: ClaimId, statement: String, anchor: String, because: List<ClaimId> }

// The whole argument: a finite claim set over one follows-from relation.
type Argument { axioms: List<Axiom>, claims: List<Claim> }
```

The three structural verdicts are **derived, not stored** (§3 single-authority: the edges are the fact, the verdicts are views) — each is a fold over `Argument` projected onto `std/graph.dag`'s `CallGraph` (`caller = claim`, `callee = each premise`):

- `argument_has_no_orphan(a)` — every `Claim` reaches some `Axiom` over `because` edges (reachability — the inert-lens BFS, re-expressed over claim nodes). An orphan is `claims ∖ reachable(axioms)` over the reverse relation, exactly `inert = universe ∖ reachable(roots)`.
- `argument_is_acyclic(a)` — `graph_has_multi_node_scc` over the `because` graph is `false` (the §4 test).
- `argument_axiom_set_is_closed(a)` — every node with empty `because` is in `{A1,A2,A3}`; no fourth root.

**(b) The instance — the DESIGN rows.** The actual A1–A3 + §1–§7 content is a fact *about gunbc's own DESIGN.md*, not a universal framework — so, exactly as `doc_reachability_project` homes the doc-graph instance outside `std/`, the **DESIGN argument rows** home in a gunbc-layer module (proposed `dsl/gunbc/design_argument.dag`) as data:

| node | kind | because |
| --- | --- | --- |
| `A1` there is a goal | axiom | — |
| `A2` time is the value | axiom | — |
| `A3` agreement is temporal | axiom | — |
| §1 solution is minimal/safe/efficient | claim | `A1`, `A2` |
| §1 grounding is intersubjective | claim | `A2`, `A3` |
| §1 reduce intersubjectivity to physics | claim | `A1`, `A2`, grounding-intersubjective |
| §2 minimize redundancy is the master move | claim | §1 minimal/safe/efficient |
| §3 single authority | claim | §2 |
| §4 closed grounded substrate makes §2–§3 decidable | claim | §2, §3 |
| §4 the structure is acyclic | claim | `A3`, §1 |
| §5 fail-closed (safety axis) | claim | §1 minimal/safe/efficient |
| §6 how to work | claim | §1, §2, §3, §4, §5 |
| §7 self-hosting / recursion | claim | §1–§6 |

These rows are the **map** step of §2's `decompress → map → reduce`: DESIGN's own prose already *names* each consequence ("From A1 and A2 —", "From A2 and A3 —", "what keeps §2 from being undone", "what makes §2–§3 decidable") — the model transcribes those stated `because:` links, it does not invent them. That the prose already carries the edges is *why* this is decidable shape, not authored opinion.

## 3. The partition — ① wall / ② lens-residue / ③ undecidable

The deliverable, per [expressibility-frontier](expressibility-frontier.md) §4 (*partition before gating*). The dividing line is the same one the frontier names: **decidability of membership**, and the sharper cut here is **shape vs content** — the *shape* of the argument is graph-decidable; the *truth of an inference step* is not.

### ① Wall — the argument is a rooted DAG (decidable; unwritable once modeled as nodes)

Three properties, all decidable graph facts, all fail-closed:

- **No orphan** — reachability to the axiom set. `argument_has_no_orphan`. The inert-lens reachability primitive, fourth substrate.
- **No cycle** — `argument_is_acyclic` via `graph_has_multi_node_scc`. The §4 acyclicity test, turned on the argument.
- **Closed axiom set** — `argument_axiom_set_is_closed`: a root that is not A1/A2/A3 is a smuggled premise.

**Why ① and not ②:** the bad state becomes *unwritable* when the argument is a substrate `Node`/`Edge` graph, because the cycle check is then **the resolver's own acyclicity check** (§4: the substrate forbids cyclic values; a cyclic `because` relation is rejected by the same machinery that rejects a cyclic program), and the orphan check is the inert-lens BFS over the same node set. A claim with no derivation *cannot be a non-axiom Claim* — `because: []` is, by construction, the axiom predicate, so writing an un-grounded claim is writing a fourth axiom, which `argument_axiom_set_is_closed` walls. The wall stacks the way the frontier's anemic-modeling row stacks (`Measure` phantom + operation signature): the *type* (`Axiom` vs `Claim`) walls one half, the *closed-set check* walls the other.

**The honest staging (the §5/§6 caveat, identical to inert-lens §5):** while the argument still lives in **DESIGN.md prose** and is *transcribed* into rows by hand, the orphan/cycle checks run over the transcription, not over a parse of the English — so until a `bind:`-style anchor ties each row to its §-anchor and a drift-gate proves the row set matches the prose (the [invert-hand-maintained](invert-hand-maintained.md) pattern), this is a ② **observing** lens over host-/hand-fed rows, promoted to a ① wall when the rows are *derived from* the doc rather than mirrored beside it. Same ratchet-during-migration → wall-when-grounded shape as #5433 / the doc-graph wall (#5484).

### ② Lens-residue — the prose↔model binding seam (decidable, not yet fenceable)

The seam you cannot yet fence (the frontier's "grounding seam" — host `Int` → `ByteSize`, here English sentence → `Claim` node):

- **Binding completeness** — every §-anchor in DESIGN that asserts a claim has a `Claim` row, and every `Claim` row points at a real §-anchor. Decidable *as a structural cross-check* (anchor exists / row exists) — a pure reader over the doc node set + the row set, the doc-reachability shape. But it cannot decide whether the row's `statement` **faithfully restates** the prose — that crosses into ③.
- **Claim single-authority** — two rows asserting the *same* claim (a claim nicknamed twice — §3 on the argument) is decidable iff claim identity is structural; while identity is by §-anchor string it is a ② dedup check, not a ① wall.

Presentability (frontier §2): the fix here is **determined, not searched** — an orphan's repair is "add the missing `because` edge the prose already implies" or "delete the claim" — so the lens can *present* the candidate edge (the §-anchor the prose cites), the way the anemic lens presents the `op_T`. It never *picks*; surfacing the missing premise is a §5 decision for the author.

### ③ Inexpressible — the soundness of each step (undecidable; permanent review)

The frontier's ③ tail — needs domain knowledge or hits the undecidability of validity. **The lens checks that the argument is *shaped* like a syllogism; it can never check that each syllogism is *valid*.**

- **Inference soundness** — "does `A1 ∧ A2` *actually entail* 'minimal/safe/efficient'?" is the validity of the step's *content*. First-order validity is at best semi-decidable (RE, not decidable); natural-language entailment is not even formalizable without a domain model. **Permanent review.**
- **Argument completeness** — "are these *all* the consequences of A1–A3, or is a claim missing?" is the §6 completeness-critic question over the argument; undecidable (you cannot enumerate all true consequences — Rice-shaped). **Permanent review.**
- **Axiom independence / minimality** — "is A3 derivable from A1 ∧ A2, or genuinely assumed?" is a derivability question — same undecidable validity tail. **Permanent review.**

**The trap to refuse (frontier §4, the "never" trap):** do **not** sell "every claim is *sound*" as a wall. That is ③ priced as ① — a ratchet wearing a wall's badge, whose success criterion quantifies "valid" over an undecidable set. The wall's success criterion is bounded and decidable: *the argument is a rooted acyclic graph over a closed axiom set.* Soundness stays honest, permanent review — and that is not a failure of the lens; it is the lens telling the truth about its own frontier (§5: "never" is the trap; check decidability before claiming a wall).

## 4. DESIGN as the first target (the §7 recursion)

The open thread names the first target explicitly: *"with this document as the first target."* So the first `Argument` instance the lens runs over is DESIGN.md's own §1–§7 (the §2 table above). This is the §7 recursion made executable: every principle governs the document that states them, and the **syllogism lens is the document checking its own serial structure is a real consequence-chain and not a circle.** The floor witness (the discriminating green/red, §5 "green by execution") is:

- **GREEN now:** `argument_has_no_orphan(design)` ∧ `argument_is_acyclic(design)` ∧ `argument_axiom_set_is_closed(design)` over the transcribed DESIGN rows.
- **RED on revert (the discriminator):** delete a `because` edge from any §-row → that claim becomes an orphan → `argument_has_no_orphan` goes RED. Add a `because` edge that points "forward" (a later § derived from an earlier claim that in turn cites it) → a cycle → `argument_is_acyclic` RED. Add a fourth root → `argument_axiom_set_is_closed` RED. **Non-vacuity floor** (inert-lens §, doc-reachability §5): an empty `Argument` is itself RED (a zero-claim universe fail-opens the orphan check).

The witness homes at `dsl/test/claim/design_argument_witness_test.dag`, floor-discovered by the `*_test.dag` + `test fn` marker convention (CLAUDE.md "Building & checks") — naming it enrolls it; no hand-wiring.

## 5. Reuse map (do not fork — §3)

| need | reuse | where |
| --- | --- | --- |
| acyclicity / cycle detection | `graph_has_multi_node_scc` | `std/graph.dag` |
| forward/reverse adjacency, DFS | `forward_adjacency` · `reverse_adjacency` · `dfs_finish_order` | `std/graph.dag` |
| reachability `universe ∖ reachable(roots)` | the inert-lens / doc-graph BFS shape | `inert_lens_modules` (`cli_run.rs`) · `doc_reachability_project.rs` |
| truth-value / syllogistic structure | `Classical = True \| False` | `std/logic.dag` |
| well-founded / acyclic grounding | initial-algebra + size-change | `std/induction.dag` |
| fail-closed lens verdict carrier | `LensVerdict` (Holds/Violation/NotApplicable/Unrealized) | `std/lens_verdict.dag` |
| doc-instance-beside-std pattern | doc rows home outside `std/`, framework inside | `doc_reachability_project` |
| witness floor-discovery | `*_test.dag` + `test fn` marker enrol | CLAUDE.md "Building & checks" |

The single biggest anti-fork: **the reachability rule is one concept across four substrates** (§2-horizontal). This lens must be the inert-lens §8 rule's *fourth row*, not a second reachability authority. If the implementation re-implements a BFS instead of re-expressing the existing one over claim nodes, that is the meta-fork the frontier §1.1 warns about (two lenses on the same ⟨reachability⟩ axis).

## 6. Frontier placement, wiring, dissolution

- **Frontier placement:** the **rooted-DAG shape is a ① wall** (decidable, construction-enforceable once claims are substrate nodes); the **prose↔model binding is ② residue** (a determined-fix reader at the grounding seam); **inference soundness / completeness / axiom-independence are ③, permanent review.**
- **Wiring (when built, not now):** `std/argument.dag` (framework) + `dsl/gunbc/design_argument.dag` (the DESIGN rows) + `dsl/test/claim/design_argument_witness_test.dag` (the three fail-closed `test fn`s + non-vacuity floor). No `cli_run.rs` edit if the rows are pure `.dag` (the orphan/cycle checks are pure folds over `graph.dag` — unlike the doc-graph's orphan half, the argument's universe is the *declared row set*, not a filesystem walk, so **no host bridge is needed**: this is strictly cheaper than the doc-graph wall, which needed a dir-walk census).
- **Dissolution:** the lens never dissolves (a serial argument is a standing property). Its **staging dissolves**: the ② "hand-transcribed rows" arc empties when the rows are *derived from* DESIGN's §-anchors via the [invert-hand-maintained](invert-hand-maintained.md) drift-gate (emit-from-authority + prove the rows match the doc), at which point the binding seam closes and the shape verdicts flip from ② observing → ① wall. The ROADMAP item stays `[ ]` until the witness *runs executably over this doc* (the open thread's own bar — "stays `[ ]` until it runs executably over this doc").

## 7. The vertical slice (the smallest executable proof, for the nod)

The minimal slice that proves the wall on **real DESIGN claims** — sized to §1-minimal, not the whole chain:

1. **`std/argument.dag`** — the three carriers (`Axiom`, `Claim`, `Argument`) + three folds (`argument_has_no_orphan`, `argument_is_acyclic`, `argument_axiom_set_is_closed`), each projecting onto `std/graph.dag`. ~no new graph code — the folds adapt `Argument → CallGraph` and call the existing `graph_has_multi_node_scc` / a reverse-reachability BFS.
2. **`dsl/gunbc/design_argument.dag`** — **only DESIGN §1**: the 3 axioms + the ~5 §1 consequence rows ("minimal/safe/efficient" ← A1,A2 · "grounding intersubjective" ← A2,A3 · "reduce to physics" ← A1,A2, grounding). The smallest *non-vacuous* argument — enough to exercise orphan, cycle, and closed-axiom on genuine claims, not synthetic ones.
3. **`dsl/test/claim/design_argument_witness_test.dag`** — four `test fn`s: the three shape verdicts GREEN over §1, plus the non-vacuity floor (`claim_count > 0`). Floor-discovered by marker.

**The discriminating receipt (§5 green-by-execution, not grep):**

- **GREEN:** all three verdicts hold over the real §1 rows.
- **RED on orphan:** delete the `because: [A2, A3]` edge on "grounding intersubjective" → it reaches no axiom → `argument_has_no_orphan` RED.
- **RED on cycle:** add `because: [reduce-to-physics]` to A1 (make an axiom derive from its own consequence) → `graph_has_multi_node_scc` true → `argument_is_acyclic` RED.
- **RED on smuggled axiom:** add a `because: []` claim that is not A1/A2/A3 → `argument_axiom_set_is_closed` RED.

That is the whole nod-able artifact: **three carriers, one §1 instance, one witness file** — pure `.dag`, no host bridge, reusing `graph.dag`. It proves the wall holds on real claims and that the discriminators go red, without committing to the full §1–§7 transcription or the ② derive-from-prose arc (§6). If the operator nods, §2–§7 rows and the binding drift-gate land incrementally on top.

## 8. Open (for the operator nod)

- **The row authority — transcribed vs derived.** v1 ships the rows hand-transcribed beside DESIGN (cheapest first target, ② observing). Is the operator's bar that v1 already *derive* the rows from §-anchors (a parse of DESIGN's "From X and Y —" prose), closing the ② seam in the first landing? That is the [invert-hand-maintained](invert-hand-maintained.md) coupling and a larger first cut.
- **Claim granularity.** The §2 table is one node per stated consequence. Finer granularity (every sentence a claim) sharpens the orphan check but widens the transcription surface; coarser (one node per §) is cheaper but lets an intra-§ orphan hide. Recommend the *stated-consequence* granularity above (DESIGN's own "From … —" boundaries) — the prose's own joints, neither finer nor coarser.
- **Independent peers.** DESIGN §1 allows a claim to be *"an independent peer"* rather than a consequence (e.g. §3's strict-layer corollary). A peer has no `because` into the chain yet is not an orphan — so the model needs a third node kind (`Peer`, an axiom-like leaf scoped to a section) or peers must root into the axiom that *licenses* them. This is the one modeling decision the partition does not yet settle; flagged for the nod.
- **The recursion's own first target order.** Should the *first* witness be DESIGN §1 alone (axioms + their immediate consequences — the smallest non-vacuous argument) and §2–§7 land incrementally, or the whole §1–§7 chain at once? Incremental is the §1-minimal first cut and matches "DESIGN as the *first* target" literally.

---

*Status: SCOPE-ONLY, awaiting operator nod. No carrier, lens, or witness is built by this PR — it adds this design doc and its ROADMAP inbound link (honoring the doc-reachability wall #5484: a new `docs/plans/X.md` adds its inbound link in the same PR).*

## Dissolution trigger (DESIGN §6)

Delete this doc when the axiom + syllogism lens is built and runs executably over DESIGN.md's own §1–§7 argument: std/argument.dag (the Axiom/Claim/Argument carriers plus the three fail-closed folds), dsl/gunbc/design_argument.dag (the transcribed DESIGN rows), and a discovered design_argument_witness_test.dag are green-by-execution with the orphan/cycle/smuggled-axiom discriminators going RED — at which point this scope-only design is superseded by the running wall and DESIGN open thread #1 is closed.
