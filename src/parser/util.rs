use std::cmp::min;

use nom::{
    error::{Error, ErrorKind, ParseError},
    Err, FindSubstring, IResult, InputLength, InputTake, Parser,
};

pub fn try_parse_result_into<'a, I, O>(result: IResult<&'a str, I>) -> IResult<&'a str, O>
where
    I: TryInto<O>,
{
    match result {
        Ok((input, s)) => match s.try_into() {
            Ok(res) => Ok((input, res)),
            Err(_) => Err(nom::Err::Failure(Error::new(input, ErrorKind::Fail))),
        },
        Err(e) => Err(e),
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
