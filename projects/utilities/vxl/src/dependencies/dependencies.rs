use crate::{ListDir, ReadFile, ResolvePrefsPaths, TerminalColumns, WriteFile, WriteStdout};
use meshconv::{
    Dependencies as MeshconvDependencies, ListDir as MeshListDir, ReadFile as MeshReadFile,
    WriteFile as MeshWriteFile,
};
use ty_preferences::Dependencies as PreferencesDependencies;
use voxconv::Dependencies as VoxconvDependencies;

/// The side effects the commands perform, injected so they run over any
/// filesystem and output. voxconv, meshconv, and ty-preferences each read
/// files through their own traits, and the commands reach a file only through
/// those crates' loaders, so the same-named methods never meet at a call site.
/// voxsmith's codecs are not injected because this crate is the command line
/// and always wants the real ones.
pub trait Dependencies:
    ReadFile
    + ListDir
    + WriteFile
    + WriteStdout
    + TerminalColumns
    + ResolvePrefsPaths
    + VoxconvDependencies
    + MeshconvDependencies
    + MeshReadFile
    + MeshListDir
    + MeshWriteFile
    + PreferencesDependencies
{
}

impl<
    T: ReadFile
        + ListDir
        + WriteFile
        + WriteStdout
        + TerminalColumns
        + ResolvePrefsPaths
        + VoxconvDependencies
        + MeshconvDependencies
        + MeshReadFile
        + MeshListDir
        + MeshWriteFile
        + PreferencesDependencies,
> Dependencies for T
{
}
