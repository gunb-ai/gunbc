# P3 ClassificationCostStanding — same-subject A/B receipts

Paired end-to-end comparison for #8055 P3 (not a lane-read selection count):

| Arm | Selector | Cost terms |
|-----|----------|------------|
| **control** | G1 `runtime_data_dependency_touched_via_carrier_closure` (main) | `prep_exec_ms` on over-selected entry groups |
| **candidate** | G2 manifest + Applied selection (pr-8055) | `classification_production_ms` + `prep_exec_ms` on narrowed entry groups |

## Harness

- `p3_cohort_probe` — branch-neutral probe; build once per arm.
- `tools/p3_cohort_ab_runner.sh` — runs one arm with `GUNBC_P3_ARM` / `GUNBC_P3_SUBJECT`.

## Pin discipline

Fleet/corpus runs must dispatch against an **immutable tag** on the merge SHA (`workflow_dispatch --ref <tag>`), and every `RECEIPT` line must show `head_sha=<merge SHA>`. Discard on mismatch.

## Subjects

- `incident` (default) — two-entry roster from `floor_skip_discovery_witness`; control selects 2/2, candidate 1/2 with `skip_before_resolve=1`.
- `discovery` — full discovery scan with Applied selection (corpus-scale; use pinned tag on fleet).

Populated `.log` files from executed paired runs land in this directory.
