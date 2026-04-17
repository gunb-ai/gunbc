> Part of: [THESIS.md](THESIS.md) — these invariants are the structural
> rules that enforce causal consistency. The thesis says "every causal
> link is validated"; this document says how.

# Compiler and Runtime Invariants

This document governs the engineering invariants for the entire
codebase: the v1 Rust compiler (`src/v1/`), the v2 self-hosted
compiler (`src/v2/`), and the DSL source (`dsl/`).

## Modeling Faithfulness Invariant

The compiler and the `.dag` source share one governing principle:
every construct must be grounded in an identifiable external fact
(axiom, specification, standard, or structural derivation). Constructs
without factual grounding are not valid authorities in this codebase.

Full modeling guidelines: [`MODELING.md`](MODELING.md).

**The compiler's role:** enforce faithfulness mechanically. When the
compiler encounters a type, coercion, or structural claim for which no
grounding fact is declared, it must produce a diagnostic error. Silent
defaults, fabrication fallbacks, and placeholder emissions are violations
of this invariant — they allow ungrounded claims to propagate through
the pipeline as if they were facts.

**Annotations are not facts.** When a structural gap requires new
information, the fix is to extend `.dag` structure (types, edges,
functions), not to add metadata or annotations. The `.dag` language is
the meta-language for expressing intersubjective agreements; there is
no meta-language above it.

**No annotation mechanisms at any layer (ruled out through M3).** The
rule against annotations applies at every layer of the stack, not
just at the `.dag` source level:

- **Language level.** `.dag` has no annotations, attributes, pragmas,
  semantic comments, or side-channels for attaching metadata to
  declarations. Every fact is a first-class structural piece of the
  language. If a feature feels like it wants an annotation, that is
  a signal the core language is missing a structural primitive.
  The fix is to add the primitive, not the annotation.

- **Compiler data model level.** The DAG substrate (Port, Behavior,
  Declaration, diagnostics) carries only structural facts load-bearing
  for causal correctness — types, spans, `produced_by` edges, port
  state, declared signatures. It does NOT grow "annotation tables,"
  "attribute maps," or side storage for lens-produced derived facts.

- **Lens level.** Lenses are pure functions from `&Dag` to derived
  values, not annotation mechanisms. A lens reads the DAG and
  computes its answer on demand. It does not write results back into
  the Dag for later consumers. Cross-lens queries combine per-lens
  call results at the call site, not via a shared annotation store.

**Why this is ruled out now:** the core language is still being
discovered. Until it is clear what structural primitives the language
actually needs, allowing annotations as an escape hatch would fill
real gaps with a decorative layer and make the gaps invisible. The
annotation layer would accrete consumers, become load-bearing, and
prevent the language from growing the structural primitives it needs.
This is exactly the pattern v2 hit — metadata bolted on to cover
missing type declarations, then impossible to remove because downstream
code depended on the metadata. The ruling: if it feels like it wants
an annotation, that is a discovery opportunity, not an implementation
question. Bring the discovery back to the language design, not to the
compiler's data model.

**When this may be revisited:** at M3 (self-hosting) or later, once
the core language has survived contact with real code and the
structural primitives have stabilized. Before M3, the default answer
is always "no annotations." Contributors should not propose annotation
mechanisms, annotation-like side tables, or "just a small metadata
map for this lens" extensions without explicit authorization that
references this rule.

**Concrete implications for the v3 substrate (M0 – M3):**

- The Dag carries only Port, Behavior, Declaration, diagnostics. No
  `annotations: HashMap<NodeId, _>` field, no `lens_results:`, no
  attribute system. The diagnostics table is not an annotation
  system — it is the fail-closed channel for compile failure, linked
  to ports by a biconditional invariant, and it is the only kind of
  side-lookup allowed in the substrate.
- The provenance lens reads `produced_by` and classifies by behavior
  kind. The depth lens walks ports. Any future lens (cost, ownership,
  effect, termination) is also a pure function of `&Dag`. None of
  them write back.
- The success bar "adding a new analysis is trivial" is measured
  concretely: write a pure function from `&Dag` to your derived
  type, in its own file, with zero substrate modifications. If a
  proposed lens cannot be built this way, the failure mode is to
  revisit the substrate's structural facts (is a needed fact missing
  from Port/Behavior/Declaration?), not to add an annotation layer.

This invariant is upstream of all others. Performance invariants assume
the model is faithful. Decidability proofs assume the structures are
well-grounded. Sustainability rules assume facts have single authorities.
If the modeling is unfaithful, the downstream invariants are protecting
the wrong thing.

## Bounded Substrate Seed

The Rust-native seed that exists before any `.dag` declaration loads is
a ratchet, not an escape hatch. The seed may stay flat or shrink; it may
not grow without explicit deletion elsewhere in the seed or a narrowly
argued exception that names why the new primitive is truly indivisible.

At the current reflection boundary the intended seed is minimal:

- Parser and tokenizer substrate for reading source text.
- Resolve / bootstrap machinery needed to load the first declarations.
- Primitive scalar groundings (`Int`, `Bool`, `String`) and the atomic
  identity handles (`NodeId`, `PortId`, `DeclarationId`, `SourceSpan`).

Everything else in the compiler substrate must live as a `.dag`
declaration. `Dag`, `Declaration`, `Behavior`, `TypeConnective`,
behavior payload records, and realization metadata are no longer valid
"just keep it in Rust" categories now that the reflection surface
exists. If a concept is structurally representable and used above the
seed boundary, the correct move is to declare it in `.dag` and attach a
realization, not to enlarge the seed.

The enforcement rule is monotonic: count seed primitives in a tracked
ratchet file or equivalent CI gate, and block PRs that increase the
count. The project can tolerate temporary seed holes only when the hole
is explicitly named, bounded, and shrinking.

## Lenses Are Substrate Declarations

The canonical form of a lens is a `.dag` declaration operating over the
reflected substrate, not a permanently hand-written Rust module. Rust
lenses are tolerated only as bootstrap scaffolds while the remaining
execution path for compiled lenses is being finished.

This invariant has two operational consequences:

- New lens features should prefer substrate-level facts and realization
  bindings over Rust-side helper APIs.
- The number of Rust bootstrap lenses is itself a ratchet. Deleting a
  Rust lens in favor of its `.dag` form is forward progress; adding a
  new permanent Rust-only lens is not.

The point is not aesthetic. A lens declared in the substrate can be
type-checked, realized, diffed, and eventually analyzed by other lenses.
A Rust lens is opaque to the substrate and therefore a standing hole in
the self-inspection claim.

## Every Dependency Is A Substrate Fact

Whenever a downstream consumer needs a fact, that fact must have a
structural home in the substrate or in a declared realization. Consumers
must not reconstruct dependencies from names, parallel tables, or hidden
calling convention knowledge.

Applied to the reflection work, this means:

- "Which field does this projection read?" is a substrate fact carried
  by the field label and realized via `FieldBinding`.
- "Which port is the primary result of this node?" is a substrate fact
  surfaced compositionally as `result_port`, not a five-arm Rust helper.
- "How do I read this reflected field from Rust?" is a realization fact
  in `rust.dag`, not a hardcoded emitter branch.

If a consumer cannot answer its question by following typed edges and
declared bindings, the fix is upstream: add the missing substrate fact
or realization binding. Reconstructing the fact locally is a violation
of Single Authority and a precursor to drift.

## Root-Cause Depth Invariant

This codebase is a DAG — both the language it compiles and its own
internal architecture. The purpose of every invariant in this document
is not just to flag violations where they appear, but to DFS the
dependency graph upstream from each violation until a sound node is
found. The violation lives at the deepest unsound node, not at the
leaf where the symptom was observed.

A downstream symptom — a heuristic, a duplicate, a fabrication — is
often correct code doing the best it can with what it received. The
bug is upstream: a missing fact, an incomplete type, a structure that
was never surfaced. Fixing the leaf treats the symptom. Fixing the
deepest unsound ancestor treats the disease.

**The rule:** when reviewing or diagnosing, do not stop at the first
node that looks wrong. Walk every parent in the dependency chain and
check each for violations. The fix belongs at the deepest node where
an invariant is broken. If a downstream stage needs information that
isn't available, the fix is to surface that information from its
origin — not to re-derive, guess, or hardcode it at the consumption
site.

**The test:** for any proposed fix, ask: "does this fix a root cause,
or does it compensate for a broken ancestor?" If the fact it relies on
originates in a producer (core types, parse, resolve, infer) but the
fix is in a consumer (emit, complexity, ownership), the fix is in the
wrong place. Move the fact upstream.

**The connection to other invariants:** "No duplicate representations"
is a consequence — duplicates arise when a downstream stage re-derives
what should come from upstream. "Heuristics indicate lost structure" is
a consequence — heuristics arise when upstream structure was never
surfaced. "No fallbacks that fabricate" is a consequence — fabrication
fills the gap left by a missing upstream fact. This invariant is the
shared root cause of all three.

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

## Early Detection Invariant

Alert others of problems as soon and as loudly as physically possible.

Errors detected at stage N must not survive silently to stage N+1.
If the compiler knows something is wrong — a type mismatch, an
unresolved name, an inference failure — it must report it at the
stage where the information is first available. Deferring errors to
later stages (or worse, to emitted code) is a design failure: it
hides the root cause behind cascading symptoms in a different context.

**The rule:** every stage boundary is a gate. Facts that are wrong
or missing must produce diagnostics at the stage that owns the fact.
If an inference failure reaches the emitter, the emitter should
`compile_error!` — but the real fix is always upstream, in the stage
that failed to resolve the fact.

**No warnings.** Every diagnostic is either an error (compilation
stops or emitted code is structurally wrong) or absent (compilation
succeeds). There is no warning severity. A condition that is wrong
enough to report is wrong enough to fail. Warnings create a class of
"known-bad but tolerated" state that erodes invariants over time — if
the compiler knows something is wrong, it must refuse to proceed, not
annotate and continue. If a condition is truly harmless, it is not a
diagnostic. If it is harmful, it is an error.

**Corollary:** emitted code should never fail to compile due to
errors the compiler could have caught. If `cargo check` on emitted
Rust finds type mismatches, those are emission bugs — the compiler
had the type information and lost it during rendering.

## Strict Forward Progress

Time flows strictly forward. Every execution step moves the
computation forward through a bounded structure — never revisiting,
never cycling.

This is the foundational invariant from which decidability,
complexity analysis, and termination all follow. A recursive
function is a logical description of a computation. Its execution
is a finite walk forward through time. The recursion is syntax;
the forward progress is physics.

**Cyclic relations are expressible; direct cyclic values are not.**
Cyclic domains (graphs with back-edges, circular dependencies,
mutual references) are representable via acyclic encodings — adjacency
maps keyed by stable IDs (`Map<NodeId, List<NodeId>>`), parent
pointers as ID references, etc. The cyclicity lives in the interpreted
relation, not in the stored value graph.

Direct cyclic object graphs (heap topology with back-edge pointers)
are not expressible as core values. Values are immutable, finitely
constructed, and acyclic in their physical structure.

**Computations over cyclic relations must still be bounded.**
An acyclic encoding does NOT automatically make every graph algorithm
acceptable. A traversal that follows adjacency links without a visited
set or other decreasing measure is still logically unbounded. The safe
rule: every computation over a cyclic relation must be justified by an
explicit finite measure — frontier size, unvisited-node count, or a
fold/repeat bound over |V| or |E|.

```
// CORRECT: fold over map_keys (finite container), not edge-following
fn reachable(g: Map<String, List<String>>) -> Map<String, Bool> {
  // bounded by |map_keys(g)| — the container, not the topology
  repeat(bound: map_keys(g) |> count, ...)
}

// REJECTED: recursive traversal without explicit finite measure
fn walk(g: Map<String, List<String>>, node: String) -> List<String> {
  let neighbors = map_get(g, node)
  neighbors |> flat_map(n => walk(g, n))  // unbounded — no visited set
}
```

**The three-part distinction:**
- Cyclic relations: **yes** (data can encode arbitrary graph topologies)
- Direct cyclic values: **no** (values are acyclic in physical structure)
- Traversals over cyclic relations: **yes, if bounded by explicit
  finite structure** (|V|, |E|, frontier size, visited count)

**Why this matters:** if execution can only move forward, then:
- Every program terminates (decidability)
- Every program has a computable cost (complexity analysis is total)
- Every program has a computable memory footprint (space analysis)
- Composition is closed (forward + forward = forward)
- The compiler itself provably terminates on all inputs

## Decidability Invariant

All `.dag` programs are decidable. Undecidable programs are structurally
unrepresentable — the language has no primitive for unbounded computation.

This follows directly from strict forward progress. If every execution
step moves forward through a bounded structure, the computation must
terminate — there are only finitely many steps to take.

This is the highest-leverage invariant in the system. If every function
terminates, then: complexity analysis is total (every function gets a
time and space bound), space analysis is total (peak memory is
computable), the compiler itself is provably terminating on all inputs,
and composition is closed — piping two `.dag` programs together is still
decidable. Without decidability, one unbounded function poisons the
entire pipeline.

### Structural proof from primitives

Decidability is a consequence of the language's modeling primitives,
not a per-function check. The proof has three parts:

**Part 1: All values are finitely constructed.**

Base values are finite: `Bit` has cardinality 2, `Word64` has 2^64.
Every constructor preserves finiteness:

| Constructor | Cardinality | Preserves finiteness |
|---|---|---|
| Product (Conj) | \|A × B\| = \|A\| · \|B\| | Product of finite = finite |
| Coproduct (Disj) | \|A + B\| = \|A\| + \|B\| | Sum of finite = finite |
| Collection append | \|list ++ [x]\| = \|list\| + 1 | Increment of finite = finite |
| Node construction | children is a finite List | Finite list of finite = finite |

There is no constructor for infinite values. A collection of 10^200000
elements is finite — it has a cardinality. The compiler does not care
about the *value* of the cardinality, only that one *exists*.

**Part 2: All iteration is bounded by finite structure.**

The language provides exactly three iteration primitives (see
`std/iteration.dag`):

| Primitive | Bound | What it processes |
|---|---|---|
| `fold` | \|collection\| | Each element of a finite collection |
| `descend` | \|tree\| | Each node of a finite tree (catamorphism) |
| `repeat(N)` | N | Explicit count, N can be up to 2^63 - 1 |

There is no `while(true)`, no unbounded `loop`, no general recursion.
These primitives are the ONLY way to iterate. Each takes a finite
structure or explicit bound and processes it in bounded steps.

**Part 3: Composition preserves boundedness.**

Bounded operations compose to bounded operations:
- Sequential: cost(a; b) = cost(a) + cost(b) — bounded + bounded = bounded
- Nested: cost(fold(list, f)) = |list| × cost(f) — bounded × bounded = bounded
- Conditional: cost(if c then a else b) = cost(c) + max(cost(a), cost(b)) — bounded

No composition of bounded primitives produces unbounded computation.
The primitives are closed under composition. QED.

### Recursive syntax is sugar

Developers write recursive functions for readability. The compiler
lowers every call pattern to a bounded primitive:

| Call pattern in recursive function | Lowers to | Why it's bounded |
|---|---|---|
| Self-call on child of input | `descend` (catamorphism) | Bounded by \|tree\| |
| Self-call inside `fold` body | Already bounded by fold | Fold bounds the iteration |
| Self-call with `n - 1` | `repeat(n, ...)` | Bounded by n |
| Mutual recursion (SCC) on children | `descend` over SCC | Bounded by \|SCC\| |
| Self-call with unchanged argument | `repeat(Forever, ...)` | Bounded by 2^63 - 1 |

No call pattern is rejected. The last row uses the bounded truth
principle: in a Bit/Word64 system, "forever" is a finite bound
(2^63 - 1 iterations). `repeat(Forever)` is not an approximation of
infinity — it is the correct answer for the largest representable
iteration count. See `std/computation.dag` (CallPattern →
LoweringTarget) and `std/iteration.dag` for the full model.

### Fail-closed compilation

Decidability is enforced at two levels:

1. **Structural (construction):** The language has no unbounded iteration
   primitive. Every call pattern maps to exactly one bounded primitive
   via the exhaustive lowering table in `std/computation.dag`.

2. **Fail-closed (compilation):** If the compiler encounters a call
   pattern it cannot classify (a gap in the classifier, not in the
   model), compilation fails with a hard error. This is a safety net —
   it catches analyzer incompleteness. In a correct implementation,
   this error is unreachable because the lowering table is exhaustive.

The complexity analyzer does not enforce decidability — it derives cost
formulas from the bounded structure that the language guarantees. If the
analyzer produces `?O(?)`, the bug is in the analyzer (it cannot see the
bound that structurally exists), not in the program.

### Tight upper bounds — no exceptions

Every function and expression in the language must have a **provably
tight** upper bound. `Conservative` certainty is a modeling deficit,
not acceptable steady state. `Unknown` certainty is a hard error.

### Cost algebra is upstream of language primitives

The cost algebra (`CostExpr`) is the **upstream authority** that
determines what the language can express, not a downstream attempt to
describe what the language already does.

```
Cost algebra defines expressible cost classes
    ↓
Language primitives must declare a cost from the algebra
    ↓
Complexity analyzer reads the declaration — trivially correct
```

A language primitive cannot be added until its cost class exists in
the algebra. This is the same structural guarantee as decidability:
just as bounded primitives make undecidability unrepresentable, the
cost algebra makes unanalyzable primitives unrepresentable.

**The current `sort_by` gap is this deficit in action.** `sort_by` was
added without `CostLog` in the algebra. The analyzer falls back to
Conservative O(n) — valid but not tight. The fix is not "add CostLog
later." The fix is: the algebra must have `CostLog` before `sort_by`
can exist. Adding the primitive without its cost class violates the
modeling order.

**The contract:** for any `.dag` program P, the complexity analyzer
produces bound B such that B is the exact tight bound for P. No
`Conservative`. No `Unknown`. Every function is `Proven`. This is
guaranteed by construction because every primitive declares its cost
in the algebra, and the algebra can express that cost exactly.

**The principle:** the cost algebra and language primitives are
co-designed, with the algebra leading. When someone proposes a new
primitive, the first question is: "what is its cost class, and can
the algebra express it?" If not, the algebra grows first. The
primitive follows.

### Practical ergonomics

Decidable does not mean small. The bound can be astronomically large:

```
// Server that handles requests for 292 million years (at 1 req/ms)
fn serve(handler: fn(Request) -> Response) {
  repeat(bound: max_int, f: (_) => handler(accept_request()))
}

// Process with generous safety margin
fn process_batch(items: List<Item>, safety_factor: Int) {
  repeat(bound: items |> count * safety_factor, f: process_next)
}
```

Developers think "serve forever" or "process with margin." The compiler
sees bounded iteration. Same program, different semantics. The developer
gets smooth ergonomics. The compiler gets total analysis.

### Closure property

If someone builds a DSL on top of `.dag`, that DSL is also decidable.
The DSL is composed from `.dag` primitives, which are all bounded.
There is no escape hatch. To express unbounded computation, someone
would need to invent a new modeling language from scratch — they cannot
reach it by composing `.dag` primitives.

### Lowering table

Every recursive pattern has a bounded iterative equivalent:

| Recursive pattern | Structural bound | Bounded lowering |
|---|---|---|
| Tree walk (visit children) | \|nodes\| (strict child descent) | `descend` over tree structure |
| Tokenizer loop (advance pos) | \|source\| (monotonic advance) | `fold` over characters with position |
| Accumulator recursion | decreasing counter or list length | `repeat(n, ...)` or `fold` with init + step |
| Mutual recursion (A↔B on children) | \|SCC\| with shared measure | `descend` over SCC-ordered nodes |
| Long-running process | explicit bound | `repeat(bound: N)` with N up to 2^63 - 1 |

Graph-like properties (cycles, unbounded iteration, general recursion)
are not expressible in the core language. Recursive syntax is surface
sugar that the compiler lowers to these bounded forms.

## Verifiability Invariant

**Design direction.** All `.dag` programs should be verifiable by
construction. Unverifiable programs should be structurally
unrepresentable — every construct carries enough structural information
for the compiler to derive its verification obligations.

**Current implementation:** Coercion data tests are auto-generated from
TypeCheckpoint/InhabitantDecl declarations (L0). Weather.dag L4 PoC
proves emitted code runs with structural witnesses. Witness generation,
algebraic law testing, and constraint oracle evaluation are not yet
implemented. See `src/v2/tests/testing-strategy.md` for the full level map.

This is the testing analog of the Decidability Invariant. Decidability
says: the structure makes unbounded computation impossible. Verifiability
says: the structure makes untestable code impossible.

### Structural proof from type system

Verifiability is a consequence of the type system, not a per-function
opt-in. The proof has three parts:

**Part 1: Every type has a constructible witness.**

Base values have canonical witnesses: `Bit` → `false`, `Int` → `0`,
`String` → `""`. Every constructor preserves witness-constructibility:

| Constructor | Witness | Constructible? |
|---|---|---|
| Product (Conj) | All fields present with child witnesses | Yes — product of constructible = constructible |
| Coproduct (Disj) | First variant with child witness | Yes — at least one variant is constructible |
| Optional | Both: present(witness) AND absent | Yes — two witnesses |
| Collection | Empty + one-element with child witness | Yes — two witnesses |
| Node | Children are a finite list of witnesses | Yes — finite list of constructible = constructible |

There is no type without a constructible witness. A type with 2^100
cardinality combinations still has a canonical witness — the compiler
doesn't enumerate all values, it constructs one representative per
structural form.

**Part 2: Every function is exercisable.**

A function takes typed parameters and returns a typed result. Since
every type has a constructible witness (Part 1), the compiler can:
- Construct input values from parameter type witnesses
- Call the function
- Check the output inhabits the return type

This is structural: the function signature IS the test specification.
The parameter types determine the inputs. The return type determines
the oracle. No hand-written test data needed.

**Part 3: Every algebra declares its own laws.**

Algebraic structures (Monoid, Ring, FreeMonoid, etc.) carry structural
laws: identity, associativity, commutativity. When a type inhabits an
algebra, the laws become verification obligations. The compiler generates
property tests from the laws and exercises them with witness values.

| Algebra | Laws | Generated test |
|---|---|---|
| Monoid | `op(identity, x) == x` | Call with identity + witness, assert equal |
| Ring | `add(zero, x) == x`, `mul(one, x) == x` | Call with zero/one + witness, assert equal |
| FreeMonoid | `concat(empty, xs) == xs`, `filter(xs, p) \|> all(p)` | Concat with empty, filter with predicate |

No composition of typed constructs produces an unverifiable program.
The type system is closed under composition for verifiability. QED.

### What this replaces

Without this invariant, testing is an obligation the developer manages
separately from the code. Tests are written after the fact, coverage
is tracked by external tools, and untested code silently ships.

With this invariant, testing is structural. The same way a `.dag`
developer cannot write an infinite loop (the structure prevents it),
they cannot write untested code (the structure generates the tests).
`under_specified` is not a status the compiler detects — it is a state
the structure makes impossible to represent.

### The one boundary

The only boundary where verifiability requires external evidence is
**integration with external systems** — real HTTP endpoints, real
databases, real cloud APIs. The compiler proves the mock contract
matches the type signature (structural). It generates integration
test artifacts for live verification (Tier 3). But it cannot prove
the real service's behavior matches the mock — that requires running
the test against the live system.

Inside the compiler's proof envelope: verification by construction.
Outside (external systems): generated tests with structural oracles.

### Relationship to decidability

Decidability and verifiability are the same structural guarantee
applied to different properties:

| Property | Mechanism | Structural source |
|---|---|---|
| Decidability | Every iteration bounded | Node.children is finite, 3 bounded primitives |
| Verifiability | Every construct testable | Types have witnesses, algebras have laws |

Both follow from the same root: `.dag` has no opaque types, no opaque
recursion, no opaque behavior. The compiler can see through all
structure. What it can see, it can prove. What it can prove, it can
verify.

## Sustainability Invariants

The governing metric for this codebase is **cost of change**: when the
language grows by one type, one expression form, or one transport, how
many files need editing? The sustainable compiler is one where that
number is 1. Every invariant below serves that goal.

### Escape Hatches (why violations keep recurring)

Each invariant below has a **structural prevention** that makes
violations unrepresentable. But violations keep recurring because the
codebase still has escape hatches — API surfaces where the wrong thing
is easy and the right thing is hard. Five escape hatches account for
the majority of all recurring violations:

| Escape hatch | What it enables | Structural fix |
|---|---|---|
| `String` return type in emitter | Hardcoded target syntax | Graph rendering — emitter walks graph, renderer produces strings |
| `node.name` field | Name-based dispatch anywhere | Delete `Node.name` — structural properties + edges only |
| `List<String>` fact storage | Copied string lists that go stale | `List<Node>` edges to definitions |
| Error sentinels in `Node` | Fabricated valid-looking error output | Typed wrappers (`InferredNode` pattern) at every boundary |
| Hand-editable generated code | Parallel implementations that diverge | Committed binary + regenerate→diff→empty CI gate |
| Raw `Node` in type rendering | Shape-based heuristic dispatch (connective/children guessing) | `TypeRendering` descriptor — precomputed, unambiguous, fail-closed |
| Adapter / bridge functions | Transitional state between old and new representations calcifies into permanent shape | Rework every consumer in the same PR as the representation change — no adapter ever lands |

Eliminating these six surfaces makes the invariants self-enforcing.
The invariants become properties of the API, not rules you have to
remember.

Active liabilities and their measured costs are tracked in the
**Open Debt** section at the bottom of this file.

The invariant headings in this document are also the canonical theme
labels for ratchets, review feedback, and queue planning. A review queue
branch must declare exactly one primary theme from this list and stop
before taking a second review item from a different theme, so CI
failures stay attributable to a single ratchet. Review queue branches
must also keep each commit strictly scoped to that invariant fix: no
unrelated helper cleanup, dead-code removal, or opportunistic
refactoring unless it is directly required for the fix to compile and
pass tests.

### No short-term solutions (this is not a production codebase)

**gunbc is not production.** There are no external users running
compiled binaries in the wild, no uptime commitments to keep, no
downstream teams whose releases depend on a stable API surface, no
breaking-change negotiations to manage, and no migration windows
that need to span multiple releases. Every refactor can be atomic.
Every API change can land in one PR that updates every caller.
Every representation change can sweep every consumer in one push.

**There is therefore no legitimate reason to introduce short-term
solutions** — adapter functions, deprecated APIs preserved
alongside their replacements, compatibility shims, feature flags
that gate half a migration, `TODO(M2): remove` markers on whole
code paths, scaffolded states with tracked dissolution triggers,
bridges between old and new data shapes, fallback code paths that
"just work" while the real fix is built.

These patterns exist in production codebases because production
codebases can't afford to break N million users in one change.
gunbc cannot break anyone. The patterns have **no defensible
motivation here** and every observed instance has calcified
instead of dissolving. The specific rules below (no bridges, no
deprecations, no parallel implementations, no fallbacks that
fabricate) are instances of this meta-principle.

**The rule:** every representation change, API change, or
refactor lands as a single atomic PR that updates every affected
consumer. If that PR is too large, the fix is to **split the
change into smaller atomic changes** — never to introduce a
transitional state with tracked removal.

**The test:** does the PR introduce any of the following?

- A new representation alongside an old one, with an adapter
  between them ("no bridges" violation)
- An old API preserved alongside its replacement, marked
  deprecated or conditionally active ("no deprecations" violation)
- Two separate implementations of the same computation
  ("no parallel implementations" violation)
- A feature flag that enables half a migration with a plan to
  flip it later (any form of gating a half-done change)
- A code path labeled "scope-bound," "dissolves in M2+,"
  "transitional," or "until X lands" — where the cleanup is
  deferred to a future commit that isn't in this PR

If yes to any, the PR is introducing a short-term solution and
violates this invariant. The fix is to do the rework in the same
PR, or to split the representation change into something smaller
that doesn't need transitional state.

**The excuse filter:** "but the refactor would be too large for
one PR" is almost always wrong. The refactoring cost that the
short-term solution is supposed to defer is exactly the
refactoring cost the solution is written to avoid — the cost
isn't reduced, just rewritten as "someone else's later problem."
And in a codebase where "later" is "whenever the current
milestone finishes," that's equivalent to "indefinitely."

If the rework genuinely cannot fit in one PR, the representation
change is the wrong size. Split the representation change into a
smaller one whose consumers can all be updated atomically — not
into "new representation now, old representation also still
here, delete the old one later."

**The honest exceptions:** there are two cases where something
that looks like a short-term solution is allowed.

1. **Emission into a target language via a declared language
   spec.** The compiler emits target-language source code, which
   is a different representation from the internal Node tree.
   That "conversion" is the whole job of the emitter (see
   "coercion = emission" in THESIS.md). Test: if the output is
   consumed by another part of the compiler, it's a bridge and
   is forbidden. If the output is target source code via a
   language spec declaration, it is emission and is allowed.
2. **Scaffolded primitive realization** (see THESIS.md §"Two
   groundings" and the `ArrowBody` enum in
   `src/v3/compiler/src/dag.rs`). Primitive Arrows may carry
   `ArrowBody::Pending` in the short window between
   substrate-shape commitment and extdeps language spec
   declarations landing. This is tracked via a
   monotonic-decrease CI ratchet. The exception is narrow and
   explicit: only primitives, only during the specific
   M1(2.5) → M3 transition, only covered by a numeric CI
   ratchet that strictly decreases.

Any other pattern that looks like a short-term solution is not
one of these two exceptions and is forbidden.

**Encountering one in existing code is an alarm signal, not a
routine cleanup.** If you spot a bridge, a deprecation marker, a
`TODO: remove in M2` on a live code path, a `_legacy` suffix, or
any other short-term-solution pattern — even in code you were
not planning to touch, even in a file adjacent to the one you
are working on, even while you were reading the code for
entirely unrelated reasons — **stop and raise it.**

The correct response is NOT:

- "This is not in my immediate task, I will note it and keep
  going."
- "I will fix it quietly in this PR while I am here."
- "Someone probably knows about it, so it is fine."
- "It is tracked in a comment already, so I can trust the
  tracking."

The correct response IS:

- **Stop your current work** long enough to understand what you
  are looking at. Read the surrounding code. Figure out when the
  short-term solution was introduced and what it was meant to
  unblock.
- **Treat it as concerning**, not as a minor nit or a
  "could-clean-up." The tone matters: describe it as a
  **structural signal that something went wrong upstream**, not
  as cosmetic debt. Normalizing the language ("just a small
  bridge," "only a minor deprecation") is how these patterns
  calcify.
- **Raise it to the reviewer or the implementer** of the code
  that contains it — even if they are a different person than
  whoever is reviewing your current PR. The bridge exists
  because someone's earlier design decision did not close a
  migration cleanly; that person is the one who should hear
  about it first.
- **Back up and assess the damage.** Does this one instance
  exist in isolation, or is it a symptom of a broader pattern?
  Are there more? What does its existence tell us about the
  current state of the subsystem? What is the root cause that
  made the short-term solution seem necessary, and does that
  root cause still apply?
- **Work on the diagnosis before the fix.** "Here is the bridge
  and here is a quick patch" is the wrong move. "Here is the
  bridge, here is what I think went wrong upstream, here is
  what I think the full cleanup scope is, and here is what I
  propose to do about it" is the right move.

The instinct to "get my own work done and flag it later" is
exactly how short-term solutions calcify. Every bridge and every
deprecation that survived to dominate a subsystem was something
that someone noticed in passing and chose not to escalate. This
invariant is not satisfied by "I personally didn't add any new
ones" — it is satisfied by "nobody saw a bridge without flagging
it." **If you see one and do not raise it, you are endorsing
its continued existence.**

What "raise it" looks like concretely:

- **In PR review:** a blocking review comment citing this
  section. Not a nit. Not a "future work" tag. A stop-sign.
- **During your own implementation:** stop the implementation
  branch, open an issue or a discussion describing the bridge
  you found, estimate what it would take to remove it, and
  decide whether to fold the removal into your current PR or
  to block your PR on its removal first. Do not keep coding
  around it and circle back later.
- **In a file you were passing through for unrelated reasons:**
  a short note: "I was reading X for unrelated work and noticed
  Y. We should discuss before I go further." Silence is not
  correct.
- **When found by a reviewer agent or an automated scan:** a
  loud failure, not an informational note. Whatever mechanism
  caught it should halt the workflow, not log and proceed.

The pattern this rule is trying to eliminate is
**normalization** — the state where bridges and deprecations
exist in the codebase and everyone walks past them. Normalization
is the terminal stage of calcification. The escalation rule is
what keeps normalization from starting.

### No duplicate representations

Every fact should be encoded in exactly one place. When two structures
represent the same information, one gets updated and the other doesn't.
The stale copy produces silently wrong behavior instead of failing.

**The test:** if changing a fact requires editing two files, one of them
is a derived copy that should be deleted or computed.

**The fix:** delete the derived representation and read from the source.
If the source isn't accessible, make it accessible — don't cache a copy
that can go stale.

**Structural prevention:** Facts are edges to definitions, not copied
strings. `kernel_types` is not `List<String> = ["Int", "Bool", ...]`
— it is `List<Node>` pointing to the actual type definition nodes. You
can't have a stale name because you don't have a name — you have a
reference. If the definition changes, the edge follows. The escape
hatch is `String`-typed fact storage; the fix is `Node`-typed
(edge-based) fact storage.

### Minimal information per interface

Every function, helper, and modeling unit should receive the minimum
information it needs — nothing more. Passing an entire collection to a
function that only inspects one element couples the function to state
it doesn't use, creates ambiguity about which instance of the state is
current, and hides the function's true dependency.

**The test:** if a function takes a parameter and immediately projects
one field or element from it, the function should take the projection
directly. `fn check_token(tokens: List<Token>)` that does `tokens |>
first` should be `fn check_token(tok: Token?)`.

**Subtle examples:**
- `peek_is_newline(tokens: List<Token>)` → only needs `Token?`
  (the current token). Passing the list creates ambiguity about
  WHICH list when the caller has multiple remaining lists in scope.
- `function_size_effects(name: String)` → only needs the function's
  structural contract, not a string key into a lookup table. The
  string forces the caller to know the name; a direct reference to
  the contract would be unambiguous.
- `classify_argument(arg_expr: Node, param_name: String, ctx: DescentContext)` →
  DescentContext bundles 7 fields, but most call sites only need 2-3.
  The bundling hides which facts the function actually depends on.

**Structural prevention:** Design function signatures from the
function's body outward: what does it READ? Pass exactly that. When a
helper only inspects a single value, take that value — not the
collection it came from, not the struct it's embedded in, not the
context that happens to carry it.

**Escape hatch:** convenience structs that bundle unrelated state
("context" objects, "environment" bags). These make it easy to pass
everything and hard to see what matters. Prefer explicit parameters
over context bundles; group into a bundle only when 3+ consumers need
the exact same set of fields.

### No case enumeration for open sets

When behavior varies by type, variant, or category, prefer a single
algorithm that walks the structure over a match/list that enumerates
known cases. Enumerated lists rot: every new case requires updating
every list, and the compiler won't tell you which lists you missed.

**The test:** if adding a new type/variant requires editing a match arm
somewhere other than the type definition itself, the code has an
enumeration that should be replaced with a structural walk.

Matching on a closed enum (`WrapperKind::List | Set | Optional | ...`)
is fine — adding a variant is a compiler error. The problem is
open-ended lists keyed by strings, type names, or error message
substrings.

**Structural prevention:** Data tables loaded at pipeline startup,
not match arms in code. The `SyntaxSpec` pattern: keywords, operators,
and item forms are data in `.dag` files. `parse_item` reads the data
— there are no match arms to add. The same pattern applies to method
dispatch (algebra types in `std/algebra.dag`), type dispatch (structural
properties on nodes), and container ops (templates in `LanguageSpec`).
The escape hatch is `if name == "..."` branches; the fix is a data
lookup where the data is the `.dag` source itself.

### No fallbacks that fabricate

Every code path either succeeds fully or fails with a clear error.
No silent degradation: no `.ok()` that swallows errors, no `continue`
that silently drops work, no fallback defaults that produce
valid-looking but wrong output. If a function cannot complete its job,
it must return `Err`.

Fabrication fallbacks are the mechanism by which duplicate
representations and missed enumerations become invisible. They convert
hard failures into silent wrong behavior.

Sample: ownership should not compile to
`Rc::try_unwrap(x).unwrap_or_else(|rc| (*rc).clone())`. Either the
compiler proves a single semantic consumer and emits the move, or it
surfaces that the proof is missing. The clone branch is a fallback,
even if it preserves correctness.

**Structural prevention:** Typed boundaries that can't represent error
states. `InferredNode = Resolved { node } | CompilerError { ... }`
already does this for inference — you can't accidentally treat an error
as a valid type. The same pattern applies to the emit boundary: emit
receives a type that can't represent error-contaminated nodes. If
inference failed, the node doesn't reach emit — not because a gate
checked for errors, but because the type system makes it
unrepresentable. The escape hatch is `String`/`Node` types that can
carry error sentinels (`"<error:...>"`, `Dynamic`, `LitNull`); the fix
is wrapper types where the error case is structurally distinct.

### Heuristics indicate lost structure

Heuristics are a code smell in compiler and runtime logic. String
matching, score-based classification, best-effort guessing, "close
enough" defaults, and inference from naming conventions usually mean
the pipeline has already thrown away information that should have been
structural.

**The principle:** do not tune the heuristic first. Trace the pipeline
upstream until you find where the needed fact stopped being explicit,
then restore that structure as close to the source as practical.

**The test:** if a code path has to guess from strings, partial shapes,
error text, or naming patterns, the real bug is upstream information
loss. The preferred fix is to carry the missing fact in the type/IR/API
boundary instead of improving the guess.

**The fix:** push structure earlier in the pipeline so the downstream
stage can make an exact decision.

**Structural prevention:** Graph rendering. The emitter walks the
typed graph and invokes the language renderer for each structural
pattern. The emitter never produces strings — it matches patterns
(product, coproduct, sequence, etc.) and the renderer converts them
to target text using `LanguageSpec`. The emitter cannot produce
`"Rc<Vec<...>>"` because it doesn't produce strings at all. The
escape hatch is string concatenation in the emitter; the fix is an
emit stage that walks the graph and delegates to the renderer. This
is the highest-leverage single change — it structurally prevents
~60% of all recurring violations (hardcoded target syntax, Rc
wrapping, container patterns, type name dispatch, method rendering).

### Design commitments must name the substrate target

**Design isn't done until every claimed non-commitment is proven.**
When a design doc claims "no substrate change needed for feature X,"
the doc must explicitly name the substrate element(s) that express
X's semantic. If it can't name them, there IS a substrate change and
the doc is smuggling it — and the implementation will surface it as
one of four routes, all of which fail review later at higher cost
than fixing the design now.

**The four smuggling routes** (the option space when a feature
requires substrate expression and the design refuses to name it):

1. **Synthesized declarations at lowering time.** The lowerer
   allocates fresh declarations per-occurrence (or deduplicated
   per-unique-shape) to carry the feature's semantic. The new
   declarations are compiler artifacts mixed into the declaration
   table, creating layering opacity for any lens that walks it.
2. **Flat coproduct growth on existing substrate variants.** An
   existing coproduct gains new variants that encode the feature,
   often as a flat expansion of an orthogonal dimensional
   distinction (e.g., `{A, B}` becomes `{A_bare, A_with, B_bare,
   B_with}`). Fails 4-pattern check at Pattern 4 (dimensional).
3. **Surface state preserved in the substrate.** Parse-level
   shapes (`SurfaceExpr::Path`, raw tokens, unlowered AST nodes)
   survive into the substrate so downstream consumers re-parse
   them at their boundary. Breaks the substrate/surface
   separation and makes "lenses walk a Dag, not a parse tree"
   structurally impossible.
4. **Parallel hand-maintained representation.** The feature's
   semantic is tracked in a second data structure outside the
   substrate, kept in sync by convention. Diverges immediately;
   the `feedback_parallel_representation_debt` concern applies.

**The review rule:** for every design doc that claims "no substrate
change needed," the reviewer must force the author to name the
existing substrate element(s) that express the feature's semantic.
"It lowers to X" is the answer; X must be a specific, existing
substrate form. If the author cannot produce X, the claim is
unverified and the design is not done.

**The check against the codebase:** thesis-level claims about
what's expressible in the substrate drift from implementation,
especially around categorical vocabulary that reads as intuitive
but isn't wired in. Before recommending a design option that
depends on an existing substrate form, verify that form exists by
reading the current code (the file and line number), not by
reciting the thesis. Reading the thesis out loud is not the same
as reading `dsl/std/algebra.dag:276-302`.

**Background:** this rule was added after PR #453, where Prereqs
1 and 2 of the substrate reflection slate each claimed "pure
lowering extension, no substrate change needed" in §11 of
`docs/substrate-reflection-design.md` without naming the
substrate target. The implementation hit all four smuggling
routes simultaneously: synthesized `FieldAccessor` accessor
declarations (route 1), flat 4-way `BranchPattern` growth
(route 2), scattered parallel representations between
`substrate.dag` and `dag.rs` (route 4), and near-adoption of
preserving `SurfaceExpr::Path` through to emit (route 3,
considered and rejected at implementation time). The design
gap was the review miss: each prereq deserved the
§3.5-level option walk Prereq 0 received. §3.6 and §3.7 now
codify those walks; this invariant generalizes the lesson
into a ratchet on all future design-doc reviews.

**Ratchet:** a design doc that adds a prereq / feature scope
saying "no substrate change needed" must, in the same doc,
either (a) name the existing substrate element that expresses
the feature (with file and line citation), or (b) provide a
§3.5-style option walk that enumerates the substrate
commitments and commits to one. A claimed non-commitment
without one of these two receipts is a blocker.

### Scaffold boundaries

Every substrate scaffold variant must have an **explicit unreachability
gate** that rejects it for user-range declarations before the compile
boundary returns `Ok`. A scaffold is any substrate form introduced to
accommodate a parser or lowering limitation — typically a variant
that says "this position has a fact the compiler can't yet validate"
(e.g., `ArrowBody::Unparsed(SourceSpan)` for block-body fns whose
body the parser cannot lower). Scaffolds are load-bearing for
std/bootstrap files whose richer content outpaces the current
parser surface. They are never acceptable in user code: letting a
user-range scaffold survive means ordinary programs can ship an
opaque body that the compiler never validates, violating
fail-closed static grounding.

**The principle:** a new substrate variant added to accommodate a
compiler limitation is not done when the variant is defined and
populated. It is done when the code that rejects it in user code
also lands. "Deliberate, documented, with a dissolution trigger"
is necessary but not sufficient — a tracked scaffold that applies
to user code is still a fail-open leak.

**The diagnostic:** ask "where does this scaffold become
unreachable?" for every scaffold variant. If the answer is "at
bootstrap time only, everything else tolerates it," the scaffold
is unbounded. If the answer is "nowhere — it's a general
placeholder," the variant is not a scaffold at all; it's load-
bearing substrate and deserves a full 4-pattern check plus
dissolution trigger, not scaffold status.

**The test:** compile a minimal user program that deliberately
produces the scaffold state (e.g., `fn foo(x: Int) -> Int { junk }`
for `ArrowBody::Unparsed`, `data foo: Int = { junk }` for
`ValueBody::Unparsed`). If `compile_to_dag` returns `Ok`, the
boundary is missing.

**The fix:** add a post-lowering sweep that walks declarations
at id `>= user_start` (the range of user-lowered declarations
after bootstrap) and emits a fail-closed diagnostic for each
scaffold variant found. Bootstrap-range declarations stay
tolerated by design. The sweep runs alongside
`resolve_pending_identifiers_strict` as a sibling gate.

**Structural prevention:** a per-scaffold regression test that
pins the boundary. The test compiles the minimal user-code
reproducer and asserts `compile_to_dag` returns `Err`. Adding a
new scaffold variant without such a test is a violation; the
test IS the boundary. In v3's test suite these are the
`m18_r14_user_*_is_rejected` tests for the two R9/M1(2.7)
scaffold variants.

**Ratchet:** a PR that introduces a new substrate variant (new
enum arm in `ArrowBody`, `ValueBody`, `BranchPattern`,
`AtomPayload`, etc.) must, in the same PR, either (a) prove the
variant is terminal substrate — 4-pattern check passes, no
dissolution trigger — or (b) prove the variant is a bounded
scaffold — unreachability gate landed, regression test landed,
dissolution trigger named. A new variant without one of these
two receipts is a blocker.

**Background:** this invariant was codified after R14 of PR #445,
when a reviewer found that `ArrowBody::Unparsed` (introduced in
M1(2.7) R9 for std/bootstrap block-body fns) had no user-range
boundary. User code could ship `fn foo(x: Int) -> Int { junk }`
and it compiled cleanly — the scaffold was "deliberate,
documented, with a dissolution trigger" at the variant level but
had no landing gate in lowering. `ValueBody::Unparsed` had the
same shape for data items. R14 added the sweep
(`reject_user_unparsed_scaffolds` in `src/v3/compiler/src/lower.rs`)
as the canonical form of the fix; this invariant generalizes that
fix into a ratchet on all future scaffold additions.

### No parallel implementations

When the same computation exists in two forms (e.g., an AST interpreter
AND a resolved DAG op), they will diverge as the language evolves.
Every new expression form must be implemented in both, and the one that
lags will be masked by a fallback (see above).

**The test:** if a code path exists only to provide a result that
another code path also produces, one of them should be deleted.

**Structural prevention:** Single source + derivation. Stage0 is
generated from `.dag` source — never hand-edited. The regeneration
script is the only path from `.dag` to `.rs`. Committed binary approach
means CI verifies regenerate → diff → empty. The escape hatch is
hand-editing generated code; the fix is making regeneration the only
write path and failing CI if the generated output doesn't match.

### No bridges

A **bridge** is an adapter function, helper module, or translation
layer that exists purely to convert one representation of a fact
into another representation of the same fact. Bridges are introduced
when a refactor lands the *new* representation but can't yet touch
every *old* consumer, so an adapter is added "temporarily" to keep
the old consumers working while the rest of the migration is
tracked as follow-up work.

**Bridges are forbidden. Do not introduce them, no matter how
well-tracked. And if you find one already in the codebase — even
in a file you were passing through — do not silently route
around it. Raise it as an alarm signal per §"No short-term
solutions."**

The refactoring cost that would make the bridge "temporary" is
exactly the refactoring cost that the bridge is supposed to defer.
The bridge doesn't reduce the cost — it just rewrites it as
"someone else's later problem," and tracked bridges calcify because
every downstream consumer learns the adapter shape, not the new
representation. By the time the dissolution trigger fires, removing
the bridge means reworking every consumer AND every consumer
downstream of those consumers that inherited the adapter's
assumptions. The debt compounds.

**Historical example (2026-04-14):** v3's `declaration_to_type_shape`
was introduced as a "localized" adapter from `DeclarationId` to
`TypeShape::Primitive(Prim)` because the substrate rework landed
a rich declaration table but didn't refactor port-level `TypeShape`.
The function matched declaration names against a hardcoded string
list (`"Int" | "Int64" | "Word64" | "Word32" | ...` → `Prim::Int`)
and was tracked as "scope-bound, dissolves in M2." It violated
three invariants at once: no duplicate representations (DeclarationId
+ Prim for the same type identity), no name-based dispatch (string
match on declaration names), and facts flow forward (the rich
declaration identity was collapsed to a coarse tag at the boundary).
Had it survived, every M1(3)+ consumer of `TypeShape` would have
learned the Prim-tagged shape instead of the declaration-carrying
shape, and the M2 rework would have had to edit every consumer
plus the adapter plus any new consumers that appeared in between.

**The test:** does the change introduce a function whose purpose is
to translate between two representations of the same fact? Signs
to look for:

- Function name or docstring matches `*_to_*`, `convert_*`,
  `adapt_*`, `bridge_*`, `as_*`.
- Body does a match on names, indices, or tags that came from one
  representation and produces a corresponding value in another.
- Comment says "localized," "scope-bound," "dissolves in M2+," or
  "the last bridge."
- Caller code "just needs" the adapter to unblock work in one area
  without touching another.

If any of these apply, the change is introducing a bridge. Stop.

**The rule:** the representation change and every consumer update
must land in the same PR. If that PR is too large, split the
representation change into a smaller one that doesn't require
adapters — but do not split it into "new representation now, rework
later." The only acceptable split is the one that keeps every
consumer consistent at every commit boundary.

**The fix when you've already written one:** back out the adapter
and the representation change together. Rework the representation
change into something that every consumer can adopt in one push.
Do not merge an adapter and track its removal — track the smaller
representation change instead.

**Structural prevention (future):** a CI audit on every M1+ PR
grep-matches function signatures and docstrings for the adapter
pattern above and fails the build if new matches appear. Until
that audit exists, this invariant is enforced by code review —
any PR reviewer can veto an adapter with a reference to this
section.

**Exception:** there is exactly one boundary where an adapter is
unavoidable: emission into a target language. The emitter converts
Node trees into target source code via a language spec — that's
`coercion = emission`, and the "conversion" is the whole point of
the emitter. But the emitter is not a bridge under this invariant
because (a) the output is in a different target world entirely, not
in the compiler's own representation, and (b) it is driven by a
declared language spec, not a compiler-internal adapter function.
Test: if the adapter's output is consumed by another part of the
compiler (not by a target world), it is a bridge and is forbidden.
If the output is target source code produced via a declared
language spec, it is emission and is allowed.

### No deprecations

A **deprecation** is any pattern that keeps an old API, data
structure, or representation alive alongside its replacement with
the intent to remove it "later." Concrete forms: `@deprecated`
annotations, `_v1` / `_legacy` / `_old` suffixes next to fresh
names, feature flags that toggle between "new" and "old"
behaviors, type aliases that re-export old names from new
modules, `TODO: delete this function when X lands` comments,
parallel function bodies selected by a runtime flag.

**Deprecations are forbidden. Do not introduce them. And if you
find one already in the codebase — even in a file you were
passing through — do not silently work around it. Raise it as
an alarm signal per §"No short-term solutions."** See that
section for the meta-principle and the escalation procedure.
Production codebases tolerate deprecations because they can't
afford to break external consumers in one release, and gunbc
has no external consumers. The refactoring cost a deprecation
defers is exactly the refactoring cost the deprecation was
written to avoid.

Deprecations are the close cousin of bridges: a **bridge**
translates between two representations of a fact that exist
simultaneously; a **deprecation** keeps an old callable/type
alive alongside its replacement so callers have time to
migrate. The failure mode is identical — the old form calcifies
because every new caller learns the presence of both
alternatives and becomes dependent on whichever one was
convenient at the time of writing. By the time the "delete old
form" commit lands, reworking every consumer has become a
bigger task than reworking every consumer at the introduction
time would have been.

**The test:** does the change introduce two versions of the
same function, type, API, or module name, where one is labeled
or intended as the "new" one and the other is labeled or
intended as the "old" one? Signs to look for:

- `@deprecated`, `#[deprecated]`, `// DEPRECATED`, or similar
  annotations.
- Names with `_v1` / `_v2` / `_old` / `_legacy` / `_new` suffixes.
- Re-exports of the form `pub use new_module::NewName as OldName`.
- Function signatures that take a boolean flag named `use_new`,
  `legacy_mode`, `v2_dispatch`, or similar.
- Match arms labeled `// old path` and `// new path` in the same
  function.
- Comments saying "keep this until X migrates" or "delete after
  M3" on callable code (not comments on data that's actively
  being transitioned via a ratchet).

**The rule:** when you rename a function, change an API shape,
or replace a type, every caller updates in the same PR. There is
no "introduce the new form, migrate callers over N PRs, delete
the old form at the end." There is only "introduce the new
form with every caller already using it, in one PR, with the old
form deleted."

**The fix when you've already written one:** back out the
deprecation and do the rename/replacement as a single atomic
change. If the rename touches many callers, the refactor is big,
but it is exactly the refactor you were going to do eventually
— doing it now is cheaper than doing it later with additional
consumers that learned the deprecated form in between.

**The fix when the rename genuinely spans multiple independent
subsystems:** the representation or API change is the wrong
size. Split it into smaller changes where each rename is
atomic within its subsystem. Do not split by "new name first,
old name deleted later" — split by "these five callers get the
new name in PR A, these four get it in PR B, and the PRs are
independent because A's callers don't import B's."

**Structural prevention (future):** CI audit on every PR that
grep-matches the deprecation signals above (annotations,
suffixes, naming patterns, comment phrases). Zero matches
required. Until the audit lands, enforced by code review —
any reviewer can veto with a reference to this section.

**Exception:** versioned external protocols (wire formats,
persisted data schemas with existing content on disk). These
are outside the closed system — a change to them can't be
atomic because the other side is outside gunbc's control.
Protocol versioning is the one place where keeping an old form
alive alongside a new form is the honest answer. Test: if the
"old" thing is consumed only by gunbc itself, it's a
deprecation and is forbidden. If the "old" thing is consumed by
an external protocol peer, by a file format with existing data
on disk, or by a declared language spec target, it's protocol
versioning and is allowed.

### Layer opacity

The whole point of gunbc's compositional modeling is that **layers
compose such that below-boundary changes are invisible to
consumers**. This is the thesis's load-bearing claim — see
`THESIS.md` §"Compositional layering: below-boundary opacity by
construction" for the motivation. This invariant is its CI-gate
enforcement: **no consumer of the substrate may observe a
below-boundary identifier by name**. Consumers read across layer
boundaries, and below-boundary identifiers are unnameable to them
by construction.

**Below-boundary identifiers are forbidden to appear as hardcoded
string literals in compiler source code** (outside of diagnostic
display paths). This includes:

- User-facing type names from `dsl/std/` (`"Int"`, `"Bool"`,
  `"Float"`, `"String"`, `"List"`, `"OrderedRing"`, `"FreeMonoid"`,
  every algebra name, every primitive alias)
- User-facing field names from `dsl/std/` algebras (`"add"`,
  `"sub"`, `"mul"`, `"lt"`, `"eq"`, every algebra-field name)
- User-facing variant labels from `dsl/std/` sum types (`"True"`,
  `"False"`, every Disj variant name)
- Canonical operator symbols when used as semantic discriminators
  (`"+"`, `"-"`, `"*"`, `"=="`, `"<"`, ...)

Hardcoding any of these in compiler source code makes the
identifier observable to the compiler's decision logic, which
means renaming the identifier changes compiler behavior — the
consumer is reaching below the layer boundary to read a name.
That is the leak. Every use of such a hardcoded string is a layer
violation regardless of whether the code currently "works."

**The rename test.** The cheapest empirical check of layer
opacity is the rename test: pick a below-boundary identifier,
rename it everywhere in its declaring module (and any `import`
statements that explicitly reference it), recompile a test
consumer, and compare the generated output against the baseline.

- If the generated output is **byte-identical**: the layer was
  opaque. Layer opacity holds for this identifier.
- If the generated output **differs structurally** (different
  types, different wrappers, different dispatch): the compiler
  was reading the identifier by name somewhere. Layer opacity is
  violated. Find the violation and fix it.
- If compilation **fails**: there is a compiler-internal
  dependency on the identifier that has no opaque resolution
  path. Same verdict — find and fix.

**Historical example (2026-04-15).** The weather example in
`dsl/examples/weather/` was compiled to Rust with v2, then the
`Float` declaration chain in `dsl/std/float.dag` was edited three
ways to probe layering. Inserting an intermediate alias
(`Float → PreciseScalar → Float64`) produced byte-identical output.
Renaming internal layers below the boundary (`Float64 →
BinaryFloat64`) produced byte-identical output. But renaming the
boundary identifier itself (`Float → FloatingPoint`) produced
structurally different generated Rust: fields became
`Box<FloatingPoint>` instead of `f64`, `Temperature` lost its
`Copy` derive, every use site gained `Rc<...>` wrappers. The
leak: v2's inference and emission have a fast path for types whose
canonical name appears in `kernel_type_set` (a string-keyed map
in `dsl/std/types.dag`, mirrored into
`src/v2/stage0/src/std_types.rs`), and a slow path for everything
else. Renaming a primitive moves it from fast path to slow path.
The leak is tracked in v2 as "Part B pending" — when inference
resolves methods from type fields structurally, `kernel_type_set`
dissolves. Until then, the v2 compiler fails the rename test on
any of the eight names in that table.

v3 PR-B's `emit_rust.rs` reproduced the same leak at the emit
layer: `index.lookup("Int", "")`, `index.lookup("Bool", "")`,
`match label.as_str() { "True" => ..., "False" => ... }`. The
mechanism is identical — string-keyed dispatch against below-
boundary identifiers — even though the compiler had spent 14
review rounds removing this pattern elsewhere. The rename test
would have caught it at PR-B introduction time had it been an
enforced gate.

**The rule:** any compiler source file that contains a hardcoded
string literal matching a below-boundary identifier from `dsl/std/`
is a layer violation and must be reworked to dispatch by
`DeclarationId` instead. The substitution mechanism is the same
as every other bridge dissolution: replace the string key with a
typed edge, walk the substrate to resolve the edge at lookup
time, and let renaming propagate through DeclarationId identity
rather than through name matching.

**The fix when you've already written one:** extend the upstream
data structure (substrate field, language-spec schema, or fact
table) to carry the DeclarationId directly instead of a string.
Resolve the identifier at parse/lower time when the name is
known, carry the DeclarationId forward, and dispatch on ID. The
specific shapes this takes:

- **Language spec realizations:** instead of `target_name: String`,
  use `for: DeclarationId`. Walk the realization declaration to
  resolve its `for` field; the resolved `DeclarationId` is the
  identity key. This is the v3 class-5 gap #6 dissolution (extend
  `ValueBody::Structural` to support `LiteralBits::DeclarationRef`).
- **Canonical primitive rosters:** instead of
  `kernel_type_set: Map<String, Bool>`, use
  `kernel_types: List<Declaration>`. The list carries typed
  references to the primitive declarations; any consumer that
  needs "is this type a kernel primitive?" does DeclarationId
  containment rather than string lookup. This is v2's Part B
  dissolution of `kernel_type_set`.
- **Variant dispatch:** instead of `match label.as_str() { "True"
  => ..., "False" => ... }`, match on `BranchPattern::ResolvedVariant(DeclarationId)`
  and compare the variant's parent Disj against the scrutinee type
  structurally. Variant identity is a DeclarationId, not a string.

**Structural enforcement: the layer-opacity lens.**

Layer opacity is a structural query over a DAG and the natural
enforcement mechanism is a reader lens, not a grep gate. The
lens takes a `BoundarySpec` describing which declarations count
as below-boundary for the analysis, walks the DAG, and returns a
list of violations — every consumer site where a below-boundary
identifier is read by name instead of by typed reference. Lenses
are the thesis's intended extensibility point for invariant
enforcement (see `THESIS.md` §"Compositional layering" and the
lens library sketched in `docs/lens-library-design.md`).

```rust
// src/v3/compiler/src/lens_layer_opacity.rs
pub struct LayerOpacityLens;

pub struct BoundarySpec {
    /// Which declarations count as below-boundary for this analysis.
    /// Compiler-level application: all declarations from dsl/std/.
    /// User-level application: all declarations inside a named layer.
    pub below_boundary: Vec<DeclarationId>,
}

pub struct Violation {
    pub location: SourceSpan,
    pub identifier: String,
    pub origin: DeclarationId,
    pub consumer_kind: ConsumerKind,
}

pub enum ConsumerKind {
    TransformDispatch,
    VariantStringMatch,
    StringLiteral,
}

impl LayerOpacityLens {
    pub fn query(dag: &Dag, boundary: &BoundarySpec) -> Vec<Violation> {
        // Pure reader. Walk every Transform, Branch pattern, and
        // Value literal. For each, flag cases that observe a
        // below-boundary identifier by name.
    }
}
```

**Opt-in application.** The lens is opt-in via a `lens_config.dag`
file (or equivalent) that declares which lenses apply to which
source trees and what boundary each application uses:

```
module lens_config

data compiler_layer_opacity_spec: LensApplication = {
  lens: "layer_opacity"
  applies_to: ["src/v3/compiler/src/**/*.rs",
               "src/v3/compiler/src/**/*.dag"]
  boundary: "all declarations in dsl/std/**"
  severity: "error"
}

// A user opting into the same discipline for their own domain:
data my_domain_opacity_spec: LensApplication = {
  lens: "layer_opacity"
  applies_to: ["dsl/examples/my_app/app_code/**"]
  boundary: "all declarations in dsl/examples/my_app/rest/**"
  severity: "error"
}
```

CI runs every configured lens application on every PR that touches
its declared scope. A violation with `severity: error` blocks the
PR. The lens is a CI gate because it's a structural query, not
because it's a text search.

**Why the lens is strictly better than the grep gate we originally
proposed:**

- **Structural, not lexical.** The lens reads the substrate
  (Transform targets, pattern nodes, value literals), not source
  text. False positives from comments, documentation, or
  diagnostic strings don't exist.
- **Opt-in per application.** Multiple applications can coexist
  (compiler source, individual user projects, library boundaries)
  with different `BoundarySpec`s. A grep pattern would need
  project-specific pattern lists maintained by hand.
- **Structured output.** Each violation carries a location,
  identifier, origin, and consumer kind. Reviewers can triage by
  severity, consumer type, or origin layer. Grep output is a
  flat list.
- **Thesis-native.** The lens uses existing v3 infrastructure
  (reader lens over Dag, same shape as `lens_provenance`,
  `lens_depth`, `lens_cost`). No new machinery.
- **Generalizes to other invariants.** The lens framework
  extends to every testable invariant over a DAG — scaffold
  boundaries, fail-closed discipline, structural duplication,
  epistemic grounding termination, and more. See
  `docs/lens-library-design.md` for the full lens library plan.

**The rename test as regression safety net.** Even with the lens
in place, the rename test remains a valuable regression check:
pick a below-boundary identifier, rename it, recompile, diff the
output byte-by-byte. If the lens ever misses a violation, the
rename test catches it at the behavioral level. The two
mechanisms compose — the lens catches violations at introduction
time, the rename test catches any that sneak past the lens.

**The long-term structural target: Rust type-level enforcement.**
Even the lens is a reader over a substrate that currently allows
the violation to exist. The endgame is to make the violation
structurally impossible via Rust types: a `DisplayName` type
without `Eq`/`Ord`/`Hash` so dispatching on a below-boundary name
fails to compile at the Rust level. That's a bigger refactor
(touches every `decl.name` read) and is tracked as compiler-
architecture work rather than a per-file lens application. The
enforcement progression is: grep → test → lens → Rust types, each
step strengthening the invariant from "detected after the fact"
toward "impossible to write."

**Exception 1: diagnostic display paths.** The compiler's
diagnostic layer legitimately mentions user-facing names when
producing error messages — "unknown type `Int`" is a useful error
even though "Int" appears as a literal. Diagnostic display is an
exception because it's emitting text for the user, not making a
compiler decision. Test: if the string literal flows into a
diagnostic message, it's display. If it flows into a
`match`/`if`/`lookup` that determines compiler behavior, it's
dispatch and is forbidden.

**Exception 2: tracked scaffolds with active dissolution.** Some
scaffolds temporarily need hardcoded names during a transition.
The v2 `kernel_type_set` is the canonical example — it exists as
a documented scaffold waiting for Part B. Such scaffolds are
allowed only if (a) they have an active `INVARIANTS.md`
§"Scaffold boundaries" receipt with a numeric ratchet or explicit
dissolution trigger, (b) the trigger is documented inline in the
scaffold, and (c) the scaffold count is tracked and monotonically
decreasing across milestones. Tracked scaffolds do not exempt the
lens — they appear as violations in the lens output and require
an inline `// lens:layer_opacity_exception` comment linking to
the dissolution receipt. The lens cross-references its violations
against the receipt file; violations without receipts are errors,
violations with receipts are warnings that surface the scaffold
count for the monotonic-decrease ratchet.

**Exception 3: substrate-internal enum variants that are not
user-renameable.** Rust enum variants on compiler-internal types
(`Behavior::Bind`, `TransformTarget::Callable`, `ArrowBody::Pending`)
are not in `dsl/std/` and cannot be renamed from user code. String
or enum-pattern matches on these are NOT layer violations because
the names are compiler-internal, not below-boundary. Test: if the
name appears in a `.dag` source file anywhere in `dsl/`, it's
below-boundary and the `lens_layer_opacity` lens applies (with
`boundary = dsl/std/**` or equivalent); if it appears only in
`src/v3/compiler/src/*.rs` as an enum discriminant, it's
compiler-internal and exempt from the lens. (This exception will itself dissolve
when `project_node_to_std` moves Node and L1 behaviors into std/
as structural declarations — at that point the behavior names
become below-boundary and the lens applies to them too.)

**Relationship to other invariants:**

- **No bridges** forbids adapter functions between two
  representations of the same fact. Layer opacity is a specific
  class of bridge: one where the adapter is "match on a string
  from below-boundary data and produce a compiler-internal
  dispatch decision." Every string-dispatch leak is also a
  no-bridges violation; the two invariants catch the same
  failures from different angles.
- **Boundary sufficiency** says stage boundaries must carry
  enough structural data that downstream stages don't need
  name-proxy reads. Layer opacity is a specific diagnostic for
  boundary insufficiency: when a consumer reads a name, the
  upstream boundary didn't carry the structural fact the
  consumer needed.
- **Emission is translation, not decision-making** says the
  emitter must not make target-language decisions via
  hardcoded logic. Layer opacity generalizes this from emission
  to every consumer (lens, interpreter, future tooling); the
  emitter is the most common offender but not the only one.

**Operational commitment.** Every consumer of the substrate that
crosses a layer boundary must pass the layer-opacity lens for
every identifier below that boundary, AND must pass the rename
test as a regression check. The lens is applied to `src/v3/
compiler/src/` by default; user projects opt in via their own
`lens_config.dag`. New consumers are audited at introduction
time (the lens runs as a CI gate); existing consumers are
audited whenever their upstream layer gains new identifiers
(i.e., whenever `dsl/std/` grows — the lens's `BoundarySpec`
auto-updates by walking the canonical std/ declaration set). The
lens is the primary enforcement; the rename test is a regression
smoke test; the long-term goal (`DisplayName` type refactor) is
the structural target that makes violations impossible to
construct. Together they form the enforcement progression from
detection toward prevention.

### Semantic authority after lowering

A sibling invariant to §"Layer opacity," pinned as a direct
response to the PR #445 meta-review (2026-04-15). Where layer
opacity says "consumers cannot observe below-boundary names,"
this invariant says the stronger thing about WHICH representation
IS authoritative after the lowering boundary:

**After lowering, declaration identity (`DeclarationId`) is the
ONLY semantic authority.** No name recovery. No parallel
`TypeShape` kernel. No bridge adapters from names back to
declarations. No "parsed façade" that carries strings alongside
the declaration table. Every semantic decision downstream of
lowering reads `Dag.declarations` directly via `DeclarationId`
and makes its choices on the structure that table carries.

**Why this invariant is necessary separately from layer opacity.**
Layer opacity forbids a specific pattern (consumers reading
below-boundary identifiers by name). Semantic authority forbids
a broader pattern: the existence of ANY secondary representation
that the compiler consults as an authority after lowering. A
compiler can satisfy layer opacity locally — never grep a name —
while still maintaining a parallel `TypeShape` that's really a
shadow copy of `Dag.declarations` with different identity. The
layer-opacity lens would not catch that parallel authority
because no individual consumer is reading a below-boundary name;
the authority is just duplicated. Semantic authority closes that
gap.

**Historical motivation.** The PR #445 meta-review identified
the pattern by walking 16 review events across ~17 hours and
finding the SAME root question surviving every round:

> Is `Dag.declarations` the consumed authority, or only a parsed
> façade beside name recovery, bootstrap special cases, and a
> parallel type kernel?

The wording changed between rounds — "bootstrap fixtures plus
`declaration_to_type_shape`" → "parallel `TypeShape` authority"
→ "global-name resolver" → "opaque `ExternalRealization` arm"
→ "imports parsed then discarded" → "lower→infer boundary still
too weak" — but the disease didn't change. Each round named a
new surface of the same underlying violation: the lowering
boundary wasn't settled as a hard authority commitment, so every
downstream stage ended up consulting some secondary
representation to fill in gaps.

The review loop was doing real work (each round closed a real
bug) but accumulating debt faster than it dissolved it. The
meta-review's verdict: **PAUSE_AND_REGROUP and graduate the
recurring pattern into an invariant instead of patching its
next instance.** This invariant is that graduation.

**The rule:** after `lower_bodies_phase` + `resolve_pending_
identifiers` complete, every downstream stage (`infer`, `lens_*`,
`emit_*`, the interpreter, any future analysis tool) reads facts
about types, functions, variants, operators, and realizations
**only** from `Dag.declarations` via `DeclarationId` lookups and
structural walks. Specifically forbidden:

- **Parallel type representations.** A `TypeShape` struct that
  carries any information that isn't reachable from
  `Dag.declarations` via its contained `DeclarationId`. The
  `declaration_to_type_shape` adapter that PR #445 introduced
  and then removed is the canonical counterexample.
- **Global name recovery.** Any function that takes a `String`
  and returns a `DeclarationId` as a post-lowering recovery
  path. Name-based lookup is a parse-time or bootstrap-time
  activity; after lowering, names exist only in diagnostic
  display paths.
- **Bootstrap-only special cases in post-lowering code.** Any
  `if current_range < bootstrap_range { special_behavior }`
  branch in a downstream stage. Bootstrap-range declarations
  may carry scaffolded state (per §"Layer opacity" exception 2),
  but downstream stages do not need to distinguish them — they
  consume the same `Dag.declarations` table and the scaffolded
  state is either handled uniformly or fails closed.
- **"Parsed façade" patterns.** Carrying a `SurfaceModule` or
  pre-lowering representation past the lowering boundary so
  downstream stages can fall back to re-reading the parse
  tree. The lowering boundary is the transition point where
  the parse tree's information must be fully absorbed into
  `Dag.declarations`.
- **Secondary authorities for realization.** An
  `ExternalRealization` target whose body is an opaque Rust
  handle or a compiler-internal lookup table instead of a
  declaration-table lookup. Realizations are declarations; they
  participate in `Dag.declarations` like any other declaration.

**The test.** This invariant has two mechanical checks, both
lens-shaped:

1. **Post-lowering name-reference audit.** A lens that walks
   every Rust function in `src/v3/compiler/src/` (or equivalent
   source tree), identifies the subset that runs after
   `lower_bodies_phase`, and flags any that read a `String`
   field from a declaration as input to a dispatch decision.
   The subset is determined structurally: functions called from
   `infer`, `lens_*`, `emit_*`, `interp`, etc. Violations are
   sites where post-lowering code reads a name, not a
   DeclarationId. This is a refinement of `lens_layer_opacity`
   scoped to post-lowering consumers.

2. **Parallel authority audit.** A lens that walks every struct
   definition in `dag.rs` and flags any that contains fields
   with names like `name: String` or `kind: String` that appear
   alongside a `DeclarationId`. The heuristic catches the
   declaration-shadow-copy pattern. Violations are sites where
   the compiler is carrying a name through the post-lowering
   data flow instead of relying on `DeclarationId` alone.

The two lenses together form the enforcement mechanism. Both are
in scope for the initial lens library (see
`docs/lens-library-design.md`) as post-MVP additions — the three
lenses currently specified (unused_parameters,
structural_duplicates, layer_opacity) are the starting point;
semantic authority lenses extend the pattern once the framework
is in place.

**The rule for consumers writing new downstream code:**

> If you are writing code that runs after `lower_bodies_phase`
> and you find yourself wanting to read a `String` field from a
> declaration, STOP. Either (a) the fact you need is available
> as a typed edge from the declaration you already have the
> `DeclarationId` for, or (b) the declaration table is missing
> a fact that should be there and you need to add the typed
> edge upstream before writing the consumer. There is no third
> option.

**The fix when you find an existing violation.** Do not patch
the consumer. Do not add a name-recovery helper. Do not add a
parallel table for the one fact you need. Instead:

1. Identify the fact the consumer wants as a declaration field.
2. Extend the declaration table to carry the fact as a typed
   edge (usually a `DeclarationId` field or a `Vec<DeclarationId>`
   child list).
3. Update `lower_bodies_phase` and `resolve_pending_identifiers`
   to populate the field.
4. Rewrite the consumer to read the typed edge.
5. The old name-reading path is deleted, not gated, in the same
   PR.

This is the standard bridge-dissolution pattern applied
specifically to the lowering boundary. Same rules as §"No
bridges," with the extra specificity that the bridge is always
between `String` names and `DeclarationId` identity.

**Historical examples of the violation:**

- `declaration_to_type_shape` adapter in PR #445's early rounds
  (dissolved in R5).
- Bootstrap-range vs user-range distinction as a post-lowering
  dispatch criterion (still under review in R13/R14).
- `kernel_type_set` as a post-lowering authority for
  "is-primitive" checks (v2, tracked as Part B).
- PR-B's `emit_rust.rs` `lookup("Int", "")` pattern (the Rust-
  level version of the same leak at the emit boundary, flagged
  in the 2026-04-15 layer-opacity review).

Each of these is the same pattern in a different layer. The
meta-review's insight: **the invariant would have prevented each
of these at introduction time** if it had existed from M0. The
review loop kept finding new instances because no general rule
named the class.

**Relationship to other invariants:**

- **Layer opacity** is a local consequence. Every semantic-
  authority violation is also a layer-opacity violation at the
  post-lowering boundary, but not every layer-opacity violation
  is a semantic-authority violation (a user project's layer
  opacity doesn't involve `Dag.declarations` directly).
- **Boundary sufficiency** says each stage's output must carry
  enough structural data for downstream consumers. Semantic
  authority is a specific application: the lowering boundary's
  output must carry every fact downstream consumers will need,
  referenced by `DeclarationId`, with no secondary authorities
  allowed as fallback.
- **No bridges** forbids adapters between two representations.
  Semantic authority forbids a specific class of adapter:
  String → DeclarationId at or after the lowering boundary.
- **Scaffold boundaries** (R16) requires scaffold variants to
  have boundary gates. Semantic authority extends the idea:
  not just scaffold variants, but ALL post-lowering reads of
  declaration data must use typed identity.

**Dissolution target.** When the semantic-authority lenses are
written and applied to `src/v3/compiler/src/`, violations
become visible in the CI output. Each violation names a
specific consumer that needs the bridge-dissolution pattern
above applied. Over time, the violation count should trend
monotonically downward; new violations are blocked at PR time;
existing ones are fixed as each downstream stage is touched.
The endgame is zero violations, at which point the invariant is
enforced by construction (there's no code that could violate
it because every declaration-reading site reads by
`DeclarationId`).

### Boundary sufficiency

A stage boundary is *sufficient* when the data it carries contains all
the structural facts the downstream stage needs, making name-based proxy
reads unnecessary. When a stage branches on a name to make a structural
decision, the boundary is insufficient — a fact is missing.

**The diagnostic:** scramble all user-defined names across a boundary.
If downstream decisions change, a structural fact is missing and the
name was used as a proxy. The scrambled-name test reveals exactly which
decisions depend on names, pointing to the missing facts.

**The fix:** always enrich the boundary, never restrict access. When
inference needs "has math methods," the fix is "put algebra membership
in the boundary data," not "hide the name." When emit needs "how to
declare a variable," the fix is "read LanguageSpec," not "prevent
hardcoding."

**Structural prevention:** Typed boundaries where insufficient data
is a compile error. If emit needs algebra membership and the boundary
doesn't carry it, emit can't compile — the field doesn't exist on the
boundary type. The escape hatch is `node.name` (any string, always
accessible, carries no structural guarantee); the fix is deleting
`Node.name` (M4/D6) so the only way to get information about a node
is through its structural properties and edges. The scrambled-name
tests are the diagnostic; `Node.name` deletion is the prevention.

### Explicit boundary contracts

Each stage of the pipeline (parse → typecheck → lower → resolve →
execute) passes a complex IR type to the next stage. The receiving
stage's preconditions must be structural — encoded in the type of the
boundary, not checked by a validation pass after the fact.

**The principle:** make illegal states unrepresentable. When a
downstream stage needs a guarantee (e.g., "all type references are
resolved"), the upstream stage must produce an output type that
*cannot* represent the unresolved case. The compiler enforces the
contract; no runtime validation walk is needed.

**The test:** if you find yourself wanting to add a validation pass
at a boundary, instead refactor the upstream stage's output type so
the invalid state is impossible to construct.

Examples (current state and target):
- After lowering (done): transport nodes are a distinct `LoweredOp::Transport`
  variant with required `ServiceCallMetadata` and `TransportObligation`.
  Transport obligations are structurally excluded from `LoweredOp::Callable`.
- After lowering (target): ports embed `ResolvedType` instead of `TypeId(String)`.
  `ResolvedType` is defined in `gunbc-ir` but not yet wired into ports;
  the migration is additive (`resolved_type` alongside `type_id`).
- After typecheck (target): the output type embeds resolved type structure,
  not a string TypeId that might not resolve.
- After resolve: the output DAG is parameterized by a trait that
  requires `Executable`, so non-executable nodes are unrepresentable.

When a boundary today uses a type that *can* represent invalid states,
that is the root cause — not the absence of a validation function.
Every fabrication fallback in FC-7 existed because the producing
stage's output type was too permissive, and the consuming stage
compensated with a fallback instead of failing.

A boundary fact table is only valid when both of these hold:

1. Every entry is an exact derivation from upstream structure. If the
   table collapses distinct bindings, guesses a classification, or drops
   witnesses needed downstream, it is a lossy representation and is
   already an invariant violation.
2. A downstream stage actually consumes the table as the authority for a
   decision. If no consumer reads it, the table is speculative metadata
   or a parallel representation waiting to diverge.

Unused or lossy fact tables are not harmless scaffolding. Unused tables
violate "No parallel implementations" / "Single-authority metadata."
Lossy tables violate "Explicit boundary contracts" / "Heuristics
indicate lost structure." The default action is to delete the table
until a concrete consumer exists, or tighten it until the missing
distinctions are structurally preserved.

New semantic boundaries must land end-to-end. A new normalize/pass/fact
layer is not accepted just because it computes plausible metadata; at
least one downstream consumer in the same change must read it as the
authority for a real compilation decision. Otherwise the layer is still
speculative metadata and should stay out of the pipeline until the
consumer exists.

### Emission is translation, not decision-making

The emitter translates an annotated graph to target-language text. It
does not make structural, semantic, or rendering decisions. Every fact
the emitter needs — sharing strategy, type representation, clone
behavior, import requirements — must be in the graph or in LanguageSpec
data before emission begins. If the emitter branches on a type name,
checks a hardcoded list, or guesses a rendering choice, a fact was
lost at an upstream boundary.

**The principle:** emission is a pure function from (annotated graph +
LanguageSpec) to text. No heuristics, no fallbacks, no per-language
decision logic. Language-specific facts live in LanguageSpec data
declarations. The shared emitter reads them.

**The test:** if adding a new target language requires writing emission
*logic* (not just data declarations), the shared emitter is making
decisions that should be data-driven. Target-language-specific code
paths in the emitter are dual representations of facts that should be
in LanguageSpec.

**Fail-closed:** if the emitter encounters a type or construct for
which it lacks a rendering annotation, it must produce a diagnostic
error — not silently emit placeholder or structurally wrong code. A
`compile_error!("...")` in generated Rust is a fabrication fallback;
the compiler should have caught the gap before reaching emission.

**Known violations (2026-03-29):**

| Decision | Current state | LanguageSpec target |
|----------|--------------|---------------------|
| Sharing/wrapping | `rc_types` map (Rust only). Go emits bare value-type structs. | `sharing_wrap_template`, `sharing_construct_template` per language |
| Clone semantics | Hardcoded `.clone()` in Rust emitter; ownership analysis elides for fan-out=1 function params | Language-level clone/copy strategy in LanguageSpec |
| Option/absence | Emitter heuristic | Absence variant spec in LanguageSpec |
| Async/await | Hardcoded `"async fn"` in Rust emitter | Async syntax template |
| Import generation | Per-emitter logic | Module system spec |
| Container iteration | Hardcoded `.iter().cloned()` | Iterator pattern template |
| Record literal Rc wrap | `Rc::new(...)` hardcoded at construction | Driven by sharing strategy |
| Empty list in record field | Emits bare `vec![]` instead of `Rc::new(vec![])` | Should derive from sharing + type |

The sharing model is the canonical instance. `.dag` has value semantics.
Each target language has its own way of expressing shared ownership:
Rust uses `Rc<T>`, Go uses `*T`, Python has reference semantics by
default. This is ONE cross-language fact with per-language syntax —
not three independent implementations in three emitters.

### E-6: No target-spec field without a same-PR consumer (2026-04-16)

A field declared on a target spec (`CallableRealization`,
`TargetExecutionModel`, `PatternRealization`, etc.) MUST be read by
at least one emitter in the same PR that introduces it. Speculative
target-spec fields — declared, parsed, stored, and then never
consulted during emission — become advisory metadata. Once the
"declare now, consume later" pattern is normalized, every subsequent
target spec inherits the same optionality, and the thesis claim
"targets are declarations, emission is mechanical translation" drifts
back toward aspiration.

**Why this is a specialization of "Emission is translation."**
"Emission is translation" already says the emitter reads LanguageSpec
data instead of making decisions. E-6 closes the other direction:
spec data must be authoritative, not advisory. A field the emitter
ignores is indistinguishable from a field the emitter doesn't know
about. Both teach the wrong architecture.

**The canonical counter-example (PR #490 diff):** a regression test
`go_gc_targets_skip_rendering_model_loading` deliberately corrupted
`go_rendering` and asserted emission succeeded. The test codifies
"declared target fact is non-authoritative" as a unit-tested
property — the clearest possible smoking gun.

**Bounded exception:** a target spec field MAY land without an
emitter consumer if paired with an explicit **dissolution ratchet**
naming the PR or lane that will wire consumption. Without the
ratchet, the field is speculative metadata.

**Structural prevention:** at PR review, grep every new field added
to a target-spec record type against the emitter source tree. Every
new field must either (a) have at least one consumer call site added
in the same diff, or (b) sit behind a named scaffold marker with a
dissolution trigger.

**Test:** the PR adding a new target-spec field must include at
least one test that fails if the field is mis-populated. A test that
asserts "spec is ignored" is a reverse signal.

### E-7: No target-private realization schema without a dissolution ratchet (2026-04-16)

When a new emission target lands, it MUST consume the shared
realization schema (`v3.std.emit_model.TypeRealization`,
`CallableRealization`, `OperatorRealization`, `PatternRealization`,
etc.) unless the shared schema is missing a fact the target
genuinely needs. In the latter case, the shared schema extends to
carry that fact. Creating a parallel target-private realization
family (`PythonTypeRealization`, `PythonCallableRealization`, etc.)
is permitted only as a time-bounded scaffold with an explicit
dissolution trigger that names the PR or lane converting the target
onto the shared schema.

**Why.** Parallel representation drift is the class of debt that
"No duplicate representations" invariant already forbids in general.
E-7 specializes it to the realization surface, where the temptation
is strongest — each new target emitter starts easy with a private
schema, then calcifies as the consumer count grows. Without a named
dissolution trigger, "this is staged for now" is indistinguishable
from "this is the intended end state."

**The canonical incident (2026-04-16):** `src/v3/spec/python.dag`
declared `PythonTypeRealization`, `PythonOperatorRealization`,
`PythonCallableRealization`, `PythonTypeInstantiationRealization`,
and `PythonPatternRealization` as a second realization family
parallel to the shared `v3.std.emit_model`. Prose dissolution notes
existed on two of the private strategy enums
(`PythonCallableStrategy`, `PythonPatternStrategy`) but not on the
five realization records themselves. The mismatch between "scaffold
documented for two enums" and "scaffold undocumented for five
records" is the smoking gun — without mechanical gating, prose
discipline drifts.

**Structural prevention:** CI grep gate — every target-private
realization data type (match on suffix: `*TypeRealization`,
`*OperatorRealization`, `*CallableRealization`,
`*PatternRealization`, `*TypeInstantiationRealization`) outside of
`v3/std/emit_model.dag` MUST have an adjacent
`🟡 SCAFFOLD ... dissolves when ...` comment with a trigger that
names a specific follow-up. A grep that returns hits without
corresponding dissolution comments fails CI.

**Dissolution trigger shape.** The trigger must name a PR, lane, or
concrete engineering task — not a vague "future consolidation."
Acceptable: "dissolves in Lane 1e (consolidation)," "dissolves when
emit_python migrates to shared CallableRealization (PR TBD)."
Unacceptable: "dissolves when emit_python matures," "eventually
folds into shared schema."

**Test:** renaming a variant or field in the shared schema must
either update the parallel target-private schema in the same PR or
fail closed with a diagnostic. Silent divergence is the hazard this
invariant prevents.

### E-8: Unsupported core behaviors fail closed, never collapse semantically (2026-04-16)

When an emitter reaches a `Behavior` variant (Value, Transform,
Branch, Loop, Bind) for which it has no rendering strategy, it MUST
return a structured `UnsupportedBehavior` error. Silently rendering
a substitute — for example, "render the loop body's result port" in
place of a Loop construct — collapses iteration semantics into a
single-iteration expression and produces emitted code that compiles
but is structurally wrong. The class of silent semantic collapse is
strictly worse than a hard error, because the output passes
downstream checks that assume correctness.

**Why this is distinct from other fail-closed rules.** The
"Fail-closed compilation" invariant (on missing rendering
annotations) and the "Emission is translation" fail-closed paragraph
both cover the case where the emitter lacks a spec entry. E-8
specializes the rule to `Behavior` variants: the emitter has a
`match` on the substrate's closed-set behavior enum, every arm must
either emit correct code or return an error. A fallthrough arm that
emits *something* is a fact-drop at the substrate boundary.

**The canonical incident (2026-04-16):** `emit_go` and `emit_python`
both implemented `Behavior::Loop` as "render the loop body's result
port," which silently collapsed `fold(list, init, f)` into the
first iteration's expression (`f(init, head(list))`) — a Loop over
a list became its first iteration's expression. The collapse was
detected only by CI on a deeply-recursive .dag fixture; most
fixtures happened to have loop bodies that looked plausible when
rendered in isolation. The principle audit caught the class three
review rounds running before the fix landed.

**Structural prevention:** every emitter's behavior-dispatch match
must enumerate all five variants explicitly, and every arm must
either return a correctly-emitted expression OR return an
`UnsupportedBehavior`-class error. Fallthrough arms, default arms,
and arms that emit structural substitutes are forbidden.

**Test at PR review:** grep each emitter for `match * { ... }`
dispatches on `Behavior`. Every arm that does not emit a faithful
construct must return an error. A comment `// TODO: support Loop
properly` over a silent substitute is an E-8 violation — the
comment is prose documentation, not a structural gate.

**Tests that were passing by accident (2026-04-16 remediation):**
when `emit_go` and `emit_python` started fail-closing on Loop,
three tests previously passing through the silent collapse
(`go_lens_unused_parameters_module`,
`emit_python_module_marks_ownership_as_skipped_for_gc_target`,
`emitted_python_lens_matches_emitted_rust_lens_on_reflected_programs`)
were marked `#[ignore]` with the explicit reason *"blocked on
emit_<lang> Behavior::Loop support; previously passed via silent
loop-body collapse."* This is the right shape for E-8 debt: the
ignore ledger names the blocking substrate capability, so the
test's re-enable condition is unambiguous.

### Single-authority metadata

The compiler should provide all metadata (tool definitions, output
paths, type registries) through its own output types (`CompileOutput`,
`InferredEntrypoint`, etc.), not through runtime callbacks, string
conventions, or hardcoded lists. Each piece of metadata should have
exactly one producer.

**Structural prevention:** Guarantee receipt. The compiler emits a
machine-readable receipt on every run that records what was discovered,
what was proven, what was tested, and what's uncertain. If a guarantee
isn't in the receipt, it doesn't exist. Markdown dashboards are derived
from the receipt — never the source of truth. The escape hatch is
metadata scattered across log output, comments, and separate scripts;
the fix is one structured artifact that CI can enforce.

### Verification predicates are substrate consumers

Any predicate in `std/verification.dag` (or equivalent verification
authority) that asserts a property of the compiler's output must
reference the compiler's own authoritative fact as a typed edge.
Parallel enum/string schemas for the same concept the compiler
already models are forbidden — they are shadow taxonomies, not
verification.

**Why this is a separate invariant.** "No duplicate representations"
forbids duplicate facts in general. This invariant specializes that
rule to the verification surface, where the temptation is strongest:
testgen needs a "view" onto compiler facts, and the cheapest local
move is to sketch a small enum or string-extraction helper
("kind + detail_contains") that approximates the real authority.
Each such approximation becomes a bridge that drifts as the real
authority evolves, and because verification is downstream of every
compiler fact, the bridges multiply quickly.

**The test:** if a test assertion would change meaning after a
rename or restructure of the authoritative compiler type (Diagnostic,
PortState, Declaration), the predicate is consuming a shadow schema,
not the substrate fact. A predicate that references `DeclarationRef`
or equivalent typed edges cannot silently drift; one that references
`DiagnosticKind::TypeMismatch` as a parallel enum can.

**The fix:** replace parallel enums with typed references into
substrate. `DiagnosticReference.kind: DiagnosticKind` (parallel enum)
becomes `DiagnosticReference.kind: DeclarationRef` (typed edge to
the real diagnostic variant declaration). String extraction helpers
(`diagnostic_detail(...)`) become structural field reads.

**Bounded scaffolds are allowed with named triggers.** If substrate
doesn't yet expose the fact in a consumable form (e.g., Diagnostic
isn't reflected into std/ yet), a shadow taxonomy can exist as a
bounded scaffold — but only with an explicit `🟡 Scaffold` marker
naming the dissolution trigger. "Dissolves when substrate.Diagnostic
becomes consumable by std/verification.dag" is load-bearing
documentation; without it, the scaffold is indistinguishable from a
permanent parallel authority.

**Historical context (2026-04-16):** PR #481's verification surface
rework dissolved the port-state and cost-bound axes (flat variants
→ structural carriers using substrate's `ComparisonOp`), but
introduced `DiagnosticKind` as a parallel enum to the compiler's
native Diagnostic type, with string-extraction helpers
(`diagnostic_kind()`, `diagnostic_detail()`) in test code bridging
back to the real fact. Both reviewers (codex + ChatGPT-browser)
independently flagged the same pattern. This invariant graduates the
finding so future verification-surface changes have a named
structural rule to test against.

**Structural prevention:** typed edges are the shape. When a
verification predicate needs a compiler concept, the substrate
reflects the concept as a declaration, and the predicate carries a
`DeclarationRef` to it. The compiler's own `meta_tag` / `inhabits`
machinery makes this typed edge equivalent to "this predicate is
testing the same concept the compiler uses internally." The escape
hatch is parallel enums with matching variant names; the fix is
refusing to write them.

## Engineering Standards

These serve sustainability indirectly by reducing the blast radius of
changes:

- **Clear interfaces.** Every public module should have a small,
  well-defined API surface. Prefer returning values over mutating
  shared state.

- **Pure core logic.** Deterministic functions from inputs to outputs.
  Side effects (filesystem, network, process spawning) belong at the
  edges, not in the middle of computation.

- **Documented I/O boundaries.** Any function that performs I/O must
  document that fact in its signature or doc comment. Callers should
  never be surprised by hidden I/O.

- **No flags in codegen.** Boolean flags that change compilation behavior
  globally (like `force_clone`) are forbidden. Every compilation decision
  must be derived from the actual type and context of the expression
  being compiled, not from a global check. Flags silently degrade and
  are impossible to remove incrementally.

## Testing Invariants

- **Behavioral only.** Tests assert observable behavior — outputs given
  inputs, error messages, public API contracts. Never assert internal
  implementation details like which private functions were called, what
  order internal steps execute in, or how many times an internal helper
  runs.

- **Source-audit tests are a narrow exception.** When a test
  intentionally reads source text as an architectural ratchet, it must
  anchor on live syntax or declarations and ignore comments or
  historical notes. A comment match is not evidence that a boundary or
  implementation still exists.

- **Hermetic unit tests only.** Tests must not touch the filesystem,
  network, or environment. All external dependencies are injected or
  mocked. A test that passes on one machine must pass on every machine.
  Corpus/integration tests (e.g., `daglang-syntax/tests/item_coverage.rs`)
  that walk the `dsl/` source tree are a recognized exception — they
  live in `tests/` directories and are clearly labeled as non-hermetic.

- **No tautological tests.** A test that mirrors the implementation —
  restating the production code in test form — proves nothing. Tests
  must encode an independent specification of *what* the code should do,
  not *how* it does it. If deleting the test body and replacing it with
  a copy of the production code would still pass, the test is
  tautological.

## Tiered Test Execution (T11)

DAG execution tests use three tiers, each proving a different layer of
correctness. Every test explicitly chooses its tier via `ExecutionMode`.

### Tier 1 — DryRun (structure)

All transport, resource-environment, and tool nodes are intercepted with
explicit mocks (`ExecutionMode::DryRun(mocks)`). Pure nodes execute
normally. This tier proves DAG wiring, port cardinality, coercion, guard
evaluation, conditional branching, and topological ordering — without
performing any real I/O. The majority of existing tests operate at this
tier.

### Tier 2 — Selective Real (computation)

The DAG executes in `ExecutionMode::Real`, but the operations themselves
are limited to safe, hermetic effects: reading environment variables,
filesystem operations in temporary directories, timestamps, and
conditional logic. No external HTTP calls or cloud API interactions.
This tier proves that computation within the DAG produces correct
*values*, not just correct *shapes*.

Reference tests: `env_var_read_real_mode`,
`real_mode_executes_resource_environment_node` in
`src/v1/09_execute/exec/src/execute/tests.rs`.

### Tier 3 — Full Real (integration)

All nodes execute for real against live services. Only viable in
controlled environments with sandboxed credentials (CI runners with
scoped tokens, disposable cloud resources). Proves end-to-end behavior
including HTTP transport and cloud API interactions. Not yet implemented;
requires credential injection infrastructure.

## Branch Review Findings

### 2026-03-21 — `v2-compiler-convergence`

- Deleted `src/v2/04a_normalize.dag` and removed the extra
  reconcile→normalize→emit boundary. The stage introduced unused and
  lossy fact tables (`func_facts`, `enum_facts`, `field_facts`) that
  were not consumed by any emitter, and some entries were already
  degraded (shadowed bindings collapsed by name, match-arm context lost,
  placeholder function classifications). Emit now consumes the existing
  reconcile boundary directly again until an exact, authoritative
  emitter-facing index is needed.

### 2026-03-21 — transport/expr dissolution review

Fixed:

| # | Violation | Fix |
|---|-----------|-----|
| TD-1 | `LitString` typo in `auth_properties` and `find_property_string` (variant does not exist) | Fixed to `LitStr` (3 sites in `00_core.dag`). Latent — no test breakage because `auth_properties` never called in current test paths. |
| TD-4 | Dead `parent_enum == "Expr"` in `05_emit_rust.dag` variant construction | 7 lines removed. |
| TD-5 | Dead `classify_transport_kind()` in `05_emit.dag`, imported but never called | Function deleted, imports removed from Go/Python emitters. |
| TD-6 | Stale DESIGN.md Layer 2 documented old `TransportBinding` sum type | Updated to Node-based transport model. |

### 2026-03-21 — semantic-boundary review

Classified as invariant violations:

- Rust emission still repairs semantics downstream instead of consuming a
  fully classified boundary: `emit_typed_field_access` branches on
  `.typed`, `.value`, `is_likely_optional_receiver(...)`, and
  `emit_typed_expr` conditionally appends `.map(Rc::new)` via
  `lookup_on_data_needs_rc_wrap(...)`. This violates "Heuristics
  indicate lost structure" / "Explicit boundary contracts."
- `lookup_in_scope` falls back to `lookup_func_sig(...).return_type` for
  function-as-value references. That fabricates a non-callable value from
  a callable binding and violates "Explicit boundary contracts" / "No
  fallbacks that fabricate."
- `node_type_equals` still contains permissive compatibility rules
  (`Dynamic` matches anything, plus same-name/same-connective/same-child-count
  fallback) that hide missing earlier normalization. This violates "No
  fallbacks that fabricate" / "Explicit boundary contracts."
- ~~Reconcile downgrades semantic gaps to `Warning`~~
  **FIXED (2026-04-01).** `OwnershipWarning` renamed to `OwnershipViolation`,
  `VariantCollisionWarning` renamed to `VariantCollision`, both promoted to
  errors. `is_error_diagnostic` now always returns `true`. No warning
  severity remains in the compiler.

Not invariant violations by themselves:

- Roadmap/docs drift (`A7 full retirement`, `P1b done`, acceptance text
  that still names future work).
- Loose ratchets and unlanded StageMetrics/performance-contract work.
  Current checked-in values: `SELF_COMPILE_ERROR_RATCHET = 2700`,
  `CLONE_RATCHET = 21000` (pipeline.rs:7845). These are backlog/test
  debt, not direct invariant violations until a concrete boundary or
  algorithm violates a stated rule.

---

## Open Debt

Three root causes account for ~50 individual sites. Fixing the root causes
eliminates the symptoms; fixing symptoms individually is whack-a-mole.

### Status (2026-04-12)

Root causes are ADDRESSED (design decided, infrastructure landed) but
not fully CLOSED. Live violations remain in the semantic-boundary
review (2026-03-21) and the Root Cause A/B/C tables describe work
that is partially done, not complete.

- Root Cause A: Infrastructure landed (EmitContext, RefKind, ParamSource).
  Migration underway. A-4 (function-as-value) and A-8/A-9 (Dynamic)
  still have live violations.
- Root Cause B: Partially addressed. B-1 (transport kind) and B-2
  (item kind) structurally dispatched. B-3 through B-6 still use
  string dispatch in some paths.
- Root Cause C: Mostly addressed. Remaining duplication is being
  dissolved by M4 Phase 2 (expression dispatch unification).

The root-cause tables below are the historical problem statement.
Items marked "done" in the tables are genuinely complete; unmarked
items are still live.

---

### Root Cause A: Reconcile→Emit Boundary is Information-Lossy (ADDRESSED)

**Status:** Design decision made, infrastructure landed. Gradual migration underway.

**Design decision (2026-03-21):** Split into two categories:

1. **Reconcile resolution bugs (A-4, A-5, A-8, A-9):** Reconcile fails to resolve
   facts it should. Fix: improve resolution, add `RefKind` and `ParamSource` types.

2. **Emit rendering decisions (A-1, A-2, A-3, A-6, A-7, A-10):** Emit owns these
   decisions but must compute them efficiently. Fix: `EmitContext` struct with 6
   cached indexes built once per emit call, O(1) lookups per expression. No
   precomputation in reconcile — rendering decisions stay with the renderer.

**Infrastructure landed:**
- `EmitContext` type + `build_emit_context` + `ctx_*` helpers in `05_emit.dag`
- `RefKind`, `ParamSource` types in `04_reconcile.dag`
- `build_intrinsic_index`, `build_primitive_set` pre-built at emit entry
- EmitContext wired into `emit_rust` entry point

**Remaining:** Migrate emit functions from individual map params to `EmitContext`
lookups. Mechanical — each function gets `ctx: EmitContext` parameter, replaces
ad-hoc scans with `ctx_*` helpers.

| # | What reconcile computes | Where it's lost | How emit compensates |
|---|------------------------|-----------------|---------------------|
| A-1 | Field access style (StoredField / EnumAccessor / OptionalUnwrap) — `build_field_summaries_*` at `04_reconcile.dag:1070-1175` | Not attached to ExprFieldAccess nodes | `emit_typed_field_access` calls `lookup_emit_field_summary_in_scope` at codegen time (redundant); `is_likely_optional_receiver` scans all type_summaries; `is_optional_field_in_any_type` / `is_enum_accessor_in_any_type` do global sweeps (`05_emit_rust.dag:1576-1601`) |
| A-2 | Known-method classification + result type — `resolve_known_method_node` in `04_reconcile.dag` | `ExprMethodCall` now carries `method_semantics`; remaining loss is that renderer leaf helpers still branch on `method` strings for target syntax | Complexity no longer compensates. Emit still has per-target method-name ladders and runtime helper tables. |
| A-3 | Call→MethodCall bridging — ExprCall handler rewrites bridged calls to `ExprMethodCall` | No longer lost after reconcile; bridged calls remain structurally distinct downstream | Emit no longer needs to rediscover bridged method shape, but Rust still carries target-specific runtime helper maps for ownership/rendering. |
| A-4 | Function-as-value reference — `lookup_in_scope` fallback to `lookup_func_sig` at `04_reconcile.dag:751-754` | ExprVar node gets return type only; callable-vs-value distinction lost | Emit cannot distinguish function reference from local binding (SB-1). Fabricates value type from callable's return type. |
| A-5 | Fold accumulator type — computed during method resolution | No longer lost on typed method nodes; carried in `IntrinsicMethodSemantics.fold_accumulator_type` | Downstream consumers can read it from `method_semantics`; remaining work is deleting renderer-local fallbacks. |
| A-6 | Rc-wrapping requirement — derivable from type summaries and scope types | Not attached per expression; Rust emit still re-derives it from a module-local `rc_types` map plus Rust-local match analysis | Emit now centralizes match probing through `RcPatternAnalysis`/`RcMatchAnalysis`; lookup-specific wrapping on data maps remains separate |
| A-7 | Variant→parent enum mapping — resolved during type resolution | Only available via global `vtoe` map, not per-expression | Emit builds module-local vtoe disambiguation (`05_emit_rust.dag:430-467`); `emit_var_ref` does fallback lookup (line 1508) |
| A-8 | Dynamic/error type propagation — `node_is_dynamic` at `04_reconcile.dag:900` | Error state encoded as `string_contains("<error:")` in type name | Emit replicates check at `05_emit_rust.dag:1473`; `node_type_equals` treats Dynamic as universally compatible (SB-2) |
| A-9 | Lambda parameter types — unresolved when collection type is Dynamic | Bound to `Dynamic` in `extend_scope_for_lambda` (`05_emit_rust.dag:1959`) | Auto-wrap disabled entirely (`let needs_wrap = false` at line 2445) because `is_already_optional` can't detect Optional inside Dynamic-typed lambdas |
| A-10 | Primitive/collection type identity — structurally known | Only available as type name strings | Emit hardcodes `"Int"`, `"Bool"`, `"Float"`, `"List"`, `"Map"`, `"Set"`, `"String"` in name-matching functions (`05_emit_rust.dag:1145-1150`, `882-908`, `1488-1494`) |

**Previously tracked as:** F6, F7, SB-1, SB-2

---

### Root Cause B: Closed Sets Dispatched as Strings

**Invariants violated:** No case enumeration for open sets, No parallel
implementations.

**The problem:** Several finite, known-at-compile-time sets are encoded as
strings and dispatched via `if x == "..."` ladders across multiple files.
Adding a value to any set requires editing every dispatch site — there is
no compiler-enforced exhaustiveness.

**Design decision required (methods only):** Are method/builtin intrinsics a
closed compiler-known set (→ enum) or structural DSL-defined facts the
compiler discovers? The language thesis says "smart facts + dumb compiler,"
so methods should eventually be data declarations in `.dag`. Pragmatically,
an `IntrinsicId` enum is the right intermediate step — it centralizes the
set and gives exhaustiveness checking. The enum definition becomes the single
authority; reconcile tags each call with an `IntrinsicId`; emit matches on
the enum instead of strings.

Transport kind, item kind, and type structure are mechanical enum conversions
with no design ambiguity.

| # | Closed set | Values | Dispatch sites | Files affected |
|---|-----------|--------|---------------|----------------|
| B-1 | Transport kind | rest, shell, file, local | 21 | 04_reconcile, 05_emit_rust, 05_emit_go, 05_emit_python |
| B-2 | Item kind (`classify_typed_item`) | type_def, type_alias, function, data_def, service_def, resource_def, extern_func, unhandled | 8 dispatch chains | 05_emit, 05_emit_rust, 05_emit_go, 05_emit_python |
| B-3 | Type structure (`classify_type_structure`) | leaf, conj, disj | 3 dispatch chains | 05_emit, 05_emit_rust, 05_emit_go, 05_emit_python |
| B-4 | Method/builtin intrinsics | ~35 methods + ~20 builtins | ~60 string branches | 04_reconcile (inference), 05_emit (classification), 05_emit_rust (lowering) |
| B-5 | Operation modifiers | idempotent, readonly, hermetic | 1 filter expression | 05_emit_rust:2836 |
| B-6 | Config property names | base_url, auth_scheme, auth_header, auth_token | `config_names` list + constructors + accessors | 00_core.dag (triple representation) |

**Previously tracked as:** TD-2, TD-3, F7 (partially — the emit-side ladder is Root Cause A)

---

### Root Cause C: Errors Propagate as Valid-Looking Fabrications

**Invariants violated:** No fallbacks that fabricate, Explicit boundary
contracts, Correctness by construction.

**The problem:** When the compiler encounters an error (missing argument,
unresolved type, unknown function), it fabricates a valid-looking node
(LitNull, Dynamic, `<error:*>` string) and continues. This lets broken
programs reach emit, which generates invalid target code containing
sentinels like `<error:unknown_with_type>` or empty strings.

**Design decision required:** Structural error representation. Currently
error state is encoded as:
- `LitNull` with `return_type: none` (37 sites across parser/reconcile/emit)
- `Dynamic` type name (universal compat in `node_type_equals`)
- `<error:*>` strings detected by `string_contains` (2 check sites, 4 production sites)
- ~~`Warning` severity for semantic errors (`access_error`, `inference_error`)~~ **FIXED (2026-04-01).** All diagnostics are now errors; `is_error_diagnostic` always returns `true`.

The fix: make error a structural variant — either an `ExprError` in ExprData
or a flag on Node — so downstream phases can test `is_error(node)` without
string parsing. Emit skips error nodes (or emits `compile_error!()`) instead
of translating fabricated values.

Parser LitNull recovery (23 sites in `02_parse.dag`) is a separate concern —
parser error recovery that produces dummy nodes with attached error
diagnostics is standard practice. The issue is that reconcile and emit don't
recognize these as error nodes and try to process them normally.

| # | Pattern | Sites | Where |
|---|---------|-------|-------|
| C-1 | LitNull sentinel for missing arguments | 5 | `05_emit_rust.dag:1751,1752,1760,1761,1786` |
| C-2 | LitNull sentinel for missing defaults/config | 9 | `04_reconcile.dag:3025,3053,3114,3158,3165,3172,3272,3293,3510` |
| C-3 | LitNull dummy for parser error recovery | 23 | `02_parse.dag` (throughout) |
| C-4 | `<error:*>` placeholder types | 4 production | `04_reconcile.dag:1531,1698,1861,2255` |
| C-5 | `<error:*>` detection via string_contains | 2 check | `04_reconcile.dag:900`, `05_emit_rust.dag:1473` |
| C-6 | `<error:unknown_*>` sentinels in emit | 2 | `05_emit_rust.dag:1766,2117` |
| C-7 | Dynamic as universal compatibility | multiple | `node_type_equals` in `04_reconcile.dag:901+`; `extend_scope_for_lambda` in `05_emit_rust.dag:1959` |
| C-8 | ~~Warning severity for semantic errors~~ | **FIXED** | `OwnershipWarning` → `OwnershipViolation`, `VariantCollisionWarning` → `VariantCollision`; `is_error_diagnostic` always returns `true` (2026-04-01) |
| C-9 | Empty node / empty string fabrication | 2 | `05_emit_rust.dag:819` (empty Node for missing field), `05_emit_rust.dag:3368` (LitNull → "") |
| C-10 | `Rc::try_unwrap` clone fallback (v1) | 1 | `fn_codegen.rs:3783` — blocked on Track D ownership proof |

**Previously tracked as:** TD-7, SB-2, SB-3

---

### v2 Pipeline Audit (2026-03-22)

Full line-by-line audit of all 14 v2 .dag files (~16,000 lines). ~100 violations
across 7 structural themes. Root Causes A/B/C above were v1-focused; these are
the v2-native counterparts. Execution order minimizes total work — each theme
unblocks or cheapens the next.

**Execution order:** 4 → 6 → 3 → 5 → 1 → 2 → 7 (interleaved)

#### Why These Exist — Three Root Causes

The 7 themes compress to 3 root causes. Understanding them prevents recurrence.

**I. The IR conflates domain facts with rendering strategy.**

The clearest example was shared semantics carrying Rust policy.
`MethodSemantics` used to carry `wrap_result_in_rc` and `pass_receiver_by_ref`,
and `CallSemantics` used to carry `needs_rc_wrap` — fields that only made sense
if compiling to Rust. Those fields are now gone from `00_core.dag`, but the
lesson remains: once target policy enters shared semantics, every downstream
consumer starts destructuring facts it doesn't own. Python and Go previously had
to pattern-match Rust-only fields they did not use.

Once the IR carries rendering hints, the boundary between "what the program is" and
"how to render it" blurs. Reconcile starts computing Rc decisions. Emit starts
re-resolving types. Dynamic becomes a catchall because the type system serves two
masters. This is the origin of Themes 3, 5, and partially 7.

Prevention: for every field on a core type, ask "would this field make sense if we
were compiling to VHDL?" If no, it doesn't belong in core.

**II. No structural fold over ExprData.**

The DAG language has sum types but no generic visitor. Every consumer writes its own
20-arm match. 5 consumers × 20 variants = 100 match arms that must stay in sync.
Adding one ExprData variant means editing 10+ functions across 6 files.

The "shared dispatch" for Theme 2 is building a `fold_expr` by hand. The reconcile
fusion (Theme 1) is manually combining two handler tables into one match. Both are
workarounds for the language lacking parametric types — you can't write
`fold_expr(handler: ExprHandler<A>, acc: A, expr: Node) -> A` without generics.

Prevention: the language design choice to omit generics forced copy-paste parallelism.
Until/unless the language gains parametric types, the mitigation is to minimize the
number of walks (target: 5) and never add a new one without deleting an existing one.

**III. Define-at-use-site instead of import-from-authority.**

When reconcile needed kernel types, it defined `is_primitive_name()`. When emit needed
them, it defined `build_primitive_set()`. When complexity needed method costs, it
defined `classify_method_cost()`. Each file solved its local problem by copying. Dead
stubs (artifact, trace) are the flip side: code written speculatively, never connected
to an authority. This is Themes 4 and 6.

Prevention: import-first discipline. Before defining a list or classifier, check if an
upstream module already has one. If not, define it in the lowest shared module, then
import.

**All seven themes are symptoms of one thing: the v2 compiler was built bottom-up.**
Each file solved its local problem correctly. Nobody enforced that shared facts flow
downward from a single authority. The fix is to invert the direction: define authorities
first, then build consumers that import from them.

#### Acceptance Criteria — End State

All themes done = every item below is checked. Organized by file so nothing
gets missed during cleanup. Items marked DELETE must not exist; items marked
GONE mean the surrounding function/field no longer exists in that file.

**`00_core.dag`**
- [x] `kernel_types: List<String>` exists (canonical list, 8 entries)
- [x] `is_kernel_type(name: String) -> Bool` exists, uses `kernel_types`
- [x] `LookupCallSemantics` has no `needs_rc_wrap` field
- [x] `IntrinsicMethodSemantics` has no `wrap_result_in_rc` field
- [x] `RuntimeBridgeSemantics` has no `wrap_result_in_rc` or `pass_receiver_by_ref` fields
- [x] `expr_self_call_info(...)` exists and computes both recursion facts in one walk
- [ ] `expr_has_self_call` GONE (compat wrapper removable after downstream imports stop depending on it)
- [ ] `expr_has_non_tail_self_call` GONE (compat wrapper removable after downstream imports stop depending on it)

**`03_resolve.dag`**
- [x] `kernel_type_names()` DELETE — callers import `kernel_types` from core

**`04_reconcile.dag`**
- [x] `is_primitive_name()` DELETE — callers import `is_kernel_type` from core
- [x] `build_type_env` kernel list (lines 3157-3164) replaced with `kernel_types` import
- [x] `build_type_env_unresolved` kernel list (lines 3260-3264) replaced with `kernel_types` import
- [x] `node_is_named_ref` inline kernel exclusion (lines 968-978) uses `is_kernel_type`
- [x] `type_needs_rc` GONE (moved to emit_rust)
- [x] `type_needs_rc_seen` GONE (moved to emit_rust)
- [x] `data_lookup_needs_rc_wrap` GONE (moved to emit_rust as `rust_lookup_receiver_needs_rc_wrap`)
- [x] `rc_wrapped: Bool` GONE from TypeSummary
- [x] `rc_wrapped_types: Map<String, Bool>` GONE from EmitGraphInfo
- [x] `rc_wrapped_types: Map<String, Bool>` GONE from EmitStateAccum
- [x] `emit_info_is_rc_wrapped_type` GONE (moved to emit_rust)
- [x] All `rc_wrapped_types` accumulation logic GONE from `build_emit_graph_info`
- [ ] `infer_expr` and type resolution fused into single walk (`infer_and_resolve_expr`)
- [ ] `collect_calls_in_expr` + `expr_has_self_call` + `expr_has_non_tail_self_call` fused into `analyze_expr_calls` returning `CallAnalysis`
- [ ] Dynamic sites audited: each classified as Correct/Lazy/Fixed, ≤5 justified remaining
- [ ] No string-based method dispatch downstream of the classifiers in reconcile

**`05_emit.dag`**
- [x] `build_primitive_set()` DELETE — callers import `kernel_types` from core
- [x] `ctx_is_rc_wrapped` GONE (Rc concern moved to emit_rust)
- [ ] Shared `emit_typed_expr` dispatch exists (single 20-arm match, target parameter)
- [ ] Shared TCO dispatcher with `TcoSyntax` config exists
- [ ] Shared service/transport traversal exists

**`05_emit_rust.dag`**
- [x] `build_module_vtoe` stub DELETE
- [x] `emit_record_lit` compat wrapper DELETE (tests updated to call `emit_record_lit_full`)
- [x] `resolve_expr_type_node` DELETE
- [x] Rc decision map is derived once in Rust emit entry/module wrappers from `type_summaries`
- [x] `type_needs_rc`, `type_needs_rc_seen` live here (moved from reconcile)
- [x] `rust_lookup_receiver_needs_rc_wrap` lives here (moved from reconcile)
- [x] All 6 Rc-probing heuristics consolidated into the Rust-side Rc pre-pass
- [ ] `emit_typed_expr` 20-arm match GONE (replaced by leaf functions called from shared dispatch)
- [ ] `emit_typed_tco_expr` parallel walk GONE (replaced by shared TCO dispatcher)
- [x] All 18 intrinsic methods handled (no fallback arms)

**`05_emit_python.dag`**
- [ ] `_unimplemented` placeholders (lines 1063, 1076) GONE — real emission or compile error
- [ ] `emit_py_typed_expr` 20-arm match GONE (replaced by leaf functions)
- [ ] All 18 intrinsic methods handled (currently 7)
- [ ] No silent fallback for unhandled methods

**`05_emit_go.dag`**
- [ ] `/* unhandled expr */` wildcard (line 592) GONE — match is exhaustive
- [x] Dead `if wrap_result_in_rc` identity branch (line 659) GONE
- [ ] `emit_go_typed_expr` 20-arm match GONE (replaced by leaf functions)
- [ ] All 18 intrinsic methods handled (currently 7)
- [ ] No silent fallback for unhandled methods

**`06_pipeline.dag`**
- [x] Artifact computation block (lines 170-179) DELETE — `_artifact_output`, `plan`, `artifact` locals all gone
- [x] Go arm wired: `Go => emit_go(typed: typed)` (not error diagnostic)
- [x] `import v2.compiler.emit_go { emit_go }` exists
- [x] `resolve_sources` refactored — shared tokenize→parse→resolve helper with `compile_sources`
- [x] Header comment (line 10) matches reality (mentions Go alongside Rust/Python)

**`07_complexity.dag`**
- [x] `intrinsic_method_cost_shape` is the only method→`CostShape` authority — no parallel `classify_method_cost` or inline string classifier remains
- [ ] `intrinsic_cost_shape` is exhaustive match on IntrinsicMethod — no Option, no None, no wildcard
- [x] `is_size_preserving_method(mname: String)` DELETE — replaced by `is_size_preserving(intrinsic: IntrinsicMethod) -> Bool`
- [x] `is_size_preserving` is exhaustive match — no string comparison
- [ ] `count_self_calls` (lines 1253-1323) DELETE — fused into `cost_of_expr` or uses `CallAnalysis` from reconcile
- [x] `cost_of_expr` reads `method_semantics` from Node, never matches on method name strings

**`07_ownership.dag`**
- [ ] Match arm patterns walk `VariantPattern` bindings (currently skipped)
- [ ] Destructuring patterns updated for any MethodSemantics field changes from Theme 5

**`08_artifact.dag`**
- [x] `plan_artifacts` ModuleBased stub arm (lines 86-88) DELETE
- [x] `plan_artifacts` ServiceBased stub arm (lines 89-91) DELETE
- [x] Only `Explicit` arm remains, or function deleted entirely

**`09_trace.dag`**
- [x] `import std.types { SourceSpan }` fixed to `import v2.std.core { SourceSpan }`
- [x] Interpreter-oriented header/comments reconciled with `src/v2/DESIGN.md` (compiler is a pure transform; no interpreter in the compiler)
- [x] Module connected to pipeline (called from `compile_sources`) or explicitly marked as future work

**Cross-cutting invariant: `00_core.dag` is target-agnostic.**

Every field on every type in `00_core.dag` must satisfy: "this field would make sense
if we were compiling to VHDL, C, or a hardware description language." Fields that
encode a specific target's memory model (Rc, borrow, GC, pointer), execution model
(async, coroutine), or syntax (indentation, braces) do not belong in core.

Rendering decisions are *computed* by emit from domain facts — never *stored* on core
types. If `type_needs_rc` is derivable from cycle detection on the type graph (it is),
it should never have been a field. If `pass_receiver_by_ref` is derivable from Rust's
borrow rules applied to the method's receiver type (it is), it should never have been
a field. The domain model records the facts; each backend derives its strategy.

After cleanup, this grep returns zero results:
```
rg -n '\b(needs_rc_wrap|wrap_result_in_rc|pass_receiver_by_ref|Rc|borrow)\b' src/v2/00_core.dag
```

**Cross-cutting invariant: no new ExprData walks without deleting an existing one.**

Until the language gains parametric types (enabling a generic `fold_expr`), the number
of full ExprData walks is capped at 5. Adding a 6th walk requires justification and
consolidation of an existing pair. The 5 allowed walks are:
1. `infer_and_resolve_expr` (reconcile) — type inference + resolution
2. `analyze_expr_calls` (reconcile) — call graph + recursion detection
3. `walk_expr` (ownership) — binding consumption classification
4. `cost_of_expr` (complexity) — symbolic cost computation
5. `emit_typed_expr` (emit, shared) — target-language rendering

**Cross-cutting invariant: import-from-authority, never define-at-use-site.**

Before defining a list, classifier, or predicate, check if an upstream module already
defines the same concept. If not, define it in the lowest shared module that all
consumers import, then import it. A fact defined at the use site will be copied to the
next use site.

**Cross-cutting verification:**
- [x] `rg -n '"String", "Int", "Bool"' src/v2/*.dag` returns only `00_core.dag`
- [x] `rg -n 'wrap_result_in_rc' src/v2/*.dag` returns 0 results
- [x] `rg -n 'pass_receiver_by_ref' src/v2/*.dag` returns 0 results
- [x] `rg -n 'needs_rc_wrap' src/v2/*.dag` returns only `05_emit_rust.dag`
- [x] `rg -n '\b(needs_rc_wrap|wrap_result_in_rc|pass_receiver_by_ref|Rc|borrow)\b' src/v2/00_core.dag` returns 0 results
- [ ] `grep -r '"Dynamic"' src/v2/` returns ≤5 results in `04_reconcile.dag`, all with justification comments
- [ ] `grep -r '_unimplemented\|/\* unhandled' src/v2/` returns 0 results
- [ ] `grep -rn 'ExprLiteral.*ExprVar.*ExprCall' src/v2/` — full 20-arm ExprData matches exist only in: `infer_and_resolve_expr`, `analyze_expr_calls`, `walk_expr` (ownership), `cost_of_expr`, `emit_typed_expr` (shared). Total: 5 walks, down from 11+.
- [ ] Adding a new ExprData variant requires editing ≤5 match arms (one per walk above)
- [ ] Adding a new intrinsic method requires editing 4 files: core (enum), reconcile (classifier), and one leaf function per target renderer

---

#### Theme 4: Kernel/Primitive Lists → Single Source of Truth

**Invariant:** No duplicate representations. Single-authority metadata.

**Problem:** 4+ copies of the same 6-8 type names (`kernel_type_names`,
`is_primitive_name`, `build_primitive_set`, `build_type_env` hardcoded list),
already drifting (`build_primitive_set` adds `"Char"`).

**Design:** Add to `00_core.dag`:
```
data kernel_types: List<String> = ["String", "Int", "Bool", "Float", "Secret", "Json", "Unit", "Bytes"]
fn is_kernel_type(name: String) -> Bool { kernel_types |> any(t => t == name) }
```

Delete `kernel_type_names()` in 03_resolve, `is_primitive_name()` in 04_reconcile,
`build_primitive_set()` in 05_emit, hardcoded list in `build_type_env`. All import
from core.

**Effort:** ~30 min. Zero risk.

---

#### Theme 6: Dead/Disconnected Infrastructure → Delete or Connect

**Invariant:** No fallbacks that fabricate. No parallel implementations.

| Dead code | Action |
|-----------|--------|
| Pipeline artifact stage (`_artifact_output`, lines 170-179) | Delete |
| `plan_artifacts` stub arms (ModuleBased/ServiceBased) | Delete |
| Artifact/Boundary types | Keep — forward-looking, types are cheap |
| Go pipeline dispatch (returns error despite emit_go existing) | Add `import emit_go`, wire `Go => emit_go(typed: typed)` |
| Trace `import std.types` | Fix to `import v2.std.core` |
| `build_module_vtoe` stub | Deleted |
| `resolve_sources` duplication | Extract shared tokenize→parse→resolve helper |
| `emit_record_lit` compat wrapper | Update tests, delete wrapper |
| Pipeline header comment | Fix to match reality |

**Effort:** ~1 hour. Low risk — mostly deletion.

---

#### Theme 3: String-Keyed Method Dispatch → Enum Everywhere

**Invariant:** No case enumeration for open sets. Single-authority metadata.

**Problem:** The enum pipeline is now mostly in place. `cost_of_expr` and
`receiver_size_var` dispatch on reconcile-provided `MethodSemantics`, and
reconcile now resolves known method semantics/result types once via
`resolve_known_method_node`. Residual string-based method logic still lives
in the source-to-semantics classifiers (`classify_reconciled_intrinsic_method`,
`classify_runtime_bridge_method`) and in per-target renderer leaf dispatch.

**Design:** After reconcile, every method call carries `MethodSemantics` with
`IntrinsicMethodSemantics { intrinsic: IntrinsicMethod, ... }`. Downstream phases
dispatch on the enum, never on strings.

Changes:
1. Keep `intrinsic_method_cost_shape(intrinsic) -> CostShape` as the single cost-shape authority
2. `resolve_known_method_node(...)` is the single reconcile authority for known method semantics and result types
3. `receiver_size_var(...)` reads `MethodSemantics`, not method-name strings
4. Delete the remaining string-based method dispatch downstream of the reconcile classifiers

Single authority chain:
```
string (source) → reconcile → IntrinsicMethod (enum)
                                  ↓
                    emit: rendering per intrinsic
                    complexity: CostShape per intrinsic
                    ownership: edge classification per intrinsic
```

**Effort:** ~2 hours. Medium risk — touches reconcile/emit/complexity.

---

#### Theme 5: Target-Specific Leakage → Ownership as Rendering Concern

**Invariant:** DAG nodes are facts, rendering is separate.

**Problem:** Rc wrapping still spans multiple places inside the Rust renderer
(`type_needs_rc`, pattern-deref heuristics, lookup-specific wrapping, DryRunMode),
even though the Rust-only fields and shared Rc indexes have now been removed from
core/reconcile/shared emit.

**Design:** Reconcile produces target-agnostic facts only. Rust-specific ownership
decisions move to a Rust-specific pre-pass within emit_rust.

Move FROM reconcile to emit_rust:
- `type_needs_rc`, `type_needs_rc_seen` → Rust renderer pre-pass [done]
- `data_lookup_needs_rc_wrap` → Rust renderer (`rust_lookup_receiver_needs_rc_wrap`) [done]
- `rc_wrapped` on TypeSummary → deleted from shared summaries; Rust derives Rc status locally [done]

Move FROM emit shared to emit_rust:
- `resolve_expr_type_node` → deleted; emitter now trusts typed nodes plus narrow local fallback [done]
- All 6 Rc-probing heuristics → compute once in a Rust-side pre-pass via `RcPatternAnalysis` / `RcMatchAnalysis` [done]

Clean up MethodSemantics:
- `wrap_result_in_rc` on IntrinsicMethodSemantics/RuntimeBridgeSemantics → Rust-only context [done in core semantics]
- `pass_receiver_by_ref` on RuntimeBridgeSemantics → same [done in core semantics]
- `needs_rc_wrap` on LookupCallSemantics → same [done in core semantics]

Target-agnostic facts reconcile SHOULD provide:
- "This type is recursive" (structural via children/connective)
- "This type participates in a cycle of depth N" (SCC analysis)
- "This value has N semantic consumers" (ownership analysis)

**Prerequisite:** None, but unblocks Theme 2.
**Effort:** ~3-4 hours. Higher risk — restructures reconcile/emit boundary.

---

#### Theme 1: Parallel ExprData Walks → Fuse Reconcile Passes

**Invariant:** No parallel implementations. No duplicate representations.

**Problem:** 8+ complete 20-arm ExprData walks. Reconcile still has 4. Adding one
expression kind edits 10+ match arms.

**Current reconcile walks:**
1. `infer_expr` — type inference (20 arms)
2. `resolve_expr_types` — type resolution (20 arms)
3. `collect_calls_in_expr` — call graph edges (20 arms)
4. `expr_self_call_info` — shared self-recursion + TCO eligibility walker in `00_core.dag` (20 arms, wrapped by `expr_has_self_call` / `expr_has_non_tail_self_call`)

**Fused design:**

Walk A (`infer_and_resolve_expr`): Combines (1) and (2). Infer each subexpression's
type, resolve it in the same traversal. Single 20-arm match.

Walk B (`analyze_expr_calls`): Combines (3) and the shared self-call analysis. Returns
`CallAnalysis { all_calls, has_self_call, has_non_tail_self_call }`.

Complexity module: `cost_of_expr` + `count_self_calls` → fuse into one walk.

Current reduction: the separate self-call/TCO walks have already been collapsed to one shared core walk. Final target remains 4 reconcile walks → 2. Cost of adding ExprData variant drops from 10+ to ~4.

**Effort:** ~4-5 hours. Medium risk — reconcile is the largest file.

---

#### Theme 2: Triple Renderer Parallelism → Shared Dispatch + Per-Target Leaves

**Invariant:** No parallel implementations.

**Problem:** 3× expression dispatch (60 match arms), 3× TCO, 3× services,
3× resources, 3× data. Python/Go only handle 7/18 intrinsic methods with
silent fabricating fallbacks.

**Design:** One shared expression dispatch in `05_emit.dag`, per-target leaf
functions in emit_rust/emit_python/emit_go.

Shared dispatch:
```
fn emit_typed_expr(texpr: Node, target: RenderTarget, ctx: EmitContext, ...) -> String {
  match texpr.expr_data {
    ExprLiteral { value: v } => target_literal(v, target)
    ExprMethodCall { ... } => target_method_call(recv_str, method, arg_strs, ms, target, ...)
    ...
  }
}
```

Per-target files shrink to leaf renderers only (`target_literal`, `target_call`,
`target_method_call`, `target_match`, etc.). TCO uses shared dispatcher with
per-target syntax config (`TcoSyntax { loop_open, break_prefix, continue_kw }`).

After Theme 3, each renderer provides `render_intrinsic(intrinsic, recv, args) -> String`
covering all 18 intrinsics.

**Prerequisite:** Theme 5 (move Rc to emit_rust) — otherwise shared dispatch
must thread Rust-specific state that Python/Go ignore.

**Size reduction estimate:**
- emit_rust: 3,571 → ~1,200 (leaves + Rust pre-pass)
- emit_python: 1,169 → ~400 (leaves only)
- emit_go: 1,196 → ~400 (leaves only)
- emit shared: 819 → ~1,500 (shared dispatch + helpers)
- Total: 6,755 → ~3,500 (~48% reduction)

**Effort:** ~8-10 hours. Highest risk — largest restructuring. Do last.

---

#### Theme 7: Fabricating Fallbacks → Fail Loud or Implement

**Invariant:** No fallbacks that fabricate. Correctness by construction.

**Problem:** Silent fallbacks produce valid-looking but wrong output.

| Fabrication | Action |
|-------------|--------|
| `Dynamic` as permissive wildcard (~25 reconcile sites) | Audit each: keep justified, fix lazy inference, convert error-masking to diagnostic. Target: <5 justified. |
| Python/Go intrinsic fallthrough (7/18) | Implement all 18 per language (Theme 2). Until then, emit `raise NotImplementedError` / `panic` with context. |
| Go `/* unhandled expr */` wildcard | Make match exhaustive — add remaining ExprData arms. |
| Transport panic/unimplemented | Emit language-specific compile error with context. |
| `plan_artifacts` ignoring config | Delete stubs (Theme 6). |

**Dynamic audit classification per site:**
- **Correct:** genuinely polymorphic position → keep
- **Lazy:** type could be inferred but isn't → fix inference
- **Error-masking:** inference failed silently → convert to diagnostic

**Effort:** Ongoing, ~1 hour per batch of 5 Dynamic sites. Interleave with Themes 1 and 5.

---

### Inference produces incomplete type structures that emit compensates for

**Invariant violated:** Correctness by construction, not by validation.

**Observation (2026-03-25):** `bare_map_node()` and `bare_list_node()` in
`04_types.dag` create container type nodes with zero children. These are
structurally incomplete — a `Map` without key/value children is not a fully
resolved type. Inference hands them to emit unchanged (via `empty_map()`,
`map_insert()`, `map_merge()` in `04_infer.dag` and `04_method.dag`).

The old per-backend emitters compensated with hardcoded fallbacks:
`"Map"` → `"BTreeMap<_, _>"`, `"List"` → `"Vec<_>"`. When the shared emitter
was extracted (P4.2), these compensations were initially lost. The shared
emitter now restores them (`emit_node_type_leaf_rc` bare container branch),
but the fix is in the wrong layer — emit shouldn't need to know that
inference might produce incomplete containers.

| # | Severity | Where | What |
|---|----------|-------|------|
| IV-1 | MED | `04_types.dag:76-78` | `bare_map_node()` creates Map with 0 children — structurally incomplete |
| IV-2 | MED | `04_infer.dag:1912-1921` | `empty_map()` returns bare container without resolving type params |
| IV-3 | MED | `04_method.dag:153,176` | `map_insert()`/`map_merge()` return bare_map_node |

**Direction:** Either inference resolves container type parameters from context
(bidirectional inference), or bare containers carry an explicit "unresolved
parameters" marker that emit can handle uniformly rather than per-backend.

---

### Silent type fabrication in emit

**Invariant violated:** No fallbacks that fabricate.

**Observation (2026-03-25):** Several emit code paths produce valid-looking
but wrong output instead of failing. The `"String"` fallback was the
canonical case — a multi-field anonymous product with a missing `return_type`
emitted `(String, SomeType)` as valid Rust that compiles but has the wrong
type. Single-field products correctly used `compile_error!`.

Fixed: multi-field anonymous product now uses `compile_error!` (2026-03-25).
CLI param type mapping (`05_emit_rust.dag:3584-3591`) still fabricates
`"String"` for structured/unknown types — left as-is because CLI surface
is P4.5 scope, but tracked here.

| # | Severity | Where | What |
|---|----------|-------|------|
| IV-4 | FIXED | `05_emit.dag:952` | Multi-field anonymous product: `"String"` → `compile_error!` |
| IV-5 | LOW | `05_emit_rust.dag:3584-3591` | CLI param type mapping fabricates `"String"` for unknown types |

---

### P4.6 bootstrap fix audit (2026-03-26)

The `bootstrap_stage0_to_stage1` test was failing with 147 cargo check
errors in emitted stage1 Rust. All 147 were fixed in PR #212. This
audit classifies each fix as either a **true root-cause fix** or a
**workaround** that papers over a deeper invariant violation.

#### True root-cause fixes (no invariant debt)

These fixes corrected genuine bugs at the right layer. No follow-up needed.

| Fix | Why it's correct |
|-----|-----------------|
| Named record literals in `languages.dag` (`ReservedWords`, `ProjectScaffold`, `SerializationSpec`, `TestConventions`) | Anonymous `{ field: value }` syntax IS a tuple in the .dag language. Using `TypeName { field: value }` is the correct way to construct named structs. Source-code bug, not an invariant issue. |
| Missing imports (`UnaryOpKind` in `05_emit.dag`, `InterpPart` in `05_emit_rust.dag`, `is_typed_service_call_receiver`/`extract_typed_service_name` in `05_emit.dag`) | Imports were genuinely missing after file decomposition. Correct layer for the fix. |
| `map_expr_children` param name `node:` → `expr_node:` in `04_resolve.dag` | Call site used wrong parameter name, causing emitter to output arguments in wrong positional order. Naming bug at the call site. |
| `return;,` syntax → let+return pattern in `05_emit_rust.dag` | `.dag` `return` inside match arms generates `return expr;,` in Rust (semicolon + comma). Restructuring to `let result = match { ... }; return result` avoids the issue at the .dag source level. |
| `adjacency_add_edge` helper in `03_resolve.dag` | Extracts fold body into a function with explicit `Map<String, List<String>>` parameter types. Gives inference the information it needs without fabrication. Honest .dag-level fix. |

#### Workarounds (invariant debt — needs follow-up)

| # | Fix | Invariant violated | Root cause | Deletion point |
|---|-----|-------------------|------------|----------------|
| IV-6 | `empty_map()` → `BTreeMap::new()` in `emit_typed_call_expr` (`05_emit_rust.dag:1835`) | **No fallbacks that fabricate.** Emit silently drops the turbofish and hopes Rust's type inference recovers the value type from context. If Rust can't infer, this produces a different error (E0282) instead of the correct type. | Inference does not propagate expected parameter types to argument expressions. `empty_map()` as an argument to `f(rc_types: Map<String, Bool>)` should infer `Map<String, Bool>`, not `Map<String, Unit>`. **Bidirectional type inference is missing.** | Fix inference to propagate expected types from function signatures to argument expressions. Then emit can use the turbofish with the correct type. Extends IV-1/IV-2. |
| IV-7 | Fold init `empty_map()` with unit-child detection (`05_emit_rust.dag:2302-2310`) | **No fallbacks that fabricate** + **Heuristics indicate lost structure.** Emit inspects the acc type node's children for `"Unit"` or `""` names to decide whether to use turbofish or partial `<BTreeMap<String, _>>::new()`. This is a heuristic that compensates for inference producing incomplete types. | Same as IV-6: inference doesn't resolve fold accumulator type parameters from the fold body. The `acc_type_node` carries `Map<String, Map<String, Unit>>` when the fold body clearly produces `Map<String, List<String>>`. | Fix inference to propagate fold body return type back to the accumulator type. Then emit receives complete types and the heuristic is unnecessary. |
| IV-8 | Fold acc type resolution with unit-child fallback to contextual type (`05_emit_rust.dag:2277-2284`) | **Heuristics indicate lost structure.** Emit checks `acc_type.children |> any(c => c.name == "Unit")` to decide whether to use the contextual (method result) type instead of the inferred accumulator type. | Same root cause as IV-6/IV-7. The emit layer is doing type resolution work that belongs in inference. | Same deletion point as IV-7. |
| IV-9 | `go_source_extension` → inline literal `".go"` in `languages.dag:163` | **No duplicate representations.** The value `".go"` is now defined in both `dsl/extdeps/languages/go/emit.dag:65` (as `data go_source_extension`) and inline in `languages.dag`. They will diverge if either changes. | The emitter inconsistently transforms `data` constant names to SCREAMING_SNAKE_CASE in import `use` statements. 6/7 Go extdep data constants are correctly uppercased; `go_source_extension` is not. Import emission doesn't distinguish function imports (stay snake_case) from data constant imports (should be SCREAMING_SNAKE). | Fix the import emission in `05_emit_rust.dag` to consistently apply SCREAMING_SNAKE_CASE for `data` constant imports. Then restore the import in `languages.dag` and delete the inline literal. |

#### Underlying root cause: no bidirectional type inference

IV-6, IV-7, and IV-8 all trace to the same root cause: **inference is
top-down only.** It resolves types from declarations and expressions
forward, but does not propagate expected types backward from:

- Function parameter signatures to argument expressions
- Fold accumulator usage in the body back to the init expression
- Let-binding type annotations back to the initializer

This is not a new finding — IV-1/IV-2 (2026-03-25) already identified
the incomplete container types. The P4.6 fixes expose the same root
cause at 124+ additional sites (every `empty_map()` call where the
value type is unresolved).

**Scope:** This is a Phase 5+ fix (inference architecture). The current
workarounds are viable because Rust's own type inference recovers the
correct types in all 124+ sites. But they are fabrications: emit
produces `BTreeMap::new()` instead of `<BTreeMap<String, bool>>::new()`,
relying on a downstream system (rustc) to compensate for information
the pipeline lost.

#### Return-in-match-arm emitter bug (not fixed, worked around)

The `return;,` fix restructured the .dag source to avoid `return` in
match arms, but the underlying emitter bug remains: when a `.dag`
`return` statement appears as a match arm body, the emitter generates
`return expr;,` (semicolon from statement termination + comma from match
arm separation). Any future .dag code using `return` inside match arms
will hit the same issue.

| # | Severity | Where | What |
|---|----------|-------|------|
| IV-10 | LOW | Emitter match-arm rendering | `return` in match arm body emits `return expr;,` — Rust syntax error. Workaround: use let+return pattern. Fix: emitter should suppress trailing `;` when the match arm body is a return/break/continue. |

---

### CollectionKind — DISSOLVED (2026-03-28)

**Previously:** `CollectionKind` enum (6 variants) on `Node`, 184 sites
across 17 files. Compiler branched on enum to distinguish collection types.
Every `Node { ... }` literal had to set `collection_kind` correctly or
`node_is_map`/`node_is_container` silently returned false.

**Resolution:** Enum deleted, field removed from Node. Collection-ness is
now derived structurally after resolution: containers are the only type
nodes with `children > 0 && connective == NoConnective`. Three structural
predicates (`node_is_collection`, `node_is_keyed_collection`,
`node_is_element_collection`) replace all enum matching. A `container_types`
data list in `00_core.dag` controls which types stay unexpanded during
resolve. Emit uses `to_snake(n.name)` as LanguageSpec template key.

| # | Status | What |
|---|--------|------|
| IV-11 | **FIXED** | `CollectionKind` enum deleted |
| IV-12 | **FIXED** | `collection_kind_for_name` deleted |
| IV-13 | **FIXED** | Normalization block deleted (no field to normalize) |

---

### L1 ratchet increase audit: 371 → 414 (+43) (2026-03-26)

Systematic root-cause analysis of the +43 L1 ratchet increase between
commit `597d852b` (ratchet set to 373) and current HEAD.

#### Source: file decomposition (+18)

Code moved from `04_infer.dag` (-61 sites) into 7 extracted modules
(+79 sites). Net +18 because:

- **+13 import lines.** Each extracted module imports predicates and
  constructors it uses (`node_is_optional`, `leaf_node`, etc.). These
  are NOT new type knowledge — the same call sites existed in the
  monolith. The ratchet script counts `\bnode_is_\w+\b` in import
  lists.
- **+5 expanded logic.** During extraction, some code was slightly
  restructured (e.g., adding explicit predicate calls where the
  monolith had inline field checks).

**Classification:** Not invariant violations. File decomposition is
structural improvement. The ratchet increase is a measurement artifact
— import lines are not "compiler type knowledge."

**Ratchet script improvement opportunity:** Exclude `^import` lines
from the count, or weight them differently.

#### Source: P5.7a bridge predicates (+7)

`04_types.dag` gained 7 new `node_is_*` sites:

| Site | What | Classification |
|------|------|---------------|
| `node_is_bridge_error_name` (def + 4 calls) | Centralizes `n.name == "Error"` check that was previously inline in `node_type_equals`/`node_type_compatible` | **Bridge.** Explicitly named as temporary (prefix `bridge_`). Deletion point: P5.6/P5.8 when Error becomes `CompilerError` flow. |
| `node_is_bridge_dynamic_name` (def + 3 calls) | Same for `n.name == "Dynamic"` | **Bridge.** Same deletion point. |
| `node_is_product`/`node_is_coproduct` (P5.7a rewrites) | Changed from `properties \|> any(p => p.name == "is_product")` to `n.connective == Some { value: Conj }` | **Improvement.** Replaced uncounted string-property check with counted structural check. Net reduction in actual type knowledge (deleted duplicate representation). |

**Classification:** 5 are explicit bridge code with deletion points.
2 are structural improvements that trade uncounted violations for
counted ones (net positive).

#### Source: P5.7b CollectionKind (+4 connective, +3 Conj/Disj)

P5.7a deleted `is_product`/`is_coproduct` property strings and made
predicates read `.connective` directly. This moved sites from the
uncounted property-string pattern to the counted `.connective` pattern.

| Category | Old pattern (uncounted) | New pattern (counted) |
|----------|------------------------|----------------------|
| `.connective` +4 | `properties \|> any(p => p.name == "is_product")` | `n.connective == Some { value: Conj }` |
| `Conj/Disj` +3 | property string `"is_product"` / `"is_coproduct"` | `Conj` / `Disj` literal in predicate match |

**Classification:** Structural improvement. The old code was WORSE
(string-keyed property checks) but uncounted. The new code is BETTER
(typed field access) but counted. No invariant violation — the ratchet
script should have been counting the old pattern too.

#### Source: emit type-name comparisons (+4)

`05_emit_rust.dag` +3 and `05_emit.dag` +2 new `.name == "..."` checks.

| Site | What | Classification |
|------|------|---------------|
| `05_emit_rust.dag` typename checks | `effective.name == "List"`, `"Vec"`, `node_is_container` in intrinsic method dispatch | **Emit rendering.** Emit legitimately reads names for target identifiers. Not an L1 violation — emit is excluded from the L1=0 gate (scrambled-name tests exclude emit). |
| `05_emit.dag` typename checks | Service call detection, simple expression rendering | **Emit rendering.** Same classification. |

**Classification:** Legitimate emit rendering. These are NOT L1
violations — the L1 gate (P5.6 scrambled-name tests) explicitly
excludes emit because emit must read names to produce target
identifiers.

#### Source: parse/compile structural production (+9)

`02_parse.dag` +6, `compile.dag` +3.

| Site | What | Classification |
|------|------|---------------|
| `02_parse.dag` connective +1, constructors +2, predicates +3 | Parser creates `Conj`/`Disj` nodes and calls `node_is_optional` for cardinality | **Parse production.** The parser MUST produce structural nodes. Not "type knowledge the compiler has" — it's "structure the parser creates." |
| `compile.dag` connective +1, conj_disj +2 | Pipeline orchestration reading connective for complexity/ownership staging | **Pipeline wiring.** Compile stage reads structural properties to route to proof stages. |

**Classification:** Necessary structural production/wiring. Not
violations.

#### Source: 03_resolve.dag adjacency helper (+2)

`adjacency_add_edge` adds 2 typename comparisons (from P4.6 fix).

**Classification:** See IV-6/IV-7 — this is a workaround for missing
bidirectional type inference. The helper's explicit `Map<String,
List<String>>` type annotation provides what inference should propagate.

#### Summary

| Source | Sites | Classification | Action |
|--------|------:|---------------|--------|
| File decomposition (imports) | +13 | Measurement artifact | Fix ratchet script to exclude import lines |
| File decomposition (logic) | +5 | Moved code, not new knowledge | None |
| P5.7a bridge predicates | +7 | 5 explicit bridge, 2 improvement | Bridge deletion at P5.6/P5.8 |
| P5.7a/b connective migration | +7 | Improvement (counted replaces uncounted) | None — old uncounted pattern was worse |
| Emit rendering | +4 | Legitimate (emit excluded from L1 gate) | None |
| Parse/compile production | +9 | Structural production | None |
| Adjacency helper | +2 | Workaround (IV-6/IV-7) | Fix bidirectional inference |
| **Total** | **+47** | | |

(+47 gross, -4 from other reductions = +43 net)

**Conclusion:** Of the +43 net increase, **0 are new invariant
violations.** The increase comes from: measurement artifacts (+13),
moved code (+5), structural improvements that trade uncounted
violations for counted ones (+7), legitimate emit/parse/pipeline
sites (+13), and workarounds for pre-existing violations (+7 bridge +2
adjacency). The ratchet script should be improved to exclude import
lines and potentially emit-only files.

---

### Cleanup

| # | Severity | Description |
|---|----------|-------------|
| F5 | LOW | `infer → reconcile` rename lacks documented contract justification. |
| SG-9 | LOW | .dag workarounds for force_clone (TokPos extraction, branch-aware use counting). Revert after verification at scale — may be redundant after R9. |
