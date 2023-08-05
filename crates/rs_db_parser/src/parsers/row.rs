use crate::{
    ast::commands::create::Column,
    errors::{custom_error, ParseResult},
    parse::{RawSpan, WithSpan},
    value::Value,
};

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug)]
pub struct RowParser<'a> {
    columns: Vec<(RawSpan<'a>, &'a Column)>,
}

impl<'a> RowParser<'a> {
    #[must_use]
    pub fn new(columns: Vec<(RawSpan<'a>, &'a Column)>) -> Self {
        Self { columns }
    }

    /// Parses a row of values.
    /// # Errors
    /// Returns an error if the input is not a valid row of values.
    /// Returns an error if the number of values does not match the number of columns.
    pub fn parse(
        &mut self,
        input: RawSpan<'a>,
    ) -> ParseResult<'a, (RawSpan<'a>, WithSpan<'a, Value>)> {
        self.columns.pop().map_or_else(
            || {
                Err(custom_error(
                    input,
                    nom_supreme::error::BaseErrorKind::Expected(
                        nom_supreme::error::Expectation::Char(')'),
                    ),
                ))
            },
            |(name_span, column)| {
                Value::parse_with_type(column.tp, input)
                    .map(|(input, value)| (input, (name_span, value)))
            },
        )
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.columns.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.columns.is_empty()
    }

    #[must_use]
    pub fn pop(&mut self) -> Option<(RawSpan<'a>, &'a Column)> {
        self.columns.pop()
    }
}
