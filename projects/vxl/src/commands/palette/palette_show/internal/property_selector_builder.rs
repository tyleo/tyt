use crate::{Error, ProfileSet, Result, commands::PaletteShowProfile};
use std::collections::HashSet;
use voxsmith::operations::palette::PropertySelector;

/// Gathers the `palette show` selectors from the flags and the landed profiles
/// in arrival order. A profile lands once, at its first arrival, with its
/// `propertiesFrom` imports depth-first ahead of it.
pub struct PropertySelectorBuilder<'a> {
    profiles: Option<&'a ProfileSet<PaletteShowProfile>>,

    selectors: Vec<PropertySelector>,

    landed: HashSet<String>,
}

impl<'a> PropertySelectorBuilder<'a> {
    /// A builder over `profiles`.
    pub(crate) fn new(profiles: Option<&'a ProfileSet<PaletteShowProfile>>) -> Self {
        PropertySelectorBuilder {
            profiles,
            selectors: Vec::new(),
            landed: HashSet::new(),
        }
    }

    /// Appends the `--property` selector.
    pub(crate) fn push_selector(&mut self, selector: PropertySelector) {
        self.selectors.push(selector);
    }

    /// Lands the profile `name`, which `origin` asks for, with its imports.
    pub(crate) fn land_profile(&mut self, origin: &str, name: &str) -> Result<()> {
        self.land(origin, name, &mut Vec::new())
    }

    /// The gathered selectors, or the single default selecting every property
    /// of every palette when none arrived.
    pub(crate) fn finish(self) -> Vec<PropertySelector> {
        if self.selectors.is_empty() {
            vec![PropertySelector::default()]
        } else {
            self.selectors
        }
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
                "the profile `{name}`'s propertiesFrom cycles: {}",
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

        for import in &profile.properties_from {
            self.land(
                &format!("the profile `{name}`'s propertiesFrom"),
                import,
                visiting,
            )?;
        }

        visiting.pop();

        self.selectors
            .extend(profile.properties.iter().map(|entry| entry.0.clone()));

        self.landed.insert(name.to_owned());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        commands::{PropertySelectorBuilder, parse_property_selector},
        profile_set_from_json,
    };
    use voxsmith::operations::palette::PropertySelector;

    /// The selector for `property` with the other fields defaulted.
    fn selector(property: &str) -> PropertySelector {
        parse_property_selector("*", property, "auto", "auto").unwrap()
    }

    #[test]
    fn imports_land_depth_first_and_each_profile_once() {
        let profiles = profile_set_from_json(&[
            ("base", r#"{ "properties": [{ "property": "baseColor" }] }"#),
            (
                "orm",
                r#"{ "propertiesFrom": ["base"], "properties": [{ "property": "metallic" }] }"#,
            ),
            (
                "pbr",
                r#"{ "propertiesFrom": ["base", "orm"], "properties": [{ "property": "emissiveColor" }] }"#,
            ),
        ]);
        let mut builder = PropertySelectorBuilder::new(Some(&profiles));

        builder.push_selector(selector("tint"));
        builder.land_profile("--profile", "pbr").unwrap();
        builder.land_profile("--properties-from", "orm").unwrap();

        assert_eq!(
            builder.finish(),
            [
                selector("tint"),
                selector("baseColor"),
                selector("metallic"),
                selector("emissiveColor"),
            ]
        );
    }

    #[test]
    fn no_selector_finishes_as_the_default() {
        let profiles = profile_set_from_json(&[("table", r#"{ "layout": "md-tables" }"#)]);
        let mut builder = PropertySelectorBuilder::new(Some(&profiles));

        builder.land_profile("--profile", "table").unwrap();

        assert_eq!(builder.finish(), [PropertySelector::default()]);
    }

    #[test]
    fn a_cycle_errors_naming_the_chain() {
        let profiles = profile_set_from_json(&[
            ("a", r#"{ "propertiesFrom": ["b"] }"#),
            ("b", r#"{ "propertiesFrom": ["a"] }"#),
        ]);
        let mut builder = PropertySelectorBuilder::new(Some(&profiles));

        let error = builder
            .land_profile("--profile", "a")
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("propertiesFrom cycles: `a` imports `b` imports `a`"),
            "{error}"
        );
    }

    #[test]
    fn an_undefined_import_errors_naming_the_importer() {
        let profiles = profile_set_from_json(&[("a", r#"{ "propertiesFrom": ["metal"] }"#)]);
        let mut builder = PropertySelectorBuilder::new(Some(&profiles));

        let error = builder
            .land_profile("--profile", "a")
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("the profile `a`'s propertiesFrom asks for the profile `metal`"),
            "{error}"
        );
    }
}
