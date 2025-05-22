slint::include_modules!();

pub mod task;

#[cfg(test)]
/// Helper macros to simplify testing: `use helixflow_slint::test::*`
pub mod test {
    pub use assert_unordered::assert_eq_unordered_sort;

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
                    .map(|element| element.accessible_label().unwrap())
                    .collect::<Vec<_>>(),
                $expected.iter().map(|&label| label.into()).collect()
            );
        };
    }
    pub use assert_components;
}
