> Part of: [post-l15-phase-plan.md](./post-l15-phase-plan.md) > [single-emitter-design.md](./single-emitter-design.md)
> Extends: [DB-2 generic walker API](./design-generic-walker-api.md), [DB-4 clean emission contract](./design-clean-emission-contract.md), [DB-8 fixed-point ratchet](./design-fixed-point-ratchet.md)
> Gates: Lane 1 Stage 1e (execution)

# Lane 1 Stage 1d — Single-emitter consolidation build plan

**Lane:** 1 (Emission unification)
**Stage:** 1d (last design stage; gates Stage 1e implementation start)
**Size:** M
**Status:** R2. Pilot-audited design; Stage 1e scaffold started on the shared `emit.rs` path.

> Role in the plan: Stage 1d produced the file-by-file build plan for
> Stage 1e's consolidation execution. This R2 revision records what the
> Rust/Go/Python pilots actually proved and narrows the first Stage 1e
> landing to an honest scaffold: one shared `emit.rs` entrypoint with Go
> migrated behind it, while Rust and Python remain on legacy emitters.

---

## Motivation

`docs/single-emitter-design.md` establishes the principle: one emitter,
reads target specs. But it stops at principles. After the
Rust/Go/Python pilots, Stage 1d had to answer a narrower question:
which pieces of the pre-pilot design survived contact with three
targets, and which had to be reshaped before code could move?

The post-pilot audit says:

- **Survives:** clean-emission dispatch is a target-spec fact; typed
  `language` filtering is the authority for realizations; template
  substitution remains the rendering primitive; unsupported core
  behaviors fail closed rather than collapsing semantically.
- **Reshaped:** Python proved `PatternBindingRule` is only half of the
  payload-binding story; `variant_payload_field_access` is equally
  load-bearing. The first shared entrypoint can land before the full
  recursive walker is shared; an adapter stage is acceptable if it
  moves one target onto the common path honestly. Python's target-
  private realization family is real Stage 1e debt, not something the
  docs can wish away.

With that evidence in hand, consolidation needed a **file-by-file build
plan** with:

- Which functions in `emit_rust.rs` / `emit.rs` (Go path) /
  `emit_python.rs`
  become target-agnostic (move into generic walker) vs target-declared
  (move into spec)
- What new spec fields each current hardcoded behavior needs
- Where substrate gaps block the consolidation
- Which target to pilot with (P2-L1 choice)
- A bridge list: every piece of name-based dispatch, hardcoded variant
  name, or target-specific convention currently in Rust code

Stage 1d is no longer speculative. Its deliverable is this audited
design set plus the first Stage 1e scaffold that proves the common
entrypoint can own one target without pretending the full walker has
already dissolved the rest.

---

## Scope

Four deliverables, all design artifacts:

### 1. Emitter function inventory

Output: `docs/emit-functions-inventory.md`

For every `fn render_*` and `fn emit_*` across the three current
emitters:

| Function | Current home | Classification | Destination |
|---|---|---|---|
| `render_operator` | emit_rust.rs | spec-driven | generic walker + spec template |
| `render_field_project` | emit_rust.rs | spec-driven | generic walker + spec template |
| `InputUseFacts::build` | emit_rust.rs | lens | ownership lens (own file) |
| `decl_is_copy` | emit_rust.rs | lens | copy-type lens (own file) |
| `algebra_field_for_operator` | emit_rust.rs | substrate walk | shared primitive |
| ... (30+ more) | | | |

**Classifications:**
- **spec-driven** → template substitution, walker reads spec, dissolves
- **lens** → extract into its own `lens_*.dag`, walker consumes facts
- **substrate walk** → generic primitive, moves to `std/substrate_walks.dag`
- **per-target integration** → stays target-specific (rustfmt invocation,
  file extension choice, etc.) — typically <5% of total

**Expected split:** ~90% spec-driven or lens, ~5% substrate walk, ~5%
per-target integration.

### 2. Spec field gap list

Output: `docs/spec-field-gaps.md`

For every spec-driven function, enumerate what the target spec currently
declares vs what the function needs. Example:

```
render_branch (match expression emission):
  Current rust.dag: match_arm template, match_expr template
  Needed: match_discriminator rule (does the target dispatch on
          enum tag? struct field? runtime type check?)
  Gap: new `data rust_match: MatchStrategy = DiscriminatorTag` needed

render_transform (callable dispatch):
  Current rust.dag: CallableRealization with template
  Needed: nothing — already sufficient
  Gap: none
```

The gap list drives what spec extensions P2 adds BEFORE the generic
walker can replace the Rust function.

### 3. Bridge inventory

Output: `docs/emit-bridges.md`

Every place in the current emitters where Rust code does something the
spec should do. Examples already known:

- `name.starts_with("rust_")` for target realization filtering (B11)
- `v.label == "Empty"` / `"Cons"` in Python pattern matching (B13)
- `algebra_field_for_operator` always resolves via OrderedRing (B14)
- `#[derive(...)]` injection in type emission (Rust-specific hardcode)
- Literal syntax variations (`"str"` vs `'str'` vs `"""str"""`)

Each bridge is scoped: what kills it, what spec field replaces it,
whether the substrate supports it today.

### 4. Consolidation proof target

Output: a section in this file.

**Corrected from an earlier revision**: an earlier version of this section proposed SPICE or English as the "consolidation pilot target." Per THESIS.md §1505 ("Two shapes of omni-emission"), SPICE netlists and natural-language documentation are **Shape B artifacts** — outputs of `.dag` PROGRAMS, not compiler targets. Using them as compiler emission pilots would be a category error. Corrected per PR #491 review.

The consolidation proof is either:

**Option A — Rust/Go/Python re-emission stability**
The consolidated walker re-emits existing Rust/Go/Python output with bit-identical results. No new target needed; the proof is that the consolidation didn't change observable behavior.

**Option B — Add one additional Shape A target** (another programming language)
Evaluate one additional programming language that could join Rust/Go/Python — Swift, Kotlin, TypeScript — as a consolidation smoking-gun ("adding a new target = one spec file"). Criteria:

| Criterion | Swift | Kotlin | TypeScript |
|---|---|---|---|
| Ownership complexity | Similar to Rust (ARC) | GC | GC |
| Pattern emission | Enums with associated values | Sealed classes | Discriminated unions |
| Verifier availability | `swiftc -parse` | `kotlinc` | `tsc --noEmit` |
| Substrate coverage | Full — standard spec shape fits | Full | Full |
| Test cost | Low | Low | Low |

**Recommendation framework, not decision:** pick the target exposing the fewest new substrate gaps. Swift's ARC maps cleanly to existing ParameterDisposition; Kotlin/TypeScript's GC maps cleanly to existing MemoryModel variants.

**Shape B is explicitly out of scope** for this lane (and all of Lane 1): per THESIS.md, SPICE / English / Verilog / YAML / Terraform / K8s manifests etc. are outputs of user programs, not compiler targets. A `.dag` program that emits SPICE is a user-space library, not a compiler feature.

This lane produces the Option A vs Option B evaluation; P2-L1 owner makes the final call informed by it.

### 5. Pessimistic fallbacks from Half B to revisit

Half B's merge reconciliation (2026-04-17) reverted two optimizations
back to pessimistic behavior because they misbehaved under merge with
main's cached-bootstrap state. These are **known-pessimistic areas**,
not silent regressions — explicitly catalog them here so the walker
design revisits them rather than inheriting the revert:

**A. `decl_is_copy` structural walk**
- **Reverted:** the structural walk over user-defined sum types was
  over-eagerly classifying variants with all-Copy payload as Copy.
  Reverted to main's "sum types are non-Copy" conservative behavior.
- **Revisit during consolidation:** structural copy-detection for
  user-defined sums is legitimately decidable. The walker can read
  each variant's payload fields and compute Copy-ness from the Conj
  structure. This should re-land during Lane 1e as part of the
  copy-type lens.
- **Path forward:** copy-type becomes a dedicated lens (it's one of
  the extractions identified in the function inventory). The lens
  does the structural walk correctly once; every consumer reads
  from it. This eliminates the local-reconstruction issue that
  caused the Half B revert.

**B. `OwnedConstructLastUse` optimization**
- **Reverted:** the optimization that emitted move-on-last-use
  (`value` instead of `value.clone()`) was unsound under template
  reordering. Reverted to always-clone behavior.
- **Revisit during consolidation:** this is the "template-aware
  ownership emission" problem. The consolidated walker has full
  template context (CleanEmissionContract + LocalScope with
  position tracking per DB-2). A template-aware move-or-clone
  decision is tractable once the walker is authority for emission
  order.
- **Path forward:** the ownership lens (extracted from
  `analyze_user_defined_callable`, per Lane 1e function inventory)
  computes borrow/move/clone decisions from substrate facts. The
  walker consumes those decisions without needing to re-derive
  ownership during emission — which is what made the Half B
  optimization unsound.

**C. Clone ratchet 1 → 5 (main was 6)**
- Current state: 5 clones in `lens_unused_parameters_generated.rs`.
  Main was at 6; Half B initially got to 1 but reverted to 5 due
  to A and B above.
- **Target after Lane 1e:** ≤ 1 (Half B's original target).
  Achievable once the ownership lens is the single authority and
  the walker reads from it.

**D. `Behavior::Loop` fail-closed in emit_go / emit_python**
- Half B landed the fix that `emit_go` and `emit_python` no longer
  silently render `Loop` as "the loop body's result port" (a
  semantic collapse). They now fail-closed on unsupported `Loop`
  emission.
- Side effect: three previously-passing-by-accident tests got
  `#[ignore]`d pending real Loop support:
  - `emit_go::tests::go_lens_unused_parameters_module`
  - `emit_python::tests::emit_python_module_marks_ownership_as_skipped_for_gc_target`
  - `emit_python::tests::emitted_python_lens_matches_emitted_rust_lens_on_reflected_programs`
- **Re-enables during Lane 1e**: once the consolidated walker
  dispatches `Loop` through each target's LanguageSpec, all three
  targets gain Loop emission simultaneously. The cleanup checklist
  for 1e includes:
  ```
  grep -rE '#\[ignore = "blocked on emit_.* Behavior::Loop support' src/v3/compiler/tests/
  # → must return zero hits at Stage 1e completion
  ```
- Also adds a Stage 1e acceptance bullet: all three Half-B-ignored
  tests pass.

These are **explicit revisit items** during Stage 1e execution. Flag in
the build plan's bridge inventory so the walker design accounts for
them.

### 6. Candidate invariants for Lane 1c INVARIANTS.md additions

Half B's principle audit suggested three structural invariants that
predate the next round of consolidation work. All three fit naturally
into Lane 1c's clean-emission-invariant work (E-5 framing) and should
be evaluated for INVARIANTS.md inclusion during that stage:

**E-6 — No target-spec field lands without a same-PR consumer.**
If `rust_foo: FooType` appears in `src/v3/spec/rust.dag`, the
emitter change that CONSUMES `rust_foo` lands in the same PR.
Prevents "declared-facts-not-wired" drift (the exact pattern that
landed B-NEW-2 and B-NEW-3 in Half B as follow-up commits).

**E-7 — No target-private realization schema lands without a
dissolution ratchet.**
If a target-specific type (e.g., `PythonPatternRealization`) is
added, the PR lists the path to dissolve it into a target-agnostic
substrate fact. Not every target-private type is pure debt (some
are genuinely per-target), but all should have an exit path or an
explicit "permanent per-target" receipt.

**E-8 — Unsupported core behaviors fail closed; never collapse
semantically.**
When an emitter encounters a core Behavior variant it doesn't
support (e.g., `emit_python` on `Loop` in GC-target mode), it
returns `EmitError::UnsupportedBehavior` — not a semantic
approximation (like "render the body's result port" as Half B's
pre-fix emit_go did for Loop). This is what Half B just enforced.

Lane 1c owns evaluating these for inclusion alongside E-5. If
adopted, they gain mechanical gates (same pattern as E-5's
post-emit verifier + L-7's grep check).

---

## 7. Walker contract

The walker's job is to turn a `Dag` + `TargetLanguageId` into bytes
that pass the target's `post_emit_verifier`. Every rendering
decision traces to a structural fact the target spec declares.
Nothing else. DB-2 locked the API surface; this section pins down
WHICH specs the walker reads and HOW they compose.

### What the walker reads (per target)

> **Authority-location note.** Per THESIS.md §"Targets are
> declarations" and §"Bootstrap staging note"
> (`THESIS.md:1066–1077`), the **canonical home** for Shape A
> language specs is `dsl/extdeps/languages/<target>/` — the
> conceptual location this design aims at. `src/v3/spec/<target>.dag`
> is a **bootstrap-loaded staging fixture** that v3's Rust
> compiler currently reads during the M1(2.5)–M1(3) window; the
> files at that path are an implementation detail of the
> bootstrap, not a second source of truth. Everywhere this plan
> refers to a target spec file, it means the current
> bootstrap-loaded fixture under `src/v3/spec/`, with the
> understanding that the fixture's content migrates to the
> canonical extdeps home as separate class-5 follow-up work
> tracked in THESIS.md. Stage 1e inherits whichever path the
> bootstrap loads at lift time; the walker's API shape is
> indifferent to the path because every read goes through typed
> accessors on `Dag`, not via hardcoded paths.

Each `TargetContext` bundles five structural authorities sourced
from the per-target spec file (currently
`src/v3/spec/<target>.dag` as bootstrap-loaded fixture —
Rust/Go/Python today). Rust and Go have all five authored and
parseable post-Stage-1c. **Python has three of the five authored
on the shared schema today: `python_clean_emission:
CleanEmissionContract` (which also carries `correction_style:
CorrectionStyle` as its embedded DB-4 sub-record),
`python_target: TargetExecutionModel`, and the DB-14-shared
accessor bindings. The two that remain on private `Python*`
scaffolds marked for dissolution in `spec/python.dag` are
`LanguageSpec` (no `python_language` root today, plus the eight
missing sub-spec records) and the shared-schema Realization
families (all five realization families use private
`Python*Realization` variants).** Migrating Python onto the
shared walker-authority surface is the explicit prerequisite
bridge sub-stage 1e.0 (see §10 Migration plan). The authority
table below describes the post-1e.0 surface all three targets
MUST present; 1e.1's `TargetContext::resolve` gate runs against
that surface:

| Authority | Source declaration | Walker uses for |
|---|---|---|
| **`LanguageSpec`** | `rust_language` / `go_language` / `python_language` (Python-scaffolded) | Token-level rendering: statement shapes, expression shapes, control-flow keywords, literal syntax, module separators, function signatures, type applications, type defs, pattern syntax, collection ops, value constructors. Surfaces the per-sub-spec records (`StatementSyntax`, `ExpressionSyntax`, `ControlFlowSyntax`, `LiteralSyntax`, `ModuleSyntax`, `FunctionSyntax`, `TypeApplicationSyntax`, `TypeDefinitionSyntax`, `PatternMatchSyntax`, `CollectionOps`, `ValueConstructionSyntax`). |
| **`CleanEmissionContract`** (DB-4) | `rust_clean_emission` / `go_clean_emission` / `python_clean_emission` | E-5 rule dispatch: `expression_wrapping`, `pattern_bindings`, `imports`, `block_return`, `variable_bindings`, `match_arm_body`, `correction_style`, `post_emit_verifier`. Governs ONLY constructive shape choices; no escape hatches. |
| **`TargetExecutionModel`** | `rust_execution_model` / `go_execution_model` / `python_target` | Memory model (`OwnershipBased` / `GarbageCollected`) and scope model (`LexicalScoping`) drive ownership-lens consumption and clone/borrow/move rendering. Read at lens-construction time, not per-node. |
| **`RealizationIndexes`** (5 families) | Filtered per target via typed `language: LanguageSpec` field on each `Realization` record | Per-declaration/operator/behavior/callable/pattern template lookups. DB-2's Realization coproduct: `TypeRealization` / `OperatorRealization` / `BehaviorRealization` / `CallableRealization` / `PatternRealization` / `TypeInstantiationRealization` / `SubstrateAccessorRealization`. Post-PR-2.5 these are keyed by typed `DeclarationRef`, not by name prefix. |
| **`CorrectionStyle`** (DB-1 via DB-4) | `rust_clean_emission.correction_style` (structurally nested) | Indent unit, line ending, string quote, trailing semicolon. Diagnostic fix generators (Stage 3b) read this alongside the walker; walker reads it when rendering indentation-sensitive output. |

A sixth authority is read but NOT per-target — it is substrate-
global and target-indexes its result:

| Authority | Source | Walker uses for |
|---|---|---|
| **`SubstrateAccessorBinding`** (DB-14) | `port_binding_<target>` / `node_binding_<target>` / `resolve_producer_binding_<target>` | DB-14 accessor call-site rendering. Filtered per target by the binding's `language` field. |

### Composition pipeline

```
emit(dag, target) := {
  ctx            := resolve_target(dag, target);
  // ctx reads the six authorities above into a single struct.
  text           := emit_dag(dag, &ctx);
  // emit_dag recurses over Dag.declarations + top_level_binds via
  // emit_behavior dispatch.
  _verified      := run_post_emit_verifier(&text, &ctx.clean_emission.post_emit_verifier);
  // PR #532 already implements this in post_emit_verifier.rs.
  EmittedSource { text, target }
}
```

Zero logic in `emit_dag`, `emit_behavior`, or any helper depends
on the target identity. Targets appear only through `ctx`. If any
recurse-site pattern-matches on `target` or calls
`name.starts_with("rust_")` / similar, that is a bug; the fact
belongs in a spec field.

### Per-construct rendering rules

The walker resolves every construct through one of three dispatch
shapes. Each shape is mechanical — "look up a record, substitute
its template's placeholders, recurse on inputs":

1. **Node-shape dispatch** — `emit_behavior` matches
   `Behavior::{Value, Transform, Branch, Loop, Bind}`. Each
   variant reads the corresponding `BehaviorRealization` (if the
   substrate exposes one — `Bind` / `Branch` / `Main` today) OR
   routes to a dedicated helper that reads the target's syntax
   sub-spec (e.g., `LanguageSpec.control_flow.if_else` for a
   Branch rendering operand-context).
2. **Type-shape dispatch** — `emit_type`, `emit_type_decl`,
   `emit_type_instantiation` look up `TypeRealization` /
   `TypeInstantiationRealization` by `DeclarationId` and
   substitute into the carrier template.
3. **Callable-shape dispatch** — `emit_transform` resolves the
   Transform's target declaration, reads
   `CallableRealization.strategy` (`ListFold`, `ListMap`, …) and
   `CallableRealization.parameters` (slot-keyed
   `ParameterDisposition`), then formats using the target's
   `CollectionOps` record.

Each rule is independent: a node's rendering does not depend on
the target's rendering of unrelated nodes. Determinism (§9) relies
on this independence.

### Contract-vs-spec split

- **`CleanEmissionContract`** carries choices (how to shape
  output). The walker DISPATCHES on these.
- **`LanguageSpec`** carries syntax (what the output looks like).
  The walker INTERPOLATES these.
- **`Realization` records** carry per-declaration translation
  (which declaration maps to which carrier). The walker LOOKS
  these up.

This three-way split is load-bearing. Conflating choice with
syntax (e.g., "Rust-with-underscore-when-unused" as one string
template) collapses back to the hand-written emitter pattern. The
split is what lets DB-4's 8 rules grow without growing the
`LanguageSpec`, and what lets `LanguageSpec` grow without touching
the contract.

### Where E-5 blowback gets detected

E-5's "no escape hatches" promise is cashed in mechanically by
`run_post_emit_verifier` (PR #532). If walker output triggers any
warning rustc/gofmt/py_compile is configured to escalate, the run
fails. Any walker change that introduces a warning class the
contract doesn't dispatch on becomes visible as a CI failure with
a verifier-stdout message pointing at the offending line. The
remediation is structural: add a new rule type to
`CleanEmissionContract`, add a dispatch point in the walker, pick
the variant per target.

---

## 8. Spec reading protocol

The post-PR-2.5 rule is strict: **no `named_variant_id` / no
`name.starts_with(…)` / no string-keyed realization lookup in any
emission path.** Every lookup goes through a typed accessor that
the Dag builds at bootstrap.

### Typed accessors (no name keys)

The walker holds a `TargetContext` whose fields are typed
`DeclarationId` handles, not `String`. Construction fails closed
if any required declaration is missing. The four access patterns:

| Pattern | Walker API | Example |
|---|---|---|
| **Spec root lookup** | Cached accessor on `Dag` (one per target) | `dag.rust_clean_emission_spec() → Option<DeclarationId>` (pattern exists, PR #532) |
| **Realization-by-key** | `RealizationIndexes::<family>_for(key: TypedRef) → Option<&RealizationRecord>` | `ctx.realizations.callable_for(t.target)`, `ctx.realizations.operator_for(operand_type, algebra_field)` |
| **Variant ID cache** | Typed variant cache materialized at `Dag::new()` | `dag.pattern_binding_rule_variants().emit_underscore_when_unused: Option<DeclarationId>` (pattern from PR 2.5) |
| **Accessor binding** | `SubstrateAccessorBinding` resolved through typed accessor field | `ctx.accessors.binding_for(port_accessor_id) → &SubstrateAccessorRealization` |

### Two-layer cache shape

Each typed variant cache follows the PR-2.5 + PR-4
`PatternBindingRuleVariants` / `VerifierOutputPolicyVariants`
shape:

```rust
pub struct PatternBindingRuleVariants {
    pub emit_binding_always: Option<DeclarationId>,
    pub emit_underscore_when_unused: Option<DeclarationId>,
    pub emit_prefixed_underscore_when_unused: Option<DeclarationId>,
    pub not_applicable_pattern_binding: Option<DeclarationId>,
}
```

Each field is `Option<DeclarationId>` because the type declaration
may be absent from a minimal Dag (e.g., during bootstrap). Absent
is a parse failure at the consumer site, surfaced via
`VerifierParseError::MalformedSpec` or equivalent.

The walker never compares by variant name string. Dispatch is
always `constructor == variants.emit_underscore_when_unused.ok_or(...)?`,
i.e., `DeclarationId` equality against a resolved typed handle.

### Walker-side accessor surface (Stage 1e instantiation)

Stage 1e creates the following new typed caches on `Dag` — one per
closed coproduct the walker dispatches on. Each cache materializes
at `Dag::new()` by looking up its variant declarations by
structural walk from the enum's parent declaration (not by name —
the parent reference itself is already typed):

- `ExpressionWrappingRuleVariants`
- `ImportRuleVariants`
- `BlockReturnRuleVariants`
- `VariableBindingRuleVariants`
- `MatchArmBodyRuleVariants`
- `PatternStrategyVariants` — **dissolution trigger: 1e.0**
  (Python schema migration deletes `PythonPatternStrategy`;
  the cache lands in 1e.3 alongside the first `emit_pattern`
  dispatch that reads the shared `PatternStrategy`).
- `CallableStrategyVariants` — **dissolution trigger: 1e.0**
  (Python schema migration deletes `PythonCallableStrategy`;
  the cache lands in 1e.2 alongside the first `emit_transform`
  lift that reads the shared `CallableStrategy`).

These follow the PR-2.5/#532 pattern mechanically — no novel
design work. Each cache lands alongside the walker's first
consumer of that rule (E-6 compliance: "no target-spec field
lands without a same-PR consumer"). Neither `PatternStrategy`
nor `CallableStrategy` survives 1e's completion as a Python-
private coproduct — 1e.0 is the named dissolution work. If that
migration slips (a Python-specific strategy proves necessary
and 1e.0 cannot unify), it is a structural finding that halts
1e per §STOP-AND-ESCALATE.

### Forbidden lookups

The walker MUST NOT:

- call `dag.declaration_by_name(…)` at emission time (bootstrap-
  only helper, see ROADMAP §Scheduled deletions row for the 83
  compiler-internal call sites);
- parse `variant.label` / `variant.constructor_name` string fields;
- compare against `name.starts_with(target_prefix)`;
- read `TargetRealization` records without first filtering by the
  typed `language: LanguageSpec` field.

Each is a banked dissolution (see post-l15-phase-plan.md §Banked
dissolutions). Walker code that does any of these is a Stage 1e
reviewer-blocker.

### Q5 compliance

The brief's load-bearing constraint: no regression to pre-PR-2.5
patterns. The typed-cache + structural-walk pattern is the single
way the walker reads specs. PR 2.5's removal of
`named_variant_id(_, "PatternBindingRule", _)` is the invariant;
the walker inherits it without softening. If Stage 1e encounters a
structural access that would require a new name-keyed lookup, the
correct response is to ADD a typed cache (one-liner, follows the
existing shape), not to reintroduce the name key.

---

## 9. Determinism plan

Walker output must be bit-identical across re-runs of the same
`(dag, target)`. DB-8 locks the mechanical gate (byte-level diff,
`tests/determinism_test.rs` per-fixture 5× re-run); this section
pins down what the walker must do to earn that gate.

### Invariant D-1 (from DB-8)

> For any inputs `(dag, target)`, two successive calls to
> `emit(dag, target)` MUST produce byte-identical output.

Fixed-point ratchet downstream consumer (Lane 3 Stage 3c) gates
on D-1 holding. Walker that fails D-1 on any fixture blocks Stage
1e acceptance.

### Non-determinism sources to eliminate

DB-8 enumerates 8 sources; this plan commits to each elimination
strategy as a walker design rule:

| Source | Rule for the walker |
|---|---|
| **`HashMap` iteration** | `TargetContext` stores `RealizationIndexes` as `BTreeMap<DeclarationId, Realization>`. Every emission-reachable map uses `BTreeMap` OR sorts keys via fully-specified keys (`DeclarationId` tuple) before iterating. |
| **`HashSet` iteration** | `arm_body_uses: BTreeSet<PortId>` throughout pattern-binding liveness. `imports_referenced: BTreeSet<DeclarationId>` in `ImportRule::IncludeOnlyReferenced`. |
| **Timestamp embedding** | Forbidden. The walker's public surface carries zero `SystemTime::now()`; a CI grep gate in 1e acceptance rejects `SystemTime`, `chrono::`, `file!()`, `line!()` in `src/v3/compiler/src/emit.rs`. |
| **Path strings** | Generated code embeds workspace-relative paths only. `source_span.file` is rewritten through `workspace_relative()` at the emission boundary if it appears in emitted bytes. Default: paths are NOT in emitted source — only diagnostics — so the common case is path-free emission. |
| **Unstable sorts** | Emission helpers that sort use `sort_by_key` with a tuple that is total-order on the collection. Example: `bindings.sort_by_key(|(port, _)| (port.node_id, port.slot))`. No ties left unbroken. |
| **Generated identifier allocation** | Counter keyed by traversal order (DAG walk), never by HashMap iteration. Walker traversal order: declarations in `dag.declarations()` source order (`Vec<Declaration>` is insertion-ordered); nodes in `dag.nodes()` source order; ports in `dag.ports()` source order. No extraneous re-sorts. |
| **Float formatting** | Not in emission paths today. If it arrives: use `ryu` or explicit format spec (`"{:e}"`), never `f64::to_string()`. |
| **Filesystem ordering** | Walker reads zero filesystem entries. (Spec-file discovery is a bootstrap concern, not emission.) |

### Input-order discipline

The walker is deterministic IFF (input order × spec order × rule
choice) is deterministic. Each component:

- **Input order** — `Dag` fields (`declarations`, `nodes`, `ports`,
  `clusters`) are `Vec<_>` with insertion order preserved. Walker
  iterates in that order. No re-sort unless explicitly stable.
- **Spec order** — spec files parse in module order; within a
  module, declarations are iterated in source order. Realization
  indexes are built by walking declarations and inserting into
  `BTreeMap` keyed by the typed `DeclarationId` of the
  realization's target.
- **Rule choice** — each rule in `CleanEmissionContract` is a
  single variant per target; the walker's `match` arms are
  exhaustive and order-independent.

### Emission-only purity

The walker reads `TargetContext` (immutable), `Dag` (immutable),
`LocalScope` (mutated in-scope only, threaded explicitly). No
global state. No lazy init after `Dag::new()`. No side channels.

Rationale: any hidden state is a potential non-determinism entry
point. DB-8 §Rationale makes this explicit ("easier to enforce 'no
HashMap in emit' than to audit every iteration point").

### CI-level enforcement

Two gates, both lands with Stage 1e:

1. **Structural grep gate** (adapts DB-8 §Enforcement via grep
   gate):
   ```bash
   grep -rnE "(HashMap|HashSet|SystemTime|file!\(|line!\()" src/v3/compiler/src/emit.rs
   # must return zero hits
   ```
   Acceptable exceptions: internal caches that don't affect emission
   output, demonstrated by the per-fixture determinism test.

2. **Per-fixture 5× re-run test** — §12 below.

Both gates are necessary. Grep catches regressions at review time;
the 5× test catches determinism bugs that grep cannot see (e.g.,
`BTreeMap::new()` iterated in insertion order when insertion order
itself depends on `HashMap`).

### Coupling with DB-8 Step-4

Walker determinism is a LOCAL prerequisite for DB-8's full self-
host cycle. DB-8 Step 4 diffs stage1.rs against stage2.rs produced
by two different binaries; if stage 1 emit is already
non-deterministic on a single binary, Step 4 fails before binary
re-compile is even a factor. The per-fixture 5× test (§12) catches
this pre-requisite failure before Stage 3c runs.

---

## 10. Migration plan

The migration is staged by CONSTRUCT, not by TARGET. Replacing one
target's emitter wholesale before the walker handles all
constructs would create a half-emitter fork — we cannot accept
that. Instead: each construct's rendering migrates from per-target
Rust into the walker simultaneously for all three targets; the
walker's default is to delegate back to the pre-existing
`emit_<target>.rs` function for constructs not yet lifted.

### Phase structure (mirrors DB-2 §Implementation plan sub-steps)

Stage 1e executes in a **prerequisite bridge sub-stage (1e.0)**
plus DB-2's six sub-stages (1e.1 through 1e.6), ordered by
dependency. This doc locks the per-sub-stage definition of done;
DB-2 locks the 1e.1–1e.6 sizes and overall shape. 1e.0 is a
Stage-1d-surfaced bridge sub-stage that did NOT appear in DB-2's
original split because the assumption there was that every target
already sat on the shared-schema authority surface. Python
currently does not (see §7 authority table note + §3 bridge
inventory); the migration has to happen SOMEWHERE in 1e, and
making it the explicit prerequisite bridge keeps 1e.1's "resolve
the full six-authority surface for all three targets" gate
honest. Any sub-stage that ships without its definition-of-done
gates blocks the next sub-stage.

| Sub | Scope | Definition of done |
|---|---|---|
| **1e.0** (new) | **Python schema migration** onto the shared walker-authority surface. Add `python_language: LanguageSpec` root + the ten sub-spec records (`python_statements: StatementSyntax`, `python_control_flow: ControlFlowSyntax`, `python_literals: LiteralSyntax`, `python_modules: ModuleSyntax`, `python_functions: FunctionSyntax`, `python_type_definitions: TypeDefinitionSyntax`, `python_patterns: PatternMatchSyntax`, `python_value_construction: ValueConstructionSyntax`, plus the existing `python_expressions` / `python_collections` / `python_type_applications` migrated from their private `Python*Syntax` types onto shared `ExpressionSyntax` / `CollectionOps` / `TypeApplicationSyntax`). Migrate every `PythonTypeRealization` → `TypeRealization` + `language: python_language` field. Same lift for `PythonOperatorRealization` → `OperatorRealization`, `PythonCallableRealization` → `CallableRealization` (with `parameters: List<CallableParameter>`), `PythonTypeInstantiationRealization` → `TypeInstantiationRealization`, `PythonPatternRealization` → `PatternRealization`. Delete the six private scaffolded types from `spec/python.dag`. Update `emit_python.rs` to read from the shared schema instead of the private family. | `grep -rn "Python[A-Z][a-zA-Z]*Realization\|PythonCallableStrategy\|PythonPatternStrategy\|PythonExpressionSyntax\|PythonCollectionOps\|PythonTypeApplicationSyntax" src/v3/spec/python.dag src/v3/compiler/src/emit_python.rs` returns zero. All `m1_4_emit_python_test` tests pass with the migrated spec file. The six scheduled-deletion dissolution triggers named in `spec/python.dag` scaffold comments fire (rows removed from §Scheduled deletions in this PR). The stale "shared schema lacks is_copy" comment is also removed — the shared `TypeRealization` already carries `is_copy: Bool` (`src/v3/std/emit_model.dag:19`). |
| **1e.1** | Scaffold `src/v3/compiler/src/emit.rs`; `pub fn emit(dag, target) -> Result<EmittedSource, EmitError>`; `TargetContext::resolve`; top-level `emit_dag` skeleton that iterates declarations + binds and currently delegates every Behavior variant to the target's existing `emit_<target>_module`. **Hard prerequisite: 1e.0 complete** — `TargetContext::resolve` now finds the full six-authority surface for all three targets (the 1e.0 bridge made Python's surface actually exist). | Three existing emit tests (m1_3_emit_rust_test, m1_3_emit_go_test, m1_4_emit_python_test) invoke `emit(dag, target)` and produce bit-identical output to pre-1e.1 (the delegation is a passthrough). New tests `emit_resolves_rust_target`, `…_go_target`, `…_python_target` verify `TargetContext::resolve` finds all six authorities — and the Python resolution is a real test, not a partial-authority fallback, because 1e.0 made the full surface authored. |
| **1e.2** | Lift Value, Transform, Loop. Add `RealizationIndexes` + `LocalScope`. For each target: remove the corresponding branches from `emit_<target>::emit_<target>_with_mode` (the walker now owns them); passes route through `TargetContext`. | `emit(dag, target)` for all three targets produces bit-identical output for every fixture where the DAG uses only Value/Transform/Loop constructs. A new regression test enumerates the fixtures that exercise these constructs and requires byte-identity. |
| **1e.3** | Lift Branch, `emit_pattern`, `PatternBindingRule` dispatch. | All three targets' pattern-binding pilot tests pass through the walker. The `PatternBindingRuleVariants` cache is the single authority (zero `named_variant_id` name lookups in walker code; `src/v3/compiler/src/emit_python.rs::render_branch_body_expr`'s `PatternBindingRule` dispatch moves to `emit.rs`). |
| **1e.4** | Lift Bind, `emit_function_declaration`, `ExpressionWrappingRule` + `BlockReturnRule` + `VariableBindingRule` + `MatchArmBodyRule` dispatches. | All Rust emit tests pass via the walker. `emit_rust_module` becomes a three-line wrapper: build `TargetContext::resolve(dag, Rust)`, call `emit(dag, Rust)`, return bytes. |
| **1e.5** | Same lift for Go and Python. `emit_go_module` / `emit_python_module` become three-line wrappers. | All Go and Python emit tests pass via the walker. The `ImportRule::IncludeOnlyReferenced` pass runs in `emit.rs`, not in per-target files. |
| **1e.6** | Delete `emit_rust.rs`, `emit_go.rs`, `emit_python.rs`. All callers flip to `emit(dag, target)` atomically. Post-emit verifier (`post_emit_verifier.rs`) is invoked from `emit()` unconditionally; un-`#[ignore]` the three pilot roundtrip tests gated on toolchain availability. | `grep -rn "fn render_" src/v3/compiler/src/emit.rs` — zero target-specific helpers. `ls src/v3/compiler/src/emit_*.rs` — returns nothing. All `#[allow(warnings)]` attributes removed from emitted wrapper modules. |

### Parallel-run + diff discipline

Per DB-2 §"Caller migration: atomic, no shims," the final caller
flip is atomic. But the PRE-flip per-construct migration needs a
mechanical guard: each sub-stage's walker must produce byte-
identical output to the pre-sub-stage per-target emitter for
every existing fixture. The mechanism:

1. Before lifting construct `X`, snapshot every fixture's emitted
   output via `pre_walker_snapshots/`.
2. Lift `X` into the walker, replacing the per-target code.
3. Re-emit every fixture; byte-compare against the snapshot.
4. If any fixture differs: the walker's lift introduced a change.
   Either fix the walker (most common — subtle template-arg
   ordering) or update the snapshot with an explicit approval in
   the PR description.

The snapshot directory is gitignored; the snapshots are produced
on-the-fly by invoking the unpatched pre-sub-stage code via git
(`git show HEAD~1:src/v3/compiler/src/emit_rust.rs`-style) in a
throwaway compilation unit. No parallel `emit.rs` / `emit_rust.rs`
coexistence beyond a single sub-stage's lifetime; no deprecation-
attribute staging shim either (see DB-2 §Rejected alternatives
for the banked dissolution of that shape).

### Retirement triggers

Each `emit_<target>.rs` retires when:

1. Every Behavior variant routes through `emit.rs` (sub-stages
   1e.2–1e.5 all done for that target).
2. N consecutive CI runs produce byte-identical output for that
   target across all fixtures (N=3 is the trigger per the brief).
3. The per-target wrapper (`emit_rust_module`, etc.) is a pure
   delegation to `emit(dag, target)`.

When all three hold for a target, that target's `emit_<target>.rs`
is a Stage 1e.6 deletion candidate. 1e.6 deletes all three
atomically to avoid half-migrated repo state.

### Wrapper exception receipt

The Stage 1e scaffold allows **one narrow wrapper shape only**:

1. The target's render body has already moved under `emit.rs`.
2. The leftover `emit_<target>.rs` file contains only the public
   compatibility entrypoints and pure delegation into `emit(dag,
   target)` / `emit_module(dag, target)`.
3. The wrapper owns **no** target-specific render helpers, spec reads,
   caches, or behavior dispatch.

For the current branch, that exception applies to **Go only**.
`emit_go.rs` is therefore an allowed Stage 1e scaffold, not a second
authority. Rust and Python do **not** get the same wrapper carve-out in
advance; each target earns it only in the PR that actually moves that
target's render body under `emit.rs`.

**Exact dissolution trigger:** Stage **1e.6**. Once all three targets
meet the retirement triggers above, every `emit_<target>.rs` wrapper is
deleted in the same PR.

**Enforcement path:** the wrapper-parity tests (`emit_go` /
`emit_go_module` now; Rust/Python only in the same PR that migrates
their bodies) prove the wrapper is a pure forwarder. The file-shape gate
is structural: any target-specific helper still living in
`emit_<target>.rs` means the target has not crossed into the wrapper
exception yet.

**Ratchet:** no PR may create a Rust or Python compatibility wrapper
until that target's implementation body has moved under `emit.rs` and a
matching wrapper-parity test lands in the same diff. That is the
anti-replication rule for the Stage 1e scaffold; it prevents a second
"tracked bridge" from appearing without the corresponding body
migration.

### Bit-identical bar

"Bit-identical" is measured against the post-PR-532 baseline
(Python pilot + post_emit_verifier CI gate landed). Changes the
walker introduces that improve emission (e.g., finally emitting
`PatternBindingRule::EmitPrefixedUnderscoreWhenUnused` for Python
— contract-authored but never consumed pre-Lane-1e) are NOT
regressions; they require an explicit snapshot approval in the
lifting PR's description, listing the test fixtures whose outputs
change and why. This mirrors the existing emit-snapshot update
process for `lens_*_generated.rs` regenerations.

### Pessimistic fallback re-landings (§5 items)

Three items from §5 land as part of this migration, triggered by
the walker owning full emission context:

- **§5.A decl_is_copy structural walk** re-lands in 1e.3 via a
  dedicated copy-type lens consumed by the walker's ownership
  decisions.
- **§5.B `OwnedConstructLastUse`** re-lands in 1e.4 via the
  ownership lens consuming `LocalScope.position` — the walker has
  full template context at emission-order time.
- **§5.C Clone ratchet** targets ≤1 by end of 1e.6 per §5.C.
- **§5.D `Behavior::Loop` re-enables** in 1e.5 when Go and Python
  targets dispatch Loop through their `LanguageSpec.control_flow`
  record; the three `#[ignore]`d tests un-ignore in the same sub-
  stage.

Each is a sub-stage acceptance bullet, not a follow-up.

---

## 11. Bootstrap-once and snapshot-ratchet

The walker is Rust-authored in Stage 1e (per DB-2 §Out-of-scope:
"Don't design for self-hosting yet"). But the walker is also the
source from which `compiler.dag` will eventually re-emit itself
(Lane 3 Stage 3c). Squaring this circle: follow the
`lens_*_generated.rs` pattern already proven in PR #477 / #518 /
#530.

### The precedent (what works today)

Three generated lens files live at:
- `src/v3/compiler/src/lens_cost_generated.rs`
- `src/v3/compiler/src/lens_provenance_generated.rs`
- `src/v3/compiler/src/lens_structural_resolution_generated.rs`
- `src/v3/compiler/src/lens_unused_parameters_generated.rs`

Each is authored by running `emit_rust_module` over the
corresponding `.dag` source (`src/v3/lenses/<name>.dag`) and
committing the result to disk. The regen path:

1. `cargo run --bin regen_v3` invokes `compile_stage_snapshots` on
   a fixture source (`default_fixed_point_source()`), runs the
   pipeline twice, asserts bit-identity, emits Rust, and writes
   the generated file under `src/v3/compiler/src/`.
2. A `compile_stage_snapshots`-based test
   (`l1_5_fixed_point_test.rs`) verifies two consecutive runs
   produce byte-identical DAG snapshots — the fixed-point
   predecessor to DB-8's full self-host ratchet.
3. The `scripts/check-stage0-freshness.sh` hook blocks commits
   that change a lens `.dag` without regenerating the `_generated.rs`
   mirror.

### Walker's Rust mirror — future shape (out of 1e scope)

When Stage 3c's compiler.dag expresses the walker itself, an
`emit_generated.rs` would join the four existing generated files.
That is not in scope for 1e; in 1e the walker IS hand-written
Rust. But 1e's shape must be COMPATIBLE with eventually
generating it, which means:

- walker code at `src/v3/compiler/src/emit.rs` has the same
  structure a `.dag` lens would produce (pure functions, no
  global state, explicit `Dag` parameter, `BTreeMap` keyed
  lookups, …);
- no Rust features that are walker-only (no `async fn`, no
  macros, no trait dispatch polymorphism) — same constraint the
  existing `lens_*_generated.rs` files respect today.

### Snapshot-ratchet for walker output

The walker's OUTPUT (emitted Rust/Go/Python for every fixture)
becomes the snapshot the 1e.X sub-stages ratchet against. Shape
per sub-stage:

```
tests/walker_snapshots/
  rust/
    arithmetic.fixture.rs        (committed — a walker run's exact output)
    branch.fixture.rs
    list_fold.fixture.rs
    ...
  go/
    …
  python/
    …
```

A new test (`tests/walker_snapshot_test.rs`) iterates every
fixture for every target, runs `emit(dag, target)`, byte-compares
against the committed snapshot. Snapshot updates land via an
explicit `cargo run --bin regen_walker_snapshots` invocation that
writes the new bytes and requires manual PR approval.

This ratchet is the 1e-scoped analog of DB-8's self-host cycle:

- 1e.X sub-stage emits a snapshot corpus; the corpus is the
  contract between sub-stages.
- Changes across sub-stage boundaries show up as snapshot diffs
  that the sub-stage's PR must justify or fix.
- After 1e.6, the snapshot corpus remains as the per-emission
  determinism + correctness gate.

### Bootstrap question — when does the walker's own Rust mirror land?

Never, via this lane. Lane 1e ships `emit.rs` hand-written. Lane
3c ships the mechanism to re-emit `emit.rs` from a `compiler.dag`
that expresses the walker. Between 1e and 3c, the walker is
hand-edited Rust on every change — the same arrangement the
existing compiler pipeline (`parse.rs`, `lower.rs`, `infer.rs`)
carries today. No bootstrap inversion; no "regen the walker from
itself" cycle during 1e.

### The three precedents consulted

- **PR #477** — established the `emit_rust_module` → generated
  file pattern for `lens_cost_generated.rs`.
- **PR #518** — extended the pattern to
  `lens_structural_resolution` with a predicate whose
  `ArrowBody::NoBody` distinction supports the fixed-point
  snapshot flow.
- **PR #530** — refreshed snapshots + tightened the fixed-point
  pattern (two passes, byte diff, `compare_stage_snapshots`
  failure message pinpoints the first differing stage).

The walker follows #477's file layout, #518's fixed-point
discipline, and #530's diff-failure UX. No novel mechanism.

---

## 12. Determinism test suite

DB-8 prescribes `tests/determinism_test.rs` as per-fixture 5×
re-run. This section locks Stage 1e's version of that test.

### Test shape (adapts DB-8 §`tests/determinism_test.rs`)

```rust
// src/v3/compiler/tests/determinism_test.rs
use v3_compiler::{compile_to_dag, emit, TargetLanguageId};

#[test]
fn emit_is_deterministic_on_every_fixture() {
    for fixture in all_fixtures() {
        for target in [TargetLanguageId::Rust, TargetLanguageId::Go, TargetLanguageId::Python] {
            let dag = compile_to_dag(&fixture.source, &fixture.name)
                .unwrap_or_else(|e| panic!("compile failed on {}: {e:?}", fixture.name));

            let outputs: Vec<String> = (0..5)
                .map(|_| emit(&dag, target).unwrap().text)
                .collect();

            for i in 1..5 {
                assert_eq!(
                    outputs[0], outputs[i],
                    "emit is non-deterministic on fixture {} target {:?}: run 0 vs run {} differ",
                    fixture.name, target, i
                );
            }
        }
    }
}
```

### Fixture coverage requirement

The test iterates the Stage 1e fixture set (one per Behavior
variant + one per major syntactic construct — ~20 fixtures at 1e
kickoff, growing to the full regression corpus by 1e.6). Every
fixture runs against every supported target. A target without
coverage of a construct is not valid: an emit call that returns
`EmitError::UnsupportedBehavior` is a legitimate outcome the test
allows, but an emit that returns `Ok(_)` with non-deterministic
bytes is a failure.

### Failure mode

The test panics on the first non-determinism it finds, with the
fixture name and target. A side test
`print_determinism_diagnosis_for_failing_fixture` is a manual
helper (not run in CI) that prints the byte diff between two
runs, for debug.

### When it runs

- **Every PR that touches `src/v3/compiler/src/emit.rs`** or
  `src/v3/spec/` or `src/v3/std/clean_emission.dag` or
  `src/v3/std/emit_model.dag` — gated by a CI conditional on
  those paths.
- **Every push to main** — unconditional.

### Coupling with DB-8's self-host cycle gate

The determinism test is a LOCAL precondition for DB-8's full
cycle. If this test fails, the self-host cycle fails
automatically — there is no way to produce a bit-identical
stage2.rs from a non-deterministic walker. Running this test
pre-cycle catches the common case (a `HashMap` iteration slipped
into emit) before the ~33s cycle runs.

### Stage 1e acceptance adds this test

Stage 1e.6 closure requires `tests/determinism_test.rs` landed,
passing across every fixture for every target, wired into the
two CI triggers above. This is a hard gate on 1e completion. Lane
3 Stage 3c inherits the passing test as precondition; 3c does
not re-design it.

### Augmenting the existing fixed-point test

`src/v3/compiler/tests/l1_5_fixed_point_test.rs` already does
pipeline fixed-point (parse/lower/infer/compute_ownership/
lens_complexity/emit are snapshotted and compared). Stage 1e.1
adds walker emission to this flow without replacing the existing
test: the `compile_stage_snapshots` "emit" stage switches its
backend from `emit_rust::emit_rust(&lower_dag)` to
`emit(&lower_dag, TargetLanguageId::Rust)`, and a new fixed-point
test iterates targets (`Rust`, `Go`, `Python`) rather than
hardcoding Rust. Determinism + pipeline fixed-point become
composable rather than duplicated.

---

## Out-of-scope

- Any implementation of the generic walker — that's P2-L1.
- Actually fixing any bridge (B11, B13, B14) — those follow in P2.
- Deciding the generic walker's Rust vs `.dag` home. The walker is
  Rust in P2 (stays in `src/v3/compiler/`), becomes `.dag` in P3
  (self-hosting cycle). Don't design for self-hosting yet.
- New substrate types beyond what the gap list identifies — if a
  type is needed by the walker but isn't in scope of any existing
  spec, note it in the gap list; don't design it here.

---

## Direction

**Inventory first, synthesis second.** Start with the mechanical work
of listing every emitter function. Patterns emerge from volume: once
every function is classified, the walker's shape is visible as the
intersection of "what's common across all classifications."

**Be ruthless about substrate gaps.** If a function classification
would require "the spec doesn't have a way to say X", write it down.
P2's first sub-stage will be spec additions; the gap list is the scope.

**Don't over-plan the walker's implementation.** This lane's output is
enough material for an implementer to write the walker next phase — not
the walker itself. Designs that specify APIs too tightly often need to
be thrown out once implementation starts.

---

## Escalation criteria

Stop work and surface if:

1. **>30% of emitter functions fall into "per-target integration"**
   — that means the consolidation premise is wrong. The theory says
   ~95% should be spec-driven or lens. If the fraction is materially
   lower, either the classification is too conservative (re-evaluate)
   or the targets are more divergent than the thesis assumed (escalate
   to thesis-level review).

2. **Substrate gap list exceeds ~10 new type additions** — that's
   roughly a phase of work by itself. Either the gap list is
   overspecified (aggregate), or consolidation needs a prerequisite
   phase that extends the substrate. Surface.

3. **No safe pilot target candidate exists** — if both SPICE and
   English have significant substrate gaps, the pilot itself becomes
   a multi-lane effort. Surface; reconsider whether P2 starts with
   Rust-only consolidation (proving against the existing target)
   before adding a new one.

4. **Name-based dispatch is pervasive beyond expectation** — if the
   bridge inventory finds 20+ distinct name-prefix or
   string-comparison sites, B11 is the tip of the iceberg. Surface;
   this may need a dedicated debridge phase before consolidation.

---

## Acceptance gates

Lane is done when all five hold:

- `docs/emit-functions-inventory.md` classifies every `fn render_*` /
  `fn emit_*` across the three emitters (count verified by `grep`).
- `docs/spec-field-gaps.md` enumerates each needed spec extension,
  tagged by priority (blocks consolidation vs nice-to-have).
- `docs/emit-bridges.md` lists every known bridge (name-based
  dispatch, hardcoded convention, per-target Rust branch) with
  dissolution target.
- Pilot target evaluation written and linked.
- P2-L1 owner (whoever takes it) reviews and signs off on the plan
  before P2 starts.

---

## Dependencies

- **Requires:** Half A + Half B merged (so the inventory reflects
  the current state, not a moving target).
- **Blocks:** P2-L1 (consolidation implementation needs this plan).
  Hard gate.
- **Does not block:** P1-L1 or P1-L2.

---

## Estimate

- Emitter function inventory: 2 days (mechanical but thorough)
- Spec field gap list: 1.5 days (requires reading each function body)
- Bridge inventory: 1 day (grep-driven)
- Pilot target evaluation: 0.5 day
- Review + sign-off cycle: 1 day

Total: ~6 implementer-days.

---

## Success signal

When P2-L1 starts, the implementer reads these three design docs and
can:

1. Open each current emit_* file knowing which functions to delete,
   extract, or keep
2. Add the needed spec extensions in a single batch commit (no
   "oh wait, I need another field" mid-implementation)
3. Write the generic walker against a clear API: "read each node's
   Behavior, look up the CallableRealization / MatchStrategy /
   whatever, substitute into the declared template, recurse on inputs"

If P2-L1 needs to pause and redesign mid-implementation, this lane
under-specified something. The escalation criteria above are designed
to catch those before they become P2's problem.
