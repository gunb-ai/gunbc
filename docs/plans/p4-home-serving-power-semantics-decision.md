# P4 home-serving selection: what the watt figure means

Status: **decision requested — nothing built.** One question, one recommendation, no module
changed.

Owner: vivid-ant-730 ("own PR4", assigned by the operator).
Depends on: P1 #13762 (merged de0fedf0005), P2 #13771 (merged 466a08da), P3 #13774 (open).
Every citation below was re-checked against `origin/main`; the ones I could not confirm are
deleted rather than hedged.

## The request P4 is meant to satisfy

`gunbc.serving.home_serving_accelerator_field` declares P4's frontier in one row:

> "DESIGN section 3c declared frontier: these lists are consumed by P4 home-serving selection,
> which imports `home_serving_add_in_card_catalog_rows` as the purchase-candidate field and
> `home_serving_reference_sxm_catalog_rows` as references only. Trigger: that fold exists and
> deleting either list makes it fail to resolve. **A witness that only asserts the lists exist
> does not close this frontier.**"
> — `dag/gunbc/serving/home_serving_accelerator_field.dag:18`

So the deliverable is a **selection fold over the card list**, not another list. Today the only
thing that touches either list is one existence claim:

```
test fn w_purchase_candidates_are_add_in_cards_and_sxm_are_references() -> Bool {
  catalog_sku_in(rows: home_serving_add_in_card_catalog_rows, row: nvidia_rtx_pro_5000_72gb_catalog)
    && catalog_sku_in(rows: home_serving_add_in_card_catalog_rows, row: nvidia_rtx_pro_6000_workstation_catalog)
    && !catalog_sku_in(rows: home_serving_add_in_card_catalog_rows, row: nvidia_b200_sxm_catalog)
    && ...
```
— `dag/test/claim/home_serving_extdeps_facts_witness.dag:129`

That is precisely the existence-assertion the frontier says does not close it. The frontier is
open, and honest about being open.

"cards × power limits" is the shape. **"What the watt figure means" is the decision that has to
be made before the shape can be built**, because the tree holds two different watt carriers that
both read 600 for the PRO 6000 and do not mean the same thing.

## The finding: two Watt carriers, both 600, meaning different things

| carrier | where | standing | what it is |
|---|---|---|---|
| `GpuModelCatalogRow.tdp_watts: Watt?` | `extdeps/gpu/types.dag:120` | cited — `CitedToAuthority`, on the datasheet (P1) | a **published ceiling** of the SKU, from NVIDIA's document |
| `ServingAcceleratorRealization.applied_power_limit: Watt?` | `gunbc/spark/serving_performance_subject.dag:126` | **carries no standing at all** | a **limit set on one run** of one machine |

Both are the same carrier type — `std.measure` `Watt` — and both currently hold 600 for the PRO
6000 Workstation. Nothing in the tree relates them: `git grep -i tdp origin/main -- dag/gunbc/spark
dag/gunbc/serving` returns **zero** hits. There is no `applied ≤ tdp` rule, and no statement of
which is which. A subject at an applied 600 W is, as the tree stands, readable as a card with a
TDP of 600 W. Those are different facts about different things — one is a property of the product,
the other a property of an afternoon.

Three measurements sharpen this from "confusing" to "actively misleading".

### (1) `tdp_watts` is load-bearing in a way no other catalog field is

Diffing the two PRO 6000 rows field by field, every field is identical except the model name, the
citation list, and TDP:

```
<   model: "RTX PRO 6000 Blackwell Workstation Edition",   >   model: "RTX PRO 6000 Blackwell Max-Q",
<   tdp_watts: watt(count: 600),                            >   tdp_watts: watt(count: 300),
<   figure_citations: rtx_pro_6000_workstation_...          >   figure_citations: rtx_pro_6000_max_q_...
```

Same thread count, same package, same 96 GiB, same 1792 GB/s, same compute capability. **TDP is
the only field that distinguishes them** — and both are in the purchase-candidate list:

```
data home_serving_add_in_card_catalog_rows: List<GpuModelCatalogRow> = [
  nvidia_rtx_pro_5000_48gb_catalog,      nvidia_rtx_pro_5000_72gb_catalog,
  nvidia_rtx_pro_6000_workstation_catalog, nvidia_rtx_pro_6000_max_q_catalog,
]                                                    -- home_serving_accelerator_field.dag:21
```

For a home-serving purpose those two are not two products: they are one product at two power
ceilings. And the PRO 5000 pair behaves the opposite way — identical except capacity (48 vs 72
GiB), both `watt(count: 300)`. So TDP and capacity are each load-bearing discriminators, across
*different* pairs, and the list cannot tell you which one is doing the work.

Because the field is `Watt?` (optional) and the list is `List<GpuModelCatalogRow>`, today
"× power limits" would multiply the card list by a field that is already inside the card row —
restating it, not sweeping it.

### (2) No receipt anywhere records an applied power limit

undefined

- `fixtures/gpu_bench/pro6000ws_600w_v2/warm_ninfer_nvfp4_pro6000ws_600w.jsonl` (91,935 B; 268
  records) has 15 keys per record: `completion_tokens, error, max_tokens, n_active, phase,
  prompt_tokens, retained, round, run, schema, session, status, t_end, t_first, t_start`. No power
  field.
- `auto.log` (2,038 B) and `results_ninfer_nvfp4_pro6000ws_600w.txt` (1,023 B): `grep -Ei
  'watt|power|tdp'` → no match in either.
- The substring "600" occurs 542 times in the JSONL. 536 of them are inside the run name
  `"run": "warm_ninfer_nvfp4_pro6000ws_600w"` (268 records × 2 occurrences per name — the name
  contains `600` twice, in `pro6000ws` and in `600w`). The remaining 6 are digit runs inside epoch
  timestamps, e.g. `"t_end": 1791770016.2**600**296`, `"t_start": 1791770056.07**600**21`,
  `"t_first": 1791770374.38**600**52 — six values among `t_start`/`t_first`/`t_end`. So not one
  occurrence of "600" is a measurement: 536 are a label and 6 are coincidental digits in a clock
  reading. (An earlier draft of this document said "only inside the run name"; that was wrong by
  those 6.)

So `600w` in this repo today means *"the 600 W SKU"*: a SKU label baked into a directory name, a
filename, and a run id. It does **not** mean *"this run executed under a 600 W limit"*, and there
is no field in which that claim could be recorded.

`applied_power_limit` has **no producer outside its own module** and is not read by anything that
judges power. All 11 sites:

| site | role |
|---|---|
| `serving_performance_subject.dag:126` | field declaration |
| `:413` | `Absent => AxisUnread`, else `AxisRead` — reads **presence only**; the watt value itself is never inspected |
| `:514` | `match a.applied_power_limit { Absent => "<unread>"; Present { value: w } => to_string(watt_count(w: w)) }` — a **wire projection**, which is realization, not identity |
| `:525`, `:531` | the sole constructor's parameter and its assignment |
| `:681` | `both_read_unequal(left: …, right: …)` inside `accelerator_known_parts_differ` — the one place the **value** is compared |
| `serving_performance_subject_witness_test.dag:136,145,154,245` | fixtures |
| `serving_performance_subject_witness_test.dag:250` | `w_applied_power_limit_is_an_accelerator_axis` |

There is no non-test caller of `serving_accelerator_realization` on main: no benchmark path, no
readback, no fold. The field is exercised only by its own witness.

And that witness is where the ambiguity becomes concrete rather than theoretical:

```
fn fx_cited(sku: GpuModelCatalogRow, watts: Nat, lanes: Int) -> … {
    applied_power_limit: Present { value: watt(count: watts) },   -- :136
}
… fx_cited(sku: nvidia_rtx_pro_6000_workstation_catalog, watts: 600, lanes: 16)   -- :226
```

The row passed in has `tdp_watts: watt(count: 600)`; the applied limit supplied beside it is
`watt(count: 600)`. They agree **by coincidence of typing, not by any rule** — and if that
coincidence were load-bearing, `w_applied_power_limit_is_an_accelerator_axis` (`:250`, which
compares 600 against 400 on the same SKU) could not tell them apart from a TDP change. That is
how a coincidence gets written into a test as though it were a relation.

### (3) The home question cannot currently be answered at all

`extdeps/nvidia/management_library.dag` — the NVML module — has **zero** occurrences of "power".
It carries `DeviceMemoryObservation` and memory observations only. There is no way to *read* what
limit a card is running under, which is the question "home serving" actually asks: not "what may
this card draw" but "what is it drawing, in a room, on a desk".

## Why this cannot be resolved by picking a number

The other three P4 inputs are ordinary product decisions. **Cards**: NVIDIA publishes both lists,
P1's rows are cited, and the frontier's lists already partition them — any defensible default
(the four current add-in rows) is safe to type. **Power rungs**: a sweep list of `Watt` values,
safe to type. **The meaning of the watt figure** is the one input that determines **which type the
whole selection is written in** — pick wrong and P4 ships a fold that multiplies two things that
were never independent dimensions, and every receipt under it inherits the confusion.

DESIGN §3d is directly about this and is stronger than a style preference:

> "A candidate violating a hard constraint is **excluded *before* Pareto, never ranked**."
> "The result is scoped, never `Optimal`: `SelectedWithin` binds a `SelectionReceipt`… A change to
> any of those is a **different decision**, so a stale choice is detectable."
> — `DESIGN.md:303` (§3d)

If the watt figure is a hard constraint (a card may not be asked to exceed its cited ceiling), it
must exclude before Pareto. If it is an objective axis, it is ranked. Those are different
positions in the fold, and the same `Watt` value cannot be both without saying which. §3d also
settles the vocabulary: `std.pareto` computes the frontier and `std.decision` chooses, and P4
must **consume** them — "a second decision algebra is the §3 fork." The real types to inhabit are
`RealizationSelectionResult` with `SelectedWithin { candidate, receipt }` and
`SelectionNeedsEvidence { needs }` (`dag/std/decision.dag:839-844`), over
`ParetoEntry { identity, display_label, readings, missing_inputs }` and
`SelectionAxis { identity, goal }` (`dag/std/pareto.dag:259,292`) — not a minted parallel.

## Options

**A. The watt figure is the catalog TDP only.** Selection sweeps `tdp_watts`; P2's
`applied_power_limit` is scaffolding P4 does not consume.
*Against*: the axis becomes meaningless. The catalog already carries TDP on the row, so "× power
limits" would re-declare a field that is already there, and the only interesting home question —
running a 600 W card at 400 W — is inexpressible.

**B. The watt figure is the applied limit, read as a property of the run.**
*For*: matches P2's construction — `applied_power_limit` sits on the realization beside `count`
and `host_link`, all run-scoped, all optional.
*Against, and fatal as stated*: there is no source for it. No NVML power field, no committed
receipt that carries one, and P3's own layering rule requires that an applied limit be **cited to
something** before selection consumes it. Selecting on it now means selecting on a number that
otherwise appears only in a run name — §4d: *"Do not assert as deduced what is only inferred. An
inference is a bet: typed as a bet, neither deleted nor promoted."*

**C. Both, kept distinct and related by an explicit rule.** The axis is one value per candidate
carrying both facts without conflating them:

```
type AppliedPowerLimit                          // makes 'a watt with no provenance' unwritable
  = AppliedLimitUnread
  | AppliedLimitGrounded { limit: Watt, standing: CitedFigureStanding }
  // standing stays the SHARED coproduct: external_authority:13-16 rules CITED/UNCITED/OBSERVED
  // "ONE STANDING, not a per-domain coproduct", so AppliedPowerLimit cannot narrow it to a
  // per-domain subset; which arm an applied limit may take is a witness + review fact, not a type fact

type HomeServingPowerPolicy
  = PowerPolicyUnread { missing: NonEmptyStr }   // neither known: refused, never defaulted
  | PowerPolicyGrounded {
      tdp_watts: Watt                            // cited ceiling; required for a candidate
      tdp_standing: CitedFigureStanding          // which datasheet cites it
      applied: AppliedPowerLimit
    }
```

`AppliedPowerLimit` exists so the bad state is unwritable rather than validated: there is no
constructor that takes a watt without also taking its standing. That is §5's preference —
*"Stronger than catching a wrong state is making it **unwritable — correctness by construction, not
validation.**"* (`DESIGN.md:383`) — and it is the same move P2 made with `ServingSubjectAxisReading`
(`AxisNotApplicable | AxisUnread | AxisRead`) rather than a pile of optional-presence matches.

Both standing fields reuse `extdeps.external_authority`'s `CitedFigureStanding`
(`dag/extdeps/external_authority.dag:40`) rather than minting a parallel provenance type. That
coproduct already answers exactly what these fields ask, with three arms and no more:

```
type CitedFigureStanding
  = CitedToAuthority { authority: ExternalAuthority }   # NVIDIA said it (datasheet, kit config)
  | TranscribedUncited { read_obligation: NonEmptyStr }  # someone said it; nobody cites it
  | ObservedByReceipt { observed: ObservedReceipt }      # this repo measured it
```

Reusing it is the §2 requirement ("net concepts must not grow by re-invention"), and it buys three
things for free: the standing algebra is the one P3 just exercised on the ninfer replay counter, so
an applied limit read off NVML and a replay counter read off a log carry their epistemic weight the
same way; `cited_figure_read_obligation` (`:57`) already returns `Absent` for both grounded arms and
`Present` only for `TranscribedUncited`, so "what must still be read" is answered once; and a
`TranscribedUncited` applied limit cannot silently act like a cited one, because it must carry its
read obligation into any refusal it causes.

Value-beside-standing is also how the catalog already does it, so this is not a new shape:
`GpuModelCatalogRow` carries `tdp_watts: Watt?` (`extdeps/gpu/types.dag:120`) next to
`figure_citations: List<FactCitation>`, and P1 pairs the CUDA count with
`rtx_pro_6000_workstation_cuda_core_standing: CitedFigureStanding = CitedToAuthority { authority:
rtx_pro_6000_workstation_datasheet_authority }` (`extdeps/gpu/nvidia.dag:127`). Candidate identity
is likewise already solved and must be consumed, not reinvented: `gpu_catalog_row_eq`
(`extdeps/gpu/types.dag:132`) fixes a row's identity as *cited vendor plus cited SKU name*, with the
comment warning that *"Package×VRAM×TDP is not enough"*.

Two consequences worth stating rather than discovering later. First, `ObservedByReceipt` is
admission-gated: `fn observed_receipt` (`:45`) lists its `admit_callers` explicitly, so the first
home-serving fold that claims an *observed* applied limit is itself an admitting change — a named
file and a review, not a call. Second, keeping `tdp_standing` beside the number rather than
assuming it is deliberate: a ceiling without its datasheet is not the same fact as a ceiling with
it, and P1 already attached the datasheet citation to exactly these rows.

**The seal is only a third of this coproduct, and C's headline claim needs that stated honestly.**
`ObservedReceipt` is `sole_constructor` (`:36`) and `observed_receipt` is `admit_callers`-gated
(`:45-46`), but `CitedToAuthority` carries `ExternalAuthority`, which is a plain open record —
`type ExternalAuthority { uri: Uri }` (`:9`), no constructor function, no admission list. So any
module that imports the coproduct can already mint `CitedToAuthority { authority:
ExternalAuthority { uri: ... } }` pointing at any string. What option C makes **unwritable is a watt
carrying no standing**, not a watt carrying a standing that does not actually state its figure. The
second failure is real and has form here: review 79389 on P1 caught exactly that — the Workstation
CUDA-core count and a PCIe `x16` were sitting `CitedToAuthority` on
`rtx_pro_6000_workstation_authority` (`extdeps/gpu/nvidia.dag:90`), whose locator is the *products*
page `www.nvidia.com/en-us/products/workstations/professional-desktop-gpus/rtx-pro-6000/`, which does
not state either figure. The fix added `rtx_pro_6000_workstation_datasheet_authority` (`:97`) with
the real PDF (`.../workstation-datasheet-blackwell-rtx-pro6000-x-nvidia-us-3519208-web.pdf`) and
re-pointed the standing and both citations at it; it landed in P1's squash `de0fedf0005`.

So the guarantee on the cited arm is a *reviewable named declaration plus the human check*, not a
type-level bar — which is also why the standing must be a named `data` row that a witness can pin,
never an inline `CitedToAuthority` written at the use site. Two guards follow, and both are cheap: a
witness pins each power standing's `authority.uri.locator` to the exact string (a citation cannot
drift to a friendlier page unnoticed, which is how 79389 got in), and the applied-limit arm, if the
operator chooses C with an observed reading, goes through the *sealed* arm rather than the open one —
which is why `AppliedLimitGrounded` should not offer `CitedToAuthority` as the cheap default for a
number this repository measured itself. `gunbc.recurring_failure_mode`
`citation_verified_against_a_rendering_that_destroyed_its_scope` already rosters the adjacent class.

with the relation stated as a rule instead of implied by a shared type: `applied_limit ≤
tdp_watts`, refused loudly when violated, and `PowerPolicyUnread` refused at the fold — §5: *"A
wrong answer is a loud error, never a warning… no plausible output is fabricated"* (`DESIGN.md:381`).
The same shape P1 chose for a missing lane count (`MachineShapeRefused { cause:
CatalogExecutionLanesUnstated }`, `extdeps/gpu/machine_shape.dag:97`) and P3 chose for a missing
capacity (`AxisGap`). The Max-Q / Workstation pair then stops being two unrelated SKUs and becomes
one card at two cited ceilings, and sweeping a 600 W card at 400 W becomes expressible without
pretending NVIDIA published a 400 W card.

Why `tdp_watts` can be **required** in `PowerPolicyGrounded` while the catalog field is `Watt?`: all
four purchase candidates do carry a cited ceiling (600 / 300 / 300 / 300), and the two rows with
`tdp_watts: none` are exactly `nvidia_b200_sxm_catalog` and `nvidia_b300_sxm_catalog` — the SXM
rows, which the frontier already designates *references only*, not candidates. So the type is
well-formed on the data as it stands, and the optionality that `Watt?` currently carries is not a
gap in the candidate field; it is the reference rows travelling in a type they should not have to
share. If a future candidate ever lacks a published ceiling, that is a `PowerPolicyUnread` refusal
at the fold, not a silent `none`.

## Recommendation

**C**, with the applied arm starting honestly at `AppliedLimitUnread`, and no constructor anywhere
that accepts a watt without its `CitedFigureStanding` — so a limit cannot enter the fold without
saying where it came from.

One line each:
- It is the only option true to what the tree actually holds: a cited ceiling we have, and an
  applied limit we do not have and cannot currently read.
- It makes "× power limits" add information instead of restating `tdp_watts`.
- It does not promote an inference to a fact in order to get the fold written (§4d), and keeps the
  two standings from collapsing into the shared `Watt` type that is the root of the ambiguity.
- It leaves A and B reachable as special cases: A is `applied: AppliedLimitUnread`, B is
  `applied: AppliedLimitGrounded { standing: ObservedByReceipt {…} }`. Choosing C now forecloses neither; choosing A or B now forecloses
  the other and re-types every receipt underneath it.
- The repo already made this call once, in the same vendor's own terms. `extdeps/systems/nvidia.dag:58`
  carries the Spark's 140 W and refuses to let it leak across a type boundary: *"the 140W figure is
  the WHOLE SoC's TDP, not the GPU alone or the CPU alone, so it is carried at the system row,
  **never misattributed onto the nested GPU catalog row's own `tdp_watts` field**."* Two power
  figures that look interchangeable and are not is exactly the case that comment was written for.

**What I need from the operator, as one decision**: confirm reading C, or overrule me with A or B.
Cards and rungs are then mine to draft against the four cited add-in rows and the two SXM
references, and I will bring the fold, not another list.

**What I will not do without it**: build the fold, add a witness, or touch
`home_serving_accelerator_field.dag`. P4's trigger is "that fold exists and deleting either list
makes it fail to resolve" — a fold written on the wrong watt meaning would satisfy the trigger
while failing the request. That is the failure mode §5 already names (*"Fluent, type-checking,
grep-passing output is exactly what looks finished without running"*, `DESIGN.md:389`) and which I
watched happen on #13774: two witnesses dead on arrival from their first commit, across three SHAs
and three approving reviews that were each individually correct and collectively no evidence.
P4's own acceptance will therefore be a discriminating input that goes red when the power rule is
inverted, not a green tick.

## The shape P4 must end in, and the ceiling it discharges

snappy-wolf-791 handed me its P4 design note as input. I verified every claim in it against source
at `a7724b72999` before letting any of it into this document; the four citation slips it contained
(two `compiler_tests.rs` lines, `cited_figure_read_obligation`'s line, and a stale commit for this
doc) are listed in Facts and none changed a conclusion. What survives verification is the shape P4
should take, and one finding that changes what P4 *is*.

**One value binding four things**, so a rate can never be detached from the subject it was measured
on — an admitted receipt identity, the `ServingPerformanceSubject`, the measured rate, and the final
standing. It should reuse what exists rather than mint: the standing is
`GpuBenchWarmStanding` (`gunbc/serving/gpu_bench_warm_standing.dag:53-57`), the subject is
`ServingPerformanceSubject` (`gunbc/spark/serving_performance_subject.dag:77`, `sole_constructor`),
and the rate carrier already exists as `TokensPerSecond = Measure<Frequency, One, Nat>`
(`std/measure.dag:1546`, with `tokens_per_second` `:1548`, `tokens_per_second_count` `:1552`, and
`MilliTokensPerSecond` `:1560` on the same axis) — so a rate is not carried as a `Nat` and no new
rate type is minted (§2). The precedent to imitate is one level down: `SubjectBoundCacheFit`
(`gunbc/spark/serving_promotion.dag:120`), a small record whose only purpose is *"a verdict that
cannot be named without its subject key"*, plus the `ServingPromotionEvidence` (`:162`) /
`ServingPromotionReceipts` (`:177`) split, which exists so a caller can hand over receipts but
cannot supply a verdict the wall computes itself — and `cache_bound_to_resolved_plan` (`:267`)
re-checks the binding at use time rather than trusting it.

**The finding that changes P4's framing: this is discharging a declared ceiling, not new scope.**
`gunbc.recurring_failure_mode.benchmark_warm_capacity_violation_read_as_engine_performance` states
its CEILING as *"through the admitted receipt constructor, an observation with a proved capacity
violation or other established warm-policy failure cannot become eligible warm-performance
evidence"*, and names its NEXT TRIGGER as *"P4 selection consumes only `WarmValid` as a funded
axis"*. The machinery that CEILING clause names does not exist in the tree.
`GpuBenchWarmStanding` is an **unsealed coproduct**; `WarmValid` appears outside its own module only
ever as a match pattern or in prose — 24 occurrences in
`dag/test/claim/gpu_bench_warm_standing_witness.dag`, 2 in
`dag/test/claim/gpu_bench_v2_receipt_inhabitance_witness.dag`, and one prose mention inside the
failure-mode row's own text — with **zero value constructions** anywhere (the one
`WarmValid { working_set_tokens: w, measured_kv_tokens: k }` at
`gpu_bench_warm_standing_witness.dag:280` is a match arm: it ends in `=>`); and the module's minting
functions carry **no `admit_callers`** (grep: 0 hits in the file). So nothing in the compiler today
stops a P4 module from writing `WarmValid { working_set_tokens: …, measured_kv_tokens: … }` literally and
pairing it with a subject it made up. The existing claims
(`the_capacity_fit_becomes_evidence_only_when_the_counter_is_faced`,
`a_replayed_prefix_refuses_what_an_unreplayed_one_funds`) prove the gate discriminates when the
value arrives through those folds; they cannot say anything about a hand-typed arm. The witness comment at
`dag/test/claim/gpu_bench_warm_standing_witness.dag:260` says the near-true thing from the other side — *"`WarmValid` IS reachable, and only
by carrying the reuse evidence into stage 3"* — which is true of the folds and silent about the
literal. (The failure-mode row itself is authored at `de0fedf0005`, `gunbai-bot[bot]`, 2026-10-14 —
P1's squash. The gap belongs to nobody in particular, and naming it is not an accusation.)

Why the seal has to be a **record with one admitted mint**, not the coproduct arm: DAG coproduct
arms are not sealed, and I read the enforcement rather than trusting the error text.
`sole_constructor_construction_diags` (`src/v1/stage0/src/v1_compiler_infer.rs:17224`) calls
`type_has_sole_constructor` and then compares `decl.span.file == span.file`; the diagnostic reads
*"cannot be constructed outside its defining module"* (`src/v1/stage0/src/v1_std_core.rs:1119`), but
the comparison is a **file path**, not a module path. Today that coincidence is harmless — across
all 6,730 `.dag` files there are exactly 6,730 real module declarations, one per file (the single
apparent exception, `dag/gunbc/recurring_failure_mode/annotation_authored_at_a_grain_the_model_does_not_carry.dag:19`,
is the phrase "module item follows it" inside prose) — but a second module sharing a file would sit
inside the first one's seal boundary. `admit_callers` is therefore not belt-and-braces: it gates by
caller **declaration coordinates** (`constructor_admission_diags_from_lookup`,
`v1_compiler_infer.rs:1737`, comparing `caller_decl_coords` against `admit_callers_entry_coords`),
which is file-agnostic. Sealed record + single mint + `admit_callers` is the only version of "bound"
that survives a future caller in a hurry, and it is exactly the `ObservedReceipt` pattern
(`extdeps/external_authority.dag:36`, `:45-46`) this repository already uses for receipts.

One thing not to "fix" while building this shape: the committed receipt ends `WarmCapacityUnread`
because `extdeps.ninfer.replayed_tokens_total` is `TranscribedUncited`, and
`cited_figure_read_obligation` (`extdeps/external_authority.dag:57`) returns `Present` only for
`TranscribedUncited` — `Absent` for `CitedToAuthority` and `ObservedByReceipt`. The honest route to a
`WarmValid` over that fixture is to make the counter *grounded*, not to soften the standing fold. And
if that grounding is an observed reading, the fold must take no arguments and read committed
in-corpus bytes, per `observed_receipt`'s admission requirement, with the honesty note at
`external_authority.dag:28-32` respected: a fold over transcribed bytes is a boundary transcription,
so name the seam and its trigger rather than claiming the substrate checks it.

undefined
1. `GpuModelCatalogRow.tdp_watts: Watt?` — `extdeps/gpu/types.dag:120`.
2. `ServingAcceleratorRealization.applied_power_limit: Watt?` — `gunbc/spark/serving_performance_subject.dag:126`; the type has 4 fields (`sku`, `count`, `host_link`, `applied_power_limit`).
3. No `applied ≤ tdp` rule exists — `git grep -i tdp origin/main -- dag/gunbc/spark dag/gunbc/serving` → 0 hits.
4. `applied_power_limit` has 11 sites: 6 in its module (`:126,413,514,525,531,681`), 5 in its own witness (`:136,145,154,245,250`). No non-test caller of `serving_accelerator_realization` on main.
5. `:413` reads presence only; `:514` renders through `to_string(watt_count(w: w))`; `:681` is the sole value comparison.
6. PRO 6000 Workstation vs Max-Q differ only in `model`, `tdp_watts` (600 vs 300), `figure_citations` — field-by-field diff of the two rows in `extdeps/gpu/nvidia.dag`.
7. PRO 5000 48 GB vs 72 GB differ only in `model`, capacity, `figure_citations`; both `watt(count: 300)`.
8. Candidate list contents and SXM reference list — `home_serving_accelerator_field.dag:22` and `:29`.
9. `warm_ninfer_nvfp4_pro6000ws_600w.jsonl` = 91,935 B, 268 records, **15** keys per record
   (`completion_tokens, error, max_tokens, n_active, phase, prompt_tokens, retained, round, run,
   schema, session, status, t_end, t_first, t_start`), none power-related. "600" occurs 542 times:
   536 inside the run name (268 × 2 — the name carries `600` twice, in `pro6000ws` and `600w`) and 6
undefined
10. `auto.log` and `results_ninfer_nvfp4_pro6000ws_600w.txt` contain no `watt|power|tdp` match.
11. `extdeps/nvidia/management_library.dag` contains no "power"; its observations are `DeviceMemoryObservation`.
12. Neither home-serving list has a consumer beyond `w_purchase_candidates_are_add_in_cards_and_sxm_are_references` (`dag/test/claim/home_serving_extdeps_facts_witness.dag:129`).
13. `std.decision` ships `RealizationSelectionResult` (`SelectedWithin` / `SelectionNeedsEvidence` / `SelectionNeedsPolicy` / …) at `dag/std/decision.dag:839-844` and `SelectionReceipt { scope, pareto_survivors, policy_authority, selected_identity }` at `:824`. `std.pareto` ships `SelectionAxis` (`:259`) and `ParetoEntry` (`:292`).
14. §3d hard-constraint and scoped-result quotes — `DESIGN.md:303`–`317`. §5 loud-error — `DESIGN.md:381`. §5 "Done means executed" — `DESIGN.md:389`. §4d symmetric-obligation bullets — `DESIGN.md:367`–`378`.
15. The Spark 140 W non-misattribution precedent — `extdeps/systems/nvidia.dag:58`, carried as `soc_thermal_design_power: watt(count: 140)` at `:113`.
16. `CitedFigureStanding` with exactly three arms — `dag/extdeps/external_authority.dag:40-43`; `fn observed_receipt` is `admit_callers`-gated at `:45-46`; `cited_figure_read_obligation` at `:57` returns `Absent` for `CitedToAuthority` and `ObservedByReceipt` and `Present` only for `TranscribedUncited`.
17. Value-beside-standing is the existing catalog convention — `GpuModelCatalogRow.tdp_watts: Watt?` (`extdeps/gpu/types.dag:120`) beside `figure_citations: List<FactCitation>`; P1's `rtx_pro_6000_workstation_cuda_core_standing: CitedFigureStanding = CitedToAuthority {…}` at `extdeps/gpu/nvidia.dag:127`. Candidate identity is already fixed by `gpu_catalog_row_eq` (`extdeps/gpu/types.dag:132`).
18. Construction-over-validation — `DESIGN.md:383`: *"Stronger than catching a wrong state is making it **unwritable — correctness by construction, not validation.**"*
19. All three PRO 6000 rows (Workstation, Max-Q, Server) are `package: PcieAddInCard` in `extdeps/gpu/nvidia.dag`; the two SXM rows carry `tdp_watts: none` and are the reference-only rows.
20. Rate carrier exists — `std/measure.dag:1546` `TokensPerSecond = Measure<Frequency, One, Nat>`, `tokens_per_second` `:1548`, `tokens_per_second_count` `:1552`, `MilliTokensPerSecond` `:1560`. Verified at `a7724b72999`.
21. Standing coproduct and its entry points — `gunbc/serving/gpu_bench_warm_standing.dag:53-57`; `gpu_bench_warm_run_admission` `:208`, `gpu_bench_warm_capacity_check` `:280`, `_from_capacity_ok` `:303`, `gpu_bench_warm_standing_after_replay` `:320`, `gpu_bench_warm_standing_with_ninfer_log` `:362`. `ServingPerformanceSubject` is `sole_constructor` at `gunbc/spark/serving_performance_subject.dag:77`; `AxisNotApplicable | AxisUnread` at `:321-322`.
22. Subject-bound-verdict precedent — `SubjectBoundCacheFit { subject_key, allocation_plan, verdict }` at `gunbc/spark/serving_promotion.dag:120`; `ServingPromotionEvidence` `:162`; `ServingPromotionReceipts` `:177`; `promotion_evidence` `:193`; `serving_promotion_standing` `:217`; `cache_bound_to_resolved_plan` `:267`.
23. Seal binds records, not arms — `src/v1/stage0/src/compiler_tests.rs:3119` `sole_constructor_violation_outside_module` and `:3167` `constructor_call_admission_refuses_unlisted_cross_module_caller`; the module's own comment at `extdeps/external_authority.dag:25-26` says *"a seal binds records, not coproduct arms"*. Arms are constructed across module boundaries in the corpus today — `Present { value: … }` at `dag/gunbc/ollama_runtime_bundle.dag:81`, `:83`, `:209` against `type Optional` / `Present` declared at `std/optional.dag:3`/`:5`. (snappy's note cited these two tests as `:3118`/`:3168`; those lines are a bare `#[test]` and a `thread::Builder` — the fns are at `:3119`/`:3167`.)
24. `WarmValid` outside its module: 24 occurrences in `dag/test/claim/gpu_bench_warm_standing_witness.dag`, 2 in `dag/test/claim/gpu_bench_v2_receipt_inhabitance_witness.dag`, 1 prose mention in the failure-mode row — ALL match arms or prose, zero value constructions (`gpu_bench_warm_standing_witness.dag:280` ends in `=>`, a match arm). `GpuBenchWarmStanding` is not `sole_constructor`; `gpu_bench_warm_standing.dag` contains 0 `sole_constructor`/`admit_callers` hits. Measured at `a7724b72999` and re-confirmed at `origin/main` `c4dc2576872`.
25. Seal enforcement compares file paths, not module paths — `sole_constructor_construction_diags` at `src/v1/stage0/src/v1_compiler_infer.rs:17224` tests `decl.span.file == span.file`, while the diagnostic text says "defining module" (`src/v1/stage0/src/v1_std_core.rs:1119`). `admit_callers` instead gates by caller declaration coordinates — `constructor_admission_diags_from_lookup`, `v1_compiler_infer.rs:1737`, `caller_decl_coords` vs `admit_callers_entry_coords`. Today every `.dag` file holds exactly one module declaration (6,730 files / 6,730 declarations; the lone regex hit at `annotation_authored_at_a_grain_the_model_does_not_carry.dag:19` is prose inside a comment).
26. The warm-capacity failure-mode row — `dag/gunbc/recurring_failure_mode/benchmark_warm_capacity_violation_read_as_engine_performance.dag`, authored `de0fedf0005`, `gunbai-bot[bot]`, 2026-10-14 — states its CEILING as "through the admitted receipt constructor …" and its NEXT TRIGGER as "P4 selection consumes only WarmValid as a funded axis". (snappy's note put `cited_figure_read_obligation` at `external_authority.dag:68-74`; it is at `:57` — `:68-74` is the `ExternalModelScope` "no revision field" comment.)
