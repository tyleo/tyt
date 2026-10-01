use crate::{Expression, SyntaxBinding};
use std::collections::BTreeSet;

/// A parsed program, the input to `check`.
#[derive(Clone, Debug, PartialEq)]
pub struct Program {
    pub(crate) bindings: Vec<SyntaxBinding>,
}

impl Program {
    /// The names the environment supplies: each name a binding reads before
    /// the program binds it, and each name `expressions` read that the
    /// program never binds.
    ///
    /// # Arguments
    /// - `expressions`: read in the scope at the program's end, as under
    ///   `check_expression`.
    pub fn free_names<'a>(
        &self,
        expressions: impl IntoIterator<Item = &'a Expression>,
    ) -> BTreeSet<String> {
        let mut bound = BTreeSet::new();
        let mut free = BTreeSet::new();

        for binding in &self.bindings {
            binding.expression.gather_free_names(&bound, &mut free);
            bound.insert(binding.name.clone());
        }

        for expression in expressions {
            expression.root.gather_free_names(&bound, &mut free);
        }

        free
    }
}

#[cfg(test)]
mod tests {
    use crate::{parse, parse_expression};
    use std::collections::BTreeSet;

    fn free_names(program: &str, expressions: &[&str]) -> BTreeSet<String> {
        let expressions: Vec<_> = expressions
            .iter()
            .map(|text| parse_expression(text).unwrap())
            .collect();

        parse(program).unwrap().free_names(&expressions)
    }

    fn names(names: &[&str]) -> BTreeSet<String> {
        names.iter().map(|&name| name.to_owned()).collect()
    }

    #[test]
    fn a_name_read_before_its_binding_is_free() {
        assert_eq!(
            free_names(
                "tint = baseColor.rgb * 0.5; roughness = pow(roughness, 2); dim = tint;",
                &[]
            ),
            names(&["baseColor", "roughness"])
        );
    }

    #[test]
    fn a_name_read_after_its_binding_is_bound() {
        assert_eq!(
            free_names("rust = tag == \"rust\"; wear = mix(0.0, 1.0, rust);", &[]),
            names(&["tag"])
        );
    }

    #[test]
    fn a_default_reads_its_name() {
        assert_eq!(
            free_names("scale = default(emissiveStrength, x);", &[]),
            names(&["emissiveStrength", "x"])
        );
    }

    #[test]
    fn expressions_read_the_end_scope() {
        assert_eq!(
            free_names(
                "rust = tag == \"rust\";",
                &["mix(roughness, 0.9, rust)", "max(metallic[swatchIndex])"]
            ),
            names(&["metallic", "roughness", "swatchIndex", "tag"])
        );
    }

    #[test]
    fn functions_and_swizzles_are_not_names() {
        assert_eq!(
            free_names(
                "",
                &["abs(-voxel(baseColor).a) > 1u8 && true || \"x\" != \"y\""]
            ),
            names(&["baseColor"])
        );
    }
}
