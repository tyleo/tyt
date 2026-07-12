use clap::ValueEnum;

/// Where a mesh's baked resources go: embedded in the mesh, in loose files
/// beside it, or both. Backs `--texture-storage`; the default follows the target
/// (embedded for `.glb`, external for `.gltf`) and is resolved by the command.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum ResourceStorage {
    /// Packed into the mesh: a GLB binary chunk or a `.gltf` data URI.
    #[value(name = "embedded")]
    Embedded,

    /// Written as loose files beside the mesh, which references them.
    #[value(name = "external")]
    External,

    /// Both: the mesh references its embedded copy and the loose files are
    /// working copies.
    #[value(name = "both")]
    Both,
}
