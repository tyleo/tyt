use crate::{
    Domain,
    checker::{CheckedKind, CheckedNode},
};
use std::collections::BTreeSet;

/// The names an expression reads and the arrays it gathers entries from.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Reads {
    /// Every name read.
    pub names: BTreeSet<String>,

    /// The names whose entries reach the result whole, with no reduction or
    /// index between.
    pub unreduced: BTreeSet<String>,

    /// The domains of the arrays a reduction or an index gathers from.
    pub reduced: BTreeSet<Domain>,
}

impl Reads {
    /// Gathers what `node` reads, `reduced` marking a reduction or an index
    /// above it.
    pub(crate) fn gather(&mut self, node: &CheckedNode, reduced: bool) {
        let mut name = |name: &str| {
            self.names.insert(name.to_owned());

            if !reduced {
                self.unreduced.insert(name.to_owned());
            }
        };

        match &node.kind {
            CheckedKind::Binary { left, right, .. }
            | CheckedKind::Comparison { left, right, .. }
            | CheckedKind::Fold { left, right, .. }
            | CheckedKind::Logical { left, right, .. } => {
                self.gather(left, reduced);
                self.gather(right, reduced);
            }

            CheckedKind::Bool(_) | CheckedKind::Number(_) | CheckedKind::StringLiteral(_) => {}

            CheckedKind::Call { arguments, .. } => {
                for argument in arguments {
                    self.gather(argument, reduced);
                }
            }

            CheckedKind::Climb { operand, .. }
            | CheckedKind::Convert { operand, .. }
            | CheckedKind::Unary { operand, .. } => self.gather(operand, reduced),

            CheckedKind::Default {
                name: bound_name,
                bound,
                fallback,
            } => {
                if bound.is_some() {
                    name(bound_name);
                }

                self.gather(fallback, reduced);
            }

            CheckedKind::Index { source, index } => {
                if source.output.domain.is_array() {
                    self.reduced.insert(source.output.domain);
                }

                self.gather(source, true);
                self.gather(index, true);
            }

            CheckedKind::Mix {
                first,
                second,
                chooser,
            } => {
                self.gather(first, reduced);
                self.gather(second, reduced);
                self.gather(chooser, reduced);
            }

            CheckedKind::Name(read) => name(read),

            CheckedKind::Reduce { operand, .. } => {
                self.reduced.insert(operand.output.domain);
                self.gather(operand, true);
            }

            CheckedKind::Swizzle { source, .. } => self.gather(source, reduced),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        CheckedProgram, Dimension, Domain, Reads, Scalar, Type, TypeEnvironment, check,
        check_expression, parse, parse_expression,
    };
    use std::collections::BTreeSet;

    fn program() -> CheckedProgram {
        let kind = |domain| Type {
            domain,
            dimension: Dimension::Vec1,
            scalar: Scalar::F32,
        };
        let environment = TypeEnvironment {
            types: [
                ("bands", kind(Domain::Voxel)),
                ("metallic", kind(Domain::Swatch)),
                ("occlusion", kind(Domain::Corner)),
            ]
            .into_iter()
            .map(|(name, kind)| (name.to_owned(), kind))
            .collect(),
        };

        check(parse("ao = faceAvg(occlusion);").unwrap(), &environment).unwrap()
    }

    fn reads_of(text: &str) -> Reads {
        check_expression(&parse_expression(text).unwrap(), &program())
            .unwrap()
            .reads()
    }

    fn names<'a>(names: impl IntoIterator<Item = &'a str>) -> BTreeSet<String> {
        names.into_iter().map(str::to_owned).collect()
    }

    #[test]
    fn reductions_and_indexes_set_a_name_apart_from_the_result() {
        let reads = reads_of("lerp(metallic, default(bands, 0.0), swatchAvg(faceAvg(occlusion)))");

        assert_eq!(reads.names, names(["bands", "metallic", "occlusion"]));
        assert_eq!(reads.unreduced, names(["bands", "metallic"]));
        assert_eq!(
            reads.reduced,
            [Domain::Corner, Domain::Face].into_iter().collect()
        );

        let indexed = reads_of("ao[1u32] + max(bands) + default(missing, 1.0)");

        assert_eq!(indexed.names, names(["ao", "bands"]));
        assert!(indexed.unreduced.is_empty());
        assert_eq!(
            indexed.reduced,
            [Domain::Voxel, Domain::Face].into_iter().collect()
        );
    }

    #[test]
    fn the_program_lists_its_bindings_in_order() {
        let bindings: Vec<(String, Domain)> = program()
            .bindings()
            .map(|(name, expression)| (name.to_owned(), expression.to_type().domain))
            .collect();

        assert_eq!(bindings, [("ao".to_owned(), Domain::Face)]);
    }
}
