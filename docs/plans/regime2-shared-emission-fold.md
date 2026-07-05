# Regime-2 emission: one shared fold for the pure-projection formats

> Kills the boutique `*_emit.dag` proliferation by collapsing the **pure-projection** serializers (`serialize_yaml`, `serialize_gitignore`, `serialize_runner_deploy`) into **one** fold of a protocol over a shared emission model. DESIGN refs: §2 (no duplicated layout logic — one concept, every format), §3 (single authority — the layout primitive lives once in `std`), §4 (the destination is emission = ingestion⁻¹; this is the seed-layer, forward-only *subset* of it), §5 (emit-only formats have no round-trip oracle — be honest at the boundary), §6 (DFS the concept DAG before minting; price the work as the displaced boutique-duplication pain, not elegance).

## 1. The two regimes (why this is a clean split, not a fork)

Emission has two regimes, and keeping them separate is correct:

- **Regime 1 — grammar-inverse** (`emission = ingestion⁻¹`): formats with a *grammar* that is parsed and emitted from the **same** rows read in both directions. Realized in v2 `06_translate`: `emit = serialize_target ∘ translate`, a **closed** fold over `TargetModel`'s `target_model_edge_translation_rules`. Languages (Rust/TS/Go/C++/DAG) live here; markdown/yaml *could* migrate here iff we choose to parse them. The faithfulness oracle is the round-trip law `ingest ∘ emit = id`. **Out of scope for this doc** — that's bright-stag's lane (markdown) + the Route-A self-host last mile.
- **Regime 2 — pure projection** (forward-only, never parsed): `.gitignore`, the runner-deploy manifest, and `ci.yml`-as-config. We **emit** these from an authority and **never ingest** them, so there is no round-trip oracle — faithfulness rests on the authored projection (§5 honesty boundary; do **not** claim a round-trip these formats can't have).

The boutique pain is entirely in **regime 2**: three hand-rolled serializers, each re-inventing indent / line-join / separator / comment layout, over three different-or-absent IRs:

| emitter | file | IR today | serialize fn |
| --- | --- | --- | --- |
| yaml (ci.yml) | `dag/extdeps/languages/yaml/types.dag` + `dag/extdeps/languages/yaml/emit.dag` + `dag/gunbc/ci_yaml_emit.dag` | `YamlValue` (a real structured IR) | `serialize_yaml` — hand-rolled recursive `match` (block/flow seq, scalar quoting, indent) |
| gitignore | `dag/gunbc/gitignore_emit.dag` | **none** — raw `concat` | `serialize_gitignore` — hand fold over `IgnoreGroup` |
| runner_deploy | `dag/gunbc/runner_deploy_emit.dag` | **none** — raw `concat` | `expected_runner_deploy_manifest` — hand fold over hosts |

That's the §2 violation: layout logic duplicated three ways.

## 2. The target shape — protocol fold over an emission model

One fold replaces three:

```
expected_X = render( project_X_to_doc(authority) , X_protocol )
```

- **The emission model** = a generic **layout / document IR** in `std` (the standard pretty-printer `Doc`: `text` / `line` / `nest` / `concat` / `sep`, whatever the faithful minimal set turns out to be). This is the "one concept, every format" carrier. **DFS `std` first** (§6): confirm no existing layout primitive before minting. Note `std.markup.Fragment` (Element/FragmentText/RawNode) is a **markup** tree (HTML tags) — a *different* concern from line/indent layout, so it is **not** the shared IR here; confirm and move on. v2's `serialize_target` has layout entangled with grammar rows, so it is not seed-usable here either. A new `std.layout` (or similarly-named) module is therefore net-new concept, not re-invention — but **prove that** by the DFS, don't assume it.
- **The protocol** = a per-format record of the genuinely format-global spelling knobs (indent unit, comment prefix, newline style — whatever survives after structure moves into the `Doc`). Prefer encoding structure in the `Doc`; keep the protocol record as thin as the formats actually require. If it ends up near-empty because the `Doc` determines everything, that is a fine outcome — say so.
- **`project_X_to_doc`** = the per-format projection that builds the `Doc` from the authority, baking any format-specific **text** decisions (e.g. yaml scalar quoting / escaping) into `Doc` text nodes at projection time — **never** into the render fold.

### The discriminator that keeps this honest (not a 4th hand-fold in disguise)

**Zero format-specific layout branches in `render`.** All format-specifics live in `project_X_to_doc` (text prep) and the thin protocol record (global knobs). If a format forces an edit to the `render` fold, the IR is wrong — stop and remodel, don't special-case. Land a witness that asserts this: the same `render` fold produces all three formats' output, parameterized only by `(doc, protocol)`.

## 3. Faithfulness (the no-regression gate)

This is a **refactor of how we emit, not what we emit.** Every migrated emitter must produce **byte-identical** output to today:

- `git diff origin/main -- .gitignore` empty after regen; same for `ci.yml` and the runner manifest.
- Keep the **public entry-point signatures** (`expected_gitignore()`, `expected_ci_yml()`, the runner manifest fn) **unchanged** — gentle-ibex-384's universal generated-artifact gate calls these via `artifact_generate`; changing the internals must not change the surface the gate folds. (Coordinate: this is the *emit* half; the gate is the *commit/drift* half — they compose, don't collide.)
- Per-format witness: `render(project_X_to_doc(authority), X_protocol) == <the current expected_X output>` on a real subject, plus a discriminating perturbation that goes RED (drop a group / flip a pattern).

## 4. Scaffold honesty + dissolution trigger

Regime-2 `render` + the `Doc` IR are seed-realized and **explicitly scaffold-marked**: the dissolution trigger is the v2 `TargetModel` grammar-row inverse subsuming them once self-host reaches the seed — the `Doc`/protocol spellings are a forward-only **subset** of the rows that `serialize_target` will consume, so this is a shrinking-roster scaffold, **not** a competing emitter. Mark it exactly as `yaml.dag` marks itself today (it carries a SCAFFOLD / §3-fork-of-v2-yaml note with a dissolution trigger — that note relocates onto the shared fold).

## 5. Sequencing

1. **`std` layout IR + `render` fold** — DFS first; mint the minimal faithful `Doc` + the single render fold; one witness that `render` is format-agnostic.
2. **gitignore + runner_deploy** — the trivial line-oriented formats (no IR today). Project each to `Doc`, delete the hand fold, byte-identical witness.
3. **yaml (ci.yml)** — the heavy one (nesting, block/flow seq, scalar quoting). `project_yaml_to_doc` maps `YamlValue → Doc`, baking quoting into text nodes; delete `serialize_yaml`'s layout half, byte-identical witness against the committed `ci.yml`.
4. **markdown — NOT here.** Markdown is regime-1-or-projection and is bright-stag's lane; this doc does not touch `std.markdown` / `roadmap_emit`. Flag the seam, don't cross it.

## 6. Open / boundaries

- Whether yaml *eventually* moves to regime 1 (parseable, round-trip-gated) is a separate decision; today it is emit-only → regime 2.
- The `std.layout` `Doc` is the first real candidate for a *shared* layout primitive that even regime-1's `serialize_target` could later reuse for its pure-layout part — **noted, not built**; do not entangle.
- `ci.yml`'s YAML-parse extra-check (`ci_yml_parses`) is gentle-ibex's gate concern, unaffected here.

## Dissolution trigger (DESIGN §6)

Delete this doc when the three regime-2 pure-projection serializers (serialize_yaml/serialize_gitignore/serialize_runner_deploy) are collapsed into one render fold over a shared std layout Doc IR — byte-identical-witnessed against the committed ci.yml/.gitignore/runner manifest with a format-agnostic-render witness — so the boutique *_emit.dag duplication is gone; the scaffold's own dissolution (the v2 TargetModel grammar-row inverse subsuming the shared fold at self-host) then carries the rest.
