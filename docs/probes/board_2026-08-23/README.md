# Certified `03_ingest` board — retained log, ref `98b18cdc81e` (2026-08-23)

**Why this exists.** The board figures for this ref were published from the probe's TSV *row*,
and the cargo log was discarded. A row carries the histogram, so counts could be quoted — but a
row cannot be re-partitioned, so no classifier could work at this ref. `nimble-wren-909` refused
to size against a ref whose log it could not inspect, which was correct. This directory is that
refusal being answered.

## Subject, ref, provenance — measured, not asserted

| | |
|---|---|
| entry | `src/v2/compiler/03_ingest.dag` (a **path**; `--entry` is not a module path) |
| ref | `98b18cdc81ec523fa3234bac4510ec00793f3454` |
| observed HEAD | `98b18cdc81ec523fa3234bac4510ec00793f3454`, from `git rev-parse HEAD` **inside** the dispatch |
| instrument | `docs/probes/curated_cargo_probe_one.sh`, `CSSL_STD_SEED_LINK=1` |
| `shim_lib_rel` | `""` — correct **by authority** (`tools.self_host_module_behavioral_transport_roster` gives `03_ingest` a `shim_lib_rel` of `""`), not by default |
| producer | `curated_cargo_probe_one+emit+seedlink+cargo`, `EMIT_COUNT_SRC=gunbc_compiled_line` |

**Binary provenance — the failure this excludes.** A cached binary produces a *false identical*
that reads as "the fix changed nothing", with no failure arm anywhere. Excluded here by
construction rather than by argument:

```
PROV_BIN_BEFORE=0        nothing existed to reuse
PROV_OUTER_COMPILED=1    v1-compiler was compiled at this ref
```

Ref provenance and binary provenance are **two facts**. Column 9 of the probe row carries the
first; these lines carry the second.

## The board

```
177 files emitted, 503 diagnostics, 0 blocking, verdict: refuse
CARGO_ERROR_TOTAL=330   HISTOGRAM_SUM=331

E0308:128  E0425:24  E0599:23  E0277:23  E0004:21  E0609:18  E0560:17  E0061:17
E0631:9    E0433:8   E0369:7   E0614:6   E0282:6   E0071:3   E0728:2   E0310:2
E0533:1    E0223:1
uncoded_unsupported_mock_expression:13   uncoded_UNRESOLVED_CompilerError:1
```

**Coded rows = 316**, and the derivation matters because subtraction here has a *hidden term*:

| method | result |
|---|---|
| direct `grep -cE '^error\[E[0-9]+\]'` on the retained log | **316** |
| direct sum of the 18-code histogram | **316** |
| `CARGO_ERROR_TOTAL − uncoded` = 330 − 14 | **316** |
| `HISTOGRAM_SUM − 1 − uncoded` = 331 − 1 − 14 | **316** |

The last method is the one that produced a published error at another ref, because
`HISTOGRAM_SUM` counts cargo's trailing `due to N previous errors` line. **Prefer the direct
count.** `CARGO_ERROR_TOTAL` (cargo's own, block grain) and `HISTOGRAM_SUM` (coded lines) are
different instruments and are never differenced against each other.

## Units in play — four, and they are not interchangeable

| figure | unit | ref |
|---|---|---|
| 330 | `CARGO_ERROR_TOTAL`, block grain | `98b18cdc81e` |
| 331 | `HISTOGRAM_SUM`, coded lines incl. trailing summary | `98b18cdc81e` |
| 316 | coded rows | `98b18cdc81e` |
| 341 | cross-code **manifestations** (an E0308 block splits into named+elided pairs) | `967b5bc` |

They supply mechanism vocabulary. They may not be added, differenced, or ranked as shares.

## Integrity

```
03_ingest.cargo.log
  bytes  234165
  sha256 6453143f8982de281f302b25f6f96c4be1ce4629072684da5601214f691a32d6
```

The sha was emitted by the remote dispatch that produced the log and re-verified byte-for-byte
after extraction. The log is published so this population can be **re-partitioned without a
rebuild** — the property that made the `967b5bc` log usable and whose absence blocked the
cross-code classifier at this ref.

## Scope

Equivalence measured elsewhere: `98b18cdc81e` and `c07d13a49f` are byte-identical in emitted
output across all 176 files of *this entry's closure*. That does not extend to another entry
with a different closure.
