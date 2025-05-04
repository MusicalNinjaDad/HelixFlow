use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use tokio::task::block_in_place;
use tokio::test;

use i_slint_backend_testing::ElementHandle;
use slint::platform::PointerEventButton;
use slint::{ComponentHandle, JoinHandle};

use helixflow::backends::surreal::SurrealDb;
use helixflow::ui::slint::{HelixFlow, create_task};

#[test(flavor = "multi_thread")]
async fn test_slint_with_surreal() {
    let backend = Arc::new(SurrealDb::create().await.unwrap());

    i_slint_backend_testing::init_integration_test_with_system_time();

    let helixflow = HelixFlow::new().unwrap();
    let task_creation_request: Rc<RefCell<Option<JoinHandle<()>>>> = Rc::new(RefCell::new(None));

    let hf = helixflow.as_weak();
    let tcr = Rc::downgrade(&task_creation_request);
    helixflow.on_create_task(create_task(hf, tcr, backend));

    let hf = helixflow.as_weak();
    let tcr = Rc::downgrade(&task_creation_request);
    slint::spawn_local(async move {
        let helixflow = hf.unwrap();
        let task_creation_request = tcr.upgrade().unwrap();
        helixflow.set_task_name("A valid task".into());
        assert_eq!(helixflow.get_task_id(), "");
        assert!(helixflow.get_create_enabled());

        let creates_: Vec<_> =
            ElementHandle::find_by_element_id(&helixflow, "HelixFlow::create").collect();
        assert_eq!(creates_.len(), 1);
        let create = &creates_[0];

        create.single_click(PointerEventButton::Left).await;
        assert!(!helixflow.get_create_enabled());
        task_creation_request.borrow_mut().take().unwrap().await;
        slint::quit_event_loop().unwrap();
    })
    .unwrap();

    block_in_place(slint::run_event_loop).unwrap();

    assert!(helixflow.get_task_id().starts_with("Tasks:"));
    assert!(helixflow.get_create_enabled());
}
