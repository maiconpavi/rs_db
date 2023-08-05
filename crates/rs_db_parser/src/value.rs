use nom::{
    bytes::complete::escaped,
    character::complete::{char, none_of, one_of},
    combinator::{cut, map, map_res},
    error::context,
    sequence::{preceded, terminated},
};

use crate::{
    ast::commands::create::SqlType,
    errors::ParseResult,
    parse::{Parse, RawSpan, WithSpan},
    parsers::parse_with_span,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Value {
    VarChar(Box<str>),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
}

impl Value {
    fn parse_inner(tp: SqlType, input: RawSpan<'_>) -> ParseResult<'_, Self> {
        match tp {
            SqlType::VarChar(size) => map(
                preceded(
                    char('\''),
                    cut(map_res(
                        terminated(escaped(none_of("\\'"), '\\', one_of("'\\")), char('\'')),
                        |s: RawSpan| {
                            if s.len() > size {
                                Err("Value too long")
                            } else {
                                Ok(s)
                            }
                        },
                    )),
                ),
                |s: RawSpan| Self::VarChar((*s).into()),
            )(input),
            SqlType::I8 => map(i8::parse, Self::I8)(input),
            SqlType::I16 => map(i16::parse, Self::I16)(input),
            SqlType::I32 => map(i32::parse, Self::I32)(input),
            SqlType::I64 => map(i64::parse, Self::I64)(input),
            SqlType::I128 => map(i128::parse, Self::I128)(input),
            SqlType::U8 => map(u8::parse, Self::U8)(input),
            SqlType::U16 => map(u16::parse, Self::U16)(input),
            SqlType::U32 => map(u32::parse, Self::U32)(input),
            SqlType::U64 => map(u64::parse, Self::U64)(input),
            SqlType::U128 => map(u128::parse, Self::U128)(input),
        }
    }

    /// Parse a value with the given type.
    /// # Errors
    /// If the type is `VarChar` and the value is not the correct length.
    pub fn parse_with_type(tp: SqlType, input: RawSpan<'_>) -> ParseResult<'_, WithSpan<'_, Self>> {
        context("Value", |i| {
            parse_with_span(i, |i| Self::parse_inner(tp, i))
        })(input)
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        match self {
            Self::VarChar(s) => s.len(),
            Self::I8(_) | Self::U8(_) => 1,
            Self::I16(_) | Self::U16(_) => 2,
            Self::I32(_) | Self::U32(_) => 4,
            Self::I64(_) | Self::U64(_) => 8,
            Self::I128(_) | Self::U128(_) => 16,
        }
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        match self {
            Self::VarChar(s) => s.is_empty(),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[allow(clippy::needless_pass_by_value)]
    fn test_case(suffix: &str, tp: SqlType, input: &str) {
        let (_, v) = Value::parse_with_type(tp, RawSpan::new(input)).unwrap();
        let mut settings = insta::Settings::new();
        settings.set_snapshot_suffix(suffix);
        settings.set_description(format!("Input: {input}\nType: {tp:?}",));
        settings.bind(|| {
            insta::assert_debug_snapshot!(v);
        });
    }

    #[test]
    fn test_value_var_char() {
        test_case("simple-str", SqlType::VarChar(5), "'hello'");

        test_case("simple-str-2", SqlType::VarChar(50), "'hello world'");

        test_case("simple-str-3", SqlType::VarChar(50), "'hello\nworld'");

        assert!(Value::parse_with_type(SqlType::VarChar(5), RawSpan::new("'123456789'")).is_err());
    }

    #[test]
    fn test_value_integers() {
        test_case("pos-i8", SqlType::I8, "19");
        test_case("neg-i8", SqlType::I8, "-19");
        test_case("pos-i16", SqlType::I16, "19");
        test_case("neg-i16", SqlType::I16, "-19");
        test_case("pos-i32", SqlType::I32, "19");
        test_case("neg-i32", SqlType::I32, "-19");
        test_case("pos-i64", SqlType::I64, "19");
        test_case("neg-i64", SqlType::I64, "-19");
        test_case("pos-i128", SqlType::I128, "19");
        test_case("neg-i128", SqlType::I128, "-19");
        test_case("pos-u8", SqlType::U8, "19");
        test_case("pos-u16", SqlType::U16, "19");
        test_case("pos-u32", SqlType::U32, "19");
        test_case("pos-u64", SqlType::U64, "19");
        test_case("pos-u128", SqlType::U128, "19");
    }
}
