#![cfg(test)]
//! Test calling Task::CRUD_fn(&SurrealDb) ...

use helixflow_core::task::Task;
use surrealdb::Uuid;

mod blocking {

    use super::*;

    use helixflow_core::task::{TaskList, blocking::CRUD};
    use helixflow_surreal::blocking::SurrealDb;

    #[test]
    fn test_create() {
        {
            let new_task = Task::new("Test Task 2", None);
            let backend = SurrealDb::new().unwrap();
            new_task.create(&backend).unwrap();
        }
    }

    #[test]
    fn test_create_then_get() {
        {
            let new_task = Task::new("Test Task 2", None);
            let backend = SurrealDb::new().unwrap();
            new_task.create(&backend).unwrap();
            let stored_task = Task::get(&backend, &new_task.id).unwrap();
            assert_eq!(stored_task, new_task);
        }
    }

    #[test]
    fn test_get_not_found() {
        {
            let backend = SurrealDb::new().unwrap();
            let id = Uuid::now_v7();
            let err = Task::get(&backend, &id).unwrap_err();
            assert_eq!(
                format!("{}", err),
                format!("backend error: Unknown task ID: {}", id)
            );
        }
    }

    #[test]
    fn create_tasklist() {
        let backend = SurrealDb::new().unwrap();
        let tasklist = TaskList::new("Test tasklist");
        tasklist.create(&backend).unwrap();
    }
}
