use crate::{
    ColorComponent, Error, Format, MeshFormat, Result,
    commands::{
        ChannelSource, MeshMethod, MeshTextureMap, ResourceStorage, TextureBake, TextureShape,
    },
    implementation,
};
use branded_id::U32Id;
use std::{
    fs,
    io::{Error as IOError, ErrorKind},
    path::Path,
};
use voxcore::{BVoxLayer, BVoxPalette, VoxMain, VoxObject, VoxValuePool, VoxValuePoolKind};
use voxsmith::{
    AtlasShape, ColorChannel, GltfAttributeKind, MaterialBake, MaterialChannel, MaterialMap,
    MaterialMeshRequest, MaterialSlot, MeshMethod as VoxsmithMeshMethod,
    ResourceStorage as VoxsmithResourceStorage, object_to_glb_bytes, object_to_gltf_bytes,
    object_to_material_glb, object_to_material_gltf,
};

/// Meshes the object at index `object` of the voxel file at `input` into a mesh
/// at `output`. With no `maps` it writes pure geometry; otherwise it bakes the
/// materials of the object's layer `layer`, a 0-based index into its layers,
/// into textures the mesh samples, writing any loose images beside `output`.
/// The object index is a position into the document's objects, as
/// [`resolve_objects`] returns.
///
/// [`resolve_objects`]: crate::implementation::resolve_objects
#[allow(clippy::too_many_arguments)]
pub fn mesh_object(
    input: &Path,
    from: Option<Format>,
    output: &Path,
    format: MeshFormat,
    scale: f64,
    method: MeshMethod,
    object: usize,
    layer: usize,
    maps: &[MeshTextureMap],
    storage: ResourceStorage,
    texture_shape: TextureShape,
) -> Result<()> {
    let state = implementation::load_state(input, from)?;

    let (_, object) = state.iter_objects().nth(object).ok_or_else(|| {
        IOError::new(
            ErrorKind::InvalidInput,
            format!("object index {object} is out of range"),
        )
    })?;

    let method = mesh_method(method);

    // No maps is the pure-geometry path, which needs neither the palettes nor
    // the atlas machinery.
    if maps.is_empty() {
        let bytes = match format {
            MeshFormat::Gltf => object_to_gltf_bytes(object, method, scale)?,
            MeshFormat::Glb => object_to_glb_bytes(object, method, scale)?,
        };

        fs::write(output, &bytes)?;

        return Ok(());
    }

    // The material path bakes one layer's materials, chosen by `--layer`.
    let layer_id = resolve_layer(object, layer)?;

    // The maps read properties from that layer's palette, so validate each
    // channel's component against the property's kind before baking.
    let palette = object
        .iter_layers()
        .nth(layer)
        .map(|(_, palette)| palette)
        .expect("resolve_layer verified the layer exists");

    validate_maps(&state, palette, layer, maps)?;

    let request = MaterialMeshRequest {
        method,
        layer: layer_id,
        scale,
        maps: maps.iter().map(material_map).collect(),
        storage: resource_storage(storage),
        shape: atlas_shape(texture_shape),
    };

    let files = match format {
        MeshFormat::Gltf => object_to_material_gltf(&state, object, &request)?,
        MeshFormat::Glb => object_to_material_glb(&state, object, &request)?,
    };

    fs::write(output, &files.mesh)?;

    // Loose images go beside the mesh, named as the document references them.
    let directory = output.parent().unwrap_or_else(|| Path::new("."));

    for (name, bytes) in &files.sidecars {
        fs::write(directory.join(name), bytes)?;
    }

    Ok(())
}

/// Resolves the `--layer` ordinal to the object's layer id, a 0-based index
/// into its layers in reference order. Errors when the object has no such
/// layer.
fn resolve_layer(object: &VoxObject, layer: usize) -> Result<U32Id<BVoxLayer>> {
    object
        .iter_layers()
        .nth(layer)
        .map(|(id, _)| id)
        .ok_or_else(|| {
            IOError::new(
                ErrorKind::InvalidInput,
                format!(
                    "object `{}` has no layer {layer}; it has {} layer(s)",
                    object.name(),
                    object.layer_count(),
                ),
            )
            .into()
        })
}

/// The kind a channel reads a property as, fixing whether it may name a color
/// component.
enum ChannelKind {
    Color { alpha: bool },
    Scalar,
}

/// Validates every map's property channels against the meshed layer's palette;
/// `layer` is the `--layer` ordinal for the error text.
fn validate_maps(
    state: &VoxMain,
    palette: U32Id<BVoxPalette>,
    layer: usize,
    maps: &[MeshTextureMap],
) -> Result<()> {
    for map in maps {
        let TextureBake::Packing(packing) = &map.bake else {
            continue;
        };

        for source in packing.sources() {
            let ChannelSource::Property { key, component, .. } = source else {
                continue;
            };

            validate_channel(state, palette, layer, &key, component)?;
        }
    }

    Ok(())
}

/// Validates one property channel against its kind: a color must name a
/// component and `.a` needs alpha, a scalar must name none.
fn validate_channel(
    state: &VoxMain,
    palette: U32Id<BVoxPalette>,
    layer: usize,
    key: &str,
    component: Option<ColorComponent>,
) -> Result<()> {
    match channel_kind(state, palette, layer, key)? {
        ChannelKind::Color { alpha } => match component {
            None => Err(Error::usage(format!(
                "`{key}` is a color; name a component, as `{key}.r`"
            ))),
            Some(ColorComponent::A) if !alpha => Err(Error::usage(format!(
                "`{key}` is a color with no alpha; use r, g, or b"
            ))),
            _ => Ok(()),
        },
        ChannelKind::Scalar => {
            if component.is_some() {
                return Err(Error::usage(format!(
                    "`{key}` is a scalar and has no color component"
                )));
            }
            Ok(())
        }
    }
}

/// The kind of the property `key` in the meshed layer: its bound pool's kind
/// when a property carries it, else the voxj unbound-default rule, a glTF
/// built-in by its spec kind and a custom property an error.
fn channel_kind(
    state: &VoxMain,
    palette: U32Id<BVoxPalette>,
    layer: usize,
    key: &str,
) -> Result<ChannelKind> {
    let palette = state
        .palette(palette)
        .expect("the meshed layer references a palette the state holds");

    let Some(property_id) = palette.property_by_name(key) else {
        // Absent from the palette: the voxj format's unbound-default rule. A
        // glTF built-in takes its spec kind and bakes its default; a custom
        // property has no default, so its type cannot be inferred.
        return match GltfAttributeKind::of(key) {
            Some(GltfAttributeKind::ColorRgba) => Ok(ChannelKind::Color { alpha: true }),
            Some(GltfAttributeKind::ColorRgb) => Ok(ChannelKind::Color { alpha: false }),
            Some(GltfAttributeKind::Scalar) => Ok(ChannelKind::Scalar),
            None => Err(Error::usage(format!(
                "`{key}` is not bound in the meshed layer's palette (layer {layer}), so its type \
                 cannot be inferred; bind it in the palette or map a glTF property"
            ))),
        };
    };

    let pool_id = palette
        .property(property_id)
        .expect("a property id from this palette resolves")
        .pool;
    let pool = state
        .value_pool(pool_id)
        .expect("a property references a pool the state holds");

    pool_kind(pool, key)
}

/// Classifies a bound value pool into a channel kind, rejecting a pool that has
/// no texel value: a string or json pool.
fn pool_kind(pool: &VoxValuePool, key: &str) -> Result<ChannelKind> {
    match pool.kind() {
        VoxValuePoolKind::Srgb { .. } | VoxValuePoolKind::LinearRgb { .. } => {
            Ok(ChannelKind::Color { alpha: false })
        }
        VoxValuePoolKind::Srgba { .. } | VoxValuePoolKind::LinearRgba { .. } => {
            Ok(ChannelKind::Color { alpha: true })
        }
        VoxValuePoolKind::Float { .. }
        | VoxValuePoolKind::Int { .. }
        | VoxValuePoolKind::Bool { .. } => Ok(ChannelKind::Scalar),
        VoxValuePoolKind::String { .. } | VoxValuePoolKind::Json { .. } => Err(Error::usage(
            format!("`{key}` is bound to a string or json pool, which has no texel value"),
        )),
    }
}

/// Maps a CLI meshing method to the voxsmith method.
fn mesh_method(method: MeshMethod) -> VoxsmithMeshMethod {
    match method {
        MeshMethod::Greedy => VoxsmithMeshMethod::Greedy,
        MeshMethod::Culled => VoxsmithMeshMethod::Culled,
        MeshMethod::Naive => VoxsmithMeshMethod::Naive,
    }
}

/// Lowers a resolved map into the voxsmith map. A map with no slot is reached
/// only through the material's extras.
fn material_map(map: &MeshTextureMap) -> MaterialMap {
    MaterialMap {
        name: map.name.clone(),
        slot: map.slot.unwrap_or(MaterialSlot::None),
        bake: material_bake(&map.bake),
    }
}

/// Lowers a resolved bake into the voxsmith bake.
fn material_bake(bake: &TextureBake) -> MaterialBake {
    match bake {
        TextureBake::RgbaColor => MaterialBake::RgbaColor,
        TextureBake::EmissiveColor => MaterialBake::EmissiveColor,
        TextureBake::Packing(packing) => {
            MaterialBake::Packing(packing.sources().iter().map(material_channel).collect())
        }
    }
}

/// Lowers one resolved channel source into the voxsmith channel.
fn material_channel(source: &ChannelSource) -> MaterialChannel {
    match source {
        ChannelSource::Zero => MaterialChannel::Zero,
        ChannelSource::One => MaterialChannel::One,
        ChannelSource::ComputedOcclusion => MaterialChannel::ComputedOcclusion,
        ChannelSource::Property {
            key,
            component,
            invert,
        } => MaterialChannel::Attribute {
            key: key.clone(),
            component: component
                .as_ref()
                .map(|component| color_channel(*component)),
            invert: *invert,
        },
    }
}

/// Maps a CLI color component to the voxsmith color channel.
fn color_channel(component: ColorComponent) -> ColorChannel {
    match component {
        ColorComponent::R => ColorChannel::R,
        ColorComponent::G => ColorChannel::G,
        ColorComponent::B => ColorChannel::B,
        ColorComponent::A => ColorChannel::A,
    }
}

/// Maps a CLI storage mode to the voxsmith storage mode.
fn resource_storage(storage: ResourceStorage) -> VoxsmithResourceStorage {
    match storage {
        ResourceStorage::Embedded => VoxsmithResourceStorage::Embedded,
        ResourceStorage::External => VoxsmithResourceStorage::External,
        ResourceStorage::Both => VoxsmithResourceStorage::Both,
    }
}

/// Maps a CLI texture shape to the voxsmith atlas shape.
fn atlas_shape(shape: TextureShape) -> AtlasShape {
    match shape {
        TextureShape::Line => AtlasShape::Line,
        TextureShape::Fit => AtlasShape::Fit,
        TextureShape::Square => AtlasShape::Square,
        TextureShape::Pot => AtlasShape::Pot,
        TextureShape::Exact(side) => AtlasShape::Exact(side),
    }
}

#[cfg(test)]
mod tests {
    use super::{resolve_layer, validate_channel};
    use crate::ColorComponent;
    use branded_id::U32Id;
    use ty_math::TyVector3U32;
    use voxcore::{
        BVoxPalette, BVoxPoolValue, VoxBound, VoxMain, VoxObject, VoxPalette, VoxValuePool,
    };

    /// The branded value id `index`.
    fn value(index: usize) -> U32Id<BVoxPoolValue> {
        U32Id::from_u32(index as u32)
    }

    /// A standalone object with `layers` layers over one shared palette. The
    /// backing state is dropped; the object owns its layer set.
    fn object_with_layers(layers: usize) -> VoxObject {
        let mut state = VoxMain::default();
        let pool = state.add_value_pool(VoxValuePool::srgba(vec![[1.0, 0.0, 0.0, 1.0]]).unwrap());
        let mut palette = VoxPalette::default();
        palette
            .add_property("baseColorFactor".to_owned(), pool, U32Id::from_u32(0))
            .unwrap();
        let material = palette.add_material(vec![value(0)]).unwrap();
        let palette = state.add_palette(palette).unwrap();

        let mut object = VoxObject::new("body".to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();
        for _ in 0..layers {
            object.add_layer(palette, material);
        }
        object
    }

    #[test]
    fn resolves_each_layer_by_its_ordinal() {
        let object = object_with_layers(2);
        let ids: Vec<_> = object.iter_layers().map(|(id, _)| id).collect();
        assert_eq!(resolve_layer(&object, 0).unwrap(), ids[0]);
        assert_eq!(resolve_layer(&object, 1).unwrap(), ids[1]);
    }

    #[test]
    fn rejects_a_layer_past_the_last() {
        let object = object_with_layers(2);
        assert!(resolve_layer(&object, 2).is_err());
    }

    #[test]
    fn rejects_the_default_layer_on_a_layerless_object() {
        let object = object_with_layers(0);
        assert!(resolve_layer(&object, 0).is_err());
    }

    /// A one-palette document binding `tint` to a four-component color, `glow`
    /// to a three-component color, and `gloss` to a float scalar.
    fn palette_state() -> (VoxMain, U32Id<BVoxPalette>) {
        let mut state = VoxMain::default();

        let tint = state.add_value_pool(VoxValuePool::srgba(vec![[1.0, 0.0, 0.0, 1.0]]).unwrap());
        let glow = state.add_value_pool(VoxValuePool::srgb(vec![[0.0, 1.0, 0.0]]).unwrap());
        let gloss = state.add_value_pool(
            VoxValuePool::float(VoxBound::Number(0.0), VoxBound::Number(1.0), vec![0.5]).unwrap(),
        );

        let mut palette = VoxPalette::default();
        palette
            .add_property("tint".to_owned(), tint, U32Id::from_u32(0))
            .unwrap();
        palette
            .add_property("glow".to_owned(), glow, U32Id::from_u32(0))
            .unwrap();
        palette
            .add_property("gloss".to_owned(), gloss, U32Id::from_u32(0))
            .unwrap();
        palette
            .add_material(vec![value(0), value(0), value(0)])
            .unwrap();

        let palette = state.add_palette(palette).unwrap();

        (state, palette)
    }

    /// Whether `key` with `component` validates against `palette`, meshed as
    /// layer 0.
    fn validates(
        state: &VoxMain,
        palette: U32Id<BVoxPalette>,
        key: &str,
        component: Option<ColorComponent>,
    ) -> bool {
        validate_channel(state, palette, 0, key, component).is_ok()
    }

    #[test]
    fn a_present_color_reads_by_component() {
        let (state, palette) = palette_state();
        // `tint` is an srgba pool: a component is required, and `.a` is allowed.
        assert!(!validates(&state, palette, "tint", None));
        assert!(validates(&state, palette, "tint", Some(ColorComponent::R)));
        assert!(validates(&state, palette, "tint", Some(ColorComponent::A)));
    }

    #[test]
    fn a_present_three_component_color_rejects_alpha() {
        let (state, palette) = palette_state();
        assert!(validates(&state, palette, "glow", Some(ColorComponent::B)));
        assert!(!validates(&state, palette, "glow", Some(ColorComponent::A)));
    }

    #[test]
    fn a_present_scalar_rejects_a_component() {
        let (state, palette) = palette_state();
        assert!(validates(&state, palette, "gloss", None));
        assert!(!validates(
            &state,
            palette,
            "gloss",
            Some(ColorComponent::R)
        ));
    }

    #[test]
    fn an_absent_builtin_takes_its_spec_kind() {
        let (state, palette) = palette_state();
        // None are bound, so each validates by its glTF spec kind and bakes its
        // default: baseColorFactor is a four-component color, occlusionStrength a
        // scalar, emissiveFactor a three-component color.
        assert!(validates(
            &state,
            palette,
            "baseColorFactor",
            Some(ColorComponent::A)
        ));
        assert!(!validates(&state, palette, "baseColorFactor", None));
        assert!(validates(&state, palette, "occlusionStrength", None));
        assert!(!validates(
            &state,
            palette,
            "occlusionStrength",
            Some(ColorComponent::R)
        ));
        assert!(!validates(
            &state,
            palette,
            "emissiveFactor",
            Some(ColorComponent::A)
        ));
    }

    #[test]
    fn an_absent_custom_property_is_an_error() {
        let (state, palette) = palette_state();
        // `subsurface` is neither bound nor a glTF property, so its type cannot
        // be inferred, whether or not a component is named.
        assert!(!validates(&state, palette, "subsurface", None));
        assert!(!validates(
            &state,
            palette,
            "subsurface",
            Some(ColorComponent::R)
        ));
    }

    #[test]
    fn a_string_pool_has_no_texel_value() {
        let mut state = VoxMain::default();
        let tag = state.add_value_pool(VoxValuePool::string(vec!["low".to_owned()]).unwrap());
        let mut palette = VoxPalette::default();
        palette
            .add_property("tag".to_owned(), tag, U32Id::from_u32(0))
            .unwrap();
        palette.add_material(vec![value(0)]).unwrap();
        let palette = state.add_palette(palette).unwrap();

        assert!(!validates(&state, palette, "tag", None));
    }
}
