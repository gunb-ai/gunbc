# E0369 B1 operator-on-carrier classification (2026-08-17)

Session `lively-ibex-709`, Root B2. Classifies the 191 E0369 rows left
unclassified in `docs/plans/self-host-cargo-refusal-root-partition.md` §18.

## Answer

| classification | partition M=11 (§18) | July 7-module bank (measured) |
|---|---:|---:|
| **repr_fork** | **191** (by mechanism; see below) | **112** distinct sites |
| **missing_trait_impl** | **0** | **0** |

**Every B1-keyword E0369 site is a representation fork, not a missing trait impl.**

The ambiguity §18 names is real for *unfiltered* E0369 — `e0369_census_2026-07-26`
documents 116 R1 sites (PartialEq derive over `im::Vector`, `dyn Fn`, interpreters)
that are **missing_trait_impl** — but those signatures do **not** contain
`CommutativeSemiring` / `Magnitude` / `Measure<` / `Semiring`, so they are **outside**
the B1 bucket and outside the 191.

Within the B1 filter, every site is an operator on the modeled algebra carrier emitted
under `FaithfulFreeMonoid` where host-native grounding would make the operator valid
without adding trait impls to `CommutativeSemiring<Magnitude>`.

## Instrument

| field | value |
|---|---|
| classifier | `docs/probes/e0369_b1_operator_classify.py` |
| July bank input | `docs/probes/e0369_census_2026-07-26/shapes/*.instances.tsv` |
| July git_sha | `fd5e321952fbfb187b3d97411da360315f39955b` |
| output TSV | `docs/probes/e0369_b1_classification_2026-08-17/sites_classified_july_bank.tsv` |
| M=11 repro | `docs/probes/run_e0369_b1_classification.sh` (curated_cargo_probe_one.sh route) |

### Per-site decision rules (applied in order)

1. **`#[derive(..., PartialEq, ...)]` line rustc cites** — inspect operand type in diagnostic:
   - `CommutativeSemiring` / `Measure<…>` → **repr_fork** (derive requests `PartialEq` on a record that should not be emitted as algebra stub)
   - (`im::Vector`, `dyn Fn`, `*Interpreter`, `EffectIoEvalBundle` would be **missing_trait_impl** — none appear in B1-filtered population)
2. **Body binop** on `CommutativeSemiring` / `Measure<…>` / `Semiring` → **repr_fork**
3. No site reached an unmatched arm.

### Falsifier (repr_fork class)

`docs/probes/root_b_primitive_repr_fork_2026-08-16.md` §6.2: forcing `HostNative` on
`06_translate` eliminates algebra-carrier errors **74 distinct sites → 0**. R1
missing_trait_impl sites (`dyn Fn ==`, `im::Vector` derive) are a disjoint population.

## By reason (July bank, 112 sites)

| reason | sites |
|---|---:|
| expr_binop: arithmetic/compare on modeled algebra carrier (FaithfulFreeMonoid) | 82 |
| derive_expansion: PartialEq on algebra-carrier record under FaithfulFreeMonoid | 30 |

## Count reconciliation (112 vs 191)

| factor | note |
|---|---|
| July census modules | 7 (`05_emit`, `06_translate`, `04_infer`, `emit_host`, `05_eval`, `emit_module`, `materialization_carriers`) |
| M=11 modules (partition §11.14) | adds `03_ingest`, `01_tokenize`, `03_normalize`, `program_partition` |
| July distinct B1 E0369 sites | **112** |
| Partition §18 M=11 B1 E0369 sites | **191** |
| delta | **79** — predominantly from the four modules absent in the July instance bank; §11.14 measures the corpus as near-saturated (9 new distinct sites across four added modules), so the E0369 delta is closure-wide repetition of the same floor sites plus `03_ingest`'s thick delta |

**Mechanism classification is uniform across both counts** — the 79 unbanked sites are
not a second mechanism; they are the same algebra-carrier repr fork seen through a wider
entry closure. Fresh M=11 re-measurement is scripted in `run_e0369_b1_classification.sh`.

## What is NOT claimed

- Fresh M=11 cargo logs were not banked in this receipt (the full M=11 loop is scripted in
  `run_e0369_b1_classification.sh`; `render_cssl_probe_lib_cargo_toml.sh` now reads the
  `NotProcessExit` return from stderr, but no M=11 run was executed for this commit). The
  classification is executed on the July instance bank and extended to 191 by mechanism
  argument above.
- Nat/Int sites whose diagnostic spells only `Rc<v2_std_nat::Nat>` without an algebra
  keyword are **excluded** from the B1 filter (strict keyword match per §18) and therefore
  from the 191 — they belong to Root B3 (`Nat`/`Int` vs `{integer}`/`i64`), not this row.
