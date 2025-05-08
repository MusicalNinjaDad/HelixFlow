//! The fundamental `Task` building block and related functions.

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use uuid::Uuid;

/// A Task
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Task {
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
            description: if let Some(desc) = description {
                Some(desc.into())
            } else {
                None
            },
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TaskCreationError {
    // The #[from] attribute automatically creates a From conversion so that any error
    // convertible to anyhow::Error (which includes MyBackendError) is wrapped.
    #[error("backend error: {0}")]
    BackendError(#[from] anyhow::Error),

    #[error("created task does not match expectations: expected {expected:?}, got {actual:?}")]
    Mismatch { expected: Task, actual: Task },
}

pub type TaskResult<T> = std::result::Result<T, TaskCreationError>;

pub mod blocking {
    use super::*;
    /// Provide an implementation of a storage backend.
    pub trait StorageBackend {
        /// Create a new task in the backend, update the `task.id` then return Ok(())
        fn create(&self, task: &Task) -> anyhow::Result<Task>;
        // fn get(&self, id: ID) -> Result<Task<ID>>;
    }

    pub trait TaskExt
    where
        Self: Sized,
    {
        fn create<B: StorageBackend>(&self, backend: &B) -> TaskResult<()>;
    }

    impl TaskExt for Task {
        /// Create this task in a given storage backend.
        fn create<B: StorageBackend>(&self, backend: &B) -> TaskResult<()> {
            let created_task = backend.create(self)?;
            if &created_task == self {
                Ok(())
            } else {
                Err(TaskCreationError::Mismatch {
                    expected: self.clone(),
                    actual: created_task,
                })
            }
        }
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
        // fn get(&self, id: u32) -> Result<Task<u32>> {
        //     match id {
        //         1 => Ok(Task {
        //             name: "Task 1".into(),
        //             id: Some(id),
        //             description: None,
        //         }),
        //         _ => Err(anyhow!("Invalid task ID: {}", id)),
        //     }
        // }
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

        // #[test]
        // fn test_get_task() {
        //     let backend = TestBackend;
        //     let task = backend.get(1).unwrap();
        //     assert_eq!(
        //         task,
        //         Task {
        //             name: "Task 1".into(),
        //             id: Some(1),
        //             description: None
        //         }
        //     )
        // }

        // #[test]
        // fn test_get_invalid_task() {
        //     let backend = TestBackend;
        //     let err = backend.get(2).unwrap_err();
        //     assert_eq!(format!("{}", err), "Invalid task ID: 2");
        // }
    }
}

pub mod non_blocking {
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
    pub trait TaskExt
    where
        Self: Sized,
    {
        async fn create<B: StorageBackend + Sync>(&self, backend: &B) -> TaskResult<()>;
    }

    #[async_trait]
    impl TaskExt for Task {
        /// Create this task in a given storage backend.
        async fn create<B: StorageBackend + Sync>(&self, backend: &B) -> TaskResult<()> {
            let created_task = backend.create(self).await?;
            if &created_task == self {
                Ok(())
            } else {
                Err(TaskCreationError::Mismatch {
                    expected: self.clone(),
                    actual: created_task,
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

    #[cfg(all(test,not(target_family="wasm")))]
    pub mod tests {
        use std::sync::Arc;

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
    }

    #[cfg(all(test,target_family="wasm"))]
    pub mod wasm_tests {
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
    }
}
