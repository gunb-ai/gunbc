# E0308 mechanism partition, re-derived (mechanism grain, M=1)

| field | value |
|---|---|
| git_sha | `2a2bd0ad59cdc4d37f0ef35a72232bac57c9bbe7` |
| entry | `src/v2/compiler/03_ingest.dag` (M=1) |
| producer | `curated_cargo_probe_one+emit+seedlink+cargo`, `CSSL_STD_SEED_LINK=1`, shim `""` (roster) |
| E0308 blocks | **199** (39.1% of `CARGO_ERROR_TOTAL=509`) |
| distinct sites | **235** (blocks < sites: one arg-mismatch block carries one note per wrong argument) |
| unclassified | 7 (3.0%), printed; RESIDUE arm known-positive |

## Roots (site grain, this subject only)

| root | sites | % |
|---|---:|---:|
| B3 modeled `Nat` vs native integer | 49 | 20.9% |
| T2 text carrier vs `String` | 34 | 14.5% |
| R1 bare↔`Rc` wrap (22 outer + 11 type-argument depth) | 33 | 14.0% |
| T3 collection carrier fork | 25 | 10.6% |
| RT-builtin host-builtin signature interception (NEW) | 20 | 8.5% |
| D alias arity / generic argument count | 13 | 5.5% |
| ARG-ORDER call argument order (NEW) | 11 | 4.7% |
| R2 Optional surface fork | 9 | 3.8% |
| W `Witness<_>` type argument | 8 | 3.4% |
| RESIDUE | 7 | 3.0% |
| A-clone generic `Clone` bound absent (NEW at E0308 grain) | 6 | 2.6% |
| B2 `Bool` vs `bool`/variant | 6 | 2.6% |
| R5 duplicate type authority | 6 | 2.6% |
| C carrier collapses to `()` | 4 | 1.7% |
| DIAG diagnostic carrier fork | 4 | 1.7% |

**T7 (`Fnv1a64Structural` ↔ `String`), the 2026-08-18 board's largest root at M=11: zero sites
here.** No attribution offered — different subject, different M (§16).

## Top pair signatures

- 28× `expected Rc<Nat>, found integer`
- 19× `expected Rc<Vector<_>>, found String`
- 18× `expected Rc<Nat>, found i64`
- 13× `expected Coverage<Rc<...>>, found CoverageDefectAcceptanceKey`
- 8× `expected OrdSet<String>, found Rc<PointwisePower<_>>`
- 7× `expected String, found Rc<Vector<_>>`
- 6× `arguments to this function are incorrect: reorder`
- 5× `expected OccurrenceId, found Rc<NodeOccurrenceId>`

## Concentration

`v2_compiler_tokenize.rs` alone carries **68 of 235 (28.9%)** — B3 36, T2 32 — across 47 files.

Receipt: [`e0308_partition_2026-08-21.md`](../e0308_partition_2026-08-21.md).
