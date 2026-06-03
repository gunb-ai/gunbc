# v4 Rust RCA Manager Worksheets — 2026-06-03

Scope: #4137 Section 11.8 Rust release-minimum / M1 rustc residual lane. These
worksheets cluster the live ratchet meter (PR #4140: **7,724** `error[E####]`
lines at post-Jun1-cascade) into routed subfamilies. Each subfamily names a
single-authority fact and a falsification probe before implementation dispatch.

**Live ratchet authority:** PR #4140 (`docs(audit): refresh Jun1 M1 rustc
residual catalog`). The committed audit tree was removed in #4192; recover the
7,724-error histogram via
`git show 65e8db2ac0:docs/audit/v4-rustc-error-catalog-2026-06-01-post-jun1-cascade.md`
(see that file's §5 repro — extended emit + `cargo check`, not the live CI gate).

**Live M1 CI gate:** `.github/ci-floor/v4-m1-rust-emit-probe.sh` (authority:
`src/v4/workflow/ci.dag`) — v2 `--target rust` emit with a
`compiled: N files emitted, 0 diagnostics` receipt only; **no `cargo check`**.

**Manager session:** `bright-crane-734` (`node://adhoc-65299e42-409`).

**Closed classes at #4140 probe:** SG-1 (`E0423 = 0`), SG-7 (v2 emit diagnostics
= 0). Count growth (+549 vs #4122) is residual-population expansion after
substrate landings, not class reopening.

## Dispatch Rule

The Rust lane advances by **routed subfamily**, not by error code:

1. No worker may land on "fix E0308" or "fix SG-2" without naming the
   single-authority fact and forbidden-pattern greps.
2. Error-count reduction is secondary evidence only.
3. Cross-language facts land in `src/v4/std/target_model.dag` (or another
   ratified canonical home) before `extdeps/languages/rust.dag` rows or translate
   consumers change.
4. Name-keyed import tables, carrier arity shims, or per-error text patches
   violate INVARIANTS P2/P5 and must be rejected.

## Routing Summary

| Route | Residual population (#4140) | Worksheet | Dispatch state |
| --- | ---:| --- | --- |
| SG-8 module graph / import / re-export | `E0425`+`E0432`+`E0433` = **1,224** (+420 vs #4122) | [SG-8 addendum](v4-sg8-rca-ratification-addendum-2026-06-03.md) | §8 **ratified** (#4143); ready for impl |
| E0308 stratified | **2,953** (dominant code, not dominant delta) | [E0308 stratified](v4-e0308-stratified-rca-worksheet-2026-06-03.md) | P0 slices route to SG-1b / SG-RC / SG-COLLECTION |
| SG-2 residual | `E0107`+`E0282` = **2,661** (+75 vs #4122) | [SG-2 residual](v4-sg2-residual-rca-worksheet-2026-06-03.md) | Consumer-coverage slice on approved SG-2 substrate |
| Stable bands | `E0277`, `E0573`, `E0369`, `E0560`, `E0121`, tail | [Stable bands](v4-stable-rustc-bands-rca-worksheet-2026-06-03.md) | Second-order; no broad SG-3 lane |

## Fanout Order

1. **SG-8** — highest delta driver; implement per #4127 worksheet after §8
   ratification (#4143). Target: `emit_imports` defining-module authority +
   parametric alias emission; F1–F4 falsification.
2. **SG-2 residual** — extend approved `TargetTypeExpressionProjection` consumer
   coverage to aliases, cached statics, function signatures, closure annotations,
   and constructor result types (`ProbePair<T,U>` five-site probe).
3. **E0308 P0** — parallel slices only through existing or ratified worksheets:
   - String/Symbol → SG-1b function-signature realization (#4099)
   - Rc/raw/Box mismatches → SG-RC-LAYERING (#4116 / manual `sg_rc_layering.dag`)
   - FreeMonoid vs `Vec<Rc<T>>` → SG-COLLECTION-PROJECTION (#4151; substrate gated on main)
4. **Stable bands** — attach to primary routes (`E0573` → SG-8, collection `E0277`
   → SG-5); hold constructor-field-admission until post-P0 remeasure.

## Worksheet Index

- `docs/planning/v4-sg8-rca-ratification-addendum-2026-06-03.md`
- `docs/planning/v4-e0308-stratified-rca-worksheet-2026-06-03.md`
- `docs/planning/v4-sg2-residual-rca-worksheet-2026-06-03.md`
- `docs/planning/v4-stable-rustc-bands-rca-worksheet-2026-06-03.md`

## Related Artifacts

- `.github/ci-floor/v4-m1-rust-emit-probe.sh` — M1 full-tree v2 rust emit + compile-log receipt (no `cargo check`)
- `src/v4/test/claim/manual/sg_rc_layering.dag` — SG-RC falsification F1–F6
- `src/v4/test/claim/manual/sg_collection_projection.dag` — SG-COLLECTION receipt
- `src/v4/std/target_model.dag` — SG-RC / collection / type-expression authority
- `docs/planning/v4-go-rca-manager-worksheets-2026-06-03.md` — parallel §11.8 lane shape
