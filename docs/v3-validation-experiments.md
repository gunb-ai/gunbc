# v3 Validation Experiments

> Part of: [v3-spec.md](v3-spec.md)
>
> **Purpose:** prove or disprove the v3 spec's claims inside v2
> before committing to a v3 build. Each experiment is bounded,
> has a clear pass/fail criterion, and teaches us something
> specific about the design.

## Experiment 1: Lambda → Bind + Define

**What it tests:** the kernel shape (5 behaviors) and the
"sameness" principle (lambda = function, no special handling).

**Scope:** in v2, make ExprLambda desugar to the same path as
a named function during DAG construction. Delete lambda-specific
downstream logic.

**Steps:**
1. In 02_parse.dag or 03_normalize.dag: when the parser sees
   `x => expr`, emit the same structure as `fn anonymous(x) { expr }`.
   Free variables (captures) become explicit input edges.
2. Delete ExprLambda variant from ExprData (or make it sugar that
   immediately lowers to ExprCall + a generated function).
3. Delete lambda-specific paths in 04_infer.dag (~77 lines),
   ownership.dag (~10 lines), 05_emit_rust.dag (3 emission modes).
4. Run the test suite. Run bootstrap.

**Pass criterion:**
- All 394 tests pass
- Bootstrap converges (regen → diff → empty)
- Net lines deleted > lines added
- No downstream code asks "is this a lambda?"

**What we learn if it passes:** the 5-behavior kernel works.
Transform with Define rule is sufficient for both lambdas and
named functions. Consumer code simplifies.

**What we learn if it fails:** lambdas have structural
differences we haven't accounted for. The spec needs to
explain what those differences are and whether they represent
genuine behaviors or missing physics.

**Estimated size:** 1-2 PRs, medium complexity. Touches parse,
infer, ownership, emit. But each touch is DELETION.

---

## Experiment 2: Carry one provenance fact, delete one CX heuristic

**What it tests:** the physics+lens principle. If the IR carries
the fact, the lens (complexity) can read it without reconstruction.

**Scope:** pick ONE function where complexity.dag reconstructs
"where did this value come from" via heuristics. Instead, carry
that fact through TypeBinding. Delete the heuristic.

**Candidate:** `classify_let_value()` in 04_infer.dag. This
function classifies whether a let-binding's value is a sub-value
of a parameter (for structural descent). CX reconstructs this
in `annotate_descent()`. Instead:
1. In 04_infer.dag: when creating the TypeBinding for a let,
   compute the SubValueRelation and store it on the binding.
2. In complexity.dag: read the SubValueRelation from the binding
   instead of reconstructing it. Delete the reconstruction logic.

**Pass criterion:**
- CX violation count does not increase (ratchet holds)
- At least one reconstruction function in complexity.dag is deleted
  or simplified
- Net lines deleted > lines added
- Bootstrap converges

**What we learn if it passes:** facts carried through bindings
dissolve downstream heuristics. The lens reads structure that
already exists. The core v2 diagnosis (construct-discard-reconstruct)
is confirmed as fixable.

**What we learn if it fails:** the binding boundary is harder to
enrich than expected, or the reconstruction does something the
simple provenance fact doesn't capture. The spec needs to address
whatever that is.

**Estimated size:** 1 PR, small-medium. Touches 04_infer.dag
(binding creation) and complexity.dag (reading). This is
essentially what Track 1 has been attempting — but scoped to
ONE function, not the whole pipeline.

---

## Experiment 3: Add a new transform, zero consumer edits

**What it tests:** the "variation is data" principle. New
transforms should be rule table entries, not structural changes
that ripple through every consumer.

**Scope:** add one new builtin operation to the .dag language.
Measure how many files need editing.

**Candidate:** add a `clamp(value, min, max)` builtin that
restricts a number to a range. This is a pure function with
known cost (O(1)), known effect (Pure), and known type
(Int → Int or Float → Float).

**Steps:**
1. Add `clamp` to the relevant extdeps or std/ authority
2. Wire it through the pipeline
3. Count every file that needed editing

**Pass criterion:**
- clamp works end-to-end (compile, emit, test)
- Files edited ≤ 3 (declaration, type rule, maybe emit template)
- complexity.dag: zero edits
- ownership.dag: zero edits
- No new match arms in any consumer

**What we learn if it passes:** the rule table approach works.
New operations are data. Consumers are generic over operations.

**What we learn if it fails:** consumers pattern-match on
specific operations somewhere. The spec needs to identify
where and explain how to make those consumers generic.

**Estimated size:** 1 PR, small.

---

## Experiment 4: Add one observational lens, zero compiler changes

**What it tests:** the lens extensibility principle. Users can
define new analyses without touching compiler code.

**Scope:** define a simple lens that reads the existing v2 IR
and produces a useful observation. Implement it as a .dag
function that the interpreter can run, NOT as compiler code.

**Candidate:** a "purity lens" — for each function, determine
whether it is pure (no service calls, no mutation). This
information is already in the IR (transport declarations,
service operations). The lens just reads it.

**Steps:**
1. Write a .dag function `is_pure(func: Node) -> Bool` that
   inspects the function body for service calls / transport
   declarations
2. Write a .dag function `purity_report(modules: List<Node>) -> List<PurityResult>`
   that applies is_pure to every function
3. Run it via `dag run purity_report.dag` on the compiler's own source

**Pass criterion:**
- The purity lens produces correct results for known-pure and
  known-impure functions
- Zero compiler .dag files edited
- Zero stage0 changes
- The lens runs via the interpreter, not a compiler pass

**What we learn if it passes:** lenses over the IR work without
compiler changes. The IR has enough structure for external
observation. User-defined lenses are feasible.

**What we learn if it fails:** the IR doesn't expose enough
structure for external observation. The spec needs to identify
what's missing and whether it's a physics gap or an access gap.

**Estimated size:** 1 PR, small. New .dag file only.

---

## Sequencing

```
Experiment 1 (lambda)     ← highest value, tests kernel shape
     |
Experiment 2 (provenance) ← tests physics+lens core claim
     |                       (can run in parallel with 1)
Experiment 3 (new transform) ← tests variation-as-data
     |                         (can run in parallel with 1+2)
Experiment 4 (purity lens)   ← tests lens extensibility
                               (can run in parallel with all)
```

All four are independent. Can run in parallel. Each is 1-2 PRs.
All must keep bootstrap green.

## Experiment 5: Measure the disease — ExprData variant edit cost

**What it tests:** the DISEASE, not the cure. How many files and
match arms need editing to add a new ExprData variant?

**Method:** count every match on `.expr_data` across all .dag files.

**Results:**

| Metric | Count |
|--------|-------|
| Files that MUST be edited | 6 |
| Files that SHOULD be reviewed | 5 |
| **Total files needing edits** | **8-11** |
| Exhaustive match arms | 4 |
| Big-dispatch match arms | 26 |
| **Total actionable match arms** | **30** |
| Estimated new lines | ~76-215 |

**Verdict:** Adding one ExprData variant costs 8-11 files and ~30
match arms. Thesis target: "cost of change = 1 file." V2 is 8-11x
over. Experiment 3 (clamp via rule table): 3 files, zero consumer
edits. Concrete improvement validated.

---

## Results

| # | Experiment | Result | Key metric |
|---|---|---|---|
| 1 | Lambda → Bind + Define | **PASS (partial)** | LambdaSemantics deleted, -30 lines. 43 ExprLambda refs remain for closure semantics. |
| 2 | Provenance on binding | **PASS (partial)** | Carry path works. classify_let_value reads scope_locals first. Reconstruction not yet fully deleted. |
| 3 | Add clamp builtin | **PASS (full)** | 3 files edited, zero consumer edits |
| 4 | Purity lens | **PASS (full)** | 1 new file, zero compiler changes, 3117 pure / 36 effectful |
| 5 | ExprData variant cost | **MEASURED** | 8-11 files, ~30 match arms per new variant |
| 6 | Layer opacity rename test | **PASS (bounded)** | 2 of 3 rename classes byte-identical; 1 leak at `kernel_type_set` (Part B-tracked) |

All experiments keep tests green (415 pass) and CX ratchet stable.

## Experiment 6: Layer opacity rename test

**Run date:** 2026-04-15 (post-commit to v3; validates the thesis's
compositional layering claim against the v2 compiler empirically).

**What it tests:** the thesis's §"Compositional layering:
below-boundary opacity by construction" claim. Specifically, that
below-boundary changes in a layer are invisible to consumers — i.e.,
the generated code is structurally unchanged when an intermediate
layer is modified internally. See `THESIS.md` §"Compositional
layering" and `INVARIANTS.md` §"Layer opacity" for the motivating
principle and the enforcement invariant.

**Scope:** compile `dsl/examples/weather/weather.dag` to Rust using
v2, then modify `dsl/std/float.dag`'s layer chain three ways and
diff the generated Rust against the baseline.

**Steps:**

1. **Baseline.** Compile weather with unmodified std/:
   `cargo run --bin v2-compiler -- compile --source-root dsl/examples/weather --source-root dsl/std --output-dir /tmp/before --target rust`.
2. **Experiment 6a — insert intermediate layer.** Copy std/ to
   a temp directory; edit `float.dag` to insert a new intermediate
   alias: `type PreciseScalar = Float64; type Float = PreciseScalar`
   instead of `type Float = Float64`. The chain lengthens from
   `Float → Float64 → Field<Word64>` to `Float → PreciseScalar →
   Float64 → Field<Word64>`. Weather.dag unchanged. Recompile to
   `/tmp/after1`.
3. **Experiment 6b — internal rename below boundary.** Edit
   `float.dag` to rename the internal types: `type BinaryFloat64 =
   Field<Word64>; type Float = BinaryFloat64`. The boundary name
   `Float` is preserved; the internal name `Float64` becomes
   `BinaryFloat64`. Weather.dag unchanged. Recompile to `/tmp/after2`.
4. **Experiment 6c — rename the boundary identifier itself.** Edit
   `float.dag` to rename `Float → FloatingPoint`. Update weather.dag
   to use the new name (expected: consumers depend on boundary
   names). Add `FloatingPoint` to `dsl/std/types.dag`'s
   `kernel_type_set`. Add an explicit `import std.float
   { FloatingPoint }` to weather.dag. Recompile to `/tmp/after3`.
5. **Diff each experiment against the baseline.** Run each
   comparison as its own `diff -r` invocation (brace-expansion
   expands to four operands, which `diff` rejects with "extra
   operand"):
   ```
   diff -r /tmp/before /tmp/after1
   diff -r /tmp/before /tmp/after2
   diff -r /tmp/before /tmp/after3
   ```
   Or equivalently with a shell loop: `for i in 1 2 3; do diff
   -r /tmp/before /tmp/after$i; done`. Each experiment is a pass
   if and only if the diff for that experiment is empty (`diff`
   exits with status 0 and produces no output).

**Pass criteria:**
- 6a (insert layer): byte-identical generated Rust. If layering
  holds for added intermediates, `diff -r` exits 0.
- 6b (internal rename): byte-identical generated Rust. If layering
  holds for below-boundary renames, `diff -r` exits 0.
- 6c (boundary rename): byte-identical generated Rust modulo
  renamed identifiers. If layering holds for canonical primitive
  names, the diff shows only the renamed type name in
  field/function signatures and nothing else.

**Results:**

- **6a (insert layer): PASS.** `diff -r` exit 0. The entire
  generated Rust project — `Cargo.toml`, `lib.rs`,
  `examples_weather.rs`, `v2_rt.rs`, `main.rs` — is byte-identical
  to the baseline. v2 walks the algebraic chain through the new
  intermediate alias and produces the same `f64` mapping.
- **6b (internal rename): PASS.** `diff -r` exit 0. Renaming
  `Float64 → BinaryFloat64` below the boundary is invisible to
  weather.dag and to the generated Rust. The compiler walks
  through the renamed type names structurally.
- **6c (boundary rename): LEAKS.** Compilation succeeds after
  the kernel_type_set update + explicit import, but the generated
  Rust is **structurally different**:
  - `celsius: f64` → `celsius: Box<FloatingPoint>` (lost the
    primitive fast-path mapping)
  - `Temperature` loses its `Copy` derive (was `#[derive(Debug,
    Clone, Copy, ...)]`, becomes `#[derive(Debug, Clone, ...)]`)
  - `pub high: Temperature` → `pub high: Rc<Temperature>`
    (cascade from lost Copy)
  - `fn to_fahrenheit(temp: Temperature) -> f64` →
    `fn to_fahrenheit(temp: Rc<Temperature>) -> FloatingPoint`

**The leak mechanism:** v2's inference and emission have a fast
path for types whose canonical name appears in `kernel_type_set`
(a `Map<String, Bool>` in `dsl/std/types.dag` line 65, mirrored
into `src/v2/stage0/src/std_types.rs`). Types in the fast path
get efficient primitive mappings (direct target type, `Copy`
derive, unboxed). Types NOT in the fast path get defensive
wrappers (`Rc<T>`, `Box<T>`, no `Copy`). Renaming a primitive
from `Float` to `FloatingPoint` moves it from fast path to slow
path because the name no longer matches the hardcoded list.

The leak is tracked in v2 as "Part B pending": a comment above
`kernel_type_set` reads *"Until Part B lands (inference resolves
methods from type fields), the compiler still uses kernel_types/
is_kernel_type for these."* v2 itself knows this is a temporary
scaffold waiting for structural replacement.

**What the experiment proves:**

- **Compositional layering holds empirically** for the first two
  rename classes (insert intermediate, internal rename below
  boundary). v2 genuinely walks the algebraic chain and the
  consumer is unaware of intermediate layer changes. The OSI-stack
  analogy is not theoretical — it's demonstrated in v2 today.
- **The leak class is "canonical-primitive-name rosters."** Any
  compiler that stages up from minimal parsing accumulates a
  hardcoded list of "known primitive names" because it's the
  cheapest way to recognize primitives. The list is the leak.
  Dissolving it requires keying the primitive roster on
  `DeclarationId` instead of `String`.
- **The leak is bounded and well-understood.** It's not "layering
  doesn't work" — it's "layering works except at the canonical-
  primitive-name boundary, which is a known scaffold." v2 has
  eight names in the roster; every other rename passes the test.
- **v3 reproduced the same leak in PR-B's emit layer.** The
  `emit_rust.rs` file has `lookup("Int", "")`, `lookup("Bool",
  "")`, and `match label.as_str() { "True" => ..., "False" =>
  ... }` patterns — identical shape to v2's `kernel_type_set`, at
  a different layer. This is what the review loop has been
  rediscovering round after round, and it is what
  `INVARIANTS.md` §"Layer opacity" exists to catch going forward.

**The rename test as a permanent invariant:** every experiment
above reduces to "rename a below-boundary identifier, recompile,
`diff -r`." The test is O(minutes) to run per identifier and
catches the failure class that has historically been responsible
for the largest share of review-round findings in both v2 and v3.
CI-gate form is a grep audit (see `INVARIANTS.md` §"Layer
opacity"); runtime form is to actually rename and diff, which is
what this experiment does.

## What the experiments actually prove

- **Transform/rule-table mechanism:** validated (exp 3 full pass)
- **Observational lens mechanism:** validated (exp 4 full pass)
- **Carry-facts-through-bindings:** validated at substrate level (exp 2 partial)
- **"Lambda = function" was too coarse:** real distinction is closure/binding
  semantics, not lambda syntax (exp 1 discovery)
- **v2 ExprData tax:** empirically real at 8-11x over target (exp 5)
- **Compositional layering holds empirically:** validated for non-boundary-rename
  layer changes (exp 6). The bounded leak class is "canonical primitive
  name rosters" — tracked in v2 as Part B, in v3 as class-5 gap #6.

## Revised acceptance criteria (per reviewer feedback)

The original criteria were too focused on optics (zero refs, net deletion).
These track ontology instead:

- **Exp 1 revised:** any remaining lambda-specific logic must be justified
  by closure/binding semantics (fresh scope, capture fan-out, iteration
  context), not by missing carried facts or surface-syntax differences.
  Bucket the 43 refs into: surface syntax, real closure semantics, residual
  v2 artifact. Only the last bucket is a deletion target.

- **Exp 2 revised:** at least one reconstruction function DELETED (not just
  bypassed). The carry path working in parallel with the old path is a
  parallel implementation, not a dissolved heuristic. The dividend is
  enabled but not yet banked.

## After validation

The experiments validate the v3 direction with honest partials:
- The design is sound where tested
- The spec needed one refinement (callback rule for closures-in-Loops)
- The transition strategy (v3-from-scratch vs incremental) is unvalidated
  by these experiments — they tested mechanisms, not migration
