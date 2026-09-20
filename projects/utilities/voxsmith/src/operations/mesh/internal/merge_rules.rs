use crate::{
    Error, Result,
    operations::mesh::{
        ArrayDomain, FaceSpan, Landing, MeshElement, MeshGeometry, MeshRecord, ProgramRun,
        Provenance, Swatches,
    },
};
use branded_id::{IdVec, U32Id};
use std::collections::HashMap;
use ty_math::TyVector3U32;
use vox_value_language::{
    BVoxel, CheckedExpression, Components, Domain, Scalar, Value, eval_expression,
};
use voxcore::{BVoxVoxel, VoxObject};

/// The slack a corner value may keep from the blend that reproduces it.
/// `f32` thirds blend within it.
const BLEND_TOLERANCE: f64 = 1e-4;

/// What greedy meshing may merge, read off the run's values over the culled
/// geometry. Each voxel takes a class, and a span merges within one class
/// where every corner value the mesh carries is the bilinear blend of the
/// span's four outer corners.
pub(crate) struct MergeRules<'a> {
    object: &'a VoxObject,
    swatches: &'a Swatches<'a>,
    culled: &'a MeshGeometry,
    classes: IdVec<BVoxel, u32>,
    corner_values: Vec<Value>,
    culled_faces: HashMap<(U32Id<BVoxVoxel>, usize, i32), usize>,
}

impl<'a> MergeRules<'a> {
    /// Derives the rules from `record` and `run`, the program run over
    /// `culled`. A value that follows the emitted faces cannot be read off
    /// the culled pre-pass, so a lifted one keeps a span to one swatch or
    /// one voxel instead of agreeing and a corner one caps merging.
    pub(crate) fn derive(
        object: &'a VoxObject,
        record: &MeshRecord,
        swatches: &'a Swatches<'a>,
        culled: &'a MeshGeometry,
        run: &ProgramRun,
    ) -> Result<Self> {
        let provenance = Provenance::of(&record.computed_bindings, &run.checked);

        let mut gathered = Gathered::default();

        for (_, expression) in run.checked.bindings() {
            for lifted in expression.climbs() {
                gathered.lift(&lifted, &provenance, run, &MeshElement::Program)?;
            }
        }

        for checked in &run.destinations {
            let destination = &checked.destination;
            let domain = checked.expression.to_type().domain;

            for lifted in checked.expression.climbs() {
                gathered.lift(&lifted, &provenance, run, &destination.element)?;
            }

            match destination.landing {
                // The writer reads a select at the faces and an attribute at
                // the corners.
                Landing::Attribute | Landing::Select => {
                    gathered.lift(&checked.expression, &provenance, run, &destination.element)?;
                }

                Landing::Json | Landing::Texture => {}
            }

            match (destination.landing, domain) {
                (Landing::Attribute | Landing::Texture, Domain::Corner) => {
                    let reads = checked.expression.reads();

                    if !provenance.varies_by_corner(&reads) {
                        // One value per face, which the final face defines.
                    } else if provenance.depends_on_geometry(&reads) {
                        gathered.capped = true;
                    } else {
                        let value = eval_expression(&checked.expression, &run.evaluated).map_err(
                            |error| Error::mesh_record(destination.element.clone(), error),
                        )?;

                        if value.scalar() == Scalar::String {
                            return Err(Error::mesh_record(
                                destination.element.clone(),
                                "holds strings, which no attribute or texture takes",
                            ));
                        }

                        gathered.corner_values.push(value);
                    }
                }

                (Landing::Texture, Domain::Swatch) => gathered.same_swatch = true,

                (Landing::Texture, Domain::Voxel) => gathered.capped = true,

                _ => {}
            }
        }

        let declared = record
            .materials
            .iter()
            .flat_map(|material| &material.uv_streams)
            .chain(
                record
                    .primitives
                    .iter()
                    .flat_map(|primitive| &primitive.uv_streams),
            )
            .flatten();

        for domain in declared {
            match domain {
                ArrayDomain::Corner | ArrayDomain::Face => {}
                ArrayDomain::Swatch => gathered.same_swatch = true,
                ArrayDomain::Voxel => gathered.capped = true,
            }
        }

        let mut strings = HashMap::new();
        let mut keys = HashMap::new();

        let classes = swatches
            .voxel_swatch_ids()
            .iter()
            .enumerate()
            .map(|(voxel, swatch_id)| {
                let mut key = Vec::new();

                if gathered.capped {
                    key.push(u32::try_from(voxel).expect("the voxel count fits u32"));
                } else {
                    if gathered.same_swatch {
                        key.push(swatch_id.to_u32());
                    }

                    for value in &gathered.agreed {
                        let entry = match value.domain() {
                            Domain::Swatch => swatch_id.to_usize_id().to_usize(),
                            Domain::Voxel => voxel,
                            _ => unreachable!("only swatch and voxel values agree"),
                        };

                        key.extend(entry_key(value, entry, &mut strings));
                    }
                }

                let next = u32::try_from(keys.len()).expect("the class count fits u32");

                *keys.entry(key).or_insert(next)
            })
            .collect();

        let culled_faces = if gathered.corner_values.is_empty() {
            HashMap::new()
        } else {
            culled
                .face_voxel_ids
                .iter()
                .enumerate()
                .map(|(face, voxel_ids)| {
                    let [voxel_id] = voxel_ids.as_slice() else {
                        unreachable!("a culled face covers one voxel");
                    };

                    let normal = culled.normals[face * 4].to_array();

                    let d = (0..3)
                        .find(|&axis| normal[axis] != 0.0)
                        .expect("a face normal lies along one axis");

                    let sign = if normal[d] > 0.0 { 1 } else { -1 };

                    ((*voxel_id, d, sign), face)
                })
                .collect()
        };

        Ok(MergeRules {
            object,
            swatches,
            culled,
            classes,
            corner_values: gathered.corner_values,
            culled_faces,
        })
    }

    /// The class of the live voxel `voxel_id`.
    pub(crate) fn voxel_class(&self, voxel_id: U32Id<BVoxVoxel>) -> u32 {
        self.classes[self.swatches.voxel_entry_id(voxel_id).to_usize_id()]
    }

    /// Whether every corner value the mesh carries is, at each corner the
    /// span covers, the bilinear blend of the span's four outer corners.
    pub(crate) fn span_fits(&self, span: &FaceSpan) -> bool {
        if self.corner_values.is_empty() {
            return true;
        }

        let (u, v) = (span.u(), span.v());
        let (u0, u1) = (span.u0 as f32, span.u1 as f32);
        let (v0, v1) = (span.v0 as f32, span.v1 as f32);

        let outer = [
            self.corner_entry(span, [span.u0, span.v0], [u0, v0]),
            self.corner_entry(span, [span.u1 - 1, span.v0], [u1, v0]),
            self.corner_entry(span, [span.u0, span.v1 - 1], [u0, v1]),
            self.corner_entry(span, [span.u1 - 1, span.v1 - 1], [u1, v1]),
        ];

        for cell in span.cells() {
            let face = self.culled_face(span, cell);

            for corner in face * 4..face * 4 + 4 {
                let position = self.culled.positions[corner].to_array();
                let along_u = f64::from((position[u] - u0) / (u1 - u0));
                let along_v = f64::from((position[v] - v0) / (v1 - v0));

                for value in &self.corner_values {
                    let width = value.dimension().width();

                    for component in 0..width {
                        let at = |entry: usize| {
                            component_f64(value.components(), entry * width + component)
                        };

                        let blend = lerp(
                            lerp(at(outer[0]), at(outer[1]), along_u),
                            lerp(at(outer[2]), at(outer[3]), along_u),
                            along_v,
                        );

                        if (at(corner) - blend).abs() > BLEND_TOLERANCE {
                            return false;
                        }
                    }
                }
            }
        }

        true
    }

    /// The corner entry of the culled face at `cell` sitting at `at` along
    /// `(u, v)`.
    fn corner_entry(&self, span: &FaceSpan, cell: [usize; 2], at: [f32; 2]) -> usize {
        let face = self.culled_face(span, span.cell(cell[0], cell[1]));

        (face * 4..face * 4 + 4)
            .find(|&corner| {
                let position = self.culled.positions[corner].to_array();
                position[span.u()] == at[0] && position[span.v()] == at[1]
            })
            .expect("a face has a corner at each end of its cell")
    }

    /// The culled face of the cell at `position` on `span`'s side.
    fn culled_face(&self, span: &FaceSpan, position: [u32; 3]) -> usize {
        let voxel_id = self
            .object
            .voxel_id(TyVector3U32::from_array(position))
            .expect("a span covers cells within the grid");

        *self
            .culled_faces
            .get(&(voxel_id, span.d, span.sign))
            .expect("greedy covers only the faces culled emitted")
    }
}

/// The rules as they accumulate over the program and the destinations.
#[derive(Default)]
struct Gathered {
    same_swatch: bool,
    capped: bool,
    agreed: Vec<Value>,
    corner_values: Vec<Value>,
}

impl Gathered {
    /// Takes in a value `expression` lifts above its domain, erroring from
    /// `element` where it fails to evaluate.
    fn lift(
        &mut self,
        expression: &CheckedExpression,
        provenance: &Provenance,
        run: &ProgramRun,
        element: &MeshElement,
    ) -> Result<()> {
        let domain = expression.to_type().domain;

        // A face value lifts onto its own corners and a plain one is the
        // same everywhere, so neither can disagree across a span.
        if !matches!(domain, Domain::Swatch | Domain::Voxel) {
            return Ok(());
        }

        if provenance.depends_on_geometry(&expression.reads()) {
            match domain {
                Domain::Swatch => self.same_swatch = true,
                _ => self.capped = true,
            }
        } else {
            let value = eval_expression(expression, &run.evaluated)
                .map_err(|error| Error::mesh_record(element.clone(), error))?;

            self.agreed.push(value);
        }

        Ok(())
    }
}

/// The entry's components as class key words, strings interned through
/// `strings`.
fn entry_key(value: &Value, entry: usize, strings: &mut HashMap<String, u32>) -> Vec<u32> {
    let width = value.dimension().width();
    let range = entry * width..(entry + 1) * width;

    match value.components() {
        Components::F32(components) => components[range]
            .iter()
            .map(|&component| {
                if component == 0.0 {
                    0
                } else {
                    component.to_bits()
                }
            })
            .collect(),
        Components::U8(components) => components[range]
            .iter()
            .map(|&component| u32::from(component))
            .collect(),
        Components::U16(components) => components[range]
            .iter()
            .map(|&component| u32::from(component))
            .collect(),
        Components::U32(components) => components[range].to_vec(),
        Components::Bool(components) => components[range]
            .iter()
            .map(|&component| u32::from(component))
            .collect(),
        Components::String(components) => components[range]
            .iter()
            .map(|component| {
                let next = u32::try_from(strings.len()).expect("the string count fits u32");

                *strings.entry(component.clone()).or_insert(next)
            })
            .collect(),
    }
}

fn component_f64(components: &Components, index: usize) -> f64 {
    match components {
        Components::F32(components) => f64::from(components[index]),
        Components::U8(components) => f64::from(components[index]),
        Components::U16(components) => f64::from(components[index]),
        Components::U32(components) => f64::from(components[index]),
        Components::Bool(components) => f64::from(u8::from(components[index])),
        Components::String(_) => {
            unreachable!("a string corner value errors before the rules hold it")
        }
    }
}

fn lerp(from: f64, to: f64, along: f64) -> f64 {
    from + (to - from) * along
}

#[cfg(test)]
mod tests {
    use crate::operations::mesh::{
        ArrayDomain, AttributeWrite, Computation, ComputedBinding, ExtraForm, ExtraSource,
        ExtraWrite, FileForm, FileWrite, MaterialRecord, MergeRules, MeshGeometry, MeshRecord,
        Method, PrimitiveRecord, ProgramRun, SlotSource, SlotWrite, Swatches, TextureShape,
        Transfer, WrittenValue, mesh_slices,
    };
    use branded_id::{IdVec, U32Id};
    use ty_math::TyVector3U32;
    use voxcore::{
        VoxMain, VoxObject, VoxPalette, VoxValuePool,
        material::{BASE_COLOR, METALLIC},
    };

    /// A main whose one palette carries `baseColor` and `metallic`, and a
    /// 2x1x1 bar painted with its two materials, which share an alpha of
    /// one and nothing else.
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

        let mut object = VoxObject::new("bar".to_owned(), TyVector3U32::new(2, 1, 1)).unwrap();
        object.retain_layer(palette_id, U32Id::from_u32(0));
        for x in 0..2 {
            let voxel_id = object.voxel_id(TyVector3U32::new(x, 0, 0)).unwrap();
            object
                .retain_voxel(voxel_id, &[U32Id::from_u32(x)])
                .unwrap();
        }

        (main, object)
    }

    fn written(expression: &str) -> WrittenValue {
        WrittenValue {
            expression: expression.to_owned(),
            transfer: Transfer::Linear,
        }
    }

    fn primitive(select: &str) -> PrimitiveRecord {
        PrimitiveRecord {
            material_id: None,
            select: select.to_owned(),
            name: None,
            normal: true,
            uv_streams: None,
            attributes: Vec::new(),
        }
    }

    fn record(program: &str) -> MeshRecord {
        MeshRecord {
            method: Method::Greedy,
            texture_shape: TextureShape::Pot,
            voxel_size: 1.0,
            computed_bindings: vec![ComputedBinding {
                name: "ao".to_owned(),
                computation: Computation::Occlusion,
            }],
            program: program.to_owned(),
            materials: IdVec::default(),
            primitives: IdVec::from_vec(vec![primitive("true")]),
            files: Vec::new(),
            mesh_extras: Vec::new(),
        }
    }

    /// The record with `expression` written as a custom attribute.
    fn with_attribute(program: &str, expression: &str) -> MeshRecord {
        let mut record = record(program);
        record.primitives[U32Id::from_u32(0).to_usize_id()]
            .attributes
            .push(AttributeWrite::Custom {
                name: "_VALUE".to_owned(),
                value: written(expression),
            });
        record
    }

    /// Whether the bar's two voxels share a class under `record`.
    fn merges(record: &MeshRecord) -> bool {
        let (main, object) = painted();
        let swatches = Swatches::resolve(&main, &object).unwrap();
        let culled = mesh_slices(&object, Method::Culled, &|_| 0, &|_| true, false);
        let run = ProgramRun::over(&object, &swatches, record, &culled).unwrap();

        let rules = MergeRules::derive(&object, record, &swatches, &culled, &run).unwrap();

        let class = |x| rules.voxel_class(object.voxel_id(TyVector3U32::new(x, 0, 0)).unwrap());
        class(0) == class(1)
    }

    #[test]
    fn a_lifted_value_has_to_agree_and_a_reduced_one_does_not() {
        assert!(merges(&record("")));
        assert!(merges(&record("x = swatchAvg(faceAvg(ao)) * metallic;")));
        assert!(merges(&record("x = face(baseColor.a);")));
        assert!(merges(&record("x = face(baseColor.a * metallic * 0.0);")));
        assert!(!merges(&record("x = face(metallic);")));
        assert!(!merges(&record("x = metallic; y = x * ao;")));
    }

    #[test]
    fn a_lifted_value_that_follows_the_faces_keeps_one_swatch_or_one_voxel() {
        // The bar's swatches average to the same open occlusion, yet the
        // pre-pass cannot vouch for the final value, so the swatch stands in.
        assert!(!merges(&record("s = swatchAvg(faceAvg(ao)); x = face(s);")));
        assert!(!merges(&record(
            "v = voxelAvg(faceAvg(ao)); x = face(v * 0.0);"
        )));
    }

    #[test]
    fn a_swatch_texture_keeps_a_span_to_one_swatch() {
        let mut texture = record("");
        texture.materials = IdVec::from_vec(vec![MaterialRecord {
            slots: vec![SlotWrite {
                property: "baseColorTexture".to_owned(),
                source: SlotSource::Value("baseColor.a".to_owned()),
            }],
            ..Default::default()
        }]);
        assert!(!merges(&texture));

        let mut png = record("");
        png.files.push(FileWrite {
            file: "alpha.png".to_owned(),
            value: written("baseColor.a"),
            form: FileForm::Png,
        });
        assert!(!merges(&png));

        let mut factor = record("");
        factor.materials = IdVec::from_vec(vec![MaterialRecord {
            slots: vec![SlotWrite {
                property: "baseColorFactor".to_owned(),
                source: SlotSource::Value("avg(baseColor)".to_owned()),
            }],
            ..Default::default()
        }]);
        assert!(merges(&factor));
    }

    #[test]
    fn a_voxel_texture_or_stream_caps_merging_at_the_voxel() {
        let mut texture = record("");
        texture.mesh_extras.push(ExtraWrite {
            name: "glow".to_owned(),
            form: ExtraForm::Image,
            source: ExtraSource::Value(written("voxel(1.0)")),
        });
        assert!(!merges(&texture));

        let mut stream = record("");
        stream.primitives[U32Id::from_u32(0).to_usize_id()].uv_streams =
            Some(vec![ArrayDomain::Voxel]);
        assert!(!merges(&stream));

        let mut swatch_stream = record("");
        swatch_stream.materials = IdVec::from_vec(vec![MaterialRecord {
            uv_streams: Some(vec![ArrayDomain::Swatch]),
            ..Default::default()
        }]);
        assert!(!merges(&swatch_stream));
    }

    #[test]
    fn selects_attributes_and_json_lift_their_values() {
        let mut select = record("");
        select.primitives = IdVec::from_vec(vec![primitive("metallic > 0.5")]);
        assert!(!merges(&select));

        let mut alpha = record("");
        alpha.primitives = IdVec::from_vec(vec![primitive("baseColor.a > 0.5")]);
        assert!(merges(&alpha));

        assert!(!merges(&with_attribute("", "metallic")));

        let mut json = record("");
        json.mesh_extras.push(ExtraWrite {
            name: "metals".to_owned(),
            form: ExtraForm::Json,
            source: ExtraSource::Value(written("face(metallic)")),
        });
        assert!(!merges(&json));
    }

    /// The bar of three along x beside a two-high column, meshed greedy
    /// under `record`'s rules, with the count of its top faces at z = 1.
    fn tops_beside_a_column(record: &MeshRecord) -> (MeshGeometry, usize) {
        let main: VoxMain = VoxMain::default();
        let mut object = VoxObject::new("bar".to_owned(), TyVector3U32::new(4, 1, 2)).unwrap();
        for [x, z] in [[0, 0], [1, 0], [2, 0], [3, 0], [3, 1]] {
            let voxel_id = object.voxel_id(TyVector3U32::new(x, 0, z)).unwrap();
            object.retain_voxel(voxel_id, &[]).unwrap();
        }
        let swatches = Swatches::resolve(&main, &object).unwrap();

        let culled = mesh_slices(&object, Method::Culled, &|_| 0, &|_| true, false);
        let run = ProgramRun::over(&object, &swatches, record, &culled).unwrap();
        let rules = MergeRules::derive(&object, record, &swatches, &culled, &run).unwrap();

        let geometry = mesh_slices(
            &object,
            Method::Greedy,
            &|voxel_id| rules.voxel_class(voxel_id),
            &|span| rules.span_fits(span),
            false,
        );

        let tops = (0..geometry.quad_count())
            .filter(|&quad| {
                geometry.normals[quad * 4].to_array() == [0.0, 0.0, 1.0]
                    && geometry.positions[quad * 4].to_array()[2] == 1.0
            })
            .count();

        (geometry, tops)
    }

    #[test]
    fn a_corner_value_splits_a_span_its_blend_cannot_reproduce() {
        // The column's upper voxel shades the bar's top at x = 3, so the
        // top's corners read 1 up to x = 2 and 2/3 at x = 3. A blend from
        // x = 0 to x = 3 misses the 1 at x = 1, which stops the merge at
        // x = 2.
        let (_, bare) = tops_beside_a_column(&record(""));
        assert_eq!(bare, 1);

        let (geometry, tops) = tops_beside_a_column(&with_attribute("", "ao"));
        assert_eq!(tops, 2);

        // The bottom, open everywhere, still merges whole.
        let bottoms = (0..geometry.quad_count())
            .filter(|&quad| geometry.normals[quad * 4].to_array() == [0.0, 0.0, -1.0])
            .count();
        assert_eq!(bottoms, 1);
    }

    #[test]
    fn a_corner_value_the_final_face_defines_merges_and_a_mixed_one_caps() {
        // One flat value per face comes from the merged face itself.
        let (_, flat) =
            tops_beside_a_column(&with_attribute("flat = faceAvg(ao);", "corner(flat)"));
        assert_eq!(flat, 1);

        // Per-corner occlusion scaled by a per-face value has no pre-pass
        // reading that survives merging, so every face stays a voxel's.
        let (_, mixed) =
            tops_beside_a_column(&with_attribute("flat = faceAvg(ao);", "ao * corner(flat)"));
        assert_eq!(mixed, 3);
    }
}
