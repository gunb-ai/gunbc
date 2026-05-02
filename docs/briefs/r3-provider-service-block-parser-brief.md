# Provider Service Block Parser Brief

**Status:** `PROPOSAL`  
**Lane:** `T-Anthropic-Wire / ProviderTypedWire C2`  
**Scope:** docs/design receipt only. No parser, AST, lowering, provider rows, or mirror retirement in this slice.

## Decision

Do not start the parser/AST/lowering code slice from the current tree.

The current v3 parser still dispatches only the existing top-level surface item
forms (`Let`, `Fn`, `Type`, `Module`, `Import`, and `Data`). There is no
`service` keyword anchor yet, so a code PR that claims to ingest canonical
provider blocks would have to open a broader grammar cascade than the narrow
slice we want here.

This brief records the exact shared service-block shape that the first code PR
should target once the grammar anchor exists, and it defines the STOP boundary
for that later implementation PR.

## Current Authority Surface

The verified canonical service authority is the existing provider extdeps
service blocks in:

- `dsl/extdeps/llm/anthropic.dag`
- `dsl/extdeps/llm/openai.dag`

Those files already carry the full shared service grammar we want the first
implementation slice to parse and lower: service config, operations, typed
inputs, typed outputs, REST transport, response mappings, and mock responses.

Design context and authority pointers for that future implementation live in:

- `docs/r3-structure.md`
- `docs/briefs/r3-provider-typed-wire-c2-scope-audit.md`

Existing shared substrate targets that should receive those parsed facts live in:

- `src/v3/std/services.dag`
- `src/v3/std/anthropic_messages.dag`
- `src/v3/std/anthropic_operations.dag`
- `src/v3/std/anthropic_schema.dag`

Current shared carriers already in the tree:

- `InputField`
- `RestEndpointBinding`
- `CallableRef`
- `Operation`

These are the current shared lowering targets for service-operation facts. They
are the right place to anchor the first parser/lowerer scaffold if the grammar
anchor can be added without broadening the compiler surface.

## Canonical Shared Grammar

The first code slice should target the canonical provider-service block shape
already present in `dsl/extdeps/llm/{anthropic,openai}.dag`:

```dag
service <Name> {
  config { ... }
  operation <Name> {
    input { ... }
    output { ... }
    transport rest { ... }
    response { ... }
    mock_response { ... }
  }
}
```

The exact surface may grow or shrink at the parser boundary, but the first
implementation PR must stay inside the shared service/operation grammar and not
introduce provider-specific row population.

## AST Surfaces To Name In The Code PR

The parser brief for the first implementation PR should name these AST-facing
surfaces explicitly:

- `ServiceBlock`
- `ServiceConfig`
- `ServiceOperationBlock`
- `ServiceTransportRest`
- `ServiceResponseBlock`
- `ServiceMockResponseBlock`

Those nodes are only parser/AST scaffolding. They are not provider rows and
they are not a mirror of the provider files themselves.

## Lowering Surfaces To Target

The first code slice after the grammar anchor should lower into the shared
service carriers that already exist, or into a minimal shared root carrier only
if the parser cannot preserve service-level configuration facts otherwise.

The acceptable lowering targets are:

- `CallableRef`
- `InputField`
- `RestEndpointBinding`
- `Operation`

If the shared grammar cannot lower cleanly into those surfaces without
re-encoding operation facts, the slice is too broad and must STOP.

## STOP Boundaries

STOP and split the work if any of the following happens:

1. Nested map/object literals are required beyond the service-block scaffold.
2. Callable-target validation requires a broader declaration-refinement change.
3. Provider row population is needed to make the parser/lowering honest.
4. Anthropic mirror retirement becomes coupled to the parser/AST/lowering slice.
5. The design needs a new provider-specific fact carrier instead of shared
   service ingestion first.

If any of those conditions appears, the right outcome is a follow-up plan PR,
not a mixed implementation PR.

## First Code Slice After This Brief

Once the grammar anchor exists, the first implementation PR should be the
minimal shared service/operation parser + AST + lowering scaffold only.

The intended order is:

1. Add shared `service` / `operation` parsing.
2. Lower into the existing shared service carriers, or a minimal shared root if
   absolutely necessary.
3. Add focused shape ratchets for the shared service/operation surface.
4. Keep provider row population and Anthropic mirror retirement in a later PR.

This keeps the slice honest: parse the canonical service blocks first, prove the
shared lowering surface can carry the facts, and defer provider-specific work
until the shared ingestion path exists.
