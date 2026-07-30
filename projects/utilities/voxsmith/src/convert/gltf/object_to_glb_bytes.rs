use crate::{Error, MeshMethod, Result, object_to_gltf_document};
use gltf::binary::{Glb, Header};
use std::borrow::Cow;
use voxcore::VoxObject;

/// Meshes `object` with `method` and writes it as a binary glTF (`.glb`): a
/// self-contained file whose geometry buffer is the GLB BIN chunk.
///
/// `scale` is the real-world edge length of one voxel in meters, applied as a
/// uniform scale to every vertex; glTF is meter-native, so the mesh opens at
/// that size. Geometry is Y-up, the inverse of the voxelizer's Z-up mapping. An
/// object with no live voxels writes a valid glTF with an empty scene.
pub fn object_to_glb_bytes(object: &VoxObject, method: MeshMethod, scale: f64) -> Result<Vec<u8>> {
    let (document, blob) = object_to_gltf_document(object, method, scale);

    let json = serde_json::to_vec(&document).map_err(Error::invalid)?;

    let glb = Glb {
        // `to_vec` recomputes the length, so the header length is a placeholder.
        header: Header {
            magic: *b"glTF",
            version: 2,
            length: 0,
        },
        json: Cow::Owned(json),
        bin: (!blob.is_empty()).then_some(Cow::Owned(blob)),
    };

    Ok(glb.to_vec()?)
}

#[cfg(test)]
mod tests {
    use crate::{MeshMethod, object_to_glb_bytes};
    use gltf::{import_slice, mesh::Mode};
    use ty_math::{TyBoundsF32, TyVector3F32, TyVector3U32};
    use voxcore::VoxObject;

    /// A 1x1x2 bar, tall on the voxel-json Z axis.
    fn bar() -> VoxObject {
        let mut object = VoxObject::new("bar".to_owned(), TyVector3U32::new(1, 1, 2)).unwrap();
        for z in 0..2 {
            let voxel_id = object.voxel_id(TyVector3U32::new(0, 0, z)).unwrap();
            object.retain_voxel(voxel_id, &[]).unwrap();
        }
        object
    }

    #[test]
    fn glb_round_trips_through_the_gltf_reader() {
        let bytes = object_to_glb_bytes(&bar(), MeshMethod::Greedy, 1.0).unwrap();
        let (document, buffers, _images) = import_slice(&bytes).unwrap();
        assert_eq!(document.meshes().count(), 1);

        let primitive = document
            .meshes()
            .next()
            .unwrap()
            .primitives()
            .next()
            .unwrap();
        assert_eq!(primitive.mode(), Mode::Triangles);
        let reader = primitive.reader(|b| Some(&buffers[b.index()][..]));
        let positions: Vec<[f32; 3]> = reader.read_positions().unwrap().collect();
        let indices: Vec<u32> = reader.read_indices().unwrap().into_u32().collect();

        // Greedy on a 1x1x2 bar: six faces, four vertices and six indices each.
        assert_eq!(positions.len(), 24);
        assert_eq!(indices.len(), 36);
    }

    #[test]
    fn z_up_bar_becomes_y_up_and_scales() {
        // The two-voxel Z height becomes four meters on glTF Y at scale 2.
        let bytes = object_to_glb_bytes(&bar(), MeshMethod::Greedy, 2.0).unwrap();
        let (document, buffers, _images) = import_slice(&bytes).unwrap();
        let primitive = document
            .meshes()
            .next()
            .unwrap()
            .primitives()
            .next()
            .unwrap();
        let reader = primitive.reader(|b| Some(&buffers[b.index()][..]));
        let positions: Vec<[f32; 3]> = reader.read_positions().unwrap().collect();

        let max = TyBoundsF32::from_points(positions.iter().map(|&p| TyVector3F32::from_array(p)))
            .expect("the bar has vertices")
            .max();
        assert!((max.x - 2.0).abs() < 1e-6, "max x {}", max.x);
        assert!((max.y - 4.0).abs() < 1e-6, "max y {}", max.y);
    }

    #[test]
    fn an_empty_object_writes_a_valid_empty_glb() {
        let empty = VoxObject::new("empty".to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();
        let bytes = object_to_glb_bytes(&empty, MeshMethod::Greedy, 1.0).unwrap();
        let (document, _buffers, _images) = import_slice(&bytes).unwrap();
        assert_eq!(document.meshes().count(), 0);
    }
}
