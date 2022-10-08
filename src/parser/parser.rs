use std::cmp::Ordering;
use nom::{bytes::complete::{tag, take}, character::complete::{multispace0, one_of}, Finish, InputTake, IResult};
use nom::character::complete::none_of;
use nom::combinator::{not, opt};
use nom::error::{context, ErrorKind, ParseError, VerboseError, VerboseErrorKind};
use nom_locate::position;

use crate::parser::model::{JsonObject, JsonString, JsonStyle, JsonType, LineEnding, SortOrder, Span};
use crate::SortAlgorithm;

mod error {
    use colored::Colorize;
    use crate::parser::model::{JsonStyle, Span};

    pub const MAYBE_SPACE: &str = "maybe_space";
    pub const POST_COLON: &str = "post_colon";
    pub const POST_COLON_TOO_MUCH: &str = "post_colon_too_much";
    pub const CRLF: &str = "crlf";
    pub const LF: &str = "lf";
    pub const NO_BREAK: &str = "no_break";
    pub const ANY_BREAK: &str = "any_break";
    pub const CR_BUT_NOT_LF: &str = "cr_but_no_lf";
    pub const NOT_ENOUGH_INDENTATION: &str = "not_enough_indentation";
    pub const TOO_MUCH_INDENTATION: &str = "too_much_indentation";
    pub const SORTING: &str = "sorting";

    pub fn context_to_message(style: &JsonStyle, context: &str) -> Option<String> {
        match context {
            MAYBE_SPACE => Some(format!("expected no whitespace here")),
            POST_COLON => match style {
                JsonStyle::STYLED { post_colon, .. } =>
                    Some(format!("expected \"{}\" after the double colon", match post_colon {
                        Some(s) => *s,
                        None => " ",
                    })),
                JsonStyle::IGNORE => None,
            },
            POST_COLON_TOO_MUCH => match style {
                JsonStyle::STYLED { post_colon, .. } =>
                    Some(format!("expected ONLY \"{}\" after the double colon", match post_colon {
                        Some(s) => *s,
                        None => " ",
                    })),
                JsonStyle::IGNORE => None,
            },
            CRLF => Some(format!("expected \"{}\" at the end of a line", "\\r\\n".yellow())),
            LF => Some(format!("expected \"{}\" at the end of a line", "\\n".yellow())),
            NO_BREAK => Some(format!("expected no linebreaks")),
            ANY_BREAK => Some(format!("expected any linebreak at the end of a line")),
            CR_BUT_NOT_LF => Some(format!("expected \"{}\" after \"{}\"", "\\n".yellow(), "\\r".yellow())),
            NOT_ENOUGH_INDENTATION => Some(format!("expected less indentation")),
            TOO_MUCH_INDENTATION => Some(format!("expected less indentation")),
            SORTING => Some(format!("expected the key to be greater than its predecessor")),
            _ => None,
        }
    }

    pub fn generate_error_message<'b>(
        style: &JsonStyle,
        err: &nom::error::VerboseError<Span<'b>>,
    ) -> Option<(Span<'b>, String)> {
        for (s, e) in &err.errors {
            if let nom::error::VerboseErrorKind::Context(ctx) = e {
                if let Some(message) = context_to_message(style, ctx) {
                    return Some((
                        s.clone(),
                        format!(
                            "In line {} at offset {}, I {}",
                            format!("{}", s.location_line()).blue(),
                            format!("{}", s.get_column()).blue(),
                            message,
                        ),
                    ))
                }
            }
        }

        None
    }
}

pub struct Parser {
    style: JsonStyle
}

impl Parser {
    pub fn new(style: JsonStyle) -> Parser {
        Parser {
            style,
        }
    }

    fn parse_string(
        s: Span,
    ) -> IResult<Span, JsonString, nom::error::VerboseError<Span>> {
        let (s, _) = tag("\"")(s)?;
        let (s, start) = position(s)?;
        let mut ms = s;
        let mut escaped = false;
        loop {
            let (s, c) = take(1_usize)(ms)?;
            if c.ends_with("\\") {
                if escaped {
                    escaped = false; // if already escaped then no longer escaped
                } else {
                    escaped = true; // if not already escaped then escape
                }
            } else {
                if c.ends_with("\"") {
                    if escaped {
                        escaped = false; // escaped
                    } else {
                        break; // unescaped -> terminator
                    }
                } else {
                    escaped = false; // if not backslash then is no longer escaped
                }
            }
            ms = s;
        }
        let (s, contents) = s.take_split(ms.location_offset() - s.location_offset());
        let (s, _) = tag("\"")(s)?;
        let (s, end) = position(s)?;

        Ok((s, JsonString {
            value: *contents,
            start,
            end,
        }))
    }

    #[inline(always)]
    fn parse_space0(
        mut s: Span,
    ) -> IResult<Span, Span, nom::error::VerboseError<Span>> {
        while let Ok((sn, _)) = one_of::<_, _, nom::error::Error<Span>>(" \t")(s) {
            s = sn
        }

        Ok((s, Span::new("")))
    }

    #[inline(always)]
    fn parse_maybe_space<'a, 'b>(
        &'a self,
        s: Span<'b>,
        style_errors: &mut Vec<nom::error::VerboseError<Span<'b>>>,
    ) -> IResult<Span<'b>, Span<'b>, nom::error::VerboseError<Span<'b>>> {
        match match self.style {
            JsonStyle::STYLED { .. } => context(error::MAYBE_SPACE, not(one_of(" \t")))(s).map(|(_, _)| s),
            JsonStyle::IGNORE => multispace0(s).map(|(s, _)| s),
        } {
            Ok(s) => {
                Ok((s, Span::new("")))
            },
            Err(e) => {
                match e {
                    nom::Err::Incomplete(_) => {
                        // Not happening
                        std::process::exit(-1);
                    }
                    nom::Err::Error(e) | nom::Err::Failure(e) => {
                        style_errors.push(e);
                        let (s, _) = Parser::parse_space0(s)?;
                        Ok((s, Span::new("")))
                    }
                }
            }
        }
    }

    #[inline(always)]
    fn parse_post_colon<'a, 'b>(
        &'a self,
        s: Span<'b>,
        style_errors: &mut Vec<nom::error::VerboseError<Span<'b>>>,
    ) -> IResult<Span<'b>, Span<'b>, nom::error::VerboseError<Span<'b>>> {
        match match self.style {
            JsonStyle::STYLED { post_colon, .. } => {
                if let Some(post_colon) = post_colon {
                    match context(error::POST_COLON, tag(post_colon))(s).map(|(s, _)| s) {
                        Ok(s) => context(error::POST_COLON_TOO_MUCH, one_of("{\""))(s).map(|_| s),
                        Err(e) => Err(e),
                    }
                } else {
                    multispace0(s).map(|(s, _)| s)
                }
            }
            JsonStyle::IGNORE => multispace0(s).map(|(s, _)| s),
        } {
            Ok(s) => {
                Ok((s, Span::new("")))
            },
            Err(e) => {
                match e {
                    nom::Err::Incomplete(_) => {
                        // Not happening
                        std::process::exit(-1);
                    }
                    nom::Err::Error(e) | nom::Err::Failure(e) => {
                        style_errors.push(e);
                        let (s, _) = multispace0(s)?;
                        Ok((s, Span::new("")))
                    }
                }
            }
        }
    }

    fn parse_entry<'a, 'b>(
        &'a self,
        s: Span<'b>,
        indent: u64,
        style_errors: &mut Vec<nom::error::VerboseError<Span<'b>>>,
    ) -> IResult<Span<'b>, (JsonString<'b>, JsonType<'b>), nom::error::VerboseError<Span<'b>>> {
        let (s, key) = Parser::parse_string(s)?;
        let (s, _) = self.parse_maybe_space(s, style_errors)?;
        let (s, _) = tag(":")(s)?;
        let (s, _) = self.parse_post_colon(s, style_errors)?;
        let (_, c) = one_of("{\"")(s)?;

        if c == '\"' {
            let (s, value) = Parser::parse_string(s)?;

            Ok((s, (key, JsonType::String(value))))
        } else {
            let (s, value) = self.parse_object(s, indent, style_errors)?;

            Ok((s, (key, JsonType::Object(value))))
        }
    }

    #[inline(always)]
    fn parse_new_line<'a, 'b>(
        &'a self,
        s: Span<'b>,
        style_errors: &mut Vec<nom::error::VerboseError<Span<'b>>>,
    ) -> IResult<Span<'b>, Span<'b>, nom::error::VerboseError<Span<'b>>> {
        match match self.style {
            JsonStyle::STYLED { line_endings, .. } => {
                match line_endings {
                    LineEnding::CRLF => context(error::CRLF, tag("\r\n"))(s).map(|(s, _)| s),
                    LineEnding::LF => context(error::LF, tag("\n"))(s).map(|(s, _)| s),
                    LineEnding::NONE => context(error::NO_BREAK, none_of("\r\n"))(s).map(|(_, _)| s),
                    LineEnding::ANY => {
                        let (s, _) = opt(tag("\r"))(s)?;
                        context(error::ANY_BREAK, tag("\n"))(s).map(|(s, _)| s)
                    }
                    LineEnding::IGNORE => {
                        let (s, v) = opt(tag("\r"))(s)?;
                        match v {
                            Some(_) => context(error::CR_BUT_NOT_LF, tag("\n"))(s).map(|(s, _)| s),
                            None => opt(tag("\n"))(s).map(|(s, _)| s),
                        }
                    }
                }
            }
            JsonStyle::IGNORE => multispace0(s).map(|(s, _)| s),
        } {
            Ok(s) => {
                Ok((s, Span::new("")))
            },
            Err(e) => {
                match e {
                    nom::Err::Incomplete(_) => {
                        // Not happening
                        std::process::exit(-1);
                    }
                    nom::Err::Error(e) | nom::Err::Failure(e) => {
                        style_errors.push(e);
                        let (s, v) = opt(tag("\r"))(s)?;
                        let s = match v {
                            Some(_) => tag("\n")(s)?.0,
                            None => opt(tag("\n"))(s)?.0,
                        };
                        Ok((s, Span::new("")))
                    }
                }
            }
        }
    }

    #[inline(always)]
    fn parse_indentation<'a, 'b>(
        &'a self,
        s: Span<'b>,
        indent: u64,
        style_errors: &mut Vec<nom::error::VerboseError<Span<'b>>>,
        last: bool,
    ) -> IResult<Span<'b>, Span<'b>, nom::error::VerboseError<Span<'b>>> {
        match match self.style {
            JsonStyle::STYLED { indentation, .. } => {
                if let Some(indentation) = indentation {
                    let mut s = Ok(s);
                    for _ in 0..indent {
                        if let Ok(s_ok) = s {
                            s = context(
                                error::NOT_ENOUGH_INDENTATION,
                                tag(indentation)
                            )(s_ok).map(|(s, _)| s);
                        } else {
                            break;
                        }
                    }

                    if last {
                        match s {
                            Ok(s) => {
                                context(
                                    error::TOO_MUCH_INDENTATION,
                                    one_of("\"}")
                                )(s).map(|_| s)
                            },
                            Err(e) => Err(e),
                        }
                    } else {
                        s
                    }
                } else {
                    multispace0(s).map(|(s, _)| s)
                }
            }
            JsonStyle::IGNORE => multispace0(s).map(|(s, _)| s),
        } {
            Ok(s) => {
                Ok((s, Span::new("")))
            },
            Err(e) => {
                match e {
                    nom::Err::Incomplete(_) => {
                        // Not happening
                        std::process::exit(-1);
                    }
                    nom::Err::Error(e) | nom::Err::Failure(e) => {
                        style_errors.push(e);
                        let (s, _) = Parser::parse_space0(s)?;
                        Ok((s, Span::new("")))
                    }
                }
            }
        }
    }

    fn parse_object<'a, 'b>(
        &'a self,
        s: Span<'b>,
        indent: u64,
        style_errors: &mut Vec<nom::error::VerboseError<Span<'b>>>,
    ) -> IResult<Span<'b>, JsonObject<'b>, nom::error::VerboseError<Span<'b>>> {
        let (s, _) = tag("{")(s)?;
        let (s, start) = position(s)?;
        let (s, _) = self.parse_new_line(s, style_errors)?;
        let (s, _) = self.parse_indentation(s, indent, style_errors, false)?;
        let (s, closing) = opt(tag("}"))(s)?;
        if closing.is_some() {
            let (s, end) = position(s)?;
            // empty object
            Ok((s, JsonObject {
                values: Vec::new(),
                start,
                end,
            }))
        } else {
            // not empty
            // expect content to be indented 1 more than parent
            let (s, _) = self.parse_indentation(s, 1, style_errors, true)?;
            let mut values: Vec<(JsonString, JsonType)> = Vec::new();
            let mut ms= s;
            loop {
                let (s, kv) = self.parse_entry(ms, indent + 1, style_errors)?;
                if let Some((key, _)) = values.last() {
                    match self.style {
                        JsonStyle::STYLED { order, sort_algorithm, .. } => {
                            match (order, match sort_algorithm {
                                SortAlgorithm::NATURAL => {
                                    crate::natural_sort::compare(key.value, kv.0.value)
                                }
                                SortAlgorithm::NORMAL => {
                                    key.value.cmp(kv.0.value)
                                }
                                SortAlgorithm::NONE => {
                                    Ordering::Greater
                                }
                            }) {
                                (SortOrder::ASC, Ordering::Less) | (SortOrder::DESC, Ordering::Greater) => {
                                    let mut e = VerboseError::from_error_kind(
                                        kv.0.start.clone(),
                                        ErrorKind::Verify
                                    );
                                    e.errors.push((
                                        kv.0.start.clone(),
                                        VerboseErrorKind::Context(error::SORTING)
                                    ));
                                    style_errors.push(e);
                                }
                                _ => {}
                            }
                        }
                        JsonStyle::IGNORE => {}
                    }
                }

                values.push(kv);
                let (s, _) = self.parse_maybe_space(s, style_errors)?;
                let (s, c) = opt(tag(","))(s)?;
                let (s, _) = self.parse_new_line(s, style_errors)?;
                if c.is_none() {
                    // expecting end
                    let (s, _) = self.parse_indentation(s, indent, style_errors, true)?;
                    let (s, _) = tag("}")(s)?;
                    ms = s;
                    break;
                } else {
                    let (s, _) = self.parse_indentation(s, indent + 1, style_errors, true)?;
                    ms = s;
                }
            }

            let (ms, end) = position(ms)?;
            Ok((ms, JsonObject {
                values: values,
                start,
                end,
            }))
        }
    }

    pub fn parse<'a, 'b>(&'a self, s: &'b str) -> Result<(JsonObject<'b>, Vec<(Span<'b>, String)>), nom::error::VerboseError<Span<'b>>> {
        let mut style_errors = Vec::new();
        let (_, json) = self.parse_object(Span::new(s), 0, &mut style_errors).finish()?;

        Ok((
            json,
            style_errors.into_iter().map(|e| {
                error::generate_error_message(
                    &self.style,
                    &e,
                ).unwrap()
            }).collect(),
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn string_parsing() {
        let span = Span::new("\"hello world\"");
        let result = Parser::parse_string(span);
        assert_eq!(result.is_ok(), true);
        let (_, json_string) = result.unwrap();
        assert_eq!(json_string.value, "hello world")
    }

    #[test]
    fn string_parsing_err() {
        let span = Span::new("hello world\"");
        let result = Parser::parse_string(span);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn entry_parsing_string_post_colon_ok() {
        let span = Span::new("\"hello\": \"world\"");
        let parser = Parser {
            style: JsonStyle::IGNORE,
        };
        let mut style_errors = Vec::new();
        let result = parser.parse_entry(span, 0, &mut style_errors);
        assert_eq!(result.is_ok(), true);
        assert_eq!(style_errors.len(), 0);
    }

    #[test]
    fn entry_parsing_string_post_colon_err() {
        let span = Span::new("\"hello\":  \"world\"");
        let parser = Parser {
            style: JsonStyle::STYLED {
                line_endings: LineEnding::IGNORE,
                indentation: None,
                post_colon: Some(" "),
                order: SortOrder::ASC,
                sort_algorithm: SortAlgorithm::NONE,
            },
        };
        let mut style_errors = Vec::new();
        let result = parser.parse_entry(span, 0, &mut style_errors);
        assert_eq!(result.is_ok(), true);
        assert_eq!(style_errors.len(), 1);
    }

    #[test]
    fn entry_parsing_string_post_colon_ignore() {
        let span = Span::new("\"hello\":  \"world\"");
        let parser = Parser {
            style: JsonStyle::STYLED {
                line_endings: LineEnding::IGNORE,
                indentation: None,
                post_colon: None,
                order: SortOrder::ASC,
                sort_algorithm: SortAlgorithm::NONE,
            },
        };
        let mut style_errors = Vec::new();
        let result = parser.parse_entry(span, 0, &mut style_errors);
        assert_eq!(result.is_ok(), true);
        assert_eq!(style_errors.len(), 0);
    }

    #[test]
    fn object_parsing_ignore() {
        let span = Span::new("{\"hello\":\"world\"}");
        let parser = Parser {
            style: JsonStyle::STYLED {
                line_endings: LineEnding::IGNORE,
                indentation: None,
                post_colon: None,
                order: SortOrder::ASC,
                sort_algorithm: SortAlgorithm::NONE,
            },
        };
        let mut style_errors = Vec::new();
        let result = parser.parse_object(span, 0, &mut style_errors);
        assert_eq!(result.is_ok(), true);
        assert_eq!(style_errors.len(), 0);
    }

    #[test]
    fn object_parsing_lf_err() {
        let span = Span::new("{\"hello\":\"world\"}");
        let parser = Parser {
            style: JsonStyle::STYLED {
                line_endings: LineEnding::LF,
                indentation: None,
                post_colon: None,
                order: SortOrder::ASC,
                sort_algorithm: SortAlgorithm::NONE,
            },
        };
        let mut style_errors = Vec::new();
        let result = parser.parse_object(span, 0, &mut style_errors);
        assert_eq!(result.is_ok(), true);
        assert_eq!(style_errors.len(), 2);
    }

    #[test]
    fn object_parsing_lf_ok() {
        let span = Span::new("{\n\"hello\":\"world\"\n}");
        let parser = Parser {
            style: JsonStyle::STYLED {
                line_endings: LineEnding::LF,
                indentation: None,
                post_colon: Some(""),
                order: SortOrder::ASC,
                sort_algorithm: SortAlgorithm::NONE,
            },
        };
        let mut style_errors = Vec::new();
        let result = parser.parse_object(span, 0, &mut style_errors);
        assert_eq!(result.is_ok(), true);
        assert_eq!(style_errors.len(), 0);
    }

    #[test]
    fn object_parsing_post_colon_crlf_err() {
        let span = Span::new("{\r\n\"hello\":\"world\"\n}");
        let parser = Parser {
            style: JsonStyle::STYLED {
                line_endings: LineEnding::CRLF,
                indentation: None,
                post_colon: None,
                order: SortOrder::ASC,
                sort_algorithm: SortAlgorithm::NONE,
            },
        };
        let mut style_errors = Vec::new();
        let result = parser.parse_object(span, 0, &mut style_errors);
        assert_eq!(result.is_ok(), true);
        assert_eq!(style_errors.len(), 1);
    }

    #[test]
    fn object_parsing_crlf_ok() {
        let span = Span::new("{\r\n\"hello\":\"world\"\r\n}");
        let parser = Parser {
            style: JsonStyle::STYLED {
                line_endings: LineEnding::CRLF,
                indentation: None,
                post_colon: Some(""),
                order: SortOrder::ASC,
                sort_algorithm: SortAlgorithm::NONE,
            },
        };
        let mut style_errors = Vec::new();
        let result = parser.parse_object(span, 0, &mut style_errors);
        assert_eq!(result.is_ok(), true);
        println!("{:#?}", &style_errors);
        assert_eq!(style_errors.len(), 0);
    }

    #[test]
    fn object_parsing_indent_post_colon_lf_ok() {
        let span = Span::new("{\n    \"hello\": \"world\"\n}");
        let parser = Parser {
            style: JsonStyle::STYLED {
                line_endings: LineEnding::LF,
                indentation: Some("    "),
                post_colon: Some(" "),
                order: SortOrder::ASC,
                sort_algorithm: SortAlgorithm::NONE,
            },
        };
        let mut style_errors = Vec::new();
        let result = parser.parse_object(span, 0, &mut style_errors);
        assert_eq!(result.is_ok(), true);
        println!("{:#?}", &style_errors);
        assert_eq!(style_errors.len(), 0);
    }

    #[test]
    fn bigger_object_parsing_indent_post_colon_lf_ok() {
        let span = Span::new(
            "{\n    \"hello\": \"world\",\n    \"how\": \"are\"\n}"
        );
        let parser = Parser {
            style: JsonStyle::STYLED {
                line_endings: LineEnding::LF,
                indentation: Some("    "),
                post_colon: Some(" "),
                order: SortOrder::ASC,
                sort_algorithm: SortAlgorithm::NONE,
            },
        };
        let mut style_errors = Vec::new();
        let result = parser.parse_object(span, 0, &mut style_errors);
        assert_eq!(result.is_ok(), true);
        assert_eq!(style_errors.len(), 0);
    }

    #[test]
    fn stacked_object_parsing_indent_post_colon_lf_ok() {
        let span = Span::new(
            "{\n    \"hello\": \"world\",\n    \"how\": {\n    }\n}"
        );
        let parser = Parser {
            style: JsonStyle::STYLED {
                line_endings: LineEnding::LF,
                indentation: Some("    "),
                post_colon: Some(" "),
                order: SortOrder::ASC,
                sort_algorithm: SortAlgorithm::NONE,
            },
        };
        let mut style_errors = Vec::new();
        let result = parser.parse_object(span, 0, &mut style_errors);
        assert_eq!(result.is_ok(), true);
        assert_eq!(style_errors.len(), 0);
    }

    #[test]
    fn stacked_filled_object_parsing_indent_post_colon_lf_ok() {
        let span = Span::new(
            "{\n    \"hello\": \"world\",\n    \"how\": {\n        \"are\": \"you\"\n    }\n}"
        );
        let parser = Parser {
            style: JsonStyle::STYLED {
                line_endings: LineEnding::LF,
                indentation: Some("    "),
                post_colon: Some(" "),
                order: SortOrder::ASC,
                sort_algorithm: SortAlgorithm::NONE,
            },
        };
        let mut style_errors = Vec::new();
        let result = parser.parse_object(span, 0, &mut style_errors);
        assert_eq!(result.is_ok(), true);
        assert_eq!(style_errors.len(), 0);
    }

    #[test]
    fn object_parsing_indent_post_colon_lf_err() {
        let s = [
            "{\n\"hello\": \"world\"\n}",
            "{    \"hello\": \"world\"\n}",
            "{\n     \"hello\": \"world\"\n}",
            "{\n    \"hello\":  \"world\"\n}",
            "{\n    \"hello\": \"world\" \n}",
            "{\n    \"hello\": \"world\"}",
        ];
        s.iter().for_each(|s| {
            let span = Span::new(*s);
            let parser = Parser {
                style: JsonStyle::STYLED {
                    line_endings: LineEnding::LF,
                    indentation: Some("    "),
                    post_colon: Some(" "),
                    order: SortOrder::ASC,
                    sort_algorithm: SortAlgorithm::NONE,
                },
            };
            let mut style_errors = Vec::new();
            let result = parser.parse_object(span, 0, &mut style_errors);
            assert_eq!(result.is_ok(), true);
            assert_eq!(style_errors.len(), 1);
        });
    }
}
