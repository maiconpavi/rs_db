use nom::{
    bytes::complete::take_while1,
    combinator::{map, map_res},
};

use crate::{
    errors::ParseResult,
    parse::{Parse, RawSpan},
};

macro_rules! impl_parse_number {
    ($($ty:ty),*) => {
        $(
            impl<'a> Parse<'a> for $ty {
                fn parse(input: RawSpan<'a>) -> ParseResult<'a, Self> {
                    map_res(parse_int, |s| s.parse())(input)
                }
            }
        )*
    };
    () => {

    };
}

impl_parse_number!(usize, i8, i16, i32, i64, i128, u8, u16, u32, u64, u128);

fn parse_int(input: RawSpan) -> ParseResult<&str> {
    map(
        take_while1(|c: char| c.is_ascii_digit() || c == '-'),
        |s: RawSpan| *s.fragment(),
    )(input)
}
