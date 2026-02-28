# CI DSL DAG Design

> Design for CG-1 through CG-5: migrating CI YAML generation from Rust string
> concatenation to DSL-first modeling. Introduces a **compositional
> format-aware rendering layer** that bridges `std/languages.dag` (language
> models) with the existing `Document` rendering engine, then builds CI
> rendering as a consumer of that layer.

## Status

Draft — 2026-02-28

## Problem

### The immediate problem

CI YAML generation lives in ~120 lines of hand-wired `push_str`/`write!` string
concatenation in `codegen_cli.rs:503-609`, plus ~60 lines of structural
validation. The Rust substrate carries a parallel `RenderConfig` builder,
`SharedStep` enum, and `yaml_block` utility in
`core/ir/src/transport/ci/render.rs` (~510 lines), along with a
`WorkflowSpec`→DAG bridge in `gunbc-dag/src/makegen/ci_render.rs` (~190 lines).

### The deeper problem: disconnected language models

`std/languages.dag` defines **Layer 0-2 language/format models** — `CommentSyntax`,
`ConfigFormat`, and data declarations for every format the project generates into:

| Format | Data Declaration | `comment.line_prefix` |
|--------|------------------|-----------------------|
| YAML | `yaml_format` | `"#"` |
| TOML | `toml_format` | `"#"` |
| Makefile | `makefile_format` | `"#"` |
| Gitignore | `gitignore_format` | `"#"` |
| Rust | `rust_language` | `"//"` |

Meanwhile, `shared/dag_util.dag` has a `Document` rendering engine that takes
`comment_prefix: String` — but every consumer manually threads this as a literal
`"#"`:

```
pragma.dag:    doc_with_header(header: ..., comment_prefix: "#", ...)
bootstrap.dag: generated_header(tool: "bootstrap") + ...   // hardcoded "#"
deps.dag:      generated_header(tool: "deps")               // hardcoded "#"
makegen.dag:   "# ====..." / "# {comment}\n"                // hardcoded "#"
```

The language models define "YAML uses `#` for comments" and "Rust uses `//`" —
but no rendering code reads this. The Rust layer has a *parallel* `CommentSyntax`
struct and `generated_header(generator, cmd, prefix)` that also takes a raw
string prefix. Two systems, zero connection.

This is exactly the "stringly-typed" and "heuristic reimplementation" smells from
the red team checklist. The comment prefix is a *derived property* of the output
format, not an independent parameter.

## Goals

1. **Format-aware rendering layer** (`std/format_render.dag`) — bridge language
   models to the `Document` rendering engine. Comment syntax, headers, and
   section markers derived from `ConfigFormat`/`Language`, not hardcoded strings.
2. **CI YAML types** (`std/ci.dag`) — `CiWorkflow`, `CiJob`, `CiStep`, etc.
3. **CI rendering as a consumer** (`std/ci_render.dag`) — uses format-aware
   primitives for comment-syntactic elements, direct string interpolation for
   YAML-structural elements.
4. **Entrypoint tool** (`tools/cigen.dag`) — extern discovery → pure render →
   `content_upsert`.
5. **Existing tool migration path** — `pragma.dag`, `bootstrap.dag`, `deps.dag`,
   `makegen.dag` can adopt format-aware rendering incrementally.
6. **Delete Rust cigen code** — ~700 lines across `codegen_cli.rs`,
   `transport/ci/render.rs`, `makegen/ci_render.rs`.

## Non-Goals

- General YAML serializer — YAML-structural rendering (mappings, sequences,
  indentation) stays as string interpolation. The format-aware layer handles
  only the *language-model-derived* properties (comments, headers).
- Runtime step-level CI rendering (animated progress) — `CiRenderer` trait stays
  in Rust.
- Provider detection (`detect_provider`, `is_ci`) — stays in Rust runtime.
- Full migration of all existing tools to format-aware rendering — this design
  enables it; follow-up work executes it.

## Architecture: Compositional Rendering Layers

```
Layer 0: std/languages.dag          "What is YAML?" — CommentSyntax, ConfigFormat
              │
Layer 1: std/format_render.dag      Format-aware primitives — derived from Layer 0
              │                     (generated_header, format_comment, format_document)
              │
Layer 2: std/ci_render.dag          CI-specific rendering — consumes Layer 1 for
         dsl/tools/makegen.dag      comment/header, direct interpolation for structure
         dsl/tools/pragma.dag
              │
Layer 3: dsl/tools/cigen.dag        Tool entrypoint — discovery → render → upsert
```

This mirrors the compositional modeling philosophy: each layer adds invariants
(format → comment syntax → rendering rules), and consumers name only the top
layer they need.

## Design

### File Layout

```
dsl/std/format_render.dag   — NEW: Format-aware rendering primitives (Layer 1)
dsl/std/ci.dag              — CG-1: CI model types + data declarations
dsl/std/ci_render.dag       — CG-2: CI rendering functions (Layer 2)
dsl/tools/cigen.dag         — CG-3: Entrypoint tool (Layer 3)
```

### Layer 1: `dsl/std/format_render.dag` — Format-Aware Rendering

The missing bridge. This module connects `std/languages.dag` (what is YAML?) to
`shared/dag_util.dag` (Document rendering engine) by deriving rendering
properties from format models.

#### Current state (every tool does this)

```dag
// pragma.dag — manually threads "#" as comment prefix
doc_with_header(
  header: generated_header(tool: "pragma"),   // hardcodes "# Generated by gunbc pragma"
  comment_prefix: "#",                         // manually specified, could be wrong
  sections: [section(lines: entries)]
)
```

#### Proposed state (format-derived)

```dag
// pragma.dag — derives comment prefix from format model
format_document(
  format: toml_format,                        // "What is TOML?" → comment.line_prefix = "#"
  generator: "pragma",
  regen_command: "cargo run -p gunbc-dag --bin gunbc-pragma",
  sections: [section(lines: entries)]
)
```

#### Proposed types and functions

```dag
module std.format_render

import std.languages { CommentSyntax, ConfigFormat, Language }
import std.types { Document, DocumentLine, DocumentSection }
import shared.dag_util { doc_with_header, render_document }

// ── Comment-syntax extraction ──────────────────────────────────────
// Derive the comment prefix from a format/language model.
// This is the bridge: format model → string prefix → Document engine.

fn comment_prefix_for(syntax: CommentSyntax) -> String {
  syntax.line_prefix ?? ""
}

fn config_comment_prefix(format: ConfigFormat) -> String {
  comment_prefix_for(syntax: format.comment)
}

fn language_comment_prefix(lang: Language) -> String {
  comment_prefix_for(syntax: lang.comment)
}

// ── Format-aware header ────────────────────────────────────────────
// "Generated by" header derived from format model. No hardcoded prefix.

fn format_header(syntax: CommentSyntax, generator: String, regen_command: String) -> String {
  let prefix = comment_prefix_for(syntax: syntax)
  "{prefix} Generated by {generator}\n{prefix} DO NOT EDIT - regenerate with: {regen_command}"
}

fn config_format_header(format: ConfigFormat, generator: String, regen_command: String) -> String {
  format_header(syntax: format.comment, generator: generator, regen_command: regen_command)
}

fn language_format_header(lang: Language, generator: String, regen_command: String) -> String {
  format_header(syntax: lang.comment, generator: generator, regen_command: regen_command)
}

// ── Format-aware comment line ──────────────────────────────────────
// Single line comment, derived from format model.

fn format_comment(syntax: CommentSyntax, text: String) -> String {
  let prefix = comment_prefix_for(syntax: syntax)
  "{prefix} {text}"
}

// ── Format-aware section header ────────────────────────────────────
// Prominent section separator, derived from format model.

fn format_section_header(syntax: CommentSyntax, title: String) -> String {
  let prefix = comment_prefix_for(syntax: syntax)
  let separator = "=" |> repeat(76)
  "{prefix} {separator}\n{prefix} {title}\n{prefix} {separator}"
}

// ── Format-aware Document construction ─────────────────────────────
// Constructs a Document with comment_prefix derived from format model.
// Composes with the existing render_document engine in dag_util.

fn format_document(
  format: ConfigFormat,
  generator: String,
  regen_command: String,
  sections: List<DocumentSection>
) -> Document {
  let header = config_format_header(
    format: format,
    generator: generator,
    regen_command: regen_command
  )
  let prefix = config_comment_prefix(format: format)
  doc_with_header(header: header, comment_prefix: prefix, sections: sections)
}

// Convenience: construct + render in one step.
fn render_format_document(
  format: ConfigFormat,
  generator: String,
  regen_command: String,
  sections: List<DocumentSection>
) -> String {
  render_document(
    document: format_document(
      format: format,
      generator: generator,
      regen_command: regen_command,
      sections: sections
    )
  )
}
```

#### Why this matters

1. **Single source of truth**: "YAML uses `#`" is defined once in `yaml_format`,
   not repeated in every rendering function.
2. **Composition, not convention**: You can't accidentally use `"//"` for a YAML
   file — the format model enforces the correct comment syntax.
3. **Incremental adoption**: Existing tools can migrate one at a time from
   `doc_with_header(..., comment_prefix: "#", ...)` to
   `format_document(format: toml_format, ...)`.
4. **Language-aware code generation**: When we emit generated Rust, we can use
   `language_format_header(lang: rust_language, ...)` to get `//` comments
   automatically, closing the gap between the emit pipeline's hardcoded
   `"//"` and the DSL language models.

#### Migration path for existing tools

| Tool | Current | After |
|------|---------|-------|
| `pragma.dag` (clippy.toml) | `doc_with_header(..., comment_prefix: "#")` | `format_document(format: toml_format, ...)` |
| `pragma.dag` (lint policy) | `doc_with_header(..., comment_prefix: "#")` | `format_document(format: toml_format, ...)` |
| `bootstrap.dag` (Makefile) | `generated_header(tool: "bootstrap")` + `"#"` | `config_format_header(format: makefile_format, ...)` |
| `bootstrap.dag` (.gitignore) | `generated_header(tool: "bootstrap")` + `"#"` | `config_format_header(format: gitignore_format, ...)` |
| `deps.dag` (deps.toml) | `generated_header(tool: "deps")` + `"#"` | `config_format_header(format: toml_format, ...)` |
| `makegen.dag` (Makefile) | `"# Generated by..."` | `config_format_header(format: makefile_format, ...)` |
| `cigen.dag` (CI YAML) | — | `config_format_header(format: yaml_format, ...)` |

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

```dag
data rust_cargo_cache_paths: List<String> = [
  "~/.cargo/bin/",
  "~/.cargo/registry/index/",
  "~/.cargo/registry/cache/",
  "~/.cargo/git/db/"
]

data ci_cargo_env: List<CiEnv> = [
  { key: "CARGO_TERM_COLOR", value: "always" },
  { key: "RUSTFLAGS", value: "-D warnings" }
]

data default_checkout: CiCheckout = {
  fetch_depth: 1,
  submodules: null
}

data default_trigger: CiTrigger = {
  push_branches: ["main"],
  pr_branches: ["main"]
}
```

### CG-2: `dsl/std/ci_render.dag` — CI Rendering Functions

CI rendering composes two levels:

1. **Format-aware** (from `format_render.dag`): header comment lines, generated-by
   notices — anything that depends on "this is a YAML file" flows through the
   language model.
2. **YAML-structural** (direct interpolation): mappings, sequences, indentation
   patterns specific to GitHub Actions / GitLab CI YAML structure.

This split is deliberate: the format-aware layer handles properties *derived from
the language model* (comment syntax), while YAML structural rendering stays as
string interpolation helpers. We don't need a general YAML serializer — the CI
YAML schema is fixed and small.

#### Function Catalog

| Function | Responsibility | Layer |
|----------|----------------|-------|
| `render_github_workflow` | Top-level GitHub Actions YAML | Structural + Format |
| `render_gitlab_workflow` | Top-level GitLab CI YAML | Structural + Format |
| `render_ci_header` | Header via `config_format_header(yaml_format, ...)` | Format-derived |
| `render_yaml_mapping` | `key: value` with indent | Structural helper |
| `render_yaml_sequence` | `- item` with indent | Structural helper |
| `render_yaml_block` | Header + indented items | Structural helper |
| `render_github_trigger` | `on:` block | Structural |
| `render_permissions` | `permissions:` block | Structural |
| `render_env_block` | `env:` / `variables:` block | Structural |
| `render_github_step` | Individual step (match on CiStep) | Structural |
| `render_github_job` | Full job block | Structural |
| `render_gitlab_job` | GitLab job block | Structural |
| `render_gitlab_cache` | GitLab cache block | Structural |

#### Key: Format-derived header

```dag
import std.languages { yaml_format }
import std.format_render { config_format_header }

fn render_ci_header(generator: String, regen_command: String) -> String {
  config_format_header(
    format: yaml_format,
    generator: generator,
    regen_command: regen_command
  )
}
```

This replaces the hardcoded `"# Generated by..."` — the `#` prefix is now
derived from `yaml_format.comment.line_prefix`.

#### YAML structural helpers

These are CI-specific and don't belong in the generic format layer. They encode
knowledge about YAML structure, not about the YAML *language model*.

```dag
fn render_yaml_mapping(indent: Int, key: String, value: String) -> String {
  let pad = "  " |> repeat(indent)
  "{pad}{key}: {value}"
}

fn render_yaml_sequence(indent: Int, items: List<String>) -> String {
  let pad = "  " |> repeat(indent)
  items |> map(item => "{pad}- {item}") |> join("\n")
}

fn render_yaml_block(indent: Int, header: String, items: List<String>) -> String {
  let pad = "  " |> repeat(indent)
  let lines = items |> map(item => "{pad}  {item}") |> join("\n")
  "{pad}{header}\n{lines}"
}
```

#### Provider rendering

```dag
fn render_github_trigger(trigger: CiTrigger) -> String {
  let push_lines = trigger.push_branches
    |> map(b => render_yaml_mapping(indent: 3, key: "", value: b))
    |> join("\n")
  let pr_lines = trigger.pr_branches
    |> map(b => render_yaml_mapping(indent: 3, key: "", value: b))
    |> join("\n")
  "on:\n  push:\n    branches:\n" +
    render_yaml_sequence(indent: 3, items: trigger.push_branches) +
    "\n  pull_request:\n    branches:\n" +
    render_yaml_sequence(indent: 3, items: trigger.pr_branches) + "\n"
}

fn render_permissions(perms: List<CiPermission>) -> String {
  let lines = perms |> map(p => render_yaml_mapping(indent: 1, key: p.scope, value: p.level))
  "permissions:\n" + lines |> join("\n") + "\n"
}

fn render_env_block(env: List<CiEnv>, indent: Int) -> String {
  env |> map(e => render_yaml_mapping(indent: indent, key: e.key, value: e.value)) |> join("\n")
}
```

#### Step rendering (match on sum type)

```dag
fn render_github_step(step: CiStep) -> String {
  match step {
    Run(s) => {
      let env_section = if s.env |> count() > 0 {
        "\n        env:\n" + render_env_block(env: s.env, indent: 5)
      } else { "" }
      "      - name: {s.name}\n        run: {s.command}{env_section}"
    }
    Uses(s) => {
      let with_section = if s.with_params |> count() > 0 {
        let params = s.with_params
          |> map((k, v) => render_yaml_mapping(indent: 5, key: k, value: v))
          |> join("\n")
        "\n        with:\n{params}"
      } else { "" }
      "      - name: {s.name}\n        uses: {s.action}{with_section}"
    }
    DagRun(s) => {
      let base_env = s.env
      let secret_env = s.secrets
        |> map(sec => { key: sec.name, value: "${{{{ secrets.{sec.name} }}}}" })
      let all_env = base_env |> append(items: secret_env)
      let env_section = if all_env |> count() > 0 {
        "\n        env:\n" + render_env_block(env: all_env, indent: 5)
      } else { "" }
      "      - name: {s.name}\n        run: {s.tool_command}{env_section}"
    }
  }
}
```

#### Top-level assembly — format-aware header, structural body

```dag
fn render_github_workflow(w: CiWorkflow) -> String {
  let header = render_ci_header(
    generator: "gunbc-codegen",
    regen_command: "cargo run -p gunbc-dag --bin gunbc-codegen -- cigen"
  )
  let trigger = render_github_trigger(trigger: w.trigger)
  let perms = render_permissions(perms: w.permissions)
  let env = "env:\n" + render_env_block(env: w.env, indent: 1) + "\n"
  let jobs = w.jobs
    |> map(j => render_github_job(job: j, workflow: w))
    |> join("\n")
  "{header}\n\nname: {w.name}\n\n{trigger}\n{perms}\n{env}\njobs:\n{jobs}"
}

fn render_gitlab_workflow(w: CiWorkflow) -> String {
  let header = render_ci_header(
    generator: "gunbc-codegen",
    regen_command: "cargo run -p gunbc-dag --bin gunbc-codegen -- cigen"
  )
  let vars = "variables:\n" + render_env_block(env: w.env, indent: 1) + "\n"
  let cache_block = match w.cache {
    null => ""
    c => render_gitlab_cache(cache: c)
  }
  let jobs = w.jobs |> map(j => render_gitlab_job(job: j)) |> join("\n")
  "{header}\n\nimage: rust:latest\n\n{vars}\nstages:\n  - ci\n\n{cache_block}\n{jobs}"
}
```

### CG-3: `dsl/tools/cigen.dag` — Entrypoint Tool

Follows the makegen entrypoint pattern: extern discovery → pure render →
`content_upsert`.

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

**Delete from `codegen_cli.rs`:**
- `generate_github_actions_template()` (~60 lines)
- `generate_gitlab_ci_template()` (~20 lines)
- `validate_github_actions_template()` (~20 lines)
- `validate_gitlab_ci_template()` (~10 lines)
- `validate_generated_ci_template()` dispatcher
- `validate_required_sections()` helper

**Change:** `cmd_cigen()` → `build_dsl_graph_for_entrypoint("tools.cigen", "cigen")`

**Estimated deletion:** ~200 lines from `codegen_cli.rs`.

### CG-5: Migrate `RenderConfig` / `SharedStep` / `yaml_block`

After CG-1–CG-4, evaluate `core/ir/src/transport/ci/render.rs`:

| Component | Status After CG-4 | Action |
|-----------|-------------------|--------|
| `CiRenderer` trait | Runtime progress rendering | **Keep** |
| `RenderConfig` struct | Dead — `CiWorkflow` replaces it | **Delete** |
| `RenderConfig::all_env()` | Dead — `ci_cargo_env` data decl | **Delete** |
| `RenderConfig::header()` | Dead — `config_format_header` in DSL | **Delete** |
| `CheckoutConfig` | Dead — `CiCheckout` in DSL | **Delete** |
| `CacheConfig` / `CacheConfig::rust()` | Dead — `build_github_cache` in DSL | **Delete** |
| `SharedStep` enum | Dead — `CiStep` sum type in DSL | **Delete** |
| `dag_to_shared_steps()` | Dead — assembly is DSL-side | **Delete** |
| `yaml_block()` | Dead — rendering is DSL | **Delete** |
| `detect_provider`, `is_ci` | Runtime only | **Keep** in `core/exec/` |

**Also `gunbc-dag/src/makegen/ci_render.rs`:**

| Component | Status After CG-4 | Action |
|-----------|-------------------|--------|
| `workflow_specs_to_dag()` | Dead | **Delete** |
| `render_github_actions_from_workflow_specs()` | Dead | **Delete** |
| `render_gitlab_ci_from_workflow_specs()` | Dead | **Delete** |

**Estimated deletion:** ~500 lines across both files.

## Extern Bridge Contract

Single extern boundary: `discover_ci_config() -> CiConfig`.

| Field | Source |
|-------|--------|
| `workflow_name` | `RenderConfig::workflow_name` (currently `"ci"`) |
| `runner` | `RunnerImage::id` (currently `"ubuntu-latest"`) |
| `timeout_minutes` | `RenderConfig::timeout_minutes` (currently `30`) |
| `permissions` | `RenderConfig::permissions` (currently `[("contents","read"),("id-token","write")]`) |
| `secrets` | `RenderConfig::secrets_env` (currently GCP secrets from tool registry) |
| `branches` | `GitConfig::ci_branches()` (currently `["main"]`) |
| `tool_command` | `CargoInvocation::command()` for the CI binary |
| `bootstrap_script` | Bootstrap verification shell script (nullable) |

## DAG Topology

```
discover_ci_config [extern]
       │
       ▼
  build_workflow(GitHub) ──► render_github_workflow ──► content_upsert (.github/workflows/ci.yml)
       │                          │
       │                    config_format_header(yaml_format, ...)
       │                          │
       │                    yaml_format.comment.line_prefix = "#"
       │
  build_workflow(GitLab) ──► render_gitlab_workflow ──► content_upsert (.gitlab-ci.yml)
```

Three effectful nodes (discover + 2× upsert). The format-aware layer is pure
computation — the `yaml_format` data declaration flows through rendering
functions at compile time.

## Structural Validity

Validation functions (`validate_required_sections`, etc.) become unnecessary:

1. `CiWorkflow` requires `trigger`, `permissions`, `env`, `jobs` — YAML sections
   structurally guaranteed.
2. `CiJob` requires `runner`, `steps` — `runs-on:` and `steps:` always present.
3. `CiStep.Uses` requires `action` — no malformed `uses:` references.
4. GitHub interpolation balance (`${{ }}`) handled by rendering functions.
5. Comment syntax is *derived* from `yaml_format` — can't accidentally use `//`.

If the types construct, the YAML is valid.

## Interaction with Emit Pipeline

The Rust emit pipeline (`core/daglang/daglang-emit/`) currently hardcodes
comment prefixes:

- `render_rust.rs` hardcodes `"//!"`, `"///"`, `"//"`
- `FileHeader` in `render_ir.rs` takes `comment_prefix: &str`
- Rust `CommentSyntax` struct exists separately from DSL models

The format-aware rendering layer in DSL establishes the pattern for a future
consolidation: the emit pipeline could consume `rust_language.comment.line_prefix`
(= `"//"`) and `rust_language.comment.doc_prefix` (= `"///"`) from
`std/languages.dag` rather than hardcoding these strings. This is out of scope
for the current CG-1:5 work but the architecture supports it naturally:

```
Future: emit pipeline reads language model → derives comment syntax
Now:    cigen reads yaml_format → derives comment syntax    (same pattern)
```

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
      id: "ci", name: "ci", runner: "ubuntu-latest", timeout_minutes: 30,
      steps: [Run { name: "Build", command: "cargo build", env: [] }],
      needs: []
    }]
  }
  let yaml = render_github_workflow(w: workflow)
  expect yaml |> starts_with(prefix: "# Generated by")
  expect yaml |> contains("name: ci")
  expect yaml |> contains("runs-on: ubuntu-latest")
}
```

### Format-Aware Rendering Tests

```dag
test format_header_yaml {
  let header = config_format_header(
    format: yaml_format,
    generator: "test",
    regen_command: "make test"
  )
  expect header |> starts_with(prefix: "#")
  expect header |> contains("Generated by test")
}

test format_header_rust {
  let header = language_format_header(
    lang: rust_language,
    generator: "test",
    regen_command: "make test"
  )
  expect header |> starts_with(prefix: "//")
}
```

### Drift Detection

Generated `.github/workflows/ci.yml` tracked by `content_upsert`'s staleness
detection. CI verifies no drift via the existing `verify` stage.

### Parity Test

One-time migration test comparing DSL-rendered vs Rust-rendered output.
Deleted after migration verified.

## Sequencing

```
format_render.dag (Layer 1)  ──► CG-1 (types) ──► CG-2 (rendering) ──► CG-3 (tool) ──► CG-4 (delete) ──► CG-5 (cleanup)
                                                        │
                                                  uses format_render
```

The format-aware rendering layer ships first (or with CG-1) since it has zero
runtime impact — it's new DSL functions that nothing depends on yet. CG-2 is the
first consumer. Existing tools migrate incrementally as follow-up work.

## Appendix A: Current Output Reference

The design targets byte-level parity with `.github/workflows/ci.yml` (68 lines):

- Header: `# Generated by gunbc-codegen` + `# DO NOT EDIT`
- Trigger: `on: push/pull_request` on `main`
- Permissions: `contents: read`, `id-token: write`
- Env: `CARGO_TERM_COLOR: always`, `RUSTFLAGS: -D warnings`
- Job: single `ci` job on `ubuntu-latest`, 30min timeout
- Steps: Checkout → Setup Rust → Cache Cargo → Verify Bootstrap → Run CI
- Secrets: 5 GCP secrets via `${{ secrets.NAME }}` syntax

## Appendix B: Consolidation Inventory

All places that hardcode comment syntax strings, now replaceable via
`format_render.dag`:

| Location | Hardcoded | Derived From |
|----------|-----------|--------------|
| `shared/dag_util.dag:79` | `"# Generated by gunbc {tool}\n"` | `config_format_header(format, ...)` |
| `shared/dag_util.dag:109` | `comment_prefix: "#"` | `config_comment_prefix(format)` |
| `tools/pragma.dag:27,42,57` | `comment_prefix: "#"` | `format_document(format: toml_format, ...)` |
| `tools/bootstrap.dag:21,29` | `generated_header(tool: ...)` | `config_format_header(format: makefile_format, ...)` / `config_format_header(format: gitignore_format, ...)` |
| `tools/deps.dag:28` | `generated_header(tool: ...)` | `config_format_header(format: toml_format, ...)` |
| `tools/makegen.dag:25-29` | `"# {comment}\n"` | `format_comment(syntax: makefile_format.comment, ...)` |
| `tools/makegen.dag:33` | `"# ==..."` | `format_section_header(syntax: makefile_format.comment, ...)` |
| `core/ir/.../comment.rs:123` | `prefix` param | Future: `language_comment_prefix(lang)` |
| `core/daglang/.../render_rust.rs` | `"//"`, `"///"` | Future: `rust_language.comment` |
