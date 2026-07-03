# Plan — fold ergonomics: make the fold the path of least resistance

**Status:** charter + ranked focus · **DESIGN.md + the carriers remain the authority** (DESIGN §6). A task's real state is its branch/PR, not this file. Linked from `ROADMAP.md` ✦ *Ergonomics LANE*.

**Carrier facts below were gathered by two tree-wide sweeps (fold inventory + non-fold-residue/friction), 2026-06-22. Re-grep the live counts before acting — they rot.**

---

## 0. Thesis — "it compiles but nothing works" traces to non-fold residue

A hand-rolled `match` carries a `_ =>` fail-open escape; a fold over a *closed* coproduct is total by construction and has none. So the causal chain is **ergonomics → adoption → fail-closed**: when the fold is awkward to reach, people hand-roll, and every hand-roll reintroduces a fail-open arm. This lane **stops new residue by making folds ergonomic**; §0's fail-open-shape walls **retire the old**. The two together drain the [model↔realization fork](model-realization-fork.md), the residue's deepest instance.

Guardrail (DESIGN §6 — ergonomics is the #1 purity-trap magnet): every item **names the fail-open class or measured friction it retires** (displaced cost), never "cleaner."

## 1. The spine is already real (accomplishments, with receipts)

The fold is not aspirational here — it is the actual architecture, at scale:

- **`fold_node`** (`src/v2/std/node.dag:228`) — one catamorphism, **69+ call sites across 7 stages** (translate 43, eval 12, name_resolve 7, infer 3, resolve 2, compile 2). Traversal is not re-coded per stage; operations come from *inhabitance*, not per-type op lists (DESIGN §4/§6).
- **`bind_outcome`** (`src/v2/std/diagnostic.dag:225`) — **246+ sites** (translate 172, eval 31, compile 31, ingest 12). The `Outcome` railway monad is the genuine spine, not a demo.
- **`coercion_fold`** (`src/v2/std/coercion.dag:279`) — **50+ sites**, *one* procedure asked three directions (ingest-forward / emit-inverse / homomorphism check) via the same `find_witness` catamorphism. The **model-side coercion fold is already built** (this reframes any "build the coercion fold" framing — see Root A).
- **#4699** — `06_translate` de-accumulated: 4,912 → 3,973 lines, `_go` accumulators **35 → 0**. The single largest "stop hand-rolling recursion-with-accumulator" receipt in the tree.
- **#5512** — front-end de-pyramided: `compile_ingest_staging` went from a 5-deep `bind_outcome` pyramid to a fold of five first-class typed stages under **`then_outcome`** (Kleisli for the `Outcome` monad); `cached_stage` wired live (seam consumed, not stranded). The seed this lane names.
- **#5428 / cardinality P4** — numeric tower grounded fail-closed; cardinality propagates through folds as a homomorphism, uint8 overflow → typed `Rejected`. A model↔realization fork instance *closed*.
- **`merge_envs`** — a 6-line root fix cut reconcile from 81% → 6% of the pipeline (~2× self-compile). The canonical "fix the language layer, not the symptom" win — the lane's whole thesis in one diff.

## 2. The diagnosis — two roots, the lane's two halves

**Root A — the realization side isn't folded (the fail-open).** The model side folds; the Rust `Value` enum is reconciled by **~120 per-site `_ => false` bridges** — `Value::eq` (`v1_interpreter.rs:707`), binop coercion, 63 inference mismatches, 91 in `complexity.rs`. A modeled coproduct (`Nat = Zero|Succ`) vs its native realization (`Int`) is compared per-site, and a miss is a *silent false* (`nat_add(85,32) == 117` → `false`). This is DESIGN's model↔realization fork, and it is where wrong answers actually hide. The deepest remaining instance is the **`Value::Null` split**: `Optional`/`Witness`/miss overloaded onto one sentinel across **~131 sites** — it resists a blanket guard because it *is* a fork, so it needs grounding, not an error arm.

**Root B — generic inference is weak, so the fold is awkward to reach (the friction).** Two concrete failures:

- generic fn-param results mis-infer as kernel `Witness`/`Optional` → must route through a typed param (the `resolve_probe` workaround in `staging.dag`; same in `target_model.dag`);
- generic type-alias instantiation fails — `type QualifiedName = FreeMonoid<Symbol>` won't define ("variant not found in type `FreeMonoid`") → **55 lines of hand-rolled `qualified_name_eq`/`for_all`** (`qualified_name.dag:25–57`), and it is the *same* root that makes v2 still hand-roll `ParseTable`.

Root A is the fail-open class the lane retires; Root B is the friction that keeps producing it.

## 3. Ranked focus (displaced cost named, per the §6 guardrail)

**#1 — fix generic inference** *(fix keystone — the root with measured fan-out)*. **Retires:** the typed-param-workaround tax and the `FreeMonoid<Symbol>` block. One well-scoped inferencer change collapses a fan-out of hand-rolls — `qualified_name`'s trio → `fold_list`/`==`, `ParseTable` → the Realization carrier, the `cached_stage` indirection → a plain inline match. This is the literal "make the fold the path of least resistance": today the fold is reachable but *taxed*; after this it is reachable *by default* — the only item that changes the default, which is the whole point of an ergonomics lane. Dissolution trigger already exists (`feature:free-monoid-entry-generic-inference`); a root fix, not a grind. **Prerequisite for #2** (makes the realization grounding expressible as a fold, not a 131-site hand-grind).

**#2 — ground each primitive into its realization** *(highest safety / displaced cost)*. **Retires:** the ~120 silent-`false` `Value` bridges. Make the straddle *unwritable* — native form `==` modeled form by construction, so the per-site arms disappear (exactly what #5428 did for the numeric tower, fail-closed). The deep sub-root is the `Value::Null` split (~131 sites, own runway). The safety axis literally: every silent `false` is a deferred bug paid later at interest.

**#3 — the lens backstop** *(cheapest gate; gates #1/#2 so new residue can't merge)*. **LANDED** (Lane 7, this PR): the **inert-abstraction lens** (`v2.lens.inert_carrier` over `inert_carrier_project.rs`) flags a type carrier that is *defined + self-tested + zero real consumer* — DESIGN §5 coverage-by-illusion (the `self-tested` gate is what separates it from the project's deliberate model-first staging); and the **non-fold-residue audit** (`v2.lens.non_fold_residue` over `non_fold_residue_project.rs`) flags a `match` whose scrutinee is a closed-coproduct param carrying a `_ =>` wildcard escape. Both are fail-closed floor witnesses (`src/v2/lens/{inert_carrier,non_fold_residue}_test.dag`, green-by-execution through `claim_batch`), each walled against a named, shrinking exception roster with a stale-roster ratchet, and each proven discriminating by synthetic RED/GREEN host controls (inert→RED, consumed→GREEN; residue→RED, total-fold→GREEN). Host-fed today; both fold into pure `.dag` Node-tree readers at gunbc#5364. Remaining audit targets (deferred, follow-up): `unwrap_or_default` in inference · hand-rolled recursion where a fold exists.

**Start with #1.** It is the most elegant (a root with measured fan-out), it is the prerequisite that makes #2 a fold instead of a grind, and it is the only one that changes the default.

## Dissolution trigger (DESIGN §6)

Delete this doc when the fold is reachable by default — generic inference no longer forces typed-param workarounds (Root B closed), the realization side grounds into the model with no per-site `_ => false` bridges (Root A closed), and the inert-abstraction lens gates new residue on the floor. At that point the absent residue + the green lens *are* the authority and this charter is redundant.
