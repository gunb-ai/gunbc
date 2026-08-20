# Parked questions and small standing notes


---

*Provenance: this lane was carried as an Open Threads row in `DESIGN.md` until 2026-08-18, when that section was deleted on operator ruling — `DESIGN.md` is a first-principles document and per-lane status in it was a dual representation. The text below is preserved verbatim from that row; its authority is the carriers it names, not this file.*

Items too small to warrant their own design document, kept so they are not lost. Each is either an operator-parked question or a standing directive whose authority is the carrier it names.

- can a lens mechanically diagnose the *leaf-side* of decomposition (§2)? (operator-parked)

- the remaining deleted-`docs/` references in `.dag` comments — provenance / `bind:` pointers into the bankrupted `docs/` tree (e.g. `docs/planning/*`, `design-*.md`) — fold into the dep-graph reform, not a blind repoint. (The named-corpus ledger marks — `Practice N`, and `INVARIANTS` / `THESIS` / `MODELING` / `RELEASE_TODO` / … citations — were swept: dropped, or re-homed to DESIGN.md §-anchors.)

- **one authorization kernel, several typed request profiles (operator directive 2026-08-01).** `std.access.AccessRequest` carries subject, action, object, typed context, and typed evidence; `AccessPolicy` is the sole policy-to-decision seam, `decision_meet` the sole conjunction, and every denial carries a typed cause at structured request coordinates. Effect, reference, publication admission, disclosure, and sealed execution are exact request products projected into that kernel, never independent allow/deny folds. Principals project external evidence — OIDC issuer/subject and observed POSIX effective principal — rather than being invented internal identities. The provisional gunbc#7541 `Verb::Reference` / `Verb::Publish` / `AudienceScopeTree` carriers dissolved: audiences are sets with join and partial-order subset, because real audiences can overlap without either containing the other. `PublicationAdmissionRequest` is only the generic request shape here: the operator-deleted Stage-0 placement wall, publisher policy, and pathname census are deliberately absent.
