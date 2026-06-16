# daglang

A language for programs as dependency graphs. You write `.dag` source: types,
data, and workflows wired by explicit causes and drains. The language is
closed—finite data, bounded iteration, and composition that preserves
boundedness—so structural claims are checkable, not conventional.

**If it compiles, the declared intent is sound and will execute as declared.**

See [THESIS.md](THESIS.md) for the full thesis behind that guarantee, and
[ROADMAP.md](ROADMAP.md) for where the project stands and the active wave of work.

## Language and compiler

| | **daglang** | **gunbc** |
|---|-------------|-----------|
| What | The `.dag` language, `dsl/std/` vocabulary, and substrate models | The compiler that validates daglang and emits target code |
| You | Author `.dag` programs | Run `gunbc` on a source tree |

```
.dag source  →  gunbc  →  tokenize → parse → resolve → infer → emit  →  target code
```

gunbc is a causal engine: it checks that every flow has a valid source and
drain, every computation terminates, and every type is consistent—then emits
to any target language as mechanical translation. Stages are pure transforms;
gunbc does not execute your program to validate it.

A single daglang program can describe an entire system. Different subgraphs
emit to different targets; gunbc owns the glue (shared types, serialization,
API contracts).

## Quick start

### Install a release binary (Linux musl or macOS)

After a tagged release (`v0.1.0` or later) is published on GitHub Releases:

```bash
curl -fsSL https://github.com/gunb-ai/gunbc/releases/latest/download/install.sh | sh
```

Pin a version (assign on the **shell** side of the pipe — not on `curl`):

```bash
curl -fsSL https://github.com/gunb-ai/gunbc/releases/latest/download/install.sh | GUNBC_VERSION=v0.1.0 sh
```

Or run a local copy: `GUNBC_VERSION=v0.1.0 sh install.sh`

Windows hosts are not covered by `install.sh` (POSIX only). Download the matching
`gunbc-*-pc-windows-msvc.exe` asset from the release page, or build from source below.

### Build from source

Build the compiler, compile the hero `.dag` to Rust, and check the emitted
crate — end-to-end in four commands:

```bash
git clone https://github.com/gunb-ai/daglang.git && cd daglang
cargo build --release -p v1-compiler --bin gunbc

./target/release/gunbc compile \
  --source-root dsl/examples/weather \
  --source-root dsl/std \
  --output-dir /tmp/weather-out \
  --target rust

cargo check --manifest-path /tmp/weather-out/Cargo.toml
```

The source is [`dsl/examples/weather/weather.dag`](dsl/examples/weather/weather.dag):
domain types, a coproduct (`Condition = Sunny | Cloudy | Rainy | Snowy`), pattern
matching, and list pipelines. gunbc emits a self-contained Rust crate; `cargo
check` succeeding is proof the emitted code is well-typed with zero hand glue.

Swap `--target rust` for `python`, `go`, or `dag` to retarget the same source.
**Honest scope for v0.1.0:** compile to Rust today (`cargo check` above); Python
and Go emit currently fail for programs using match-as-expression /
nested-if-as-expression (hero demo `weather.dag` is affected); fixes in flight,
expected v0.1.1.
The release binary is `target/release/gunbc` (crate `v1-compiler`, bin defined
in `src/v1/stage0/Cargo.toml`).

```bash
cargo test -p v1-compiler-tests             # compiler tests
cargo clippy --all-targets -- -D warnings   # lint
```

## Compiler status (v1, v2)

The tree holds two compiler generations. **Quick Start above runs v1 today.** Active
work is in v2. (The former v3 generation has been removed.)

| | **v1** | **v2** |
|---|--------|--------|
| Path | [`src/v1/`](src/v1/) | [`src/v2/`](src/v2/) |
| Role | Production compiler (`gunbc` CLI) | Next substrate + pipeline |
| Source | `.dag` pipelines + shrinking Rust `stage0` | `.dag` in the compiler tree; v1 is the bootstrap seed |
| What works today | Parse, infer, emit to Rust/Python/Go; large test suite | `std/` / `extdeps/` model depth; pipeline `.dag` structurally compiles in CI |

**v1 — shipping.** Self-hosted from `.dag` (tokenize through emit, plus complexity and
ownership). Use `gunbc` with `dsl/std/` and `dsl/extdeps/` for your programs. The weather
demo is v1 end-to-end.

**v2 — in progress (honest).** v2 combines a typed Node + Behavior substrate with a
full pipeline (tokenize → parse → resolve → infer → emit → translate). In CI, v1
`gunbc` compiles and type-checks all of `src/v2` with zero diagnostics. Runnable v2
stage0, full multi-target emission, execute-verified structural tests, and the self-host
fixed point are still landing — **v1 remains the reference for end-to-end emit until v2
closes that loop.** See [ROADMAP.md](ROADMAP.md).

## What gunbc proves

| Property | How |
|----------|-----|
| Type safety | Structural type checking at every binding |
| Exhaustiveness | Every match covers every variant |
| Termination | Structural descent proof for every recursive function |
| Coercion completeness | Fail-closed algebra inhabitant lookup per target |
| Ownership | Binding fan-out analysis, clone/move decisions |
| Cross-target consistency | All targets derive from the same declarations |

What it can't prove—external reality (does the REST endpoint exist? is the
database up?)—it generates tests for.

## Project structure

```
THESIS.md           Why daglang exists — start here
ROADMAP.md          Current state and direction
INVARIANTS.md       Five principles that protect the thesis
MODELING.md         How to extend the language safely

docs/               Project-wide design
  architecture.md     Substrate: Node + Edge
  coercion-design.md  Type coercion algebra

dsl/                Portable daglang vocabulary
  std/                Shared types, algebra, iteration
  extdeps/            External system models (cloud, git, shell)

src/v1/             gunbc compiler (.dag source, shipping)
  00_core.dag         Core types
  02_parse.dag        Tokenizer + parser
  04_infer.dag        Type inference + provenance
  05_emit.dag         Emission (shared + per-target)
  complexity.dag      Termination proofs
  ownership.dag       Ownership analysis

src/v3/             Frozen predecessor (reference; not shipped)

src/v2/             Next compiler generation (in progress)
  std/                Substrate vocabulary
  extdeps/            Language and transport models
  compiler/           Pipeline stages (tokenize … emit … translate)
  lens/               Correctness lenses
  test/claim/         Structural TestClaim corpus

src/v1/tests/       v2 compiler test suite
  testing-strategy.md Testing philosophy
```

## Documentation

Read top-down. Each doc links to its parent and children.

1. **[THESIS.md](THESIS.md)** — the goal: causal soundness for daglang
2. **[ROADMAP.md](ROADMAP.md)** — what's done, what's next
3. **[INVARIANTS.md](INVARIANTS.md)** — rules that enforce soundness
4. **[MODELING.md](MODELING.md)** — how to model new concepts
5. Design docs in `docs/` and `src/v1/` — drill into specifics
