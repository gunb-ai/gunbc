# Affected-set emission: five compiler-side mechanisms

Pinned tree `faf6583461a4f7d042ae670e563758869d439159`.
Entry `src/v2/lens/affected_set.dag`.
Instrument `docs/probes/curated_cargo_probe_one.sh` (emit -> cssl_assemble -> `cargo build --release --lib`).
Board: **123 error lines, 122 primary sites** (the 123rd is the span-less `could not compile` summary).

This document describes defects. **It proposes no repairs**, deliberately: the aim is to make each
defect cheap to understand, not to pre-decide its fix.

## Scope and evidence status

Of the 122 primary sites, **24** land in `v2_lens_application.rs`, `std_change.rs` and
`v2_lens_affected_set.rs`. Those 24 partition with no residue into six mechanisms. Five are
compiler-side and are documented here; the sixth (`ABSENT_CLONE_BOUND`, 2 rows) is an already-established
mechanism owned elsewhere and is listed only so the arithmetic closes.

| id | mechanism | rows | evidence |
|----|-----------|------|----------|
| A | coproduct realization has no unused-parameter carrier | 10 | **measured** (executed counterfactual) |
| B | fn-typed params captured into an `Rc` closure demanding `'static` | 4 | read |
| C | `ABSENT_CLONE_BOUND` (established, not documented here) | 2 | read |
| D | authority substitution after resolution (emission rebinds to a v1_rt builtin) | 5 | **measured** (peer board, 28 blocks) |
| E | empty list literal element type resolves to unit | 2 | read |
| F | by-value closure params against a reference-yielding iterator | 1 | read |

10 + 4 + 2 + 5 + 2 + 1 = 24.

**Measured** means an executed experiment discriminates the claim. **Read** means the mechanism is
derived from the rustc text and the `.dag` source without an executed counterfactual. That distinction
is load-bearing and is not flattened anywhere below.

---

## A — coproduct realization has no unused-parameter carrier

**Rows:** 10 (`E0392` x4, `E0282` x6), all in `v2_lens_application.rs`. **Measured.**

### Source

`v2.lens.application` `LensApplicationConfig` (`src/v2/lens/application.dag:75`)

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

## B — fn-typed params captured into an `Rc` closure demanding `'static`

**Rows:** 4 (`E0310` x4), in `std_change.rs`. **Read.**

### Source

`std.change` `keyed_three_way_patch_monoid` (`dag/std/change.dag:150`)

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
import v2.std.collection { Absent, List, Map, Present, list_at_optional, map_get, map_insert }   // :5
import v2.std.algebra    { TailAbsent, TailFound, contains, length, list_append, ... }           // :6
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
8505:  let is_rt = callee_is_function_value == false && map_contains_key(rt_functions(), func)
9149:  } else if map_contains_key(rt_functions(), function_name) == false {
9789:  if map_contains_key(rt_functions(), method_name) {
```

Line 8505 is the mechanism in one expression. The **first** conjunct is keyed on resolved semantics
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

## E — empty list literal element type resolves to unit

**Rows:** 2 (`E0308` x1, `E0631` x1, same line). **Read.**

### Source

`std.change` `keyed_collect_keys` (`dag/std/change.dag:252`)

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

---

## F — by-value closure params against a reference-yielding iterator

**Rows:** 1 (`E0631`). **Read.**

### Source

`v2.lens.affected_set` `affected_set_closure` (`src/v2/lens/affected_set.dag:415`)

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

## Reproduction

```bash
CSSL_STD_SEED_LINK=1 \
PROBE_KEEP_LOG_DIR=<dir> \
PROBE_EXPECT_BASE_SHA=$(git rev-parse HEAD) \
docs/probes/curated_cargo_probe_one.sh src/v2/lens/affected_set.dag
```

The probe's output directory is a `mktemp -d` removed on exit; only the kept `affected_set.cargo.log`
survives. On a remote dispatch, any extraction must run inside the same dispatch.

**Two instrument defects govern whether these numbers are reproducible.**

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
nothing about editing this document would have revealed it.

Every arm behind this document printed a positive control proving its edit applied *before* the measurement
ran. Remote controls must be constructed by reversing the edit in the working tree — never by naming a git
ref, since the remote container has no upstream ref and `git checkout origin/main -- <path>` silently no-ops.
