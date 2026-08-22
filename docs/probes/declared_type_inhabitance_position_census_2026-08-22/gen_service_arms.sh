set -u
A="$1"; rm -rf "$A"; mkdir -p "$A"
emit() { d="$A/$1"; mkdir -p "$d"; printf '%s' "$2" > "$d/probe.dag"; }
svc() {
cat <<EOF2
module probe.$1

import std.types { String, Bool, Int, List }

service probe.Svc$1 {
  operation Run {
    input { arg: String }
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
# argv is stored as the transport node's CHILDREN (00_core shell_transport_node)
emit "argv_child_undefined"  "$(svc argv_child_undefined  'transport shell { argv: ["echo", nosuchname_zzz] }')"
emit "argv_not_a_list"       "$(svc argv_not_a_list       'transport shell { argv: nosuchname_zzz }')"
# stdin is stored as a PROPERTY of the transport node -- the walked side
emit "stdin_prop_undefined"  "$(svc stdin_prop_undefined  'transport shell { argv: ["cat"], stdin: nosuchname_zzz }')"
emit "stdin_prop_control"    "$(svc stdin_prop_control    'transport shell { argv: ["cat"], stdin: "{arg}" }')"
