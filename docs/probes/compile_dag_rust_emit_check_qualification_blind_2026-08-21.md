# `compile_dag_rust_emit_check` cannot express a RED whose trigger is an authored qualified spelling (2026-08-21)

**This is a finding about the INSTRUMENT, not about any one repair.** It is filed separately from
[`e0599_as_ref_variant_field_layer_2026-08-21.md`](e0599_as_ref_variant_field_layer_2026-08-21.md),
which is where it was discovered, because it bounds the standard witness route for the whole corpus
and would be invisible to anyone reading only that lane.

Session `sunny-pike-280`. Found by trying to enrol a witness and failing to make it go red.

## The claim

`compile_dag_rust_emit_check(source, path, includes, excludes)` — the virtual-fixture harness every
emitter witness in `dag/test/claim/` uses — **does not preserve the authored namespace
qualification of a type reference**. A fixture whose defect is triggered by the difference between
`held: Doc` and `held: std.layout.Doc` therefore returns the SAME verdict on a binary that has the
defect and a binary that does not.

The failure mode is the dangerous direction: such a witness is **green in both directions**. Nobody
authoring one would notice, because green reads as passing. It is coverage by illusion (DESIGN §6),
and a class of would-be witnesses is silently converted into inert ones.

## Evidence (executed, both directions, one fixture)

The fixture:

```
module vfsl4
import std.layout { Doc }

type Holder4
  = HoldsDoc { held: std.layout.Doc }     <- qualified spelling: the trigger
  | HoldsNothing

type Holder4Record {
  held: std.layout.Doc                     <- positive control (a position already repaired)
}
```

Two binaries, identical except for one emitter change (gunbc#8814), same tree otherwise:

| route | assertion | pre-fix binary | post-fix binary | discriminating? |
|---|---|---|---|---|
| `gunbc compile --entry <fixture>` | variant field rendering | `held: Doc` | `held: Rc<Doc>` | **yes** |
| `gunbc compile --entry <fixture>` | struct field rendering | `held: Rc<Doc>` | `held: Rc<Doc>` | control holds |
| `compile_dag_rust_emit_check` | `includes ["held: Rc<Doc>,"]` | **`true`** | `true` | **NO — vacuous** |
| `compile_dag_rust_emit_check` | `includes ["pub held: Rc<Doc>,"]` | `true` | `true` | control holds |

The third row is the finding: the same assertion that discriminates through `gunbc compile` is
already satisfied by the *defective* compiler when it is asked through the harness.

## Two further mechanics, confirmed while establishing the above

1. **`--source-root` flags do not reach the fixture.** The harness resolves through
   `resolve_virtual_source_with_imports` against `build_module_path_index_from_witness_roots` — the
   repo's witness roots — so source roots passed to `gunbc run` change nothing about what the
   fixture sees. A fixture that compiles under `gunbc compile --source-root src/v2` can still fail
   inside the harness.
2. **`v2.*` fixtures are largely unreachable, and this is re-confirmed rather than assumed.** A
   fixture importing `v2.std.nat { Nat }` returns `false` from the harness with EMPTY includes and
   excludes — i.e. it never compiles, so no assertion of any kind is being evaluated. This is the
   same gap `dag/test/claim/checkpoint_identity_keying_witness_test.dag` records for its cross-decl
   row (`v2.std.node`'s bare-qualified references are not reachable through this surface's
   import-closure discovery). Executed here, not carried over.

Both mechanics mean a `false` from this harness is not a refusal oracle: it collapses "the assertion
failed" with "the fixture never compiled". Pair every `includes`/`excludes` row with an
`includes: [], excludes: []` compile-only row before believing a red.

## What to do with it

**Before enrolling any emitter witness, run its fixture through a binary that HAS the defect and
require RED there.** If the trigger is an authored spelling, expect this harness to erase it. The
honest substitute — used by gunbc#8814 — is a recorded pre/post `gunbc compile` receipt with a
positive control, stating in the PR and the probe doc that no witness is enrolled and why.

**Next rung.** A fixture surface that preserves authored qualification, or an emitter-level
assertion over a real corpus module rather than a virtual one, restores this class to
*mechanically preventable*. Until then, witnesses over spelling-sensitive emitter behaviour sit at
*mitigatable* and the gap is this document.
