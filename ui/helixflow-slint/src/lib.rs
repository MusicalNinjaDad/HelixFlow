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

    use i_slint_backend_testing::{ElementHandle, ElementRoot, init_no_event_loop};

    include!(concat!(env!("OUT_DIR"), "/tasks.rs"));

    #[test]
    fn test_ui_elements() {
        init_no_event_loop();
        let helixflow = HelixFlow::new().unwrap();

        let all_elements = ElementHandle::query_descendants(&helixflow.root_element()).find_all();
        for (i, element) in all_elements.iter().enumerate() {
            let type_name = element.type_name();
            let label = element
                .accessible_label()
                .unwrap_or_else(|| "<no label>".into());
            println!("Element {i}: type = {:#?}, label = {label}", type_name);
        }
        dbg!(all_elements.len());

        let buttons: Vec<_> =
            ElementHandle::find_by_element_type_name(&helixflow, "Button").collect();
        assert_eq!(buttons.len(), 1);
        let create = &buttons[0];
        assert_eq!(create.accessible_label().unwrap().as_str(), "Create");

        let ids: Vec<_> =
            ElementHandle::find_by_element_id(&helixflow, "HelixFlow::task_id_display").collect();
        assert_eq!(ids.len(), 1);
        let id = &ids[0];
        assert_eq!(id.accessible_label().unwrap().as_str(), "Task ID");
        assert_eq!(id.accessible_value().unwrap().as_str(), "");

        let inputboxes: Vec<_> =
            ElementHandle::find_by_element_type_name(&helixflow, "LineEdit").collect();
        assert_eq!(inputboxes.len(), 1);
        let task_name = &inputboxes[0];
        assert_eq!(task_name.accessible_label().unwrap().as_str(), "Task name");
        assert_eq!(
            task_name.accessible_placeholder_text().unwrap().as_str(),
            "Task name"
        );
        assert_eq!(task_name.accessible_value().unwrap().as_str(), "");
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
