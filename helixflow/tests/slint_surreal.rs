use std::rc::Rc;

use helixflow_core::task::{Task, blocking::CRUD};
use slint::platform::PointerEventButton;
use slint::{ComponentHandle, Global};

use helixflow_slint::{CurrentTask, HelixFlow, task::blocking::create_task, test::*};
use helixflow_surreal::blocking::SurrealDb;

#[test]
fn test_create_task() {
    prepare_slint!();

    let backend = Rc::new(SurrealDb::new().unwrap());

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
