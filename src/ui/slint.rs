use slint::slint;

slint! {
    import { Button } from "std-widgets.slint";
    export component HelixFlow inherits Window {
        VerticalLayout {
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
        let elements: Vec<_> =
            ElementHandle::find_by_element_type_name(&helixflow, "Button").collect();
        assert_eq!(elements.len(), 1);
        let create = &elements[0];
        assert_eq!(create.accessible_label().unwrap().as_str(), "Create");
    }
}
