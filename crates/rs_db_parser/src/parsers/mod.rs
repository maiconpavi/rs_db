use nom::{
    character::complete::{char, multispace0},
    multi::separated_list1,
    sequence::delimited,
    IResult,
};
use nom_locate::LocatedSpan;

use crate::{
    errors::ParseResult,
    parse::{RawSpan, WithSpan},
};

pub mod identifier;
pub mod number;
pub mod row;

pub(crate) fn comma_sep<'a, O, E, F>(
    f: F,
) -> impl FnMut(RawSpan<'a>) -> IResult<RawSpan<'a>, Vec<O>, E>
where
    F: nom::Parser<RawSpan<'a>, O, E>,
    E: nom::error::ParseError<RawSpan<'a>>,
{
    delimited(
        multispace0,
        separated_list1(delimited(multispace0, char(','), multispace0), f),
        multispace0,
    )
}

pub(crate) fn map_raw_span<'a, T: 'a>(
    span: RawSpan<'a>,
    f: impl FnOnce(&'a str) -> T,
) -> LocatedSpan<T> {
    unsafe {
        LocatedSpan::new_from_raw_offset(
            span.location_offset(),
            span.location_line(),
            f(span.fragment()),
            (),
        )
    }
}

pub(crate) fn truncate_raw_span<'a>(first: &RawSpan<'a>, second: &RawSpan<'a>) -> RawSpan<'a> {
    let offset1 = first.location_offset();
    let offset2 = second.location_offset();
    let diff = offset2 - offset1;
    unsafe {
        LocatedSpan::new_from_raw_offset(
            first.location_offset(),
            first.location_line(),
            &first.fragment()[..diff],
            (),
        )
    }
}

pub(crate) fn parse_with_span<'a, T>(
    input: RawSpan<'a>,
    f: impl FnOnce(RawSpan<'a>) -> ParseResult<'a, T>,
) -> ParseResult<'a, WithSpan<'a, T>> {
    let (input2, value) = f(input)?;
    Ok((input2, (truncate_raw_span(&input, &input2), value)))
}
