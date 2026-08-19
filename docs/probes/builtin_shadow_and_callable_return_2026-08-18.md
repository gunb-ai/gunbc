# materialization_carriers: two emitter roots closed, measured before and after (2026-08-18)

**Session:** `witty-heron-413` (child of `smart-ram-730`, self-host root partition).
**Subject:** the `src/v2/compiler/materialization_carriers.dag` emitted crate, driven toward
`cargo` green.

Every number here is from a run made for this document. Nothing is transcribed from an
earlier census.

## 1. Instrument

```
gunbc compile --source-root dag --source-root src/v2 \
  --entry src/v2/compiler/materialization_carriers.dag --target rust \
  --dependency-pool-index primary-precedence
cssl_assemble --out-dir <out> --entry-dag <entry> --root .
cargo build --release --lib --message-format=json
```

`CSSL_STD_SEED_LINK=1`, no lane shim (the raw cssl-assembled `lib.rs`). Counts are **errors as
rustc reports them in the JSON stream**, primary spans only — not distinct sites of a July
cause-signature census, and not the rendered text log.

Binaries: `gunbc` / `cssl_assemble` built locally from this branch at each measurement point;
`regen_stage0` run after every `.dag` emitter edit so the seed and its authority agree.

## 2. The count a session driving this module actually faces

**60**, not the 386 distinct sites `docs/plans/self-host-cargo-refusal-root-partition.md` §11.2
records for this entry. Those are different denominators: §11.2 counts deduplicated
`(file, line, column, code, cause signature)` rows over the July closure; this counts errors
rustc emits today, after which it stops type-checking items it can no longer reach. 60 is the
reachable-today number; it is not a refutation of 386.

Baseline histogram (main at `11254b04fc`):

```
E0308 28 · E0277 16 · E0425 3 · E0422 2 · E0369 2 · E0282 2 · E0061 2 · E0599 2 · E0560 1 · unreachable_pattern 2
```

Nine mechanisms, every error in exactly one. Ownership as confirmed with `smart-ram-730`:
Clone bounds (A) and Optional (C) open, ContentHash/String (T7) taken by `calm-lynx-547`.

## 3. Root 1 — a builtin answered a call the program had already bound

`v2.std.staging` declares `fn cached_stage(lookup: fn(A) -> CacheProbe<B>, stage: ...)`. The
emitted Rust called `v1_rt::lookup(&x)` — the map builtin — not the parameter. The name question
was answered by the authored spelling at four independent tiers: `04_infer`'s method-template
resolution, its `empty_map`/`empty_set` arms, its `infer_builtin_call_type` arm, and the Rust
emitter's `rt_functions()` table plus its `get`/`with`/`to_string`/`discriminant` special forms.

**This one only errored because the arities disagreed** (`v1_rt::lookup` takes 2 arguments, the
program supplied 1 → E0061). Where a builtin accepts what the program supplies, both realizations
answer from the builtin and the program's own function is never called, with no diagnostic
anywhere. That is not hypothetical — it is the second fixture below, measured:

| specimen | before | after |
|---|---|---|
| `fixtures/builtin_shadow/free_call_shadow_specimen.dag` (param named `lookup`) | emitted `v1_rt::lookup(&x)` → E0061; interpreted `3` (builtin returned None, fell through) | emitted `lookup(lookup(x))`; interpreted `3` |
| `fixtures/builtin_shadow/method_template_shadow_specimen.dag` (param named `to_string`) | emitted `(x).to_string()`; **interpreted `7`** | emitted `to_string(x)`; **interpreted `SHADOWED`** |

The second row is the class at its worst: a program whose declared function returns `"SHADOWED"`
returned `7`, in both realizations, silently. Reproduce with
`gunbc run --source-root fixtures --entry fixtures/builtin_shadow/method_template_shadow_specimen.dag --function shadowed_method_template_result`.

**The fix decides the question once, where scope is known.** The corpus already had the
authority — `body_locals`, the shadowing set `call_locals_shadow_note` uses to skip fn-sig
lookup. It now also gates the builtin tiers, and `04_infer` records the answer on the node as a
new `CallSemantics` variant, `FunctionValueCallSemantics`. The Rust emitter consumes that
variant instead of re-deciding by name: a function-value application renders as ordinary
positional application of the bound identifier. `ctx.lookup_fn` (module declarations) stays below
the builtins exactly as before; only the lexical tier moved, in both the compile path and
`v1_interpreter.rs eval_call`, so the two realizations agree by construction rather than by
coincidence.

## 4. Root 2 — a closure where an arrow type was declared

A `.dag` fn whose declared return type is an arrow returns a callable value; Rust renders that
type as `Rc<dyn Fn(..) -> ..>` through `rust_callable_type_template`. The body's closure was
emitted bare, which is an unsized-coercion type error, not a value of the declared type (E0308 at
`v2_std_staging.rs:16/44` and `v2_compiler_materialization_carriers.rs:183`).

The realization is now a target row beside the type template —
`rust_callable_value_wrap_template = "Rc::new(move {0})"` — applied when the declared return
connective is `Arrow` **and** the returned expression is a lambda. Both conditions are structural;
nothing scans the rendered string, and a target without the row is unaffected by construction.

## 5. Result, three numbers as requested

Measured twice, on two bases, because this branch merged `origin/main` mid-work and main's own
#8410 (Rust realization keyed on declaration identity) moved the baseline:

| base | before | after this branch |
|---|---:|---:|
| `11254b04fc` (main when this session opened) | 60 | 58 |
| `2c65eeacf3` (`origin/main` at merge, includes #8410) | 53 | **51** |

The branch's own delta is the same on both bases — 8 retired, 6 newly exposed, net −2 — which is
the check that it is measuring the fix rather than the base.

```
baseline                53   (origin/main 2c65eeacf3)
after both roots        51
  retired (gross)        8   E0061 x2, E0308 x5, E0282 x1
  newly exposed          6   E0599 "no method `clone` on type parameter A"
  net                   -2
```

The six new rows are **honest errors the two defects were hiding**: with the call now bound to
the parameter and the closure now coerced, the missing `Clone` bound on the generic parameters is
what rustc reaches next. They belong to mechanism 1 (Clone bounds), which is unowned and is the
largest remaining cluster in this module — 28 of the surviving 51, split in §10.

Surviving histogram (on the merged base): `E0277 16 · E0308 15 · E0599 9 · E0425 3 · E0422 2 · E0369 2 · unreachable_pattern 2 · E0560 1 · E0282 1`.

## 6. Two findings this work surfaced and did NOT fix

**(a) `regen_stage0` on main does not produce a tree that builds.** Running it on a clean
checkout of `11254b04fc` (no source change) rewrites two files in a way that breaks compilation:
it prunes `v1_rt::obs_human_elapsed`, which hand-maintained `cli_run.rs` calls (E0425), and drops
the `Hash` derive from `ItemKind`, which hand-maintained `coproduct_reflection.rs` requires
(E0599 x2). The self-host fixed point is therefore stale on main, and the CI job that used to
catch it was removed in the floor cut. This branch restores those two files to their committed
content after each regen so the tree builds; that is a declared deviation from raw regen output,
not a claim that the fixed point holds.

**(b) Mechanism 4 (`OccurrenceId` emitted twice) has a 12-line repro and a wrong-looking obvious
fix.** `type X = other.module.X` emits a fresh `pub struct X(pub PhantomData<()>)` rather than an
alias, because `is_self_referential_opaque_type_resolved` compares the RHS's bare leaf name to the
alias's own name — the same spelling-vs-identity class as §3. Adding the declaration-identity
discriminator (`type_reference_decl_file(rhs) == decl_identity_file(item)`) does remove the
phantom, but the alias then renders `pub type X = X;` (E0391 cycle) because the leaf branch of
`render_rust_alias_rhs_type` never qualifies, and the obvious qualification via
`alias_rhs_rust_qualify_module_filename` answers from the **bare-name registry**, which maps `X`
to the alias itself and additionally mis-qualifies host-grounded names
(`crate::std_types::String`). Both halves were implemented, measured, and **reverted** rather than
landed half-right: the qualification must come from the resolved declaring module, not from a
name-keyed registry. Left as a named root with an executed repro, not as a silent gap.

## 7. The board: all 51 surviving errors, partitioned at mechanism grain

Every one of the 51 is in exactly one row; the counts sum to 51 by construction, and the
partition was assigned from the rustc JSON (code + primary span + label), never from a keyword
over rendered text. Measured on this branch's head against `origin/main` `2c65eeacf3`.

`DFS position` names the layer that decides the fact, not the file the error appears in — the
whole point of the partition is that most of these errors are downstream of a decision made
somewhere else.

| # | mechanism | pop | specimen (emitted site) | DFS position | authority | consumer | touch set | depends on | target shape |
|---|---|---:|---|---|---|---|---|---|---|
| 1 | **Clone bound not emitted on generic params** — split into 1a/1b in §10 | **28** | `v2_std_staging.rs:44` `no method clone on type parameter A`; `v2_std_algebra.rs:45` derive | emit, two distinct seams | `v1_item_clone_bounded_param_names` (fn sigs) + `trait_derive_emit` (derives) | rustc trait solver | `05_emit_rust.dag`, `trait_derive_emit.dag` | none — it is the root | modeled `TraitBoundWitness` consumed by `emit_fn_def`; **never** a scan for `.clone()`/`.iter()`. Keep E0599 (generic fn bodies) and E0277 (per-derive bounds at struct decls) separate: same trait word, different consumers, different repairs |
| 2 | **Optional carrier fork** | 6 | `std_cache_interface.rs:638` `expected Option<String>, found None` (a `None` *variant*); `:695` list head returning `Option` where the value is expected; `extdeps_uri.rs:752` `Absent` as a fold init against an `Option` accumulator | infer + emit | `std.optional` / the `Value::Null` split | emitted Rust; the interpreter's `Value::Null` overload | `04_infer.dag`, `05_emit_rust.dag` | the `Value::Null` split plan | one Optional carrier with one emitted spelling; `Present`/`Absent` and `Option` stop being two surfaces |
| 3 | **Unsynthesized use-line (root K)** | 5 | `extdeps_realization_compile_stage_memo.rs:82` `ProviderRetention` not found; `v2_compiler_materialization_carriers.rs:141` `NonEmptyStr` not found | emit, import synthesis | `reference_derived_use_lines` (`05_emit_rust.dag`) | rustc name resolution | `05_emit_rust.dag` | namespace-resolution lane | the note's own §5: a candidate that registry-resolves but fails export proof must **refuse**, typed and counted, instead of silently emitting nothing |
| 4 | **Int literal accepted into a branded-string field** | 4 | `extdeps_realization_compile_stage_memo.rs:95` `observed_at: 0`, `expected String, found integer` | **corpus authoring, and a hole in the compile seam** | `dag/std/types.dag` `LogicalTime = NonEmptyStr where brand("LogicalTime")` | record-literal field checking | `dag/extdeps/realization/{compile_stage,parse_table}_memo.dag` (4 authored rows) + the field-literal conformance check | none | **rustc caught what the typechecker did not.** The rows are simply wrong (`0` where a branded non-empty string is declared) and repairing them is a two-line edit — but the seam that let an `Int` literal into a `where`-refined `String` field is the real subject, and it is a §5 fail-open, not a formatting slip |
| 5 | **Type alias materialised as a second type** | 3 | `v2_std_node.rs:69` `expected std_occurrence_identity::OccurrenceId, found v2_std_node::OccurrenceId` | emit, type-decl branch | `is_self_referential_opaque_type_resolved` + `render_rust_alias_rhs_type` (`05_emit_rust.dag`) | every boundary that passes the aliased type | `05_emit_rust.dag` | resolved-declaring-module lookup that is **not** the bare-name registry | §6(b) above: identity discriminator **and** qualification from the resolved declaring module, landed together. Either alone is a different error with the same root |
| 6 | **Nested coproduct pattern flattened to the outer discriminant** | 2 | `v2_std_node.rs:533/537` `unreachable pattern` | emit, match lowering | `classify_arrow_body_form` in `src/v2/std/node.dag` matches `TypeNode { connective: Atom {..} }` then `TypeNode { connective: Conj }` | emitted `match` | `05_emit_rust.dag` match lowering | none | **the count understates this one.** Both arms lower to the same outer arm, with the inner test as a refutable `let ... else { unreachable!() }`, so arm 2 is dead *and* a `Conj` node reaches `unreachable!()` at runtime — the model selects arm 2, the emitted program panics. rustc reports it only as a lint; the defect is a realization divergence |
| 7 | **ContentHash carrier vs `String` (T7)** | 1 | `v2_std_node.rs:1334` `partial_cmp` on `Rc<Fnv1a64Structural>` | model + emit | `std.content_hash` | sort key in the canonical-hash fold | owned by `calm-lynx-547` — **do not touch** | their lane | (theirs). Note main's #8410 already retired 6 of the 7 rows this mechanism had here |
| 8 | **Record literal through a shared-wrapped alias** | 1 | `std_verification.rs:22` `Rc<Measure<(),(),i64>> has no field count` | emit, record-literal head | `dag/std/verification.dag` `NanosecondDuration = Measure<Time, Nano, Nat>` | emitted constructor | `05_emit_rust.dag` | none | construct then wrap (`Rc::new(Measure { .. })`); the alias head must not be used as the literal head when the alias renders shared-wrapped. (The `Time, Nano` → `()` phantom collapse at the same site is mechanism 1's neighbour, not this row) |
| 9 | **Type annotations needed** | 1 | `std_realization_measurement.rs:197` cannot infer `S` on `cost_account_predicted_zero` | emit, turbofish elision | — | rustc inference | `05_emit_rust.dag` | none | emit the type argument the model already knows instead of relying on inference at a phantom-parameter call |

**Allocation reading.** Mechanism 1 is 55% of what is left and is the only row whose repair is a
design rather than a fix; 2/3/5 are known roots with named owners or lanes; 4/6/8 are new here and
each is small, self-contained, and in `05_emit_rust.dag` or four authored rows — but **4 and 6 are
worth more than their counts**, because each is a fail-open that rustc happened to notice: one is a
type error the compile seam admitted, the other is a match the emitted program gets wrong at
runtime.

## 8. Mechanism 4 decided by execution: the floor holds — the hole is at the SECOND refinement hop

`smart-ram-730` asked for one discriminating compile, on the reading that an `Int` literal into a
`String`-typed field might be a base-type failure (below-baseline floor) rather than a refinement
failure. Run with a positive control in the same arm, because a bare refusal signal is also what a
missing emitted file looks like.

| # | subject | verdict |
|---|---|---|
| A | `type Box { s: String }` · `Box { s: 0 }` | **REFUSED** — `type mismatch: expected 'Primitive(String)', got 'Primitive(Int)'` |
| B | positive control: `Box { s: "ok" }` | **ACCEPTED**, 5 files emitted, 0 diagnostics |
| C | `type Row2 { s: NonEmptyStr }` · `Row2 { s: 0 }` (`NonEmptyStr = String where non_empty`, depth 1) | **REFUSED** — `expected 'Product(NonEmptyStr)', got 'Primitive(Int)'` |
| D | `type T1 = String where brand("T1")` · `RowA { v: 0 }` (depth 1, brand) | **REFUSED** — `expected 'Product(T1)', got 'Primitive(Int)'` |
| E | `type Row { at: LogicalTime }` · `Row { at: 0 }` (`LogicalTime = NonEmptyStr where brand(...)`, depth 2) | **ACCEPTED**, 8 files emitted |
| F | local depth-2, no std: `type S1 = String where non_empty` · `type S2 = S1 where brand("S2")` · `RowB { v: 0 }` | **ACCEPTED** (no diagnostic for the row) |

**So the ordinary compiler floor is intact, and the below-baseline reading is refuted** (A refuses,
B is green). One refinement hop is intact too (C, D). The hole opens at the **second** hop — a
refined alias whose base is itself a refined alias — and it is neither brand-specific nor
std-specific: F reproduces it in eight lines with no import.

**The tell is the diagnostic the accepting arm actually emits**, which self-describes the failure:

```
advisory[...]: where-refinement unenforced: predicate 'non_empty' on 'Product(LogicalTime)'
               — non-literal value at refined position
```

The literal `0` at a string-refined position is classified as a **non-literal value**, so the arm
degrades to an advisory instead of refusing. That is DESIGN §5's absorbing fallback exactly: the
arm that cannot decide widens to "unenforced" rather than stopping the line, and the deficit's
frequency is zero by construction because nothing counts it. At depth 1 the base-type comparison
answers first and refuses; at depth 2 the expected type is reported as `Product(LogicalTime)` and
the base type is no longer the thing being compared, leaving the refinement check as the only
judge — and it absorbs.

`where_refinement_enforcement_witness_note` claims this population by name — "record fields", and
"inherited predicates compose through refined-alias chains (review 45121 — NonEmptyStr where
brand)". **The composition is precisely where it stops holding**, so the note is a §4b rung-honesty
failure rather than an unbuilt wall: an inflated class never ranks for climbing.

How far the fail-open runs, since it is not confined to compile: the emitted artifact for E is

```rust
serde_json::from_value(serde_json::json!({"at": 0})).expect("valid data definition")
```

— an `Int` in a `String` field, behind an `.expect` that asserts validity. The compile seam admits
it, the emitter serializes it, and the claim of validity is made at the point it would panic.

**Disposition:** the four authored `observed_at: 0` rows are a two-line repair, but they are the
symptom. The subject is the depth-2 arm, and the repair is that a value whose base type does not
match the refined type's base must **refuse** at the literal site rather than report the predicate
unenforced. Left routed, not fixed here — this section is the receipt that decides which lane owns it.

## 9. Root 2's residue, measured rather than argued (review 53557)

The review observed that `rust_callable_return_wrap` fires only when the returned expression is
itself a lambda, so a declared arrow return whose lambda sits in a tail position is untouched.
Confirmed by compiling all three forms:

```rust
pub fn pick(flag: bool) -> Rc<dyn Fn(i64) -> i64> {
    if flag.clone() { |n| (n.clone() + 1) } else { |n| (n.clone() + 2) }   // bare, unwrapped
}
pub fn pick_match(flag: bool) -> Rc<dyn Fn(i64) -> i64> {
    match flag.clone() { true => |n| ..., false => |n| ... }                // bare, unwrapped
}
pub fn pick_let(flag: bool) -> Rc<dyn Fn(i64) -> i64> {
    { let f = |n| ...; f }                                                 // bare, unwrapped
}
```

Emitted with **0 diagnostics** in all three cases. So the class is **narrowed, not closed**, and
this PR's §4 wording ("both conditions are structural") describes what the wrap decides, not a
guarantee that every arrow return is covered.

**Why the obvious widening is wrong:** the wrap applies to a rendered string, and
`Rc::new(move if c { .. } else { .. })` is not Rust. Closing the class is a tail-position walk that
wraps each tail *leaf*, which is emission-shape work.

**Trigger, and the honest arm in the meantime:** a bare closure at a declared-arrow return can
never compile, so the residue's correct treatment is a **located refusal** rather than silent
emission of invalid Rust — refusing it cannot break anything that currently works. Neither the walk
nor the refusal lands here: both are emitter-shape changes, and a corpus census belongs with them.
Recorded in `05_emit_rust.dag` beside the function so the next reader meets the residue where the
decision is made, not only in this receipt.

## 10. Settling row 1: it is two mechanisms, and neither is the checkpoint-scalar dispatch

`smart-ram-730` raised three signals against board row 1 at once: `silent-raven-853` traced the
E0277 population to a realization-dispatch mechanism (a non-generic container's field missing the
checkpoint-scalar path), `deep-swift-570`'s actual retirement is 2 sites rather than a cluster, and
my own PR summary line did not add up. All three are answered from the same rustc JSON, at
`(code, primary span, label, children)` grain.

### 10.1 The arithmetic was wrong in the summary, not in the table

The board table said 28 and enumerated it. The PR summary sentence said "16 E0277 plus 9 E0599",
which is wrong twice: it folded in the T7 `partial_cmp` E0599 (board row 7, not row 1) and dropped
E0369 ×2 and E0308 ×2. Corrected above. The row was always 28 = 16 + 6 + 2 + 2 + 2.

### 10.2 The E0277 population is NOT the checkpoint-scalar mechanism — measured, not argued

Every one of the 16 E0277 rows names a **type parameter** (`T`, `A`, `B`, `C`), and each one's
`children` say which obligation raised it:

```
required for `im::Vector<T>` to implement Debug / Serialize / Deserialize      (10 rows)
required by a bound in `bind_outcome` / `resolve_probe`                        ( 6 rows)
```

Two independent negative checks on the dispatch reading, in this closure:

- **Zero** of the 51 errors mention `Nat`, `Magnitude`, or `CommutativeSemiring` anywhere in the
  diagnostic JSON (message, labels, or children).
- `Nat` in this closure resolves to `dag/std/nat.dag` and is emitted as `pub type Nat = i64;` —
  the checkpoint-scalar path already answers correctly here. Seven `Nat`-typed struct fields exist
  in the emitted crate and none of them errors, because the alias itself is `i64`.

So the mechanism `silent-raven-853` describes accounts for **0 of 16** in this denominator. That is
not a contradiction of their trace — a gate that only fires for generic containers is a real defect
wherever a v2-side declaration wins the name — it is a statement that *this* module's closure does
not exercise it. Which yields a **falsifiable prediction**: when their fix lands, this module's
count should move by **zero**. If it moves, the movement is composition with something else and
belongs in the next delta receipt, not in this row.

### 10.3 The real split: by where the missing bound belongs, not by error code

Both error codes appear on both sides, which is exactly why a code-keyed split reads as confusing.
The mechanism-keyed split is clean and sums:

| row | mechanism | pop | evidence | repair site |
|---|---|---:|---|---|
| **1a** | bound missing on a generic type's **derived impls** | **12** | E0277 ×10 (`im::Vector<T>` needs `T: Clone` for Debug/Serialize/Deserialize on `FreeMonoidUniqueState<T>`, `ListTailResult<T>`) · E0369 ×2 (derived `PartialEq` comparing `Rc<Vector<T>>`) | the `#[derive(...)]` emission for a generic declaration — `trait_derive_emit.dag` |
| **1b** | bound missing on a generated **fn / inherent-impl signature** whose body clones a type param | **16** | E0277 ×6 (`required by a bound in bind_outcome` / `resolve_probe`) · E0599 ×6 (`no method clone found for type parameter A`) · E0599 ×2 (`CacheLookupResult<T>` — see the correction below) · E0308 ×2 (`expected type parameter T, found &T` — "`T` does not implement `Clone`, so `&T` was cloned instead") | `v1_item_clone_bounded_param_names` / `emit_fn_def` in `05_emit_rust.dag` |

12 + 16 = 28. **Corrected 2026-08-19 by reading the specimens** `silent-raven-853` asked for: the
two `CacheLookupResult<T>` E0599 rows were in 1a and are fn-side. Their span text is a call site —
`match (*lookup.clone()).clone()` — and rustc's children say *consider restricting the type
parameter*, because `#[derive(Clone)]` on the enum already generates `impl<T: Clone> Clone`, which
is correct; what is missing is `T: Clone` on the enclosing generic fn (`realize_route<T>`,
`dag/std/cache_interface.dag:325` and its sibling at `:334`). Full site list with raw diagnostics
for that head: `docs/probes/row_1a_sites_f0e7c9df89.md` + `.json`.

This preserves the separation `smart-ram-730` asked for — different consumers,
different repairs — but draws it where the repair actually differs. A `TraitBoundWitness` consumed
by `emit_fn_def` addresses **1b**; **1a** is the derive emitter, and the two can land independently.

`deep-swift-570`'s 2 sites being small is therefore consistent with the row being 28: the row is
not one lane's backlog, it is the module's whole Clone-bound population across two emitters.

## 11. Pre-registered predictions, and what T7 actually costs this module

Written **before** the repairs they concern land, so the tests are pre-registered rather than
reconstructed after the fact. Denominator for all of them: the 51 errors measured on branch head
`27077d456d` against `origin/main` `7a59ef751c`, by the instrument in §1.

### 11.1 Prediction — `silent-raven-853`'s container-gate fix moves this module by ZERO

Their fix makes the checkpoint-scalar identity path reachable from a non-generic container's
fields. Per §10.2 that mechanism has **0 of 51** rows here, and `Nat` in this closure is the
`dag/std/nat.dag` declaration emitted as the `i64` alias. So:

- **predicted:** 51 → 51, and no row changes shape.
- **falsifier:** any movement at all. If the count drops, the mechanism was present and §10.2 is
  wrong. If a **new refusal family** appears, that is `smart-ram-730`'s v2-side hazard biting after
  all — a name resolving to the bodyless `src/v2/std/nat.dag` declaration, which the path
  deliberately refuses, so the error changes shape rather than disappearing.
- Either outcome is reported as a delta receipt, not folded into row 1.

### 11.2 T7's live footprint here is ONE row, and `99 E0308 sites` is not this module

`stern-fox-619`'s task is titled "99 E0308 sites". Measured across three trees:

| tree | total errors | E0308 | T7 rows (ContentHash ↔ String) |
|---|---:|---:|---|
| `11254b04fc` (session open) | 60 | 28 | **8** — all E0308, all `v2_std_node.rs` (`expected String, found Rc<Fnv1a64Structural>` and its inverse) |
| `2c65eeacf3` (`origin/main`, includes #8410) | 53 | 20 | **1** — E0599 `partial_cmp` on `Rc<Fnv1a64Structural>` |
| branch head | 51 | 15 | **1** — same row |

So **#8410 already retired all 8 of this module's T7 E0308 sites**, and the residue here is a
single E0599. Of the 15 E0308 rows that remain, **none is T7**: they are 4 Optional-carrier, 4
branded-field literals, 3 alias-materialised-twice, 2 `&T`-cloned (row 1b), 2 Optional-in-`uri`.

Therefore 99 is a different denominator — whole-corpus, or the July cause-signature census — and it
should not be read as this module's backlog. Scoping a repair to retire "99 sites" against this
module would find one.

**One honesty note about how that table was produced.** The T7 column came from a keyword scan for
`Fnv1a64`/`ContentHash` over the diagnostic JSON, and it produced a **false positive** in every
row: `std_cache_interface.rs:580` matched on a substring while actually being the
`CacheLookupResult<T>` derived-`Clone` row (1a). It is excluded from the counts above by reading the
specimen. That is the same failure mode this receipt warns about in §1 — a keyword over text is a
hypothesis, and the specimen is the measurement.

