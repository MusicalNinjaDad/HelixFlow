//! The fundamental `Task` building block and related functions.

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::borrow::Cow;
use std::ops::ControlFlow;
use std::ops::FromResidual;
use std::ops::Try;
use uuid::Uuid;
use uuid::uuid;

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
    fn from_output(output: Self::Output) -> Self {
        todo!()
    }
}

impl<LEFT, RIGHT> FromResidual<Contains<LEFT, RIGHT>> for Contains<LEFT, RIGHT>
where
    LEFT: HelixFlowItem,
    RIGHT: HelixFlowItem,
{
    fn from_residual(residual: Contains<LEFT, RIGHT>) -> Self {
        todo!()
    }
}

impl<LEFT, RIGHT> FromResidual<Contains<LEFT, RIGHT>> for HelixFlowResult<()>
where
    LEFT: HelixFlowItem,
    RIGHT: HelixFlowItem,
{
    fn from_residual(residual: Contains<LEFT, RIGHT>) -> Self {
        todo!()
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

    #[error("Relationship contains Errors")]
    Relationship {},
}

pub type HelixFlowResult<T> = std::result::Result<T, HelixFlowError>;

#[cfg(test)]
mod tests {
    use super::*;

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
}

pub mod blocking {
    use super::*;

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
        Self: Sized,
    {
        fn create_linked_item<B: Relate<Self>>(self, backend: &B) -> HelixFlowResult<()>;
    }

    pub trait Linkable<REL: Link> {
        type Right;
        fn link(&self, right: &Self::Right) -> REL;
        fn get_linked_items<B: Relate<REL, Left = Self>>(
            &self,
            backend: &B,
        ) -> HelixFlowResult<impl Iterator<Item = REL>>;
    }

    /// Methods to relate items in a backend
    pub trait Relate<REL: Link> {
        type Left;
        /// Create and link the related item
        fn create_linked_item(&self, link: &REL) -> HelixFlowResult<REL>;
        fn get_linked_items(&self, left: &Self::Left)
        -> HelixFlowResult<impl Iterator<Item = REL>>;
    }

    impl Link for Contains<TaskList, Task> {
        fn create_linked_item<B: Relate<Contains<TaskList, Task>>>(
            self,
            backend: &B,
        ) -> HelixFlowResult<()> {
            if self.left.is_ok() && self.right.is_ok() {
                let created = backend.create_linked_item(&self)?;
                let expected = self.right?;
                match created.right {
                    Ok(task) if task == expected => Ok(()),
                    Ok(_) => Err(HelixFlowError::Mismatch {
                        expected: Box::new(expected.clone()),
                        actual: Box::new(created.right?.clone()),
                    }),
                    Err(e) => Err(e),
                }
            } else {
                self.left?;
                self.right?;
                // Unreachable hack - either left or right are Err so one of above lines returns
                Ok(())
            }
        }
    }

    impl Linkable<Contains<TaskList, Task>> for TaskList {
        type Right = Task;
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
            B: Relate<Contains<TaskList, Task>, Left = TaskList>,
        {
            backend.get_linked_items(self)
        }
    }

    impl TaskList {
        // TODO: Update this to return a `TaskResult<TaskList>`
        pub fn all<B: StorageBackend>(
            backend: &B,
        ) -> HelixFlowResult<impl Iterator<Item = HelixFlowResult<Task>>> {
            Ok(backend.get_all_tasks()?)
        }

        /// Get all Tasks belonging to this TaskList
        pub fn tasks<B: StorageBackend>(
            &self,
            backend: &B,
        ) -> HelixFlowResult<impl Iterator<Item = HelixFlowResult<Task>>> {
            Ok(backend.get_tasks_in(&self.id)?)
        }
    }

    /// Provide an implementation of a storage backend.
    pub trait StorageBackend {
        fn get_all_tasks(&self) -> anyhow::Result<impl Iterator<Item = HelixFlowResult<Task>>>;

        fn get_tasks_in(
            &self,
            id: &Uuid,
        ) -> anyhow::Result<impl Iterator<Item = HelixFlowResult<Task>>>;
    }

    #[derive(Clone, Copy)]
    pub struct TestBackend;

    impl Store<Task> for TestBackend {
        fn create(&self, task: &Task) -> HelixFlowResult<Task> {
            match task.name {
                Cow::Borrowed("FAIL") => Err(anyhow!("Failed to create task").into()),
                Cow::Borrowed("MISMATCH") => {
                    Ok(Task::new(task.name.clone(), task.description.clone()))
                }
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
        fn create(&self, item: &TaskList) -> HelixFlowResult<TaskList> {
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
        type Left = TaskList;
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
            left: &Self::Left,
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

    /// Hardcoded cases to unit test the basic `Task` interface
    impl StorageBackend for TestBackend {
        fn get_all_tasks(&self) -> anyhow::Result<impl Iterator<Item = HelixFlowResult<Task>>> {
            Ok(vec![
                Ok(Task {
                    name: "Task 1".into(),
                    id: uuid!("0196b4c9-8447-7959-ae1f-72c7c8a3dd36"),
                    description: None,
                }),
                Ok(Task {
                    name: "Task 2".into(),
                    id: uuid!("0196ca5f-d934-7ec8-b042-ae37b94b8432"),
                    description: None,
                }),
            ]
            .into_iter())
        }
        fn get_tasks_in(
            &self,
            id: &Uuid,
        ) -> anyhow::Result<impl Iterator<Item = HelixFlowResult<Task>>> {
            self.get_all_tasks()
        }
    }

    #[cfg(test)]
    pub mod tests {
        use std::assert_matches::assert_matches;
        use wasm_bindgen_test::*;

        use super::*;

        wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

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
        fn list_all_tasks() {
            let backend = TestBackend;
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
            let all_tasks: Vec<HelixFlowResult<Task>> = TaskList::all(&backend).unwrap().collect();
            assert_eq!(
                all_tasks
                    .into_iter()
                    .map(|task| task.unwrap())
                    .collect::<Vec<Task>>(),
                vec![task1, task2]
            );
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
            use crate::task::{Contains, blocking::Link};
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
            use crate::task::{Contains, blocking::Link};
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
}

/// Non-blocking version of Task CRUD.
pub mod non_blocking {
    // TODO update to full functionality and identical sematics based on blocking version.
    use async_trait::async_trait;

    use super::*;

    /// Provide an implementation of a storage backend.
    #[async_trait]
    pub trait StorageBackend {
        /// Create a new task in the backend, update the `task.id` then return Ok(())
        async fn create(&self, task: &Task) -> anyhow::Result<Task>;
        // fn get(&self, id: ID) -> Result<Task<ID>>;
    }

    #[async_trait]
    pub trait CRUD
    where
        Self: Sized,
    {
        async fn create<B: StorageBackend + Sync>(&self, backend: &B) -> HelixFlowResult<()>;
    }

    #[async_trait]
    impl CRUD for Task {
        /// Create this task in a given storage backend.
        async fn create<B: StorageBackend + Sync>(&self, backend: &B) -> HelixFlowResult<()> {
            let created_task = backend.create(self).await?;
            if &created_task == self {
                Ok(())
            } else {
                Err(HelixFlowError::Mismatch {
                    expected: Box::new(self.clone()),
                    actual: Box::new(created_task),
                })
            }
        }
    }

    #[derive(Clone, Copy)]
    pub struct TestBackend;

    /// Hardcoded cases to unit test the basic `Task` interface
    #[async_trait]
    impl StorageBackend for TestBackend {
        async fn create(&self, task: &Task) -> anyhow::Result<Task> {
            match task.name {
                Cow::Borrowed("FAIL") => Err(anyhow!("Taskname: FAIL")),
                Cow::Borrowed("MISMATCH") => {
                    Ok(Task::new(task.name.clone(), task.description.clone()))
                }
                _ => Ok(task.clone()),
            }
        }
    }

    #[cfg(all(test, not(target_family = "wasm")))]
    pub mod tests {
        use std::{assert_matches::assert_matches, sync::Arc};

        use super::*;
        use macro_rules_attribute::apply;
        use smol_macros::{Executor, test};

        #[apply(test)]
        async fn test_new_task(exec: &Executor<'_>) {
            let new_task = Task::new("Test Task 1", None);
            let backend = Arc::new(TestBackend);
            let be = Arc::downgrade(&backend);
            exec.spawn(async move {
                let backend = be.upgrade().unwrap();
                new_task.create(backend.as_ref()).await
            })
            .await
            .unwrap();
        }

        #[apply(test)]
        async fn test_failed_to_create_task() {
            let new_task = Task::new("FAIL", None);
            let backend = TestBackend;
            let err = new_task.create(&backend).await.unwrap_err();
            assert_matches!(err, HelixFlowError::BackendError(_))
        }

        #[apply(test)]
        async fn test_mismatched_task_created() {
            let new_task = Task::new("MISMATCH", None);
            let backend = TestBackend;
            let err = new_task.create(&backend).await.unwrap_err();
            assert_matches!(
                err,
                HelixFlowError::Mismatch {
                    expected: _,
                    actual: _
                }
            )
        }
    }

    #[cfg(all(test, target_family = "wasm"))]
    pub mod wasm_tests {
        use std::assert_matches::assert_matches;
        use std::sync::Arc;

        use super::*;
        use wasm_bindgen_test::wasm_bindgen_test;

        #[wasm_bindgen_test]
        async fn test_new_task() {
            let new_task = Task::new("Test Task 1", None);
            let backend = Arc::new(TestBackend);
            let be = Arc::downgrade(&backend);
            async move {
                let backend = be.upgrade().unwrap();
                new_task.create(backend.as_ref()).await
            }
            .await
            .unwrap();
        }

        #[wasm_bindgen_test]
        async fn test_failed_to_create_task() {
            let new_task = Task::new("FAIL", None);
            let backend = TestBackend;
            let err = new_task.create(&backend).await.unwrap_err();
            assert_matches!(err, TaskCreationError::BackendError(_))
        }

        #[wasm_bindgen_test]
        async fn test_mismatched_task_created() {
            let new_task = Task::new("MISMATCH", None);
            let backend = TestBackend;
            let err = new_task.create(&backend).await.unwrap_err();
            assert_matches!(
                err,
                TaskCreationError::Mismatch {
                    expected: _,
                    actual: _
                }
            )
        }
    }
}
