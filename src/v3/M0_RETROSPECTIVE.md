# M0 Retrospective

**Status:** Complete. 35 acceptance tests passing on `quick-ant-526`. PR #441
ready for final review. The M0 substrate proves the v3 thesis primitives
work under adversarial review.

## Substrate decisions and whether they held

The five L1 behaviors — Value, Transform, Branch, Loop, Bind — held through
10 milestones and three reviewer rounds without modification. The C1 STOP
signal ("wanting a 6th variant") never fired. The thesis claim that these
are the terminal decomposition survived contact with building against it.

**Decisions that held without changes:**

- Five L1 behaviors as terminal (C1)
- `FunctionRef` on Transform (C2 dissolved to nothing at M0.3 — no
  TransformRule variants, operators resolve to `std::int::add` etc.
  at parse time)
- Define as a Bind with non-empty `params`, not a Transform variant
- Fail-closed compile boundary via `Err(CompileError::Semantic(dag))`
- `mark_unresolved` as the single authority for port state transitions
- Direct Vec indexing on `Dag::node` with topological invariant
- The provenance lens at 50 lines — under the 60-line physics-gap
  threshold

**Decisions that bent under review:**

- `Port.value_type: Option<TypeShape>` started as a two-valued type
  maintaining a runtime invariant. ChatGPT's state-space audit at M0.1
  caught it as structurally wrong: `None` conflated three meanings
  (Uninferred, Unresolved, and failed). M0.6 refactored to
  `PortState::{Uninferred, Resolved(T), Unresolved}`. The illegal mixed
  state became unrepresentable by type. One commit, no cascading
  changes. The invariant is now structural, not behavioral.
- Spans were originally tracked in a `node_spans` side table. Codex's
  "facts flow forward" audit pointed out this was the same pattern as
  v2's provenance reconstruction bug. M0.6 moved the span to a field on
  every `Behavior` variant. Side table deleted.

## Substrate constraints exercised

| Constraint | Exercised? | Notes |
|---|---|---|
| G1 target-agnostic types | Yes | `Prim::Int` stayed symbolic; no `as i64` outside the tokenizer |
| G2 cross-artifact spans | Partially | `SourceSpan.file` always populated; M0 only has one source file per compile |
| G3 parser-agnostic IR | Yes | `parse.rs` has zero `Dag` code references |
| G4 per-target cost characteristics | No | Cost lens deferred to M1 — first exercise will be emission |
| G5 no `TypeError` in `Err` | Yes | Semantic errors stay on the Dag; `CompileError::Semantic(Dag)` is a handoff, not a classification |

G4 is the one unexercised guardrail. It becomes load-bearing the moment
M1 starts emission, since the cost lens has to read per-target costs from
`LanguageSpec` declarations.

## Coproduct dissolution status

- **C1** (`Behavior`) — **kept at 5 variants**. Terminal by the 4-pattern
  check. All four dissolution patterns attempted; all failed. Pressure
  never materialized.
- **C2** (`TransformRule`) — **dissolved to nothing at M0.3**. Transform
  is `Apply(FunctionRef)`. The dissolution was triggered by Test 3
  needing `Call` and `Define` variants, and the answer was to delete
  TransformRule entirely rather than extend.
- **C3** (`TypeShape`) — **kept at 1 variant scaffold** (`Primitive(Prim)`).
  Pressure never materialized because M0 doesn't need Record/Sum/List/
  Function. Dissolution target `{connective, children}` requires std/
  Product/Coproduct declarations — deferred to M1+.
- **`Diagnostic`** — **extended from 3 to 5 variants** at M0.5/M0.8
  (added `ArityMismatch`, `ResolveError`). This is deferred dissolution:
  the 5-field target shape `{span, category, subject, detail,
  producing_node}` requires typed references (`TypeRef`, `FieldRef`)
  that don't exist yet. Extension is documented with justification at
  the top of `diagnostics.rs`.

## Reviewer feedback loops

Three review rounds, all producing correct fixes.

**Round 1 (M0.1 reviews by ChatGPT and Codex):** caught state-space
ambiguity, silent failures in inference, panic on forward references,
span drops, and linear node lookup. Triggered M0.5 (fail-closed
discipline) and M0.6 (PortState + spans + compile boundary).

**Round 2 (M0.5 review by ChatGPT):** caught unknown-type-name silent
default, branch Bool check missing, call-site signature trust when body
conflicts, recursion/Loop fabrication, test audit too narrow. Triggered
M0.8 (reviewer blockers + 7 regression tests).

**Round 3 (M0.6 review by ChatGPT):** near-identical to the M0.5 review
(likely re-read stale snapshot). The 4 items the M0.6 snapshot actually
contained were fixed in M0.8; the 3 items the review claimed were already
fixed in M0.6.

The pattern the three rounds collectively revealed: external reviewers
excel at "missing checks" — places where the compiler should reject a
program but doesn't. Internal review (my own triage) excels at "missing
tests" — coverage gaps on already-correct code. Both feedback paths are
necessary; neither catches the other's bug class.

## Late-M0 review round (blockers 1–4)

A final external review after M0.9 caught four substantive issues.
Three were real correctness bugs and the fourth was a discipline gap.
All landed in M0.10–M0.12 before M0 close:

**Blocker 1 (recursion annotated not rewritten):** the Loop wrapper
for recursive functions is structural decoration — its body still
contains literal self-referential Transform nodes (e.g., `count_down(n - 1)`
as a Transform with target "count_down"). The thesis claim that
recursion reduces to bounded iteration is half-true: the Loop exists,
but the recursive call nodes inside it are still function calls that
the substrate doesn't interpret specially. **Partial fix in M0.10**
(Loop/body reconciliation catches the fact that the body's actual type
must match the declared return type). **Residual deferred to M1**: the
self-referential Transforms inside Loop bodies are not yet rewritten to
a substrate primitive like `Recur`. A full fix connects to M1's
declaration-table restructuring — once function references are
DeclarationIds rather than strings, the self-call can become a
recognized "this is the enclosing Loop's function" rather than a
generic Transform. Flagged as a known M0 limitation in this
retrospective rather than hidden.

**Blocker 2 (pre-seeded Loop.output never reconnected):** `lower_fn_item`
pre-seeded `loop_output` with the declared return type — load-bearing
for the fixpoint, since recursive calls inside the body need the
function's Bind.value port Resolved to infer their own types. But
nothing reconciled the pre-seeded type with the body's actual return
type, so `fn bad(n: Int) -> Bool = if n == 0 then 0 else bad(n - 1)`
(where the then-branch is Int and the declaration says Bool) silently
typed as Bool at every call site. The producer-fact-reaches-consumer
property failed for recursive functions — the v2 disease recurring in
a new costume. **Fixed in M0.10**: infer's `decide()` for `LoopNode`
no longer returns Retry unconditionally. It reads the body's output
port and returns `Decision::Set(l.output, body_type)`, letting the
apply loop's existing conflict detection catch pre-seed/actual
mismatches. Regression test: `test_recursive_function_with_wrong_body_type_is_rejected`.

**Blocker 3 (mutual recursion silently accepted):** `is_recursive`
only catches direct self-calls, so `fn even(n) = odd(n - 1)` paired
with `fn odd(n) = even(n - 1)` evaded both the Loop wrap path AND
any rejection. Body inference stalled on Retry forever, leaving the
body_return_port pre-seeded value as a fake Resolved answer. The
thesis claim "termination by construction" failed silently.
**Fixed in M0.11**: a pre-lowering pass (`compute_mutually_recursive`)
builds the call graph over fn items, computes transitive reachability,
and identifies cycles of size > 1. Functions in those cycles are
rejected at lowering with a specific "mutual recursion is not yet
supported" diagnostic before any body lowering happens. Regression
test: `test_mutual_recursion_is_rejected`.

**Blocker 4 (missing dissolution ledger):** discipline gap. Four
enums — `PortState`, `LiteralValue`, `Origin`, `CompileError` —
lacked the dissolution receipts the modeling doc requires (every
kept enum names either a terminal argument or a deferred-dissolution
trigger). **Fixed in M0.12**: added receipts to each. PortState and
CompileError are terminal. LiteralValue is deferred (dissolves with
the primitive substrate refactor at M1). Origin is a judgment call —
it's a Pattern 1 candidate (redundant with Behavior kind) kept as an
enum for readability, with an explicit revisit trigger if consumer
patterns change.

**Also in M0.12: no-annotations invariant.** The user's direction
during this review round was to rule out annotation mechanisms at
every layer of the stack through M3. `INVARIANTS.md` §"Modeling
Faithfulness" now extends "Annotations are not facts" (which already
covered `.dag` source level) to compiler data model level and lens
level. Lenses are pure functions from `&Dag` to derived values, not
annotation mechanisms that write back to nodes. Concrete implication:
the Dag carries only Port/Behavior/Declaration/diagnostics, with no
annotation tables, no attribute maps, no lens-result side storage.
The success bar for new lenses is "pure function, zero substrate
modifications, its own file." This rule is ruled out now because the
core language is still being discovered — allowing annotations as an
escape hatch would fill real gaps with a decorative layer and make
the gaps invisible.

## The primitive substrate gap (parallel-representation debt)

M0 built the control-flow substrate (L1 behaviors, Ports, diagnostics)
to completion, with dissolution discipline applied at every checkpoint.
The **data substrate** — specifically, how types and primitive
operations are represented — was scaffolded rather than completed, and
the scaffold is in the wrong shape.

**The scaffold:**

- `TypeShape::Primitive(Prim)` where `Prim = Int | Bool | String` — a
  flat enum in `types.rs`
- `FunctionRef { name: String }` — a string wrapper in `dag.rs`
- `primitive_signature(name: &str) -> Option<Signature>` — a hardcoded
  Rust function in `infer.rs` mapping operator names to type signatures

**Why this is the wrong shape**, even though each scaffold is honestly
marked as a scaffold: it is a **parallel representation** of facts that
already live canonically in v2's `dsl/std/`. v2 declares Int, Bool,
String and their operations in `.dag` source. v3's scaffold re-asserts
the same facts in Rust code, creating a single-authority violation
the modeling discipline forbids. Every primitive in v3's table is an
unnecessary second source of truth.

Additionally, `FunctionRef { name: String }` is name-based dispatch —
the M8 pattern the modeling discipline explicitly rejects. The
substrate's notion of "primitive" is an opaque label, not a reference
to a structurally-rich declaration. `primitive_signature()` is literal
string-dispatch inside the compiler — exactly the pattern v3 is
supposed to cure.

**What the correct shape looks like:** a single declaration table
owned by the Dag, where every named thing (types, functions,
operations, eventually algebras and effects) is a `Declaration`.
References are `DeclarationId`s, not strings. Primitives are
Declarations pre-populated at `Dag::new()` (bootstrap); user code
declarations join the same table via the same mechanism; the
substrate doesn't distinguish "primitive" from "user function" —
both are Declarations, only their source differs (hardcoded bootstrap
at M0, parsed from `std/` at M1, parsed from user source at M2+).

**Debt classification:** this is **parallel-representation debt**,
not **dissolution debt**. It's not a coproduct that needs to collapse
into a better shape with fewer variants — it's a scaffold whose
authority should live elsewhere. The fix is "delete the parallel copy
and consume the canonical source," not "collapse variants." The
control-flow substrate (five behaviors, ports, diagnostics) does not
have this debt; the data substrate has it in three localized places.

**Why this happened:** M0's scoping decision focused on the
control-flow substrate (L1 behaviors, Port state, fail-closed
discipline) and deferred the data substrate to M1 under the framing
"we have to start somewhere." That framing was too narrow. The data
substrate is an equally important part of the thesis (single source
of truth, no parallel representations, name-based dispatch forbidden),
and building parallel scaffolds — even honest scaffolds — created
migration debt. The scaffold discipline ("mark it as a scaffold, name
the dissolution target") is not a substitute for the single-authority
principle when a canonical source already exists.

**Resolution path:** M1's **first** task before any other M1 work
(cost lens, emission, LanguageSpec) is the primitive substrate
restructuring — replace `Prim`/`FunctionRef`/`primitive_signature`
with a `Declaration` table on the Dag, consumed via `DeclarationId`
references. The primitives stay hardcoded at first (at `Dag::new()`
via a `bootstrap_primitives` pass), then M1 replaces the bootstrap
with std/ parsing. The substrate doesn't change during the std/
wiring — only the source of the Declarations changes. This sequencing
puts the restructuring under pressure from real consumer requirements
(the cost lens needs to read per-declaration algebra, emission needs
to read per-declaration cost characteristics), which is the right
place for it. See `ROADMAP.md` §M1 for the explicit ordering.

**Lesson for future substrate work:** when there's an existing
canonical source for a class of facts, the substrate should consume
the canonical source from the start, not build a parallel
representation as a scaffold. Parallel representations create
migration debt, and migration debt compounds. The "honest scaffold"
discipline is not a substitute for the single-authority principle —
a scaffold that captures the wrong abstractions creates the same
debt as a non-scaffolded parallel representation would.

## Deferred limitations

| Item | Reason | Target |
|---|---|---|
| Full descent analysis (lexicographic, non-subtraction, structural) | M0 partial analysis rejects the obvious cases; full analysis is 2–3 hours of substantive work | M1+ |
| Diagnostic 5-field target shape | Requires typed references (TypeRef, FieldRef) not yet in substrate | M1+ |
| `TypeShape` dissolution to `{connective, children}` | Requires std/ Product/Coproduct/Function declarations | M1+ |
| Mutual recursion detection | Current `is_recursive` finds only direct self-calls | M1+ |
| Imperative fixpoint loop with `changed: bool` | Sketch-mode OK; the recursive form fights Rust's lack of TCO | M3 port attempt |
| Counter-based `NodeId` allocation | Sketch-mode OK; dissolves in .dag via fold-accumulator | M3 port attempt |
| Property-based invariant testing | Needs proptest wiring; valuable but not urgent | M1+ |
| Full structural pre-infer/post-infer phase split on Dag | M0.6 refactor made the biconditional structural enough; full phase split is a larger change | M1+ |

## What M1 inherits

- A working `tokenize → parse → lower → infer` pipeline, 35 acceptance
  tests
- Five L1 behaviors with structural spans, central Port table, PortState
  enum
- Fail-closed compile boundary (`Err(CompileError::Semantic(Dag))`)
- `mark_unresolved` as the single authority for port state transitions
- Provenance lens at 50 lines (the v3-vs-v2 proof point for "lenses read
  physics")
- Partial descent analysis (zero-arg rejected, syntactic `param - const`
  required)
- Producer-fact-reaches-consumer path: call sites read the function's
  `Bind.value` state, so body/signature mismatches propagate to every
  call site
- User function signature registry (`dag.signatures`) that breaks
  inference fixpoint cycles for recursion
- Sketch-mode framing in ROADMAP so the functional-vs-imperative question
  stays answered
- 10+ feedback memories encoding the review discipline, checkpoint
  patterns, and dissolution rules
- v2's test suite available as an oracle for equivalence checks when M1
  emission starts

## What M1 has to build fresh

**M1 ordering matters.** The primitive substrate restructuring comes
FIRST, before cost lens or emission, because it's substrate work that
everything else builds on. See the "primitive substrate gap" section
above for the rationale.

1. **Primitive substrate restructuring (first M1 task).** Replace the
   parallel-representation scaffolds (`Prim` enum, `FunctionRef`-as-
   string, hardcoded `primitive_signature` match table) with a
   `Declaration` table on the Dag, consumed via `DeclarationId`
   references. Primitives remain hardcoded at `bootstrap_primitives`
   time. No std/ parsing yet — that comes after.
2. **std/ parsing.** Replace the bootstrap with a std/ parse pass.
   Declarations now come from `.dag` source rather than Rust code.
   The substrate doesn't change during this swap — only the source
   of the Declarations changes.
3. **Cost lens (first reader lens that returns computed values).**
   Originally framed as "writer lens #1" — that framing is historical;
   see the postscript at the top of the Success bar section. The
   cost lens is a pure function over substrate + `rust.dag`, not a
   persistent writer. Must land with zero substrate modifications
   beyond whatever the Declaration table needs for algebra/cost
   metadata.
4. **Emission to a target language (Rust first).** Only after the
   cost lens proves the substrate is extensible. `LanguageSpec` with
   per-target cost characteristics (G4 guardrail exercised).
5. **Ownership/effect lenses** if the M1 test corpus forces them.
6. **More sophisticated descent analysis** if any M1 test program
   needs it (current partial descent rejects the obvious bad cases;
   lexicographic orderings and non-subtraction measures are M1+).

## Success bar: trivial to add new analyses

The v3 thesis claims that adding a new analysis (a new lens over the DAG)
should be trivial — proportional to the analysis's conceptual complexity,
not proportional to substrate modifications required. v2 failed this bar
catastrophically: `complexity.dag` was 5000 lines of provenance
reconstruction because v2 had thrown away origin information during
inference. v3's equivalent is 50 lines because the facts are carried
forward.

**Status at end of M0: validated for observational lenses (read-only).**
Two independent data points:

- **Provenance lens** (M0.4) — 50 lines, one method
  (`Origin::origin_of(port)`), reads `produced_by` and dispatches on
  the producer's behavior kind. Zero substrate modifications.
- **Depth lens** (post-M0.9 follow-up) — 66 lines including 20 of doc
  comment, one method (`DepthLens::depth_of(port)`), walks ports
  backward through input chains and returns the longest path to any
  leaf. Zero substrate modifications. Three tests pass, including
  Branch-path-max composition.

Two lenses both landed in tens of lines each with zero substrate
edits. The bar holds for observational lenses by demonstration, not
just by design claim.

**Question resolved post-M1(2.7): there are no writer lenses.**

> **Postscript (2026-04-15).** The question below — "how does a lens
> store its results?" — is now closed. The answer is that the question
> itself was the wrong framing. Per the minimality invariant that
> landed in the M1(2.7) review cycle, **lenses are pure readers over
> substrate + language spec; they do not persist state and do not
> need a storage mechanism.** The four options enumerated below (Port field, per-lens
> side table, Port annotations, generalized diagnostic table) are all
> rescinded — none of them land. Cost, ownership, effects, purity, and
> any future lens are all pure functions of `(substrate, lang_spec)`,
> recomputed on demand. Memoization, if ever needed for performance,
> is a transparent local cache added later based on profiling — not
> part of the lens contract and not a substrate-level decision. The
> M0 retrospective's framing ("M1 will force the decision") was
> correct that M1 would force the question; it turned out the forcing
> resolved the question by dissolving it, not by picking an option.
>
> This section is retained as **historical context** showing what M0
> thought the open question looked like. Future readers should treat
> the option list below as a snapshot of M0-era thinking that does
> NOT reflect current doctrine.

**M0-era framing (historical, superseded by §12.6).** M0 has no
writer lenses. The mechanism for "how does a lens store its results"
was not yet committed at M0 time — there were three or four plausible
options, each with different cost profiles:

- **Option A:** add a `cost: Option<Cost>` (or similar) field to Port.
  Works, but privileges the cost lens and doesn't generalize. Port
  grows unbounded.
- **Option B:** each lens owns a side table `HashMap<PortId, LensResult>`.
  Works, cheap, doesn't privilege any lens. Cross-lens queries require
  threading multiple tables.
- **Option C:** general-purpose annotations on Port —
  `annotations: HashMap<LensName, LensValue>`. Unified place, cross-lens
  reads uniform, but introduces questions about invariants across
  annotations.
- **Option D:** generalize the diagnostic table pattern. Rename
  `DiagnosticTable` to something like `PortAnnotations`, with
  `Diagnostic` as one annotation kind among many. Most ambitious and
  most thesis-aligned — everything is an annotation, diagnostics are
  just the annotations that cause compile failure.

**Resolution (M1(2.7)+):** Option **E** — none of the above. Lenses
are readers. The storage question is moot because there is nothing
to store. The minimality invariant (pure-reader lenses, no
persistent state, memoization is a transparent local concern if
ever needed) is now doctrine.

**M1 forcing function (historical context).** The M0 retrospective
originally expected the cost lens to force the storage decision
during M1. What actually happened: the minimality invariant arrived
first (M1(2.7) review cycle), and the storage question dissolved
rather than resolving in favor of a specific option. The forcing
function worked — it just forced a re-framing of the question rather
than an answer within the original frame.

**Explicit acceptance criterion for M1:** by the end of M1, the following
question must have a confident answer — *"If we came up with a new lens
tomorrow, what's the minimum line count of substrate modifications
needed?"* The target answer is **zero**. Any other answer is a
substrate gap that must be closed before M2.

The agent should attempt to build the cost lens as a new file
(`lens_cost.rs`) that imports from `dag.rs` and reads the DAG, with no
modifications to Port, Behavior, Dag, or any substrate type. If the
agent hits a wall where it needs to write results and no existing
place works, that's the signal to pause and design the lens-storage
mechanism explicitly, once, as a substrate-level commitment.

## One meta-observation

M0 is compressed relative to typical project milestones — 10 commits,
38 tests, 3 review rounds, 1 structural refactor, 2 observational
lenses. The shape is the right shape for a mature project (substrate
validation → hardening → feature work → parallelized extension), just
faster. The review-and-respond loop is now a working feedback mechanism:
external reviewers catch missing checks, the agent produces correct
fixes, and the cycle produces code that tightens under pressure rather
than loosening.

The theory work for M0 is done. The closure work (the retrospective,
this document) closes the milestone. M1 should feel like
implementation, not design — except for the one design question M0
explicitly defers to M1: how lenses store results. **(Post-M1(2.7)
update: this deferred question resolved by dissolution — lenses
don't store anything. See the postscript at the top of the "Success
bar" section above.)**

## M1(2.5) addendum — substrate rework (historical snapshot)

> **Historical note.** This section captures the substrate rework
> at the M1(2.5) handoff. Several concrete details here (the embedded
> fixture strings, `inject_primitive_operators`, the initial 42-green
> test count) were explicitly transitional and were removed during
> M1(2.6). For current v3 status and test counts, read
> [`ROADMAP.md`](../../ROADMAP.md) — the retrospective is closed
> history.

M1(2.5) supersedes the scaffolded primitive substrate from M0.1/M1(1).
`LiteralValue`, `FunctionRef { name: String }`, `Signature`,
`primitive_signature`, `register_signature`, and `lookup_function` are
all deleted. Their replacement is the six-variant `TypeConnective` enum
(`Atom`, `Conj`, `Disj`, `Arrow`, `Cardinality`, `Instantiation`) carried
by `Declaration`s in a new `Dag.declarations` table, plus `ArrowBody`
with `UserDefined(NodeId) | ExternalRealization(DeclarationId) | Pending`
(M1_DESIGN.md §3, §Q7).

**At the M1(2.5) handoff** (superseded by M1(2.6), see ROADMAP):
bootstrap (`src/v3/compiler/src/bootstrap.rs`) parsed four minimal
fixture modules (`logic`, `bit`, `algebra`, `types`) from embedded
strings at `Dag::new()` time, and injected primitive operator Arrow
declarations so user-code dispatch (`1 + 2`) worked without a §8.9
inhabitance walk. Both mechanisms were explicitly scaffolded and both
were removed in M1(2.6) — bootstrap now consumes the real
`dsl/std/*.dag` files via `include_str!`, and operator dispatch goes
through `infer::resolve_operator_arrow`. See ROADMAP §"M1(2.6)" for
the current shape.

**Handoff test count** (M1(2.5) only, superseded):
M0 test count grew from 38 to 40; the M1(2.5) PR kept all 40 green
and added two new substrate tests (`parse_std_algebra_and_walk_int_add`
and `parse_synthetic_service_all_layers`) for a total of 42 green
tests. Every subsequent push to PR #445 (the M1(2.5)/M1(2.6) PR)
added more tests — see ROADMAP §"Status at a glance" for the current
count.
