use std::collections::HashSet;

use super::SgfParseError;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Double {
    One,
    Two,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Point {
    pub x: u8,
    pub y: u8,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Move {
    Pass,
    Move(Point),
}

#[derive(Clone, Debug)]
pub struct Text(String);

impl std::ops::Deref for Text {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug)]
pub struct SimpleText(String);

impl std::ops::Deref for SimpleText {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug)]
pub enum SgfProp {
    // Move Properties
    B(Move),
    KO,
    MN(i64),
    W(Move),
    // Setup Properties (illegal to place two colors on one point)
    AB(Vec<Point>),
    AE(Vec<Point>),
    AW(Vec<Point>),
    // Node Annotation properties
    C(Text),
    DM(Double),
    GB(Double),
    GW(Double),
    HO(Double),
    N(SimpleText),
    UC(Double),
    V(f64),
    // Move annotation properties (illegal without a move in the node)
    BM(Double),
    DO,
    IT,
    TE(Double),
    // Markup Properties (illegal to have more than one on a point)
    // TODO: AR(list of point:point) - validate unique, distinct points
    CR(Vec<Point>),
    DD(Vec<Point>),
    // TODO: LB(list of point:simpletext)
    // TODO: LN(list of point:point) - validate unique, distinct points
    MA(Vec<Point>),
    SL(Vec<Point>),
    SQ(Vec<Point>),
    TR(Vec<Point>),
    // Root Properties
    // AP(simpletext:simpletext)
    CA(SimpleText),
    FF(i64), // range 1-4
    GM(i64), // range 1-16, only handle Go = 1!
    ST(i64), // range 0-3
    SZ((u8, u8)),
    // Game info properties
    HA(i64), // >=2, AB should be set within same node
    KM(f64),
    AN(SimpleText),
    BR(SimpleText),
    BT(SimpleText),
    CP(SimpleText),
    DT(SimpleText),
    EV(SimpleText),
    GN(SimpleText),
    GC(Text),
    ON(SimpleText),
    OT(SimpleText),
    PB(SimpleText),
    PC(SimpleText),
    PW(SimpleText),
    RE(SimpleText),
    RO(SimpleText),
    RU(SimpleText),
    SO(SimpleText),
    TM(f64),
    US(SimpleText),
    WR(SimpleText),
    WT(SimpleText),
    // Timing Properties
    BL(f64),
    OB(i64),
    OW(i64),
    WL(f64),
    // Miscellaneous properties
    // TODO: FG (nothing | num:simpletext)
    PM(i64), // range 1-2
    VW(Vec<Point>),
    TB(Vec<Point>),
    TW(Vec<Point>),
    Unknown(String, Vec<String>),
}

impl SgfProp {
    pub fn new(ident: String, values: Vec<String>) -> Result<SgfProp, SgfParseError> {
        match &ident[..] {
            "B" => Ok(SgfProp::B(parse_single_value(&values)?)),
            "KO" => verify_empty(&values).map(|()| Ok(SgfProp::KO))?,
            "MN" => Ok(SgfProp::MN(parse_single_value(&values)?)),
            "W" => Ok(SgfProp::W(parse_single_value(&values)?)),
            "AB" => Ok(SgfProp::AB(parse_list_point(&values)?)),
            "AE" => Ok(SgfProp::AE(parse_list_point(&values)?)),
            "AW" => Ok(SgfProp::AW(parse_list_point(&values)?)),
            "C" => Ok(SgfProp::C(parse_single_value(&values)?)),
            "DM" => Ok(SgfProp::DM(parse_single_value(&values)?)),
            "GB" => Ok(SgfProp::GB(parse_single_value(&values)?)),
            "GW" => Ok(SgfProp::GW(parse_single_value(&values)?)),
            "HO" => Ok(SgfProp::HO(parse_single_value(&values)?)),
            "N" => Ok(SgfProp::N(parse_single_value(&values)?)),
            "UC" => Ok(SgfProp::UC(parse_single_value(&values)?)),
            "V" => Ok(SgfProp::V(parse_single_value(&values)?)),
            "DO" => verify_empty(&values).map(|()| Ok(SgfProp::DO))?,
            "IT" => verify_empty(&values).map(|()| Ok(SgfProp::IT))?,
            "BM" => Ok(SgfProp::BM(parse_single_value(&values)?)),
            "TE" => Ok(SgfProp::TE(parse_single_value(&values)?)),
            "CR" => Ok(SgfProp::CR(parse_list_point(&values)?)),
            "DD" => Ok(SgfProp::DD(parse_elist_point(&values)?)),
            "MA" => Ok(SgfProp::MA(parse_list_point(&values)?)),
            "SL" => Ok(SgfProp::SL(parse_list_point(&values)?)),
            "SQ" => Ok(SgfProp::SQ(parse_list_point(&values)?)),
            "TR" => Ok(SgfProp::TR(parse_list_point(&values)?)),
            "CA" => Ok(SgfProp::CA(parse_single_value(&values)?)),
            "FF" => {
                let value = parse_single_value(&values)?;
                if value < 0 || value > 4 {
                    Err(SgfParseError::InvalidPropertyValue)?;
                }
                Ok(SgfProp::FF(value))
            }
            "GM" => {
                let value = parse_single_value(&values)?;
                // Only Go is supported
                if value != 1 {
                    Err(SgfParseError::InvalidPropertyValue)?;
                }
                Ok(SgfProp::GM(value))
            }
            "ST" => {
                let value = parse_single_value(&values)?;
                if value < 0 || value > 3 {
                    Err(SgfParseError::InvalidPropertyValue)?;
                }
                Ok(SgfProp::ST(value))
            }
            "SZ" => Ok(SgfProp::SZ(parse_size(&values)?)),
            "HA" => {
                let value: i64 = parse_single_value(&values)?;
                if !value >= 2 {
                    Err(SgfParseError::InvalidPropertyValue)?;
                }
                Ok(SgfProp::HA(value))
            }
            "KM" => Ok(SgfProp::KM(parse_single_value(&values)?)),
            "AN" => Ok(SgfProp::AN(parse_single_value(&values)?)),
            "BR" => Ok(SgfProp::BR(parse_single_value(&values)?)),
            "BT" => Ok(SgfProp::BT(parse_single_value(&values)?)),
            "CP" => Ok(SgfProp::CP(parse_single_value(&values)?)),
            "DT" => Ok(SgfProp::DT(parse_single_value(&values)?)),
            "EV" => Ok(SgfProp::EV(parse_single_value(&values)?)),
            "GN" => Ok(SgfProp::GN(parse_single_value(&values)?)),
            "GC" => Ok(SgfProp::GC(parse_single_value(&values)?)),
            "ON" => Ok(SgfProp::ON(parse_single_value(&values)?)),
            "OT" => Ok(SgfProp::OT(parse_single_value(&values)?)),
            "PB" => Ok(SgfProp::PB(parse_single_value(&values)?)),
            "PC" => Ok(SgfProp::PC(parse_single_value(&values)?)),
            "PW" => Ok(SgfProp::PW(parse_single_value(&values)?)),
            "RE" => Ok(SgfProp::RE(parse_single_value(&values)?)),
            "RO" => Ok(SgfProp::RO(parse_single_value(&values)?)),
            "RU" => Ok(SgfProp::RU(parse_single_value(&values)?)),
            "SO" => Ok(SgfProp::SO(parse_single_value(&values)?)),
            "TM" => Ok(SgfProp::TM(parse_single_value(&values)?)),
            "US" => Ok(SgfProp::US(parse_single_value(&values)?)),
            "WR" => Ok(SgfProp::WR(parse_single_value(&values)?)),
            "WT" => Ok(SgfProp::WT(parse_single_value(&values)?)),
            "BL" => Ok(SgfProp::BL(parse_single_value(&values)?)),
            "OB" => Ok(SgfProp::OB(parse_single_value(&values)?)),
            "OW" => Ok(SgfProp::OW(parse_single_value(&values)?)),
            "WL" => Ok(SgfProp::WL(parse_single_value(&values)?)),
            "PM" => {
                let value = parse_single_value(&values)?;
                if value < 1 || value > 2 {
                    Err(SgfParseError::InvalidPropertyValue)?;
                }
                Ok(SgfProp::PM(value))
            }
            "VW" => Ok(SgfProp::VW(parse_elist_point(&values)?)),
            "TB" => Ok(SgfProp::TB(parse_elist_point(&values)?)),
            "TW" => Ok(SgfProp::TW(parse_elist_point(&values)?)),
            _ => Ok(SgfProp::Unknown(ident, values)),
        }
    }
}

fn verify_empty(values: &Vec<String>) -> Result<(), SgfParseError> {
    if !(values.len() == 0 || (values.len() == 1 && values[0].is_empty())) {
        Err(SgfParseError::InvalidPropertyValue)?;
    }
    Ok(())
}

fn parse_single_value<T: std::str::FromStr>(values: &Vec<String>) -> Result<T, SgfParseError> {
    if values.len() != 1 {
        Err(SgfParseError::InvalidPropertyValue)?;
    }
    values[0]
        .parse()
        .map_err(|_| SgfParseError::InvalidPropertyValue)
}

fn parse_list_point(values: &Vec<String>) -> Result<Vec<Point>, SgfParseError> {
    let points = parse_elist_point(values)?;
    if points.is_empty() {
        Err(SgfParseError::InvalidPropertyValue)?;
    }

    Ok(points)
}

fn parse_elist_point(values: &Vec<String>) -> Result<Vec<Point>, SgfParseError> {
    let mut points = HashSet::new();
    for value in values.iter() {
        let parts: Vec<&str> = value.split(":").collect();
        if parts.len() == 1 {
            let point = parts[0].parse()?;
            if points.contains(&point) {
                Err(SgfParseError::InvalidPropertyValue)?;
            }
            points.insert(point);
        } else if parts.len() == 2 {
            let upper_left: Point = parts[0].parse()?;
            let lower_right: Point = parts[1].parse()?;
            if upper_left.x > lower_right.x || upper_left.y > lower_right.y {
                Err(SgfParseError::InvalidPropertyValue)?;
            }
            for x in upper_left.x..lower_right.x {
                for y in upper_left.y..lower_right.y {
                    let point = Point{ x: x, y: y };
                    if points.contains(&point) {
                        Err(SgfParseError::InvalidPropertyValue)?;
                    }
                    points.insert(point);
                }
            }
        } else {
            Err(SgfParseError::InvalidPropertyValue)?
        }
    }

    Ok(points.into_iter().collect())
}

fn parse_size(values: &Vec<String>) -> Result<(u8, u8), SgfParseError> {
    if values.len() != 1 {
        Err(SgfParseError::InvalidPropertyValue)?;
    }
    let parts: Vec<&str> = values[0].split(":").collect();
    if parts.len() == 1 {
        let size = parts[0].parse().map_err(|_| SgfParseError::InvalidPropertyValue)?;
        Ok((size, size))
    } else if parts.len() == 2 {
        let width = parts[0].parse().map_err(|_| SgfParseError::InvalidPropertyValue)?;
        let height = parts[1].parse().map_err(|_| SgfParseError::InvalidPropertyValue)?;
        Ok((width, height))
    } else {
        Err(SgfParseError::InvalidPropertyValue)?
    }
}

impl std::str::FromStr for Move {
    type Err = SgfParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "" => Ok(Move::Pass),
            _ => Ok(Move::Move(s.parse()?)),
        }
    }
}

impl std::str::FromStr for Point {
    type Err = SgfParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let chars: Vec<char> = s.chars().collect();
        if chars.len() != 2 {
            return Err(SgfParseError::InvalidPropertyValue);
        }

        fn map_char(c: char) -> Result<u8, SgfParseError> {
            if c.is_ascii_lowercase() {
                Ok(c as u8 - 'a' as u8)
            } else if c.is_ascii_uppercase() {
                Ok(c as u8 - 'A' as u8)
            } else {
                Err(SgfParseError::InvalidPropertyValue)
            }
        }

        Ok(Point {
            x: map_char(chars[0])?,
            y: map_char(chars[1])?,
        })
    }
}

impl std::str::FromStr for Text {
    type Err = SgfParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Text(s.to_string()))
    }
}

impl std::str::FromStr for SimpleText {
    type Err = SgfParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(SimpleText(s.to_string()))
    }
}

impl std::str::FromStr for Double {
    type Err = SgfParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "1" {
            Ok(Double::One)
        } else if s == "2" {
            Ok(Double::Two)
        } else {
            Err(SgfParseError::InvalidPropertyValue)
        }
    }
}
