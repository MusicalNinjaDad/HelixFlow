use helixflow::task::{Task, TestBackend};
use helixflow::ui::slint::HelixFlow;
use i_slint_backend_testing::ElementHandle;
use slint::platform::PointerEventButton;
use slint::ComponentHandle;

#[test]
fn test_set_task_id() {
    i_slint_backend_testing::init_integration_test_with_system_time();

    let helixflow = HelixFlow::new().unwrap();
    let backend = TestBackend;
    let hf = helixflow.as_weak();

    dbg!("Setting callback");
    helixflow.on_create_task({
        dbg!("Begin Callback");
        move || {
            dbg!("Begin Callback main closure");
            let helixflow = hf.unwrap();
            helixflow.set_create_enabled(false);
            dbg!("Button disabled");
            assert!(!helixflow.get_create_enabled());
            let task_name: String = helixflow.get_task_name().into();
            dbg!(&task_name);
            let mut task = Task::<u32> {
                name: task_name.into(),
                description: None,
                id: None,
            };
            dbg!("Requesting task creation ...");
            slint::spawn_local(async_compat::Compat::new(async move {
                dbg!("Begin async block to get task id ...");
                task.create(&backend).await.unwrap();
                let task_id = task.id.unwrap();
                dbg!(&task_id);
                helixflow.set_task_id(format!("{task_id}").into());
                helixflow.set_create_enabled(true);
                dbg!("Button enabled");
                slint::quit_event_loop().unwrap();
                dbg!("End async call to get task id ...");
            }))
            .unwrap();
            dbg!("End callback main closure");
        }
    });
    dbg!("Callback set");

    let hf = helixflow.as_weak();

    dbg!("Spawning test...");
    slint::spawn_local(async move {
        dbg!("Starting test");
        let helixflow = hf.unwrap();
        helixflow.set_task_name("A valid task".into());
        dbg!(helixflow.get_task_name());
        dbg!(helixflow.get_task_id());
        dbg!(helixflow.get_create_enabled());
        assert_eq!(helixflow.get_task_id(), "");
        assert!(helixflow.get_create_enabled());

        let creates_: Vec<_> =
            ElementHandle::find_by_element_id(&helixflow, "HelixFlow::create").collect();
        assert_eq!(creates_.len(), 1);
        let create = &creates_[0];
        
        dbg!("Clicking button...");
        // create.invoke_accessible_default_action();
        create.single_click(PointerEventButton::Left).await;
        dbg!("Button clicked");
    })
    .unwrap();

    dbg!("Running event loop");
    slint::run_event_loop().unwrap();
    dbg!("Evnt loop unwrapped");
    dbg!(helixflow.get_task_id());
    dbg!(helixflow.get_create_enabled());
    assert_eq!(helixflow.get_task_id(), "1");
    assert!(helixflow.get_create_enabled());
    // panic!();
}

// [tests/ui_integration.rs:15:5] "Setting callback" = "Setting callback"
// [tests/ui_integration.rs:17:9] "Begin Callback" = "Begin Callback"
// [tests/ui_integration.rs:47:5] "Callback set" = "Callback set"
// [tests/ui_integration.rs:51:5] "Spawning test..." = "Spawning test..."
// [tests/ui_integration.rs:74:5] "Running event loop" = "Running event loop"
// [tests/ui_integration.rs:53:9] "Starting test" = "Starting test"
// [tests/ui_integration.rs:56:9] helixflow.get_task_name() = "A valid task"
// [tests/ui_integration.rs:57:9] helixflow.get_task_id() = ""
// [tests/ui_integration.rs:58:9] helixflow.get_create_enabled() = true
// [tests/ui_integration.rs:67:9] "Clicking button..." = "Clicking button..."
// [tests/ui_integration.rs:19:13] "Begin Callback main closure" = "Begin Callback main closure"
// [tests/ui_integration.rs:22:13] "Button disabled" = "Button disabled"
// [tests/ui_integration.rs:25:13] &task_name = "A valid task"
// [tests/ui_integration.rs:31:13] "Requesting task creation ..." = "Requesting task creation ..."
// [tests/ui_integration.rs:44:13] "End callback main closure" = "End callback main closure"
// [tests/ui_integration.rs:70:9] "Button clicked" = "Button clicked"
// [tests/ui_integration.rs:33:17] "Begin async block to get task id ..." = "Begin async block to get task id ..."
// [tests/ui_integration.rs:36:17] &task_id = 1
// [tests/ui_integration.rs:39:17] "Button enabled" = "Button enabled"
// [tests/ui_integration.rs:41:17] "End async call to get task id ..." = "End async call to get task id ..."
// [tests/ui_integration.rs:76:5] "Evnt loop unwrapped" = "Evnt loop unwrapped"
// [tests/ui_integration.rs:77:5] helixflow.get_task_id() = "1"
// [tests/ui_integration.rs:78:5] helixflow.get_create_enabled() = true