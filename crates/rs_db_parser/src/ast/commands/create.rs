use nom::{
    branch::alt,
    character::complete::{char, multispace0, multispace1},
    combinator::map,
    error::context,
    sequence::{delimited, preceded, separated_pair, tuple},
};
use nom_supreme::tag::complete::tag_no_case;

use crate::{
    errors::ParseResult,
    parse::{Parse, RawSpan, WithSpan},
    parsers::{comma_sep, identifier::identifier, parse_with_span},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SqlType {
    VarChar(usize),
    I8,
    I16,
    I32,
    I64,
    I128,
    U8,
    U16,
    U32,
    U64,
    U128,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RawColumn<'a> {
    pub name: RawSpan<'a>,
    pub tp: WithSpan<'a, SqlType>,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Column {
    pub name: Box<str>,
    pub tp: SqlType,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Statement<'a> {
    pub table_name: RawSpan<'a>,
    pub columns: Box<[RawColumn<'a>]>,
}

impl<'a> Parse<'a> for SqlType {
    fn parse(input: RawSpan<'a>) -> ParseResult<'a, Self> {
        context(
            "Column Type",
            alt((
                map(
                    preceded(
                        tag_no_case("varchar"),
                        delimited(char('('), usize::parse, char(')')),
                    ),
                    Self::VarChar,
                ),
                map(tag_no_case("int8"), |_| Self::I8),
                map(tag_no_case("int16"), |_| Self::I16),
                map(tag_no_case("int32"), |_| Self::I32),
                map(tag_no_case("int64"), |_| Self::I64),
                map(tag_no_case("int128"), |_| Self::I128),
                map(tag_no_case("uint8"), |_| Self::U8),
                map(tag_no_case("uint16"), |_| Self::U16),
                map(tag_no_case("uint32"), |_| Self::U32),
                map(tag_no_case("uint64"), |_| Self::U64),
                map(tag_no_case("uint128"), |_| Self::U128),
            )),
        )(input)
    }
}

impl<'a> Parse<'a> for RawColumn<'a> {
    fn parse(input: RawSpan<'a>) -> ParseResult<'a, Self> {
        context(
            "Column",
            map(
                tuple((context("Column Name", identifier), char(' '), |i| {
                    parse_with_span(i, SqlType::parse)
                })),
                |(name, _, tp)| Self { name, tp },
            ),
        )(input)
    }
}

impl<'a> Parse<'a> for Statement<'a> {
    fn parse(input: RawSpan<'a>) -> ParseResult<'a, Self> {
        context(
            "Create Table",
            map(
                separated_pair(
                    preceded(
                        tuple((
                            multispace0,
                            tag_no_case("create"),
                            multispace1,
                            tag_no_case("table"),
                            multispace1,
                        )),
                        context("Table Name", identifier),
                    ),
                    multispace1,
                    column_definitions,
                ),
                |(table_name, columns)| Self {
                    table_name: (*table_name.fragment()).into(),
                    columns,
                },
            ),
        )(input)
    }
}

fn column_definitions(input: RawSpan<'_>) -> ParseResult<'_, Box<[RawColumn]>> {
    context(
        "Column Definitions",
        map(
            delimited(char('('), comma_sep(RawColumn::parse), char(')')),
            |columns| columns.as_slice().into(),
        ),
    )(input)
}

impl<'a> From<RawColumn<'a>> for Column {
    fn from(value: RawColumn<'a>) -> Self {
        Self {
            name: (*value.name.fragment()).into(),
            tp: value.tp.1,
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    #[test]
    fn test_parse_sql_type() {
        assert_eq!(
            SqlType::parse("varchar(10)".into()).unwrap().1,
            SqlType::VarChar(10)
        );
        assert_eq!(SqlType::parse("int8".into()).unwrap().1, SqlType::I8);
        assert_eq!(SqlType::parse("int16".into()).unwrap().1, SqlType::I16);
        assert_eq!(SqlType::parse("int32".into()).unwrap().1, SqlType::I32);
        assert_eq!(SqlType::parse("int64".into()).unwrap().1, SqlType::I64);
        assert_eq!(SqlType::parse("int128".into()).unwrap().1, SqlType::I128);
        assert_eq!(SqlType::parse("uint8".into()).unwrap().1, SqlType::U8);
        assert_eq!(SqlType::parse("uint16".into()).unwrap().1, SqlType::U16);
        assert_eq!(SqlType::parse("uint32".into()).unwrap().1, SqlType::U32);
        assert_eq!(SqlType::parse("uint64".into()).unwrap().1, SqlType::U64);
        assert_eq!(SqlType::parse("uint128".into()).unwrap().1, SqlType::U128);
    }

    fn test_case_column_parse(suffix: &str, input: &str) {
        let value = RawColumn::parse(input.into()).unwrap().1;
        let mut settings = insta::Settings::new();
        settings.set_snapshot_suffix(suffix);
        settings.set_description(format!("Input: {input}"));
        settings.bind(|| insta::assert_debug_snapshot!(value));
    }
    fn test_case_statement_parse(suffix: &str, input: &str) {
        let value = Statement::parse(input.into()).unwrap().1;
        let mut settings = insta::Settings::new();
        settings.set_snapshot_suffix(suffix);
        settings.set_description(format!("Input: {input}"));
        settings.bind(|| insta::assert_debug_snapshot!(value));
    }

    #[test]
    fn test_parse_column() {
        test_case_column_parse("col-integer", "iD int8");
        test_case_column_parse("col-str", "column_name varchar(10)");
    }

    #[test]
    fn test_parse_statement() {
        test_case_statement_parse("1", "CREATE TABLE table_name (id int8)");
        test_case_statement_parse(
            "2",
            r#"
            CREATE TABLE table_name (
            id int8,
            name VARCHAR(10),
            age UINT8
            )"#,
        );
    }
}
