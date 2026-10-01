use crate::{NumberSuffix, SyntaxNode};

impl SyntaxNode {
    /// The tree as a prefix form, `(+ a (* b c))`, for the tests.
    pub(crate) fn render(&self) -> String {
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
                    Some(NumberSuffix::F64) => "f64",
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
