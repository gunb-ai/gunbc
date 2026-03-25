use crate::v2_core::*;
use crate::tokenize::*;
use crate::parse::*;
use crate::resolve::*;
use crate::normalize::*;
use crate::infer::*;
use crate::emit::*;
use crate::emit_rust::*;
use crate::emit_python::*;
use crate::emit_go::*;
use crate::complexity::*;
use crate::ownership::*;
use crate::artifact::*;
use crate::v2_rt;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SourceFile {
    pub path: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PipelineResult {
    pub files: Rc<Vec<Rc<TextFile>>>,
    pub diagnostics: Rc<Vec<Rc<Diagnostic>>>,
    pub complexity: Rc<ComplexityReport>,
    pub ownership: Rc<Vec<Rc<OwnershipProof>>>,
    pub artifact_plan: Rc<ArtifactPlan>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FrontendResult {
    pub graph: Option<Rc<ModuleGraph>>,
    pub diagnostics: Rc<Vec<Rc<Diagnostic>>>,
}

pub fn extract_func_entries(typed: Rc<ResolvedGraph>) -> Rc<Vec<Rc<FuncEntry>>> {
    {
    let mut __flat_mapped_0 = Vec::new();
    for __elem_1 in typed.modules.iter().cloned() {
        __flat_mapped_0.extend(({
    let mut __mapped_4 = Vec::new();
    for __elem_5 in ({
    let mut __filtered_2 = Vec::new();
    for __elem_3 in __elem_1.items.iter().cloned() {
        if __elem_3.body.clone().is_some() {
    __filtered_2.push(__elem_3);
};
    }
    Rc::new(__filtered_2)
}).iter().cloned() {
        __mapped_4.push(Rc::new(FuncEntry { name: __elem_5.name.clone(), body: __elem_5.body.clone().unwrap(), params: __elem_5.params.clone() }));
    }
    Rc::new(__mapped_4)
}).iter().cloned());
    }
    Rc::new(__flat_mapped_0)
}
}

pub fn extract_ownership_proofs(typed: Rc<ResolvedGraph>) -> Rc<Vec<Rc<OwnershipProof>>> {
    {
    let mut __flat_mapped_0 = Vec::new();
    for __elem_1 in typed.modules.iter().cloned() {
        __flat_mapped_0.extend(({
    let mut __mapped_4 = Vec::new();
    for __elem_5 in ({
    let mut __filtered_2 = Vec::new();
    for __elem_3 in __elem_1.items.iter().cloned() {
        if __elem_3.body.clone().is_some() {
    __filtered_2.push(__elem_3);
};
    }
    Rc::new(__filtered_2)
}).iter().cloned() {
        __mapped_4.push(analyze_ownership(&__elem_5.name, __elem_5.params.clone(), __elem_5.body.clone().unwrap()));
    }
    Rc::new(__mapped_4)
}).iter().cloned());
    }
    Rc::new(__flat_mapped_0)
}
}

pub fn ownership_diagnostics(proofs: Rc<Vec<Rc<OwnershipProof>>>) -> Rc<Vec<Rc<Diagnostic>>> {
    {
    let mut __flat_mapped_0 = Vec::new();
    for __elem_1 in proofs.iter().cloned() {
        __flat_mapped_0.extend(({
    let mut __flat_mapped_2 = Vec::new();
    for __elem_3 in __elem_1.decisions.iter().cloned() {
        __flat_mapped_2.extend((match __elem_3.as_ref() {
    OwnershipDecision::SharedError { binding, consumer_count: count, sites, .. } => {
        Rc::new(vec!(Rc::new(Diagnostic { message: v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("ownership: binding '".to_string(), binding.clone()), "' in ".to_string()), __elem_1.func_name.clone()), " has ".to_string()), v2_rt::to_string(count.clone())), " consumers -- cannot guarantee O(1) mutation (".to_string()), {
    let mut __joined_4 = String::new();
    let mut __first_6 = true;
    for __elem_5 in sites.iter().cloned() {
        if !__first_6 {
    __joined_4.push_str(&", ".to_string());
};
        __first_6 = false;
        __joined_4.push_str(&__elem_5);
    }
    __joined_4
}), ")".to_string()), severity: Severity::Warning, span: Some(SourceSpan { start: 0_i64, end: 0_i64 }), module_name: Some(__elem_1.func_name.clone()), category: None })))
    }
    _ => {
        Rc::new(Vec::new())
    }
}).iter().cloned());
    }
    Rc::new(__flat_mapped_2)
}).iter().cloned());
    }
    Rc::new(__flat_mapped_0)
}
}

pub fn empty_artifact_plan() -> Rc<ArtifactPlan> {
    Rc::new(ArtifactPlan { artifacts: Rc::new(Vec::new()), boundaries: Rc::new(Vec::new()) })
}

pub fn compile_bundle_error(message: &str) -> Rc<Diagnostic> {
    Rc::new(Diagnostic { severity: Severity::Error, message: message.to_string(), span: None, module_name: None, category: None })
}

pub fn emit_artifact(typed: Rc<ResolvedGraph>, artifact: Rc<Artifact>) -> Rc<EmitResult> {
    match artifact.target.clone() {
    RenderTarget::Rust => {
        emit_rust(typed.clone())
    }
    RenderTarget::Python => {
        emit_python(typed.clone())
    }
    RenderTarget::Go => {
        emit_go(typed.clone())
    }
    RenderTarget::Dag => {
        emit_dag_artifact(typed.clone())
    }
}
}

pub fn emit_dag_artifact(typed: Rc<ResolvedGraph>) -> Rc<EmitResult> {
    let module_names = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in typed.modules.iter().cloned() {
        __mapped_0.push(v2_rt::concat(v2_rt::concat("\"".to_string(), __elem_1.module.name.clone()), "\"".to_string()));
    }
    Rc::new(__mapped_0)
};
    let modules_json = {
    let mut __acc_2 = "".to_string();
    for __elem_3 in module_names.iter().cloned() {
        __acc_2 = if __acc_2.clone() == "" {
    __elem_3.clone()
} else {
    v2_rt::concat(v2_rt::concat(__acc_2, ", ".to_string()), __elem_3.clone())
};
    }
    __acc_2
};
    let json = v2_rt::concat(v2_rt::concat("{\n  \"version\": \"0.1.0\",\n  \"modules\": [".to_string(), modules_json.clone()), "],\n  \"files\": []\n}".to_string());
    Rc::new(EmitResult { files: Rc::new(vec!(Rc::new(TextFile { path: "dag-artifact.json".to_string(), content: json.clone() }))), diagnostics: Rc::new(Vec::new()) })
}

pub fn boundary_ref_error(names: Rc<Vec<String>>, ref_name: &str) -> Rc<Vec<Rc<Diagnostic>>> {
    {
let __cond = {
    let mut __any_0 = false;
    for __elem_1 in names.iter().cloned() {
        if __elem_1.clone() == ref_name {
    __any_0 = true;
    break;
};
    }
    __any_0
};
if __cond {
    Rc::new(Vec::new())
} else {
    Rc::new(vec!(compile_bundle_error(&v2_rt::concat(v2_rt::concat("boundary references unknown artifact '".to_string(), ref_name.to_string()), "'".to_string()))))
}
}
}

pub fn validate_boundaries(plan: Rc<ArtifactPlan>) -> Rc<Vec<Rc<Diagnostic>>> {
    let names = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in plan.artifacts.iter().cloned() {
        __mapped_0.push(__elem_1.name.clone());
    }
    Rc::new(__mapped_0)
};
    {
    let mut __flat_mapped_2 = Vec::new();
    for __elem_3 in plan.boundaries.iter().cloned() {
        __flat_mapped_2.extend(v2_rt::concat(boundary_ref_error(names.clone(), &__elem_3.from_artifact), boundary_ref_error(names.clone(), &__elem_3.to_artifact)).iter().cloned());
    }
    Rc::new(__flat_mapped_2)
}
}

pub fn emit_from_artifact_plan(typed: Rc<ResolvedGraph>, artifact_plan: Rc<ArtifactPlan>) -> Rc<EmitResult> {
    if ({
    let __len_0 = artifact_plan.artifacts.clone().len();
    __len_0 as i64
}) == 0_i64 {
    return Rc::new(EmitResult { files: Rc::new(Vec::new()), diagnostics: Rc::new(vec!(compile_bundle_error("compile_sources planned no artifacts"))) });
};
    let boundary_diags = validate_boundaries(artifact_plan.clone());
    if ({
    let __len_1 = boundary_diags.clone().len();
    __len_1 as i64
}) > 0_i64 {
    return Rc::new(EmitResult { files: Rc::new(Vec::new()), diagnostics: boundary_diags.clone() });
};
    let results = {
    let mut __mapped_2 = Vec::new();
    for __elem_3 in artifact_plan.artifacts.iter().cloned() {
        __mapped_2.push(emit_artifact(typed.clone(), __elem_3.clone()));
    }
    Rc::new(__mapped_2)
};
    let all_files = {
    let mut __flat_mapped_4 = Vec::new();
    for __elem_5 in results.iter().cloned() {
        __flat_mapped_4.extend(__elem_5.files.clone().iter().cloned());
    }
    Rc::new(__flat_mapped_4)
};
    let all_diags = {
    let mut __flat_mapped_6 = Vec::new();
    for __elem_7 in results.iter().cloned() {
        __flat_mapped_6.extend(__elem_7.diagnostics.clone().iter().cloned());
    }
    Rc::new(__flat_mapped_6)
};
    Rc::new(EmitResult { files: all_files.clone(), diagnostics: all_diags.clone() })
}

pub fn collect_diagnostics(parse_results: Rc<Vec<Rc<ParseResult>>>) -> Rc<Vec<Rc<Diagnostic>>> {
    {
    let mut __acc_0: Rc<Vec<Rc<Diagnostic>>> = Rc::new(Vec::new());
    for __elem_1 in parse_results.iter().cloned() {
        __acc_0 = match __elem_1.error.as_ref().map(|__rc| __rc.as_ref()) {
    Some(diag) => {
        let diag = Rc::new(diag.clone());
        {
    let __rc_3 = __acc_0;
    let mut __appended_2 = Rc::try_unwrap(__rc_3).unwrap_or_else(|rc| (*rc).clone());
    __appended_2.push(diag.clone());
    Rc::new(__appended_2)
}
    }
    None => {
        __acc_0.clone()
    }
};
    }
    __acc_0
}
}

pub fn front_end_sources(sources: Rc<Vec<Rc<SourceFile>>>) -> Rc<FrontendResult> {
    let tokenized = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in sources.iter().cloned() {
        __mapped_0.push(tokenize(&__elem_1.content));
    }
    Rc::new(__mapped_0)
};
    let parse_results = {
    let mut __mapped_2 = Vec::new();
    for __elem_3 in tokenized.iter().cloned() {
        __mapped_2.push(parse(__elem_3.clone()));
    }
    Rc::new(__mapped_2)
};
    let parse_diagnostics = collect_diagnostics(parse_results.clone());
    let has_parse_errors = {
    let mut __any_4 = false;
    for __elem_5 in parse_results.iter().cloned() {
        if __elem_5.error.clone().is_some() {
    __any_4 = true;
    break;
};
    }
    __any_4
};
    if has_parse_errors.clone() {
    Rc::new(FrontendResult { graph: None, diagnostics: parse_diagnostics.clone() })
} else {
    let modules = {
    let mut __mapped_6 = Vec::new();
    for __elem_7 in parse_results.iter().cloned() {
        __mapped_6.push(__elem_7.module.clone().unwrap());
    }
    Rc::new(__mapped_6)
};
    let graph = resolve_modules(modules.clone());
    Rc::new(FrontendResult { graph: Some(graph.clone()), diagnostics: v2_rt::concat(parse_diagnostics.clone(), graph.diagnostics.clone()) })
}
}

pub fn resolve_sources(sources: Rc<Vec<Rc<SourceFile>>>) -> Rc<CompileResult> {
    let frontend = front_end_sources(sources.clone());
    Rc::new(CompileResult { files: Rc::new(Vec::new()), diagnostics: frontend.diagnostics.clone() })
}

pub fn compile_sources(sources: Rc<Vec<Rc<SourceFile>>>, target: RenderTarget) -> Rc<PipelineResult> {
    let frontend = front_end_sources(sources.clone());
    match frontend.graph.as_ref().map(|__rc| __rc.as_ref()) {
    None => {
        Rc::new(PipelineResult { files: Rc::new(Vec::new()), diagnostics: frontend.diagnostics.clone(), complexity: empty_complexity_report(), ownership: Rc::new(Vec::new()), artifact_plan: empty_artifact_plan() })
    }
    Some(graph) => {
        let graph = Rc::new(graph.clone());
        {
    let graph_diags = graph.diagnostics.clone();
    let resolve_errors = {
    let mut __filtered_0 = Vec::new();
    for __elem_1 in graph_diags.iter().cloned() {
        if __elem_1.severity.clone() == Severity::Error {
    __filtered_0.push(__elem_1);
};
    }
    Rc::new(__filtered_0)
};
    if ({
    let __len_2 = resolve_errors.clone().len();
    __len_2 as i64
}) > 0_i64 {
    return Rc::new(PipelineResult { files: Rc::new(Vec::new()), diagnostics: frontend.diagnostics.clone(), complexity: empty_complexity_report(), ownership: Rc::new(Vec::new()), artifact_plan: empty_artifact_plan() });
};
    let norm = normalize_graph(graph.clone());
    let norm_diags = norm.diagnostics.clone();
    let norm_errors = {
    let mut __filtered_3 = Vec::new();
    for __elem_4 in norm_diags.iter().cloned() {
        if __elem_4.severity.clone() == Severity::Error {
    __filtered_3.push(__elem_4);
};
    }
    Rc::new(__filtered_3)
};
    if ({
    let __len_5 = norm_errors.clone().len();
    __len_5 as i64
}) > 0_i64 {
    return Rc::new(PipelineResult { files: Rc::new(Vec::new()), diagnostics: v2_rt::concat(frontend.diagnostics.clone(), norm_diags.clone()), complexity: empty_complexity_report(), ownership: Rc::new(Vec::new()), artifact_plan: empty_artifact_plan() });
};
    let typed = reconcile(norm.graph.clone());
    let typed_diags = typed.diagnostics.clone();
    let typecheck_errors = {
    let mut __filtered_6 = Vec::new();
    for __elem_7 in typed_diags.iter().cloned() {
        if __elem_7.severity.clone() == Severity::Error {
    __filtered_6.push(__elem_7);
};
    }
    Rc::new(__filtered_6)
};
    if ({
    let __len_8 = typecheck_errors.clone().len();
    __len_8 as i64
}) > 0_i64 {
    return Rc::new(PipelineResult { files: Rc::new(Vec::new()), diagnostics: v2_rt::concat(v2_rt::concat(frontend.diagnostics.clone(), norm_diags.clone()), typed_diags.clone()), complexity: empty_complexity_report(), ownership: Rc::new(Vec::new()), artifact_plan: empty_artifact_plan() });
};
    let func_entries = extract_func_entries(typed.clone());
    let complexity = if ({
    let __len_9 = func_entries.clone().len();
    __len_9 as i64
}) > 100_i64 {
    empty_complexity_report()
} else {
    build_complexity_report(func_entries.clone())
};
    let ownership = extract_ownership_proofs(typed.clone());
    let ownership_diags = ownership_diagnostics(ownership.clone());
    let artifact_plan = default_artifact_plan({
    let mut __mapped_12 = Vec::new();
    for __elem_13 in typed.modules.iter().cloned() {
        __mapped_12.push(__elem_13.module.name.clone());
    }
    Rc::new(__mapped_12)
}, target);
    let emit_result = emit_from_artifact_plan(typed.clone(), artifact_plan.clone());
    let emit_files = emit_result.files.clone();
    let emit_diags = emit_result.diagnostics.clone();
    let emit_errors = {
    let mut __filtered_14 = Vec::new();
    for __elem_15 in emit_diags.iter().cloned() {
        if __elem_15.severity.clone() == Severity::Error {
    __filtered_14.push(__elem_15);
};
    }
    Rc::new(__filtered_14)
};
    let final_files = if ({
    let __len_16 = emit_errors.clone().len();
    __len_16 as i64
}) > 0_i64 {
    Rc::new(Vec::new())
} else {
    emit_files.clone()
};
    Rc::new(PipelineResult { files: final_files.clone(), diagnostics: v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(frontend.diagnostics.clone(), norm_diags.clone()), typed_diags.clone()), ownership_diags.clone()), emit_diags.clone()), complexity: complexity.clone(), ownership: ownership.clone(), artifact_plan: artifact_plan.clone() })
}
    }
}
}

