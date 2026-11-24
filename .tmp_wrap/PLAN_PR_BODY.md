## Plan-only

Roadmap-authority change only: `dag/gunbc/roadmap/roadmap_authority.dag`. No `src/v2` lowering change, no seed change, no emitted code, no implementation and no acceptance. Dispatched to **measure**, not to build. Per the operator freeze this is the lane's one planning PR; the two funded measurement legs (consolidated claim_batch + census) run on the final head only after this merges and the fixtures are confirmed settled.

## What this does

Implements neat-badger-614 **final split ruling of 2026-11-24** (relayed by warm-fox-517 on PR #16957), which is newer than the row text this node was dispatched on and therefore governs.

**1. Restates `v2-core-emitter-lowers-conjunction-in-value-position`** (amend pattern at `roadmap_authority.dag` ~2185: rewritten `fields(...)` wrapped in `updated(t: fields(...), on:, note:)`, prior notes preserved as history):

- The corpus done-condition moves to a **prefix-free Conj subject emitting** — specifically the conjunction of two well-typed Bool operands (the `src/v2/test/claim/self_host/conj_value_subject_fixture.dag` shape), not merely prefix-free in general. This is achievable and **not a weakening**: `conj_binary_emits` and `disj_binary_emits` already PASS at head `59fedef65b8`, executed evidence that the Conj and Disj arms lower.
- The inner-named-reason condition is **re-targeted** to the emission-time nested refusal, met-as-executed (see evidence below).
- The **Int-wrapper (`nwrap`) claim is retired** from this node; the wrapper behaviour moves to the prefix row as a **named second gap** awaiting that row's ruling — deliberately **not** written here as correct behaviour.
- The compatibility-corpus emitter row's dependency on the prefix-bearing corpus artifact now routes through the prefix row.

**2. Adds `v2-core-emitter-emits-prefix-not-in-value-position`** (new row, `dispatchable()`, identity `rn_7M2KQ4VDE9P1TXY8SRB3NLCW0A`, carrier `src/v2/std/compilers/target_model.dag`), carrying the claims gated on prefix emission:

- Migrated **prefix group**: the prefix-not emit claim + the two precedence controls.
- Migrated **corpus group**: the corpus emit claim + its two refusal controls (the census-forced subject `gunbc.rust_decl_type_overlay` carries a `!closed_alias` prefix operand).
- **Fail-closed precedence requirement**, in the row's own criteria language: a precedence claim must **refuse when `emitted_text_for` yields nothing**, never compare two empty strings as equal nor search for a substring in empty text. The two current precedence failures are exactly that vacuous pattern.
- **Named second gap**: a well-typed wrapped negate (`y + (-x)`, Int) refuses before emission at the **inference** stage because `op_negate` has no catalog row, instead of the inner named reason surfacing at emission. Recorded as a gap awaiting this row's ruling.
- Its **first measurement names its own refusal lexeme** via the narrow dump of the refusal families at `src/v2/std/compilers/target_model.dag` lines 1083, 4968 with 3014, 10254 and 11754, on `prefix_not_operand_subject_fixture.dag`.

## Precedent quoted (per warm-fox)

The prefix row's lexeme-naming red_control follows the measurement-first shape of `whole-corpus-diagnostic-census-callable-from-dag` (line ~3400), whose `red_control` is literally: `OPEN: red_at and green_at are not measured. The first pair would be ...` — the row is dispatched to measure its own red. Same shape here: `OPEN: ... the first red_at and green_at pair would be a claim that the prefix subject ... emits, red while the prefix operand refuses at emission on an unenumerated lexeme.`

## Executed evidence cited in the row text

**probe2** — entry `src/v2/test/claim/self_host/stage0_prefix_inner_reason_probe2.dag`, tree `9dfda4f26c261b1b37cd95359c5e687cf8260252`, hit count **56** witness verdicts, HOST `srv1` BOOT `c095c0cd`, claim_batch sha256 `9cb21df4c20325fc53ba4187cf4c55bd08037862a45d3900c5cc5519e360a177`, real exit status **1**. Seven pass verbatim:

```
PASS negate_lex_is_arity_invalid_holds
PASS nwrap_stage_is_inference_holds
PASS corpus_reason_equals_prefix_reason_holds
PASS conjpre_stage_is_emission_holds
PASS conjpre_reason_equals_prefix_reason_holds
PASS disjpre_stage_is_emission_holds
PASS disjpre_reason_equals_prefix_reason_holds
```

`corpus_reason_equals_prefix_reason_holds` PASS is the executed justification that the corpus failure is pinned on its single prefix operand (it refuses with the same reason as the standalone prefix), which is why a prefix-free Conj subject is achievable for the conjunction row. The `conjpre`/`disjpre` PASSes are the executed backing for the re-targeted inner-reason condition.

**probe1** (stage pins) — entry `src/v2/test/claim/self_host/stage0_prefix_inner_reason_probe.dag`, tree `e948da35ce0`, hit count **31**: the standalone negate refuses at emission with `target_value_expression_emit_primitive_apply_arity_invalid`.

No green_at sha is claimed anywhere; acceptance is recorded separately via `roadmap_event_cli` and cannot be pre-accepted from the row. This criteria rewrite establishes no implementation or acceptance; earlier receipts must be assessed against the live criteria digest.

## `target_serialize` claim — resolved, not plan content

A `target_serialize`-pinned claim is **not** added here: a grep over the whole `src/v2/test/claim/` directory returns **zero** references to a `target_serialize`-pinned claim, so there is no pin to update and any edit would have been invented — a plan-only PR must not mix a pinned-claim change into a roadmap-authority change (warm-fox withdrew this item). The `target_serialize` emitted diff (the callback passed down to `target_project_binary_primitive_apply_in_arrow_scope` / `target_project_transform_primitive_apply_in_arrow_scope` / `target_arrow_scope_positional_operand_value`, plus the emitter-inserted `stacker::maybe_grow` stack-depth guard that the now self-referential `translate_project_transform_in_arrow_scope` requires) is disclosed verbatim in the **implementation** PR #16957 body, which satisfies neat-badger's condition on that file. If a future row wants a claim pinning the emitted text of `target_serialize`, that is a criteria change that comes through neat-badger before it is written, not an inline addition here.

## Gate

The parse/typecheck arbiter is **v2-bootstrap** (it compiles the whole `.dag` corpus including `dag/gunbc/roadmap`, so a passing `RoadmapNodeIdentity` from `gunbc.roadmap.roadmap_model` and the row shape are accepted by the model) — there is no separate `roadmap-authority-sync` check and no `plan-only` label in the repo. Checks at the approved head `4813a739df8528eafaef98ea67e89d6f274dd9c3`, all pass:

- `v2-bootstrap` — pass, **6m53s** (compiles `roadmap_authority.dag`; accepts the new identity + row shape)
- `heal-generated-artifacts` — pass, **18m58s**
- `Hosted-runner shadow gate: configuration` — pass, **5s**

Review `97899` (claude/opus) — **APPROVE** on this head; readiness `READY: True` (1/1 dashboard approval, no REQUEST_CHANGES on the head, GitHub mergeStateStatus CLEAN).

Identity `rn_7M2KQ4VDE9P1TXY8SRB3NLCW0A` is 26 uppercase alphanumerics, neighbour-shaped; if the gate ever rejects an identity or field I will paste its raw output rather than guess a second shape.
