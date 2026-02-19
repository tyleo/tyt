use crate::Dependencies;
use tyt_cubemap::DependenciesImpl as TytCubemapDependenciesImpl;
use tyt_fbx::DependenciesImpl as TytFbxDependenciesImpl;
use tyt_fs::DependenciesImpl as TytFSDependenciesImpl;
use tyt_image::DependenciesImpl as TytImageDependenciesImpl;
use tyt_material::DependenciesImpl as TytMaterialDependenciesImpl;
use tyt_meta::DependenciesImpl as TytMetaDependenciesImpl;
use tyt_vmax::DependenciesImpl as TytVMaxDependenciesImpl;

#[derive(Clone, Copy, Debug, Default)]
pub struct DependenciesImpl;

impl Dependencies for DependenciesImpl {
    type TytCubemapDependencies = TytCubemapDependenciesImpl;
    type TytFSDependencies = TytFSDependenciesImpl;
    type TytFbxDependencies = TytFbxDependenciesImpl;
    type TytImageDependencies = TytImageDependenciesImpl;
    type TytMaterialDependencies = TytMaterialDependenciesImpl;
    type TytMetaDependencies = TytMetaDependenciesImpl;
    type TytVMaxDependencies = TytVMaxDependenciesImpl;

    fn tyt_cubemap_dependencies(&self) -> Self::TytCubemapDependencies {
        TytCubemapDependenciesImpl
    }

    fn tyt_fbx_dependencies(&self) -> Self::TytFbxDependencies {
        TytFbxDependenciesImpl
    }

    fn tyt_fs_dependencies(&self) -> Self::TytFSDependencies {
        TytFSDependenciesImpl
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

    fn tyt_vmax_dependencies(&self) -> Self::TytVMaxDependencies {
        TytVMaxDependenciesImpl
    }
}
