use std::collections::HashSet;
use std::hash::Hash;
use std::str::FromStr;

use super::{FromCompressedList, SgfPropError};
// use super::{SimpleText, Text};

pub fn parse_single_value<T: FromStr>(values: &[String]) -> Result<T, SgfPropError> {
    if values.len() != 1 {
        return Err(SgfPropError {});
    }
    values[0].parse().map_err(|_| SgfPropError {})
}

pub fn parse_tuple<T1: FromStr, T2: FromStr>(value: &str) -> Result<(T1, T2), SgfPropError> {
    let (s1, s2) = split_compose(value)?;
    Ok((
        s1.parse().map_err(|_| SgfPropError {})?,
        s2.parse().map_err(|_| SgfPropError {})?,
    ))
}

pub fn parse_elist<T: FromStr + FromCompressedList + Eq + Hash>(
    values: &[String],
) -> Result<HashSet<T>, SgfPropError> {
    let mut elements = HashSet::new();
    for value in values {
        if value.contains(':') {
            let (upper_left, lower_right): (T, T) = parse_tuple(value)?;
            elements.extend(T::from_compressed_list(&upper_left, &lower_right)?.into_iter());
        } else {
            let item = value.parse().map_err(|_| SgfPropError {})?;
            elements.insert(item);
        }
    }
    Ok(elements)
}

pub fn parse_list<T: FromStr + FromCompressedList + Eq + std::hash::Hash>(
    values: &[String],
) -> Result<HashSet<T>, SgfPropError> {
    let points = parse_elist::<T>(values)?;
    if points.is_empty() {
        return Err(SgfPropError {});
    }

    Ok(points)
}

pub fn parse_list_composed<T: FromStr + Eq + Hash>(
    values: &[String],
) -> Result<HashSet<(T, T)>, SgfPropError> {
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

pub fn split_compose(value: &str) -> Result<(&str, &str), SgfPropError> {
    let parts: Vec<&str> = value.split(':').collect();
    if parts.len() != 2 {
        return Err(SgfPropError {});
    }

    Ok((parts[0], parts[1]))
}

pub fn verify_empty(values: &[String]) -> Result<(), SgfPropError> {
    if !(values.is_empty() || (values.len() == 1 && values[0].is_empty())) {
        return Err(SgfPropError {});
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::parse_list;
    use crate::go::Point;
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
        .map(|(x, y)| Point { x, y })
        .collect();

        let result: HashSet<_> = parse_list::<Point>(&values).unwrap();

        assert_eq!(result, expected);
    }
}
