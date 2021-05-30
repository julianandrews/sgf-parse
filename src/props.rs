pub mod utils;
mod values;

use std::collections::HashSet;
use std::str::FromStr;

use crate::errors::SgfPropError;
use crate::traits::Game;
use utils::{
    parse_elist_point, parse_list_point, parse_list_stone, parse_single_simple_text_value,
    parse_tuple, split_compose,
};

pub use values::{Color, Double, SimpleText, Text};

// TODO: Handle game specific properties differently

/// An SGF [property type](https://www.red-bean.com/sgf/sgf4.html#2.2.1).
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
/// specification and all properties will return the approprite enum instance with parsed data.
/// Unrecognized properties will return `SgfProp::Unknown(identifier, values)`. Recognized
/// general or game specific properties with invalid values will return
/// `SgfProp::Invalid(identifier, values)`.
///
/// See [property value types](https://www.red-bean.com/sgf/sgf4.html#types) for a list of types
/// recognized by SGF. For parsing purposes the following mappings are used:
/// * 'Number' => [i64](https://doc.rust-lang.org/std/primitive.i64.html)
/// * 'Real' => [f64](https://doc.rust-lang.org/std/primitive.f64.html)
/// * 'Double' => [Double](enum.Double.html)
/// * 'Color' => [Color](enum.Color.html)
/// * 'SimpleText' => [SimpleText](struct.SimpleText.html)
/// * 'Text' => [Text](struct.Text.html)
/// * 'Point' => Game specific Point value (e.g.: [GoPoint](game/struct.GoPoint.html))
/// * 'Stone' => Game specific Stone value (e.g.: [GoPoint](game/struct.GoPoint.html))
/// * 'Move' => Game specific Move value (e.g.: [GoPoint](game/struct.GoMove.html))
/// * 'List' => [HashSet](https://doc.rust-lang.org/std/collections/struct.HashSet.html)
/// * 'Compose' => [tuple](https://doc.rust-lang.org/std/primitive.tuple.html) of the composed values
#[derive(Clone, Debug, PartialEq)]
pub enum SgfProp<G: Game> {
    // Move Properties
    B(G::Move),
    KO,
    MN(i64),
    W(G::Move),
    // Setup Properties
    AB(HashSet<G::Stone>),
    AE(HashSet<G::Point>),
    AW(HashSet<G::Stone>),
    PL(Color),
    // Node Annotation properties
    C(Text),
    DM(Double),
    GB(Double),
    GW(Double),
    HO(Double),
    N(SimpleText),
    UC(Double),
    V(f64),
    // Move annotation properties
    BM(Double),
    DO,
    IT,
    TE(Double),
    // Markup Properties
    AR(HashSet<(G::Point, G::Point)>),
    CR(HashSet<G::Point>),
    DD(HashSet<G::Point>),
    LB(HashSet<(G::Point, SimpleText)>),
    LN(HashSet<(G::Point, G::Point)>),
    MA(HashSet<G::Point>),
    SL(HashSet<G::Point>),
    SQ(HashSet<G::Point>),
    TR(HashSet<G::Point>),
    // Root Properties
    AP((SimpleText, SimpleText)),
    CA(SimpleText),
    FF(i64),
    GM(i64),
    ST(i64),
    SZ((u8, u8)),
    // Game info properties
    HA(i64),
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
    FG(Option<(i64, SimpleText)>),
    PM(i64),
    VW(HashSet<G::Point>),
    TB(HashSet<G::Point>),
    TW(HashSet<G::Point>),
    Unknown(String, Vec<String>),
    Invalid(String, Vec<String>),
}

impl<G: Game> SgfProp<G> {
    /// Returns a new property parsed from the provided identifier and values
    ///
    /// Unrecognized properties will get mapped to `SgfProp::Unknown`. Invalid
    /// values for recognized properties will get mapped to `SgfProp::Invalid`.
    ///
    /// # Examples
    /// ```
    /// use sgf_parse::SgfProp;
    /// use sgf_parse::game::GoGame;
    ///
    /// // SgfProp::B(Point{ x: 2, y: 3 }
    /// let prop = SgfProp::<GoGame>::new("B".to_string(), vec!["cd".to_string()]);
    /// // SgfProp::AB(vec![Point{ x: 2, y: 3 }, Point { x: 3, y: 3 }])
    /// let prop = SgfProp::<GoGame>::new("AB".to_string(), vec!["cd".to_string(), "dd".to_string()]);
    /// // SgfProp::Unknown("FOO", vec!["Text"])
    /// let prop = SgfProp::<GoGame>::new("FOO".to_string(), vec!["Text".to_string()]);
    /// ```
    pub fn new(identifier: String, values: Vec<String>) -> Self {
        let result = match &identifier[..] {
            "B" => parse_single_value(&values).map(Self::B),
            "KO" => verify_empty(&values).map(|()| Self::KO),
            "MN" => parse_single_value(&values).map(Self::MN),
            "W" => parse_single_value(&values).map(Self::W),
            "AB" => parse_list_stone::<G>(&values).map(Self::AB),
            "AE" => parse_list_point::<G>(&values).map(Self::AE),
            "AW" => parse_list_stone::<G>(&values).map(Self::AW),
            "PL" => parse_single_value(&values).map(Self::PL),
            "C" => parse_single_text_value(&values).map(Self::C),
            "DM" => parse_single_value(&values).map(Self::DM),
            "GB" => parse_single_value(&values).map(Self::GB),
            "GW" => parse_single_value(&values).map(Self::GW),
            "HO" => parse_single_value(&values).map(Self::HO),
            "N" => parse_single_simple_text_value(&values).map(Self::N),
            "UC" => parse_single_value(&values).map(Self::UC),
            "V" => parse_single_value(&values).map(Self::V),
            "DO" => verify_empty(&values).map(|()| Self::DO),
            "IT" => verify_empty(&values).map(|()| Self::IT),
            "BM" => parse_single_value(&values).map(Self::BM),
            "TE" => parse_single_value(&values).map(Self::TE),
            "AR" => parse_list_composed_point::<G>(&values).map(Self::AR),
            "CR" => parse_list_point::<G>(&values).map(Self::CR),
            "DD" => parse_elist_point::<G>(&values).map(Self::DD),
            "LB" => parse_labels::<G>(&values).map(Self::LB),
            "LN" => parse_list_composed_point::<G>(&values).map(Self::LN),
            "MA" => parse_list_point::<G>(&values).map(Self::MA),
            "SL" => parse_list_point::<G>(&values).map(Self::SL),
            "SQ" => parse_list_point::<G>(&values).map(Self::SQ),
            "TR" => parse_list_point::<G>(&values).map(Self::TR),
            "AP" => parse_application(&values).map(Self::AP),
            "CA" => parse_single_simple_text_value(&values).map(Self::CA),
            "FF" => match parse_single_value(&values) {
                Ok(value) => {
                    if !(0..=4).contains(&value) {
                        Err(SgfPropError {})
                    } else {
                        Ok(Self::FF(value))
                    }
                }
                _ => Err(SgfPropError {}),
            },
            "GM" => parse_single_value(&values).map(Self::GM),
            "ST" => match parse_single_value(&values) {
                Ok(value) => {
                    if !(0..=3).contains(&value) {
                        Err(SgfPropError {})
                    } else {
                        Ok(Self::ST(value))
                    }
                }
                _ => Err(SgfPropError {}),
            },
            "SZ" => parse_size(&values).map(Self::SZ),
            "HA" => match parse_single_value(&values) {
                Ok(value) => {
                    if value < 2 {
                        Err(SgfPropError {})
                    } else {
                        Ok(Self::HA(value))
                    }
                }
                _ => Err(SgfPropError {}),
            },
            "KM" => parse_single_value(&values).map(Self::KM),
            "AN" => parse_single_simple_text_value(&values).map(Self::AN),
            "BR" => parse_single_simple_text_value(&values).map(Self::BR),
            "BT" => parse_single_simple_text_value(&values).map(Self::BT),
            "CP" => parse_single_simple_text_value(&values).map(Self::CP),
            "DT" => parse_single_simple_text_value(&values).map(Self::DT),
            "EV" => parse_single_simple_text_value(&values).map(Self::EV),
            "GN" => parse_single_simple_text_value(&values).map(Self::GN),
            "GC" => parse_single_text_value(&values).map(Self::GC),
            "ON" => parse_single_simple_text_value(&values).map(Self::ON),
            "OT" => parse_single_simple_text_value(&values).map(Self::OT),
            "PB" => parse_single_simple_text_value(&values).map(Self::PB),
            "PC" => parse_single_simple_text_value(&values).map(Self::PC),
            "PW" => parse_single_simple_text_value(&values).map(Self::PW),
            "RE" => parse_single_simple_text_value(&values).map(Self::RE),
            "RO" => parse_single_simple_text_value(&values).map(Self::RO),
            "RU" => parse_single_simple_text_value(&values).map(Self::RU),
            "SO" => parse_single_simple_text_value(&values).map(Self::SO),
            "TM" => parse_single_value(&values).map(Self::TM),
            "US" => parse_single_simple_text_value(&values).map(Self::US),
            "WR" => parse_single_simple_text_value(&values).map(Self::WR),
            "WT" => parse_single_simple_text_value(&values).map(Self::WT),
            "BL" => parse_single_value(&values).map(Self::BL),
            "OB" => parse_single_value(&values).map(Self::OB),
            "OW" => parse_single_value(&values).map(Self::OW),
            "WL" => parse_single_value(&values).map(Self::WL),
            "FG" => parse_figure(&values).map(Self::FG),
            "PM" => match parse_single_value(&values) {
                Ok(value) => {
                    if !(1..=2).contains(&value) {
                        Err(SgfPropError {})
                    } else {
                        Ok(Self::PM(value))
                    }
                }
                _ => Err(SgfPropError {}),
            },
            "VW" => parse_elist_point::<G>(&values).map(Self::VW),
            "TB" => parse_elist_point::<G>(&values).map(Self::TB),
            "TW" => parse_elist_point::<G>(&values).map(Self::TW),
            _ => return Self::Unknown(identifier, values),
        };
        result.unwrap_or(Self::Invalid(identifier, values))
    }

    /// Returns a the identifier associated with the `SgfProp`.
    ///
    /// # Examples
    /// ```
    /// use sgf_parse::SgfProp;
    /// use sgf_parse::game::GoGame;
    ///
    /// let prop = SgfProp::<GoGame>::new("W".to_string(), vec!["de".to_string()]);
    /// assert_eq!(prop.identifier(), "W");
    /// let prop = SgfProp::<GoGame>::new("FOO".to_string(), vec!["de".to_string()]);
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
            Self::Invalid(identifier, _) => identifier.to_string(),
        }
    }

    /// Returns the `PropertyType` associated with the property.
    ///
    /// # Examples
    /// ```
    /// use sgf_parse::{PropertyType, SgfProp};
    /// use sgf_parse::game::GoGame;
    ///
    /// let prop = SgfProp::<GoGame>::new("W".to_string(), vec!["de".to_string()]);
    /// assert_eq!(prop.property_type(), Some(PropertyType::Move));
    /// let prop = SgfProp::<GoGame>::new("FOO".to_string(), vec!["de".to_string()]);
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

fn verify_empty(values: &[String]) -> Result<(), SgfPropError> {
    if !(values.is_empty() || (values.len() == 1 && values[0].is_empty())) {
        return Err(SgfPropError {});
    }
    Ok(())
}

fn parse_single_value<T: FromStr>(values: &[String]) -> Result<T, SgfPropError> {
    if values.len() != 1 {
        return Err(SgfPropError {});
    }
    values[0].parse().map_err(|_| SgfPropError {})
}

fn parse_single_text_value(values: &[String]) -> Result<Text, SgfPropError> {
    if values.len() != 1 {
        return Err(SgfPropError {});
    }
    Ok(Text {
        text: values[0].clone(),
    })
}

fn parse_list_composed_point<G: Game>(
    values: &[String],
) -> Result<HashSet<(G::Point, G::Point)>, SgfPropError> {
    let mut pairs = HashSet::new();
    for value in values.iter() {
        let pair = parse_tuple(value)?;
        if pair.0 == pair.1 || pairs.contains(&pair) {
            return Err(SgfPropError {});
        }
        pairs.insert(pair);
    }

    Ok(pairs)
}

fn parse_size(values: &[String]) -> Result<(u8, u8), SgfPropError> {
    if values.len() != 1 {
        return Err(SgfPropError {});
    }
    let value = &values[0];
    if value.contains(':') {
        parse_tuple(value)
    } else {
        let size = value.parse().map_err(|_| SgfPropError {})?;
        Ok((size, size))
    }
}

fn parse_labels<G: Game>(
    values: &[String],
) -> Result<HashSet<(G::Point, SimpleText)>, SgfPropError> {
    let mut labels = HashSet::new();
    for value in values.iter() {
        let (s1, s2) = split_compose(value)?;
        labels.insert((
            s1.parse().map_err(|_| SgfPropError {})?,
            SimpleText {
                text: s2.to_string(),
            },
        ));
    }
    if labels.is_empty() {
        return Err(SgfPropError {});
    }

    Ok(labels)
}

fn parse_figure(values: &[String]) -> Result<Option<(i64, SimpleText)>, SgfPropError> {
    if values.is_empty() || (values.len() == 1 && values[0].is_empty()) {
        return Ok(None);
    }
    if values.len() > 1 {
        return Err(SgfPropError {});
    }
    let (s1, s2) = split_compose(&values[0])?;

    Ok(Some((
        s1.parse().map_err(|_| SgfPropError {})?,
        SimpleText {
            text: s2.to_string(),
        },
    )))
}

fn parse_application(values: &[String]) -> Result<(SimpleText, SimpleText), SgfPropError> {
    if values.len() != 1 {
        return Err(SgfPropError {});
    }
    let (s1, s2) = split_compose(&values[0])?;
    Ok((
        SimpleText {
            text: s1.to_string(),
        },
        SimpleText {
            text: s2.to_string(),
        },
    ))
}
