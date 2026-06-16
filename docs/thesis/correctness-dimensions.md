## Correctness dimensions

Correctness is not one property — it is many orthogonal dimensions:
termination, type safety, ownership, side effects, purity,
idempotence, space bounds. In traditional systems these are separate
tools (type checker, linter, static analyzer, profiler) that you
opt into. In gunbc, they are **inescapable properties of the
system**, like conservation laws in physics. You don't opt into
gravity.

Every dimension is:
1. **Declared in `std/`** as a structural type with lattice
   operations (meet, join, top, bottom)
2. **Computed at binding sites** during inference — no separate
   analysis pass
3. **Carried through the IR** on bindings, from computation to
   consumption
4. **Enforced universally** — all code is subject to all dimensions,
   no escape hatch, no wrapper functions

The compiler doesn't have "a complexity pass" and "an ownership
pass." It has one mechanism that reads whatever dimensions `std/`
declares and enforces them all uniformly. Adding a new dimension
means declaring a lattice in `std/` and its binding-site rule.
The compiler carries it generically. Cost of change: one file.

Current dimensions and status:

| Dimension | Declared in | Lattice? | Carried on bindings? | Enforced? |
|-----------|------------|----------|---------------------|-----------|
| Type safety | std/types.dag | N/A (structural) | TypeBinding.resolved | Yes (blocking) |
| Termination | std/termination.dag | BoundedLattice | TypeBinding.provenance + ExprCall.descent_evidence | Partial (421 violations, non-blocking) |
| Coercion | (not a separate dimension — coercion IS emission; CX proves bounds on emission functions) | — | — | Partial (fail-closed where implemented) |
| Ownership | ownership.dag | Not yet | Not yet (separate pass) | Partial (SharedError blocks) |
| Side effects | std/behavioral.dag | Not yet | Not yet | No (declared, not consumed) |
| Purity | (not declared) | — | — | No |
| Idempotence | std/effects.dag | Lattice (derived from EffectShape) | Not yet | No (algebra declared, not consumed) |
| Space bounds | (not declared) | — | — | No |

The architecture is: **as dimensions move from "separate pass" to
"lattice on bindings," the compiler gets more correct without
getting more complex.** Each dimension dissolved into the binding
mechanism is one fewer analysis pass, one fewer set of heuristics,
one fewer source of reconstruction bugs.

### User-defined dimensions

The mechanism is not compiler-internal. If the architecture is
correct, users can declare their own correctness dimensions — the
compiler enforces them with the same machinery it uses for
termination and ownership.

Examples:
- **Security classification** — `Public | Internal | Secret` as a
  lattice. Secret data can't flow to a Public drain without a
  declassifier. Enforced at every binding.
- **Regulatory compliance** — `PHI | NonPHI` for HIPAA. Patient
  data can't flow to non-compliant storage.
- **Financial provenance** — every monetary computation carries
  provenance to its authorization source.

A user declares a lattice, attaches it to their types, and the
compiler enforces it universally. No special tooling. No
annotations. The same non-consensual enforcement that applies to
termination applies to their proprietary model.

**This is the test of the architecture.** If user-defined
dimensions work the same as built-in ones, the mechanism is
general. If they require special compiler support, the mechanism
is incomplete.

Design: [src/v1/dimensions-design.md](../../src/v1/dimensions-design.md)
— the general mechanism abstracted from CX and ownership.
