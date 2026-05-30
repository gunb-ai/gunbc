# MW-D8 Wave 1 Exit-Condition Running Ledger — 2026-05-30

**Manager session:** `sharp-otter-407` (Close/Receipt lane).
**Authority:** PR #3983 §7 MW-D8 — Wave 1 exit requires **all five** conditions met. PM dispatched 2026-05-30 ~19:40Z.
**Vocabulary:** two-axis `ship_disposition` × `engineering_state` per PR #3949 §1 (Close/Receipt manager-pass receipt).
**Adjudication rule:** Close/Receipt lane adjudicates whether each cited evidence PR **actually closes** the named condition. A worksheet, scaffold, or substrate landing is NOT sufficient to flip a condition to `PROVEN` — see PR #3949 §1 closure invariant.
**Closes when:** all five rows reach `ship_disposition: PROVEN`. At that point this lane authors a separate Wave 1 receipt-of-complete artifact.

---

## §1. Conditions table (live)

| # | Condition | `ship_disposition` | `engineering_state` | Evidence PR | Last update | Revisit-by |
| - | --------- | ------------------ | ------------------- | ----------- | ----------- | ---------- |
| C1 | R1 produces leaf-model verdict | `PROVEN` | n/a (closure reached) | #3972 merged 2026-05-30 18:24Z | 2026-05-30 19:45Z | n/a — closed |
| C2 | SG-7 `ci.dag` recursion dissolved | `GAP` | `SCAFFOLD_PRESENT` (worksheet only) | #3977 worksheet merged; impl PR in flight by `smart-stag-871` (ETA ~22:50Z) | 2026-05-30 19:45Z | 2026-05-31 12:00Z (24h if impl PR not landed) |
| C3 | `Upsert<T>` landed | `PROVEN` | n/a (closure reached) | #3981 + #3989 merged | 2026-05-30 19:45Z | n/a — closed |
| C4 | `ci_selection_receipt_shadow` generatable | `GAP` | `NO_ARTIFACT_FOUND` | none — `smart-stag-871` queued post-SG-7 (C2 prerequisite) | 2026-05-30 19:45Z | 2026-06-02 12:00Z (≤72h; blocked on C2) |
| C5 | R2a / R2b / R3-external / R3-internal authoring ready-to-run | `GAP` | `PARTIAL_GATE_PRESENT` | #4000 open (`sharp-swift-715` via `quick-tern-735`) | 2026-05-30 19:45Z | 2026-05-31 12:00Z (24h if not landed) |

**Headline:** 2 of 5 conditions `PROVEN` (C1, C3). 3 remaining (C2, C4, C5). C4 depends on C2 closing first.

---

## §2. Per-condition adjudication notes

### §2.1 C1 — R1 leaf-model verdict (PROVEN)

**Evidence:** PR #3972 merged 2026-05-30 18:24Z produces the first R1 leaf-model verdict. Lane adjudication: condition asks for a *verdict*, and the merged PR carries the verdict surface. `ship_disposition: PROVEN`. **Falsification check:** none requested in MW-D8 framing; if a verdict-shape audit later surfaces that the verdict is structurally-only-present-but-not-executed, this row reopens.

### §2.2 C2 — SG-7 `ci.dag` recursion dissolved (GAP / SCAFFOLD_PRESENT)

**Evidence:** PR #3977 merged the SG-7 **worksheet** only. Per PR #3949 §2.1, a substrate-only landing is `SUBSTRATE_CLOSED` at best, **not** `RECEIPT_CLOSED` or `PROVEN`. The condition asks for the recursion to be **dissolved**, which requires the implementation PR landing (in flight by `smart-stag-871`, ETA ~22:50Z). Lane adjudication: **worksheet ≠ dissolution**. The row stays `GAP / SCAFFOLD_PRESENT` until the impl PR lands AND a per-PR census shows the recursion eliminated.

**Falsification check the impl PR should carry:** an executable receipt that the recursive shape is structurally impossible by construction (or at minimum, an audit that the recursive call sites are gone from `ci.dag` HEAD). Without that, the impl PR landing flips engineering_state to `EXECUTION_NOT_WIRED` but ship_disposition remains `GAP`.

### §2.3 C3 — `Upsert<T>` landed (PROVEN)

**Evidence:** PR #3981 + PR #3989 both merged. The condition is "landed"; both PRs are on main. `ship_disposition: PROVEN`. If a follow-on audit finds the `Upsert<T>` substrate has activation debt (no consumer wired), that triggers the anti-shelfware policy under PR #3949 §4 but does NOT reopen this condition — the C3 phrasing was "landed", not "activated".

### §2.4 C4 — `ci_selection_receipt_shadow` generatable (GAP / NO_ARTIFACT_FOUND)

**Evidence:** none currently. `smart-stag-871` queued to author post-SG-7 closure (i.e. depends on C2 flipping to PROVEN first). Lane adjudication: row stays `GAP / NO_ARTIFACT_FOUND` until a generator artifact exists. When a PR lands claiming to generate the shadow, this lane adjudicates whether the generator actually runs (engineering_state: `EXECUTION_NOT_WIRED` if claim-only; `PROVEN` only if a fixture-fired generation receipt accompanies it).

**Dependency note:** C4's revisit-by is set 72h out because C2 is its prerequisite. If C2 slips, C4's window extends pari-passu via this lane's per-PR revisit-by update — not a separate operator extension.

### §2.5 C5 — R2a/R2b/R3-external/R3-internal authoring ready-to-run (GAP / PARTIAL_GATE_PRESENT)

**Evidence:** PR #4000 open (`sharp-swift-715` via `quick-tern-735`). The condition asks for authoring to be **ready-to-run**, which is engineering-state `PARTIAL_GATE_PRESENT` once the authoring scaffold is structurally complete and `PROVEN` once an actual run produces verdicts for each of the four (R2a, R2b, R3-external, R3-internal).

**Lane adjudication:** "ready-to-run" is a four-element AND — a PR landing R2a authoring without R2b/R3-external/R3-internal authoring keeps the row in `GAP / PARTIAL_GATE_PRESENT`. The row flips to `PROVEN` only when all four authoring tracks are runnable.

---

## §3. Update protocol

This lane updates the ledger on these triggers:

1. **In-flight PR lands** — re-adjudicate the affected row, update `ship_disposition` / `engineering_state` / `Last update` / `Revisit-by`.
2. **In-flight PR slips past its ETA** — row stays unchanged but `Last update` advances; if the slip crosses a `Revisit-by` deadline, this lane authors a single dashboard message to the owning manager naming the deadline crossing (not a separate PR).
3. **Closure invariant violation surfaced** — e.g., a row was marked `PROVEN` but a follow-on audit shows the receipt was scaffold-only; this lane reopens the row and amends the adjudication note.

Each update is a single docs commit on this branch (or a follow-on small PR if this PR is already merged). No code, no substrate, no `.dag`.

---

## §4. Wave 1 close ceremony

When all five rows reach `ship_disposition: PROVEN`:

1. This lane authors `docs/audit/v4-wave1-close-receipt-2026-MM-DD.md` — a separate artifact citing each row's evidence PR + the final adjudication.
2. Wave 1 close receipt grade per PR #3949 §2: `RECEIPT_CLOSED` (since by then each row's executable receipt exists and falsification was checked at adjudication time).
3. PR #3983 §7 MW-D8 is then formally met; PM owns the Wave 2 dispatch decision (PR #3983 §5 territory, out of scope here).

---

## §5. What this ledger is NOT

- **Not a dispatch instrument.** This lane does not author or re-route the in-flight worker PRs. C2's `smart-stag-871` impl, C4's queued shadow generator, C5's `sharp-swift-715` authoring all remain owned by their respective sibling managers.
- **Not Wave 2 planning.** PR #3983 §5 owns Wave 2 sequencing; this ledger closes at Wave 1 exit.
- **Not a TASKS.md amendment.** No predicate or operational-authority text is altered.

## §6. Related artifacts

- PR #3983 (`docs/planning/v4-merge-wave-and-next-waves-2026-05-30.md`) §7 MW-D8 — dispatch authority.
- PR #3949 (`docs/planning/v4-close-receipt-manager-pass-2026-05-30.md`) §1 — two-axis vocabulary; §2 — close grades; §4 — anti-shelfware deadlines.
- PR #3973 (`docs/planning/v4-done-predicate-tasks-mapping-2026-05-30.md`) — line-anchor mapping; cross-link for downstream consumers wanting to trace MW-D8 conditions into TASKS.md predicates.
- PR #3972, PR #3977, PR #3981, PR #3989, PR #4000 — Wave 1 worker PRs cited in §1.
