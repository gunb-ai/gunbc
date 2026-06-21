# Plan — Access / Ownership / Visibility model in `.dag`

Working doc for branch `worktree-access-model`. North-star reference so the original goal
stays anchored while we build bottom-up. Not necessarily for landing in gunbc (cf. the
"no internal planning markdown in gunbc" norm) — keep it as a worktree scratch unless we
decide otherwise.

> **METHOD CORRECTION (2026-06-20).** The early `src/v2/std/access.dag` (Steps 1–2, PR #5414)
> silently re-coined Zanzibar as a std abstraction — the §3 nickname violation. Corrected
> approach: model several real access-control systems faithfully in `dsl/extdeps/access/`
> (cited, real names — as `extdeps/cloud/gcp/iam` + `extdeps/github` already do), THEN derive the
> std abstraction from their commonality and validate it against each ("see if anything breaks").
> So Steps 1–2's std/access.dag is **superseded**; the Step-3 lens (independent) survives.
>
> **Rework status (done & verified):**
> - `dsl/extdeps/access/{zanzibar,aws_iam,posix,rbac}.dag` — four systems modeled faithfully
>   (cited, real names); compile 0-diagnostics + discriminating red. (Plus existing `cloud/gcp/iam`,
>   `github` as free data points.)
> - `dsl/std/access.dag` — the DERIVED abstraction: `AccessRequest<S,A,O>`, `AccessDecision =
>   Permit|Deny` (combined Deny-wins), `AccessPolicy<S,A,O>` (the §4 Realization interface).
> - `dsl/std/access_validation_test.dag` — POSIX (DAC) + RBAC both fully inhabit it; **7 decision
>   witnesses green by execution**; broken POSIX class-resolution → RED.
> - **Findings ("what breaks"):** POSIX's object-attached ACL inhabits `AccessPolicy` directly;
>   RBAC's global policy must be curried in (a closure) — a real strain; Zanzibar fuses action+grouping.
> - **Context — RESOLVED (no opaque `C`).** AWS IAM ABAC "context" *dissolves*: condition keys are
>   attribute-carrying S/A/O (`s3:prefix` → action param) + grounded coordinates (`aws:SourceIp`
>   internal-CIDR → `NetworkLocality`; `aws:CurrentTime` → `Timestamp`). IAM inhabits the SAME
>   `AccessPolicy<S,A,O>` — std core UNCHANGED. Proven in `test.claim.access_iam_validation`
>   (4 witnesses green by execution; broken `SourceIp→locality` decomposition → red).
> - Deleted superseded `src/v2/std/{access,access_test}.dag`.
> - **Next:** validate Zanzibar into std.access; rework the Step-3 lens to consume `std.access`;
>   ownership invariants.

---

## 0. The original goal (don't lose this)

Abandon the old `ctrl/` **session dashboard** (a ~1M-line JS control plane for orchestrating
fleets of AI coding agents) and rebuild what's worth keeping in `.dag`, applying ctrl's
hard-won principles but **not** its code. Strategic aim: **dismantle ctrl step by step so we
can get on gunbc full-time** — consolidate the fleet/control-plane into gunbc (which **stays
public**); secret *values* never committed (handled by `SecretRef` → GCP Secret Manager);
one `operator_fleet.dag` authority everything derives from.

**This work-stream is the first dismantling step:** model *code visibility / privacy*
properly, so private parts of the (public) gunbc tree are kept private by a real fact rather
than by which repo a file happens to sit in. Once this exists, ctrl's private logic can move
into gunbc behind real labels.

## 1. The reframe (what we're actually separating)

Today the **repo split is a nickname** for privacy: the *only* privacy lever is git topology
— code is private iff it lives in the private `gunb-ai/ctrl` repo (submodule-pinned into the
public `gunb-ai/gunbc`; no scrub/mirror). That fuses concepts that must live apart (DESIGN §3).
The informal version already exists as code comments — `operator_fleet_network.dag:8`
("secrets remain in ctrl"), `ci_fleet.dag:4` ("operator inventory remains private (ctrl)").
Those comments **are** the labels we're formalizing.

Separate into four independent facts:

| Concept | What it is | Status |
|---|---|---|
| **Ownership** | *who owns* a node — the root of authority (only the owner may grant). | **build first** |
| **Visibility / access** | *who may read / write* a node. ReBAC relations. | build second |
| **Repo** | *where* code is stored + the storage's own access level (`Repository.private`). | exists: `extdeps/github` — reference, never re-coin |
| **Publication** | §4 Realization: (labeled tree + audience) → which units → which repo/CI. The 2-repo split is *one handler*. | build last |

Composes with `SecretRef` (orthogonal axis): **SecretRef keeps secret *values* out of git;
Visibility keeps private *code/data* out of the public projection.**

## 2. Anchor & decisions (settled)

- **Anchor: Google Zanzibar / ReBAC.** A relation tuple `⟨object, relation, subject⟩` is
  *literally a substrate `Edge`* (relation = label, subject = target node). Subjects are
  Zanzibar usersets: `Everyone` (public / "globally readable"), a specific node (`Is`), or a
  userset (`MembersOf{node, via}` = members of an org, readers of a node). Standard, and
  graph-native — the access graph is a **subgraph of the node graph**, not a side-store.
  (Alternative weighed & rejected: object-capability — doesn't express "this org can read this".)
- **Grounded on real `.dag` nodes.** Principals (users, orgs, agents) are **real `.dag`
  nodes**; referenced by the existing node identity — `Symbol` literal `^name`, or
  `QualifiedName` (`QnCons { head: ^seg, tail: … QnEmpty }`). **No minted `EntityId` /
  string id** (the §3 anti-nickname correction).
- **Placement: the v2 tree (`src/v2/…`).** The model references and walks real nodes, and
  `dsl/`-tree files **cannot import `v2.std.node`** (`dsl/std/content_hash.dag:4`). Precedent:
  `src/v2/lens/extdeps_shape_transport_policy.dag` is a lens that already keys facts on real
  `QualifiedName`s; `src/v2/lens/extdeps_shape_transport_policy/module_refs.dag` authors
  `QualifiedName` literals. v2 already hosts policy (`src/v2/workflow/ci_floor_plan.dag`).
- **Fail-closed (§5).** Unowned / unlabeled ⇒ private to the operator, **never public**.
  Matches "I don't want every piece of code visible to the public."
- **Verification standard (§5).** "Done" = compile **green by execution** + a
  **discriminating input that goes red** + an executed `test fn`. A `*_test.dag` with a
  `test fn` auto-enrolls in the CI floor by naming convention.

## 3. Substrate facts we build on (grounding receipts)

- `src/v2/std/node.dag` — `type Symbol`, `type Node { kind, children: List<Edge>, occurrence_id }`,
  `type Edge { label: EdgeLabel, target: Node }`, `EdgeLabel = Named{name: Symbol} | Positional`,
  `fold_node` (the catamorphism every stage reuses — DESIGN §6).
- `src/v2/std/qualified_name.dag` — `QualifiedName = QnEmpty | QnCons { head: Symbol, tail }`;
  `qualified_name_eq`, `qualified_name_singleton`. (Gated free-monoid interim — fine to use.)
- Symbol literal syntax = `^name` (e.g. `^operator`, `^cargo_build`).
- `extdeps/github/github.dag:35` — `Repository { owner, name, full_name, private: Bool, default_branch }`
  (the repo's own access level — distinct from a node's visibility).
- `std/algebra.dag:310` — `BoundedLattice<T>` (kept for the *effective-visibility leak rule*,
  not for visibility levels themselves).

## 4. Step plan (bottom-up; each step verified before the next)

- [x] **Step 0 — research + reframe.** Map old dashboard, extract principles, pick anchor,
      ground placement. Retire the `dsl/gunbc/visibility.dag` lattice sketch (wrong primitive
      + wrong tree).
- [x] **Step 1 — Ownership.** `v2.std.access` framework: `Relation = Owner|Reader|Writer|Member`,
      `Subject = Everyone | Is{node} | MembersOf{node,via}`, `Grant { object, relation, subject }`.
      `operator` + `gunb_ai_org` declared as **real nodes** (`access_test.dag`). `OwnerResolution =
      Owned|Unowned`; `owner_of` / `effective_owner` (unowned ⇒ operator, fail-closed) / `is_owner`.
      **Receipt:** `src/v2/std/{access,access_test}.dag` — typecheck 0-diagnostics + discriminating
      red; 4/4 `test fn`s green by execution (`gunbc run --claim-run`), false-claim exits 1. STILL
      OPEN here: the "exactly one owner" + inheritance + DAC invariants (enforced by the Step-3 lens, not yet).
- [x] **Step 2 — Visibility.** `can_read` / `can_write` (relation hierarchy Owner⊒Writer⊒Reader),
      `subject_admits` resolving `Everyone`(public) / `Is` / `MembersOf` (one-level org membership),
      `is_public`, and the **leak rule** `effective_public` (public AND every exposed dep public —
      a public node importing a private one cannot be published; fail-closed meet-fold).
      **Receipt:** 17/17 `test fn`s green by execution; compile 0-diagnostics; targeted
      discriminating check — breaking `effective_public` to ignore deps turns the leak test RED.
      STILL OPEN here: recursive userset resolution (members-of-members) is one-level only; the
      `effective_public` dep flags are passed in, not yet walked from the real graph (Step 3).
- [~] **Step 3 — Enforcement lens (PURE CORE DONE; live wiring deferred).** `src/v2/lens/visibility.dag`
      — `ModuleVisibilityFact { module: QualifiedName, public, imports }`, `VisibilityLeak`,
      `scan_visibility_leaks` (a public module importing a non-public one = a located leak; fail-closed:
      an unknown import is non-public ⇒ a leak). The import-graph realization of access.dag's leak rule.
      **Receipt:** 4/4 lens-unit witnesses green by execution (leak caught, clean passes, fail-closed
      unknown import, private-importer exempt); compile 0-diagnostics; breaking the scanner turns the
      leak witness RED. **DEFERRED (decision point):** the LIVE corpus witness ("0 leaks in the real
      tree") needs live `ModuleVisibilityFact`s — module QN + `public` (access.is_public over grants)
      + `imports` (`v2.std.dependency.dependency_lens`, kind `ModuleDependsOn`). That bridge is a Rust
      seed / corpus-as-node handle (cf. extdeps lens). Also still un-enforced: the ownership invariants
      (one-owner / inheritance / DAC) — additional scanning functions, not yet written.
- [ ] **Step 4 — Publication / git-CI realization.** Project the labeled tree per audience →
      destination repo + CI config; dissolve the manual repo split into derived data. Decide
      canonical-vs-mirror direction (see open threads).
- [ ] **Later — consolidation.** Fold into `operator_fleet.dag` authority; migrate ctrl's
      "remains private (ctrl)" comments into real labels as ctrl is dismantled.

## 5. Open threads / decisions pending

- **Owner cardinality / grant authority** — default: one owner per node; only owner grants
  (DAC). Alt: co-owners; writers may re-grant.
- **Inheritance** — default: a node inherits its container's owner unless it overrides locally.
  Alt: every node explicit.
- **Reference granularity** — `Symbol` (coarse, simple, what Step 1 uses) vs `QualifiedName`
  (precise, addresses deep nodes). Referential integrity of `^name` is unchecked by types — a
  lens checks it (Step 3).
- **Publication direction** (gunbc stays public) — (A) public gunbc canonical + private
  sibling for non-public units, vs (B) private canonical + derived public mirror. Decide at Step 4.
- **Where this lands** — model in v2 tree (yes). This plan doc + whether the policy facts ship
  in public gunbc — TBD.
