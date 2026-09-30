# Authorized storage-key bootstrap

The user explicitly authorized temporary use of an operator GCP access token for
initial setup, then requested completing the flows. Read-only cloud metadata first
returned HTTP 404 for the secret, version 1 and its policy. The existing native
`.dag` secret actuator created and verified version 1. A separate invocation
reconciled and reread the declared fleet accessor cell. Both invocations exited 0,
under 6 GiB/no swap. The missing-credential control exited 1 without cloud effects.

This was explicit operator bootstrap, not an app approval or a fabricated GitHub
run. The token arrived on echo-disabled stdin, was held in a private 0600 temporary
file for the native interpreter, and that file was removed in the runner's finally
path. It was never put in command arguments or repository content. Logs contain
only secret identity, version, digest prefix and policy standing; the temporary
credential path is redacted in the copied log.

Separate live metadata returned 404 for iam-observe and iam-converge service
accounts, and 200/disabled=false for fleet-cloud-convergence. The permanent IAM
bootstrap is being developed independently from main on codex/gcp-iam-bootstrap.
No service account, pool, role or deny policy was created by these two runs.
No host credential was installed; no VM launched or site deployed.
