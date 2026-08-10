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

Populated `.log` files from executed paired runs land in `docs/plans/receipts/p3-classification-cost-ab/`.

## Verdict shape (ClassificationCostStanding)

Five-valued only — never collapse to a Boolean “faster”:

`Improved | Regressed{delta} | WithinExistingEnvelope{observed} | ExceededExistingEnvelope | EvidenceIncomplete`

`Regressed` and `WithinExistingEnvelope` can hold **simultaneously** (the likeliest outcome); that combination is mergeable but licenses no CI-cost claim.

## Per-group comparison (composition control)

Comparing average per-group cost between two floor runs is confounded by **group mix**, not just host variance. Example from held #8055 work: one diff’s affected set ran 441 entry groups; a std-touching diff ran 1050 — different long-tail composition makes naive averages incomparable.

**Method (witty-raven-412):** compare per-group cost on the **intersection** of group identifiers present in **both** runs. Same subjects by construction; no extra run required (~70 min contended floor each).

The `incident` probe subject (two-entry roster) is the discriminating same-subject harness; corpus-scale `discovery` runs need intersection analysis when group counts differ.

## Calibration (2026-08-10, valiant-ant-57)

- **Host variance on identical work:** ~1.12× (not ≥1.76× — that was a mid-batch snapshot, retracted). Five of six falsifier lanes reproduce within ±3% across two runs on different days.
- **±3% is measured for falsifier lanes only.** Whether floor discovery groups reproduce that tightly is **unmeasured** — treat as inference if used.

## Fleet log retrieval trap

`gh run view --job <id> --log` **truncates the tail** of long steps (where typed refusals are written). Use:

```bash
gh api repos/gunb-ai/gunbc/actions/jobs/<id>/logs --allow-escape-sequences
```

Without `--allow-escape-sequences` the API writes 0 bytes. Do not compare completeness by byte size (CLI prefixes every line) or by `Complete job` surviving truncation — compare line counts or grep the terminal diagnostic by name.
