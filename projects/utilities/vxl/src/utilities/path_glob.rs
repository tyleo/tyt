use std::str::FromStr;

/// One `--select` value: a glob over hierarchy paths, matched with the
/// project-standard globset rules shared with `hierarchy show`'s `pattern`. This
/// type carries the pattern with `**/` auto-prepended unless it already starts
/// with `**/`, so a bare pattern matches at any depth. The matching itself runs
/// through `Dependencies::match_glob`, which keeps the globset engine behind the
/// `impl` feature. The flag repeats; selecting a matched node's subtree is the
/// resolver's job.
#[derive(Clone, Debug)]
pub struct PathGlob {
    pattern: String,
}

impl PathGlob {
    /// The normalized glob pattern to hand to `Dependencies::match_glob`.
    pub fn pattern(&self) -> &str {
        &self.pattern
    }
}

impl FromStr for PathGlob {
    type Err = String;

    /// Parses a glob, auto-prepending `**/` unless it already starts with it so
    /// a bare pattern matches at any depth. An empty pattern is rejected.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.is_empty() {
            return Err("an empty glob matches nothing".to_string());
        }
        let pattern = if value.starts_with("**/") {
            value.to_string()
        } else {
            format!("**/{value}")
        };
        Ok(PathGlob { pattern })
    }
}

#[cfg(test)]
mod tests {
    use crate::PathGlob;

    fn glob(pattern: &str) -> PathGlob {
        pattern.parse().unwrap()
    }

    #[test]
    fn auto_prepends_unless_already_present() {
        assert_eq!(glob("door").pattern(), "**/door");
        assert_eq!(glob("a/b").pattern(), "**/a/b");
        assert_eq!(glob("**/door").pattern(), "**/door");
    }

    #[test]
    fn rejects_empty() {
        assert!("".parse::<PathGlob>().is_err());
    }
}
