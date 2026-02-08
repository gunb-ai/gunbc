# DAG Workflow Audit

**Date:** 2026-02-06

**Scope:** All DAG graph definitions across the workspace — binary workflows, library tools, workspace composition, and advanced patterns.

---

## Overview

The codebase uses a custom DAG execution framework (`gunbc-ir` + `gunbc-exec`) where graphs are built from typed nodes connected by ports and edges. All business logic lives in **pure** nodes; I/O is isolated to **transport** nodes. Graphs are composed via SubDags, loops, and resource acquisition.

**Totals:**
- 7 binary workflows
- 5 library tool graphs (gist, deps, clippy, review, llm-chat)
- ~15 distinct graph builder functions
- ~150+ nodes across all graphs

---

## Binary Workflows

All binary workflows live in `gunbc-dag/src/` and are registered via `#[tool_target]`.

### 1. Codegen (`codegen/graph.rs`)

**Builder:** `build_codegen_graph()` / `build_codegen_graph_with_mode(mode: ExecMode)`

**Purpose:** Run code generation and write a stamp file.

**Pipeline (8 nodes, 11 edges):**
```
PrepareCodegenExists -> ExecuteCodegenExists -> ParseCodegenExists
                                                       |
                                           PrepareCodegenCommand -> ExecuteCodegen -> ParseCodegenResult
                                                                                            |
                                                                                PrepareStampWrite -> ExecuteStampWrite
```

**Op enum:** `CodegenGraphOp = Codegen(CodegenOp) | Transport(TransportOps)`

**Notes:** Conditional execution — skips if codegen is already fresh. ExecMode threaded through.

---

### 2. Bootstrap (`bootstrap/graph.rs`)

**Builder:** `build_bootstrap_graph()`

**Purpose:** Scan workspace crates and generate Makefile + .gitignore files.

**Pipeline (15 nodes, 21 edges):**
```
PrepareScan -> ExecuteScan -> ParseScanResult
                                     |
                     +---------------+----------------+
                     v                                v
          GenerateMakefile                  GenerateGitignore
               |                                     |
        (Makefile upsert chain)           (Gitignore upsert chain)
```

Each upsert chain: `Generate -> PrepareRead -> ExecuteRead -> Compare -> PrepareWrite -> ExecuteWrite`

**Op enum:** `BootstrapGraphOp = Bootstrap(BootstrapOp) | PrepareFileRead | PrepareFileWrite | Blob(BlobOps) | Transport`

**Notes:** Two parallel upsert chains. Content comparison skips writes when files are fresh.

---

### 3. Build (`build/graph.rs`)

**Builder:** `build_build_graph()`

**Purpose:** Build -> Test + Clippy (parallel) -> Summary report.

**Pipeline (10 nodes):**
```
PrepareBuild -> ExecuteBuild -> ParseBuild
                                    |
                    +---------------+----------------+
                    v                                v
  PrepareTest -> ExecuteTest -> ParseTest   PrepareClippy -> ExecuteClippy -> ParseClippy
                    |                                                          |
                    +--------------------------+-------------------------------+
                                               v
                                           Summary
```

**Op enum:** `BuildGraphOp = Build(BuildOp) | Transport(TransportOps)`

**Notes:** Test and Clippy run in parallel after build completes. Summary fans in both results.

---

### 4. Makegen (`makegen/graph.rs`)

**Builder:** `build_makegen_graph()`

**Purpose:** Generate a Makefile from the tool registry.

**Pipeline (7 nodes, 9 edges):**
```
LoadRegistry -> RenderMakefile -> PrepareFileRead -> ExecuteRead -> CompareContent -> PrepareFileWrite -> ExecuteWrite
```

**Op enum:** `MakegenGraphOp = Makegen(MakegenOp) | PrepareFileRead | PrepareFileWrite | Blob(BlobOps) | Transport`

**Notes:** Standard content upsert — skips write if generated content matches existing file.

---

### 5. CI (`ci/graph.rs`)

**Builder:** `build_ci_graph()` / `build_ci_graph_with_mode(mode: ExecMode)`

**Purpose:** Full CI pipeline orchestrating all other tools.

**Pipeline (many nodes):**
```
SetupDeps: PrepareFileExists -> Execute -> ParseDepsExists
Prep:      (Inlined Codegen DAG) -> ParseCodegenResult
           -> PrepareTestgenCommand -> Execute -> ParseTestgenResult
Build:     PrepareBuildCommand -> Execute -> ParseBuildResult
Test:      PrepareTestCommand -> Execute -> ParseTestResult       (parallel)
Lint:      PrepareClippyLint -> ClippyLint -> ParseClippyLint    (parallel)
Guards:    PrepareGuardrailCheck -> Execute -> ParseGuardrailResult
Verify:    PrepareVerifyCheck -> Execute -> ParseVerifyResult
Report:    Report (pure fan-in)
```

**Op enum:** `CIGraphOp = CI(CIOp) | Codegen(CodegenOp) | PrepareFileExists(EmbeddedFileExistsOp) | Transport | CliTool(CliToolOp)`

**Notes:** Largest graph. Inlines the codegen DAG directly. Clippy uses a self-acquiring CLI tool node (`upsert_tool_with()`). Test + Lint run in parallel after build.

---

### 6. Pragma (`pragma/graph.rs`)

**Builder:** `build_pragma_graph()`

**Purpose:** Generate three policy files: clippy.toml, disallowed-methods allowlist, pragma-lint policy.

**Pipeline (18 nodes, 24 edges):**
Three independent upsert chains running in parallel, one per file:
```
RenderClippy    -> (upsert chain)
RenderAllowlist -> (upsert chain)
RenderPolicy    -> (upsert chain)
```

**Op enum:** `PragmaGraphOp = Pragma(PragmaOp) | PrepareFileRead | PrepareFileWrite | Blob(BlobOps) | Transport`

**Notes:** All three chains are parallel roots — no inter-dependencies.

---

### 7. Testgen (`testgen_dag/graph.rs`)

**Builder:** `build_testgen_graph(targets: &[&TestgenTarget], output_dir: &Path)`

**Purpose:** Generate test files for N targets discovered from the tool registry.

**Pipeline (N x 6 nodes, N x 8 edges):**
```
For each target:
  Generate_{name} -> (upsert chain for that target's test file)
```

**Op enum:** `TestgenGraphOp = Testgen(TestgenOp) | PrepareFileRead | PrepareFileWrite | Blob(BlobOps) | Transport`

**Notes:** Dynamic graph construction — chain count depends on how many testgen targets are registered. All chains are independent.

---

## Library Tool Graphs

### 8. Gist (`lib/tools/gist/src/graph.rs`)

**Builders:**
- `build_gist_graph(mode: GistMode, extensions: Vec<String>, public: bool)`
- Three modes: Snapshot, Diff, Recent

**Snapshot mode (17 nodes, 21 edges):**
```
PrepareLsFiles -> ExecuteListFiles -> ParseLsFiles -> LoopBuilder(ReadFileBody) -> CollectFileContents -> RenderMarkdown
  (parallel)  PrepareCurrentBranch -> ExecuteCurrentBranch -> ParseCurrentBranch
  (parallel)  PrepareRemoteBranches -> ExecuteRemoteBranches -> ParseRemoteBranches
                                                                       |
                                                   PrepareGistRequest -> ExecuteGist -> ParseGistResponse
```

**Loop body DAG:** `PrepareReadFile -> Execute -> ParseReadFile` (per file)

**Resources:** `fs:write`, `clock`

**Notes:** Only workflow using `LoopBuilder` for runtime iteration. Diff/Recent modes substitute the file-reading loop with git-diff chains.

---

### 9. Deps (`lib/tools/deps/src/graph.rs`)

**Builders:**
- `build_deps_graph()` — install dependencies
- `build_deps_generate_graph()` — generate deps.toml

**Install graph (8 nodes):**
```
PlatformEnv -> PrepareLoadManifest -> ExecuteLoadManifest -> ParseManifest -> GenerateScripts -> PrepareExecuteInstalls -> ExecuteInstalls -> ParseExecuteResult
```

**Generate graph (4 nodes):**
```
LoadToolRegistry -> RenderDepsToml -> PrepareFileWrite -> ExecuteTransport
```

**Resources:** Platform resource (cross-platform tool resolution)

**Op enum:** `DepsGraphOp = Deps(DepsOp) | Env(PlatformEnv) | PrepareFileWrite | Transport`

---

### 10. Clippy (`lib/tools/clippy/src/graph.rs`)

**Builder:** `build_clippy_upsert(args: &[&str])` — returns a `Node<CliToolOp>` (SubDag)

**SubDag structure (3 nodes):** `Check -> Create -> Resolve`

**Notes:** Generic CLI tool upsert pattern. `build_clippy_lint_all()` is a preset with `--all-targets -- -D warnings`.

---

### 11. Review (`lib/review/src/graph.rs`)

**Builders:**
- `build_review_phase_graph()` — review a blob with LLM
- `build_inline_review_graph()` — review inline content
- `build_diff_review_graph()` — review git diff
- `build_multi_source_review_graph()` — multi-source with merge

**Review phase (10 nodes):**
```
PrepareFetch -> ExecuteBlob -> ParseFetch -> PreparePrompt -> PrepareLlm -> ResolveAuth -> CredentialEnv -> ExecuteLlm -> ParseLlm -> ParseResponse
```

**Diff review (12 nodes):**
```
Config -> PrepareDiff -> ExecuteDiff -> ParseDiff -> FormatArtifact -> (LLM chain) -> ParseResponse
```

**Resources:** Credential resource (LLM API keys)

**Op enum:** `ReviewGraphOp = Blob(BlobOps) | Git(GitOps) | Review(ReviewOps) | Llm(LlmOps) | CloudEnv | Cloud(CloudSecretManagerGraphOp) | Transport`

---

### 12. LLM Chat (`lib/llm-ops/src/graph.rs`)

**Builder:** `build_chat_completion_graph()`

**Pipeline (5 nodes):**
```
PrepareChatRequest -> ResolveAuth -> CredentialEnv -> Execute -> ParseChatResponse
```

**Resources:** Credential resource (provider-specific API keys)

**Op enum:** `LlmGraphOp = Llm(LlmOps) | Transport | CloudEnv | Cloud(CloudSecretManagerGraphOp)`

**Notes:** Embeddable SubDAG used by the Review tool.

---

## Workspace Composition (`gunbc-dag/src/workspace/`)

The workspace layer wraps all tool graphs into a unified `WorkspaceOp` enum for fractal DAG composition:

```
WorkspaceOp = Ci | Codegen | Deps | DepsEnv | Makegen | Gist | Bootstrap | Clippy | Language | Primitive | Transport
```

SubDAG builders: `build_bootstrap_subdag()`, `build_ci_subdag()`, `build_clippy_subdag()`, `build_deps_install_subdag()`, `build_deps_generate_subdag()`, `build_gist_subdag()`, `build_languages_subdag()`, `build_makegen_subdag()`

These allow embedding any tool as a single node in larger orchestration DAGs.

---

## Recurring Patterns

### Content Upsert (read-compare-skip)

Used by: Bootstrap, Makegen, Pragma, Testgen

```
Generate -> PrepareRead -> ExecuteRead -> Compare(BlobOps) -> PrepareWrite -> ExecuteWrite(skippable)
```

Compares generated content against existing file; skips the write transport if content matches.

### Transport Triplet

Used by: CI (extensively)

```
Prepare(pure) -> Execute(transport) -> Parse(pure)
```

Helper: `add_transport_triplet()` / `add_skippable_transport_triplet()`

### LoopBuilder

Used by: Gist (snapshot mode)

```
LoopUnpack -> (body SubDag per element) -> LoopPack
```

Runtime iteration with per-element body DAG execution. Element injected via `set_input()`.

### SubDag Composition

Used by: Clippy, workspace layer, loop bodies

Embeds a complete DAG as a single node. Port auto-inference from inner DAG boundaries. Resource access propagated automatically.

### Resource Acquisition

Environment nodes provide resources with explicit access modes:
- `PlatformEnv` — platform detection
- `FsEnv` — filesystem scope
- `ClockEnv` — timestamps
- `CloudEnv` + `CloudSecretManagerGraphOp` — API credentials (via cloud secret manager)

Access modes: `Read` (shared), `Write` (exclusive), `Exclusive` (no other access). Validated by `derive_resource_accesses()` and `validate_resource_wiring_recursive()`.

---

## File Locations

| Workflow | File |
|----------|------|
| Codegen | `gunbc-dag/src/codegen/graph.rs` |
| Bootstrap | `gunbc-dag/src/bootstrap/graph.rs` |
| Build | `gunbc-dag/src/build/graph.rs` |
| Makegen | `gunbc-dag/src/makegen/graph.rs` |
| CI | `gunbc-dag/src/ci/graph.rs` |
| Pragma | `gunbc-dag/src/pragma/graph.rs` |
| Testgen | `gunbc-dag/src/testgen_dag/graph.rs` |
| Gist | `lib/tools/gist/src/graph.rs` |
| Deps | `lib/tools/deps/src/graph.rs` |
| Clippy | `lib/tools/clippy/src/graph.rs` |
| Review | `lib/review/src/graph.rs` |
| LLM Chat | `lib/llm-ops/src/graph.rs` |
| Workspace | `gunbc-dag/src/workspace/ops.rs` + `subdags/*.rs` |
| DAG IR | `core/ir/src/dag.rs`, `builder.rs`, `node.rs` |
| Executor | `core/exec/src/execute.rs`, `lower.rs` |
| Tool Registry | `core/tool-registry/src/lib.rs` |
