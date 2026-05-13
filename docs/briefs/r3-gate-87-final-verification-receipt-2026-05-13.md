# R3 Gate #87 Final Verification Receipt

Gate: `lens_cementing_test_discipline_complete`

Date: 2026-05-13

## Scope

This receipt closes the 87-E verification slice for gate #87. It records the
merge-visible evidence needed to validate the lens-cementing discipline over
the `src/v3/compiler/regen.dag` `LensRegistryEntry` enumeration surface.

The gate invariant is the one named in `docs/r3-structure.md`: every
`regen.dag` lens row has a gate-87 cementing receipt, each receipt is exercised
through the PB-B-1 runner table, and temporary `Compiles` receipts are explicit
Band-C placeholders with paired Rust pins or named dissolution triggers where a
behavioral `.dag` witness is not authorable yet.

## Child Work Items And PRs

| Slice | Receipt |
| --- | --- |
| 87-A registry inventory / runner ratchet | PR #2843, `G87-A registry/runner/dispatch inventory ratchet for lens cementing completeness` |
| 87-B real-v2 lens receipts | PR #2846, `G87-B real-v2 lens cementing receipts and temporary Rust blocker audit` |
| 87-C helper / v3-native placeholder discipline | PR #2845, `G87-C: gate-87 v3-native lens cementing - discriminating receipts` |
| 87-C parallelism drift repair | PR #2860 / PR #2872, `parallelism` cementing receipt + ratchet repair |
| 87-D dispatch projection handoff | PR #2842, `docs(r3): G87-D Band-C cementing census handoff to #84 bulk-port` |
| Dispatch brief | PR #2851 / PR #2872 decomposition brief for the A-E work split |

## Verification Commands

Focused commands:

```text
cargo test -p v3-compiler --test integration r3_gate_87 -- --nocapture
cargo test -p v3-compiler --test integration cementing_dispatch -- --nocapture
cargo test -p v3-compiler --test integration lens_capability_register_rows_match_md_v2_cementing_projection -- --nocapture
```

Coverage intent:

- `r3_gate_87_regen_lens_registry_names_match_fixture_inventory` verifies the
  live `regen.dag` registry names match the single PB-B-1 runner table.
- `t_pb_b_1_dag_runner_test::r3_gate_87_cementing_regen_lens_suites_pass_through_runner`
  executes every `tests/dag/t_r3_gate_87_cementing_regen_*.dag` suite from the
  shared table and requires `ClaimResult::Pass`.
- `cementing_dispatch` projection tests keep the dispatch projection aligned
  with the same runner inventory and register rows instead of relying on a
  second hand-maintained cementing list.
- `lens_capability_register_rows_match_md_v2_cementing_projection` keeps the
  capability-register Band-C v2-cementing projection aligned with the markdown
  authority for rows that name real v2 counterparts.

Result:

- `r3_gate_87`: passed via ctrl-build on PR #2872 commit
  `6123b8d34e8cb2fdde52c2876cfd75597b9e3f40`: 11 passed, 0 failed, 0 ignored;
  1057 filtered out. The run included the `parallelism` receipt, the registry
  inventory ratchet, and the PB-B-1 runner wiring test.
- `cementing_dispatch`: passed via ctrl-build on the same PR #2872 commit:
  2 passed, 0 failed.
- `lens_capability_register_rows_match_md_v2_cementing_projection`: passed via
  ctrl-build on the same PR #2872 commit: 1 passed, 0 failed.

## Closure Disposition

Gate #87 is closeable from merge-visible artifacts when the focused commands
above are green on the implementation branch and the PR body preserves the child
receipt list plus validation result. Rows outside `src/v3/compiler/regen.dag`
remain under the broader Band-C / register-ratchet lane and are not part of
this gate's enumeration surface.

