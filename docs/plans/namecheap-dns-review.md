# Namecheap DNS review planner

This is the review-only consumer of `gunbc-namecheap-observation/v1`. It produces
one complete before/proposed zone review for service addresses and DNS-01
challenge changes. It reads files and a clock, and writes a review file. It does
not retrieve a secret, call Namecheap or Tailscale, file an approval, acquire a
lease, change DNS, or claim deployment convergence.

The earlier `work/daily-end-to-end` `dev_dns_convergence` work listed pending
requirements but had no receipt consumer. This planner implements that next
step against the current observation authority rather than copying its older
record representation or credential onboarding. The private network strategy
was consulted: ingress identity, backend placement, and outbound connectivity
remain separate. The two application names remain tailnet-only requirements.

## Inputs and provenance

`gunbc.namecheap.planning.run.main` accepts `observation_path`, `intent_path`,
`expected_run`, `expected_attempt`, `expected_revision`, and `output_path`.
The expected identity must come from trusted Actions artifact retrieval;
reading these values from the same JSON and passing them back is not independent
verification. File parsing alone never attests a provider call. The entry checks
schema, run identity, account, domain, reported egress, exact numeric Secret
Manager resource, timestamp, provider result fields, and complete unique host
records. Unknown provider fields remain attached, and the entire original
receipt is carried into the review.

The fixed review freshness window is 30 minutes, inclusive, with future-dated
and noncanonical timestamps refused. This permits artifact retrieval and cold
compiler startup. It is NOT an eventual apply freshness policy: apply must
observe under the acquired writer fence immediately before its re-admission.

Intent schema (addresses and evidence references below are examples, NOT observed
service assignments):

```json
{
  "schema": "gunbc-dev-dns-intent/v1",
  "services": [
    {"service": "svc:tracker-dev", "ipv4": "100.100.1.1", "observation_ref": "<verified service-address observation>"},
    {"service": "svc:approvals-dev", "ipv4": "100.100.1.2", "observation_ref": "<verified service-address observation>"}
  ],
  "challenges": []
}
```

`DevDnsService` is the single authority for each relative owner and service ID.
The zone is derived from `namecheap_account_report`; no backend host is an intent
field. The planner accepts IPv4 candidates in the shared-address range only.
A matching address is NOT proof of TailVIP assignment, a service advertisement,
or tailnet authorization. Evidence references are review inputs and remain
unverified by this offline planner. Existing AAAA at a managed owner refuses;
IPv6 intent support awaits the repository's IPv6 text codec. CNAME, NS, ALIAS,
other unsupported owner data, and multiple A records also refuse rather than
silently removing conflicting records. Existing TXT/CAA at the service owner
are retained. Case and fully qualified owner spellings are compared using the
existing DNS case-folding authority while original provider fields are preserved.

A challenge entry has exactly `service`, `action` (`publish` or `retire`), `value`
(the DNS-01 TXT digest), and `ownership_ref` (the ACME order/ownership evidence).
It changes only that exact TXT value under `_acme-challenge.<service-label>`.
Other TXT values survive. A delegated challenge owner refuses; this slice does
not follow delegations. Duplicate or contradictory intents refuse. A reference
is not proof of ownership: retirement must be re-admitted against verified
ownership evidence before a future writer can act. The TXT shape validator is
owned by `extdeps.acme.dns01`, citing RFC 8555 section 8.4.

## Output and future approval binding

The JSON result contains `plan` and `sha256`. The plan carries the complete
original observation, intent, proposed host fields, unchanged result fields,
record-change/no-op standing, and shared zone writer key `namecheap/zone/gunb.ai`.
Existing record order and fields are preserved. Address updates change only
`Address`; existing TTL and unknown metadata survive. New records use TTL 300.
Identical input yields identical content, and an already satisfied address/token
is a record no-op. No-op does not assert mail, TLS or deployment convergence.

The digest is SHA-256 over the existing injective `consent_field` framing of the
actual serialized plan content. It binds the observation identity and numeric
credential version, every retained provider field, requested evidence references,
proposed records, and remaining apply obligations. This is deterministic for
identical inputs, not a claim that differently ordered equivalent JSON has one
digest; conservative ordering changes can require renewed review.

`dns_review_still_matches` checks a fresh observation against the frozen provider
fields, program revision, credential generation and intent. It rejects drift,
including changes in unknown provider fields. A true result is a necessary
comparison only, not authorization, a lease, or provider CAS. The eventual
mutation must reuse `std.scoped_authorization` and `gunbc.auth.approval_gate`,
with the exact admitted subject and attempt. This planner deliberately does not
file a premature mutation request while its apply requirements remain open.

`mail_mode=unobserved` and `write_authority=withheld` are preserved. Before apply:
independently observe mail mode; prove a lossless mapping of all retained fields
to setHosts (unknown fields cannot be silently dropped); verify service/policy
and ACME ownership evidence; acquire one fenced writer for ALL zone changes;
reobserve under that fence with the pinned credential; present the fully admitted
plan through the existing approval app; and independently read back afterward.
The provider offers whole-zone replacement, not a CAS over this observation.
Console/API writers outside the fence remain a coordination prerequisite.

## Running and controls

With a current gunbc binary under an enforced memory scope, run:

```sh
systemd-run --user --scope -p MemoryMax=26G --quiet \
  /absolute/path/to/gunbc run --source-root dag --source-root src/v2 \
  --entry dag/gunbc/namecheap/planning/run.dag \
  --arg observation_path=/path/to/namecheap-observation.json \
  --arg intent_path=/path/to/dns-intent.json \
  --arg expected_run=RUN_ID --arg expected_attempt=ATTEMPT \
  --arg expected_revision=OBSERVATION_PROGRAM_SHA \
  --arg output_path=/path/to/dns-review.json
```

A failure exits nonzero and does not publish a new plan. A pre-existing output
file may still exist; consumers must require success and check the bound input
identity rather than treating file existence as a new receipt.

The `.dag` controls are `test.claim.namecheap_dns_plan_witness_test`. The offline
file-path test is `test/namecheap/planner_files.py`: it runs the real file entry,
checks preservation and combined service/challenge output, verifies SHA-256 with
an independent Python oracle, and plants a mismatched run identity. It uses only
synthetic provider data, not a live credential or authenticated observation.

Authorities: [Namecheap setHosts](https://www.namecheap.com/support/api/methods/domains-dns/set-hosts/),
[Tailscale Services](https://tailscale.com/docs/features/tailscale-services),
[ACME DNS-01](https://www.rfc-editor.org/rfc/rfc8555#section-8.4).

Local validation: all 23 planner controls passed. The real file entry passed the
preservation, stable-origin, combined DNS/DNS-01, independent digest and wrong-run
controls. The existing query-transport control also passed after its import
closure staging helper was shared with the planner control. Repository-wide
qualification remains the exact-head CI run on the follow-on PR.
