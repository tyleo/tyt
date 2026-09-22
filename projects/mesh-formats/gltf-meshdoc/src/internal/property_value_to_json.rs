use branded_id::U32Id;
use meshdoc::{BMeshTexture, MeshPropertyValue, MeshState};
use serde_json::{Value, json};
use std::collections::HashMap;

/// A property value as its `vxl.values` entry, the inverse of
/// [`property_value_from_json`](crate::property_value_from_json). A texture
/// writes as a texture info over `texture_indices`, and a file as the
/// relative URI of the state's file.
pub fn property_value_to_json(
    value: &MeshPropertyValue,
    texture_indices: &HashMap<U32Id<BMeshTexture>, u32>,
    state: &MeshState,
) -> Value {
    match value {
        MeshPropertyValue::Bool(value) => json!(value),
        MeshPropertyValue::Bools(values) => json!(values),
        MeshPropertyValue::BoolRows(rows) => json!(rows),
        MeshPropertyValue::Int(value) => json!(value),
        MeshPropertyValue::Ints(values) => json!(values),
        MeshPropertyValue::IntRows(rows) => json!(rows),
        MeshPropertyValue::Float(value) => json!(value),
        MeshPropertyValue::Floats(values) => json!(values),
        MeshPropertyValue::FloatRows(rows) => json!(rows),
        MeshPropertyValue::Text(value) => json!(value),
        MeshPropertyValue::Texts(values) => json!(values),
        MeshPropertyValue::TextRows(rows) => json!(rows),
        MeshPropertyValue::Texture(texture_ref) => json!({
            "index": texture_indices[&texture_ref.texture_id],
            "texCoord": texture_ref.uv_stream_id.to_u32(),
        }),
        MeshPropertyValue::File(file_id) => json!({
            "uri": state
                .file(*file_id)
                .expect("a property points at a live file in a valid state")
                .name,
        }),
    }
}
