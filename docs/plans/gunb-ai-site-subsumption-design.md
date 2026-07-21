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
- **Stage-N (behavior residue):** the genuinely-runtime remainder — deck prev/next, keyboard traversal, spine waypoints, clipboard receipts, counterpart-highlight where CSS `:has()` cannot reach, `aria` state — grew with the fifth-pass behavioral re-aim (§4l) and is modeled as **typed behavior rows** (element × verb × response × timing token) realized twice: CSS states where no state machine is needed, `TsProgram`-emitted JS where one is (~200–400 lines, the same proven `serialize_typescript_program` path as the srv1 `server.js`) → `site/assets/nav.js`. No new compiler surface; the asset is `DagEmitted` from day one. The no-fetch law stands: every depth ships in the emitted DOM; behaviors respond to input and reveal shipped truth — never fetch, never compute content client-side.
- **Stage-J (compile-path trigger):** re-author Stage-N behaviors as ordinary `.dag` fns emitted through the ECMAScript/TypeScript TargetModel **when** the TS target's bar (c) greens (`typescript-gap-census.md`); that event is this frontier row's named migration trigger. Explicitly **not** a dependency of shipping the site — sequencing the public site behind the TS self-host lane would be the purity trap (§6). Stage-J is also the natural first *external* consumer of the JS target, which gives that shelved lane its displaced-cost pricing when it wakes.

## 4. The redesign: calm intricacy, with derived ornament

### 4a. What carries over unchanged

The frontend `DESIGN.md`'s *structural* discipline survives intact — it is this repo's §5 written for pixels: the fact/relation/boundary/receipt test, HTML-owns-text/SVG-owns-structure, panels as the four content types, one mark everywhere, pages-compress-not-enumerate, the override protocol. These port into `dag/gunbc/site/` as typed vocabulary (panel types as a closed coproduct — "clay only in Invariant panels" becomes construction, not review).

### 4b. What changes: the register

The current site is *austere* — quiet but thin. The brief is **calm intricacy** (operator, 2026-07-20, correcting this section's earlier Deco-Gothic draft: not moody or edgy — a gargoyle is *on edge* by function, a vigilance creature, and vigilance is the wrong feeling): density that soothes rather than looms. The reference class is **structures known to be complex that read calm**: star atlases and constellations · bioluminescent depths (glowworm caves, plankton, "glowfish") · kumiko and girih latticework · leaf venation and river deltas · frost dendrites · banknote guilloché. Four of these were rendered as candidate register studies; the first approve/reject pass (third operator brief, same day) selected **stars/constellations as the leading register**, singled out the **blue/saturated** light as the liked element, and **rejected the text treatment** — outcomes carried as §4k checklist rows. Concretely:

- **Composition:** generous dark negative space; intricacy concentrated in *bounded fields* (friezes, panel borders, the hero figure) that sit **in** calm rather than filling it; depth strata instead of skyline setbacks — the deeper the engagement (§4d), the denser and softer-lit the field.
- **Materials (amended again, seventh pass 2026-07-20 — the rendered cyan REJECTED):** the first study pass singled out the **blue/saturated** star-field register; the first *rendered* cyan (`hsl(194 88% 76%)`) then read as "light/gamer blue — very low quality/taste" (operator). The corrected diagnosis is a modelable law, the **glare envelope**: the gamer tell is *saturation and brightness together* — a material may be bright OR chromatic, never both. So the field stays **deep indigo-black** (the blue the operator liked was the *sky*, not the accent), and the light family becomes **warm starlight** (low-sat warm white-gold — what stars actually are), with `quiet_envelope` refusing sat>45 ∧ light>68 by construction (`gunbc.site.tokens`). Text stays **warm stone**, **brass** the rare focus accent, **clay** singular for boundary. Chroma stays bounded **by area**; banned by name: industrial/steampunk cues and saturated fields — and now bright-saturated *anything*.
- **Linework:** one stroke weight was the old law; the new register uses a *closed scale* of engraved densities (hairline field / rule / heavy rule) — still a closed vocabulary, just deeper, exactly like the type scale — plus a **lit variant** per weight (core hairline + wide low-alpha halo strokes). Gradients are admitted **only as emitted light** (halo/bloom around lit geometry, never a surface wash — rule 10's gradient-blob ban stays intact), landed as a rule amendment through the override protocol, with a declared **luminance budget** (a max lit-to-matte ratio per viewport, a committed token row) so the register can never drift to neon.
- **Typography (engraved-display amendment WITHDRAWN — operator, third pass: "i don't like the text so far"):** the letterspaced engraved-caps treatment is dropped; the frontend `DESIGN.md` rule 3 ("no display font") stands un-amended. The wordmark returns to a quiet lowercase sans; body stays the quiet sans; mono only for code/receipt/diagnostic at its optical size (§4i). Type does zero theatrical work — the constellation does the theater.
- **Boundary marker:** the gargoyle's *job* survives, calmly — a boundary is where **the light stops**: a dark, unlit gap in an otherwise-lit structure (in the constellation register, a **dark nebula** — a real, known-complex object whose entire identity is *structure visible as absence of light*). Uniqueness stays derived: the gap's silhouette is generated from the diagnostic's content hash, so no two refusals are drawn alike. Rule 1 intact: the ornament *is* the boundary annotation, with none of the menace.

### 4c. Derived ornament (the load-bearing idea)

Intricacy is admitted **only if computed from a real repo fact** — the design-system analogue of §4 grounding, and the reason a generated site can afford what a hand site cannot:

- **Frieze bands** between sections: a repeating relief pattern generated from the actual module dependency graph (nodes/edges of `dag/std` → geometric tracery), regenerated by the same emit that builds the page — the ornament drifts only when the truth does. *(Parked per the fifth pass, §4l — optional, moodboard-decided; the intricacy budget re-aimed at behavior.)*
- **Coffered fields / borders:** panel border micro-patterns derived from the content hash of the panel's own facts (a visual `ContentHash` — two panels with identical content carry identical engraving). *(Parked per the fifth pass, §4l, same terms.)*
- **The hero figure:** the layer DAG (`std ← extdeps ← compiler ← workflow`) drawn as an **asterism** — a small constellation whose brightest star is the authority node — so the site's most prominent ornament literally *is* the architecture diagram.
- **Light is derivation made visible (the glow's rule-1 mapping):** a lit edge is a relation the compiler actually walks; a lit node is a checked fact; a receipt renders as the lit ledger row. Refusal does **not** glow — a boundary (clay) *breaks* the light: a cold, unlit gap in an otherwise-lit path. So the register reads as a closed semantic: **matte = declared · light = derived · dark gap = refused** — the epistemic chain drawn literally, which is what licenses "the site framed as a glowing DAG" (operator, 2026-07-20) as fact rather than mood. The glow itself is bioluminal — dim, wide, comforting (operator: "glowfish," never "steampunk video game").
- Rule 6 ("no animation unless it clarifies a structural transition") survives with one amendment candidate: light-catching on engraved lines during scroll (a static-geometry highlight, not motion of elements) may qualify as clarifying depth; decided at the moodboard phase, through the override protocol, not silently.

Every generator lands with the same honesty as any emitter: deterministic (same facts → same bytes; the drift gate enforces this for free), and refused-not-faked when a fact it needs is unavailable.

### 4d. The fractal depth grammar (LOD — the axis the old system lacked)

The frontend `DESIGN.md` already declares fractal repetition **across scale** (fact block → panel → section → page → site). The operator's refinement adds the orthogonal axis it lacked: **depth of engagement** — the same grammar repeats as attention closes in, so looking closer always resolves more *structure*, never more decoration. This is the video-game level-of-detail discipline (the quality that makes Discord feel "like a game," decomposed: (a) a world with consistent materials, (b) exploration rewarded, (c) rich feedback — an earlier draft rationed (c); the fifth operator pass corrected that: **feedback is the intricacy carrier**, tuned calm — still until touched, always answers when touched, §4l). The depth ladder, fixed site-wide:

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

Treatment: node points luminous (§4b light, family per §4k rows 3–5), edges hairline; favicon = dark tile + soft light points. Any generated concept imagery is **direction only** — the landed mark is constructed geometry (all coordinates on one grid, strokes from the closed scale), emitted to `site/assets/mark.svg` and drift-gated like every other artifact.

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

Interaction grammar, closed: **hover = approach · click/focus = enter · Esc = step back**. Every hover reveal has a focus twin; motion is permitted only as a *response* to a user action (the rule-6 amendment: a response clarifies interaction state — still until touched, always answers when touched). The full verb/response/timing vocabulary and the coverage law live in §4l.

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

### 4k. Register checklist — the sign-off instrument (v1, third operator pass, 2026-07-20)

The operator asked for the register as a *verbal checklist* to keep taste consistent across passes ("i'm starting to doubt myself"). The audit across all five briefs finds them **convergent, not contradictory** — "futuristic," "fractal," "calm," "glowfish," "simpler/safer," and "intricate-in-behavior" compose into one thesis because the LOD grammar (§4d) assigns them to different depths:

> **Quiet at arm's length; responsive up close; every response true.**

Simplicity is a D0/D1 property; intricacy is a D2/D3 property — and by the fifth pass, explicitly a property of **behavior more than of drawing** (§4l). §4c and §4l hold whatever is shown *or answered* to the same derivation bar. The proof-object updates from the banknote to the **instrument**: a fine instrument is plain to look at and exact to the touch. Exactly **three** things changed across the five passes, each recorded rather than absorbed: the palette pivot (warm → indigo/starlight, rows 3–5), the type treatment (engraved caps proposed, rejected, withdrawn — row 7), and the intricacy re-aim (ornament → behavior — rows 2/9/15, §4l). The signal has otherwise been stable.

The consistency test for any future proposal is two questions: **does it keep D0/D1 quiet?** and **is its D2/D3 intricacy — drawn or behavioral — derived from a real fact?** Yes + yes = consistent with every brief so far; anything that makes D0 louder is rejected regardless of how good it looks in isolation.

**Positioning vs the meta registers (fourth/fifth-pass comprehension aid):** at D0/D1 the page sits deliberately beside OpenAI/Claude — centered quiet column, calm type — as the **night inverse of Claude's paper** (indigo field, warm text). The deliberate opposites: Apple's scroll theater (row 9 forbids ambient motion) and template-SaaS enumeration (row 14 bans its element classes outright). Discord's real contribution, per the fifth pass, is not costume but **behavioral depth** — visually simple, feedback incredibly intricate — imported calm (§4l), with Linear as the standing proof that feedback-craft composes with quiet. The row no meta site can copy remains 13: their detail, drawn *or* behavioral, is authored per-surface; ours is generated from rows, so its consistency is structural.

Rows (approve / strike / amend per row; unstruck rows bind Phase 0):

| # | Axis | Proposed setting | Operator signal |
|---|---|---|---|
| 1 | macro layout | safe: one centered column, familiar scan order, no experimental grid | "simpler/safer" |
| 2 | intricacy placement | **behavioral, not ornamental** (fifth pass): the depth lives in feedback coverage + D3 reveals; visually only the hero constellation and the mark carry figure-detail; friezes/coffering parked as optional | "intricate ≠ necessarily visual — Discord is visually simple, its behavior/feedback incredibly intricate" |
| 3 | field | deep indigo-black (blue-leaning void) — the blue lives in the *sky*, not the accent | "like the blue in particular" |
| 4 | light family | **warm starlight** (low-sat warm white-gold; day theme: indigo ink) under the **glare envelope** — bright OR chromatic, never both (seventh pass: the rendered saturated cyan rejected as "gamer blue"); chroma still bounded by area. **Ninth pass — the accent is a stellar class** (`gunbc.site.starlight`, Morgan–Keenan grounding): candidates O/A (cool white) · G (sun-gold, live today) · K (amber), rendered side-by-side in the emitted accent study; **no green candidate can exist** (no green stars — physics, not taste); green enters as **verdigris patina material** on edges/hairlines only; M prunes itself (fails the envelope at lit brightness). Decision = pick a class row | "other colors besides blue? green at first… maybe something unique" |
| 5 | warm counterweight | text warm stone; brass rare focus accent; clay singular boundary | colors "in the realm of possibility" |
| 6 | glow | dim, wide, soft blooms; luminance-budget token row | "glowfish, not steampunk" |
| 7 | type | quiet lowercase sans wordmark; no display/engraved treatment (rule 3 restored); mono only for code/receipts at optical size | "don't like the text so far" |
| 8 | atlas machinery | no graticule/tick fields by default; the figure carries the detail; labels appear only at D3 | "simpler" + likes constellations |
| 9 | motion | **still until touched; always answers when touched** — no ambient/idle motion ever; every user action gets an immediate quiet response from the closed timing scale; `prefers-reduced-motion` = responses become instant state changes (never removed) | "calm" + fifth pass |
| 10 | world-theming | none in copy — stations are structure, never narrative (no metaphor named on-page) | "safer" |
| 11 | mark | A Confluence primary (4 stars + hairlines); B Asterism as the hero figure | "I like your representation of DAGs" |
| 12 | copy voice | terse claims, no marketing verbs (frontend `PROPOSAL.md` register); study-image garble was model noise, not a copy proposal | "don't like the text" (copy half) |
| 13 | derived-detail rule | every intricate element computed from a repo fact (constellation = layer DAG · frieze = module graph · nebula = diagnostic hash) | the standing strategy |
| 14 | four-object test | every visible element = fact / relation / boundary / receipt | continuity with frontend `DESIGN.md` |
| 15 | interaction vocabulary | closed verb set (hover=approach · click/focus=enter · Esc=step back · copy · arrow-traverse) × closed response set (brighten · reveal · counterpart-highlight · receipt) × a ~4-token timing scale; **coverage law: no dead hovers, no pretending chrome** — countable at emit | fifth pass (the Discord re-read, §4l) |
| 16 | themes | dark + light as **total role→material assignments from one model** (`Theme` rows in `gunbc.site.tokens`): totality, role-area coherence, glare envelope, and contrast all witnessed; every page color routes through a role var — a literal color is a counted census row | seventh pass ("model dark + light themes in dag, coherent interfaces") |
| 17 | theme reframe | **universal by derivation** (eighth pass): a theme flip cross-fades the whole surface on one clock (`reframe` timing token, 320ms) — every role-routed color transitions for free (`html.reframing` rules derived from the same `BuildRule` rows); the emitted watcher pins `:root[data-theme]` so the fade cannot race the media query; no JS = instant recolor (degraded, never removed); reduced-motion = instant | "universally nail dark→light and light→dark — everyone either gets it for free or has to implement it" |

Signed rows become construction (token rows + lenses) in Phase 0; struck rows loop with a replacement proposal. The checklist itself follows §6: it dissolves into the `dag/gunbc/site/` token/lens carriers as each row lands — the doc row is the authority only until its carrier exists.

### 4l. Behavioral intricacy — the correction that re-aims the budget (fifth operator pass, 2026-07-20)

The operator's correction: **"intricate" was never necessarily visual** — Discord is visually simple; what is incredibly intricate is its *behavior and feedback*. That re-reads the §4d decomposition: the game-feel lesson is not ornament and not variety — it is **coverage × consistency × timing**. Discord's feedback vocabulary is small; it feels intricate because *every* element implements it, identically, at precise timings — consistent physics is what makes a world. (Linear is the industry proof the same craft composes with quiet.) So the intricacy budget re-aims from drawing to response:

- **The law: still until touched; always answers when touched.** No ambient or idle motion, ever (the calm half). Every user action receives an immediate, quiet, proportional response (the intricate half). `prefers-reduced-motion` converts responses to instant state changes — feedback is never *removed*, only de-animated.
- **A closed interaction vocabulary, modeled as rows** and realized twice (§2 Realization — CSS states where stateless, `TsProgram`-emitted JS where stateful; §3 Stage-N):
  - **verbs:** hover = approach · click/focus = enter · Esc = step back · copy · arrow-traverse (↑/↓ waypoints, ←/→ deck);
  - **responses:** brighten (a star lifts, its name appears) · reveal (one depth increment) · counterpart-highlight (the same fact lights in code and graph at once) · receipt (the action answers in the compiler's voice: `copied · sha256 …`);
  - **timing:** a closed ~3-token duration/easing scale (respond fast · reveal deliberate · restore gentle) — timings are tokens, like the stroke scale, never per-site choices.
- **The coverage law (what actually reads as intricacy):** every interactive element implements every applicable verb — no dead hovers, no default focus rings, designed press/selection/visited states — and the inverse honesty: nothing non-interactive pretends (cursor affordances never lie). Coverage is **countable at emit**: behavior rows per element are enumerable, so "element with verb X and no response row" is a located defect, not a review vibe.
- **Responses are derived (§4c's honesty rule extended to time):** a response must reveal a *real* fact — the actual type, the actual hash, the actual receipt. A hover that performs but reveals nothing is decoration in time instead of space, and equally banned.
- **Budget consequence:** the visual-ornament generator program shrinks — frieze bands and hash-coffering are **parked** (optional, moodboard-decided); the hero constellation and the mark remain the only figure-detail carriers; the nebula boundary treatment stays but simple (refusal = the light stops + the diagnostic in the compiler's voice). The saved budget funds the behavior rows.

What this buys the product story: feedback consistency is *generated* — one vocabulary, N elements, zero drift between one element's hover and its neighbor's — so the site's **feel** demonstrates the compiler the way its ornament was going to: the intricacy you feel is compiled.

## 5. Phases (each independently landable; triggers named)

- **Phase 0 — model + moodboard.** `dag/gunbc/site/` module home; `Material` tokens + derivation fns + the §4i craft-floor scales (spacing · type · stroke · radius · contrast · luminance budget, as typed rows); panel/type/SVG/ornament typed vocabulary; the §4f component rows with their depth ladders (`render(fact, depth)` fold shape); the §4e mark candidates (A–D) rendered from specs at 16/24/48/120 px; the §4l behavior rows (verbs · responses · timing tokens) + the hero-constellation and hash-nebula generators (frieze/coffering parked per §4l); emit `moodboard.html` + `primitives.html` (every primitive at every depth D0–D3; D4 = its emitted source form) as committed, drift-gated artifacts (published at non-linked paths once Phase 2 lands; reviewable on the srv1/tailnet path before that). **Gate: the §4k checklist signed row-by-row (approve/strike/amend), then register studies re-cut to the signed rows only — small batches, built to loop cheaply; nothing proceeds on unsigned rows.** No public-site change.
  - **Slice 1 LANDED (sixth pass, 2026-07-20 — concept → generated visual page, by execution).** Six stacked modules under `dag/gunbc/site/`, each consuming only the layers below (§2): `register_principles` (the laws as rows, each naming its enforcing carrier) → `tokens` (palette with `AreaClass` chroma ceilings — a saturated field is inadmissible by row; the timing scale; `BuildRule`, whose `MotionTrigger` has **no ambient variant**, so still-until-touched is unwritable, and whose rows derive the `prefers-reduced-motion` projection) → `interaction` (verbs × responses × timing as `BehaviorRow`s; `coverage_gaps` = the countable no-dead-hover law) → `mark` (Confluence as node/edge rows; `render_mark(depth)` = one authority at D0/D1/D3; `all_forks_rejoin` = the confluence theorem, witnessed) → `constellation` (layer-DAG asterism; star names in an HTML legend — hover answers in the counterpart via `:has()`, proof two surfaces render one fact row) → `moodboard` (the composed page; stylesheet **derived** from the behavior rows via `realize_behavior`, unrealized rows counted). Witnesses: `dag/test/claim/site_register_witness_test.dag` — keystone green by execution (exit 0) with in-file RED discriminators (saturated field refused · dead hover located · unrealized behavior counted · severed fork refutes confluence · keystone exit 1 under sabotage, verified).
  - **Slice 3 LANDED (ninth pass, 2026-07-21 — the accent decision grounded and rendered).** `gunbc.site.starlight`: the night accent IS starlight, so candidates are **Morgan–Keenan stellar classes** modeled as rows (O B A F G K M; perceived colors under the envelope; `lit_variant`/`edge_variant` derived, not hand-picked). Two physics walls fell out: **no green class exists** (blackbody mixes to white — green is not a possible starlight; it enters only as `verdigris_patina` material on hairlines), and **M prunes itself** (its lit variant fails the glare envelope — witnessed). `gunbc.site.accent_study` emits the comparison page: four candidate panels (O·A·G·K) scoping the same four figure-role vars over identical constellation fact rows, plus the verdigris-as-material exhibit and the accent-independent day atlas. Witness: `site_accent_study_witness_test.dag` keystone green (candidates admissible · no-green-class · M-pruned · G == live night figure · page emits). Preview-only; the signed class's rows land in `tokens.night_theme` and the study dissolves.
  - **Slice 2 LANDED (seventh + eighth passes, 2026-07-20 — themes + glare envelope + universal reframe, by execution).** The rendered cyan rejected → `quiet_envelope` (bright ∨ chromatic, never both; the rejected `hsl(194 88% 76%)` is now the witness's negative control, and `witness_gamer_cyan_absent_from_page` proves no literal of it survives in the emission). `Theme` = total role→material assignment (11 `ThemeRole`s; totality/area-coherence/contrast/envelope all witnessed; **night** = warm starlight on indigo, **day** = indigo ink on warm paper — same fact rows, two projections via `:root` vars + one `prefers-color-scheme` block). **Universal reframe** (`theme_transition.dag`): `html.reframing` cross-fade rules **derived** from the same `BuildRule` rows (role-routed colors transition for free; `unthemed_color_decls` = the counted opt-out census, empty on the live page, RED-controlled); emitted watcher (`TsProgram` → `nav.js`, `node --check` green) pins `:root[data-theme]` so the fade cannot race the UA's media-query restyle (the race was **measured**: per-frame luminance analysis showed single-frame snaps before the fix, 12–16-frame ~300ms fades after — ffprobe receipts in-session); reframe duration is one token (`reframe_page`, 320ms) shared by CSS and JS. Not yet landed: generated-artifact registration (page + `nav.js` not yet committed artifacts), `primitives.html`, mark candidates B–D as specs, craft-floor spacing/type scales, luminance-budget row, hash-nebula generator, an in-page theme control (the OS toggle is the only flip surface today).
- **Phase 1 — the page.** `site_content.dag` (copy, install command, deck card facts with witnessed samples per §3 Stage-D); static card rendering at emit time; all depths shipped in the DOM with CSS-state reveals + focus parity; Stage-N `nav.js` via `TsProgram`; emit `site/index.html` + `site/assets/site.css` + `mark.svg` + `CNAME` + the §4h agent surface; icons as digest-pinned seed-retained rows; a declared page-weight budget row (the all-depths DOM is bounded by a committed byte budget, drift-visible, not a vibe); all on the generated-artifact gate. Site is fully reviewable at the Pages preview URL and on srv1. **Gate: DESIGN-rules lens/review pass + witnessed-sample witnesses green + §4l behavior-coverage count green (no dead hovers, no pretending chrome).**
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
