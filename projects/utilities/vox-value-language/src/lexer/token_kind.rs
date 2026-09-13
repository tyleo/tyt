use crate::{function::Function, lexer::NumberLiteral};

/// What a token is.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum TokenKind {
    /// `&&`.
    And,

    /// `=`.
    Assign,

    /// `,`.
    Comma,

    /// The attached dot that begins a swizzle.
    Dot,

    /// `==`.
    Equal,

    /// `false`.
    False,

    /// A reserved function name.
    Function(Function),

    /// `>`.
    Greater,

    /// `>=`.
    GreaterEqual,

    /// A bare name.
    Identifier(String),

    /// The attached bracket that begins an index.
    LeftBracket,

    /// `(`.
    LeftParen,

    /// `<`.
    Less,

    /// `<=`.
    LessEqual,

    /// `-`.
    Minus,

    /// `!`.
    Not,

    /// `!=`.
    NotEqual,

    /// A number.
    Number(NumberLiteral),

    /// `||`.
    Or,

    /// `+`.
    Plus,

    /// A backtick-quoted name, without its backticks.
    QuotedName(String),

    /// `]`.
    RightBracket,

    /// `)`.
    RightParen,

    /// `;`.
    Semicolon,

    /// `/`.
    Slash,

    /// `*`.
    Star,

    /// A string literal, without its quotes.
    StringLiteral(String),

    /// `true`.
    True,

    /// `^`.
    Xor,
}

impl TokenKind {
    /// How an error names the token.
    pub(crate) fn describe(&self) -> String {
        match self {
            TokenKind::And => "`&&`".to_owned(),
            TokenKind::Assign => "`=`".to_owned(),
            TokenKind::Comma => "`,`".to_owned(),
            TokenKind::Dot => "`.`".to_owned(),
            TokenKind::Equal => "`==`".to_owned(),
            TokenKind::False => "`false`".to_owned(),
            TokenKind::Function(function) => format!("`{}`", function.name()),
            TokenKind::Greater => "`>`".to_owned(),
            TokenKind::GreaterEqual => "`>=`".to_owned(),
            TokenKind::Identifier(name) => format!("the name `{name}`"),
            TokenKind::LeftBracket => "`[`".to_owned(),
            TokenKind::LeftParen => "`(`".to_owned(),
            TokenKind::Less => "`<`".to_owned(),
            TokenKind::LessEqual => "`<=`".to_owned(),
            TokenKind::Minus => "`-`".to_owned(),
            TokenKind::Not => "`!`".to_owned(),
            TokenKind::NotEqual => "`!=`".to_owned(),
            TokenKind::Number(literal) => format!("the number `{}`", literal.text),
            TokenKind::Or => "`||`".to_owned(),
            TokenKind::Plus => "`+`".to_owned(),
            TokenKind::QuotedName(name) => format!("the name `{name}`"),
            TokenKind::RightBracket => "`]`".to_owned(),
            TokenKind::RightParen => "`)`".to_owned(),
            TokenKind::Semicolon => "`;`".to_owned(),
            TokenKind::Slash => "`/`".to_owned(),
            TokenKind::Star => "`*`".to_owned(),
            TokenKind::StringLiteral(text) => format!("the string `\"{text}\"`"),
            TokenKind::True => "`true`".to_owned(),
            TokenKind::Xor => "`^`".to_owned(),
        }
    }
}
