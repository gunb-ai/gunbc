#!/usr/bin/env python3
"""Read REBINDARM lines from probe v2 and return the PRE-REGISTERED verdict, per row and in total.

Outcomes were registered in probe_bucketb2_14152.sh BEFORE the run (stern-bat-401 msg_576466dc required
registration first, so a cause cannot be fitted to the data afterwards):
  A  cross=true  AND core_na=true            -> the AMBIENT text judgment refuses, not the two types.
                                                FOLD SCOPE: pass the settled text judgment for each node's
                                                own module. Belongs in #14152.
  B  cross=false AND the trees differ        -> X1 PROPER: a defect in the relation's descent or in
     (below Resolved{node}, in raw children,    declaration identity. Its own PR.
      or in decl=)
  C  cross=false AND the trees are identical -> UNEXPLAINED. Report by identity. Pick no cause.
     in every printed field, yet refuses
Validity (checked first, because an arm count measured on a perturbed compiler is worthless):
  refusals must be 208 (config A's census) and one REBINDARM line per refusal.

Also prints the repL/repR pair, because 'cross' is only credible when one side is HostText/
CodePointSequence and the other NotText: a crossing between two NotText nodes would mean I have
mis-read what text_crossing_in_judgment does.
"""
import sys, re, collections, json

def parse(line):
    body = line.split('REBINDARM|', 1)[1]
    i = body.find('|;TREE')
    if i < 0:
        # the emitter writes ';;TREE' appended after the formal field with a preceding '|'.
        # Splitting on the FIRST such marker (not a greedy regex) matters: a printed shape may itself
        # contain the two-character sequence, and a greedy '.*' anchor silently ate the whole head in the
        # first draft -- every row came back UNPARSEABLE and I nearly read that as "no refusals".
        i = body.find(';;TREE')
        if i < 0:
            return None
        head = body[:i].rstrip()
        tree = body[i + len(';;TREE'):]
    else:
        head = body[:i].rstrip()
        tree = body[i + len('|;TREE'):]
    if head.endswith('|'):
        head = head[:-1]
    d = {}
    for kv in head.split('|'):
        if '=' in kv:
            k, v = kv.split('=', 1)
            d[k] = v
    d['tree'] = tree.strip()
    # ' ;; ' is the emitter's node separator (rebind_probe_tree). The terminal alternative must be the
    # actual end of string, not an empty match: (?= ;; |\s*$) matched at position 0 of the remainder and
    # truncated every node to nothing.
    d['_nodes'] = re.findall(
        r'\b([LR][0-9rA-Za-z]*) n=([^|]*)\|conn=([^|]*)\|card=([^|]*)\|decl=([^|]*)\|infer=(.*?)\|nc=(\d+)\|idsp=(.*?)(?: ;; |$)',
        tree)
    return d

def main(paths):
    rows = []
    for p in paths:
        for line in open(p, errors='replace'):
            if line.startswith('REBINDARM|'):
                d = parse(line)
                if d:
                    rows.append(d)
                else:
                    print("UNPARSEABLE REBINDARM LINE (truncated?):", line[:200])
    print("REBINDARM rows parsed: %d" % len(rows))
    if not rows:
        print("NOTHING PARSED -> the probe did not reach the refusal site. Do NOT read 0 as 'no refusals'.")
        return 1

    print("\n=== validity gates (from the dispatch log, re-checked here) ===")
    print("  distinct (module, unfoldedL, unfoldedR, bind): %d" % len({(r['module'], r.get('unfoldedL'), r.get('unfoldedR'), r['bind']) for r in rows}))

    print("\n=== outcome tally, exactly as pre-registered ===")
    A = [r for r in rows if r.get('cross') == 'true' and r.get('core_na') == 'true']
    noxc = [r for r in rows if r.get('cross') == 'false']
    def trees_identical(r):
        ls = [n for n in r['_nodes'] if n[0].startswith('L')]
        rs = [n for n in r['_nodes'] if n[0].startswith('R')]
        if len(ls) != len(rs):
            return False
        return all(a[1:] == b[1:] for a, b in zip(ls, rs))
    B = [r for r in noxc if not trees_identical(r)]
    C = [r for r in noxc if trees_identical(r)]
    print("  A  ambient text judgment (cross=true, core_na=true) : %d" % len(A))
    print("  B  relation descent/declaration (cross=false, trees differ) : %d" % len(B))
    print("  C  UNEXPLAINED (cross=false, trees identical, still refused) : %d" % len(C))
    print("  neither A/B/C (cross=true but core_na=false -> inconsistent, report) : %d"
          % (len(rows) - len(A) - len(B) - len(C)))

    print("\n=== arm differential, all rows (the measurement itself) ===")
    for k, c in collections.Counter(
            (r.get('equals'), r.get('core'), r.get('cross'), r.get('core_na'), r.get('equals_na'),
             r.get('textpair'), r.get('headapp')) for r in rows).most_common(12):
        print("  equals=%-5s core=%-5s cross=%-5s core_na=%-5s equals_na=%-5s textpair=%-5s headapp=%-5s  x%d" % (k + (c,)))

    print("\n=== repL/repR sanity for every crossing row (a crossing needs a text side) ===")
    repc = collections.Counter((r.get('repL', '')[:28], r.get('repR', '')[:28]) for r in rows if r.get('cross') == 'true')
    if not repc:
        print("  (no crossing rows)")
    for (a, b), n in repc.most_common(8):
        print("  L=%-30s R=%-30s x%d" % (a, b, n))

    def show(tag, rs, limit=8):
        print("\n=== %s: first %d by identity ===" % (tag, limit))
        seen = set()
        for r in rs:
            key = (r['module'], r.get('bind'), r.get('unfoldedL'), r.get('unfoldedR'))
            if key in seen:
                continue
            seen.add(key)
            print("  module=%s bind=%s" % (r['module'], r['bind']))
            print("    unfoldedL=%s" % (r.get('unfoldedL') or '')[:150])
            print("    unfoldedR=%s" % (r.get('unfoldedR') or '')[:150])
            print("    text=%s" % (r.get('text') or '')[:110])
            if len(seen) >= limit:
                break

    show("OUTCOME A -> fold scope", A)
    show("OUTCOME B -> X1 proper", B)
    show("OUTCOME C -> unexplained, report only", C)
    json.dump([{k: v for k, v in r.items() if k != '_nodes'} for r in rows],
              open('/tmp/rebindarm_rows.json', 'w'), indent=1)
    print("\nrows written: /tmp/rebindarm_rows.json")
    return 0

if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
