use helixflow_core::task::Task;
use std::rc::Weak;

slint::include_modules!();

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
    use assert_unordered::assert_eq_unordered_sort;
    use rstest::*;

    use i_slint_backend_testing::{ElementHandle, ElementRoot, init_no_event_loop};
    use slint::ComponentHandle;

    include!(concat!(env!("OUT_DIR"), "/src/tasks.rs"));

    macro_rules! get {
        ($component:expr, $id:expr) => {{
            let elements: Vec<_> = ElementHandle::find_by_element_id($component, $id).collect();
            assert_eq!(elements.len(), 1, "{} elements found with id: {}", elements.len(), $id);
            elements.into_iter().next().unwrap()
        }};
    }

    macro_rules! assert_components {
        ($actual:expr, $expected:expr) => {
            assert_eq_unordered_sort!(
                $actual
                    .map(|element| element.accessible_label().unwrap())
                    .collect::<Vec<_>>(),
                $expected.iter().map(|&label| label.into()).collect()
            );
        };
    }

    #[fixture]
    fn helixflow() -> HelixFlow {
        init_no_event_loop();
        let helixflow = HelixFlow::new().unwrap();

        // List all elements on test failure
        let all_elements = ElementHandle::query_descendants(&helixflow.root_element()).find_all();
        for (i, element) in all_elements.iter().enumerate() {
            let type_name = element.type_name();
            let label = element
                .accessible_label()
                .unwrap_or_else(|| "<no label>".into());
            println!("Element {i}: type = {:#?}, label = {label}", type_name);
        }
        dbg!(all_elements.len());

        helixflow
    }

    #[rstest]
    fn correct_elements(helixflow: HelixFlow) {
        let inputboxes = ElementHandle::find_by_element_type_name(&helixflow, "LineEdit");
        let buttons = ElementHandle::find_by_element_type_name(&helixflow, "Button");

        let expected_inputboxes = ["Task name"];
        let expected_buttons = ["Create"];

        assert_components!(inputboxes, expected_inputboxes);
        assert_components!(buttons, expected_buttons);
    }

    mod accessibility {
        use i_slint_backend_testing::AccessibleRole;

        use super::*;

        #[rstest]
        fn task_name(helixflow: HelixFlow) {
            let task_name = get!(&helixflow, "HelixFlow::task_name_entry");
            assert_eq!(task_name.accessible_label().unwrap().as_str(), "Task name");
            assert_eq!(
                task_name.accessible_placeholder_text().unwrap().as_str(),
                "Task name"
            );
            assert_eq!(task_name.accessible_value().unwrap().as_str(), "");
            assert_eq!(task_name.accessible_role(), Some(AccessibleRole::TextInput));
        }

        #[rstest]
        fn task_id(helixflow: HelixFlow) {
            let task_id = get!(&helixflow, "HelixFlow::task_id_display");
            assert_eq!(task_id.accessible_label().unwrap().as_str(), "Task ID");
            assert_eq!(task_id.accessible_value().unwrap().as_str(), "");
            assert_eq!(task_id.accessible_role(), Some(AccessibleRole::Text));
        }

        #[rstest]
        fn create(helixflow: HelixFlow) {
            let create = get!(&helixflow, "HelixFlow::create");
            assert_eq!(create.accessible_label().unwrap().as_str(), "Create");
            assert_eq!(create.accessible_role(), Some(AccessibleRole::Button));
        }
    }

    mod callbacks {
        use super::*;

        #[rstest]
        fn button_click(helixflow: HelixFlow) {
            let hf = helixflow.as_weak();
            helixflow.on_create_task(move || {
                hf.unwrap().set_task_id("1".into());
            });

            let create = get!(&helixflow, "HelixFlow::create");
            let task_id = get!(&helixflow, "HelixFlow::task_id_display");

            assert_eq!(task_id.accessible_value().unwrap().as_str(), "");
            create.invoke_accessible_default_action();
            assert_eq!(task_id.accessible_label().unwrap().as_str(), "Task ID");
            assert_eq!(task_id.accessible_value().unwrap().as_str(), "1");
        }
    }
}
