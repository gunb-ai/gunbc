# DSL-First Workflows: A Short Introduction (Clippy + Gist)

gunbc is a **DSL-first workflow compiler** where everything is a DAG. You write `.dag` files, the compiler validates and lowers them to typed Graph IR, then you execute them in:
- `DryRun`: intercept boundary nodes (no real I/O)
- `Real`: run with actual transports

What you get:
- The `.dag` definition and the runtime graph are the same artifact.
- Workflow wiring stays correct by construction (less hand-written wiring tests).
- Failures localize to nodes and boundaries instead of hidden control flow.
- Adding a new service or tool requires only a `.dag` file — zero hand-written Rust.

What the compiler guarantees:
- Acyclic workflow
- Type + cardinality compatibility on edges
- SubDag interfaces match how they are used
- Resource ports are fully wired (no dangling handles)
- Service operations are type-checked against their annotations

You still test semantics (parsers/renderers) and boundary behavior (transport errors/auth).

## Example 1 (Minimal): Clippy Upsert

Clippy = Rust linter distributed as a rustup component.

DSL definition (`dsl/tools/clippy.dag`):

```
func clippy_lint(paths: List<String>?) -> { clean: Bool, findings: String }
  uses clippy: Clippy
{
  tool = upsert(
    check: clippy.check(),
    create: clippy.install(),
    resolve: clippy.resolve()
  )
  result = cargo.Build.Clippy() [after tool]
  return { clean: result.success, findings: result.stderr }
}
```

The compiler lowers this to a SubDag node that packages check -> install -> run. Validation proves the wiring; tests cover semantics and boundary behavior.

## Example 2 (Real): Gist Snapshot

Gist snapshot = turn a repo's files into a GitHub gist.

Service definition (`dsl/services/github/gist.dag`):

```
service github.Gist {
  @endpoint("https://api.github.com")
  @auth(BearerToken)

  operation Create {
    input { description: String, files: Map<String, String>, public: Bool = false }
    output { url: Url @json("html_url"), id: GistId }
    @rest(POST, "/gists")
  }
}
```

Tool workflow (`dsl/tools/gist.dag`):

```
func gist_snapshot(base_ref: CommitSha?) -> { url: Url }
  uses fs: Filesystem(mode: Read)
{
  ctx = branch_context()
  files = git.Core.LsFiles()
  read_result = read_text_files(paths: files.files)
  markdown = render_snapshot(files: read_result.files)
  result = share_content(markdown: markdown, branch: ctx.branch, base_ref: base_ref)
  return { url: result.url }
}
```

The compiler generates explicit transport boundaries, a loop SubDag for file reads, and all the `prepare → execute → parse` triplets from service calls. MockSpec, tests, and CLI are generated from the compiled Graph IR.

## Read the Full Version

For the complete comparisons, diagrams, and generated artifacts, see:
- `docs/ab-writing-workflows.md`
