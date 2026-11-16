Read-only observation of srv4's pinned PXE edge, transcribed from a CI run.

Who ran it: still-fox-404 dispatched fleet-converge.yml at ref main with host=srv4,
mode=site_pxe_edge_observe. Workflow run 43789720870,
https://github.com/gunb-ai/gunbc/actions/runs/43789720870 , observe step of the
fleet-converge job, completed 2026-11-16T02:17:25Z. The entry the step ran:
  gunbc run --entry dag/gunbc/fleet/site_pxe_edge_converge.dag
        --function site_pxe_edge_observe_ci_wet
observe_wire_lines.txt is the step's OWN wire block, copied verbatim (timestamps kept),
plus the two ssh legs that exited 1 with the command and stderr only.

What this is evidence of:
- srv4 was reached over the fleet ssh roster with a model-materialized known-hosts anchor
  (rows for srv1, srv2, srv3, srv4, srv14; provenance scanned-first-contact).
- The converge's stale-install set (fn site_pxe_edge_stale_install_paths in that module: every
  file under the owned install roots that no model-rendered member produces) FOUND five files,
  two of them under a seed directory named for the MAC ac-1f-6b-e6-75-eb -- the install NIC of the
  H11SSL-i unit this branch models. Outcome in this mode: StaleLeftObserveOnly.
- /srv/gunbc-site-pxe-edge/http/seed/ac-1f-6b-e6-75-eb/user-data and .../meta-data: the hand-staged
  seed the unit was installed with. In the model's eyes those two paths are stale, i.e. an APPLY
  run would `rm -f` them (fn site_pxe_edge_remove_stale).
- srv4's live /etc/gunbc-site-pxe-edge.dnsmasq.conf DIFFERS from what the model renders
  (file ... before=differs outcome=not-applied). The hand-added dnsmasq line is expected to be in
  that difference, though this read reports only "differs", not the line-level delta.
- srv4's dnsmasq log shows 2 PXE/DHCP request lines from MAC ac:1f:6b:e6:75:eb in the 24h window
  2026-11-15T02:17:20Z..2026-11-16T02:17:20Z. The run attaches its own caveat to that line
  ("dnsmasq saw these DHCP senders, not a host attribution") and an unmet obligation: the dnsmasq
  version installed on srv4 is UNVERIFIED, so the log layout being parsed is dnsmasq v2.90's, not
  proven to be srv4's.
- The run refused before writing anything, and TWO refusals are visible in the block. (a) The entry's own
  verdict line reads terminal=REFUSED, with chain Aarch64 withheld because member-unobservable:
  sha256sum of /srv/gunbc-site-pxe-edge/tftp/snponly.efi reports "No such file or directory", so the pinned
  Aarch64 chainloader cannot be digested and the chain is withheld. (b) The CI step's shell exited 1 on
  `cmp -s - /etc/gunbc-site-pxe-edge.dnsmasq.conf`, the step that compares srv4's live dnsmasq config
  against what the model renders; exit 1 means the two differ. Nothing in the block attributes (b) to (a)
  or the reverse.

What this is NOT evidence of:
- Not that anything on srv4 was changed. Nothing was written; the rollback commands in the block are
  STAGED plans the observe prints, never executed (placement=not-attempted observe-only).
- Not a host attribution: the request count does not prove which machine asked, only which MAC sent.
- Not the content of the dnsmasq difference, and not the hand-staged grub install ENTRY: that entry
  for ac:1f:6b:e6:75:eb is not among the five stale paths, which leaves open where it lives and
  whether the stale fold owns it at all.
- Not the live-evidence digest's meaning: live-evidence readings=0 with
  digest=sha256:cb94e7f0faf01647cc17ba88d432f2223403affa892bf26080415140d0b94f89 is what the run
  itself printed; the artifact the workflow uploaded (site-pxe-edge-live-receipt.zip, 206 bytes)
  carries only the header line "site-pxe-edge-live-receipt v1" -- no readings, because the observe
  refused before taking any.

The lines were copied by still-fox-404 from the run's log via the GitHub API; the transcription was
not itself captured by this repository.
