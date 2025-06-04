//! Use nextest - these tests will fail on cargo test as they MUST run is separate processes
//! for `i_slint_backend_testing::init_integration_test_with_system_time`

use std::rc::Rc;

use slint::platform::PointerEventButton;
use slint::{ComponentHandle, Global};

use helixflow_core::task::{Task, TestBackend};
use helixflow_slint::{CurrentTask, HelixFlow, task::create_task, test::*};

#[test]
fn test_set_task_id() {
    prepare_slint!();

    let helixflow = HelixFlow::new().unwrap();
    let backend = Rc::new(TestBackend);

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

    let current_task: Task = CurrentTask::get(&helixflow).get_task().try_into().unwrap();
    assert_eq!(current_task.name, "A valid task");
    assert_eq!(current_task.description, None);
    assert!(!current_task.id.is_nil());
    assert_eq!(current_task.id.get_version(), Some(uuid::Version::SortRand));

    let task_id_display = get!(&helixflow, "TaskBox::task_id_display");
    assert_eq!(
        task_id_display.accessible_value().unwrap(),
        current_task.id.to_string()
    );

    let create = get!(&helixflow, "TaskBox::create");
    assert!(helixflow.get_create_enabled());
    assert!(create.accessible_enabled().unwrap());
}
