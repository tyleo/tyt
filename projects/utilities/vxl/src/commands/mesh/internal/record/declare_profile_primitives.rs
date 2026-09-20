use crate::{
    Result,
    commands::{MaterialTable, Profile, check_expression},
};
use branded_id::U32Id;
use voxsmith::operations::mesh::PrimitiveRecord;

/// The primitive declarations `profile`, which `origin` applies, holds in
/// list order, each drawing with the material it mentions in `materials` or
/// with none. The rest of each entry applies after the flags claim theirs.
pub(crate) fn declare_profile_primitives(
    materials: &mut MaterialTable,
    profile: &Profile,
    origin: &str,
) -> Result<Vec<PrimitiveRecord>> {
    (0..)
        .zip(&profile.primitives)
        .map(|(index, entry)| {
            let entry_origin = format!("{origin}'s primitives entry {index}");

            let material_id = match entry.material {
                Some(material) => {
                    materials.material(&entry_origin, material)?;
                    Some(U32Id::from_u32(material))
                }

                None => None,
            };

            let select = entry.select.clone().unwrap_or_else(|| "true".to_owned());

            check_expression(&format!("{entry_origin}'s select"), &select)?;

            Ok(PrimitiveRecord {
                material_id,
                select,
                name: None,
                normal: true,
                uv_streams: None,
                attributes: Vec::new(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::declare_profile_primitives;
    use crate::commands::{MaterialTable, Profile};
    use branded_id::U32Id;

    #[test]
    fn each_entry_declares_its_material_and_select() {
        let profile: Profile = serde_json::from_str(
            r#"{
                "materials": [{}, {}],
                "primitives": [
                    { "select": "solid", "material": 0 },
                    { "select": "glowing", "material": 1 },
                    {}
                ]
            }"#,
        )
        .unwrap();
        let mut materials = MaterialTable::declared(2, "the profile `x`".to_owned());

        let primitives =
            declare_profile_primitives(&mut materials, &profile, "the profile `x`").unwrap();

        assert_eq!(primitives.len(), 3);
        assert_eq!(primitives[0].material_id, Some(U32Id::from_u32(0)));
        assert_eq!(primitives[0].select, "solid");
        assert_eq!(primitives[1].material_id, Some(U32Id::from_u32(1)));
        assert_eq!(primitives[2].material_id, None);
        assert_eq!(primitives[2].select, "true");
    }

    #[test]
    fn a_material_outside_the_count_errors() {
        let profile: Profile =
            serde_json::from_str(r#"{ "materials": [{}], "primitives": [{ "material": 1 }] }"#)
                .unwrap();
        let mut materials = MaterialTable::declared(1, "the profile `x`".to_owned());

        let error = declare_profile_primitives(&mut materials, &profile, "the profile `x`")
            .unwrap_err()
            .to_string();
        assert!(error.contains("primitives entry 0"), "{error}");
    }
}
