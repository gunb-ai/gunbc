# v4 SG-COLLECTION-PROJECTION Worksheet — FreeMonoid boundary projection

> **Status:** WORKSHEET DRAFT — routed from Rust RCA residual catalog 2026-06-01.
> **Date:** 2026-06-01
> **Dispatch anchor:** `docs/audit/v4-rustc-error-catalog-2026-06-01-post-jun1-cascade.md` §4 `E0308 / collection projection`; `docs/planning/v4-predicate-dependency-graph-2026-06-01-eod.md` §6 row 6.
> **Canonical home:** extend `src/v4/std/target_model.dag` `TargetCollectionRealization` for sequence carriers; Rust realization rows live in `src/v4/extdeps/languages/rust.dag`; consumer is `src/v4/compiler/06_translate.dag`.
> **Shape:** one systemic fix, not a Rust template patch.

---

## Mechanical dispatch rule

No SG-COLLECTION-PROJECTION implementation worker may land by changing emitted Rust call sites, inserting `collect()`/`into_iter()` locally, or hard-coding `Vec<Rc<T>>` in a compiler helper. The worker first consumes the single-authority target collection realization row named below.

Acceptance is falsification-probe behavior at the collection boundary, not raw rustc error-count reduction.

---

## §10.0-adapted worksheet

```text
SG class: SG-COLLECTION-PROJECTION
Representative emitted failure:
  expected Vec<Rc<Edge>>, found FreeMonoid<_>
  expected Vec<Rc<PrimitiveFactBundle>>, found FreeMonoid<_>
  expected Vec<Rc<FormalProduction>>, found FreeMonoid<_>
  expected Vec<Rc<Node>>, found FreeMonoid<_>

Immediate local patch:
  - Teach each failing constructor/return/field site to spell `.iter().map(Rc::new).collect()`,
    or special-case FreeMonoid in the Rust serializer as `Vec<Rc<T>>`.

Why that patch is forbidden:
  - It creates a second authority for the same boundary fact (INVARIANTS.md P2).
  - It mixes two decisions: source collection semantics (`FreeMonoid<T>`) and Rust storage
    at a use-site boundary (`Vec<Rc<T>>`).
  - It bypasses SG-RC-LAYERING: `Rc<T>` is a use-site ownership realization, not a property
    of FreeMonoid itself.

DFS path:
  std/ authority:
    - `src/v4/std/algebra.dag` defines FreeMonoid semantics.
    - `src/v4/std/target_model.dag` already owns target collection realization for SG-5 Set.
    - `src/v4/std/target_model.dag` already owns target ownership/use-site realization for
      SG-RC-LAYERING.
  extdeps/language authority:
    - `src/v4/extdeps/languages/rust.dag` already declares Rust target model rows, collection
      spelling tokens, and ownership rows for `target_carrier_free_monoid`.
  compiler stage consuming it:
    - `src/v4/compiler/06_translate.dag` projects type expressions and applies use-site
      ownership rows.
  existing scaffold/dissolution notes:
    - SG-5 collection realization is Set-only today.
    - SG-RC-LAYERING owns raw/Rc/Box use-site decisions.

Deepest unsound boundary:
  The current model has no row saying that a source `FreeMonoid<T>` sequence at a Rust consumer
  boundary realizes as the target collection storage form `Vec<...>`, with the element shell
  independently projected through ownership at the element-storage site.

Systemic fix:
  Extend `TargetCollectionRealization` so the catalog can carry multiple source-carrier rows
  and a sequence representation kind, then add the Rust row:

    source_carrier: target_carrier_free_monoid_node()
    primary.form.kind: TargetCollectionReprVec
    primary.form.apply: Rust `Vec<...>` generic apply tokens
    element projection: consume target ownership/use-site rows for the element shell, yielding
      `Rc<T>` where the target model says collection storage owns references by `Rc`
    alternatives: []
    fallback_diagnostic: typed no-representation diagnostic for FreeMonoid sequence projection

  The final Rust spelling `Vec<Rc<T>>` is therefore a composition of two single-authority facts:
    1. collection source carrier `FreeMonoid<T>` -> target storage family `Vec<_>`
    2. element use-site ownership -> `Rc<T>` when Rust storage requires shared references

Non-goals:
  - Redefining FreeMonoid as Vec in std/.
  - Treating `Rc` as part of the FreeMonoid collection carrier.
  - Folding this into SG-5 Set/BoundedLattice acceptance.
  - Per-type exceptions for Edge, PrimitiveFactBundle, FormalProduction, Node, or similar
    constructors.

Falsification probes:
  - `FreeMonoid<Edge>` projected at a Rust struct-field/constructor boundary emits
    `Vec<Rc<Edge>>` only by consuming the collection row plus ownership row.
  - Removing the FreeMonoid collection row produces a typed collection-realization diagnostic,
    not a fallback to raw `FreeMonoid<_>` and not an emitter guess.
  - Removing the ownership row leaves the collection family known (`Vec<_>`) but fails closed
    at the element ownership projection, proving the two authorities are separate.

Metric allowed only as secondary:
  The catalog's ~170 E0308 FreeMonoid/Vec errors should shrink, but the acceptance gate is
  the typed falsification behavior above.
```

---

## Worker brief

```text
Implement SG-COLLECTION-PROJECTION per this worksheet.

MUST:
  - Extend the existing TargetCollectionRealization authority instead of adding a parallel
    FreeMonoid-specific table.
  - Add a Rust FreeMonoid sequence row in `src/v4/extdeps/languages/rust.dag`.
  - Refactor `06_translate` to select the FreeMonoid row and apply element use-site ownership
    through the SG-RC-LAYERING catalog.
  - Land manual falsification claims covering row-present, row-missing, and ownership-row-missing
    behavior.

MUST NOT:
  - Patch emitted Rust sites individually.
  - Spell `Vec<Rc<T>>` as a compiler-local template outside the target model row + ownership row
    composition.
  - Change FreeMonoid semantics or make Vec the std/ authority for FreeMonoid.
```

---

## Related artifacts

- `docs/audit/v4-rustc-error-catalog-2026-06-01-post-jun1-cascade.md`
- `docs/planning/v4-predicate-dependency-graph-2026-06-01-eod.md`
- `docs/planning/v4-sg5-sg6-collection-bounded-lattice-worksheet-2026-05-30.md`
- `docs/planning/v4-sg-rc-layering-worksheet-2026-05-31.md`
