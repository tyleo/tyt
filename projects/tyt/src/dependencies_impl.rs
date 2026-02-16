use crate::Dependencies;
use tyt_fbx::DependenciesImpl as TytFbxDependenciesImpl;
use tyt_material::DependenciesImpl as TytMaterialDependenciesImpl;

#[derive(Clone, Copy, Debug, Default)]
pub struct DependenciesImpl;

impl Dependencies for DependenciesImpl {
    type TytFbxDependencies = TytFbxDependenciesImpl;
    type TytMaterialDependencies = TytMaterialDependenciesImpl;

    fn tyt_fbx_dependencies(&self) -> Self::TytFbxDependencies {
        TytFbxDependenciesImpl
    }

    fn tyt_material_dependencies(&self) -> Self::TytMaterialDependencies {
        TytMaterialDependenciesImpl
    }
}
