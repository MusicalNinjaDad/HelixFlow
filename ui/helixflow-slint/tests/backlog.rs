#![feature(cfg_boolean_literals)]
use std::rc::Rc;

use helixflow_core::task::{
    blocking::{StorageBackend, TestBackend},
};
use helixflow_slint::{Backlog, SlintTask, test::*};
use slint::{ComponentHandle, ModelRc, VecModel};
use uuid::uuid;

#[test]
fn update_tasks_in_event_loop() {
    prepare_slint!();
    let backlog = Backlog::new().unwrap();
    list_elements!(&backlog);
    let task1 = SlintTask {
        name: "Test task 1".into(),
        id: "1".into(),
    };
    let task2 = SlintTask {
        name: "Test task 2".into(),
        id: "2".into(),
    };
    let tasks = vec![task1, task2];
    let backlog_entries: VecModel<SlintTask> = tasks.clone().into();
    let bl = backlog.as_weak();
    slint::spawn_local(async move {
        let backlog = bl.unwrap();
        backlog.set_tasks(ModelRc::new(backlog_entries));
        slint::quit_event_loop().unwrap();
    })
    .unwrap();
    run_slint_loop!();
    println!("===Loop finished===");
    list_elements!(&backlog);
    let backlog_tasks = ElementHandle::find_by_element_type_name(&backlog, "TaskListItem");
    assert_values!(backlog_tasks, tasks);
}

#[test]
fn initialise_backlog() {
    prepare_slint!();

    let backlog = Backlog::new().unwrap();
    list_elements!(&backlog);

    let backend = Rc::new(TestBackend);
    let be = Rc::downgrade(&backend);

    let backlog_id = uuid!("0196fe23-7c01-7d6b-9e09-5968eb370549");

    backlog.init(be, &backlog_id);

    assert_eq!(backlog.get_backlog_name(), "Test TaskList 1");
    let backlog_tasks = ElementHandle::find_by_element_type_name(&backlog, "TaskListItem");
    let expected_tasks: Vec<SlintTask> = backend
        .get_tasks_in(&backlog_id)
        .unwrap()
        .map(|task| task.unwrap().into())
        .collect();
    assert_values!(backlog_tasks, expected_tasks);
}
