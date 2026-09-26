use crate::CliValue;
use voxsmith::operations::voxelize::ResolutionReference;

impl CliValue for ResolutionReference {
    const VARIANTS: &'static [Self] = &[
        ResolutionReference::LongestWorld,
        ResolutionReference::ShortestWorld,
        ResolutionReference::WorldX,
        ResolutionReference::WorldY,
        ResolutionReference::WorldZ,
        ResolutionReference::LongestObject,
        ResolutionReference::ShortestObject,
        ResolutionReference::LongestObjectX,
        ResolutionReference::LongestObjectY,
        ResolutionReference::LongestObjectZ,
        ResolutionReference::ShortestObjectX,
        ResolutionReference::ShortestObjectY,
        ResolutionReference::ShortestObjectZ,
    ];

    fn name(self) -> &'static str {
        match self {
            ResolutionReference::LongestWorld => "longest-world",
            ResolutionReference::ShortestWorld => "shortest-world",
            ResolutionReference::WorldX => "world-x",
            ResolutionReference::WorldY => "world-y",
            ResolutionReference::WorldZ => "world-z",
            ResolutionReference::LongestObject => "longest-object",
            ResolutionReference::ShortestObject => "shortest-object",
            ResolutionReference::LongestObjectX => "longest-object-x",
            ResolutionReference::LongestObjectY => "longest-object-y",
            ResolutionReference::LongestObjectZ => "longest-object-z",
            ResolutionReference::ShortestObjectX => "shortest-object-x",
            ResolutionReference::ShortestObjectY => "shortest-object-y",
            ResolutionReference::ShortestObjectZ => "shortest-object-z",
        }
    }

    fn help(self) -> &'static str {
        match self {
            ResolutionReference::LongestWorld => "The longest side of the world bounds",
            ResolutionReference::ShortestWorld => "The shortest side of the world bounds",
            ResolutionReference::WorldX => "The world bounds' extent along x",
            ResolutionReference::WorldY => "The world bounds' extent along y",
            ResolutionReference::WorldZ => "The world bounds' extent along z",
            ResolutionReference::LongestObject => "The longest side of any object",
            ResolutionReference::ShortestObject => "The shortest side of any object",
            ResolutionReference::LongestObjectX => "The longest extent along x of any object",
            ResolutionReference::LongestObjectY => "The longest extent along y of any object",
            ResolutionReference::LongestObjectZ => "The longest extent along z of any object",
            ResolutionReference::ShortestObjectX => "The shortest extent along x of any object",
            ResolutionReference::ShortestObjectY => "The shortest extent along y of any object",
            ResolutionReference::ShortestObjectZ => "The shortest extent along z of any object",
        }
    }
}
