use crate::{ListDir, ReadFile, TerminalColumns, WriteFile, WriteStdout};

/// The side effects the commands perform, injected so they run over any
/// filesystem and output. The format codecs and voxsmith's encoders are not
/// injected. The commands bind voxconv's and voxsmith's impls because this
/// crate is the command line and always wants the real ones.
pub trait Dependencies: ReadFile + ListDir + WriteFile + WriteStdout + TerminalColumns {}

impl<T: ReadFile + ListDir + WriteFile + WriteStdout + TerminalColumns> Dependencies for T {}
