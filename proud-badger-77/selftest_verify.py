#!/usr/bin/env python3
"""Self-test for /tmp/verify_nal.py. Builds synthetic rev7-shaped receipt bundles
(no closure.txt/closure.err) and asserts the classifier's verdict. Run: python3 /tmp/selftest_verify.py"""
import subprocess, os, sys
ROSTER = open('/tmp/roster.csv').read().strip()
def build(d, mode="good"):
    os.makedirs(d, exist_ok=True)
    for f in os.listdir(d): os.remove(os.path.join(d, f))
    roster = [x.strip() for x in ROSTER.split(",") if x.strip()]
    lines = []
    for i, c in enumerate(roster):
        if mode == "unobserved" and i == 0: lines.append(f"UNOBSERVED {c}")
        elif mode == "suffixed" and i == 0:  lines.append(f"PASS {c} (panicked)")
        elif mode == "missing" and i == 0:   pass
        elif mode == "dup" and i == 0:        lines += [f"PASS {c}", f"FAIL {c}"]
        else: lines.append(f"{'FAIL' if c.startswith('nal_arm_') else 'PASS'} {c}")
    open(d + '/verdicts.txt', 'w').write("\n".join(lines) + "\n")
    led = []
    for c in roster:
        want = "returned-true" if f"PASS {c}" in lines else "returned-false"
        if mode == "ledger_mismatch" and c == roster[3]:
            want = "returned-false" if want == "returned-true" else "returned-true"
        led.append(f"gunbc.test.claim.compiler.nominal.{c}\texpect-hold\t{want}\tdetail")
        if mode == "dupledger" and c == roster[1]:
            led.append(f"gunbc.test.claim.compiler.nominal.{c}\texpect-hold\t{want}\tdup")
        if mode == "extraledger":
            led.append("gunbc.test.claim.compiler.nominal.not_in_roster_holds\texpect-hold\treturned-true\tx")
    open(d + '/outcomes.json', 'w').write("\n".join(led) + "\n")
    open(d + '/END_receipt.txt', 'w').write("OUTCOME=OBSERVED rc=0 table_complete 42/42\n")
    open(d + '/pins.txt', 'w').write("head 5b02c6dd3d963cb9863b163a69cba56e900f8698\n")
    sha = "92afb15ade7bcee735f5ec689d2b79d29d75e086a34207b277b3194841ef7739"
    open(d + '/bin.sha256.before', 'w').write(("0" * 64 if mode == "binswap" else sha) + "\n")
    open(d + '/bin.sha256.after', 'w').write(sha + "\n")
    open(d + '/prep.stdout', 'w').write("PREP_OK\n")
    open(d + '/meas.stdout', 'w').write("OUTCOME=OBSERVED rc=0\n")
    open(d + '/claim_rc', 'w').write("0\n")
cases = [("good","OBSERVED"),("unobserved","NO_OBSERVATION"),("suffixed","NO_OBSERVATION"),
         ("missing","NO_OBSERVATION"),("dup","NO_OBSERVATION"),("binswap","NO_OBSERVATION"),
         ("ledger_mismatch","NO_OBSERVATION"),("dupledger","NO_OBSERVATION"),("extraledger","NO_OBSERVATION")]
ok = True
for mode, expect in cases:
    d = "/tmp/t_" + mode; build(d, mode)
    r = subprocess.run(["python3", "/tmp/verify_nal.py", d, "--roster", ROSTER], capture_output=True, text=True)
    out = (r.stdout + r.stderr).strip().splitlines()
    head = next((l for l in out if "OBSERV" in l), out[0] if out else "?")
    good = head.startswith(expect); ok &= good
    print(("PASS" if good else "FAIL"), f"{mode:>15}", "->", head[:90])
print("SELF-TEST", "GREEN" if ok else "RED")
sys.exit(0 if ok else 1)
