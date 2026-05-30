# Supported in v0.1.0

This document is the **normative support contract** for the **v0.1.0** public
release of [daglang](https://github.com/gunb-ai/daglang) and the **gunbc**
compiler. README, release notes, and the project website must not claim
capabilities beyond what is listed here.

**Compiler generation.** v0.1.0’s supported product surface is the **v2**
self-hosted compiler (`gunbc`, crate `v2-compiler`). Everything under
`src/v3/` and `src/v4/` may appear in the repository but is **not** on this
contract unless explicitly called out in the [Alpha and work-in-progress](#alpha-and-work-in-progress) section.

---

## Two public tiers

| Tier | Meaning for you |
| ---- | ---------------- |
| **Supported** | We intend v0.1.0 to work here. Bug reports are welcome. Documented examples and checks below are the regression bar. |
| **Alpha / WIP** | Ships in-tree for transparency. **No compile guarantee, no regression contract.** You may hit errors, missing features, or stale docs. |

If a feature is not listed under **Supported**, treat it as **unsupported** —
even if the CLI accepts a flag or the repository contains related code.

---

## Supported emit targets (v2)

v0.1.0 advertises **three** emit targets, one per v2 emitter
(`src/v2/05_emit_rust.dag`, `05_emit_python.dag`, `05_emit_go.dag`):

| Target | Support level in v0.1.0 | Verification bar |
| ------ | ------------------------ | ------------------ |
| **rust** | **Full compile target** | Documented examples emit Rust that passes `cargo check` (or `rustc` on the emitted crate). |
| **python** | **Full compile target** | Emitted `.py` files pass `python3 -m py_compile`. The interpreter example below must run. |
| **go** | **Full compile target** | Emitted Go passes `go build` and `go vet` on the documented example tree. |

**What “supports” means.** Saying v0.1.0 supports a target always means: for the
[supported source subset](#supported-daglang-subset) and [shipped examples](#shipped-examples),
`gunbc compile --target <name>` succeeds and the external toolchain check in the
table passes. It does **not** mean every `.dag` file in the repository compiles to
that target.

**Not on the v0.1.0 emit contract**

- **TypeScript** — modeled in v4 (`src/v4/extdeps/languages/typescript.dag`) as
  **alpha only**; see [TypeScript (v4 alpha)](#typescript-v4-alpha).
- **C, C++, LLVM IR, Swift, Java, WASM, and other language models** — not v0.1.0
  public support.
- **`--target dag`** — accepted by the CLI for compiler development (typed DAG
  artifact JSON). It is **not** a user-facing v0.1.0 product target; do not build
  production workflows on it.

Any other `--target` value is rejected at startup with a non-zero exit and an
explicit error (fail-closed).

---

## Supported daglang subset

v0.1.0 supports a **small, example-backed** slice of the language — not the full
`dsl/std/` corpus and not arbitrary programs under `src/v2/` or `src/v4/`.

**Anchors.** Support is defined by what the shipped examples compile and run:

- [`dsl/examples/weather/weather.dag`](../dsl/examples/weather/weather.dag)
- [`dsl/examples/interp_test/interp_test.dag`](../dsl/examples/interp_test/interp_test.dag)

plus the **`dsl/std/`** vocabulary those programs import transitively (primitives,
`List`, `concat`, `to_string`, list algebra, and related std modules).

**Language features on the contract** (as used in those examples):

- `module` declarations and multi-root `--source-root` layout (entry modules in
  the first root; dependencies resolved from additional roots).
- Product types (records) and sum types (coproducts / enums), including variants
  with payloads.
- Functions with typed parameters and return types; top-level `fn` definitions.
- `let` bindings; `if` / `else`.
- `match` with variant patterns and payload destructuring.
- List literals and `List<T>`.
- Pipeline syntax (`|>`) with `map`, `filter`, and `fold`.
- String concatenation (`concat`) and `to_string` for numeric values.
- Closures in pipeline callbacks (as in `interp_test.dag`).

**Explicitly unsupported for v0.1.0** (non-exhaustive):

- Programs outside the construct set above (e.g. workflows, services, REST/Shell
  transports, effects, and most of `dsl/extdeps/`).
- Compiling the compiler’s own `src/v2/` pipeline as user daglang.
- “Works on my machine” `.dag` that imports modules not reachable from the
  documented `--source-root` layout.
- Any claim of **exhaustive** `dsl/std/` coverage — only the std modules pulled
  by the shipped examples are in scope.

Using unsupported constructs should **fail closed**: the compiler reports
diagnostics and exits non-zero rather than emitting partial or guessed target
code.

---

## Shipped examples

| Example | Role | Supported commands |
| ------- | ---- | ------------------ |
| **Weather** | Hero compile demo (types, enums, match, list pipeline) | `gunbc compile` with `--target rust`, `python`, or `go`; Rust path verified with `cargo check` on the output tree. |
| **Interpreter test** | End-to-end execution demo | `gunbc run` with `--source-root dsl/examples/interp_test` and `--source-root dsl/std`. |

**Weather — Rust gate** (from repo root after `cargo build --release -p v2-compiler --bin gunbc`):

```bash
./target/release/gunbc compile \
  --source-root dsl/examples/weather \
  --source-root dsl/std \
  --output-dir /tmp/weather-out \
  --target rust
cargo check --manifest-path /tmp/weather-out/Cargo.toml
```

**Interpreter test — run gate:**

```bash
./target/release/gunbc run \
  --source-root dsl/examples/interp_test \
  --source-root dsl/std
```

Other files under `dsl/examples/` (including transport demos) are **not** on the
v0.1.0 contract unless listed above.

---

## CLI on the support contract

| Command | Supported flags / behavior |
| ------- | ------------------------- |
| `gunbc compile` | `--source-root` (repeatable), `--output-dir`, `--target` ∈ {`rust`, `python`, `go`} for the contract; `--source-dir` legacy single-tree mode. |
| `gunbc run` | `--source-root` (repeatable), `--function` (default `main`). |
| Global | `--dry-run` (mock service calls). |
| `gunbc --help` | Usage text for the above. |

Subcommands and flags not listed here are **unsupported** for v0.1.0.

---

## Installing gunbc

**Always supported:** build from source with a stable Rust toolchain (see
`rust-toolchain.toml`):

```bash
cargo build --release -p v2-compiler --bin gunbc
```

The release binary is `target/release/gunbc`.

**Prebuilt binaries** (GitHub Releases on tag `v0.1.0`):

Release CI builds the following **target triples** when the release workflow is
green. **Only triples that passed the release dry-run before tag are advertised**
on the download page; if your platform is missing, use the source build.

| OS | Architecture | Target triple | Release asset (basename) |
| -- | ------------- | ------------- | ------------------------- |
| Linux | x86_64 | `x86_64-unknown-linux-musl` | `gunbc-x86_64-unknown-linux-musl` |
| Linux | arm64 | `aarch64-unknown-linux-musl` | `gunbc-aarch64-unknown-linux-musl` |
| macOS | x86_64 | `x86_64-apple-darwin` | `gunbc-x86_64-apple-darwin` |
| macOS | Apple silicon | `aarch64-apple-darwin` | `gunbc-aarch64-apple-darwin` |
| Windows | x86_64 | `x86_64-pc-windows-msvc` | `gunbc-x86_64-pc-windows-msvc.exe` |
| Windows | arm64 | `aarch64-pc-windows-msvc` | `gunbc-aarch64-pc-windows-msvc.exe` |

**Not guaranteed in v0.1.0:** Homebrew, `.deb`/APT, or crates.io distribution.
Use source build or release assets only.

**Host OS for development:** Linux and macOS are the primary environments used in
CI. Windows is supported via release artifacts and local `cargo build`; emitted
*user* code targets depend on the table in [Supported emit targets](#supported-emit-targets-v2).

---

## Alpha and work-in-progress

### v3 and v4 trees

`src/v3/` and `src/v4/` ship in the public repository **labeled alpha / WIP**.
They are **not** on the v0.1.0 support contract:

- No promise of clean compile across the full v4 tree.
- No regression discipline for v4 substrate changes.
- v4 compiler pipeline and bootstrap workflows may require internal CI bridges
  not available to end users.

v2 (`gunbc` + this document) is the supported path for trying daglang today.

### TypeScript (v4 alpha)

TypeScript is **v4 early-support only** — **not** v0.1.0 supported emit.

- Substrate: `src/v4/extdeps/languages/typescript.dag` (and related ECMAScript
  model).
- **TypeScript output is not currently checked against `tsc`; report-and-track
  basis only.** Do not expect clean `tsc` emit from v4 in v0.1.0.
- Structural tests (parse, grammar round-trip) may exist; they do not establish
  a user-facing compile guarantee.

### Other v4 language models

C++, LLVM IR, Swift, Java, and additional models under `src/v4/extdeps/languages/`
may appear as substrate. Unless a future amendment to this file promotes a
surface to **Supported**, treat them as **alpha / WIP** with no user contract.

---

## Explicitly out of scope for v0.1.0

- Self-hosting fixed point (v4 compiling v4 without bootstrap bridges).
- React / framework application generation.
- Arbitrary corpus emit or “all of `dsl/std/`.”
- v4-done predicate closure as a release gate (internal maturation only).
- TypeScript, C++, and LLVM as **supported** emit targets (see above).

---

## Fail-closed guarantee

For v0.1.0, **unsupported must not mean undefined**:

1. **Unknown emit targets** — CLI exits with an error naming `rust`, `python`,
   `go`, and `dag` (the latter is developer-only, not contract-supported).
2. **Compile-time errors** — parse, resolve, infer, and emit report diagnostics;
   hard errors yield a non-zero exit after printing them (complexity analyzer
   limitations may emit non-blocking warnings only where documented in the
   compiler).
3. **Missing modules or roots** — missing `--source-root`, duplicate `module`
   paths, or unreadable files fail with an explicit panic or diagnostic, not
   silent omission.
4. **No silent partial emit** — if emission cannot complete for a supported
   example with a supported target, the run fails; users should not rely on
   incomplete output trees.

If you observe silent success on an unsupported path, please file a bug — that
is a defect against this contract.

---

## Reporting issues

For **supported** surfaces, open an issue on the public repository with:

- `gunbc --help` output (or version/commit).
- Exact `gunbc compile` / `gunbc run` command line.
- Target triple (for binary issues) or `rustc --version` / `go version` /
  `python3 --version` (for emit issues).
- Relevant `.dag` source or example name (`weather` or `interp_test`).

For **alpha / WIP** surfaces, issues are still welcome but may be closed as
“not v0.1.0 contract” unless they affect a supported path.

---

## Amendments

Promoting a feature from alpha to **Supported** requires an update to this file
(and the documented verification gates). Demotions use the same process. Patch
releases may narrow support only with clear release-note callouts.
