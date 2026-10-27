use crate::{
    Dependencies, RequiredSelection, Result, VoxelInput, VoxjOutput, cli_value_parser,
    edit_document,
};
use clap::Parser;
use ty_math::TyAxis3;
use voxcore::BVoxObject;
use voxsmith::operations::object::flip_object_voxels;

/// Mirrors objects' voxels in place along one grid axis.
#[derive(Clone, Debug, Parser)]
#[command(name = "flip")]
pub struct ObjectVoxelsFlip {
    #[command(flatten)]
    input: VoxelInput,

    #[command(flatten)]
    output: VoxjOutput,

    #[command(flatten)]
    selection: RequiredSelection<BVoxObject>,

    /// The grid axis to mirror along.
    #[arg(value_name = "axis", long, value_parser = cli_value_parser::<TyAxis3>())]
    axis: TyAxis3,
}

impl ObjectVoxelsFlip {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        edit_document(&dependencies, &self.input, self.output, |main| {
            let object_ids = self.selection.resolve_objects(main)?;

            Ok(flip_object_voxels(main, &object_ids, self.axis)?)
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::ObjectVoxelsFlip;
    use clap::Parser;
    use ty_math::TyAxis3;

    #[test]
    fn parses_the_axis() {
        let flip = ObjectVoxelsFlip::try_parse_from([
            "flip",
            "scene.voxj",
            "--select",
            "crate",
            "--axis",
            "z",
        ])
        .unwrap();

        assert_eq!(flip.axis, TyAxis3::Z);
        assert!(
            ObjectVoxelsFlip::try_parse_from([
                "flip",
                "scene.voxj",
                "--select",
                "crate",
                "--axis",
                "w"
            ])
            .is_err()
        );
    }
}
