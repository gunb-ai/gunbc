# Parse-Population Baseline Workloads

This document defines the two fixed benchmark populations for the
`emission-real-consumer-profiles` row.

Pinned SHA: `65230868362de3efc0ddfeeaeef5d83eeb2fa385` (origin/main at time of writing)

The row's brief references `56d460116d6252f6ec8dfe889ea358c804685861`; the top-170
selection is verified identical at both shas (`diff` empty).

## Selection rule

**Live roots.** The parse-population verb in CI walks
`--source-root dag --source-root src/v2` (witnesses.yml, the
"Walk parse-population reusing the nearest published ancestor's receipts"
step). These are the two live roots; `src/v1/` is the seed compiler's
hand-written source and is **out of scope** — the row profiles the
emitted v2 binary, not the v1 seed.

**Population universe.** All files matching `*.dag` under `dag/` and
`src/v2/` in the git tree at a pinned SHA.  At the pinned SHA this is
10,562 files (8,704 under `dag/`, 1,858 under `src/v2/`).

**Ranking.** Sort the population by **descending byte size** (blob size
from `git cat-file -s`). Ties broken by lexicographic path ascending for
determinism.

**Exclusions.** None beyond the root restriction.  Every `.dag` file in
the two roots is eligible.

**Population 1 — Five Largest.** Positions 1–5 in the sorted list.
These are the five largest inputs the parser must handle.  All five are
data-shaped or generated files (roadmap rows, receipt markings, Wikipedia
hardware tables, a generated roster).  This population measures the parser
on the largest inputs that exist, which is what the row asked for.  They
are not chosen for algorithmic interest.

**Population 2 — 165-Module Sample.** Positions 6–170 in the sorted list
(disjoint from Population 1).  This is the next 165 modules by descending
size, giving a sample that covers the upper size distribution while
remaining complementary to Population 1.  It contains a mix of code
(`src/v2/compiler/`, `src/v2/std/`) and data (`dag/`) modules; the mix is
the natural consequence of size ordering, not a selection for interest.

**Combined coverage.** 170 of 10,562 modules (1.6%).
Population 1: 5 modules, 4,575,463 bytes (avg 915,092).
Population 2: 165 modules, 29,960,631 bytes (avg 181,579).
Total: 34,536,094 bytes.  Combined: 139 from `dag/`, 31 from `src/v2/`.

## How to reproduce

```bash
SHA=65230868362de3efc0ddfeeaeef5d83eeb2fa385
git ls-tree -r --name-only "$SHA" -- dag/ src/v2/ \
  | grep '\.dag$' \
  | while read f; do
      echo "$(git cat-file -s "$SHA:$f") $f"
    done \
  | sort -t' ' -k1,1rn -k2,2 > all_dag_sorted.txt

# Population 1: lines 1-5
head -5 all_dag_sorted.txt

# Population 2: lines 6-170
sed -n '6,170p' all_dag_sorted.txt
```

## Population 1: Five Largest (positions 1–5)

| # | Bytes | Path |
|---|-------|------|
| 1 | 1421761 | `dag/gunbc/roadmap/roadmap_authority.dag` |
| 2 | 947773 | `dag/gunbc/roadmap/roadmap_belt_actuate.dag` |
| 3 | 750634 | `dag/product/p4261/e1_receipt_markings.dag` |
| 4 | 748625 | `dag/extdeps/wikipedia/list_of_nvidia_graphics_processing_units.dag` |
| 5 | 706670 | `src/v2/workflow/floor_grandfathered_roster.dag` |

## Population 2: 165-Module Sample (positions 6–170)

The 165-row roster is **not transcribed into this file** — a hand copy of command
output can go stale while still looking authoritative. It is regenerated
deterministically by the reproduce command above (`sed -n '6,170p'
all_dag_sorted.txt`), which is the authority for the exact membership. The
aggregate totals are printed here so the population is checkable without the
roster:

- 165 modules, 29,960,631 bytes (avg 181,579).
- Roots: 135 from `dag/`, 30 from `src/v2/`.
- Spot-check anchors (regenerate and confirm these match): position 6 =
  `src/v2/compiler/body_lowering_fold.dag` (662,404 B); position 170 =
  `src/v2/extdeps/languages/swift.dag` (92,576 B).

## Notes

- **All five of Population 1 are data-shaped.** Four are tables or
  rosters generated from external data (Wikipedia GPU lists, product
  receipt markings) and one is a generated floor roster.  They measure
  the parser under maximum input pressure, not under algorithmic variety.
  This is what the row's first_slice specified ("the 5 largest dag
  modules").
- Population 2's mix (135 dag/ + 30 src/v2/) arises from the size ordering
  and is not curated.  It includes compiler-internal code modules
  (`body_lowering_fold`, `infer`, `source_authority`) alongside data
  modules, which gives better algorithmic coverage than Population 1.
- The 165-module count matches neither fleet shard counts nor any CI
  parameter.  It is the row's own declaration; this document makes it
  reproducible.
