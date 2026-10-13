#!/usr/bin/env bash
# Interim instrument for dag/gunbc/emit_state_threading_specimen.dag (roadmap node
# emission-building-block-realization, slice 1). Run via:
#   ctrl-build --remote -- bash tools/emit_state_threading_reading.sh
# (build and run must share one dispatch: runners are amd64, sessions arm64).
#
# PREDICTIONS, written before the first run (n = steps over a prebuilt n-element source; the window
# excludes source construction):
#   emitted : ~3 alloc ops/step (Rc<SpecAcc>, Rc<SpecCursor>, copy-on-write of the shared im::Vector
#             tail chunk at the push), ~700 B/step  -> n=4096: ~12k ops, ~2.9 MB; linear in n.
#   minimum : amortized vector growth only (one chunk per 64 pushes plus spine) -> n=4096: <150 ops;
#             ops/step -> ~0.02.
#   result  : identical in both forms (behavior agreement).
# READING: not transcribed here (DESIGN 6); the script prints it on every run (form=... ops= bytes=).
# DISSOLUTION: this script is an interim, hand-authored instrument. It is retired by a row in
#   gunbc.instrument_targets bound through gunbc.target_binding TargetProducer that emits, builds and
#   counts in one dispatch (capability: a modeled counting-allocator harness producer). Not yet built.
# Rc clone count is not observable through an allocator; it is read from the emitted source instead.
set -euo pipefail
cargo build --release -p v1-compiler --bin gunbc
out=/tmp/ecq_state_threading
rm -rf "$out"
./target/release/gunbc compile --source-root dag --source-root src/v2 \
  --entry dag/gunbc/emit_state_threading_specimen.dag --output-dir "$out" --target rust
crate=$(sed -n 's/^name *= *"\(.*\)"/\1/p' "$out/Cargo.toml" | head -1 | tr - _)
mkdir -p "$out/src/bin"
cat > "$out/src/bin/runner.rs" <<RS
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicU64, Ordering};
use std::rc::Rc;
use im::{HashMap, Vector};
use $crate::gunbc_emit_state_threading_specimen::{spec_build, spec_step, spec_walk_inline, spec_walk_inline_fieldwise, spec_walk_named, SpecAcc, SpecCursor};

struct Counting;
static OPS: AtomicU64 = AtomicU64::new(0);
static BYTES: AtomicU64 = AtomicU64::new(0);
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        OPS.fetch_add(1, Ordering::Relaxed);
        BYTES.fetch_add(l.size() as u64, Ordering::Relaxed);
        System.alloc(l)
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) { System.dealloc(p, l) }
}
#[global_allocator]
static A: Counting = Counting;

fn window<R>(f: impl FnOnce() -> R) -> (R, u64, u64) {
    let (o0, b0) = (OPS.load(Ordering::Relaxed), BYTES.load(Ordering::Relaxed));
    let r = f();
    (r, OPS.load(Ordering::Relaxed) - o0, BYTES.load(Ordering::Relaxed) - b0)
}

// The justified-minimum form: the same algorithm and the same collections, with the record by value,
// fields moved out of the dying accumulator, the observation borrowing the retained source.
struct MCursor { source: Rc<Vector<i64>>, index: i64 }
struct MAcc { remaining: MCursor, tokens: Vector<i64>, pos: i64, done: bool, context: HashMap<String, i64>, tag: i64 }
fn mstep(acc: MAcc) -> MAcc {
    let MAcc { remaining, mut tokens, pos, done: _, context, tag } = acc;
    match remaining.source.get(remaining.index as usize).copied() {
        None => MAcc { remaining, tokens, pos, done: true, context, tag },
        Some(h) => {
            tokens.push_back(h);
            MAcc { remaining: MCursor { source: remaining.source, index: remaining.index + 1 }, tokens, pos: pos + 1, done: false, context, tag }
        }
    }
}

fn main() {
    let (_p, po, pb) = window(|| std::hint::black_box(vec![1u8; 4096]).len());
    if po == 0 || pb < 4096 { println!("counter=disconnected"); std::process::exit(3); }
    for n in [4096i64, 16384] {
        let src = spec_build(n);
        let (r, ops, bytes) = window(|| {
            let mut acc = Rc::new(SpecAcc { remaining: Rc::new(SpecCursor { source: src.clone(), index: 0 }), tokens: Rc::new(Vector::new()), pos: 0, done: false, context: Rc::new(HashMap::new()), tag: 7 });
            for _ in 0..src.len() { acc = if acc.done { acc } else { spec_step(acc) }; }
            acc.tokens.len() as i64 + acc.pos + acc.tag
        });
        for (name, f) in [("inline", spec_walk_inline as fn(Rc<Vector<i64>>) -> i64), ("inline_fieldwise", spec_walk_inline_fieldwise), ("named_fn", spec_walk_named)] {
            let (r, ops, bytes) = window(|| f(src.clone()));
            println!("form={name} n={n} ops={ops} bytes={bytes} ops_per_step={:.3} result={r}", ops as f64 / n as f64);
        }
        println!("form=emitted n={n} ops={ops} bytes={bytes} ops_per_step={:.3} result={r}", ops as f64 / n as f64);
        let (r, ops, bytes) = window(|| {
            let mut acc = MAcc { remaining: MCursor { source: src.clone(), index: 0 }, tokens: Vector::new(), pos: 0, done: false, context: HashMap::new(), tag: 7 };
            for _ in 0..src.len() { acc = if acc.done { acc } else { mstep(acc) }; }
            acc.tokens.len() as i64 + acc.pos + acc.tag
        });
        println!("form=minimum n={n} ops={ops} bytes={bytes} ops_per_step={:.3} result={r}", ops as f64 / n as f64);
    }
}
RS
cd "$out" && cargo run --release --bin runner 2>&1 | tail -60
