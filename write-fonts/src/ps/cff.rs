//! CFF and CFF2 fonts.

pub mod dict;
pub mod v1;
pub mod v2;

/// Packs the offset array and object data of an INDEX.
///
/// Returns the `off_size`, the packed offsets and the concatenated object
/// data. The `CFF` and `CFF2` INDEX formats differ only in the width of their
/// count field, so everything after it is built the same way.
///
/// See <https://learn.microsoft.com/en-us/typography/opentype/spec/cff2#5-index-data>.
fn index_parts(items: Vec<Vec<u8>>) -> (u8, Vec<u8>, Vec<u8>) {
    // Offsets are relative to the byte before the object data, so they start
    // at one.
    let mut offset_values = Vec::with_capacity(items.len() + 1);
    let mut current_offset = 1u32;
    offset_values.push(current_offset);
    for item in &items {
        current_offset += item.len() as u32;
        offset_values.push(current_offset);
    }

    // off_size is the smallest number of bytes that holds the last offset.
    let max_offset = *offset_values.last().unwrap();
    let off_size = if max_offset <= 0xFF {
        1u8
    } else if max_offset <= 0xFFFF {
        2u8
    } else if max_offset <= 0xFFFFFF {
        3u8
    } else {
        4u8
    };

    let mut offsets = Vec::with_capacity(offset_values.len() * off_size as usize);
    for offset in &offset_values {
        let bytes = offset.to_be_bytes();
        offsets.extend(&bytes[4 - off_size as usize..]);
    }

    let data = items.into_iter().flatten().collect();
    (off_size, offsets, data)
}
