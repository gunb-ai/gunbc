# Disputed extdeps rows: a classification rule tested on real cases

Operator ruling (2026-10-05): placement batches pause because about half of
placements were rejected on review — the classification rule is not yet ready to
be used as a template. This document builds the rule the way the census built
its population: by classifying a representative, disputable sample at the
identity grain, stating for every row who owns the semantic information, and
saying what each disposition would and would not fix.

The four classes:

- **(a) upstream-owned fact** — a versioned upstream artifact (spec, REST
  reference, man page) states the exact surface the declaration transcribes.
  The declaration is a faithful, field-for-field model of that surface.
- **(b) shared semantics independent of any product** — semantics with no
  product behind them that more than one consumer relies on.
- **(c) product policy** — the product's own choice: dispatch algebras,
  serializer policies, pinned versions, transport realizations, repo-normalized
  tables. No upstream can own these; an upstream can only be *invoked* by them.
- **(d) unsupported/unconsumed** — no upstream attests the surface and no
  consumer uses the declaration. Deletion candidate, after identity-grade
  verification that nothing consumes it.

## The rule (decision procedure)

1. **Name the semantic information.** What does the disputed declaration
   actually state? Not what its citation names — what it states.
2. **Ask who can state it authoritatively.** If a versioned upstream artifact
   states it, and the declaration is field-for-field that artifact's surface,
   the row is (a). If the statement is a choice the product makes (which
   operations exist, which dialect's spellings to emit, which version to pin,
   which transport to use), it is (c) regardless of how real the citation is.
   If nothing states it and nothing consumes it, it is (d).
3. **(b) is a discipline, not a refuge.** Class (b) only applies when the
   semantics are recorded somewhere nameable — if no source can be cited
   because no source exists, the row is (c) or (d), not (b). In the twelve-row
   sample below, every row that first looked like (b) turned out to be (a)
   (the semantics have an RFC or vendor owner) once the owner question was
   asked seriously. That is a sample observation, not a prediction about the
   frontier: twelve hand-picked disputed rows cannot establish that (b) is
   rare in general. What the sample does establish is the procedure: when a
   row looks like (b), the next step is to hunt for the recording source, and
   if none exists the row is reclassified — never settled as (b) by default.
4. **Misfile test (the recurring review failure).** A citation can be live and
   real and still not own the declaration. The subject has to BE the cited
   surface, field-for-field. The four observed misfile shapes, each a real
   pulled row: a tool's CLI citing the standard the tool implements (ssh
   client, coreutils sha256sum); a repo-normalized table citing the spec it
   normalizes (json grammar rows); a dispatch algebra citing one tool it
   drives (OpenBmcOperation citing busctl); a repo policy row citing the
   nearest spec page (semver scheme citing precedence its own dispatch
   contradicts).

## The cases

Twelve rows, chosen to span the dispute shapes the review history actually
produced: hub/shape vocabulary, self-normalized tables, tool-vs-standard
mixing, unconsumed islands, and a multi-module upstream family.

### (a) upstream-owned facts — scopeable, some after repair

**1. `cloud/gcp/errors.dag` — `GcpRpcCode` (17 `google.rpc.Code` values).**
Category (a). Owner: Google's API design docs, the `google.rpc.Code` table
(current anchor `cloud.google.com/apis/design/errors#http_mapping`). The type
is a clean transcription of that enumeration; nothing is added and nothing
reshaped. No production consumer — grep-grade only: the sole reference outside
the module is its own grounding witness test. **What scoping would NOT fix,
and what DESIGN 3c forbids:** under 3c a declaration nothing consumes is red
however well modelled, so this row is a delete-or-name-a-consumer decision,
not a scope. The transcription's faithfulness only matters if a consumer
appears or the row is re-homed; the census is the instrument that tracks the
island, not the placement gate.

**2. `docker/cli.dag` — `DockerCreateSpec` and its flag types.**
Category (a). Owner: Docker's CLI reference, the `docker container create`
page (the module's anchor). `DockerRestartPolicy`, `DockerNamespaceMode`,
`DockerDeviceMapping`, `DockerEnvAssignment` are literal CLI flag grammars —
the cited page's own surface, and the page is the tool's own reference, not a
standard the tool implements. This is the counter-example to the
tool-CLI-citing-standard misfile: a tool surface is (a) when the citation is
the tool's authority. Production consumers exist (grep-grade: `DockerCreateSpec`
appears in `dag/gunbc/spark/*` serving modules and `docker/container_inspect.dag`).
**What scoping would fix:** retires the row and grounds the spark consumers'
vocabulary in a live page. **What it would not fix:** nothing known — but the
field-for-field diff against the create page must still be run before scoping;
that is the standing lesson.

**3. The `github/*` family (19 frontier rows, one upstream).**
Category (a) as a family with a per-row condition. Owner: docs.github.com's
REST reference — one upstream, many surfaces. The family rule from batch 5:
two modules of one upstream is fine; the condition is that each module's
anchor cites the specific REST page for ITS surface (the `Gist` module scoped
in #13258 passes this way; `GitHubErrorShape` failed it when its locator
redirected to generic getting-started material). **The per-row rule, not a
batch template:** placement batches are paused, and nothing here proposes
restarting one; the family rule is that each of the 19 rows is decided
separately — same upstream, but each row still needs its own live, specific
page and its own field-for-field diff before any scope. A family only means
you do not have to re-litigate who the upstream is per row.

**4. `crypto/hash.dag` — the algorithm side (`HashAlgorithm`, `Digest`,
SHA-256/512 per FIPS 180-4).**
Category (a) — half of a mixed module. Owner: NIST FIPS 180-4 (the anchor).
The algorithm types are the standard's own surface. The other half is (c): the
module's shell-verifier note binds `sha256sum(1)` with two production
consumers (grep-grade: the sccache pin and live-deploy verification paths name
the verifier) — a tool realization the
standard does not state, which is why the #13189 review pulled the `Sha256Sum`
scope. **What a split would fix:** the algorithm side scopes cleanly against
FIPS; the shell-verify side stops borrowing the standard's authority.
**What it would not fix:** the shell verifier's consumers keep working either
way — the split is honest bookkeeping, not behavior.

### (c) product policy — the misfiled majority

**5. `bmc/openbmc_operation.dag` — `OpenBmcOperation`.**
Category (c). Owner: this repo — the dispatch algebra over file ops, systemctl,
busctl, jq, mkdir/rm/cp, sleep is OUR vocabulary of what a BMC operation is.
The busctl man page (the anchor) owns only busctl's argv semantics, and the
#13110 review pulled the row for exactly this. Consumed internally
(grep-grade: `openbmc_fan_control` and `openbmc_password_ssh_transport` name
it) — real internal
consumers, so deletion is wrong. **What re-homing the algebra to the product
layer would fix:** the frontier row leaves because the module stops claiming to
be extdeps at all. **What it would not fix:** the internal consumers keep
depending on it — they should, it is the product's own vocabulary.

**6. `ssh/session.dag` — `service ssh.Session`, `ssh_client_error_exit_status`.**
Category (c) surface inside a mixed module. Owner of the actual semantics:
OpenSSH's ssh(1) manual — the module even quotes it ("exits with the exit
status of the remote command or with 255"), yet the anchor is RFC 4254, which
owns channel messages, not the client's exit contract. The exit-status 255 and
the client argv modeling are the CLIENT manual's surface; the client's own
reference is the citation this module wants. **What re-anchoring to ssh(1)
would fix:** the client-surface declarations become scopeable. **What it would
not fix:** any RFC 4254 channel-message declarations, which belong in a
different module if they are wanted at all — one module per authority, the
same rule that split `Policy#Binding` from the IAM overview.

**7. `languages/json/grammar.dag` — `json_production_rows`.**
Category (c) with an (a) core. Owner of the table: this repo — the columns
(`rfc_section`, `lhs`, `rhs`, `value_variant`) are a repo-normalized grammar
whose `lhs` names a local `literal` nonterminal and whose `value_variant` maps
to repo AST names; the #13189 review pulled the row on exactly this. The raw
RFC 8259 production strings embedded in the rows are (a)-owned by the RFC.
**What re-homing would fix:** the table stops borrowing RFC authority for a
dispatch shape the RFC does not have. **What it would not fix:** nothing
consumes it for parsing (the production path is `json/parse.dag`, scoped
separately) — so the table is a documentation artifact in code, and deletion
is a live alternative to re-homing; that call belongs to the operator.

**8. `languages/markdown.dag` — `commonmark_gfm_spellings`.**
Category (c). Owner: this repo's serializer policy — choosing which dialect's
spellings to emit over CommonMark 0.31.2 and GFM extensions (tables, task
lists) is a product decision; no single upstream owns "both dialects at once."
Consumed (grep-grade: `truth_table_projection` and the markdown tests name
the spellings) — real
consumers. **What re-classification would fix:** the row stops asking for a
citation that cannot exist. **What it would not fix:** the serializer behavior,
which is fine and consumed.

**9. `tools/curl.dag` — `curl_cli_tool` and its policy rows.**
Category (c) with an (a) core. Owner of the CLI facts: curl's own man page
(exit codes 6/7/28/35/52/56 are the man page's surface — the anchor is
correct for those). Owner of `curl_pinned_version` (>= 7.68) and the localhost
timeout rows: this repo's policy — a minimum version and per-host timeouts are
choices, not curl facts. The #13189 pull also cited an apt install
realization. **What a split would fix:** the exit-code table scopes against
the man page; the policy rows stop living under an external-authority anchor.
**What it would not fix:** the consumers of the pinned version — the policy is
real and should just be labeled as ours.

**10. `version/semver.dag` — `semver_scheme`.**
Category (c) with a filed defect. Owner of precedence semantics: semver.org
section 11. The scheme dispatches `semver_identity_compare` — lexical, not
precedence — so the citation attests a decision rule the code does not
implement: the recurring failure mode filed as
`a_citation_attests_a_decision_rule_the_code_does_not_implement` (#13235). The
type family (`SemVerVersion` and friends) LOOKS like an (a) transcription of
the spec's syntax, but that reading is unverified field-for-field — no
diff against semver.org's grammar was run for this document, so the family's
(a) status is a hypothesis, not a finding; the scheme row is product policy
(which comparison this repo
uses for identity), and that part is the dispute this document asserts. **What fixing the comparison would fix:** the defect, and
then the scheme could honestly cite precedence. **What it would not fix:**
whether a version scheme — a product choice — belongs in extdeps at all.

### (d) unsupported/unconsumed — and why the one candidate did not delete

**11. `extdeps/ebay/mock_corpus.dag` — NOT deleted; a new discriminator found.**
The census (2026-08-22, re-derived 08-26) classifies this module
STILL-UNCONSUMED, and the disposition held it only on a weak
"named somewhere" ground. Chasing the deletion found a surface the census's
mention scan and every path grep miss: in
`src/v1/stage0/src/cli_run/external_authority.rs`, the function
`external_authority_is_clean_tree_roster_excluded_for_module_path` (line 211)
excludes `module_path.ends_with(".mock_corpus")` from the live-anchor walk —
compiled code keyed to a NAME CONVENTION, not to this module's identity. The
module is also typed `PublishedMockCase` data under `std.hermetic_replay`, and
its siblings (`bmc`, `cloud/gcp`, `cron` mock corpora) sit in the same exempt
machinery. So the class is product machinery (hermetic-replay corpus) living
in the extdeps tree by directory accident, and one member can be deleted only
after a **convention-pattern scan of compiled code** (`ends_with(...)` /
`starts_with(...)` over module paths) — a check that must join the deletion
lane's checklist, because it is identity-grade evidence the current census
does not collect. **What deleting one member would fix:** nothing that
matters — the walk degrades by one excluded path; the corpus itself is
unreferenced today by every surface scanned (grep-grade only, which is
exactly the grade this case proves insufficient). **What it would not fix:** the misfiling; the correct
disposition for the mock-corpus family is re-homing to the fixture namespace
the v1 code already recognizes (`extdeps.fixture.`), which is a namespace move
and needs the operator's one decision. No deletion is shipped in this PR.

**12. `dag/extdeps/docker/container_stats.dag` — unconsumed AND unfaithful.**
Category (c)-leaning (d)-adjacent; the row is back on the frontier (returned
by the merged #13304 pullback). No production caller (grep-grade: the review
on #13304 confirmed it independently); the
module adds a repo-authored `cpu_percent` field and reshapes `networks`, so it
is not the Engine API's stats payload field-for-field. Owner of the real
surface: Docker's Engine API stats reference. **What deleting the module would
fix:** one fewer false-claim-shaped artifact; the row leaves the frontier.
**What it would not fix:** nothing consumes it, so nothing breaks — but
deleting discards a half-faithful transcription. Under DESIGN 3c the
dispositions are the same as case 1: name a consumer or delete. If a consumer
is wanted, the repair (drop `cpu_percent`, restore the API shapes, diff
field-for-field) comes first; scoping only becomes available once the
module is consumed, and the "repair-then-scope" order alone would still leave
an unconsumed red.

## What this changes for bulk work

1. **The frontier is mostly (c) wearing (a)'s clothes.** Of twelve disputed
   rows, four are cleanly (a) (one after repair, one family with a per-row
   condition), seven are (c) with (a) cores or (c) outright, and the one (d)
   candidate failed deletion on a new discriminator. That ratio is the review
   rejection rate, now explained: the frontier's rows were harvested by
   anchor-presence, and an anchor is exactly the thing (c) rows can borrow.
2. **(c) rows must not get extdeps scopes** — that is the no-credit rule.
   Their dispositions are: re-home to the product layer (5, 6, 7), re-label as
   policy under a split (4, 9, 10), or stop requesting an impossible citation
   (8). None of these is a placement batch; each is a small, reviewable move.
3. **(d) deletion needs two identity instruments, not one:** the census's
   reachability walk AND a convention-pattern scan of compiled code over
   module-path names. The second is new, cheap, and would have caught the
   mock_corpus hold honestly.
4. **Unconsumed is a delete-or-name-a-consumer decision, not a scope.**
   DESIGN 3c: a declaration nothing consumes is red however well modelled, so
   an unconsumed (a) row is not made scopeable by its faithfulness — the
   options are to name a consumer (then scope) or to delete. In this sample
   that lands on cases 1 and 12: case 1 is a strong delete-or-consume
   candidate (clean transcription, zero consumers), case 12 is a plain delete
   unless a consumer is wanted and the transcription is repaired first.

## Verification status of this document

Every consumption and no-consumption claim above is **grep-grade**: each was
re-checked on this branch by tree-wide search at symbol and module-path grain
(the greps are noted per case), and none is presented as identity-verified.
Case 11 is the proof that this grade has a known hole — a convention-keyed
surface in compiled code
(`external_authority_is_clean_tree_roster_excluded_for_module_path`,
`src/v1/stage0/src/cli_run/external_authority.rs` line 211) that no path grep
sees — so any deletion decision needs the identity-grade instruments named in
point 3, not these greps. The census numbers are
its own (2026-08-22 / re-derived 2026-08-26) and carry that clock. No code, no
tsv, and no roster is changed by this PR; it is a classification document, and
bulk work waits on the operator's ruling on it.
