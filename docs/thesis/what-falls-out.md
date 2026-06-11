## What falls out

### Zero bugs

If every causal link from source to drain is validated, there are
no bugs. A bug is a broken causal link — a field that doesn't
exist, a branch that isn't handled, a computation that doesn't
terminate, a type that doesn't match. The compiler checks every
link. What it can't check statically, it generates tests for.

Three tiers of the zero-bug guarantee:

### Tier 1: Structural bugs — impossible by construction

These bugs cannot be written. The type system, exhaustiveness
checking, and structural descent proofs make them unrepresentable.

| Bug class | Mechanism | Status |
|-----------|-----------|--------|
| Field typos in generated code | Emitter derives names from declarations | DONE |
| Field typos in `.dag` source | FieldNotFound diagnostic | DONE |
| Non-exhaustive match | NonExhaustiveMatch diagnostic | DONE |
| Type mismatches | TypeMismatch diagnostic (branches, args, returns) | DONE |
| Bare container types | ArityMismatch diagnostic | DONE |
| Map key type mismatch | Infer-stage type check | DONE |
| Stale imports | UnresolvedType / MissingExport diagnostics | DONE |
| Circular dependencies | CircularDependency diagnostic | DONE |
| Cross-target drift | Single `.dag` declaration → all targets | DONE |
| Diamond dependency divergence | Module graph deduplicates imports | DONE |
| Non-termination | Structural descent proof (CX gate) | **421 violations → 0, then blocking** |
| Non-idempotent workflow | Effect algebra composition (std/effects.dag) | **not started** — algebra declared, compiler consumption not wired |
| Record literal completeness | Missing-field diagnostic | **partial** |
| Coercion completeness | Fail-closed inhabitant lookup; coercion = emission (not a separate mechanism) | **partial** — schema + dispatch + per-language data done; single emitter (Lane C) not started |

**Gating items:** CX gate (421 → 0, then blocking) and emission
completeness (every .dag→target conversion is a declared .dag
function with CX-proven bounds). Coercion is not a separate gate
— it is emission. When the single emitter (Track 13) lands,
coercion completeness is a consequence.

**Note:** Tier 1 status claims reflect what the compiler enforces
today, not aspirational targets. "DONE" means the diagnostic exists
and blocks compilation. Items marked "partial" have gaps documented
in their design docs.

### Tier 2: Runtime safety — proven safe or total

These bugs compile today but crash at runtime. Closing them means
the compiled program cannot panic, trap, or produce silent wrong
data from safe operations.

| Bug class | Current state | Path to zero |
|-----------|--------------|--------------|
| Division by zero | Unchecked | Model divisor as NonZero or emit checked_div |
| Integer overflow | Wraps or panics (Rust-dependent) | Bounded arithmetic or checked ops |
| String/array out-of-bounds | Silently returns empty string | Require bounds proof or emit checked access |
| Optional force-unwrap | Unchecked panic | Require match/if-let to extract; no `.force()` |
| Partial functions | Some runtime helpers are partial | Make all runtime functions total |

**Design principle:** either prove the precondition at compile time
(refinement types) or make the operation total (return Option,
use checked arithmetic). No partial functions in the runtime.

### Tier 3: Verification from structure

In a causal engine, structure and behavior are coupled. The `.dag`
source IS the behavior specification — not a separate oracle. The
compiler has both the intent (declarations) and the output (emitted
code). Verification is: **does the emitted code faithfully translate
the `.dag` evaluation?**

The compiler can generate witness values for any type (all data is
finite, all types have known inhabitants). For any function
`f(x, y)`, the compiler can evaluate it at the `.dag` level for
generated inputs, emit it to each target, execute the emitted code,
and compare results. The `.dag` source is the oracle. No
hand-written tests needed.

This is not "testing" in the traditional sense. It is **emission
verification** — proving that the mechanical translation is
faithful.

| Test level | What it proves | Status |
|------------|---------------|--------|
| L0: Structural tests from data | Coercion mappings are complete and consistent | DONE |
| L1: Pipeline unit tests | Compiler stages produce correct output | DONE (393 tests) |
| L2: Bootstrap self-hosting | Compiler can compile itself | DONE |
| L3: Syntax validity | Emitted code parses in target language | DONE |
| L4: Semantic correctness | Emitted code executes, matches `.dag` evaluation | **not implemented** |
| L5: Cross-language equivalence | Same `.dag` → same behavior in Rust/Python/Go | **not implemented** |
| L6: Exhaustive form coverage | Every structural form compiles to every target | **not implemented** |
| L7: Algebraic law verification | fold/map/filter obey their declared laws | **not implemented** |

**Gating items:** L4 (semantic correctness) is the critical gap.
The compiler can evaluate `.dag` functions directly (closed,
decidable, finite). The emitted code must agree. Until L4 is
gated, "emission is mechanical translation" is unverified.

---

