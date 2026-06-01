# v4 Rust RCA Manager Pass — 2026-06-01

> **Status:** RCA ROUTING PASS — implementation fanout gated by worksheet approvals.
> **Manager session:** `vivid-lynx-81` (`adhoc-65299e42-409`).
> **Authority inputs:** PR #4137 §11.8 manager role; PR #4140 fresh M1 rustc catalog; `INVARIANTS.md` P2/P5; `src/v3/SELF_HOSTING.md` model-before-implementation discipline.
> **Live ratchet meter:** `docs/audit/v4-rustc-error-catalog-2026-06-01-post-jun1-cascade.md` — 7,724 rustc `error[E####]` lines; v2 emit clean (351 emitted files, 0 diagnostics); SG-1 and SG-7 remain closed.

---

## Routing Summary

| Route | Residual population | State | Next action |
| --- | ---:| --- | --- |
| SG-8 module graph / import-export / re-export | `E0425` + `E0432` + `E0433` = 1,224 on #4140; +420 vs #4122 | Existing worksheet on main; §8 ratification still open | `docs/planning/v4-sg8-rca-ratification-addendum-2026-06-01.md`; dispatch only after proud-fox-405 ratifies |
| E0308 stratified | 2,953 | Split into root-cause subfamilies, not one broad implementation lane | `docs/planning/v4-e0308-stratified-rca-worksheet-2026-06-01.md` |
| SG-2 residual | `E0107` + `E0282` = 2,661 | Existing SG-2 worksheet approved; residual requires alias/cache/signature preservation slice | `docs/planning/v4-sg2-residual-rca-worksheet-2026-06-01.md` |
| Stable trait/value bands | `E0277`, `E0369`, `E0573`, `E0560`, `E0121`, plus tail | Mixed second-order bands; no new SG-3 blob | `docs/planning/v4-stable-rustc-bands-rca-worksheet-2026-06-01.md` |

---

## Probe Receipts

#4140 is the live ratchet authority. This manager pass also regenerated the M1 probe locally to recover an E0308 pair histogram because the original `/tmp` rustc log was not present in this container.

Local commands:

```bash
ctrl-build -- /opt/cargo/bin/cargo build -p v2-compiler --release

ctrl-build -- env \
  V2_COMPILER=target/release/gunbc \
  V4_M1_RUST_EMIT_OUT=/tmp/v4-rust-emit-jun1-vivid-lynx-81 \
  V4_M1_CARGO_CHECK_JOBS=4 \
  bash scripts/v4-m1-rust-emit-probe.sh
```

Local result: v2 emit clean and E0308/SG-2/stable-band counts matched #4140 exactly. `E0432`/`E0433` were lower locally, so SG-8 counts in this pass intentionally defer to #4140 rather than replacing the ratchet meter.

| Code | #4140 | Local regenerated probe | Use in this pass |
| --- | ---:| ---:| --- |
| `E0308` | 2,953 | 2,953 | local pair histogram is accepted for stratification |
| `E0107` | 1,654 | 1,654 | local carrier histogram is accepted for SG-2 residual shape |
| `E0282` | 1,007 | 1,007 | local file histogram is accepted for SG-2 residual shape |
| `E0277` | 330 | 330 | accepted |
| `E0369` | 110 | 110 | accepted |
| `E0573` | 159 | 159 | accepted |
| `E0560` | 126 | 126 | accepted |
| `E0121` | 44 | 44 | accepted |
| SG-8 family | 1,224 | 861 | #4140 remains authority |

---

## Dispatch Rules

1. No SG-class implementation dispatch may land until the relevant worksheet names the single-authority fact and proud-fox-405 ratifies the worksheet gate.
2. Error-count reduction is secondary evidence only. Acceptance is falsification behavior plus forbidden-spot-fix greps.
3. Rust emitter changes that introduce name-keyed carrier tables, import shim lists, or per-error text patches violate P2/P5 and must be rejected.
4. Cross-language facts land in `src/v4/std/target_model.dag` or another ratified canonical home first; Rust rows in `extdeps/languages/rust.dag` are per-target data, not the global authority.

---

## Fanout Order

1. **SG-8:** wait for §8 ratification of `docs/planning/v4-sg8-module-graph-carrier-reexports-worksheet-2026-05-31.md`; then spawn implementation worker for `emit_imports` defining-module authority + parametric alias emission.
2. **SG-2 residual:** fan out to Target Realization only after confirming the already-approved `TargetTypeExpressionProjection` implementation surface covers aliases, cached statics, function signatures, closure annotations, and constructor result types.
3. **E0308 P0 slices:** route String/Symbol to SG-1b, Rc/raw/Box to SG-RC-LAYERING, FreeMonoid/Vec to SG-COLLECTION-PROJECTION, and TestClaim double-wrap to ownership/use-site rows. Do not dispatch "fix E0308".
4. **Stable bands:** dispatch only the bands with independent single-authority facts that already have an owning worksheet (`E0277` trait eligibility via SG-5, `E0121` placeholder ban as fail-closed projection). Hold `E0560` as worksheet-only until a post-P0 remeasure proves a constructor-field-admission lane is still needed. Leave second-order bands attached to their primary route.
