# Two type-relation gaps surfaced by deleting the `v2.*` argument-type exemption

Deleting `module_skips_direct_call_arg_check` (cut B) makes the direct-call
argument-type judgment run over `v2.*` modules for the first time. The witness
floor then refuses on its prepared subject:

```
required-ci: floor refused: subject=ac87f26533b2a89c modules_resolved=3867 modules_excluded=4
```

19 distinct located diagnostics, 9 modules, two families. **Neither family is
source conformance debt.** Both are gaps in the type relation, each reproduced
below at a grain that carries no production vocabulary, and each with a green
control that fails to fire.

Measured on `71d7da4e92` + the deletion, binary md5 `7e7da4bf5ef93fc24e2d8fd600e87a3b`.

---

## Family 2 — a transparent alias whose RHS is a generic instantiation is not peeled

8 sites, one per formatter module. Production shape:

```
type GofmtConfigPatch = ConfigPatchRecord<GofmtConfig>
fn gofmt_layer(base: GofmtConfig, patch: GofmtConfigPatch) -> GofmtConfig {
  config_patch_layer(base: base, patch: patch)
}
```

`config_patch_layer`'s formal is `ConfigPatchRecord<Config>` — the one shared
authority in `v2.std.patch`, correctly parameterized and correctly instantiated
eight times. There are not eight per-tool models.

**Reproduction** (12 lines, no formatters, no `ConfigPatchRecord`):

```
module v2.probe.aliasgen
type Box<T>
fn takes_box<T>(b: Box<T>) -> Bool { true }
type IntBox = Box<Int>
fn c_alias_of_generic(b: IntBox)  -> Bool { takes_box(b: b) }   // RED
fn c_direct_generic(b: Box<Int>)  -> Bool { takes_box(b: b) }   // GREEN control
```

```
type mismatch: expected 'Primitive(Box)', got 'Node(IntBox)'
```

Both sides fail at once: the expected side has **lost its type argument**
(`Primitive(Box)`, not `Box<Int>`) and the got side is the **alias name
unresolved** (`Node(IntBox)`). Neither was peeled.

This is a gap in the transparent-alias relation landed by #8873. #8873 merged,
so this is a main-branch seam, not a PR review note.

---

## Family 1 — the `T?` cardinality is dropped across an import boundary

11 sites, all in `src/v2/extdeps/github/gha_fold_pilot_emit.dag`. The values are
fields of a `RunStep` variant declared in `dag/extdeps/github/actions.dag` as
`name: String?`, passed straight through to formals spelled `Optional<String>`.
Nothing is unwrapped in the source.

**Reproduction** (3 modules):

```
module v2.probe.optdecl                       // the DECLARING module
import v2.std.optional { Optional }
type S2 = A2 { fs: String?, fl: Optional<String> } | B2 { z: Int }
```

```
module v2.probe.xsugar                        // RED
import v2.probe.optdecl { S2, A2, B2 }
import v2.std.optional { Optional }
fn takes_opt_s(o: Optional<String>) -> Bool { true }
fn c_cross_sugar(s: S2) -> Bool {
  match s { A2 { fs: x, fl: y } => takes_opt_s(o: x)  B2 { z: _ } => true }
}
```

```
type mismatch: expected 'Coproduct(Optional)', got 'Primitive(String)'
```

`v2.probe.xlong` is identical but passes `y`, the long-form field, and is clean.

**The defect needs both variables at once.** The full factorial:

| declaring module | field spelling | result |
|---|---|---|
| same as the call | `String?` | CLEAN |
| same as the call | `Optional<String>` | CLEAN |
| imported | `Optional<String>` | CLEAN |
| **imported** | **`String?`** | **RED** |

So the `?` sugar resolves to `Coproduct(Optional)` within its declaring module
and does **not** resolve when the variant's field type is read through an import
— the exported field type arrives as the bare inner type with the cardinality
dropped. It is neither a sugar defect nor a match-binder defect; both were
cleared by the same-module cells, and clearing them is what located this.

Three readings were refuted before this one, and are recorded so they are not
re-derived: the sugar is not un-equated with `Optional<T>`; the match binder does
not lose the wrapper; and these sites are **not** an instance of the `T?`-as-
kernel-cardinality class — under that reading the same-module sugar cell would
have had to refuse, and it did not.

**That third clause was scoped too widely as first written, and the correction is
load-bearing (2026-08-22, `crisp-crab-430`, with a controlled cell run against the
reproduction above).** It is sound only *at the parameter position*. At the
**construction** position the kernel-cardinality reading survives, measured with a
green control in one module and no boundary crossed:

```
type Holder { d: probe.carddecl.Digest2? }
Holder { d: Present { value: Digest2 { hex: "aa" } } }                    // CLEAN
Holder { d: v2.std.optional.Present { value: Digest2 { hex: "bb" } } }    // REFUSES
    type mismatch: expected 'Product(Digest2)', got 'Coproduct(Optional)'
```

**These are two relations, not one, and the discriminator is the precondition.**
The cells in this document red only on a tree carrying the cut B deletion and are
silent on stock main; that cell reds on a stock binary, against a subject under
`dag/` the exemption never covered. Different gate, different position, different
precondition. So a `T?` field expects `Product(T)` and admits the kernel's bare
`Present` while refusing `v2.std.optional.Present` — in-module — whereas at a
formal parameter both spellings interchange in-module and the cardinality is lost
through an import.

**And that inverts the reading of one of this document's own clean cells.** If
`T?` is genuinely a kernel cardinality distinct from `v2.std.optional.Optional<T>`,
then the same-module `T?` → `Optional<String>` parameter cell passing is not
evidence of correctness — it is a candidate **false accept**, a leniency at the
parameter position in the opposite direction from seam B's false refusal. This is
named as a candidate and not as a finding: no cell here discriminates "the two
types are the same at this position" from "the comparison at this position does not
distinguish them." Closing it needs a construction-position control at a parameter
position, which nobody has run.

---

## Method note

The first family-1 probe came back clean on both cells and nearly closed the
question. It was a faithful model of the *hypothesis*, not of the *site*: every
difference not deliberately varied had been silently set to the easy value —
same module, same file, same source root. **A probe that green-lights on the
first try has usually reproduced the author's belief, not the defect.** Before
running a reproduction, list what was held fixed, not only what was varied; the
held-fixed list is where the defect hides.
