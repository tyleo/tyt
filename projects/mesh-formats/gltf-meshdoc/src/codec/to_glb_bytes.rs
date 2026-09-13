use crate::{
    EncodeBase64, GltfMeshMain, GltfWriteOptions, Result, codec::GltfBytes, frame_glb, to_gltf_file,
};

/// Writes a [`GltfMeshMain`] to the bytes of a `.glb` file through
/// `dependencies`, the binary container form of [`to_gltf_file`] and the
/// inverse of [`from_gltf_bytes`](crate::codec::from_gltf_bytes). The
/// geometry buffer is the `BIN` chunk. The images go where `options` says.
pub fn to_glb_bytes<D: EncodeBase64>(
    dependencies: &D,
    main: &GltfMeshMain,
    options: &GltfWriteOptions,
) -> Result<GltfBytes> {
    let file = to_gltf_file(dependencies, main, options)?;

    let json = file.root.to_vec().map_err(gltf::Error::Deserialize)?;

    Ok(GltfBytes {
        primary: frame_glb(&json, file.blob.as_deref())?,
        loose_files: file.loose_files,
    })
}

#[cfg(all(test, feature = "impl"))]
mod tests {
    use crate::{
        DependenciesImpl, GltfImageStorage, GltfWriteOptions,
        codec::{from_gltf_bytes, gltf_loose_uris, to_glb_bytes, to_gltf_bytes},
        snapshot, test_main, to_gltf_mesh_main,
    };
    use branded_id::U32Id;
    use std::collections::BTreeMap;

    /// Both containers round-trip a synthesized main: bytes to main to bytes
    /// to main, the two loaded mains equal, ext included.
    #[test]
    fn both_containers_round_trip() {
        let main = to_gltf_mesh_main(test_main());
        let options = GltfWriteOptions::default();

        // The document's files land beside either container.
        let glb = to_glb_bytes(&DependenciesImpl, &main, &options).unwrap();
        assert!(glb.primary.starts_with(b"glTF"));
        let names: Vec<&String> = glb.loose_files.keys().collect();
        assert_eq!(names, ["textures/detail.jpg", "values.json"]);

        let gltf = to_gltf_bytes(&DependenciesImpl, &main, &options).unwrap();
        assert!(gltf.primary.starts_with(b"{"));
        assert_eq!(gltf.loose_files, glb.loose_files);

        for bytes in [glb, gltf] {
            let loaded =
                from_gltf_bytes(&DependenciesImpl, &bytes.primary, bytes.loose_files.clone())
                    .unwrap();
            loaded.validate().unwrap();

            let again = to_glb_bytes(&DependenciesImpl, &loaded, &options).unwrap();
            let reloaded =
                from_gltf_bytes(&DependenciesImpl, &again.primary, again.loose_files).unwrap();

            assert_eq!(snapshot(&reloaded), snapshot(&loaded));
            assert_eq!(snapshot(&loaded).objects, snapshot(&main).objects);
        }
    }

    /// A JSON container with loose images lists them, and loads back from
    /// the files passed beside it.
    #[test]
    fn loose_files_are_listed_and_resolved() {
        let main = to_gltf_mesh_main(test_main());
        let options = GltfWriteOptions {
            images: GltfImageStorage::Loose,
        };

        let bytes = to_gltf_bytes(&DependenciesImpl, &main, &options).unwrap();
        let uris = gltf_loose_uris(&bytes.primary).unwrap();
        assert_eq!(uris, ["skin.png", "textures/detail.jpg", "values.json"]);

        let loaded = from_gltf_bytes(&DependenciesImpl, &bytes.primary, bytes.loose_files).unwrap();
        assert_eq!(loaded.file_count(), 3);
        assert_eq!(
            loaded.image_bytes(U32Id::from_u32(0)),
            main.image_bytes(U32Id::from_u32(0))
        );

        assert!(from_gltf_bytes(&DependenciesImpl, &bytes.primary, BTreeMap::new()).is_err());
    }

    /// A default main writes through its ext and loads back empty.
    #[test]
    fn round_trips_the_default_main() {
        let bytes = to_glb_bytes(
            &DependenciesImpl,
            &Default::default(),
            &GltfWriteOptions::default(),
        )
        .unwrap();

        let loaded = from_gltf_bytes(&DependenciesImpl, &bytes.primary, BTreeMap::new()).unwrap();

        assert_eq!(loaded.object_count(), 0);
    }
}
