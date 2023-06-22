
/// The `Declare` trait serves to convert a structure
/// into a stringified rust representation of its initialization.
///
/// #### Example
/// Let's say we have
/// ```rust
/// # use asnr_traits::Declare;
///
/// pub struct Foo {
///   pub bar: u8
/// }
/// // The implementation of `Declare` for `Foo` would look like this:
/// impl Declare for Foo {
///   fn declare(&self) -> String {
///     format!("Foo {{ bar: {} }}", self.bar)
///   }
/// }
/// ```
pub trait Declare {
    /// Returns a stringified representation of the implementing struct's initialization
    ///
    /// #### Example
    /// ```rust
    /// # use asnr_traits::Declare;
    /// # pub struct Foo { pub bar: u8 }
    /// # impl Declare for Foo {
    /// #  fn declare(&self) -> String { format!("Foo {{ bar: {} }}", self.bar) }
    /// # }
    /// let foo = Foo { bar: 1 };
    /// assert_eq!("Foo { bar: 1 }".to_string(), foo.declare());
    /// ```
    fn declare(&self) -> String;
}
