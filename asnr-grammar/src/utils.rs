use alloc::{vec::Vec, string::String, collections::BTreeMap};

use crate::{information_object::{InformationObjectClassField, ObjectFieldIdentifier}, ToplevelDeclaration, ASN1Value};

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

pub(crate) fn find_tld_or_enum_value_by_name(type_name: &String, name: &String, tlds: &BTreeMap<String, ToplevelDeclaration>) -> Option<ASN1Value> {
  if let Some(ToplevelDeclaration::Value(v)) = tlds.get(name) {
    return Some(v.value.clone())
  } else {
    for (_, tld) in tlds.iter() {
      if let Some(value) = tld.get_distinguished_or_enum_value(Some(type_name), name) {
        return Some(value)
      }
    }
    // Make second attempt without requiring a matching type name
    // This is the current best shot at linking inner subtypes
    for (_, tld) in tlds.iter() {
      if let Some(value) = tld.get_distinguished_or_enum_value(None, name) {
        return Some(value)
      }
    }
  }
  None
}

pub(crate) fn walk_object_field_ref_path<'a>(
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
