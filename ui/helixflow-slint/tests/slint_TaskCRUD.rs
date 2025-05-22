//! Use nextest - these tests will fail on cargo test as they MUST run is separate processes
//! for `i_slint_backend_testing::init_integration_test_with_system_time`

use std::rc::Rc;

use i_slint_backend_testing::ElementHandle;
use slint::ComponentHandle;
use slint::platform::PointerEventButton;

use helixflow_core::task::blocking::TestBackend;
use helixflow_slint::{HelixFlow, task::blocking::create_task, test::*};

#[test]
fn test_set_task_id() {
    use i_slint_backend_testing::ElementRoot;
    prepare_slint!();

    let helixflow = HelixFlow::new().unwrap();
    let backend = Rc::new(TestBackend);

    // List all elements on test failure
    let all_elements = ElementHandle::query_descendants(&helixflow.root_element()).find_all();
    for (i, element) in all_elements.iter().enumerate() {
        let type_name = element.type_name();
        let label = element
            .accessible_label()
            .unwrap_or_else(|| "<no label>".into());
        let elementid = element.id().unwrap_or_else(|| "<no ID>".into());
        println!(
            "Element {i}: id = {elementid}, type = {:#?}, label = {label}",
            type_name
        );
    }
    dbg!(all_elements.len());

    let hf = helixflow.as_weak();
    let be = Rc::downgrade(&backend);
    helixflow.on_create_task(create_task(hf, be));

    let hf = helixflow.as_weak();

    slint::spawn_local(async move {
        let helixflow = hf.unwrap();
        helixflow.set_task_name("A valid task".into());
        assert_eq!(helixflow.get_task_id(), "");
        assert!(helixflow.get_create_enabled());

        let creates_: Vec<_> =
            ElementHandle::find_by_element_id(&helixflow, "TaskBox::create").collect();
        assert_eq!(creates_.len(), 1);
        let create = &creates_[0];

        create.single_click(PointerEventButton::Left).await;
        slint::quit_event_loop().unwrap();
    })
    .unwrap();

    run_slint_loop!();

    let task_uuid = uuid::Uuid::parse_str(&helixflow.get_task_id()).unwrap();
    assert!(!task_uuid.is_nil());
    assert_eq!(task_uuid.get_version(), Some(uuid::Version::SortRand));
    assert!(helixflow.get_create_enabled());
}
