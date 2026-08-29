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
    let artifact =
        crate::v1_compiler_tokenize::tokenize_artifact(content.to_string(), file.to_string());
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
