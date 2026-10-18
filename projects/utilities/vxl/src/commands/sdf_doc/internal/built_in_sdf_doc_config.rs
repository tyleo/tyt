use crate::commands::SdfDocConfig;
use ty_preferences::{DeserializePrefs, JsoncCodec};

/// The `sdfDoc` section of vxl's built-in layer, which loads before every
/// `.vxlconfig` layer.
pub fn built_in_sdf_doc_config() -> SdfDocConfig {
    JsoncCodec
        .deserialize_prefs(include_bytes!("built_in.vxlconfig"), "sdfDoc")
        .expect("the built-in layer reads as a `.vxlconfig`")
        .expect("the built-in layer holds an `sdfDoc` section")
}

#[cfg(test)]
mod tests {
    use crate::commands::built_in_sdf_doc_config;
    use sdfj::SdfjFile;
    use sdfj_sdfcore::from_sdfj_file;
    use serde_json::{Value, json};
    use voxsmith::{
        operations::sdf_doc::{SdfSampleOptions, sample},
        utilities::{GridResolution, VoxelFrame},
    };

    /// The built-in `materials` library's document.
    fn materials() -> SdfjFile {
        built_in_sdf_doc_config().build.libraries.embedded["materials"]
            .document
            .clone()
    }

    #[test]
    fn the_built_in_layer_defines_only_the_materials_library() {
        let config = built_in_sdf_doc_config();

        let libraries = config.build.libraries;
        assert!(libraries.files.is_empty());
        assert_eq!(libraries.embedded.keys().collect::<Vec<_>>(), ["materials"]);
        assert!(config.build.profiles.is_empty());
    }

    #[test]
    fn the_materials_library_names_every_material_the_modeling_api_lists() {
        let materials = materials();

        let names: Vec<_> = materials
            .names
            .materials
            .entries()
            .iter()
            .map(|entry| (entry.key.as_str(), entry.value))
            .collect();

        let listed = [
            "amethyst",
            "bark",
            "birch",
            "bone",
            "brass",
            "brick",
            "bronze",
            "clay",
            "copper",
            "diamond",
            "dirt",
            "ebony",
            "ember",
            "emerald",
            "flame",
            "glow",
            "gold",
            "granite",
            "grass",
            "ice",
            "iron",
            "lava",
            "leaf",
            "leather",
            "mahogany",
            "marble",
            "moss",
            "oak",
            "pine",
            "rope",
            "ruby",
            "sand",
            "sandstone",
            "sapphire",
            "silver",
            "slate",
            "snow",
            "steel",
            "stone",
            "straw",
            "topaz",
            "walnut",
            "water",
        ];
        assert_eq!(
            names,
            listed.into_iter().zip(0..).collect::<Vec<(&str, usize)>>()
        );
        assert_eq!(materials.materials.len(), listed.len());
    }

    #[test]
    fn every_material_and_its_default_shades_pass_the_material_checks() {
        let materials = materials();
        let count = materials.materials.len();

        let mut file: Value = serde_json::to_value(&materials).unwrap();
        let entries = file["materials"].as_array_mut().unwrap();
        entries.extend((0..count).flat_map(|shades| {
            (0..3).map(move |index| json!({ "kind": "shade", "shades": shades, "index": index }))
        }));
        file["shades"] = (0..count)
            .map(|base| json!({ "base": base, "count": 3 }))
            .collect();
        file["shapes3d"] = json!([{ "kind": "sphere", "center": [0, 0, 0], "radius": 1 }]);
        file["steps"] = json!([{ "kind": "add", "name": "ball", "shape": 0, "material": 0 }]);
        file["objects"] = json!([{ "name": "materials", "steps": [0] }]);
        file["nodes"] = json!([{ "name": "materials", "childObjects": [0], "childNodes": [] }]);
        file["rootNodes"] = json!([0]);
        let file: SdfjFile = serde_json::from_value(file).unwrap();

        let options = SdfSampleOptions {
            resolution: GridResolution::VoxelSize(0.5),
            frame: VoxelFrame::World,
            record_writes: false,
        };
        let sampling = sample(&from_sdfj_file(&file).unwrap(), &options).unwrap();

        assert_eq!(sampling.materials.len(), count * 4);
    }
}
