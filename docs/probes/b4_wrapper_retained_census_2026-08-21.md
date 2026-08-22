# Boundary 4: what "wrapper-retained" actually is (2026-08-21)

| | |
|---|---|
| repository ref | current `main` plus the seven-boundary receipt lane (gunbc#8755) and this census |
| producer | `v2.workflow.body_lowering_retention_census` `retention_census`, invoked from inside `product_receipt_stage` on the same emission that produced the diagnostics |
| artifact | `target/product-receipt/b4-retention-census.txt`, written by the run |
| population | **ONE MODULE'S wrapper-retained observations — see §0. An earlier revision of this row claimed the whole population; that claim was false and is corrected below rather than deleted.** |

---

## 0a. SECOND CORRECTION, AND IT IS LARGER THAN §0: THIS POPULATION DOES NOT HOLD THE DOOR

§0 corrected the SUBJECT these numbers describe. This corrects why anyone was reading them at all.

**All 32 wrapper-retained observations are produced on an `Accepted` path.** `v2.compiler.body_lowering_fold` `body_lower_wrapper_retained_shell` returns `Accepted { value: shell, diagnostics: ... }` -- it is the corpus's single, deliberate producer of retention and it never refuses. They reach boundary 4's `Rejected` list only as carriage, through `v2.std.diagnostic` `rejected_with_pending`. The same holds for the 7 `parse_grammar_choice_overlap_residue`, which `v2.compiler.02_parse` `grammar_validate_and_analyze` attaches to an `Accepted`'s diagnostics field.

So of the 40 diagnostics at boundary 4, **39 are carried and exactly one rejects**. Closing every observation partitioned in this document would advance boundary 4 by zero boundaries. The measurements below stay real and reproducible; what is retired is the premise that they size the work.

Full reading, with what it destroys and what it does not claim: -> [b4 causal versus carried](b4_causal_versus_carried_2026-08-21.md).

## 0. CORRECTION (2026-08-21, same day): THIS IS ONE MODULE'S POPULATION, NOT THE CLOSURE'S

**The header of this document originally read "every wrapper-retained observation, whole — not a sample". That is false.** It was written from two subjects that happened to differ, and never tested against a subject that should have agreed. Four executed runs falsify it, and they are reported here rather than in a second document because a superseded claim and its correction must not be two artifacts.

| subject | modules in closure | B4 diagnostics | wrapper-retained families |
|---|---|---|---|
| `probe/solo.dag` — one type decl, **zero imports** | **1** | 11 = 7 / 3 / 1 `namespace_graft_body_dissolved_refused` | 2 `type_variant`, 1 `type_alias_rhs` |
| `probe/tv.dag` — the same type decl + a fn + 3 std imports | 4 | 11 = 7 / 3 / 1 `normalized_tree_..._not_normalized` | 2 `type_variant`, 1 `type_alias_rhs` |
| `probe/nov.dag` — a fn only, **the same 3 std imports** | 4 | 18 = 7 / 10 / 1 `fold_lowering_shape_invalid` | 7 `fn_type`, 3 `field_decl_block` |
| `src/v2/std/logic.dag` | 3 | **18 = 7 / 10 / 1 `fold_lowering_shape_invalid`** | **7 `fn_type`, 3 `field_decl_block`** |
| `src/v2/compiler/07_target_carriers.dag` | 29 | **18 = 7 / 10 / 1 `fold_lowering_shape_invalid`** | **7 `fn_type`, 3 `field_decl_block`** |

Three independent facts, each of which alone refutes the whole-population claim:

1. **A 1-module closure and a 4-module closure containing it report identically.** `solo` and `tv` agree exactly. The three std modules `tv` adds contribute nothing, so the report cannot be summing over the closure.
2. **Two closures sharing three of their four modules report 3 and 10.** `tv` and `nov` differ only in the entry file. A closure-wide census would have to contain the shared modules' contribution in both, so the smaller could not be smaller.
3. **A 3-module, a 4-module and a 29-module closure of entirely different content report byte-identical receipts.** `logic.dag`, `nov.dag` and `07_target_carriers.dag` agree to the diagnostic, including the terminal cause.

Fact 3 also names the mechanism. `nov` and `07_target_carriers` both **import `v2.std.logic`**, and running `logic.dag` as its own subject reproduces their receipt exactly. So the emitter refuses on the first module in the closure that fails and reports **that module's** diagnostics; the closure determines only which module is met first. `solo` and `tv` refuse on their own type declaration and never reach their imports, which is why adding imports changed nothing.

**What the correction destroys, stated explicitly so nothing downstream keeps standing on it:**

- **§2's "five missing producers" is one module's content**, not the compiler's population. `00_compile.dag`'s 32 observations across 5 families describe whichever single module that closure refuses on. The ranking built from 10/7/7/7/1 ranks that module's declarations.
- **§3's explanation is the wrong cause.** It attributed the two subjects' disagreement to overlapping-but-distinct closures. They disagree because they refuse on different modules. The section's *conclusion* — that neither figure is a corpus census — survives and is in fact stronger than it was argued: the figures are not closure censuses either.
- **`dag_surface_fn_type` "going DOWN 7 → 1 on the larger closure"** was never a population shrinking. The two runs were reporting two different modules.

**What survives untouched:** every count in this document is a real, reproducible measurement of a real module. Nothing here is fabricated and nothing needs re-measuring — what was wrong is the SUBJECT the numbers were attributed to, which is a labelling error, not a measurement one. The five families are genuinely unlowered productions; there is simply no evidence here about how many such observations the corpus holds.

**What is NOT established, and is not guessed at here:** the precise selection rule. "First module that fails" fits all five runs, but resolution order was not read out of the implementation, and no run was constructed to distinguish "first in dependency order" from "first in some other order". A lane that needs the rule must read it, not inherit it from this paragraph.

**The consequence for anyone sizing boundary 4: the receipt cannot currently size it.** A refusal that reports one module is a correct fail-closed refusal — the line stops at the first defect, which is the factory model working — but it is not a census, and this document mistook the one for the other. Sizing the real population needs an emitter that continues past a refused module and accumulates per-module, which is a change to the emitter, not to this census.


---

## 1. The question, and why the reason string is the wrong subject

Boundary 4 of the product receipt refuses with 40 diagnostics on the compiler's own closure, of which **32 carry `body_lowering_reason_wrapper_retained_emitted`**. The tempting task is "fix wrapper-retained".

That reason is ONE name over SEVERAL distinct absent producers. `body_lowering_fold_note` says so in its own words: an out-of-scope *emitted identity* with no registered producer lands wrapper-retained — and "an emitted identity" is a family, not an instance. A lane pointed at the reason string repairs whichever family it meets first and reports the count moving, which reads as progress on the population and is progress on one member of it.

So the partition key is the **emitted production identity the fold refused to lower**, joined with the retained node's own shape. That is the grain at which "a missing producer" is a thing someone can go and write.

**No pipeline file was edited to obtain this.** The wrapper-retained diagnostic already carries `at: node_locus(node: n)`, whose anchor holds the retained node itself, so the production identity is reachable from existing evidence through `v2.extdeps.languages.dag` `parse_production_emitted_identity_optional`. This is a read of what the diagnostic already carries, not an enrichment of `body_lowering_fold.dag`.

---

## 2. THE PARTITION — the module that `src/v2/compiler/00_compile.dag`'s 107-module closure refuses on

*(§0 corrects the subject of this section: these are one module's observations, reached through that closure, not the closure's.)*

```
families: 5
observations: 32

  10  dag_surface_type_variant        [Type/Conj]  (example arity 2)
   7  dag_surface_type_alias_rhs      [Type/Conj]  (example arity 2)
   7  dag_surface_field_decl_block    [Type/Conj]  (example arity 2)
   7  dag_surface_field_init          [Type/Conj]  (example arity 2)
   1  dag_surface_fn_type             [Type/Conj]  (example arity 2)
```

**Five missing producers, not one.** The largest is `dag_surface_type_variant` at 10 of 32 — but the distribution is flat rather than dominated: the top four are 10/7/7/7, so no single producer closes even a third of the population. That is a materially different plan from what the total suggested.

### 2.1 The finding the partition made visible, and it reframes the prescription

**Every one of the 32 is a TYPE-SURFACE production, and none is an expression body.** `type_variant`, `type_alias_rhs`, `field_decl_block`, `field_init`, `fn_type` — all five families, all 32 observations, at node shape `Type/Conj`.

The reason is called *body* lowering, and the standing prescription attached to it in the corpus is the general body producer — the MVP `03_body_producer` being fixture-bound rather than lowering real ingested **fn bodies**. That prescription is aimed at a population this one does not contain. Nothing here is a fn body; these are declaration-surface constructs.

Stated at the grain the evidence supports: this does not show the general body producer is unnecessary, and it does not show these five would not be swept up by it. What it shows is that **the 32 are not evidence for it**, and a lane that took "32 wrapper-retained" as the case for building it would be citing a population that is not the one the prescription names. Whether the type surface needs its own producers is a modelling question this census raises and cannot answer.

---

## 3. The 29-module control, and why the families are NOT a global census

*(§0 supersedes this section's CAUSE. Its conclusion holds and is strengthened; its explanation — overlapping-but-distinct closures — is wrong. The two subjects refuse on different modules.)*

Subject `src/v2/compiler/07_target_carriers.dag`:

```
families: 2
observations: 10

   7  dag_surface_fn_type             [Type/Conj]  (example arity 2)
   3  dag_surface_field_decl_block    [Type/Conj]  (example arity 2)
```

| family | 29-module closure | 107-module closure |
|---|---|---|
| `dag_surface_fn_type` | 7 | **1** |
| `dag_surface_field_decl_block` | 3 | 7 |
| `dag_surface_type_variant` | 0 | 10 |
| `dag_surface_type_alias_rhs` | 0 | 7 |
| `dag_surface_field_init` | 0 | 7 |

**`dag_surface_fn_type` goes DOWN, 7 → 1, on the larger closure.** A count that falls as the population grows is the tell that these are not nested subsets: `07_target_carriers.dag` is not inside `00_compile.dag`'s import closure, so the two are overlapping-but-distinct module sets, not a small and a large view of one thing.

The consequence binds anyone reading these numbers: **each partition is a per-closure population, and neither is a corpus census.** Differencing them tells you the families are closure-dependent; it does not give a repository-wide total, and summing them would double-count whatever the two closures share. A corpus-wide figure needs a subject that is the corpus, which this receipt can take but has not.

---

## 4. What this does NOT carry, and why it is not a gap to close

**Source file and exact `DeclarationRef` are absent, deliberately.** The diagnostic's locus anchors the retained node, not its declaration, and the emission is over the whole closure as one program, so module attribution would come from an ancestor the diagnostic does not carry. Obtaining it means the producer carrying a declaration anchor — an edit to `body_lowering_fold.dag`, a load-bearing pipeline file.

That edit is not made, and the reason is not cost. The column existed to support a join between this population and the v1 seed's rustc E0308 board, and that join was ruled non-decisive in the same adjudication that named the column: matching declarations establish nothing, because one function can independently expose a missing v2 body producer *and* a v1 representation mismatch. Only an intervention decides the coupling, and an intervention needs **one** declaration found by any means — including by hand from a production identity above — not an attribution column over all 32.

A later lane that needs the anchor should make that change with a concrete consumer to point at, rather than inheriting a speculative one from here.

**The census depends on `symbol_lexeme`, a host-intercepted bridge**, so like the receipt's cause partition it is available to the interpreted transport and would not survive a compiled realization unchanged.

---

## 5. Evidence

Five claims in `src/v2/test/claim/workflow/body_lowering_retention_census_test.dag`, all constructed fixtures — the partition's ability to tell two absent producers apart is a property of the partition, not of what the emitter refused today.

- two productions at one node shape stay two families (**the claim the lane rests on**)
- one production observed repeatedly is one family with a count, not N families of one
- one production at two node shapes stays two families — the shape is part of the key, not decoration
- the observation total is preserved across every partition
- the family key names both halves

The census is written **from inside the transaction that produced the diagnostics**. A census rebuilt afterwards by re-running the emitter would be a second observation of a second run. A failed census write **refuses the stage** rather than leaving the file absent — an absent file would make the receipt understate a population it had actually observed, which is the silent narrow this repository forbids in the same breath as the silent widen.

---

## 6. Reproducing

```
./target/release/gunbc run --source-root dag --source-root src/v2 \
  --entry src/v2/workflow/product_receipt_transport.dag \
  --function run_seven_boundary_product_receipt
cat target/product-receipt/b4-retention-census.txt
```

Any other subject — this is how a third closure's families are produced:

```
./target/release/gunbc run --source-root dag --source-root src/v2 \
  --entry src/v2/workflow/product_receipt_transport.dag \
  --function run_product_receipt_for_entry --arg entry=<path.dag>
```

---

## 7. A GRAMMAR FINDING, deliberately carrying NO COUNT

This is filed separately from every partition above because it is a fact about the **grammar and the body-lowering dispatch**, established by reading both, and it is **not** supported by any population figure in this document. §0 explains why no count in here can be attached to it. It is recorded with its evidence so it keeps its value and claims nothing more.

`v2.compiler.body_lowering_fold` `body_lower_production_emitted` classifies each emitted production into preserved / deferred / pass-through / routed, and anything unmatched falls through to `body_lower_wrapper_retained_shell`. The preserved set is `body_lower_is_metadata_preserved_emitted`, a **hand-written list of twelve symbols**.

Laid against the grammar's production table in `v2.extdeps.languages.dag`, the declaration-surface region is fully classified except for four symbols, whose immediate neighbours in that same table are all preserved:

| production | grammar expression reaches | classified today |
|---|---|---|
| `dag_surface_type_expr` | terminals only | **preserved** |
| `dag_surface_fn_type` | `type_expr` | falls through |
| `dag_surface_type_variant` | `type_expr` | falls through |
| `dag_surface_type_alias_rhs` | `type_variant` | falls through |
| `dag_surface_field_decl_block` | `type_expr` (via the field-decl helper) | falls through |
| `dag_surface_field_init` | **`expr`** | falls through |

Two things follow, and only the first is about the four.

**The four cannot contain an expression.** Their grammar expressions reach `type_expr` and terminals and nothing else — checked through `dag_grammar_field_decl_block_expr` → `dag_grammar_field_decl_list_helper` → `dag_grammar_field_decl_expr` for the one case where it is not immediate. They are the same pure type surface the preserved list already recognizes.

**`field_init` is a different class and must not be swept in with them.** Its production is `ident ":" expr` — structurally the same shape as `dag_surface_arg`, which is already **pass-through** rather than preserved. Preserving `field_init` would strand a real expression inside an unlowered wrapper. Anything that treats these five as one repair is wrong on the fifth.

**The underlying shape, which is the part worth acting on.** The preserved list is a second representation of a fact the grammar already carries, and it fell behind because it must be maintained by hand — a §5 validation-where-construction-was-available tell. The discriminating criterion needs no new field on the production row: *does this production's grammar expression reach the `expr` nonterminal*. Deriving the classification from that would make the list unable to fall behind, rather than detecting when it has.

**What is NOT claimed here:** that any of this is worth doing, how many observations it would close, or that preserving is the correct disposition for the four. The first two need a population this document cannot supply (§0). The third is a modelling question about body lowering that reading the dispatch does not answer. No edit to `body_lowering_fold.dag` is proposed on this evidence.
