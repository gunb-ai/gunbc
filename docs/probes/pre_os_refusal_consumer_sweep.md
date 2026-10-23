# Pre-OS refusal consumer sweep

Scope: the assessment/replay and inherited MegaRAC observation changes in #11211,
including their first production consumers. Authority: DESIGN §3/§5 and
`gunbc.recurring_failure_mode.bool_projection_over_a_typed_carrier`. This is a
review map, not proof of structural enforcement or a live collection receipt.

| Producer → first consumer | Disposition |
| --- | --- |
| `read_verified_memory_capture` → `pre_os_read_observation`, archive replay | Read/hash failure remains `ObservationRefused`; timeline association and production diagnostic absence carry it to refused assessment. Archived readings remain typed results. |
| `recording_owner` / `subject_agreement` → `pre_os_bound_envelope` | Every negative arm retains its original ownership or disagreement, with owner/receiver location. |
| `commit_attests_chunk` → `pre_os_committed_envelope` | Missing, mismatched and incomparable binding retain their carriers. `pre_os_composable_payload` passes the refusal to the timeline's console admission. |
| `continuity_covers` → `pre_os_chunk_continuity` → console/SEL | Unknown, short and incomparable coverage remain distinct. The timeline gates assessments and cycle extraction on current-chunk admission (review 66180 repair). |
| `source_byte_fact` → `pre_os_diagnostic_absence` | Every gap and silence arm refuses with the original source carrier; only observed bytes proceed to terminal and inventory checks. |
| `pre_os_console_span` → timeline | A join refusal is separate from current-chunk admission. Valid standalone chunks may assess; invalid current chunks cannot. Original composition cause remains in the row. |
| Firmware parser → `pre_os_terminal_from_console` / training assessment | Unknown statements and opaque error-code-only reports refuse; their statement/count survives. `NoTerminalStated` alone is indeterminate. The reduced `PreOsMilestones.training` field was removed: assessment consumes `firmware`. |
| Boot observation → `PreOsMilestones` → boot assessment | Boot remains `ObservationAttempt`; refusal no longer becomes `BootOutcomeUnobserved`. Observed resets/stalls diverge; reached userspace satisfies; genuinely unobserved boot is indeterminate. No marker in assessed nonempty console is unobserved; a started but incomplete marker or zero bytes refuses. |
| `sel_rendered_record` → `sel_read` → summary/delta | Snapshot parsing now retains the original parse cause and offending row. The sealed watermark carries parsed events; summary and delta no longer reparse and drop refusal arms. Duplicate/non-increasing IDs refuse. |
| Previous SEL cursor refusal → `pre_os_ingest_sel` | Recovery adds a location to the original refusal, instead of replacing its cause with a baseline label. |
| Generation equality → composition/SEL delta | A negative comparison becomes `PreOsSourceChanged` carrying both complete envelopes, including unavailable-generation reasons; no source continuity is admitted. |
| `pre_os_inventory_observation` → configuration/absence | Coverage and inventory failures stay refused; the underlying inventory membership predicate describes a relation, not a typed refusal. Wall-clock interval coverage is explicitly refused by this consumer. |
| `retention_exposure` → timeline/view | The typed exposure, including downstream rejection and local-commit failure, remains in the row and renderer. Independent local assessment does not claim forwarding succeeded. |
| MegaRAC configuration/session/readback standings → attach outcome/renderers | Unreadable configurations stop attachment, attach outcomes remain distinct, and session release is reported separately. The image-list second parse cannot reach its empty fallback after the preceding identical pure parse has refused; this is control-flow redundancy, not an admitted failure path. |
| `sol_host_fact_from_collector` → existing witnesses | No production consumer in this slice. Its legacy acknowledgement/coverage standing must not be cited as evidence of continuous collection. The live path consumes acquisition carriers instead. |

Remaining reductions inspected: occurrence equality is an identity relation;
conflict predicates become located refusals carrying the envelope. Removing a
failed cursor from the fold's active source index does not remove its timeline
row. `new_records`/`cycles` empty lists are projections beside retained typed
continuity/assessment refusals; no production absence claim consumes these lists
alone. Source-line string/optional parsers are checked at their enclosing parser
boundary; they cannot admit malformed SEL snapshots. These are scoped consumer
facts, not permission for a future consumer to interpret an empty list as silence.

The sweep adds controls for refused versus absent boot observations, SEL parse
cause/row retention, prior-cursor cause retention, and opaque error-code refusal.
Existing controls cover carrier rejection, independent boot, valid standalone
chunks, recovery, and archive replay. Current-head execution and re-review must
validate the changes; authoring these controls is not an executed RED, and this
note claims no fresh mutation campaign. Persistence remains mitigatable, and live
acquisition plus supervision-to-envelope realization remain outside this slice.
