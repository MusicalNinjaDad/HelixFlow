//! The fundamental `Task` building block and related functions.

use std::borrow::Cow;

use anyhow::{Ok, Result, anyhow};

/// A Task
#[allow(dead_code)]
struct Task<ID> {
    name: Cow<'static, str>,
    id: Option<ID>,
    description: Option<Cow<'static, str>>,
}

/// Provide an implementation of a storage backend.
trait StorageBackend<ID> {
    /// Create a new task in the selected storage backend.
    /// Returned Result should include the task ID.
    #[allow(dead_code)]
    fn create(&self, task: &mut Task<ID>) -> Result<()>;
}

impl Task<u32> {
    /// Create a new task in the selected storage backend.
    /// Returned Result should include the task ID.
    #[allow(dead_code)]
    fn create<B: StorageBackend<u32>>(&mut self, backend: &B) -> Result<()> {
        backend.create(self)?;
        Ok(())
    }
}

#[allow(dead_code)]
struct TestBackend;

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
}

#[cfg(test)]
mod tests {
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
}
