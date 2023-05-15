use nom::{
    error::{Error, ErrorKind},
    IResult, Parser,
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
    O1: Into<O2> {
      move |input: I| {
        let (input, o1) = parser.parse(input)?;
        Ok((input, o1.into()))
      }
    }