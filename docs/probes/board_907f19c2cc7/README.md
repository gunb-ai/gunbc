// Retained emission board for src/v2/compiler/03_ingest.dag at the first main SHA whose
// required floor reports failed=0. The log is kept so classification is offline analysis over
// a fixed artifact rather than a rebuild per iteration; the counts below are derived from the
// log in this directory and from nothing else.

# 03_ingest emission board — `907f19c2cc7`

## Subject and provenance

| field | value |
|---|---|
| subject SHA | `907f19c2cc7cf31d2525236c93d3c92332182cde` |
| floor at that SHA | run `32633501354` — `planned=10672 executed=10672 terminal=10672 failed=0` |
| instrument | `docs/probes/curated_cargo_probe_one.sh src/v2/compiler/03_ingest.dag ""`, `CSSL_STD_SEED_LINK=1` |
| producer | `curated_cargo_probe_one+emit+seedlink+cargo` |
| compiler binary | built from this SHA in the same dispatch (`PROV_BIN_BEFORE=0` — no cached binary) |
| retained log | `03_ingest.cargo.log`, 234163 bytes, sha256 `b39d6f428920eccb8629cc731a35c6c1555eb548b20e0d13edc65c66ab8d40fb` |
| transport of the log | gzip+base64 over the dispatch's stdout; decoded sha256 verified equal to the sha computed on the runner before transport |

## Board

```
177 files emitted, 503 diagnostics, cargo refuse
```

Coded rustc errors, by direct grep of the retained log (`^error\[E[0-9]+\]`): **316**

| code | n | code | n | code | n |
|---|---|---|---|---|---|
| E0308 | 128 | E0560 | 17 | E0282 | 6 |
| E0425 | 24 | E0061 | 17 | E0071 | 3 |
| E0599 | 23 | E0631 | 9 | E0728 | 2 |
| E0277 | 23 | E0433 | 8 | E0310 | 2 |
| E0004 | 21 | E0369 | 7 | E0533 | 1 |
| E0609 | 18 | E0614 | 6 | E0223 | 1 |

Uncoded: `unsupported_mock_expression` 13, `UNRESOLVED_CompilerError` 1. Probe histogram columns: 330 / 331.

## Why the four counts differ, and which one to quote

`503 diagnostics` is the emitter's own count and includes warnings and notes. `316` is coded
rustc errors by direct grep and is the number to quote for the board. `330`/`331` are the probe's
histogram sums, which fold the uncoded rows in and differ from each other by one. They answer
different questions; quoting one where another was measured is the unit error this corpus keeps
paying for.

## Stability against the previously certified board

The certified board at `98b18cdc` reported the same 316 and the same per-code histogram,
position for position. That identity is real rather than an artifact, and the discriminators are:

- `PROV_BIN_BEFORE=0` on this run — the compiler was rebuilt from this tree, not reused;
- the row carries this SHA, not the prior one;
- the log bytes differ across all three runs taken today — 234165 / 234279 / 234163, with three
  distinct sha256 — while the error population is identical. A run that did not execute cannot
  produce a byte-different log with the same population.

The night's merges between those SHAs were witness, fleet, roster and prose changes; none touched
the emitter, so an unchanged emission of `03_ingest` is the expected result and is now measured
rather than assumed.

## What this artifact is for

Classification transfers onto **this** log. Prior classifications carry forward only where the
load-bearing identity still joins — emitted module, enclosing declaration, callee or field
identity, error code or a declared successor code, normalized expected/found. Generated line and
column are evidence, not identity. A historical row absent from this log gets no further analysis.
