use std::str::FromStr;

use super::SgfPropError;

/// An SGF [Color](https://www.red-bean.com/sgf/sgf4.html#types) value.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Color {
    Black,
    White,
}

/// An SGF [Double](https://www.red-bean.com/sgf/sgf4.html#double) value.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Double {
    One,
    Two,
}

/// An SGF [SimpleText](https://www.red-bean.com/sgf/sgf4.html#types) value.
///
/// The text itself will be the raw text as stored in an sgf file. Displays formatted and escaped
/// as [here](https://www.red-bean.com/sgf/sgf4.html#simpletext).
///
/// # Examples
/// ```
/// use sgf_parse::SimpleText;
///
/// let text = SimpleText { text: "Comment:\nall whitespace\treplaced".to_string() };
/// assert_eq!(format!("{}", text), "Comment: all whitespace replaced");
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct SimpleText {
    pub text: String,
}

/// An SGF [Text](https://www.red-bean.com/sgf/sgf4.html#types) value.
///
/// The text itself will be the raw text as stored in an sgf file. Displays formatted and escaped
/// as [here](https://www.red-bean.com/sgf/sgf4.html#text).
///
/// # Examples
/// ```
/// use sgf_parse::Text;
/// let text = Text { text: "Comment:\nnon-linebreak whitespace\treplaced".to_string() };
/// assert_eq!(format!("{}", text), "Comment:\nnon-linebreak whitespace replaced");
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Text {
    pub text: String,
}

impl FromStr for Double {
    type Err = SgfPropError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "1" {
            Ok(Self::One)
        } else if s == "2" {
            Ok(Self::Two)
        } else {
            Err(SgfPropError {})
        }
    }
}

impl FromStr for Color {
    type Err = SgfPropError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "B" {
            Ok(Self::Black)
        } else if s == "W" {
            Ok(Self::White)
        } else {
            Err(SgfPropError {})
        }
    }
}

impl std::fmt::Display for SimpleText {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = format_text(&self.text)
            .replace("\r\n", " ")
            .replace("\n\r", " ")
            .replace("\n", " ")
            .replace("\r", " ");
        f.write_str(&text)
    }
}

impl std::fmt::Display for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = format_text(&self.text);
        f.write_str(&text)
    }
}

fn format_text(s: &str) -> String {
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

#[cfg(test)]
mod test {
    #[test]
    pub fn format_text() {
        let text = super::Text {
            text: "Comment with\trandom whitespace\nescaped \\] and \\\\ and a soft \\\nlinebreak"
                .to_string(),
        };
        let expected = "Comment with random whitespace\nescaped ] and \\ and a soft linebreak";

        assert_eq!(format!("{}", text), expected);
    }

    #[test]
    pub fn format_simple_text() {
        let text = super::SimpleText { text:
            "Comment with\trandom\r\nwhitespace\n\rescaped \\] and \\\\ and\na soft \\\nlinebreak"
                .to_string()
        };
        let expected = "Comment with random whitespace escaped ] and \\ and a soft linebreak";

        assert_eq!(format!("{}", text), expected);
    }
}
