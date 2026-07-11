use crate::{
    Dependencies, FillColor, FillMode, GridResolution, MaterialMode, MaxPaletteMaterials,
    MeshFormat, PaletteReductionOptions, Result, VoxjEncodingOptions,
};
use clap::{ArgGroup, Parser};
use std::{
    io::{Error as IOError, ErrorKind},
    path::PathBuf,
};

/// Rasterizes a mesh into a voxel grid, the inverse of `mesh`.
#[derive(Clone, Debug, Parser)]
#[command(
    name = "voxelize",
    group(ArgGroup::new("resolution").required(true).args(["voxel_grid_length", "meters_per_voxel"]))
)]
pub struct Voxelize {
    /// The input glTF (`.gltf`) or GLB (`.glb`) mesh.
    #[arg(value_name = "input")]
    input: PathBuf,

    /// The output `.voxj` or `.voxjz` document to write. Defaults to the input
    /// path with a `.voxj` extension, or `.voxjz` when `--format zip`.
    #[arg(value_name = "output")]
    output: Option<PathBuf>,

    /// Source mesh format. Inferred from the input extension when omitted.
    #[arg(value_name = "from", long)]
    from: Option<MeshFormat>,

    /// Grid resolution in voxels along the longest axis; the other axes are
    /// sized to preserve aspect. Mutually exclusive with `--meters-per-voxel`.
    #[arg(value_name = "voxel-grid-length", long)]
    voxel_grid_length: Option<u32>,

    /// Edge length of one voxel in meters; each axis count is the mesh extent on
    /// that axis divided by this, rounded up. Recorded as the placing node's
    /// scale so the model keeps its source size. Mutually exclusive with
    /// `--voxel-grid-length`.
    #[arg(value_name = "meters-per-voxel", long)]
    meters_per_voxel: Option<f64>,

    /// How the mesh fills the grid, independent of `--material-mode`.
    #[arg(value_name = "fill-mode", long, default_value = "solid")]
    fill_mode: FillMode,

    /// Where each voxel's color comes from, independent of `--fill-mode`.
    #[arg(value_name = "material-mode", long, default_value = "auto")]
    material_mode: MaterialMode,

    /// Color of voxels a mode cannot sample: `none` or a `#RRGGBBAA` hex.
    #[arg(value_name = "fill-color", long, default_value = "none")]
    fill_color: FillColor,

    /// Name for the voxelized object. Defaults to the mesh's own name, else the
    /// input file stem.
    #[arg(value_name = "name", long)]
    name: Option<String>,

    /// Most materials the palette may hold; sampling over this reduces to it.
    /// `none` disables the cap.
    #[arg(value_name = "max-palette-materials", long, default_value = "256")]
    max_palette_materials: MaxPaletteMaterials,

    #[command(flatten)]
    reduction_options: PaletteReductionOptions,

    #[command(flatten)]
    encoding_options: VoxjEncodingOptions,
}

impl Voxelize {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        if let Some(meters) = self.meters_per_voxel
            && (meters <= 0.0 || meters.is_nan())
        {
            return Err(usage("--meters-per-voxel must be greater than 0"));
        }

        if self.voxel_grid_length == Some(0) {
            return Err(usage("--voxel-grid-length must be at least 1"));
        }

        // The clap `ArgGroup` guarantees exactly one of the two is set.
        let resolution = match (self.voxel_grid_length, self.meters_per_voxel) {
            (Some(length), _) => GridResolution::VoxelGridLength(length),
            (_, Some(meters)) => GridResolution::MetersPerVoxel(meters),
            (None, None) => return Err(usage("set --voxel-grid-length or --meters-per-voxel")),
        };

        let format = self.encoding_options.resolve_format(self.output.as_deref());

        let output = self
            .output
            .unwrap_or_else(|| self.input.with_extension(format.extension()));

        let encoding = self.encoding_options.encoding();

        let color_format = self.encoding_options.color_format();

        let reduction = self.reduction_options.resolve(self.max_palette_materials);

        dependencies.voxelize(
            &self.input,
            self.from,
            &output,
            resolution,
            self.fill_mode,
            self.material_mode,
            self.fill_color.rgba(),
            self.name.as_deref(),
            reduction,
            encoding,
            format,
            color_format,
        )
    }
}

/// A usage error for a rule clap cannot express, exiting non-zero with a
/// message.
fn usage(message: &str) -> crate::Error {
    IOError::new(ErrorKind::InvalidInput, message).into()
}
