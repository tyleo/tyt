use crate::ext::{QbclExt, QbclExtMetadata, QbclExtNode, QbclExtThumbnail};
use qbcl::qbcl::{QbclFile, QbclMetadata};
use voxcore::{VoxExt, VoxMain};

/// Puts the ext a read yields on the bare state. [`QbclExt`] keeps it. `()`
/// drops it.
pub trait QbclExtSink: VoxExt + Sized {
    /// Puts on `state` the ext of a read of `file`. `nodes` holds each
    /// hierarchy node's entry in listing order.
    fn record_file(state: VoxMain<()>, file: &QbclFile, nodes: Vec<QbclExtNode>) -> VoxMain<Self>;
}

impl QbclExtSink for QbclExt {
    fn record_file(state: VoxMain<()>, file: &QbclFile, nodes: Vec<QbclExtNode>) -> VoxMain<Self> {
        state.put_ext(QbclExt {
            program_version: file.program_version,
            file_version: file.file_version,
            thumbnail: QbclExtThumbnail {
                width: file.thumbnail.width,
                height: file.thumbnail.height,
                pixels: file
                    .thumbnail
                    .pixels
                    .iter()
                    .map(|pixel| [pixel.r, pixel.g, pixel.b, pixel.a])
                    .collect(),
            },
            metadata: metadata_provenance(&file.metadata),
            guid: file.guid,
            nodes: nodes.into_iter().map(Some).collect(),
        })
    }
}

impl QbclExtSink for () {
    fn record_file(
        state: VoxMain<()>,
        _file: &QbclFile,
        _nodes: Vec<QbclExtNode>,
    ) -> VoxMain<Self> {
        state
    }
}

/// The ext provenance for the metadata strings.
fn metadata_provenance(metadata: &QbclMetadata) -> QbclExtMetadata {
    QbclExtMetadata {
        title: metadata.title.clone(),
        description: metadata.description.clone(),
        tags: metadata.tags.clone(),
        author: metadata.author.clone(),
        company: metadata.company.clone(),
        website: metadata.website.clone(),
        copyright: metadata.copyright.clone(),
    }
}
