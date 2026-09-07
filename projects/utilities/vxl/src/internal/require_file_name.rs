/// Validates that `value` is a bare file name written beside the output file:
/// non-empty and free of any path separator. A path is a mistake rather than
/// something to silently strip to its file name.
pub(crate) fn require_file_name(value: &str) -> Result<String, String> {
    if value.is_empty() {
        return Err("a file name cannot be empty".to_string());
    }

    if value.contains('/') || value.contains('\\') {
        return Err(format!(
            "`{value}` is a file name written beside the output file, so it cannot contain \
             a path separator"
        ));
    }

    Ok(value.to_string())
}

#[cfg(test)]
mod tests {
    use crate::require_file_name;

    #[test]
    fn accepts_a_bare_file_name() {
        assert_eq!(require_file_name("skin.png").unwrap(), "skin.png");
    }

    #[test]
    fn rejects_an_empty_name() {
        assert!(require_file_name("").is_err());
    }

    #[test]
    fn rejects_a_name_with_a_path_separator() {
        assert!(require_file_name("textures/skin.png").is_err());
        assert!(require_file_name("textures\\skin.png").is_err());
        assert!(require_file_name("../skin.png").is_err());
    }
}
