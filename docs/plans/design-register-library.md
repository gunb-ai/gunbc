# Hierarchical design-register library

Status: **draft for operator review** (DOC-ONLY — no code lands from this doc; no edits to `dag/gunbc/site/` while PR [#6927](https://github.com/gunb-ai/gunbc/pull/6927) is in review). **Follow-on to** [gunb.ai site subsumption](gunb-ai-site-subsumption-design.md) (the in-flight PR that proves the register on one surface). Phase A implementation **fires on #6927 merge** — that PR's dissolution trigger (cutover receipt at `https://gunb.ai/`) is independent; this lane's Phase A trigger is **#6927 merged to main** with the `gunbc.site.*` carriers stable.

## 1. The displaced cost

PR #6927 (`register_principles` → `tokens` → `interaction` → `mark` → `constellation` → `moodboard`, plus `starlight` / `theme_transition` / `accent_study`) proves the register **by execution** — 24+ witness clauses, keystone green. The defect is **layer fusion** (§3): `gunbc.site.*` currently mixes:

- **L1/L2 machinery** (laws, `Material`, `Theme`, `BuildRule`, coverage witnesses) with
- **L3 identity** (G-class starlight accent, Confluence mark selection) and
- **L5 instance** (gunbc layer-DAG constellation facts, moodboard composition, copy).

A second consumer — product UI / dashboard surfaces, self-host artifact pages, a future docs site — would today **fork** `tokens.dag` or import `gunbc.site.*` and inherit gunb.ai identity. The lift extracts L1/L2 (and eventually L4) into a **site-independent library**; gunb.ai becomes consumer #1. Denominated benefit (§6): **a new brand is a handful of L3 rows; a new site kind is an L4 archetype row; neither forks the register.**

## 2. Layer model (operator-aligned 2026-07-21)

Mirror the `std ← extdeps ← compiler ← workflow` shape: cited facts low, house machinery mid, product bindings high. Strict import DAG — each layer imports only strictly lower layers.

```text
L5  site instance     (register × identity × archetype × content) — a Realization (§2)
 ↑
L4  surface archetypes   MarketingSite | DocsSite | Dashboard | ArtifactReport
 ↑
L3  identity            accent row · material family · mark — per-brand residue
 ↑
L2  register machinery  ThemeRole · Material · Theme · timing · coherence witnesses
 ↑
L1  house register      taste invariants as WALLS — laws + enforcing carriers
 ↑
L0  perceptual law      cited universal facts — extdeps-shaped, zero house opinion
```

### L0 — Perceptual law (extdeps-shaped)

**Owns:** frameworks cited by name and version — modeled for what the upstream authority actually states, never re-coined (§3).

| Fact family | Cite (target) | Library use |
|---|---|---|
| Luminance contrast | WCAG 2.x (computed contrast ratio for text/UI pairs) | replaces the HSL lightness-gap proxy (see below) |
| Perceptual uniformity | OKLCH ΔL (or cited equivalent) where WCAG is insufficient | optional second oracle, one authority |
| CVD separation | cited color-vision deficiency separation guidelines | categorical palette validation (feeds L4 Dashboard) |
| Vestibular safety | `prefers-reduced-motion` / vestibular disorder guidance | motion-duration ceilings; instant-fallback is derived, not optional removal |
| Readability | typographic readability ramps (size × line-height × measure) | craft-floor type scale (site subsumption §4i), grounded once |

**Does NOT own:** the glare envelope (`bright ∨ chromatic, never both`) — that is an **L1 house wall**, enforced in L2. L0 supplies perceptual inputs; L1 decides house admissibility.

**Contrast proxy debt (must dissolve):** today's `theme_contrast_ok` in `tokens.dag` checks HSL `lightness_gap >= 40` (text/canvas) and `>= 20` (figure/canvas). That is a **non-perceptual proxy** — it can green while real contrast fails (e.g. same-hue pairs with large lightness delta but inadequate chromatic contrast). Phase A re-grounds contrast at **L0 as computed WCAG** (or OKLCH ΔL once cited); `theme_contrast_ok` becomes a thin call into L0, and the lightness-gap-only law **dissolves** with a RED witness (pair that passes proxy, fails WCAG).

**Morgan–Keenan:** stellar classification is cited physics and may live in `extdeps/perception/` as reference rows. The **gunb.ai accent choice** (G-class live, study candidates O/A/G/K) is **L3 identity**, not library — see §3.

### L1 — House register (walls)

**Owns:** taste invariants as **construction walls** — each law a row naming its enforcing carrier (`RegisterPrinciple.enforced_by`), same pattern as `register_principles.dag`.

| Law (from #6927) | Wall | Enforcing carrier (today) |
|---|---|---|
| Glare envelope | bright **OR** chromatic, never both | `quiet_envelope` — rejected `hsl(194 88% 76%)` is the RED control |
| Still until touched | no ambient motion | `MotionTrigger` has **no ambient variant by type** |
| Chroma by area | saturation ceilings per `AreaClass` | `chroma_admissible` / `area_max_sat` |
| Theme coherence | total role→material assignment | `theme_gaps` countable (mirrors interaction coverage) |
| Behavioral intricacy | no dead hovers; responses derived | `coverage_gaps` in `interaction.dag` |
| Derived truth | intricate output from real facts | archetype/L5 fact-row parameterization |
| Universal reframe | theme flip on one clock | `reframe_page` token + derived `html.reframing` rules |

Site-independent, **brand-owned** (the house register is gunbc's instrument register, not gunb.ai's marketing palette). A second brand still imports L1; it does not fork it.

### L2 — Register machinery

**Owns:** the typed vocabulary and witnesses — **lift moves, does not rewrite** (receipt: byte-identical emission after Phase A).

From `tokens.dag` (already implemented in #6927):

- 11 `ThemeRole`s + `role_area` / `role_var`
- `AreaClass` (`PointArea` | `HairlineArea` | `TextArea` | `FieldArea`)
- `Material`, `Theme` as **total** `role → material` assignment
- timing scale (`respond_fast`, `reveal_deliberate`, `restore_gentle`, `reframe_page`)
- coherence witnesses: `theme_gaps` (totality), `theme_area_coherent`, `chroma_admissible` (admissibility), `theme_contrast_ok` (→ L0 after re-ground)
- `BuildRule` / `MotionTrigger` / `realize_behavior` fold

From `interaction.dag`: closed `Verb` × `Response` × `TimingToken` rows + `coverage_complete`.

From `theme_transition.dag`: generic reframe derivation (`reframe_css`, `unthemed_color_decls` census).

**Target home (proposed):** `dag/gunbc/design/` — module prefix `gunbc.design.*` (peer of `gunbc.site`, below `std`/`extdeps` imports). Split into cohesive modules (`principles`, `material`, `interaction`, `theme`, `emit_rules`) — paths are discriminators, not gospel (§3).

### L3 — Identity (per-brand residue)

**Owns:** the rows that make one brand recognizably *it* among house-register-compliant themes:

- **accent row** — for gunb.ai: Morgan–Keenan **binding** (G-class sun-gold live; `accent_study` candidates) stays in `gunbc.site.identity`, not the library
- **material family** — named materials (`night_canvas`, `warm_starlight`, `verdigris_patina`, …)
- **mark** — geometry spec + selected candidate (`mark.dag` Confluence A)

**Design goal:** a new brand is a **handful of L3 rows** atop L2 machinery — new `Theme` assignments + accent + mark params, zero edits to L1/L2.

### L4 — Surface archetypes (per site-kind)

**Owns:** parameter sets keyed by **site kind**, mapping content roles to layout patterns and declaring which `AreaClass` values dominate.

| Archetype | Dominant register | Layout character |
|---|---|---|
| `MarketingSite` | `FieldArea` + `PointArea` figures | centered column, hero constellation, quiet D0/D1 |
| `DocsSite` | `TextArea` + hairline structure | readable measure, specimen panels, spine nav |
| `Dashboard` | text + **data visualization** | dense tables, status chips, time series |
| `ArtifactReport` | ledger/receipt emphasis | proof rows, digest copy, minimal chrome |

**Second-consumer discipline (§6):** L4 rows land **with the second real site**, not speculatively. Named near-term second consumers: roadmap dashboard (already HTML-emitted), self-host artifact pages. Until then, gunb.ai moodboard remains the L5 composition reference; archetypes are **named scaffolds** with dissolution triggers, not implemented modules.

#### `DataVizArea` extension (Phase C)

The marketing register's quiet envelope **rightly forbids** saturated categorical colors on large fields. Dashboard needs **mutually distinguishable series/status colors** — model as:

- new `AreaClass` variant: `DataVizArea` with its **own** saturation/brightness ceilings (distinct from `PointArea`, still under L1 glare envelope unless a cited L0 exception applies)
- under the `Dashboard` archetype only: a **validated categorical/status parameter set** (N series colors, pairwise CVD-separation check via L0)
- **never** an ad-hoc envelope exemption on `FieldArea` — that would be a §5 fail-open (widening the wall instead of naming the precision frontier)

### L5 — Site instance

**Owns:** the product surface as a **Realization** (§2):

```text
SiteInstance = Register(L1+L2 library) × Identity(L3) × Archetype(L4) × Content
```

- **Register** — import `gunbc.design.*` (after lift)
- **Identity** — `gunbc.site.identity` rows
- **Archetype** — `MarketingSite` for gunb.ai (default until L4 lands)
- **Content** — copy, layer-DAG facts (`constellation` stars), pages (`moodboard`, `accent_study`), `site_workflow` → Pages

Four references plus content facts. Hosting (`roadmap_static_site.SiteArtifact`, generated `pages.yml`) is a **second realization handler** of the same allocation spec — not a sixth layer.

## 3. Override semantics

Defaults flow **down** the hierarchy; **nearest-ancestor wins** for parameters.

| Kind | Override rule | Example |
|---|---|---|
| **Laws (L1 walls)** | **no override path** — coherence witnesses run on every instance regardless of brand/archetype choices | cannot exempt gamer-cyan on one page |
| **L2 parameters** | site may tighten, never loosen without typed `Exempt { reason }` counted in witnesses | shorter `reframe_page` duration |
| **L3 identity** | replaces brand defaults; must still pass L1+L2 witnesses | swap G-class for K-class accent |
| **L4 archetype** | selects layout/AreaClass dominance; cannot disable L1 motion law | `Dashboard` enables `DataVizArea`, not ambient animation |
| **L5 content** | fact rows only; no parallel color literals | constellation star positions |

**Fail-closed tell:** a leaf override that compiles only by editing a witness predicate is a hard reject (§5) — the wall moves to L1 or the override is refused.

## 4. Lift-not-fork discipline (§3)

| Current (`gunbc.site.*`) | After lift |
|---|---|
| `register_principles` | `gunbc.design.principles` (L1) |
| `tokens` | `gunbc.design.material` + `gunbc.design.theme` (L2) |
| `interaction` | `gunbc.design.interaction` (L2) |
| `theme_transition` | `gunbc.design.theme` reframe fold (L2) |
| `mark` geometry | `gunbc.design.mark` spec (L3 shared) + identity binding |
| `starlight` cite rows | `extdeps.perception.*` (L0) — optional shared cite |
| `starlight` accent binding | `gunbc.site.identity` (L3) |
| `constellation` generator | `gunbc.design.archetypes.constellation` (L4, parameterized) |
| `constellation` layer facts | `gunbc.site.facts` (L5) |
| `moodboard` / `accent_study` | `gunbc.site.*` compose-only (L5) |

**Phase A bar — emission-identical witness (§5):** after L1/L2 lift + site re-import, the moodboard (and accent study) emissions are **byte-identical or behaviorally identical** to pre-lift, proven by execution. The existing `site_register_witness_test` clauses remain green; a new `design_register_lift_parity_witness` compares digest before/after. No silent drift.

**While #6927 is open:** this doc names the lift only; **do not move or edit** `dag/gunbc/site/`.

## 5. Phases

| Phase | Trigger | Work | Acceptance |
|---|---|---|---|
| **A — Lift L1/L2 + re-ground site** | **#6927 merged** | extract `gunbc.design.*`; gunb.ai imports; L0 contrast re-ground; dissolve lightness-gap proxy | emission-identical witness green; all #6927 register witnesses green on new import paths |
| **B — Archetype layer** | **second real site** scoped (dashboard HTML or docs) | `MarketingSite` / `DocsSite` / `Dashboard` / `ArtifactReport` rows; parameterized constellation | second site uses L4 + L3 only; zero `gunbc.design` edits beyond shared scales |
| **C — `DataVizArea`** | Dashboard archetype consumer live | new `AreaClass` + validated categorical/status set under L1 walls | marketing pages refuse `DataVizArea` on hero fields; dashboard series colors pass L0 CVD check |

DOC-ONLY until Phase A trigger (#6927 merge). Phases B/C are design-named; no L4 module work until B's second consumer is scheduled.

## 6. Second-consumer discipline (§6 purity trap)

| Layer | Lift when | Named consumers |
|---|---|---|
| L0 | Phase A (contrast re-ground) | all themes |
| L1/L2 | Phase A | gunb.ai site + **product UI / roadmap dashboard** + **self-host artifact pages** |
| L3 | per brand | gunb.ai first |
| L4 | Phase B only | second site (dashboard or docs) |
| L5 | per site | gunb.ai (#6927) |

Do not speculatively implement L4 archetypes for a site that does not exist — scaffolds name a dissolution trigger (`second_site_scheduled`).

## 7. Witness split (post-lift)

- **Library witnesses** (`gunbc.design.*_witness_test`): gamer-cyan refused, saturated field refused, theme gap located, dead hover gap, `MotionTrigger` has no ambient, unthemed-color census, confluence theorem — **no gunb.ai strings**
- **Site witnesses** (`gunbc.site.*_witness_test`): moodboard preconditions, G-class == live night figure, page emission byte checks, layer-DAG authority star
- **Lift parity witness** (Phase A): digest equality before/after module move

## 8. Sibling docs (no dual representation)

| Authority | This doc's relationship |
|---|---|
| [gunb.ai site subsumption](gunb-ai-site-subsumption-design.md) | **Proves** the register (#6927); owns §4k checklist, Phase 0–3 hosting, component vocabulary §4f |
| `roadmap_static_site.dag` | Hosting realization for L5 artifacts |
| DESIGN.md §2–§5 | Governing — Realization, single authority, fail-closed, construction over validation |

This doc does **not** duplicate the §4k seventeen-row checklist or accent-study narrative.

## 9. Open questions (escalate if blocked)

- **Module home:** `gunbc.design.*` vs `std/design_register/` — proposed `gunbc.design.*` because the register is product-house vocabulary, not a universal framework. Escalate if operator wants std placement.
- **WCAG vs OKLCH:** pick one primary L0 contrast oracle for Phase A; the other is optional second cite.
- **Mark in L3 vs shared L2 geometry:** Confluence geometry is brand-neutral; only candidate *selection* is L3. Shared `gunbc.design.mark` spec is likely.

## Dissolution trigger (DESIGN §6)

This document dissolves when Phase C is complete **or** when the library carriers subsume all sections and this md re-registers as a `gunbc.plan.Plan` row. Until Phase A lands, the mark on the carrier is this file plus the #6927 `gunbc.site.*` modules.
