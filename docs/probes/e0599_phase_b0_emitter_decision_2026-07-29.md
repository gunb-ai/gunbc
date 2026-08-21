# E0599 Phase B0 — emitter-decision census (census only)

**Session:** tidy-swift-81 · **Status:** census only; no model authority and no emitter
change lands in this PR.
**Brief:** P-fn Phase B0 (operator, 2026-07-29) — supersedes phases B and C of
`docs/plans/e0599-implementation-proposal.md` and supersedes Phase A's own
recommendation.
**Predecessor:** `docs/probes/e0599_phase_a_body_evidence_2026-07-28.md` (#7389).
**Receipt:** `docs/probes/e0599_phase_b0_emitter_decision_receipt_2026-07-29.tsv`
**Authority:** `dag/tools/e0599_emitter_decision_census.dag`
**Witness:** `dag/test/claim/e0599_emitter_decision_census_witness_test.dag`

---

## 0. What producer this measures — read first

**Everything below characterizes the V1 *seed* emitter** (`src/v1/05_emit_rust.dag`,
executed through `gunbc`) and what it synthesizes when emitting v2 modules.

Per the operator design ruling of 2026-07-29, **`CompilerFixedPoint` recenters on the V2
emitter**: the v1 stage0 runtime may temporarily *execute* the v2 emitter as `.dag`, but
the v1 emitter stops *deciding* the generated Rust — emitter authority exits v1 first,
interpreter execution later.

So, stated plainly so this census cannot be misread later:

* these 167 diagnostic locations and 600 occurrences are **diagnostic comparison only**;
* they are **not `CompilerFixedPoint` progress**, and this census carries **no dependency
  edge into `CompilerFixedPoint`** and **does not gate v2 bootstrap**;
* they are **not a v2 sizing authority**. The v2 emitter is a *different producer*, built
  from target-model rows consumed by the shared translate/serialize path, so no row here
  transfers to it by assumption — some may, some are artifacts of seed lowering templates
  v2 does not have.

What it *is* load-bearing for: a V1 seed-emitter **symptom inventory**, a **candidate**
bootstrap-adapter inventory, and a historical comparison point.

It is **not** the bootstrap adapter queue. That queue is the intersection

```
this census  ∩  exact native-v2-emitter bootstrap closure  ∩  defects that demonstrably
                                                              block generation 1
```

and until the direct v2 Rust door exists, **that intersection is unknown** (operator
verdict, 2026-07-29). Nothing here authorizes a seed adapter on its own.

---

## 1. The result

Phase A proved the Clone requirement does not originate in a `.dag` body. B0 asks the
next question at the grain where it *does* originate — the emitter's lowering decision —
and every site classifies:

| requirement cause | diagnostic locations | occurrences | share of occurrences |
|---|---:|---:|---:|
| `TargetApiRequirement` | 48 | 168 | 28.0% |
| `OwnedDeconstructionRequirement` | 18 | 63 | 10.5% |
| `CloneSharedRequirement` | **101** | **369** | **61.5%** |
| `NoRequirement` | 0 | 0 | — |
| `Unresolved` | **0** | **0** | — |
| **TOTAL** | **167** | **600** | 100% |

**Read the left column carefully.** 167 counts *distinct diagnostic locations in emitted
artifacts* — keyed on emitted-artifact digest + file + primary span + method + receiver
expression. It is **not** a count of distinct seed-emitter *decisions*: this probe cannot
observe an emitter decision without instrumenting `src/v1/05_emit_rust.dag`, which is
deliberately unchanged. Treat this as an **emitted receiver-shape census**; it does not
enumerate exact adapter tasks. §3.3 gives the full 78 → 167 attribution — occurrences and
every per-cause occurrence count are unchanged, so the refinement regrouped rows and moved
no diagnostic between causes.

Measured, and then **re-measured from scratch** after `origin/main` was merged again. Two
shas matter and they name different things, so the receipt carries both — and the receipt
is the authority for them, not this prose:

- `measured_corpus_sha` = **`749ca01e73b`** (#7415) — the last commit touching `src/`, i.e.
  the seed emitter and the AST it runs over. This is what the numbers are *about*.
- `base_sha` = **`94ea41e4ee1`** — the commit the measurement *ran at*.

That merge changed the seed binary itself (`gunbc` sha256 `55a06adf…` → `d0f681a9…`,
because #7415 compiles into it), so the census was **not** argued to be unaffected: the
binaries were rebuilt and all seven modules re-emitted and rebuilt into a fresh work dir.
The result is **byte-identical** to the previous stamping — every rollup and every per-site
row — which is a purity check the argument alone would not have given. §3.1 reports the
earlier corpus move and attributes every unit of it. **The brief's
acceptance figures — 79 sites / 590 occurrences — were met exactly at the sha the brief
was written against**, and the corpus has since moved; §3.1 reports both and attributes
every unit of the difference. Zero unresolved, no default or "other" bucket absorbing
anything (the `Unresolved` variant is real and fires — §6 proves it in two directions),
and Phase A's 35 former unknowns all classified through the real compilation path.

## 2. The load-bearing correction: the clone seam is the majority — *candidate*, not removable

The brief rules that routing R1/R3 into ownership work is the wrong axis, and adds:

> More decisively: most of the population is not removable by ownership work at all — see
> verified facts below.

**Measured, the population split is the other way round.** `CloneSharedRequirement` — the
one cause the brief assigns to `emitter-ownership-defork` — is **369 of 600 occurrences
(61.5%)**. The two representation-imposed causes together are 231 occurrences (38.5%).
That share survived the corpus moving twice, and it survived the site-identity
refinement in §3.3 (every per-cause *occurrence* count is unchanged).

**What that does and does not establish** (operator verdict, 2026-07-29 — this section
previously overclaimed and the overclaim was the single most important correction):

This census does **not** execute the per-site ownership verdict. It records which emitter
predicate or alternative arm is *associated* with a site. It does **not** determine whether
that site is `SoleOwner`, at last use, movable, or safely borrowable. So the supported
conclusion is:

> 369 occurrences are produced at seams owned by the ownership/lowering lane. Their actual
> removability **remains to be proved.**

Split more precisely, and this is the form B1/B2 should inherit:

| | occurrences | what is established |
|---|---:|---|
| `EmitterArmPresent` | 230 | a move arm exists — **not** that the ownership verdict should select it |
| `NoEmitterArm` | 139 | no alternative arm exists — and a new lowering **may still require `Clone`** under sharing |

A rationale withdrawn in the same pass: two lowering rows previously cited `Rc::make_mut`
as evidence a copy-free alternative exists. It is not. `make_mut` is clone-on-write, is
declared `where T: Clone`, and clones the pointee whenever another strong `Rc` is live —
so it **requires the very bound this census counts** and cannot evidence its removal.

Consequently **369 is not a predicted burn-down** and must not be used as one, and
`CloneSharedRequirement` means only *the seed emitter inserted a clone at this seam* —
never *the clone can safely be deleted*. That meaning is now fixed in the authority itself
(`e0599_cause_semantics_note`) rather than in prose that can drift from it.

What the finding *does* change is where the population sits: the clone seam is the
majority of V1 symptoms, so it is the larger **candidate** surface — while remaining
unproved, and explicitly not a reason to make ownership work a prerequisite for anything.

The split by census family is unusually clean, which is why the classification is worth
trusting even though the removability question is open:

| root family | occurrences | cause composition |
|---|---:|---|
| R2 vector method bounds | 168 | **100% `TargetApiRequirement`** |
| R3 container clone bounds | 163 | **100% `CloneSharedRequirement`** |
| R1 clone bound on type param | 269 | 206 `CloneShared` (76.6%) / 63 `OwnedDeconstruction` (23.4%) |

R2 and R3 are each a single cause end to end. Only R1 is mixed — and its mix is exactly
the head-extract lowering (63) against everything else.

## 3. What was measured, by execution

Fresh `gunbc` + `cssl_assemble`, `CSSL_STD_SEED_LINK=1`, empty shim, the authority
`Cargo.toml`, `cargo build --release --lib` with `--message-format=json` so each
diagnostic carries an exact span rather than a scraped line. Causes load from
`dag/tools/e0599_emitter_decision_census.dag` and root families from Phase A's
`dag/tools/e0599_probe_census.dag`, both through `gunbc` — **no second table in the
joiner**.

### 3.1 Instrument soundness, and what four merges did to the population

Two measurements are reported, because they say different things and neither substitutes
for the other.

**(a) Soundness, at the sha the brief was written against.** The instrument reproduced the
frozen Phase A census **Δ=0 on every module** — 04_infer 88, 05_emit 93, 05_eval 91,
06_translate 93, emit_host 96, emit_module 93, materialization_carriers 81, **total 635**,
with R1+R2+R3 = **590** across **79** unique sites. That is a fact about that sha and it
is what establishes the instrument is sound before being used to conclude anything. It is
also the brief's acceptance figure, met to the unit.

**(b) The merged head (`914da873`), after four PRs landed mid-review.** The population
moved:

| module | merged head | pre-merge | Δ |
|---|---:|---:|---:|
| 04_infer | 94 | 88 | +6 |
| 05_emit | 97 | 93 | +4 |
| 05_eval | 97 | 91 | +6 |
| 06_translate | 97 | 93 | +4 |
| emit_host | 100 | 96 | +4 |
| emit_module | 97 | 93 | +4 |
| materialization_carriers | 81 | 81 | 0 |
| **TOTAL** | **663** | **635** | **+28** |

R1+R2+R3 goes 590 → **600**, and unique sites go 79 → **78** (both under the *old* site
identity — the identity refinement in §3.3 came later and is attributed separately there;
quoting the old-identity numbers here keeps this delta comparable). Sites falling while
occurrences rise needs explaining, so it is explained rather than reported:

| change | sites | cause |
|---|---:|---|
| `target_collection_fold_list_from_node` (`06_translate`) — `DerefCloneWholeValue` ×2, `IdentReferenceClone` ×2 | **−4** | #7324 rewrote this function; its defect sites are gone |
| `coproduct_nullary_inhabitant_lookup_step` (`node_query`) — `IdentReferenceClone` | **+2** | new generic code from #7324 |
| `decode_node_list_item_step` (`fold_assembly`) — `DerefCloneWholeValue` | **+1** | new generic code from #7324 |
| **net** | **−1** | 79 − 4 + 3 = **78** |

Every unit is attributed to #7324, and **no site disappeared for any reason other than the
function being rewritten**. This is corpus growth and corpus churn, not instrument drift:
the mechanism is unchanged, the population tracks the code.

**This was an unplanned generalization test, and the classifier passed it.** The new sites
are code the taxonomy was never designed against. Their receivers —
`(*decode_item(item_node.clone()))`, `inhabitant`, `(*acc.clone())`,
`(*coproduct_nullary_inhabitant_lookup_all(…))` — all classify with **existing** rows, and
**`Unresolved` stayed at zero**. A fifth new diagnostic, `lookup` on
`Rc<im::HashMap<…>>`, is correctly dropped by the family filter before the classifier sees
it (it is not `clone`/`is_empty`/`iter`, so it is R6, outside R1/R2/R3 scope).

**What did not move — and why that is NOT evidence.** `TargetApiRequirement` (old-identity 19 sites /
168 occurrences) and `OwnedDeconstructionRequirement` (7 / 63) are identical across both
measurements, and the entire delta sits inside `CloneSharedRequirement`. It is tempting to
read that as the representation-imposed causes being intrinsically stable, which is what
the mechanism analysis predicts. **That reading is not supported, and the attribution table
above is what refutes it:** every changed site — all four removed and all three added — is
a clone-family lowering operation (`DerefCloneWholeValue`, `IdentReferenceClone`) and
therefore `CloneSharedRequirement`. #7324 rewrote a fold; it did not touch collection
method usage at all.

So those two counts held still because **nothing that changed could have moved them**.
The observation is *entailed* by the change set, not independent of it, and it carries no
evidential weight about intrinsic stability in either direction. Separating the two
readings needs a landing that actually perturbs vector-method usage. Recorded this way
because an over-read stability claim is exactly the kind of thing a later sizing decision
gets built on (caught in review by calm-badger-682).

The per-emitted-file distribution is otherwise unchanged (`algebra` 413, `diagnostic` 112,
`optional` 18, `node` 14, `fidelity_carriers` 12, `collection` 6, `witness` 6, `staging`
1), with `node_query` 12 and `fold_assembly` 6 joining it.

**No-perturbation control.** This PR adds a `dag/tools/` module, a witness and docs, all of
which enter the whole-tree index that `gunbc compile` builds. Re-emitting and rebuilding
`04_infer` at the final tree state reproduced its count exactly, so the additions do not
perturb emitted output.

### 3.2 Phase A's 35 unknowns are all resolved

Phase A carried 35 diagnostics as `unknown` — 14 behind a standalone `parse_module`
refusal on `std/node.dag`, 21 in three off-roster files. B0 reads the **emitted artifact
at the exact reported span** instead of re-parsing the source standalone, so the refusal
class does not arise: all sites resolve to an enclosing emitted fn, and **all
resolve to a `.dag` source module** (joined on each module's own `module` declaration, not
a filename heuristic — `v2.compiler.translate` lives in `06_translate.dag`).

### 3.2b DEFECT, 2026-08-19 — this document reports TWO DIFFERENT `139`s and at least one is wrong

**Marked unreliable rather than resolved or dropped** (operator instruction, `smart-ram-730`; recorded
by `quick-lynx-620` during B1 carrier reconciliation).

The figure `139` appears in two different partitions of the same 369-occurrence `CloneShared`
population, with *different site counts*:

| partition | row | sites | occurrences |
|---|---|---:|---:|
| receiver shape (§3.3) | `DerefCloneWholeValue` | 39 | 139 |
| ownership arm (§ below) | `NoEmitterArm` | 20 | 139 |

Both partitions sum to 369 independently (222 + 139 + 8; 230 + 139), so these are two partitions of one
population, not one fact cited twice — the equal cell value is a coincidence.

**Why it cannot be a benign coincidence.** `NoEmitterArm`'s cited authority lines
(`:6959`/`:7587`/`:7893`/`:8748`) are a proper *subset* of `DerefCloneWholeValue`'s
(`:6434`/`:6959`/`:7587`/`:7893`/`:8748`). A proper subset carrying an identical occurrence count
requires `:6434` to contribute exactly zero occurrences while being listed as an inhabited authority.
That cannot hold. So one of: a count is wrong, `:6434` is misattributed, or the two rows do not mean
what their columns say.

**Not resolved here, and deliberately not dropped.** The instrument that produced these counts was a
shell scaffold (`docs/probes/e0599_emitter_decision_census.sh`) since deleted, so the measurement is
not reproducible at this tree. A number nobody can reproduce is marked unreliable rather than adopted
or quietly removed — dropping it would erase the evidence that the contradiction exists.

**Consumers must not size work against either 139.** Both are emitted-Rust *occurrence* counts under a
lowering-operation partition. Neither is a carrier-site count. Measured carrier populations at
`e0c5e25444`: the B1 carrier `RequiredTraitWitness` had **18** references across 8 files and its three
variants **34** across 5 files, corpus-wide. `src/v1/trait_bound_witness.dag` cites "139 counted
occurrences" for `DerefCloneWholeValue`; that citation inherits this defect.

**Dissolution:** re-running a reproducible census over the same population, at which point this section
is replaced by the corrected partition rather than amended.

### 3.3 The receiver expression is the discriminator

rustc's primary span highlights the failing method segment exactly, so the receiver is the
balanced postfix expression ending at the preceding `.`. Across all 600 occurrences the
whole population reduces to **six inhabited** receiver shapes (of seven declared rows —
`FreeMonoidCatchallBind` is declared and explicitly uninhabited), each one a literal the
emitter concatenates:

| lowering operation | emitter authority | sites | occurrences | cause |
|---|---|---:|---:|---|
| `IdentReferenceClone` | `05_emit_rust.dag:6320`/`:6330` via `sharing.clone_value` | 60 | 222 | CloneShared |
| `DerefCloneWholeValue` | `:6434`/`:6959`/`:7587`/`:7893`/`:8748` via `sharing.deref_clone` | 39 | 139 | CloneShared |
| `FreeMonoidEmptyTest` | `:7859` (`emit_native_freemonoid_match`), `:8715` tco fork | 28 | 98 | TargetApi |
| `FreeMonoidTailIterate` | `:7817` (`freemonoid_tail_let_from_fm`) | 20 | 70 | TargetApi |
| `FreeMonoidHeadExtract` | `:7838`, `:8694` tco fork | 18 | 63 | OwnedDeconstruction |
| `FieldAccessClone` | `:6438` (`emit_typed_field_access`, non-owned arm) | 2 | 8 | CloneShared |

**A receiver-shape class is not a producer decision.** `DerefCloneWholeValue` above bundles
boxed field extraction, record spread, and match-scrutinee lowering under one row because
they emit a *similar Rust receiver shape* — not because the emitter made one decision. That
distinction is why this document is an emitted receiver-shape census and stops short of
enumerating adapter tasks.

#### Site identity, and the full 78 → 167 attribution

Site identity was `(file, line, method, receiver_type)`, which excluded the primary span
column, the receiver expression, and the emitted artifact's content. Two distinct clone
expressions on one generated line with the same receiver type therefore collapsed into a
single row and inherited the first expression's cause. The key is now

```
emitted artifact digest + file + primary span (line:col-line:col) + method + receiver expr
```

The receiver expression is *in the key*, so a heterogeneous group is **unwritable** rather
than merely detected — construction over validation (§5). The residual assertion is a
backstop against a future key change, not the mechanism.

Every step measured, not asserted:

| identity | locations | Δ | what the added discriminator separates |
|---|---:|---:|---|
| `(file, line, method, receiver_type)` — the old key | 78 | — | — |
| `+ receiver_expr` | 81 | **+3** | three groups each held **two** distinct expressions and were classified by the first — the predicted defect, real |
| `+ primary span` | 90 | **+9** | the same expression twice on one line, at different columns |
| `+ emitted artifact digest` | 167 | **+77** | copies of one path across module closures whose **bytes differ** |

The three genuinely-heterogeneous groups, named:

```
src/extdeps_communication_fidelity_carriers.rs:53  clone  ->  left.carried | right.carried
src/v2_std_algebra.rs:826                          clone  ->  candidate    | item
src/v2_std_algebra.rs:837                          clone  ->  candidate    | item
```

The +77 is concentrated in **4 of 10** emitted paths — those whose content differs between
module closures, where collapsing them had been an unverified assumption. The other six
paths are byte-identical across closures and collapse correctly (verified directly:
`v2_std_algebra.rs` hashes identically in `04_infer`, `06_translate` and `emit_host`).

**Occurrences are unchanged at 600, and every per-cause and per-operation occurrence count
is identical to the pre-refinement measurement.** So this is a regrouping: no diagnostic
changed cause, and the §2 shares are untouched.

### 3.4 Phase A's emitter-site table was incomplete — the dominant producer was missing

Phase A receipted 17 `sharing.*` template applications. **None of them is the FreeMonoid
match lowering**, which produces 231 of the 600 occurrences (38.5%) from
`emit_native_freemonoid_match` and its byte-identical TCO fork. Those three literals
(`:7817`, `:7838`, `:7859`) are added here.

The fork itself is a §3 observation, recorded not fixed: `freemonoid_nonempty_branch_body`
(`:7833`) and `freemonoid_tco_nonempty_branch_body` (`:8689`) carry byte-identical
`head_let` and `is_empty` literals, while sharing only `freemonoid_tail_let_from_fm`.

### 3.5 The flagship specimen

```
source    fn fold_list<T, A>(xs: FreeMonoid<T>, empty: A, cons: fn(A, T) -> A) -> A
            match xs { Empty => empty  Cons { head: h, tail: t } => ... }
                                                      src/v2/std/algebra.dag:50

emitted   pub fn fold_list<T, A: Clone>(mut xs: Rc<Vec<T>>, ...) -> A {
            { let __fm = xs.clone();
              if __fm.is_empty() {            // T: Clone — TargetApi        (im :180/:315)
                break empty
              } else {
                let h = (*__fm)[0].clone();   // T: Clone — OwnedDeconstruction (im :1838)
                let t = Rc::new((*__fm).iter() // T: Clone — TargetApi        (im :180/:383)
                  .skip(1).cloned().collect());
```

One source `match`, three diagnostics, **two different causes** — and the legacy helper
bound `A`, while every one of them needs `T`.

## 4. The legacy helper cross-reference: the bounds are disjoint, not merely insufficient

`v1_generic_params_needing_clone_bound` produces **18** bounds in the `04_infer` closure
and **22** across the union of all seven, once `v1_rt.rs` — the hand-written seed runtime,
not emitter output — is excluded. (At the sha the brief was written against these were
**17** and **20**, matching the brief's acceptance figure and reproducing Phase A exactly;
#7324 moved them along with everything else — see §3.1. The receipt is the authority.)

Of those 22, only **3** host any defect site, and in every one the helper bound
a **different type parameter** than the site requires:

| emitted fn | helper bound | site requires | disjoint |
|---|---|---|---|
| `fold_list` | `<T, A: Clone>` | `T: Clone` | yes |
| `fold_list_right` | `<T, A: Clone>` | `T: Clone` | yes |
| `fold_node_topdown` | `<A, R: Clone>` | `A: Clone` | yes |

So the census column reads **covered 0 / not_covered 78** — and that zero is structural,
not a counting artifact. The helper's rule (a) is driven by *value-clone of the return*
and lands on the return's type param; the sites are driven by *element and collection
access* and land on the element's type param.

This confirms the brief's sequencing from the other side: **C1 must be additive** because
the two sets do not intersect at all, and **C3 can never fire on a representation rule
alone** — no representation contract will ever produce rule (a)'s 11 bounds, because they
are not about a representation.

## 5. Fields a sharing model would need (captured, not committed)

Per the brief, B0 commits no carrier. The census records, per site: source fn · source
semantic construct · exact source type-parameter declaration · selected target
representation · selected lowering operation · ownership verdict where relevant · required
target trait · requirement cause · external authority · legacy-helper coverage. Two
observations for B1, both grounded:

1. **The tree has already ruled the carrier relationship.** #7324's
   `dag/extdeps/languages/rust/derive_contracts.dag` declares
   `TargetDeriveSupplementalGenericBoundRequirementSet` a `Scaffold` whose
   `dissolves_to: SingleAuthority` binds
   `v2.std.compilers.target_model.RequiredTraitWitness`. A *separate* clone-bound carrier
   would therefore be a §3 fork against an already-declared dissolution target.
2. **`CloneShared` is already named in the ownership lane.**
   `docs/plans/emitter-ownership-defork.md` designs
   `UseSiteVerdict = MoveWhole | MoveField | Borrow | CloneShared { decision }`. The 52
   `CloneSharedRequirement` sites are that verdict's inhabitants, so B2 joins an existing
   concept rather than minting one.

**Not every `CloneShared` site is the same distance from a fix.** The census records, per
site, whether the emitter already *has* a move arm gated on an ownership predicate:

| ownership alternative | sites | occurrences | what B2 must do |
|---|---:|---:|---|
| `EmitterArmPresent` | 32 | 230 | flip an existing predicate — `moves_by_value` (`:6318`) or `base_is_owned` (`:6435`) already select a bare move; the clone is the else-arm |
| `NoEmitterArm` | 20 | 139 | **add** an arm — `:6959`/`:7587` clone on both branches, `:7893`/`:8748` is unconditional, and `05_emit_rust.dag` contains **zero** `Rc::try_unwrap`/`make_mut` (the seed runtime `v1_rt.rs` uses that shape, the emitter never emits it) |

So of the 369 clone-seam occurrences, 230 are gated by a verdict that already
exists and 139 need new emitter capability. That is a sizing fact B2 should inherit rather
than rediscover.

**What this column is not.** It records *which ownership predicate governs the site* and
*whether a move arm exists*, not the per-site boolean `make_decision` verdict
(`SoleOwner` or not). Producing that boolean means running `ownership.dag`'s analysis at
emit time, which needs either an instrumented emitter — deliberately out of scope, the
fence keeps `05_emit_rust.dag` unmodified — or a re-implementation of the analysis here,
which would be exactly the §3 fork this lane exists to remove. B2 consumes that verdict
by construction anyway, so it is the right place to read it; B0 stops at naming the
predicate. Stated so the column is not read as more than it is.

The `FieldAccessClone` row makes the ownership seam mechanical: the emitter picks a
**move** at `:6435` (`base_is_owned`) and this **clone** at `:6438` otherwise, so the copy
fires exactly when the ownership verdict says not-owned — and that verdict comes from
`owned_bindings`, the ad-hoc third set the defork doc says "is not built by
`ownership.dag` at all" and "can never contain" a parameter.

## 6. Green by execution, with a discriminating RED

`dag/test/claim/e0599_emitter_decision_census_witness_test.dag` — **27 green**, of which
**8** are RED controls, in the two classes named below (destination and routing) plus the
scope/rollup single-authority controls added on `review 44371`.

The RED controls are the load-bearing half. Perturbing the classifier's fail-closed
arm into the absorbing default DESIGN §5 forbids (`Absent => CloneSharedRequirement`)
turns **exactly** these three *destination* controls red and nothing else:

```
FAIL e0599_b0_red_out_of_scope_method_refuses
FAIL e0599_b0_red_unnamed_receiver_shape_refuses
FAIL e0599_b0_red_empty_receiver_refuses
```

Restored, 27/27 green.

**What `Unresolved = 0` does and does not claim.** It is measured *given these
predicates*. A destination perturbation like the one above proves the fail-closed arm
**can** fire; it cannot prove that every unclassifiable input **reaches** it, because a
permissive predicate upstream means some shapes never arrive. That gap was real and was
found in review: `e0599_receiver_is_field_access` accepted any dotted receiver, so
`pair.0` would have classified as `FieldAccessClone` instead of refusing.

It matters because the shapes genuinely differ on the axis this census reports: named
field access is gated by `base_is_owned` (`:6435`) and so is `EmitterArmPresent`, while
tuple projection at `:6421`/`:6422`/`:6412` applies `clone_value` **unconditionally** and
is `NoEmitterArm`. Measured, **zero** tuple projections exist in the population — the
complete dotted-receiver set is `left.carried` and `right.carried`, 4 occurrences each,
one site — so **no number moved**. But latent is not absent: the predicate would have
mis-attributed the moment one appeared, and nothing would have said so.

Fixed, and pinned by a **routing** control rather than a destination one:

```
FAIL e0599_b0_red_routing_tuple_projection_first_refuses
FAIL e0599_b0_red_routing_tuple_projection_second_refuses
```

— which is what reverting the predicate to its permissive form produces, and nothing else
reds. So the census's frontier is now asserted by execution in both directions: the arm
fires, and unclassifiable shapes route to it.

**Second control — the joiner's field contract.** The classification line carries exactly
seven fields, and the joiner refuses on any mismatch rather than padding. Perturbing the
authority to emit an eighth field produces a located refusal naming the offending key and
exits 1:

```
REFUSED: cause authority returned 8 fields (expected 7) for key ('clone', '(*__fm)[0]')
```

**Third control — the scope set and the rollup roster are loaded, not restated.** This one
corrects a claim this receipt itself was making falsely. `review 44371`
(cursor/composer-2.5) found that the joiner carried
`R123 = {"R1CloneBoundOnTypeParam", "R2VectorMethodBounds", "R3ContainerCloneBounds"}` as a
hand-set while the receipt's `family_authority` line asserted the scope filter went
"via gunbc, no second table". The claim was false, and the same pattern held for the
cause-rollup order, which restated all five variant labels.

The failure direction is the one that hides: rename a family in `.dag` and the filter
silently matches fewer sites — a smaller census, no error, the DESIGN §5 absorbing
fallback wearing "narrow" instead of "widen". Both now load from their authorities
(`e0599_write_mechanistic_root_family_labels_blob`,
`e0599_write_rollup_cause_labels_blob`), whose labels derive from the same
`e0599_root_family_label` / `e0599_requirement_cause_label` the classification uses, so
**zero** family or cause literals remain in the joiner. An empty roster and a measured
cause missing from the roster both refuse, located.

Proven not to move the measurement: re-running the census with the wired joiner over the
same work dir reproduces the previous TSV **byte-for-byte** (`diff` clean). Pinned by
`e0599_b0_mechanistic_scope_is_exactly_the_three_roots`,
`e0599_b0_red_mechanistic_scope_excludes_the_tail_families`,
`e0599_b0_rollup_roster_covers_every_cause_the_classifier_can_emit`, and
`e0599_b0_rollup_roster_retains_the_unresolved_arm`.

**Fourth control — an unresolved site stops the line.** `review 44380`
(codex/codex-default) caught that the joiner *counted* unresolved sites and then exited
**0**, so a newly unrecognized emitter shape would have produced a green, incomplete
census — indistinguishable from a complete one to any consumer. The count was reported;
the line did not stop.

Fixed the way DESIGN §5's factory model prescribes, and the ordering is the point: the
full census is emitted **first**, so every deficit is inspectable and located (the
sanctioned stopped-line audit "reports, it does not green"), and *then* the probe refuses
with a non-zero exit naming each unclassified site. Proven by perturbation — neutering
`e0599_receiver_is_bare_ident` so bare identifiers reach `LoweringUnidentified`:

```
exit code: 1
REFUSED: 31 site(s) reached the Unresolved arm — the census above is INCOMPLETE and must
not be consumed as a measurement. No lowering row in tools.e0599_emitter_decision_census
names these shapes, so their requirement cause is unknown, not absent. Located:
src/extdeps_communication_fidelity_carriers.rs:46 source_medium_carried
(method='clone' receiver='carried'); src/v2_std_algebra.rs:76 list_reverse …
```

…with every per-site row still printed (78 under the site identity in force when that
perturbation was run; the identity refinement came after). Measured `unresolved` is **0**
today, so the arm
is dead-in-corpus and kept as the fail-closed backstop — which is exactly why it needed a
perturbation to prove it fires at all.

## 7. Corrections to the brief's verified facts (it asked for re-verification)

1. **The linked crate is `im` 15.1.0, not `im-rc`.** The probe manifest authority
   (`dag/tools/self_host_curated_seed_linked_harness.dag:34`) declares
   `im = { version = "15.1", features = ["serde"] }`, `Cargo.lock` carries `im 15.1.0`
   with no `im-rc` entry, and every emitted module opens
   `use im::{vector as vec, HashMap, OrdSet as BTreeSet, Vector as Vec}`. The two crates
   are generated from one source (`Arc` vs `Rc`), so **every line number in the brief is
   correct to the line** — impl at `:180`, `is_empty` `:315`, `Clone` `:1686`, `Debug`
   `:1701`, `PartialEq` `:1716`; `iter` is `:383` and `Index` is `:1838`. Only the crate
   name changes, and for a citation that is the whole point.
2. Everything else re-verified as stated: `RequiredTraitWitness` is
   `Ord | Hash | Eq` with no `Clone` variant and `TargetCollectionRealization` carries no
   bound field; `emit_fn_def` computes `clone_param_names` at `:5205` and renders the
   signature at `:5214-5218` **before** the body at `:5227`/`:5233`; the helper's two
   structural rules are `return_is_bare_generic && !body_is_param_ref && param == ret` and
   `v1_generic_param_used_as_collection_element`.

## 8. The #7324 authority question (raised, not decided)

The brief asks whether #7324's keying is intentional. Measured: `derive_contracts.dag`
declares **both** `rust_1_93_vec_trait_implementations_authority` (std `Vec`) and
`im_15_1_0_vector_trait_implementations_authority` — but
`rust_vec_supplemental_generic_bound_contracts()` keys `Debug`, `Clone` and `PartialEq` on
the **std** one, and the `im` row has **zero production consumers** (referenced only from
`derive_contracts.dag` itself and the #7324 test).

Against the real impl bounds that under-derives on two of the three:

| derive | std `Vec<T>` | `im::Vector<A>` | agrees? |
|---|---|---|---|
| `Clone` | `T: Clone` | `A: Clone` (`:1686`) | yes |
| `Debug` | `T: Debug` | `A: Clone + Debug` (`:1701`) | **no** |
| `PartialEq` | `T: PartialEq` | `A: Clone + PartialEq` (`:1716`) | **no** |

Since every emitted `Vec` *is* `im::Vector`, a contract keyed on std `Vec` yields
`T: Debug` where `T: Clone + Debug` is required.

**Answered.** Raised with `vivid-swift-837`, which confirmed (2026-07-29) that the std
keying was *not* intentional — there is no separate production std `Vec` representation —
and corrected it in **#7399**: the inapplicable std/serde `Vec` authorities are deleted,
exact `im` 15.1.0 line authorities added, and `Debug`/`PartialEq`/`Serialize`/`Deserialize`
each modelled with a supplemental `T: Clone`. #7399 is model-only and **merged on 2026-07-29**; this census consumes it
(see the partial dissolve-on in §9 and the authority's own note).

**Consequence for this module, recorded as a dissolution trigger:** the local
`E0599ExternalAuthority` rows here are an *interim* citation. P-fn sequences **after**
#7399 and must not mint a parallel Vec authority — when #7399 lands, delete
`e0599_im_vector_inherent_impl` / `_index_impl` / `_clone_impl` and import the
`derive_contracts` rows instead. That trigger is carried on the module's own note, not
only here.

## 9. Scope honoured

* Census only — **no model authority and no consumer flip lands here**; no carrier is
  pre-committed (§5 records the fields, and names the two existing carriers B1 should
  argue against, without choosing).
* `src/v1/05_emit_rust.dag` is **unmodified** — the emitter is read as authority, and its
  decisions are read off the emitted artifact it produced, never re-implemented.
* No second classification table: causes and root families both load through `gunbc` from
  their single authorities.
* `v1_type_param_needs_clone_bound` not grown; `RequiredTraitWitness` untouched; no new
  syntax walker; P-derive (#7324) untouched.
* No E0277/E0308 claims.

## 10. Side observations, recorded not fixed

* **A parse error in one `dag/tools/` module fails the whole-tree emit of unrelated
  entries, and not always legibly.** With this module mid-edit, `06_translate`/`emit_host`
  reported the parse error honestly, but `05_eval` instead reported 29 *misleading*
  `unlisted import use` errors against its own dotted-path references, which resolve fine
  once the unrelated module parses. The loud-but-wrong locus is the LOUDNESS gap class the
  import-strip diagnosis already names.
* **The grammar accepts a leading `&&`/`||` on a continuation line but rejects leading
  `==`/`!=`** (`expected expression, found EqEq` / `found Ne`). Hit twice while authoring;
  cost is cosmetic (the single-line spelling is ordinary), so this is an inconsistency
  report, not a blocker.

*Dissolve-on: when B1 lands the sharing model on the materialization axis, the rows in
`dag/tools/e0599_emitter_decision_census.dag` migrate to it and this file's §5 folds into
that plan; §§1–4 stay as the receipt.*
