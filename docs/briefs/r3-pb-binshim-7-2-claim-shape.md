# R3 PB — §7.2 BinShim equivalence TestClaim shape (docs-only shape artifact + STOP+PING)

**Status:** SHAPE ARTIFACT (docs-only). Authored 2026-05-01 by PB Manager continuation per dispatch on inbox #1149 (quick-newt archived; PB picks up the §7.2 shape lane).

**Goal of this PR:** lock the `regen_lens_bin_shim_emits_behaviorally_equivalent_to_hand_rust` `TestClaim` shape against live substrate and current fixture conventions, document the locked predicate choice, and identify exactly what blocks landing this as a real `.dag` fixture today.

**This PR does NOT** author a `.dag` `TestClaim` declaration. See §"STOP+PING — why no `.dag` claim lands yet" below.

## Live substrate verification (origin/main HEAD)

Per `gh fetch origin main` 2026-05-01:

| Item | Status | Authority / line |
|---|---|---|
| `TestClaim` carrier | LIVE | `src/v3/std/verification.dag:type TestClaim { name: String, source: String, file_name: String, predicate: TestPredicate, requires: List<ResourceReference> }` |
| `TestSuite` carrier | LIVE | `src/v3/std/verification.dag:type TestSuite { name: String, claims: List<TestClaim> }` |
| `ExecuteCommand` `TestPredicate` variant | LIVE | `src/v3/std/verification.dag:148-152` (`{ command: String, args: List<String>, expect_exit_code: Int }`) |
| `ForAllTargets` `TestPredicate` variant | LIVE | `src/v3/std/verification.dag:161-165` (same shape, scoped per target) |
| `BinShim` carrier | LIVE | `src/v3/std/bin_shim.dag:19` (post-#1361) |
| `regen_lens_shim` instance under `dsl/std/runtime/bin_shims/` | NOT YET LIVE | per `dsl/std/runtime/bin_shims/README.md` STOP+PING (entry-function gap) |
| BinShim emitter (`.dag` program) | NOT YET LIVE | per `docs/briefs/r3-pb-binshim-emitter-readiness.md` |
| Emitted-Rust `regen_lens.rs` artifact | NOT YET PRODUCIBLE | depends on emitter + instance |
| Comparison script (the §7.2 fixture's host receipt) | NOT YET LIVE | this PR's STOP+PING |

## Locked predicate choice

Per design-doc §7.2: "The precise predicate shape is deferred to the worker authoring the fixture; the design lock fixes the *intent*, not the comparison mechanism."

**This PR locks the choice as Mechanism (1) "canonicalize-then-diff" via `ExecuteCommand`** — the most live-substrate-friendly of the three plausible mechanisms in §7.2:

| Mechanism | Why locked / deferred |
|---|---|
| (1) **Canonicalize-then-diff** *(LOCKED here)* | Maps cleanly to the existing `ExecuteCommand("bash", ["-c", …], 0)` host-receipt pattern (precedent at `src/v3/compiler/tests/fixtures/r1_gates.dag:94-108` for `p0_no_fabrication_sentinel` + `p0_rest_ops_aligned`). Tolerates `// AUTO-GENERATED` header drift + whitespace formatting differences inevitable between hand-Rust and `.dag`-emitted Rust per design-doc §7.2 framing. |
| (2) AST-equivalence via `syn` | Strictest; would also map to `ExecuteCommand` calling a `syn`-based comparator binary, but adds a Rust-side dependency surface this PR doesn't need to commit to now. Available as a follow-up tighten under a Substrate Manager §P1 disposition if mechanism (1)'s tolerance is too loose. |
| (3) Behavioral run-and-compare | Captures the actual contract (exit codes / stdout / filesystem effects) but requires holding two binaries simultaneously during cutover. Mechanism (1) is sufficient for §7.2's structural intent ("emitted Rust is *behaviorally* equivalent to hand-Rust shim — not byte-identical"); behavioral runs add operational surface that's out of §7.2's stated scope. |

Mechanism (1)'s `ExecuteCommand` shell pipeline (locked here; the worker's job at fixture-landing time is to write the host script):

```
rustfmt < emitted-shim.rs > /tmp/a
rustfmt < hand-shim.rs    > /tmp/b
sed '/^\/\/ AUTO-GENERATED/d' /tmp/a > /tmp/a.canonical
diff /tmp/a.canonical /tmp/b
```

Exit code 0 iff the two forms are equivalent modulo the `AUTO-GENERATED` header + rustfmt-canonicalized whitespace.

## Locked TestClaim shape

Following the live `r1_gates.dag` `ExecuteCommand` host-receipt fixture convention (`source: "module <name>_host\n"`, `file_name: "<name>.v3"`, `predicate: ExecuteCommand("bash", ["-c", "exec bash \"$(git rev-parse --show-toplevel)/scripts/<script>.sh\""], 0)`, `requires: []`), the `.dag` declaration shape this lane will land at fixture-authoring time:

```dag
// In src/v3/compiler/tests/fixtures/r3_pb_binshim_equivalence.dag (path TBD; PB worker picks):

module v3.compiler.tests.fixtures.r3_pb_binshim_equivalence

import std.verification {
  TestClaim,
  TestSuite,
  TestPredicate,
  ExecuteCommand,
}

data regen_lens_bin_shim_emits_behaviorally_equivalent_to_hand_rust: TestClaim = {
  name: "regen_lens_bin_shim_emits_behaviorally_equivalent_to_hand_rust",
  source: "module regen_lens_bin_shim_equivalence_host\n",
  file_name: "regen_lens_bin_shim_emits_behaviorally_equivalent_to_hand_rust.v3",
  predicate: ExecuteCommand(
    "bash",
    ["-c", "exec bash \"$(git rev-parse --show-toplevel)/scripts/r3_pb_regen_lens_bin_shim_equivalence.sh\""],
    0
  ),
  requires: []
}

data r3_pb_binshim_equivalence_suite: TestSuite = {
  name: "r3_pb_binshim_equivalence_suite",
  claims: [regen_lens_bin_shim_emits_behaviorally_equivalent_to_hand_rust]
}
```

This shape compiles/parses today (every type used is live; no `DeclarationRef` forward references; predicate variant fully live with `String`/`List<String>`/`Int` literal field values). The only forward-string is the path to a host script that does not yet exist.

## STOP+PING — why no `.dag` claim lands yet

Even though the TestClaim above parses against live substrate, **landing it as a real `.dag` fixture today would be dishonest**. The runner evaluates fixtures by invoking the `predicate`. For `ExecuteCommand`, that means actually running the named script — and the script `scripts/r3_pb_regen_lens_bin_shim_equivalence.sh` does not exist on origin/main. Three things are missing, in dispatch-able order:

1. **Comparison script** — `scripts/r3_pb_regen_lens_bin_shim_equivalence.sh` (path TBD; PB worker picks at fixture-authoring time). Implements the locked Mechanism (1) shell pipeline above. Cannot be authored honestly today because it needs both an emitted `regen_lens.rs` AND a stashed copy of the hand-Rust form to diff against, neither of which exists yet on main.
2. **Emitted `regen_lens.rs`** — produced by the BinShim emitter once Items 4+5 (PB-Runtime + bin-shim emit pattern) land per `docs/briefs/r3-pb-binshim-emitter-readiness.md`. The emitter is gated on R2-Evaluator stabilization + `regen_lens_shim` instance authoring (which in turn is gated on the `<bin_name>_main` entry-function landing per `dsl/std/runtime/bin_shims/README.md` STOP+PING).
3. **Hand-Rust snapshot for diff** — at retirement time, the retirement PR copies the current `src/v3/compiler/src/bin/regen_lens.rs` to a fixture path (e.g. `src/v3/compiler/tests/fixtures/regen_lens_hand.rs.expected`) immediately before invoking the emitter to overwrite the original path. This snapshot is not on main; it lands as part of the retirement PR.

Authoring an evaluable `.dag` claim that calls a non-existent script — even if it parses — is the same fail-red-permanently pattern the convergence-matrix audit (#1235) and BinShim framework README (#1347/#1368) explicitly reject. **The shape is locked here; the live fixture lands when the three gating items above are met.**

## Order of unblock

Per dispatch's request for "exact blocker and next unblock order":

1. R2-Evaluator stable (T-Substrate-Lens-Primitive substrate + PR-A through PR-E lane closure per `docs/briefs/r2-evaluator-manager.md`).
2. Item 4 (PB-Runtime interpreter-as-data) landed per `docs/design-pb-runtime-interpreter.md` §3.
3. `<bin_name>_main` entry-function declaration for `regen_lens` lands (per `dsl/std/runtime/bin_shims/README.md` STOP+PING).
4. `data regen_lens_shim: BinShim = { ... entry: regen_lens_main, ... }` instance under `dsl/std/runtime/bin_shims/regen_lens.dag` lands.
5. BinShim emitter (`.dag` program) lands per `docs/briefs/r3-pb-binshim-emitter-readiness.md`.
6. Retirement PR runs the emitter to produce `src/v3/compiler/src/bin/regen_lens.rs` (auto-generated form), stashes the hand-Rust form to a fixture-snapshot path, authors the `scripts/r3_pb_regen_lens_bin_shim_equivalence.sh` host script, lands the §7.2 fixture as a real `.dag` claim, and registers the suite in the test runner. SG-0 census + `REGEN_OUTPUTS` deltas land atomically (per `docs/briefs/r3-pb-regen-lens-consumer-audit.md` §"Per-handoff rows").

The §7.2 claim shape locked in this PR is the artifact step (6) consumes. It will not change at retirement time except for filling in the script path PB-worker picks.

## Non-goals (verbatim from dispatch)

- No `ReleaseDeferredClaim` / `SubstrateResearchDeferredClaim` generic staging.
- No invented `TestPredicate` variant (none needed; `ExecuteCommand` is sufficient for Mechanism (1)).
- No BinShim emitter authoring.
- No `regen_lens_shim` instance authoring.
- No `regen_lens.rs` retirement.
- No comparison-script authoring.
- Manager-brief / README updates are link-only; no implementation-progress claim.

## Cross-refs

- Parent design lock: [`docs/design-pb-runtime-interpreter.md`](../design-pb-runtime-interpreter.md) §7.2 (BinShim equivalence fixture).
- Parent planning brief: [`docs/briefs/r3-pb-binshim-retirement-worker.md`](r3-pb-binshim-retirement-worker.md) §"Acceptance — what a future implementation PR must prove" §7.2.
- Emitter readiness: [`docs/briefs/r3-pb-binshim-emitter-readiness.md`](r3-pb-binshim-emitter-readiness.md).
- Instance-declaration framework: [`dsl/std/runtime/bin_shims/README.md`](../../dsl/std/runtime/bin_shims/README.md).
- Sub-gate skeleton (consumer of this shape at retirement time): [`docs/briefs/r3-pb-t-lensproducer-sub3-regen-lens-retirement.md`](r3-pb-t-lensproducer-sub3-regen-lens-retirement.md).
- Consumer audit (per-handoff atomic deltas): [`docs/briefs/r3-pb-regen-lens-consumer-audit.md`](r3-pb-regen-lens-consumer-audit.md).
- PB Manager brief: [`docs/briefs/r2-pure-bootstrap-manager.md`](r2-pure-bootstrap-manager.md).
- `ExecuteCommand` host-receipt fixture precedent (the live shape this lane mirrors): `src/v3/compiler/tests/fixtures/r1_gates.dag:94-108` (`p0_no_fabrication_sentinel` + `p0_rest_ops_aligned`).
- Live `TestPredicate` variants and `TestClaim` / `TestSuite` carriers: `src/v3/std/verification.dag` (origin/main HEAD at audit time: `type TestPredicate` at `:109`, `ExecuteCommand` at `:148-152`, `type TestClaim` at `:299`, `type TestSuite` at `:307` — line numbers drift; grep for the type/variant name to re-anchor).
- Live `BinShim` carrier: `src/v3/std/bin_shim.dag:19`.
- Live `ExecuteCommand` variant: `src/v3/std/verification.dag:148-152`.
