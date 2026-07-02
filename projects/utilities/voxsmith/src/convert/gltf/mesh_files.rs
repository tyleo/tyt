/// A written mesh and the loose resource files that go beside it.
pub struct MeshFiles {
    /// The `.gltf` or `.glb` bytes, referencing its embedded or external
    /// resources.
    pub mesh: Vec<u8>,

    /// The loose files to write beside the mesh, each a relative name and its
    /// bytes. Empty when every resource is embedded.
    pub sidecars: Vec<(String, Vec<u8>)>,
}
