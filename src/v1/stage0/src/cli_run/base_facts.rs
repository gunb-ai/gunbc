//! THE FLOOR JUDGES BASE-REVISION FACTS WITH THE BASE REVISION'S OWN COMPILER.
//!
//! The required floor compares a change against its base, and three of the base-side facts it
//! needs are EVALUATED, not read: the cost-debt roster (`v2.workflow.floor_cost_debt`
//! `floor_cost_debt_roster`), the parse environment (`extdeps.languages.dag.syntax`
//! `dag_parse_environment`) and the kernel-name set (`std.types` `kernel_type_set`). Evaluating a
//! base closure with the HEAD binary is judging one revision with another revision's compiler: a
//! head that deletes a builtin the base still calls makes the base unevaluable (gunbc#13378, the
//! `from_code_point` deletion), and a head that changes a builtin's meaning makes it silently
//! different. So the head never evaluates a base closure. It asks the base revision's own
//! `claim_executor` -- the merge queue's compiler pair candidate for that revision, fetched and
//! verified by the floor job (`gunbc.compiler_gate_workflow`) -- through `--base-fact`, and reads
//! the answer as data.
//!
//! THE CONTRACT IS VERSIONED. The verb's output is one JSON object carrying `schema`; a schema the
//! reading binary does not know REFUSES as `BaseFactsSchemaUnknown`, never best-effort. The verb
//! only exists on revisions that landed with it, so a base older than this module has no verb and
//! its compiler refuses typed (`BaseFactRefused`) -- the capability turns on one landing later.
//!
//! NO FALLBACK TO THE HEAD SEED (DESIGN section 5). Every arm that cannot obtain the base's own
//! answer is a typed refusal naming why: no compiler for that revision (`BaseCompilerUnavailable`),
//! its merge-queue run still in flight (`BaseCompilerPending`), a compiler supplied for a different
//! revision than the one this binary decided is the base (`BaseCompilerRevisionMismatch`).

use std::path::{Path, PathBuf};

/// The version of the `--base-fact` output contract this binary emits AND the only one it reads.
/// Bump it with any change to `BaseFactKind`'s names or any fact's value shape.
pub const BASE_FACTS_SCHEMA: u64 = 1;

/// The facts the floor reads at the base, each evaluated by the base revision's own compiler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaseFactKind {
    /// `floor_cost_debt_roster` at the revision: a list of qualified identities.
    CostDebtRoster,
    /// `dag_parse_environment` at the revision: the `ParseEnvironment` wire value.
    ParseEnvironment,
    /// The keys of `kernel_type_set` at the revision: a list of names.
    KernelNames,
}

pub fn base_fact_kind_name(kind: BaseFactKind) -> &'static str {
    match kind {
        BaseFactKind::CostDebtRoster => "cost_debt_roster",
        BaseFactKind::ParseEnvironment => "parse_environment",
        BaseFactKind::KernelNames => "kernel_names",
    }
}

pub fn parse_base_fact_kind(name: &str) -> Result<BaseFactKind, String> {
    [
        BaseFactKind::CostDebtRoster,
        BaseFactKind::ParseEnvironment,
        BaseFactKind::KernelNames,
    ]
    .into_iter()
    .find(|k| base_fact_kind_name(*k) == name)
    .ok_or_else(|| {
        format!(
            "--base-fact {name:?} names no base fact; known: cost_debt_roster, \
             parse_environment, kernel_names"
        )
    })
}

/// Why the base revision's own answer could not be obtained. Every arm is a refusal; none admits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BaseFactRefusal {
    /// No compiler exists for the base revision and none will: it landed without a merge-queue
    /// run (a direct or admin merge), or its candidate artifact expired. Merging main in moves the
    /// base to a revision that has one.
    BaseCompilerUnavailable { revision: String, why: String },
    /// The base revision's merge-queue run is still in flight, so its compiler does not exist YET.
    /// An ordering miss, not a defect: the same change is judgeable once that run finishes.
    BaseCompilerPending { revision: String, run: String },
    /// The floor job supplied a compiler for a revision other than the base this binary decided.
    BaseCompilerRevisionMismatch { decided: String, supplied: String },
    /// The base compiler ran and did not answer: it refused the fact, lacks the verb (a base older
    /// than this contract), or could not be spawned.
    BaseFactRefused {
        revision: String,
        fact: &'static str,
        cause: String,
    },
    /// The base compiler answered under a contract version this binary does not read.
    BaseFactsSchemaUnknown { revision: String, emitted: String },
    /// The answer is not the shape the contract declares for that fact.
    BaseFactMalformed {
        revision: String,
        fact: &'static str,
        cause: String,
    },
}

pub fn base_fact_refusal_text(refusal: &BaseFactRefusal) -> String {
    match refusal {
        BaseFactRefusal::BaseCompilerUnavailable { revision, why } => format!(
            "cause=BaseCompilerUnavailable revision={revision} -- no compiler built from the base \
             revision is available ({why}); base facts are judged only by the base's own \
             compiler, never this one. Merge main in to move the base to a queue-landed revision"
        ),
        BaseFactRefusal::BaseCompilerPending { revision, run } => format!(
            "cause=BaseCompilerPending revision={revision} run={run} -- the base revision's \
             merge-queue run is still in flight, so its compiler does not exist yet; this is an \
             ordering miss, judgeable once that run completes"
        ),
        BaseFactRefusal::BaseCompilerRevisionMismatch { decided, supplied } => format!(
            "cause=BaseCompilerRevisionMismatch decided={decided} supplied={supplied} -- the \
             floor job supplied a compiler for a revision other than the base this binary decided"
        ),
        BaseFactRefusal::BaseFactRefused {
            revision,
            fact,
            cause,
        } => format!(
            "cause=BaseFactRefused revision={revision} fact={fact} -- the base compiler did not \
             answer: {cause}"
        ),
        BaseFactRefusal::BaseFactsSchemaUnknown { revision, emitted } => format!(
            "cause=BaseFactsSchemaUnknown revision={revision} emitted={emitted} known=\
             {BASE_FACTS_SCHEMA} -- the base compiler answered under a contract this binary does \
             not read"
        ),
        BaseFactRefusal::BaseFactMalformed {
            revision,
            fact,
            cause,
        } => format!(
            "cause=BaseFactMalformed revision={revision} fact={fact} -- the answer is not the \
             declared shape: {cause}"
        ),
    }
}

/// What the floor job supplied about the base compiler. Read from the environment the job sets
/// after its fetch-and-verify steps; the REVISION is only a claim this binary checks against its
/// own decision, never the authority for which commit is the base.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BaseCompilerSupply {
    /// The job ran no fetch at all (a local run, or a job that does not supply one).
    NotSupplied,
    Absent {
        revision: String,
        why: String,
    },
    Pending {
        revision: String,
        run: String,
    },
    Present {
        revision: String,
        executable: PathBuf,
    },
}

pub const BASE_COMPILER_STANDING_ENV: &str = "GUNBC_BASE_COMPILER_STANDING";
pub const BASE_COMPILER_REVISION_ENV: &str = "GUNBC_BASE_COMPILER_REVISION";
pub const BASE_COMPILER_EXECUTABLE_ENV: &str = "GUNBC_BASE_COMPILER";
pub const BASE_COMPILER_DETAIL_ENV: &str = "GUNBC_BASE_COMPILER_DETAIL";

/// The supply as the floor job's environment states it. An unrecognised standing is not absence.
pub fn base_compiler_supply_from_env() -> Result<BaseCompilerSupply, String> {
    let var = |k: &str| std::env::var(k).ok().filter(|v| !v.is_empty());
    let Some(standing) = var(BASE_COMPILER_STANDING_ENV) else {
        return Ok(BaseCompilerSupply::NotSupplied);
    };
    let revision = var(BASE_COMPILER_REVISION_ENV).ok_or_else(|| {
        format!("{BASE_COMPILER_STANDING_ENV}={standing} with no {BASE_COMPILER_REVISION_ENV}")
    })?;
    let detail = var(BASE_COMPILER_DETAIL_ENV).unwrap_or_default();
    match standing.as_str() {
        "absent" => Ok(BaseCompilerSupply::Absent { revision, why: detail }),
        "pending" => Ok(BaseCompilerSupply::Pending { revision, run: detail }),
        "present" => Ok(BaseCompilerSupply::Present {
            revision,
            executable: var(BASE_COMPILER_EXECUTABLE_ENV).map(PathBuf::from).ok_or_else(|| {
                format!("{BASE_COMPILER_STANDING_ENV}=present with no {BASE_COMPILER_EXECUTABLE_ENV}")
            })?,
        }),
        other => Err(format!(
            "{BASE_COMPILER_STANDING_ENV}={other:?} is not one of present, absent, pending"
        )),
    }
}

/// THE HEAD SIDE: `kind` at `revision`, answered by the base revision's own compiler, run in `repo`.
pub fn base_fact_from_base_compiler(
    supply: &BaseCompilerSupply,
    repo: &Path,
    revision: &str,
    kind: BaseFactKind,
) -> Result<serde_json::Value, BaseFactRefusal> {
    let fact = base_fact_kind_name(kind);
    let executable = match supply {
        BaseCompilerSupply::NotSupplied => {
            return Err(BaseFactRefusal::BaseCompilerUnavailable {
                revision: revision.to_string(),
                why: format!(
                    "this run was given no base compiler ({BASE_COMPILER_STANDING_ENV} unset)"
                ),
            })
        }
        BaseCompilerSupply::Absent { revision: r, .. }
        | BaseCompilerSupply::Pending { revision: r, .. }
        | BaseCompilerSupply::Present { revision: r, .. }
            if r != revision =>
        {
            return Err(BaseFactRefusal::BaseCompilerRevisionMismatch {
                decided: revision.to_string(),
                supplied: r.clone(),
            })
        }
        BaseCompilerSupply::Absent { why, .. } => {
            return Err(BaseFactRefusal::BaseCompilerUnavailable {
                revision: revision.to_string(),
                why: why.clone(),
            })
        }
        BaseCompilerSupply::Pending { run, .. } => {
            return Err(BaseFactRefusal::BaseCompilerPending {
                revision: revision.to_string(),
                run: run.clone(),
            })
        }
        BaseCompilerSupply::Present { executable, .. } => executable,
    };
    let refused = |cause: String| BaseFactRefusal::BaseFactRefused {
        revision: revision.to_string(),
        fact,
        cause,
    };
    let out = std::process::Command::new(executable)
        .args(["--base-fact", fact, "--base-revision", revision])
        .current_dir(repo)
        .output()
        .map_err(|e| refused(format!("spawn {}: {e}", executable.display())))?;
    if !out.status.success() {
        return Err(refused(format!(
            "exited {}: {}",
            out.status,
            String::from_utf8_lossy(&out.stderr).trim()
        )));
    }
    read_base_fact_answer(&out.stdout, revision, kind)
}

/// Decode one `--base-fact` answer against the contract this binary reads.
pub fn read_base_fact_answer(
    stdout: &[u8],
    revision: &str,
    kind: BaseFactKind,
) -> Result<serde_json::Value, BaseFactRefusal> {
    let fact = base_fact_kind_name(kind);
    let malformed = |cause: String| BaseFactRefusal::BaseFactMalformed {
        revision: revision.to_string(),
        fact,
        cause,
    };
    let answer: serde_json::Value =
        serde_json::from_slice(stdout).map_err(|e| malformed(format!("not JSON: {e}")))?;
    match answer.get("schema") {
        Some(serde_json::Value::Number(n)) if n.as_u64() == Some(BASE_FACTS_SCHEMA) => {}
        other => {
            return Err(BaseFactRefusal::BaseFactsSchemaUnknown {
                revision: revision.to_string(),
                emitted: other.map_or("<missing>".to_string(), |v| v.to_string()),
            })
        }
    }
    if answer.get("fact").and_then(|v| v.as_str()) != Some(fact) {
        return Err(malformed(format!("answers fact {:?}", answer.get("fact"))));
    }
    if answer.get("revision").and_then(|v| v.as_str()) != Some(revision) {
        return Err(malformed(format!(
            "answers revision {:?}",
            answer.get("revision")
        )));
    }
    answer
        .get("value")
        .cloned()
        .ok_or_else(|| malformed("carries no value".to_string()))
}

/// A list-of-names fact as names.
pub fn base_fact_names(
    value: serde_json::Value,
    revision: &str,
    kind: BaseFactKind,
) -> Result<Vec<String>, BaseFactRefusal> {
    serde_json::from_value::<Vec<String>>(value).map_err(|e| BaseFactRefusal::BaseFactMalformed {
        revision: revision.to_string(),
        fact: base_fact_kind_name(kind),
        cause: format!("not a list of names: {e}"),
    })
}

/// THE BASE SIDE: this binary, built from the base revision, evaluates `kind` at `revision` out of
/// `repo` and returns the contract's answer object. Run as `claim_executor --base-fact`.
pub fn emit_base_fact(
    repo: &Path,
    revision: &str,
    kind: BaseFactKind,
) -> Result<serde_json::Value, String> {
    use super::namespace_baseline as nb;
    let value = match kind {
        BaseFactKind::CostDebtRoster => {
            let tree = super::cost_debt_scratch_dir("base-fact")?;
            let read = super::cost_debt_base_tree_extract(repo, revision, &tree)
                .and_then(|()| super::cost_debt_roster_in_tree(&tree));
            std::fs::remove_dir_all(&tree).ok();
            serde_json::to_value(read?).map_err(|e| e.to_string())?
        }
        BaseFactKind::ParseEnvironment => {
            let env = nb::load_parse_environment_at(repo, revision)
                .map_err(|e| nb::environment_load_refusal_text(&e))?;
            serde_json::to_value(&*env).map_err(|e| e.to_string())?
        }
        BaseFactKind::KernelNames => {
            let names = nb::kernel_names_at(repo, revision, &nb::LiveDagIndex::new())
                .map_err(|e| nb::environment_load_refusal_text(&e))?;
            serde_json::to_value(names).map_err(|e| e.to_string())?
        }
    };
    Ok(serde_json::json!({
        "schema": BASE_FACTS_SCHEMA,
        "fact": base_fact_kind_name(kind),
        "revision": revision,
        "value": value,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn answer(schema: serde_json::Value) -> Vec<u8> {
        serde_json::json!({"schema": schema, "fact": "kernel_names", "revision": "r", "value": ["Int"]})
            .to_string()
            .into_bytes()
    }

    #[test]
    fn an_unknown_contract_version_refuses_never_best_effort() {
        let err = read_base_fact_answer(
            &answer(serde_json::json!(2)),
            "r",
            BaseFactKind::KernelNames,
        )
        .expect_err("schema 2 is unknown");
        assert!(
            matches!(err, BaseFactRefusal::BaseFactsSchemaUnknown { .. }),
            "{err:?}"
        );
        let ok = read_base_fact_answer(
            &answer(serde_json::json!(BASE_FACTS_SCHEMA)),
            "r",
            BaseFactKind::KernelNames,
        )
        .expect("the known version reads");
        assert_eq!(
            base_fact_names(ok, "r", BaseFactKind::KernelNames).unwrap(),
            vec!["Int"]
        );
    }

    #[test]
    fn an_answer_for_another_fact_or_revision_is_malformed() {
        let a = answer(serde_json::json!(BASE_FACTS_SCHEMA));
        assert!(matches!(
            read_base_fact_answer(&a, "r", BaseFactKind::CostDebtRoster),
            Err(BaseFactRefusal::BaseFactMalformed { .. })
        ));
        assert!(matches!(
            read_base_fact_answer(&a, "other", BaseFactKind::KernelNames),
            Err(BaseFactRefusal::BaseFactMalformed { .. })
        ));
    }

    #[test]
    fn every_non_present_supply_refuses_with_its_own_cause_and_never_evaluates_here() {
        let repo = Path::new(".");
        let ask = |s: &BaseCompilerSupply| {
            base_fact_from_base_compiler(s, repo, "base", BaseFactKind::KernelNames).unwrap_err()
        };
        assert!(matches!(
            ask(&BaseCompilerSupply::NotSupplied),
            BaseFactRefusal::BaseCompilerUnavailable { .. }
        ));
        assert!(matches!(
            ask(&BaseCompilerSupply::Absent {
                revision: "base".into(),
                why: "admin merge".into()
            }),
            BaseFactRefusal::BaseCompilerUnavailable { .. }
        ));
        assert_eq!(
            ask(&BaseCompilerSupply::Pending {
                revision: "base".into(),
                run: "42".into()
            }),
            BaseFactRefusal::BaseCompilerPending {
                revision: "base".into(),
                run: "42".into()
            }
        );
        assert!(matches!(
            ask(&BaseCompilerSupply::Present {
                revision: "other".into(),
                executable: "/bin/true".into()
            }),
            BaseFactRefusal::BaseCompilerRevisionMismatch { .. }
        ));
    }

    #[test]
    fn a_base_compiler_without_the_verb_refuses_as_base_fact_refused() {
        // `/bin/false` stands for a base older than this contract: it exits non-zero to the verb.
        let s = BaseCompilerSupply::Present {
            revision: "base".into(),
            executable: "/bin/false".into(),
        };
        assert!(matches!(
            base_fact_from_base_compiler(&s, Path::new("."), "base", BaseFactKind::CostDebtRoster),
            Err(BaseFactRefusal::BaseFactRefused { .. })
        ));
    }
}
