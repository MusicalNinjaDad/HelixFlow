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
        /// `&mut` because the creation process will update the `Task.id`
        ///
        /// Don't forget to check for, and handle, any `Error`s, even though you don't need the `Ok`.
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

        use super::*;

        #[test]
        fn test_new_task() {
            let new_task = Task::new("Test Task", None);
            assert_eq!(new_task.name, "Test Task");
            assert!(new_task.description.is_none());
            assert!(!new_task.id.is_nil());
            assert_eq!(new_task.id.get_version(), Some(uuid::Version::SortRand));
        }

        #[test]
        fn test_new_task_blank() {
            let new_task = Task::new("", None);
            assert_eq!(new_task.name, "");
            assert!(new_task.description.is_none());
            assert!(!new_task.id.is_nil());
            assert_eq!(new_task.id.get_version(), Some(uuid::Version::SortRand));
        }

        #[test]
        fn test_create_task() {
            let new_task = Task::new("Test Task 1", None);
            let backend = TestBackend;
            new_task.create(&backend).unwrap();
        }

        #[test]
        fn test_failed_to_create_task() {
            let new_task = Task::new("FAIL", None);
            let backend = TestBackend;
            let err = new_task.create(&backend).unwrap_err();
            assert_matches!(err, TaskCreationError::BackendError(_))
        }

        #[test]
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

// pub mod non_blocking {
//     use async_trait::async_trait;

//     use super::*;

//     #[async_trait]
//     pub trait TaskExt<ID, B>
//     where
//         B: StorageBackend<ID> + Send + Sync,
//         ID: Send,
//     {
//         async fn create(&mut self, backend: &B) -> Result<()>;
//     }

//     #[async_trait]
//     impl<ID, B> TaskExt<ID, B> for Task<ID>
//     where
//         B: StorageBackend<ID> + Send + Sync,
//         ID: Send,
//     {
//         async fn create(&mut self, backend: &B) -> Result<()> {
//             backend.create(self).await
//         }
//     }

//     /// Provide an implementation of a storage backend.
//     #[async_trait]
//     pub trait StorageBackend<ID> {
//         /// Create a new task in the backend, update the `task.id` then return Ok(())
//         async fn create(&self, task: &mut Task<ID>) -> Result<()>;
//     }

//     #[derive(Clone, Copy)]
//     pub struct TestBackend;

//     /// Hardcoded cases to unit test the basic `Task` interface
//     #[async_trait]
//     impl StorageBackend<u32> for TestBackend {
//         async fn create(&self, task: &mut Task<u32>) -> Result<()> {
//             match task.name {
//                 Cow::Borrowed("FAIL") => Err(anyhow!("Taskname: FAIL")),
//                 _ => {
//                     task.id = Some(1);
//                     Ok(())
//                 }
//             }
//         }
//     }

//     #[cfg(test)]
//     pub mod tests {
//         use std::sync::Arc;

//         use super::*;
//         use macro_rules_attribute::apply;
//         use smol_macros::{test, Executor};

//         #[apply(test)]
//         async fn test_new_task(exec: &Executor<'_>) {
//             let mut new_task = Task {
//                 name: "Test Task 1".into(),
//                 id: None,
//                 description: None,
//             };
//             let backend = Arc::new(TestBackend);
//             let be = Arc::downgrade(&backend);
//             exec.spawn(async move {
//                 let backend = be.upgrade().unwrap();
//                 new_task.create(backend.as_ref()).await}).await.unwrap();
//             // assert_eq!(new_task.name, "Test Task 1");
//             // assert_eq!(new_task.description, None);
//             // assert_eq!(new_task.id, Some(1));
//         }
//     }
// }
