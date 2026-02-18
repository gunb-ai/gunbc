# Shared Abstractions: gunbc + the-gunbai

**Status**: Implemented — February 2026
**Companion**: [`dsl-codegen-roadmap.md`](./dsl-codegen-roadmap.md) (Track F1)

This document describes the shared abstraction layer between the `gunbc` and
`the-gunbai` repositories. Both repos model computation as directed acyclic
graphs (DAGs) with typed ports, but diverge in domain-specific details. The
shared abstractions defined here enable cross-repo compatibility without
creating a direct crate dependency.

---

## 1. EdgeKind

**gunbc**: `gunbc_ir::EdgeKind` (`core/ir/src/dag.rs`)
**gunbai**: `gunbai_ir::EdgeKind` (`crates/gunbai-ir/src/graph.rs`)

Both repos now define the same 3-variant enum:

| Variant | Data? | Ordering? | Gating? | Use case |
|---|:---:|:---:|:---:|---|
| `DataFlow` | yes | yes | no | Standard value transfer |
| `Control` | no | yes | no | Sequencing side effects |
| `TriggerGate` | yes | yes | yes | Conditional execution |

**gunbc details**: `Edge::new()` defaults to `DataFlow`. New constructors
`Edge::control()` and `Edge::trigger()` create the other kinds. The `kind`
field has `#[serde(default)]` so existing serialized DAGs deserialize
correctly (all edges become `DataFlow`).

**API surface**:
- `EdgeKind::carries_data()` — true for DataFlow and TriggerGate
- `EdgeKind::creates_ordering()` — always true
- `EdgeKind::is_gating()` — true only for TriggerGate
- `Edge::carries_data()` / `Edge::is_gating()` — delegate to kind

---

## 2. Effect

**gunbc**: `gunbc_ir::Effect` (`core/ir/src/effect.rs`)
**gunbai**: `gunbai_types::Effect` (`crates/gunbai-types/src/effect.rs`)

2-bit classification (not an enum):

| `writes_world` | `deterministic` | Constant | Meaning |
|:---:|:---:|---|---|
| false | true | `PURE` | Safe to cache, no side effects |
| false | false | `READ` | Non-deterministic read (LLM, polling) |
| true | true | `WRITE_DETERMINISTIC` | Idempotent mutation |
| true | false | `WRITE` | Non-deterministic side effect |

**API surface**:
- `Effect::cacheable()` — true iff `deterministic && !writes_world`
- `Effect::requires_policy()` — true iff `writes_world`
- `Default` is `PURE`

**Design note**: This is orthogonal to `ObligationCategory` from
`daglang-lower`. ObligationCategory describes *what obligation* a node
carries for test generation; Effect describes *whether* the node mutates
external state or can be memoized.

---

## 3. Value Bridge

**gunbc**: `gunbc_ir::value_bridge` (`core/ir/src/value_bridge.rs`)

The two repos' `Value` enums overlap in primitives but diverge in
domain-specific variants:

### Shared variants (lossless conversion)

| gunbc | gunbai |
|---|---|
| `Unit` | `Null` |
| `Bool(bool)` | `Bool(bool)` |
| `Str(String)` | `String(String)` |
| `Int(i64)` | `Int(i64)` |
| `List(Vec<Value>)` | `List(Vec<Value>)` |
| `Json(serde_json::Value)` | `Json(serde_json::Value)` |
| `Secret(SecretString)` | `Secret(SecretRef)` |

### gunbc-only variants

| Variant | Purpose |
|---|---|
| `Set(Vec<Value>)` | Unordered unique collection |
| `Map(BTreeMap<String, Value>)` | String-keyed map |
| `Request(TransportRequest)` | I/O boundary: outgoing |
| `Response(TransportResponse)` | I/O boundary: incoming |
| `Skipped` | Guard-skipped execution |

### gunbai-only variants

| Variant | Purpose |
|---|---|
| `Float(f64)` | Floating point (gunbc uses `Json` for floats) |
| `Bytes(Vec<u8>)` | Raw bytes |
| `Artifact(ArtifactRef)` | Content-addressed large data |
| `Capability(CapabilityHandle)` | Typed auth handles |

### Bridge API

- `classify_value(value) -> ValueCategory` — Shared or GunbcOnly
- `to_bridge_json(value) -> Option<serde_json::Value>` — convert to JSON wire format
- `from_bridge_json(json) -> Value` — convert from JSON wire format

The JSON wire format is the recommended serialization path for cross-repo
data exchange. Secrets are redacted to `"***"`. gunbc-only I/O variants
(`Request`, `Response`, `Skipped`) return `None`.

---

## 4. PortType

**gunbc**: `gunbc_ir::PortType` (`core/ir/src/port_type.rs`)
**gunbai**: `gunbai_types::PortType` (`crates/gunbai-types/src/block.rs`)

Both repos now define a structural type enum:

| Variant | TypeId string | Notes |
|---|---|---|
| `Json` | `"Json"` | JSON-serializable data |
| `String` | `"String"` | Plain string |
| `Bytes` | `"Bytes"` | Raw bytes |
| `Bool` | `"Bool"` | Boolean |
| `Int` | `"Int"` | Integer |
| `Float` | `"Float"` | Floating point |
| `List(inner)` | `"List<inner>"` | Recursive list |
| `Secret` | `"Secret"` | Sensitive, never logged |
| `Any` | `"Any"` | Wildcard (default) |

**gunbc bridge**: gunbc's existing `TypeId` (opaque string) converts to/from
`PortType` via `From<&TypeId>` and `PortType::to_type_id()`. Legacy TypeId
strings are handled:
- `"StringList"` → `List(String)`
- `"Unit"` / `"Void"` → `Any`
- `"OptionalString"` → `String`
- Unknown strings → `Any` (fail-open)

**Compatibility rules**:
- `Any` is compatible with everything
- `Secret` is strict — only compatible with `Secret` or `Any`
- `List` checks inner type recursively

---

## 5. Reconciliation Strategy

The repos remain independent workspaces (no cross-crate dependency). Shared
abstractions are defined independently in each repo with identical semantics.
This is intentional:

1. **No coupling**: Either repo can evolve independently
2. **Wire compatibility**: JSON serialization is the bridge format
3. **Semantic parity**: Same type names, same method names, same behavior

If a direct crate dependency is ever added (e.g., `gunbc-ir` depends on
`gunbai-types`), these types should be consolidated into the shared crate
and re-exported from both.

---

## File Inventory

| File | Types | Purpose |
|---|---|---|
| `core/ir/src/dag.rs` | `EdgeKind`, `Edge` | Edge semantics (F1.1) |
| `core/ir/src/effect.rs` | `Effect` | 2-bit effect classification (F1.2) |
| `core/ir/src/value_bridge.rs` | `ValueCategory`, bridge fns | Value conversion (F1.3) |
| `core/ir/src/port_type.rs` | `PortType` | Structural port types (F1.4) |
