# Temporary bootstrap authority convergence receipt

Source: `70009ad9d2c8f184153976576f5a1ec2211da1b3` on draft PR #12565.
The branch binary passed 12 focused controls over 843 byte-identical modules,
within 6 GiB with swap disabled. The full landing suite was not run.

The organization-scoped run verified the operator subject and primary IAM email,
verified project ancestry, granted the time-bounded Deny Admin cell, installed
and read back the project deny policy, then removed the exact temporary grant
and verified its absence. The retirement journal and sanitized execution log
are included. Existing unrelated grants were outside the owned cell.

The capabilities stage subsequently applied the project create-role binding,
then stopped on a successful pool getIamPolicy response containing `{}` with
neither a body etag nor an HTTP ETag. No pool policy write followed. Account
privilege probes and workflow trust were not reached. The bootstrap as a whole
is incomplete; no VM or fleet deployment acceptance is claimed.

The earlier project-scoped attempt was rejected by GCP with HTTP 400: Deny Admin
is grantable at organization scope. Its log and initial seven-control receipt
are historical, not qualification of the final source. The new run also binds
the verified stable subject to the actual primary email `brian@gunb.ai`.

## Remaining dependency

The pool's initial IAM policy needs an established safe publication contract.
A successful empty read does not supply an optimistic-concurrency token. Do not
replace this with an unconditional setIamPolicy or fabricate an etag. Google's
[Policy contract](https://docs.cloud.google.com/iam/docs/reference/rest/v1/Policy)
describes etag as the concurrency protection; the current required-etag wire
shape reports this boundary as a decode refusal. A follow-up should model the
missing publication prerequisite explicitly, including provider-supported
initialization or established exclusive ownership, before enabling the write.

The temporary credential file was removed by the launcher. No token is included.
