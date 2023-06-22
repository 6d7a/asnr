extern crate asnr_traits;
#[macro_use]
extern crate asnr_grammar_derive;

use asnr_traits::Declare;

#[derive(Declare)]
pub struct RelationalConstraint {
  pub field_name: String,
  /// The level is null if the field is in the outermost object set of the declaration.
  /// The level is 1-n counting from the innermost object set of the declaration
  pub level: usize,
}

#[derive(Declare)]
pub struct ObjectIdentifierArc {
  pub name: Option<String>,
  pub number: Option<u128>,
}

#[test]
fn declares() {
    println!("{}", RelationalConstraint {field_name: "test".to_owned(), level: 2}.declare());
}