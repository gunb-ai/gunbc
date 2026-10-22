#!/bin/bash
# Scratch (never lands): E7 = windowed emitted-text dump, SELF-WRITTEN bin (workspace
# patch caching kept the stale E4 bin; heredoc is the proven fresh-content route, E1b).
set -uo pipefail
MEMTOTAL_KB=$(awk "/MemTotal/{print \$2}" /proc/meminfo)
[ "$MEMTOTAL_KB" -ge 60000000 ] || { echo "runner memory too low: $MEMTOTAL_KB kB"; exit 3; }

cat > src/v1/stage0/src/bin/unary_prefix_emit_win_tmp.rs <<'RSEOF'
// Lane scratch (never lands): windowed emitted-text dump for substring calibration.
use std::rc::Rc;
use v1_compiler::v1_compiler_artifact::RenderTarget;
use v1_compiler::v1_compiler_compile::{compile_sources, SourceFile};

fn dump(label: &str, module: &str, params: &str, ret: &str, body: &str) {
    let src = format!("module {module}\n\nfn probe_case({params}) -> {ret} {{\n  {body}\n}}\n");
    let sources: Vec<Rc<SourceFile>> = vec![Rc::new(SourceFile {
        path: format!("probe_{label}.dag"),
        content: src,
    })];
    let result = compile_sources(Rc::new(sources.into()), RenderTarget::Rust);
    let mut found = false;
    for f in result.files.iter() {
        if let Some(idx) = f.content.find("fn probe_case") {
            found = true;
            let mut end = (idx + 320).min(f.content.len());
            while end > idx && !f.content.is_char_boundary(end) {
                end -= 1;
            }
            println!("BEGIN {label} | body: {body} | file {}", f.path);
            println!("{}", &f.content[idx..end]);
            println!("END {label}");
        }
    }
    if !found {
        println!("BEGIN {label} | body: {body} | NO probe_case among {} files", result.files.len());
        for f in result.files.iter() {
            println!("  file: {}", f.path);
        }
        println!("END {label}");
    }
}

fn main() {
    dump("minus_fixed", "probe.unary_prefix.minus", "", "Int", "-1 - 2 - 3");
    dump("minus_bug", "probe.unary_prefix.minusbug", "", "Int", "-(1 - 2 - 3)");
    dump("bang_fixed", "probe.unary_prefix.bang", "t: Bool, f: Bool", "Bool", "!t && f && t");
    dump("bang_bug", "probe.unary_prefix.bangbug", "t: Bool, f: Bool", "Bool", "!(t && f && t)");
}
RSEOF

cargo build --release -p v1-compiler --bin unary_prefix_emit_win_tmp 2>&1 | tail -2
[ -x ./target/release/unary_prefix_emit_win_tmp ] || { echo "FATAL: bin missing"; exit 4; }
./target/release/unary_prefix_emit_win_tmp > /tmp/dumpwin.txt 2>&1
echo "dump-lines: $(wc -l < /tmp/dumpwin.txt)"
cat /tmp/dumpwin.txt
