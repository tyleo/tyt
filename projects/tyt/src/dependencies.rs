use tyt_fbx::Dependencies as TytFbxDependencies;
use tyt_material::Dependencies as TytMaterialDependencies;

pub trait Dependencies {
    type TytFbxDependencies: TytFbxDependencies;
    type TytMaterialDependencies: TytMaterialDependencies;

    fn tyt_fbx_dependencies(&self) -> Self::TytFbxDependencies;
    fn tyt_material_dependencies(&self) -> Self::TytMaterialDependencies;
}
