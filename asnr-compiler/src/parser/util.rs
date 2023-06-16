use std::cmp::min;

use nom::{
    error::{ErrorKind, ParseError},
    Err, FindSubstring, IResult, InputLength, InputTake, Parser,
};

pub fn hex_to_bools(c: char) -> [bool; 4] {
  match c {
      '1' => [false, false, false, true],
      '2' => [false, false, true, false],
      '3' => [false, false, true, true],
      '4' => [false, true, false, false],
      '5' => [false, true, false, true],
      '6' => [false, true, true, false],
      '7' => [false, true, true, true],
      '8' => [true, false, false, false],
      '9' => [true, false, false, true],
      'A' => [true, false, true, false],
      'B' => [true, false, true, true],
      'C' => [true, true, false, false],
      'D' => [true, true, false, true],
      'E' => [true, true, true, false],
      'F' => [true, true, true, true],
      _ => [false, false, false, false],
  }
}

pub fn map_into<I, O1, O2, E, F>(mut parser: F) -> impl FnMut(I) -> IResult<I, O2, E>
where
    F: Parser<I, O1, E>,
    O1: Into<O2>,
{
    move |input: I| {
        let (input, o1) = parser.parse(input)?;
        Ok((input, o1.into()))
    }
}

pub fn take_until_or<T, Input, Error: ParseError<Input>>(
    tag1: T,
    tag2: T,
) -> impl Fn(Input) -> IResult<Input, Input, Error>
where
    Input: InputTake + FindSubstring<T>,
    T: InputLength + Clone,
{
    move |i: Input| {
        let t1 = tag1.clone();
        let t2 = tag2.clone();
        let res: IResult<_, _, Error> = match (i.find_substring(t1), i.find_substring(t2))
        {
            (None, None) => Err(Err::Error(Error::from_error_kind(i, ErrorKind::TakeUntil))),
            (None, Some(index)) | (Some(index), None) => Ok(i.take_split(index)),
            (Some(i1), Some(i2)) => Ok(i.take_split(min(i1, i2))),
        };
        res
    }
}
