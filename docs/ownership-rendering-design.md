# Ownership Rendering Design

> **Parent docs:** `THESIS.md` (causal engine), `INVARIANTS.md`
> §"Facts Flow Forward" (FF-1, FF-8), `src/v3/SELF_HOSTING.md`
> §14.7 (ownership track).
>
> **Purpose:** design the ownership rendering model for v3's
> emitter. The goal is to separate the independent facts that
> compose into a rendering decision, declare each fact at its
> natural authority, and let the correct target-language rendering
> emerge from the composition.
>
> **Status:** design phase. No implementation yet. This doc is
> the working surface for getting the model right before building.

---

## §1. The problem

v3's emitter currently inserts `.clone()` on every port
reference (emit_rust.rs lines 1415/1418). The first generated
artifact — 287 lines of `lens_unused_parameters_generated.rs` —
contains 90 clone calls. This is the v2 pattern that caused
20-minute self-compiles (FF-1) and O(n) container clone costs
(FF-8).

The naive fix is per-behavior-type ownership rules (6 cases).
The right fix is to identify the independent facts that compose
into the rendering decision and declare each one upstream.

**This is a first-class pipeline feature, not a lens.** The
test: would any emitter NOT want this information? No. Every
target language needs fan-out (dead code at fan-out=0), value
semantics (sharing safety), and escape analysis (lifetime
constraints). Lenses are optional analyses that some consumers
care about (complexity, effects). Ownership facts are like type
inference — always computed, always available, consumed by every
emitter. The types live in `std/`, the facts are computed as a
pipeline stage between inference and emission, and the
target-specific rendering strategy lives in the realization
spec.

---

## §2. Dimensions of the decision

When the emitter reaches a port reference, the rendering depends
on several independent facts. Each fact should be declared at its
natural authority — not reconstructed or assumed at render time.

### Dimension 1: Value semantics (immutability)

**The fact:** is this value immutable?

**Where it comes from:** the source language. `.dag` values are
immutable by construction — the language has no mutation
primitive. An ingested external language value might be mutable.

**Why it matters:** if a value is immutable, sharing it between
multiple consumers is always safe. No consumer can modify it
out from under another. This collapses the ownership question
from "who owns this?" to "how does the target share it?"

**Where to declare:** `std/values.dag` (or similar). The purity
and immutability guarantees of the `.dag` language should be
structural facts, not assumptions. An `ExternalRealization` that
ingests a mutable value carries a different fact.

**Resolved (Q4):** per-language for .dag native code (all values
immutable). Per-declaration for external language ingestion
(future work, L3+).

### Dimension 2: Consumer count (fan-out)

**The fact:** how many consumers reference this value?

**Where it comes from:** the DAG structure. Count the behaviors
that have this port as an input.

**Why it matters:** fan-out = 1 means the value has a single
consumer, so transfer is always safe (move in Rust, pass in Go).
Fan-out > 1 means multiple consumers need access — the target
language's sharing mechanism applies.

**Where to declare:** this is already structural in the DAG.
`produced_by` edges are the authority. No separate declaration
needed — the emitter counts them during its index-build pass.

**Resolved (Q1, Q2).** Count distinct **consumer nodes**, not
port-id references. A Loop node that references the same port
in `source`, `init`, and `bound.count` is ONE consumer. Lambda
captures are explicit edges — the captured port's consumer
count includes inner behaviors naturally. Both v2 edge cases
dissolve. See §5 Q1, Q2 for the full reasoning.

### Dimension 3: Target language sharing model

**The fact:** how does this target language share immutable
values between multiple consumers?

**Where it comes from:** the target language's memory model.
This is a fact about Rust/Go/Python/etc., not about `.dag`.

**Why it matters:** different targets have fundamentally
different answers:
- **Rust:** move (zero cost, single consumer), borrow (`&T`,
  zero cost, limited by lifetimes), reference-count (`Rc<T>`,
  O(1) share cost, no lifetime limits), clone (`.clone()`,
  O(size) cost, when all else fails)
- **Go:** pass (everything is GC'd, zero explicit sharing cost)
- **Python:** pass (reference counted, zero explicit sharing cost)

**Where to declare:** the target's realization spec
(`rust.dag`, `go.dag`, `python.dag`). The sharing strategy is
a property of the target language, alongside its type mappings
and operator syntax.

**Resolved (Q3).** The schema is a 2-3 field `SharingModel`
mapping consumer-count cases to a `Mechanism` enum. See §5 Q3.

### Dimension 4: Value lifetime / escape analysis

**The fact:** does this value outlive its producer's scope?

**Where it comes from:** the DAG's scope structure. A value
produced inside a function and returned escapes. A value
produced and consumed within the same scope doesn't.

**Why it matters:** this determines which sharing mechanisms are
available. In Rust:
- Non-escaping + single consumer → move
- Non-escaping + multiple consumers → borrow (`&T`)
- Escaping + multiple consumers → `Rc<T>` or clone
  (borrow can't outlive the scope)

**Where to declare:** this may be derivable from the DAG
structure (does the value flow to the function's return port?).
If so, no separate declaration needed — the emitter traces
the flow during its walk.

**Phasing.** Phase 1 skips escape analysis — uses the 2-field
SharingModel (single=Move, multi=Clone). Phase 2 adds escape
analysis by tracing whether a port's value flows to the
function's return port (a structural DAG walk). Phase 2 unlocks
the 3-field model where non-escaping multi-consumer ports use
Borrow instead of Clone. This is the difference between "most
clones eliminated" (Phase 1) and "all unnecessary clones
eliminated" (Phase 2).

### Dimension 5: Container representation

**The fact:** is this value a scalar or a container? If a
container, what's the element count?

**Where it comes from:** the type system. `Int` is a scalar.
`List<T>` is a container. The clone cost depends on this:
cloning an `Int` is O(1), cloning a `List<T>` is O(n).

**Why it matters:** for target languages with explicit copying
(Rust), the cost of cloning a container is much higher than
cloning a scalar. The sharing mechanism choice should account
for this: `Rc<Vec<T>>` (O(1) share) vs `Vec<T>.clone()` (O(n)
share).

**Where to declare:** the type's Cardinality in the substrate
already tells you this. Atom = scalar, collection = container.
The realization spec can declare per-type sharing preferences.

**Future refinement.** The realization spec could declare a
cost model for cloning (`clone_cost: O(1) | O(n) | O(n*m)`)
so the emitter can choose between Rc (O(1) share) and Clone
(O(size)) for containers. Not needed for Phase 1 — the
SharingModel handles it. Phase 2 can add cost-aware mechanism
selection if the clone ratchet shows container clones dominating.

---

## §3. The composition

The rendering decision for a port reference is a function of
all five dimensions:

```
render(port, consumer) =
  let semantics = value_semantics(port)          -- D1
  let fanout = consumer_count(port)              -- D2
  let strategy = target.sharing_strategy         -- D3
  let escapes = escapes_scope(port)              -- D4
  let cost = clone_cost(type_of(port))           -- D5
  in
    strategy.choose(semantics, fanout, escapes, cost)
```

Each dimension is an independent fact read from its authority.
The `strategy.choose` function is declared in the target's
realization spec — it's the target language's answer to "given
these facts, what's the cheapest correct rendering?"

For Rust, `choose` might be:

```
if fanout == 1 then Move
else if semantics == Immutable && !escapes then Borrow
else if cost == O(1) then Clone
else Rc
```

For Go: `choose _ = Pass` (always).

The key: no behavior-type-specific rules. No per-variant
ownership logic. The five facts compose into one decision via
the target's strategy, and the strategy is declared data.

**Pipeline position.** Ownership facts are computed as a
pipeline stage between inference and emission:

```
parse → lower → infer → ownership → emit
                         ↑
                    reads: types, ports, DAG edges
                    produces: per-port OwnershipFact
                    consumed by: every emitter
```

This is the same architectural position as inference — a stage
that enriches the DAG with derived facts before emission reads
them. Lenses (complexity, effects, etc.) run after ownership
and read its output as input. The pipeline ordering ensures
that complexity analysis accounts for real clone costs, not
conservative assumptions.

---

## §4. What v2 had to reconstruct vs what v3 can declare

| Fact | v2: reconstructed in ownership.dag | v3: declared where? |
|---|---|---|
| Immutability | Implicit (no mutation in .dag). Ownership.dag doesn't check it — just assumes. | Declare in std/. Make it a structural fact. |
| Fan-out | Computed by walking ExprData tree, counting name references. 200+ lines. | Read from DAG edges. Already structural. |
| Binding kind | Threaded through VarBindingKind (Local/MatchBound/Function/Variant). 100+ lines. | Port state + behavior type. Already structural. |
| Fold linearity | Reconstructed by walking fold body, checking accumulator use pattern. 150+ lines. | **Dissolved.** Count consumer NODES, not port references. Loop node = 1 consumer regardless of how many structural roles a port fills. |
| Lambda capture | Reconstructed by walking lambda body, double-counting outer names. 50+ lines. | **Dissolved.** Captures are explicit DAG edges. Consumer count includes them naturally. No special lambda logic. |
| Target strategy | Hardcoded in emitter (Rc vs clone vs move logic in emit_rust). | Declare in rust.dag. |

v2's 719 lines are mostly reconstruction of facts that were lost
between stages. v3's substrate carries more structural
information (explicit ports, explicit produced_by edges, explicit
behavior types), so much of the reconstruction dissolves.

The remaining open questions are fold linearity and lambda
capture — these are the cases where structural fan-out doesn't
match semantic fan-out. These need design attention.

---

## §5. Design questions — RESOLVED

### Q1: Fold accumulator linearity — RESOLVED

**Problem.** A naive port-reference count on the Loop node
overcounts. Currently `source`, `init`, and `bound.count` all
point to the same parameter port (lower.rs line 1762-1767).
That's structural fan-out ≥ 3 on one port, but the Loop
doesn't consume the value three times — it consumes it once
in three different roles (what to iterate, initial accumulator,
iteration bound).

**Resolution: count CONSUMERS, not REFERENCES.** A single
behavior node that references the same port in three structural
roles is one consumer, not three. The fan-out count should be:
"how many distinct behavior nodes consume this port?" not "how
many port-id fields in the DAG mention this port?"

Concretely: walk the DAG's behavior nodes. For each node, collect
the set of input ports (deduplicated). Each node contributes
+1 to each port's consumer count. The Loop node references
`param_ports[0]` in three fields, but it's ONE node — consumer
count = 1.

This dissolves Q1 entirely. The fold accumulator has consumer
count = 1 from the Loop node (regardless of how many structural
roles it fills) + whatever consumers exist in the body. The body
references the accumulator through its own behaviors, which are
separate nodes with their own consumer counts. No linearity
annotation needed. No special fold handling. The general rule
(count distinct consumer nodes, not port references) handles it.

**The v2 pattern this prevents.** v2's 150+ lines of fold
accumulator analysis existed because v2 counted NAME references,
not structural consumers. Names can appear multiple times in
source text; DAG nodes are distinct.

### Q2: Lambda capture sharing — RESOLVED

**Problem.** A lambda (Bind) that captures an outer port
increases that port's fan-out. Does this need special handling?

**Resolution: no.** A capture is just another consumer in the
DAG. The captured port appears as an input to behaviors inside
the Bind's body. Those behaviors are distinct nodes. The
consumer count for the captured port includes them naturally.

If a port is used once outside a lambda and once inside, it has
consumer count = 2. The target's sharing strategy handles
multi-consumer ports identically regardless of whether the
second consumer is "inside a lambda" or not. No special lambda
logic.

**The v2 pattern this prevents.** v2's 50+ lines of lambda
double-counting existed because v2 walked the AST and had to
manually discover that a name inside a lambda body was a capture.
v3's DAG has explicit port edges — the capture is structural.

### Q3: SharingStrategy schema — RESOLVED

The schema is minimal. For an immutable-value language, the
decision depends on: consumer count and whether the value
escapes its scope. The target declares what to do in each case:

```dag
type SharingModel {
  single_consumer: Mechanism
  multi_consumer_local: Mechanism
  multi_consumer_escaping: Mechanism
}

type Mechanism = Move | Borrow | RcWrap | Clone | Pass
```

Target declarations:

```dag
// rust.dag
data rust_sharing: SharingModel = {
  single_consumer: Move
  multi_consumer_local: Borrow
  multi_consumer_escaping: Clone
}

// go.dag
data go_sharing: SharingModel = {
  single_consumer: Pass
  multi_consumer_local: Pass
  multi_consumer_escaping: Pass
}
```

Three fields. Five mechanism options. The emitter reads the
model and applies it. No per-behavior-type logic. No hierarchy
to declare — the model directly maps the facts to the
mechanism.

**Phase 1 simplification.** Phase 1 can use a 2-field model
(single/multi) without escape analysis. Phase 2 adds the
third field (multi_escaping) and the escape analysis to
distinguish the two multi-consumer cases.

**Rendering syntax.** Each Mechanism variant has a known
rendering in the target language. The emitter knows how to
render `Move` in Rust (bare name), `Borrow` in Rust (`&name`),
`Clone` in Rust (`name.clone()`). These are a fixed set — not
per-target data. The Mechanism enum IS the abstraction layer
between the ownership model and the target's syntax.

### Q4: Value semantics declaration scope — RESOLVED

**For .dag native code: per-language.** All `.dag` values are
immutable by construction. This is declared once as a substrate
fact:

```dag
// std/values.dag
type SourceMutability = Immutable | Mutable

// The .dag language substrate declares:
data dag_value_semantics: SourceMutability = Immutable
```

The ownership pipeline reads this. If the substrate says
`Immutable`, sharing is always safe. If a future extension adds
mutability (or an ingested external language has mutable values),
the pipeline reads `Mutable` and uses a more conservative
strategy.

**For external language ingestion: per-declaration.** When
ingesting Python/Go/etc., each `ExternalRealization` declaration
carries its `SourceMutability`. Python `list` → `Mutable`. Python
`tuple` → `Immutable`. Go `struct` → depends on whether it has
pointer receivers.

This is future work (L3+). Phase 1-2 only deal with .dag native
code, so per-language `Immutable` is sufficient.

### Q5: Interaction with the complexity lens — RESOLVED

The complexity lens needs to know the cost of a clone to compute
accurate bounds. Because ownership is a first-class pipeline
fact (not an optional lens), the complexity lens simply reads
it as input — the same way it reads type information. No
circular dependency. Clone cost is a real cost; the complexity
lens accounts for it by reading the ownership pipeline's output.
If ownership says "move" (zero cost), complexity sees zero cost.
If ownership says "clone" (O(n)), complexity sees O(n).

---

## §6. Phasing

All design questions are resolved (§5). Implementation can
proceed.

| Phase | Depends on | Deliverable |
|---|---|---|
| **Phase 1** (L1.5) | Nothing — design is complete | 1. Declare `SourceMutability = Immutable` in `std/values.dag`. 2. Consumer-count index in emitter (count distinct consumer NODES per port, not references). 3. 2-field `SharingModel` in `rust.dag` (single=Move, multi=Clone). 4. Emitter reads consumer count + sharing model, renders accordingly. 5. Clone-count ratchet test on generated lens code. |
| **Phase 2** (L2) | Phase 1 | 1. Add escape analysis (does value flow to function return port?). 2. Extend SharingModel to 3 fields (add `multi_consumer_local: Borrow`). 3. Emitter uses borrow for non-escaping multi-consumer, clone for escaping. |
| **Phase 3** (L3) | Phase 2 | Self-analysis: ownership pipeline runs on generated compiler code. Clone-count ratchet at zero on all generated artifacts. |

**What dissolved.** v2's 719-line ownership.dag had ~200 lines
of fold linearity analysis and ~50 lines of lambda capture
handling. Both dissolve in v3 because Q1 and Q2 are resolved
by the "count consumer nodes, not references" rule. The
remaining v2 ownership work (edge classification, VarBindingKind
threading, try_unwrap proofs) is Rust-target-specific and lives
in the SharingModel's rendering logic — NOT in a 719-line lens.

**Phase 3 replaces the full v2 migration.** v2's ownership.dag
existed to reconstruct facts the upstream model didn't carry.
v3's model carries them structurally. Phase 3 is self-analysis
confirmation, not a port of v2 code.

---

## §7. Testing approach

**The roundtrip test for ownership:**

```
source program with known fan-out pattern
  → compile → emit to Rust
  → count .clone() calls in output
  → assert count matches expected (not "fewer than X" — exact)
```

**Test fixtures should cover each dimension independently:**

- D1: immutable value → no clone needed (vs hypothetical mutable → clone needed)
- D2: fan-out=1 → move, fan-out=2 → share/clone, fan-out=0 → dead code
- D3: same program emitted to Rust vs Go → Rust has sharing decisions, Go has none
- D4: escaping value → can't borrow (Rust-specific)
- D5: scalar clone O(1) vs container clone O(n) → different strategy

**The regression anchor:** for each generated artifact (lens
code, pipeline stage code), pin the clone count. The ratchet only
goes down. If a change increases the clone count, the ownership
model regressed.

---

## §8. When this doc updates

This doc evolves as:
- All design questions resolved as of 2026-04-16
- Phase 1 implementation starts → §6 phases get implementation
  details
- New dimensions discovered → add to §2
- v2 ownership.dag migration starts → cross-reference §4
