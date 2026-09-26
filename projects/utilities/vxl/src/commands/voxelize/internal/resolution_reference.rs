use crate::CliValue;
use voxsmith::operations::voxelize::ResolutionReference;

impl CliValue for ResolutionReference {
    const VARIANTS: &'static [Self] = &[
        ResolutionReference::LongestWorld,
        ResolutionReference::ShortestWorld,
        ResolutionReference::WorldX,
        ResolutionReference::WorldY,
        ResolutionReference::WorldZ,
    ];

    fn name(self) -> &'static str {
        match self {
            ResolutionReference::LongestWorld => "longest-world",
            ResolutionReference::ShortestWorld => "shortest-world",
            ResolutionReference::WorldX => "world-x",
            ResolutionReference::WorldY => "world-y",
            ResolutionReference::WorldZ => "world-z",
        }
    }

    fn help(self) -> &'static str {
        match self {
            ResolutionReference::LongestWorld => "The longest side of the mesh's world bounds",
            ResolutionReference::ShortestWorld => {
                "The shortest side of the mesh's world bounds with any extent"
            }
            ResolutionReference::WorldX => "The mesh's world extent along x",
            ResolutionReference::WorldY => "The mesh's world extent along y",
            ResolutionReference::WorldZ => "The mesh's world extent along z",
        }
    }
}
