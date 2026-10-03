#!/bin/sh
# read_outcome_recount.sh -- the filesystem_read_outcome_adoption_standing instrument.
#
# Named by the row's RE-DERIVATION recipe in dag/extdeps/filesystem/filesystem_io.dag and
# re-runnable, so the count is never transcribed session output (DESIGN §6).
#
# What it implements, per that recipe:
#   * enumerate Filesystem.Read assignment calls in tracked .dag source (any `name =` binding
#     form: let/node/plain); comment and string-literal text is masked before matching, so
#     prose never counts;
#   * exclude the adoption-standing data declaration itself, so the instrument cannot count
#     its own prose;
#   * classify each binding at identity grain (path:line:binding): converted only when every
#     success, error, and content projection of the bound result is an argument of the modeled
#     fold filesystem_read_outcome (balanced-paren spans); otherwise unconverted;
#   * sites whose projections all feed filesystem_exact_read (the sibling modeled fold) are
#     reported as their own class, exact_read_typed: they are already typed outcomes and are
#     not this row debt (converting them to read_outcome would be a downgrade);
#   * claim fixtures are the dag/test/claim/ prefix;
#   * count unconverted rows and distinct paths.
#
# Calibration (recorded in the PR that checked this tool in):
#   * b21b710d5378387ae0c841f07323292c5d72faba (the baseline the row's count was recorded on):
#     92 sites / 48 files, 0 converted -- exact agreement with the row's recorded baseline.
#   * 6471587dbaa (DP-M5's head): 246 bindings / 31 converted / 8 exact_read_typed / 207 raw
#     unconverted / 98 files with raw-unconverted / 43 claim fixtures. DP-M5's session
#     reported 242/28/205/103/39 from a one-off script that no longer exists and that itself
#     disagreed with the row's b21 baseline by 2 (90/46); the deltas here are the honest
#     reconciliation, not a claim that the two were the same instrument.
#
# Usage: tools/read_outcome_recount.sh [TREE_ROOT]     (default: the repo root of this script)
# Output: TSV to stdout: one line per unconverted site `path<TAB>line<TAB>binding<TAB>fields`,
# then a trailing summary line `SUMMARY<TAB>unconverted=<n><TAB>fixtures=<n>
# <TAB>nonfixtures=<n><TAB>files=<n>`.

set -eu

ROOT="${1:-$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)}"

find "$ROOT" -name .git -prune -o -name target -prune -o -name node_modules -prune -o \
  -name '*.dag' -type f -print | sort | while IFS= read -r file; do
  rel=${file#"$ROOT"/}
  awk -v rel="$rel" '
function mask(line,   out, i, n, c, ins) {
  out = ""; ins = 0; n = length(line)
  for (i = 1; i <= n; i++) {
    c = substr(line, i, 1)
    if (ins) {
      if (c == "\\" && i < n) { out = out "  "; i++; continue }
      if (c == "\"") { ins = 0; out = out "\"" } else { out = out " " }
      continue
    }
    if (c == "\"") { ins = 1; out = out "\""; continue }
    if (c == "/" && i < n && substr(line, i + 1, 1) == "/") { break }
    out = out c
  }
  return out
}
function call_spans(text, name,   spans, m, re, depth, i, n, c, count, tl) {
  spans = ""
  re = name "[ \t]*\\("
  tl = length(text)
  pos = 1
  while (match(substr(text, pos), re)) {
    s = pos + RSTART - 1
    depth = 0; i = s + RLENGTH - 1
    while (i <= tl) {
      c = substr(text, i, 1)
      if (c == "\"") { i++; while (i <= tl && substr(text, i, 1) != "\"") {
        if (substr(text, i, 1) == "\\") i++; i++ } }
      else if (c == "(") depth++
      else if (c == ")") { depth--; if (depth == 0) { spans = spans s "," i " "; break } }
      i++
    }
    pos = s + RLENGTH
  }
  return spans
}
function in_spans(p, spans,   a, b, k, arr, j) {
  j = split(spans, arr, " ")
  for (k = 1; k <= j; k++) {
    split(arr[k], ab, ",")
    if (p >= ab[1] && p <= ab[2]) return 1
  }
  return 0
}
{ lines[NR] = mask($0) }
END {
  # join masked lines, preserving offsets; NUL-free text
  text = ""
  startline[1] = 1
  for (r = 1; r <= NR; r++) { linestart[r] = length(text) + 1; text = text lines[r] "\n" }
  text_len = length(text)

  # exclude the adoption-standing data declaration so the instrument cannot count its own prose
  if (match(text, /\ndata[ \t]+filesystem_read_outcome_adoption_standing\b/) || \
      index(text, "data filesystem_read_outcome_adoption_standing") == 1) {
    ds = RSTART; if (ds == 0) ds = 1
    de = ds
    while (de <= text_len) {
      c = substr(text, de, 1)
      if (de > ds && c != " " && c != "\t" && c != "\n") {
        # column-0 start of the next top-level declaration ends the excluded region
        if (substr(text, de - 1, 1) == "\n") break
      }
      de++
    }
    text = substr(text, 1, ds - 1) sprintf("%*s", de - ds, "") substr(text, de)
    text_len = length(text)
  }

  conv_spans = call_spans(text, "filesystem_read_outcome")
  sib_spans = call_spans(text, "filesystem_exact_read")

  n = 0
  region_key = ""
  while (match(substr(text, n + 1), /Filesystem\.Read[ \t]*\(/)) {
    m_abs = n + RSTART
    m_len = RLENGTH
    # enclosing top-level region: previous column-0 line start .. next column-0 line start
    ra = 1
    k = m_abs
    while (k > 1) { c = substr(text, k - 1, 1); if (c == "\n" && substr(text, k, 1) != " " && substr(text, k, 1) != "\t") { ra = k; break }; k-- }
    rb = text_len + 1
    k = m_abs + m_len
    while (k <= text_len) { if (substr(text, k, 1) == "\n" && k + 1 <= text_len && substr(text, k + 1, 1) != " " && substr(text, k + 1, 1) != "\t" && substr(text, k + 1, 1) != "\n") { rb = k + 1; break }; k++ }

    # fold-call spans relative to this region, computed once per region
    if (ra "," rb != region_key) {
      region_key = ra "," rb
      body = substr(text, ra, rb - ra)
      rconv = call_spans(body, "filesystem_read_outcome")
      rsib = call_spans(body, "filesystem_exact_read")
    }

    # binding? the code before the call on its line must end with `name =`
    ls = m_abs; while (ls > 1 && substr(text, ls - 1, 1) != "\n") ls--
    before = substr(text, ls, m_abs - ls)
    sub(/[ \t]+$/, "", before)
    if (before ~ /(^|[^A-Za-z0-9_])(let[ \t]+|node[ \t]+)?[A-Za-z_][A-Za-z0-9_]*[ \t]*=$/) {
      match(before, /[A-Za-z_][A-Za-z0-9_]*[ \t]*=$/)
      nm = substr(before, RSTART, RLENGTH)
      sub(/[ \t]*=$/, "", nm)

      # every projection use of the binding in the region
      uses = ""; nuses = 0; inconv = 0; insib = 0; fields = ""
      for (f = 1; f <= 4; f++) {
        fld = (f == 1 ? "error_kind" : (f == 2 ? "content" : (f == 3 ? "success" : "error")))
        use_re = "(^|[^A-Za-z0-9_])" nm "[ \t]*\\.[ \t]*" fld "([^A-Za-z0-9_]|$)"
        rest = body
        off = 0
        while (match(rest, use_re)) {
          p = off + RSTART
          nuses++
          uses = uses p " "
          if (in_spans(p, rconv)) inconv++
          if (in_spans(p, rconv) || in_spans(p, rsib)) insib++
          if (index(fields, fld) == 0) fields = (fields == "" ? fld : fields "," fld)
          rest = substr(rest, RSTART + RLENGTH)
          off = off + RSTART + RLENGTH - 1
        }
      }
      # compute 1-based line of the binding
      ln = 1; k = 1
      while (k < m_abs) { if (substr(text, k, 1) == "\n") ln++; k++ }
      if (nuses > 0 && inconv == nuses) continue_skip = 1   # converted: not this row debt
      else if (nuses > 0 && insib == nuses) continue_skip = 1  # exact_read_typed: sibling fold
      else print rel "\t" ln "\t" nm "\t" fields
    }
    n = m_abs + m_len - 1
  }
}
' "$file"
done | awk -F'\t' '
  { print; u++; if ($1 != prev) { files++; prev = $1 }
    if ($1 ~ /^dag\/test\/claim\//) fx++; else nf++ }
  END { printf "SUMMARY\tunconverted=%d\tfixtures=%d\tnonfixtures=%d\tfiles=%d\n", u + 0, fx + 0, nf + 0, files + 0 }
'
