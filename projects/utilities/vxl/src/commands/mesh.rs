use crate::{Dependencies, Format, MeshFormat, MeshMethod, Result, SelectIndex};
use clap::Parser;
use std::{
    io::{Error as IOError, ErrorKind},
    path::PathBuf,
};

/// Triangulates one object's voxels into a glTF or GLB mesh as pure geometry,
/// with no hierarchy-node transform applied.
#[derive(Clone, Debug, Parser)]
#[command(name = "mesh")]
pub struct Mesh {
    /// The input voxel file, in any supported format.
    #[arg(value_name = "input")]
    input: PathBuf,

    /// The output mesh. Defaults to the input path with the mesh extension.
    #[arg(value_name = "output")]
    output: Option<PathBuf>,

    /// Target mesh format, glTF text (`.gltf`) or binary (`.glb`). Inferred from
    /// the output extension when omitted, defaulting to `.glb`.
    #[arg(value_name = "to", long)]
    to: Option<MeshFormat>,

    /// Source voxel format. Inferred from the input extension when omitted.
    #[arg(value_name = "from", long)]
    from: Option<Format>,

    /// Real-world edge length of one voxel in meters, applied as a uniform scale
    /// to every output vertex.
    #[arg(value_name = "scale", long, default_value = "1.0")]
    scale: f64,

    /// Meshing strategy.
    #[arg(value_name = "method", long, default_value = "greedy")]
    method: MeshMethod,

    /// Choose the object by hierarchy-path glob, matched as `hierarchy show`
    /// matches node paths, so a node path selects its subtree. Repeatable;
    /// unions with `--select-index`.
    #[arg(value_name = "select", long)]
    select: Vec<String>,

    /// Choose the object by index, an integer or an `a-b` range. Repeatable;
    /// unions with `--select`.
    #[arg(value_name = "select-index", long)]
    select_index: Vec<SelectIndex>,
}

impl Mesh {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        if self.scale <= 0.0 || self.scale.is_nan() {
            return Err(usage("--scale must be greater than 0"));
        }

        let format = self
            .to
            .or_else(|| self.output.as_deref().and_then(MeshFormat::from_path))
            .unwrap_or(MeshFormat::Glb);

        let output = self
            .output
            .unwrap_or_else(|| self.input.with_extension(format.extension()));

        let objects = dependencies.resolve_objects(
            &self.input,
            self.from,
            &self.select,
            &self.select_index,
        )?;

        // `mesh` outputs pure geometry for one object, so the selection must
        // name exactly one; the resolver stays flag-agnostic and this policy,
        // with its flag-named guidance, lives here on the command.
        let object = match objects.as_slice() {
            [object] => *object,

            [] => {
                return Err(usage(
                    "no object matched the selection; check --select and --select-index",
                ));
            }

            objects => {
                return Err(usage(&format!(
                    "the selection resolved to {} objects, but `mesh` outputs exactly one; \
                     narrow it with --select or --select-index",
                    objects.len(),
                )));
            }
        };

        dependencies.mesh_object(
            &self.input,
            self.from,
            &output,
            format,
            self.scale,
            self.method,
            object,
        )
    }
}

/// A usage error for a rule clap cannot express, exiting non-zero with a
/// message.
fn usage(message: &str) -> crate::Error {
    IOError::new(ErrorKind::InvalidInput, message).into()
}
