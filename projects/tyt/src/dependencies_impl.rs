use crate::Dependencies;
use tyt_cubemap::DependenciesImpl as TytCubemapDependenciesImpl;
use tyt_fbx::DependenciesImpl as TytFbxDependenciesImpl;
use tyt_image::DependenciesImpl as TytImageDependenciesImpl;
use tyt_material::DependenciesImpl as TytMaterialDependenciesImpl;
use tyt_meta::DependenciesImpl as TytMetaDependenciesImpl;

#[derive(Clone, Copy, Debug, Default)]
pub struct DependenciesImpl;

impl Dependencies for DependenciesImpl {
    type TytCubemapDependencies = TytCubemapDependenciesImpl;
    type TytFbxDependencies = TytFbxDependenciesImpl;
    type TytImageDependencies = TytImageDependenciesImpl;
    type TytMaterialDependencies = TytMaterialDependenciesImpl;
    type TytMetaDependencies = TytMetaDependenciesImpl;

    fn tyt_cubemap_dependencies(&self) -> Self::TytCubemapDependencies {
        TytCubemapDependenciesImpl
    }

    fn tyt_fbx_dependencies(&self) -> Self::TytFbxDependencies {
        TytFbxDependenciesImpl
    }

    fn tyt_image_dependencies(&self) -> Self::TytImageDependencies {
        TytImageDependenciesImpl
    }

    fn tyt_material_dependencies(&self) -> Self::TytMaterialDependencies {
        TytMaterialDependenciesImpl
    }

    fn tyt_meta_dependencies(&self) -> Self::TytMetaDependencies {
        TytMetaDependenciesImpl
    }
}
