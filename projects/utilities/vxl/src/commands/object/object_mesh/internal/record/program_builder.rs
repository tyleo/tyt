use crate::{Error, ProfileSet, Result, commands::MeshProfile};
use std::collections::HashSet;
use vox_value_language::parse;
use voxsmith::operations::object::{Computation, ComputedBinding};

/// Joins the program from its fragments and gathers the computed bindings,
/// the flags' first and then each landed profile's. A profile lands once, at
/// its first arrival, its `valuesFrom` imports depth-first ahead of it.
pub struct ProgramBuilder<'a> {
    profiles: Option<&'a ProfileSet<MeshProfile>>,

    computed: Vec<ComputedBinding>,

    hand_computed: usize,

    fragments: Vec<String>,

    landed: HashSet<String>,
}

impl<'a> ProgramBuilder<'a> {
    /// A builder over `profiles` holding the flags' `computed` bindings.
    pub(crate) fn new(
        profiles: Option<&'a ProfileSet<MeshProfile>>,
        computed: Vec<ComputedBinding>,
    ) -> Self {
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

/// The program fragment `text`, which `origin` holds, with its terminator
/// appended. An all-whitespace fragment errors, as does one that does not
/// parse.
fn parse_fragment(origin: &str, text: &str) -> Result<String> {
    if text.trim().is_empty() {
        return Err(Error::usage(format!(
            "{origin} holds only whitespace where bindings go"
        )));
    }

    let fragment = format!("{text};");

    parse(&fragment).map_err(|error| {
        Error::usage(format!(
            "{origin} holds `{text}`, which does not parse: {error}"
        ))
    })?;

    Ok(fragment)
}

#[cfg(test)]
mod tests {
    use crate::{
        ProfileSet,
        commands::{
            ProgramBuilder, built_in_profiles,
            object::object_mesh::internal::record::program_builder::parse_fragment,
        },
        profile_set_from_json,
    };
    use std::collections::BTreeSet;
    use vox_value_language::{Dimension, Domain, Scalar, Type, TypeEnvironment, check, parse};
    use voxcore::material::{
        BASE_COLOR, EMISSIVE_COLOR, EMISSIVE_STRENGTH, IOR, METALLIC, OCCLUSION_STRENGTH,
        ROUGHNESS, TRANSMISSION,
    };
    use voxsmith::operations::object::{ArrayDomain, Computation, ComputedBinding};

    #[test]
    fn imports_land_depth_first_and_each_profile_once() {
        let profiles = ProfileSet::layered(built_in_profiles(), []);
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
                "baseColor",
                "occlusionStrength",
                "roughness",
                "metallic",
                "emissiveColor",
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
    fn the_built_ins_read_the_recommended_material_properties() {
        let profiles = ProfileSet::layered(built_in_profiles(), []);
        let mut builder = ProgramBuilder::new(Some(&profiles), Vec::new());

        builder.land_profile("--profile", "pbr").unwrap();

        let (program, _) = builder.finish();
        let swatch = |dimension| Type {
            domain: Domain::Swatch,
            dimension,
            scalar: Scalar::F64,
        };
        let environment = TypeEnvironment {
            types: [
                (BASE_COLOR, Dimension::Vec4),
                (EMISSIVE_COLOR, Dimension::Vec3),
                (EMISSIVE_STRENGTH, Dimension::Vec1),
                (IOR, Dimension::Vec1),
                (METALLIC, Dimension::Vec1),
                (OCCLUSION_STRENGTH, Dimension::Vec1),
                (ROUGHNESS, Dimension::Vec1),
                (TRANSMISSION, Dimension::Vec1),
            ]
            .into_iter()
            .map(|(name, dimension)| (name.to_owned(), swatch(dimension)))
            .collect(),
        };
        let checked = check(parse(&program).unwrap(), &environment).unwrap();

        // A read of a name no earlier binding defines reaches the palette. An
        // unbound `default` never counts as a read, so a stale property name
        // drops out of the set instead of passing as its fallback.
        let mut bound = BTreeSet::new();
        let mut properties = BTreeSet::new();
        for (name, expression) in checked.bindings() {
            properties.extend(
                expression
                    .reads()
                    .names
                    .into_iter()
                    .filter(|read| !bound.contains(read.as_str())),
            );
            bound.insert(name);
        }

        assert_eq!(
            properties,
            [
                BASE_COLOR,
                EMISSIVE_COLOR,
                EMISSIVE_STRENGTH,
                METALLIC,
                OCCLUSION_STRENGTH,
                ROUGHNESS,
            ]
            .map(str::to_owned)
            .into()
        );
    }

    #[test]
    fn values_append_at_their_position() {
        let profiles = ProfileSet::layered(built_in_profiles(), []);
        let mut builder = ProgramBuilder::new(Some(&profiles), Vec::new());

        builder.push_value("a = 1").unwrap();
        builder.land_profile("--values-from", "albedo").unwrap();
        builder.push_value("b = a").unwrap();

        let (program, _) = builder.finish();
        assert!(program.starts_with("a = 1;\nbaseColor"), "{program}");
        assert!(
            program.ends_with("albedo = baseColor;\nb = a;"),
            "{program}"
        );
    }

    #[test]
    fn an_import_cycle_errors() {
        let profiles = profile_set_from_json(&[
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
        let profiles = profile_set_from_json(&[
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
        let profiles = profile_set_from_json(&[("broken", r#"{ "values": ["a = 1", "b ="] }"#)]);
        let mut builder = ProgramBuilder::new(Some(&profiles), Vec::new());

        let error = builder
            .land_profile("--profile", "broken")
            .unwrap_err()
            .to_string();
        assert!(error.contains("`broken`'s values entry 1"), "{error}");
    }

    #[test]
    fn a_fragment_gains_its_terminator() {
        assert_eq!(parse_fragment("--value", "a = 1").unwrap(), "a = 1;");
        assert_eq!(
            parse_fragment("--value", "a = 1; b = a;").unwrap(),
            "a = 1; b = a;;"
        );
    }

    #[test]
    fn whitespace_and_broken_fragments_error_at_their_origin() {
        let error = parse_fragment("--value", "  ").unwrap_err().to_string();
        assert!(error.contains("--value"), "{error}");

        let error = parse_fragment("the profile `x`'s values entry 0", "a =")
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("the profile `x`'s values entry 0"),
            "{error}"
        );
        assert!(error.contains("`a =`"), "{error}");
    }
}
