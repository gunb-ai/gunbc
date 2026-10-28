# Cassette inventory convergence implementation

Inventory/readback is implemented; relation/route/selection migration remains open.
The printer runner has not switched to inventory-derived actuation. These operations
neither start printers nor admit powered operation.

## Implemented boundary

`assembly_inventory.required_printed_parts` derives one slot per printed occurrence
in the selected residential assembly (28 in the current producer). Review manifests
expose occurrences without inventing geometry/process revisions. `print_workflow.plan_remaining`
binds revisions from verified preparation output and shadow-assesses an independent
inventory capture. Unprepared slots request preparation; they cannot reserve or satisfy inventory.

`std.goal_assessment` owns the verdict. Read failure is `ObservationRefused`;
coverage gaps, active attempts, pending inspection, and incompatible revisions
remain indeterminate. Missing and rejected parts are known deviations. Known
deviations survive alongside unknowns. A satisfied assessment carries the
observed inventory entries, including physical identity and inspection evidence.

The planner distinguishes print/replacement, observation, attempt reconciliation,
inspection and compatibility review; unknowns never all become prints. Unknown failure
causes do not select a remedy.

`assembly_inventory_store` uses the append-only file CAS store at
`/var/lib/gunbc/printer-starts`, slot `cassette-prototype-inventory`. One CAS publication
reserves all requested parts with their originating plate attempts; competing writers
cannot reserve a subset. Readback derives baseline `generation:N` from the store,
so receipt-name reuse cannot cause an ABA match. Unread, malformed or absent stores
refuse mutation; initialization is explicit and create-if-absent. Scope is this prototype only.

Inspection binds slot, compatible revision, physical item, originating attempt,
evidence reference and timestamp. It refuses another attempt's part or reuse across
slots; updates preserve siblings. Re-reserving a rejected slot retains its item,
revision, attempt and inspection evidence in the current capture's `retired_rejections`.
Reuse refusal checks active and retired identities without caller-side CAS replay;
each store generation also retains the prior transition. `capture_validity` owns
duplicate/evidence checks separately from goal assessment. `record_inspection`
independently rereads and reassesses publication; publication success is not physical acceptance.

## Workflow operations

All operations live on `product.printed_chassis.print_workflow`:

| Operation | Inputs | Effect |
| --- | --- | --- |
| `plan_remaining` | `inventory_file`, `prepared_output` | Read an independent capture; emit shadow assessment and candidate work |
| `inventory_initialize` | `capture_file` | Validate and create the prototype inventory once |
| `plan_remaining_stored` | `prepared_output` | Read the canonical store and assess the current residential desired slots |
| `reserve_plan` | `prepared_output`, `baseline`, `receipt` | Verify prepared artifact bytes and assembly membership and atomically reserve its slots/attempts; no printer commands |
| `record_inspection` | `prepared_output`, `baseline`, `receipt`, `slot`, `attempt`, `physical_item`, `observed_at`, `accepted` | Fenced inspection transition, independent reread, reassessment |

`accepted` is Boolean. Leave unresolved inspection pending; a separate
unresolved-inspection receipt remains an open migration item.

`prepared_output` must be the absolute directory produced by preparation. The
identity reader hashes `cad/assembly.json` and each actual sliced project, checks
the corresponding `cad_binding` and project digest in its readback, and obtains
slot membership and attempt from `prepared-queues.json`. Duplicate slots or plate
attempts refuse. Inspection must name that prepared slot's attempt. Neither command
accepts a free-form revision. This identity check complements the preparation gates;
it does not independently rerun CAD/slicer admission or make a forged receipt valid.

A revision binds the CAD manifest digest **and exact sliced-project digest**.
Repacking or changing process/CAD changes identity and requires explicit compatibility
reconciliation before reusing an accepted or rejected item. That operation remains unimplemented.

Schema `gunbc.cassette-inventory.v2` refuses earlier free-label captures; migration
must preserve rejected-item history. This work created no production inventory and
performed no live migration. Any v1 store requires reconciliation against retained
preparation/inspection evidence before separate migration authorization.

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

A covered slot without an entry means observed absence; an uncovered slot means
unobserved. CAD membership alone cannot establish coverage. Accepted/rejected entries
require evidence and timestamp; reservations require an attempt; `awaiting-inspection`
requires physical identity and attempt.

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

The independent fan subplan needs no invented power station, DC population or switch
placement. Full-stack DC/Ethernet census awaits those selections. Fan routes remain
design lower bounds, not verified cut lengths or powered-operation approval.

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
- Make preparation and the print-request convergence (the part-D successor of the deleted
  `gunbc.fleet.printer_batch`) consume the reserved desired slots; reconcile telemetry into
  inspection-pending inventory without releasing an uncertain start. The start claims under
  `/var/lib/gunbc/printer-starts`, written by `gunbc.fleet.printer_actuation`, remain the guards.
  Nothing currently publishes `PartAwaitingInspection` into this store (the batch controller's
  `.completed-awaiting-inspection.json` went with it); the trigger to restore it is the request
  convergence recording a completion receipt. Inspection can consume a reservation with a
  caller-supplied physical identity; FINISH alone still cannot accept a part.
- Add explicit recovery/cancellation of stale reservations and unresolved
  inspection receipts. No timeout currently frees a reservation automatically.
- Exercise real concurrent store publication and restart through the integrated
  controller. Pure transition controls and the existing CAS implementation do
  not establish that end-to-end integration.
- Land the shared effectful convergence cycle with the fleet admission spine as
  its first consumer, as the migration plan requires.

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
These checks neither initialize nor accept physical inventory, nor start printers. Required CI,
native emission and concurrent controller/store integration remain separate checks.
