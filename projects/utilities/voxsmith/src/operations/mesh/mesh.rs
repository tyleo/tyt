use crate::{
    Error, Result,
    operations::mesh::{
        ArrayDomain, Atlases, MergeRules, MeshElement, MeshGeometry, MeshRecord, Method,
        PrimitiveRecord, ProgramRun, Streams, Swatches, mesh_slices,
    },
};
use branded_id::U32Id;
use meshdoc::{MeshHierarchyNode, MeshMain, MeshObject, MeshPrimitive, MeshTriangle};
use ty_math::TyVector3F64;
use voxcore::{VoxExt, VoxMain, VoxObject};

/// Meshes `object` under `record` into a document of one object under one
/// root node, both named as `object` is. Positions are in meters on the
/// grid's Z-up axes. An error names the record element it rose from.
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

    let swatches = Swatches::resolve(main, object)?;

    let primitive_record = check_meshed(record)?;

    let geometry = if record.method == Method::Greedy {
        let culled = mesh_slices(object, Method::Culled, &|_| 0, &|_| true, false);

        let run = ProgramRun::over(object, &swatches, record, &culled)?;

        let streams = Streams::derive(record, &run.destinations)?;

        let rules = MergeRules::derive(object, record, &swatches, &culled, &run, &streams)?;

        mesh_slices(
            object,
            Method::Greedy,
            &|voxel_id| rules.voxel_class(voxel_id),
            &|span| rules.span_fits(span),
            false,
        )
    } else {
        mesh_slices(object, record.method, &|_| 0, &|_| true, false)
    };

    let run = ProgramRun::over(object, &swatches, record, &geometry)?;

    let streams = Streams::derive(record, &run.destinations)?;

    let atlases = Atlases::new(record.texture_shape, &swatches, &geometry);

    let primitive = primitive_of(
        &geometry,
        record.voxel_size,
        primitive_record,
        streams.primitive_list(U32Id::from_u32(0)),
        &atlases,
    )?;

    place(object.name(), primitive)
}

/// The one primitive the run meshes, or the first element it cannot mesh
/// yet.
fn check_meshed(record: &MeshRecord) -> Result<&PrimitiveRecord> {
    let not_yet = |element: MeshElement| Error::mesh_record(element, "is not meshed yet");

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
                "holds no primitive, and a run needs at least one",
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

    if let Some(attribute) = primitive_record.attributes.first() {
        return Err(not_yet(MeshElement::PrimitiveAttribute {
            primitive_id,
            name: attribute.name().to_owned(),
        }));
    }

    Ok(primitive_record)
}

/// The primitive of `geometry` at `voxel_size` meters per voxel.
fn primitive_of(
    geometry: &MeshGeometry,
    voxel_size: f64,
    primitive_record: &PrimitiveRecord,
    stream_list: &[ArrayDomain],
    atlases: &Atlases<'_>,
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

    for &domain in stream_list {
        primitive.push_uv_stream(atlases.uvs(domain)?)?;
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
            ArrayDomain, AttributeWrite, Computation, ComputedBinding, ExtraForm, ExtraSource,
            ExtraWrite, FileForm, FileWrite, MaterialRecord, MeshElement, MeshRecord, Method,
            PrimitiveRecord, TextureShape, Transfer, WrittenValue, mesh,
        },
    };
    use branded_id::{IdVec, U32Id};
    use meshdoc::MeshMain;
    use ty_math::{TyVector2F64, TyVector3F64, TyVector3U32};
    use voxcore::{
        VoxMain, VoxObject, VoxPalette, VoxValuePool,
        material::{BASE_COLOR, METALLIC},
    };

    /// A 2x1x1 bar of two live voxels with no layers.
    fn bar() -> VoxObject {
        let mut object = VoxObject::new("bar".to_owned(), TyVector3U32::new(2, 1, 1)).unwrap();
        for x in 0..2 {
            let voxel_id = object.voxel_id(TyVector3U32::new(x, 0, 0)).unwrap();
            object.retain_voxel(voxel_id, &[]).unwrap();
        }
        object
    }

    /// A main whose one palette carries `baseColor` and `metallic`, and the
    /// bar painted with its two materials.
    fn painted() -> (VoxMain, VoxObject) {
        let mut main: VoxMain = VoxMain::default();
        let colors = main.retain_value_pool(
            VoxValuePool::vec_4_float(vec![[1.0, 0.0, 0.0, 1.0], [0.0, 0.0, 1.0, 1.0]]).unwrap(),
        );
        let metals = main.retain_value_pool(VoxValuePool::float(vec![1.0, 0.0]).unwrap());

        let mut palette = VoxPalette::default();
        palette
            .retain_property(BASE_COLOR.to_owned(), colors, U32Id::from_u32(0))
            .unwrap();
        palette
            .retain_property(METALLIC.to_owned(), metals, U32Id::from_u32(0))
            .unwrap();
        palette
            .retain_material(vec![U32Id::from_u32(0), U32Id::from_u32(0)])
            .unwrap();
        palette
            .retain_material(vec![U32Id::from_u32(1), U32Id::from_u32(1)])
            .unwrap();
        let palette_id = main.retain_palette(palette).unwrap();

        let mut object = bar();
        object.retain_layer(palette_id, U32Id::from_u32(0));
        for x in 0..2 {
            let voxel_id = object.voxel_id(TyVector3U32::new(x, 0, 0)).unwrap();
            object.release_voxel(voxel_id).unwrap();
            object
                .retain_voxel(voxel_id, &[U32Id::from_u32(x)])
                .unwrap();
        }

        (main, object)
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
    fn the_primitive_writes_its_declared_streams_in_order_at_texel_centers() {
        let mut record = record(Method::Greedy);
        record.primitives[U32Id::from_u32(0).to_usize_id()].uv_streams =
            Some(vec![ArrayDomain::Face, ArrayDomain::Swatch]);

        let document = meshed(&record);
        let (_, object) = document.iter_objects().next().unwrap();
        let (_, primitive) = object.iter_primitives().next().unwrap();

        assert_eq!(primitive.uv_stream_count(), 2);

        // Six faces fill a 4x4 canvas in raster order, and the one swatch
        // fills a 1x1.
        let faces = primitive.uv_stream(U32Id::from_u32(0)).unwrap();
        assert_eq!(faces.len(), 24);
        assert_eq!(
            faces[U32Id::from_u32(0).to_usize_id()],
            TyVector2F64::new(0.125, 0.125)
        );
        assert_eq!(
            faces[U32Id::from_u32(20).to_usize_id()],
            TyVector2F64::new(0.375, 0.375)
        );

        let swatches = primitive.uv_stream(U32Id::from_u32(1)).unwrap();
        assert!(swatches.iter().all(|&uv| uv == TyVector2F64::new(0.5, 0.5)));
    }

    #[test]
    fn a_stream_list_naming_a_domain_twice_errors() {
        let primitive_id = U32Id::from_u32(0);
        let mut record = record(Method::Greedy);
        record.primitives[primitive_id.to_usize_id()].uv_streams =
            Some(vec![ArrayDomain::Face, ArrayDomain::Face]);

        assert_eq!(
            failing_element(&record),
            MeshElement::PrimitiveUvStreams { primitive_id }
        );
    }

    #[test]
    fn an_exact_canvas_too_small_for_a_written_atlas_errors() {
        let primitive_id = U32Id::from_u32(0);

        let mut swatch = record(Method::Greedy);
        swatch.texture_shape = TextureShape::Exact(1);
        swatch.primitives[primitive_id.to_usize_id()].uv_streams = Some(vec![ArrayDomain::Swatch]);
        meshed(&swatch);

        let mut face = record(Method::Greedy);
        face.texture_shape = TextureShape::Exact(1);
        face.primitives[primitive_id.to_usize_id()].uv_streams = Some(vec![ArrayDomain::Face]);
        assert_eq!(failing_element(&face), MeshElement::TextureShape);
    }

    #[test]
    fn a_layer_over_an_unknown_palette_errors() {
        let main: VoxMain = VoxMain::default();
        let mut object = bar();
        object.retain_layer(U32Id::from_u32(9), U32Id::from_u32(0));

        assert!(mesh(&main, &object, &record(Method::Greedy)).is_err());
    }

    #[test]
    fn the_program_reads_the_palette_and_the_computed_values() {
        let (main, object) = painted();
        let mut record = record(Method::Greedy);
        record.computed_bindings = vec![
            ComputedBinding {
                name: "swatchIndex".to_owned(),
                computation: Computation::Index(ArrayDomain::Swatch),
            },
            ComputedBinding {
                name: "ao".to_owned(),
                computation: Computation::Occlusion,
            },
            ComputedBinding {
                name: "voxelPosition".to_owned(),
                computation: Computation::VoxelPosition,
            },
        ];
        record.program = "tint = baseColor.rgb * metallic; last = max(swatchIndex); \
                          width = max(voxelPosition.x); open = min(ao);"
            .to_owned();

        mesh(&main, &object, &record).unwrap().validate().unwrap();
    }

    #[test]
    fn greedy_splits_where_a_climbed_value_differs_and_nowhere_else() {
        let (main, object) = painted();

        let quads = |program: &str| {
            let mut record = record(Method::Greedy);
            record.computed_bindings.push(ComputedBinding {
                name: "ao".to_owned(),
                computation: Computation::Occlusion,
            });
            record.program = program.to_owned();
            let document = mesh(&main, &object, &record).unwrap();
            let (_, object) = document.iter_objects().next().unwrap();
            let (_, primitive) = object.iter_primitives().next().unwrap();
            primitive.triangle_count() / 2
        };

        // The bar's two materials differ in metallic and share an alpha of
        // one. A computed occlusion the program never reads changes nothing.
        assert_eq!(quads(""), 6);
        assert_eq!(quads("x = face(baseColor.a);"), 6);
        assert_eq!(quads("x = swatchAvg(faceAvg(ao)) * metallic;"), 6);
        assert_eq!(quads("x = face(metallic);"), 10);
    }

    #[test]
    fn a_program_error_names_the_program() {
        let (main, object) = painted();
        let mut record = record(Method::Greedy);
        record.program = "x = nothing;".to_owned();

        let error = mesh(&main, &object, &record).unwrap_err();

        assert!(matches!(
            error,
            Error::MeshRecord {
                element: MeshElement::Program,
                ..
            }
        ));
        assert!(
            error
                .to_string()
                .contains("`nothing` has no value in scope"),
            "{error}"
        );
    }

    #[test]
    fn a_computed_binding_shadowing_a_property_or_bound_twice_errors() {
        let (main, object) = painted();

        let mut shadowing = record(Method::Greedy);
        shadowing.computed_bindings.push(ComputedBinding {
            name: METALLIC.to_owned(),
            computation: Computation::Occlusion,
        });
        let error = mesh(&main, &object, &shadowing).unwrap_err();
        assert!(error.to_string().contains("shadows"), "{error}");

        let mut twice = record(Method::Greedy);
        twice.computed_bindings = vec![
            ComputedBinding {
                name: "ao".to_owned(),
                computation: Computation::Occlusion,
            },
            ComputedBinding {
                name: "ao".to_owned(),
                computation: Computation::VoxelPosition,
            },
        ];
        let error = mesh(&main, &object, &twice).unwrap_err();
        assert!(error.to_string().contains("is bound twice"), "{error}");
    }
}
