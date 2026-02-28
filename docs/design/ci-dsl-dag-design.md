# CI DSL DAG Design

> Design for CG-1 through CG-5: migrating CI YAML generation from Rust string
> concatenation to DSL-first modeling via **stacked tautologies** — each
> external dependency modeled as its own layer of facts, composed upward.

## Status

Implemented — 2026-02-28

## Problem

### The immediate problem

CI YAML generation lives in ~120 lines of hand-wired `push_str`/`write!` string
concatenation in `codegen_cli.rs:503-609`, plus ~60 lines of structural
validation. The Rust substrate carries a parallel `RenderConfig` builder,
`SharedStep` enum, and `yaml_block` utility in
`core/ir/src/transport/ci/render.rs` (~510 lines), along with a
`WorkflowSpec`→DAG bridge in `gunbc-dag/src/makegen/ci_render.rs` (~190 lines).

### The deeper problem: conflated external dependencies

The old code conflates three distinct external systems:

1. **YAML** — a serialization language with indentation rules, mappings,
   sequences, `#` comments
2. **GitHub Actions** — a CI service that uses YAML, with its own schema
   (`on:`, `jobs:`, `steps:`, `${{ secrets.NAME }}`)
3. **GitLab CI** — another CI service with different YAML schema
   (`stages:`, `needs:`, `script:`, `$VARIABLE`)

These were mixed in a single `RenderConfig` struct and procedural functions
that knew about YAML syntax, GitHub Actions schema, and GitLab CI schema
simultaneously. The DSL language models (`std/languages.dag`) already defined
`yaml_format` with comment syntax — but no rendering code read it.

### The principle

Each external dependency should be modeled as its own tautology — facts about
what the system *is*, not hacks or quirks. Then we compose those tautologies:

```
YAML (language)  →  GitHub Actions / GitLab CI (interfaces)  →  our config (policy)  →  tool
```

This is the same compositional modeling used throughout: TCP → TLS → HTTP →
REST → provider → operation. Each layer adds invariants.

## Architecture: Stacked Tautologies

```
Layer 0: std/languages.dag          "What is YAML?"  — ConfigFormat, CommentSyntax
         std/format_render.dag      Format-aware rendering bridge
         std/yaml_render.dag        YAML serialization primitives
              │
Layer 1: extdeps/github_actions.dag "What is GitHub Actions?" — Workflow, Job, Step
         extdeps/gitlab_ci.dag      "What is GitLab CI?"      — Pipeline, Job, Variable
              │
Layer 2: config/ci.dag              Our CI requirements — branches, runner, permissions
              │
Layer 3: tools/cigen.dag            Rendering + tool entrypoint
```

Import direction: `tools/ → config/ → extdeps/ → std/`. Never backwards.

## File Layout

| File | Layer | Purpose |
|------|-------|---------|
| `dsl/std/format_render.dag` | 0 | Format-aware rendering bridge: `ConfigFormat` → comment syntax |
| `dsl/std/yaml_render.dag` | 0 | YAML serialization: mappings, sequences, indentation |
| `dsl/extdeps/github_actions.dag` | 1 | "What is GitHub Actions?" — types + data |
| `dsl/extdeps/gitlab_ci.dag` | 1 | "What is GitLab CI?" — types + data |
| `dsl/config/ci.dag` | 2 | Our CI configuration data declarations |
| `dsl/tools/cigen.dag` | 3 | Rendering functions + `func cigen()` entrypoint |

## Layer 0: Format-Aware Rendering

### `std/format_render.dag` — Language Model → Rendering Bridge

Bridges `std/languages.dag` to the `Document` rendering engine in
`shared/dag_util.dag`. Derives comment syntax from `ConfigFormat` / `Language`
data instead of hardcoded strings.

**Key functions:**

| Function | Signature | Replaces |
|----------|-----------|----------|
| `comment_prefix_for` | `(syntax: CommentSyntax) -> String` | Every hardcoded `"#"` |
| `config_format_header` | `(format: ConfigFormat, generator, regen_command) -> String` | `generated_header(tool)` |
| `language_format_header` | `(lang: Language, generator, regen_command) -> String` | Rust `generated_header()` |
| `format_document` | `(format: ConfigFormat, ..., sections) -> Document` | `doc_with_header(..., comment_prefix: "#")` |

**Consolidation enabled** — existing tools can migrate incrementally:

| Tool | Current | After |
|------|---------|-------|
| `pragma.dag` | `doc_with_header(..., comment_prefix: "#")` | `format_document(format: toml_format, ...)` |
| `bootstrap.dag` | `generated_header(tool: "bootstrap")` | `config_format_header(format: makefile_format, ...)` |
| `deps.dag` | `generated_header(tool: "deps")` | `config_format_header(format: toml_format, ...)` |
| `makegen.dag` | `"# ====..."` | `format_section_header(syntax: makefile_format.comment, ...)` |

### `std/yaml_render.dag` — YAML Serialization Primitives

Follows the `std/markdown_render.dag` pattern: format-specific rendering in
`std/`. Encodes YAML's serialization rules as pure functions.

**Key functions:**

| Function | Signature | Encodes |
|----------|-----------|---------|
| `yaml_mapping` | `(key, value) -> String` | `key: value` |
| `yaml_sequence_item` | `(value) -> String` | `- value` |
| `yaml_indent` | `(level, text) -> String` | 2-space indent |
| `yaml_indented_mapping` | `(level, key, value) -> String` | Indented `key: value` |
| `yaml_block` | `(header, level, items) -> String` | Header + indented items |
| `yaml_sequence_block` | `(header, level, items) -> String` | Header + `- items` |
| `yaml_header` | `(generator, regen_command) -> String` | `# Generated by ...` (via format_render) |
| `yaml_comment` | `(text) -> String` | `# text` (via format_render) |

The YAML comment function derives its `#` prefix from `yaml_format.comment`
in `std/languages.dag` — it does not hardcode `"#"`.

## Layer 1: External Dependency Models

### `extdeps/github_actions.dag` — "What is GitHub Actions?"

Follows the `extdeps/clippy.dag` and `extdeps/make.dag` pattern: types + data,
zero opinions about our repo.

**Types:**

```
Trigger        — push_branches, pr_branches
Permission     — scope, level
EnvVar         — key, value
SecretRef      — name
Cache          — key, paths, restore_keys
Checkout       — fetch_depth?, submodules?
RunnerImage    — id
Step           = Run { name, command, env }
               | Uses { name, action, with_params }
Job            — id, name, runner, timeout_minutes, steps, needs
Workflow       — name, trigger, permissions, env, cache?, jobs
```

**Data declarations:**

```
ubuntu_latest, ubuntu_22_04, macos_latest, windows_latest  — RunnerImage
checkout_action, rust_toolchain_action, cache_action        — well-known action refs
output_dir = ".github/workflows"                            — output location
```

### `extdeps/gitlab_ci.dag` — "What is GitLab CI?"

**Types:**

```
Variable       — key, value
Need           — job
CacheConfig    — key, paths
Job            — id, stage, script, needs, variables
Pipeline       — image, stages, variables, cache?, jobs
```

**Data declarations:**

```
default_image = "rust:latest"
output_path   = ".gitlab-ci.yml"
```

## Layer 2: Our Configuration

### `config/ci.dag` — Our CI Requirements

Composes Layer 1 types with our repo's requirements. Static data declarations
only — dynamic values come from the extern bridge.

```
ci_trigger           — push/PR on ["main"]
ci_runner            — ubuntu_latest
ci_timeout_minutes   — 30
ci_permissions       — [contents:read, id-token:write]
ci_env               — [CARGO_TERM_COLOR:always, RUSTFLAGS:-D warnings]
ci_cache_paths       — [~/.cargo/bin/, registry/index/, registry/cache/, git/db/]
ci_checkout          — fetch_depth: 1
ci_workflow_name     — "ci"
ci_generator         — "gunbc-codegen"
ci_regen_command     — "cargo run -p gunbc-dag --bin gunbc-codegen -- cigen"
```

## Layer 3: Tool

### `tools/cigen.dag` — Rendering + Entrypoint

Follows the `tools/makegen.dag` pattern: rendering functions + extern bridge
+ `func cigen() -> { written: Bool }`.

**Extern bridge:** `discover_ci_config() -> CiDiscovery`

Only dynamic values:

| Field | Source |
|-------|--------|
| `secrets: List<String>` | Tool registry (GCP secrets) |
| `tool_command: String` | `CargoInvocation::command()` |
| `bootstrap_script: String?` | Crate graph verification script |

Static config comes from `config/ci.dag`, not the extern bridge.

**Rendering architecture:**

The rendering functions compose YAML primitives with provider schemas:

```
GitHub Actions rendering = yaml_render ∘ github_actions schema ∘ our config
GitLab CI rendering      = yaml_render ∘ gitlab_ci schema     ∘ our config
```

Both share the YAML substrate but produce different output structures.

**GitHub Actions provider-specific helpers:**

| Function | Responsibility |
|----------|----------------|
| `gha_secret_ref(name)` | `${{ secrets.NAME }}` syntax |
| `gha_runner_os_ref()` | `${{ runner.os }}` syntax |
| `gha_hashfiles_ref(pattern)` | `${{ hashFiles('...') }}` syntax |
| `build_github_cache()` | Cache with GitHub-specific key pattern |
| `build_github_steps(discovery)` | Checkout → Setup → Cache → Bootstrap → Run |
| `render_github_workflow(w)` | Full YAML rendering |

**GitLab CI provider-specific helpers:**

| Function | Responsibility |
|----------|----------------|
| `build_gitlab_pipeline(discovery)` | Pipeline with GitLab-specific structure |
| `render_gitlab_pipeline(p)` | Full YAML rendering |

**DAG topology:**

```
discover_ci_config [extern]
       │
       ├─► build_github_workflow ─► render_github_workflow ─► content_upsert (.github/workflows/ci.yml)
       │
       └─► build_gitlab_pipeline ─► render_gitlab_pipeline ─► content_upsert (.gitlab-ci.yml)
```

Three effectful nodes (discover + 2× upsert). All rendering is pure.

## CG-4: Delete Rust cigen Code

**Delete from `codegen_cli.rs`:**
- `generate_github_actions_template()` (~60 lines)
- `generate_gitlab_ci_template()` (~20 lines)
- `validate_github_actions_template()` (~20 lines)
- `validate_gitlab_ci_template()` (~10 lines)
- `validate_generated_ci_template()` dispatcher
- `validate_required_sections()` helper

**Change:** `cmd_cigen()` → `build_dsl_graph_for_entrypoint("tools.cigen", "cigen")`

## CG-5: Cleanup Dead Code

| Component | Location | Action |
|-----------|----------|--------|
| `RenderConfig` struct | `transport/ci/render.rs` | **Delete** |
| `CheckoutConfig`, `CacheConfig` | `transport/ci/render.rs` | **Delete** |
| `SharedStep` enum | `transport/ci/render.rs` | **Delete** |
| `dag_to_shared_steps()` | `transport/ci/render.rs` | **Delete** |
| `yaml_block()` | `transport/ci/render.rs` | **Delete** |
| `CiRenderer` trait | `transport/ci/render.rs` | **Keep** (runtime progress) |
| `workflow_specs_to_dag()` | `makegen/ci_render.rs` | **Delete** |
| `render_*_from_workflow_specs()` | `makegen/ci_render.rs` | **Delete** |
| `detect_provider`, `is_ci` | `exec/ci_context.rs` | **Keep** (runtime) |

**Estimated deletion:** ~700 lines across `codegen_cli.rs`,
`transport/ci/render.rs`, `makegen/ci_render.rs`.

## Structural Validity

Validation functions become unnecessary:

1. `Workflow` requires `trigger`, `permissions`, `env`, `jobs` — YAML sections
   structurally guaranteed.
2. `Job` requires `runner`, `steps` — `runs-on:` and `steps:` always present.
3. `Step.Uses` requires `action` — no malformed `uses:` references.
4. GitHub interpolation syntax is encapsulated in `gha_secret_ref()` et al.
5. Comment syntax derived from `yaml_format` — can't accidentally use `//`.

If the types construct, the YAML is valid.

## Sequencing

```
format_render.dag ─┐
yaml_render.dag    ├─► extdeps/*.dag ──► config/ci.dag ──► tools/cigen.dag ──► CG-4 (delete) ──► CG-5 (cleanup)
                   │
                   └─► (existing tools migrate incrementally)
```

The DSL files (Layers 0-3) are complete. CG-4 and CG-5 are the Rust-side
cutover and cleanup — they depend on wiring the extern bridge
(`discover_ci_config`) and registering the new tool with the executor.
