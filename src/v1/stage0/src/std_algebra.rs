use self::AlgebraProfile::*;
use self::AlgebraTypeTemplate::*;
use self::CollectionSizeEffect::*;
use self::ContainerSource::*;
use self::CostShape::*;
use self::Ordering::*;
use crate::std_error_primitives::DivError::*;
use crate::std_error_primitives::Result::*;
pub use crate::std_error_primitives::{DivError, Result};
use crate::v1_rt;
use crate::v1_rt::Witness;
use crate::v1_rt::Witness::{Holds, Violates};
use crate::NonEmptyBTreeSet;
use crate::NonEmptyVec;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone)]
pub struct Magma<T> {
    pub op: Rc<dyn Fn(T, T) -> T>,
}

#[derive(Clone)]
pub struct Semigroup<T> {
    pub op: Rc<dyn Fn(T, T) -> T>,
}

#[derive(Clone)]
pub struct Monoid<T> {
    pub op: Rc<dyn Fn(T, T) -> T>,
    pub identity: Box<T>,
}

#[derive(Clone)]
pub struct CommutativeMonoid<T> {
    pub op: Rc<dyn Fn(T, T) -> T>,
    pub identity: Box<T>,
}

#[derive(Clone)]
pub struct Group<T> {
    pub op: Rc<dyn Fn(T, T) -> T>,
    pub identity: Box<T>,
    pub inverse: Rc<dyn Fn(T) -> T>,
}

#[derive(Clone)]
pub struct AbelianGroup<T> {
    pub op: Rc<dyn Fn(T, T) -> T>,
    pub identity: Box<T>,
    pub inverse: Rc<dyn Fn(T) -> T>,
}

#[derive(Clone)]
pub struct Semiring<T> {
    pub add: Rc<dyn Fn(T, T) -> T>,
    pub zero: Box<T>,
    pub mul: Rc<dyn Fn(T, T) -> T>,
    pub one: Box<T>,
}

#[derive(Clone)]
pub struct CommutativeSemiring<T> {
    pub add: Rc<dyn Fn(T, T) -> T>,
    pub zero: Box<T>,
    pub mul: Rc<dyn Fn(T, T) -> T>,
    pub one: Box<T>,
}

#[derive(Clone)]
pub struct Ring<T> {
    pub add: Rc<dyn Fn(T, T) -> T>,
    pub zero: Box<T>,
    pub negate: Rc<dyn Fn(T) -> T>,
    pub mul: Rc<dyn Fn(T, T) -> T>,
    pub one: Box<T>,
}

#[derive(Clone)]
pub struct OrderedRing<T> {
    pub add: Rc<dyn Fn(T, T) -> T>,
    pub sub: Rc<dyn Fn(T, T) -> T>,
    pub zero: Box<T>,
    pub negate: Rc<dyn Fn(T) -> T>,
    pub mul: Rc<dyn Fn(T, T) -> T>,
    pub div: Rc<dyn Fn(T, T) -> Rc<Result<T, DivError>>>,
    pub one: Box<T>,
    pub compare: Rc<dyn Fn(T, T) -> Ordering>,
    pub eq: Rc<dyn Fn(T, T) -> bool>,
    pub ne: Rc<dyn Fn(T, T) -> bool>,
    pub lt: Rc<dyn Fn(T, T) -> bool>,
    pub le: Rc<dyn Fn(T, T) -> bool>,
    pub gt: Rc<dyn Fn(T, T) -> bool>,
    pub ge: Rc<dyn Fn(T, T) -> bool>,
}

#[derive(Clone)]
pub struct Field<T> {
    pub add: Rc<dyn Fn(T, T) -> T>,
    pub zero: Box<T>,
    pub negate: Rc<dyn Fn(T) -> T>,
    pub mul: Rc<dyn Fn(T, T) -> T>,
    pub one: Box<T>,
    pub reciprocal: Rc<dyn Fn(T) -> T>,
    pub compare: Rc<dyn Fn(T, T) -> Ordering>,
}

#[derive(Clone)]
pub struct Lattice<T> {
    pub meet: Rc<dyn Fn(T, T) -> T>,
    pub join: Rc<dyn Fn(T, T) -> T>,
}

#[derive(Clone)]
pub struct BoundedLattice<T> {
    pub meet: Rc<dyn Fn(T, T) -> T>,
    pub join: Rc<dyn Fn(T, T) -> T>,
    pub top: Box<T>,
    pub bottom: Box<T>,
}

#[derive(Clone)]
pub struct BooleanAlgebra<T> {
    pub meet: Rc<dyn Fn(T, T) -> T>,
    pub join: Rc<dyn Fn(T, T) -> T>,
    pub complement: Rc<dyn Fn(T) -> T>,
    pub top: Box<T>,
    pub bottom: Box<T>,
}

#[derive(Clone)]
pub struct FreeMonoid<T> {
    pub concat: Rc<dyn Fn(Rc<Vec<T>>, Rc<Vec<T>>) -> Rc<Vec<T>>>,
    pub empty: Rc<Vec<T>>,
    pub append: Rc<dyn Fn(T) -> Rc<Vec<T>>>,
    pub slice: Rc<dyn Fn(i64, i64) -> Rc<Vec<T>>>,
    pub length: Rc<dyn Fn() -> i64>,
    pub is_empty: Rc<dyn Fn() -> bool>,
    pub count: Rc<dyn Fn() -> i64>,
    pub first: Rc<dyn Fn() -> Option<T>>,
    pub last: Rc<dyn Fn() -> Option<T>>,
    pub map: Rc<dyn Fn(Rc<dyn Fn(T) -> T>) -> Rc<Vec<T>>>,
    pub filter: Rc<dyn Fn(Rc<dyn Fn(T) -> bool>) -> Rc<Vec<T>>>,
    pub fold: Rc<dyn Fn(T, Rc<dyn Fn(T, T) -> T>) -> T>,
    pub flat_map: Rc<dyn Fn(Rc<dyn Fn(T) -> Rc<Vec<T>>>) -> Rc<Vec<T>>>,
    pub any: Rc<dyn Fn(Rc<dyn Fn(T) -> bool>) -> bool>,
    pub all: Rc<dyn Fn(Rc<dyn Fn(T) -> bool>) -> bool>,
    pub enumerate: Rc<dyn Fn() -> Rc<Vec<(i64, T)>>>,
    pub reverse: Rc<dyn Fn() -> Rc<Vec<T>>>,
    pub skip: Rc<dyn Fn(i64) -> Rc<Vec<T>>>,
    pub take: Rc<dyn Fn(i64) -> Rc<Vec<T>>>,
    pub sort_by: Rc<dyn Fn(Rc<dyn Fn(T, T) -> i64>) -> Rc<Vec<T>>>,
    pub contains: Rc<dyn Fn(T) -> bool>,
}

#[derive(Clone)]
pub struct PartialFunction<K, V> {
    pub lookup: Rc<dyn Fn(K) -> Witness<V>>,
    pub empty: Rc<PartialFunction<K, V>>,
    pub get: Rc<dyn Fn(K) -> Option<V>>,
    pub insert: Rc<dyn Fn(K, V) -> Rc<PartialFunction<K, V>>>,
    pub merge: Rc<dyn Fn(Rc<PartialFunction<K, V>>) -> Rc<PartialFunction<K, V>>>,
    pub keys: Rc<dyn Fn() -> Rc<FreeMonoid<K>>>,
    pub values: Rc<dyn Fn() -> Rc<FreeMonoid<V>>>,
    pub has: Rc<dyn Fn(K) -> bool>,
    pub contains_key: Rc<dyn Fn(K) -> bool>,
    pub size: Rc<dyn Fn() -> i64>,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(tag = "_variant")]
pub enum Ordering {
    Less,
    Equal,
    Greater,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(tag = "_variant")]
pub enum AlgebraProfile {
    OrderedRingProfile,
    ApproximateFieldProfile,
    BooleanAlgebraProfile,
    BooleanAlgebraCollectionProfile,
    FreeMonoidScalarProfile,
    FreeMonoidCollectionProfile,
    PartialFunctionProfile,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum ContainerSource {
    SameAsReceiver,
    Named { name: String },
}
impl ContainerSource {
    pub fn name(&self) -> String {
        match self {
            ContainerSource::SameAsReceiver => panic!("no name on unit variant"),
            ContainerSource::Named { name: __val, .. } => __val.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum AlgebraTypeTemplate {
    ReceiverSelf,
    ReceiverElement,
    ReceiverKey,
    ReceiverValue,
    NamedTemplate {
        name: String,
    },
    ContainerOf {
        source: Rc<ContainerSource>,
        element: Rc<AlgebraTypeTemplate>,
    },
    OptionalOf {
        inner: Rc<AlgebraTypeTemplate>,
    },
    WitnessOf {
        inner: Rc<AlgebraTypeTemplate>,
    },
    TupleOf {
        first: Rc<AlgebraTypeTemplate>,
        second: Rc<AlgebraTypeTemplate>,
    },
    CallableOf {
        params: Rc<Vec<Rc<AlgebraTypeTemplate>>>,
        return_type: Rc<AlgebraTypeTemplate>,
    },
    AlgebraTypeVariable {
        id: String,
    },
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(tag = "_variant")]
pub enum CollectionSizeEffect {
    ShrinkEffect,
    ProjectionEffect,
    IdentityEffect,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(tag = "_variant")]
pub enum CostShape {
    ShapeConstant,
    ShapeLinearScan,
    ShapeIterateBody,
    ShapeSortBody,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AlgebraFieldTemplate {
    pub name: String,
    pub param_types: Rc<Vec<Rc<AlgebraTypeTemplate>>>,
    pub return_type: Rc<AlgebraTypeTemplate>,
    pub size_effect: Option<CollectionSizeEffect>,
    pub cost_shape: Option<CostShape>,
    pub callback_element_position: Option<i64>,
}

pub fn kernel_algebra_profile() -> Rc<HashMap<String, AlgebraProfile>> {
    thread_local! {
        static CACHED: Rc<HashMap<String, AlgebraProfile>> = {
            let mut __m = HashMap::new();
            __m.insert("Int".to_string(), AlgebraProfile::OrderedRingProfile);
            __m.insert("Float".to_string(), AlgebraProfile::ApproximateFieldProfile);
            __m.insert("Bool".to_string(), AlgebraProfile::BooleanAlgebraProfile);
            __m.insert("String".to_string(), AlgebraProfile::FreeMonoidScalarProfile);
            __m.insert("List".to_string(), AlgebraProfile::FreeMonoidCollectionProfile);
            __m.insert("Set".to_string(), AlgebraProfile::BooleanAlgebraCollectionProfile);
            __m.insert("Map".to_string(), AlgebraProfile::PartialFunctionProfile);
            Rc::new(__m)
        };
    }
    CACHED.with(|c: &Rc<HashMap<String, AlgebraProfile>>| c.clone())
}

pub fn ordered_ring_templates() -> Rc<Vec<Rc<AlgebraFieldTemplate>>> {
    Rc::new(vec![
        Rc::new(AlgebraFieldTemplate {
            name: "add".to_string(),
            param_types: Rc::new(vec![
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            ]),
            return_type: Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            size_effect: None,
            cost_shape: None,
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "zero".to_string(),
            param_types: Rc::new(vec![]),
            return_type: Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            size_effect: None,
            cost_shape: None,
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "negate".to_string(),
            param_types: Rc::new(vec![Rc::new(AlgebraTypeTemplate::ReceiverSelf)]),
            return_type: Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            size_effect: None,
            cost_shape: None,
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "mul".to_string(),
            param_types: Rc::new(vec![
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            ]),
            return_type: Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            size_effect: None,
            cost_shape: None,
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "one".to_string(),
            param_types: Rc::new(vec![]),
            return_type: Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            size_effect: None,
            cost_shape: None,
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "compare".to_string(),
            param_types: Rc::new(vec![
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            ]),
            return_type: Rc::new(AlgebraTypeTemplate::NamedTemplate {
                name: "Ordering".to_string(),
            }),
            size_effect: None,
            cost_shape: None,
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "clamp".to_string(),
            param_types: Rc::new(vec![
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            ]),
            return_type: Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            size_effect: None,
            cost_shape: Some(CostShape::ShapeConstant),
            callback_element_position: None,
        }),
    ])
}

pub fn approximate_field_templates() -> Rc<Vec<Rc<AlgebraFieldTemplate>>> {
    Rc::new(vec![
        Rc::new(AlgebraFieldTemplate {
            name: "add".to_string(),
            param_types: Rc::new(vec![
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            ]),
            return_type: Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            size_effect: None,
            cost_shape: None,
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "zero".to_string(),
            param_types: Rc::new(vec![]),
            return_type: Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            size_effect: None,
            cost_shape: None,
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "negate".to_string(),
            param_types: Rc::new(vec![Rc::new(AlgebraTypeTemplate::ReceiverSelf)]),
            return_type: Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            size_effect: None,
            cost_shape: None,
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "mul".to_string(),
            param_types: Rc::new(vec![
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            ]),
            return_type: Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            size_effect: None,
            cost_shape: None,
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "one".to_string(),
            param_types: Rc::new(vec![]),
            return_type: Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            size_effect: None,
            cost_shape: None,
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "reciprocal".to_string(),
            param_types: Rc::new(vec![Rc::new(AlgebraTypeTemplate::ReceiverSelf)]),
            return_type: Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            size_effect: None,
            cost_shape: None,
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "compare".to_string(),
            param_types: Rc::new(vec![
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            ]),
            return_type: Rc::new(AlgebraTypeTemplate::NamedTemplate {
                name: "Ordering".to_string(),
            }),
            size_effect: None,
            cost_shape: None,
            callback_element_position: None,
        }),
    ])
}

pub fn boolean_algebra_templates() -> Rc<Vec<Rc<AlgebraFieldTemplate>>> {
    Rc::new(vec![
        Rc::new(AlgebraFieldTemplate {
            name: "meet".to_string(),
            param_types: Rc::new(vec![
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            ]),
            return_type: Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            size_effect: None,
            cost_shape: None,
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "join".to_string(),
            param_types: Rc::new(vec![
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            ]),
            return_type: Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            size_effect: None,
            cost_shape: None,
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "complement".to_string(),
            param_types: Rc::new(vec![Rc::new(AlgebraTypeTemplate::ReceiverSelf)]),
            return_type: Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            size_effect: None,
            cost_shape: None,
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "top".to_string(),
            param_types: Rc::new(vec![]),
            return_type: Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            size_effect: None,
            cost_shape: None,
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "bottom".to_string(),
            param_types: Rc::new(vec![]),
            return_type: Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            size_effect: None,
            cost_shape: None,
            callback_element_position: None,
        }),
    ])
}

pub fn boolean_algebra_collection_templates() -> Rc<Vec<Rc<AlgebraFieldTemplate>>> {
    Rc::new(vec![
        Rc::new(AlgebraFieldTemplate {
            name: "union".to_string(),
            param_types: Rc::new(vec![
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            ]),
            return_type: Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            size_effect: None,
            cost_shape: None,
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "intersect".to_string(),
            param_types: Rc::new(vec![
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            ]),
            return_type: Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            size_effect: None,
            cost_shape: None,
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "diff".to_string(),
            param_types: Rc::new(vec![
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            ]),
            return_type: Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            size_effect: None,
            cost_shape: None,
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "member".to_string(),
            param_types: Rc::new(vec![
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
                Rc::new(AlgebraTypeTemplate::ReceiverElement),
            ]),
            return_type: Rc::new(AlgebraTypeTemplate::NamedTemplate {
                name: "Bool".to_string(),
            }),
            size_effect: None,
            cost_shape: None,
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "contains".to_string(),
            param_types: Rc::new(vec![
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
                Rc::new(AlgebraTypeTemplate::ReceiverElement),
            ]),
            return_type: Rc::new(AlgebraTypeTemplate::NamedTemplate {
                name: "Bool".to_string(),
            }),
            size_effect: None,
            cost_shape: None,
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "filter".to_string(),
            param_types: Rc::new(vec![
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
                Rc::new(AlgebraTypeTemplate::CallableOf {
                    params: Rc::new(vec![Rc::new(AlgebraTypeTemplate::ReceiverElement)]),
                    return_type: Rc::new(AlgebraTypeTemplate::NamedTemplate {
                        name: "Bool".to_string(),
                    }),
                }),
            ]),
            return_type: Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            size_effect: Some(CollectionSizeEffect::IdentityEffect),
            cost_shape: Some(CostShape::ShapeIterateBody),
            callback_element_position: Some(0),
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "map".to_string(),
            param_types: Rc::new(vec![
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
                Rc::new(AlgebraTypeTemplate::CallableOf {
                    params: Rc::new(vec![Rc::new(AlgebraTypeTemplate::ReceiverElement)]),
                    return_type: Rc::new(AlgebraTypeTemplate::AlgebraTypeVariable {
                        id: "MappedElement".to_string(),
                    }),
                }),
            ]),
            return_type: Rc::new(AlgebraTypeTemplate::ContainerOf {
                source: Rc::new(ContainerSource::SameAsReceiver),
                element: Rc::new(AlgebraTypeTemplate::AlgebraTypeVariable {
                    id: "MappedElement".to_string(),
                }),
            }),
            size_effect: None,
            cost_shape: Some(CostShape::ShapeIterateBody),
            callback_element_position: Some(0),
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "flat_map".to_string(),
            param_types: Rc::new(vec![
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
                Rc::new(AlgebraTypeTemplate::CallableOf {
                    params: Rc::new(vec![Rc::new(AlgebraTypeTemplate::ReceiverElement)]),
                    return_type: Rc::new(AlgebraTypeTemplate::ContainerOf {
                        source: Rc::new(ContainerSource::SameAsReceiver),
                        element: Rc::new(AlgebraTypeTemplate::AlgebraTypeVariable {
                            id: "MappedElement".to_string(),
                        }),
                    }),
                }),
            ]),
            return_type: Rc::new(AlgebraTypeTemplate::ContainerOf {
                source: Rc::new(ContainerSource::SameAsReceiver),
                element: Rc::new(AlgebraTypeTemplate::AlgebraTypeVariable {
                    id: "MappedElement".to_string(),
                }),
            }),
            size_effect: None,
            cost_shape: Some(CostShape::ShapeIterateBody),
            callback_element_position: Some(0),
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "fold".to_string(),
            param_types: Rc::new(vec![
                Rc::new(AlgebraTypeTemplate::AlgebraTypeVariable {
                    id: "FoldAccumulator".to_string(),
                }),
                Rc::new(AlgebraTypeTemplate::CallableOf {
                    params: Rc::new(vec![
                        Rc::new(AlgebraTypeTemplate::AlgebraTypeVariable {
                            id: "FoldAccumulator".to_string(),
                        }),
                        Rc::new(AlgebraTypeTemplate::ReceiverElement),
                    ]),
                    return_type: Rc::new(AlgebraTypeTemplate::AlgebraTypeVariable {
                        id: "FoldAccumulator".to_string(),
                    }),
                }),
            ]),
            return_type: Rc::new(AlgebraTypeTemplate::AlgebraTypeVariable {
                id: "FoldAccumulator".to_string(),
            }),
            size_effect: None,
            cost_shape: Some(CostShape::ShapeIterateBody),
            callback_element_position: Some(1),
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "any".to_string(),
            param_types: Rc::new(vec![
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
                Rc::new(AlgebraTypeTemplate::CallableOf {
                    params: Rc::new(vec![Rc::new(AlgebraTypeTemplate::ReceiverElement)]),
                    return_type: Rc::new(AlgebraTypeTemplate::NamedTemplate {
                        name: "Bool".to_string(),
                    }),
                }),
            ]),
            return_type: Rc::new(AlgebraTypeTemplate::NamedTemplate {
                name: "Bool".to_string(),
            }),
            size_effect: None,
            cost_shape: Some(CostShape::ShapeIterateBody),
            callback_element_position: Some(0),
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "all".to_string(),
            param_types: Rc::new(vec![
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
                Rc::new(AlgebraTypeTemplate::CallableOf {
                    params: Rc::new(vec![Rc::new(AlgebraTypeTemplate::ReceiverElement)]),
                    return_type: Rc::new(AlgebraTypeTemplate::NamedTemplate {
                        name: "Bool".to_string(),
                    }),
                }),
            ]),
            return_type: Rc::new(AlgebraTypeTemplate::NamedTemplate {
                name: "Bool".to_string(),
            }),
            size_effect: None,
            cost_shape: Some(CostShape::ShapeIterateBody),
            callback_element_position: Some(0),
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "count".to_string(),
            param_types: Rc::new(vec![Rc::new(AlgebraTypeTemplate::ReceiverSelf)]),
            return_type: Rc::new(AlgebraTypeTemplate::NamedTemplate {
                name: "Int".to_string(),
            }),
            size_effect: None,
            cost_shape: Some(CostShape::ShapeLinearScan),
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "length".to_string(),
            param_types: Rc::new(vec![Rc::new(AlgebraTypeTemplate::ReceiverSelf)]),
            return_type: Rc::new(AlgebraTypeTemplate::NamedTemplate {
                name: "Int".to_string(),
            }),
            size_effect: None,
            cost_shape: Some(CostShape::ShapeLinearScan),
            callback_element_position: None,
        }),
    ])
}

pub fn free_monoid_scalar_templates() -> Rc<Vec<Rc<AlgebraFieldTemplate>>> {
    Rc::new(vec![
        Rc::new(AlgebraFieldTemplate {
            name: "concat".to_string(),
            param_types: Rc::new(vec![
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            ]),
            return_type: Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            size_effect: None,
            cost_shape: None,
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "empty".to_string(),
            param_types: Rc::new(vec![]),
            return_type: Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            size_effect: None,
            cost_shape: None,
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "length".to_string(),
            param_types: Rc::new(vec![Rc::new(AlgebraTypeTemplate::ReceiverSelf)]),
            return_type: Rc::new(AlgebraTypeTemplate::NamedTemplate {
                name: "Int".to_string(),
            }),
            size_effect: None,
            cost_shape: None,
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "is_empty".to_string(),
            param_types: Rc::new(vec![Rc::new(AlgebraTypeTemplate::ReceiverSelf)]),
            return_type: Rc::new(AlgebraTypeTemplate::NamedTemplate {
                name: "Bool".to_string(),
            }),
            size_effect: None,
            cost_shape: None,
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "chars".to_string(),
            param_types: Rc::new(vec![Rc::new(AlgebraTypeTemplate::ReceiverSelf)]),
            return_type: Rc::new(AlgebraTypeTemplate::ContainerOf {
                source: Rc::new(ContainerSource::Named {
                    name: "List".to_string(),
                }),
                element: Rc::new(AlgebraTypeTemplate::NamedTemplate {
                    name: "Int".to_string(),
                }),
            }),
            size_effect: None,
            cost_shape: Some(CostShape::ShapeLinearScan),
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "split".to_string(),
            param_types: Rc::new(vec![
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            ]),
            return_type: Rc::new(AlgebraTypeTemplate::ContainerOf {
                source: Rc::new(ContainerSource::Named {
                    name: "List".to_string(),
                }),
                element: Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            }),
            size_effect: None,
            cost_shape: Some(CostShape::ShapeLinearScan),
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "join".to_string(),
            param_types: Rc::new(vec![
                Rc::new(AlgebraTypeTemplate::ContainerOf {
                    source: Rc::new(ContainerSource::Named {
                        name: "List".to_string(),
                    }),
                    element: Rc::new(AlgebraTypeTemplate::ReceiverSelf),
                }),
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            ]),
            return_type: Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            size_effect: None,
            cost_shape: None,
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "contains".to_string(),
            param_types: Rc::new(vec![
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            ]),
            return_type: Rc::new(AlgebraTypeTemplate::NamedTemplate {
                name: "Bool".to_string(),
            }),
            size_effect: None,
            cost_shape: None,
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "starts_with".to_string(),
            param_types: Rc::new(vec![
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            ]),
            return_type: Rc::new(AlgebraTypeTemplate::NamedTemplate {
                name: "Bool".to_string(),
            }),
            size_effect: None,
            cost_shape: None,
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "ends_with".to_string(),
            param_types: Rc::new(vec![
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            ]),
            return_type: Rc::new(AlgebraTypeTemplate::NamedTemplate {
                name: "Bool".to_string(),
            }),
            size_effect: None,
            cost_shape: None,
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "trim".to_string(),
            param_types: Rc::new(vec![Rc::new(AlgebraTypeTemplate::ReceiverSelf)]),
            return_type: Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            size_effect: None,
            cost_shape: None,
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "to_lower".to_string(),
            param_types: Rc::new(vec![Rc::new(AlgebraTypeTemplate::ReceiverSelf)]),
            return_type: Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            size_effect: None,
            cost_shape: None,
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "to_upper".to_string(),
            param_types: Rc::new(vec![Rc::new(AlgebraTypeTemplate::ReceiverSelf)]),
            return_type: Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            size_effect: None,
            cost_shape: None,
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "replace".to_string(),
            param_types: Rc::new(vec![
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            ]),
            return_type: Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            size_effect: None,
            cost_shape: None,
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "substring".to_string(),
            param_types: Rc::new(vec![
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
                Rc::new(AlgebraTypeTemplate::NamedTemplate {
                    name: "Int".to_string(),
                }),
                Rc::new(AlgebraTypeTemplate::NamedTemplate {
                    name: "Int".to_string(),
                }),
            ]),
            return_type: Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            size_effect: None,
            cost_shape: None,
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "to_int".to_string(),
            param_types: Rc::new(vec![Rc::new(AlgebraTypeTemplate::ReceiverSelf)]),
            return_type: Rc::new(AlgebraTypeTemplate::NamedTemplate {
                name: "Int".to_string(),
            }),
            size_effect: None,
            cost_shape: None,
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "to_string".to_string(),
            param_types: Rc::new(vec![Rc::new(AlgebraTypeTemplate::ReceiverSelf)]),
            return_type: Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            size_effect: None,
            cost_shape: None,
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "reverse".to_string(),
            param_types: Rc::new(vec![Rc::new(AlgebraTypeTemplate::ReceiverSelf)]),
            return_type: Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            size_effect: None,
            cost_shape: None,
            callback_element_position: None,
        }),
    ])
}

pub fn free_monoid_collection_templates() -> Rc<Vec<Rc<AlgebraFieldTemplate>>> {
    Rc::new(vec![
        Rc::new(AlgebraFieldTemplate {
            name: "map".to_string(),
            param_types: Rc::new(vec![
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
                Rc::new(AlgebraTypeTemplate::CallableOf {
                    params: Rc::new(vec![Rc::new(AlgebraTypeTemplate::ReceiverElement)]),
                    return_type: Rc::new(AlgebraTypeTemplate::AlgebraTypeVariable {
                        id: "MappedElement".to_string(),
                    }),
                }),
            ]),
            return_type: Rc::new(AlgebraTypeTemplate::ContainerOf {
                source: Rc::new(ContainerSource::SameAsReceiver),
                element: Rc::new(AlgebraTypeTemplate::AlgebraTypeVariable {
                    id: "MappedElement".to_string(),
                }),
            }),
            size_effect: None,
            cost_shape: Some(CostShape::ShapeIterateBody),
            callback_element_position: Some(0),
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "filter".to_string(),
            param_types: Rc::new(vec![
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
                Rc::new(AlgebraTypeTemplate::CallableOf {
                    params: Rc::new(vec![Rc::new(AlgebraTypeTemplate::ReceiverElement)]),
                    return_type: Rc::new(AlgebraTypeTemplate::NamedTemplate {
                        name: "Bool".to_string(),
                    }),
                }),
            ]),
            return_type: Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            size_effect: Some(CollectionSizeEffect::IdentityEffect),
            cost_shape: Some(CostShape::ShapeIterateBody),
            callback_element_position: Some(0),
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "flat_map".to_string(),
            param_types: Rc::new(vec![
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
                Rc::new(AlgebraTypeTemplate::CallableOf {
                    params: Rc::new(vec![Rc::new(AlgebraTypeTemplate::ReceiverElement)]),
                    return_type: Rc::new(AlgebraTypeTemplate::ContainerOf {
                        source: Rc::new(ContainerSource::SameAsReceiver),
                        element: Rc::new(AlgebraTypeTemplate::AlgebraTypeVariable {
                            id: "MappedElement".to_string(),
                        }),
                    }),
                }),
            ]),
            return_type: Rc::new(AlgebraTypeTemplate::ContainerOf {
                source: Rc::new(ContainerSource::SameAsReceiver),
                element: Rc::new(AlgebraTypeTemplate::AlgebraTypeVariable {
                    id: "MappedElement".to_string(),
                }),
            }),
            size_effect: None,
            cost_shape: Some(CostShape::ShapeIterateBody),
            callback_element_position: Some(0),
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "fold".to_string(),
            param_types: Rc::new(vec![
                Rc::new(AlgebraTypeTemplate::AlgebraTypeVariable {
                    id: "FoldAccumulator".to_string(),
                }),
                Rc::new(AlgebraTypeTemplate::CallableOf {
                    params: Rc::new(vec![
                        Rc::new(AlgebraTypeTemplate::AlgebraTypeVariable {
                            id: "FoldAccumulator".to_string(),
                        }),
                        Rc::new(AlgebraTypeTemplate::ReceiverElement),
                    ]),
                    return_type: Rc::new(AlgebraTypeTemplate::AlgebraTypeVariable {
                        id: "FoldAccumulator".to_string(),
                    }),
                }),
            ]),
            return_type: Rc::new(AlgebraTypeTemplate::AlgebraTypeVariable {
                id: "FoldAccumulator".to_string(),
            }),
            size_effect: None,
            cost_shape: Some(CostShape::ShapeIterateBody),
            callback_element_position: Some(1),
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "any".to_string(),
            param_types: Rc::new(vec![
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
                Rc::new(AlgebraTypeTemplate::CallableOf {
                    params: Rc::new(vec![Rc::new(AlgebraTypeTemplate::ReceiverElement)]),
                    return_type: Rc::new(AlgebraTypeTemplate::NamedTemplate {
                        name: "Bool".to_string(),
                    }),
                }),
            ]),
            return_type: Rc::new(AlgebraTypeTemplate::NamedTemplate {
                name: "Bool".to_string(),
            }),
            size_effect: None,
            cost_shape: Some(CostShape::ShapeIterateBody),
            callback_element_position: Some(0),
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "all".to_string(),
            param_types: Rc::new(vec![
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
                Rc::new(AlgebraTypeTemplate::CallableOf {
                    params: Rc::new(vec![Rc::new(AlgebraTypeTemplate::ReceiverElement)]),
                    return_type: Rc::new(AlgebraTypeTemplate::NamedTemplate {
                        name: "Bool".to_string(),
                    }),
                }),
            ]),
            return_type: Rc::new(AlgebraTypeTemplate::NamedTemplate {
                name: "Bool".to_string(),
            }),
            size_effect: None,
            cost_shape: Some(CostShape::ShapeIterateBody),
            callback_element_position: Some(0),
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "count".to_string(),
            param_types: Rc::new(vec![Rc::new(AlgebraTypeTemplate::ReceiverSelf)]),
            return_type: Rc::new(AlgebraTypeTemplate::NamedTemplate {
                name: "Int".to_string(),
            }),
            size_effect: None,
            cost_shape: Some(CostShape::ShapeLinearScan),
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "first".to_string(),
            param_types: Rc::new(vec![Rc::new(AlgebraTypeTemplate::ReceiverSelf)]),
            return_type: Rc::new(AlgebraTypeTemplate::OptionalOf {
                inner: Rc::new(AlgebraTypeTemplate::ReceiverElement),
            }),
            size_effect: Some(CollectionSizeEffect::ProjectionEffect),
            cost_shape: Some(CostShape::ShapeLinearScan),
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "last".to_string(),
            param_types: Rc::new(vec![Rc::new(AlgebraTypeTemplate::ReceiverSelf)]),
            return_type: Rc::new(AlgebraTypeTemplate::OptionalOf {
                inner: Rc::new(AlgebraTypeTemplate::ReceiverElement),
            }),
            size_effect: Some(CollectionSizeEffect::ProjectionEffect),
            cost_shape: Some(CostShape::ShapeLinearScan),
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "skip".to_string(),
            param_types: Rc::new(vec![
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
                Rc::new(AlgebraTypeTemplate::NamedTemplate {
                    name: "Int".to_string(),
                }),
            ]),
            return_type: Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            size_effect: Some(CollectionSizeEffect::ShrinkEffect),
            cost_shape: Some(CostShape::ShapeIterateBody),
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "take".to_string(),
            param_types: Rc::new(vec![
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
                Rc::new(AlgebraTypeTemplate::NamedTemplate {
                    name: "Int".to_string(),
                }),
            ]),
            return_type: Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            size_effect: None,
            cost_shape: Some(CostShape::ShapeIterateBody),
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "sort_by".to_string(),
            param_types: Rc::new(vec![Rc::new(AlgebraTypeTemplate::ReceiverSelf)]),
            return_type: Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            size_effect: Some(CollectionSizeEffect::IdentityEffect),
            cost_shape: Some(CostShape::ShapeSortBody),
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "append".to_string(),
            param_types: Rc::new(vec![
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
                Rc::new(AlgebraTypeTemplate::ReceiverElement),
            ]),
            return_type: Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            size_effect: None,
            cost_shape: Some(CostShape::ShapeConstant),
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "contains".to_string(),
            param_types: Rc::new(vec![
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
                Rc::new(AlgebraTypeTemplate::ReceiverElement),
            ]),
            return_type: Rc::new(AlgebraTypeTemplate::NamedTemplate {
                name: "Bool".to_string(),
            }),
            size_effect: None,
            cost_shape: None,
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "enumerate".to_string(),
            param_types: Rc::new(vec![Rc::new(AlgebraTypeTemplate::ReceiverSelf)]),
            return_type: Rc::new(AlgebraTypeTemplate::ContainerOf {
                source: Rc::new(ContainerSource::SameAsReceiver),
                element: Rc::new(AlgebraTypeTemplate::TupleOf {
                    first: Rc::new(AlgebraTypeTemplate::NamedTemplate {
                        name: "Int".to_string(),
                    }),
                    second: Rc::new(AlgebraTypeTemplate::ReceiverElement),
                }),
            }),
            size_effect: Some(CollectionSizeEffect::IdentityEffect),
            cost_shape: Some(CostShape::ShapeIterateBody),
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "reverse".to_string(),
            param_types: Rc::new(vec![Rc::new(AlgebraTypeTemplate::ReceiverSelf)]),
            return_type: Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            size_effect: Some(CollectionSizeEffect::IdentityEffect),
            cost_shape: Some(CostShape::ShapeLinearScan),
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "join".to_string(),
            param_types: Rc::new(vec![
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
                Rc::new(AlgebraTypeTemplate::NamedTemplate {
                    name: "String".to_string(),
                }),
            ]),
            return_type: Rc::new(AlgebraTypeTemplate::NamedTemplate {
                name: "String".to_string(),
            }),
            size_effect: None,
            cost_shape: Some(CostShape::ShapeLinearScan),
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "concat".to_string(),
            param_types: Rc::new(vec![
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            ]),
            return_type: Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            size_effect: None,
            cost_shape: Some(CostShape::ShapeLinearScan),
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "list_push".to_string(),
            param_types: Rc::new(vec![
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
                Rc::new(AlgebraTypeTemplate::ReceiverElement),
            ]),
            return_type: Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            size_effect: None,
            cost_shape: None,
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "length".to_string(),
            param_types: Rc::new(vec![Rc::new(AlgebraTypeTemplate::ReceiverSelf)]),
            return_type: Rc::new(AlgebraTypeTemplate::NamedTemplate {
                name: "Int".to_string(),
            }),
            size_effect: None,
            cost_shape: Some(CostShape::ShapeLinearScan),
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "is_empty".to_string(),
            param_types: Rc::new(vec![Rc::new(AlgebraTypeTemplate::ReceiverSelf)]),
            return_type: Rc::new(AlgebraTypeTemplate::NamedTemplate {
                name: "Bool".to_string(),
            }),
            size_effect: None,
            cost_shape: Some(CostShape::ShapeLinearScan),
            callback_element_position: None,
        }),
    ])
}

pub fn partial_function_templates() -> Rc<Vec<Rc<AlgebraFieldTemplate>>> {
    Rc::new(vec![
        Rc::new(AlgebraFieldTemplate {
            name: "get".to_string(),
            param_types: Rc::new(vec![
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
                Rc::new(AlgebraTypeTemplate::ReceiverKey),
            ]),
            return_type: Rc::new(AlgebraTypeTemplate::OptionalOf {
                inner: Rc::new(AlgebraTypeTemplate::ReceiverValue),
            }),
            size_effect: None,
            cost_shape: None,
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "map_get".to_string(),
            param_types: Rc::new(vec![
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
                Rc::new(AlgebraTypeTemplate::ReceiverKey),
            ]),
            return_type: Rc::new(AlgebraTypeTemplate::OptionalOf {
                inner: Rc::new(AlgebraTypeTemplate::ReceiverValue),
            }),
            size_effect: None,
            cost_shape: None,
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "lookup".to_string(),
            param_types: Rc::new(vec![
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
                Rc::new(AlgebraTypeTemplate::ReceiverKey),
            ]),
            return_type: Rc::new(AlgebraTypeTemplate::WitnessOf {
                inner: Rc::new(AlgebraTypeTemplate::ReceiverValue),
            }),
            size_effect: None,
            cost_shape: None,
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "map_insert".to_string(),
            param_types: Rc::new(vec![
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
                Rc::new(AlgebraTypeTemplate::ReceiverKey),
                Rc::new(AlgebraTypeTemplate::ReceiverValue),
            ]),
            return_type: Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            size_effect: None,
            cost_shape: None,
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "map_merge".to_string(),
            param_types: Rc::new(vec![
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            ]),
            return_type: Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            size_effect: None,
            cost_shape: None,
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "has".to_string(),
            param_types: Rc::new(vec![
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
                Rc::new(AlgebraTypeTemplate::ReceiverKey),
            ]),
            return_type: Rc::new(AlgebraTypeTemplate::NamedTemplate {
                name: "Bool".to_string(),
            }),
            size_effect: None,
            cost_shape: None,
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "map_has".to_string(),
            param_types: Rc::new(vec![
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
                Rc::new(AlgebraTypeTemplate::ReceiverKey),
            ]),
            return_type: Rc::new(AlgebraTypeTemplate::NamedTemplate {
                name: "Bool".to_string(),
            }),
            size_effect: None,
            cost_shape: None,
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "map_contains_key".to_string(),
            param_types: Rc::new(vec![
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
                Rc::new(AlgebraTypeTemplate::ReceiverKey),
            ]),
            return_type: Rc::new(AlgebraTypeTemplate::NamedTemplate {
                name: "Bool".to_string(),
            }),
            size_effect: None,
            cost_shape: None,
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "keys".to_string(),
            param_types: Rc::new(vec![Rc::new(AlgebraTypeTemplate::ReceiverSelf)]),
            return_type: Rc::new(AlgebraTypeTemplate::ContainerOf {
                source: Rc::new(ContainerSource::Named {
                    name: "List".to_string(),
                }),
                element: Rc::new(AlgebraTypeTemplate::ReceiverKey),
            }),
            size_effect: None,
            cost_shape: None,
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "map_keys".to_string(),
            param_types: Rc::new(vec![Rc::new(AlgebraTypeTemplate::ReceiverSelf)]),
            return_type: Rc::new(AlgebraTypeTemplate::ContainerOf {
                source: Rc::new(ContainerSource::Named {
                    name: "List".to_string(),
                }),
                element: Rc::new(AlgebraTypeTemplate::ReceiverKey),
            }),
            size_effect: None,
            cost_shape: None,
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "values".to_string(),
            param_types: Rc::new(vec![Rc::new(AlgebraTypeTemplate::ReceiverSelf)]),
            return_type: Rc::new(AlgebraTypeTemplate::ContainerOf {
                source: Rc::new(ContainerSource::Named {
                    name: "List".to_string(),
                }),
                element: Rc::new(AlgebraTypeTemplate::ReceiverValue),
            }),
            size_effect: None,
            cost_shape: None,
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "map_values".to_string(),
            param_types: Rc::new(vec![Rc::new(AlgebraTypeTemplate::ReceiverSelf)]),
            return_type: Rc::new(AlgebraTypeTemplate::ContainerOf {
                source: Rc::new(ContainerSource::Named {
                    name: "List".to_string(),
                }),
                element: Rc::new(AlgebraTypeTemplate::ReceiverValue),
            }),
            size_effect: None,
            cost_shape: None,
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "with".to_string(),
            param_types: Rc::new(vec![
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
                Rc::new(AlgebraTypeTemplate::ReceiverKey),
                Rc::new(AlgebraTypeTemplate::ReceiverValue),
            ]),
            return_type: Rc::new(AlgebraTypeTemplate::ReceiverSelf),
            size_effect: None,
            cost_shape: None,
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "contains".to_string(),
            param_types: Rc::new(vec![
                Rc::new(AlgebraTypeTemplate::ReceiverSelf),
                Rc::new(AlgebraTypeTemplate::ReceiverKey),
            ]),
            return_type: Rc::new(AlgebraTypeTemplate::NamedTemplate {
                name: "Bool".to_string(),
            }),
            size_effect: None,
            cost_shape: None,
            callback_element_position: None,
        }),
        Rc::new(AlgebraFieldTemplate {
            name: "length".to_string(),
            param_types: Rc::new(vec![Rc::new(AlgebraTypeTemplate::ReceiverSelf)]),
            return_type: Rc::new(AlgebraTypeTemplate::NamedTemplate {
                name: "Int".to_string(),
            }),
            size_effect: None,
            cost_shape: None,
            callback_element_position: None,
        }),
    ])
}

pub fn algebra_templates_for_profile(profile: AlgebraProfile) -> Rc<Vec<Rc<AlgebraFieldTemplate>>> {
    match profile {
        AlgebraProfile::OrderedRingProfile => ordered_ring_templates(),
        AlgebraProfile::ApproximateFieldProfile => approximate_field_templates(),
        AlgebraProfile::BooleanAlgebraProfile => boolean_algebra_templates(),
        AlgebraProfile::BooleanAlgebraCollectionProfile => boolean_algebra_collection_templates(),
        AlgebraProfile::FreeMonoidScalarProfile => free_monoid_scalar_templates(),
        AlgebraProfile::FreeMonoidCollectionProfile => free_monoid_collection_templates(),
        AlgebraProfile::PartialFunctionProfile => partial_function_templates(),
    }
}

pub fn algebra_type_param_names(profile: AlgebraProfile) -> Rc<Vec<String>> {
    match profile {
        AlgebraProfile::OrderedRingProfile => Rc::new(vec![]),
        AlgebraProfile::ApproximateFieldProfile => Rc::new(vec![]),
        AlgebraProfile::BooleanAlgebraProfile => Rc::new(vec![]),
        AlgebraProfile::BooleanAlgebraCollectionProfile => Rc::new(vec!["T".to_string()]),
        AlgebraProfile::FreeMonoidScalarProfile => Rc::new(vec![]),
        AlgebraProfile::FreeMonoidCollectionProfile => Rc::new(vec!["T".to_string()]),
        AlgebraProfile::PartialFunctionProfile => Rc::new(vec!["K".to_string(), "V".to_string()]),
    }
}
