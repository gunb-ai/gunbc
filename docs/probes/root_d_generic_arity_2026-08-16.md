# Root D — generic argument count (E0107): live measurement and root cause, 2026-08-16

Lane: `vivid-wren-870`. Coordination surface: `docs/plans/self-host-cargo-refusal-root-partition.md`
(`smart-ram-730`, draft PR #8333). This file is the receipt; the shared doc should cite it rather
than restate it.

## 1. What was measured, and with what

Instrument: `docs/probes/curated_cargo_probe_one.sh` (emit -> `cssl_assemble` -> cargo), per the
shared doc's working agreement 2. `frontier_probe_survey` was not used.

- worktree `/home/briansrls/.worktrees/gunbc/vivid-wren-870`, branch `session/vivid-wren-870`
- entry `src/v2/compiler/06_translate.dag`, `CSSL_STD_SEED_LINK=1`, no lane shim
- release `gunbc` + `cssl_assemble` built in this worktree

Live result for `06_translate`: emit succeeds (92 files, 218 diagnostics), cargo refuses with
**671 coded errors**, of which **E0107 = 21**. July's snapshot for the same module was 364 total /
E0107 13, so this is consistent with the shared doc's "the wall grew" note. No cause is attributed
to that growth here.

The 21 split by exact text:

```
 9  E0107: type alias takes 0 generic arguments but 1 generic argument was supplied
 8  E0107: type alias takes 0 generic arguments but 2 generic arguments were supplied
 4  E0107: missing generics for enum `Witness`
```

**Sizing beyond this one module is NOT claimed.** The July corpus figure (~76) is not refreshed by
this receipt. What is established below is the mechanism, on live evidence, at exact sites.

## 2. The emitted text, at the exact sites

```
src/v2_std_collection.rs:41   pub fn optional_present_witness<T>(opt: Option<T>) -> Rc<Witness> {
src/v2_std_collection.rs:68       pub value: Rc<Witness>,
src/v2_std_qualified_name.rs:296 pub fn qualified_name_from_node(root: Rc<Node>) -> Rc<Outcome<QualifiedName<String>>> {
src/std_integer.rs:96         pub type PositiveInt = Rc<Nat<Magnitude>>;
```

Authored sources:

- `src/v2/std/collection.dag` — `fn optional_present_witness<T>(opt: Optional<T>) -> Witness<T>`,
  and `type IndexedLookup<T> { value: Witness<T> }`. The argument is **dropped**.
- `dag/std/nat.dag` — `type Nat = CommutativeSemiring<Magnitude>` (zero parameters);
  `dag/std/integer.dag` — `type PositiveInt = Nat where gt_zero`. The emitted text keeps the
  **alias name** `Nat` and appends the **resolved definition's** argument `Magnitude`. Same shape
  for `QualifiedName = FreeMonoid<Symbol>` -> `QualifiedName<String>`.

So the two halves are not opposites of each other in cause: one **drops** a reference's own
argument, the other **fuses** an authored name with a resolved definition's arguments. Both are
decided by the same predicate, below.

## 3. Root cause, by execution

Method: the emitter's generated realization `src/v1/stage0/src/v1_compiler_emit_rust.rs` was
temporarily instrumented (probe-then-revert, working agreement 4; the tree is reverted, no
instrumentation is committed) to print the node reaching `render_rust_decl_type` and the string it
returned.

Observed, emitting `src/v2/std/collection.dag`:

```
4  ROOTD decl_type leaf=Witness raw_name=Witness nkids=1 kids=["T"] applied_prop=false
4  ROOTD result   raw=Witness nkids=1 conn=NoConnective -> Rc<Witness>
```

The node is **correct** — name `Witness`, one child `T`, `NoConnective`. The argument is dropped
inside the renderer, not upstream. Two further probes localise it: `render_rust_applied_type` and
`rust_fn_sig_peel_closed_alias` are **never called** for these nodes, which leaves exactly one arm
that can return a bare name from a children-bearing node:

`v1.compiler.05_emit_rust` `rust_render_checkpoint_scalar_bare` -> `rust_scalar_checkpoint_render_base`
-> `lookup_checkpoint(target: Rust, dag_name: leaf)`.

That is the **E0109 construction wall** documented in-tree at
`rust_checkpoint_scalar_phantom_params_note`: *"A checkpoint scalar has arity 0 in Rust; DAG phantom
params must not surface as `i64<T>`."* It strips type arguments whenever the leaf name resolves to a
checkpoint scalar. And `extdeps.languages.rust.types` `rust_type_checkpoints` contains:

```
{ dag_name: "Witness", target_type: "Witness", ... }
{ dag_name: "witness", target_type: "Witness", ... }
```

`Witness` is not a scalar. It is the generic enum `Witness<C>` declared in `dag/std/witness.dag`,
and `std.types` `container_type_arity` independently records `"Witness": 1`. So **two authorities
disagree about one type's arity** (DESIGN section 3), the emitter believes the checkpoint table, and
every `Witness<T>` in type position loses its argument. That is the whole of half 2 (`missing
generics for enum Witness`).

Half 1 is the same function taking its *other* exit. For `Nat` the node reaching the renderer is
`raw_name=CommutativeSemiring nkids=1 kids=["Magnitude"]` while `rust_fn_sig_leaf_name` returns the
authored `Nat` — the renderer composes the authored **name** with the resolved node's **children**.
`Nat` is absent from `rust_type_checkpoints`, so whether the arity-0 strip fires is decided by the
fallback `rust_seed_host_numeric_alias(name, corpus_repr)`, which answers `i64` for `Nat`/`Int`
**only when `corpus_repr_is_host`**. Otherwise no checkpoint matches, the strip does not fire, and
the fused `Nat<Magnitude>` is emitted.

## 4. This is a third instance of the closure-shape meta-root

Asked by `smart-ram-730` (msg_b0361cc2): does Root D's mechanism contain a closure-shape branch?
**Yes, on half 1, and it is the same `RustCorpusRepr` flag `eager-deer-389` root-caused for Root B.**

- seed closure (`HostNative`): `Nat`/`Int` render as `i64`, arity 0, arguments dropped, compiles.
- pure-v2 closure (`FaithfulFreeMonoid`): no checkpoint matches, and the same node renders as
  authored-alias-name + resolved-definition-args, which cannot compile.

So half 1 is invisible in any closure containing `src/v1` paths and appears only in seed-free
closures. Half 2 (`Witness`) is **not** closure-shape dependent — it is a hard table row and would
fire in a seed closure too.

Caveat kept deliberately: I attempted a direct control by adding `--source-root src/v1` to the
minimal `dag/std/integer.dag` emit and the output did not flip (`Nat<Magnitude>` both times). That
does **not** refute the branch — `corpus_has_v1_seed_source_indices` reads the loaded modules'
source indices, and adding a source root does not pull a v1 module into that closure. The
closure-shape claim above rests on reading the branch, plus the observed non-firing of the strip in
this closure; a discriminating control that genuinely varies the flag has **not** been executed.

## 5. Minimal reproducer (per `eager-deer-389`'s method)

No cargo needed — the defect is visible in emitted text:

```
gunbc compile --source-root dag --source-root src/v2 \
  --entry dag/std/integer.dag --output-dir <out> --target rust \
  --dependency-pool-index primary-precedence
grep -n 'PositiveInt' <out>/src/std_integer.rs     # -> pub type PositiveInt = Nat<Magnitude>;
```

and for half 2, the same command with `--entry src/v2/std/collection.dag`:

```
grep -n 'Witness' <out>/src/v2_std_collection.rs   # -> pub fn ...(...) -> Rc<Witness>
```

Each is ~2m40s against ~8 minutes for a compiler-module cargo probe.

## 6. What follows, and what does not

- The fix for half 2 is **construction, not validation**: `Witness` should not be a checkpoint
  scalar row. Its arity has a single authority already (`std.types` `container_type_arity`), and the
  E0109 wall should derive arity from it rather than from membership of a target spelling table
  that cannot express arity at all. Note the shape of the defect: the strip is a **widening failure
  arm** (DESIGN section 5) — it answers "drop the arguments" for two different questions, *this type
  has no arguments in Rust* and *I have no row for this type*.
- Half 1's fix is not the same edit and is not proposed here: the renderer's composition of an
  authored leaf name with a resolved node's children is a distinct defect, and it is entangled with
  the corpus-repr flag that Root B owns. Fixing the `Witness` row will not move it.
- Nothing here sizes the corpus. A live E0107 census across modules is still owed.
