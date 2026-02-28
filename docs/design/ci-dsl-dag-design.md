# CI DSL DAG Design

> Design for CG-1 through CG-5: migrating CI YAML generation from Rust string
> concatenation to DSL-first modeling. Follows the makegen pattern: extern
> discovery → pure DSL rendering → `content_upsert`.

## Status

Draft — 2026-02-28

## Problem

CI YAML generation lives in ~120 lines of hand-wired `push_str`/`write!` string
concatenation in `codegen_cli.rs:503-609`, plus ~60 lines of structural
validation (`validate_github_actions_template`, `validate_gitlab_ci_template`).
The Rust substrate also carries a parallel `RenderConfig` builder, `SharedStep`
enum, and `yaml_block` utility in `core/ir/src/transport/ci/render.rs` (~510
lines), along with a `WorkflowSpec`→DAG bridge in
`gunbc-dag/src/makegen/ci_render.rs` (~190 lines).

This is the same pattern the makegen migration eliminated: domain types are
implicit in Rust code, rendering is procedural string assembly, and validation
reimplements structural invariants that the type system could enforce. The DSL
already has proven infrastructure for this exact shape (see `tools/makegen.dag`).

## Goals

1. **Model CI YAML structure as DSL types** — `CiWorkflow`, `CiJob`, `CiStep`,
   `CiTrigger`, `CiPermission`, `CiCache`, `CiEnv`, plus provider sum type
   `CiProvider = GitHub | GitLab`.
2. **Pure rendering functions** — `render_github_workflow`, `render_gitlab_workflow`
   with small composable helpers. No Rust string assembly for CI YAML.
3. **Single entrypoint tool** — `func cigen() -> { written: Bool }` following
   the makegen pattern.
4. **Delete Rust cigen code** — ~200 lines from `codegen_cli.rs`, dead code from
   `core/ir/src/transport/ci/render.rs`.
5. **Structural validity by construction** — if the DSL types construct, the YAML
   is valid. No separate validation pass.

## Non-Goals

- General YAML serializer (YAML indentation via string literals is sufficient).
- Runtime step-level CI rendering (animated progress) — `CiRenderer` trait stays
  in Rust for now.
- Provider detection (`detect_provider`, `is_ci`) — stays in Rust runtime.

## Design

### File Layout

```
dsl/std/ci.dag            — CG-1: Layer 0 types + data declarations
dsl/std/ci_render.dag     — CG-2: Pure rendering functions
dsl/tools/cigen.dag       — CG-3: Entrypoint tool
```

### CG-1: `dsl/std/ci.dag` — CI Model Types

Layer 0 tautological definitions. "What is a CI workflow?" — answered
structurally, the same way `std/languages.dag` answers "What is Rust?"

#### Type Hierarchy

```
CiProvider (sum)          — GitHub | GitLab
CiWorkflow (record)      — top-level workflow/pipeline
  CiJob (record)          — a job within a workflow
    CiStep (sum)          — Run | Uses | DagRun
  CiTrigger (record)     — push/PR trigger configuration
  CiPermission (record)  — scope + level pair
  CiCache (record)       — key, paths, restore keys
  CiEnv (record)         — key + value pair
  CiSecret (record)      — name + provider reference syntax
```

#### Proposed Types

```dag
module std.ci

// "What CI provider are we targeting?"
type CiProvider = GitHub | GitLab

// "What is a CI trigger?"
type CiTrigger {
  push_branches: List<String>
  pr_branches: List<String>
}

// "What is a CI permission?"
type CiPermission {
  scope: String
  level: String
}

// "What is a CI environment variable?"
type CiEnv {
  key: String
  value: String
}

// "What is a CI secret reference?"
type CiSecret {
  name: String
}

// "What is a CI cache?"
type CiCache {
  key: String
  paths: List<String>
  restore_keys: List<String>
}

// "What is a checkout configuration?"
type CiCheckout {
  fetch_depth: Int?
  submodules: String?
}

// "What is a CI step?"
type CiStep
  = Run { name: String, command: String, env: List<CiEnv> }
  | Uses { name: String, action: String, with_params: Map<String, String> }
  | DagRun { name: String, tool_command: String, env: List<CiEnv>, secrets: List<CiSecret> }

// "What is a CI job?"
type CiJob {
  id: String
  name: String
  runner: String
  timeout_minutes: Int
  steps: List<CiStep>
  needs: List<String>
}

// "What is a CI workflow?"
type CiWorkflow {
  name: String
  provider: CiProvider
  trigger: CiTrigger
  permissions: List<CiPermission>
  env: List<CiEnv>
  cache: CiCache?
  jobs: List<CiJob>
}
```

#### Data Declarations — Shared Configs

Tautological data for this project's CI. These are the "what is our CI?" declarations,
analogous to `config/build_targets.dag` for Make.

```dag
// Rust project cargo cache paths — reusable across providers.
data rust_cargo_cache_paths: List<String> = [
  "~/.cargo/bin/",
  "~/.cargo/registry/index/",
  "~/.cargo/registry/cache/",
  "~/.cargo/git/db/"
]

// Standard cargo env vars for CI.
data ci_cargo_env: List<CiEnv> = [
  { key: "CARGO_TERM_COLOR", value: "always" },
  { key: "RUSTFLAGS", value: "-D warnings" }
]

// Default checkout config.
data default_checkout: CiCheckout = {
  fetch_depth: 1,
  submodules: null
}

// Default trigger: main branch for push and PR.
data default_trigger: CiTrigger = {
  push_branches: ["main"],
  pr_branches: ["main"]
}
```

### CG-2: `dsl/std/ci_render.dag` — Pure Rendering Functions

Follows the makegen pattern: small composable `fn`s, `|> map` + `|> join("\n")`.
YAML indentation via string literals — no general YAML serializer needed.

#### Function Catalog

| Function | Signature | Responsibility |
|----------|-----------|----------------|
| `render_github_workflow` | `(w: CiWorkflow) -> String` | Top-level GitHub Actions YAML |
| `render_gitlab_workflow` | `(w: CiWorkflow) -> String` | Top-level GitLab CI YAML |
| `render_github_trigger` | `(t: CiTrigger) -> String` | `on:` block |
| `render_permissions` | `(perms: List<CiPermission>) -> String` | `permissions:` block |
| `render_env_block` | `(env: List<CiEnv>) -> String` | `env:` / `variables:` block |
| `render_github_cache` | `(cache: CiCache) -> String` | Cache action step |
| `render_github_step` | `(step: CiStep, provider: CiProvider) -> String` | Individual step |
| `render_github_job` | `(job: CiJob, w: CiWorkflow) -> String` | Full job block |
| `render_checkout_step` | `(checkout: CiCheckout) -> String` | Checkout action |
| `render_secret_env` | `(secrets: List<CiSecret>, provider: CiProvider) -> String` | Secret references |
| `render_gitlab_job` | `(job: CiJob) -> String` | GitLab job block |
| `render_gitlab_cache` | `(cache: CiCache) -> String` | GitLab cache block |

#### Key Rendering Patterns

**GitHub Actions — trigger block:**
```dag
fn render_github_trigger(trigger: CiTrigger) -> String {
  let push_lines = trigger.push_branches |> map(b => "      - {b}") |> join("\n")
  let pr_lines = trigger.pr_branches |> map(b => "      - {b}") |> join("\n")
  "on:\n  push:\n    branches:\n{push_lines}\n  pull_request:\n    branches:\n{pr_lines}\n"
}
```

**GitHub Actions — env block (reused pattern):**
```dag
fn render_env_block(env: List<CiEnv>, indent: String) -> String {
  env |> map(e => "{indent}{e.key}: {e.value}") |> join("\n")
}
```

**GitHub Actions — step rendering (match on sum type):**
```dag
fn render_github_step(step: CiStep) -> String {
  match step {
    Run(s) => {
      let env_block = if s.env |> count() > 0 {
        "\n        env:\n" + render_env_block(env: s.env, indent: "          ")
      } else { "" }
      "      - name: {s.name}\n        run: {s.command}{env_block}"
    }
    Uses(s) => {
      let with_block = if s.with_params |> count() > 0 {
        let params = s.with_params |> map((k, v) => "          {k}: {v}") |> join("\n")
        "\n        with:\n{params}"
      } else { "" }
      "      - name: {s.name}\n        uses: {s.action}{with_block}"
    }
    DagRun(s) => {
      let base_env = s.env |> map(e => { key: e.key, value: e.value })
      let secret_env = s.secrets |> map(sec => { key: sec.name, value: "${{{{ secrets.{sec.name} }}}}" })
      let all_env = base_env |> append(items: secret_env)
      let env_block = if all_env |> count() > 0 {
        "\n        env:\n" + render_env_block(env: all_env, indent: "          ")
      } else { "" }
      "      - name: {s.name}\n        run: {s.tool_command}{env_block}"
    }
  }
}
```

**GitLab CI — job rendering:**
```dag
fn render_gitlab_job(job: CiJob) -> String {
  let needs_block = if job.needs |> count() > 0 {
    let needs_lines = job.needs |> map(n => "    - {n}") |> join("\n")
    "\n  needs:\n{needs_lines}"
  } else { "" }
  let script_lines = job.steps |> map(s => match s {
    Run(r) => "    - {r.command}"
    DagRun(d) => "    - {d.tool_command}"
    _ => ""
  }) |> filter(l => l != "") |> join("\n")
  "{job.id}:\n  stage: ci{needs_block}\n  script:\n{script_lines}\n"
}
```

#### Header Rendering

```dag
fn render_ci_header(generator: String, regen_command: String) -> String {
  "# Generated by {generator}\n# DO NOT EDIT - regenerate with: {regen_command}\n"
}
```

#### Top-Level Assembly

**GitHub Actions:**
```dag
fn render_github_workflow(w: CiWorkflow) -> String {
  let header = render_ci_header(
    generator: "gunbc-codegen",
    regen_command: "cargo run -p gunbc-dag --bin gunbc-codegen -- cigen"
  )
  let trigger = render_github_trigger(trigger: w.trigger)
  let perms = render_permissions(perms: w.permissions)
  let env = "env:\n" + render_env_block(env: w.env, indent: "  ") + "\n"
  let jobs = w.jobs |> map(j => render_github_job(job: j, workflow: w)) |> join("\n")
  "{header}\nname: {w.name}\n\n{trigger}\n{perms}\n{env}\njobs:\n{jobs}"
}
```

**GitLab CI:**
```dag
fn render_gitlab_workflow(w: CiWorkflow) -> String {
  let header = render_ci_header(
    generator: "gunbc-codegen",
    regen_command: "cargo run -p gunbc-dag --bin gunbc-codegen -- cigen"
  )
  let vars = "variables:\n" + render_env_block(env: w.env, indent: "  ") + "\n"
  let cache_block = match w.cache {
    null => ""
    c => render_gitlab_cache(cache: c)
  }
  let jobs = w.jobs |> map(j => render_gitlab_job(job: j)) |> join("\n")
  "{header}\nimage: rust:latest\n\n{vars}\nstages:\n  - ci\n\n{cache_block}\n{jobs}"
}
```

### CG-3: `dsl/tools/cigen.dag` — Entrypoint Tool

Follows the makegen entrypoint pattern exactly: extern discovery → pure render → `content_upsert`.

```dag
module tools.cigen

import std.ci {
  CiWorkflow, CiJob, CiStep, CiTrigger, CiPermission,
  CiCache, CiSecret, CiEnv, CiCheckout, CiProvider,
  rust_cargo_cache_paths, ci_cargo_env, default_checkout, default_trigger
}
import std.ci_render { render_github_workflow, render_gitlab_workflow }
import std.patterns { content_upsert }

// ── Extern bridge ──────────────────────────────────────────────────
// Discovery function returns structured config from Rust runtime.
// This is the only non-pure boundary — everything else is DSL rendering.

type CiConfig {
  workflow_name: String
  runner: String
  timeout_minutes: Int
  permissions: List<CiPermission>
  secrets: List<CiSecret>
  branches: List<String>
  tool_command: String
  bootstrap_script: String?
}

extern func discover_ci_config() -> CiConfig

// ── Workflow assembly ──────────────────────────────────────────────
// Constructs CiWorkflow records from discovered config.
// Pure function — all provider differences are handled in rendering.

fn build_checkout_step(checkout: CiCheckout) -> CiStep {
  let with_params = match checkout.fetch_depth {
    null => {}
    depth => { "fetch-depth": "{depth}" }
  }
  Uses {
    name: "Checkout",
    action: "actions/checkout@v4",
    with_params: with_params
  }
}

fn build_rust_setup_step() -> CiStep {
  Uses {
    name: "Setup Rust",
    action: "dtolnay/rust-toolchain@stable",
    with_params: {}
  }
}

fn build_cache_step(cache: CiCache) -> CiStep {
  let path_str = cache.paths |> join("\n            ")
  let restore_str = cache.restore_keys |> join("\n            ")
  Uses {
    name: "Cache Cargo",
    action: "actions/cache@v4",
    with_params: {
      "path": "|\n            {path_str}",
      "key": cache.key,
      "restore-keys": "|\n            {restore_str}"
    }
  }
}

fn build_bootstrap_step(script: String) -> CiStep {
  Run {
    name: "Verify Bootstrap Invariants",
    command: "|\n{script}",
    env: []
  }
}

fn build_ci_run_step(config: CiConfig) -> CiStep {
  DagRun {
    name: "Run CI Pipeline",
    tool_command: config.tool_command,
    env: [{ key: "CARGO_INCREMENTAL", value: "\"1\"" }],
    secrets: config.secrets
  }
}

fn build_github_cache() -> CiCache {
  {
    key: "cargo-${{{{ runner.os }}}}-${{{{ hashFiles('**/Cargo.lock') }}}}",
    paths: rust_cargo_cache_paths,
    restore_keys: ["cargo-${{{{ runner.os }}}}-"]
  }
}

fn build_github_steps(config: CiConfig) -> List<CiStep> {
  let checkout = build_checkout_step(checkout: default_checkout)
  let setup = build_rust_setup_step()
  let cache = build_cache_step(cache: build_github_cache())
  let bootstrap = match config.bootstrap_script {
    null => []
    script => [build_bootstrap_step(script: script)]
  }
  let run = build_ci_run_step(config: config)
  [checkout, setup, cache] |> append(items: bootstrap) |> append(items: [run])
}

fn build_workflow(config: CiConfig, provider: CiProvider) -> CiWorkflow {
  let trigger = {
    push_branches: config.branches,
    pr_branches: config.branches
  }
  let steps = match provider {
    GitHub => build_github_steps(config: config)
    GitLab => [build_ci_run_step(config: config)]
  }
  let job = {
    id: config.workflow_name,
    name: config.workflow_name,
    runner: config.runner,
    timeout_minutes: config.timeout_minutes,
    steps: steps,
    needs: []
  }
  {
    name: config.workflow_name,
    provider: provider,
    trigger: trigger,
    permissions: config.permissions,
    env: ci_cargo_env,
    cache: match provider {
      GitHub => build_github_cache()
      GitLab => null
    },
    jobs: [job]
  }
}

// ── Output paths ───────────────────────────────────────────────────

fn github_output_path(name: String) -> String {
  ".github/workflows/{name}.yml"
}

fn gitlab_output_path() -> String {
  ".gitlab-ci.yml"
}

// ── Entry point ────────────────────────────────────────────────────

func cigen() -> { written: Bool }
  uses fs: Filesystem(mode: ReadWrite)
{
  config = discover_ci_config()

  github_workflow = build_workflow(config: config, provider: GitHub)
  github_yaml = render_github_workflow(w: github_workflow)
  github_result = content_upsert(
    content: github_yaml,
    path: github_output_path(name: config.workflow_name)
  )

  gitlab_workflow = build_workflow(config: config, provider: GitLab)
  gitlab_yaml = render_gitlab_workflow(w: gitlab_workflow)
  gitlab_result = content_upsert(
    content: gitlab_yaml,
    path: gitlab_output_path()
  )

  return { written: github_result.written || gitlab_result.written }
}
```

### CG-4: Delete Rust cigen Code

**What to delete from `codegen_cli.rs`:**
- `generate_github_actions_template()` (~60 lines)
- `generate_gitlab_ci_template()` (~20 lines)
- `validate_github_actions_template()` (~20 lines)
- `validate_gitlab_ci_template()` (~10 lines)
- `validate_generated_ci_template()` dispatcher
- `validate_required_sections()` helper

**What to change:**
- `cmd_cigen()` → call `build_dsl_graph_for_entrypoint("tools.cigen", "cigen")`
  (same pattern as `cmd_codegen()`)

**Estimated deletion:** ~200 lines from `codegen_cli.rs`.

### CG-5: Migrate `RenderConfig` / `SharedStep` / `yaml_block`

After CG-1–CG-4, evaluate `core/ir/src/transport/ci/render.rs`:

| Component | Status After CG-4 | Action |
|-----------|-------------------|--------|
| `CiRenderer` trait | Still used by runtime progress rendering | **Keep** |
| `RenderConfig` struct | Dead — DSL `CiWorkflow` replaces it | **Delete** |
| `RenderConfig::all_env()` | Dead — `ci_cargo_env` data declaration | **Delete** |
| `RenderConfig::header()` | Dead — `render_ci_header` in DSL | **Delete** |
| `CheckoutConfig` | Dead — `CiCheckout` in DSL | **Delete** |
| `CacheConfig` / `CacheConfig::rust()` | Dead — `build_github_cache` in DSL | **Delete** |
| `SharedStep` enum | Dead — `CiStep` sum type in DSL | **Delete** |
| `dag_to_shared_steps()` | Dead — workflow assembly is DSL-side | **Delete** |
| `yaml_block()` | Dead — rendering is DSL string interpolation | **Delete** |
| Provider detection (`detect_provider`, `is_ci`) | Runtime-only, not CI YAML gen | **Keep** in `core/exec/src/ci_context.rs` |

**Also evaluate `gunbc-dag/src/makegen/ci_render.rs`:**

| Component | Status After CG-4 | Action |
|-----------|-------------------|--------|
| `workflow_specs_to_dag()` | Dead — DSL builds workflows directly | **Delete** |
| `render_github_actions_from_workflow_specs()` | Dead | **Delete** |
| `render_gitlab_ci_from_workflow_specs()` | Dead | **Delete** |

**Estimated deletion:** ~500 lines across both files.

## Extern Bridge Contract

The single extern boundary is `discover_ci_config() -> CiConfig`. The Rust
implementation collects:

| Field | Source |
|-------|--------|
| `workflow_name` | `RenderConfig::workflow_name` (currently `"ci"`) |
| `runner` | `RunnerImage::id` (currently `"ubuntu-latest"`) |
| `timeout_minutes` | `RenderConfig::timeout_minutes` (currently `30`) |
| `permissions` | `RenderConfig::permissions` (currently `[("contents","read"),("id-token","write")]`) |
| `secrets` | `RenderConfig::secrets_env` (currently GCP secrets from tool registry) |
| `branches` | `GitConfig::ci_branches()` (currently `["main"]`) |
| `tool_command` | `CargoInvocation::command()` for the CI binary |
| `bootstrap_script` | The bootstrap verification shell script (nullable) |

This is structurally identical to how `makegen.dag` uses `discover_tools()` —
the extern returns structured data, all rendering is pure DSL.

## DAG Topology

```
discover_ci_config [extern]
       │
       ▼
  build_workflow (GitHub)──► render_github_workflow ──► content_upsert (.github/workflows/ci.yml)
       │
  build_workflow (GitLab)──► render_gitlab_workflow ──► content_upsert (.gitlab-ci.yml)
```

Three nodes in the effectful path (discover + 2× upsert). Everything else is
pure computation — invisible to the execution engine.

## Structural Validity

The Rust validation functions (`validate_required_sections`, etc.) become
unnecessary because:

1. `CiWorkflow` requires `trigger`, `permissions`, `env`, `jobs` — the YAML
   sections are structurally guaranteed to exist.
2. `CiJob` requires `runner`, `steps` — `runs-on:` and `steps:` are always present.
3. `CiStep.Uses` requires `action` — no malformed `uses:` references.
4. GitHub interpolation balance (`${{ }}`) is handled by the rendering functions,
   not by post-hoc string scanning.

If the types construct, the YAML is valid. This is the "structural impossibility
of defects" principle from the red team philosophy.

## Test Strategy

### Unit Tests (in DSL)

```dag
test cigen_github_render {
  let workflow = {
    name: "ci",
    provider: GitHub,
    trigger: { push_branches: ["main"], pr_branches: ["main"] },
    permissions: [{ scope: "contents", level: "read" }],
    env: [{ key: "CARGO_TERM_COLOR", value: "always" }],
    cache: null,
    jobs: [{
      id: "ci",
      name: "ci",
      runner: "ubuntu-latest",
      timeout_minutes: 30,
      steps: [
        Run { name: "Build", command: "cargo build", env: [] }
      ],
      needs: []
    }]
  }
  let yaml = render_github_workflow(w: workflow)
  expect yaml |> contains("name: ci")
  expect yaml |> contains("runs-on: ubuntu-latest")
  expect yaml |> contains("cargo build")
}
```

### Drift Detection

The generated `.github/workflows/ci.yml` is already tracked by `content_upsert`'s
staleness detection. CI verifies no drift via the existing `verify` stage.

### Parity Test

A one-time migration test compares the DSL-rendered output against the current
Rust-rendered output to ensure byte-for-byte parity (or document intentional
differences). This test is deleted after the migration is verified.

## Sequencing

```
CG-1 (types)  ──► CG-2 (rendering) ──► CG-3 (tool) ──► CG-4 (delete Rust) ──► CG-5 (cleanup)
```

Each step is independently mergeable. CG-1 and CG-2 have zero runtime impact
(new DSL files, no wiring changes). CG-3 introduces the new tool alongside the
old one. CG-4 is the cutover. CG-5 is dead code cleanup.

## Appendix: Current Output Reference

The design targets byte-level parity with the current generated
`.github/workflows/ci.yml` (68 lines). Key structural elements:

- Header: `# Generated by gunbc-codegen` + `# DO NOT EDIT`
- Trigger: `on: push/pull_request` on `main`
- Permissions: `contents: read`, `id-token: write`
- Env: `CARGO_TERM_COLOR: always`, `RUSTFLAGS: -D warnings`
- Job: single `ci` job on `ubuntu-latest`, 30min timeout
- Steps: Checkout → Setup Rust → Cache Cargo → Verify Bootstrap → Run CI
- Secrets: 5 GCP secrets via `${{ secrets.NAME }}` syntax
