use crate::{
    CreateDirAll, CreateDirLink, CreateTempDir, DisplayImage, ListDir, ReadFile, ReadPathKind,
    RemoveDir, RenamePath, ResolvePrefsPaths, RunProgram, TerminalColumns, WriteFile, WriteStderr,
    WriteStdout,
};
use meshconv::{
    Dependencies as MeshconvDependencies, ListDir as MeshListDir, ReadFile as MeshReadFile,
    WriteFile as MeshWriteFile,
};
use sdfconv::{Dependencies as SdfconvDependencies, ReadFile as SdfReadFile};
use ty_preferences::Dependencies as PreferencesDependencies;
use voxconv::Dependencies as VoxconvDependencies;

/// The side effects the commands perform, injected so they run over any
/// filesystem and output. voxconv, meshconv, sdfconv, and ty-preferences each
/// read files through their own traits, and the commands reach a file only
/// through those crates' loaders, so the same-named methods never meet at a
/// call site. voxsmith's codecs are not injected because this crate is the
/// command line and always wants the real ones.
pub trait Dependencies:
    ReadFile
    + ListDir
    + WriteFile
    + WriteStderr
    + WriteStdout
    + TerminalColumns
    + DisplayImage
    + ResolvePrefsPaths
    + CreateTempDir
    + RunProgram
    + ReadPathKind
    + CreateDirAll
    + CreateDirLink
    + RenamePath
    + RemoveDir
    + VoxconvDependencies
    + MeshconvDependencies
    + MeshReadFile
    + MeshListDir
    + MeshWriteFile
    + SdfconvDependencies
    + SdfReadFile
    + PreferencesDependencies
{
}

impl<
    T: ReadFile
        + ListDir
        + WriteFile
        + WriteStderr
        + WriteStdout
        + TerminalColumns
        + DisplayImage
        + ResolvePrefsPaths
        + CreateTempDir
        + RunProgram
        + ReadPathKind
        + CreateDirAll
        + CreateDirLink
        + RenamePath
        + RemoveDir
        + CreateDirAll
        + CreateDirLink
        + RenamePath
        + RemoveDir
        + ReadPathKind
        + CreateDirAll
        + CreateDirLink
        + RenamePath
        + RemoveDir
        + CreateDirAll
        + CreateDirLink
        + RenamePath
        + RemoveDir
        + VoxconvDependencies
        + MeshconvDependencies
        + MeshReadFile
        + MeshListDir
        + MeshWriteFile
        + SdfconvDependencies
        + SdfReadFile
        + PreferencesDependencies,
> Dependencies for T
{
}
