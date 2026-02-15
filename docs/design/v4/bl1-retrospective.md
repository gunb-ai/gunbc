# BL1 Retrospective — Problem Analysis

**Status**: Draft — January 2026
**Scope**: What worked, what didn't, and why — analysis of the understanding
system's first iteration. For the V2 design, see
[`v2-contracts-design.md`](v2-contracts-design.md).

---

## Executive Summary

BL1 attempted to unify the behavioral vocabulary across 19 tool
understandings. It succeeded at three things and failed at three things:

**Succeeded:**
- Unified DAG (`gunbai-dag`) — one graph model, one vocabulary
- `SetSpec<T>` — explicit intent replaces ambiguous empty slices
- Upsert pattern — typed phases with compile-time validation

**Failed:**
- Tool-first modeling → divergent behavior vocabulary (19 tools, no shared names)
- String-typed semantic channels → silent bugs in 6 of 7 channels
- Missing behavior sub-DAG → annotations tag behaviors but don't connect them

**The fundamental mistake**: V1 modeled tool facts; V2 models tool
obligations. The understanding system allowed modeling the world however you
want, then optionally tagging it with epistemological constructs. V2 makes
the missing middle DAG structural and the semantic channels typed.

---

## 1. The DAG That Was Half-Built

The system has three natural levels of causal structure. Only two were built:

| Level | Structure | Status |
|---|---|---|
| Understanding → Understanding | `depends_on` with `ancestors()`/`descendants()` | Working |
| Behavior → Behavior (within a tool) | Should be sub-DAG from pattern template | **Missing** |
| Block → Block (execution graph) | GraphIR with typed edges, ports, waves | Working |

The top level knows *which tools* relate. The bottom level knows *which
operations* connect. Nothing captures *which behaviors within a tool form a
causal sequence*.

Understandings already form a DAG — every understanding has
`depends_on: &[UnderstandingDependency]`, used by depgen for install ordering
and by transitive prerequisite derivation. But inside each understanding,
behaviors are a flat `&[Behavior]` with no edges. The upsert pattern tried to
impose internal structure via `UpsertPhase` annotations, but annotations
don't create edges.

`to_blocks()` converts each behavior into an isolated Block with no edges.
The causal relationships between behaviors (Check must run before Create,
Create's output feeds Resolve) are not representable.

This is the structural problem. Everything else — string-typed semantics,
divergent vocabulary, missing resolve phases — is a symptom of modeling
behaviors as a flat list instead of a causal graph.

---

## 2. What Worked

### Unified DAG (`gunbai-dag`)

The DG-DAG migration consolidated 6 independent graph representations into one
parent/child model with consistent vocabulary (`parents_of`, `children_of`).
Before unification, the codebase used `dependencies`/`dependents`,
`requires`/`depends_on`, and `producer`/`consumer` interchangeably. The unified
model eliminated an entire class of confusion.

**Reference**: `docs/design/dag-unification.md`, `crates/gunbai-dag/`

### Understanding as Structured Data

Modeling tools and APIs as declarative `Understanding` structs
(`crates/gunbai-integrations-contracts/src/understanding/mod.rs`) rather than
prose documentation gave generators something to consume. The `Understanding`
type with `behaviors`, `depends_on`, `constraints`, and `assumptions` fields
captures enough structure that install scripts, CI checks, and documentation can
all be derived from the same source.

### Upsert Pattern — Typed Phases with Validation

The upsert pattern (`behaviors/upsert.rs`) is the system's clearest success.
Three phases (`Check`, `Create`, `Resolve`) with compile-time semantics
(`UpsertSemantics`) and runtime validation (`validate_upsert_consistency`)
ensure that every tool conforming to the pattern has the right properties:

- Check/Resolve require `ReadOnly`
- Create requires `Idempotent` or `IdempotentWithKey`
- `test_all_understandings_have_valid_upsert_phases()` catches violations at test time

This worked because the pattern was designed first, then tools conformed to it.

**Reference**: `crates/gunbai-integrations-contracts/src/understanding/behaviors/upsert.rs`

### Set Theory Rigor (`SetSpec<T>`)

`SetSpec<T>` (`understanding/set_theory.rs`) eliminated the ambiguity where
`&[]` could mean either "all behaviors" (Universal) or "no scopes" (Empty).
The enum makes intent explicit:

```rust
enum SetSpec<'a, T> {
    Empty,           // ∅
    Universal,       // B
    These(&'a [T]),  // {a, b, c}
}
```

The ST1–ST7 migration touched 42+ files. After migration, `is_member`,
`is_subset_of`, and `intersects` operations work correctly without guessing
the caller's intent.

**Reference**: `docs/design/set-theory-rigor.md`, `understanding/set_theory.rs`

### Property Migration (Requires → Structured References)

The RQ1–RQ6 migration replaced `Property::Requires(&'static str)` (freeform
strings like `"network access"`, `"sudo or root"`) with structured
`Requirement` values that reference real understandings (`infra/network`,
`infra/privileges`, `tool/curl`). This made dependency graphs machine-readable
and enabled automated validation.

**Reference**: `docs/design/requires-to-understandings.md`

### Producer-Centric Generators

Generators own their output. `gunbai-depgen` generates `deps.toml` and install
scripts; `gunbai-invariantsgen` generates `INVARIANTS.md` and `AGENTS.md`.
This ownership model means there's exactly one place to look when generated
output is wrong.

---

## 3. What Went Wrong

### Bottom-Up Vocabulary

19 tool understanding files (`understanding/tools/*.rs`) were written before a
shared behavioral vocabulary existed. Each tool author independently chose
behavior names, property combinations, and structural conventions. The result:
14+ tool files with subtly different patterns for the same operations (verify,
install, resolve).

This divergence is visible in the variety of behavior IDs across tools — some
use `"check"`, others `"verify"`, others `"detect"`. The naming was ad-hoc
because no shared vocabulary existed to conform to.

### One Formalized Pattern

Only upsert got the full treatment: typed phases, phase-specific property
requirements, a validation function, generated contract tests (BP9), and
documentation. Every other behavioral pattern — lifecycle, retry, failure
semantics, CRUD, watch, lock, cache — either stayed as prose or was
speculatively defined and deleted.

BL1 exists precisely because this gap was recognized: the system has one
real pattern and ~18 tools that partially conform to it.

### Speculative Abstractions

`docs/design/behavior-patterns.md` documents that transaction, retry, CRUD,
watch, lock, and cache patterns were defined speculatively and then deleted
because no concrete use case drove them. The design doc now explicitly states:
re-add only when a concrete need arises.

This was wasted work that also distorted the architecture — types were added
to support patterns that never materialized, then removed in cleanup passes.

### String-Typed Semantics

Several semantic fields remain freeform strings with no validation or codegen
leverage:

| Field | Example | Problem |
|-------|---------|---------|
| `FailsWhen(&'static str)` | `"resource does not exist"` | No structured failure taxonomy; can't generate error handlers |
| `EdgeCase(&'static str)` | freeform | No categorization; can't aggregate or validate |
| `OutputBehavior(&'static str)` | `"success:http_304"`, `"coerce_success:..."` | Ad-hoc convention-based prefixes, not typed |
| `Behavior.id: &'static str` | `"check"`, `"verify"` | No enum; naming inconsistency across tools |
| `UnderstandingDependency.target` | `"tool:git"`, `"secret:TOKEN"` | Convention-based prefixes instead of typed variants |

These strings compile fine regardless of content. Typos, inconsistent naming,
and semantic drift are invisible until runtime (or never caught at all).

### Missing Resolve Phase

The upsert contract requires Check → Create → Resolve, but many tool
understandings define Check and Create behaviors without a corresponding
Resolve phase. This violates the upsert contract — after ensuring a resource
exists, there's no standardized way to obtain a reference to it.

The `validate_upsert_consistency` function catches missing properties on
existing phases but doesn't enforce that all three phases are present.
V2 makes partial upsert unrepresentable by construction (see
[`v2-contracts-design.md`](v2-contracts-design.md) §7).

Note: Resolve should return a `ResolvedHandle`. For tools that don't produce
a meaningful handle (e.g., install-only tools where "exists" is the only
result), a unit/confirmation handle is valid — the contract requires the
*shape*, not a rich value.

### Confidence Without Verification

The CR1–CR4 migration removed `Confidence` annotations from 183 uses across
17 files because property claims weren't backed by tests. The fix was to
replace confidence ratings with generated tests (`ValidatesWith`), but this
exposed that the original property claims were aspirational rather than verified.
V2 replaces confidence annotations with `PropertyClaim { property, verified_by }`
— verification is structural, not cultural (see
[`v2-contracts-design.md`](v2-contracts-design.md) §P4).

---

## 4. Complete Audit of String-Typed Semantic Channels

Seven distinct channels carry semantic meaning as `&'static str`:

| Channel | Occurrences | Files | Consumer impact |
|---|---|---|---|
| `FailsWhen(&str)` | 107 | 32 | Mock predicate generation — generators must pattern-match free text to produce error handlers |
| `OutputBehavior(&str)` | 16 | 6 | String prefix protocol (`success:`, `coerce_success:`) parsed by convention in generators |
| `ValidatesWith(&str)` | 38 | 19 | References behavior IDs by string — broken if behavior is renamed |
| `EdgeCase(&str)` | 133 | 35 | Pure documentation — no migration needed, lowest priority |
| `Behavior.id: &str` | every behavior | ~40 | Block ID construction (`"{understanding.id}/{behavior.id}"`), registry lookups |
| `Understanding.id: &str` | every understanding | ~40 | Registry key, dependency resolution, cross-references |
| `dependency target: &str` | ~60 | ~20 | Parsed by `parse_dependency_target` into kind/name pairs |

**Key observation**: `EdgeCase` is the only channel that is genuinely
documentation. The other six carry semantic load that generators, validators,
or the registry depend on at runtime. Typos in any of the six are silent bugs.

**Risk ranking** (severity of silent bugs, not just occurrence count):

| Channel | Risk | Rationale |
|---------|------|-----------|
| `ValidatesWith(&str)` | **Critical** | Broken if behavior renamed; silent test gap |
| `dependency target: &str` | **High** | Parsed at runtime; typo = missing dependency |
| `Behavior.id: &str` | **High** | Used in block IDs, registry lookups |
| `FailsWhen(&str)` | **Medium** | Generators pattern-match; wrong error handlers |
| `OutputBehavior(&str)` | **Medium** | Prefix protocol; convention drift |
| `Understanding.id: &str` | **Low** | Rarely changes; validated by registry |
| `EdgeCase(&str)` | **None** | Pure documentation |

### V2 replacement mapping

Each string channel maps to a specific V2 typed replacement:

| V1 string channel | V2 replacement | Lane | Notes |
|---|---|---|---|
| `FailsWhen(&str)` | `FailureKind` enum + `CustomFailureCode` | A/B | Multiple failures per behavior allowed via `&[FailureKind]` |
| `OutputBehavior(&str)` | `OutputSemantics` enum + sub-enums | A/B | Eliminates prefix protocol entirely |
| `ValidatesWith(&str)` | **Removed** — structural edges or `BehaviorRef::from_role()` | A | Patterned behaviors reference roles, not string IDs |
| `Behavior.id: &str` | Derived `BehaviorId` for patterned; `CustomBehaviorId` via macro | A/B | Display string stability guaranteed by `Display` impl |
| `dependency target: &str` | `DependencyTarget` enum with typed IDs | A/B | No runtime parsing |
| `Understanding.id: &str` | `UnderstandingId` newtype + `understanding_id!()` macro | A | Const validation choke point |
| `EdgeCase(&str)` | Keep as `EdgeCase(String)` — doc-only | C | Explicitly declared Lane C |

This table is the contract between BL1 and V2: it specifies exactly what
disappears and what replaces it.

---

## 5. Blast Radius

### Dependency fan-out

- **18 crates** depend on `gunbai-integrations-contracts`
- **37 files** across those crates consume understanding types directly
- Changes to core types (`Behavior`, `Property`, `Understanding`) propagate to
  every consumer

### Critical path

The following systems break first when contracts change:

1. **depgen** — reads `Understanding.depends_on` to generate `deps.toml` and
   install scripts
2. **understandingdocs** — reads every field to generate documentation
3. **behavior contract tests** (BP9) — validates property/phase consistency
4. **ci-codegen** — generates CI checks from invariant definitions

### Key consumer patterns

- **String concatenation**: `"{understanding.id}/{behavior.id}"` is used to
  construct block IDs, test names, and documentation anchors.
- **Property pattern matching**: generators `match` on `Property` variants to
  extract semantics. Adding or renaming variants requires updating every match
  arm.

---

## 6. Revised Problem Statement

BL1 as originally scoped addresses vocabulary unification — giving tools a
shared behavioral vocabulary so `"check"` and `"verify"` converge. That's
necessary but insufficient. The deeper problem is structural:

**The understanding system currently allows modeling the world however you want,
then optionally tagging it with epistemological constructs.**

Upsert phases (`UpsertPhase`), set theory (`SetSpec<T>`), typed semantics —
these are all opt-in annotations on a fundamentally stringly-typed data model.
Nothing prevents an understanding author from ignoring upsert, using raw string
IDs, and shipping a behavior that "works" but participates in none of the
system's guarantees.

The epistemology must be **structural, not annotative**. The type system must
make it impossible to construct a behavior that doesn't declare its role in the
system's semantic model. An understanding that doesn't participate in upsert
should be explicitly excluded, not silently missing a field.

**V1 modeled tool facts; V2 models tool obligations.**

### Prior art (reference, not anchor)

The predecessor Go codebase (`OaaS_v2/pkg/dag`, documented in
`dag-systems-overview.md`) is useful as a reference but V2 should be designed
from first principles — not as a port of any prior system.

Key observations worth carrying forward:

- **Binding time matters.** Contracts bound at construction time catch
  violations earlier than contracts bound at validation time or runtime.

- **The Go system had no modeling layer.** There were no "understandings."
  There was real Go code (`func() error`) with a DAG overlaid on it —
  `NodeContract` declared what each function provides/requires/exports/imports,
  and `CompileAndValidate` enforced constraints before execution. The DAG
  constrained *real code*, not a *model of external systems*.

- **Understandings are genuinely new.** When the Rust system introduced
  understandings, it created something the Go system never had: a separate
  modeling layer that describes external tools *without executing them*.
  This is valuable — you can generate docs, install scripts, and tests from
  a model without running the tool. But because the Go system had no modeling
  layer to draw from, the Rust modeling layer was built as flat documentation
  (behaviors with string properties) rather than as a causal contract graph.

- **The Go DAG worked because nodes were executable.** The contract
  constrained actual `func() error` code. The Rust understanding system
  tried to get the same contract benefits from a model, but built the model
  as descriptions rather than as a DAG. The result: the execution layer
  (GraphIR) is causal, but the modeling layer (understandings) is flat.

---

## 7. What to Keep vs What to Restart

### Keep (stable, working, not part of the problem)

- **`gunbai-dag`** — unified DAG with consistent vocabulary; no string semantics
- **`SetSpec<T>`** — already structural, already enforces intent
- **Producer-centric generators** — ownership model is correct
- **Invariant system** — structural enforcement of repo rules
- **Progress visualization** — terminal UI, no contracts dependency
- **Store contract** — data persistence layer
- **DSL planning** — future work, not yet coupled
- **LaTeX codegen (`tectonic`)** — document generation pipeline
- **LLM workflow** — orchestration layer
- **All completed migrations**: RQ1–RQ6, CR1–CR4, NF1–NF11, ST1–ST7

### Restart scope (the contracts crate data model)

- **Core types**: `Behavior`, `Property`, `UnderstandingDependency`,
  `Requirement` — these carry the string-typed semantic channels
- **Pattern membership**: `upsert_phase: Option<UpsertPhase>` allows silent
  non-participation
- **String semantic channels** → must become typed (see §4 mapping table)

---

## Migration Code Glossary

| Code | Migration |
|------|-----------|
| DG-DAG | DAG unification (6 graph representations → `gunbai-dag`) |
| ST1–ST7 | Set theory rigor (`SetSpec<T>` introduction, 42+ files) |
| RQ1–RQ6 | Requires → structured understanding references |
| CR1–CR4 | Confidence removal (183 uses across 17 files) |
| NF1–NF11 | Naming/formatting normalization |
| BP1, BP9 | Behavior pattern formalization + generated contract tests |

---

## References

| Topic | Location |
|-------|----------|
| Core types | `crates/gunbai-integrations-contracts/src/understanding/mod.rs` |
| Upsert pattern | `crates/gunbai-integrations-contracts/src/understanding/behaviors/upsert.rs` |
| Set theory | `crates/gunbai-integrations-contracts/src/understanding/set_theory.rs` |
| Tool understandings | `crates/gunbai-integrations-contracts/src/understanding/tools/*.rs` |
| DAG unification | `docs/design/dag-unification.md` |
| Behavior patterns | `docs/design/behavior-patterns.md` |
| Set theory design | `docs/design/set-theory-rigor.md` |
| Requires migration | `docs/design/requires-to-understandings.md` |
| Completed work | `TODONE.md` (BP1, ST1–ST7, RQ1–RQ6, CR1–CR4, DG-DAG, BP9) |
| BL1 task | `TODO.md` |
| Go-era DAG system | [`dag-systems-overview.md`](dag-systems-overview.md) |
| **V2 Design** | **[`v2-contracts-design.md`](v2-contracts-design.md)** |
| **V3 Minimal** | **[`v3-contracts-minimal.md`](v3-contracts-minimal.md)** |
| Abstraction Calculus | Internal document — foundational inspiration |
