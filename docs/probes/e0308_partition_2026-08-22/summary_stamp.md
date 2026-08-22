# E0308 repartition on current main (mechanism grain, M=1)

| field | value |
|---|---|
| git_sha | `967b5bc1b92ee66250e06a7870c132b48a16b80a` (requested `967b5bc1b92`; echoed from inside the remote dispatch and pinned by `PROBE_EXPECT_BASE_SHA`) |
| entry | `src/v2/compiler/03_ingest.dag` (M=1) |
| producer | `curated_cargo_probe_one+emit+seedlink+cargo`, `CSSL_STD_SEED_LINK=1`, shim `""` |
| emitted roster | 177 files, 503 emit diagnostics (same roster count as the 2026-08-21 run — same subject) |
| raw E0308 blocks | **128** (40.5% of 316 coded rows; `CARGO_ERROR_TOTAL=329`, `HISTOGRAM_SUM=330`) |
| canonical sites | **154** |
| clusters | 14 + residue |
| unclassified residue | **7 (4.5%)**, printed in full; residue arm known-positive |
| classifier | `docs/probes/e0308_classify_sites.py` (committed; re-runnable over the published raw log) |

## Clusters (site grain, this subject only) — every one a CANDIDATE root

Repartitioned 2026-08-22 after a keying defect found by review; precedence between delta-keyed,
carrier-keyed and context-keyed arms is now declared in the classifier and in the receipt's
*keying ruling*, not left to source order.

| cluster | sites | % |
|---|---:|---:|
| R1 bare↔`Rc` wrap (19 outer / 15 type-argument depth) | 34 | 22.1% |
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

## Prior-root dispositions

- **StillLive:** R1, R2, T3, D, A-clone, ARG-ORDER, B3, W, B2, C, DIAG (D, ARG-ORDER and B2 are
  count-for-count identical, same file, same pairs).
- **Removed:** R5 (`OccurrenceId`/`NodeOccurrenceId`) — zero occurrences anywhere in the TSV.
- **Converted out of E0308:** RT-builtin — `v1_rt::lookup` went generic in gunbc#8792; the callee
  now shows up in 17 `E0061` blocks instead.
- **Unjoinable (masked, NOT closed):** T2 (34→0) and most of B3 (49→10). Their file,
  `v2_compiler_tokenize.rs`, now fails at `E0599` before inference reaches those expressions.
- **New:** ELEM-COLL (4), BOX-WRAP (1).

## Cost shape

Four clusters cover 57% of sites; six more are single- or two-file and cover another 49. Tail = 7
residue sites in 5 files. This is a small number of producer decisions with wide fanout, not a site-by-site
tail — the same shape the 2026-08-21 board reported on a different population.

## The 182 in the commissioning brief

Traced to the **465-board** at `4ce177491202217a1a1520b413a62b7c7dfe9f71` and withdrawn by its
author, along with the accompanying 465-era histogram. Its retained series — 465-board 182,
399-board 174, 339-board 135 — is monotone and this run's 128 is its next point. Nothing here is
differenced against it.

Receipt: [`e0308_partition_2026-08-22.md`](../e0308_partition_2026-08-22.md).
