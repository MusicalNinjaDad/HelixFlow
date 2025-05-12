use std::{
    panic::{self, PanicHookInfo},
    rc::Rc,
    sync::OnceLock,
};

use helixflow_core::task::{Task, blocking::CRUD};
use i_slint_backend_testing::ElementHandle;
use slint::ComponentHandle;
use slint::platform::PointerEventButton;

use helixflow_slint::{HelixFlow, blocking::create_task};
use helixflow_surreal::blocking::SurrealDb;

#[test]
fn test_slint_with_surreal() {
    // Slint's event_loop doesn't propogate panics from background tasks
    //   so we need to actively track if any occur.
    static PANICKED: OnceLock<bool> = OnceLock::new();
    static DEFAULT_HOOK: OnceLock<Box<dyn Fn(&PanicHookInfo) + Sync + Send + 'static>> =
        OnceLock::new();
    let _ = DEFAULT_HOOK.set(panic::take_hook());

    panic::set_hook(Box::new(|info| {
        DEFAULT_HOOK.get().unwrap()(info);
        let _ = PANICKED.set(true);
    }));

    let backend = Rc::new(SurrealDb::new().unwrap());

    i_slint_backend_testing::init_integration_test_with_system_time();

    let helixflow = HelixFlow::new().unwrap();

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

    assert!(PANICKED.get().is_none_or(|panicked| { !panicked }));

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
