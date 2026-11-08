# SOURCE TRACE — head-of-list producers on the wc census lane at f08a6ae4
Status: SOURCE READING ONLY. Nothing here was executed. No attribution of the observed r=X to any row.

## 0. Lane mechanics (why the head can be anything)
- wc_verdict_of (v2/test/claim/wildcard_arm_resolve.dag:88-90): WcRefused.reason = first.head.reason VERBATIM; at_atom Absent unless head.at is NodeLocus on an Atom node (grammar.dag:1422-1428 Absent for ComputationNode/Conj/Disj/Arrow/Instantiation); sole_chain = (rest == []). No reason rewrite anywhere on this path (native_authored_first_cause_diagnostic/native_diagnostics_report_first_cause fire on the PREPARE path only).
- Census entry: native_test_resolve_module_walk (source_root_eval.dag:1151) -> resolve_walk_in_context (name_resolve.dag:1815). ctx=OK = Accepted arm -> per-module resolution_context_namespace; its Rejected gives ResolveWalkRefused{first: r, rest: [], obs: namespace_refused} — s=T is STRUCTURAL there.
- rejected_with_pending (std/diagnostic.dag:273) PREPENDS pending advisories before the chain head; resolve.dag:2863-2869 merges sibling advisories into the first chain only. resolve.dag:1543-44 comment: "A consumer that read the HEAD as the cause read an advisory ... such a reading was never the contract." wc_verdict_of is that consumer.
- Vocabulary sweep: reason: caret literals in resolve.dag+name_resolve.dag = 61: 36 are the lane-37 (+1 lane name appears only via the ResolveRefusalReason union map, resolve.dag:1393-1406) + 25 OUTSIDE the 37 (22 real Diagnostic reasons + 3 observation-only fields: resolve_observation_{context,namespace,bind_scope_prerequisite}_refused — ObservationIncomplete reasons, never head.reason).
- a=- is weakly discriminating: any non-Atom node_locus or any InvariantPortLocus/DeclarationLocus/Textual head gives Absent.
- resolve_diagnostic/resolve_diagnostics (name_resolve.dag:356-368): at = InvariantPortLocus(^resolve_diagnostic_port) -> a=- guaranteed; singleton -> s=T guaranteed.

## GROUP N — namespace-admission arm (per-module; structural s=T; identical across the 4 census closures)
| # | Producer (file:fn) | reason symbol(s) | locus/shape |
|---|---|---|---|
| N1 | validate_diag PREPEND via rejected_with_pending at namespace_for_subject_produced/root (name_resolve:1132,1169,1182,1194) + resolve_walk_prefix_pending(pending: namespace_diag) (:1832) | SHARED-VALIDATE / NAMESPACE ADVISORIES — vocabulary of the accepted-side validate pass (ingest/normalize/projection advisories), module OUTSIDE resolve/name_resolve | head=first advisory; a=- if advisory locus non-Atom |
| N2 | namespace_provider_admission_conflict_diagnostic (name_resolve:1087) | ^resolve_reason_namespace_provider_admission_conflict (IN 37) | textual_whole_file_diagnostic (Textual -> a=- OK) singleton; CANNOT yield r=X (in-37) |
| N3 | validated_module_root_for_name (name_resolve:1003-1004) | ^resolve_imported_module_refused (:988 DeclarationLocus singleton -> a=- OK, s=T OK); ^resolve_module_not_found | module_not_found = DeclarationLocus singleton -> a=- OK, s=T OK |
| N4 | module_root_find (name_resolve:574-575) | ^resolve_duplicate_module_name (via resolve_diagnostics, InvariantPortLocus) | a=- OK, s=T OK |
| N5 | admit_import_exact/repeated/prefix/bare/edge/relative/dedup (name_resolve:713-953) via import_admission_diagnostics (:753) | ^resolve_import_not_visible, ^resolve_cross_tree_import_denied, ^resolve_cross_tree_import_ambiguous | port-locus singleton -> a=- OK, s=T OK |
| N6 | admit_export_symbol (name_resolve:424) | ^resolve_ambiguous_export | export family |
| N7 | qualified_name_from_module_node cluster (v2/extdeps/languages/dag.dag:8213-8620) | ^qualified_name_structure_invalid, ^qualified_name_projection_gated, ^module_root_qualified_name_production_absent | node_locus on root/spine node (non-Atom kinds -> a=-); singleton; OUTSIDE both lane files |
| N7b | pass-through of find_labeled_child/find_named_child rejections into the same cluster | ^named_child_missing, ^named_child_ambiguous (v2/std/node_query.dag:997,886,1003) | OUTSIDE both lane files |
| N8 | admit_imported_module_existence (:884-925) | re-stamps head.at/correction to the import authored locus; REASON PRESERVED (module_not_found via module_not_found_diagnostic_from_missing :1028 DeclarationLocus) | |

## GROUP W — walk arm (namespace admitted; walk's own vocabulary reaches head)
| # | Producer | reason(s) | shape |
|---|---|---|---|
| W1 | ambiguous_symbol_diagnostics (resolve.dag:1546) | HEAD = advisory ambiguous_lookup_class_diagnostic: ^resolve_ambiguous_competing_declaration / _qualified_head_shadows_absolute / _at_binding_position / _binder_vs_imported_declaration / _imported_vs_imported_declaration (OUTSIDE 37; the fatal resolve_reason_ambiguous_symbol sits LAST) | 1 chain (s=T OK); at=node_locus(lookup node) -> a=- only if non-Atom (qualified-head/binding-position nodes can be non-Atom) |
| W2 | resolve_anonymous_record_refused (resolve.dag:3447) + resolve_anonymous_map_diagnostic | 11 names ^resolve_anonymous_map_*/_record_* (OUTSIDE 37) | node_locus; a=- if non-Atom |
| W3 | import_admission_diagnostics fires per-import inside the walk too | N5's 3 names | port singleton |
| W4 | per-node 37-side fatals (unbound_symbol :1417, declarer_refused :1428, anonymous_parameter_referenced :1449, binder_duplicate_in_frame :850, binder_hides_visible_value :858, realized_declaration_mismatch/shadowed :5014-5037, where_predicate_unbound :5065, type_param_shadows_visible_name :5314, kernel_type_spelling_foreign :2233, test_code_referenced :2125, observation_incomplete :3090, malformed_tree :1409) | the 37-side literals | at=reference/declaration node: usually ATOM -> a=atom (INCONSISTENT with observed a=-); a=- only where node is non-Atom |
| W5 | pending advisories merged to first-chain head (resolve.dag:2869) | accepted-side advisory vocabulary (PARTIAL enumeration — includes declared-elsewhere/observation advisories) | head=advisory |

## GROUP C — constructed (non-caret-literal) reason values
| # | Site | Value | Lane reachability as head |
|---|---|---|---|
| C1 | resolve_refusal_reason_symbol (resolve.dag:1393-1406), used at :2084 | constructed from typed union; values are all INSIDE the 37 | yes, but never yields X |
| C2 | resolve_diagnostics(reason: r), r = ImportRowsFailed.reason = diagnostics_fatal_reason of PARSE diagnostics (v2/lens/reference_deps.dag:121, re-emitted via module_acquisition) | parse-subsystem reasons verbatim | NOT on census head lane: import-target attribution arms (source_root_eval.dag:2687-2750) carry their own target verdicts and pass f/rs through unchanged; listed for completeness |
| C3 | all other reason-parameter constructors (resolve_diagnostics/resolve_diagnostic/resolve_anonymous_record_refused/resolve_anonymous_map_diagnostic/import_admission_diagnostics/ambiguous_lookup_class_diagnostic/realized_declaration_*/test_code_referenced/resolve_type_application_diagnostic/resolve_requirement_diagnostic) | every call site in-lane passes a CARET LITERAL (enumerated; also tree-wide for resolve_diagnostic*: only variable site is C2) | literals per rows above |

## Shape fit for the observed X{r=X,a=-,s=T} (all four modules)
Simultaneously consistent with r=X AND a=- AND s=T: N1 (validate-advisory head), N3 (both names, DeclarationLocus), N4, N5, N6 (^resolve_ambiguous_export via port-locus resolve_diagnostics — verified), N7, N7b, W3 (port/declaration/non-Atom-locus singletons outside the 37); W1/W2 with non-Atom nodes. 37-side fatals (W4) fit s=T but usually require an ATOM head node (a=atom), inconsistent with the observed a=- — subject to verification of each site's actual node kind. Uniformity of the reason across arm/field/binder/body matches a per-module admission/entry failure (GROUP N) most directly; this is a shape argument, NOT an attribution — attribution is not established by this trace.

## Coverage caveats (labelled)
- 36/37 lane names verified as reason: literals in the two lane files; resolve_reason_generated_reference_declaration_absent reaches head only via C1's union path.
- W5 advisory vocabulary enumerated PARTIALLY; N2/N6 loci not individually re-read this pass.
- All rows = source reading at f08a6ae4; none executed.

## GROUP P — LOCATED unconditional prepend (post-parent item (b) addition)
- resolve.dag:1436-1441 declarer_refused_diagnostics(n, provider: ClosureProviderRefusal) = rejected_with_pending(pending: Some { diagnostics: provider.refusal }, rejected: singleton(^resolve_reason_declarer_refused @node_locus(n))) — UNCONDITIONALLY prepends the provider row's refusal chain. head = provider.refusal.head.reason (NON-37; vocabulary of ModuleUnavailable/DeclarationRefused rows built at resolve.dag:152-180 from the shared context's closure declarations — acquisition/ingest/declaration-read subsystems), head.at = provider chain's own at (can be non-node -> a=-). Fatal declarer_refused (in-37) sits LAST. Call sites :2048, :2077, :2160, :2339. Design comment :1425-1427: "THE DECLARER FACT IS BORN HERE (DESIGN 6b) ... a census joins on the IDENTITY, never on the chain's text."
- Census admission uses Admission.imports = Empty (source_root_eval.dag:1144 native_test_module_admission) => import-row admission reasons (N3/N5) unreachable on the census module lane.
- No always-prepended advisory found for the probe modules: QN machinery emits rejected-only (dag.dag:8213-8596); map_get_checked accepts with None (std/witness.dag:58).
- Candidate P fits r=X ∧ a=- ∧ s=T ∧ four-closure uniformity IF the probe references resolve to refused provider rows — SOURCE READING ONLY; condition not verified; no attribution.
