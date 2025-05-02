use helixflow::task::{Task,TestBackend};
use helixflow::ui::slint::HelixFlow;
use slint::ComponentHandle;
use tokio::runtime;

#[test]
fn test_set_task_id() {
    i_slint_backend_testing::init_integration_test_with_system_time();

    slint::spawn_local(async move {
        let helixflow = HelixFlow::new().unwrap();
        let backend = TestBackend;
        let rt = runtime::Builder::new_current_thread().enable_all().build().unwrap();
        let hf = helixflow.as_weak();
        helixflow.on_create_task(move || {
            let helixflow = hf.unwrap();
            let task_name: String = helixflow.get_task_name().into();
            let mut task = Task::<u32> {
                name: task_name.into(),
                description: None,
                id: None
            };
            rt.block_on(task.create(&backend)).unwrap();
            let task_id = task.id.unwrap();
            helixflow.set_task_id(format!("{task_id}").into());
        });
        helixflow.set_task_name("A valid task".into());
        assert_eq!(helixflow.get_task_id(), "");
        helixflow.invoke_create_task();
        assert_eq!(helixflow.get_task_id(), "1");
        slint::quit_event_loop().unwrap();
    }).unwrap();

    slint::run_event_loop().unwrap();
}