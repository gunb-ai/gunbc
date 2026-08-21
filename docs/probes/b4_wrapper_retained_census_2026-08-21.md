# Boundary 4: what "wrapper-retained" actually is (2026-08-21)

| | |
|---|---|
| repository ref | current `main` plus the seven-boundary receipt lane (gunbc#8755) and this census |
| producer | `v2.workflow.body_lowering_retention_census` `retention_census`, invoked from inside `product_receipt_stage` on the same emission that produced the diagnostics |
| artifact | `target/product-receipt/b4-retention-census.txt`, written by the run |
| population | every wrapper-retained observation, whole — not a sample |

---

## 1. The question, and why the reason string is the wrong subject

Boundary 4 of the product receipt refuses with 40 diagnostics on the compiler's own closure, of which **32 carry `body_lowering_reason_wrapper_retained_emitted`**. The tempting task is "fix wrapper-retained".

That reason is ONE name over SEVERAL distinct absent producers. `body_lowering_fold_note` says so in its own words: an out-of-scope *emitted identity* with no registered producer lands wrapper-retained — and "an emitted identity" is a family, not an instance. A lane pointed at the reason string repairs whichever family it meets first and reports the count moving, which reads as progress on the population and is progress on one member of it.

So the partition key is the **emitted production identity the fold refused to lower**, joined with the retained node's own shape. That is the grain at which "a missing producer" is a thing someone can go and write.

**No pipeline file was edited to obtain this.** The wrapper-retained diagnostic already carries `at: node_locus(node: n)`, whose anchor holds the retained node itself, so the production identity is reachable from existing evidence through `v2.extdeps.languages.dag` `parse_production_emitted_identity_optional`. This is a read of what the diagnostic already carries, not an enrichment of `body_lowering_fold.dag`.

---

## 2. THE PARTITION — subject `src/v2/compiler/00_compile.dag`, 107 modules

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
