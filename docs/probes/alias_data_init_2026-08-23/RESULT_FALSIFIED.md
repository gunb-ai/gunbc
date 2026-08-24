# RESULT: the pre-registered prediction was FALSIFIED, and the repair is reverted

Registered prediction (`PRE_REGISTRATION.md`, committed before any treatment existed):
**E0308 128 → 115, coded board 316 → 303**, by removing 13 `v2_lens_coverage.rs` rows.

## After-arm

Same instrument, same entry, treatment commit `8ea34b3065` — whose diffstat **includes the
regenerated `src/v1/stage0/src/v1_compiler_emit_rust.rs`**, so the probe compiled a mirror carrying
the change. That is checkable from the commit rather than from the author's account of their window.

| | before-arm | after-arm | predicted |
|---|---|---|---|
| E0308 | 128 | **128** | 115 |
| coded rows | 316 | **316** | 303 |
| the 13 `Coverage` rows | present | **present** | removed |
| uncoded `unsupported mock expression` | 13 | **55** | — |
| `CARGO_ERROR_TOTAL` | 330 | **372** | — |

**Refusal condition 3 fired.** The registration said a fall to exactly 115 with the 13 named rows
gone and nothing else changed was the only clean pass. The target rows did not move at all.

The treatment was **not** inert — it added 42 refusals, which is what proves it executed. Those 42
were already-defective rows moving from *silently wrong* to *loudly refused*, the direction DESIGN §5
asks for; they are recorded as a separate candidate rather than smuggled in on a falsified prediction.

## Why the fixture passed while its own subject did not

The four-cell fixture went green on every cell. The corpus did not move. **A fixture that greens
while the rows it was built to explain do not is specification-without-execution one level up.**

The discriminator, found by instrumenting the mirror directly and compiling only
`src/v2/lens/coverage.dag`:

```
[DBG] name="CoverageDefectAcceptance" direct=false selfconn=NoConnective expconj=Conj   ×13
```

The alias resolution **works** — `expconj=Conj` on all 13. The rows survive because of the *second*
conjunct of the patched branch:

```
} else if has_nested_records_node(…) && !data_value_has_cross_refs(value: value) {
```

Corpus values reference an enum variant (`lens: CoverageDefectKey::DiscriminantPredicate`), so
`data_value_has_cross_refs` is true, the serde branch is declined, and control falls through to
`emit_typed_expr` — where the record collapses. The original fixture used `lens: true`, a bare
literal with **no cross-refs**, so it took the serde path and went green.

`controls/alias_data_init_pair.dag` now carries a `via_alias_crossref` cell that emits
`Rc::new(FlagKey { lens: Flag::Up })` — **the corpus shape exactly**. The fixture reproduces a real
corpus row; the earlier one did not.

## What stands and what does not

**Stands, each proven by execution:** the alias is unresolved at that site;
`record_lit_alias_struct_expansion` is the accessor `infer` uses and `emit` does not; resolving it
changes emission.

**Does not stand:** that this is the decision producing the 13 rows. "13 rows from one decision" is
**unproven**. The rows are produced in `emit_typed_expr`'s handling of an alias-typed record literal
on the cross-ref path.

**Also does not stand:** an earlier conclusion in this lane that the value side was fine. That rested
on a `build_alias_in_body` arm which was *also* cross-ref-free and therefore could not have detected
the defect at all.

## Why the repair is reverted rather than landed

Beyond missing its target, the change caused regen drift across mirrors it never regenerated
(`extdeps_languages_dag_emit.rs`, `extdeps_languages_dag_syntax.rs`, `extdeps_languages_go_emit.rs`,
and more) — so its blast radius is far wider than the 13 rows it was aimed at. The emitter is
restored byte-identical to this branch's base `907f19c2cc`.

## Two mechanisms recorded for other lanes

1. **Execution-provenance loss.** A failed regen leaves the *previous* successful candidate in the
   output directory; an install step gated on the directory existing rather than on the regen's exit
   status will silently test an older treatment. The worst case — hit here — is when the stale
   artifact is the author's own prior patch, similar enough to survive a sanity read. Gate on exit
   status.
2. **The elided-spelling grep artifact.** rustc elides the type in the headline
   (`` expected `Coverage<Rc<...>>` ``) and spells it in full only in the trailing note. Counting the
   full spelling returned 0 and read exactly like a fix; counting it on the *before*-arm returned 13.
   A count with no known-positive control is a string search that happens to return a number.

## Grammar specimen (not taken here — recorded for dispatch)

`if x.connective == Conj { … } else { … }` fails with `expected LBrace, found keyword 'else'`,
because `Conj { … }` parses as a **record literal** and swallows the then-branch. The diagnostic
points at the `else`, which has nothing to do with the cause. Rust forbids struct literals in
condition position for exactly this reason. The corpus already pays for this in workarounds
(parenthesising, or routing through `is_product_type`).
