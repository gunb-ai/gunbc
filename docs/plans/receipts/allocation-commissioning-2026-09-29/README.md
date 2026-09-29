# Allocation commissioning checkpoint, 2026-09-29

Read-only observation from allocation source `7b21531d57622c8464e9ce9808ac31081f9a4f98`.
The attached SSH census used strict host-key checking, batch authentication, and
root metadata reads only. Exit 1 is the explicit missing-path result, not a
successful commissioning claim. No service or file was changed on srv1.

The candidate `srv1-13` VM service remains failed. Its cell slice reports inactive,
no fragment, and unlimited memory. The protected-state key file, allocation store,
and readiness store are absent. None of these observations establishes that the
slot is free, sanitized, or admitted for workspace use.

PR #12505 remains open/draft, with no status check rollup returned at this check.
The parent #12465 remains open/draft at `ada7daefc28696ce4ba242437cd26daf61bd0bab`.
The socket dependency #12482 is open at `1e74401a16caf0ec57c7eaeb35e4d9d966bbe6a9`;
this allocation tree contains its earlier integration, not a qualification of
that later sibling head. Technical HOLD and integration checks remain applicable.

The cloud material/accessor bootstrap succeeded previously. PR #12565 separately
installed/read back the deny protection and retired its temporary human grant.
The ordinary approval workflow's initial iam-observe federation still depends on
unfinished bootstrap trust. Approval is not itself an administrator credential.
The operator has now confirmed an exclusive pool-policy maintenance window; the
one-off initialization change and its qualification are tracked in #12565.

The next live obligations remain protected-state credential custody and service
commissioning, exact budgeted slot transfer with recovery/sanitation, pinned
runtime installation, production preparation and selected-host routing, recurring
consumer/access observations, and release/expiry/restart/same-slot-reuse acceptance.
There is no successful VM allocation receipt yet.

Follow-up: ordinary GCP IAM workflow run 36505895733 attempt 2 passed the app
approval and apply/readback path. Credential custody is isolated in draft #12580
on main; it remains uninstalled. The allocation transport now reads the shared
host-path authority used by that cut. All four state authentication controls pass
using the existing allocation branch binary over 68 byte-identical modules under
6 GiB/no swap. Follow-up host metadata still does not establish free capacity.
