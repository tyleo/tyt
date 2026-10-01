use crate::{CheckedBinding, CheckedProgram, Error, Program, Result, TypeEnvironment, check_root};

/// Checks a program against the names the environment supplies, settling
/// every binding's type in order.
///
/// # Arguments
/// - `environment`: the types of the names the program reads but never
///   defines.
pub fn check(program: Program, environment: &TypeEnvironment) -> Result<CheckedProgram> {
    let mut scope = environment.types.clone();
    let mut bindings = Vec::with_capacity(program.bindings.len());

    for binding in program.bindings {
        let expression =
            check_root(&binding.expression, &scope).map_err(|failure| Error::Check {
                binding: Some(binding.name.clone()),
                failure,
            })?;

        scope.insert(binding.name.clone(), expression.output);
        bindings.push(CheckedBinding {
            name: binding.name,
            expression,
        });
    }

    Ok(CheckedProgram {
        environment: environment.clone(),
        bindings,
        scope,
    })
}

#[cfg(test)]
mod tests {
    use crate::{
        CheckFailure, CheckedProgram, Dimension, Domain, Error, Scalar, Type, TypeEnvironment,
        check, check_expression, parse, parse_expression, ty,
    };

    fn environment(names: &[(&str, Domain, Dimension, Scalar)]) -> TypeEnvironment {
        TypeEnvironment {
            types: names
                .iter()
                .map(|(name, domain, dimension, scalar)| {
                    ((*name).to_owned(), ty(*domain, *dimension, *scalar))
                })
                .collect(),
        }
    }

    /// The names the page examples read: the palette properties, a tag, and
    /// the computed values.
    fn palette() -> TypeEnvironment {
        environment(&[
            ("baseColor", Domain::Swatch, Dimension::Vec4, Scalar::F64),
            ("roughness", Domain::Swatch, Dimension::Vec1, Scalar::F64),
            ("metallic", Domain::Swatch, Dimension::Vec1, Scalar::F64),
            (
                "emissiveColor",
                Domain::Swatch,
                Dimension::Vec3,
                Scalar::F64,
            ),
            (
                "emissiveStrength",
                Domain::Swatch,
                Dimension::Vec1,
                Scalar::F64,
            ),
            ("tag", Domain::Swatch, Dimension::Vec1, Scalar::String),
            (
                "computedOcclusion",
                Domain::Corner,
                Dimension::Vec1,
                Scalar::F64,
            ),
            ("voxelPosition", Domain::Voxel, Dimension::Vec3, Scalar::U32),
            ("swatchIndex", Domain::Swatch, Dimension::Vec1, Scalar::U32),
        ])
    }

    fn checked(text: &str, environment: &TypeEnvironment) -> CheckedProgram {
        match check(parse(text).unwrap(), environment) {
            Ok(checked) => checked,
            Err(error) => panic!("{text} failed: {error}"),
        }
    }

    fn get(program: &CheckedProgram, name: &str) -> Type {
        *program
            .get(name)
            .unwrap_or_else(|| panic!("{name} is unbound"))
    }

    #[test]
    fn bindings_settle_in_order_and_later_ones_read_earlier_ones() {
        let program = checked(
            "tint = baseColor.rgb; dim = tint * 0.5; dark = dim.r < 0.2;",
            &palette(),
        );

        assert_eq!(
            get(&program, "tint"),
            ty(Domain::Swatch, Dimension::Vec3, Scalar::F64)
        );
        assert_eq!(
            get(&program, "dim"),
            ty(Domain::Swatch, Dimension::Vec3, Scalar::F64)
        );
        assert_eq!(
            get(&program, "dark"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::Bool)
        );
        assert_eq!(
            check(parse("dim = tint * 0.5; tint = 1.0;").unwrap(), &palette()),
            Err(Error::Check {
                binding: Some("dim".to_owned()),
                failure: CheckFailure::UnknownName {
                    name: "tint".to_owned()
                }
            })
        );
    }

    #[test]
    fn a_binding_redefines_a_name_let_style() {
        let program = checked(
            "roughness = pow(roughness, 2); count = 1u32; count = f64(count) * 0.5;",
            &palette(),
        );

        assert_eq!(
            get(&program, "roughness"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::F64)
        );
        assert_eq!(
            get(&program, "count"),
            ty(Domain::Plain, Dimension::Vec1, Scalar::F64)
        );
        assert_eq!(program.bindings.len(), 3);
    }

    #[test]
    fn the_end_scope_answers_environment_names_too() {
        let program = checked("", &palette());

        assert_eq!(
            get(&program, "baseColor"),
            ty(Domain::Swatch, Dimension::Vec4, Scalar::F64)
        );
        assert_eq!(program.get("missing"), None);
        assert!(program.bindings.is_empty());
    }

    #[test]
    fn a_check_error_names_its_binding() {
        assert_eq!(
            check(
                parse("a = 1.0; b = baseColor + 1; c = 2.0;").unwrap(),
                &palette()
            ),
            Err(Error::Check {
                binding: Some("b".to_owned()),
                failure: CheckFailure::DimensionMismatch {
                    operation: "+".to_owned(),
                    found: vec![Dimension::Vec4, Dimension::Vec1]
                }
            })
        );
        assert_eq!(
            check(parse("x = 1;").unwrap(), &palette()),
            Err(Error::Check {
                binding: Some("x".to_owned()),
                failure: CheckFailure::UntypedLiteral
            })
        );
        assert_eq!(
            check(parse("x = 1;").unwrap(), &palette())
                .unwrap_err()
                .to_string(),
            "in `x`: a bare whole-number literal takes its type from context, and nothing here fixes one"
        );
    }

    #[test]
    fn the_empty_program_and_the_empty_statement_check() {
        assert!(checked("", &palette()).bindings.is_empty());
        assert_eq!(checked(";; a = 1.0;;", &palette()).bindings.len(), 1);
    }

    #[test]
    fn an_expression_checks_in_the_end_scope() {
        let program = checked("ao = faceAvg(computedOcclusion);", &palette());
        let select = parse_expression("faceAvg(ao) < 0.7").unwrap();

        assert_eq!(
            check_expression(&select, &program).unwrap_err(),
            Error::Check {
                binding: None,
                failure: CheckFailure::ReductionSource {
                    operation: "faceAvg".to_owned(),
                    found: Domain::Face
                }
            }
        );

        let select = parse_expression("ao < 0.7").unwrap();

        assert_eq!(
            check_expression(&select, &program).unwrap().to_type(),
            ty(Domain::Face, Dimension::Vec1, Scalar::Bool)
        );
    }

    #[test]
    fn the_defaults_profile_fills_what_the_palette_lacks() {
        let program = checked(
            "baseColor = swatch(default(baseColor, rgba(1, 1, 1, 1)));
             occlusionStrength = swatch(default(occlusionStrength, 1));
             roughness = swatch(default(roughness, 1));
             metallic = swatch(default(metallic, 1));
             emissiveColor = swatch(default(emissiveColor, rgb(0, 0, 0)));
             emissiveStrength = swatch(default(emissiveStrength, 1));",
            &palette(),
        );

        assert_eq!(
            get(&program, "baseColor"),
            ty(Domain::Swatch, Dimension::Vec4, Scalar::F64)
        );
        assert_eq!(
            get(&program, "occlusionStrength"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::F64)
        );
        assert_eq!(
            get(&program, "emissiveColor"),
            ty(Domain::Swatch, Dimension::Vec3, Scalar::F64)
        );
        assert_eq!(
            program.bindings[1].expression.render(),
            "(swatch (default `occlusionStrength`? 1f64))"
        );
        assert_eq!(
            program.bindings[2].expression.render(),
            "(swatch (default `roughness` 1f64))"
        );
    }

    #[test]
    fn the_built_in_profiles_check() {
        let program = checked(
            "occlusionStrength = swatch(default(occlusionStrength, 1));
             albedo = baseColor;
             orm = rgb(occlusionStrength, roughness, metallic);
             maxStrength = max(emissiveStrength);
             emissive = emissiveColor * emissiveStrength / max(maxStrength, 0.001);
             white = rgb(1, 1, 1);",
            &palette(),
        );

        assert_eq!(
            get(&program, "albedo"),
            ty(Domain::Swatch, Dimension::Vec4, Scalar::F64)
        );
        assert_eq!(
            get(&program, "orm"),
            ty(Domain::Swatch, Dimension::Vec3, Scalar::F64)
        );
        assert_eq!(
            get(&program, "maxStrength"),
            ty(Domain::Plain, Dimension::Vec1, Scalar::F64)
        );
        assert_eq!(
            get(&program, "emissive"),
            ty(Domain::Swatch, Dimension::Vec3, Scalar::F64)
        );
        assert_eq!(
            get(&program, "white"),
            ty(Domain::Plain, Dimension::Vec3, Scalar::F64)
        );
    }

    #[test]
    fn the_user_profiles_check() {
        let program = checked(
            "smoothness = 1 - roughness;
             maxStrength = max(emissiveStrength);
             mse = rgb(metallic, smoothness, emissiveStrength / max(maxStrength, 0.001));
             heat = step(0.001, emissiveStrength);
             accent = avg(baseColor.rgb);
             ao = max(computedOcclusion, 0.2);
             glowing = emissiveStrength > 0;
             solid = !glowing;
             albedo = baseColor;
             opaqueWhite = rgba(1, 1, 1, 1);
             aoFace = faceAvg(computedOcclusion);
             crevice = aoFace < 0.9;
             open = !crevice;
             mode = mix(\"OPAQUE\", \"BLEND\", min(baseColor.a) < 1);
             glass = tag == \"glass\";
             palette = u8(swatchIndex);",
            &palette(),
        );

        assert_eq!(
            get(&program, "mse"),
            ty(Domain::Swatch, Dimension::Vec3, Scalar::F64)
        );
        assert_eq!(
            get(&program, "heat"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::F64)
        );
        assert_eq!(
            get(&program, "accent"),
            ty(Domain::Plain, Dimension::Vec3, Scalar::F64)
        );
        assert_eq!(
            get(&program, "ao"),
            ty(Domain::Corner, Dimension::Vec1, Scalar::F64)
        );
        assert_eq!(
            get(&program, "solid"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::Bool)
        );
        assert_eq!(
            get(&program, "opaqueWhite"),
            ty(Domain::Plain, Dimension::Vec4, Scalar::F64)
        );
        assert_eq!(
            get(&program, "crevice"),
            ty(Domain::Face, Dimension::Vec1, Scalar::Bool)
        );
        assert_eq!(
            get(&program, "mode"),
            ty(Domain::Plain, Dimension::Vec1, Scalar::String)
        );
        assert_eq!(
            get(&program, "glass"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::Bool)
        );
        assert_eq!(
            get(&program, "palette"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::U8)
        );
    }

    #[test]
    fn the_mesh_page_programs_check() {
        let program = checked(
            "crevice = faceAvg(computedOcclusion) < 0.7;
             bands = mod(voxelPosition.y, 2);
             albedo = baseColor * lerp(0.8, 1, f64(bands));
             rawEmissive = emissiveColor * emissiveStrength;
             height = f64(voxelPosition.y) / f64(max(max(voxelPosition.y), 1));
             aoFace = faceAvg(computedOcclusion);
             faceCount = swatchSum(face(1u32));
             ao = swatchSum(aoFace) / f64(max(faceCount, 1));
             lab = oklabFromRgb(baseColor.rgb);
             reddish = distance(lab, oklabFromRgb(rgb(1, 0, 0))) < 0.25;
             darker = rgbFromOklab(lab * rgb(0.8, 1, 1));
             hue = mod(oklchFromRgb(baseColor.rgb).z + 0.1, 1);",
            &palette(),
        );

        assert_eq!(
            get(&program, "crevice"),
            ty(Domain::Face, Dimension::Vec1, Scalar::Bool)
        );
        assert_eq!(
            get(&program, "bands"),
            ty(Domain::Voxel, Dimension::Vec1, Scalar::U32)
        );
        assert_eq!(
            get(&program, "albedo"),
            ty(Domain::Voxel, Dimension::Vec4, Scalar::F64)
        );
        assert_eq!(
            get(&program, "rawEmissive"),
            ty(Domain::Swatch, Dimension::Vec3, Scalar::F64)
        );
        assert_eq!(
            get(&program, "height"),
            ty(Domain::Voxel, Dimension::Vec1, Scalar::F64)
        );
        assert_eq!(
            get(&program, "faceCount"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::U32)
        );
        assert_eq!(
            get(&program, "ao"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::F64)
        );
        assert_eq!(
            get(&program, "reddish"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::Bool)
        );
        assert_eq!(
            get(&program, "darker"),
            ty(Domain::Swatch, Dimension::Vec3, Scalar::F64)
        );
        assert_eq!(
            get(&program, "hue"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::F64)
        );
    }
}
