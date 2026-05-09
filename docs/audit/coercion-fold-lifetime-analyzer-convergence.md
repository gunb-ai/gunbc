# Coercion-Fold x Lifetime-Analyzer Convergence

**Date:** 2026-05-01  
**Lane:** T-Ground-Coercion-Fold / T-Ground-Lifetime-Analyzer  
**Scope:** audit / integration spec only. No code changes.

## Summary

`docs/design-emission-model.md` Example 4 requires Coercion-Fold to select a
Rust string-family inhabitant from program structure: a transient function
parameter has `ownership = Borrowed`, `lifetime = caller`, and therefore matches
`&str`.

The integration is currently stub-shaped. `fold_program_to_target` already
accepts a `LifetimeAnalysisReport`, but the `ScratchIntExamples` arm asserts the
report is empty and ignores it. The lifetime analyzer can derive the R2
ownership / lifetime / growability facts from `LifetimeProgram` fixtures, but
`Dag` -> `LifetimeProgram` extraction still returns an empty program for
bootstrap rows and fails closed for runtime `data` / `fn` surface.

Post-Slice-C, the declared LanguageSpec projection should consume lifetime facts
per binding and filter string-family inhabitance candidates. Grounding consumes
the substrate rows; it should not author string-family substrate facts locally.

## Lifetime-Analyzer Current State

The public report is:

```rust
pub type LifetimeAnalysisReport = BTreeMap<BindingId, LifetimeFacts>;

pub struct LifetimeFacts {
    pub ownership: Ownership,
    pub lifetime: LifetimeScope,
    pub growable: Growability,
    pub encoding: Encoding,
}
```

Axis vocabulary emitted today:

| Axis | Current variants | Status |
|---|---|---|
| `Ownership` | `Owned`, `Borrowed` | R2 program-intent outcomes. `Conditional` remains target-side / R3. |
| `LifetimeScope` | `Self_`, `Caller` | R2 program lifetime roles. `Self_` corresponds to self-contained/top-level or return-owned values; `Caller` corresponds to function-parameter call-frame scope. |
| `Growability` | `Yes`, `No`, `NotApplicable` | Derived from growth-use witnesses when the target axis is load-bearing. |
| `Encoding` | `Utf8FreeMonoidChar` | Single-arm stub for `.dag` `String` / `FreeMonoid<Char>` UTF-8 until LanguageSpec declares the full vocabulary. |

The analyzer’s structural input is a `LifetimeProgram`:

- `BindingId` is the report key.
- `BindingDef` carries a name, `BindingRole`, use-site list, and
  `ProgramTypeFamily`.
- `BindingRole` models the R2 cases: `TopLevelData`, `FunctionParameter`, and
  `FunctionReturn`.
- `UseKind` models the current R2 use witnesses:
  `Transient`, `StoreOrEscape`, `GrowthMutation`, plus test-only fail-closed
  witnesses `BorrowExclusive` and `IndeterminateGrowability`.
- `R3Construct::{Closure, Async, Pin}` is explicit and maps to
  `EmissionDiagnostic::OutOfR2Scope`.

What is structurally derived today:

| Program shape | Current analyzer result |
|---|---|
| Example 3 top-level `String` data with no growth / escape | `Owned`, `Self_`, `No`, `Utf8FreeMonoidChar` |
| Example 4 Case A function parameter with transient uses only | `Borrowed`, `Caller`, `NotApplicable`, `Utf8FreeMonoidChar` |
| Example 4 Case B function parameter stored / escaped | `Owned`, `Caller`, growability from use witnesses |
| Function return value | `Owned`, `Self_`, `NotApplicable`, `Utf8FreeMonoidChar` |
| Contradictory borrow + escape | `ContradictoryUse` |
| Indeterminate load-bearing growability | `UnderRefined { axis: "growability" }` |
| Unclassified binding type family | `UnderRefined { axis: "encoding" }` |
| Closure / async / Pin markers | `OutOfR2Scope { construct }` |

What is mocked / not structurally derived from `Dag` yet:

- Worked Examples 3 and 4 are represented by explicit `LifetimeProgram`
  constructors in unit tests.
- `extract_lifetime_program(&Dag)` returns `Ok(LifetimeProgram::empty())` for
  bootstrap-only DAGs.
- Runtime-appended `data` or `fn` declarations fail closed with
  `LifetimeProgramExtractionPending`; extraction does not yet walk real binding
  definitions, function bodies, store/escape sites, or call sites.
- `LanguageSpecAxes` is a small test-facing input with only
  `string_growability_axis_load_bearing`; it is not yet extracted from declared
  LanguageSpec rows.

## Example 4 Fold Steps

Design-doc Example 4:

```text
data name: String = "Alice"
fn greet(n: String) -> Unit { ... }
greet(name)
```

Case A is the convergence proof: `greet` uses `n` transiently and does not store
or return it, so the Rust parameter realization should be `&str`.

| Fold step | Coercion-Fold responsibility | Lifetime fact consumed | Substrate row matched | Current gap |
|---|---|---|---|---|
| Read program source type / algebra for `n` | Resolve the binding's declared `.dag` type to `String` / `FreeMonoid<Char>` candidate family. | `encoding = Utf8FreeMonoidChar` is available from analyzer fixture type family. | Future string-family `inhabits` rows keyed by `FreeMonoid<Char>`. | `Dag` extraction does not yet classify runtime function parameter types into `ProgramTypeFamily`; target string-family rows are missing. |
| Determine ownership from function-body use | Ask lifetime analyzer for binding `n` and read `ownership`. | `Ownership::Borrowed` for transient-only parameter use. | Candidate row with `ownership = Borrowed`, e.g. Rust `&str`. | Requires stable `BindingId` correspondence between lowered program binding and fold binding. |
| Determine lifetime scope | Read lifetime scope for binding `n`. | `LifetimeScope::Caller`. | Candidate row with caller/source borrowed lifetime, e.g. Rust `&str`. | Target-side lifetime vocabulary must align with analyzer vocabulary. |
| Determine growability requirement | Read growability for binding `n`. | `Growability::NotApplicable` when ownership is borrowed. | Candidate row whose growability is not applicable / conditional-compatible for borrowed string. | Target string rows must declare how `NotApplicable` matches borrowed candidates; no local fold convention should invent this. |
| Select unique Rust realization | Filter declared projection candidates by algebra + ownership + lifetime + growability. | Full `(Borrowed, Caller, NotApplicable, Utf8FreeMonoidChar)` tuple. | Unique Rust `StrSlice` / `&str` row. | Per-target string-family rows (`String`, `Box<str>`, `&str`, `Cow<str>`) are not authored at HEAD. |
| Emit call-site borrow | After parameter target is selected as borrowed, emit call-site conversion `greet(&name)`. | The callee parameter row demands borrowed input for the call duration. | Target call/borrow rendering row or method-template contract, depending on final emit wiring. | This is downstream of Coercion-Fold selection; not part of Slice C body itself. |

Case B (`StoreOrEscape`) should select an owned realization instead of `&str`.
The analyzer already returns `Ownership::Owned` for that fixture; the fold still
needs declared target rows to decide between owned growable (`String`) and owned
non-growable (`Box<str>`) according to the growability fact.

## R2-Scope-Locked vs R3 Cases

R2 scope is explicit in `v3_grounding_lifetime`:

| Case | Current support | Boundary |
|---|---|---|
| Top-level data bindings (Example 3) | Supported in `LifetimeProgram` fixture and analyzer body. | `Dag` extraction pending. |
| Function parameters with transient vs escaping use (Example 4) | Supported in `LifetimeProgram` fixtures and analyzer body. | `Dag` extraction pending. |
| Function return values | Supported in analyzer body: always `Owned`, `Self_`, `NotApplicable`. | Return-position extraction pending. |
| Closures | Explicit `R3Construct::Closure` -> `OutOfR2Scope`. | Structural at analyzer input once extraction emits the marker. |
| Async lifetimes | Explicit `R3Construct::Async` -> `OutOfR2Scope`. | Structural at analyzer input once extraction emits the marker. |
| Pin / self-referential patterns | Explicit `R3Construct::Pin` -> `OutOfR2Scope`. | Structural at analyzer input once extraction emits the marker. |

The R3 boundary is structural inside `LifetimeProgram` because unsupported
constructs are represented by `R3Construct` and fail closed. It is still
implicit at the `Dag` boundary because extraction does not yet classify runtime
closures, async, or Pin/self-referential surfaces; those should become explicit
markers rather than ad hoc string diagnostics when lowering exposes them.

## Integration Data Flow

Post-Slice-C data flow should be:

1. `fold_program_to_target(dag, lifetime_facts, language_spec)` receives a
   declared LanguageSpec projection instead of `ScratchIntExamples`.
2. The caller either passes a precomputed `LifetimeAnalysisReport`, or
   Coercion-Fold calls the public analyzer entry:

   ```rust
   let axes = language_spec.axes_for_string_family();
   let lifetime_facts = analyze_lifetime_facts(dag, &axes)?;
   ```

3. The fold walks program binding rows in the `Dag` and resolves each binding to
   its source type / algebra family.
4. For each string-family binding, the fold looks up `LifetimeFacts` by the same
   `BindingId` produced by lowering / lifetime extraction.
5. The declared projection exposes string candidates:

   ```rust
   candidates_for_type_and_algebra(std_string, free_monoid_char)
       .filter(|row| row.ownership == facts.ownership)
       .filter(|row| row.lifetime == facts.lifetime)
       .filter(|row| growability_matches(facts.growable, row.growability))
       .filter(|row| row.encoding == facts.encoding)
   ```

6. Zero candidates returns `NoInhabitant`; multiple candidates return
   `UnderRefined` on the distinguishing axis; one candidate returns a selected
   target inhabitance row reference.

The current `debug_assert!(lifetime_facts.is_empty())` should disappear only
when a declared projection arm consumes report entries. It should remain true
for any surviving `ScratchIntExamples` path, because those examples do not
correspond to real program bindings.

## Substrate-Gap Surfacing

Known missing substrate facts:

- Per-target string-family inhabitance rows are missing from
  `src/v3/spec/{rust,python,go}.dag`. At HEAD those files carry broad rows such
  as `rust_string`, `python_string`, and `go_string`, but not the design-doc
  family rows for Rust `String`, `Box<str>`, `&str`, `Cow<str>` with ownership /
  lifetime / growability axes.
- `dsl/std/string_type.dag` currently has a single `String` row rather than the
  full string-family substrate needed by Examples 3 and 4.
- `LanguageSpecAxes` is not yet projected from target rows; it is a small
  analyzer input used by tests.
- `Dag` -> `LifetimeProgram` extraction cannot yet read the runtime binding/use
  graph that would produce Example 3/4 reports from actual source.

Axis alignment check:

| Analyzer axis/value | Design-doc target-row vocabulary | Alignment decision needed |
|---|---|---|
| `Ownership::Owned` | `ownership = Owned` / `owned = yes` | Same concept. Substrate rows should use a canonical enum-like value, not per-target spellings. |
| `Ownership::Borrowed` | `ownership = Borrowed` / `owned = no` | Same concept. Prefer `Borrowed` as the row value to avoid boolean loss around `Conditional`. |
| `LifetimeScope::Self_` | `lifetime = self`, `self-contained` | Same concept for owned/self-contained rows. Rows should normalize on one substrate value; Rust identifier escaping (`Self_`) should remain Rust-only. |
| `LifetimeScope::Caller` | `lifetime = source` / caller-bounded borrowed lifetime | Same conceptual role for Example 4. Substrate should choose one name and document whether `source` and `caller` are synonyms or distinct lifetimes. |
| `Growability::Yes` | `growable = yes` | Same concept. |
| `Growability::No` | `growable = no` | Same concept. |
| `Growability::NotApplicable` | borrowed candidate / n/a | Needs explicit row vocabulary. Do not encode this as absent data; absence would blur "not applicable" with "unknown". |
| `Encoding::Utf8FreeMonoidChar` | `FreeMonoid<Char>` / UTF-8 `str` semantics | Same concept. Encoding should likely remain implied by algebra for `FreeMonoid<Char>`, matching design-doc correction. |

No additional Substrate-owned program appears beyond the already known string
family rows plus extraction/projection details, but the string-family row slice
must include canonical axis vocabulary. If Substrate lands rows with lowercase
strings (`borrowed`, `self`) while the analyzer exposes Rust enum values
(`Borrowed`, `Self_`) and no shared declaration identity, Grounding should
STOP+PING `#1130` rather than adding a local string normalization table.

## Convergence Test Shape

Post-integration tests should mirror the Slice C table discipline: a minimal
`LifetimeAnalysisReport` fixture plus a minimal declared LanguageSpec projection
fixture, with assertions on selected row identity.

| Design example | Fixture shape | Expected assertion |
|---|---|---|
| Example 3 top-level `String` data | Program binding `name` has `LifetimeFacts { ownership: Owned, lifetime: Self_, growable: No, encoding: Utf8FreeMonoidChar }`. Rust projection has `String`, `Box<str>`, `&str`, `Cow<str>` string-family rows. | Fold selects the `Box<str>` inhabitance row under the strict design-doc reading: owned + non-growable + self-contained. |
| Example 3 growable top-level data | Same as Example 3, but use sites include `GrowthMutation`, producing `Growability::Yes`. | Fold selects Rust `String` row. This test proves growability is consumed, not ignored. |
| Example 4 Case A transient parameter | Binding `n` has `LifetimeFacts { ownership: Borrowed, lifetime: Caller, growable: NotApplicable, encoding: Utf8FreeMonoidChar }`; projection includes the Rust borrowed string row. | Fold selects the `&str` / `StrSlice` inhabitance row and records that the call-site must borrow. |
| Example 4 Case B stored / escaped parameter | Binding `n` has `Ownership::Owned`, `LifetimeScope::Caller`, growability from use witnesses; projection includes owned rows. | Fold rejects `&str` and selects an owned row or fails under-refined if growability cannot distinguish `String` vs `Box<str>`. |
| Function return value | Binding `ret` has `Ownership::Owned`, `LifetimeScope::Self_`, `Growability::NotApplicable`. | Fold selects an owned self-contained row; borrowed rows must not match return values. |
| R3 closure / async / Pin | Analyzer fixture returns `OutOfR2Scope`. | Coercion-Fold propagates the diagnostic and does not attempt candidate search. |

The fixture should not use `TargetInhabitance` enum variants. It should assert
declaration identity for the selected inhabitance row and realization row, as in
the Slice C prep audit.

## Substrate-Prerequisite Checklist

| Prerequisite | Required shape | Owner / boundary |
|---|---|---|
| String-family inhabitance rows | Per-target rows for `FreeMonoid<Char>` candidates with ownership, lifetime, growability, and realization refs. Rust needs at least `String`, `Box<str>`, `&str`, and `Cow<str>` or an explicit deferral for conditional candidates. | Substrate Manager / LanguageSpec population (`#1130`) |
| Canonical axis vocabulary | Shared substrate values for ownership, lifetime, growability, and not-applicable/conditional cases; no local Grounding string normalization. | Substrate Manager (`#1130`) |
| LanguageSpec axis projection | Grounding-readable projection of whether string growability is load-bearing for a target. | Substrate rows first; Grounding reader after rows land |
| Program binding/use extraction | `Dag` lowering exposes binding ids, roles, type family, transient/store/escape/growth use sites, and R3 markers. | Substrate / compiler-lowering owner; Grounding consumes |
| Binding identity contract | The `BindingId` key in `LifetimeAnalysisReport` must be the same key Coercion-Fold uses when selecting per-binding target inhabitance. | Grounding + lowering integration |
| Diagnostic propagation | `OutOfR2Scope`, `UnderRefined`, `ContradictoryUse`, and `LifetimeProgramExtractionPending` stay typed and are mapped to emission diagnostics without collapsing to compiler-internal strings. | Grounding diagnostic consumer |

If the string-family substrate row slice lands without canonical axis values, or
if lowering exposes bindings without a stable report key shared by the lifetime
analyzer and Coercion-Fold, Grounding should pause and route a P1 cross-program
signal to `#1130`. Those would block honest Example 4 integration.
