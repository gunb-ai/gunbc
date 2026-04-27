# R2 Release — B7 priority-hint relay to Pure Bootstrap Manager `(cross-manager signal, not a worker brief)`

> **Cross-manager signal documentation.** Per [`docs/briefs/debt-paydown-synthesis-2026-04-25.md` §5 (lines 526-528)](debt-paydown-synthesis-2026-04-25.md) + [`docs/r2-structure.md` §"R2 Release Manager"](../r2-structure.md) Goal 5 / B-wave Tier 2. **NOT a worker brief.** Documents the signal that R2 Release Manager relays to R2 Pure Bootstrap Manager once both spawn at R1 close. Pre-spawn authoring per inbox #828 PM portion.

## Why this is a signal-doc, not a worker brief

The R2 Release Manager owns release coordination and surfaces signals across managers per [`docs/r2-structure.md` §"R2 Release Manager"](../r2-structure.md). The synthesis doc names B7 as: *"Priority hint to Zero-Floor Manager: lift `patch_lower_helpers_*` retirement to PB-Tier1-Sweep priority."* (Now: post-R2-spawn, this routes to **R2 Pure Bootstrap Manager**, which migrates from / supersedes the prior Zero-Floor Manager per PR #827's manager-rework.)

Worker briefs author concrete code-or-doc deliverables. Cross-manager signals route priority/scope information without authoring deliverables themselves — the receiving manager dispatches the actual work. B7 is the latter shape.

This file documents the signal content + delivery discipline so that when R2 Release Manager spawns (post-R1 close), the relay is dispatchable as a single-line cross-manager queue entry rather than a fresh authoring cycle.

## Signal content

**From:** R2 Release Manager
**To:** R2 Pure Bootstrap Manager
**Trigger:** R2 spawn (both managers spin up at R1 close per [`docs/r2-structure.md` §"Transition mechanics"](../r2-structure.md))
**Channel:** cross-manager queue per the R1 `Cross-manager notifications queued` brief pattern (per [`docs/r2-structure.md` §"Manager structure"](../r2-structure.md) escalation signal channel discipline)

**Payload:**

> Priority hint: lift `patch_lower_helpers_generated_type_alias_refinement` retirement to **PB-Tier1-Sweep priority** within R2 Pure Bootstrap Manager's owned-deliverable queue.
>
> Authority for this priority hint:
> - [`docs/briefs/debt-paydown-synthesis-2026-04-25.md` §"Tier 2" item 7](debt-paydown-synthesis-2026-04-25.md) — names this as *"the 'first PB cleanup target' framing already in the row"* (originally PR #809).
> - [`docs/briefs/debt-paydown-synthesis-2026-04-25.md` row at line 120](debt-paydown-synthesis-2026-04-25.md) — current row for `patch_lower_helpers_*`: dissolution trigger reads *"first PB cleanup target once generated `lower_helpers` can emit the refinement field natively."*
> - [`docs/briefs/r2-pure-bootstrap-manager.md` §"Owned deliverables"](r2-pure-bootstrap-manager.md) (lands on PR #835 merge) — names **"Tier 2 patch_lower_helpers_* retirement (if survives R1)"** as a post-R1 owned deliverable; this signal lifts its priority within that owned-deliverable set.
>
> Suggested ordering: dispatch this retirement as one of the first R2 PB cleanup workers; the bridge it retires (`lib.rs:1143-1180` exact-string rustfmt patching) is known-fragile and the retirement is unblocked once `lower_helpers` can emit the refinement field natively (per the named dissolution trigger).

**Fulfilled 2026-04-27:** PR #1014 retired the helper and both lower-helpers patch call sites after generated `lower_helpers` emitted the `refinement` field natively. This B7 priority hint is closed; it remains here as historical relay context.

## Pre-spawn vs post-spawn

- **Pre-spawn (now):** this file documents the signal content. No relay action — both managers are PROPOSAL state.
- **Post-spawn (R2 promotion):** R2 Release Manager queues this signal in the cross-manager queue as one of its first dispatch actions. R2 Pure Bootstrap Manager acks and adjusts its owned-deliverable priority ordering accordingly. This file becomes a **closure receipt** — the signal-content snapshot that the relay implements.

## Closure trigger

This signal-doc's purpose dissolves when:
1. R2 Pure Bootstrap Manager dispatches the `patch_lower_helpers_*` retirement worker, OR
2. R1 closes with `patch_lower_helpers_*` already retired (in which case the signal is moot — synthesis-doc row marks the dissolution as already-fired).

In either case, the signal-doc's content has been delivered and consumed. Mark this file as RESOLVED in a follow-up doc-cleanup PR; do not preserve it past its purpose.

## Cross-refs

- Parent: [`docs/briefs/debt-paydown-synthesis-2026-04-25.md` §5](debt-paydown-synthesis-2026-04-25.md) (B-wave Tier 2 dispatch ordering).
- Source row: [`docs/briefs/debt-paydown-synthesis-2026-04-25.md` line 120](debt-paydown-synthesis-2026-04-25.md) (`patch_lower_helpers_*`).
- Receiving manager: [`docs/briefs/r2-pure-bootstrap-manager.md`](r2-pure-bootstrap-manager.md) (lands on PR #835 merge).
- Sending manager: [`docs/briefs/r2-release-manager.md`](r2-release-manager.md) (lands on PR #835 merge).
- Channel: [`docs/r2-structure.md` §"Manager structure"](../r2-structure.md) escalation signal channel discipline.
- Originating issue: PR #809.
