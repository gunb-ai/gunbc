# iPXE ARM64 UEFI SNP-only chainloader — build receipt

Pinned by `gunbc.fleet.site_pxe_edge_converge` `site_pxe_edge_chainloader_pin`.

- upstream: https://github.com/ipxe/ipxe, `git clone --depth=1`, revision `c51ca76a6e56c5627aea6dc32da4c18d5e6cf394` (HEAD on 2026-10-17)
- build target: `bin-arm64-efi/snponly.efi` (extdeps.firmware.ipxe `ipxe_arm64_efi_snponly`, Architecture Aarch64)
- command (cwd `src/`): `make -j4 ARCH=arm64 CROSS_COMPILE=aarch64-linux-gnu- bin-arm64-efi/snponly.efi`
- toolchain: aarch64-linux-gnu-gcc (Debian 14.2.0-19) 14.2.0; GNU Make 4.4.1; GNU ld (GNU Binutils for Debian) 2.44
- size: 337920 bytes
- sha256: `18b1cbd27420fb4961e7a58d266e98a66cba49b3047e6abfc2c697733aaa7e05`  (`sha256sum snponly-arm64.efi`)
- artifact: `artifacts/ipxe/snponly-arm64.efi` (installed on the TFTP root as `snponly.efi`)
- Secure Boot: UNOBSERVED. Unsigned self-build; no Altra board's Secure Boot state read. With Secure Boot enabled and no enrolled
  certificate the firmware will refuse to load it; with it disabled it should load. Settled by an enrolled cert or an observed boot attempt.
- Reproducibility: iPXE embeds a build timestamp/version, so a rebuild may differ in bytes; the committed file is the authority for the digest.
