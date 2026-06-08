# Thin-Shim CI — Phase A3 Run-All: Feasibility / Blocker Report

**Status:** blocker/design report (**docs only** — no `ci.dag`, `ci.yml`, or ratchet edits)
**Parent plan:** ctrl#1490 (D3 subordinate); design note #4527 (`docs/planning/thin-shim-ci-dag-dissolution-design-2026-06-08.md`)
**Lane:** A3 — implement `gunbc-ci` run-all (load CI model, run every job, real exit code) — *first runtime consumption of `ci.dag`*

---

## 1. Verdict

**A3 run-all cannot be implemented mechanically today.** The design note (#4527) lists A3 as
"ready now," but inspection of the actual model and runner shows two independent gaps, **either of
which alone blocks a real run-all**. The dispositive one (B2) requires a model-authority change that
is explicitly **Mgr-C-gated** and outside this lane (no `ci.dag` edits without a cut-list).

Per the brief, the A3 deliverable is therefore this **precise blocker/design report**, plus a small
honest diagnostic improvement to the runner stub that names the missing substrates — *not* a
fabricated run-all that mirrors `ci.yml` step text in Rust.

---

## 2. What was inspected

| Artifact | Finding |
|----------|---------|
| `src/v3/compiler/src/bin/gunbc_ci.rs` | `--workflow ci --event …` path is a fail-closed stub (exit 2 unless `GUNBC_CI_ALLOW_DISPATCH_STUB=1`). Uses frozen-v3 **single-source** `compile_to_dag`. |
| `src/v4/workflow/ci.dag` (6,337 lines) | `CiPipeline { jobs, gates }`, `CiJob { id, command, needs }`, `CiCommand` (abstract tags). 28 `import`s. Commands project only to **cache-digest `Node`s** (`ci_command_projection_node`), never to runnable shell. |
| Whole model | Exactly **one** command string exists: `m1_rust_dag_emit_parity_receipt_test` (a parity-receipt comparison string), not a per-job dispatch table. |
| `.github/workflows/ci.yml` + `.github/ci-floor/*.sh` | The **actual** shell commands (`cargo build -p v2-compiler --release`, probe scripts, …) live here, keyed off the abstract model only by hand. |
| `tools/ci_affected_components/src/lib.rs` | A **structural Rust mirror** of `ci.dag`'s selection predicates (`//! Structural mirror of …`). Already a dual-representation the project is dissolving — extending it is forbidden. |
| `tools/ci_workflow_ratchet/tests/v4_workflow_ci_runner_dag_smoke_test.rs` | Header: *"full `compile_to_dag` import merge **deferred** until cross-module v4 load lands (same posture as peer v4 smoke tests)."* The harness only **tokenizes/parses** `ci.dag`. |
| `src/v2/stage0/src/cli_run.rs` | The real v4 runtime loader is the **v2** `dag run` path (`compile_to_resolved` over a transitive import closure; real `ProcessExit`/`Bool` exit-code contract). The `v3-compiler` crate that hosts `gunbc-ci` does **not** link it. |

---

## 3. The two blockers

### B1 — Runtime-loader gap (no cross-module v4 load in the runner)

`gunbc-ci` lives in the `v3-compiler` crate and reaches only frozen-v3 `compile_to_dag(source, file)`
— **single source**. `ci.dag` is a v4 module with 28 transitive imports. Cross-module v4 load is
explicitly **deferred** (the runner smoke and all peer v4 smokes only tokenize/parse). The real v4
loader is the **v2** multi-source path (`dag run` → `compile_to_resolved`), which this crate does not
link, and wiring v2 into the frozen v3 crate is itself a load-bearing change (cf. "don't build
durable get-off-v3 infra out of v3 parts"). `compile_to_dag_modules_in_order` exists but is not the
sanctioned v4 load path and is not proven on this closure.

### B2 — Execution-authority gap (dispositive)

Even with a loader, **the model carries no per-job runnable command.** `CiCommand` arms
(`V2BootstrapCompileCommand`, `LensCiCommand`, `TestCommand`, …) are abstract tags whose only
projection is a cache-digest `Node`. The shell to run each job lives in `ci.yml` / `ci-floor/*.sh`.
Two consequences:

1. The `.dag` interpreter is **pure** — it cannot shell out to run `cargo`/`bash`. "Run every job"
   is inherently a **host** action, not an in-model one.
2. A host run-all therefore needs a `CiCommand → shell command` table. That table **does not exist
   in the model** and authoring it in Rust would be a **banned dual-authority** — a second copy of
   `ci.yml` step text (white-box / "2FA for code"), and exactly the "no reimplementing selection in
   Rust beyond a dumb path" line the brief draws.

B2 holds regardless of B1, so it is the controlling blocker.

---

## 4. What *would* unblock A3 (Mgr-C / operator gated — not this lane)

1. **Model command-authority (Phase A4 model change).** Give `CiCommand` a runnable-command
   projection so the runner dispatches **from the model**, not a Rust table. The lone existing
   string (`m1_rust_dag_emit_parity_receipt_test`) shows the intended shape: a modeled command
   string (or `ShellScript` path, like the M1 probe's `host_script`) per command arm. This is a
   `ci.dag` edit → requires a Mgr-C cut-list. It is also the natural home for A4's "delete duplicate
   YAML mirror text," since once the command lives in the model, the `ci.yml` copy is the duplicate.
2. **Cross-module v4 runtime loader** reachable by the runner (the v2 `dag run` closure loader, or
   its sanctioned successor) so `gunbc-ci` can load `ci.dag` + its 28-import closure into an
   executable graph. This is substrate work, not a YAML/CI edit.

With both in place, run-all becomes mechanical: load model → enumerate `CiPipeline.jobs` →
topologically order by `needs` → for each job, restore cache (per-op digest already modeled) → run
the **modeled** command → save cache → real exit code. No Rust mirror of YAML.

**Ordering note (confirms A1, warm-ibex):** do **not** thin/rewire `ci.yml` (A2) before run-all is
real — dispatch is still a stub. A3 is the prerequisite, and A3 is itself gated on the two items
above. Branch-protection job IDs and ratchet drift are separate Mgr-C/operator decisions.

---

## 5. This PR's scope

- This report.
- `gunbc_ci.rs`: the `--workflow` dispatch stub keeps its fail-closed exit-2 contract and the
  `GUNBC_CI_ALLOW_DISPATCH_STUB` smoke escape unchanged; the diagnostic is enriched to name the two
  concrete missing substrates (B1 loader, B2 command-authority) and point at this report, so the next
  implementer hits the real gate, not an opaque "not wired yet."

No `ci.dag`, `ci.yml`, `ci-floor/*`, or ratchet edits.
