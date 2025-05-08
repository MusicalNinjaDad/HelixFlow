//! The fundamental `Task` building block and related functions.

use anyhow::{Ok, Result, anyhow};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::borrow::Cow;

/// A Task
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Task {
    pub name: Cow<'static, str>,
    pub id: Uuid,
    pub description: Option<Cow<'static, str>>,
}

pub mod blocking {
    use super::*;
    /// Provide an implementation of a storage backend.
    pub trait StorageBackend {
        /// Create a new task in the backend, update the `task.id` then return Ok(())
        fn create(&self, task: &Task) -> Result<Task>;
        // fn get(&self, id: ID) -> Result<Task<ID>>;
    }

    pub trait TaskExt where Self: Sized {
        fn create<B: StorageBackend>(&self, backend: &B) -> Result<Self>;
    }

    impl TaskExt for Task {
        /// Create this task in a given storage backend.
        /// `&mut` because the creation process will update the `Task.id`
        ///
        /// Don't forget to check for, and handle, any `Error`s, even though you don't need the `Ok`.
        fn create<B: StorageBackend>(&self, backend: &B) -> Result<Task> {
            backend.create(self)
        }
    }

    #[derive(Clone, Copy)]
    pub struct TestBackend;

    /// Hardcoded cases to unit test the basic `Task` interface
    impl StorageBackend for TestBackend {
        fn create(&self, task: &Task) -> Result<Task> {
            match task.name {
                Cow::Borrowed("FAIL") => Err(anyhow!("Taskname: FAIL")),
                _ => {
                    Ok(task.clone())
                }
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
        use super::*;

        #[test]
        fn test_new_task() {
            let new_task = Task {
                name: "Test Task 1".into(),
                id: Uuid::now_v7(),
                description: None,
            };
            let backend = TestBackend;
            let created_task = new_task.create(&backend).unwrap();
            assert_eq!(created_task, new_task);
        }

        #[test]
        fn test_failed_to_create_task() {
            let new_task = Task {
                name: "FAIL".into(),
                id: Uuid::now_v7(),
                description: None,
            };
            let backend = TestBackend;
            let err = new_task.create(&backend);
            assert!(err.is_err());
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
