#!/usr/bin/env bash
# Do the bytes the bash lane CLAIMS to emit actually parse?
#
# Every positive control in the bash emit witnesses compares the serializer's
# output against an expected string. That comparison is only as trustworthy as
# the expected string, and an expected string copied from what the builder
# printed agrees with its subject by construction -- see
# dag/gunbc/recurring_failure_mode/a_control_that_shares_its_derivation_with_its_subject.dag,
# specimen four, which is exactly how a broken `for` loop shipped green
# (gunbc#16248, review 93224): `for i in 1 2 do echo up; done` is a bash syntax
# error, and the control was green.
#
# So this harness asks a party that is NOT the model: the real interpreter, in
# syntax-only mode. It reads the expected strings OUT OF the witness files (the
# witnesses stay the single authority over those bytes; nothing here re-states
# one), writes each to a file, and runs `bash -n` on it. Anything the language
# refuses exits non-zero here regardless of what the builder emitted.
#
# Scope, stated plainly: `bash -n` proves bash ACCEPTS the shape. It does not
# prove the construct MEANS the right thing -- that is what the srv1 render
# digest equality and the dry-run argv comparison are for.
#
# Usage: check_goldens_parse.sh [witness.dag ...]
# With no arguments it checks the bash construct witness.

set -euo pipefail

witnesses=("$@")
if [ "${#witnesses[@]}" -eq 0 ]; then
  witnesses=("src/v2/test/claim/bash_emit_constructs.dag")
fi

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

for witness in "${witnesses[@]}"; do
  if [ ! -f "$witness" ]; then
    echo "MISSING witness: $witness" >&2
    exit 1
  fi

  out="$work/$(basename "$witness" .dag)"
  mkdir -p "$out"

  # Decode .dag string escapes (\n \t \" \\ \{ \} \u{...}) into the raw bytes
  # the serializer is claimed to produce.
  python3 - "$witness" "$out" <<'PY'
import io, os, re, sys

witness, out_dir = sys.argv[1], sys.argv[2]
text = io.open(witness, encoding="utf-8").read()
pattern = re.compile(r'^data ([A-Za-z0-9_]*golden[A-Za-z0-9_]*): String = "(.*)"\s*$', re.M)
escapes = {'n': '\n', 't': '\t', '"': '"', '\\': '\\', '{': '{', '}': '}',
           "'": "'", '$': '$', 'r': '\r'}

def decode(raw):
    out, i = [], 0
    while i < len(raw):
        char = raw[i]
        if char == '\\' and i + 1 < len(raw):
            nxt = raw[i + 1]
            if nxt == 'u':
                match = re.match(r'u\{([0-9A-Fa-f]+)\}', raw[i + 1:])
                if match:
                    out.append(chr(int(match.group(1), 16)))
                    i += 1 + match.end()
                    continue
            if nxt in escapes:
                out.append(escapes[nxt])
                i += 2
                continue
        out.append(char)
        i += 1
    return ''.join(out)

count = 0
for name, body in pattern.findall(text):
    io.open(os.path.join(out_dir, name + ".sh"), "w", encoding="utf-8").write(decode(body))
    count += 1
print("%s: %d expected strings read from the witness" % (witness, count))
PY

  # One file may hold several statements separated by the witness's own '\n'
  # escapes, which is a complete script; each is handed to bash as written.
  status=0
  for script in "$out"/*.sh; do
    name="$(basename "$script")"
    if bash -n "$script" 2>"$script.err"; then
      echo "BASHN OK   $name"
    else
      status=1
      echo "BASHN FAIL $name"
      sed 's/^/           /' "$script.err"
    fi
  done

  if [ "$status" -ne 0 ]; then
    echo "bash -n refused at least one expected string from $witness" >&2
    exit 1
  fi
done

echo "all expected strings parse"
