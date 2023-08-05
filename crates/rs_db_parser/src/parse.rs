use std::collections::HashMap;

use nom::Finish;
use nom_locate::LocatedSpan;

use crate::{
    ast::commands::create::Column,
    errors::{FormattedError, ParseResult, RawParseError},
};

pub type TableMap = HashMap<Box<str>, ColumnMap>;
pub type ColumnMap = HashMap<Box<str>, Column>;
pub type RawSpan<'a> = LocatedSpan<&'a str>;
pub type WithSpan<'a, T> = (RawSpan<'a>, T);

pub trait Parse<'a>: Sized {
    /// Parse the input and return the result.
    /// # Errors
    /// Returns a [`std::nom::Err`] of [`crate::errors::CustomParseError`] if the input is not a valid format.
    fn parse(input: RawSpan<'a>) -> ParseResult<'a, Self>;

    /// Parse the from a [`&str`] input and return the result.
    /// # Errors
    /// Returns a [`std::nom::Err`] of [`crate::errors::CustomParseError`] if the input is not a valid format.
    fn parse_from_raw(input: &'a str) -> ParseResult<'a, Self> {
        Self::parse(RawSpan::new(input))
    }

    /// Parse the input and return the result/
    /// # Errors
    /// Returns a `Vec<FormattedError>` if the input is not a valid format.
    fn parse_format_error(input: &'a str) -> Result<Self, FormattedError<'a>> {
        parse_format_error(input, Self::parse)
    }
}

#[allow(clippy::module_name_repetitions)]
/// Parse the input and return the result.
/// # Errors
/// Returns a [`std::nom::Err`] of [`crate::errors::CustomParseError`] if the input is not a valid format.
pub fn parse_format_error<'a, F, T>(input: &'a str, f: F) -> Result<T, FormattedError<'a>>
where
    F: nom::Parser<RawSpan<'a>, T, RawParseError<'a>>,
{
    match nom::combinator::all_consuming(f)(RawSpan::new(input)).finish() {
        Ok((_, result)) => Ok(result),
        Err(err) => Err(crate::errors::format_parse_error(input, err)),
    }
}
