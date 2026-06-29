use crate::Dependencies;
use tyt_claude::DependenciesImpl as TytClaudeDependenciesImpl;
use tyt_cubemap::DependenciesImpl as TytCubemapDependenciesImpl;
use tyt_fbx::DependenciesImpl as TytFbxDependenciesImpl;
use tyt_fs::DependenciesImpl as TytFSDependenciesImpl;
use tyt_image::DependenciesImpl as TytImageDependenciesImpl;
use tyt_material::DependenciesImpl as TytMaterialDependenciesImpl;
use tyt_meshy::DependenciesImpl as TytMeshyDependenciesImpl;
use tyt_meta::DependenciesImpl as TytMetaDependenciesImpl;
use tyt_oai::DependenciesImpl as TytOAIDependenciesImpl;
use tyt_vmax::DependenciesImpl as TytVMaxDependenciesImpl;
use vxl::DependenciesImpl as VxlDependenciesImpl;

#[derive(Clone, Copy, Debug, Default)]
pub struct DependenciesImpl;

impl Dependencies for DependenciesImpl {
    type TytClaudeDependencies = TytClaudeDependenciesImpl;
    type TytCubemapDependencies = TytCubemapDependenciesImpl;
    type TytFSDependencies = TytFSDependenciesImpl;
    type TytFbxDependencies = TytFbxDependenciesImpl;
    type TytImageDependencies = TytImageDependenciesImpl;
    type TytMaterialDependencies = TytMaterialDependenciesImpl;
    type TytMeshyDependencies = TytMeshyDependenciesImpl;
    type TytMetaDependencies = TytMetaDependenciesImpl;
    type TytOAIDependencies = TytOAIDependenciesImpl;
    type TytVMaxDependencies = TytVMaxDependenciesImpl;
    type VxlDependencies = VxlDependenciesImpl;

    fn tyt_claude_dependencies(&self) -> Self::TytClaudeDependencies {
        TytClaudeDependenciesImpl
    }

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

    fn tyt_meshy_dependencies(&self) -> Self::TytMeshyDependencies {
        TytMeshyDependenciesImpl
    }

    fn tyt_meta_dependencies(&self) -> Self::TytMetaDependencies {
        TytMetaDependenciesImpl
    }

    fn tyt_oai_dependencies(&self) -> Self::TytOAIDependencies {
        TytOAIDependenciesImpl
    }

    fn tyt_vmax_dependencies(&self) -> Self::TytVMaxDependencies {
        TytVMaxDependenciesImpl
    }

    fn vxl_dependencies(&self) -> Self::VxlDependencies {
        VxlDependenciesImpl
    }
}
