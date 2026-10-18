use crate::{
    Dependencies, Error, ProfileSet, ProgramFlag, ProgramFlags, Result, VoxelInput, VoxjOutput,
    check_expression,
    commands::{PaletteEditProfile, PaletteEditProgramBuilder, load_palette_edit_profile_set},
    edit_document, flag_occurrences, parse_id_selector,
};
use clap::{ArgAction, Parser};
use std::collections::BTreeMap;
use voxcore::BVoxPalette;
use voxsmith::{
    operations::palette::{PropertyWrite, edit_palettes},
    utilities::{IdSelector, resolve_palette_selectors},
};

/// Runs a value-language program over each selected palette and writes the
/// results into its properties. Each property the program or a write reads
/// enters as a swatch array holding one entry per material, in material order.
/// Every selected palette evaluates before any write lands, so an error leaves
/// the document untouched.
#[derive(Clone, Debug, Parser)]
#[command(name = "edit")]
pub struct PaletteEdit {
    #[command(flatten)]
    input: VoxelInput,

    #[command(flatten)]
    output: VoxjOutput,

    /// Which palettes to edit: an integer, an inclusive `a-b` range, or `*` for
    /// every palette. Repeatable; the union selects each palette once.
    #[arg(
        value_name = "palette",
        long,
        value_parser = parse_id_selector::<BVoxPalette>,
        default_value = "*"
    )]
    index: Vec<IdSelector<BVoxPalette>>,

    /// Applies a profile's values and property writes. Wherever the flag sits,
    /// the profile's values join the program ahead of every `--value` and
    /// `--values-from` binding, with its `valuesFrom` imports first. A
    /// `--write-property` flag replaces the profile's write of that property.
    /// Repeatable; the profiles stack in line order, and a property two of
    /// them write errors. The profiles come from every `.vxlconfig`'s
    /// `palette.edit.profiles`, the user's `~/.vxlconfig` first and then each
    /// directory from the git root down to the working directory. A name reads
    /// from the last file supplying it.
    #[arg(value_name = "profile", long, action = ArgAction::Append)]
    profile: Vec<String>,

    #[command(flatten)]
    program_flags: ProgramFlags,

    /// Evaluates the expression at the program's end and writes it to the
    /// property. A plain value lands on every material, and a swatch array
    /// lands one entry per material. Materials sharing a value that are
    /// written alike keep sharing it. A property the palette lacks is added on
    /// a new value pool. A property written twice errors. Repeatable.
    #[arg(
        value_names = ["dst-property", "src-expr"],
        long,
        num_args = 2,
        action = ArgAction::Append,
    )]
    write_property: Vec<String>,
}

impl PaletteEdit {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let profiles = self
            .uses_profiles()
            .then(|| load_palette_edit_profile_set(&dependencies))
            .transpose()?;

        let program = self.program(profiles.as_ref())?;

        let writes = self.writes(profiles.as_ref())?;

        edit_document(&dependencies, &self.input, self.output, |main| {
            let palette_ids = resolve_palette_selectors(main, &self.index)?;

            Ok(edit_palettes(main, &palette_ids, &program, &writes)?)
        })
    }

    /// Whether any flag reads a profile.
    fn uses_profiles(&self) -> bool {
        !self.profile.is_empty() || self.program_flags.uses_profiles()
    }

    /// The program: each `--profile`'s values first, then the `--value` and
    /// `--values-from` bindings in line order. `profiles` holds the loaded set
    /// when any flag reads a profile.
    fn program(&self, profiles: Option<&ProfileSet<PaletteEditProfile>>) -> Result<String> {
        let mut builder = PaletteEditProgramBuilder::new(profiles);

        for name in &self.profile {
            builder.land_profile("--profile", name)?;
        }

        for flag in &self.program_flags.entries {
            match flag {
                ProgramFlag::Value(text) => builder.push_value(text)?,
                ProgramFlag::ValuesFrom(name) => builder.land_profile("--values-from", name)?,
            }
        }

        Ok(builder.finish())
    }

    /// The writes: each `--write-property` in line order, then the profile
    /// stack's writes of every other property in property order. `profiles`
    /// holds the loaded set when any flag reads a profile.
    fn writes(
        &self,
        profiles: Option<&ProfileSet<PaletteEditProfile>>,
    ) -> Result<Vec<PropertyWrite>> {
        let mut writes: Vec<PropertyWrite> = Vec::new();

        for [property, expression] in flag_occurrences::<2>(&self.write_property) {
            let flag = "--write-property";

            if writes.iter().any(|write| &write.property == property) {
                return Err(Error::usage(format!("{flag} writes `{property}` twice")));
            }

            check_expression(flag, expression)?;

            writes.push(PropertyWrite {
                property: property.clone(),
                expression: expression.clone(),
            });
        }

        if self.profile.is_empty() {
            return Ok(writes);
        }

        let stack = stack_palette_edit_profiles(
            profiles.expect("--profile loads the profiles"),
            "--profile",
            &self.profile,
        )?;

        for (property, stacked) in stack {
            if writes.iter().any(|write| write.property == property) {
                continue;
            }

            check_expression(
                &format!(
                    "the profile `{}`'s properties key `{property}`",
                    stacked.profile
                ),
                stacked.expression,
            )?;

            writes.push(PropertyWrite {
                property: property.to_owned(),
                expression: stacked.expression.to_owned(),
            });
        }

        Ok(writes)
    }
}

/// A profile stack's write of one property.
#[derive(Debug, PartialEq)]
struct StackedWrite<'a> {
    /// The member writing the property.
    profile: &'a str,

    expression: &'a str,
}

/// The property writes of the profiles `names`, which `origin` lists, stacked
/// by property. A property two members both write errors. The stack carries no
/// values because each member lands its values and imports by name.
fn stack_palette_edit_profiles<'a>(
    profiles: &'a ProfileSet<PaletteEditProfile>,
    origin: &str,
    names: &'a [String],
) -> Result<BTreeMap<&'a str, StackedWrite<'a>>> {
    let mut stack: BTreeMap<&str, StackedWrite<'_>> = BTreeMap::new();

    for (position, name) in (0..).zip(names) {
        if names[..position].contains(name) {
            return Err(Error::usage(format!("{origin} lists `{name}` twice")));
        }

        let member = profiles.get(origin, name)?;

        for (property, expression) in &member.properties {
            if let Some(earlier) = stack.get(property.as_str()) {
                return Err(Error::usage(format!(
                    "the profile `{name}` sets properties key `{property}`, which the profile \
                     `{}` sets already",
                    earlier.profile
                )));
            }

            stack.insert(
                property,
                StackedWrite {
                    profile: name,
                    expression,
                },
            );
        }
    }

    Ok(stack)
}

#[cfg(test)]
mod tests {
    use crate::{
        ProfileSet,
        commands::{
            PaletteEdit, PaletteEditProfile,
            palette::palette_edit::palette_edit::{StackedWrite, stack_palette_edit_profiles},
        },
        owned_names, profile_set_from_json,
    };
    use branded_id::U32Id;
    use clap::Parser;
    use voxsmith::{operations::palette::PropertyWrite, utilities::IdSelector};

    /// The command parsed from `args` after the input.
    fn parse(args: &[&str]) -> PaletteEdit {
        let mut argv = vec!["edit", "robot.voxj"];
        argv.extend_from_slice(args);
        PaletteEdit::try_parse_from(argv).unwrap()
    }

    fn profiles() -> ProfileSet<PaletteEditProfile> {
        profile_set_from_json(&[
            ("tags", r#"{ "values": ["rust = tag == \"rust\""] }"#),
            (
                "weathered",
                r#"{ "valuesFrom": ["tags"], "properties": { "roughness": "mix(roughness, 0.9, rust)" } }"#,
            ),
            (
                "polished",
                r#"{ "valuesFrom": ["tags"], "properties": { "metallic": "1.0", "roughness": "0.1" } }"#,
            ),
            ("broken", r#"{ "properties": { "metallic": "1 +" } }"#),
        ])
    }

    fn write(property: &str, expression: &str) -> PropertyWrite {
        PropertyWrite {
            property: property.to_owned(),
            expression: expression.to_owned(),
        }
    }

    #[test]
    fn index_defaults_to_every_palette_and_repeats() {
        assert_eq!(parse(&[]).index, [IdSelector::all()]);

        assert_eq!(
            parse(&["--index", "2", "--index", "0-1"]).index,
            [
                IdSelector::id(U32Id::from_u32(2)),
                IdSelector::range(U32Id::from_u32(0)..=U32Id::from_u32(1)).unwrap(),
            ]
        );
    }

    #[test]
    fn no_flag_runs_an_empty_program_and_reads_no_profile() {
        let edit = parse(&[]);

        assert!(!edit.uses_profiles());
        assert_eq!(edit.program(None).unwrap(), "");
        assert_eq!(edit.writes(None).unwrap(), []);
    }

    #[test]
    fn the_writes_keep_their_line_order() {
        let edit = parse(&[
            "--write-property",
            "wear",
            "0.5",
            "--write-property",
            "roughness",
            "wear",
        ]);

        assert_eq!(
            edit.writes(None).unwrap(),
            [write("wear", "0.5"), write("roughness", "wear")]
        );
    }

    #[test]
    fn a_property_written_twice_errors() {
        let edit = parse(&[
            "--write-property",
            "wear",
            "0.5",
            "--write-property",
            "wear",
            "0.6",
        ]);

        let error = edit.writes(None).unwrap_err().to_string();
        assert!(
            error.contains("--write-property writes `wear` twice"),
            "{error}"
        );
    }

    #[test]
    fn a_broken_write_errors_at_its_flag() {
        let edit = parse(&["--write-property", "wear", "1 +"]);

        let error = edit.writes(None).unwrap_err().to_string();
        assert!(error.contains("--write-property holds `1 +`"), "{error}");
    }

    #[test]
    fn profile_values_come_ahead_of_the_line_ordered_flags() {
        let profiles = profiles();
        let edit = parse(&[
            "--value",
            "a = 1",
            "--values-from",
            "tags",
            "--profile",
            "weathered",
        ]);

        assert!(edit.uses_profiles());
        assert_eq!(
            edit.program(Some(&profiles)).unwrap(),
            "rust = tag == \"rust\";\na = 1;"
        );
    }

    #[test]
    fn values_from_leaves_the_properties_behind() {
        let profiles = profiles();
        let edit = parse(&["--values-from", "weathered"]);

        assert!(edit.uses_profiles());
        assert_eq!(edit.writes(Some(&profiles)).unwrap(), []);
    }

    #[test]
    fn a_flag_replaces_the_stack_write_and_the_rest_fill_in_property_order() {
        let profiles = profiles();
        let edit = parse(&[
            "--profile",
            "polished",
            "--write-property",
            "roughness",
            "0.5",
        ]);

        assert_eq!(
            edit.writes(Some(&profiles)).unwrap(),
            [write("roughness", "0.5"), write("metallic", "1.0")]
        );
    }

    #[test]
    fn a_broken_profile_write_errors_at_its_key_unless_a_flag_replaces_it() {
        let profiles = profiles();

        let error = parse(&["--profile", "broken"])
            .writes(Some(&profiles))
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("the profile `broken`'s properties key `metallic` holds `1 +`"),
            "{error}"
        );

        assert!(
            parse(&["--profile", "broken", "--write-property", "metallic", "1"])
                .writes(Some(&profiles))
                .is_ok()
        );
    }

    #[test]
    fn the_writes_stack_by_property() {
        let profiles = profiles();
        let names = owned_names(&["tags", "weathered"]);

        let stack = stack_palette_edit_profiles(&profiles, "--profile", &names).unwrap();

        assert_eq!(
            stack.into_iter().collect::<Vec<_>>(),
            [(
                "roughness",
                StackedWrite {
                    profile: "weathered",
                    expression: "mix(roughness, 0.9, rust)",
                }
            )]
        );
    }

    #[test]
    fn a_property_two_members_write_errors_naming_both() {
        let profiles = profiles();

        let error = stack_palette_edit_profiles(
            &profiles,
            "--profile",
            &owned_names(&["weathered", "polished"]),
        )
        .unwrap_err()
        .to_string();
        assert!(
            error.contains(
                "the profile `polished` sets properties key `roughness`, which the profile \
                 `weathered` sets already"
            ),
            "{error}"
        );
    }

    #[test]
    fn a_member_listed_twice_errors() {
        let profiles = profiles();

        let error =
            stack_palette_edit_profiles(&profiles, "--profile", &owned_names(&["tags", "tags"]))
                .unwrap_err()
                .to_string();
        assert!(error.contains("--profile lists `tags` twice"), "{error}");
    }

    #[test]
    fn an_undefined_profile_errors_at_its_flag() {
        let profiles = profiles();

        let error = parse(&["--profile", "matte"])
            .program(Some(&profiles))
            .unwrap_err()
            .to_string();
        assert!(error.contains("--profile"), "{error}");
        assert!(error.contains("`matte`"), "{error}");
    }
}
