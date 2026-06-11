# v4 Branch H Source-Authority Worksheet

> **Status:** H.7.1 / RR-H worksheet draft for `node://adhoc-3f45e712-719`.
> **Authority:** ctrl#1425 §2.7 + §3 Branch H (Wave 2 dispatch 2026-06-02).
> **Scope:** H.1-H.6 design contracts; H.7.2/H.7.3 are gated until this worksheet is ratified.

## Guard

`dag-artifact.json` and the current `gunbc compile --target dag` output are boundary/debug IR
receipts only. They are not canonical `.dag` source and must not become source-authoring authority.
Branch H keeps three surfaces separate:

- semantic compiler graph / IR
- canonical `.dag` source
- boundary/debug receipts

Any H.7.2 implementation must prove source round-trip over canonical `.dag` text, not JSON IR.

## H.1 Canonical `.dag` AST

Canonical source authority is a first-class `.dag` syntax/source tree: module header, imports,
declarations, type expressions, values, function bodies, comments/trivia policy, and stable ordering.
The AST must model source facts explicitly enough for mechanical parse/print/edit consumers. It is
not the post-infer semantic graph.

## H.2 Serializer

The serializer prints canonical `.dag` source from the H.1 AST with deterministic whitespace,
ordering, escaping, and naming. It must emit text that the parser accepts as source. It must not
serialize through `dag-artifact.json`.

## H.3 Parse/Print Law

The first law is normalized source equality:

```text
source.dag -> H.1 AST -> canonical_source.dag -> H.1 AST
```

The acceptance receipt compares normalized H.1 ASTs. Semantic IR equality may be an additional
check, but cannot replace the source-law receipt.

## H.4 Programmatic Edit Algebra

Edits operate on the canonical source AST: insert/remove/update declarations, imports, fields, and
function bodies with explicit loci. Edits must preserve parseability or fail closed with a typed
diagnostic. Rust-to-DAG decompilation and per-language bidirectional edit extensions are out of
Wave 2 scope.

## H.5 CLI Surface

Wave 2 exposes only bounded source-authority commands:

- `gunbc compile --target dag` must become canonical source output when H.7.3 lands.
- Until then, JSON IR behavior is explicitly boundary/debug behavior and must be named as such by
  docs/receipts.
- `gunbc testgen` and `gunbc test` are CLI contract rows for Runtime/TestClaim coordination, not
  source authority themselves.

## H.6 Bootstrap + CI Consumers

Bootstrap, CI receipts, Branch B-min round-trip, Branch E DAG-regen, and Branch F CI digest
consumers may consume H receipts only after the canonical source law exists. They must not consume
`dag-artifact.json` as evidence of source round-trip.

## H.7 Wave 2 Floor

- **H.7.1:** this worksheet plus RR-H guard/acceptance matrix.
- **H.7.2:** after H.7.1 ratification, one small compiler module proves
  `source.dag -> H.1 AST -> canonical_source.dag -> H.1 AST` with normalized source-AST equality.
  Semantic IR equality may be recorded as a secondary check only; it cannot replace or route around
  the H.1/H.2 source-law receipt.
- **H.7.3:** expose canonical source through `gunbc compile --target dag`; if JSON IR remains needed
  for debug, it must move to a boundary/debug-named surface.

## RR-H Acceptance Matrix

| Row | Required evidence | Forbidden evidence |
|---|---|---|
| H.1 | `.dag` source AST carrier | post-infer JSON node map |
| H.2 | deterministic canonical source serializer | JSON pretty-printer |
| H.3 | normalized AST parse/print equality | `dag-artifact.json` byte equality |
| H.4 | typed edit operations over source AST | Rust-to-DAG decompilation |
| H.5 | CLI routes named by source vs debug semantics | hidden JSON IR under source name |
| H.6 | bootstrap/CI consume H receipts | downstream source claims from JSON IR |

## Handoffs

When this worksheet opens, notify B-min (`zesty-owl-311`), Cross-target Emission
(`silent-bear-54`), Branch E (`royal-badger-408`), and Runtime/TestClaim (`royal-gull-451`). When
H.7.2 lands, notify `silent-crane-669` for F.11b CI receipt consumption.
