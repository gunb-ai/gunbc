# E0308 root partition — mechanism grain (2026-08-18)

Read-only partition of the dominant emitted-Rust error class. Session `sharp-owl-720`.
Supersedes the July gate-1 E0308 bucket shares in
[`gate1_repr_mismatch_e0308_diagnosis_2026-07-24.md`](gate1_repr_mismatch_e0308_diagnosis_2026-07-24.md)
for **ownership and sizing** — those buckets mixed E0308 with stale DIAGNOSTICS/WITNESS
percentages; this receipt is E0308-only at site grain.

**RE-DERIVED, not superseded (2026-08-21):** `smart-otter-254` re-derived the categories from live
pairs on `03_ingest` at `2a2bd0ad59…` — 15 categories, four of which are absent here, and **T7 (the
largest root below) has zero sites on that subject**. That run is M=1 and this one is M=11, so
neither board is a delta on the other; read
[`e0308_partition_2026-08-21.md`](e0308_partition_2026-08-21.md) before planning against the shares
below.

## Method

| field | value |
|---|---|
| date | 2026-08-18 |
| git_sha | `4e427773b78f04704dc9425a7acebdf719651da0` |
| route | `gunbc compile --source-root dag --source-root src/v2 --entry <mod> --target rust --dependency-pool-index primary-precedence` → `cssl_assemble` → `cargo build --release --lib` |
| contract | `CSSL_STD_SEED_LINK=1`, empty shim (per `curated_cargo_probe_one.sh` invocation contract) |
| entry modules | `05_emit`, `06_translate`, `04_infer`, `03_ingest`, `emit_host`, `01_tokenize`, `materialization_carriers`, `emit_module`, `03_normalize`, `program_partition`, `05_eval` (`src/v2/compiler/<name>.dag`) |
| unit of count | one distinct `(file, line, col, E0308, expected/found pair)` |
| root assignment | mechanism names from partition §11.3/§11.4, keyed on the expected/found pair |

`frontier_probe_survey` was not used.

## Headline

**E0308 is not one root.** At mechanism grain the live corpus carries **408 distinct sites**
(**1555** rustc error blocks summed over M=11 — inflation **3.81×** within the E0308 class alone).
The partition lands on **13 mechanism roots**, three of which carry most of the weight:

| root | sites | % | partition §11 cross-ref |
|---|---:|---:|---|
| **T7** — seed-prelude `Hash` collision (`Fnv1a64Structural` ↔ `String`) | 99 | 24.3% | §11.10–11.11 |
| **R1** — bare↔`Rc` wrap decision | 91 | 22.3% | R1 in §11.3 |
| **RESIDUE** — unclassified pairs | 59 | 14.5% | §11.7 tail |
| T2 — text carrier (`String` vs `Vector`/`FreeMonoid`) | 38 | 9.3% | T2 |
| T3 — collection record vs `im` carrier | 32 | 7.8% | T3 |
| B3 — modeled `Nat` vs native integer | 18 | 4.4% | B3 |
| B2 — `Bool` enum vs `bool` | 17 | 4.2% | B2 |
| RESIDUE-witness | 15 | 3.7% | July Root 2 shrunk, not zero |
| R5 — duplicate type authority | 15 | 3.7% | R5 |
| C — Optional collapses to `()` | 11 | 2.7% | C |
| B1-repr — algebra carrier repr | 6 | 1.5% | §18 repr-shaped subset |
| RESIDUE-diagnostics | 4 | 1.0% | July Root 1 dead |
| T4 — record as tuple | 3 | 0.7% | T4 |

**July falsifications confirmed at E0308-only grain:**

- DIAGNOSTICS carrier fork: **4 sites (1.0%)**, not 26–30%.
- `Witness<_>` parametrization: **15 sites (3.7%)** under `RESIDUE-witness`, not 18–23%;
  dominant pairs are `Witness<ExitOk>` vs concrete `Witness<Rc<Outcome<…>>>`.

## Per-module E0308 dominance (blocks / coded errors)

The dispatch title's **~40–47%** holds on the shared floor modules; larger closures run hotter.

| module | E0308 blocks | coded errors | E0308 share | distinct sites |
|---|---:|---:|---:|---:|
| 05_emit | 101 | 256 | 39.5% | 86 |
| 06_translate | 101 | 256 | 39.5% | 86 |
| 04_infer | 100 | 233 | 42.9% | 85 |
| emit_module | 109 | 264 | 41.3% | 94 |
| program_partition | 103 | 268 | 38.4% | 87 |
| 03_normalize | 90 | 217 | 41.5% | 75 |
| 03_ingest | 396 | 747 | 53.0% | 358 |
| emit_host | 242 | 460 | 52.6% | 216 |
| 05_eval | 211 | 377 | 56.0% | 186 |
| 01_tokenize | 74 | 104 | 71.2% | 73 |
| materialization_carriers | 28 | 58 | 48.3% | 16 |

`05_emit`, `06_translate`, and `04_infer` share the same **86/85** site set at pair grain
(byte-identical floor, consistent with §11.2). `03_ingest` and `emit_host` are thick deltas.

## Top pair signatures (corpus-wide)

```
61  expected Rc<Fnv1a64Structural>, found String          → T7
38  expected String, found Rc<Fnv1a64Structural>            → T7
19  expected Rc<Vector<_>>, found String                   → T2/T3
18  expected Rc<Nat>, found i64                            → B3
18  expected Nat, found Rc<Nat>                           → R1/B3
15  expected Rc<Nat>, found Nat                           → R1
14  expected bool, found Bool                             → B2
13  expected Coverage<Rc<…>>, found CoverageDefectAcceptanceKey → RESIDUE (Root D alias)
```

## Artifacts

- Per-site TSV (banked evidence): [`e0308_partition_2026-08-18/sites_classified.tsv`](e0308_partition_2026-08-18/sites_classified.tsv)

To repeat this measurement: use the **Method** table above — same `gunbc compile` route,
`CSSL_STD_SEED_LINK=1`, empty shim, M=11 entry set, and deduplicate E0308 diagnostics to
distinct `(file, line, col, expected/found pair)` sites before assigning mechanism roots per §11.
Raw cargo logs were scratch-only at measurement time and are not retained in tree.

## Recommendation

Plan against **mechanism roots**, not the error code:

1. **T7 (99 sites)** — checkpoint table row keyed on bare `Hash`; same fix family as §11.11
   (`rust_scalar_checkpoint_render_base` identity-keying). Highest leverage within E0308.
2. **R1 (91 sites)** — wrap-decision predicate coverage gap; pairs with §11.22 consistency sweep.
3. **RESIDUE (59 sites)** — largest entry is `Coverage<Rc<…>>` vs `CoverageDefectAcceptanceKey`
   (Root D alias-arity); chase before inventing new roots.
4. **T2/T3 (70 sites combined)** — text and collection carrier forks; separate walls.

No single construction predicate covers E0308 — the class is a **bundle of orthogonal repr forks**
already named in §11, now sized at E0308-only grain.
