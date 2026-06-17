# §3 conflation follow-up program — plan & handoff

**Status:** plan / handoff. NOT a design-authority doc — DESIGN.md §3 is the authority; this is a
transient work plan for the follow-up program and should be deleted when the program completes.
Lives on a side branch, never merged to `main`.

**Origin:** session `zesty-lark-54` ("git extdeps is wrong - lets find the scope"). The sample fix
landed in **#5059 (MERGED)**: it established the §3 rule and one worked example at each sub-fact.
This program applies that same rule across the *verified, complete* scope below. All sites were
re-verified present on `main` at handoff time (2026-06-17).

---

## 1. The rule (DESIGN.md §3 — read it first, do not paraphrase from here)

A dependency's interface is three separable facts that must NOT be fused into one row:
- **(a) interface shape** — the parameterized contract (inputs → outputs, exit/error semantics).
  This is what `extdeps/` owns. Agnostic, dispatch-free.
- **(b) transport** (shell / REST / SDK) — a §2 Realization *handler* bound to the shape, one of N.
  NOT a fact about the dependency. The tell it has fused: the same op forked once per transport.
- **(c) business policy** (which base ref, which flag, an idempotency protocol the dep doesn't
  provide) — a *workflow* fact; modeling it in extdeps is a layer inversion. The tell: an argv
  carrying a literal it should receive as a parameter (`origin/main...HEAD`, `--all-targets`).

DESIGN.md generalizes this to **any interface↔realization seam, not only service ops** — the same
de-fusion applies to a cited-data std surface and a per-OS taxonomy: the agnostic *shape* stays
central (`std/`), the realization specifics + the dispatch that selects a realization move outward
(`extdeps/`). A std projection that `match`-es over its realizations names them → import-arrow
inversion. (See `std/os.dag` "projection only" vs `extdeps/os/*` "dispatch lives in extdeps".)

Enforcement carrier: the lens **`v2.lens.extdeps_shape_transport_policy`** (corpus +
`lens_unit` witnesses under `src/v2/compiler/extdeps_shape_transport_policy/`).

---

## 2. Seam A — extdeps service-ops (4 sites; mechanical, low-risk, peripheral layer)

All are **policy-leak / transport-fusion**: a declared param the argv ignores, or a literal that
should be a consumer input. Re-verify each line before editing (line numbers drift).

| # | Site | Defect | Fix |
|---|------|--------|-----|
| 1 | `dsl/extdeps/cloud/gcp/gcp.dag` (`gcloud.Auth.Login`, ~L108) | declares `update_adc: Bool = true` but argv hardwires `--update-adc` → **dead param** | gate the flag on the param (this file's `{version}`/`{project_id}` interp convention), or drop the dead param |
| 2 | `dsl/extdeps/github/gists.dag:66` (`operation Create`) | hardcodes files-map key `"snapshot.md"` (a gunbc workflow name; GistFile key is arbitrary String) — no `filename` input → layer inversion | add `filename: String` input, key the map by it |
| 3 | `dsl/extdeps/runtime/local.dag:14` | `fallback_auth_command: "gcloud auth print-access-token"` re-carries (flattened string) gcp's already-de-fused `AuthPrintAccessToken`; ALSO a dead field (`patterns.dag` `LocalDev { runtime: _ }`) | remove the string; if fallback dispatch is needed it's gunbc/auth *workflow* policy over the existing extdeps shape |
| 4 | `dsl/extdeps/rust/cargo_build.dag:42,50` | Clippy declares `deny_args` but argv hardcodes `-- -D warnings` (dead param + dup of `clippy_deny_args` data); Test hardcodes `--workspace` scope | thread the params (the `Run` op already threads `{package}`/`{bin}`, proving argv param-splice works) |

### 2a. The high-leverage move — do this FIRST (it makes Seam A provable, not grep-asserted)
3 of 4 are ONE mechanically-detectable sub-pattern: **a declared `input` param (or `data` row)
whose name never appears in the operation's argv tokens = a dead param hardwiring a
consumer-chosen literal while falsely advertising selectability.** This is *already* the lens's
named dissolution trigger: harden `v2.lens.extdeps_shape_transport_policy` from text-scan into the
**structural Node-tree argv projection** — flag any `input` param absent from its op's argv. That
single structural check catches sites 1 & 4 (and the already-tracked cargo/llm leaks). Land the
lens hardening with a +/- witness, then fix the 4 sites each proven RED→GREEN by the hardened lens.

---

## 3. Seam B — std interface-conflation (harder; std is more load-bearing → escalate per file)

Grouped by kind. **`std/` carries a higher bar than `extdeps/`** — escalate via
`dashboard-ops escalate` before editing a load-bearing std file under a brief that predates the
relevant model PR.

- **🔴 SECURITY (do standalone, first) — `dsl/std/react_markup.dag`:** re-mints a *diverging* XSS
  href allowlist (`is_allowed_url_scheme` allowlist) vs `html_markup`'s `validate_href` /
  `href_scheme_is_javascript` denylist (VERIFIED divergent — a real drift bug), and forks the
  element tree (`ReactAttr`==`HtmlAttr`, `ReactMarkupNode`==`HtmlNode`). Fix: one
  `std.markup.MarkupNode`, delete react's href fork, reuse `validate_href`.
- **Serialize-result triplication:** `MarkdownSerializeResult` (`markdown.dag`) byte-identical to
  `HtmlSerializeResult` (+ react's `MarkupEmitResult`) — 3 authorities for one serialize-or-reject
  over the one shared Fragment fold. Collapse to one.
- **ISA/platform nicknames — `dsl/std/types.dag`:** `Arch` (3rd ISA spelling, superset of
  `CpuArchitecture`) + `Platform` (dead, ZERO consumers, byte-subset of `Os`). Collapse onto
  `std.cpu.types.CpuArchitecture` + `Os`. (Plus LOW: `type DeclarationRef = String` defined in
  BOTH `emit_model.dag` & `serialization.dag` — dedupe to one home.)
- **Dispatch-in-interface — `dsl/std/fidelity.dag`:** `transport_depth`/`transport_hermetic`
  `match tc { ShellLocal=>… RestNetwork=>… }` is realization-name dispatch *in std* (1:1 echo of
  `extdeps/transports/{shell,file,rest}`). Fix: move rows+dispatch to extdeps; std folds
  (projection, like `os.dag`).
- **One-realization-taxonomy:**
  - `dsl/std/languages.dag` — ~64 per-target `data rust_*/go_*/python_*/typescript_*` decls
    duplicate the live `extdeps/languages/*` authority. ⚠️ **CAVEAT: needs a per-decl consumer
    check before deleting — do NOT blind-delete 64 rows.** Within `dsl/` only the agnostic TYPES
    are imported; the per-target DATA *appears* dead but verify each.
  - `dsl/std/coercion.dag` — `TypeCheckpoint`/`InhabitantDecl` name Rust-only axes (`is_copy`,
    `literal_suffix`); the POSIX-st_mode-in-std shape, 3/4 realizations pass `none`. Lift to rust extdeps.
  - `dsl/std/symbols.dag` — only the ANSI escape-code color table modeled in std (`SemanticColor`
    names ANSI/CSS/CI as siblings), hardwired to `colors.go`. Move `AnsiMapping` to a terminal extdeps renderer.

### Honest caveats (flag, do NOT force)
- `dsl/std/logic.dag` `Classical`(True|False) ≡ `Bool`(True|False) is a **self-admitted scaffold
  deferred on a real compiler limitation** (Branch-input typecheck). Note it; do not force a collapse
  that the compiler can't yet express.
- The `languages.dag` 64-row delete is gated on the per-decl consumer check above.

---

## 4. Already-handled — do NOT re-flag
gcp OAuth2 fork (#5059); git `ChangedFiles` `origin/main...HEAD` (tracked w/ dissolution trigger);
cargo `--all-targets` Build leak (the lens's own cited example); `llm/cli.dag:74` gemini `text`/`plan`
flag values (tracked HONEST-MODELING DEBT in the file header). `CpuArchitecture`/`TargetArchitecture`
already cited in DESIGN.md. The audit correctly did NOT flag `CpuVendor`(silicon) vs `Vendor`(LLVM
triple) — genuinely distinct; do not "collapse" them.

---

## 5. Recommended sequencing (one PR per row; keep each narrow)
1. **Lens hardening** — structural argv projection (§2a). High leverage; unblocks/proves Seam A.
2. **Seam A** — fix the 4 extdeps sites; each lands with a +/- witness now that the lens is structural.
3. **Seam B by risk:** react_markup security fix (standalone) → nickname collapses (Arch/Platform,
   DeclarationRef) → fidelity dispatch-move → one-realization-taxonomy (languages w/ per-decl check,
   coercion, symbols). Each std-file PR: escalate first if load-bearing.

## 6. Working discipline (non-negotiable)
- **Verify by execution, not grep (§5).** "Done" = a real consumer green by execution PLUS a
  discriminating input that goes RED when the behavior is wrong. The lens RED witnesses execute
  against live `dsl/extdeps/**` / `dsl/std/**`.
- **Run the CI floor** (the two composed passes in `.github/workflows/ci.yml`: `claim_batch` +
  `gunbc run dsl/tools/ci_floor_gates.dag`) green before declaring any PR ready. Build via
  `CARGO_BUILD_JOBS=6 ctrl-build -- cargo build -p v1-compiler --release --bin claim_batch --bin gunbc`.
- **Decompress→map→reduce; net concepts must not grow by re-invention.** Each fix DFS the concept
  DAG, map onto the existing authority, reduce the duplicate. A fix that mints a fresh authority for
  a concept that already exists is a failed decomposition.
- **Escalate before touching DESIGN.md-named load-bearing files.** std > extdeps in load-bearing bar.
- **No-merge-queue tax is real** on this corpus — re-sync `main` + re-run the floor on the MERGED
  result; expect ci.yml/floor conflicts when concurrent PRs touch the floor.

## 7. References
- DESIGN.md §2 (minimize redundancy), §3 (single authority / the tri-split), §5 (fail-closed), §6 (lenses).
- Lens: `v2.lens.extdeps_shape_transport_policy` + `src/v2/compiler/extdeps_shape_transport_policy/`.
- The #5059 diff is the worked template (gcp de-fusion, git DiffNameOnly, the corpus regression guard).
