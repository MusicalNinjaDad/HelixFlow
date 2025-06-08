//! The actual HelixFlow implementation. This provides all the building blocks and functionality
//! needed for the app.

#![feature(assert_matches)]
#![feature(associated_type_defaults)]
#![feature(coverage_attribute)]
#![feature(try_trait_v2)]

use std::any::Any;

pub mod task;

/// Marker trait for our data items
pub trait HelixFlowItem
where
    // required for Mismatch Error (which uses `Box<dyn HelixFlowItem>`)
    Self: std::fmt::Debug + Send + Sync + 'static + Any,
{
    fn as_any(&self) -> &dyn Any;
}
