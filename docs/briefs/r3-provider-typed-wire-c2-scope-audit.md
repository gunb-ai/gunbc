---
status: AUDIT
owning_manager: Substrate Manager / R3 Grounding coordination
lane: T-Anthropic-Wire / ProviderTypedWire C2
authored: 2026-05-02
---

# ProviderTypedWire C2 Scope Audit

## Decision

Do not author a provider-specific mirror or a new fact-bearing
`ProviderTypedWire<P>` carrier in this slice.

The shared path is:

1. Substrate owns the generic service/operation/transport carriers and the
   parse/lower surface that turns canonical `service { operation { transport
   rest { ... } } }` blocks into those carriers.
2. Grounding owns provider row population from canonical provider files
   (`dsl/extdeps/llm/anthropic.dag`, `dsl/extdeps/llm/openai.dag`, and later
   provider directories) once the shared surface can ingest them.
3. `ProviderTypedWire<P>` remains at most a thin handle over parsed `Service<P>`
   if a concrete cross-provider consumer needs a parametric value. It must not
   re-encode operation facts.

This follows the reframe in
[`r3-substrate-provider-typed-wire-worker.md`](r3-substrate-provider-typed-wire-worker.md):
the canonical service block is the authority. A replacement carrier that only
captures endpoint/request/response fragments would drop facts and multiply
per-provider mirrors.

## Current Substrate

`src/v3/std/services.dag` already owns a narrow PR-alpha carrier set:

| Carrier | Current role | C2 disposition |
|---|---|---|
| `InputField {}` | Empty per-input metadata slot, keyed by `Operation.inputs: Map<String, InputField>`. | Keep shared. Extend only when shared service ingestion needs field metadata such as defaults or body participation. Do not add a duplicated `name` field. |
| `RestEndpointBinding { method, path }` | Minimal REST endpoint facts consumed by existing operation-effect derivation. | Keep shared. Do not add provider-specific endpoint mirrors. Future shared fields can include request body binding, headers, response status mapping, and mock responses if the parsed service block lowers them here or into sibling shared carriers. |
| `CallableRef { decl }` | Typed wrapper around `DeclarationRef`; declares that an operation row points at a callable. | Keep as transitional until `DeclarationRef` can be refined to callable declarations. |
| `Operation { callable, inputs, endpoint }` | Shared row keyed by callable identity plus input map and endpoint. | Keep as the current row-population target, but recognize it is not a full service-block model. It is insufficient to dissolve provider mirrors by itself. |

The integration ratchet
`src/v3/compiler/tests/integration/services_carrier_shape_test.rs` pins this
scope: `Operation` deliberately has only `callable`, `inputs`, and `endpoint`;
`RestEndpointBinding` deliberately has only `method` and `path`; `PathTemplate`
is imported from `std.effects`, not redefined.

## Current Anthropic Chain

The Anthropic v3 chain is a staged bridge, not the desired final authority:

| File | What it owns today | Why it is transitional |
|---|---|---|
| `src/v3/std/anthropic_schema.dag` | A v3 mirror of the provider-domain types reachable from Anthropic Messages. | The header names the real authority as `dsl/extdeps/llm/anthropic.dag` and says the mirror retires when v3 can parse/load service DSL with full fidelity. |
| `src/v3/std/anthropic_messages.dag` | `fn anthropic_messages(...) -> AnthropicMessages200Body`, with an honest `ArrowBody::Unparsed` host body. | It exists so `Operation.callable.decl` can point at a real callable before service-operation lowering exists. |
| `src/v3/std/anthropic_operations.dag` | `data anthropic_operations: List<Operation> = []`. | The intended Messages row is deferred: nested `Map<String, InputField>` literals in record fields are not accepted yet, and `InputField` has no default metadata. |

The canonical source still carries more facts than `Operation` can express:
service config (`endpoint`, auth, rate limit, retry), operation inputs and
defaults, typed output projections, request body composition, headers, response
status mapping, and mock responses. Anthropic has one operation today;
OpenAI already has two (`ChatCompletion`, `Responses`). A C2 carrier must
therefore model a service containing many operations, not an Anthropic-only
Messages row.

## Required Parser/Lowering Work

The next shared implementation slice is service-block ingestion, not
`ProviderTypedWire<P>`.

Substrate-owned parse/lower work:

1. Add v3 surface AST for service items:
   `Service { name, config, operations }`, `Operation { name, input, output,
   transport, response, mock_response }`, and REST transport fields.
2. Parse `service`, `config`, `operation`, `input`, `output`, `transport rest`,
   `response`, and `mock_response` blocks with the shapes used by
   `dsl/extdeps/llm/anthropic.dag` and `dsl/extdeps/llm/openai.dag`.
3. Lower service blocks into shared substrate declarations, not provider-specific
   mirrors. The expected shared carrier root is `Service<P>` or equivalent,
   with child carriers for service config, operations, input fields, output
   projections, REST transport, response mapping, and mocks.
4. Support nested map/object values in record-field positions, including
   empty `Map<String, X>` values, so `Operation.inputs` and service headers/body
   maps can be represented structurally.
5. Add shared input-field metadata for defaults. `max_tokens: Int = 4096` in
   Anthropic is the current concrete witness that defaults are not an
   Anthropic-only concern.
6. Keep `PathTemplate` / `UrlPathToken` authority in `std.effects`; service
   lowering should consume that path model rather than declare another one.
7. Fail closed on unresolved callable targets, duplicate operation names,
   duplicate input/output field names, path parameters that do not resolve to
   inputs, and any response/body/header projection that references an unknown
   operation field.

Grounding-owned row population after that slice:

1. Populate provider service rows by ingesting canonical provider files rather
   than writing provider twins under `src/v3/std/`.
2. Fill Anthropic Messages and OpenAI ChatCompletion/Responses rows from the
   same shared `Service<P>` path.
3. Ratchet provider-specific schema facts against the canonical files only as
   a transition; retire lockstep mirrors once parsing is the authority.
4. Keep provider-domain coproducts where they are genuinely domain-specific
   (`AnthropicUserContentBlock`, `AnthropicAssistantContentBlock`,
   `ResponseFormat`, `ToolChoice`, `OpenAiFinishReason`, etc.), but do not copy
   shared transport/service machinery per provider.
   Shared LLM concepts such as message `Role` stay in `dsl/extdeps/llm/llm.dag`
   and must not be reintroduced as provider-local authority.

## ProviderTypedWire Alias

No alias should land yet.

`ProviderTypedWire<P>` is only justified if a concrete consumer needs a
parametric handle over a parsed provider service. If it lands, it should be
thin:

```dag
type ProviderTypedWire<P> {
  provider_identity: P
  service: Service<P>
}
```

It must not contain endpoint, request-body, response, header, mock, projection,
rate-limit, retry, or input-field facts. Those facts belong to the parsed
`Service<P>` graph.

## Next Implementable Slice

Recommended next PR:

**`feat(v3): add service-block parse/lower scaffolding for shared provider ingestion`**

Smallest useful slice:

1. Add shared surface AST nodes for `service`, `operation`, and `transport rest`.
2. Parse a minimal service block containing:
   - one service name,
   - one operation name,
   - an `input` block with typed fields and optional defaults,
   - `transport rest { method, path, body?, headers? }`,
   - `response` status mappings.
3. Lower to new shared service carriers or, if the full carrier set is too large
   for one PR, lower to `ValueBody::Unparsed` with a typed `Service` declaration
   scaffold and explicit diagnostics for unsupported sub-blocks. Dissolution
   trigger for that scaffold: delete the `ValueBody::Unparsed` service fallback
   once shared service child carriers lower config, operations, input metadata
   including defaults, output projections, REST transport method/path/body/
   headers, response status mapping, and mocks without unsupported-sub-block
   diagnostics.
4. Add a fixture using the existing Anthropic Messages service block shape and
   assert it parses as service structure rather than forcing
   `anthropic_schema.dag` / `anthropic_messages.dag` mirrors.

Do not include provider-directory relocation or mirror deletion in that first
slice. Those are follow-ups after the parser/lower path can read canonical
provider files without losing facts.

## STOP/PING Conditions

Stop and route to Substrate + Grounding if any of these occurs:

- The service grammar needed by Anthropic and OpenAI cannot be represented by a
  single shared `Service<P>` carrier without dropping facts.
- The proposed `ProviderTypedWire<P>` grows fields that duplicate parsed service
  facts.
- Nested map/object literals require a general value-body redesign rather than a
  narrow parser/lowering extension.
- Grounding wants to populate more provider mirrors before the shared
  service-block ingestion path exists.

## Concrete Split

Substrate owns:

- `Service<P>` and child carrier shapes.
- Parser and lowering for canonical service blocks.
- Nested map/object literal support needed by service rows.
- Input-field default metadata.
- Callable-target validation and service-block structural diagnostics.

Grounding owns:

- Provider-specific row population from canonical provider files.
- Provider-domain type facts that are genuinely divergent.
- Cross-provider completeness tests once the shared service ingestion path
  exists.
- Retirement PRs deleting v3 mirrors after canonical ingestion is live.

This audit therefore closes C2 as a design-decision slice: the next code should
build shared service ingestion first; `ProviderTypedWire<P>` is deferred until a
consumer proves it needs a thin parametric handle.
