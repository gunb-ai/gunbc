# Cassette inventory convergence implementation

This is the inventory/readback portion of the assembly convergence migration.
The relation/route/selection migration remains open. The existing printer runner
has not been switched to inventory-derived actuation, and these operations do
not start printers or claim powered-operation admission.

## Implemented boundary

`assembly_inventory.required_printed_parts` derives one desired slot per printed
occurrence in the selected residential assembly (28 in the current producer).
The review manifests expose those occurrences without inventing a bound
geometry/process revision. `print_workflow.plan_remaining` derives bound revisions from a verified preparation
output for a shadow assessment of an independent inventory capture. A slot without
a prepared project requests preparation; it cannot reserve or satisfy inventory.

`std.goal_assessment` owns the verdict. Read failure is `ObservationRefused`;
coverage gaps, active attempts, pending inspection, and incompatible revisions
remain indeterminate. Missing and rejected parts are known deviations. Known
deviations survive alongside unknowns. A satisfied assessment carries the
observed inventory entries, including physical identity and inspection evidence.

The planner projection distinguishes print/replacement candidates, observation,
attempt reconciliation, inspection, and compatibility review. It never maps all
unknowns to printing. It does not choose a remedy for an unknown cause of failure.

`assembly_inventory_store` uses the existing append-only file CAS store at
`/var/lib/gunbc/printer-starts`, slot `cassette-prototype-inventory`. All requested
parts, each bound to its own originating plate attempt, are reserved in one CAS
publication. A competing writer cannot reserve a subset. The baseline exposed
by stored readback is `generation:N`, derived from the store, not an operator's
receipt label. Receipt-name reuse therefore cannot cause an ABA baseline match.
Unread, malformed and absent stores refuse mutation; initialization is explicit
and create-if-absent only. These operations are scoped to this one prototype.

Inspection names slot, compatible revision, physical item, originating attempt,
evidence reference and observation timestamp. It cannot accept a different
attempt's part or reuse a physical item across two slots. Updating one slot
preserves its siblings. Re-reserving a rejected slot moves its physical item,
revision, attempt and inspection evidence into `retired_rejections` in the current
capture. Both active and retired identities participate in reuse refusal; this does
not depend on a caller replaying CAS history. Every store generation also retains
the prior transition. `capture_validity` owns duplicate/evidence checks separately
from goal assessment.
`record_inspection` independently rereads the published store and reassesses;
it does not use publication success as evidence of physical acceptance.

## Workflow operations

All operations live on `product.printed_chassis.print_workflow`:

| Operation | Inputs | Effect |
| --- | --- | --- |
| `plan_remaining` | `inventory_file`, `prepared_output` | Read an independent capture; emit shadow assessment and candidate work |
| `inventory_initialize` | `capture_file` | Validate and create the prototype inventory once |
| `plan_remaining_stored` | `prepared_output` | Read the canonical store and assess the current residential desired slots |
| `reserve_plan` | `prepared_output`, `baseline`, `receipt` | Verify prepared artifact bytes and assembly membership and atomically reserve its slots/attempts; no printer commands |
| `record_inspection` | `prepared_output`, `baseline`, `receipt`, `slot`, `attempt`, `physical_item`, `observed_at`, `accepted` | Fenced inspection transition, independent reread, reassessment |

`accepted` is a Boolean acceptance/rejection decision. If inspection remains
unresolved, leave the part pending; do not invent an acceptance decision. A
separately recorded unresolved-inspection receipt is still an open migration item.

`prepared_output` must be the absolute directory produced by preparation. The
identity reader hashes `cad/assembly.json` and each actual sliced project, checks
the corresponding `cad_binding` and project digest in its readback, and obtains
slot membership and attempt from `prepared-queues.json`. Duplicate slots or plate
attempts refuse. Inspection must name that prepared slot's attempt. Neither command
accepts a free-form revision. This identity check complements the preparation gates;
it does not independently rerun CAD/slicer admission or make a forged receipt valid.

A revision contains the CAD manifest digest **and the exact sliced-project digest**.
Repacking, changing a slicer process, or changing CAD therefore changes identity.
This is deliberately conservative: a changed project requires explicit compatibility
reconciliation rather than silently treating a previously accepted or rejected item
as interchangeable. That reconciliation operation remains to be implemented.

Capture schema is `gunbc.cassette-inventory.v2`. Earlier free-label captures refuse;
there is no silent migration or dropping of rejected-item history. No production
inventory was created by this work, so no live migration was performed. Any existing
v1 store must be reconciled against retained preparation and inspection evidence
before a separate migration can be authorized.

Capture shape (illustrative hashes, not an observation of the user's hardware):

```json
{
  "schema": "gunbc.cassette-inventory.v2",
  "snapshot": "independent-capture-id",
  "retired_rejections": [],
  "covered_slots": ["M140", "G140"],
  "entries": [
    {
      "slot": "M140",
      "revision": {
        "cad_manifest_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "project_sha256": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
      },
      "status": "accepted",
      "physical_item": "unique-physical-item-id",
      "attempt": "originating-plate-attempt",
      "evidence": "retained-inspection-receipt-or-photo-reference",
      "observed_at": "2026-10-05T20:00:00Z"
    },
    {
      "slot": "G140",
      "revision": {
        "cad_manifest_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "project_sha256": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
      },
      "status": "reserved",
      "attempt": "another-plate-attempt"
    }
  ]
}
```

A covered slot with no entry means observed absence. An uncovered slot means no
observation, even if other slots were observed. Do not mark the whole desired set
covered simply because the CAD lists it. Additional accepted/rejected entries
require evidence and timestamp; reserved entries require the attempt. The
`awaiting-inspection` state requires both physical identity and attempt.

## Selected fan facts

The harness and header-current screen now require the selected fan's lead,
sharing and current facts as inputs; neither imports a P8 value implicitly.
Rear fan candidates carry optional wiring facts. P14 and P8 have their own rows;
unread P12/F9 wiring stays absent. `residential_review.main` now calls `selected_fan_harness_json`: the selected
`RearFanBank` constructs the `FanHarnessPlan` shared by `HarnessPlan` and the fan
cable fold, and calls `fan_bank_header_load`. The emitted review includes lead,
header span plus explicit service allowance, extension need, native fan-cable count (excluding any unselected extension), header
load, and the chain read obligation. Missing candidate facts produce an open result
rather than borrowing P8 values. A single PST fan contributes no fictitious link.

This factors the fan subplan out of the stack plan so the production review need
not invent a power station, DC population or switch placement. The full stack
DC/Ethernet census remains conditional on those selections. Fan route lengths
remain design lower bounds, not verified cut lengths or powered-operation approval.

ARCTIC's [P14 PWM PST specification, p.1](https://www.arctic.de/media/1d/98/04/1693306037/Spec_Sheet_P14_PWM_PST_EN.pdf)
was downloaded/read 2026-10-05: 400 mm lead, four-pin plug and socket, 0.12 A at
12 V. The read receipt retains byte count and SHA-256 in
`gunbc.specification_citation_read_provenance.arctic_p14_wiring_read_receipt`.
This does not establish board-header capacity, allowable chain count, a routed
cable length, or installed cooling performance.

## Still required before actuation cutover

- Bind component choices to selection receipts and add explicit compatibility
  reconciliation for changed prepared-project identities.
- Attach all mechanical, profile and electrical obligations to their owners,
  implement the port/provider relation, and consume it in review/preparation.
- Derive selected cable routes, bends, service volumes and BOM from those
  relations. The older harness lengths remain lower-bound estimates.
- Make preparation and `printer_batch` consume the reserved desired slots;
  reconcile telemetry into inspection-pending inventory without releasing an
  uncertain start. Today the runner's existing hold/start claims remain its guards.
  `printer_batch` writes `.completed-awaiting-inspection.json` at completion, but
  does not yet publish `PartAwaitingInspection` into this store. Inspection can
  consume a reservation with a caller-supplied physical identity; FINISH alone
  still cannot accept a part.
- Add explicit recovery/cancellation of stale reservations and unresolved
  inspection receipts. No timeout currently frees a reservation automatically.
- Exercise real concurrent store publication and restart through the integrated
  controller. Pure transition controls and the existing CAS implementation do
  not establish that end-to-end integration.
- Land the shared effectful convergence cycle with the fleet admission spine as
  its first consumer, as the migration plan requires.

No physical inventory was initialized or accepted during development, and no
printer start was issued by these checks.

## Validation of this stage

18 inventory claims and 12 stack/feed claims pass in the source interpreter.
A separate file-backed integration check admits matching artifact hashes and
refuses changed project bytes with unchanged receipts (two passing controls).
The claims cover digest changes, retained rejected identities through JSON readback,
unprepared reservation refusal, the selected residential P14 harness, missing fan
facts, and the existing inventory and feed controls. The production residential review emits the selected P14
lead (400 mm), route estimate with allowance (470 mm), extension requirement and
120 mA header load with its unresolved capacity. Workflow checks use synthetic
files in `/tmp`; the full `plan_remaining` entrypoint returned 28 desired slots,
27 preparation requests and one observation request for the bound, uncovered slot.
These checks do not initialize inventory or start printers. Required CI,
native emission and concurrent controller/store integration remain separate checks.
