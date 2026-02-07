# A/B Writing Workflows: Minimal to Real (Clippy + Gist)

This doc shows two A/B comparisons: a minimal tool workflow (clippy upsert) and a real workflow (gist snapshot). Each example is written in three traditional styles and then modeled as a gunbc DAG, with proofs and generated artifacts called out.

Example 1 (minimal): clippy upsert
- Goal: ensure clippy is installed, then run it
- Build-time config: `args: [String]` (CLI flags for the run step)
- Runtime input: `trigger: Unit`
- Output: `result: CliResult`

Example 2 (real): gist snapshot
- Mode: Snapshot (list files, read contents, render code blocks, create gist)
- Input: `repo_path: String`
- Output: `url: String`
- I/O boundaries: git commands and gist creation via explicit transport execution

## Core Difference: Graph-First Programs

Instead of writing the program function-by-function and relying on control flow, gunbc models the **program itself as a DAG**. The structure is explicit, validated, and compiled into a form that can be checked and test-generated. This makes wiring, boundaries, and dataflow a first-class, provable artifact.

## Proofs by Construction (No Tests Needed)

When a DAG validates, the following structural properties are proven by construction:
- The workflow is acyclic.
- All edges are type-compatible.
- All edges are cardinality-compatible.
- SubDag interfaces match their parent usage.
- Entrypoints and boundaries are inferred structurally from connectivity.
- Resource inputs can be validated as fully wired (no dangling resource ports).

---

## A. Minimal Tool: clippy upsert

### A.1 Imperative (Rust-style)

```rust
fn run_clippy(args: &[&str]) -> Result<()> {
    if !clippy_installed()? {
        install_clippy()?;
    }
    run_clippy_command(args)?;
    Ok(())
}
```

Helper implementations (simple sketch):

```rust
fn clippy_installed() -> Result<bool> {
    // Call `rustup component list --installed` or `clippy-driver --version`
    // and return true if clippy is present.
    Ok(true)
}

fn install_clippy() -> Result<()> {
    // Run `rustup component add clippy` and return error on failure.
    Ok(())
}

fn run_clippy_command(args: &[&str]) -> Result<()> {
    // Execute `cargo clippy {args...}` and return error on non-zero exit.
    Ok(())
}
```

What you get:
- Straightforward control flow.

What you must ensure manually:
- The check/install/run wiring is consistent and complete.
- The "already installed" fast path is correct.

---

### A.2 OO (Java-style)

```java
final class ClippyRunner {
    private final ToolInstaller installer;
    private final ToolRunner runner;

    void run(String[] args) throws IOException {
        if (!installer.isInstalled("clippy")) {
            installer.install("clippy");
        }
        runner.run("clippy", args);
    }
}
```

What you get:
- Clear seams for testing and substitution.

What you must ensure manually:
- The upsert pattern stays consistent across call sites.

---

### A.3 Functional (Haskell-style)

```haskell
runClippy :: [String] -> IO ()
runClippy args = do
  installed <- clippyInstalled
  unless installed installClippy
  runClippyCommand args
```

What you get:
- Explicit effects and composability.

What you must ensure manually:
- The upsert structure is preserved and tested.

---

### A.4 gunbc DAG (Clippy Upsert)

Clippy uses the generic CLI upsert pattern. The builder returns a **SubDag node** containing the check -> install -> run flow.

```rust
use gunbc_ir::node::Node;
use gunbc_ir::transport::cli::{self, build_cli_upsert, CliToolOp};

pub fn build_clippy_upsert(args: &[&str]) -> Node<CliToolOp> {
    build_cli_upsert(&cli::CLIPPY, args)
}
```

Shape of the sub-DAG (simplified):

```text
+--------------------- clippy (SubDag) ---------------------+
| [check] -> [create] -> [resolve]                          |
|    \-------------------- already installed ---------------/ |
+-----------------------------------------------------------+
```

Actual code (core upsert builder in `core/ir/src/transport/cli.rs`):

```rust
pub fn build_cli_upsert(tool: &'static CliToolDef, args: &[&str]) -> Node<CliToolOp> {
    UpsertBuilder::new(tool.id)
        .with_check(CliToolOp::check(tool))   // is the tool installed?
        .with_create(CliToolOp::install(tool)) // if not, install it
        .with_resolve(CliToolOp::run(tool, args)) // then run the tool
        .with_input_port("trigger", "Unit")
        .with_output_port("result", "CliResult")
        .build()
}
```

What gunbc compilation proves (beyond Rust/Java compilers):
- The upsert flow is acyclic and structurally complete.
- All edges are type-compatible and cardinality-compatible.
- The SubDag interface matches how the parent graph uses it.

Real run (DAG execution):

```rust
use gunbc_exec::{execute_with_mode, ExecutionMode};
use gunbc_clippy::build_clippy_dag;

let dag = build_clippy_dag(&["--all-targets", "--", "-D", "warnings"]);
let _log = execute_with_mode(&dag, ExecutionMode::Real)?;
```

Tests (clippy):
- Unit tests exist for clippy tooling (see Appendix B for the test index).
- No generated tests yet: the clippy upsert is a sub-DAG node and not registered as a testgen target.
- Sample test code: Appendix A.

---

## B. Real Workflow: gist snapshot

Other gist modes exist (Diff, Recent), but the example below stays on Snapshot to keep the comparison tight.

### B.1 Imperative (Rust-style)

```rust
fn run_gist_snapshot(repo_path: &Path, public: bool) -> Result<String> {
    let files = git_ls_files(repo_path)?;

    let mut contents = BTreeMap::new();
    for file in files {
        let text = std::fs::read_to_string(repo_path.join(&file))?;
        contents.insert(file, text);
    }

    let markdown = render_code_snapshot(&contents);

    let branch = git_current_branch(repo_path)?;
    let remote_branch = git_remote_branches_at_head(repo_path)?;

    let request = prepare_gist_request(markdown, branch, remote_branch, None, public)?;
    let response = execute_transport(request)?;
    let url = parse_gist_response(response)?;

    Ok(url)
}
```

Helper stubs (commented intent):

```rust
fn git_ls_files(repo_path: &Path) -> Result<Vec<String>> {
    // Run `git ls-files` in repo_path and parse into a list of file paths.
    Ok(vec![])
}

fn render_code_snapshot(contents: &BTreeMap<String, String>) -> String {
    // Render file contents into fenced code blocks.
    String::new()
}

fn git_current_branch(repo_path: &Path) -> Result<Option<String>> {
    // Run `git rev-parse --abbrev-ref HEAD` and return branch name if available.
    Ok(None)
}

fn git_remote_branches_at_head(repo_path: &Path) -> Result<Option<String>> {
    // Run `git branch -r --points-at HEAD` and pick a remote branch if present.
    Ok(None)
}

fn prepare_gist_request(
    markdown: String,
    branch: Option<String>,
    remote_branch: Option<String>,
    base_ref: Option<String>,
    public: bool,
) -> Result<TransportRequest> {
    // Construct an HTTP request payload for gist creation.
    unimplemented!()
}

fn execute_transport(req: TransportRequest) -> Result<TransportResponse> {
    // Perform the I/O boundary (HTTP request) and return a response.
    unimplemented!()
}

fn parse_gist_response(resp: TransportResponse) -> Result<String> {
    // Parse JSON and extract the gist URL.
    unimplemented!()
}
```

What you get:
- Familiar, direct control over ordering and error handling.
- Full freedom to wire dependencies however you like.

What you must ensure manually:
- The workflow structure is acyclic and complete.
- I/O boundaries are isolated, mocked, and tested correctly.
- Optional inputs and skip paths are handled consistently.

---

### B.2 OO (Java-style)

```java
final class GistWorkflow {
    private final GitClient git;
    private final Renderer renderer;
    private final GistClient gist;
    private final FileSystem fs;

    String runSnapshot(Path repoPath, boolean isPublic) throws IOException {
        List<String> files = git.lsFiles(repoPath);
        Map<String, String> contents = new HashMap<>();
        for (String f : files) {
            contents.put(f, fs.readString(repoPath.resolve(f)));
        }
        String markdown = renderer.renderCodeSnapshot(contents);

        String branch = git.currentBranch(repoPath);
        String remoteBranch = git.remoteBranchesAtHead(repoPath);

        GistRequest request = gist.prepare(markdown, branch, remoteBranch, null, isPublic);
        GistResponse response = gist.execute(request);
        return gist.parseUrl(response);
    }
}
```

What you get:
- Dependency injection seams for testing.
- Clear object boundaries.

What you must ensure manually:
- The order and completeness of the workflow wiring.
- Exhaustive tests for success and per-boundary failures.

---

### B.3 Functional (Haskell-style)

```haskell
runSnapshot :: RepoPath -> Bool -> IO Url
runSnapshot repoPath isPublic = do
  files     <- gitLsFiles repoPath
  contents  <- readFiles repoPath files
  markdown  <- renderCodeSnapshot contents
  branch    <- gitCurrentBranch repoPath
  remote    <- gitRemoteBranchesAtHead repoPath
  response  <- executeTransport (prepareGistRequest markdown branch remote Nothing isPublic)
  pure (parseGistResponse response)
```

What you get:
- Explicit effect boundaries and clean composition.

What you must ensure manually:
- The graph structure is well-formed (acyclic, compatible, complete).
- The transport boundary behavior and skip paths are fully tested.

---

### B.4 gunbc DAG (Gist Snapshot)

#### DAG structure (real nodes, simplified view)

Snapshot mode in `lib/tools/gist/src/graph.rs` builds this structure:

```text
[prepare_list_files] -> [execute_list_files] -> [parse_list_files] -> [read_files_loop]
[read_files_loop] -> [collect_file_contents] -> [render_markdown] -> [prepare_gist_request] -> [execute_gist] -> [parse_gist_response] -> url

[prepare_current_branch] -> [execute_current_branch] -> [parse_current_branch] --branch--> [prepare_gist_request]
[prepare_remote_branches] -> [execute_remote_branches] -> [parse_remote_branches] --remote_branch--> [prepare_gist_request]

[fs_env] --fs:write--> [prepare_gist_request]
[clock_env] --clock--> [prepare_gist_request]

read_files_loop = LoopBuilder(
  [prepare_read_file] -> [execute_read_file] -> [parse_read_file]
)
```

Real code (excerpt from `lib/tools/gist/src/graph.rs`):

```rust
let prepare_gist_request = builder.add_node_after(
    Node::opaque(
        "prepare_gist_request",
        vec![
            scalar("markdown", "String"),
            optional("branch", "String"),
            optional("remote_branch", "String"),
            optional("base_ref", "String"),
            resource("fs", "FilesystemHandle", AccessMode::Read),
            resource("clock", "Timestamp", AccessMode::Read),
        ],
        vec![scalar("request", "TransportRequest"), scalar("skip", "Bool")],
        GistGraphOp::Gist(GistOps::PrepareRequest { public }),
    ),
    &render_markdown,
)?;

builder.add_edge(
    render_markdown.out("markdown"),
    prepare_gist_request.in_port("markdown"),
)?;
builder.add_edge(
    current_branch.parse.out("branch"),
    prepare_gist_request.in_port("branch"),
)?;
builder.add_edge(
    remote_branches.parse.out("remote_branch"),
    prepare_gist_request.in_port("remote_branch"),
)?;
builder.add_edge(
    fs_env.out("fs:write"),
    prepare_gist_request.in_port("res:fs"),
)?;
builder.add_edge(
    clock_env.out("clock"),
    prepare_gist_request.in_port("res:clock"),
)?;
```

Key points:
- All I/O is concentrated in `TransportOps::Execute` nodes.
- The file read loop is a SubDag with its own transport boundary.
- `fs_env` and `clock_env` provide resource inputs to `prepare_gist_request`.
- `prepare_gist_request` consumes: `markdown`, `branch?`, `remote_branch?`, `base_ref?`, `res:fs`, and `res:clock`.

Real run (DAG execution):

```rust
use gunbc_exec::{execute_with_mode, ExecutionMode};
use gunbc_lib_gist::graph::{build_gist_graph, GistMode};

let dag = build_gist_graph(GistMode::Snapshot, vec![], false)?;
let _log = execute_with_mode(&dag, ExecutionMode::Real)?;
```

#### What gunbc compilation proves (beyond Rust/Java compilers)

These are graph-level guarantees that general-purpose compilers do not provide:
- The workflow is acyclic.
- All edges are type-compatible and cardinality-compatible.
- SubDag interfaces (like the loop body) match their parent usage.
- Entrypoints and boundaries are inferred structurally from connectivity.
- Resource wiring is validated so resource inputs are not left dangling.

#### Generated artifacts and tests (gist snapshot)

From this DAG, gunbc generates:
- A workflow signature (mode-dependent) that is validated against the DAG.
- A typed MockSpec extracted from the DAG structure in `lib/tools/gist/src/graph_mock.rs`.
- A generated test suite in `lib/tools/gist/src/generated_tests_snapshot.rs`.

Full generated test index (names + descriptions): Appendix B.
Sample generated tests: Appendix A.

Selected generated tests include:
- Signature validation (declared signature matches inferred DAG).
- DryRun completion (smoke test).
- Transport interception for each `execute_*` boundary.
- Per-transport failure scenarios and skip propagation.
- Optional input handling for ports with cardinality `0..1` or `0..*`.
- Window tests that execute contiguous subgraphs derived from the DAG.

Example shape from current output:

```rust
// Generated by gunbc-testgen
// Proven by construction: acyclicity, type compatibility, cardinality satisfaction.

#[test]
fn test_transport_interception() {
    // ... asserts all execute_* nodes are intercepted in DryRun
}
```

---

## Why this matters (short version)

Traditional code makes workflow structure implicit. gunbc makes it explicit, validated, and test-generated. You still write the business logic, but the workflow wiring, I/O boundaries, and many scenario tests become mechanically derived.

---

## Appendix A: Sample Tests

### A.1 Clippy (unit test)

```rust
#[test]
fn test_clippy_upsert_is_subdag() {
    let node = build_clippy_upsert(&["--all-targets"]);
    assert_eq!(node.id.0, "clippy");

    assert!(
        matches!(node.body, NodeBody::SubDag(_)),
        "Expected SubDag, got {:?}",
        node.body
    );
}
```

### A.2 Gist (generated test: DryRun completion)

```rust
#[test]
fn test_dryrun_completion() {
    if !guard_test("test_dryrun_completion", TestClass::Hermetic, FermiCost::S, &["shell"], &[]) {
    return ();
};
    let dag = crate ::
build_gist_graph(crate :: GistMode :: Snapshot, vec! [], false).unwrap();
    let log = execute_with_mode(&dag, ExecutionMode::DryRun(mock_spec().to_boundary_mocks())).expect("DryRun execution should complete without crash");
    assert!(!log.entries.is_empty(), "execution should produce log entries");
}
```

### A.3 Gist (generated test: Transport interception)

```rust
#[test]
fn test_transport_interception() {
    if !guard_test("test_transport_interception", TestClass::Hermetic, FermiCost::S, &["shell"], &[]) {
    return ();
};
    let dag = crate ::
build_gist_graph(crate :: GistMode :: Snapshot, vec! [], false).unwrap();
    let result = assert_boundary_mockable(&dag, mock_spec().to_boundary_mocks());
    assert!(result.is_ok(), "All transports should be interceptable: {:?}", result.error);
    assert!(result.boundary_nodes.iter().any(|n| n == "execute_list_files"), "transport executor 'execute_list_files' should be in intercepted list");
    assert!(result.boundary_nodes.iter().any(|n| n == "execute_current_branch"), "transport executor 'execute_current_branch' should be in intercepted list");
    assert!(result.boundary_nodes.iter().any(|n| n == "execute_remote_branches"), "transport executor 'execute_remote_branches' should be in intercepted list");
    assert!(result.boundary_nodes.iter().any(|n| n == "execute_gist"), "transport executor 'execute_gist' should be in intercepted list");
}
```

## Appendix B: Test Index

### B.1 Clippy Unit Tests (non-generated)
- test_lint_id_allow_name — LintId renders correct #[allow(...)] name for rustc and clippy.
- test_policy_to_allowance — CratePolicy with allow_disallowed_methods yields a CrateAllowance.
- test_policy_without_allowance — CratePolicy without allowance yields None.
- test_clippy_upsert_is_subdag — build_clippy_upsert returns a SubDag node named clippy.
- test_clippy_lint_all_has_correct_args — build_clippy_lint_all returns the clippy node.
- test_clippy_dag_structure — build_clippy_dag produces a single-node DAG.
- test_subdag_contains_upsert_nodes — clippy sub-DAG contains check/create/resolve nodes.
- test_clippy_check_op — Clippy::check uses tool id clippy.
- test_clippy_lint_all_op — Clippy::lint_all produces a Run op with --all-targets.
- test_disallowed_method_creation — DisallowedMethod stores path and reason.
- test_crate_allowance_creation — CrateAllowance stores crate name and reason.
- test_clippy_config_builder — ClippyConfig builder collects disallow/allow entries.
- test_transport_pattern_preset — transport_pattern preset includes expected disallows/allowances.
- test_generate_clippy_toml — generate_clippy_toml emits expected sections and methods.
- test_render_implementation — ClippyConfigRenderer renders header and content.

### B.2 Gist Snapshot Generated Tests
- test_signature_matches_dag — Declared signature matches the DAG.
- test_dryrun_completion — DryRun execution completes without crash.
- test_transport_interception — All transport executors are intercepted in DryRun.
- test_optional_missing_collect_file_contents_filenames — Optional input collect_file_contents.filenames missing should not error.
- test_optional_wrong_type_collect_file_contents_filenames — Optional input collect_file_contents.filenames wrong type should error.
- test_optional_missing_collect_file_contents_contents_list — Optional input collect_file_contents.contents_list missing should not error.
- test_optional_wrong_type_collect_file_contents_contents_list — Optional input collect_file_contents.contents_list wrong type should error.
- test_optional_missing_prepare_gist_request_branch — Optional input prepare_gist_request.branch missing should not error.
- test_optional_wrong_type_prepare_gist_request_branch — Optional input prepare_gist_request.branch wrong type should error.
- test_optional_missing_prepare_gist_request_remote_branch — Optional input prepare_gist_request.remote_branch missing should not error.
- test_optional_wrong_type_prepare_gist_request_remote_branch — Optional input prepare_gist_request.remote_branch wrong type should error.
- test_optional_missing_prepare_gist_request_base_ref — Optional input prepare_gist_request.base_ref missing should not error.
- test_optional_wrong_type_prepare_gist_request_base_ref — Optional input prepare_gist_request.base_ref wrong type should error.
- test_scenario_all_succeed — Scenario: all transports succeed.
- test_scenario_execute_list_files_fails — Scenario: execute_list_files transport fails.
- test_scenario_execute_current_branch_fails — Scenario: execute_current_branch transport fails.
- test_scenario_execute_remote_branches_fails — Scenario: execute_remote_branches transport fails.
- test_scenario_execute_gist_fails — Scenario: execute_gist transport fails.
- test_skip_propagation_execute_list_files — Skip propagation when execute_list_files returns Skipped.
- test_skip_propagation_execute_current_branch — Skip propagation when execute_current_branch returns Skipped.
- test_skip_propagation_execute_remote_branches — Skip propagation when execute_remote_branches returns Skipped.
- test_skip_propagation_execute_gist — Skip propagation when execute_gist returns Skipped.
- test_boundaries_mockable — All boundary nodes are mockable in DryRun.
- test_boundary_parse_gist_response_mockable — Boundary parse_gist_response is mockable.
- test_mock_spec_self_consistent — MockSpec is self-consistent against the DAG.
- test_input_expectations_documented — Input expectations are declared in the MockSpec.
- test_window_read_files_loop_unpack_through_read_files_loop_pack — Window: read_files_loop/unpack through read_files_loop/pack.
- test_window_read_files_loop_pack_through_collect_file_contents — Window: read_files_loop/pack through collect_file_contents.
- test_window_collect_file_contents_through_render_markdown — Window: collect_file_contents through render_markdown.
- test_window_render_markdown_through_prepare_gist_request — Window: render_markdown through prepare_gist_request.
- test_window_prepare_gist_request_through_execute_gist — Window: prepare_gist_request through execute_gist.
- test_window_execute_gist_through_parse_gist_response — Window: execute_gist through parse_gist_response.
- test_window_read_files_loop_unpack_through_collect_file_contents — Window: read_files_loop/unpack through collect_file_contents.
- test_window_read_files_loop_pack_through_render_markdown — Window: read_files_loop/pack through render_markdown.
- test_window_collect_file_contents_through_prepare_gist_request — Window: collect_file_contents through prepare_gist_request.
- test_window_render_markdown_through_execute_gist — Window: render_markdown through execute_gist.
- test_window_prepare_gist_request_through_parse_gist_response — Window: prepare_gist_request through parse_gist_response.
- test_window_read_files_loop_unpack_through_render_markdown — Window: read_files_loop/unpack through render_markdown.
- test_window_read_files_loop_pack_through_prepare_gist_request — Window: read_files_loop/pack through prepare_gist_request.
- test_window_collect_file_contents_through_execute_gist — Window: collect_file_contents through execute_gist.
- test_window_render_markdown_through_parse_gist_response — Window: render_markdown through parse_gist_response.
- test_window_read_files_loop_unpack_through_prepare_gist_request — Window: read_files_loop/unpack through prepare_gist_request.
- test_window_read_files_loop_pack_through_execute_gist — Window: read_files_loop/pack through execute_gist.
- test_window_collect_file_contents_through_parse_gist_response — Window: collect_file_contents through parse_gist_response.
- test_window_parse_remote_branches_through_prepare_gist_request — Window: parse_remote_branches through prepare_gist_request.
- test_window_read_files_loop_unpack_through_execute_gist — Window: read_files_loop/unpack through execute_gist.
- test_window_read_files_loop_pack_through_parse_gist_response — Window: read_files_loop/pack through parse_gist_response.
- test_window_parse_list_files_through_prepare_gist_request — Window: parse_list_files through prepare_gist_request.
- test_window_parse_remote_branches_through_execute_gist — Window: parse_remote_branches through execute_gist.
- test_window_read_files_loop_unpack_through_parse_gist_response — Window: read_files_loop/unpack through parse_gist_response.
- test_window_parse_current_branch_through_prepare_gist_request — Window: parse_current_branch through prepare_gist_request.
- test_window_parse_list_files_through_execute_gist — Window: parse_list_files through execute_gist.
- test_window_parse_remote_branches_through_parse_gist_response — Window: parse_remote_branches through parse_gist_response.
- test_window_execute_remote_branches_through_prepare_gist_request — Window: execute_remote_branches through prepare_gist_request.
- test_window_parse_current_branch_through_execute_gist — Window: parse_current_branch through execute_gist.
- test_window_parse_list_files_through_parse_gist_response — Window: parse_list_files through parse_gist_response.
- test_window_execute_list_files_through_prepare_gist_request — Window: execute_list_files through prepare_gist_request.
- test_window_execute_remote_branches_through_execute_gist — Window: execute_remote_branches through execute_gist.
- test_window_parse_current_branch_through_parse_gist_response — Window: parse_current_branch through parse_gist_response.
- test_window_execute_current_branch_through_prepare_gist_request — Window: execute_current_branch through prepare_gist_request.
- test_window_execute_list_files_through_execute_gist — Window: execute_list_files through execute_gist.
- test_window_execute_remote_branches_through_parse_gist_response — Window: execute_remote_branches through parse_gist_response.
- test_window_prepare_remote_branches_through_prepare_gist_request — Window: prepare_remote_branches through prepare_gist_request.
- test_window_execute_current_branch_through_execute_gist — Window: execute_current_branch through execute_gist.
- test_window_execute_list_files_through_parse_gist_response — Window: execute_list_files through parse_gist_response.
- test_window_prepare_list_files_through_prepare_gist_request — Window: prepare_list_files through prepare_gist_request.
- test_window_prepare_remote_branches_through_execute_gist — Window: prepare_remote_branches through execute_gist.
- test_window_execute_current_branch_through_parse_gist_response — Window: execute_current_branch through parse_gist_response.
- test_window_prepare_current_branch_through_prepare_gist_request — Window: prepare_current_branch through prepare_gist_request.
- test_window_prepare_list_files_through_execute_gist — Window: prepare_list_files through execute_gist.
- test_window_prepare_remote_branches_through_parse_gist_response — Window: prepare_remote_branches through parse_gist_response.
- test_window_fs_env_through_prepare_gist_request — Window: fs_env through prepare_gist_request.
- test_window_prepare_current_branch_through_execute_gist — Window: prepare_current_branch through execute_gist.
- test_window_prepare_list_files_through_parse_gist_response — Window: prepare_list_files through parse_gist_response.
- test_window_clock_env_through_prepare_gist_request — Window: clock_env through prepare_gist_request.
- test_window_fs_env_through_execute_gist — Window: fs_env through execute_gist.
- test_window_prepare_current_branch_through_parse_gist_response — Window: prepare_current_branch through parse_gist_response.
- test_window_clock_env_through_execute_gist — Window: clock_env through execute_gist.
- test_window_fs_env_through_parse_gist_response — Window: fs_env through parse_gist_response.
- test_window_clock_env_through_parse_gist_response — Window: clock_env through parse_gist_response.
- test_example_fs_env_provides_filesystem_handle_for_gist_filename_generation — Node example: fs_env — provides filesystem handle for gist filename generation.
- test_example_clock_env_provides_timestamp_for_gist_filename_generation — Node example: clock_env — provides timestamp for gist filename generation.
- test_example_prepare_current_branch_prepares_git_rev_parse_request_for_current_branch — Node example: prepare_current_branch — prepares git rev parse request for current branch.
- test_example_parse_current_branch_parses_current_branch_name_from_git_output — Node example: parse_current_branch — parses current branch name from git output.
- test_example_prepare_remote_branches_prepares_git_branch_r_points_at_head_request — Node example: prepare_remote_branches — prepares git branch r points at head request.
- test_example_parse_remote_branches_parses_remote_branch_name_from_git_output — Node example: parse_remote_branches — parses remote branch name from git output.
- test_example_prepare_gist_request_builds_gist_creation_request_from_markdown — Node example: prepare_gist_request — builds gist creation request from markdown.
- test_example_parse_gist_response_extracts_gist_url_from_response_json — Node example: parse_gist_response — extracts gist url from response json.
- test_example_prepare_list_files_prepares_git_ls_files_request — Node example: prepare_list_files — prepares git ls files request.
- test_example_parse_list_files_parses_git_ls_files_output_into_a_file_list — Node example: parse_list_files — parses git ls files output into a file list.
- test_example_collect_file_contents_zips_filenames_contents_into_a_map_skipping_empty_content — Node example: collect_file_contents — zips filenames contents into a map skipping empty content.
- test_example_render_markdown_renders_markdown_code_snapshot — Node example: render_markdown — renders markdown code snapshot.
