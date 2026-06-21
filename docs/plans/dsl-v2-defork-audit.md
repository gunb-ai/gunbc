# Plan — De-fork `dsl/` ↔ `src/v2/` (collapse the duplicated standard library)

**Status:** carrier-grounded audit + sequencing · **DESIGN.md + the carriers remain the authority** —
this doc is an audit/tracker, not a fact ledger (DESIGN §6 "no parallel-ledger docs"). Each collapse
dissolves into a mark on its carrier (a deleted file, a re-pointed import) when it lands. A task's real
state is its branch/PR, not this file. Linked from `ROADMAP.md §3`.

**Verified against the live tree on 2026-06-21.** Line numbers are receipts; re-check before acting.

---

## 0. Thesis — the only duplication is v2's bootstrap copies of dsl

`dsl/` is the single authority (the standard library + the grounded `extdeps/` domain models +
the CI spec). `src/v2/` is the **compiler**, and a compiler needs a standard library to run. It
cannot import `dsl/` across trees yet, so during bootstrap it made **mirror copies** of pieces of
`dsl/std` inside `src/v2/std`. Those copies are the entire fork surface.

So "de-fork" is not "move dsl into v2" and not a blanket direction. It is: **delete v2's duplicate
copies and point v2 at the dsl authority**, until no concept has two homes — and no genuinely
historical fork survives (a name shared by two copies, or two names for one concept). Folder/module
naming must reflect one authority, not the fork's history.

The fork surface is small: **11 overlapping `std` basenames** (plus a couple in `extdeps`, e.g.
`yaml`). `dsl/extdeps/**` domain models (~140 dsl-only files), `dsl/product/`, `dsl/gunbc/`, and
~60 dsl-only `std` files have **no v2 counterpart** — they are authority, not fork.

---

## 1. The blocker — cross-tree import is wired but switched off

The machinery to import `dsl/` from `src/v2/` landed (cross-tree import model + resolution wiring,
proven by `src/v2/test/claim/name_resolve_cross_tree_resolution_test.dag`). It is **fail-closed by
default** and must stay that way until grounded:

- `src/v2/compiler/03_name_resolve.dag:644` — the real compile entry `resolve_with_admission`
  hardcodes `order: FundamentalityUnknown`.
- `src/v2/std/cross_tree/import_model.dag` — `FundamentalityUnknown → CrossTreeDenied`. So every
  real cross-tree import is denied today. The green witness only passes because it *explicitly*
  supplies `MoreFundamental`, which no real compile does.
- `src/v2/std/cross_tree/resolution.dag` header — **hard ACTIVATION GATE**: supplying a non-Unknown
  order for real compilation MUST NOT land until grounded `source_root` tagging replaces the current
  QualifiedName-prefix fallback (relying on a `v2.`-prefix to guess a file's tree is a DESIGN §4
  violation).

**Activation prerequisite (the one thing standing between "wired" and "on"):** tag each file with
which `--source-root` it came from at ingest. Dissolve-on named in the resolution header: add
`source_root: SourceRootRef` to `DagSourceReadWitness` (`src/v2/compiler/source_authority.dag`); the
host tags by source-root; then delete the QN-prefix fallback and flip the real-compile order.

Separate concrete blocker for the first real cross-tree *data* import:
`src/v2/std/probe_selector.dag:52` — v2 cannot import `dsl/product/compute_fabric` (`Option<T>` vs
`Optional<T>`; `std.*` namespace collision under dual source-root). Repro:
`dsl/test/claim/probe_selector_compute_fabric_import_repro_test.dag`.

---

## 2. Fork census (the 11 overlapping `std` basenames)

Classification by reading both files. **Collapse** = one is a copy/nickname, delete it and re-point.
**Decide** = same name, genuinely different job (different layer or concern) → rename to disambiguate
*or* merge, per concept. **Not-a-fork** = different concept, the shared name is the only collision.

| concept | dsl/std | src/v2/std | verdict |
|---|---|---|---|
| **algebra** | `Magma/Semigroup/Monoid<T>` … | byte-similar `Magma/Semigroup/Monoid<T>` … | **Collapse** — clearest historical duplicate |
| **logic** | `Classical = True \| False` | `Bool = True \| False` (+ Bool*Fact) | **Collapse** — nickname (`Classical`↔`Bool`); pick one name |
| **nat** | `Nat = CommutativeSemiring<Magnitude>` | `type Nat` + `nat_semiring: CommutativeSemiring<Nat>` | **Collapse** — same semiring concept, two encodings |
| **reducible** | `ReduceVerdict` + combine | header: "ported from dsl/std/reducible.dag" | **Collapse** — declared port |
| **measure** | carrier authority | header: "MIRROR … delete when v2 loads dsl/std directly" | **Collapse** — declared mirror |
| **integer** | `Int8 = Compose<Int, MachineWidth<8>>` … (width surface) | `Int = GroupCompletion<Nat>`, `UInt = Nat` (algebraic) | **Decide** — different layers of one tower; merge, don't copy-delete |
| **effects** | `EffectShape/CreateCause/KeySource` | `IdempotentShape/BreakingShape/…` | **Decide** — overlapping, diverged feature sets |
| **float** | `Real = ApproximateField<…>` + `Ieee754Float` | `Float32Interchange = Word32` (bits) | **Decide** — algebraic vs bit-level layer |
| **coercion** | `TypeCheckpoint/InhabitantDecl` (IR schema) | `FindWitnessRejectionKind/CoercionQuality` (witness results) | **Not-a-fork** — different concern; rename to stop name collision |
| **node** | `compiler_inductive_fields` (metadata *about* Node) | `Symbol/OccurrenceId` (*is* the Node substrate) | **Not-a-fork** — rename to stop name collision |
| **verification** | `TestClaim/AssertKind` (test-data model) | `TestgenTier/TestClassification` (testgen metadata) | **Not-a-fork** — rename to stop name collision |

Note: `node` and `verification` v2 copies are partly `🟡 gated — feature:coproduct-variant-enumeration`,
not pure forks.

---

## 3. Sequencing

1. **Activate cross-tree import** (§1 prerequisite). Grounded `source_root` tagging at ingest, then
   flip the real-compile order off `FundamentalityUnknown`. *Touches load-bearing pipeline files
   (`source_authority.dag`, `03_name_resolve.dag`) — escalate before editing.* Nothing below is safe
   or easy until this lands.
2. **Collapse the five declared duplicates** — `reducible`, `measure` (already self-describe as
   port/mirror), then `algebra`, `logic` (`Classical`→`Bool`), `nat`. Delete the v2 copy, re-point
   imports to `dsl/std/*`. Each must stay green by execution (the existing witnesses are the oracle).
3. **Resolve the same-name/different-job pairs** — `integer`, `effects`, `float` (merge the layers),
   and rename the not-a-forks (`coercion`, `node`, `verification`) so one name never denotes two
   concepts. This is the naming-hygiene goal: module/folder names reflect one authority.
4. **Unblock the first data import** — `Option`/`Optional` + `std.*` namespace collision
   (`probe_selector.dag:52`) so `dsl/product/compute_fabric` imports cleanly into v2.

---

## 4. Dissolution trigger (DESIGN §6)

Delete this doc when the fork census reaches zero — when no `std` basename exists in both
`dsl/std` and `src/v2/std` for a concept with a single authority, and the cross-tree-import
activation gate in `src/v2/std/cross_tree/resolution.dag` has dissolved. At that point the carriers
(absent files, re-pointed imports) tell the whole story and this audit is redundant.
