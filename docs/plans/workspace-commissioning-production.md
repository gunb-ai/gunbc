# Initial workspace commissioning: implementation standing

This branch extends allocation integration `7e64f4bcd7a0b93d88e82577596c26c1bd8eadfa`. It is not a commissioned-host receipt. No reservation, VM launch or privileged host apply has occurred.

## Implemented source connections

- Initial admission remains the existing budget/purpose/history/sanitation authority.
- A separate commissioning CAS journal records prepared and committed provenance: slot, plan, revision, controller/image identities, CPU and memory boundary, no-swap policy, sanitation identity and initial generation.
- The existing file-CAS implementation owns conditional publication. Commissioning journal generations never consume allocation/hold generations. Missing or unreadable roots refuse.
- The bounded reconciliation routine composes prepare publication, existing readiness persistence, commit publication, and final readback. Publication rechecks readiness and CAS; restart recovery uses recorded authority and refuses advanced or quarantined readiness while still prepared.
- Preparation and offer production now require a sealed committed readback. Initial admission and a prepared journal do not satisfy that type. Supply rechecks the protected journal and readiness; restoring the original commissioning basis does not reset current cell generation.
- The existing controller installer provisions the separate root-owned journal directory and reads ownership back. It does not create a journal record or initial readiness.

## Still required before a host run

The reconciliation routine is caller-restricted to the installed slot controller's directive path, but **that path does not call it yet**. Its exclusive slot invocation is a required precondition, not an independently implemented lease service. No standalone privileged commissioning CLI is provided.

The remaining subject must bind a reviewed fleet plan to fresh host facts: exact installed runtime/image, 28 GiB/no-swap cell and separate controller budget, withdrawal of competing starts, complete history, slot-scoped sanitation (or recovery of a real prior attempt), and expected journal/readiness heads. It must route that subject through the existing slot controller and preserve exclusion until journal commit/readback. Unknown physical residue must refuse; no invented attempt identity is allowed.

Initial host observations and the reviewed commissioning plan/apply projection remain unfinished. Consequently this source must not be deployed as completed commissioning. The slot remains unavailable until the full production connection and wet acceptance are qualified.

## Qualification

Exploratory controls: 8 transition claims and 4 canonical-record claims passed under 6 GiB/no swap. Preparation and controller composition closures typechecked under the same cap. Those are local source checks over recorded dependency snapshots, not concurrency, filesystem durability, installed-runtime or fleet acceptance evidence. Compiler seal probes and final exact-head checks are ongoing.

Storage prerequisite #12657 remains on review HOLD. Its corrected read-only plan run 36621851623 is awaiting srv1 execution; run 36616988377 must not be applied because its script omitted both installation operations. Runner availability is owned elsewhere; this lane leaves runner services alone.
