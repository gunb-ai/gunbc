# materialization_carriers Row 3 (unsynthesized use-line): 51 → 49 at `b2dd729f92` (2026-08-19)

**Session:** `swift-moth-294` (vertical integrator, dashboard node `adhoc-497a24aa-140`).
**Instrument:** identical to #8460/#8490 (`gunbc compile → cssl_assemble → cargo build --release --lib`
on `src/v2/compiler/materialization_carriers.dag`, `CSSL_STD_SEED_LINK=1`, raw cssl `lib.rs`,
plain-text cargo log, errors counted as `^error[` lines). Both arms executed remotely
(BuildBuddy) against **the same base SHA**, binaries built in the same dispatch as the
measurement, so binary and tree agree by construction.

## Base — and a correction to the "unmoved 51" standing

Base: `main@b2dd729f92954e50e3d8d9bbd65022c2d58f14dd`. Total is again 51, but the
**composition has moved** since #8490's receipt at `dd9c2307550` — two different 51s now
exist, so the earlier "unmoved site-for-site" reading is stale for the current tip
(reported to `smart-ram-730`, confirmed):

```
BEFORE (this base):  E0308 20 · E0277 16 · E0599 3 · E0425 3 · E0422 2 · E0369 2 · E0282 2 · E0061 2 · E0560 1   = 51
receipt @dd9c2307:   E0308 15 · E0277 16 · E0599 9 · E0425 3 · E0422 2 · E0369 2 · E0282 1 · E0560 1 (+2 lints)  = 51 counting unreachable_patterns
```

Row 3's five sites are byte-identical to the receipt on this base:
- `extdeps_realization_compile_stage_memo.rs:82` / `extdeps_realization_parse_table_memo.rs:103` — E0422
  `cannot find struct ... ProviderRetention`, at `retention: Rc::new(ProviderRetention {` — a head the
  **emitter** wrote for the authored anonymous literal `retention: { release_policy: …, capacity: … }`.
- `v2_compiler_materialization_carriers.rs:141,145,149` — E0425 on `NonEmptyStr` in emitted fn signatures
  where the authored source spells only the brand alias (`CacheInterfaceId` / `ArtifactKindId`).

## The change (E0422 half)

`src/v1/05_emit_rust.dag`: third attestation arm for `reference_derived_use_lines` —
`collect_anonymous_record_lit_heads` admits a candidate the emitter itself will render as the head of an
**anonymous** record literal (the same `tn-Absent` decision chain the candidate proposer already uses:
`emit_inferred_type_leaf_name` when the summary is known, else `anonymous_record_lit_surface_name`).
The authored-source text gate structurally cannot attest these names — the source never spells them —
and every downstream wall (registry resolve, transitive export proof, kernel / local-decl /
already-imported filters) is unchanged. Stage0 projection spliced at exactly these hunks
(`v1_compiler_emit_rust.rs`); the surrounding committed-seed drift found during this work
(#8466 wiring, #8460 callable-wrap, #8516 function-value lowering — all present in a clean regen,
absent from the committed seed) is deliberately **not** carried; routed to its owners.

## Result — same instrument, same base, spliced seed

```
AFTER:  E0308 20 · E0277 16 · E0599 3 · E0425 3 · E0369 2 · E0282 2 · E0061 2 · E0560 1   = 49
```

E0422 row → 0 (both sites; the memo modules now carry
`pub use crate::std_cache_interface::{ProviderRetention};`). **No other row moved** — the delta is
exactly the two claimed sites, verifiable at composition grain above, not only by total.

Witness: `dag/test/claim/anonymous_record_head_use_line_witness_test.dag` — reproduces the mechanism
through the real emit pipeline (fixture references `ProviderRetention` only through an anonymous
argument-position literal; the name is spelled nowhere in fixture source), RED against the pre-fix
emitter, green with the fix; negative assertion pins the over-collection boundary.

## Declared residue (Row 3's remaining 3 sites — next packet, not silently absorbed)

The E0425 `NonEmptyStr` trio did **not** move, and an attempted companion arm was **removed after
measurement showed zero effect** (a mechanism with no measured effect must not ship): the signature's
top-level type-node leaf is `CacheInterfaceId` (already imported); the `NonEmptyStr` spelling is
produced by a deeper alias-peel inside `render_rust_fn_sig_type`'s resolved-type rendering, so neither
the candidate set nor a top-leaf attestation carries the name the emitted signature actually spells.
The repair needs the renderer's own leaf decision (preserve the authored alias leaf, or attest the
rendered base), root-caused separately.
