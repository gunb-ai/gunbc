# Dependency acquisition: why every new dependency is a modeling project

**Status:** gap analysis, 2026-09-07. Reached from a concrete blocker — the approval loop needs an
ntfy server on srv1, and "install ntfy" turned out to be a slice rather than a data row. The
question this answers is the operator's: *the convergence is supposed to handle this; what is the
gap from here to there?*

**Framing correction, operator, 2026-09-07:** fleet convergence "was never supposed to be host keys
— I think we started making a world convergence but got halfway through like all things." That is
the right frame and this document adopts it. What follows is not "convergence lacks a feature"; it
is an inventory of which half got built.

---

## 1. The finding

**There is no `Dependency` concept.** There is no type whose inhabitants are "a piece of software
this fleet depends on", and consequently no single place that answers *where does it come from, how
is it verified, and how do we know it is present*.

What exists instead is `gunbc.host_effect` `HostEffect`, a closed coproduct of **34 arms**. It is
genuinely the world-convergence spine the operator describes — it has arms for host OS, BMC,
deploys, runners, compile pools, and OS install media. But its vocabulary is **one arm per
instance, not one arm per concept**, and at least eight of the 34 arms mean the same thing:

`OsInstallActuatorToolchainEnsure` · `EnsureBuildCacheInstance` · `ProvisionCodexRuntime` ·
`EnsureCompilePoolSlice` · `Srv3SeededInstallMediaToolchainEnsure` · `LiveDeployEnsureDependency` ·
`EnsureActionsRunnerRelease` · `EnsureRunnerSlotDirectory`

Every one of those is *ensure some software is present and configured on a host*. A world
convergence with one arm per thing in the world is an **enumeration wearing a convergence's name**,
and its cost per new dependency is constant — it never amortizes. That is DESIGN §2 exactly: the
same work, duplicated once per dependency, priced as if each were new.

## 2. The census — six dependencies, five mechanisms, five homes

| dependency | acquisition mechanism | authority home | actuated through |
|---|---|---|---|
| tailscale, tmux | apt package, `dpkg -s` guard then `apt-get install` | `live_deploy.spec` `EnsuredDependencyKind` | `LiveDeployEnsureDependency` |
| docker-ce | third-party apt repo + **pinned GPG fingerprint** | `extdeps.container.docker_ce` | `runner_host_docker_provision_script` |
| sccache | pinned GitHub release tarball + **per-arch sha256** | `extdeps.cache.sccache` | `EnsureBuildCacheInstance` |
| actions-runner | GitHub release artifact | `extdeps.github.actions_runner` | `EnsureActionsRunnerRelease` |
| codex | npm registry + lockfile integrity | `gunbc.package_delivery` (3,391 lines) | `ProvisionCodexRuntime` |
| **npm itself** | **a prose string** | `gunbc.host_cli_dependency` | **nothing** |

The verification posture is good and was clearly reasoned each time — docker's keyring fingerprint
is pinned against a published fact, sccache's tarball is sha256-verified per architecture. **The
defect is not rigor; it is that the rigor was re-derived from scratch six times**, and lives in
five modules that share no vocabulary.

### The last row is the class the operator named

`gunbc.host_cli_dependency` carries the fallback for a dependency with no bespoke arm:

```
data codex_wet_materialization_host_cli_requirements: List<HostCliEnrollmentRequirement> = [
  HostCliEnrollmentRequirement {
    tool: "npm",
    provision: "apt install npm on falsifier cadence runner image",
  },
]
```

`provision` is a `NonEmptyStr` — **an instruction to a human, typed as a string**.

It is worth being precise about what is and is not wrong here, because the obvious reading is the
wrong one. The module's *refusal* behaviour is correct and deliberately so: an absent tool stops
with a typed `HostDependencyAbsent` carrying tool and hint, and the module's own annotation says
"absent npm stops as HostDependencyAbsent, **never auto-install inside the witness**". That is
§5 fail-closed, and auto-installing inside a witness would be the absorbing fallback.

**The defect is that the refusal has no route to a fix.** It hands a person a sentence, and there is
no modeled thing that sentence refers to — so the convergence cannot act on it, no census can count
what is unprovisioned, and nothing goes red when the hint rots. An unmodeled dependency is invisible
to every instrument that would otherwise rank it for work.

## 3. The gap is already named by the corpus

This is not an unrecognized problem. `extdeps.cache.sccache` states it in its own dissolution
trigger:

> It dissolves when `host_effect_apply` binds a **verified-download-plus-install effect
> (FetchAndVerify then InstallBinary)** without the shell leaf, at which point
> `sccache_install_script` is deleted rather than edited.

Neither `FetchAndVerify` nor `InstallBinary` exists anywhere in the corpus — grep returns exactly
that one comment. The missing concept was identified, its dissolution trigger was written, and the
capability was never built. `extdeps.container.docker_ce` carries a parallel trigger for the same
reason.

So `sccache_install_script` and `docker_ce_repo_install_script` remain **shell leaves**: install
sequences assembled as strings, typed as `String`, with the modeled row supplying only version and
digest. The shell-leaf residue and the missing `Dependency` concept are **one gap seen twice**.

## 4. What "install ntfy" costs today

Concretely, why the blocker is a slice and not a row. Adding ntfy through the existing shape
requires:

1. a new `extdeps` module for the release, its version and per-arch digests (sccache's shape);
2. a **35th `HostEffect` arm**, plus its realize handler and its plan handler;
3. an actuation site — `live_deploy` (scheduled for deletion) or `runner_host_deploy` (runner-scoped, and ntfy is not a runner concern);
4. a shell-leaf install script that lands already carrying a dissolution trigger it cannot discharge;
5. a systemd unit, which has no general home either — units are emitted per service family (`RunnerUnitStanding`, `LaunchUnitStanding`) rather than by one authority.

Five pieces of bespoke modeling to install one binary — and every one of them is work the *next*
dependency will do again from scratch. **That is the cost the operator is feeling, and it is a real
cost rather than an impression.**

## 5. The route

The shape of the repair follows from §2: model the concept once, derive every use.

**Phase 1 — `Dependency` in `extdeps`.** Identity, version, and an acquisition source as a
coproduct over the mechanisms already in evidence:

```
AptPackage { package }
AptRepoPackage { package, repo_url, keyring_url, keyring_path, keyring_fingerprint }
PinnedRelease { url_template, digest_by_arch }
NpmPackage { name, lock_integrity }
```

Every arm is a shape the corpus already implements correctly somewhere — this is decompression and
reduction of existing rows, not invention. Verification stops being per-dependency diligence and
becomes a property of the arm: a `PinnedRelease` **cannot be constructed without a digest**, which
moves supply-chain verification from §4b rung 1 (each author remembered) to rung 4 (unwritable
otherwise). That is the safety payoff, and it is larger than the DRY payoff.

**Phase 2 — one arm.** `EnsureDependency { dep: Dependency }` replaces the bespoke arms; the six
existing dependencies become **data rows**. This is a §3 replacement migration and it must cut at
the root: build the arm, move all six at once, delete the old arms and their shell leaves in the
same motion. Six is a tractable atomic cut, and a surviving `EnsuredDependencyKind` would be an
attractor — every later dependency would be answered in its vocabulary.

**Phase 3 — the prose route closes.** `host_cli_dependency`'s `provision: NonEmptyStr` becomes a
reference to a `Dependency`. The refusal then *names the acquisition that would satisfy it*, so an
absent tool is a routable obligation rather than a sentence, and unprovisioned dependencies become
countable.

**Phase 4 — ntfy is a data row**, and so is the next one.

**Ordering note.** Phase 1 is worth landing on its own even if Phase 2 waits: it is additive, it
gives the digest-by-construction win immediately, and the six existing rows can migrate one at a
time *into* it. Phase 2's cut is what must be atomic — not the whole program.

## 6. What this does not settle

- **Units.** `EnsureDependency` gets the binary onto the host; it does not run it. Unit emission has
  no general authority either (`live_deploy` for apps, per-family standings elsewhere), and that is
  a **separate half-built convergence** deserving its own analysis. The ntfy work needs both.
- **`live_deploy`'s disposition.** It holds app deployment members and the only working apt-ensure
  path, and is scheduled for deletion with no declared cut. Phase 2 would take the apt path out of
  it, which makes the cut smaller — but does not make it.
- **ntfy's actual packaging.** Whether ntfy ships an apt repo, a `.deb`, or only a release tarball
  was **not verified while writing this** (no apt available in the session container). It changes
  which arm the row uses and nothing structural, but it should be checked rather than assumed
  before the row is written.
