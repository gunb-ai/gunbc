# Dashboard Control-Plane Incident Ledger — 2026-05-30

**Manager session:** `sharp-otter-407` (Close/Receipt lane).
**Authority:** PR #3983 §7 MW-D6 — *"ONE bounded dashboard-control-plane incidents audit under Close/Receipt or PM; NOT compiler critical path"*.
**Scope:** five patterns surfaced 2026-05-30. Bounded; no compiler-critical-path work. **This is a session-layer incident audit, NOT a substrate / `.dag` / Rust change.**

Each row: pattern → observed symptom → primary evidence → impact on managers/workers → recommended dashboard fix. Evidence column cites specific session-level incidents wherever this lane could witness them; patterns the Close/Receipt lane has not directly witnessed are marked **[indirect]** and rely on the PM brief naming the pattern.

---

## §1. Incident ledger

### §1.1 `closeout-leaf-placeholder` — closeout-leaf auto-spawn misinterprets title as authoring directive

| Field | Value |
| ----- | ----- |
| **Symptom** | The "manager-session persistence" pattern attaches a closeout leaf-task to a role-node to keep it open past all children DONE. The dashboard then auto-spawns a worker against that closeout leaf. The spawned worker cannot tell that the leaf title is a no-op placeholder vs an authoring directive, and starts authoring substrate. |
| **Primary evidence** | 2026-05-29 — `smart-wolf-496` attached `lively-owl-291` as a closeout leaf for the Determinism-as-effect lane. `lively-owl-291` read its closeout title as an authoring directive and pushed PR #3921, which directly violated the ratified Determinism-as-orthogonal-substrate design (folded Determinism into existing `OperationEffect` instead of the authoritative separate-carrier-file approach landed in PR #3922). Required PM-level intervention + close-PR authority + corrective coordination. |
| **Impact** | A rogue PR that contradicts a recently ratified design lands at the boundary of a manager's lane; cleanup is expensive (force-archive, supersession PR close, design-defense correspondence). Failure mode is silent — the placeholder *looks like* normal child activity. |
| **Recommended fix** | (a) Dashboard recognises a structured `CLOSEOUT_PLACEHOLDER` work-item flag and does NOT auto-spawn a worker against flags so marked. (b) Until that lands: title-convention workaround documented in this lane's memory — prefix closeout titles with `CLOSEOUT-PLACEHOLDER (DO NOT AUTHOR):` and send a stand-down message within ~30s of leaf creation. (c) Dashboard's draft-PR-attach nudge against a placeholder session is a high-signal indicator the worker has misread the title — surface as an alert. |

### §1.2 `auto-open-on-undeleted-branch` — duplicate auto-PR opened against a branch with an existing live PR

| Field | Value |
| ----- | ----- |
| **Symptom** | The dashboard auto-opens a draft PR for a session's branch as soon as one is detected. If the session's original PR has not yet been closed/merged AND the branch still exists, the dashboard opens a second PR against the same branch — a duplicate. |
| **Primary evidence** | 2026-05-30 03:53Z — PR #3949 (Close/Receipt manager-pass) was merged at 04:00Z, but the branch `session/sharp-otter-407` was not deleted at merge time (squash-merge default behavior). Subsequently the dashboard auto-opened PR #3966 against the *same* branch with the *same* two files, generating a duplicate. This lane closed #3966 manually as a duplicate of the already-merged #3949. |
| **Impact** | Reviewer + dashboard cycles spent on a duplicate PR that adds nothing; risk of accidental "ready" flip on the duplicate which could trigger another review wave; manager has to author a close-as-duplicate comment. Cosmetic but real cost. |
| **Recommended fix** | (a) Suppress dashboard auto-open when the head branch already has an open OR recently-merged PR (within last N minutes). (b) Or: delete-branch-on-merge by default (squash-merge with branch deletion) so the trigger never fires. (c) Either fix reduces the duplicate-PR class to zero without changing manager flow. |

### §1.3 `title-truncation-blocks-spawn` — work-item title truncation strips load-bearing brief content

| Field | Value |
| ----- | ----- |
| **Symptom** | Work-item titles are truncated mid-sentence when stored or relayed through dashboard messaging. The trailing portion (often the *active verb* of the directive) is dropped. Spawned workers / re-spawned successor sessions read only the prefix and lose dispatch shape. |
| **Primary evidence** | This very session's own work-item title: `"Close/Receipt Manager — authority over close predicates, two-axis disposition vocabulary (ship_disposition × engineering_state), ladder↔questionnaire complementarity, anti-shelfware deadline policy per PR #3938 https://github.com/gunb-ai/gunbc/pull/3938 §11.1. NO implementation work — adjudicate rec"`. Truncated mid-word at `"adjudicate rec"`. The complete original directive (recoverable only because the PM re-sent it via dashboard-message) ended `"...adjudicate receipts only"`. A successor session re-spawned on this role would have read only the prefix and not known whether its mandate was authority or implementation. |
| **Impact** | Successor / re-spawned sessions can lose the disambiguating fragment of their brief. For manager roles where the truncation falls on the authority verb (this case: "NO implementation work — adjudicate **recei**\[pts only\]"), the brief becomes ambiguous and a fresh agent may improvise into the wrong lane. |
| **Recommended fix** | (a) Raise the title-storage limit, OR (b) store full title + a separate truncated display field, OR (c) when truncation occurs, append `…` and require the brief body (separate field) to carry the full directive — and have spawn-time brief assembly pull the body, not the title. The current system's spawn-time briefing relies on the title in ways that the truncation defeats. |

### §1.4 `nudge-flips-held` — count-nudge to a manager whose inbox is digesting reads as a delivery failure to senders **[indirect]**

| Field | Value |
| ----- | ----- |
| **Symptom** | When a manager session has reports, messages to that manager are HELD and coalesced into a throttled digest. The sender's `dashboard-message send` returns `state: held / delivered: false`. The dashboard prints a NOTE explaining this is backpressure, not failure — but the structured return shape still includes a `delivered: false` that downstream tooling or alerting may consume as an error. |
| **Primary evidence** | Every outbound message this lane sent to `nimble-dove-733` returned `state=held delivered=false`. The accompanying NOTE clarifies "Held ≠ lost"; the recipient gets a count-nudge and pulls bodies with `dashboard-ops messages mine`. Sender-side correct behavior is to trust the held state, not retry. **[indirect on the "flip" specifically — this lane has not seen a transition between held and delivered for the same message.]** |
| **Impact** | (a) Tooling that consumes the structured result as success/failure boolean may treat `delivered=false` as a transient error and retry — defeating the digest backpressure. (b) Sender-side anxiety / unnecessary `--priority high` escalations to bypass digest. (c) "Should-I-resend" decision noise across the manager tree. |
| **Recommended fix** | (a) Introduce a third state alongside delivered/held: e.g., `delivered_to_digest: true`, distinct from `delivered: false`, so callers can tell normal-backpressure from actual-failure. (b) Document the held-vs-delivered shape in tool descriptions so callers don't have to read NOTE prose. (c) Optionally: surface in the sidebar that a held message is in the digest queue — visible enough that senders see it without re-checking. |

### §1.5 `spawned-worker-not-in-messaging-table` — newly-spawned worker temporarily unmessageable **[indirect]**

| Field | Value |
| ----- | ----- |
| **Symptom** | When the dashboard auto-spawns a worker for a fresh work-item, there is a window during which the worker's session id is allocated but not yet present in the messaging-routing table. Attempts to `dashboard-message send --to <new-session-id>` during that window fail to address the session. |
| **Primary evidence** | **[indirect]** This lane has not authored a child this session. The pattern is named in the PM brief; the failure mode is consistent with the dashboard's "spawn within ~30s" cadence vs the eager-message-immediately-after-create flow several other managers describe (per related-session messaging digests). |
| **Impact** | (a) Manager attempts to message a newly-spawned worker within the spawn-window to set context / stand-down a placeholder fail silently. (b) Combined with §1.1 (closeout-leaf-placeholder), this is the failure path: the manager wants to send a stand-down message immediately after the leaf is created, but the worker isn't yet addressable, so they read the title and start authoring before any context arrives. (c) The §1.1 mitigation depends on §1.5 being fixed. |
| **Recommended fix** | (a) Block `dashboard-ops work-items create` from returning until the spawned worker is registered in the messaging table — so the parent's first message-after-create is guaranteed to address. (b) Or: queue messages to a not-yet-registered session id and flush on registration — so the parent's `dashboard-message send` succeeds eagerly. Either approach closes the spawn-vs-message race. |

---

## §2. Cross-pattern observations

- **§1.1 and §1.5 compound.** The closeout-leaf-placeholder mitigation (stand-down message within ~30s of leaf creation) depends on `spawned-worker-not-in-messaging-table` being closed; otherwise the stand-down send fails silently and the worker reads the title unopposed. Either fix on its own halves the failure surface; both fixed closes the class.
- **§1.2 and §1.3 are independent surface issues.** Title-truncation affects re-spawn / successor brief assembly; auto-open-on-undeleted-branch is a duplicate-PR shape. No coupling.
- **§1.4 is alerting/UX, not control-plane integrity.** Held messages are not lost. Fix improves caller ergonomics; absence of fix does not create rogue behavior — only sender-side anxiety.

## §3. Recommended priority ordering

Suggested dashboard-side priority (highest-leverage first):

1. **§1.5 spawn-vs-message race** — unblocks the §1.1 mitigation and removes the silent-fail edge.
2. **§1.1 closeout-leaf-placeholder structured flag** — eliminates the rogue-PR class entirely; biggest blast-radius reduction.
3. **§1.2 auto-open-on-undeleted-branch** — small ergonomic win, easy fix (suppress on existing PR / branch-delete-on-merge).
4. **§1.3 title-truncation** — affects re-spawn correctness; fix is storage-layer.
5. **§1.4 nudge-flips-held** — alerting/UX hygiene; lowest blast radius.

This ordering is a recommendation, not a directive — dashboard ops owns the actual sequencing.

---

## §4. What this audit is NOT

- **Not a compiler-path artifact.** Per PR #3983 §7 MW-D6 the audit is explicitly *NOT* compiler critical path; nothing here touches substrate, `.dag` lowering, emit, infer, parse, or any Rust seed file.
- **Not a fix PR.** Recommended fixes name shape, not patches. Dashboard ops authors the implementations.
- **Not exhaustive.** Five named patterns; other dashboard control-plane bugs may exist and are out of scope for this bounded audit.

## §5. Related artifacts

- PR #3983 (`docs/planning/v4-merge-wave-and-next-waves-2026-05-30.md`) — MW-D6 dispatch authority for this audit.
- `feedback_closeout_leaf_placeholder.md` in this lane's session memory — §1.1 workaround playbook in operational form.
- `docs/planning/v4-close-receipt-manager-pass-2026-05-30.md` — Close/Receipt lane vocabulary + close grades; the lane that authored this audit.
