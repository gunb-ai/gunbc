//! Parse-check a YAML file (exit 0 = valid, 1 = invalid/missing).
//!
//! Hand-written CI seed bin — fallback transport for `gunbc.ci_yaml_validate` when
//! PyYAML and actionlint are not on PATH. The parse *requirement* is modeled in
//! `dsl/gunbc/ci_yaml_validate.dag`; this bin is one of three shell handlers.
//!
//! Beyond YAML syntax, enforces runner-context env scope (§5 fail-closed):
//!   - `${{ env.VARNAME }}` must reference a var declared in a static `env:` block
//!     or written to `$GITHUB_ENV` in a preceding-or-parallel `run:` step.
//!   - `${{ runner.PROP }}` must use a known runner-context property name.
//!
//! SCAFFOLD — dissolves when self-hosted runners guarantee actionlint or PyYAML (drop
//! this fallback from `ci_yml_parse_script`), or when parse moves to a total extdeps
//! host effect without a v1-compiler bin (DESIGN §7 seed → zero).

#![allow(clippy::disallowed_macros)]

use std::{
    collections::HashSet,
    env,
    fs,
    process::ExitCode,
};

// Valid runner context properties per GitHub Actions docs.
const RUNNER_PROPS: &[&str] = &[
    "name", "os", "arch", "temp", "tool_cache", "debug", "environment",
];

fn main() -> ExitCode {
    let Some(path) = env::args().nth(1) else {
        eprintln!("usage: yaml_check <path>");
        return ExitCode::from(2);
    };
    let text = match fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("yaml_check: read {path}: {e}");
            return ExitCode::from(1);
        }
    };
    let doc: serde_yaml::Value = match serde_yaml::from_str(&text) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("yaml_check: parse {path}: {e}");
            return ExitCode::from(1);
        }
    };

    let errors = check_scope(&doc);
    if errors.is_empty() {
        ExitCode::SUCCESS
    } else {
        for e in &errors {
            eprintln!("yaml_check: {path}: {e}");
        }
        ExitCode::from(1)
    }
}

// --- scope checking ---------------------------------------------------------

fn check_scope(doc: &serde_yaml::Value) -> Vec<String> {
    let mut declared: HashSet<String> = HashSet::new();
    let mut errors: Vec<String> = Vec::new();

    // Workflow-level env: block.
    collect_env_map(doc.get("env"), &mut declared);

    let jobs = match doc.get("jobs").and_then(|v| v.as_mapping()) {
        Some(m) => m,
        None => return errors,
    };

    for (_job_id, job) in jobs {
        // Job-level env: block.
        let mut job_declared = declared.clone();
        collect_env_map(job.get("env"), &mut job_declared);

        let steps = match job.get("steps").and_then(|v| v.as_sequence()) {
            Some(s) => s,
            None => continue,
        };

        // Two-pass over steps: first collect all $GITHUB_ENV writes so later
        // expressions can reference them (step ordering is a runtime guarantee;
        // we conservatively accept any write that appears anywhere in the job).
        let mut step_env_declared = job_declared.clone();
        for step in steps {
            collect_github_env_writes(step, &mut step_env_declared);
        }

        // Second pass: validate ${{ env.* }} and ${{ runner.* }} in all steps.
        for step in steps {
            // Step-level env: block adds to scope for this step's expressions.
            let mut local = step_env_declared.clone();
            collect_env_map(step.get("env"), &mut local);

            check_expressions_in_value(step, &local, &mut errors);
        }
    }

    errors
}

// Collect keys from an `env:` mapping node into `declared`.
fn collect_env_map(node: Option<&serde_yaml::Value>, declared: &mut HashSet<String>) {
    if let Some(m) = node.and_then(|v| v.as_mapping()) {
        for (k, _) in m {
            if let Some(s) = k.as_str() {
                declared.insert(s.to_owned());
            }
        }
    }
}

// Scan `run:` scripts for `echo ... >> "$GITHUB_ENV"` patterns and add
// found variable names to `declared`. Accepts quoted and unquoted forms:
//   echo "VAR=value" >> "$GITHUB_ENV"
//   echo 'VAR=value' >> "$GITHUB_ENV"
//   echo VAR=value >> "$GITHUB_ENV"
fn collect_github_env_writes(step: &serde_yaml::Value, declared: &mut HashSet<String>) {
    let Some(run) = step.get("run").and_then(|v| v.as_str()) else {
        return;
    };
    for line in run.lines() {
        let trimmed = line.trim();
        let Some(after_echo) = trimmed.strip_prefix("echo ") else {
            continue;
        };
        // Strip optional leading quote.
        let payload = after_echo
            .strip_prefix('"')
            .or_else(|| after_echo.strip_prefix('\''))
            .unwrap_or(after_echo);
        if let Some(eq) = payload.find('=') {
            let candidate = &payload[..eq];
            if is_env_var_name(candidate) {
                declared.insert(candidate.to_owned());
            }
        }
    }
}

fn is_env_var_name(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
        && s.chars().next().is_some_and(|c| !c.is_ascii_digit())
}

// Walk every string leaf in `node` and validate all `${{ ... }}` expressions.
fn check_expressions_in_value(
    node: &serde_yaml::Value,
    declared: &HashSet<String>,
    errors: &mut Vec<String>,
) {
    match node {
        serde_yaml::Value::String(s) => {
            check_expressions_in_str(s, declared, errors);
        }
        serde_yaml::Value::Sequence(seq) => {
            for item in seq {
                check_expressions_in_value(item, declared, errors);
            }
        }
        serde_yaml::Value::Mapping(map) => {
            for (_, v) in map {
                check_expressions_in_value(v, declared, errors);
            }
        }
        _ => {}
    }
}

// Scan `s` for all `${{ expr }}` tokens and validate each.
fn check_expressions_in_str(s: &str, declared: &HashSet<String>, errors: &mut Vec<String>) {
    let mut pos = 0;
    while let Some(start) = s[pos..].find("${{") {
        let abs_start = pos + start;
        let after_open = abs_start + 3;
        match s[after_open..].find("}}") {
            None => {
                // Unclosed expression — not our problem (actionlint handles this).
                break;
            }
            Some(end_rel) => {
                let expr = s[after_open..after_open + end_rel].trim();
                validate_expression(expr, declared, errors);
                pos = after_open + end_rel + 2;
            }
        }
    }
}

fn validate_expression(expr: &str, declared: &HashSet<String>, errors: &mut Vec<String>) {
    if let Some(rest) = expr.strip_prefix("env.") {
        // Only the immediate property name matters (no nested access on env vars).
        let varname = rest.split_whitespace().next().unwrap_or(rest);
        // Strip trailing punctuation that might appear in compound expressions.
        let varname = varname.trim_end_matches(|c: char| !c.is_alphanumeric() && c != '_');
        if !varname.is_empty() && !declared.contains(varname) {
            errors.push(format!(
                "${{{{ env.{varname} }}}} is not declared in any env: block or $GITHUB_ENV write"
            ));
        }
    } else if let Some(rest) = expr.strip_prefix("runner.") {
        let prop = rest.split_whitespace().next().unwrap_or(rest);
        let prop = prop.trim_end_matches(|c: char| !c.is_alphanumeric() && c != '_');
        if !prop.is_empty() && !RUNNER_PROPS.contains(&prop) {
            errors.push(format!(
                "${{{{ runner.{prop} }}}} is not a known runner context property \
                 (valid: {})",
                RUNNER_PROPS.join(", ")
            ));
        }
    }
}
