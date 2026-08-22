#!/usr/bin/env python3
"""Join the shadow ledger to the published E0308 partition at EMITTED-FILE grain.

The join is deliberately weak and says so: the board's per-site key is (file,line,col) in
EMITTED Rust and the shadow's key is (caller module, caller decl, callee, formal index) in
.dag source. There is no line correspondence between them and manufacturing one would be
the join-by-generated-line-number failure. What IS shared is the emitted FILE, which is a
pure function of the caller module -- so file grain is the strongest key both instruments
can honestly agree on, and every number below is reported at that grain.
"""
import sys, csv, collections

shadow_path, board_path = sys.argv[1], sys.argv[2]

def emitted_file(module: str) -> str:
    return "src/" + module.replace(".", "_") + ".rs"

shadow = list(csv.DictReader(open(shadow_path), delimiter='\t'))
board  = list(csv.DictReader(open(board_path),  delimiter='\t'))

wd = [r for r in shadow if r["outcome"] == "WouldDiagnose" and r["exempt"] == "exempt"]
shadow_by_file = collections.Counter(emitted_file(r["caller_module"]) for r in wd)
board_by_file  = collections.Counter(r["file"] for r in board)

files = sorted(set(shadow_by_file) | set(board_by_file),
               key=lambda f: (-board_by_file.get(f,0), -shadow_by_file.get(f,0), f))

print(f"shadow WouldDiagnose (exempt v2.* callers): {len(wd)} relations")
print(f"board E0308 sites:                          {len(board)}")
print()
print(f"{'emitted file':52s} {'board':>6s} {'shadow':>7s}")
both_board = both_shadow = 0
for f in files:
    b, s = board_by_file.get(f,0), shadow_by_file.get(f,0)
    if b and s:
        both_board += b; both_shadow += s
    print(f"{f:52s} {b:6d} {s:7d}")
print()
print(f"files carrying BOTH a board site and a shadow candidate: "
      f"{sum(1 for f in files if board_by_file.get(f) and shadow_by_file.get(f))}")
print(f"  board sites in those files : {both_board} / {len(board)} "
      f"({100*both_board/len(board):.1f}% of the E0308 population)")
print(f"  shadow candidates in those files: {both_shadow} / {len(wd)}")
print()
print("board sites in files with NO shadow candidate at all "
      "(cannot be source conformance debt at this seam):",
      sum(v for k,v in board_by_file.items() if not shadow_by_file.get(k)))
