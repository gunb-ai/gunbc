# P3 ClassificationCostStanding — same-subject A/B receipts

Paired end-to-end comparison for #8055 P3 (not a lane-read selection count):

| Arm | Selector | Cost terms |
|-----|----------|------------|
| **control** | G1 `runtime_data_dependency_touched_via_carrier_closure` (main) | `prep_exec_ms` on over-selected entry groups |
| **candidate** | G2 manifest + Applied selection (pr-8055) | `classification_production_ms` + `prep_exec_ms` on narrowed entry groups |

## Harness

- `p3_cohort_probe` — branch-neutral probe; build once per arm, invoke directly (no shell wrapper).

### Paired run (one arm per build)

```bash
# control arm — build on main
ctrl-build -- cargo build -p v1-compiler --release --bin p3_cohort_probe
GUNBC_P3_ARM=control GUNBC_P3_SUBJECT=incident ./target/release/p3_cohort_probe 2>&1 | tee control.log

# candidate arm — build on pr-8055 / merge SHA
GUNBC_P3_ARM=candidate GUNBC_P3_SUBJECT=incident ./target/release/p3_cohort_probe 2>&1 | tee candidate.log
```

Compare `RECEIPT` lines: `classification_production_ms + prep_exec_ms` (candidate) vs `prep_exec_ms` at higher `selected_entry_groups` (control). Assert `head_sha` on each receipt matches the pinned tag.

Env:

- `GUNBC_P3_ARM=control|candidate` — arm label in the receipt
- `GUNBC_P3_SUBJECT=incident|discovery` — subject (default `incident`)

## Pin discipline

Fleet/corpus runs must dispatch against an **immutable tag** on the merge SHA (`workflow_dispatch --ref <tag>`), and every `RECEIPT` line must show `head_sha=<merge SHA>`. Discard on mismatch.

## Subjects

- `incident` (default) — two-entry roster from `cli_run::selection_control_incident_subject_roster`; control selects 2/2, candidate 1/2 with `skip_before_resolve=1`.
- `discovery` — full discovery scan with Applied selection (corpus-scale; use pinned tag on fleet).

Populated `.log` files from executed paired runs land in this directory.
