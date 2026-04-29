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

- `Behavior::Value(v)` → emit `LiteralValue(v.payload)`.
- `Behavior::Transform(t)` → recursive evaluate `t.inputs`; apply `t.target` to result. Per `t.target` variant:
  - `Callable(decl_id)` → resolve `decl_id`'s declared Arrow type; on `ArrowBody::UserDefined(body_node_id)`, bind `t.inputs` to the function's declared parameters in a fresh evaluation frame (see §3.3); evaluate `body_node_id` in that frame; the frame pops when the call returns. Other `ArrowBody` variants (`ExternalRealization` / `Pending` / `NoBody` / `Unparsed`) signal non-evaluable cases — `ExternalRealization` dispatches to the host-bound implementation; the rest are evaluation-time errors per `feedback_fail_closed_discipline.md`.
  - `FieldProject { field_label, field_child }` → project `field_label` from the (single-input) record value.
  - `Operator(op_kind)` → apply the primitive operator to the input values.
- `Behavior::Branch(b)` → evaluate `b.input`; pattern-match against `b.paths`; recursive evaluate the selected `path.body` (a `NodeId`) with `path.binding` bound in a fresh frame.
- `Behavior::Loop(l)` → fold over `l.bound` (cardinality bounded iteration or descent-bounded recursion per the `LoopBound` coproduct); evaluate `l.body` (a `NodeId`) per iteration with accumulator bound to `l.init`'s value initially, threaded through subsequent iterations.
- `Behavior::Bind(b)` → registers a binding (`b.name` becomes resolvable; the value reachable through `b.result_port` becomes the bound value). Does NOT execute a body — `BindNode` carries no body field; the body that *uses* the bound name lives at downstream Transform/Bind nodes referencing `b.name` (via `Callable(decl_id)` for function-form binds) or via PortId wiring (for `let`/`where`-form binds). The fresh evaluation frame for function-form Bind happens at the `Transform(Callable(...))` site, not at the `Bind` site itself.

These five rules ARE the runtime. The `.dag` declaration of `evaluate`'s body IS the spec.

**Substrate-shape note.** Function bodies live on the type's `ArrowBody::UserDefined(NodeId)`, not on `BindNode`. `BindNode` registers names + parameter ports; the body it "binds" is reached via the corresponding type declaration. This matches the v3 substrate at `src/v3/std/substrate.dag` `ArrowBody = UserDefined(NodeId) | ExternalRealization(DeclarationId) | Pending | NoBody | Unparsed(SourceSpan)`.

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

The closed set of hand-Rust binary entrypoints under `src/v3/compiler/src/bin/` is the canonical authority **`EXPECTED_HAND_AUTHORED_NON_TEST`** at `src/v3/compiler/tests/integration/sg0_census_test.rs:170` (filtered by path prefix `src/v3/compiler/src/bin/`). At authoring time of this lock, that subset includes:

```
src/v3/compiler/src/bin/r1c_e_emit_gates.rs
src/v3/compiler/src/bin/regen_bootstrap.rs
src/v3/compiler/src/bin/regen_lens.rs
src/v3/compiler/src/bin/regen_parse.rs
src/v3/compiler/src/bin/regen_parse_tables.rs
src/v3/compiler/src/bin/regen_tokenize.rs
src/v3/compiler/src/bin/regen_v3.rs
src/v3/compiler/src/bin/self_host_fixed_point.rs
```

**The ratchet authority — not this listing — is canonical.** Refresh by reading the live ratchet at retirement-PR authoring time; any new bin-shim added to the source tree must appear in `EXPECTED_HAND_AUTHORED_NON_TEST` (sub-ratchet contract per `sg0_census_test.rs:347`) and is therefore covered by the retirement program by construction. The TestClaim shape in §7.3 cites this authority directly rather than this snapshot list.

Each entry is a thin host shim — typically <200 lines — that:
1. Constructs a `Dag` (loads bootstrap or compiles a registry-declared `.dag` file)
2. Runs a compiler pipeline phase (compile / tokenize / parse / regen)
3. Writes output to a registry-declared path

The dissolution observation: **none of the actual logic in these shims is Rust-specific.** Each pipeline is expressible as a `.dag` function returning a process-exit carrier — see §4.2 below.

### 4.2 The emit pattern

A bin-shim is a `.dag` declaration that points at a `.dag` entry function, NOT a separate pipeline DSL. The `.dag` language already has function calls + record returns; bin-shims compose load/compile/write as ordinary `.dag` calls. No parallel-representation pipeline coproduct.

```
// Entry point is an ordinary .dag function returning a process-exit carrier.
fn regen_lens_main() -> std.process.ProcessExit {
  let dag = std.bootstrap.load_dag();
  let entries = compile.lens_registry.read(dag, regen_dag_lens_registry);
  for entry in entries {
    let module_text = emit.rust.emit_module(dag, entry);
    std.fs.write(entry.generated_file, module_text);
  }
  std.process.ProcessExit.success
}

// Bin-shim = entry-point declaration + binary metadata.
data regen_lens_shim: BinShim = {
  entrypoint_name: "regen_lens"
  description: "Unified lens-regen driver. Reads LensRegistryEntry records and regenerates each lens module."
  entry: regen_lens_main
}
```

Where `BinShim` is a substrate-declared carrier (lives in `dsl/std/runtime/bin_shim.dag` or equivalent — Substrate Manager picks per their dispatch):

```
type BinShim {
  entrypoint_name: NonEmptyStr     // becomes binary name in Cargo.toml [[bin]] target
  description: String              // populates the Rust file's doc comment
  entry: DeclarationRef            // points at a `.dag` fn () -> std.process.ProcessExit
}
```

**No `PipelineStep` DSL.** Earlier drafts of this doc proposed a coproduct over `LoadDag` / `CompileLensRegistry` / `EmitRustModules` / etc. as separate pipeline-step variants. That was parallel-representation: `.dag` is already the pipeline DSL — function calls compose; records return; sequencing is structural. Reintroducing a step-DSL would duplicate the language inside itself. Codex review on PR #1176 caught this as a substrate-fact-introduction-without-P1 violation; corrected to the simpler `entry: DeclarationRef` shape.

**`std.process.ProcessExit` substrate prerequisite.** The entry function returns a process-exit carrier (success / failure with exit code + optional message). `std.process.ProcessExit` does NOT yet exist in the substrate at HEAD; it's a substrate-fact-introduction prerequisite for Item 5 retirement, owned by Substrate Manager (per anti-bridge invariant #2 in §6 below). The carrier shape is downstream of the actual feature ask; sketch:

```
// Substrate-fact-introduction prerequisite — Substrate Manager owns the shape.
// Sketch (subject to P1 procedure when authored):
type ProcessExit
  = Success
  | Failure { exit_code: Int, message: String }
```

Until Substrate Manager declares `std.process.ProcessExit`, bin-shim retirement workers MUST signal a P1 escalation rather than authoring locally. This makes the substrate-prerequisite explicit and sequenced: PR-PreF-style work (Substrate-side authoring of `ProcessExit`) gates Item 5 retirement.

The emitter for `BinShim` declarations is a `.dag` program (analogous to existing emit modules per `dsl/extdeps/languages/rust/emit.dag`) that produces a Rust file shaped like:

```rust
// AUTO-GENERATED from dsl/std/runtime/bin_shims/regen_lens.dag — DO NOT EDIT.
//
// {description}

use gunbc_runtime::{Dag, ProcessExit};

fn main() -> ExitCode {
    match {entry_fn_qualified_name}(&Dag::new()) {
        ProcessExit::Success => ExitCode::SUCCESS,
        ProcessExit::Failure { exit_code, message } => {
            eprintln!("{message}");
            ExitCode::from(exit_code as u8)
        }
    }
}
```

The emitted Rust is standardized — per-shim variation is the entry function name only. The Rust shape itself is one template, parameterized by `entry_fn_qualified_name`.

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

**Live-substrate predicate match.** `DifferentialEquals { subject_ref, oracle_ref, input_ref }` is an existing `TestPredicate` variant at `src/v3/std/verification.dag:177-181`. The cited `DeclarationRef`s (`pb_runtime_evaluate`, `r2_evaluator_evaluate`, `corpus`) are forward-declarations gated on the prerequisite landings: `r2_evaluator_evaluate` lands with R2-Evaluator's PR-A (Rust crate); `pb_runtime_evaluate` lands with PB-Runtime's `.dag` declaration in R3; `corpus` is authored alongside the worker dispatching this fixture. Worker authors the TestClaim declaration immediately as `ReleaseDeferredClaim`-shape (per `src/v3/std/verification.dag` `ReleaseDeferredClaim` discipline introduced in PR #1128) until prerequisite refs resolve.

Fails if PB-Runtime produces a different `Value` than R2-Evaluator for the same input. Verifies §6 anti-bridge invariant #1.

### 7.2 BinShim equivalence fixture

For each existing bin-shim (`regen_lens.rs` first), emit Rust from the `.dag` `BinShim` declaration; verify the emitted Rust is **behaviorally equivalent** to the existing hand-Rust shim — not byte-identical. Hand-authored Rust and `.dag`-emitted Rust will inevitably differ in formatting choices, comment shapes (the emitted form carries an `AUTO-GENERATED` header), and incidental whitespace; the dissolution discipline cares about behavioral equivalence, not character-level identity.

The precise predicate shape is deferred to the worker authoring the fixture; the design lock fixes the *intent*, not the comparison mechanism. Three plausible mechanisms:

- **Canonicalize-then-diff:** `rustfmt` both sides; strip `AUTO-GENERATED` header from emitted side; then `diff` with `expect_exit_code: 0`. Catches semantic differences while tolerating formatting drift.
- **AST-equivalence:** parse both sides via `syn`; compare AST modulo span/comment metadata. Strictest semantic check; immune to formatting/comment differences by construction.
- **Behavioral-equivalence:** run both shims on a fixed input set; compare exit codes + stdout + filesystem effects. Captures the actual contract (the shim's purpose) rather than its source-text shape.

```
TestClaim {
  name: "regen_lens_bin_shim_emits_behaviorally_equivalent_to_hand_rust"
  // predicate: <one of the three mechanisms above; worker picks at fixture-authoring time>
  // ...
}
```

Fails if the emitted Rust shim does not produce the same runtime behavior as the hand-Rust shim (exit codes / stdout / filesystem effects on a fixed input set). Verifies §4.3 dissolution discipline.

**Note on the original `diff`-with-`expect_exit_code: 0` shape:** an earlier draft used naive byte-`diff` which is in tension with "modulo whitespace + comment differences." That tension is resolved by deferring the precise predicate to the worker authoring the fixture — the design lock fixes the equivalence intent, not the comparison shape.

### 7.3 No-new-bin-shim-hand-Rust fixture

PB census-style gate verifying `src/v3/compiler/src/bin/` is a closed set after Item 5 lands. The actual ratchet authority is `EXPECTED_HAND_AUTHORED_NON_TEST` at `src/v3/compiler/tests/integration/sg0_census_test.rs:170` (filtered by path prefix `src/v3/compiler/src/bin/`); the TestClaim cites that authority directly rather than a parallel list.

**Substrate prerequisite.** `CensusBoundCheck { authority, list_constant, bound }` is an existing `TestPredicate` variant at `src/v3/std/verification.dag:196-200`. However, the cited `list_constant: expected_hand_authored_bin_shims` does NOT exist at HEAD — current declared `CensusListConstant` values are `expected_hand_authored_non_test` (substrate.verification.dag:35) and `expected_hand_authored_test` (substrate.verification.dag:36) only. Adding a new constant is a substrate-fact-introduction event under §P1 owned by Substrate Manager. The bin-shim subset is more naturally a *filter* over `expected_hand_authored_non_test` (path prefix `src/v3/compiler/src/bin/`); whether this is best expressed as (a) a new top-level `CensusListConstant` or (b) a derived predicate over the existing constant + filter is a P1 disposition Substrate Manager owns when the retirement worker dispatches.

```
TestClaim {
  name: "no_new_bin_shim_hand_rust"
  // predicate: CensusBoundCheck { authority: <SG-0 ratchet authority>, list_constant: <P1-introduced; see above>, bound: <closed-set count post-retirement> }
  // OR equivalent shape using existing list_constant + subset_predicate via CensusSubsetCount
  // ...
}
```

`CensusSubsetCount { authority, list_constant, subset_predicate }` (`src/v3/std/verification.dag:201-205`) is a plausible alternative shape if the bin-shim set is best expressed as a path-prefix subset of `expected_hand_authored_non_test` — Substrate Manager picks the cleaner P1 disposition.

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
