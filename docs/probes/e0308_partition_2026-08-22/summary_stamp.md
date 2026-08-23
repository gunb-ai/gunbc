# E0308 repartition on current main (mechanism grain, M=1)

**CORRECTED 2026-08-23:** this is an E0308 projection, not mechanism grain. `A-clone = 12` reaches
E0277 and E0599 too and has cross-code population 22. All percentages below are code-local shares,
not mechanism shares; the cost/staffing inference is withdrawn. See
[`rustc_mechanism_partition_2026-08-23.md`](../rustc_mechanism_partition_2026-08-23.md). That
population is at this stamp's `967b5bc1b92` ref, not the current certified `98b18cdc81e` board.

## Completeness standing (reporting convention adopted 2026-08-22)

```
VISIBLE CANARY        coded rustc rows: 315 | subject: 03_ingest closure (M=1) | ref: 967b5bc1b92
                      canonical E0308 sites: 154
DIAGNOSTIC COVERAGE   standing: PARTIAL
  known censor        LexMatchThunk.apply / generic-instantiation failure (E0599 aborts before
                      inference reaches the expressions behind it)
  historical masked   ~68 canonical tokenize sites (2026-08-21 board, different ref)
  current successor   UNMEASURED until the censor is removed
```

A bare row count with no coverage standing overstates what is known. **Do not add 68 to any board
total** — different units (canonical sites vs coded rustc rows) and different tree states. When the
censor is removed the board will RISE, and that is diagnostic completion, not regression.

| field | value |
|---|---|
| git_sha | `967b5bc1b92ee66250e06a7870c132b48a16b80a` (requested `967b5bc1b92`; echoed from inside the remote dispatch and pinned by `PROBE_EXPECT_BASE_SHA`) |
| entry | `src/v2/compiler/03_ingest.dag` (M=1) |
| producer | `curated_cargo_probe_one+emit+seedlink+cargo`, `CSSL_STD_SEED_LINK=1`, shim `""` |
| emitted roster | 177 files, 503 emit diagnostics (same roster count as the 2026-08-21 run — same subject) |
| raw E0308 blocks | **128** (40.6% of **315** coded rows; `CARGO_ERROR_TOTAL=329`, `HISTOGRAM_SUM=330`) |
| canonical sites | **154** |
| clusters | 14 + residue |
| unclassified residue | **7 (4.5%)**, printed in full; residue arm known-positive |
| classifier | `docs/probes/e0308_classify_sites.py` (committed; re-runnable over the published raw log) |

### Coded-row count corrected 316 → 315 (2026-08-22), and one derived percentage moved

The stamp and the receipt both reported **316** coded rustc rows. The published raw log yields
**315**, which is also what this board's own name in the receipt's series table already said
(`315-board`) — so the receipt disagreed with its own series counter, and the log settles it.

The arithmetic that produced the wrong number, stated so it is not re-derived: `HISTOGRAM_SUM=330`
counts `^error(\[E[0-9]+\])?:` lines (`curated_cargo_probe_one.sh` `HISTOGRAM_SUM`), and cargo's
own trailing `error: could not compile ... due to 329 previous errors` line matches that pattern.
So 330 = 329 real error blocks + 1 summary line. Subtracting only the 14 uncoded rows
(`uncoded_unsupported_mock_expression:13` + `uncoded_UNRESOLVED_CompilerError:1`) from 330 gives
316; the summary line must come off first. 330 − 1 − 14 = **315**, and the per-code histogram sums
to 315 independently (`grep -o '^error\[E[0-9]*\]' | sort | uniq -c`), so the two agree.
`CARGO_ERROR_TOTAL=329` = 315 coded + 14 uncoded is the third agreeing reading.

**Derived percentages that moved:** exactly one. The E0308 share of the coded board is
128 / 315 = **40.6%**, not 128 / 316 = 40.5%. Every other percentage in this stamp and in the
receipt is denominated in the **154 canonical sites**, not in coded rows, so no cluster share, the
95.5% classified figure, or the 4.5% residue figure changes. The 128, the 154, the cluster counts,
the histogram and every disposition are unaffected — this was a denominator arithmetic slip, not a
measurement error.

## Clusters (site grain, this subject only) — every one a CANDIDATE root

Repartitioned 2026-08-22 after a keying defect found by review; R1's four delta values are published as four, because a two-bucket rollup erases the element arm and element depth is a separate producer root; precedence between delta-keyed,
carrier-keyed and context-keyed arms is now declared in the classifier and in the receipt's
*keying ruling*, not left to source order.

| cluster | sites | % |
|---|---:|---:|
| R1 bare↔`Rc` wrap (17 outer / 10 type-argument / 5 element / 2 outer-of-container) | 34 | 22.1% |
| R2 Optional surface fork | 24 | 15.6% |
| D alias arity / generic argument count | 16 | 10.4% |
| T3 collection carrier fork | 14 | 9.1% |
| A-clone generic `Clone` bound absent | 12 | 7.8% |
| ARG-ORDER call argument order | 11 | 7.1% |
| B3 modeled `Nat` vs native integer | 10 | 6.5% |
| RESIDUE | 7 | 4.5% |
| C carrier collapses to `()` | 6 | 3.9% |
| B2 `Bool` vs `bool`/variant | 6 | 3.9% |
| ELEM-COLL element vs its own collection (**NEW**) | 5 | 3.2% |
| W `Witness<_>` type argument | 5 | 3.2% |
| DIAG diagnostic carrier fork | 3 | 1.9% |
| BOX-WRAP `Box` wrap decision (**NEW**) | 1 | 0.6% |

## Carrier flags (beside the root, not a category)

Six sites sit on the `Measure` carrier whose stage0 alias-emission collapse is already documented
with a dissolve-on at `std.measure` `billing_month_as_hour_count_representation_note`, split R1 4 /
D 2 / C 2 by their deltas. They are joinable by the `carrier_flags` column rather than pooled into
an arm: at `std_cache_interface.rs:652`/`:667` the collapse is on BOTH sides, so it is invisible in
the mismatch and no pair-keyed rule can recover it.

## Prior-root dispositions

- **StillLive:** R1, R2, T3, D, A-clone, ARG-ORDER, B3, W, B2, C, DIAG (D, ARG-ORDER and B2 are
  count-for-count identical, same file, same pairs).
- **Removed:** R5 (`OccurrenceId`/`NodeOccurrenceId`) — zero occurrences anywhere in the TSV.
- **Converted out of E0308:** RT-builtin — `v1_rt::lookup` went generic in gunbc#8792; the callee
  now shows up in 17 `E0061` blocks instead.
- **Unjoinable (masked, NOT closed):** T2 (34→0) and most of B3 (49→10). Their file,
  `v2_compiler_tokenize.rs`, now fails at `E0599` before inference reaches those expressions.
- **New:** ELEM-COLL (4), BOX-WRAP (1).

## Cost shape inside E0308 only — staffing inference withdrawn

Four candidates cover 57% of E0308 manifestations; six more are single- or two-file and cover another 49. Tail = 7
residue sites in 5 files. This is a small number of producer decisions with wide fanout, not a site-by-site
tail inside this code. It does not rank cross-code mechanisms.

## The 182 in the commissioning brief

Traced to the **465-board** at `4ce177491202217a1a1520b413a62b7c7dfe9f71` and withdrawn by its
author, along with the accompanying 465-era histogram. Its retained series — 465-board 182,
399-board 174, 339-board 135 — is monotone and this run's 128 is its next point. Nothing here is
differenced against it.

Receipt: [`e0308_partition_2026-08-22.md`](../e0308_partition_2026-08-22.md).
