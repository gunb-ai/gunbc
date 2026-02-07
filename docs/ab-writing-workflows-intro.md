# Graph-First Workflows: A Short Introduction (Clippy + Gist)

gunbc models workflows as **typed DAGs**. You build a graph, validate it, then execute it in:
- `DryRun`: intercept boundary nodes (no real I/O)
- `Real`: run with actual transports

What you get:
- The diagram and the code are the same artifact.
- Workflow wiring stays correct by construction (less hand-written wiring tests).
- Failures localize to nodes and boundaries instead of hidden control flow.

What validation guarantees:
- Acyclic workflow
- Type + cardinality compatibility on edges
- SubDag interfaces match how they are used
- Resource ports are fully wired (no dangling handles)

You still test semantics (parsers/renderers) and boundary behavior (transport errors/auth).

## Example 1 (Minimal): Clippy Upsert

Clippy = Rust linter distributed as a rustup component.

```text
Build-time:  args: [String]
Runtime:     trigger: Unit
Output:      result: CliResult

dag = build_clippy_graph(args)
{result} = execute(dag, {trigger: ()})
```

gunbc modeling: a SubDag node that packages check -> install -> run. Validation proves the wiring; tests cover semantics and boundary behavior.

## Example 2 (Real): Gist Snapshot

Gist snapshot = turn a repo's files into a GitHub gist.

```text
Build-time:  mode = Snapshot, extensions, public
Runtime:     repo_path: String
Output:      url: String

dag = build_gist_graph(Snapshot, extensions, public)
{url} = execute(dag, {repo_path})
```

gunbc modeling: explicit transport boundaries, a loop SubDag, and generated artifacts (signature, MockSpec, tests, CLI).

## Read the Full Version

For the complete comparisons, diagrams, and generated artifacts, see:
- `docs/ab-writing-workflows.md`
