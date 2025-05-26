#![feature(assert_matches)]

slint::include_modules!();

pub mod task;

/// Helper macros & re-exports to simplify testing: `use helixflow_slint::test::*`
pub mod test {
    pub use std::{
        panic::{self, PanicHookInfo},
        sync::OnceLock,
    };

    // TODO: Stick this module and following dependencies behind a feature flag.
    pub use assert_unordered::assert_eq_unordered_sort;
    pub use i_slint_backend_testing::{ElementHandle, ElementRoot};

    #[macro_export]
    #[doc(hidden)]
    /// Slint's event_loop doesn't propogate panics from background task so we create a custom panic
    /// handler to actively track if any occur before calling `init_integration_test_with_system_time`.
    ///
    /// Use `run_slint_loop!()` to run the even loop and then check for panics.
    macro_rules! prepare_slint {
        () => {
            static PANICKED: OnceLock<bool> = OnceLock::new();
            static DEFAULT_HOOK: OnceLock<Box<dyn Fn(&PanicHookInfo) + Sync + Send + 'static>> =
                OnceLock::new();
            let _ = DEFAULT_HOOK.set(panic::take_hook());

            panic::set_hook(Box::new(|info| {
                DEFAULT_HOOK.get().unwrap()(info);
                let _ = PANICKED.set(true);
            }));
            i_slint_backend_testing::init_integration_test_with_system_time();
        };
    }
    pub use prepare_slint;

    #[macro_export]
    #[doc(hidden)]
    /// Run the event loop and check whether anything within it `panic`ked...
    macro_rules! run_slint_loop {
        () => {
            slint::run_event_loop().unwrap();
            assert!(
                PANICKED.get().is_none_or(|panicked| { !panicked }) // just in case it was set to `false` for some reason
            );
        };
    }
    pub use run_slint_loop;

    #[macro_export]
    #[doc(hidden)]
    // List all slint elements to stdout (shown on test failure)
    macro_rules! list_elements {
        ($root:expr) => {
            let all_elements = ElementHandle::query_descendants(&$root.root_element()).find_all();
            for (i, element) in all_elements.iter().enumerate() {
                let elementid = element.id().unwrap_or_else(|| "<no ID>".into());
                let role = element.accessible_role();
                let type_name = element.type_name();
                let label = element
                    .accessible_label()
                    .unwrap_or_else(|| "<no label>".into());
                let value = element.accessible_value().unwrap_or_else(|| "<no value>".into());
                println!(
                    "Element {i}: id = {elementid}, role = {:#?}, type = {:#?}, label = {label}, value = {value}",
                    role, type_name
                );
            }
        };
    }
    pub use list_elements;

    #[macro_export]
    #[doc(hidden)]
    /// Get a component's `ElementHandle` via the _unique_ id: `get!(&parent, "Parent::id")`
    ///
    /// **Panics** if the id is not unique.
    macro_rules! get {
        ($component:expr, $id:expr) => {{
            let elements: Vec<_> = ElementHandle::find_by_element_id($component, $id).collect();
            assert_eq!(
                elements.len(),
                1,
                "{} elements found with id: {}",
                elements.len(),
                $id
            );
            elements.into_iter().next().unwrap()
        }};
    }
    pub use get;

    #[macro_export]
    #[doc(hidden)]
    /// Assert that the actual components match those expected based on the accessibility labels.
    ///
    /// This will _ignore_ any elements _without_ an accessibility label.
    ///
    /// ```rust,no_run
    /// let inputboxes: impl Iterator<Item = ElementHandle> = ElementHandle::find_by_element_type_name(&taskbox, "LineEdit");
    ///
    /// let expected_inputboxes = ["Task name"];
    ///
    /// assert_components!(inputboxes, expected_inputboxes);
    /// ```
    macro_rules! assert_components {
        ($actual:expr, $expected:expr) => {
            assert_eq_unordered_sort!(
                $actual
                    .filter_map(|element| element.accessible_label())
                    .collect::<Vec<_>>(),
                $expected
                    .iter()
                    .map(|label| label.to_shared_string())
                    .collect(),
                "`{}` does not match `{}`",
                stringify!($actual),
                stringify!($expected)
            );
        };
    }
    pub use assert_components;
}
