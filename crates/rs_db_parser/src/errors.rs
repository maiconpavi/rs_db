use nom::IResult;
use nom_locate::LocatedSpan;
use nom_supreme::error::{BaseErrorKind, ErrorTree, StackContext};

use crate::parse::RawSpan;

pub type RawParseError<'a> = ErrorTree<RawSpan<'a>>;
pub type CustomParseError<T> = ErrorTree<LocatedSpan<T>>;

pub type ParseResult<'a, T> = IResult<RawSpan<'a>, T, RawParseError<'a>>;

pub(crate) fn custom_error<'a, T: 'a>(
    input: LocatedSpan<T>,
    kind: BaseErrorKind<&'static str, Box<dyn std::error::Error + Send + Sync + 'static>>,
) -> nom::Err<CustomParseError<T>> {
    nom::Err::Error(CustomParseError::Base {
        location: input,
        kind,
    })
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Column not found")]
    ColumnNotFound,

    #[error("Column declared, but not used")]
    ColumnNotUsed,
}

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
#[error("Parse Error")]
pub struct FormattedError<'b> {
    #[source_code]
    src: &'b str,

    #[label("{kind}")]
    span: miette::SourceSpan,

    kind: BaseErrorKind<&'b str, Box<dyn std::error::Error + Send + Sync + 'static>>,

    #[related]
    others: Vec<FormattedErrorContext<'b>>,
}

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
#[error("Parse Error Context")]
pub struct FormattedErrorContext<'b> {
    #[source_code]
    src: &'b str,

    #[label("{context}")]
    span: miette::SourceSpan,

    context: StackContext<&'b str>,
}

#[must_use]
pub fn format_parse_error<'a>(input: &'a str, err: RawParseError<'a>) -> FormattedError<'a> {
    match err {
        RawParseError::Base { location, kind } => FormattedError {
            src: input,
            span: miette::SourceSpan::new(location.location_offset().into(), 0.into()),
            kind,
            others: Vec::new(),
        },
        RawParseError::Stack { base, contexts } => {
            let mut base = format_parse_error(input, *base);
            let mut contexts = contexts
                .into_iter()
                .map(|(location, context)| FormattedErrorContext {
                    src: input,
                    span: miette::SourceSpan::new(location.location_offset().into(), 0.into()),
                    context,
                })
                .collect::<Vec<_>>();
            base.others.append(&mut contexts);
            base
        }
        RawParseError::Alt(alt_errors) => alt_errors
            .into_iter()
            .map(|e| format_parse_error(input, e))
            .max_by_key(|e| e.others.len())
            .expect("alt errors should not be empty"),
    }
}
