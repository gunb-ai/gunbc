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
- The entry's own verdict line reads terminal=REFUSED. Reading the code that prints it (this lane, 2026-11-16,
  main revision 22649d22d2fd15332770e4c344fecc8c388b89f6): the line comes from site_pxe_edge_overall_succeeded
  -> site_pxe_edge_run_succeeded, and in the SitePxeEdgeObserveOnly arm a run fails if any WITHHELD chain has a
  cause that site_pxe_edge_chain_cause_is_unobserved calls unobserved -- MemberUnobservable is true there, while
  ChainUnpinned and MemberDigestMismatch are false. This run's Aarch64 chain is withheld with
  member-unobservable (the digest leg could not run: the file is absent), so the OBSERVE run refuses. The same
  fn's SitePxeEdgeApply arm never consults run.chains at all, so this is a fact about observe mode, not about
  an apply being blocked.
- The `cmp -s - /etc/gunbc-site-pxe-edge.dnsmasq.conf (exit=1)` leg is NOT a failure of the run. It is
  observe_site_pxe_edge_file's own far-side comparison (typed_remote_file_compare_argv: the model pipes its
  rendered bytes on stdin and asks the far side to cmp), and exit 1 is that leg ANSWERING: the live file
  differs. The standing it decodes to is EdgeFileDiffers, and the ObserveOnly arm of site_pxe_edge_run_succeeded
  fails only on EdgeFileUnobservable, not on EdgeFileDiffers. So a differing live config is normal observe
  output; the nonzero exit of the CI step is the entry's ProcessExit from the verdict above, not from this leg.

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
