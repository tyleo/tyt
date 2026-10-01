use crate::{
    Error, ProfileSet, Result,
    commands::{RenderProfile, ViewEntry},
};
use std::collections::{BTreeMap, BTreeSet};

/// Profiles stacked into one profile to apply whole. Views merge by name,
/// element by element. The lights merge as one element, the rig. A
/// profile's `viewsFrom` and `lightsFrom` imports land depth-first ahead of
/// its views and rig. Each profile's views and rig land once, however many
/// members and imports bring them. An element two profiles set errors.
pub struct RenderProfileStack<'a> {
    profiles: &'a ProfileSet<RenderProfile>,

    profile: RenderProfile,

    // Each set element and the profile that set it.
    claims: BTreeMap<String, String>,

    landed_views: BTreeSet<String>,

    landed_lights: BTreeSet<String>,
}

impl<'a> RenderProfileStack<'a> {
    /// An empty stack over `profiles`.
    pub(crate) fn new(profiles: &'a ProfileSet<RenderProfile>) -> Self {
        RenderProfileStack {
            profiles,
            profile: RenderProfile::default(),
            claims: BTreeMap::new(),
            landed_views: BTreeSet::new(),
            landed_lights: BTreeSet::new(),
        }
    }

    /// Lands each profile of `names`, which `origin` lists, whole. A name
    /// listed twice errors.
    pub(crate) fn land_profiles(&mut self, origin: &str, names: &[String]) -> Result<()> {
        for name in check_unique(origin, names)? {
            let member = self.profiles.get(origin, name)?;

            if let Some(width) = member.width {
                self.claim(name, "width".to_owned())?;
                self.profile.width = Some(width);
            }

            if let Some(height) = member.height {
                self.claim(name, "height".to_owned())?;
                self.profile.height = Some(height);
            }

            if let Some(background) = member.background {
                self.claim(name, "background".to_owned())?;
                self.profile.background = Some(background);
            }

            if let Some(occlusion) = member.occlusion {
                self.claim(name, "occlusion".to_owned())?;
                self.profile.occlusion = Some(occlusion);
            }

            if let Some(voxel_size) = member.voxel_size {
                self.claim(name, "voxelSize".to_owned())?;
                self.profile.voxel_size = Some(voxel_size);
            }

            if let Some(strength) = member.bloom_strength {
                self.claim(name, "bloomStrength".to_owned())?;
                self.profile.bloom_strength = Some(strength);
            }

            if let Some(radius) = member.bloom_radius {
                self.claim(name, "bloomRadius".to_owned())?;
                self.profile.bloom_radius = Some(radius);
            }

            if let Some(threshold) = member.bloom_threshold {
                self.claim(name, "bloomThreshold".to_owned())?;
                self.profile.bloom_threshold = Some(threshold);
            }

            self.land_views(origin, name, &mut Vec::new())?;
            self.land_lights(origin, name, &mut Vec::new())?;
        }

        Ok(())
    }

    /// Lands the views alone of each profile of `names`, which `origin`
    /// lists. A name listed twice errors.
    pub(crate) fn land_view_sets(&mut self, origin: &str, names: &[String]) -> Result<()> {
        for name in check_unique(origin, names)? {
            self.land_views(origin, name, &mut Vec::new())?;
        }

        Ok(())
    }

    /// Lands the rig alone of each profile of `names`, which `origin` lists.
    /// A name listed twice errors.
    pub(crate) fn land_rigs(&mut self, origin: &str, names: &[String]) -> Result<()> {
        for name in check_unique(origin, names)? {
            self.land_lights(origin, name, &mut Vec::new())?;
        }

        Ok(())
    }

    /// Whether any landed profile set a view.
    pub(crate) fn has_views(&self) -> bool {
        !self.profile.views.is_empty()
    }

    /// Whether any landed profile set the rig.
    pub(crate) fn has_rig(&self) -> bool {
        !self.profile.lights.is_empty()
    }

    /// The stacked profile.
    pub(crate) fn finish(self) -> RenderProfile {
        self.profile
    }

    /// Lands the views of the profile `name`, which `origin` asks for, after
    /// its `viewsFrom` imports.
    fn land_views(&mut self, origin: &str, name: &str, visiting: &mut Vec<String>) -> Result<()> {
        check_cycle(visiting, name, "viewsFrom")?;

        if self.landed_views.contains(name) {
            return Ok(());
        }

        let profile = self.profiles.get(origin, name)?;

        visiting.push(name.to_owned());

        for import in &profile.views_from {
            self.land_views(
                &format!("the profile `{name}`'s viewsFrom"),
                import,
                visiting,
            )?;
        }

        visiting.pop();

        for (view_name, entry) in &profile.views {
            let entry_origin = format!("view `{view_name}`");

            if let Some(transform) = &entry.transform {
                self.claim(name, format!("{entry_origin}'s transform"))?;
                self.view(view_name).transform = Some(transform.clone());
            }

            if let Some(projection) = entry.projection {
                self.claim(name, format!("{entry_origin}'s projection"))?;
                self.view(view_name).projection = Some(projection);
            }

            if let Some(fov) = entry.fov {
                self.claim(name, format!("{entry_origin}'s fov"))?;
                self.view(view_name).fov = Some(fov);
            }

            if let Some(scale) = entry.scale {
                self.claim(name, format!("{entry_origin}'s scale"))?;
                self.view(view_name).scale = Some(scale);
            }

            if let Some(select) = &entry.select {
                self.claim(name, format!("{entry_origin}'s select"))?;
                self.view(view_name).select = Some(select.clone());
            }
        }

        self.landed_views.insert(name.to_owned());

        Ok(())
    }

    /// Lands the rig of the profile `name`, which `origin` asks for, after its
    /// `lightsFrom` imports.
    fn land_lights(&mut self, origin: &str, name: &str, visiting: &mut Vec<String>) -> Result<()> {
        check_cycle(visiting, name, "lightsFrom")?;

        if self.landed_lights.contains(name) {
            return Ok(());
        }

        let profile = self.profiles.get(origin, name)?;

        visiting.push(name.to_owned());

        for import in &profile.lights_from {
            self.land_lights(
                &format!("the profile `{name}`'s lightsFrom"),
                import,
                visiting,
            )?;
        }

        visiting.pop();

        if !profile.lights.is_empty() {
            self.claim(name, "lights".to_owned())?;
            self.profile.lights = profile.lights.clone();
        }

        self.landed_lights.insert(name.to_owned());

        Ok(())
    }

    fn view(&mut self, name: &str) -> &mut ViewEntry {
        self.profile.views.entry(name.to_owned()).or_default()
    }

    fn claim(&mut self, name: &str, element: String) -> Result<()> {
        if let Some(earlier) = self.claims.get(&element) {
            return Err(Error::usage(format!(
                "the profile `{name}` sets {element}, which the profile `{earlier}` sets already"
            )));
        }

        self.claims.insert(element, name.to_owned());

        Ok(())
    }
}

/// `names`, which `origin` lists, checked to list no name twice.
fn check_unique<'n>(origin: &str, names: &'n [String]) -> Result<&'n [String]> {
    for (position, name) in (0..).zip(names) {
        if names[..position].contains(name) {
            return Err(Error::usage(format!("{origin} lists `{name}` twice")));
        }
    }

    Ok(names)
}

/// Errors when `name` already sits on the import chain `visiting`. `key` is
/// the import key the error reports.
fn check_cycle(visiting: &[String], name: &str, key: &str) -> Result<()> {
    let Some(start) = visiting.iter().position(|visited| visited == name) else {
        return Ok(());
    };

    let chain: Vec<_> = visiting[start..]
        .iter()
        .map(String::as_str)
        .chain([name])
        .map(|name| format!("`{name}`"))
        .collect();

    Err(Error::usage(format!(
        "the profile `{name}`'s {key} cycles: {}",
        chain.join(" imports ")
    )))
}

#[cfg(test)]
mod tests {
    use crate::{
        NamedCliValue, ProfileSet, Result,
        commands::{ProjectionKind, RenderProfile, RenderProfileStack, built_in_render_profiles},
        owned_names, profile_set_from_json,
    };

    /// The profiles `names` stacked whole.
    fn stack(profiles: &ProfileSet<RenderProfile>, names: &[&str]) -> Result<RenderProfile> {
        let mut stack = RenderProfileStack::new(profiles);
        stack.land_profiles("--profile", &owned_names(names))?;
        Ok(stack.finish())
    }

    #[test]
    fn views_merge_by_name_and_a_rig_joins_a_view_set() {
        let profiles = profile_set_from_json(&[
            (
                "views",
                r#"{
                    "width": 256,
                    "views": {
                        "hero": { "transform": { "kind": "orbit", "azimuth": 45, "elevation": 30 } },
                        "plan": { "transform": { "kind": "orbit", "azimuth": 0, "elevation": 90 } }
                    }
                }"#,
            ),
            (
                "flat-plan",
                r#"{
                    "height": 128,
                    "views": { "plan": { "projection": "orthographic" } }
                }"#,
            ),
            (
                "rig",
                r#"{ "lights": [{ "kind": "hemisphere", "strength": 1 }] }"#,
            ),
        ]);

        let stacked = stack(&profiles, &["views", "flat-plan", "rig"]).unwrap();

        assert_eq!(stacked.width.map(u32::from), Some(256));
        assert_eq!(stacked.height.map(u32::from), Some(128));
        assert_eq!(stacked.views.keys().collect::<Vec<_>>(), ["hero", "plan"]);
        assert!(stacked.views["plan"].transform.is_some());
        assert_eq!(
            stacked.views["plan"].projection,
            Some(NamedCliValue(ProjectionKind::Orthographic))
        );
        assert_eq!(stacked.lights.len(), 1);
    }

    #[test]
    fn an_element_two_members_set_errors_naming_both() {
        let profiles = profile_set_from_json(&[
            ("a", r#"{ "views": { "hero": { "fov": 40 } } }"#),
            ("b", r#"{ "views": { "hero": { "fov": 50 } } }"#),
            ("wide", r#"{ "width": 1 }"#),
            ("wider", r#"{ "width": 2 }"#),
        ]);

        let error = stack(&profiles, &["a", "b"]).unwrap_err().to_string();
        assert!(
            error.contains(
                "the profile `b` sets view `hero`'s fov, which the profile `a` sets already"
            ),
            "{error}"
        );

        let error = stack(&profiles, &["wide", "wider"])
            .unwrap_err()
            .to_string();
        assert!(error.contains("`wider` sets width"), "{error}");
    }

    #[test]
    fn two_rigs_error_and_a_member_listed_twice_errors() {
        let profiles = ProfileSet::layered(built_in_render_profiles(), []);

        let error = stack(&profiles, &["studio", "flat"])
            .unwrap_err()
            .to_string();
        assert!(error.contains("`flat` sets lights"), "{error}");

        let error = stack(&profiles, &["hero", "hero"]).unwrap_err().to_string();
        assert!(error.contains("--profile lists `hero` twice"), "{error}");

        let stacked = stack(&profiles, &["turnaround", "studio"]).unwrap();
        assert_eq!(
            stacked.views.keys().collect::<Vec<_>>(),
            ["back", "front", "hero", "left", "right"]
        );
        assert_eq!(stacked.lights.len(), 2);
    }

    #[test]
    fn views_from_imports_views_alone_ahead_of_the_profile_s_own() {
        let profiles = profile_set_from_json(&[
            (
                "base",
                r#"{
                    "width": 256,
                    "views": { "hero": { "transform": { "kind": "orbit", "azimuth": 45, "elevation": 30 } } },
                    "lights": [{ "kind": "hemisphere", "strength": 1 }]
                }"#,
            ),
            (
                "wide-hero",
                r#"{ "viewsFrom": ["base"], "views": { "hero": { "fov": 60 } } }"#,
            ),
            (
                "clash",
                r#"{ "viewsFrom": ["base"], "views": { "hero": { "transform": { "kind": "orbit", "azimuth": 0, "elevation": 0 } } } }"#,
            ),
        ]);

        let stacked = stack(&profiles, &["wide-hero"]).unwrap();
        assert!(stacked.views["hero"].transform.is_some());
        assert!(stacked.views["hero"].fov.is_some());
        assert_eq!(stacked.width, None);
        assert!(stacked.lights.is_empty());

        let error = stack(&profiles, &["clash"]).unwrap_err().to_string();
        assert!(
            error.contains(
                "the profile `clash` sets view `hero`'s transform, which the profile `base` sets already"
            ),
            "{error}"
        );
    }

    #[test]
    fn lights_from_imports_one_rig() {
        let mut profiles = built_in_render_profiles();
        for (name, json) in [
            (
                "lit",
                r#"{ "viewsFrom": ["top"], "lightsFrom": ["studio"] }"#,
            ),
            ("two-rigs", r#"{ "lightsFrom": ["studio", "flat"] }"#),
            (
                "own-rig",
                r#"{ "lightsFrom": ["studio"], "lights": [{ "kind": "hemisphere" }] }"#,
            ),
        ] {
            profiles.insert(name.to_owned(), serde_json::from_str(json).unwrap());
        }
        let profiles = ProfileSet::from_profiles(profiles);

        let stacked = stack(&profiles, &["lit"]).unwrap();
        assert_eq!(stacked.views.keys().collect::<Vec<_>>(), ["top"]);
        assert_eq!(stacked.lights.len(), 2);

        let error = stack(&profiles, &["two-rigs"]).unwrap_err().to_string();
        assert!(error.contains("`flat` sets lights"), "{error}");

        let error = stack(&profiles, &["own-rig"]).unwrap_err().to_string();
        assert!(error.contains("`own-rig` sets lights"), "{error}");
    }

    #[test]
    fn a_profile_s_views_land_once_however_many_bring_them() {
        let profiles = profile_set_from_json(&[
            (
                "d",
                r#"{ "views": { "d": { "transform": { "kind": "orbit", "azimuth": 0, "elevation": 0 } } } }"#,
            ),
            ("b", r#"{ "viewsFrom": ["d"] }"#),
            ("c", r#"{ "viewsFrom": ["d"] }"#),
            ("a", r#"{ "viewsFrom": ["b", "c"] }"#),
        ]);

        let stacked = stack(&profiles, &["a", "d"]).unwrap();
        assert_eq!(stacked.views.keys().collect::<Vec<_>>(), ["d"]);
    }

    #[test]
    fn an_import_cycle_and_an_unknown_import_error() {
        let profiles = profile_set_from_json(&[
            ("a", r#"{ "viewsFrom": ["b"] }"#),
            ("b", r#"{ "viewsFrom": ["a"] }"#),
            ("lost", r#"{ "lightsFrom": ["nowhere"] }"#),
        ]);

        let error = stack(&profiles, &["a"]).unwrap_err().to_string();
        assert!(
            error.contains("the profile `a`'s viewsFrom cycles: `a` imports `b` imports `a`"),
            "{error}"
        );

        let error = stack(&profiles, &["lost"]).unwrap_err().to_string();
        assert!(
            error.contains("the profile `lost`'s lightsFrom asks for the profile `nowhere`"),
            "{error}"
        );
    }

    #[test]
    fn the_halves_land_alone_and_a_whole_profile_lands_its_half_once() {
        let profiles = ProfileSet::layered(built_in_render_profiles(), []);

        let mut stack = RenderProfileStack::new(&profiles);
        stack
            .land_view_sets("--views-from", &owned_names(&["turnaround"]))
            .unwrap();
        assert!(stack.has_views() && !stack.has_rig());

        stack
            .land_rigs("--lights-from", &owned_names(&["studio"]))
            .unwrap();
        assert!(stack.has_rig());

        // `hero` came in through `turnaround`, so the whole profile lands
        // nothing new.
        stack
            .land_profiles("--profile", &owned_names(&["hero"]))
            .unwrap();
        let profile = stack.finish();
        assert_eq!(profile.views.len(), 5);
        assert_eq!(profile.lights.len(), 2);

        let mut stack = RenderProfileStack::new(&profiles);
        let error = stack
            .land_rigs("--lights-from", &owned_names(&["studio", "studio"]))
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("--lights-from lists `studio` twice"),
            "{error}"
        );
    }
}
