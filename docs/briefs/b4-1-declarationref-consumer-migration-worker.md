# B4.1 — `DeclarationRef` consumer migration for runner identity bridges `(M, Tier 1)`

> **Worker brief.** Reports through Director (`zesty-bear-812`).
> First B4 Phase 1 sub-brief from
> [`docs/briefs/b4-identity-carrier-substrate-pass.md`](b4-identity-carrier-substrate-pass.md).
> This is a consumer-migration brief, not a carrier-landing brief:
> `DeclarationRef` already exists at `src/v3/spec/v3_l1.dag`.

## Read first

- **[`src/v3/spec/v3_l1.dag:62-69`](../../src/v3/spec/v3_l1.dag)** —
  live `DeclarationRef` authority. A record field typed as
  `DeclarationRef` lowers identifier/dotted-path field values to
  `FieldValue::Reference(DeclarationId)`.
- **[`src/v3/compiler/src/lower.rs:2803`](../../src/v3/compiler/src/lower.rs)** —
  lowerer support for the `DeclarationRef` sentinel.
- **[`src/v3/std/verification.dag`](../../src/v3/std/verification.dag)** —
  existing verification consumers: `LensOutputEquals`,
  `DifferentialEquals`, `MockBackedInvariant`, etc. already carry
  `DeclarationRef` fields.
- **[`src/v3/compiler/src/test_runner.rs:20-48`](../../src/v3/compiler/src/test_runner.rs)** —
  canonical `named_function_count.dag` `include_str!` bridge and
  fixture-filename routing comments, including
  `cost_bind_for_claim_file`.
- **[`src/v3/compiler/src/test_runner.rs:1589-1765`](../../src/v3/compiler/src/test_runner.rs)** —
  `LensOutputEquals` program-input sentinel path.
- **[`src/v3/compiler/src/test_runner.rs:1829-1866`](../../src/v3/compiler/src/test_runner.rs)** —
  `DifferentialEquals` program-input sentinel path.
- **[`src/v3/compiler/tests/fixtures/r1_gates.dag`](../../src/v3/compiler/tests/fixtures/r1_gates.dag)**
  and **[`src/v3/compiler/tests/t_demo/t_demo_fixtures.dag`](../../src/v3/compiler/tests/t_demo/t_demo_fixtures.dag)** —
  fixture declarations that currently define/use
  `r1_lens_output_input_from_program`.
- **`feedback_groundedness_gates_lenses`**,
  **`feedback_no_metadata_markers`**, and
  **`feedback_construction_over_ratchets`**.

## Existing-authority audit

`DeclarationRef` is already landed and consumed. Grep audit:

- `src/v3/spec/v3_l1.dag` declares `type DeclarationRef {}`.
- `src/v3/std/verification.dag` imports it and uses it for
  `LensOutputEquals.lens_ref`, `input_ref`, `expected_ref`,
  `DifferentialEquals.subject_ref`, `oracle_ref`, `input_ref`, and
  other verification payloads.
- `src/v3/std/emit_model.dag`, `src/v3/std/resources.dag`,
  `src/v3/std/substrate.dag`, and `src/v3/spec/python.dag` already
  use `DeclarationRef`.
- `lower.rs` resolves fields whose declared type walks to
  `DeclarationRef` into `FieldValue::Reference`.

Conclusion: do **not** add a new universal declaration-reference
carrier. The remaining B4.1 work is a role/consumer migration on top
of the existing `DeclarationRef`: replace runner-local sentinel and
filename bridges with structural references that identify the program
input role and canonical lens declaration without string dispatch.

## Frame

The current runner has two different facts collapsed into strings:

1. **Program input identity.** `r1_lens_output_input_from_program`
   means "the input value is the claim program DAG reflected from
   `TestClaim.source`", not an ordinary declaration value. The current
   code compares declaration names to this sentinel.
2. **Canonical lens identity.** `named_function_count` uses
   `include_str!("../lenses/named_function_count.dag")` and
   filename checks to recover the canonical lens program.
3. **Output-bind identity.** `cost_bind_for_claim_file` maps
   `TestClaim.file_name` to ordinary bind names such as
   `merge_sort_out` and `lane_e_diff_out`. This is a fixture-filename
   bridge for "which bind is the asserted output?", not a property of
   the source program.

Plain `DeclarationRef` already carries "this field names a
declaration" but it does not, by itself, distinguish ordinary
declaration values from the special program-input role, nor does it
load the referenced lens program, nor does it say which bind is the
claim output. B4.1 should land the smallest structural role layer
needed by the runner, then migrate the consumers.

## Slice

1. **Model the program-input role structurally.**
   - Prefer a typed declaration in `src/v3/std/verification.dag`
     that represents the program-input carrier/role used by
     lens-output and differential checks.
   - The runner should detect this role by `DeclarationId` or typed
     `ValueBody`/meta-tag shape, not by the name
     `r1_lens_output_input_from_program`.
   - Keep the public fixture surface compatible only if the carrier is
     explicit and structural; do not introduce a replacement sentinel
     string.

2. **Model canonical lens identity structurally.**
   - Audit whether an existing `DeclarationRef` in the claim payload
     can point directly at the canonical lens declaration loaded from
     the same compiled fixture DAG.
   - If not, land the minimal runner/lens registry carrier in
     `std.verification` or a more appropriate existing std/spec file.
   - Delete or confine the `include_str!` canonical-lens bridge once
     the runner can obtain the referenced lens program structurally.

3. **Model output-bind identity structurally (§0.2).**
   - Replace `cost_bind_for_claim_file(TestClaim.file_name)` with a
     structural reference from the claim payload to the output bind
     declaration.
   - Prefer a `DeclarationRef` field (or a typed role wrapper over a
     `DeclarationRef`) that names the exact bind declaration whose
     value port should be compared.
   - If the current verification payload cannot express "this bind is
     the claim output" without a broader role model, STOP and split a
     named B4.1a follow-up brief before implementation.
   - Do not replace the file-name map with another string-to-string
     registry.

4. **Migrate `LensOutputEquals`.**
   - Replace `PROGRAM_INPUT_SENTINEL` checks at
     `test_runner.rs:1594`, `:1617`, `:1642`, and `:1709` with the
     structural program-input role.
   - Preserve fail-closed behavior for ordinary `input_ref`
     declarations with missing value bodies.

5. **Migrate `DifferentialEquals`.**
   - Replace the `PROGRAM_INPUT_SENTINEL` check at
     `test_runner.rs:1855` with the same structural program-input
     role.

6. **Update fixtures and tests.**
   - Rewrite `r1_gates.dag`, `r1_gates.template.dag`, and
     `t_demo_fixtures.dag` to use the structural carrier.
   - Add regression coverage that an ordinary declaration named like
     the old sentinel is not treated as program input unless it carries
     the structural role.
   - Add regression coverage that an ordinary fixture filename no
     longer selects a claim output bind unless the claim carries the
     structural output-bind reference.

## Acceptance

- [ ] No `PROGRAM_INPUT_SENTINEL` constant remains in `test_runner.rs`.
- [ ] `test_runner.rs` no longer dispatches on the literal
      `"r1_lens_output_input_from_program"`.
- [ ] Canonical lens lookup no longer depends on fixture filename or
      `include_str!` once the referenced lens identity is structurally
      available. If this proves to require a separate registry carrier,
      STOP and split that registry carrier as B4.1a.
- [ ] `cost_bind_for_claim_file` and its `TestClaim.file_name` routing
      are removed or split into a named B4.1a follow-up before any
      runner migration PR proceeds.
- [ ] Claim output-bind selection is carried by a structural
      `DeclarationRef`/role reference, not by fixture filename or bind
      name strings.
- [ ] Fixtures use `DeclarationRef` plus the new structural role layer;
      no replacement sentinel string is introduced.
- [ ] Regression test proves name-only spoofing of the old sentinel
      does not select the program-input path.
- [ ] Regression test proves filename-only spoofing does not select an
      output bind without the structural output-bind reference.
- [ ] `cargo test --workspace --exclude v2-compiler-tests` passes.
- [ ] `cargo clippy --all-targets -- -D warnings` clean.
- [ ] `cargo fmt --all --check` clean.
- [ ] DB-8 fixed-point converges bit-identically, or any intentional
      std/verification fixture drift is recorded and approved by
      Director.

## STOP-AND-ESCALATE

- **Plain `DeclarationRef` is insufficient and the needed role layer
  wants a broader substrate change** (for example, a generic
  `RoleRef<T>` or declaration metadata model). Stop and re-scope with
  Director/Zero-Floor before editing consumers.
- **Canonical lens identity requires loading a second DAG by path.**
  Do not replace one `include_str!` bridge with another string registry;
  split a structural lens-registry carrier brief.
- **A fixture cannot express the program-input role without a string
  sentinel.** Stop; the carrier is under-specified.
- **DB-8 drifts unexpectedly.** Stop immediately.

## Non-goals

- Not touching B4.2 fold-shape carrier or the `lens_apply.rs`
  `std/algebra.dag` skip.
- Not touching B4.3 emit-helper carriers.
- Not touching B4.4 extdeps fixture-set carrier.
- Not deleting file-preference rank in this PR; §0.7 likely needs a
  dedicated declaration-source carrier after the B4.1 runner slice.

## Reporting

Single PR for B4.1 once implemented. Title:
`fix(v3): B4.1 migrate runner identity bridges to DeclarationRef roles`.
PR body must cite this brief, include the existing-authority audit
above, and record whether canonical lens identity stayed in B4.1 or
was split to B4.1a.
