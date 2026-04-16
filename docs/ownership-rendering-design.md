# Ownership Rendering Design

> **Parent docs:** `THESIS.md` (causal engine), `INVARIANTS.md`
> §"Facts Flow Forward" (FF-1, FF-8), `src/v3/SELF_HOSTING.md`
> §14.7 (ownership track).
>
> **Purpose:** design the ownership rendering model for v3's
> emitter so that generated code has zero unnecessary clones.
>
> **Status:** design complete (2026-04-16). All questions resolved.

---

## §1. The problem

v3's emitter currently inserts `.clone()` on every port
reference. The first generated artifact — 287 lines of
`lens_unused_parameters_generated.rs` — contains 90 clone
calls. Every one is a READ (function parameter, match
scrutinee, field access, comparison). No consumer in this
lens needs ownership — they all just look at the value.

This is the v2 pattern that caused 20-minute self-compiles
(FF-1) and O(n) container clone costs (FF-8).

---

## §2. The key insight: read vs construct

In a pure language (all values immutable, all functions pure),
there are exactly two things a consumer does with a value:

1. **Read** — inspect it, match on it, access a field, pass
   it to a function, compare it. The value is unchanged.
2. **Construct** — use it as a field in a new record, an
   element in a new list, or a return value. A new value is
   created that contains this one.

This is the PRIMARY ownership dimension. Everything else
(fan-out, escape analysis, container cost) is secondary.

### §2.1 Why read vs construct is primary

In a pure language, ALL function parameters are reads. The
emitter currently generates everything as owned (`T`) when it
should generate reads as borrowed (`&T`). That's why the
generated lens code has 90 clones — every parameter is passed
by value, forcing clones at every call site.

Fan-out (how many consumers) is NOT the right primary
dimension. A value with fan-out=5 where all 5 consumers are
reads needs ZERO clones — just pass `&T` to each. A value
with fan-out=1 where the consumer constructs a record needs
zero clones too — just move the value into the record.
Fan-out only matters as a modifier at construction sites
(see §2.4).

### §2.2 Classifying use kind from the DAG

The consumer's behavior type determines use kind:

| Consumer behavior | Use kind | Why |
|---|---|---|
| Transform input (function call argument) | Read | Pure function reads its arguments |
| Branch scrutinee | Read | Inspects value to choose a path |
| Match pattern destructure | Read | Reads variant tag + fields |
| Field access (FieldProject) | Read | Projects one field |
| Comparison operand | Read | Inspects value |
| Record literal field | **Construct** | Value becomes part of a new record |
| List cons element | **Construct** | Value becomes part of a new list node |
| Function return value | **Construct** | Value becomes the function's output |

The emitter already knows which case it's in — it's implicit
in the emission walk (rendering a function call vs rendering a
record literal). No separate analysis pass needed for the
common case.

### §2.3 Target rendering for read vs construct

Each target language declares how to render reads and
constructions:

```dag
type RenderingModel {
  read: ReadStrategy
  construct: ConstructStrategy
}

type ReadStrategy
  = Borrow           // Rust: &T
  | PassByValue      // Go/Python: just pass T (GC handles sharing)

type ConstructStrategy
  = CopyOrClone      // Rust: copy if Copy type, else clone
  | PassByValue      // Go/Python: just pass T
```

Target declarations:

```dag
// rust.dag
data rust_rendering: RenderingModel = {
  read: Borrow
  construct: CopyOrClone
}

// go.dag
data go_rendering: RenderingModel = {
  read: PassByValue
  construct: PassByValue
}
```

### §2.4 When fan-out matters (construction sites only)

Fan-out is secondary. It only applies at construction sites
in Rust:

- **Last use at a construction site** → move (zero cost).
  The value transfers into the new record/list/return.
- **Non-last use at a construction site** → clone (O(size)
  for non-Copy types, O(1) for Copy types).

"Last use" is a modeled fact: **is this the last consumer of
this port in evaluation order?** The emitter computes this
during its walk by tracking which ports have remaining
consumers after the current render point.

For reads, fan-out is irrelevant — borrow is always zero
cost regardless of how many readers exist.

### §2.5 Copy types

Some target-language types are trivially copyable (Rust's
`Copy` trait: integers, booleans, references). For these,
construction doesn't need a clone — the bits are copied.

The realization spec declares which types are Copy:

```dag
// In rust.dag, on each TypeRealization:
data rust_int: TypeRealization = {
  target: Int
  carrier: "i64"
  is_copy: true       // i64 implements Copy
}

data rust_source_span: TypeRealization = {
  target: SourceSpan
  carrier: "SourceSpan"
  is_copy: false      // contains String, not Copy
}
```

Construction rendering for Rust:
- `is_copy: true` → just use the value (compiler copies bits)
- `is_copy: false` + last use → move
- `is_copy: false` + not last use → `.clone()`

---

## §3. The composition

```
render(port, consumer) =
  let use_kind  = classify_use(consumer)         -- §2.2
  let strategy  = target.rendering_model         -- §2.3
  in
    match use_kind {
      Read      → strategy.read                  -- &T in Rust
      Construct → apply_construct_strategy(
                    strategy.construct,
                    is_copy(type_of(port)),       -- §2.5
                    is_last_use(port, consumer))  -- §2.4
    }
```

For Rust, this expands to:

| Use kind | is_copy | Last use? | Rendering | Cost |
|----------|---------|-----------|-----------|------|
| Read | any | any | `&value` | 0 |
| Construct | true | any | `value` | 0 (bits copied) |
| Construct | false | yes | `value` | 0 (moved) |
| Construct | false | no | `value.clone()` | O(size) |

For Go: always `value`. No distinctions needed.

**Expected impact on generated lens code:**
- ALL function parameters → `&T` → zero clones at call sites
- ALL match scrutinees → `match &value` → zero clones
- ALL field accesses → `&value.field` → zero clones
- Record construction with Copy fields → zero clones
- Record construction with non-Copy fields → clone only those
- For `unused_parameters.dag`: `NodeId` (Copy), `PortId`
  (Copy), `i64` (Copy), `SourceSpan` (not Copy, contains
  String) → ~5 clones total, all at `UnusedParameter` record
  construction sites.

---

## §4. What v2 reconstructed vs what v3 declares

| Fact | v2 (719 lines in ownership.dag) | v3 |
|---|---|---|
| Immutability | Implicit assumption | Declared in std/ (`SourceMutability = Immutable`) |
| Use kind (read vs construct) | Not modeled — v2 doesn't distinguish | Primary dimension (§2.2), read from behavior type |
| Fan-out | 200+ lines of ExprData tree walking | Structural in DAG, secondary to use-kind |
| Binding kind | 100+ lines of VarBindingKind threading | Dissolved — port state + behavior type carry this |
| Fold linearity | 150+ lines of fold body analysis | Dissolved — count consumer NODES not references (§5 Q1) |
| Lambda capture | 50+ lines of body double-counting | Dissolved — captures are explicit DAG edges (§5 Q2) |
| Copy type classification | Not modeled (Rc/clone heuristics) | Declared per-type in realization spec (`is_copy` field) |
| Last-use tracking | Not modeled (clone everything) | Computed during emission walk |
| Target strategy | Hardcoded in emitter | Declared in rust.dag (`RenderingModel`) |

**v2's 719 lines dissolved to:** one classification (read vs
construct from behavior type), one per-type fact (`is_copy`),
one walk-time computation (last use), and one target
declaration (`RenderingModel`). No separate ownership lens
needed for the common case.

---

## §5. Resolved design questions

### Q1: Fold accumulator linearity — DISSOLVED

Count consumer NODES, not port-id references. A Loop node
that references the same port in `source`, `init`, and
`bound.count` is ONE consumer. No fold-specific analysis
needed.

### Q2: Lambda capture sharing — DISSOLVED

Captures are explicit DAG edges. Consumer count includes
inner behaviors naturally. No lambda-specific logic needed.

### Q3: Target strategy schema — RESOLVED

`RenderingModel { read: ReadStrategy, construct: ConstructStrategy }`
with `is_copy` per-type in the realization spec. See §2.3,
§2.5.

### Q4: Immutability scope — RESOLVED

Per-language for .dag native code (`SourceMutability =
Immutable`). Per-declaration for external language ingestion
(future, L3+).

### Q5: Complexity lens interaction — RESOLVED

Complexity lens reads the ownership pipeline's output. If
ownership says "move" (zero cost), complexity sees zero cost.
If "clone" (O(n)), complexity sees O(n). No circular
dependency.

---

## §6. Phasing

| Phase | When | Deliverable |
|---|---|---|
| **Phase 1** | L1.5 | 1. Declare `SourceMutability = Immutable` in `std/values.dag`. 2. Add `is_copy: Bool` to TypeRealization declarations in `rust.dag`. 3. Add `RenderingModel` to `rust.dag`. 4. Emitter generates `&T` parameters for all reads, clones only at construction sites for non-Copy types. 5. Clone-count test: exact count on generated lens, expect ~5 (only SourceSpan construction sites). |
| **Phase 2** | L2 | 1. Last-use tracking during emission walk (currently all constructions clone; with last-use, the final construction moves instead). 2. Expected clone count drops to ~2-3 (only non-last constructions of SourceSpan). |
| **Phase 3** | L3 | Self-analysis: ownership pipeline runs on generated compiler code. Clone count at or near zero on all artifacts. |

**What dissolved.** v2's 719-line ownership.dag is not ported.
The read-vs-construct model replaces it. The Rust-specific
concerns (is_copy, last-use) live in the realization spec and
emitter walk, not in a 719-line lens.

---

## §7. Testing approach

**Tests should verify reads are borrows and constructions
are the only clone sites.**

Core fixtures:

1. **Read-only function** — all parameters read, zero clones.
   `fn f(a: Int, b: Int) -> Int = a + b` → emitted Rust has
   `fn f(a: &i64, b: &i64) -> i64` with zero `.clone()`.

2. **Record construction with Copy fields** — zero clones.
   `let p: Point = { x: 1, y: 2 }` → no `.clone()` (Int is
   Copy).

3. **Record construction with non-Copy field** — exactly one
   clone per non-Copy field.
   `let r: Result = { value: some_string, code: 0 }` →
   `some_string.clone()` (String is not Copy), `0` (no clone,
   Int is Copy).

4. **Multiple reads of same value** — zero clones regardless
   of fan-out.
   `fn f(x: Dag) -> Int = count(x.nodes) + count(x.ports)` →
   `x` borrowed twice, zero clones.

5. **Generated lens clone count** — exact count on
   `lens_unused_parameters_generated.rs`, pinned at ~5.
   The ratchet only goes down.

Property tests:

6. **No clone at any read site** — scan emitted Rust for
   `.clone()` calls; every one must be at a record literal,
   list construction, or return expression, never at a
   function argument or match scrutinee.

7. **Every non-Copy construction has a clone** — scan emitted
   Rust for record literal fields; every non-Copy field that
   isn't the last use must have `.clone()`.

---

## §8. When this doc updates

- Phase 1 lands → §6 graduates
- Clone count test pinned → §7 gets actual numbers
- Phase 2 lands → last-use tracking verified
- All phases land → doc archives
