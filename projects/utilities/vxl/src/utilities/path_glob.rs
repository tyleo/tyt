use std::str::FromStr;

/// One `--select` value: a hierarchy-path glob, `**/`-prepended unless it
/// already starts with `**/` so a bare pattern matches at any depth.
#[derive(Clone, Debug)]
pub struct PathGlob {
    pattern: String,
}

impl PathGlob {
    /// The normalized glob pattern.
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
