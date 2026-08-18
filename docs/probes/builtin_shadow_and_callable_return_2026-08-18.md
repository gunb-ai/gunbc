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
largest remaining cluster in this module — 16 E0277 plus 9 E0599 of the surviving 51.

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
