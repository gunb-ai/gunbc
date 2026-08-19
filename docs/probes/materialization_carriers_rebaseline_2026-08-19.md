# materialization_carriers: re-baseline on current main, board unchanged site-for-site (2026-08-19)

**Session:** `eager-ant-366` (measurement authority for this module, dashboard node
`adhoc-dfa0b3ca-c9f`, succeeds `witty-heron-413`/PR #8460, archived).
**Subject:** `src/v2/compiler/materialization_carriers.dag`, same instrument as #8460.

Nothing here is transcribed from the predecessor's document. Every number was produced by a run
made for this receipt, against current `main`.

## 1. Instrument (unchanged from #8460 §1 — single authority, not re-derived)

```
gunbc compile --source-root dag --source-root src/v2 \
  --entry src/v2/compiler/materialization_carriers.dag --target rust \
  --dependency-pool-index primary-precedence --output-dir <out>
cssl_assemble --out-dir <out> --entry-dag src/v2/compiler/materialization_carriers.dag --root .
cd <out> && cargo build --release --lib --message-format=json
```

`CSSL_STD_SEED_LINK=1`, no lane shim (raw cssl-assembled `lib.rs`), `Cargo.toml` rendered via
`docs/probes/lib/render_cssl_probe_lib_cargo_toml.sh` (the harness's own single authority for
that file — not a parallel heredoc). Counts are errors as rustc reports them in the JSON stream,
**at (code, primary span, message, children) grain** — never a keyword/text scan.

Binaries: `gunbc` and `cssl_assemble` built locally on this branch
(`CTRL_BUILD_WRAP_CARGO=0 cargo build --release -p v1-compiler --bin gunbc --bin cssl_assemble`)
immediately before this run, so the binaries and the measured tree agree.

## 2. Base

`main` at `72676cf0be992a6305d312d2e93ca7fc1fd7edef` (HEAD at measurement time) — the tip after
#8460, #8410, and #8417, plus three unrelated merges since #8460 (#8465 grammar sync tokens,
#8458 delete fleet-converge unit renderer, #8453 systemd unit_file; none touch
`materialization_carriers.dag`, its emitter files, or `dag/std/algebra.dag`/`cache_interface.dag`
per file history).

`gunbc compile` itself: 0 blocking errors, 187 advisory diagnostics (policy
`gunbc.compile_clean_diagnostic_policy`) — unchanged in kind from #8460, not a new defect class.

## 3. The count: **51, unmoved**

```
cargo build --release --lib --message-format=json  →  exit 101, 51 error diagnostics
E0277 16 · E0308 15 · E0599 9 · E0425 3 · E0422 2 · E0369 2 · unreachable_patterns 2 · E0560 1 · E0282 1
```

This is the same total #8460 §5 reported against `origin/main 2c65eeacf3` (the base that already
included #8410). **Stated explicitly per the instruction to pair every zero (here: zero movement)
with verification, not silence:** this is not just a total-count coincidence. Every one of the 51
diagnostics was matched by primary span + code against #8460 §7's board and §10.3's 1a/1b split,
and every one lands in the same row, at the same file:line:col, as before. Nothing merged since
#8460 touches this closure. The unmoved 51 is a real "nothing changed here" result, not an
instrument reading a stale base — confirmed by re-deriving the board from scratch below rather
than by trusting the total alone.

## 4. The board, restated against `72676cf0be`, row-for-row identical to #8460 §7/§10.3

| # | mechanism | pop | unchanged? |
|---|---|---:|---|
| 1a | Clone bound missing on generic **derived impls** | 12 | yes — same 10 E0277 (`im::Vector<T>` Debug/Serialize/Deserialize) + 2 E0369 (derived `PartialEq`) at `v2_std_algebra.rs:43,45,84,88` |
| 1b | Clone bound missing on generated **fn/inherent-impl signatures** | 16 | yes — same 6 E0277 + 6 E0599 (`v2_std_staging.rs`, `v2_compiler_materialization_carriers.rs`) + 2 E0599 `CacheLookupResult<T>` (`std_cache_interface.rs:564,580`) + 2 E0308 (`v2_std_algebra.rs:118`, `v2_std_staging.rs:31`) |
| 2 | Optional carrier fork | 6 | yes — same 4 E0308 `std_cache_interface.rs:638,695,699,703` + 2 E0308 `extdeps_uri.rs:752,756` |
| 3 | Unsynthesized use-line (root K) | 5 | yes — same 2 E0422 `ProviderRetention` + 3 E0425 `NonEmptyStr` |
| 4 | Int literal into branded-string field | 4 | yes — same 4 E0308, `compile_stage_memo.rs:95,101`, `parse_table_memo.rs:116,122` |
| 5 | Type alias materialised twice | 3 | yes — same 3 E0308, `v2_std_node.rs:69,78,84` |
| 6 | Nested coproduct pattern flattened | 2 | yes — same 2 `unreachable_patterns`, `v2_std_node.rs:533,537` |
| 7 | ContentHash carrier vs `String` (T7) | 1 | yes — same 1 E0599 `partial_cmp`, `v2_std_node.rs:1334` — **owned by `calm-lynx-547`, untouched** |
| 8 | Record literal through shared-wrapped alias | 1 | yes — same 1 E0560, `std_verification.rs:22` |
| 9 | Type annotations needed | 1 | yes — same 1 E0282, `std_realization_measurement.rs:197` |

`12+16+6+5+4+3+2+1+1+1 = 51`. Every row's specimen file:line:col was re-checked against this
run's JSON, not assumed carried forward.

## 5. T7 / "99 E0308 sites" — resolved for `stern-fox-619`, restated against this base

Re-confirmed at `72676cf0be`: this module's **live T7 footprint is row 7 above — 1 site, and it
is E0599 (`partial_cmp`), not E0308.** #8410 already retired all 8 of this module's prior T7 E0308
sites (per #8460 §11.2, measured across `11254b04fc` = 8 T7 rows, `2c65eeacf3` = 1, branch head =
1). The "99 E0308 sites" figure is `docs/probes/e0308_root_partition_2026-08-18.md`'s corpus-wide,
13-mechanism-root denominator (root T7, 99/408 sites, 24.3%, across 11 entry modules) — a
different, larger population than any one module's reachable-today count. `stern-fox-619`'s task
title quoting "99 E0308 sites" is quoting that whole-corpus census, not this module: scoping the
NARROW T7 repair to `materialization_carriers` alone will dent this module's board by at most the
1 E0599 row above (and even that is `calm-lynx-547`'s row, out of scope for a table-absent-names
NARROW fix per prediction B below) — the "99 sites" headline is a corpus-wide claim, not something
this module's build can deliver on its own.

## 6. Predictions, restated against this base, before the causing PRs land

Both predictions from #8460 §11 hold verbatim; restated here against `72676cf0be` so the base is
current when each PR lands.

**Prediction A** (`silent-raven-853`, reachability-gate fix): this module's closure exercises the
generic-container checkpoint-scalar mechanism **zero times** — none of the 51 diagnostics mention
`Nat`, `Magnitude`, or `CommutativeSemiring` in message/labels/children (re-checked against this
run's JSON, not carried forward from #8460). Prediction: the fix moves this module's count by
**zero**. Any movement is composition with something else, reported as its own delta, never folded
into row totals above.

**Prediction B** (`stern-fox-619`, T7 NARROW to table-absent names): row 7 (T7, 1 site) is the only
T7-attributable row in this module. Prediction: table-present names (`Int`/`String`/`Hash`
answering from the table) leave every other row (1a/1b/2–6/8/9, 50 sites) untouched. Any movement
outside row 7 means the NARROW scope reached wider than declared.

## 7. Standing offer

Unchanged from #8460, continued: any lane sends a head SHA, this session re-runs the instrument
above against it and publishes the delta, with two-arm discipline (the fix measured on two bases)
whenever a landing might be confused with base drift. Addressed to `silent-raven-853`,
`deep-swift-570`, `stern-fox-619`, and `smart-ram-730`.
