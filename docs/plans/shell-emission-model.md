# Shell emission model — orchestration intent to bash via emit(intent, Bash)

> Operator-signed design (multi-agent audit, verified against live tree). Shell emission lane: medium-agnostic orchestration intent → `emit(intent, Bash)` via the existing `05_emit_orchestration` dispatcher — **not** a new `ShellProgram` AST. DESIGN refs: §2 (intent vs transport vs spelling), §3 (single authority; `program.dag` rostered for dissolution), §4 (`emit = serialize_target ∘ translate` — endgame Q3), §5 (emit-only faithfulness; green by execution vs frozen bytes), §6 (purity-trap fence: no construct without a named live site). Complements [regime-2 shared emission fold](regime2-shared-emission-fold.md) (`std.layout.Doc` = **layout only**, never shell content) and [format-model-reconciliation](format-model-reconciliation.md) (record spelling). Parent arc: [emission-ingestion-inverse.md](emission-ingestion-inverse.md) gap (B).

**Status:** planning tracker · **`.dag` carrier is authority** (§6). Linked from `ROADMAP.md` §6. **This PR is docs/authority only** — no emitter/serializer code.

## 1. The finding (reframes the frontier)

Shell is emitted as **raw strings** everywhere — even `host_effect.dag` carries `ShellCommand{script: String}`. But the arc is **~20% built**, not greenfield:

- **Intent coproduct exists** — `src/v2/std/orchestration.dag`: `Run` / `Step{Do,If,For,While,Retry}` / `Pipeline` / `Predicate`.
- **Bidirectional bash language exists** — `src/v2/extdeps/languages/bash.dag` (~2201 lines, POSIX-cited).
- **Intent→bash lowering exists but is bespoke** — `src/v2/compiler/05_emit_orchestration.dag` is a per-construct dispatcher, **not** the same `target_model_edge_translation_rules` table that emits Rust/TS (`06_translate.dag` has zero orchestration refs).
- **Control-flow emission is the greenfield work** — `If`/`For`/`While` all return `outcome_rejected` today; only `Do{Run}` + a hardcoded 2-level `Retry` lower. `Run.command:String` is the **same anemic leaf** as `host_effect.ShellCommand.script`.

## 2. The model (locked)

**ADOPT:** `intent(std)` → `bash(extdeps)` via `emit(intent, Bash)`, extending the existing dispatcher.

**REJECT:** a new `ShellProgram` AST — `dsl/extdeps/languages/bash/program.dag` is already rostered for dissolution ([emission-ingestion-inverse.md](emission-ingestion-inverse.md) §2).

**CUT:** host-op vocab (`EnsurePackage`/`EnableService`/`ServePort`) — single consumer (`live_deploy`) + idempotency is already a `host_effect.Policy` fact (`OneShotIdempotent`). Desugar `live_deploy` verbs **inline** to `If{Not{ExitZero{…}}, Do{…}}` intent; no minted verb.

**Binding purity-trap fence (§6):** no construct (grammar arm / intent variant / desugar) added without a **named live site** that emits it; do not grow ingest-direction bash gates to serve emit-only coverage.

**`std.layout.Doc` stays for LAYOUT only** (line/indent/heredoc-body framing), never for shell content — [regime-2](regime2-shared-emission-fold.md) owns that half.

```
std/orchestration (intent)  →  05_emit_orchestration (dispatcher)
                              →  v2.extdeps.languages.bash (rows)
                              →  shell text
```

## 3. Effects are first-class (load-bearing)

**host_effect Phase B:** the `ShellCommand{script:String}` payload dissolves onto **modeled orchestration intent** (a `Pipeline`), **NOT** onto the doomed `program.dag`+`serialize_bash` sidecar. The existing [host-effect-orchestration.md](host-effect-orchestration.md) Phase-B text pointing at `program.dag` **predates** `emit(intent,Bash)` and is **superseded** by this plan.

Load-bearing: `host_effect.dag` is a DESIGN-named seam. **Gated** on srv3 `OsInstalled` + Receipt-lock (#5725) milestones — record direction here; execution sequences behind those gates.

## 4. Faithfulness boundary (§5)

Shell-orchestration sites are **emit-only** (regime-2 class): no round-trip oracle. Faithfulness = **byte-identity vs the current committed emitted output** + a discriminating one-byte-perturbation RED tooth.

**Critical honesty — `live_deploy`:** has **no committed artifact**. Its "drift gate" (`dsl/test/claim/live_deploy/emit_test.dag:123-128`) compares `expected_live_deploy_apply_script()` to **itself** (the emit fn) = a self-referential fabricated-green trap (#6023 disease). `live_deploy` must **freeze** its current output as a committed golden literal **before** it is provable.

**Committed real goldens that exist:** `.github/workflows/ci.yml`, `.github/fleet-converge.sh`.

**Done bar:** green by **execution** vs frozen bytes, RED on perturb — never typechecks/emits/self-referential-gate.

## 5. Slice sequence (each gated by a frozen committed byte oracle)

1. **Slice 0 — CI EAGAIN-retry cutover:** route `dsl/gunbc/ci_spec.dag` `ci_cargo_eagain_retry_core` (nested-concat blob, :68) through `render(emit(Retry,Bash))`; proven by committed `ci.yml` drift-gate staying byte-identical + existing teeth witness `orch_retry_env_value_has_teeth_holds`. Env is welded (`orch_emit_run_env_welded`, dissolves gunbc#5846) but byte-exact.
2. **Slice 1 — control-flow emission (greenfield):** `If` first (new `orch_emit_step::If` arm + parameterized `orch_if_target_model` bridge + bash round-trip fixture + byte golden), then `For`/`While`. The real greenfield work + the §4 payoff for control flow.
3. **Slice 2 — fleet_converge:** `fleet_converge_emit.dag` (~21KB) → committed golden `.github/fleet-converge.sh` — the biggest displaced cost.
4. **Slice 3 — live_deploy:** ONLY after Q1 execution-gates clear **and** after freezing output as a committed golden. Heredoc foreign-media bodies (JS server, systemd unit) stay opaque `FailClosed` — a **permanent ratchet**, not migration debt.
5. **Slice 4 — tail consumers:** `bmc_token_federation` → `ci_workflow` inline `RunStep`s (case/`uname` → model as `TargetArchitecture`; cross-link ROADMAP §1 `1-inline-shell-defork`) → githooks.

## 6. Sidecar dissolution (parallel)

`dsl/extdeps/languages/bash/program.dag` (`ShellProgram`/`serialize_bash`) has **11 live importers** (`ci_spec`, `ci_yaml_validate`, `local_tidy_spec`, `dsl/tools/{build_step,dsl_compile_clean_transport,emit_determinism_transport,emit_host_transport,extdeps_external_authority_transport,layering_imports_transport}`, + 2 test witnesses). Migrate all onto the v2 bidirectional bash language, then delete `program.dag`+`serialize_bash`.

Tracked in [emission-ingestion-inverse.md](emission-ingestion-inverse.md) / `emission_ingestion_inverse.dag` ("11 importers shrinking to 0") — **cross-link only, do not duplicate** the roster here.

## 7. Named residue / dissolution triggers

- **(a) Dispatcher endgame (OPEN Q3):** dissolve `05_emit_orchestration`'s hand-dispatch into `06_translate`'s rule table — the §4 endgame that makes "a new construct is a row" TRUE.
- **(b) `live_deploy_emit_shell_dissolution_trigger`:** retires only at host_effect Phase B (escalated).
- **(c) Roster honesty:** `medium_structure_exception_roster` entries for `live_deploy`/`githooks` **never fully retire** while foreign-media heredoc bodies stay opaque — split "migratable raw control flow" (eventual wall) vs "permanent foreign-media framing" (permanent ratchet); check rosters by **model walk**, not grep.

## 8. Open questions (unresolved)

- **Q3:** commit to dissolving `05_emit_orchestration` dispatch into `06_translate` rules, or accept per-construct bridges as steady state?
- **Q4:** cross-tree — `dsl/extdeps/shell/shell.dag` AND `src/v2/extdeps/shell.dag` both declare module `extdeps.shell` (flat-intern collision), and `bash_command_fold.dag:3` imports the dsl `program.dag` — resolve via `--dependency-pool-index` precedence now, or fold into the dsl/v2 consolidation thread?

## Dissolution trigger (DESIGN §6)

Delete this doc when all shell-emit sites route through emit(intent,Bash), the sidecar program.dag is deleted, and host_effect Phase B has dissolved ShellCommand{script} onto modeled orchestration intent.
