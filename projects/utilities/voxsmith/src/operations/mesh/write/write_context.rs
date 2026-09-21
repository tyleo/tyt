use crate::{
    dependencies::mesh::EncodePng,
    operations::mesh::{Atlases, MeshRecord, ProgramRun, Streams},
};
use branded_id::U32Id;
use meshdoc::BMeshFile;
use std::collections::HashMap;

/// The run-wide inputs every writer reads.
#[derive(Clone, Copy)]
pub(crate) struct WriteContext<'a, D: EncodePng> {
    pub dependencies: &'a D,
    pub record: &'a MeshRecord,
    pub run: &'a ProgramRun,
    pub streams: &'a Streams,
    pub atlases: &'a Atlases<'a>,
    pub file_ids: &'a HashMap<String, U32Id<BMeshFile>>,
}
