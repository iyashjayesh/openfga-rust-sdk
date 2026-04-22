//! All OpenFGA data models.
//!
//! These types mirror the `model_*.go` files from the Go SDK and are
//! serialised/deserialised from the OpenFGA REST API JSON payloads.

mod authorization_model;
mod batch_check;
mod check;
mod consistency;
mod contextual_tuples;
mod error_codes;
mod expand;
mod list_objects;
mod list_users;
mod misc;
mod read;
mod store;
mod tuple;
mod write;

// assertion and userset re-export subsets of the above modules
mod assertion;
mod userset;

pub use authorization_model::*;
pub use batch_check::*;
pub use check::*;
pub use consistency::*;
pub use contextual_tuples::*;
pub use error_codes::*;
pub use expand::*;
pub use list_objects::*;
pub use list_users::*;
pub use misc::*;
pub use read::*;
pub use store::*;
pub use tuple::*;
pub use write::*;
