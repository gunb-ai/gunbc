# Transport Extdep Design

## Problem

The compiler hardcodes transport knowledge: `TransportKind` enum,
`transport_kind()` string classifier, per-language emit functions for
REST/Shell/File. Adding a transport requires compiler changes. This
violates the same invariant as hardcoded language knowledge — the
compiler should read transport facts from `.dag` definitions, not
own them.

## Architecture

Transports are external execution capabilities, structurally identical
to languages: an external system (HTTP, subprocess, filesystem) that
the compiler must target. They follow the same extdep model.

```
dsl/extdeps/
  transports/
    rest.dag         Layer 1: REST/HTTP transport facts
    shell.dag        Layer 1: subprocess transport facts
    file.dag         Layer 1: filesystem transport facts
  languages/
    rust/
      transport.dag  Per-language: Rust runtime deps + struct field templates
    python/
      transport.dag  Per-language: Python runtime deps
    go/
      transport.dag  Per-language: Go runtime deps
```

**Local is NOT a transport.** A service operation with no transport
binding is a direct function call. No extdep, no config, no runtime
dependency. The absence of a transport IS the local case.

## Layer 0: Transport Primitives (std/types.dag — already exists)

These types already exist in `dsl/std/types.dag:410-452`:

```dag
type TransportRequest { method: String, url: String, headers: Json, body: String }
type TransportResponse { status: Int, headers: Json, body: String }
type ShellResponse { exit_code: Int, stdout: String, stderr: String }
type RestResponse { status: Int, headers: Json, body: Json }
type FileResponse { path: String, success: Bool, content: String }
```

No changes needed at Layer 0.

## Layer 1: Transport Definitions

### extdeps/transports/rest.dag

```dag
// extdeps/transports/rest.dag -- HTTP REST transport
//
// Spec: RFC 9110 (HTTP Semantics)
//       https://www.rfc-editor.org/rfc/rfc9110
// Auth: RFC 7235 (HTTP/1.1 Authentication)
//       https://www.rfc-editor.org/rfc/rfc7235
// URI:  RFC 3986 (Uniform Resource Identifier)
//       https://www.rfc-editor.org/rfc/rfc3986
//
// Models the REST transport binding for service operations.
// A service operation with `transport rest { ... }` executes via
// HTTP request to the configured endpoint.

module extdeps.transports.rest

import std.types { String, Bool, Secret }

// RFC 9110 Section 9: request methods
type HttpMethod = GET | POST | PUT | PATCH | DELETE | HEAD | OPTIONS

// Configuration schema for a REST transport binding.
// Parser validates that transport blocks labeled "rest" conform.
type RestTransportConfig {
  base_url: String              // RFC 3986 absolute-URI
  auth_token: Secret?           // RFC 7235 Section 4.2: credentials
  auth_header: String?          // Header field name (default: "Authorization")
  content_type: String?         // RFC 9110 Section 8.3 (default: "application/json")
  async_required: Bool          // REST calls are inherently async (network I/O)
}

// Default values for optional config fields.
data rest_default_auth_header: String = "Authorization"
data rest_default_content_type: String = "application/json"
data rest_async_required: Bool = true

// Service struct fields this transport requires.
// The emitter reads this list to generate config struct fields.
data rest_config_fields: List<ConfigField> = [
  ConfigField { name: "base_url", type_name: "String", required: true,
                default_value: "http://localhost" },
  ConfigField { name: "auth_token", type_name: "String", required: false,
                default_value: "" }
]
```

### extdeps/transports/shell.dag

```dag
// extdeps/transports/shell.dag -- Subprocess transport
//
// Spec: POSIX.1-2017 Shell Command Language
//       https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html
// Exec: POSIX.1-2017 exec family
//       https://pubs.opengroup.org/onlinepubs/9699919799/functions/exec.html
//
// Models the shell transport binding for service operations.
// A service operation with `transport shell { ... }` executes via
// subprocess invocation with captured stdout/stderr.

module extdeps.transports.shell

import std.types { String, Bool, Int }

// Exit code interpretation per POSIX.1-2017 Section 2.8.2.
// Exit status 0 = success, nonzero = failure.
type ExitBehavior {
  success_code: Int       // default: 0
  capture_stdout: Bool    // default: true
  capture_stderr: Bool    // default: true
}

type ShellTransportConfig {
  working_dir: String?    // POSIX PWD; None = inherit from parent
  async_required: Bool    // false: subprocess blocks (can be run async by caller)
}

data shell_default_exit_behavior: ExitBehavior = ExitBehavior {
  success_code: 0, capture_stdout: true, capture_stderr: true
}

data shell_config_fields: List<ConfigField> = [
  ConfigField { name: "working_dir", type_name: "String?", required: false,
                default_value: "." }
]
```

### extdeps/transports/file.dag

```dag
// extdeps/transports/file.dag -- Filesystem transport
//
// Spec: POSIX.1-2017 File I/O
//       https://pubs.opengroup.org/onlinepubs/9699919799/functions/read.html
//       https://pubs.opengroup.org/onlinepubs/9699919799/functions/write.html
//
// Models the file transport binding for service operations.
// A service operation with `transport file { ... }` executes via
// filesystem read/write at the configured path.

module extdeps.transports.file

import std.types { String, Bool }

// File operations per POSIX.1-2017 Section 2.5.1.
type FileOp = FileRead | FileWrite | FileDelete

type FileTransportConfig {
  base_path: String       // POSIX pathname prefix
  encoding: String?       // default: "utf-8"
  async_required: Bool    // false: filesystem ops are typically sync
}

data file_config_fields: List<ConfigField> = [
  ConfigField { name: "base_path", type_name: "String", required: true,
                default_value: "." }
]
```

## Shared Schema Type

Used by all transport definitions to declare what service struct
fields they need:

```dag
// In std/types.dag or extdeps/transports/transport.dag
type ConfigField {
  name: String
  type_name: String
  required: Bool
  default_value: String
}
```

## Per-Language Transport Rendering

Each language extdep declares what runtime dependencies a transport
needs and provides template data for struct field generation.

### extdeps/languages/rust/transport.dag

```dag
// extdeps/languages/rust/transport.dag
//
// Rust runtime dependencies and rendering facts for transports.
// The compiler reads these to generate Cargo.toml dependencies
// and service struct implementations.

module extdeps.languages.rust.transport

// Crate dependencies per transport.
// These are facts about the Rust ecosystem, not compiler knowledge.
data rust_rest_dependencies: List<String> = ["reqwest", "serde_json", "tokio"]
data rust_shell_dependencies: List<String> = []
data rust_file_dependencies: List<String> = []

// Rust type for each config field (may differ from .dag type).
data rust_rest_field_types: Map<String, String> = {
  "base_url": "String",
  "auth_token": "String"
}

data rust_shell_field_types: Map<String, String> = {
  "working_dir": "Option<String>"
}

data rust_file_field_types: Map<String, String> = {
  "base_path": "String"
}
```

## How the Compiler Uses This

### Parse time

Parser sees `transport rest { base_url: "..." }`. It:
1. Resolves `rest` to `extdeps.transports.rest` (import resolution)
2. Creates transport node with edge to the rest transport definition
3. Parses config properties against `RestTransportConfig` schema
4. Rejects unknown transport names as parse errors (no fabrication)

### Resolve time

Resolver validates config property types against the transport
definition schema. `base_url` must be String, `auth_token` must be
Secret, etc.

### Emit time

Emitter reads from the transport definition node:
1. `rest_config_fields` → generates service struct fields
2. Language extdep `rust_rest_field_types` → Rust types for those fields
3. Language extdep `rust_rest_dependencies` → Cargo.toml entries
4. Transport call rendering is still procedural (Lane C moves it to
   language plugins), but reads FACTS from the extdep instead of
   hardcoding them

### What this eliminates

- `TransportKind` enum: deleted. Transport identity = edge to definition.
- `transport_kind()`: deleted. No string-to-enum conversion.
- `make_transport_node(name: "rest", ...)`: replaced. Transport node
  carries edge to definition, not a name string.
- Fallback to Local: deleted. No transport = no transport node.
  Parser rejects unknown transport names.

### What this does NOT do yet (Lane C)

- Transport call rendering templates are still procedural emit code.
  Lane C extracts them to language plugin `.dag` files.
- The `emit_rest_call` / `emit_shell_call` functions still exist in
  the compiler, but they read config facts from the extdep instead
  of hardcoding REST/Shell/File knowledge.

## Open Questions

1. **ConfigField type location**: should it live in `std/types.dag`
   (Layer 0) or `extdeps/transports/transport.dag` (Layer 1)?

2. **Transport resolution mechanism**: how does the parser resolve
   `rest` to `extdeps.transports.rest`? Is it an import? A keyword?
   A well-known path convention?

3. **Schema validation**: should the compiler validate transport config
   properties against the `RestTransportConfig` type at parse time
   or resolve time?

4. **Custom headers / env vars**: REST headers and Shell env vars are
   open-ended (user-defined keys). How does the config schema handle
   the open set while validating the known fields?
