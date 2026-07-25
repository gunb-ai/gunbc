# extdeps positioning restructure (§3 hygiene)

**Status:** authoritative scope doc — the durable carrier for this restructure.
**Origin:** operator positioning audit (2026-07-25) + coordinator (snappy-moth-330) rulings.
**Nature:** pure `git mv` + module/import repoint. **Zero behavior change. Zero service-name changes. One PR.**
**Sequencing:** fires after D2 #7192 and D3 #7193 merge (both merged 2026-07-25); lands **before** the namespace import-deletion (Dispatch-2).

This doc is the authoritative brief. Execute the moves below against it; if any move forces a
behavior change or a service-name change, STOP and escalate — that is the signal it is not a pure
positioning move.

---

## Why (the §3 test)

A directory under `extdeps/` is legitimately either (a) a single **upstream** (one cited dependency,
its files) or (b) a **category-namespace** of per-upstream cited files. Two violation shapes:

- one upstream split across use-categories (e.g. OpenSSH appearing under both `access/` and `diagnostic/`);
- a directory that nests **by use, not by authority** (a grab-bag: general-purpose tools filed under
  `os/` because they run on an OS, rather than under their own upstream home).

The `extdeps/os/` directory is the flagship grab-bag; `diagnostic/` is the same class. This PR
de-grab-bags them and consolidates the split upstreams, with no change to what any service *does*.

---

## SCOPE (findings 1–4 + os/ de-grab-bag; finding 5 = later follow-ups)

### os/ de-grab-bag

`os/` today holds identity taxonomy, systemd, userland tools, a Linux ABI, provisioning media, and an
exec limit — a grab-bag. Split it:

- **`extdeps/systemd/`** — extract `systemd`, `systemctl`, `systemd_contracts`, `oomd` (the systemd
  upstream; D1's `ListUnits`/`Status` additions ride `systemctl.dag`). `os.Id` moves with `id.dag` to
  the userland-tools home.
- **userland tools** — `hostname` + `free` → the `tools/` home (beside the consolidated coreutils; see finding 3).
- **`extdeps/linux/`** — `proc_meminfo` (Linux `/proc` ABI).
- **`extdeps/provisioning/`** — the 6 `ubuntu_*install_media*` files.
- **`extdeps/exec/`** — `exec_arg_limit`.
- `os/` keeps ONLY the identity taxonomy (`ubuntu`/`windows`/`macos` leaves) + dispatch; the hub
  `os.dag` goes **LOOSE** (finding 4 below).

### Finding 1 — OpenSSH unify

`access/ssh.dag` + `diagnostic/ssh.dag` (holds `service ssh.Session`, C5 `ExecArgv`) → one
`extdeps/ssh/` home. One upstream, one dir.

### Finding 2 — `diagnostic/` dissolves (nests by use, not authority — same class as os/)

- `edac` → `extdeps/linux/`; `ipmi` → beside `extdeps/bmc/`; `ssh` → finding 1.
- per-dir `mock_corpus` siblings move WITH what they mock.

### Finding 3 — tool-home convention [SIGNED]

- **Convention (now + standing):** citation grounds the upstream; a CLI upstream is a leaf
  `tools/<x>.dag`, promoted to its own `<x>/` dir at ≥3 files.
- **Apply now:** consolidate the gnu-userland split — `gnu_coreutils`, `diffutils`, `sed`, `grep` all
  under `tools/` (own dir only if ≥3 files); `shell/` shrinks to exactly the exec/credentials seam.

### Finding 4 — hub-file placement [SIGNED: LOOSE, operator 2026-07-25]

- **Convention:** a category hub is a **LOOSE** file at the parent (`extdeps/os.dag` declaring
  `module extdeps.os`), sitting beside its `os/` leaf subdir — the cpu/memory/vendor/bmc exemplar.
  NOT `os/os.dag` (`extdeps.os.os` is a nickname to kill), and NOT `os/os.dag` carrying a minimal name
  (path⇄name incongruence for zero benefit — the exact hand-coincidental binding the module-identity
  lane wants derived).
- The live tree has **three** variants of this fork (`os.os`/`cache.cache` nicknames; `cpu`/`memory`
  loose; `tools/tools.dag` in-dir-minimal); the resolver does NOT enforce path↔module congruence.
  LOOSE strictly dominates and matches the tree majority.
- **Three hub moves, priced by size — DO NOT lump them:**
  - `tools/tools.dag` → `extdeps/tools.dag` — **PURE git mv.** Module already `extdeps.tools`; name
    unchanged; ZERO import repoints.
  - `os/os.dag` → `extdeps/os.dag` — **REAL rename** `extdeps.os.os → extdeps.os`; repoint every
    consumer's import.
  - `cache/cache.dag` → `extdeps/cache.dag` — **REAL rename** `extdeps.cache.cache → extdeps.cache`;
    repoint every consumer's import. (ONLY the hub rename joins this PR; the `cache/` cited-vs-internal
    file split stays finding 5.)
  - Price the import-repoint sweep on **os/ and cache/ only**; tools/ carries none.

---

## RAILS (hard)

- **`shell/exec.dag` and `shell/credentials.dag` DO NOT MOVE** — the meta-exec confinement wall's named
  target; that move belongs to the sequestration milestone, and touching it churns lens rosters.
- **ZERO service-name changes anywhere.** (`os.Hostname`, `systemd.Systemctl` service-vs-folder
  reconciliation is the NAMESPACE lane's job — noted, NOT churned. Receipt that this is discovery not
  invention: the service already declares `service systemd.Systemctl`; the move makes the folder agree.)
- **compile-clean + affected witnesses green by execution; one PR.**
- **DISJOINT from the SCM-economics PR** (`extdeps/sec/`, `extdeps/pricing/`): this bundle's move set
  (tools/shell/os/cache/render/runtime + gnu-userland) touches neither, so no ordering constraint either way.

---

## NOT in this PR

- **Finding 5** (cited-vs-internal splits in `cache`/`render`/`runtime`): per-dir follow-ups,
  destination judgment per file, not pure git-mv.
- **Periphery charter** (realization/transports/communication): operator call, grounding-inventory lane.
- **`src/v2/extdeps`**: the two-std de-fork lane owns it.

---

## Escalation

Escalate to the parent (snappy-moth-330) on any doubt — `dashboard-message send --to snappy-moth-330 --body "..."`.
Do NOT route around a parse/resolver obstacle: that is the unmarked-workaround anti-pattern (DESIGN §5).
The right two landing states are the real move or a declared, escalated blocker — never a dodge.
