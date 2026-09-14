# Indirect Optional elimination consumer

The direct .dag comparator claims passed both Map cases but refused all three Optional cases. After correcting the new kernel owner's missing ident field, the refusal is a non-exhaustive match on Present { value: Resolved { node: ... } } at is_compiler_error: that function expects InferredNode, not Optional<InferredNode>.

The route is left.inferred.value -> infer_lookup.field_access_summary -> FieldAccessStyle.OptionalUnwrap -> v1_interpreter.eval_field_access. The interpreter's OptionalUnwrap arm returns any non-Null base unchanged. Explicit Present values remain variants, so the old identity operation leaves the wrapper intact. This indirect style consumer was outside the direct return_cardinality/CardOptional read census. No interpreter change or comparator-specific unwrap has been made. Scope ruling requested in msg_3dec1847: canonical elimination at its authority versus a separately owned existing interpreter value fork with B evidence on the emitted/native route. Neither branch is silently assumed.

These are interpreter-route receipts against the main committed seed executing the edited .dag function bodies. They are not a native regenerated-seed receipt and do not falsify B by themselves. The native matrix remains enrolled in infer_semantics_witness.

A further authored-source control covers type Chain<T> = End | Link { next: Chain<T>? }. Main returns CardOptional for Link.next. The finite self-reference bypass in resolve_node_bounded now applies only to an unmarked reference, so marked input reaches the same Optional ingestion boundary. This change and its control are in optional-root-latest.patch, after the exact 9a3c4e23dd5 transaction input. Candidate GREEN remains unobserved.


Gatekeeper ruling msg_bd0664f9: this interpreter VALUE fork is outside B. Native emitted execution must establish B's Optional control verdicts. The three unchanged Optional controls share floor_expected_red_chunk_interpreter_optional_unwrap for expected-red and non-verdict enrollment (the observed outcome is PatternMatchFailure, not semantic false). The two Map controls stay ordinary passing controls. The recurring_failure_mode row names the invalid state, harm, observed mitigation and ceiling by interpreter-route deletion. No interpreter production changes or comparator-side unwrapping are admitted. Enrollment declarations parse without diagnostics; a composed required-floor observation is still owed.

optional-root-next.patch remains the exact regenB.EdGCFA input. optional-root-latest.patch additionally includes the recursive authored Optional field correction/control and this enrollment. No generated candidates have been installed.
