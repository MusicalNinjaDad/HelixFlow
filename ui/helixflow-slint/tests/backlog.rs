use std::rc::Rc;

use uuid::uuid;

use slint::{ComponentHandle, ModelRc, VecModel};

use helixflow_core::{
    CRUD, Linkable,
    task::{TaskList, TestBackend},
};
use helixflow_slint::{Backlog, SlintTask, task::load_backlog, test::*};

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

    let backlog_id = uuid!("0196fe23-7c01-7d6b-9e09-5968eb370549");
    let tasklist = TaskList::get(backend.as_ref(), &backlog_id).unwrap();
    backlog.set_tasklist(tasklist.into());

    let be = Rc::downgrade(&backend);
    let bl = backlog.as_weak();
    backlog.on_load(load_backlog(bl, be));
    backlog.invoke_load();

    let backlog_name = get!(&backlog, "Backlog::backlog_title");
    assert_eq!(
        backlog_name.accessible_value().unwrap(),
        "Test TaskList 1".to_shared_string()
    );
    let backlog_tasks = ElementHandle::find_by_element_type_name(&backlog, "TaskListItem");
    let expected_tasks: Vec<SlintTask> = TaskList::get(backend.as_ref(), &backlog_id)
        .unwrap()
        .get_linked_items(backend.as_ref())
        .unwrap()
        .map(|link| link.right)
        .map(Result::unwrap)
        .map(Into::into)
        .collect();
    assert_values!(backlog_tasks, expected_tasks);
}
