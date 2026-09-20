use crate::{
    Error, Result,
    dependencies::mesh::{PngChannels, PngImage},
    operations::mesh::{ArrayDomain, Atlases, MeshElement, Transfer, encode_components},
};
use ty_math::TyFloatExt;
use vox_value_language::{Components, Domain, Value};

/// Bakes `value` onto the `bake` atlas under `transfer`, a cell per entry
/// with a lower value climbing into each face. Unused cells stay transparent
/// black.
pub(crate) fn bake_png(
    element: &MeshElement,
    value: &Value,
    transfer: Transfer,
    bake: ArrayDomain,
    atlases: &Atlases<'_>,
) -> Result<PngImage> {
    let Components::F32(components) = value.components() else {
        return Err(Error::mesh_record(
            element.clone(),
            format!("is a {}, and a png takes f32 components", value.to_type()),
        ));
    };

    let width = value.dimension().width();

    let channels = match width {
        1 => PngChannels::Grey,
        2 => PngChannels::GreyAlpha,
        3 => PngChannels::Rgb,
        _ => PngChannels::Rgba,
    };

    let layout = atlases.layout(bake)?;
    let canvas_width = layout.width() as usize;
    let mut samples = vec![0u8; canvas_width * layout.height() as usize * width];

    let entry_of = |entry: usize| &components[entry * width..(entry + 1) * width];

    let corners = if bake == ArrayDomain::Corner { 4 } else { 1 };

    for cell in 0..atlases.cell_count(bake) {
        for corner in 0..corners {
            let entry = if value.domain() == Domain::Corner {
                cell * 4 + corner
            } else {
                let mut entries = atlases.cell_entries(bake, value.domain(), cell).into_iter();

                let entry = entries.next().expect("a cell reads an entry");

                assert!(
                    entries.all(|other| entry_of(other) == entry_of(entry)),
                    "a cell's pieces agree on a baked value"
                );

                entry
            };

            let encoded = encode_components(element, entry_of(entry), transfer, true)?;

            let [x, y] = layout.texel(cell, corner);
            let start = (y as usize * canvas_width + x as usize) * width;

            for (sample, component) in samples[start..start + width].iter_mut().zip(encoded) {
                *sample = component.to_unorm8();
            }
        }
    }

    Ok(PngImage {
        width: layout.width(),
        height: layout.height(),
        channels,
        transfer,
        samples,
    })
}
