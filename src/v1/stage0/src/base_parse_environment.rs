//! THE PARSE ENVIRONMENT OF A REVISION THAT IS NOT THE RUNNING BINARY'S.
//!
//! `ParseEnvironment` (`std.syntax`) is threaded through the tokenizer and parser so that reading a
//! revision's source does not mean reading it under whatever grammar this binary was built with.
//! That thread is useless until something can PRODUCE an environment other than the compiled-in
//! `dag_parse_environment`, and this module is that producer: it reads the environment's declaring
//! closure at a git revision and evaluates it, so a base revision is read under the base
//! revision's own grammar.
//!
//! WHY THIS EXISTS: the class is
//! `gunbc.recurring_failure_mode.base_readability_gate_refuses_a_grammar_change`. A gate that
//! compares base-side declarations against head-side ones parses both with one compiler, and has
//! no representable arm for "the base is well formed under its OWN grammar and unreadable only
//! under the head's" -- so a change that edits the grammar itself is refused in proportion to how
//! thoroughly it succeeded.
//!
//! DECODE ROUTE, AND WHY IT IS NOT A HAND-WRITTEN DECODER. An evaluated data item is an
//! interpreter `Value`; the tokenizer needs a typed `ParseEnvironment`. The route taken is
//! `Value` -> `value_to_wire_json` -> `serde_json::from_value`, because the wire encoder resolves
//! its tag policy from `v1_compiler_emit_rust` -- the same policy that emitted the `#[serde(...)]`
//! attributes on the mirror struct. Encoder and decoder therefore cannot disagree about shape
//! unless the emitter disagrees with itself. A hand-written `Value` -> `ParseEnvironment` decoder
//! would be a second authority for the type's shape (section 3) across nine types, and would drift
//! silently the first time a field was added to `SyntaxSpec`.
//!
//! WHAT IT DOES NOT COVER, stated because the boundary is the value of the type. This reproduces
//! the DECLARATIVE environment of the base revision: which words are keywords, which item forms
//! exist, which operators bind how. It does NOT reproduce the base revision's PARSER. Body parsers
//! are dispatched on `body_kind` to hand-written code compiled into this binary, so a base whose
//! body parser behaved differently is not reproduced by supplying its environment, and must not be
//! claimed to be. That population stays outside the covered set.

use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

use crate::std_syntax::ParseEnvironment;

/// The module whose declarations ARE the dag realization's parse environment.
const ENVIRONMENT_MODULE: &str = "extdeps.languages.dag.syntax";
/// The data item within it that carries the environment value.
const ENVIRONMENT_ITEM: &str = "dag_parse_environment";
/// Where the corpus of `.dag` declarations lives, relative to the repository root.
const DAG_SOURCE_ROOT: &str = "dag";

/// Why an environment could not be produced for a revision.
///
/// EVERY ARM IS A REFUSAL, NEVER A SUBSTITUTION. The tempting arm when a base environment cannot
/// be read is to fall back to the head's -- which is precisely the assumption this module exists to
/// remove, and would fail open exactly on the changes that alter the grammar. Section 5's absorbing
/// fallback in its purest form: nothing is missed, so the arm reads as safe, while the only signal
/// that the base was unreadable is destroyed. So the failure is typed and located, and the caller
/// decides what an unreadable base means for its own verdict.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnvironmentLoadRefusal {
    /// A module in the closure has no file at this revision.
    ModuleMissing { module: String, revision: String },
    /// Two files at this revision declare the same module.
    ModuleDuplicated {
        module: String,
        paths: Vec<String>,
        revision: String,
    },
    /// A file in the closure could not be read out of the object store.
    BlobUnreadable {
        path: String,
        revision: String,
        cause: String,
    },
    /// The closure was read, but the corpus did not resolve or evaluate.
    ClosureNotEvaluable { revision: String, cause: String },
    /// The item evaluated, but its value did not decode into the typed environment.
    ///
    /// This is the arm that fires on emitted-schema drift -- a field the wire encoder omits that
    /// the mirror struct requires. It is separated from `ClosureNotEvaluable` because the two have
    /// different owners: that one is a defect in the revision being read, this one is a defect in
    /// THIS binary's schema agreement with its own emitter.
    ValueNotDecodable { revision: String, cause: String },
}

impl std::fmt::Display for EnvironmentLoadRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ModuleMissing { module, revision } => write!(
                f,
                "module `{module}` has no file at revision {revision}, so that revision's parse \
                 environment cannot be read"
            ),
            Self::ModuleDuplicated {
                module,
                paths,
                revision,
            } => write!(
                f,
                "module `{module}` is declared by {} files at revision {revision} ({}), so which \
                 one carries the parse environment is undecided",
                paths.len(),
                paths.join(", ")
            ),
            Self::BlobUnreadable {
                path,
                revision,
                cause,
            } => write!(f, "{path} is unreadable at revision {revision}: {cause}"),
            Self::ClosureNotEvaluable { revision, cause } => write!(
                f,
                "the parse environment closure at revision {revision} did not evaluate: {cause}"
            ),
            Self::ValueNotDecodable { revision, cause } => write!(
                f,
                "the parse environment at revision {revision} evaluated but did not decode into \
                 this binary's `ParseEnvironment`: {cause}"
            ),
        }
    }
}

/// One `.dag` source read out of the object store.
#[derive(Debug, Clone)]
pub struct RevisionSource {
    pub path: String,
    pub module: String,
    pub content: String,
}

/// The module name a `.dag` source declares, if its first non-empty line is a module header.
///
/// Deliberately the same shape the module index uses: a file whose header cannot be read is not
/// silently skipped, it simply does not enter the closure under any name, and the module that
/// imported it then refuses as `ModuleMissing` -- a located refusal rather than a quiet absence.
fn declared_module(content: &str) -> Option<String> {
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        return line.strip_prefix("module ").map(|m| m.trim().to_string());
    }
    None
}

/// The module names a `.dag` source imports.
fn imported_modules(content: &str) -> Vec<String> {
    content
        .lines()
        .filter_map(|line| {
            let rest = line.strip_prefix("import ")?;
            let name: String = rest
                .trim()
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '.')
                .collect();
            if name.is_empty() {
                None
            } else {
                Some(name)
            }
        })
        .collect()
}

/// Every `.dag` path under the source root at a revision, with its declared module.
///
/// Read from the object store rather than the worktree: the point of this module is to read a
/// revision that is NOT checked out, and a worktree read would silently answer about the head.
fn revision_module_map(
    revision: &str,
) -> Result<BTreeMap<String, Vec<String>>, EnvironmentLoadRefusal> {
    let listing = std::process::Command::new("git")
        .args(["ls-tree", "-r", "--name-only", revision, DAG_SOURCE_ROOT])
        .output()
        .map_err(|e| EnvironmentLoadRefusal::BlobUnreadable {
            path: DAG_SOURCE_ROOT.to_string(),
            revision: revision.to_string(),
            cause: format!("git ls-tree failed to start: {e}"),
        })?;
    if !listing.status.success() {
        return Err(EnvironmentLoadRefusal::BlobUnreadable {
            path: DAG_SOURCE_ROOT.to_string(),
            revision: revision.to_string(),
            cause: String::from_utf8_lossy(&listing.stderr).trim().to_string(),
        });
    }
    let mut by_module: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for path in String::from_utf8_lossy(&listing.stdout).lines() {
        if !path.ends_with(".dag") {
            continue;
        }
        let content = read_blob(revision, path)?;
        if let Some(module) = declared_module(&content) {
            by_module.entry(module).or_default().push(path.to_string());
        }
    }
    Ok(by_module)
}

/// One file's bytes at a revision.
fn read_blob(revision: &str, path: &str) -> Result<String, EnvironmentLoadRefusal> {
    let out = std::process::Command::new("git")
        .arg("show")
        .arg(format!("{revision}:{path}"))
        .output()
        .map_err(|e| EnvironmentLoadRefusal::BlobUnreadable {
            path: path.to_string(),
            revision: revision.to_string(),
            cause: format!("git show failed to start: {e}"),
        })?;
    if !out.status.success() {
        return Err(EnvironmentLoadRefusal::BlobUnreadable {
            path: path.to_string(),
            revision: revision.to_string(),
            cause: String::from_utf8_lossy(&out.stderr).trim().to_string(),
        });
    }
    String::from_utf8(out.stdout).map_err(|e| EnvironmentLoadRefusal::BlobUnreadable {
        path: path.to_string(),
        revision: revision.to_string(),
        cause: format!("not valid UTF-8: {e}"),
    })
}

/// The transitive import closure of the environment module at a revision.
///
/// WALKED, NOT LISTED. A hardcoded roster of the closure's members would be a second authority for
/// what the environment depends on, and would go stale the first time `std.syntax` gained an
/// import -- silently, because a stale roster still resolves. The walk reads each module's own
/// `import` lines, so the closure is whatever the revision says it is.
pub fn environment_closure_at(
    revision: &str,
) -> Result<Vec<RevisionSource>, EnvironmentLoadRefusal> {
    let by_module = revision_module_map(revision)?;
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut queue = vec![ENVIRONMENT_MODULE.to_string()];
    let mut sources = Vec::new();
    while let Some(module) = queue.pop() {
        if !seen.insert(module.clone()) {
            continue;
        }
        let paths =
            by_module
                .get(&module)
                .ok_or_else(|| EnvironmentLoadRefusal::ModuleMissing {
                    module: module.clone(),
                    revision: revision.to_string(),
                })?;
        if paths.len() > 1 {
            return Err(EnvironmentLoadRefusal::ModuleDuplicated {
                module: module.clone(),
                paths: paths.clone(),
                revision: revision.to_string(),
            });
        }
        let path = &paths[0];
        let content = read_blob(revision, path)?;
        for imported in imported_modules(&content) {
            if !seen.contains(&imported) {
                queue.push(imported);
            }
        }
        sources.push(RevisionSource {
            path: path.clone(),
            module,
            content,
        });
    }
    Ok(sources)
}

/// Decode an evaluated environment value into this binary's `ParseEnvironment`.
///
/// Split out from the revision path so the decode seam can be exercised against an independent
/// oracle -- the compiled-in `dag_parse_environment()` -- without a git revision in the way. A
/// failure here is a schema disagreement inside this binary, not a fact about any revision.
pub fn decode_environment_value(
    value: &crate::v1_interpreter::Value,
    ctx: &crate::v1_interpreter::InterpContext,
    revision: &str,
) -> Result<Rc<ParseEnvironment>, EnvironmentLoadRefusal> {
    let wire = crate::cli_run::value_to_wire_json(value, ctx).map_err(|e| {
        EnvironmentLoadRefusal::ValueNotDecodable {
            revision: revision.to_string(),
            cause: format!("wire-encode: {e}"),
        }
    })?;
    serde_json::from_value::<ParseEnvironment>(wire)
        .map(Rc::new)
        .map_err(|e| EnvironmentLoadRefusal::ValueNotDecodable {
            revision: revision.to_string(),
            cause: e.to_string(),
        })
}

/// Evaluate `dag_parse_environment` out of a materialized corpus rooted at `root`.
///
/// `root` is a directory containing a `dag/` tree; the caller owns its lifetime. Materializing to a
/// directory rather than feeding sources in memory is deliberate: the module index and entry
/// resolver are the SAME ones the compiler uses on the live tree, so the base corpus goes through
/// the identical route rather than a second, base-only ingestion path that could diverge.
pub fn evaluate_environment_in(
    root: &std::path::Path,
    revision: &str,
) -> Result<Rc<ParseEnvironment>, EnvironmentLoadRefusal> {
    let dag_root = root.join(DAG_SOURCE_ROOT);
    let entry = dag_root.join("extdeps/languages/dag/syntax.dag");
    let index = crate::cli_run::build_multi_entry_index(&[dag_root.display().to_string()]);
    let (graph, indices) = crate::cli_run::resolve_entry_with_index_for_discovery_corpus(
        &index,
        &entry.display().to_string(),
    )
    .map_err(|e| EnvironmentLoadRefusal::ClosureNotEvaluable {
        revision: revision.to_string(),
        cause: e,
    })?;
    let ctx = crate::cli_run::make_eval_context(
        &graph,
        indices,
        crate::v1_interpreter::ExecutionMode::Wet,
    );
    let value = crate::v1_interpreter::with_active_context(&ctx, || {
        crate::v1_interpreter::eval_data_item_value(&ctx, ENVIRONMENT_ITEM)
    })
    .map_err(|e| EnvironmentLoadRefusal::ClosureNotEvaluable {
        revision: revision.to_string(),
        cause: format!("eval {ENVIRONMENT_ITEM}: {e}"),
    })?
    .ok_or_else(|| EnvironmentLoadRefusal::ClosureNotEvaluable {
        revision: revision.to_string(),
        cause: format!("{ENVIRONMENT_ITEM} is not a data item in `{ENVIRONMENT_MODULE}`"),
    })?;
    decode_environment_value(&value, &ctx, revision)
}
