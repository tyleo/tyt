use crate::{
    Result,
    commands::{parse_flag_value, push_uv_stream},
};
use voxsmith::operations::mesh::ArrayDomain;

/// The stream list `uvs`, which `origin` declares, each domain a known name
/// listed once.
pub(crate) fn parse_uv_list(origin: &str, uvs: &[String]) -> Result<Vec<ArrayDomain>> {
    let mut streams = Vec::new();

    for domain in uvs {
        let domain: ArrayDomain = parse_flag_value(&format!("{origin}'s uvs"), domain)?;

        push_uv_stream(&mut streams, domain, || format!("{origin}'s uvs lists"))?;
    }

    Ok(streams)
}

#[cfg(test)]
mod tests {
    use super::parse_uv_list;
    use voxsmith::operations::mesh::ArrayDomain;

    #[test]
    fn a_list_parses_in_order_and_repeats_no_domain() {
        assert_eq!(
            parse_uv_list("the profile `x`", &["swatch".to_owned(), "face".to_owned()]).unwrap(),
            [ArrayDomain::Swatch, ArrayDomain::Face]
        );
        assert!(parse_uv_list("the profile `x`", &["face".to_owned(), "face".to_owned()]).is_err());
        assert!(parse_uv_list("the profile `x`", &["edge".to_owned()]).is_err());
    }
}
