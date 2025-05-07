use std::rc::Rc;

use i_slint_backend_testing::ElementHandle;
use slint::ComponentHandle;
use slint::platform::PointerEventButton;

use helixflow_core::task::blocking::TestBackend;
use helixflow_slint::{HelixFlow, blocking::create_task};

#[test]
fn test_set_task_id() {
    i_slint_backend_testing::init_integration_test_with_system_time();

    let helixflow = HelixFlow::new().unwrap();
    let backend = Rc::new(TestBackend);

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
            ElementHandle::find_by_element_id(&helixflow, "HelixFlow::create").collect();
        assert_eq!(creates_.len(), 1);
        let create = &creates_[0];

        create.single_click(PointerEventButton::Left).await;
        slint::quit_event_loop().unwrap();
    })
    .unwrap();

    slint::run_event_loop().unwrap();
    assert_eq!(helixflow.get_task_id(), "1");
    assert!(helixflow.get_create_enabled());
}
