# v4 Compilation Milestones

**Purpose:** Define what "the v4 compiler is running" means at each discrete stage,
what evidence confirms each milestone, and what is blocking each one.
This is a planning authority — not a task ledger. Task details and dependencies
live in `src/v4/TASKS.md`; this doc answers "when can we compile?" at each level.

**Relationship to bootstrap.dag:** `src/v4/workflow/bootstrap.dag` models the
correct four-stage chain (seed→stage0→stage1→stage2, fixpoint stage1==stage2).
This doc maps that chain to observable milestones and identifies the current gap
between the structural model and what is actually happening in CI.

---

## The bootstrap chain (what bootstrap.dag says)

```
v4 .dag source
      │
      │  compiled by: v2_pipeline (the seed)
      ↓
stage0 binary    ← first milestone: does this binary exist?
      │
      │  compiled by: stage0 binary
      ↓
stage1 binary    ← second milestone: does this binary produce the same output?
      │
      │  compiled by: stage1 binary
      ↓
stage2 binary    ← fixpoint: stage1 output == stage2 output (bit-identical)
```

This is designed correctly in bootstrap.dag. The question is which steps
are currently real vs. structural models only.

---

## What "compile" means at each layer

There are three different things CI calls "compile" — it's important to
distinguish them because only one produces an executable:

| Step | Command | What it checks | Does it produce a runnable binary? |
|---|---|---|---|
| **Structural compile** | `v2-compiler compile --target dag src/v4` | Parse + type-check all v4 .dag files | No — produces .dag IR artifacts |
| **Rust emit** | `v2-compiler compile --target rust src/v4` | Parse + type-check + emit Rust source | No — produces .rs files, needs rustc |
| **Full bootstrap** | rust emit → rustc → binary | Full pipeline to a running binary | **Yes** |

**Current CI state:** the "v2 → v4 bootstrap compile (fail-closed full)" step uses
`--target dag`. It is a structural check, not a compilation to a running binary.
`--target rust` is available and used for specific v4 files (lens-ci registry step,
T-22 host eval receipt), but has not been run over the full src/v4 source tree and
linked into a binary in CI.

---

## Milestone 0 — Structural compile (CURRENT, CI green)

**Definition:** v2 can parse and type-check all v4 .dag files with zero diagnostics.

**Evidence:** CI step "v2 → v4 bootstrap compile (fail-closed full)" exits 0.

**What this proves:** Every v4 type is well-formed. Every import resolves. Every
function signature is consistent with its body's type. The structural design is
sound.

**What this does NOT prove:** Any function body does what it claims. The
tokenizer tokenizes. The parser parses. The evaluator evaluates. The emitter emits.
None of these are executed during M0.

**Status:** ✓ Achieved. This is the current state of the project.

---

## Milestone 1 — Rust emit from full v4 source (NOT YET IN CI)

**Definition:** `v2-compiler compile --target rust src/v4` succeeds over the full
source tree and `rustc` compiles the output to a stage0 binary without errors.

**Evidence:** CI step that runs `v2-compiler compile --target rust src/v4 --output-dir /tmp/v4-stage0 && rustc ...` exits 0. A `v4-compiler` binary exists and runs.

**What this proves:** The v2 emitter understands all v4 type constructs well enough
to produce valid Rust. The generated Rust is structurally coherent. This is the
first moment that "v4" exists as an executable artifact.

**What this does NOT prove:** The stage0 binary can compile a v4 program correctly.
It may exist but produce wrong or empty output for any input.

**Known blockers:**
1. **T-10 (05_emit.dag) is 45 lines — a stub.** The `emit` function composes
   `serialize_target ∘ translate` but the composition is skeletal. v2 can emit Rust
   *for* the emit stage (turning it into a Rust function), but that Rust function
   will also be a stub that produces no output for any real input.
2. **No CI step exists** that runs `--target rust` over all of src/v4 and attempts
   rustc. This needs to be added to discover any emit-time errors before attempting M2.
3. **Dependency:** this milestone does NOT require T-6/T-7 algorithm walks to be real.
   The Rust emitted from the stub implementations will compile; the stubs just won't
   do anything useful at runtime.

**Required work to reach M1:**
- Add CI step: `v2-compiler compile --target rust src/v4 --output-dir ... && rustc main.rs -o v4-stage0`
- Fix any Rust emit errors that surface (likely some v4 type constructs not yet
  handled by v2's Rust emitter)
- This is primarily a CI wiring task + v2 emitter gap-filling, not v4 modeling work

---

## Milestone 2 — Stage0 can compile a trivial v4 program (first real execution)

**Definition:** The stage0 binary, given a minimal .dag input (e.g. a single `fn add(a, b) = a + b`), tokenizes it, parses it, resolves it, infers it, and emits valid Rust output.

**Evidence:** A CI test that invokes `v4-stage0-compiler compile trivial.dag --output-dir /tmp/out && rustc out/trivial.rs -o /tmp/trivial && /tmp/trivial` exits 0.

**What this proves:** The pipeline is real end-to-end for at least one program.
The first moment that v4 actually compiles something.

**Known blockers:**
1. **T-6 lexer walk ("not realized").** `01_tokenize.dag` has `ModeledLexRules { root: _ } → Rejected("lexical walk not realized")`. The lex rule data is modeled but the walk algorithm that uses it is a stub. This MUST be real for M2. One worker, one file.
2. **T-7 parser walk — `parse_expr` stub.** `02_parse.dag` has been partially filled (#3626): `parse()` now dispatches through `grammar_lookup_production` → `parse_production` → `parse_expr`. The remaining stub is `parse_expr` itself, which has a `🟡 gated — feature:T-7-parse-walk-realization` marker and always returns `ParseExprRejected`. dag.dag grammar data is filled; `parse_expr` needs implementing to match the grammar rule structure against the token stream.
3. **T-10 emit must produce real output.** `05_emit.dag` must compose translate output into actual Rust source text for the target. Currently 45 lines.
4. **Trivial input scope:** the minimal viable input only needs to exercise the pipeline for the .dag language's grammar (dag.dag is the only language with lex/grammar data filled). Python, Go, etc. are not needed for M2.

**Required work to reach M2:**
- Implement lexer walk in 01_tokenize.dag (algorithm, 1 worker, ~200-400 lines)
- Implement `parse_expr` in 02_parse.dag (the remaining stub; `parse_production` and `parse()` structure already real)
- Wire T-10 emit to produce real Rust source text from a translated node tree
- T-8 normalize + resolve: modeled and in the chain; single-file trivial input does not hit the T-28 cross-file bridge gate
- T-9 infer: modeled and in the chain; no additional work for trivial input

**Sequential dependency:** T-6 walk → T-7 `parse_expr` → T-8 normalize/resolve (modeled) → T-9 infer (modeled) → T-10 emit → M2.
T-8 and T-9 are modeled but required stages; they are not additional blockers for trivial input.

---

## Milestone 3 — Stage0 compiles the v4 pipeline itself (stage1 exists)

**Definition:** The stage0 binary, given src/v4 as input, produces a new binary
(stage1). stage1 is functionally complete: it can compile v4 programs, not just trivial ones.

**Evidence:** `v4-stage0-compiler compile src/v4 --output-dir /tmp/stage1 && rustc /tmp/stage1/main.rs -o v4-stage1`

**What this proves:** The v4 compiler can compile the v4 compiler. Self-hosting
is structurally achieved (even if not yet bit-identical).

**Known blockers (beyond M2):**
1. **T-9 infer fully exercised.** The infer stage must process all v4 type constructs
   present in src/v4 itself. Currently modeled; exercised at M2 only for trivial input.
2. **T-8 resolve cross-file bindings (T-28 bridge).** `resolve_with_graph` currently
   drops the graph parameter (`🟡` gate). Cross-file imports (which src/v4 has extensively)
   require this to be real.
3. **Full lex/grammar data for .dag language.** dag.dag already has wave-1 lex/grammar
   data. It needs to be complete enough to parse all of src/v4's constructs.

---

## Milestone 4 — Fixpoint: stage1 output == stage2 output (T-15)

**Definition:** `hash(stage1 output of compiling src/v4) == hash(stage2 output of compiling src/v4)`.
The v4 compiler is self-consistent: compiling with stage0 and compiling with stage1 produce
the same output.

**Evidence:** CI step in workflow/ci.dag that computes content_hash of both stage outputs
and asserts equality. Documented in bootstrap.dag `bootstrap_plan_fixpt_witness`.

**Status:** Defined structurally in bootstrap.dag (content hash pins are placeholder Symbols,
awaiting real B1 content_hash computation). Milestone reached only after M3 is stable.

**Note:** M4 is T-15. It is the "v4 done" gate per TASKS.md. All other tasks (T-16 omni
demo, T-36 ingest round-trip, etc.) feed into whether the fixpoint binary is also
feature-complete, but the fixpoint itself is a correctness property, not a feature property.

---

## Parallel track — Execution validation (TestClaim runner)

The milestones above are about the *compiler pipeline* producing Rust output.
Separately, the TestClaim runner (T-22 + T-34) enables *executing* v4 expressions
and verifying behavioral claims. These are parallel tracks:

```
Compiler pipeline track              Execution/eval track
─────────────────────────            ──────────────────────────
M0: structural compile ✓             v4_evaluator wave-1 stubs (all deferred)
M1: Rust emit → binary               T-34 concrete runtime (primitive ops real)
M2: trivial program compiles         T-22 eval executes simple expressions
M3: self-compilation                 TestClaim receipts execute and pass
M4: fixpoint (T-15)                  Full TestClaim corpus green
```

These tracks are independent until M3/M4, where they converge: a complete v4 binary
should be able to run its own TestClaim suite.

**Current eval/runtime state:**
- `v4_evaluator.dag`: every primitive operation (`call_primitive`, `choose_branch`,
  `step_loop`, `call`) returns `v4_eval_wave1_semantics_deferred` — always rejected.
- `05_eval.dag`: 1099 lines of eval logic, structurally correct, consuming runtime
  hooks that all reject immediately. Nothing can execute.
- **To unlock:** fill `v4_evaluator.dag` primitive hooks with real implementations
  (T-34 Wave 2). This is the single unlock for the eval track.

---

## Design gaps to resolve (not implementation gaps)

These are decisions required before the corresponding work can be dispatched:

**Gap 1 — CI wiring for M1 (v2 emit → rustc)**
No CI step runs `--target rust` over full src/v4 and attempts to link a binary.
Adding this step will reveal which v4 type constructs v2's Rust emitter doesn't
handle yet. This should be the *first* action — it immediately surfaces M1 blockers.

**Gap 2 — T-6/T-7 algorithm scope**
The lexer and parser algorithm walks must be written in `.dag` per the Pure Bootstrap
Zero mandate (`THESIS.md`, `docs/design-pure-bootstrap-zero.md`): v4 compiler behavior
is authored in `.dag`, not hand-Rust. Writing them in Rust would be a P5 scaffold
(`docs/modeling-discipline.md` Practice 5 — Progress Is Dissolution) and requires an
explicit dissolution receipt before a worker can be dispatched on that path.

Authoring in `.dag` does NOT make M2 depend on T-22/T-34. The path is: T-6/T-7
algorithms written in `.dag` → v2 emits Rust from them (--target rust, same as the
rest of src/v4) → stage0 binary includes those emitted Rust functions and runs them
natively. T-22 (the v4 interpreter) and T-34 (the runtime) are needed for the
TestClaim *execution* track — evaluating v4 expressions at development time — but
not for the compiled binary to execute its pipeline. The two tracks are genuinely
independent at M2.

**Gap 3 — T-28 cross-file resolution**
`resolve_with_graph` has a `🟡` gate marking that it drops the module graph. Cross-file
imports (which every v4 file uses) won't resolve correctly until this is real. M3
requires this. M2 with a single-file trivial input does not.

**Gap 4 — T-10 emit scope vs. translate**
`06_translate.dag` is 707 lines and more developed. `05_emit.dag` is 45 lines. Is
translate sufficient and emit just needs the composition wired, or is there substantial
emit work remaining? Clarifying this determines whether T-10 is a day of wiring or weeks
of work.

---

## Recommended immediate actions

In priority order:

1. **Add M1 CI step** (`--target rust` + rustc) and surface what breaks. Zero code
   changes to src/v4 required — just a CI wiring change. Output: a list of v2 emitter
   gaps for v4 constructs.

2. **Implement T-6 lexer walk** in 01_tokenize.dag (one worker, one file, fills the
   `ModeledLexRules` arm). v2 emits Rust from it; stage0 runs it natively.

3. **Implement T-7 `parse_expr`** in 02_parse.dag (one worker, fills the remaining
   stub). Structure above `parse_expr` already real from #3626.

4. **Fill T-34 primitive hooks** in v4_evaluator.dag (T-34 Wave 2) in parallel with
   T-6/T-7. Unlocks the eval/TestClaim track independently of the compiler pipeline.

5. **Wire T-10 emit** once translate output is understood. Determine if this is wiring
   or new modeling.
