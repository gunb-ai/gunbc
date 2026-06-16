# Design: Omni-Emission Host Transport — descriptor data, one fold, one primitive

> **Status: DESIGN — operator priority (2026-06-10, via still-raven-546).** Dissolve the
> `run_emit_host_<lang>` hand-list: per-target host transport (how a target compiles, runs,
> and speaks runtime values) becomes **modeled data** in `extdeps/languages/`, consumed by
> **one** general transport fold (`find_witness`, fail-closed), backed by **one** generic
> process primitive in Rust. The descriptor must serve omni-**ingestion** too — the
> bidirectional symmetry is a design obligation, not a later wish.
> Supersedes B3 of the TS-skeleton brief (the TS run routes through a descriptor row, not a
> fourth hand function).
>
> **Endorsed by still-raven-546 (2026-06-10) with rulings on all four escalated questions
> (§6).** Build sequencing per the endorsement: model + rows + primitive land first, the TS
> round-trip is proven through the descriptor, and only then does the hand-list deletion
> (§4 step 4) get its explicit go — it is load-bearing.

## 1. Problem (measured hand-list, 2026-06-10)

Adding emit-host support for one target today costs **five hand edits across three layers**,
none of them data:

| Layer | Hand-list site | Shape |
|---|---|---|
| Rust runner | `tools/emit_host_runner/src/lib.rs:443-571` — `run_emit_host_rust` / `_python` / `_go` | three near-identical bodies: write fixture file(s) → spawn command(s) → bounded capture → `host_exit_from_bounded` → receipt. Only the file names and command lines differ. |
| Stdout parse | `lib.rs:217-237` — `runtime_value_parse_rust` / `_python` / `_go` | `_python` and `_go` literally call `_rust` (five-byte check) — pure duplication |
| `.dag` dispatch | `src/v2/compiler/emit_host.dag:234-250` (`run_emit_host`) and `:185-197` (`runtime_value_parse`) | if/else chains on **string equality against `authority_source_text` pins** (`:79-83`) — dispatch keyed on a source-text spelling, not an identity |
| Eval hooks | `src/v3/compiler/src/emit_host_eval.rs:45-302` — `try_dispatch_emit_host_rust` / `_go` / `_python`, chained at `lib.rs:1280-1305` | three per-target intercepts matched by declaration *name string* + file suffix; Python asymmetrically hand-reifies the receipt carriers (T-PB-B debt) instead of calling the `.dag` `emit_host_receipt_from_source` |
| Tests | `v2_emit_host_harness_test.rs:866-900` | a surface roster asserting the three names exist — a declaration-shape mirror of the hand-list |

`src/v2/std/` has **no** modeled `Command`/`Process`/`Transport` type at all — the entire
"run an external program" concept lives only in hand Rust. #4621's TypeScript row would be
hand-edit number four of each. Cost-of-change for target #19 must be **one descriptor row,
zero Rust** (CLAUDE.md: the answer should be 1).

## 2. The reframe: the transport is already bidirectional

The existing flow already exercises **both** coercion directions per
`design-bidirectional-coercion.md`:

- **emit** — render `.dag`-modeled computation into target source text (the value-emit
  schema, `design-value-emit-schema.md`);
- **ingest** — parse the target's *runtime output bytes* back into a modeled
  `RuntimeValue` (`runtime_value_parse`).

So "descriptor must serve omni-ingestion" is not a future extension: the stdout codec **is**
an ingestion row today, hand-written three times. The descriptor's job is to carry both
directions as data — source-rendering rows on one side (the value-emit catalog), a
runtime-value codec on the other — so one transport reads them backward (emit) and forward
(ingest), never two parallel mechanisms (P2).

## 3. Design

### 3.1 The descriptor — data in `extdeps/languages/<lang>.dag`

```
// std (substrate types, M4 closed coproducts throughout):
type HostTransportDescriptor {
  workspace: List<WorkspaceFile>        // fixture layout: where emitted source lands,
                                        //   plus fixed manifests (Cargo.toml)
  build: List<ProcessInvocation>        // ordered build steps; [] for run-direct targets
  run: ProcessInvocation                // THE logical run — its stdout is the
                                        //   runtime-value channel. Exactly-one-run is
                                        //   true BY CONSTRUCTION (Q-T2 ratified shape):
                                        //   zero runs, two runs, run-not-last are
                                        //   unrepresentable — no well_formed rule needed
  runtime_value_codec: RuntimeValueCodec
}

type WorkspaceFile {
  path: RelPath                         // e.g. "src/main.rs", "fixture.py", "fixture.ts"
  content: WorkspaceContent
}
type WorkspaceContent
  = EmittedSource                       // the emit pipeline's output goes here
  | FixedText { text: String }          // manifest boilerplate (Cargo.toml)

type ProcessInvocation {
  tool: HostTool                        // modeled identity (Q-T4 ruling), not a path string
  args: List<InvocationArg>
}
type HostTool {
  identity: Symbol                      // ^cargo, ^python3, ^go, ^tsc, ^node — the
                                        //   descriptor names WHO; the host primitive
                                        //   resolves it (PATH lookup, env overrides,
                                        //   version) at the host boundary
}
type InvocationArg
  = LiteralArg { text: String }         // "build", "--quiet", "run"
  | WorkspacePath { path: RelPath }     // "fixture.py", "main.go"
  | ProducedArtifact { path: RelPath }  // "target/debug/fixture", "fixture.js"

type RuntimeValueCodec
  = FixedWidthBytes { width: Nat }      // wave-now: the MVP-2 five-byte contract
  // arms land with their producers (E-10): a structured codec arrives when a
  // target speaks structured values, as a bidirectional row — render expected
  // bytes (emit direction) / parse observed bytes (ingest direction)
```

Worked rows (each one descriptor in its language's `extdeps` module):

- **rust**: workspace `[Cargo.toml fixed, src/main.rs emitted]`; build
  `[cargo build --quiet --manifest-path …]`; run `ProducedArtifact target/debug/fixture`;
  codec `FixedWidthBytes 5`.
- **python**: workspace `[fixture.py emitted]`; build `[]`; run `python3 fixture.py`.
- **go**: workspace `[main.go emitted]`; build `[]`; run `go run main.go`.
- **typescript** (#4621, first new row — the dogfood): workspace `[fixture.ts emitted]`;
  build `[tsc fixture.ts]`; run `node ProducedArtifact fixture.js`.

TypeScript is the proof the schema generalizes: it is the first **two-step** non-cargo
pipeline, and it lands as a row, not a function.

### 3.2 One transport fold (no per-target functions)

```
fn run_emit_host(target: TargetModel, source: TargetSource, fixture_inputs: Inputs)
    -> Outcome<EmitHostRunReceipt>
```

becomes **one body**: `find_witness` selects the target's `HostTransportDescriptor` from
the closed candidate set of language rows (no descriptor ⇒ typed `Rejected`, exactly
today's `emit_host_run_unsupported_target_diagnostic` — C-8 fail-closed, never a default
transport); then a fold over `build` invokes the one primitive (§3.3), short-circuiting on
the first non-`Holds` build exit, followed by `run`; the run's bounded stdout flows into
`emit_host_receipt_from_source` — **one** receipt assembly path, which dissolves the
Python hand-reify asymmetry as a side effect. `runtime_value_parse` likewise becomes one
fn folding the codec datum (`FixedWidthBytes w` ⇒ length check against `w`), deleting the
`_python`/`_go` aliases.

Dispatch is keyed by **language identity** (the descriptor row's own selection witness),
not `authority_source_text` string equality — the pin-matching at `emit_host.dag:239-244`
is itself part of the hand-list and dissolves with it (Q-T1).

### 3.3 The single generic process primitive (the only Rust left)

```rust
pub fn run_host_process(
    invocation: &ResolvedInvocation,   // tool resolved to an executable + argv with
                                       //   paths resolved against work_dir
    work_dir: &Path,
    bounds: &RunBounds,                // timeout + output truncation caps (today's values)
) -> Result<BoundedProcessOutput, HostSetupFailure>
```

Tool resolution is the host boundary (Q-T4 ruling): the descriptor carries the modeled
`HostTool` identity; the primitive resolves it to an executable (PATH lookup, env
overrides like `GUNBC_PYTHON`, version) and fails closed as `HostSetupFailure` when
resolution misses. Identity is data; resolution is host.

plus the existing workspace write (`fs::create_dir_all` + file writes, driven by the
descriptor's `workspace` list) and the existing `host_exit_from_bounded` exit-witness
parse — which are already target-agnostic. **One** eval hook
(`try_dispatch_run_host_process`) replaces the three `try_dispatch_emit_host_*` intercepts
and the three-deep chain at `lib.rs:1280-1305`. Everything that *differs* between targets
is upstairs in data; the primitive knows nothing about cargo, python, tsc, or five bytes.

### 3.4 What fail-closed means here (C-8, P2)

- No descriptor row for the target ⇒ `Rejected`, typed diagnostic (unchanged behavior).
- Build step exits non-zero ⇒ receipt with `ExitWitness::Violates` + `BuildLog`
  (unchanged carrier, `host_run.dag:33-55`).
- Missing host tool ⇒ `HostSetupFailure` (unchanged).
- Codec mismatch ⇒ `RuntimeValueParseFailure` with expected/actual (unchanged, now
  data-parameterized).

No behavior loosens; the change is *where the facts live*.

## 4. Refactor / dissolution plan (ordered, each step lands green)

1. **Model first**: substrate types (§3.1) in `src/v2/std/host_run.dag` (they are the
   missing "command semantics" the explorer measured as absent) + four descriptor rows in
   `extdeps/languages/{rust,python,go,typescript}.dag`. Consumer lands same-PR (E-6/E-10):
   step 2's fold reads them.
2. **One primitive + one hook**: `run_host_process` in `tools/emit_host_runner`;
   `try_dispatch_run_host_process` in `emit_host_eval.rs`. The three old runners remain
   temporarily as callers-of-the-primitive only if needed for a green intermediate; no new
   caller may name them.
3. **Fold the `.dag` side**: `run_emit_host` + `runtime_value_parse` become descriptor
   folds; delete the authority-pin if/else chains and the `runtime_value_parse_python/_go`
   duplicates; receipt assembly unifies on `emit_host_receipt_from_source` (closes T-PB-B).
4. **Delete the hand-list** — **HELD for explicit go (manager ruling 2026-06-10;
   load-bearing)**: `run_emit_host_rust/python/go` (lib.rs and the fail-closed `.dag`
   stubs), `try_dispatch_emit_host_rust/_go/_python` + chain, the `emit_host_bridge.rs`
   per-target `_transport` wrappers (the W3 parity runner takes the generic transport + a
   language argument instead of three function pointers). Gate: steps 1–3 landed green
   **and** the TS round-trip proven through the descriptor row, then go.
5. **Tests**: the surface roster at `v2_emit_host_harness_test.rs:866-900` is a
   declaration-shape mirror of the hand-list — per the white-box-tests-are-2FA policy it is
   **deleted**, not re-pointed. Behavior tests (emit-vs-eval, cross-target parity) re-route
   through the generic transport unchanged in what they assert; the discriminating receipt
   is *TypeScript running through a row that no Rust function names* (mutate the descriptor's
   step args ⇒ red).
6. **Census**: update the SG-0 entries for `emit_host_bridge.rs` / `emit_host_eval.rs`
   (their recorded dissolution condition — "substrate eval owns host dispatch" — is exactly
   what lands here).

Receipts per step: `gunbc compile src/v2` zero diagnostics; emit-vs-eval claims stay green
by execution; suite-delta 0 by conservation; plus the step-5 mutate-red.

## 5. Omni-ingestion symmetry (the obligation, scoped)

The descriptor is shared, not emit-private:

- **Already shared now**: `runtime_value_codec` *is* the ingest direction of the runtime
  boundary (§2) — one row, read forward by `runtime_value_parse`, and (when a consumer
  needs it) backward to render expected bytes for a fixture.
- **Lands with its consumer** (E-10, no speculative fields): ingestion of target *source*
  (reading a TS/Rust project into modeled nodes) reuses the same language module's
  grammar/production rows — the value-emit schema's §4.2 projection rows read backward —
  and the descriptor's `workspace`/`build`/`run` vocabulary when ingestion needs to *execute*
  target toolchains (e.g. type-query a foreign project). No `IngestionDescriptor` twin is
  ever declared; the symmetry is one-descriptor-two-directions, or it is a P2 violation.

## 6. Rulings (escalated 2026-06-10, ruled by still-raven-546 same day)

- **Q-T1 — descriptor selection key: RULED — language identity**, never the
  `authority_source_text` pin (spelling-as-identity, the same lesson as
  `binding_id`/brand). The key is the language identity carried by `TargetModel` (the same
  key the value-emit projection rows use). If `TargetModel` turns out not to carry a
  stable language identity yet, that field is a prerequisite, not a reason to keep string
  pins.
- **Q-T2 — step semantics: RATIFIED with a refinement (2026-06-10)** — the split shape
  `build: List<ProcessInvocation>` + `run: ProcessInvocation` is **primary** (§3.1), not a
  fallback. Rationale: a flat step-list + `StepRole` + `well_formed` check would make
  "exactly one run, last" a *validated* invariant (invalid states representable, then
  rejected); the split makes those states **unrepresentable** — correctness by
  construction over validate-then-reject, the house philosophy. The one-run `well_formed`
  rule is dropped as vacuous. Semantics unchanged: strict build ordering, first
  non-`Holds` build exit short-circuits to a `Violates` receipt, `run`'s stdout is the
  runtime-value channel. No concrete multi-run case exists; a hypothetical "test then
  run" is two transports, not one descriptor.
- **Q-T3 — bounds ownership: RULED — bounds are a separate policy axis**, never descriptor
  fields. The descriptor says WHAT to run; bounds (timeouts/resource limits) are HOW MUCH —
  the compute-fabric policy seam (the `MultiplicityPolicy`/`TargetDeclaredPriority`
  family). Bounds layer on at invocation, host-side in `RunBounds` for now; surfaced to
  the operator as a fabric-policy touchpoint. Proceed bounds-free.
- **Q-T4 — tool identity: RULED — modeled identity in the descriptor** (`HostTool`,
  §3.1), not a bare string; the host primitive resolves it (PATH, env overrides, version)
  at the host boundary (§3.3). Identity is data; resolution is host. A fuller tool
  registry (provisioning) remains out of scope until it has a consumer.

## 7. Non-goals

- No general shell/script modeling — `ProcessInvocation` is argv-only, no pipes, no shell
  interpretation (the closed shape is the point).
- No toolchain installation/provisioning; missing tools remain `HostSetupFailure`.
- No change to the receipt carriers (`host_run.dag`) or the P2 exit/logical-run boundary.
- No source-ingestion build in this wave (§5 scopes the symmetry; consumers gate it).
- Not a generic "task runner" for CI — the CI north-star lane is separate; this transport
  serves emit-vs-eval and its ingestion dual only.
