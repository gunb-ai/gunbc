# RR-B Omni-Ingestion Worksheet - Branch B-min

> Status: W2 Branch B-min worksheet and substrate ruling. Scope stays below B-full: `.dag` only, no per-language ingest expansion, no hand-coded parser walk, no Rust floor.

## 10-Field Worksheet

| Field | Entry |
|---|---|
| 1. Work item | `node://adhoc-299588a3-66b` — B-min `.dag` ingestion floor + RR-B worksheet. |
| 2. Branch | Branch B-min now; B-full is explicitly deferred. |
| 3. Authority | `ConcreteSyntaxSchema` in `src/v4/std/grammar.dag` is the single concrete-syntax authority carrier, including a terminal coverage witness tying grammar terminals to lexer token rules. |
| 4. Projections | `LexPattern`, `GrammarExpr`, `ParseGrammar`, and `GrammarSchemaProbeBinding` are operational projections/bridges until CP-1b convergence. |
| 5. Floor claim | `src/v4/test/claim/round_trip/dag_ingestion_floor_min.dag` re-derives `.dag` wave-1 lex, grammar, and C5 trivia readiness from `dag.dag`. |
| 6. B-min rows | Bmin.1 single declared grammar parse path; Bmin.2 structural morphism from authority to parser projection; Bmin.3 `WellFormedFormalGrammar` validating constructor; Bmin.4 round-trip claim; Bmin.5 canonical source emission. |
| 7. Dependency boundary | Bmin.1-3 may proceed before H.7.2. Bmin.4-5 wait for H.7.1 source authority: canonical `.dag` source AST, deterministic serializer, and normalized source AST parse/print law. |
| 8. B.2 findings | Fold operator #7 omni-ingestion findings into RR-B before B-full dispatch; missing Go parse claim is coordination debt, not B-min implementation scope. |
| 9. Acceptance | New evidence is `.dag` `TestClaim` data or typed substrate data; no fabricated parse, no third syntax authority, no permanent dual carrier. |
| 10. Handoffs | Coordinate parse/print law with Source-authority Mgr; coordinate any Go parse-claim movement with the Go owner (`gentle-lynx`) before touching Go parse claims. |

## Branch B-Min Rows

| Row | Status | Required shape |
|---|---|---|
| Bmin.1 | Open | `.dag` parses through one declared `ConcreteSyntaxSchema`; no bespoke `.dag` parser walk. |
| Bmin.2 | Staged | `ConcreteSyntaxSchema.terminal_coverage` fails closed unless each consumed `FormalTerminal.identity` is covered by `LexRules` token rules; downstream parser projection still needs the structural morphism from this authority. |
| Bmin.3 | Open | `WellFormedFormalGrammar` is the validating constructor/witness boundary for formal grammar authority. |
| Bmin.4 | Blocked on H.7.1 | Round-trip claim may consume Bmin.1-3 only after source authority can state the normalized source AST parse/print law. Do not use `dag-artifact.json` or `--target dag` JSON IR equality as source authority. |
| Bmin.5 | Blocked on H.7.1 | Canonical source emission waits for canonical `.dag` source AST plus deterministic serializer; do not claim bit-identical fidelity before W1b compare lands. |

## Branch B.2 Findings

| Finding | Disposition |
|---|---|
| B2.1 `ConcreteSyntaxSchema` must be the authority | Landed as the named carrier in `std/grammar.dag`; terminal coverage is part of the carrier, and projections remain bridges. |
| B2.2 Go parse claim missing | Record as anomaly; do not add Go parse scope in B-min without Go-owner coordination. |
| B2.3 LexPattern CP-1b is the omni-ingestion gate | Ruling: `LexPattern` projects from `ConcreteSyntaxSchema`; it is not an authority. |
| B2.4 GrammarExpr CP-1b shares the same fate | Ruling: `GrammarExpr` is parser projection until derived from formal grammar authority. |
| B2.5 `GrammarSchemaProbeBinding` roster is scattered | RR-B follow-up should replace markdown inventory with typed roster data. |
| B2.6 English prose stays outside the lossless core | Existing fail-closed boundary claim remains the correct pattern. |
| B2.7 LLVM IR IN-B probe is structured, not text adapter | Existing SSA/CFG/phi claim remains evidence for modeled ingest shape. |
| B2.8 B-full per-language ingest is anti-scope | Defer the eight-row B-full expansion until B-min authority is stable. |
| B2.9 Source authority owns parse/print law | Bmin.4-5 hand off to H.3/H.7 before round-trip/canonical-source claims; JSON IR artifacts are boundary/debug output only. |

## Current PR Slice

This PR intentionally lands only:

- the `ConcreteSyntaxSchema` carrier, terminal coverage witness, and projection ruling;
- the B-min `.dag` ingestion floor claim for existing `.dag` wave-1 authority readiness;
- this RR-B worksheet folding B-min plus B.2 findings.

It does not land the full Bmin.1-3 parser-projection morphism implementation, a Go parse claim, B-full ingest rows, or canonical source emission. Those are downstream dispatches once the authority and dependency boundaries are accepted.
