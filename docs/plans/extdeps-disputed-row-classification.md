# Disputed extdeps rows: a two-axis classification template, tested on twelve cases

Operator ruling (2026-10-05): placement batches pause because about half of
placements were rejected on review — the classification rule is not yet ready to
be used as a template. This document builds the rule the way the census built
its population: by classifying a representative, disputable sample, and it
records **two judgments per case**, because the disputes of the last several
batches were two different questions being answered with one label:

- **Semantic standing** — who owns the semantic information the declaration
  carries: *exact upstream fact* (a versioned upstream artifact states the
  surface and the declaration transcribes it **field-for-field**), *shared
  formal semantics* (recorded somewhere nameable but not one product's
  artifact), *product policy* (the product's own choice: dispatch algebras,
  serializer policies, pinned versions, normalizing tables, transports), or
  *unsupported* (nothing states it and no product purpose holds it up).
- **Consumption standing** — per DESIGN 3c's three honest states: *executing*
  (consumed by execution in this tree), *declared frontier with trigger* (a
  named consumer lands in a named later change, trigger stated beside it), or
  *dangling* (nothing consumes it — red regardless of how well modeled it is).

The **disposition follows from both axes together**: scope (fact + executing),
re-home or re-label (policy + executing), delete or name-a-consumer (fact or
unsupported + dangling), repair (a mixed module whose subjects must be separated
before either axis can be judged honestly). No single axis decides.

**Field-for-field is the predicate, not a later formality.** Semantic standing
is *decided* by diffing the declaration against the upstream artifact it cites:
if the declaration adds, drops, reshapes or renames anything the artifact does
not state, the standing is not exact-upstream-fact, whatever the anchor says.
Every "must still be diffed before scoping" hedges below were errors in the
previous draft; the diff is how the judgment is made.

**Misfile shapes (the recurring review failure).** A citation can be live and
real and still not own the declaration. Observed, each a real pulled row: a
tool's CLI citing the standard the tool implements (ssh client, coreutils
sha256sum); a repo-normalized table citing the spec it normalizes (json grammar
rows); a dispatch algebra citing one tool it drives (OpenBmcOperation citing
busctl); a repo extension inside an otherwise transcribed enumeration silently
breaking the predicate (a `Other{raw}` variant beside 17 transcribed codes);
a repo plan citing fragments of a CLI grammar it does not fully reproduce.

## The twelve cases

Twelve disputed cases — one of them a 19-row family — chosen to span the dispute
shapes the review history actually produced. Every consumption claim is
grep-grade (see the verification note); semantic standings were decided by
reading each module against its cited artifact.

**1. `cloud/gcp/errors.dag` — `GcpRpcCode` (17 `google.rpc.Code` values plus a
repo-added `GcpRpcCodeOther { raw }`).**
Semantic standing: **exact upstream fact, predicate failing.** Owner: Google's
API design docs, the `google.rpc.Code` table (`cloud.google.com/apis/design/errors#http_mapping`).
The 17 values are transcribed; the added `Other { raw }` variant is not on the
upstream surface, so the module as a whole is not field-for-field and the (a)
predicate does not pass.
Consumption standing: **dangling** (grep-grade: the only reference outside the
module is its own grounding witness test).
Disposition: **delete or name a consumer** (DESIGN 3c). Repair — moving the
`Other` variant out of the transcribed enumeration — is worth doing only if a
consumer or a declared frontier with a trigger appears; repairing a dangling
module still leaves it dangling.

**2. `docker/cli.dag` — `DockerCreateSpec` and its flag types.**
Semantic standing: **product policy.** The type is a repo-normalized plan: its
flag grammars are partial (Docker's CLI reference owns `--restart` fully —
`on-failure:N` cannot be expressed by `DockerRestartPolicy` as written — and
`DockerNamespaceMode` covers only host/private), so it borrows CLI grammar
fragments rather than transcribing the cited surface. The earlier draft's
"counter-example to the misfile" was wrong: field-for-field fails, so this is
not an upstream fact yet.
Consumption standing: **executing** (grep-grade: `DockerCreateSpec` is consumed
by the `dag/gunbc/spark/*` serving modules and `docker/container_inspect.dag`).
Disposition: **repair.** Either complete the grammars field-for-field against
Docker's CLI reference — after which the (a) predicate is judged again on the
diff, not asserted — or re-label the plan as repo vocabulary in the product
layer. The consumers keep working either way.

**3. The `github/*` family (one upstream, many surfaces).**
The family rule (held sound from batch 5): one upstream, docs.github.com's REST
reference, but the two recorded judgments are made **per row**: each surface's
semantic standing is decided against its specific REST page (the `Gist` module
passes that way; `GitHubErrorShape` failed when its locator redirected to
generic getting-started material), and each row's consumption standing is its
own. Placement batches are paused; nothing here proposes restarting one — the
family only means who the upstream is is not re-litigated per row.

**4. `crypto/hash.dag` — `HashAlgorithm`/`Digest` beside `sha256sum` machinery.**
Semantic standing: **mixed — repair required.** The FIPS 180-4 facts
(algorithm enumeration, digest widths) are exact upstream facts, but they sit as
local carriers inside a module whose other half is `sha256sum(1)` machinery — a
tool realization with its own owner (the coreutils manual) — so the module as a
whole is not the FIPS surface and not the tool's surface either.
Consumption standing: **executing** (grep-grade: the shell-verifier side is
exercised by the sccache pin and live-deploy verification paths).
Disposition: **repair — split.** The FIPS facts move to a module of their own
and are re-judged field-for-field against FIPS 180-4; the verifier machinery
re-homes to the product layer. The split is honest bookkeeping: the consumers
keep working.

**5. `ssh/session.dag` — `service ssh.Session` and the ssh(1) facts.**
Semantic standing: **mixed — split required.** The module quotes ssh(1) ("exits
with the exit status of the remote command or with 255") — an exact upstream
fact owned by the OpenSSH client's manual, which is the citation this subject
wants, not RFC 4254 — but it also carries repo transports, session records and
mocks, which are product machinery with no upstream owner.
Consumption standing: **executing for the machinery subjects** (grep-grade:
repo consumers name them); the ssh(1)-fact subjects' consumption must be judged
per subject after the split.
Disposition: **repair — separate the subjects** (one module per authority, the
same rule that split `Policy#Binding` from the IAM overview), then re-judge each
side on both axes.

**6. `bmc/openbmc_operation.dag` — `OpenBmcOperation`.**
Semantic standing: **product policy.** Owner: this repo — the dispatch algebra
over file ops, systemctl, busctl, jq, mkdir/rm/cp, sleep is our vocabulary of
what a BMC operation is; the busctl man page owns only busctl's argv semantics.
Consumption standing: **executing** (grep-grade: `openbmc_fan_control` and
`openbmc_password_ssh_transport` name it).
Disposition: **re-home** to the product layer. The frontier row leaves because
the module stops claiming to be extdeps at all; the internal consumers keep
depending on it — they should, it is the product's own vocabulary.

**7. `languages/json/grammar.dag` — `json_production_rows`.**
Semantic standing: **product policy.** Owner: this repo — the columns
(`rfc_section`, `lhs`, `rhs`, `value_variant`) are a repo-normalized grammar
whose `lhs` names a local `literal` nonterminal and whose `value_variant` maps
to repo AST names; the raw RFC 8259 production strings embedded in the rows are
upstream fragments, not the surface the table models. The #13189 review pulled
the row on exactly this.
Consumption standing: **dangling** (grep-grade: no production consumer; the
parse path is `json/parse.dag`, scoped separately).
Disposition: **delete or re-home** — the table is a documentation artifact in
code; both dispositions are live and the call belongs to the operator.

**8. `languages/markdown.dag` — `commonmark_gfm_spellings`.**
Semantic standing: **product policy.** Owner: this repo's serializer policy —
choosing which dialect's spellings to emit over CommonMark 0.31.2 and GFM
extensions (tables, task lists) is a product decision; no single upstream owns
"both dialects at once."
Consumption standing: **executing** (grep-grade: `truth_table_projection` and
the markdown tests name the spellings).
Disposition: **re-label** as product policy; the row stops requesting a
citation that cannot exist. The serializer behavior, which is fine and
consumed, is untouched.

**9. `tools/curl.dag` — `curl_cli_tool` and its policy rows.**
Semantic standing: **mixed — repair required.** The exit-code table (6/7/28/35/
52/56) is an exact upstream fact owned by curl's own man page, which the anchor
cites correctly. The pinned minimum version (>= 7.68), the localhost timeout
rows, and the apt install realization are product policy — choices, not curl
facts.
Consumption standing: **executing** (grep-grade: the tool rows have repo
consumers).
Disposition: **repair — split** the policy rows out from under the
external-authority anchor; the CLI-fact side is then re-judged field-for-field
against the man page on its own diff.

**10. `version/semver.dag` — `semver_scheme`.**
Semantic standing: **exact upstream fact, unfaithfully implemented.** Owner:
semver.org section 11 — precedence is upstream-owned, and the scheme's lexical
dispatch (`semver_identity_compare`) was an unfaithful implementation of it, not
a product choice; the previous draft's "product policy" label was wrong. The
citation attests a decision rule the code did not implement: the recurring
failure mode filed as `a_citation_attests_a_decision_rule_the_code_does_not_implement`
(#13235).
Consumption standing: **executing** (grep-grade: the v1 stage0 mirror consumes
it).
Disposition: **repair the implementation** (the filed row); whether the type
family passes the field-for-field predicate against semver.org's grammar is
re-judged from that diff — it has not been verified and is not asserted here.

**11. `extdeps/ebay/mock_corpus.dag` — dangling repo data; the deletion-safety
check.**
Semantic standing: **unsupported.** The module is repo-authored
`PublishedMockCase` hermetic-replay data; no upstream states it.
Consumption standing: **dangling.** The census (2026-08-22, re-derived 08-26)
classifies it STILL-UNCONSUMED. The earlier draft called the v1 mirror's
`ends_with(".mock_corpus")` predicate (in
`external_authority_is_clean_tree_roster_excluded_for_module_path`,
`src/v1/stage0/src/cli_run/external_authority.rs` line 211) a consumption
surface — that was wrong: the predicate reads **no module content**, it is a
name-keyed classification, and a classification is not a consumer. Deleting one
member changes nothing the predicate computes.
Disposition: **delete vs declared future consumer** — the correct disposition
for the mock-corpus family is operator's, between deletion and re-homing to a
fixture namespace with a named trigger. The convention scan survives only as a
**deletion-safety check** (before deleting, confirm no compiled logic keys on
the module's name in a way that reads its content), not as consumption
evidence — and it is the check the census's mention scan does not collect.

**12. `dag/extdeps/docker/container_stats.dag` — dangling and not
field-for-field.**
Semantic standing: **product policy over upstream fragments, predicate
failing.** It adds a repo-authored `cpu_percent` field and reshapes `networks`,
so it is not the Engine API's stats payload; owner of the real surface:
Docker's Engine API stats reference.
Consumption standing: **dangling** (grep-grade: no production caller; the
#13304 review confirmed it independently).
Disposition: **delete or name a consumer** (DESIGN 3c). Repairing the
transcription — dropping `cpu_percent`, restoring the API shapes, re-diffing —
is worthwhile only once a consumer or a declared frontier with a trigger
exists; on today's tree the honest move is deletion or a named trigger, and the
previous draft's "repair-then-scope" was a 3c violation in proposal form.

## What this template changes for bulk work

1. **Two judgments, two instruments.** Semantic standing is decided by a
   field-for-field diff against the cited artifact; consumption standing is
   decided by the census-style reachability walk at identity grain, labeled
   grep-grade until it is not. Neither judgment can be made from the anchor
   alone — an anchor is exactly the thing product-policy rows can borrow.
2. **Dispositions are derivable, not argued.** fact + executing → scope;
   fact + dangling → delete or declare a trigger; policy + executing →
   re-home or re-label; unsupported + dangling → delete (safety-checked);
   mixed → repair by splitting subjects, then re-judge each side. A review
   that has to argue past the template is telling the author an axis was
   judged without its instrument.
3. **Shared formal semantics stays a discipline.** In these twelve cases no
   row finally landed there: each candidate for it resolved to an
   upstream-owned fact once the recording source was named. That is a
   statement about this sample, not about the frontier — twelve hand-picked
   disputed cases cannot establish how rare shared-but-unowned semantics is
   in general. What the sample does establish is the procedure: when a row
   looks shared, hunt for the recording source; if none exists, the row is
   product policy or unsupported, never settled as shared by default.
4. **Unconsumed is a delete-or-declare decision, not a scope.** DESIGN 3c: a
   declaration nothing consumes is red however well modeled. In this sample
   that lands on cases 1, 7, 11 and 12; cases 1 and 12 additionally fail the
   field-for-field predicate, so their repair work is gated on a consumer or
   trigger being declared first.

## Verification status of this document

Every consumption and no-consumption claim above is **grep-grade**: each was
re-checked on this branch by tree-wide search at symbol and module-path grain
(the greps are noted per case), and none is presented as identity-verified. The
deletion-safety lesson of case 11 stands with its corrected meaning: compiled
code can key on module *names* without reading module *content*
(`external_authority_is_clean_tree_roster_excluded_for_module_path`,
`src/v1/stage0/src/cli_run/external_authority.rs` line 211), so any deletion
carries a convention-pattern scan alongside the reachability walk. Semantic
standings were decided by reading each module against its cited artifact; where
a field-for-field diff has not been run, the standing above says so rather than
asserting the predicate. The census numbers are its own (2026-08-22 /
re-derived 2026-08-26) and carry that clock. No code, no tsv, and no roster is
changed by this PR; it is a classification template, and bulk work waits on the
operator's ruling on it.
