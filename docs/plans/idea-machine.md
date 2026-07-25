# Plan — idea → idea compiler (the idea-machine)

**Status:** planning tracker · **DESIGN.md + carriers are authority** (§6). A task's real state is its branch/PR, not this file. Linked from `ROADMAP.md` §6 *idea → idea compiler*. No prior plan doc existed — the landed commits + carriers are the only authority; this collects them.

**Carrier facts verified against the live tree 2026-06-21.** Re-check receipts before acting.

## 0. Thesis — stop anchoring on code

A program is a canonical `Node` tree — *the idea*. "Code" is just one **medium** the idea can be carried in. So compile is not code→code; it is **ingest any medium → idea → emit any medium / evaluate**, all through **one grammar read in both directions** (DESIGN §2: the same `target_model_edge_translation_rules` rows select *forward* to parse and *backward* to emit — the structural inverse, not a second emitter). Adding a medium is **data rows in `extdeps/`, never a compiler edit** (N+M, not N×M).

Two axes, at very different maturity: the **medium axis** (what representation carries the idea — mostly landed) and the **language axis** (which surface syntaxes — footholds only).

## 1. Medium axis — mostly landed

The "recorded technical medium" framework: how an idea moves between representations with fidelity.

- `Medium<R>` + `DecodeFidelity` (`Lossless | Lossy`) carrier — `dag/extdeps/communication/medium.dag`. R-parameterized so text / verdicts / nodes / binaries share one model with no text-bias.
- `LanguageModel` **unified** — #5222 dissolved 13 per-language `*LanguageModel` forks into one carrier (`src/v2/std/language_model.dag`); each language is an inhabitant.
- `Source` / `TargetSource` / `DagSource` nickname fork collapsed → `Medium<String>` — #5246.
- Compile codomain uniform — `compile(Eval) → EvalResult{value: Medium<Node>}` (#5191 + #5298): **evaluation results are first-class ideas (`Node`), not stringified**, `Lossless` (exact-or-fail-closed). Discriminating RED: a unit literal is correctly rejected as unrepresentable.
- Cross-tree `Medium<Node>` instantiation proven — #5217.

Staged: `FidelityDisposition<Feature>` (`src/v2/extdeps/languages/fidelity.dag`) composes up to medium-level `DecodeFidelity` at the decode boundary (today feature-level stays in extdeps/languages).

## 2. Language axis — footholds, breadth partial

- 15+ targets at the seed grammar subset (`src/v2/extdeps/languages/`: rust, python, go, dag, typescript, cpp, lean, wasm, java, kotlin, swift, llvm_ir, machine_code, english, …).
- **English is a proven emit target** — `english.dag` (article-led SVO + copula), and `src/v2/test/claim/manual/english_emit_add_test.dag` round-trips emit(add)→tokenize→parse→identity.
- Partial / fail-open: English **ingest** uses a catch-all `english_token_word` → fail-open on out-of-subset prose (also a **§0 lock-down item**); only emit is proven.

## 3. Remaining work (dependency-ordered → ROADMAP §6 *idea → idea compiler* substeps)

1. **English vocabulary closure** → fail-closed English ingest (replace catch-all with closed/typed word set). *Intersects the §0 lock-down.*
2. **English ingest round-trip** — tokenize/parse proof for ingest (today `boundary/english_ingest_fail_closed.dag` marks it fail-closed; complete the bidirectional round-trip).
3. **Cross-media targets beyond syntax** — JSON / protobuf / react / diagram as **first-class media** (structured, not stringified). React/JSX couples syntax + runtime value. **HTML is now a first-class `Medium<Fragment>`** (`dag/extdeps/languages/html.dag`): `HtmlSpellings`-parameterized serializer + recursive-descent ingester prove the round-trip law (ingest∘serialize = identity, `DecodeFidelity`-bounded); `validate_href` enforces fail-closed on ingest for unsafe URL schemes; 5-witness receipt in `dag/test/claim/html_roundtrip_test.dag`.
4. **`Medium<A> ↔ Medium<B>` homomorphisms** — generalize translate's `coercion_fold` beyond strings (Realization pattern applied to media).
5. **FidelityDisposition compose-up** — per-feature dispositions reduce to medium-level `DecodeFidelity`.
6. **Eval runtime generalization** — move the seed literal semantics from synthetic pins into `bool_model_core` primitives (T-22); per-Node-ID identity, not fixture-global singletons.

## 4. Relationships

- **Self-host** ([v2-self-hosting.md](v2-self-hosting.md)): the multi-target emitter (rust + typescript) is the *same* `fold_node` + reverse-parse machinery — self-host's targets are idea-machine media.
- **Lock-down** ([fail-closed-lockdown.md](fail-closed-lockdown.md)): English ingest fail-open is a lock-down item; the idea-machine's "exact-or-fail-closed" (eval `Lossless`) is the §5 posture.
- **Website product** (ROADMAP §7 *HTML / React rendering*): react/html as a first-class medium is item 3 above — §7 depends on this lane.

## Dissolution trigger (DESIGN §6)

Delete this doc when ingest+emit are proven across ≥2 non-code media fail-closed (e.g. English round-trip + one structured medium) and the `Medium<A>↔Medium<B>` homomorphism is general — at which point "adding a medium = rows" is a witnessed property and this tracker is redundant.
