# Self-host cargo refusals: the root partition (shared coordination surface)

**Purpose.** One stable place for the sessions working the v2 self-host wall to agree on
what the wall IS, which root each defect belongs to, and who owns which root. Operator-directed
2026-08-16. This document is coordination state, not an authority: every claim here names how it
was measured, and an unmeasured claim says so.

**Sessions sharing this surface:** `smart-ram-730` (self-host frontier / root partition) ·
`gentle-dove-833` (interpreter cut Y1 — emit `v2.compiler.eval`).

---

## 1. The milestone was mis-stated, and this is the correction

"1/27 self-emitted" was read for months as *get a compiler module to emit Rust*.
**Modules already emit Rust.** Twenty of twenty-one emit between 24 and 133 files. They then
fail `cargo build`.

The milestone is: **emitted Rust that compiles.**

Three separate places still say otherwise and are known-stale:

- `v2.compiler.self_host.frontier_band_a_emit_readiness` `compiler_frontier_band_a_emit_readiness_note`
  — routes the blocker to `parse_grammar_choice_overlap_residue` and thence to the namespace
  import-grammar cut. Predates gunbc#8265, which peels overlap-residue heads off assemble
  receipts, so the reason it names is a mask.
- the `compiler_frontier_roster` rows — several say these modules stop at `ProbeStageAssemble`.
  The banked receipt says they emit sixty files. Both cannot be true. **Being deleted**
  (operator ruling 2026-08-16, §5 below).
- `docs/plans/v2-self-hosting.md` — "0/27 self-host-green" is still true, but its framing
  invites the emission reading.

**A wrong root to avoid.** `v2.compiler.infer` `node_grounding_frontier_note` says v2 derives
only 3 of 12 node kinds; the other nine carry `infer_grounding_not_derived`. True, and NOT a
blocker — `GroundingNotDerived` sits on the **Accepted** path (DESIGN §4b names it a live
specimen of `FrontierAccepted`, "the typed-located-counted diagnostic whose phase result is
still `Accepted`"). smart-ram-730 read that count as a refusal and had to withdraw it.
Likewise `v2_emitter_direct_rust_door_contract` IS red, but it refuses on **source fidelity**
against a canned string with ~17 hand-authored groundings — a fixture-exactness check, not the
emission path. Do not generalize from either.

## 2. The measurement everything here rests on

`docs/probes/curated_cargo_frontier_probe_sweep.tsv` — banked `941e8034862`, **2026-07-26**.
Produced by `docs/probes/curated_cargo_probe_one.sh` (emit → `cssl_assemble` → cargo).

Caveats the receipt carries, not to be rounded off:

- **Three weeks stale.** Treat as shape, not current counts. Refresh before planning against numbers.
- **`first_error` is `UNRESOLVED_CompilerError` on 20 of 21 rows.** Only the residual histogram
  carries signal. "E0308 dominant" is honest; "first error is E0308" is not.
- `01_tokenize` is the sole row whose first error the classifier coded:
  `error[E0432]: unresolved import crate::std_nat` → `CONFIRMED-namespace`.

**Do NOT route refusal readings through `frontier_probe_survey`.** It has produced zero receipts
on any host since at least 2026-08-06 — six kills in the banked receipt, plus a silent 27-minute
death on a dedicated BuildBuddy runner and a kernel OOM on 2026-08-16. The in-tree note blames
shared-host memory pressure; reproduction on a dedicated runner refutes that. It is scaffolding
with its own deletion trigger; do not repair it. `curated_cargo_probe_one.sh` is the working tool.

**Instrument-vintage trap.** `build.rs` deliberately omits `cargo:rerun-if-changed` on
`.git/HEAD`, so an embedded commit stamp AGES until someone touches `build.rs`. The pin refusal
is the only thing between that and surveying today's tree with a month-old instrument.
`touch src/v1/stage0/build.rs` refreshes it.

## 3. The census

9,444 error instances, 20 modules, 24 distinct rustc codes. Top three are 75%.

| code | instances | modules |
|---|---:|---:|
| E0308 mismatched types | 3260 | 20/20 |
| E0277 trait bound | 1947 | 20/20 |
| E0599 no method | 1912 | 20/20 |
| E0369 binary op unsupported | 671 | 20/20 |
| E0107 generic arg count | 403 | 20/20 |
| E0063 missing struct field | 337 | 19 |
| E0282 type annotations needed | 216 | 20 |
| E0597 borrow lifetime | 202 | 19 |
| E0614 cannot deref | 167 | 18 |
| E0609 no field | 125 | 18 |
| E0061 44 · E0392 40 · E0560 23 · E0631 18 · E0004 18 · E0433 16 · E0310 14 · E0425 12 · E0615/E0573/E0271/E0223 4 · E0533 2 · E0432 1 | | |

Per module, total then histogram:

```
03_ingest              1053  E0308:432 E0277:193 E0599:125 E0369:90 E0107:58 E0614:32 E0597:25 E0282:23 E0063:22 ...
00_compile             1052  E0308:432 E0277:193 E0599:125 E0369:90 E0107:58 E0614:31 ...
source_authority        861  E0308:377 E0277:178 E0599:112 E0369:75 E0107:35 ...
emit_host               621  E0308:202 E0277:129 E0599:96  E0369:79 E0107:31 ...
02_parse                595  E0308:178 E0277:162 E0599:93  E0369:73 E0107:25 ...
emit_produced           435  E0308:141 E0599:96  E0277:90  E0614:21 ...
03_normalize            419  E0308:200 E0599:88  E0277:65  ...
05_eval                 410  E0308:117 E0599:91  E0277:81  E0369:29 E0614:22 ...
program_partition       387  E0308:116 E0599:104 E0277:76  ...
05_emit_orchestration   381  E0308:119 E0599:93  E0277:81  ...
emit_module             372  E0308:122 E0599:93  E0277:76  ...
emit_semantic_decl      366  E0308:115 E0599:93  E0277:76  ...
06_translate            364  E0308:114 E0599:93  E0277:76 E0063:18 E0369:14 E0597:13 E0107:13 E0282:9 E0609:6 E0614:3 E0392:2 E0560:1 E0061:1 E0004:1
05_emit                 364  E0308:114 E0599:93  E0277:76 E0063:18 E0369:14 E0597:13 E0107:13 E0282:9 E0609:6 E0614:3 E0392:2 E0560:1 E0061:1 E0004:1
04_infer                327  E0308:91  E0599:88  E0277:75  ...
materialization_carrier 324  E0277:90  E0599:81  E0308:77  E0369:60 E0107:13 E0061:2 E0282:1
03_name_resolve         315  E0599:89  E0308:89  E0277:65  ...
03_resolve              305  E0599:88  E0308:84  E0277:65 E0063:18 E0369:13 E0107:13 E0282:9 E0609:6 E0614:3 E0597:2 E0392:2 E0061:1 E0004:1
fold_lowering           291  E0599:82  E0308:78  E0277:65 E0063:18 E0369:13 E0107:13 E0282:9 E0609:5 E0614:3 E0392:2 E0597:1 E0061:1 E0004:1
01_tokenize             202  E0599:89  E0308:62  E0277:35  E0369:5 E0107:5 E0597:2 E0282:2 E0432:1 E0063:1
program_assembly          0  emit_fail — the one module that does not reach cargo
```

## 4. Hypotheses, each with what would falsify it

**These are hypotheses. None is confirmed. Do not plan against them as findings.**

**H1 — there is a shared floor, and it is most of the volume.**
`05_emit` is 35 source lines; `06_translate` is 4,226. Both total **364**, with identical
histograms code-for-code. `fold_lowering` (164 lines) is 291 with the same tail. Reading: the
shared emitted closure fails the same way in every crate, ~290 deep, and each module adds a
delta. If true, this is one core plus twenty deltas — the 9,444 is the same defects counted
twenty times, and a core fix drops every module at once.
*Falsified by:* extracting real error TEXTS for two modules and finding the intersection small.
Histogram similarity is suggestive, not proof. **This is the next measurement to run.**

**H2 — E0308 + E0277 + E0599 are one root, not three.**
That triple is the signature of a type-representation fork, which DESIGN already tracks: every
primitive modeled as a coproduct, realized as a native `Value`, reconciled by per-site bridges,
"so coverage is accidental and non-compositional." A modeled `Nat` landing where a native i64 is
expected yields a mismatch at the value, a missing method on the wrong carrier, and a missing
impl for that form.
*Falsified by:* error texts showing the three codes citing disjoint type pairs, or E0599s whose
receivers are unrelated to any coproduct/native straddle.
*Risk being managed:* three roots must not be merged merely because they correlate.

**H3 — `01_tokenize` is unrepresentative.**
It sits BELOW the floor (202) with a different profile — no E0063:18, no E0614, no E0392,
no E0004; E0369/E0107 at 5 rather than 13. Smaller closure, not just a smaller module. It was
smart-ram-730's first pick on the strength of being the only coded first error; that pick is
**withdrawn pending H1**, because fixing an outlier may teach nothing about the other nineteen.

## 5. Frontier roster — DELETED (operator ruling, 2026-08-16)

> "I would not have any dual authority rows in the frontier — either make it derived from live
> state, or non-existent asap — otherwise people get too enamored with it." … "let's delete it,
> it's confusing and unhelpful, which is negative value."

Rationale, in the roster's own terms: `execution_measured_seed_retained_row` takes
`measured_blocker`, `located_stage` and `located_reason` as **ordinary parameters**, so a row
that claims measurement is structurally indistinguishable from one that asserts it — the
constructor NAME asserts a provenance the TYPE does not carry
(`frontier_roster_provenance_constructor_inflation_note` says so in tree). Ten of twenty-seven
were never execution-measured at any head. It is an attractor: three separate readers took its
rows as measurements this week, and one census was withdrawn over it.

Deleted rather than derived: everything worth keeping is already derivable elsewhere — the
module list from the filesystem, composition from source in one command, cargo status from the
sweep receipt. The disposition/blocker/stage fields are the part that cannot be derived and were
the part that lied. Per DESIGN §3 delete-first, **the deletion is the census**: real consumers
refuse loudly. Expected load-bearing consumer: the crate-layout emitter
(`compiler_frontier_crate_layout_note`).

## 6. Division of labour

`05_eval` totals 410, of which ~290 looks like shared floor under H1. So the child's lane is
currently blocked behind defects that are not eval's and cannot be fixed from inside eval.
Consequence: **neither session fixes this per-module.** Root ownership is assigned here once the
partition is confirmed, so the two lanes work disjoint roots rather than the same wall twice.

| root | owner | status |
|---|---|---|
| (partition pending H1/H2 confirmation) | — | not yet assigned |

## 7. Open, and who is asked

- H1 confirmation by error-text intersection — smart-ram-730, next action.
- Refresh the three-week-old sweep with `curated_cargo_probe_one.sh` — smart-ram-730.
- Independent root partition requested from the linked side channel; its answer lands here, and
  its claims are evidence to check, not authority (it has already cited one symbol that does not
  exist in the tree).
