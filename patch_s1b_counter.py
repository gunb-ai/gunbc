#!/usr/bin/env python3
"""S1B COUNTER -- instrumentation for the parent's question, in the same dispatch as probe v4.

  stern-bat-401 msg_0b50238d: "add a counter inside resolved_type's minting arms (Absent / else) for
  calls reached from node_type_equals_core during the fold's comparisons. If it's 0, S1b stays parked
  for good; if not, report the rows."

WHY IT LANDS IN THE EMITTED MIRROR AND NOT THE .dag. `resolved_type` is in src/v1/04_types.dag, whose
type-compatibility authority is owned by quiet-eagle-395 (#14256) -- I am forbidden to edit it, and the
parent's own instruction is that this is a MEASUREMENT, not a change. So the counter goes into the
generated stage0 mirror `src/v1/stage0/src/v1_compiler_infer_types.rs` in a throwaway commit on the
scratch branch, exactly like the fold probe. NOTHING here is meant for #14152.

THE ARMS, AS THEY ACTUALLY EMIT. The DAG source has an Absent arm and an else arm, both minting
error_type; the emitter collapses them into one `_ => error_type()`. A single counter therefore cannot
answer "Absent or else", so the instrument re-splits the collapsed arm by whether `inferred` is None
(the Absent arm) or Some(non-Resolved) (the else arm) and counts them separately. Counting only while
`S1B_DEPTH > 0` is what makes the answer mean "reached from node_type_equals_core during the fold's
comparisons" rather than "ever, anywhere in the compiler".
"""
import sys

# The emitted shape, verified by hand against the rehearsal mirror before this file was written.
ANCHOR = '''pub fn resolved_type(n: Rc<Node>) -> Rc<Node> {
    match n.inferred.clone().as_deref().cloned() {
        Some(InferredNode::Resolved { node: rt, .. }) => rt.clone(),
        _ => error_type(),
    }
}'''

REPLACEMENT = '''static S1B_MINT_ABSENT: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(0);
static S1B_MINT_ELSE: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(0);
static S1B_MINT_ALL: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(0);
static S1B_DEPTH: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(0);
// NON-VACUITY COUNTERS. A bracket that was never entered and a bracket that ran and genuinely saw no
// minting BOTH read 0, and the difference is the entire answer: '0 therefore S1b stays parked' is only
// sound if the instrument was live. TOTAL counts every resolved_type call, BRACKETED every call seen
// while the depth was positive, and ENTERS every bracket. If TOTAL is large and BRACKETED is 0, the
// 0 is real. If ENTERS is 0, the instrument never ran and the 0 means nothing.
static S1B_TOTAL: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(0);
static S1B_BRACKETED: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(0);
static S1B_ENTERS: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(0);

/// Bracket the fold's comparison. Minting is counted only while the depth is positive, which is what
/// restricts the answer to calls reached THROUGH node_type_equals_core from the fold, and excludes every
/// other resolved_type call in the compiler (which would make the number meaningless).
pub fn s1b_enter() {
    S1B_ENTERS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    S1B_DEPTH.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
}
pub fn s1b_totals() -> (i64, i64, i64) {
    (
        S1B_TOTAL.load(std::sync::atomic::Ordering::Relaxed),
        S1B_BRACKETED.load(std::sync::atomic::Ordering::Relaxed),
        S1B_ENTERS.load(std::sync::atomic::Ordering::Relaxed),
    )
}
pub fn s1b_exit() {
    S1B_DEPTH.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
}
pub fn s1b_mint_snapshot() -> i64 {
    S1B_MINT_ALL.load(std::sync::atomic::Ordering::Relaxed)
}
pub fn s1b_mint_absent() -> i64 {
    S1B_MINT_ABSENT.load(std::sync::atomic::Ordering::Relaxed)
}
pub fn s1b_mint_else() -> i64 {
    S1B_MINT_ELSE.load(std::sync::atomic::Ordering::Relaxed)
}
fn s1b_count_mint(is_absent: bool) {
    S1B_TOTAL.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    if S1B_DEPTH.load(std::sync::atomic::Ordering::Relaxed) > 0 {
        S1B_BRACKETED.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        S1B_MINT_ALL.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if is_absent {
            S1B_MINT_ABSENT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        } else {
            S1B_MINT_ELSE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
    }
}

pub fn resolved_type(n: Rc<Node>) -> Rc<Node> {
    match n.inferred.clone().as_deref().cloned() {
        Some(InferredNode::Resolved { node: rt, .. }) => rt.clone(),
        Some(_) => {
            // the DAG 'else' arm: inferred present but not a Resolved node
            s1b_count_mint(false);
            error_type()
        }
        None => {
            // the DAG 'Absent' arm
            s1b_count_mint(true);
            error_type()
        }
    }
}'''


def patch(path, out=None):
    src = open(path).read()
    report = {"anchor": src.count(ANCHOR), "already": src.count("s1b_enter")}
    if report["already"]:
        return report | {"ok": True, "note": "already patched"}
    if report["anchor"] != 1:
        return report | {"ok": False, "why": "anchor must match exactly once",
                         "first_line": src.splitlines()[100] if len(src.splitlines()) > 100 else ""}
    new = src.replace(ANCHOR, REPLACEMENT, 1)
    open(out or path, "w").write(new)
    return report | {"ok": True, "bytes": len(new)}


def selftest():
    synth = ("use std::rc::Rc;\nfn error_type() -> Rc<Node> { unimplemented!() }\n"
             "enum InferredNode { Resolved { node: Rc<Node> } }\nstruct Node;\n"
             + ANCHOR + "\npub fn other() {}\n")
    open("/tmp/s1b_selftest_seed.rs", "w").write(synth)
    r = patch("/tmp/s1b_selftest_seed.rs", "/tmp/s1b_selftest_out.rs")
    if not r.get("ok"):
        print("SELFTEST FAIL:", r)
        return 1
    out = open("/tmp/s1b_selftest_out.rs").read()
    for need in ("pub fn s1b_enter", "pub fn s1b_mint_snapshot", "s1b_count_mint(false)",
                 "s1b_count_mint(true)", "pub fn s1b_totals", "S1B_BRACKETED.fetch_add"):
        if need not in out:
            print("SELFTEST FAIL: missing", need)
            return 1
    print("SELFTEST OK", r)
    return 0


if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1] == "--selftest":
        sys.exit(selftest())
    r = patch(sys.argv[1], sys.argv[2] if len(sys.argv) > 2 else None)
    print(r)
    sys.exit(0 if r.get("ok") else 4)
