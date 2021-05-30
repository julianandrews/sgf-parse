use std::collections::HashSet;
use std::fmt;

use super::traits::{Game, ToSgf};
use super::{props, SgfNode};
use crate::game::GameTree;

/// Returns the SGF as a `String` from a collection of `SgfNode` objects.
///
/// # Examples
/// ```
/// use sgf_parse::{parse, serialize};
///
/// let original = "(;SZ[13:13];B[de](;W[ef])(;W[de];B[ac]))";
/// let gametrees = parse(original).unwrap();
/// let parsed = serialize(&gametrees);
///
/// assert_eq!(parsed, original);
/// ```
pub fn serialize<'a>(nodes: impl IntoIterator<Item = &'a GameTree>) -> String {
    let node_text = nodes
        .into_iter()
        .map(|node| node.to_string())
        .collect::<Vec<String>>()
        .join(")(");
    format!("({})", node_text)
}

impl<G: Game> fmt::Display for SgfNode<G> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let prop_string = self
            .properties()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join("");
        let child_count = self.children().count();
        let child_string = match child_count {
            0 => "".to_string(),
            1 => self.children().next().unwrap().to_string(),
            _ => self
                .children()
                .map(|x| format!("({})", x.to_string()))
                .collect::<Vec<_>>()
                .join(""),
        };
        write!(f, ";{}{}", prop_string, child_string)
    }
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

impl ToSgf for props::Double {
    fn to_sgf(&self) -> String {
        match self {
            Self::One => "1".to_string(),
            Self::Two => "2".to_string(),
        }
    }
}

impl ToSgf for props::Color {
    fn to_sgf(&self) -> String {
        match self {
            Self::Black => "B".to_string(),
            Self::White => "W".to_string(),
        }
    }
}

impl ToSgf for props::Text {
    fn to_sgf(&self) -> String {
        escape_string(&self.text)
    }
}

impl ToSgf for props::SimpleText {
    fn to_sgf(&self) -> String {
        escape_string(&self.text)
    }
}

impl<G: Game> fmt::Display for props::SgfProp<G> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let prop_string = match self {
            Self::B(x) => x.to_sgf(),
            Self::KO => "".to_string(),
            Self::MN(x) => x.to_sgf(),
            Self::W(x) => x.to_sgf(),
            Self::AB(x) => x.to_sgf(),
            Self::AE(x) => x.to_sgf(),
            Self::AW(x) => x.to_sgf(),
            Self::PL(x) => x.to_sgf(),
            Self::C(x) => x.to_sgf(),
            Self::DM(x) => x.to_sgf(),
            Self::GB(x) => x.to_sgf(),
            Self::GW(x) => x.to_sgf(),
            Self::HO(x) => x.to_sgf(),
            Self::N(x) => x.to_sgf(),
            Self::UC(x) => x.to_sgf(),
            Self::V(x) => x.to_sgf(),
            Self::AR(x) => x.to_sgf(),
            Self::CR(x) => x.to_sgf(),
            Self::DO => "".to_string(),
            Self::IT => "".to_string(),
            Self::BM(x) => x.to_sgf(),
            Self::TE(x) => x.to_sgf(),
            Self::DD(x) => x.to_sgf(),
            Self::LB(x) => x.to_sgf(),
            Self::LN(x) => x.to_sgf(),
            Self::MA(x) => x.to_sgf(),
            Self::SL(x) => x.to_sgf(),
            Self::SQ(x) => x.to_sgf(),
            Self::TR(x) => x.to_sgf(),
            Self::AP(x) => x.to_sgf(),
            Self::CA(x) => x.to_sgf(),
            Self::FF(x) => x.to_sgf(),
            Self::GM(x) => x.to_sgf(),
            Self::ST(x) => x.to_sgf(),
            Self::SZ(x) => x.to_sgf(),
            Self::HA(x) => x.to_sgf(),
            Self::KM(x) => x.to_sgf(),
            Self::AN(x) => x.to_sgf(),
            Self::BR(x) => x.to_sgf(),
            Self::BT(x) => x.to_sgf(),
            Self::CP(x) => x.to_sgf(),
            Self::DT(x) => x.to_sgf(),
            Self::EV(x) => x.to_sgf(),
            Self::GN(x) => x.to_sgf(),
            Self::GC(x) => x.to_sgf(),
            Self::ON(x) => x.to_sgf(),
            Self::OT(x) => x.to_sgf(),
            Self::PB(x) => x.to_sgf(),
            Self::PC(x) => x.to_sgf(),
            Self::PW(x) => x.to_sgf(),
            Self::RE(x) => x.to_sgf(),
            Self::RO(x) => x.to_sgf(),
            Self::RU(x) => x.to_sgf(),
            Self::SO(x) => x.to_sgf(),
            Self::TM(x) => x.to_sgf(),
            Self::US(x) => x.to_sgf(),
            Self::WR(x) => x.to_sgf(),
            Self::WT(x) => x.to_sgf(),
            Self::BL(x) => x.to_sgf(),
            Self::OB(x) => x.to_sgf(),
            Self::OW(x) => x.to_sgf(),
            Self::WL(x) => x.to_sgf(),
            Self::FG(x) => x.to_sgf(),
            Self::TB(x) => x.to_sgf(),
            Self::TW(x) => x.to_sgf(),
            Self::PM(x) => x.to_sgf(),
            Self::VW(x) => x.to_sgf(),
            Self::Unknown(_, x) => x.to_sgf(),
            Self::Invalid(_, x) => x.to_sgf(),
        };
        write!(f, "{}[{}]", self.identifier(), prop_string)
    }
}

fn escape_string(s: &str) -> String {
    s.replace("\\", "\\\\")
        .replace("]", "\\]")
        .replace(":", "\\:")
}

#[cfg(test)]
mod test {
    use super::super::parse;
    use super::serialize;

    #[test]
    pub fn simple_sgf() {
        let sgf = "(;C[Some comment];B[de]FOO[bar][baz];W[fe])(;B[de];W[ff])";
        let game_trees = parse(sgf).unwrap();
        let result = serialize(&game_trees);
        assert_eq!(result, sgf);
    }
}
