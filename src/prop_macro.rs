macro_rules! sgf_prop {
    ($name:ident, $mv:ty, $pt:ty, $st:ty, { $($variants:tt)* }) => {
        /// An SGF Property with identifier and value.
        ///
        /// All [general properties](https://www.red-bean.com/sgf/properties.html) from the SGF
        /// specification and all game specific properties will return the approprite enum
        /// instance with parsed data. Unrecognized properties will return
        /// `Unknown`. Recognized general or game specific properties with invalid values will
        /// return `Invalid`.
        ///
        /// See [property value types](https://www.red-bean.com/sgf/sgf4.html#types) for a list of types
        /// recognized by SGF. For parsing purposes the following mappings are used:
        /// * 'Number' => [`i64`]
        /// * 'Real' => [`f64`]
        /// * 'Double' => [`Double`](`crate::props::Double`)
        /// * 'Color' => [`Color`](`crate::props::Color`)
        /// * 'SimpleText' => [`SimpleText`](`crate::props::SimpleText`)
        /// * 'Text' => [`Text`](`crate::props::Text`)
        /// * 'Point' => Game specific Point value (e.g.: [`crate::go::Point`])
        /// * 'Stone' => Game specific Stone value (e.g.: [`crate::go::Point`])
        /// * 'Move' => Game specific Move value (e.g.: [`crate::go::Move`])
        /// * 'List' => [`std::collections::HashSet`]
        /// * 'Compose' => [`tuple`] of the composed values
        #[derive(Clone, Debug, PartialEq)]
        pub enum $name {
            // Move properties
            B($mv),
            KO,
            MN(i64),
            W($mv),
            // Setup properties
            AB(std::collections::HashSet<$st>),
            AE(std::collections::HashSet<$pt>),
            AW(std::collections::HashSet<$st>),
            PL(crate::props::Color),
            // Node annotation properties
            C(crate::props::Text),
            DM(crate::props::Double),
            GB(crate::props::Double),
            GW(crate::props::Double),
            HO(crate::props::Double),
            N(crate::props::SimpleText),
            UC(crate::props::Double),
            V(f64),
            // Move annotation properties
            BM(crate::props::Double),
            DO,
            IT,
            TE(crate::props::Double),
            // Markup properties
            AR(std::collections::HashSet<($pt, $pt)>),
            CR(std::collections::HashSet<$pt>),
            DD(std::collections::HashSet<$pt>),
            LB(std::collections::HashSet<($pt, crate::props::SimpleText)>),
            LN(std::collections::HashSet<($pt, $pt)>),
            MA(std::collections::HashSet<$pt>),
            SL(std::collections::HashSet<$pt>),
            SQ(std::collections::HashSet<$pt>),
            TR(std::collections::HashSet<$pt>),
            // Root properties
            AP((crate::props::SimpleText, crate::props::SimpleText)),
            CA(crate::props::SimpleText),
            FF(i64),
            GM(i64),
            ST(i64),
            SZ((u8, u8)),
            // Game info properties
            AN(crate::props::SimpleText),
            BR(crate::props::SimpleText),
            BT(crate::props::SimpleText),
            CP(crate::props::SimpleText),
            DT(crate::props::SimpleText),
            EV(crate::props::SimpleText),
            GN(crate::props::SimpleText),
            GC(crate::props::Text),
            ON(crate::props::SimpleText),
            OT(crate::props::SimpleText),
            PB(crate::props::SimpleText),
            PC(crate::props::SimpleText),
            PW(crate::props::SimpleText),
            RE(crate::props::SimpleText),
            RO(crate::props::SimpleText),
            RU(crate::props::SimpleText),
            SO(crate::props::SimpleText),
            TM(f64),
            US(crate::props::SimpleText),
            WR(crate::props::SimpleText),
            WT(crate::props::SimpleText),
            // Timing properties
            BL(f64),
            OB(i64),
            OW(i64),
            WL(f64),
            // Miscellaneous properties
            FG(Option<(i64, crate::props::SimpleText)>),
            PM(i64),
            VW(std::collections::HashSet<$pt>),
            Unknown(String, Vec<String>),
            Invalid(String, Vec<String>),
            // Game specific properties
            $($variants)*
        }

        impl $name {
            fn parse_general_prop(identifier: String, values: Vec<String>) -> Self {
                use crate::props::parse::{
                    parse_elist, parse_list, parse_list_composed, parse_single_value, verify_empty,
                };

                let result = match &identifier[..] {
                    "B" => parse_single_value(&values).map(Self::B),
                    "KO" => verify_empty(&values).map(|()| Self::KO),
                    "MN" => parse_single_value(&values).map(Self::MN),
                    "W" => parse_single_value(&values).map(Self::W),
                    "AB" => parse_list(&values).map(Self::AB),
                    "AE" => parse_list(&values).map(Self::AE),
                    "AW" => parse_list(&values).map(Self::AW),
                    "PL" => parse_single_value(&values).map(Self::PL),
                    "C" => parse_single_value(&values).map(Self::C),
                    "DM" => parse_single_value(&values).map(Self::DM),
                    "GB" => parse_single_value(&values).map(Self::GB),
                    "GW" => parse_single_value(&values).map(Self::GW),
                    "HO" => parse_single_value(&values).map(Self::HO),
                    "N" => parse_single_value(&values).map(Self::N),
                    "UC" => parse_single_value(&values).map(Self::UC),
                    "V" => parse_single_value(&values).map(Self::V),
                    "DO" => verify_empty(&values).map(|()| Self::DO),
                    "IT" => verify_empty(&values).map(|()| Self::IT),
                    "BM" => parse_single_value(&values).map(Self::BM),
                    "TE" => parse_single_value(&values).map(Self::TE),
                    "AR" => parse_list_composed(&values).map(Self::AR),
                    "CR" => parse_list(&values).map(Self::CR),
                    "DD" => parse_elist(&values).map(Self::DD),
                    "LB" => parse_labels(&values).map(Self::LB),
                    "LN" => parse_list_composed(&values).map(Self::LN),
                    "MA" => parse_list(&values).map(Self::MA),
                    "SL" => parse_list(&values).map(Self::SL),
                    "SQ" => parse_list(&values).map(Self::SQ),
                    "TR" => parse_list(&values).map(Self::TR),
                    "AP" => parse_application(&values).map(Self::AP),
                    "CA" => parse_single_value(&values).map(Self::CA),
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
                    "AN" => parse_single_value(&values).map(Self::AN),
                    "BR" => parse_single_value(&values).map(Self::BR),
                    "BT" => parse_single_value(&values).map(Self::BT),
                    "CP" => parse_single_value(&values).map(Self::CP),
                    "DT" => parse_single_value(&values).map(Self::DT),
                    "EV" => parse_single_value(&values).map(Self::EV),
                    "GN" => parse_single_value(&values).map(Self::GN),
                    "GC" => parse_single_value(&values).map(Self::GC),
                    "ON" => parse_single_value(&values).map(Self::ON),
                    "OT" => parse_single_value(&values).map(Self::OT),
                    "PB" => parse_single_value(&values).map(Self::PB),
                    "PC" => parse_single_value(&values).map(Self::PC),
                    "PW" => parse_single_value(&values).map(Self::PW),
                    "RE" => parse_single_value(&values).map(Self::RE),
                    "RO" => parse_single_value(&values).map(Self::RO),
                    "RU" => parse_single_value(&values).map(Self::RU),
                    "SO" => parse_single_value(&values).map(Self::SO),
                    "TM" => parse_single_value(&values).map(Self::TM),
                    "US" => parse_single_value(&values).map(Self::US),
                    "WR" => parse_single_value(&values).map(Self::WR),
                    "WT" => parse_single_value(&values).map(Self::WT),
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
                    "VW" => parse_elist(&values).map(Self::VW),
                    _ => return Self::Unknown(identifier, values),
                };
                result.unwrap_or(Self::Invalid(identifier, values))
            }

            fn general_identifier(&self) -> Option<String> {
                match self {
                    Self::B(_) => Some("B".to_string()),
                    Self::KO => Some("KO".to_string()),
                    Self::MN(_) => Some("MN".to_string()),
                    Self::W(_) => Some("W".to_string()),
                    Self::AB(_) => Some("AB".to_string()),
                    Self::AE(_) => Some("AE".to_string()),
                    Self::AW(_) => Some("AW".to_string()),
                    Self::PL(_) => Some("PL".to_string()),
                    Self::C(_) => Some("C".to_string()),
                    Self::DM(_) => Some("DM".to_string()),
                    Self::GB(_) => Some("GB".to_string()),
                    Self::GW(_) => Some("GW".to_string()),
                    Self::HO(_) => Some("HO".to_string()),
                    Self::N(_) => Some("N".to_string()),
                    Self::UC(_) => Some("UC".to_string()),
                    Self::V(_) => Some("V".to_string()),
                    Self::DO => Some("DO".to_string()),
                    Self::IT => Some("IT".to_string()),
                    Self::BM(_) => Some("BM".to_string()),
                    Self::TE(_) => Some("TE".to_string()),
                    Self::AR(_) => Some("AR".to_string()),
                    Self::CR(_) => Some("CR".to_string()),
                    Self::DD(_) => Some("DD".to_string()),
                    Self::LB(_) => Some("LB".to_string()),
                    Self::LN(_) => Some("LN".to_string()),
                    Self::MA(_) => Some("MA".to_string()),
                    Self::SL(_) => Some("SL".to_string()),
                    Self::SQ(_) => Some("SQ".to_string()),
                    Self::TR(_) => Some("TR".to_string()),
                    Self::AP(_) => Some("AP".to_string()),
                    Self::CA(_) => Some("CA".to_string()),
                    Self::FF(_) => Some("FF".to_string()),
                    Self::GM(_) => Some("GM".to_string()),
                    Self::ST(_) => Some("ST".to_string()),
                    Self::SZ(_) => Some("SZ".to_string()),
                    Self::AN(_) => Some("AN".to_string()),
                    Self::BR(_) => Some("BR".to_string()),
                    Self::BT(_) => Some("BT".to_string()),
                    Self::CP(_) => Some("CP".to_string()),
                    Self::DT(_) => Some("DT".to_string()),
                    Self::EV(_) => Some("EV".to_string()),
                    Self::GN(_) => Some("GN".to_string()),
                    Self::GC(_) => Some("GC".to_string()),
                    Self::ON(_) => Some("ON".to_string()),
                    Self::OT(_) => Some("OT".to_string()),
                    Self::PB(_) => Some("PB".to_string()),
                    Self::PC(_) => Some("PC".to_string()),
                    Self::PW(_) => Some("PW".to_string()),
                    Self::RE(_) => Some("RE".to_string()),
                    Self::RO(_) => Some("RO".to_string()),
                    Self::RU(_) => Some("RU".to_string()),
                    Self::SO(_) => Some("SO".to_string()),
                    Self::TM(_) => Some("TM".to_string()),
                    Self::US(_) => Some("US".to_string()),
                    Self::WR(_) => Some("WR".to_string()),
                    Self::WT(_) => Some("WT".to_string()),
                    Self::BL(_) => Some("BL".to_string()),
                    Self::OB(_) => Some("OB".to_string()),
                    Self::OW(_) => Some("OW".to_string()),
                    Self::WL(_) => Some("WL".to_string()),
                    Self::FG(_) => Some("FG".to_string()),
                    Self::PM(_) => Some("PM".to_string()),
                    Self::VW(_) => Some("VW".to_string()),
                    Self::Invalid(identifier, _) => Some(identifier.to_string()),
                    Self::Unknown(identifier, _) => Some(identifier.to_string()),
                    #[allow(unreachable_patterns)]
                    _ => None,
                }
            }

            fn general_property_type(&self) -> Option<PropertyType> {
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

            fn serialize_prop_value(&self) -> Option<String> {
                match self {
                    Self::B(x) => Some(x.to_sgf()),
                    Self::KO => Some("".to_string()),
                    Self::MN(x) => Some(x.to_sgf()),
                    Self::W(x) => Some(x.to_sgf()),
                    Self::AB(x) => Some(x.to_sgf()),
                    Self::AE(x) => Some(x.to_sgf()),
                    Self::AW(x) => Some(x.to_sgf()),
                    Self::PL(x) => Some(x.to_sgf()),
                    Self::C(x) => Some(x.to_sgf()),
                    Self::DM(x) => Some(x.to_sgf()),
                    Self::GB(x) => Some(x.to_sgf()),
                    Self::GW(x) => Some(x.to_sgf()),
                    Self::HO(x) => Some(x.to_sgf()),
                    Self::N(x) => Some(x.to_sgf()),
                    Self::UC(x) => Some(x.to_sgf()),
                    Self::V(x) => Some(x.to_sgf()),
                    Self::AR(x) => Some(x.to_sgf()),
                    Self::CR(x) => Some(x.to_sgf()),
                    Self::DO => Some("".to_string()),
                    Self::IT => Some("".to_string()),
                    Self::BM(x) => Some(x.to_sgf()),
                    Self::TE(x) => Some(x.to_sgf()),
                    Self::DD(x) => Some(x.to_sgf()),
                    Self::LB(x) => Some(x.to_sgf()),
                    Self::LN(x) => Some(x.to_sgf()),
                    Self::MA(x) => Some(x.to_sgf()),
                    Self::SL(x) => Some(x.to_sgf()),
                    Self::SQ(x) => Some(x.to_sgf()),
                    Self::TR(x) => Some(x.to_sgf()),
                    Self::AP(x) => Some(x.to_sgf()),
                    Self::CA(x) => Some(x.to_sgf()),
                    Self::FF(x) => Some(x.to_sgf()),
                    Self::GM(x) => Some(x.to_sgf()),
                    Self::ST(x) => Some(x.to_sgf()),
                    Self::SZ(x) => Some(x.to_sgf()),
                    Self::AN(x) => Some(x.to_sgf()),
                    Self::BR(x) => Some(x.to_sgf()),
                    Self::BT(x) => Some(x.to_sgf()),
                    Self::CP(x) => Some(x.to_sgf()),
                    Self::DT(x) => Some(x.to_sgf()),
                    Self::EV(x) => Some(x.to_sgf()),
                    Self::GN(x) => Some(x.to_sgf()),
                    Self::GC(x) => Some(x.to_sgf()),
                    Self::ON(x) => Some(x.to_sgf()),
                    Self::OT(x) => Some(x.to_sgf()),
                    Self::PB(x) => Some(x.to_sgf()),
                    Self::PC(x) => Some(x.to_sgf()),
                    Self::PW(x) => Some(x.to_sgf()),
                    Self::RE(x) => Some(x.to_sgf()),
                    Self::RO(x) => Some(x.to_sgf()),
                    Self::RU(x) => Some(x.to_sgf()),
                    Self::SO(x) => Some(x.to_sgf()),
                    Self::TM(x) => Some(x.to_sgf()),
                    Self::US(x) => Some(x.to_sgf()),
                    Self::WR(x) => Some(x.to_sgf()),
                    Self::WT(x) => Some(x.to_sgf()),
                    Self::BL(x) => Some(x.to_sgf()),
                    Self::OB(x) => Some(x.to_sgf()),
                    Self::OW(x) => Some(x.to_sgf()),
                    Self::WL(x) => Some(x.to_sgf()),
                    Self::FG(x) => Some(x.to_sgf()),
                    Self::PM(x) => Some(x.to_sgf()),
                    Self::VW(x) => Some(x.to_sgf()),
                    Self::Unknown(_, x) => Some(x.to_sgf()),
                    Self::Invalid(_, x) => Some(x.to_sgf()),
                    #[allow(unreachable_patterns)]
                    _ => None,
                }
            }

            pub fn general_validate_properties(properties: &[Self], is_root: bool) -> Result<(), crate::InvalidNodeError> {
                use crate::InvalidNodeError;
                let mut identifiers = HashSet::new();
                let mut markup_points = HashSet::new();
                let mut setup_node = false;
                let mut move_node = false;
                let mut move_seen = false;
                let mut exclusive_node_annotations = 0;
                let mut move_annotation_count = 0;
                for prop in properties {
                    match prop {
                        Prop::B(_) => {
                            move_seen = true;
                            if identifiers.contains("W") {
                                return Err(InvalidNodeError::MultipleMoves(format!(
                                    "{:?}",
                                    properties.to_vec()
                                )));
                            }
                        }
                        Prop::W(_) => {
                            move_seen = true;
                            if identifiers.contains("B") {
                                return Err(InvalidNodeError::MultipleMoves(format!(
                                    "{:?}",
                                    properties.to_vec()
                                )));
                            }
                        }
                        Prop::CR(ps) | Prop::MA(ps) | Prop::SL(ps) | Prop::SQ(ps) | Prop::TR(ps) => {
                            for p in ps.iter() {
                                if markup_points.contains(&p) {
                                    return Err(InvalidNodeError::RepeatedMarkup(format!(
                                        "{:?}",
                                        properties.to_vec()
                                    )));
                                }
                                markup_points.insert(p);
                            }
                        }
                        Prop::DM(_) | Prop::UC(_) | Prop::GW(_) | Prop::GB(_) => {
                            exclusive_node_annotations += 1
                        }
                        Prop::BM(_) | Prop::DO | Prop::IT | Prop::TE(_) => move_annotation_count += 1,
                        Prop::Invalid(identifier, values) => {
                            return Err(InvalidNodeError::InvalidProperty(format!(
                                "{}, {:?}",
                                identifier, values
                            )))
                        }
                        _ => {}
                    }
                    match prop.property_type() {
                        Some(PropertyType::Move) => move_node = true,
                        Some(PropertyType::Setup) => setup_node = true,
                        Some(PropertyType::Root) => {
                            if !is_root {
                                return Err(InvalidNodeError::UnexpectedRootProperties(format!(
                                            "{:?}",
                                            properties
                                )));
                            }
                        }
                        _ => {}
                    }
                    let ident = prop.identifier();
                    if identifiers.contains(&ident) {
                        return Err(InvalidNodeError::RepeatedIdentifier(format!(
                                    "{:?}",
                                    properties.to_vec()
                        )));
                    }
                    identifiers.insert(prop.identifier());
                }
                if setup_node && move_node {
                    return Err(InvalidNodeError::SetupAndMove(format!(
                                "{:?}",
                                properties.to_vec()
                    )));
                }
                if identifiers.contains("KO") && !(identifiers.contains("B") || identifiers.contains("W")) {
                    return Err(InvalidNodeError::KoWithoutMove(format!(
                                "{:?}",
                                properties.to_vec()
                    )));
                }
                if move_annotation_count > 1 {
                    return Err(InvalidNodeError::MultipleMoveAnnotations(format!(
                                "{:?}",
                                properties.to_vec()
                    )));
                }
                if move_annotation_count == 1 && !move_seen {
                    return Err(InvalidNodeError::UnexpectedMoveAnnotation(format!(
                                "{:?}",
                                properties.to_vec()
                    )));
                }
                if exclusive_node_annotations > 1 {
                    return Err(InvalidNodeError::MultipleExclusiveAnnotations(format!(
                                "{:?}",
                                properties.to_vec()
                    )));
                }
                Ok(())
            }
        }


        fn parse_size(values: &[String]) -> Result<(u8, u8), SgfPropError> {
            if values.len() != 1 {
                return Err(SgfPropError {});
            }
            let value = &values[0];
            if value.contains(':') {
                crate::props::parse::parse_tuple(value)
            } else {
                let size = value.parse().map_err(|_| SgfPropError {})?;
                Ok((size, size))
            }
        }

        fn parse_labels(
            values: &[String],
        ) -> Result<HashSet<($pt, crate::SimpleText)>, SgfPropError> {
            let mut labels = HashSet::new();
            for value in values.iter() {
                let (s1, s2) = crate::props::parse::split_compose(value)?;
                labels.insert((
                        s1.parse().map_err(|_| SgfPropError {})?,
                        crate::SimpleText {
                            text: s2.to_string(),
                        },
                ));
            }
            if labels.is_empty() {
                return Err(SgfPropError {});
            }

            Ok(labels)
        }

        fn parse_figure(values: &[String]) -> Result<Option<(i64, crate::SimpleText)>, SgfPropError> {
            if values.is_empty() || (values.len() == 1 && values[0].is_empty()) {
                return Ok(None);
            }
            if values.len() > 1 {
                return Err(SgfPropError {});
            }
            let (s1, s2) = crate::props::parse::split_compose(&values[0])?;

            Ok(Some((
                        s1.parse().map_err(|_| SgfPropError {})?,
                        crate::SimpleText {
                            text: s2.to_string(),
                        },
            )))
        }

        fn parse_application(values: &[String]) -> Result<(crate::SimpleText, crate::SimpleText), SgfPropError> {
            if values.len() != 1 {
                return Err(SgfPropError {});
            }
            let (s1, s2) = crate::props::parse::split_compose(&values[0])?;
            Ok((
                    crate::SimpleText {
                        text: s1.to_string(),
                    },
                    crate::SimpleText {
                        text: s2.to_string(),
                    },
            ))
        }
    }
}
