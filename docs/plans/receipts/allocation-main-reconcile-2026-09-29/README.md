# Main reconciliation qualification

Source `2ad001dc88dcd554e0de9a2dd2853ab3f77124b8` includes main at
`c39426540cef73eff9d734feb489168e292d089f`. The branch-local release build
passed under 6 GiB/no swap. Full preview serve reached readiness under
12 GiB/no swap; the receipt records duration, peak and zero OOM events.

Eighteen focused controls passed under 6 GiB/no swap. The authenticated
Unix socket test passed cookie/header transport, append/read, exact above-cap
refusal, wrong-key and unrostered-peer refusals, and reread after restart.
The final nonzero transport-refusal control is expected: unavailable transport
must not count as the exact protocol refusal. All copied closure modules match
this source revision. No fixture credentials are included.

The canonical generator OOM-killed at the local 6 GiB cap. CI 36588368366 completed its floor lane with a stale debt-roster refusal for
`roadmap_event_cli.dag#step`. The source no longer carries that pair; the
follow-up retires that row as `ImportsFixed`. The generated lane is still
running. These receipts do not establish integrated-floor qualification.
No production installation, reservation, or VM acceptance occurred.

Follow-up `6b213577c` includes the roster repair. Its branch-local witness executor
build passed in 4m48s under 6 GiB/no swap. The full floor run terminated with exit
143 and scope `Result=oom-kill`, before a measurement receipt or verdict. This
is not a passing floor result; no repeated same-budget attempt is planned.
