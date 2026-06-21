# Plan — the `Disposition` carrier (confront the skipped modeling decisions)

**Status:** planning tracker · **DESIGN.md + carriers are authority** (§6 no parallel ledger). Linked
from `ROADMAP.md §0`. The first instance of a standing practice: now that `.dag` is mature **and lens
exists** (it did not when the `🟡` comments were written), every modeling ambiguity we used to skip gets
**confronted** — resolved into construction or a justified-terminal lens, not a comment.

**Verified against the live tree 2026-06-21.** Receipts below; re-check before acting.

## 0. The problem — disposition lives in comments, unreadable by lens

A coproduct's *disposition* — is it closed **for a reason** (`Terminal`) or a **stand-in awaiting a
construction** (`Scaffold`) — is recorded today only as freeform comments:

- `// 🟢 TERMINAL — closed by the REST protocol grammar …` (`extdeps/object_storage/object_storage.dag:52`)
- `// 🟡 gated — feature:… — bind node://… ` + `// dissolve-on-arrival: project this corpus directly from the service decl` (`extdeps/*/mock_corpus.dag`)

A lens is a pure reader over the **Node tree**; comments are not Nodes, so **nothing can read these**
(same reason `render.dag` is "not witnessable"). The untagged/indeterminate state — "is this closed for
a reason or just unfinished?" — is the fail-open state, and it is arbitrary (DESIGN §1: arbitrary =
convention standing where necessity was available).

There is also no typed carrier: `Terminal`/`Scaffold` are text only; `src/v2/lens/registry.dag` lists
lenses (`LensRegistryEntryV0`) for **9 of ~35** files with no disposition field.

## 1. The carrier — one concept, two carriers (§2 horizontal, single authority)

Lens-lifecycle tags and coproduct dissolve-markers are the **same concept**. One typed carrier:

```
type Disposition =
  | Terminal { reason: String }                      // legitimately closed/lens-forever (the unstructurable: complexity / cost / necessity)
  | Scaffold { dissolves_to: ConstructionMechanism, bind: NodeRef }   // stand-in until that construction lands
```

Applied to **both** a coproduct `type` decl and a lens registry entry. Single authority — not two
marking schemes. (`dissolves_to` is itself modeled, not a string: the mechanism that will make the
scaffold dead code — e.g. `SingleAuthority`, `RealizationDispatch`, `SubstrateMandatoryTag`.)

## 2. The decision — #1 vs #2 vs middle (proactive vs lens-enforced)

| approach | what | verdict |
|---|---|---|
| **#1 proactive** | can't *define* a coproduct untagged (substrate wellformedness) | the end-state (purest construction), but a **load-bearing §4 substrate change** → escalate; **flag-day** whole-tree migration; **derived** coproducts need disposition derived, not authored |
| **#2 lens-only** | optional tag, a lens checks presence | validation; the weak form |
| **middle (CHOSEN)** | construction-*capable* carrier + lens enforces presence **selectively, ratcheting coverage outward** | rolls out region-by-region; discovers the derived-coproduct cases; proves the taxonomy **by use** before cementing into the grammar |

**Chosen: middle now, #1 as the named end-state.** The lens is itself a
`Scaffold{dissolves_to: SubstrateMandatoryTag}` — when coverage = whole tree **and** the substrate can
require the tag at definition, the lens is dead code and **dissolves**. The enforcement mechanism is
disposed of *by* the discipline (same shape as the numeric-tower guard going dead). Jumping straight to
#1 is rejected only on sequencing (substrate change + flag-day + unproven taxonomy), not on principle.

## 3. What is actually enforceable (construction vs lens vs retro)

- **Presence of a disposition** — *construction.* Non-optional field on the carrier ⇒ you cannot author
  the carrier without committing to `Terminal`/`Scaffold`. The type enforces it; **no meta-lens** (which
  would itself be validation, the rule §0 just adopted).
- **Redundancy not cleaned up** — *fail-closed lens.* RED when a `Scaffold{dissolves_to: X}` coexists
  with an already-present `X` (scaffold + its named successor both exist = §2 parallel-representation
  debt). The one hard gate.
- **Correctness of `Terminal` vs `Scaffold`** — *retro, judgment, NOT a gate.* Classifying "this
  property is structuralizable into a single authority" is the synthesis-feasibility limit /
  leaf-decomposition-diagnosis (both operator-parked open threads). Mis-tagging is caught by a periodic
  **advisory report** (list every `Scaffold`, flag ones whose `dissolves_to` already exists, surface
  `Terminal`s for re-justification), never RED.

## 4. The deepest form, where it applies — dissolve the marker entirely

For the `mock_corpus` scaffolds (`dissolve-on-arrival: project from the service decl`), scaffold and
successor are two **realizations of one corpus interface** — so the ideal is not marker+lens at all but
the §2 Realization pattern: dispatch selects the projected handler when present → the hand-authored arm
is **dead code**. No marker, no lens. The `Disposition`+redundancy-lens is the **residue** for cases
that are *not* a single interface.

## 5. Sequencing

1. Model `Disposition` (+ `ConstructionMechanism`) as a typed carrier in std.
2. Add a non-optional disposition field to `LensRegistryEntryV0`; **complete the registry to all ~35
   lenses** (or move disposition onto each lens module, discovered tree-wide like `*_test.dag`) — this
   roster gap is the prerequisite to making the field non-optional. Each existing lens resolves to
   `Terminal{reason}` (complexity/cost/necessity) or `Scaffold{dissolves_to}` (everything §0 tier-1).
3. Migrate the `🟡` extdeps comments → typed `Disposition` fields, region by region.
4. Land the redundancy lens (§3) over all dispositions; enroll it on the first region.
5. Ratchet the enforced region outward PR by PR.
6. **Escalate** the substrate change (#1: can't-define-untagged) when coverage = whole tree and the
   taxonomy is proven; then the lens dissolves.

## 6. Dissolution trigger (DESIGN §6)

Delete this doc when `Disposition` is a substrate-mandatory field on every coproduct (#1 reached, the
ratcheting lens is dead code) and the redundancy lens is whole-tree — at which point an untagged or
not-cleaned-up coproduct is unwritable and this tracker is redundant.
