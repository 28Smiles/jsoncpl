use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

use nom::{bytes::complete::{tag, take}, character::complete::{multispace0, one_of}, InputTake, IResult};
use nom_locate::{LocatedSpan, position};

#[cfg(test)]
use serde_json::Value;

pub type Span<'a> = LocatedSpan<&'a str>;

#[derive(Debug, Clone)]
pub enum JsonType<'a> {
    Object(JsonObject<'a>),
    String(JsonString<'a>)
}

#[derive(Debug)]
pub struct JsonObject<'a> {
    pub values: Vec<(JsonString<'a>, JsonType<'a>)>,
    pub position: Span<'a>
}

#[derive(Debug, Clone, Copy)]
pub struct JsonString<'a> {
    pub value: &'a str,
    pub position: Span<'a>
}

impl <'a>Clone for JsonObject<'a> {
    fn clone(&self) -> Self {
        JsonObject {
            values: self.values.iter().map(|(key, value)| {
                (*key, value.clone())
            }).collect(),
            position: self.position
        }
    }
}

pub trait Pretty {
    fn pretty_impl(&self, buf: &mut String, indent: i32, current_indent: i32, line_ending: &'static str);
    fn pretty(&self, indent: i32, current_indent: i32, line_ending: &'static str) -> String {
        let mut buf = String::new();
        self.pretty_impl(&mut buf, indent, current_indent, line_ending);

        return buf;
    }
}

impl <'a>Pretty for JsonType<'a> {
    fn pretty_impl(&self, buf: &mut String, indent: i32, current_indent: i32, line_ending: &'static str) {
        match self {
            JsonType::Object(value) => {
                value.pretty_impl(buf, indent, current_indent, line_ending);
            }
            JsonType::String(value) => {
                value.pretty_impl(buf, indent, current_indent, line_ending);
            }
        }
    }
}

impl <'a>Pretty for JsonString<'a> {
    fn pretty_impl(&self, buf: &mut String, _: i32, _: i32, _: &'static str) {
        buf.push('"');
        buf.push_str(self.value);
        buf.push('"');
    }
}

impl <'a>Pretty for JsonObject<'a> {
    fn pretty_impl(&self, buf: &mut String, indent: i32, current_indent: i32, line_ending: &'static str) {
        buf.push('{');
        buf.push_str(line_ending);
        for (key, value) in &self.values {
            for _ in 0..current_indent {
                buf.push(' ');
            }
            key.pretty_impl(buf, indent, current_indent, line_ending);
            buf.push_str(": ");
            value.pretty_impl(buf, indent, current_indent + indent, line_ending);
            buf.push(',');
            buf.push_str(line_ending);
        }
        buf.pop().unwrap();
        if self.values.len() > 0 {
            for _ in 0..line_ending.len() {
                buf.pop().unwrap(); // Remove trailing comma
            }
            buf.push_str(line_ending);
            for _ in 0..(current_indent - indent) {
                buf.push(' ');
            }
        }
        buf.push('}');
    }
}

impl <'s>Hash for JsonString<'s> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl <'s>PartialEq for JsonString<'s> {
    fn eq(&self, other: &Self) -> bool {
        self.value.eq(other.value)
    }
}

impl <'s>Eq for JsonString<'s> {
}

impl <'s>PartialOrd for JsonString<'s> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.value.partial_cmp(other.value)
    }
}

impl <'s>Ord for JsonString<'s> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value.cmp(other.value)
    }
}

fn parse_string(s: Span) -> IResult<Span, JsonString> {
    let (s, _) = multispace0(s)?;
    let (s, _) = tag("\"")(s)?;
    let (s, pos) = position(s)?;
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

    Ok((s, JsonString {
        value: *contents,
        position: pos
    }))
}

fn parse_entry(s: Span) -> IResult<Span, (JsonString, JsonType)> {
    let (s, key) = parse_string(s)?;
    let (s, _) = multispace0(s)?;
    let (s, _) = tag(":")(s)?;
    let (s, _) = multispace0(s)?;
    let (_, c) = one_of("{\"")(s)?;

    if c == '\"' {
        let (s, value) = parse_string(s)?;

        Ok((s, (key, JsonType::String(value))))
    } else {
        let (s, value) = parse_object(s)?;

        Ok((s, (key, JsonType::Object(value))))
    }
}

fn parse_object(s: Span) -> IResult<Span, JsonObject> {
    let (s, _) = multispace0(s)?;
    let (s, _) = tag("{")(s)?;
    let (s, pos) = position(s)?;
    let (s, _) = multispace0(s)?;
    let (_, c) = one_of("}\"")(s)?;

    if c == '\"' {
        let mut values = Vec::new();
        let mut ms= s;
        loop {
            let (s, kv) = parse_entry(ms)?;
            values.push(kv);
            let (s, _) = multispace0(s)?;
            let (s, c) = one_of("},")(s)?;
            ms = s;

            if c == '}' {
                break;
            }
        }

        Ok((ms, JsonObject {
            values: values,
            position: pos
        }))
    } else {
        let (s, _) = tag("}")(s)?;

        Ok((s, JsonObject {
            values: Vec::new(),
            position: pos
        }))
    }
}

pub fn parse_root(s: &String) -> Option<JsonObject> {
    parse_object(Span::new(s)).ok().map(|(_, o)| o)
}

#[cfg(test)]
fn assert_eq(r: Value, c: JsonType) {
    match r {
        Value::String(rs) => {
            match c {
                JsonType::Object(_) => { panic!("Expected String, found Object"); }
                JsonType::String(cs) => { assert_eq!(&*rs, &*unescape::unescape(cs.value).unwrap()); }
            }
        }
        Value::Object(ro) => {
            match c {
                JsonType::Object(co) => {
                    for ((rkey, rvalue), (ckey, cvalue)) in ro.into_iter().zip(co.values) {
                        assert_eq!(&*rkey, ckey.value);
                        assert_eq(rvalue, cvalue);
                    }
                }
                JsonType::String(_) => { panic!("Expected Object, found String"); }
            }
        }
        _ => {
            panic!("Unknown Type");
        }
    }
}

#[cfg(test)]
fn test_compare(content: String) {
    let parsed = parse_root(&content).expect("Could not parse json");
    let reference: Value = serde_json::from_str(content.as_str())
        .expect("Could not parse json");
    assert_eq(reference, JsonType::Object(parsed));
}

#[test]
fn parse_simple() {
    let content = String::from("{ \"hello\": \"world\" }");
    let result = parse_root(&content);
    assert_eq!(result.is_some(), true);
    let values: &Vec<(JsonString, JsonType)> = &result.unwrap().values;
    assert_eq!(values.len(), 1);
    let (key, value) = values.get(0).unwrap();
    assert_eq!(key.value, "hello");
    match value {
        JsonType::Object(_) => assert_eq!(true, false),
        JsonType::String(s) => assert_eq!(s.value, "world")
    }
}

#[test]
fn parse_file_test1() {
    let content = include_str!("../testdata/json/test1.json");
    let content = String::from(content);
    test_compare(content);
}

#[test]
fn parse_file_test2() {
    let content = include_str!("../testdata/json/test2.json");
    let content = String::from(content);
    test_compare(content);
}

#[test]
fn parse_file_test3() {
    let content = include_str!("../testdata/json/test3.json");
    let content = String::from(content);
    test_compare(content);
}

#[test]
fn parse_file_test4() {
    let content = include_str!("../testdata/json/test4.json");
    let content = String::from(content);
    test_compare(content);
}

#[test]
fn pretty_file_test1() {
    let content = include_str!("../testdata/json/test1.json");
    let content = String::from(content);

    let parsed = parse_root(&content).expect("Could not parse json");
    let parsed_pretty = parsed.pretty(2, 2, "\n");
    assert_eq!(&*parsed_pretty, content);
}

#[test]
fn pretty_file_test2() {
    let content = include_str!("../testdata/json/test2.json");
    let content = String::from(content);

    let parsed = parse_root(&content).expect("Could not parse json");
    let parsed_pretty = parsed.pretty(2, 2, "\n");
    assert_eq!(&*parsed_pretty, content);
}

#[test]
fn pretty_file_test3() {
    let content = include_str!("../testdata/json/test3.json");
    let content = String::from(content);

    let parsed = parse_root(&content).expect("Could not parse json");
    let parsed_pretty = parsed.pretty(2, 2, "\n");
    assert_eq!(&*parsed_pretty, content);
}

#[test]
fn pretty_file_test4() {
    let content = include_str!("../testdata/json/test4.json");
    let content = String::from(content);

    let parsed = parse_root(&content).expect("Could not parse json");
    let parsed_pretty = parsed.pretty(2, 2, "\n");
    assert_eq!(&*parsed_pretty, content);
}