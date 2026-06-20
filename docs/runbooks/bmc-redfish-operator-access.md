# BMC Redfish operator access runbook (srv1 / srv2)

Enable and verify out-of-band Redfish telemetry on the self-hosted CI fleet hosts. This runbook is operator-private access guidance; concrete host IPs and rotated credentials live in **ctrl** (`plans.fabric.operator_fleet`), not in this repo.

## Prerequisites

- Management-network reachability to each host's **dedicated BMC NIC** (not the host OS IP).
- ASRock Rack ALTRAD8UD-1L2T boards ship **OpenBMC** (Redfish + IPMI). Factory login is cited in `dsl/extdeps/bmc/openbmc.dag` (`root` / `0penBmc`) until first-contact rotation.
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

**First contact (factory default):**

```bash
curl -sk -u 'root:0penBmc' "https://${BMC_HOST}/redfish/v1/Systems/system"
```

**After rotation:** use the credential from ctrl's secret store — never pass rotated passwords on the shell argv in shared logs; prefer `~/.netrc` or the gunbc tool with env-injected secrets.

Auth failure with factory credentials usually means rotation already happened (expected on production hosts).

## 3. Read telemetry resources (inventory + power + thermal)

| Resource | Redfish path | Modeled shape (`dsl/extdeps/bmc/types.dag`) |
|----------|--------------|-----------------------------------------------|
| System inventory (CPU model, core count, RAM) | `/redfish/v1/Systems/system` | `RedfishSystemInventory` |
| Power (wall draw, limits) | `/redfish/v1/Chassis/Self/Power` | `RedfishPowerSubsystem` / `RedfishPowerControl` |
| Thermal | `/redfish/v1/Chassis/Self/Thermal` | `RedfishThermalSubsystem` |
| Sensors (breadth) | `/redfish/v1/Chassis/Self/Sensors` | diagnostic layer (`redfish.Http.GetSensors`) |

Example reads:

```bash
curl -sk -u 'root:0penBmc' "https://${BMC_HOST}/redfish/v1/Systems/system" \
  | jq '{Model, PowerState, ProcessorSummary, MemorySummary}'

curl -sk -u 'root:0penBmc' "https://${BMC_HOST}/redfish/v1/Chassis/Self/Power" \
  | jq '.PowerControl[] | {Name, PowerConsumedWatts, PowerLimitWatts}'

curl -sk -u 'root:0penBmc' "https://${BMC_HOST}/redfish/v1/Chassis/Self/Thermal" \
  | jq '.Temperatures[] | {Name, ReadingCelsius}'
```

**Grounding mapping:** `ProcessorSummary.Model` + `ProcessorSummary.CoreCount` → CPU core observations (`Count` is socket count, not cores); `MemorySummary.TotalSystemMemoryGiB` → total RAM (`ByteSize`); `PowerControl.PowerConsumedWatts` → Energy axis (`Watt` / `HardwareAxes.power` in `product.hardware_selection`) when reported, otherwise absent.

## 4. gunbc read-only poll (modeled transport)

From the gunbc repo worktree:

```bash
BMC_HOST=<srvN-bmc-ip> gunbc run --source-root dsl \
  --entry dsl/gunbc/tools/bmc_read_telemetry.dag --function bmc_read_telemetry
```

Exit 0 means all three GETs (System, Power, Thermal) succeeded. First-contact only:

```bash
BMC_HOST=<srvN-bmc-ip> gunbc run --source-root dsl \
  --entry dsl/gunbc/tools/bmc_first_contact.dag --function bmc_first_contact
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
- [ ] `Chassis/Self/Power` reports `PowerConsumedWatts` when host is on.
- [ ] Factory password rotated; ctrl holds live BMC credential.
- [ ] `bmc_read_telemetry` exits 0 from gunbc worktree.

## 7. Security notes

- BMC HTTPS uses a self-signed certificate — `-k` / `curl -k` is expected for OOB management.
- Do not commit BMC IPs, serial numbers, or passwords to gunbc (public repo). ctrl owns deployment facts.
- IPMI (RMCP+) is a **separate** cited interface (`std.bmc.Ipmi`); Redfish is preferred for inventory and power telemetry.

## Related model files

- `dsl/extdeps/bmc/types.dag` — Redfish resource shapes
- `dsl/extdeps/bmc/redfish.dag` — projection / grounding seam
- `dsl/extdeps/diagnostic/redfish.dag` — curl transport operations
- `dsl/test/claim/bmc_redfish_grounding_witness_test.dag` — projection witness
