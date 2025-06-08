//! The actual HelixFlow implementation. This provides all the building blocks and functionality
//! needed for the app.

#![feature(assert_matches)]
#![feature(associated_type_defaults)]
#![feature(coverage_attribute)]
#![feature(try_trait_v2)]

use std::any::Any;

use uuid::Uuid;

pub mod task;

/// Marker trait for our data items
pub trait HelixFlowItem
where
    // required for Mismatch Error (which uses `Box<dyn HelixFlowItem>`)
    Self: std::fmt::Debug + Send + Sync + 'static + Any,
{
    fn as_any(&self) -> &dyn Any;
}

#[derive(Debug, thiserror::Error)]
pub enum HelixFlowError {
    // The #[from] anyhow::Error will convert anything that offers `into anyhow::Error`.
    #[error("backend error: {0}")]
    BackendError(#[from] anyhow::Error),

    #[error("created item does not match expectations: expected {expected:?}, got {actual:?}")]
    Mismatch {
        expected: Box<dyn HelixFlowItem>,
        actual: Box<dyn HelixFlowItem>,
    },

    #[error("task id ({id:?}) is not a valid UUID v7")]
    InvalidID { id: String },

    #[error("404 No {itemtype} found with id {id}")]
    NotFound { itemtype: String, id: Uuid },

    #[error("Relationship between {left:?} and {right:?} contains Errors")]
    RelationshipBetweenErrors {
        left: Box<HelixFlowResult<Box<dyn HelixFlowItem>>>,
        right: Box<HelixFlowResult<Box<dyn HelixFlowItem>>>,
    },
}

pub type HelixFlowResult<T> = std::result::Result<T, HelixFlowError>;