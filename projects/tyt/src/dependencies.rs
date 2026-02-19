use tyt_cubemap::Dependencies as TytCubemapDependencies;
use tyt_fbx::Dependencies as TytFbxDependencies;
use tyt_fs::Dependencies as TytFSDependencies;
use tyt_image::Dependencies as TytImageDependencies;
use tyt_material::Dependencies as TytMaterialDependencies;
use tyt_meta::Dependencies as TytMetaDependencies;
use tyt_vmax::Dependencies as TytVMaxDependencies;

pub trait Dependencies {
    type TytCubemapDependencies: TytCubemapDependencies;
    type TytFSDependencies: TytFSDependencies;
    type TytFbxDependencies: TytFbxDependencies;
    type TytImageDependencies: TytImageDependencies;
    type TytMaterialDependencies: TytMaterialDependencies;
    type TytMetaDependencies: TytMetaDependencies;
    type TytVMaxDependencies: TytVMaxDependencies;

    fn tyt_cubemap_dependencies(&self) -> Self::TytCubemapDependencies;
    fn tyt_fbx_dependencies(&self) -> Self::TytFbxDependencies;
    fn tyt_fs_dependencies(&self) -> Self::TytFSDependencies;
    fn tyt_image_dependencies(&self) -> Self::TytImageDependencies;
    fn tyt_material_dependencies(&self) -> Self::TytMaterialDependencies;
    fn tyt_meta_dependencies(&self) -> Self::TytMetaDependencies;
    fn tyt_vmax_dependencies(&self) -> Self::TytVMaxDependencies;
}
