use crate::{
    Dependencies, Result,
    commands::{ObjectVoxelsFlip, ObjectVoxelsQuantize, ObjectVoxelsRotate, ObjectVoxelsTranslate},
};
use clap::Subcommand;

/// The `object voxels` command group.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum ObjectVoxelsCommand {
    #[command(name = "flip")]
    ObjectVoxelsFlip(ObjectVoxelsFlip),

    #[command(name = "quantize")]
    ObjectVoxelsQuantize(ObjectVoxelsQuantize),

    #[command(name = "rotate")]
    ObjectVoxelsRotate(ObjectVoxelsRotate),

    #[command(name = "translate")]
    ObjectVoxelsTranslate(ObjectVoxelsTranslate),
}

impl ObjectVoxelsCommand {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        match self {
            ObjectVoxelsCommand::ObjectVoxelsFlip(flip) => flip.execute(dependencies),

            ObjectVoxelsCommand::ObjectVoxelsQuantize(quantize) => quantize.execute(dependencies),

            ObjectVoxelsCommand::ObjectVoxelsRotate(rotate) => rotate.execute(dependencies),

            ObjectVoxelsCommand::ObjectVoxelsTranslate(translate) => {
                translate.execute(dependencies)
            }
        }
    }
}
