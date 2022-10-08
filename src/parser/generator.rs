use crate::parser::model::{JsonObject, JsonString, JsonStyle, JsonType, LineEnding};
use crate::{SortAlgorithm, SortOrder};

pub struct Generator {
    style: JsonStyle
}

impl Generator {
    pub fn new(style: JsonStyle) -> Generator {
        Generator {
            style,
        }
    }

    pub fn generate(&self, mut json: JsonObject) -> String {
        let mut buffer = String::new();
        match self.style {
            JsonStyle::STYLED { sort_algorithm, order, .. } => {
                match sort_algorithm {
                    SortAlgorithm::NATURAL => {
                        match order {
                            SortOrder::ASC => {
                                json.sort_by(|a, b| {
                                    crate::natural_sort::compare(a.value, b.value)
                                })
                            }
                            SortOrder::DESC => {
                                json.sort_by(|a, b| {
                                    crate::natural_sort::compare(a.value, b.value).reverse()
                                })
                            }
                        }
                    }
                    SortAlgorithm::NORMAL => {
                        match order {
                            SortOrder::ASC => {
                                json.sort_by(|a, b| {
                                    a.value.cmp(b.value)
                                })
                            }
                            SortOrder::DESC => {
                                json.sort_by(|a, b| {
                                    a.value.cmp(b.value).reverse()
                                })
                            }
                        }
                    }
                    SortAlgorithm::NONE => {}
                }
            }
            JsonStyle::IGNORE => {}
        }
        self.generate_object(json, 0, &mut buffer);

        buffer
    }

    fn new_line(&self, buffer: &mut String) {
        match self.style {
            JsonStyle::STYLED { line_endings, .. } => {
                match line_endings {
                    LineEnding::CRLF => buffer.push_str("\r\n"),
                    LineEnding::LF | LineEnding::ANY | LineEnding::IGNORE => buffer.push('\n'),
                    LineEnding::NONE => {},
                }
            }
            JsonStyle::IGNORE => {},
        }
    }

    fn new_indent(&self, indent: u64, buffer: &mut String) {
        match self.style {
            JsonStyle::STYLED { indentation, .. } => {
                if let Some(indentation) = indentation {
                    for _ in 0..indent {
                        buffer.push_str(indentation);
                    }
                } else {
                    for _ in 0..indent {
                        buffer.push_str("    "); // Default
                    }
                }
            }
            JsonStyle::IGNORE => {},
        }
    }

    fn generate_object(&self, json: JsonObject, indent: u64, buffer: &mut String) {
        buffer.push('{');
        self.new_line(buffer);
        let mut values = json.values.into_iter().peekable();
        while let Some(value) = values.next() {
            self.new_indent(indent + 1, buffer);
            self.generate_entry(value, indent + 1, buffer);
            if values.peek().is_some() {
                buffer.push(',');
            }
            self.new_line(buffer);
        }
        self.new_indent(indent, buffer);
        buffer.push('}');
    }

    fn generate_entry(&self, (key, value): (JsonString, JsonType), indent: u64, buffer: &mut String) {
        buffer.push('\"');
        buffer.push_str(key.value);
        buffer.push('\"');
        buffer.push(':');
        match self.style {
            JsonStyle::STYLED { post_colon, .. } => {
                if let Some(post_colon) = post_colon {
                    buffer.push_str(post_colon);
                } else {
                    buffer.push(' ');
                }
            }
            JsonStyle::IGNORE => {}
        }
        match value {
            JsonType::Object(object) => {
                self.generate_object(object, indent, buffer);
            }
            JsonType::String(string) => {
                buffer.push('\"');
                buffer.push_str(string.value);
                buffer.push('\"');
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::parser::model::Span;
    use crate::{SortAlgorithm, SortOrder};
    use super::*;

    #[test]
    fn generate_style_1_empty() {
        let generator = Generator {
            style: JsonStyle::STYLED {
                line_endings: LineEnding::LF,
                indentation: Some("    "),
                post_colon: Some(" "),
                order: SortOrder::ASC,
                sort_algorithm: SortAlgorithm::NONE,
            },
        };
        let generated = generator.generate(JsonObject {
            start: Span::new(""),
            end: Span::new(""),
            values: Vec::from([]),
        });

        assert_eq!(generated, "{\n}")
    }

    #[test]
    fn generate_style_1_filled() {
        let generator = Generator {
            style: JsonStyle::STYLED {
                line_endings: LineEnding::LF,
                indentation: Some("    "),
                post_colon: Some(" "),
                order: SortOrder::ASC,
                sort_algorithm: SortAlgorithm::NONE,
            },
        };
        let generated = generator.generate(JsonObject {
            start: Span::new(""),
            end: Span::new(""),
            values: Vec::from([
                (JsonString {
                    start: Span::new(""),
                    end: Span::new(""),
                    value: "hello",
                },
                 JsonType::String(JsonString {
                     start: Span::new(""),
                     end: Span::new(""),
                     value: "world",
                 }))
            ]),
        });

        assert_eq!(generated, "{\n    \"hello\": \"world\"\n}")
    }

    #[test]
    fn generate_style_1_stacked() {
        let generator = Generator {
            style: JsonStyle::STYLED {
                line_endings: LineEnding::LF,
                indentation: Some("    "),
                post_colon: Some(" "),
                order: SortOrder::ASC,
                sort_algorithm: SortAlgorithm::NONE,
            },
        };
        let generated = generator.generate(JsonObject {
            start: Span::new(""),
            end: Span::new(""),
            values: Vec::from([
                (JsonString {
                    start: Span::new(""),
                    end: Span::new(""),
                    value: "hello",
                },
                 JsonType::String(JsonString {
                     start: Span::new(""),
                     end: Span::new(""),
                     value: "world",
                 })),
                (JsonString {
                    start: Span::new(""),
                    end: Span::new(""),
                    value: "how",
                },
                 JsonType::Object(JsonObject {
                     start: Span::new(""),
                     end: Span::new(""),
                     values: Vec::from([
                         (JsonString {
                             start: Span::new(""),
                             end: Span::new(""),
                             value: "are",
                         },
                          JsonType::String(JsonString {
                              start: Span::new(""),
                              end: Span::new(""),
                              value: "you",
                          })),
                     ])
                 }))
            ]),
        });

        assert_eq!(
            generated,
            "{\n    \"hello\": \"world\",\n    \"how\": {\n        \"are\": \"you\"\n    }\n}"
        )
    }

    #[test]
    fn generate_style_2_stacked() {
        let generator = Generator {
            style: JsonStyle::STYLED {
                line_endings: LineEnding::NONE,
                indentation: Some(""),
                post_colon: Some(""),
                order: SortOrder::ASC,
                sort_algorithm: SortAlgorithm::NONE,
            },
        };
        let generated = generator.generate(JsonObject {
            start: Span::new(""),
            end: Span::new(""),
            values: Vec::from([
                (JsonString {
                    start: Span::new(""),
                    end: Span::new(""),
                    value: "hello",
                },
                 JsonType::String(JsonString {
                     start: Span::new(""),
                     end: Span::new(""),
                     value: "world",
                 })),
                (JsonString {
                    start: Span::new(""),
                    end: Span::new(""),
                    value: "how",
                },
                 JsonType::Object(JsonObject {
                     start: Span::new(""),
                     end: Span::new(""),
                     values: Vec::from([
                         (JsonString {
                             start: Span::new(""),
                             end: Span::new(""),
                             value: "are",
                         },
                          JsonType::String(JsonString {
                              start: Span::new(""),
                              end: Span::new(""),
                              value: "you",
                          })),
                     ])
                 }))
            ]),
        });

        assert_eq!(
            generated,
            "{\"hello\":\"world\",\"how\":{\"are\":\"you\"}}"
        )
    }

    #[test]
    fn generate_style_3_stacked() {
        let generator = Generator {
            style: JsonStyle::STYLED {
                line_endings: LineEnding::CRLF,
                indentation: Some("  "),
                post_colon: Some(" "),
                order: SortOrder::ASC,
                sort_algorithm: SortAlgorithm::NONE,
            },
        };
        let generated = generator.generate(JsonObject {
            start: Span::new(""),
            end: Span::new(""),
            values: Vec::from([
                (JsonString {
                    start: Span::new(""),
                    end: Span::new(""),
                    value: "hello",
                },
                 JsonType::String(JsonString {
                     start: Span::new(""),
                     end: Span::new(""),
                     value: "world",
                 })),
                (JsonString {
                    start: Span::new(""),
                    end: Span::new(""),
                    value: "how",
                },
                 JsonType::Object(JsonObject {
                     start: Span::new(""),
                     end: Span::new(""),
                     values: Vec::from([
                         (JsonString {
                             start: Span::new(""),
                             end: Span::new(""),
                             value: "are",
                         },
                          JsonType::String(JsonString {
                              start: Span::new(""),
                              end: Span::new(""),
                              value: "you",
                          })),
                     ])
                 }))
            ]),
        });

        assert_eq!(
            generated,
            "{\r\n  \"hello\": \"world\",\r\n  \"how\": {\r\n    \"are\": \"you\"\r\n  }\r\n}"
        )
    }
}
