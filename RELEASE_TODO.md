# Release TODO — June 1 Public Launch

Tracking all cleanup, migration, and infrastructure work needed before
the public release. Sections are ordered roughly by dependency; items
within a section are independent unless noted.

**Retirement:** delete this file after `v0.1.0` public release tags and the
public repo is live. Migrate any post-launch residuals to `ROADMAP.md` or the
issue tracker by that point — this file is not a permanent backlog.

---

## 0. Merge gate (do first)

- [ ] PR #3826 — rename binary `v2-compiler` → `gunbc` — merge when CI green
- [ ] Confirm `cargo build --release -p v2-compiler --bin gunbc` produces
      `target/release/gunbc` on a clean checkout after #3826 lands

---

## 1. GitHub plan migration (Enterprise → Teams)

**Context:** GitHub Enterprise Cloud is ~$21/user/month. GitHub Teams is $4/user/month.
Teams has everything needed here: branch protection, required status checks,
self-hosted runners, GitHub Apps, Actions.

### Check for Enterprise-only features in use before downgrading

- [x] Audit org settings for SAML/SSO — if any teammates or bots authenticate
      via SAML, that breaks on Teams (SAML is Enterprise-only)
      — **2026-05-29**: none (`samlIdentityProvider` null; operator confirmed). Receipt: `docs/admin/github-enterprise-to-teams-audit-2026-05-29.md`
- [x] Check if audit log API (`/orgs/{org}/audit-log`) is used by the
      session dashboard or any script — it's Enterprise-only
      — **2026-05-29**: API 404 on current plan; no callers in gunbc or ctrl session-dashboard
- [x] Check installed GitHub Apps on `gunb-ai` org — verify none require
      Enterprise tier (most don't)
      — **2026-05-29**: operator attestation (one app, Teams-compatible); UI confirm before downgrade
- [x] Check `required_two_factor_authentication` org policy — available on Teams
      — **2026-05-29**: disabled (`two_factor_requirement_enabled: false`)
- [x] Check any IP allowlist settings — Enterprise-only; Teams has no equivalent
      — **2026-05-29**: none (ip-allow-list endpoint 404)
- [x] Verify self-hosted runner registration doesn't rely on Enterprise runner groups
      (runner groups ARE available on Teams, but the API path differs slightly)
      — **2026-05-29**: CI uses label selectors only; operator confirmed no Enterprise runner groups
- [x] Review `.github/workflows/ci.yml` for any `enterprise:` keys or
      Enterprise-specific Actions features
      — **2026-05-29**: no `enterprise:` keys in any workflow under `.github/workflows/`

### Migration steps

- [ ] Contact GitHub support or use org billing settings to downgrade plan
- [ ] Verify Actions minutes quota — Teams gets 3,000 min/month free for private
      repos; self-hosted runners don't count against quota regardless of plan
- [ ] Verify Packages/storage quota is adequate
- [ ] Test a CI run on a dev branch after migration before touching main

### Likely safe (no action needed)

- Branch protection rules — available on Teams
- Required status checks — available on Teams
- Self-hosted runners — available on Teams
- GitHub Actions — same feature set on Teams
- Deploy keys, webhooks, GitHub Apps — same on Teams
- Private repos — same on Teams

---

## 2. Public/private repo split

**Goal:** `gunb-ai/gunbc` stays private (internal). A new public-facing repo
is seeded with a clean squash snapshot. GitHub does not allow two repos with
the same owner/name, so pick one topology and stick to it.

### One-time setup

- [ ] Create public repo — decide on name (mutually exclusive options):
  - **Option A — rename internal, reclaim slug:** rename current `gunb-ai/gunbc`
    → `gunb-ai/gunbc-internal` (or `gunbc-dev`), then create new public
    `gunb-ai/gunbc` — cleanest public URL, but requires updating all internal
    CI remote references
  - **Option B — keep slug, add suffix:** keep current `gunb-ai/gunbc` (private)
    and create public `gunb-ai/gunbc-public` — no CI changes needed, but public
    URL has a `-public` suffix
- [ ] Write `scripts/publish-snapshot.sh` — the sync script:
  ```bash
  # Creates a clean export commit and force-pushes to public repo
  # Run from internal repo main branch
  SNAPSHOT=$(git rev-parse --short HEAD)
  git worktree add /tmp/gunbc-pub-export main
  cd /tmp/gunbc-pub-export
  # Strip internal dirs
  rm -rf docs/briefs docs/history docs/debt docs/review-findings \
         docs/admin docs/db-history docs/postmortems docs/audit \
         docs/r3 docs/proposals docs/perf docs/decisions \
         src/v3 src/v4/TASKS.md src/v4/BRIEF_TEMPLATE.md \
         src/v4/CULTURE.md wip scripts/session-dashboard \
         src/v2/CM.md src/v2/CM-inventory.md src/v2/cx-violation-triage.md
  # Strip internal comments from root docs (see §5 below)
  git add -A
  git commit -m "snapshot from internal@${SNAPSHOT}"
  git push --force public main
  git worktree remove /tmp/gunbc-pub-export
  ```
- [ ] Add `public` git remote pointing at public repo
- [ ] Tag initial public release: `v0.1.0`

### What goes in the public snapshot

Keep:
- `README.md` (rewritten — see §4)
- `THESIS.md` (stripped of provenance jargon — see §4)
- `INVARIANTS.md` (stripped of operational tracking — see §4)
- `MODELING.md`, `CODING.md`, `TESTING.md`, `BOOTSTRAP.md`
- `dsl/std/`, `dsl/extdeps/`, `dsl/demos/`, `dsl/examples/`
- `src/v2/` (minus CM.md, CM-inventory.md, cx-violation-triage.md)
- `src/v4/std/`, `src/v4/extdeps/`, `src/v4/lens/`, `src/v4/compiler/`
- `.github/workflows/ci.yml` (trimmed to just the public-relevant jobs)
- `Cargo.toml`, `rust-toolchain.toml`, `Cargo.lock`

Strip entirely:
- `docs/briefs/` (368 agent work files — #1 priority)
- `docs/history/`, `docs/debt/`, `docs/review-findings/`, `docs/admin/`
- `docs/db-history/`, `docs/postmortems/`, `docs/audit/`, `docs/r3/`
- `docs/proposals/`, `docs/perf/`, `docs/decisions/`
- `src/v3/` (frozen, not part of the public story)
- `src/v4/TASKS.md`, `src/v4/BRIEF_TEMPLATE.md`, `src/v4/CULTURE.md`
- `wip/`
- `scripts/session-dashboard/`
- `tools/gen_gunbc_ci_workflow_dag/` (internal CI tooling)
- `.cursor/`

### Add internal-dir marker in this (private) repo

- [ ] Create `ops/` or `_internal/` at root and move internal dirs there —
      makes git browse cleaner even in the private repo, and the publish
      script only needs to `rm -rf _internal/` instead of a long list

---

## 3. Root doc cleanup (strip internal jargon)

These files are public-facing but contain session IDs, "operator-ratified",
T-## task numbers, and multi-paragraph embedded tracking blocks.

### `THESIS.md`

- [ ] Remove the v4 supersession block at line 5 (mentions "operator-ratified
      substrate per PR", "PM sunny-wolf-435", etc.) — replace with a simple
      "v4 is the active development phase" note
- [ ] Strip all "operator-ratified", "retracted (2026-05-15)" provenance markers
      throughout — state what the thesis IS today, not how it evolved
- [ ] Keep the intellectual content (derived homomorphism, concept unification,
      self-hosting four facets) — that's the good stuff

### `INVARIANTS.md`

- [ ] Strip the per-rule appendix table (rows like
      `neat-hawk-87 cascade 2026-05-19`, `sunny-wolf-435`, session IDs)
- [ ] Strip the multi-paragraph `GroundingMap` dissolution wall-of-text embedded
      inside P2 — that belongs in internal docs, not the invariants index
- [ ] Keep the five principles and their worked examples — those are the
      public-facing content
- [ ] Remove "escalation rule" and "when to run this" instructions (written
      for AI workers, not human readers)

### `ROADMAP.md`

- [ ] Write a 1-2 page external-facing `ROADMAP.md` covering: what works today
      (v2 self-hosted compiler), what v4 is building toward, and rough
      milestone shape — no T-## tracking numbers, no session IDs
- [ ] Move the current operational ROADMAP.md to `_internal/ROADMAP_OPS.md`

### `README.md`

- [ ] Add v4 section explaining what it is (substrate modeling + rewrite in
      progress) and what works (std/ and extdeps/ model depth, compiler stages)
- [ ] Update project structure to show v2 + v4 (currently only shows v2)
- [ ] Update quick start to use `gunbc` binary name
- [ ] Add honest v4 status: "compiler pipeline compiles and type-checks .dag;
      emission is in progress"

---

## 4. Comment stripping from code files

**Scope:** all `.dag` files in `src/v2/`, `src/v4/`, `dsl/`. Also hand-maintained
Rust in `src/v2/stage0/src/{cli_run,v2_compiler_dag_collect,rest_transport_facts}.rs`.

**What to strip:** explanatory comments that describe WHAT the code does.
**What to keep:** comments explaining WHY (non-obvious invariant, workaround,
boundary contract) — per CODING.md.

### v2 .dag files (46k lines — substantial effort)

- [ ] `src/v2/00_core.dag` — long section headers (`// ===...===`) and
      explanatory paragraphs; keep module-level header and section dividers
- [ ] `src/v2/02_parse.dag` — parser has many inline explanation comments
- [ ] `src/v2/04_infer.dag` — type inference explanations
- [ ] `src/v2/05_emit_rust.dag` — largest file (7400 lines); strip most inline comments
- [ ] `src/v2/05_emit_go.dag`, `05_emit_python.dag` — clean up TODO comments (2 each)
- [ ] `src/v2/languages.dag` — 2 TODO comments at lines 323/346; strip or fix
- [ ] `src/v2/complexity.dag` — explanatory prose
- [ ] `src/v2/ownership.dag` — same

### v4 .dag files

- [ ] `src/v4/std/*.dag` — file headers have internal status markers
      (`// Status: T-1 modeled`, `// Note: bind_outcome...`) — strip or simplify
- [ ] `src/v4/extdeps/languages/*.dag` — all have internal status + scaffolding markers
- [ ] `src/v4/compiler/*.dag` — pipeline stage files have extensive inline comments
- [ ] `src/v4/lens/*.dag` — similar

### dsl/ files

- [ ] `dsl/std/*.dag` — generally clean; light pass needed
- [ ] `dsl/extdeps/*.dag` — moderate pass needed

### Approach

A sed/awk script can remove obvious `// comment` lines in one pass.
Lines that start with `//` (after trimming) and aren't module headers or
anchors are candidates. Review pass to restore any load-bearing ones.

Rough script skeleton:
```bash
find src/v2 src/v4 dsl -name "*.dag" | xargs \
  sed -i '/^\s*\/\//d'  # removes standalone comment lines
# Then manual pass for inline trailing comments
```

**IMPORTANT — preserve load-bearing markers before running:**
- `🟡 feature:` lines — dissolution triggers (especially `src/v4/`)
- `🟢` / `🔴` lines — coproduct classification tags (required model marks)
- `// Anchor:` lines — structural anchors referenced elsewhere
- File-path/header comments — module identity lines at the top of each file
- WHY-comments (non-obvious invariants, workarounds per CODING.md)
The sed above is a **first-pass draft only** — do not run it without an allowlist-preserving wrapper or a post-run audit. Preferred: strip prose manually file-by-file, keeping the above intact.

---

## 5. Binary distribution

### Phase 1a — GitHub Releases binaries + workflow (do for June 1)

- [ ] `src/v4/workflow/release.dag` — semantic authority: six-target matrix
      (`x86_64`/`aarch64` linux-musl, `x86_64`/`aarch64` apple-darwin,
      `x86_64`/`aarch64` pc-windows-msvc), `release_published_target_triples`,
      `release_published_artifact_names` (`.exe` on windows rows), GH Release pipeline
- [ ] Hand-synced `.github/workflows/release.yml` — `v*` tag push; musl via `cross`;
      native darwin + windows runners; upload modeled `artifact_basename` assets only
- [ ] Tag `v0.1.0` from main after Phase 1a + 1b land

### Phase 1b — install (blocks public `curl | sh` / install UX; follow-on PR)

- [x] `src/v4/install/install.dag` — `InstallTarget` / OS-arch detection / env policy;
      references `release.dag` `release_published_target_triples` (no duplicate triple literals)
- [x] Emit/project `install.sh` + helper scripts from model (hand-synced interim until ShellStatic)
- [x] Re-enable install assets in GH Release bundle after install.dag lands

### Phase 2 — Homebrew tap (good for macOS users, do week of June 1)

- [ ] Create `gunb-ai/homebrew-gunbc` repo
- [ ] Write `Formula/gunbc.rb`:
  ```ruby
  class Gunbc < Formula
    desc "A causal compiler: write .dag, get Rust/Python/Go"
    homepage "https://github.com/gunb-ai/gunbc"
    version "0.1.0"
    # sha256 + url per platform
    def install
      bin.install "gunbc"
    end
  end
  ```
- [ ] Users install via: `brew install gunb-ai/gunbc/gunbc`

### Phase 3 — apt/deb (for Linux users, can slip past June 1)

- [ ] Generate `.deb` package from the musl binary using `cargo-deb` or a
      simple `dpkg-deb` script
- [ ] Option A: host on GitHub Releases as a `.deb` (simplest)
- [ ] Option B: set up an APT repo via GitHub Pages or Cloudflare R2
      (users add a source line and `apt install gunbc`)
- [ ] `cargo-deb` is the fastest path:
  ```toml
  # in Cargo.toml under [package.metadata.deb]
  depends = ""
  section = "devel"
  priority = "optional"
  ```

### Phase 4 — cargo install (for Rust devs)

- [ ] Publish `gunbc` to crates.io (requires cleaning up `Cargo.toml`
      workspace metadata, adding description/license/repository fields)
- [ ] `cargo install gunbc` just works after that
- [ ] Note: crates.io requires all workspace members to be publishable or
      explicitly `publish = false` — audit all crates

---

## 6. Other housecleaning

### Cargo.toml workspace

- [ ] Strip internal commentary from root `Cargo.toml`:
      comments like `"# v2 compiler — RESTORED 2026-05-15"`,
      `"# T-WAD Slice 4"`, `"# T-Ground-Pilot probe"` etc.
- [ ] Add `[workspace.metadata]` with `description`, `repository`,
      `homepage` fields for crates.io readiness
- [ ] Audit member crates: add `publish = false` to all internal crates
      that shouldn't be on crates.io (`v2-compiler-tests`, all `grounding_*`,
      `execute_command_bootstrap`, `gen_gunbc_ci_workflow_dag`)
- [ ] `src/v1/` — Cargo.toml says "ARCHIVED. Can be deleted." — delete it
- [ ] Remove `src/v3/` from workspace members (or just remove `src/v3/`
      from the public snapshot)

### `wip/chatgpt_reviewer.dag`

- [ ] Delete from public snapshot (internal automation bot) — or move to `_internal/`

### `src/v2/` internal design docs

- [ ] `CM.md`, `CM-inventory.md` — move to `_internal/` or delete from public
- [ ] `cx-violation-triage.md` — same
- [ ] `DESIGN.md`, `compiler-laws.md`, `parser-design.md` — these are actually
      good; keep or lightly edit for external readers

### `.github/` cleanup

- [ ] `ci-spot-rerun.yml`, `tier3-baseline-capture.yml` — check if these are
      internal-only workflows; strip from public if so
- [ ] `PULL_REQUEST_TEMPLATE.md` — currently written for internal agent workers;
      rewrite for external contributors

### `docs/` docs worth keeping for external readers

These are worth keeping or lightly editing for the public snapshot:
- `docs/architecture.md`
- `docs/v3-spec.md` (maybe — explains the language surface)
- `docs/design-pb-runtime-interpreter.md`
- `docs/demos/`
- `docs/thesis/` subdirectory (concept-unification, derived-homomorphism)

---

## 7. Timeline sketch

| Date | Milestone |
|------|-----------|
| Now | PR #3826 merged (binary rename) |
| May 29 | GitHub plan migration decision + execution |
| May 29–30 | Comment stripping pass (v2 .dag files) |
| May 30 | Root doc cleanup (THESIS, INVARIANTS, ROADMAP, README) |
| May 30 | `release.yml` CI for binary builds |
| May 31 | Create public repo, push clean snapshot, tag v0.1.0 |
| May 31 | Homebrew formula |
| June 1 | Publish |
| June 1+ | apt/deb, cargo install |
