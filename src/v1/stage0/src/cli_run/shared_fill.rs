//! WHO PAID, AND WHO BENEFITED — the floor's shared-computation ledger.
//!
//! The required floor charges wall time PER CLAIM. Much of that time is not the claim's: the
//! seed carries a population of process-global corpus-scan caches (a whole-pool parse, a module
//! index, several corpus census reports) that are filled ONCE and read free thereafter. The
//! first claim to reach one pays the entire fill; every later claim reaching the same
//! computation pays nothing. Measured on gunbc#8455: two rows of one witness file calling the
//! same projection, 35ms apart, cost 12197ms and 31ms — 393x for the same call.
//!
//! That makes the per-row number the floor already prints unusable for the decision it is
//! actually used for. "This witness costs 12 seconds" and "this witness ran first" are
//! different facts with different remedies, and the ceiling refuses on the first while
//! measuring the second. Deciding to pare a witness on that number removes a row whose own
//! cost may be milliseconds, and re-assigns the fill to whichever row now touches it first.
//!
//! This ledger records the missing half. Every instrumented cache reports, per fill: what it
//! cost, which claim paid, and every claim and module that later read it. From that the
//! marginal quantity is derivable and the attributed one is not:
//!
//! ```text
//!   what removing module M saves  =  M's rows' exclusive time
//!                                 +  fills whose consumer set is entirely inside M
//! ```
//!
//! A fill consumed from outside M survives M's removal — the cost migrates to the next
//! toucher rather than disappearing. A fill with exactly one consumer is genuinely that
//! claim's.
//!
//! REPORTING ONLY. Nothing here refuses, skips, greens, or changes any verdict; it prints a
//! table at the end of the fold. Per DESIGN §5 that is the sanctioned second mode — a
//! stopped-line audit that reports and does not green.

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;
use std::time::Instant;

/// One filled shared computation.
struct Fill {
    /// Declared input the cache is keyed on, so two fills of one cache under different roots
    /// are two rows rather than one averaged one.
    key: String,
    /// Wall from entering the fill to leaving it, INCLUDING any fill it triggered underneath.
    inclusive_nanos: u64,
    /// Inclusive minus the fills nested inside it. These caches compose — building the module
    /// graph builds the reference edges and the path index on its way — so summing inclusive
    /// figures counts the inner scans once per enclosing scan. Measured on the first floor
    /// receipt: a 17890ms module-graph fill contained a 12054ms path-index fill and a 5141ms
    /// reference-edge fill, leaving ~695ms of its own. The total is therefore summed over SELF,
    /// and the inclusive figure is carried beside it because it is what names the caller that
    /// caused the inner scans.
    self_nanos: u64,
    /// `None` when the fill happened outside the claim fold — during preparation, discovery or
    /// a gate. That is a real and separate state from "some claim paid it": preparation cost is
    /// not any witness's, and collapsing the two would let the fold's own overhead read as a
    /// witness's expense.
    filler: Option<String>,
    /// Every claim that READ this fill after it existed, filler excluded.
    consumers: BTreeSet<String>,
}

#[derive(Default)]
struct Ledger {
    /// cache name -> its fills, in fill order.
    caches: BTreeMap<&'static str, Vec<Fill>>,
    /// Hits that found no recorded fill. Never silently dropped: a nonzero count means a cache
    /// was filled by a path this module does not observe, so its rows understate the sharing.
    unattributed_hits: u64,
}

thread_local! {
    static LEDGER: RefCell<Ledger> = RefCell::new(Ledger::default());
    /// The claim the fold is currently evaluating, or `None` outside the fold.
    static CURRENT_CLAIM: RefCell<Option<String>> = const { RefCell::new(None) };
    /// One accumulator per fill currently in flight, innermost last. Each holds the inclusive
    /// time of the fills that completed inside it.
    static FILL_CHILD_NANOS: RefCell<Vec<u64>> = const { RefCell::new(Vec::new()) };
}

/// Enter a fill. Every `begin_fill` is answered by exactly one `record_fill` or `abandon_fill`,
/// or the nesting accounting drifts silently — which is why the abandon arm exists rather than
/// letting the un-built branch of `once` simply return.
pub(crate) fn begin_fill() {
    FILL_CHILD_NANOS.with(|s| s.borrow_mut().push(0));
}

/// Leave a fill that did not happen after all (another thread initialized the cell first).
pub(crate) fn abandon_fill() {
    FILL_CHILD_NANOS.with(|s| {
        s.borrow_mut().pop();
    });
}

/// Close the innermost fill: take its children's total, and hand its own inclusive time up to
/// its parent so the parent can subtract it in turn.
fn close_fill(inclusive: u64) -> u64 {
    FILL_CHILD_NANOS.with(|s| {
        let mut stack = s.borrow_mut();
        let children = stack.pop().unwrap_or(0);
        if let Some(parent) = stack.last_mut() {
            *parent = parent.saturating_add(inclusive);
        }
        inclusive.saturating_sub(children)
    })
}

/// Name the claim now executing. Called once per claim by the required-floor fold; cleared when
/// the fold ends so that later gate work cannot be charged to the last witness that ran.
pub(crate) fn set_current_claim(qualified: Option<&str>) {
    CURRENT_CLAIM.with(|c| *c.borrow_mut() = qualified.map(str::to_string));
}

fn current_claim() -> Option<String> {
    CURRENT_CLAIM.with(|c| c.borrow().clone())
}

/// Record a fill that has just been computed.
pub(crate) fn record_fill(cache: &'static str, key: &str, nanos: u64) {
    let filler = current_claim();
    let self_nanos = close_fill(nanos);
    LEDGER.with(|l| {
        l.borrow_mut().caches.entry(cache).or_default().push(Fill {
            key: key.to_string(),
            inclusive_nanos: nanos,
            self_nanos,
            filler,
            consumers: BTreeSet::new(),
        })
    });
}

/// Record a read that was served by an existing fill.
pub(crate) fn record_hit(cache: &'static str, key: &str) {
    let Some(claim) = current_claim() else {
        // A hit outside the fold consumes the fill but is not a witness's benefit, so it is not
        // a consumer. It is still not nothing: counting it keeps the hit total honest.
        LEDGER.with(|l| l.borrow_mut().unattributed_hits += 1);
        return;
    };
    LEDGER.with(|l| {
        let mut ledger = l.borrow_mut();
        match ledger
            .caches
            .get_mut(cache)
            .and_then(|fills| fills.iter_mut().find(|f| f.key == key))
        {
            Some(fill) => {
                if fill.filler.as_deref() != Some(claim.as_str()) {
                    fill.consumers.insert(claim);
                }
            }
            None => ledger.unattributed_hits += 1,
        }
    });
}

/// Wrap a `OnceLock`-backed shared computation: time the fill, attribute it, and count the
/// reads. The `Cell` is how the fill is distinguished from a read — `get_or_init` runs the
/// closure only on the initializing call, so timing around it unconditionally would charge
/// every reader the fill's duration.
pub(crate) fn once<T>(
    cell: &'static OnceLock<T>,
    cache: &'static str,
    key: &str,
    build: impl FnOnce() -> T,
) -> &'static T {
    if let Some(value) = cell.get() {
        record_hit(cache, key);
        return value;
    }
    let built = Cell::new(false);
    begin_fill();
    let start = Instant::now();
    let value = cell.get_or_init(|| {
        built.set(true);
        build()
    });
    if built.get() {
        record_fill(cache, key, start.elapsed().as_nanos() as u64);
    } else {
        abandon_fill();
        record_hit(cache, key);
    }
    value
}

/// The module a qualified claim name belongs to (`module.function` -> `module`).
fn module_of(qualified: &str) -> &str {
    match qualified.rfind('.') {
        Some(i) => &qualified[..i],
        None => qualified,
    }
}

/// Mirror of `gunbc.observation_ci_render` `ci_shared_fill_row_text`. The authority for how the
/// line READS — and for the disposition it carries — is the `.dag`; this transports the numbers.
pub(crate) fn render_shared_fill_row_text_mirror(
    cache: &str,
    key: &str,
    fill_ms: u128,
    inclusive_ms: u128,
    paid_by: Option<&str>,
    consumer_claims: usize,
    consumer_modules: usize,
    modules: &str,
) -> String {
    format!(
        "[floor-shared-fill] cache={cache} key={key} fill_ms={fill_ms} \
         inclusive_ms={inclusive_ms} paid_by={} consumer_claims={consumer_claims} \
         consumer_modules={consumer_modules} disposition={} modules={modules}",
        paid_by.unwrap_or("<outside-fold>"),
        shared_fill_disposition_tag(paid_by.is_some(), consumer_modules),
    )
}

/// Mirror of `gunbc.observation_ci_render` `ci_shared_fill_total_text`.
pub(crate) fn render_shared_fill_total_text_mirror(
    fills: usize,
    fill_ms: u128,
    shared_fill_ms: u128,
    unattributed_hits: u64,
) -> String {
    format!(
        "[floor-shared-fill] TOTAL fills={fills} fill_ms={fill_ms} \
         shared_fill_ms={shared_fill_ms} unattributed_hits={unattributed_hits}"
    )
}

/// Mirror of `gunbc.witness_row_cost` `shared_fill_disposition` composed with its tag. Three
/// states because three remedies: preparation cost no witness can be pared to recover, a fill
/// one module owns and loses with itself, and a fill that outlives the removal of any single
/// consumer.
fn shared_fill_disposition_tag(paid_inside_fold: bool, consumer_modules: usize) -> &'static str {
    if !paid_inside_fold {
        "outside-fold"
    } else if consumer_modules > 1 {
        "shared"
    } else {
        "exclusive"
    }
}

/// Render the ledger as `[floor-shared-fill]` lines.
///
/// One line per fill, then one total. The per-fill line is what a paring decision reads:
/// `exclusive` fills are those a single module consumed, so removing that module genuinely
/// removes them; `shared` fills survive whoever is removed, because the next claim to touch the
/// computation pays the same seconds.
pub(crate) fn report() -> String {
    LEDGER.with(|l| {
        let ledger = l.borrow();
        let mut out = String::new();
        let mut total_nanos: u64 = 0;
        let mut shared_nanos: u64 = 0;
        let mut fills_total: usize = 0;
        for (cache, fills) in &ledger.caches {
            for fill in fills {
                fills_total += 1;
                total_nanos += fill.self_nanos;
                let modules: BTreeSet<&str> = fill
                    .filler
                    .iter()
                    .map(|f| module_of(f))
                    .chain(fill.consumers.iter().map(|c| module_of(c)))
                    .collect();
                if fill.filler.is_some() && modules.len() > 1 {
                    shared_nanos += fill.self_nanos;
                }
                out.push_str(&render_shared_fill_row_text_mirror(
                    cache,
                    &fill.key,
                    u128::from(fill.self_nanos / 1_000_000),
                    u128::from(fill.inclusive_nanos / 1_000_000),
                    fill.filler.as_deref(),
                    fill.consumers.len(),
                    modules.len(),
                    &modules.iter().copied().collect::<Vec<_>>().join(","),
                ));
                out.push('\n');
            }
        }
        out.push_str(&render_shared_fill_total_text_mirror(
            fills_total,
            u128::from(total_nanos / 1_000_000),
            u128::from(shared_nanos / 1_000_000),
            ledger.unattributed_hits,
        ));
        out.push('\n');
        out
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reset() {
        LEDGER.with(|l| *l.borrow_mut() = Ledger::default());
        FILL_CHILD_NANOS.with(|s| s.borrow_mut().clear());
        set_current_claim(None);
    }

    /// THE SEED HALF OF THE ORACLE PAIR. The literals are the ones
    /// `test.claim.observation_ci_render_witness_test` `w_shared_fill_row_line_holds` and
    /// `w_shared_fill_total_line_holds` assert against the `.dag` authority, so a format change
    /// on either side reds one of the two instead of forking the line silently.
    #[test]
    fn the_rendered_lines_match_the_dag_authority_literals() {
        assert_eq!(
            render_shared_fill_row_text_mirror(
                "reference_edges",
                "dag+src/v2",
                695,
                17890,
                Some("test.claim.foo.w_bar"),
                3,
                2,
                "test.claim.foo,test.claim.other",
            ),
            "[floor-shared-fill] cache=reference_edges key=dag+src/v2 fill_ms=695 \
             inclusive_ms=17890 paid_by=test.claim.foo.w_bar consumer_claims=3 \
             consumer_modules=2 disposition=shared modules=test.claim.foo,test.claim.other"
        );
        assert_eq!(
            render_shared_fill_total_text_mirror(9, 60000, 42000, 0),
            "[floor-shared-fill] TOTAL fills=9 fill_ms=60000 shared_fill_ms=42000 \
             unattributed_hits=0"
        );
    }

    #[test]
    fn a_fill_read_by_a_second_module_is_shared_not_exclusive() {
        reset();
        set_current_claim(Some("test.claim.a.w_one"));
        begin_fill();
        record_fill("c", "k", 5_000_000_000);
        set_current_claim(Some("test.claim.b.w_two"));
        record_hit("c", "k");
        set_current_claim(None);
        let text = report();
        assert!(
            text.contains("disposition=shared") && text.contains("consumer_modules=2"),
            "a fill two modules read cannot be removed by deleting one of them: {text}"
        );
        assert!(
            text.contains("shared_fill_ms=5000"),
            "the shared total must carry the fill: {text}"
        );
    }

    /// WHICH MODULES, NOT HOW MANY. A count cannot answer "did the row that inherited this bill
    /// read this exact fill before" — that is an identity join, and DESIGN §5 rules a count is
    /// not one. The named list is what lets a relocation be checked at identity grain instead of
    /// by matching magnitudes.
    #[test]
    fn the_consumer_modules_are_named_not_merely_counted() {
        reset();
        set_current_claim(Some("test.claim.payer.w_one"));
        begin_fill();
        record_fill("c", "k", 1_000_000);
        set_current_claim(Some("test.claim.rider.w_two"));
        record_hit("c", "k");
        set_current_claim(None);
        let text = report();
        assert!(
            text.contains("modules=test.claim.payer,test.claim.rider"),
            "a reader must be able to ask whether a specific module rode this fill: {text}"
        );
    }

    #[test]
    fn a_fill_only_its_own_module_reads_is_exclusive() {
        reset();
        set_current_claim(Some("test.claim.a.w_one"));
        begin_fill();
        record_fill("c", "k", 1_000_000);
        set_current_claim(Some("test.claim.a.w_two"));
        record_hit("c", "k");
        set_current_claim(None);
        let text = report();
        assert!(
            text.contains("disposition=exclusive") && text.contains("shared_fill_ms=0"),
            "one module's own fill is removable with it: {text}"
        );
    }

    /// THE DEFECT THE FIRST FLOOR RECEIPT EXPOSED. These caches compose, so an outer fill's
    /// wall contains its inner fills' wall. Summing inclusive figures counted the module graph's
    /// nested path-index and reference-edge scans twice. The total must be over SELF, and the
    /// inner fill must keep its own figure — it is the one that names the real scan.
    #[test]
    fn a_nested_fill_is_not_counted_twice_in_the_total() {
        reset();
        set_current_claim(Some("test.claim.a.w_one"));
        begin_fill(); // outer: module-graph-shaped
        begin_fill(); // inner: path-index-shaped, entirely inside the outer
        record_fill("inner", "k", 12_000_000_000);
        record_fill("outer", "k", 17_000_000_000);
        set_current_claim(None);
        let text = report();
        assert!(
            text.contains("cache=inner key=k fill_ms=12000 inclusive_ms=12000"),
            "the inner scan keeps its own figure: {text}"
        );
        assert!(
            text.contains("cache=outer key=k fill_ms=5000 inclusive_ms=17000"),
            "the outer fill's own work is its wall minus what it triggered: {text}"
        );
        assert!(
            text.contains("fill_ms=17000 shared_fill_ms=0"),
            "the total is 17s of real scanning, not the 29s an inclusive sum would report: \
             {text}"
        );
    }

    #[test]
    fn a_fill_outside_the_fold_is_not_charged_to_any_witness() {
        reset();
        set_current_claim(None);
        begin_fill();
        record_fill("c", "k", 9_000_000);
        let text = report();
        assert!(
            text.contains("disposition=outside-fold") && text.contains("paid_by=<outside-fold>"),
            "preparation cost is not a witness's: {text}"
        );
    }

    #[test]
    fn a_hit_with_no_recorded_fill_is_counted_never_dropped() {
        reset();
        set_current_claim(Some("test.claim.a.w_one"));
        record_hit("never_filled_here", "k");
        let text = report();
        assert!(
            text.contains("unattributed_hits=1"),
            "a cache filled by an unobserved path must say so rather than read as unshared: \
             {text}"
        );
    }
}
