use crate::{GitIgnoreRegexKind, Result};
use globset::{GlobBuilder, GlobMatcher};

/// A gitignore glob pattern without a sign, compiled to a matcher.
///
/// A trailing `/` marks the pattern directory only. A leading or interior `/`
/// anchors it to the root, and so does a `**`, which already reaches any depth;
/// a pattern with no `/` and no `**` floats to match at any depth. `*` and `?`
/// match within a segment, `**` crosses segments, and `[...]` is a character
/// class, all through the `globset` crate.
#[derive(Clone, Debug)]
pub struct UnsignedGitIgnoreRegex {
    /// The compiled matcher, `None` when the pattern matches only the empty
    /// string.
    matcher: Option<GlobMatcher>,

    /// Whether the pattern matches directories only.
    kind: GitIgnoreRegexKind,
}

impl UnsignedGitIgnoreRegex {
    /// Compiles a gitignore glob into a matcher.
    pub fn from_span(pattern: &str) -> Result<Self> {
        // A trailing `/` marks the pattern directory only.
        let (body, kind) = match pattern.strip_suffix('/') {
            Some(rest) => (rest, GitIgnoreRegexKind::Directory),
            None => (pattern, GitIgnoreRegexKind::FileOrDirectory),
        };

        // A leading `/` anchors to the root.
        let (body, leading_slash) = match body.strip_prefix('/') {
            Some(rest) => (rest, true),
            None => (body, false),
        };

        // An empty or root pattern matches only the empty string.
        if body.is_empty() {
            return Ok(Self {
                matcher: None,
                kind,
            });
        }

        // With no slash and no `**`, float the pattern to match at any depth. A
        // slash or a `**` already reaches the intended depth.
        let float = !leading_slash && !body.contains('/') && !body.contains("**");

        let glob = if float {
            format!("**/{body}")
        } else {
            body.to_string()
        };

        let matcher = GlobBuilder::new(&glob)
            .literal_separator(true)
            .build()?
            .compile_matcher();

        Ok(Self {
            matcher: Some(matcher),
            kind,
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

    /// Whether this pattern matches directories only or files and directories.
    pub fn kind(&self) -> GitIgnoreRegexKind {
        self.kind
    }

    /// Whether `input` matches this pattern.
    pub fn is_match(&self, input: &str) -> bool {
        match &self.matcher {
            Some(matcher) => matcher.is_match(input),
            None => input.is_empty(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{GitIgnoreRegexKind, UnsignedGitIgnoreRegex};

    fn matches(pattern: &str, input: &str) -> bool {
        UnsignedGitIgnoreRegex::from_span(pattern)
            .unwrap()
            .is_match(input)
    }

    #[test]
    fn a_no_slash_pattern_floats_to_any_depth() {
        // A bare name matches that name at any depth but must be a whole segment.
        assert!(matches("door", "door"));
        assert!(matches("door", "house/door"));
        assert!(matches("door", "a/b/door"));
        assert!(!matches("door", "backdoor"));
        assert!(!matches("door", "door/knob"));
    }

    #[test]
    fn a_leading_slash_anchors_to_the_root() {
        assert!(matches("/door", "door"));
        assert!(!matches("/door", "house/door"));
        assert!(!matches("/door", "a/b/door"));
    }

    #[test]
    fn an_interior_slash_anchors_to_the_root() {
        assert!(matches("house/door", "house/door"));
        assert!(!matches("house/door", "town/house/door"));
    }

    #[test]
    fn star_and_question_stay_in_one_segment() {
        assert!(matches("a*c", "abc"));
        assert!(!matches("a*c", "a/c"));
        assert!(matches("a?c", "abc"));
        assert!(!matches("a?c", "a/c"));
        assert!(!matches("a?c", "ac"));
    }

    #[test]
    fn double_star_crosses_segments() {
        assert!(matches("a/**", "a/b"));
        assert!(matches("a/**", "a/b/c"));
        assert!(!matches("a/**", "a"));
        assert!(matches("**/x", "x"));
        assert!(matches("**/x", "a/x"));
        assert!(matches("**/x", "a/b/x"));
        assert!(!matches("**/x", "xa"));
    }

    #[test]
    fn a_character_class_matches_one_of_its_members() {
        // Classes are matched, not treated as literals.
        assert!(matches("f[oi]le", "file"));
        assert!(matches("f[oi]le", "fole"));
        assert!(!matches("f[oi]le", "fale"));
    }

    #[test]
    fn a_trailing_slash_sets_the_directory_kind() {
        let dir = UnsignedGitIgnoreRegex::from_span("logs/").unwrap();
        assert_eq!(dir.kind(), GitIgnoreRegexKind::Directory);

        let both = UnsignedGitIgnoreRegex::from_span("logs").unwrap();
        assert_eq!(both.kind(), GitIgnoreRegexKind::FileOrDirectory);

        // A directory pattern still matches by name; the kind gates the
        // aggregators.
        assert!(dir.is_match("a/logs"));
    }

    #[test]
    fn a_bare_double_star_matches_everything() {
        assert!(matches("**", "a"));
        assert!(matches("**", "a/b/c"));
    }

    #[test]
    fn an_empty_or_root_pattern_matches_only_the_empty_string() {
        let empty = UnsignedGitIgnoreRegex::from_span("").unwrap();
        assert!(empty.is_match(""));
        assert!(!empty.is_match("a"));

        let root = UnsignedGitIgnoreRegex::from_span("/").unwrap();
        assert_eq!(root.kind(), GitIgnoreRegexKind::Directory);
        assert!(root.is_match(""));
        assert!(!root.is_match("a"));
    }

    #[test]
    fn inert_lines_compile_to_none() {
        assert!(
            UnsignedGitIgnoreRegex::from_span_ignore_inert("")
                .unwrap()
                .is_none()
        );
        assert!(
            UnsignedGitIgnoreRegex::from_span_ignore_inert("# a comment")
                .unwrap()
                .is_none()
        );
        assert!(
            UnsignedGitIgnoreRegex::from_span_ignore_inert("door")
                .unwrap()
                .is_some()
        );
    }

    #[test]
    fn from_spans_ignore_inert_drops_blank_and_comment_lines() {
        let patterns = ["door", "", "# comment", "wall/"];
        let compiled = UnsignedGitIgnoreRegex::from_spans_ignore_inert(&patterns).unwrap();
        assert_eq!(compiled.len(), 2);
    }

    #[test]
    fn a_malformed_glob_is_an_error() {
        assert!(UnsignedGitIgnoreRegex::from_span("a[").is_err());
    }
}
