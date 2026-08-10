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
# control arm — build at tag peeled from e1c688aec8 (pre-#8055 main)
ctrl-build -- cargo build -p v1-compiler --release --bin p3_cohort_probe
GUNBC_P3_ARM=control GUNBC_P3_SUBJECT=incident ./target/release/p3_cohort_probe 2>&1 | tee control.log

# candidate arm — build at tag peeled from 7f7da93340 (#8055 merge)
GUNBC_P3_ARM=candidate GUNBC_P3_SUBJECT=incident ./target/release/p3_cohort_probe 2>&1 | tee candidate.log
```

Compare `RECEIPT` lines: `classification_production_ms + prep_exec_ms` (candidate) vs `prep_exec_ms` at higher `selected_entry_groups` (control). Assert `head_sha` on each receipt matches the pinned tag.

Env:

- `GUNBC_P3_ARM=control|candidate` — arm label in the receipt
- `GUNBC_P3_SUBJECT=incident|discovery` — subject (default `incident`)

## Pin discipline

Fleet/corpus runs must dispatch against an **immutable tag** (`workflow_dispatch --ref <tag>` — branch or tag name only, never a raw SHA), and every `RECEIPT` line must show `head_sha=<tag target>`. Discard on mismatch.

### Subject pins (declared 2026-08-10, post-#8055 merge)

#8055 merged as `7f7da93340` (2026-08-10T12:57:20Z). `main` has since advanced (`e640f20e61` #8100, `742e634848` #8095). The harness PR branch tracks **current `main`** for probe plumbing only; the **A/B arms do not silently float with `main` tip**.

| Arm | Pin | Rationale |
|-----|-----|-----------|
| **control** | Tag on `e1c688aec8` (`7f7da93340^`, last `main` before #8055) | G1 selector + over-selected prep/exec — exact pre-cutover subject |
| **candidate** | Tag on `7f7da93340` (#8055 merge) | G2 manifest + narrowed prep/exec — exact lane subject |
| **harness tooling** | Current `main` at harness merge (includes probe binary + receipt) | Measurement plumbing; not an A/B arm subject |

**Deliberate split:** #8095 (floor phase journal on falsifier kills) improves kill forensics but is **not** part of the ClassificationCostStanding candidate subject. Forensics validation may run separately on current `main`; paired cost A/B builds use the tags above.

After tagging, verify each run's `head_sha` equals the tag peel before comparing receipts. Use **group intersection** (below) when comparing per-group cost across arms.

### Pre-flight (required before committing two ~3h fleet runs)

`workflow_dispatch` on a tag is a third event shape (baseline resolves as `PushParent`, not `pull_request` merge-target). Selection narrows against a **diff** — if selection is inert (`selected_entry_groups == total_entry_groups` or `selection_state != SelectionApplied`), the candidate arm pays classification with no prep/exec savings and the A/B is harness artifact, not lane property.

**Cheap check:** dispatch **one** arm (candidate tag on `7f7da93340` is enough), then grep the log before interpreting:

```bash
gh api repos/gunb-ai/gunbc/actions/jobs/<job-id>/logs --allow-escape-sequences \
  | rg '\[selection-degradation\] selection_state=|selected_entry_groups='
```

Pass criteria: `selection_state=SelectionApplied` **and** `selected_entry_groups < total_entry_groups`. If not, stop — design needs an explicit diff subject (e.g. `GUNBC_DIFF_WINDOW_PATH` / injected diff) instead of a bare tag dispatch.

**Sequencing:** run behind vivid-gull-155 lane read (`falsifier-pin-8055-7f7da933`, run 31393103250) — not concurrent on the falsifier window.

**Execution owner (2026-08-10):** vivid-gull-155 owns lane read + paired A/B (`bad445b5-6be`) after LanePredictOnly boundary capture on run 31393103250. This session (#8099 harness) holds — no falsifier or A/B dispatch from eager-owl-483. Green read on lane falsifier uses planted-red-row convention (selection-control's one counted divergence = health).

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
