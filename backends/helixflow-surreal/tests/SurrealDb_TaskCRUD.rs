//! Test calling Task::CRUD_fn(&SurrealDb) ...

#![feature(assert_matches)]
#![feature(cfg_boolean_literals)]
#![cfg(false)]
#![cfg(test)]

use assert_unordered::assert_eq_unordered_sort;
use std::assert_matches::assert_matches;

use surrealdb::Uuid;

use helixflow_core::task::{CRUD, HelixFlowError, Link, Linkable, Task, TaskList};
use helixflow_surreal::SurrealDb;

#[test]
fn test_create() {
    {
        let new_task = Task::new("Test Task 2", None);
        let backend = SurrealDb::new(None).unwrap();
        new_task.create(&backend).unwrap();
    }
}

#[test]
fn test_create_then_get() {
    {
        let new_task = Task::new("Test Task 2", None);
        let backend = SurrealDb::new(None).unwrap();
        new_task.create(&backend).unwrap();
        let stored_task = Task::get(&backend, &new_task.id).unwrap();
        assert_eq!(stored_task, new_task);
    }
}

#[test]
fn test_get_not_found() {
    {
        let backend = SurrealDb::new(None).unwrap();
        let id = Uuid::now_v7();
        let err = Task::get(&backend, &id).unwrap_err();
        assert_matches!(
            err,
            HelixFlowError::NotFound { itemtype, id }
            if itemtype == "Task" && id == id
        );
    }
}

#[test]
fn create_tasklist() {
    let backend = SurrealDb::new(None).unwrap();
    let tasklist = TaskList::new("Test tasklist");
    tasklist.create(&backend).unwrap();
    let stored_tasklist = TaskList::get(&backend, &tasklist.id).unwrap();
    assert_eq!(stored_tasklist, tasklist);
}

#[test]
fn create_task_in_tasklist() {
    let backend = SurrealDb::new(None).unwrap();
    let tasklist = TaskList::new("Test tasklist");
    tasklist.create(&backend).unwrap();
    let task = Task::new("Test Task 2", None);
    let link = tasklist.link(&task);
    link.create_linked_item(&backend).unwrap();
    let tasks: Vec<Task> = tasklist
        .get_linked_items(&backend)
        .unwrap()
        .map(|link| link.right.unwrap())
        .collect();
    assert_eq!(tasks, vec![task]);
}

#[test]
fn create_two_tasks_in_tasklist() {
    let backend = SurrealDb::new(None).unwrap();
    let tasklist = TaskList::new("Test tasklist");
    tasklist.create(&backend).unwrap();
    let task2 = Task::new("Test Task 2", None);
    tasklist.link(&task2).create_linked_item(&backend).unwrap();
    let task3 = Task::new("Test Task 3", None);
    tasklist.link(&task3).create_linked_item(&backend).unwrap();
    let tasks: Vec<Task> = tasklist
        .get_linked_items(&backend)
        .unwrap()
        .map(|link| link.right.unwrap())
        .collect();
    // TODO - Return in sorted order using a sort-criteria stored in the relationship
    assert_eq_unordered_sort!(tasks, vec![task2, task3]);
}
