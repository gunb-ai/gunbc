# BMC Redfish operator access runbook (srv1 / srv2)

Enable and verify out-of-band Redfish telemetry on the self-hosted CI fleet hosts. This runbook is operator-private access guidance; concrete host IPs and rotated credentials live in **ctrl** (`plans.fabric.operator_fleet`), not in this repo.

## Prerequisites

- Management-network reachability to each host's **dedicated BMC NIC** (not the host OS IP).
- ASRock Rack ALTRAD8UD-1L2T boards ship **OpenBMC** (Redfish + IPMI). Factory login is cited in `dag/extdeps/bmc/openbmc.dag` (`root` / `0penBmc`) until first-contact rotation.
- `curl` on your operator workstation (or use the gunbc tool below).

## 1. Confirm Redfish is enabled

OpenBMC exposes Redfish by default on HTTPS port 443.

```bash
# Replace with the BMC management IP from ctrl (not the host OS address).
export BMC_HOST=<srvN-bmc-ip>

curl -sk "https://${BMC_HOST}/redfish/v1/" | head
```

Expected: JSON with `"RedfishVersion"` and a `Systems` link collection.

If connection fails:

1. Verify you are on the management VLAN / VPN segment that routes to the BMC NIC.
2. Ping the BMC IP (ICMP may be disabled; try curl even if ping fails).
3. Confirm the host is powered (BMC is reachable when AC is connected; host OS need not be up).

## 2. Authenticate (factory vs rotated)

Resolve each host's BMC management IP from **ctrl** `plans.fabric.operator_fleet` (not the host OS address). Do not record fleet IPs in this public repo.

**First contact (factory default only — already rotated on srv1/srv2):**

```bash
curl -sk -u 'root:0penBmc' "https://${BMC_HOST}/redfish/v1/Systems/system"
```

**Production (rotated secret):** use a `0600` netrc file — password MUST NOT appear on curl argv:

```bash
# ~/.bmc-netrc (mode 0600): machine <srvN-bmc-ip> login root password <rotated>
export BMC_NETRC_FILE=~/.bmc-netrc
curl -sk --netrc-file "$BMC_NETRC_FILE" "https://${BMC_HOST}/redfish/v1/Systems/system"
```

Auth failure with factory credentials means rotation already happened (expected on production hosts).

## 3. Read telemetry resources (inventory + sensors)

**Live shape on ALTRAD8UD-1L2T (OpenBMC / Redfish 1.15, verified srv2 2026-06-20):**

| Resource | Redfish path | Modeled shape |
|----------|--------------|---------------|
| System inventory | `/redfish/v1/Systems/system` | `RedfishSystemInventory` (CoreCount, MemorySummary, Model, Manufacturer) |
| Chassis sensors (power/temp) | `/redfish/v1/Chassis/ALTRAD8UD_1L2T/Sensors` | `RedfishChassisSensorCollection` |
| Chassis Power/Thermal | `/redfish/v1/Chassis/ALTRAD8UD_1L2T/Power` etc. | **EMPTY** on this board — use Sensors |

Chassis member id is `ALTRAD8UD_1L2T` (underscores), cited in `extdeps/boards/asrock_rack.dag`.

Example reads (netrc auth):

```bash
export BMC_HOST=<srvN-bmc-ip>
curl -sk --netrc-file "$BMC_NETRC_FILE" "https://${BMC_HOST}/redfish/v1/Systems/system" \
  | jq '{Model, Manufacturer, PowerState, ProcessorSummary, MemorySummary}'

curl -sk --netrc-file "$BMC_NETRC_FILE" \
  "https://${BMC_HOST}/redfish/v1/Chassis/ALTRAD8UD_1L2T/Sensors" \
  | jq '.Members[].@odata.id' 
# Per-sensor: power_PWR_Core_VRD (~95W), power_PWR_SOC_VRD (~16.5W), temperature_TEMP_* …
```

**Grounding mapping:** `ProcessorSummary.CoreCount` → `HardwareThreadCount`; `MemorySummary.TotalSystemMemoryGiB` → `ByteSize`; power sums `power_PWR_*` sensor readings when Chassis Power is empty.

## 4. gunbc read-only poll (modeled transport)

From the gunbc repo worktree (rotated cred — requires netrc):

```bash
BMC_HOST=<srvN-bmc-ip> BMC_NETRC_FILE=~/.bmc-netrc gunbc run --source-root dag \
  --entry dag/gunbc/tools/bmc_read_telemetry.dag --function bmc_read_telemetry
```

Exit 0 means Systems/system + Chassis sensors GET succeeded. Factory first-contact only (unrotated):

```bash
BMC_HOST=<srvN-bmc-ip> gunbc run --source-root dag \
  --entry dag/gunbc/tools/bmc_first_contact.dag --function bmc_first_contact
```

## 5. Power control (limits) — read before write

`PowerLimitWatts` on a `PowerControl` member is the Redfish setpoint for capping draw. **Changing limits is a management write (PATCH)** and is **not** exercised by the diagnostic read tools above.

Before setting limits:

1. Read current `PowerConsumedWatts` and any existing `PowerLimitWatts`.
2. Coordinate with operator policy in ctrl (workflow tier) — limit changes affect CI thermal headroom.

OpenBMC PATCH shape (operator reference only; policy lives in ctrl):

```bash
# Example — adjust MemberId and watts per live PowerControl list.
curl -sk -u 'root:<password>' -X PATCH \
  -H 'Content-Type: application/json' \
  -d '{"PowerControl": [{"MemberId": "0", "PowerLimitWatts": 400}]}' \
  "https://${BMC_HOST}/redfish/v1/Chassis/Self/Power"
```

## 6. srv1 / srv2 checklist

For each host:

- [ ] BMC IP documented in ctrl `operator_fleet` (management NIC).
- [ ] Redfish `/redfish/v1/` reachable from operator network.
- [ ] Auth works (factory or rotated secret).
- [ ] `Systems/system` reports expected `ProcessorSummary` and `MemorySummary` (128 GiB = 8×16 GiB DIMMs on current fleet).
- [ ] `Chassis/{chassis_id}/Sensors` reports `power_PWR_*` readings when host is on (ALTRAD8UD Chassis Power/Thermal are empty — see §3).
- [ ] Factory password rotated; ctrl holds live BMC credential.
- [ ] `bmc_read_telemetry` exits 0 from gunbc worktree.

## 7. Security notes

- BMC HTTPS uses a self-signed certificate — `-k` / `curl -k` is expected for OOB management.
- Do not commit BMC IPs, serial numbers, or passwords to gunbc (public repo). ctrl owns deployment facts.
- IPMI (RMCP+) is a **separate** cited interface (`std.bmc.Ipmi`); Redfish is preferred for inventory and power telemetry.

## Related model files

- `dag/extdeps/bmc/types.dag` — Redfish resource shapes
- `dag/extdeps/bmc/redfish.dag` — projection / grounding seam
- `dag/extdeps/bmc/http.dag` — curl transport operations
- `dag/test/claim/bmc_redfish_grounding_witness_test.dag` — projection witness
