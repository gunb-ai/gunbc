# Cassette inventory convergence implementation

This is the inventory/readback portion of the assembly convergence migration.
The relation/route/selection migration remains open. The existing printer runner
has not been switched to inventory-derived actuation, and these operations do
not start printers or claim powered-operation admission.

## Implemented boundary

`assembly_inventory.required_printed_parts` derives one desired slot per printed
occurrence in the selected residential assembly (28 in the current producer).
The review manifests expose those occurrences without inventing a bound
geometry/process revision. `print_workflow.plan_remaining` binds an explicitly
supplied revision for a shadow assessment of an independent inventory capture.

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
preserves its siblings. Every store generation retains the prior transition.
`record_inspection` independently rereads the published store and reassesses;
it does not use publication success as evidence of physical acceptance.

## Workflow operations

All operations live on `product.printed_chassis.print_workflow`:

| Operation | Inputs | Effect |
| --- | --- | --- |
| `plan_remaining` | `inventory_file`, `revision` | Read an independent capture; emit shadow assessment and candidate work |
| `inventory_initialize` | `capture_file` | Validate and create the prototype inventory once |
| `plan_remaining_stored` | `revision` | Read the canonical store and assess the current residential desired slots |
| `reserve_plan` | `plan_file`, `revision`, `baseline`, `receipt` | Validate existing plate-plan membership and atomically reserve its slots/attempts; no printer commands |
| `record_inspection` | `baseline`, `receipt`, `slot`, `revision`, `attempt`, `physical_item`, `observed_at`, `accepted` | Fenced inspection transition, independent reread, reassessment |

`accepted` is a Boolean acceptance/rejection decision. If inspection remains
unresolved, leave the part pending; do not invent an acceptance decision. A
separately recorded unresolved-inspection receipt is still an open migration item.

Capture shape (illustrative, not an observation of the user's hardware):

```json
{
  "snapshot": "independent-capture-id",
  "covered_slots": ["M140", "G140"],
  "entries": [
    {
      "slot": "M140",
      "revision": "explicit-reviewed-geometry-and-process-revision",
      "status": "accepted",
      "physical_item": "unique-physical-item-id",
      "attempt": "originating-plate-attempt",
      "evidence": "retained-inspection-receipt-or-photo-reference",
      "observed_at": "2026-10-05T20:00:00Z"
    },
    {
      "slot": "G140",
      "revision": "explicit-reviewed-geometry-and-process-revision",
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
unread P12/F9 wiring stays absent. The existing rear-cooling review exposes the
selected lead and bank rated current.

ARCTIC's [P14 PWM PST specification, p.1](https://www.arctic.de/media/1d/98/04/1693306037/Spec_Sheet_P14_PWM_PST_EN.pdf)
was downloaded/read 2026-10-05: 400 mm lead, four-pin plug and socket, 0.12 A at
12 V. The read receipt retains byte count and SHA-256 in
`gunbc.specification_citation_read_provenance.arctic_p14_wiring_read_receipt`.
This does not establish board-header capacity, allowable chain count, a routed
cable length, or installed cooling performance.

## Still required before actuation cutover

- Bind geometry/process revisions and component choices to selection receipts;
  the shadow CLI currently requires an explicit revision from its caller.
- Attach all mechanical, profile and electrical obligations to their owners,
  implement the port/provider relation, and consume it in review/preparation.
- Derive selected cable routes, bends, service volumes and BOM from those
  relations. The older harness lengths remain lower-bound estimates.
- Make preparation and `printer_batch` consume the reserved desired slots;
  reconcile telemetry into inspection-pending inventory without releasing an
  uncertain start. Today the runner's existing hold/start claims remain its guards.
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

- 15 `assembly_inventory_witness` claims pass in the source interpreter,
  including the actual residential producer, independent-read refusal, partial
  acceptance, pending inspection, incompatible revision, duplicate identities,
  all-or-nothing reservation, stale baseline, receipt-label ABA, and retained
  inspection evidence.
- 10 `printed_chassis_stack_feed_witness` claims pass, including the selected
  P14 current control and the existing supply/voltage/harness regressions.
- The `print_workflow.plan_remaining` CLI was exercised against a synthetic
  capture with empty coverage. It derived 28 desired printed slots and emitted
  only observation work, with an indeterminate assessment and no print commands.
- Corpus source parsing and `git diff --check` pass. These local checks do not
  claim required CI, native emission, concurrent live-store integration, or
  physical acceptance of any part.
