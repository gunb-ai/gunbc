set -u
A="$1"; rm -rf "$A"; mkdir -p "$A"
emit() { d="$A/$1"; mkdir -p "$d"; printf '%s' "$2" > "$d/probe.dag"; }
svc() {
cat <<EOF2
module probe.$1

import std.types { String, Bool, Int, List }

service probe.Svc$1 {
  operation Run {
    input { $3 }
    output { stdout: String from "stdout" }
    $2
    exit {
      0 => Unit
      nonzero => String "x"
    }
  }
}
EOF2
}
emit "svc_resolve_control" "$(svc svc_resolve_control 'transport shell { argv: ["cat"] }' 'arg: NoSuchTypeZzz')"
emit "svc_infer_typeerror" "$(svc svc_infer_typeerror 'transport shell { argv: ["cat"], stdin: 1 }' 'arg: String')"
emit "svc_green_control"   "$(svc svc_green_control   'transport shell { argv: ["cat"], stdin: "{arg}" }' 'arg: String')"
emit "fn_infer_typeerror" 'module probe.fn_infer_typeerror

import std.types { String, Bool, Int }

fn takes_string(s: String) -> Int { 1 }

fn caller() -> Int { takes_string(s: 1) }
'
