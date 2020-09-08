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
    /// # Errors
    /// If the identifier is a recognized SGF FF[4] property type and the provided values aren't
    /// correct for that property type, then an error is returned.
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
    pub fn new(identifier: String, values: Vec<String>) -> Result<Self, SgfParseError> {
        match &identifier[..] {
            "B" => Ok(Self::B(parse_single_value(&values)?)),
            "KO" => verify_empty(&values).map(|()| Ok(Self::KO))?,
            "MN" => Ok(Self::MN(parse_single_value(&values)?)),
            "W" => Ok(Self::W(parse_single_value(&values)?)),
            "AB" => Ok(Self::AB(parse_list_point(&values)?)),
            "AE" => Ok(Self::AE(parse_list_point(&values)?)),
            "AW" => Ok(Self::AW(parse_list_point(&values)?)),
            "PL" => Ok(Self::PL(parse_single_value(&values)?)),
            "C" => Ok(Self::C(parse_single_text_value(&values)?)),
            "DM" => Ok(Self::DM(parse_single_value(&values)?)),
            "GB" => Ok(Self::GB(parse_single_value(&values)?)),
            "GW" => Ok(Self::GW(parse_single_value(&values)?)),
            "HO" => Ok(Self::HO(parse_single_value(&values)?)),
            "N" => Ok(Self::N(parse_single_simple_text_value(&values)?)),
            "UC" => Ok(Self::UC(parse_single_value(&values)?)),
            "V" => Ok(Self::V(parse_single_value(&values)?)),
            "DO" => verify_empty(&values).map(|()| Ok(Self::DO))?,
            "IT" => verify_empty(&values).map(|()| Ok(Self::IT))?,
            "BM" => Ok(Self::BM(parse_single_value(&values)?)),
            "TE" => Ok(Self::TE(parse_single_value(&values)?)),
            "AR" => Ok(Self::AR(parse_list_composed_point(&values)?)),
            "CR" => Ok(Self::CR(parse_list_point(&values)?)),
            "DD" => Ok(Self::DD(parse_elist_point(&values)?)),
            "LB" => Ok(Self::LB(parse_labels(&values)?)),
            "LN" => Ok(Self::LN(parse_list_composed_point(&values)?)),
            "MA" => Ok(Self::MA(parse_list_point(&values)?)),
            "SL" => Ok(Self::SL(parse_list_point(&values)?)),
            "SQ" => Ok(Self::SQ(parse_list_point(&values)?)),
            "TR" => Ok(Self::TR(parse_list_point(&values)?)),
            "AP" => Ok(Self::AP(parse_application(&values)?)),
            "CA" => Ok(Self::CA(parse_single_simple_text_value(&values)?)),
            "FF" => {
                let value = parse_single_value(&values)?;
                if value < 0 || value > 4 {
                    return Err(SgfParseError::InvalidPropertyValue);
                }
                Ok(Self::FF(value))
            }
            "GM" => {
                let value = parse_single_value(&values)?;
                // Only Go is supported
                if value != 1 {
                    return Err(SgfParseError::InvalidPropertyValue);
                }
                Ok(Self::GM(value))
            }
            "ST" => {
                let value = parse_single_value(&values)?;
                if value < 0 || value > 3 {
                    return Err(SgfParseError::InvalidPropertyValue);
                }
                Ok(Self::ST(value))
            }
            "SZ" => Ok(Self::SZ(parse_size(&values)?)),
            "HA" => {
                let value: i64 = parse_single_value(&values)?;
                if !value >= 2 {
                    return Err(SgfParseError::InvalidPropertyValue);
                }
                Ok(Self::HA(value))
            }
            "KM" => Ok(Self::KM(parse_single_value(&values)?)),
            "AN" => Ok(Self::AN(parse_single_simple_text_value(&values)?)),
            "BR" => Ok(Self::BR(parse_single_simple_text_value(&values)?)),
            "BT" => Ok(Self::BT(parse_single_simple_text_value(&values)?)),
            "CP" => Ok(Self::CP(parse_single_simple_text_value(&values)?)),
            "DT" => Ok(Self::DT(parse_single_simple_text_value(&values)?)),
            "EV" => Ok(Self::EV(parse_single_simple_text_value(&values)?)),
            "GN" => Ok(Self::GN(parse_single_simple_text_value(&values)?)),
            "GC" => Ok(Self::GC(parse_single_text_value(&values)?)),
            "ON" => Ok(Self::ON(parse_single_simple_text_value(&values)?)),
            "OT" => Ok(Self::OT(parse_single_simple_text_value(&values)?)),
            "PB" => Ok(Self::PB(parse_single_simple_text_value(&values)?)),
            "PC" => Ok(Self::PC(parse_single_simple_text_value(&values)?)),
            "PW" => Ok(Self::PW(parse_single_simple_text_value(&values)?)),
            "RE" => Ok(Self::RE(parse_single_simple_text_value(&values)?)),
            "RO" => Ok(Self::RO(parse_single_simple_text_value(&values)?)),
            "RU" => Ok(Self::RU(parse_single_simple_text_value(&values)?)),
            "SO" => Ok(Self::SO(parse_single_simple_text_value(&values)?)),
            "TM" => Ok(Self::TM(parse_single_value(&values)?)),
            "US" => Ok(Self::US(parse_single_simple_text_value(&values)?)),
            "WR" => Ok(Self::WR(parse_single_simple_text_value(&values)?)),
            "WT" => Ok(Self::WT(parse_single_simple_text_value(&values)?)),
            "BL" => Ok(Self::BL(parse_single_value(&values)?)),
            "OB" => Ok(Self::OB(parse_single_value(&values)?)),
            "OW" => Ok(Self::OW(parse_single_value(&values)?)),
            "WL" => Ok(Self::WL(parse_single_value(&values)?)),
            "FG" => Ok(Self::FG(parse_figure(&values)?)),
            "PM" => {
                let value = parse_single_value(&values)?;
                if value < 1 || value > 2 {
                    return Err(SgfParseError::InvalidPropertyValue);
                }
                Ok(Self::PM(value))
            }
            "VW" => Ok(Self::VW(parse_elist_point(&values)?)),
            "TB" => Ok(Self::TB(parse_elist_point(&values)?)),
            "TW" => Ok(Self::TW(parse_elist_point(&values)?)),
            _ => Ok(Self::Unknown(identifier, values)),
        }
    }

    /// Returns a the identifier associated with the `SgfProp`.
    ///
    /// # Examples
    /// ```
    /// use sgf_parse::SgfProp;
    ///
    /// let prop = SgfProp::new("W".to_string(), vec!["de".to_string()]).unwrap();
    /// assert_eq!(prop.identifier(), "W");
    /// let prop = SgfProp::new("FOO".to_string(), vec!["de".to_string()]).unwrap();
    /// assert_eq!(prop.identifier(), "FOO");
    /// ```
    pub fn identifier(&self) -> String {
        match self {
            Self::B(_) => "B".to_string(),
            Self::KO => "KO".to_string(),
            Self::MN(_) => "MN".to_string(),
            Self::W(_) => "W".to_string(),
            Self::AB(_) => "AB".to_string(),
            Self::AE(_) => "AE".to_string(),
            Self::AW(_) => "AW".to_string(),
            Self::PL(_) => "PL".to_string(),
            Self::C(_) => "C".to_string(),
            Self::DM(_) => "DM".to_string(),
            Self::GB(_) => "GB".to_string(),
            Self::GW(_) => "GW".to_string(),
            Self::HO(_) => "HO".to_string(),
            Self::N(_) => "N".to_string(),
            Self::UC(_) => "UC".to_string(),
            Self::V(_) => "V".to_string(),
            Self::DO => "DO".to_string(),
            Self::IT => "IT".to_string(),
            Self::BM(_) => "BM".to_string(),
            Self::TE(_) => "TE".to_string(),
            Self::AR(_) => "AR".to_string(),
            Self::CR(_) => "CR".to_string(),
            Self::DD(_) => "DD".to_string(),
            Self::LB(_) => "LB".to_string(),
            Self::LN(_) => "LN".to_string(),
            Self::MA(_) => "MA".to_string(),
            Self::SL(_) => "SL".to_string(),
            Self::SQ(_) => "SQ".to_string(),
            Self::TR(_) => "TR".to_string(),
            Self::AP(_) => "AP".to_string(),
            Self::CA(_) => "CA".to_string(),
            Self::FF(_) => "FF".to_string(),
            Self::GM(_) => "GM".to_string(),
            Self::ST(_) => "ST".to_string(),
            Self::SZ(_) => "SZ".to_string(),
            Self::HA(_) => "HA".to_string(),
            Self::KM(_) => "KM".to_string(),
            Self::AN(_) => "AN".to_string(),
            Self::BR(_) => "BR".to_string(),
            Self::BT(_) => "BT".to_string(),
            Self::CP(_) => "CP".to_string(),
            Self::DT(_) => "DT".to_string(),
            Self::EV(_) => "EV".to_string(),
            Self::GN(_) => "GN".to_string(),
            Self::GC(_) => "GC".to_string(),
            Self::ON(_) => "ON".to_string(),
            Self::OT(_) => "OT".to_string(),
            Self::PB(_) => "PB".to_string(),
            Self::PC(_) => "PC".to_string(),
            Self::PW(_) => "PW".to_string(),
            Self::RE(_) => "RE".to_string(),
            Self::RO(_) => "RO".to_string(),
            Self::RU(_) => "RU".to_string(),
            Self::SO(_) => "SO".to_string(),
            Self::TM(_) => "TM".to_string(),
            Self::US(_) => "US".to_string(),
            Self::WR(_) => "WR".to_string(),
            Self::WT(_) => "WT".to_string(),
            Self::BL(_) => "BL".to_string(),
            Self::OB(_) => "OB".to_string(),
            Self::OW(_) => "OW".to_string(),
            Self::WL(_) => "WL".to_string(),
            Self::FG(_) => "FG".to_string(),
            Self::PM(_) => "PM".to_string(),
            Self::VW(_) => "VW".to_string(),
            Self::TB(_) => "TB".to_string(),
            Self::TW(_) => "TW".to_string(),
            Self::Unknown(identifier, _) => identifier.to_string(),
        }
    }

    /// Returns the `PropertyType` associated with the property.
    ///
    /// # Examples
    /// ```
    /// use sgf_parse::{SgfProp, PropertyType};
    ///
    /// let prop = SgfProp::new("W".to_string(), vec!["de".to_string()]).unwrap();
    /// assert_eq!(prop.property_type(), Some(PropertyType::Move));
    /// let prop = SgfProp::new("FOO".to_string(), vec!["de".to_string()]).unwrap();
    /// assert_eq!(prop.property_type(), None);
    /// ```
    pub fn property_type(&self) -> Option<PropertyType> {
        match &self {
            Self::B(_) => Some(PropertyType::Move),
            Self::KO => Some(PropertyType::Move),
            Self::MN(_) => Some(PropertyType::Move),
            Self::W(_) => Some(PropertyType::Move),
            Self::AB(_) => Some(PropertyType::Setup),
            Self::AE(_) => Some(PropertyType::Setup),
            Self::AW(_) => Some(PropertyType::Setup),
            Self::PL(_) => Some(PropertyType::Setup),
            Self::DO => Some(PropertyType::Move),
            Self::IT => Some(PropertyType::Move),
            Self::BM(_) => Some(PropertyType::Move),
            Self::TE(_) => Some(PropertyType::Move),
            Self::DD(_) => Some(PropertyType::Inherit),
            Self::AP(_) => Some(PropertyType::Root),
            Self::CA(_) => Some(PropertyType::Root),
            Self::FF(_) => Some(PropertyType::Root),
            Self::GM(_) => Some(PropertyType::Root),
            Self::ST(_) => Some(PropertyType::Root),
            Self::SZ(_) => Some(PropertyType::Root),
            Self::HA(_) => Some(PropertyType::GameInfo),
            Self::KM(_) => Some(PropertyType::GameInfo),
            Self::AN(_) => Some(PropertyType::GameInfo),
            Self::BR(_) => Some(PropertyType::GameInfo),
            Self::BT(_) => Some(PropertyType::GameInfo),
            Self::CP(_) => Some(PropertyType::GameInfo),
            Self::DT(_) => Some(PropertyType::GameInfo),
            Self::EV(_) => Some(PropertyType::GameInfo),
            Self::GN(_) => Some(PropertyType::GameInfo),
            Self::GC(_) => Some(PropertyType::GameInfo),
            Self::ON(_) => Some(PropertyType::GameInfo),
            Self::OT(_) => Some(PropertyType::GameInfo),
            Self::PB(_) => Some(PropertyType::GameInfo),
            Self::PC(_) => Some(PropertyType::GameInfo),
            Self::PW(_) => Some(PropertyType::GameInfo),
            Self::RE(_) => Some(PropertyType::GameInfo),
            Self::RO(_) => Some(PropertyType::GameInfo),
            Self::RU(_) => Some(PropertyType::GameInfo),
            Self::SO(_) => Some(PropertyType::GameInfo),
            Self::TM(_) => Some(PropertyType::GameInfo),
            Self::US(_) => Some(PropertyType::GameInfo),
            Self::WR(_) => Some(PropertyType::GameInfo),
            Self::WT(_) => Some(PropertyType::GameInfo),
            Self::BL(_) => Some(PropertyType::Move),
            Self::OB(_) => Some(PropertyType::Move),
            Self::OW(_) => Some(PropertyType::Move),
            Self::WL(_) => Some(PropertyType::Move),
            Self::PM(_) => Some(PropertyType::Inherit),
            Self::VW(_) => Some(PropertyType::Inherit),
            _ => None,
        }
    }
}

fn verify_empty(values: &[String]) -> Result<(), SgfParseError> {
    if !(values.is_empty() || (values.len() == 1 && values[0].is_empty())) {
        return Err(SgfParseError::InvalidPropertyValue);
    }
    Ok(())
}

fn parse_single_value<T: FromStr>(values: &[String]) -> Result<T, SgfParseError> {
    if values.len() != 1 {
        return Err(SgfParseError::InvalidPropertyValue);
    }
    values[0]
        .parse()
        .map_err(|_| SgfParseError::InvalidPropertyValue)
}

fn parse_single_text_value(values: &[String]) -> Result<String, SgfParseError> {
    if values.len() != 1 {
        return Err(SgfParseError::InvalidPropertyValue);
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

            // Remove soft line breaks
            if chars[i] == '\n' {
                if i + 1 < chars.len() && chars[i + 1] == '\r' {
                    i += 1;
                }
            } else if chars[i] == '\r' {
                if i + 1 < chars.len() && chars[i + 1] == '\n' {
                    i += 1;
                }
            } else {
                // Push any other literal char following '\'
                output.push(chars[i]);
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

fn parse_single_simple_text_value(values: &[String]) -> Result<String, SgfParseError> {
    if values.len() != 1 {
        return Err(SgfParseError::InvalidPropertyValue);
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

fn parse_list_point(values: &[String]) -> Result<Vec<Point>, SgfParseError> {
    let points = parse_elist_point(values)?;
    if points.is_empty() {
        return Err(SgfParseError::InvalidPropertyValue);
    }

    Ok(points)
}

fn parse_elist_point(values: &[String]) -> Result<Vec<Point>, SgfParseError> {
    let mut points = HashSet::new();
    for value in values.iter() {
        if value.contains(':') {
            let (upper_left, lower_right): (Point, Point) = parse_tuple(value)?;
            if upper_left.x > lower_right.x || upper_left.y > lower_right.y {
                return Err(SgfParseError::InvalidPropertyValue);
            }
            for x in upper_left.x..=lower_right.x {
                for y in upper_left.y..=lower_right.y {
                    let point = Point { x, y };
                    if points.contains(&point) {
                        return Err(SgfParseError::InvalidPropertyValue);
                    }
                    points.insert(point);
                }
            }
        } else {
            let point = value.parse()?;
            if points.contains(&point) {
                return Err(SgfParseError::InvalidPropertyValue);
            }
            points.insert(point);
        }
    }

    Ok(points.into_iter().collect())
}

fn parse_list_composed_point(values: &[String]) -> Result<Vec<(Point, Point)>, SgfParseError> {
    let mut pairs = HashSet::new();
    for value in values.iter() {
        let pair = parse_tuple(value)?;
        if pair.0 == pair.1 || pairs.contains(&pair) {
            return Err(SgfParseError::InvalidPropertyValue);
        }
        pairs.insert(pair);
    }

    Ok(pairs.into_iter().collect())
}

fn split_compose(value: &str) -> Result<(&str, &str), SgfParseError> {
    let parts: Vec<&str> = value.split(':').collect();
    if parts.len() != 2 {
        return Err(SgfParseError::InvalidPropertyValue);
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

fn parse_size(values: &[String]) -> Result<(u8, u8), SgfParseError> {
    if values.len() != 1 {
        return Err(SgfParseError::InvalidPropertyValue);
    }
    let value = &values[0];
    if value.contains(':') {
        parse_tuple(value)
    } else {
        let size = value
            .parse()
            .map_err(|_| SgfParseError::InvalidPropertyValue)?;
        Ok((size, size))
    }
}

fn parse_labels(values: &[String]) -> Result<Vec<(Point, String)>, SgfParseError> {
    let mut labels = vec![];
    for value in values.iter() {
        let (s1, s2) = split_compose(value)?;
        labels.push((
            s1.parse()
                .map_err(|_| SgfParseError::InvalidPropertyValue)?,
            parse_simple_text(s2),
        ));
    }
    if labels.is_empty() {
        return Err(SgfParseError::InvalidPropertyValue);
    }

    Ok(labels)
}

fn parse_figure(values: &[String]) -> Result<Option<(i64, String)>, SgfParseError> {
    if values.is_empty() || (values.len() == 1 && values[0] == "") {
        return Ok(None);
    }
    if values.len() > 1 {
        return Err(SgfParseError::InvalidPropertyValue);
    }
    let (s1, s2) = split_compose(&values[0])?;

    Ok(Some((
        s1.parse()
            .map_err(|_| SgfParseError::InvalidPropertyValue)?,
        parse_simple_text(s2),
    )))
}

fn parse_application(values: &[String]) -> Result<(String, String), SgfParseError> {
    if values.len() != 1 {
        return Err(SgfParseError::InvalidPropertyValue);
    }
    let (s1, s2) = split_compose(&values[0])?;
    Ok((parse_simple_text(s1), parse_simple_text(s2)))
}

impl FromStr for Move {
    type Err = SgfParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "" => Ok(Self::Pass),
            _ => Ok(Self::Move(s.parse()?)),
        }
    }
}

impl FromStr for Point {
    type Err = SgfParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn map_char(c: char) -> Result<u8, SgfParseError> {
            if c.is_ascii_lowercase() {
                Ok(c as u8 - b'a')
            } else if c.is_ascii_uppercase() {
                Ok(c as u8 - b'A')
            } else {
                Err(SgfParseError::InvalidPropertyValue)
            }
        }

        let chars: Vec<char> = s.chars().collect();
        if chars.len() != 2 {
            return Err(SgfParseError::InvalidPropertyValue);
        }

        Ok(Self {
            x: map_char(chars[0])?,
            y: map_char(chars[1])?,
        })
    }
}

impl FromStr for Double {
    type Err = SgfParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "1" {
            Ok(Self::One)
        } else if s == "2" {
            Ok(Self::Two)
        } else {
            Err(SgfParseError::InvalidPropertyValue)
        }
    }
}

impl FromStr for Color {
    type Err = SgfParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "B" {
            Ok(Self::Black)
        } else if s == "W" {
            Ok(Self::White)
        } else {
            Err(SgfParseError::InvalidPropertyValue)
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    #[test]
    pub fn parse_text() {
        let text = "Comment with\trandom whitespace\nescaped \\] and \\\\ and a soft \\\nlinebreak";
        let expected = "Comment with random whitespace\nescaped ] and \\ and a soft linebreak";

        assert_eq!(super::parse_text(text), expected);
    }

    #[test]
    pub fn parse_simple_text() {
        let text =
            "Comment with\trandom\r\nwhitespace\n\rescaped \\] and \\\\ and\na soft \\\nlinebreak";
        let expected = "Comment with random whitespace escaped ] and \\ and a soft linebreak";

        assert_eq!(super::parse_simple_text(text), expected);
    }

    #[test]
    pub fn parse_list_point() {
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
        .map(|(x, y)| super::Point { x, y })
        .collect();

        let result: HashSet<_> = super::parse_list_point(&values)
            .unwrap()
            .into_iter()
            .collect();

        assert_eq!(result, expected);
    }
}
