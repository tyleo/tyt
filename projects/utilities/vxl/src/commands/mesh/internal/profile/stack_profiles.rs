use crate::{
    Error, Result,
    commands::{Profile, ProfileSet},
};
use std::collections::BTreeMap;

/// The profiles `names`, which `origin` lists, stacked into one profile to
/// apply whole. Lists merge by position and the longest sets the stack's
/// count. An element two members both set errors. The stack carries no
/// program elements because each member lands its values, imports, and
/// computed bindings by name.
pub(crate) fn stack_profiles(
    profiles: &ProfileSet,
    origin: &str,
    names: &[String],
) -> Result<Profile> {
    let mut stack = Profile::default();
    let mut claims = BTreeMap::new();

    for (position, name) in (0..).zip(names) {
        if names[..position].contains(name) {
            return Err(Error::usage(format!("{origin} lists `{name}` twice")));
        }

        let member = profiles.get(origin, name)?;

        if let Some(size) = member.voxel_size {
            claim(&mut claims, name, "voxelSize".to_owned())?;
            stack.voxel_size = Some(size);
        }

        if let Some(method) = member.method {
            claim(&mut claims, name, "method".to_owned())?;
            stack.method = Some(method);
        }

        if let Some(shape) = member.texture_shape {
            claim(&mut claims, name, "textureShape".to_owned())?;
            stack.texture_shape = Some(shape);
        }

        for (template, entries) in &member.files.json {
            let target = stack.files.json.entry(template.clone()).or_default();

            for (key, entry) in entries {
                claim(
                    &mut claims,
                    name,
                    format!("files.json template `{template}`'s key `{key}`"),
                )?;
                target.insert(key.clone(), entry.clone());
            }
        }

        for (template, entry) in &member.files.png {
            claim(
                &mut claims,
                name,
                format!("files.png template `{template}`"),
            )?;
            stack.files.png.insert(template.clone(), entry.clone());
        }

        if stack.materials.len() < member.materials.len() {
            stack
                .materials
                .resize_with(member.materials.len(), Default::default);
        }

        for (index, entry) in (0..).zip(&member.materials) {
            let target = &mut stack.materials[index];
            let entry_origin = format!("materials entry {index}");

            if let Some(material_name) = &entry.name {
                claim(&mut claims, name, format!("{entry_origin}'s name"))?;
                target.name = Some(material_name.clone());
            }

            if let Some(uvs) = &entry.uvs {
                claim(&mut claims, name, format!("{entry_origin}'s uvs"))?;
                target.uvs = Some(uvs.clone());
            }

            for (property, slot) in &entry.slots {
                claim(
                    &mut claims,
                    name,
                    format!("{entry_origin}'s slot `{property}`"),
                )?;
                target.slots.insert(property.clone(), slot.clone());
            }

            for (extra_name, extra) in &entry.extras {
                claim(
                    &mut claims,
                    name,
                    format!("{entry_origin}'s extra `{extra_name}`"),
                )?;
                target.extras.insert(extra_name.clone(), extra.clone());
            }
        }

        if stack.primitives.len() < member.primitives.len() {
            stack
                .primitives
                .resize_with(member.primitives.len(), Default::default);
        }

        for (index, entry) in (0..).zip(&member.primitives) {
            let target = &mut stack.primitives[index];
            let entry_origin = format!("primitives entry {index}");

            if let Some(primitive_name) = &entry.name {
                claim(&mut claims, name, format!("{entry_origin}'s name"))?;
                target.name = Some(primitive_name.clone());
            }

            if let Some(select) = &entry.select {
                claim(&mut claims, name, format!("{entry_origin}'s select"))?;
                target.select = Some(select.clone());
            }

            if let Some(material) = entry.material {
                claim(&mut claims, name, format!("{entry_origin}'s material"))?;
                target.material = Some(material);
            }

            if let Some(normal) = entry.normal {
                claim(&mut claims, name, format!("{entry_origin}'s normal"))?;
                target.normal = Some(normal);
            }

            if let Some(uvs) = &entry.uvs {
                claim(&mut claims, name, format!("{entry_origin}'s uvs"))?;
                target.uvs = Some(uvs.clone());
            }

            for (attribute, expression) in &entry.builtins {
                claim(
                    &mut claims,
                    name,
                    format!("{entry_origin}'s builtins key `{attribute}`"),
                )?;
                target
                    .builtins
                    .insert(attribute.clone(), expression.clone());
            }

            for (custom_name, value) in &entry.customs {
                claim(
                    &mut claims,
                    name,
                    format!("{entry_origin}'s customs key `{custom_name}`"),
                )?;
                target.customs.insert(custom_name.clone(), value.clone());
            }
        }

        for (extra_name, extra) in &member.mesh_extras {
            claim(
                &mut claims,
                name,
                format!("meshExtras entry `{extra_name}`"),
            )?;
            stack.mesh_extras.insert(extra_name.clone(), extra.clone());
        }
    }

    Ok(stack)
}

/// Records that the profile `name` sets `element`, erroring when an earlier
/// member of the stack set it.
fn claim<'a>(claims: &mut BTreeMap<String, &'a str>, name: &'a str, element: String) -> Result<()> {
    if let Some(earlier) = claims.get(&element) {
        return Err(Error::usage(format!(
            "the profile `{name}` sets {element}, which the profile `{earlier}` sets already"
        )));
    }

    claims.insert(element, name);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::stack_profiles;
    use crate::commands::{NamedCliValue, Profile, ProfileSet, SlotEntry};
    use std::collections::BTreeMap;
    use voxsmith::operations::mesh::Method;

    /// A set holding the profiles `entries` defines as json.
    fn profiles(entries: &[(&str, &str)]) -> ProfileSet {
        let profiles: BTreeMap<String, Profile> = entries
            .iter()
            .map(|(name, json)| ((*name).to_owned(), serde_json::from_str(json).unwrap()))
            .collect();

        ProfileSet::from_profiles(profiles)
    }

    fn names(names: &[&str]) -> Vec<String> {
        names.iter().map(|name| (*name).to_owned()).collect()
    }

    #[test]
    fn the_writers_merge_by_position_and_the_program_stays_behind() {
        let profiles = profiles(&[
            (
                "albedo",
                r#"{
                    "valuesFrom": ["defaults"],
                    "computeOcclusion": "ao",
                    "values": ["albedo = baseColorFactor"],
                    "voxelSize": 0.1,
                    "materials": [
                        {
                            "name": "body",
                            "slots": { "baseColorTexture": { "kind": "value", "value": "albedo" } }
                        }
                    ],
                    "primitives": [{ "select": "solid", "material": 0 }],
                    "files": { "json": { "{file-stem}.json": { "ior": { "transfer": "linear", "value": "ior" } } } },
                    "meshExtras": { "accent": { "kind": "json-value", "transfer": "srgb", "value": "accent" } }
                }"#,
            ),
            (
                "orm",
                r#"{
                    "method": "culled",
                    "materials": [
                        { "slots": { "occlusionTexture": { "kind": "value", "value": "orm" } } },
                        { "name": "glow" }
                    ],
                    "primitives": [{ "builtins": { "COLOR_0": "albedo" } }],
                    "files": {
                        "json": { "{file-stem}.json": { "tint": { "transfer": "srgb", "value": "tint" } } },
                        "png": { "{file-stem}-orm.png": { "transfer": "linear", "value": "orm" } }
                    },
                    "meshExtras": { "heat": { "kind": "image-file", "file": "{file-stem}-heat.png" } }
                }"#,
            ),
        ]);

        let stack = stack_profiles(&profiles, "--profile", &names(&["albedo", "orm"])).unwrap();

        assert!(stack.values_from.is_empty());
        assert!(stack.values.is_empty());
        assert!(stack.compute_occlusion.0.is_empty());
        assert_eq!(stack.voxel_size, Some(0.1));
        assert_eq!(stack.method, Some(NamedCliValue(Method::Culled)));

        let [body, glow] = stack.materials.as_slice() else {
            panic!("two materials");
        };
        assert_eq!(body.name.as_deref(), Some("body"));
        assert_eq!(
            body.slots.keys().collect::<Vec<_>>(),
            ["baseColorTexture", "occlusionTexture"]
        );
        assert_eq!(
            body.slots["occlusionTexture"],
            SlotEntry::Value {
                value: "orm".to_owned()
            }
        );
        assert_eq!(glow.name.as_deref(), Some("glow"));

        let [primitive] = stack.primitives.as_slice() else {
            panic!("one primitive");
        };
        assert_eq!(primitive.select.as_deref(), Some("solid"));
        assert_eq!(primitive.material, Some(0));
        assert_eq!(primitive.builtins["COLOR_0"], "albedo");

        assert_eq!(
            stack.files.json["{file-stem}.json"]
                .keys()
                .collect::<Vec<_>>(),
            ["ior", "tint"]
        );
        assert_eq!(stack.files.png.len(), 1);
        assert_eq!(
            stack.mesh_extras.keys().collect::<Vec<_>>(),
            ["accent", "heat"]
        );
    }

    #[test]
    fn an_element_two_members_set_errors_naming_both() {
        let profiles = profiles(&[
            (
                "a",
                r#"{ "materials": [{ "slots": { "baseColorTexture": { "kind": "value", "value": "a" } } }] }"#,
            ),
            (
                "b",
                r#"{ "materials": [{ "slots": { "baseColorTexture": { "kind": "value", "value": "b" } } }] }"#,
            ),
            ("greedy", r#"{ "method": "greedy" }"#),
            ("culled", r#"{ "method": "culled" }"#),
        ]);

        let error = stack_profiles(&profiles, "--profile", &names(&["a", "b"]))
            .unwrap_err()
            .to_string();
        assert!(
            error.contains(
                "the profile `b` sets materials entry 0's slot `baseColorTexture`, which the \
                 profile `a` sets already"
            ),
            "{error}"
        );

        let error = stack_profiles(&profiles, "--profile", &names(&["greedy", "culled"]))
            .unwrap_err()
            .to_string();
        assert!(error.contains("`culled` sets method"), "{error}");
    }

    #[test]
    fn a_member_listed_twice_errors() {
        let profiles = ProfileSet::built_in();

        let error = stack_profiles(&profiles, "--profile", &names(&["orm", "orm"]))
            .unwrap_err()
            .to_string();
        assert!(error.contains("--profile lists `orm` twice"), "{error}");
    }

    #[test]
    fn an_undefined_member_errors() {
        let profiles = ProfileSet::built_in();

        assert!(stack_profiles(&profiles, "--profile", &names(&["albedo", "metal"])).is_err());
    }
}
