use nom::{bytes::complete::take_while_m_n, combinator::map};

use crate::{errors::ParseResult, parse::RawSpan};

pub(crate) fn identifier(input: RawSpan<'_>) -> ParseResult<RawSpan<'_>> {
    map(
        take_while_m_n(0, 128, |c: char| c.is_ascii_alphanumeric() || c == '_'),
        |s: RawSpan| s,
    )(input)
}
