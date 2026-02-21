# Strategy: Everything is a DAG (Daglang Projections)

This document explores the data boundaries and configuration formats currently holding state across the `gunbc` repository. The overarching architectural philosophy of this project is **"Everything is a DAG"**, meaning tools, schemas, features, runtimes, and artifacts should all be authored as nodes in `.dag` files and *projected* into reality (e.g. GitHub issues, SQLite databases, cloud infrastructure) via a compiler model. 

Currently, several domains still use static YAML, JSON ledgers, or manual Rust coordination. 

This is an audit of those domains and how we will transition them to pure Daglang.

---

## 1. Feature Intents & Issue Management
**Current State**: Authored as isolated flat files in YAML (`TODO/issue-intent-template.yaml`).
**The Problem**: A flat YAML file has no native concept of dependencies. If we have a mega-feature (e.g., "Full Auth Revamp"), the tasks are just independent `.yaml` intents. The SDLC worker has no context that "Token Generation" must be implemented before "User Login View". It's just a loose collection of issues.

**The Daglang Solution**:
Convert `intents` to a `dsl/projects/` taxonomy.
Instead of running `gunbc-sdlc intake --intent intent.yaml`, we run `daglang project apply dsl/projects/auth.dag`.

```dag
module projects.auth_revamp

feature generate_tokens {
    title = "JWT Token Generation"
    objective = "Securely generate and sign user JWTs"
    acceptance_tests = ["verify JWT signature output"]
}

feature login_view {
    title = "Implement Login UI"
    objective = "Provide user inputs for email and password"
    depends_on = [generate_tokens] // Explicit dependency mapping
}
```
**Benefits**:
The Orchestrator engine will not spawn the SDLC workflow (Code generation) on `login_view` until the Artifact Ledger proves `generate_tokens` is transitioned to `Stage: Closed/Merged`.

**Downstream Translation (Native GitHub Integration)**:
When `daglang project apply` compiles the graph, it doesn't just create isolated issues. It translates `.dag` dependencies into the native format of the downstream provider. For GitHub, this means:
- Emitting standard linking keywords in the issue body (e.g. `Depends on #12`, `Blocks #14`).
- Automatically generating Markdown checklists for sub-tasks (e.g., `- [ ] #15`) in Epic/Feature parent issues.
- Leveraging GitHub's native "Tracked Issues" API to reflect the exact DAG topology in the GitHub UI, ensuring human developers and AI agents share identical context without needing to open the CLI.

---

## 2. Infrastructure Deployment Intents
**Current State**: Authored as YAML (`TODO/infra-intent-template.yaml`) mapping out runtime dependencies (`claim_store`, `outcome_ledger`, secrets, and topologies).
**The Problem**: YAML has no ability to evaluate system health before provisioning. It also relies on a custom Rust parser (`gunbc-dag/src/bin/infra.rs`) to reconcile the JSON state mappings.

**The Daglang Solution**:
Treat infrastructure nodes the same way we define `build` pipelines. Cloud services, SQlite paths, and worker scale are typed components.

```dag
module infra.sdlc_runtime

resource claim_store_db {
    backend = "sqlite"
    dsn = "var/sdlc/claims.db"
    fail_closed = true
}

resource sdlc_orchestrator {
    worker_count = 5
    depends_on = [claim_store_db]
    secrets = [github_token, openai_key]
}
```
**Benefits**: Infrastructure drift detection becomes a standard DAG resolution path. If the DAG output fails, it flags `Needs Rebuild` via the standard `daglang` toolset instead of a disconnected SDLC binary.

---

## 3. SDLC Artifact & Run State Ledgers
**Current State**: Custom JSON-blob state files mapped via Serde inside `target/sdlc/`. 
**The Problem**: Ledgers (`intake-ledger.json`, `artifact-ledger.json`) act as disjointed databases. While the `SDLC` worker operates safely on these, the broader workspace execution engine (`WorkspaceOp`) cannot observe or query them safely as dependencies without parsing JSON inside Rust.

**The Daglang Solution**:
Bring State definitions into the DAG DSL via specialized `data` nodes, and project the SQLite or JSON cache layer identically to how we handle compilation caches. We can build a `Resolver` phase in `gunbc-exec` that handles reading/verifying these states safely before running workflow ticks.

## Next Steps

To action this holistic transition into Daglang projections, the logical execution steps are:

1. **Daglang Primitives Update**
   - Introduce new domain types in `core/ir/`. The compiler needs primitives for `Project`, `Feature`, `Resource`, and `DataRecord`.
2. **Implement Project Resolution Engine**
   - Extend `gunbc-dag/src/bin/sdlc.rs` to parse `.dag` targets rather than flat `.yaml`.
   - Update `reconcile_entries` logic: Use the topological sort of the graph to filter which issues are "blocking" versus "actionable" when generating the GitHub issue payloads.
3. **Deprecate Flat Targets**
   - Delete `TODO/*.yaml` intents in favor of tracking internal roadmaps entirely in `dsl/projects/`. 
   - Tie the compilation of these graphs directly to GitHub actions.
