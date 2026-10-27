use crate::{Error, ProfileSet, Result, commands::PaletteEditProfile, parse_fragment};
use std::collections::HashSet;

/// Joins the program from its fragments in arrival order. A profile lands
/// once, at its first arrival, with its `valuesFrom` imports depth-first ahead
/// of it.
pub struct PaletteEditProgramBuilder<'a> {
    profiles: Option<&'a ProfileSet<PaletteEditProfile>>,

    fragments: Vec<String>,

    landed: HashSet<String>,
}

impl<'a> PaletteEditProgramBuilder<'a> {
    /// A builder over `profiles`.
    pub(crate) fn new(profiles: Option<&'a ProfileSet<PaletteEditProfile>>) -> Self {
        PaletteEditProgramBuilder {
            profiles,
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

    /// The joined program.
    pub(crate) fn finish(self) -> String {
        self.fragments.join("\n")
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

        for (position, fragment) in (0..).zip(&profile.values) {
            self.fragments.push(parse_fragment(
                &format!("the profile `{name}`'s values entry {position}"),
                fragment,
            )?);
        }

        self.landed.insert(name.to_owned());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ProfileSet,
        commands::{PaletteEditProfile, PaletteEditProgramBuilder},
        profile_set_from_json,
    };

    fn profiles() -> ProfileSet<PaletteEditProfile> {
        profile_set_from_json(&[
            ("tags", r#"{ "values": ["rust = tag == \"rust\""] }"#),
            (
                "weathered",
                r#"{ "valuesFrom": ["tags"], "values": ["worn = rust"], "properties": { "roughness": "0.9" } }"#,
            ),
            (
                "polished",
                r#"{ "valuesFrom": ["tags", "weathered"], "values": ["shine = !worn"] }"#,
            ),
        ])
    }

    #[test]
    fn imports_land_depth_first_and_each_profile_once() {
        let profiles = profiles();
        let mut builder = PaletteEditProgramBuilder::new(Some(&profiles));

        builder.land_profile("--profile", "polished").unwrap();
        builder.land_profile("--values-from", "weathered").unwrap();

        assert_eq!(
            builder.finish(),
            "rust = tag == \"rust\";\nworn = rust;\nshine = !worn;"
        );
    }

    #[test]
    fn values_append_at_their_position() {
        let profiles = profiles();
        let mut builder = PaletteEditProgramBuilder::new(Some(&profiles));

        builder.push_value("a = 1").unwrap();
        builder.land_profile("--values-from", "tags").unwrap();
        builder.push_value("b = a").unwrap();

        assert_eq!(builder.finish(), "a = 1;\nrust = tag == \"rust\";\nb = a;");
    }

    #[test]
    fn an_import_cycle_errors() {
        let profiles = profile_set_from_json::<PaletteEditProfile>(&[
            ("a", r#"{ "valuesFrom": ["b"] }"#),
            ("b", r#"{ "valuesFrom": ["a"] }"#),
        ]);
        let mut builder = PaletteEditProgramBuilder::new(Some(&profiles));

        let error = builder
            .land_profile("--profile", "a")
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("valuesFrom cycles: `a` imports `b` imports `a`"),
            "{error}"
        );
    }

    #[test]
    fn an_undefined_import_errors_naming_the_importer() {
        let profiles =
            profile_set_from_json::<PaletteEditProfile>(&[("a", r#"{ "valuesFrom": ["tags"] }"#)]);
        let mut builder = PaletteEditProgramBuilder::new(Some(&profiles));

        let error = builder
            .land_profile("--profile", "a")
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("the profile `a`'s valuesFrom asks for the profile `tags`"),
            "{error}"
        );
    }

    #[test]
    fn a_broken_profile_value_errors_at_its_entry() {
        let profiles = profile_set_from_json::<PaletteEditProfile>(&[(
            "broken",
            r#"{ "values": ["a = 1", "b ="] }"#,
        )]);
        let mut builder = PaletteEditProgramBuilder::new(Some(&profiles));

        let error = builder
            .land_profile("--profile", "broken")
            .unwrap_err()
            .to_string();
        assert!(error.contains("`broken`'s values entry 1"), "{error}");
    }
}
