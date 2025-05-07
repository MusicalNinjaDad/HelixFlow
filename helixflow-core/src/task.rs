//! The fundamental `Task` building block and related functions.

use anyhow::{Ok, Result, anyhow};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

/// A Task
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Task<ID> {
    pub name: Cow<'static, str>,
    pub id: Option<ID>,
    pub description: Option<Cow<'static, str>>,
}

pub mod blocking {
    use super::*;
    /// Provide an implementation of a storage backend.
    pub trait StorageBackend<ID> {
        /// Create a new task in the backend, update the `task.id` then return Ok(())
        fn create(&self, task: &mut Task<ID>) -> Result<()>;
        fn get(&self, id: ID) -> Result<Task<ID>>;
    }

    impl<ID> Task<ID> {
        /// Create this task in a given storage backend.
        /// `&mut` because the creation process will update the `Task.id`
        ///
        /// Don't forget to check for, and handle, any `Error`s, even though you don't need the `Ok`.
        pub fn create<B: StorageBackend<ID>>(&mut self, backend: &B) -> Result<()> {
            backend.create(self)?;
            Ok(())
        }
    }

    #[derive(Clone, Copy)]
    pub struct TestBackend;

    /// Hardcoded cases to unit test the basic `Task` interface
    impl StorageBackend<u32> for TestBackend {
        fn create(&self, task: &mut Task<u32>) -> Result<()> {
            match task.name {
                Cow::Borrowed("FAIL") => Err(anyhow!("Taskname: FAIL")),
                _ => {
                    task.id = Some(1);
                    Ok(())
                }
            }
        }
        fn get(&self, id: u32) -> Result<Task<u32>> {
            match id {
                1 => Ok(Task {
                    name: "Task 1".into(),
                    id: Some(id),
                    description: None,
                }),
                _ => Err(anyhow!("Invalid task ID: {}", id)),
            }
        }
    }

    #[cfg(test)]
    pub mod tests {
        use super::*;

        #[test]
        fn test_new_task() {
            let mut new_task = Task {
                name: "Test Task 1".into(),
                id: None,
                description: None,
            };
            let backend = TestBackend;
            let _ = new_task.create(&backend);
            assert_eq!(new_task.name, "Test Task 1");
            assert_eq!(new_task.description, None);
            assert_eq!(new_task.id, Some(1));
        }

        #[test]
        fn test_failed_to_create_task() {
            let mut new_task = Task {
                name: "FAIL".into(),
                id: None,
                description: None,
            };
            let backend = TestBackend;
            let err = new_task.create(&backend);
            assert_eq!(new_task.name, "FAIL");
            assert_eq!(new_task.description, None);
            assert_eq!(new_task.id, None);
            assert!(err.is_err());
        }

        #[test]
        fn test_get_task() {
            let backend = TestBackend;
            let task = backend.get(1).unwrap();
            assert_eq!(
                task,
                Task {
                    name: "Task 1".into(),
                    id: Some(1),
                    description: None
                }
            )
        }

        #[test]
        fn test_get_invalid_task() {
            let backend = TestBackend;
            let err = backend.get(2).unwrap_err();
            assert_eq!(format!("{}", err), "Invalid task ID: 2");
        }
    }
}
