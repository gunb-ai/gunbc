# gunbc v0.1 Launch Plan

**Target date:** 2026-06-01 (19 calendar days from 2026-05-13)
**Scope:** Public soft-launch — installable compiler binary + two runnable sample programs + landing page.
**Audience:** Technical (developers comfortable with terminal). No GUI installer.

---

## Soft-launch discipline

Per standing project rule: **only claim what actually exists; never mid-construction as "actively working."** Demos must be runnable end-to-end. Past tense for capabilities. All landing-page claims gated on a verification pass (Lane G).

---

## Locked-in decisions (2026-05-13)

| Decision | Choice |
|---|---|
| License | MIT OR Apache-2.0 (dual, Rust ecosystem standard) |
| Telemetry | None in v0.1. No phone-home, no crash reports. |
| Installer model | bun-style `curl \| sh`; no signed GUI installer for v0.1 |
| Target audience | Technical, terminal-comfortable |
| Emit-time discipline | gunbc refuses `--target rust` if rustc absent; offers brew/apt/rustup install |

## Open decisions (block Day 1)

| # | Decision | Default if unanswered |
|---|---|---|
| H3 | Skip Windows v0.1? | **Yes (skip)** — adds ~3 days cross-compile + PowerShell installer |
| H4 | Examples in separate `gunb-ai/examples` repo vs inline `examples/` dir? | **Separate repo** — pinnable, lower-friction for newcomers |
| H5 | Landing page status? | Assume blank; build from wireframe |
| H6 | Public announcement (HN/Twitter) on launch day? | **No** — "available, no fanfare" soft-launch |

---

## What actually runs end-to-end on `main` (2026-05-13 inventory)

**No `gunbc` CLI exists today.** v3-compiler is a library invoked from integration tests. Building the CLI shim is Lane A.

| Demo | File | Status |
|---|---|---|
| `weather.dag` | `dsl/examples/weather/weather.dag` | ✅ Compiles in test — types, enums, pattern matching, map/filter |
| `todo_service.dag` | `dsl/demos/todo_service.dag` | ✅ Compiles in test — emits OpenAPI YAML, SQL DDL, Markdown, Rust backend |
| `rest_test.dag` | — | ❌ Intentionally gutted, do not ship |
| `shell_test.dag` | — | ❌ Untested compile path, do not ship |
| Go/Python emit | — | ❌ `#[ignore]`d, ownership port deferred |

**v0.1 demo lineup: weather.dag + todo_service.dag. Two demos, both verified end-to-end.**

---

## Critical path (10 working days, single-threaded)

```
A1 CLI design ──> A2 CLI impl ──> B1 Release workflow ──> C1 install.sh ──> G3 fresh-VM verify ──> tag v0.1.0
   1d              3d              1d                       2d                3d                     0d
```

Everything else parallelizes around this. Slip risk concentrates in A2 (CLI impl) and G3 (fresh-VM matrix).

---

## Swimlanes

### Lane A — Compiler CLI (1–2 engineers)

| ID | Task | Effort | Start | Gates |
|---|---|---|---|---|
| A1 | Design clap CLI surface; document in `docs/cli.md` | 1d | Day 1 | none |
| A2 | New `bin/gunbc.rs` wrapping `compile_to_dag()` + per-target emitters | 3d | Day 2 | A1 |
| A3 | Terminal diagnostic rendering (colored spans, suggestions) | 1d | Day 5 | A2 |
| A4 | `gunbc --help` polish, optional `man gunbc` | 0.5d | Day 5 | A2 |
| A5 | CLI integration tests against both demos | 1d | Day 5 | A2 |

**v0.1 CLI surface:**
```
gunbc compile <file.dag> [--target rust|openapi|sql|markdown] [--out DIR]
gunbc tools install rust   # see Lane D
gunbc --version
gunbc --help
```

### Lane B — Release infrastructure (1 engineer)

| ID | Task | Effort | Start | Gates |
|---|---|---|---|---|
| B1 | GitHub Actions matrix release: aarch64+x86_64 darwin/linux | 1d | Day 5 | A2 produces clean build |
| B2 | SHA256 checksums in release artifacts | 0.25d | Day 6 | B1 |
| B3 | Versioning scheme: `v0.1.0` for launch, semver thereafter | — | Day 1 | none |
| B4 | Release notes template | 0.25d | Day 1 | none |

### Lane C — Installer (1 engineer)

| ID | Task | Effort | Start | Gates |
|---|---|---|---|---|
| C1 | `install.sh`: detect arch, download, verify SHA, install to `~/.gunbc/bin/`, edit shell rc with consent | 2d | Day 6 | B1 |
| C2 | Host at `gunb.ai/install` (DNS + static host) | 1d | Day 7 | C1 |
| C3 | `gunbc uninstall` path | 0.5d | Day 8 | A2, C1 |

**Reference patterns:** bun.sh/install, rustup-init.sh.
**Idempotent:** re-running is a no-op or upgrade.
**Consent:** never edit shell rc without prompt unless `--yes` flag.

### Lane D — Toolchain orchestration (2 engineers; "masterclass" piece)

| ID | Task | Effort | Start | Gates |
|---|---|---|---|---|
| D1 | `gunbc tools install rust` surface design | 0.5d | Day 2 | A1 |
| D2 | Toolchain detection (`rustc --version`, cargo presence) | 0.5d | Day 3 | D1 |
| D3 | Install routes: brew (Mac), apt/dnf (Linux), rustup-init.sh fallback | 2d | Day 3 | D2 |
| D4 | Interactive prompt + `--yes` flag for CI | 0.5d | Day 5 | D3 |
| D5 | Wire into emit path: refuse `--target rust` without rustc, offer install | 1d | Day 6 | D4, A2 |
| D6 | *(v0.2 stretch)* Model D1–D5 as a `.dag`; self-host the install generator | — | post-launch | — |

**v0.1 scope:** Rust only. Python/Go toolchain install deferred (matches their `#[ignore]`d emit status).

### Lane E — Sample programs (2 engineers + ~5 doc-writers from the swarm)

| ID | Task | Effort | Start | Gates |
|---|---|---|---|---|
| E1 | Verify `weather.dag` via `gunbc compile` on 4 platforms | 0.5d | Day 5 | A2 |
| E2 | Verify `todo_service.dag` — all 4 emit targets, output diffs match expected | 1d | Day 5 | A2 |
| E3 | Examples repo setup (location pending H4) | 0.5d | Day 1 | H4 |
| E4 | Per-example README — past tense, "what this shows", "what's not yet shown" | 3d | Day 7 | E1, E2 |
| E5 | `run.sh` per example, pinned to v0.1.0 binary | 0.5d | Day 7 | C1 |

### Lane F — Landing page (2–3 designers + 2 writers from the swarm)

| ID | Task | Effort | Start | Gates |
|---|---|---|---|---|
| F1 | Wireframe: hero, install one-liner, omni-layer demo, examples link | 1d | Day 1 | none |
| F2 | Quickstart doc: install → compile weather → compile todo_service → see 4 outputs | 2d | Day 4 | A2 |
| F3 | FAQ / "known limitations" section — explicit list of what doesn't work in v0.1 | 1d | Day 7 | G1 |
| F4 | Landing copy iteration (swarm-able) | 5d (parallel) | Day 7 | F1 |

### Lane G — Verification & honesty audit (10–15 workers from the swarm)

| ID | Task | Effort | Start | Gates |
|---|---|---|---|---|
| G1 | Baseline claim audit: every claim → `runs / aspirational / removed` | 2d | Day 1 | none |
| G2 | Strip aspirational language unless under explicit "Roadmap" heading | 4d (parallel) | Day 3 | G1 |
| G3 | **Fresh-VM end-to-end matrix:** clean macOS (x86, aarch64), Ubuntu, Debian, Arch, NixOS, Fedora. Each runs install.sh + both demos. | 3d | Day 10 | C1, E1, E2 |
| G4 | Bug triage from G3, fixes route back to Lane A/C/D | 3d | Day 14 | G3 |

**G3 is the launch gate.** No tag without all 6 distros green.

---

## Calendar (working backwards from 2026-06-01)

| Date | Days out | Milestone |
|---|---|---|
| 2026-05-13 | 19 | Plan ratified, decisions H3–H6 resolved, work items created |
| 2026-05-14 | 18 | Lane A1/B3/B4/E3/F1/G1 in flight |
| 2026-05-17 | 15 | A2 mergeable; D2/D3 in flight |
| 2026-05-19 | 13 | B1 green, first release artifact produced |
| 2026-05-21 | 11 | install.sh live; D5 wired |
| 2026-05-23 | 9 | All Lane E demos verified; Lane F copy 80% done |
| 2026-05-26 | 6 | **G3 fresh-VM matrix begins** |
| 2026-05-29 | 3 | G3 green or G4 bug-fix loop in last cycle |
| 2026-05-31 | 1 | Final tag `v0.1.0` |
| 2026-06-01 | 0 | Public launch — install.sh + examples + landing live |

---

## Risk register

| # | Risk | Mitigation |
|---|---|---|
| R1 | A2 slips → entire critical path slips | Ship A2 as soon as weather.dag runs; don't gate on todo_service polish |
| R2 | Compiler regression in either demo during 19d | Daily CI alert on `m1_5_omni_shape_b_openapi_test` + weather.dag tests |
| R3 | D5 scope creep ("just works" tempting to over-polish) | v0.1 scope: rust only. Punt python/go toolchain install entirely. |
| R4 | G3 finds platform-specific bugs in week 3 | Buffer days 14–17 reserved for G4 fix loop |
| R5 | Soft-launch trap: shipping aspirational claims | G1 + G2 audit; nothing on landing page without runnable proof |
| R6 | DNS/hosting for gunb.ai/install not ready | Validate H5 Day 1; fallback to GitHub raw URL if needed |

---

## Acceptance criteria for v0.1 launch

1. `curl -fsSL gunb.ai/install | sh` puts a working `gunbc` on PATH across macOS (x86+aarch64) and Linux (x86+aarch64).
2. `gunbc compile examples/weather.dag` produces correct Rust output.
3. `gunbc compile examples/todo_service.dag --target openapi` produces valid OpenAPI 3.1 YAML.
4. `gunbc compile examples/todo_service.dag --target rust` produces buildable Rust source.
5. Missing rustc on `--target rust` triggers honest install prompt that succeeds via brew/apt/rustup.
6. Both example READMEs describe ONLY behaviors that work in v0.1.
7. Landing-page FAQ explicitly enumerates what v0.1 cannot do (Go/Python emit, Windows, etc.).
8. Zero phone-home traffic from installed binary.
9. License files present in both source and release artifacts.
10. All claims on landing page verified against runnable artifact by Lane G.

---

## Staffing fit for 50 workers

The 50-worker leverage applies to **polish, docs, and verification breadth**, not critical-path engineering. Suggested allocation:

| Lane | Workers | Why |
|---|---|---|
| A (CLI engineering) | 2 | Bounded surface; more bodies = merge conflicts |
| B (release infra) | 1 | One-time setup |
| C (installer) | 1 | Bounded surface |
| D (toolchain) | 2 | Per-platform install logic parallelizes |
| E (samples + READMEs) | 7 | 2 engineers + 5 doc writers |
| F (landing) | 5 | 2 designers + 3 copy writers iterating |
| G (verification) | 25–30 | Fresh-VM matrix is where the 50 actually scale — distro × demo × platform |
| Reserve | 3–5 | Bug triage flex into Lane A/C/D |

The bottleneck is **review bandwidth**, not engineer-hours. Critical-path PRs need fast turnaround; queue them ahead of polish PRs.

---

## Not in v0.1 (explicit non-goals)

- Windows binary (defer to v0.2 unless H3 reversed)
- Signed `.pkg` / `.msi` GUI installers
- Homebrew tap / winget / .deb / AppImage (defer to v0.2)
- Go and Python emit targets (`#[ignore]`d upstream)
- Lens-output demos as standalone runnables (currently only test-claim form)
- IDE plugins / LSP
- `gunbc fmt` / `gunbc init`
- Self-hosting D6 (`.dag`-generated install logic) — v0.2 thesis demo
- Public announcement / PR push (H6 decision pending)
