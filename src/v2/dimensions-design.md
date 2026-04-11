> Part of: [THESIS.md](../../THESIS.md) > **Correctness dimensions**
> See also: [cx-design.md](cx-design.md), [ownership-design.md](ownership-design.md)

# Correctness Dimensions: General Mechanism

This document abstracts the general pattern from the CX and
ownership implementations. Every correctness dimension — built-in
or user-defined — should follow this architecture.

---

## The pattern

Every correctness dimension is the same four things:

```
1. DECLARE    a lattice in std/         (the vocabulary)
2. COMPUTE    at each binding site      (the producer)
3. CARRY      through the IR            (the transport)
4. ENFORCE    at consumption points     (the consumer)
```

No separate analysis passes. No reconstruction heuristics. The
compiler's inference pass is the single producer. Everything
downstream is a consumer.

---

## 1. DECLARE: lattice in std/

A dimension is a type that inhabits `BoundedLattice<T>`:

```dag
type D {
  // The dimension's value space — what it tracks
}

inhabits BoundedLattice<D> {
  bottom: ...   // most restrictive / least information
  top: ...      // most permissive / full information
  meet: ...     // conservative merge (weaker wins)
  join: ...     // optimistic merge (stronger wins)
}
```

### Current dimensions

| Dimension | Type | Bottom | Top | Meet semantics |
|-----------|------|--------|-----|---------------|
| Provenance | `SubValueRelation` | `SubValueUnknown` | `StrictSubValue` | Weaker evidence wins |
| Descent evidence | `DescentEvidence` | `DescentUnknown` | `Strict` | Less-proven wins |
| Ownership (future) | `OwnershipKind` | `Shared` | `Owned` | More-shared wins |
| Side effects (future) | `EffectLevel` | `Pure` | `WritesExternal` | More-effectful wins |

### User-defined example

```dag
type SecurityLevel = Public | Internal | Confidential | Secret

inhabits BoundedLattice<SecurityLevel> {
  bottom: Public
  top: Secret
  meet: min_security    // at join points, take the lower clearance
  join: max_security    // for required clearance, take the higher
}
```

### What the declaration provides

The lattice structure gives the compiler everything it needs to
carry the dimension through arbitrary control flow:

- **If/match merging**: `meet(branch_a, branch_b)` — conservative
- **Sequential composition**: value from last expression in block
- **Function call**: read callee contract or fail-closed to bottom
- **Lambda boundary**: read callback contract or fail-closed

The compiler doesn't need per-dimension merge logic. It reads the
lattice operations from the declaration.

---

## 2. COMPUTE: at each binding site

When inference creates a TypeBinding, it computes the dimension
value. There are 7 binding-creation sites (documented in
cx-design.md §Binding-site audit). Each site has a rule per
dimension:

| Binding site | Provenance rule | Ownership rule (future) | Effect rule (future) |
|---|---|---|---|
| **Function parameter** | `PreservedValue` | `Owned` (caller's value) | `Pure` (no effect yet) |
| **Let-binding** | Derive from value expr | Derive from value expr | Derive from value expr |
| **Match arm binding** | Compose with scrutinee | Inherit from scrutinee | Inherit from scrutinee |
| **Lambda param (collection)** | `IteratedSubValue` | Element ownership | Inherit from collection |
| **Lambda param (callable)** | From callee contract | From callee contract | From callee contract |
| **Lambda param (no context)** | `SubValueUnknown` (bottom) | `Shared` (bottom) | `Pure` (bottom) |
| **For-each variable** | `IteratedSubValue` | Element ownership | Inherit from collection |

### The general rule

For each binding site:
1. Look up the source binding's dimension value from scope
2. Apply the site-specific composition (field access, iteration,
   arithmetic, construction)
3. Store the result on the new binding

If the composition can't be determined: **fail-closed to bottom.**
Bottom is always safe (most restrictive). The compiler never
approximates upward.

### Abstracting from CX

CX currently does this with `classify_binding_provenance` and
`derive_field_provenance` — two functions that compute
SubValueRelation at binding sites. The generalization: these
become one generic function parameterized by the dimension:

```
fn compute_dimension_at_binding<D: BoundedLattice>(
  site: BindingSite,
  source_value: D,
  access: AccessShape,
) -> D {
  match access {
    FieldAccess { field } => compose_field(source_value, field)
    Iteration { element } => compose_iteration(source_value, element)
    Arithmetic { op, by } => compose_arithmetic(source_value, op, by)
    Construction          => D.bottom  // new value, no relation
    Unknown               => D.bottom  // fail-closed
  }
}
```

Each dimension provides `compose_field`, `compose_iteration`, etc.
as part of its declaration. Dimensions that don't care about a
particular access shape return `D.bottom` (fail-closed).

### Abstracting from ownership

Ownership currently runs as a separate pass that re-walks function
bodies counting consumers. Under this architecture:

- `OwnershipKind` (Owned/Shared/Moved) is computed at each binding
  site based on how many consumers the binding has
- The "separate pass" dissolves — consumer counting happens during
  inference when expressions reference bindings
- `SharedError` becomes: "binding has OwnershipKind = Shared but
  is used in a consuming position"

The reconstruction heuristics (fold detection by string name,
accumulator detection by terminal name, field move collection by
AST re-walk) all dissolve because the provenance dimension already
tells the ownership dimension what it needs to know.

---

## 3. CARRY: through the IR

Dimension values live on bindings. Two options:

### Option A: Fields on TypeBinding

```dag
type TypeBinding {
  name: String
  resolved: Node
  provenance: SubValueRelation    // dimension 1
  ownership: OwnershipKind        // dimension 2
  effects: EffectLevel            // dimension 3
  // ... grows with each dimension
}
```

**Pros:** Simple, direct access, no indirection.
**Cons:** TypeBinding grows with each dimension. Violates "types
proportional to common case" if most bindings don't need most
dimensions.

### Option B: Compiler-external tables (preferred)

```dag
// TypeBinding stays lean:
type TypeBinding {
  name: String
  resolved: Node
}

// Dimensions live in compiler tables keyed by binding identity:
type DimensionTable<D> = Map<BindingId, D>

// The compiler carries one table per declared dimension:
provenance_table: DimensionTable<SubValueRelation>
ownership_table: DimensionTable<OwnershipKind>
effects_table: DimensionTable<EffectLevel>
```

**Pros:** TypeBinding doesn't grow. Each dimension is independent.
User-defined dimensions don't require TypeBinding changes.
Aligns with `feedback_node_not_god_struct` — compiler state stays
off core types.
**Cons:** Indirection (table lookup vs. field access). Requires
binding identity (currently string name).

### Current state: hybrid

TypeBinding currently has `provenance: SubValueRelation` (Option A
for one dimension). The general architecture should move toward
Option B, but Option A is acceptable for the first few built-in
dimensions. The transition:

1. **Now:** provenance on TypeBinding directly (done, S1-S5)
2. **Next:** ownership as a second field on TypeBinding (simple)
3. **Later:** extract to tables when user-defined dimensions arrive
   (needs binding identity — connects to InternTable work, Track 3)

---

## 4. ENFORCE: at consumption points

Downstream consumers read dimension values and act on them.
Enforcement is dimension-specific but follows a common pattern:

### Patterns of enforcement

**Gate (compile-time rejection):**
- CX: if any function has `SubValueUnknown` on a recursive call
  argument → `CostUnknown` → diagnostic (future: blocking)
- Ownership: if a binding has multiple consumers → `SharedError`
  → blocking diagnostic
- Security (user): if Secret data flows to Public drain → blocking
  diagnostic

**Optimization (emit-time decision):**
- Ownership: if binding fan-out = 1 → emit move; if > 1 → emit
  clone. Read from dimension table.
- Purity: if function is pure → eligible for memoization,
  parallelism

**Reporting (non-blocking information):**
- CX: complexity report with proven bounds per function
- Space: peak memory estimate per function

### Generic enforcement interface

Each dimension declares its enforcement rule:

```dag
type DimensionEnforcement<D> {
  gate: fn(value: D, context: EnforcementContext) -> Diagnostic?
  // Returns Some(diagnostic) if the value violates the dimension
  // at this consumption point. None if OK.
}
```

The compiler iterates over all declared dimensions at each
enforcement point and collects diagnostics. No per-dimension
wiring in the pipeline.

---

## Composition across function boundaries

The hardest part. When function `f` calls function `g`, how do
dimension values compose?

### Callee contracts (the general solution)

A function declares the relationship between its inputs and
outputs for each dimension:

```dag
type DimensionContract<D> {
  // For each output/callback parameter, what dimension value
  // does it have relative to the inputs?
  param_relations: List<ParamRelation<D>>
}

type ParamRelation<D> {
  output_param: Int       // which output/callback param
  source_param: Int       // which input param it derives from
  relation: D             // the dimension value of the derivation
}
```

For provenance, this is CallbackContract (cx-design.md §Lambda
provenance): fold's callback element is `IteratedSubValue` of
the collection parameter.

For ownership, this would be: fold's callback receives a borrowed
reference to the accumulator, not ownership.

For effects, this would be: fold's callback inherits the effect
level of the fold itself.

### Standard library contracts

Collection methods (fold, map, filter, descend) get contracts
declared in std/ via `AlgebraMethodSemantics`:

| Method | Provenance contract | Ownership contract | Effect contract |
|--------|---|---|---|
| `fold(coll, init, f)` | f's element = IteratedSubValue of coll | f borrows acc, owns element | f inherits fold's effect |
| `map(coll, f)` | f's element = IteratedSubValue of coll | f owns element | f inherits map's effect |
| `descend(tree, f)` | f's node = StrictSubValue of tree | f borrows node | f inherits descend's effect |

### User-defined function contracts

For non-std functions, two options:
1. **Infer the contract** — analyze the function body (decidable
   in a closed system). Compute once, cache on function signature.
2. **Fail-closed** — unknown contract → bottom for all dimensions.
   Safe but conservative.

Option 1 is the long-term target. Option 2 is the bootstrap.

---

## Migration path

### Phase 1: Provenance (current — CX)

SubValueRelation on TypeBinding. S1-S5 done. C2-C6 switch CX
to read it. Single dimension, field on TypeBinding.

### Phase 2: Ownership on bindings

Add OwnershipKind to TypeBinding (or external table). Compute
at binding sites during inference. Dissolve the separate
ownership pass. SharedError becomes a gate check on the
dimension value.

### Phase 3: Effects

Declare EffectLevel in std/behavioral.dag with lattice. Compute
at binding sites (function calls mark effectful, pure code stays
pure). Carry through bindings. Enforce: effectful code can't
appear in pure context.

### Phase 4: Generic dimension carrier

Extract the pattern into a general mechanism. TypeBinding carries
a dimension table (or external tables keyed by binding ID).
The compiler discovers dimensions from std/ declarations. Adding
a dimension = adding a .dag file.

### Phase 5: User-defined dimensions

Users declare lattices in their own modules. The compiler carries
them with the same mechanism. No compiler changes needed.

---

## Design questions (open)

1. **Binding identity.** Tables keyed by binding identity need
   stable IDs. Currently bindings are identified by string name.
   InternTable (Track 3, PR #367) provides `ident: Int` but it's
   not wired through TypeBinding yet. This is a prerequisite for
   Option B (external tables).

2. **Dimension interaction.** Are dimensions truly orthogonal, or
   do some interact? Example: provenance and ownership interact —
   if a binding is StrictSubValue (provenance) and the sub-value
   is the only consumer (ownership), the compiler can move instead
   of clone. The interaction is at the enforcement level, not the
   carrying level — each dimension is computed independently, but
   enforcement can read multiple dimensions.

3. **Performance.** Each dimension adds O(1) work per binding site.
   With N dimensions, total overhead is O(N × bindings). For
   built-in dimensions (3-5), this is negligible. For user-defined
   dimensions, need to verify that the lattice operations are
   bounded (they are, in a closed system — all lattices are finite).

4. **Composition rule expressiveness.** The current composition
   rules (field access, iteration, arithmetic, construction) are
   sufficient for provenance and ownership. User-defined dimensions
   may need custom composition rules. The declaration interface
   must be expressive enough without being Turing-complete
   (decidability invariant).

---

## Relationship to other docs

- **THESIS.md §Correctness dimensions** — the high-level principle
  (orthogonal, non-consensual, lattice-based)
- **cx-design.md** — the first dimension (provenance/termination),
  including binding-site audit, composition algebra, and migration
  plan
- **ownership-design.md** — the second dimension (ownership),
  including shared infrastructure with CX
- **INVARIANTS.md** — bounded kernel (dimensions don't add
  recursion), fail-closed (bottom on unknown), single authority
  (one declaration per dimension)
