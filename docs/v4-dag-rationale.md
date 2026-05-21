# v4 substrate rationale (historical archive)

**Non-authoritative.** Canonical modeling lives in the **live** `src/v4/**/*.dag`
substrate, `DECISIONS.md`, `INVARIANTS.md`, and `MODELING.md`. This file is an
**archaeological appendix** only: fenced text preserves superseded in-file
commentary captured around #3229. **Do not cite it as ratified operator text.**
Practice-4 coproduct **🟢/🟡** classifications and tracked-scaffold dissolution
triggers removed from live `.dag` bodies by strict de-prose are **authoritative
only in `DECISIONS.md` Part 6 (PR #3229)** — not here.

> For integer/float inhabitance, alias discipline, and IEEE primitive resolution,
> read `DECISIONS.md` and the current `std/*.dag` / `extdeps/languages/*.dag` headers.


## `verilog.dag` — preamble modeling notes

```
// ─────────────────────────────────────────────────────────────────────
// Modeling notes (read before validating against the anchor)
//
// SPEC, NOT LIBRARY (M3 / L-2). The anchor is IEEE 1364-2005 — the
// language specification. The set of synthesizable subsets used by any
// particular tool (Xilinx XST, Synopsys Design Compiler, Yosys), the
// vendor-specific timing libraries, and the simulation-only constructs
// (`$display`, `$monitor`, file I/O system tasks) are NOT modeled: an
// invocation of `$display` is just a system-task call (a Symbol-named
// call site); modeling the system-task universe is infinite and the
// wrong layer. SystemVerilog (IEEE 1800) extensions (interfaces,
// always_comb / always_ff / always_latch sugar, packed structs, classes)
// are scope-negative for this file — they are a later language model.
// The 1364-2005 surface is the closed L-2 fidelity authority.
//
// DECLARATIVE MODEL, NOT A RECOGNIZER (T-4 authoring contract). This
// file is the grammar/type/semantics-skeleton as DATA. It declares no
// `parse`/`emit` procedure: those are the GENERIC walker (compiler
// 01_tokenize / 02_parse / 05_emit, B2-OMNI) reading this model. The
// expression sub-grammar (operators, function calls, concatenation,
// system functions) is owned by the bidirectional `LanguageModel`
// substrate that the T-4 bundle introduces — when that shape lands,
// the Verilog statement/expression productions slot into the Verilog
// `LanguageModel` instance; the closed classifiers + Conj records
// declared here are the irreducible structural carrier that does not
// prejudge that shape. Lexical-surface bodies of always-blocks,
// continuous-assign right-hand sides, and control-flow conditions are
// carried as verbatim `String` lexemes at this phase (the spice.dag
// `SpiceValue.lexeme` precedent — `parse ∘ emit` is identity on the
// textual surface; the sub-grammar interprets the lexeme when the
// expression model lands).
//
// IN-B PROBE FINDING (the deliverable — the STOP discipline is the
// product). Verilog HW concurrency models as a plain `LanguageModel`
// over the closed 5 L1 behaviors with NO 6th `Concurrent` / `Parallel`
// behavior and NO core/pipeline change. The probe's stress axis — the
// concurrent semantics of `always` / continuous-assign / module
// instantiation — resolves into ORDINARY Node structure, surfacing
// the right intra-vs-cross-program partition along the way. The
// mapping, member-by-member:
//
//   - MODULE BODY ↦ Conj of parallel siblings.
//     A Verilog `module` body is a flat list of items
//     (`ModuleItem` — continuous-assigns, always-blocks, instances,
//     declarations). Concurrent execution across those siblings is
//     STRUCTURAL: each item is a child of the module-body Conj, and
//     concurrent semantics is the SAME shape as a Conj of submodule
//     instantiations (which every C / Rust / Python module already
//     models as Conj children). The simulator/synthesizer schedule
//     (event-driven evaluation) is the mechanical realization of the
//     dependency graph (THESIS:18 "parallelism is the default —
//     independence is visible in the structure"); it is NOT a
//     separate behavior. Tempting alternative: a `Concurrent` behavior
//     wrapping each module item. REJECTED — it would parallel-
//     represent what the Conj's children already say
//     (feedback_no_annotations: a property structurally visible is
//     never duplicated as a marker). This is precisely the IN-B win.
//
//   - CONTINUOUS ASSIGN (`assign net = expr;`) ↦ Transform.
//     A continuous assignment is a function from the right-hand-side
//     expression's input signals to the left-hand-side net — exactly
//     the Transform behavior (function application, node.dag:209
//     "Transform=application"). The "continuous" aspect — re-evaluate
//     whenever an input changes — is the SCHEDULE of the surrounding
//     dataflow graph, not a property of the Transform itself.
//
//   - ALWAYS BLOCK ↦ Loop (one step) + boundary-driven iteration.
//     Per node.dag A2 footnote (genuine unboundedness — "servers /
//     streams / REPLs is a TERMINATING step iterated by the
//     coordination-boundary driver — never an unbounded `.dag` loop"),
//     each `always @(...)` block's ONE TRIGGER is a BOUNDED step
//     (one event-fire: read inputs, sequence the statements, write
//     outputs). The eternal-iteration ("forever, every clock edge")
//     lives at the simulator/synthesizer's coordination boundary; the
//     `.dag` model carries the BOUNDED step, not the unbounded
//     iteration. The trigger-event-list (`SensitivityItem`s) is the
//     bound that gates the next iteration. Tempting alternative: an
//     unbounded `Eternal` / `Concurrent` behavior. REJECTED — the A2
//     bounded-Loop discipline is the existing surface for this exact
//     pattern.
//
//   - PROCEDURAL STATEMENT SEQUENCE (within `begin ... end`) ↦ Bind.
//     The statements inside an always-block (or initial-block) form
//     a sequential composition — exactly the Bind behavior
//     (node.dag:209 "Bind=let"). Blocking (`=`) vs non-blocking
//     (`<=`) assignment is a SCHEDULING discriminator on the Bind's
//     write semantics (blocking commits before the next statement;
//     non-blocking schedules the write for the end of the time-step),
//     not a new behavior. The `ProceduralAssignKind` enum below
//     carries that discriminator as data on the assignment statement.
//
//   - IF / CASE ↦ Branch.
//     Verilog `if ... else` and `case ... endcase` are guarded sums
//     over a discriminator — exactly the Branch behavior
//     (node.dag:209 "Branch=sums"). The case-item ordering and the
//     full-case / parallel-case synthesis directives are emission-
//     time properties (the synthesizer's concern); the structural
//     surface is the Branch.
//
//   - SIGNAL DECLARATION (Wire / Reg / Vector) ↦ effect-typed value.
//     A Verilog `wire`, `reg`, or `wire [7:0]` is a TYPED signal
//     carrier; the type IS the effect-typing locus per the
//     v4-close-interrogation §16 IN-B disposition ("effects intrinsic
//     to the type signature, NOT an annotation layer"). A continuous-
//     assign Transform's domain/codomain types name Wire/Reg with
//     packing, and the typing is what constrains valid composition
//     (a `reg`-typed driver can drive an `output reg` port; a
//     procedural assignment is only legal to a `reg`-typed
//     destination; etc.). This file's `NetDeclaration` /
//     `VariableDeclaration` / `PortDeclaration` records are the
//     effect-typed carriers; there is no separate Verilog-effect-
//     annotation layer.
//
//   - MODULE INSTANCE ↦ structural Conj child.
//     `ModuleName instance_name (.port(signal), ...);` is a child
//     Node in the parent module-body Conj — the same shape as a
//     submodule reference in any other LanguageModel. Concurrent
//     execution of the instance alongside its sibling items is the
//     same Conj-of-parallel-siblings shape as multiple always-blocks
//     (above); no new behavior.
//
// INTRA-PROGRAM vs CROSS-PROGRAM EFFECT-TYPING (the partition the
// probe surfaces). The scaffold-header `Consumes` line citing
// `extdeps/coordination.dag` for "concurrency via effect-typed Bind
// — IN-B" was a SCAFFOLD-AUTHOR ANTICIPATION that lumped all
// concurrency under one carrier. The probe surfaces the structurally
// correct partition: (a) INTRA-program concurrency (multiple
// `always`-blocks within one Verilog module sharing signal nets) is
// what this file models — the effect-typing carriers are the
// signal-type declarations (`NetType` / `VariableDeclaration` variants /
// `PackingShape`)
// declared HERE, in this language file; (b) CROSS-program coordination
// (one program's output deployed to host A, another to host B,
// communicating via Endpoint / DeploymentUnit / sync-async-stream-
// pubsub) is what `extdeps/coordination.dag` (T-4.8) models — a
// distinct concern with a distinct anchor (Wikipedia Distributed
// computing + Messaging pattern). The IN-B principle "effect-typed
// Bind composition, no 6th behavior" holds in BOTH partitions, with
// the effect-typing carriers in their respective per-language /
// per-coordination home files (P2 single-authority). This is the
// per-language effect-typing finding the probe was authored to
// surface — and it surfaces here on the SUCCESS side, not the STOP
// side.
//
// INHABITANCE (DEFERRED — std/ owner, not this file). The scaffold-
// declared "Inhabitance: bit-vectors (wire/reg [N:0]) per
// std/algebra.dag" is DEFERRED until the std/ scalar wave declares the
// ratified instance-value shape in its own carriers. Conceptually, an
// N-bit unsigned signal grounds in `Semiring<UInt<N>>` (or
// `OrderedRing<Int<N>>` for signed-as-twos-complement reg/integer
// arithmetic) — but declaring a `data verilog_wire_n_semiring:
// Semiring<UInt<N>> = …` instance-value at this layer would be the
// per-file `<Algebra><Concrete>` improvisation PR #3171 held as
// precedent against. The instance-value form lands when std/integer.dag
// / std/logic.dag inhabitance walks land and the operator ratifies how
// those rows attach to language surfaces; the algebra import + the
// instance-value declarations are added in ONE pass at that owner, not
// improvised per language file.
//
// PROBE LIMITS (honest framing — what this file does NOT validate).
// The IN-B finding covers the STRUCTURAL fit of Verilog HW concurrency
// to the 5 L1 behaviors. It does NOT validate:
//   - that a synthesizer actually produces gates from `emit`'s output
//     (the "clear win" in the header — that requires invoking a
//     synthesizer process, an `extdeps/posix.dag` consumer
//     boundary, not modeled here).
//   - the expression sub-grammar of `assign net = expr;` right-hand
//     sides and procedural statement bodies (verbatim lexemes at
//     this phase; the T-4 LanguageModel bundle owns the recursive
//     expression productions when it lands).
//   - SystemVerilog (IEEE 1800) extensions (`always_comb` /
//     `always_ff` / `always_latch` sugar, interfaces, classes,
//     packed structs) — a later language-version model edits this
//     file (under the L-2 pin discipline) when it lands.
//   - Simulator-only / testbench constructs (`initial` with
//     `$display`, `$finish`, file I/O system tasks, `force` /
//     `release`, fork-join, `wait`) — `initial` is admitted as a
//     `ModuleItem` variant for the once-only-driven case; the
//     `$task` universe is the system-task call site, not the
//     concurrency primitive.
//   - Timing-checks (`$setup`, `$hold`, `specify` blocks) — a
//     separate timing-annotation layer (SDF — Standard Delay
//     Format), out of scope for this language model.
//
// PRACTICE-4 LEDGER DISCIPLINE. Every closed-Disj coproduct below
// carries the full 🟢/🟡/🔴 classification + all FIVE patterns
// attempted (Fact-placement / Variant-is-data / Algebraic-form /
// Dimensional / Parameterized-family-Practice-7) per modeling-
// discipline.md §4 (operator-ratified binding 2026-05-15, gate (b);
// PR #3163 made the 4-pattern miss the proof case — do not omit
// Practice-7). Conj records carry no ledger (only coproducts
// dissolve); scaffold-residual fields (VectorRange's lexeme-pair
// bridge to the constant_expression sub-grammar, VerilogCost's
// raw-Int axes) carry a 🟡 bridge note mirroring the std/
// diagnostic.dag / ptx Dim3 precedent.
```

## `verilog.dag` — header Stress / Clear win / Status detail

```
// Stress (T-4.9 — a B2-OMNI falsification probe): hardware CONCURRENCY vs
//   the 5 L1 behaviors. `always @(posedge clk)` and continuous assignment
//   are concurrent dataflow, NOT sequential control flow. The IN-B
//   decision (concurrency = Bind composition + effect typing, NO 6th L1
//   behavior — coordination.dag) is VALIDATED here: if Verilog cannot be
//   modeled without a 6th `Concurrent` behavior, that is a C1 stop-signal
//   escalation. Catching that now — before the substrate is load-bearing
//   — is precisely the point of adding this early.
// Clear win: ONE `.dag` FSM → simulable Verilog + a Rust reference model,
//   same Node, zero translator (hardware + software from one source).

// Status: T-4.9 modeled 2026-05-16 (bright-crane-527). IN-B probe RESULT:
//   PASS — Verilog hardware concurrency models cleanly under the existing
//   five L1 behaviors with NO 6th `Concurrent` behavior and NO core/
//   pipeline change (see modeling notes "IN-B PROBE FINDING"). The probe
//   surfaces the structurally-correct intra-vs-cross-program partition:
//   hardware concurrency (parallel `always`-blocks / continuous assigns
//   within ONE module) is the Conj-of-parallel-siblings + Bind-chain
//   shape, with per-language effect-typing intrinsic to the wire/reg/
//   vector signal type — exactly the THESIS:401 IN-B disposition.
//   The scaffold-header `Consumes` line is preserved verbatim and carries
//   two known stale citations: (1) `std/primitive.dag` was deleted by
//   PR #3152 (STRUCTURE.md §"Scalar/numeric concept decomposition"); the
//   six concept-anchored scalar files are the replacement, and `Symbol`
//   is now substrate-ambient at std/node.dag (K-1). (2) `extdeps/
//   coordination.dag` is an unfilled scaffold (T-4.8) and its modeled
//   scope is CROSS-PROGRAM coordination (Endpoint / DeploymentUnit /
//   sync-async-stream-pubsub semantics, v4-close-interrogation.md §16) —
//   a different concern from intra-program hardware concurrency, which
//   this file demonstrates needs no cross-program coordination carrier
//   at all. Both stale citations are surfaced, not edited: the contract
//   lines are immutable, and the per-file Consumes correction is
//   pending the systemic operator scaffold-Consumes reconciliation
//   (T-4 mgr synthesis msg_1fddb75c) — uniform with ptx #3170, spice
//   #3168, and llvm_ir #3171. The body's `import` reflects what this
//   file actually uses. Splitting this file requires explicit operator
//   ratification (substrate extension = stop signal).
```

## `verilog.dag` — import deferral note

```
// std/node.dag is imported: it makes the K-1 opaque `Symbol` (the
// name-reference identity used pervasively in the Verilog grammar —
// module names, port names, signal names, instance names) resolvable
// through this file's import closure. std/algebra.dag is NOT imported:
// this file's scaffold-declared `Inhabitance: bit-vectors per
// std/algebra.dag` is DEFERRED to operator-pending scalar inhabitance
// in the std/ carriers (see the Inhabitance section in modeling notes);
// declaring per-target OrderedRing/Semiring instance-values here would
// be the per-file `<Algebra><Concrete>` improvisation that P2
// single-authority forbids (#3171 worker
// vivid-crab-154 / snappy-badger-570 held precedent). std/diagnostic.dag
// is NOT imported: this declarative LanguageModel declares no
// parse/emit body — those ride the operator-pending D1 Outcome<T>
// carrier (`Produced{value:T} | Rejected{diagnostic:Diagnostic}`) when
// ratified in std/diagnostic.dag; improvising a per-file Result shape
// now is exactly the v4 value-or-Diagnostic STOP first surfaced by
// T-4.6 (keen-wren-419). `Int`, `Bool`, `String`, `List` are
// kernel-ambient per STRUCTURE.md §"Kernel-ambient types" — no import.
```

## `llvm_ir.dag` — modeling notes

```
// ─────────────────────────────────────────────────────────────────────
// Modeling notes (read before validating against the anchor)
//
// SPEC, NOT LIBRARY (M3 / L-2). Anchor = the LLVM Language Reference
// Manual, pinned at LLVM 18. The model is the LangRef grammar + type
// system + value/instruction taxonomy. LLVM intrinsics, target
// libraries, and pass-specific metadata are NOT modeled — an intrinsic
// call is just a `Call` whose callee is a named function (a program in
// the modeled language = a `Node`), exactly as M3 / the T-4 authoring
// contract requires. Modeling the intrinsic/library universe is infinite
// and the wrong layer.
//
// DECLARATIVE MODEL, NOT A RECOGNIZER (T-4 authoring contract). This
// file is the grammar/type/semantics as DATA. It declares no
// `ingest`/`emit` procedure: those are the GENERIC walker (compiler
// 01_tokenize/02_parse/05_emit, B2-OMNI) reading this model. Concretely
// this is why the file consumes neither a `Result<T, Diagnostic>` carrier
// nor std/diagnostic.dag: there is an open operator-tier STOP on the
// absence of a shared value-or-Diagnostic carrier in src/v4 (first
// surfaced by T-4.6); a declarative LanguageModel does not need it and
// must not improvise one. Fail-closed behavior off the lossless core is
// the walker's obligation; this file's contribution is to DECLARE the
// lossless core F structurally (see `FidelityDisposition`), which is
// pure node.dag Disj data — no Diagnostic, no Result.
//
// B2-OMNI PROBE FINDING (the deliverable — the STOP discipline is the
// product). LLVM IR models as a plain `LanguageModel` with NO 6th
// behavior and NO core/pipeline change. The structurally-unusual SSA
// shapes resolve to ORDINARY Node structure — and modeling them HONESTLY
// drove the right structural form:
//   - "every value defined once" (SSA) — the value graph (operands
//     reference their producers) realized the A1-FAITHFUL, FINITE way:
//     a value-producing instruction / φ carries `result: Symbol` (its
//     def-site name); an operand is a NAME-REFERENCE
//     (`LlvmValue.LocalRef`) resolved through function scope to that
//     def-site (or a parameter / a global / an inline constant). This
//     IS the scaffold's "value graph is Node edges" — node.dag A1's
//     cross-declaration idiom (references are name `Atom`s resolved
//     through the namespace; precedent: diagnostic.dag `reason:
//     Symbol`). Single-assignment is the structural fact that exactly
//     one def-site declares each name (INVARIANTS P2). CRUCIAL: the
//     def-use graph is CYCLIC for every loop (a loop-carried φ names an
//     instruction whose operand names the φ) — a name-reference keeps
//     the bounded Node tree FINITE while expressing that cycle, which is
//     PRECISELY why A1 mandates name-reference recursion. An EARLIER cut
//     INLINED the producer node into each use (`InstructionValue {
//     instruction }` / `PhiValue { phi }`); that could not finitely
//     represent cyclic loop-carried SSA — it excluded ALL loops, a
//     P1/M3 spec-narrowing and the exact A1 violation A1 exists to
//     prevent. The name-reference form is the correct finite
//     REALIZATION of the value-graph intent (operands → producers), not
//     an abandonment of it. `result: Symbol` is a
//     declaration FIELD = a name-reference (K-1-opaque, resolved
//     structurally), NOT a runtime Symbol mint — the K-1 no-mint rule
//     was specific to the removed inhabitance LAMBDAS, not declaration-
//     class fields. The textual `%x`/`%1` spelling is `DeclaredNormalized`
//     (α-equivalent — `Symbol` is opaque, the content-hash fold is
//     rename-invariant).
//   - phi — phi is an ORDINARY Node, hoisted to the block's structural
//     `phis` region (`PhiNode`), so "a phi after a non-phi instruction"
//     is UNREPRESENTABLE (P2), not a checker rule. It is NOT a variant
//     of the general instruction sum (its block-start position is
//     structural). At the computation layer phi is `Branch`-shaped (it
//     selects an incoming value by which CFG edge was taken) — it reuses
//     the EXISTING closed `Behavior::Branch`, it does not add one. Per
//     IN-B: control/data flow is Node edges; phi is a Node.
//   - dominance — NOT a field or flag (feedback_no_annotations). The CFG
//     is carried structurally (each block's terminator names its
//     successor labels — `block_successors`). "Definition dominates all
//     uses" is a LENS read over that Node graph (the same shape as the
//     complexity/effect lenses), derived, never stored; a `dominates`
//     edge would be a second authority for what the CFG already
//     determines (INVARIANTS P2).
//
// A2 TOTALITY IS A `.dag`-PROGRAM PROPERTY, NOT A MODELED-LANGUAGE ONE.
// An LLVM CFG can contain irreducible / non-terminating loops (a back-
// edge `br` to a dominating block). That is faithful to LangRef and is
// MODELED here as ordinary CFG data — std/node.dag A2 (Loop is bounded,
// total-by-construction) constrains `.dag` programs the compiler
// ACCEPTS, never the external languages it models/emits (M3: extdeps
// model real specs). `emit (.dag → LLVM IR)` only ever emits the bounded
// forms a `.dag` program can express; ingesting arbitrary LLVM whose
// termination is not structurally evident is the walker's fail-closed
// boundary (declared in F below), not a totality this file may claim.
//
// INHABITANCE IS OWNED ELSEWHERE (P2) — see the "Inhabitance — OWNED
// ELSEWHERE" section near the end. This file declares only the LLVM
// type/value/instruction VOCABULARY; WHICH algebra a scalar grounds in
// is owned by the std/ scalar wave (std/integer.dag/float.dag/logic.dag,
// T-3) + the operator-pending inhabitance-instance form, by P2 single-
// authority. This file does NOT declare, assert, or restate that
// grounding (an extdeps language file restating it is a second-authority
// P2 violation).
// Faithful observations for that std/ scalar owner (NOT this file's
// authority): LLVM `iN` arithmetic wraps mod 2^N ⇒ a finite ring
// that is NOT order-compatible (the algebra.dag Float≠Field analog);
// `fcmp` is NaN-unordered; pointers have no total numeric algebra.
//
// Every coproduct below carries the full Practice-4 🟢/🟡/🔴
// classification + the 5-pattern dissolution ledger (modeling-
// discipline.md §4 — operator-ratified binding 2026-05-15; FIVE patterns
// attempted, the fifth being Parameterized-family / Practice-7).
// Records (Conj) carry no ledger. The closed opcode/predicate enums are
// the LangRef-defined sets: an addition is an upstream LangRef revision
// (the M3 spec-anchor analog of std/node.dag's C1 stop-signal — a
// visible, ratified edit, never a silent `Other`/`...`).
```

## `llvm_ir.dag` — header Stress / C5 / narrative

```
// Stress (T-4.12 — a B2-OMNI falsification probe): does the Node
//   substrate + B2-OMNI generalize DOWN the abstraction stack, not just
//   across surface languages? SSA form (every value defined once;
//   phi nodes; dominance) is a structural shape unlike source ASTs. If
//   LLVM IR cannot be a plain `LanguageModel` without core/pipeline
//   changes, B2-OMNI is leaking — surfaced now, before load-bearing.
// Clear win: `.dag → LLVM IR` (then LLVM lowers to machine code), AND
//   `LLVM IR → Node` ingest — the down-stack half of O(N+M).
// C5-fidelity note: F here is essentially all-structural — LLVM IR has
//   negligible cosmetic surface, so `emit∘ingest` is ~identity (no
//   trivia to normalize). Contrast source models where F may include
//   comments/formatting by intent.
```

## `llvm_ir.dag` — import deferral note

```
// std/node.dag is imported: it makes the K-1 opaque `Symbol` (the
// name-reference identity used pervasively in the grammar — block
// labels, SSA/parameter/global names) resolvable through this file's
// import closure. std/algebra.dag is NOT imported: this file's declared
// `Consumes` of it is its INHABITANCE, which is DEFERRED until the std/
// scalar owner ratifies inhabitance instance rows (see the Inhabitance
// section). The algebra import + the ratified instance-value form are
// added only at that owner; importing algebra now while unused would
// be dead code, and improvising the instances now would be the exact
// per-file `<Algebra><Concrete>` form P2 single-authority forbids.
```

## `ptx.dag` — modeling notes

```
// Modeling notes (read before validating against the anchor)
//
// THE IN-B FALSIFICATION PROBE (T-4.14, B2-OMNI + IN-B, DECISIONS.md
// L-3). The brief's §D framing is binding: "the STOP discipline is the
// deliverable — if the substrate (5 behaviors / 6 connectives / fail-
// closed) genuinely can't model the target, that is a substrate
// finding to surface, not force." For T-4.14: "STOP if tempted toward
// a 6th behavior." This section records the structural fit and
// names the operator-tier triggers that would have flipped the
// finding to STOP — neither fired.
//
// PROBE OUTCOME — the 5 behaviors hold; the SIMT model fits without a
// 6th. The mapping, member-by-member:
//
//   - KERNEL ↦ Transform.
//     A PTX `.entry` is a function `(thread_coord, kernel_params) ->
//     side-effects-on-state-space`. That is exactly the Transform
//     behavior (function application — INVARIANTS:72 typed-total-λ
//     fragment, node.dag:209 "Transform=application"). The kernel's
//     body is a sub-DAG of the 5 behaviors, identical to a CPU
//     function's body — no kernel-specific behavior surfaces.
//     "Run once per thread" is the SCHEDULING of the surrounding Loop
//     (see next), not a property of the kernel function itself; the
//     Transform IS the kernel callable.
//
//   - THREAD GRID ↦ Loop.
//     The execution-of-a-kernel-across-its-grid is bounded recursion
//     over the grid's bounded thread-index cardinality (gridDim ×
//     blockDim, both Cardinality-bounded). This is exactly the A2
//     `Loop` discipline (node.dag:42-68: bounded recursion, total-by-
//     construction). Per THESIS:19 / node.dag:209 "Parallelism is the
//     default because independence is visible in the structure" —
//     iterations of the kernel-Loop over distinct thread-coords are
//     INDEPENDENT unless they communicate (through `.shared` /
//     `.global` reads/writes or barriers, both visible structural
//     edges). The SIMT schedule (lock-step warp execution) is the
//     mechanical realization of the dependency graph; it is NOT a
//     separate behavior. Tempting alternative: a `Parallel` /
//     `Kernel` behavior carrying "this Loop is data-parallel". That
//     temptation would have been the STOP trigger — but it duplicates
//     what the dependency graph already says (THESIS:19 again:
//     "sequential execution is what needs justification"). Adding
//     such a behavior would be a feedback_no_annotations violation
//     (annotating a property structurally already visible). REJECTED;
//     no STOP needed.
//
//   - BARRIER ↦ Bind.
//     `bar.sync` / `membar` join concurrent threads at a
//     synchronization point. That is exactly the Bind behavior
//     (node.dag:209 "Bind=let" — sequencing). A barrier's SCOPE (CTA
//     / GPU / system) is a Conj-record field on the Bind (the
//     `BarrierScope` classifier below), not a new behavior; same
//     pattern as `bar.sync 0` vs `membar.gl` differing only in scope.
//     Tempting alternative: a `Synchronize` behavior. REJECTED — Bind
//     is the existing sequencing primitive; "synchronize across
//     concurrent threads" is a property of the surrounding parallel
//     Loop reading the Bind, not of the Bind itself.
//
//   - STATE SPACE ↦ effect-typed parameter (structural variant
//     payload). A PTX value's `.reg` / `.shared::cta` /
//     `.shared::cluster` / `.global` / `.const` / `.local` /
//     `.param::entry` / `.param::func` residence is a structural
//     payload on `RegisterScalar`'s typed variants — specifically
//     the `state_space: RegisterStateSpace` field on
//     BitsScalar / UnsignedScalar / SignedScalar / FloatScalar
//     (RegisterStateSpace is the 6-variant user-declarable typed-
//     scalar-resident
//     sub-classifier of StateSpace, excluding `.tex` since the
//     `.tex` state space holds opaque texture/sampler descriptors
//     per PTX 8.5 §5.1.10, not typed scalars). PredicateScalar is
//     a BARE variant with NO state_space field — per PTX 8.5
//     §5.1.1 predicates are virtual predicate registers (`.reg
//     .pred`) with no memory load/store path; residence is
//     implicit `.reg`. Memory ops (`ld.global`, `st.shared::cta`)
//     are Transforms whose input/output state-space tags constrain
//     valid composition. This IS the THESIS:396 IN-B claim made
//     concrete: effects are "intrinsic to the type signature, NOT
//     an annotation layer." Tempting alternative: a
//     `StateSpaceTransfer` behavior wrapping every ld/st. REJECTED
//     — it would parallel-represent what the Transform's
//     signature already says; the effect-typed parameter is the
//     unannotated primitive.
//
//   - @p PREDICATE ↦ Branch.
//     PTX predicated execution (`@p mov.b32 r1, r2;`) is a guarded
//     Branch on the `.pred` carrier: `if p then op else skip`. The
//     `Branch` behavior already covers this (node.dag:209
//     "Branch=sums"). Predicate negation (`@!p`) is the
//     `BooleanAlgebra<.pred>.complement` instance-value — to land
//     downstream when std/logic.dag's inhabitance form lands;
//     declaring it here would parallel-represent it pre-substrate
//     (P2). For T-4.14 the structural fit is the deliverable; the
//     instance-value follows.
//
// CONCLUSION. The probe's IN-B half closes with no STOP. The B2-OMNI
// half (PTX grammar as declarative bidirectional data) hinges on
// T-4's LanguageModel shape — when that bundled shape lands, this
// file's coordinate classifiers slot into the PTX `LanguageModel`
// instance without revision (their I/O contract is the immutable
// header above).
//
// SCOPE NEGATIVE (operator-tier, do not work around): warp-level
// intrinsics (`%warpid`, `%laneid`, `%smid`), texture / surface
// references, async copies (`cp.async`), tensor cores (`wmma`,
// `mma.sync`), and the `bf16` / vector-pack types (`.f16x2`,
// `.f32x2`) are deliberately not modeled at T-4.14. They sit either
// (a) inside the bundled LanguageModel shape (grammar productions —
// T-4's work), (b) inside the cost-realization detail per release
// pin (PtxCost note below), or (c) inside the SIMT-effect-typed Bind
// substrate that coordination.dag (T-4.8) owns. The classifiers
// declared here are the irreducible SIMT coordinate set; the rest
// composes over them when their substrate lands.
//
// PRACTICE-4 LEDGER DISCIPLINE. Every closed-Disj carrier below
// carries the full 🟢/🟡/🔴 + all five patterns (Fact-placement /
// Variant-is-data / Algebraic-form / Dimensional / Parameterized-
// family-Practice-7) per modeling-discipline.md §4 (operator-ratified
// binding 2026-05-15, gate (b) — #3163 made the 4-pattern miss the
// proof case; do not omit Practice-7). Conj records carry no ledger
// (only coproducts dissolve); scaffold-residual Conj fields (Dim3
// raw Int offsets) carry a 🟡 bridge note mirroring std/diagnostic.dag's
// `Extent.ByteRange` precedent.
```

## `ptx.dag` — header IN-B headline / Owned-elsewhere detail

```
//        (forced by DECISIONS.md L-3 — PTX is the SIMT *IR*; CUDA-C++ is
//        a Shape A source already covered by general modeling, so it
//        would not be a down-the-stack probe at all): the SIMT execution
//        model (grid / block / thread), the memory hierarchy
//        (register / shared / global / etc.), kernels and barriers.
// Anchor: NVIDIA "Parallel Thread Execution ISA Version 8.5" —
//         https://docs.nvidia.com/cuda/parallel-thread-execution/index.html
//         (release PDF: https://docs.nvidia.com/cuda/pdf/ptx_isa_8.5.pdf,
//         CUDA Toolkit 12.5). This is the L-2 versioned fidelity authority
//         the carriers below are pinned to (BarrierScope's `.cluster`,
//         ThreadCoordSource's cluster hierarchy, and StateSpace's eight
//         memory regions are derived from this release; a future ISA
//         release modifying any of them is a visible C1 edit). The
//         "CUDA C++ Programming Guide" documents the SIMT execution
//         model the spec encodes.
//
// IN-B FINDING (the T-4.14 probe's deliverable, operator-ratified
//   STOP-or-find discipline per §D): SIMT data-parallelism IS modelable
//   as the existing 5 L1 behaviors — NO 6th `Parallel` / `Kernel`
//   behavior is required. The full mapping lives in the modeling notes
//   section below; the headline:
//     kernel        → Transform (a function over (thread_coord, params))
//     thread grid   → Loop (bounded recursion over the grid's bounded
//                     thread-index cardinality; THESIS:19 "parallelism
//                     is the default" — independence is read from the
//                     dependency graph, scheduling is mechanical)
//     barrier       → Bind (sequencing synchronization point)
//     state space   → effect-typed parameter (a structural Conj field
//                     on the value, per THESIS:396 IN-B "intrinsic to
//                     the type signature, not an annotation layer")
//     @p predicate  → Branch (predicated execution = guarded Branch on
//                     a `.pred` carrier)
//   The 5 behaviors hold; the probe surfaces no substrate gap (no STOP).
//
// Owns:
//   - PTX coordinate classifiers as closed structural carriers:
//     StateSpace (memory hierarchy — with ParamScope / SharedScope
//     sub-qualifiers structurally enforced on `.param` / `.shared`
//     per PTX 8.5 §5.1.7-8 / §6 memory-operand grammar),
//     BarrierScope (CTA/cluster/GPU/sys),
//     ThreadAxis (X/Y/Z), ThreadCoordSource (thread / CTA-in-cluster /
//     cluster / CTA-in-grid hierarchy per PTX ISA 8.5),
//     BitsWidth / IntegerWidth / FloatWidth (per-kind admissible-
//     width sub-enums — kind × width admissibility is structurally
//     enforced by RegisterScalar's Disj-with-payloads below, not
//     deferred to a check), RegisterScalarKind
//     (Bits/Unsigned/Signed/Float/Predicate)
//   - PTX value shapes: RegisterScalar (Disj-with-heterogeneous-
//     payloads, kind × admissible-width × admissible-residence
//     STRUCTURALLY enforced — see RegisterScalar declaration);
//     Dim3 / ThreadCoord (Conj records — Dim3 carries the 🟡 scaffold
//     bridge to std/cardinality.dag, same as std/diagnostic.dag's
//     ByteRange); ThreadHierarchyShape (Disj over launch shapes:
//     FlatLaunch | ImplicitClusterLaunch | ExplicitClusterLaunch
//     per PTX 8.5 §11 + `%is_explicit_cluster`)
//   - PTX cost-realization carrier shape: PtxCost (per-instruction +
//     occupancy components — the per-target cost projection U1 reads)
//   - the IN-B mapping (modeling notes): SIMT → 5 behaviors, no 6th
//
// Owned ELSEWHERE (do not duplicate here):
//   - the bidirectional LanguageModel substrate (grammar productions,
//     C5-fidelity disposition tags, ingest∘emit roundtrip carrier) — the
//     BUNDLED work of T-4 across the 5 Shape A source languages, where
//     "the SHAPE is the work" (TASKS.md T-4 authoring contract). When
//     that shape lands, this file's classifiers slot into the PTX
//     `LanguageModel` instance; the classifiers themselves do not
//     prejudge that shape.
//   - PTX integer / float scalar grounding (`.u32` inhabits `Semiring`,
//     `.s32` inhabits `OrderedRing`, `.f32` inhabits `ApproximateField`)
//     — flows from std/integer.dag / std/machine.dag / std/float.dag
//     (T-3 wave A3) when those carriers + their algebra instance-values
//     land. The PTX register-type vocabulary (kind × width) declared
//     here is the SURFACE through which that grounding parameterizes.
//     Pre-T-3-A3 this file declares no algebra instance-value, by P2
//     single-authority: re-declaring an inhabitance whose carrier file
//     is unwritten would foreclose its eventual authority.
//   - SIMT-as-Bind composition's effect-typing carrier — that is the
//     subject of extdeps/coordination.dag (T-4.8); the IN-B finding
//     here demonstrates the mapping fits the 5 behaviors, and
//     coordination.dag (when it lands) owns the effect-typed Bind
//     carrier the kernel-as-Transform composes through.
//
// Consumes:
//   - nothing — this file is closed-enum + Conj-record declarations
//     only. `Int` is kernel-ambient (Dim3 component; the per-kind
//     width sub-enums BitsWidth/IntegerWidth/FloatWidth are closed
//     enums, not Int-carrying). No algebra instance-values are declared at this
//     phase (see "Owned ELSEWHERE" — PTX scalar inhabitance is
//     downstream of T-3 wave A3 + T-4's LanguageModel shape).
//   (The scaffold header's prior `std/node.dag, std/algebra.dag,
//   std/primitive.dag` Consumes line predated PR #3152's
//   std/primitive.dag decomposition AND the operator-ratified §0
//   scope-narrowing of this probe to "closed classifiers + IN-B
//   finding"; the revised Consumes line above reflects what the file
//   actually imports, not the scaffold's anticipatory list. Header
//   Consumes change pending operator ratification of the systemic
//   scaffold-Consumes reconciliation (T-4 mgr synthesis
//   msg_1fddb75c) — the surface+trigger discipline lets the
//   operator ratify all per-file Consumes strikes uniformly in one
//   pass; this is not worker-settled.)
```

## `integer.dag` — superseded commentary (archive)

```
// Historical capture only. Canonical: src/v4/std/integer.dag + DECISIONS.md + INVARIANTS P1:42.
// Single std/ authority for integer algebra inhabitance; Nat → GroupCompletion<Int>;
// fixed widths as Compose with std/machine widths; Tier-2 divide/modulo via Outcome;
// intrinsic Symbol rows for divide-by-zero diagnostics; per-language extdeps aliases
// to Int8…UInt128 — no parallel numeric ring substrate in std/.
```

## `float.dag` — superseded commentary (archive)

```
// Historical capture only. Canonical: src/v4/std/float.dag + DECISIONS.md D3 + INVARIANTS P1/P3.
// Float32/64 nominal wrappers over FloatBody; ApproximateField<Float> data witness
// deferred until grounded primitive bodies exist; IEEE primitive ops realized under
// extdeps/languages/* per THESIS:203 open-system path; Tier-2 NaN compare uses
// intrinsic Symbol rows (diagnostic.dag / node.dag K-1 name-reference idiom).
```
