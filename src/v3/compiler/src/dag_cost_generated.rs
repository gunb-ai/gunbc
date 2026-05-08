// AUTO-GENERATED from `src/v3/std/algebra.dag`.
// Regenerate instead of hand-editing.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CollectionSizeEffect {
    ShrinkEffect,
    ProjectionEffect,
    IdentityEffect,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CostShape {
    ShapeConstant,
    ShapeLinearScan,
    ShapeIterateBody,
    ShapeSortBody,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodContract {
    pub algebra_id: DeclarationId,
    pub method_id: DeclarationId,
    pub size_effect: Option<CollectionSizeEffect>,
    pub cost_shape: Option<CostShape>,
    pub callback_element_position: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DegreeAtLeastTwo {
    DegreeTwo,
    DegreeSuccessor { previous: Box<DegreeAtLeastTwo> },
}

impl DegreeAtLeastTwo {
    pub const TWO: Self = Self::DegreeTwo;

    pub fn new(value: i64) -> Option<Self> {
        match value {
            2 => Some(Self::DegreeTwo),
            v if v > 2 => Some(Self::DegreeSuccessor {
                previous: Box::new(Self::new(v - 1)?),
            }),
            _ => None,
        }
    }

    pub fn raw(&self) -> i64 {
        match self {
            Self::DegreeTwo => 2,
            Self::DegreeSuccessor { previous } => previous.raw() + 1,
        }
    }
}

type BoxedSymbolicCostList = NonSingletonList<Box<SymbolicCost>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolicCost {
    ConstantCost { _0: i64 },
    LinearCost { _0: SizeVariable },
    PolynomialCost {
        var: SizeVariable,
        degree: DegreeAtLeastTwo,
    },
    ProductCost { _0: BoxedSymbolicCostList },
    SumCost { _0: BoxedSymbolicCostList },
    LogCost { _0: SizeVariable },
    UnknownCost { _0: String },
}

#[derive(Debug, Clone)]
pub struct SizeVariable {
    pub source_port: PortId,
    pub display_name: Option<String>,
}

impl PartialEq for SizeVariable {
    fn eq(&self, other: &Self) -> bool {
        self.source_port == other.source_port
    }
}

impl Eq for SizeVariable {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AsymptoticClass {
    ClassConstant,
    ClassLog,
    ClassLinear,
    ClassLinearithmic,
    ClassQuadratic,
    ClassPolynomial { degree: PositiveDescentAmount },
    ClassExponential,
    ClassUnknown,
}

pub fn sequential(a: SymbolicCost, b: SymbolicCost) -> SymbolicCost {
    normalize(SymbolicCost::SumCost {
        _0: boxed_cost_list_pair(a, b),
    })
}

pub fn iterate(bound: SymbolicCost, body: SymbolicCost) -> SymbolicCost {
    normalize(SymbolicCost::ProductCost {
        _0: boxed_cost_list_pair(bound, body),
    })
}

pub fn max_path(paths: &[SymbolicCost]) -> SymbolicCost {
    paths
        .iter()
        .fold(SymbolicCost::ConstantCost { _0: 0 }, |acc, candidate| {
            if dominates(candidate, &acc) {
                candidate.clone()
            } else if dominates(&acc, candidate) {
                acc
            } else {
                sequential(acc, candidate.clone())
            }
        })
}

pub fn normalize(cost: SymbolicCost) -> SymbolicCost {
    match cost {
        SymbolicCost::SumCost { _0: terms } => {
            reduce_sum(drop_additive_zero_terms(boxed_terms_to_vec(&terms)))
        }
        SymbolicCost::ProductCost { _0: terms } => {
            reduce_product(drop_multiplicative_one(collapse_on_multiplicative_zero(
                boxed_terms_to_vec(&terms),
            )))
        }
        other => other,
    }
}

fn boxed_cost_list_pair(a: SymbolicCost, b: SymbolicCost) -> BoxedSymbolicCostList {
    BoxedSymbolicCostList {
        first: Box::new(a),
        second: Box::new(b),
        rest: Vec::new(),
    }
}

fn boxed_terms_to_vec(terms: &BoxedSymbolicCostList) -> Vec<SymbolicCost> {
    terms.iter().map(|term| term.as_ref().clone()).collect()
}

fn drop_additive_zero_terms(terms: Vec<SymbolicCost>) -> Vec<SymbolicCost> {
    terms
        .into_iter()
        .filter(|t| !matches!(t, SymbolicCost::ConstantCost { _0: 0 }))
        .collect()
}

fn collapse_on_multiplicative_zero(terms: Vec<SymbolicCost>) -> Vec<SymbolicCost> {
    if terms
        .iter()
        .any(|t| matches!(t, SymbolicCost::ConstantCost { _0: 0 }))
    {
        vec![SymbolicCost::ConstantCost { _0: 0 }]
    } else {
        terms
    }
}

fn drop_multiplicative_one(terms: Vec<SymbolicCost>) -> Vec<SymbolicCost> {
    terms
        .into_iter()
        .filter(|t| !matches!(t, SymbolicCost::ConstantCost { _0: 1 }))
        .collect()
}

fn reduce_sum(mut terms: Vec<SymbolicCost>) -> SymbolicCost {
    terms = drop_dominated_in_sum(terms);
    match terms.len() {
        0 => SymbolicCost::ConstantCost { _0: 0 },
        1 => terms.into_iter().next().unwrap(),
        _ => SymbolicCost::SumCost {
            _0: boxed_cost_list_from_vec(terms),
        },
    }
}

fn reduce_product(terms: Vec<SymbolicCost>) -> SymbolicCost {
    match terms.len() {
        0 => SymbolicCost::ConstantCost { _0: 1 },
        1 => terms.into_iter().next().unwrap(),
        2 => {
            let mut iter = terms.into_iter();
            let a = iter.next().unwrap();
            let b = iter.next().unwrap();
            combine_binary_product(a, b)
        }
        _ => SymbolicCost::ProductCost {
            _0: boxed_cost_list_from_vec(terms),
        },
    }
}

fn boxed_cost_list_from_vec(terms: Vec<SymbolicCost>) -> BoxedSymbolicCostList {
    NonSingletonList::from_vec(terms.into_iter().map(Box::new).collect()).unwrap()
}

fn combine_binary_product(a: SymbolicCost, b: SymbolicCost) -> SymbolicCost {
    if let (SymbolicCost::LinearCost { _0: va }, SymbolicCost::LinearCost { _0: vb }) = (&a, &b) {
        if va == vb {
            return SymbolicCost::PolynomialCost {
                var: va.clone(),
                degree: DegreeAtLeastTwo::TWO,
            };
        }
    }
    SymbolicCost::ProductCost {
        _0: boxed_cost_list_pair(a, b),
    }
}

fn drop_dominated_in_sum(terms: Vec<SymbolicCost>) -> Vec<SymbolicCost> {
    let mut keep: Vec<SymbolicCost> = Vec::with_capacity(terms.len());
    for term in terms {
        let term_dominated = keep.iter().any(|k| dominates(k, &term));
        if term_dominated {
            continue;
        }
        keep.retain(|k| !dominates(&term, k));
        keep.push(term);
    }
    keep
}

/// Structural mirror of composite `dominates` in `v3.std.algebra` (`.dag`:
/// `nsl_to_list` + `fold` / `dominate_scan_init` / `fold_or_dominate_scan`; there is
/// no separate `any_dominates` in the `.dag` surface). The Rust helper is the same
/// semantics as an explicit NSL walk over `first` / `second` / `rest`, not a second
/// authority. `ProductCost` / `SumCost` dominate `b` iff any NSL child dominates `b`.
fn any_dominates(terms: &BoxedSymbolicCostList, b: &SymbolicCost) -> bool {
    if dominates(terms.first.as_ref(), b) {
        return true;
    }
    if dominates(terms.second.as_ref(), b) {
        return true;
    }
    terms
        .rest
        .iter()
        .any(|child| dominates(child.as_ref(), b))
}

pub fn dominates<A, B>(a: A, b: B) -> bool
where
    A: std::borrow::Borrow<SymbolicCost>,
    B: std::borrow::Borrow<SymbolicCost>,
{
    let a = a.borrow();
    let b = b.borrow();
    match a {
        SymbolicCost::UnknownCost { .. } => true,
        SymbolicCost::ConstantCost { .. } => matches!(b, SymbolicCost::ConstantCost { .. }),
        SymbolicCost::LinearCost { _0: va } => match b {
            SymbolicCost::ConstantCost { .. } | SymbolicCost::LogCost { .. } => true,
            SymbolicCost::LinearCost { _0: vb } => va == vb,
            SymbolicCost::PolynomialCost { var: _, degree: _ } => false,
            _ => false,
        },
        SymbolicCost::PolynomialCost {
            var: va,
            degree: ka,
        } => match b {
            SymbolicCost::ConstantCost { .. } | SymbolicCost::LogCost { .. } => true,
            SymbolicCost::LinearCost { _0: vb } => va == vb,
            SymbolicCost::PolynomialCost {
                var: vb,
                degree: kb,
            } => va == vb && ka.raw() >= kb.raw(),
            _ => false,
        },
        SymbolicCost::LogCost { _0: va } => match b {
            SymbolicCost::ConstantCost { .. } => true,
            SymbolicCost::LogCost { _0: vb } => va == vb,
            _ => false,
        },
        SymbolicCost::ProductCost { _0: terms } | SymbolicCost::SumCost { _0: terms } => {
            any_dominates(terms, b)
        }
    }
}

pub fn classify<C>(cost: C) -> AsymptoticClass
where
    C: std::borrow::Borrow<SymbolicCost>,
{
    match cost.borrow() {
        SymbolicCost::ConstantCost { .. } => AsymptoticClass::ClassConstant,
        SymbolicCost::LinearCost { .. } => AsymptoticClass::ClassLinear,
        SymbolicCost::LogCost { .. } => AsymptoticClass::ClassLog,
        SymbolicCost::PolynomialCost { .. } => AsymptoticClass::ClassQuadratic,
        SymbolicCost::ProductCost { .. }
        | SymbolicCost::SumCost { .. }
        | SymbolicCost::UnknownCost { .. } => AsymptoticClass::ClassUnknown,
    }
}
