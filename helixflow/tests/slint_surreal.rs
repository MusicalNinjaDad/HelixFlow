use std::rc::Rc;

use helixflow_core::task::{Task, blocking::CRUD};
use i_slint_backend_testing::ElementHandle;
use slint::ComponentHandle;
use slint::platform::PointerEventButton;

use helixflow_slint::{HelixFlow, task::blocking::create_task, test::*};
use helixflow_surreal::blocking::SurrealDb;

#[test]
fn test_slint_with_surreal() {
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
        assert_eq!(helixflow.get_task_id(), "");
        assert!(helixflow.get_create_enabled());

        let create = get!(&helixflow, "TaskBox::create");
        create.single_click(PointerEventButton::Left).await;

        slint::quit_event_loop().unwrap();
    })
    .unwrap();

    run_slint_loop!();

    let task_uuid = uuid::Uuid::parse_str(&helixflow.get_task_id()).unwrap();
    assert!(!task_uuid.is_nil());
    assert_eq!(task_uuid.get_version(), Some(uuid::Version::SortRand));
    assert!(helixflow.get_create_enabled());
    let ui_task = Task {
        name: helixflow.get_task_name().to_string().into(),
        id: task_uuid,
        description: None,
    };
    let db_task = Task::get(backend.as_ref(), &task_uuid).unwrap();
    assert_eq!(ui_task, db_task);
}
