use crate::Result;
use globset::GlobBuilder;
use std::io::{Error as IOError, ErrorKind};

/// Matches `pattern` against each candidate hierarchy path with globset, the
/// project's standard glob engine, built with path separators literal. Returns
/// one boolean per candidate in order; an invalid glob is an error.
pub fn match_glob(pattern: &str, candidates: &[&str]) -> Result<Vec<bool>> {
    let matcher = GlobBuilder::new(pattern)
        .literal_separator(true)
        .build()
        .map_err(|e| IOError::new(ErrorKind::InvalidInput, e))?
        .compile_matcher();
    Ok(candidates.iter().map(|c| matcher.is_match(c)).collect())
}

#[cfg(test)]
mod tests {
    use crate::implementation::match_glob;

    #[test]
    fn full_match_not_substring() {
        let candidates = ["door", "backdoor", "house/door", "a/b/door"];
        assert_eq!(
            match_glob("**/door", &candidates).unwrap(),
            vec![true, false, true, true]
        );
        assert_eq!(
            match_glob("**/*door*", &candidates).unwrap(),
            vec![true, true, true, true]
        );
    }

    #[test]
    fn star_and_question_stay_in_segment() {
        let candidates = ["abc", "a/c"];
        assert_eq!(
            match_glob("**/a*c", &candidates).unwrap(),
            vec![true, false]
        );
        assert_eq!(
            match_glob("**/a?c", &candidates).unwrap(),
            vec![true, false]
        );
    }

    #[test]
    fn double_star_crosses_slash() {
        let candidates = ["a", "a/b", "a/b/c"];
        assert_eq!(
            match_glob("**/a/**", &candidates).unwrap(),
            vec![false, true, true]
        );
    }
}
