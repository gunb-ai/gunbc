# Disputed extdeps rows: a two-axis classification template, tested on the adjudicable subset

Operator ruling (2026-10-05): placement batches pause because about half of
placements were rejected on review — the classification rule is not yet ready to
be used as a template. This document records **two judgments per case**, because
the disputes of the last several batches were two different questions being
answered with one label:

- **Semantic standing** — who owns the semantic information the declaration
  carries: *exact upstream fact* (a versioned upstream artifact states the
  surface and the declaration transcribes it field-for-field), *shared formal
  semantics* (recorded somewhere nameable but not one product's artifact),
  *product policy* (the product's own choice: dispatch algebras, serializer
  policies, normalizing tables, partial grammars, transports), or *unsupported*
  (nothing states it and no product purpose holds it up).
- **Consumption standing** — per DESIGN 3c's three honest states: *executing*
  (consumed by execution in this tree), *declared frontier with trigger* (a
  named consumer lands in a named later change, trigger stated beside it), or
  *dangling* (nothing consumes it — red regardless of how well modeled it is).

The **disposition follows from both axes together**: scope (fact + executing),
re-home or re-label (policy + executing), delete or name-a-consumer (fact or
unsupported + dangling), repair (a mixed module whose subjects must be separated
before either axis can be judged honestly). No single axis decides.

## The evidence must fit the surface

A standing is only as good as its evidence, and the evidence has to be of the
kind the surface admits:

- **Data shapes are judged by field/constructor diff** against the cited
  artifact. If the declaration adds, drops, reshapes or renames anything the
  artifact does not state, the standing is not exact-upstream-fact — the diff
  is how the judgment is made, not a formality after it.
- **Decision procedures are judged by behavioural conformance** to the
  artifact's stated rule. A precedence rule, an ordering, a matching policy: the
  question is whether the implementation orders/matches the way the artifact
  says on its cases — not whether its fields resemble the artifact's. A field
  diff cannot establish conformance, and a passing field diff cannot excuse a
  failing behaviour.
- **Consumption is established only by an executing route** — a fold, an entry
  point, a realization, an emitted artifact that reads the declaration. A grep
  hit names a candidate site; it cannot establish that the site executes. A
  name-keyed pattern establishes nothing: a predicate that reads no module
  content is a classification, not a consumer. A mirror's definition of a
  declaration is not a consumer of it. A witness that exercises a call inside a
  test is not a production route.
- **Both consumption labels are positive claims at this grade.** `executing`
  asserts a route exists; `dangling` asserts no route exists across the decoded
  surfaces. Grep-grade evidence supports neither — it supports **unadjudicated:
  grep-grade only**, and that is what this document records wherever its own
  evidence is grep-grade. The one exception in the sample is `semver_scheme`,
  whose `dangling` label below is a recorded ruling of the side chat (review
  5424597802), kept with its evidence grade attached and marked as a ruling,
  not an adjudication.

**Misfile shapes (the recurring review failure).** A citation can be live and
real and still not own the declaration. Observed, each a real pulled row: a
tool's CLI citing the standard the tool implements; a repo-normalized table
citing the spec it normalizes; a dispatch algebra citing one tool it drives; a
repo extension inside an otherwise transcribed enumeration silently breaking the
predicate; a repo plan citing fragments of a CLI grammar it does not fully
reproduce.

## The recorded cases

Seven cases with both axes recorded. Semantic standings were decided against
the cited artifacts (the instruments those surfaces admit); every consumption
standing below is grep-grade, so each carries its evidence kind in place —
the four `unadjudicated` labels are this document's own grade, and the three
`dangling` labels are recorded rulings of the side chat (reviews 5423944315 and
5424597802), kept with their grade attached. Dispositions that would change
with the consumption walk are marked provisional and name what would settle
them.

**1. `docker/cli.dag` — `DockerCreateSpec` and its flag types.**
Semantic standing: **product policy.** The type is a repo-normalized plan: its
flag grammars are partial (Docker's CLI reference owns `--restart` fully —
`on-failure:N` cannot be expressed by `DockerRestartPolicy` as written, cli.dag
lines 32-39 — and `DockerNamespaceMode` covers only host/private, line 43), so
it borrows CLI grammar fragments rather than transcribing the cited surface.
Consumption standing: **unadjudicated: grep-grade only** — candidate consumer
sites (grep hits): the `dag/gunbc/spark/*` serving modules and
`docker/container_inspect.dag` name `DockerCreateSpec`; no execution-grade walk
has run.
Disposition: **repair** (consumption-independent). Either complete the grammars
field-for-field against Docker's CLI reference — after which the (a) predicate
is judged again on the diff — or re-label the plan as repo vocabulary in the
product layer. What the walk decides is the module's longer-term standing
(kept-and-consumed vs delete-or-declare); that waits.

**2. `bmc/openbmc_operation.dag` — `OpenBmcOperation`.**
Semantic standing: **product policy.** Owner: this repo — the dispatch algebra
over file ops, systemctl, busctl, jq, mkdir/rm/cp, sleep is our vocabulary of
what a BMC operation is; the busctl man page owns only busctl's argv semantics.
Consumption standing: **unadjudicated: grep-grade only** — candidate consumer
sites (grep hits): `openbmc_fan_control` and `openbmc_password_ssh_transport`
name it.
Disposition: provisional **re-home** to the product layer. The algebra is
product vocabulary regardless of the walk — that part is consumption-
independent; what the walk decides is whether the row leaves as re-homed
(kept-and-consumed) or as deleted. The internal consumers, if the walk confirms
them, keep depending on it — they should, it is the product's own vocabulary.

**3. `languages/json/grammar.dag` — `json_production_rows`.**
Semantic standing: **product policy.** Owner: this repo — the columns
(`rfc_section`, `lhs`, `rhs`, `value_variant`) are a repo-normalized grammar
whose `lhs` names a local `literal` nonterminal and whose `value_variant` maps
to repo AST names; the raw RFC 8259 production strings embedded in the rows are
upstream fragments, not the surface the table models.
Consumption standing: **dangling, as ruled** (side chat, review 5423944315;
grep-grade: no production consumer found; the parse path is `json/parse.dag`,
scoped separately).
Disposition: **delete or re-home** — the table is a documentation artifact in
code; both dispositions are live and the call belongs to the operator.

**4. `languages/markdown.dag` — `commonmark_gfm_spellings`.**
Semantic standing: **product policy.** Owner: this repo's serializer policy —
choosing which dialect's spellings to emit over CommonMark 0.31.2 and GFM
extensions (tables, task lists) is a product decision; no single upstream owns
"both dialects at once."
Consumption standing: **unadjudicated: grep-grade only** — candidate consumer
sites (grep hits): `truth_table_projection` and the markdown tests name the
spellings.
Disposition: **re-label** as product policy (consumption-independent); the row
stops requesting a citation that cannot exist. What the walk decides is kept-
vs-deleted; the serializer behavior itself is untouched.

**5. `tools/curl.dag` — `curl_cli_tool` and its policy rows.**
Semantic standing: **mixed — repair required.** The exit-code table (6/7/28/35/
52/56) is an exact upstream fact owned by curl's own man page, which the anchor
cites correctly. The pinned minimum version (>= 7.68), the localhost timeout
rows, and the apt install realization are product policy — choices, not curl
facts.
Consumption standing: **unadjudicated: grep-grade only** — candidate consumer
sites (grep hits) name the tool rows.
Disposition: **repair — split** the policy rows out from under the
external-authority anchor (consumption-independent); the CLI-fact side is then
re-judged field-for-field against the man page on its own diff. Kept-vs-deleted
waits on the walk.

**6. `extdeps/ebay/mock_corpus.dag` — dangling repo data; the deletion-safety
check.**
Semantic standing: **unsupported.** The module is repo-authored
`PublishedMockCase` hermetic-replay data; no upstream states it.
Consumption standing: **dangling, as ruled** (side chat, review 5424597802).
The census (2026-08-22, re-derived 08-26)
classifies it STILL-UNCONSUMED, and the v1 mirror's
`ends_with(".mock_corpus")` predicate (in
`external_authority_is_clean_tree_roster_excluded_for_module_path`,
`src/v1/stage0/src/cli_run/external_authority.rs` line 211) does not change
that: the predicate reads no module content, so it is a classification, not a
consumer.
Disposition: **delete vs declared future consumer** — operator's, between
deletion and re-homing to a fixture namespace with a named trigger. The
convention scan survives only as a **deletion-safety check** (before deleting,
confirm no compiled logic keys on the module's name in a way that reads its
content), not as consumption evidence — and it is the check the census's
mention scan does not collect.

**7. `dag/extdeps/docker/container_stats.dag` — dangling and not
field-for-field.**
Semantic standing: **product policy over upstream fragments.** It adds a
repo-authored `cpu_percent` field and reshapes `networks`, so it is not the
Engine API's stats payload; owner of the real surface: Docker's Engine API
stats reference.
Consumption standing: **dangling, as ruled** (side chat, review 5423944315;
grep-grade: no production caller; the #13304 review confirmed it
independently).
Disposition: **delete or name a consumer** (DESIGN 3c). Repairing the
transcription — dropping `cpu_percent`, restoring the API shapes, re-diffing —
is worthwhile only once a consumer or a declared frontier with a trigger
exists; on today's tree the honest move is deletion or a named trigger.

## Set aside — not adjudicated at execution grade

Five cases from the sample could not be adjudicated on both axes now. They are
recorded so the work is not lost, with their consumption standings marked
**unadjudicated: grep-grade only** — grep hits name candidate sites and cannot
establish that the site executes (the one recorded ruling in this section,
`semver_scheme`'s dangling, is attributed above).

- **`cloud/gcp/errors.dag` — `GcpRpcCode`**: semantic standing **non-exact**.
  The module adds `GcpRpcCodeOther { raw: NonEmptyStr }` (errors.dag line 32)
  beside the 17 transcribed `google.rpc.Code` values, so there is no
  exact-upstream subject until the enumeration is split from the repo variant
  and the enumeration alone is diffed against
  `cloud.google.com/apis/design/errors#http_mapping`. Consumption:
  unadjudicated (grep-grade: only its grounding witness test references the
  module).
- **`crypto/hash.dag` — `HashAlgorithm`/`Digest`**: **FIPS-derived fragments
  plus local carriers.** `Sha256`/`Sha512` is not FIPS 180-4's family naming —
  the carriers are repo-local and sit beside `sha256sum(1)` machinery (coreutils
  manual owns the tool). There is no exact-upstream subject until the FIPS
  facts are split out and diffed against FIPS 180-4, and the verifier machinery
  is judged against the coreutils manual. Consumption: unadjudicated
  (grep-grade: the sccache pin and live-deploy paths name the verifier).
- **`ssh/session.dag` — `service ssh.Session`**: **mixed subjects.** The module
  quotes ssh(1)'s exit-255 contract (an exact upstream fact owned by the
  OpenSSH client manual — not RFC 4254, which owns channel messages) but also
  carries repo transports, session records and mocks. Until the subjects are
  separated (one module per authority), neither side's semantic standing can be
  judged. Consumption: unadjudicated (grep-grade: repo consumers name the
  machinery subjects).
- **`version/semver.dag` — `semver_scheme`**: an **upstream-owned decision
  procedure** (semver.org section 11 precedence) whose fitting evidence is
  **behavioural conformance** — does `compare` order identifiers the way §11
  says — not a field diff. That conformance has not been established; the
  dispatch was lexical (`semver_identity_compare`), which is the recurring
  failure mode filed as
  `a_citation_attests_a_decision_rule_the_code_does_not_implement` (#13235).
  Consumption standing: **dangling, as ruled** — the side chat's ruling
  (review 5424597802): only the witness test calls `semver_scheme.compare`
  (`extdeps_version_semver_witness_test.dag`), and the v1 stage0 mirror defines
  `semver_scheme()` — a definition is not a consumer. Recorded with its
  evidence grade attached: under this document's own rule the witness-only
  grep supports `unadjudicated`, so the ruled label is kept as a ruling, and
  any delete-or-scope decision on this module should not fire until an
  execution-grade walk confirms it. Whether the type family passes a field
  diff against semver.org's grammar is likewise unverified and not asserted
  here.
- **The `github/*` family**: **dropped from this document.** The per-row rule
  (each surface judged against its specific REST page, field-for-field; each
  row's consumption its own) is carried as a rule, but no row of the 19 was
  adjudicated here — none has had a field-for-field diff or an execution-grade
  consumer walk run for it in this work.

## What this template changes for bulk work

1. **Two judgments, two instruments — and the instrument must fit the
   surface.** Field/constructor diffs for data shapes; behavioural conformance
   for decision procedures; an executing route for consumption. An anchor alone
   decides nothing: an anchor is exactly the thing product-policy rows can
   borrow, and a grep hit, a mirror definition or a name-keyed pattern is
   exactly what a dangling row can borrow.
2. **Dispositions are derivable, not argued** — from admissible evidence only:
   fact + executing → scope; fact + dangling → delete or declare a trigger;
   policy + executing → re-home or re-label; unsupported + dangling → delete
   (safety-checked); mixed → repair by splitting subjects, then re-judge each
   side. Where the evidence grade is below the axis, the axis is recorded
   unadjudicated — and a disposition that would depend on it is recorded as
   provisional, naming what would settle it (the execution-grade walk or the
   fitting diff), rather than fired on the lower-grade evidence.
3. **Shared formal semantics stays a discipline.** Among the cases recorded
   here, none finally landed there. That is a statement about this sample, not
   about the frontier. The procedure stands: when a row looks shared, hunt for
   the recording source; if none exists, the row is product policy or
   unsupported, never settled as shared by default.
4. **Unconsumed is a delete-or-declare decision, not a scope** (DESIGN 3c) —
   and "unconsumed" itself is only decided at the evidence grade above: a row
   whose consumption is unadjudicated is not thereby dangling, but it is not
   scopeable either; the execution-grade walk has to run first.

## Verification status of this document

Every consumption claim above is **grep-grade**: no executing route has been
established at identity grade for any case in this document, so the four
`unadjudicated` labels are the grade's honest reading and the three `dangling`
labels are recorded rulings of the side chat with that grade attached — none
of them is presented as execution-established. The semantic standings of the
recorded cases were decided against the cited artifacts (the docker/cli line
numbers are quoted from the module); standings in the set-aside section are
recorded with their missing evidence named. The census numbers are its own (2026-08-22 / re-derived 2026-08-26) and
carry that clock. No code, no tsv, and no roster is changed by this PR; it is
a classification template, and bulk work waits on the operator's ruling on it.
