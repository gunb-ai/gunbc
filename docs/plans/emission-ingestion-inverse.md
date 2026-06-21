# Emission = Ingestion⁻¹ — extending the row-driven inverse past syntax to the intent layer

> Plan doc for two roadmap items: the **§0 realization-vocabulary containment guard** (stability-window
> construction wall) and the **§6 emission=ingestion⁻¹-past-syntax** intent layer (expansion). Both come
> from one audit, triggered by an anemic CI helper. DESIGN refs: §3 (single authority; transport vs
> intent), §4 (`emit = serialize_target ∘ translate`, one grammar read both directions; N+M not N×M),
> §5 (fail-closed; construction over validation), §6 (idea→idea; Medium<R>), §7 (round-trip fixed point).

## 1. Trigger: an anemic CI helper

`dsl/tools/build_step.dag`'s `build_verify_echo` hand-authors bash (`echo … >&2`) **and** the
GitHub-Actions `::error::` annotation prefix in one function. It fuses three separate medium
authorities — the print *verb* (`echo`, bash), the *stream* (`>&2`, bash), and the *annotation format*
(`::error::`, GitHub Actions, a medium layered on bash). The intent is simply *"emit an Error-severity
diagnostic."* Because the medium is welded in, the code **cannot re-emit to Rust** (no stdout→stderr
redirect analog; Rust uses `eprintln!`). It is medium-agnostic intent expressed in a target AST.

## 2. The audit: a bash-only bypass of a half-connected stack

Sweep of the corpus found `.dag` consumers that import the bash-AST sidecar
`extdeps.languages.bash.program` (`ShellStmt`/`ShellWord`/`serialize_bash`) to express portable intent.
**The live grep is the authority for this roster — 10 importers at authoring time, shrinking to 0 as
the bash-sidecar arc migrates them.** A frozen count would rot (the roster is *derived*, not a
hand-maintained list — the single-authority point):

- `dsl/tools/`: `build_step`, `host_prelude`, `dsl_compile_clean_transport`, `emit_host_transport`,
  `layering_imports_transport`, `resolved_imports_transport`
- `dsl/gunbc/`: `ci_yaml_validate`, `ci_spec` (the #5432 build-verification wiring, landed after the
  first pre-merge grep — exactly the rot a frozen count invites)
- `src/v2/workflow/`: `compiler_closure_ingest_transport`, `source_root_ingest_transport`

(Two `*_test` importers — `bash_serializer_witness_test`, `build_artifact_verification_witness_test` —
are intentionally **not** walled: the guard scans consumer-source roots only, so test harnesses that
exercise the serializer are excluded by construction, not a roster gap.)

The intent primitives **already exist** but are not wired to a realization path, so consumers reach past
them and hand-author shell:

| Intent (exists) | Where | Its realization (exists, **unconnected**) |
| --- | --- | --- |
| `Diagnostic{reason, Locus, correction}` + `Severity` + `Outcome<T>` (monoid, fail-closed) | `src/v2/std/diagnostic.dag` | `LogAnnotation` (`::error::`/`::warning::`) — `dsl/extdeps/github/log_annotations.dag` |
| `EffectShape` (idempotency by construction) | `dsl/std/effects.dag` | — |
| filesystem predicates (exists/mtime/kind) | `dsl/std/filesystem.dag` (+ posix/ntfs) | `find -newer` / `[ -x ]` (hand-typed) |
| `ProcessExit` | `dsl/std/process.dag` | cli_run.rs host tap |

The single missing seam: the medium-agnostic `Diagnostic` and its GHA realization `log_annotations`
both exist, but **no `emit(Diagnostic, target)` row connects them.** `render_log_annotation` is a
*hand-rolled forward emitter* — the exact thing the language layer abolished (see §3).

**Cross-language conclusion:** `grep` for `serialize_rust|serialize_go|RustProgram|GoProgram` in
consumers is empty. Every Rust/Go/Kotlin/TS use is a legitimate language-model / grammar-claim /
emit-fold / fixture. The anemia is **bash-only**, for a structural reason: bash is the only language we
use for *orchestration*; the others are only ever single emit *targets*. So it will not spread to
hand-authored Rust — it will keep forcing *more bash* until orchestration is modeled as intent (gap B).

## 3. The architecture: emission = ingestion⁻¹

`emit = serialize_target ∘ translate` is **already the realized architecture for language emit.**
`src/v2/compiler/06_translate.dag` does grammar-inverse serialize over the **same** `v2.std.grammar`
`FormalProduction` / `derive_grammar_relation_row_node` rows that ingest folds *forward*:

```
ingest:  source ──tokenize→parse→normalize──▶ Node      (select production forward)
emit:    Node   ──translate→serialize_target──▶ source   (select the SAME row backward)
```

One authority (the grammar rows), read in two directions ⇒ a new target is *rows*, never a new emitter
(N+M, not N×M). Realized for rust/python/go/cpp/typescript. `v2.extdeps.languages.bash` is migrating
bash onto this (slices 1–5d landed), dissolving the hand-rolled `serialize_bash` sidecar.

Three-state picture in these terms:

| Layer | emit = ingest⁻¹? | Evidence |
| --- | --- | --- |
| **Languages** (syntax) | ✅ realized | `06_translate` grammar-inverse over shared rows |
| **Bash** | 🔵 in progress | `v2.extdeps.languages.bash` slices 1–5d; `program.dag` forward-emitter still alive |
| **Intent / effect / orchestration** | ❌ absent | no ingest of "a diagnostic"/"a pipeline" → no Node → nothing to invert → consumers hand-author shell |

**Honesty boundary:** emit = ingest⁻¹ is an *exact* inverse only where ingest is `Lossless` (the
`DecodeFidelity` boundary, §4/§7). For lossy media (English catch-all token, dropped comments on `.dag`
round-trip) emit is a *section*: `ingest ∘ emit = id` on the canonical core but `emit ∘ ingest ≠ id`.
The architecture declares this **per medium** via `DecodeFidelity` / `Medium<R>` — never pretending
every medium round-trips. The correctness oracle is the round-trip law itself (§4/§7 fixed point).

## 4. The two gaps (the §6 item)

Extending emission=ingestion⁻¹ past syntax to the intent layer:

- **(A) diagnostic-realization rows** — `Diagnostic{Severity}` (`src/v2/std/diagnostic.dag`) →
  `{Bash: echo>&2, GitHubActions: ::error::, Rust: eprintln}` as **rows**, dissolving the hand-rolled
  `render_log_annotation` forward emitter. Filesystem predicates de-fuse the same way
  (`std/filesystem` → `{Bash: test/find, Rust: fs::metadata}`).
- **(B) orchestration-as-intent** — a `Pipeline`/`Step`/`Run`/`Check` vocabulary so transports author
  *intent* and `emit(intent, Bash)` renders shell. Today shell is **both** the surface *and* the only
  target for orchestration — this is the deeper gap and the reason the anemia is bash-only.

**Cross-arc edges (first-class, not prose):**
- enabler: the **bash-sidecar dissolution arc** (warm-badger-46) — it dissolves `program.dag`, the
  forward-emitter these rows replace.
- enforced-not-eroded by the **§0 containment guard** (§5 below): the guard forbids re-introducing a
  forward emitter, so the intent layer is built *under* a wall.

## 5. The §0 containment guard (stability-window construction wall)

> The only legitimate *producers* of target-language AST are (1) the `extdeps/languages/**` models and
> (2) the emit fold (`05_emit`/`06_translate`/`candidate_generation`). Everything else authors intent.

**Rule:** a module may import the target-AST construction vocabulary
(`extdeps.languages.bash.program` → `ShellStmt`/`serialize_bash`, + any future per-language AST sidecar)
**iff** it is in the realization-edge allow-set. Any other importer is a `RealizationVocabularyLeak`.

**Mechanism (N+M, not a new lens):** a sibling rule over `v2.lens.layering_imports`'s existing
host-enumerated `LayerImportFact{layer, path, import_module}` rows. Predicate:
`import_module ∈ target_ast_vocab_modules ∧ path ∉ realization_edge_allowset`.

**Sequencing (honest, fail-closed):** the importers in the live-grep roster (10 today) go RED today, so
it ships with them as a **named, shrinking exception roster** = a ratchet, not an instant wall. **Dissolve-on:** the
bash-sidecar arc migrates each consumer to `emit(intent, Bash)` ⇒ roster empties ⇒ guard flips to a
pure wall ⇒ `program.dag` is deletable. Discriminating witness: a fresh non-edge module importing
`ShellStmt` goes RED; an edge module does not. This is what makes `shell(intent())` a realization-edge
feature, never authored inside consumer code. **Ties to §6** (the item it protects).

## 6. Independent §3-hygiene cleanup (not a roadmap item)

Found in the same sweep, fixable now with existing authority (dispatched separately): `lit(text: "dsl")`
hardcoded as a policy literal in `compiler_closure_ingest_transport` (×3) + `source_root_ingest_transport`
(×1) — should fold `witness_layer_roots`, the way `layering_imports_transport`'s `source_root_flags()`
already does. A §3 policy-leak (an argv carrying a literal it should receive as a parameter).
