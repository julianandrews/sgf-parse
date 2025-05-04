pub fn tokenize(
    text: &str,
) -> impl Iterator<Item = Result<(Token, std::ops::Range<usize>), LexerError>> + '_ {
    Lexer { text, cursor: 0 }
}

#[derive(Debug, PartialEq)]
pub enum Token {
    StartGameTree,
    EndGameTree,
    StartNode,
    Property((String, Vec<String>)),
}

/// Error type for failures to tokenize text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LexerError {
    UnexpectedPropertyIdentifier,
    MissingPropertyIdentifier,
    UnexpectedEndOfProperty,
}

impl std::fmt::Display for LexerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LexerError::UnexpectedPropertyIdentifier => {
                write!(f, "Unexpected property identifier value")
            }
            LexerError::MissingPropertyIdentifier => {
                write!(f, "Missing property identifier")
            }
            LexerError::UnexpectedEndOfProperty => write!(f, "Unexpected end of property"),
        }
    }
}

impl std::error::Error for LexerError {}

struct Lexer<'a> {
    text: &'a str,
    cursor: usize,
}

impl Lexer<'_> {
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

    fn get_property(&mut self) -> Result<(String, Vec<String>), LexerError> {
        Ok((self.get_prop_ident()?, self.get_prop_values()?))
    }

    fn get_prop_ident(&mut self) -> Result<String, LexerError> {
        let mut prop_ident = vec![];
        loop {
            match self.peek_char() {
                Some('[') => break,
                Some(c) if c.is_ascii() => {
                    self.cursor += 1;
                    prop_ident.push(c);
                }
                Some(_c) => return Err(LexerError::UnexpectedEndOfProperty),
                None => return Err(LexerError::MissingPropertyIdentifier),
            }
        }

        Ok(prop_ident.iter().collect())
    }

    fn get_prop_values(&mut self) -> Result<Vec<String>, LexerError> {
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

    fn get_prop_value(&mut self) -> Result<String, LexerError> {
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
                None => return Err(LexerError::UnexpectedEndOfProperty),
            }
        }

        Ok(prop_value.iter().collect())
    }
}

impl Iterator for Lexer<'_> {
    type Item = Result<(Token, std::ops::Range<usize>), LexerError>;

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
    use super::tokenize;
    use super::Token::*;

    #[test]
    fn lexer() {
        let sgf = "(;SZ[9]C[Some comment];B[de];W[fe])(;B[de];W[ff])";
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
        let tokens: Vec<_> = tokenize(sgf).collect::<Result<_, _>>().unwrap();

        assert_eq!(tokens, expected);
    }

    #[test]
    fn handles_old_style_properties() {
        let sgf = "(;CoPyright[text])";
        let expected = vec![
            (StartGameTree, 0..1),
            (StartNode, 1..2),
            (
                Property(("CoPyright".to_string(), vec!["text".to_string()])),
                2..17,
            ),
            (EndGameTree, 17..18),
        ];
        let tokens: Vec<_> = tokenize(sgf).collect::<Result<_, _>>().unwrap();

        assert_eq!(tokens, expected);
    }
}
