# Q5a: callable parameter variance (read only, nothing run)

## 1. Comparison site (src/v1/04_infer.dag, symbols)
- callable_inhabitance_mismatch (arrow vs arrow arm) -> callable_signature_mismatch -> callable_arrow_mismatch.
- callable_arrow_mismatch: arity must be equal; then `f_params |> enumerate |> any` calls callable_component_ground_mismatch(formal_part: callable_param_type(formal param i), actual_part: callable_param_type(actual param i)); then the same call on the two resolved return types. PARAM AND RETURN ARE ORIENTED IDENTICALLY (formal_part = declared side, actual_part = offered side). There is no arm that flips the parameter direction.
- callable_component_ground_mismatch: arrow/arrow recurses; one-sided arrow refuses only if either side is ground; otherwise, only when BOTH sides are conformance_ground_type, it returns !node_type_compatible(formal_part, instantiated). Non-ground parts are admitted unjudged.
- node_type_compatible (04_types.dag) is undirected: error and type-variable sides are true, and its own arrow arm uses callable_signatures_agree with the same symmetric agree on params and returns.

## 2. Direction of the defect (by reading; unmeasured)
The relation is symmetric, so the comparison is not invariant in the strict sense: it is direction-blind. Consequences by case:
- ADMITS wrong code where compat holds in the wrong direction. Declared fn(String) -> X, offered fn(NonEmptyStr) -> X: the offered fn demands a narrower argument than callers may supply (unsound). A symmetric compat that treats NonEmptyStr/String as compatible admits it. Whether it does is exactly what the cells measure.
- REFUSES correct code where the relation is directional-only. Declared fn(NonEmptyStr) -> X, offered fn(String) -> X is sound (contravariant parameter). If compat is true only in the widening direction the symmetric call can still refuse it depending on argument order. Same for returns the other way.
- Both are possible; which one occurs is a property of node_type_compatible's actual asymmetry, which I did not run. Do not assume either.

## 3. Authority
- DESIGN section 4: coercion is the implicit-safe subset of declared conversions (total, exact, unique, pure, consent-free); there is no subtyping relation. `Bool` does not implicitly inhabit `Int`.
- grep for contravariance, covariance, variance and function-space over dag/std, dag/gunbc, src/v2 and DESIGN.md finds only prose about spatial covariance and a v2 witness comment (infer_free_monoid_element_inhabitance_witness) saying a covariance route that widens one step is not yet declared. NOTHING declares function-space variance. That is a MODELING OBLIGATION (declare the rule over the coercion/conversion rows: parameter positions check the conversion from declared to offered, returns from offered to declared), not a fix to the comparison.

## 4. RED/control cells (dag/test/claim/callable_param_variance_differential_witness.dag)
Each cell is a function passed to a parameter declared as a function type, pair (W = wider, N = narrower, with a declared one-step conversion N->W, e.g. NonEmptyStr->String):
- C1 control: same types fn(String)->Int offered at fn(String)->Int. Accepted.
- A (sound by contravariance): declared fn(N)->Int, offered fn(W)->Int. Expected under a variance rule: accepted. Measures whether today refuses correct code.
- B (unsound): declared fn(W)->Int, offered fn(N)->Int. Expected: refused. Measures whether today admits wrong code.
- C (return, sound): declared fn(Int)->W, offered fn(Int)->N. Expected accepted.
- D (return, unsound): declared fn(Int)->N, offered fn(Int)->W. Expected refused.
The A/B and C/D pairs are mirror images: a symmetric relation gives the same verdict to both members of a pair, so any pair with equal verdicts discriminates direction-blindness; unequal verdicts show a directional bug. Expected classes are stated beside each cell; the measured column is empty until run.
