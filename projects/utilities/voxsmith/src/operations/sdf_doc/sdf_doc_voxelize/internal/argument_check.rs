use std::{fmt::Display, result::Result as StdResult};
use ty_math::{TyVector2F64, TyVector3F64};

/// The share of a bound that a value may pass it by. Sums and differences of
/// meters round off within it.
const ROUNDING: f64 = 1e-9;

/// Checks one call's arguments. Each error reads `<call> <argument> must be
/// <expectation>, not <value>`.
#[derive(Clone, Copy, Debug)]
pub struct ArgumentCheck {
    call: &'static str,
}

impl ArgumentCheck {
    /// The check of the call `call`.
    pub fn new(call: &'static str) -> Self {
        Self { call }
    }

    /// The error that `argument` reads `value` where it has to meet
    /// `expectation`.
    pub fn fail(&self, argument: &str, expectation: &str, value: impl Display) -> String {
        format!(
            "{} {argument} must be {expectation}, not {value}",
            self.call
        )
    }

    /// Errors unless `holds`.
    pub fn expect(
        &self,
        holds: bool,
        argument: &str,
        expectation: &str,
        value: impl Display,
    ) -> StdResult<(), String> {
        if holds {
            Ok(())
        } else {
            Err(self.fail(argument, expectation, value))
        }
    }

    /// Errors unless `value` reads above zero.
    pub fn above_zero(&self, argument: &str, value: f64) -> StdResult<(), String> {
        self.expect(value > 0.0, argument, "above zero", value)
    }

    /// Errors unless every component of `value` reads above zero.
    pub fn each_above_zero(&self, argument: &str, value: TyVector3F64) -> StdResult<(), String> {
        self.expect(
            value.cmpgt(TyVector3F64::ZERO).all(),
            argument,
            "above zero on each axis",
            vector(value),
        )
    }

    /// Errors when a component of `value` reads zero.
    pub fn each_nonzero(&self, argument: &str, value: TyVector3F64) -> StdResult<(), String> {
        self.expect(
            value.cmpne(TyVector3F64::ZERO).all(),
            argument,
            "nonzero on each axis",
            vector(value),
        )
    }

    /// Errors unless `value` reads zero or more.
    pub fn at_least_zero(&self, argument: &str, value: f64) -> StdResult<(), String> {
        self.expect(value >= 0.0, argument, "zero or more", value)
    }

    /// Errors unless `value` reads at least the bound `least` that `expectation`
    /// describes. `value` may fall short of `least` by up to `ROUNDING` times
    /// `least`.
    pub fn at_least(
        &self,
        argument: &str,
        value: f64,
        least: f64,
        expectation: &str,
    ) -> StdResult<(), String> {
        self.expect(
            least - value <= least.abs() * ROUNDING,
            argument,
            expectation,
            value,
        )
    }

    /// Errors unless `value` reads at most the bound `most` that `expectation`
    /// describes. `value` may pass `most` by up to `ROUNDING` times `most`.
    pub fn at_most(
        &self,
        argument: &str,
        value: f64,
        most: f64,
        expectation: &str,
    ) -> StdResult<(), String> {
        self.expect(
            value - most <= most.abs() * ROUNDING,
            argument,
            expectation,
            value,
        )
    }

    /// Errors unless `value` is a whole number of at least `least`.
    pub fn whole_at_least(&self, argument: &str, value: f64, least: f64) -> StdResult<(), String> {
        self.expect(
            value.fract() == 0.0 && value >= least,
            argument,
            &format!("a whole number of at least {least}"),
            value,
        )
    }

    /// Errors unless `value` is a whole number above zero.
    pub fn whole_above_zero(&self, argument: &str, value: f64) -> StdResult<(), String> {
        self.expect(
            value.fract() == 0.0 && value > 0.0,
            argument,
            "a whole number above zero",
            value,
        )
    }

    /// Errors unless `seed` is a whole number in the 32-bit range.
    pub fn seed(&self, seed: f64) -> StdResult<(), String> {
        self.expect(
            seed.fract() == 0.0 && seed >= f64::from(i32::MIN) && seed <= f64::from(i32::MAX),
            "seed",
            "a whole number from -2^31 to 2^31 - 1",
            seed,
        )
    }

    /// Errors unless `to` passes `from`.
    pub fn range(&self, from: f64, to: f64) -> StdResult<(), String> {
        self.expect(to > from, "to", &format!("past from {from}"), to)
    }

    /// Errors unless `to` differs from `from`.
    pub fn ends_apart(&self, from: f64, to: f64) -> StdResult<(), String> {
        self.expect(to != from, "to", &format!("other than from {from}"), to)
    }

    /// Errors unless `max` sits apart from `min` on each axis.
    pub fn opposite_corners(&self, min: TyVector3F64, max: TyVector3F64) -> StdResult<(), String> {
        self.expect(
            max.cmpne(min).all(),
            "max",
            &format!("apart from min {} on each axis", vector(min)),
            vector(max),
        )
    }

    /// Errors unless `max` sits apart from `min` on each axis of the plane.
    pub fn opposite_corners2d(
        &self,
        min: TyVector2F64,
        max: TyVector2F64,
    ) -> StdResult<(), String> {
        self.expect(
            max.cmpne(min).all(),
            "max",
            &format!("apart from min [{}, {}] on each axis", min.x, min.y),
            format!("[{}, {}]", max.x, max.y),
        )
    }

    /// Errors unless `max` passes `min` on each axis of the plane.
    pub fn corners2d(&self, min: TyVector2F64, max: TyVector2F64) -> StdResult<(), String> {
        self.expect(
            max.cmpgt(min).all(),
            "max",
            &format!("past min [{}, {}] on each axis", min.x, min.y),
            format!("[{}, {}]", max.x, max.y),
        )
    }

    /// Errors unless `count` is at least `least`.
    pub fn count(&self, argument: &str, count: usize, least: usize) -> StdResult<(), String> {
        self.expect(
            count >= least,
            argument,
            &format!("a list of at least {least}"),
            format!("a list of {count}"),
        )
    }
}

fn vector(value: TyVector3F64) -> String {
    format!("[{}, {}, {}]", value.x, value.y, value.z)
}
