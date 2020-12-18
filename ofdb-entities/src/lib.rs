//#![deny(missing_docs)] // TODO: Complete missing documentation and enable this option
#![deny(missing_debug_implementations)]
#![deny(broken_intra_doc_links)]
#![cfg_attr(test, deny(warnings))]

//! # ofdb-entities
//!
//! Reusable, agnostic domain entities for OpenFairDB.
//!
//! The entities only contain generic functionality that does not reveal any application-specific business logic.

pub mod activity;
pub mod address;
pub mod category;
pub mod clearance;
pub mod comment;
pub mod contact;
pub mod email;
pub mod event;
pub mod geo;
pub mod id;
pub mod links;
pub mod location;
pub mod nonce;
pub mod organization;
pub mod password;
pub mod place;
pub mod rating;
pub mod review;
pub mod revision;
pub mod subscription;
pub mod tag;
pub mod time;
pub mod user;
pub mod url {
    pub use url::{ParseError, Url};
}

#[cfg(any(test, feature = "builders"))]
pub mod builders;
