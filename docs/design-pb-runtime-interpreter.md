# design-pb-runtime-interpreter.md

**Director-authored design lock — Items 4 + 5 (Tier 1 escalation, 2026-04-29).**

Resolves the fourth and fifth open questions in the pre-spawn Tier 1 escalation per PM coordination 2026-04-29T00:30:52Z + 2026-04-29T00:36:47Z (Director ↔ PM). Items 4 (PB-Runtime interpreter-as-data shape) + 5 (PB-1 generated bin-shim emit pattern) ship in one doc per PM directive ("Items 4+5 — co-design pattern is right; interpreter-as-data is the substrate `fold_lens<C>` consumes").

Authored as a **standalone doc** (week-scale, per PM cadence directive) rather than a fold into existing PB design docs because the surface spans both the R2-Evaluator runtime model (Items 4) and the PB-1 lane scope (Item 5) — two cross-program concerns that benefit from a single design-lock authority.

**Co-design contract:** this lock co-designs with R2 PB Manager (`docs/briefs/r2-pure-bootstrap-manager.md`) + R2 Evaluator Manager (`docs/briefs/r2-evaluator-manager.md`). Neither manager is the sole authority on Items 4+5 because PB-Runtime spans both runtime values (Evaluator's territory) and the bootstrap-as-data shape (PB's territory). This doc is the seam.

---

## 1. Scope

This doc locks two coupled facts:

- **Item 4 — PB-Runtime interpreter-as-data shape.** What the runtime that executes `.dag` bodies *is* when expressed as a `.dag` program, not a Rust crate. The structural shape that `fold_lens<C>` consumes; the substrate the Evaluator lowers to as the closed-system thesis matures.
- **Item 5 — PB-1 generated bin-shim emit pattern.** How the binary entrypoints (`regen_lens.rs`, `regen_v3.rs`, etc.) emit from `.dag` declarations rather than being hand-Rust scripts. The generalization of the pattern that retires the last hand-Rust file class on the SG-0 = 0 path.

Both items together gate **R3-T-LensProducer-Retirement** sub-gate 3 (`regen_lens_dot_rs_retired`). Sub-gates 1 + 2 (`lens_apply_dot_rs_retired` + `lens_testgen_dot_rs_retired`) are gated on Item 4 only.

Scope intentionally excludes:

- The first-time-bootstrap N=0 resolution choice (`docs/design-pure-bootstrap-zero.md` §"First-time bootstrap" lists three; pick is downstream of ecosystem-strategy taste).
- The Cargo trampoline shape (out-of-tree shim per `design-pure-bootstrap-zero.md` resolution; not v3's source-tree concern).
- R2-Evaluator's runtime-value internals (Rust-side closed-over environments, lazy/eager strategy, memoization) — those land per `docs/briefs/r2-evaluator-manager.md` PR-A through PR-E. Items 4+5 specify the *target shape* the Evaluator lowers to, not the Rust-side runtime mechanics during R2.

## 2. Relationship to R2-Evaluator (the load-bearing distinction)

The most consequential framing — answering the question both managers will ask first.

**PB-Runtime ≡ R2-Evaluator's runtime model expressed as a `.dag` program.** They are not parallel runtimes. They are the same runtime under two different presentations:

| Presentation | Form | Where it lives | When it lands |
|---|---|---|---|
| **R2-Evaluator (Rust)** | Rust crate that executes `.dag` function bodies, manages closed-over environments, constructs witnesses | `src/v3/compiler/src/` (Evaluator Manager's deliverables) | R2 phase, per `docs/briefs/r2-evaluator-manager.md` PR-A through PR-E |
| **PB-Runtime (`.dag`)** | `.dag` program that *describes* the same execution semantics, structurally | `dsl/std/runtime/` (or equivalent — name TBD per PB Manager dispatch) | R3 phase, after R2-Evaluator's Rust crate stabilizes; bootstraps from R2-Evaluator until the closed-system loop fires |

The relationship is dissolution-shaped, not parallel-representation:

1. **R2 (now):** R2-Evaluator's Rust crate is the runtime authority. It compiles + executes `.dag` bodies including the `.dag` program that *describes* the runtime.
2. **R3 dissolution:** the `.dag` runtime description (PB-Runtime) reaches feature-parity with the Rust crate. R2-Evaluator's Rust mirror retires once compiled-PB-Runtime + emitted-stage0-Rust round-trips bit-identically.
3. **Closed-system terminal:** the runtime is bootstrapped from `.dag` exclusively. `gunbc-runtime` (the universal runtime crate per `docs/design-pure-bootstrap-zero.md` §"First-time bootstrap" resolution 2) is the only Rust outside v3's source tree.

This is the same dissolution shape as `analyze_symbolic_cost_dimension` (Rust scaffold; dissolves into `.dag` lens once Evaluator can run the relevant std bodies, per `docs/design-dimension-abstraction.md`) but generalized to the runtime itself.

**Anti-bridge:** PB-Runtime MUST NOT diverge from R2-Evaluator's semantics. If PB-Runtime declares a behavior R2-Evaluator's Rust doesn't implement, that's a structural error; if R2-Evaluator implements a behavior PB-Runtime can't express, that's a structural error. Convergence is non-optional; the two presentations of the same runtime stay structurally identical by construction (the `.dag` declaration of PB-Runtime IS the spec the Rust crate's tests verify against).

## 3. Item 4 — PB-Runtime interpreter-as-data

### 3.1 The 5-primitive constraint

Per `feedback_compiler_is_dag_processor.md`: the compiler knows ONLY `Node` / `Conj` / `Disj` / `Cardinality` / `Bit`. The interpreter therefore operates over only those 5 primitives. PB-Runtime's `.dag` program declares execution semantics for each:

| Primitive | Execution semantic | Substrate authority |
|---|---|---|
| `Node` | Look up by `NodeId`; dispatch on `Behavior` variant (Value / Transform / Branch / Loop / Bind) | `src/v3/std/substrate.dag` `Behavior` |
| `Conj` | Materialize record fields; project named field on access | `src/v3/std/substrate.dag` `TypeConnective::Conj` |
| `Disj` | Materialize variant; dispatch on tag at branch | `src/v3/std/substrate.dag` `TypeConnective::Disj` |
| `Cardinality` | Bound iteration count or descent witness; loop terminates structurally | `src/v3/std/substrate.dag` `TypeConnective::Cardinality` + `LoopBound` |
| `Bit` | Atomic value; literal bits | `src/v3/std/substrate.dag` `LiteralBits` |

**Anything beyond these 5 primitives is a downstream lens, not the interpreter.** Cost analysis is a lens. Termination is a lens (descent evidence). Parallelism is a lens. Effect inference is a lens. None of these are part of PB-Runtime; they fold *over* PB-Runtime's outputs.

**Mapping note — "5 execution primitives" ↔ "6 type connectives + 5 L1 behaviors" (per `r2-structure.md` §6 Evaluator brief).** These are different vocabularies at different scopes; not a fork:

- The **5 dispatch primitives** above (`Node` / `Conj` / `Disj` / `Cardinality` / `Bit`) are the **DAG-processor's execution vocabulary** — what the interpreter dispatches on at evaluate-step level. `Node` is the carrier of behavior (each `Behavior` variant lives inside a `Node`); the other four are type-connective shapes encountered during evaluation.
- The **6 type connectives** in `src/v3/std/substrate.dag` (`Atom` / `Conj` / `Disj` / `Arrow` / `Cardinality` / `Instantiation`) are the **substrate type system's expressivity** — what `.dag` type declarations can describe. R2-Evaluator's runtime-value model authors typed runtime values for each of these six connectives' inhabitants.
- The **5 L1 behaviors** (`Value` / `Transform` / `Branch` / `Loop` / `Bind`) are **identical in both vocabularies** — they're the `Behavior` variants every interpreter step dispatches on, regardless of which presentation scope you're in.

Concretely, the `Value` coproduct in §3.2 below represents the inhabitants of all 6 type connectives that PB-Runtime needs to carry at evaluation time: `LiteralValue` (Bit + Atom inhabitants via `LiteralBits`), `RecordValue` (Conj inhabitants), `VariantValue` (Disj inhabitants), `NodeRef` (carrier of `Behavior`-bearing nodes — including `Bind` nodes whose binding constructs Arrow values structurally), `CardinalityValue` (Cardinality inhabitants). `Arrow` and `Instantiation` aren't standalone Value variants because Arrow values are structurally represented via `Bind` nodes (closures = bound bodies the runtime navigates via `NodeRef`), and `Instantiation` is a type-level phenomenon that erases at runtime (the runtime carries the instantiated value, not a parametric witness). This matches the Evaluator's PR-A scope without forking the vocabulary.

Anti-bridge invariant #6 (§6 below) reaffirms this: PB-Runtime's `Value` and R2-Evaluator's runtime-value model share a structural definition; the apparent "5 vs 6" numbering reflects different scopes (dispatch vs type-system expressivity), not a fork.

### 3.2 What PB-Runtime IS (concrete shape)

A `.dag` program declared at `dsl/std/runtime/runtime.dag` (path TBD; PB Manager picks per their dispatch) exposing one entry point:

```
fn evaluate(program: Dag, entry: NodeId, args: List<Value>) -> Value
```

Where `Value` is the runtime-value type (closed coproduct over the 5 primitives' inhabitants, declared in `dsl/std/runtime/value.dag`):

```
type Value
  = LiteralValue(LiteralBits)                          // Bit
  | RecordValue(List<NamedField>)                      // Conj
  | VariantValue { tag: DeclarationId, payload: Value } // Disj
  | NodeRef(NodeId)                                    // Node (carries identity, not inlined sub-tree)
  | CardinalityValue(LoopBound)                        // Cardinality (already a substrate type)
```

The implementation of `evaluate` is itself a `.dag` program — a fold over `Behavior` variants per the Evaluator's PR-B runtime-value model. Each `Behavior` variant has a structural-evaluation rule:

- `Behavior::Value(v)` → emit `LiteralValue(v.payload)`
- `Behavior::Transform(t)` → recursive evaluate of inputs; apply `t.target` (callable / field-project / operator) to result
- `Behavior::Branch(b)` → evaluate `b.input`; pattern-match against `b.paths`; recursive evaluate selected `path.body` with `path.binding` bound
- `Behavior::Loop(l)` → fold over `l.bound` (cardinality bounded iteration or descent-bounded recursion); evaluate `l.body` per iteration with accumulator
- `Behavior::Bind(b)` → bind `b.params` to provided argument values in a fresh evaluation frame (see §3.3 below); evaluate `b.body` in that frame; the frame goes out of scope when the call returns.

These five rules ARE the runtime. The `.dag` declaration of `evaluate`'s body IS the spec.

### 3.3 What is NOT a `Value` (runtime evaluator state vs observable result)

`Value` (§3.2) is the **observable-result domain** — what `evaluate(...)` returns and what flows through computation as data. The evaluator additionally carries **internal evaluation-state carriers** that are NOT `Value` variants:

- **`EvalFrame`** — a stack-discipline carrier mapping `PortId` → bound `Value` for the duration of a `Bind` body's evaluation (or a `Loop` iteration's body evaluation). Owned by R2-Evaluator's PR-A; declared in PB-Runtime's `.dag` form alongside `evaluate` but lives in evaluator-state space, not `Value` space.
- **`EvalStateStack`** — the stack of `EvalFrame`s representing nested binding scopes during evaluation. Same scope-shape: evaluator-internal, not a `Value` inhabitant.

These exist because the Evaluator brief's "closed-over environments + binding scopes for `Loop` / `Bind`" language IS load-bearing — but the load it bears is at the evaluator's runtime-state layer, not at the `Value` layer that flows between callers. The `EvalFrame`/`EvalStateStack` carriers ARE structural (declarable in `.dag`, consumed by PB-Runtime); they're just NOT inhabitants of `Value`.

**Why closures-with-captures are NOT first-class `Value` variants in v3:** the substrate doesn't admit them by construction. `TransformTarget = Callable(DeclarationId) | FieldProject | Operator(OperatorKind)` — `Callable` resolves to a *top-level declaration*, not a captured-state-bearing closure. `Behavior::Bind.params: List<PortId>` is parameter-binding at call time, with no provision for capturing local state at definition time. A `Value` carrying a `NodeRef(NodeId)` to a Bind node is *identity*, not a closure — call sites bind params at call time; there's no captured-state slot.

Concretely:
- A function returned from another function (`f(x) → λy. y + x`) is NOT expressible in v3 today. Functions are top-level (`Callable(DeclarationId)`); the only "function value" is a structural reference.
- `EvalFrame` data does NOT escape its `Bind` evaluation: when a `Bind` returns, its frame pops; nothing in `Value` carries a reference to it.
- "Closed-over environment" in the Evaluator brief = "the active frame stack during evaluation provides binding lookups." It does not mean "values in transit can carry environments."

**If v3 evolves to need first-class closures-with-captures** (e.g., higher-order programming as a deliberate language extension), that's a substrate-fact-introduction event: a new `EvaluatorClosure { node_ref: NodeId, captured_frame: EvalFrame }` carrier under §P1 escalation to Substrate Manager (per anti-bridge invariant #2 below). The carrier could be a new `Value` variant *or* a separate observable-result type — the disposition is downstream of the actual feature ask. **Until that happens, R2-Evaluator's PR-A worker MUST NOT add a closure/captured-environment value variant.** Worker A's "closed-over environments" implementation = `EvalFrame` + `EvalStateStack` (evaluator-internal); NOT a Value inhabitant.

This resolves the apparent "implement closures vs do not add closure/runtime-env value" ambiguity: implement the evaluator's frame-stack discipline as `EvalFrame`/`EvalStateStack` carriers, not as Value variants. The Value coproduct stays closed at the 5 primitives' observable inhabitants; the evaluator's runtime state lives in parallel, structurally typed.

### 3.4 What this is NOT

- **Not** a separate language. PB-Runtime is `.dag` — same surface as everything else.
- **Not** a parallel value system. `Value` reuses substrate carriers (`LiteralBits`, `LoopBound`); only the runtime-value coproduct is new.
- **Not** "an interpreter in `.dag` to bootstrap an interpreter in Rust." That's circular. The dissolution path runs the OTHER direction: R2-Evaluator (Rust) is the bootstrap; PB-Runtime (`.dag`) is the terminal form.
- **Not** the same as `Lens<C>.read`. Lenses fold over already-evaluated programs (or fold structurally without execution per `docs/design-reflection-completeness.md`); PB-Runtime is the layer that turns a `.dag` program into a runtime value.

### 3.5 Reflection vs evaluation distinction (cross-reference)

Per `docs/design-reflection-completeness.md` §3 + §6: reflection is *static structural projection*; evaluation is *running the reflected program*. PB-Runtime is the evaluation half:

- **Reflection** (per `design-reflection-completeness.md`): `Dag` → `FieldValue` (lens-input shape; structural facts; no execution)
- **Evaluation** (this doc, Item 4): `Dag` + entry + args → `Value` (runtime result; execution-driven)

Both run on the same substrate carriers. They produce different things. PB-Runtime is the substrate that makes "running a `.dag` program" structurally expressible, just as reflection is the substrate that makes "analyzing a `.dag` program" structurally expressible.

`fold_lens<C>` consumes the *reflected* `FieldValue` (static); the Evaluator/PB-Runtime is what executes the underlying `.dag` body when a lens needs a runtime witness (e.g., for L4 emit/eval match in `docs/r3-structure.md` T-Verification-L4-L7-Direct).

## 4. Item 5 — PB-1 generated bin-shim emit pattern

### 4.1 The bin-shim class

Hand-Rust binary entrypoints currently in `src/v3/compiler/src/bin/`:

```
r1c_e_emit_gates.rs
regen_bootstrap.rs
regen_lens.rs
regen_parse.rs
regen_parse_tables.rs
regen_tokenize.rs
regen_v3.rs
self_host_fixed_point.rs
```

Each is a thin host shim — typically <200 lines — that:
1. Constructs a `Dag` (loads bootstrap or compiles a registry-declared `.dag` file)
2. Runs a compiler pipeline phase (compile / tokenize / parse / regen)
3. Writes output to a registry-declared path

The dissolution observation: **none of the actual logic in these shims is Rust-specific.** Each is a 4-step pipeline of (load → compile → project → write) that maps cleanly to a `.dag` program with `ExecuteCommand` or `WriteFile` substrate primitives.

### 4.2 The emit pattern

A bin-shim is a `.dag` declaration of the form:

```
data regen_lens_shim: BinShim = {
  entrypoint_name: "regen_lens"
  description: "Unified lens-regen driver. Reads LensRegistryEntry records and regenerates each lens module."
  pipeline: [
    LoadDag { source: BootstrapDag },
    CompileLensRegistry { registry_decl: regen_dag_lens_registry },
    EmitRustModules { output_dir: workspace_root_relative("src/v3/compiler/src/") },
  ]
  exit_code_on_success: 0
  exit_code_on_failure: 1
}
```

Where `BinShim` is a substrate-declared carrier (lives in `dsl/std/runtime/bin_shim.dag`):

```
type BinShim {
  entrypoint_name: NonEmptyStr           // becomes binary name in Cargo.toml [[bin]] target
  description: String                     // populates the Rust file's doc comment
  pipeline: NonEmptyList<PipelineStep>    // sequential steps; each is a `.dag` operation
  exit_code_on_success: Int
  exit_code_on_failure: Int
}

type PipelineStep
  = LoadDag { source: DagSource }
  | CompileLensRegistry { registry_decl: DeclarationRef }
  | EmitRustModules { output_dir: WorkspacePath }
  | RunTestSuite { fixture_dag: WorkspacePath }
  | // ... (other pipeline-step variants per lane scope)
```

The emitter for `BinShim` declarations is a `.dag` program (analogous to existing emit modules per `dsl/extdeps/languages/rust/emit.dag`) that produces a Rust file shaped like:

```rust
// AUTO-GENERATED from dsl/std/runtime/bin_shims/regen_lens.dag — DO NOT EDIT.
//
// {description}

use gunbc_runtime::{Dag, Pipeline};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::from({exit_code_on_success}),
        Err(message) => {
            eprintln!("{message}");
            ExitCode::from({exit_code_on_failure})
        }
    }
}

fn run() -> Result<(), String> {
    let dag = Dag::new();
    Pipeline::new()
        // ... (one .step() per pipeline element, with substrate-declared step bodies)
        .execute(&dag)
}
```

The emitted Rust is standardized — the only per-shim variation is the pipeline composition + entry-point name. The Rust shape itself is one template.

### 4.3 Dissolution path for the existing bin-shims

For each of the 8 existing bins:

1. **Author the `.dag` declaration** — one `data <name>_shim: BinShim = { ... }` per bin, in `dsl/std/runtime/bin_shims/`. Today's hand-Rust becomes the structural source; the per-shim `.dag` declaration is a pure data witness of the existing pipeline.
2. **Verify equivalence** — emitted Rust from the `.dag` declaration produces bit-identical output to the existing hand-Rust shim when fed identical inputs. T-FixedPoint discipline (one of T-LensProducer-Retirement's R3 sub-gates).
3. **Retire the hand-Rust shim** — delete the file; Cargo.toml `[[bin]]` target now points at the emitted-from-`.dag` shim. Each retirement is one PR + one closure-ledger row movement.

`regen_lens.rs` is the canonical first slice for sub-gate 3 (`regen_lens_dot_rs_retired`). The other 7 follow under T-LensProducer-Retirement's R3 sub-gate 3 cascade.

### 4.4 First-time bootstrap (escape hatch)

Per `docs/design-pure-bootstrap-zero.md` §"First-time bootstrap" three resolutions, all give 0 hand-Rust *in v3's source tree*. The bin-shim emit pattern is compatible with each:

- **Resolution 1 (shipped pre-built binary):** the first invocation of any bin-shim uses the released binary; subsequent invocations use the regenerated-from-`.dag` form. No bootstrap concern at the bin-shim level.
- **Resolution 2 (universal-runtime crate):** `gunbc-runtime` provides `Dag`, `Pipeline`, and the trampoline; emitted bin-shims call into it. Same shape as resolution 1; the runtime crate is the trampoline.
- **Resolution 3 (procedural macro):** `#[gunbc::data]` resolves the `.dag` content at compile time; bin-shims are macro expansions of the `BinShim` declaration. Same shape; the macro is the trampoline.

This doc does not pick the resolution — that's downstream of ecosystem-strategy taste per the pure-bootstrap-zero doc. Bin-shim emit pattern works under any of the three.

## 5. Cascade and gates

### 5.1 R3-T-LensProducer-Retirement sub-gates (per `docs/r3-structure.md`)

| Sub-gate | Gated on | Lands when |
|---|---|---|
| `lens_apply_dot_rs_retired` | Item 4 (PB-Runtime interpreter-as-data) | PB-Runtime can execute the `.dag` lens-application body that replaces `lens_apply.rs:reflect_program_dag_nodes_in_file` + `apply_lens_declaration` |
| `lens_testgen_dot_rs_retired` | Item 4 (same gate) | PB-Runtime can execute the `.dag` testgen body that replaces `lens_testgen.rs` |
| `regen_lens_dot_rs_retired` | Item 5 (bin-shim emit pattern) | `regen_lens` ships as `data regen_lens_shim: BinShim = { ... }` + emitted Rust; equivalence-verified vs the current hand-Rust shim |

Sub-gates 1 + 2 are tightly coupled (same dissolution mechanism); sub-gate 3 is independent (different mechanism). This is why the locks are bundled into one doc — both sub-gate dispositions share Item 4's load-bearing PB-Runtime concept; sub-gate 3 layers Item 5 on top.

### 5.2 SG-0 = 0 cascade

Per `docs/design-pure-bootstrap-zero.md` §"In-tree floor target": SG-0 zero requires every hand-Rust file in `src/v3/` retired. T-LensProducer-Retirement closes the largest single class of hand-Rust (the program-sized `.rs` files); the bin-shim retirement closes the binary-entrypoint class. Together they reach SG-0 = 0 in `src/v3/compiler/src/` modulo first-time-bootstrap trampoline (which is out-of-tree per the chosen N=0 resolution).

### 5.3 Critical-path interaction with R2-Evaluator

Items 4+5 do NOT block R2-Evaluator landing. R2-Evaluator's PR-A through PR-E (per `docs/briefs/r2-evaluator-manager.md`) author the Rust-side runtime that PB-Runtime then mirrors structurally. Sequence:

1. R2-Evaluator (Rust) lands per its PR cadence (R2 phase)
2. PB-Runtime `.dag` program lands consuming the locked spec from this doc (R3 phase, post R2-Evaluator)
3. Dissolution: R2-Evaluator (Rust) retires once PB-Runtime (`.dag`) reaches feature-parity + bit-identical compile

R3-T-Tier3-Dissolution lane consumes this same path for the four hand-Rust mirrors (`termination`, `computation`, `induction`, `effect-carrier`).

### 5.4 Cross-program coordination: PB Manager + Evaluator Manager

Both managers consume Items 4+5:

- **R2 PB Manager** (`cool-stag-230` / `r2-pure-bootstrap-manager.md`): owns T-LensProducer-Retirement R3 lane (with sub-gates per §5.1); owns concrete `BinShim` declarations for PB-owned shims (instance-row authoring under `dsl/std/runtime/bin_shims/`) + bin-shim retirement dispatch; dispatches workers on bin-shim retirement once Item 5 lands. **Boundary**: PB owns the retirement lane and the per-shim instance declarations; any generalized substrate-shape change to the `BinShim` carrier itself (new `PipelineStep` variants, additional fields on the carrier, etc.) follows the §P1 substrate-fact-introduction procedure with escalation to Substrate Manager — same shape as anti-bridge invariant #2 below for `Value`. This split keeps PB's lane scope crisp while preserving Substrate's authority over carrier-type evolution.
- **R2 Evaluator Manager** (`snappy-moth-795` / `r2-evaluator-manager.md`): owns R2-Evaluator (Rust) per PR-A through PR-E; once R2-Evaluator stabilizes, signals PB Manager to start PB-Runtime `.dag` authoring.

The seam: both managers cite this doc as authority. Sub-brief authoring per manager is autonomous; cross-program escalation only on substrate-shape conflicts (e.g., the `Value` coproduct shape needs new substrate carriers — that escalates to Substrate Manager).

## 6. Anti-bridge invariants

PB-Runtime + bin-shim retirement workers MUST hold the following while landing the dissolution:

1. **No PB-Runtime divergence from R2-Evaluator semantics.** The `.dag` declaration of PB-Runtime IS the spec the Rust crate's tests verify against. If they diverge, the dissolution is broken; convergence is non-optional.

2. **No new `Value` primitives.** `Value` (per §3.2) is closed over the 5 substrate primitives' inhabitants. Adding a new primitive (e.g., `ClosureValue`, `EffectValue`, etc.) is a substrate-fact-introduction event requiring P1 procedure escalation to Substrate Manager. PB-Runtime workers MUST NOT extend `Value` locally. **Note**: `EvalFrame` / `EvalStateStack` (per §3.3) are NOT `Value` extensions — they're evaluator-internal state carriers in parallel substrate-typed space. Adding evaluator-internal carriers is the Evaluator Manager's PR-A scope and does NOT trigger this anti-bridge; only adding `Value` variants does.

3. **No bin-shim hand-Rust additions.** Once Item 5 lands, NEW bin-shims author as `BinShim` declarations + emitted Rust, not as hand-Rust. Hand-Rust under `src/v3/compiler/src/bin/` becomes a closed set with named retirement targets; it does not grow.

4. **No emitter-specific value model in PB-Runtime.** The `BinShim` emitter is one of many `.dag` emitters; its shape mirrors `dsl/extdeps/languages/rust/emit.dag` per the Q1/Q3/Q6.5 cascade. Workers MUST NOT carry parallel emit logic for bin-shims.

5. **No "PB-Runtime as separate language" framing.** PB-Runtime is `.dag`. Its declaration uses the same connectives, behaviors, and substrate carriers as every other `.dag` program. There is no PB-Runtime DSL distinct from the rest of the language.

6. **No fork between Item 4's `Value` and R2-Evaluator's runtime-value model.** R2-Evaluator's PR-A authors typed runtime values for the 6 type connectives + 5 L1 behaviors (per `r2-structure.md` §6 Evaluator brief items). Item 4's `Value` IS that runtime-value type expressed in `.dag`. They share a structural definition; if they fork, that's the structural error case anti-bridge invariant #1 names.

## 7. TestClaim shapes (verification surface)

Three shapes exercise the locks at landing time. These are bootstrap hooks per `src/v3/std/verification.dag` `TestPredicate`'s existing variants — they consume the dissolution as observable equivalence, not as a separately-encoded "PB-Runtime exists" predicate.

### 7.1 PB-Runtime equivalence fixture

For a small corpus of `.dag` programs (initially: arithmetic on `Int`; `List` map/fold; one `Lens<C>` instance application), evaluate via R2-Evaluator (Rust) and via PB-Runtime (`.dag`); assert structural equality of results.

```
TestClaim {
  name: "pb_runtime_equivalent_to_evaluator_on_corpus"
  predicate: DifferentialEquals { subject_ref: pb_runtime_evaluate, oracle_ref: r2_evaluator_evaluate, input_ref: corpus }
  // ...
}
```

Fails if PB-Runtime produces a different `Value` than R2-Evaluator for the same input. Verifies §6 anti-bridge invariant #1.

### 7.2 BinShim equivalence fixture

For each existing bin-shim (`regen_lens.rs` first), emit Rust from the `.dag` `BinShim` declaration; compare to the existing hand-Rust shim line-by-line modulo whitespace + comment differences.

```
TestClaim {
  name: "regen_lens_bin_shim_emits_equivalent_to_hand_rust"
  predicate: ExecuteCommand {
    command: "diff",
    args: [emitted_path, hand_rust_path],
    expect_exit_code: 0
  }
  // ...
}
```

Fails if the emitted Rust diverges materially. Verifies §4.3 dissolution discipline.

### 7.3 No-new-bin-shim-hand-Rust fixture

PB census-style gate verifying `src/v3/compiler/src/bin/` is a closed set after Item 5 lands. Each new `[[bin]]` Cargo.toml entry MUST cite a `BinShim` declaration, not a hand-Rust file path that doesn't exist as `data <name>_shim: BinShim = { ... }`.

```
TestClaim {
  name: "no_new_bin_shim_hand_rust"
  predicate: CensusBoundCheck {
    authority: hand_rust_bin_shim_count,
    list_constant: expected_hand_authored_bin_shims,
    bound: <closed-set count post-retirement>
  }
  // ...
}
```

Fails if hand-Rust bin-shim count exceeds the closed-set retirement schedule. Verifies §6 anti-bridge invariant #3.

## 8. Cross-references

- `docs/design-pure-bootstrap-zero.md` §"Bootstrap as data" + §"First-time bootstrap" — this doc operationalizes the (γ) model + escape hatch.
- `docs/design-pure-bootstrap-zero-audit.md` — audit trail for the 0-floor target this doc closes part of.
- `docs/design-pure-bootstrap.md` — older PB design context.
- `docs/r3-structure.md` §"Lane structure" T-LensProducer-Retirement row — names this doc's locks as the gate-prereq for sub-gates 1+2+3.
- `docs/r2-structure.md` §6 (Evaluator Manager) — R2-Evaluator runtime-value model + body evaluator + lens application + witness construction items 4 cite.
- `docs/briefs/r2-pure-bootstrap-manager.md` — PB Manager's program scope; consumes this doc for T-LensProducer-Retirement R3 lane authoring.
- `docs/briefs/r2-evaluator-manager.md` — Evaluator Manager's program scope; consumes this doc for R2-Evaluator-to-PB-Runtime convergence path.
- `docs/design-reflection-completeness.md` §3 + §6 — reflection vs evaluation distinction; PB-Runtime is the evaluation half.
- `docs/design-lens-framework.md` §Q6.5 — Q6.5's `LensInstanceKindWitness` shape is a downstream consumer of the runtime values PB-Runtime produces; convergence per §6 anti-bridge invariant #1.
- `docs/design-emission-model.md` Q1 refinement — `BoundedLattice<DescentEvidence>` for `LoopBound::Descent` is the algebra PB-Runtime's `Behavior::Loop` evaluation rule consumes.
- `src/v3/std/substrate.dag` — substrate carriers PB-Runtime consumes (`Behavior`, `LoopBound`, `LiteralBits`, `BranchPath`, etc.).
- `src/v3/compiler/src/bin/` — current hand-Rust bin-shims; dissolution targets per §4.3.
- `feedback_compiler_is_dag_processor.md` — 5-primitive constraint; informs §3.1.
- `feedback_executable_emission.md` — emission must produce fully executable code; §4.2 emit pattern.
- `feedback_no_generated_code_on_disk.md` — generated code must never be editable; emitted bin-shims are AUTO-GENERATED with the standard header.
- `feedback_parallel_representation_debt.md` — anti-bridge invariant shape; PB-Runtime ↔ R2-Evaluator convergence.
- `INVARIANTS.md` §P1 — substrate-fact-introduction procedure; §6 anti-bridge invariant #2 cites.
- `INVARIANTS.md` §P2 — boundary discipline; §2 dissolution shape preserves single-authority.

## 9. Status

**LOCKED 2026-04-29 (Director-authored, PM-confirmed sub-question dispositions per inbox #828 2026-04-29T00:36:47Z).**

Consumed by:
- R2 PB Manager brief (`docs/briefs/r2-pure-bootstrap-manager.md`) — PM consumes locks per agreed lockstep before next worker brief refresh cycle. Per PM commitment 2026-04-29T02:38Z: pre-emptive consumption lands before any PB Manager respawn fires.
- R2 Evaluator Manager brief (`docs/briefs/r2-evaluator-manager.md`) — same PM consumption pass.
- R3 T-LensProducer-Retirement lane brief (R3 phase, when authored).

PM (deep-wolf-155 / inbox #846) coordinates worker brief updates once this doc lands per established canonical-consumer pattern.
