mod generator;
mod parser;
mod validator;

fn main() {
  let _tld = parser::top_level_declaration(r#"/** 
        * This DE represents the magnitude of the acceleration vector in a defined coordinate system.
        *
        * The value shall be set to:
        * - `0` to indicate no acceleration,
        * - `n` (`n > 0` and `n < 160`) to indicate acceleration equal to or less than n x 0,1 m/s^2, and greater than (n-1) x 0,1 m/s^2,
        * - `160` for acceleration values greater than 15,9 m/s^2,
        * - `161` when the data is unavailable.
        *
        * @unit 0,1 m/s^2
        * @category: Kinematic information
        * @revision: Created in V2.1.1
      */
      AccelerationMagnitudeValue ::= INTEGER {
          positiveOutOfRange (160),
          unavailable        (161)  
      } (0.. 161, ...)"#).unwrap().1;
}