use crate::{
    Error, Result,
    operations::mesh::{MeshElement, MeshGeometry, MeshRecord, PrimitiveRecord, mesh_slices},
};
use branded_id::U32Id;
use meshdoc::{MeshHierarchyNode, MeshMain, MeshObject, MeshPrimitive, MeshTriangle};
use ty_math::TyVector3F64;
use voxcore::{VoxExt, VoxMain, VoxObject};

/// Meshes `object` under `record` into a document of one object placed by
/// one root node, both named as `object` is. Positions are in meters,
/// `record.voxel_size` per voxel, on the grid's Z-up axes. Errors if a layer
/// references a palette `main` does not hold, if the voxel size is not
/// positive, or if the record holds an element the run cannot mesh yet.
pub fn mesh<T: VoxExt>(
    main: &VoxMain<T>,
    object: &VoxObject,
    record: &MeshRecord,
) -> Result<MeshMain<()>> {
    if !(record.voxel_size.is_finite() && record.voxel_size > 0.0) {
        return Err(Error::mesh_record(
            MeshElement::VoxelSize,
            "must be finite and greater than 0",
        ));
    }

    main.effective_palette(object)?;

    let primitive_record = check_meshed(record)?;

    let geometry = mesh_slices(object, record.method, &|_| 0, false);

    let primitive = primitive_of(&geometry, record.voxel_size, primitive_record)?;

    place(object.name(), primitive)
}

/// The one primitive the run meshes, or the first element it cannot mesh
/// yet.
fn check_meshed(record: &MeshRecord) -> Result<&PrimitiveRecord> {
    let not_yet = |element: MeshElement| Error::mesh_record(element, "is not meshed yet");

    if let Some(binding) = record.computed_bindings.first() {
        return Err(not_yet(MeshElement::ComputedBinding {
            name: binding.name.clone(),
        }));
    }

    if !record.program.trim().is_empty() {
        return Err(not_yet(MeshElement::Program));
    }

    if !record.materials.is_empty() {
        return Err(not_yet(MeshElement::Materials));
    }

    if let Some(file) = record.files.first() {
        return Err(not_yet(MeshElement::File {
            file: file.file.clone(),
        }));
    }

    if let Some(extra) = record.mesh_extras.first() {
        return Err(not_yet(MeshElement::MeshExtra {
            name: extra.name.clone(),
        }));
    }

    let [primitive_record] = record.primitives.as_slice() else {
        if record.primitives.is_empty() {
            return Err(Error::mesh_record(
                MeshElement::Primitives,
                "hold no primitive, and the table holds at least one",
            ));
        }

        return Err(not_yet(MeshElement::PrimitiveSelect {
            primitive_id: U32Id::from_u32(1),
        }));
    };

    let primitive_id = U32Id::from_u32(0);

    if primitive_record.select != "true" {
        return Err(not_yet(MeshElement::PrimitiveSelect { primitive_id }));
    }

    if primitive_record.material_id.is_some() {
        return Err(not_yet(MeshElement::PrimitiveMaterial { primitive_id }));
    }

    if primitive_record.uv_streams.is_some() {
        return Err(not_yet(MeshElement::PrimitiveUvStreams { primitive_id }));
    }

    if let Some(attribute) = primitive_record.attributes.first() {
        return Err(not_yet(MeshElement::PrimitiveAttribute {
            primitive_id,
            name: attribute.name().to_owned(),
        }));
    }

    Ok(primitive_record)
}

/// The primitive of `geometry` scaled to `voxel_size` meters per voxel,
/// carrying what `primitive_record` asks of it.
fn primitive_of(
    geometry: &MeshGeometry,
    voxel_size: f64,
    primitive_record: &PrimitiveRecord,
) -> Result<MeshPrimitive> {
    let positions = geometry
        .positions
        .iter()
        .map(|position| TyVector3F64::from(position.as_dvec3() * voxel_size))
        .collect();

    let triangles = geometry
        .indices
        .chunks_exact(3)
        .map(|corners| MeshTriangle {
            vertex_ids: [
                U32Id::from_u32(corners[0]),
                U32Id::from_u32(corners[1]),
                U32Id::from_u32(corners[2]),
            ],
        })
        .collect();

    let mut primitive = MeshPrimitive::new(positions, triangles)?;

    if primitive_record.normal {
        primitive.set_normals(Some(
            geometry
                .normals
                .iter()
                .map(|normal| normal.as_dvec3())
                .collect(),
        ))?;
    }

    if let Some(name) = &primitive_record.name {
        primitive.set_name(name.clone());
    }

    Ok(primitive)
}

/// A document holding `primitive` in one object named `name`, placed by one
/// root node of the same name.
fn place(name: &str, primitive: MeshPrimitive) -> Result<MeshMain<()>> {
    let mut document = MeshMain::default();

    let mut object = MeshObject::new(name.to_owned());

    object.retain_primitive(primitive);

    let object_id = document.retain_object(object)?;

    let node_id = document.retain_hierarchy_node(MeshHierarchyNode {
        name: name.to_owned(),
        child_object_ids: vec![object_id],
        ..Default::default()
    })?;

    document.set_root_hierarchy_node_ids(vec![node_id])?;

    Ok(document)
}

#[cfg(test)]
mod tests {
    use crate::{
        Error,
        operations::mesh::{
            AttributeWrite, Computation, ComputedBinding, ExtraForm, ExtraSource, ExtraWrite,
            FileForm, FileWrite, MaterialRecord, MeshElement, MeshRecord, Method, PrimitiveRecord,
            TextureShape, Transfer, WrittenValue, mesh,
        },
    };
    use branded_id::{IdVec, U32Id};
    use meshdoc::MeshMain;
    use ty_math::{TyVector3F64, TyVector3U32};
    use voxcore::{VoxMain, VoxObject};

    /// A 2x1x1 bar of two live voxels with no layers.
    fn bar() -> VoxObject {
        let mut object = VoxObject::new("bar".to_owned(), TyVector3U32::new(2, 1, 1)).unwrap();
        for x in 0..2 {
            let voxel_id = object.voxel_id(TyVector3U32::new(x, 0, 0)).unwrap();
            object.retain_voxel(voxel_id, &[]).unwrap();
        }
        object
    }

    /// The implicit primitive, taking every face with no material.
    fn implicit_primitive() -> PrimitiveRecord {
        PrimitiveRecord {
            material_id: None,
            select: "true".to_owned(),
            name: None,
            normal: true,
            uv_streams: None,
            attributes: Vec::new(),
        }
    }

    /// A geometry-only record under `method` at one meter per voxel.
    fn record(method: Method) -> MeshRecord {
        MeshRecord {
            method,
            texture_shape: TextureShape::Pot,
            voxel_size: 1.0,
            computed_bindings: Vec::new(),
            program: String::new(),
            materials: IdVec::default(),
            primitives: IdVec::from_vec(vec![implicit_primitive()]),
            files: Vec::new(),
            mesh_extras: Vec::new(),
        }
    }

    /// The bar meshed under `record`.
    fn meshed(record: &MeshRecord) -> MeshMain<()> {
        let main: VoxMain = VoxMain::default();
        let document = mesh(&main, &bar(), record).unwrap();
        document.validate().unwrap();
        document
    }

    /// The element the bar fails to mesh on under `record`.
    fn failing_element(record: &MeshRecord) -> MeshElement {
        let main: VoxMain = VoxMain::default();
        match mesh(&main, &bar(), record).unwrap_err() {
            Error::MeshRecord { element, .. } => element,
            error => panic!("expected a record error, got {error}"),
        }
    }

    #[test]
    fn geometry_alone_is_one_bare_primitive_under_one_root() {
        let mut record = record(Method::Greedy);
        record.voxel_size = 2.0;

        let document = meshed(&record);

        assert_eq!(document.image_count(), 0);
        assert_eq!(document.material_count(), 0);
        assert_eq!(document.root_hierarchy_node_ids().len(), 1);

        let (_, object) = document.iter_objects().next().unwrap();
        assert_eq!(object.name(), "bar");
        let (_, primitive) = object.iter_primitives().next().unwrap();
        assert_eq!(primitive.material_id(), None);
        assert_eq!(primitive.uv_stream_count(), 0);
        assert_eq!(primitive.name(), "");
        assert!(primitive.normals().is_some());

        // Greedy merges the bar into a box: six faces of four vertices and
        // two triangles, scaled to two meters per voxel and kept Z-up.
        assert_eq!(primitive.vertex_count(), 24);
        assert_eq!(primitive.triangle_count(), 12);
        let max = primitive
            .positions()
            .iter()
            .fold(TyVector3F64::NEG_INFINITY, |max, position| {
                max.max(*position)
            });
        assert_eq!(max, TyVector3F64::new(4.0, 2.0, 2.0));
    }

    #[test]
    fn the_method_sets_the_face_count() {
        for (method, quads) in [
            (Method::Culled, 10),
            (Method::Greedy, 6),
            (Method::Naive, 12),
        ] {
            let document = meshed(&record(method));
            let (_, object) = document.iter_objects().next().unwrap();
            let (_, primitive) = object.iter_primitives().next().unwrap();
            assert_eq!(primitive.triangle_count(), quads * 2, "{method:?}");
        }
    }

    #[test]
    fn the_primitive_record_names_the_primitive_and_drops_the_normals() {
        let mut record = record(Method::Greedy);
        record.primitives[U32Id::from_u32(0).to_usize_id()].name = Some("body".to_owned());
        record.primitives[U32Id::from_u32(0).to_usize_id()].normal = false;

        let document = meshed(&record);
        let (_, object) = document.iter_objects().next().unwrap();
        let (_, primitive) = object.iter_primitives().next().unwrap();
        assert_eq!(primitive.name(), "body");
        assert!(primitive.normals().is_none());
    }

    #[test]
    fn a_non_positive_voxel_size_errors() {
        for voxel_size in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            let mut record = record(Method::Greedy);
            record.voxel_size = voxel_size;
            assert_eq!(failing_element(&record), MeshElement::VoxelSize);
        }
    }

    #[test]
    fn an_empty_primitive_table_errors() {
        let mut record = record(Method::Greedy);
        record.primitives = IdVec::default();
        assert_eq!(failing_element(&record), MeshElement::Primitives);
    }

    #[test]
    fn an_element_not_meshed_yet_errors_and_names_itself() {
        let written = WrittenValue {
            expression: "albedo".to_owned(),
            transfer: Transfer::Linear,
        };

        let mut with_binding = record(Method::Greedy);
        with_binding.computed_bindings.push(ComputedBinding {
            name: "ao".to_owned(),
            computation: Computation::Occlusion,
        });
        assert_eq!(
            failing_element(&with_binding),
            MeshElement::ComputedBinding {
                name: "ao".to_owned()
            }
        );

        let mut with_program = record(Method::Greedy);
        with_program.program = "albedo = baseColorFactor;".to_owned();
        assert_eq!(failing_element(&with_program), MeshElement::Program);

        let mut with_material = record(Method::Greedy);
        with_material.materials = IdVec::from_vec(vec![MaterialRecord::default()]);
        assert_eq!(failing_element(&with_material), MeshElement::Materials);

        let mut with_file = record(Method::Greedy);
        with_file.files.push(FileWrite {
            file: "bar-albedo.png".to_owned(),
            value: written.clone(),
            form: FileForm::Png,
        });
        assert_eq!(
            failing_element(&with_file),
            MeshElement::File {
                file: "bar-albedo.png".to_owned()
            }
        );

        let mut with_extra = record(Method::Greedy);
        with_extra.mesh_extras.push(ExtraWrite {
            name: "albedo".to_owned(),
            form: ExtraForm::Json,
            source: ExtraSource::Value(written.clone()),
        });
        assert_eq!(
            failing_element(&with_extra),
            MeshElement::MeshExtra {
                name: "albedo".to_owned()
            }
        );

        let primitive_id = U32Id::from_u32(0);

        let mut with_select = record(Method::Greedy);
        with_select.primitives[primitive_id.to_usize_id()].select = "solid".to_owned();
        assert_eq!(
            failing_element(&with_select),
            MeshElement::PrimitiveSelect { primitive_id }
        );

        let mut with_two = record(Method::Greedy);
        with_two.primitives.push(implicit_primitive());
        assert_eq!(
            failing_element(&with_two),
            MeshElement::PrimitiveSelect {
                primitive_id: U32Id::from_u32(1)
            }
        );

        let mut with_streams = record(Method::Greedy);
        with_streams.primitives[primitive_id.to_usize_id()].uv_streams = Some(Vec::new());
        assert_eq!(
            failing_element(&with_streams),
            MeshElement::PrimitiveUvStreams { primitive_id }
        );

        let mut with_attribute = record(Method::Greedy);
        with_attribute.primitives[primitive_id.to_usize_id()]
            .attributes
            .push(AttributeWrite::Custom {
                name: "_PALETTE".to_owned(),
                value: written,
            });
        assert_eq!(
            failing_element(&with_attribute),
            MeshElement::PrimitiveAttribute {
                primitive_id,
                name: "_PALETTE".to_owned()
            }
        );
    }

    #[test]
    fn a_layer_over_an_unknown_palette_errors() {
        let main: VoxMain = VoxMain::default();
        let mut object = bar();
        object.retain_layer(U32Id::from_u32(9), U32Id::from_u32(0));

        assert!(mesh(&main, &object, &record(Method::Greedy)).is_err());
    }
}
