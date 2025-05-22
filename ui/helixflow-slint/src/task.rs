use helixflow_core::task::{Task, TaskCreationError, TaskResult};
use slint::ToSharedString;
use std::rc::Weak;
use uuid::Uuid;

use crate::{HelixFlow, SlintTask};

impl TryFrom<SlintTask> for Task {
    type Error = TaskCreationError;
    fn try_from(task: SlintTask) -> TaskResult<Task> {
        let name: String = task.name.into();
        Ok(if task.id.is_empty() {
            Task::new(name, None)
        } else {
            let id = match Uuid::try_parse(task.id.as_str()) {
                Ok(id) => Ok(id),
                Err(_) => Err(TaskCreationError::InvalidID { id: task.id.into() }),
            };
            Task {
                name: name.into(),
                id: id?,
                description: None,
            }
        })
    }
}

impl From<Task> for SlintTask {
    fn from(task: Task) -> Self {
        Self {
            name: task.name.into_owned().into(),
            id: task.id.to_shared_string(),
        }
    }
}

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
    use super::*;
    use crate::test::*;
    use rstest::*;
    use std::assert_matches::assert_matches;

    use i_slint_backend_testing::{ElementHandle, ElementRoot, init_no_event_loop};
    use slint::{ComponentHandle, SharedString};

    use uuid::uuid;

    include!(concat!(env!("OUT_DIR"), "/src/task.rs"));

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

    mod slint_task {
        use super::*;

        #[rstest]
        fn task_no_id() {
            let slint_task = crate::SlintTask {
                name: SharedString::from("Task 1"),
                id: SharedString::from(""),
            };
            let task: Task = slint_task.try_into().unwrap();
            assert_eq!(task.name, "Task 1");
            assert!(!task.id.is_nil());
            assert_eq!(task.description, None);
        }

        #[rstest]
        fn task_with_id() {
            let slint_task = crate::SlintTask {
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

        #[rstest]
        fn task_invalid_id() {
            let slint_task = crate::SlintTask {
                name: SharedString::from("Task 1"),
                id: SharedString::from("foo"),
            };
            let task: TaskResult<Task> = slint_task.try_into();
            let err = task.unwrap_err();
            assert_matches!(err, TaskCreationError::InvalidID {id} if id == "foo");
        }

        #[rstest]
        fn from_task() {
            let task = Task {
                name: "Task 1".into(),
                id: uuid!("0196b4c9-8447-7959-ae1f-72c7c8a3dd36"),
                description: None,
            };
            let slint_task = crate::SlintTask {
                name: SharedString::from("Task 1"),
                id: SharedString::from("0196b4c9-8447-7959-ae1f-72c7c8a3dd36"),
            };
            assert_eq!(slint_task, task.into());
        }
    }

    mod callbacks {
        use super::*;

        #[rstest]
        fn button_click(taskbox: TaskBox) {
            let tb = taskbox.as_weak();
            taskbox.on_create_task(move || {
                CurrentTask::get(&tb.unwrap()).set_task(SlintTask {
                    name: "".into(),
                    id: "1".into(),
                });
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
