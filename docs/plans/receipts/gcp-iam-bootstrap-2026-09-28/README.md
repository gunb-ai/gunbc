# IAM bootstrap qualification and partial commissioning

Source: `a23a0f99cbfe3f39da52cd40638eb44c85e76c6c`, based on main
`2886b9e396d5ca6026c54212f27c52ea4113fba4`. Independent draft: #12565.

The branch's own release build succeeded. Its binary passed all three focused
controls, printed the exact bootstrap plan, and refused the missing-credential
control with the expected reason before creating a started receipt. All commands
ran under `MemoryMax=6G`, `MemorySwapMax=0`. `receipt.json` records binary and
closure-manifest hashes. The runbook reproduces the byte-identical closure; this
is not a full landing-suite result.

## Cloud result: stopped before permissions were granted

Attempt `20260928-a23a0f99c-1` submitted creation of the IAM job's pool, provider,
and observer account, but immediate readback did not establish their readiness.
It refused without reaching role or permission stages.

Attempt `20260928-a23a0f99c-2` reread the same named resources, created the apply
account and missing bare Namecheap pool/account, and completed the identity
stage. Existing target resources were read, not replaced. It then created all
seven exact role definitions and independently reread them.

GCP refused the next stage's deny-policy creation with HTTP 403:
`Permission iam.googleapis.com/denypolicies.create denied on resource
cloudresourcemanager.googleapis.com/projects/582015116396`.

No capability bindings or workflow impersonation bindings were applied by this
bootstrap. No target provider was created. The privilege probes and real
OIDC/approval workflow acceptance have not run. Existing unrelated identities and
access were not removed. Roles are definitions, not grants to the new accounts.

The operator-supplied token was read through a private temporary file, never a
command argument. The launcher removed that file after each attempt. Receipt
copies redact the temporary pathname and contain no token. The refusal is a
missing permission, not an observed token-expiry error.

## Resume

An existing administrator credential or approved administrative workflow must
supply the deny-policy authority on `gunbai-secrets`. The bootstrap never grants
that authority to itself. Run the same modeled entry with a new receipt identity;
completed named subjects and role definitions are reread before further effects.
Later capability and impersonation checks may expose additional missing
permissions; they have not been qualified by this partial run.

Successful deny readback, capability reconciliation, observer/apply checks, and
final workload trust must precede the real approval-workflow acceptance run.

## Fresh-token retry

Attempt `20260928-a23a0f99c-3` used a newly supplied operator token and reread
the existing identities and seven role definitions. GCP again refused deny-policy
creation with HTTP 403 for `iam.denypolicies.create`. Capability and workflow
trust stages were not reached. Temporary credential removal was independently
checked. Refreshing the token did not supply the missing IAM permission.
