use mvox::MVoxMaterialType;

/// The `_type` token for a material shading model. Known variants map to their
/// documented tokens; an unmodeled variant keeps its stored string.
pub fn material_type_token(material_type: &MVoxMaterialType) -> String {
    match material_type {
        MVoxMaterialType::Diffuse => "_diffuse".to_owned(),
        MVoxMaterialType::Metal => "_metal".to_owned(),
        MVoxMaterialType::Glass => "_glass".to_owned(),
        MVoxMaterialType::Emit => "_emit".to_owned(),
        MVoxMaterialType::Other(token) => token.clone(),
    }
}
