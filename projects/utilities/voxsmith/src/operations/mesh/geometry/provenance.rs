use crate::operations::mesh::{ArrayDomain, Computation, ComputedBinding};
use std::collections::BTreeSet;
use vox_value_language::{CheckedProgram, Domain, Reads};

/// Which names follow the emitted faces and which vary from corner to
/// corner of one face, settled over the environment and the program's
/// bindings in order. A value read off a culled pre-pass survives merging
/// only where it depends on neither.
pub(crate) struct Provenance {
    /// The names whose entries change with the faces the mesher emits.
    dependent: BTreeSet<String>,

    /// The names whose corners differ by the voxel geometry alone, as
    /// computed occlusion does.
    corner_varying: BTreeSet<String>,
}

impl Provenance {
    /// Settles the names `computed_bindings` and `checked` bind.
    pub(crate) fn of(computed_bindings: &[ComputedBinding], checked: &CheckedProgram) -> Self {
        let mut provenance = Provenance {
            dependent: BTreeSet::new(),
            corner_varying: BTreeSet::new(),
        };

        for binding in computed_bindings {
            match binding.computation {
                Computation::Index(ArrayDomain::Corner | ArrayDomain::Face) => {
                    provenance.dependent.insert(binding.name.clone());
                }

                Computation::Index(ArrayDomain::Swatch | ArrayDomain::Voxel)
                | Computation::VoxelPosition => {}

                Computation::Occlusion => {
                    provenance.corner_varying.insert(binding.name.clone());
                }
            }
        }

        // A redefinition takes its last binding's standing.
        for (name, expression) in checked.bindings() {
            let reads = expression.reads();
            let dependent = provenance.depends_on_geometry(&reads);
            let corner_varying = provenance.varies_by_corner(&reads);

            for (set, holds) in [
                (&mut provenance.dependent, dependent),
                (&mut provenance.corner_varying, corner_varying),
            ] {
                if holds {
                    set.insert(name.to_owned());
                } else {
                    set.remove(name);
                }
            }
        }

        provenance
    }

    /// Whether a value with `reads` changes with the faces the mesher
    /// emits.
    pub(crate) fn depends_on_geometry(&self, reads: &Reads) -> bool {
        reads
            .reduced
            .iter()
            .any(|domain| matches!(domain, Domain::Corner | Domain::Face))
            || reads.names.iter().any(|name| self.dependent.contains(name))
    }

    /// Whether a value with `reads` differs from corner to corner of one
    /// face by the voxel geometry alone.
    pub(crate) fn varies_by_corner(&self, reads: &Reads) -> bool {
        reads
            .unreduced
            .iter()
            .any(|name| self.corner_varying.contains(name))
    }
}

#[cfg(test)]
mod tests {
    use crate::operations::mesh::{ArrayDomain, Computation, ComputedBinding, Provenance};
    use vox_value_language::{
        Dimension, Domain, Scalar, Type, TypeEnvironment, check, check_expression, parse,
        parse_expression,
    };

    #[test]
    fn bindings_inherit_what_they_read_and_reductions_follow_the_faces() {
        let kind = |domain, scalar| Type {
            domain,
            dimension: Dimension::Vec1,
            scalar,
        };
        let environment = TypeEnvironment {
            types: [
                ("ao", kind(Domain::Corner, Scalar::F32)),
                ("cornerIndex", kind(Domain::Corner, Scalar::U32)),
                ("metallic", kind(Domain::Swatch, Scalar::F32)),
            ]
            .into_iter()
            .map(|(name, kind)| (name.to_owned(), kind))
            .collect(),
        };
        let computed = [
            ComputedBinding {
                name: "ao".to_owned(),
                computation: Computation::Occlusion,
            },
            ComputedBinding {
                name: "cornerIndex".to_owned(),
                computation: Computation::Index(ArrayDomain::Corner),
            },
        ];
        let checked = check(
            parse(
                "soft = lerp(1.0, ao, 0.8); flat = faceAvg(ao); perFace = corner(flat); \
                 mixed = ao * perFace; tint = swatchAvg(flat) * metallic;",
            )
            .unwrap(),
            &environment,
        )
        .unwrap();

        let provenance = Provenance::of(&computed, &checked);

        let standing = |text: &str| {
            let reads = check_expression(&parse_expression(text).unwrap(), &checked)
                .unwrap()
                .reads();
            (
                provenance.depends_on_geometry(&reads),
                provenance.varies_by_corner(&reads),
            )
        };

        assert_eq!(standing("metallic"), (false, false));
        assert_eq!(standing("ao"), (false, true));
        assert_eq!(standing("soft"), (false, true));
        assert_eq!(standing("flat"), (true, false));
        assert_eq!(standing("perFace"), (true, false));
        assert_eq!(standing("mixed"), (true, true));
        assert_eq!(standing("tint"), (true, false));
        assert_eq!(standing("cornerIndex"), (true, false));
        assert_eq!(standing("ao[1u32]"), (true, false));
        assert_eq!(standing("max(ao)"), (true, false));
    }
}
