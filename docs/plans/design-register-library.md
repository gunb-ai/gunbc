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

### L4 — Surface archetypes (formal model)

**Owns:** how a **content graph** becomes a **surface** — not a fourth palette, but a preset over decomposed structural axes plus an emission grammar. L4 references L0–L3 **by name only** (see §2.1); it never copies values.

#### Archetypes are presets, not primitives

The `Hermetic | Wet` precedent applies verbatim (§3 state-space conflation): `MarketingSite` / `DocsSite` / `Dashboard` bundle **independent axes** and must **dissolve into them**, with the three names surviving only as **preset rows** — convenience bundles, not new types.

**Candidate axes** (§2 independence test — net concepts must not grow; derive any axis that proves dependent):

| Axis | Variants | Independence |
|---|---|---|
| **Reading posture** | `Contemplative` \| `Reference` \| `Operational` | independent — the register thesis ("quiet at arm's length") is a **posture** statement |
| **Content shape** | `Narrative` \| `ReferenceGraph` \| `LiveFacts` | independent — drives layout grammar, not color |
| **Liveness** | `StaticEmit` \| `Regenerated` \| `Live` | independent — emission cadence / refresh contract |
| **Interaction verb set** | subset of L2 `Verb` | independent — which verbs the surface admits |

**Density is derived, not an axis:** visual density follows from `posture × content_shape` (e.g. `Operational × LiveFacts` ⇒ dense tables; `Contemplative × Narrative` ⇒ centered column). No separate `Density` enum — adding it would duplicate information.

**Preset rows** (names are opinion; axes are structure):

| Preset | Posture | Content shape | Liveness | Verbs (L2) |
|---|---|---|---|---|
| `MarketingSite` | Contemplative | Narrative | StaticEmit | `Approach`, `Traverse` (hover + navigate) |
| `DocsSite` | Reference | ReferenceGraph | Regenerated | `Approach`, `Traverse`, `Enter`, `Copy` (navigate + search + anchor + copy) |
| `Dashboard` | Operational | LiveFacts | Live | filter, drill, refresh *(extend L2 verb enum at Phase B — not ad-hoc strings)* |
| `ArtifactReport` | Reference | ReferenceGraph | StaticEmit | `Copy`, `Traverse` (ledger emphasis) |

**Decomposition test:** the axis product generates points the three marketing names cannot — `ArtifactReport`, blog hybrids (`Contemplative` + `ReferenceGraph`), status pages (`Operational` + `StaticEmit`) — with **zero new primitives**. That is the proof the decomposition is real.

#### What an archetype formally is

Three components, all referencing L2/L3 **by name** (never embedding `Material` values or HSL literals):

```text
Archetype = RoleDemandProfile × InteractionVerbSet × EmissionGrammar
```

**(a) Role-demand profile** — which `ThemeRole`s and `AreaClass`es the surface **consumes**, with budgets (max figure count, allowed `DataVizArea` slots, luminance budget share). The Dashboard profile is where **`DataVizArea` demand** lives — not an envelope exemption on `FieldArea`.

**(b) Interaction verb set** — a declared subset of the closed L2 `Verb` vocabulary. Coverage law from L1 still applies: every admitted verb has response rows for every interactive element the grammar emits.

**(c) Emission grammar** — rows mapping **content-node kinds** → markup patterns whose regions consume roles. Same shape as language emit: `GrammarRelationRow` / `target_model_edge_translation_rules` (§4 one-grammar-read-both-directions). An archetype is to a content graph what a target language is to a program — an **emission row set**, read backward for ingest when a surface round-trip is needed.

This is the §2 horizontal unification: one grammar, N site kinds — not N hand-rolled page composers.

#### `DataVizArea` extension (Phase C)

The marketing register's quiet envelope **rightly forbids** saturated categorical colors on large fields. Dashboard needs **mutually distinguishable series/status colors** — modeled in the **role-demand profile**, not by weakening L1:

- new `AreaClass` variant: `DataVizArea` with its **own** OKLCH chroma ceiling (Class 2 reground)
- under the `Dashboard` preset only: a **validated categorical/status parameter set** (N series colors, pairwise CVD-separation via L0)
- **never** an ad-hoc envelope exemption on `FieldArea` — §5 fail-open

**Second-consumer discipline (§6):** L4 preset rows and emission grammar land **with the second real site** (Phase B), not speculatively. gunb.ai moodboard remains the L5 composition reference until then.

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

### 2.1 Interface law, propagation, and where opinion lives

#### The interface law (names-not-values)

Every layer exports:

1. a **closed vocabulary** (types, enums, row kinds), and
2. **laws** over that vocabulary (witnesses, refusing constructors).

The layer above references the vocabulary **by name only** — never copies values (no inlined HSL, no duplicated threshold literals, no parallel `ThemeRole` enum). It must satisfy the laws; coherence is **witnessed per L5 instance**.

**Names-not-values** is what makes an L0–L3 edit propagate to every site **by derivation**: change the accent row (L3), reground the glare envelope (L1), or add a WCAG cite (L0), and every `SiteInstance` re-emits from the same graph — no hand hunt through CSS files.

| Layer | Exports (vocabulary) | Exports (laws) | Imports by name from |
|---|---|---|---|
| L0 | perceptual ops, cite rows | computed theorems (contrast, CVD) | extdeps cite tables |
| L1 | `PrincipleId`, wall predicates | `RegisterPrinciple` + `enforced_by` | L0 ops |
| L2 | `ThemeRole`, `Verb`, `Material`, `BuildRule` | `theme_coherent`, `coverage_complete` | L1 walls |
| L3 | `MarkSpec`, accent class ref, material family ids | identity witness bundle | L2 roles/materials |
| L4 | axis variants, preset ids, grammar row kinds | role-demand + verb-set admissibility | L2/L3 names |
| L5 | content node kinds, fact rows | instance coherence (full stack) | L1–L4 names |

#### Propagation is the compiler's own change story

Sites are **emitted from the graph** (§7). Therefore:

- **Edit-at-Ln blast radius = the affected set** — the same machinery as CI witness selection (`v2.lens.affected_set`): a change to `gunbc.design.material` selects every L5 site import-closure that references those names; regen is scoped to that closure, not the whole repo by default.
- **Every site re-runs coherence witnesses** on any upstream law change — a site that breaks under a new OKLCH wall or WCAG re-ground **reds loudly** (typed, located refusal), never silently drifts in committed CSS (§5 fail-closed propagation).
- **No new mechanism** — this is *why* the library lives in the substrate instead of a hand-maintained frontend repo.

#### Where opinion lives (formal / opinionated split)

| Layer | Opinion? | What is signed |
|---|---|---|
| **L0** | **No** — cited / mechanical | upstream standards only |
| **L1** | **Yes** — house taste | walls (§4k checklist rows → `RegisterPrinciple`) |
| **L2** | **No** — mechanical over L1 | machinery is derived; timing tokens are discrete rows, not vibes |
| **L3** | **Yes** — brand identity | accent class, material family, mark selection |
| **L4 axes** | **No** — structural | posture / shape / liveness / verbs are closed enums |
| **L4 presets** | **Yes** — which axis tuple gets a friendly name | `MarketingSite` row = operator-signed bundle |
| **L5** | **Yes** — content | copy, facts, page composition |

**Rule:** opinions are exactly the rows an operator signs; everything else is **derived or cited**. A preset without a sign-off row is a scaffold with a dissolution trigger, not production authority.

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
| **B — Archetype layer** | **second real site** scoped (dashboard HTML or docs) | axis decomposition + preset rows + emission grammar (`GrammarRelationRow`); `DataVizArea` in Dashboard role-demand profile | second site = L3 + L4 preset + L5 content only; hybrid presets (e.g. ArtifactReport) need no new primitives |
| **C — `DataVizArea`** | Dashboard archetype consumer live | new `AreaClass` + validated categorical/status set under L1 walls | marketing pages refuse `DataVizArea` on hero fields; dashboard series colors pass L0 CVD check |

DOC-ONLY until Phase A trigger (#6927 merge). Phases B/C are design-named; no L4 module work until B's second consumer is scheduled.

### Phase A sub-phases (constants + de-stringing — §10)

Phase A is not only a module move. Operator review (2026-07-21) flagged hardcoded constants and string-based modeling in the #6927 stack; the lift sequences de-stringing work by **displaced cost**:

| Order | Sub-phase | Work (§10 class) | Acceptance |
|---|---|---|---|
| **A.0** | Lift + key-typing | extract `gunbc.design.*`; closed enums / symbol refs for element and node keys (Class 3) | emission-identical witness; coverage witnesses total by construction |
| **A.1** | CSS/SVG grammar | `extdeps/languages/{css,svg}` rows; typed `CssDecl` emission (Class 1c) | invalid property/easing unwritable; `paint()` hand-`join()` dissolved |
| **A.2** | Perceptual re-ground | WCAG thresholds (Class 1b) + OKLCH L1 walls (Class 2) | lightness-gap proxy RED; glare/chroma walls use perceptual chroma ceiling |
| **A.3** | Stellar derivation | Morgan–Keenan rows with `temperature_kelvin`; color derived (Class 1a) | no-green witness is computed theorem; hand hue-range check dissolves |

A.0 and A.1 may overlap the structural lift; A.2–A.3 land before Phase B. All remain DOC-ONLY in this PR.

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

## 10. Constants provenance and de-stringing (operator addendum 2026-07-21)

The #6927 stack works but carries three classes of **constants-without-provenance** and **string-as-model** defects. The library lift is the moment to fix them — phased, doc-only here, implementation in Phase A sub-phases (§5).

**Priority by displaced cost:** (1) CSS/SVG grammar rows — every future page benefits, aligns with bash-AST confinement precedent (#6854); (2) OKLCH regrounding — three proxy laws become perceptual; (3) stellar derivation — medium effort, high story value; (4) key-typing rides the lift itself (A.0).

### Class 1 — Cited-fact constants → extdeps (L0)

Facts that already have a real upstream authority. Today they are hand literals; the library grounds them once and **derives** downstream values.

#### 1a. Stellar-class colors (`starlight.dag`)

**Today:** each `StellarClass` row carries hand-eyeballed HSL `hue`/`sat_pct`/`light_pct`; `perceived` is a bare `String`. The no-green witness uses a hand hue-range check (`90`–`170`).

**Target:** each row carries `temperature_kelvin` cited to the Morgan–Keenan framework (letter + effective temperature per the cited tables). **Color is derived**, not stored:

```text
temperature_kelvin → Planckian locus → CIE 1931 chromaticity → sRGB
```

Each step cites a real standard (Planck blackbody, CIE 1931, sRGB transfer). The grounded row stores temperature and letter; `figure`/`lit_variant`/`edge_variant` are **projections** of the derivation under the L1 glare envelope and L2 `AreaClass` ceilings.

**Witness upgrade:** `no_green_stellar_class` becomes a **computed theorem** over the derivation (no blackbody peak in the green band that survives sRGB gamut mapping) — not a magic hue interval. The `90`–`170` range constant **dissolves**.

**Phase:** A.3. **L3 residue:** which class letter is the live accent (G-class for gunb.ai) stays in `gunbc.site.identity`; only the cite + derivation live in L0.

#### 1b. WCAG contrast thresholds

**Today:** `theme_contrast_ok` uses HSL lightness-gap proxies (`>= 40` text/canvas, `>= 20` figure/canvas) — already flagged for dissolution in §2.

**Target:** `extdeps/perception/wcag.dag` (or equivalent) rows citing WCAG 2.x ratio thresholds (`4.5` normal text, `3.0` large text / UI components — exact cite to the spec section). `theme_contrast_ok` becomes `wcag_contrast_ratio(fg, bg) >= threshold_for(role_pair)` over **computed** sRGB luminance from the L0 oracle.

**Phase:** A.2 (paired with OKLCH regrounding).

#### 1c. CSS and SVG vocabulary

**Today:** `CssDecl.prop`, easing names, `scheme`, selectors, and markup `tag` are bare `String`. Emission uses hand `join()` serializers in `tokens.dag` `paint()` / `paint_alpha()` and `theme_transition.dag` rule folds. An invalid property name or easing string is writable and fails only at browser parse time (or never).

**Target:** `extdeps/languages/css/` and `extdeps/languages/svg/` grammar rows on the same pattern as `extdeps/languages/yaml/` — §4 one-grammar-read-both-directions. `CssDecl` carries a typed property/easing variant (or a grammar-owned atom); emission is `serialize_css` over typed decls, not string concat. Invalid property, easing, or SVG attribute becomes **unwritable** at model time.

**Precedent:** bash-AST confinement (`realization_vocabulary_containment`) — language construction vocab lives in `extdeps/languages/`, not workflow/site modules.

**Phase:** A.1 — **highest displaced cost** (every page and dashboard surface inherits).

### Class 2 — House-choice constants → named L1 rows, regrounded perceptually

Facts that are **genuine house choices** (not citeable universal law) but today operate on **non-perceptual HSL proxies**. They stay as L1 walls with explicit provenance rows; the lift **re-grounds the axes** in OKLCH.

| Constant (today) | Location | Proxy problem | Target |
|---|---|---|---|
| `45` / `68` glare cutoffs | `quiet_envelope` | HSL `sat_pct` + `light_pct` | OKLCH **chroma ceiling** + **lightness ceiling** per envelope arm (bright ∨ chromatic, never both) |
| `area_max_sat` per `AreaClass` | `chroma_admissible` | HSL saturation % | OKLCH **chroma ceiling per area class** — fewer knobs, perceptually uniform |
| lightness-gap `40` / `20` | `theme_contrast_ok` | non-WCAG proxy | **dissolves** into Class 1b WCAG compute |

**Principle:** L1 rows name the house choice (`GlareEnvelope`, `ChromaByArea`) with a `provenance: HouseChoice { signed: §4k row }` marker; L2 enforcement calls L0 perceptual ops on OKLCH coordinates derived from `Material`, not raw HSL compares.

**Timing scale exception:** `respond_fast` / `reveal_deliberate` / `restore_gentle` / `reframe_page` stay as **named L2 tokens** (discrete duration/easing rows). A base-time × ratio generator is **purity-trap territory** (§6) until a second consumer needs parametric timing — do not build it in Phase A.

**Phase:** A.2 (after or in parallel with WCAG rows; must complete before Phase B archetypes).

### Class 3 — String-as-identity keys → typed refs (construction)

The substrate's own lesson applies: a load-bearing string literal is an edge living in prose ([module identity vs storage](module-identity-storage-binding-design.md)) — invisible to totality checks, typo-prone, fail-open on coverage.

| Site (today) | String key | Defect | Target |
|---|---|---|---|
| `interaction.dag` `row_covers` | `element: String` on `BehaviorRow` / `InteractiveElement` | typo silently misses coverage | closed `ElementId` enum (or `SymbolRef`) — one variant per archetype element; `coverage_gaps` **total by construction** |
| `mark.dag` edges | `from_key` / `to_key: String` | severed fork can compile | `MarkNodeId` enum aligned with `mark_nodes` rows |
| `constellation.dag` edges | `from_key` / `to_key: String` | same | `StarId` enum aligned with `layer_stars` (L5 facts supply rows; L4 generator is parameterized over typed ids) |

**Phase:** A.0 — **rides the structural lift**. No separate schedule; untyped keys are inadmissible in `gunbc.design.*` from the first commit.

### Class → layer → phase summary

```text
Class 1 (cite + derive)  → L0 extdeps     → A.1 (css/svg) · A.2 (WCAG) · A.3 (stellar)
Class 2 (house + OKLCH)  → L1 walls       → A.2
Class 3 (typed keys)     → L2/L4 machinery → A.0 (lift)
```

Every item names a **dissolution trigger**: hand hue-range removed when A.3 greens; lightness-gap removed when A.2 greens; `join()` paint removed when A.1 greens; string element keys removed at A.0 lift.

## Dissolution trigger (DESIGN §6)

This document dissolves when Phase C is complete **or** when the library carriers subsume all sections and this md re-registers as a `gunbc.plan.Plan` row. Until Phase A lands, the mark on the carrier is this file plus the #6927 `gunbc.site.*` modules.
