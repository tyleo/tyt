use crate::{
    function::Function,
    lexer::NumberLiteral,
    parser::{BinaryOperator, ComparisonOperator, LogicalOperator, UnaryOperator},
};

/// A node of the untyped syntax tree.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum SyntaxNode {
    Binary {
        operator: BinaryOperator,
        left: Box<SyntaxNode>,
        right: Box<SyntaxNode>,
    },

    Bool(bool),

    Call {
        function: Function,
        arguments: Vec<SyntaxNode>,
    },

    Comparison {
        operator: ComparisonOperator,
        left: Box<SyntaxNode>,
        right: Box<SyntaxNode>,
    },

    Default {
        name: String,
        fallback: Box<SyntaxNode>,
    },

    Index {
        source: Box<SyntaxNode>,
        index: Box<SyntaxNode>,
    },

    Logical {
        operator: LogicalOperator,
        left: Box<SyntaxNode>,
        right: Box<SyntaxNode>,
    },

    Name(String),
    Number(NumberLiteral),
    StringLiteral(String),

    Swizzle {
        source: Box<SyntaxNode>,
        member: String,
    },

    Unary {
        operator: UnaryOperator,
        operand: Box<SyntaxNode>,
    },
}

#[cfg(test)]
impl SyntaxNode {
    /// The tree as a prefix form, `(+ a (* b c))`, for the tests.
    pub(crate) fn render(&self) -> String {
        use crate::lexer::NumberSuffix;

        match self {
            SyntaxNode::Binary {
                operator,
                left,
                right,
            } => format!("({} {} {})", operator, left.render(), right.render()),

            SyntaxNode::Bool(value) => value.to_string(),

            SyntaxNode::Call {
                function,
                arguments,
            } => {
                let arguments = arguments
                    .iter()
                    .map(SyntaxNode::render)
                    .collect::<Vec<_>>()
                    .join(" ");

                format!("({} {arguments})", function.name())
            }

            SyntaxNode::Comparison {
                operator,
                left,
                right,
            } => format!("({} {} {})", operator, left.render(), right.render()),

            SyntaxNode::Default { name, fallback } => {
                format!("(default `{name}` {})", fallback.render())
            }

            SyntaxNode::Index { source, index } => {
                format!("([] {} {})", source.render(), index.render())
            }

            SyntaxNode::Logical {
                operator,
                left,
                right,
            } => format!("({} {} {})", operator, left.render(), right.render()),

            SyntaxNode::Name(name) => format!("`{name}`"),

            SyntaxNode::Number(literal) => {
                let suffix = match literal.suffix {
                    None => "",
                    Some(NumberSuffix::F32) => "f32",
                    Some(NumberSuffix::U8) => "u8",
                    Some(NumberSuffix::U16) => "u16",
                    Some(NumberSuffix::U32) => "u32",
                };

                format!("{}{suffix}", literal.text)
            }

            SyntaxNode::StringLiteral(text) => format!("\"{text}\""),

            SyntaxNode::Swizzle { source, member } => {
                format!("(. {} {member})", source.render())
            }

            SyntaxNode::Unary { operator, operand } => {
                format!("({} {})", operator, operand.render())
            }
        }
    }
}
