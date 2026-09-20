use crate::{Dimension, Domain, Scalar};
use std::fmt::{Display, Formatter, Result as FmtResult};

/// The checking rule an expression broke.
#[derive(Clone, Debug, PartialEq)]
pub enum CheckFailure {
    /// The operands' dimensions do not pair under the operation's rule.
    DimensionMismatch {
        operation: String,
        found: Vec<Dimension>,
    },

    /// `any` or `all` took something other than a comparison written in
    /// place.
    FoldNeedsComparison { operation: String },

    /// An index was not unsigned.
    IndexScalar { found: Scalar },

    /// A literal does not fit the type its context fixed.
    LiteralOutOfRange { text: String, scalar: Scalar },

    /// The numeric operands took more than one type.
    MixedScalars {
        operation: String,
        found: Vec<Scalar>,
    },

    /// An operand that had to be a bool was not.
    NonBoolOperand { operation: String, found: Scalar },

    /// An operand that had to be a number was not.
    NonNumericOperand { operation: String, found: Scalar },

    /// A reduction's source sat at or below its destination.
    ReductionSource { operation: String, found: Domain },

    /// A plain operand met an operation that takes an array.
    RequiresArray { operation: String },

    /// The operand's dimension is not the one the operation takes.
    RequiresDimension {
        operation: String,
        expected: Dimension,
        found: Dimension,
    },

    /// The operation takes `f32` alone.
    RequiresF32 { operation: String, found: Scalar },

    /// An array met an operation that takes a plain value.
    RequiresPlain { operation: String, found: Domain },

    /// A climb named a domain below its operand's.
    StepDown { operation: String, found: Domain },

    /// Strings met a comparison other than `==` and `!=`.
    StringOrder { operation: String },

    /// A swizzle mixed the `rgba` and `xyzw` alphabets.
    SwizzleAlphabets { member: String },

    /// A swizzle held a character outside both alphabets.
    SwizzleCharacter { member: String, character: char },

    /// A swizzle named a component its source lacks.
    SwizzleComponent {
        member: String,
        character: char,
        found: Dimension,
    },

    /// A swizzle ran past four components.
    SwizzleLength { member: String },

    /// A name has no value in scope.
    UnknownName { name: String },

    /// A bare whole-number literal met nothing that fixes its type.
    UntypedLiteral,

    /// A comparison above vec1 sat outside `any` and `all`.
    WideComparison { operation: String, found: Dimension },
}

impl Display for CheckFailure {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        match self {
            CheckFailure::DimensionMismatch { operation, found } => write!(
                formatter,
                "`{operation}` does not pair the dimensions {}",
                list(found)
            ),

            CheckFailure::FoldNeedsComparison { operation } => {
                write!(
                    formatter,
                    "`{operation}` takes a comparison written in place"
                )
            }

            CheckFailure::IndexScalar { found } => {
                write!(formatter, "an index is unsigned, not {found}")
            }

            CheckFailure::LiteralOutOfRange { text, scalar } => {
                write!(formatter, "the literal `{text}` does not fit {scalar}")
            }

            CheckFailure::MixedScalars { operation, found } => write!(
                formatter,
                "`{operation}` takes one numeric type, not {}",
                list(found)
            ),

            CheckFailure::NonBoolOperand { operation, found } => {
                write!(formatter, "`{operation}` takes a bool, not {found}")
            }

            CheckFailure::NonNumericOperand { operation, found } => {
                write!(formatter, "`{operation}` takes a number, not {found}")
            }

            CheckFailure::ReductionSource { operation, found } => write!(
                formatter,
                "`{operation}` reduces an array above its destination, not a {found} value"
            ),

            CheckFailure::RequiresArray { operation } => {
                write!(formatter, "`{operation}` takes an array, not a plain value")
            }

            CheckFailure::RequiresDimension {
                operation,
                expected,
                found,
            } => write!(formatter, "`{operation}` takes {expected}, not {found}"),

            CheckFailure::RequiresF32 { operation, found } => {
                write!(formatter, "`{operation}` takes f32 alone, not {found}")
            }

            CheckFailure::RequiresPlain { operation, found } => write!(
                formatter,
                "`{operation}` takes a plain value, not a {found} array"
            ),

            CheckFailure::StepDown { operation, found } => write!(
                formatter,
                "`{operation}` climbs, and a {found} value sits above it"
            ),

            CheckFailure::StringOrder { operation } => write!(
                formatter,
                "`{operation}` orders numbers; strings take `==` and `!=` alone"
            ),

            CheckFailure::SwizzleAlphabets { member } => write!(
                formatter,
                "the swizzle `{member}` mixes the rgba and xyzw alphabets"
            ),

            CheckFailure::SwizzleCharacter { member, character } => write!(
                formatter,
                "the swizzle `{member}` holds `{character}`, outside rgba and xyzw"
            ),

            CheckFailure::SwizzleComponent {
                member,
                character,
                found,
            } => write!(
                formatter,
                "the swizzle `{member}` names `{character}`, which a {found} lacks"
            ),

            CheckFailure::SwizzleLength { member } => {
                write!(
                    formatter,
                    "the swizzle `{member}` runs past four components"
                )
            }

            CheckFailure::UnknownName { name } => {
                write!(formatter, "`{name}` has no value in scope")
            }

            CheckFailure::UntypedLiteral => write!(
                formatter,
                "a bare whole-number literal takes its type from context, and nothing here fixes one"
            ),

            CheckFailure::WideComparison { operation, found } => write!(
                formatter,
                "a {found} comparison names its fold: `{operation}` sits inside `any` or `all`"
            ),
        }
    }
}

/// Joins items with commas for a message.
fn list<T: Display>(items: &[T]) -> String {
    items
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}
