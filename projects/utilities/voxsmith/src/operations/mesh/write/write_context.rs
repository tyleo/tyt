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
    pub(crate) dependencies: &'a D,
    pub(crate) record: &'a MeshRecord,
    pub(crate) run: &'a ProgramRun,
    pub(crate) streams: &'a Streams,
    pub(crate) atlases: &'a Atlases<'a>,
    pub(crate) file_ids: &'a HashMap<String, U32Id<BMeshFile>>,
}
