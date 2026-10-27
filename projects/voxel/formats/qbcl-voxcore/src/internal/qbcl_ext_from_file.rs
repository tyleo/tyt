use crate::{QbclExt, QbclExtMetadata, QbclExtNode, QbclExtThumbnail};
use branded_id::U32Id;
use qbcl::qbcl::{QbclFile, QbclMetadata};
use std::collections::BTreeMap;
use voxcore::BVoxHierarchyNode;

/// The ext of `file`'s header around `nodes`, one entry per hierarchy node.
pub fn qbcl_ext_from_file(
    file: &QbclFile,
    nodes: BTreeMap<U32Id<BVoxHierarchyNode>, QbclExtNode>,
) -> QbclExt {
    QbclExt {
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
        nodes,
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
