//! ONE ACQUISITION OF A SOURCE FILE, NOT ONE PER WALK.
//!
//! Four whole-tree walks stand between `gunbc compile` and a parsed closure --
//! `entry_resolve::build_module_path_index_uncached`, `entry_resolve::reference_resolution_facts`,
//! `extend_sources_to_both_closure_fixpoint`, and the census-only fill. Each one independently
//! read, tokenized and newline-indexed the same file, so on the seed closure the 3,031-file tree
//! was tokenized 12,121 times: measured `distinct_file_spellings=3031`, of which 3,030 were
//! tokenized exactly 4x.
//!
//! The closure front end (`entry_resolve::via_index_parse_one_source` and the parse-cache miss
//! arm) is a fifth reader of the same bytes under the same spelling, so it asks here too rather
//! than re-lexing every closure file the pool census already lexed.
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
    /// The heads reading of these bytes, parsed in a FILE-LOCAL space (empty intern table,
    /// occurrence ordinals from zero), filled on first demand. See `heads_reading_for`.
    heads: RefCell<Option<Rc<HeadsReading>>>,
}

thread_local! {
    static POOL: RefCell<HashMap<(String, usize, u64), Rc<Acquired>>> = RefCell::new(HashMap::new());
}

thread_local! {
    /// Everything `attribute` has recorded on this thread, so a caller timing a span that forces
    /// acquisitions can report itself NET of them instead of counting the same work twice.
    static ATTRIBUTED: std::cell::Cell<std::time::Duration> =
        const { std::cell::Cell::new(std::time::Duration::ZERO) };
}

/// The lexing, the newline index and the heads parse are demanded by whichever walk reaches a file
/// first, so they are recorded HERE, at the producer, as their own `[pre-entry]` rows. Timed at the first demander
/// instead, they were reported under that demander's name: the module path index is built inside
/// the import-edge facts, so `graph_facts_import_edges` carried the whole pool's acquisition and
/// heads parse while its own work is a line scan.
fn attribute(row: &'static str, elapsed: std::time::Duration) {
    super::pre_entry_phase::record(row, super::pre_entry_phase::PhaseScale::Tree, elapsed);
    ATTRIBUTED.with(|a| a.set(a.get() + elapsed));
}

/// Total time `attribute` has recorded on this thread; difference two readings around a span.
pub(crate) fn attributed() -> std::time::Duration {
    ATTRIBUTED.with(|a| a.get())
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
    let lex_started = std::time::Instant::now();
    let artifact = crate::v1_compiler_tokenize::tokenize_artifact(
        content.to_string(),
        file.to_string(),
        crate::extdeps_languages_dag_syntax::dag_parse_environment(),
    );
    attribute("pool_source_tokenize", lex_started.elapsed());
    let newline_started = std::time::Instant::now();
    let newline_index = build_newline_index(file.to_string(), content.to_string());
    attribute("pool_source_newline_index", newline_started.elapsed());
    let acquired = Rc::new(Acquired {
        content: Rc::new(content.to_string()),
        artifact,
        newline_index,
        heads: RefCell::new(None),
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

/// WHAT THE HEADS READING RETAINS: exactly the fields its consumers read, and nothing else. Its
/// consumers are `module_path_index::parse_module_binding` (the module name and span, and the
/// refusal) and the pool census's `census_heads::project_heads_reading` (the module node, the
/// refusal, and the file-local intern strings and occurrence-allocator end it relabels into the
/// pool's space). The parse's occurrence transport, its full intern index and its allocator are
/// not read by either, so they are not retained: holding the whole parse result for every pool
/// file for the life of the process was the +16% peak-RSS regression the #12656 bisect measured.
pub struct HeadsReading {
    /// RETAINED FOR THE PROCESS, and that is the obligated lifetime rather than a leak. Every
    /// `MultiEntryIndex` whose pool contains this file projects this node (`pool_parse`), and a
    /// run builds its indexes on demand, one per module-name set its phases ask for (measured on
    /// #12765's floor: the shared index, a v1 attribution index and the `namespace_baseline`
    /// [dag] closure, besides the duplicate that PR removed). When the first census projects the
    /// node, nothing at this layer can know whether a later phase will build another index whose
    /// pool contains the file: that set is decided by the run's phase routing, and consulting it
    /// from the acquisition layer would invert the layers. Releasing after the first projection
    /// would force a re-parse for the next index, which is ruled out. The frontier and its
    /// dissolution are rostered in `gunbc.resolver_cost_frontier`.
    pub module: Option<Rc<crate::v1_std_core::Node>>,
    pub error: Option<Rc<crate::v1_std_core::ErrorNode>>,
    /// The file-local intern strings, in local-id order (local id `k` is `local_strings[k]`).
    pub local_strings: Rc<RtVec<String>>,
    /// The file-local occurrence allocator's next id after the parse.
    pub local_next: i64,
}

#[cfg(test)]
thread_local! {
    /// Heads parses per (spelling, bytes) acquisition key: the control that the heads reading is
    /// parsed at most once per key.
    pub(crate) static HEADS_PARSES: RefCell<HashMap<(String, usize, u64), usize>> =
        RefCell::new(HashMap::new());
}

/// THE ONE HEADS READING of a file, computed once per (spelling, bytes).
///
/// It is parsed in a FILE-LOCAL space: an empty intern table and occurrence ordinals from zero.
/// That makes it a pure function of its key, which is what lets two consumers share it:
/// `module_path_index::parse_module_binding` reads it as-is, since it projects only the module
/// name, its span and the refusal, none of which carry an id. The pool census reads it through
/// `census_heads::project_heads_reading`, which maps it into the pool's threaded intern table
/// and occurrence space. That projection is total, so the census needs no second parse.
pub fn heads_reading_for(file: &str, content: &str) -> Rc<HeadsReading> {
    let acquired = acquire(file, content);
    if let Some(hit) = acquired.heads.borrow().clone() {
        return hit;
    }
    #[cfg(test)]
    {
        let (len, hash) = content_fingerprint(content);
        HEADS_PARSES.with(|p| {
            *p.borrow_mut()
                .entry((file.to_string(), len, hash))
                .or_default() += 1
        });
    }
    let mut indices = im::HashMap::new();
    indices.insert(file.to_string(), acquired.newline_index.clone());
    let heads_started = std::time::Instant::now();
    let parsed = crate::v1_compiler_parse::parse_heads_with_table(
        acquired.artifact.tokens.clone(),
        Rc::new(indices),
        crate::v1_std_core::empty_intern_table(),
    );
    attribute("pool_heads_parse", heads_started.elapsed());
    let reading = Rc::new(HeadsReading {
        module: parsed.result.module.clone(),
        error: parsed.result.error.clone(),
        local_strings: parsed.intern_table.strings.clone(),
        local_next: parsed
            .intern_table
            .authored_token_ordinals
            .allocator
            .next_id,
    });
    *acquired.heads.borrow_mut() = Some(reading.clone());
    reading
}
