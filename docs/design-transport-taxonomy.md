> Part of: [post-l15-phase-plan.md](./post-l15-phase-plan.md) | Unblocks: Lane 4 Stage 4a, Lane 2 Stage 2b (idempotency lens reads transport facts)

# Design DB-6 — Transport type taxonomy

**Design blocker:** DB-6
**Consumers:** Lane 4 Stage 4a (transport declarations + `dag run` interpreter); Lane 2 Stage 2b (idempotency lens consumes `TransportVariant` to derive effect shape); `dsl/extdeps/*/` declarations (already use `transport rest {...}` / `transport shell {...}` as string-tagged today — migrate to typed)
**Status:** Design ready for implementer review.

---

## Problem

Today `dsl/extdeps/*.dag` declares transports as string-annotation blocks:

```dag
operation Exchange {
  ...
  transport rest { method: POST, path: "/v1/token" }
}

operation DeployScript {
  ...
  transport shell { command: "gcloud", args: ["auth", "login"] }
}
```

The `rest` and `shell` keywords are **parser-level tags** that lower into generic `Transport` nodes carrying `kind: String`. Consumers (effect derivation, Rust emitter's HTTP client generation) reconstruct per-variant fields from string matching on `kind`.

This is the same name-based-dispatch antipattern as `rust_` prefix filtering. Fixing it: declare the transport taxonomy as a typed coproduct, parser lowers to the typed variant directly, consumers dispatch on variant not on string.

Bounded closed set required for Lane 4a: which transports to ship?

---

## Design

### The taxonomy (closed coproduct)

```dag
// src/v3/std/transports.dag (new)
module std.transports

import std.list { List }
import std.types { HttpMethod, Url, NonEmptyStr }

// 🟢 TERMINAL. Transport kinds are the closed set of runtime
// mechanisms gunbc knows how to invoke. Each variant carries its
// own typed payload — no string matching on "kind".
type TransportKind
  = RestTransport(RestTransportData)
  | ShellTransport(ShellTransportData)
  | GrpcTransport(GrpcTransportData)
  | LocalFunctionTransport(LocalFunctionData)

// --- REST --------------------------------------------------------

// HTTP request/response over REST. Most cloud APIs (GCP, AWS,
// GitHub, etc.) are REST.
type RestTransportData {
  method: HttpMethod                 // GET, PUT, POST, DELETE, PATCH, HEAD, OPTIONS
  path: String                       // e.g. "/v1/secrets/{id}"
  body_shape: RestBodyShape
  response_codes: List<Int>          // expected 2xx codes; fail-closed on unexpected
}

type RestBodyShape
  = NoBody                           // GET / DELETE / HEAD
  | JsonBody(DeclarationId)          // type of the body struct
  | FormUrlEncoded(List<String>)     // form field names
  | RawBytes                         // for upload/download

// --- Shell --------------------------------------------------------

// Shell command invocation. `dag run` spawns a subprocess;
// emission generates `Command::new(...)` (Rust) /
// `subprocess.run(...)` (Python) / `exec.Command(...)` (Go).
type ShellTransportData {
  command: String                    // the executable, e.g. "gcloud"
  args: List<String>                 // argv; may contain {placeholders}
  stdin: ShellStdinShape
  stdout_interpretation: ShellStdoutShape
  exit_code_policy: ExitCodePolicy
}

type ShellStdinShape
  = NoStdin
  | LiteralStdin(String)
  | InputFieldStdin(String)          // feeds one input field as stdin

type ShellStdoutShape
  = IgnoreStdout
  | ParseJson(DeclarationId)
  | ParseLines                       // List<String> one per line
  | RawStdout

type ExitCodePolicy
  = ZeroIsSuccess
  | SpecificExitCodes(List<Int>)

// --- gRPC ---------------------------------------------------------

// gRPC calls. Alternative to REST for some Google APIs and
// intra-service calls. Binary protocol, streaming supported.
type GrpcTransportData {
  service: String                    // fully-qualified service, e.g. "google.cloud.storage.v2.Storage"
  method: String                     // e.g. "GetObject"
  request_type: DeclarationId        // typed request message
  response_type: DeclarationId       // typed response message
  streaming: StreamingMode
}

type StreamingMode
  = Unary                            // single request, single response
  | ServerStreaming                  // single request, response stream
  | ClientStreaming                  // request stream, single response
  | BidiStreaming                    // both

// --- Local function (for intra-program calls) -------------------

// Not a "transport" in the network sense — models direct function
// invocation for ops that are local to the generated program. This
// exists so that workflows can mix local helpers and remote ops
// without special-casing.
type LocalFunctionData {
  callable: DeclarationId            // the function being invoked
}
```

### Migration path from string tags

`dsl/extdeps/*.dag` currently has:

```dag
transport rest { method: POST, path: "/v1/token" }
```

Parser lowers this into a `TransportDeclaration` with `TransportKind::RestTransport(RestTransportData { method: Post, path: "/v1/token", body_shape: JsonBody(...), ... })`.

Syntax changes are minimal: the parser rule for `transport <kind> { ... }` stays; the lowered AST uses the typed coproduct.

### Consumer dispatch pattern

Effect derivation (v2's `derive_effect_shape(method, path)`) becomes:

```dag
fn derive_effect_shape(transport: TransportKind) -> EffectShape {
  match transport {
    RestTransport(rest) => derive_rest_effect(rest.method, rest.path)
    ShellTransport(shell) => derive_shell_effect(shell.command, shell.args)
    GrpcTransport(grpc) => derive_grpc_effect(grpc.service, grpc.method, grpc.streaming)
    LocalFunctionTransport(local) => ReadEffect
  }
}
```

No string comparison anywhere. Each transport has its own derivation function.

### `dag run` interpreter dispatch

```rust
// Rust-side for Lane 4 Stage 4a
fn invoke_transport(
    transport: &TransportKind,
    inputs: &TransportInputs,
) -> Result<TransportOutput, TransportError> {
    match transport {
        TransportKind::RestTransport(data) => invoke_rest(data, inputs),
        TransportKind::ShellTransport(data) => invoke_shell(data, inputs),
        TransportKind::GrpcTransport(data) => invoke_grpc(data, inputs),
        TransportKind::LocalFunctionTransport(data) => invoke_local(data, inputs),
    }
}
```

Each `invoke_*` function handles runtime concerns (HTTP client, subprocess spawn, gRPC stub) specific to the transport. Shared machinery in `invoke_transport` handles retry, rate limiting, logging.

### Target emitters

Each target spec declares how to emit a call for each transport variant:

```dag
// spec/rust.dag additions
data rust_rest_emission: RestEmissionStrategy = {
  client_module: "reqwest"
  construct_client: "reqwest::Client::new()"
  call_template: "{client}.{method_lowercase}(\"{path}\").send()"
  ...
}

data rust_shell_emission: ShellEmissionStrategy = {
  module: "std::process::Command"
  construct_template: "std::process::Command::new(\"{command}\").args(&[{args}]).output()"
  ...
}
```

Generic walker (DB-2) dispatches on transport variant + reads the target's emission strategy for that variant. Adding gRPC support to Rust = adding `rust_grpc_emission` data item; no walker change.

---

## Rationale

**Why closed set of 4 variants?** Because these cover every transport gunbc has actually needed:
- REST: all cloud SaaS APIs (GCP, AWS, GitHub, Stripe, …)
- Shell: local tool invocation (gcloud CLI, terraform, docker, kubectl)
- gRPC: some Google APIs, intra-service calls
- LocalFunction: in-program function dispatch (not strictly "transport" but models calls uniformly)

Anything else can be added later. Shipping 4 well is better than enumerating 12 speculatively.

**Why not "any transport is a string + bag of attributes" (open extensibility)?** Because open-string sets inherit the name-dispatch problem. If the goal is "users declare new transports," add them as substrate-level additions (requires substrate PR), not runtime string soup.

**Why carry `RestBodyShape`, `ShellStdinShape`, etc. as sub-types?** Because the variant choice determines downstream code generation (e.g., how to serialize the body, how to feed stdin). Opaque strings here would push the decision to the emitter, which would then need to string-match. Keep decisions in the substrate.

**Why include LocalFunctionTransport?** Because workflows mix local helpers and remote calls; if local calls aren't "transports," every consumer needs a separate branch for local-vs-remote. Making local-dispatch a transport variant unifies the handling.

**Why typed `HttpMethod` (existing) but String `method` field in gRPC?** Because HTTP methods are a closed verb set; gRPC methods are arbitrary user-declared names on a service. Different modeling fits different domains.

**Why separate `response_codes: List<Int>` on REST?** Because extdeps already declares expected response codes per op. Surfacing them structurally on the transport lets the generic walker emit exhaustive response handling. Removes another layer of string-keyed lookup.

---

## Rejected alternatives

**Single `Transport` type with `kind: String` + `attributes: Map<String, String>`** — opaque, name-dispatched, every consumer reinvents the shape. Rejected.

**Separate trait per transport in Rust** — implementation detail; not a substrate concern. Taxonomy is data. Rejected (at substrate level).

**Include WebSocket / queue transports** — no current consumer needs them. Add when a real use case arrives. Rejected for now.

**Transport declared as a first-class type (like Behavior), not a Conj sub-type** — overcomplicates. Transport is data tied to an operation, not an independent execution node. Rejected.

**Model async vs sync as a transport variant** — wrong axis; async is an emission mode (Lane 4d), not a transport kind. Rejected.

---

## Implementation notes

### Parser changes

- `transport rest { ... }` currently parses into `Transport { kind: "rest", fields: List<(String, Value)> }` (string-typed)
- After DB-6: parses into `TransportKind::RestTransport(RestTransportData { method: ..., path: ..., ... })`
- The `rest` / `shell` / `grpc` / `local` keywords become variant constructors recognized by the parser (similar to how v3 recognizes `Empty` / `Cons` as variant patterns today)

### Validation at lowering

Each transport variant has required fields; lowering fails with diagnostic if any is missing. Example:

```dag
transport rest {
  method: POST
  // path missing → LoweringError: "REST transport requires `path` field"
}
```

Stage 1a of Lane 4 covers this.

### Effect derivation simplification

`dsl/std/effects.dag`'s `derive_effect_shape(method: HttpMethod, path: PathTemplate)` becomes `derive_rest_effect(method, path)` and is called from the top-level `derive_effect_shape(transport: TransportKind)` dispatcher. v2's per-op effect tests migrate directly.

### Backward compatibility during migration

`dsl/extdeps/*.dag` files stay valid — parser converts `transport rest { method: POST, path: "/..." }` to the typed variant transparently. No file edits needed when DB-6 lands; user-facing syntax unchanged.

---

## Associations

- **Lane 4 Stage 4a** ([lane4-completion.md](./lane4-completion.md)) — this is the core substrate change for transports
- **Lane 2 Stage 2b** ([lane2-compile-time-proofs.md](./lane2-compile-time-proofs.md)) — idempotency lens reads `TransportKind` to derive `EffectShape`
- **`dsl/std/effects.dag`** — `derive_effect_shape` refactored to dispatch on `TransportKind`
- **`dsl/extdeps/*.dag`** — no syntax changes; lowering updated
- **`src/v3/std/transports.dag`** — NEW file with the taxonomy
- **`src/v3/spec/rust.dag`** — add `rust_rest_emission`, `rust_shell_emission`, etc. for each variant
- **`src/v3/compiler/src/emit.rs`** (DB-2 walker) — dispatch on `TransportKind` variant when emitting transport calls

---

## Acceptance (Lane 4 Stage 4a owns)

- [ ] `std/transports.dag` declares `TransportKind` coproduct with 4 variants + sub-types per variant
- [ ] Parser lowers `transport rest {...}` / `shell` / `grpc` / `local` into the typed variant; lowering fails on missing required fields
- [ ] `dsl/extdeps/cloud/gcp/*.dag` (STS, Secret Manager, IAM) round-trip through the new pipeline with zero syntax changes
- [ ] Effect derivation dispatches on `TransportKind` variant (no string matching)
- [ ] `dag run` interpreter invokes transports via `invoke_transport(TransportKind, inputs)` dispatch
- [ ] Target emitters each declare `*_rest_emission`, `*_shell_emission`, etc. strategies in their spec
- [ ] v2's 16 effects tests (from `src/v2/tests/src/effects.rs`) have v3 equivalents passing against the new typed transports

---

## Open questions

1. **Is 4 variants enough?** Cover REST, shell, gRPC, local. Possibly add WebSocket / message queue / SSH later. Defer until concrete need.

2. **Do transports need a `timeout: Duration?` field, a `retry: RetryPolicy?` field, a `rate_limit: RateLimit?` field?** Likely yes — today these are declared at `service { config { ... } }` level in extdeps. Decision: keep them at service/operation level, not on `TransportKind`. The transport variant is WHAT is being called; service config is HOW.

3. **Authentication declaration** — today extdeps declare `AuthScheme` per service. Stays at service level, not on transport. Confirmed.

4. **Should `RestTransportData.path` be a parsed `PathTemplate` (typed with parameters marked)** not a `String`? v2's `dsl/std/http_path.dag` declares `PathTemplate`; reuse rather than re-parse. Yes: `path: PathTemplate`.
