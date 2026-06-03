# v4 Go RCA Manager Worksheets - 2026-06-03

Scope: #4137 Section 11.8 Go release-minimum lane. These worksheets close the
`go.dag` leaf-model R1/R2 L0 handoff, name the L1 fixture-scale receipt, and
sequence the next L2 fixture-execution step without claiming full self-compile.

## Dispatch Rule

The Go lane advances by rungs:

1. L0: leaf-model facts from `go.dag` are wired to happy plus falsification
   fixture pairs.
2. L1: a bounded compiler slice emits Go and records a `go build` receipt.
3. L2: fixture execution consumes modeled `TestClaim` rows directly.

The temporary Rust boundary tests and shell host runners are receipts only while
T-22 `run_target_verification` is not live. Authority remains in `.dag` rows.

## Worksheet A - R1 Int Surface Spelling

Authority rows:

- Fact: `src/v4/extdeps/languages/go.dag` (`go_surface_spelling_int`).
- Fixture: `src/v4/lens/leaf_model_verification.dag`
  (`go_r1_fixture`, `go_r1_happy_fixture_source`,
  `go_r1_falsification_fixture_source`).
- Claim wiring: `src/v4/test/claim/language_model/go_r1.dag`.
- Temporary runner:
  `src/v3/compiler/tests/boundary/v4_leaf_model_go_r1_r2_r3_external_test.rs`.

Happy fixture: `func r1() int { return 0 }` must compile under `go build`.

Falsification: `func r1() i32 { return 0 }` must reject with the modeled
undefined-type diagnostic. This proves the claim is tied to Go's predeclared
`int` surface spelling, not to a hand-maintained target string table.

L0 close condition: `claim_go_r1_fixture_pair_wired` proves the fixture pair
references `go_surface_spelling_int` through `go_leaf_model_fact_atom`.

## Worksheet B - R2a Int Algebra Operations

Authority rows:

- Facts: `src/v4/extdeps/languages/go.dag` (`go_facts_int`,
  `go_integer_algebra_inhabitance` through the fixture fact node).
- Fixture: `src/v4/lens/leaf_model_verification.dag`
  (`go_r2a_fixture`, `go_r2a_happy_fixture_source`,
  `go_r2a_falsification_fixture_source`).
- Claim wiring: `src/v4/test/claim/language_model/go_r2a.dag`.
- Temporary runner:
  `src/v3/compiler/tests/boundary/v4_leaf_model_go_r1_r2_r3_external_test.rs`.

Happy fixture: `a + b` and `a < b` over `int` must compile under `go build`.

Falsification: `a.log2_exact()` must reject with the modeled undefined-method
diagnostic. This proves the R2a worksheet covers only operations present in the
Go integer algebra fact bundle; it does not smuggle in wider integer APIs.

L0 close condition: `claim_go_r2a_fixture_pair_wired` proves the fixture pair
references `go_r2a_algebra_inhabitance_fact_node()`.

## Worksheet C - R2b Int64 Runtime Overflow

Authority rows:

- Fact: `src/v4/extdeps/languages/go.dag` (`go_facts_int64`).
- Fixture: `src/v4/lens/leaf_model_verification.dag`
  (`go_r2b_fixture`, `go_r2b_happy_fixture_source`,
  `go_r2b_falsification_fixture_source`).
- Claim wiring: `src/v4/test/claim/language_model/go_r2b.dag`.
- Temporary runner:
  `src/v3/compiler/tests/boundary/v4_leaf_model_go_r1_r2_r3_external_test.rs`.

Happy fixture: `math.MaxInt64 + 1` must execute as `math.MinInt64` under
`go run`.

Falsification: the same program with an intentionally wrong expected value must
panic and surface the modeled runtime-panic diagnostic. This is still L0
leaf-model verification because it proves Go's own runtime behavior for one
leaf fact; it is not cross-target parity.

L0 close condition: `claim_go_r2b_fixture_pair_wired` proves the fixture pair
references `go_integer_facts_node(facts: go_facts_int64)`.

## Worksheet D - L1 Fixture-Scale Go Build Receipt

Authority rows:

- Worksheet: `docs/planning/v4-go-l1-compiler-slice-compile-worksheet-2026-06-01.md`.
- Slice id: `go_l1_nat_semiring_rung2`.
- Claim wiring: `src/v4/test/claim/nat_semiring/rung_l1_go_compiler_slice.dag`.
- Temporary host transport: `scripts/v4-phase1-nat-semiring-go-compiler-slice-gate.sh`.

The L1 worksheet starts only after Worksheets A-C are present, because the L1
receipt depends on Go leaf facts being structurally named before a compiler
slice can claim Go compile success.

Acceptance is a structured bounded-slice `go build` receipt. It does not claim
L2 cross-target execution parity, L3 fixed point, or L4 compiler self-compile.

## Worksheet E - L2 Fixture-Execution Sequence

Next target: retire the Go host bridge by routing Worksheets A-C through the
modeled `TestClaim` execution path.

Required shape:

- Preserve the existing claim ids and fact anchors.
- Consume the same happy plus falsification fixture pairs.
- Emit compile/runtime verdict receipts through the shared
  `LeafModelVerificationRunReceipt` and
  `LeafModelRuntimeVerificationRunReceipt` carriers.
- Delete the temporary Go boundary test from the SG-0 census in the same PR that
  generated or modeled execution takes over.

Non-goals:

- No new Go-specific verdict carrier.
- No hand-authored parallel fixture roster outside `LeafModelFixture<C>` or its
  currently gated Go bridge carriers.
- No self-compile claim before L2 fixture execution exists.

## Current Closeout State

- L0 R1/R2a/R2b: on main (#4243 claims + boundary bridge); worksheets A–C closed.
- L1 fixture-scale: **in flight** — PR [#4367](https://github.com/gunb-ai/gunbc/pull/4367)
  (`go_l1_nat_semiring_rung2` emit fixes on `session/smart-moth-425`); worker
  `swift-lark-898` addressing cursor REQUEST_CHANGES + PR body.
- L2 fixture-execution: sequenced by Worksheet E; implementation remains gated
  on T-22 `run_target_verification`.

## Related Artifacts

- `docs/planning/v4-go-leaf-model-verification-worksheet-2026-06-01.md`
- `docs/planning/v4-go-l1-compiler-slice-compile-worksheet-2026-06-01.md`
- `src/v4/extdeps/languages/go.dag`
- `src/v4/lens/leaf_model_verification.dag`
- `src/v4/test/claim/language_model/go_r1.dag`
- `src/v4/test/claim/language_model/go_r2a.dag`
- `src/v4/test/claim/language_model/go_r2b.dag`
- `src/v4/test/claim/nat_semiring/rung_l1_go_compiler_slice.dag`
