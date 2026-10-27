use crate::{
    Error, Result,
    utilities::{IdSelector, QuantizeOptions, apply_quantize_plan, choose_quantize_plan},
};
use branded_id::{RangeInclusiveExt, U32Id};
use voxcore::{BVoxLayer, BVoxObject, BVoxPalette, VoxExt, VoxMain};

/// Quantizes the layers `layer_indices` picks on each of `object_ids` so each
/// samples at most `options.max_materials` materials of its palette, leaving
/// palettes and value pools untouched. Without `layer_indices`, every layer
/// whose palette binds `options.property` is picked.
pub fn quantize_object_voxels<T: VoxExt>(
    main: &mut VoxMain<T>,
    object_ids: &[U32Id<BVoxObject>],
    layer_indices: &[IdSelector<BVoxLayer>],
    shared: bool,
    options: &QuantizeOptions,
) -> Result<()> {
    let name = &options.property;

    // Each object's picked layers, grouped by palette in order of first
    // appearance.
    let mut object_groups = Vec::new();
    for &object_id in object_ids {
        let object = main
            .object(object_id)
            .expect("a selected object is one of the main's");

        let mut picked = Vec::new();
        if layer_indices.is_empty() {
            for (layer_id, palette_id) in object.iter_layers() {
                if binds_property(main, palette_id, name) {
                    picked.push((layer_id, palette_id));
                }
            }
        } else {
            for selector in layer_indices {
                let Some(range) = selector.as_range() else {
                    continue;
                };

                for layer_id in range.clone().into_id_range() {
                    if object.layer_palette_id(layer_id).is_none() {
                        return Err(Error::invalid(format!(
                            "object {object_id} has no layer {layer_id}"
                        )));
                    }
                }
            }

            for (layer_id, palette_id) in object.iter_layers() {
                if !layer_indices
                    .iter()
                    .any(|selector| selector.contains(layer_id))
                {
                    continue;
                }

                if !binds_property(main, palette_id, name) {
                    return Err(Error::invalid(format!(
                        "layer {layer_id} of object {object_id} references palette {palette_id}, \
                         which has no property `{name}`"
                    )));
                }

                picked.push((layer_id, palette_id));
            }
        }

        let mut groups: Vec<LayerGroup> = Vec::new();
        for (layer_id, palette_id) in picked {
            group_layer(&mut groups, palette_id, (object_id, layer_id));
        }
        object_groups.push(groups);
    }

    if object_groups.iter().all(|groups| groups.is_empty()) {
        return Err(Error::invalid(format!(
            "no selected layer references a palette with property `{name}`"
        )));
    }

    let groups = if shared {
        let mut shared_groups = Vec::new();
        for (palette_id, layers) in object_groups.into_iter().flatten() {
            for layer in layers {
                group_layer(&mut shared_groups, palette_id, layer);
            }
        }
        shared_groups
    } else {
        object_groups.into_iter().flatten().collect()
    };

    for (palette_id, layers) in groups {
        if let Some(plan) = choose_quantize_plan(main, palette_id, &layers, options)? {
            apply_quantize_plan(main, &plan, &layers, options.dither)?;
        }
    }

    Ok(())
}

/// A palette and the picked layers on it, which cluster together.
type LayerGroup = (
    U32Id<BVoxPalette>,
    Vec<(U32Id<BVoxObject>, U32Id<BVoxLayer>)>,
);

/// Adds `layer` to `palette_id`'s group.
fn group_layer(
    groups: &mut Vec<LayerGroup>,
    palette_id: U32Id<BVoxPalette>,
    layer: (U32Id<BVoxObject>, U32Id<BVoxLayer>),
) {
    match groups
        .iter_mut()
        .find(|(group_palette_id, _)| *group_palette_id == palette_id)
    {
        Some((_, layers)) => layers.push(layer),
        None => groups.push((palette_id, vec![layer])),
    }
}

/// Whether `palette_id` binds the property `name`.
fn binds_property<T: VoxExt>(
    main: &VoxMain<T>,
    palette_id: U32Id<BVoxPalette>,
    name: &str,
) -> bool {
    main.palette(palette_id)
        .expect("a layer references a live palette")
        .property_id_by_name(name)
        .is_some()
}

#[cfg(test)]
mod tests {
    use crate::{
        operations::object::quantize_object_voxels,
        utilities::{
            Dither, IdSelector, PartitionProperties, PropertyInterpretation, QuantizeOptions,
            ReductionMethod,
        },
    };
    use branded_id::{IdRange, U32Id};
    use std::num::NonZeroUsize;
    use std::ops::RangeInclusive;
    use ty_math::TyVector3U32;
    use voxcore::{
        BVoxLayer, BVoxMaterial, BVoxObject, BVoxPalette, VoxMain, VoxObject, VoxPalette,
        VoxValuePool,
        material::{BASE_COLOR, ROUGHNESS},
    };

    /// Median-cut quantizing of `baseColor` with no dither, capped at
    /// `max_materials`.
    fn options(max_materials: usize) -> QuantizeOptions {
        QuantizeOptions {
            max_materials: NonZeroUsize::new(max_materials).unwrap(),
            property: BASE_COLOR.to_owned(),
            interpret_property: PropertyInterpretation::Auto,
            alpha: None,
            partition: PartitionProperties::Named(Vec::new()),
            method: ReductionMethod::MedianCut,
            space: None,
            dither: Dither::None,
        }
    }

    /// A palette binding `name` to a value pool of opaque linear `colors`, one
    /// material per color.
    fn color_palette(
        main: &mut VoxMain,
        name: &str,
        colors: &[[f64; 3]],
    ) -> (U32Id<BVoxPalette>, Vec<U32Id<BVoxMaterial>>) {
        let value_pool_id = main.retain_value_pool(
            VoxValuePool::vec_4_float(
                colors
                    .iter()
                    .map(|&[red, green, blue]| [red, green, blue, 1.0])
                    .collect(),
            )
            .unwrap(),
        );
        let mut palette = VoxPalette::default();
        palette
            .retain_property(name.to_owned(), value_pool_id)
            .unwrap();
        let material_ids: Vec<_> = IdRange::from_len(colors.len())
            .map(|value_id| palette.retain_material(vec![value_id]).unwrap())
            .collect();
        let palette_id = main.retain_palette(palette).unwrap();
        (palette_id, material_ids)
    }

    /// An object on a line whose voxel `i` samples `samples[i]` of each
    /// layer's palette.
    fn add_object(
        main: &mut VoxMain,
        layers: &[(U32Id<BVoxPalette>, Vec<U32Id<BVoxMaterial>>)],
    ) -> U32Id<BVoxObject> {
        let count = layers[0].1.len();
        let mut object =
            VoxObject::new("o".to_owned(), TyVector3U32::new(count as u32, 1, 1)).unwrap();
        for (palette_id, _) in layers {
            object.retain_layer(*palette_id).unwrap();
        }
        for (index, voxel_id) in IdRange::from_len(count).enumerate() {
            let row: Vec<_> = layers.iter().map(|(_, samples)| samples[index]).collect();
            object.retain_voxel(voxel_id, &row).unwrap();
        }
        main.retain_object(object).unwrap()
    }

    /// The materials object `object_id` samples through layer `layer_index`,
    /// voxel by voxel.
    fn samples(
        main: &VoxMain,
        object_id: U32Id<BVoxObject>,
        layer_index: usize,
    ) -> Vec<U32Id<BVoxMaterial>> {
        let object = main.object(object_id).unwrap();
        let (layer_id, _) = object.iter_layers().nth(layer_index).unwrap();
        object
            .iter_live_samples(layer_id)
            .unwrap()
            .map(|(_, material_id)| material_id)
            .collect()
    }

    /// The ids of object `object_id`'s layers, in layer order.
    fn layer_ids(main: &VoxMain, object_id: U32Id<BVoxObject>) -> Vec<U32Id<BVoxLayer>> {
        main.object(object_id)
            .unwrap()
            .iter_layers()
            .map(|(layer_id, _)| layer_id)
            .collect()
    }

    fn range(range: RangeInclusive<U32Id<BVoxLayer>>) -> IdSelector<BVoxLayer> {
        IdSelector::range(range).unwrap()
    }

    const RED: [f64; 3] = [1.0, 0.0, 0.0];

    const NEAR_RED: [f64; 3] = [0.98, 0.0, 0.0];

    const BLUE: [f64; 3] = [0.0, 0.0, 1.0];

    /// Two objects over one palette of red, near red, and blue. The first
    /// samples red twice and the others once; the second samples near red three
    /// times and blue once.
    fn two_objects() -> (VoxMain, [U32Id<BVoxObject>; 2], Vec<U32Id<BVoxMaterial>>) {
        let mut main = VoxMain::default();
        let (palette_id, m) = color_palette(&mut main, BASE_COLOR, &[RED, NEAR_RED, BLUE]);
        let first = add_object(&mut main, &[(palette_id, vec![m[0], m[0], m[1], m[2]])]);
        let second = add_object(&mut main, &[(palette_id, vec![m[1], m[1], m[1], m[2]])]);
        (main, [first, second], m)
    }

    #[test]
    fn each_object_clusters_apart_and_the_palette_stays() {
        let (mut main, [first, second], m) = two_objects();
        quantize_object_voxels(&mut main, &[first, second], &[], false, &options(2)).unwrap();

        // The first object's near red merges onto its more-sampled red; the
        // second already fits.
        assert_eq!(samples(&main, first, 0), [m[0], m[0], m[0], m[2]]);
        assert_eq!(samples(&main, second, 0), [m[1], m[1], m[1], m[2]]);

        let (_, palette) = main.iter_palettes().next().unwrap();
        assert_eq!(palette.material_count(), 3);
        let (_, value_pool) = main.iter_value_pools().next().unwrap();
        assert_eq!(value_pool.len(), 3);
        assert_eq!(main.validate(), Ok(()));
    }

    #[test]
    fn shared_clusters_the_selection_together() {
        // Together near red outnumbers red, so red merges onto it.
        let (mut main, [first, second], m) = two_objects();
        quantize_object_voxels(&mut main, &[first, second], &[], true, &options(2)).unwrap();
        assert_eq!(samples(&main, first, 0), [m[1], m[1], m[1], m[2]]);
        assert_eq!(samples(&main, second, 0), [m[1], m[1], m[1], m[2]]);
    }

    #[test]
    fn unselected_objects_stay() {
        let (mut main, [first, second], m) = two_objects();
        quantize_object_voxels(&mut main, &[second], &[], true, &options(1)).unwrap();
        assert_eq!(samples(&main, first, 0), [m[0], m[0], m[1], m[2]]);
        assert_eq!(samples(&main, second, 0), [m[1], m[1], m[1], m[1]]);
    }

    #[test]
    fn layer_indices_pick_layers() {
        let mut main = VoxMain::default();
        let (base_palette_id, b) = color_palette(&mut main, BASE_COLOR, &[RED, NEAR_RED]);
        let (tint_palette_id, t) = color_palette(&mut main, BASE_COLOR, &[RED, NEAR_RED]);
        let object_id = add_object(
            &mut main,
            &[
                (base_palette_id, vec![b[0], b[1]]),
                (tint_palette_id, vec![t[0], t[1]]),
            ],
        );

        let second_layer = [IdSelector::id(layer_ids(&main, object_id)[1])];
        quantize_object_voxels(&mut main, &[object_id], &second_layer, false, &options(1)).unwrap();
        assert_eq!(samples(&main, object_id, 0), [b[0], b[1]]);
        assert_eq!(samples(&main, object_id, 1), [t[0], t[0]]);

        quantize_object_voxels(&mut main, &[object_id], &[], false, &options(1)).unwrap();
        assert_eq!(samples(&main, object_id, 0), [b[0], b[0]]);
    }

    #[test]
    fn the_default_layers_skip_palettes_without_the_property() {
        let mut main = VoxMain::default();
        let (base_palette_id, b) = color_palette(&mut main, BASE_COLOR, &[RED, NEAR_RED]);
        let (other_palette_id, o) = color_palette(&mut main, "tint", &[RED, NEAR_RED]);
        let object_id = add_object(
            &mut main,
            &[
                (base_palette_id, vec![b[0], b[1]]),
                (other_palette_id, vec![o[0], o[1]]),
            ],
        );

        quantize_object_voxels(&mut main, &[object_id], &[], false, &options(1)).unwrap();
        assert_eq!(samples(&main, object_id, 0), [b[0], b[0]]);
        assert_eq!(samples(&main, object_id, 1), [o[0], o[1]]);

        let layer_ids = layer_ids(&main, object_id);

        let error = quantize_object_voxels(
            &mut main,
            &[object_id],
            &[range(layer_ids[0]..=layer_ids[1])],
            false,
            &options(1),
        )
        .unwrap_err();
        assert_eq!(
            error.to_string(),
            "layer 1 of object 0 references palette 1, which has no property `baseColor`"
        );
    }

    #[test]
    fn errors_on_a_missing_layer_or_an_unbound_property() {
        let (mut main, [first, _], _) = two_objects();

        let layer_ids = layer_ids(&main, first);

        let error = quantize_object_voxels(
            &mut main,
            &[first],
            &[range(layer_ids[0]..=U32Id::from_u32(2))],
            false,
            &options(1),
        )
        .unwrap_err();
        assert_eq!(error.to_string(), "object 0 has no layer 1");

        let roughness = QuantizeOptions {
            property: ROUGHNESS.to_owned(),
            ..options(1)
        };
        let error =
            quantize_object_voxels(&mut main, &[first], &[], false, &roughness).unwrap_err();
        assert_eq!(
            error.to_string(),
            "no selected layer references a palette with property `roughness`"
        );
    }
}
