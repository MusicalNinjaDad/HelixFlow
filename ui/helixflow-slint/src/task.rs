use helixflow_core::task::{HelixFlowError, HelixFlowResult, Task, TaskList};
use slint::{Global, ModelRc, SharedString, ToSharedString};
use std::{fmt::Display, rc::Weak};
use uuid::Uuid;

use crate::{Backlog, CurrentTask, HelixFlow, SlintTask, SlintTaskList};

impl TryFrom<SlintTask> for Task {
    type Error = HelixFlowError;
    fn try_from(task: SlintTask) -> HelixFlowResult<Task> {
        Ok(if task.id.is_empty() {
            Task::new(task.name.to_string(), None)
        } else {
            Task {
                name: task.name.to_string().into(),
                id: Uuid::try_parse(task.id.as_str())
                    .map_err(|_| HelixFlowError::InvalidID { id: task.id.into() })?,
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

impl From<SlintTask> for SharedString {
    fn from(task: SlintTask) -> Self {
        task.name
    }
}

impl Display for SlintTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl From<TaskList> for SlintTaskList {
    fn from(tasklist: TaskList) -> Self {
        Self {
            name: tasklist.name.to_shared_string(),
            id: tasklist.id.to_shared_string(),
        }
    }
}

impl TryFrom<SlintTaskList> for TaskList {
    type Error = HelixFlowError;
    fn try_from(tasklist: SlintTaskList) -> HelixFlowResult<Self> {
        Ok(if tasklist.id.is_empty() {
            TaskList::new(tasklist.name.to_string())
        } else {
            TaskList {
                name: tasklist.name.to_string().into(),
                id: Uuid::try_parse(tasklist.id.as_str()).map_err(|_| {
                    HelixFlowError::InvalidID {
                        id: tasklist.id.into(),
                    }
                })?,
            }
        })
    }
}

pub trait BacklogSignature {
    fn get_tasklist(&self) -> SlintTaskList;
    fn set_tasks(&self, model: ModelRc<SlintTask>);
}

impl BacklogSignature for Backlog {
    fn get_tasklist(&self) -> SlintTaskList {
        self.get_tasklist()
    }
    fn set_tasks(&self, model: ModelRc<SlintTask>) {
        self.set_tasks(model);
    }
}

pub mod blocking {
    use crate::Backlog;

    use super::*;
    use helixflow_core::task::{
        Contains, TaskList,
        blocking::{CRUD, Linkable, Relate, Store},
    };
    use slint::{ComponentHandle, ModelRc, VecModel};

    pub fn create_task<BKEND>(
        helixflow: slint::Weak<HelixFlow>,
        backend: Weak<BKEND>,
    ) -> impl FnMut() + 'static
    where
        BKEND: Store<Task> + 'static,
    {
        move || {
            let helixflow = helixflow.unwrap();
            let backend = backend.upgrade().unwrap();
            helixflow.set_create_enabled(false);
            let task_name: String = helixflow.get_task_name().into();
            let task = Task::new(task_name, None);
            task.create(backend.as_ref()).unwrap();
            CurrentTask::get(&helixflow).set_task(task.into());
            helixflow.set_create_enabled(true);
        }
    }

    pub fn load_backlog<ROOT, BKEND>(
        root_component: slint::Weak<ROOT>,
        backend: Weak<BKEND>,
    ) -> impl FnMut() + 'static
    where
        BKEND: Relate<Contains<TaskList, Task>> + 'static,
        ROOT: ComponentHandle + BacklogSignature + 'static,
    {
        move || {
            let root_component = root_component.unwrap();
            let backend = backend.upgrade().unwrap();
            let tasklist = root_component.get_tasklist();
            let tl = TaskList::try_from(tasklist).unwrap();
            let backlog_entries: VecModel<SlintTask> = tl
                .get_linked_items(backend.as_ref())
                .unwrap()
                .map(|task| task.right.unwrap().into())
                .collect();
            root_component.set_tasks(ModelRc::new(backlog_entries));
        }
    }

    impl Backlog {
        pub fn init<BKEND>(&self, backend: Weak<BKEND>, id: &Uuid)
        where
            BKEND: Store<TaskList> + Relate<Contains<TaskList, Task>> + 'static,
        {
            let backend = backend.upgrade().unwrap();
            let contents = TaskList::get(backend.as_ref(), id).unwrap();
            let backlog_entries: VecModel<SlintTask> = contents
                .get_linked_items(backend.as_ref())
                .unwrap()
                .map(|task| task.right.unwrap().into())
                .collect();
            // self.set_backlog_name(contents.name.into_owned().into());
            self.set_tasks(ModelRc::new(backlog_entries));
        }
    }
}

#[cfg(test)]
mod test_rs {
    use super::*;

    use rstest::*;

    use std::assert_matches::assert_matches;
    use uuid::uuid;

    #[rstest]
    fn task_no_id() {
        let slint_task = SlintTask {
            name: "Task 1".into(),
            id: "".into(),
        };
        let task: Task = slint_task.try_into().unwrap();
        assert_eq!(task.name, "Task 1");
        assert!(!task.id.is_nil());
        assert_eq!(task.description, None);
    }

    #[rstest]
    fn task_with_id() {
        let slint_task = SlintTask {
            name: "Task 1".into(),
            id: "0196b4c9-8447-7959-ae1f-72c7c8a3dd36".into(),
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
        let slint_task = SlintTask {
            name: "Task 1".into(),
            id: "foo".into(),
        };
        let task: HelixFlowResult<Task> = slint_task.try_into();
        let err = task.unwrap_err();
        assert_matches!(err, HelixFlowError::InvalidID {id} if id == "foo");
    }

    #[rstest]
    fn from_task() {
        let task = Task {
            name: "Task 1".into(),
            id: uuid!("0196b4c9-8447-7959-ae1f-72c7c8a3dd36"),
            description: None,
        };
        let slint_task = SlintTask {
            name: "Task 1".into(),
            id: "0196b4c9-8447-7959-ae1f-72c7c8a3dd36".into(),
        };
        assert_eq!(slint_task, task.into());
    }
}

#[cfg(test)]
mod test_slint {
    use super::*;
    use crate::test::*;
    use rstest::*;

    use i_slint_backend_testing::init_no_event_loop;
    use slint::ComponentHandle;

    mod taskbox {
        use super::*;
        use crate::TaskBox;

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

        mod callbacks {
            use super::*;
            use slint::Global;

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

    mod backlog {
        use slint::{ModelRc, VecModel};

        use super::*;
        use crate::Backlog;

        #[fixture]
        fn backlog() -> Backlog {
            init_no_event_loop();

            let backlog = Backlog::new().unwrap();
            list_elements!(&backlog);
            backlog
        }

        #[rstest]
        fn correct_elements(backlog: Backlog) {
            let texts = ElementHandle::find_by_element_type_name(&backlog, "Text");
            let expected_texts = ["Backlog name"];
            assert_components!(texts, expected_texts);

            let inputboxes = ElementHandle::find_by_element_type_name(&backlog, "LineEdit");
            let expected_inputboxes = ["New task name"];
            assert_components!(inputboxes, expected_inputboxes);

            let buttons = ElementHandle::find_by_element_type_name(&backlog, "Button");
            let expected_buttons = ["Create new task"];
            assert_components!(buttons, expected_buttons);

            let lists = ElementHandle::find_by_element_type_name(&backlog, "ListView");
            let expected_lists = ["Tasks"];
            assert_components!(lists, expected_lists);

            let tasks = ElementHandle::find_by_element_type_name(&backlog, "TaskListItem");
            let expected_task_labels = ["Task 1", "Task 2"];
            assert_components!(tasks, expected_task_labels);

            let tasks = ElementHandle::find_by_element_type_name(&backlog, "TaskListItem");
            let expected_task_values = ["Error loading tasks", "from database"];
            assert_values!(tasks, expected_task_values);
        }

        #[rstest]
        fn show_tasks(backlog: Backlog) {
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
            backlog.set_tasks(ModelRc::new(backlog_entries));
            list_elements!(&backlog);
            let backlog_tasks = ElementHandle::find_by_element_type_name(&backlog, "TaskListItem");
            assert_values!(backlog_tasks, &tasks);
        }

        #[rstest]
        fn quick_create(backlog: Backlog) {
            let bl = backlog.as_weak();
            backlog.on_quick_create_task(move |mut task: SlintTask| {
                task.id = "1".into();
                bl.unwrap().set_tasklist(SlintTaskList {
                    name: format!("{}: {}", task.id, task.name).into(),
                    id: "2".into(),
                });
            });
            let backlog_title = get!(&backlog, "Backlog::backlog_title");
            assert_eq!(
                backlog_title.accessible_value().unwrap().as_str(),
                "Backlog"
            );
            let new_task_entry = get!(&backlog, "Backlog::new_task_entry");
            new_task_entry.set_accessible_value("New task");
            let quick_create = get!(&backlog, "Backlog::quick_create_button");
            quick_create.invoke_accessible_default_action();
            assert_eq!(
                backlog_title.accessible_value().unwrap().as_str(),
                "1: New task"
            );
        }
    }
}
