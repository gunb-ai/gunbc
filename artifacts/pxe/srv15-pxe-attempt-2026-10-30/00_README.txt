srv15 (bobo) network-boot attempts against the site PXE edge on srv4, 2026-10-30 (read 2026-11-10, session smart-crane-730)

WHAT HAPPENED: srv15's arm64 UEFI PXE client (NIC 9c:6b:00:e7:ca:63, lease 192.168.1.204) booted twice from the edge,
at 13:21Z and 13:26Z. Each time the edge's proxyDHCP answered with bootaa64.efi (signed shim), shim loaded
grubaa64.efi (Ubuntu's signed network GRUB), and GRUB fetched grub/arm64-efi/*.lst (not found) and then
grub/grub.cfg. It never requested grub/grub.cfg-01-9c-6b-00-e7-ca-63, the per-MAC file that held its install
binding. grub/grub.cfg was the edge's fallback (`exit`), so the host returned to its next boot device and never
reached the installer. The HTTP unit served no ISO or seed to it.

WHY: the signed GRUB's embedded memdisk configuration loads $prefix/grub.cfg directly ("normal (memdisk)/grub.cfg"
and "%s/grub.cfg" in its strings), so the GNU GRUB manual's per-MAC search does not run for this build.

FILES (captured bytes, checksummed in SHA256SUMS)
- edge_dnsmasq_journal.txt: `journalctl -u gunbc-site-pxe-edge -o short-iso-precise` on srv4 for 13:21:00-13:27:30Z
  on 2026-10-30, minus the periodic "available DHCP subnet" lines.
- edge_grub_tree_2026-11-10.txt: the edge's grub/ directory listing and every grub.cfg* body as served on 2026-11-10,
  the sha256 of grubaa64.efi and bootaa64.efi, and the two embedded-configuration strings from grubaa64.efi.

CONSUMER: gunbc.fleet.site_pxe_edge_converge site_pxe_edge_fallback_grub_cfg, which now loads the client's own
grub/install/<mac>.cfg from GRUB's net_default_mac instead of relying on the per-MAC search.
