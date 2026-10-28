srv14 first contact and inventory (2026-10-28, session snappy-bear-100)
Unit: ASRock Rack ALTRAD8UD-1L2T, BMC 192.168.1.250 (router static reservation bmc-altrad8ud-1l2t-6,
BMC MAC 9c:6b:00:e7:c9:fa, read by the operator). The same unit as
artifacts/bmc/altrad8ud-1l2t-6-inband-recovery-2026-10-16/.

PROVENANCE
- Unauthenticated Redfish ServiceRoot: gunbc.tools.bmc_intake_first_contact (HTTP 200).
- Published credential root/0penBmc before rotation: 403 Base.1.16.0.PasswordChangeRequired.
- Rotation: gunbc.tools.bmc_onboard srv14_converge_credential. It is the legacy route, run once as an
  operator-approved exception. The secret bmc-srv14-admin was created in gunbai-secrets, version 1 added and
  read back, the BMC password was set, and GetSystem re-authenticated with the new credential. After that the
  published credential answered 401.
- redfish_*.json: authenticated GETs with the rotated credential, bodies as returned. This firmware serves no
  Systems/system/EthernetInterfaces and no SecureBoot resource (ResourceNotFound, kept as returned).
- uefi_shell.txt: an IPMI SOL session (ipmitool 1.8.19 on srv1, lanplus cipher suite 17) to a host sitting at
  the UEFI Shell. Commands: ifconfig -l, map, pci, dmpstore SecureBoot, dmpstore SetupMode. ANSI escapes and
  CR were stripped; nothing else was changed.
- uefi_devpaths.txt: the same session route; map -r -t net, dh -p DevicePath.

READINGS (each is restated from the files above, which govern)
- Board serial BR80S6102300077; BIOS 3.10; BMC 3.22.00; 1 socket, 80 cores; power On.
- Memory: 4 x 64 GiB Samsung M393A8K40B22-CWD at 2666, in dimm0/1/4/5 (256 GiB); dimm2/3/6/7 absent.
- Host NICs (UEFI ifconfig -l): eth1 9C:6B:00:E7:CA:9C media present (the only cabled port); eth2 ...CA:9A and
  eth3 ...CA:9B disconnected; eth0 E2:FB:38:27:6D:91 at 169.254.0.18 is the host-to-BMC USB link.
  By device path, ...CA:9A/...CA:9B are functions 0/1 of one device (the dual-port X550, 8086:1563) and
  ...CA:9C is function 0 of a single-function device (the I210, 8086:1533). That is an INFERENCE from the
  function pattern: the shell truncated the bus segment of each path.
- Storage: one NVMe, 144D:A802, EUI 53-92-00-61-6B-38-25-00, carrying a GPT with 4 partitions (a prior OS).
  The BMC's Storage collection lists nothing.
- Secure Boot: the global-variable namespace has no SecureBoot and no SetupMode variable.
