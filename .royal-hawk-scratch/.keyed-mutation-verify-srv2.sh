set -euo pipefail
export PATH="/home/briansrls/.cargo/bin:$PATH"
verification_root=/home/briansrls/royal-hawk-241
mkdir -p "$verification_root"
verification_run=$(mktemp -d "$verification_root/verification.XXXXXX")
git clone --no-checkout https://github.com/gunb-ai/gunbc.git "$verification_run/repo"
git -C "$verification_run/repo" fetch origin royal-hawk-241/keyed-module-roots
git -C "$verification_run/repo" checkout --detach e9423d7e0180d3a2c0ea3f99535f88107056d555
mkdir "$verification_run/runner-temp"
printf '%s\n' "$verification_run/verification.log"
cd "$verification_run/repo"
cat > .keyed-mutation-controls.py <<'MUTATION_SCRIPT'
import os
from pathlib import Path
import re
import subprocess
import tempfile

source = Path('src/v2/compiler/03_name_resolve.dag')
original = source.read_text()
start = original.index('                match map_lookup(m: prior, key: name) {')
end = original.index('\n            }\n          } else {', start)
without_membership = original[:start] + '''                ModuleRootsValid {
                  seen: map_insert(m: prior, key: name, value: root),
                  diagnostics: merged_diag
                }''' + original[end:]
needle = 'seen: map_insert(m: prior, key: name, value: root),'
assert original.count(needle) == 1
without_insert = original.replace(needle, 'seen: prior,')
controls = [
    ('membership_removed', without_membership, 'keyed_validation_preserves_general_ambiguity_diagnostic'),
    ('insert_removed', without_insert, 'validated_subject_index_matches_general_lookup'),
]
try:
    for label, mutated, witness in controls:
        source.write_text(mutated)
        env = dict(os.environ)
        env['RUNNER_TEMP'] = tempfile.mkdtemp(prefix='mutation-', dir=env['RUNNER_TEMP'])
        run = subprocess.run(['./target/release/gunbc', 'run', '--source-root', 'dag', '--source-root', 'src/v2', '--entry', 'src/v2/test/claim/name_resolve/admission_fail_closed_test.dag', '--function', witness, '--claim-run'], env=env, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        print(run.stdout, end='', flush=True)
        semantic_red = run.returncode != 0 and re.search(r'^FAIL ' + re.escape(witness) + r'\s*$', run.stdout, re.MULTILINE)
        if not semantic_red:
            raise RuntimeError(f'{label}: expected an executed false witness, not a resolution/build refusal (rc={run.returncode})')
        print(f'MUTATION_CONTROL {label} EXPECTED_SEMANTIC_RED', flush=True)
finally:
    source.write_text(original)
assert source.read_text() == original
MUTATION_SCRIPT
systemd-run --user --scope -p MemoryMax=24G env PATH="$PATH" RUNNER_TEMP="$verification_run/runner-temp" CTRL_BUILD_MODE=local bash -c '
set -euo pipefail
cargo build --release -p v1-compiler --bin gunbc
failed=0
for witness in validated_subject_index_matches_general_lookup keyed_validation_preserves_general_ambiguity_diagnostic keyed_validation_equal_duplicate_precedes_malformed keyed_validation_unequal_duplicate_precedes_malformed keyed_validation_malformed_precedes_duplicate validate_module_roots_malformed_root_is_rejected resolve_module_not_found_preserves_missing_module_identity resolve_duplicate_module_roots_is_rejected; do
  if env PATH="$PATH" RUNNER_TEMP="$(mktemp -d "$RUNNER_TEMP/witness.XXXXXX")" ./target/release/gunbc run --source-root dag --source-root src/v2 --entry src/v2/test/claim/name_resolve/admission_fail_closed_test.dag --function "$witness" --claim-run; then
    printf "VERIFY %s PASS\n" "$witness"
  else
    result=$?
    printf "VERIFY %s FAIL rc=%s\n" "$witness" "$result"
    failed=1
  fi
done

test "$failed" -eq 0
python3 .keyed-mutation-controls.py
' > "$verification_run/verification.log" 2>&1
printf '%s\n' "$verification_run/verification.log"
