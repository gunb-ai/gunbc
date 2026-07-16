// Seed realization for v2.compiler.materialization_carriers (Wave 2 Band A).
// Hand-retained Rust oracle — independent of the self-emitted artifact under test.
// Dissolve-on: self-emit cutover retires this module when v2.compiler.materialization_carriers
// is emitted-only and the behavioral harness is modeled (smc_scaffold_dissolution_trigger).

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum Materialization {
    Memoize,
    Recompute,
    Share,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum FrameKind {
    SharedStateFrame,
    IsolatedChildrenFrame,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Frame {
    pub name: String,
    pub kind: FrameKind,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FrameDemand {
    pub identity: String,
    pub site: Vec<Frame>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CacheProvider {
    pub id: String,
    pub scope: Vec<Frame>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum LadderVerdict {
    AcceptedSingleRecompute { identity: String },
    RefusedScopeTooNarrow { provider_id: String },
    RefusedNoProvider { identity: String },
}

pub fn materialization_allows_memo_store(m: Materialization) -> bool {
    match m {
        Materialization::Memoize => true,
        Materialization::Recompute | Materialization::Share => false,
    }
}

fn v2_compile_process_frame() -> Frame {
    Frame {
        name: "v2-compile-ingest".to_string(),
        kind: FrameKind::SharedStateFrame,
    }
}

fn v2_compile_process_site() -> Vec<Frame> {
    vec![v2_compile_process_frame()]
}

fn v2_compile_workflow_frame() -> Frame {
    Frame {
        name: "v2-compile-workflow".to_string(),
        kind: FrameKind::IsolatedChildrenFrame,
    }
}

fn v2_compile_run_frame(name: &str) -> Frame {
    Frame {
        name: name.to_string(),
        kind: FrameKind::SharedStateFrame,
    }
}

fn parse_table_memo_demand() -> FrameDemand {
    FrameDemand {
        identity: "parse-nonterminal-memo".to_string(),
        site: v2_compile_process_site(),
    }
}

fn compile_stage_memo_demand() -> FrameDemand {
    FrameDemand {
        identity: "compile-pipeline-stage".to_string(),
        site: v2_compile_process_site(),
    }
}

fn parse_table_memo_provider() -> CacheProvider {
    CacheProvider {
        id: "parse_table_memo_cache".to_string(),
        scope: v2_compile_process_site(),
    }
}

fn compile_stage_memo_provider() -> CacheProvider {
    CacheProvider {
        id: "compile_stage_memo_cache".to_string(),
        scope: v2_compile_process_site(),
    }
}

fn frame_path_lca(a: &[Frame], b: &[Frame]) -> Vec<Frame> {
    let mut lca = Vec::new();
    for (fa, fb) in a.iter().zip(b.iter()) {
        if fa == fb {
            lca.push(fa.clone());
        } else {
            break;
        }
    }
    lca
}

fn distinct_identities(demands: &[FrameDemand]) -> Vec<String> {
    let mut out = Vec::new();
    for d in demands {
        if !out.iter().any(|id| id == &d.identity) {
            out.push(d.identity.clone());
        }
    }
    out
}

fn ladder_group<'a>(demands: &'a [FrameDemand], identity: &str) -> Vec<&'a FrameDemand> {
    demands
        .iter()
        .filter(|d| d.identity == identity)
        .collect()
}

fn group_obligation_lca(group: &[FrameDemand]) -> Vec<Frame> {
    if group.is_empty() {
        return Vec::new();
    }
    if group.len() == 1 {
        return group[0].site.clone();
    }
    group[1..]
        .iter()
        .fold(group[0].site.clone(), |acc, d| frame_path_lca(&acc, &d.site))
}

fn group_is_redundant(group: &[&FrameDemand]) -> bool {
    group.len() > 1
}

fn discharge_verdict(
    identity: &str,
    lca: &[Frame],
    providers: &[CacheProvider],
) -> LadderVerdict {
    if providers.iter().any(|p| p.scope == lca) {
        return LadderVerdict::AcceptedSingleRecompute {
            identity: identity.to_string(),
        };
    }
    if providers.is_empty() {
        return LadderVerdict::RefusedNoProvider {
            identity: identity.to_string(),
        };
    }
    LadderVerdict::RefusedScopeTooNarrow {
        provider_id: providers[0].id.clone(),
    }
}

fn group_redundant_verdict(
    identity: &str,
    group: &[&FrameDemand],
    providers: &[CacheProvider],
) -> LadderVerdict {
    let owned: Vec<FrameDemand> = group.iter().map(|d| (*d).clone()).collect();
    let lca = group_obligation_lca(&owned);
    discharge_verdict(identity, &lca, providers)
}

fn group_verdict(
    identity: &str,
    group: &[&FrameDemand],
    providers: &[CacheProvider],
) -> LadderVerdict {
    let owned: Vec<FrameDemand> = group.iter().map(|d| (*d).clone()).collect();
    let lca = group_obligation_lca(&owned);
    if group_is_redundant(group) {
        group_redundant_verdict(identity, group, providers)
    } else {
        discharge_verdict(identity, &lca, providers)
    }
}

fn materialization_ladder_holds(demands: &[FrameDemand], providers: &[CacheProvider]) -> bool {
    materialization_ladder(demands, providers)
        .iter()
        .all(|v| matches!(v, LadderVerdict::AcceptedSingleRecompute { .. }))
}

fn materialization_ladder(
    demands: &[FrameDemand],
    providers: &[CacheProvider],
) -> Vec<LadderVerdict> {
    distinct_identities(demands)
        .iter()
        .map(|identity| {
            let group = ladder_group(demands, identity);
            group_verdict(identity, &group, providers)
        })
        .collect()
}

fn verdict_is_refused_scope_too_narrow(v: &LadderVerdict, provider_id: &str) -> bool {
    matches!(
        v,
        LadderVerdict::RefusedScopeTooNarrow { provider_id: id } if id == provider_id
    )
}

pub fn v2_compiler_materialization_demands() -> Vec<FrameDemand> {
    vec![parse_table_memo_demand(), compile_stage_memo_demand()]
}

pub fn v2_compiler_cache_providers() -> Vec<CacheProvider> {
    vec![parse_table_memo_provider(), compile_stage_memo_provider()]
}

pub fn v2_compiler_materialization_ladder_holds() -> bool {
    materialization_ladder_holds(
        &v2_compiler_materialization_demands(),
        &v2_compiler_cache_providers(),
    )
}

pub fn v2_compiler_catalog_projection_refusals() -> i64 {
    0
}

pub fn parse_table_memo_provider_id() -> String {
    "parse_table_memo_cache".to_string()
}

pub fn compile_stage_memo_provider_id() -> String {
    "compile_stage_memo_cache".to_string()
}

pub fn parse_table_memo_artifact_kind() -> String {
    "parse_table".to_string()
}

pub fn parse_table_memo_materialization() -> Materialization {
    Materialization::Memoize
}

pub fn compile_stage_memo_materialization() -> Materialization {
    Materialization::Memoize
}

pub fn parse_table_carrier_grounded_on_catalog() -> bool {
    parse_table_memo_artifact_kind() == "parse_table"
        && parse_table_memo_provider_id() == "parse_table_memo_cache"
        && parse_table_memo_materialization() == Materialization::Memoize
}

fn parse_table_memo_plural_demands() -> Vec<FrameDemand> {
    vec![
        FrameDemand {
            identity: "parse-nonterminal-memo".to_string(),
            site: vec![
                v2_compile_workflow_frame(),
                v2_compile_run_frame("compile-run-a"),
                v2_compile_process_frame(),
            ],
        },
        FrameDemand {
            identity: "parse-nonterminal-memo".to_string(),
            site: vec![
                v2_compile_workflow_frame(),
                v2_compile_run_frame("compile-run-b"),
                v2_compile_process_frame(),
            ],
        },
    ]
}

pub fn parse_table_memo_plural_holds_with_provider() -> bool {
    materialization_ladder_holds(
        &parse_table_memo_plural_demands(),
        &v2_compiler_cache_providers(),
    )
}

pub fn parse_table_memo_plural_scope_too_narrow_count() -> i64 {
    materialization_ladder(
        &parse_table_memo_plural_demands(),
        &v2_compiler_cache_providers(),
    )
    .iter()
    .filter(|v| verdict_is_refused_scope_too_narrow(v, "parse_table_memo_cache"))
    .count() as i64
}
