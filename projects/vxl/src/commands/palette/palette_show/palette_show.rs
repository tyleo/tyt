use crate::{
    Dependencies, Error, ProfileSet, Result, VoxelInput, Width, cli_value_parser,
    commands::{
        PaletteShowLayoutEntry, PaletteShowProfile, PropertyFlag, PropertyFlags,
        PropertySelectorBuilder, load_palette_show_profile_set, parse_property_selector,
    },
    load_vox_main,
};
use clap::{ArgAction, Parser};
use std::num::NonZeroU8;
use voxcore::VoxMain;
use voxsmith::operations::palette::{
    PaletteShowLabel, PaletteShowLayout, PaletteShowOptions, PaletteShowTableShape,
    PropertySelector, palette_show,
};

/// Prints one or more palette value collections.
#[derive(Clone, Debug, Parser)]
#[command(name = "show")]
pub struct PaletteShow {
    #[command(flatten)]
    input: VoxelInput,

    /// Applies a profile's selectors and layout. Wherever the flag sits, the
    /// profile's selectors come ahead of every `--property` and
    /// `--properties-from` selector, with its `propertiesFrom` imports first.
    /// `--layout` replaces the profile's layout with every element it carries,
    /// and the other display flags refine the layout that results. Repeatable;
    /// the profiles stack in line order, and a layout two of them set errors.
    /// The profiles come from every `.vxlconfig`'s `palette.show.profiles`, the
    /// user's `~/.vxlconfig` first and then each directory from the git root
    /// down to the working directory. A name reads from the last file supplying
    /// it.
    #[arg(value_name = "profile", long, action = ArgAction::Append)]
    profile: Vec<String>,

    #[command(flatten)]
    property_flags: PropertyFlags,

    /// How to lay out the value collections. Defaults to `text-rows`.
    #[arg(
        value_name = "layout",
        long,
        value_parser = cli_value_parser::<PaletteShowLayout>()
    )]
    layout: Option<PaletteShowLayout>,

    /// How the text layouts label each value collection. Defaults to `concat`.
    #[arg(
        value_name = "label",
        long,
        value_parser = cli_value_parser::<PaletteShowLabel>()
    )]
    label: Option<PaletteShowLabel>,

    /// The Markdown level of the shallowest heading a heading-emitting render
    /// prints. Defaults to `1`, a `#` heading.
    #[arg(value_name = "header-level", long)]
    header_level: Option<NonZeroU8>,

    /// How the `md-tables` and `box-tables` layouts shape their tables.
    /// Defaults to `nested`.
    #[arg(
        value_name = "table-shape",
        long,
        value_parser = cli_value_parser::<PaletteShowTableShape>()
    )]
    table_shape: Option<PaletteShowTableShape>,

    /// Width the `text-rows` layout wraps to: `terminal` (default), `unlimited`,
    /// or a column count. Other layouts error on it.
    #[arg(value_name = "width", long)]
    width: Option<Width>,
}

impl PaletteShow {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let profiles = self
            .uses_profiles()
            .then(|| load_palette_show_profile_set(&dependencies))
            .transpose()?;

        let selectors = self.selectors(profiles.as_ref())?;

        let display = self.display(profiles.as_ref())?;

        let from = self.input.resolve_format()?;

        let main: VoxMain = load_vox_main(&dependencies, from, &self.input.path)?;

        // Only `text-rows` wraps, so it alone resolves the `terminal` default.
        let width = match display.layout {
            PaletteShowLayout::TextRows => {
                resolve_width(&dependencies, display.width.unwrap_or(Width::Terminal))
            }

            _ => None,
        };

        let options = PaletteShowOptions {
            layout: display.layout,
            label: display.label,
            header_level: display.header_level,
            table_shape: display.table_shape,
            width,
        };

        let output = palette_show(&main, &selectors, &options)?;

        Ok(dependencies.write_stdout(output.as_bytes())?)
    }

    /// Whether any flag reads a profile.
    fn uses_profiles(&self) -> bool {
        !self.profile.is_empty() || self.property_flags.uses_profiles()
    }

    /// The selectors: each `--profile`'s first, then the `--property` and
    /// `--properties-from` selectors in line order. `profiles` holds the loaded
    /// set when any flag reads a profile.
    fn selectors(
        &self,
        profiles: Option<&ProfileSet<PaletteShowProfile>>,
    ) -> Result<Vec<PropertySelector>> {
        let mut builder = PropertySelectorBuilder::new(profiles);

        for name in &self.profile {
            builder.land_profile("--profile", name)?;
        }

        for entry in &self.property_flags.entries {
            match entry {
                PropertyFlag::PropertiesFrom(name) => {
                    builder.land_profile("--properties-from", name)?;
                }

                PropertyFlag::Property([palette, property, presentation, reading]) => {
                    let selector =
                        parse_property_selector(palette, property, presentation, reading)
                            .map_err(|message| Error::usage(format!("--property: {message}")))?;

                    builder.push_selector(selector);
                }
            }
        }

        Ok(builder.finish())
    }

    /// The layout the run renders. `--layout` replaces the profile's layout with
    /// every element it carries, and the other display flags then refine the
    /// result. A width outside `text-rows` errors.
    fn display(
        &self,
        profiles: Option<&ProfileSet<PaletteShowProfile>>,
    ) -> Result<PaletteShowLayoutEntry> {
        let stacked = match profiles {
            Some(profiles) => {
                stack_palette_show_profiles(profiles, "--profile", &self.profile)?.layout
            }

            None => None,
        };

        let layout = self
            .layout
            .map(PaletteShowLayoutEntry::from)
            .or(stacked)
            .unwrap_or(PaletteShowLayoutEntry::from(PaletteShowLayout::TextRows));

        let display = PaletteShowLayoutEntry {
            layout: layout.layout,
            label: self.label.or(layout.label),
            header_level: self.header_level.or(layout.header_level),
            table_shape: self.table_shape.or(layout.table_shape),
            width: self.width.or(layout.width),
        };

        if display.width.is_some() && display.layout != PaletteShowLayout::TextRows {
            return Err(Error::usage(
                "a width was set, but the layout is not `text-rows`",
            ));
        }

        Ok(display)
    }
}

/// The column budget a `Width` resolves to, or `None` for no wrapping. A
/// `Terminal` width with no terminal on stdout, as when the output is piped,
/// also resolves to no wrapping.
fn resolve_width<D: Dependencies>(dependencies: &D, width: Width) -> Option<usize> {
    match width {
        Width::Unlimited => None,
        Width::Columns(columns) => Some(columns),
        Width::Terminal => dependencies.terminal_columns(),
    }
}

/// The profiles `names`, which `origin` lists, stacked into one profile to
/// apply whole. A layout two members both set errors. The stack carries no
/// selectors because each member lands its selectors and imports by name.
fn stack_palette_show_profiles(
    profiles: &ProfileSet<PaletteShowProfile>,
    origin: &str,
    names: &[String],
) -> Result<PaletteShowProfile> {
    let mut stack = PaletteShowProfile::default();
    let mut layout_claim: Option<&str> = None;

    for (position, name) in (0..).zip(names) {
        if names[..position].contains(name) {
            return Err(Error::usage(format!("{origin} lists `{name}` twice")));
        }

        let member = profiles.get(origin, name)?;

        if let Some(layout) = member.layout {
            if let Some(earlier) = layout_claim {
                return Err(Error::usage(format!(
                    "the profile `{name}` sets layout, which the profile `{earlier}` sets already"
                )));
            }

            layout_claim = Some(name);
            stack.layout = Some(layout);
        }
    }

    Ok(stack)
}

#[cfg(test)]
mod tests {
    use crate::{
        ProfileSet, Width,
        commands::{
            PaletteShow, PaletteShowLayoutEntry, PaletteShowProfile,
            palette::palette_show::palette_show::stack_palette_show_profiles,
            parse_property_selector,
        },
        owned_names, profile_set_from_json,
    };
    use clap::Parser;
    use voxsmith::operations::palette::{
        PaletteShowLabel, PaletteShowLayout, PaletteShowTableShape, PropertySelector,
    };

    /// The command parsed from `args` after the input.
    fn parse(args: &[&str]) -> PaletteShow {
        let mut argv = vec!["show", "model.voxj"];
        argv.extend_from_slice(args);
        PaletteShow::try_parse_from(argv).unwrap()
    }

    fn profiles() -> ProfileSet<PaletteShowProfile> {
        profile_set_from_json(&[
            (
                "pbr",
                r#"{ "properties": [{ "property": "baseColor" }, { "property": "metallic" }] }"#,
            ),
            (
                "orm",
                r#"{ "propertiesFrom": ["pbr"], "properties": [{ "property": "roughness" }], "layout": "box-tables" }"#,
            ),
            (
                "table",
                r#"{ "layout": { "kind": "md-tables", "tableShape": "flat" } }"#,
            ),
            (
                "rows",
                r#"{ "layout": { "kind": "text-rows", "width": 80 } }"#,
            ),
        ])
    }

    /// The selector for `property` with the other fields defaulted.
    fn selector(property: &str) -> PropertySelector {
        parse_property_selector("*", property, "auto", "auto").unwrap()
    }

    fn bare(layout: PaletteShowLayout) -> PaletteShowLayoutEntry {
        PaletteShowLayoutEntry::from(layout)
    }

    #[test]
    fn no_flag_selects_everything_as_text_rows_and_reads_no_profile() {
        let show = parse(&[]);

        assert!(!show.uses_profiles());
        assert_eq!(show.selectors(None).unwrap(), [PropertySelector::default()]);
        assert_eq!(
            show.display(None).unwrap(),
            bare(PaletteShowLayout::TextRows)
        );
    }

    #[test]
    fn a_bad_property_field_errors_naming_the_flag() {
        let show = parse(&["--property", "0", "baseColor", "rainbow", "auto"]);

        let error = show.selectors(None).unwrap_err().to_string();
        assert!(error.contains("--property: `rainbow`"), "{error}");
    }

    #[test]
    fn profile_selectors_come_ahead_of_the_line_ordered_flags() {
        let profiles = profiles();
        let show = parse(&[
            "--property",
            "*",
            "tint",
            "auto",
            "auto",
            "--properties-from",
            "orm",
            "--profile",
            "pbr",
        ]);

        assert!(show.uses_profiles());
        assert_eq!(
            show.selectors(Some(&profiles)).unwrap(),
            [
                selector("baseColor"),
                selector("metallic"),
                selector("tint"),
                selector("roughness"),
            ]
        );
    }

    #[test]
    fn properties_from_leaves_the_layout_behind() {
        let profiles = profiles();
        let show = parse(&["--properties-from", "orm"]);

        assert_eq!(
            show.display(Some(&profiles)).unwrap(),
            bare(PaletteShowLayout::TextRows)
        );
    }

    #[test]
    fn the_layout_flag_replaces_the_profile_layout_whole() {
        let profiles = profiles();

        let show = parse(&["--profile", "table", "--layout", "text-rows"]);
        assert_eq!(
            show.display(Some(&profiles)).unwrap(),
            bare(PaletteShowLayout::TextRows)
        );

        let show = parse(&["--profile", "table", "--layout", "md-tables"]);
        assert_eq!(
            show.display(Some(&profiles)).unwrap(),
            bare(PaletteShowLayout::MdTables)
        );
    }

    #[test]
    fn a_display_flag_refines_the_profile_layout() {
        let profiles = profiles();
        let show = parse(&[
            "--profile",
            "table",
            "--table-shape",
            "nested",
            "--label",
            "header",
        ]);

        assert_eq!(
            show.display(Some(&profiles)).unwrap(),
            PaletteShowLayoutEntry {
                label: Some(PaletteShowLabel::Header),
                table_shape: Some(PaletteShowTableShape::Nested),
                ..bare(PaletteShowLayout::MdTables)
            }
        );

        let show = parse(&["--profile", "rows", "--width", "unlimited"]);
        assert_eq!(
            show.display(Some(&profiles)).unwrap().width,
            Some(Width::Unlimited)
        );
    }

    #[test]
    fn a_width_outside_text_rows_errors() {
        let profiles = profiles();

        for args in [
            &["--layout", "md-tables", "--width", "80"][..],
            &["--layout", "json-pretty", "--width", "unlimited"],
            &["--profile", "table", "--width", "terminal"],
        ] {
            let error = parse(args)
                .display(Some(&profiles))
                .unwrap_err()
                .to_string();
            assert!(error.contains("a width was set"), "{args:?}: {error}");
        }

        assert!(
            parse(&["--profile", "rows", "--layout", "md-tables"])
                .display(Some(&profiles))
                .is_ok()
        );
    }

    #[test]
    fn two_profiles_setting_the_layout_error_even_under_the_flag() {
        let profiles = profiles();
        let show = parse(&[
            "--profile",
            "orm",
            "--profile",
            "table",
            "--layout",
            "text-rows",
        ]);

        let error = show.display(Some(&profiles)).unwrap_err().to_string();
        assert!(
            error.contains("the profile `table` sets layout, which the profile `orm` sets already"),
            "{error}"
        );
    }

    #[test]
    fn the_layout_stacks_and_the_selectors_stay_behind() {
        let profiles = profile_set_from_json(&[
            ("pbr", r#"{ "properties": [{ "property": "baseColor" }] }"#),
            (
                "table",
                r#"{ "layout": { "kind": "md-tables", "tableShape": "flat" } }"#,
            ),
        ]);

        let stack =
            stack_palette_show_profiles(&profiles, "--profile", &owned_names(&["pbr", "table"]))
                .unwrap();

        assert!(stack.properties.is_empty());
        assert_eq!(
            stack.layout,
            Some(PaletteShowLayoutEntry {
                table_shape: Some(PaletteShowTableShape::Flat),
                ..PaletteShowLayoutEntry::from(PaletteShowLayout::MdTables)
            })
        );
    }

    #[test]
    fn a_layout_two_members_set_errors_naming_both() {
        let profiles = profile_set_from_json(&[
            ("rows", r#"{ "layout": "text-rows" }"#),
            ("table", r#"{ "layout": "md-tables" }"#),
        ]);

        let error =
            stack_palette_show_profiles(&profiles, "--profile", &owned_names(&["rows", "table"]))
                .unwrap_err()
                .to_string();
        assert!(
            error
                .contains("the profile `table` sets layout, which the profile `rows` sets already"),
            "{error}"
        );
    }

    #[test]
    fn a_member_listed_twice_errors() {
        let profiles = profile_set_from_json(&[("rows", r#"{ "layout": "text-rows" }"#)]);

        let error =
            stack_palette_show_profiles(&profiles, "--profile", &owned_names(&["rows", "rows"]))
                .unwrap_err()
                .to_string();
        assert!(error.contains("--profile lists `rows` twice"), "{error}");
    }
}
