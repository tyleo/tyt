use crate::{
    ColorComponent, Error, Format, MeshFormat, Result,
    commands::{
        ChannelSource, MeshMethod, MeshTextureMap, ResourceStorage, TextureBake, TextureShape,
    },
    implementation,
};
use std::{
    fs,
    io::{Error as IOError, ErrorKind},
    path::Path,
};
use voxcore::{VoxEffectivePalette, VoxMain, VoxObject, VoxValuePool, VoxValuePoolKind};
use voxsmith::{
    AtlasShape, ColorChannel, GltfAttributeKind, MaterialBake, MaterialChannel, MaterialMap,
    MaterialMeshRequest, MaterialSlot, MeshMethod as VoxsmithMeshMethod,
    ResourceStorage as VoxsmithResourceStorage, object_to_glb_bytes, object_to_gltf_bytes,
    object_to_material_glb, object_to_material_gltf,
};

/// Meshes the object at index `object_index` of the voxel file at `input` into a
/// mesh at `output`. With no `maps` it writes pure geometry. Otherwise it bakes the
/// object's flattened layer materials, merged per property name by the
/// format's layer-override rule, into textures the mesh samples, writing any
/// loose images beside `output`. The object index is a position into the
/// document's objects, as [`resolve_objects`] returns.
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
    object_index: usize,
    maps: &[MeshTextureMap],
    storage: ResourceStorage,
    texture_shape: TextureShape,
) -> Result<()> {
    let state = implementation::load_state(input, from)?;

    let (_, object) = state.iter_objects().nth(object_index).ok_or_else(|| {
        IOError::new(
            ErrorKind::InvalidInput,
            format!("object index {object_index} is out of range"),
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

    // The maps read each property through its winning layer, so validate each
    // channel's component against the winning pool's kind before baking.
    validate_maps(&state, object, maps)?;

    let request = MaterialMeshRequest {
        method,
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

/// The kind a channel reads a property as, fixing whether it may name a color
/// component.
enum ChannelKind {
    Color { alpha: bool },
    Scalar,
}

/// Validates every map's property channels against the object's effective
/// palette, each key's kind read through its winning layer.
fn validate_maps(state: &VoxMain, object: &VoxObject, maps: &[MeshTextureMap]) -> Result<()> {
    let effective = state
        .effective_palette(object)
        .map_err(|error| IOError::new(ErrorKind::InvalidInput, error))?;

    for map in maps {
        let TextureBake::Packing(packing) = &map.bake else {
            continue;
        };

        for source in packing.sources() {
            let ChannelSource::Property { key, component, .. } = source else {
                continue;
            };

            validate_channel(&effective, &key, component)?;
        }
    }

    Ok(())
}

/// Validates one property channel against its kind: a color must name a
/// component and `.a` needs alpha, a scalar must name none.
fn validate_channel(
    effective: &VoxEffectivePalette,
    key: &str,
    component: Option<ColorComponent>,
) -> Result<()> {
    match channel_kind(effective, key)? {
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

/// The kind of the property `key`: its winning layer's pool kind when a layer
/// supplies it, else the voxj unbound-default rule, a glTF built-in by its
/// spec kind and a custom property an error.
fn channel_kind(effective: &VoxEffectivePalette, key: &str) -> Result<ChannelKind> {
    let Some(property_id) = effective.property_id_by_name(key) else {
        // No layer supplies it: the voxj format's unbound-default rule. A
        // glTF built-in takes its spec kind and bakes its default. A custom
        // property has no default, so its type cannot be inferred.
        return match GltfAttributeKind::of(key) {
            Some(GltfAttributeKind::ColorRgba) => Ok(ChannelKind::Color { alpha: true }),
            Some(GltfAttributeKind::ColorRgb) => Ok(ChannelKind::Color { alpha: false }),
            Some(GltfAttributeKind::Scalar) => Ok(ChannelKind::Scalar),
            None => Err(Error::usage(format!(
                "`{key}` is not bound by any of the object's layers, so its type cannot be \
                 inferred; bind it in a palette or map a glTF property"
            ))),
        };
    };

    let value_pool = effective
        .property(property_id)
        .expect("a resolved name identifies one of the effective palette's properties")
        .value_pool();

    value_pool_kind(value_pool, key)
}

/// Classifies a bound value pool into a channel kind, rejecting a pool that has
/// no texel value: a string or json pool.
fn value_pool_kind(value_pool: &VoxValuePool, key: &str) -> Result<ChannelKind> {
    match value_pool.kind() {
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
    use super::validate_channel;
    use crate::ColorComponent;
    use branded_id::U32Id;
    use ty_math::TyVector3U32;
    use voxcore::{
        BVoxPalette, BVoxValuePoolValue, VoxBound, VoxMain, VoxObject, VoxPalette, VoxValuePool,
    };

    /// The branded value id `index`.
    fn value_id(index: usize) -> U32Id<BVoxValuePoolValue> {
        U32Id::from_u32(index as u32)
    }

    /// A one-palette document binding `tint` to a four-component color, `glow`
    /// to a three-component color, and `gloss` to a float scalar.
    fn palette_state() -> (VoxMain, U32Id<BVoxPalette>) {
        let mut state = VoxMain::default();

        let tint_value_pool_id =
            state.add_value_pool(VoxValuePool::srgba(vec![[1.0, 0.0, 0.0, 1.0]]).unwrap());
        let glow_value_pool_id =
            state.add_value_pool(VoxValuePool::srgb(vec![[0.0, 1.0, 0.0]]).unwrap());
        let gloss_value_pool_id = state.add_value_pool(
            VoxValuePool::float(VoxBound::Number(0.0), VoxBound::Number(1.0), vec![0.5]).unwrap(),
        );

        let mut palette = VoxPalette::default();
        palette
            .add_property("tint".to_owned(), tint_value_pool_id, U32Id::from_u32(0))
            .unwrap();
        palette
            .add_property("glow".to_owned(), glow_value_pool_id, U32Id::from_u32(0))
            .unwrap();
        palette
            .add_property("gloss".to_owned(), gloss_value_pool_id, U32Id::from_u32(0))
            .unwrap();
        palette
            .add_material(vec![value_id(0), value_id(0), value_id(0)])
            .unwrap();

        let palette_id = state.add_palette(palette).unwrap();

        (state, palette_id)
    }

    /// Whether `key` with `component` validates against an object layering the
    /// given palettes in order.
    fn validates(
        state: &VoxMain,
        palette_ids: &[U32Id<BVoxPalette>],
        key: &str,
        component: Option<ColorComponent>,
    ) -> bool {
        let mut object = VoxObject::new("body".to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();
        for &palette_id in palette_ids {
            object.add_layer(palette_id, U32Id::from_u32(0));
        }
        let effective = state.effective_palette(&object).unwrap();
        validate_channel(&effective, key, component).is_ok()
    }

    #[test]
    fn a_present_color_reads_by_component() {
        let (state, palette_id) = palette_state();
        // `tint` is an srgba pool: a component is required, and `.a` is allowed.
        assert!(!validates(&state, &[palette_id], "tint", None));
        assert!(validates(
            &state,
            &[palette_id],
            "tint",
            Some(ColorComponent::R)
        ));
        assert!(validates(
            &state,
            &[palette_id],
            "tint",
            Some(ColorComponent::A)
        ));
    }

    #[test]
    fn a_present_three_component_color_rejects_alpha() {
        let (state, palette_id) = palette_state();
        assert!(validates(
            &state,
            &[palette_id],
            "glow",
            Some(ColorComponent::B)
        ));
        assert!(!validates(
            &state,
            &[palette_id],
            "glow",
            Some(ColorComponent::A)
        ));
    }

    #[test]
    fn a_present_scalar_rejects_a_component() {
        let (state, palette_id) = palette_state();
        assert!(validates(&state, &[palette_id], "gloss", None));
        assert!(!validates(
            &state,
            &[palette_id],
            "gloss",
            Some(ColorComponent::R)
        ));
    }

    #[test]
    fn an_absent_builtin_takes_its_spec_kind() {
        let (state, palette_id) = palette_state();
        // None are bound, so each validates by its glTF spec kind and bakes its
        // default: baseColorFactor is a four-component color, occlusionStrength a
        // scalar, emissiveFactor a three-component color.
        assert!(validates(
            &state,
            &[palette_id],
            "baseColorFactor",
            Some(ColorComponent::A)
        ));
        assert!(!validates(&state, &[palette_id], "baseColorFactor", None));
        assert!(validates(&state, &[palette_id], "occlusionStrength", None));
        assert!(!validates(
            &state,
            &[palette_id],
            "occlusionStrength",
            Some(ColorComponent::R)
        ));
        assert!(!validates(
            &state,
            &[palette_id],
            "emissiveFactor",
            Some(ColorComponent::A)
        ));
    }

    #[test]
    fn an_absent_custom_property_is_an_error() {
        let (state, palette_id) = palette_state();
        // `subsurface` is neither bound nor a glTF property, so its type cannot
        // be inferred, whether or not a component is named.
        assert!(!validates(&state, &[palette_id], "subsurface", None));
        assert!(!validates(
            &state,
            &[palette_id],
            "subsurface",
            Some(ColorComponent::R)
        ));
    }

    #[test]
    fn a_string_value_pool_has_no_texel_value() {
        let mut state = VoxMain::default();
        let tag_value_pool_id =
            state.add_value_pool(VoxValuePool::string(vec!["low".to_owned()]).unwrap());
        let mut palette = VoxPalette::default();
        palette
            .add_property("tag".to_owned(), tag_value_pool_id, U32Id::from_u32(0))
            .unwrap();
        palette.add_material(vec![value_id(0)]).unwrap();
        let palette_id = state.add_palette(palette).unwrap();

        assert!(!validates(&state, &[palette_id], "tag", None));
    }

    #[test]
    fn a_key_takes_its_winning_layers_kind() {
        // Two palettes bind `finish` to different kinds: a float scalar and a
        // four-component color. The last layer's palette wins, so the layer
        // order flips which component rule applies.
        let mut state = VoxMain::default();
        let scalar_value_pool_id = state.add_value_pool(
            VoxValuePool::float(VoxBound::Number(0.0), VoxBound::Number(1.0), vec![0.5]).unwrap(),
        );
        let color_value_pool_id =
            state.add_value_pool(VoxValuePool::srgba(vec![[1.0, 0.0, 0.0, 1.0]]).unwrap());

        let mut scalar_palette = VoxPalette::default();
        scalar_palette
            .add_property(
                "finish".to_owned(),
                scalar_value_pool_id,
                U32Id::from_u32(0),
            )
            .unwrap();
        scalar_palette.add_material(vec![value_id(0)]).unwrap();
        let scalar_palette_id = state.add_palette(scalar_palette).unwrap();

        let mut color_palette = VoxPalette::default();
        color_palette
            .add_property("finish".to_owned(), color_value_pool_id, U32Id::from_u32(0))
            .unwrap();
        color_palette.add_material(vec![value_id(0)]).unwrap();
        let color_palette_id = state.add_palette(color_palette).unwrap();

        // Color wins: a component is required and `.a` allowed.
        assert!(!validates(
            &state,
            &[scalar_palette_id, color_palette_id],
            "finish",
            None
        ));
        assert!(validates(
            &state,
            &[scalar_palette_id, color_palette_id],
            "finish",
            Some(ColorComponent::A)
        ));

        // Scalar wins: no component allowed.
        assert!(validates(
            &state,
            &[color_palette_id, scalar_palette_id],
            "finish",
            None
        ));
        assert!(!validates(
            &state,
            &[color_palette_id, scalar_palette_id],
            "finish",
            Some(ColorComponent::R)
        ));
    }
}
