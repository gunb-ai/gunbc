# srv3 OS installation

**Goal:** install through the modeled media and boot workflow, with independent evidence of the resulting OS. Preparation is not installation: `gunbc.srv3.srv3_os_install_actuate_scope` owns that boundary.

**Current gate:** read `gunbc.os_install_mechanism.srv3_install_mechanism_solution`, the seeded-plan standing and `gunbc.os_install_actuator_selection` first. The source currently refuses the mechanism/seeded plan; historical 2.07.00 WebSocket observations do not qualify the later firmware. An unsatisfiable selection supplies no default actuator. The source-declared boot grant is expired; its dissolution requires an out-of-band attempt-bound grant channel, not a re-dated constant.

**Method, once those gates are discharged:** run on the selected actuator, using `gunbc run --source-root dag --entry <module-file> --function <entry>`:

| Order | Module under `dag/gunbc/srv3/` | Entry |
|---|---|---|
| 1 | `srv3_os_install_actuator_toolchain_ensure.dag` | `srv3_os_install_actuator_toolchain_ensure` |
| 2 | `srv3_install_media_fetch.dag` | `srv3_install_media_fetch` |
| 3 | `srv3_seeded_install_media.dag` | `srv3_seeded_install_media_toolchain_ensure`, then `srv3_seeded_install_media_remaster` |
| 4 | `srv3_os_install_actuate.dag` | `srv3_bmcweb_session_login`, then `srv3_nbd_proxy_serve` |
| 5 | `srv3_boot_once_cd.dag` | Authorized workflow only; default `srv3_boot_once_cd` refuses without approval |

The serve route resolves the seeded image through its derivation record and rechecks its bytes. Missing, unread or mismatching media refuses; never substitute the stock ISO or legacy network-seed plan. Login uses the observed credential phase and private temporary files; do not publish token files or credential-bearing failure bodies. Serving uses persistent transient units: observe their readiness and stop them after the attempt. Boot authorization must cover both override and reset, the exact attempt/intent and an unexpired readable clock. The override is `Once`, not continuous.

## Install debug diagnostic fold

**Validation:** `gunbc.srv3.srv3_os_install_diagnostic.diagnose_srv3_install` consumes serve, Redfish, WebSocket, virtual-media, SOL and router observations. Read the modeled hang thresholds. Endpoint presence, a consumed override, accepted reset or frozen KVM cannot establish installation; unknown serve state remains inconclusive. The dynamic host lease must be distinguished from the BMC reservation, and post-install witnesses remain necessary. The old standalone diagnostic-checklist entry is absent from this checkout.

Record attempt, operator/grant, selected actuator, seeded image identity and digest, serve/boot outcomes, console receipt, router MAC/address/hostname and final modeled verdict in the sign-off thread. Preserve refusals and partial observations. This cleanup runs no hardware operation and supplies no install receipt.
