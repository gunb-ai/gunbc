# Allocation floor name-resolution repair

CI run 36512520028 at 0990a2ab7 reached the floor and refused its changed-witness
preflight: three stale debt rows and unrostered bare provider names. The exact
published measurement receipt is ci-before-repair.json. This superseded run was
cancelled after retaining its failure evidence; no green floor is claimed.

79155ee24aaf11742f3452e99437b51fac65eb27 uses existing receiver operations for
collection and string operations, explicit local pattern bindings, and retires
only the three already-absent debt pairs as ImportsFixed. It neither imports
unrelated test functions suggested by the diagnostic nor modifies the compiler,
relaxes the gate, or adds debt entries. Allocation key length uses string_length.

The existing controls pass under 6 GiB/no swap with byte-identical dependency
closures: request decoding 4, session handling 11, JWT/OIDC 15, workspace access
and resource configuration 5. The unchanged Rust binary is identified in
receipt.json. These focused checks do not establish the whole floor; replacement
integrated CI run 36514441574 is pinned to the repair. Full server startup on the repair also passed: all 851 sources, 288.254 seconds,
peak 12,009,013,248 bytes (11.18 GiB), 12 GiB cap, no swap/OOM. The source diff was
empty and the same branch-local binary was used; see preview-receipt.json. No
service was installed or deployed.
