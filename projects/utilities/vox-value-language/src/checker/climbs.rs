use crate::{
    CheckedExpression,
    checker::{CheckedKind, CheckedNode},
};

/// Collects into `lifted` every array value below `node` that its context
/// lifts to a higher domain. A reduction's operand and an index's parts
/// are read where they stand, and everything else lifts to its parent's
/// domain.
pub(crate) fn climbs(node: &CheckedNode, lifted: &mut Vec<CheckedExpression>) {
    let paired: Vec<&CheckedNode> = match &node.kind {
        CheckedKind::Binary { left, right, .. }
        | CheckedKind::Comparison { left, right, .. }
        | CheckedKind::Fold { left, right, .. }
        | CheckedKind::Logical { left, right, .. } => vec![left, right],

        CheckedKind::Bool(_)
        | CheckedKind::Name(_)
        | CheckedKind::Number(_)
        | CheckedKind::StringLiteral(_) => Vec::new(),

        CheckedKind::Call { arguments, .. } => arguments.iter().collect(),

        CheckedKind::Climb { operand, .. }
        | CheckedKind::Convert { operand, .. }
        | CheckedKind::Unary { operand, .. } => vec![operand],

        CheckedKind::Default {
            name,
            bound,
            fallback,
        } => {
            if let Some(kind) = bound
                && kind.domain.is_array()
                && kind.domain < node.output.domain
            {
                lifted.push(CheckedExpression {
                    root: CheckedNode {
                        kind: CheckedKind::Name(name.clone()),
                        output: *kind,
                    },
                });
            }

            vec![fallback]
        }

        CheckedKind::Index { source, index } => {
            climbs(source, lifted);
            climbs(index, lifted);

            Vec::new()
        }

        CheckedKind::Mix {
            first,
            second,
            chooser,
        } => vec![first, second, chooser],

        CheckedKind::Reduce { operand, .. } => {
            climbs(operand, lifted);

            Vec::new()
        }

        CheckedKind::Swizzle { source, .. } => vec![source],
    };

    for child in paired {
        if child.output.domain.is_array() && child.output.domain < node.output.domain {
            lifted.push(CheckedExpression {
                root: child.clone(),
            });
        }

        climbs(child, lifted);
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        CheckedProgram, Dimension, Domain, Scalar, Type, TypeEnvironment, check, check_expression,
        parse, parse_expression,
    };

    fn program() -> CheckedProgram {
        let kind = |domain, dimension| Type {
            domain,
            dimension,
            scalar: Scalar::F32,
        };
        let environment = TypeEnvironment {
            types: [
                ("albedo", kind(Domain::Swatch, Dimension::Vec4)),
                ("bands", kind(Domain::Voxel, Dimension::Vec1)),
                ("metallic", kind(Domain::Swatch, Dimension::Vec1)),
                ("occlusion", kind(Domain::Corner, Dimension::Vec1)),
            ]
            .into_iter()
            .map(|(name, kind)| (name.to_owned(), kind))
            .collect(),
        };

        check(parse("ao = faceAvg(occlusion);").unwrap(), &environment).unwrap()
    }

    fn climbs(text: &str) -> Vec<String> {
        check_expression(&parse_expression(text).unwrap(), &program())
            .unwrap()
            .climbs()
            .iter()
            .map(|climb| format!("{} {}", climb.to_type().domain, climb.root.render()))
            .collect()
    }

    #[test]
    fn an_operand_below_its_pairing_lifts() {
        assert!(climbs("albedo.a * metallic").is_empty());
        assert_eq!(climbs("albedo.a * ao"), ["swatch (. `albedo` 3)"]);
        assert_eq!(
            climbs("lerp(albedo.a, 1.0, f32(bands)) * occlusion"),
            [
                "voxel (lerp (. `albedo` 3) 1f32 (f32 `bands`))",
                "swatch (. `albedo` 3)"
            ]
        );
        assert_eq!(
            climbs("metallic > 0.5 && bands == 1.0"),
            ["swatch (> `metallic` 0.5f32)"]
        );
    }

    #[test]
    fn climb_functions_and_bound_defaults_lift_and_reductions_and_indexes_do_not() {
        assert_eq!(climbs("face(bands)"), ["voxel `bands`"]);
        assert_eq!(
            climbs("corner(face(metallic))"),
            ["face (face `metallic`)", "swatch `metallic`"]
        );
        assert_eq!(
            climbs("default(metallic, 0.5) * ao"),
            ["swatch (default `metallic` 0.5f32)"]
        );
        assert_eq!(climbs("default(metallic, ao)"), ["swatch `metallic`"]);
        assert!(climbs("default(missing, 0.5) * ao").is_empty());
        assert!(climbs("swatchAvg(faceAvg(occlusion)) * metallic").is_empty());
        assert!(climbs("voxelAvg(ao * 2.0) * bands").is_empty());
        assert!(climbs("face(metallic[1u32]) + max(bands)").is_empty());
        assert_eq!(
            climbs("face(max(metallic, 0.5))"),
            ["swatch (max `metallic` 0.5f32)"]
        );
    }
}
