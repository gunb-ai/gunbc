// WHOLE-CORPUS EFFECT PROBE for the record-literal-names-a-union-type refusal
// (`TypeNameCarriesNoRecordConstructor`, floor gap: SystemdRunProperty #13942/#14149).
//
// A measurement, not a gate. It runs in two modes and both print to stdout, line-buffered, so a run
// that is cut short still carries what it measured:
//
//   MODE SCAN (default)  -- walks the SAME whole-tree closure the compile-clean floor gate walks, and
//        answers, without running inference: which declared names in the corpus carry NO record
//        constructor (their connective is Disj, reached through alias expansion), and which literal
//        sites spell `Name {`. That cross product is the SOUND SUPERSET of the sites the refusal can
//        fire on: the refusal requires a name in record-literal position whose declaration is a
//        coproduct, and a record literal in a .dag source is always spelled `Name {`. Anything the
//        refusal could newly refuse is in this list.
//
//   MODE COMPILE (ZZ_FILES=<paths, newline separated> or ZZ_FILES=ALL) -- compiles the named files
//        together with their transitive imports, through the same resolve+infer pipeline the floor
//        gate uses, and prints every diagnostic of every file, by class, gate, severity, located
//        text and message. Run it at the base commit and at the fix commit and diff: a class that
//        appears only at the fix is a NEW refusal; a class whose count drops is a diagnostic the fix
//        ABSORBED (which the refusal must never do -- it is prepended into the concat, not returned
//        early).
//
// WHY NOT ONE WHOLE-CORPUS PASS: the corpus is ~6k .dag files and the cold build plus a single
// whole-tree typecheck exceeded the runner's wall clock, with output only flushed at exit -- a
// measurement that cannot survive being interrupted is not a measurement. The scan is what makes the
// compile list sound rather than guessed: it is derived from the declaration population, not from a
// keyword search for a name I happened to know.
//
// Heavyweight and chdir-global, so #[ignore]d like `witness_layer_roots_compile_clean_check`:
//   cargo test -p v1-compiler --lib union_name_corpus_probe -- --ignored --nocapture --test-threads=1
#![allow(clippy::all, dead_code, unused_imports)]

#[cfg(test)]
mod probe {
    use crate::cli_run::*;
    use crate::v1_std_core::{diagnostic_to_message, diagnostic_to_span};
    use std::collections::{BTreeMap, BTreeSet};
    use std::rc::Rc;

    fn elapsed_secs(started: &std::time::Instant) -> u64 {
        started.elapsed().as_secs()
    }

    fn line_of(content: &str, offset: i64) -> i64 {
        if offset < 0 {
            return -1;
        }
        let bytes = content.as_bytes();
        let end = (offset as usize).min(bytes.len());
        1 + bytes[..end].iter().filter(|b| **b == b'\n').count() as i64
    }

    fn module_of(content: &str) -> String {
        for line in content.lines() {
            let t = line.trim_start();
            if let Some(rest) = t.strip_prefix("module ") {
                return rest
                    .trim()
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .to_string();
            }
        }
        String::new()
    }

    // BLANK THE LINE COMMENTS, WITHOUT TOUCHING STRING LITERALS. Brace depth and the arm separator
    // are read off this copy: a `{` inside a comment (a corpus file documents exactly that case) and
    // a `|` inside a comment are not structure, and a `//` inside a string (a URI) must not truncate a
    // declaration. STRING CONTENTS SURVIVE, because a literal may be spelled inside a source that is
    // itself a string -- the specimen's forged source is a nested compile, and blanking strings would
    // blind the measurement to the very site the row is about.
    fn blank_line_comments(content: &str) -> String {
        let mut out = String::with_capacity(content.len());
        let mut in_string = false;
        let mut escaped = false;
        let mut in_comment = false;
        let chars: Vec<char> = content.chars().collect();
        let mut i = 0usize;
        while i < chars.len() {
            let c = chars[i];
            if in_comment {
                if c == '\n' {
                    in_comment = false;
                    out.push('\n');
                } else {
                    out.push(' ');
                }
                i += 1;
                continue;
            }
            if in_string {
                out.push(c);
                if escaped {
                    escaped = false;
                } else if c == '\\' {
                    escaped = true;
                } else if c == '"' {
                    in_string = false;
                }
                i += 1;
                continue;
            }
            if c == '"' {
                in_string = true;
                out.push(c);
                i += 1;
                continue;
            }
            if c == '/' && chars.get(i + 1) == Some(&'/') {
                in_comment = true;
                out.push(' ');
                out.push(' ');
                i += 2;
                continue;
            }
            out.push(c);
            i += 1;
        }
        out
    }

    // THE POPULATION OF NAMES THAT CARRY NO RECORD CONSTRUCTOR, read from the text.
    //
    // A `type Name ... =` DECLARATION IS READ AS A BLOCK, not a line: the specimen of this floor gap
    // is declared as `type SystemdRunProperty =` with its arms on the following lines, so a
    // line-oriented classifier called it a non-union and the measurement would have missed the very
    // site the row is about. The block runs to the matching close of its braces (an arm may open a
    // brace body), and then: a `|` at brace depth 0 inside the block is a coproduct, a bare name is an
    // alias edge. A name is NOT concluded to be a record from this pass -- that reading (a line-based
    // one, in the version this replaces) is what let the specimen through. Resolve, in MODE COMPILE,
    // is what confirms any candidate; this pass only has to be a SUPERSET, so its bias is to include.
    fn declared_union_and_alias_names(content: &str) -> (BTreeSet<String>, BTreeSet<String>) {
        let mut unions = BTreeSet::new();
        let mut aliases = BTreeSet::new();
        let chars: Vec<char> = content.chars().collect();
        let mut i = 0usize;
        while i + 5 <= chars.len() {
            // a `type` keyword at the start of a line
            let line_start = i == 0 || chars[i - 1] == '\n';
            if line_start
                && chars[i..].starts_with(&['t', 'y', 'p', 'e'][..])
                && chars
                    .get(i + 4)
                    .map_or(false, |c| c.is_whitespace() || *c == ' ')
            {
                // head: up to the '=' of the declaration. THE HEAD MAY BREAK ACROSS LINES -- the
                // specimen is spelled `type SystemdRunProperty` NEWLINE `= NamedUnitSetting { .. }
                // | EmptyCapabilityBoundingSet`, so stopping at a newline classified the union that
                // this whole row is about as no declaration at all. A `{` before any `=` is a record
                // (no constructor question to ask) and ends the scan.
                let mut j = i + 4;
                while j < chars.len() && chars[j] != '=' && chars[j] != '{' {
                    if chars[j] == '\n' && j - i > 200 {
                        break;
                    }
                    j += 1;
                }
                if j < chars.len() && chars[j] == '=' {
                    let head: String = chars[i + 4..j].iter().collect();
                    let name = head
                        .split('<')
                        .next()
                        .unwrap_or("")
                        .trim()
                        .split_whitespace()
                        .last()
                        .unwrap_or("")
                        .to_string();
                    // body: to the close of the braces this declaration opens, or to the next
                    // top-level keyword line if it opens none
                    let mut k = j + 1;
                    let mut depth = 0i64;
                    let mut bar_at_zero = false;
                    let mut body: Vec<char> = Vec::new();
                    while k < chars.len() {
                        let c = chars[k];
                        if c == '{' {
                            depth += 1;
                        } else if c == '}' {
                            depth -= 1;
                            if depth < 0 {
                                break;
                            }
                        } else if depth == 0 {
                            if c == '|' {
                                bar_at_zero = true;
                            } else if c == '\n' {
                                // a new top-level line ends a body that opened no brace
                                let mut p = k + 1;
                                while p < chars.len() && (chars[p] == ' ' || chars[p] == '\t') {
                                    p += 1;
                                }
                                if p < chars.len() && chars[p] != ' ' && chars[p] != '\t' {
                                    let word: String =
                                        chars[p..p + 8.min(chars.len() - p)].iter().collect();
                                    if word.starts_with("type ")
                                        || word.starts_with("fn ")
                                        || word.starts_with("data ")
                                        || word.starts_with("import ")
                                        || word.starts_with("export ")
                                    {
                                        break;
                                    }
                                }
                            }
                        }
                        body.push(c);
                        k += 1;
                    }
                    if !name.is_empty() {
                        let body_text: String = body
                            .iter()
                            .filter(|c| **c != '\r')
                            .collect::<String>()
                            // strip line comments: a `|` in a comment is not an arm
                            .lines()
                            .map(|l| l.split("//").next().unwrap_or("").to_string())
                            .collect::<Vec<String>>()
                            .join("\n");
                        if bar_at_zero {
                            unions.insert(name);
                        } else {
                            let first = body_text
                                .lines()
                                .map(|l| l.trim())
                                .find(|l| !l.is_empty())
                                .unwrap_or("");
                            let target = first.split('<').next().unwrap_or("").trim().to_string();
                            let bare = target.split('.').last().unwrap_or("").to_string();
                            let record_shaped = body_text.contains('{');
                            if !record_shaped
                                && !bare.is_empty()
                                && bare
                                    .chars()
                                    .next()
                                    .map_or(false, |c| c.is_ascii_uppercase())
                            {
                                aliases.insert(format!("{name}>{bare}"));
                            }
                        }
                    }
                    i = k.max(i + 1);
                    continue;
                }
            }
            i += 1;
        }
        (unions, aliases)
    }

    // Every `Name {` / `Name\n{` spelling in the code part of a line, with the name's last segment and
    // the byte offset of the name. Return-type heads (`-> Name {`) and declaration heads (`type Name`,
    // `data Name`, `fn Name`) are excluded: only an EXPRESSION is a literal.
    fn literal_spellings(content: &str) -> Vec<(String, usize)> {
        let mut out = Vec::new();
        for (idx, ch) in content.char_indices() {
            if ch != '{' {
                continue;
            }
            // walk back over whitespace/newlines
            let head = &content[..idx];
            let mut end = head.len();
            while end > 0 {
                let prev = content[..end].chars().last().unwrap_or('\0');
                if prev.is_whitespace() {
                    end -= prev.len_utf8();
                } else {
                    break;
                }
            }
            if end == 0 {
                continue;
            }
            let bytes: Vec<char> = content[..end].chars().collect();
            let mut i = bytes.len();
            // `Name<Arg> {` -- peel the type arguments so the NAME is the candidate, not its argument
            if i > 0 && bytes[i - 1] == '>' {
                let mut d = 0i64;
                while i > 0 {
                    let c = bytes[i - 1];
                    if c == '>' {
                        d += 1;
                    } else if c == '<' {
                        d -= 1;
                        if d == 0 {
                            i -= 1;
                            break;
                        }
                    }
                    i -= 1;
                }
            }
            while i > 0 {
                let c = bytes[i - 1];
                if c.is_ascii_alphanumeric() || c == '_' || c == '.' {
                    i -= 1;
                } else {
                    break;
                }
            }
            let word: String = bytes[i..].iter().collect();
            if word.is_empty() {
                continue;
            }
            // `-> Word {` is a function's return type spelled as a brace body, not a literal.
            let before: String = bytes[..i].iter().collect();
            let before_trim = before.trim_end();
            if before_trim.ends_with("->") || before_trim.ends_with("-<") {
                continue;
            }
            let before_last_word = before_trim
                .rsplit(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                .next()
                .unwrap_or("")
                .to_string();
            if before_last_word == "type"
                || before_last_word == "data"
                || before_last_word == "fn"
                || before_last_word == "import"
            {
                continue;
            }
            let last = word.split('.').last().unwrap_or("").to_string();
            if last
                .chars()
                .next()
                .map_or(false, |c| c.is_ascii_uppercase())
            {
                out.push((last, idx));
            }
        }
        out
    }

    // THE CORPUS IS READ OFF DISK, NOT FROM THE COMPILE-CLEAN CLOSURE. The closure walk builds the
    // whole-tree index, which cost more than the typecheck being measured and would have made the
    // measurement the thing that times out; and a disk walk is a SOUND SUPERSET of the closure -- it
    // also covers modules the floor gate does not enrol, so nothing the refusal can fire on is missed.
    fn corpus_on_disk() -> Vec<(String, String)> {
        let mut out: Vec<(String, String)> = Vec::new();
        let mut stack: Vec<std::path::PathBuf> = vec![std::path::PathBuf::from(".")];
        while let Some(dir) = stack.pop() {
            let Ok(entries) = std::fs::read_dir(&dir) else {
                continue;
            };
            for e in entries.flatten() {
                let p = e.path();
                let name = e.file_name().to_string_lossy().to_string();
                if p.is_dir() {
                    if name == "target" || name == ".git" || name == "node_modules" {
                        continue;
                    }
                    stack.push(p);
                } else if name.ends_with(".dag") {
                    if let Ok(text) = std::fs::read_to_string(&p) {
                        // REPO-RELATIVE, WITHOUT THE LEADING "./" THE WALK ADDS: the probe prints
                        // these paths, and a file list handed back in is matched against them. A
                        // leading "./" made a 141-file selection choose 0 files -- a run that measures
                        // nothing, visible only because the phase lines print both counts.
                        let rel = p.to_string_lossy().to_string();
                        let rel = rel.strip_prefix("./").map(|r| r.to_string()).unwrap_or(rel);
                        out.push((rel, text));
                    }
                }
            }
        }
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    }

    fn imported_modules(content: &str) -> Vec<String> {
        let mut out = Vec::new();
        for line in content.lines() {
            let t = line.trim_start();
            if let Some(rest) = t.strip_prefix("import ") {
                let m = rest
                    .trim()
                    .split('{')
                    .next()
                    .unwrap_or("")
                    .trim()
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .to_string();
                if !m.is_empty() {
                    out.push(m);
                }
            }
        }
        out
    }

    #[test]
    #[ignore = "whole-tree closure walk; run explicitly with --ignored --nocapture"]
    fn union_name_effect() {
        let started = std::time::Instant::now();
        std::env::set_current_dir(crate::cli_run::workspace_root()).expect("anchor");
        // SELECTORS ARRIVE AS TEST ARGUMENTS, NOT ENVIRONMENT: ctrl-build forwards no env to the
        // runner, so an env-var selector silently selected nothing -- a measurement that quietly
        // measures nothing. `-- zz-dump=<substring>` and `-- zz-files=ALL|<path>,<path>`.
        let arg_value = |prefix: &str| -> String {
            std::env::args()
                .find_map(|a| a.strip_prefix(prefix).map(|v| v.to_string()))
                .unwrap_or_default()
        };
        let mode = arg_value("zz-files=");
        let dump = arg_value("zz-dump=");
        println!(
            "PROBE mode={} dump={}",
            if mode.is_empty() { "scan" } else { "compile" },
            dump
        );

        let sources = corpus_on_disk();
        let content_by_path: BTreeMap<String, String> = sources.iter().cloned().collect();
        let module_by_path: BTreeMap<String, String> = sources
            .iter()
            .map(|(p, c)| (p.clone(), module_of(c)))
            .collect();
        println!(
            "PROBE phase=dag-files-done files={} t={}s",
            sources.len(),
            elapsed_secs(&started)
        );

        // ---- the population of names that carry no record constructor, and the alias chains into it
        let mut union_names: BTreeSet<String> = Default::default();
        let mut alias_edges: BTreeMap<String, String> = Default::default();
        for (_, content) in sources.iter() {
            let (u, a) = declared_union_and_alias_names(&blank_line_comments(content));
            union_names.extend(u);
            for pair in a {
                let mut it = pair.split('>');
                let from = it.next().unwrap_or("").to_string();
                let to = it.next().unwrap_or("").to_string();
                if !from.is_empty() && !to.is_empty() {
                    alias_edges.insert(from, to);
                }
            }
        }
        // expand aliases to a fixed point so `type ZzT = ZzU` / `type ZzU = A | B` is judged as a union
        loop {
            let mut grew = false;
            for (from, to) in alias_edges.clone() {
                if !union_names.contains(&from) && union_names.contains(&to) {
                    union_names.insert(from);
                    grew = true;
                }
            }
            if !grew {
                break;
            }
        }
        println!(
            "PROBE phase=population-done union_like_names={} t={}s",
            union_names.len(),
            elapsed_secs(&started)
        );

        // ZZ_DUMP=<substring>: say, for every file whose path contains it, every spelling found and
        // whether that name is in the no-record-constructor population. A measurement of the
        // measurement: it is how the missing SystemdRunProperty site was found.
        if !dump.is_empty() {
            for (path, content) in sources.iter() {
                if !path.contains(&dump) {
                    continue;
                }
                for (last, off) in literal_spellings(content) {
                    println!(
                        "DUMP\t{}\toffset {}\tname={}\tin_population={}",
                        path,
                        off,
                        last,
                        union_names.contains(&last)
                    );
                }
            }
            for name in union_names.iter().filter(|n| n.contains(&dump)) {
                println!("DUMP_POPULATION\t{name}");
            }
        }

        // ---- the candidate literal sites: a `Name {` spelling whose Name is one of those names
        let mut candidate_paths: BTreeSet<String> = Default::default();
        let mut sites = 0usize;
        for (path, content) in sources.iter() {
            let hits: Vec<(String, usize, i64)> = literal_spellings(content)
                .into_iter()
                .filter(|(last, _)| union_names.contains(last))
                .map(|(last, off)| (last, off, line_of(content, off as i64)))
                .collect();
            if hits.is_empty() {
                continue;
            }
            candidate_paths.insert(path.clone());
            for (name, off, line) in hits {
                sites += 1;
                println!(
                    "CANDIDATE\t{}\tline {}\toffset {}\tmodule={}\tname={}",
                    path,
                    line,
                    off,
                    module_by_path.get(path).cloned().unwrap_or_default(),
                    name
                );
            }
        }
        println!(
            "PROBE phase=scan-done candidates={sites} candidate_files={} t={}s",
            candidate_paths.len(),
            elapsed_secs(&started)
        );

        if mode.is_empty() {
            return;
        }

        // MODE PERFILE: run each candidate through the SAME host function the .dag probes call
        // (gunbc.compile_census_probe's `compile_dag_diagnostic_census` builtin), one file at a time.
        // That is the measurement the corpus's own probes make, so the numbers here are the numbers
        // those probes see; the cost is one compile per file, with imports read from disk.
        if mode.trim().starts_with("PERFILE") {
            // `zz-files=PERFILE` takes the scan's own candidates; `zz-files=PERFILE-FILE=<path>`
            // takes a newline/comma list from a file. The measured list travels as a file, not as a
            // shell argument (it is 141 paths), and it is committed next to this probe as
            // union_name_corpus_probe_candidates.txt -- regenerated by SCAN mode, so a reviewer can
            // diff the list the numbers were taken from against the one the scan produces today.
            let list: Vec<String> = if mode.trim() == "PERFILE" {
                candidate_paths.iter().cloned().collect()
            } else if mode.trim().starts_with("PERFILE-CHUNK=") {
                // SHARDING IS PART OF THE MEASUREMENT, not a convenience: the runner caps one invocation
                // at 60 minutes of wall clock, the census costs ~35s a file because each file is compiled
                // with its transitive imports, and a measurement that is killed before it prints prints
                // nothing. `zz-files=PERFILE-CHUNK=2-of-4` takes the 2nd quarter of the committed list.
                let spec = mode.trim().trim_start_matches("PERFILE-CHUNK=");
                let (index, parts) = spec.split_once("-of-").unwrap_or(("1", "1"));
                let index: usize = index.parse().unwrap_or(1).max(1);
                let parts: usize = parts.parse().unwrap_or(1).max(1);
                let all: Vec<String> = candidate_paths.iter().cloned().collect();
                let per = all.len().div_ceil(parts);
                let start = (index - 1) * per;
                all.into_iter().skip(start).take(per).collect()
            } else if let Some(rest) = mode.trim().strip_prefix("PERFILE-FILE=") {
                let text = std::fs::read_to_string(rest)
                    .unwrap_or_else(|e| panic!("probe list {rest}: {e}"));
                text.split([',', '\n'])
                    .map(|p| p.trim().to_string())
                    .filter(|p| !p.is_empty())
                    .collect()
            } else {
                mode.split_once(':')
                    .map(|(_, rest)| {
                        rest.split(',')
                            .map(|p| p.trim().to_string())
                            .filter(|p| !p.is_empty())
                            .collect()
                    })
                    .unwrap_or_default()
            };
            for path in list.iter() {
                let content = match content_by_path.get(path) {
                    Some(c) => c.clone(),
                    std::option::Option::None => {
                        println!("PERFILE_SKIP\t{path}\tnot-in-corpus");
                        continue;
                    }
                };
                let census = crate::cli_run::emit_host::compile_dag_diagnostic_census(&content);
                match census {
                    crate::cli_run::CompileDiagnosticCensus::Observed(rows) => {
                        for row in rows.iter() {
                            println!(
                                "ROW\t{path}\tclass={}\tsubject={}\tblocking={}\tcount={}",
                                row.diagnostic_class, row.subject_name, row.blocking, row.count
                            );
                        }
                        println!("PERFILE_DONE\t{path}\trows={}", rows.len());
                    }
                    crate::cli_run::CompileDiagnosticCensus::NotRunnable(cause) => {
                        println!("PERFILE_NOT_RUNNABLE\t{path}\tcause={cause}");
                    }
                }
            }
            println!(
                "PROBE phase=perfile-done files={} t={}s",
                list.len(),
                elapsed_secs(&started)
            );
            return;
        }

        // ---- MODE COMPILE: the named files plus their transitive imports, through resolve+infer
        let path_by_module: BTreeMap<String, String> = module_by_path
            .iter()
            .filter(|(_, m)| !m.is_empty())
            .map(|(p, m)| (m.clone(), p.clone()))
            .collect();
        let mut selected: BTreeSet<String> = Default::default();
        if mode.trim() == "ALL" {
            selected = candidate_paths.clone();
        } else if let Some(rest) = mode.trim().strip_prefix("LISTFILE=") {
            // the MEASURED candidate list, from a file: 7 KB of paths cannot travel through a shell
            // argument, and a list the measurement carries itself is auditable
            let text =
                std::fs::read_to_string(rest).unwrap_or_else(|e| panic!("probe list {rest}: {e}"));
            for p in text.split([',', '\n']) {
                let p = p.trim();
                if !p.is_empty() {
                    selected.insert(p.to_string());
                }
            }
        } else {
            for p in mode.split(',').map(|l| l.trim()).filter(|l| !l.is_empty()) {
                selected.insert(p.to_string());
            }
            // transitive imports, so the selection resolves
            let mut frontier: Vec<String> = selected.iter().cloned().collect();
            while let Some(p) = frontier.pop() {
                let Some(content) = content_by_path.get(&p) else {
                    continue;
                };
                for m in imported_modules(content) {
                    if let Some(ip) = path_by_module.get(&m) {
                        if selected.insert(ip.clone()) {
                            frontier.push(ip.clone());
                        }
                    }
                }
            }
        }
        let chosen: Vec<Rc<crate::v1_compiler_compile::SourceFile>> = sources
            .iter()
            .filter(|(p, _)| selected.contains(p))
            .map(|(p, c)| {
                Rc::new(crate::v1_compiler_compile::SourceFile {
                    path: p.clone(),
                    content: c.clone(),
                })
            })
            .collect();
        println!(
            "PROBE phase=compile-begin files={} t={}s",
            chosen.len(),
            elapsed_secs(&started)
        );
        let result = crate::v1_compiler_compile::compile_to_resolved(Rc::new(chosen.into()));
        println!(
            "PROBE phase=compile-done raw_diagnostics={} t={}s",
            result.diagnostics.len(),
            elapsed_secs(&started)
        );

        let mut per_file: BTreeMap<String, BTreeMap<String, usize>> = Default::default();
        for d in result.diagnostics.iter() {
            let (class, _) = compile_clean_diagnostic_histogram_key(d);
            let disposition = crate::v1_std_core::diagnostic_disposition(d.diagnostic.clone());
            let severity = match disposition.severity {
                crate::v1_std_core::DiagnosticSeverity::SeverityError => "error",
                crate::v1_std_core::DiagnosticSeverity::SeverityNonError => "non-error",
            };
            let gate = match &*disposition.gate {
                crate::v1_std_core::DiagnosticGateDisposition::GateBlocking => "blocking",
                crate::v1_std_core::DiagnosticGateDisposition::GateAdvisoryTypecheck => {
                    "advisory-typecheck"
                }
                crate::v1_std_core::DiagnosticGateDisposition::GateRenderedUncounted { .. } => {
                    "rendered-uncounted"
                }
            };
            let span = diagnostic_to_span(d.diagnostic.clone());
            let content = content_by_path.get(&span.file).cloned().unwrap_or_default();
            let located = {
                let bytes = content.as_bytes();
                let a = (span.start as usize).min(bytes.len());
                let b = (span.end as usize).min(bytes.len());
                let mut s = String::from_utf8_lossy(&bytes[a.min(b)..b]).to_string();
                if s.len() > 60 {
                    s.truncate(60);
                }
                s.replace('\n', "\\n")
            };
            println!(
                "DIAG\t{}\tline {}\t{}\t{}\t{}\tmodule={}\tlocated=[{}]\t{}",
                span.file,
                line_of(&content, span.start),
                class,
                gate,
                severity,
                d.module_name,
                located.replace('\t', " "),
                diagnostic_to_message(d.diagnostic.clone())
                    .replace('\n', " ")
                    .replace('\t', " "),
            );
            *per_file
                .entry(span.file.clone())
                .or_default()
                .entry(format!("{class}\t{gate}\t{severity}"))
                .or_default() += 1;
        }
        println!("--- PER FILE (path, class, gate, severity, count) ---");
        for (f, m) in &per_file {
            for (k, n) in m {
                println!("PERFILE\t{f}\t{k}\t{n}");
            }
        }
        println!(
            "PROBE phase=done files={} diagnostics={} t={}s",
            selected.len(),
            result.diagnostics.len(),
            elapsed_secs(&started)
        );
    }
}
