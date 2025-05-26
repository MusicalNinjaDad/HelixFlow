use helixflow_slint::{Backlog, SlintTask, test::*};
use slint::{ComponentHandle, ModelRc, VecModel};

#[test]
fn update_tasks_in_event_loop() {
    prepare_slint!();
    let backlog = Backlog::new().unwrap();
    list_elements!(&backlog);
    let task1 = SlintTask {
        name: "Test task 1".into(),
        id: "1".into(),
    };
    let task2 = SlintTask {
        name: "Test task 2".into(),
        id: "2".into(),
    };
    let tasks = vec![task1, task2];
    let backlog_entries: VecModel<SlintTask> = tasks.clone().into();
    let bl = backlog.as_weak();
    slint::spawn_local(async move {
        let backlog = bl.unwrap();
        backlog.set_tasks(ModelRc::new(backlog_entries));
        slint::quit_event_loop().unwrap();
    })
    .unwrap();
    run_slint_loop!();
    println!("===Loop finished===");
    list_elements!(&backlog);
    let backlog_tasks = ElementHandle::find_by_element_type_name(&backlog, "TaskListItem");
    assert_values!(backlog_tasks, tasks);
}
