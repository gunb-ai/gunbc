# Rustc mechanism board — cross-code classifier, 03_ingest (2026-08-23)

**Session:** `nimble-wren-909`. **Work item:** `node://adhoc-11d59365-bc4`.

This is a measurement over the published 2026-08-22 `03_ingest` log, not a compiler repair. It
corrects the unit of the earlier E0308 partition: a rustc code is an observed manifestation, not a
mechanism boundary. The same absent-bound producer reaches E0308, E0277, and E0599, so neither a
population nor a share computed inside E0308 is a population or share of that mechanism.

## Subject and instrument

The subject, ref, producer, raw log, and E0308 canonicalization contract are unchanged from
[`e0308_partition_2026-08-22.md`](e0308_partition_2026-08-22.md). The new committed classifier is
[`rustc_mechanism_classify.py`](rustc_mechanism_classify.py); its complete output is
[`mechanisms_classified.tsv`](e0308_partition_2026-08-22/mechanisms_classified.tsv).

The common unit is a **coded diagnostic manifestation**:

- E0308 retains the earlier classifier's canonical expansion and global deduplication: 128 rustc
  blocks become 154 manifestations.
- Every other coded rustc block contributes one manifestation.
- The resulting board has **341 manifestations**: 154 E0308 + 187 across the other codes. It is
  not cargo's 329-error total and must not be compared to that block-grain counter.

The classifier recognizes only the one cross-code mechanism established here. Its other 319 rows
are `UNCLASSIFIED`, not code-derived guesses. The former E0308 label is carried separately as
`e0308_candidate_projection`; it reproduces that view without promoting it to root identity.

## Established cross-code mechanism

| mechanism | E0308 | E0277 | E0599 | population | board share |
|---|---:|---:|---:|---:|---:|
| `ABSENT_CLONE_BOUND` | 12 | 6 | 4 | **22** | **6.5%** of 341 manifestations |

The discriminator is not the word `clone` alone:

- E0308 requires the existing `A-clone` candidate plus rustc's explanation that the type parameter
  does not implement `Clone`, so its reference was cloned instead.
- E0277 requires `the trait bound <parameter>: Clone is not satisfied`.
- E0599 requires `no method named clone found for type parameter <parameter>`.

This excludes unrelated calls to `.clone()`, derive failures for non-Clone traits, and ordinary
representation mismatches whose source expressions happen to clone values.

## Consequence for the E0308 publication

The earlier table's `A-clone = 12` is a correct **E0308 projection** and an unsound mechanism
population. Its `7.8%` is a correct share of the 154 E0308 manifestations and an unsound mechanism
share. The mechanism population is 22 on this board; its share is 6.5% of the cross-code
manifestation denominator.

The same limitation applies to every other E0308 candidate until it is classified across codes.
Their rows remain useful as code-local projections, but the prior staffing conclusions based on
their relative shares are withdrawn. No cross-code population is inferred for them here.

To repeat without rebuilding:

```sh
python3 docs/probes/rustc_mechanism_classify.py \
  docs/probes/e0308_partition_2026-08-22/03_ingest.cargo.log /tmp/mechanisms.tsv
```
