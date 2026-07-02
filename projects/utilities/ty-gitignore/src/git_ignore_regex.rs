use crate::{Result, UnsignedGitIgnoreRegex};

/// A gitignore glob pattern with a sign. A leading `!` makes the sign negative
/// to exclude, else it is positive to include.
#[derive(Clone, Debug)]
pub struct GitIgnoreRegex {
    /// The compiled pattern without its sign.
    unsigned: UnsignedGitIgnoreRegex,

    /// True to include, false to exclude.
    sign: bool,
}

impl GitIgnoreRegex {
    /// Compiles a gitignore glob into a signed pattern. A leading `!` negates.
    pub fn from_span(pattern: &str) -> Result<Self> {
        let (body, sign) = match pattern.strip_prefix('!') {
            Some(rest) => (rest, false),
            None => (pattern, true),
        };

        Ok(Self {
            unsigned: UnsignedGitIgnoreRegex::from_span(body)?,
            sign,
        })
    }

    /// Compiles a gitignore glob, returning `None` for an empty line or a
    /// comment line beginning with `#`.
    pub fn from_span_ignore_inert(pattern: &str) -> Result<Option<Self>> {
        if pattern.is_empty() || pattern.starts_with('#') {
            return Ok(None);
        }

        Ok(Some(Self::from_span(pattern)?))
    }

    /// Compiles each pattern in order.
    pub fn from_spans<S: AsRef<str>>(patterns: &[S]) -> Result<Vec<Self>> {
        patterns
            .iter()
            .map(|pattern| Self::from_span(pattern.as_ref()))
            .collect()
    }

    /// Compiles each pattern in order, skipping empty and comment lines.
    pub fn from_spans_ignore_inert<S: AsRef<str>>(patterns: &[S]) -> Result<Vec<Self>> {
        let mut out = Vec::new();

        for pattern in patterns {
            if let Some(regex) = Self::from_span_ignore_inert(pattern.as_ref())? {
                out.push(regex);
            }
        }

        Ok(out)
    }

    /// The compiled pattern without its sign.
    pub fn unsigned(&self) -> &UnsignedGitIgnoreRegex {
        &self.unsigned
    }

    /// True to include, false to exclude.
    pub fn sign(&self) -> bool {
        self.sign
    }

    /// The sign when the pattern matches `path`, or `None` when it does not.
    pub fn is_match(&self, path: &str) -> Option<bool> {
        if self.unsigned.is_match(path) {
            Some(self.sign)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::GitIgnoreRegex;

    #[test]
    fn a_plain_pattern_has_a_positive_sign() {
        let regex = GitIgnoreRegex::from_span("door").unwrap();

        assert!(regex.sign());
        assert_eq!(regex.is_match("house/door"), Some(true));
        assert_eq!(regex.is_match("wall"), None);
    }

    #[test]
    fn a_bang_pattern_has_a_negative_sign() {
        let regex = GitIgnoreRegex::from_span("!door").unwrap();

        assert!(!regex.sign());
        assert_eq!(regex.is_match("house/door"), Some(false));
        assert_eq!(regex.is_match("wall"), None);
    }

    #[test]
    fn the_inert_check_runs_before_the_bang_is_stripped() {
        // A `#` line is inert, but `!#foo` is a negated pattern for a literal
        // `#foo`.
        assert!(
            GitIgnoreRegex::from_span_ignore_inert("# comment")
                .unwrap()
                .is_none()
        );

        let regex = GitIgnoreRegex::from_span_ignore_inert("!#foo")
            .unwrap()
            .unwrap();

        assert!(!regex.sign());
        assert_eq!(regex.is_match("#foo"), Some(false));
    }

    #[test]
    fn from_spans_keeps_every_line_including_inert_ones() {
        let regexes = GitIgnoreRegex::from_spans(&["a", "!b"]).unwrap();

        assert_eq!(regexes.len(), 2);
        assert!(regexes[0].sign());
        assert!(!regexes[1].sign());
    }
}
