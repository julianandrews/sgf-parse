use std::collections::HashSet;
use std::str::FromStr;

use super::SgfParseError;

/// An SGF [Double](https://www.red-bean.com/sgf/sgf4.html#double) value.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Double {
    One,
    Two,
}

/// An SGF [Color](https://www.red-bean.com/sgf/sgf4.html#types) value.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Color {
    Black,
    White,
}

/// An SGF [Point](https://www.red-bean.com/sgf/go.html#types) value for the Game of Go.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Point {
    pub x: u8,
    pub y: u8,
}

/// An SGF [Move](https://www.red-bean.com/sgf/go.html#types) value for the Game of Go.
///
/// Moves may either be a pass, or a [Point](struct.Point.html)
///
/// # Examples
/// ```
/// use sgf_parse::{parse, SgfProp, Move};
///
/// let node = parse("(;B[de])").unwrap().into_iter().next().unwrap();
/// for prop in node.properties() {
///     match prop {
///         SgfProp::B(Move::Move(point)) => println!("B move at {:?}", point),
///         _ => {}
///     }
/// }
/// ```
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Move {
    Pass,
    Move(Point),
}

/// An SGF [Property Type](https://www.red-bean.com/sgf/sgf4.html#2.2.1).
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PropertyType {
    Move,
    Setup,
    Root,
    GameInfo,
    Inherit,
}

/// An SGF Property with identifier and value.
///
/// All [general properties](https://www.red-bean.com/sgf/properties.html) from the SGF
/// specification and all [go specific properties](https://www.red-bean.com/sgf/go.html) will
/// return the approprite enum instance with parsed data. Unrecognized properties, or properties
/// from other games will return `Unknown(identifier, values)`.
///
/// See [Property Value Types](https://www.red-bean.com/sgf/sgf4.html#types) for a list of types
/// recognized by SGF. For parsing purposes the following mappings are used:
/// * 'Number' => [i64](https://doc.rust-lang.org/std/primitive.i64.html)
/// * 'Real' => [f64](https://doc.rust-lang.org/std/primitive.f64.html)
/// * 'Double' => [Double](enum.Double.html)
/// * 'Color' => [Color](enum.Color.html)
/// * 'SimpleText' => [String](https://doc.rust-lang.org/std/string/struct.String.html)
///     (formatted and escaped as [here](https://www.red-bean.com/sgf/sgf4.html#text))
/// * 'Text' => [String](https://doc.rust-lang.org/std/string/struct.String.html)
///     (formatted and escaped as [here](https://www.red-bean.com/sgf/sgf4.html#simpletext))
/// * 'Point' => [Point](struct.Point.html)
/// * 'Stone' => [Point](struct.Point.html)
/// * 'Move' => [Move](enum.Move.html)
/// * 'List' => [Vec](https://doc.rust-lang.org/std/vec/struct.Vec.html)
/// * 'Compose' => a [tuple](https://doc.rust-lang.org/std/primitive.tuple.html) of the composed values
#[derive(Clone, Debug)]
pub enum SgfProp {
    // Move Properties
    B(Move),
    KO,
    MN(i64),
    W(Move),
    // Setup Properties
    AB(Vec<Point>),
    AE(Vec<Point>),
    AW(Vec<Point>),
    PL(Color),
    // Node Annotation properties
    C(String),
    DM(Double),
    GB(Double),
    GW(Double),
    HO(Double),
    N(String),
    UC(Double),
    V(f64),
    // Move annotation properties
    BM(Double),
    DO,
    IT,
    TE(Double),
    // Markup Properties
    AR(Vec<(Point, Point)>),
    CR(Vec<Point>),
    DD(Vec<Point>),
    LB(Vec<(Point, String)>),
    LN(Vec<(Point, Point)>),
    MA(Vec<Point>),
    SL(Vec<Point>),
    SQ(Vec<Point>),
    TR(Vec<Point>),
    // Root Properties
    AP((String, String)),
    CA(String),
    FF(i64),
    GM(i64),
    ST(i64),
    SZ((u8, u8)),
    // Game info properties
    HA(i64),
    KM(f64),
    AN(String),
    BR(String),
    BT(String),
    CP(String),
    DT(String),
    EV(String),
    GN(String),
    GC(String),
    ON(String),
    OT(String),
    PB(String),
    PC(String),
    PW(String),
    RE(String),
    RO(String),
    RU(String),
    SO(String),
    TM(f64),
    US(String),
    WR(String),
    WT(String),
    // Timing Properties
    BL(f64),
    OB(i64),
    OW(i64),
    WL(f64),
    // Miscellaneous properties
    FG(Option<(i64, String)>),
    PM(i64),
    VW(Vec<Point>),
    TB(Vec<Point>),
    TW(Vec<Point>),
    Unknown(String, Vec<String>),
}

impl SgfProp {
    /// Returns a new property parsed from the provided identifier and values
    ///
    /// # Examples
    /// ```
    /// use sgf_parse::SgfProp;
    ///
    /// // SgfProp::B(Point{ x: 2, y: 3 }
    /// let prop = SgfProp::new("B".to_string(), vec!["cd".to_string()]);
    /// // SgfProp::AB(vec![Point{ x: 2, y: 3 }, Point { x: 3, y: 3 }])
    /// let prop = SgfProp::new("AB".to_string(), vec!["cd".to_string(), "dd".to_string()]);
    /// // SgfProp::Unknown("FOO", vec!["Text"])
    /// let prop = SgfProp::new("FOO".to_string(), vec!["Text".to_string()]);
    /// ```
    pub fn new(identifier: String, values: Vec<String>) -> Result<SgfProp, SgfParseError> {
        match &identifier[..] {
            "B" => Ok(SgfProp::B(parse_single_value(&values)?)),
            "KO" => verify_empty(&values).map(|()| Ok(SgfProp::KO))?,
            "MN" => Ok(SgfProp::MN(parse_single_value(&values)?)),
            "W" => Ok(SgfProp::W(parse_single_value(&values)?)),
            "AB" => Ok(SgfProp::AB(parse_list_point(&values)?)),
            "AE" => Ok(SgfProp::AE(parse_list_point(&values)?)),
            "AW" => Ok(SgfProp::AW(parse_list_point(&values)?)),
            "PL" => Ok(SgfProp::PL(parse_single_value(&values)?)),
            "C" => Ok(SgfProp::C(parse_single_text_value(&values)?)),
            "DM" => Ok(SgfProp::DM(parse_single_value(&values)?)),
            "GB" => Ok(SgfProp::GB(parse_single_value(&values)?)),
            "GW" => Ok(SgfProp::GW(parse_single_value(&values)?)),
            "HO" => Ok(SgfProp::HO(parse_single_value(&values)?)),
            "N" => Ok(SgfProp::N(parse_single_simple_text_value(&values)?)),
            "UC" => Ok(SgfProp::UC(parse_single_value(&values)?)),
            "V" => Ok(SgfProp::V(parse_single_value(&values)?)),
            "DO" => verify_empty(&values).map(|()| Ok(SgfProp::DO))?,
            "IT" => verify_empty(&values).map(|()| Ok(SgfProp::IT))?,
            "BM" => Ok(SgfProp::BM(parse_single_value(&values)?)),
            "TE" => Ok(SgfProp::TE(parse_single_value(&values)?)),
            "AR" => Ok(SgfProp::AR(parse_list_composed_point(&values)?)),
            "CR" => Ok(SgfProp::CR(parse_list_point(&values)?)),
            "DD" => Ok(SgfProp::DD(parse_elist_point(&values)?)),
            "LB" => Ok(SgfProp::LB(parse_labels(&values)?)),
            "LN" => Ok(SgfProp::LN(parse_list_composed_point(&values)?)),
            "MA" => Ok(SgfProp::MA(parse_list_point(&values)?)),
            "SL" => Ok(SgfProp::SL(parse_list_point(&values)?)),
            "SQ" => Ok(SgfProp::SQ(parse_list_point(&values)?)),
            "TR" => Ok(SgfProp::TR(parse_list_point(&values)?)),
            "AP" => Ok(SgfProp::AP(parse_application(&values)?)),
            "CA" => Ok(SgfProp::CA(parse_single_simple_text_value(&values)?)),
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
            "AN" => Ok(SgfProp::AN(parse_single_simple_text_value(&values)?)),
            "BR" => Ok(SgfProp::BR(parse_single_simple_text_value(&values)?)),
            "BT" => Ok(SgfProp::BT(parse_single_simple_text_value(&values)?)),
            "CP" => Ok(SgfProp::CP(parse_single_simple_text_value(&values)?)),
            "DT" => Ok(SgfProp::DT(parse_single_simple_text_value(&values)?)),
            "EV" => Ok(SgfProp::EV(parse_single_simple_text_value(&values)?)),
            "GN" => Ok(SgfProp::GN(parse_single_simple_text_value(&values)?)),
            "GC" => Ok(SgfProp::GC(parse_single_text_value(&values)?)),
            "ON" => Ok(SgfProp::ON(parse_single_simple_text_value(&values)?)),
            "OT" => Ok(SgfProp::OT(parse_single_simple_text_value(&values)?)),
            "PB" => Ok(SgfProp::PB(parse_single_simple_text_value(&values)?)),
            "PC" => Ok(SgfProp::PC(parse_single_simple_text_value(&values)?)),
            "PW" => Ok(SgfProp::PW(parse_single_simple_text_value(&values)?)),
            "RE" => Ok(SgfProp::RE(parse_single_simple_text_value(&values)?)),
            "RO" => Ok(SgfProp::RO(parse_single_simple_text_value(&values)?)),
            "RU" => Ok(SgfProp::RU(parse_single_simple_text_value(&values)?)),
            "SO" => Ok(SgfProp::SO(parse_single_simple_text_value(&values)?)),
            "TM" => Ok(SgfProp::TM(parse_single_value(&values)?)),
            "US" => Ok(SgfProp::US(parse_single_simple_text_value(&values)?)),
            "WR" => Ok(SgfProp::WR(parse_single_simple_text_value(&values)?)),
            "WT" => Ok(SgfProp::WT(parse_single_simple_text_value(&values)?)),
            "BL" => Ok(SgfProp::BL(parse_single_value(&values)?)),
            "OB" => Ok(SgfProp::OB(parse_single_value(&values)?)),
            "OW" => Ok(SgfProp::OW(parse_single_value(&values)?)),
            "WL" => Ok(SgfProp::WL(parse_single_value(&values)?)),
            "FG" => Ok(SgfProp::FG(parse_figure(&values)?)),
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
            _ => Ok(SgfProp::Unknown(identifier, values)),
        }
    }

    /// Returns a the identifier associated with the SgfProp.
    ///
    /// # Examples
    /// ```
    /// use sgf_parse::SgfProp;
    ///
    /// // Prints "W"
    /// let prop = SgfProp::new("W".to_string(), vec!["de".to_string()]).unwrap();
    /// println!("Identifier: {}", prop.identifier());
    /// // Prints "FOO"
    /// let prop = SgfProp::new("FOO".to_string(), vec!["de".to_string()]).unwrap();
    /// println!("Identifier: {}", prop.identifier());
    /// ```
    pub fn identifier(&self) -> String {
        match self {
            SgfProp::B(_) => "B".to_string(),
            SgfProp::KO => "KO".to_string(),
            SgfProp::MN(_) => "MN".to_string(),
            SgfProp::W(_) => "W".to_string(),
            SgfProp::AB(_) => "AB".to_string(),
            SgfProp::AE(_) => "AE".to_string(),
            SgfProp::AW(_) => "AW".to_string(),
            SgfProp::PL(_) => "PL".to_string(),
            SgfProp::C(_) => "C".to_string(),
            SgfProp::DM(_) => "DM".to_string(),
            SgfProp::GB(_) => "GB".to_string(),
            SgfProp::GW(_) => "GW".to_string(),
            SgfProp::HO(_) => "HO".to_string(),
            SgfProp::N(_) => "N".to_string(),
            SgfProp::UC(_) => "UC".to_string(),
            SgfProp::V(_) => "V".to_string(),
            SgfProp::DO => "DO".to_string(),
            SgfProp::IT => "IT".to_string(),
            SgfProp::BM(_) => "BM".to_string(),
            SgfProp::TE(_) => "TE".to_string(),
            SgfProp::AR(_) => "AR".to_string(),
            SgfProp::CR(_) => "CR".to_string(),
            SgfProp::DD(_) => "DD".to_string(),
            SgfProp::LB(_) => "LB".to_string(),
            SgfProp::LN(_) => "LN".to_string(),
            SgfProp::MA(_) => "MA".to_string(),
            SgfProp::SL(_) => "SL".to_string(),
            SgfProp::SQ(_) => "SQ".to_string(),
            SgfProp::TR(_) => "TR".to_string(),
            SgfProp::AP(_) => "AP".to_string(),
            SgfProp::CA(_) => "CA".to_string(),
            SgfProp::FF(_) => "FF".to_string(),
            SgfProp::GM(_) => "GM".to_string(),
            SgfProp::ST(_) => "ST".to_string(),
            SgfProp::SZ(_) => "SZ".to_string(),
            SgfProp::HA(_) => "HA".to_string(),
            SgfProp::KM(_) => "KM".to_string(),
            SgfProp::AN(_) => "AN".to_string(),
            SgfProp::BR(_) => "BR".to_string(),
            SgfProp::BT(_) => "BT".to_string(),
            SgfProp::CP(_) => "CP".to_string(),
            SgfProp::DT(_) => "DT".to_string(),
            SgfProp::EV(_) => "EV".to_string(),
            SgfProp::GN(_) => "GN".to_string(),
            SgfProp::GC(_) => "GC".to_string(),
            SgfProp::ON(_) => "ON".to_string(),
            SgfProp::OT(_) => "OT".to_string(),
            SgfProp::PB(_) => "PB".to_string(),
            SgfProp::PC(_) => "PC".to_string(),
            SgfProp::PW(_) => "PW".to_string(),
            SgfProp::RE(_) => "RE".to_string(),
            SgfProp::RO(_) => "RO".to_string(),
            SgfProp::RU(_) => "RU".to_string(),
            SgfProp::SO(_) => "SO".to_string(),
            SgfProp::TM(_) => "TM".to_string(),
            SgfProp::US(_) => "US".to_string(),
            SgfProp::WR(_) => "WR".to_string(),
            SgfProp::WT(_) => "WT".to_string(),
            SgfProp::BL(_) => "BL".to_string(),
            SgfProp::OB(_) => "OB".to_string(),
            SgfProp::OW(_) => "OW".to_string(),
            SgfProp::WL(_) => "WL".to_string(),
            SgfProp::FG(_) => "FG".to_string(),
            SgfProp::PM(_) => "PM".to_string(),
            SgfProp::VW(_) => "VW".to_string(),
            SgfProp::TB(_) => "TB".to_string(),
            SgfProp::TW(_) => "TW".to_string(),
            SgfProp::Unknown(identifier, _) => identifier.to_string(),
        }
    }

    /// Returns the [PropertyType](enum.PropertyType.html) associated with the property.
    ///
    /// # Examples
    /// ```
    /// use sgf_parse::SgfProp;
    ///
    /// // Prints "W"
    /// let prop = SgfProp::new("W".to_string(), vec!["de".to_string()]).unwrap();
    /// println!("Identifier: {}", prop.identifier());
    /// // Prints "FOO"
    /// let prop = SgfProp::new("FOO".to_string(), vec!["de".to_string()]).unwrap();
    /// println!("Identifier: {}", prop.identifier());
    /// ```
    pub fn property_type(&self) -> Option<PropertyType> {
        match &self {
            SgfProp::B(_) => Some(PropertyType::Move),
            SgfProp::KO => Some(PropertyType::Move),
            SgfProp::MN(_) => Some(PropertyType::Move),
            SgfProp::W(_) => Some(PropertyType::Move),
            SgfProp::AB(_) => Some(PropertyType::Setup),
            SgfProp::AE(_) => Some(PropertyType::Setup),
            SgfProp::AW(_) => Some(PropertyType::Setup),
            SgfProp::PL(_) => Some(PropertyType::Setup),
            SgfProp::DO => Some(PropertyType::Move),
            SgfProp::IT => Some(PropertyType::Move),
            SgfProp::BM(_) => Some(PropertyType::Move),
            SgfProp::TE(_) => Some(PropertyType::Move),
            SgfProp::DD(_) => Some(PropertyType::Inherit),
            SgfProp::AP(_) => Some(PropertyType::Root),
            SgfProp::CA(_) => Some(PropertyType::Root),
            SgfProp::FF(_) => Some(PropertyType::Root),
            SgfProp::GM(_) => Some(PropertyType::Root),
            SgfProp::ST(_) => Some(PropertyType::Root),
            SgfProp::SZ(_) => Some(PropertyType::Root),
            SgfProp::HA(_) => Some(PropertyType::GameInfo),
            SgfProp::KM(_) => Some(PropertyType::GameInfo),
            SgfProp::AN(_) => Some(PropertyType::GameInfo),
            SgfProp::BR(_) => Some(PropertyType::GameInfo),
            SgfProp::BT(_) => Some(PropertyType::GameInfo),
            SgfProp::CP(_) => Some(PropertyType::GameInfo),
            SgfProp::DT(_) => Some(PropertyType::GameInfo),
            SgfProp::EV(_) => Some(PropertyType::GameInfo),
            SgfProp::GN(_) => Some(PropertyType::GameInfo),
            SgfProp::GC(_) => Some(PropertyType::GameInfo),
            SgfProp::ON(_) => Some(PropertyType::GameInfo),
            SgfProp::OT(_) => Some(PropertyType::GameInfo),
            SgfProp::PB(_) => Some(PropertyType::GameInfo),
            SgfProp::PC(_) => Some(PropertyType::GameInfo),
            SgfProp::PW(_) => Some(PropertyType::GameInfo),
            SgfProp::RE(_) => Some(PropertyType::GameInfo),
            SgfProp::RO(_) => Some(PropertyType::GameInfo),
            SgfProp::RU(_) => Some(PropertyType::GameInfo),
            SgfProp::SO(_) => Some(PropertyType::GameInfo),
            SgfProp::TM(_) => Some(PropertyType::GameInfo),
            SgfProp::US(_) => Some(PropertyType::GameInfo),
            SgfProp::WR(_) => Some(PropertyType::GameInfo),
            SgfProp::WT(_) => Some(PropertyType::GameInfo),
            SgfProp::BL(_) => Some(PropertyType::Move),
            SgfProp::OB(_) => Some(PropertyType::Move),
            SgfProp::OW(_) => Some(PropertyType::Move),
            SgfProp::WL(_) => Some(PropertyType::Move),
            SgfProp::PM(_) => Some(PropertyType::Inherit),
            SgfProp::VW(_) => Some(PropertyType::Inherit),
            _ => None,
        }
    }
}

fn verify_empty(values: &Vec<String>) -> Result<(), SgfParseError> {
    if !(values.len() == 0 || (values.len() == 1 && values[0].is_empty())) {
        Err(SgfParseError::InvalidPropertyValue)?;
    }
    Ok(())
}

fn parse_single_value<T: FromStr>(values: &Vec<String>) -> Result<T, SgfParseError> {
    if values.len() != 1 {
        Err(SgfParseError::InvalidPropertyValue)?;
    }
    values[0]
        .parse()
        .map_err(|_| SgfParseError::InvalidPropertyValue)
}

fn parse_single_text_value(values: &Vec<String>) -> Result<String, SgfParseError> {
    if values.len() != 1 {
        Err(SgfParseError::InvalidPropertyValue)?;
    }
    Ok(parse_text(&values[0]))
}

fn parse_text(s: &str) -> String {
    // See https://www.red-bean.com/sgf/sgf4.html#text
    let mut output = vec![];
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '\\' && i + 1 < chars.len() {
            i += 1;
            let c = chars[i];

            // Remove soft line breaks
            if c == '\n' {
                if i + 1 < chars.len() && chars[i + 1] == '\r' {
                    i += 1;
                }
            } else if c == '\r' {
                if i + 1 < chars.len() && chars[i + 1] == '\n' {
                    i += 1;
                }
            } else {
                // Push any other literal char following '\'
                output.push(c);
            }
        } else if c.is_whitespace() && c != '\r' && c != '\n' {
            if i + 1 < chars.len() {
                let next = chars[i + 1];
                // Treat \r\n or \n\r as a single linebreak
                if (c == '\n' && next == '\r') || (c == '\r' && next == '\n') {
                    i += 1;
                }
            }
            // Replace whitespace with ' '
            output.push(' ');
        } else {
            output.push(c);
        }
        i += 1;
    }

    output.into_iter().collect()
}

fn parse_single_simple_text_value(values: &Vec<String>) -> Result<String, SgfParseError> {
    if values.len() != 1 {
        Err(SgfParseError::InvalidPropertyValue)?;
    }
    Ok(parse_simple_text(&values[0]))
}

fn parse_simple_text(s: &str) -> String {
    parse_text(s)
        .replace("\r\n", " ")
        .replace("\n\r", " ")
        .replace("\n", " ")
        .replace("\r", " ")
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
        if value.contains(":") {
            let (upper_left, lower_right): (Point, Point) = parse_tuple(&value)?;
            if upper_left.x > lower_right.x || upper_left.y > lower_right.y {
                Err(SgfParseError::InvalidPropertyValue)?;
            }
            for x in upper_left.x..=lower_right.x {
                for y in upper_left.y..=lower_right.y {
                    let point = Point { x: x, y: y };
                    if points.contains(&point) {
                        Err(SgfParseError::InvalidPropertyValue)?;
                    }
                    points.insert(point);
                }
            }
        } else {
            let point = value.parse()?;
            if points.contains(&point) {
                Err(SgfParseError::InvalidPropertyValue)?;
            }
            points.insert(point);
        }
    }

    Ok(points.into_iter().collect())
}

fn parse_list_composed_point(values: &Vec<String>) -> Result<Vec<(Point, Point)>, SgfParseError> {
    let mut pairs = HashSet::new();
    for value in values.iter() {
        let pair = parse_tuple(value)?;
        if pair.0 == pair.1 || pairs.contains(&pair) {
            Err(SgfParseError::InvalidPropertyValue)?;
        }
        pairs.insert(pair);
    }

    Ok(pairs.into_iter().collect())
}

fn split_compose(value: &str) -> Result<(&str, &str), SgfParseError> {
    let parts: Vec<&str> = value.split(":").collect();
    if parts.len() != 2 {
        Err(SgfParseError::InvalidPropertyValue)?;
    }

    Ok((parts[0], parts[1]))
}

fn parse_tuple<T1: FromStr, T2: FromStr>(value: &str) -> Result<(T1, T2), SgfParseError> {
    let (s1, s2) = split_compose(value)?;
    Ok((
        s1.parse()
            .map_err(|_| SgfParseError::InvalidPropertyValue)?,
        s2.parse()
            .map_err(|_| SgfParseError::InvalidPropertyValue)?,
    ))
}

fn parse_size(values: &Vec<String>) -> Result<(u8, u8), SgfParseError> {
    if values.len() != 1 {
        Err(SgfParseError::InvalidPropertyValue)?;
    }
    let value = &values[0];
    if value.contains(":") {
        parse_tuple(value)
    } else {
        let size = value
            .parse()
            .map_err(|_| SgfParseError::InvalidPropertyValue)?;
        Ok((size, size))
    }
}

fn parse_labels(values: &Vec<String>) -> Result<Vec<(Point, String)>, SgfParseError> {
    let mut labels = vec![];
    for value in values.iter() {
        let (s1, s2) = split_compose(&value)?;
        labels.push((
            s1.parse()
                .map_err(|_| SgfParseError::InvalidPropertyValue)?,
            parse_simple_text(s2),
        ));
    }
    if labels.len() == 0 {
        Err(SgfParseError::InvalidPropertyValue)?;
    }

    Ok(labels)
}

fn parse_figure(values: &Vec<String>) -> Result<Option<(i64, String)>, SgfParseError> {
    if values.len() == 0 || (values.len() == 1 && values[0] == "") {
        return Ok(None);
    }
    if values.len() > 1 {
        Err(SgfParseError::InvalidPropertyValue)?;
    }
    let (s1, s2) = split_compose(&values[0])?;

    Ok(Some((
        s1.parse()
            .map_err(|_| SgfParseError::InvalidPropertyValue)?,
        parse_simple_text(s2),
    )))
}

fn parse_application(values: &Vec<String>) -> Result<(String, String), SgfParseError> {
    if values.len() != 1 {
        Err(SgfParseError::InvalidPropertyValue)?;
    }
    let (s1, s2) = split_compose(&values[0])?;
    Ok((parse_simple_text(s1), parse_simple_text(s2)))
}

impl FromStr for Move {
    type Err = SgfParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "" => Ok(Move::Pass),
            _ => Ok(Move::Move(s.parse()?)),
        }
    }
}

impl FromStr for Point {
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

impl FromStr for Double {
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

impl FromStr for Color {
    type Err = SgfParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "B" {
            Ok(Color::Black)
        } else if s == "W" {
            Ok(Color::White)
        } else {
            Err(SgfParseError::InvalidPropertyValue)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashSet;

    #[test]
    pub fn test_parse_text() {
        let text = "Comment with\trandom whitespace\nescaped \\] and \\\\ and a soft \\\nlinebreak";
        let expected = "Comment with random whitespace\nescaped ] and \\ and a soft linebreak";

        assert_eq!(parse_text(&text), expected);
    }

    #[test]
    pub fn test_parse_simple_text() {
        let text =
            "Comment with\trandom\r\nwhitespace\n\rescaped \\] and \\\\ and\na soft \\\nlinebreak";
        let expected = "Comment with random whitespace escaped ] and \\ and a soft linebreak";

        assert_eq!(parse_simple_text(&text), expected);
    }

    #[test]
    pub fn test_parse_list_point() {
        let values = vec!["pq:ss".to_string(), "so".to_string(), "lr:ns".to_string()];
        let expected: HashSet<_> = vec![
            (15, 16),
            (16, 16),
            (17, 16),
            (18, 16),
            (15, 17),
            (16, 17),
            (17, 17),
            (18, 17),
            (15, 18),
            (16, 18),
            (17, 18),
            (18, 18),
            (18, 14),
            (11, 17),
            (12, 17),
            (13, 17),
            (11, 18),
            (12, 18),
            (13, 18),
        ]
        .into_iter()
        .map(|(x, y)| Point { x: x, y: y })
        .collect();

        let result: HashSet<_> = parse_list_point(&values).unwrap().into_iter().collect();

        assert_eq!(result, expected);
    }
}
