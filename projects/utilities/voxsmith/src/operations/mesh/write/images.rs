use crate::{
    Error, Result,
    dependencies::mesh::EncodePng,
    operations::mesh::{ArrayDomain, Atlases, CheckedDestination, MeshElement, Transfer, bake_png},
};
use branded_id::U32Id;
use meshdoc::{
    BMeshFile, BMeshTexture, MeshImage, MeshImageMediaType, MeshImageSource, MeshMagFilter,
    MeshMain, MeshMinFilter, MeshTexture, MeshWrap,
};
use std::collections::HashMap;
use vox_value_language::{EvaluatedProgram, eval_expression};

/// The document's textures, one per embedded value and one per referenced
/// file. Two slots naming one value share its image.
#[derive(Default)]
pub(crate) struct Images {
    embedded: HashMap<(String, ArrayDomain), (Transfer, MeshElement, U32Id<BMeshTexture>)>,
    files: HashMap<String, U32Id<BMeshTexture>>,
}

impl Images {
    /// The texture over `checked`'s value baked on the `bake` atlas under
    /// `transfer`, embedded into `document` on its first use. Errors if the
    /// value already embeds under the other transfer.
    #[expect(clippy::too_many_arguments, reason = "the bake reads the whole run")]
    pub(crate) fn embed<D: EncodePng>(
        &mut self,
        dependencies: &D,
        document: &mut MeshMain<()>,
        element: &MeshElement,
        checked: &CheckedDestination,
        evaluated: &EvaluatedProgram,
        transfer: Transfer,
        bake: ArrayDomain,
        atlases: &Atlases<'_>,
    ) -> Result<U32Id<BMeshTexture>> {
        let text = checked.destination.text.clone();

        if let Some((embedded, by, texture_id)) = self.embedded.get(&(text.clone(), bake)) {
            if *embedded != transfer {
                return Err(Error::mesh_record(
                    element.clone(),
                    format!("embeds `{text}` as {transfer}, where {by} embeds it as {embedded}"),
                ));
            }

            return Ok(*texture_id);
        }

        let value = eval_expression(&checked.expression, evaluated)
            .map_err(|error| Error::mesh_record(element.clone(), error))?;

        let image = bake_png(element, &value, transfer, bake, atlases)?;

        let png = dependencies.encode_png(&image).map_err(Error::Png)?;

        let texture_id = retain_texture(document, text.clone(), MeshImageSource::Bytes(png), bake)?;

        self.embedded
            .insert((text, bake), (transfer, element.clone(), texture_id));

        Ok(texture_id)
    }

    /// The texture over the written file `file`, baked on the `bake` atlas,
    /// an image over the file retained into `document` on its first use.
    pub(crate) fn reference(
        &mut self,
        document: &mut MeshMain<()>,
        file: &str,
        file_id: U32Id<BMeshFile>,
        bake: ArrayDomain,
    ) -> Result<U32Id<BMeshTexture>> {
        if let Some(texture_id) = self.files.get(file) {
            return Ok(*texture_id);
        }

        let texture_id = retain_texture(
            document,
            file.to_owned(),
            MeshImageSource::File(file_id),
            bake,
        )?;

        self.files.insert(file.to_owned(), texture_id);

        Ok(texture_id)
    }
}

/// A png image named `name` over `source`, drawn through a clamped texture
/// filtered for the `bake` atlas.
fn retain_texture(
    document: &mut MeshMain<()>,
    name: String,
    source: MeshImageSource,
    bake: ArrayDomain,
) -> Result<U32Id<BMeshTexture>> {
    let image_id = document.retain_image(MeshImage {
        name,
        media_type: MeshImageMediaType::Png,
        source,
    })?;

    let (mag_filter, min_filter) = filters(bake);

    Ok(document.retain_texture(MeshTexture {
        image_id,
        mag_filter: Some(mag_filter),
        min_filter: Some(min_filter),
        wrap_s: MeshWrap::ClampToEdge,
        wrap_t: MeshWrap::ClampToEdge,
    })?)
}

/// The filters over the `bake` atlas. The corner atlas blends its 2x2 block
/// across the face, and every other atlas reads each face's texel exactly.
fn filters(bake: ArrayDomain) -> (MeshMagFilter, MeshMinFilter) {
    match bake {
        ArrayDomain::Corner => (MeshMagFilter::Linear, MeshMinFilter::Linear),
        ArrayDomain::Face | ArrayDomain::Swatch | ArrayDomain::Voxel => {
            (MeshMagFilter::Nearest, MeshMinFilter::Nearest)
        }
    }
}
