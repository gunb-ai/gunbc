# Handoff — Phase 0: test-duration ratchet + layer taxonomy

Phase 0 worker (`quick-crab-901`) owns measurement infrastructure and
narrow directory reorg only. Test semantics are unchanged. Deeper
migrations (m0_acceptance, m1_substrate, lens tests, thesis tests)
are explicitly out of scope and move under their own workers.

**Scope clarification — this PR is the report-only baseline.** The
acceptance-criterion phrasing "CI fails on any test >2s" is split
across two PRs by design:
- **This PR (quick-crab-901)** lands the ratchet script, the exemption
  file, the CI step, and the directory reorg. The CI step runs the
  ratchet and publishes violations in the log, but is
  `continue-on-error: true`, so main still merges with arbitrarily
  slow per-test timings. This is the baseline-publication PR.
- **Follow-up PR** reads the first full CI run's report, populates
  `scripts/slow-test-exemptions.txt` with one line per currently-slow
  test + a reason, and removes `continue-on-error: true` from the CI
  step. That is the PR that makes the ratchet binding.

Splitting this way keeps the Phase 0 diff narrow (measurement + org
only) and keeps the paydown decisions in a second diff with an
attached baseline to justify each entry, instead of landing an
unpopulated exemption file and a green-but-informative CI step as a
single contradiction.

## What changed

### 1. Per-test 2s ratchet

- New: `scripts/check-test-timeout.sh` — parses a **pre-captured**
  libtest log (path passed as `$1`), reads the `<N.NNNs>` trailer on
  each `ok|FAILED` line, and exits non-zero when any single `#[test]`
  exceeds the budget (default **2000 ms**, overridable via
  `TEST_TIMEOUT_MS`). The existing v3 full-suite CI step now runs with
  `RUSTC_BOOTSTRAP=1 cargo test -- -Z unstable-options --report-time`
  and tees the output to `/tmp/v3-test-timings.log`; the ratchet step
  parses that file. This keeps the CI job to a single `cargo test`
  invocation — no second full-suite run that would starve later
  quality gates inside the 25-minute v3 job budget. A local-fallback
  mode (invoke the script with no args) does run `cargo test` itself
  for users without a captured log.
- New: `scripts/slow-test-exemptions.txt` — paydown list. Tests named
  here are tolerated with a `::warning::` line; every other slow test
  fails the ratchet. Empty on landing.
- New CI step in `.github/workflows/ci.yml` (`v3` job, above the
  clippy gate): `v3 tests (per-test 2s ratchet, report-only baseline)`.
  Marked `continue-on-error: true` **and** `if: always()` — the first
  keeps the job green while the exemption file is empty (baseline
  publication); the second keeps the baseline visible on red runs too
  (full-suite failure, 1200s wall-clock overflow), which is precisely
  when the timing data is most diagnostic. `tee` captures the timing
  log before the upstream budget check runs, so partial data survives.
  A small guard in the step handles the degenerate case where the
  upstream step errored before `cargo test` even ran, emitting a
  `::notice::` instead of trying to parse a missing log. The follow-up
  PR flips `continue-on-error` off to make the ratchet binding.

**Why `RUSTC_BOOTSTRAP=1`.** libtest's `--report-time` is still
unstable on the 1.93 toolchain (tracking issue rust-lang/rust#64888).
The bootstrap flag is the narrow unlock — it enables the *libtest
flag*, not unstable language features. Migrate off it when the flag
stabilizes or when the project adopts `cargo-nextest` (which has a
native `slow-timeout`).

### 2. Layer taxonomy

TESTING.md § "Test layers" defines three tiers: `unit` (<5ms),
`integration` (<100ms), `boundary` (<2s). The directory under
`tests/` now mirrors that partition.

- `tests/unit/README.md` (new) — documents the convention; empty of
  `.rs` files on landing. Unit tests today still live in-crate as
  `#[cfg(test)] mod tests` inside `src/v3/compiler/src/`, per
  TESTING.md § "Mocks and dependency injection".
- `tests/integration/` — unchanged location; now one of three
  taxonomy-named peer dirs.
- `tests/boundary/README.md` (new) — holds target-language
  roundtrip tests.

Layer-tagging convention: each test file carries a module-level
doc comment `//! **Layer:** <unit|integration|boundary> ...`. Grep
reports the live partition:

```
grep -rn '\*\*Layer:\*\*' src/v3/compiler/tests/
```

The **directory is the load-bearing classification**; the header is
a human-readable echo that keeps the layer visible when reading the
file on its own.

### 3. Files moved (boundary)

All four target-emission suites moved from `tests/integration/` into
`tests/boundary/`. Paths only — content and test names unchanged.

| Old | New |
|---|---|
| `tests/integration/m1_3_emit_go_test.rs` | `tests/boundary/m1_3_emit_go_test.rs` |
| `tests/integration/m1_3_emit_rust_test.rs` | `tests/boundary/m1_3_emit_rust_test.rs` |
| `tests/integration/m1_4_emit_python_test.rs` | `tests/boundary/m1_4_emit_python_test.rs` |
| `tests/integration/m2_emit_multi_field_struct_variant_test.rs` | `tests/boundary/m2_emit_multi_field_struct_variant_test.rs` |

Mirrored in:
- `tests/integration.rs` — `#[path]` entries updated.
- `tests/integration/sg0_census_test.rs::EXPECTED_HAND_AUTHORED` —
  path rename (net-zero; no new hand-authored files). Added an inline
  comment explaining the Phase 0 reorg.

**Single-binary preservation.** The consolidated `tests/integration.rs`
binary still includes every file (from `integration/`, `boundary/`,
and — eventually — `unit/`) via `#[path]`. The one-bootstrap compile
amortization documented in `tests/integration.rs` holds; no new
top-level `tests/*.rs` binary was added. The directory names carry
the taxonomy; the binary carries the compile-cost amortization.

### 4. Layer tags landed

Representative tags added (the mechanism is extensible; not every
file was touched):

- `tests/boundary/m1_3_emit_go_test.rs` — `Layer: boundary`
- `tests/boundary/m1_3_emit_rust_test.rs` — `Layer: boundary`
- `tests/boundary/m1_4_emit_python_test.rs` — `Layer: boundary`
- `tests/boundary/m2_emit_multi_field_struct_variant_test.rs` — `Layer: boundary`
- `tests/integration/l1_5_fixed_point_test.rs` — `Layer: integration`
  (one representative; the dir is the authority)

## Tests now failing or flaky

**None from the reorg itself.** Confirmed:
- `cargo check -p v3-compiler --tests` clean.
- `cargo fmt --all --check` clean.
- `cargo test -p v3-compiler --test integration sg0_census_test::` — 6/6 pass.
- `cargo test -p v3-compiler --test integration -- m1_3_emit_go_test
  m1_3_emit_rust_test m1_4_emit_python_test
  m2_emit_multi_field_struct_variant_test` — 48 passed, 9 ignored,
  0 failed (unchanged vs pre-move).

**Tests the 2s ratchet will likely flag on first CI run** (not
measured end-to-end in this worker — the full-suite pass costs
~12–20 min and yields information that belongs in the exemption
file, not in a narrow reorg diff):

- `m1_5_testgen_test::*` — TESTING.md § "Migration audit" explicitly
  flags this suite as "over-expensive; reshape or `#[ignore]`-by-default".
- `lane2_stage_2e_parallelism_test::*` — `.github/workflows/ci.yml`
  line 111 calls out this suite for redoing full `compile_to_dag`
  per `#[test]`.
- `determinism_test::*` — 5× re-emit matrix, release build. Wall-clock
  per-test is expected to exceed 2s.
- Some `tests/boundary/*` cases — rustc cold-invocation cost. Per-file
  `OnceLock<PathBuf>` amortization already in place, but individual
  `#[test]`s that compile their own harness can still exceed 2s.

## Next steps (unclaimed) — the binding half

1. **Populate `scripts/slow-test-exemptions.txt`.** Read the first
   merged-main CI run's report-only baseline (or run
   `scripts/check-test-timeout.sh` locally — no args invokes the
   fallback that runs `cargo test` itself), and add each violating
   test to the exemption file with a one-line reason that cites a
   ROADMAP item or project memory.
   **Entry format:** the token libtest emits as the second
   whitespace-delimited field, e.g. `m1_5_testgen_test::heavy_case`
   — no binary prefix, no `<…s>` timing. The exemption file header
   documents the exact shape and an example line.
2. **Flip the CI step to blocking.** Remove
   `continue-on-error: true` from the
   `v3 tests (per-test 2s ratchet, report-only baseline)` step in
   `.github/workflows/ci.yml` and rename to
   `v3 tests (per-test 2s ratchet)` once the exemption file reflects
   the real baseline. This is the PR that satisfies the Phase 0
   acceptance criterion "CI fails on any test >2s."
3. **Paydown (owned by sibling workers).** Each exempt test is a
   migration target — either speed it up (share bootstrap via
   `OnceLock`, shrink fixtures, collapse fine-grained cases),
   reshape it (spot-check instead of exhaustive sweep), or
   `#[ignore]`-by-default.
4. **Unit track.** When the tracked `Dag` builder surface lands
   (TESTING.md § "Mocks over compile"), migrate lens / accessor
   tests out of `#[cfg(test)]` mod blocks into `tests/unit/*.rs`
   against the public mocking surface.
