use tyt_fbx::Dependencies as TytFbxDependencies;

pub trait Dependencies {
    type TytFbxDependencies: TytFbxDependencies;

    fn tyt_fbx_dependencies(&self) -> Self::TytFbxDependencies;
}
