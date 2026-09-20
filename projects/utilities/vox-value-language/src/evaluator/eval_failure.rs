use crate::{Domain, Scalar};
use std::fmt::{Display, Formatter, Result as FmtResult};

/// What went wrong while computing a value.
#[derive(Clone, Debug, PartialEq)]
pub enum EvalFailure {
    /// An unsigned difference fell below zero.
    BelowZero { left: u64, right: u64 },

    /// `rgbFromOklch` took a negative chroma.
    Chroma { chroma: f32 },

    /// A climb onto a merged face found its pieces disagreeing.
    ClimbDisagreement { target: Domain, entry: usize },

    /// An unsigned division or `mod` by zero.
    DivisionByZero,

    /// A min, max, or average reduction met a destination entry with
    /// nothing to reduce.
    EmptyDestination {
        operation: String,
        target: Domain,
        entry: usize,
    },

    /// An exact conversion took an `f32` with a fractional part.
    Fraction { value: f32, target: Scalar },

    /// `rgbFromOklch` took a hue outside `[0, 1]`.
    HueRange { hue: f32 },

    /// An index reached past the array's entries.
    IndexOutOfRange { index: u32, entries: usize },

    /// `f32` holds no exact image of the `u32`.
    Inexact { value: u32 },

    /// A clamp or ramp took bounds in the wrong order.
    InvertedBounds {
        operation: String,
        low: f32,
        high: f32,
    },

    /// An `f32` result came out NaN or infinite.
    NonFinite { operation: String },

    /// A converted value falls outside the target type's range.
    OutOfRange { value: f64, target: Scalar },

    /// An unsigned result exceeds its type.
    Overflow { operation: String },
}

impl Display for EvalFailure {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        match self {
            EvalFailure::BelowZero { left, right } => {
                write!(formatter, "{left} - {right} falls below zero")
            }

            EvalFailure::Chroma { chroma } => {
                write!(
                    formatter,
                    "`rgbFromOklch` takes a chroma of 0 or above, not {chroma}"
                )
            }

            EvalFailure::ClimbDisagreement { target, entry } => write!(
                formatter,
                "{target} entry {entry} merges pieces whose values disagree"
            ),

            EvalFailure::DivisionByZero => write!(formatter, "division by zero"),

            EvalFailure::EmptyDestination {
                operation,
                target,
                entry,
            } => write!(
                formatter,
                "`{operation}` reduces nothing into {target} entry {entry}"
            ),

            EvalFailure::Fraction { value, target } => {
                write!(
                    formatter,
                    "{value} has a fraction, and `{target}` takes whole values"
                )
            }

            EvalFailure::HueRange { hue } => {
                write!(formatter, "`rgbFromOklch` takes a hue in [0, 1], not {hue}")
            }

            EvalFailure::IndexOutOfRange { index, entries } => {
                write!(formatter, "index {index} reaches past {entries} entries")
            }

            EvalFailure::Inexact { value } => {
                write!(formatter, "f32 holds no exact image of {value}")
            }

            EvalFailure::InvertedBounds {
                operation,
                low,
                high,
            } => write!(
                formatter,
                "`{operation}` takes its bounds in order, not {low} above {high}"
            ),

            EvalFailure::NonFinite { operation } => {
                write!(formatter, "`{operation}` produced a non-finite value")
            }

            EvalFailure::OutOfRange { value, target } => {
                write!(formatter, "{value} falls outside `{target}`")
            }

            EvalFailure::Overflow { operation } => {
                write!(formatter, "`{operation}` overflowed its type")
            }
        }
    }
}
