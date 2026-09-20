use crate::{Dimension, Domain, ParseFailure, Scalar};
use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
    ops::Range,
};

/// An error from the language pipeline.
#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    /// A value's component count disagrees with its domain and dimension.
    ComponentCount {
        domain: Domain,
        dimension: Dimension,
        components: usize,
    },

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
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        match self {
            Error::ComponentCount {
                domain,
                dimension,
                components,
            } => write!(
                formatter,
                "{components} components do not fill a {domain} {dimension} value"
            ),

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
        }
    }
}

impl StdError for Error {}
