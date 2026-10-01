# Body lowering: each producer declares the shape it emits (MQ-5 model)

Model for MQ-5 condition (2). Ruling: neat-boar-16, 2026-09-30, option A. Reviewer: neat-boar-16.
Failure-mode row: `gunbc.recurring_failure_mode` `lowered_output_reread_as_raw_parse_at_v2_body_lowering`.

**Precondition: the MQ-5 flip, gunbc#12862, lands before stage 1 starts.** This plan's staging is fail-closed only because of that flip. It is the PR that defines the symbols cited below, which do not resolve until it lands:

- `body_lowering_reason_unrecognised_lowered_shape`
- `body_lower_is_unrecognised_lowered_shape` and `body_lower_unrecognised_lowered_shape_diagnostics`
- `body_lower_find_core_substrate_passes_unrecognised`
- the failure-mode row `lowered_output_reread_as_raw_parse_at_v2_body_lowering`

Without the flip, a reducer that has not yet moved still re-reads an unrecognised shape as raw parse, silently. So no stage of this plan may land ahead of #12862.

## The gap

`v2.compiler.body_lowering_fold` is a bottom-up fold over a tree that holds two kinds of node at once:

- raw parse, as the parser emitted it;
- lowered output, as some producer built it.

Nothing on a `Node` says which kind a node is, or which producer built it. So every reader asks a shape question, `body_lower_is_core_substrate`, and the answer to "is this lowered?" is a hand-maintained allowlist. The allowlist currently accepts:

- an `Arrow`;
- any `ComputationNode`;
- the int literal atom and the string literal atom;
- the symbol literal;
- the qualified-name spine;
- the construct, with an authored or an elided tag;
- the field projection.

A producer whose shape is missing from that list is re-read as raw parse, and the parse readers' first-match descents answer a part of it.

The MQ-5 flip (`body_lowering_reason_unrecognised_lowered_shape`) makes that case refuse, located at the node, instead of truncating. It does **not** reach construction:

- recognition is still a shape test kept in sync by hand;
- the refusal cannot name the producer.

`body_producer_forward` cannot supply the answer either. Its rows key the *surface* identity a producer consumes, not the shape it emits.

Why a returned wrapper alone fails: a producer that returned `Lowered { node, shape }` would lose the tag at the next `node_rebuild`. The fold writes each child result back into its parent as a plain `Edge` target. The tag therefore has to live in the tree the fold threads, not only in a producer's return value.

## The carrier

```
type LoweredShape
  = | LoweredArrow
    | LoweredComputation { behavior: Behavior }
    | LoweredIntLiteral
    | LoweredStringLiteral
    | LoweredSymbolLiteral
    | LoweredQualifiedSpine
    | LoweredConstruct { tag: ConstructTagReading }
    | LoweredFieldProjection

type BodyTerm
  = | RawParse { node: Node }
    | Lowered { node: Node, shape: LoweredShape }

type BodyTermEdge { label: EdgeLabel, target: BodyTerm }
```

- **Home.** The types live in `v2.std.compilers.body_lowering`, the existing stage-vocabulary module the fold already imports. They are stage-local and are not a substrate type.
- **Ingress.** The parse tree enters the stage once as `RawParse`. Children are folded to `BodyTerm`.
- **Producers.** Each producer returns `Lowered` with its own arm, written as a constant in the producer. That makes "is this shape core substrate" a fact of construction, not of classification.
- **Readers.** A reader matches `RawParse` against `Lowered`. A `Lowered` term is never searched, descended or re-read, so the re-read arm cannot be written at all.
- **Exit.** The stage exits to `Node` exactly once, at the normalize boundary (`body_lower_finish` in `03_normalize`). It erases the `BodyTerm` wrapper there. That exit is the only place a `Lowered` becomes a bare `Node`.
- **Upstream completeness.** Lowering from a `RawParse` that no producer claims is the existing `unsupported_form` refusal, now typed against `RawParse`.

## What it deletes, and when

The PR that moves the **last** reducer onto `BodyTerm` deletes all four of these together:

- `body_lower_is_core_substrate`;
- `body_lower_is_unrecognised_lowered_shape` and `body_lower_unrecognised_lowered_shape_diagnostics`;
- `body_lower_find_core_substrate_optional`, the first-match descendant search, which has nothing left to search for;
- `body_lower_find_core_substrate_passes_unrecognised`.

No reducer is ever answered by both the predicate and the tag (neat-boar-16 ruling). The reason `body_lowering_reason_unrecognised_lowered_shape` then has no producer. Its `compile_door_cause_ownership` row flips, and the failure-mode row climbs to *structurally impossible*.

## Staging (the fold is about 9.5k lines)

Binding rules for every stage (neat-boar-16 review of this plan):

- **This document is the plan, not the model.** The `.dag` types `LoweredShape`, `BodyTerm` and `BodyTermEdge` land in stage 1, together with their first consumer (the operand reader). They never land unconsumed.
- **One reducer family per stage, moved whole.** No stage moves a family partly.
- **No silent change to lowering output.** Every stage carries conservation controls showing no atom is dropped. It also carries a native census (`claim_executor --v2-native-route`, base vs head) showing zero change to lowering outcomes.

Each stage is one PR. Each stage moves a reducer family **whole**: every producer it calls returns `BodyTerm`, and every reader it calls matches on `BodyTerm`. A family is never partly moved.

1. **Carrier and ingress/egress.**
   - `LoweredShape`, `BodyTerm` and `BodyTermEdge` land, together with the `RawParse` ingress at `body_lower_fold` entry and the single egress at `body_lower_finish`.
   - Inside this first PR, the producers of the literal atoms and the symbol literal return `Lowered`, and the operand reader (`body_lower_operand_ref_read` and its callers) consumes them. So the carrier lands consumed (DESIGN §3c), never dangling.
2. **Spine and postfix.** Qualified-name spine, field projection, the postfix chain fold and its readers.
3. **Construct.** The construct and the record literal.
4. **Operator tower.** Operators and `Transform` producers.
5. **Control forms.** `Branch`, `Bind`, `Loop` and `Match` producers: match, if, let and loop.
6. **Declarations.** The fn, data and test declaration producers, plus `Arrow`.
7. **Cut.** This is the deletion PR above: the last reducer moves and the predicate is deleted.

While stages 1–6 are in flight, the MQ-5 flip refusal keeps every reducer that has not yet moved fail-closed. A reducer that has moved never consults the predicate.

## Consumers (DESIGN §3c)

| Declaration | Consumer | Route |
|---|---|---|
| `LoweredShape` arms | the reducers that match on them | `body_lower_fold` → reducer → `match term { Lowered { shape } … }` |
| `BodyTerm` | every reducer in `body_lowering_fold` | the threaded fold value |
| `BodyTerm` egress | `03_normalize` `body_lower_finish` | the single exit to `Node` |

## Controls

Carried over from the flip PR, which are re-pointed and not deleted (DESIGN §4b(4)):

- an unknown shape refuses, located;
- a field projection lowers whole;
- a raw sequence is not output.

New in this programme:

1. **Every producer arm.** For each `LoweredShape` arm, a supplied input whose producer emits it reaches the egress as that node unchanged. Reference conservation shows no atom dropped.
2. **Route inhabitance.** The real-text claims `value_position_whole_read` and `reference_conservation_admission` stay green through each stage. They are the §3 pairing that the supplied controls rely on.
3. **The re-read is unwritable.** At the cut, a reader handed a `Lowered` term has no arm that descends into it. The mutation that restores the shape predicate in one reducer fails to compile, because there is no `Node` to ask, instead of failing a test.
4. **Census.** At the cut, re-run the MQ-5 census (`claim_executor --v2-native-route`, the base-vs-head diff of the PR #12550 recipe). It also covers the 27 files currently refused during lowering, once they clear.

## Not in scope

- The 27 files refused during lowering (`let_body_not_reachable_from_node`, `wrapper_retention`) belong to their own rows.
- The `EdgeLabel` work (#12799, quiet-koi-814) and the occurrence images (#12790, bold-fox-455) touch the same fold, so each stage rebases over whichever of them lands first. `BodyTerm` wraps `Node` and changes neither the labels nor the occurrences.
