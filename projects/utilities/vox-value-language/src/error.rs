use crate::{CheckFailure, Dimension, Domain, EvalFailure, ParseFailure, Scalar, Type};
use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
    ops::Range,
};

/// An error from the language pipeline.
#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    /// An expression broke a checking rule, inside the binding or on its
    /// own.
    Check {
        binding: Option<String>,
        failure: CheckFailure,
    },

    /// A value's component count disagrees with its domain and dimension.
    ComponentCount {
        domain: Domain,
        dimension: Dimension,
        components: usize,
    },

    /// An environment array's entry count disagrees with its domain's
    /// length.
    EntryCount {
        name: String,
        expected: usize,
        found: usize,
    },

    /// A value could not be computed, inside the binding or on its own.
    Eval {
        binding: Option<String>,
        failure: EvalFailure,
    },

    /// A face lists no voxel pieces.
    FacePieces { face: usize },

    /// The type environment holds a name the value environment lacks.
    MissingValue { name: String },

    /// An environment `f32` array holds a NaN or infinite component.
    NonFiniteInput { name: String },

    /// A bool or string was given a width above vec1.
    NonNumericWidth {
        scalar: Scalar,
        dimension: Dimension,
    },

    /// The text broke a token or grammar rule over the named byte range.
    Parse {
        range: Range<usize>,
        failure: ParseFailure,
    },

    /// A face's piece points past the voxel table.
    PieceVoxel { face: usize, voxel: u32 },

    /// The value environment holds a name the type environment lacks.
    UnexpectedValue { name: String },

    /// A value's type disagrees with the type the program was checked
    /// against.
    ValueType {
        name: String,
        expected: Type,
        found: Type,
    },
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        match self {
            Error::Check {
                binding: Some(binding),
                failure,
            } => write!(formatter, "in `{binding}`: {failure}"),

            Error::Check {
                binding: None,
                failure,
            } => write!(formatter, "{failure}"),

            Error::ComponentCount {
                domain,
                dimension,
                components,
            } => write!(
                formatter,
                "{components} components do not fill a {domain} {dimension} value"
            ),

            Error::EntryCount {
                name,
                expected,
                found,
            } => write!(
                formatter,
                "`{name}` holds {found} entries where its domain has {expected}"
            ),

            Error::Eval {
                binding: Some(binding),
                failure,
            } => write!(formatter, "in `{binding}`: {failure}"),

            Error::Eval {
                binding: None,
                failure,
            } => write!(formatter, "{failure}"),

            Error::FacePieces { face } => write!(formatter, "face {face} lists no voxel pieces"),

            Error::MissingValue { name } => {
                write!(formatter, "`{name}` has a type but no value")
            }

            Error::NonFiniteInput { name } => {
                write!(formatter, "`{name}` holds a non-finite component")
            }

            Error::NonNumericWidth { scalar, dimension } => {
                write!(formatter, "a {scalar} is vec1 alone, not {dimension}")
            }

            Error::Parse { range, failure } => {
                write!(
                    formatter,
                    "at bytes {}..{}: {failure}",
                    range.start, range.end
                )
            }

            Error::PieceVoxel { face, voxel } => {
                write!(
                    formatter,
                    "face {face} names voxel {voxel} past the voxel table"
                )
            }

            Error::UnexpectedValue { name } => {
                write!(formatter, "`{name}` has a value but no type")
            }

            Error::ValueType {
                name,
                expected,
                found,
            } => write!(
                formatter,
                "`{name}` was checked as {expected} and holds {found}"
            ),
        }
    }
}

impl StdError for Error {}
