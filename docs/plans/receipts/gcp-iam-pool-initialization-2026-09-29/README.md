# Exclusive pool initialization source qualification

The operator confirmed on 2026-09-29 that no other administrator will update the
pool policies during this bootstrap. The change models a one-off initial policy
publication under that exclusivity assumption, then requires an observed etag for
all normal updates. This is not provider-enforced CAS during initialization.

All 15 focused controls passed under 6 GiB with no swap, using the branch release
binary and 843 byte-identical dependency modules. Controls distinguish empty
wire policies from update authority and reject nonempty unfenced policies,
unsupported policy versions, existing audit configuration, undeclared pools,
and incomplete or revisionless initialization readback.

The complete bootstrap module typechecked in this closure. The new cloud-write
path and interrupted-publication recovery have not been exercised against GCP.
No live pool initialization or workflow credential-minting acceptance is claimed.
The create-only publication journal stays in the same recovery workspace; an
uncertain publication cannot be retried from a fresh workspace.

An existing administrator credential is still needed for initial bootstrap. The
normal approval workflow first authenticates as iam-observe through its dedicated
pool, then requests approval and obtains iam-converge credentials. Its initial
trust is not yet installed, so it cannot create that trust for itself. There is
no usable gcloud installation, ADC file, or configured token file in this shell.
