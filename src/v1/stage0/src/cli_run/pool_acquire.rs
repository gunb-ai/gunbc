//! ONE ACQUISITION OF A SOURCE FILE, NOT ONE PER WALK.
//!
//! Four whole-tree walks stand between `gunbc compile` and a parsed closure --
//! `entry_resolve::build_module_path_index_uncached`, `entry_resolve::reference_resolution_facts`,
//! `extend_sources_to_both_closure_fixpoint`, and the census-only fill. Each one independently
//! read, tokenized and newline-indexed the same file, so on the seed closure the 3,031-file tree
//! was tokenized 12,121 times: measured `distinct_file_spellings=3031`, of which 3,030 were
//! tokenized exactly 4x.
//!
//! That multiplicity is the whole content of this module. Lexing a file is a PURE FUNCTION of its
//! bytes and the file spelling those bytes are reported under -- there is one right answer, so
//! there is one authority for it, and the walks ask rather than each recompute (DESIGN §2:
//! duplicated work loses on cost, safety and complexity at once).
//!
//! WHAT THIS DELIBERATELY DOES NOT DO. It does not merge the walks, unify their collectors
//! (`collect_dag_files` sorts per directory, `collect_dag_files_tolerant` sorts globally), unify
//! their path keys, or touch their refusal policies. Those differ, and unifying them would change
//! which file wins a duplicate module path -- a behaviour decision, not a cleanup. Every walk
//! keeps its own order, key and policy and simply stops re-lexing what another walk already lexed.
//!
//! WHY THE KEY IS (SPELLING, CONTENT) AND NOT A PATH. A memo keyed on a path asserts that the
//! file has not changed, which is a claim about the world this process cannot make -- that is the
//! `cache_impurity` failure mode. The key here is the ACTUAL INPUT: the spelling, plus a hash of
//! the exact bytes. A file rewritten mid-run hashes differently and is re-lexed; nothing goes
//! stale, because nothing is keyed on an identity weaker than the input itself. The spelling is
//! part of the key because `tokenize` bakes it into every span, so two spellings of one file are
//! two different answers and must not share an entry.

// CLIPPY ROSTER -- 1 finding(s) this module trips today, listed one lint per line with
// its count. Until this commit the generated crate root allowed `clippy::all` plus six
// rustc groups on behalf of every module under it, so `cargo clippy --all-targets -- -D
// warnings` decided nothing here; the root now excuses only the generated modules it
// speaks for (v1.compiler.emit_rust generated_rust_lint_relaxations), and this is what
// that leaves visible. The list is MONOTONE NON-INCREASING: a name leaves when its last
// site is repaired, and a lint not named below reds the build, which is the whole point.
#![allow(
    dead_code,  // 1
)]
// cli_run.rs is this module's PARENT, and an `#![allow]` there reaches every module
// under it -- the same cascade this commit removed at the crate root, one level down.
// These are the names its roster carries that this module does not trip, restored to
// warn so `-D warnings` still judges them here. A name moves from this list to the
// allow list above only with a counted site, never silently.
#![warn(
    clippy::assertions_on_constants,
    clippy::clone_on_copy,
    clippy::cloned_ref_to_slice_refs,
    clippy::collapsible_str_replace,
    clippy::disallowed_macros,
    clippy::doc_lazy_continuation,
    clippy::empty_line_after_doc_comments,
    clippy::enum_variant_names,
    clippy::iter_kv_map,
    clippy::manual_is_multiple_of,
    clippy::manual_strip,
    clippy::map_identity,
    clippy::missing_const_for_thread_local,
    clippy::needless_borrow,
    clippy::needless_lifetimes,
    clippy::only_used_in_recursion,
    clippy::ptr_arg,
    clippy::redundant_closure,
    clippy::single_char_add_str,
    clippy::too_many_arguments,
    clippy::type_complexity,
    clippy::unnecessary_to_owned,
    clippy::unneeded_struct_pattern,
    clippy::useless_vec,
    unused_imports,
    unused_mut
)]

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::v1_compiler_tokenize::V1LexArtifact;
use crate::v1_std_core::{build_newline_index, NewlineIndex, Token};
use im::Vector as RtVec;

/// FNV-1a over the source bytes. The hash is a KEY COMPONENT, never a decision: a collision would
/// return another file's tokens, so it is paired with the length and the spelling below, and the
/// stored content is compared on a hit before the entry is served.
fn content_fingerprint(content: &str) -> (usize, u64) {
    let mut h: u64 = 1469598103934665603;
    for b in content.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(1099511628211);
    }
    (content.len(), h)
}

struct Acquired {
    content: Rc<String>,
    artifact: Rc<V1LexArtifact>,
    newline_index: Rc<NewlineIndex>,
}

thread_local! {
    static POOL: RefCell<HashMap<(String, usize, u64), Rc<Acquired>>> = RefCell::new(HashMap::new());
}

fn acquire(file: &str, content: &str) -> Rc<Acquired> {
    let (len, hash) = content_fingerprint(content);
    let key = (file.to_string(), len, hash);
    if let Some(hit) = POOL.with(|p| p.borrow().get(&key).cloned()) {
        // The fingerprint narrows; the bytes decide. A hit whose content does not match is a
        // collision, and serving it would be another file's tokens under this file's name.
        if hit.content.as_str() == content {
            return hit;
        }
    }
    let artifact = crate::v1_compiler_tokenize::tokenize_artifact(
        content.to_string(),
        file.to_string(),
        crate::extdeps_languages_dag_syntax::dag_parse_environment(),
    );
    let newline_index = build_newline_index(file.to_string(), content.to_string());
    let acquired = Rc::new(Acquired {
        content: Rc::new(content.to_string()),
        artifact,
        newline_index,
    });
    POOL.with(|p| p.borrow_mut().insert(key, acquired.clone()));
    acquired
}

/// The tokens `tokenize(content, file)` would produce, computed once per (spelling, bytes).
pub fn tokens_for(file: &str, content: &str) -> Rc<RtVec<Rc<Token>>> {
    acquire(file, content).artifact.tokens.clone()
}

/// The lexical artifact (tokens plus the annotation channel), same contract as `tokens_for`.
pub fn artifact_for(file: &str, content: &str) -> Rc<V1LexArtifact> {
    acquire(file, content).artifact.clone()
}

/// The newline index `build_newline_index(file, content)` would produce, computed once.
pub fn newline_index_for(file: &str, content: &str) -> Rc<NewlineIndex> {
    acquire(file, content).newline_index.clone()
}

/// A file's DECLARATION NAMES under the heads reading, once per bytes. Two whole-pool readers
/// want them: `pool_parse` (the fail-closed census the loader forces) and the reference
/// producer's tolerant name index (`entry_resolve::reference_name_index`). They spell files
/// differently, so the token memo above cannot join them, but the names are a function of the
/// bytes alone -- the spelling only labels spans. So the key is the content, and a heads parse
/// either reader already ran answers the other. `None` is a heads-reading refusal, remembered as
/// such: the tolerant reader skips that file, and the fail-closed reader never publishes one.
type DeclNamesSlot = Vec<(Rc<String>, Option<Rc<Vec<String>>>)>;

thread_local! {
    static DECL_NAMES: RefCell<HashMap<(usize, u64), DeclNamesSlot>> = RefCell::new(HashMap::new());
}

fn decl_names_hit(content: &str) -> Option<Option<Rc<Vec<String>>>> {
    let key = content_fingerprint(content);
    DECL_NAMES.with(|m| {
        m.borrow().get(&key).and_then(|slot| {
            slot.iter()
                .find(|(bytes, _)| bytes.as_str() == content)
                .map(|(_, names)| names.clone())
        })
    })
}

fn decl_names_store(content: &str, names: Option<Rc<Vec<String>>>) {
    let key = content_fingerprint(content);
    DECL_NAMES.with(|m| {
        let mut m = m.borrow_mut();
        let slot = m.entry(key).or_default();
        if !slot.iter().any(|(bytes, _)| bytes.as_str() == content) {
            slot.push((Rc::new(content.to_string()), names));
        }
    });
}

/// Record the names a successful heads parse of `content` declared.
pub fn publish_heads_decl_names(content: &str, names: Vec<String>) {
    decl_names_store(content, Some(Rc::new(names)));
}

/// The names the heads reading of `content` declares, running `read` only on a miss.
pub fn heads_decl_names(
    content: &str,
    read: impl FnOnce() -> Option<Vec<String>>,
) -> Option<Rc<Vec<String>>> {
    if let Some(hit) = decl_names_hit(content) {
        return hit;
    }
    let names = read().map(Rc::new);
    decl_names_store(content, names.clone());
    names
}
