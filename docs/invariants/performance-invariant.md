## Performance Invariant

Performance is a correctness property for this repo, not a cleanup pass
for later. For every exposed interface, reusable helper, and hot path,
we should know the worst-case time and space bound before we commit to
the design.

The standard is not "fast enough on today's inputs." The standard is
"the asymptotic behavior is understood, intentional, and appropriate for
the role this code plays." Accidental quadratic behavior, repeated full
rescans, hidden reparsing, and large incidental clones are design bugs.

**The rule:** choose the data structure and algorithm that satisfy the
required bound up front. Complexity is part of the interface contract,
especially for APIs that may be called inside larger traversals.

**The test:** if you cannot state the upper bound for a non-trivial
algorithm or interface, the design is incomplete. If a call pattern
turns one scan into `N` scans, or one allocation into `N` large clones,
assume the implementation is wrong until proven otherwise.

**The fix:** write down the dominant operations, then implement to the
target bound directly. Prefer one-time indexing over repeated lookup,
single-pass structural walks over nested rescans, and data ownership
that avoids whole-structure cloning in loops.

### Facts Flow Forward (2026-03-26)

Every performance regression in this compiler traces to one structural
pattern: **a fact is computed at stage X, lost during transformation to
stage Y, and Y compensates with a conservative strategy that is correct
but suboptimal.** The fix is never "optimize the compensation." The fix
is always "stop losing the fact."

The .dag language is pure-functional with lexical scope. In this model,
every property needed for optimal rendering is already expressed by the
source: purity means no aliasing, lexical scope means every binding's
consumers are visible in the syntax tree, named composition means the
data-flow graph IS the program text. If the compiler needs to guess,
a fact was lost.

**The governing rule:** the rendering must preserve the cost model of
the source language. Every guarantee the source provides — purity,
immutability, lexical scope, structural composition — must be
exploited in the rendering to maintain O(1) where the source says O(1).

If the rendering assigns higher cost to an operation than the source
intent, there is a guarantee being ignored. The fix is to exploit the
guarantee, not to optimize the compensation.

| .dag guarantee | What it means | Rendering should exploit | Conservative fallback |
|---|---|---|---|
| **Purity** | Values never mutated | Read = borrow, no copy | Read = clone (defensive) |
| **Lexical scope** | Lifetime = scope | Move semantics, stack alloc | Rc heap allocation |
| **Immutable strings** | Characters are views | `&str` slice (zero-copy) | `String` allocation (heap) |
| **Structural composition** | Graphs have indexed structure | Indexed O(1) lookup | Linear scan |

**Diagnosis:** when you encounter a performance issue or a compensating
mechanism: (1) identify the fact being recomputed or the guarantee
being ignored, (2) find where it was first available, (3) trace where
it was lost, (4) fix the rendering to exploit the guarantee.

#### Known instances

| # | Fact | Computed at | Lost during | Compensation | Cost | Status |
|---|------|-------------|-------------|--------------|------|--------|
| FF-1 | Binding fan-out (use-count) | .dag AST (lexical scope) | v1 emitter rendering to Rust | Rc-wrap all types, clone every use | Every fold O(n²). 20-min self-compile. | **FIXED.** Match-arm count bug (max→add) was root cause of ~50 false single-use classifications. Full fan-out model: clone only at fan-out > 1. Reconcile: 20min → 244ms. v2 ownership analysis (`ownership.dag`) wired into Rust emitter — function params with fan-out=1 move instead of clone. Let-bindings and match-bound variables blocked on VarBindingKind propagation. |
| FF-2 | Resolved structural type | Infer (`.inferred`) | Bare name references at stage boundary | Emit re-resolves through TypeEnv | 12+ re-resolution sites | **FIXED** (C-series) |
| FF-3 | Expression children | Parse (construction) | ExprData variant fields | 12 manual walks (~1800 lines) | Every analysis needs full ExprData match | **FIXED** (P5.11) |
| FF-4 | Module dependency order | Resolve (topo sort) | `dep_order` field + re-sort | Extra field, unnecessary sort pass | Minor | **FIXED** (P5.2) |
| FF-5 | Adjacency structure | `node_type_deps` | Kahn re-scans all items each iteration | Filter-based ready detection | O(n²×d) per module vs O(V+E) | **FIXED.** Indexed Kahn with in-degree map + reverse adjacency + queue drain. |
| FF-6 | Diagnostic properties | Construction (`diagnostic_node()`) | (Previously: separate types) | (Previously: type-specific accessors) | Minor | **FIXED** (P5.3) |
| FF-7 | Service operation structure | Parse (declaration) | (Previously: separate OperationDef) | (Previously: type-specific accessors) | Minor | **FIXED** (P5.4) |
| FF-8 | Container sharing representation | `.dag` value semantics (pure, lexical scope) | Rust container templates: Rc for user types, bare Vec/HashMap/String for built-ins | Emitter inserts `.clone()` on multi-use bindings; O(n) for bare collections, O(1) for Rc-wrapped types. Parser: 991 Vec clones per parse. | Parser: 37s → 0.4s. Tokenizer: 7s → 0.06s. Full compiler: hang → 0.65s. (Hand-patched generated files proved the class.) | **ROOT-CAUSED (2026-03-27), fix pending.** The Rust container templates in `LanguageSpec` must produce shared representations (`Rc<Vec<{0}>>`, etc.). Template + emitter + runtime changes must land atomically with stage0 regeneration. See FF-8 detail. |

#### The fan-out fix (FF-1) in detail

The .dag language guarantees that fan-out is a syntactic property —
count the name references in a binding's scope. The rendering
transformation must preserve this:

- Fan-out = 0 → dead code, don't emit
- Fan-out = 1 → move (the binding is consumed exactly once)
- Fan-out > 1 → duplicate at the fork point

The v1 emitter's contract for use-count preservation: **each .dag
consumption maps to exactly one target-language move.** Rendering-
introduced references (field access, auto-deref, method dispatch) are
borrows, not moves. The emitter must not introduce move-sites that
weren't in the source.

**Status: FIXED (2026-03-26).** The full fan-out model is active.
The match-arm use-count bug (`current.max(max_in_arms)` → `current +
max_in_arms`) was the single root cause of ~50 false single-use
classifications. With that fixed, the Rc-type clone overrides
(`is_rc_named`, `is_rc_collection`, `assume_rc`) were removed.
Clone decision is now purely fan-out + match-bound-var status.
Reconcile: ~20 minutes → 244ms (release mode).

#### Kahn cycle detection fix (FF-5)

**Status: FIXED (2026-03-26).** `04_cycle.dag` rewritten with indexed
in-degree map + reverse adjacency + queue drain. O(V+E), single pass.

#### Container representation — the recurring performance class (FF-8)

Every performance regression in this compiler (FF-1, FF-5, FF-8, the OOM
incident) traces to the same ad-hoc split in the Rust container
templates: user-defined types get shared representations (Rc), but
built-in collection types (List, Map, Set, String) get bare
representations (Vec, HashMap, String). Since the `.dag` language has
value semantics and the emitter inserts `.clone()` on every multi-use
binding, the clone cost for bare collections is O(n) — catastrophic
in any function that threads a collection through multiple calls.

**Status: PARTIALLY FIXED (2026-03-29).** The ad-hoc split between
user types (Rc) and collections (bare) has been eliminated. Container
templates are now bare (`Vec<{0}>`, `HashMap<{0}, {1}>`). Rc-wrapping
is a single rendering decision via the `rc_types` map, built by
`build_rc_types()`, which includes both user types and collection types.
Three duplicate Rc predicates deleted.

**Remaining:** the sharing model is Rust-only. Go emits bare structs
(O(fields) copy cost). See "Emission is translation, not
decision-making" invariant for the cross-language design target.

#### Import resolution is the caller's job — it should be the compiler's (FF-9)

**Status: PARTIALLY FIXED (2026-03-27).** Test harness now does
import-driven transitive resolution via `resolve_imports_transitively`.
Stage0 binary and bootstrap still use manual file assembly.

**The violation:** The compiler takes a flat `List<SourceFile>` and compiles
whatever it's given. Import declarations (`import std.types { List }`)
are validated against the provided sources — if `std.types` isn't in the
list, the import fails. The compiler has no way to discover and load a
module that wasn't pre-loaded by the caller.

This means:
- The stage0 binary manually `collect_dag_files` from a directory
- The test harness resolves imports transitively (fixed 2026-03-27)
- The bootstrap test manually copies specific std files

**What's lost:** The import declarations in `.dag` source files are the
complete, authoritative dependency graph. The compiler already parses
these imports and validates them. But it treats them as assertions about
what the caller provided, not as demands for what to load.

**The fix:** Import-driven source resolution. The compiler (or a thin
layer above it) resolves imports to files:

1. The caller provides a **source root** (or roots), not a flat file list
2. The compiler parses the entry point, discovers imports, loads
   transitively referenced modules from the source roots
3. Only files reachable from the entry point's import graph are loaded
4. The resolve stage already builds the dependency graph — the missing
   piece is wiring it to file discovery

Each module loaded exactly once (HashMap memoization). Diamond deps
(A imports B and C, both import D) hit the seen check. O(V+E).

**Impact:** Eliminates the kernel seed (modules that need `List` import
it; the import loads `std.types` which loads `std.algebra`). Tests use
the same resolution as production. Every compilation loads exactly what
it needs — minimal and universal.

#### Ratchet

Fan-out is not "metadata" to be computed and carried — it is the
out-degree of a binding's edges, already present in the graph structure.
The emitter doesn't need new information. It needs the right default
rendering per language, declared in `LanguageSpec`.

**The 2026-03-27 incident (proof of class):**

Hand-patched generated stage0 files proved the fix class:
- Parser: `Vec<Rc<Token>>` → `Rc<Vec<Rc<Token>>>` (991 clone sites,
  O(n) → O(1))
- Results: parse 37s → 0.4s, tokenize 7s → 0.06s, full compiler
  hang → 0.65s

**2026-03-29 fix:** Container templates made bare. Rc-wrapping unified
via `rc_types` map (single authority). 689 redundant `.clone()` removed
from stage0. Self-compile completes in ~2 min at 112MB. Regen pipeline
produces 40 files with 0 diagnostics.

