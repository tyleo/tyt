use crate::{
    Dependencies, Result,
    commands::{
        ObjectAdd, ObjectDownsample, ObjectDuplicate, ObjectLink, ObjectMesh, ObjectRemove,
        ObjectRender, ObjectReorder, ObjectSet, ObjectTrim, ObjectUnlink, ObjectUpsample,
        ObjectVoxels,
    },
};
use clap::Subcommand;

/// The `object` command group.
// The command line parses once, so the variants' sizes never matter.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum ObjectCommand {
    #[command(name = "add")]
    ObjectAdd(ObjectAdd),

    #[command(name = "downsample")]
    ObjectDownsample(ObjectDownsample),

    #[command(name = "duplicate")]
    ObjectDuplicate(ObjectDuplicate),

    #[command(name = "link")]
    ObjectLink(ObjectLink),

    #[command(name = "mesh")]
    ObjectMesh(ObjectMesh),

    #[command(name = "remove")]
    ObjectRemove(ObjectRemove),

    #[command(name = "render")]
    ObjectRender(ObjectRender),

    #[command(name = "reorder")]
    ObjectReorder(ObjectReorder),

    #[command(name = "set")]
    ObjectSet(ObjectSet),

    #[command(name = "trim")]
    ObjectTrim(ObjectTrim),

    #[command(name = "unlink")]
    ObjectUnlink(ObjectUnlink),

    #[command(name = "upsample")]
    ObjectUpsample(ObjectUpsample),

    #[command(name = "voxels")]
    ObjectVoxels(ObjectVoxels),
}

impl ObjectCommand {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        match self {
            ObjectCommand::ObjectAdd(add) => add.execute(dependencies),
            ObjectCommand::ObjectDownsample(downsample) => downsample.execute(dependencies),
            ObjectCommand::ObjectDuplicate(duplicate) => duplicate.execute(dependencies),
            ObjectCommand::ObjectLink(link) => link.execute(dependencies),
            ObjectCommand::ObjectMesh(mesh) => mesh.execute(dependencies),
            ObjectCommand::ObjectRemove(remove) => remove.execute(dependencies),
            ObjectCommand::ObjectRender(render) => render.execute(dependencies),
            ObjectCommand::ObjectReorder(reorder) => reorder.execute(dependencies),
            ObjectCommand::ObjectSet(set) => set.execute(dependencies),
            ObjectCommand::ObjectTrim(trim) => trim.execute(dependencies),
            ObjectCommand::ObjectUnlink(unlink) => unlink.execute(dependencies),
            ObjectCommand::ObjectUpsample(upsample) => upsample.execute(dependencies),
            ObjectCommand::ObjectVoxels(voxels) => voxels.execute(dependencies),
        }
    }
}
