# gunb.ai site subsumption — hosting + frontend generated from `.dag`, and the calm-intricate redesign

Status: **scoping proposal** (no code lands from this doc). Two coupled asks, one lane:

1. **Subsume `gunb-ai/frontend`** — the public site's hosting and its HTML/CSS/JS become artifacts *emitted by gunbc from `.dag`*, committed and drift-gated in this repo, published to GitHub Pages at `gunb.ai`. The `frontend` repo archives.
2. **Redesign** — replace the current austere register with a **futuristic, intricately-detailed, fractal** one, *without* abandoning the structural discipline that the frontend repo's `DESIGN.md` codified. Operator brief, refined three times on 2026-07-20: first "NYC high-rises with stone gargoyles"; then *components slightly more complex at deeper levels* (a video-game LOD discipline); then the register correction — **not gothic/moody/edgy: intricate, detailed, and calm**, with the glow read as "glowfish" (a comforting, soft green bioluminescence), never "steampunk video game." The gargoyle *idea* — functional ornament at boundaries — survives; the creature does not.

The two asks reinforce each other: intricate ornament is unaffordable in a hand-maintained site (§1 complexity — every carved detail is a maintenance liability) and nearly free in a generated one. **Derived ornament is the pitch**: the intricacy is computed from real repo facts, so the site's density is itself a demo of the compiler. The emblem is *functional* ornament — a leaf's veins, a river's delta: transport structure that reads as beauty — which preserves the frontend `DESIGN.md` thesis ("every visible element is a fact, relation, boundary, or receipt") while overturning its minimalism. The economics run the same direction: now that a clean minimal landing page costs one prompt, minimalism has stopped signaling care — the scarce signal is **unfakeable detail**, and derived detail is unfakeable in the strong sense (you cannot render it without actually having the structure; a banknote's guilloché is beautiful because it encodes anti-counterfeit function).

---

## 1. Current state (surveyed 2026-07-20)

### 1a. `gunb-ai/frontend` (the repo being subsumed)

- **Hosting:** GitHub Pages, `build_type: workflow` (`.github/workflows/pages.yml` stages `index.html` + `assets/` + `CNAME` into `_site` and deploys — publish-only, no build step). Custom domain `gunb.ai` live, HTTPS enforced, cert approved through 2026-08-30 covering `gunb.ai` + `www.gunb.ai`. DNS: apex A → the four Pages IPs (185.199.108–111.153), `www` CNAME → `gunb-ai.github.io`. DNS does **not** change in this plan.
- **Site content:** `index.html` (81 lines: masthead + hero + install command + card deck mount + footer), `assets/site.css` (680 lines; `:root` tokens with hue/sat/lightness *derivation* — `--moss`/`--clay`/`--brass` computed from `--hue-*` bases), `assets/cards.js` (1,029 lines, hand-written IIFE, no build step: a story-deck renderer — 3 cards over one `users.dag` example, each card = code-panel spec + graph spec + receipt rows, plus layout math, SVG generation, prev/next nav), `assets/mark.svg` + favicon/touch-icon PNGs, `CNAME`.
- **Governance:** `DESIGN.md` (10 hard rules + materials palette + panel/type/SVG vocabularies + override protocol), `PROPOSAL.md` (V1 spec), `legacy/` (prior site + `moodboard.html`, private reference). `docs/DEV-PREVIEW-ARCHITECTURE.md` documents a Mac-mini/tailnet preview mirror — superseded here by gunbc's own srv1 deploy path.

### 1b. gunbc machinery already in place (what we reuse, all proven by execution)

- **HTML emission:** `MarkupNode` → `serialize_html_source` / `try_serialize_html_source` (URL-refusal fail-closed), proven by the roadmap dashboard pages (`dag/gunbc/roadmap_page.dag`, CI-deployed to srv1) and gunbhub (`dag/gunbc/gunbhub_serve.dag`), with XSS-refusal witnesses (`html_markup_xss_witness_test.dag`).
- **CSS:** `dag/gunbc/roadmap_style.dag` `css_rule(selector, props)` — an honestly-marked scaffold (`css_rule_props_scaffold`, `props` is an anemic `String`). This lane is its declared consumer pressure (§2b).
- **Browser/server JS:** hand-built `TsProgram` AST → `serialize_typescript_program` (`dag/gunbc/node_http_server_emit.dag`), proven end-to-end — the emitted `server.js` runs on srv1 today. There is **no** `.dag → JS` compile path: `src/v2/extdeps/languages/ecmascript.dag` is an orphan TargetModel (0 consumers) and the TypeScript target is red at bar (c) per `docs/plans/typescript-gap-census.md`. This is the one genuine gap, and it is **phased, not blocking** (§3, Stage-J frontier).
- **Generated-artifact + drift gate:** `dag/gunbc/generated_artifact.dag` / `generated_artifact_emit.dag` / `dag/tools/generated_artifact_gate.dag` — byte-equality gate + per-artifact `extra_valid` + red-receipt perturb, already governing `.github/workflows/ci.yml`, `.githooks/*`, `ROADMAP.md`, `DESIGN.md`, plan docs, stage0 crate layout. The site files become new rows on this exact spine.
- **GHA workflow as a model:** `dag/gunbc/ci_workflow.dag` → `project_workflow_to_yaml` → `serialize_yaml` (`ci_yaml_emit.dag`) with a parse-back `extra_valid`. A Pages deploy workflow is one more `Workflow` value on this authority — not a new mechanism.
- **Hosting/deploy:** `dag/gunbc/live_deploy/` serves the roadmap dashboard on srv1 via `tailscale serve` with digest read-back receipts; `dag/gunbc/roadmap_static_site.dag` already models `SiteArtifact` (kind · url_path · body · digest), `WebServiceShape`, `DomainRoute`, `StaticSiteAllocation` — and `SiteArtifactKind.HtmlIndex` is a never-constructed scaffold waiting for exactly this consumer.
- **Roadmap alignment:** `2-stateless-frontend` (operator directive 2026-07-01) — "point a domain at an IP and serve the process as minimally as possible… green only when an independent read proves domain → endpoint → process → digest." This lane **is** that milestone's product-shaped consumer: GitHub Pages is a second *realization* of the same `StaticSiteAllocation` spec the srv1 node-serve path realizes (§2 Realization — one spec, N hosting handlers), and the receipt shape transfers verbatim (served digest == artifact digest, read back from `https://gunb.ai/`).

### 1c. The two-authority violation this plan removes

Today the product story is forked (§3): the *site* (facts about gunbc, its palette, its example programs) lives hand-written in a second repo, while gunbc emits a *different* HTML surface (roadmap dashboard) from models. Every fact on the site — the install command, the `.dag` code samples, the diagnostic shown in the invariant panel, even the palette — is a claim about this repo maintained by hand in that one, i.e. classic drift surface. Subsumption is not a hosting convenience; it is de-forking the product's public claims onto their single authority.

---

## 2. Target shape

### 2a. Ownership and layout

- New module home: `dag/gunbc/site/` (peer of `roadmap_*`): `site_tokens.dag` (materials palette + craft-floor scales), `site_primitives.dag` (panel/type/SVG vocabulary), `site_mark.dag` (the mark spec + its LOD renders, §4e), `site_content.dag` (copy, install command, deck card facts), `site_page.dag` (index composition), `site_ornament.dag` (derived-ornament generators, §4), `site_emit.dag` (artifact bodies), `site_workflow.dag` (Pages deploy `Workflow`).
- Committed artifacts (all `GeneratedArtifact` rows, drift-gated): `site/index.html`, `site/assets/site.css`, `site/assets/nav.js` (Stage-N residue, §3), `site/assets/mark.svg`, `site/CNAME`, `site/primitives.html` (the design system's witness page — deliberately committed and published at a non-linked path, since it is literally the unit test of the visual language), and `.github/workflows/pages.yml`.
- Binary icons (favicons, touch/maskable PNGs): **seed-retained asset rows** — committed bytes pinned by a digest row with a declared derivation trigger ("derive from `mark.svg` when a raster-emit handler exists"), never silently hand-editable (the digest row makes drift loud). Do not build a PNG rasterizer for this.
- `gunb-ai/frontend` archives after cutover (history preserved; `legacy/` moodboard stays reachable there). No mirror, no push-sync — a gunbc-pushes-to-frontend variant was considered and rejected as keeping two authorities alive (§3).

### 2b. CSS: tokens by construction

Port the frontend's token system as **typed facts, not strings**: `Material { name, hue, sat, lightness }` rows + derivation fns (`dim`, `subtle`, `up`) so every color in the emitted `site.css` is *derived from a material* — frontend `DESIGN.md` rule 4 ("no new color without deriving it from the material palette") stops being a review rule and becomes **unwritable by construction** (§5). This is also the declared consumer that begins dissolving the `css_rule` props-as-`String` scaffold: introduce the minimal typed property surface the site actually uses (tokens, lengths, font stacks) and keep raw-string props only as a counted, declared residue. Full CSS grammar modeling is explicitly out of scope (§6 purity trap).

### 2c. Publishing: Pages as a second hosting realization

- `.github/workflows/pages.yml` = one more `Workflow` value emitted through the existing `serialize_yaml` path, with the same drift + parse gate as `ci.yml`. Shape mirrors the frontend's proven workflow: stage `site/` → `upload-pages-artifact` → `deploy-pages`, on push to main. No build step in the workflow — CI's generated-artifact gate already guarantees the committed bytes equal the model's emission, so the deploy is dumb-publish (the §5 wall is in CI, not in the deploy job).
- The gunbc repo is public and has **no Pages site today** (verified: `GET /repos/gunb-ai/gunbc/pages` → 404), so the slot is free; one site per repo is not a constraint.
- **Receipt:** extend the srv1 digest read-back pattern (`roadmap_site_readback_curl_digest_check`) to the public surface: post-deploy, an independent read of `https://gunb.ai/` (and `/assets/site.css`) must hash to the committed artifacts' digests — the roadmap item's "domain → endpoint → process → digest" receipt, with Pages-CDN as the process. Read-back failure is `NotConverged`, never success. This can run as a scheduled falsifier-style check as well as post-deploy, since Pages deploys are eventually consistent (bounded poll via the existing `BoundedPoll` emit, never an unbounded retry).

### 2d. Domain cutover (manual, ordered, reversible)

DNS never changes (apex already points at Pages IPs; `www` CNAME already targets `gunb-ai.github.io`). The cutover is entirely GitHub-side Pages config:

1. Enable Pages (workflow build) on `gunb-ai/gunbc`; merge the generated `pages.yml`; verify the site at `gunb-ai.github.io/gunbc`-equivalent Pages URL **before** touching the domain.
2. **Verify `gunb.ai` as an org-level verified domain** first (Settings → Pages → verified domains) — this closes the takeover window while the custom domain is detached from one repo and not yet attached to the other.
3. Remove custom domain from `gunb-ai/frontend` → add `gunb.ai` (+ `www`) to `gunb-ai/gunbc` Pages → cert re-issues (minutes; brief HTTPS gap acceptable, do it deliberately, verify with the §2c receipt).
4. Archive `gunb-ai/frontend`; its `pages.yml` is disabled by the archive.

Rollback at every step is the inverse config edit; nothing is destructive.

## 3. JavaScript: dissolve first, compile later (the typed asset frontier)

The instinct "generate the JS via `.dag`" has a sharper answer than compiling 1,029 lines of renderer: **most of `cards.js` should not exist at runtime.** It is a *renderer over static card specs* — code-panel lines, graph nodes/edges, roles, receipt rows — i.e. data plus layout math, all knowable at emit time. gunbc's whole thesis is to run that fold at compile time. So, mirroring the self-host frontier pattern (each module `SelfEmitted | SeedRetained{reason, trigger}`), every site asset gets a typed disposition `DagEmitted | SeedRetained{reason, migration_trigger}` and the JS splits in three:

- **Stage-D (deck → emit-time):** model the deck as `.dag` facts (`StoryCard { code_panel, graph, receipt, pivot }` etc.) and render each card to **static HTML+SVG at emit time** — the layout math (`columnXs`, elbow routing, label widths) becomes ordinary `.dag` fns, executed once in CI instead of on every visitor's CPU. This deletes ~90% of `cards.js` and is *better* than compiling it: zero client JS to keep correct, and the card content joins the drift-gated single authority. Bonus wall (§5): the `.dag` code shown in the cards and the diagnostic shown in the invariant panel become **witnessed samples** — CI compiles the displayed sample and asserts it produces exactly the displayed output/diagnostic. The marketing site cannot lie about the compiler; a sample that drifts reds the build.
- **Stage-N (nav residue):** the genuinely-runtime remainder (deck prev/next, ←/→ keys and spine waypoints, `aria` state) is ~50–150 lines. Author it as a hand-built `TsProgram` AST serialized by the **existing, proven** `serialize_typescript_program` path (same mechanism as the srv1 `server.js`) → `site/assets/nav.js`. No new compiler surface; the asset is `DagEmitted` from day one. The §4d depth-reveal interactions add **no** JS by construction: every depth ships in the emitted DOM and is revealed by CSS interaction states and `<details>` — never fetched, never computed client-side.
- **Stage-J (compile-path trigger):** re-author Stage-N behaviors as ordinary `.dag` fns emitted through the ECMAScript/TypeScript TargetModel **when** the TS target's bar (c) greens (`typescript-gap-census.md`); that event is this frontier row's named migration trigger. Explicitly **not** a dependency of shipping the site — sequencing the public site behind the TS self-host lane would be the purity trap (§6). Stage-J is also the natural first *external* consumer of the JS target, which gives that shelved lane its displaced-cost pricing when it wakes.

## 4. The redesign: calm intricacy, with derived ornament

### 4a. What carries over unchanged

The frontend `DESIGN.md`'s *structural* discipline survives intact — it is this repo's §5 written for pixels: the fact/relation/boundary/receipt test, HTML-owns-text/SVG-owns-structure, panels as the four content types, one mark everywhere, pages-compress-not-enumerate, the override protocol. These port into `dag/gunbc/site/` as typed vocabulary (panel types as a closed coproduct — "clay only in Invariant panels" becomes construction, not review).

### 4b. What changes: the register

The current site is *austere* — quiet but thin. The brief is **calm intricacy** (operator, 2026-07-20, correcting this section's earlier Deco-Gothic draft: not moody or edgy — a gargoyle is *on edge* by function, a vigilance creature, and vigilance is the wrong feeling): density that soothes rather than looms. The reference class is **structures known to be complex that read calm**: star atlases and constellations · bioluminescent depths (glowworm caves, plankton, "glowfish") · kumiko and girih latticework · leaf venation and river deltas · frost dendrites · banknote guilloché. Four of these are rendered as candidate register studies for the Phase-0 approve/reject pass. Concretely:

- **Composition:** generous dark negative space; intricacy concentrated in *bounded fields* (friezes, panel borders, the hero figure) that sit **in** calm rather than filling it; depth strata instead of skyline setbacks — the deeper the engagement (§4d), the denser and softer-lit the field.
- **Materials (the palette holds — operator: colors are OK):** `void`/`stone`/`warm-white` as before; moss/**verdigris** carries the relation role (it *is* the soft-green family); clay stays singular for boundary/diagnostic; brass stays rare, only the human-focus pivot. **Light** is a treatment, not a hue: **bioluminal** — dim, wide, soft blooms in the verdigris family over the void ground; comforting, never sharp. Banned by name: industrial/steampunk cues (gears, rivets, HUD chrome, amber warning glow) and neon saturation.
- **Linework:** one stroke weight was the old law; the new register uses a *closed scale* of engraved densities (hairline field / rule / heavy rule) — still a closed vocabulary, just deeper, exactly like the type scale — plus a **lit variant** per weight (core hairline + wide low-alpha halo strokes). Gradients are admitted **only as emitted light** (halo/bloom around lit geometry, never a surface wash — rule 10's gradient-blob ban stays intact), landed as a rule amendment through the override protocol, with a declared **luminance budget** (a max lit-to-matte ratio per viewport, a committed token row) so the register can never drift to neon.
- **Typography:** mono stays the native register for code/receipt/diagnostic. The wordmark and section heads get a **quiet engraved treatment** (letterspaced caps in the star-atlas register — precision engraving, not gothic display) — this still amends the old "no display font" rule through the documented override protocol, justified structurally (the wordmark is a boundary marker, not body voice).
- **Boundary marker:** the gargoyle's *job* survives, calmly — a boundary is where **the light stops**: a dark, unlit gap in an otherwise-lit structure (in the constellation register, a **dark nebula** — a real, known-complex object whose entire identity is *structure visible as absence of light*). Uniqueness stays derived: the gap's silhouette is generated from the diagnostic's content hash, so no two refusals are drawn alike. Rule 1 intact: the ornament *is* the boundary annotation, with none of the menace.

### 4c. Derived ornament (the load-bearing idea)

Intricacy is admitted **only if computed from a real repo fact** — the design-system analogue of §4 grounding, and the reason a generated site can afford what a hand site cannot:

- **Frieze bands** between sections: a repeating relief pattern generated from the actual module dependency graph (nodes/edges of `dag/std` → geometric tracery), regenerated by the same emit that builds the page — the ornament drifts only when the truth does.
- **Coffered fields / borders:** panel border micro-patterns derived from the content hash of the panel's own facts (a visual `ContentHash` — two panels with identical content carry identical engraving).
- **The hero figure:** the layer DAG (`std ← extdeps ← compiler ← workflow`) drawn as an **asterism** — a small constellation whose brightest star is the authority node — so the site's most prominent ornament literally *is* the architecture diagram.
- **Light is derivation made visible (the glow's rule-1 mapping):** a lit edge is a relation the compiler actually walks; a lit node is a checked fact; a receipt renders as the lit ledger row. Refusal does **not** glow — a boundary (clay) *breaks* the light: a cold, unlit gap in an otherwise-lit path. So the register reads as a closed semantic: **matte = declared · light = derived · dark gap = refused** — the epistemic chain drawn literally, which is what licenses "the site framed as a glowing DAG" (operator, 2026-07-20) as fact rather than mood. The glow itself is bioluminal — dim, wide, comforting (operator: "glowfish," never "steampunk video game").
- Rule 6 ("no animation unless it clarifies a structural transition") survives with one amendment candidate: light-catching on engraved lines during scroll (a static-geometry highlight, not motion of elements) may qualify as clarifying depth; decided at the moodboard phase, through the override protocol, not silently.

Every generator lands with the same honesty as any emitter: deterministic (same facts → same bytes; the drift gate enforces this for free), and refused-not-faked when a fact it needs is unavailable.

### 4d. The fractal depth grammar (LOD — the axis the old system lacked)

The frontend `DESIGN.md` already declares fractal repetition **across scale** (fact block → panel → section → page → site). The operator's refinement adds the orthogonal axis it lacked: **depth of engagement** — the same grammar repeats as attention closes in, so looking closer always resolves more *structure*, never more decoration. This is the video-game level-of-detail discipline (the quality that makes Discord feel "like a game," decomposed: (a) a world with consistent materials, (b) exploration rewarded, (c) juice/feedback — we import (a) and (b) whole and ration (c) to depth transitions so the calm register survives). The depth ladder, fixed site-wide:

```text
D0  glance   (~3s)      silhouette — one claim, the skyline
D1  scan     (~30s)     facades — sections and panels
D2  read     (~3min)    engravings — code, labels, receipt rows
D3  approach (interact) hover/expand — names, types, hashes; the fine structure up close
D4  inspect  (source)   the maker's-mark layer — emission receipt in the document itself
```

Two construction rules make depth cheap and honest (§2/§3/§5):

- **One authority per component, all depths derived.** `render(fact, depth)` is one fold with a depth budget — a component's D0 and D3 are projections of the *same* fact rows, so levels cannot disagree. (The failure this kills is exactly why hand-built sites cannot afford density: the tooltip drifts from the diagram it annotates, and every added level multiplies the drift surface. Here N levels cost one generator.)
- **All depths ship in the emitted DOM; interaction only reveals.** Hover/expand are CSS states and `<details>` elements — never a fetch, never client-side computation. D0–D2 are fully static; D3 has keyboard parity (`:focus-visible` twins; `<details>` is keyboard-native); JS stays the Stage-N residue. D4 is deliberate, not an easter egg: the audience is compiler engineers, for whom view-source is a second front door — the document leads with a composed comment carrying the emission receipt and a verify command.

### 4e. The mark — one graph, every scale (logo rethink, operator 2026-07-20)

The current mark (a rounded-rect creature face) is in the mascot register: it carries no structure, its geometry is not grid-derived, and at 16 px it reads as a blob. **Replace, not refine.** The replacement is a **system, not a drawing**: the mark is a tiny graph *spec* in `site_mark.dag` plus the same `render(fact, depth)` fold as every component (§4d) — which makes rule 7 ("one mark everywhere") literal: favicon = the D0 render, nav = D1, hero = D3. The levels cannot disagree because they are one authority; the logo is the first citizen of the fractal grammar, not an exception to it.

Candidates for the moodboard (each rendered from a spec, judged at 16/24/48/120 px and hero scale):

- **A — Confluence (recommended primary).** The smallest honest DAG: four nodes, fork and re-join (`a → b · a → c · b → d · c → d`). It is the product's whole argument in four nodes — one fact forks into two copies (drift) and must re-join (single authority); the re-join is what gunbc sells. It is also the **diamond property** (confluence) from rewriting theory — derivations agree — so the mark is a theorem, not a doodle. At 16 px: four lit points and four hairlines, reading as a small constellation.
- **B — Asterism.** The layer DAG (`std ← extdeps ← compiler ← workflow`) as a small constellation, the authority layer its brightest star — the hero-scale composition (A's diamond is its inner figure, so A and B remain one geometry family, not two marks).
- **C — Keystone.** A single lit node at the apex of an arch of edges — the single authority that holds the structure (remove the keystone and the arch falls). Strong meaning, weaker small-size survival; candidate for section caps rather than primary.
- **D — Delta.** Fork-and-re-join as a river draws it: a distributary braid narrowing to one mouth (nature's confluence DAG; leaf venation is the same figure at another scale). The organic sibling of A — the same four-node truth rendered as flow instead of geometry.

(The former gargoyle-constellation candidate is **dropped** with the register correction — a vigilance creature is on-edge by design, the exact feeling being removed. Boundary surfaces get the §4b dark-gap/nebula treatment instead.)

Treatment: node points luminous (§4b light), edges hairline; favicon = dark tile + soft green points. Any generated concept imagery is **direction only** — the landed mark is constructed geometry (all coordinates on one grid, strokes from the closed scale), emitted to `site/assets/mark.svg` and drift-gated like every other artifact.

### 4f. Component vocabulary (first cut, for the moodboard)

Eight components; each must exhibit its **full depth ladder** on `primitives.html` before production use (the frontend repo's primitives-before-pages law, now enforced per-depth). All are compositions of the four panel types plus the §4c ornament generators.

| Component | Building role | D0 → D3 |
|---|---|---|
| **Constellation** | hero / masthead | wordmark under the layer-DAG asterism → stars name the layers (`std ← extdeps ← compiler ← workflow`) → hover: per-layer module counts → expand: module lists with real content hashes |
| **Cornerstone** | identity / install | the install command, engraved → version · commit · date beneath → build-receipt digest → a copyable `verify` command that recomputes it |
| **Frieze** | section separator | tracery band computed from the module graph of the section's subject → hover names the modules and edges → click-through to the real source |
| **Specimen** | fact / code panel | a witnessed `.dag` sample beside its graph, both rendered from the same fact rows → hover a symbol: its type appears and its node/edges highlight in *both* surfaces at once |
| **Nebula** | boundary marker | a dark, unlit gap breaking the lit structure at Invariant panels, the 404, refusal states — its silhouette generated from the content hash of the diagnostic it marks, so **no two are identical** → hover: what stopped here ("non-exhaustive `match` — the light ends at this edge") → expand: the full refusal receipt |
| **Ledger** | receipt panel | aligned proof rows → digests are real → per-row copyable re-verification command |
| **Spine** | nav / scroll waypoints | the page's **own section graph rendered as a small lit DAG** — the current waypoint's node glows, visited ones stay faintly lit; ↑/↓ keyboard — the one component where game-feel is explicit (the site navigates itself as what it is: a DAG) |
| **Hallmark** | footer + D4 | maker's mark — "emitted from `dag/gunbc/site` @ hash · gunbc version" — rendered in the footer *and* as the document's leading comment |

The nebula family keeps the procedural-uniqueness move the game register does best — uniqueness is *derived* (diagnostic hash → silhouette parameters), so delight and honesty coincide: a one-of-a-kind figure that cannot exist without a real diagnostic behind it — now rendered as calm absence (the light stopping) rather than a creature.

### 4g. Page flow (one page; depth is the z-axis, not page count)

The stations are register-agnostic (the surface metaphor binds at the Phase-0 register pick — a descent into lit depths, a night-sky survey, a lattice unfolding); the site gets **deeper, never longer**:

```text
arrival     D0 silhouette: the hero asterism + one sentence
argument    drift → duplication → one structural description
            (the card deck reframed as three quiet panels of one system)
strata      one claim per stratum: CHECK · EMIT · FAIL-CLOSED —
            each stratum = specimen + ledger (+ a nebula where a boundary exists)
records     the receipts: build/CI/roadmap digests, re-verification commands
horizon     where it is headed: self-hosting; language design opened up
close       cornerstone repeated · hallmark
```

Interaction grammar, closed: **hover = approach · click/focus = enter · Esc = step back**. Every hover reveal has a focus twin; motion is permitted only as a depth transition (the rule-6 amendment candidate: detail resolving on approach clarifies *containment* — a structural transition, not decoration).

### 4h. Two audiences, one authority

A growing share of visits are agents, not humans. The same `site_content.dag` facts emit an **agent surface** (`llms.txt` + a JSON facts file with digests) beside the human page — one model, two realizations (§2), landed as ordinary `SiteArtifact` rows. The human gets the engraving; the agent gets the ledger. No claim ever has a second copy.

### 4i. The craft floor (the "amateurish" diagnosis, made unwritable)

Operator verdict on the current site (2026-07-20): cheap/amateurish-looking. The bones are sound — the token derivation and panel discipline are professional structure — so the verdict localizes to a specific, enumerable set of **craft tells**, and every one of them is a *convention-vs-construction* failure (§5): a hand author must remember each rule on every edit; a generator makes the rule a wall. The floor:

| Amateur tell | Construction rule (the wall) |
|---|---|
| spacing that is "near" things instead of on a rhythm | one modular spacing scale; every margin/padding is a scale row — off-scale lengths unwritable |
| flat hierarchy — everything medium-sized, medium-weight (the fear of extremes) | a steep type scale with few steps; each type role pinned to exactly one step |
| harsh defaults — pure white on pure black, default link blue, default focus rings | every color pair drawn from the `Material` table; contrast ratios computed at emit (craft and a11y are the same check) |
| mono set at sans size (mono runs optically large) | per-family optical-size ratios as token rows |
| all-caps without tracking; faked small caps | letterspacing bound to the case treatment inside the type-role row |
| mixed corner radii and stroke weights | one radius authority; strokes only from the closed §4b scale |
| muddy borders — neither visible nor invisible | border alphas as derived steps of the material, never hand-picked hexes |
| unbalanced vector geometry in the mark | §4e — the mark is a grid spec, not a drawing |
| glow used as decoration (the neon slide) | §4b luminance budget + §4c light-semantics: glow only on derivation geometry |

The point of the table: the operator does not need to become a designer. One good sign-off at the moodboard gate, and the floor holds it everywhere, forever, by construction — **the professional look *is* the enforcement.** (This is also the §1 argument turned on pixels: hand-tuning every page is the amateur *process* regardless of the author's taste; a crafted result at maintained cost requires the rules to live in one place.)

### 4j. Process (unchanged from the frontend repo's own law)

Primitives before pages: the register lands first as a **moodboard/workpad** (authored in `dag/gunbc/site/`, emitted to a non-linked path), then `primitives.html` (every material, type role, panel, SVG primitive, ornament generator — each shown at every depth of its ladder — the design system's witness page), then the production `index.html` composed only of primitives that survived there. Operator reviews the register at the moodboard gate before any production page is built — the aesthetic call stays a human sign-off; the plan only guarantees that whatever is signed becomes construction, not convention.

## 5. Phases (each independently landable; triggers named)

- **Phase 0 — model + moodboard.** `dag/gunbc/site/` module home; `Material` tokens + derivation fns + the §4i craft-floor scales (spacing · type · stroke · radius · contrast · luminance budget, as typed rows); panel/type/SVG/ornament typed vocabulary; the §4f component rows with their depth ladders (`render(fact, depth)` fold shape); the §4e mark candidates (A–D) rendered from specs at 16/24/48/120 px; first ornament generators (frieze-from-module-graph, hash-coffering, hash-nebula); emit `moodboard.html` + `primitives.html` (every primitive at every depth D0–D3; D4 = its emitted source form) as committed, drift-gated artifacts (published at non-linked paths once Phase 2 lands; reviewable on the srv1/tailnet path before that). **Gate: operator approve/reject over small batches of register-candidate studies (glowworm grotto · celestial atlas · kumiko lattice · delta/venation) + mark selection — the operator is a hard approver by their own account, so the gate is built to loop cheaply; nothing proceeds on an unapproved register.** No public-site change.
- **Phase 1 — the page.** `site_content.dag` (copy, install command, deck card facts with witnessed samples per §3 Stage-D); static card rendering at emit time; all depths shipped in the DOM with CSS-state reveals + focus parity; Stage-N `nav.js` via `TsProgram`; emit `site/index.html` + `site/assets/site.css` + `mark.svg` + `CNAME` + the §4h agent surface; icons as digest-pinned seed-retained rows; a declared page-weight budget row (the all-depths DOM is bounded by a committed byte budget, drift-visible, not a vibe); all on the generated-artifact gate. Site is fully reviewable at the Pages preview URL and on srv1. **Gate: DESIGN-rules lens/review pass + witnessed-sample witnesses green.**
- **Phase 2 — hosting subsumption.** `site_workflow.dag` → generated `pages.yml` (drift + parse gate); enable Pages on gunbc; §2c read-back receipt wired (post-deploy + scheduled); §2d cutover runbook executed (org domain verification → detach/attach → receipt green at `https://gunb.ai/`); archive `gunb-ai/frontend`. **This discharges the `2-stateless-frontend` milestone-A receipt shape on the public domain** (the srv1 path stays as the tailnet staging realization of the same allocation spec).
- **Phase 3 — dissolutions (each on its own trigger).** Stage-J: nav behaviors re-authored as `.dag` fns through the JS/TS TargetModel when bar (c) greens. `css_rule` scaffold: dissolve remaining raw-string props into the typed surface as the site's usage covers them (counted residue, never a blanket rewrite). Icon rasterization: derive PNGs from `mark.svg` if/when a raster handler exists for another consumer. `roadmap_static_site.HtmlIndex`: constructed or deleted once the site artifacts land (it stops being a scaffold either way).

## 6. Risks / open questions

- **Fonts:** the current site loads JetBrains Mono + Noto Sans from Google Fonts (an external runtime dependency and a privacy consideration). Recommend self-hosting under `site/assets/fonts/` as digest-pinned seed-retained rows. Decide in Phase 1.
- **Pages eventual consistency:** the post-deploy receipt must bounded-poll (existing `BoundedPoll` emit), and a CDN-cached stale read must be distinguishable from a wrong deploy (compare against *previous* digest → `StaleServe`, typed, not a generic mismatch).
- **Cert gap at cutover:** minutes-scale HTTPS interruption while the cert re-issues on the new repo. Accepted; step 2's org-level domain verification closes the security half of the window.
- **workpad/legacy content:** `workpad.html` (125 KB of design exploration) and `legacy/` do not migrate — they remain in the archived repo's history. Only `legacy/moodboard.html`'s *palette reasons* port (as `Material` row commentary).
- **Repo size/na noise:** the site adds committed HTML/CSS artifacts to a compiler repo. Contained under `site/` with `linguist-generated` attributes; the drift gate already governs bigger artifacts (`ROADMAP.md`).
- **Accessibility of depth:** hover-only reveals are invisible to keyboard and touch users — every D3 reveal needs a `:focus-visible` twin or a `<details>` carrier (keyboard-native), and `prefers-reduced-motion` collapses depth transitions to instant. This is a Phase-1 gate criterion, not a polish item.
- **Page weight:** shipping all depths in the DOM grows `index.html`; governed by the Phase-1 byte-budget row. If a component's full D3 payload breaks the budget (e.g. per-module hash lists), its D3 truncates with a typed "continued at GitHub" boundary — never a client-side fetch.
- **Machine-room freshness:** receipts are **as-of-emit** (committed digests, "as of build \<hash\>") — honest and static. Live telemetry would need a service behind Pages; out of scope, noted as a possible later srv1-backed enhancement on the same fact rows.
- **Sound:** default **no** — one register break and the stone goes plastic. Recorded as a moodboard-gate decision so it is decided, not drifted into.
- **Design taste is not automatable:** the plan makes the *signed* register enforceable; it cannot generate the sign-off. Phase 0's moodboard gate is deliberately a human decision point built for iteration (small candidate batches, approve/reject per item — the first full batch was rejected on register grounds and corrected the same day, which is the gate working). §4's calm-intricacy direction is a proposal to react to, not a settled spec.

## Dissolution trigger (DESIGN §6)

This document dissolves when Phase 2's cutover receipt is green at `https://gunb.ai/`: the site vocabulary's authority is then its carriers under `dag/gunbc/site/`, the hosting authority is the generated `pages.yml` + its read-back receipt, and this md re-registers as a `gunbc.plan.Plan` row (or deletes) per the standing plan-artifact convention.
