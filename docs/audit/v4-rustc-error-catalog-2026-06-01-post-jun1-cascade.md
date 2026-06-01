# v4 rustc error catalog -> post-Jun1-cascade remeasure — 2026-06-01

**Manager session:** `sleek-heron-13` (M1 rustc probe lane).
**Authority:** dashboard node `adhoc-7b46e080-3cd` ("Fresh M1 rustc probe post-Jun1-cascade").
**Live ratchet meter:** this catalog replaces `docs/audit/v4-rustc-error-catalog-2026-05-31-post-p5.md` / PR #4122 as the current full-tree M1 rustc residual measurement. The #4122 catalog remains the 2026-05-31 post-#4115 baseline.
**Reference commit:** `origin/main` at **`483d82a78`** (`docs(planning): post-#4111 burn-down — HEAD refresh + landing log row (#4112)`), verified by `git rev-parse HEAD origin/main` immediately before the probe.
**Probe:** `scripts/v4-m1-rust-emit-probe.sh`, with `V2_COMPILER=target/release/gunbc`, `V4_M1_RUST_EMIT_OUT=/tmp/v4-rust-emit-jun1-sleek-heron-13`, `V4_M1_CARGO_CHECK_JOBS=4`.
**Raw summary:** `docs/audit/v4-rustc-error-catalog-2026-06-01-post-jun1-cascade.m1-probe-summary.txt`.

---

## §1 Headline

| Metric | 2026-06-01 post-Jun1-cascade | 2026-05-31 post-#4115 (#4122) | Delta |
| ------ | ----------------------------:| -----------------------------:| -----:|
| v2 emit diagnostics | **0** | 0 | 0 |
| `.rs` files emitted | 351 | 345 | +6 |
| `.rs` files on disk | 350 | 344 | +6 |
| rustc `error[E####]` lines | **7,724** | 7,175 | **+549** |
| Distinct emitted `.rs` files with errors | 310 | 305 | +5 |
| Top code | `E0308` (2,953) | `E0308` (2,905) | unchanged leader |
| `E0423` (SG-1 closure check) | **0** | 0 | 0 |

**Plain-English summary.** The stale #4122 count was 7,175 errors at the post-#4115 measurement point. After the Jun 1 cascade landings now on `origin/main` (#4118 SG-1b implementation, #4124 SG-2 worksheet closure/P5 smoke, #4116/#4133/#4135 SG-RC-LAYERING implementation/follow-up, #4127 SG-8 worksheet, #4120 T-38-PR2, #4101 CI bankruptcy), the fresh full-tree M1 rustc residual is **7,724**. That is **+549 errors** over #4122, while v2 emit remains clean at **0 diagnostics**.

This is a measurement receipt, not a class closure claim. The dominant error remains `E0308`; the largest visible movement is the SG-8 code family (`E0425`/`E0432`/`E0433`) growing by **+420** combined, which is consistent with more module/re-export surface being present after the worksheet/substrate landings rather than with SG-1 or SG-7 reopening.

---

## §2 Code histogram

| Code | 2026-06-01 | #4122 | Delta | Notes |
| ---- | ---------:| -----:| -----:| ----- |
| `E0308` | 2,953 | 2,905 | +48 | mismatched types; still the dominant residual |
| `E0107` | 1,654 | 1,629 | +25 | generic arity |
| `E0282` | 1,007 | 957 | +50 | type annotations needed |
| `E0425` | 538 | 485 | +53 | unresolved value/type names |
| `E0432` | 415 | 238 | +177 | unresolved imports |
| `E0277` | 330 | 330 | 0 | trait bound |
| `E0433` | 271 | 81 | +190 | failed to resolve |
| `E0573` | 159 | 159 | 0 | expected type, found variant |
| `E0560` | 126 | 122 | +4 | missing struct field |
| `E0369` | 110 | 110 | 0 | binary op on `Rc<T>` |
| `E0121` | 44 | 44 | 0 | placeholder `_` in item signature |
| `E0391` | 29 | 29 | 0 | cyclic dependency |
| `E0599` | 28 | 28 | 0 | no method found |
| `E0392` | 12 | 12 | 0 | unused type parameter |
| `E0061` | 12 | 10 | +2 | wrong argument count |
| `E0614` | 8 | 8 | 0 | cannot dereference |
| `E0609` | 6 | 6 | 0 | unknown field |
| `E0559` | 5 | 5 | 0 | variant field mismatch |
| `E0428` | 4 | 4 | 0 | duplicate definition |
| `E0252` | 4 | 4 | 0 | duplicate import |
| `E0610` | 2 | 2 | 0 | field access on primitive |
| `E0283` | 2 | 2 | 0 | type annotations needed |
| `E0109` | 2 | 2 | 0 | type arguments not allowed |
| `E0505` | 1 | 1 | 0 | move while borrowed |
| `E0422` | 1 | 1 | 0 | cannot find struct/variant |
| `E0072` | 1 | 0 | +1 | recursive type without indirection |
| **TOTAL** | **7,724** | **7,175** | **+549** | |

**Histogram reconciliation.** The top-25 lines in the raw probe summary sum to 7,723. The full rustc log has one additional `E0072` outside the displayed top-25, yielding the recorded total of 7,724.

---

## §3 Delta Readout

| Family | Codes | Delta | Readout |
| ------ | ----- | -----:| ------- |
| SG-8 / module graph + re-exports | `E0425` + `E0432` + `E0433` | **+420** | Biggest movement; #4127 worksheet is now on main, and this probe provides the fresh residual population after that surface expansion. |
| SG-2 / generic arity + inference | `E0107` + `E0282` | **+75** | Generic/inference family grew modestly with six more emitted files. |
| Type mismatch / ownership/value boundary | `E0308` | **+48** | Still the largest code, but not the main source of the +549 delta. |
| SG-3 stable bands | `E0277` + `E0573` + `E0369` + `E0121` | 0 | These stayed pinned relative to #4122. |
| Long tail | all remaining codes | +6 | Mostly `E0560` (+4), `E0061` (+2), and new single `E0072`. |

SG-1 remains closed at this probe (`E0423 = 0`). SG-7 remains closed at this probe (`v2 emit diagnostics = 0`). The count increase is therefore residual-population growth after additional substrate/model surface landed, not a reopening of either closed class.

---

## §4 Repro

```bash
PATH=/opt/cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin \
  CARGO_BUILD_JOBS=4 \
  /opt/cargo/bin/cargo build -p v2-compiler --release

PATH=/opt/cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin \
  V2_COMPILER=target/release/gunbc \
  V4_M1_RUST_EMIT_OUT=/tmp/v4-rust-emit-jun1-sleek-heron-13 \
  V4_M1_CARGO_CHECK_JOBS=4 \
  bash scripts/v4-m1-rust-emit-probe.sh
```

`ctrl-build -- cargo build ...` was not used for the successful build in this session because the local shim path recursed through `/usr/local/bin/ctrl-build -- cargo ...`. Direct `/opt/cargo/bin/cargo` with `CARGO_BUILD_JOBS=4` avoided the shim recursion and kept the build capped.

---

## §5 Related Artifacts

- `docs/audit/v4-rustc-error-catalog-2026-06-01-post-jun1-cascade.m1-probe-summary.txt` — raw probe summary committed with this catalog.
- `docs/audit/v4-rustc-error-catalog-2026-05-31-post-p5.md` and `.m1-probe-summary.txt` — #4122 baseline at 7,175 errors.
- `scripts/v4-m1-rust-emit-probe.sh` — probe script used for both measurements.
