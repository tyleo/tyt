use crate::CliValue;
use voxconv::ReadFormat;

impl CliValue for ReadFormat {
    const VARIANTS: &'static [Self] = &[
        ReadFormat::Goxl,
        ReadFormat::MVox,
        ReadFormat::Qb,
        ReadFormat::Qbt,
        ReadFormat::Qbcl,
        ReadFormat::VMax,
        ReadFormat::Voxj,
    ];

    fn name(self) -> &'static str {
        ReadFormat::name(self)
    }

    fn help(self) -> &'static str {
        match self {
            ReadFormat::Goxl => "Goxel, the `.gox` file",
            ReadFormat::MVox => "MagicaVoxel, the `.vox` file",
            ReadFormat::Qb => "Qubicle Binary, the `.qb` file",
            ReadFormat::Qbt => "Qubicle Binary Tree, the `.qbt` file",
            ReadFormat::Qbcl => "Qubicle Construction Library, the `.qbcl` file",
            ReadFormat::VMax => "Voxel Max, the `.vmax` package directory",
            ReadFormat::Voxj => "Voxel Json, the `.voxj` and `.voxjz` documents",
        }
    }
}
