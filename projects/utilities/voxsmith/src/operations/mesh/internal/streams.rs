use crate::{
    Error, Result,
    operations::mesh::{
        ArrayDomain, CheckedDestination, ExtraForm, ExtraSource, FileForm, MeshElement, MeshRecord,
        SlotProperty, SlotSource, table_index,
    },
};
use branded_id::{IdVec, U32Id};
use meshdoc::{BMeshMaterial, BMeshPrimitive};
use std::collections::{BTreeSet, HashMap};

/// Every primitive's stream list and the domain each texture bakes at.
#[derive(Debug)]
pub(crate) struct Streams {
    primitives: IdVec<BMeshPrimitive, Vec<ArrayDomain>>,

    /// Each texture element's bake domain; a written png and an image of the
    /// object bake where their values sit.
    bakes: HashMap<MeshElement, ArrayDomain>,
}

impl Streams {
    /// Derives the lists from `record` and its checked `destinations`. A
    /// texture bakes at the lowest listed domain at or above its value's,
    /// except a written file, whose image already has its value's layout.
    pub(crate) fn derive(record: &MeshRecord, destinations: &[CheckedDestination]) -> Result<Self> {
        let domains: HashMap<&MeshElement, ArrayDomain> = destinations
            .iter()
            .filter_map(|checked| {
                ArrayDomain::of(checked.expression.to_type().domain)
                    .map(|domain| (&checked.destination.element, domain))
            })
            .collect();

        let texture_domain = |element: &MeshElement| -> Result<ArrayDomain> {
            domains.get(element).copied().ok_or_else(|| {
                Error::mesh_record(
                    element.clone(),
                    "holds a plain value, and a texture needs one texel per entry",
                )
            })
        };

        let mut file_domains = HashMap::new();

        for file in &record.files {
            if matches!(file.form, FileForm::Png) {
                let domain = texture_domain(&MeshElement::File {
                    file: file.file.clone(),
                })?;

                file_domains.insert(file.file.clone(), domain);
            }
        }

        // The domain of the written png `file`, referenced from `element`.
        let file_domain = |file: &str, element: &MeshElement| -> Result<ArrayDomain> {
            file_domains.get(file).copied().ok_or_else(|| {
                Error::mesh_record(
                    element.clone(),
                    format!("references `{file}`, which the run writes as no png"),
                )
            })
        };

        let mut bakes: HashMap<MeshElement, ArrayDomain> = file_domains
            .iter()
            .map(|(file, &domain)| (MeshElement::File { file: file.clone() }, domain))
            .collect();

        for extra in &record.mesh_extras {
            if matches!(extra.form, ExtraForm::Image) {
                let element = MeshElement::MeshExtra {
                    name: extra.name.clone(),
                };

                let domain = match &extra.source {
                    ExtraSource::File(file) => file_domain(file, &element)?,
                    ExtraSource::Value(_) => texture_domain(&element)?,
                };

                bakes.insert(element, domain);
            }
        }

        let mut materials: IdVec<BMeshMaterial, Vec<ArrayDomain>> = IdVec::default();
        let mut material_bakes: IdVec<BMeshMaterial, BTreeSet<ArrayDomain>> = IdVec::default();

        for (index, material) in record.materials.iter().enumerate() {
            let material_id = U32Id::from_u32(table_index(index));

            // Each texture as `(element, value domain, fixed)`, a written
            // file fixing the layout its image already has.
            let mut textures: Vec<(MeshElement, ArrayDomain, bool)> = Vec::new();

            for slot in &material.slots {
                let property = SlotProperty::parse(&slot.property)
                    .expect("the destinations checked every slot property");

                if !property.is_texture() {
                    continue;
                }

                let element = MeshElement::Slot {
                    material_id,
                    property: slot.property.clone(),
                };

                let texture = match &slot.source {
                    SlotSource::File(file) => (file_domain(file, &element)?, true),
                    SlotSource::Value(_) => (texture_domain(&element)?, false),
                };

                textures.push((element, texture.0, texture.1));
            }

            for extra in &material.extras {
                if !matches!(extra.form, ExtraForm::Image) {
                    continue;
                }

                let element = MeshElement::MaterialExtra {
                    material_id,
                    name: extra.name.clone(),
                };

                let texture = match &extra.source {
                    ExtraSource::File(file) => (file_domain(file, &element)?, true),
                    ExtraSource::Value(_) => (texture_domain(&element)?, false),
                };

                textures.push((element, texture.0, texture.1));
            }

            let list = match &material.uv_streams {
                Some(declared) => {
                    check_list(declared, MeshElement::MaterialUvStreams { material_id })?;
                    declared.clone()
                }

                None => {
                    let mut list: Vec<ArrayDomain> =
                        textures.iter().map(|(_, domain, _)| *domain).collect();
                    list.sort_unstable();
                    list.dedup();
                    list
                }
            };

            let mut baked = BTreeSet::new();

            for (element, domain, fixed) in textures {
                let bake = if fixed {
                    if !list.contains(&domain) {
                        return Err(Error::mesh_record(
                            element,
                            format!(
                                "references a file baked at `{domain}`, and the material's \
                                 stream list holds no `{domain}` stream"
                            ),
                        ));
                    }

                    domain
                } else {
                    list.iter()
                        .copied()
                        .filter(|&listed| listed >= domain)
                        .min()
                        .ok_or_else(|| {
                            Error::mesh_record(
                                element.clone(),
                                format!(
                                    "holds a `{domain}` value, and the material's stream list \
                                     holds no domain at or above it"
                                ),
                            )
                        })?
                };

                baked.insert(bake);
                bakes.insert(element, bake);
            }

            materials.push(list);
            material_bakes.push(baked);
        }

        let mut primitives: IdVec<BMeshPrimitive, Vec<ArrayDomain>> = IdVec::default();

        for (index, primitive) in record.primitives.iter().enumerate() {
            let primitive_id = U32Id::from_u32(table_index(index));

            if let Some(material_id) = primitive.material_id
                && material_id.to_usize_id().to_usize() >= materials.len()
            {
                return Err(Error::mesh_record(
                    MeshElement::PrimitiveMaterial { primitive_id },
                    format!(
                        "draws material {}, which the material table does not hold",
                        material_id.to_u32()
                    ),
                ));
            }

            let list = match (&primitive.uv_streams, primitive.material_id) {
                (Some(declared), material_id) => {
                    let element = MeshElement::PrimitiveUvStreams { primitive_id };

                    check_list(declared, element.clone())?;

                    if let Some(material_id) = material_id {
                        for &bake in &material_bakes[material_id.to_usize_id()] {
                            if !declared.contains(&bake) {
                                return Err(Error::mesh_record(
                                    element,
                                    format!(
                                        "omits `{bake}`, which material {} bakes at",
                                        material_id.to_u32()
                                    ),
                                ));
                            }
                        }
                    }

                    declared.clone()
                }

                (None, Some(material_id)) => materials[material_id.to_usize_id()].clone(),

                (None, None) => Vec::new(),
            };

            primitives.push(list);
        }

        for (index, list) in materials.iter().enumerate() {
            let material_id: U32Id<BMeshMaterial> = U32Id::from_u32(table_index(index));

            for &bake in &material_bakes[material_id.to_usize_id()] {
                // The material's list is what a primitive writes by default,
                // so its position stands where no primitive draws the material.
                let mut position = list
                    .iter()
                    .position(|&domain| domain == bake)
                    .expect("a material's list holds every domain it bakes at");

                let mut first_primitive_id: Option<U32Id<BMeshPrimitive>> = None;

                for (index, primitive) in record.primitives.iter().enumerate() {
                    if primitive.material_id != Some(material_id) {
                        continue;
                    }

                    let primitive_id = U32Id::from_u32(table_index(index));

                    let at = primitives[primitive_id.to_usize_id()]
                        .iter()
                        .position(|&domain| domain == bake)
                        .expect(
                            "a drawing primitive's list holds every domain its material bakes at",
                        );

                    match first_primitive_id {
                        None => {
                            position = at;
                            first_primitive_id = Some(primitive_id);
                        }

                        Some(first_primitive_id) if at != position => {
                            return Err(Error::mesh_record(
                                MeshElement::PrimitiveUvStreams { primitive_id },
                                format!(
                                    "writes the `{bake}` stream material {} bakes at as stream \
                                     {at}, where primitive {} writes it as stream {position}, \
                                     so the material's textures cannot name one stream",
                                    material_id.to_u32(),
                                    first_primitive_id.to_u32()
                                ),
                            ));
                        }

                        Some(_) => {}
                    }
                }
            }
        }

        Ok(Streams { primitives, bakes })
    }

    /// The streams primitive `primitive_id` writes, in stream order.
    pub(crate) fn primitive_list(&self, primitive_id: U32Id<BMeshPrimitive>) -> &[ArrayDomain] {
        &self.primitives[primitive_id.to_usize_id()]
    }

    /// The domain the texture at `element` bakes at.
    pub(crate) fn bake(&self, element: &MeshElement) -> ArrayDomain {
        *self
            .bakes
            .get(element)
            .expect("every texture destination bakes somewhere")
    }
}

/// Errors from `element` if `list` names a domain twice.
fn check_list(list: &[ArrayDomain], element: MeshElement) -> Result<()> {
    let mut seen = BTreeSet::new();

    for &domain in list {
        if !seen.insert(domain) {
            return Err(Error::mesh_record(
                element,
                format!("lists `{domain}` twice"),
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        Error, Result,
        operations::mesh::{
            ArrayDomain, Destination, ExtraForm, ExtraSource, ExtraWrite, FileForm, FileWrite,
            MaterialRecord, MeshElement, MeshRecord, Method, PrimitiveRecord, SlotSource,
            SlotWrite, Streams, TextureShape, Transfer, WrittenValue,
        },
    };
    use branded_id::{IdVec, U32Id};
    use vox_value_language::{Dimension, Domain, Scalar, Type, TypeEnvironment, check, parse};

    fn written(expression: &str) -> WrittenValue {
        WrittenValue {
            expression: expression.to_owned(),
            transfer: Transfer::Linear,
        }
    }

    fn value_slot(property: &str, expression: &str) -> SlotWrite {
        SlotWrite {
            property: property.to_owned(),
            source: SlotSource::Value(expression.to_owned()),
        }
    }

    fn file_slot(property: &str, file: &str) -> SlotWrite {
        SlotWrite {
            property: property.to_owned(),
            source: SlotSource::File(file.to_owned()),
        }
    }

    fn material(uv_streams: Option<Vec<ArrayDomain>>, slots: Vec<SlotWrite>) -> MaterialRecord {
        MaterialRecord {
            uv_streams,
            slots,
            ..Default::default()
        }
    }

    fn primitive(material: Option<u32>, uv_streams: Option<Vec<ArrayDomain>>) -> PrimitiveRecord {
        PrimitiveRecord {
            material_id: material.map(U32Id::from_u32),
            select: "true".to_owned(),
            name: None,
            normal: true,
            uv_streams,
            attributes: Vec::new(),
        }
    }

    /// A record of `materials` and `primitives` writing `height.png` over
    /// the voxel `height`.
    fn record(materials: Vec<MaterialRecord>, primitives: Vec<PrimitiveRecord>) -> MeshRecord {
        MeshRecord {
            method: Method::Greedy,
            texture_shape: TextureShape::Pot,
            voxel_size: 1.0,
            computed_bindings: Vec::new(),
            program: String::new(),
            materials: IdVec::from_vec(materials),
            primitives: IdVec::from_vec(primitives),
            files: vec![FileWrite {
                file: "height.png".to_owned(),
                value: written("height"),
                form: FileForm::Png,
            }],
            mesh_extras: Vec::new(),
        }
    }

    /// The streams of `record` over a scope binding the swatch `albedo`, the
    /// voxel `height`, the face `flat`, and the corner `ao`.
    fn derive(record: &MeshRecord) -> Result<Streams> {
        let environment = TypeEnvironment {
            types: [
                ("albedo", Domain::Swatch),
                ("ao", Domain::Corner),
                ("flat", Domain::Face),
                ("height", Domain::Voxel),
            ]
            .into_iter()
            .map(|(name, domain)| {
                (
                    name.to_owned(),
                    Type {
                        domain,
                        dimension: Dimension::Vec1,
                        scalar: Scalar::F32,
                    },
                )
            })
            .collect(),
        };
        let checked = check(parse("").unwrap(), &environment).unwrap();

        let destinations = Destination::of_record(record)?
            .into_iter()
            .map(|destination| destination.check(&checked))
            .collect::<Result<Vec<_>>>()?;

        Streams::derive(record, &destinations)
    }

    fn failing_element(record: &MeshRecord) -> MeshElement {
        match derive(record).unwrap_err() {
            Error::MeshRecord { element, .. } => element,
            error => panic!("expected a record error, got {error}"),
        }
    }

    fn slot(material: u32, property: &str) -> MeshElement {
        MeshElement::Slot {
            material_id: U32Id::from_u32(material),
            property: property.to_owned(),
        }
    }

    #[test]
    fn a_material_list_derives_from_its_textures_sorted_up_the_ladder() {
        let mut material = material(
            None,
            vec![
                value_slot("occlusionTexture", "ao"),
                value_slot("emissiveStrength", "max(ao)"),
                file_slot("metallicRoughnessTexture", "height.png"),
                value_slot("baseColorTexture", "albedo"),
            ],
        );
        material.extras.push(ExtraWrite {
            name: "glow".to_owned(),
            form: ExtraForm::Image,
            source: ExtraSource::Value(written("flat")),
        });
        let record = record(
            vec![material],
            vec![primitive(Some(0), None), primitive(None, None)],
        );

        let streams = derive(&record).unwrap();

        assert_eq!(
            streams.primitive_list(U32Id::from_u32(0)),
            [
                ArrayDomain::Swatch,
                ArrayDomain::Voxel,
                ArrayDomain::Face,
                ArrayDomain::Corner
            ]
        );
        assert_eq!(streams.primitive_list(U32Id::from_u32(1)), []);

        assert_eq!(
            streams.bake(&slot(0, "baseColorTexture")),
            ArrayDomain::Swatch
        );
        assert_eq!(
            streams.bake(&slot(0, "metallicRoughnessTexture")),
            ArrayDomain::Voxel
        );
        assert_eq!(
            streams.bake(&slot(0, "occlusionTexture")),
            ArrayDomain::Corner
        );
        assert_eq!(
            streams.bake(&MeshElement::MaterialExtra {
                material_id: U32Id::from_u32(0),
                name: "glow".to_owned(),
            }),
            ArrayDomain::Face
        );
        assert_eq!(
            streams.bake(&MeshElement::File {
                file: "height.png".to_owned(),
            }),
            ArrayDomain::Voxel
        );
    }

    #[test]
    fn a_declared_material_list_bakes_each_texture_at_the_lowest_domain_at_or_above_its_own() {
        let climbing = record(
            vec![material(
                Some(vec![ArrayDomain::Face]),
                vec![value_slot("baseColorTexture", "albedo")],
            )],
            vec![primitive(Some(0), None)],
        );
        let streams = derive(&climbing).unwrap();
        assert_eq!(
            streams.bake(&slot(0, "baseColorTexture")),
            ArrayDomain::Face
        );
        assert_eq!(
            streams.primitive_list(U32Id::from_u32(0)),
            [ArrayDomain::Face]
        );

        let below = record(
            vec![material(
                Some(vec![ArrayDomain::Face]),
                vec![value_slot("occlusionTexture", "ao")],
            )],
            vec![primitive(Some(0), None)],
        );
        assert_eq!(failing_element(&below), slot(0, "occlusionTexture"));

        let fixed = record(
            vec![material(
                Some(vec![ArrayDomain::Face]),
                vec![file_slot("metallicRoughnessTexture", "height.png")],
            )],
            vec![primitive(Some(0), None)],
        );
        assert_eq!(failing_element(&fixed), slot(0, "metallicRoughnessTexture"));

        let twice = record(
            vec![material(
                Some(vec![ArrayDomain::Face, ArrayDomain::Face]),
                Vec::new(),
            )],
            vec![primitive(Some(0), None)],
        );
        assert_eq!(
            failing_element(&twice),
            MeshElement::MaterialUvStreams {
                material_id: U32Id::from_u32(0)
            }
        );
    }

    #[test]
    fn a_declared_primitive_list_replaces_the_default_and_covers_its_material() {
        let albedo = || material(None, vec![value_slot("baseColorTexture", "albedo")]);

        let replaced = record(
            vec![albedo()],
            vec![primitive(
                Some(0),
                Some(vec![ArrayDomain::Face, ArrayDomain::Swatch]),
            )],
        );
        assert_eq!(
            derive(&replaced)
                .unwrap()
                .primitive_list(U32Id::from_u32(0)),
            [ArrayDomain::Face, ArrayDomain::Swatch]
        );

        let bare = record(
            vec![albedo()],
            vec![primitive(None, Some(vec![ArrayDomain::Corner]))],
        );
        assert_eq!(
            derive(&bare).unwrap().primitive_list(U32Id::from_u32(0)),
            [ArrayDomain::Corner]
        );

        let streams_element = MeshElement::PrimitiveUvStreams {
            primitive_id: U32Id::from_u32(0),
        };

        let omitting = record(
            vec![albedo()],
            vec![primitive(Some(0), Some(vec![ArrayDomain::Face]))],
        );
        assert_eq!(failing_element(&omitting), streams_element);

        let twice = record(
            vec![albedo()],
            vec![primitive(
                Some(0),
                Some(vec![ArrayDomain::Swatch, ArrayDomain::Swatch]),
            )],
        );
        assert_eq!(failing_element(&twice), streams_element);
    }

    #[test]
    fn the_primitives_drawing_a_material_place_its_streams_alike() {
        let albedo = || material(None, vec![value_slot("baseColorTexture", "albedo")]);

        let agreeing = record(
            vec![albedo()],
            vec![
                primitive(Some(0), Some(vec![ArrayDomain::Swatch, ArrayDomain::Face])),
                primitive(Some(0), None),
            ],
        );
        derive(&agreeing).unwrap();

        let disagreeing = record(
            vec![albedo()],
            vec![
                primitive(Some(0), None),
                primitive(Some(0), Some(vec![ArrayDomain::Face, ArrayDomain::Swatch])),
            ],
        );
        assert_eq!(
            failing_element(&disagreeing),
            MeshElement::PrimitiveUvStreams {
                primitive_id: U32Id::from_u32(1)
            }
        );
    }

    #[test]
    fn a_plain_value_an_unwritten_file_or_an_unknown_material_cannot_bake() {
        let plain = record(
            vec![material(None, vec![value_slot("baseColorTexture", "1.0")])],
            vec![primitive(Some(0), None)],
        );
        assert_eq!(failing_element(&plain), slot(0, "baseColorTexture"));

        let unwritten = record(
            vec![material(
                None,
                vec![file_slot("baseColorTexture", "missing.png")],
            )],
            vec![primitive(Some(0), None)],
        );
        assert_eq!(failing_element(&unwritten), slot(0, "baseColorTexture"));

        let mut plain_png = record(Vec::new(), vec![primitive(None, None)]);
        plain_png.files[0].value = written("1.0");
        assert_eq!(
            failing_element(&plain_png),
            MeshElement::File {
                file: "height.png".to_owned()
            }
        );

        let mut plain_extra = record(Vec::new(), vec![primitive(None, None)]);
        plain_extra.mesh_extras.push(ExtraWrite {
            name: "glow".to_owned(),
            form: ExtraForm::Image,
            source: ExtraSource::Value(written("max(ao)")),
        });
        assert_eq!(
            failing_element(&plain_extra),
            MeshElement::MeshExtra {
                name: "glow".to_owned()
            }
        );

        let unknown = record(Vec::new(), vec![primitive(Some(3), None)]);
        assert_eq!(
            failing_element(&unknown),
            MeshElement::PrimitiveMaterial {
                primitive_id: U32Id::from_u32(0)
            }
        );
    }
}
