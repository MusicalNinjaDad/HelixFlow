//! The fundamental `Task` building block and related functions.

use std::{
    any::Any,
    borrow::Cow,
    ops::{ControlFlow, FromResidual, Try},
};

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use uuid::{Uuid, uuid};

/// Marker trait for our data items
pub trait HelixFlowItem
where
    // required for Mismatch Error (which uses `Box<dyn HelixFlowItem>`)
    Self: std::fmt::Debug + Send + Sync + 'static + Any,
{
    fn as_any(&self) -> &dyn Any;
}

impl HelixFlowItem for Task {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
impl HelixFlowItem for TaskList {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// A Task
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Task {
    // TODO check value of using a Cow here ... are we really saving when passing around - or would
    // it be better to have something that is `Copy`?
    pub name: Cow<'static, str>,
    pub id: Uuid,
    pub description: Option<Cow<'static, str>>,
}

impl Task {
    /// Create a new `Task` with valid `id`, suitable for usage as database key.
    ///
    /// `name` & `Some(description)` must be of the same type to avoid the need to specify the
    /// theoretical type of a `None` description.
    ///
    /// Even though `name` must be given, it may be an empty string `""` - semantically every
    /// Task has a name, even if this is blank, but not every Task has a description.
    pub fn new<S1>(name: S1, description: Option<S1>) -> Task
    where
        S1: Into<Cow<'static, str>>,
    {
        Task {
            name: name.into(),
            id: Uuid::now_v7(),
            description: description.map(|desc| desc.into()),
        }
    }
}

/// A list of tasks
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TaskList {
    pub name: Cow<'static, str>,
    pub id: Uuid,
}

impl TaskList {
    /// Create a new `TaskList` with valid `id`, suitable for usage as database key.
    pub fn new<S>(name: S) -> TaskList
    where
        S: Into<Cow<'static, str>>,
    {
        TaskList {
            name: name.into(),
            id: Uuid::now_v7(),
        }
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
// TODO: Add derive macro to generate Relationship, Try & FromResidual for valid type pairings
pub trait Relationship
where
    Self: Sized,
{
    type Left;
    type Right;
}

impl Relationship for Contains<TaskList, Task> {
    type Left = TaskList;
    type Right = Task;
}

#[derive(Debug)]
pub struct Contains<LEFT, RIGHT> {
    pub left: HelixFlowResult<LEFT>,
    pub sortorder: String,
    pub right: HelixFlowResult<RIGHT>,
}

impl<LEFT, RIGHT> Try for Contains<LEFT, RIGHT>
where
    LEFT: HelixFlowItem,
    RIGHT: HelixFlowItem,
{
    type Output = Self; // Continue
    type Residual = Self; // Break
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        if self.left.is_ok() && self.right.is_ok() {
            ControlFlow::Continue(self)
        } else {
            ControlFlow::Break(self)
        }
    }
    fn from_output(_output: Self::Output) -> Self {
        unimplemented!("Contains? should only be used in funtions returning a Result")
    }
}

impl<LEFT, RIGHT> FromResidual<Contains<LEFT, RIGHT>> for Contains<LEFT, RIGHT>
where
    LEFT: HelixFlowItem,
    RIGHT: HelixFlowItem,
{
    fn from_residual(_residual: Contains<LEFT, RIGHT>) -> Self {
        unimplemented!("Contains? should only be used in funtions returning a Result")
    }
}

impl<LEFT, RIGHT> FromResidual<Contains<LEFT, RIGHT>> for HelixFlowResult<()>
where
    LEFT: HelixFlowItem,
    RIGHT: HelixFlowItem,
{
    fn from_residual(residual: Contains<LEFT, RIGHT>) -> Self {
        Err(HelixFlowError::RelationshipBetweenErrors {
            left: match residual.left {
                Ok(item) => Box::new(Ok(Box::new(item))),
                Err(e) => Box::new(Err(e)),
            },
            right: match residual.right {
                Ok(item) => Box::new(Ok(Box::new(item))),
                Err(e) => Box::new(Err(e)),
            },
        })
    }
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

impl Link for Contains<TaskList, Task> {
    fn create_linked_item<B: Relate<Contains<TaskList, Task>>>(
        self,
        backend: &B,
    ) -> HelixFlowResult<()> {
        let valid_relationship = self?;
        let created = backend.create_linked_item(&valid_relationship)?;
        let _tasklist_ok = created.left?;
        let expected = valid_relationship.right?;
        match created.right {
            Ok(task) if task == expected => Ok(()),
            Ok(_) => Err(HelixFlowError::Mismatch {
                expected: Box::new(expected.clone()),
                actual: Box::new(created.right?.clone()),
            }),
            Err(e) => Err(e),
        }
    }
}

impl Linkable<Contains<TaskList, Task>> for TaskList {
    fn link(&self, task: &Task) -> Contains<TaskList, Task> {
        Contains {
            left: Ok(self.clone()),
            sortorder: "a".into(),
            right: Ok(task.clone()),
        }
    }
    fn get_linked_items<B>(
        &self,
        backend: &B,
    ) -> HelixFlowResult<impl Iterator<Item = Contains<TaskList, Task>>>
    where
        B: Relate<Contains<TaskList, Task>>,
    {
        backend.get_linked_items(self)
    }
}

#[derive(Clone, Copy)]
pub struct TestBackend;

impl Store<Task> for TestBackend {
    fn create(&self, task: &Task) -> HelixFlowResult<Task> {
        match task.name {
            Cow::Borrowed("FAIL") => Err(anyhow!("Failed to create task").into()),
            Cow::Borrowed("MISMATCH") => Ok(Task::new(task.name.clone(), task.description.clone())),
            _ => Ok(task.clone()),
        }
    }

    fn get(&self, id: &Uuid) -> HelixFlowResult<Task> {
        match id.to_string().as_str() {
            "0196b4c9-8447-7959-ae1f-72c7c8a3dd36" => Ok(Task {
                name: "Task 1".into(),
                id: *id,
                description: None,
            }),
            "0196ca5f-d934-7ec8-b042-ae37b94b8432" => Ok(Task {
                name: "Task 2".into(),
                id: *id,
                description: None,
            }),
            _ => Err(HelixFlowError::NotFound {
                itemtype: "Task".into(),
                id: *id,
            }),
        }
    }
}

impl Store<TaskList> for TestBackend {
    fn create(&self, _item: &TaskList) -> HelixFlowResult<TaskList> {
        todo!()
    }
    fn get(&self, id: &Uuid) -> HelixFlowResult<TaskList> {
        match id.to_string().as_str() {
            "0196fe23-7c01-7d6b-9e09-5968eb370549" => Ok(TaskList {
                name: "Test TaskList 1".into(),
                id: *id,
            }),
            _ => Err(HelixFlowError::NotFound {
                itemtype: "Tasklist".into(),
                id: *id,
            }),
        }
    }
}

impl Relate<Contains<TaskList, Task>> for TestBackend {
    fn create_linked_item(
        &self,
        link: &Contains<TaskList, Task>,
    ) -> HelixFlowResult<Contains<TaskList, Task>> {
        let tasklist = link.left.as_ref().unwrap().clone();
        match tasklist.id.to_string().as_str() {
            "0196fe23-7c01-7d6b-9e09-5968eb370549" => Ok(Contains {
                left: Ok(tasklist),
                sortorder: link.sortorder.clone(),
                right: self.create(link.right.as_ref().unwrap()),
            }),
            _ => Err(HelixFlowError::NotFound {
                itemtype: "Tasklist".into(),
                id: tasklist.id,
            }),
        }
    }
    fn get_linked_items(
        &self,
        left: &TaskList,
    ) -> HelixFlowResult<impl Iterator<Item = Contains<TaskList, Task>>> {
        match left.id.to_string().as_str() {
            "0196fe23-7c01-7d6b-9e09-5968eb370549" => {
                let tasks = vec![
                    Task {
                        name: "Task 1".into(),
                        id: uuid!("0196b4c9-8447-7959-ae1f-72c7c8a3dd36"),
                        description: None,
                    },
                    Task {
                        name: "Task 2".into(),
                        id: uuid!("0196ca5f-d934-7ec8-b042-ae37b94b8432"),
                        description: None,
                    },
                ];
                Ok(tasks.into_iter().map(|task| left.link(&task)))
            }
            _ => Err(HelixFlowError::NotFound {
                itemtype: "Tasklist".into(),
                id: left.id,
            }),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use std::assert_matches::assert_matches;
    use wasm_bindgen_test::*;

    use super::*;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[test]
    fn try_contains_oks() -> HelixFlowResult<()> {
        let tasklist = TaskList::new("tasklist");
        let task = Task::new("task", None);
        let contains = Contains {
            left: Ok(tasklist.clone()),
            sortorder: "a".into(),
            right: Ok(task.clone()),
        };
        let contains2 = Contains {
            left: Ok(tasklist.clone()),
            sortorder: "a".into(),
            right: Ok(task.clone()),
        };
        let contains = contains?;
        assert_eq!(contains.left.unwrap(), contains2.left.unwrap());
        assert_eq!(contains.right.unwrap(), contains2.right.unwrap());
        Ok(())
    }

    #[test]
    fn try_contains_err_left() {
        let task = Task::new("task", None);
        let contains: Contains<TaskList, Task> = Contains {
            left: Err(HelixFlowError::NotFound {
                itemtype: "Tasklist".into(),
                id: uuid!("0196b4c9-8447-7959-ae1f-72c7c8a3dd36"),
            }),
            sortorder: "try_contains_err_left".into(),
            right: Ok(task.clone()),
        };
        fn is_valid(relationship: Contains<TaskList, Task>) -> HelixFlowResult<()> {
            relationship?;
            Ok(())
        }
        let err = is_valid(contains).unwrap_err();
        assert_matches!(
            err,
            HelixFlowError::RelationshipBetweenErrors { left, right }
            if matches!(
                left.as_ref(),
                Err(boxed_err) if matches!(
                    boxed_err,
                    HelixFlowError::NotFound {itemtype, id}
                    if itemtype == "Tasklist" && id == &uuid!("0196b4c9-8447-7959-ae1f-72c7c8a3dd36")
                )
            ) && matches!(
                right.as_ref(),
                Ok(boxed_task) if boxed_task.as_any().downcast_ref::<Task>() == Some(&task)
            )
        )
    }

    #[wasm_bindgen_test(unsupported = test)]
    fn test_new_task() {
        let new_task = Task::new("Test Task", None);
        assert_eq!(new_task.name, "Test Task");
        assert!(new_task.description.is_none());
        assert!(!new_task.id.is_nil());
        assert_eq!(new_task.id.get_version(), Some(uuid::Version::SortRand));
    }

    #[wasm_bindgen_test(unsupported = test)]
    fn test_new_task_blank() {
        let new_task = Task::new("", None);
        assert_eq!(new_task.name, "");
        assert!(new_task.description.is_none());
        assert!(!new_task.id.is_nil());
        assert_eq!(new_task.id.get_version(), Some(uuid::Version::SortRand));
    }

    #[wasm_bindgen_test(unsupported = test)]
    fn test_create_task() {
        let new_task = Task::new("Test Task 1", None);
        let backend = TestBackend;
        new_task.create(&backend).unwrap();
    }

    #[wasm_bindgen_test(unsupported = test)]
    fn test_failed_to_create_task() {
        let new_task = Task::new("FAIL", None);
        let backend = TestBackend;
        let err = new_task.create(&backend).unwrap_err();
        assert_matches!(err, HelixFlowError::BackendError(_))
    }

    #[wasm_bindgen_test(unsupported = test)]
    fn test_mismatched_task_created() {
        let new_task = Task::new("MISMATCH", None);
        let backend = TestBackend;
        let err = new_task.create(&backend).unwrap_err();
        assert_matches!(
            err,
            HelixFlowError::Mismatch {
                expected: _,
                actual: _
            }
        )
    }

    #[wasm_bindgen_test(unsupported = test)]
    fn test_get_task() {
        let backend = TestBackend;
        let id = uuid!("0196b4c9-8447-7959-ae1f-72c7c8a3dd36");
        let task = Task::get(&backend, &id).unwrap();
        assert_eq!(
            task,
            Task {
                name: "Task 1".into(),
                id,
                description: None
            }
        )
    }

    #[wasm_bindgen_test(unsupported = test)]
    fn test_get_invalid_task() {
        let backend = TestBackend;
        let id = uuid!("0196b4c9-8447-78db-ae8a-be68a8095aa2");
        let err = Task::get(&backend, &id).unwrap_err();
        assert_eq!(
            format!("{}", &err),
            "404 No Task found with id 0196b4c9-8447-78db-ae8a-be68a8095aa2"
        );
        assert_matches!(
                err,
                HelixFlowError::NotFound { itemtype, id }
                if itemtype == "Task" && id == uuid!("0196b4c9-8447-78db-ae8a-be68a8095aa2"));
    }

    #[test]
    fn get_tasks_in_tasklist() {
        let backend = TestBackend;
        let backlog = TaskList {
            name: "Backlog".into(),
            id: uuid!("0196fe23-7c01-7d6b-9e09-5968eb370549"),
        };
        let task1 = Task {
            name: "Task 1".into(),
            id: uuid!("0196b4c9-8447-7959-ae1f-72c7c8a3dd36"),
            description: None,
        };
        let task2 = Task {
            name: "Task 2".into(),
            id: uuid!("0196ca5f-d934-7ec8-b042-ae37b94b8432"),
            description: None,
        };
        let tasks: Vec<Contains<TaskList, Task>> =
            backlog.get_linked_items(&backend).unwrap().collect();
        assert_eq!(
            tasks
                .into_iter()
                .map(|link| link.right.unwrap())
                .collect::<Vec<Task>>(),
            vec![task1, task2]
        );
    }

    #[test]
    fn create_task_in_tasklist() {
        use crate::task::{Contains, Link};
        let backend = TestBackend;
        let backlog = TaskList {
            name: "Backlog".into(),
            id: uuid!("0196fe23-7c01-7d6b-9e09-5968eb370549"),
        };
        let task3 = Task::new("Test task 3", None);
        let relationship: Contains<TaskList, Task> = backlog.link(&task3);
        relationship.create_linked_item(&backend).unwrap();
    }

    #[test]
    fn create_task_in_tasklist_mismatch() {
        use crate::task::{Contains, Link};
        let backend = TestBackend;
        let backlog = TaskList {
            name: "Backlog".into(),
            id: uuid!("0196fe23-7c01-7d6b-9e09-5968eb370549"),
        };
        let task3 = Task::new("MISMATCH", None);
        let relationship: Contains<TaskList, Task> = backlog.link(&task3);
        let mismatch = relationship.create_linked_item(&backend).unwrap_err();
        assert_matches!(
            mismatch,
            HelixFlowError::Mismatch { expected, actual: _ }
            if expected.as_ref().as_any().downcast_ref::<Task>() == Some(&task3)
        )
    }
}
