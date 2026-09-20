use crate::{
    Error, ParseFailure, Result,
    function::Function,
    lexer::{NumberLiteral, NumberSuffix, Token, TokenKind},
};
use std::ops::Range;

/// Splits program text into tokens.
pub(crate) fn lex(text: &str) -> Result<Vec<Token>> {
    let mut lexer = Lexer {
        text,
        bytes: text.as_bytes(),
        position: 0,
        tokens: Vec::new(),
    };

    lexer.run()?;

    Ok(lexer.tokens)
}

struct Lexer<'a> {
    text: &'a str,
    bytes: &'a [u8],
    position: usize,
    tokens: Vec<Token>,
}

impl Lexer<'_> {
    fn run(&mut self) -> Result<()> {
        while let Some(&byte) = self.bytes.get(self.position) {
            let start = self.position;

            match byte {
                b' ' | b'\t' | b'\n' | b'\r' => self.position += 1,
                b'0'..=b'9' => self.number(start, false)?,
                b'.' => self.dot(start)?,
                b'a'..=b'z' | b'A'..=b'Z' | b'_' => self.identifier(start),
                b'`' => self.quoted_name(start)?,
                b'"' => self.string_literal(start)?,
                b'[' => self.left_bracket(start)?,
                _ => self.operator(start)?,
            }
        }

        Ok(())
    }

    fn byte_at(&self, position: usize) -> Option<u8> {
        self.bytes.get(position).copied()
    }

    fn push(&mut self, kind: TokenKind, range: Range<usize>) {
        self.tokens.push(Token { kind, range });
    }

    fn fail<T>(&self, range: Range<usize>, failure: ParseFailure) -> Result<T> {
        Err(Error::Parse { range, failure })
    }

    fn skip_digits(&mut self) {
        while self.byte_at(self.position).is_some_and(is_digit) {
            self.position += 1;
        }
    }

    fn skip_identifier(&mut self) {
        while self.byte_at(self.position).is_some_and(is_identifier_char) {
            self.position += 1;
        }
    }

    /// Lexes a number by maximal munch. A leading dot has already been
    /// consumed when `from_dot` is set.
    fn number(&mut self, start: usize, from_dot: bool) -> Result<()> {
        let mut fraction = from_dot;

        self.skip_digits();

        if !from_dot && self.byte_at(self.position) == Some(b'.') {
            let after_dot = self.byte_at(self.position + 1);

            if after_dot.is_some_and(is_digit) {
                self.position += 1;
                fraction = true;
                self.skip_digits();
            } else if !after_dot.is_some_and(is_identifier_start) {
                // A trailing dot belongs to the number unless a member follows.
                self.position += 1;
                fraction = true;
            }
        }

        let text = self.text[start..self.position].to_owned();
        let suffix_start = self.position;

        self.skip_identifier();

        let suffix = if suffix_start == self.position {
            None
        } else {
            let spelling = &self.text[suffix_start..self.position];
            let Some(suffix) = NumberSuffix::from_spelling(spelling) else {
                return self.fail(
                    suffix_start..self.position,
                    ParseFailure::UnknownSuffix {
                        suffix: spelling.to_owned(),
                    },
                );
            };

            if fraction {
                return self.fail(
                    suffix_start..self.position,
                    ParseFailure::SuffixOnFraction {
                        suffix: spelling.to_owned(),
                    },
                );
            }

            Some(suffix)
        };

        self.push(
            TokenKind::Number(NumberLiteral {
                text,
                fraction,
                suffix,
            }),
            start..self.position,
        );

        Ok(())
    }

    /// Lexes a dot as a number start or as the attached swizzle dot.
    fn dot(&mut self, start: usize) -> Result<()> {
        if self.byte_at(start + 1).is_some_and(is_digit) {
            self.position += 1;
            return self.number(start, true);
        }

        if start == 0 || is_whitespace(self.bytes[start - 1]) {
            return self.fail(start..start + 1, ParseFailure::DetachedPostfix);
        }

        if !self.byte_at(start + 1).is_some_and(is_identifier_start) {
            return self.fail(start..start + 1, ParseFailure::MemberExpected);
        }

        self.position += 1;
        self.push(TokenKind::Dot, start..start + 1);

        Ok(())
    }

    fn identifier(&mut self, start: usize) {
        self.skip_identifier();

        let name = &self.text[start..self.position];
        let kind = match name {
            "true" => TokenKind::True,
            "false" => TokenKind::False,

            _ => match Function::from_name(name) {
                Some(function) => TokenKind::Function(function),
                None => TokenKind::Identifier(name.to_owned()),
            },
        };

        self.push(kind, start..self.position);
    }

    fn quoted_name(&mut self, start: usize) -> Result<()> {
        let Some(end) = self.find(b'`', start + 1) else {
            return self.fail(start..self.bytes.len(), ParseFailure::UnterminatedName);
        };

        if end == start + 1 {
            return self.fail(start..end + 1, ParseFailure::EmptyName);
        }

        self.position = end + 1;
        self.push(
            TokenKind::QuotedName(self.text[start + 1..end].to_owned()),
            start..end + 1,
        );

        Ok(())
    }

    fn string_literal(&mut self, start: usize) -> Result<()> {
        let Some(end) = self.find(b'"', start + 1) else {
            return self.fail(start..self.bytes.len(), ParseFailure::UnterminatedString);
        };

        self.position = end + 1;
        self.push(
            TokenKind::StringLiteral(self.text[start + 1..end].to_owned()),
            start..end + 1,
        );

        Ok(())
    }

    fn find(&self, byte: u8, from: usize) -> Option<usize> {
        self.bytes[from..]
            .iter()
            .position(|&candidate| candidate == byte)
            .map(|offset| from + offset)
    }

    fn left_bracket(&mut self, start: usize) -> Result<()> {
        if start == 0 || is_whitespace(self.bytes[start - 1]) {
            return self.fail(start..start + 1, ParseFailure::DetachedPostfix);
        }

        self.position += 1;
        self.push(TokenKind::LeftBracket, start..start + 1);

        Ok(())
    }

    fn operator(&mut self, start: usize) -> Result<()> {
        let next = self.byte_at(start + 1);
        let (kind, length) = match self.bytes[start] {
            b'+' => (TokenKind::Plus, 1),
            b'-' => (TokenKind::Minus, 1),
            b'*' => (TokenKind::Star, 1),
            b'/' => (TokenKind::Slash, 1),
            b'(' => (TokenKind::LeftParen, 1),
            b')' => (TokenKind::RightParen, 1),
            b']' => (TokenKind::RightBracket, 1),
            b',' => (TokenKind::Comma, 1),
            b';' => (TokenKind::Semicolon, 1),
            b'^' => (TokenKind::Xor, 1),
            b'=' if next == Some(b'=') => (TokenKind::Equal, 2),
            b'=' => (TokenKind::Assign, 1),
            b'!' if next == Some(b'=') => (TokenKind::NotEqual, 2),
            b'!' => (TokenKind::Not, 1),
            b'<' if next == Some(b'=') => (TokenKind::LessEqual, 2),
            b'<' => (TokenKind::Less, 1),
            b'>' if next == Some(b'=') => (TokenKind::GreaterEqual, 2),
            b'>' => (TokenKind::Greater, 1),
            b'&' if next == Some(b'&') => (TokenKind::And, 2),
            b'|' if next == Some(b'|') => (TokenKind::Or, 2),

            _ => {
                let character = self.text[start..]
                    .chars()
                    .next()
                    .expect("the position sits on a character boundary");

                return self.fail(
                    start..start + character.len_utf8(),
                    ParseFailure::UnexpectedCharacter { character },
                );
            }
        };

        self.position += length;
        self.push(kind, start..start + length);

        Ok(())
    }
}

fn is_digit(byte: u8) -> bool {
    byte.is_ascii_digit()
}

fn is_identifier_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_'
}

fn is_identifier_char(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

fn is_whitespace(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\n' | b'\r')
}

#[cfg(test)]
mod tests {
    use crate::{
        Error, ParseFailure,
        function::Function,
        lexer::{NumberLiteral, NumberSuffix, TokenKind, lex},
    };
    use std::ops::Range;

    fn kinds(text: &str) -> Vec<TokenKind> {
        lex(text)
            .unwrap()
            .into_iter()
            .map(|token| token.kind)
            .collect()
    }

    fn ranges(text: &str) -> Vec<Range<usize>> {
        lex(text)
            .unwrap()
            .into_iter()
            .map(|token| token.range)
            .collect()
    }

    fn failure(text: &str) -> (Range<usize>, ParseFailure) {
        match lex(text).unwrap_err() {
            Error::Parse { range, failure } => (range, failure),
            error => panic!("expected a parse error, found {error:?}"),
        }
    }

    fn number(text: &str, fraction: bool, suffix: Option<NumberSuffix>) -> TokenKind {
        TokenKind::Number(NumberLiteral {
            text: text.to_owned(),
            fraction,
            suffix,
        })
    }

    fn name(text: &str) -> TokenKind {
        TokenKind::Identifier(text.to_owned())
    }

    #[test]
    fn whitespace_separates_tokens_and_is_otherwise_insignificant() {
        assert_eq!(
            kinds(" a\t=\n1 ;\r\n"),
            vec![
                name("a"),
                TokenKind::Assign,
                number("1", false, None),
                TokenKind::Semicolon,
            ]
        );
        assert_eq!(kinds(""), vec![]);
    }

    #[test]
    fn numbers_munch_maximally() {
        assert_eq!(kinds("1.5"), vec![number("1.5", true, None)]);
        assert_eq!(kinds(".5"), vec![number(".5", true, None)]);
        assert_eq!(kinds("2."), vec![number("2.", true, None)]);
        assert_eq!(kinds("42"), vec![number("42", false, None)]);
        assert_eq!(
            kinds("2u8"),
            vec![number("2", false, Some(NumberSuffix::U8))]
        );
        assert_eq!(
            kinds("2u16 2u32 2f32"),
            vec![
                number("2", false, Some(NumberSuffix::U16)),
                number("2", false, Some(NumberSuffix::U32)),
                number("2", false, Some(NumberSuffix::F32)),
            ]
        );
    }

    #[test]
    fn a_trailing_dot_before_a_letter_begins_a_swizzle() {
        assert_eq!(
            kinds("2.rr"),
            vec![number("2", false, None), TokenKind::Dot, name("rr")]
        );
        assert_eq!(
            kinds("2.5.rr"),
            vec![number("2.5", true, None), TokenKind::Dot, name("rr")]
        );
        assert_eq!(
            kinds("0.5.rrr"),
            vec![number("0.5", true, None), TokenKind::Dot, name("rrr")]
        );
    }

    #[test]
    fn a_swizzle_member_may_be_a_reserved_name() {
        assert_eq!(
            kinds("v.r"),
            vec![name("v"), TokenKind::Dot, TokenKind::Function(Function::R)]
        );
        assert_eq!(
            kinds("v.rgb"),
            vec![
                name("v"),
                TokenKind::Dot,
                TokenKind::Function(Function::Rgb)
            ]
        );
    }

    #[test]
    fn an_unknown_suffix_errors() {
        assert_eq!(
            failure("2abc"),
            (
                1..4,
                ParseFailure::UnknownSuffix {
                    suffix: "abc".to_owned()
                }
            )
        );
        assert_eq!(
            failure("2u"),
            (
                1..2,
                ParseFailure::UnknownSuffix {
                    suffix: "u".to_owned()
                }
            )
        );
    }

    #[test]
    fn a_suffix_on_a_fraction_errors() {
        assert_eq!(
            failure("1.5u8"),
            (
                3..5,
                ParseFailure::SuffixOnFraction {
                    suffix: "u8".to_owned()
                }
            )
        );
        assert_eq!(
            failure(".5f32"),
            (
                2..5,
                ParseFailure::SuffixOnFraction {
                    suffix: "f32".to_owned()
                }
            )
        );
    }

    #[test]
    fn multi_character_operators_are_single_tokens() {
        assert_eq!(
            kinds("== != <= >= < > && || ^ ! = + - * /"),
            vec![
                TokenKind::Equal,
                TokenKind::NotEqual,
                TokenKind::LessEqual,
                TokenKind::GreaterEqual,
                TokenKind::Less,
                TokenKind::Greater,
                TokenKind::And,
                TokenKind::Or,
                TokenKind::Xor,
                TokenKind::Not,
                TokenKind::Assign,
                TokenKind::Plus,
                TokenKind::Minus,
                TokenKind::Star,
                TokenKind::Slash,
            ]
        );
    }

    #[test]
    fn a_binding_equals_is_never_carved_from_a_comparison() {
        assert_eq!(kinds("a==b"), vec![name("a"), TokenKind::Equal, name("b")]);
        assert_eq!(kinds("a=b"), vec![name("a"), TokenKind::Assign, name("b")]);
    }

    #[test]
    fn there_is_no_double_minus_token() {
        assert_eq!(
            kinds("--x"),
            vec![TokenKind::Minus, TokenKind::Minus, name("x")]
        );
    }

    #[test]
    fn a_lone_ampersand_or_bar_errors() {
        assert_eq!(
            failure("a & b"),
            (2..3, ParseFailure::UnexpectedCharacter { character: '&' })
        );
        assert_eq!(
            failure("a | b"),
            (2..3, ParseFailure::UnexpectedCharacter { character: '|' })
        );
    }

    #[test]
    fn an_unexpected_character_names_itself_with_its_byte_range() {
        assert_eq!(
            failure("a # b"),
            (2..3, ParseFailure::UnexpectedCharacter { character: '#' })
        );
        assert_eq!(
            failure("a \u{e9} b"),
            (
                2..4,
                ParseFailure::UnexpectedCharacter {
                    character: '\u{e9}'
                }
            )
        );
    }

    #[test]
    fn postfixes_attach_to_their_source() {
        assert_eq!(
            kinds("v.rg"),
            vec![name("v"), TokenKind::Dot, TokenKind::Function(Function::Rg)]
        );
        assert_eq!(
            kinds("tint[0]"),
            vec![
                name("tint"),
                TokenKind::LeftBracket,
                number("0", false, None),
                TokenKind::RightBracket,
            ]
        );
        assert_eq!(failure("v .rg"), (2..3, ParseFailure::DetachedPostfix));
        assert_eq!(failure("v. rg"), (1..2, ParseFailure::MemberExpected));
        assert_eq!(failure("tint [0]"), (5..6, ParseFailure::DetachedPostfix));
        assert_eq!(failure("v."), (1..2, ParseFailure::MemberExpected));
        assert_eq!(failure(".r"), (0..1, ParseFailure::DetachedPostfix));
    }

    #[test]
    fn a_dot_before_a_digit_is_a_number_anywhere() {
        assert_eq!(
            kinds("a - .5"),
            vec![name("a"), TokenKind::Minus, number(".5", true, None)]
        );
        assert_eq!(
            kinds("a*.5"),
            vec![name("a"), TokenKind::Star, number(".5", true, None)]
        );
    }

    #[test]
    fn backticks_quote_what_a_bare_name_cannot_hold() {
        assert_eq!(
            kinds("`foo bar`"),
            vec![TokenKind::QuotedName("foo bar".to_owned())]
        );
        assert_eq!(
            kinds("`1st`"),
            vec![TokenKind::QuotedName("1st".to_owned())]
        );
        assert_eq!(
            kinds("`min`"),
            vec![TokenKind::QuotedName("min".to_owned())]
        );
        assert_eq!(
            kinds("`a\"b`"),
            vec![TokenKind::QuotedName("a\"b".to_owned())]
        );
        assert_eq!(kinds("foo bar"), vec![name("foo"), name("bar")]);
    }

    #[test]
    fn an_unterminated_or_empty_quoted_name_errors() {
        assert_eq!(failure("`foo"), (0..4, ParseFailure::UnterminatedName));
        assert_eq!(failure("``"), (0..2, ParseFailure::EmptyName));
    }

    #[test]
    fn string_literals_take_any_character_but_the_quote() {
        assert_eq!(
            kinds("\"MASK\""),
            vec![TokenKind::StringLiteral("MASK".to_owned())]
        );
        assert_eq!(
            kinds("\"a b `c` \\ d\""),
            vec![TokenKind::StringLiteral("a b `c` \\ d".to_owned())]
        );
        assert_eq!(kinds("\"\""), vec![TokenKind::StringLiteral(String::new())]);
        assert_eq!(failure("\"open"), (0..5, ParseFailure::UnterminatedString));
    }

    #[test]
    fn reserved_names_lex_as_keywords() {
        assert_eq!(
            kinds("min max default swatch"),
            vec![
                TokenKind::Function(Function::Min),
                TokenKind::Function(Function::Max),
                TokenKind::Function(Function::Default),
                TokenKind::Function(Function::Swatch),
            ]
        );
        assert_eq!(kinds("true false"), vec![TokenKind::True, TokenKind::False]);
        assert_eq!(kinds("minimum True"), vec![name("minimum"), name("True")]);
    }

    #[test]
    fn identifiers_start_with_a_letter_or_underscore() {
        assert_eq!(
            kinds("_x x1 baseColorFactor"),
            vec![name("_x"), name("x1"), name("baseColorFactor")]
        );
    }

    #[test]
    fn tokens_carry_their_byte_ranges() {
        assert_eq!(ranges("ab = 1.5;"), vec![0..2, 3..4, 5..8, 8..9]);
        assert_eq!(ranges("`a b`.rg"), vec![0..5, 5..6, 6..8]);
    }
}
