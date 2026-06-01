# gunbc Roadmap

Where the project stands and where it is headed. For the intellectual goal, read [THESIS.md](THESIS.md). For rules that protect that goal, read [INVARIANTS.md](INVARIANTS.md). For how to extend the language safely, read [MODELING.md](MODELING.md).

> **v4 is the active development phase.** New substrate modeling and compiler pipeline work live in [`src/v4/`](src/v4/). v3 is frozen. v2 remains the production self-hosted compiler today.

## What works today (v2)

The v2 compiler in [`src/v2/`](src/v2/) is self-hosted from `.dag` source: tokenize, parse, infer, emit, complexity, and ownership pipelines are authored as `.dag` programs with a small Rust bootstrap (`stage0`) that shrinks over time.

You can:

- Write `.dag` programs using `dsl/std/` and `dsl/extdeps/`
- Compile and validate causal structure (types, termination, effects, ownership)
- Emit to Rust, Python, and Go from the same declarations
- Run the compiler and test suite via Cargo (`cargo test -p v2-compiler-tests`)

The `gunbc` CLI (from the `v2-compiler` crate) is the primary entry point for compiling v4 trees during bootstrap work.

## What v4 is building toward

v4 combines substrate depth (typed Node + Behavior kernel, algebra-grounded std library, rich `extdeps/`) with a full compiler pipeline rewritten in that substrate:

| Area | Status (approximate) |
|------|----------------------|
| `src/v4/std/` — shared vocabulary (node, algebra, effects, grammar, …) | Substantial; core carriers landed |
| `src/v4/extdeps/` — language and transport models | Broad coverage; ongoing grounding work |
| Compiler stages (`01_tokenize` … `06_translate`, `00_compile`) | Parse and type-check `.dag`; emission in progress |
| Lenses (complexity, cost, coverage, testgen, …) | Many structural lenses; runner integration ongoing |
| Pure bootstrap / self-host | Trajectory to zero hand-maintained Rust; `self_host.dag` ratchet |
| Tests as `.dag` `TestClaim` data | Growing corpus under `src/v4/test/claim/` |

### Public Operational Lanes

| Row | Public tracking intent |
|-----|------------------------|
| T-PB-B / `pb_rust_tests_outside_residual_zero` | Move remaining hand-authored Rust boundary and smoke tests into `.dag` `TestClaim` / generated-runner coverage, keeping same-path SG-0 expansions at +0 new paths until the matching claim runner executes those facts directly. |

**Honest v4 status:** the v4 pipeline compiles and type-checks `.dag` over `src/v4` in CI. Lowering, full multi-target emission, and execute-verified test claims are still landing. v2 remains the reference for end-to-end emit until v4 closes the loop.

Design direction: **model local, derive global** — every target modeled once in shared vocabulary; translations are derived homomorphisms, not hand-written adapters ([docs/thesis/the-derived-homomorphism.md](docs/thesis/the-derived-homomorphism.md)).

## Milestone shape

Work is organized around closing the bootstrap loop, not a calendar:

1. **Substrate complete** — std + extdeps fact-bundles ground external primitives without hollow aliases.
2. **Compiler pipeline** — tokenize → parse → resolve → infer → emit → translate with fail-closed diagnostics.
3. **Lenses and tests** — structural `TestClaim` predicates evaluated by generated or substrate runners; lenses over the same Node tree users write.
4. **Self-host fixed point** — `compiler.dag` emits bit-identical stage0; hand-maintained file count → 0 per [docs/design-pure-bootstrap-zero.md](docs/design-pure-bootstrap-zero.md).
5. **Public release** — v2 + v4 story documented; binaries via GitHub Releases; public repo snapshot with these root docs.

Earlier release-program lanes (complexity parity, testgen, multi-target emit, pure-bootstrap floors) informed v4 scope; detailed operational tracking for that era lives in [`_internal/ROADMAP_OPS.md`](_internal/ROADMAP_OPS.md) for maintainers migrating from the internal repo.

## How to read the tree

```
THESIS.md          — why gunbc exists
INVARIANTS.md      — five principles + elaborations
MODELING.md        — concept DAG / modeling discipline
ROADMAP.md         — this file

dsl/std/           — portable vocabulary
dsl/extdeps/       — external systems

src/v2/            — shipping self-hosted compiler
src/v4/            — next substrate + compiler
```

## Contributing orientation

- Prefer extending `std/` or `extdeps/` before adding compiler-local types.
- Every scaffold needs a dissolution trigger; progress reduces duplicate authority.
- Run `cargo test --workspace`, `cargo clippy --all-targets -- -D warnings`, and `cargo fmt --all --check` before pushing.

For deep design context: [docs/architecture.md](docs/architecture.md), [docs/v3-spec.md](docs/v3-spec.md) (language surface, still relevant to v4), and essays under [docs/thesis/](docs/thesis/).
