use crate::{
    Domain, Scalar,
    checker::{CheckedNode, ElementwiseFunction, Fold, NumberValue, Reduction, Rounding},
    parser::{BinaryOperator, ComparisonOperator, LogicalOperator, UnaryOperator},
};

/// What a checked node computes.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum CheckedKind {
    Binary {
        operator: BinaryOperator,
        left: Box<CheckedNode>,
        right: Box<CheckedNode>,
    },

    Bool(bool),

    Call {
        function: ElementwiseFunction,
        arguments: Vec<CheckedNode>,
    },

    Climb {
        target: Domain,
        operand: Box<CheckedNode>,
    },

    Comparison {
        operator: ComparisonOperator,
        left: Box<CheckedNode>,
        right: Box<CheckedNode>,
    },

    Convert {
        target: Scalar,
        rounding: Option<Rounding>,
        operand: Box<CheckedNode>,
    },

    Default {
        name: String,
        bound: bool,
        fallback: Box<CheckedNode>,
    },

    Fold {
        fold: Fold,
        operator: ComparisonOperator,
        left: Box<CheckedNode>,
        right: Box<CheckedNode>,
    },

    Index {
        source: Box<CheckedNode>,
        index: Box<CheckedNode>,
    },

    Logical {
        operator: LogicalOperator,
        left: Box<CheckedNode>,
        right: Box<CheckedNode>,
    },

    Mix {
        first: Box<CheckedNode>,
        second: Box<CheckedNode>,
        chooser: Box<CheckedNode>,
    },

    Name(String),
    Number(NumberValue),

    Reduce {
        reduction: Reduction,
        target: Domain,
        operand: Box<CheckedNode>,
    },

    StringLiteral(String),

    Swizzle {
        source: Box<CheckedNode>,
        components: Vec<usize>,
    },

    Unary {
        operator: UnaryOperator,
        operand: Box<CheckedNode>,
    },
}
