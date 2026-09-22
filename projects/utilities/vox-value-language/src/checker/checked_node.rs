use crate::{Type, checker::CheckedKind};

/// A node of the checked tree, its type settled.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CheckedNode {
    /// What the node computes.
    pub(crate) kind: CheckedKind,

    /// The type the node computes.
    pub(crate) output: Type,
}

#[cfg(test)]
impl CheckedNode {
    /// The tree as a prefix form with every literal typed, for the tests.
    pub(crate) fn render(&self) -> String {
        use crate::checker::{Fold, NumberValue, Rounding};

        match &self.kind {
            CheckedKind::Binary {
                operator,
                left,
                right,
            } => format!("({operator} {} {})", left.render(), right.render()),

            CheckedKind::Bool(value) => value.to_string(),

            CheckedKind::Call {
                function,
                arguments,
            } => {
                let arguments = arguments
                    .iter()
                    .map(CheckedNode::render)
                    .collect::<Vec<_>>()
                    .join(" ");

                format!("({} {arguments})", function.to_function().name())
            }

            CheckedKind::Climb { target, operand } => {
                format!("({target} {})", operand.render())
            }

            CheckedKind::Comparison {
                operator,
                left,
                right,
            } => format!("({operator} {} {})", left.render(), right.render()),

            CheckedKind::Convert {
                target,
                rounding,
                operand,
            } => {
                let mode = match rounding {
                    None => "",
                    Some(Rounding::Ceil) => "ceil_",
                    Some(Rounding::Floor) => "floor_",
                    Some(Rounding::Round) => "round_",
                };

                format!("({mode}{target} {})", operand.render())
            }

            CheckedKind::Default {
                name,
                bound,
                fallback,
            } => {
                let marker = if bound.is_some() { "" } else { "?" };

                format!("(default `{name}`{marker} {})", fallback.render())
            }

            CheckedKind::Fold { fold, operand } => {
                let fold = match fold {
                    Fold::All => "all",
                    Fold::Any => "any",
                };

                format!("({fold} {})", operand.render())
            }

            CheckedKind::Index { source, index } => {
                format!("([] {} {})", source.render(), index.render())
            }

            CheckedKind::Logical {
                operator,
                left,
                right,
            } => format!("({operator} {} {})", left.render(), right.render()),

            CheckedKind::Mix {
                first,
                second,
                chooser,
            } => format!(
                "(mix {} {} {})",
                first.render(),
                second.render(),
                chooser.render()
            ),

            CheckedKind::Name(name) => format!("`{name}`"),

            CheckedKind::Number(value) => match value {
                NumberValue::F32(value) => format!("{value}f32"),
                NumberValue::U8(value) => format!("{value}u8"),
                NumberValue::U16(value) => format!("{value}u16"),
                NumberValue::U32(value) => format!("{value}u32"),
            },

            CheckedKind::Reduce {
                reduction,
                target,
                operand,
            } => format!("({} {})", reduction.name(*target), operand.render()),

            CheckedKind::StringLiteral(text) => format!("\"{text}\""),

            CheckedKind::Swizzle { source, components } => {
                let components = components.iter().map(usize::to_string).collect::<String>();

                format!("(. {} {components})", source.render())
            }

            CheckedKind::Unary { operator, operand } => {
                format!("({operator} {})", operand.render())
            }
        }
    }
}
