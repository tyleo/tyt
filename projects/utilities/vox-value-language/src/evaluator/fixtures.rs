use crate::{
    BFace, BSwatch, BVoxel, CheckedProgram, Components, Dimension, Domain, EvaluatedProgram,
    Groupings, Result, TypeEnvironment, Value, ValueEnvironment, check, check_expression, eval,
    eval_expression, parse, parse_expression,
};
use branded_id::{IdVec, U32Id};
use std::collections::HashMap;

/// The lamp: two swatches, a steel base under a glowing glass bulb, two
/// voxels, and ten faces of one voxel each.
pub(crate) fn lamp() -> ValueEnvironment {
    let mut environment = ValueEnvironment {
        values: HashMap::new(),
        groupings: groupings(
            &[0, 1],
            &[&[0], &[0], &[0], &[0], &[0], &[1], &[1], &[1], &[1], &[1]],
        ),
    };

    environment.values = [
        (
            "baseColorFactor",
            f32s(
                Domain::Swatch,
                Dimension::Vec4,
                &[0.5, 0.5, 0.5, 1.0, 1.0, 0.9, 0.6, 0.6],
            ),
        ),
        (
            "roughnessFactor",
            f32s(Domain::Swatch, Dimension::Vec1, &[0.9, 0.4]),
        ),
        (
            "metallicFactor",
            f32s(Domain::Swatch, Dimension::Vec1, &[1.0, 0.0]),
        ),
        (
            "emissiveFactor",
            f32s(
                Domain::Swatch,
                Dimension::Vec3,
                &[0.0, 0.0, 0.0, 1.0, 0.9, 0.6],
            ),
        ),
        (
            "emissiveStrength",
            f32s(Domain::Swatch, Dimension::Vec1, &[0.0, 4.0]),
        ),
        ("tag", strings(Domain::Swatch, &["steel", "glass"])),
        ("flag", bools(Domain::Swatch, &[false, true])),
        ("count", u32s(Domain::Swatch, Dimension::Vec1, &[3, 7])),
        (
            "wide",
            u8s(Domain::Voxel, Dimension::Vec2, &[200, 100, 50, 25]),
        ),
        (
            "voxelPosition",
            u32s(Domain::Voxel, Dimension::Vec3, &[0, 0, 0, 0, 1, 0]),
        ),
        (
            "faceValue",
            f32s(
                Domain::Face,
                Dimension::Vec1,
                &(0..10).map(|face| face as f32).collect::<Vec<_>>(),
            ),
        ),
        (
            "computedOcclusion",
            f32s(
                Domain::Corner,
                Dimension::Vec1,
                &(0..40)
                    .map(|corner| corner as f32 / 40.0)
                    .collect::<Vec<_>>(),
            ),
        ),
        (
            "unit",
            f32s(Domain::Plain, Dimension::Vec3, &[1.0, 0.0, 0.0]),
        ),
    ]
    .into_iter()
    .map(|(name, value)| (name.to_owned(), value))
    .collect();

    environment
}

/// The step: one stone swatch, three voxels in an L, and ten greedy faces,
/// the bottom and the left side merged across two voxels each and the front
/// and back split into a tall quad and a short one.
pub(crate) fn step() -> ValueEnvironment {
    let mut environment = ValueEnvironment {
        values: HashMap::new(),
        groupings: groupings(
            &[0, 0, 0],
            &[
                &[0, 1],
                &[0, 2],
                &[0, 2],
                &[1],
                &[0, 2],
                &[1],
                &[2],
                &[1],
                &[1],
                &[2],
            ],
        ),
    };

    environment.values = [
        (
            "baseColorFactor",
            f32s(Domain::Swatch, Dimension::Vec4, &[0.55, 0.5, 0.45, 1.0]),
        ),
        (
            "height",
            f32s(Domain::Voxel, Dimension::Vec1, &[0.0, 0.0, 1.0]),
        ),
        (
            "faceValue",
            f32s(
                Domain::Face,
                Dimension::Vec1,
                &(0..10).map(|face| face as f32).collect::<Vec<_>>(),
            ),
        ),
        (
            "computedOcclusion",
            f32s(
                Domain::Corner,
                Dimension::Vec1,
                &(0..40).map(|corner| corner as f32).collect::<Vec<_>>(),
            ),
        ),
    ]
    .into_iter()
    .map(|(name, value)| (name.to_owned(), value))
    .collect();

    environment
}

/// Two voxels of two swatches, the second voxel buried with no faces.
pub(crate) fn buried() -> ValueEnvironment {
    let mut environment = ValueEnvironment {
        values: HashMap::new(),
        groupings: groupings(&[0, 1], &[&[0], &[0], &[0]]),
    };

    environment.values = [(
        "faceValue".to_owned(),
        f32s(Domain::Face, Dimension::Vec1, &[1.0, 2.0, 3.0]),
    )]
    .into_iter()
    .collect();

    environment
}

/// No voxels at all: every array domain is empty.
pub(crate) fn empty() -> ValueEnvironment {
    ValueEnvironment {
        values: [(
            "faceValue".to_owned(),
            f32s(Domain::Face, Dimension::Vec1, &[]),
        )]
        .into_iter()
        .collect(),
        groupings: Groupings::default(),
    }
}

/// The groupings from each voxel's swatch and each face's voxel pieces.
pub(crate) fn groupings(voxel_swatches: &[u32], face_voxels: &[&[u32]]) -> Groupings {
    Groupings {
        voxel_swatches: IdVec::<BVoxel, U32Id<BSwatch>>::from(
            voxel_swatches
                .iter()
                .map(|&swatch| U32Id::from_u32(swatch))
                .collect::<Vec<_>>(),
        ),
        face_voxels: IdVec::<BFace, Vec<U32Id<BVoxel>>>::from(
            face_voxels
                .iter()
                .map(|pieces| pieces.iter().map(|&voxel| U32Id::from_u32(voxel)).collect())
                .collect::<Vec<_>>(),
        ),
    }
}

/// The type environment every value in the environment implies.
pub(crate) fn types_of(environment: &ValueEnvironment) -> TypeEnvironment {
    TypeEnvironment {
        types: environment
            .values
            .iter()
            .map(|(name, value)| (name.clone(), value.to_type()))
            .collect(),
    }
}

/// Checks and evaluates the program over the environment.
pub(crate) fn run(
    text: &str,
    environment: &ValueEnvironment,
) -> Result<(CheckedProgram, EvaluatedProgram)> {
    let checked = check(parse(text)?, &types_of(environment))?;
    let evaluated = eval(&checked, environment)?;

    Ok((checked, evaluated))
}

/// Evaluates one expression in the environment's scope.
pub(crate) fn evaluate(text: &str, environment: &ValueEnvironment) -> Result<Value> {
    let (checked, evaluated) = run("", environment)?;
    let expression = check_expression(&parse_expression(text)?, &checked)?;

    eval_expression(&expression, &evaluated)
}

pub(crate) fn f32s(domain: Domain, dimension: Dimension, values: &[f32]) -> Value {
    Value::new(domain, dimension, Components::F32(values.to_vec())).unwrap()
}

pub(crate) fn u8s(domain: Domain, dimension: Dimension, values: &[u8]) -> Value {
    Value::new(domain, dimension, Components::U8(values.to_vec())).unwrap()
}

pub(crate) fn u16s(domain: Domain, dimension: Dimension, values: &[u16]) -> Value {
    Value::new(domain, dimension, Components::U16(values.to_vec())).unwrap()
}

pub(crate) fn u32s(domain: Domain, dimension: Dimension, values: &[u32]) -> Value {
    Value::new(domain, dimension, Components::U32(values.to_vec())).unwrap()
}

pub(crate) fn wide_bools(domain: Domain, dimension: Dimension, values: &[bool]) -> Value {
    Value::new(domain, dimension, Components::Bool(values.to_vec())).unwrap()
}

pub(crate) fn bools(domain: Domain, values: &[bool]) -> Value {
    Value::new(domain, Dimension::Vec1, Components::Bool(values.to_vec())).unwrap()
}

pub(crate) fn strings(domain: Domain, values: &[&str]) -> Value {
    Value::new(
        domain,
        Dimension::Vec1,
        Components::String(values.iter().map(|value| (*value).to_owned()).collect()),
    )
    .unwrap()
}

/// Asserts two `f32` lists agree within a small tolerance.
pub(crate) fn assert_close(found: &Value, expected: &[f32]) {
    let Components::F32(found) = found.components() else {
        panic!("{found:?} is not f32");
    };

    assert_eq!(
        found.len(),
        expected.len(),
        "{found:?} against {expected:?}"
    );

    for (found, expected) in found.iter().zip(expected) {
        assert!(
            (found - expected).abs() <= 1e-5 * expected.abs().max(1.0),
            "{found} is not {expected}"
        );
    }
}
