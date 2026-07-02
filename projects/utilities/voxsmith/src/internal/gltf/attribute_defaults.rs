/// The spec default for a recommended scalar attribute, so a map never fails on
/// a cell that omits it. A key with no standard default (a custom attribute)
/// returns `None`, which the bake reads as `0`.
pub(crate) fn default_scalar(key: &str) -> Option<f64> {
    match key {
        "metallic" => Some(0.0),
        "roughness" => Some(1.0),
        "occlusion" => Some(1.0),
        "emissive" => Some(0.0),
        "ior" => Some(1.5),
        "transmission" => Some(0.0),
        _ => None,
    }
}
