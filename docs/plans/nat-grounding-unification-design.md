# `nat` grounding-unification: one `Nat` authority across the two std trees

Status: **census complete (execution, 2026-08-01)** · decision-input for the operator grounding-cluster lane · linked from [dag-v2-defork-audit.md](dag-v2-defork-audit.md) grounding cluster · supersedes the audit's stale **"LIVE — 4 entries"** co-occurrence figure for `nat` (see §2).

Executes the per-concept authority design the de-fork audit requires before any repoint: *"the grounded dag authority for each has to be designed before any fan-out can repoint to it."* This is that design for **`nat`**, sequenced **after** `algebra`'s `FreeMonoid` shadow-removal (#6341) and **with** `integer`/`float` (numeric tower `Int = GroupCompletion<Nat>`). Escalated (smart-ant-466); audit marks **BLOCKED + LAST** among grounding-cluster repoints.

Reasoned serially per the DESIGN preamble: §1 fixes the problem, each later section is a consequence.

## 1. The problem (measured, by execution)

Two files define `nat` on the two std trees:

- `dag/std/nat.dag` — module `std.nat`, **4 decls**, **61 live importers** in `dag/` (see §4.0).
- `src/v2/std/nat.dag` — module `v2.std.nat`, **12 decls**, **37 live importers** in `src/v2/` (see §4.0).

They share **one unqualified type name** (`Nat`) and **one shared fn name** (`nat_compare`). The shared `Nat` body **diverges**: dag = thin alias `Nat = CommutativeSemiring<Magnitude>`; v2 = coproduct `Nat = Zero | Succ { prev: Nat }`. The shared `nat_compare` body also **diverges** (semiring order via `<`/`>` vs coproduct order via `==` and `<`).

When a single import closure pulls in both trees, `Nat` binds ambiguously. Today this is the **benign record-with-record shadow** (not the coproduct-variant-drop that broke `verification` under A1): dag's `Nat` is a record alias, not a competing coproduct variant set. It still violates §3 single authority and will fire the flag-ANY type-name-collision wall once the five-basename grounding de-fork completes ([resolver-type-name-collision-wall.md](resolver-type-name-collision-wall.md) §4).

Runtime is **ahead of the substrate**: #5428 grounded Nat construction-side in the interpreter (`Zero → Value::Int(0)`, `Succ { prev: Int(k) } → Value::Int(k+1)`); `dag/std/coercion.dag` `grounded_primitive_coproduct_cast_note` treats Nat as Int at the cast boundary. The **model fork** (semiring alias vs coproduct) is what remains.

## 2. Co-occurrence census (re-verified 2026-08-01)

Method: BFS import closure from every `*_test.dag` floor entry under `dag/test/` and `src/v2/test/` (946 entries); flag closures containing both `std.nat` and `v2.std.nat`.

| metric | 2026-06-23 audit (§2A) | **2026-08-01 (this census)** |
| --- | --- | --- |
| floor `*_test.dag` entries scanned | 351 | **946** |
| closures with **both** `std.nat` + `v2.std.nat` | **4** | **138** |
| all `src/v2/**/*.dag` closures with both | (not reported) | **157** |

The growth is closure expansion (more floor entries, wider CI/gunbc surfaces importing `std.nat` from v2 workflow modules while the v2 compiler tree still pulls `v2.std.nat` through `integer`/`datetime`/lens chains) — the same latent→LIVE risk the audit warns about for `{effects, float, integer}`.

**Direct `std.nat` import inside `src/v2/` (5 modules)** — each one's closure also reaches `v2.std.nat`:

- `src/v2/workflow/ci_floor_plan.dag`
- `src/v2/workflow/ci_placement.dag`
- `src/v2/std/orchestration.dag`
- `src/v2/std/effect_plan.dag`
- `src/v2/lens/affected_set/corpus_dependency_view.dag`

Sample floor entries with live co-occurrence (first 10): `dag/test/claim/ci_materialization_witness_test.dag`, `ci_workflow_witness_test.dag`, `commit_workflow_witness_test.dag`, `effect_plan_bash_materialize_real_execution_witness_test.dag`, … (**128 more**).

## 3. Declaration census (complete, by execution)

### 3.1 `dag/std/nat.dag` (4 decls)

| decl | kind | body |
| --- | --- | --- |
| `Nat` | type alias | `= CommutativeSemiring<Magnitude>` |
| `nat_compare` | fn | semiring `<` / `>` |
| `nat_max` | fn | semiring `>` |
| `nat_min` | fn | semiring `<` |

### 3.2 `src/v2/std/nat.dag` (12 decls)

| decl | kind | notes |
| --- | --- | --- |
| `Nat` | coproduct | `Zero \| Succ { prev: Nat }` |
| `nat_cata` | fn | fold |
| `nat_add` | fn | |
| `nat_mul` | fn | |
| `nat_compare` | fn | coproduct `==` / `<` |
| `is_zero` | fn | manual `match` on coproduct (§3.3 A′ — must dissolve into `nat_cata`) |
| `nat_lte` | fn | |
| `nat_gte` | fn | |
| `nat_additive_commutative_monoid` | data | `CommutativeMonoid<Nat>` witness |
| `nat_semiring` | data | `CommutativeSemiring<Nat>` witness |
| `NatAlgebraLawObligation` | type | law roster row |
| `nat_declared_algebra_law_obligations` | fn | 6 law rows |

Imports: `std.algebra` (structures + `Ordering`), `v2.std.node { Symbol }`, `v2.std.collection { List }` — the law-obligation roster is **v2.node-bound** (`Symbol` anchors).

### 3.3 Classification (decidable rule for workers)

| category | decls | action |
| --- | --- | --- |
| **A — structural authority (MOVE to `std.nat`)** | `Nat`, `Zero`, `Succ`, `nat_cata`, `nat_add`, `nat_mul`, `nat_lte`, `nat_gte`, `nat_compare` (coproduct impl), `nat_additive_commutative_monoid`, `nat_semiring` | Coproduct + Peano ops are the grounded model (#5428); become the single `std.nat` authority. |
| **A′ — MOVE with predicate dissolution (not verbatim)** | `is_zero` | Today `v2.std.nat.is_zero` is a forbidden coproduct predicate — manual `match` on `Zero`/`Succ` while `nat_cata` is already the canonical fold. Migration **re-expresses** as `nat_cata(n: n, zero: true, succ: fn(_) { false })` (or an equivalent derived one-liner) and **deletes** the manual walker; never copy the match arm verbatim into `std.nat`. |
| **B — DELETE (alias dies)** | dag `Nat = CommutativeSemiring<Magnitude>` | Not a second definition — replaced by coproduct + inhabitance-derived semiring instance (`nat_semiring` data). |
| **C — STAY in v2 tree (node-bound)** | `NatAlgebraLawObligation`, `nat_declared_algebra_law_obligations` | Import `v2.std.node { Symbol }`; cannot cross to `dag/std` until `node` defork (audit category (c)). Thin `v2.std.nat` (or `v2.std.algebra_laws`) module importing `std.nat` for `Nat`/`Zero`/`Succ`/ops. |
| **D — MERGE / re-home** | dag `nat_max`, `nat_min` | 4 + 2 live importer modules in `dag/` (§4.0); re-express on coproduct `nat_compare` or move beside coproduct ops in unified `std.nat`. |
| **E — OUT OF SCOPE (adjacent modules)** | `src/v2/std/algebra_laws/nat_semiring.dag`, `src/v2/workflow/nat_semiring_rung*_eval.dag`, `src/v2/test/claim/manual/nat_law_anchors.dag` | Law **witness** harnesses; repoint imports when A lands, not part of the type fork itself. |

**Shared-name verdict:** 1 shared type (`Nat`, bodies diverge); 1 shared fn (`nat_compare`, bodies diverge). **No** extra shared type-names beyond `Nat` (unlike `algebra`'s 16).

## 4. Importer census (symbols)

### 4.0 Census counting rules (subject universe)

Importer counts classify **live modules** — `.dag` files with an authored top-level `import <module>` line — not transitive closure reachability and not substring matches elsewhere in the file.

| rule | predicate | `dag/` `std.nat` | `src/v2/` `v2.std.nat` | `src/v2/` `std.nat` |
| --- | --- | --- | --- | --- |
| **Census (this doc)** | `^import <module>\\b` on a `.dag` line | **61** | **37** | **5** |
| loose grep | `import <module>` anywhere in file | 63 | 40* | 5 |

\*The three `src/v2/` files above 37 are **false positives**: `import v2.std.native_agreement` matches the substring `import v2.std.nat` under a naive grep — not nat importers.

**`dag/` reconciliation (63 vs 61):** the two files in the loose count but not the census count carry `import std.nat` only inside **embedded witness fixture strings**, not as live import declarations:

- `dag/test/claim/where_refinement_enforcement_witness_test.dag` (10 string literals)
- `dag/test/claim/root4_measure_missing_generics_witness_test.dag` (1 string literal)

Zero `dag/` importer paths contain `fixture`. The census subject is the **61 modules** in the table below; each has exactly one classification (direct importer of `std.nat` or `v2.std.nat`, or co-occurrence-only via closure — §2).

### 4.1 `v2.std.nat` (37 live importer modules)

| symbol | import sites |
| --- | --- |
| `Nat` | 21 |
| `Zero` | 21 |
| `Succ` | 7 |
| `nat_add` | 3 |
| `nat_compare` | 3 |
| `nat_mul` | 2 |
| `is_zero` | 1 |
| `nat_lte` | 1 |
| `nat_gte` | 1 |
| `NatAlgebraLawObligation` | 1 |
| `nat_declared_algebra_law_obligations` | 1 |

**Load-bearing importers:** `v2.std.integer` (`Int = GroupCompletion<v2.std.nat.Nat>`), `v2.std.float`, `v2.std.datetime` (56 refs), `v2.std.cardinality`, `v2.compiler.01_tokenize`, `v2.lens.cost`, `v2.lens.testgen`, `v2.test.claim.generated.algebra_law_conformance`.

### 4.2 `std.nat` (61 live importer modules in `dag/`)

| symbol | import sites |
| --- | --- |
| `Nat` | 64 |
| `nat_max` | 4 |
| `nat_min` | 2 |
| `nat_compare` | 2 |

**v2-tree modules importing `std.nat` directly:** 5 (listed §2). No `src/v1/**/*.dag` import of either nat module (v1 tests embed `import std.nat` in string fixtures only).

## 5. Grounding entanglement (why this is a design, not a repoint)

| entanglement | evidence | consequence |
| --- | --- | --- |
| **#5428 numeric tower (runtime)** | `v1_interpreter.rs` Nat eval; `cross_representation_equality_test`; `std_coercion.rs` grounded_primitive note | Coproduct is the **realized** form; dag semiring alias is stale relative to runtime. |
| **`integer` fork** | `std.integer.Int` (`AbelianGroup<GroupCompletion<Nat>>`) vs `v2.std.integer.Int` (`GroupCompletion<v2.std.nat.Nat>`) | Unification must land **with** integer repoint; `Nat` in `GroupCompletion<Nat>` must be the coproduct authority. |
| **`float` fork** | imports `v2.std.nat` ops | Same tower lane. |
| **Emitter / trait derive** | `GroupCompletion<Nat>` phantom params; `trait_derive_shape_grounding_lane_handoff` | `Nat` coproduct must be the type arg GroupCompletion carries after merge. |
| **Algebra law roster** | `nat_declared_algebra_law_obligations` + generated conformance | Ops + instances must stay consistent with unified `std.nat`. |
| **Magnitude alias** | dag `Nat = CommutativeSemiring<Magnitude>` | `std.magnitude` path dies; semiring witness becomes `nat_semiring` on coproduct (category A). |

**Not entangled with v1 seed module rename:** unlike `coercion`/`node`, no `src/v1/**/*.dag` imports `std.nat` — no seed-regen rename cascade from `dag/std/nat.dag` edits alone.

## 6. Target authority (consequence of §1–§5 + operator §3b ruling)

Per the 2026-06-22 operator ruling on the grounding cluster (coproduct structural authority; grounded-realization wins):

1. **`Nat = Zero | Succ { prev: Nat }` lives once in `std.nat`** (`dag/std/nat.dag`).
2. **Peano ops + `nat_semiring` / additive monoid instances** move to `std.nat` (category A).
3. **`dag` semiring alias and magnitude-backed `nat_compare`/`nat_max`/`nat_min`** delete or re-derive from coproduct ops (categories B/D).
4. **`v2.std.nat` survives thin** — category C law-obligation roster (+ any node-bound residue); `import std.nat { Nat, Zero, Succ, … }`.
5. **`integer`/`float` repoint** uses `std.nat.Nat` inside `GroupCompletion<Nat>` — mechanical once steps 1–4 land (integer/float defork rows).

This is **not** "delete v2 copy and repoint" alone: the dag-side alias must be **replaced**, not merged additively.

## 7. Sequencing, walls, and gates

**Prerequisites (do not skip):**

- `algebra` `FreeMonoid` shadow removed (#6341) — same pattern, same wall family.
- Keystone generic inference for recursive coproducts (`generic_alias_coproduct_instantiation_test.dag`) — Nat `Succ` tail is the same fixpoint class as `FreeMonoid` `Cons`.

**Hard sequencing (audit + resolver wall):**

- `nat` unification is **paired with `integer`/`float`** repoint in the mechanical wave after coproduct authority exists.
- **flag-ANY collision wall** lands only after `{algebra, nat, effects, float, integer}` all reach single authority — `nat` is in the LIVE pair with `algebra`.

**Walls surfaced by execution (analogous to algebra §9):**

| wall | detail |
| --- | --- |
| **Law roster node-binding** | `nat_declared_algebra_law_obligations` uses `Symbol` — stays v2-side (category C) like algebra encoding fns. |
| **Qualified `v2.std.nat.Nat` in integer** | `v2.std.integer.Int` body cites `GroupCompletion<v2.std.nat.Nat>` | Must become `GroupCompletion<Nat>` under `std.nat` after repoint. |
| **Co-occurrence surface** | 138 floor entries — any partial collapse must be **atomic** (§5 auto-committer hazard). |

**Explicitly NOT this lane's first PR:** full `integer`/`float` tower, `GroupCompletion<M>` pair construction (#7197), hollow-alias wall — tracked separately.

## 8. Validation (§5 prove-by-execution)

When implementation lands:

- **Discriminating witness:** closure importing both trees resolves `Nat` to coproduct (`Zero`/`Succ` constructible); red control = restore v2 duplicate `Nat` alias and assert shadow / wall fires.
- **Numeric tower:** `cross_representation_equality_test` + nat add/mul laws green on unified authority.
- **Co-occurrence:** sample of the 138 entries typecheck with single `Nat`.
- **Byte-identical emit fixpoint** if `dag/std/nat.dag` body changes trigger seed regen (verify — may be avoidable if only v2 duplicate deletes first).

## 9. Dissolution

Delete this doc when `Nat`/`Zero`/`Succ` + Peano ops live only in `std.nat`, `v2.std.nat` imports rather than redefining them, the 138-entry co-occurrence class is gone, and `integer`'s `GroupCompletion<Nat>` cites the unified type. Full defork-audit row for `nat` can then read **DONE**.
