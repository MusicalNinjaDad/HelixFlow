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
    use std::rc::Rc;

    use rstest::*;

    use i_slint_backend_testing::{ElementHandle, ElementRoot, init_no_event_loop};
    use slint::ComponentHandle;

    include!(concat!(env!("OUT_DIR"), "/src/tasks.rs"));

    macro_rules! get {
        ($component:expr, $id:expr) => {{
            let elements: Vec<_> = ElementHandle::find_by_element_id($component, $id).collect();
            assert_eq!(elements.len(), 1);
            elements.into_iter().next().unwrap()
        }};
    }

    mod ui_elements {
        use super::*;

        #[fixture]
        fn helixflow() -> HelixFlow {
            init_no_event_loop();
            let helixflow = HelixFlow::new().unwrap();

            // List all elements on test failure
            let all_elements =
                ElementHandle::query_descendants(&helixflow.root_element()).find_all();
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
            let inputboxes: Vec<_> =
                ElementHandle::find_by_element_type_name(&helixflow, "LineEdit").collect();
            let buttons: Vec<_> =
                ElementHandle::find_by_element_type_name(&helixflow, "Button").collect();

            assert_eq!(inputboxes.len(), 1);
            assert_eq!(buttons.len(), 1);
        }

        #[rstest]
        fn task_name(helixflow: HelixFlow) {
            let task_name = get!(&helixflow, "HelixFlow::task_name_entry");
            assert_eq!(task_name.accessible_label().unwrap().as_str(), "Task name");
            assert_eq!(
                task_name.accessible_placeholder_text().unwrap().as_str(),
                "Task name"
            );
            assert_eq!(task_name.accessible_value().unwrap().as_str(), "");
        }

        #[rstest]
        fn task_id(helixflow: HelixFlow) {
            let task_id = get!(&helixflow, "HelixFlow::task_id_display");
            assert_eq!(task_id.accessible_label().unwrap().as_str(), "Task ID");
            assert_eq!(task_id.accessible_value().unwrap().as_str(), "");
        }

        #[rstest]
        fn create(helixflow: HelixFlow) {
            let create = get!(&helixflow, "HelixFlow::create");
            assert_eq!(create.accessible_label().unwrap().as_str(), "Create");
        }
    }

    #[test]
    fn test_button_click() {
        init_no_event_loop();
        let helixflow = Rc::new(HelixFlow::new().unwrap());

        let all_elements = ElementHandle::query_descendants(&helixflow.root_element()).find_all();
        for (i, element) in all_elements.iter().enumerate() {
            let type_name = element.type_name();
            let label = element
                .accessible_label()
                .unwrap_or_else(|| "<no label>".into());
            let value = element
                .accessible_value()
                .unwrap_or_else(|| "<no value>".into());
            let id = element.id().unwrap_or_else(|| "<no ID>".into());
            println!(
                "Element {i}: id = {:#?}, type = {:#?}, label = {label}, value = {:#?}",
                id, type_name, value
            );
        }
        dbg!(all_elements.len());

        let things_called_create: Vec<_> =
            ElementHandle::find_by_accessible_label(helixflow.as_ref(), "Create").collect();
        assert_eq!(things_called_create.len(), 1);
        let create = &things_called_create[0];
        assert_eq!(create.type_name().unwrap().as_str(), "Button");

        let ids: Vec<_> =
            ElementHandle::find_by_element_id(helixflow.as_ref(), "HelixFlow::task_id_display")
                .collect();
        assert_eq!(ids.len(), 1);
        let id = &ids[0];

        let hf = helixflow.as_weak();
        helixflow.on_create_task(move || {
            hf.unwrap().set_task_id("1".into());
        });

        assert_eq!(id.accessible_value().unwrap().as_str(), "");
        create.invoke_accessible_default_action();
        assert_eq!(id.accessible_label().unwrap().as_str(), "Task ID");
        assert_eq!(id.accessible_value().unwrap().as_str(), "1");
    }
}
