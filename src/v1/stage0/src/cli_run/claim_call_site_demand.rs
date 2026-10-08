//! PER-CLAIM CALL-SITE DEMAND IDENTITY — the observation `v2.workflow.floor_pure_producer_share`
//! `derive_cross_claim_share` decides over.
//!
//! The row type is declared in `.dag` (`CallSiteDemandObservation`, keyed by
//! `std.computation_identity`, causes the closed `CallSiteDemandCause`); this module is ONE
//! REALIZATION of it. It exists only because no `.dag` carrier yet hands the floor the call sites a
//! planned claim reaches together with their argument rows -- demand identity is the declared
//! frontier of demand-engine M1.b -- and its seed-growth receipt
//! (`gunbc.floor_call_site_demand_seed_growth`) names that capability as the trigger that deletes
//! it.
//!
//! WHAT IT DOES. For each planned claim it walks the claim function's reach over the prepared
//! subject -- calls, and value references that name a declaration -- and reads every call site in
//! every reached body. A site whose callee is a resolved pure source declaration and whose every
//! argument normalizes to a CLOSED expression (literals, list literals, variant constructors,
//! references to module declarations, binary operators and pure calls over those) has a
//! computation identity: the callee plus the canonical normalized argument row. Every other site
//! is counted under the `.dag` cause it falls in. Rows leave aggregated per identity with the
//! number of DISTINCT planned claims reaching it, because that count is a fact across the
//! claim-frame boundary only the seed can see; the threshold, the identity grade and every
//! exclusion are the `.dag` fold's.
//!
//! WHAT A WRONG JUDGMENT HERE CAN COST, stated because it bounds how much this module must be
//! trusted: the tier keys every store on the evaluated argument VALUES and verifies the stored
//! canonical preimage before any serve, so a site misjudged closed costs at most a store the claim
//! did not need, and a site misjudged open costs a missed share. Neither can serve a wrong value.
//! The normalizer is conservative by construction: an expression form it does not name is open.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::rc::Rc;

use crate::v1_compiler_infer_items::ResolvedGraph;

/// The prepared subject's source text index, as `v1_std_core` reads names through it.
type SourceIndices = im::HashMap<String, Rc<NewlineIndex>>;
use crate::v1_std_core::{
    arg_value, authored_name_at, source_text_at, CallTargetIdentity, ExprData, LeafOwner,
    NewlineIndex, Node, VarBindingKind,
};

/// The `.dag` `CallSiteDemandCause` arms, by their declared spelling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum CallSiteDemandCause {
    CalleeUnresolved,
    CalleeIsAValue,
    CalleeDeclaresEffects,
    ArgumentNotClosedConstant,
    CallShapeUnread,
}

impl CallSiteDemandCause {
    pub(crate) fn variant(self) -> &'static str {
        match self {
            CallSiteDemandCause::CalleeUnresolved => "CalleeUnresolved",
            CallSiteDemandCause::CalleeIsAValue => "CalleeIsAValue",
            CallSiteDemandCause::CallShapeUnread => "CallShapeUnread",
            CallSiteDemandCause::CalleeDeclaresEffects => "CalleeDeclaresEffects",
            CallSiteDemandCause::ArgumentNotClosedConstant => "ArgumentNotClosedConstant",
        }
    }
}

/// The normalizer named on every closed row's `NormalizedIdentical` identity grade.
pub(crate) const CLOSED_ARGUMENT_NORMALIZER: &str =
    "v1_compiler.cli_run.claim_call_site_demand closed_argument_row";

/// One `.dag` `CallSiteDemandObservation`, aggregated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CallSiteDemandRow {
    Closed {
        producer: String,
        argument_preimage: String,
        /// The distinct DECLARED claims reaching this identity, `module.function`.
        claims: Vec<String>,
        /// How many of `claims` this run plans.
        planned_claims: u64,
        sites: Vec<String>,
        /// Each distinct (claim, read) pair over this identity's sites, claims as in `claims`.
        reads: Vec<(String, ConsumerRead)>,
    },
    Unadmissible {
        cause: CallSiteDemandCause,
        claims: u64,
        sites: u64,
    },
}

/// A call site's span, as the tier admits it at run time: `(file, start, end)` in bytes.
pub(crate) type SiteKey = (String, i64, i64);

pub(crate) fn site_key_of(node: &Node) -> SiteKey {
    (node.span.file.to_string(), node.span.start, node.span.end)
}

/// Immediate projection (or typed Unread) for each call that is a field-access base.
/// A field access whose base is not a readable call still marks every call child `Unread`
/// (`ProjectionBaseUnreadable`) so the call cannot later default to `WholeValue`.
pub(crate) fn immediate_consumer_projections(
    nodes: &[Rc<Node>],
    source_indices: Rc<SourceIndices>,
) -> HashMap<SiteKey, ConsumerRead> {
    let mut projections: HashMap<SiteKey, ConsumerRead> = HashMap::new();
    for n in nodes {
        if !matches!(n.expr_data.as_ref(), ExprData::ExprFieldAccess { .. }) {
            continue;
        }
        let unread_call_children = |projections: &mut HashMap<SiteKey, ConsumerRead>| {
            for child in n.children.iter() {
                if matches!(child.expr_data.as_ref(), ExprData::ExprCall { .. }) {
                    projections.insert(
                        site_key_of(child),
                        ConsumerRead::Unread(ConsumerReadCause::ProjectionBaseUnreadable),
                    );
                }
            }
        };
        let Ok(base) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            crate::v1_std_core::field_access_base(n.clone())
        })) else {
            unread_call_children(&mut projections);
            continue;
        };
        if !matches!(base.expr_data.as_ref(), ExprData::ExprCall { .. }) {
            unread_call_children(&mut projections);
            continue;
        }
        let field = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            crate::v1_std_core::field_access_field_at(n.clone(), source_indices.clone())
        }));
        let read = match field {
            Err(_) => ConsumerRead::Unread(ConsumerReadCause::ProjectionFieldNameUnreadable),
            Ok(f) if f.is_empty() => {
                ConsumerRead::Unread(ConsumerReadCause::ProjectionFieldNameEmpty)
            }
            Ok(f) => ConsumerRead::ProjectedField(f),
        };
        projections.insert(site_key_of(&base), read);
    }
    projections
}

pub(crate) fn render_site(site: &SiteKey) -> String {
    format!("{}:{}-{}", site.0, site.1, site.2)
}

/// The `.dag` `ConsumerRead`: what the expression consuming a call site's result reads off it.
/// Only the IMMEDIATE projection is observed (`f(x).field`); any other consumer reads the whole
/// value. A projection whose field name cannot be read is `Unread` with its `.dag`
/// `ConsumerReadCause` -- `ConsumerReadUnobserved { cause }` -- never a guessed grain.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum ConsumerRead {
    WholeValue,
    ProjectedField(String),
    Unread(ConsumerReadCause),
}

/// The `.dag` `ConsumerReadCause` arms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum ConsumerReadCause {
    ProjectionFieldNameUnreadable,
    ProjectionFieldNameEmpty,
    ProjectionBaseUnreadable,
    ShareMapLookupUnreadable,
}

impl ConsumerReadCause {
    pub(crate) fn variant(self) -> &'static str {
        match self {
            ConsumerReadCause::ProjectionFieldNameUnreadable => "ProjectionFieldNameUnreadable",
            ConsumerReadCause::ProjectionFieldNameEmpty => "ProjectionFieldNameEmpty",
            ConsumerReadCause::ProjectionBaseUnreadable => "ProjectionBaseUnreadable",
            ConsumerReadCause::ShareMapLookupUnreadable => "ShareMapLookupUnreadable",
        }
    }
}

enum SiteFact {
    Closed {
        producer: String,
        preimage: String,
        read: ConsumerRead,
    },
    Unadmissible(CallSiteDemandCause),
    /// An unadmissible site whose callee IS known, kept by producer so a reader can disposition
    /// each producer (whose site is open, or effectful) by identity.
    UnadmissibleOf {
        producer: String,
        cause: CallSiteDemandCause,
    },
}

struct DeclFacts {
    reads: Vec<(String, String)>,
    sites: Vec<(SiteKey, SiteFact)>,
}

/// The observation over one prepared subject. Declarations are read lazily and each body is
/// walked once for the whole run, however many claims reach it.
pub(crate) struct CallSiteDemandObserver<'a> {
    graph: &'a ResolvedGraph,
    source_indices: Rc<SourceIndices>,
    decls: HashMap<(String, String), Rc<Node>>,
    facts: HashMap<(String, String), Rc<DeclFacts>>,
    open_producers: Vec<(String, CallSiteDemandCause, u64, u64)>,
}

#[derive(Default)]
struct IdentityCell {
    claims: u64,
    last_claim: usize,
    /// The claim indices counted in `claims`, in planning order.
    claim_indices: Vec<usize>,
    /// How many of them are planned (index below the planned count).
    planned: u64,
    sites: BTreeSet<SiteKey>,
    /// (claim index, read), each pair once.
    reads: BTreeSet<(usize, ConsumerRead)>,
}

impl<'a> CallSiteDemandObserver<'a> {
    pub(crate) fn new(graph: &'a ResolvedGraph, source_indices: Rc<SourceIndices>) -> Self {
        let mut decls = HashMap::new();
        for module in graph.modules.iter() {
            let module_path = module.type_env.module_path.clone();
            for item in module.items.iter() {
                if !item.name.is_empty() {
                    decls.insert((module_path.clone(), item.name.clone()), item.clone());
                }
            }
        }
        CallSiteDemandObserver {
            graph,
            source_indices,
            decls,
            facts: HashMap::new(),
            open_producers: Vec::new(),
        }
    }

    /// The declaration node for a planned claim or producer, by qualified identity.
    pub(crate) fn decl(&self, module_path: &str, name: &str) -> Option<&Rc<Node>> {
        self.decls.get(&(module_path.to_string(), name.to_string()))
    }

    /// `claims[..planned]` are this run's planned claims; the rest are the other claims declared
    /// in the prepared subject. Every one contributes demand; `planned_claims` counts the first kind.
    pub(crate) fn observe(
        &mut self,
        claims: &[(String, String)],
        planned: usize,
    ) -> (Vec<CallSiteDemandRow>, Vec<String>) {
        let mut closed: BTreeMap<(String, String), IdentityCell> = BTreeMap::new();
        let mut open: BTreeMap<CallSiteDemandCause, IdentityCell> = BTreeMap::new();
        let mut open_by_producer: BTreeMap<(String, CallSiteDemandCause), IdentityCell> =
            BTreeMap::new();
        let mut unresolved_claims = Vec::new();
        for (index, (module_path, function)) in claims.iter().enumerate() {
            let claim_number = index + 1;
            let root = (module_path.clone(), function.clone());
            if !self.decls.contains_key(&root) {
                unresolved_claims.push(format!("{module_path}.{function}"));
                continue;
            }
            let mut seen: HashSet<(String, String)> = HashSet::new();
            let mut frontier = vec![root.clone()];
            seen.insert(root);
            while let Some(decl) = frontier.pop() {
                let Some(facts) = self.facts_of(&decl) else {
                    continue;
                };
                for read in &facts.reads {
                    if seen.insert(read.clone()) {
                        frontier.push(read.clone());
                    }
                }
                for (site, fact) in &facts.sites {
                    let cell = match fact {
                        SiteFact::Closed {
                            producer,
                            preimage,
                            read,
                        } => {
                            let cell = closed
                                .entry((producer.clone(), preimage.clone()))
                                .or_default();
                            cell.reads.insert((index, read.clone()));
                            cell
                        }
                        SiteFact::Unadmissible(cause) => open.entry(*cause).or_default(),
                        SiteFact::UnadmissibleOf { producer, cause } => {
                            let by = open_by_producer
                                .entry((producer.clone(), *cause))
                                .or_default();
                            if by.last_claim != claim_number {
                                by.last_claim = claim_number;
                                by.claims += 1;
                            }
                            by.sites.insert(site.clone());
                            open.entry(*cause).or_default()
                        }
                    };
                    if cell.last_claim != claim_number {
                        cell.last_claim = claim_number;
                        cell.claims += 1;
                        cell.claim_indices.push(index);
                        if index < planned {
                            cell.planned += 1;
                        }
                    }
                    cell.sites.insert(site.clone());
                }
            }
        }
        let mut rows: Vec<CallSiteDemandRow> = closed
            .into_iter()
            .map(
                |((producer, argument_preimage), cell)| CallSiteDemandRow::Closed {
                    producer,
                    argument_preimage,
                    claims: cell
                        .claim_indices
                        .iter()
                        .map(|i| format!("{}.{}", claims[*i].0, claims[*i].1))
                        .collect(),
                    planned_claims: cell.planned,
                    sites: cell.sites.iter().map(render_site).collect(),
                    reads: cell
                        .reads
                        .iter()
                        .map(|(i, read)| {
                            (format!("{}.{}", claims[*i].0, claims[*i].1), read.clone())
                        })
                        .collect(),
                },
            )
            .collect();
        rows.extend(
            open.into_iter()
                .map(|(cause, cell)| CallSiteDemandRow::Unadmissible {
                    cause,
                    claims: cell.claims,
                    sites: cell.sites.len() as u64,
                }),
        );
        self.open_producers = open_by_producer
            .into_iter()
            .map(|((producer, cause), cell)| {
                (producer, cause, cell.claims, cell.sites.len() as u64)
            })
            .collect();
        (rows, unresolved_claims)
    }

    /// The last observation's unadmissible sites with a known callee, per (producer, cause):
    /// (producer, cause, distinct claims, sites). Rendered by the floor for disposition only.
    pub(crate) fn open_producers(&self) -> &[(String, CallSiteDemandCause, u64, u64)] {
        &self.open_producers
    }

    fn facts_of(&mut self, decl: &(String, String)) -> Option<Rc<DeclFacts>> {
        if let Some(f) = self.facts.get(decl) {
            return Some(f.clone());
        }
        let node = self.decls.get(decl)?.clone();
        let facts = Rc::new(self.read_declaration(&decl.0, &node));
        self.facts.insert(decl.clone(), facts.clone());
        Some(facts)
    }

    fn name_of(&self, node: &Rc<Node>) -> String {
        let authored = authored_name_at(self.source_indices.clone(), node.clone());
        if authored.is_empty() {
            node.name.clone()
        } else {
            authored
        }
    }

    fn source_of(&self, node: &Node) -> Option<String> {
        let index = self.source_indices.get(node.span.file.as_str())?;
        Some(source_text_at(index.clone(), node.span.clone()))
    }

    /// A spelling read from `module`, resolved the way module-scope lookup resolves it: a
    /// qualified spelling names its declaring module; a bare one is a sibling, or the unique
    /// owner the graph records for that leaf. An ambiguous or absent leaf is unresolved.
    fn resolve(&self, module: &str, spelling: &str) -> Option<(String, String)> {
        if let Some((qualifier, leaf)) = spelling.rsplit_once('.') {
            let key = (qualifier.to_string(), leaf.to_string());
            return self.decls.contains_key(&key).then_some(key);
        }
        let sibling = (module.to_string(), spelling.to_string());
        if self.decls.contains_key(&sibling) {
            return Some(sibling);
        }
        match self
            .graph
            .item_leaf_owner_modules
            .get(spelling)
            .map(|o| o.as_ref())
        {
            Some(LeafOwner::SingleOwner { module: owner }) => {
                let key = (owner.clone(), spelling.to_string());
                self.decls.contains_key(&key).then_some(key)
            }
            _ => None,
        }
    }

    /// The static callee of a call site, or the typed reason there is none.
    fn call_target(
        &self,
        module: &str,
        call: &Rc<Node>,
    ) -> Result<(String, String), CallSiteDemandCause> {
        use crate::v1_std_core::CallSemantics;
        if let ExprData::ExprCall {
            call_semantics: Some(semantics),
            ..
        } = call.expr_data.as_ref()
        {
            // Matched EXHAUSTIVELY on the semantics rather than through `target()`, which has no
            // arm for a function-value call: a new call shape is then a compile error here, not a
            // run-time panic in the floor.
            let target = match semantics.as_ref() {
                CallSemantics::FunctionValueCallSemantics => {
                    return Err(CallSiteDemandCause::CalleeIsAValue)
                }
                CallSemantics::PlainCallSemantics { target }
                | CallSemantics::ResolvedDirectCallSemantics { target, .. }
                | CallSemantics::LookupCallSemantics { target } => target.clone(),
            };
            match target.as_ref() {
                CallTargetIdentity::SourceDeclarationCall {
                    owner_module_path,
                    decl_name,
                } => {
                    let key = (owner_module_path.clone(), decl_name.clone());
                    return if self.decls.contains_key(&key) {
                        Ok(key)
                    } else {
                        Err(CallSiteDemandCause::CalleeUnresolved)
                    };
                }
                CallTargetIdentity::LocallyBoundCall { .. } => {
                    return Err(CallSiteDemandCause::CalleeIsAValue)
                }
                CallTargetIdentity::RuntimePrimitiveCall { .. } => {
                    return Err(CallSiteDemandCause::CalleeUnresolved)
                }
                CallTargetIdentity::CallableTargetUndetermined => {}
            }
        }
        let spelling =
            crate::v1_std_core::expr_call_func_at(call.clone(), self.source_indices.clone());
        self.resolve(module, &spelling)
            .ok_or(CallSiteDemandCause::CalleeUnresolved)
    }

    fn read_declaration(&self, module: &str, decl: &Rc<Node>) -> DeclFacts {
        let mut binders: HashSet<String> = HashSet::new();
        for p in decl.params.iter() {
            binders.insert(self.name_of(p));
        }
        let mut nodes: Vec<Rc<Node>> = Vec::new();
        let mut stack: Vec<Rc<Node>> = Vec::new();
        if let Some(body) = &decl.body {
            stack.push(body.clone());
        }
        stack.extend(decl.children.iter().cloned());
        while let Some(n) = stack.pop() {
            if matches!(n.expr_data.as_ref(), ExprData::ExprLet) {
                binders.insert(self.name_of(&n));
            }
            for p in n.params.iter() {
                binders.insert(self.name_of(p));
            }
            stack.extend(n.children.iter().cloned());
            if let Some(body) = &n.body {
                stack.push(body.clone());
            }
            if let Some(t) = &n.transport {
                stack.push(t.clone());
            }
            nodes.push(n);
        }
        // Immediate projection, or Unread when the base cannot be read as a call.
        let projections = immediate_consumer_projections(&nodes, self.source_indices.clone());
        let mut reads: BTreeSet<(String, String)> = BTreeSet::new();
        let mut sites: Vec<(SiteKey, SiteFact)> = Vec::new();
        for n in &nodes {
            match n.expr_data.as_ref() {
                ExprData::ExprCall { .. } => {
                    // NO CALL SHAPE MAY STOP THE FLOOR OR VANISH: a panic while reading one site
                    // becomes that site's typed, counted cause.
                    let read = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        let target = self.call_target(module, n);
                        let read = projections
                            .get(&site_key_of(n))
                            .cloned()
                            .unwrap_or(ConsumerRead::WholeValue);
                        let fact = self.site_fact(module, n, target.clone(), &binders, read);
                        (target.ok(), fact)
                    }));
                    let (target, fact) = match read {
                        Ok(read) => read,
                        Err(_) => (
                            None,
                            SiteFact::Unadmissible(CallSiteDemandCause::CallShapeUnread),
                        ),
                    };
                    if let Some(t) = &target {
                        reads.insert(t.clone());
                    }
                    sites.push((site_key_of(n), fact));
                }
                ExprData::ExprVar { binding_kind } => {
                    if matches!(
                        binding_kind.as_deref(),
                        Some(VarBindingKind::MatchBoundBinding)
                            | Some(VarBindingKind::VariantValueBinding { .. })
                    ) {
                        continue;
                    }
                    let name = self.name_of(n);
                    if binders.contains(&name) {
                        continue;
                    }
                    if let Some(t) = self.resolve(module, &name) {
                        reads.insert(t);
                    }
                }
                _ => {}
            }
        }
        DeclFacts {
            reads: reads.into_iter().collect(),
            sites,
        }
    }

    fn site_fact(
        &self,
        module: &str,
        call: &Rc<Node>,
        target: Result<(String, String), CallSiteDemandCause>,
        binders: &HashSet<String>,
        read: ConsumerRead,
    ) -> SiteFact {
        let target = match target {
            Ok(t) => t,
            Err(cause) => return SiteFact::Unadmissible(cause),
        };
        if self
            .decls
            .get(&target)
            .is_some_and(|callee| !callee.uses.is_empty())
        {
            return SiteFact::UnadmissibleOf {
                producer: format!("{}.{}", target.0, target.1),
                cause: CallSiteDemandCause::CalleeDeclaresEffects,
            };
        }
        match self.closed_arguments(module, call, binders) {
            Some(preimage) => SiteFact::Closed {
                producer: format!("{}.{}", target.0, target.1),
                preimage,
                read,
            },
            None => SiteFact::UnadmissibleOf {
                producer: format!("{}.{}", target.0, target.1),
                cause: CallSiteDemandCause::ArgumentNotClosedConstant,
            },
        }
    }

    /// The canonical argument row, in source order, each argument `name=expr` (`=expr` when
    /// positional). `None` when any argument is open.
    fn closed_arguments(
        &self,
        module: &str,
        call: &Rc<Node>,
        binders: &HashSet<String>,
    ) -> Option<String> {
        let mut parts = Vec::new();
        for arg in call.children.iter().filter(|a| !a.children.is_empty()) {
            let name = authored_name_at(self.source_indices.clone(), arg.clone());
            let value = self.closed_expression(module, &arg_value(arg.clone()), binders)?;
            parts.push(format!("{name}={value}"));
        }
        Some(format!("({})", parts.join(",")))
    }

    fn closed_expression(
        &self,
        module: &str,
        expr: &Rc<Node>,
        binders: &HashSet<String>,
    ) -> Option<String> {
        match expr.expr_data.as_ref() {
            ExprData::ExprLiteral { .. } | ExprData::ExprElaboratedLiteral { .. } => {
                Some(format!("lit:{}", self.source_of(expr)?))
            }
            ExprData::ExprListLit => {
                let items = expr
                    .children
                    .iter()
                    .map(|c| self.closed_expression(module, c, binders))
                    .collect::<Option<Vec<_>>>()?;
                Some(format!("[{}]", items.join(",")))
            }
            ExprData::ExprBinOp { op, .. } => {
                let operands = expr
                    .children
                    .iter()
                    .map(|c| self.closed_expression(module, c, binders))
                    .collect::<Option<Vec<_>>>()?;
                Some(format!("{op:?}({})", operands.join(",")))
            }
            ExprData::ExprVar { binding_kind } => {
                let name = self.name_of(expr);
                match binding_kind.as_deref() {
                    Some(VarBindingKind::MatchBoundBinding) => None,
                    Some(VarBindingKind::VariantValueBinding { parent_enum, .. }) => {
                        Some(format!("variant:{parent_enum}.{name}"))
                    }
                    _ if binders.contains(&name) => None,
                    _ => self
                        .resolve(module, &name)
                        .map(|(m, d)| format!("ref:{m}.{d}")),
                }
            }
            ExprData::ExprCall { .. } => {
                let target = self.call_target(module, expr).ok()?;
                if self.decls.get(&target).is_some_and(|c| !c.uses.is_empty()) {
                    return None;
                }
                let args = self.closed_arguments(module, expr, binders)?;
                Some(format!("call:{}.{}{}", target.0, target.1, args))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn graph_of(sources: &[(&str, &str)]) -> (Rc<ResolvedGraph>, Rc<SourceIndices>) {
        let files: Vec<Rc<crate::v1_compiler_compile::SourceFile>> = sources
            .iter()
            .map(|(path, content)| {
                Rc::new(crate::v1_compiler_compile::SourceFile {
                    path: path.to_string(),
                    content: content.to_string(),
                })
            })
            .collect();
        let result = crate::v1_compiler_compile::compile_to_resolved(Rc::new(files.into()));
        (
            result.graph.as_ref().expect("fixture graph").clone(),
            result.source_indices.clone(),
        )
    }

    // The #12506 shape in miniature: a module-constant fixture text assembled behind a nullary
    // helper that several claims call, beside a claim with its own fixture text, a helper whose
    // argument is a parameter, and an effectful callee.
    const FIXTURE: &str = "module fixture.n7\n\
         fn assemble(src: String) -> Int { 3 }\n\
         fn shared() -> Int { assemble(src: \"module p\") }\n\
         fn from_param(text: String) -> Int { assemble(src: text) }\n\
         fn reads() -> String uses net: Network { \"\" }\n\
         fn reading_helper() -> String { reads() }\n\
         fn claim_a() -> Bool { shared() == 3 }\n\
         fn claim_b() -> Bool { shared() == shared() }\n\
         fn claim_c() -> Bool { assemble(src: \"module own\") == 3 }\n\
         fn claim_d() -> Bool { from_param(text: \"x\") == 3 }\n\
         fn claim_e() -> Bool { reading_helper() == \"\" }\n\
         fn apply(f: fn(Int) -> Int) -> Int { f(1) }\n\
         fn inc(n: Int) -> Int { n }\n\
         fn claim_f() -> Bool { apply(f: inc) == 1 }\n";

    fn observe(claims: &[&str]) -> Vec<CallSiteDemandRow> {
        observe_source(FIXTURE, claims)
    }

    fn observe_source(source: &str, claims: &[&str]) -> Vec<CallSiteDemandRow> {
        let (graph, indices) = graph_of(&[("workspace/src/n7.dag", source)]);
        let mut observer = CallSiteDemandObserver::new(&graph, indices);
        let planned: Vec<(String, String)> = claims
            .iter()
            .map(|c| ("fixture.n7".to_string(), c.to_string()))
            .collect();
        let (rows, unresolved) = observer.observe(&planned, planned.len());
        assert!(
            unresolved.is_empty(),
            "every fixture claim resolves: {unresolved:?}; indexed={:?}; modules={:?}; diagnostics={}",
            observer.decls.keys().collect::<Vec<_>>(),
            graph.modules.iter().map(|m| m.type_env.module_path.clone()).collect::<Vec<_>>(),
            graph.diagnostics.len()
        );
        rows
    }

    fn closed_claims(
        rows: &[CallSiteDemandRow],
        producer: &str,
        preimage_contains: &str,
    ) -> Option<u64> {
        rows.iter().find_map(|r| match r {
            CallSiteDemandRow::Closed {
                producer: p,
                argument_preimage,
                claims,
                ..
            } if p == producer && argument_preimage.contains(preimage_contains) => {
                Some(claims.len() as u64)
            }
            _ => None,
        })
    }

    fn has_cause(rows: &[CallSiteDemandRow], cause: CallSiteDemandCause) -> bool {
        rows.iter()
            .any(|r| matches!(r, CallSiteDemandRow::Unadmissible { cause: c, .. } if *c == cause))
    }

    // THE DISCRIMINATING PAIR at the observation: the shared fixture text reached through one
    // helper by two claims counts TWO distinct claims -- and claim_b's second call does not make
    // it three -- while the claim's own fixture text counts one.
    #[test]
    fn a_constant_reached_by_two_claims_counts_two_and_a_private_one_counts_one() {
        let rows = observe(&["claim_a", "claim_b", "claim_c"]);
        assert_eq!(
            closed_claims(&rows, "fixture.n7.assemble", "module p"),
            Some(2),
            "the shared fixture is demanded by exactly two distinct claims: {rows:?}"
        );
        assert_eq!(
            closed_claims(&rows, "fixture.n7.assemble", "module own"),
            Some(1),
            "a claim's private fixture is demanded once: {rows:?}"
        );
    }

    // THE STATIC DEMAND IS THE PLANNED CLAIMS' AND NOTHING WIDER: with claim_b not planned, the
    // shared fixture is reached by one claim, so no reuse obligation exists for this run.
    #[test]
    fn an_unplanned_claim_contributes_no_demand() {
        let rows = observe(&["claim_a", "claim_c"]);
        assert_eq!(
            closed_claims(&rows, "fixture.n7.assemble", "module p"),
            Some(1)
        );
    }

    // DECLARED DEMAND, PLANNED COUNT: with claim_b declared but not planned, the shared fixture is
    // still reached by two declared claims, and exactly one of them is planned.
    #[test]
    fn declared_unplanned_claims_count_as_demand_and_are_not_counted_planned() {
        let (graph, indices) = graph_of(&[("workspace/src/n7.dag", FIXTURE)]);
        let mut observer = CallSiteDemandObserver::new(&graph, indices);
        let population: Vec<(String, String)> = ["claim_a", "claim_b"]
            .iter()
            .map(|c| ("fixture.n7".to_string(), c.to_string()))
            .collect();
        let (rows, _) = observer.observe(&population, 1);
        let row = rows.iter().find_map(|r| match r {
            CallSiteDemandRow::Closed {
                producer,
                argument_preimage,
                claims,
                planned_claims,
                ..
            } if producer == "fixture.n7.assemble" && argument_preimage.contains("module p") => {
                Some((claims.len(), *planned_claims))
            }
            _ => None,
        });
        assert_eq!(row, Some((2, 1)), "{rows:?}");
    }

    // NEVER WIDEN: a call whose argument is the enclosing function's parameter has no closed
    // identity, so it is counted under its cause instead of admitting `assemble` wholesale.
    #[test]
    fn a_parameter_bound_argument_is_counted_open() {
        let rows = observe(&["claim_d"]);
        assert!(
            has_cause(&rows, CallSiteDemandCause::ArgumentNotClosedConstant),
            "the parameter-bound site must be counted open: {rows:?}"
        );
        assert!(
            closed_claims(&rows, "fixture.n7.assemble", "text").is_none(),
            "no closed identity may be minted over a parameter: {rows:?}"
        );
    }

    // THE FUNCTION-VALUE CONTROL (the shape that panicked floor run 37083945879): a call through a
    // parameter of function type is counted under its own cause, the walk does not panic, and the
    // rest of the claim's reach is still observed (the `apply(f: inc)` site is closed).
    #[test]
    fn a_function_value_call_is_counted_as_a_value_callee_never_a_panic() {
        let rows = observe(&["claim_f"]);
        assert!(
            has_cause(&rows, CallSiteDemandCause::CalleeIsAValue)
                || has_cause(&rows, CallSiteDemandCause::CalleeUnresolved),
            "the f(1) site must be counted under a callee cause: {rows:?}"
        );
        assert!(
            !has_cause(&rows, CallSiteDemandCause::CallShapeUnread),
            "a function-value call is a recognised shape, not an unread one: {rows:?}"
        );
        assert!(
            closed_claims(&rows, "fixture.n7.apply", "ref:fixture.n7.inc").is_some(),
            "the closed apply(f: inc) site is still observed: {rows:?}"
        );
    }

    // An effectful callee is counted under its own cause, never closed, however constant its
    // arguments are.
    #[test]
    fn an_effectful_callee_is_counted_under_its_cause() {
        let rows = observe(&["claim_e"]);
        assert!(
            has_cause(&rows, CallSiteDemandCause::CalleeDeclaresEffects),
            "the effectful nullary call must be counted under CalleeDeclaresEffects: {rows:?}"
        );
        assert!(closed_claims(&rows, "fixture.n7.reads", "").is_none());
    }
    // THE PROJECTION FACT: the bundle shape of #13113 in miniature. Two claims read different
    // fields projected immediately off one closed call, a third reads one of them, and a fourth
    // consumes the whole value; each (claim, read) pair is reported once, so the fold can tell a
    // bundle of disjoint slices from one shared value.
    const BUNDLE: &str = "module fixture.n7\n\
         type Pair {\n  a: Bool\n  b: Bool\n}\n\
         fn verdicts() -> Pair { Pair { a: true, b: false } }\n\
         fn claim_a() -> Bool { verdicts().a }\n\
         fn claim_b() -> Bool { verdicts().b }\n\
         fn claim_a_again() -> Bool { verdicts().a && verdicts().a }\n\
         fn claim_whole() -> Bool { verdicts() == verdicts() }\n";

    fn reads_of(rows: &[CallSiteDemandRow], producer: &str) -> Vec<(String, ConsumerRead)> {
        rows.iter()
            .find_map(|r| match r {
                CallSiteDemandRow::Closed {
                    producer: p, reads, ..
                } if p == producer => Some(reads.clone()),
                _ => None,
            })
            .unwrap_or_default()
    }

    #[test]
    fn each_claims_immediate_projection_is_reported_once_and_a_bare_use_reads_the_whole() {
        let rows = observe_source(
            BUNDLE,
            &["claim_a", "claim_b", "claim_a_again", "claim_whole"],
        );
        let reads = reads_of(&rows, "fixture.n7.verdicts");
        let field = |f: &str| ConsumerRead::ProjectedField(f.to_string());
        assert_eq!(
            reads,
            vec![
                ("fixture.n7.claim_a".to_string(), field("a")),
                ("fixture.n7.claim_b".to_string(), field("b")),
                ("fixture.n7.claim_a_again".to_string(), field("a")),
                (
                    "fixture.n7.claim_whole".to_string(),
                    ConsumerRead::WholeValue
                ),
            ],
            "{rows:?}"
        );
    }

    // review 77161: a field access whose first child is not the call still has a call child;
    // that call is Unread(ProjectionBaseUnreadable), never WholeValue via unwrap_or.
    #[test]
    fn an_unreadable_field_access_base_marks_the_call_child_unread() {
        use crate::std_types::SourceSpan;
        use crate::v1_std_core::{
            make_expr_error_node, make_expr_node, ExprErrorKind, NodeOccurrenceIdentity,
        };
        let span = |start: i64, end: i64| {
            Rc::new(SourceSpan {
                file: "probe.dag".to_string(),
                start,
                end,
            })
        };
        let err = make_expr_error_node(
            Rc::new(NodeOccurrenceIdentity::OccurrenceSynthetic),
            ExprErrorKind::InternalExprError,
            "missing base".to_string(),
            span(1, 2),
        );
        let call = make_expr_node(
            Rc::new(NodeOccurrenceIdentity::OccurrenceSynthetic),
            Rc::new(ExprData::ExprCall {
                call_semantics: None,
                descent_evidence: None,
            }),
            crate::v1_std_core::empty_node_list(),
            None,
            span(10, 20),
        );
        let access = make_expr_node(
            Rc::new(NodeOccurrenceIdentity::OccurrenceSynthetic),
            Rc::new(ExprData::ExprFieldAccess { summary: None }),
            Rc::new(vec![err, call.clone()].into()),
            None,
            span(1, 30),
        );
        let projections =
            immediate_consumer_projections(&[access, call.clone()], Rc::new(im::HashMap::new()));
        let read = projections
            .get(&site_key_of(&call))
            .cloned()
            .unwrap_or(ConsumerRead::WholeValue);
        assert_eq!(
            read,
            ConsumerRead::Unread(ConsumerReadCause::ProjectionBaseUnreadable),
            "{projections:?}"
        );
    }
}
