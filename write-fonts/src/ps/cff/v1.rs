//! CFF version 1.

include!("../../../generated/generated_cff.rs");

impl Index {
    /// Construct an `Index` from a list of byte items.
    ///
    /// A `CFF` INDEX counts its objects in a `u16`, so it holds at most
    /// [`u16::MAX`] of them.
    pub fn from_items(items: Vec<Vec<u8>>) -> Self {
        if items.is_empty() {
            // The spec ends an empty INDEX after its count, which this cannot
            // express: <https://github.com/googlefonts/fontations/issues/1719>
            return Index::new(0, 1, vec![1], vec![]);
        }

        debug_assert!(items.len() <= u16::MAX as usize, "INDEX count overflow");
        let count = items.len() as u16;
        let (off_size, offsets, data) = super::index_parts(items);
        Index::new(count, off_size, offsets, data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use read_fonts::ps::cff::index::Index as ReadIndex;

    fn round_trip(items: Vec<Vec<u8>>) -> Vec<u8> {
        let expected = items.clone();
        let bytes = crate::dump_table(&Index::from_items(items)).unwrap();
        let index = ReadIndex::new(&bytes, false).unwrap();
        assert_eq!(index.count() as usize, expected.len());
        for (i, item) in expected.iter().enumerate() {
            assert_eq!(index.get(i).unwrap(), item, "object {i} does not match");
        }
        bytes
    }

    #[test]
    fn from_items_round_trips() {
        assert_eq!(
            round_trip(vec![vec![1, 2, 3], vec![4, 5]]),
            [0, 2, 1, 1, 4, 6, 1, 2, 3, 4, 5]
        );
    }

    /// An object array longer than 255 bytes needs wider offsets.
    #[test]
    fn from_items_widens_offsets() {
        let bytes = round_trip(vec![vec![0; 300]]);
        assert_eq!(bytes[2], 2, "off_size");
        assert_eq!(&bytes[3..7], [0, 1, 1, 45], "offsets");
    }
}
