use nom::{
    character::complete::{char, multispace0, multispace1},
    combinator::map_opt,
    error::context,
    sequence::{delimited, preceded, terminated, tuple},
};
use nom_supreme::tag::complete::tag_no_case;

use crate::{
    errors::{custom_error, ParseResult},
    parse::{ColumnMap, RawSpan, TableMap, WithSpan},
    parsers::row::RowParser,
    parsers::{comma_sep, identifier::identifier},
    value::Value,
};

#[derive(Debug, Clone, Hash)]
pub struct Statement<'a> {
    pub table_name: RawSpan<'a>,
    pub values: Box<[(RawSpan<'a>, WithSpan<'a, Value>)]>,
}

impl<'a> Statement<'a> {
    #[must_use]
    pub fn len(&self) -> usize {
        self.values.iter().map(|(_k, (_, v))| v.len()).sum()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

fn parse_values<'a>(
    columns: &'a ColumnMap,
    input: RawSpan<'a>,
) -> ParseResult<'a, Vec<(RawSpan<'a>, WithSpan<'a, Value>)>> {
    let (input1, value_names): (RawSpan, Vec<RawSpan>) = context(
        "Column Definitions",
        delimited(
            multispace1,
            delimited(
                char('('),
                delimited(
                    multispace0,
                    context("Column Names", comma_sep(identifier)),
                    multispace0,
                ),
                char(')'),
            ),
            delimited(multispace1, tag_no_case("values"), multispace1),
        ),
    )(input)?;

    let mut columns_found = vec![];
    for name in &value_names {
        if let Some(column) = columns.get(*name.fragment()) {
            columns_found.insert(0, (*name, column));
        } else {
            return Err(custom_error(
                *name,
                nom_supreme::error::BaseErrorKind::External(Box::new(
                    crate::errors::ParseError::ColumnNotFound,
                )),
            ));
        }
    }

    let mut row_parser = RowParser::new(columns_found);

    let (input2, values) = context(
        "Column Values",
        terminated(
            delimited(
                char('('),
                delimited(
                    multispace0,
                    context("Column Names", comma_sep(|i| row_parser.parse(i))),
                    multispace0,
                ),
                char(')'),
            ),
            multispace0,
        ),
    )(input1)?;

    row_parser.pop().map_or_else(
        || Ok((input2, values)),
        |(name, _)| {
            Err(custom_error(
                name,
                nom_supreme::error::BaseErrorKind::External(Box::new(
                    crate::errors::ParseError::ColumnNotUsed,
                )),
            ))
        },
    )
}

impl<'a> Statement<'a> {
    /// Parses an `INSERT` statement.
    /// # Errors
    /// Returns an error if the input is not a valid `INSERT` statement.
    pub fn parse_with_table_map(
        table_map: &'a TableMap,
        input: RawSpan<'a>,
    ) -> ParseResult<'a, Self> {
        let (input, (_, _, (table_name, columns))) = context(
            "Insert Statement",
            tuple((
                tag_no_case("insert"),
                preceded(multispace1, tag_no_case("into")),
                preceded(
                    multispace1,
                    map_opt(context("Table Name", identifier), |table_name| {
                        let columns = table_map.get(*table_name.fragment())?;
                        Some((table_name, columns))
                    }),
                ),
            )),
        )(input)?;

        let (input, values) = context("Insert Statement", |i| parse_values(columns, i))(input)?;

        Ok((
            input,
            Self {
                table_name,
                values: values.into(),
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use miette::GraphicalTheme;

    use crate::{
        ast::commands::create::{Column, SqlType},
        parse::parse_format_error,
    };

    use super::*;

    fn get_table_map() -> TableMap {
        let mut table_map = TableMap::new();
        table_map.insert(
            "test_table".into(),
            [
                Column {
                    name: "id".into(),
                    tp: SqlType::I32,
                },
                Column {
                    name: "name".into(),
                    tp: SqlType::VarChar(255),
                },
            ]
            .into_iter()
            .map(|column| (column.name.clone(), column))
            .collect(),
        );
        table_map
    }

    #[allow(clippy::needless_pass_by_value)]
    fn test_case(suffix: &str, input: &str) {
        let table_map = get_table_map();

        let (_, statement) =
            Statement::parse_with_table_map(&table_map, RawSpan::new(input)).unwrap();
        let mut settings = insta::Settings::new();
        settings.set_snapshot_suffix(suffix);
        settings.set_description(format!("Input: {input}",));
        settings.bind(|| {
            insta::assert_debug_snapshot!(statement);
        });
    }

    fn test_case_err(suffix: &str, input: &str) {
        let table_map = get_table_map();
        match parse_format_error(input, |i| Statement::parse_with_table_map(&table_map, i)) {
            Ok(_) => panic!("Expected error"),
            Err(err) => {
                let mut s = String::new();
                miette::GraphicalReportHandler::new()
                    .with_cause_chain()
                    .with_theme(GraphicalTheme::unicode_nocolor())
                    .with_context_lines(10)
                    .render_report(&mut s, &err)
                    .unwrap();
                let mut settings = insta::Settings::new();
                settings.set_snapshot_suffix(suffix);
                settings.set_description(format!("Input: {input}",));
                settings.bind(|| {
                    insta::assert_snapshot!(s);
                });
            }
        }
    }

    #[test]
    fn test_parse_values() {
        let table_map = get_table_map();
        let column_map = table_map.get("test_table").unwrap();
        let input = RawSpan::new(" (id, name) VALUES ( 1, 'test' ) ");
        let (_, values) = parse_values(column_map, input).unwrap();
        let mut settings = insta::Settings::new();
        settings.set_description(format!("Input: {input}",));
        settings.bind(|| {
            insta::assert_debug_snapshot!(values);
        });
    }

    #[test]
    fn test_statement() {
        test_case(
            "1",
            r#"INSERT INTO test_table (id, name) VALUES ( 2, 'test asdasd') "#,
        );
    }

    #[test]
    fn test_invalid_statement() {
        test_case_err(
            "more-values",
            r#"INSERT INTO test_table (id, name) VALUES ( 2, 'test asdasd', '3') "#,
        );
        test_case_err(
            "less-values",
            r#"INSERT INTO test_table (id, name) VALUES ( 2) "#,
        );
        test_case_err(
            "wrong-column",
            r#"INSERT INTO test_table (id, age) VALUES ( 2, 3) "#,
        );
    }
}
