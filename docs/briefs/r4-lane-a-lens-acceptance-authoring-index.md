# Lane A PREFIX — Acceptance-authoring index (umbrella)

> **What this is:** the *index only* for the PREFIX lens Acceptance-authoring
> program. It invents no spec prose — it points at the three canonical
> authorities and tracks the sibling Acceptance/keystone PRs for coherence.
> Authored by the umbrella coordination lane (`witty-eagle-676`,
> `node://adhoc-382bf89d-041`) under Lane A manager `fierce-cat-31`.
>
> **Umbrella scope:** program coherence — one interface pin, one witness
> discipline. Batches author and land *their own* Acceptance/keystone PRs as
> siblings; this lane tracks, de-duplicates, and escalates drift. It does
> **not** ship a `src/v4/lens` acceptance harness (that is impl-side /
> T-PB-B census territory).

## Canonical authorities (consume — do not re-spec here)

| Path | Role | On `main`? |
|------|------|-----------|
| `docs/briefs/r4-lane-a-lens-prefix-t23-t12-ci.md` | Source brief `PREFIX-LENS-CI-1` — slices A–C, Acceptance-PR batches, `DISPATCH_HOLD`, P5/SG-0, TEST SURFACE. | Pending — lands via PR #3315 (`session/fierce-cat-31`). |
| `docs/briefs/r4-lane-a-lens-interface-freeze-pin.md` | **Interface pin** — single authority batches author against: frozen carrier digest (`SectionRef`, `EnforcedApplication`, `IntrospectApplication`, `LensEnforcement`/`EnforceableLens`), CLI v0 template, `LENS_ID` registry, §5.1 synthesis default. | Pending — lands via PR #3315. |
| `docs/briefs/r4-lane-a-lens-prefix-acceptance.md` | Acceptance artifact home — runnable AC table + red/green witnesses; evolves per batch, operator-signed. | Pending — lands via PR #3315. |

Until PR #3315 merges, the three paths above are authoritative on
`session/fierce-cat-31`. This index has a **manual depends_on** on PR #3315:
its cross-references resolve once that PR squash-merges to `main`. Any §1–§3
revision of the interface pin requires an **operator-signed** amendment.

## Program map — batches under one interface pin

| Batch | Acceptance scope | Sibling session | Work-item node |
|------:|------------------|-----------------|----------------|
| 1 | Interface-Freeze keystone — pin doc + `application.dag` header + CLI/registry | `swift-crane-263` | `node://adhoc-74ae2595-731` |
| 2 | PREFIX driver / registry + whole-corpus gate (slices A–C) | `crisp-carp-224` | `node://adhoc-3f9728c1-fb6` |
| 3 | Cost + complexity — Wave-1 #1 complexity + §5.1 synthesis (shared `SymbolicCost` algebra) | `smart-ferret-44` | `node://adhoc-a404640e-b00` |
| 4 | Wave-1 remainder — parallelism / effect_enumeration / idempotency + structural readers | `keen-carp-354` | `node://adhoc-b7c5718a-730` |
| 5–6 | Wave-2 dissolution L1.1–L1.12 — coherent sub-batches; coordinate `jolly-ibex-599` | `warm-koi-304` | `node://adhoc-55718e90-2af` |

Batches may split into linked PRs but must stay coherent with the interface
pin + source brief. Implementation dispatch stays `DISPATCH_HOLD` per batch:
workers touch impl only after that batch's Acceptance PR is operator-signed.

## Coherence discipline (the umbrella's standing checks)

- **One interface pin.** Any row asserting v4 parse-tree / field-name shape
  for `EnforcedApplication` / `SectionRef` cites
  `r4-lane-a-lens-interface-freeze-pin.md` §1 — no batch re-specs carriers.
- **Runnable-AC column.** Until the driver binary lands, runnable AC uses
  `TBD — gunbc-prefix-lens-driver v0 …` or the interim
  `v2-compiler compile --source-root src/v4` invocation per source brief
  Fork A. No batch claims the CLI is live without a receipt.
- **Witness immutability.** Red/green witness blocks and expectation rows in
  `r4-lane-a-lens-prefix-acceptance.md` are immutable except red→green
  transitions with an operator-signed amendment. Implementation workers do
  not edit witnesses.
- **Red witness ≠ unmergeable CI.** Negative behavior is a *passing* test
  asserting `DimensionFail` / `Violates`; the whole-corpus job never doubles
  as a "prove diagnostics by failing the job" harness.
- **No parallel spec prose.** New Acceptance substance extends the three
  paths above; this index stays an index.

## Status legend

`Pending` — not yet on `main`. `Signed` — operator-signed Acceptance PR.
`Merged` — landed on `main`. Update this index as batch PRs land so the
program's coherence state has one rendering.
