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
        let stored_tasklist = TaskList::get(&backend, &tasklist.id).unwrap();
        assert_eq!(stored_tasklist, tasklist);
    }

    #[test]
    fn create_task_in_tasklist() {
        let backend = SurrealDb::new().unwrap();
        let tasklist = TaskList::new("Test tasklist");
        tasklist.create(&backend).unwrap();
        let task = Task::new("Test Task 2", None);
        task.create_linked(&backend, &tasklist).unwrap();
        let tasks: Vec<Task> = tasklist
            .tasks(&backend)
            .unwrap()
            .map(Result::unwrap)
            .collect();
        assert_eq!(tasks, vec![task]);
    }

    #[test]
    fn create_two_tasks_in_tasklist() {
        let backend = SurrealDb::new().unwrap();
        let tasklist = TaskList::new("Test tasklist");
        tasklist.create(&backend).unwrap();
        let task2 = Task::new("Test Task 2", None);
        task2.create_linked(&backend, &tasklist).unwrap();
        let task3 = Task::new("Test Task 3", None);
        task3.create_linked(&backend, &tasklist).unwrap();
        let tasks: Vec<Task> = tasklist
            .tasks(&backend)
            .unwrap()
            .map(Result::unwrap)
            .collect();
        assert_eq!(tasks, vec![task2, task3]);
    }
}
