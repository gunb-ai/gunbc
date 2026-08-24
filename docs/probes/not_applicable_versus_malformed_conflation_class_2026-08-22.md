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

| gunbc#9075 | parse / §4c annotation adjudication | `source annotation names no subject` | an annotation genuinely lacking a following module item vs **parse never reaching module scope**, so the check cannot see one |

Four stages, four independent discoveries, one shape. That is a property of this corpus's diagnostic vocabulary rather than four coincidences.

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


---

## 6. A fourth specimen, and the first one that is CROSS-STAGE

*2026-08-24, from gunbc#9075's CI run 32725969809. Added because it extends the class in a way
the three above do not: the conflated arm is not in the producer that failed.*

### What happened

An invalid `else match` in `src/v1/04_types.dag` left braces unbalanced. That run's
`required-ci: parse` phase emitted **311 FAIL lines**, which partition into exactly three texts:

```
309  source annotation names no subject: no module item follows it. Move it above the declaration it describes.
  1  expected LBrace, found keyword 'match'      <- the actual defect, named precisely
  1  expected RParen, found Colon
```

### Why it is this class

§4c attaches annotations to module-scope declarations, and the check asks *does a module item
follow this annotation*. With braces unbalanced the parser never returns to module scope — so from
its view nothing follows anything, and the check fires **correctly by its own logic on a premise
that an earlier stage destroyed**. One reason symbol over two states with opposite owners:

- **MALFORMED** — the annotation really does dangle, and its author must move it;
- **NOT APPLICABLE** — the annotation is fine and *this check could not see its subject*, which is
  the parser's problem and nobody else's.

### What is new here

**The conflation is cross-stage.** The three specimens above are each one producer whose own
predecessor test returned "no". Here the check has no defective predecessor test — it has a
*poisoned input*, produced two stages upstream. §4's recognition rule still reaches it, but only if
you extend "what did the arm's predecessor test return" to include *did the pipeline that built my
input succeed*. That is a question no arm currently asks.

**The remediation text is actively wrong, at scale.** *"Move it above the declaration it describes"*
instructs a reader to relocate 309 correctly-placed annotations. §3's specimens send an
investigation to the wrong subject; this one issues a confident, specific, wrong instruction 309
times. That is worse than an unhelpful diagnostic and worse than silence, because a reader acts on
the majority.

**And the correct diagnostic was already there.** This is the encouraging half: nothing needs to be
written. `expected LBrace, found keyword 'match'` was emitted, first try, exactly right — at a
signal-to-noise ratio of **1:154**.

### The repair

Not a split, and not a new symbol. **A parse failure should make §4c adjudication for that file
refuse rather than answer**: `cannot adjudicate annotations: parse did not reach module scope`.
The check does not need to be smarter; it needs an arm for *my input is not trustworthy*. That is
§5's fail-closed shape — refuse rather than assert a specific wrong cause — and it costs one
condition.

This also satisfies §5's caution from §4 above: it is not instrument work dressed as progress,
because it removes 309 wrong instructions rather than relabelling them.

### A method note, recorded because it nearly corrupted this entry

The first version of this specimen claimed **zero** of the 311 named the cause. That was false, and
it came from grepping for `annotation|expected item declaration|parse error` — none of which appear
in `expected LBrace, found keyword 'match'`. The pattern could not match a true positive, so the
absence was a property of the search.

The correction, and it generalises to any run failing with hundreds of diagnostics:

> **Partition the message texts before reading any of them.**

`... | sed 's|<prefix>||' | sort | uniq -c | sort -rn` turned 311 lines into three rows in one
command, and the two singletons at the bottom were the entire content. Reading top-down, or
grepping for what you expect, finds the 309 every time.

## 7. A fifth specimen — and the first one caught by the recognition rule rather than by reading the producer

*`src/v1/04_types.dag` `make_container_type`, found by smart-wolf-868 on 2026-08-24 while closing an
unrelated emission gate.*

Every specimen so far was found the hard way: someone read the producer and discovered the arm was
answering for two states. This one was found the way §4 says it should be — **from the shape of the
call**, without reading the diagnostic's implementation at all.

    match container_param_name(kind_name: kind_name, index: 0) {
      Present { value: param_name } => ...build the container type...
      Absent => KernelTypeBuild {
        ty: missing_kernel_container_profile_type(kind_name: kind_name),
        diagnostics: [kernel_container_profile_miss_diagnostic(kind_name: kind_name)]
      }
    }

`container_param_name` is a **lookup**. It returned `Absent`. The recognition rule fires on exactly
that, and it is right: the single `Absent` arm answers for two states with opposite owners —

    "this IS a container kind and its profile row is missing"  -> a defect; someone must add the row
    "this is not a container kind at all"                      -> not applicable; nothing is wrong

— and reports both as `kernel_container_profile_miss_diagnostic`, a *malformed* verdict. The symptom
was a synthetic name (`workspace_band_paints`) drawing a complaint that its profile row was missing,
for something that was never a container.

### What makes this specimen worth its own section: the repair was applied three times

The finder's fix was an `is_kernel_type` guard **at the call site**, returning a nominal type for
non-container names while preserving the missing-profile diagnostic for real container kinds. Correct,
and it works. But it was **the third such guard** — the `ReceiverElement`, `Key` and `Value` arms had
each already acquired one.

**That repetition is itself the diagnostic.** A distinction that must be re-established at every call
site is a distinction the callee should have drawn once. Three guards do not make the fourth caller
safe; they make the fourth caller's absence of a guard invisible, because nothing refuses — the
un-guarded call simply gets the wrong diagnostic again, exactly as this one did.

**So the structural repair is to split the reason at the lookup, not at its callers:** give
`container_param_name`'s absence two answers — `NotAContainerKind` and `ProfileRowMissing` — so that
`make_container_type` must match on which, and a caller that forgets **cannot compile**. That is §5's
construction move applied to this class: the conflated state stops being writable rather than being
re-detected per site, and the three accumulated guards are deleted rather than joined by a fourth.

**This is the general form of the repair for the whole class**, and this specimen is the one that
shows why the per-site fix is not merely weaker but actively concealing: each guard removes one
symptom and leaves the producer entitled to keep answering for both states.

### One caution recorded with it

The guard reads a **third** predicate into a decision that
`docs/probes/carrier_resolution_authority_fork_2026-08-24.md` censuses as already forked across two
unjoined authorities — `std.algebra` `kernel_algebra_profile_value` and `std.types`
`container_template_alias_rows`. Before the split above is built, it should be scoped against that
census rather than beside it: the canonical authority proposed there would carry container-resolution
as one field, which subsumes this repair. And that document's §4 constraint binds any such merge —
preserve every refusal **and the stage each refusal comes from**, since flattening moves which stage
refuses only for inputs that are already broken, so nothing goes red to announce it.
