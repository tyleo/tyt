use ty_fbx::Dependencies as TyFbxDependencies;

pub trait Dependencies {
    type TyFbxDependencies: TyFbxDependencies;

    fn ty_fbx_dependencies(&self) -> Self::TyFbxDependencies;
}
