# Namecheap credential custody

The owner reports `projects/582015116396/secrets/namecheap-api-key` provisioned.
The project number matches `fleet_secrets_project_number`; the project ID comes
from `fleet_secrets_gcp_project` (`gunbai-secrets`). No new version is provisioned
by this lane, and no key is requested in chat or in a persistent local file.

The owner also supplied account `briansrls` and an API allowlist entry named `dev`
for `96.224.201.7`. These are typed operator reports in
`gunbc.namecheap.account`, not evidence of an authenticated API call or a
permanent DNS-controller placement.

`gunbc.namecheap.credential` binds the container once. Its `latest` reference identifies the
container for IAM and is resolved once by the discovery observer, which records
the returned numeric version. The numeric selector controls apply to future
reviewed mutation consumers, not this discovery path: an approval over a DNS
mutation must not silently switch credential generations through an alias. `gunbc.namecheap.credential_read` uses the existing
`fetch_secret_ref_credential` with `WorkloadIdentityToken`; the shared reader
checks the response resource against the requested project, secret and version.
It does not print or persist the payload.

The dedicated `namecheap-dns` federation joins `gcp_iam_converge_targets`. Its
trust is pinned by the shared fleet job claim authority to this repository, the
fleet-converge workflow at main, a dispatch on main, and the `namecheap-dns`
environment. Its only secret grant is accessor on `namecheap-api-key`. The
existing IAM workflow plans as iam-observe, files a request in the approval app,
impersonates iam-converge only after approval, rechecks the approved plan, applies
and independently reads back. Enrollment is not evidence of an applied grant.
The new target also changes the resource-local bootstrap bindings needed by the
IAM controller; those must be observed before running its apply.
This preserves the repo's distinction between routine per-secret federated
reads and operation-specific human approvals. For DNS mutations, the consumer
must use the existing `approval_gate` and scoped authorization over the frozen
zone plan; secret possession alone never authorizes a zone replacement. This
increment does not yet connect that DNS mutation consumer.

Live status: secret creation and the allowlist are operator-reported. No enabled
version or applied IAM binding has been independently observed. This local shell
has no WIF token or gcloud installation. No provider request, IAM write, key
access, or DNS mutation has been executed. The `namecheap_observe` fleet mode now has a dedicated job that supplies WIF
and records the exact secret version plus authenticated getHosts readback. It
still must land on main and its federation must converge before a live dispatch
can pass the provider's main-only claim pins.
The earlier DNS model lives on `work/daily-end-to-end`; this lane is based on the
current repository so it can reuse the landed approval app rather than copying
its newer authorities into that older checkout. The previous local `api_key_file`
onboarding proposal is superseded by this Secret Manager reference.

Validation on the current main base: a combined full-root run evaluated all
21 IAM convergence controls, five credential controls, 15 XML/provider controls,
and four workflow controls successfully, then regenerated the workflow through
`tools.generated_artifact_gate.main_wet_one`. Its receipt counts 46 because it
also ran one redundant aggregate IAM check, subsequently removed; the retained
45 controls all ran in that invocation. The generated YAML was independently
parsed and checked for the dispatch mode, dedicated identity, event SHA checkout,
SSH exclusion, and success-only receipt upload. The self-contained loopback
transport harness passed and observed the fixture query arriving without the
fixture key in curl's argv. These are local checks, not live GCP or Namecheap
observations.


## Read-only verification workflow

`fleet-converge` mode `namecheap_observe` uses the selected fleet runner and the
`namecheap-dns` environment. The build and observer both check out the event SHA;
`expected_revision` cannot substitute another revision under the trusted WIF
identity. The shared fleet job is excluded, and this job receives no SSH key.

The entry checks public IPv4 egress against the owner-reported allowlist, fetches
one Secret Manager version through the shared checked reader, and issues only
`namecheap.domains.dns.getHosts`. The encoded query travels to curl via stdin,
not argv or a persistent credential file. There is no setter or configurable
command in this provider interface. Every read has a timeout. Transport and
provider refusals omit raw response bodies, and a response that contains the
credential is withheld before decoding; the decoded retained result and host
attributes are checked again before the observation can reach the receipt.

The XML subset reader refuses unsupported syntax and ambiguous envelopes rather
than guessing: DTDs and external entities, duplicate attributes, extra documents,
namespace overrides, malformed records, duplicate provider record IDs, and a
response for another domain/command cannot yield an observation. It preserves
all provider result and host attributes, including unknown ones. Unsupported XML features
(including numeric character references) are a located read refusal; this is not
a claim to implement every XML document.

`target/namecheap-observation.json` records the run ID, attempt, revision, start
time, exact credential version, account, observed public egress, domain and
returned result and host fields. The job uploads it only after success, so failure cannot
publish a previous run's receipt. `mail_mode=unobserved` and
`write_authority=withheld` are deliberate: getHosts does not independently prove
the console's mail mode, and this observer grants no DNS mutation authority.

The remaining live sequence is: land the reviewed code on main; observe/update
the existing IAM controller's resource-local bootstrap reach for the new target;
run `gcp_iam_converge` and approve its exact plan in the existing app; then run
`namecheap_observe` on a runner whose public egress is allowlisted. Secret
possession, a modeled grant, and a pure witness are not substitutes for those
readbacks. Full-zone DNS mutation, DNS-01 renewal coordination, and dashboard
fleet cutover remain separate unfinished consumers of this observation.

Provider authority: [Namecheap getHosts](https://www.namecheap.com/support/api/methods/domains-dns/get-hosts/).

Landing boundary: the active main ruleset requires the `witnesses` status and
merge queue. The IAM authority also requires reviewed main code; the feature
branch cannot impersonate the dedicated main-pinned identity. A read-only query
of the latest 100 fleet workflow dispatches found no `gcp_iam_converge` run. That
is limited history, not proof that bootstrap never happened. The bootstrap
reach and independent readback are still unverified.

## Review 5331195485 corrections

The writer, generated console display and artifact uploader now consume
`gunbc.namecheap.observation_artifact.namecheap_observation_receipt_path`. The
workflow shell control reproduces the original unmatched receipt read on the old
generated YAML, then exercises the regenerated success path with only the
declared JSON output. Its planted unmatched read must fail.

The production client keeps the raw response exclusion and also checks all
decoded retained result and host attribute names and values, including unknown
fields, before returning an observation. Six publication controls cover raw,
URI-encoded, and XML-entity-encoded echoes plus ordinary entity preservation.
Disabling the decoded guard in an isolated test copy makes the three XML echo
controls fail while raw/URI and ordinary-value controls still pass. Refusals
contain no offending value and never claim a partially redacted snapshot.

The previous head's CI floor identified bare algebra-provider references and an
ambiguous shorthand `domain` binder. List operations now use their native method
forms, and the observer uses an explicit binder. Uppercase `Host` remains unchanged.

Correction validation: the combined full-root run passed 51 controls and
regenerated the workflow. After the CI-reference corrections, all 15 parser and
six publication controls passed again. The regenerated emitted shell passed the
clean-root success control, left the exact upload artifact, and rejected the
planted unmatched receipt read. `git diff --check` passed. Exact-head CI is the
remaining repository-wide check; no live provider or IAM actuation was used.
