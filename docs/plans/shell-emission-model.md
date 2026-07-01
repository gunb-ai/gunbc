# Shell emission model — orchestration intent to bash via grammar rows

> Formal plan for shell as a **first-class emission target**: medium-agnostic orchestration intent (`std/orchestration`) renders to bash through grammar-inverse rows at the realization edge — not through the deletable `program.dag` forward emitter. Realizes [emission-ingestion-inverse.md](emission-ingestion-inverse.md) gap **(B)**; design proposal: [orchestration-as-intent-design.md](orchestration-as-intent-design.md). DESIGN refs: §2 (one concept — intent vs transport vs spelling), §3 (intent central / dispatch+rows peripheral; `program.dag` is a nickname scaffold), §4 (`emit = serialize_target ∘ translate`, N+M), §5 (fail-closed; construction over validation), §6 (priced in displaced cost).

**Status:** planning tracker · **`.dag` carrier is authority**. Linked from `ROADMAP.md` §6. Complementary to [regime-2 shared emission fold](regime2-shared-emission-fold.md) (config/doc *projection*) and [format-model-reconciliation](format-model-reconciliation.md) (record spelling) — those own non-shell text; **this doc owns orchestration → shell**.

## 1. The gap

Shell is the only language used for *orchestration* (CI steps, hooks, fleet converge) but was never a modeled emission target — consumers hand-author bash as `concat` trees or import the `program.dag` `ShellStmt` sidecar directly (11 importers on a shrinking frozen roster per the §0 containment guard). Displaced cost: every new control-flow pattern is another bespoke string; the anemia cannot spread to Rust/Go (they are single-shot emit targets only) but forces *more bash* until intent is modeled.

Historical anchor: ROADMAP #510 named "Shell-emission target (needs design)" when pre-push hooks bypassed the compiler's dependency machinery.

## 2. The model — three layers, one fold direction

```
author intent (std/orchestration)  →  grammar rows (v2.extdeps.languages.bash)
                                      →  shell text (bash_command_fold / 05_emit_orchestration)
```

| layer | home | role |
| --- | --- | --- |
| **intent** | `src/v2/std/orchestration.dag` | medium-agnostic `Run`/`Step`/`Pipeline` coproduct (control flow, retry, predicates) |
| **realization rows** | `src/v2/extdeps/languages/bash.dag` | parameterized grammar productions (`bash_if_*`, `bash_retry_*`, env-prefix, pipe, …) |
| **emit fold** | `src/v2/compiler/05_emit_orchestration.dag` + `bash_command_fold.dag` | `emit(intent, Bash)` = grammar-inverse serialize; **never** extends `program.dag` |

**Discriminator (not a fourth hand-fold):** consumer modules import **no** bash AST. Only the realization edge authors target syntax; the §0 `RealizationVocabularyContainment` guard enforces this with a shrinking roster.

**Honesty boundary:** orchestration shell is emit-primary today (no `ingest(Pipeline)`), but the architecture is grammar-inverse so rows can gain ingest later without a second emitter — same class as bash syntax slices 1–5d.

## 3. Sibling plans (one home each)

- **[emission-ingestion-inverse.md](emission-ingestion-inverse.md)** — parent arc (gap A diagnostics + gap B orchestration + gap C ci.yml shim + §0 containment guard). This doc is the **implementation tracker for gap B's shell half**.
- **[orchestration-as-intent-design.md](orchestration-as-intent-design.md)** — sign-ready **design** (vocabulary grounding, tier-1 vs tier-2, worked consumers `ci_cargo_eagain_retry_core` + `fleet_converge_emit`). This doc does not restate it; it tracks landings against it.
- **[regime-2 shared emission fold](regime2-shared-emission-fold.md)** — **different regime**: forward-only `Doc` layout projection for config artifacts (yaml, gitignore, runner manifest). Not orchestration shell.
- **[host-effect-orchestration.md](host-effect-orchestration.md)** — **execution** interface (`apply(target, effect, policy)`). Shell emission produces the artifacts that `EmitArtifactThenThinRun` executes; compose, don't merge.

## 4. Landed (execution-grounded)

- **`std/orchestration.dag`** — `Run`, `Step`/`Do`/`Retry`, `Predicate`, `EnvBinding`, `Pipeline` (tier-1 vocabulary per design doc).
- **`05_emit_orchestration.dag`** — `orch_emit_pipeline` / `orch_emit_run` / `orch_emit_step`:
  - env-**free** `Run` → delegates to `bash_fold_raw_line_target_model` + `bash_word_pass_emitted` (byte-identical witness: `orch_run_empty_env_delegates_to_shell_emit_holds`).
  - `Retry` with two `LogMatches` escalation levels → unrolled bash (witnessed in `orchestration_retry_emit_test.dag`).
  - `If`/`For`/`While` → **fail-closed `Rejected`** (not silently omitted).
- **`bash_command_fold.dag`** — grammar-inverse fold over `ShellStmt` productions; fold==serialize witnesses green.
- **Gap A partial** — `EmitDirective` bash rows landed (#5505); orchestration `Retry` warnings compose on them.

## 5. Sequencing

1. **Bash AST gaps (modeling)** — `EnvUnset` + multi-binding `FreeMonoid<EnvBinding>` nesting in bash grammar (dissolves `orch_emit_run_env_welded` single-string render). Tracked: gunbc#5846.
2. **Parameterized control-flow productions** — lift fixed-literal `bash_if_*` / `for` / `while` rows to accept `BoundToken` child emissions per [orchestration-as-intent-design.md](orchestration-as-intent-design.md) §4.2; wire `orch_emit_step` `If`/`For`/`While` arms.
3. **Consumer migration** — each of the 11 `program.dag` importers rewrites to `Pipeline` intent + `orch_emit_*`; roster entry deleted per migration. Order: `dsl/tools/*` transports → `ci_spec` retry core → `fleet_converge_emit` (hardest).
4. **Roster empties → pure wall** — containment guard flips ratchet→wall; delete `extdeps/languages/bash/program.dag` forward emitter.
5. **ci.yml shim (gap C)** — held until keystone: bootstrap + EAGAIN retry cascade emitted from intent into thin GHA YAML ([emission-ingestion-inverse.md](emission-ingestion-inverse.md) §4(C)).

## 6. Open / boundaries

- Tier-2 residue (`fleet_converge_emit` functions, arithmetic) stays named second-tier per design doc — not a reason to widen tier-1 or touch `program.dag`.
- `Retry` lowering is **unroll** (heterogeneous escalation), not uniform `for seq` — modeling decision locked in design doc §4.2.
- Pre-push / commit-pipeline hooks are consumers of this model once keystone consumers land; not a separate emitter fork.

## Dissolution trigger (DESIGN §6)

Delete this doc when orchestration intent emits exclusively through grammar rows (every `program.dag` importer migrated, the §0 containment roster empty, `program.dag` deleted), `If`/`For`/`While`/`Retry`/`Run` coverage matches the tier-1 vocabulary witnessed byte-identically on `ci_cargo_eagain_retry_core` and the majority tier-1 surface of `fleet_converge_emit`, and gap C's ci.yml embedded-shell count has collapsed to GHA-native structure only — at which point shell emission is a witnessed property of the substrate, not a tracked arc.
