use crate::{
    BinaryOperator, ComparisonOperator, Function, LogicalOperator, NumberLiteral, UnaryOperator,
};
use std::collections::BTreeSet;

/// A node of the untyped syntax tree.
#[derive(Clone, Debug, PartialEq)]
pub enum SyntaxNode {
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

impl SyntaxNode {
    /// Adds to `free` each name the node reads that `bound` lacks. A
    /// `default` reads its name too.
    pub(crate) fn gather_free_names(&self, bound: &BTreeSet<String>, free: &mut BTreeSet<String>) {
        let mut name = |name: &String| {
            if !bound.contains(name) {
                free.insert(name.clone());
            }
        };

        match self {
            SyntaxNode::Binary { left, right, .. }
            | SyntaxNode::Comparison { left, right, .. }
            | SyntaxNode::Logical { left, right, .. } => {
                left.gather_free_names(bound, free);
                right.gather_free_names(bound, free);
            }

            SyntaxNode::Bool(_) | SyntaxNode::Number(_) | SyntaxNode::StringLiteral(_) => {}

            SyntaxNode::Call { arguments, .. } => {
                for argument in arguments {
                    argument.gather_free_names(bound, free);
                }
            }

            SyntaxNode::Default {
                name: read,
                fallback,
            } => {
                name(read);
                fallback.gather_free_names(bound, free);
            }

            SyntaxNode::Index { source, index } => {
                source.gather_free_names(bound, free);
                index.gather_free_names(bound, free);
            }

            SyntaxNode::Name(read) => name(read),

            SyntaxNode::Swizzle { source, .. } => source.gather_free_names(bound, free),

            SyntaxNode::Unary { operand, .. } => operand.gather_free_names(bound, free),
        }
    }
}
