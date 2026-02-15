# DSL Golden Examples

Standalone `.dag` files that define what the gunbc DSL looks like for every real workflow. These are **spec examples** (golden targets for the parser), not runnable code -- the compiler doesn't exist yet.

## Purpose

1. **Parser golden targets** -- the first thing the compiler must parse
2. **Parity test anchors** -- each `.dag` file maps 1:1 to a Rust graph builder
3. **Modeling validation** -- prove the DSL syntax handles all real patterns

## Reading order

Start with the foundation, then work outward:

```
std/          Foundation: types, resources, patterns
services/     Pure declarations: typed I/O + transport annotations
cloud/        Credential acquisition (uses services + patterns)
tools/        Journeys that compose everything above
pipelines/    Multi-stage composition of tools
```

### Recommended first read

1. `std/types.dag` -- see all the types
2. `std/resources.dag` -- Filesystem, Network, Clock, AuthContext
3. `std/patterns.dag` -- content_upsert, credential_chain (the reusable shapes)
4. `tools/makegen.dag` -- simplest tool (~30 lines, uses content_upsert)
5. `tools/gist.dag` -- complex tool (4 modes, loops, service composition)
6. `pipelines/ci.dag` -- everything wired together

## Rust mapping

Each `.dag` file replaces a specific Rust graph builder:

| DSL file | Rust file | Rust LOC |
|---|---|---|
| `tools/makegen.dag` | `gunbc-dag/src/makegen/graph.rs` | 220 |
| `tools/gist.dag` | `lib/tools/gist/src/graph.rs` | 1,449 |
| `tools/dag_viz.dag` | `gunbc-dag/src/dag_viz/graph.rs` | 1,347 |
| `tools/clippy.dag` | `lib/tools/clippy/src/graph.rs` | 186 |
| `tools/deps.dag` | `lib/tools/deps/src/graph.rs` | ~200 |
| `tools/bootstrap.dag` | `gunbc-dag/src/bootstrap/graph.rs` | ~300 |
| `tools/codegen.dag` | `gunbc-dag/src/codegen/graph.rs` | ~200 |
| `tools/testgen.dag` | `gunbc-dag/src/testgen_dag/graph.rs` | ~200 |
| `tools/pragma.dag` | `gunbc-dag/src/pragma/graph.rs` | ~300 |
| `tools/build.dag` | `gunbc-dag/src/build/graph.rs` | ~250 |
| `tools/docgen.dag` | `gunbc-dag/src/docgen/graph.rs` | ~500 |
| `cloud/gcp/credential.dag` | `lib/gcp-ops/src/graph.rs` | 1,700+ |
| `pipelines/ci.dag` | `gunbc-dag/src/ci/graph.rs` | ~600 |

**Total: ~7,500 lines of Rust graph builders replaced by ~700 lines of DSL.**

## File inventory

### `std/` -- Standard library

- **`types.dag`** -- Primitives, refinement types, domain enums, records. Everything from `Unit` to `DagTopology`.
- **`resources.dag`** -- `Filesystem`, `Network`, `Clock`, `AuthContext`. Resource lifecycle declarations with capabilities.
- **`patterns.dag`** -- `content_upsert`, `upsert`, `emit`, `credential_chain`, `transaction`, `retry`. The reusable DAG shapes that tools compose.

### `services/` -- Service declarations

- **`git.dag`** -- `git.Core`: CurrentBranch, RemoteBranches, LsFiles, Diff, RevList, Show
- **`cargo.dag`** -- `cargo.Build`: Build, Test, Clippy, Doc, Run
- **`shell.dag`** -- `gcloud.Auth`, `oauth2.Google`, `shell.Find`, `shell.Codegen`, `rustup.Component`, `shell.Which`
- **`github/gist.dag`** -- `github.Gist`: Create (REST API + mock_response)
- **`gcp/secret_manager.dag`** -- `gcp.SecretManager`: AccessVersion, CreateSecret, AddVersion
- **`gcp/iam.dag`** -- `gcp.IAM`: GenerateAccessToken; `gcp.ResourceManager`: Get/SetIamPolicy
- **`gcp/sts.dag`** -- `gcp.STS`: Exchange; `github.OIDC`: GetToken; `gcp.Metadata`: GetIdentityToken

### `cloud/` -- Cloud provider integration

- **`gcp/credential.dag`** -- `acquire_gcp_secret` journey. Wraps `credential_chain` pattern. Entry point for all GCP-authenticated tools.

### `tools/` -- Tool journeys

- **`makegen.dag`** -- Simplest: `render_makefile` fn + `content_upsert` pattern
- **`gist.dag`** -- 4 modes: `gist_upload`, `gist_snapshot`, `gist_diff`, `gist_recent`. Loops, service composition, credential chain.
- **`dag_viz.dag`** -- 4 modes: `dag_viz_snapshot`, `dag_viz_diff`, `dag_viz_recent`, `dag_viz_save`. Meta-tool that introspects the DAG system.
- **`clippy.dag`** -- `upsert` pattern (check/install/resolve) + lint run
- **`deps.dag`** -- Dependency install (platform-aware loop) + config generation
- **`bootstrap.dag`** -- Workspace scan + 2 parallel content_upserts (Makefile, .gitignore)
- **`codegen.dag`** -- Conditional execution: check stamp, run if stale, write stamp
- **`testgen.dag`** -- Dynamic parallel: N targets, each with independent content_upsert
- **`pragma.dag`** -- 3 parallel content_upserts (clippy.toml, allowlist, policy)
- **`build.dag`** -- `cargo build` then parallel `cargo test` + `cargo clippy`, aggregate
- **`docgen.dag`** -- Read 13 source files, render single doc, content_upsert

### `pipelines/` -- Multi-stage pipelines

- **`ci.dag`** -- 12-stage CI pipeline composing all tools with `after` dependencies, parallel groups, conditional execution (`when`), and aggregate reporting.

## Collection operations as IR nodes

A key design property: collection operations (`map`, `filter`, `fold`, `join`, etc.) inside `fn` bodies are **not** compiled as opaque function calls. The compiler lowers them to IR-level collection nodes (`MapNode`, `FilterNode`, `FoldNode`, `JoinNode`, etc.) whose inner transforms are scalar functions.

This means every program is a complete dataflow graph -- nothing is hidden. The executor can parallelize `MapNode` across workers, stream `MapNode -> FilterNode` without materializing intermediates, and fuse trivial adjacent maps into single passes.

Two kinds of parallelism are visible in the IR:
- **Task-parallel**: journey-level `for` loops (each iteration has I/O)
- **Data-parallel**: `fn`-level `|> map/filter/fold` (each element is a pure transform)

See `dsl-design.md` section 4.2 for the full two-tier model.

## Design doc

These files are the concrete instantiation of the language spec in `docs/design/v4/dsl-design.md`. The spec defines the grammar; these files prove it works for every real workflow.
