## What .dag catches that normal compilers don't

These are concrete examples of bugs and inefficiencies that .dag
rejects at compile time. A normal compiler (Rust, Go, Python, etc.)
would compile every one of these without complaint. .dag catches
them because the closed system gives the compiler enough algebraic
structure to prove they are wrong.

### Structural bugs (impossible to write)

| What .dag catches | What a normal compiler does | How .dag catches it |
|---|---|---|
| Non-terminating recursion (`check_type(resolve(name))` where resolved type is recursive) | Compiles fine. Stack overflow at runtime. Real bug in TypeScript, Rust, Haskell compilers. | CX demands structural descent proof. `resolve(name)` is a lookup, not descent — `SubValueUnknown`. Rejected. |
| Accidentally quadratic (`process(items)` inside `items |> map(...)`) | Compiles fine. O(n²) at runtime. | CX tracks cost composition. `fold(n, fold(n, ...))` = O(n²). If a cheaper equivalent exists (single fold), compile error. |
| Infinite mutual recursion (`f(n) → g(n) → f(n)`) | Compiles fine. Stack overflow at runtime. | CX analyzes SCCs. Neither call shows descent. Both rejected. |
| Recursion on sibling instead of child (`process(node)` instead of `process(child)`) | Compiles fine. Infinite loop at runtime. | CX sees `PreservedValue` (same node), not `StrictSubValue`. Rejected. |
| Work-list that grows unboundedly | Compiles fine. OOM or infinite loop at runtime. | `repeat(N)` requires explicit bound. No unbounded iteration primitive exists. |

### Redundant work (wasteful but compiles)

| What .dag catches | What a normal compiler does | How .dag catches it |
|---|---|---|
| `list |> reverse |> reverse` | Compiles fine. Wastes O(n) work. | `reverse` is an involution (`f ∘ f = id`). Composition simplifies to identity. Compile error: "equivalent to doing nothing." |
| `data |> serialize |> deserialize` | Compiles fine. Wastes serialization cost. | Declared inverse pair. Composition = identity. |
| `map(f) |> map(g)` (two passes) | Compiles fine. Two traversals where one suffices. | Map fusion law: `map(f) ∘ map(g) = map(f ∘ g)`. One pass is cheaper. |
| Clone a value used only once | Compiles fine (Rust requires it in some contexts). Wastes allocation + copy. | Ownership analysis: fan-out = 1. Last use can move. Clone is redundant. |
| Infrastructure bringup that re-provisions already-running services | Compiles fine. Wastes API calls and time. | Effect algebra: all operations are lattice meets (upsert). Workflow is idempotent — re-running is benign but the compiler can flag the redundancy. |

### Effect safety (silent bugs at runtime)

| What .dag catches | What a normal compiler does | How .dag catches it |
|---|---|---|
| Non-idempotent workflow marked as safe to retry | Compiles fine. Duplicates data on retry. | Effect algebra derives idempotency from effect shape. `POST /logs` (List append) is not idempotent. Compiler shows which operation breaks it. |
| Write-then-overwrite (dead effect) | Compiles fine. First write is wasted. | Effect composition: `upsert(k, v1) ∘ upsert(k, v2) = upsert(k, v2)`. First effect is subsumed. |
| `create_resource()` in a retry loop | Compiles fine. Creates duplicates on retry. | `POST` without key = `CreateEffect` = not idempotent. Compile error inside `repeat()` or retry context. |

### Complexity violations (wrong algorithm)

| What .dag catches | What a normal compiler does | How .dag catches it |
|---|---|---|
| O(n²) where O(n) suffices | Compiles fine. Slow at runtime. | CX proves cost. Algebraic simplification finds cheaper equivalent. KF-2 rejects. |
| Unbounded recursion depth | Compiles fine. Stack overflow at runtime on deep inputs. | CX proves depth bound from structural descent. No bound = rejected. |
| `fib(n-1) + fib(n-2)` (O(2ⁿ)) | Compiles fine. Exponential at runtime. | CX branching guard: multiple recursive calls with arithmetic descent = exponential. Rejected unless memoized or reformulated. |

Concrete `.dag` code examples with compiler errors:
[docs/error-examples.md](docs/error-examples.md) — serves as TDD
targets for the compiler. Each example is a test case: the .dag
code should compile today, and the error message is the acceptance
criterion for when the feature lands.

### The common pattern

Every row in every table above is the same mechanism: the compiler
has the algebraic structure (descent proofs, effect shapes, cost
algebra, inverse declarations), composes operations symbolically,
and checks whether the composition satisfies the required property.
No special-case analysis. No lint rules. No opt-in annotations.
The algebra does the work.

---

