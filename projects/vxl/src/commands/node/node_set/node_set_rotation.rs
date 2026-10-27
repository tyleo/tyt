use crate::{
    Dependencies, RequiredSelection, Result, VoxelInput, VoxjOutput, cli_value_parser,
    edit_document, parse_finite_f64, vector3_f64,
};
use clap::{ArgAction, ArgGroup, Parser};
use ty_math::{TyAngleUnit, TyQuaternionExt, TyQuaternionF64};
use voxcore::BVoxHierarchyNode;
use voxsmith::operations::node::set_node_rotations;

/// Sets nodes' rotations from Euler angles or a quaternion.
#[derive(Clone, Debug, Parser)]
#[command(
    name = "rotation",
    group(ArgGroup::new("value").required(true).args(["rotation", "quaternion"]))
)]
pub struct NodeSetRotation {
    #[command(flatten)]
    input: VoxelInput,

    #[command(flatten)]
    output: VoxjOutput,

    #[command(flatten)]
    selection: RequiredSelection<BVoxHierarchyNode>,

    /// Euler angles about the fixed x, y, then z axes, as
    /// `node list --show-transforms` prints them, in `--unit` units. That
    /// listing prints `rad` by default.
    #[arg(
        value_names = ["x", "y", "z"],
        long,
        num_args = 3,
        allow_negative_numbers = true,
        action = ArgAction::Set,
        value_parser = parse_finite_f64
    )]
    rotation: Option<Vec<f64>>,

    /// The unit of the `--rotation` angles.
    #[arg(
        value_name = "unit",
        long,
        default_value = "deg",
        conflicts_with = "quaternion",
        value_parser = cli_value_parser::<TyAngleUnit>()
    )]
    unit: TyAngleUnit,

    /// The rotation as the unit quaternion the document stores.
    #[arg(
        value_names = ["x", "y", "z", "w"],
        long,
        num_args = 4,
        allow_negative_numbers = true,
        action = ArgAction::Set,
        value_parser = parse_finite_f64
    )]
    quaternion: Option<Vec<f64>>,
}

impl NodeSetRotation {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let rotation = self.rotation();

        edit_document(&dependencies, &self.input, self.output, |main| {
            let node_ids = self.selection.resolve_nodes(main)?;

            Ok(set_node_rotations(main, &node_ids, rotation)?)
        })
    }

    fn rotation(&self) -> TyQuaternionF64 {
        let Some(quaternion) = &self.quaternion else {
            let euler = vector3_f64(
                self.rotation
                    .as_deref()
                    .expect("clap requires --rotation or --quaternion"),
            );

            return TyQuaternionF64::from_euler_radians(
                euler.map(|angle| self.unit.to_radians(angle)),
            );
        };

        let [x, y, z, w] = quaternion[..] else {
            panic!("clap takes exactly four values, got {}", quaternion.len());
        };

        TyQuaternionF64::from_xyzw(x, y, z, w)
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::NodeSetRotation;
    use clap::{Error as ClapError, Parser};
    use ty_math::{TyQuaternionExt, TyQuaternionF64, TyVector3F64};

    fn parse(args: &[&str]) -> Result<NodeSetRotation, ClapError> {
        let mut argv = vec!["rotation", "scene.voxj", "--select", "house/door"];
        argv.extend_from_slice(args);

        NodeSetRotation::try_parse_from(argv)
    }

    #[test]
    fn euler_angles_default_to_degrees() {
        let rotation = parse(&["--rotation", "0", "37", "0"]).unwrap().rotation();

        let euler = rotation.to_euler_radians();

        assert!((euler - TyVector3F64::new(0.0, 37.0_f64.to_radians(), 0.0)).length() < 1e-12);
    }

    #[test]
    fn radians_and_quaternions_parse() {
        let radians = parse(&["--rotation", "0", "0", "-1.5", "--unit", "rad"])
            .unwrap()
            .rotation();

        assert!((radians.to_euler_radians().z + 1.5).abs() < 1e-12);

        let quaternion = parse(&["--quaternion", "0", "-1", "0", "0"])
            .unwrap()
            .rotation();

        assert_eq!(quaternion, TyQuaternionF64::from_xyzw(0.0, -1.0, 0.0, 0.0));
    }

    #[test]
    fn exactly_one_form_is_required_and_unit_needs_euler_angles() {
        assert!(parse(&[]).is_err());
        assert!(
            parse(&[
                "--rotation",
                "0",
                "0",
                "0",
                "--quaternion",
                "0",
                "0",
                "0",
                "1"
            ])
            .is_err()
        );
        assert!(parse(&["--quaternion", "0", "0", "0", "1", "--unit", "rad"]).is_err());
    }
}
