# SG-3-CASCADE — cascade-only determination (no §10.0 worksheet) — 2026-05-31

**Worker session:** `sunny-ibex-617` (work item `node://adhoc-72cdfd15-8ec`).
**Parent:** `keen-heron-687` (Target Realization Manager — authority over `TargetAtomRealization`, `TargetTypeExpressionProjection`, `TargetCollectionRealization`, per-language realization rows per PR #3938 §11.1).
**Task:** "SG-3-CASCADE §10.0 worksheet **or** cascade-only receipt — trait/field/binary mop-up per #4086."
**Authority:** #4086 §5 routing table (SG-3-CASCADE = "EXISTING — mop-up after primaries"); `proud-pike-680` routing receipt msg_6db2dc9e; `docs/planning/v4-predicate-dependency-graph-2026-05-31.md` §3.3 (SG-3-CASCADE row left as unnamed "extend existing worksheet" — the routing ambiguity this receipt resolves).

---

## §0 Determination (single authority)

**SG-3-CASCADE gets NO §10.0 worksheet. It is a cascade-only residual class.**

Every error band attributed to SG-3-CASCADE is the rustc-visible *consequence* of a missing modeled
fact that is **already named and already owned by a primary class** (SG-1-FOLLOWON, SG-2,
SG-RC-LAYERING, SG-8, SG-COLLECTION-PROJECTION). SG-3-CASCADE introduces **no independent missing
modeled fact** of its own — there is no carrier in `std/` it is the realization gap for, and therefore
no DFS path and no canonical home to anchor a §10.0 worksheet on.

A §10.0 worksheet exists to name a missing modeled fact and route its substrate landing (see the
SG-RC-LAYERING worksheet, `docs/planning/v4-sg-rc-layering-worksheet-2026-05-31.md` §10.0:
`SG class → Representative emitted failure → DFS path → canonical home`). SG-3-CASCADE has nothing to
put in those fields that is not already in another class's worksheet. **Authoring one would invent
substrate work that does not exist** — the same anti-pattern as cementing a heuristic to satisfy a
count ratchet. The ratchet (rustc-error population) is downstream of the primary realizations, not a
path to them.

This **resolves the predicate-dependency-graph §3.3 ambiguity**: the SG-3-CASCADE row's
"extend existing worksheet" is incorrect — it extends *no* worksheet. It is **retired as a dispatchable
class** and tracked only as a derived residual meter (see §4 falsification criterion).

---

## §1 Measurement basis

This receipt does **not** re-run the M1 probe. The current live ratchet meter is the committed
post-#4115 probe (`docs/audit/v4-rustc-error-catalog-2026-05-31-post-p5.m1-probe-summary.txt`,
PR #4122, `origin/main` at `d015b76dd` — two docs/substrate commits below current HEAD `f7ce371c6`),
which is the authoritative current measurement. Four of the five SG-3-named bands are **pinned (zero Δ)**
across the #4086 → #4122 interval; the fifth (`E0560`) ticked **+4**, and that tick is itself a
*primary-driven* motion (new SG-2 field-projection sites from the #4105/#4107 substrate landings — see
§2), not independent SG-3 growth. A fresh probe would regenerate the same numbers and add no measurement
signal. (M1-probe re-run was additionally infeasible at author time — the build host was
fork-exhausted: ~8k live processes, `fork: Resource temporarily unavailable`. This receipt is an
analytic classification, not a new probe.)

SG-3-CASCADE bands, from the #4122 §2 histogram (the "trait/field/binary mop-up" of the task title):

| Band | rustc code | Count (#4122) | Δ vs #4086 |
| ---- | ---------- | ------------:| ----------:|
| trait bound not satisfied | `E0277` | 330 | 0 |
| expected type, found variant | `E0573` | 159 | 0 |
| struct field missing | `E0560` | 122 | +4 |
| binary op on `Rc<T>` | `E0369` | 110 | 0 |
| placeholder `_` in signature | `E0121` | 44 | 0 |
| **named-band subtotal** | | **765** | +4 |
| unclassified-E0308 mop-up | `E0308` (residual) | ~625 | (residual) |
| **SG-3-CASCADE total** | | **~1,390** | −46 |

Both motions in this table are primary-driven, not SG-3-intrinsic: the named bands are flat except
`E0560 +4` (new SG-2 projection surface from substrate landings), and the residual-E0308 mop-up
**shrank −46** *because* SG-RC-LAYERING absorbed +80 of what was previously counted as E0308 mop-up
(the −46 is net of the +4 E0560 tick and other re-bucketing). That motion is
itself the signature of a derived class: SG-3-CASCADE's count moves as a *function of the primaries'
reclassification*, not on its own.

---

## §2 Per-band traceability — each band → a primary's missing fact

The cascade-only claim is falsifiable per band: if any band carries a missing fact **not** owned by a
primary, that band would justify a worksheet. None does. Grounding is rustc-code semantics + the #4122
catalog's own per-code concept attribution + the owning worksheet/dispatch already on record.

| SG-3 band | Why it fires (rustc semantics) | Upstream missing fact | Owned by | Owning worksheet / dispatch |
| --------- | ------------------------------ | --------------------- | -------- | --------------------------- |
| `E0277` trait bound | `T: Bound` fails because the operand's *realized type* is wrong (e.g. a `Symbol` where `String: Display` was expected, or an `Rc<T>` where the bound is stated on `T`). Trait **impls themselves are emitted from substrate** — the bound set is not the gap; the operand type is. | atom-/signature-realization + reference-layering | SG-1-FOLLOWON / SG-RC-LAYERING | #4099 (SG-1b), #4100 (SG-RC) |
| `E0573` expected type, found variant | A path resolves to an enum *variant* where a *type* was expected — a name/path projection defect, not a missing concept. | path / name resolution projection | SG-8 | SG-8 dispatch (#4086 §5) |
| `E0560` struct field missing | A struct literal carries a field the struct *definition* does not declare (or vice-versa): the type-def emit and the literal emit derive the field set **independently** from the same carrier — the SG-2 type-expression-projection signature. | type-expression / field-set projection | SG-2 | #3962 (SG-2 worksheet) |
| `E0369` binary op on `Rc<T>` | `a + b` / `a == b` where an operand is `Rc<T>` instead of `T` — a pure over-wrap at the operand boundary. | per-use-site reference layering | SG-RC-LAYERING | #4100 |
| `E0121` placeholder `_` in signature | An emitted signature carries `_` where a concrete type belongs — signature realization incomplete. | function-signature realization | SG-1-FOLLOWON / SG-1b | #4099 |
| residual `E0308` mop-up | Second-order type mismatches: a body returns the wrong type *because a callee's type is wrong upstream*. Defined as the E0308 **not** soaked by SG-1-FOLLOWON / SG-RC-LAYERING / SG-COLLECTION-PROJECTION — i.e. the fixpoint tail of the primaries. | (no independent fact — fixpoint artifact) | all primaries | n/a (dissolves with primaries) |

**Every row resolves to a fact another class already owns.** No row names a carrier in `std/` for which
SG-3-CASCADE would be the realization gap. The DFS-path / canonical-home fields of a §10.0 worksheet
would be empty (or duplicate another worksheet's) — which is the operational definition of "no worksheet."

---

## §3 Why not "extend an existing worksheet" either

The predicate-graph §3.3 routing said "extend existing worksheet," but the bands do not extend a
*single* worksheet — they fan out across **five** primaries (SG-1-FOLLOWON, SG-2, SG-RC-LAYERING, SG-8,
SG-COLLECTION-PROJECTION). There is no host worksheet to extend without either (a) duplicating the same
fact into a second worksheet, or (b) arbitrarily attaching a multi-origin residual to one primary it
only partially belongs to. Both are parallel-ledger anti-patterns. The correct routing is: **the bands
are already covered by their primaries' worksheets**; SG-3-CASCADE needs no row of its own in the
dispatch table.

---

## §4 Falsification criterion (how this receipt can be proven wrong)

SG-3-CASCADE is now a **derived residual meter**, not a dispatchable class. The cascade-only claim
predicts:

1. **Monotonic dissolution.** As each primary closes (SG-1-FOLLOWON via TR-lane realization under
   `keen-heron-687`; SG-RC-LAYERING #4100; SG-2 #3962; SG-8; SG-COLLECTION-PROJECTION via SG-5/SG-6),
   the SG-3-CASCADE total must **shrink monotonically toward ~0 with no new substrate authored for
   SG-3 itself**. Each named band should fall in lockstep with its owning primary's burn-down.
2. **No residual floor requiring new substrate.** When all five primaries are closed, the SG-3-CASCADE
   residual must reach ~0. **If a non-trivial floor remains after the primaries close**, that floor is a
   *newly surfaced* independent missing fact — at which point (and only then) a worksheet is justified
   for that specific floor, authored against whatever carrier it implicates. Until then, no worksheet.

The check is mechanical: on the next post-primary-close M1 probe, confirm `E0277 + E0573 + E0560 +
E0369 + E0121 + residual-E0308` tracks the primaries down. The next probe owner (Close/Receipt lane)
records the band deltas against this receipt; a band that *fails* to track its primary is the
falsification signal.

---

## §5 What this receipt is NOT

- **Not a §10.0 worksheet.** It is the determination that no worksheet is warranted — the explicit
  "cascade-only receipt" arm of the task.
- **Not a worker dispatch.** The SG-3 bands are dispatched *through their primaries'* worksheets
  (#4099 / #4100 / #3962 / SG-8 / SG-5-SG-6). No SG-3-specific worker should be spawned.
- **Not a new probe / not a ratchet re-measurement.** #4122 remains the live ratchet meter; this receipt
  consumes it, does not replace it.
- **Not a reclassification of any primary.** It does not move E0277/E0573/E0560/E0369/E0121 out of their
  primary attribution — it confirms they are *consequences* of those primaries and removes SG-3-CASCADE
  as a separate dispatch surface.
- **Not a SG-1 / SG-7 reopening.** Both stay closed (E0423 = 0; v2 emit diagnostics = 0) at #4122.

## §6 Related artifacts

- `docs/audit/v4-rustc-error-catalog-2026-05-31-post-p5.md` (PR #4122) — live ratchet meter; §2 histogram and §3–§5 primary-band breakdowns this receipt traces against.
- `docs/audit/v4-rustc-error-catalog-2026-05-31.md` (PR #4086) — post-SG-1 baseline; §5 routing table that named SG-3-CASCADE "mop-up after primaries."
- `docs/planning/v4-predicate-dependency-graph-2026-05-31.md` §3.3 — the routing table whose SG-3-CASCADE "extend existing worksheet" ambiguity this receipt resolves.
- `docs/planning/v4-sg-rc-layering-worksheet-2026-05-31.md` (#4100) — the §10.0 worksheet shape this class was measured against and found to have nothing to fill.
- PR #4099 (SG-1b) / #3962 (SG-2) — owning worksheets for the E0121/E0277 and E0560 bands.
- `proud-pike-680` routing receipt msg_6db2dc9e — canonical primary-class routing source.
