//! SetSpec implementations for gistgen operations.
//!
//! Each type declares its expected behavior for 0/1/N/null cardinalities.
//! Tests are generated automatically from these declarations.
//!
//! ## Design
//!
//! All types can be viewed through set semantics:
//!
//! | Type Category | Cardinality | SetSpec Cases |
//! |---------------|-------------|---------------|
//! | Non-nullable scalar (`String`, `Bool`) | always 1 | `One` only |
//! | Optional scalar (`Option<T>`) | 0..1 | `One`, `Null` |
//! | Collection (`StrList`, `MapStrStr`) | 0..N | `Zero`, `One`, `N` |
//! | Optional collection | 0..N | `Zero`, `One`, `N`, `Null` |
//!
//! ## Goal: Catch Real Integration Bugs
//!
//! SetSpec proves that adjacent nodes speak the same language:
//!
//! ```text
//! Node A outputs:  can produce {Zero, One, N}
//! Node B expects:  requires {One, N} (rejects Zero)
//!                               ↓
//! Composition test: A.Zero → B = ERROR (expected by design)
//! ```

use std::collections::{BTreeMap, HashMap};

use gunbc_exec::Value;
use gunbc_ir::Secret;
use gunbc_test::{
    AcceptsSpec, Cardinality, ProducesCase, ProducesSpec, SetSpec, SetSpecCase, SetSpecOutput,
};

// =============================================================================
// RepoFiles: enumerate + filter + read pipeline
// =============================================================================

/// SetSpec for the file enumeration/filtering pipeline.
/// Declares what outputs to expect for each input cardinality.
pub struct RepoFilesSpec;

impl SetSpec for RepoFilesSpec {
    fn cases() -> Vec<SetSpecCase> {
        vec![
            // Zero files in repo
            SetSpecCase {
                cardinality: Cardinality::Zero,
                inputs: repo_with_files(vec![]),
                expected: SetSpecOutput::ok([
                    ("files", Value::StrList(vec![])),
                    ("contents", Value::MapStrStr(BTreeMap::new())),
                ]),
            },
            // One file in repo
            SetSpecCase {
                cardinality: Cardinality::One,
                inputs: repo_with_files(vec![("a.txt", "alpha")]),
                expected: SetSpecOutput::ok([
                    ("files", Value::StrList(vec!["a.txt".into()])),
                    ("contents", Value::MapStrStr(btree([("a.txt", "alpha")]))),
                ]),
            },
            // N files in repo
            SetSpecCase {
                cardinality: Cardinality::N,
                inputs: repo_with_files(vec![("a.txt", "alpha"), ("b.md", "beta")]),
                expected: SetSpecOutput::ok([
                    ("files", Value::StrList(vec!["a.txt".into(), "b.md".into()])),
                    ("contents", Value::MapStrStr(btree([("a.txt", "alpha"), ("b.md", "beta")]))),
                ]),
            },
            // Null repo path
            SetSpecCase {
                cardinality: Cardinality::Null,
                inputs: HashMap::new(), // missing repo input
                expected: SetSpecOutput::err("repo"),
            },
        ]
    }

    fn set_port() -> Option<&'static str> {
        Some("contents")
    }
}

// =============================================================================
// GistApi: format + call + parse + extract pipeline
// =============================================================================

/// SetSpec for the Gist API pipeline.
pub struct GistApiSpec;

impl SetSpec for GistApiSpec {
    fn cases() -> Vec<SetSpecCase> {
        vec![
            // Zero files -> error (can't create empty gist)
            SetSpecCase {
                cardinality: Cardinality::Zero,
                inputs: gist_request_with_files(BTreeMap::new()),
                expected: SetSpecOutput::err("files"),
            },
            // One file -> success
            SetSpecCase {
                cardinality: Cardinality::One,
                inputs: gist_request_with_files(btree([("a.txt", "alpha")])),
                expected: SetSpecOutput::ok([
                    ("gist_url", Value::Str("https://gist.github.com/mock/".into())),
                ]),
            },
            // N files -> success
            SetSpecCase {
                cardinality: Cardinality::N,
                inputs: gist_request_with_files(btree([("a.txt", "alpha"), ("b.md", "beta")])),
                expected: SetSpecOutput::ok([
                    ("gist_url", Value::Str("https://gist.github.com/mock/".into())),
                ]),
            },
            // Null/missing request -> error
            SetSpecCase {
                cardinality: Cardinality::Null,
                inputs: HashMap::new(),
                expected: SetSpecOutput::err("request"),
            },
        ]
    }

    fn set_port() -> Option<&'static str> {
        Some("request")
    }
}

// =============================================================================
// Auth: check + create + resolve upsert pattern
// =============================================================================

/// SetSpec for the auth upsert pattern.
pub struct AuthSpec;

impl SetSpec for AuthSpec {
    fn cases() -> Vec<SetSpecCase> {
        vec![
            // Token exists (check succeeds) -> resolve uses check_token
            SetSpecCase {
                cardinality: Cardinality::One,
                inputs: auth_with_token(Some("existing_token")),
                expected: SetSpecOutput::ok([
                    ("token", Value::Secret(Secret("existing_token".into()))),
                ]),
            },
            // No token (check fails, create runs) -> resolve uses create_token
            SetSpecCase {
                cardinality: Cardinality::Zero,
                inputs: auth_with_token(None),
                expected: SetSpecOutput::ok([
                    ("token", Value::Secret(Secret("created_token".into()))),
                ]),
            },
        ]
    }

    fn set_port() -> Option<&'static str> {
        Some("token")
    }
}

// =============================================================================
// Helpers
// =============================================================================

fn repo_with_files(files: Vec<(&str, &str)>) -> HashMap<String, Value> {
    let mut inputs = HashMap::new();
    let file_list: Vec<String> = files.iter().map(|(name, _)| name.to_string()).collect();
    let contents: BTreeMap<String, String> = files
        .into_iter()
        .map(|(name, content)| (name.to_string(), content.to_string()))
        .collect();
    inputs.insert("files".into(), Value::StrList(file_list));
    inputs.insert("contents".into(), Value::MapStrStr(contents));
    inputs
}

fn gist_request_with_files(files: BTreeMap<String, String>) -> HashMap<String, Value> {
    use serde_json::json;

    let files_json: serde_json::Map<String, serde_json::Value> = files
        .into_iter()
        .map(|(name, content)| (name, json!({ "content": content })))
        .collect();

    let request = json!({
        "description": "test gist",
        "public": false,
        "files": files_json,
    });

    let mut inputs = HashMap::new();
    inputs.insert("request".into(), Value::Str(request.to_string()));
    inputs.insert("token".into(), Value::Secret(Secret("test_token".into())));
    inputs
}

fn auth_with_token(token: Option<&str>) -> HashMap<String, Value> {
    let mut inputs = HashMap::new();
    match token {
        Some(t) => {
            inputs.insert("check_token".into(), Value::Secret(Secret(t.into())));
            inputs.insert("create_token".into(), Value::Skipped);
        }
        None => {
            inputs.insert("check_token".into(), Value::Skipped);
            inputs.insert("create_token".into(), Value::Secret(Secret("created_token".into())));
        }
    }
    inputs
}

fn btree<const N: usize>(pairs: [(&str, &str); N]) -> BTreeMap<String, String> {
    pairs
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

// =============================================================================
// ProducesSpec / AcceptsSpec: Composition-based bug detection
// =============================================================================

/// ProducesSpec for ReadFiles operation.
///
/// ReadFiles takes a file list and produces a map of contents.
/// It preserves cardinality: empty list → empty map, one file → one entry, etc.
pub struct ReadFilesProduces;

impl ProducesSpec for ReadFilesProduces {
    fn produces() -> Vec<(Cardinality, ProducesCase)> {
        vec![
            (Cardinality::Zero, ProducesCase::Ok(Cardinality::Zero)),   // empty list → empty map
            (Cardinality::One, ProducesCase::Ok(Cardinality::One)),    // one file → one entry
            (Cardinality::N, ProducesCase::Ok(Cardinality::N)),        // N files → N entries
            (Cardinality::Null, ProducesCase::Err),                    // missing input → error
        ]
    }

    fn name() -> &'static str {
        "ReadFiles"
    }
}

/// AcceptsSpec for ReadFiles operation.
///
/// ReadFiles accepts any file list cardinality, but rejects null input.
pub struct ReadFilesAccepts;

impl AcceptsSpec for ReadFilesAccepts {
    fn accepts() -> Vec<Cardinality> {
        vec![Cardinality::Zero, Cardinality::One, Cardinality::N]
    }

    fn rejects() -> Vec<Cardinality> {
        vec![Cardinality::Null]
    }

    fn name() -> &'static str {
        "ReadFiles"
    }
}

/// ProducesSpec for BuildGistRequest operation.
///
/// BuildGistRequest takes a map of file contents and produces a JSON request string.
/// It rejects empty maps (can't create gist with no files).
pub struct BuildGistRequestProduces;

impl ProducesSpec for BuildGistRequestProduces {
    fn produces() -> Vec<(Cardinality, ProducesCase)> {
        vec![
            (Cardinality::Zero, ProducesCase::Err),                    // empty map → error
            (Cardinality::One, ProducesCase::Ok(Cardinality::One)),    // one file → valid request
            (Cardinality::N, ProducesCase::Ok(Cardinality::One)),      // N files → valid request (singular output)
            (Cardinality::Null, ProducesCase::Err),                    // missing input → error
        ]
    }

    fn name() -> &'static str {
        "BuildGistRequest"
    }
}

/// AcceptsSpec for BuildGistRequest operation.
///
/// BuildGistRequest requires at least one file. Rejects Zero and Null.
pub struct BuildGistRequestAccepts;

impl AcceptsSpec for BuildGistRequestAccepts {
    fn accepts() -> Vec<Cardinality> {
        vec![Cardinality::One, Cardinality::N]
    }

    fn rejects() -> Vec<Cardinality> {
        vec![Cardinality::Zero, Cardinality::Null]
    }

    fn name() -> &'static str {
        "BuildGistRequest"
    }
}

/// ProducesSpec for EnumerateFiles operation.
///
/// EnumerateFiles walks a directory and produces a list of file paths.
/// It can produce any cardinality depending on what's in the directory.
pub struct EnumerateFilesProduces;

impl ProducesSpec for EnumerateFilesProduces {
    fn produces() -> Vec<(Cardinality, ProducesCase)> {
        vec![
            (Cardinality::Zero, ProducesCase::Ok(Cardinality::Zero)),   // empty dir
            (Cardinality::One, ProducesCase::Ok(Cardinality::One)),    // one file
            (Cardinality::N, ProducesCase::Ok(Cardinality::N)),        // many files
            (Cardinality::Null, ProducesCase::Err),                    // missing repo path
        ]
    }

    fn name() -> &'static str {
        "EnumerateFiles"
    }
}

/// ProducesSpec for FilterFiles operation.
///
/// FilterFiles takes a list and a glob, returns matching files.
/// It preserves or reduces cardinality (never increases).
pub struct FilterFilesProduces;

impl ProducesSpec for FilterFilesProduces {
    fn produces() -> Vec<(Cardinality, ProducesCase)> {
        vec![
            // Empty input → empty output
            (Cardinality::Zero, ProducesCase::Ok(Cardinality::Zero)),
            // One file that matches → One, or that doesn't → Zero
            // Conservative: can produce Zero or One
            (Cardinality::One, ProducesCase::Ok(Cardinality::One)),
            // N files, some may not match → could be Zero, One, or N
            // Conservative: can produce any of Zero/One/N
            (Cardinality::N, ProducesCase::Ok(Cardinality::N)),
            (Cardinality::Null, ProducesCase::Err),
        ]
    }

    fn name() -> &'static str {
        "FilterFiles"
    }
}

/// AcceptsSpec for FilterFiles operation.
pub struct FilterFilesAccepts;

impl AcceptsSpec for FilterFilesAccepts {
    fn accepts() -> Vec<Cardinality> {
        vec![Cardinality::Zero, Cardinality::One, Cardinality::N]
    }

    fn rejects() -> Vec<Cardinality> {
        vec![Cardinality::Null]
    }

    fn name() -> &'static str {
        "FilterFiles"
    }
}

/// ProducesSpec for GistApi (format + call + parse + extract pipeline).
///
/// GistApi takes a request JSON and produces a gist URL.
pub struct GistApiProduces;

impl ProducesSpec for GistApiProduces {
    fn produces() -> Vec<(Cardinality, ProducesCase)> {
        vec![
            (Cardinality::Zero, ProducesCase::Err),                    // empty request → error
            (Cardinality::One, ProducesCase::Ok(Cardinality::One)),    // valid request → URL
            (Cardinality::N, ProducesCase::Ok(Cardinality::One)),      // valid request → URL
            (Cardinality::Null, ProducesCase::Err),                    // missing request → error
        ]
    }

    fn name() -> &'static str {
        "GistApi"
    }
}

/// AcceptsSpec for GistApi.
pub struct GistApiAccepts;

impl AcceptsSpec for GistApiAccepts {
    fn accepts() -> Vec<Cardinality> {
        vec![Cardinality::One, Cardinality::N]
    }

    fn rejects() -> Vec<Cardinality> {
        vec![Cardinality::Zero, Cardinality::Null]
    }

    fn name() -> &'static str {
        "GistApi"
    }
}
