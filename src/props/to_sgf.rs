use std::collections::HashSet;

use crate::{Color, Double, SimpleText, Text};

pub trait ToSgf {
    fn to_sgf(&self) -> String;
}

impl ToSgf for Vec<String> {
    fn to_sgf(&self) -> String {
        self.join("][")
    }
}

impl<P: ToSgf> ToSgf for HashSet<P> {
    fn to_sgf(&self) -> String {
        self.iter()
            .map(|x| x.to_sgf())
            .collect::<Vec<String>>()
            .join("][")
    }
}

impl<A: ToSgf, B: ToSgf> ToSgf for (A, B) {
    fn to_sgf(&self) -> String {
        format!("{}:{}", self.0.to_sgf(), self.1.to_sgf())
    }
}

impl<T: ToSgf> ToSgf for Option<T> {
    fn to_sgf(&self) -> String {
        match self {
            None => "".to_string(),
            Some(x) => x.to_sgf(),
        }
    }
}

impl ToSgf for u8 {
    fn to_sgf(&self) -> String {
        self.to_string()
    }
}

impl ToSgf for i64 {
    fn to_sgf(&self) -> String {
        self.to_string()
    }
}

impl ToSgf for f64 {
    fn to_sgf(&self) -> String {
        self.to_string()
    }
}

impl ToSgf for Double {
    fn to_sgf(&self) -> String {
        match self {
            Self::One => "1".to_string(),
            Self::Two => "2".to_string(),
        }
    }
}

impl ToSgf for Color {
    fn to_sgf(&self) -> String {
        match self {
            Self::Black => "B".to_string(),
            Self::White => "W".to_string(),
        }
    }
}

impl ToSgf for Text {
    fn to_sgf(&self) -> String {
        escape_string(&self.text)
    }
}

impl ToSgf for SimpleText {
    fn to_sgf(&self) -> String {
        escape_string(&self.text)
    }
}

fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace(']', "\\]")
        .replace(':', "\\:")
}
