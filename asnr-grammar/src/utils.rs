use alloc::vec::Vec;

use crate::information_object::{InformationObjectClassField, ObjectFieldIdentifier};

pub fn int_type_token<'a>(min: i128, max: i128) -> &'a str {
    match max - min {
        r if r <= u8::MAX.into() && min >= 0 => "u8",
        r if r <= u8::MAX.into() => "i8",
        r if r <= u16::MAX.into() && min >= 0 => "u16",
        r if r <= u16::MAX.into() => "i16",
        r if r <= u32::MAX.into() && min >= 0 => "u32",
        r if r <= u32::MAX.into() => "i32",
        r if r <= u64::MAX.into() && min >= 0 => "u64",
        r if r <= u64::MAX.into() => "i64",
        _ => "i128",
    }
}

pub fn walk_object_field_ref_path<'a>(
    fields: &'a Vec<InformationObjectClassField>,
    path: &'a Vec<ObjectFieldIdentifier>,
    mut index: usize,
) -> Option<&'a InformationObjectClassField> {
    fields.iter().find_map(|f| {
        path.get(index)
            .map(|id| {
                (&f.identifier == id).then(|| {
                  if path.len() == (index + 1) {
                    Some(f)
                  } else {
                    index += 1;
                    walk_object_field_ref_path(fields, path, index)
                  }
                })
            })
            .flatten()
    }).flatten()
}
