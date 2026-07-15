# No-smuggled-programs wall — a string literal is an ATOM, never a COMPOSITION

> Operator directive, 2026-07-14. **Signed 2026-07-14** — the operator answered the four §8 open questions (Q1 realized `lang` field · Q2 `TargetText` name as proposed · Q3 the `*_source_text` concat pyramids are a dual representation and dissolve into one row-derived tokens→text fold, folded into Slice A0 · Q4 ubuntu-media Face-B entries are interim, dissolving to the in-process typed reconcile lane). **Scope of this PR:** the design doc + corpus census **and** HALF A **Slice A0** (the Q3 row-derived `bound_tokens_source_text` fold + the surgical-7 orchestration→bash productions threaded through it). Design and Slice A0 landed together in one PR because an eager auto-committer captured the first Slice-A0 edit into this branch mid-authoring; rather than force-push the invalidated docs-only approvals away, the PR carries the combined, honestly-scoped change (parent ruling, 2026-07-14). The full `06_translate` serialize-family carrier migration (Slice A1) and the HALF B lens remain **staged behind their own sign checkpoints** (§7). DESIGN refs: §2 (one concept — statement-joining is *one* piece of language knowledge, not re-spelled per call site), §3 (single authority — joins live in the `extdeps/languages` rows, never in workflow code; no parallel carrier), §4 (the richer source always exists in a closed system — the grammar itself is the classifier, never a regex heuristic; emission = ingestion⁻¹), §5 (correctness by construction — make the splice class *unwritable*, not caught-after-the-fact; fail-closed with a typed/located/counted refusal — the Unknown bucket is not an absorbing fallback), §7 (the wall holds exactly where grammar rows exist — the `DecodeFidelity` honesty boundary), §6 (priced as the displaced pain: hand-spliced control flow that drifts silently and blocks every language's emit path at once).
>
> **Relation to prior art.** Complements [shell-emission-model](shell-emission-model.md) (intent→bash via `emit(intent, Bash)`; **`std.layout.Doc` = LAYOUT only, never shell content** — a constraint this doc obeys, see §3), [regime2-shared-emission-fold](regime2-shared-emission-fold.md) (`Doc` for pure-projection layout), and generalizes the existing [`medium_structure_containment`](../../src/v2/lens/medium_structure_containment.dag) lens (§4). This wall is one instance of the [enforcement-intent](enforcement-intent-design.md) `StandingIntent` pattern (§6) — align shapes, do **not** re-implement that design.

## 1. The rule

**We do not smuggle embedded programs inside code as strings.** Program content is *modeled* and emitted through the normal channels (the grammar rows read in both directions, DESIGN §4). Concretely, about a `String` literal:

- **ATOM — allowed.** A word, path, name, or literal *value*: `"BUILD_LOG"`, `"origin/main"`, `"$HASH"` as an opaque token, a lexeme spelling in a grammar row (`span(text: "; ")`). An atom carries no program structure; it is a leaf.
- **COMPOSITION — forbidden.** A statement separator (`"; "`), a control production (`"; then "`, `"; fi"`, `"; else "`, `"for … ; do"`, `"done"`), an operator-joined subprogram (`" && "`, `" || "` gluing two commands), or a framing fragment (`"$("`…`")"`, `concat("$", n)`). A composition encodes *grammar* — it belongs to the language, so it must come from the language's rows, not from a `concat` in workflow or compiler code.

The distinction is **decidable and grounded** (DESIGN §5): a bare string literal is an atom; a string produced by `concat`/`list_append`/interpolation of ≥2 program fragments, **or** a string literal that *parses* (under a language with `extdeps/languages` grammar rows) to a composition, is the forbidden case. Because it is decidable and grounded, it is a **wall** (construction), not a ratchet — with a lens for the residue the construction cannot reach (§4). The rule is **general across languages**, not bash-specific; bash is first only because that is where the live pain is.

The displaced cost (DESIGN §6): hand-spliced control flow is a second representation of the grammar (`"; fi"` is *bash knowledge* re-spelled at each call site). It drifts silently — nothing checks that the splice matches the grammar — and it is per-language-per-call-site, so it defeats "a new target is a row" for the *joining* half. Killing the splice class is content-free (it says nothing about *what* the program does) and closes every language at once.

## 2. Where the composition happens today (the two faces)

The corpus census (§5) finds composition-into-String at two faces, and each face gets its own half:

- **Face A — the emit seam** (`Outcome<Medium<String>>` / `-> String` fold families). Emitted subprograms flow as raw `String`, so a workflow author *can* write `concat(emitted_text, "; fi")` and it typechecks. **HALF A** re-types this seam so the flowing value is a typed carrier, not `String` — the splice fails to typecheck (§3).
- **Face B — substrate string literals that never touch the seam** (heredoc bodies, `RawLine { text: "…" }`). No emit fold runs over these; they are authored program text sitting in `.dag` data. **HALF B** is a lens: parse each substrate string literal with the grammar of a language that has `extdeps/languages` rows; a literal that parses to a composition is a typed/located/counted violation (§4).

**Legitimacy channel (both faces).** Some strings *look* like program fragments but are **data, not authority**: a grammar row's own lexeme spelling (`span(text: "; then ")` — this is the *definition* of the atom, the single authority the wall points *at*); a test golden that is compared, never executed; a cited extdeps payload (a real upstream API request body). These ride the **declared exception roster** (`medium_structure_exception_roster`, already live), checked by **model walk, not grep** (§4). The lens *reads* the grammar rows to classify; it must not flag the rows that *are* the grammar.

## 3. HALF A — construction (the emit-seam type change)

### 3.1 The carrier decision (correction: not `Doc`)

`shell-emission-model` §2 rules **`std.layout.Doc` is LAYOUT only, never shell content**. So the content carrier is *not* `Doc`. Three candidates, weighed:

| candidate | verdict |
| --- | --- |
| **A. `std.layout.Doc`** as the content carrier | **Rejected.** Violates the layout-only constraint; `Doc`'s `DocConcat` would re-admit arbitrary content composition (any two `Doc`s concat), which is exactly the splice the wall forbids. `Doc` stays for pure-projection *layout* (regime-2). |
| **B. Nominal `TargetText<Lang>`** — a newtype over emitted source, distinct from `String`, home in `07_target_carriers.dag` | **Chosen.** Nominally ≠ `String`, so `concat("; fi", t)` fails to typecheck (the operator's stated consequence). Composition is only via grammar-owned join eliminators (§3.2); the sole `String→carrier` introduction is `text_atom` (an atom). Cross-language mixing (`TargetText<Bash>` vs `TargetText<Rust>`) also fails to typecheck. |
| **C. `Doc`-wrapped-per-language** | **Rejected.** Re-admits `Doc` content composition under a tag; still puts shell content in `DocText`; entangles layout with content — the exact seam `shell-emission-model` §2 keeps apart. |

**Chosen carrier — `TargetText<Lang>`** (name to be confirmed at sign; `EmittedSource<Lang>` is a synonym candidate), in `src/v2/compiler/07_target_carriers.dag` (the module that already owns `Medium<String>`/`lossless_source` — no parallel authority, DESIGN §3):

```
type TargetText<Lang> {          // Lang is realized as a lang tag, not phantom — the carrier
  lang: TargetLang               //   knows its language so render picks layout and the lens picks grammar
  source: String                 // the emitted lexeme/subprogram; an ATOM only at introduction
  fidelity: DecodeFidelity       // subsumes today's Medium fidelity thread
}
```

- **The only introduction** `String → TargetText<Lang>` is `text_atom(lexeme, lang) -> TargetText<Lang>` — and its precondition is that `lexeme` is a *single* language-classified token (an atom). This is the one place a string literal is legitimately introduced as content, and it is an atom by construction.
- **Whether `Lang` is a phantom type parameter or the realized `lang` field is a sub-decision** (weigh at sign): a phantom param gives compile-time cross-language safety but the .dag type machinery must support it; a realized field gives groundedness (the carrier drives its own render/lint dispatch) at the cost of value-level (not type-level) cross-language checking. Lean: **realized field** (grounded, dispatch-bearing), with the nominal-distinctness-from-`String` — which is all the operator's stated consequence requires — coming from the newtype itself. Flag as a dissolve-on if the phantom param later proves cheap.

### 3.2 The eliminators (the only sanctioned exits)

Per the directive, exactly two operations may consume/compose carriers; nothing else:

- **(a) whole-value render into the final artifact sink** — `render_artifact(t: TargetText<Lang>) -> Medium<String>`, called **once**, at the top boundary where the artifact string is handed to the sink (`serialize_target` / `orch_emit_pipeline`'s return). This is the single `carrier → String` exit. The existing `Medium<String>` remains the artifact-sink type (no change downstream of the seam).
- **(b) grammar-owned joins** — statement sequencing/joining is language knowledge, so it lives in the `extdeps/languages/<lang>` rows and is realized as `TargetText<Lang> × … → TargetText<Lang>` combinators driven by the grammar rows (the same `orch_construct_seq2` / `realize_if` / `realize_if_else` rows the orchestration path *already* routes through — see `05_emit_orchestration.dag`, which is ~80% grammar-owned today). No `concat: (TargetText, TargetText) -> TargetText` exists; the *only* joiners are the grammar-row combinators. A new language's joins are new rows, not new compiler code (DESIGN §4).

Consequence, stated by the operator and now grounded: **`concat("; fi", emitted_text)` fails to typecheck** — `concat` wants `String`, `emitted_text : TargetText<Bash>`; and there is no carrier-level `concat`. The splice class is unwritable, content-free, all languages at once.

The reach-in residual — a determined author writes `TargetText<Bash>{ source: concat("; fi", t.source), … }`, extracting `.source` to splice and re-wrapping — is **not** closed by the type alone (records expose fields). It is closed by **HALF B's lens** (§4), which catches a `concat`/interpolation producing shell content regardless of the wrapper. HALF A makes the *natural* path grammar-owned; HALF B backstops the *deliberate* bypass. Together they close the class (DESIGN §5 construction-first, lens-for-residue).

### 3.3 Staged migration (do not boil the ocean; do not race P4-G3)

The `-> String` / `-> Outcome<Medium<String>>` fold families are large. Stage by live pain:

1. **Slice A0 — the orchestration→bash seam** (`05_emit_orchestration.dag` + `v2.std.orchestration_emit.OrchestrationEmitMedium` + `v2.extdeps.languages.bash*`). This is the live-pain path. Retype `orch_emit_*_spelling` and the `OrchestrationEmitMedium.realize_*` signatures from `String` to `TargetText<Bash>`; move the residual in-compiler framing splices — `concat("'", pattern, "'")` (grep-quote, :286), `orch_emit_call_args` word-join (:333), `$(…)` cmdsubst framing (:343–345), `concat("$", n)` var-ref (:350) — into bash grammar rows. **Faithfulness: byte-identical** emitted output (§3.4).
2. **Slice A1 — the compiler serialize family** (`06_translate.dag`: `serialize_concrete_syntax_tokens_to_source_string` and the ~30-fn type-expr family, `concat(partial, spelling)` at :811/:821/:947/:969/:1071 and `list_append(head, open, args, close)` at :2051+). Named here in the general statement; **migration stages separately** (its own PR behind this one) — the whole compiler emit path must not be coupled into the first construction PR. `06_translate` is DESIGN-load-bearing; it gets its own sign checkpoint.
3. **Instance owned elsewhere:** `dag/gunbc/fleet_converge_emit.dag`'s `fresh_standup_*_line` builders (the operator-named `concat(emitted, "; fi")` specimen at :102, `"; "`/`"; else "` at :114–153) are being rewritten by the **P4-G3 lane** into one intent tree, dissolving the splice instance. **I own the CLASS, they own the INSTANCE** — this doc cites the file as the motivating specimen but does not edit it.

### 3.4 Faithfulness (the no-regression gate)

Face-A migration is a **refactor of how we emit, not what we emit** (regime-2's bar): every migrated emitter produces **byte-identical** output to today. Gate = `git diff origin/main` empty on the committed goldens (`.github/fleet-converge.sh`, the orchestration-emit witness outputs) after regen, **plus** a discriminating one-byte perturbation that goes RED. Green **by execution** vs frozen bytes — never typecheck/emit/self-referential-gate (DESIGN §5; and note the `live_deploy` self-referential drift-gate trap flagged in `shell-emission-model` §4).

### 3.5 Scaffold honesty / dissolution

`TargetText<Lang>` and its atom/render/join surface are the seed-layer forward subset of the v2 `TargetModel` grammar rows (`serialize_target`'s inverse). Mark it scaffold with the dissolution trigger = the grammar-row inverse subsuming it at self-host. A **self-host frontier row** (`06_translate` migrated | seed-retained{reason, trigger}) tracks Slice A1 — countable, never a silent escape hatch (DESIGN §7).

## 4. HALF B — the lens (substrate strings that never touch the seam)

**Generalize the existing `medium_structure_containment` lens** (`src/v2/lens/medium_structure_containment.dag`, 399 lines, already live with an exception roster + growth ratchet + canonical witness). Two substantive changes; everything else is reused:

1. **Grammar-as-classifier, not markers.** The lens today classifies with a **marker heuristic** (`medium_syntax_markers = ["${{", "#!/", ">/dev/null", …]`). That is exactly the regex heuristic DESIGN §4 forbids ("the richer source exists"). Replace it: a substrate string literal is a **composition** iff it *parses*, under a language whose `extdeps/languages` grammar rows exist, to ≥2 statements / a control production / an operator-join. The **grammar is the classifier** — reuse the bash ingest fold (`ingest`-direction of the same rows §3.2's joins read). A literal that parses to a single atom/word is clean; one that parses to a composition is a violation.
2. **The `Unknown` bucket is first-class and counted** (DESIGN §5, cuts both ways). A literal whose parse is **ambiguous** (or that targets a language with no rows yet) is neither silently clean nor blanket-errored as prose — it is a typed, located, **counted** `AmbiguousParse` disposition. Never absorb ("assume clean") and never widen ("all prose is a violation"). The wall **honestly holds exactly where grammar rows exist** — the `DecodeFidelity` boundary (DESIGN §7): `Lossless` grammar → the wall bites; no/`Lossy` grammar → `Unknown`, reported not gated.

**Legitimacy channel = the existing roster, by model walk.** Data-not-authority strings (test goldens compared-not-executed, cited extdeps payloads, a lens's own comparison-key builder like `duplicate_computation.computation_word_token`'s `concat("$", n)`) stay on `medium_structure_exception_roster`. Membership is decided by **model walk, not grep** (is the string reached only by compare/cite sites, never an emit/execute sink?). Keep the roster-growth ratchet (baseline frozen) so a new exception reds until justified.

**Reuse, no parallel authority** (parent directive): `LensRegistry`/`LensContract`, `ConstructionJustification` (this lens's justification = `WallAfterGrounding { dissolves_to: grammar_classifier }` for the marker→grammar swap, then `RatchetForever` for the genuinely-Unknown residue), `subject_roster`, and the `medium_structure_exception_roster` all already exist. HALF B *extends* them; it mints no new roster.

**Language rollout:** bash first (live pain — the ubuntu-media `RawLine` census below). JSON/YAML/etc. ride the **same registry** automatically when their `extdeps/languages` rows land — no per-language lens code (DESIGN §2/§4).

## 5. Corpus census receipt (required before any gate flips)

Method note: the table below is **grep-seeded and hand-classified against the live tree** to prove scope and the classifier boundaries for sign-off. The **executable, grammar-classified census** (the lens run producing counted dispositions) lands *with* HALF B; the roster and the `AmbiguousParse` count are its output, not this doc's. This receipt establishes that the gate will neither false-positive on atoms/lexeme-rows nor miss the known specimens.

### 5.1 Face A — emit-seam composition (HALF A targets)

| site | specimen | class | disposition |
| --- | --- | --- | --- |
| `05_emit_orchestration.dag:350` | `concat("$", n)` (var-ref) | in-compiler framing splice | Slice A0 → bash row |
| `05_emit_orchestration.dag:333` | `orch_emit_call_args` `concat(acc, concat(" ", a))` (word-join) | in-compiler framing splice | Slice A0 → bash row |
| `05_emit_orchestration.dag:343–345` | `concat("$(", …, ")")` (cmdsubst frame) | in-compiler framing splice | Slice A0 → bash row |
| `05_emit_orchestration.dag:286` | `concat("'", concat(pattern, "'"))` (quote frame) | in-compiler framing splice | Slice A0 → bash row |
| `06_translate.dag:811/821/947/969/1071` | `concat(partial, spelling/source)` (token fold) | compiler serialize fold | Slice A1 (staged separately) |
| `06_translate.dag:2051+` (~30 fns) | `list_append(head, open, args, close)` (type-expr) | compiler serialize fold | Slice A1 (staged separately) |
| `bash.dag:1916`, `bash_orch_if.dag:42–43` | `concat("if ", cond, "; then ", body, "; fi")` etc. | **hand-concat inside the language module** — still a splice (joins belong to *rows*, emitted through the fold, not a `concat` helper); forks with the row-driven `bash_orch_if` emit | Slice A0 sibling → dissolve to the row-driven emit |
| `bash_program_emit.dag:252`, `bash/program.dag:116/126` | `concat(acc, concat("; ", part))` (stmt join) | workflow/sidecar splice | rides `program.dag` dissolution (`emission-ingestion-inverse`); cite, don't fork |
| `fleet_converge_emit.dag:102/114–153` | `concat(emitted, "; fi")`, `"; "`, `"; else "` | **motivating specimen** | **P4-G3 owns the rewrite** — cited, not edited here |

### 5.2 Face B — substrate literals parsing to compositions (HALF B targets)

| site | specimen | disposition |
| --- | --- | --- |
| `dag/extdeps/os/ubuntu_seeded_install_media_remaster.dag:118–151` | `RawLine { text: "if [ -f \"$FINAL\" ]; then" }`, `"fi"`, `"if ! grep -qF … ; then"`, `… \|\| { echo …; exit 1; }` | bash composition literals in substrate → HALF B violation (or intent-model migration; parses under bash rows) |
| `dag/extdeps/os/ubuntu_install_media_fetch.dag:64–82` | `RawLine { text: "for url in " + … + "; do" }`, `"done"`, `"if … ; then"`, `"fi"` | bash composition literals in substrate → HALF B violation |
| `bash.dag:1095–1119` | the CI EAGAIN-retry blob (`") 2>&1 | tee …; then\n  if grep -qiE '"` …) | nested-concat pipeline/control blob → HALF B violation (routes through `emit(Retry,Bash)` per shell-emission-model slice-0) |

### 5.3 Legitimate atoms / data-not-authority (must NOT flag)

| site | why clean |
| --- | --- |
| `bash.dag:189/197/432/455…`, `bash_command_fold.dag:1337–1345`, `bash_orch_if.dag:52–54` | `span(text: "; then ")` / `bash_fold_lex_rule(text: "; ")` — the grammar's **own lexeme definitions**; these are the authority the wall points *at*, not violations. Classified by model walk (they *are* the `extdeps/languages` rows). |
| `go/emit.dag:93` | `go_tuple_separator: String = "; "` — a lexeme constant (atom), consumed by a grammar row. |
| `duplicate_computation.dag:48` | `concat("$", n)` builds a **comparison key** inside a lens (compared, never executed) — data-not-authority → roster. |
| `nbd_proxy_serve.dag:40` | `concat("$", "BMCWEB_SESSION_TOKEN")` — a var-*reference name* used as a data token, not an emitted program. Borderline; resolve to atom (`"$BMCWEB_SESSION_TOKEN"` is one shell word) — the grammar classifies it as a single parameter-expansion word, i.e. an atom, so **clean**. |

The §5.3 rows are the discriminating negative controls: a correct classifier is **RED on §5.1/§5.2 and GREEN on §5.3**. A classifier that flags §5.3's grammar rows would be flagging the authority — the tell of a marker heuristic, not a grammar classifier.

## 6. Alignment with `StandingIntent` (enforcement-intent)

This wall is one `StandingIntent` row (do **not** re-implement enforcement-intent-design here — align shapes only):

- **property:** `no-smuggled-programs` (a new `LensIdV0`/property the two mechanisms answer).
- **desired_scope:** whole corpus (default) — Face A over the emit modules, Face B over the substrate scan roots (`medium_structure_containment`'s existing `medium_emit_side_scan_roots`).
- **mechanisms (the `LensContract`s that answer the intent):** HALF A = construction (`WallNow` once the carrier lands; the `ConstructionJustification.authority` = `TargetText`'s introduction/eliminator surface); HALF B = the generalized `medium_structure_containment` lens (`AuditOnly` until the census-clean receipt, then `Blocking`).
- **coverage receipt:** §5's census, in executable form — the lens's counted dispositions (`Violation` / `Clean` / `AmbiguousParse`) over the scan roots, with the roster-growth ratchet. **Under-scope is a failing receipt** unless a typed `NarrowingReason` justifies it (e.g. `ExternalRuntimeOnly` for a language with no rows yet → the `DecodeFidelity` honesty boundary, §4).
- **self-application (§7 fractal):** the lens's own `concat`-based key builders (like `duplicate_computation`'s) are on the roster by model walk — the wall does not exempt itself, it rosters its own data-not-authority sites explicitly.

## 7. Sequencing + the sign gate

1. **This PR — design doc + corpus census + HALF A Slice A0.** Post-sign (§ header). Slice A0 = the Q3 row-derived fold and the surgical-7 orchestration→bash productions (`bash_stmt_if` + `bash_orch_if`'s six: `stmt_if_else`, `test_str_eq`, `test_str_empty`, `test_str_nonempty`, `log_matches_grep`, `stmt_or_ignore_fail`). Each production's hand-concat `*_source_text` pyramid is **deleted in the same motion** as its migration; unmigrated productions keep their pyramids as honest staged debt until their slice. The `*_target_model` fns and `OrchestrationEmitMedium.realize_if`/`realize_if_else` now thread `Outcome` (§5: a malformed row **refuses**, never fabricates). Byte-identity proven by the existing `orchestration_bash_test` witnesses (`source.carried == "if [ -z \"$1\" ]; then verdict=absent; fi"` etc.) staying green — emit re-derives from the row tokens, so output is unchanged.
   - **Single-authority consolidation (not a new fork).** The row-derived fold `bound_tokens_source_text` (and its helper `bound_spelling_from_map`) land in `std` as **the** tokens→source-text authority; the pre-existing compiler duplicates `serialize_concrete_syntax_tokens_to_source_string` and `target_binding_spelling_lookup` (`06_translate`/`target_model`) are **refactored to delegate** to them. Net concept count **decreases** — this dissolves an existing §2/§3 fork rather than widening one.
2. **PR 2 — HALF B lens** (grammar-classifier swap + `AmbiguousParse` bucket + census receipt), `AuditOnly` → census-clean → `Blocking`.
3. **PR 3+ — HALF A Slice A1** (`06_translate` serialize-family → `TargetText` carrier), its own load-bearing sign checkpoint. Slice A0 only re-points the existing token fold at the std authority; the carrier-type migration of the serialize family stays here.

**Coordination:** P4-G3 concurrently dissolves the `fleet_converge_emit` splice instance (§3.3) — no file race; the census cites it as the motivating specimen.

## 8. Open questions (for sign)

- **Q1 — carrier form:** phantom `<Lang>` param vs realized `lang` field (§3.1). Lean realized; confirm the .dag type machinery / dispatch trade-off.
- **Q2 — carrier name:** `TargetText<Lang>` vs `EmittedSource<Lang>` vs extend `Medium` nominally. Lean `TargetText<Lang>` in `07_target_carriers.dag`.
- **Q3 — in-language hand-concat (§5.1 row 7):** is a `concat` *inside* `bash.dag` a splice, or acceptable language-internal code? This doc treats it as a splice (joins belong to rows emitted through the fold); confirm — it decides whether `bash.dag:1916`/`bash_orch_if.dag:42` are Slice A0 or permanently roster.
- **Q4 — Face-B disposition:** for a substrate composition literal that is genuinely bootstrap-window bash (shell-emission-model's two sanctioned windows), is HALF B a *violation* (must become intent) or a *rostered exception* (permanent foreign-media/bootstrap framing)? Cross-reference `shell-emission-model`'s pre-runtime census; likely `ubuntu_install_media_*` is RUNTIME-PRESENT → intent-migration, not roster.

## Dissolution trigger (DESIGN §6)

Delete this doc when: (a) the emit seam threads `TargetText<Lang>` end-to-end with the two eliminators as the only exits and `concat`-of-emitted-content fails to typecheck across the orchestration and compiler serialize paths; (b) `medium_structure_containment` classifies by grammar-parse (marker heuristic deleted) with a counted `AmbiguousParse` bucket and a census-clean receipt; and (c) the `no-smuggled-programs` `StandingIntent` gate proves both mechanisms from receipts, `Blocking`, whole-corpus.
