In-band IPMI (SSIF) BMC credential recovery — bmc-altrad8ud-1l2t-6 (192.168.1.250)
Executed by session royal-seal-46 on 2026-10-16. LAN reached via jump host srv2.

PROVENANCE — read before citing. Two classes of evidence here:

CAPTURED (raw bytes in this directory, checksummed in SHA256SUMS):
  02_inband_channel.txt   steps 1 & 2 + the factory-password refusal (step 3).
                          Fresh re-capture taken 2026-10-16 ~06:5xZ, AFTER the reset, against the
                          SAME rescue host (.251) and the BMC in its post-reset factory state.
                          Reproducible read-only facts: rescue OS reachable over SSH (OpenSSH 9.6p1,
                          hostname ubuntu-server), /dev/ipmi0 + ipmi_ssif/ipmi_devintf/ipmi_msghandler
                          loaded, ipmitool 1.8.19, mc info (ASRock, Firmware Revision 3.22), channel 1
                          = 802.3 LAN, user list 1 => ID 1 root ADMINISTRATOR, lan print => .250.
                          Passwordless in-band system interface (SSIF — ipmi_ssif loaded, ipmi_si
                          absent): every ipmitool call uses /dev/ipmi0 with NO -I lanplus and
                          NO credential on the command line.
  04_post_reset_readback.txt   step 4 terminal state. GET root/0penBmc => HTTP 403 with body
                          MessageId Base.1.16.0.PasswordChangeRequired. This is the
                          factory-credential-active first-contact state that re-enters bmc_onboard.
                          NAMED .txt, NOT .json: the file is an annotated capture (### request/status
                          lines around the body), not a JSON document, so nothing should try to parse
                          it as one. WHAT IT DOES NOT HOLD: no firmware/version context. The 3.22
                          attribution comes from mc info in 02_inband_channel.txt (Firmware Revision
                          3.22); the authenticated GET that reported FirmwareVersion 3.22.00 had its
                          body unsaved (see PRIVILEGED WRITE 1). Read no version from this file.

PROVIDER: the rescue OS arrived by PHYSICAL USB medium (operator inserted a prepared Ubuntu live
  USB; it DHCP'd to .251). Netboot was NOT used; mark the netboot provider UNEXECUTED.

OBSERVED IN SESSION, NOT BYTE-CAPTURED (do not model as captured; the terminal result IS captured):
  - The factory-password refusal is an IPMI refusal ("Invalid data field in request", exit 1), NOT a
    Redfish set — there is no Redfish MessageId for it. Both the 16- and 20-byte forms were rejected.
    (This IS re-captured in 02_inband_channel.txt, taken post-reset; it reproduces identically.)
  - PRIVILEGED WRITE 1 (temp admin password set over the in-band SSIF channel, 20-char, then Redfish GET => HTTP 200):
    executed in the original run ~06:40Z; the temp password was wiped by the reset, so this step is
    NOT reproducible and was not saved as bytes. Observed exit: "Set User Password command successful
    (user 1)"; Redfish verify HTTP 200, FirmwareVersion 3.22.00.
  - PRIVILEGED WRITE 2 (Redfish Manager.ResetToDefaults): executed ~06:45Z. The action advertised
    ResetType@Redfish.AllowableValues=["ResetAll"] but the firmware required the parameter spelled
    ResetToDefaultsType; POST {"ResetType":"ResetAll"} => HTTP 400 (ResetToDefaultsType required),
    POST {"ResetToDefaultsType":"ResetAll"} => HTTP 200. Response bodies were not saved. The RESULT
    is independently captured in 04 (BMC down ~2.5 min, returned at .250, factory 403).

The temp password is intentionally not recorded anywhere.
