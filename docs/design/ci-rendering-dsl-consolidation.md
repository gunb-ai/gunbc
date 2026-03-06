# CI Rendering DSL Consolidation

> **Status update (2026-03-06)**: Phase 1 is complete on this branch. The
> Rust-side CI YAML renderers (`core/ir/src/transport/ci/render.rs`, provider
> YAML helpers, and `WorkflowConfig`) are deleted. The remaining work in this
> doc is Phase 2 and Phase 3 inside the live `.dag` generation path.

## Goal

Make CI artifact generation follow the repo's intended shape:

`config data -> typed provider model -> compositional .dag assembly -> final render leaf -> content_upsert`

and delete the Rust-side CI YAML rendering paths that no longer own repo truth.

This doc is narrower than `docs/design/unified-emission.md`. It focuses on the
CI/render cluster around `core/ir/src/transport/ci/render.rs` and the nearby
legacy surfaces that should be removed or folded into the DSL path now.

## Problem

The repo currently has three distinct CI rendering stories:

1. `dsl/config/ci.dag` + `dsl/tools/cigen.dag`
   Current repo-generation path. This is what actually writes
   `.github/workflows/ci.yml` and `.gitlab-ci.yml`.

2. `core/ir/src/transport/ci/render.rs` + `core/ir/src/transport/ci/providers/{github,gitlab}.rs`
   An intermediate Rust rendering model built around `CiRenderer`,
   `RenderConfig`, `SharedStep`, and `dag_to_shared_steps()`.

3. `core/ir/src/transport/github_actions.rs::WorkflowConfig`
   An older GitHub-only YAML renderer that predates the `CiRenderer` path.

Only (1) is repo truth today. (2) and (3) are retained rendering islands.

That leaves us with the exact failure mode we just hit elsewhere:

- policy is modeled in one place
- rendering logic exists in another
- drift is easy because the non-authoritative path still looks legitimate

The cache-path incident was the same class of bug:
the repo had the right CI policy in `config.ci`, but `tools/cigen.dag`
had drifted into renderer-owned policy.

`render.rs` and `WorkflowConfig` are older variants of the same smell:
they keep a parallel "CI rendering architecture" alive even though the repo
already decided that CI generation lives in the DSL tool path.

## Current Surfaces

### Authoritative now

- `dsl/config/ci.dag`
  Repo CI policy: triggers, env, permissions, cache config, stages, image.

- `dsl/extdeps/github_actions.dag`
  Provider schema for GitHub Actions.

- `dsl/extdeps/gitlab_ci.dag`
  Provider schema for GitLab CI.

- `dsl/tools/cigen.dag`
  The current CI generation tool.

- `gunbc-app/src/extern_ops.rs::DiscoverCiConfigOp`
  Runtime bridge that supplies discovery-only data to `tools.cigen`.

### Historical / deleted on branch

- `core/ir/src/transport/ci/render.rs`
  `CiRenderer`, `RenderConfig`, `CheckoutConfig`, `CacheConfig`,
  `SharedStep`, `dag_to_shared_steps()`.

- `core/ir/src/transport/ci/providers/github.rs`
  `CiProvider` for workflow commands, plus a separate YAML renderer.

- `core/ir/src/transport/ci/providers/gitlab.rs`
  `CiProvider` for workflow commands, plus a separate YAML renderer.

- `core/ir/src/transport/github_actions.rs::WorkflowConfig`
  Legacy GitHub YAML model and renderer.

### Adjacent follow-on render lanes

These are related, but should not block the CI cleanup:

- `dsl/tools/makegen.dag`
- `dsl/tools/justgen.dag`
- `core/codegen/src/cli_gen.rs`
- `gunbc-app/src/ci/ops.rs`
- `lib/markdown/src/lib.rs`
- `lib/design-ops/src/lib.rs`

The same architectural direction should apply to them, but CI is the easiest
place to delete an obviously dead Rust renderer immediately.

## Root Cause

This split exists because the repo migrated in stages:

1. `WorkflowConfig` introduced typed GitHub-side CI rendering in Rust.
2. `CiRenderer` / `SharedStep` generalized that idea to GitHub + GitLab.
3. `tools/cigen.dag` moved repo generation into the DSL.
4. The Rust renderers were never deleted.

So the repo is not blocked by missing theory. It is carrying an unfinished
migration bridge.

## Target Architecture

### Rule 1: CI YAML generation is DSL-owned

The only authoritative path for repository CI files is:

`config.ci -> tools.cigen -> content_upsert`

Rust may still provide extern discovery for facts the DSL cannot derive yet,
but Rust no longer owns CI YAML rendering logic.

### Rule 2: Provider schemas stay typed

`extdeps.github_actions` and `extdeps.gitlab_ci` remain the typed schema
definitions for provider concepts.

`tools.cigen.dag` should evolve toward assembling typed provider values first:

- `Workflow`
- `Job`
- `Step`
- `Pipeline`
- `Cache`
- `Variable`

and only render to YAML at the leaf.

### Rule 3: Rendering is a leaf, not a policy owner

No renderer should choose:

- cache paths
- permissions
- trigger branches
- runner image
- stage ordering

Those belong in `config.ci` or in discovery-only extern data.

### Rule 4: Runtime discovery crosses the boundary as structure

`CiDiscovery` should eventually stop returning shell-text fragments like:

- `tool_command: String`
- `bootstrap_script: String?`

and instead return typed step inputs, for example:

- `run_command: List<String>`
- `bootstrap_lines: List<String>`
- `secret_env: List<String>`

or a provider-neutral `CiStepSpec` value set.

That is not required to delete `render.rs`, but it is required for the full
compositional end state.

## Migration Plan

### Phase 1: Delete dead Rust CI YAML renderers

This phase is complete on this branch.

Delete:

- `core/ir/src/transport/ci/render.rs`
- `CiRenderer` impls and YAML render helpers in:
  - `core/ir/src/transport/ci/providers/github.rs`
  - `core/ir/src/transport/ci/providers/gitlab.rs`
- reexports of `CiRenderer`, `RenderConfig`, `SharedStep`, `CacheConfig`,
  `CheckoutConfig`, `dag_to_shared_steps`
- `WorkflowConfig` rendering from `core/ir/src/transport/github_actions.rs`
  if it has no live callers

Keep:

- `CiProvider`
- provider command formatting
- runner catalogs
- provider detection used by runtime/terminal CI command output

Acceptance:

- repo CI files are still generated by `gunbc-codegen cigen`
- no live code path renders CI YAML from Rust
- grep for `CiRenderer|SharedStep|RenderConfig|WorkflowConfig` in live Rust code
  returns zero matches, except in historical docs if retained

### Phase 2: Make `tools.cigen.dag` structurally compositional

This phase replaces string-heavy YAML assembly inside `tools/cigen.dag` with
typed provider-value assembly.

Do:

- add typed DAG constructors/helpers for GitHub/GitLab provider values
- build `Workflow` / `Pipeline` values in `tools.cigen.dag`
- move provider-specific layout decisions into typed constructors or typed data
- keep the final YAML serialization as the last leaf

Do not:

- reintroduce a Rust-side CI renderer

Acceptance:

- `tools.cigen.dag` builds typed provider values before serialization
- `render_github_workflow()` and `render_gitlab_pipeline()` no longer own policy
- YAML rendering helpers are leaf serialization helpers only

### Phase 3: Replace raw `CiDiscovery` strings with typed step data

Move from:

- `tool_command: String`
- `bootstrap_script: String?`

to typed data such as:

- `ci_run: CiCommand`
- `bootstrap: List<CiCommandLine>`
- `secrets: List<String>`

or an equivalent typed step/input model.

Acceptance:

- no raw multi-line shell script crosses the Rust/DSL boundary for CI generation
- the DSL can validate and compose CI steps structurally

## What Is Blocking Us?

### Not blocking

Nothing fundamental blocks Phase 1.

The repo already generates CI from the DSL path, and searches today show the
Rust `CiRenderer` / `WorkflowConfig` path is not the source of truth for repo
CI output.

Deleting that dead path is cleanup, not a compiler research problem.

### Medium-scope work, but not a hard blocker

Phase 2 is also not blocked by missing language features.

The repo already has:

- typed provider schemas in `.dag`
- `config.ci` as data
- `content_upsert`
- enough string/list composition to build typed intermediate values

What remains is straightforward modeling work:

- stop assembling YAML too early
- add typed provider-value constructors/helpers
- narrow the extern discovery boundary

### Real open design decisions

These need decisions, but they are not blockers to starting:

1. What is the typed replacement for `CiDiscovery.tool_command`?
   Recommendation: model commands as structured argv/lines, not raw shell text.

2. Should YAML itself get a richer structured IR?
   Recommendation: not required for this lane. First move CI policy and step
   composition into typed provider values; only then decide whether the final
   YAML leaf needs a richer IR.

3. Should provider command formatting (`CiProvider`) also move into `.dag`?
   Recommendation: no, not in this cut. Runtime CI command formatting is a
   different surface from CI YAML artifact generation.

## Recommended Immediate Cut

Done now:

1. Delete `render.rs` and the Rust CI YAML rendering path.
2. Delete `WorkflowConfig`.
3. Add a contract test that CI YAML is generated only through `tools/cigen.dag`
   and that no Rust CI renderer symbols remain in live code.

Do next:

1. Refactor `tools/cigen.dag` to build typed provider values.
2. Replace `CiDiscovery` raw strings with typed step data.

## Why The Repo Does Not Already Look Like This

Because the repo completed the "move generation to DSL" step before finishing
the "delete the Rust bridge" step.

That is common migration debt, not evidence that the compositional approach is
wrong or blocked. The current state is simply a transitional shape that outlived
its usefulness.

## Acceptance Criteria

This lane is complete when:

1. Repository CI YAML generation has exactly one implementation path:
   `config.ci -> tools.cigen -> content_upsert`.
2. No live Rust code renders CI YAML.
3. `tools.cigen.dag` builds typed provider values before final serialization.
4. CI discovery crosses the Rust/DSL boundary as structure, not raw shell text.
