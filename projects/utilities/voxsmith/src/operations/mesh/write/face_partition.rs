use crate::{
    Error, Result,
    operations::mesh::{Landing, MeshElement, MeshGeometry, MeshRecord, ProgramRun, Swatches},
};
use branded_id::{IdVec, U32Id};
use meshdoc::BMeshPrimitive;
use vox_value_language::{Components, Dimension, Domain, Scalar, eval_expression};

/// The faces each primitive draws. The selects route every face to exactly
/// one primitive.
pub(crate) struct FacePartition {
    faces: IdVec<BMeshPrimitive, Vec<usize>>,
}

impl FacePartition {
    /// Routes `geometry`'s faces by the selects `run` evaluated, each read at
    /// the faces with lower domains climbing in.
    pub(crate) fn derive(
        record: &MeshRecord,
        run: &ProgramRun,
        swatches: &Swatches<'_>,
        geometry: &MeshGeometry,
    ) -> Result<Self> {
        let mut faces: IdVec<BMeshPrimitive, Vec<usize>> =
            IdVec::from_vec(vec![Vec::new(); record.primitives.len()]);

        let mut takers: Vec<Option<U32Id<BMeshPrimitive>>> = vec![None; geometry.quad_count()];

        for checked in &run.destinations {
            let destination = &checked.destination;

            if destination.landing != Landing::Select {
                continue;
            }

            let MeshElement::PrimitiveSelect { primitive_id } = destination.element else {
                unreachable!("a select destination rises from a primitive's select");
            };

            let kind = checked.expression.to_type();

            if kind.scalar != Scalar::Bool || kind.dimension != Dimension::Vec1 {
                return Err(Error::mesh_record(
                    destination.element.clone(),
                    format!("is a {kind}, and a select routes by a bool"),
                ));
            }

            if kind.domain == Domain::Corner {
                return Err(Error::mesh_record(
                    destination.element.clone(),
                    "reads at the corners, and a select routes whole faces",
                ));
            }

            let value = eval_expression(&checked.expression, &run.evaluated)
                .map_err(|error| Error::mesh_record(destination.element.clone(), error))?;

            let Components::Bool(entries) = value.components() else {
                unreachable!("the checker settled bool");
            };

            for (face, taker) in takers.iter_mut().enumerate() {
                if !face_answer(entries, value.domain(), swatches, geometry, face) {
                    continue;
                }

                if let Some(other_id) = *taker {
                    return Err(Error::mesh_record(
                        destination.element.clone(),
                        format!(
                            "takes face {face}, which primitive {} takes too",
                            other_id.to_u32()
                        ),
                    ));
                }

                *taker = Some(primitive_id);
                faces[primitive_id.to_usize_id()].push(face);
            }
        }

        if let Some(face) = takers.iter().position(Option::is_none) {
            return Err(Error::mesh_record(
                MeshElement::Primitives,
                format!("leaves face {face} to no primitive, as no select takes it"),
            ));
        }

        Ok(FacePartition { faces })
    }

    /// The faces primitive `primitive_id` draws, in emission order.
    pub(crate) fn faces(&self, primitive_id: U32Id<BMeshPrimitive>) -> &[usize] {
        &self.faces[primitive_id.to_usize_id()]
    }
}

/// Whether the select with `entries` at `domain` takes `face`. A merged
/// face's voxels agree because the merge rules keep a select to one answer
/// across a span.
fn face_answer(
    entries: &[bool],
    domain: Domain,
    swatches: &Swatches<'_>,
    geometry: &MeshGeometry,
    face: usize,
) -> bool {
    match domain {
        Domain::Corner => unreachable!("a corner select errors before it routes"),

        Domain::Face => entries[face],

        Domain::Plain => entries[0],

        Domain::Swatch | Domain::Voxel => {
            let mut answers = geometry.face_voxel_ids[face].iter().map(|&voxel_id| {
                let voxel_entry_id = swatches.voxel_entry_id(voxel_id);

                let entry = match domain {
                    Domain::Swatch => swatches.voxel_swatch_ids()[voxel_entry_id.to_usize_id()]
                        .to_usize_id()
                        .to_usize(),
                    _ => voxel_entry_id.to_usize_id().to_usize(),
                };

                entries[entry]
            });

            let answer = answers.next().expect("a face covers a voxel");

            assert!(
                answers.all(|other| other == answer),
                "a face's voxels agree on a select"
            );

            answer
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Error,
        operations::mesh::{
            ArrayDomain, Computation, ComputedBinding, FacePartition, MeshElement, MeshRecord,
            Method, PrimitiveRecord, ProgramRun, Swatches, TextureShape, object_to_mesh_geometry,
        },
    };
    use branded_id::{IdVec, U32Id};
    use ty_math::TyVector3U32;
    use voxcore::{VoxMain, VoxObject, VoxPalette, VoxValuePool, material::METALLIC};

    /// A main whose one palette carries `metallic`, and a 2x1x1 bar painted
    /// with its two materials.
    fn painted() -> (VoxMain, VoxObject) {
        let mut main: VoxMain = VoxMain::default();
        let metals = main.retain_value_pool(VoxValuePool::float(vec![1.0, 0.0]).unwrap());

        let mut palette = VoxPalette::default();
        palette
            .retain_property(METALLIC.to_owned(), metals, U32Id::from_u32(0))
            .unwrap();
        palette.retain_material(vec![U32Id::from_u32(0)]).unwrap();
        palette.retain_material(vec![U32Id::from_u32(1)]).unwrap();
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

    /// The culled bar's faces routed by `selects` under `program`, or the
    /// element the routing fails on.
    fn routed(program: &str, selects: &[&str]) -> Result<Vec<Vec<usize>>, MeshElement> {
        let (main, object) = painted();
        let swatches = Swatches::resolve(&main, &object).unwrap();
        let culled = object_to_mesh_geometry(&object, Method::Culled);
        let record = MeshRecord {
            method: Method::Culled,
            texture_shape: TextureShape::Pot,
            voxel_size: 1.0,
            computed_bindings: vec![
                ComputedBinding {
                    name: "ao".to_owned(),
                    computation: Computation::Occlusion,
                },
                ComputedBinding {
                    name: "faceIndex".to_owned(),
                    computation: Computation::Index(ArrayDomain::Face),
                },
            ],
            program: program.to_owned(),
            materials: IdVec::default(),
            primitives: IdVec::from_vec(selects.iter().map(|select| primitive(select)).collect()),
            files: Vec::new(),
            mesh_extras: Vec::new(),
        };
        let run = ProgramRun::over(&object, &swatches, &record, &culled).unwrap();

        match FacePartition::derive(&record, &run, &swatches, &culled) {
            Ok(partition) => Ok((0..selects.len())
                .map(|index| partition.faces(U32Id::from_u32(index as u32)).to_vec())
                .collect()),
            Err(Error::MeshRecord { element, .. }) => Err(element),
            Err(error) => panic!("expected a record error, got {error}"),
        }
    }

    #[test]
    fn each_select_takes_its_faces_and_lower_domains_climb_in() {
        let faces = routed(
            "shiny = metallic > 0; dull = !shiny; low = faceIndex < 5u32;",
            &["shiny", "dull && !low", "dull && low"],
        )
        .unwrap();

        // The culled bar's ten faces alternate between its two voxels. The
        // face bool splits the dull voxel's five again.
        assert_eq!(faces, [vec![0, 2, 4, 6, 8], vec![5, 7, 9], vec![1, 3]]);
    }

    #[test]
    fn a_false_select_takes_nothing_where_the_rest_cover_the_mesh() {
        let faces = routed("", &["false", "true"]).unwrap();

        assert!(faces[0].is_empty());
        assert_eq!(faces[1].len(), 10);
    }

    #[test]
    fn an_unclaimed_or_twice_claimed_face_errors() {
        assert_eq!(routed("", &["false"]).unwrap_err(), MeshElement::Primitives);
        assert_eq!(
            routed("shiny = metallic > 0;", &["true", "shiny"]).unwrap_err(),
            MeshElement::PrimitiveSelect {
                primitive_id: U32Id::from_u32(1)
            }
        );
    }

    #[test]
    fn a_select_that_is_no_face_bool_errors() {
        for select in ["metallic", "ao > 0.5"] {
            assert_eq!(
                routed("", &[select]).unwrap_err(),
                MeshElement::PrimitiveSelect {
                    primitive_id: U32Id::from_u32(0)
                },
                "{select}"
            );
        }
    }
}
