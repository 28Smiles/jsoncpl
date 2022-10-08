use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use nom_locate::LocatedSpan;

pub type Span<'a> = LocatedSpan<&'a str>;

#[derive(Debug, Copy, Clone)]
pub enum JsonStyle {
    STYLED {
        line_endings: LineEnding,
        indentation: Option<&'static str>,
        post_colon: Option<&'static str>,
        sort_algorithm: SortAlgorithm,
        order: SortOrder,
    },
    IGNORE
}

#[derive(Debug, Copy, Clone)]
pub enum SortOrder {
    ASC,
    DESC,
}

#[derive(Debug, Copy, Clone)]
pub enum SortAlgorithm {
    NATURAL,
    NORMAL,
    NONE,
}

#[derive(Debug, Copy, Clone)]
pub enum LineEnding {
    CRLF,
    LF,
    NONE,
    ANY,
    IGNORE
}

#[derive(Debug, Clone)]
pub enum JsonType<'a> {
    Object(JsonObject<'a>),
    String(JsonString<'a>)
}

#[derive(Debug)]
pub struct JsonObject<'a> {
    pub values: Vec<(JsonString<'a>, JsonType<'a>)>,
    pub start: Span<'a>,
    pub end: Span<'a>,
}

impl<'a> JsonObject<'a> {
    pub fn sort_by<F>(&mut self, mut compare: F)
        where
            F: FnMut(&JsonString<'a>, &JsonString<'a>) -> Ordering + Copy, {
        self.values.sort_by(|(a, _), (b, _)| compare(a, b));
        self.values.iter_mut().for_each(|(_, value)| {
            if let JsonType::Object(object) = value {
                object.sort_by(compare);
            }
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct JsonString<'a> {
    pub value: &'a str,
    pub start: Span<'a>,
    pub end: Span<'a>,
}

impl <'a>Clone for JsonObject<'a> {
    fn clone(&self) -> Self {
        JsonObject {
            values: self.values.iter().map(|(key, value)| {
                (*key, value.clone())
            }).collect(),
            start: self.start,
            end: self.end,
        }
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