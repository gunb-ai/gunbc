# E0308 re-baseline — Route-A last mile (faithful emitted seed)

Re-baselines the type-mismatch (`E0308`) gap between the **faithful full regen** of the
stage0 seed and `cargo`-green. The carrier mark in `regen_stage0.rs` estimated *"~150
latent emitter-completeness gaps"* from wiring the std-tower modules; this measures the
real number and **categorizes it by variety** so the deferred regen-fixpoint lane
(`node://adhoc-80af9ff8-40f`) can be scoped from data, not an estimate.

## What "the REAL crate" is

The committed stage0 seed is a **hand-synced mirror**, not a regen fixpoint: it
deliberately leaves several std-tower modules **unwired** in `lib.rs`
(`std_measure` / `std_integer` / `std_machine_constraints` /
`std_realization_schedule` / `extdeps_version_semver` / `extdeps_cargo_version`). A
*faithful* full regen — emitting from the `.dag` authorities and wiring **every**
emitted module — is what surfaces the latent gaps.

The REAL crate = the **emitted seed closure** (108 modules from `compile_stage0`
over `src/v1` + `dsl`) **+ the 22 hand-written periphery modules** copied verbatim
(`v1_interpreter` / `cli_run` / `v1_rt` / lens-kernel project modules / `wire_value_serialize`
/ `v1_compiler_dag_collect{,_support}`).

## Reproduce

A non-destructive harness mode assembles that crate without touching the committed
seed (`src/v1/stage0/src/bin/regen_stage0.rs`):

```
cargo build -p v1-compiler --bin regen_stage0
./target/debug/regen_stage0 --emit-fresh <dir>          # ~20 min: full v1-seed self-compile
cd <dir>
RUSTC_WRAPPER="" cargo build --message-format=json --lib > build.json 2>&1

# error-code histogram (total + E0308 share):
grep -oE '"code":"E[0-9]{4}"' build.json | sort | uniq -c | sort -rn

# E0308 categorized by expected -> found type pair (one pair per diagnostic; sums to 38).
# `scan(...)[0]` takes the first occurrence so the per-diagnostic note doesn't double-count;
# the lone "<pair-on-note>" is a HashMap<&str> case whose label wraps across lines (family A).
jq -r 'select(.reason=="compiler-message") | .message | select(.code.code=="E0308")
        | ([.rendered | scan("expected `[^`]+`, found `[^`]+`")][0]) // "<pair-on-note>"' build.json \
  | sort | uniq -c | sort -rn
```

`--emit-fresh` runs the same assembly phases as a normal regen (emit closure → copy
periphery → patches → rustfmt) into a caller-named dir and **stops** — no copy-back, no
temp cleanup. The committed seed is never mutated. (`RUSTC_WRAPPER=""` bypasses the
`sccache` wrapper, which is unrelated infra and flaked on a dep compile during this
measurement.)

## Baseline (faithful emitted seed @ this branch tip)

`cargo build --lib` on the assembled crate: **73 total errors**, of which **38 are
`E0308`**. (The "~150" estimate was stale by ~4×.)

### E0308 by root cause — 2 emitter causes cover 89%

| n | family | root cause | fix locus |
|---|--------|-----------|-----------|
| **24** (+1 multi-line-label variant = **25**) | **A — map-literal `&str` key** | emitter writes map **keys** as bare `&str` literals (`__m.insert("true", …)`) without `.to_string()`; the values *do* get `.to_string()`, so the map infers `HashMap<&str, V>` against the declared `HashMap<String, V>` | map-literal emission (one fix → −25 E0308, 66%) |
| **10** | **B — Box/Rc/Option wrap** | auto-wrap/unwrap coercion not inserted: `Box<String>`↔`String` (5+2), `Rc<X>` vs `Option<Rc<X>>` missing `Some()` (3) | coercion-insertion at call/construct sites |
| **3** | **C — scalar `&str`/`String`** | string literal not `.to_string()` in scalar position | same coercion path as A, scalar arm |

The dominant lever is **family A** — a single emitter change (coerce map-literal keys to
`String`) clears ~two-thirds of all `E0308`. Worst files: `v1_compiler_stage0_crates.rs`
(6), `std_types.rs` (5), `extdeps_languages_dag_syntax.rs` (4).

### Raw categorizer output (expected → found, top pairs)

```
 10  HashMap<String, String>        => HashMap<&str, String>
  7  HashMap<String, bool>          => HashMap<&str, bool>
  5  Box<String>                    => String
  3  &str                           => String
  2  HashMap<String, Rc<LiteralValue>> => HashMap<&str, Rc<LiteralValue>>
  3  Rc<X>                          => Option<Rc<X>>          (SemVerIdentifier/ScheduleWitnessEntry/Runnable)
  1  i64                            => Box<i64>
  1  String                         => Box<String>
  + 5 further HashMap<String,V> => HashMap<&str,V> singletons (AlgebraProfile, TokenShape, NodeFieldRole, FunctionSizeEffect, i64)
```

## Non-E0308 context (the other 35 errors) — distinct classes, separate owners

These do **not** count as E0308 but block green, and two classes **mask** an unknown
number of additional E0308 (see caveat):

| n | code | class | where |
|---|------|-------|-------|
| 12 | E0560 | **measure-tower field emission** — `Rc<Measure<Q,U,i64>> has no field count` | all `std_measure.rs` |
| 8 | E0433 | **unwired periphery** — `inert_carrier_project` / `non_fold_residue_project` not declared in emitted `lib.rs` (regen patch-list is stale vs the 3 newest `HAND_MAINTAINED` modules) | `v1_interpreter.rs` |
| 5 | E0425 | unresolved fn/type (`v1_rt::utf8_decode_bytes`, `SemVerConstraint`) | `extdeps_cargo.rs`, `v1_interpreter.rs`, `v1_probe_emit_interp.rs` |
| 2 | E0573 | `Time` variant used as type | `std_realization_schedule.rs` |
| 2 | E0599 | `clone` on unconstrained type param | `std_measure.rs`, `std_types.rs` |
| 2 | E0091 | unused type param (`Algebra` / `MachineConstraint`) | `std_machine_constraints.rs` |
| 1 each | E0277 / E0252 / E0107 / E0063 | std-tower generic-emission residue | `std_types` / `std_realization_schedule` / `std_termination` |

### Honesty caveat (masking)

The **38 is a lower bound**. `E0433` (8, in `v1_interpreter`) and `E0425` (5) are
*resolution* failures — `rustc` aborts type-checking in those regions, so any `E0308`
they would surface is hidden today. The map-key (A) and Box/Rc (B) families are
independent of these regions and are the trustworthy, actionable core. One confound was
already removed for this measurement: the emitted `Cargo.toml` omitted `im-rc` (used by
the hand-written `v1_interpreter`), which produced 2 spurious `E0432`; the harness now
adds it (mirrors the committed stage0 `Cargo.toml`).

## Implication for the regen-fixpoint lane

`E0308`-to-green is **not** 150 independent gaps — it is **two emitter root causes**
(map-literal key coercion; Box/Rc/Option auto-wrap) covering 34/38, plus a small scalar
arm. The remaining green work splits cleanly into separately-owned classes: the
**measure-tower** generic emission (12 `E0560` + the `std_measure`/`std_machine_constraints`
residue) and **regen-harness wiring** (the 8 `E0433`). This re-baseline is the input the
carrier-mark dissolution trigger was waiting on.
