use slint::slint;

slint! {
    import { Button, LineEdit, VerticalBox } from "std-widgets.slint";
    export component HelixFlow inherits Window {
        VerticalBox {
            LineEdit {
                placeholder-text: "Task name";
            }
            id := Text {
                text: "None";
            }
            Button {
                text: "Create";
            }
        }
    }
}

#[cfg(test)]
mod test {
    use i_slint_backend_testing::{init_no_event_loop, ElementHandle, ElementRoot};

    use super::*;

    #[test]
    fn test_ui_elements() {
        init_no_event_loop();
        let helixflow = HelixFlow::new().unwrap();
        let all_elements = ElementHandle::query_descendants(&helixflow.root_element()).find_all();
        for (i, element) in all_elements.iter().enumerate() {
            let type_name = element.clone().type_name();
            let label = element.clone().accessible_label().unwrap_or_else(|| "<no label>".into());
            println!("Element {i}: type = {:#?}, label = {label}", type_name);
        }
        dbg!(all_elements.len());
        let buttons: Vec<_> =
            ElementHandle::find_by_element_type_name(&helixflow, "Button").collect();
        assert_eq!(buttons.len(), 1);
        let create = &buttons[0];
        assert_eq!(create.accessible_label().unwrap().as_str(), "Create");
        let ids: Vec<_> = ElementHandle::find_by_element_id(&helixflow, "id").collect();
        assert_eq!(ids.len(), 1);
        let id = &ids[0];
        assert_eq!(id.accessible_value().unwrap().as_str(), "None");
        let inputboxes: Vec<_> = ElementHandle::find_by_element_type_name(&helixflow, "LineEdit").collect();
        assert_eq!(inputboxes.len(), 1);
        let task_name = &inputboxes[0];
        assert_eq!(task_name.accessible_value().unwrap().as_str(), "Task name");
    }
}
