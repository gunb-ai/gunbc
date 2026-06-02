# RR-B Omni-Ingestion Worksheet - Branch B-min

> Status: Branch B-min implementation worksheet. This is a `.dag` authority and planning slice only: no Rust floor, no parser-stage migration, and no compiler-stage edits.

## Scope

Branch B-min establishes the smallest `.dag` ingestion floor that can fail closed today without fabricating ingest support. It binds the existing `.dag` wave-1 language model authorities into a dedicated `TestClaim` row:

- Lex authority: `src/v4/extdeps/languages/dag.dag` `dag_round_trip_lex_ready`.
- Grammar authority: `src/v4/extdeps/languages/dag.dag` `dag_round_trip_grammar_ready`.
- C5 trivia fidelity authority: `src/v4/extdeps/languages/dag.dag` `dag_round_trip_normalization_declared`.
- Floor claim: `src/v4/test/claim/round_trip/dag_ingestion_floor_min.dag`.

The floor deliberately reuses `dag_round_trip_wave1_authorities_ready()` instead of re-declaring axis predicates. That keeps `dag.dag` as the single authority for the wave-1 ingest-readiness cluster and avoids a parallel worksheet-local checklist.

## RR-B Target

RR-B is the next omni-ingestion release-readiness worksheet. Its job is to turn the current per-language ingest probe pattern into an auditable roster without implying that full source ingest is landed.

Required rows:

| Row | Authority | Floor intent |
|---|---|---|
| `.dag` wave-1 | `src/v4/extdeps/languages/dag.dag` | Lossless-core ingest readiness plus declared trivia normalization. |
| English boundary | `src/v4/test/claim/boundary/english_ingest_fail_closed.dag` | Natural-language prose stays outside the lossless core and returns a typed diagnostic. |
| LLVM IR probe | `src/v4/test/claim/manual/llvm_ir_b2_omni_in_b_probe.dag` | IN-B structured SSA/CFG/phi probe remains modeled, not a text adapter. |
| Grammar probe family | `GrammarSchemaProbeBinding` rows in `src/v4/extdeps/languages/*` and `src/v4/extdeps/formats/*` | Each ingest candidate names lex, grammar, fixture, production, and expected parse surface. |

## Acceptance Bar

- New readiness evidence must be `TestClaim` data or typed substrate data.
- A claim label must not imply bit-identical emit-to-ingest fidelity until the W1b compare lands.
- Arbitrary English or other non-lossless inputs must reject with typed diagnostics, not plausible parses.
- Any temporary Bool projection must name a dissolve-on-arrival path to a typed receipt carrier.
- No new hand-authored Rust test floor is allowed for this slice.

## Follow-Up

RR-B should replace the scattered probe inventory with a typed roster carrier once the claim runner can consume `GrammarSchemaProbeBinding` directly. The dissolve target is a single roster-shaped `.dag` authority, not a Rust census entry or duplicated markdown checklist.
