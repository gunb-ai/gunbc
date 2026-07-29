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

## 1. The result

Phase A proved the Clone requirement does not originate in a `.dag` body. B0 asks the
next question at the grain where it *does* originate — the emitter's lowering decision —
and every site classifies:

| requirement cause | unique sites | occurrences | share |
|---|---:|---:|---:|
| `TargetApiRequirement` | 19 | 168 | 28.5% |
| `OwnedDeconstructionRequirement` | 7 | 63 | 10.7% |
| `CloneSharedRequirement` | **53** | **359** | **60.8%** |
| `NoRequirement` | 0 | 0 | — |
| `Unresolved` | **0** | **0** | — |
| **TOTAL** | **79** | **590** | 100% |

Acceptance, met to the unit: **79 unique sites**, **590 replicated occurrences**, **zero
unresolved**, **no default or "other" bucket absorbing anything** (the `Unresolved`
variant is real and fires — §6 proves it by perturbation), and Phase A's 35 former
unknowns all classified through the real compilation path.

## 2. The load-bearing correction: most of the population *is* ownership-removable

The brief rules that routing R1/R3 into ownership work is the wrong axis, and adds:

> More decisively: most of the population is not removable by ownership work at all — see
> verified facts below.

**Measured, that is the other way round.** `CloneSharedRequirement` — the one cause the
brief assigns to `emitter-ownership-defork` — is **359 of 590 occurrences (60.8%)** and
**53 of 79 sites (67.1%)**. The two representation-imposed causes together are 231
occurrences (39.2%).

This does not disturb the brief's *ruling* — B0 was authorized regardless, and the split
is exactly what B0 was told to measure rather than infer. It does change the sizing that
B1/B2 inherit: **B2 ("join the ownership verdict at genuinely clone-producing seams
only") is not a tail-cleanup pass, it is the majority of the population**, while the
representation contract B1 models covers 39.2%. Reporting rather than improvising, per
the brief's own instruction.

The split by census family is unusually clean, which is why it is worth trusting:

| root family | occurrences | cause composition |
|---|---:|---|
| R2 vector method bounds | 168 | **100% `TargetApiRequirement`** |
| R3 container clone bounds | 161 | **100% `CloneSharedRequirement`** |
| R1 clone bound on type param | 261 | 198 `CloneShared` (75.9%) / 63 `OwnedDeconstruction` (24.1%) |

R2 and R3 are each a single cause end to end. Only R1 is mixed — and its mix is exactly
the head-extract lowering (63) against everything else.

## 3. What was measured, by execution

Fresh `gunbc` + `cssl_assemble`, `CSSL_STD_SEED_LINK=1`, empty shim, the authority
`Cargo.toml`, `cargo build --release --lib` with `--message-format=json` so each
diagnostic carries an exact span rather than a scraped line. Causes load from
`dag/tools/e0599_emitter_decision_census.dag` and root families from Phase A's
`dag/tools/e0599_probe_census.dag`, both through `gunbc` — **no second table in the
joiner**.

### 3.1 The instrument reproduces the census exactly

| module | measured | census baseline | Δ |
|---|---:|---:|---:|
| 04_infer | 88 | 88 | 0 |
| 05_emit | 93 | 93 | 0 |
| 05_eval | 91 | 91 | 0 |
| 06_translate | 93 | 93 | 0 |
| emit_host | 96 | 96 | 0 |
| emit_module | 93 | 93 | 0 |
| materialization_carriers | 81 | 81 | 0 |
| **TOTAL** | **635** | **635** | **0** |

**No-perturbation control.** This PR adds a `dag/tools/` module, a witness and docs, all
of which enter the whole-tree index that `gunbc compile` builds. Re-emitting and
rebuilding `04_infer` at the final tree state yields **E0599 = 88**, identical to the
measurement run — so the additions do not perturb emitted output and the census is valid
at the sha it is stamped with.

R1+R2+R3 = 590, and the per-emitted-file distribution reproduces Phase A's to the unit
(`algebra` 413, `diagnostic` 112, `optional` 18, `node` 14, `fidelity_carriers` 12,
`translate` 8, `collection` 6, `witness` 6, `staging` 1).

### 3.2 Phase A's 35 unknowns are all resolved

Phase A carried 35 diagnostics as `unknown` — 14 behind a standalone `parse_module`
refusal on `std/node.dag`, 21 in three off-roster files. B0 reads the **emitted artifact
at the exact reported span** instead of re-parsing the source standalone, so the refusal
class does not arise: all 79 sites resolve to an enclosing emitted fn, and **all 79
resolve to a `.dag` source module** (joined on each module's own `module` declaration, not
a filename heuristic — `v2.compiler.translate` lives in `06_translate.dag`).

### 3.3 The receiver expression is the discriminator

rustc's primary span highlights the failing method segment exactly, so the receiver is the
balanced postfix expression ending at the preceding `.`. Across all 590 occurrences the
whole population reduces to **seven** distinct receiver shapes, each one a literal the
emitter concatenates:

| lowering operation | emitter authority | sites | occurrences | cause |
|---|---|---:|---:|---|
| `IdentReferenceClone` | `05_emit_rust.dag:6320`/`:6330` via `sharing.clone_value` | 31 | 214 | CloneShared |
| `DerefCloneWholeValue` | `:6434`/`:6959`/`:7587`/`:7893`/`:8748` via `sharing.deref_clone` | 21 | 137 | CloneShared |
| `FreeMonoidEmptyTest` | `:7859` (`emit_native_freemonoid_match`), `:8715` tco fork | 11 | 98 | TargetApi |
| `FreeMonoidTailIterate` | `:7817` (`freemonoid_tail_let_from_fm`) | 8 | 70 | TargetApi |
| `FreeMonoidHeadExtract` | `:7838`, `:8694` tco fork | 7 | 63 | OwnedDeconstruction |
| `FieldAccessClone` | `:6438` (`emit_typed_field_access`, non-owned arm) | 1 | 8 | CloneShared |

### 3.4 Phase A's emitter-site table was incomplete — the dominant producer was missing

Phase A receipted 17 `sharing.*` template applications. **None of them is the FreeMonoid
match lowering**, which produces 231 of the 590 occurrences (39.2%) from
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

`v1_generic_params_needing_clone_bound` produces **17** bounds in the `04_infer` closure
(reproducing Phase A exactly) and **20** across the union of all seven, once `v1_rt.rs` —
the hand-written seed runtime, not emitter output — is excluded.

Of those 20, only **3** host any of the 79 defect sites, and in every one the helper bound
a **different type parameter** than the site requires:

| emitted fn | helper bound | site requires | disjoint |
|---|---|---|---|
| `fold_list` | `<T, A: Clone>` | `T: Clone` | yes |
| `fold_list_right` | `<T, A: Clone>` | `T: Clone` | yes |
| `fold_node_topdown` | `<A, R: Clone>` | `A: Clone` | yes |

So the census column reads **covered 0 / not_covered 79** — and that zero is structural,
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
   `UseSiteVerdict = MoveWhole | MoveField | Borrow | CloneShared { decision }`. The 53
   `CloneSharedRequirement` sites are that verdict's inhabitants, so B2 joins an existing
   concept rather than minting one.

**Not every `CloneShared` site is the same distance from a fix.** The census records, per
site, whether the emitter already *has* a move arm gated on an ownership predicate:

| ownership alternative | sites | occurrences | what B2 must do |
|---|---:|---:|---|
| `EmitterArmPresent` | 32 | 222 | flip an existing predicate — `moves_by_value` (`:6318`) or `base_is_owned` (`:6435`) already select a bare move; the clone is the else-arm |
| `NoEmitterArm` | 21 | 137 | **add** an arm — `:6959`/`:7587` clone on both branches, `:7893`/`:8748` is unconditional, and `05_emit_rust.dag` contains **zero** `Rc::try_unwrap`/`make_mut` (the seed runtime `v1_rt.rs` uses that shape, the emitter never emits it) |

So of the 359 ownership-removable occurrences, 222 are gated by a verdict that already
exists and 137 need new emitter capability. That is a sizing fact B2 should inherit rather
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

`dag/test/claim/e0599_emitter_decision_census_witness_test.dag` — **19 green**.

The three RED controls are the load-bearing half. Perturbing the classifier's fail-closed
arm into the absorbing default DESIGN §5 forbids (`Absent => CloneSharedRequirement`)
turns **exactly** those three red and nothing else:

```
FAIL e0599_b0_red_out_of_scope_method_refuses
FAIL e0599_b0_red_unnamed_receiver_shape_refuses
FAIL e0599_b0_red_empty_receiver_refuses
```

Restored, 19/19 green. A zero in the `Unresolved` row is therefore a measurement, not a
dead arm.

**Second control — the joiner's field contract.** The classification line carries exactly
seven fields, and the joiner refuses on any mismatch rather than padding. Perturbing the
authority to emit an eighth field produces a located refusal naming the offending key and
exits 1:

```
REFUSED: cause authority returned 8 fields (expected 7) for key ('clone', '(*__fm)[0]')
```

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
each modelled with a supplemental `T: Clone`. #7399 is model-only and pending operator
merge.

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
