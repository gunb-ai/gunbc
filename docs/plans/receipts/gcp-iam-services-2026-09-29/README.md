# Successful GCP IAM bootstrap, 2026-09-29

At source `85bdb4e19`, all ten modeled bootstrap stages completed with exit 0.
Cloud Resource Manager was enabled and independently read back on the exact
project; the other required APIs were already enabled. Resource-specific grants,
positive and negative permission probes, and the observer estate read succeeded.
The observer could read the pinned approval-submission credential; the writer
received exact HTTP 403 reading that same known-readable credential.

Both temporary exact-account probe grants were removed and absence verified
before the final observer/apply workload-trust bindings were installed/read back.
The earlier temporary organization Deny Admin grant was also verified absent.
Issued probe tokens retain their original 600-second lifetime; grant removal is
not revocation of already issued tokens. The operator credential file was removed.

18 focused controls passed over 844 byte-identical modules under 6 GiB/no swap.
The full landing suite was not run. This is bootstrap commissioning, not a VM
allocation or a completed approval-workflow acceptance run.

Workflow acceptance was separately dispatched on main as run `36505895733`,
source `15b977d417c1a7de52eae96f48bcd3955fb1f94c`, mode `gcp_iam_converge`, host
`srv1`. That run uses its own GitHub OIDC credentials, not the supplied operator
token. Its success and approval delivery must be established independently.

The acceptance run subsequently completed its release build and its observer
GitHub OIDC authentication step successfully. The plan/file/wait step remains
in progress; approval delivery, apply authentication, and apply/readback have
not yet been established.

Attempt 1 subsequently passed request approval and writer GitHub OIDC
impersonation. Apply accepted a Namecheap provider creation, then refused because
the created subject was not yet visible on immediate readback. No later effects
were applied. This is a partial cloud change, not a successful estate convergence.

Attempt 2 reruns the failed job on the same source, with a fresh run-attempt
identity and a freshly observed plan. It has passed approval and writer OIDC
impersonation and is executing apply/readback. No administrator token is involved.

Attempt 2 completed successfully. The existing approval app admitted the fresh
request; the apply identity was minted only afterward, and the existing workflow
completed its approved effects and readback. Run: https://github.com/gunb-ai/gunbc/actions/runs/36505895733/attempts/2 .
This closes the ordinary GCP IAM workflow acceptance case, not host credential
custody, protected storage commissioning, or VM acceptance.
