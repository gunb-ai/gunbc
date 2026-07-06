# `algebra` grounding-unification: one FreeMonoid authority across the two std trees

Status: **DRAFT for operator sign-off.** Owner: calm-lark-461. Design-only — no load-bearing edit lands under this doc until the operator signs the shape.
Executes the prerequisite the de-fork audit names for category (b): *"the grounded dag authority for each has to be designed before any fan-out can repoint to it"* (`dag/gunbc/plans/dag_v2_defork_audit.dag`, census dissolution note). This is that per-concept authority design for **`algebra`**, the anchor the operator ruled on 2026-06-22 (§3b of the audit). It does **not** re-open that ruling; it makes it implementable and surfaces the load-bearing decisions the ruling left open.

Reasoned serially per the DESIGN preamble: §1 fixes the problem, each later section is a consequence.

## 1. The problem (measured, by execution)

Two files define `algebra` on the two std trees:
- `dag/std/algebra.dag` — module `std.algebra`, **332 lines, 32 importers** (the `dag/` tree + the `src/v1` seed).
- `src/v2/std/algebra.dag` — module `v2.std.algebra`, **816 lines, 290 importers** (the v2 compiler).

They share **unqualified type names** — the algebraic-structure records (`Magma`/`Semigroup`/`Monoid`/`Group`/…) and, most sharply, the `FreeMonoid<T> = Empty | Cons { head, tail }` coproduct. When a single import closure pulls in *both* trees, those unqualified names bind ambiguously and the resolver silently **drops the coproduct's variant bindings** — the `dag/std/algebra.dag:115` `free_monoid_coproduct_authority` note records the live reproduction (2026-07-05: *"undefined variable: Empty"* in a closure containing both trees). The audit measures this as the **LIVE fail-open** pair `{algebra (75 floor entries), nat (4)}`: it is benign today only because it shadows record-with-record rather than dropping a variant, and a flag-ANY guard wall reds it the moment the fork lands.

This is §5 fail-open (a wrong resolution passes silently) sitting on a §3 single-authority violation (one concept, two homes). It is also the concrete blocker under the QualifiedName-key parking I closed in `type-env-single-authority-design.md` §5.1: `QualifiedName = FreeMonoid<Symbol>`, so it cannot get a clean `std.qualified_name` home — reachable from the v1 SymbolIndex — until `FreeMonoid` has a single authority. **Unforking `algebra` is the gate that unparks that item.**

## 2. The fork, by execution (what actually collides)

The two files are **not** byte-identical — only the `FreeMonoid` coproduct is. They serve disjoint populations, and only a subset of their surface overlaps:

| Surface | `dag/std/algebra.dag` (authority tree) | `src/v2/std/algebra.dag` (compiler tree) | Collides? |
|---|---|---|---|
| Structure records (`Magma`…`BooleanAlgebra`) | present (record-of-methods) | present (record-of-methods) | **YES** — shared unqualified names |
| `FreeMonoid` / `Empty` / `Cons` | coproduct (line 111) | coproduct (line 73) | **YES** — the variant-drop hazard |
| Template/profile machinery (`AlgebraProfile`, `AlgebraFieldTemplate`, `kernel_algebra_profile`, `free_monoid_scalar_templates`, `algebra_templates_for_profile`) | **only here** (complexity-analyzer inhabitance projection) | absent | no |
| Operational fold ops (`fold_list`, `fold_list_right`, `list_head`, `list_tail`, `skip`, `freemonoid_empty`) | **absent** | **only here** | no |
| Node-encoding machinery (`algebra_inhabitance_node`, `*_type_node`, `*_node`) | absent | **only here** (encodes algebra AS `Node` for `04_infer`) | no |

So the disjoint machinery (templates vs operational-fold vs Node-encoding) is safe; the **collision is exactly the structure records + `FreeMonoid` coproduct** — the concepts, not the derived surface. Per the operator ruling (§3b): the coproduct is the structural authority; the `dag` record-of-methods surface is *derived from inhabitance* (DESIGN §4 — ops from inhabitance, no per-type ops), a projection, **not** a second definition.

## 3. The target authority (consequence of §1–§2 + the ruling)

One home per concept, chosen by the layer DAG (`std ← extdeps ← compiler ← workflow`):

1. **`FreeMonoid` coproduct + the algebraic-structure records live once, in `std.algebra`** (the `dag/std` tree — the authority both trees can import). This is the concept layer; it belongs in std, and the dag tree is std's home.
2. **The operational fold ops** (`fold_list`, `list_head`, `list_tail`, `fold_list_right`, `skip`, `freemonoid_empty`) are generic, layer-appropriate std — they **move from `v2.std.algebra` into `std.algebra`** (or a `std.list` sub-module split off it — see §7 Q1). They are the "ops from inhabitance" surface of the coproduct; they belong beside it.
3. **The Node-encoding machinery** (`algebra_inhabitance_node`, `*_type_node`, `*_node`) is **compiler-layer, not std** — it encodes std algebra types *as `Node`s* for the `04_infer` inhabitance check. It stays in the v2 compiler tree (relayered out of `v2.std.algebra` into a compiler module), importing the single `std.algebra` authority. It is realization/dispatch, which §3 keeps peripheral, never in the concept.
4. **`v2.std.algebra`'s duplicate structure-record + `FreeMonoid` decls are deleted**; its 290 importers repoint to `std.algebra`.
5. **The aliases the ruling authorized** (`String = FreeMonoid<Char>`, `List<T> = FreeMonoid<T>`, `QualifiedName = FreeMonoid<Symbol>`), grounding to native `Vec` in the seed and the faithful coproduct in pure-v2, are introduced **on the single authority** — but their blast radius is huge (String/List touch nearly everything), so §7 Q4 asks whether they land here or in a follow-on. `QualifiedName` is the small, contained alias that this doc's motivating item needs; it can land first.

The template/profile machinery (dag-only) and the operational-fold machinery (moving to dag) then sit in one file/module cluster with no cross-tree twin — the shadow is structurally impossible because there is only one `FreeMonoid`.

## 4. What moves where (the mechanical plan, once §3 is signed)

- **Into `std.algebra`** (dag tree): the 6 operational fold ops, verbatim, from `v2.std.algebra`. Reconcile any structure-record method-signature drift between the two record forms (the audit flags algebra as a *divergent grounding*, so the records may not be identical — a shared-body diff gates this; the coproduct is the only proven-identical part).
- **Into a new v2 compiler module** (e.g. `v2.compiler.algebra_encoding` or beside `04_infer`): the Node-encoding fns, importing `std.algebra`.
- **Delete** from `src/v2/std/algebra.dag`: the structure-record + `FreeMonoid`/`Empty`/`Cons` decls now homed in `std.algebra`.
- **Repoint** the 290 `import v2.std.algebra { … }` sites to `import std.algebra { … }` (+ the encoding-module import where they used Node-encoding fns). Mechanical, but large — a single fold over the import sites, staged atomically (§5 hazard).
- **Then** `QualifiedName` gets its `std.qualified_name` home (FreeMonoid<Symbol> now resolvable from both trees), unparking `type-env-single-authority-design.md` §5.1.

## 5. Sequencing & gates (do not collide with live lanes)

- **Gated on the keystone residue.** The audit's Root-B keystone (generic-alias instantiation) partially landed (#5552), but *raw variant-matching a value bound from a recursive `Cons.tail` field* still fails in the `04_infer` fixpoint (*"variant not found in type FreeMonoid"*). Consumers avoid it via the `list_head`/algebra idiom, so the move is safe **if** the merged authority keeps that idiom — but the definition-unification cannot rely on raw `Cons.tail` matching until the residue clears. **This residue is load-bearing `03_resolve`/`04_infer` work; it is a prerequisite, not part of this lane** (§7 Q3).
- **Root A is jolly-cat's.** The `src/v1/05_emit_rust.dag` grounding emit-seam (host `Vec`) is a separate lane; **this design does not touch `05_emit_rust.dag`**. The alias grounding (§3.5) *consumes* Root A's seam but does not author it.
- **Atomicity hazard.** The dashboard auto-committer can snapshot a multi-file rename mid-edit → an internally-inconsistent commit → phantom "not found in scope" red on a frozen merge-sha CI run. **Every file of the collapse stages together in one push.**
- **Land-green ordering.** The flag-ANY co-occurrence wall reds the moment the fork is *present*, so the collapse lands green only when the delete + repoint are in the same push. `nat` (escalated, smart-ant-466) stays BLOCKED + LAST per the audit; `algebra` is the anchor that goes first.

## 6. Validation (§5 prove-by-execution, not typecheck)

- **The 75 co-occurrence floor entries go green** with the flag-ANY wall armed (the wall that reds on a present fork is the red control; a green run with it armed proves the fork is gone).
- **Byte-identical emit fixpoint** (`bootstrap_fixed_point`) — the collapse is behavior-preserving; not one emitted byte changes.
- **The shadow repro** (the 2026-07-05 *"undefined variable: Empty"* closure) is added as a witness that goes green only post-collapse and red on the forked tree.
- **`QualifiedName` promotion executes mechanically** afterward — the discriminating downstream signal that the authority is genuinely single.
- `cargo test --workspace` green; the algebra/FreeMonoid witnesses (`generic_alias_coproduct_instantiation_test.dag`) green.

## 7. Open decisions for the operator (the load-bearing calls I cannot make)

These are the model-before-implement forks that need your sign-off before any edit:

- **Q1 — operational-fold home: `std.algebra` or a split `std.list`?** The 6 fold ops are generic list operations. Folding them into `std.algebra` keeps one file; splitting a `std.list` module is cleaner layering but adds a module. Which?
- **Q2 — structure-record reconciliation.** The audit classes `algebra` as a *divergent grounding*, so the `Magma`…`BooleanAlgebra` records may differ in method signatures between the two files. Is reconciling them in-scope here, or is only `FreeMonoid` + the operational ops unforked now, with the structure records deferred to the numeric-tower grounding they entangle with?
- **Q3 — the `04_infer` recursive-`Cons.tail` residue.** Prerequisite dispatched separately (load-bearing infer work), or pulled into this lane? My read: separate — it pre-dates and is load-bearing, so it should be its own escalation.
- **Q4 — aliases: land `String`/`List` here, or only `QualifiedName`?** `String`/`List` aliasing has repo-wide blast radius and rides Root A's emit-seam; `QualifiedName` is contained. My recommendation: land only `QualifiedName` here (it unparks the motivating item), defer `String`/`List` to the Root-A-coordinated follow-on.
- **Q5 — module naming for the relayered Node-encoding fns.** `v2.compiler.algebra_encoding`, or fold into an existing `04_infer` support module?

## 8. Dissolution

This doc is deleted when `algebra` denotes one concept with one authority: `src/v2/std/algebra.dag` no longer defines `FreeMonoid`/`Empty`/`Cons` or the structure records, the flag-ANY co-occurrence wall lands green over the corpus, and `QualifiedName` has promoted to `std.qualified_name`. At that point the audit's category-(b) anchor is resolved and the carriers tell the story.
