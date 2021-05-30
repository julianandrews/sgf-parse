use std::collections::HashSet;
use std::str::FromStr;

use super::Game;
use super::SgfPropError;
use super::SimpleText;

pub fn parse_single_simple_text_value(values: &[String]) -> Result<SimpleText, SgfPropError> {
    if values.len() != 1 {
        return Err(SgfPropError {});
    }
    Ok(SimpleText {
        text: values[0].clone(),
    })
}

pub fn parse_list_point<G: Game>(values: &[String]) -> Result<HashSet<G::Point>, SgfPropError> {
    let points = parse_elist_point::<G>(values)?;
    if points.is_empty() {
        return Err(SgfPropError {});
    }

    Ok(points)
}

pub fn parse_elist_point<G: Game>(values: &[String]) -> Result<HashSet<G::Point>, SgfPropError> {
    G::parse_point_list(values)
}

pub fn parse_list_stone<G: Game>(values: &[String]) -> Result<HashSet<G::Stone>, SgfPropError> {
    let stones = G::parse_stone_list(values)?;
    if stones.is_empty() {
        return Err(SgfPropError {});
    }

    Ok(stones)
}

pub fn split_compose(value: &str) -> Result<(&str, &str), SgfPropError> {
    let parts: Vec<&str> = value.split(':').collect();
    if parts.len() != 2 {
        return Err(SgfPropError {});
    }

    Ok((parts[0], parts[1]))
}

pub fn parse_tuple<T1: FromStr, T2: FromStr>(value: &str) -> Result<(T1, T2), SgfPropError> {
    let (s1, s2) = split_compose(value)?;
    Ok((
        s1.parse().map_err(|_| SgfPropError {})?,
        s2.parse().map_err(|_| SgfPropError {})?,
    ))
}

#[cfg(test)]
mod test {
    use crate::game::{GoGame, GoPoint};
    use std::collections::HashSet;

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
        .map(|(x, y)| GoPoint { x, y })
        .collect();

        let result: HashSet<_> = super::parse_list_point::<GoGame>(&values).unwrap();

        assert_eq!(result, expected);
    }
}
