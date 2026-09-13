use crate::{ListDir, ReadFile, TerminalColumns, WriteFile, WriteStdout};
use meshconv::{
    Dependencies as MeshconvDependencies, ListDir as MeshListDir, ReadFile as MeshReadFile,
    WriteFile as MeshWriteFile,
};
use voxconv::Dependencies as VoxconvDependencies;

/// The side effects the commands perform, injected so they run over any
/// filesystem and output. The voxel codecs come through voxconv's
/// [`Dependencies`](voxconv::Dependencies) and the mesh codecs through
/// meshconv's [`Dependencies`](meshconv::Dependencies); [`DependenciesImpl`]
/// forwards each to its crate's impl. Each crate reads and writes files
/// through its own file traits. The commands reach a file only through
/// `voxconv::load` and `save` or `meshconv::load` and `save`, so the
/// same-named traits never meet at a call site. voxsmith's encoders and
/// decoders are not injected; the commands bind its impl because this crate
/// is the command line and always wants the real one.
///
/// [`DependenciesImpl`]: crate::DependenciesImpl
pub trait Dependencies:
    ReadFile
    + ListDir
    + WriteFile
    + WriteStdout
    + TerminalColumns
    + VoxconvDependencies
    + MeshconvDependencies
    + MeshReadFile
    + MeshListDir
    + MeshWriteFile
{
}

impl<
    T: ReadFile
        + ListDir
        + WriteFile
        + WriteStdout
        + TerminalColumns
        + VoxconvDependencies
        + MeshconvDependencies
        + MeshReadFile
        + MeshListDir
        + MeshWriteFile,
> Dependencies for T
{
}
