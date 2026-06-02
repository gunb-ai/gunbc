# v4 Completeness Audit: ctrl#1425 vs Deleted v4 Docs

> **Status:** Implementation audit receipt.
> **Work item:** `node://adhoc-6c2a6f6e-26b`.
> **Verifier:** `scripts/v4-completeness-audit-adhoc-6c2a6f6e-26b.sh`.
> **Scope:** compare the merged #1425 receipt-key pattern with the deleted v4 closeout docs, then classify live v4 gaps for omni-ingestion, omni-emission, and R4-promoted thesis claims.

## §1 Source Inputs

`ctrl#1425` landed as GitHub PR #1425, merge commit `1b47df92e2177a78e254680e5953f311d975068a`. Its live artifact is narrow: `src/v3/compiler/src/self_host_receipt_p0.rs` pins stable top-level keys for `target/self_host/receipt.json`, and `src/v3/compiler/src/bin/self_host_fixed_point.rs` consumes those constants.

That pattern is a P2/P3 receipt-key guard. It proves key-name stability for a small v3 self-host receipt. It does not prove any v4 closeout probe, target emission, grammar ingestion, or R4-promoted thesis claim.

The deleted v4 source docs inspected from the pre-purge history were:

- `docs/audit/v4-close-interrogation-validation-2026-05-30.md`
- `docs/audit/v4-close-ledger-2026-05-30.md`
- `docs/audit/r4-ctrl-phase15-subsystem-receipt-trail.md`

Those files should not be restored as live authorities. They were historical audits. The live replacement needed here is a small current receipt that names what v4 has and what remains blocked.

## §2 Headline Verdict

`ctrl#1425` is not an implementation close receipt for v4. It is a useful analogy for stable receipt-key discipline only.

Live v4 has substrate for the asked areas:

- omni-ingestion carriers: `src/v4/std/lexing.dag`, `src/v4/std/grammar.dag`, `src/v4/compiler/01_tokenize.dag`, `src/v4/compiler/02_parse.dag`
- omni-emission carriers: `src/v4/std/target_model.dag`, `src/v4/compiler/05_emit.dag`, `src/v4/compiler/06_translate.dag`, per-language files under `src/v4/extdeps/languages/`
- R4-promoted claim substrate: `src/v4/std/verification.dag`, `src/v4/test/claim/`, `src/v4/extdeps/frameworks/react.dag`, `src/v4/extdeps/coordination.dag`, and additional Shape-A language models

Live v4 is not close-ready for those areas because the executable receipts are still missing or staged behind yellow-gated dissolution points. The old close-ledger conclusion remains directionally correct: substrate-present is not the same as PROVEN.

## §3 Omni-Ingestion Findings

The canonical carriers are present:

- `src/v4/std/lexing.dag` defines `LexPattern`, `LexRule`, `LexRuleSet`, `LexRules`, `Token`, and `TokenStream`.
- `src/v4/std/grammar.dag` defines `FormalGrammar`, `WellFormedFormalGrammar`, `GrammarExpr`, `GrammarRoot`, `ParseGrammar`, and `GrammarSchema`.
- `src/v4/compiler/01_tokenize.dag` consumes the lexing model.
- `src/v4/compiler/02_parse.dag` consumes the grammar model.

The blockers are explicit in the files:

- `LexPattern` and `GrammarExpr` have not converged onto a single bidirectional concrete-syntax carrier.
- `LexRules` still has `VoidLexRules` staging.
- `FormalGrammar` can represent invalid states until `WellFormedFormalGrammar` has a validating constructor and required consumers.
- `compiler/01_tokenize.dag` and `compiler/02_parse.dag` still contain hand-written coproduct walks marked as predicate-dissolution interims.

Close condition: the grammar and lexing carrier must be the single authority for source ingestion, parser/tokenizer consumers must use derived structural morphisms, and the verifier must include an executable parse/ingest falsification receipt.

## §4 Omni-Emission Findings

The canonical carrier home is present:

- `src/v4/std/target_model.dag` owns `TargetModel`, grammar-relation edge keys, `TargetAtomRealization`, value templates, and target type expression carriers.
- `src/v4/compiler/05_emit.dag` and `src/v4/compiler/06_translate.dag` consume target models rather than owning a separate target vocabulary.
- Rust, Python, Go, TypeScript, C++, LLVM IR, PTX, Verilog, WASM, and related language models exist under `src/v4/extdeps/languages/`.

The blockers are also explicit:

- `target_model.dag` still has yellow gates for typed concrete syntax token variants and the canonical string value carrier.
- Per-language host verification scripts remain boundary receipts for some leaf models.
- `RoundTripClaim` and `TestClaim` carriers exist, but close proof requires run verdicts and adversarial falsification transcripts, not carrier declaration alone.

Close condition: target realization lookups and source rendering need executable `TestClaimRun` receipts per target/rung, including no-inhabitance and malformed-target falsification cases.

## §5 R4-Promoted Thesis Gaps

The deleted close docs promoted several former R4 areas into v4 scope: arbitrary ingestion, additional Shape-A languages, framework substrates, and multi-program/network coordination.

Current live state:

- Additional Shape-A language substrate exists under `src/v4/extdeps/languages/`.
- React framework substrate exists at `src/v4/extdeps/frameworks/react.dag`.
- Coordination substrate exists at `src/v4/extdeps/coordination.dag`.
- Test claim and verification carriers exist in `src/v4/std/verification.dag` and `src/v4/test/claim/`.

Gap classification:

- These are substrate-present, not PROVEN.
- The missing proof class is executable: run verdicts, falsification fixtures, and close receipts that answer the exact probes.
- Restoring the deleted v4 audit docs would create stale doc authority. The live path is to keep compact receipts like this one and wire executable checks as the substrate matures.

## §6 Required Follow-Ups

1. Omni-ingestion: converge `LexPattern` and `GrammarExpr` onto one bidirectional grammar carrier, and replace hand-written tokenizer/parser coproduct walks with substrate-derived morphisms.
2. Omni-emission: close the `TargetModel` yellow gates for typed concrete syntax tokens and canonical string carriers, then prove target realization with executable run receipts.
3. R4-promoted claims: keep them in v4 scope, but only mark them PROVEN when `TestClaimRun` verdicts and falsification transcripts exist for the named probes.
4. Receipt discipline: use #1425 as the pattern for stable key naming where receipts emit JSON, but do not count key pins as semantic close evidence.

