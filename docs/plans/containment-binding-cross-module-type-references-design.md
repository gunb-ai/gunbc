# Design — containment binding for cross-module type references

**Status:** DRAFT (sharp-deer-661). Branch: `origin/session/sharp-deer-661` · PR #7621. **This note** is design-only — N2 implementation does not land from the document.

**Authority consumed:** [namespace-resolution-design.md §13](namespace-resolution-design.md#13-resolution-is-unique-on-chain-not-nearest-operator-ruling-ratified-2026-07-21) (unique-on-chain, binding-not-gate, `global_bare` dies as resolution); [namespace unique-on-chain — operational plan](namespace-unique-on-chain-operational-plan.md) (`TypeReference` vs `ValueReference` category, one population fold); [import-strip witness-discovery cascade diagnosis §14](import-strip-witness-discovery-cascade-diagnosis.md#14-post-flip-re-observation--class-b-is-type-only-and-it-fails-open-confirmed-by-execution-2026-07-25) (execution receipt that types diverge from values after the flip; cite **Class B mechanism**, not the historical file roster — §3.2).

**Blocks:** Dispatch 2 mechanical import deletion for type positions; [namespace-resolution-design.md §8](namespace-resolution-design.md#8) terminal step 5 (delete `import` grammar — deps derived from `container.member` references) is **not reachable for type references** on the current resolver in either bare or qualified spelling until this lands.

**Import-deletion graph:** node **N2** (this note). Depends on **N1** (swift-fox-347 — refusal arm for pool-present-but-unbound type misses). The dependency is **verifiability**, not politeness: until N1 lands, a green compile is not evidence that any binding change worked, so N2 implementation cannot be discriminated today even if written.

**Upstream (canonical kernel — active roadmap, not landed for values or types):** `namespace-reference-derived-closure` (candidate producer) and `namespace-canonical-binding` (production changeover) remain **active** roadmap nodes ([operational plan §4](namespace-unique-on-chain-operational-plan.md#4-fresh-canonical-resolver-consuming-edge)). **#7178 predates them** — Dispatch 1 (`NamespaceOnlyY` default ON) improved **legacy** value/fn cross-module binding (import-scoped containment / `str_bindings` paths), **not** production `OccurrenceBindingResult`. N2's **terminal** target is the canonical producer + one fold for `TypeReference`; implementation is **blocked on or must land in concert with** those producer/changeover nodes. Do **not** read N2 as parity extension of an already-live canonical path for values — both categories still need the kernel; types additionally never received even the #7178 legacy extension.

**N2 recut (quiet-hawk-219, 2026-08-01, post-discriminator):** neither “extend a missing walk” nor “fix lookup miss-ordering.” `symbol_index_lookup` on the qualified type path **already consults and hits** when the definer is pool-present; the defect is **treating that flat lookup as binding authority** and peeling to `Product(<anon>)`. N2 completes the **type half** of the incomplete #7178 flip by routing **v1 `TypeReference` occurrences through `namespace-reference-derived-closure` → `occurrence_binding_from_candidates` → `OccurrenceBindingResult`** ([operational plan §4](namespace-unique-on-chain-operational-plan.md#4-fresh-canonical-resolver-consuming-edge)), with category admissibility in the **population collection** step, not a post-lookup gate. That is the **terminal** authority — not yet the live production path for values either. Flat `symbol_index_lookup` / `lookup_type_by_name` as binding selectors **dissolve** at changeover — retained only as bounded migration oracles until census parity. N2 must not make types import-reachable again.

---

## 1. The class — incomplete #7178 flip

Dispatch 1 (#7178, `NamespaceOnlyY` default ON) gave **cross-module value and function references** improved **legacy** namespace-only containment binding (import-scoped / `str_bindings` paths — §14 shows value/fn sites improved relative to pre-flip). **Type references never received even that legacy extension.** The **canonical** candidate producer (`namespace-reference-derived-closure`) and production changeover (`namespace-canonical-binding`) are **not** live for values or types yet — N2's terminal design targets that kernel, not an already-shipped value parity. Until N1+N2 land, type positions still depend on import-scoped `str_bindings` / `ancestry_str_bindings` for legitimate binding — or, worse, on pool coincidence via flat `symbol_index_lookup` hits that peel to `Product(<anon>)`.

N1 (swift-fox-347) installs an interim **refusal floor** on the **legacy infer path**: pool-present + not legitimately bound → located refusal, never anonymous product. N1 uses import-reachability as a **transitional scaffold predicate** on that legacy path. N2 **routes `TypeReference` through the canonical fold** so the scaffold and lookup-as-binding both dissolve.

This is not a cosmetic parity gap. It is a **§5 safety defect** sitting under import-strip Dispatch 2:

1. **Missing binding authority for types (N2)** — lookup can succeed; admission does not. Row 5 proves qualified cross-module types are not **legitimately** bound without import reachability or containment authority.
2. **Fail-open peel (N1)** — illegitimate or absent binding is accepted and peeled to `Product(<anon>)` (`node_type_shape` when `ident_span` is absent) instead of `UnresolvedType` / `AmbiguousReference`.

Import deletion for values is sound; for types it is **unsound and not observable from a green compile** until N1+N2 close the flip.

---

## 2. Execution receipt (do not re-derive — cite the probe)

Measured at main+#7178, `gunbc compile --target dag`, one scratch entry per probe; subject `std.types.ContentHash` referenced from a module that does **not** import `std.types`. Full table: [import-strip diagnosis §14.1](import-strip-witness-discovery-cascade-diagnosis.md#141-what-was-run).

| # | spelling | import-reachable? | pool has `std.types`? | observed |
|---|---|---|---|---|
| 1 | bare | yes (own import) | yes | GREEN |
| 2 | bare | yes (transitive import) | yes | GREEN + advisory `UnlistedImportUse` |
| 3 | bare | no | **no** | RED — `unresolved type 'ContentHash'`, located (**correct**) |
| 4 | bare | no | yes | `Product(<anon>)` — silent except where `if` juxtaposes branches |
| 5 | qualified `std.types.ContentHash` | no | yes | identical to row 4 |

**Load-bearing controls in the same probe file:**

- Same-module bare **callee** in an `if` branch is clean — failure is **type-position specific**.
- Adding an irrelevant import (sibling module that does not reach `std.types`) does not help — not a zero-import degeneracy; it is reachability/binding of the **name**.

**Row 5 is the load-bearing row (calm-badger-682, 2026-08-01):** a fully qualified cross-module type name is **not legitimately bound** today without import reachability or containment admission — lookup may still hit (§3.1).

**N1 refusal floor (Phase A):** pool-present + not legitimately bound must turn row 4 into row 3's refusal shape (`UnresolvedType`, located) — never `Product(<anon>)` — while rows 1 and 2 stay green. That is the **interim** success criterion only; **N2 (Phase B)** may additionally permit rows 4–5 to bind correctly when containment exposes the declaration on the ancestor chain (§7 Phase B discriminating pair).

---

## 3. Root cause — discriminator answered (qualified path); admission missing

The **symptom** is settled by §14 execution. The **qualified row-5 mechanism** is probe-closed (§3.1). Bare row 4 may still differ (`global_bare_lookup` vs qualified projection); fold when swift-fox-347’s full rows 1–5 receipt lands.

### 3.0 Partial-but-reads-complete invariants (quiet-hawk-219 framing correction)

`src/v1/04_env.dag` invariant strings describe **lookup**, not **admission**:

- `global_bare_fallback_invariant` — census includes **type, fn, and data** decls in `symbol_index` / `global_bare` tracking.
- `qualified_module_projection_invariant` — dotted paths in **type positions** call `lookup_qualified_module_projection` → `symbol_index_lookup`.

**Do not record this as stale prose or a prose/execution contradiction.** The discriminator showed the invariant notes are **accurate about the half they describe** (lookup — consult + hit on row 5) and **silent about the half that was never built** (admission — whether a hit is a **legitimate binding** for this occurrence). That is a distinct class, and in one way worse than rotten prose: rotten prose is falsifiable (grep the symbol, find the claim false). A **partial-but-reads-complete** invariant passes cite-the-symbol review — the symbols exist, the described lookup behavior is real — while the reader concludes the mechanism **resolves** qualified type references. True for lookup; false for outcome.

**Record in implementation receipts:**

- The invariant described lookup.
- The admission half was never built.
- The description was not wrong; it was **partial in a way that made the gap invisible and unfalsifiable by symbolic citation**.

**Phase B dissolve-on:** extend both invariant strings to name lookup **and** admission explicitly (symbol-index hit is necessary, not sufficient — binding requires reachability/containment authority per §13 `TypeReference`). That closes the partial-read gap; it is not “correcting false prose.”

**Reusable review tell (cite-the-symbol corpus):** the question is not *“is this claim true?”* but *“what does this note **not** claim about the outcome?”* — what outcome does a reader infer that the invariant never states?

### 3.0b Adjacent defect (N1-owned — do not lose between lanes)

swift-fox-347: `peel_alias_once_for_field_access` (`src/v1/04_infer.dag` / `v1_compiler_infer.rs`) can **drop `resolve_node` diagnostics** during alias peeling — silent refusal loss on a path adjacent to the type-reference fail-open. **Owner: N1 PR** (swift-fox-347), with its own discriminating witness; cited here so it is not dropped between N1 and N2.

### 3.1 Discriminator result (swift-fox-347, 2026-08-01 — **closed for row 5 qualified path**)

**Question:** for the §14.1 row-5 probe (`std.types.ContentHash`, pool-present, not import-reachable), is `symbol_index_lookup` consulted, and what does it return?

**Result (instrumented `lookup_qualified_module_projection`, swift-fox-347 → sharp-deer-661):**

- `symbol_index_lookup` **is consulted** on row 5.
- It **hits** (`hit=true`) for the fully qualified name when the definer module is in the pool.
- The old fail-open was **not** “lookup misses → fabricate.” It **accepted the hit** and peeled downstream to `Product(<anon>)` — pool coincidence masquerading as a binding.

**Settled mechanism (row 5 qualified):** flat lookup succeeds and is wrongly treated as binding. N2 does **not** add a post-lookup admission gate — it routes type occurrences through the **canonical candidate producer** + one `occurrence_binding_from_candidates` fold (`OccurrenceBindingResult`), dissolving lookup-as-binding.

**Still open:** bare row 4 path. swift-fox-347 sent full rows 1–5 + population status to quiet-hawk-219.

### 3.2 Class B file roster is stale — cite mechanism, enumerate fresh (quiet-hawk-219, 2026-08-01)

[import-strip diagnosis §12](import-strip-witness-discovery-cascade-diagnosis.md#12-reconciliation-probe--§9s-loader-vs-env-split-theory-refuted-the-real-mechanism-is-pool-membership-coincidence-execution-confirmed) records a **historical Class B file roster** — batch-1’s ~74 import-stripped `dag/extdeps` files, with `dag/extdeps/bmc/types.dag` as the reconciliation probe specimen. That roster **does not reproduce as today’s population** and must **not** be read as a work list.

**Census join (2026-08-01):** hard type-reference sites on the Class B mechanism today — `bmc/types.dag` **0** (its imports were restored since the diagnosis), `determinism.dag` **1**, `node.dag` **2**. The **Class B mechanism** is confirmed as the same blocker the type census measures: stripped-file bare cross-module type references that bind only by **pool-membership coincidence**, not by closure-independent containment admission. The file list was a **snapshot of where the mechanism last bit**, not a durable subject universe.

**Do not pick up the historical roster as a starting point.** A stale file list is worse than no list — it routes implementers toward files that no longer exhibit the defect while missing files that do.

**General rule (third instance of this class today):** positional namings of a **population** outlive the thing they named, same as (1) `file:line` citations where a symbol exists (DESIGN §3 cite-the-symbol) and (2) partial-but-reads-complete invariant strings (§3.0). **Cite the mechanism as the durable name** (`Class B` pool-membership coincidence for stripped-file type references; `resolution_divergence_census` / type-site census for measurement). **Enumerate the population fresh at use time** — never inherit a diagnosis-era file roster as implementation scope.

---

## 4. N1 ↔ N2 swap seam (design constraint)

N1 and N2 share one outcome: **no illegitimate bind, never `Product(<anon>)`**. They differ on **which path** carries binding authority during migration.

| Layer | N1 (swift-fox-347, interim scaffold) | N2 (this note, terminal) |
|---|---|---|
| **Binding authority** | Legacy infer path: import-reachability (`str_bindings` / `ancestry_str_bindings`) — refuse when not import-reachable | **Canonical path:** `namespace-reference-derived-closure` candidate producer (category-admissible containment population for `TypeReference`) → **one** `occurrence_binding_from_candidates` fold → `OccurrenceBindingResult` |
| **On reject** | Located `UnresolvedType` / `AmbiguousReference`; never `Product(<anon>)` | Same refusal surface (`OccurrenceUnbound` / `OccurrenceAmbiguous` projected to type diagnostics) |
| **On accept** | Only when import-reachable on legacy path today | `OccurrenceBound` from canonical fold — ancestor-chain unique-on-chain, **without** import |

**Swap rule:** N1 lands the **refusal floor on the legacy infer path** (transitional scaffold — loud, counted, dissolution = `TypeReference` consumes `OccurrenceBindingResult`). N2 **does not** keep lookup and add a post-hoc admission predicate — that would institutionalize a second binding decision path (DESIGN §3 fork). N2 wires `TypeReference` through **`namespace-reference-derived-closure` + one canonical fold** (same **target** kernel values will migrate to at changeover — not what #7178 already shipped); then removes the import scaffold and dissolves `lookup_type_by_name` / flat `symbol_index_lookup` as **binding** selectors (migration-oracle retention only until census parity).

**Anti-goal:** making types “import-reachable” again — that points backward; Dispatch 2 deletes imports.

---

## 5. Target semantics — category-filtered population, one fold

No second namespace mechanism. A type reference uses the same §13 algorithm through the operational plan’s **one candidate producer + one fold** — category admissibility filters the **population before** `occurrence_binding_from_candidates`, not after a separate lookup:

**Bare reference** (single segment): resolve to the **unique binder** on the occurrence's ancestor chain; the bound declaration must be **type-position admissible** (type-like).

**Qualified reference** (`std.types.ContentHash`, …): resolve the **first segment** to the unique binder on the ancestor chain — typically a **namespace/module container** (`std` is not type-like). Each **nonterminal** segment is a container/module-member projection (`std` → `types`). Apply type-position admissibility only at the **terminal** projection: the final name must bind to a type-like declaration (`ContentHash` as `type` or type-alias).

> **First segment only (§13):** zero binders on the ancestor chain → `UnresolvedType`; two-or-more → `AmbiguousReference` with full candidate list; exactly one → bind and project remaining segments through the one `.` / containment projection op. The unique-on-chain fold is **not** re-run on later segments — projection descends through the bound container; projection failures (zero/many members) are located refusals at the projection step, not a second ancestor-chain population fold.

**Type-like** means declarations admissible at type positions: `type` and alias nodes whose resolved target is type-like — **not** `fn`/`func` and **not** `data` (value declarations). Namespace-only containers are legitimate **qualification prefixes**, not terminal type targets. Symmetric to value positions admitting callables and value `data`, refusing bare type decls as callees.

**Cross-module** is not a separate tier. A declaration in a sibling namespace (e.g. `std.types.ContentHash`) is visible when it is **exposed by a scope on the referencing occurrence's ancestor chain** — not when the declaring module path is a string prefix of the referencing module path. Top-level containers like `std` are exposed by the root / enclosing module scope even though `std` is not a path ancestor of `extdeps.*` modules. That is exactly what makes `import` redundant at Rule-1 end-state: `std.types.ContentHash` in a signature **is** the dependency edge.

**Qualified paths:** nonterminal segments project through namespace/module containment; terminal segment must be type-like. Never route qualified type paths solely through flat `symbol_index_lookup` from corpus root without position.

**Terminal posture (inherits §13):** delete `global_bare` / import-scoped `ancestry_str_bindings` as *resolution* mechanisms once containment binding for both categories is proven; they remain migration oracles until then.

---

## 6. Where the binding lives — canonical producer + one fold

| Consumer | Today | Target |
|---|---|---|
| v2 model | `symbol_index_lexical_lookup` (`std.symbol_index`) — parallel binding result | `SymbolIndex.entries` as materialized containment storage; project candidates and fold through `OccurrenceBindingResult` — dissolve `LexicalLookup` as parallel result carrier ([operational plan cleanup](namespace-unique-on-chain-operational-plan.md#cleanup-census)) |
| v2 resolve | `resolve_atom` / policy pilot tests | Type positions through candidate producer with `TypeReference` category |
| v1 typecheck | `lookup_binding_by_name` / `lookup_type_by_name` / flat `symbol_index_lookup` as binding selectors | **Route type occurrences through canonical candidate producer → `OccurrenceBindingResult`**; dissolve lookup maps as binding selectors — oracle retention only until census parity (explicit dissolution trigger) |
| Census / codemod oracle | `containment_resolve_fn_v1_for_module` (fn-only) | Generalize to `containment_resolve_decl_v1(category, …)`; extend `resolution_divergence_census` to **type annotation sites** (fn params/returns, `data`, field types, type args) |
| Fail-open chokepoint | infer/unify accepts anonymous `Conj` without `ident_span` | **First fix:** any type leaf without `OccurrenceBound` / resolved binding → typed refusal, never widen to anonymous product |

**Dissolve-on markers (named, not optional):**

- `infer_arrow_domain_binding_heuristic_note` (`src/v2/compiler/04_infer.dag`) — delete DFS "first Arrow with matching domain" when `TypeReference` consumes `OccurrenceBindingResult`.
- `lookup_type_by_name` / `lookup_qualified_module_projection` as **binding** paths — dissolve when canonical producer lands; retain for migration census only.
- `reference_derived_use_lines_note` (`src/v1/05_emit_rust.dag`) — emit-side missing `use` lines; separate §5 fail-open from binding, but promote with hard type refusals once rows 1–2 hold without import lists.
- Selective-import masked `UnlistedImportUse` advisory for type positions — promote to hard refusal once containment binding proves rows 1–2 without import lists (pairs with emit lane above).

Do **not** add a second type lookup in emit, closure derivation, or census — §2 horizontal: one closure, N consumers.

---

## 7. Sequencing — §5 order is mandatory

From [import-strip diagnosis §14.4](import-strip-witness-discovery-cascade-diagnosis.md#144-consequence-for-the-wave-rule):

### Phase A — refuse the fail-open (**N1**, swift-fox-347)

**Trigger:** independent of whether import strip resumes.

**Obligation:** pool-present + not-bound at type position → located, typed, counted refusal (`UnresolvedType` or `AmbiguousReference`), **never** `Product(<anon>)`.

**Discriminating pair:** row 4 becomes row 3; fabricated-vs-fabricated `if` branches must not typecheck clean.

**Rung target:** mitigatable → mechanically preventive (witness executes on every compile).

Until Phase A lands, **no strip green is typing evidence** for types.

### Phase B — canonical binding for `TypeReference` (N2)

**Trigger:** Phase A green on §14.1 matrix; row-5 qualified discriminator closed.

**Obligation:** route **type positions** through **`namespace-reference-derived-closure` → `occurrence_binding_from_candidates` → `OccurrenceBindingResult`** — not lookup + post-hoc gate. **Sequencing:** lands with (or after) the active producer/changeover roadmap nodes; #7178 legacy value/fn binding is **not** a substitute prerequisite. Category-admissible population for `TypeReference`; changeover dissolves legacy lookup-as-binding for types alongside the value/fn migration.

1. Route v1 type occurrences through canonical candidate producer + fold (replace lookup-as-binding).
2. Remove N1 import-reachability scaffold from infer legacy path.
3. Generalize census oracle to type sites (`containment_resolve_decl_v1`).

**Dissolve-on for invariant strings:** extend `global_bare_fallback_invariant` and `qualified_module_projection_invariant` to name lookup **and** admission (hit ≠ bind) — close partial-read gap, not “fix stale prose.”

**Discriminating pair:** rows 1–2 stay green; rows 4–5 become located refusals **or** correct binds when the declaration is exposed on the ancestor chain (fixture-dependent); row 3 unchanged.

### Phase C — re-authorize Dispatch 2 for types

**Trigger:** Phase B + full §14.1 matrix green; `resolution_divergence_census` shows type-site agree for stripped fixtures.

**Effect:** import-line deletion for type-only cross-module refs becomes sound; pairs with namespace §8 step 5 only after corpus-wide flip.

---

## 8. Witnesses (to author when implementation starts — none exist as executing claims yet)

**Primary matrix:** replay §14.1 eight probes as `dag/test/claim/` rows (or extend import-strip probe harness) with executed RED/positive controls per row.

**Minimal discriminating subset:**

| witness | asserts |
|---|---|
| `cross_module_type_pool_absent_refuses` | row 3 — `UnresolvedType`, located |
| `cross_module_type_pool_present_unbound_refuses` | row 4 after Phase A — refusal, not `Product(<anon>)` |
| `cross_module_type_qualified_unbound_refuses` | row 5 — same as row 4 after Phase A |
| `cross_module_type_containment_bind_bare` | stripped module, ancestor-visible decl, bare type in fn sig — GREEN without import |
| `cross_module_type_containment_bind_qualified` | `std.types.ContentHash` with projection — GREEN when chain reaches `std.types` |
| `cross_module_value_control_stays_green` | row 4 control — bare callee still binds (no regression) |
| `import_reachable_stays_green` | rows 1–2 unchanged |
| `namespace_only_type_homonym_refuses` | extend `namespace_only_policy_test` beyond policy pilot — two on-chain type homonyms → `AmbiguousReference` |

**Census witness:** type-site `resolution_divergence_census` row for a stripped fixture showing `ImportUnresolved` → `Agree` once containment binds (mirrors fn path #6936 methodology). **Enumerate the subject population at witness authoring time** (§3.2) — do not replay import-strip §12’s batch-1 file roster.

**Regression guard:** `node.dag` six `ContentHash` `if` sites are **accident detectors**, not the fix — witnesses must catch silent mis-typing without relying on branch juxtaposition.

**N1 adjacent (swift-fox-347):** witness that `peel_alias_once_for_field_access` preserves or surfaces `resolve_node` diagnostics — no silent drop on alias peel (§3.0b).

---

## 9. Scope boundaries

**In scope for this note:** bare and qualified **type** references across module boundaries at compile time (resolve + infer binding); parity with §13 unique-on-chain; closing the §14 fail-open.

**Explicitly out of scope (separate lanes):**

- Emit-side `reference_derived_use_lines` / Rust `use` line derivation (downstream consumer; still §5 fail-open for emission, not for binding).
- `feature:indexed-variant-path-projectable` **within** a single imported type (`Type.Variant` paths) — converges on projection machinery but has its own acceptance row; do not block Phase B on variant-alias shadow cases unless they share the same bug.
- Runtime/interpreter registration order (namespace-resolution-design §12 — separate three-way fork).
- Deleting the `import` grammar (terminal §8 step 5) — this note is a **prerequisite**, not the deletion PR.
- `global_bare` deletion — terminal oracle removal after both categories prove parity.

**Load-bearing files (escalate before editing under a narrower brief):** `v1_compiler_infer_env.rs`, `v1_compiler_infer_resolve.rs`, `std.symbol_index`, `03_resolve.dag`, `symbol_index_fill.dag`.

---

## 10. Open verification threads (honest gaps)

- **§3.1 discriminator — row 5 qualified:** **closed** — flat lookup hits and wrongly binds; N2 = canonical producer + fold, not post-lookup gate. Bare row 4 outcome pending.
- **`peel_alias_once_for_field_access` diagnostic drop:** N1-owned witness (§3.0b).
- **Globally unique names vs chain-only:** terminal §13 deletes whole-pool unique shortcut; Phase B applies via category-filtered population in candidate producer.
- **Re-validate §10 determinism LOUDNESS** after Phase A — field-type shapes only if still reproducing.
- **Invariant strings (partial-but-reads-complete):** extend lookup-only `04_env.dag` invariants with admission half when Phase B lands (§3.0).
- **Class B roster hygiene:** historical batch-1 / `bmc/types.dag` probe file list is stale (§3.2); census join today: `bmc/types.dag` 0, `determinism.dag` 1, `node.dag` 2 hard sites.

---

## 11. Related documents

- [import-strip witness-discovery cascade diagnosis §13–14](import-strip-witness-discovery-cascade-diagnosis.md) — wave rule + post-flip type-only receipt (**Class B mechanism**; §12 batch-1 roster is historical snapshot only — §3.2)
- [namespace-resolution-design.md §13](namespace-resolution-design.md#13-resolution-is-unique-on-chain-not-nearest-operator-ruling-ratified-2026-07-21) — ratified lookup semantics
- [namespace unique-on-chain — operational plan](namespace-unique-on-chain-operational-plan.md) — `TypeReference` category, one population fold
- [type environment: single import authority](type-env-single-authority-design.md) — env projection the walk migrates onto
- [layering-imports gate repoint scoping](layering-imports-reference-repoint-design.md) — CI gate that waits on namespace binding for strip waves
