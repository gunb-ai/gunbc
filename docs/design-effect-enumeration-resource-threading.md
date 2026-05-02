# Effect Enumeration — Resource Threading & OperationEffect Retirement

> Part of: [`docs/r3-structure.md`](r3-structure.md) lane T-Lens-Behavioral-Parity slice 4 | [`docs/v3-lens-capability-register.md`](v3-lens-capability-register.md) row `effect_enumeration.dag` | [`../INVARIANTS.md`](../INVARIANTS.md)
>
> Companion authority: [`docs/design-db18-workflow-effect-carrier.md`](design-db18-workflow-effect-carrier.md) (`WorkflowEffect` substrate input carrier) | [`docs/design-composed-effect-reshape.md`](design-composed-effect-reshape.md) (`CompositionVerdict` output carrier).
>
> **Purpose:** specify the substrate-shape transition that takes `effect_enumeration.dag` from BEHAVIORALLY PARTIAL to BEHAVIORALLY COMPLETE. Concretely: (a) retire ambient resource/transport metadata as the source of effect facts, (b) make resource-threaded callable signatures the structural authority, (c) pin caller-side effect sets explicitly via the `Operation` declaration set, (d) retire `OperationEffect` as the carrier that smuggles transport facts into the algebra.
>
> **Authority discipline:** R3 design doc; the implementation lane is T-Lens-Behavioral-Parity slice 4 (see [`docs/r3-structure.md`](r3-structure.md) row 146). This doc resolves the design questions that block lane dispatch.

## What this document is

[`docs/v3-lens-capability-register.md`](v3-lens-capability-register.md) row 42 says of `effect_enumeration.dag`:

> Audit verdict is path (ii): live primitives still require ambient resource/transport metadata rather than returned-modified-resource signatures (`dsl/std/resources.dag`, `dsl/std/primitives.dag`, `dsl/extdeps/shell.dag`, `dsl/extdeps/github/auth.dag`). Full `OperationEffect` retirement, resource-threading migration, and caller-side effect-set pinning are follow-up work.

That sentence names four substrate-level moves bundled under one closure gate (`effect_enumeration_lens_behaviorally_complete`). This doc specifies them as one atomic migration. Per [`feedback_closed_system_effects`](../README.md#) and [`../INVARIANTS.md`](../INVARIANTS.md) §P1: effects are not a parallel taxonomy alongside the substrate — they derive structurally from typed primitive composition. Today's substrate violates that by carrying effect intent in three places (resource block, transport block, `OperationEffect` record); this design collapses all three onto **two orthogonal structural authorities** (per §2.4): (a) the callable's arrow signature carries the **effect set** (which resource types are involved); (b) the callable's algebra inhabitance (`inhabits IdempotentRead<R>` / `inhabits Mutating<R>`) carries the **effect kind** (read/write/append). Neither is convention-level; both are structural. The signature alone cannot distinguish read from write since `R in / R out` and `R in / R' out` have the same typed signature.

## §1. Ambient-metadata audit (current state)

The four files named in row 42 carry effect intent in three structurally distinct ambient channels. Each one expresses "this operation has effect X" without that fact appearing in the operation's *type signature*. The audit catalogs each channel concretely.

### §1.1 Channel A: resource block ambient kind/mode

`dsl/std/resources.dag` declares `Filesystem`, `Network`, `Clock`, `AuthContext` as `resource` blocks with `kind: Capability | Observation` and `mode: Read | ReadWrite | Write` *ambient on the block*:

```dag
resource Filesystem {
  kind: Capability        // Channel A: ambient effect kind
  mode: ReadWrite         // Channel A: ambient access mode
  acquire {}
  release {}

  capability read {
    input  { path: TextFilePath }
    output { content: String }   // Channel A leak: signature does not carry Filesystem
  }
  capability write {
    input  { path: FilePath, content: String }
    output { written: Bool }     // Channel A leak: signature does not say "writes Filesystem"
  }
}
```

The capability's *signature* is `path → content` and `(path, content) → Bool`; the `Filesystem` resource itself never appears in the input/output types. The fact that `read` reads filesystem state and `write` mutates it lives in the surrounding `resource Filesystem { mode: ReadWrite }` block, not in the function type. A consumer reading just the capability signature cannot tell `read` from a pure `String → String`.

**Ambient consumers today:** the legacy `dsl/std/effects.dag::is_idempotent_effect` and the v3 `derive_effect_shape(method, path)` both bypass the signature and consult these ambient facts. Per [`feedback_lenses_not_passes`](../README.md#) and [`../INVARIANTS.md`](../INVARIANTS.md) §P1 *"heuristics indicate lost structure"*: every consumer that has to "look up the resource block" is evidence that the structural fact (the resource lives in the signature) was dropped at the wrong layer.

### §1.2 Channel B: transport block ambient effect-shape

`dsl/extdeps/shell.dag` declares 4 services × 8 operations using a `transport shell { argv: [...] }` block plus a `readonly` keyword:

```dag
service shell.Find {
  operation ListDirs {
    input  { path: FilePath, max_depth: Int = 1, min_depth: Int = 1 }
    output { dirs: List<FilePath> from "stdout_lines" }
    readonly                                              // Channel B: ambient effect shape
    transport shell { argv: ["find", ...] }               // Channel B: ambient transport identity
    exit { 0 => Unit; nonzero => String "Find command failed" }
  }
}
```

Three ambient facts here:
1. **`readonly` keyword** — names the effect shape outside the type signature.
2. **`transport shell { argv: [...] }`** — names that the operation interacts with the host shell (carries an implicit `Process` resource).
3. **`exit { ... }`** — narrows the failure mode but lives outside the return type.

Audit: across `dsl/extdeps/` there are 27 services and 67 operations using exactly this pattern. `dsl/extdeps/git.dag`, `cargo.dag`, `gunbc.dag`, `cron.dag`, `browser.dag`, `llm/cli.dag`, `apt.dag`, `brew.dag`, `rustup.dag` all repeat the shape. `dsl/extdeps/llm/openai.dag` uses `transport rest { ... }` for the same pattern over HTTP. Every `transport <kind> { ... }` block is one Channel-B leak: the operation's *callable type* is `(input fields) → (output fields)` and never mentions the resource the transport block names.

### §1.3 Channel C: `uses x: Resource` ambient binding

`dsl/extdeps/github/auth.dag::github_token` and 19 other functions across the codebase declare `uses` clauses ambient on the function:

```dag
func github_token(
  project_id: NonEmptyStr = "gunbai-secrets",
  secret_name: NonEmptyStr = "github-token"
) -> GitHubSecretManagerPat
  uses net: Network                                       // Channel C: ambient resource binding
{
  cred = gcp_secret_credential(project_id: project_id, secret_name: secret_name)
  return { token: cred.token }
}
```

The function signature claims to be a pure `(NonEmptyStr, NonEmptyStr) → GitHubSecretManagerPat`. The `uses net: Network` clause is the ambient-effect leak: the function's *actual* shape is a network read, but the type system cannot see that because `Network` does not appear in inputs or output. **20 call sites across `dsl/`** use this clause: `dsl/tools/bootstrap.dag`, `dsl/tools/readme.dag`, `dsl/extdeps/github/auth.dag`, `dsl/gunbc/auth/credentials.dag`, `dsl/gunbc/auth/patterns.dag` (×3), `dsl/gunbc/tools/gist.dag` (×3), and the `dsl/extdeps/extdeps.md` design example.

### §1.4 Channel D: `OperationEffect` record (the smuggled-fact carrier)

The downstream consequence of A/B/C is that `src/v3/std/effects.dag` carries `OperationEffect` as a separate record reconstructing what the signature should have said:

```dag
type OperationEffect {
  operation_name: String
  shape: EffectShape         // smuggled from resource/transport ambient metadata
}

fn derive_op_effect(
  operation_name: String,
  method_str: String,         // raw transport string
  path_str: String            // raw transport string
) -> DeriveOpEffectResult { ... }
```

The pipeline `derive_effect_shape(method: HttpMethod, path: PathTemplate) -> EffectShape` walks transport strings and produces an effect shape per the HTTP-method × path-key cross-product. **This is the parallel taxonomy [`feedback_closed_system_effects`](../README.md#) names as the bug pattern**: an effect record alongside the type signature, populated by walking transport metadata that should have been a signature shape from the start. Every line of `derive_op_effect` in `src/v3/std/effects.dag:707-755` is a structural-decompression heuristic operating on what should have been declared facts.

### §1.5 Audit summary — three channels feeding one carrier

| Channel | File | Mechanism | Ambient fact |
|---|---|---|---|
| A | `dsl/std/resources.dag` | `resource X { kind, mode }` block + `capability` blocks inside it | resource identity + access mode lives outside capability type |
| B | `dsl/extdeps/shell.dag` (× 27 services) | `transport shell { argv }` + `readonly` keyword | transport identity + readonly intent live outside operation type |
| C | `dsl/extdeps/github/auth.dag` (× 20 functions) | `uses x: Resource` clause on `func` / `pattern` | resource binding lives outside function type |
| D | `src/v3/std/effects.dag::OperationEffect` | record alongside operation declaration | downstream re-derivation of A/B/C as a side-channel taxonomy |

All four collapse to the same root cause: **a callable signature `(inputs) → output` does not carry the resource it threads.** The structural fix is one shape change at the substrate level; A/B/C/D dissolve as consequences.

## §2. Resource-threading target shape

Per [`feedback_closed_system_effects`](../README.md#): *every external mutable resource (file handles, sockets, db connections, log files, transactions) is modeled as a typed parameter that's returned modified. `log.info(msg, log: LogFile) → LogFile'`.* The substrate target is the same pattern, with one structural addition (the modified resource appears in the return type) and a corresponding deletion (the ambient blocks).

### §2.1 Before vs after — capability declaration

```dag
// BEFORE — Channel A (dsl/std/resources.dag)
resource Filesystem {
  kind: Capability
  mode: ReadWrite
  capability read {
    input  { path: TextFilePath }
    output { content: String }
  }
  capability write {
    input  { path: FilePath, content: String }
    output { written: Bool }
  }
}

// AFTER — resource-threaded signatures, no resource block
type Filesystem { /* opaque carrier, threaded through ops */ }

func read(fs: Filesystem, path: TextFilePath) -> { fs: Filesystem, content: String }
// reads structurally: same Filesystem in / same Filesystem out → no mutation observable in signature.

func write(fs: Filesystem, path: FilePath, content: String) -> { fs: Filesystem, written: Bool }
// writes thread the Filesystem resource through; kind (read vs write) declared via
// algebra inhabitance — see §2.4 + §8.1.
```

The structural fact the signature carries is the **effect set** — the set of resource types involved in the operation (here, `Filesystem`). The signature does NOT structurally distinguish read from write — `R in / R out` and `R in / R' out` (same type, different value) have the *same typed signature* unless value-preservation is itself a substrate-level fact, which `.dag` does not encode at the type level.

**The read/write distinction is declared via algebra inhabitance on the callable**, not derived from signature shape. Per [`docs/design-composed-effect-reshape.md`](design-composed-effect-reshape.md) (PR #529 R3), `EffectShape = IsIdempotent(IdempotentShape) | IsBreaking(BreakingShape)` is the existing partition that captures this distinction structurally. The post-retirement substrate retains this carrier as an algebra-inhabitance declaration on the callable: `inhabits IdempotentRead<Filesystem>` (commutes, repeatable) vs `inhabits Mutating<Filesystem>` (non-commuting, observable side-effect). The lens reads inhabitance for kind classification; the signature reads only for the effect set. Two structural carriers, two orthogonal facts — neither convention-level.

See [§5.1](#51-readwrite-distinction-substrate-authority) for the full algebra-inhabitance shape and [§8.1](#81-readwrite-distinction-substrate-authority) for the resolved design question.

### §2.2 Before vs after — service operation

```dag
// BEFORE — Channel B (dsl/extdeps/shell.dag)
service shell.Exec {
  operation Run {
    input  { script: String }
    output { exit_code: Int from "exit_code"
             stdout: String from "stdout"
             stderr: String from "stderr" }
    transport shell { argv: ["sh", "-lc", "{script}"] }
  }
}

// AFTER — Process is a threaded resource; transport block becomes target-side realization (E-9)
type Process { /* opaque, threaded */ }

func shell_exec_run(proc: Process, script: String)
  -> { proc: Process, exit_code: Int, stdout: String, stderr: String }
```

The `transport shell { argv: [...] }` block does **not** appear in the source-level callable. It moves to `Arrow.body` per [`../INVARIANTS.md`](../INVARIANTS.md) §E-9 *External Realization Lives On Arrow.body* — a per-target binding that the emitter consumes when lowering to a target language. The source-level shape is just the typed signature.

### §2.3 Before vs after — auth function

```dag
// BEFORE — Channel C (dsl/extdeps/github/auth.dag)
func github_token(
  project_id: NonEmptyStr = "gunbai-secrets",
  secret_name: NonEmptyStr = "github-token"
) -> GitHubSecretManagerPat
  uses net: Network
{ ... }

// AFTER — Network threaded through signature
func github_token(
  net: Network,
  project_id: NonEmptyStr = "gunbai-secrets",
  secret_name: NonEmptyStr = "github-token"
) -> { net: Network, token: GitHubSecretManagerPat }
{ ... }
```

The `uses net: Network` clause disappears. The function's *type* declares its effect dependency: `Network` in input ∩ output identifies the effect set. The kind (this `github_token` is a read against the secrets-API service; it does not mutate Network state) is declared via algebra inhabitance: `inhabits IdempotentRead<Network>`. The lens reads both — the signature for set, the inhabitance for kind — per §2.4.

### §2.4 The unified rule

**The unified rule** (split into two structural facts, addressing the cursor BLOCKING finding 2026-05-02 at line 151):

> **(a) Effect SET — derived from signature**: a callable's effect set is exactly the set of resource types appearing in *both* its inputs *and* its output. (Pure operations: no resource type in either position.)
>
> **(b) Effect KIND — declared via algebra inhabitance**: each callable in the effect set declares its kind per resource via an `inhabits` clause: `inhabits IdempotentRead<R>` (read), `inhabits Mutating<R>` (write), or `inhabits Append<R>` (append). The kind is a structural fact (algebra inhabitance), not a signature-shape inference.

The two facts are orthogonal: the signature carries which resources are involved; the inhabitance carries how each is touched. Neither is convention-level; both are structural.

**Lens implementation**: the `effect_enumeration.dag` lens walks `inputs / output / body` per `src/v3/lenses/effect_enumeration.dag:152-164` for the effect set, and looks up the callable's `EffectShape` inhabitance for the kind:

```dag
// pseudocode (post-migration shape)
fn classify_effect(callable: Callable, resource: ResourceType) -> EffectShape =
  if callable_inhabits(callable, idempotent_read_for(resource)) then
    IsIdempotent(ReadShape)
  else if callable_inhabits(callable, mutating_for(resource)) then
    IsBreaking(WriteShape)
  else if callable_inhabits(callable, append_for(resource)) then
    IsIdempotent(AppendShape)         // appends are idempotent under merge
  else
    Diagnostic::EffectKindUndeclared { callable, resource }
```

The signature alone is insufficient — the lens MUST consult algebra inhabitance to classify reads vs writes. A callable with the resource in input and output but no kind inhabitance is a fail-closed Diagnostic (`EffectKindUndeclared`), not a silent default.

The substrate migration is correspondingly two parts: (a) thread resources through signatures (covered in §2.1-§2.3 above; structurally identifies the effect set); (b) declare `inhabits IdempotentRead<R>` / `inhabits Mutating<R>` on each effectful callable (declares the kind structurally).

## §3. Caller-side effect-set pinning

The user-facing closure gate names "caller-side effect-set pinning" as a follow-up component. Under the resource-threaded design, pinning composes naturally with §2 rather than introducing a parallel mechanism.

### §3.1 What pinning means structurally

A caller pins its effect set when its *own* callable signature constrains which resource types appear in its inputs. The composed-callable rule:

> If callable `f(R, x) → (R, y)` is invoked from inside callable `g(...) → ...`, then `R` must appear in `g`'s inputs (or be acquired structurally inside `g`'s body via a constructor that itself fits the threading rule). A caller cannot hide a transitive resource dependency.

This is enforced by-construction: `g`'s body cannot construct an arbitrary `R: Filesystem` value (the type is opaque per §2.1 and §2.5 below); the only way for `g`'s body to use `f` is if `g` accepts `R` as an input or produces `R` from a `Filesystem`-constructing primitive that itself threads. The closure of resource types across body composition IS the pinned set; no separate annotation is required.

### §3.2 The pinning surface — `Operation` declarations

The pinning *substrate carrier* already exists: `src/v3/std/services.dag::Operation`:

```dag
type Operation {
  callable: CallableRef          // declaration whose signature carries the resource thread
  inputs:   Map<String, InputField>
  endpoint: RestEndpointBinding  // per-target realization (E-9 binding)
}
```

The `Operation` row pins (a) which callable declares the operation, (b) the input field set (already keyed by name with cardinality enforced by `Map`), and (c) the per-target binding. **Resource pinning IS the `callable: CallableRef` field plus the threaded signature on the referenced declaration.** No new top-level carrier is required; the thread-through-signature rule of §2 plus the existing `Operation.callable: CallableRef` provides the pinning surface.

When the lens walks a program, it discovers each `Operation`, follows `callable` to the arrow declaration, and reads the resource set off the arrow signature directly. This is the same pattern as [`docs/design-db18-workflow-effect-carrier.md`](design-db18-workflow-effect-carrier.md) §"Authority site for `WorkflowEffect`" — facts live at their natural site, not in a parallel record.

### §3.3 What does *not* pin (rejected)

Per [`feedback_no_annotations`](../README.md#) and [`feedback_no_metadata_markers`](../README.md#), the following pinning shapes are **not admitted** in the substrate:

- A `pinned_effects: Set<EffectKind>` field on a `Callable` declaration. (Annotation; duplicates the structural fact.)
- A `requires { fs: Filesystem }` clause distinct from the `inputs` field. (Parallel-representation debt with the typed input.)
- A lens-level "declared vs derived" cross-check. ([`feedback_closed_system_effects`](../README.md#): the modeling IS the declaration.)

If a caller wants to *constrain* the effect set tighter than its body composition implies, that is asymmetric tightening — done by writing a wrapper function with a narrower signature, not by adding a meta-annotation. Per [`feedback_closed_system_effects`](../README.md#) §"How to apply" item 4: "asymmetric tightening at the caller site (caller pins ⊆ smaller set) — structural type matching, not separate lens."

## §4. `OperationEffect` retirement path

Per [`docs/r3-structure.md`](r3-structure.md) row 146 + multiple cross-lane closure gates: `OperationEffect` retirement is load-bearing for the entire R3 lens-behavioral-parity slice. After §2 + §3 land, `OperationEffect`'s payload becomes derivable rather than authored, and the carrier dissolves rather than gets reshaped.

### §4.1 Where `OperationEffect` lives today

| Site | Role | Migrates to |
|---|---|---|
| `src/v3/std/effects.dag:421` (declaration) | Per-op record `{ operation_name, shape }` | Deleted |
| `src/v3/std/effects.dag:431` (`DeriveOpEffectResult`) | Output of `derive_op_effect` | Deleted (function dissolves) |
| `src/v3/std/effects.dag:454` (`CompositionVerdict.BrokenBy`) | `first_breaker: ElementRef<OperationEffect>` | Reshape to `ElementRef<Operation>` |
| `src/v3/std/effects.dag:501` (`compose_effects`) | `(List<OperationEffect>) → CompositionVerdict` | Reshape to `(List<Operation>) → CompositionVerdict` (signature-derived shapes) |
| `src/v3/std/effects.dag:550` (`WorkflowEffect.LinearEffect.ops`) | `List<OperationEffect>` | Reshape to `List<Operation>` (per [`docs/design-db18-workflow-effect-carrier.md`](design-db18-workflow-effect-carrier.md) §"Constraints" — DB-18 locks `WorkflowEffect`'s 4-variant set + variant payload shape; element-type refinement *within* `LinearEffect.ops: List<X>` is additive tightening, not a STOP-AND-ESCALATE-class variant change. The variant set stays `LinearEffect | BranchEffect | ParallelEffect | LoopEffect` and `ops` stays a `List<X>` — only the element type X retypes from `OperationEffect` to `Operation` once `Operation`'s threaded signature carries the same composition input shape `OperationEffect` did) |
| `src/v3/std/effects.dag:723` (`derive_op_effect`) | `(name, method_str, path_str) → OperationEffect` | Deleted (heuristic dissolves) |
| `src/v3/std/effects.dag:840` (`check_modifier_vs_derivation`) | `(OperationEffect, declared_*) → ModifierCheck` | Deleted (no `idempotent` modifier to check; structural shape is the declaration) |
| `src/v3/compiler/src/dag.rs:86` (Rust mirror) | Hand-Rust mirror | Deleted (consumer migrates to `Operation`) |
| `src/v3/compiler/src/workflow_idempotency.rs:23` | `ElementRef<OperationEffect>` consumer | Reshape to `ElementRef<Operation>` |
| `src/v3/compiler/src/workflow_parallelism.rs:137` | `CompositionVerdict::BrokenBy { first_breaker }` consumer | Same — payload type reshapes |

### §4.2 Why deletion, not refinement

[`docs/design-composed-effect-reshape.md`](design-composed-effect-reshape.md) R3 retired `ComposedEffect` by deletion when reviewer feedback showed the carrier paired two facts the type system could not correlate. The same argument applies to `OperationEffect`: it pairs `operation_name: String` with `shape: EffectShape`, where `shape` is supposed to be derivable from the operation's transport metadata. Once the resource-threaded signature *is* the structural fact, both fields of `OperationEffect` are derivable from one richer authority (`Operation` with its `callable: CallableRef`):

- `operation_name` ← `callable.decl` resolution (already the convention per `src/v3/std/services.dag:108-115`).
- `shape: EffectShape` ← arrow-signature read off `callable.decl`'s arrow body (already implemented as `callable_arrow_effect` in the lens).

Per [`feedback_parallel_representation_debt`](../README.md#) and [`../INVARIANTS.md`](../INVARIANTS.md) §P5 — when a canonical source exists, consume it rather than scaffold. The canonical source is `Operation` (already declared at §3.2). `OperationEffect` is the scaffold; deletion is the dissolution receipt.

### §4.3 The four-variant `EffectShape` partition (kept)

[`docs/design-composed-effect-reshape.md`](design-composed-effect-reshape.md) R2 partitioned `EffectShape` into `IsIdempotent(IdempotentShape) | IsBreaking(BreakingShape)`. **That partition is kept** — the post-retirement shape reads `EffectShape` from the callable's *algebra inhabitance* (per §2.4 + §8.1: `inhabits IdempotentRead<R>` → `IsIdempotent(ReadShape)`; `inhabits Mutating<R>` → `IsBreaking(WriteShape)`) and the partition still classifies the result. What changes is the *source* of the shape: today it's `derive_effect_shape(method, path)` walking ambient metadata; after migration it's a structural read off the declared algebra inhabitance on the callable. (The signature is consulted separately for the effect *set*; kind classification reads inhabitance.) The downstream algebra (`compose_effects`, `is_idempotent_effect`, `BoundedLattice` composition over `IdempotentShape` per `src/v3/std/effects.dag:55`) is unchanged.

### §4.4 The replacement signature

```dag
// AFTER — compose_effects consumes Operation rows directly
fn compose_effects(ops: List<Operation>) -> CompositionVerdict {
  // For each op:
  //   - Resolve op.callable.decl
  //   - Read the arrow signature
  //   - Apply §2.4(a) (resource in inputs ∩ output) for effect set
  //   - Apply §2.4(b) (algebra inhabitance lookup) for effect kind → EffectShape
  //   - Project through IsIdempotent / IsBreaking partition
  // Same lattice composition as today; same verdict shape.
}
```

The `WorkflowEffect.LinearEffect.ops: List<Operation>` reshape is the upstream change; `compose_effects` follows. `WorkflowEffect`'s four-variant outer shape is untouched (DB-18 lock); only the inner `ops` element type migrates.

## §5. Migration scope (call-site count + touch-pattern)

Per [`feedback_audit_adjacent_authority_first`](../README.md#): catalog every consumer's structural questions before editing the substrate.

### §5.1 Substrate touch-pattern (single PR)

| Change class | Files | Count |
|---|---|---|
| Resource-block deletion + capability re-shape | `dsl/std/resources.dag` | 1 file, 4 resource blocks (`Filesystem`, `Network`, `Clock`, `AuthContext`) |
| `transport shell` block → E-9 `Arrow.body` realization | `dsl/extdeps/*.dag` | 23 files, 27 services, 67 operations |
| `transport rest` block → E-9 `Arrow.body` realization | `dsl/extdeps/llm/openai.dag`, `cloud/gcp/secret_manager.dag`, etc. | ~5 files (subset of 23 above) |
| `uses x: Resource` clause → threaded input/output | `dsl/tools/*.dag`, `dsl/extdeps/*.dag`, `dsl/gunbc/auth/*.dag` | 20 call sites |
| `OperationEffect` deletion | `src/v3/std/effects.dag`, `src/v3/compiler/src/dag.rs`, `workflow_idempotency.rs`, `workflow_parallelism.rs` | 4 files |
| `WorkflowEffect.LinearEffect.ops` element-type reshape | `src/v3/std/effects.dag` (1 line of carrier) + downstream consumers | 1 substrate edit + 2 consumer edits |
| Lens consumer (no logic change; existing fold matches new shape) | `src/v3/lenses/effect_enumeration.dag` | 0 logic change; status flip from PARTIAL to COMPLETE |
| Capability-register row update | `docs/v3-lens-capability-register.md` | 1 row |

**Total touch:** ~54 `.dag` files + 4 Rust files + 2 doc files.

### §5.2 Why this fits one PR

The migration is wide but **shallow per file**. Each `transport shell { argv: [...] }` block deletion is a mechanical move to E-9 `Arrow.body` realization; per [`docs/r3-structure.md`](r3-structure.md) the E-9 binding pattern is already landed (DB-14: external primitives materialize through `Arrow.body` plus target bindings). The substrate carriers exist; this migration is *consumer*-side population.

The depth-cost is concentrated in two places: (a) `OperationEffect` deletion + `Operation`-consumer wiring in `effects.dag` (~150 lines of `effects.dag` that go away), and (b) the four `Filesystem` / `Network` / `Clock` / `AuthContext` capability re-shapes that propagate to 20 `uses` call-sites. Both are mechanical once the design is locked.

### §5.3 Cross-program coordination

| Manager | Scope | Files |
|---|---|---|
| **Substrate Manager** | `OperationEffect` retirement, `compose_effects` reshape, `WorkflowEffect.LinearEffect.ops` element type | `src/v3/std/effects.dag`, Rust mirrors |
| **Substrate Manager** (extension) | Resource-threaded primitives in `dsl/std/resources.dag`, `dsl/std/primitives.dag` | `dsl/std/*.dag` |
| **Grounding Manager** (T-Ground-Services lineage per `services.dag` header) | Per-service `Operation` row population for `dsl/extdeps/*.dag` (the PR-β..ω lineage already named in `services.dag:9-14`) | `dsl/extdeps/**/*.dag` |
| **Verification Manager** | Closure gate `effect_enumeration_lens_behaviorally_complete` cementing test | `src/v3/compiler/tests/integration/cementing/` |

The three managers ship one coordinated PR per the [`feedback_bundle_workstreams_per_pr`](../README.md#) discipline. The cross-manager touch is what makes this slice L-XL in [`docs/r3-structure.md`](r3-structure.md) row 146.

## §6. Atomic-migration shape (no bridge)

Per [`../INVARIANTS.md`](../INVARIANTS.md) §P5 *No Short-Term Solutions* and *No Bridges* + [`feedback_construction_over_ratchets`](../README.md#) — the migration lands as a single change. Per [`docs/design-composed-effect-reshape.md`](design-composed-effect-reshape.md) R3 (the precedent for this shape): "the brief explicitly does not pre-declare the handle ... When Stage 2b lands, if the report needs to render 'position of the breaker in the workflow', that is the moment to promote." Same principle: no `OperationEffect`-staging, no resource-block-deprecation marker, no parallel `transport-block-or-Arrow-body` window.

### §6.1 What atomic means here

In one PR:
1. Resource-threaded signatures land in `dsl/std/resources.dag` (4 resources × ~3 capabilities each = ~12 signature reshapes).
2. Every `dsl/extdeps/*.dag` operation migrates from `transport <kind>` block to typed signature + `Arrow.body` E-9 realization.
3. Every `uses x: Resource` clause across 20 call-sites migrates to threaded input/output.
4. `OperationEffect` deletes; `compose_effects(List<Operation>) -> CompositionVerdict` ships.
5. `WorkflowEffect.LinearEffect.ops` element type migrates from `OperationEffect` to `Operation`.
6. `effect_enumeration.dag` status row flips from PARTIAL to COMPLETE; cementing test lands in same PR.

### §6.2 Why atomic is feasible (not aspirational)

Three structural reasons make this PR-shippable rather than requiring a multi-stage migration:

- **The lens body changes only in its kind-classification dispatch.** `src/v3/lenses/effect_enumeration.dag::callable_arrow_effect` today derives kind from signature/body shape — that path retires. The new lens body reads the **effect set** directly from the callable's arrow signature (the existing substrate query — resource types in input ∩ output; structurally derivable per §2.4(a) without any lens-application-surface artifact) and dispatches on `callable_inhabits(callable, idempotent_read_for(resource))` / `callable_inhabits(callable, mutating_for(resource))` for the **kind classification** (per §2.4(b)). Both queries are within T-Lens-Behavioral-Parity slice 4 scope; **neither depends on `LensEnforcement` from T-Lens-Application-Surface** (the cascade flows the other direction — lens-behavioral-parity COMPLETE → lens-application-surface). The fold structure (per-callable walk; aggregation into the report carrier) is unchanged; only the per-callable kind classifier changes from signature-shape inference to inhabitance lookup. **This change is in scope and required** — without it, the new inhabitance facts §2.4 declares would not be consumed (facts-flow-forward / P2 violation per cursor BLOCKING at sha 96899484 line 17).
- **`Operation` already exists.** `src/v3/std/services.dag:122` declares the carrier with `callable: CallableRef + inputs: Map + endpoint: RestEndpointBinding` — exactly what §3.2 needs. No new substrate type. The PR-β..ω lineage in `services.dag:9-14` is the dispatch frame for the per-extdep population work.
- **`Arrow.body` E-9 binding is landed (DB-14).** The transport block migration target is already a live substrate facility per [`../INVARIANTS.md`](../INVARIANTS.md) §E-9 and DB-14. Moving `transport shell { argv: [...] }` to per-target binding is mechanical, not novel substrate work.

### §6.3 The wrong shape — what a bridge would look like

For [`feedback_no_short_term_solutions`](../README.md#) cross-reference: any of the following is **rejected** by this design and reviewers should KEEP_ITERATING on a PR that proposes them.

- A `legacy_effect: Option<OperationEffect>` field on `Operation`. (Bridge as steady state — admits both old and new authority.)
- A `derive_op_effect_v2` next to `derive_op_effect` while consumers migrate one at a time. (Parallel implementation debt.)
- A `#[deprecated]` marker on `OperationEffect` while the field stays in the schema. (Per §P5: "no deprecations — deprecation markers are a production-code tool, not a legitimate steady state.")
- An "ambient mode" flag on `resource Filesystem { ... }` to keep both shapes addressable during migration. (Per [`feedback_state_space_vs_behavioral_invariants`](../README.md#): admits illegal coexistence states.)
- A `transport_compat_block` keyword that shadows `transport shell` for back-compat. (Channel-B leak preserved with a different name.)

The bug pattern these all share: the migration becomes the new steady state because each consumer can defer cutover. The atomic shape forces every consumer to migrate simultaneously, which is the only structurally-honest path under §P5.

## §7. Cascade gates

Per [`docs/r3-structure.md`](r3-structure.md) row 146:

- **Internal cascade:** none. Slice 4 is parallel-dispatchable with slices 1-3 (complexity / cost / parallelism) once the producer foundation lands. Each slice is independent of the others' substrate work.
- **External cascade:** T-E-P-Producer-Broadening + R2-Evaluator landed (the standard R3 worker-dispatch precondition). T-E-P-Producer-Broadening is required because the lens fold reads per-call descent evidence in adjacent slices; effect_enumeration's fold itself does not consume per-call evidence, but the cementing test for COMPLETE status requires the broader producer surface so cross-slice composition is testable.

Pre-cascade *design-doc* work is permitted (this doc); pre-cascade *substrate work* (the §6.1 PR) waits for the external cascade to clear.

## §8. Resolved design questions

Per [`feedback_design_before_implement`](../README.md#) — resolve all design questions before implementation; audit code in design phase.

### §8.1 Read/write distinction substrate authority — RESOLVED: algebra inhabitance, not signature shape

**Question:** Cursor BLOCKING finding 2026-05-02 (PR #1480 sha ef21e1a0): `read(fs: R, path) → (fs: R, content)` and `write(fs: R, ...) → (fs: R, ...)` have *the same typed signature* unless value-preservation is itself a substrate fact. The signature alone cannot distinguish read from write — `ReadShaped` vs `WriteShaped` would remain convention-level. Where does the structural distinction live?

**Resolved:** the read/write distinction is declared via **algebra inhabitance on the callable**, not derived from signature shape.

- The signature carries the **effect set** (which resources are involved) — structurally, from input ∩ output.
- Each callable declares its **kind** per resource via algebra inhabitance: `inhabits IdempotentRead<R>` (read), `inhabits Mutating<R>` (write), `inhabits Append<R>` (append).
- The lens reads inhabitance for kind classification; absence of any kind inhabitance with the resource in the effect set is a fail-closed Diagnostic (`EffectKindUndeclared`), not a silent default.

This preserves the existing `EffectShape = IsIdempotent(IdempotentShape) | IsBreaking(BreakingShape)` partition from [`docs/design-composed-effect-reshape.md`](design-composed-effect-reshape.md) (PR #529 R3) — the partition continues to classify effects, but its source is the declared inhabitance carrier rather than `derive_effect_shape(method, path)` walking ambient metadata.

**Why algebra inhabitance, not a field on `Operation`**: keeping the kind on the callable's *algebra* (rather than on each per-call `Operation`) preserves single-authority — every call site of the same callable has the same kind, and the kind is a property of *what the callable does* (read or write), not *how it's invoked at this site*. This is the same pattern as `Semiring`/`BoundedLattice` inhabitance on type declarations.

**Why algebra inhabitance, not a phantom marker on the resource type** (`Filesystem<Read>` vs `Filesystem<Write>`): a single `Filesystem` value is used in both read and write contexts (a typical use case). Phantom-marking the resource would force a cast at every read↔write boundary, which is annotation-style noise per `feedback_no_annotations`.

**Implementation note:** every effectful primitive in `dsl/std/primitives.dag` and `dsl/std/resources.dag` declares its inhabitance explicitly. The migration is mechanical: each existing capability declaration (currently named `read` or `write`) gains an explicit `inhabits IdempotentRead<R>` or `inhabits Mutating<R>` clause. The lens consumer dispatches on the inhabitance lookup; no signature-shape parsing for kind classification.

**Cross-link**: see §2.1 (where the cursor finding was anchored) and §2.4 (the unified rule split into effect-set-from-signature + effect-kind-from-inhabitance).

### §8.2 Acquisition primitives: how does `Filesystem` enter scope? — RESOLVED: typed acquisition primitive

**Question:** Today, `acquire {}` blocks on `resource` declarations are the entry point for resource handles. After migration, where do `Filesystem` values come from?

**Resolved:** typed acquisition primitives that thread the *capability environment* the program is invoked under. The program's top-level entry point accepts `(Process, Filesystem, Network, Clock, AuthContext)` (or some subset) as inputs; downstream callers thread these forward. Per [`docs/design-db18-workflow-effect-carrier.md`](design-db18-workflow-effect-carrier.md) discipline: "the authority site is the node it describes" — the entry point is the authority for capability acquisition; nothing else mints `Filesystem` values.

**Implementation note:** the existing `ResourceHandle { type, resource_id, key, cap: Secret }` carrier in `src/v3/std/resources.dag:18` is the substrate basis; `cap: Secret` is the per-process opacity authority (already aligned with DB-18's `BoolPortRef` Track-9 typed-handle pattern). Acquisition primitives produce these handles structurally; user code cannot construct one.

### §8.3 `transport shell { argv }` — does it move to `Arrow.body` or disappear? — RESOLVED: `Arrow.body` E-9 binding

**Question:** When `service shell.Find { operation ListDirs { transport shell { argv: [...] } } }` migrates, does the `argv` template become a per-target realization on the callable's `Arrow.body`, or is it folded into the signature somehow?

**Resolved:** moves to `Arrow.body` per [`../INVARIANTS.md`](../INVARIANTS.md) §E-9 + DB-14. The signature stays target-language-agnostic (`(Process, FilePath, Int, Int) → (Process, List<FilePath>)`); the per-target binding for the Rust emitter is `argv: ["find", ...]` realized as a `std::process::Command` invocation; the per-target binding for the Python emitter is the same logical shape rendered as `subprocess.run(...)`; etc.

**Implementation note:** this is exactly the lowering pattern DB-14 lands; transport-block migration is a *consumer* of an existing substrate facility.

### §8.4 `readonly` keyword — does it survive? — RESOLVED: deleted

**Question:** `dsl/extdeps/shell.dag` operations carry a `readonly` keyword on operations. Does that survive migration?

**Resolved:** deleted. Per [`feedback_closed_system_effects`](../README.md#): *"effects are not annotations."* `readonly` is exactly the annotation pattern P1 names. After migration, **read kind is declared via algebra inhabitance** (`inhabits IdempotentRead<R>` per §2.4 + §8.1) — a structural fact on the callable declaration. The `readonly` keyword duplicates that inhabitance: same fact, two carriers. Per P2 single-authority and `feedback_no_annotations`, the annotation goes; the inhabitance stays. The keyword's only role today is short-circuiting the algebra ahead of the (heuristic) `derive_op_effect`; once the algebra reads the declared `inhabits IdempotentRead<R>` directly, the keyword is redundant *and* structurally drift-prone (`feedback_state_space_vs_behavioral_invariants`: a behavioral invariant maintained by convention rather than by the type — the keyword could disagree with the inhabitance).

**Implementation note:** existing `is_idempotent_effect` / `operation_is_breaking` continue to work — they read the partition off the `EffectShape` derived from algebra inhabitance (`IdempotentRead<R>` → `IsIdempotent(ReadShape)`), not off a keyword. The `readonly` deletion is mechanical: every operation currently using `readonly` already has a corresponding `inhabits IdempotentRead<R>` declared at the migration site (one-to-one mapping during the atomic migration PR per §6).

### §8.5 `OperationEffect.evidence: IdempotencyEvidence` — does it follow `OperationEffect` to deletion? — RESOLVED: yes, derived projection

**Question:** v2 `dsl/std/effects.dag::OperationEffect` carries an `evidence: IdempotencyEvidence` field; v3 already projects on demand via `derive_idempotency_evidence`. Does `IdempotencyEvidence` survive the migration?

**Resolved:** `IdempotencyEvidence` survives as a projection function (the type stays; the field on `OperationEffect` was already removed in v3). After `OperationEffect` deletes, `derive_idempotency_evidence(shape: EffectShape) -> IdempotencyEvidence` becomes `derive_idempotency_evidence(op: Operation) -> IdempotencyEvidence` (computes shape from `op.callable`'s signature, then projects). Per [`docs/design-composed-effect-reshape.md`](design-composed-effect-reshape.md) lessons R1→R3: every projection is a derivation from the single authority; the projection function survives, the parallel-record carrier dissolves.

### §8.6 Coexistence with `WorkflowEffect` — does this design touch DB-18's lock? — RESOLVED: no

**Question:** [`docs/design-db18-workflow-effect-carrier.md`](design-db18-workflow-effect-carrier.md) Acceptance item 1 locks `WorkflowEffect`'s 4-variant shape. Does `LinearEffect.ops: List<OperationEffect> → List<Operation>` regress that lock?

**Resolved:** no. DB-18 Acceptance item 1 locks the **set of variants** (four) and the **mandatory fields of each variant**; additive-extension fields are explicitly in scope for future stages. The element type of `LinearEffect.ops` is a *type refinement of one mandatory field* (the inner element type), not a variant change. Per DB-18's STOP-AND-ESCALATE rules: variant addition or shape regression escalates; element-type refinement is in-scope additive evolution. This design changes `List<OperationEffect>` to `List<Operation>` — a single carrier type tightening — and does not add or remove variants.

## §9. Relationship to existing authority

This design extends:

- [`docs/design-db18-workflow-effect-carrier.md`](design-db18-workflow-effect-carrier.md) — the `WorkflowEffect` substrate input carrier. **No variant changes**; this design refines `LinearEffect.ops` element type per §8.6.
- [`docs/design-composed-effect-reshape.md`](design-composed-effect-reshape.md) — the `CompositionVerdict` output carrier. **No verdict-shape changes**; `BrokenBy.first_breaker` retypes from `ElementRef<OperationEffect>` to `ElementRef<Operation>` per §4.1.
- [`../INVARIANTS.md`](../INVARIANTS.md) §P1 — load-bearing for §1 audit (heuristics indicate lost structure; the three ambient channels are heuristic compressions of the missing signature shape).
- [`../INVARIANTS.md`](../INVARIANTS.md) §C-8 — load-bearing for §6 atomic shape (no fabricated "in-flight" state where both old and new authorities coexist).
- [`../INVARIANTS.md`](../INVARIANTS.md) §P5 *No Short-Term Solutions* / *No Bridges* — load-bearing for §6.3 (rejected bridge shapes).
- [`../INVARIANTS.md`](../INVARIANTS.md) §E-9 *External Realization Lives On Arrow.body* — load-bearing for §8.3 (transport-block migration target).
- [`feedback_closed_system_effects`](../README.md#) — load-bearing for §2 (resource-threading discipline) and §8.4 (`readonly` deletion).
- [`feedback_construction_over_ratchets`](../README.md#) — load-bearing for §6 (model first; the heuristic dissolves naturally; no analyzer-local patching).
- [`feedback_lenses_not_passes`](../README.md#) — load-bearing for §1 audit (every "look up the resource block" consumer is evidence of a missing physics).
- [`feedback_parallel_representation_debt`](../README.md#) — load-bearing for §4.2 (the canonical source `Operation` exists; consume it, don't scaffold).

This document does NOT modify:

- The `WorkflowEffect` 4-variant shape (DB-18 lock; per §8.6).
- The `CompositionVerdict` 2-variant shape (`ComposedEffect` reshape R3 lock; per §4.4).
- The `EffectShape` partition (`IsIdempotent | IsBreaking`; per §4.3).
- The `BoundedLattice<EffectShape>` algebra (per `src/v3/std/effects.dag` invariants).
- The lens fold structure (per-callable walk; report aggregation). The per-callable kind classifier IS modified — it changes from signature-shape inference to inhabitance lookup (per §6.2 first reason; required for facts-flow-forward of the new inhabitance authority).

## §10. Implementation order (sketch)

Within T-Lens-Behavioral-Parity slice 4 (per [`docs/r3-structure.md`](r3-structure.md) row 146 closure gate `effect_enumeration_lens_behaviorally_complete`):

1. **Resource-threaded primitive landing** (Substrate Manager). Reshape `dsl/std/resources.dag` `Filesystem` / `Network` / `Clock` / `AuthContext` capabilities to threaded signatures per §2.1. Resource blocks delete; capability functions become free-standing arrow declarations.
2. **`uses` clause migration** (Substrate Manager). Migrate 20 `uses x: Resource` call-sites to threaded inputs/outputs per §2.3. Same PR as step 1 (atomic per §6).
3. **Transport-block migration to E-9 binding** (Grounding Manager). Migrate 27 services × 67 operations across `dsl/extdeps/*.dag` from `transport <kind> { ... }` blocks to `Arrow.body` per-target realizations per §8.3. Same PR.
4. **`OperationEffect` retirement** (Substrate Manager). Delete `OperationEffect` declaration + `derive_op_effect` + Rust mirror; reshape `compose_effects(List<Operation>) → CompositionVerdict`; reshape `WorkflowEffect.LinearEffect.ops: List<Operation>`. Same PR.
5. **Lens kind-classifier rewrite** (Substrate Manager). Rewrite `src/v3/lenses/effect_enumeration.dag::callable_arrow_effect`'s per-callable kind dispatch from signature/body shape inference to algebra-inhabitance lookup (`callable_inhabits(callable, idempotent_read_for(resource))` / `callable_inhabits(callable, mutating_for(resource))`). The fold structure (per-callable walk + report aggregation) is unchanged; only the per-callable kind classifier changes. Required for the new `inhabits IdempotentRead<R>` / `inhabits Mutating<R>` facts to flow forward into lens output. Same PR.
6. **Cementing test** (Verification Manager). Author `effect_enumeration_lens_behaviorally_complete` cementing test under `src/v3/compiler/tests/integration/cementing/` per [`TESTING.md`](../TESTING.md) Band-C discipline. Tests minimal-`Dag` shapes asserting (a) read-shaped operations report `ReadShaped`, (b) write-shaped operations report `WriteShaped`, (c) operations with no resource thread report `NoEffect`, (d) coverage gaps surface explicitly. Same PR. **Cross-lens cementing format alignment**: this Rust cementing test is the staged form. Dissolution trigger: at T-Tests-As-Data-Completeness step 5 (per [`docs/design-tests-as-data-completeness.md`](design-tests-as-data-completeness.md) §6 step 5 — *cementing dispatch port*), the test ports to a `.dag` `TestClaim`/`QuantifiedTestClaim` declaration. All three behavioral-parity lenses (complexity / cost / effect-enumeration) follow the same staging — Rust cementing today, .dag port at migration step.
7. **Capability-register row update** (Verification Manager). Flip `docs/v3-lens-capability-register.md` row for `effect_enumeration.dag` from `BEHAVIORALLY PARTIAL` to `BEHAVIORALLY COMPLETE`; clear the "What v2 has that v3 drops" cell per the discipline at register §"Discipline" item 2. Same PR.

Steps 1–7 land as **one PR** per §6 atomic discipline. Internal sequencing within the PR is mechanical — substrate authoring (1) → consumer migration (2, 3) → algebra reshape (4) → lens kind-classifier rewrite (5) → tests (6) → docs (7) — but the PR is a single atomic unit per [`../INVARIANTS.md`](../INVARIANTS.md) §P5.

Total estimate (per L-XL sizing in lane row 146): substrate + extdeps consumer migration is the L-XL surface; lens-fold structure is unchanged; per-callable kind classifier rewrite (signature-shape → inhabitance lookup) is S; cementing test is M; doc is S. End-to-end: 3-5 weeks worker time at standard R3 cadence given coordinated cross-manager dispatch.

---

**This document is a design spec, not a ship target.** It resolves the structural design questions blocking T-Lens-Behavioral-Parity slice 4 substrate dispatch. The slice itself runs once cascade gates clear (T-E-P-Producer-Broadening + R2-Evaluator landed). All §8 design questions resolved in-doc; no Director ratification required before substrate authoring begins on the resource-threading migration.
