use crate::{Result, WriteStderr};
use voxcore::{
    VoxState,
    check::{VoxCheckStatus, check_slash_free_names},
};

/// Warns on standard error about each node and object name holding `/`,
/// which a `--select` path reads as a step to a child.
pub fn warn_slashed_names(dependencies: &impl WriteStderr, state: &VoxState) -> Result<()> {
    let messages = match check_slash_free_names(state).status {
        VoxCheckStatus::Failed(messages) => messages,
        VoxCheckStatus::Passed => return Ok(()),
        VoxCheckStatus::Unverifiable => unreachable!("a state holds every name it checks"),
    };

    let mut warnings: String = messages
        .iter()
        .map(|message| format!("warning: {message}\n"))
        .collect();
    warnings.push_str(
        "warning: a --select path reads \"/\" as a step to a child; --select-index picks by id\n",
    );

    Ok(dependencies.write_stderr(warnings.as_bytes())?)
}

#[cfg(test)]
mod tests {
    use crate::{WriteStderr, warn_slashed_names};
    use std::{cell::RefCell, io::Result as IOResult};
    use voxcore::{VoxHierarchyNode, VoxMain};

    #[derive(Default)]
    struct CapturedStderr {
        stderr: RefCell<Vec<u8>>,
    }

    impl WriteStderr for CapturedStderr {
        fn write_stderr(&self, contents: &[u8]) -> IOResult<()> {
            self.stderr.borrow_mut().extend_from_slice(contents);

            Ok(())
        }
    }

    fn main_named(name: &str) -> VoxMain {
        let mut main = VoxMain::default();

        let node = VoxHierarchyNode {
            name: name.to_owned(),
            ..Default::default()
        };

        main.retain_hierarchy_node(node).unwrap();

        main
    }

    #[test]
    fn a_slashed_name_warns_and_points_to_select_index() {
        let dependencies = CapturedStderr::default();

        warn_slashed_names(&dependencies, main_named("a/b").state()).unwrap();

        assert_eq!(
            String::from_utf8(dependencies.stderr.take()).unwrap(),
            "warning: hierarchy node 0 name \"a/b\" holds \"/\"\n\
             warning: a --select path reads \"/\" as a step to a child; --select-index picks by \
             id\n"
        );
    }

    #[test]
    fn slash_free_names_write_nothing() {
        let dependencies = CapturedStderr::default();

        warn_slashed_names(&dependencies, main_named("house").state()).unwrap();

        assert!(dependencies.stderr.take().is_empty());
    }
}
