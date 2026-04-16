use crate::dag::{AtomPayload, Dag, Declaration, DeclarationId, Field, TypeConnective};
use std::collections::HashMap;

const MAX_RENDER_DEPTH: usize = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedClaim {
    pub declaration_name: String,
    pub declaration_source: String,
}

pub struct TestgenLens<'a> {
    dag: &'a Dag,
}

impl<'a> TestgenLens<'a> {
    pub fn new(dag: &'a Dag) -> Self {
        Self { dag }
    }

    pub fn query(&self) -> Vec<GeneratedClaim> {
        let mut claims = Vec::new();
        let mut next_claim_id = 0usize;
        for decl_id in self.named_std_type_ids() {
            let decl = self.dag.declaration(decl_id);
            let subst = self.default_subst(decl);
            let Some(type_expr) = self.render_type_expr(decl_id, &subst) else {
                continue;
            };
            match &decl.connective {
                TypeConnective::Conj { children } => {
                    let compile_value = self.render_value_expr(decl_id, &subst, 0);
                    if let Some(value_expr) = &compile_value {
                        self.push_claim(
                            &mut claims,
                            &mut next_claim_id,
                            format!("{type_expr} compiles"),
                            format!("let witness: {type_expr} = {value_expr}\n"),
                            format!("{}_compiles.v3", sanitize(&type_expr)),
                            "Compiles".to_string(),
                        );
                    }
                    let missing_field = self.missing_field_name(children);
                    self.push_claim(
                        &mut claims,
                        &mut next_claim_id,
                        format!("{type_expr} rejects missing field"),
                        format!("fn probe(value: {type_expr}) -> Int = value.{missing_field}\n"),
                        format!("{}_missing_field.v3", sanitize(&type_expr)),
                        format!("FailsWithDiagnostic({})", quote_string(&format!(
                            "field `{missing_field}` does not exist"
                        ))),
                    );
                    if compile_value.is_some()
                        && decl.span.file == "src/v3/std/verification.dag"
                        && self.supports_type_mismatch_claim(children)
                    {
                        self.push_claim(
                            &mut claims,
                            &mut next_claim_id,
                            format!("{type_expr} rejects field type mismatch"),
                            format!(
                                "let witness: {type_expr} = {}\n",
                                self.render_mismatch_record(children, &subst)
                            ),
                            format!("{}_type_mismatch.v3", sanitize(&type_expr)),
                            format!("FailsWithDiagnostic({})", quote_string("TypeMismatch")),
                        );
                    }
                }
                TypeConnective::Disj { variants } => {
                    for variant in variants {
                        if let Some(value_expr) =
                            self.render_variant_witness(decl_id, variant, &subst, 0)
                        {
                            self.push_claim(
                                &mut claims,
                                &mut next_claim_id,
                                format!("{type_expr} variant {} compiles", variant.label),
                                format!("let witness: {type_expr} = {value_expr}\n"),
                                format!(
                                    "{}_{}_compiles.v3",
                                    sanitize(&type_expr),
                                    sanitize(&variant.label)
                                ),
                                "Compiles".to_string(),
                            );
                        }
                    }
                    if variants.len() > 1 {
                        self.push_claim(
                            &mut claims,
                            &mut next_claim_id,
                            format!("{type_expr} requires exhaustive match"),
                            format!(
                                "fn probe(value: {type_expr}) -> Int = match value {{ {} => 0 }}\n",
                                self.match_pattern(&variants[0])
                            ),
                            format!("{}_non_exhaustive.v3", sanitize(&type_expr)),
                            format!("FailsWithDiagnostic({})", quote_string("non-exhaustive")),
                        );
                    }
                }
                _ => {}
            }
        }
        claims
    }

    fn push_claim(
        &self,
        claims: &mut Vec<GeneratedClaim>,
        next_claim_id: &mut usize,
        claim_name: String,
        program_source: String,
        file_name: String,
        predicate_expr: String,
    ) {
        let declaration_name = format!("generated_test_claim_{:03}", *next_claim_id);
        *next_claim_id += 1;
        claims.push(GeneratedClaim {
            declaration_name: declaration_name.clone(),
            declaration_source: format!(
                "data {declaration_name}: TestClaim = {{\n  name: {},\n  source: {},\n  file_name: {},\n  predicate: {}\n}}\n",
                quote_string(&claim_name),
                quote_string(&program_source),
                quote_string(&file_name),
                predicate_expr
            ),
        });
    }

    fn named_std_type_ids(&self) -> Vec<DeclarationId> {
        let mut chosen: HashMap<String, DeclarationId> = HashMap::new();
        for decl in self.dag.declarations() {
            let Some(name) = &decl.name else {
                continue;
            };
            if decl.value_body.is_some() || !is_std_file(&decl.span.file) {
                continue;
            }
            match chosen.get(name).copied() {
                None => {
                    chosen.insert(name.clone(), decl.id);
                }
                Some(existing) => {
                    let new_rank = std_preference_rank(&decl.span.file);
                    let old_rank = std_preference_rank(&self.dag.declaration(existing).span.file);
                    if new_rank > old_rank {
                        chosen.insert(name.clone(), decl.id);
                    }
                }
            }
        }
        let mut ids: Vec<_> = chosen.into_values().collect();
        ids.sort_by_key(|id| self.dag.declaration(*id).name.clone());
        ids
    }

    fn default_subst(&self, decl: &Declaration) -> HashMap<DeclarationId, DeclarationId> {
        let int_id = self
            .dag
            .declaration_by_name("Int")
            .expect("bootstrap should load Int")
            .id;
        decl.type_params
            .iter()
            .map(|param| (*param, int_id))
            .collect()
    }

    fn render_type_expr(
        &self,
        decl_id: DeclarationId,
        subst: &HashMap<DeclarationId, DeclarationId>,
    ) -> Option<String> {
        let decl = self.dag.declaration(decl_id);
        if let Some(name) = &decl.name {
            if !matches!(
                decl.connective,
                TypeConnective::Atom(AtomPayload::TypeParam(_))
            ) {
                if decl.type_params.is_empty() {
                    return Some(name.clone());
                }
                let args: Option<Vec<_>> = decl
                    .type_params
                    .iter()
                    .map(|param| self.render_type_expr(*subst.get(param)?, subst))
                    .collect();
                return Some(format!("{name}<{}>", args?.join(", ")));
            }
        }
        match &decl.connective {
            TypeConnective::Atom(AtomPayload::TypeParam(_)) => {
                self.render_type_expr(*subst.get(&decl_id)?, subst)
            }
            TypeConnective::Instantiation {
                template,
                arguments,
            } => {
                let template_decl = self.dag.declaration(*template);
                let template_name = template_decl.name.as_ref()?;
                let args: Option<Vec<_>> = arguments
                    .iter()
                    .map(|arg| self.render_type_expr(arg.value, subst))
                    .collect();
                Some(format!("{template_name}<{}>", args?.join(", ")))
            }
            _ => None,
        }
    }

    fn render_value_expr(
        &self,
        decl_id: DeclarationId,
        subst: &HashMap<DeclarationId, DeclarationId>,
        depth: usize,
    ) -> Option<String> {
        if depth > MAX_RENDER_DEPTH {
            return None;
        }
        let decl = self.dag.declaration(decl_id);
        if self
            .render_type_expr(decl_id, subst)
            .is_some_and(|ty| ty.starts_with("List<"))
            && depth > 0
        {
            return Some("empty()".to_string());
        }
        match decl.name.as_deref() {
            Some("Int") => Some("1".to_string()),
            Some("Bool") => Some("true".to_string()),
            Some("String") => Some("\"x\"".to_string()),
            Some(_) if !is_std_file(&decl.span.file) => None,
            _ => match &decl.connective {
                TypeConnective::Atom(AtomPayload::TypeParam(_)) => {
                    self.render_value_expr(*subst.get(&decl_id)?, subst, depth + 1)
                }
                TypeConnective::Instantiation {
                    template,
                    arguments,
                } => {
                    let mut specialized = subst.clone();
                    for arg in arguments {
                        specialized.insert(arg.parameter, arg.value);
                    }
                    self.render_value_expr(*template, &specialized, depth + 1)
                }
                TypeConnective::Conj { children } => {
                    let fields: Option<Vec<_>> = children
                        .iter()
                        .map(|field| {
                            Some(format!(
                                "{}: {}",
                                field.label,
                                self.render_value_expr(field.ty, subst, depth + 1)?
                            ))
                        })
                        .collect();
                    Some(format!("{{ {} }}", fields?.join(", ")))
                }
                TypeConnective::Disj { variants } => variants
                    .iter()
                    .find_map(|variant| self.render_variant_expr(variant, subst, depth + 1)),
                _ => None,
            },
        }
    }

    fn render_variant_expr(
        &self,
        variant: &Field,
        subst: &HashMap<DeclarationId, DeclarationId>,
        depth: usize,
    ) -> Option<String> {
        let TypeConnective::Conj { children } = &self.dag.declaration(variant.ty).connective else {
            return None;
        };
        if children.is_empty() {
            return Some(variant.label.clone());
        }
        let payloads: Option<Vec<_>> = children
            .iter()
            .map(|field| self.render_value_expr(field.ty, subst, depth + 1))
            .collect();
        Some(format!("{}({})", variant.label, payloads?.join(", ")))
    }

    fn render_variant_witness(
        &self,
        sum_decl_id: DeclarationId,
        variant: &Field,
        subst: &HashMap<DeclarationId, DeclarationId>,
        depth: usize,
    ) -> Option<String> {
        if self.dag.declaration(sum_decl_id).name.as_deref() == Some("List") {
            return match variant.label.as_str() {
                "Empty" => Some("[]".to_string()),
                "Cons" => {
                    let TypeConnective::Conj { children } =
                        &self.dag.declaration(variant.ty).connective
                    else {
                        return None;
                    };
                    let head = self.render_value_expr(children.first()?.ty, subst, depth + 1)?;
                    Some(format!("[{head}]"))
                }
                _ => None,
            };
        }
        self.render_variant_expr(variant, subst, depth)
    }

    fn render_mismatch_record(
        &self,
        children: &[Field],
        subst: &HashMap<DeclarationId, DeclarationId>,
    ) -> String {
        let mismatch_index = 0usize;
        format!(
            "{{ {} }}",
            children
                .iter()
                .enumerate()
                .map(|(idx, field)| {
                    let value = if idx == mismatch_index {
                        self.render_wrong_value_expr(field.ty, subst)
                    } else {
                        self.render_value_expr(field.ty, subst, 1)
                            .expect("type mismatch claims should reuse a valid witness for non-targeted fields")
                    };
                    format!("{}: {}", field.label, value)
                })
                .collect::<Vec<_>>()
                .join(", ")
        )
    }

    fn render_wrong_value_expr(
        &self,
        decl_id: DeclarationId,
        subst: &HashMap<DeclarationId, DeclarationId>,
    ) -> String {
        match self.render_type_expr(decl_id, subst).as_deref() {
            Some("Int") => "\"oops\"".to_string(),
            _ => "1".to_string(),
        }
    }

    fn missing_field_name(&self, children: &[Field]) -> String {
        let mut candidate = "missing_field".to_string();
        while children.iter().any(|field| field.label == candidate) {
            candidate.push('_');
        }
        candidate
    }

    fn match_pattern(&self, variant: &Field) -> String {
        match &self.dag.declaration(variant.ty).connective {
            TypeConnective::Conj { children } if children.is_empty() => variant.label.clone(),
            TypeConnective::Conj { .. } => format!("{}(payload)", variant.label),
            _ => variant.label.clone(),
        }
    }

    fn supports_type_mismatch_claim(&self, children: &[Field]) -> bool {
        !children.is_empty()
            && children.iter().all(|field| {
                !matches!(
                    self.effective_connective(field.ty),
                    Some(TypeConnective::Arrow { .. })
                )
            })
    }

    fn effective_connective(&self, decl_id: DeclarationId) -> Option<&TypeConnective> {
        let decl = self.dag.declaration(decl_id);
        match &decl.connective {
            TypeConnective::Atom(AtomPayload::TypeParam(_)) => None,
            TypeConnective::Instantiation { .. } if decl.name.is_some() => Some(&decl.connective),
            TypeConnective::Instantiation { template, .. } => {
                Some(&self.dag.declaration(*template).connective)
            }
            other => Some(other),
        }
    }
}

fn is_std_file(file: &str) -> bool {
    matches!(file, "src/v3/std/list.dag" | "src/v3/std/verification.dag")
}

fn std_preference_rank(file: &str) -> usize {
    if file.starts_with("src/v3/std/") {
        2
    } else {
        0
    }
}

fn sanitize(s: &str) -> String {
    s.chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect()
}

fn quote_string(s: &str) -> String {
    let escaped = s
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n");
    format!("\"{escaped}\"")
}
