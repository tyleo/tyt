use crate::{ListDir, ReadFile, TerminalColumns, WriteFile, WriteStdout};
use voxconv::Dependencies as VoxconvDependencies;

/// The side effects the commands perform, injected so they run over any
/// filesystem and output. The format codecs come through voxconv's
/// [`Dependencies`](voxconv::Dependencies), which [`DependenciesImpl`]
/// forwards to voxconv's impl. voxsmith's encoders are not injected. The
/// commands bind voxsmith's impl because this crate is the command line and
/// always wants the real one.
///
/// [`DependenciesImpl`]: crate::DependenciesImpl
pub trait Dependencies:
    ReadFile + ListDir + WriteFile + WriteStdout + TerminalColumns + VoxconvDependencies
{
}

impl<T: ReadFile + ListDir + WriteFile + WriteStdout + TerminalColumns + VoxconvDependencies>
    Dependencies for T
{
}
