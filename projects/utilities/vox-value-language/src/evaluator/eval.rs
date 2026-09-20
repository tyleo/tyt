use crate::{
    CheckedProgram, Error, EvaluatedProgram, Result, ValueEnvironment,
    evaluator::{eval_node, validate_environment},
};

/// Evaluates a checked program over the environment's values, computing
/// every binding in order.
pub fn eval(program: &CheckedProgram, environment: &ValueEnvironment) -> Result<EvaluatedProgram> {
    let lengths = validate_environment(program, environment)?;
    let mut values = environment.values.clone();

    for binding in &program.bindings {
        let value = eval_node(
            &binding.expression,
            &values,
            &environment.groupings,
            &lengths,
        )
        .map_err(|failure| Error::Eval {
            binding: Some(binding.name.clone()),
            failure,
        })?;

        values.insert(binding.name.clone(), value);
    }

    Ok(EvaluatedProgram {
        values,
        groupings: environment.groupings.clone(),
        lengths,
    })
}

#[cfg(test)]
mod tests {
    use crate::{
        BFace, BSwatch, BVoxel, CheckedProgram, Components, Dimension, Domain, Error, EvalFailure,
        Groupings, Scalar, Type, TypeEnvironment, ValueEnvironment, check, eval,
        evaluator::{
            assert_close, bools, f32s, groupings, lamp, run, step, strings, types_of, u8s, u32s,
        },
        parse,
    };
    use branded_id::{IdVec, U32Id};
    use std::collections::HashMap;

    fn ty(domain: Domain, dimension: Dimension, scalar: Scalar) -> Type {
        Type {
            domain,
            dimension,
            scalar,
        }
    }

    #[test]
    fn bindings_evaluate_in_order_and_later_ones_read_earlier_ones() {
        let (_, evaluated) = run(
            "tint = baseColorFactor.rgb; dim = tint * 0.5; dark = dim.r < 0.3;",
            &lamp(),
        )
        .unwrap();

        assert_close(
            evaluated.get("tint").unwrap(),
            &[0.5, 0.5, 0.5, 1.0, 0.9, 0.6],
        );
        assert_close(
            evaluated.get("dim").unwrap(),
            &[0.25, 0.25, 0.25, 0.5, 0.45, 0.3],
        );
        assert_eq!(
            evaluated.get("dark").unwrap(),
            &bools(Domain::Swatch, &[true, false])
        );
    }

    #[test]
    fn a_binding_redefines_a_name_let_style() {
        let (_, evaluated) = run(
            "roughnessFactor = pow(roughnessFactor, 2); count = count * 2u32;",
            &lamp(),
        )
        .unwrap();

        assert_close(evaluated.get("roughnessFactor").unwrap(), &[0.81, 0.16]);
        assert_eq!(
            evaluated.get("count").unwrap(),
            &u32s(Domain::Swatch, Dimension::Vec1, &[6, 14])
        );
        assert_eq!(evaluated.get("missing"), None);
    }

    #[test]
    fn the_end_scope_answers_environment_names_too() {
        let (_, evaluated) = run("", &lamp()).unwrap();

        assert_eq!(
            evaluated.get("tag").unwrap(),
            &strings(Domain::Swatch, &["steel", "glass"])
        );
    }

    #[test]
    fn an_eval_error_names_its_binding() {
        let error = run("a = 1.0; b = count / 0; c = 2.0;", &lamp()).unwrap_err();

        assert_eq!(
            error,
            Error::Eval {
                binding: Some("b".to_owned()),
                failure: EvalFailure::DivisionByZero
            }
        );
        assert_eq!(error.to_string(), "in `b`: division by zero");
    }

    #[test]
    fn the_lamp_bakes_as_the_pages_show() {
        let (_, evaluated) = run(
            "occlusionStrength = swatch(default(occlusionStrength, 1));
             albedo = baseColorFactor;
             orm = rgb(occlusionStrength, roughnessFactor, metallicFactor);
             maxStrength = max(emissiveStrength);
             emissive = emissiveFactor * emissiveStrength / max(maxStrength, 0.001);
             white = rgb(1, 1, 1);
             smoothness = 1 - roughnessFactor;
             mse = rgb(metallicFactor, smoothness, emissiveStrength / max(maxStrength, 0.001));
             heat = step(0.001, emissiveStrength);
             accent = avg(baseColorFactor.rgb);
             glowing = emissiveStrength > 0;
             solid = !glowing;
             opaqueWhite = rgba(1, 1, 1, 1);
             mode = mix(\"OPAQUE\", \"BLEND\", min(baseColorFactor.a) < 1);
             glass = tag == \"glass\";
             palette = u8(swatchIndex);
             rawEmissive = emissiveFactor * emissiveStrength;",
            &with_swatch_index(lamp()),
        )
        .unwrap();
        let get = |name: &str| evaluated.get(name).unwrap();

        assert_close(get("occlusionStrength"), &[1.0, 1.0]);
        assert_close(get("albedo"), &[0.5, 0.5, 0.5, 1.0, 1.0, 0.9, 0.6, 0.6]);
        assert_close(get("orm"), &[1.0, 0.9, 1.0, 1.0, 0.4, 0.0]);
        assert_close(get("maxStrength"), &[4.0]);
        assert_close(get("emissive"), &[0.0, 0.0, 0.0, 1.0, 0.9, 0.6]);
        assert_close(get("white"), &[1.0, 1.0, 1.0]);
        assert_close(get("mse"), &[1.0, 0.1, 0.0, 0.0, 0.6, 1.0]);
        assert_close(get("heat"), &[0.0, 1.0]);
        assert_close(get("accent"), &[0.75, 0.7, 0.55]);
        assert_eq!(get("glowing"), &bools(Domain::Swatch, &[false, true]));
        assert_eq!(get("solid"), &bools(Domain::Swatch, &[true, false]));
        assert_close(get("opaqueWhite"), &[1.0, 1.0, 1.0, 1.0]);
        assert_eq!(get("mode"), &strings(Domain::Plain, &["BLEND"]));
        assert_eq!(get("glass"), &bools(Domain::Swatch, &[false, true]));
        assert_eq!(
            get("palette"),
            &u8s(Domain::Swatch, Dimension::Vec1, &[0, 1])
        );
        assert_close(get("rawEmissive"), &[0.0, 0.0, 0.0, 4.0, 3.6, 2.4]);
    }

    #[test]
    fn the_step_shades_its_crease_as_the_pages_show() {
        let (_, evaluated) = run(
            "ao = faceAvg(computedOcclusion);
             aoVoxel = voxelAvg(ao);
             faceCount = swatchSum(face(1u32));
             aoSwatch = swatchSum(ao) / f32(max(faceCount, 1));
             crevice = ao < 10;
             open = !crevice;
             height = f32(voxelHeight) / f32(max(max(voxelHeight), 1));
             bands = mod(voxelHeight, 2);
             albedo = baseColorFactor * lerp(0.8, 1, f32(bands));",
            &with_voxel_height(step()),
        )
        .unwrap();
        let get = |name: &str| evaluated.get(name).unwrap();

        assert_close(
            get("ao"),
            &(0..10)
                .map(|face| (4 * face) as f32 + 1.5)
                .collect::<Vec<_>>(),
        );
        assert_close(
            get("aoVoxel"),
            &[
                (1.5 + 5.5 + 9.5 + 17.5) / 4.0,
                (1.5 + 13.5 + 21.5 + 29.5 + 33.5) / 5.0,
                (5.5 + 9.5 + 17.5 + 25.5 + 37.5) / 5.0,
            ],
        );
        assert_eq!(
            get("faceCount"),
            &u32s(Domain::Swatch, Dimension::Vec1, &[14])
        );
        assert_close(
            get("aoSwatch"),
            &[
                (0..10).map(|face| (4 * face) as f32 + 1.5).sum::<f32>() / 14.0
                    + (5.5 + 9.5 + 17.5 + 1.5) / 14.0,
            ],
        );
        assert_eq!(
            get("crevice"),
            &bools(
                Domain::Face,
                &[
                    true, true, true, false, false, false, false, false, false, false
                ]
            )
        );
        assert_eq!(
            get("open"),
            &bools(
                Domain::Face,
                &[
                    false, false, false, true, true, true, true, true, true, true
                ]
            )
        );
        assert_close(get("height"), &[0.0, 0.0, 1.0]);
        assert_eq!(
            get("bands"),
            &u32s(Domain::Voxel, Dimension::Vec1, &[0, 0, 1])
        );
        assert_close(
            get("albedo"),
            &[
                0.44, 0.4, 0.36, 0.8, 0.44, 0.4, 0.36, 0.8, 0.55, 0.5, 0.45, 1.0,
            ],
        );
    }

    fn with_swatch_index(mut environment: ValueEnvironment) -> ValueEnvironment {
        environment.values.insert(
            "swatchIndex".to_owned(),
            u32s(Domain::Swatch, Dimension::Vec1, &[0, 1]),
        );

        environment
    }

    fn with_voxel_height(mut environment: ValueEnvironment) -> ValueEnvironment {
        environment.values.insert(
            "voxelHeight".to_owned(),
            u32s(Domain::Voxel, Dimension::Vec1, &[0, 0, 1]),
        );

        environment
    }

    // Environment validation.

    fn checked_lamp(types: &TypeEnvironment) -> CheckedProgram {
        check(parse("").unwrap(), types).unwrap()
    }

    #[test]
    fn the_values_must_match_the_types_the_program_was_checked_with() {
        let environment = lamp();
        let mut types = types_of(&environment);

        types.types.insert(
            "extra".to_owned(),
            ty(Domain::Plain, Dimension::Vec1, Scalar::F32),
        );
        assert_eq!(
            eval(&checked_lamp(&types), &environment),
            Err(Error::MissingValue {
                name: "extra".to_owned()
            })
        );

        let mut types = types_of(&environment);

        types.types.insert(
            "count".to_owned(),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::F32),
        );
        assert_eq!(
            eval(&checked_lamp(&types), &environment),
            Err(Error::ValueType {
                name: "count".to_owned(),
                expected: ty(Domain::Swatch, Dimension::Vec1, Scalar::F32),
                found: ty(Domain::Swatch, Dimension::Vec1, Scalar::U32)
            })
        );

        let mut types = types_of(&environment);

        types.types.remove("count");
        assert_eq!(
            eval(&checked_lamp(&types), &environment),
            Err(Error::UnexpectedValue {
                name: "count".to_owned()
            })
        );
    }

    #[test]
    fn every_array_holds_its_domains_length() {
        let mut environment = lamp();

        environment.values.insert(
            "count".to_owned(),
            u32s(Domain::Swatch, Dimension::Vec1, &[1, 2, 3]),
        );
        assert_eq!(
            eval(&checked_lamp(&types_of(&environment)), &environment),
            Err(Error::EntryCount {
                name: "count".to_owned(),
                expected: 2,
                found: 3
            })
        );

        let mut environment = lamp();

        environment.values.insert(
            "computedOcclusion".to_owned(),
            f32s(Domain::Corner, Dimension::Vec1, &[0.5; 36]),
        );
        assert_eq!(
            eval(&checked_lamp(&types_of(&environment)), &environment),
            Err(Error::EntryCount {
                name: "computedOcclusion".to_owned(),
                expected: 40,
                found: 36
            })
        );
    }

    #[test]
    fn an_f32_input_must_be_finite() {
        let mut environment = lamp();

        environment.values.insert(
            "roughnessFactor".to_owned(),
            f32s(Domain::Swatch, Dimension::Vec1, &[0.5, f32::NAN]),
        );
        assert_eq!(
            eval(&checked_lamp(&types_of(&environment)), &environment),
            Err(Error::NonFiniteInput {
                name: "roughnessFactor".to_owned()
            })
        );
    }

    #[test]
    fn the_groupings_fix_the_lengths_and_must_hold_together() {
        let environment = ValueEnvironment {
            values: HashMap::new(),
            groupings: groupings(&[0, 1], &[&[0], &[]]),
        };

        assert_eq!(
            eval(&checked_lamp(&types_of(&environment)), &environment),
            Err(Error::FacePieces { face: 1 })
        );

        let environment = ValueEnvironment {
            values: HashMap::new(),
            groupings: groupings(&[0, 1], &[&[0], &[2]]),
        };

        assert_eq!(
            eval(&checked_lamp(&types_of(&environment)), &environment),
            Err(Error::PieceVoxel { face: 1, voxel: 2 })
        );

        let environment = ValueEnvironment {
            values: [(
                "sparse".to_owned(),
                f32s(
                    Domain::Swatch,
                    Dimension::Vec1,
                    &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
                ),
            )]
            .into_iter()
            .collect(),
            groupings: Groupings {
                voxel_swatches: IdVec::<BVoxel, U32Id<BSwatch>>::from(vec![U32Id::from_u32(5)]),
                face_voxels: IdVec::<BFace, Vec<U32Id<BVoxel>>>::default(),
            },
        };
        let (_, evaluated) = run(
            "top = swatchSum(voxel(sparse)); n = max(sparse);",
            &environment,
        )
        .unwrap();

        assert_close(
            evaluated.get("top").unwrap(),
            &[0.0, 0.0, 0.0, 0.0, 0.0, 6.0],
        );
        assert_close(evaluated.get("n").unwrap(), &[6.0]);
        assert_eq!(
            evaluated.get("sparse").unwrap().components(),
            &Components::F32(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0])
        );
    }
}
