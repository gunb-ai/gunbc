# v4 Branch E E.3 DAG-Regen Design Closure

> **Status:** DESIGN CLOSURE AFTER H.7.2 - no full DAG-regen implementation in this note.
> **Work item:** `node://adhoc-ed410c90-7cb` - Branch E Mgr (`royal-badger-408`).
> **Unblocking receipt:** Branch H.7.2 source-authority contract merged in PR #4298.

## Closure Statement

Branch E.3 is no longer blocked on an unnamed serializer gap. H.7.2 establishes the source-authority
round-trip shape in `src/v4/compiler/source_authority.dag`: canonical `.dag` source is produced
through `target_serialize_source_from_model`, then reparsed, and the accepted law carries both
`SourceAstEqual` and `SemanticIrEqual` payloads. JSON IR/debug artifacts remain excluded.

This closes the W2 RR-E design question and gives W3+ implementation a single authority path:

```text
source.dag
  -> H.1/H.7.2 source AST/semantic IR surfaces
  -> canonical_source.dag via target_serialize_source_from_model
  -> H.1/H.7.2 source AST/semantic IR surfaces
  -> SourceAstEqual primary receipt, SemanticIrEqual secondary receipt
```

## E.3 Minimal Vertical Slice

The first Branch E.3 implementation slice should use one small compiler module and prove:

1. The module parses into the H source-authority surface without using `dag-artifact.json`.
2. The canonical source target is a `.dag` source `TargetModel`, not an emitted Rust target.
3. `source_authority_round_trip` returns `Holds { SourceAuthorityRoundTripLaw { ... } }`.
4. The source-law receipt is normalized H.1 AST equality; semantic IR equality is required but
   cannot replace the source-law receipt.
5. A falsification fixture rejects a JSON IR-only or emitted-Rust-only path.

## Boundaries

- Branch H owns canonical `.dag` source authority and the serializer contract.
- Branch E consumes H.7.2 for bootstrap-as-data and DAG-source regeneration.
- Branch C remains the owner of `06_translate` / Shape-A scaling; Branch E consumes C.5 green and
  must not expand `05_emit` or co-own translate.
- Rust -> DAG decompilation remains R5 anti-scope.
- Hand-maintained Rust dissolution is downstream of source-authority and bootstrap fixed-point
  receipts; census deltas are not the migration path.

## Next Dispatch Shape

Branch E.3 implementation should land as a small source-law receipt PR before any broad regen work:

```text
E3.1/H.7.2 contract receipt
  -> E3.4 one-module source round trip
  -> E3.5 bootstrap-as-data fixed-point row wiring
  -> E3.6 wider compiler-module DAG regen scaling
```

The PR is not allowed to call `gunbc compile --target dag` a source regenerator unless that target
is backed by the H.7.2 canonical `.dag` source TargetModel and the normalized source AST equality
receipt.
