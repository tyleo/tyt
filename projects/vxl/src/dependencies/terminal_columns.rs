/// Reads the terminal's width.
pub trait TerminalColumns {
    /// The column count of the terminal on standard output, or `None` when
    /// standard output is not a terminal.
    fn terminal_columns(&self) -> Option<usize>;
}
