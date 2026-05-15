# Path B for Tokenize + Parse — Brief Set (No Workarounds)

**Status:** DRAFT — pending operator review of split + scope before individual brief files dispatch under deep-wolf-155.

**Authority:** Operator directive 2026-05-15 — "could we try to do path B for tokenize/parse (NO workarounds) now — i'd like to spawn workers under you directly so we can discuss (without so many layers)."

**Routing:** Workers spawn directly under `deep-wolf-155` (PM). No Director/Mgr layer. Workers report findings + blockers directly to PM in the conversation thread; PM iterates with operator as needed.

## What this brief set targets

True 100% `.dag` retirement of tokenize + parse — including the 6 codegen-driver hand-Rust files (~2,707 lines) + the 6 test harnesses (~1,797 lines) currently still hand-Rust. Per the modeling doc at `docs/r3-retirement-modeling-emit-rs.md` §3 / catalogue `docs/r3-rust-retirement-catalogue.md`, this is Phase 5-equivalent retirement scoped down to tokenize/parse only.

**No workarounds** — operator explicitly asked to pursue the substrate-language features properly rather than fold-with-accumulator / explicit concat / skip-rustfmt fallbacks. The substrate-language matures as a side effect.

## Dispatch graph

```
Phase 0 (parallel, no dependencies):
  Brief 1 — Substrate-language: Generic methods (non-endomorphic map + per-method type params)
  Brief 2 — Substrate-language: String templating + conversion primitives
  Brief 3 — Substrate-language: Char-class structural completion (finish in-progress scaffold)

Phase 1 (parallel, no dependencies on Phase 0):
  Brief 4 — Host effects: File I/O bundle (read + write)
  Brief 5 — Host effects: Process spawn (for rustfmt)
  Brief 6 — Meta-circular: compile_to_dag foreign-function bridge

Phase 2 (depends on Phases 0+1 substantially landing):
  Brief 7 — Tokenize codegen driver authoring (.dag replacement for regen_tokenize.rs)
  Brief 8 — Parse tables codegen driver authoring (.dag replacement for regen_parse_tables_emit.rs)
  Brief 9 — Parse codegen driver authoring (.dag replacement for regen_parse_emit.rs)
```

Briefs 1-6 are substrate-language work. They're not specific to tokenize/parse — once landed, multiple other retirement paths unblock (Phase 2 test-harness dissolution, Phase 3 lens-as-Rust dissolution, Phase 5 emit retirement). **The substrate-language gaps are the actual bottleneck, not the tokenize/parse codegen drivers themselves.**

Briefs 7-9 cannot start until enough of 1-6 lands. Workers in Phase 2 will hit walls if their substrate prereqs aren't there; that's the point — surface gaps as they appear, iterate.

---

## Brief 1 — Substrate-Language: Generic Methods on FreeMonoid<T> (non-endomorphic map + per-method type params)

**Owner**: deep-wolf-155 direct dispatch.

**Source of blocker**: `dsl/std/algebra.dag:387-393` comment explicitly names this gap:
> `map: endomorphic fn(T) -> T -> FreeMonoid<T> (not fn(T)->U with U free)`
> `fold: monoid-shaped on T — init: T plus step fn(T, T) -> T -> T`
> `(Those two stay on T because executable authority here cannot yet name per-method result/accumulator type parameters — same emitter gap as lattice lifting above.)`

**Scope**:
- Investigate: what's the substrate-language gap? Is it parser-level (per-method type param syntax not recognized), lower-level (lowering doesn't bind per-method type variables), infer-level (inference doesn't propagate per-method type parameters), or emit-level (emit can't render generic monomorphizations)?
- Land: per-method type parameter support so `FreeMonoid<T>.map<U>(fn(T) -> U) -> FreeMonoid<U>` works end-to-end (parse → lower → infer → emit).
- Land: corresponding fix for `FreeMonoid<T>.fold<Acc>(init: Acc, fn(Acc, T) -> Acc) -> Acc`.

**Deliverables**:
1. Investigation report (markdown or commit message) naming the exact pipeline stage where the gap lives.
2. Substrate-language change PR landing per-method type params on FreeMonoid methods + analogous parametric carriers.
3. Test fixture demonstrating non-endomorphic map (`.dag` program lowers + infers + emits cleanly).

**Acceptance criteria** (substrate-fact-at-HEAD):
- `cargo test -p v3-compiler --test integration generic_method_type_params_test` passes.
- A `.dag` fixture in `dsl/std/test/` or analogous demonstrates `List<Int>.map<String>(int_to_string)` and the lens-fold compiles + executes correctly.
- The comment at `dsl/std/algebra.dag:387-393` is deleted in the same PR (the limitation it documents is gone).

**Risks + open questions to surface back**:
- Whether per-method type param support requires substrate-LANGUAGE changes (parser grammar extension) or just lower/infer extension.
- Whether existing call sites that already implicitly assume endomorphic map need migration.
- Whether `fold<Acc>` introduces an Acc-≠-T illegal-state class that needs Practice-2 enforcement (cf. `feedback_state_space_vs_behavioral_invariants`).

**Estimated effort**: 2-6 months substrate-language work. Investigation-first (1-2 weeks) should clarify which stage of the pipeline owns the gap.

---

## Brief 2 — Substrate-Language: String Templating + Conversion Primitives

**Owner**: deep-wolf-155 direct dispatch.

**Source of blocker**: regen_tokenize.rs has 220 `push_str` / `format!` / `writeln!` call sites. Many use templated strings (e.g., `format!("TokenKind::{label}")`). `.dag` substrate has `String = FreeMonoid<Char>` with `concat` but no `format(template, args)` function and (need to verify) possibly no `int_to_string` / `char_to_string` primitives.

**Scope**:
- Investigate: confirm what string-conversion primitives exist in `dsl/std/` today. Grep `int_to_string`, `Int.to_string`, `Char.to_string`, etc.
- Land: `fn format(template: String, args: List<String>) -> String` in stdlib (or substrate-language string-interpolation if that's preferred shape).
- Land: any missing primitive conversions (`int_to_string`, `char_to_string`, `bool_to_string`).
- Land: tests demonstrating round-trip — `.dag` program uses format to build a string, asserts result equality with concat-based equivalent.

**Deliverables**:
1. Investigation report enumerating which primitives exist + which don't.
2. PR landing missing primitives + format function.
3. Test fixture(s) demonstrating format-style templating + primitive conversions.

**Acceptance criteria**:
- `cargo test -p v3-compiler --test integration string_templating_test` passes.
- `.dag` fixture: `let msg = format("hello, {0}! count={1}", [name, int_to_string(n)])` produces expected string.

**Risks + open questions**:
- Should format-style templating be a runtime function or a substrate-language feature (compile-time string interpolation like Rust's `format!` macro)? Trade-off: runtime function is simpler; compile-time interpolation gives better diagnostics.
- Argument-index syntax (`{0}` vs `{}`) — which?
- Type-safety: should `format("{0}", [non_string])` error at lower or infer time?

**Estimated effort**: 1-3 months. Investigation might reveal this is much smaller than expected if existing primitives are richer than I assume.

---

## Brief 3 — Substrate-Language: Char-Class Structural Completion

**Owner**: deep-wolf-155 direct dispatch.

**Source of blocker**: `src/v3/compiler/tokenize.dag` §"Tracked scaffold (character-level under-consumption)" lines 23-58 names this scaffold:
> The scan phases below slice the ASCII/Unicode codepoint space in two parallel forms, neither of which consumes the character-level authorities that already exist in `dsl/std/` … `std.unicode` declares `CharClass` + `char_in_class` (canonical `.dag` authority). `ascii_scan_order` makes tokenizer scanner precedence a structural `List<CharClass>` consumed directly by `regen_tokenize`; ASCII predicate bodies remain a bounded generator bridge until `char_in_class` semantics are structurally consumed.

So `data ascii_scan_order: List<CharClass> = [Whitespace, Digit, IdentStart, IdentContinue]` exists, but `char_in_class(c, IdentStart)` is NOT structurally executable yet — predicate bodies live in `regen_tokenize.rs` as hand-Rust `push_str("is_ascii_whitespace")` etc.

**Scope**:
- Investigate: read `std.unicode` (`dsl/std/unicode.dag` or analogous) + identify what's still NYI for `char_in_class` to be structurally executable.
- Land: the missing piece(s) so `.dag` code can call `char_in_class(c, IdentStart)` at runtime.
- Land: tokenize.dag stops requiring the hand-Rust predicate bridge for ASCII class membership.

**Deliverables**:
1. Investigation report on `std.unicode` current state.
2. Substrate-language / stdlib changes landing structural `char_in_class`.
3. tokenize.dag refactor: ASCII predicate bridge deleted; `char_in_class` consumed structurally.
4. The tracked-scaffold comment at tokenize.dag lines 23-58 is deleted in the same PR.

**Acceptance criteria**:
- `cargo test -p v3-compiler --test integration char_class_structural_test` passes.
- `tokenize_generated.rs` no longer contains hardcoded `is_ascii_whitespace` / `is_ascii_digit` etc. predicate calls — they come from `char_in_class` consumption.

**Risks + open questions**:
- Whether `std.unicode::char_in_class` needs Unicode-block carrier support (per the `std.unicode` reference in tokenize.dag comments) or just ASCII subset.
- Whether the existing ASCII predicate Rust functions (`is_ascii_whitespace` etc.) should stay as a faster path with `.dag` driving choice, or be deleted entirely.

**Estimated effort**: 1-2 months. The scaffold is named + scoped; completion should be tractable.

---

## Brief 4 — Host Effects: File I/O Bundle (read + write)

**Owner**: deep-wolf-155 direct dispatch.

**Source of blocker**: regen_tokenize.rs calls `std::fs::read_to_string(dag_path)` (to load `tokenize.dag` source) + `std::fs::write(out_path, formatted)` (to write `tokenize_generated.rs`). No `.dag` substrate exists for these.

**Scope**:
- Investigate: existing `WorkflowEffect` taxonomy at `src/v3/std/effects.dag` + how the `.dag` interpreter dispatches effects.
- Land: `FileReadEffect { path: String } -> Result<String, FileError>` + `FileWriteEffect { path: String, content: String } -> Result<Unit, FileError>` substrate carriers.
- Land: `WorkflowEffect` variant additions (or sibling carrier) + idempotency classification (FileWrite is BREAKING; FileRead is IDEMPOTENT).
- Land: runtime implementation in whatever Rust harness currently executes `.dag` programs.
- Land: tests — a `.dag` program reads a file, transforms content, writes to another path.

**Deliverables**:
1. Investigation report on current effect taxonomy + runtime dispatch.
2. Substrate carrier PR (`FileReadEffect`, `FileWriteEffect`, error carriers).
3. Runtime implementation PR.
4. Test fixture demonstrating read-transform-write roundtrip.

**Acceptance criteria**:
- `cargo test -p v3-compiler --test integration file_io_effect_test` passes.
- A `.dag` fixture demonstrates: read `fixture.txt` → uppercase → write `fixture_upper.txt`. Output matches expectation.
- Effect classification (`IsBreaking` for FileWrite, `IsIdempotent` for FileRead) is structurally enforced via `EffectShape`.

**Risks + open questions**:
- Where the effect-execution boundary lives — is `.dag` the runtime, or is Rust the runtime and `.dag` declares effects that Rust executes? The architectural choice ripples downstream.
- Error taxonomy: how detailed should `FileError` variants be (NotFound / PermissionDenied / IoError catch-all / ...)?
- Whether the substrate enforces filesystem-effect ordering at the dag-walker level (FileWrite before subsequent FileRead of the same path).
- Sandboxing: does the `.dag` interpreter have a notion of trusted filesystem paths, or do all `.dag` programs get full host filesystem access?

**Estimated effort**: 1-2 months for read + write together. May expand if effect-execution architecture turns out to be substantial substrate work.

---

## Brief 5 — Host Effects: Process Spawn (for rustfmt)

**Owner**: deep-wolf-155 direct dispatch.

**Source of blocker**: regen_tokenize.rs `rustfmt_stdout` function spawns `rustfmt --emit stdout` via `Command::new("rustfmt")`, pipes stdin/stdout, captures formatted output. No `.dag` substrate for process spawning.

**Scope**:
- Investigate: similar to Brief 4 — how does the runtime execute effects, and is process-spawn a sibling effect class or fundamentally different (longer-lived child process, stdin/stdout streaming)?
- Land: `ProcessSpawnEffect { cmd: String, args: List<String>, stdin: String } -> Result<ProcessOutput, ProcessError>` substrate carrier.
- Land: `ProcessOutput { stdout: String, stderr: String, exit_code: Int }` carrier.
- Land: runtime implementation.
- Land: test fixture demonstrating spawn of a trivial command (`echo hello`) + result assertion.

**Deliverables**:
1. Investigation report on process-spawn architecture.
2. Substrate carrier PR.
3. Runtime implementation PR.
4. Test fixture demonstrating spawn + capture + assertion.

**Acceptance criteria**:
- `cargo test -p v3-compiler --test integration process_spawn_effect_test` passes.
- A `.dag` fixture demonstrates: spawn `rustfmt --emit stdout` with `let x: i64 = 1;` on stdin → expect formatted output.

**Risks + open questions**:
- Whether stdin/stdout streaming is supported, or only one-shot capture.
- Effect classification — is process-spawn always BREAKING? Some processes (idempotent commands) might be IDEMPOTENT.
- Error taxonomy for process failures (non-zero exit, signal termination, command-not-found, etc.).
- Sandbox concerns similar to Brief 4 — does `.dag` have a notion of trusted commands?

**Estimated effort**: 2-3 months. Process spawn is structurally more complex than file I/O.

---

## Brief 6 — Meta-circular: compile_to_dag Foreign-Function Bridge

**Owner**: deep-wolf-155 direct dispatch.

**Source of blocker**: regen_tokenize.rs calls `crate::compile_to_dag(source, file)` — the Rust function that runs the full parse/lower/infer pipeline on a `.dag` source string and returns a `Dag` value. To call this from `.dag`, we need either (a) a host-bridge or (b) the full self-hosted compiler. Operator's NO WORKAROUNDS means we should pursue the structurally honest path.

**Scope**:
- Investigate the architectural choice: foreign-function-interface vs self-hosted-compiler.
  - **(a) FFI bridge**: `.dag` substrate declares `fn compile_to_dag_foreign(source: String, file: String) -> Result<Dag, CompileError>` as a foreign function; runtime calls into Rust. Pragmatic. **Does not retire the Rust compile_to_dag function itself**.
  - **(b) Self-hosted**: parse / lower / infer are `.dag`-authored; compile_to_dag is itself substrate. **This is Phase 5**.

  Operator framing: NO WORKAROUNDS suggests (b) is the right target, but (b) is the full Phase 5 (1-2 years). Pragmatically (a) unblocks tokenize/parse codegen-driver retirement WITHOUT solving Phase 5. **Surface this trade-off back to operator before committing to either path.**

- Land (assuming operator picks (a) for tokenize/parse scope): FFI substrate + runtime bridge.
- Land (assuming operator picks (b)): this brief becomes the Phase 5 brief set and scope expands dramatically.

**Deliverables (path-dependent on operator decision)**:
- For (a): foreign-function substrate + runtime + test fixture demonstrating `.dag` → calls Rust → gets Dag back.
- For (b): full Phase 5 scope expansion document.

**Acceptance criteria** (assuming (a)):
- `cargo test -p v3-compiler --test integration ffi_compile_to_dag_test` passes.
- A `.dag` fixture loads `tokenize.dag` source via FileReadEffect (Brief 4), calls compile_to_dag via FFI bridge, walks the resulting Dag, asserts a specific declaration exists.

**Risks + open questions**:
- Operator's NO WORKAROUNDS stance — is (a) FFI a workaround, or is it the structurally correct intermediate step toward (b)? Need explicit operator ratification.
- Type marshaling: how does `Dag` cross the FFI boundary? Is it a `.dag` data value already (since `Dag` is substrate)?
- Error marshaling: `CompileError` is currently a Rust enum; needs `.dag` counterpart if FFI returns it.
- Recursion concerns: if `.dag` code calls compile_to_dag on `.dag` source that itself calls compile_to_dag, can the runtime handle the recursion?

**Estimated effort**: 1-2 months for (a); 1-2 years for (b).

---

## Brief 7 — Tokenize Codegen Driver Authoring

**Owner**: deep-wolf-155 direct dispatch.

**Prerequisites**: Briefs 1-6 substantially landed (per-method type params, format templating, char-class structural, file I/O, process spawn for rustfmt, compile_to_dag bridge).

**Scope**:
- Author `src/v3/compiler/tokenize_codegen.dag` (or analogous) — the `.dag` substrate that replaces `regen_tokenize.rs`. Reads `tokenize.dag` + `std/tokenize.dag` + shared syntax authority + emits Rust source for `tokenize_generated.rs`.
- The driver itself uses the substrate-language features from Briefs 1-3 + host effects from Briefs 4-5 + meta-circular bridge from Brief 6.
- Retirement PR: `regen_tokenize.rs` deleted; `tokenize_generated.rs` is now produced by `.dag`-driven codegen.

**Deliverables**:
1. `tokenize_codegen.dag` substrate authoring.
2. Retirement PR deleting `src/v3/compiler/src/regen_tokenize.rs` (1,186 lines hand-Rust) + `src/v3/compiler/src/bin/regen_tokenize.rs` (9 lines).
3. Parity test: byte-identical `tokenize_generated.rs` produced by old hand-Rust driver vs new `.dag`-driven codegen.

**Acceptance criteria**:
- `cargo test -p v3-compiler --test integration tokenize_codegen_byte_identical_test` passes.
- Substrate-fact-at-HEAD: deleting `tokenize.dag` + regenerating produces a deterministic empty output (verifies substrate-driven, not paper-shrink).
- Substrate-fact-at-HEAD: editing `tokenize.dag` (e.g., adding a new keyword row) + regenerating produces correctly-updated `tokenize_generated.rs`.

**Risks + open questions**:
- This brief surfaces ALL substrate-language + effect gaps that Briefs 1-6 didn't anticipate. Likely: 1-2 additional substrate-language features needed (TBD what they are; workers will surface).
- The byte-identical parity test is the discriminator against paper-shrink. If parity passes via `tools/tokenize.rs.in` template-clone, the retirement fails the substrate-growth check.

**Estimated effort**: 2-4 months once Briefs 1-6 land.

---

## Brief 8 — Parse Tables Codegen Driver Authoring

**Owner**: deep-wolf-155 direct dispatch.

**Prerequisites**: Briefs 1-7 (worker may start in parallel with Brief 7 once 1-6 land; Brief 7's findings inform 8).

**Scope**:
- Author `src/v3/compiler/parse_tables_codegen.dag` — `.dag` substrate that replaces `regen_parse_tables_emit.rs` (1,284 lines).
- Retire `src/v3/compiler/src/regen_parse_tables_emit.rs` + `src/v3/compiler/src/bin/regen_parse_tables.rs` (62 lines).

**Deliverables**:
1. `parse_tables_codegen.dag` authoring.
2. Retirement PR deleting old hand-Rust drivers.
3. Parity test: byte-identical `parse_tables_generated.rs`.

**Acceptance criteria**:
- `cargo test -p v3-compiler --test integration parse_tables_codegen_byte_identical_test` passes.

**Risks + open questions**:
- `parse_tables_emit.rs` is larger than `regen_tokenize.rs` (1,284 vs 1,186 lines) — likely surfaces additional substrate-language gaps.

**Estimated effort**: 2-4 months parallel to Brief 7.

---

## Brief 9 — Parse Codegen Driver Authoring

**Owner**: deep-wolf-155 direct dispatch.

**Prerequisites**: Briefs 1-8.

**Scope**:
- Author `src/v3/compiler/parse_codegen.dag` — `.dag` substrate that replaces `regen_parse_emit.rs` (124 lines, smallest of the three drivers).
- Retire `src/v3/compiler/src/regen_parse_emit.rs` + `src/v3/compiler/src/bin/regen_parse.rs` (42 lines).

**Deliverables**:
1. `parse_codegen.dag` authoring.
2. Retirement PR deleting old hand-Rust drivers.
3. Parity test: byte-identical `parse_generated.rs`.

**Acceptance criteria**:
- `cargo test -p v3-compiler --test integration parse_codegen_byte_identical_test` passes.

**Risks + open questions**:
- This is the smallest driver. If Briefs 7+8 worked, this should be the mechanical follow-on. If it surfaces unexpected gaps, those are the LAST blocker class for codegen-driver retirement.

**Estimated effort**: 1-2 months as the final convergence.

---

## Coordination notes

**Spawning**: per operator directive, workers spawn under deep-wolf-155 directly. Use `dashboard-ops work-items create "<brief-title>"` (or operator equivalent) to dispatch. The dashboard auto-spawn poller routes the worker as a child of the issuing session.

**Reporting**: workers report findings + blockers via dashboard-message direct to deep-wolf-155 (not via Director/Mgr layer). PM iterates with operator as needed.

**Iteration expectation**: NO WORKAROUNDS means we're committing to discover substrate-language gaps as workers hit them. Each worker is expected to:
1. Run their brief's investigation first
2. Surface findings to PM
3. Re-scope if the gap is bigger than the brief assumed
4. Land in pieces (initial PR may not close the brief — partial substrate-language work + follow-up PRs is expected)

**Anti-paper-shrink discriminator (applies to all 9 briefs)**: every PR's substrate-growth must be substantive (new `.dag` types / data values / fns); no PR may satisfy a brief by moving content into `tools/*.rs.in` template files. The byte-identical parity test in Briefs 7-9 specifically guards against this — paper-shrink would satisfy textual parity trivially but the substrate-growth check would fail.

**Honest framing**: cumulative effort 6-12 months optimistic, 12-18 months realistic. Operator's "1-2 years for full Phase 5" estimate was for the whole compiler; this brief set is tokenize/parse-scoped and faster. But the substrate-language work in Briefs 1-6 unblocks broader retirement paths (Phase 2 test harness, Phase 3 lens-as-Rust, Phase 5 emit) — value extends beyond tokenize/parse alone.

## Open questions for operator before dispatch

1. **Brief 6 architectural choice**: FFI bridge (a) vs full self-hosted (b). NO WORKAROUNDS suggests (b), but (b) is Phase 5 in full and would expand this brief set dramatically. Recommendation: dispatch (a) for tokenize/parse scope; (b) is its own program.

2. **Spawn cadence**: spawn all 6 substrate-language briefs in parallel now (workers run simultaneously), or sequentially (one finishes before next starts)? Parallel is faster but risks duplicate investigation work; sequential is slower but workers learn from each other.

3. **Test harness retirement scope**: this brief set covers codegen drivers (NON_TEST hand-Rust) only. The 6 tokenize/parse TEST harnesses (~1,797 lines) are NOT in scope here; they retire via Phase 2 (Gap 11 TestClaim infrastructure). Should a Brief 10 be added for those tests, or is that out-of-scope for "tokenize/parse Path B"?

4. **Substrate-language work attribution**: Briefs 1-6 deliver substrate-language features that benefit broader retirement. Should those briefs explicitly cite their downstream beneficiaries (Phase 2/3/5) so the work attributes correctly across programs?

5. **Worker count + parallelism**: 9 briefs but probably 3-4 workers in parallel is the practical cap. Want me to specify which 3-4 to spawn first?
