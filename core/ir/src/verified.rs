//! Verified DAG wrapper — proof that structural checks have passed.
//!
//! `VerifiedDag<T>` is a newtype over `Dag<T>` that can only be constructed
//! by passing `verify_dag()`. This gates downstream stages (resolve, emit)
//! behind verification at the type level.

use crate::dag::Dag;
use crate::validate::{verify_dag, VerifyError};
use serde::Serialize;

/// A DAG that has passed all structural verification checks.
///
/// Cannot be constructed directly — use [`VerifiedDag::verify`].
/// Downstream stages (resolve, emit, derive) should require this type
/// to enforce verification at compile time.
#[derive(Debug, Clone)]
pub struct VerifiedDag<T>(Dag<T>);

impl<T: Serialize> Serialize for VerifiedDag<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

impl<T> VerifiedDag<T> {
    /// Verify a DAG and wrap it if all checks pass.
    pub fn verify(dag: Dag<T>) -> Result<Self, Vec<VerifyError>> {
        let errors = verify_dag(&dag);
        if errors.is_empty() {
            Ok(Self(dag))
        } else {
            Err(errors)
        }
    }

    /// Borrow the inner DAG.
    pub fn as_dag(&self) -> &Dag<T> {
        &self.0
    }

    /// Consume the wrapper, returning the inner DAG.
    pub fn into_inner(self) -> Dag<T> {
        self.0
    }
}

impl<T> std::ops::Deref for VerifiedDag<T> {
    type Target = Dag<T>;

    fn deref(&self) -> &Dag<T> {
        &self.0
    }
}
