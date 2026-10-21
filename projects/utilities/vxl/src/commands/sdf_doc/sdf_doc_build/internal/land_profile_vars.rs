use crate::{Error, ProfileSet, Result, commands::SdfDocBuildProfile};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

/// Lands the vars of the profile `name`, which `origin` asks for, into `vars`
/// after its `varsFrom` imports. A profile lands once however many imports
/// bring it, so a diamond import cannot undo an override.
pub fn land_profile_vars(
    profiles: &ProfileSet<SdfDocBuildProfile>,
    origin: &str,
    name: &str,
    vars: &mut BTreeMap<String, Value>,
) -> Result<()> {
    land(
        profiles,
        origin,
        name,
        vars,
        &mut Vec::new(),
        &mut BTreeSet::new(),
    )
}

fn land(
    profiles: &ProfileSet<SdfDocBuildProfile>,
    origin: &str,
    name: &str,
    vars: &mut BTreeMap<String, Value>,
    visiting: &mut Vec<String>,
    landed: &mut BTreeSet<String>,
) -> Result<()> {
    if let Some(start) = visiting.iter().position(|visited| visited == name) {
        let chain: Vec<_> = visiting[start..]
            .iter()
            .map(String::as_str)
            .chain([name])
            .map(|name| format!("`{name}`"))
            .collect();

        return Err(Error::usage(format!(
            "the profile `{name}`'s varsFrom cycles: {}",
            chain.join(" imports ")
        )));
    }

    if landed.contains(name) {
        return Ok(());
    }

    let profile = profiles.get(origin, name)?;

    visiting.push(name.to_owned());

    for import in &profile.vars_from {
        land(
            profiles,
            &format!("the profile `{name}`'s varsFrom"),
            import,
            vars,
            visiting,
            landed,
        )?;
    }

    visiting.pop();

    vars.extend(profile.vars.clone());
    landed.insert(name.to_owned());

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        ProfileSet, Result,
        commands::{SdfDocBuildProfile, land_profile_vars},
        profile_set_from_json,
    };
    use serde_json::{Value, json};
    use std::collections::BTreeMap;

    /// The vars the profile `name` lands.
    fn vars(
        profiles: &ProfileSet<SdfDocBuildProfile>,
        name: &str,
    ) -> Result<BTreeMap<String, Value>> {
        let mut vars = BTreeMap::new();
        land_profile_vars(profiles, "--profile", name, &mut vars)?;
        Ok(vars)
    }

    #[test]
    fn imports_land_in_order_ahead_of_the_profile_s_vars() {
        let profiles = profile_set_from_json(&[
            ("table", r#"{ "vars": { "legs": 4, "height": 0.75 } }"#),
            (
                "oak",
                r#"{ "runtime": "bun", "vars": { "wood": "oak", "legs": 3 } }"#,
            ),
            (
                "tall",
                r#"{ "varsFrom": ["table", "oak"], "vars": { "height": 1.1 } }"#,
            ),
        ]);

        assert_eq!(
            vars(&profiles, "tall").unwrap(),
            BTreeMap::from([
                ("height".to_owned(), json!(1.1)),
                ("legs".to_owned(), json!(3)),
                ("wood".to_owned(), json!("oak")),
            ])
        );
    }

    #[test]
    fn a_profile_two_imports_bring_lands_once() {
        let profiles = profile_set_from_json(&[
            ("base", r#"{ "vars": { "legs": 4 } }"#),
            (
                "three",
                r#"{ "varsFrom": ["base"], "vars": { "legs": 3 } }"#,
            ),
            (
                "wood",
                r#"{ "varsFrom": ["base"], "vars": { "wood": "oak" } }"#,
            ),
            ("stool", r#"{ "varsFrom": ["three", "wood"] }"#),
        ]);

        assert_eq!(vars(&profiles, "stool").unwrap()["legs"], json!(3));
    }

    #[test]
    fn an_import_cycle_and_an_unknown_import_error() {
        let profiles = profile_set_from_json(&[
            ("a", r#"{ "varsFrom": ["b"] }"#),
            ("b", r#"{ "varsFrom": ["a"] }"#),
            ("lost", r#"{ "varsFrom": ["nowhere"] }"#),
        ]);

        let error = vars(&profiles, "a").unwrap_err().to_string();
        assert!(
            error.contains("the profile `a`'s varsFrom cycles: `a` imports `b` imports `a`"),
            "{error}"
        );

        let error = vars(&profiles, "lost").unwrap_err().to_string();
        assert!(
            error.contains("the profile `lost`'s varsFrom asks for the profile `nowhere`"),
            "{error}"
        );
    }
}
