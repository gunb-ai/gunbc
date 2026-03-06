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

`CiDiscovery` must stop returning shell-text fragments like:

- `tool_command: String`
- `bootstrap_script: String?`

The chosen replacement is a shared CI script model:

- `ScriptLine = Command { argv: List<String> } | Raw { shell: String } | Comment { text: String }`
- `ScriptBlock { lines: List<ScriptLine> }`
- `CiDiscovery { secrets: List<String>, bootstrap: ScriptBlock?, run: ScriptBlock }`

Rules:

- `Command` is the preferred form for invocations we can represent as argv.
- `Raw` is allowed only for shell structures not yet modeled structurally
  (for example, a `for ...; do` loop).
- `Comment` preserves explanatory lines without collapsing them into opaque
  shell text.

That model is required for the full compositional end state.

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
- add leaf serializer modules over provider schemas:
  - `extdeps/github_actions_render.dag`
  - `extdeps/gitlab_ci_render.dag`
- build `Workflow` / `Pipeline` values in `tools.cigen.dag`
- move provider-specific layout decisions into typed constructors or typed data
- keep the final YAML serialization as the last leaf

Do not:

- reintroduce a Rust-side CI renderer

Acceptance:

- `tools.cigen.dag` builds typed provider values before serialization
- `tools.cigen.dag` no longer owns provider YAML layout
- provider YAML rendering lives in leaf serializer modules only

### Phase 3: Replace raw `CiDiscovery` strings with typed step data

Move from:

- `tool_command: String`
- `bootstrap_script: String?`

to typed data such as:

- `run: ScriptBlock`
- `bootstrap: ScriptBlock?`
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

### Resolved design decisions

1. Typed replacement for `CiDiscovery.tool_command`
   Chosen: shared `ScriptBlock` / `ScriptLine` model, not raw shell text and
   not argv-only. The bootstrap step contains real shell control flow, so the
   bridge must be able to represent both structured argv commands and rare raw
   shell lines.

2. Where provider YAML rendering lives
   Chosen: provider YAML serialization becomes a leaf module over the provider
   schema, not inline logic inside `tools.cigen.dag`.

   Recommended module split:
   - `extdeps/github_actions.dag` = provider schema + pure constructors
   - `extdeps/github_actions_render.dag` = YAML leaf serializer
   - `extdeps/gitlab_ci.dag` = provider schema + pure constructors
   - `extdeps/gitlab_ci_render.dag` = YAML leaf serializer

3. Whether YAML itself needs a richer universal IR first
   Chosen: no. This lane stops at typed provider values plus provider leaf
   serializers. A universal YAML DOM can be evaluated later if still useful.

4. Whether `CiProvider` runtime command formatting moves in this cut
   Chosen: no. Runtime CI command formatting is a different surface from CI
   artifact generation and is not a blocker for this lane.

5. How shell quoting is handled
   Chosen: `Command { argv }` is rendered through a shared shell-quoting helper
   at serialization time. `Raw { shell }` is emitted verbatim and must stay
   rare, documented, and test-covered.

## Execution Plan

### Step 1: Introduce shared CI script types

Add a new shared module for provider-neutral CI script structure, for example
`extdeps/ci_script.dag`, with:

- `ScriptLine = Command { argv: List<String> } | Raw { shell: String } | Comment { text: String }`
- `ScriptBlock { lines: List<ScriptLine> }`
- `render_script_line_shell()`
- `render_script_block_lines()`

Acceptance:

- the model can represent both the current cargo invocation and the bootstrap
  shell loop without collapsing them into one multiline string
- shell quoting for `Command.argv` is centralized in one helper

### Step 2: Promote provider serialization to leaf modules

Add pure serializer modules that consume typed provider values and emit YAML:

- `extdeps/github_actions_render.dag`
- `extdeps/gitlab_ci_render.dag`

These modules may depend on:

- `extdeps.yaml`
- provider schema types
- shared CI script rendering helpers

Acceptance:

- provider YAML layout is owned by provider leaf modules, not by `tools.cigen.dag`
- serializer inputs are typed `Workflow` / `Pipeline` values

### Step 3: Move static CI policy into typed provider templates

Evolve `config.ci.dag` from a bag of loose scalars toward typed base values:

- `ci_github_base_workflow: Workflow`
- `ci_gitlab_base_pipeline: Pipeline`

Keep separately exported leaf constants only where they are not part of provider
schema shape (for example generator metadata or output paths).

Acceptance:

- static GitHub/GitLab policy is represented as typed provider values
- `tools.cigen.dag` imports a small number of typed bases instead of many
  independent policy scalars

### Step 4: Replace the raw discovery bridge with typed script data

Change the extern boundary in `gunbc-app/src/extern_ops.rs` and `tools/cigen.dag`
to emit:

- `secrets: List<String>`
- `bootstrap: ScriptBlock?`
- `run: ScriptBlock`

Guidance:

- use `Command { argv }` for the cargo invocations derived from
  `CargoInvocation::command_parts()` / `run_with_args()`
- use `Raw { shell }` only for the bootstrap loop and similarly irreducible
  shell syntax
- preserve explanatory comments as `Comment { text }`

Acceptance:

- no raw multiline script blob crosses the Rust/DSL boundary
- cargo command structure is preserved as argv where possible

### Step 5: Rewrite `tools.cigen.dag` as typed assembly only

After Steps 1-4, `tools.cigen.dag` should:

- import typed provider schemas and typed base config values
- import typed discovery output
- build GitHub `Step` / `Job` / `Workflow` values
- build GitLab `Job` / `Pipeline` values
- call provider leaf serializers
- write outputs via `content_upsert`

It should no longer:

- contain provider-specific YAML indentation/layout logic
- own raw `render_github_workflow()` / `render_gitlab_pipeline()` string builders
- manipulate bootstrap/run shell content as opaque multiline strings

Acceptance:

- `tools.cigen.dag` reads like assembly/composition, not a YAML template engine

### Step 6: Add ratchet tests for the new architecture

Keep the existing CI cache-path contract tests and add:

- a unit/compile test proving `discover_ci_config` now yields typed script data
- a pure render test for GitHub workflow serialization from typed `Workflow`
- a pure render test for GitLab pipeline serialization from typed `Pipeline`
- a grep-based acceptance check that `tools/cigen.dag` no longer contains the
  provider-specific YAML render helper family

Acceptance:

- reintroducing raw-string CI discovery or inline provider YAML layout fails
  tests quickly

### Step 7: Apply the same pattern to adjacent render lanes

After the CI lane is closed, use it as the reference pattern for:

- `dsl/tools/makegen.dag`
- `dsl/tools/justgen.dag`
- `core/codegen/src/cli_gen.rs`
- `gunbc-app/src/ci/ops.rs`
- `lib/markdown/src/lib.rs`
- `lib/design-ops/src/lib.rs`

That is follow-on work, not a blocker for closing the CI lane itself.

## Recommended Immediate Cut

Done now:

1. Delete `render.rs` and the Rust CI YAML rendering path.
2. Delete `WorkflowConfig`.
3. Add a contract test that CI YAML is generated only through `tools/cigen.dag`
   and that no Rust CI renderer symbols remain in live code.

Do next:

1. Introduce shared CI script types and provider leaf serializers.
2. Promote static CI policy to typed base provider values in `config.ci`.
3. Replace `CiDiscovery` raw strings with typed script data.
4. Rewrite `tools.cigen.dag` as typed assembly only.
5. Add ratchet tests so the cleanup cannot regress silently.

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
5. Provider YAML layout lives in leaf serializer modules, not in `tools.cigen.dag`.
