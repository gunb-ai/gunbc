# Branch D Contract — D.1 + D.2 (W4.6 design)

> **Status:** DESIGN FOR W4.6 — Class 2 contract (implementation D.3–D.6 = W5.3).
> **Author:** Cross-target Emission Mgr (`silent-bear-54`), 2026-06-02.
> **Work item:** `node://adhoc-b4e8b554-bae` · PM dispatch `msg_2f3dd88f`.
> **Worksheet:** `v4-cross-target-emission-rr-d-worksheet-2026-06-02.md`.

## Stack-on-stable (2026-06-02)

| Dependency | State | Notes |
|------------|--------|--------|
| C.1–C.5 substrate | ✅ `main` (#4297, #4322 C.3b) | Shared `ConcreteSyntaxToken`; `05_emit` thin |
| C.6–C.10 RCA | In flight | Interface freeze sent; not blocking **contract** |
| G.0 schema | ✅ `main` (#4306) | `PerTargetGroundingReceipt` carriers |
| G.1.3 Go / G.1.4 TS | ✅ `main` | Per PM |
| G.1.1 Rust / G.1.2 Python | CI / in flight | Contract cites carriers; do not block on all G.1 |

Design-only: **no compiler substrate edits** in this PR except optional grep-receipt wiring in a follow-on if needed.

## One tree, two emission channels (thesis / §2.3)

One semantic **`InferredTree`** (substrate Node IR after infer) is the shared input. It does **not** imply one renderer:

| Channel | Owner | Output | Branch |
|---------|--------|--------|--------|
| **Shape A** | Compiler `translate ∘ serialize_target` | `TargetSource` per `TargetModel` (Rust, Python, Go, TS, …) | **C** (landed substrate) |
| **Shape B** | User `.dag` programs over typed **format/framework facts** | OpenAPI YAML, SQL DDL, React TSX/JSX strings, … | **D** (this contract) |

**D.1** and **D.2** formalize Shape B only. Shape A multi-target behavior is **consumed** from Branch C (RCA charters); D must not add compiler format emitters.

```text
InferredTree (shared IR)
  ├─ Shape A: 05_emit → 06_translate → TargetSource     [compiler — C]
  └─ Shape B: extdeps facts → user .dag emitter → artifact [user program — D]
```

## §2.3.1 GUARD (ratified — do not violate)

- Shape B = user `.dag` programs over typed values in `v4.extdeps.formats.*` / `v4.extdeps.frameworks.*`.
- Compiler **MUST NOT** grow OpenAPI / SQL / React / Markdown **render authority** in `05_emit` / `06_translate`.
- `v4.extdeps` holds **external facts** (P1); gunbc emit policy stays Shape A + user programs.

## §2.7.4 — source / IR / receipt (three surfaces)

| Surface | Authority | Example |
|---------|-----------|---------|
| **Source** | Canonical `.dag` text / ingestion | `source.dag` round-trip (Branch H) |
| **IR** | `InferredTree` / Node substrate | Compiler stages |
| **Receipt** | JSON / TestClaim / host verification | `PerTargetGroundingReceipt`, format round-trip claims |

**Contract rule:** Format artifacts (`OpenApiDocument`, `SqlTableDefinition`, React projection strings) are **receipt or user-program output**, never aliased as `TargetSource`. Probes and dashboards must name paths separately.

---

## D.1 — Format contract (OpenAPI + SQL)

**Deliverable:** One Node tree drives **≥1 Shape B format artifact class** via **distinct user programs**, not via compiler `emit`.

### Authority (existing — live on `main`)

| Module | Role | Parse/emit axes |
|--------|------|-----------------|
| `v4.extdeps.formats.openapi` | OAS 3.1 document facts | Gated T-6/T-7; wire types landed |
| `v4.extdeps.formats.sql` | ISO 9075 DDL/relational facts | DDL carriers landed; parse/emit gated |

Supporting facts: `json`, `yaml`, `json_schema` (OpenAPI media); no compiler coupling today.

### Allowed consumer graph

```text
InferredTree + grounding facts (G.*)
  → projection morphism (user .dag or future W5 helper) : Tree → OpenApiDocument | Sql*
  → format-local serialize (user .dag: concat/fold/match on format types)
  → artifact file (openapi.yaml, migration.sql)
  → optional external toolchain (openapi-generator, psql)
```

**Invariants:**

1. **No** `import v4.extdeps.formats.*` under `src/v4/compiler/05_emit.dag` or `06_translate.dag`.
2. Format **parse** and **emit** morphisms live in format modules or user programs — not compiler stages.
3. Multiple format artifacts from one tree = **multiple user programs** (or one program with multiple outputs), not multiple compiler targets.

### Anti-import receipt (W4.6 / CI)

```bash
# Falsification probe — target: zero matches after contract PR
rg 'v4\.extdeps\.formats' src/v4/compiler/ || true
rg 'v4\.extdeps\.frameworks' src/v4/compiler/ || true
```

Current tree: **0 matches** (verified 2026-06-02). Contract PR may add a `test/claim` or CI grep row citing this receipt; no substrate change required.

### D.1 non-goals (W4.6)

- Full OpenAPI/SQL parse/emit implementation (W5 / format lanes).
- Dissolving `openapi-json-yaml` selector coproducts (W5 when media-type projection exists).
- T-24 CI YAML emission (Compiler Spine).

---

## D.2 — Framework contract (React)

**Deliverable:** One Node tree drives **≥1 Shape B framework artifact** via user `.dag` programs projecting `v4.extdeps.frameworks.react` carriers.

### Authority (existing)

| Module | Role |
|--------|------|
| `v4.extdeps.frameworks.react` | React 19.2 component/hook/element facts (T-4.7) |

### Allowed consumer graph

```text
InferredTree + cross-target Shape A modules (optional)
  → user .dag: walk ReactComponent / ReactHookSite / ReactElement facts
  → emit TSX/JSX string (Shape A target program that *prints* framework syntax)
  → OR downstream bundler (external)
```

**Distinction:** The **compiler may emit a Rust/Python/TS program** (Shape A) that, when run, prints React. The compiler must **not** embed JSX template strings in `06_translate` as a pseudo-target.

### Framework selection (coordination note)

W5.3 D.3–D.6 omni Shape B demos remain gated on **bright-moth-style** framework selection + full G.1. This contract only freezes **React** as the reference framework carrier in `extdeps`; alternate frameworks get new `extdeps/frameworks/*` modules, not compiler paths.

### D.2 non-goals (W4.6)

- Compiler JSX/TSX emit helpers.
- Populating every `ReactHookSite` variant in emission tables.
- Bundler/vite integration (external).

---

## W5.3 queue (after this contract merges)

| Row | Scope | Gate |
|-----|--------|------|
| D.3–D.6 | Omni Shape B demos (OpenAPI round-trip, SQL DDL sample, React projection sample) | Full G.1 + framework selection |
| D.1 impl | Typed boundary records → `TestClaim` / RoundTrip where PROVEN | Format lanes T-6/T-7 |

---

## Preflight checklist (contract PR)

- [ ] Doc describes live `extdeps` modules (P1).
- [ ] GUARD §2.3.1 encoded in D.1/D.2 consumer graphs.
- [ ] §2.7.4 three-surface table present.
- [ ] Anti-import grep receipt documented (0 compiler imports today).
- [ ] No `05_emit` / `06_translate` edits.
- [ ] RR-D worksheet cross-link updated to point here.

## Handoffs

- **RCA mgrs:** Shape A only; format/framework rows are not `TargetModel` entries.
- **bright-moth / Go / vivid-eagle / vivid-lynx:** G.1 fact bundles feed projections; D does not duplicate G groundings.
- **Branch H:** Source authority separate; D consumes IR, not `dag-artifact.json` as source.
