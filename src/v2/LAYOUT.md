# v2 `.dag` tree layout (Google C++ naming)

Every `.dag` file path under `src/v2/` mirrors its `module` declaration the way
[Google C++ style](https://google.github.io/styleguide/cppguide.html#Names_and_Order_of_Includes)
maps namespaces to directories: directory segments are the module path; the
filename is the final segment.

## Rules

1. **No `__` in basenames.** The legacy flat-dir encoding (`lens_cost__atom_zero.dag`)
   is retired. Nested modules use real subdirectories (`lens_cost/atom_zero.dag`).
   `ci_claim_gate` fails closed on any `.dag` basename containing `__`.

2. **Implementation modules** (`module v2.{layer}.{…}`): path is
   `{layer}/{…}.dag` relative to `src/v2/`, with each dot-separated segment
   after `v2.{layer}` becoming a subdirectory.

   Example: `module v2.std.compilers.lexing` → `src/v2/std/compilers/lexing.dag`

3. **Test claim modules** (`module v2.test.{…}`): path is
   `test/claim/{…}.dag` relative to `src/v2/`, with each dot-separated segment
   after `v2.test` becoming a subdirectory under `test/claim/`.

   Example: `module v2.test.lens_cost.atom_zero` →
   `src/v2/test/claim/lens_cost/atom_zero.dag`

   Example (deeper): `module v2.test.host_language_transport_script.corpus.migrated_transports_clean` →
   `src/v2/test/claim/host_language_transport_script/corpus/migrated_transports_clean.dag`

4. **Layer roots** (`std`, `extdeps`, `compiler`, `lens`, `workflow`, …) hold
   implementation authority. Bool-witness / TestClaim corpus lives under
   `test/claim/` regardless of which layer the claim exercises.

5. **`test fn` / `test data` declarations** may appear in `test/claim/` files
   (module path is the authority) or in legacy `*_test.dag` files elsewhere.
   They must not appear in implementation-layer files.

## CI enforcement

`cargo run -p ci_claim_gate --release -- --source-root src/v2 …` runs filename
hygiene (rules 1 and 3) before the witness green pass. Branch protection runs
this binary as the CI floor (see `.github/workflows/ci.yml`).

## Discovery

`discover_owned_data` defaults to `--scan-dir src/v2/test/claim`. The claim
gate discovers the corpus from `--scan-dir src/v2/test`.
