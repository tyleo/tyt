use crate::{
    Dependencies, Error, Result,
    commands::create_command::{CrateNaming, PROJECTS_DIR},
};
use std::path::Path;

/// Locates an existing command crate for `suffix`, returning its `projects/`
/// group directory and whether its name carries the `tyt-` prefix.
///
/// `dir` and `prefix` narrow the search when given; whatever is left out is
/// inferred by looking on disk for a scaffolded crate, recognized by its root
/// command-enum file `src/{module}.rs`. That file also pins down the prefix,
/// so a command crate like `tyt-vmax` is never confused with a same-named
/// library crate such as the `vmax` codec. Errors when nothing matches or when
/// more than one crate does.
pub fn find_parent_crate(
    deps: &impl Dependencies,
    root: &Path,
    suffix: &str,
    dir: Option<&str>,
    prefix: Option<bool>,
) -> Result<(String, bool)> {
    let projects = root.join(PROJECTS_DIR);

    let dirs: Vec<String> = match dir {
        Some(dir) => vec![dir.to_string()],
        None => {
            let mut groups: Vec<String> = deps
                .read_dir(&projects)?
                .into_iter()
                .filter(|path| path.is_dir())
                .filter_map(|path| {
                    path.file_name()
                        .and_then(|name| name.to_str())
                        .map(String::from)
                })
                .collect();
            groups.sort();
            groups
        }
    };
    let prefixes: Vec<bool> = match prefix {
        Some(prefix) => vec![prefix],
        None => vec![true, false],
    };

    let mut matches: Vec<(String, bool)> = Vec::new();
    for group in &dirs {
        for &prefixed in &prefixes {
            let naming = CrateNaming::new(prefixed, suffix);
            let enum_file = projects
                .join(group)
                .join(&naming.package)
                .join(format!("src/{}.rs", naming.module));
            if enum_file.is_file() {
                matches.push((group.clone(), prefixed));
            }
        }
    }

    match matches.as_slice() {
        [] => Err(Error::Meta(format!(
            "parent crate `{suffix}` not found under {}",
            projects.display()
        ))),
        [only] => Ok(only.clone()),
        many => {
            let locations: Vec<String> = many
                .iter()
                .map(|(group, prefixed)| {
                    format!(
                        "projects/{group}/{}",
                        CrateNaming::new(*prefixed, suffix).package
                    )
                })
                .collect();
            Err(Error::Meta(format!(
                "parent crate `{suffix}` is ambiguous; matched {}. Specify --dir and/or --prefix.",
                locations.join(", ")
            )))
        }
    }
}
