use tyt_cubemap::Dependencies as TytCubemapDependencies;
use tyt_fbx::Dependencies as TytFbxDependencies;
use tyt_image::Dependencies as TytImageDependencies;
use tyt_material::Dependencies as TytMaterialDependencies;

pub trait Dependencies {
    type TytCubemapDependencies: TytCubemapDependencies;
    type TytFbxDependencies: TytFbxDependencies;
    type TytImageDependencies: TytImageDependencies;
    type TytMaterialDependencies: TytMaterialDependencies;

    fn tyt_cubemap_dependencies(&self) -> Self::TytCubemapDependencies;
    fn tyt_fbx_dependencies(&self) -> Self::TytFbxDependencies;
    fn tyt_image_dependencies(&self) -> Self::TytImageDependencies;
    fn tyt_material_dependencies(&self) -> Self::TytMaterialDependencies;
}
