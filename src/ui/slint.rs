use slint::slint;

slint! {
    export component HelixFlow inherits Window {

    }
}

#[cfg(test)]
mod test {
    use i_slint_backend_testing::init_no_event_loop;

    use super::*;

    #[test]
    fn test_ui_elements() {
        init_no_event_loop();
        let _helixflow = HelixFlow::new().unwrap();
    }
}
