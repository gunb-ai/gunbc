# E0599 `as_ref` cluster — one missing field position on the shared-layer decision (2026-08-21)

Session `sunny-pike-280`, the E0599 `as_ref` lane of the self-host cargo-refusal root partition
([`../plans/self-host-cargo-refusal-root-partition.md`](../plans/self-host-cargo-refusal-root-partition.md)).

**Every number below was measured in this session, on this machine, with the instrument named.**
Nothing is transcribed from an earlier census.

## Method

| field | value |
|---|---|
| date | 2026-08-21 |
| baseline tree | `d72ffe8708b07a6bd59f8d9425fe1c052b625a17` |
| after tree | the same commit **plus the working-tree diff of this PR** — the probe's `HEAD_SHA` column reads `d72ffe8708b` for BOTH rows, so the two rows are NOT distinguishable by that field; the compiler binary is what differs, and it was rebuilt between them |
| instrument | `docs/probes/curated_cargo_probe_one.sh` |
| contract | `CSSL_STD_SEED_LINK=1`, no lane shim, `PROBE_KEEP_LOG_DIR` |
| entry | `src/v2/compiler/03_ingest.dag` |
| route | `gunbc compile` → `cssl_assemble` → `cargo build --release --lib` |
| unit of count | one `error[E0599]` block in the retained `cargo.log` (occurrences, not distinct sites) |

## The finding

18 of the board's 41 E0599 are one message, one method, one file:

```
18 error[E0599]: the method `as_ref` exists for reference `&v2_std_nat::Nat`,
                 but its trait bounds were not satisfied
```

All 18 are in `src/v2_lens_cost.rs`, at **9 distinct source lines** — each line contributes two
occurrences because the nested-pattern lowering emits the receiver twice, once in the arm guard
and once in the arm prelude:

```rust
SymbolicCost::ConstantCost { ref value, .. } if matches!(value.as_ref(), Nat::Zero)
  => { let Nat::Zero = value.as_ref() else { unreachable!() }; b.clone() },
```

`&Nat` has no `AsRef` impl, so the call resolves against the blanket
`impl<T: AsRef<U>> AsRef<U> for &T` and reports "exists, but its trait bounds were not satisfied"
rather than "no method named".

## The root — the lowering is right and the declaration is wrong

`SymbolicCost::ConstantCost` is authored in `v2.lens.cost` as `ConstantCost { value: v2.std.nat.Nat }`
— a **namespace-qualified** spelling. Every position in the emitted crate treats that field as
`Rc<Nat>`: `constant_cost` takes `Rc<Nat>`, constructs the variant with an `Rc`, and the
destructuring at `symbolic_cost_dominates` hands the field straight to `nat_dominates(Rc<Nat>)`.
Only the variant's own **declaration** rendered it bare:

```rust
pub enum SymbolicCost {
    ConstantCost { value: Nat },        // before
    LinearCost   { variable: Rc<SizeVariable> },
    ...
```

`rust_carrier_is_at_shared_layer` already unified four positions that used to answer
"is this carrier at the shared reference layer?" from a rendering or from the authored spelling
(see its note in `v1.compiler.emit_rust`). The **enum variant record field** position was not among
them: it kept a `needs_box_wrapping`-only rule, so it fell through to the name-keyed render path,
where a qualified authored name misses a `shared_types` set keyed on bare leaf names — and the field
rendered bare. A same-module (unqualified) reference hits that set and renders `Rc<..>`, which is why
`variable: Rc<SizeVariable>` is correct in the same declaration.

## The change

One authority, `rust_field_carrier_final_type`, asked by the struct-field position (which already had
the rule) and by both variant-record-field branches (which did not). Positional (tuple) payload
fields render through `render_variant_payload_type` and are deliberately **out of scope** — a
distinct path whose layer decision has to be established on its own evidence.

## Measured effect — same instrument, same entry, before and after

| | before | after | Δ |
|---|---:|---:|---:|
| cargo `due to N previous errors` | 498 | 477 | **−21** |
| coded-error-line histogram sum | 499 | 478 | −21 |
| **E0599** | **41** | **23** | **−18** |
| E0308 | 205 | 202 | −3 (**net** — see the MOVED subset below) |
| every other class (E0277, E0004, E0597, E0425, E0609, E0560, E0061, E0063, E0631, E0369, E0282, E0433, E0614, E0071, E0728, E0310, E0573, E0533, E0223) | — | unchanged | 0 |
| emitted files | 177 | 177 | 0 |

**No class increased in total.** One class moved at site grain, and it is reported as MOVED rather
than folded into the net.

**The FULL sorted histogram was diffed, not the targeted class** — including the uncoded column,
where a repair that shadows a match arm shows up as `unreachable_pattern` and a totals reading would
miss it. Both runs carry the identical uncoded pair (`uncoded_unsupported_mock_expression:13`,
`uncoded_UNRESOLVED_CompilerError:1`), `unreachable pattern` is **0** in both logs, and the warning
count is 4 in both. The whole diff of the 21-class histogram is exactly the two rows above. The `as_ref` message is absent from the after log; the remaining 23 E0599 are
seven other messages (`apply` on `Rc<LexMatchThunk>` ×7, `Present` on native `Option` ×5,
`clone` on type parameter `A` ×4, `GlobalBare*` ×4, `Empty` on `im::Vector<A>` ×2, `lookup` ×1) —
each a different root, none touched here.

Errors anchored in `v2_lens_cost.rs` went **25 → 3**. The three survivors: one pre-existing E0433
(`ConversionOpcode` not imported) and two E0308 that are a *doubly*-nested pattern
(`Succ { prev: Zero }`) whose inner `Rc` field is not dereferenced — a separate lowering defect,
present at the same two lines before this change. The other four E0308 in that file (at the
constructor, the accessor and one call site) were the *mirror* direction of the same straddle —
"expected `Nat`, found `Rc<Nat>`" — and they resolved because the declaration stopped disagreeing
with them.

### The MOVED subset — 4 new E0308, named rather than netted away

`v2_lens_complexity_accumulator_copy_analyze.rs` goes **5 → 8** E0308. Four sites are NEW
(record literals at lines 275/284/298/309, each `expected Rc<LetBinding>, found LetBinding`), and
five older ones resolved across that file and its sibling `..._copy.rs`. The cause is the symmetric
half of this same defect and it is **not** repaired here: the **value** position — the record
literal that constructs the variant — has no shared-layer arm either (`wrap_rust_record_field_value`
knows about `Fn`-Rc, `BoundedLattice` and `Box`, and nothing else). Before this change the bare
declaration and the bare literal agreed with each other and disagreed with every consumer; now the
declaration agrees with the consumers and the literal is the one position left behind. That is a
strictly better state — the layer authority decides it, not the spelling — but it is a MOVE for
those four sites, not a resolution, and the two `.rs` files above are the exact place to look for
it. It is the next cut, and it is the one the emitter's own note already names as out of scope of
the earlier four-position repair ("two value positions rendered the type to TEXT and prefix-matched
`Rc<`").

**Emitted-closure blast radius**, measured independently by diffing the whole emitted crate for
`src/v2/lens/cost/expr.dag` before and after: **8 changed lines, 4 field declarations**, all
`Nat` → `Rc<Nat>` (`SymbolicCost::ConstantCost.value`, plus `upper`, `count`, `address_space` in
`v2_std_cardinality` and `v2_extdeps_languages_llvm_ir`). Nothing else in the closure moved.

## Discriminating evidence, and why it is a receipt rather than an enrolled control

A fixture that isolates the trigger, compiled with the pre-fix and post-fix binaries at otherwise
identical trees:

```
type Holder4                                  pre-fix        post-fix
  = HoldsDoc { held: std.layout.Doc }         held: Doc      held: Rc<Doc>     <- the defect
  | HoldsNothing
type Holder4Record {
  held: std.layout.Doc                        Rc<Doc>        Rc<Doc>           <- positive control
}
```

The struct field is the positive control: it proves the closure genuinely is at the shared layer, so
the variant field's bareness is a position defect and not a property of the carrier. The same
before/after was reproduced on the real specimen (`v2.std.nat.Nat` through `src/v2/lens/cost`).

**This is NOT enrolled as a witness, and the reason is a property of the only available surface.**
`compile_dag_rust_emit_check` — the virtual-fixture harness every emitter witness in
`dag/test/claim/` uses — returns `true` for the variant assertion on the **pre-fix** binary: its
`resolve_virtual_source_with_imports` path does not preserve the authored qualification that is the
entire trigger, so the RED is unreachable through it. Enrolling the row anyway would have added a
control that is green in both directions — the vacuous green DESIGN §4b calls rung inflation. The
second candidate specimen (`v2.std.nat`) is unreachable through that harness for the orthogonal
reason `checkpoint_identity_keying_witness_test.dag` already records: the fixture hard-fails on
`v2.std.node`'s bare-qualified references before the field is ever rendered (re-confirmed by
execution here, not assumed).

**Next-rung trigger, named rather than stalled:** a fixture surface that preserves the authored
namespace qualification — or an emitter-level assertion over a real corpus module rather than a
virtual one — makes this class mechanically preventable. Until then it is *mitigatable*: the
invalid state is writable again by any sixth field position that re-derives the layer locally,
which is the same ceiling `rust_carrier_is_at_shared_layer` already declares and this change
inherits unchanged.

## One instrument note, recorded because it cost an hour

Two `gunbc compile` runs on a ~10-file fixture timed out at 200s with `user 1.6s / sys 2m8s` and
10 GB of block reads against 30 MB RSS — host page-cache thrash from a neighbouring session, not
the compiler. The same command with the same binary completed in 1m32s minutes later. A timeout on
this host is not evidence about the binary; the pre/post comparison above was re-run to completion
on both binaries before any of it was believed.
