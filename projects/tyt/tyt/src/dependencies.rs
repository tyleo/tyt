use tyt_claude::Dependencies as TytClaudeDependencies;
use tyt_cubemap::Dependencies as TytCubemapDependencies;
use tyt_fbx::Dependencies as TytFbxDependencies;
use tyt_fs::Dependencies as TytFSDependencies;
use tyt_image::Dependencies as TytImageDependencies;
use tyt_material::Dependencies as TytMaterialDependencies;
use tyt_meshy::Dependencies as TytMeshyDependencies;
use tyt_meta::Dependencies as TytMetaDependencies;
use tyt_oai::Dependencies as TytOAIDependencies;
use tyt_vmax::Dependencies as TytVMaxDependencies;
use voxl::Dependencies as VoxlDependencies;

pub trait Dependencies {
    type TytClaudeDependencies: TytClaudeDependencies;
    type TytCubemapDependencies: TytCubemapDependencies;
    type TytFSDependencies: TytFSDependencies;
    type TytFbxDependencies: TytFbxDependencies;
    type TytImageDependencies: TytImageDependencies;
    type TytMaterialDependencies: TytMaterialDependencies;
    type TytMeshyDependencies: TytMeshyDependencies;
    type TytMetaDependencies: TytMetaDependencies;
    type TytOAIDependencies: TytOAIDependencies;
    type TytVMaxDependencies: TytVMaxDependencies;
    type VoxlDependencies: VoxlDependencies;

    fn tyt_claude_dependencies(&self) -> Self::TytClaudeDependencies;
    fn tyt_cubemap_dependencies(&self) -> Self::TytCubemapDependencies;
    fn tyt_fbx_dependencies(&self) -> Self::TytFbxDependencies;
    fn tyt_fs_dependencies(&self) -> Self::TytFSDependencies;
    fn tyt_image_dependencies(&self) -> Self::TytImageDependencies;
    fn tyt_material_dependencies(&self) -> Self::TytMaterialDependencies;
    fn tyt_meshy_dependencies(&self) -> Self::TytMeshyDependencies;
    fn tyt_meta_dependencies(&self) -> Self::TytMetaDependencies;
    fn tyt_oai_dependencies(&self) -> Self::TytOAIDependencies;
    fn tyt_vmax_dependencies(&self) -> Self::TytVMaxDependencies;
    fn voxl_dependencies(&self) -> Self::VoxlDependencies;
}
