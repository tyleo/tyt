use crate::{
    Error, Result,
    dependencies::mesh::EncodePng,
    operations::mesh::{
        Atlases, FacePartition, Images, MergeRules, MeshElement, MeshRecord, Method, ProgramRun,
        Streams, Swatches, WriteContext, mesh_slices, object_to_mesh_geometry, table_index,
        write_attributes, write_extras, write_files, write_materials, write_primitive,
    },
};
use branded_id::U32Id;
use meshdoc::{MeshHierarchyNode, MeshMain, MeshObject, MeshPrimitive, MeshProperty};
use std::collections::HashMap;
use voxcore::{VoxExt, VoxMain, VoxObject};

/// Meshes `object` under `record` into a document of one object under one root
/// node, both named as `object` is. Positions are in meters on the grid's axes,
/// which the document shares. `dependencies` encodes the pngs. An error names
/// the record element it rose from.
pub fn mesh<D: EncodePng, T: VoxExt>(
    dependencies: &D,
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

    if record.primitives.is_empty() {
        return Err(Error::mesh_record(
            MeshElement::Primitives,
            "holds no primitive, and a run needs at least one",
        ));
    }

    let geometry = if record.method == Method::Greedy {
        let culled = object_to_mesh_geometry(object, Method::Culled);

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
        object_to_mesh_geometry(object, record.method)
    };

    let run = ProgramRun::over(object, &swatches, record, &geometry)?;

    let streams = Streams::derive(record, &run.destinations)?;

    let atlases = Atlases::new(record.texture_shape, &swatches, &geometry);

    let partition = FacePartition::derive(record, &run, &atlases)?;

    let files = write_files(dependencies, record, &run, &streams, &atlases)?;

    let mut document = MeshMain::default();

    let file_ids = files
        .into_iter()
        .map(|file| {
            let name = file.name.clone();
            Ok((name, document.retain_file(file)?))
        })
        .collect::<Result<HashMap<_, _>>>()?;

    let mut images = Images::default();

    let context = WriteContext {
        dependencies,
        record,
        run: &run,
        streams: &streams,
        atlases: &atlases,
        file_ids: &file_ids,
    };

    write_materials(&context, &mut document, &mut images)?;

    let properties = write_extras(
        &context,
        &mut document,
        &mut images,
        &record.mesh_extras,
        |name| MeshElement::MeshExtra {
            name: name.to_owned(),
        },
        |bake| streams.mesh_stream_id(bake),
    )?;

    let primitives = record
        .primitives
        .iter()
        .enumerate()
        .map(|(index, primitive_record)| {
            let primitive_id = U32Id::from_u32(table_index(index));
            let faces = partition.faces(primitive_id);

            let mut primitive = write_primitive(
                &geometry,
                faces,
                record.voxel_size,
                primitive_record,
                streams.primitive_list(primitive_id),
                &atlases,
            )?;

            write_attributes(
                &mut primitive,
                primitive_id,
                &primitive_record.attributes,
                faces,
                &run,
                &atlases,
            )?;

            Ok(primitive)
        })
        .collect::<Result<Vec<_>>>()?;

    place(document, object.name(), primitives, properties)
}

/// `document` with `primitives` and `properties` in one object named
/// `name`, placed by one root node of the same name.
fn place(
    mut document: MeshMain<()>,
    name: &str,
    primitives: Vec<MeshPrimitive>,
    properties: Vec<MeshProperty>,
) -> Result<MeshMain<()>> {
    let mut object = MeshObject::new(name.to_owned());

    for primitive in primitives {
        object.retain_primitive(primitive);
    }

    object.set_properties(properties);

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
        Error, Result,
        dependencies::DependenciesImpl,
        operations::mesh::{
            ArrayDomain, AttributeWrite, Computation, ComputedBinding, ExtraForm, ExtraSource,
            ExtraWrite, FileForm, FileWrite, MaterialRecord, MeshElement, MeshRecord, Method,
            PrimitiveRecord, SlotSource, SlotWrite, TextureShape, Transfer, WrittenValue, mesh,
        },
    };
    use branded_id::{IdVec, U32Id};
    use meshdoc::{
        BMeshMaterial, MeshAlphaMode, MeshAttributeComponents, MeshImageSource, MeshMagFilter,
        MeshMain, MeshMinFilter, MeshPropertyValue, MeshWrap,
    };
    use png::{ColorType, Decoder, Info};
    use std::io::Cursor;
    use ty_math::{TyLinSrgbF64, TyLinSrgbaF64, TyVector2F64, TyVector3F64, TyVector3U32};
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
        let document = mesh(&DependenciesImpl, &main, &bar(), record).unwrap();
        document.validate().unwrap();
        document
    }

    /// The element the bar fails to mesh on under `record`.
    fn failing_element(record: &MeshRecord) -> MeshElement {
        let main: VoxMain = VoxMain::default();
        record_error(mesh(&DependenciesImpl, &main, &bar(), record))
    }

    /// The element the painted bar fails to mesh on under `record`.
    fn failing_painted_element(record: &MeshRecord) -> MeshElement {
        let (main, object) = painted();
        record_error(mesh(&DependenciesImpl, &main, &object, record))
    }

    /// The element `result`'s record error rose from.
    fn record_error(result: Result<MeshMain<()>>) -> MeshElement {
        match result.unwrap_err() {
            Error::MeshRecord { element, .. } => element,
            error => panic!("expected a record error, got {error}"),
        }
    }

    /// A slot write of `property` from `source`.
    fn slot(property: &str, source: SlotSource) -> SlotWrite {
        SlotWrite {
            property: property.to_owned(),
            source,
        }
    }

    /// A slot write of `property` from the value `expression`.
    fn value_slot(property: &str, expression: &str) -> SlotWrite {
        slot(property, SlotSource::Value(expression.to_owned()))
    }

    /// One material of `slots` in the table.
    fn materials(slots: Vec<SlotWrite>) -> IdVec<BMeshMaterial, MaterialRecord> {
        IdVec::from_vec(vec![MaterialRecord {
            slots,
            ..MaterialRecord::default()
        }])
    }

    /// An extras entry `name` of `form` from `source`.
    fn extra(name: &str, form: ExtraForm, source: ExtraSource) -> ExtraWrite {
        ExtraWrite {
            name: name.to_owned(),
            form,
            source,
        }
    }

    /// An extras entry `name` of `form` from the value `expression` under
    /// `transfer`.
    fn value_extra(
        name: &str,
        form: ExtraForm,
        expression: &str,
        transfer: Transfer,
    ) -> ExtraWrite {
        extra(
            name,
            form,
            ExtraSource::Value(WrittenValue {
                expression: expression.to_owned(),
                transfer,
            }),
        )
    }

    /// A write of `expression` to the modeled `attribute`.
    fn builtin(attribute: &str, expression: &str) -> AttributeWrite {
        AttributeWrite::Builtin {
            attribute: attribute.to_owned(),
            expression: expression.to_owned(),
        }
    }

    /// A write of `expression` to the custom attribute `name` under
    /// `transfer`.
    fn custom(name: &str, expression: &str, transfer: Transfer) -> AttributeWrite {
        AttributeWrite::Custom {
            name: name.to_owned(),
            value: WrittenValue {
                expression: expression.to_owned(),
                transfer,
            },
        }
    }

    /// A write of `expression` into `file`, a png or the JSON entry `name`.
    fn file_write(
        file: &str,
        name: Option<&str>,
        expression: &str,
        transfer: Transfer,
    ) -> FileWrite {
        FileWrite {
            file: file.to_owned(),
            value: WrittenValue {
                expression: expression.to_owned(),
                transfer,
            },
            form: match name {
                Some(name) => FileForm::Json {
                    name: name.to_owned(),
                },
                None => FileForm::Png,
            },
        }
    }

    /// The decoded samples and header of the png `document` holds as `name`.
    fn decoded_png(document: &MeshMain<()>, name: &str) -> (Vec<u8>, Info<'static>) {
        let (_, file) = document.file_by_name(name).unwrap();
        decoded(&file.bytes)
    }

    /// The decoded samples and header of the png `bytes`.
    fn decoded(bytes: &[u8]) -> (Vec<u8>, Info<'static>) {
        let mut reader = Decoder::new(Cursor::new(bytes.to_vec()))
            .read_info()
            .unwrap();
        let mut buffer = vec![0u8; reader.output_buffer_size().unwrap()];
        let output = reader.next_frame(&mut buffer).unwrap();
        buffer.truncate(output.buffer_size());
        (buffer, reader.info().clone())
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
        // two triangles, scaled to two meters per voxel on the grid's axes.
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
    fn the_selects_split_the_faces_into_primitives_with_their_own_streams() {
        let (main, object) = painted();
        let mut record = record(Method::Greedy);
        record.program = "shiny = metallic > 0; dull = !shiny;".to_owned();
        record.primitives = IdVec::from_vec(vec![
            PrimitiveRecord {
                select: "shiny".to_owned(),
                name: Some("shiny".to_owned()),
                uv_streams: Some(vec![ArrayDomain::Face]),
                ..implicit_primitive()
            },
            PrimitiveRecord {
                select: "dull".to_owned(),
                ..implicit_primitive()
            },
        ]);

        let document = mesh(&DependenciesImpl, &main, &object, &record).unwrap();
        document.validate().unwrap();
        let (_, object) = document.iter_objects().next().unwrap();
        assert_eq!(object.primitive_count(), 2);

        // The select splits the greedy bar at the material seam, five faces
        // a side.
        let primitives: Vec<_> = object
            .iter_primitives()
            .map(|(_, primitive)| primitive)
            .collect();
        assert_eq!(primitives[0].name(), "shiny");
        assert_eq!(primitives[0].triangle_count(), 10);
        assert_eq!(primitives[0].uv_stream_count(), 1);
        assert_eq!(
            primitives[0].uv_stream(U32Id::from_u32(0)).unwrap().len(),
            20
        );
        assert_eq!(primitives[1].name(), "");
        assert_eq!(primitives[1].triangle_count(), 10);
        assert_eq!(primitives[1].uv_stream_count(), 0);
        for primitive in &primitives {
            for triangle in primitive.triangles().iter() {
                assert!(
                    triangle
                        .vertex_ids
                        .iter()
                        .all(|vertex_id| vertex_id.to_usize_id().to_usize() < 20)
                );
            }
        }
    }

    #[test]
    fn a_select_routes_a_face_merged_across_swatches_that_agree() {
        // Greedy merges the faces of the last two swatches because they
        // share `metallic`, and the select reads one answer off each merged
        // face.
        let mut main: VoxMain = VoxMain::default();
        let colors = main.retain_value_pool(
            VoxValuePool::vec_4_float(vec![
                [1.0, 0.0, 0.0, 1.0],
                [0.0, 1.0, 0.0, 1.0],
                [0.0, 0.0, 1.0, 1.0],
            ])
            .unwrap(),
        );
        let metals = main.retain_value_pool(VoxValuePool::float(vec![1.0, 0.0]).unwrap());
        let mut palette = VoxPalette::default();
        palette
            .retain_property(BASE_COLOR.to_owned(), colors, U32Id::from_u32(0))
            .unwrap();
        palette
            .retain_property(METALLIC.to_owned(), metals, U32Id::from_u32(0))
            .unwrap();
        for (color, metal) in [(0, 0), (1, 1), (2, 1)] {
            palette
                .retain_material(vec![U32Id::from_u32(color), U32Id::from_u32(metal)])
                .unwrap();
        }
        let palette_id = main.retain_palette(palette).unwrap();
        let mut object = VoxObject::new("bar".to_owned(), TyVector3U32::new(3, 1, 1)).unwrap();
        object.retain_layer(palette_id, U32Id::from_u32(0));
        for x in 0..3 {
            let voxel_id = object.voxel_id(TyVector3U32::new(x, 0, 0)).unwrap();
            object
                .retain_voxel(voxel_id, &[U32Id::from_u32(x)])
                .unwrap();
        }

        let mut record = record(Method::Greedy);
        record.program = "shiny = metallic > 0; dull = !shiny;".to_owned();
        record.primitives = IdVec::from_vec(vec![
            PrimitiveRecord {
                select: "shiny".to_owned(),
                ..implicit_primitive()
            },
            PrimitiveRecord {
                select: "dull".to_owned(),
                ..implicit_primitive()
            },
        ]);

        let document = mesh(&DependenciesImpl, &main, &object, &record).unwrap();
        let (_, object) = document.iter_objects().next().unwrap();
        let primitives: Vec<_> = object
            .iter_primitives()
            .map(|(_, primitive)| primitive)
            .collect();
        assert_eq!(primitives[0].triangle_count(), 10);
        assert_eq!(primitives[1].triangle_count(), 10);
    }

    #[test]
    fn a_false_select_writes_an_empty_primitive() {
        let mut record = record(Method::Greedy);
        record.primitives.push(PrimitiveRecord {
            select: "false".to_owned(),
            ..implicit_primitive()
        });

        let document = meshed(&record);
        let (_, object) = document.iter_objects().next().unwrap();
        let primitives: Vec<_> = object
            .iter_primitives()
            .map(|(_, primitive)| primitive)
            .collect();
        assert_eq!(primitives[0].triangle_count(), 12);
        assert_eq!(primitives[1].vertex_count(), 0);
    }

    #[test]
    fn a_face_no_select_takes_or_two_selects_take_errors() {
        let mut unclaimed = record(Method::Greedy);
        unclaimed.primitives[U32Id::from_u32(0).to_usize_id()].select = "false".to_owned();
        assert_eq!(failing_element(&unclaimed), MeshElement::Primitives);

        let mut twice = record(Method::Greedy);
        twice.primitives.push(implicit_primitive());
        assert_eq!(
            failing_element(&twice),
            MeshElement::PrimitiveSelect {
                primitive_id: U32Id::from_u32(1)
            }
        );
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
    fn an_attribute_lands_at_the_corners_with_lower_domains_climbing_in() {
        let (main, object) = painted();
        let mut record = record(Method::Greedy);
        record.computed_bindings = vec![
            ComputedBinding {
                name: "cornerIndex".to_owned(),
                computation: Computation::Index(ArrayDomain::Corner),
            },
            ComputedBinding {
                name: "swatchIndex".to_owned(),
                computation: Computation::Index(ArrayDomain::Swatch),
            },
        ];
        record.program = "half = metallic * 0.5;".to_owned();
        record.primitives = IdVec::from_vec(vec![
            PrimitiveRecord {
                select: "metallic > 0".to_owned(),
                attributes: vec![
                    builtin("COLOR_0", "baseColor"),
                    custom("_CORNER", "u16(cornerIndex)", Transfer::Linear),
                    custom("_HALF", "half", Transfer::Srgb),
                    custom("_METAL", "metallic", Transfer::Linear),
                    custom("_PALETTE", "u8(swatchIndex)", Transfer::Linear),
                ],
                ..implicit_primitive()
            },
            PrimitiveRecord {
                select: "metallic == 0".to_owned(),
                attributes: vec![builtin("COLOR_0", "baseColor.rgb")],
                ..implicit_primitive()
            },
        ]);

        let document = mesh(&DependenciesImpl, &main, &object, &record).unwrap();
        document.validate().unwrap();
        let (_, object) = document.iter_objects().next().unwrap();
        let primitives: Vec<_> = object
            .iter_primitives()
            .map(|(_, primitive)| primitive)
            .collect();

        let attribute = |name: &str| {
            primitives[0]
                .iter_vertex_attributes()
                .find(|(_, attribute)| attribute.name == name)
                .map(|(_, attribute)| attribute)
                .unwrap()
        };

        // Each voxel's five faces draw in one primitive. The metallic red
        // voxel draws first.
        assert_eq!(primitives[0].vertex_count(), 20);
        assert!(
            primitives[0]
                .colors()
                .unwrap()
                .iter()
                .all(|&color| color == TyLinSrgbaF64::new(1.0, 0.0, 0.0, 1.0))
        );
        assert_eq!(primitives[0].vertex_attribute_count(), 4);

        // A corner value lands each corner exactly, so a face's corners run
        // in a block of four.
        let corners = attribute("_CORNER");
        assert_eq!(corners.width, 1);
        let MeshAttributeComponents::U16(indices) = &corners.components else {
            panic!("a u16 value lands as u16");
        };
        assert_eq!(indices.len(), 20);
        assert!(indices.chunks_exact(4).all(|block| {
            block[0] % 4 == 0 && block == [block[0], block[0] + 1, block[0] + 2, block[0] + 3]
        }));
        assert!(indices.windows(2).all(|pair| pair[0] < pair[1]));

        let half = attribute("_HALF");
        let MeshAttributeComponents::F64(halves) = &half.components else {
            panic!("an f32 value lands as f64");
        };
        assert!(
            halves
                .iter()
                .all(|&component| (component - 0.735_356_983_052_449_5).abs() < 1e-9)
        );

        assert_eq!(
            attribute("_METAL").components,
            MeshAttributeComponents::F64(vec![1.0; 20])
        );

        assert_eq!(
            attribute("_PALETTE").components,
            MeshAttributeComponents::U8(vec![0; 20])
        );

        // A vec3 color takes an alpha of one.
        assert_eq!(primitives[1].vertex_count(), 20);
        assert!(
            primitives[1]
                .colors()
                .unwrap()
                .iter()
                .all(|&color| color == TyLinSrgbaF64::new(0.0, 0.0, 1.0, 1.0))
        );
    }

    #[test]
    fn an_attribute_written_wrong_errors_and_names_the_attribute() {
        let primitive_id = U32Id::from_u32(0);

        for attribute in [
            builtin("COLOR_0", "baseColor * 2.0"),
            builtin("COLOR_0", "metallic"),
            builtin("TANGENT", "baseColor"),
            custom("_INDEX", "swatchIndex", Transfer::Linear),
            custom("_PALETTE", "u8(swatchIndex)", Transfer::Srgb),
            custom("_SHINY", "metallic > 0", Transfer::Linear),
            custom("_TINT", "metallic * 2.0", Transfer::Srgb),
        ] {
            let name = attribute.name().to_owned();

            let mut record = record(Method::Greedy);
            record.computed_bindings.push(ComputedBinding {
                name: "swatchIndex".to_owned(),
                computation: Computation::Index(ArrayDomain::Swatch),
            });
            record.primitives[primitive_id.to_usize_id()]
                .attributes
                .push(attribute);

            assert_eq!(
                failing_painted_element(&record),
                MeshElement::PrimitiveAttribute {
                    primitive_id,
                    name: name.clone()
                },
                "{name}"
            );
        }
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

        assert!(mesh(&DependenciesImpl, &main, &object, &record(Method::Greedy)).is_err());
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

        mesh(&DependenciesImpl, &main, &object, &record)
            .unwrap()
            .validate()
            .unwrap();
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
            let document = mesh(&DependenciesImpl, &main, &object, &record).unwrap();
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

        let error = mesh(&DependenciesImpl, &main, &object, &record).unwrap_err();

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
        let error = mesh(&DependenciesImpl, &main, &object, &shadowing).unwrap_err();
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
        let error = mesh(&DependenciesImpl, &main, &object, &twice).unwrap_err();
        assert!(error.to_string().contains("is bound twice"), "{error}");
    }

    #[test]
    fn a_png_file_bakes_at_its_values_domain_under_its_transfer() {
        let (main, object) = painted();
        let mut record = record(Method::Greedy);
        record.files = vec![
            file_write("bar-metal.png", None, "metallic", Transfer::Linear),
            file_write("bar-color.png", None, "baseColor", Transfer::Srgb),
        ];

        let document = mesh(&DependenciesImpl, &main, &object, &record).unwrap();
        document.validate().unwrap();
        assert_eq!(document.file_count(), 2);
        assert_eq!(document.image_count(), 0);

        // Two swatches fill the first row of a 2x2 canvas. The rest stays
        // transparent black.
        let (samples, info) = decoded_png(&document, "bar-metal.png");
        assert_eq!(info.color_type, ColorType::Grayscale);
        assert_eq!((info.width, info.height), (2, 2));
        assert_eq!(samples, [255, 0, 0, 0]);
        assert!(info.srgb.is_none());

        let (samples, info) = decoded_png(&document, "bar-color.png");
        assert_eq!(info.color_type, ColorType::Rgba);
        assert_eq!(
            samples,
            [255, 0, 0, 255, 0, 0, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0]
        );
        assert!(info.srgb.is_some());
    }

    #[test]
    fn a_corner_png_takes_the_block_layout() {
        let mut record = record(Method::Greedy);
        record.computed_bindings.push(ComputedBinding {
            name: "ao".to_owned(),
            computation: Computation::Occlusion,
        });
        record.files = vec![file_write("bar-ao.png", None, "ao", Transfer::Linear)];

        let document = meshed(&record);

        // Six faces take six 2x2 blocks on a 4x4-cell canvas. Every corner
        // of the bar is open.
        let (samples, info) = decoded_png(&document, "bar-ao.png");
        assert_eq!((info.width, info.height), (8, 8));
        assert_eq!(samples.iter().filter(|&&sample| sample == 255).count(), 24);
        assert_eq!(samples.iter().filter(|&&sample| sample == 0).count(), 40);
    }

    #[test]
    fn a_corner_texture_samples_linear() {
        let material_id = U32Id::from_u32(0);
        let mut record = record(Method::Greedy);
        record.computed_bindings.push(ComputedBinding {
            name: "ao".to_owned(),
            computation: Computation::Occlusion,
        });
        record.files = vec![file_write("bar-ao.png", None, "ao", Transfer::Linear)];
        record.materials = materials(vec![
            value_slot("metallicRoughnessTexture", "ao"),
            slot(
                "occlusionTexture",
                SlotSource::File("bar-ao.png".to_owned()),
            ),
        ]);
        record.primitives[U32Id::from_u32(0).to_usize_id()].material_id = Some(material_id);

        let document = meshed(&record);

        // The embedded bake and the referenced file both sit on the corner
        // atlas, so both textures blend their blocks.
        let material = document.material(material_id).unwrap();
        for texture_ref in [
            material.metallic_roughness_texture.unwrap(),
            material.occlusion_texture.unwrap(),
        ] {
            let texture = document.texture(texture_ref.texture_id).unwrap();
            assert_eq!(texture.mag_filter, Some(MeshMagFilter::Linear));
            assert_eq!(texture.min_filter, Some(MeshMinFilter::Linear));
            assert_eq!(texture.wrap_s, MeshWrap::ClampToEdge);
        }
    }

    #[test]
    fn json_entries_on_one_path_merge_in_record_order() {
        let (main, object) = painted();
        let mut record = record(Method::Greedy);
        record.files = vec![
            file_write("bar.json", Some("metal"), "metallic", Transfer::Linear),
            file_write("bar.json", Some("shiny"), "metallic > 0", Transfer::Linear),
            file_write("bar.json", Some("count"), "2u32", Transfer::Linear),
            file_write("bar.json", Some("grey"), "0.5", Transfer::Srgb),
        ];

        let document = mesh(&DependenciesImpl, &main, &object, &record).unwrap();
        let (_, file) = document.file_by_name("bar.json").unwrap();
        let text = String::from_utf8(file.bytes.clone()).unwrap();

        assert!(
            text.starts_with(
                "{\n  \"metal\": [\n    1.0,\n    0.0\n  ],\n  \"shiny\": [\n    true,\n    \
                 false\n  ],\n  \"count\": 2,\n  \"grey\": 0.73"
            ),
            "{text}"
        );
        assert!(text.ends_with("\n}\n"), "{text}");
    }

    #[test]
    fn a_file_written_wrong_errors_and_names_the_file() {
        let element = MeshElement::File {
            file: "bar.png".to_owned(),
        };

        let mut hot = record(Method::Greedy);
        hot.files = vec![file_write("bar.png", None, "face(1.5)", Transfer::Linear)];
        assert_eq!(failing_element(&hot), element);

        let mut counted = record(Method::Greedy);
        counted.files = vec![file_write("bar.png", None, "face(1u32)", Transfer::Linear)];
        assert_eq!(failing_element(&counted), element);

        let mut twice = record(Method::Greedy);
        twice.files = vec![
            file_write("bar.png", Some("n"), "1", Transfer::Linear),
            file_write("bar.png", Some("n"), "2", Transfer::Linear),
        ];
        assert_eq!(failing_element(&twice), element);

        let mut mixed = record(Method::Greedy);
        mixed.files = vec![
            file_write("bar.png", Some("n"), "1", Transfer::Linear),
            file_write("bar.png", None, "face(1)", Transfer::Linear),
        ];
        assert_eq!(failing_element(&mixed), element);
    }

    #[test]
    fn a_slot_value_fills_its_factor_or_embeds_its_texture() {
        let (main, object) = painted();
        let primitive_id = U32Id::from_u32(0);
        let material_id = U32Id::from_u32(0);
        let mut record = record(Method::Greedy);
        record.materials = materials(vec![
            value_slot("baseColorTexture", "baseColor"),
            value_slot("metallicFactor", "max(metallic)"),
            value_slot("emissiveFactor", "rgb(1, 0.5, 0)"),
            value_slot("doubleSided", "true"),
            value_slot("alphaMode", "\"MASK\""),
            value_slot("alphaCutoff", "0.25"),
            value_slot("ior", "0.0"),
        ]);
        record.primitives[primitive_id.to_usize_id()].material_id = Some(material_id);

        let document = mesh(&DependenciesImpl, &main, &object, &record).unwrap();
        document.validate().unwrap();

        let material = document.material(material_id).unwrap();
        assert_eq!(material.metallic_factor, 1.0);
        assert_eq!(material.emissive_factor, TyLinSrgbF64::new(1.0, 0.5, 0.0));
        assert!(material.double_sided);
        assert_eq!(material.alpha_mode, MeshAlphaMode::Mask);
        assert_eq!(material.alpha_cutoff, 0.25);
        assert_eq!(material.ior, 0.0);

        // The texture samples the swatch atlas through the one stream,
        // nearest and clamped. Its image carries the sRGB chunk.
        let texture_ref = material.base_color_texture.unwrap();
        assert_eq!(texture_ref.uv_stream_id, U32Id::from_u32(0));
        let texture = document.texture(texture_ref.texture_id).unwrap();
        assert_eq!(texture.mag_filter, Some(MeshMagFilter::Nearest));
        assert_eq!(texture.wrap_s, MeshWrap::ClampToEdge);
        assert_eq!(document.image_count(), 1);
        let (samples, info) = decoded(document.image_bytes(texture.image_id).unwrap());
        assert!(info.srgb.is_some());
        assert_eq!(&samples[..8], [255, 0, 0, 255, 0, 0, 255, 255]);

        let (_, object) = document.iter_objects().next().unwrap();
        let (_, primitive) = object.iter_primitives().next().unwrap();
        assert_eq!(primitive.material_id(), Some(material_id));
        assert_eq!(primitive.uv_stream_count(), 1);
    }

    #[test]
    fn two_slots_naming_one_value_share_its_image_and_a_file_slot_references_its_file() {
        let (main, object) = painted();
        let material_id = U32Id::from_u32(0);
        let mut record = record(Method::Greedy);
        record.program = "orm = rgb(1, 0.5, metallic);".to_owned();
        record.files = vec![file_write(
            "bar-albedo.png",
            None,
            "baseColor",
            Transfer::Srgb,
        )];
        record.materials = materials(vec![
            value_slot("occlusionTexture", "orm"),
            value_slot("metallicRoughnessTexture", "orm"),
            slot(
                "baseColorTexture",
                SlotSource::File("bar-albedo.png".to_owned()),
            ),
        ]);
        record.primitives[U32Id::from_u32(0).to_usize_id()].material_id = Some(material_id);

        let document = mesh(&DependenciesImpl, &main, &object, &record).unwrap();
        document.validate().unwrap();

        let material = document.material(material_id).unwrap();
        assert_eq!(
            material.occlusion_texture,
            material.metallic_roughness_texture
        );
        assert_eq!(document.image_count(), 2);

        let (file_id, _) = document.file_by_name("bar-albedo.png").unwrap();
        let texture_ref = material.base_color_texture.unwrap();
        let texture = document.texture(texture_ref.texture_id).unwrap();
        let image = document.image(texture.image_id).unwrap();
        assert_eq!(image.source, MeshImageSource::File(file_id));
    }

    #[test]
    fn a_declared_material_no_primitive_draws_emits_unused() {
        let mut record = record(Method::Greedy);
        record.materials = materials(Vec::new());

        let document = meshed(&record);

        assert_eq!(document.material_count(), 1);
        let (_, object) = document.iter_objects().next().unwrap();
        let (_, primitive) = object.iter_primitives().next().unwrap();
        assert_eq!(primitive.material_id(), None);
    }

    #[test]
    fn a_slot_written_wrong_errors_and_names_the_slot() {
        let slot_element = |property: &str| MeshElement::Slot {
            material_id: U32Id::from_u32(0),
            property: property.to_owned(),
        };

        let failing = |slots: Vec<SlotWrite>, files: Vec<FileWrite>| {
            let mut record = record(Method::Greedy);
            record.materials = materials(slots);
            record.files = files;
            failing_painted_element(&record)
        };

        assert_eq!(
            failing(vec![value_slot("metallicFactor", "metallic")], vec![]),
            slot_element("metallicFactor")
        );
        assert_eq!(
            failing(vec![value_slot("metallicFactor", "2")], vec![]),
            slot_element("metallicFactor")
        );
        assert_eq!(
            failing(vec![value_slot("doubleSided", "1")], vec![]),
            slot_element("doubleSided")
        );
        assert_eq!(
            failing(vec![value_slot("alphaMode", "\"GLOW\"")], vec![]),
            slot_element("alphaMode")
        );
        assert_eq!(
            failing(
                vec![slot(
                    "baseColorTexture",
                    SlotSource::File("bar-rough.png".to_owned())
                )],
                vec![file_write(
                    "bar-rough.png",
                    None,
                    "baseColor",
                    Transfer::Linear
                )]
            ),
            slot_element("baseColorTexture")
        );
        assert_eq!(
            failing(
                vec![
                    value_slot("baseColorTexture", "baseColor"),
                    value_slot("occlusionTexture", "baseColor"),
                ],
                vec![]
            ),
            slot_element("occlusionTexture")
        );
    }

    #[test]
    fn an_extra_lands_typed_and_an_image_extra_samples_through_its_stream() {
        let (main, object) = painted();
        let material_id = U32Id::from_u32(0);
        let mut record = record(Method::Greedy);
        record.files = vec![
            file_write("bar-color.png", None, "baseColor", Transfer::Srgb),
            file_write("bar.json", Some("metal"), "metallic", Transfer::Linear),
        ];
        record.materials = materials(vec![value_slot("baseColorTexture", "baseColor")]);
        record.materials[material_id.to_usize_id()].extras = vec![
            value_extra("metal", ExtraForm::Json, "metallic", Transfer::Linear),
            value_extra("colors", ExtraForm::Json, "baseColor", Transfer::Linear),
            value_extra("shiny", ExtraForm::Json, "metallic > 0", Transfer::Linear),
            value_extra("count", ExtraForm::Json, "2u32", Transfer::Linear),
            value_extra("tag", ExtraForm::Json, "\"steel\"", Transfer::Linear),
            value_extra("albedo", ExtraForm::Image, "baseColor", Transfer::Srgb),
        ];
        record.primitives[U32Id::from_u32(0).to_usize_id()].material_id = Some(material_id);
        record.mesh_extras = vec![
            value_extra("peak", ExtraForm::Json, "max(metallic)", Transfer::Linear),
            value_extra("metalMap", ExtraForm::Image, "metallic", Transfer::Linear),
            extra(
                "rows",
                ExtraForm::Json,
                ExtraSource::File("bar.json".to_owned()),
            ),
            extra(
                "colorMap",
                ExtraForm::Image,
                ExtraSource::File("bar-color.png".to_owned()),
            ),
        ];

        let document = mesh(&DependenciesImpl, &main, &object, &record).unwrap();
        document.validate().unwrap();

        let material = document.material(material_id).unwrap();
        let property = |name: &str| material.property(name).unwrap().value.clone();
        assert_eq!(property("metal"), MeshPropertyValue::Floats(vec![1.0, 0.0]));
        assert_eq!(
            property("colors"),
            MeshPropertyValue::FloatRows(vec![vec![1.0, 0.0, 0.0, 1.0], vec![0.0, 0.0, 1.0, 1.0]])
        );
        assert_eq!(
            property("shiny"),
            MeshPropertyValue::Bools(vec![true, false])
        );
        assert_eq!(property("count"), MeshPropertyValue::Int(2));
        assert_eq!(property("tag"), MeshPropertyValue::Text("steel".to_owned()));

        // The image extra shares the slot's texture. The other two images
        // are the mesh's metallic map and the referenced file.
        assert_eq!(
            property("albedo"),
            MeshPropertyValue::Texture(material.base_color_texture.unwrap())
        );
        assert_eq!(document.image_count(), 3);

        let (_, object) = document.iter_objects().next().unwrap();
        let property = |name: &str| object.property(name).unwrap().value.clone();
        assert_eq!(property("peak"), MeshPropertyValue::Float(1.0));

        let (file_id, _) = document.file_by_name("bar.json").unwrap();
        assert_eq!(property("rows"), MeshPropertyValue::File(file_id));

        let MeshPropertyValue::Texture(metal_map) = property("metalMap") else {
            panic!("an image extra lands as a texture");
        };
        assert_eq!(metal_map.uv_stream_id, U32Id::from_u32(0));
        let texture = document.texture(metal_map.texture_id).unwrap();
        let (samples, info) = decoded(document.image_bytes(texture.image_id).unwrap());
        assert!(info.srgb.is_none());
        assert_eq!(&samples[..2], [255, 0]);

        let MeshPropertyValue::Texture(color_map) = property("colorMap") else {
            panic!("an image extra lands as a texture");
        };
        let (png_id, _) = document.file_by_name("bar-color.png").unwrap();
        let texture = document.texture(color_map.texture_id).unwrap();
        assert_eq!(
            document.image(texture.image_id).unwrap().source,
            MeshImageSource::File(png_id)
        );
    }

    #[test]
    fn an_extra_written_wrong_errors_and_names_the_extra() {
        let mesh_element = |name: &str| MeshElement::MeshExtra {
            name: name.to_owned(),
        };

        let failing = |extras: Vec<ExtraWrite>, files: Vec<FileWrite>| {
            let mut record = record(Method::Greedy);
            record.mesh_extras = extras;
            record.files = files;
            failing_painted_element(&record)
        };

        assert_eq!(
            failing(
                vec![value_extra(
                    "pairs",
                    ExtraForm::Json,
                    "u8(baseColor * 255)",
                    Transfer::Linear
                )],
                vec![]
            ),
            mesh_element("pairs")
        );
        assert_eq!(
            failing(
                vec![value_extra(
                    "count",
                    ExtraForm::Json,
                    "2u32",
                    Transfer::Srgb
                )],
                vec![]
            ),
            mesh_element("count")
        );
        assert_eq!(
            failing(
                vec![extra(
                    "rows",
                    ExtraForm::Json,
                    ExtraSource::File("bar.png".to_owned())
                )],
                vec![file_write("bar.png", None, "metallic", Transfer::Linear)]
            ),
            mesh_element("rows")
        );
        assert_eq!(
            failing(
                vec![extra(
                    "rows",
                    ExtraForm::Json,
                    ExtraSource::File("missing.json".to_owned())
                )],
                vec![]
            ),
            mesh_element("rows")
        );
        assert_eq!(
            failing(
                vec![
                    value_extra("peak", ExtraForm::Json, "1.0", Transfer::Linear),
                    value_extra("peak", ExtraForm::Json, "2.0", Transfer::Linear),
                ],
                vec![]
            ),
            mesh_element("peak")
        );

        // The slot embeds `metallic` linear where the extra asks for it sRGB.
        let material_id = U32Id::from_u32(0);
        let mut twice = record(Method::Greedy);
        twice.materials = materials(vec![value_slot("occlusionTexture", "metallic")]);
        twice.materials[material_id.to_usize_id()].extras = vec![value_extra(
            "metalMap",
            ExtraForm::Image,
            "metallic",
            Transfer::Srgb,
        )];
        twice.primitives[U32Id::from_u32(0).to_usize_id()].material_id = Some(material_id);
        assert_eq!(
            failing_painted_element(&twice),
            MeshElement::MaterialExtra {
                material_id,
                name: "metalMap".to_owned()
            }
        );
    }
}
