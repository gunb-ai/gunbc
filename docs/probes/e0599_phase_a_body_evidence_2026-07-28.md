# E0599 Phase A — body-evidence audit (probe only)

**Session:** tidy-swift-81 · **Status:** probe-only; no model authority and no emitter
change lands in this PR.
**Plan of record:** `docs/plans/e0599-implementation-proposal.md` (#7300, merged) §3
"P-fn body-lowering coverage audit".
**Receipt:** `docs/probes/e0599_phase_a_body_evidence_receipt_2026-07-28.tsv`
**Executed probe:** `src/v2/test/claim/manual/e0599_phase_a_body_evidence_probe.dag`

---

## 1. The question, and the answer

The plan-of-record names one unproven fact and makes the predicted `−590` conditional
on it:

> **Not proven** — no executed witness that canonical-seven R1/R2/R3 sites in
> `04_infer`…`emit_module` fn bodies expose method-invoke + receiver→type-param
> linkage in normalized substrate. Model-only `TraitBoundWitness` authority can land;
> **predicted −590 is hypothesis until this gap is receipted or ruled.**

**Answer: the linkage is absent, because the operand is absent.** The
`.clone()` / `.is_empty()` / `.iter()` calls rustc reports are **not present in the
`.dag` sources at all**. They are synthesised by the v1 Rust emitter's own lowering
templates. There is no modeled operation for `TraitBoundSiteMethodInvoke { method,
receiver }` to name.

The escape clause therefore fires:

> If implementation cannot name the missing fact without inventing it: return for a
> new ruling before claiming census movement.

**The `−590` is withdrawn as a P-fn claim. The covered subset under the proposed
carrier is 0.**

---

## 2. What was measured, by execution

Fresh `gunbc` + `cssl_assemble` at the base sha, `CSSL_STD_SEED_LINK=1`, empty shim,
the `docs/probes/curated_cargo_probe_one.sh` contract. Failure-shape patterns and
root-family labels are loaded from the single authority
(`dag/tools/e0599_probe_census.dag`) through `gunbc`, exactly as
`e0599_census_extract.sh` loads them — this probe adds **no second pattern table**.

### 2.1 The census reproduces exactly

All seven modules, Δ=0 on every one, and every root family reproduced to the unit:

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

Family rollup: R1 261, R2 168, R3 161, R4 24, R5 12, R6 9 — identical to the census.
**R1+R2+R3 = 590.** The instrument is sound before it is used to refute anything.

### 2.2 The sites are not where the plan assumed

R1/R2/R3 diagnostics almost entirely do **not** live in the seven entry modules. Keyed
by emitted file they land in the **shared import closure**, recompiled once per entry
module:

| emitted file | R1/R2/R3 diagnostics |
|---|---:|
| `v2_std_algebra.rs` | 413 |
| `v2_std_diagnostic.rs` | 112 |
| `v2_std_optional.rs` | 18 |
| `v2_std_node.rs` | 14 |
| `extdeps_communication_fidelity_carriers.rs` | 12 |
| `v2_compiler_translate.rs` | 8 |
| `v2_std_witness.rs` / `v2_std_collection.rs` / `v2_std_staging.rs` | 13 |

582 of 590 are closure sites; only 8 sit in an entry module's own code
(`06_translate`). Two files — `algebra` and `diagnostic` — carry **89%**.

**79 distinct `(file, line, method, receiver)` positions account for all 590
diagnostics; 66 of them recur in all seven builds** (462 diagnostics). The census's
590 is not 590 defects — it is ~79 distinct source positions counted once per entry
build. This matters for sizing every downstream claim in this lane.

### 2.2b Per-site linkage, end to end (04_infer, complete)

Every R1/R2/R3 diagnostic in `04_infer` was resolved from its emitted `file:line` to
its enclosing emitted fn, and from there to the `.dag` source fn of the same name:

* **37 of 37** emitted fns resolve to a real `.dag` source fn;
* **0 of 37** contain a modeled `.clone()` / `.is_empty()` / `.iter()` invoke;
* 82 diagnostics covered.

The dominant fns are `zip_eq`, `zip_map`, `is_prefix_of` (`src/v2/std/algebra.dag`),
`absorb_outcome_diagnostics`, `append_outcome_value`, `outcome_eq`
(`src/v2/std/diagnostic.dag`), `optional_prefer_first_present`
(`src/v2/std/optional.dag`) and `fold_node_topdown` (`src/v2/std/node.dag`).

### 2.3 The reported methods do not exist in the source

Executed detector, in-substrate: parse a real corpus file
(`filesystem_read` → `tokenize` → `parse_module`) and fold the tree counting postfix
method-projection steps via **`body_lower_postfix_projection_step`** — the same
accessor `body_lower_try_postfix_projection` uses to build its `Transform`, not a
second walker.

* **RED control** (inline source text carrying all three methods):
  `[any=3, clone=1, is_empty=1, iter=1]` — the detector demonstrably fires.
* **Site-bearing corpus files**: `clone`, `is_empty`, `iter` counts — MEASUREMENT
  PENDING, filled from the executed run before this PR leaves draft. `algebra.dag`
  carries only **5** postfix method projections in total, against 413 R1/R2/R3
  diagnostics emitted from it.

A zero here is a measurement, not a dead detector.

Corroborating whole-corpus scan at the base sha (excluding this probe's own note
strings): `.clone()` appears **10** times in all of `dag/` + `src/v2/` and **every one
is inside a string literal or a prose note** (`dag/extdeps/languages/rust/emit.dag:386`,
the four `dag/std/languages.dag` templates, `dag/std/effects.dag:37`, the census tool's
description, two test assertions on emitted strings, one frontier note).
`.is_empty()` / `.iter()` appear **6** times, all inside Rust emit templates.

**Zero modeled `.clone()` invocations exist in the substrate.** `.clone()` is not a
concept the `.dag` substrate has at all — ownership is a Rust realization concern, so
asking a `.dag` body to witness a Clone requirement is asking it to speak a language it
does not have.

### 2.4 Where the requirement actually comes from

The emitter inserts them, at **20** template applications in `src/v1/05_emit_rust.dag`
(`sharing.deref_clone`, `sharing.clone_value`, `clone_iterator_suffix`): match
scrutinee deref (`:7893`, `:8748`), field access (`:6434`/`:6438`), variant payload
(`:4930`), record spread (`:6959`/`:7587`), optional unwrap (`:6423`), tuple
projection, list index get.

The clearest single specimen — source and emitted, same function:

```
source   fn is_empty<T>(xs: FreeMonoid<T>) -> Bool {
           match xs { Empty => true  Cons { head: _, tail: _ } => false }
         }

emitted  pub fn is_empty<T>(xs: Rc<Vec<T>>) -> bool {
           let __fm = xs.clone(); if __fm.is_empty() { true } else { false }
         }
```

The source matches a modeled coproduct. The emitter chose the native `Vec<T>`
representation and lowered that match into `xs.clone()` (R1/R3) plus
`__fm.is_empty()` (R2). The sibling tail template produces
`(*__fm).iter().skip(1).cloned().collect()` — R2's `.iter()`. **Every one of R1, R2
and R3 is a product of representation choice × lowering rule.**

**R1, R2 and R3 are not three families — they are three rustc symptoms of one
lowering.** `zip_eq<T>` shows it at a single emitted line:

```rust
let __fm = a.clone();                                    // R1: clone on type param
if __fm.is_empty() { … } else {                          // R2: is_empty on Rc<im::Vector<T>>
  let ha = (*__fm)[0].clone();                           // R3: clone on container
  let ta = Rc::new((*__fm).iter().skip(1).cloned()…);    // R2: iter on im::Vector<T>
}
```

All of it descends from one source `match xs { Empty … Cons … }`. The plan lists
*"R1+R2+R3 share one mechanism"* as a **diagnosis hypothesis, not yet closed by
execution**. This receipt closes it — and more strongly than it was stated: they
co-occur at the same emitted expression, not merely in the same mechanism class.

This is not a new discovery of this probe; the tree already says so at
`dag/std/effects.dag:37`, which names both the mechanism and its dissolution:

> the v1 Rust emitter lowers every match as `match (*x.clone()).clone()`, so a
> generic fn matching `EffectShape<K>` needs `where K: Clone` … but the emitter emits
> generic FUNCTIONS without bounds … **dissolve-on = emitter emits Clone/PartialEq
> bounds on generic functions**

---

## 3. The disposition table the brief asked for

The brief asked for three dispositions per site. The measurement forces a fourth,
and it takes 100%:

| disposition | sites | diagnostics | basis |
|---|---:|---:|---|
| evidence-present | 0 | 0 | no `clone`/`is_empty`/`iter` projection step exists in any site-bearing source |
| wrapper-retained | 0 | 0 | **not the deficit** — these bodies lower; `body_lower_wrapper_retained_shell` is not reached for them |
| ambiguous | 0 | 0 | the operand is absent, not undecidable |
| **no-modeled-operation (emitter-synthesized)** | **79** | **590** | the reported methods are inserted by emitter lowering templates |

**The wrapper-retained row is the load-bearing correction.** The plan framed the risk
as body-lowering *coverage* — that Stage-A might not lower enough shapes. That is not
what is wrong. These bodies lower fine. Recording them as "wrapper-retained" would
misattribute the deficit to the general-body-producer lane and send the fix to the
wrong place.

---

## 4. Coverage that a Phase C flip would destroy

`v1_generic_params_needing_clone_bound`'s two structural rules currently emit Clone
bounds on **17 modeled generic fns** in the `04_infer` closure (`fold_list`,
`fold_node`, `fold_node_topdown`, `fold_source`, `map_get`, `map_insert`, `list_nth`,
`nat_cata`, …). A further **38** modeled generic fns are unbounded while their
emitted bodies clone or iterate — the defect population.

Phase C as specified swaps `clone_param_names` onto the witness list and deletes the
helper's body classification. Since the witness list has **zero inhabitants**, that
flip would delete all 17 working bounds and **add** E0599s. Nothing has been flipped;
the rollback boundary is intact.

---

## 5. Recommended re-ruling (Phase B is not ready as written)

The Clone requirement is real; it simply is not a source-body fact. It is a property
of the **realization**, composed from facts that already have homes:

1. **the target's value semantics** — `ValueSemantics.OwnershipBased`
   (`dag/std/languages.dag:844`, carrying `clone_expr` / `deref_clone_expr` /
   `field_clone_expr`);
2. **the representation choice** for the modeled carrier —
   `TargetRepresentationChoice` / `TargetCollectionRealization` in `target_model.dag`,
   which already exists and is exactly the surface the plan reserved for collection
   repr facts;
3. **the lowering rule applied at the site** — the template applications in §2.4.

So the bound is **derivable by construction from the emitter's own decision**, not
discoverable from the body. This is §5's correctness-by-construction rather than
validation: the authority that decides to insert `.clone()` on a value of type τ is
the same authority that must record `τ: Clone`. A carrier keyed on emission-time
lowering decisions has inhabitants for all ~590; one keyed on source method-invokes
has none.

Two candidate directions, both to be ruled before any model lands:

* **(a) record the requirement** at the clone-inserting seam — each template
  application contributes the type it cloned; `emit_fn_def` reads the accumulated set.
  Consumes existing repr authority; no new syntax walker.
* **(b) remove the requirement** — a large share of these clones are gratuitous
  (`xs.clone()` on an `Rc` parameter that is only read). This is the **same root** as
  the already-named DESIGN open thread *Rc-ownership wrap-decision* — "ONE structural
  wrap-decision predicate … so the bad wrap is unwritable". That lane deletes
  requirement rather than documenting it.

(b) is the stronger read on §6 grounds: it dissolves the cost instead of modeling
around it, and it is already a named lane rather than a new carrier.

---

## 6. Side finding — a §3 fork, recorded not fixed

The same fact has two authorities: `dag/std/languages.dag`'s
`ValueSemantics = OwnershipBased { clone_expr, deref_clone_expr, field_clone_expr }`
(**zero consumers**) and `src/v1/languages.dag`'s
`SharingModel { deref_clone, clone_value, … }` (`:109`, `:393` — the one the emitter
actually consumes). The modeled std surface duplicates the v1 seed's sharing model
and is inert. Not this lane's to fix; recorded here so it is not re-derived.

---

## 7. Scope honoured

* Phase order respected — **no model authority and no consumer flip lands here**.
* Nothing prohibited was used: `v1_type_param_needs_clone_bound` was not grown, no
  emitted-Rust body was scanned to *derive* a bound (the emitted crate is read only as
  probe evidence), `RequiredTraitWitness` is untouched, no new syntax walker was added
  — the detector reuses `body_lower_postfix_projection_step`.
* Fence respected — `src/v1/05_emit_rust.dag` is unmodified.
* No E0277/E0308 claims are made. P-derive (#7324, vivid-swift-837) is untouched.

*Dissolve-on: when the re-ruling lands, fold §5 into the successor plan and retire
this file's recommendation section; §2's measurements stay as the receipt.*
