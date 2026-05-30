# Supported in v0.1.0

This document is the **normative support contract** for the **v0.1.0** public
release of [daglang](https://github.com/gunb-ai/daglang) and the **gunbc**
compiler. README, release notes, and the project website must not claim
capabilities beyond what is listed here.

| Tier | Section |
| ---- | ------- |
| **Supported** | [§1 — v0.1.0 supported](#1-v010-supported) |
| **Alpha / WIP** | [§2 — Alpha and work-in-progress](#2-alpha-and-work-in-progress) |

If a feature is not listed under **§1**, treat it as **unsupported** — even if the
CLI accepts a flag or the repository contains related code.

**Compiler generation.** v0.1.0’s supported product surface is the **v2**
self-hosted compiler (`gunbc`, crate `v2-compiler`). Everything under `src/v3/` and
`src/v4/` is **§2 alpha / WIP** unless this file explicitly promotes it.

Related (alpha disposition detail, not normative for §1): when present in the
public tree, see `docs/release/v0.1.0-v4-ship-disposition.md` (v4 ship-disposition
supplement). Release readiness snapshot: `docs/RELEASE_v0.1.0.md` (when landed).

---

## 1. v0.1.0 supported

### v2 per-target confidence

v0.1.0 advertises **three** v2 emit targets (`src/v2/05_emit_rust.dag`,
`05_emit_python.dag`, `05_emit_go.dag`). Confidence below is **for the supported
small-`dsl` examples only** — not for arbitrary `src/v4/` programs.

| Target | Confidence | §1 contract summary |
| ------ | ------------ | ------------------- |
| **rust** | **HIGH** | Full compile target for documented examples; see verification bar below. |
| **python** | **MEDIUM** | Small `dsl/` examples only; **v4-substrate Python NOT supported.** |
| **go** | **MEDIUM-LOW** | Small smoke only; **v4-fixture Go build is open.** |

**Rust = HIGH confidence for v0.1.0 supported.** Verified via scripts/v4-mvp1-e2e-gate.sh on main CI + cargo test -p v2-compiler-tests pipeline/bootstrap emit tests. Works for fixtures/v4-mvp1/add.dag + small in-tree modules. **Limit:** full src/v4 emit produces ~7951 rustc errors (SG-1 Symbol/E0423 + SG-2 generic arity/E0107 dominate) — this is v4 substrate, NOT v0.1.0 supported.

**Python = MEDIUM confidence.** Verified via v2-compiler-tests unit tests (same_source_emits_to_rust_and_python, scrambled_name_emit_python). **NOT** a dedicated add.dag → python CI gate. **Known open bug:** emit_python TCO bug on complex modules (emit_tco_unified path); phase1/nat_semiring v4-scoped emit often fails py_compile. **#3996 did NOT fix this.** Honest for small dsl examples. DO NOT label v4-substrate Python 'supported'.

**Go = MEDIUM-LOW confidence.** Verified via v2-compiler-tests only. **Known open bug:** phase1/nat_semiring go build still substrate-red (multi-file package/module-path mismatches reported as missing-package); #4015 in flight is gate-alignment honesty, NOT emitter fix. Small smoke OK; v4 fixture build still open; lagging Rust/Python.

**What “supports” means.** For the [supported daglang subset](#supported-daglang-subset) and [shipped examples](#shipped-examples), `gunbc compile --target <name>` succeeds and the external toolchain check for that target passes at the confidence level above. It does **not** mean every `.dag` file in the repository compiles to that target.

**Not on the v0.1.0 emit contract (§1)**

- **TypeScript** — **§2 v4 alpha only** (not v2 / not §1).
- **C, C++, LLVM IR, Swift, Java, WASM, and other language models** — **§2** or out of scope.
- **`--target dag`** — CLI accepts it for compiler development (typed DAG artifact JSON). **Not** a user-facing v0.1.0 product target.

Any other `--target` value is rejected at startup with a non-zero exit and an
explicit error (fail-closed).

#### Known open bugs per target

| Target | Open issue | Impact on §1 |
| ------ | ---------- | ------------- |
| **rust** | Full-tree `src/v4` emit → ~7,951 `rustc` errors (SG-1 `E0423`, SG-2 `E0107` dominate) | Does **not** reduce §1 confidence for small `dsl/` examples; **do not** treat full v4-tree Rust emit as supported. |
| **python** | `emit_tco_unified` TCO bug on complex modules; `phase1/nat_semiring` v4-scoped emit often fails `py_compile` (#3996 did not fix) | §1 limited to **small `dsl/` examples**; v4-substrate Python is **not** supported. |
| **go** | `phase1/nat_semiring` `go build` still substrate-red (multi-file package / module-path mismatches); #4015 is gate-alignment honesty, not an emitter fix | §1 **small smoke** only; v4-fixture Go build remains **open**. |

#### Verification bar (§1 examples)

| Target | Toolchain check on documented examples |
| ------ | -------------------------------------- |
| **rust** | Emitted crate passes `cargo check`. |
| **python** | Emitted `.py` passes `python3 -m py_compile`; `gunbc run` works for `interp_test`. |
| **go** | Emitted tree passes `go build` and `go vet` on the weather example layout. |

### Weather hero demo

Weather demo is **COMPILE-VERIFIED** (emit-to-Rust compiles per check-clean-checkout-build.sh ignoring bootstrap_l4_structural). There is **NO routine CI proof of cargo RUN binary** at HEAD. The Quick Start command sequence is verified to **PRODUCE** Rust output and **PASS** `cargo check`; running the binary end-to-end is being verified by adhoc-bec7923f-4d7 (merry-bear-248) — embed the verification transcript reference once their report arrives.

**Weather — compile gate** (from repo root after `cargo build --release -p v2-compiler --bin gunbc`):

```bash
./target/release/gunbc compile \
  --source-root dsl/examples/weather \
  --source-root dsl/std \
  --output-dir /tmp/weather-out \
  --target rust
cargo check --manifest-path /tmp/weather-out/Cargo.toml
```

Swap `--target rust` for `python` or `go` only within the confidence limits above.

### Supported daglang subset

v0.1.0 supports a **small, example-backed** slice of the language — not the full
`dsl/std/` corpus and not arbitrary programs under `src/v2/` or `src/v4/`.

**Anchors.**

- [`dsl/examples/weather/weather.dag`](../dsl/examples/weather/weather.dag)
- [`dsl/examples/interp_test/interp_test.dag`](../dsl/examples/interp_test/interp_test.dag)

plus the **`dsl/std/`** vocabulary those programs import transitively.

**Language features on the contract** (as used in those examples):

- `module` declarations and multi-root `--source-root` layout.
- Product types (records) and sum types (coproducts / enums), including variants with payloads.
- Functions with typed parameters and return types; top-level `fn` definitions.
- `let` bindings; `if` / `else`; `match` with variant patterns and payload destructuring.
- List literals and `List<T>`; pipeline syntax (`|>`) with `map`, `filter`, and `fold`.
- `concat`, `to_string`, and closures in pipeline callbacks (`interp_test.dag`).

**Explicitly unsupported for v0.1.0** (non-exhaustive): workflows, services, REST/Shell transports, effects, most of `dsl/extdeps/`, compiling `src/v2/` as user daglang, and any claim of exhaustive `dsl/std/` coverage.

### Shipped examples

| Example | Role | §1 commands |
| ------- | ---- | ----------- |
| **Weather** | Hero **compile** demo | `gunbc compile` → rust / python / go per confidence table; Rust path: `cargo check` on output (run verification pending — see [Weather hero demo](#weather-hero-demo)). |
| **Interpreter test** | Execution demo | `gunbc run` with `--source-root dsl/examples/interp_test` and `--source-root dsl/std`. |

```bash
./target/release/gunbc run \
  --source-root dsl/examples/interp_test \
  --source-root dsl/std
```

Other files under `dsl/examples/` are **not** on the §1 contract unless listed above.

### CLI on the support contract

| Command | Supported flags / behavior |
| ------- | ------------------------- |
| `gunbc compile` | `--source-root` (repeatable), `--output-dir`, `--target` ∈ {`rust`, `python`, `go`} for §1; `--source-dir` legacy mode. |
| `gunbc run` | `--source-root` (repeatable), `--function` (default `main`). |
| Global | `--dry-run`. |
| `gunbc --help` | Usage for the above. |

### Installing gunbc

**Always supported:** build from source (`rust-toolchain.toml`):

```bash
cargo build --release -p v2-compiler --bin gunbc
```

**Prebuilt binaries** (GitHub Releases on tag `v0.1.0`): only triples that passed the release dry-run before tag are advertised.

| OS | Architecture | Target triple | Release asset (basename) |
| -- | ------------- | ------------- | ------------------------- |
| Linux | x86_64 | `x86_64-unknown-linux-musl` | `gunbc-x86_64-unknown-linux-musl` |
| Linux | arm64 | `aarch64-unknown-linux-musl` | `gunbc-aarch64-unknown-linux-musl` |
| macOS | x86_64 | `x86_64-apple-darwin` | `gunbc-x86_64-apple-darwin` |
| macOS | Apple silicon | `aarch64-apple-darwin` | `gunbc-aarch64-apple-darwin` |
| Windows | x86_64 | `x86_64-pc-windows-msvc` | `gunbc-x86_64-pc-windows-msvc.exe` |
| Windows | arm64 | `aarch64-pc-windows-msvc` | `gunbc-aarch64-pc-windows-msvc.exe` |

**Not guaranteed in v0.1.0:** Homebrew, `.deb`/APT, crates.io.

### Fail-closed guarantee (§1)

For v0.1.0 supported paths, **unsupported must not mean undefined**:

1. **Unknown emit targets** — non-zero exit; error names allowed targets.
2. **Compile-time errors** — diagnostics printed; hard errors → non-zero exit.
3. **Missing modules or roots** — explicit failure, not silent omission.
4. **No silent partial emit** on §1 examples with §1 targets.

---

## 2. Alpha and work-in-progress

**Alpha / WIP** surfaces ship in-tree for transparency. **No compile guarantee, no
regression contract, no fail-closed bar.** Users accept current-state errors.

### v3 and v4 trees (general)

`src/v3/` and `src/v4/` are **not** on the §1 contract. v2 (`gunbc` + §1) is the
supported path for trying daglang in v0.1.0.

### v4 Rust (alpha)

**Rust** → "alpha with verification framework + measured gap count"
(~7,951 errors on full-tree v4 emit, falling as SG-1/SG-5 land).

Full-tree `src/v4` Rust emit is **not** §1 supported. Per-target leaf-model runners
(R1/R2a/R2b/R3-external) exist for substrate work; that does not promote full-tree
emit to §1.

### v4 TypeScript (alpha)

**TypeScript** → "alpha substrate-only, no verification path, exploratory" (must
say this honestly).

TypeScript is **v4 early-support only** — **not** v0.1.0 §1 supported emit.

**TypeScript output is not currently checked against `tsc`; report-and-track
basis only.** Do not expect clean `tsc` emit from v4 in v0.1.0.

Substrate: `src/v4/extdeps/languages/typescript.dag` (and related ECMAScript model).
Structural tests (parse, grammar round-trip) do not establish a user-facing compile
guarantee.

### Other v4 language models

C++, LLVM IR, Swift, Java, and additional models under `src/v4/extdeps/languages/`
ship as **alpha / WIP** with no user contract unless a future amendment promotes
them to §1.

### Explicitly out of scope (§2 and beyond)

- Self-hosting fixed point (v4 compiling v4 without bootstrap bridges).
- React / framework application generation.
- Arbitrary corpus emit or “all of `dsl/std/`.”
- v4-done predicate closure as a v0.1.0 release gate.

---

## Reporting issues

For **§1 supported** surfaces, open an issue with: `gunbc --help` (or version/commit), exact command line, host/target toolchain versions, and example name (`weather` or `interp_test`).

For **§2 alpha / WIP**, issues are welcome but may be closed as “not v0.1.0 §1 contract” unless they affect a §1 path.

---

## Amendments

Promoting a feature from §2 to **§1** requires an update to this file and documented verification gates. Demotions use the same process. Patch releases may narrow §1 only with clear release-note callouts.
