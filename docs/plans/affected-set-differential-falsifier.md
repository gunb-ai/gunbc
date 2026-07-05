# Affected-set differential falsifier + non-hermetic residue cadence

Status: design draft for operator sign (the cadence surface touches `gunbc.commit_workflow`, load-bearing).
Lane: affected-set (gentle-owl-459). Companion to `interface-summary-declared-use-arity.md` (#6244) and the PR-A typed-refusal rulings.

## What this is — and the trap it is not

When per-PR selection runs only the affected subset, the standing question is: *how do we know the selection is not silently missing edges?* The tempting answer is a nightly rerun-everything "backstop" — which is the DESIGN §5 absorbing fallback wearing a safety costume: it answers doubt with the corpus-denominated superset, its cost grows with the tree, and it zeroes the observable frequency of every selection deficit (nothing is ever *attributed* to a missing edge, because the backstop caught it silently).

The operator's objection (2026-07-04) was exactly this: needing a backstop signals we do not trust the mechanism. Correct. This note replaces the backstop with two components that *earn* trust and then retire the expensive half:

## Component 1 — non-hermetic residue cadence (permanent)

Some witnesses cannot be selected by diff **by construction**: their subjects read the live tree, external services, wall-clock state, or host configuration (`reads_live_tree` dispositions, `floor:host_scaffold` markers, service-transport smokes). No diff-derived affected set can prove them unaffected, because their inputs are not in the diff.

- These form a **named, enumerable class** — the residue roster. Enumeration is mechanical: the same disposition/marker facts the floor runner already consumes (#6224's fail-closed skip policy for live-tree witnesses names most of it).
- The residue runs on **cadence** (off-PR, scheduled), not per-PR — because their trigger is time/environment, not the change.
- This component is **permanent and honest**: it is not a fallback for selection, it is the correct trigger model for witnesses whose inputs selection cannot see. It carries no dissolution trigger.

## Component 2 — differential falsifier (interim, self-dissolving)

While confidence in selection is being established, an off-PR cadence run executes the **cold full floor** and compares it against **selection's predictions** for the same tree state:

- For every witness the selection predicted *unaffected* (would-skip) that the cold run shows *red*: that is a **divergence** — a counted, typed, located finding naming its missing-edge class (import edge absent from the module graph, path→module attribution wrong, provenance gap, non-hermetic subject mis-classified as hermetic). Each divergence files as a work item against the class, not as a rerun.
- A PR-A `Refused` row is **not** a divergence — refusals are already counted, attributed reds; the differential only falsifies confident predictions.
- The comparison consumes: the module-grain closure (#6274's `entry_affected_by_touched_paths`), enrollment kind/span (#6247 — a declared span tells the falsifier *which seams* a witness certifies, so a divergence attributes to a seam, not just a file), and Ruling 1/2 semantics for the empty and refused arms.

**Dissolution trigger:** divergence rate = 0 across N consecutive cadence windows (N to be signed with the cadence; proposed N=8). At that point the differential retires and only Component 1 remains. If divergence recurs later (a new edge class enters the language), the differential re-arms for another N-window cycle — re-arming is itself a counted event.

## Why this is §5-clean

The distinction DESIGN §5 draws: an absorbing fallback is a *failure arm* that widens; a **deliberate interim fallback that is loud, budget-bounded, and lands with its dissolution trigger** is a named neighbor, not the trap. The differential is the latter:

- **Loud:** every divergence is a typed finding with a named class; zero divergences is a published receipt, not silence.
- **Budget-bounded:** off-PR cadence, one cold run per window — cost is fixed per window, not multiplied into every PR.
- **Dissolving:** the trigger is objective (divergence rate), not vibes; the mechanism measurably argues for its own retirement.
- Crucially it **preserves the deficit signal** the backstop would destroy: a missed edge becomes a ranked work item with a frequency, which is what lets §6 price the fix.

## What needs operator sign

1. The cadence surface: a scheduled entry on `gunbc.commit_workflow` / CI workflow (load-bearing carrier) — shape and window length.
2. N (consecutive clean windows) for the dissolution trigger.
3. Confirmation that residue-roster enumeration may reuse the `reads_live_tree` / host-scaffold disposition facts as its authority (no parallel roster).

## Sequencing

Lands after PR-A (consumes its refusal semantics); before the compile-gate wiring PR trusts selection for skip decisions at scale. The falsifier is the evidence engine that lets the opt-in roster dissolve-on (`commit_workflow.dag` enrollment returning to discovery-shrunk-by-affected-set) fire with receipts instead of hope.
