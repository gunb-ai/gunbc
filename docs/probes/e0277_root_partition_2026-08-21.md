# E0277 root partition — trait × self-type grain (2026-08-21)

Read-only partition of the second-largest emitted-Rust error class. Session `bright-moth-92`,
working the E0277 lane of the self-host cargo-refusal root partition
([`../plans/self-host-cargo-refusal-root-partition.md`](../plans/self-host-cargo-refusal-root-partition.md)).
Supersedes [`e0277_trait_bound_census_2026-07-26.md`](e0277_trait_bound_census_2026-07-26.md) for
**sizing and ownership** — that census counted occurrences, not sites, and its dominant family is
no longer dominant.

**Every number here was measured today, in one dispatch, at one checkout.** Nothing is transcribed
from the July TSVs or from the 2026-08-19 pinned census; where either is quoted it is labelled as
superseded.

## Method

| field | value |
|---|---|
| date | 2026-08-21 |
| git_sha | `bb21f8563849b01cce9c978e4a1d9b170058c418` |
| route | `gunbc compile --source-root dag --source-root src/v2 --entry <mod> --target rust --dependency-pool-index primary-precedence` -> `cssl_assemble` -> `cargo build --release --lib` |
| instrument | `docs/probes/curated_cargo_probe_one.sh` (working agreement 2) |
| contract | `CSSL_STD_SEED_LINK=1`, no lane shim, `PROBE_KEEP_LOG_DIR` |
| entry modules | `05_emit`, `03_ingest`, `emit_host`, `05_eval`, `01_tokenize`, `materialization_carriers` (`src/v2/compiler/<name>.dag`) |
| unit of count | one distinct `(file, line, col)` in the emitted crate |
| key | `(unsatisfied trait, self type)` from the rustc message |

`frontier_probe_survey` was not used.

**One checkout, by construction — and this replaced a failed attempt, recorded because the failure
is cheap to repeat.** The first attempt ran three parallel dispatches pinned with
`PROBE_EXPECT_BASE_SHA` taken from an earlier dispatch's resolved HEAD. All three refused
(`SAME_BASE_REFUSE`) after paying for the remote build, because main moved twice in the interval.
`ctrl-build --remote` resolves the repo-root HEAD *when the run starts*, so a cross-dispatch pin on
a repository this active is a coin flip. The measurement above is a single dispatch that captures
`HEAD` inside itself and exports it as the pin, which makes "all six modules at one tree" a property
of the run rather than a hope. The pin behaved exactly as its contract says it should: it stopped
the line rather than producing six numbers from three trees.

## Headline

**E0277 is five root labels over four mechanisms, at 82 distinct sites** (365 rustc error blocks summed over M=6 —
**4.45x** inflation within the E0277 class alone), with **zero unclassified**.

| root | sites | % of E0277 sites | owner |
|---|---:|---:|---|
| **T5b** — serde/Debug demanded over closure-bearing values | 35 | 42.7% | UNOWNED (dispatched, see below) |
| **A** — generic parameter bound not emitted | 30 | 36.6% | Root A lane (`smart-ram-730`) |
| **R3** — `Rc<dyn Fn..>` where an `Fn` bound is expected | 9 | 11.0% | UNOWNED (dispatched) |
| **T7** — map-key derives (`Hash`/`Eq`) missing on `Fnv1a64Structural` | 7 | 8.5% | blocked in tree, see below |
| **T5a** — map-key derive (`Eq`) missing on `OccurrenceId` | 1 | 1.2% | same, see below |

Per-site rows: [`e0277_partition_2026-08-21/sites_classified.tsv`](e0277_partition_2026-08-21/sites_classified.tsv).
Stamp: [`e0277_partition_2026-08-21/summary_stamp.md`](e0277_partition_2026-08-21/summary_stamp.md).

## The July census's central claim is falsified at site grain

`e0277_trait_bound_census_2026-07-26.md` reported three families and ranked the generic-`Clone`
family as "the dominant family", to be root-caused first because it was believed shared with E0599.
Live, at distinct-site grain:

- the generic-parameter family (root **A**) is **30 of 82 (36.6%)** — real, second, not dominant;
- the derive family, which July split across its family 2 (`Hash`/`Eq` on named carriers) and
  family 3 (serde/Debug on interpreter structs), is **43 of 82 (52.4%)** once both halves are
  counted — and it is **not one root**, because 35 of those 43 cannot be fixed by adding a derive
  at all (below);
- July's family 2 self types (`Node`, `EnvironmentBindingKey`) carry **zero** E0277 sites today.
  What remains of that family is `Fnv1a64Structural` (7) and `OccurrenceId` (1).

**No claim is made that a fix closed family 2.** The population moved and this measurement does not
attribute the move; §16 of the partition applies (a site count measures where the compiler pointed).

## Root T5b — 35 sites, and it is a modeling decision, not a repair

The failing self types are values that *contain functions*:

```
 8  PartialFunction<String, Rc<...>>        (a record of closures - partition 11.18)
 5  bare `dyn Fn(..) -> Rc<Outcome<..>>`
 4  CompiledLexRule
 3  EffectIoEvalBundle
 3  ValueInterpreter
 2  each: TransformInterpreter, BranchInterpreter, LoopInterpreter, BindInterpreter, MatchInterpreter
```

against `serde::Deserialize` (19), `Debug` (12) and `serde::Serialize` (4). This is partition
§11.23's T5b, measured E0277-only and at site grain: the derive roster applies serde and `Debug`
unconditionally to every record and coproduct, so **any** declaration transitively reaching a
closure fails. `Rc<dyn Fn>` is not serializable and not `Debug`; there is no derive to add. The
requirement has to go, or the declaration has to split into a serializable description plus a
non-serializable realization, or the function field has to become a resolvable named reference.
That is a decision above the emitter, which is why this root has stayed unowned while being the
largest.

**Concentration is high**: `v2_std_runtime.rs` (13), `v2_std_compilers_target_model.rs` (11) and
`v2_compiler_compile.rs` (4), `v2_compiler_tokenize.rs` (4) and `v2_compiler_eval.rs` (3) carry all
of it. These are shared floor files —
per DESIGN §11 working agreement 3, the root belongs to the file's owner, not to whichever entry
module surfaced it.

## Root A — 30 sites, and 5 of them demand a bound the mechanism cannot express

By self type: `T` 15 · `P` 5 · `U` 3 · `A` 3 · `B` 2 · `S` 1 · `C` 1.
By trait: **`Clone` 25, `Ord` 5.**

The `Clone` 25 are the family `v1.trait_derive_emit`'s three triggers exist to close
(`v1_clone_bound_seed_for_item`, the well-formedness fixpoint, and the fn-declaration trigger).
That they are still live at 25 sites is the measurement Root A's lane asked for; this receipt does
not diagnose why, and explicitly does not claim gunbc#7691 failed — a residue is not a refutation.

**The `Ord` 5 are new, and they are structural rather than incidental.** Every one is
`std_authorization_profile.rs`, one declaration:

```rust
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
...
    members: Rc<BTreeSet<P>>,          // the trait `Ord` is not implemented for `P`
```

whose authority is `std.authorization_profile` `AudienceSet` — `EnumeratedAudience { members: Set<P> }`.
The modeled `Set<P>` realizes as a `BTreeSet`, which requires `Ord` of its element exactly the way
`im::Vector<A>` requires `Clone` of its element. But the whole v1 supplemental-bound mechanism is
**`Clone`-only**: the predicates are named `v1_generic_param_used_as_collection_element`,
`v1_type_param_needs_clone_bound`, `v1_clone_bounded_type_params` — a fixpoint over *one* trait.
There is no arm that can emit `P: Ord`, so these 5 sites are not a gap in Root A's coverage; they
are outside its expressible range.

**This matters more than 5 sites.** `trait_derive_emit_item_clone_bound_contract_fork_note` already
records that v2 models supplemental generic bounds **per derived impl with cited upstream
authorities** (`v2.std.compilers.target_model` `target_derive_supplemental_generic_bound_contract`),
and that the carrier "has no consumer in v1 seed emit today". The `Ord` sites are a live, executed
specimen of what the v1 approximation cannot represent and the v2 contract can — i.e. evidence for
the wire-through that note's dissolution clause names, arriving from the requirement side rather
than from the fork-hygiene side. The note's warning still binds: the wire-through changes the
**grain** (per-derive, not per-type), and unioning per-derive requirements back onto the type
declaration would reproduce v1's over-constraint under v2's name.

## Root R3 — 9 sites, the function-value carrier at call position

```
expected a `Fn(_)` closure, found `Rc<dyn Fn(_) -> Rc<Outcome<_>>>`
```

nine times, all nine in `v2_compiler_compile.rs`. A modeled function value is
carried as `Rc<dyn Fn..>` and handed to a parameter emitted with an `Fn` bound; `Rc<dyn Fn>` does
not itself implement `Fn` unless dereferenced. This is partition §11.3's R3 measured E0277-only.
Small, mechanically uniform, and — unlike T5b — plausibly an emitter-side call-position fix rather
than a modeling decision. It is dispatched separately for exactly that reason: pricing it against
T5b would let a 9-site emitter repair sit behind a corpus-wide modeling question.

## Root T7/T5a — 8 sites, and the blocker is already declared in tree

`Fnv1a64Structural: std::hash::Hash` (4) and `Fnv1a64Structural: Eq` (3), all seven in
`v2_extdeps_runtimes_v2_effect_io_pure.rs` — carried in the TSV as root **T7**. Beside them sits a
single **T5a** row, `OccurrenceId: Eq` in `v2_std_node.rs`: same mechanism (a derive roster that does
not consult how the type is used), different carrier, and it is one site rather than a family. The
two are reported together as 8 because they close together; they are kept as separate root labels in
the TSV because merging a 1 into a 7 on mechanism similarity is how a root acquires members it was
never measured to have.

**Do not dispatch this as new work.** `v1.trait_derive_emit` `map_key_alias_hop_gap_note` describes
this population exactly, names the same four `v2_effect_io_pure` sites, records that the obvious fix
(follow every alias right-hand side) was **attempted, measured, and reverted** because it drags
`Int`/`Nat` into map-key positions and diverges two stage0 files, and states its dissolution: a
realization binding keyed on `DeclarationRef`, which is the same threading the identity-keyed
`lookup_checkpoint` cut is blocked on. So this root is *characterized and blocked*, not unowned. Its
size today is 8 E0277 sites; the note's own count of four is the `v2_effect_io_pure` subset.

## Controls run before any number above was used

1. **The round number was checked, not assumed.** `03_ingest` and `emit_host` both reported
   **exactly** `E0277:100`, which is the signature of a truncating instrument. Checked three ways:
   the logs end normally (`could not compile ... due to 360 previous errors`, and the summed
   per-code histogram is 360, so nothing is missing in aggregate); the last E0277 block sits at log
   line 5422 of 6612, so rustc kept emitting after the hundredth; and a local known-positive control
   — a synthetic file with 120 identical `T: Clone` failures — emits **120** `error[E0277]` blocks
   under the same rustc, so no per-code cap at 100 exists. The two 100s are real counts.
2. **The classifier's refusal arm was proven reachable.** Zero RESIDUE is only a result if RESIDUE
   can fire: a fabricated `the trait bound \`ZZFake: SomeWeirdTrait\` is not satisfied` classifies as
   RESIDUE, not into a neighbouring root. An absence claim from an unvalidated classifier is not
   evidence of absence.
3. **The measurement is one tree.** See Method — and the failed three-dispatch attempt that made
   the point.

## What is NOT claimed

- **No corpus-wide E0277 size.** M=6, so 82 is a lower bound on distinct sites across all 41
  modules. The five floor modules omitted here were byte-identical to `05_emit` at `E0277:55` in the
  2026-08-19 census, and §11.14 measured that four extra floor modules added nine new sites over
  seven — so the corpus figure is expected to be near 82, not far above it. That is an expectation
  with a cited precedent, not a measurement.
- **No burn-down prediction.** A root's site count is an upper bound on defects with an unmeasured
  fan-out (partition §9). 35 T5b sites are not 35 decisions; they are ~10 declarations (§11.23).
- **No claim that any prior fix worked or failed.** Populations moved since July and this receipt
  does not attribute the movement.
- **No mechanism claim for Root A's residual 25.** Located and counted; not diagnosed.
