//! The fundamental `Task` building block and related functions.

use anyhow::{Ok, Result};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

/// A Task
#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct Task<ID> {
    pub name: Cow<'static, str>,
    pub id: Option<ID>,
    pub description: Option<Cow<'static, str>>,
}

/// Provide an implementation of a storage backend.
pub(crate) trait StorageBackend<ID> {
    /// Create a new task in the backend, update the `task.id` then return Ok(())
    #[allow(dead_code)]
    async fn create(&self, task: &mut Task<ID>) -> Result<()>;
}

impl<ID> Task<ID> {
    /// Create this task in a given storage backend.
    /// `&mut` because the creation process will update the `Task.id`
    ///
    /// Don't forget to check for and handle any `Error`s, even though you don't need the `Ok`.
    #[allow(dead_code)]
    pub(crate) async fn create<B: StorageBackend<ID>>(&mut self, backend: &B) -> Result<()> {
        backend.create(self).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;

    struct TestBackend;

    /// Hardcoded cases to unit test the basic `Task` interface
    impl StorageBackend<u32> for TestBackend {
        async fn create(&self, task: &mut Task<u32>) -> Result<()> {
            match task.name {
                Cow::Borrowed("FAIL") => Err(anyhow!("Taskname: FAIL")),
                _ => {
                    task.id = Some(1);
                    Ok(())
                }
            }
        }
    }

    #[tokio::test]
    async fn test_new_task() {
        let mut new_task = Task {
            name: "Test Task 1".into(),
            id: None,
            description: None,
        };
        let backend = TestBackend;
        let _ = new_task.create(&backend).await;
        assert_eq!(new_task.name, "Test Task 1");
        assert_eq!(new_task.description, None);
        assert_eq!(new_task.id, Some(1));
    }

    #[tokio::test]
    async fn test_failed_to_create_task() {
        let mut new_task = Task {
            name: "FAIL".into(),
            id: None,
            description: None,
        };
        let backend = TestBackend;
        let err = new_task.create(&backend).await;
        assert_eq!(new_task.name, "FAIL");
        assert_eq!(new_task.description, None);
        assert_eq!(new_task.id, None);
        assert!(err.is_err());
    }
}
