pub use crate::std_coercion::TypeCheckpoint;
use crate::v1_rt;
use crate::v1_rt::Witness;
use crate::v1_rt::Witness::{Holds, Violates};
use crate::NonEmptyBTreeSet;
use crate::NonEmptyVec;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::rc::Rc;

pub fn dag_type_checkpoints() -> Rc<Vec<Rc<TypeCheckpoint>>> {
    thread_local! {
        static CACHED: Rc<Vec<Rc<TypeCheckpoint>>> = {
            serde_json::from_value(serde_json::json!([{"dag_name": "Int", "target_type": "Int", "default_expr": "0", "is_copy": null, "literal_suffix": null}, {"dag_name": "Float", "target_type": "Float", "default_expr": "0.0", "is_copy": null, "literal_suffix": null}, {"dag_name": "Bool", "target_type": "Bool", "default_expr": "false", "is_copy": null, "literal_suffix": null}, {"dag_name": "String", "target_type": "String", "default_expr": "\"\"", "is_copy": null, "literal_suffix": null}]))
                .expect("valid data definition")
        };
    }
    CACHED.with(|c: &Rc<Vec<Rc<TypeCheckpoint>>>| c.clone())
}
