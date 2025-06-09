#![feature(cfg_boolean_literals)]

use std::rc::Rc;

use helixflow_core::state::State;
use slint::platform::PointerEventButton;
use slint::{ComponentHandle, Global};

use helixflow_core::{
    CRUD,
    task::{Task, TaskList},
};
use helixflow_slint::{
    CurrentTask, HelixFlow,
    task::{create_task, create_task_in_backlog, load_backlog},
    test::*,
};
use helixflow_surreal::SurrealDb;

#[test]
fn test_create_task() {
    prepare_slint!();

    let backend = Rc::new(SurrealDb::new(None).unwrap());

    let helixflow = HelixFlow::new().unwrap();
    list_elements!(&helixflow);

    let hf = helixflow.as_weak();
    let be = Rc::downgrade(&backend);
    helixflow.on_create_task(create_task(hf, be));

    let hf = helixflow.as_weak();
    slint::spawn_local(async move {
        let helixflow = hf.unwrap();
        helixflow.set_task_name("A valid task".into());

        let task_id_display = get!(&helixflow, "TaskBox::task_id_display");
        assert_eq!(task_id_display.accessible_value().unwrap(), "");

        let create = get!(&helixflow, "TaskBox::create");
        assert!(helixflow.get_create_enabled());
        assert!(create.accessible_enabled().unwrap());
        create.single_click(PointerEventButton::Left).await;

        slint::quit_event_loop().unwrap();
    })
    .unwrap();

    run_slint_loop!();

    let ui_task: Task = CurrentTask::get(&helixflow).get_task().try_into().unwrap();

    let task_id_display = get!(&helixflow, "TaskBox::task_id_display");
    assert_eq!(
        task_id_display.accessible_value().unwrap(),
        ui_task.id.to_string()
    );

    let db_task = Task::get(backend.as_ref(), &ui_task.id).unwrap();
    assert_eq!(ui_task, db_task);

    let create = get!(&helixflow, "TaskBox::create");
    assert!(helixflow.get_create_enabled());
    assert!(create.accessible_enabled().unwrap());
}

#[test]
fn add_tasks_to_backlog() {
    prepare_slint!();

    let backend = Rc::new(SurrealDb::new(None).unwrap());

    let helixflow = HelixFlow::new().unwrap();
    list_elements!(&helixflow);

    let backlog = TaskList::new("This week");
    backlog.create(backend.as_ref()).unwrap();
    helixflow.set_backlog(backlog.into());

    let hf = helixflow.as_weak();
    let be = Rc::downgrade(&backend);
    helixflow.on_load_backlog(load_backlog(hf, be));

    let hf = helixflow.as_weak();
    let be = Rc::downgrade(&backend);
    helixflow.on_create_backlog_task(create_task_in_backlog(hf, be));

    helixflow.invoke_load_backlog();
    let hf = helixflow.as_weak();
    slint::spawn_local(async move {
        let helixflow = hf.unwrap();
        let task_entry = get!(&helixflow, "Backlog::new_task_entry");
        task_entry.set_accessible_value("New task 1");
        let create = get!(&helixflow, "Backlog::quick_create_button");
        create.single_click(PointerEventButton::Left).await;
        slint::quit_event_loop().unwrap();
    })
    .unwrap();

    run_slint_loop!();

    let tasks = ElementHandle::find_by_element_type_name(&helixflow, "TaskListItem");
    let expected_task_values = ["New task 1"];
    assert_values!(tasks, expected_task_values);
    let task_entry = get!(&helixflow, "Backlog::new_task_entry");
    assert_eq!(task_entry.accessible_value().unwrap(), "");
}


#[cfg(false)]
#[test]
fn store_ui_state() {
    use uuid::Uuid;

    prepare_slint!();

    let backend = Rc::new(SurrealDb::new(None).unwrap());

    let helixflow = HelixFlow::new().unwrap();
    list_elements!(&helixflow);

    let backlog = TaskList::new("This week");
    let state_id = Uuid::now_v7();

    let mut ui_state: State = State::new(&state_id);
    ui_state.visible_backlog(&backlog);
    ui_state.create(backend.as_ref()).unwrap();
    
    let stored_backlog = State::get(backend.as_ref(), &state_id).unwrap().visible_backlog_id();

    assert_eq!(stored_backlog, &Some(backlog.id));
}