# gunb.ai → gunbc projection: pre-modelling inventory

Survey date: 2026-10-16. Lane: `neat-boar-777`, worktree `session/neat-boar-777`
(fast-forwarded to `origin/main` @ `eab89b31a73`, which carries PR #14131 —
`design.brand_mark_assets` and `brand_mark_assets_cli` are on main).

This is the inventory the brief demands **before any modelling**. It reads only;
no site file was modified. Every row was checked against the live site
(`https://gunb.ai`) and against `git ls-tree HEAD` of `gunb-ai/frontend`
(HEAD `3152d4c`, 2026-06-16, "v1: remove Architecture nav link from masthead (#20)").

## 1. What gunb.ai actually serves

GitHub Pages on `gunb-ai/frontend`: `build_type=workflow`, source `main/`,
`custom_domain=gunb.ai`, url `gunb-ai.github.io/frontend`. The deploy workflow
(`.github/workflows/pages.yml`, 1 run ever, 2026-06-16) stages **only**
`index.html`, `assets/`, `CNAME` into `_site`. So the served surface is 10 files,
95,930 bytes; the live `/` is byte-identical to the tracked `index.html` (verified
by `diff`).

| published file | bytes | what it is |
|---|---|---|
| `index.html` | 3,593 | one page: masthead (logo→mark.svg, "gunbc", GitHub link), hero (h1 + 4-line tagline + install box + 2 CTAs), 3 value cards, footer |
| `CNAME` | 8 | `gunb.ai` |
| `assets/site.css` | 23,490 | 680 lines, 116 rules, 4 media queries, 36 custom properties, 0 undefined `var()` |
| `assets/cards.js` | 45,058 | deck + constellation runtime — **does nothing on this page** (§3) |
| `assets/mark.svg` | 644 | hand-authored fork glyph, a second version of `design.brand_mark`'s glyph (§4) |
| `assets/favicon-16.png`, `favicon-32.png`, `apple-touch-icon.png`, `maskable-icon-192.png`, `maskable-icon-512.png` | 19.6 KB | **referenced by nothing published** (§3) |

Visible copy on `/` is 16 text lines, all hand-typed HTML — h1, 4 tagline lines,
3 card headings + bodies, footer, title. Not one of them exists anywhere in `dag/`
(grepped each string against `*.dag`: zero hits). The whole page's content is
unsourced text.

### Branch content that is never served

`workpad.html` (125 KB), `legacy/` (12 pages + fixtures, ~435 KB), `docs/`
(PREVIEW-HOST, DEV-PREVIEW-ARCHITECTURE), `PROPOSAL.md`, `DESIGN.md`, `scripts/`
(2 shell scripts) — 577,855 bytes, tracked in git, excluded from `_site`, and
confirmed 404 at gunb.ai (`/workpad.html`, `/legacy/showcase.html`,
`/docs/PREVIEW-HOST.md`, `/PROPOSAL.md` all 404). `docs/PREVIEW-HOST.md` and
`scripts/install-mac-mini-autopull.sh` describe the retired "port 7777 on the
mac-mini" preview system.

## 2. Staleness

- Site last touched **2026-06-16**; `gunbc` has **2,043 commits** on main since.
- `cards.js` mounts a deck whose content is hand-authored JS describing a fictitious
  `Users` model (`profile_rs`/`email_rs`/`token_rs`) that exists in no `.dag`.
- `/assets/site.webmanifest` and `/assets/favicon.png` → 404: `legacy/site.webmanifest`
  points `icons[]` at `/assets/*`, and `assets/` holds only 16/32px favicons + two
  maskables + an apple-touch icon. Icons are served and never referenced.
- `gunb-ai/gunbc` still has Pages enabled (`build_type=legacy`, source `main/`) and
  404s — stale config, since main carries no site.
- `gunbc.io` does not resolve (NXDOMAIN): the only domain in play is `gunb.ai`.

## 3. Dead weight in the published set

`cards.js` (45 KB, 47% of the published bytes) is downloaded on every visit and
renders nothing: `initDecks()` bails on `document.querySelector('.card-track')` and
`index.html` has no `.card-track` — its own header comment says "the public page
keeps a static composition; the live deck lives in workpad.html". It is served to
the public page because the workflow copies all of `assets/`, and it is only
`git`-shared with the never-served `workpad.html`. Its one real job — driving
`[data-deck="users.dag"]` (index.html:69) — can never run.

## 4. What `gunbc` already models (homes to consume, not rebuild)

`gunbc.design.*` (on main): `brand_mark` (1,198 L — the glyph, wordmark, lockup
`Fragment`s + geometric controls), `brand_mark_assets` (144 L — byte-exact PNG/ICO
bytes, `BrandLogoFile` = path+bytes+media_type), `brand_mark_assets_cli` (out-dir
writer via `Filesystem.WriteBytes`, already lays out `gunb.ai/` `ios/` `github-apps/`),
`brand_mark_raster` (666 L), `brand_mark_svg_wire`, `palette` (HSL `Swatch` + 24-colour
named set, `SwatchName` incl. `Ok`), `typefaces` (family/style/weight axes), `theme`
(398 L, `Theme` + `ThemeRole` incl. `CanvasRole`/`TextRole`, `theme_var_decls`),
`material` (register machinery), `icon` (971 L), `geometry`, `proportion`, `ink`.

`gunbc.site.*`: `markup` (`txt`/`el`/`el_class`/`el_attrs`, `Attribute`), `document`
(`PageMetadata`, `PageStylesheet`, `render_page_document`), `brand`
(`site_brand_mark`, `site_brand_link`, `site_favicon_source` — already rendered into
the workspace site's `<head>` and served at `site_favicon_path` by
`roadmap/roadmap_serve.dag:389,1450`), `tokens` (L5 façade note pointing at
`design.material`+`design.theme`), `web_fonts` (296 L), `layout`, `accent_study`,
`moodboard`, `specimen`, `specimen_meter_client`, `constellation`, `interaction`,
`status`, `controls`, `choice`, `element/*` (button, card, chip, text, styles,
tokens, section_heading, …).

Review-site precedent already exists: `site.accent_study` and `site.moodboard`
render options to HTML and are auditioned on srv1 (`briansrls@100.69.18.126`,
aarch64, 128 cores; playwright chromium + `headless_shell` present — screenshot
path verified: 4,363-byte PNG). srv1 does **not** share this worktree (`~/.worktrees`
there is a different 149-entry set); the brief's "shares the worktree" is wrong —
generated output must be `rsync`ed. `--arg` is not a `gunbc run` flag; the
out-dir goes to `v2_cli_main(argv)` as a positional, per `brand_mark_assets_cli`.

**Colour home check.** `assets/site.css` is better off than expected: 0 colour
literals outside `:root`, 36 well-formed tokens, no dangling `var()`. The gap is
provenance, not discipline: its hues (warm 38°, moss 148°, brass 40°, clay 16°)
and names (`--void`, `--brass`, `--clay`) share **no name or value** with
`design.palette`'s 24 swatches. Two independent colour systems exist. Its
`--gap-*`/`--measure-*`/font stacks have no `.dag` home either (no `spacing.dag`
or `typography.dag` under `site/`; `design.typefaces` owns families/styles/weights).

## 5. Piece → home, or gap

| piece (published) | home in gunbc today | verdict |
|---|---|---|
| brand mark in masthead | `design.brand_mark.brand_mark_glyph()` + `brand_mark_svg_wire` | **re-home** (`mark.svg` is a hand copy) |
| favicons / touch / maskable PNGs | `design.brand_mark_assets.brand_asset_files()` (byte-exact) | **re-home** — and fix the manifest paths that 404 |
| colours (36 tokens) | `design.palette` swatches + `design.theme` roles | **gap**: no `gunb.ai` house register binding the 4 hues to swatch names |
| type scale, spacing, radius, measures | partial (`design.typefaces` families; no scale/space register under `site/`) | **gap**: measures/scale need a typed home |
| page shell (head/meta/stylesheet link) | `site.document.render_page_document` | consume |
| markup primitives | `site.markup`, `site.element.*` | consume |
| font loading | `site.web_fonts` | consume |
| copy: h1, 4 tagline lines, 3 cards, footer, title | none | **gap**: page-copy content model |
| links (GitHub CTA, repo) | none | **gap**: typed link table with liveness check |
| install snippet | none | **gap** (and: must not be a second version of an install command that lives in the repo's own docs) |
| deck (`data-deck`, 45 KB JS) | fictitious `Users` model; nothing in `dag/` | **drop** (nothing upstream to project); or model it and let the instrument own the numbers |
| `legacy/`, `workpad.html`, `docs/`, `*.md`, `scripts/` | never published | **retire with the repo** |
| Pages publishing | no site-build module exists (`site.hosting`/`site.build` absent) | **gap**: build+publish layer, new CI workflow → operator sign-off |

## 6. Decisions this inventory forces (operator)

1. **Deck**: drop the 45 KB `cards.js` + `[data-deck]` slot from the generated site,
   or rebuild it from a real instrument? It ships dead today; "parity" cannot mean
   reproducing dead weight.
2. **Value cards / tagline**: 3 cards + 4 tagline lines are the only content claims
   on the page. They need a typed home in gunbc, not transcription. Which is the
   upstream that answers "how many modules/declarations does gunbc have" — an
   instrument (the repo has `instruments/*`, `declaration_instrument`), or fixed
   operator copy?
3. **Colour**: adopt `design.palette` as upstream and bind the gunb.ai house to it
   (names + values change on the live site), or model the existing 4 hues as new
   swatches in `design.palette` and keep today's look?
4. **Pages target** (unchanged from the brief): keep `gunb-ai/frontend` as the
   Pages repo with gunbc pushing `_site`, or publish from gunbc and retire frontend.
   DNS/domain is outward-facing — nothing goes live without an explicit ask.
5. **New CI workflow** for build+deploy needs explicit sign-off (job roster closed).

## 7. What is next, once the picks land

Layer order per the brief, each with 2–4 rendered options on srv1 and an operator
pick before the next: tokens → primitives → layout → content model → pages →
build+deploy. Each layer ships a witness/control that renders it and checks a
property that goes red when the layer breaks (not a grep, not a typecheck).
