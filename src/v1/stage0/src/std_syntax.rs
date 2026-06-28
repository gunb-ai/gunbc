use self::AlgebraFieldKind::*;
use self::BinOp::*;
use self::BodyKind::*;
use self::ItemFormKind::*;
use self::LiteralValue::*;
use crate::v1_rt;
use crate::v1_rt::Witness;
use crate::v1_rt::Witness::{Holds, Violates};
use crate::NonEmptyBTreeSet;
use crate::NonEmptyVec;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(tag = "_variant")]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
    NullCoalesce,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum LiteralValue {
    LitStr { value: String },
    LitInt { value: i64 },
    LitFloat { value: String },
    LitBool { value: bool },
    LitNull,
    LitSymbol { value: String },
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(tag = "_variant")]
pub enum BodyKind {
    ExprBody,
    BlockBody,
    TypeBody,
    ValueBody,
    NoBody,
    ServiceBody,
    ResourceBody,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(tag = "_variant")]
pub enum ItemFormKind {
    FuncForm,
    StructForm,
    EnumForm,
    TypeAliasForm,
    ModuleForm,
    OtherForm,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ItemForm {
    pub kind: ItemFormKind,
    pub keyword: String,
    pub has_type_params: bool,
    pub has_params: bool,
    pub has_return_type: bool,
    pub return_required: bool,
    pub has_uses: bool,
    pub body_kind: BodyKind,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(tag = "_variant")]
pub enum AlgebraFieldKind {
    AlgAdd,
    AlgMul,
    AlgReciprocal,
    AlgQuotient,
    AlgRemainder,
    AlgCompare,
    AlgMeet,
    AlgJoin,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AlgebraFieldEntry {
    pub kind: AlgebraFieldKind,
    pub field_name: String,
}

pub fn algebra_field_entries() -> Rc<Vec<Rc<AlgebraFieldEntry>>> {
    thread_local! {
            static CACHED: Rc<Vec<Rc<AlgebraFieldEntry>>> = {
                Rc::new(vec![Rc::new(AlgebraFieldEntry {
        kind: AlgebraFieldKind::AlgAdd,
        field_name: "add".to_string(),
    }), Rc::new(AlgebraFieldEntry {
        kind: AlgebraFieldKind::AlgMul,
        field_name: "mul".to_string(),
    }), Rc::new(AlgebraFieldEntry {
        kind: AlgebraFieldKind::AlgReciprocal,
        field_name: "reciprocal".to_string(),
    }), Rc::new(AlgebraFieldEntry {
        kind: AlgebraFieldKind::AlgQuotient,
        field_name: "quotient".to_string(),
    }), Rc::new(AlgebraFieldEntry {
        kind: AlgebraFieldKind::AlgRemainder,
        field_name: "remainder".to_string(),
    }), Rc::new(AlgebraFieldEntry {
        kind: AlgebraFieldKind::AlgCompare,
        field_name: "compare".to_string(),
    }), Rc::new(AlgebraFieldEntry {
        kind: AlgebraFieldKind::AlgMeet,
        field_name: "meet".to_string(),
    }), Rc::new(AlgebraFieldEntry {
        kind: AlgebraFieldKind::AlgJoin,
        field_name: "join".to_string(),
    })])
            };
        }
    CACHED.with(|c: &Rc<Vec<Rc<AlgebraFieldEntry>>>| c.clone())
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OperatorSpec {
    pub symbol: String,
    pub left_bp: i64,
    pub right_bp: i64,
    pub binop: Option<BinOp>,
    pub algebra_field: Option<AlgebraFieldKind>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SyntaxSpec {
    pub item_forms: Rc<Vec<Rc<ItemForm>>>,
    pub operators: Rc<Vec<Rc<OperatorSpec>>>,
    pub keyword_literals: Rc<HashMap<String, Rc<LiteralValue>>>,
    pub keyword_set: Rc<HashMap<String, bool>>,
}
