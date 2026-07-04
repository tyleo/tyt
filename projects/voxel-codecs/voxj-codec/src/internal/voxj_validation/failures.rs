use crate::Check;

/// Accumulates validation failures, each tagged with the check it violates. In
/// fail-fast mode the driver stops once the first failure lands; in report mode
/// it gathers them all. A check consults [`go`](Self::go) before further work a
/// prior failure has already made moot.
pub struct Failures {
    fail_fast: bool,
    items: Vec<(Check, String)>,
}

impl Failures {
    /// An empty accumulator, fail-fast or report mode per `fail_fast`.
    pub fn new(fail_fast: bool) -> Self {
        Self {
            fail_fast,
            items: Vec::new(),
        }
    }

    /// Records one failure.
    pub fn report(&mut self, check: Check, message: String) {
        self.items.push((check, message));
    }

    /// Whether to keep scanning: false once a fail-fast run holds its first
    /// failure.
    pub fn go(&self) -> bool {
        !self.fail_fast || self.items.is_empty()
    }

    /// The failures gathered, in discovery order.
    pub fn into_items(self) -> Vec<(Check, String)> {
        self.items
    }
}
