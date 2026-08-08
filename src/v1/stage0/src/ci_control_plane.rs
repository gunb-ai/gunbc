//! Owned CI control plane v0 — poll-first subject discovery, durable queue,
//! one local executor, GitHub Checks projection.
//!
//! Authority: `docs/plans/owned-ci-control-plane-design.md`
//!
//! SCAFFOLD — seed-retained host transport; projection authority `gunbc.owned_ci_seed`.
//! dissolve-on: REST bridge; .dag state coproducts replace Rust JSON carriers; subject_key/run_id eval.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::cli_run::{
    make_eval_context, resolve_entry_graph_shared, run_value, witness_layer_roots,
};
use crate::v1_interpreter::{ExecutionMode, Value};
use serde::{Deserialize, Serialize};

pub const DEFAULT_POLL_INTERVAL_SECS: u64 = 60;

/// Receipt marker for scaffold census (`rg OWNED_CI_CONTROL_PLANE_SEED_SCAFFOLD_MARKER`).
pub const OWNED_CI_CONTROL_PLANE_SEED_SCAFFOLD_MARKER: &str =
    "OWNED_CI_CONTROL_PLANE_SEED_SCAFFOLD_MARKER";

const OWNED_CI_SEED_ENTRY: &str = "dag/gunbc/owned_ci_seed.dag";

#[derive(Debug, Clone)]
pub struct SeedAuthority {
    pub repo_owner: String,
    pub repo_name: String,
    pub repo_full_name: String,
    pub check_name: String,
    pub state_root_rel: String,
    pub default_lease_seconds: u64,
    pub stage_labels: Vec<String>,
    pub floor_plan_entry: String,
    pub floor_plan_function: String,
    pub poll_missing_github_token_refusal: String,
    pub poll_empty_github_token_refusal: String,
    pub merge_target_branch: String,
}

impl SeedAuthority {
    pub fn load() -> Result<Self, String> {
        let roots = witness_layer_roots();
        let (graph, si) = resolve_entry_graph_shared(&roots, OWNED_CI_SEED_ENTRY)?;
        let ctx = make_eval_context(&graph, si, ExecutionMode::Wet);
        Ok(Self {
            repo_owner: eval_string(&ctx, "owned_ci_seed_repo_owner")?,
            repo_name: eval_string(&ctx, "owned_ci_seed_repo_name")?,
            repo_full_name: eval_string(&ctx, "owned_ci_seed_repo_full_name")?,
            check_name: eval_string(&ctx, "owned_ci_seed_check_name")?,
            state_root_rel: eval_string(&ctx, "owned_ci_seed_state_root_rel")?,
            default_lease_seconds: eval_int(&ctx, "owned_ci_seed_default_lease_seconds")?,
            stage_labels: eval_string_list(&ctx, "owned_ci_seed_stage_labels")?,
            floor_plan_entry: eval_string(&ctx, "owned_ci_seed_floor_plan_entry")?,
            floor_plan_function: eval_string(&ctx, "owned_ci_seed_floor_plan_function")?,
            poll_missing_github_token_refusal: eval_string(
                &ctx,
                "owned_ci_seed_poll_missing_github_token_refusal",
            )?,
            poll_empty_github_token_refusal: eval_string(
                &ctx,
                "owned_ci_seed_poll_empty_github_token_refusal",
            )?,
            merge_target_branch: eval_string(&ctx, "owned_ci_seed_merge_target_branch")?,
        })
    }
}

fn eval_int(ctx: &crate::v1_interpreter::InterpContext, function: &str) -> Result<u64, String> {
    match run_value(ctx, function)? {
        Value::Int(n) if n >= 0 => Ok(n as u64),
        Value::Int(n) => Err(format!("{function} must be non-negative Int, got {n}")),
        other => Err(format!(
            "{function} must return Int, got {other:?} (fail-closed)"
        )),
    }
}

fn eval_string(
    ctx: &crate::v1_interpreter::InterpContext,
    function: &str,
) -> Result<String, String> {
    match run_value(ctx, function)? {
        Value::Str(s) => Ok(s),
        other => Err(format!(
            "{function} must return String, got {other:?} (fail-closed)"
        )),
    }
}

fn eval_string_list(
    ctx: &crate::v1_interpreter::InterpContext,
    function: &str,
) -> Result<Vec<String>, String> {
    match run_value(ctx, function)? {
        Value::List(items) => items
            .iter()
            .map(|item| match item {
                Value::Str(s) => Ok(s.clone()),
                other => Err(format!(
                    "{function} list items must be String, got {other:?} (fail-closed)"
                )),
            })
            .collect(),
        other => Err(format!(
            "{function} must return List, got {other:?} (fail-closed)"
        )),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub workspace_root: PathBuf,
    pub mirror_path: PathBuf,
    pub serve_base_url: String,
    pub poll_interval_secs: u64,
    pub lease_secs: u64,
    pub lease_holder: String,
    pub once: bool,
    pub dry_run: bool,
    pub enqueue_rerun_sha: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let workspace_root = std::env::var("GUNBC_WORKSPACE_ROOT")
            .map(PathBuf::from)
            .or_else(|_| std::env::current_dir().map_err(|e| format!("cwd unavailable: {e}")))?;
        let mirror_path = std::env::var("OWNED_CI_MIRROR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| workspace_root.join(".gunbc/owned-ci/mirror.git"));
        let serve_base_url = std::env::var("OWNED_CI_SERVE_BASE_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:8787".to_string());
        let once = std::env::var("OWNED_CI_ONCE").is_ok();
        let dry_run = std::env::var("OWNED_CI_DRY_RUN").is_ok();
        let enqueue_rerun_sha = std::env::var("OWNED_CI_ENQUEUE_RERUN_SHA").ok();
        let lease_holder = format!("{}-{}", hostname(), std::process::id());
        Ok(Self {
            workspace_root,
            mirror_path,
            serve_base_url,
            poll_interval_secs: DEFAULT_POLL_INTERVAL_SECS,
            lease_secs: 0,
            lease_holder,
            once,
            dry_run,
            enqueue_rerun_sha,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnixWholeSeconds {
    pub seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubjectLedger {
    pub main_head_sha: Option<String>,
    pub subjects: Vec<SubjectEntry>,
    #[serde(default)]
    pub last_pr_poll_refusal: Option<String>,
    #[serde(default)]
    pub last_pr_poll_refusal_at_unix: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubjectEntry {
    pub subject_key: String,
    pub head_sha: String,
    pub kind: String,
    pub pr_number: Option<u64>,
    pub last_enqueued_run_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunIndex {
    pub runs: Vec<RunIndexRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunIndexRow {
    pub run_id: String,
    pub head_sha: String,
    pub subject_key: String,
    pub status: String,
    pub conclusion: Option<String>,
    pub created_at_unix: u64,
    pub updated_at_unix: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueLease {
    pub holder: String,
    pub expires_at: UnixWholeSeconds,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageRecord {
    pub stage: String,
    pub status: String,
    pub detail: Option<String>,
    pub started_at_unix: Option<u64>,
    pub finished_at_unix: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicationState {
    pub kind: String,
    pub check_run_id: Option<u64>,
    pub local_conclusion: Option<String>,
    pub details_url: Option<String>,
    pub cause: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunRecord {
    pub run_id: String,
    pub head_sha: String,
    pub subject_key: String,
    pub queue_state: String,
    pub publication_state: PublicationState,
    pub stages: Vec<StageRecord>,
    pub details_url: String,
    pub lease: Option<QueueLease>,
    pub created_at_unix: u64,
    pub updated_at_unix: u64,
}

#[derive(Debug, Clone)]
pub struct DiscoveredSubject {
    pub subject_key: String,
    pub head_sha: String,
    pub kind: String,
    pub pr_number: Option<u64>,
}

#[derive(Debug, Clone)]
struct ForkRefusal {
    pr_number: u64,
    head_repo_full_name: String,
    head_sha: String,
}

#[derive(Debug, Clone)]
struct NonMainBaseRefusal {
    pr_number: u64,
    base_ref: String,
    head_sha: String,
}

#[derive(Debug, Clone)]
struct DiscoverOutcome {
    admitted: Vec<DiscoveredSubject>,
    fork_refusals: Vec<ForkRefusal>,
    non_main_base_refusals: Vec<NonMainBaseRefusal>,
    pr_poll_refusal: Option<String>,
}

pub struct CiControlPlane {
    pub config: Config,
    pub authority: SeedAuthority,
}

impl CiControlPlane {
    pub fn new(config: Config) -> Result<Self, String> {
        let authority = SeedAuthority::load()?;
        let mut config = config;
        config.lease_secs = authority.default_lease_seconds;
        Ok(Self { config, authority })
    }

    fn state_root(&self) -> PathBuf {
        self.config
            .workspace_root
            .join(&self.authority.state_root_rel)
    }

    pub fn ensure_layout(&self) -> Result<(), String> {
        let root = self.state_root();
        for rel in [
            "queue/pending",
            "queue/claimed",
            "queue/completed",
            "runs",
            "receipts",
            "worktrees",
        ] {
            fs::create_dir_all(root.join(rel)).map_err(|e| format!("mkdir {}: {e}", rel))?;
        }
        if !self.config.mirror_path.exists() {
            fs::create_dir_all(self.config.mirror_path.parent().unwrap_or(Path::new(".")))
                .map_err(|e| format!("mkdir mirror parent: {e}"))?;
            run_cmd(
                &self.config.workspace_root,
                Command::new("git")
                    .arg("clone")
                    .arg("--bare")
                    .arg(format!(
                        "https://github.com/{}.git",
                        self.authority.repo_full_name
                    ))
                    .arg(&self.config.mirror_path),
            )?;
        }
        if !root.join("index.json").exists() {
            self.write_index(&RunIndex { runs: vec![] })?;
        }
        if !root.join("subject-ledger.json").exists() {
            self.write_ledger(&SubjectLedger {
                main_head_sha: None,
                subjects: vec![],
                last_pr_poll_refusal: None,
                last_pr_poll_refusal_at_unix: None,
            })?;
        }
        Ok(())
    }

    pub fn run_loop(&mut self) -> Result<(), String> {
        self.ensure_layout()?;
        loop {
            self.poll_once()?;
            if self.config.once {
                break;
            }
            thread::sleep(Duration::from_secs(self.config.poll_interval_secs));
        }
        Ok(())
    }

    pub fn poll_once(&mut self) -> Result<(), String> {
        self.fetch_mirror()?;
        self.reclaim_expired_claims()?;
        let outcome = self.discover_subjects()?;
        self.reconcile_subjects(&outcome.admitted)?;
        if let Some(cause) = &outcome.pr_poll_refusal {
            self.record_pr_poll_refusal(cause)?;
        }
        self.reconcile_fork_refusals(&outcome.fork_refusals)?;
        self.reconcile_non_main_base_refusals(&outcome.non_main_base_refusals)?;
        if self.config.dry_run {
            if let Some(run_id) = self.peek_next_pending()? {
                eprintln!("owned-ci: dry-run would execute {run_id} (queue left pending)");
            }
        } else if let Some(run_id) = self.claim_next_pending()? {
            self.execute_run(&run_id)?;
        }
        Ok(())
    }

    fn peek_next_pending(&self) -> Result<Option<String>, String> {
        let pending_dir = self.state_root().join("queue/pending");
        let mut entries: Vec<_> = fs::read_dir(&pending_dir)
            .map_err(|e| format!("read pending queue: {e}"))?
            .filter_map(Result::ok)
            .filter(|e| e.path().extension().is_some_and(|x| x == "json"))
            .collect();
        entries.sort_by_key(|e| e.file_name());
        Ok(entries.first().and_then(|entry| {
            entry
                .path()
                .file_stem()
                .and_then(|s| s.to_str())
                .map(str::to_string)
        }))
    }

    fn reclaim_expired_claims(&mut self) -> Result<(), String> {
        let claimed_dir = self.state_root().join("queue/claimed");
        let pending_dir = self.state_root().join("queue/pending");
        let now = now_unix()?;
        let entries: Vec<_> = fs::read_dir(&claimed_dir)
            .map_err(|e| format!("read claimed queue: {e}"))?
            .filter_map(Result::ok)
            .filter(|e| e.path().extension().is_some_and(|x| x == "json"))
            .collect();
        for entry in entries {
            let run_id = entry
                .path()
                .file_stem()
                .and_then(|s| s.to_str())
                .ok_or_else(|| "claimed queue file has no stem".to_string())?
                .to_string();
            let mut record = self.read_run(&run_id)?;
            let reclaim = record
                .lease
                .as_ref()
                .is_some_and(|lease| now >= lease.expires_at.seconds);
            if !reclaim {
                continue;
            }
            let from = entry.path();
            let to = pending_dir.join(format!("{run_id}.json"));
            fs::rename(&from, &to).map_err(|e| format!("reclaim rename {run_id}: {e}"))?;
            record.queue_state = "pending".to_string();
            record.lease = None;
            record.updated_at_unix = now;
            self.write_run(&record)?;
            self.update_index_row(&run_id, "pending", None)?;
            eprintln!("owned-ci: reclaimed expired lease for run {run_id}");
        }
        Ok(())
    }

    fn fetch_mirror(&self) -> Result<(), String> {
        run_cmd(
            &self.config.workspace_root,
            Command::new("git")
                .arg("-C")
                .arg(&self.config.mirror_path)
                .arg("fetch")
                .arg("--prune")
                .arg("origin")
                .arg(&self.authority.merge_target_branch),
        )?;
        run_cmd(
            &self.config.workspace_root,
            Command::new("git")
                .arg("-C")
                .arg(&self.config.mirror_path)
                .args([
                    "fetch",
                    "origin",
                    "+refs/pull/*/head:refs/remotes/origin/pr/*",
                ]),
        )?;
        Ok(())
    }

    fn discover_subjects(&self) -> Result<DiscoverOutcome, String> {
        let mut admitted = Vec::new();
        let branch = &self.authority.merge_target_branch;
        let merge_target_sha = rev_parse(
            &self.config.mirror_path,
            &format!("refs/remotes/origin/{branch}"),
        )
        .or_else(|_| rev_parse(&self.config.mirror_path, &format!("refs/heads/{branch}")))?;
        admitted.push(DiscoveredSubject {
            subject_key: format!("{branch}:{merge_target_sha}"),
            head_sha: merge_target_sha,
            kind: "main_push".to_string(),
            pr_number: None,
        });

        if let Some(rerun_sha) = &self.config.enqueue_rerun_sha {
            validate_commit_sha(rerun_sha)?;
            admitted.push(DiscoveredSubject {
                subject_key: format!("rerun:{rerun_sha}"),
                head_sha: rerun_sha.clone(),
                kind: "operator_rerun".to_string(),
                pr_number: None,
            });
        }

        let (prs, fork_refusals, non_main_base_refusals, pr_poll_refusal) = if self.config.dry_run {
            eprintln!("owned-ci: dry-run skips GitHub PR poll (mirror-only subject discovery)");
            (Vec::new(), Vec::new(), Vec::new(), None)
        } else {
            let token = std::env::var("GITHUB_TOKEN");
            match token {
                Ok(token) if token.is_empty() => (
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    Some(self.authority.poll_empty_github_token_refusal.clone()),
                ),
                Ok(token) => match self.discover_open_prs(&token) {
                    Ok((prs, fork_refusals, non_main_base_refusals)) => {
                        (prs, fork_refusals, non_main_base_refusals, None)
                    }
                    Err(cause) => (Vec::new(), Vec::new(), Vec::new(), Some(cause)),
                },
                Err(_) => (
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    Some(self.authority.poll_missing_github_token_refusal.clone()),
                ),
            }
        };
        admitted.extend(prs);
        Ok(DiscoverOutcome {
            admitted,
            fork_refusals,
            non_main_base_refusals,
            pr_poll_refusal,
        })
    }

    fn record_pr_poll_refusal(&self, cause: &str) -> Result<(), String> {
        let mut ledger = self.read_ledger()?;
        ledger.last_pr_poll_refusal = Some(cause.to_string());
        ledger.last_pr_poll_refusal_at_unix = Some(now_unix()?);
        self.write_ledger(&ledger)?;
        eprintln!("owned-ci: PR poll refused (main mirror reconciliation still proceeds): {cause}");
        Ok(())
    }

    fn discover_open_prs(
        &self,
        token: &str,
    ) -> Result<
        (
            Vec<DiscoveredSubject>,
            Vec<ForkRefusal>,
            Vec<NonMainBaseRefusal>,
        ),
        String,
    > {
        let mut admitted = Vec::new();
        let mut fork_refusals = Vec::new();
        let mut non_main_base_refusals = Vec::new();
        let mut url = format!(
            "https://api.github.com/repos/{}/pulls?state=open&per_page=100",
            self.authority.repo_full_name
        );

        loop {
            let response = ureq::get(&url)
                .set("Authorization", &format!("Bearer {token}"))
                .set("Accept", "application/vnd.github+json")
                .set("User-Agent", "gunbc-owned-ci")
                .call()
                .map_err(|e| format!("list pulls failed: {e}"))?;
            let status = response.status();
            let link_header = response.header("Link").unwrap_or_default().to_string();
            let body = response
                .into_string()
                .map_err(|e| format!("read pulls body: {e}"))?;
            if status != 200 {
                return Err(format!("list pulls HTTP {status}: {body}"));
            }
            let pulls: Vec<serde_json::Value> =
                serde_json::from_str(&body).map_err(|e| format!("parse pulls json: {e}"))?;
            self.extend_open_pull_subjects(
                &pulls,
                &mut admitted,
                &mut fork_refusals,
                &mut non_main_base_refusals,
            )?;

            match parse_github_link_rel_next(&link_header) {
                Some(next_url) => url = next_url,
                None => {
                    if link_header.contains("rel=\"next\"") || link_header.contains("rel=next") {
                        return Err(
                            "list pulls pagination truncated: Link header advertises rel=next but next URL could not be parsed"
                                .to_string(),
                        );
                    }
                    break;
                }
            }
        }

        Ok((admitted, fork_refusals, non_main_base_refusals))
    }

    fn extend_open_pull_subjects(
        &self,
        pulls: &[serde_json::Value],
        admitted: &mut Vec<DiscoveredSubject>,
        fork_refusals: &mut Vec<ForkRefusal>,
        non_main_base_refusals: &mut Vec<NonMainBaseRefusal>,
    ) -> Result<(), String> {
        for pull in pulls {
            let number = pull["number"].as_u64();
            let head_sha = pull["head"]["sha"].as_str().map(str::to_string);
            let head_repo = pull["head"]["repo"]["full_name"].as_str();
            let (Some(number), Some(head_sha)) = (number, head_sha) else {
                let snippet =
                    serde_json::to_string(pull).unwrap_or_else(|_| "<unserializable>".to_string());
                return Err(format!(
                    "list pulls malformed payload: open PR missing number or head.sha (fail-closed, not silent skip): {snippet}"
                ));
            };
            let base_ref = pull["base"]["ref"].as_str();
            let Some(base_ref) = base_ref else {
                let snippet =
                    serde_json::to_string(pull).unwrap_or_else(|_| "<unserializable>".to_string());
                return Err(format!(
                    "list pulls malformed payload: open PR missing base.ref (fail-closed, not silent skip): {snippet}"
                ));
            };
            let Some(head_repo) = head_repo.filter(|repo| !repo.is_empty()) else {
                let snippet =
                    serde_json::to_string(pull).unwrap_or_else(|_| "<unserializable>".to_string());
                return Err(format!(
                    "list pulls malformed payload: open PR missing or empty head.repo.full_name (fail-closed, not silent skip): {snippet}"
                ));
            };
            if head_repo != self.authority.repo_full_name {
                fork_refusals.push(ForkRefusal {
                    pr_number: number,
                    head_repo_full_name: head_repo.to_string(),
                    head_sha,
                });
                continue;
            }
            if base_ref != self.authority.merge_target_branch {
                non_main_base_refusals.push(NonMainBaseRefusal {
                    pr_number: number,
                    base_ref: base_ref.to_string(),
                    head_sha,
                });
                continue;
            }
            admitted.push(DiscoveredSubject {
                subject_key: format!("pr:{head_sha}"),
                head_sha,
                kind: "pull_request".to_string(),
                pr_number: Some(number),
            });
        }
        Ok(())
    }

    fn reconcile_subjects(&mut self, discovered: &[DiscoveredSubject]) -> Result<(), String> {
        let mut ledger = self.read_ledger()?;
        let mut index = self.read_index()?;
        for subject in discovered {
            let already = ledger
                .subjects
                .iter()
                .any(|s| s.subject_key == subject.subject_key && s.last_enqueued_run_id.is_some());
            if already {
                continue;
            }
            let run_id = new_run_id(&subject.head_sha)?;
            let now = now_unix()?;
            let details_url = format!(
                "{}/ci/run/{}",
                self.config.serve_base_url.trim_end_matches('/'),
                run_id
            );
            let record = RunRecord {
                run_id: run_id.clone(),
                head_sha: subject.head_sha.clone(),
                subject_key: subject.subject_key.clone(),
                queue_state: "pending".to_string(),
                publication_state: PublicationState {
                    kind: "not_started".to_string(),
                    check_run_id: None,
                    local_conclusion: None,
                    details_url: Some(details_url.clone()),
                    cause: None,
                },
                stages: default_stages(&self.authority.stage_labels),
                details_url,
                lease: None,
                created_at_unix: now,
                updated_at_unix: now,
            };
            self.write_run(&record)?;
            self.enqueue_pending(&run_id, &record)?;
            index.runs.push(RunIndexRow {
                run_id: run_id.clone(),
                head_sha: subject.head_sha.clone(),
                subject_key: subject.subject_key.clone(),
                status: "pending".to_string(),
                conclusion: None,
                created_at_unix: now,
                updated_at_unix: now,
            });
            ledger.subjects.push(SubjectEntry {
                subject_key: subject.subject_key.clone(),
                head_sha: subject.head_sha.clone(),
                kind: subject.kind.clone(),
                pr_number: subject.pr_number,
                last_enqueued_run_id: Some(run_id.clone()),
            });
            if subject.kind == "main_push" {
                ledger.main_head_sha = Some(subject.head_sha.clone());
            }
            self.write_index(&index)?;
            self.write_ledger(&ledger)?;
            if !self.config.dry_run {
                self.create_check_or_mark_pending(&record)?;
            }
        }
        Ok(())
    }

    fn reconcile_fork_refusals(&mut self, refusals: &[ForkRefusal]) -> Result<(), String> {
        if refusals.is_empty() {
            return Ok(());
        }
        let mut ledger = self.read_ledger()?;
        let mut index = self.read_index()?;
        for refusal in refusals {
            let subject_key = format!("fork-refused:{}", refusal.head_repo_full_name);
            let already = ledger.subjects.iter().any(|s| {
                s.kind == "fork_refused"
                    && s.pr_number == Some(refusal.pr_number)
                    && s.last_enqueued_run_id.is_some()
            });
            if already {
                continue;
            }
            let run_id = new_run_id(&refusal.head_sha)?;
            let now = now_unix()?;
            let reason = format!(
                "fork PR #{} from {} refused in v0 (same-repo only)",
                refusal.pr_number, refusal.head_repo_full_name
            );
            let details_url = format!(
                "{}/ci/run/{}",
                self.config.serve_base_url.trim_end_matches('/'),
                run_id
            );
            let record = RunRecord {
                run_id: run_id.clone(),
                head_sha: refusal.head_sha.clone(),
                subject_key: subject_key.clone(),
                queue_state: "refused".to_string(),
                publication_state: PublicationState {
                    kind: "not_started".to_string(),
                    check_run_id: None,
                    local_conclusion: None,
                    details_url: Some(details_url.clone()),
                    cause: Some(reason.clone()),
                },
                stages: vec![],
                details_url,
                lease: None,
                created_at_unix: now,
                updated_at_unix: now,
            };
            self.write_run(&record)?;
            index.runs.push(RunIndexRow {
                run_id: run_id.clone(),
                head_sha: refusal.head_sha.clone(),
                subject_key: subject_key.clone(),
                status: "refused".to_string(),
                conclusion: Some("refused".to_string()),
                created_at_unix: now,
                updated_at_unix: now,
            });
            ledger.subjects.push(SubjectEntry {
                subject_key,
                head_sha: refusal.head_sha.clone(),
                kind: "fork_refused".to_string(),
                pr_number: Some(refusal.pr_number),
                last_enqueued_run_id: Some(run_id),
            });
            eprintln!("owned-ci: recorded fork PR refusal — {reason}");
        }
        self.write_index(&index)?;
        self.write_ledger(&ledger)?;
        Ok(())
    }

    fn reconcile_non_main_base_refusals(
        &mut self,
        refusals: &[NonMainBaseRefusal],
    ) -> Result<(), String> {
        if refusals.is_empty() {
            return Ok(());
        }
        let mut ledger = self.read_ledger()?;
        let mut index = self.read_index()?;
        for refusal in refusals {
            let subject_key = format!("non-main-refused:{}", refusal.base_ref);
            let already = ledger.subjects.iter().any(|s| {
                s.kind == "non_main_base_refused"
                    && s.pr_number == Some(refusal.pr_number)
                    && s.last_enqueued_run_id.is_some()
            });
            if already {
                continue;
            }
            let run_id = new_run_id(&refusal.head_sha)?;
            let now = now_unix()?;
            let reason = format!(
                "PR #{} targets {}; v0 admits same-repo PR → {} only",
                refusal.pr_number, refusal.base_ref, self.authority.merge_target_branch
            );
            let details_url = format!(
                "{}/ci/run/{}",
                self.config.serve_base_url.trim_end_matches('/'),
                run_id
            );
            let record = RunRecord {
                run_id: run_id.clone(),
                head_sha: refusal.head_sha.clone(),
                subject_key: subject_key.clone(),
                queue_state: "refused".to_string(),
                publication_state: PublicationState {
                    kind: "not_started".to_string(),
                    check_run_id: None,
                    local_conclusion: None,
                    details_url: Some(details_url.clone()),
                    cause: Some(reason.clone()),
                },
                stages: vec![],
                details_url,
                lease: None,
                created_at_unix: now,
                updated_at_unix: now,
            };
            self.write_run(&record)?;
            index.runs.push(RunIndexRow {
                run_id: run_id.clone(),
                head_sha: refusal.head_sha.clone(),
                subject_key: subject_key.clone(),
                status: "refused".to_string(),
                conclusion: Some("refused".to_string()),
                created_at_unix: now,
                updated_at_unix: now,
            });
            ledger.subjects.push(SubjectEntry {
                subject_key,
                head_sha: refusal.head_sha.clone(),
                kind: "non_main_base_refused".to_string(),
                pr_number: Some(refusal.pr_number),
                last_enqueued_run_id: Some(run_id),
            });
            eprintln!("owned-ci: recorded non-main-base PR refusal — {reason}");
        }
        self.write_index(&index)?;
        self.write_ledger(&ledger)?;
        Ok(())
    }

    fn claim_next_pending(&mut self) -> Result<Option<String>, String> {
        let pending_dir = self.state_root().join("queue/pending");
        let mut entries: Vec<_> = fs::read_dir(&pending_dir)
            .map_err(|e| format!("read pending queue: {e}"))?
            .filter_map(Result::ok)
            .filter(|e| e.path().extension().is_some_and(|x| x == "json"))
            .collect();
        entries.sort_by_key(|e| e.file_name());
        let Some(entry) = entries.first() else {
            return Ok(None);
        };
        let run_id = entry
            .path()
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| "pending queue file has no stem".to_string())?
            .to_string();
        let claimed_dir = self.state_root().join("queue/claimed");
        let from = entry.path();
        let to = claimed_dir.join(format!("{run_id}.json"));
        fs::rename(&from, &to).map_err(|e| format!("claim rename {run_id}: {e}"))?;
        let mut record = self.read_run(&run_id)?;
        let now = now_unix()?;
        record.queue_state = "claimed".to_string();
        record.lease = Some(QueueLease {
            holder: self.config.lease_holder.clone(),
            expires_at: UnixWholeSeconds {
                seconds: now + self.config.lease_secs,
            },
        });
        record.updated_at_unix = now;
        self.write_run(&record)?;
        self.update_index_row(&run_id, "claimed", None)?;
        if !self.config.dry_run {
            if let Err(cause) = self.try_update_check(&record, "in_progress", None) {
                self.mark_publication_pending(&record, None, &cause)?;
            }
        }
        Ok(Some(run_id))
    }

    fn execute_run(&mut self, run_id: &str) -> Result<(), String> {
        let mut record = self.read_run(&run_id)?;
        let worktree = self
            .state_root()
            .join("worktrees")
            .join(record.run_id.clone());
        if worktree.exists() {
            fs::remove_dir_all(&worktree)
                .map_err(|e| format!("remove stale worktree {run_id}: {e}"))?;
        }
        fs::create_dir_all(worktree.parent().unwrap())
            .map_err(|e| format!("mkdir worktree parent: {e}"))?;
        run_cmd(
            &self.config.workspace_root,
            Command::new("git")
                .arg("-C")
                .arg(&self.config.mirror_path)
                .args([
                    "worktree",
                    "add",
                    "--detach",
                    worktree.to_str().unwrap(),
                    &record.head_sha,
                ]),
        )?;

        let stages = self.authority.stage_labels.clone();
        let mut failed = false;
        for stage in &stages {
            let now = now_unix()?;
            set_stage_running(&mut record, stage, now);
            self.write_run(&record)?;
            let result = match stage.as_str() {
                "build" => self.stage_build(&worktree),
                "regen" => self.stage_regen(&worktree),
                "floor" => self.stage_floor(&worktree),
                other => Err(format!(
                    "unknown execution stage {other} from CiExecutionPlan"
                )),
            };
            let now = now_unix()?;
            match result {
                Ok(()) => set_stage_succeeded(&mut record, stage, now),
                Err(detail) => {
                    set_stage_failed(&mut record, stage, &detail, now);
                    failed = true;
                    break;
                }
            }
            self.append_stage_receipt(run_id, stage.as_str(), &record)?;
            self.write_run(&record)?;
        }

        let conclusion = if failed { "failure" } else { "success" };
        record.queue_state = if failed {
            "failed".to_string()
        } else {
            "completed".to_string()
        };
        record.updated_at_unix = now_unix()?;
        self.write_run(&record)?;
        self.update_index_row(run_id, &record.queue_state, Some(conclusion))?;
        self.move_queue_file(run_id, "claimed", "completed")?;
        self.write_run_summary_receipt(run_id, &record)?;

        if let Err(cause) = self.try_update_check(&record, "completed", Some(conclusion)) {
            self.mark_publication_pending(&record, Some(conclusion), &cause)?;
        }
        Ok(())
    }

    fn create_check_or_mark_pending(&self, record: &RunRecord) -> Result<(), String> {
        if let Err(cause) = self.try_create_check(record) {
            self.mark_publication_pending(record, None, &cause)?;
        }
        Ok(())
    }

    fn mark_publication_pending(
        &self,
        record: &RunRecord,
        local_conclusion: Option<&str>,
        cause: &str,
    ) -> Result<(), String> {
        let mut updated = record.clone();
        updated.publication_state = PublicationState {
            kind: "pending".to_string(),
            check_run_id: record.publication_state.check_run_id,
            local_conclusion: local_conclusion.map(str::to_string),
            details_url: Some(record.details_url.clone()),
            cause: Some(cause.to_string()),
        };
        updated.updated_at_unix = now_unix()?;
        self.write_run(&updated)
    }

    fn stage_build(&self, worktree: &Path) -> Result<(), String> {
        let status = Command::new("cargo")
            .arg("build")
            .arg("--workspace")
            .arg("--release")
            .current_dir(worktree)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|e| format!("cargo build spawn: {e}"))?;
        status_ok(status, "cargo build --workspace --release")
    }

    fn stage_regen(&self, worktree: &Path) -> Result<(), String> {
        let bin = worktree.join("target/release/regen_stage0");
        if !bin.exists() {
            return Err("regen_stage0 binary missing after build".to_string());
        }
        let status = Command::new(&bin)
            .current_dir(worktree)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|e| format!("regen_stage0 spawn: {e}"))?;
        status_ok(status, "regen_stage0")
    }

    fn stage_floor(&self, worktree: &Path) -> Result<(), String> {
        let bin = worktree.join("target/release/claim_executor");
        if !bin.exists() {
            return Err("claim_executor binary missing after build".to_string());
        }
        let status = Command::new(&bin)
            .args([
                "--source-root",
                "dag",
                "--source-root",
                "src/v2",
                "--plan-entry",
                &self.authority.floor_plan_entry,
                "--plan-function",
                &self.authority.floor_plan_function,
                "--notice-title",
                "owned-ci",
            ])
            .current_dir(worktree)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|e| format!("claim_executor spawn: {e}"))?;
        status_ok(status, "claim_executor gunbc_ci_floor_plan")
    }

    fn try_create_check(&self, record: &RunRecord) -> Result<(), String> {
        let token =
            std::env::var("GITHUB_TOKEN").map_err(|_| "GITHUB_TOKEN missing".to_string())?;
        if token.is_empty() {
            return Err("GITHUB_TOKEN empty".to_string());
        }
        let body = serde_json::json!({
            "name": self.authority.check_name,
            "head_sha": record.head_sha,
            "status": "queued",
            "details_url": record.details_url,
            "external_id": record.run_id,
        });
        let url = format!(
            "https://api.github.com/repos/{}/check-runs",
            self.authority.repo_full_name
        );
        let response = ureq::post(&url)
            .set("Authorization", &format!("Bearer {token}"))
            .set("Accept", "application/vnd.github+json")
            .set("User-Agent", "gunbc-owned-ci")
            .send_json(body)
            .map_err(|e| format!("create check-run: {e}"))?;
        let status = response.status();
        let text = response
            .into_string()
            .map_err(|e| format!("create check-run body: {e}"))?;
        if status != 201 {
            return Err(format!("create check-run HTTP {status}: {text}"));
        }
        let json: serde_json::Value =
            serde_json::from_str(&text).map_err(|e| format!("parse check-run: {e}"))?;
        let id = json["id"]
            .as_u64()
            .ok_or_else(|| "check-run missing id".to_string())?;
        let mut updated = record.clone();
        updated.publication_state = PublicationState {
            kind: "queued".to_string(),
            check_run_id: Some(id),
            local_conclusion: None,
            details_url: Some(record.details_url.clone()),
            cause: None,
        };
        self.write_run(&updated)?;
        Ok(())
    }

    fn try_update_check(
        &self,
        record: &RunRecord,
        status: &str,
        conclusion: Option<&str>,
    ) -> Result<(), String> {
        let check_run_id = record
            .publication_state
            .check_run_id
            .ok_or_else(|| "no check_run_id to update".to_string())?;
        let token =
            std::env::var("GITHUB_TOKEN").map_err(|_| "GITHUB_TOKEN missing".to_string())?;
        let mut body = serde_json::json!({
            "status": status,
            "details_url": record.details_url,
        });
        if let Some(c) = conclusion {
            body["conclusion"] = serde_json::Value::String(c.to_string());
            body["output"] = serde_json::json!({
                "title": format!("owned-ci {c}"),
                "summary": format!("run {} subject {}", record.run_id, record.subject_key),
            });
        }
        let url = format!(
            "https://api.github.com/repos/{}/check-runs/{check_run_id}",
            self.authority.repo_full_name
        );
        let response = ureq::patch(&url)
            .set("Authorization", &format!("Bearer {token}"))
            .set("Accept", "application/vnd.github+json")
            .set("User-Agent", "gunbc-owned-ci")
            .send_json(body)
            .map_err(|e| format!("update check-run: {e}"))?;
        let code = response.status();
        let text = response
            .into_string()
            .map_err(|e| format!("update check-run body: {e}"))?;
        if code != 200 {
            return Err(format!("update check-run HTTP {code}: {text}"));
        }
        let mut updated = record.clone();
        updated.publication_state.kind = if status == "completed" {
            "completed".to_string()
        } else {
            "in_progress".to_string()
        };
        if let Some(c) = conclusion {
            updated.publication_state.local_conclusion = Some(c.to_string());
        }
        self.write_run(&updated)?;
        Ok(())
    }

    fn read_ledger(&self) -> Result<SubjectLedger, String> {
        read_json(self.state_root().join("subject-ledger.json"))
    }

    fn write_ledger(&self, ledger: &SubjectLedger) -> Result<(), String> {
        write_json_atomic(self.state_root().join("subject-ledger.json"), ledger)
    }

    fn read_index(&self) -> Result<RunIndex, String> {
        read_json(self.state_root().join("index.json"))
    }

    fn write_index(&self, index: &RunIndex) -> Result<(), String> {
        write_json_atomic(self.state_root().join("index.json"), index)
    }

    fn read_run(&self, run_id: &str) -> Result<RunRecord, String> {
        validate_run_id(run_id)?;
        read_json(self.run_path(run_id)?)
    }

    fn write_run(&self, record: &RunRecord) -> Result<(), String> {
        validate_run_id(&record.run_id)?;
        let path = self.run_path(&record.run_id)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("mkdir run dir: {e}"))?;
        }
        write_json_atomic(path, record)
    }

    fn run_path(&self, run_id: &str) -> Result<PathBuf, String> {
        validate_run_id(run_id)?;
        Ok(self.state_root().join("runs").join(run_id).join("run.json"))
    }

    fn enqueue_pending(&self, run_id: &str, record: &RunRecord) -> Result<(), String> {
        validate_run_id(run_id)?;
        let path = self
            .state_root()
            .join("queue/pending")
            .join(format!("{run_id}.json"));
        write_json_atomic(path, record)
    }

    fn move_queue_file(&self, run_id: &str, from: &str, to: &str) -> Result<(), String> {
        validate_run_id(run_id)?;
        let from_path = self
            .state_root()
            .join("queue")
            .join(from)
            .join(format!("{run_id}.json"));
        let to_path = self
            .state_root()
            .join("queue")
            .join(to)
            .join(format!("{run_id}.json"));
        if from_path.exists() {
            fs::rename(from_path, to_path).map_err(|e| format!("queue move {run_id}: {e}"))?;
        } else {
            return Err(format!(
                "queue move refused: {run_id} missing from queue/{from} (fail-closed, not silent no-op)"
            ));
        }
        Ok(())
    }

    fn update_index_row(
        &self,
        run_id: &str,
        status: &str,
        conclusion: Option<&str>,
    ) -> Result<(), String> {
        let mut index = self.read_index()?;
        let now = now_unix()?;
        let mut found = false;
        for row in &mut index.runs {
            if row.run_id == run_id {
                row.status = status.to_string();
                row.conclusion = conclusion.map(str::to_string);
                row.updated_at_unix = now;
                found = true;
                break;
            }
        }
        if !found {
            return Err(format!(
                "index update refused: run_id {run_id} absent from index.json (fail-closed, not silent no-op)"
            ));
        }
        self.write_index(&index)
    }

    fn append_stage_receipt(
        &self,
        run_id: &str,
        stage: &str,
        record: &RunRecord,
    ) -> Result<(), String> {
        validate_run_id(run_id)?;
        let path = self
            .state_root()
            .join("receipts")
            .join(run_id)
            .join(format!("{stage}.json"));
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("mkdir receipt dir: {e}"))?;
        }
        write_json_atomic(path, record)
    }

    fn write_run_summary_receipt(&self, run_id: &str, record: &RunRecord) -> Result<(), String> {
        let summary = format!(
            "run_id={}\nhead_sha={}\nstatus={}\npublication={}\n",
            record.run_id, record.head_sha, record.queue_state, record.publication_state.kind,
        );
        let path = self
            .config
            .workspace_root
            .join("target/owned-ci-run-receipts")
            .join(format!("{run_id}.txt"));
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("mkdir target: {e}"))?;
        }
        fs::write(&path, summary).map_err(|e| format!("write summary receipt: {e}"))
    }
}

fn default_stages(labels: &[String]) -> Vec<StageRecord> {
    labels
        .iter()
        .map(|stage| StageRecord {
            stage: stage.clone(),
            status: "pending".to_string(),
            detail: None,
            started_at_unix: None,
            finished_at_unix: None,
        })
        .collect()
}

fn set_stage_running(record: &mut RunRecord, stage: &str, now: u64) {
    for s in &mut record.stages {
        if s.stage == stage {
            s.status = "running".to_string();
            s.started_at_unix = Some(now);
        }
    }
    record.updated_at_unix = now;
}

fn set_stage_succeeded(record: &mut RunRecord, stage: &str, now: u64) {
    for s in &mut record.stages {
        if s.stage == stage {
            s.status = "succeeded".to_string();
            s.finished_at_unix = Some(now);
        }
    }
    record.updated_at_unix = now;
}

fn set_stage_failed(record: &mut RunRecord, stage: &str, detail: &str, now: u64) {
    for s in &mut record.stages {
        if s.stage == stage {
            s.status = "failed".to_string();
            s.detail = Some(detail.to_string());
            s.finished_at_unix = Some(now);
        }
    }
    record.updated_at_unix = now;
}

fn new_run_id(head_sha: &str) -> Result<String, String> {
    let prefix = head_sha.chars().take(8).collect::<String>();
    Ok(format!("{}-{}", now_unix()?, prefix))
}

fn now_unix() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .map_err(|e| format!("system clock before UNIX epoch: {e}"))
}

fn hostname() -> String {
    std::env::var("HOSTNAME").unwrap_or_else(|_| "local".to_string())
}

fn parse_github_link_rel_next(link_header: &str) -> Option<String> {
    for part in link_header.split(',') {
        let part = part.trim();
        if !part.contains("rel=\"next\"") && !part.contains("rel=next") {
            continue;
        }
        let start = part.find('<')?;
        let end = part.find('>')?;
        if end <= start {
            continue;
        }
        return Some(part[start + 1..end].to_string());
    }
    None
}

fn rev_parse(repo: &Path, reference: &str) -> Result<String, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["rev-parse", reference])
        .output()
        .map_err(|e| format!("git rev-parse spawn: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "git rev-parse {reference}: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn run_cmd(cwd: &Path, cmd: &mut Command) -> Result<(), String> {
    let status = cmd
        .current_dir(cwd)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| format!("spawn {}: {e}", cmd.get_program().to_string_lossy()))?;
    status_ok(status, &format!("{}", cmd.get_program().to_string_lossy()))
}

fn status_ok(status: ExitStatus, label: &str) -> Result<(), String> {
    if status.success() {
        Ok(())
    } else {
        Err(format!("{label} exited with {status}"))
    }
}

fn validate_commit_sha(sha: &str) -> Result<(), String> {
    if sha.len() != 40 || !sha.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(format!(
            "commit sha refused (expected 40 hex digits): {:?}",
            sha
        ));
    }
    Ok(())
}

fn validate_run_id(run_id: &str) -> Result<(), String> {
    if run_id.is_empty()
        || run_id == "."
        || run_id == ".."
        || run_id.contains('/')
        || run_id.contains('\\')
        || run_id.contains('\n')
        || run_id.contains('\r')
        || run_id.contains('\0')
    {
        return Err(format!(
            "run_id refused (unsafe path segment): {:?}",
            run_id
        ));
    }
    let Some((unix, hex)) = run_id.split_once('-') else {
        return Err(format!(
            "run_id refused (expected {{unix}}-{{8hex}}): {:?}",
            run_id
        ));
    };
    if unix.is_empty() || !unix.chars().all(|c| c.is_ascii_digit()) {
        return Err(format!(
            "run_id refused (unix prefix must be decimal digits): {:?}",
            run_id
        ));
    }
    if hex.len() != 8 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(format!(
            "run_id refused (suffix must be exactly 8 hex digits): {:?}",
            run_id
        ));
    }
    if hex.contains('-') {
        return Err(format!(
            "run_id refused (multiple hyphen segments): {:?}",
            run_id
        ));
    }
    Ok(())
}

fn read_json<T: for<'de> Deserialize<'de>>(path: PathBuf) -> Result<T, String> {
    let text = fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("parse {}: {e}", path.display()))
}

fn write_json_atomic<T: Serialize>(path: PathBuf, value: &T) -> Result<(), String> {
    let text = serde_json::to_string_pretty(value).map_err(|e| format!("serialize json: {e}"))?;
    let tmp = path.with_extension(format!("tmp-{}", std::process::id()));
    {
        let mut file =
            fs::File::create(&tmp).map_err(|e| format!("create {}: {e}", tmp.display()))?;
        file.write_all(text.as_bytes())
            .map_err(|e| format!("write {}: {e}", tmp.display()))?;
        file.sync_all()
            .map_err(|e| format!("sync {}: {e}", tmp.display()))?;
    }
    fs::rename(&tmp, &path).map_err(|e| format!("rename {}: {e}", path.display()))?;
    if let Some(parent) = path.parent() {
        if let Ok(dir) = fs::File::open(parent) {
            let _ = dir.sync_all();
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_run_id_includes_sha_prefix() {
        let id = new_run_id("deadbeef01234567").expect("run id");
        assert!(id.contains("deadbeef"));
    }

    #[test]
    fn parse_github_link_rel_next_extracts_next_url() {
        let link = "<https://api.github.com/repos/gunb-ai/gunbc/pulls?state=open&per_page=100&page=2>; rel=\"next\", <https://api.github.com/repos/gunb-ai/gunbc/pulls?state=open&per_page=100&page=3>; rel=\"last\"";
        let next = parse_github_link_rel_next(link).expect("next url");
        assert_eq!(
            next,
            "https://api.github.com/repos/gunb-ai/gunbc/pulls?state=open&per_page=100&page=2"
        );
    }

    #[test]
    fn parse_github_link_rel_next_none_when_absent() {
        assert!(parse_github_link_rel_next("").is_none());
    }

    #[test]
    fn validate_commit_sha_accepts_git_oid() {
        assert!(validate_commit_sha("4b825dc642cb6eb9a060e54bf8d69288fbee4904").is_ok());
    }

    #[test]
    fn validate_commit_sha_refuses_short_hex() {
        assert!(validate_commit_sha("deadbeef").is_err());
    }

    #[test]
    fn validate_run_id_refuses_traversal() {
        assert!(validate_run_id("../etc/passwd").is_err());
        assert!(validate_run_id("a/b").is_err());
    }

    #[test]
    fn validate_run_id_accepts_produced_shape() {
        assert!(validate_run_id("1730000000-deadbeef").is_ok());
    }

    #[test]
    fn extend_open_pull_subjects_refuses_malformed_payload() {
        let plane = CiControlPlane {
            config: Config {
                workspace_root: std::env::temp_dir(),
                mirror_path: std::env::temp_dir(),
                serve_base_url: "http://127.0.0.1:8787".to_string(),
                poll_interval_secs: 60,
                lease_secs: 3600,
                lease_holder: "test".to_string(),
                once: true,
                dry_run: true,
                enqueue_rerun_sha: None,
            },
            authority: SeedAuthority {
                repo_owner: "gunb-ai".to_string(),
                repo_name: "gunbc".to_string(),
                repo_full_name: "gunb-ai/gunbc".to_string(),
                check_name: "gunbc-ci".to_string(),
                state_root_rel: ".gunbc/owned-ci".to_string(),
                default_lease_seconds: 3600,
                stage_labels: vec!["build".to_string()],
                floor_plan_entry: "src/v2/workflow/ci_floor_plan.dag".to_string(),
                floor_plan_function: "gunbc_ci_floor_plan".to_string(),
                poll_missing_github_token_refusal: "missing".to_string(),
                poll_empty_github_token_refusal: "empty".to_string(),
                merge_target_branch: "main".to_string(),
            },
        };
        let mut admitted = Vec::new();
        let mut fork_refusals = Vec::new();
        let mut non_main_base_refusals = Vec::new();
        let pulls = vec![serde_json::json!({ "head": { "sha": "abc" } })];
        let err = plane
            .extend_open_pull_subjects(
                &pulls,
                &mut admitted,
                &mut fork_refusals,
                &mut non_main_base_refusals,
            )
            .expect_err("malformed pull");
        assert!(err.contains("missing number or head.sha"));
        assert!(admitted.is_empty());
    }

    #[test]
    fn extend_open_pull_subjects_refuses_missing_base_ref() {
        let plane = CiControlPlane {
            config: Config {
                workspace_root: std::env::temp_dir(),
                mirror_path: std::env::temp_dir(),
                serve_base_url: "http://127.0.0.1:8787".to_string(),
                poll_interval_secs: 60,
                lease_secs: 3600,
                lease_holder: "test".to_string(),
                once: true,
                dry_run: true,
                enqueue_rerun_sha: None,
            },
            authority: SeedAuthority {
                repo_owner: "gunb-ai".to_string(),
                repo_name: "gunbc".to_string(),
                repo_full_name: "gunb-ai/gunbc".to_string(),
                check_name: "gunbc-ci".to_string(),
                state_root_rel: ".gunbc/owned-ci".to_string(),
                default_lease_seconds: 3600,
                stage_labels: vec!["build".to_string()],
                floor_plan_entry: "src/v2/workflow/ci_floor_plan.dag".to_string(),
                floor_plan_function: "gunbc_ci_floor_plan".to_string(),
                poll_missing_github_token_refusal: "missing".to_string(),
                poll_empty_github_token_refusal: "empty".to_string(),
                merge_target_branch: "main".to_string(),
            },
        };
        let mut admitted = Vec::new();
        let mut fork_refusals = Vec::new();
        let mut non_main_base_refusals = Vec::new();
        let pulls = vec![serde_json::json!({
            "number": 1,
            "head": { "sha": "abc", "repo": { "full_name": "gunb-ai/gunbc" } }
        })];
        let err = plane
            .extend_open_pull_subjects(
                &pulls,
                &mut admitted,
                &mut fork_refusals,
                &mut non_main_base_refusals,
            )
            .expect_err("missing base.ref");
        assert!(err.contains("missing base.ref"));
        assert!(admitted.is_empty());
    }

    #[test]
    fn extend_open_pull_subjects_refuses_missing_head_repo_full_name() {
        let plane = CiControlPlane {
            config: Config {
                workspace_root: std::env::temp_dir(),
                mirror_path: std::env::temp_dir(),
                serve_base_url: "http://127.0.0.1:8787".to_string(),
                poll_interval_secs: 60,
                lease_secs: 3600,
                lease_holder: "test".to_string(),
                once: true,
                dry_run: true,
                enqueue_rerun_sha: None,
            },
            authority: SeedAuthority {
                repo_owner: "gunb-ai".to_string(),
                repo_name: "gunbc".to_string(),
                repo_full_name: "gunb-ai/gunbc".to_string(),
                check_name: "gunbc-ci".to_string(),
                state_root_rel: ".gunbc/owned-ci".to_string(),
                default_lease_seconds: 3600,
                stage_labels: vec!["build".to_string()],
                floor_plan_entry: "src/v2/workflow/ci_floor_plan.dag".to_string(),
                floor_plan_function: "gunbc_ci_floor_plan".to_string(),
                poll_missing_github_token_refusal: "missing".to_string(),
                poll_empty_github_token_refusal: "empty".to_string(),
                merge_target_branch: "main".to_string(),
            },
        };
        let mut admitted = Vec::new();
        let mut fork_refusals = Vec::new();
        let mut non_main_base_refusals = Vec::new();
        let pulls = vec![serde_json::json!({
            "number": 1,
            "head": { "sha": "abc" },
            "base": { "ref": "main" }
        })];
        let err = plane
            .extend_open_pull_subjects(
                &pulls,
                &mut admitted,
                &mut fork_refusals,
                &mut non_main_base_refusals,
            )
            .expect_err("missing head.repo.full_name");
        assert!(err.contains("head.repo.full_name"));
        assert!(admitted.is_empty());
        assert!(fork_refusals.is_empty());
    }

    #[test]
    fn extend_open_pull_subjects_records_non_main_base_refusal() {
        let plane = CiControlPlane {
            config: Config {
                workspace_root: std::env::temp_dir(),
                mirror_path: std::env::temp_dir(),
                serve_base_url: "http://127.0.0.1:8787".to_string(),
                poll_interval_secs: 60,
                lease_secs: 3600,
                lease_holder: "test".to_string(),
                once: true,
                dry_run: true,
                enqueue_rerun_sha: None,
            },
            authority: SeedAuthority {
                repo_owner: "gunb-ai".to_string(),
                repo_name: "gunbc".to_string(),
                repo_full_name: "gunb-ai/gunbc".to_string(),
                check_name: "gunbc-ci".to_string(),
                state_root_rel: ".gunbc/owned-ci".to_string(),
                default_lease_seconds: 3600,
                stage_labels: vec!["build".to_string()],
                floor_plan_entry: "src/v2/workflow/ci_floor_plan.dag".to_string(),
                floor_plan_function: "gunbc_ci_floor_plan".to_string(),
                poll_missing_github_token_refusal: "missing".to_string(),
                poll_empty_github_token_refusal: "empty".to_string(),
                merge_target_branch: "main".to_string(),
            },
        };
        let mut admitted = Vec::new();
        let mut fork_refusals = Vec::new();
        let mut non_main_base_refusals = Vec::new();
        let pulls = vec![serde_json::json!({
            "number": 42,
            "head": { "sha": "deadbeef", "repo": { "full_name": "gunb-ai/gunbc" } },
            "base": { "ref": "develop" }
        })];
        plane
            .extend_open_pull_subjects(
                &pulls,
                &mut admitted,
                &mut fork_refusals,
                &mut non_main_base_refusals,
            )
            .expect("extend subjects");
        assert!(admitted.is_empty());
        assert!(fork_refusals.is_empty());
        assert_eq!(non_main_base_refusals.len(), 1);
        assert_eq!(non_main_base_refusals[0].pr_number, 42);
        assert_eq!(non_main_base_refusals[0].base_ref, "develop");
    }

    #[test]
    fn update_index_row_refuses_missing_run_id() {
        let workspace_root =
            std::env::temp_dir().join(format!("owned-ci-index-test-{}", std::process::id()));
        let plane = CiControlPlane {
            config: Config {
                workspace_root: workspace_root.clone(),
                mirror_path: workspace_root.clone(),
                serve_base_url: "http://127.0.0.1:8787".to_string(),
                poll_interval_secs: 60,
                lease_secs: 3600,
                lease_holder: "test".to_string(),
                once: true,
                dry_run: true,
                enqueue_rerun_sha: None,
            },
            authority: SeedAuthority {
                repo_owner: "gunb-ai".to_string(),
                repo_name: "gunbc".to_string(),
                repo_full_name: "gunb-ai/gunbc".to_string(),
                check_name: "gunbc-ci".to_string(),
                state_root_rel: ".gunbc/owned-ci".to_string(),
                default_lease_seconds: 3600,
                stage_labels: vec!["build".to_string()],
                floor_plan_entry: "src/v2/workflow/ci_floor_plan.dag".to_string(),
                floor_plan_function: "gunbc_ci_floor_plan".to_string(),
                poll_missing_github_token_refusal: "missing".to_string(),
                poll_empty_github_token_refusal: "empty".to_string(),
                merge_target_branch: "main".to_string(),
            },
        };
        plane.ensure_layout().expect("layout");
        plane
            .write_index(&RunIndex { runs: vec![] })
            .expect("write empty index");
        let err = plane
            .update_index_row("1730000000-deadbeef", "completed", Some("success"))
            .expect_err("missing index row");
        assert!(err.contains("absent from index.json"));
    }

    #[test]
    fn move_queue_file_refuses_missing_source() {
        let workspace_root =
            std::env::temp_dir().join(format!("owned-ci-queue-test-{}", std::process::id()));
        let plane = CiControlPlane {
            config: Config {
                workspace_root: workspace_root.clone(),
                mirror_path: workspace_root.clone(),
                serve_base_url: "http://127.0.0.1:8787".to_string(),
                poll_interval_secs: 60,
                lease_secs: 3600,
                lease_holder: "test".to_string(),
                once: true,
                dry_run: true,
                enqueue_rerun_sha: None,
            },
            authority: SeedAuthority {
                repo_owner: "gunb-ai".to_string(),
                repo_name: "gunbc".to_string(),
                repo_full_name: "gunb-ai/gunbc".to_string(),
                check_name: "gunbc-ci".to_string(),
                state_root_rel: ".gunbc/owned-ci".to_string(),
                default_lease_seconds: 3600,
                stage_labels: vec!["build".to_string()],
                floor_plan_entry: "src/v2/workflow/ci_floor_plan.dag".to_string(),
                floor_plan_function: "gunbc_ci_floor_plan".to_string(),
                poll_missing_github_token_refusal: "missing".to_string(),
                poll_empty_github_token_refusal: "empty".to_string(),
                merge_target_branch: "main".to_string(),
            },
        };
        plane.ensure_layout().expect("layout");
        let err = plane
            .move_queue_file("1730000000-deadbeef", "claimed", "completed")
            .expect_err("missing queue file");
        assert!(err.contains("missing from queue/claimed"));
    }
}
