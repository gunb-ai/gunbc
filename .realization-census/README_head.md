# Realization-decision census

Pin `1dd3eda5696a` (origin/main, 2026-10-24). This census classifies; it does not redesign or fix anything.

The question it answers: **how much of gunbc decides for itself where its data lives, what it caches, how it reaches a resource, what authority it acts with, and which host it runs on**, instead of receiving those facts from the one place that should decide them.

## How to read the files

| file | one row per | identity column |
|---|---|---|
| `classes.tsv` | error class (read this first) | `class_id` |
| `modules.tsv` | hand-read module | `module` |
| `findings.tsv` | instance: one decision at one symbol | `finding_id` |
| `receipt.json` | the run: pin, script hashes, slice commits, populations, agreement figures | — |
| `method.md` | how every number was produced, every merge ruling, every caveat | — |

Each dimension has a **decision site**:

| site | meaning | problem? |
|---|---|---|
| None | the module decides nothing in this dimension | no |
| Home | the module is the one declared home of the fact | no |
| Passed | the module receives the fact as a parameter | no |
| Rederived | it recomputes or re-selects a fact it could have been handed | yes, mild |
| Literal | it hard-codes the fact (path, host, user, port, env name) with no home behind it | yes |
| Forked | it re-implements a shared mechanism beside the library (its own write, lock, mkdir, sweep, memo) | yes |

The L number (L0 to L5) is kept beside each site for sorting. A module's site in a dimension is its worst instance there. Every problem cell is backed by named rows in `findings.tsv`, except 7 reader cells listed in `merge_report.txt`.

## The reading, in plain language

**Most of the problem is storage, and most of storage is the same three habits.**

- Kept records are written with a bare file write where a store library already exists (`storage_raw_state_beside_library`: 57 modules, 135 instances).
- Store roots are spelled out as literal paths: `/var/lib/gunbc/...` appears in 22 modules (`storage_root_literal`: 62 modules).
- Receipts go to repo-relative `target/...` paths for which no home exists at all (`storage_relative_receipt_path`: 55 modules).

Beside these, modules make their own directories, delete their own trees and build their own create-once locks (`storage_hand_directory_setup`, `storage_hand_retention_sweep`, `storage_hand_exclusion`: 34, 31 and 37 modules).

Raw `Filesystem.Write` is the single most duplicated mechanism in the corpus: 79 modules.

**Placement is the second problem, and it is two habits.**

- Hosts, units and ports are named literally: `srv1` in 20 modules; unit names and `/etc/systemd/system/` in 35 (`placement_host_literal`, `placement_unit_name_or_dir`).
- An entry point's instance or host is re-selected further down, the class behind the wrong-instance bug: `srv1_live_dashboard_instance()` wrappers, one function per host, `if host == operator_host_srv1` guards (`placement_instance_repick`: 45 modules). The brief's 26 wrapper pins are a subset of these. The roadmap slice found about 15 more.

**Access is mostly ambient environment and literal secret paths.**

- Modules call `shell.Env.Get` directly to find a root, token or identity: 38 modules.
- Credential and key locations are written as literal paths, some beside a `SecretRef` for the same secret (`access_credential_location`: 37 modules).
- Kernel-interface paths (`/proc`, `/sys`) are read outside the `extdeps` models of those interfaces (`access_kernel_abi_path`: 15 modules).

**Authorization is small, but it is ambient root.**

- 12 modules elevate with `sudo` in place (`authz_ambient_sudo`) rather than receiving a capability.
- `gunbc.auth.privileged_effect_census` already classifies such sites, which is why that class sits at rung 1 rather than 0; it refuses none of them.

**Caching is a seed problem.**

- Production `.dag` code barely caches by hand (2 modules).
- The hand-written Rust seed has process-wide memo tables with hand-picked keys and no invalidation in 21 files (`caching_process_memo_table`).
- One emitter template wraps every generated constant in a memo: one finding, not 90 (`caching_generated_constant_memo`, `src/v1/05_emit_rust::emit_data_def`).

**The extdeps layer is mostly clean.** 70 of the 123 extdeps modules read are the faithful home of an upstream fact. 9 have a problem:

- 5 carry our own policy inside an upstream model (install paths under `/var/lib/gunbc/bin`, `/opt/actions-runner`, the `srv3` install-actuator label, the sccache install location);
- 3 hand-roll a temp-directory lifecycle;
- 1 re-spells its own binary path.

**By module class:**

| class | hand-read | with a problem | worst site Rederived / Literal / Forked |
|---|---:|---:|---|
| production | 681 of 2,226 | 318 | 31 / 187 / 100 |
| extdeps | 123 of 1,129 | 9 | 1 / 5 / 3 |
| seed (hand-written) | 71 of 315 | 42 | 0 / 2 / 40 |
| seed-generated | 90 | 0 | one template finding |
| compiler, instrument, ledger, test, roster | counted, not read | — | — |

## The shape of the fix, per decision site (the brief's wording, not a design)

- **Rederived: receive the value instead of re-deriving it.** The caller already holds the instance, store or host, so passing it removes the second derivation.
- **Literal: declare once and consume.** The fact gets one home (an existing authority where there is one, named in `owning_authority`) and every other spelling of it goes away. Where `owning_authority` says "none exists", the missing home is named and nothing is invented.
- **Forked: adopt the library and delete.** The module's own write, lock, mkdir, sweep or memo is replaced by the store, CAS, directory or materialization library it sits beside.

`classes.tsv` carries, per class:

- the current rung on DESIGN.md §4b's ladder;
- the attainable ceiling, with reason;
- the wall that would hold it (an `admit_callers` roster, a type, a declared `requires`, or derivation from one declaration);
- the red control that would prove that wall;
- the residual, and the next trigger.

## What this census does not say

- **A rung of 0 means "no wall observed by this census", not "proved unguarded".** The census looked for decision sites, not for gates that already refuse them.
- **The hand-read depth is uneven.** The large modules in slices 03, 04, 05, 06 and 09 were read by targeted search around every lead plus a declaration scan, not line by line. Their None/Home/Passed cells are lower confidence than their problem cells. Slice 06's specimens describe the mechanism and name the calls rather than quoting lines. `method.md` lists the depth per slice.
- **The no-signal remainder is bounded, not cleared.** All 82 modules matching the two shapes the mechanics missed were read: 12 have a problem. For the other 1,551 no-signal production modules, a sample of 51 found none, so their miss rate is under about 6% at 95% confidence. That is up to roughly 90 unread modules, of unknown shape.
- **The worst-module order is a reading order, not a severity model.** It sums problem levels across dimensions.
- **Seeds were recomputed, not copied.** The parent's precomputed seeds were unreachable across hosts, so every lead here is recomputed at the pin, and the cross-check against those seeds was never done.

