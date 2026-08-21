# Boundary 4: 39 of the 40 diagnostics are carried, ONE rejects (2026-08-21)

| | |
|---|---|
| repository ref | `main` at `e336acc4f0`, plus the fold-lowering split in this lane |
| what establishes this | a read of two producers' return constructors, grep-verifiable by symbol; **not** a new measurement |
| the measurements it reinterprets | the executed partitions already banked in [the seven-boundary product receipt](seven_boundary_product_receipt_2026-08-21.md) §2 and §3, and [the b4 wrapper-retained census](b4_wrapper_retained_census_2026-08-21.md) |

---

## 0. The claim, and the exact grain it is made at

The receipt reports boundary 4 as `Refused -- emitter rejected the admitted population, diagnostics: 40`, partitioned:

```
   7  parse_grammar_choice_overlap_residue
  32  body_lowering_reason_wrapper_retained_emitted
   1  body_lowering_reason_unsupported_form
```

Every reader of that block — including the two documents that built rankings on it — has read the partition as a partition **of the refusal**. It is not. It is a partition of the diagnostic *list* the refusal carries, and a `Rejected` outcome's list in this pipeline holds two different kinds of thing.

**Two of the three causes are produced on `Accepted` paths.**

- `v2.compiler.body_lowering_fold` `body_lower_wrapper_retained_shell` returns `Accepted { value: shell, diagnostics: … }`. It is, by the corpus's own design, the *single* producer of retention (`body_lowering_rejection_propagation_note` made it so deliberately, by deleting the second, inferential one). It never refuses. **All 32.**
- `v2.compiler.02_parse` `grammar_validate_and_analyze` attaches `grammar_overlap_residue_diagnostics` to the `diagnostics` field of an `Accepted`, and `grammar_validate_for_parse` passes it through as `residue`. It never refuses on that row. **All 7.**

They reach the `Rejected` list through `v2.std.diagnostic` `rejected_with_pending`, which appends the pending carriage ahead of the genuine refusal. So:

> **The B4 door is held by exactly ONE diagnostic per subject.**
>
> - `src/v2/compiler/00_compile.dag` (107 modules): the single `body_lowering_reason_unsupported_form`.
> - `src/v2/compiler/07_target_carriers.dag` (29), `probe/nov.dag` (4), `src/v2/std/logic.dag` (3): the single `fold_lowering_shape_invalid`.

## 1. What this destroys, stated so nothing downstream keeps standing on it

- **The b4 wrapper-retained census sizes a non-blocking population.** Its own §0 already corrected the census from *the closure's* population to *one module's*. This is the second and larger correction on the same numbers: whichever module they describe, closing all 32 advances boundary 4 by **zero** boundaries. The census's measurements remain real and reproducible; what is retired is the reason anyone was reading them.
- **The receipt's `10/7/7/7/1` missing-producer ranking is a ranking of carried observations.** Its §3.1 called `body_lowering_reason_wrapper_retained_emitted` the population "any B4 work should be denominated against" and then struck that sentence for being one module's rather than a closure's. It should have been struck twice.
- **`parse_grammar_choice_overlap_residue` being 7 on every subject is now explained twice over.** The census's mechanism — a fork between two parse entry points, invariant because it is a property of the grammar — stands unchanged and is corroborated by this reading: a constant carried through an `Accepted` cannot be indexed by anything the refusal did.

## 2. What is NOT claimed

- **This is not a new execution.** It reinterprets executed partitions by reading which constructor each producer returns. That read is decidable by symbol (`body_lower_wrapper_retained_shell`, `grammar_validate_and_analyze`) and is the kind of claim `feature:cited-symbol-resolution` can check; it is not a receipt, and it is filed as a reading rather than as a measurement.
- **It does not establish that the two door-holders are the *only* refusing arms across all subjects.** It establishes that on the five subjects measured to date, the causes reported alongside the refusal include exactly one that a refusing producer can emit. A subject whose partition shows two or more non-carried causes would be a genuine surprise worth stopping on, exactly as a non-7 residue would be.
- **It does not size the residual work.** After the door-holder on a subject is repaired, the next refusal is whatever the emitter meets next; that is a number only the receipt can take, and taking it needs the emitter to be re-run per repair.

## 3. The consequence for the instrument itself

The receipt's B4 line renders `EmitterRejected { causes: emitter_diagnostic_causes(ds) }` over the whole list, so it confidently answers *which diagnostics appeared* while every reader asks it *what closed the door*. That is not a labelling slip in the render — the distinction is genuinely destroyed at `rejected_with_pending`, which takes `pending` and `rejected` and returns one flat `NonEmptyDiagnostics` with no record of the seam.

**No repair to the receipt is proposed here, and the reason is that the cheap repair is the wrong one.** Re-deriving causality in the render by listing which reason symbols are "carried" would be a second representation of a fact the producers already carry in their return constructors — validation standing exactly where construction was available, and a list that must be maintained by hand as producers are added. The construction move is for the refusal to carry its cause distinguishably, which is a change to the shared `Outcome`/`NonEmptyDiagnostics` carriers and is not made under this lane's brief.

**Next-rung trigger:** a diagnostic carriage that distinguishes pending from causal at the seam that already knows the difference. Until then the receipt's B4 partition is honest about what it counts and silent about what rejects, and this document is what stands between that silence and the next ranking built on it.
