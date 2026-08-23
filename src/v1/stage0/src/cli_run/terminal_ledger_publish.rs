//! Publishing the floor's terminal ledger.
//!
//! THE SEED DOES NOT RENDER THE ARTIFACT. It holds the rows and performs the host write; the
//! bytes come from evaluating `v2.workflow.floor_terminal_ledger_wire`, the way `cli_run` already
//! calls `wire_fnv1a64_content_hash_hex` rather than hashing in Rust beside it. That is what keeps
//! the artifact from becoming a third representation beside the v2 schema and this seed's
//! hand-written mirror of it — the mirror is already declared debt, and answering it by minting
//! another copy of the grammar would be the fix that deepens the defect.
//!
//! WHAT CROSSES THE BOUNDARY, and why it is checked rather than merely small: two type systems
//! have to meet somewhere, so this file names each row's terminal arm in the `.dag` module's
//! vocabulary. It also sends the disposition IT derived. The module derives a disposition
//! independently, from the tag and the expectation, and refuses the whole ledger when the two
//! disagree — so a divergence between this mapping and the module's stops the line instead of
//! publishing a plausible row. Scope, stated honestly: that catches a divergence in the MAPPINGS.
//! The two `ClaimDisposition` declarations are still authored beside each other and nothing here
//! compares them.
//!
//! DISSOLUTION: the same event that retires the rest of the bridge — the self-emitted v2 claim
//! executor. When v2 drives the floor's execution it holds identity and outcome together, renders
//! its own ledger, and this file is deleted whole.

use crate::v1_interpreter::{self, str_value, ExecutionMode, InterpContext, Value};
use std::rc::Rc;

/// The grammar's path RELATIVE TO A SOURCE ROOT, not to the process working directory.
///
/// The first spelling here was `src/v2/workflow/…`, which is what the entry looks like from the
/// workspace root — and it refused for every caller whose CWD was anywhere else, naming the file
/// it could not find. That refusal was correct and located, which is how the mismatch was read in
/// one pass rather than debugged; the repair is to stop encoding the caller's CWD in a constant.
/// Joining each declared root and taking the one that exists ties the lookup to the roots the
/// caller already supplied.
const TERMINAL_LEDGER_WIRE_ENTRY_UNDER_ROOT: &str = "workflow/floor_terminal_ledger_wire.dag";

fn ledger_wire_entry(source_roots: &[String]) -> Result<String, String> {
    source_roots
        .iter()
        .map(|root| std::path::Path::new(root).join(TERMINAL_LEDGER_WIRE_ENTRY_UNDER_ROOT))
        .find(|candidate| candidate.is_file())
        .map(|found| found.to_string_lossy().into_owned())
        .ok_or_else(|| {
            format!(
                "TERMINAL-LEDGER REFUSAL cause=WireAuthorityAbsent \
                 entry={TERMINAL_LEDGER_WIRE_ENTRY_UNDER_ROOT} roots={roots:?} — the ledger's \
                 grammar is not under any declared source root, so no evidence can be rendered.",
                roots = source_roots
            )
        })
}

/// The published artifact, and the name a refused run leaves instead.
///
/// Two names, never one written twice: a refusal must not overwrite a ledger a previous run
/// published, and a reader must not have to open a file to learn which kind it is holding.
pub const TERMINAL_LEDGER_PATH: &str = "target/required_floor_terminal_ledger.tsv";
pub const TERMINAL_LEDGER_DIAGNOSIS_PATH: &str =
    "target/required_floor_terminal_ledger.partial.tsv";

/// The wire vocabulary, mirroring `v2.workflow.floor_terminal_ledger_wire`. Every token here is
/// refused by that module if it does not recognise it, so a divergence surfaces as a typed
/// refusal naming the offending identity rather than as a row that reads differently on each side.
pub struct SeedLedgerRow {
    pub qualified: String,
    pub expectation_wire: &'static str,
    pub terminal_tag: &'static str,
    pub terminal_detail: String,
    pub seed_disposition_wire: &'static str,
}

pub enum LedgerPublication {
    Published {
        path: String,
        bytes: usize,
    },
    RefusedWithDiagnosis {
        reason: String,
        offending: String,
        path: String,
    },
}

fn build_ledger_wire_ctx(source_roots: &[String]) -> Result<InterpContext, String> {
    // THE CONTEXT IS BUILT HERE RATHER THAN HELD ACROSS THE FOLD, and that is a decision about
    // FAILURE MODE, not about magnitude.
    //
    // `run_required_floor` drops its `hermetic` frame before the fold, under a measured note: the
    // frame held a folded 10,114-site manifest and its caches, the runner throttles at
    // `memory.high`, and the last run sat within ~1MB of the watermark. Holding a resolved context
    // across the fold instead would pay this cost in MEMORY on a run already at the line — and in
    // a shared cgroup the kernel kills the largest task, which can be someone else's work, for
    // reasons that say nothing about any witness. Building it here pays in TIME instead:
    // deterministic, attributable, and it degrades by being slow rather than by being killed.
    //
    // The corpus walk and parse are NOT re-paid: `run_required_floor` warms the shared
    // `MultiEntryIndex` during preparation precisely so no single claim pays for it, and this
    // resolve goes through that same shared index. What remains is this entry closure's typecheck.
    // The caller prints the measured wall time on every run rather than trusting this paragraph.
    let entry = ledger_wire_entry(source_roots)?;
    let index = super::process_shared_index(source_roots);
    let (graph, indices) = super::resolve_entry_with_index_for_discovery_corpus(&index, &entry)
        .map_err(|e| {
            format!(
                "TERMINAL-LEDGER REFUSAL cause=WireAuthorityUnresolved entry={entry} detail={e} \
                 — the ledger's grammar could not be resolved, so no evidence can be rendered \
                 and the run is not green."
            )
        })?;
    Ok(super::make_eval_context(
        &graph,
        indices,
        ExecutionMode::Hermetic,
    ))
}

fn row_values(ctx: &InterpContext, rows: &[SeedLedgerRow]) -> Vec<Value> {
    rows.iter()
        .map(|row| Value::Record {
            type_name: ctx.sym("SeedLedgerRow"),
            fields: Rc::new(vec![
                (ctx.sym("qualified"), str_value(row.qualified.clone())),
                (ctx.sym("expectation_wire"), str_value(row.expectation_wire)),
                (ctx.sym("terminal_tag"), str_value(row.terminal_tag)),
                (
                    ctx.sym("terminal_detail"),
                    str_value(row.terminal_detail.clone()),
                ),
                (
                    ctx.sym("seed_disposition_wire"),
                    str_value(row.seed_disposition_wire),
                ),
            ]),
        })
        .collect()
}

/// Write through a temporary and rename, so a reader never observes a half-written ledger.
///
/// `rename` within one filesystem is atomic, so the published name either does not exist or names
/// a complete file. Writing in place would leave a truncated ledger looking like a short run —
/// which is exactly the state the schema's footer exists to make impossible, undone by the write.
fn publish_atomically(path: &str, text: &str) -> Result<usize, String> {
    let tmp = format!("{path}.tmp");
    if let Some(parent) = std::path::Path::new(path).parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            format!(
                "TERMINAL-LEDGER REFUSAL cause=DirectoryUncreatable path={} detail={e}",
                parent.display()
            )
        })?;
    }
    std::fs::write(&tmp, text.as_bytes()).map_err(|e| {
        format!("TERMINAL-LEDGER REFUSAL cause=TemporaryWriteFailed path={tmp} detail={e}")
    })?;
    std::fs::rename(&tmp, path).map_err(|e| {
        format!("TERMINAL-LEDGER REFUSAL cause=PublishRenameFailed from={tmp} to={path} detail={e}")
    })?;
    Ok(text.len())
}

/// Render through the `.dag` authority and publish, or refuse with the diagnosis it produced.
///
/// EVERY ARM EITHER PUBLISHES OR REFUSES. There is no arm that returns Ok having written nothing:
/// evidence that is optional exactly when it fails is the instrumentation-optional shape, and a
/// green run with a silently missing ledger is precisely the state this artifact exists to
/// prevent. A refusal still leaves the rows — under the diagnosis name, in a format the ledger
/// reader refuses — so the run that went wrong is the one that keeps its evidence.
/// The two destinations are PARAMETERS rather than read from the constants inside, so a caller
/// cannot be surprised about where its evidence went and two callers cannot collide on one path.
/// The floor passes the published constants; the tests below pass per-test paths, which is also
/// what stops four tests racing on one file and reading each other's bytes.
pub fn publish_terminal_ledger(
    source_roots: &[String],
    repository_snapshot_wire: &str,
    prepared_subject_digest: &str,
    ledger_path: &str,
    diagnosis_path: &str,
    rows: &[SeedLedgerRow],
) -> Result<LedgerPublication, String> {
    let started = std::time::Instant::now();
    let ctx = build_ledger_wire_ctx(source_roots)?;
    let resolved_ms = started.elapsed().as_millis();
    let args = vec![
        (
            Some("repository_snapshot".to_string()),
            str_value(repository_snapshot_wire),
        ),
        (
            Some("prepared_subject".to_string()),
            str_value(prepared_subject_digest),
        ),
        (
            Some("rows".to_string()),
            Value::List(Rc::new(row_values(&ctx, rows).into())),
        ),
    ];
    let rendered = v1_interpreter::with_active_context(&ctx, || {
        v1_interpreter::run_in_context_with_args(&ctx, "render_ledger_from_seed", &args, false)
    })
    .map_err(|e| {
        format!(
            "TERMINAL-LEDGER REFUSAL cause=RenderFailed detail={e} — the ledger's grammar did not \
             produce a render, so there is no evidence for this run."
        )
    })?;
    let outcome = read_render(&ctx, &rendered)?;
    let published = match outcome {
        Render::Text(text) => {
            let bytes = publish_atomically(ledger_path, &text)?;
            LedgerPublication::Published {
                path: ledger_path.to_string(),
                bytes,
            }
        }
        Render::Refused {
            reason,
            offending,
            diagnosis,
        } => {
            publish_atomically(diagnosis_path, &diagnosis)?;
            LedgerPublication::RefusedWithDiagnosis {
                reason,
                offending,
                path: diagnosis_path.to_string(),
            }
        }
    };
    // THE COST IS PRINTED, NOT ASSERTED. An asserted cost is a claim in a PR body that decays; a
    // printed one is an instrument, so whoever wants it removed can see what removing it buys.
    eprintln!(
        "[floor-phase] phase=terminal-ledger-publish state=completed resolve_ms={} total_ms={} rows={}",
        resolved_ms,
        started.elapsed().as_millis(),
        rows.len()
    );
    Ok(published)
}

/// Read one published terminal ledger through the `.dag` grammar and derive exactly its
/// budget-refused identities. The grammar verifies binding, footer population and row digest;
/// this seed seam only transports the resulting closed arm.
pub fn budget_refused_population_from_ledger(
    source_roots: &[String],
    text: &str,
    expected_repository_snapshot: &str,
    expected_prepared_subject: &str,
) -> Result<Vec<String>, String> {
    let ctx = build_ledger_wire_ctx(source_roots)?;
    let args = vec![
        (Some("text".to_string()), str_value(text)),
        (
            Some("expected_repository_snapshot".to_string()),
            str_value(expected_repository_snapshot),
        ),
        (
            Some("expected_prepared_subject".to_string()),
            str_value(expected_prepared_subject),
        ),
    ];
    let value = v1_interpreter::with_active_context(&ctx, || {
        v1_interpreter::run_in_context_with_args(
            &ctx,
            "budget_refused_population_from_ledger",
            &args,
            false,
        )
    })
    .map_err(|e| format!("COST-MEASUREMENT REFUSAL cause=LedgerReadFailed detail={e}"))?;
    let Value::Variant {
        variant_name,
        fields,
        ..
    } = value
    else {
        return Err("COST-MEASUREMENT REFUSAL cause=LedgerReadShape".to_string());
    };
    if ctx.sym_eq(variant_name, "BudgetRefusedPopulationRefused") {
        return Err(format!(
            "COST-MEASUREMENT REFUSAL cause={}",
            variant_field(&fields, &ctx, "cause").unwrap_or_else(|| "unreadable-cause".into())
        ));
    }
    if !ctx.sym_eq(variant_name, "BudgetRefusedPopulationDerived") {
        return Err("COST-MEASUREMENT REFUSAL cause=LedgerReadUnknownArm".to_string());
    }
    let identities = fields
        .iter()
        .find_map(|(symbol, value)| ctx.sym_eq(*symbol, "identities").then_some(value))
        .ok_or_else(|| "COST-MEASUREMENT REFUSAL cause=LedgerPopulationAbsent".to_string())?;
    let Value::List(items) = identities else {
        return Err("COST-MEASUREMENT REFUSAL cause=LedgerPopulationNotList".to_string());
    };
    items
        .iter()
        .map(|value| match value {
            Value::Str(identity) => Ok(identity.to_string()),
            _ => Err("COST-MEASUREMENT REFUSAL cause=LedgerIdentityNotString".to_string()),
        })
        .collect()
}

enum Render {
    Text(String),
    Refused {
        reason: String,
        offending: String,
        diagnosis: String,
    },
}

fn variant_field(
    fields: &[(crate::v1_interpreter::Symbol, Value)],
    ctx: &InterpContext,
    name: &str,
) -> Option<String> {
    fields.iter().find_map(|(sym, value)| {
        if ctx.sym_eq(*sym, name) {
            match value {
                Value::Str(s) => Some(s.to_string()),
                _ => None,
            }
        } else {
            None
        }
    })
}

/// Every shape this cannot read is a REFUSAL, never a default. A render whose shape the seed does
/// not recognise means the two sides disagree about the grammar's own result type, and publishing
/// anything at that point would be publishing a guess.
fn read_render(ctx: &InterpContext, value: &Value) -> Result<Render, String> {
    let Value::Variant {
        variant_name,
        fields,
        ..
    } = value
    else {
        return Err(format!(
            "TERMINAL-LEDGER REFUSAL cause=RenderShapeUnexpected got={} — expected a LedgerRender.",
            value.type_label_public()
        ));
    };
    if ctx.sym_eq(*variant_name, "LedgerRendered") {
        let text = variant_field(fields, ctx, "text").ok_or_else(|| {
            "TERMINAL-LEDGER REFUSAL cause=RenderMissingText — LedgerRendered carried no `text`."
                .to_string()
        })?;
        return Ok(Render::Text(text));
    }
    if ctx.sym_eq(*variant_name, "LedgerUnrenderable") {
        let reason = variant_field(fields, ctx, "reason").ok_or_else(|| {
            "TERMINAL-LEDGER REFUSAL cause=RenderMissingReason — LedgerUnrenderable carried no \
             `reason`. The seed and the grammar disagree about the refusal's own shape, which is \
             the disagreement this reader exists to catch; defaulting it would publish a blank \
             where the cause belongs."
                .to_string()
        })?;
        let offending = variant_field(fields, ctx, "offending").ok_or_else(|| {
            "TERMINAL-LEDGER REFUSAL cause=RenderMissingOffending — LedgerUnrenderable carried no \
             `offending`."
                .to_string()
        })?;
        let diagnosis = variant_field(fields, ctx, "diagnosis").ok_or_else(|| {
            "TERMINAL-LEDGER REFUSAL cause=RefusalMissingDiagnosis — LedgerUnrenderable carried no \
             `diagnosis`, so the refusal would have destroyed the rows it refused."
                .to_string()
        })?;
        return Ok(Render::Refused {
            reason,
            offending,
            diagnosis,
        });
    }
    Err(format!(
        "TERMINAL-LEDGER REFUSAL cause=RenderVariantUnknown — the grammar answered a LedgerRender \
         arm this seed does not recognise, so the two sides disagree about the result type."
    ))
}

#[cfg(test)]
mod terminal_ledger_publish_law {
    //! GREEN BY EXECUTION, WITH A DISCRIMINATING RED — the whole seam, not a mock of it.
    //!
    //! These call the real `publish_terminal_ledger`: it resolves the real `.dag` grammar, renders
    //! through the real interpreter, and writes real bytes. A test that stubbed the render would
    //! prove only that this file can format a struct, which is not the claim. The claim is that
    //! the seed and the module agree, and only running both can establish it.

    use super::*;

    fn row(qualified: &str, tag: &'static str, disposition: &'static str) -> SeedLedgerRow {
        SeedLedgerRow {
            qualified: qualified.to_string(),
            expectation_wire: "expect-hold",
            terminal_tag: tag,
            terminal_detail: String::new(),
            seed_disposition_wire: disposition,
        }
    }

    /// ABSOLUTE ROOTS, because `cargo test` runs from the crate directory while the floor runs
    /// from the workspace root. Relative roots made every one of these fail with
    /// `WireAuthorityUnresolved` — a real refusal, correctly typed and located, and it named the
    /// entry it could not find, which is how the environment mismatch was visible in one read
    /// rather than debugged. Production keeps its CWD-relative roots like every other call site;
    /// it is the test's environment that differs, so it is the test that adapts.
    fn roots() -> Vec<String> {
        let root = super::super::workspace_root();
        vec![
            root.join("dag").to_string_lossy().into_owned(),
            root.join("src/v2").to_string_lossy().into_owned(),
        ]
    }

    /// One destination per test: these run in parallel and a shared path would have them reading
    /// each other's bytes, which is a green that means nothing.
    fn test_path(name: &str) -> String {
        std::env::temp_dir()
            .join(format!("gunbc-terminal-ledger-test-{name}.tsv"))
            .to_string_lossy()
            .into_owned()
    }

    /// The published paths are workspace-relative in production; under `cargo test` the CWD is the
    /// crate directory, so they are read back through the same root the roots use.
    fn published_path(path: &str) -> std::path::PathBuf {
        let p = std::path::Path::new(path);
        if p.is_file() {
            p.to_path_buf()
        } else {
            super::super::workspace_root().join(path)
        }
    }

    #[test]
    fn an_honest_population_publishes_a_ledger_the_grammar_can_read_back() {
        let rows = vec![
            row("test.claim.a.holds", "returned-true", "passed"),
            row("test.claim.b.holds", "returned-true", "passed"),
        ];
        let published = publish_terminal_ledger(
            &roots(),
            "1234567890abcdef1234567890abcdef12345678",
            "0123456789abcdef",
            &test_path("honest"),
            &test_path("honest-diagnosis"),
            &rows,
        )
        .expect("publish");
        match published {
            LedgerPublication::Published { path, bytes } => {
                let text =
                    std::fs::read_to_string(published_path(&path)).expect("read published ledger");
                assert!(bytes > 0);
                // header + two rows + footer, and the footer states the row count.
                assert_eq!(text.lines().count(), 4, "ledger shape: {text}");
                assert!(text.starts_with("gunbc-terminal-ledger/1\t"), "{text}");
                assert!(
                    text.contains("1234567890abcdef1234567890abcdef12345678"),
                    "the published commit must appear in the binding: {text}"
                );
                assert!(
                    text.lines().last().unwrap().starts_with("#end\t2\t"),
                    "{text}"
                );
            }
            LedgerPublication::RefusedWithDiagnosis {
                reason, offending, ..
            } => {
                panic!("expected a published ledger, got refusal {reason} at {offending}")
            }
        }
    }

    #[test]
    fn an_unpublished_run_is_shaped_as_unbound_rather_than_carrying_a_fake_commit() {
        let rows = vec![row("test.claim.a.holds", "returned-true", "passed")];
        let published = publish_terminal_ledger(
            &roots(),
            "unpublished",
            "0123456789abcdef",
            &test_path("unpublished"),
            &test_path("unpublished-diagnosis"),
            &rows,
        )
        .expect("publish");
        let LedgerPublication::Published { path, .. } = published else {
            panic!("expected a published ledger")
        };
        let text = std::fs::read_to_string(published_path(&path)).expect("read");
        let header = text.lines().next().unwrap();
        assert!(
            header.contains("\tunpublished\t"),
            "an unpublished run must say so in the binding, not carry a commit-shaped value: {header}"
        );
    }

    /// THE DISCRIMINATING RED FOR THE COMPARATOR. This seed's disposition and the module's are two
    /// independent derivations; a row where they disagree must refuse, and the diagnosis must
    /// survive naming the offending identity.
    #[test]
    fn a_disposition_that_disagrees_with_its_tag_refuses_and_keeps_the_rows() {
        let rows = vec![
            row("test.claim.a.holds", "returned-true", "passed"),
            // A pass-shaped terminal labelled `failed`: the module derives `passed` from the tag.
            row("test.claim.b.holds", "returned-true", "failed"),
        ];
        let published = publish_terminal_ledger(
            &roots(),
            "1234567890abcdef1234567890abcdef12345678",
            "0123456789abcdef",
            &test_path("disagree"),
            &test_path("disagree-diagnosis"),
            &rows,
        )
        .expect("publish call itself must succeed — the refusal is a value, not an error");
        match published {
            LedgerPublication::Published { .. } => {
                panic!("a disagreeing disposition must NOT publish a ledger")
            }
            LedgerPublication::RefusedWithDiagnosis {
                reason,
                offending,
                path,
            } => {
                assert_eq!(reason, "seed-disposition-disagrees");
                assert_eq!(offending, "test.claim.b.holds");
                let diagnosis =
                    std::fs::read_to_string(published_path(&path)).expect("read diagnosis");
                // every row survives, and the file cannot be read as a ledger
                assert!(diagnosis.contains("test.claim.a.holds"), "{diagnosis}");
                assert!(diagnosis.contains("test.claim.b.holds"), "{diagnosis}");
                assert!(
                    diagnosis.starts_with("gunbc-terminal-ledger-partial/1\t"),
                    "a diagnosis must not open with the ledger format token: {diagnosis}"
                );
                assert!(
                    !diagnosis.contains("\n#end\t"),
                    "a diagnosis must carry no footer: {diagnosis}"
                );
            }
        }
    }

    /// Every `ClaimOutcome` arm maps to a tag the module recognises. A tag this seed spells
    /// wrongly refuses the whole ledger, so this is the cheapest place to find out.
    #[test]
    fn every_terminal_tag_this_seed_can_emit_is_one_the_grammar_admits() {
        let rows = vec![
            row("test.claim.pass", "returned-true", "passed"),
            row("test.claim.fail", "returned-false", "failed"),
            row(
                "test.claim.unreadable",
                "returned-unreadable",
                "observation-unreadable-before-verdict",
            ),
            row(
                "test.claim.runtime",
                "runtime-errored",
                "runtime-errored-before-verdict",
            ),
            row(
                "test.claim.budget",
                "budget-refused",
                "budget-refused-before-verdict",
            ),
            row(
                "test.claim.tool",
                "host-tool-unresolved",
                "host-tool-unresolved-before-verdict",
            ),
            row("test.claim.route", "route-gap", "route-gap-before-verdict"),
        ];
        let published = publish_terminal_ledger(
            &roots(),
            "1234567890abcdef1234567890abcdef12345678",
            "0123456789abcdef",
            &test_path("tags"),
            &test_path("tags-diagnosis"),
            &rows,
        )
        .expect("publish");
        match published {
            LedgerPublication::Published { path, .. } => {
                let text = std::fs::read_to_string(published_path(&path)).expect("read");
                assert_eq!(text.lines().count(), 9, "{text}");
            }
            LedgerPublication::RefusedWithDiagnosis {
                reason, offending, ..
            } => {
                panic!("tag vocabulary disagrees with the grammar: {reason} at {offending}")
            }
        }
    }
}
