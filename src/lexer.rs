use super::errors::SgfParseError;

#[derive(Debug, PartialEq)]
pub enum Token {
    StartGameTree,
    EndGameTree,
    StartNode,
    Property((String, Vec<String>)),
}

pub struct Lexer<'a> {
    text: &'a str,
    cursor: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(text: &'a str) -> Self {
        Lexer { text, cursor: 0 }
    }

    fn trim_leading_whitespace(&mut self) {
        while self.cursor < self.text.len()
            && (self.text.as_bytes()[self.cursor] as char).is_ascii_whitespace()
        {
            self.cursor += 1;
        }
    }

    fn get_char(&mut self) -> Option<char> {
        let result = self.text[self.cursor..].chars().next();
        result.iter().for_each(|c| self.cursor += c.len_utf8());

        result
    }

    fn peek_char(&self) -> Option<char> {
        self.text[self.cursor..].chars().next()
    }

    fn get_property(&mut self) -> Result<(String, Vec<String>), SgfParseError> {
        Ok((self.get_prop_ident()?, self.get_prop_values()?))
    }

    fn get_prop_ident(&mut self) -> Result<String, SgfParseError> {
        let mut prop_ident = vec![];
        loop {
            match self.peek_char() {
                Some('[') => break,
                Some(c) if c.is_ascii_uppercase() => {
                    self.cursor += 1;
                    prop_ident.push(c);
                }
                Some(_c) => {
                    return Err(SgfParseError::ParseError(
                        "Unexpected property identifier value".to_string(),
                    ))
                }
                None => {
                    return Err(SgfParseError::ParseError(
                        "Missing property identified".to_string(),
                    ))
                }
            }
        }

        Ok(prop_ident.iter().collect())
    }

    fn get_prop_values(&mut self) -> Result<Vec<String>, SgfParseError> {
        let mut prop_values = vec![];
        loop {
            self.trim_leading_whitespace();
            match self.peek_char() {
                Some('[') => {
                    self.cursor += 1;
                    prop_values.push(self.get_prop_value()?);
                }
                _ => break,
            }
        }

        Ok(prop_values)
    }

    fn get_prop_value(&mut self) -> Result<String, SgfParseError> {
        let mut prop_value = vec![];
        let mut escaped = false;
        loop {
            match self.get_char() {
                Some(']') if !escaped => break,
                Some('\\') if !escaped => escaped = true,
                Some(c) => {
                    escaped = false;
                    prop_value.push(c);
                }
                None => {
                    return Err(SgfParseError::ParseError(
                        "Unexpected end of property".to_string(),
                    ))
                }
            }
        }

        Ok(prop_value.iter().collect())
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<(Token, std::ops::Range<usize>), SgfParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        let span_start = self.cursor;
        let token = match self.peek_char() {
            Some('(') => {
                self.cursor += 1;
                Token::StartGameTree
            }
            Some(')') => {
                self.cursor += 1;
                Token::EndGameTree
            }
            Some(';') => {
                self.cursor += 1;
                Token::StartNode
            }
            None => return None,
            _ => match self.get_property() {
                Ok(property) => Token::Property(property),
                Err(e) => return Some(Err(e)),
            },
        };
        let span = span_start..self.cursor;
        self.trim_leading_whitespace();

        Some(Ok((token, span)))
    }
}

#[cfg(test)]
mod test {
    use super::Lexer;
    use super::Token::*;

    #[test]
    fn lexer() {
        let sgf = "(;SZ[9]C[Some comment];B[de];W[fe])(;B[de];W[ff])";
        let lexer = Lexer::new(sgf);
        let expected = vec![
            (StartGameTree, 0..1),
            (StartNode, 1..2),
            (Property(("SZ".to_string(), vec!["9".to_string()])), 2..7),
            (
                Property(("C".to_string(), vec!["Some comment".to_string()])),
                7..22,
            ),
            (StartNode, 22..23),
            (Property(("B".to_string(), vec!["de".to_string()])), 23..28),
            (StartNode, 28..29),
            (Property(("W".to_string(), vec!["fe".to_string()])), 29..34),
            (EndGameTree, 34..35),
            (StartGameTree, 35..36),
            (StartNode, 36..37),
            (Property(("B".to_string(), vec!["de".to_string()])), 37..42),
            (StartNode, 42..43),
            (Property(("W".to_string(), vec!["ff".to_string()])), 43..48),
            (EndGameTree, 48..49),
        ];
        let tokens: Vec<_> = lexer.collect::<Result<_, _>>().unwrap();

        assert_eq!(tokens, expected);
    }
}
