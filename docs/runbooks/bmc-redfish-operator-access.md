# BMC Redfish reads

**Goal:** read system inventory and chassis sensors through the dedicated management interface. Resolve the host and rotated credential from fleet/access authorities; keep deployment addresses and secrets out of this public runbook.

**Method:** with management-network access and a private `0600` netrc file, invoke the modeled read:

```sh
BMC_HOST=<management-host> BMC_NETRC_FILE=<private-netrc> gunbc run --source-root dag \
  --entry dag/gunbc/tools/bmc_read_telemetry.dag --function bmc_read_telemetry
```

First contact uses `gunbc.tools.bmc_first_contact.bmc_first_contact`, whose shared authentication path places and removes a private per-call netrc. A failed factory login does not by itself establish rotation. Never put credentials in command arguments.

**Validation:** exit zero establishes successful system and sensor GETs, not agreement with expected inventory. Independently compare the returned hardware and readings with the intended host. `extdeps.boards.asrock_rack` owns resource identifiers and firmware-specific surfaces; ALTRAD8UD observations use chassis Sensors where Power/Thermal is empty. Projection controls live in `test.claim.bmc.bmc_redfish_grounding_witness`.

No response: check AC, management routing and authentication separately. Power-limit PATCH is a separate management write requiring its own policy and authorization; these read tools do not perform it. A BMC's self-signed TLS transport is not evidence of its identity.
