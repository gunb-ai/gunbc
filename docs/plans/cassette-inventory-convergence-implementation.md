# Cassette inventory convergence

**Goal:** assess printed assembly completeness from observed physical inventory. **Method:** desired occurrences from `assembly_inventory.required_printed_parts`, artifact-bound preparation, append-only CAS inventory and independent inspection/readback. **Validation:** identity/refusal/transition controls, then real concurrent publication and controller restart. This stage does not switch the printer runner, start printers or admit powered operation; relation/route/selection migration remains open.

`std.goal_assessment` owns verdicts: read failure→`ObservationRefused`; uncovered slots, active attempts, pending inspection and incompatible revisions remain indeterminate; missing/rejected parts are known deviations, retained alongside unknowns. Satisfaction carries physical identities and inspection evidence. Planning distinguishes print/replacement, observation, reconciliation, inspection and compatibility review; unknowns are never automatically prints or a guessed remedy.

`assembly_inventory_store` owns the prototype slot `cassette-prototype-inventory` under `/var/lib/gunbc/printer-starts`. Reserve all selected slots with their own originating attempts in **one CAS publication**, never a competing partial subset. Baseline `generation:N` comes from store generation, not receipt labels, preventing receipt-name ABA. Unread/malformed/absent stores refuse mutation; initialization is explicit create-if-absent.

Inspection binds slot, compatible revision, physical item, originating attempt, evidence and timestamp. Reject wrong attempts and item reuse across slots; preserve sibling entries. Re-reservation retains rejected identity/revision/attempt/evidence in `retired_rejections`; active **and retired** identities participate in reuse refusal without replaying history. Every generation retains its transition. `capture_validity` owns duplicate/evidence checks separately; `record_inspection` independently rereads publication and reassesses, never treats write success as physical acceptance.

`product.printed_chassis.print_workflow` operations:

| Operation | Contract |
| --- | --- |
| `plan_remaining` | Independent `inventory_file` + `prepared_output` → shadow assessment/candidates. |
| `inventory_initialize` | Validate `capture_file`; create once. |
| `plan_remaining_stored` | Canonical store + preparation → current residential assessment. |
| `reserve_plan` | Verify preparation bytes/membership; fenced atomic reservation by baseline/receipt, no printer command. |
| `record_inspection` | Fenced slot/attempt/item/evidence/time/acceptance transition, reread and reassessment. |

Preparation is an absolute output directory. Hash actual `cad/assembly.json` and sliced projects against `cad_binding`/readback digests; derive membership/attempts from `prepared-queues.json`. Duplicate slots/attempts refuse. Unprepared slots request preparation and cannot reserve/satisfy; no free-form revision is accepted. Revision includes CAD **and exact sliced-project** digests, so repacking/process/CAD edits require explicit compatibility reconciliation, still unimplemented. Identity checks neither rerun CAD/slicer admission nor legitimate forged receipts.

Schema `gunbc.cassette-inventory.v2` refuses old free-label captures; never silently discard rejected history. No production inventory was initialized; a v1 store requires evidence reconciliation and separately authorized migration. Covered-without-entry means observed absence; uncovered means unknown, never inferred from CAD. Accepted/rejected entries need evidence/time, reserved needs attempt, awaiting-inspection needs item+attempt. `accepted` is a decision: unresolved inspection remains pending; its distinct receipt is still owed.

Selected fan facts flow through `RearFanBank`→`FanHarnessPlan`→`HarnessPlan`/cable fold, `selected_fan_harness_json` and `fan_bank_header_load`. P14/P8 have their own facts; unread P12/F9 stays absent. Never borrow P8 values or invent a PST link for one fan. Report lead, route plus service allowance, extension need, native cables excluding unselected extensions, header load and chain-read obligation. Fan review needs no invented power station/switch/DC population; full-stack census waits for those choices. Routes remain lower bounds. [P14 specification](https://www.arctic.de/media/1d/98/04/1693306037/Spec_Sheet_P14_PWM_PST_EN.pdf) read 2026-10-05 is bound by `arctic_p14_wiring_read_receipt`; it proves neither header capacity, chain allowance, final routing nor cooling.

Before actuation cutover:

- Bind selections/compatibility and owner-held mechanical/profile/electrical obligations; implement port/provider relations consumed by review/preparation, then derive routes/bends/service/BOM.
- Make preparation and print-request convergence consume reserved slots; telemetry records completion into inspection-pending without releasing uncertain starts. `printer_actuation` start claims remain guards. Deleted `printer_batch` completion files are not live publishers; restoration requires request-convergence completion receipts. A reservation may be inspected with supplied physical identity; FINISH never accepts a part.
- Add explicit stale-reservation recovery/cancellation and unresolved-inspection receipts; no timeout currently frees slots.
- Execute concurrent store/restart integration and shared effectful convergence, fleet admission first. Pure controls/CAS implementation do not establish integration.

Historical source-interpreter inventory/stack/feed claims and file-backed artifact controls passed; synthetic `/tmp` workflow checks neither initialized inventory nor started printers. Controls include changed bytes with unchanged receipts, digest changes, retired identity roundtrip, unprepared reservations, selected P14 and missing fan facts. Required CI, native emission and concurrent controller integration remain separate evidence.
