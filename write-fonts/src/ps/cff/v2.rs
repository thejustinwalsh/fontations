//! CFF version 2.

include!("../../../generated/generated_cff2.rs");

impl Index {
    /// Construct an `Index` from a list of byte items.
    pub fn from_items(items: Vec<Vec<u8>>) -> Self {
        if items.is_empty() {
            // The spec ends an empty INDEX after its count, which this cannot
            // express: <https://github.com/googlefonts/fontations/issues/1719>
            return Index::new(0, 1, vec![1], vec![]);
        }

        let count = items.len() as u32;
        let (off_size, offsets, data) = super::index_parts(items);
        Index::new(count, off_size, offsets, data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index2_from_items() {
        let items = vec![vec![1, 2, 3], vec![4, 5]];
        let index = Index::from_items(items);
        assert_eq!(index.count, 2);
    }
}
