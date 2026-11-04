srv15 (bobo) BMC credential recovery and host inventory (2026-10-29, session snappy-bear-100)
Unit: ASRock Rack ALTRAD8UD-1L2T, BMC 192.168.1.248 (router reservation bmc-altrad8ud-1l2t-7,
BMC MAC 9c:6b:00:e7:c9:e7; the BMC announces itself over mDNS as altrad8ud-1l2t-4). Board serial BR80S6102300049.

WHY A RECOVERY: the BMC's root credential was unknown and recorded nowhere. The published credential
root/0penBmc answered HTTP 401 on Redfish (not 403 PasswordChangeRequired, which a factory-state BMC gives),
so it had been changed by hand without a record. No secret for this unit existed in gunbai-secrets.

ROUTE (operator-approved in conversation, 2026-10-29), the in-band SSIF route first executed on srv14's unit
(artifacts/bmc/altrad8ud-1l2t-6-inband-recovery-2026-10-16/), without its temporary password or factory reset:
1. The operator power-cycled the unit with the rescue USB inserted (Lexar, Ubuntu Server 24.04.5 arm64, NoCloud
   seed on the stick). No monitor or KVM: the host booted the stick by itself and took a DHCP lease at
   192.168.1.204 from host NIC 9c:6b:00:e7:ca:63, hostname ubuntu-server, observed by tcpdump on srv1.
2. In-band, read only: ipmitool 1.8.19 over /dev/ipmi0 (ipmi_ssif). mc info Firmware Revision 3.22; user 1 = root,
   ADMINISTRATOR; lan print 1 = 192.168.1.248, DHCP, MAC 9c:6b:00:e7:c9:e7.
3. The secret bmc-srv15-admin was created in gunbai-secrets (it did not exist; the run refused if it did), and
   version 1 was added: 16 characters, at least one each of upper, lower, digit and one of _.@#%-, the policy of
   extdeps.entropy Urandom.ReadPassword. It was read back and compared before use.
4. PRIVILEGED WRITE: `ipmitool user set password 1 <read-back value> 20` over the in-band channel, the value on
   stdin of the SSH session, never on a command line outside the rescue host. Exit 0.
5. READBACK: Redfish GET /redfish/v1/Managers/bmc with root and the stored secret: HTTP 200, FirmwareVersion
   3.22.00. With root/0penBmc: HTTP 401.
The password is not recorded anywhere but the secret.

FILES (captured bytes, checksummed in SHA256SUMS)
- host_identity.txt: from the rescue OS: dmidecode system/baseboard/bios/memory, lscpu, free, /sys/class/net
  (mac, carrier, driver, pci), lsblk, lspci, efivars names, ipmitool mc info and lan print. One locale warning line
  emitted by the remote shell was removed; nothing else changed.
- secureboot_disk_bootorder.txt: efivars SecureBoot and SetupMode as od bytes (4 attribute bytes, then the value),
  sfdisk -l of the NVMe. efibootmgr was not installed on the rescue OS (its line says so).
- redfish_*.json: authenticated GETs with the rotated credential, bodies as returned. This firmware serves no
  Systems/system/EthernetInterfaces and no Systems/system/SecureBoot (ResourceNotFound, kept as returned).

READINGS (restated from the files, which govern)
- BIOS 4.01 (03/11/2026), BMC 3.22.00. Ampere Neoverse-N1, 1 socket, 80 cores.
- Memory: 4 x 64 GiB Samsung M393A8G40D40-CRB at 2133 MT/s in DDR4_A1/B1/E1/F1 (256 GiB); C1/D1/G1/H1 empty.
- Host NICs: enP2p4s0 9c:6b:00:e7:ca:63 igb (I210, 0002:04:00.0) carrier=1, the only cabled port;
  enP3p3s0f0/f1 9c:6b:00:e7:ca:61/62 ixgbe (X550) carrier=0. The cdc_ether interface is the host-to-BMC USB link.
- Disk: one WD Green SN350 1TB NVMe, no partition table (blank).
- Secure Boot: SecureBoot = 0 and SetupMode = 1 (no platform key enrolled). srv14's BIOS 3.10 exposes neither
  variable; this unit's BIOS 4.01 does.

NETWORK BOOT ENABLED (operator go, 2026-10-29)
- efibootmgr.txt: the UEFI boot entries read from the rescue OS before the change: BootOrder 0002,0001, only
  "UEFI OS" (the USB stick) and the built-in EFI Shell, no network entry.
- The firmware's BIOS settings are not writable over Redfish on this build (Systems/system/Bios has no Attributes
  and no Settings resource, only Bios.ResetBios). The change was made in Aptio Setup over the BMC host console
  (SSH to the BMC on port 2200, driven from srv1): a Redfish one-time BootSourceOverrideTarget BiosSetup, then
  Advanced > Network Stack Configuration. Read there: Network Stack [Enabled], IPv4 PXE Support [Disabled],
  IPv4/IPv6 HTTP and IPv6 PXE [Disabled]. Changed: IPv4 PXE Support to [Enabled], nothing else. Saved with
  Exit > Save Changes and Exit (F10 does not cross the serial console; it arrives as Escape).
- pxe_discover_after_bios_change.txt: tcpdump on srv1 during the next boot (one-time Redfish override Pxe):
  a DHCPDISCOVER from 9c:6b:00:e7:ca:63 with Vendor-Class "PXEClient:Arch:00011:UNDI:003016" (arm64 UEFI). The
  console printed "Start PXE over IPv4 on MAC: 9C-6B-00-E7-CA-63". The site edge did not yet know this MAC, so
  nothing answered; that is the expected state before the edge's srv15 request lands.
