//! The fundamental `Task` building block and related functions.

use std::borrow::Cow;

use anyhow::{Ok, Result};

/// A Task
#[allow(dead_code)]
struct Task<ID> {
    name: Cow<'static, str>,
    id: Option<ID>,
    description: Option<Cow<'static, str>>,
}

impl Task<u32> {
    /// Create a new task in the selected storage backend.
    /// Returned Result should include the task ID.
    #[allow(dead_code)]
    fn create(&mut self) -> Result<&mut Self> {
        self.id = Some(1);
        Ok(self)
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
        let _ = new_task.create();
        assert_eq!(new_task.name, "Test Task 1");
        assert_eq!(new_task.description, None);
        assert_eq!(new_task.id, Some(1));
    }
}
