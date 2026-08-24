# One effect, two transport readinesses — and the assumption that would have shipped a silent no-op

**Measured by `stern-boar-129` on 2026-08-24 while converting a string-bodied shell install to typed
steps. That session has since been archived, so this brief exists because the finding would otherwise
survive only in inter-session messages. Every claim below was re-verified against `origin/main` after
the fact, by a different session, before publication.**

## The finding

A single host effect — install a systemd slice unit — decomposes into two halves that **do not have the
same transport readiness**, and nothing in the corpus says so.

- The **argv half** (publish, rename, daemon-reload, start) types cleanly today on `SshShell`.
  `gunbc.host_effect_realize` routes it through the typed argv path, and `SshShell` is a supported arm.
- The **bytes half** (write the unit file's contents) is **`FleetSsh`-only**. `gunbc.typed_remote_file_write`
  `converge_typed_remote_file` and its neighbours all take `context: FleetSshExecutionContext` and shape
  their invocation through `shape_fleet_ssh_exec`. There is no `SshShell` arm.

## Why that combination is dangerous rather than merely awkward

**Every production route to this effect arrives on `SshShell`.** `gunbc.host_identity_access`
`host_identity_ssh_access` constructs `transport: SshShell { ssh_host }`, and `host_identity_srv1_access`
through `host_identity_srv4_access` are all built by it. Zero production callers arrive on `FleetSsh`.

So a conversion that assumes *this effect is typeable on the transport we use* — a per-module,
per-effect assumption that reads as obviously true — produces code that:

- **compiles clean**, because both halves are individually well-typed;
- **passes its witnesses**, because a witness that supplies a `FleetSshExecutionContext` exercises the
  path that works;
- and **refuses on all four hosts in production**, because no caller ever supplies one.

That is a green-looking PR that silently does nothing, everywhere. Worse, the effect it would have
replaced is the one whose work item exists *because the slice is modeled but unreachable* — so the
failure mode is the original defect, reintroduced one layer down, wearing the repair's clothes.

## How it was caught, which is the transferable part

Not by care, and not by a gate. By one question, asked in a specific form:

> **Does any production path reach this via that transport *today*?**
> Answer it from **what a caller passes**, never from **what the enum can express.**

The enum reading is always cheaper to reach and always available — `HostEffectTransport` has an
`SshShell` arm, therefore `SshShell` is supported, therefore the conversion is safe. That reasoning is
locally valid and produces the wrong answer, because the arm's existence and the arm's *reachability
for this operation* are different facts with different owners.

The two questions diverge silently. This is the same shape DESIGN records as **reachability read as
occupancy**, arrived at from the opposite direction: there, a live guard is deleted because nothing
currently lands in it; here, a dead path is adopted because the type says something *could*.

## The state this leaves, stated rather than left to be rediscovered

- The string-bodied install **is on main**. `gunbc.host_effect_realize` `compile_pool_slice_install_body`
  builds an executable program as a `String` and runs it through `shell_exec_via_bash`. It carries a
  self-authored dissolution trigger naming `#5828`.
- The correct end state is **already claimed elsewhere**: `extdeps.ssh.session`'s
  `ssh_exec_portable_words_supersedes_exec_argv_note` dissolve-on names `host_effect_realize` *first*
  among the consumers that must route through a `FleetSshExecutionContext`.
- The repair that was designed and never landed — typed argv for publish/rename/reload/start, plus a
  **typed, located, countable refusal** for the bytes half — **has no owner**, because the session that
  designed it was archived when its PR merged.

**The refusal must name both the missing `FleetSshExecutionContext` and the transport that *is*
present.** This is the requirement most likely to be lost between now and whoever picks the repair up,
and it is not a presentation preference: a refusal that says only *no context* cannot be joined to
`extdeps.ssh.session`'s dissolve-on row without re-deriving the measurement above — and preventing that
re-derivation is the entire reason this brief exists. A refusal naming one half of a two-transport
seam is a diagnostic about a fact nobody disputed.

That last point is the reason for the brief. **A deficit that is real, landed, and uncounted has a
frequency of zero by construction**, and the lane holding the correct end state keeps its obligation
with no evidence that anyone needs it discharged. Nothing on main currently says that the bytes half
cannot be reached in production.

## One adjacent receipt, since it was found in the same conversion

The original publication step was `sudo install -m 0644 staged <dest>`, with a read-back-and-compare
performed on the **staged** file. That verifies the wrong artifact: `install` opens the destination and
writes through it rather than renaming, so `<dest>` passes through a truncated state that the staged
read-back cannot see, and `<dest>` is the only file systemd ever opens.

Measured: `printf 'OLDCONTENT' > dst` → inode `8280789`; `install -m 0644 src dst` → inode `8280789`,
**unchanged**. An atomic publish produces a *new* inode, because `rename(2)` replaces the directory
entry rather than the file's contents.

The repair is to publish to a temp name in the **same directory** and rename. Note the constraint that
makes it work: same-directory guarantees same-filesystem, and a cross-filesystem rename degrades to
copy-and-unlink, silently reopening the window.

> This measures a mechanism, not an incident. No evidence exists that systemd ever read a partial unit
> here. It is a hole in the *claim* — "the window is closed" — rather than a demonstrated production
> failure, and it is worth fixing for that reason alone: an overclaimed guarantee never ranks for
> climbing.
