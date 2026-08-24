# Affected-set emission: five compiler-side mechanisms

Pinned tree `faf6583461a4f7d042ae670e563758869d439159`.
Entry `src/v2/lens/affected_set.dag`.
Instrument `docs/probes/curated_cargo_probe_one.sh` (emit -> cssl_assemble -> `cargo build --release --lib`).
Board: **123 error lines, 122 primary sites** (the 123rd is the span-less `could not compile` summary).

This document describes defects. **It proposes no repairs**, deliberately: the aim is to make each
defect cheap to understand, not to pre-decide its fix. That is a stance on *repair design*, and it is
not a licence to leave the classes untracked — §4b(2) forbids a silent stall, so every mechanism below
carries a disposition and a next trigger in the [Disposition](#disposition) section, including the ones
whose honest disposition is *unowned*. Naming what would move a class is not the same as deciding how
to fix it.

## Scope and evidence status

Of the 122 primary sites, **24** land in `v2_lens_application.rs`, `std_change.rs` and
`v2_lens_affected_set.rs`. Those 24 partition with no residue into six mechanisms. Five are
compiler-side and are documented here; the sixth (`ABSENT_CLONE_BOUND`, 2 rows) is an already-established
mechanism owned elsewhere and is listed only so the arithmetic closes.

| id | mechanism | rows | evidence |
|----|-----------|------|----------|
| A | coproduct realization has no unused-parameter carrier | 10 | **measured** (executed counterfactual) |
| B | callable lifetime obligations arise at `Rc<dyn Fn>` materialization and propagate | 4 | **measured** (counterfactual, negative) |
| C | `ABSENT_CLONE_BOUND` (established, not documented here) | 2 | read |
| D | authority substitution after resolution (emission rebinds to a v1_rt builtin) | 5 | **measured** (peer board, 28 blocks) |
| E | an undetermined empty-list element type is answered as `unit`, silently | 2 | **measured** (counterfactual, negative) |
| F | fold's unused-element strip changes the item type without telling the signature | 1 | **measured** (counterfactual) — **REPAIRED**, gunbc#9101 |

10 + 4 + 2 + 5 + 2 + 1 = 24.

**Measured** means an executed experiment discriminates the claim. **Read** means the mechanism is
derived from the rustc text and the `.dag` source without an executed counterfactual. That distinction
is load-bearing and is not flattened anywhere below.

---

## A — coproduct realization has no unused-parameter carrier

**Rows:** 10 (`E0392` x4, `E0282` x6), all in `v2_lens_application.rs`. **Measured.**

### Source

`v2.lens.application` `LensApplicationConfig` (`src/v2/lens/application.dag`)

```dag
type LensApplicationConfig<Output, Budget, Projected>
  = LensIntrospect
  | LensEnforce {
      budget: Budget
    }
```

Three type parameters declared; only `Budget` appears in a variant payload. The sole instantiation is the `config` field of `v2.lens.application` `apply_advisory_lens`,
`LensApplicationConfig<Report, Report, Report>`. The sibling `EnforcedApplication<Output, Budget, Projected>`
passes the same trio through to `EnforceableLens`, so the three-parameter signature is a module-wide shape.

### Emitted

```rust
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum LensApplicationConfig<Output, Budget, Projected> {
    LensIntrospect,
    LensEnforce { budget: Budget },
}
```

The emission is faithful to the declaration. Rust rejects it: an unused type parameter is an error,
and the `serde` derives cannot infer for parameters that appear in no field.

### rustc

```
error[E0392]: type parameter `Output` is never used
  --> src/v2_lens_application.rs:97:32
error[E0392]: type parameter `Projected` is never used
  --> src/v2_lens_application.rs:97:48
error[E0282]: type annotations needed
  --> src/v2_lens_application.rs:95:10        <- the #[derive(...)] line
```

### Relevant existing policy

`src/v1/05_emit_rust.dag` already implements an unused-parameter carrier for two of three item kinds:

- struct realization: `struct_unused_param_names` appends a `_phantom: std::marker::PhantomData<...>` field
- alias realization: `alias_unused_param_names` emits the RHS as `PhantomData<...>`
- **coproduct realization: no analogous carrier**

So the policy exists and is implemented; one item kind does not participate. This is stated because it
bears on how the mechanism is understood — an un-migrated item kind, not an undecided question.

### Counterfactual (executed)

Prediction registered **before** the run: dropping the two structurally-absent parameters retires exactly
10 rows, and the `E0308` at `:107` **survives**, because that row belongs to `ABSENT_CLONE_BOUND` (mechanism C)
and is a different mechanism.

|  | control | `LensApplicationConfig<Budget>` |
|---|---|---|
| board primary | 122 | **112** |
| `v2_lens_application` primary | 11 | **1** |
| `E0392` board-wide | 4 | **0** |
| `E0282` board-wide | 9 | **3** |

Exactly 10 retired. The 6 `E0282` removed board-wide are precisely the 6 that were in this file. The
`:107` `E0308` survived, as predicted.

**No row relocated.** Thirteen codes identical across both arms: `E0308` 43/43, `E0004` 19/19, `E0560` 9/9,
`E0277` 6/6, `E0614` 5/5, `E0609` 5/5, `E0599` 5/5, `E0369` 4/4, `E0310` 4/4, `E0425` 3/3, `E0061` 3/3,
`E0631` 2/2, uncoded 1/1. Rows were retired, not moved.

The surviving `:107` row is **measured** independence of A and C, not assumed independence.

**What the counterfactual establishes:** in this pinned closure, no static or runtime behaviour depends on
`Output` or `Projected` being parameters of this type. **What it does not establish:** that the parameters
were never intended to carry anything. A zero-occupancy result is also what a placeholder for an unbuilt
variant produces.

---

## B — callable lifetime obligations arise at `Rc<dyn Fn>` materialization and propagate through callable-valued parameters

**Rows:** 4 (`E0310` x4), in `std_change.rs`. **Measured** — and the measurement SUPERSEDED this
mechanism's description. It was recorded as *fn-typed params captured into an `Rc` closure demanding
`'static`*, which named the first symptom as though it were the mechanism. It is not: the obligation is
created wherever a Rust `dyn` callable is materialized, and it **propagates transitively through
callable-valued parameters**. The counterfactual below is what forced the rewrite, and it is a
NEGATIVE result — the repair that follows from the old description relocates rows instead of retiring
them.

### Source

`std.change` `keyed_three_way_patch_monoid` (`dag/std/change.dag`)

```dag
fn keyed_three_way_patch_monoid<K, V>(
  key_eq: fn(K, K) -> Bool,
  value_eq: fn(V, V) -> Bool,
) -> Monoid<KeyedThreeWayPatch<K, V>> {
  Monoid {
    op: fn(left, right) {
      keyed_three_way_patch_append(left: left, right: right, key_eq: key_eq, value_eq: value_eq)
    },
    identity: keyed_three_way_patch_empty(),
  }
}
```

A monoid parameterized by its equality relations. The `op` closure captures both.

### Emitted / rustc

The parameters realize as `impl Fn(K, K) -> bool + Clone` and the closure is stored behind `Rc`, which
requires the captured types outlive `'static`. No such bound is emitted.

```
error[E0310]: the parameter type `K` may not live long enough
   --> src/std_change.rs:140:9
error[E0310]: the parameter type `impl Fn(K, K) -> bool + Clone` may not live long enough
   --> src/std_change.rs:203:73
    = help: consider adding an explicit lifetime bound
```

Four rows: `K` and `V` at `:140`, and the two `impl Fn` types at `:203`. Two sites, four parameters.

### The two obligations, which the old description collapsed into one

The four rows are not four instances of one thing. They are two mechanisms that share an error code:

1. **Closure environment lifetime.** `op: fn(left, right) { .. key_eq .. }` lexically captures `key_eq`
   and `value_eq`; the values it captures must satisfy the target callable's lifetime requirement.
   A capture walk can see this one.
2. **Callable value lifetime.** `op: Rc::new(keyed_patch_append)` — a **bare function reference**, no
   lexical capture at all, so no capture walk can see it — yet the resulting type is still
   `Rc<dyn Fn(..) + 'static>` and the callable value itself must satisfy `'static`. The `K` and `V`
   rows are this one, and a bound on the fn-typed *parameters* cannot discharge it, because the
   obligation is on the type parameters the `dyn Fn` mentions.

### Counterfactual (executed) — the local repair RELOCATES, it does not retire

The repair the old description implies: derive `+ 'static` per parameter from the capture set of a
wrapped closure, covering the record-field wrap site that the emitter's existing return-connective
gate cannot see. Implemented, regenerated, installed, and measured on this tree — both arms, binary
stamps cleared between them, with the positive control taken on the **installed mirror** rather than on
the source tree.

|  | control | with the local repair |
|---|---|---|
| board primary | 100 | 100 |
| `E0310` | 4 | **4** |

Same count, **different four** — which the count alone cannot show and only reading the blocks does:

- `K` / `V` still refused at `op: Rc::new(keyed_patch_append)` — obligation 2, untouched, as it must be.
- the two `impl Fn` rows **moved from the definition site to the call site**, now refused at
  `keyed_three_way_patch_monoid(key_eq.clone(), value_eq.clone())` inside `std_change` itself.

The repair discharges the obligation where it is created and re-creates it one call up, in a caller
whose own fn-typed parameters carry no `'static`. By the standard mechanism A's counterfactual set —
*rows were retired, not moved* — this is not a repair, and it is not landed. The emitted specimen did
change as intended (`key_eq: impl Fn(K, K) -> bool + Clone + 'static`) and the generation-2 seed built
with zero errors, so the negative result is about the mechanism, not about the patch being broken.

### What the emitter's own note claimed, and what replaces it

`v1.compiler.emit_rust` `emit_rust_param_type` gates the bound on the enclosing function returning an
arrow, and its note calls that "precise rather than a proxy" because the `Rc::new(move |..|)` site
"exists exactly when the function returns an arrow". The `exactly` is false: `keyed_three_way_patch_monoid`
returns a RECORD with an arrow-typed field, reaches the second wrap site
(`rust_callable_field_value_wrap`), and the return-type test cannot see it. The replacement invariant the
measurement supports is stronger than a wider return-type test:

> the obligation site is **every lowering site that creates or accepts a Rust `dyn` callable requiring
> `'static`** — not every source construct whose return type happens to be arrow-shaped.

Deriving that correctly is a fixpoint over the call graph (infer callable lifetime requirements,
propagate them backward through callable-valued arguments, repeat to stability, with SCC handling for
recursion). That is a lifetime-propagation engine, not an affected-set repair: this board **exposed**
the mechanism, it does not **own** it — the same ownership line that keeps mechanism D with #9060.

**Instrument note, because it nearly cost this result.** A first attempt at the arms produced a perfect
null — `E0310` 4 → 4, every code identical — from an arm that could not have shown anything: `claim_executor`
was never built in that dispatch, so regen produced no candidate, the install silently no-opped, and the
"repair" arm measured the same committed mirror as the control. It was caught by the `cp: cannot stat`
line, not by the numbers, which were entirely plausible. Every arm behind the table above carries a
positive control on the artifact under test.

---

## D — authority substitution after resolution: emission rebinds a resolved v2 `std` call to a v1_rt builtin

**Rows:** 5 in this closure (`E0061` x2, `E0282` x2, `E0614` x1). **Measured on a peer board.**

Two names denote different concepts in the two layers:

| name | v1_rt (`v1.runtime_rust`) | v2 `std` |
|------|------------------------------------|----------|
| `contains` | `fn contains(s: String, sub: String) -> bool` — substring test | `fn contains<T>(xs: FreeMonoid<T>, item: T, eq: fn(T,T) -> Bool) -> Bool` (`v2.std.algebra` `contains`) — collection membership |
| `map_get` | `fn map_get<K,V>(m: &HashMap<K,V>, key: K) -> Option<V>` | `fn map_get<K,V>(m: Map<K,V>, key: K) -> Outcome<Optional<V>>` (`v2.std.collection` `map_get`) |

`map_insert` and `empty_map` collide by name as well and produce **zero rows on either board today**.
They are **not the same case** and must not be treated as one category:

- **`empty_map` — a positive control.** `v2.std.collection` carries `empty_map_host_binding: PrimitiveContract`,
  and `empty_map` routes through `empty_map_primitive_delegate`, a self-recursive stub the host intercepts.
  It is genuinely primitive-bound. After any repair it must **still** lower through the runtime bridge; a fix
  that makes every corpus declaration beat the registry breaks it while looking like success.
- **`map_insert` — an unresolved specimen.** It carries `map_insert_host_binding: PrimitiveContract` *and*
  authors a closure-backed body implementing insertion. The binding says primitive, the body says modeled,
  and nothing in the tree chooses. Zero rustc rows does not resolve that, and a rewrite compiling does not either.

### The imports are explicit

`src/v2/lens/affected_set.dag`

```dag
import v2.std.collection { Absent, List, Map, Present, list_at_optional, map_get, map_insert }
import v2.std.algebra    { TailAbsent, TailFound, contains, length, list_append, ... }
```

and the call sites use named arguments matching the v2 signature:

```dag
fn dependency_kind_in_lens_frontier_policy(kind: DependencyKind) -> Bool {
  contains(xs: lens_frontier_dependency_kinds(), item: kind, eq: fn(left, right) { left == right })
}
fn dimension_in_list(xs: List<AffectedDimension>, item: AffectedDimension) -> Bool {
  contains(xs: xs, item: item, eq: fn(left, right) { left == right })
}
match map_get(decisions, key) { ... }   // in `mark_excluded`
```

**Source resolution is not broken.** The exact imported declaration *is* found. Rust emission then
discards the declaration identity, looks the bare spelling up in `rt_functions()`, and rebinds. That is
**authority substitution after resolution** — a failure mode DESIGN.md names — and it is a stronger
statement than "shadowing" or "a resolution defect": the deciding fact is written, resolved, and
available, and nothing forces the emitter to consume it.

### Emitted / rustc

```rust
v1_rt::contains(lens_frontier_dependency_kinds(), kind.clone(), |left, right| (left.clone() == right.clone()))
```

```
error[E0061]: this function takes 2 arguments but 3 arguments were supplied
   --> src/v2_lens_affected_set.rs:404:5
    |     expected `String`, found `Rc<Vector<DependencyKind>>`
note: function defined here
   --> src/v1_rt.rs:307:8
    | pub fn contains(s: String, sub: String) -> bool { string_contains(&s, sub) }
```

and for `map_get`, where the v2 signature returns `Outcome<Optional<V>>` but the bound builtin returns `Option<V>`:

```
error[E0614]: type `std::option::Option<Rc<FrontierDecision>>` cannot be dereferenced
   --> src/v2_lens_affected_set.rs:692:93
    | match (*v1_rt::map_get(&decisions, key.clone())).clone() {
```

### Relevant existing policy

The compiler already implements *resolve once, carry the identity, realize that identity* — for one call form.
`v1.compiler.core` declares:

```dag
type CallSemantics
  = PlainCallSemantics
  | LookupCallSemantics
  | FunctionValueCallSemantics
```

and `05_emit_rust.dag` records, at the site that consumes it, that the compiler answers the
"question once, where scope is known, and records it as `FunctionValueCallSemantics`." There is also a
corpus fixture for the adjacent shape: `fixtures/builtin_shadow/free_call_shadow_specimen.dag`.

### Where the substitution happens

Three sites in `05_emit_rust.dag` consult the runtime table by leaf spelling. A grep for
`map_contains_key(rt_functions()` returns exactly these — the census is exhaustive, not a sample:

```dag
v1.compiler.emit_rust `emit_typed_call`
  let is_rt = callee_is_function_value == false && map_contains_key(rt_functions(), func)

v1.compiler.emit_rust `emit_rust_generic_method_call`
  } else if map_contains_key(rt_functions(), function_name) == false {

v1.compiler.emit_rust `rust_receiver_has_callable_method_field`
  if map_contains_key(rt_functions(), method_name) {
```

The site in `emit_typed_call` is the mechanism in one expression. The **first** conjunct is keyed on resolved semantics
(`callee_is_function_value`); the **second** is keyed on `func`, the leaf spelling. So emission does not ignore
`CallSemantics` — the carrier as declared distinguishes only function-value from not-function-value, and
`PlainCallSemantics` carries no target identity for the second conjunct to consult. **The substitution happens
inside a single `&&`.**

The three sites are three call forms of one mechanism, which is why completion must be measured as
*post-resolution callable identity substitution = 0* rather than per rustc code: each form surfaces as whatever
rustc happens to say about the wrong callee it reached, so a per-code measure can green on one form while the
mechanism stays live in the others. The 28 measured blocks already span `E0061`, `E0282` and `E0614`.

So a recorded-semantics seam exists and emission consumes it *for function values*. Direct callable targets
have no analogous recorded identity, which is why emission re-decides them from the leaf spelling. Stated here
as a property of the defect — the same way mechanism A's struct/alias `PhantomData` policy is stated — not as a
proposed repair.

### Measurement

Counted by peer session `smart-ram-730` on `docs/probes/board_2026-08-23/03_ingest.cargo.log`
(331 error blocks), counting **distinct error blocks whose span mentions the builtin**, not string occurrences:

```
v1_rt::contains     22 blocks    E0061 x17, E0282 x5
v1_rt::map_get       6 blocks    E0614 x6
v1_rt::map_insert    0
v1_rt::empty_map     0
```

**28 of 331 blocks — 8.5% of that board — from two name collisions.** All 17 `E0061`s read
"takes 2 arguments but 3 arguments were supplied", which is the three-argument v2 `contains<T>(xs, item, eq)`
arriving at the two-parameter substring builtin.

The mechanism spans **three rustc codes simultaneously** (`E0061`, `E0282`, `E0614`), which is why a
code-partitioned view does not surface it: each code's rows look like unrelated minorities.

---

## E — an undetermined empty-list element type is answered as `unit`, silently

**Rows:** 2 (`E0308` x1, `E0631` x1, same line). **Measured** — and, like B, the measurement moved the
mechanism upstream of where the rows appear. The rows are a *downstream symptom*; the defect is that
inference fabricates an answer where it has none.

### Source

`std.change` `keyed_collect_keys` (`dag/std/change.dag`)

```dag
reverse(rows |> fold(init: [], f: fn(acc, row) {
  if fold(acc, init: false, f: fn(found, k) { found || key_eq(k, row.row_key) }) {
    acc
  } else {
    concat([row.row_key], acc)
  }
}))
```

The outer fold is initialized `init: []`. Its element type is determined by the returning branch,
`concat([row.row_key], acc)`, i.e. `K`.

### Emitted / rustc

```rust
acc.iter().cloned().fold(false, |found: bool, k: ()| (found || key_eq(k.clone(), row.row_key.clone())))
```

The element type is emitted as `()`.

```
error[E0631]: type mismatch in closure arguments
   --> src/std_change.rs:279:120
    = note: expected closure signature `fn(_, K) -> _`
               found closure signature `fn(_, ()) -> _`
error[E0308]: mismatched types
   --> src/std_change.rs:279:170
```

Both rows are the same emitted line.

### The producing arm, and why this is a §5 defect rather than a missing feature

`v1.compiler.infer`, the `ExprListLit` arm, chooses the element type of an empty literal:

```dag
Absent => unit_type      // elem_type_node, when there is no expected type
...
Absent => []             // empty_list_diags — no diagnostic
```

Its two siblings refuse with located diagnostics — *"empty list literal: expected type has no element
type"* and *"...expected type is not a collection"*. The arm reached when there is **no expected type at
all** answers `unit` and says nothing. That is *"I could not determine the element type"* rendered as
*"the element type is unit"*: the fabricated plausible output §5 forbids, and the ⊥-as-answer /
⊥-as-ignorance conflation in the recurring-failure list.

It also explains the distance between cause and report. `fold(init: [], ..)` takes the fabricated
`unit`, the accumulator becomes `List<()>`, the inner fold's binder emits as `|found: bool, k: ()|`, and
rustc refuses **that** — a message naming neither the empty literal nor the missing expected type, two
stages from the decision that caused it.

### Counterfactual (executed) — the local refusal is NOT affordable

The repair the defect invites: make that arm refuse instead of fabricating. Implemented, regenerated,
installed, rebuilt, and run against this entry, with the positive control taken on the installed mirror
(`CONTROL_installed=1`, generation-2 build clean at 0 errors).

| | result |
|---|---|
| sites hitting the fabricating arm, **affected-set closure alone** | **24** |
| entry's blocking diagnostics, control → refusal | 0 → **12** |

The fabrication is load-bearing: 24 empty-list literals in this one closure reach that arm, and refusing
locally does not surface E's two rows — it stops the entry compiling at 12 blocking errors. A refusal
that converts one downstream rustc row into twelve blocking refusals is not the fail-closed repair; it
is the same local-patch error B already paid for, in the opposite direction.

**Where E actually lives.** The `fold(init: [])` site *has* a determinable element type — the outer
fold's accumulator — and inference lacks it only because the callee's type variable is unsolved at the
point the literal is judged. The repair is therefore upstream: solve the accumulator type variable, or
defer the literal's judgement until the expected type is known, so the arm becomes unreachable rather
than refusing. Until that lands, the fabrication is the *only* thing keeping 24 sites compiling, which
is why the arm is left exactly as `main` has it rather than reddened.

---

## F — fold's unused-element strip changes the item type without telling the closure signature

**Rows:** 1 (`E0631`). **Measured, and REPAIRED** — gunbc#9101. One producer fact, `elem_unused`, decides
two things: it strips `.cloned()` (so the iterator yields `&T`) and it should decide the lambda's element
annotation (which still declared `T`). The second consequence was never told. `emit_typed_fold_lambda`
now forces `_` at the element position, so inference supplies whichever of `T` / `&T` the iterator yields.
Counterfactual, both arms on one tree with the positive control on the installed mirror: board primary
100 -> 99, `E0631` 2 -> 1, eleven other codes byte-identical, generation-2 seed clean. Retired, not
relocated. The surviving `E0631` is mechanism E's — a different mechanism sharing the code.

### Source

`v2.lens.affected_set` `affected_set_closure` (`src/v2/lens/affected_set.dag`)

```dag
let state = fold(
  affected_set_closure_convergence_rounds(dependencies: dependencies),
  init: AffectedSetClosureFixpointState { ... },
  ...
)
```

### Emitted / rustc

Emitted as `.iter().fold(...)` — which yields references — with a by-value closure parameter:

```
error[E0631]: type mismatch in closure arguments
   --> src/v2_lens_affected_set.rs:539:90
    = note: expected closure signature `fn(Rc<_>, &Rc<_>) -> _`
               found closure signature `fn(Rc<_>, Rc<_>) -> _`
help: consider adjusting the signature so it borrows its argument
```


---

## Disposition

§4b(2) separates *cannot move* from *can move after one grounding* from *can move now but unbuilt*, and
requires the distinction to be stated rather than inferred. None of these six is in the first category.
The trigger for a **read** mechanism is always the same shape — an executed counterfactual at the pinned
tree that discriminates the claim — and that is deliberately a trigger, not a repair: it establishes what
the mechanism *is* before anyone designs a fix, which is the same order this document's two measured
mechanisms already went through.

| id | evidence today | disposition | next trigger |
|----|----------------|-------------|--------------|
| A | measured | **repair open: #9041** | merge retires the 10 rows. One question survives the merge and the board cannot answer it: a zero-occupancy counterfactual does not distinguish a dead parameter from a placeholder for an unbuilt variant. That belongs to `v2.lens.application`'s author, not to this board. |
| B | **measured** (executed counterfactual, negative) | **unowned, and reclassified** — the local repair was implemented, measured, and rejected; what remains is a lifetime-propagation engine this board does not own | build the transitive obligation derivation: requirements inferred at every `dyn`-callable materialization site and propagated backward through callable-valued parameters to a fixpoint. Until then the 4 rows stand, and the honest reason is recorded rather than a trigger nobody can act on. |
| C | read | **owned elsewhere** — the corpus-wide `ABSENT_CLONE_BOUND` population (`docs/probes/rustc_mechanism_partition_2026-08-23.md`, 22 manifestations at `967b5bc1b92`) | none here. This board contributes 2 rows to that population and tracks nothing separately; a second trigger beside that document's would be a second authority for one class. |
| D | measured (peer board) | **lane open: #9060**, which states itself to be PR A of the resolved-call identity repair and reserves PR B for carrying resolved callable identity through all three Rust-emission seams | PR B of that lane. The seam this board adds is a defect-side property: the decisive `&&` consults resolved semantics in its first conjunct and the leaf spelling in its second, so what is missing is a recorded target identity for `PlainCallSemantics`, not a different table lookup — which is the fact PR B is reserved to carry. **Not #8952**, though the two share the `map_get` collision and it is the near-miss worth naming: #8952 refuses the ambiguity at *resolution*, while D is emission rebinding a call that resolution already answered correctly. Same collision, opposite sides of the resolve boundary, different repairs. |
| E | **measured** (executed counterfactual, negative) | **unowned, and reclassified** — the local refusal was implemented and measured: 24 sites in this closure depend on the fabricated `unit`, and refusing turns 1 downstream row into 12 blocking ones | solve the empty literal's element type from the callee's unsolved type variable, or defer its judgement until the expected type is known, so the fabricating arm becomes unreachable rather than refusing. The arm is left as `main` has it until then, because it is currently the only thing keeping those 24 sites compiling. |
| F | **measured** (executed counterfactual) | **REPAIRED** — gunbc#9101, with the regenerated stage0 mirror at its fixed point | none. `E0631` 2 -> 1 with eleven codes byte-identical; the row is retired, not relocated. |

**Three mechanisms are declared unowned, and that is the disposition rather than a gap in it.** B, E and F
are 7 rows between them; nothing in the repository holds them today, and writing a lane row here would
manufacture an owner that does not exist. What the declaration buys is that their absence is now countable
— an unowned row is a thing that can be picked up, where an undocumented mechanism is not.

---

## Reproduction

The recipe pins the tree and scripts the blocker-lift, because neither is optional: at the pinned
tree the parse blocker of defect 1 below is present, so a run without step 2 refuses with `EMIT_REFUSE`
and produces no board at all.

```bash
# 1. the tree the board was measured at
git checkout faf6583461a4f7d042ae670e563758869d439159

# 2. lift the one parse blocker (defect 1). #9027 repaired it by moving a trailing annotation
#    block above the file's final declaration; taking that one file from the merge commit is the
#    exact form of "with the blocker lifted", and it touches nothing else.
git checkout 1ed02057a5fac683893afcbb427fa8933cc0f2a4 -- \
  dag/test/manual/command_runner_local_argv_receipt_test.dag

# 3. measure
CSSL_STD_SEED_LINK=1 \
PROBE_KEEP_LOG_DIR=<dir> \
PROBE_EXPECT_BASE_SHA=faf6583461a4f7d042ae670e563758869d439159 \
docs/probes/curated_cargo_probe_one.sh src/v2/lens/affected_set.dag
```

Step 2 leaves the working tree differing from `HEAD`, which is the condition defect 2's stamp does not
detect — but it is harmless *here*, and for a stated reason rather than by luck: the lifted file is
`dag/` subject data the binary compiles at run time (case 1 of the discriminator below), not a build
input of the compiler, so no stale binary can misreport it. Checking out the pinned SHA in step 1 moves
`HEAD` and therefore misses the `target/release/gunbc.tree` key on its own, forcing the rebuild.

The A arm is this same recipe with the #9041 diff applied to `src/v2/lens/application.dag` before step 3.

The probe's output directory is a `mktemp -d` removed on exit; only the kept `affected_set.cargo.log`
survives. On a remote dispatch, any extraction must run inside the same dispatch.

**Three instrument defects govern whether these numbers are reproducible.**

1. **Clean `main` refuses this entry outright.** `dag/test/manual/command_runner_local_argv_receipt_test.dag`
   ends in a trailing unattached `//` block, which DESIGN §4c refuses. An `--entry` compile is *scoped in what
   it emits but whole-tree in what it parses* — the run reports `indexed 3851 modules … resolved 88 sources`,
   where 88 is the closure and 3851 is the census the file sits in. Fix: #9027. Every measurement here was
   taken with that one blocker lifted.
2. **The probe reuses a stale compiler.** `probe_binary_tree_key` keys the `<binary>.tree` stamp on
   `git rev-parse HEAD` alone, with no diff. Measured consequence: the same tree that refuses with a fresh
   compiler emitted 93 files with a stale one, because the §4c refusal is compiler behaviour rather than data.
   Fix: #9018, which keys on HEAD plus the SHA-256 of `git diff HEAD` and declares untracked new files a
   remaining gap. **The at-risk class is narrower than "measurements with uncommitted edits"** — all five must
   hold: the working tree differs from HEAD; that difference changes a real build input of the tool being
   measured; a prior binary exists under the unchanged HEAD-derived stamp; the probe reuses it; and the claimed
   effect depends on the changed behaviour. It does *not* impugn clean committed-ref measurements, forced-rebuild
   runs, subject-data-only changes under an unchanged compiler, or clean-main reproductions.

3. **A `.dag` authority change does not reach the binary at all without regeneration.** This is distinct from
   defect 2 and is *not* solved by fixing the stamp. The probe builds `cargo build --release -p v1-compiler
   --bin gunbc`, which compiles the **generated** `src/v1/stage0/src/*.rs` mirror. Editing a `src/v1/*.dag`
   authority changes nothing the compiler is built from until `claim_executor --required-regen` regenerates the
   mirror and it is installed — so **even a fresh `cargo build` compiles the old mirror**. That is an
   authority→generated-input break, not cache impurity.

   **Recorded because it invalidated two arms behind this document.** Two probe arms varied `src/v1/*.dag`
   files and reported that those edits made no difference. Each printed a positive control — a grep proving the
   edited text was present in the tree. That control was **vacuous**: it established the source text, not that
   the compiler reflected it. Both arms were guaranteed to behave identically to a pristine tree whatever the
   `.dag` said, so they could not have detected an effect had one existed. The conclusion those arms were
   offered for happens to hold, for a *stronger* reason than they gave — a `.dag` edit provably cannot affect
   that binary without regen — but the arms themselves carried no information. **A positive control must
   witness the artifact under test, not the input a producer would have consumed.** Nothing else in this
   document rests on them; every measurement here ran on a pristine `src/v1` tree.

   **The discriminator, because the rule above is not "no inline `.dag` edit is ever a valid arm".** The test is
   *which producer stands between the edit and the artifact under test*:

   | | edit target | reaches the artifact? |
   |---|---|---|
   | 1 | **subject data** the binary compiles at run time (`dag/gunbc/*`, `dag/test/claim/*`) | **yes** — inline edit is live, the arm is valid |
   | 2 | **a `.dag` compiler authority** under `src/v1/*.dag` | **no** — the binary is built from the generated mirror; vacuous without regen |
   | 3 | **hand-maintained seed Rust** under `src/v1/stage0/src` | **yes** — a plain `cargo build` picks it up |

   Verified rather than assumed: `src/v1/stage0/src/v1_compiler_parse.rs` opens with
   `// Generated by v1 compiler -- do not edit. // Source module: v1.compiler.parse` — it is the mirror of the
   `02_parse.dag` the vacuous arms edited. `src/v1/stage0/src/cli_run.rs` carries no such header and has no
   corresponding `.dag`, so it is case 3.

**Note that #9018 fixes only defect 2.** Defects 1 and 3 are untouched by it, and defect 3 cannot be fixed by a
stamp of any kind — a cache key cannot repair a build that never consumed the file that changed. The class is
not closed.

**A note on citation form, since this document was authored from a worktree 29 commits behind `main`.** All
nineteen symbols cited here were re-verified against `origin/main` rather than the authoring tree, and all
resolve. Exactly one cited file had moved (`src/v1/05_emit_rust.dag`, +105/-91), and because its symbols are
cited by name with no line number, nothing rotted. That is the §3 symbolic-citation rule paying off in the
precise scenario it was written for: a positional citation into that file would have been silently wrong, and
nothing about editing this document would have revealed it. The only positions this document carries
are `file:line:col` offsets *inside verbatim rustc output* against the emitted `src/*.rs` mirror — a
generated artifact with no symbol to name, which is the case §3 explicitly leaves to a position.

Every arm behind this document printed a positive control proving its edit applied *before* the measurement
ran. Remote controls must be constructed by reversing the edit in the working tree — never by naming a git
ref, since the remote container has no upstream ref and `git checkout origin/main -- <path>` silently no-ops.
