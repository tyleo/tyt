/// A field value made of numbers.
pub trait FindNonFinite {
    /// The value's first NaN or infinity.
    fn find_non_finite(&self) -> Option<f64>;
}

impl FindNonFinite for f64 {
    fn find_non_finite(&self) -> Option<f64> {
        (!self.is_finite()).then_some(*self)
    }
}

impl<T: FindNonFinite, const N: usize> FindNonFinite for [T; N] {
    fn find_non_finite(&self) -> Option<f64> {
        self.iter().find_map(FindNonFinite::find_non_finite)
    }
}

impl<T: FindNonFinite> FindNonFinite for Option<T> {
    fn find_non_finite(&self) -> Option<f64> {
        self.as_ref().and_then(FindNonFinite::find_non_finite)
    }
}

impl<T: FindNonFinite> FindNonFinite for Vec<T> {
    fn find_non_finite(&self) -> Option<f64> {
        self.iter().find_map(FindNonFinite::find_non_finite)
    }
}
