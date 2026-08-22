# One reason symbol over two states: "malformed input" and "this path was never applicable", found three times in three stages

| | |
|---|---|
| what this is | a CLASS record, written after the third independent instance in one lane, so the fourth is recognised rather than rediscovered |
| how each was found | by reading the producer, in every case — never from the diagnostic's text |
| lane | the B4 product door (`v2.workflow.product_receipt_transport`), 2026-08-21 / 2026-08-22 |

---

## 1. The shape

A refusal reason stands over two states that a reader, and every consumer, will collapse:

- **MALFORMED** — the input really is wrong, and the author of the input must change it.
- **NOT APPLICABLE** — the input is fine, and *this strategy* has nothing to say about it. The capability is missing, or the caller took a path that was never going to answer.

They have opposite repairs. Malformed is fixed by the input's author; not-applicable is fixed by whoever owns the strategy, usually by writing the capability that does not exist yet. A single symbol over both routes every reader to the first repair, because a diagnostic that fires is read as an accusation against its subject.

**The tell is at the producer, not in the message.** In all three instances below the arm was reached through a *test that answered "no"* — a spine walk that found nothing, an unregistered lookup, an alternative that did not match — and the "no" was rendered as a rejection instead of as a miss.

## 2. The three specimens

| | stage | reason symbol | the two states it carried |
|---|---|---|---|
| gunbc#8801 | body lowering | (the let-form arm) | a malformed `let` vs a well-formed one this node could not lower |
| gunbc#8828 | body lowering / normalize | `body_lowering_reason_unsupported_form` | a malformed form vs the ordinary statement-form `let`, whose body is not in its own subtree at all |
| this record | parse | `parse_g0_tokens_remain` | a real partial parse leaving tokens over (`parse_production_prepared`) vs a **`PreparedVoidGrammar` beside a non-empty source** (`parse_module_prepared`) — where no parse was attempted and the fact is about grammar preparation, not about the source |

Three stages, three independent discoveries, one shape. That is a property of this corpus's diagnostic vocabulary rather than three coincidences.

## 3. Why it is expensive, in the currency this repository already counts

- **It sends the investigation to the wrong subject.** #8828's specimen grepped to 104 sites, because the "defect" was the ordinary spelling of a `let`. A reader who takes such a diagnostic at face value goes looking for a bad module and finds the whole corpus.
- **It hides a missing capability inside a bug report.** A not-applicable arm firing on the ordinary case is a *capability that was never written*, wearing a malformed-input costume. Counted as a defect it never ranks for building.
- **It survives counting.** Every one of these was invisible to a diagnostic tally: the count was right, and the partition it summarised was two different things.

## 4. The recognition rule, and the repair

**Recognition.** At the producer, ask what the arm's *predecessor test* returned. If the arm is downstream of a search, a lookup, an optional match or an alternative that came back `Absent`/`None`/`false`, it is a not-applicable arm and it needs its own reason — regardless of how the message reads.

**Repair.** Split the reason at the producers. It is not a behaviour change: both arms still refuse, they simply stop refusing under a shared name. Then the ledger partitions itself and names its own subject.

**And the caution that belongs beside the repair:** *a split is instrument work, not progress.* A better-labelled diagnostic is not a boundary advancing. It earns its keep only by naming a subject that then gets repaired.

## 5. The third specimen was resolved WITHOUT the split, and how

Worth recording because it is the cheaper move and it generalises: **check whether both arms are reachable before minting a symbol.** If one is dead on the path in question, the door has exactly one cause and can be named immediately.

Here the receipt already carried the answer. `overlap_residue_stage_from_prepared` returns `Absent` for `PreparedVoidGrammar`, so a void grammar cannot produce `parse_grammar_choice_overlap_residue` — and the refusal in hand carries **7** of them alongside the single `parse_g0_tokens_remain`, in the pending-then-rejected order that `parse_module_prepared` produces via `rejected_with_pending`. So the grammar in that run prepared as `PreparedModeled`, the void arm did not fire, and the cause is the genuine leftover-token arm in `parse_production_prepared` over a real member's source.

No symbol was minted. The evidence to eliminate one arm was already inside the artifact that reported the other.

*(The two producers still share one reason, and the class above still applies to them. What this section establishes is which one fired here, not that the conflation is harmless.)*
