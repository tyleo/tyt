use crate::{NamedCliValue, Profile, ProfileDescription};
use sdfj_builder::JavaScriptRuntime;
use serde::Deserialize;

/// An `sdf-doc build` profile, whose elements each mirror a flag.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct SdfDocBuildProfile {
    /// Printed beside the profile name in the profile listings.
    pub(crate) description: Option<ProfileDescription>,

    /// Mirrors `--intern`.
    pub(crate) intern: Option<bool>,

    /// Mirrors `--library`.
    pub(crate) libraries: Vec<String>,

    /// Mirrors `--runtime`.
    pub(crate) runtime: Option<NamedCliValue<JavaScriptRuntime>>,
}

impl Profile for SdfDocBuildProfile {
    fn description(&self) -> Option<&ProfileDescription> {
        self.description.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use crate::{NamedCliValue, commands::SdfDocBuildProfile};
    use sdfj_builder::JavaScriptRuntime;

    #[test]
    fn every_key_reads_into_its_element() {
        let profile: SdfDocBuildProfile = serde_json::from_str(
            r#"{
                "description": "Models under Bun",
                "intern": false,
                "libraries": ["materials", "props"],
                "runtime": "bun"
            }"#,
        )
        .unwrap();

        assert_eq!(profile.description.unwrap().as_str(), "Models under Bun");
        assert_eq!(profile.intern, Some(false));
        assert_eq!(profile.libraries, ["materials", "props"]);
        assert_eq!(profile.runtime, Some(NamedCliValue(JavaScriptRuntime::Bun)));
    }

    #[test]
    fn a_bad_key_or_value_errors() {
        for json in [
            r#"{ "runtime": "python" }"#,
            r#"{ "output": "chair.sdfj" }"#,
            r#"{ "profile": "bun" }"#,
            r#"{ "libraries": "materials" }"#,
            r#"{ "intern": "no" }"#,
        ] {
            assert!(
                serde_json::from_str::<SdfDocBuildProfile>(json).is_err(),
                "{json}"
            );
        }
    }
}
