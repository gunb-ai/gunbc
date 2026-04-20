### Target realization efficiency

A direct consequence of the two-groundings distinction: **the
cost complexity of a concept is different at the `.dag` level
versus at the target level, and the compiler can compute both.**

**`.dag`-level complexity** is declared via CX (the complexity
dimension — see §"Correctness dimensions"). `Int.add` at the
algebra level is O(1) because it's a single field access on
`OrderedRing.add`. The CX claim is independent of any target.

**Target-level complexity** is declared in the language spec as
**realization cost** per primitive. Concrete examples:

- `rust.dag` declares: `Int64.add realizes as i64 + i64, cost
  O(1), ~1 machine instruction, ~1ns on commodity hardware`.
- `rust.dag` declares: `Int256.add realizes as carry-propagated
  quad-word addition, cost O(4), ~4 machine instructions`
  (assuming no native 256-bit hardware).
- `python.dag` declares: `Int.add realizes as Python int
  addition, cost O(1) amortized for small ints, O(log n) for big
  ints, with GC overhead`.
- `spice.dag` (if it exists as a language spec) declares: `Bool.and
  realizes as AND gate, cost O(1) gate delay`.

Target-level complexity **composes** with `.dag`-level CX. For a
`.dag` program that does `k × Int.add`, the target cost is `k ×
realization_cost(Int.add, target)`. The compiler walks the
declared program, composes `.dag` CX with language-spec
realization costs per primitive, and yields per-target
complexity bounds for any function. No execution required.

**Why this matters for omni-emission.** With cost complexity as
a first-class thesis claim, the question "which target is fastest
for this workflow?" becomes a compile-time question the compiler
can answer statically. `create_order` in Rust: 2μs median.
Python: 180μs median. Go: 3μs median. Target selection becomes
a cost-aware optimization with bounded complexity estimates,
not an execution experiment.

**Why this is not speculative.** `.dag` already commits to CX as
a correctness dimension (see §"Correctness dimensions"). The
language spec already exists as a declaration pattern (see
§"Concept unification" → "coercion = emission"). Composing the
two requires adding a `realization_cost` field per primitive in
the language spec, and a composition rule in the CX analysis
pass. Both are small additions to existing mechanisms. The
thesis makes the claim here; implementation falls out when the
emitter lands.

**The sustainability consequence.** Adding a new target language
automatically yields target-specific cost analysis for every
existing workflow — zero new work per workflow. Cost-aware target
selection is a free consequence of declaring realization costs
in the language spec.

