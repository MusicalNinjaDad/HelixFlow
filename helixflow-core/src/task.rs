//! The fundamental `Task` building block and related functions.

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use uuid::Uuid;
use uuid::uuid;
/// A Task
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
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

/// Iterator of `Task`s
pub struct TaskList;

#[derive(Debug, thiserror::Error)]
pub enum TaskCreationError {
    // The #[from] anyhow::Error will convert anything that offers `into anyhow::Error`.
    #[error("backend error: {0}")]
    BackendError(#[from] anyhow::Error),

    #[error("created task does not match expectations: expected {expected:?}, got {actual:?}")]
    Mismatch {
        expected: Box<Task>,
        actual: Box<Task>,
    },

    #[error("task id ({id:?}) is not a valid UUID v7")]
    InvalidID { id: String },
    // TODO Rename to TaskCRUDError and add "Unknown ID"
}

pub type TaskResult<T> = std::result::Result<T, TaskCreationError>;

pub mod blocking {
    use super::*;

    pub trait CRUD
    where
        Self: Sized,
    {
        fn create<B: StorageBackend>(&self, backend: &B) -> TaskResult<()>;
        fn get<B: StorageBackend>(backend: &B, id: &Uuid) -> TaskResult<Task>;
    }

    impl CRUD for Task {
        /// Create this task in a given storage backend.
        fn create<B: StorageBackend>(&self, backend: &B) -> TaskResult<()> {
            let created_task = backend.create(self)?;
            if &created_task == self {
                Ok(())
            } else {
                Err(TaskCreationError::Mismatch {
                    expected: Box::new(self.clone()),
                    actual: Box::new(created_task),
                })
            }
        }

        /// Get task from `backend` by `id`
        fn get<B: StorageBackend>(backend: &B, id: &Uuid) -> TaskResult<Task> {
            Ok(backend.get(id)?)
        }
    }

    impl TaskList {
        pub fn all<B: StorageBackend>(backend: &B) -> TaskResult<impl Iterator<Item = Task>> {
            Ok(backend.get_tasks()?)
        }
    }

    /// Provide an implementation of a storage backend.
    pub trait StorageBackend {
        /// Create a new task in the backend, update the `task.id`.
        ///
        /// The returned Task should be the actual stored record from the backend - to allow
        /// validation by `Task::create()`
        fn create(&self, task: &Task) -> anyhow::Result<Task>;

        /// Get an existing task from the backend
        fn get(&self, id: &Uuid) -> anyhow::Result<Task>;

        fn get_tasks(&self) -> anyhow::Result<impl Iterator<Item = Task>>;
    }

    #[derive(Clone, Copy)]
    pub struct TestBackend;

    /// Hardcoded cases to unit test the basic `Task` interface
    impl StorageBackend for TestBackend {
        fn create(&self, task: &Task) -> anyhow::Result<Task> {
            match task.name {
                Cow::Borrowed("FAIL") => Err(anyhow!("Taskname: FAIL")),
                Cow::Borrowed("MISMATCH") => {
                    Ok(Task::new(task.name.clone(), task.description.clone()))
                }
                _ => Ok(task.clone()),
            }
        }
        fn get(&self, id: &Uuid) -> anyhow::Result<Task> {
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
                _ => Err(anyhow!("Unknown task ID: {}", id)),
            }
        }
        fn get_tasks(&self) -> anyhow::Result<impl Iterator<Item = Task>> {
            Ok(vec![
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
            ]
            .into_iter())
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
            assert_matches!(err, TaskCreationError::BackendError(_))
        }

        #[wasm_bindgen_test(unsupported = test)]
        fn test_mismatched_task_created() {
            let new_task = Task::new("MISMATCH", None);
            let backend = TestBackend;
            let err = new_task.create(&backend).unwrap_err();
            assert_matches!(
                err,
                TaskCreationError::Mismatch {
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
                    id: id,
                    description: None
                }
            )
        }

        #[wasm_bindgen_test(unsupported = test)]
        fn test_get_invalid_task() {
            let backend = TestBackend;
            let id = uuid!("0196b4c9-8447-78db-ae8a-be68a8095aa2");
            let err = backend.get(&id).unwrap_err();
            assert_eq!(
                format!("{}", err),
                "Unknown task ID: 0196b4c9-8447-78db-ae8a-be68a8095aa2"
            );
        }

        #[test]
        fn list_tasks() {
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
            let all_tasks: Vec<Task> = TaskList::all(&backend).unwrap().collect();
            assert_eq!(all_tasks, vec![task1, task2]);
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
        async fn create<B: StorageBackend + Sync>(&self, backend: &B) -> TaskResult<()>;
    }

    #[async_trait]
    impl CRUD for Task {
        /// Create this task in a given storage backend.
        async fn create<B: StorageBackend + Sync>(&self, backend: &B) -> TaskResult<()> {
            let created_task = backend.create(self).await?;
            if &created_task == self {
                Ok(())
            } else {
                Err(TaskCreationError::Mismatch {
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
            assert_matches!(err, TaskCreationError::BackendError(_))
        }

        #[apply(test)]
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
