#!/usr/bin/env python3
"""Pull the regenerated mirrors out of a regen3 dispatch log and write them next to the
harness, verifying each decodes to valid Rust with the expected decision markers.

The log is ANSI-stained and line-wrapped at 100 columns by `base64 -w 100`, so the block
is collected between the variant-tagged BEGIN/END markers, whitespace-stripped, and decoded
with gzip. Any decode failure or missing marker is an error, not a silent empty file.
"""
import sys, re, gzip, base64, pathlib

MARKS_KEEP   = {"gate": 0, "stale": 1, "planned": 1}
MARKS_DELETE = {"gate": 0, "stale": 0, "planned": 1}

def extract(log, variant):
    raw = pathlib.Path(log).read_bytes().decode("utf-8", "replace")
    raw = re.sub(r"\x1b\[[0-9;]*m", "", raw)
    begin = "MIRROR-B64-BEGIN %s" % variant
    end   = "MIRROR-B64-END %s" % variant
    i = raw.find(begin); 
    if i < 0: return None, "no %r in log" % begin
    j = raw.find(end, i)
    if j < 0: return None, "no %r (dispatch may still be running)" % end
    body = raw[i + len(begin):j]
    body = "".join(ch for ch in body.split() if re.match(r"[A-Za-z0-9+/=]", ch))
    if len(body) < 10000: return None, "suspiciously short base64 body (%d chars)" % len(body)
    try:
        data = gzip.decompress(base64.b64decode(body))
    except Exception as e:
        return None, "decode failed: %s" % e
    return data, "ok %d bytes" % len(data)

def marks(text):
    return {"gate": text.count("fn call_needs_generic_rebinding_pass"),
            "stale": text.count("fn typed_expr_is_stale"),
            "planned": text.count("fn infer_planned_lambda_recheck")}

def main():
    log = sys.argv[1]
    out = pathlib.Path(sys.argv[2]) if len(sys.argv) > 2 else pathlib.Path(".sdm")
    ok = True
    for variant, want in (("keep", MARKS_KEEP), ("delete", MARKS_DELETE)):
        data, why = extract(log, variant)
        if data is None:
            print("EXTRACT-FAIL %s: %s" % (variant, why)); ok = False; continue
        text = data.decode("utf-8")
        m = marks(text)
        bad = {k: (m[k], want[k]) for k in want if m[k] != want[k]}
        if bad:
            print("EXTRACT-MARK-MISMATCH %s got=%s want=%s" % (variant, m, want)); ok = False; continue
        if not text.lstrip().startswith("//") and "gen v1.compiler.infer" not in text[:4000]:
            print("EXTRACT-WARN %s: header does not look like generator output" % variant)
        p = out / ("gen_%s_v1_compiler_infer.rs" % variant)
        p.write_bytes(data)
        import hashlib
        print("EXTRACT-OK %s -> %s bytes=%d lines=%d sha256=%s marks=%s" % (
            variant, p, len(data), text.count("\n"), hashlib.sha256(data).hexdigest()[:16], m))
    return 0 if ok else 1

if __name__ == "__main__":
    sys.exit(main())
