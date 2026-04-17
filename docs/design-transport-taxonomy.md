> Part of: [post-l15-phase-plan.md](./post-l15-phase-plan.md) | Unblocks: Lane 4 Stage 4a, Lane 2 Stage 2b (idempotency lens reads transport specs)

# Design DB-6 — Transports as spec declarations (no closed coproduct)

**Design blocker:** DB-6
**Consumers:** Lane 4 Stage 4a (transport declarations + `dag run` interpreter); Lane 2 Stage 2b (idempotency lens); `dsl/extdeps/*/` declarations
**Status:** Design ready for implementer review.

---

## Critical correction from prior revision

An earlier version of this doc proposed a closed `TransportKind` coproduct — `RestTransport | ShellTransport | GrpcTransport | LocalFunctionTransport` — with typed per-variant sub-data AND per-transport dispatchers in the Rust emitter / interpreter.

Reviewer (PR #491, 2026-04-17) correctly flagged this as **parallel authority** per THESIS.md:117-139:

> *"the interpreter does not have per-transport handlers. It reads the same transport specs as the emitter (`extdeps/transports/`). ... Adding a new transport (gRPC, WebSocket, etc.) means adding a spec in `extdeps/transports/` — zero compiler changes, zero emitter changes, zero interpreter changes."*
>
> *"The sustainability test: when the system grows by one transport or one language, how many files need editing? The answer should be 1: the spec file. If it's more, there's a parallel list somewhere that will drift and break."*

A closed `TransportKind` enum in Rust plus per-variant dispatch code IS that parallel list.

**The corrected design: transports are declared in `extdeps/transports/` specs. The compiler's substrate knows only that a Transport IS a spec reference. No closed enum, no per-variant dispatch.**

---

## Design

### Substrate shape (minimal)

User / extdeps code:

```dag
service gcp.STS {
  operation Exchange {
    transport rest {
      method: POST
      path: "/v1/token"
      body_shape: JsonBody(TokenExchangeRequest)
    }
  }
}
```

Parser lowers this into a substrate `TransportDeclaration`:

```dag
// src/v3/std/substrate.dag — single substrate carrier
type TransportDeclaration {
  spec_ref: DeclarationId   // points at `rest` in extdeps/transports/rest.dag
  fields: List<FieldEntry>  // method, path, body_shape — the literal declared fields
}
```

`spec_ref` points to the transport spec file. `fields` is the bag of declarations the user supplied. The substrate makes NO assumptions about what "rest" means; that's the spec's job.

### Transport specs as authority

`dsl/extdeps/transports/rest.dag`:

```dag
module extdeps.transports.rest

type RestTransportFields {
  method: HttpMethod
  path: String
  body_shape: RestBodyShape
  response_codes: List<Int>
}

// Primitive operations the emitter/interpreter consume.
data rest_invocation: TransportInvocation = {
  steps: [
    SerializeBody,
    OpenConnection,
    IssueMethod,
    ParseResponse
  ]
}
```

`dsl/extdeps/transports/shell.dag`:

```dag
module extdeps.transports.shell

type ShellTransportFields {
  command: String
  args: List<String>
  stdin: ShellStdinShape
  stdout_interpretation: ShellStdoutShape
  exit_code_policy: ExitCodePolicy
}

data shell_invocation: TransportInvocation = {
  steps: [
    ConstructArgv,
    SpawnSubprocess,
    FeedStdin,
    CaptureStdoutStderr,
    MapExitToOutput
  ]
}
```

Each transport spec declares required fields + abstract invocation steps. These are data declarations — no Rust code changes to add a new transport.

### Emitter reads the spec

The Rust emitter does NOT have `match transport.kind { Rest => ..., Shell => ... }`. Instead:

```rust
fn emit_transport_call(
    dag: &Dag,
    ctx: &TargetContext,
    transport_decl: &TransportDeclaration,
) -> Result<String, EmitError> {
    let spec = dag.declaration(transport_decl.spec_ref)?;
    let invocation = spec.find_data("invocation")
        .ok_or(EmitError::MalformedTransportSpec)?;
    let steps = invocation.steps.iter().map(|step| {
        render_step_for_target(ctx, step, &transport_decl.fields)
    }).collect::<Result<Vec<_>, _>>()?;
    Ok(join_rendered(&steps, &ctx.clean_emission.line_ending))
}
```

`render_step_for_target` dispatches on the **step name** (`SerializeBody`, `OpenConnection`, etc.) by looking up per-target realization in the target spec. Adding gRPC = add `extdeps/transports/grpc.dag` declaring its steps + add `rust_grpc_steps` realizations in `rust.dag`.

**No new Rust code in the walker.**

### Interpreter reads the spec

The `dag run` interpreter (Lane 4 Stage 4a) reads the transport spec's `invocation` steps and runs each step using platform primitives. NOT `match transport.kind` — dispatch on step name, with interpreter-side handlers per abstract step (process primitive, http primitive, file primitive).

The interpreter is small: ~N handlers where N = number of abstract primitive steps (process, http, file, possibly grpc). Adding a new transport that composes these primitives adds ZERO interpreter code.

### What the parser changes

Current parser: `transport rest { method: POST, path: "/v1/token" }` lowers to `Transport { kind: "rest", fields: Map<String, Value> }` (string-typed `kind`).

After DB-6:
- `rest` is a DECLARATION REFERENCE to `extdeps/transports/rest.dag`
- Parser resolves the reference at lowering time
- `TransportDeclaration { spec_ref: DeclarationId(rest_spec), fields: [...] }` is the lowered substrate form
- `spec_ref` is a real typed reference, not a string-keyed lookup

Validation at lowering: `rest.dag` declares `RestTransportFields { method, path, body_shape, response_codes }`. Parser verifies the user's field set matches; missing/extra fields fail with diagnostic. This is the "typed" part of the design the original wanted to express — typed per-spec, not per-closed-enum.

### Effect derivation consumes declared behavior

```dag
fn derive_effect_shape_from_transport(
  d: Dag,
  transport: TransportDeclaration
) -> EffectShape {
  let spec = d.declaration(transport.spec_ref)
  match spec.find_declaration("derive_effect") {
    Some(derive_fn) => apply(derive_fn, transport.fields)
    None => UnknownEffect("transport spec does not declare effect derivation")
  }
}
```

Each transport spec declares how effects derive from its fields (e.g., `rest.derive_effect(method, path) = ...`). Compiler doesn't need to know "REST means HTTP means idempotent when PUT with key" — the REST spec declares that.

---

## Rationale

**Why no closed enum?** Because a closed enum forces every new transport to be a compiler change. The thesis explicitly forbids this.

**Why typed per-spec, not per-closed-taxonomy?** Each transport spec CAN declare its own field shape and validation. `rest.dag` declares `RestTransportFields`; user code `transport rest { method: POST }` is validated against THAT shape. No compiler-wide "TransportKind" enum needed.

**Why abstract `steps` in the invocation spec?** N transports × M targets = N*M specific rendering paths. Factoring into abstract primitives (SerializeBody, OpenConnection, etc.) collapses to N+M declarations: each transport declares its steps; each target declares per-step templates. Adding a transport = declare its steps. Adding a target = declare per-step templates.

**Why doesn't this just become another "the compiler knows primitives X, Y, Z" closed list?** Because the steps are target-agnostic names. The compiler doesn't KNOW what `SerializeBody` means; it looks up `rust_serialize_body: CodeTemplate` in the target spec. Missing realization = concrete missing-realization error, not "unsupported."

**Brand-new primitive mechanisms (e.g., WebSocket)?** The transport spec declares its steps. If the step set grows, that's per-spec. The interpreter side may need a new platform primitive — that's the one exception: the interpreter has a fixed set of platform primitives it knows how to invoke (process, http, file, possibly WebSocket). Growing that set IS a compiler change, but only when an entirely new platform mechanism is needed.

---

## Rejected alternatives

**Closed `TransportKind` coproduct with typed sub-variants + per-variant dispatch** (original DB-6 proposal) — **thesis violation per THESIS.md:117-139.** Creates parallel authority. Rejected.

**String-keyed transport with no validation** (pre-DB-6 status quo) — no validation means broken transport declarations fail at emission time, not at compile time. Rejected.

**Move all transport logic into Rust code** — maximal compiler involvement per transport. Rejected.

---

## Implementation notes

### Parser change

`transport <name> { <fields> }` — parser resolves `<name>` as a declaration reference to `extdeps/transports/<name>.dag`. Error if no such spec. Validates field shape per the spec's declared fields type.

### Substrate addition

One new substrate type `TransportDeclaration`. The spec's declared fields type (e.g., `RestTransportFields`) isn't in substrate — it's an ordinary `.dag` type declared in the spec file.

### Emitter dispatch

Walker reads the transport spec's `invocation` data item. Walks the steps list. Per step, looks up per-target realization. Renders.

### Interpreter dispatch

Reads same `invocation` steps. Per step, dispatches to a platform-primitive handler. Platform primitive set IS fixed (small, maybe 5–10 total), but combinations per transport are open-ended.

---

## Example: adding gRPC transport after DB-6 lands

1. Create `dsl/extdeps/transports/grpc.dag`:
   ```dag
   type GrpcTransportFields { service: String, method: String, ... }
   data grpc_invocation: TransportInvocation = {
     steps: [SerializeProto, OpenGrpcChannel, InvokeMethod, ParseProtoResponse]
   }
   ```

2. Add per-target step realizations in `spec/rust.dag`:
   ```dag
   data rust_serialize_proto: CodeTemplate = "{service}::{method}_request({fields})"
   ...
   ```

3. Done. No compiler changes. No interpreter changes. No walker changes.

Users now write `transport grpc { service: "...", method: "..." }` and the compiler routes it through the new spec.

---

## Associations

- **Lane 4 Stage 4a** ([lane4-completion.md](./lane4-completion.md)) — transport declarations + `dag run` interpreter consume this shape
- **Lane 2 Stage 2b** ([lane2-compile-time-proofs.md](./lane2-compile-time-proofs.md)) — idempotency lens reads the transport spec's declared effect derivation
- **`dsl/std/effects.dag`** — `derive_effect_shape` refactored to look up spec-declared derivation
- **`dsl/extdeps/transports/`** — per-transport spec files (rest.dag, shell.dag, grpc.dag, ...)
- **`src/v3/std/substrate.dag`** — add minimal `TransportDeclaration { spec_ref, fields }` carrier. No `TransportKind` enum.
- **`src/v3/spec/rust.dag`, etc.** — per-target step realizations
- **Thesis anchor** — THESIS.md:117-139 (spec IS the implementation; sustainability test)

---

## Acceptance (Lane 4 Stage 4a owns)

- [ ] `src/v3/std/substrate.dag` adds `TransportDeclaration { spec_ref: DeclarationId, fields: List<FieldEntry> }`. **No `TransportKind` enum.**
- [ ] `dsl/extdeps/transports/rest.dag`, `shell.dag` declare required fields + invocation steps
- [ ] Parser lowers `transport <name> { ... }` into typed `TransportDeclaration` with spec-ref resolution
- [ ] Effect derivation looks up spec-declared derivation functions; no string-matching on transport names
- [ ] Adding `grpc.dag` spec + Rust gRPC-step realizations enables gRPC end-to-end with zero Rust compiler/walker/interpreter code changes
- [ ] v2's 16 effects tests have v3 equivalents passing against spec-driven dispatch
- [ ] CI gate: grep for `match.*TransportKind` or `match transport.kind` in Rust returns zero results

---

## Open questions

1. **Initial set of platform primitives the interpreter knows?** Rough estimate: process (Command::spawn), http (reqwest-like), file (stdlib), possibly grpc (tonic-like). Start minimal; grow per concrete need.

2. **Per-spec validation of declared fields** — each transport spec declares its field shape; parser validates user declarations. Overlapping field names across specs have different semantics (e.g., `path` = URL in REST vs filesystem in Shell) — spec-per-transport field types prevent confusion.

3. **Migration path for existing extdeps** — current `dsl/extdeps/cloud/gcp/*.dag` uses `transport rest { ... }`. After DB-6: parser resolves `rest` to `extdeps/transports/rest.dag`, which must EXIST. Creating those spec files is concurrent with this design's implementation.

4. **Transports that don't decompose into steps?** If a transport needs custom logic not expressible as step composition, it needs to break into primitives addable to the interpreter. Pressure on such transports to decompose is GOOD — keeps primitive-set small. If genuinely bespoke, that's a thesis-level discussion (growing the primitive set), not a stage-level bypass.
