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

## Cross-lane check (keen-ibex-435, negative — refined)

Checked against `emit_operation_method` / `emit_capability_method` in `src/v1/05_emit_rust.dag` (shape: `render_rust_type(..., emit_info: empty_emit_graph_info())` on return/wire positions). **None** of the 112 measured B1-keyword E0369 sites are in extdeps service-client code; all seven files are `std_*` (`std_measure`, `std_nat`, `std_cache_interface`, `std_realization_*`, `std_verification`).

**Signature check (bold-lark refined prediction):** if blind generic-scope / applied-binding rendering caused these E0369s, rustc would cite a carrier with `_` or an unresolved type variable where a named generic belongs. **0/112** sites show that signature — every diagnostic names fully concrete algebra carriers (`CommutativeSemiring<Magnitude>`, `Measure<…>`, including the `Rc` vs `Rc<Rc<…>>` repr-fork pair). The third “wrong type from empty `emit_info`” cause is **dead for this lane**; `repr_fork` classification stands.

## What is NOT claimed

- Fresh M=11 cargo logs were not banked in this receipt. The authoritative count here is the
  **July 7-module instance bank** (112 distinct B1-keyword sites), extended to 191 by mechanism
  argument — not a live `curated_cargo_probe_one.sh` sweep on current main.
- **CI run 32019076745 @ `42a29aa` (2026-08-17):** `ci` failed on floor job kill/timeout (~95 min),
  not witness refusal — discovery reported **9948 passed · 0 failed**; batches 1–2 green. Retry via
  new head, not `gh run rerun` (frozen merge ref).
- **CI run 32030285591 @ `fcbfd3ba` (2026-08-17):** same shape — **9948 passed · 0 failed**,
  `floor_outcome=failure` at ~13.9 GB peak, job killed at ~100 min before post-discovery batches
  logged. Not a classification or probe defect; sessionpush `pull_request` run (run count 1).
- **CI run 32039364615 @ `a1de4ade` (2026-08-17):** infra flake — `ci` job failed at **Set up job**
  (~2 min) with GitHub **429** on artifact archive download; `gunbc ci` never ran. Retry via new
  head.
- **CI run 32040389516 @ `6738550a` (2026-08-17):** same **429** at **Set up job** on **both**
  `build` and `ci` (~1 min each); artifact `upload-artifact` tarball download refused. No floor
  signal. Retry via new head after rate-limit window.
- **CI run 32040672883 @ `7cbccf41` (2026-08-17):** `build` failed at **Set up job** with GitHub
  **503** on `setup-rust-toolchain` artifact download; `ci` refused via `FloorUpstreamAlreadyRed`
  (fail-fast on upstream red — not an independent floor failure). Retry via new head.
- **CI run 32040993674 @ `e512b7e1` (2026-08-17):** `build`/regen/heal **success**; `ci` alone failed
  at **Set up job** with **503** on `upload-artifact` tarball — `gunbc ci` never ran. Retry via new
  head.
- **Instrument integrity (2026-08-17):** on main through `7cfeb6f0`, the probe scaffold called
  `gunbc run` on a `String`-returning entry — refused by the #8286 `NotProcessExit` wall. A stderr
  value-capture workaround was tried and **reverted** (it re-opened the fail-open that wall exists to
  prevent). The shell route is now `curated_probe_cargo_toml_write_from_cssl_authority` (ProcessExit +
  `Filesystem.Write`, same pattern as `e0599_write_*_blob`). Loudness fixes from #8373 remain:
  non-zero exit on `HARNESS_REFUSE`/`EMIT_REFUSE`, stale-log clearing, fresh `mktemp` log dir, paired
  `paired_rustc_errors` beside counts.
- **Reading rule:** a zero is only readable beside a nonzero from the same invocation. The July bank
  pairs 112 classified sites with per-site TSV rows (nonzero enumeration); it does not rely on a bare
  grep-over-missing-log zero.
- Nat/Int sites whose diagnostic spells only `Rc<v2_std_nat::Nat>` without an algebra
  keyword are **excluded** from the B1 filter (strict keyword match per §18) and therefore
  from the 191 — they belong to Root B3 (`Nat`/`Int` vs `{integer}`/`i64`), not this row.
