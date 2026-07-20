use crate::{TreeGridError, TreeGridOptions};

impl TreeGridOptions {
    /// Checks the options for the JSON renders, which consume none:
    /// every set option is rejected.
    pub fn resolve_json(&self) -> Result<(), TreeGridError> {
        self.no_label()?;
        self.no_width()?;
        self.no_header_level()?;
        self.no_table_shape()?;
        self.no_hierarchy_options()
    }
}

#[cfg(test)]
mod tests {
    use crate::{TreeGridError, TreeGridOptions};

    #[test]
    fn json_resolves_bare_and_rejects_every_option() {
        assert_eq!(TreeGridOptions::default().resolve_json(), Ok(()));
        assert_eq!(
            TreeGridOptions::default().with_width(80).resolve_json(),
            Err(TreeGridError::WidthWithoutRows)
        );
    }
}
