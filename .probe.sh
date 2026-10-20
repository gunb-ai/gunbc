set -u
unset GUNBC_BIND_MEMORY_CGROUP_BYTES
cargo build --release 2>&1 | tail -2
G=$(find . -path '*release/gunbc' -type f | head -1)
mkdir -p zzprobe
cat > zzprobe/defs.dag <<'EOT'
module probe.defs

type Chain<T>
  = End
  | Link { head: T, tail: Chain<T> }

type Path = Chain<String>

type Box<T> { v: T }

type BoxS = Box<String>

type Pair<A, B> { a: A, b: B }

type PairS = Pair<String, Int>

fn mk() -> Path { End }
EOT
cat > zzprobe/use.dag <<'EOT'
module probe.use

import probe.defs { Chain, Path, mk }
import std.algebra { empty_map, map_insert }

fn snoc_p(p: Path, s: String) -> Path { Link { head: s, tail: p } }
fn snoc_d(p: Chain<String>, s: String) -> Chain<String> { Link { head: s, tail: p } }

fn alias_param(p: Path, xs: List<String>) -> Map<String, Path> {
  fold(xs, init: empty_map(), f: fn(acc, x) { map_insert(acc, x, p) })
}
fn alias_call(p: Path, xs: List<String>) -> Map<String, Path> {
  fold(xs, init: empty_map(), f: fn(acc, x) { map_insert(acc, x, snoc_p(p: p, s: x)) })
}
fn direct_param(p: Chain<String>, xs: List<String>) -> Map<String, Chain<String>> {
  fold(xs, init: empty_map(), f: fn(acc, x) { map_insert(acc, x, p) })
}
fn direct_call(p: Chain<String>, xs: List<String>) -> Map<String, Chain<String>> {
  fold(xs, init: empty_map(), f: fn(acc, x) { map_insert(acc, x, snoc_d(p: p, s: x)) })
}
fn mixed_call(p: Path, xs: List<String>) -> Map<String, Chain<String>> {
  fold(xs, init: empty_map(), f: fn(acc, x) { map_insert(acc, x, snoc_p(p: p, s: x)) })
}
EOT
$G compile --source-root zzprobe --source-root dag --source-root src/v2 --entry zzprobe/use.dag --output-dir /tmp/pout9 --target rust 2>&1 | grep "DBG\|compiled"
echo PROBEOUT; grep -n "fn \|rc_empty_map::<" /tmp/pout9/src/v1/stage0/src/probe_use.rs | cut -c1-330
