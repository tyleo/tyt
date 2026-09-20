use crate::{
    Error, Result,
    commands::{ProfileSet, parse_fragment},
};
use std::collections::HashSet;
use voxsmith::operations::mesh::{Computation, ComputedBinding};

/// Joins the program from its fragments and gathers the computed bindings,
/// the flags' first and then each landed profile's. A profile lands once, at
/// its first arrival, its `valuesFrom` imports depth-first ahead of it.
pub(crate) struct ProgramBuilder<'a> {
    profiles: Option<&'a ProfileSet>,
    computed: Vec<ComputedBinding>,
    hand_computed: usize,
    fragments: Vec<String>,
    landed: HashSet<String>,
}

impl<'a> ProgramBuilder<'a> {
    /// A builder over `profiles` holding the flags' `computed` bindings.
    pub(crate) fn new(profiles: Option<&'a ProfileSet>, computed: Vec<ComputedBinding>) -> Self {
        ProgramBuilder {
            profiles,
            hand_computed: computed.len(),
            computed,
            fragments: Vec::new(),
            landed: HashSet::new(),
        }
    }

    /// Appends the `--value` fragment `text`.
    pub(crate) fn push_value(&mut self, text: &str) -> Result<()> {
        self.fragments.push(parse_fragment("--value", text)?);

        Ok(())
    }

    /// Lands the profile `name`, which `origin` asks for, with its imports.
    pub(crate) fn land_profile(&mut self, origin: &str, name: &str) -> Result<()> {
        self.land(origin, name, &mut Vec::new())
    }

    /// The joined program and the computed bindings.
    pub(crate) fn finish(self) -> (String, Vec<ComputedBinding>) {
        (self.fragments.join("\n"), self.computed)
    }

    fn land(&mut self, origin: &str, name: &str, visiting: &mut Vec<String>) -> Result<()> {
        if let Some(start) = visiting.iter().position(|visited| visited == name) {
            let chain: Vec<_> = visiting[start..]
                .iter()
                .map(String::as_str)
                .chain([name])
                .map(|name| format!("`{name}`"))
                .collect();

            return Err(Error::usage(format!(
                "the profile `{name}`'s valuesFrom cycles: {}",
                chain.join(" imports ")
            )));
        }

        if self.landed.contains(name) {
            return Ok(());
        }

        let profile = self
            .profiles
            .expect("a profile flag loads the profiles")
            .get(origin, name)?;

        visiting.push(name.to_owned());

        for import in &profile.values_from {
            self.land(
                &format!("the profile `{name}`'s valuesFrom"),
                import,
                visiting,
            )?;
        }

        visiting.pop();

        for (domain, bound) in profile.compute_index.bindings() {
            self.push_profile_computed(name, bound, Computation::Index(domain))?;
        }

        for bound in &profile.compute_occlusion.0 {
            self.push_profile_computed(name, bound, Computation::Occlusion)?;
        }

        for bound in &profile.compute_voxel_position.0 {
            self.push_profile_computed(name, bound, Computation::VoxelPosition)?;
        }

        for (position, fragment) in (0..).zip(&profile.values) {
            self.fragments.push(parse_fragment(
                &format!("the profile `{name}`'s values entry {position}"),
                fragment,
            )?);
        }

        self.landed.insert(name.to_owned());

        Ok(())
    }

    /// Adds a computed binding of the profile `name`. A name the flags
    /// compute keeps the flags' binding, and one another profile computes
    /// errors.
    fn push_profile_computed(
        &mut self,
        name: &str,
        bound: &str,
        computation: Computation,
    ) -> Result<()> {
        if self.computed[..self.hand_computed]
            .iter()
            .any(|binding| binding.name == bound)
        {
            return Ok(());
        }

        if self.computed.iter().any(|binding| binding.name == bound) {
            return Err(Error::usage(format!(
                "the profile `{name}` computes `{bound}`, which another profile computes already"
            )));
        }

        self.computed.push(ComputedBinding {
            name: bound.to_owned(),
            computation,
        });

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::ProgramBuilder;
    use crate::commands::{Profile, ProfileSet};
    use std::collections::BTreeMap;
    use voxsmith::operations::mesh::{ArrayDomain, Computation, ComputedBinding};

    /// A set of the built-ins under the profiles `entries` defines as json.
    fn profiles(entries: &[(&str, &str)]) -> ProfileSet {
        let profiles: BTreeMap<String, Profile> = entries
            .iter()
            .map(|(name, json)| ((*name).to_owned(), serde_json::from_str(json).unwrap()))
            .collect();

        ProfileSet::from_profiles(profiles)
    }

    #[test]
    fn imports_land_depth_first_and_each_profile_once() {
        let profiles = ProfileSet::built_in();
        let mut builder = ProgramBuilder::new(Some(&profiles), Vec::new());

        builder.land_profile("--profile", "pbr").unwrap();
        builder.land_profile("--values-from", "orm").unwrap();

        let (program, _) = builder.finish();
        let bindings: Vec<_> = program
            .lines()
            .map(|line| line.split(" = ").next().unwrap())
            .collect();

        assert_eq!(
            bindings,
            [
                "baseColorFactor",
                "occlusionStrength",
                "roughnessFactor",
                "metallicFactor",
                "emissiveFactor",
                "emissiveStrength",
                "albedo",
                "orm",
                "maxStrength",
                "emissive",
                "white",
            ]
        );
    }

    #[test]
    fn values_append_at_their_position() {
        let profiles = ProfileSet::built_in();
        let mut builder = ProgramBuilder::new(Some(&profiles), Vec::new());

        builder.push_value("a = 1").unwrap();
        builder.land_profile("--values-from", "albedo").unwrap();
        builder.push_value("b = a").unwrap();

        let (program, _) = builder.finish();
        assert!(program.starts_with("a = 1;\nbaseColorFactor"), "{program}");
        assert!(
            program.ends_with("albedo = baseColorFactor;\nb = a;"),
            "{program}"
        );
    }

    #[test]
    fn an_import_cycle_errors() {
        let profiles = profiles(&[
            ("a", r#"{ "valuesFrom": ["b"] }"#),
            ("b", r#"{ "valuesFrom": ["a"] }"#),
        ]);
        let mut builder = ProgramBuilder::new(Some(&profiles), Vec::new());

        let error = builder
            .land_profile("--profile", "a")
            .unwrap_err()
            .to_string();
        assert!(error.contains("`a` imports `b` imports `a`"), "{error}");
    }

    #[test]
    fn a_computed_name_keeps_the_flags_binding_and_two_profiles_collide() {
        let profiles = profiles(&[
            ("face", r#"{ "computeIndex": { "face": "ao" } }"#),
            ("occlusion", r#"{ "computeOcclusion": "ao" }"#),
        ]);

        let hand = vec![ComputedBinding {
            name: "ao".to_owned(),
            computation: Computation::Index(ArrayDomain::Voxel),
        }];
        let mut builder = ProgramBuilder::new(Some(&profiles), hand.clone());
        builder.land_profile("--profile", "occlusion").unwrap();
        assert_eq!(builder.finish().1, hand);

        let mut builder = ProgramBuilder::new(Some(&profiles), Vec::new());
        builder.land_profile("--profile", "face").unwrap();
        assert!(builder.land_profile("--values-from", "occlusion").is_err());
    }

    #[test]
    fn a_broken_profile_value_errors_at_its_entry() {
        let profiles = profiles(&[("broken", r#"{ "values": ["a = 1", "b ="] }"#)]);
        let mut builder = ProgramBuilder::new(Some(&profiles), Vec::new());

        let error = builder
            .land_profile("--profile", "broken")
            .unwrap_err()
            .to_string();
        assert!(error.contains("`broken`'s values entry 1"), "{error}");
    }
}
