# gunbc

A causal engine for programs. Validates that declared intent is
sound — every data flow has a valid source and drain, every
computation terminates, every type is consistent — then emits
to any target language as mechanical translation.

**If it compiles, the intent is sound and will execute as declared.**

See [THESIS.md](THESIS.md) for the full thesis.

## Quick start

```bash
git clone <repo> && cd gunbc
cargo build --release -p v2-compiler --bin gunbc
cargo test -p v2-compiler-tests    # compiler tests
cargo clippy --all-targets -- -D warnings  # lint
```

The release binary is `target/release/gunbc`. Use it to compile v4 trees, for example:

```bash
./target/release/gunbc compile --source-root src/v4
```

## How it works

Programs are written in `.dag` — a closed language where all data
is finite, all iteration is bounded, and composition preserves
boundedness. The compiler validates the causal graph from source
to drain:

```
.dag source → tokenize → parse → resolve → infer → emit → target code
```

Every stage is a pure transform. The compiler never executes the
programs it validates. Emission is mechanical: Rust, Python, Go
today — any target that can represent the primitives.

A single `.dag` program can describe an entire system. Different
subgraphs emit to different targets. The compiler owns the glue
between artifacts (shared types, serialization, API contracts).

## v2 and v4

| | v2 (`src/v2/`) | v4 (`src/v4/`) |
|---|----------------|----------------|
| Role | Production self-hosted compiler | Next substrate + pipeline |
| Maturity | Emit to Rust/Python/Go; large test suite | Model depth in `std/` and `extdeps/`; compiler stages compile `.dag` |
| Source | `.dag` + small Rust `stage0` bootstrap | `.dag` only in the compiler tree (bootstrap shrinking) |

**v4 status (honest):** the v4 compiler pipeline compiles and type-checks `.dag` over `src/v4` in CI. Multi-target emission and execute-verified `TestClaim` runners are in progress. See [ROADMAP.md](ROADMAP.md) for milestone shape.

## What the compiler proves

| Property | How |
|----------|-----|
| Type safety | Structural type checking at every binding |
| Exhaustiveness | Every match covers every variant |
| Termination | Structural descent proof for every recursive function |
| Coercion completeness | Fail-closed algebra inhabitant lookup per target |
| Ownership | Binding fan-out analysis, clone/move decisions |
| Cross-target consistency | All targets derive from the same declarations |

What it can't prove — external reality (does the REST endpoint
exist? is the database up?) — it generates tests for.

## Project structure

```
THESIS.md           Why this project exists — start here
ROADMAP.md          Current state and direction
INVARIANTS.md       Five principles that protect the thesis
MODELING.md         How to extend the language safely

docs/               Project-wide design
  architecture.md     Substrate: Node + Edge
  coercion-design.md  Type coercion algebra

dsl/                Portable domain vocabulary
  std/                Shared types, algebra, iteration
  extdeps/            External system models (cloud, git, shell)

src/v2/             Self-hosted compiler (.dag source, shipping)
  00_core.dag         Core types
  02_parse.dag        Tokenizer + parser
  04_infer.dag        Type inference + provenance
  05_emit.dag         Emission (shared + per-target)
  complexity.dag      Termination proofs
  ownership.dag       Ownership analysis

src/v4/             Next compiler generation (in progress)
  std/                Substrate vocabulary
  extdeps/            Language and transport models
  compiler/           Pipeline stages (tokenize … emit … translate)
  lens/               Correctness lenses
  test/claim/         Structural TestClaim corpus

src/v2/tests/       v2 compiler test suite
  testing-strategy.md Testing philosophy
```

## Documentation

Read top-down. Each doc links to its parent and children.

1. **[THESIS.md](THESIS.md)** — the goal: causal soundness
2. **[ROADMAP.md](ROADMAP.md)** — what's done, what's next
3. **[INVARIANTS.md](INVARIANTS.md)** — rules that enforce soundness
4. **[MODELING.md](MODELING.md)** — how to model new concepts
5. Design docs in `docs/` and `src/v2/` — drill into specifics
