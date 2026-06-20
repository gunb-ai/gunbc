# Substrate feedback: primitive grounding (the model↔realization fork)

Surfaced 2026-06-20 while authoring `src/v2/lens/anemia.dag` **by execution** (the friction was real, not
theoretical — receipts below). This is a standalone note for a separate work item; it is feedback on the
substrate primitives, not a plan for the anemia lens. Could alternatively fold into DESIGN.md "Open
threads" — left separate here per the request.

## Thesis

Several foundational primitives carry **two representations that aren't reconciled**: a clean algebraic
*model* and a native *realization*, and they silently disagree. This is the §3 fork/nickname pattern,
except the substrate is forking *itself* at the bottom layer.

| concept | model (the `.dag`) | realization (v1 interp) | symptom |
|---|---|---|---|
| `Nat` | Peano `Zero \| Succ` (`std/nat.dag`) | native machine int | `nat_add(85,32) == 117` → **`false`** |
| `Char` | `= Nat` (nominal alias, `std/text.dag`) | `Int` code point at runtime | `if`-branches `Nat` vs `Char` clash; arithmetic needs native ops |
| `String` | `FreeMonoid<Char>` | `Value::Str` (native) | folds work, but dual repr surprises |
| `fold` | `fold_list` (modeled, `std/algebra`) | `fold` (Rust builtin) | two folds: builtin infers element types but isn't in scope for `String`; `fold_list` works on `String` but leaves the element generic |

## The one that actually costs time: silent `==`

The forks above are mostly *loud* (a `Char` vs `Nat` type error, "fold not found") — terse but they point
you right; that's fail-closed working. The expensive one is **silent**:

```
nat_add(85, 32) == 117   →   false        # two encodings of 117 compare unequal, no error
```

A Peano `Succ`-chain never `==` an `Int` literal. By §5 ("a wrong answer is a loud error, never a
warning") this is the one thing the substrate shouldn't permit: it's a fail-*open*. It masks itself —
identical inputs compute identically, so a test on `f("url") == f("url")` passes while `f("URL") ==
f("url")` silently fails, sending you to grep `v1_interpreter.rs` for the representation split.

Reproduce (≈1s each):
```
cargo build -p v1-compiler --bin claim_batch    # cargo is at ~/.cargo/bin
# add a `fn dbg() -> Bool { nat_add(a: 85, b: 32) == 117 }` to a *_test.dag, then:
./target/debug/claim_batch --source-root src/v2 --source-root dsl \
  --entry <file>_test.dag --function dbg --claim-run        # → FAIL
# but native Int does the right thing:  (85 + 32) == 117      → PASS
# and folded chars ARE Int code points:  fold "A" → c == 65   → PASS
```

## Why — two readings

- **Transitional (charitable).** Migration debt: native int/str/`fold` are the v1 *seed*;
  Peano/`FreeMonoid`/`fold_list` are the v2 *model* meant to replace it. Per §7 the seed "shrinks to
  zero" and the fork closes when self-hosting completes. On this reading it's unfinished, not wrong.
- **Structural (the more useful one).** Peano `Nat` is itself **ungrounded**. §1 says reduce to physics;
  §2's own proudest example is integers as `Compose<Int, MachineWidth<N>>` — numbers grounded in the
  machine word. Peano is a mathematician's convention, not the physical realization, so it was *always*
  going to disagree with the int it runs on. The pattern to fix it already exists in the doc; it just
  hasn't reached `Nat`/`String`/`Char`/`fold`.

## Suggested levers (in priority order)

1. **Fail-closed `==` across representations** — make a cross-representation compare either *correct* or a
   *typed error*, never a silent `false`. Cheapest, highest-value: turns an hour of interp-grepping into a
   one-line diagnostic. Most load-bearing instance of §5 at the primitive layer.
2. **Ground `Nat`/`Int` to the realization** (the `MachineWidth` Realization, §2) so model == realization
   and there is no seam to miscompare across. The structural fix; subsumes (1) for numbers.
3. **One `fold`** — dissolve `fold`/`fold_list` into a single iteration authority with full element-type
   inference over both `List` and `String` (§3 single authority for iteration).
4. **Decide `type X = Y` alias semantics** — `Char = Nat` is nominal (surprising for an "alias"); either
   make plain aliases transparent or name the newtype intent. A clear rule removes the `Char`/`Nat` clash.

## Framing

Intuitiveness here isn't a separate axis from the existing principles — it's the **lagging indicator** of
having finished them. Every spot that forced me to learn an arbitrary fact ("which fold?", "Char isn't
Nat", "Peano ≠ literal") is a spot where a stated principle — single authority (one fold), ground-to-
physics (one number), fail-closed (no silent `==`) — isn't yet realized at the bottom. Make those true at
the primitives and the friction disappears, because there's nothing left to know.

Receipts for the specific v2-authoring facts are in the session's memory note `v2_dag_authoring_gotchas`.
