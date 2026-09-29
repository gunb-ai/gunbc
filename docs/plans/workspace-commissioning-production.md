# Initial workspace commissioning: implementation standing

This branch extends allocation integration `7e64f4bcd7a0b93d88e82577596c26c1bd8eadfa`. It is not a commissioned-host receipt. No reservation, VM launch or privileged host apply has occurred.

## Implemented source connections

- Initial admission remains the existing budget/purpose/history/sanitation authority.
- A separate commissioning CAS journal records prepared and committed provenance: slot, plan, revision, controller/image identities, CPU and memory boundary, no-swap policy, sanitation identity and initial generation.
- The existing file-CAS implementation owns conditional publication. Commissioning journal generations never consume allocation/hold generations. Missing or unreadable roots refuse. Both the directory and the selected immutable generation file must be root-owned and protected against other writers.
- The bounded reconciliation routine composes prepare publication, existing readiness persistence, commit publication, and final readback. Publication rechecks readiness and CAS; restart recovery uses recorded authority and refuses advanced or quarantined readiness while still prepared.
- Preparation and offer production now require a sealed committed readback. Initial admission and a prepared journal do not satisfy that type. Supply rechecks the protected journal and readiness; restoring the original commissioning basis does not reset current cell generation.
- The existing controller installer provisions the separate root-owned journal directory and reads ownership back. It does not create a journal record or initial readiness.

## Still required before a host run

The reconciliation routine is caller-restricted to the installed slot controller's directive path, but **that path does not call it yet**. Its exclusive slot invocation is a required precondition, not an independently implemented lease service. No standalone privileged commissioning CLI is provided.

The remaining subject must bind a reviewed fleet plan to fresh host facts: exact installed runtime/image, 28 GiB/no-swap cell and separate controller budget, withdrawal of competing starts, complete history, slot-scoped sanitation (or recovery of a real prior attempt), and expected journal/readiness heads. It must route that subject through the existing slot controller and preserve exclusion until journal commit/readback. Unknown physical residue must refuse; no invented attempt identity is allowed.

Initial host observations and the reviewed commissioning plan/apply projection remain unfinished. Consequently this source must not be deployed as completed commissioning. The slot remains unavailable until the full production connection and wet acceptance are qualified.

## Qualification

Exploratory controls: 8 transition claims and 4 canonical-record claims passed under 6 GiB/no swap. Preparation and controller composition closures typechecked under the same cap. Those are local source checks over recorded dependency snapshots, not concurrency, filesystem durability, installed-runtime or fleet acceptance evidence. At source head `7e4ae78ab02818da0e47241555d2f372d74c5392`, all 12 claims and the controller composition passed under 6 GiB/no swap. The full serve entry reached loopback readiness in 271.405 seconds under 12 GiB/no swap, with no OOM events. That is a local startup check, not deployment or VM acceptance. At `bb9a1d67b6704e2495f9a62d7767d9f505803bcc`, the 12 claims and controller composition passed again under 6 GiB/no swap after record-custody hardening. Three scoped compiler admission probes also passed over the recorded dependency snapshot; their receipt records the intervening indentation-only normalization. These prove constructor-access boundaries, not that every imported compiler diagnostic is absent. No privileged filesystem or concurrent host commissioning test has run.

Storage prerequisite #12657 remains on review HOLD. Its corrected read-only plan run [36621851623](https://github.com/gunb-ai/gunbc/actions/runs/36621851623) succeeded at `28861d7d4a491c89a3c0784f5d3c5e1df950ca44`. Downloaded artifact inspection established both root-owned 0700 directory ensures in the human plan and executable script, with matching host/revision/run/generation receipt fields. No apply has run; required PR checks remain pending. The obsolete run 36616988377 must not be applied because its script omitted both installation operations. Runner availability is owned elsewhere; this lane leaves runner services alone.
