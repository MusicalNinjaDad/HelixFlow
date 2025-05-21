slint::include_modules!();

pub mod task;

#[cfg(test)]
pub mod test {
    pub use assert_unordered::assert_eq_unordered_sort;

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
    pub(crate) use get;

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
    pub(crate) use assert_components;
}
