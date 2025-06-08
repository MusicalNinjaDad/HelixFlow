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

pub trait CRUD
where
    Self: Sized,
{
    fn create<B: Store<Self>>(&self, backend: &B) -> HelixFlowResult<()>;
    fn get<B: Store<Self>>(backend: &B, id: &Uuid) -> HelixFlowResult<Self>;
}

/// Methods to store and retrieve `ITEM` in a backend
pub trait Store<ITEM> {
    /// Create a new `ITEM` in the backend.
    ///
    /// The returned `ITEM` should be the actual stored record from the backend - to allow
    /// validation by `CRUD<ITEM>::create()`
    fn create(&self, item: &ITEM) -> HelixFlowResult<ITEM>;

    /// Get an `ITEM` from the backend
    fn get(&self, id: &Uuid) -> HelixFlowResult<ITEM>;
}

impl<ITEM> CRUD for ITEM
where
    ITEM: HelixFlowItem + PartialEq + Clone,
{
    /// Create this item in a given storage backend.
    fn create<B: Store<ITEM>>(&self, backend: &B) -> HelixFlowResult<()> {
        let created_item = backend.create(self)?;
        if &created_item == self {
            Ok(())
        } else {
            Err(HelixFlowError::Mismatch {
                expected: Box::new(self.clone()),
                actual: Box::new(created_item),
            })
        }
    }

    /// Get item from `backend` by `id`
    fn get<B: Store<ITEM>>(backend: &B, id: &Uuid) -> HelixFlowResult<ITEM> {
        backend.get(id)
    }
}

/// A valid usage of a relationship struct, defining acceptable types for left & right.
///
/// E.g. to allow `Contains`to be used for `TaskList -> Contains -> Task`:
/// ```ignore
/// impl Relationship for Contains<TaskList, Task> {
///    type Left = TaskList;
///    type Right = Task;
/// }
/// ```
// TODO: Add derive macro to generate Link, Linkable, Relate, Try & FromResidual for valid type pairings
// Can't do this in a blanket impl as it requires guarantees for fields `left`and `right`
pub trait Relationship
where
    Self: Sized,
{
    type Left: HelixFlowItem;
    type Right: HelixFlowItem;
}

/// `impl Link<REL> for LEFT` gives `Left Rel:(-> link_type -> Right)`
pub trait Link
where
    Self: Relationship,
{
    fn create_linked_item<B: Relate<Self>>(self, backend: &B) -> HelixFlowResult<()>;
}

pub trait Linkable<REL: Link> {
    fn link(&self, right: &REL::Right) -> REL;
    fn get_linked_items<B: Relate<REL>>(
        &self,
        backend: &B,
    ) -> HelixFlowResult<impl Iterator<Item = REL>>;
}

/// Methods to relate items in a backend
pub trait Relate<REL: Link> {
    /// Create and link the related item
    fn create_linked_item(&self, link: &REL) -> HelixFlowResult<REL>;
    fn get_linked_items(&self, left: &REL::Left) -> HelixFlowResult<impl Iterator<Item = REL>>;
}
