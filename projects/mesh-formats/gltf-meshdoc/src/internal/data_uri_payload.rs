/// The media type and base64 payload of a `data:<media type>;base64,<payload>`
/// URI, or `None` for any other URI.
pub fn data_uri_payload(uri: &str) -> Option<(&str, &str)> {
    let rest = uri.strip_prefix("data:")?;

    let (header, payload) = rest.split_once(',')?;

    let media_type = header.strip_suffix(";base64")?;

    Some((media_type, payload))
}

#[cfg(test)]
mod tests {
    use crate::data_uri_payload;

    #[test]
    fn splits_a_base64_data_uri_and_rejects_the_rest() {
        assert_eq!(
            data_uri_payload("data:image/png;base64,AAAA"),
            Some(("image/png", "AAAA"))
        );
        assert_eq!(data_uri_payload("data:text/plain,hello"), None);
        assert_eq!(data_uri_payload("image.png"), None);
    }
}
