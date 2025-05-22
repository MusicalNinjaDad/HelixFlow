use helixflow_core::task::Task;
use std::rc::Weak;

use crate::HelixFlow;

pub mod blocking {
    use super::*;
    use helixflow_core::task::blocking::{CRUD, StorageBackend};

    pub fn create_task<BKEND>(
        helixflow: slint::Weak<HelixFlow>,
        backend: Weak<BKEND>,
    ) -> impl FnMut() + 'static
    where
        BKEND: StorageBackend + 'static,
    {
        move || {
            let helixflow = helixflow.unwrap();
            let backend = backend.upgrade().unwrap();
            helixflow.set_create_enabled(false);
            let task_name: String = helixflow.get_task_name().into();
            let task = Task::new(task_name, None);
            task.create(backend.as_ref()).unwrap();
            let task_id = task.id;
            helixflow.set_task_id(format!("{task_id}").into());
            helixflow.set_create_enabled(true);
        }
    }
}

#[cfg(test)]
mod test {
    use crate::test::*;
    use rstest::*;

    use i_slint_backend_testing::{ElementHandle, ElementRoot, init_no_event_loop};
    use slint::{ComponentHandle, SharedString};

    use helixflow_core::task::{Task, TaskCreationError, TaskResult};
    use uuid::{Uuid, uuid};

    include!(concat!(env!("OUT_DIR"), "/src/task.rs"));

    impl TryFrom<SlintTask> for Task {
        type Error = TaskCreationError;
        fn try_from(task: SlintTask) -> TaskResult<Task> {
            let name: String = task.name.into();
            let id = match Uuid::try_parse(task.id.as_str()) {
                Ok(id) => Ok(id),
                Err(_) => Err(TaskCreationError::InvalidID { id: task.id.into() })
            };
            Ok(Task {
                name: name.into(),
                id: id?,
                description: None
            })
        }
    }

    #[fixture]
    fn taskbox() -> TaskBox {
        init_no_event_loop();

        let taskbox = TaskBox::new().unwrap();
        list_elements!(&taskbox);
        taskbox
    }

    #[rstest]
    fn correct_elements(taskbox: TaskBox) {
        let inputboxes = ElementHandle::find_by_element_type_name(&taskbox, "LineEdit");
        let buttons = ElementHandle::find_by_element_type_name(&taskbox, "Button");

        let expected_inputboxes = ["Task name"];
        let expected_buttons = ["Create"];

        assert_components!(inputboxes, expected_inputboxes);
        assert_components!(buttons, expected_buttons);
    }

    #[rstest]
    fn slint_task_conversion() {
        let slint_task = SlintTask {
            name: SharedString::from("Task 1"),
            id: SharedString::from("0196b4c9-8447-7959-ae1f-72c7c8a3dd36"),
        };
        let task: Task = slint_task.try_into().unwrap();
        let expected_task = Task {
            name: "Task 1".into(),
            id: uuid!("0196b4c9-8447-7959-ae1f-72c7c8a3dd36"),
            description: None,
        };
        assert_eq!(task, expected_task);
    }

    mod accessibility {
        use i_slint_backend_testing::AccessibleRole;

        use super::*;

        #[rstest]
        fn task_name(taskbox: TaskBox) {
            let task_name = get!(&taskbox, "TaskBox::task_name_entry");
            assert_eq!(task_name.accessible_label().unwrap().as_str(), "Task name");
            assert_eq!(
                task_name.accessible_placeholder_text().unwrap().as_str(),
                "Task name"
            );
            assert_eq!(task_name.accessible_value().unwrap().as_str(), "");
            assert_eq!(task_name.accessible_role(), Some(AccessibleRole::TextInput));
        }

        #[rstest]
        fn task_id(taskbox: TaskBox) {
            let task_id = get!(&taskbox, "TaskBox::task_id_display");
            assert_eq!(task_id.accessible_label().unwrap().as_str(), "Task ID");
            assert_eq!(task_id.accessible_value().unwrap().as_str(), "");
            assert_eq!(task_id.accessible_role(), Some(AccessibleRole::Text));
        }

        #[rstest]
        fn create(taskbox: TaskBox) {
            let create = get!(&taskbox, "TaskBox::create");
            assert_eq!(create.accessible_label().unwrap().as_str(), "Create");
            assert_eq!(create.accessible_role(), Some(AccessibleRole::Button));
        }
    }

    mod callbacks {
        use super::*;

        #[rstest]
        fn button_click(taskbox: TaskBox) {
            let hf = taskbox.as_weak();
            taskbox.on_create_task(move || {
                hf.unwrap().set_task_id("1".into());
            });

            let create = get!(&taskbox, "TaskBox::create");
            let task_id = get!(&taskbox, "TaskBox::task_id_display");

            assert_eq!(task_id.accessible_value().unwrap().as_str(), "");
            create.invoke_accessible_default_action();
            assert_eq!(task_id.accessible_label().unwrap().as_str(), "Task ID");
            assert_eq!(task_id.accessible_value().unwrap().as_str(), "1");
        }
    }
}
