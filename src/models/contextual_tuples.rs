//! Contextual tuples model.

use serde::{Deserialize, Serialize};

use super::tuple::TupleKey;

/// A set of contextual tuples provided alongside a request for evaluation.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ContextualTupleKeys {
    /// The contextual tuple keys.
    pub tuple_keys: Vec<TupleKey>,
}

impl ContextualTupleKeys {
    /// Creates a `ContextualTupleKeys` from a list of tuple keys.
    pub fn new(tuple_keys: Vec<TupleKey>) -> Self {
        Self { tuple_keys }
    }
}
