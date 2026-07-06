# `algebra` grounding-unification: one FreeMonoid authority across the two std trees

Status: **Shape SIGNED by operator 2026-07-06** (Q1–Q5 answered — see §7). Owner: calm-lark-461. Implementation may proceed on the signed shape; the load-bearing core (`std.algebra` collapse + the `04_infer` residue) stays model-first and verified by-execution.
Executes the prerequisite the de-fork audit names for category (b): *"the grounded dag authority for each has to be designed before any fan-out can repoint to it"* (`dag/gunbc/plans/dag_v2_defork_audit.dag`, census dissolution note). This is that per-concept authority design for **`algebra`**, the anchor the operator ruled on 2026-06-22 (§3b of the audit). It does **not** re-open that ruling; it makes it implementable.

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

One home per concept, chosen by the layer DAG (`std ← extdeps ← compiler ← workflow`). **This section is reconciled with the signed decisions in §7/§7.1** — where the pre-sign-off draft was broader, it now states the resolved scope:

1. **`FreeMonoid` coproduct lives once, in `std.algebra`** (the `dag/std` tree — the authority both trees can import; already present at `dag/std/algebra.dag:111`). The algebraic-structure records (`Magma`…`BooleanAlgebra`) **stay forked for now** — the shared-body diff (§7.1 step 1) showed 12 of 14 differ, a genuine divergent grounding, so per signed Q2 they **defer** to a separate operator-held follow-on. This lane relocates `FreeMonoid`/`Empty`/`Cons`, not the records.
2. **The operational FreeMonoid ops move from `v2.std.algebra` into `std.algebra`** — the full enumerated set (§7.1 step 1): `fold_list`, `fold_list_right`, `fold_non_empty`, `list_head`, `list_tail`, `skip`, `freemonoid_empty`, `for_all`, `any`, `filter`, `contains`, `count_where`, `is_empty` (+ helper types `ListTailResult`/`ListHeadResult`) and the `PointwisePower<T>` type. They are the "ops from inhabitance" surface of the coproduct and belong beside it (signed Q1 — kept in `std.algebra`, not split to `std.list`).
3. **The Node-encoding fns** (`algebra_inhabitance_node`, `*_type_node`, `*_node`) **stay in `std.algebra`** per signed Q5 — the surface is tiny (`algebra_inhabitance_node` 1 consumer, the `*_type_node`/`*_node` family 0), they build only on `std.node` + the coproduct (no upward dependency), and they are a projection of the algebra types into `Node` — std-appropriate. Genuinely dead zero-consumer fns are **pruned** as a §2 win (not relayered to a new compiler module).
4. **`v2.std.algebra`'s duplicate `FreeMonoid`/`Empty`/`Cons` decls + the moved ops are deleted** (the records are **not** deleted — they stay, deferred per §3.1); its 290 importers repoint to `std.algebra` for the moved symbols.
5. **Only the `QualifiedName = FreeMonoid<Symbol>` alias lands here** (signed Q4). The `String = FreeMonoid<Char>` / `List<T> = FreeMonoid<T>` aliases have repo-wide blast radius and ride Root A's emit-seam; they are a separate follow-on (Root A is currently unowned — §5).

Because `FreeMonoid`/`Empty`/`Cons` then exist in exactly one place, the **dangerous coproduct-variant-drop shadow is structurally impossible**. The record collision remains (benign record-with-record; §7.1 step 1) until the deferred record grounding.

## 4. What moves where (the mechanical plan on the signed shape)

- **Into `std.algebra`** (dag tree), verbatim from `v2.std.algebra`: the **~15 operational ops + helper types + `PointwisePower`** enumerated in §3.2/§7.1 step 1. Keep the low-consumer **Node-encoding fns in `std.algebra`** (signed Q5), pruning genuinely dead zero-consumer ones. **Records are NOT moved** — they stay forked, deferred per signed Q2 (§7.1 step 1 diff: 12 of 14 differ).
- **Delete** from `src/v2/std/algebra.dag`: the `FreeMonoid`/`Empty`/`Cons` decls + the moved ops now homed in `std.algebra`. (The structure records are **left in place** — deferred.)
- **Repoint** the 290 `import v2.std.algebra { … }` sites to `import std.algebra { … }` for the moved symbols. Mechanical, but large — a single fold over the import sites, staged atomically (§5 hazard).
- **Then** `QualifiedName` gets its `std.qualified_name` home (`FreeMonoid<Symbol>` now resolvable from both trees), unparking `type-env-single-authority-design.md` §5.1.

## 5. Sequencing & gates (do not collide with live lanes)

- **The keystone residue is IN-SCOPE here** (signed Q3). The audit's Root-B keystone (generic-alias instantiation) partially landed (#5552); the residue was *raw variant-matching a value bound from a recursive `Cons.tail` field* failing in the `04_infer` fixpoint (*"variant not found in type FreeMonoid"*). Recon (§7.1 step 2) finds no such pattern live in `04_infer` and the error string absent, so it **appears already resolved** — to be confirmed by executing the keystone witness before the collapse relies on raw `Cons.tail` matching. Load-bearing `03_resolve`/`04_infer` work: model-first, escalate if a real fixpoint gap resurfaces.
- **Root A is a separate lane, currently UNOWNED.** The `src/v1/05_emit_rust.dag` grounding emit-seam (host `Vec`) is not authored here; **this design does not touch `05_emit_rust.dag`**. Its prior owner (jolly-cat) is gone from the tree (§7 Q4). This lane is **not blocked on Root A** — it lands `QualifiedName` only, which needs no host-`Vec` alias grounding; the `String`/`List` alias follow-on is what depends on Root A being re-dispatched.
- **Atomicity hazard.** The dashboard auto-committer can snapshot a multi-file rename mid-edit → an internally-inconsistent commit → phantom "not found in scope" red on a frozen merge-sha CI run. **Every file of the collapse (FreeMonoid delete + moved ops + 290 repoints) stages together in one push.**
- **Land-green ordering.** The `FreeMonoid` collapse (delete + moved ops + repoint) lands green only when all its files are in the same push. Because the **records stay forked** here (deferred, §3.1), the flag-ANY co-occurrence wall for the algebra pair does **not** go green on this lane — see §6. `nat` (escalated, smart-ant-466) stays BLOCKED + LAST per the audit; `algebra`'s FreeMonoid is the anchor that goes first.

## 6. Validation (§5 prove-by-execution, not typecheck)

**This lane's acceptance gates (all by-execution):**
- **Byte-identical emit fixpoint** (`bootstrap_fixed_point`) — the collapse is behavior-preserving; not one emitted byte changes.
- **The `FreeMonoid` shadow repro** (the 2026-07-05 *"undefined variable: Empty"* closure over both trees) is added as a witness that goes green only post-collapse and red on the forked `FreeMonoid` — the discriminating red control for *this* lane.
- **`QualifiedName` promotion executes mechanically** afterward — the downstream signal that `FreeMonoid` is genuinely single-authority.
- `cargo test --workspace` green; the FreeMonoid keystone witness (`generic_alias_coproduct_instantiation_test.dag`) green.

**Explicitly NOT this lane's gate (a later milestone):**
- **The flag-ANY co-occurrence wall over the ~75 algebra floor entries.** That wall reds on *any* shared unqualified name, and the structure records stay forked here (deferred per Q2), so it greens for the algebra pair **only after the record grounding also lands**. A worker on this lane must not treat flag-ANY-green as required now — this lane's fork-gone proof is the `FreeMonoid` shadow repro above, not the full-pair wall.

## 7. Resolved decisions (operator, 2026-07-06)

- **Q1 — operational-fold home → `std.algebra`** (not a `std.list` split). Decided by consumers + naturalness (operator: "look at consumers, which is more natural"): the 6 ops (`fold_list` 100 consumers, `skip` 84, `list_head`/`list_tail`/`fold_list_right` ~15, `freemonoid_empty` 5) are consumed pervasively across the v2 tree and operate on `FreeMonoid` specifically — they are the free monoid's "ops from inhabitance," so they belong beside the coproduct. `dag/std/list.dag` already exists but is a near-empty stub over a *different* type (`std.types.List`, not `FreeMonoid`); routing the FreeMonoid fold ops there would fuse the List-vs-FreeMonoid representation question, which is the deferred Q4 alias work. Keep them in `std.algebra`.
- **Q2 — reconcile the structure records if reasonable; FreeMonoid unfork is the priority** (operator). So: the `FreeMonoid` coproduct + `Empty`/`Cons` + the fold ops unfork **first and unconditionally**; the `Magma`…`BooleanAlgebra` record reconciliation rides along **only if** a shared-body diff shows the two record forms are already equal (or trivially so). If they diverge non-trivially (a real *divergent grounding*), the records stay as a follow-on and this lane collapses only `FreeMonoid` + ops — the priority is met without blocking on the record grounding.
- **Q3 — pull the `04_infer` recursive-`Cons.tail` residue into this lane** (operator). It is load-bearing `03_resolve`/`04_infer` work and stays model-first, but it is in-scope here, not a separate escalation. (Reversed my earlier "separate" read on the operator's call.)
- **Q4 — land only `QualifiedName` here; `String`/`List` aliases are a separate follow-on** (operator will dispatch that after Root A). ⚠️ **Root-A ownership gap:** the audit assigns Root A (the `05_emit_rust.dag` grounding emit-seam) to **jolly-cat**, who is **no longer in the session tree** (operator observed 2026-07-06). The `String`/`List` alias follow-on depends on Root A landing, so Root A needs a fresh owner before that follow-on can start. This lane does **not** need Root A (QualifiedName-only, no host-`Vec` alias grounding required), so it is not blocked — but the gap is flagged for the operator to re-dispatch.
- **Q5 — prefer a std/extdeps home for the Node-encoding fns over a new compiler module** (operator: "roll into std or extdeps if you can"). Feasible because the surface is tiny: `algebra_inhabitance_node` has **1** live consumer (`src/v2/lens/leaf_model_verification.dag:613`) and the `*_type_node`/`*_node` family has **0**. So rather than mint `v2.compiler.algebra_encoding`, keep the encoding fns in `std.algebra` (they build only on `std.node` + the coproduct, no upward dependency) — they are a projection of the algebra types into `Node`, std-appropriate — and prune any genuinely dead `*_node`/`*_type_node` with zero consumers as part of the collapse (a §2 win rather than a relayer).

## 7.1 Implementation sequencing (on the signed shape)

Dependency-ordered; the collapse is **one atomic push** per the §5 auto-committer hazard (stage every file together).

1. **Shared-body diff of the structure records** (`Magma`…`BooleanAlgebra`) — **DONE (2026-07-06).** Only `Magma` + `Lattice` are byte-identical; the other 12 records **differ** between the trees (a real divergent grounding). So Q2's branch is decided: **records DEFER** — this lane collapses **`FreeMonoid` + `Empty`/`Cons` + the operational ops only.** The remaining record shadow is the *benign* record-with-record kind (the audit's own distinction), so leaving it does not reintroduce the dangerous coproduct-variant-drop; the record grounding is a separate operator-held follow-on (numeric-tower-entangled). Consequence: the flag-ANY co-occurrence wall (§6) lands green for the algebra pair **only after** the record grounding too — this lane removes the *dangerous* FreeMonoid shadow and unparks QualifiedName, but the wall activation is a later milestone, not this PR's gate.
   - **Operational surface to move (enumerated by execution):** ~15 FreeMonoid ops — `fold_list`, `fold_list_right`, `fold_non_empty`, `list_head`, `list_tail`, `skip`, `freemonoid_empty`, `for_all`, `any`, `filter`, `contains`, `count_where`, `is_empty` (+ their helper types `ListTailResult`/`ListHeadResult`) — plus the `PointwisePower<T>` type. All operate on `FreeMonoid`; all move to `std.algebra`.
2. **`04_infer` recursive-`Cons.tail` residue** (Q3, load-bearing) — `src/v2/compiler/04_infer.dag` contains **no raw `Cons`/`.tail` match pattern** and the *"variant not found"* string is absent, so the residue appears already resolved; the keystone witness `generic_alias_coproduct_instantiation_test.dag` is present. **Confirm by execution** (run the keystone test) before relying on it; if it is genuinely resolved, step 2 is a no-op and the collapse is mechanical-plus-verification. Escalate only if a real fixpoint gap resurfaces.
3. **`std.algebra` authority merge** — move the ~15 enumerated ops + helper types + `PointwisePower` into `dag/std/algebra.dag`; keep the low-consumer Node-encoding fns there (Q5), prune dead ones. **Records are NOT moved** (deferred, Q2).
4. **Delete** `src/v2/std/algebra.dag`'s `FreeMonoid`/`Empty`/`Cons` + the moved ops decls (records stay); **repoint** the 290 `import v2.std.algebra` sites to `import std.algebra` for the moved symbols — a single fold, staged in the same push as the delete.
5. **`QualifiedName` → `std.qualified_name`** (Q4-contained), unparking `type-env §5.1`.
6. **Validation** per §6 — **this lane's gates**: byte-identical fixpoint, the `FreeMonoid` shadow-repro witness, mechanical QualifiedName promotion, `cargo test` + keystone witness green. The flag-ANY wall over the full algebra pair is **not** a gate here (records deferred; §6).

Steps 3–4 are the atomic collapse; the 290-importer repoint is mechanical and parallelizable (candidate for dispatched sub-lanes) while the load-bearing core (steps 2–3) stays with this session.

## 8. Dissolution

This doc is deleted when `algebra` denotes one concept with one authority: `src/v2/std/algebra.dag` no longer defines `FreeMonoid`/`Empty`/`Cons` or the structure records, the flag-ANY co-occurrence wall lands green over the corpus, and `QualifiedName` has promoted to `std.qualified_name`. At that point the audit's category-(b) anchor is resolved and the carriers tell the story.
