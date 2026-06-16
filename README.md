# daglang

A language for programs as dependency graphs. You write `.dag` source: types,
data, and workflows wired by explicit causes and drains. The language is
closed—finite data, bounded iteration, and composition that preserves
boundedness—so structural claims are checkable, not conventional.

**If it compiles, the declared intent is sound and will execute as declared.**

See [DESIGN.md](DESIGN.md) for the worldview, principles, and modeling discipline behind that
guarantee, and [ROADMAP.md](ROADMAP.md) for where the project stands and the active wave of work.

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
cargo build --release -p v2-compiler --bin gunbc

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
The release binary is `target/release/gunbc` (crate `v2-compiler`, bin defined
in `src/v2/stage0/Cargo.toml`).

```bash
cargo test -p v2-compiler-tests             # compiler tests
cargo clippy --all-targets -- -D warnings   # lint
```

## Compiler status (v2, v3, v4)

The tree holds three compiler generations. **Quick Start above runs v2 today.** Active
work is in v4. v3 is frozen reference material, not a supported path.

| | **v2** | **v3** | **v4** |
|---|--------|--------|--------|
| Path | [`src/v2/`](src/v2/) | [`src/v3/`](src/v3/) | [`src/v4/`](src/v4/) |
| Role | Production compiler (`gunbc` CLI) | Frozen predecessor | Next substrate + pipeline |
| Source | `.dag` pipelines + shrinking Rust `stage0` | Large `.dag` + Rust tree (prior generation) | `.dag` in the compiler tree; bootstrap seed shrinking |
| What works today | Parse, infer, emit to Rust/Python/Go; large test suite | Reference only — not extended or shipped | `std/` / `extdeps/` model depth; pipeline `.dag` structurally compiles in CI |

**v2 — shipping.** Self-hosted from `.dag` (tokenize through emit, plus complexity and
ownership). Use `gunbc` with `dsl/std/` and `dsl/extdeps/` for your programs. The weather
demo is v2 end-to-end.

**v3 — frozen.** v3 explored behaviors, lenses, and substrate reflection; the tree
remains under `src/v3/` for study and regression context. New modeling and compiler work
live in v4 only.

**v4 — in progress (honest).** v4 combines a typed Node + Behavior substrate with a
full pipeline (tokenize → parse → resolve → infer → emit → translate). In CI, v2
`gunbc` compiles and type-checks all of `src/v4` with zero diagnostics. Runnable v4
stage0, full multi-target emission, execute-verified structural tests, and the self-host
fixed point are still landing — **v2 remains the reference for end-to-end emit until v4
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
DESIGN.md           Worldview, principles, modeling discipline — start here
ROADMAP.md          Current state and direction
CLAUDE.md           Agent/harness instructions

dsl/                Portable daglang vocabulary
  std/                Shared types, algebra, iteration
  extdeps/            External system models (cloud, git, shell)

src/v2/             gunbc compiler (.dag source, shipping)
  00_core.dag         Core types
  02_parse.dag        Tokenizer + parser
  04_infer.dag        Type inference + provenance
  05_emit.dag         Emission (shared + per-target)
  complexity.dag      Termination proofs
  ownership.dag       Ownership analysis

src/v3/             Frozen predecessor (reference; not shipped)

src/v4/             Next compiler generation (in progress)
  std/                Substrate vocabulary
  extdeps/            Language and transport models
  compiler/           Pipeline stages (tokenize … emit … translate)
  lens/               Correctness lenses
  test/claim/         Structural TestClaim corpus

src/v2/tests/       v2 compiler test suite
```

## Documentation

1. **[DESIGN.md](DESIGN.md)** — the worldview, the principles (the review spine), and how to model.
   Currently a working harvest TODO: the prior doc corpus (THESIS/INVARIANTS/MODELING and ~80 design
   docs) was bankrupted 2026-06-16 and is being rebuilt from first principles. The old docs remain in
   git history.
2. **[ROADMAP.md](ROADMAP.md)** — current state and the active wave.
