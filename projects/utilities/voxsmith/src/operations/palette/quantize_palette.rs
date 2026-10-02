use crate::{
    Error, Result,
    utilities::{QuantizeOptions, apply_quantize_plan, choose_quantize_plan},
};
use branded_id::U32Id;
use std::collections::HashSet;
use voxcore::{BVoxLayer, BVoxObject, BVoxPalette, VoxExt, VoxMain};

/// Quantizes palette `palette_index` of `main` to at most
/// `options.max_materials` materials, snapping every layer referencing it. The
/// materials no voxel samples afterward drop along with the values only they
/// held.
pub fn quantize_palette<T: VoxExt>(
    main: &mut VoxMain<T>,
    palette_index: usize,
    options: &QuantizeOptions,
) -> Result<()> {
    let palette_id = main
        .iter_palettes()
        .nth(palette_index)
        .map(|(palette_id, _)| palette_id)
        .ok_or_else(|| {
            Error::invalid(format!(
                "palette index {palette_index} is out of range; the document has {} palette(s)",
                main.palette_count()
            ))
        })?;

    let layers: Vec<_> = main
        .iter_objects()
        .flat_map(|(object_id, object)| {
            object
                .iter_layers()
                .filter(|&(_, layer_palette_id)| layer_palette_id == palette_id)
                .map(move |(layer_id, _)| (object_id, layer_id))
        })
        .collect();

    if let Some(plan) = choose_quantize_plan(main, palette_id, &layers, options)? {
        apply_quantize_plan(main, &plan, &layers, options.dither)?;
    }

    release_unsampled_materials(main, palette_id, &layers)?;

    main.gc()?;

    Ok(())
}

/// Releases the materials of `palette_id` no live voxel of `layers` samples,
/// then the value-pool values only those materials held.
fn release_unsampled_materials<T: VoxExt>(
    main: &mut VoxMain<T>,
    palette_id: U32Id<BVoxPalette>,
    layers: &[(U32Id<BVoxObject>, U32Id<BVoxLayer>)],
) -> Result<()> {
    let mut sampled_ids = HashSet::new();
    for &(object_id, layer_id) in layers {
        let samples = main
            .object(object_id)
            .and_then(|object| object.iter_live_samples(layer_id))
            .expect("a referencing layer is one of its object's");
        sampled_ids.extend(samples.map(|(_, material_id)| material_id));
    }

    let palette = main
        .palette(palette_id)
        .expect("the quantized palette is one of the main's");

    let doomed_ids: HashSet<_> = palette
        .iter_materials()
        .filter(|material_id| !sampled_ids.contains(material_id))
        .collect();

    // The values the doomed materials hold, in listing order.
    let mut doomed_value_ids = Vec::new();
    for material_id in palette.iter_materials() {
        if !doomed_ids.contains(&material_id) {
            continue;
        }

        for (property_id, property) in palette.iter_properties() {
            let value_id = palette
                .value_id(material_id, property_id)
                .expect("a live material holds a value for every property");
            doomed_value_ids.push((property.value_pool_id, value_id));
        }
    }

    main.release_materials(palette_id, &doomed_ids)?;

    // Every value some material of any palette still holds.
    let mut held_value_ids = HashSet::new();
    for (_, palette) in main.iter_palettes() {
        for (property_id, property) in palette.iter_properties() {
            for material_id in palette.iter_materials() {
                let value_id = palette
                    .value_id(material_id, property_id)
                    .expect("a live material holds a value for every property");
                held_value_ids.insert((property.value_pool_id, value_id));
            }
        }
    }

    let mut released_value_ids = HashSet::new();
    for (value_pool_id, value_id) in doomed_value_ids {
        if !held_value_ids.contains(&(value_pool_id, value_id))
            && released_value_ids.insert((value_pool_id, value_id))
        {
            main.release_value_pool_value(value_pool_id, value_id)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        operations::palette::quantize_palette,
        utilities::{
            AlphaMode, ColorSpace, Dither, PartitionProperties, PropertyInterpretation,
            QuantizeOptions, ReductionMethod,
        },
    };
    use branded_id::U32Id;
    use std::num::NonZeroUsize;
    use ty_math::{TyLinSrgbaF64, TySrgbaU8, TyVector3U32};
    use voxcore::{
        BVoxMaterial, BVoxObject, BVoxPalette, BVoxValuePoolValue, VoxMain, VoxObject, VoxPalette,
        VoxValueColumn, VoxValuePool,
        color::{lin_srgba_f64_from_srgba_u8, srgba_u8_from_lin_srgba_f64},
        material::{BASE_COLOR, METALLIC, ROUGHNESS},
    };

    /// The branded value id `index`.
    fn value_id(index: usize) -> U32Id<BVoxValuePoolValue> {
        U32Id::from_u32(index as u32)
    }

    /// Median-cut quantizing of `baseColor` in OKLab with no dither, capped at
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

    /// The linear-light components of a `#RRGGBBAA` hex string.
    fn linear_rgba(hex: &str) -> [f64; 4] {
        let digits = hex.strip_prefix('#').expect("a leading #");
        let byte = |index: usize| {
            u8::from_str_radix(&digits[index * 2..index * 2 + 2], 16).expect("two hex digits")
        };
        lin_srgba_f64_from_srgba_u8(TySrgbaU8::new(byte(0), byte(1), byte(2), byte(3))).into()
    }

    /// A `#RRGGBBAA` uppercase hex string of a stored linear color.
    fn hex_of(color: [f64; 4]) -> String {
        let bytes = <[u8; 4]>::from(srgba_u8_from_lin_srgba_f64(TyLinSrgbaF64::from(color)));
        format!(
            "#{:02X}{:02X}{:02X}{:02X}",
            bytes[0], bytes[1], bytes[2], bytes[3]
        )
    }

    /// The value a material holds for the property `name`.
    fn material_value<'a, T>(
        main: &'a VoxMain,
        palette_id: U32Id<BVoxPalette>,
        material_id: U32Id<BVoxMaterial>,
        name: &str,
        column: fn(&'a VoxValuePool) -> Option<VoxValueColumn<'a, T>>,
    ) -> &'a T {
        let property_id = main
            .palette(palette_id)
            .unwrap()
            .property_id_by_name(name)
            .unwrap();

        let (value_pool, value_id) = main
            .material_value(palette_id, material_id, property_id)
            .unwrap();

        column(value_pool).unwrap().get(value_id).unwrap()
    }

    /// The `#RRGGBBAA` hex of a material's `baseColor`, uppercase.
    fn material_hex(
        main: &VoxMain,
        palette_id: U32Id<BVoxPalette>,
        material_id: U32Id<BVoxMaterial>,
    ) -> String {
        hex_of(*material_value(
            main,
            palette_id,
            material_id,
            BASE_COLOR,
            VoxValuePool::vec_4_float_values,
        ))
    }

    /// One object over a palette of `properties`, each drawing from its own
    /// value pool, and one material per `rows` entry picking a value index per
    /// property. Voxel `i` samples material `i`, and `repeats[i]` adds extra
    /// voxels on material `i`.
    fn main_with_rows(
        properties: Vec<(&str, VoxValuePool)>,
        rows: &[Vec<usize>],
        repeats: &[usize],
    ) -> (VoxMain, U32Id<BVoxPalette>, U32Id<BVoxObject>) {
        let mut main = VoxMain::default();

        let mut palette = VoxPalette::default();
        for (name, value_pool) in properties {
            let value_pool_id = main.retain_value_pool(value_pool);
            palette
                .retain_property(name.to_owned(), value_pool_id, value_id(0))
                .unwrap();
        }
        let material_ids: Vec<_> = rows
            .iter()
            .map(|row| {
                palette
                    .retain_material(row.iter().map(|&index| value_id(index)).collect())
                    .unwrap()
            })
            .collect();

        let count: usize = rows.len() + repeats.iter().sum::<usize>();
        let mut object =
            VoxObject::new("o".to_owned(), TyVector3U32::new(count as u32, 1, 1)).unwrap();
        let palette_id = main.retain_palette(palette).unwrap();
        object.retain_layer(palette_id, material_ids[0]);

        let mut voxel_index = 0u32;
        for (index, &material_id) in material_ids.iter().enumerate() {
            for _ in 0..1 + repeats.get(index).copied().unwrap_or(0) {
                object
                    .retain_voxel(U32Id::from_u32(voxel_index), &[material_id])
                    .unwrap();
                voxel_index += 1;
            }
        }
        let object_id = main.retain_object(object).unwrap();
        (main, palette_id, object_id)
    }

    /// One object over a palette of `#RRGGBBAA` `colors`, each material with a
    /// distinct `tag` float so a merge's whole-material take is visible.
    fn main_with_colors(
        colors: &[&str],
        repeats: &[usize],
    ) -> (VoxMain, U32Id<BVoxPalette>, U32Id<BVoxObject>) {
        main_with_rows(
            vec![
                (
                    BASE_COLOR,
                    VoxValuePool::vec_4_float(
                        colors.iter().map(|color| linear_rgba(color)).collect(),
                    )
                    .unwrap(),
                ),
                (
                    "tag",
                    VoxValuePool::float((0..colors.len()).map(|index| index as f64).collect())
                        .unwrap(),
                ),
            ],
            &(0..colors.len())
                .map(|index| vec![index, index])
                .collect::<Vec<_>>(),
            repeats,
        )
    }

    /// One object of size `bounds` over a `baseColor` palette of `colors` and
    /// one live voxel per `(position, color index)` entry.
    fn grid_main(
        bounds: TyVector3U32,
        colors: &[&str],
        voxels: &[(TyVector3U32, usize)],
    ) -> (VoxMain, U32Id<BVoxPalette>, U32Id<BVoxObject>) {
        let mut main = VoxMain::default();

        let base_value_pool_id = main.retain_value_pool(
            VoxValuePool::vec_4_float(colors.iter().map(|color| linear_rgba(color)).collect())
                .unwrap(),
        );

        let mut palette = VoxPalette::default();
        palette
            .retain_property(BASE_COLOR.to_owned(), base_value_pool_id, value_id(0))
            .unwrap();
        let material_ids: Vec<_> = (0..colors.len())
            .map(|index| palette.retain_material(vec![value_id(index)]).unwrap())
            .collect();

        let mut object = VoxObject::new("o".to_owned(), bounds).unwrap();
        let palette_id = main.retain_palette(palette).unwrap();
        object.retain_layer(palette_id, material_ids[0]);

        for &(position, color_index) in voxels {
            let voxel_id = object.voxel_id(position).unwrap();
            object
                .retain_voxel(voxel_id, &[material_ids[color_index]])
                .unwrap();
        }

        let object_id = main.retain_object(object).unwrap();
        (main, palette_id, object_id)
    }

    fn material_count(main: &VoxMain, palette_id: U32Id<BVoxPalette>) -> usize {
        main.palette(palette_id).unwrap().material_count()
    }

    /// The `baseColor` hexes still in the palette, sorted.
    fn colors(main: &VoxMain, palette_id: U32Id<BVoxPalette>) -> Vec<String> {
        let mut colors: Vec<_> = main
            .palette(palette_id)
            .unwrap()
            .iter_materials()
            .map(|material_id| material_hex(main, palette_id, material_id))
            .collect();
        colors.sort();
        colors
    }

    /// The material the voxel at `position` samples through the object's
    /// first layer.
    fn voxel_material(
        main: &VoxMain,
        object_id: U32Id<BVoxObject>,
        position: TyVector3U32,
    ) -> U32Id<BVoxMaterial> {
        let object = main.object(object_id).unwrap();
        let (layer_id, _) = object.iter_layers().next().unwrap();
        let voxel_id = object.voxel_id(position).unwrap();
        object.voxel_material(voxel_id, layer_id).unwrap()
    }

    /// The `baseColor` hex the voxel at `position` samples.
    fn voxel_color(
        main: &VoxMain,
        object_id: U32Id<BVoxObject>,
        palette_id: U32Id<BVoxPalette>,
        position: TyVector3U32,
    ) -> String {
        material_hex(main, palette_id, voxel_material(main, object_id, position))
    }

    /// The number of values in the value pool the property `name` draws from
    /// in the first palette that carries it.
    fn value_pool_len(main: &VoxMain, name: &str) -> usize {
        for (_, palette) in main.iter_palettes() {
            if let Some(property_id) = palette.property_id_by_name(name) {
                let value_pool_id = palette.property(property_id).unwrap().value_pool_id;
                return main.value_pool(value_pool_id).unwrap().len();
            }
        }
        panic!("no palette carries {name}");
    }

    /// The error message quantizing the first palette under `options` gives.
    fn error_of(main: &mut VoxMain, options: &QuantizeOptions) -> String {
        quantize_palette(main, 0, options).unwrap_err().to_string()
    }

    #[test]
    fn leaves_a_palette_within_the_cap_unchanged() {
        let (mut main, palette_id, _) = main_with_colors(&["#FF0000FF", "#00FF00FF"], &[]);
        quantize_palette(&mut main, 0, &options(5)).unwrap();
        assert_eq!(material_count(&main, palette_id), 2);
    }

    #[test]
    fn merges_near_colors_and_keeps_a_real_representative_material() {
        // Two near reds and a blue; cap 2 fuses the reds onto the more-sampled
        // second red, whose whole material (tag 1) survives.
        let (mut main, palette_id, object_id) =
            main_with_colors(&["#FE0000FF", "#FF0000FF", "#0000FFFF"], &[0, 3, 0]);
        quantize_palette(&mut main, 0, &options(2)).unwrap();
        assert_eq!(main.validate(), Ok(()));
        assert_eq!(colors(&main, palette_id), ["#0000FFFF", "#FF0000FF"]);

        let material_id = voxel_material(&main, object_id, TyVector3U32::new(0, 0, 0));
        assert_eq!(
            material_value(
                &main,
                palette_id,
                material_id,
                "tag",
                VoxValuePool::float_values
            ),
            &1.0
        );
    }

    #[test]
    fn every_method_reduces_to_the_cap() {
        for method in [
            ReductionMethod::Kmeans,
            ReductionMethod::MedianCut,
            ReductionMethod::Octree,
        ] {
            let (mut main, palette_id, _) =
                main_with_colors(&["#FF0000FF", "#00FF00FF", "#0000FFFF", "#FFFF00FF"], &[]);
            quantize_palette(
                &mut main,
                0,
                &QuantizeOptions {
                    method,
                    ..options(2)
                },
            )
            .unwrap();
            let after = material_count(&main, palette_id);
            assert!((1..=2).contains(&after), "method {method:?} left {after}");
            assert_eq!(main.validate(), Ok(()), "method {method:?}");
        }
    }

    #[test]
    fn dither_is_inert_under_the_cap_and_reduces_over_it() {
        let (mut main, palette_id, _) = main_with_colors(&["#FF0000FF", "#00FF00FF"], &[]);
        let dithered = QuantizeOptions {
            dither: Dither::FloydSteinberg,
            ..options(5)
        };
        quantize_palette(&mut main, 0, &dithered).unwrap();
        assert_eq!(material_count(&main, palette_id), 2);

        for dither in [Dither::FloydSteinberg, Dither::Ordered] {
            let (mut main, palette_id, _) =
                main_with_colors(&["#FF0000FF", "#00FF00FF", "#0000FFFF"], &[]);
            quantize_palette(
                &mut main,
                0,
                &QuantizeOptions {
                    dither,
                    ..options(2)
                },
            )
            .unwrap();
            assert_eq!(material_count(&main, palette_id), 2, "dither {dither:?}");
            assert_eq!(main.validate(), Ok(()), "dither {dither:?}");
        }
    }

    #[test]
    fn reduces_in_every_color_space() {
        for space in [ColorSpace::Lab, ColorSpace::Oklab, ColorSpace::Srgb] {
            let (mut main, palette_id, _) =
                main_with_colors(&["#FF0000FF", "#00FF00FF", "#0000FFFF", "#FFFF00FF"], &[]);
            let spaced = QuantizeOptions {
                space: Some(space),
                ..options(2)
            };
            quantize_palette(&mut main, 0, &spaced).unwrap();
            assert_eq!(material_count(&main, palette_id), 2, "space {space:?}");
            assert_eq!(main.validate(), Ok(()), "space {space:?}");
        }
    }

    #[test]
    fn ordered_dither_lands_a_known_pattern() {
        // srgb space, so coords are the #-bytes over 255: black (0,0,0), mid
        // #800000 (~0.502,0,0), red (1,0,0), all on x. The four mid voxels at
        // z=0 are under test; the black and seven red seeds at z=1,2 make black
        // and red the representatives (red outvotes mid 7 to 4).
        let black = "#000000FF";
        let mid = "#800000FF";
        let red = "#FF0000FF";
        let mut voxels = vec![
            (TyVector3U32::new(0, 0, 0), 1),
            (TyVector3U32::new(1, 0, 0), 1),
            (TyVector3U32::new(2, 0, 0), 1),
            (TyVector3U32::new(3, 0, 0), 1),
            (TyVector3U32::new(0, 0, 1), 0),
        ];
        for x in 1..4 {
            voxels.push((TyVector3U32::new(x, 0, 1), 2));
        }
        for x in 0..4 {
            voxels.push((TyVector3U32::new(x, 0, 2), 2));
        }
        let (mut main, palette_id, object_id) =
            grid_main(TyVector3U32::new(4, 1, 3), &[black, mid, red], &voxels);

        let ordered = QuantizeOptions {
            space: Some(ColorSpace::Srgb),
            dither: Dither::Ordered,
            ..options(2)
        };
        quantize_palette(&mut main, 0, &ordered).unwrap();
        assert_eq!(material_count(&main, palette_id), 2);
        assert_eq!(main.validate(), Ok(()));

        // Representatives differ only on x, so only the x offset decides:
        // bayer(x,0,0) is 0, 48, 6, 54 for x=0..3, below/above the midpoint
        // 31.5, so mid snaps black, red, black, red.
        let expected = [black, red, black, red];
        for (x, want) in expected.iter().enumerate() {
            let got = voxel_color(
                &main,
                object_id,
                palette_id,
                TyVector3U32::new(x as u32, 0, 0),
            );
            assert_eq!(&got, want, "x = {x}");
        }
    }

    #[test]
    fn ordered_dither_never_moves_a_voxel_off_its_representative() {
        // A 4x4x4 grid of red and blue with one near red beside red. Red and
        // blue stay the representatives, so every red or blue voxel keeps its
        // color whatever the Bayer offset at its position.
        let colors = ["#FF0000FF", "#FE0000FF", "#0000FFFF"];
        let mut voxels = Vec::new();
        for x in 0..4 {
            for y in 0..4 {
                for z in 0..4 {
                    let color_index = if (x + y + z) % 2 == 0 { 0 } else { 2 };
                    voxels.push((TyVector3U32::new(x, y, z), color_index));
                }
            }
        }
        voxels[1].1 = 1;
        let (mut main, palette_id, object_id) =
            grid_main(TyVector3U32::new(4, 4, 4), &colors, &voxels);

        let ordered = QuantizeOptions {
            dither: Dither::Ordered,
            ..options(2)
        };
        quantize_palette(&mut main, 0, &ordered).unwrap();

        for &(position, color_index) in &voxels {
            if color_index == 1 {
                continue;
            }
            assert_eq!(
                voxel_color(&main, object_id, palette_id, position),
                colors[color_index],
                "position {position:?}"
            );
        }
    }

    #[test]
    fn floyd_steinberg_dither_lands_a_known_pattern() {
        // The same x-axis colors on a (1,1,10) line: the mid voxels are the
        // highest ids (z=6..9) so error diffuses only among them; the black and
        // five red seeds (z=0..5) snap to their own representative with zero
        // residual. Red outvotes mid 5 to 4.
        let black = "#000000FF";
        let mid = "#800000FF";
        let red = "#FF0000FF";
        let mut voxels = vec![(TyVector3U32::new(0, 0, 0), 0)];
        for z in 1..6 {
            voxels.push((TyVector3U32::new(0, 0, z), 2));
        }
        for z in 6..10 {
            voxels.push((TyVector3U32::new(0, 0, z), 1));
        }
        let (mut main, palette_id, object_id) =
            grid_main(TyVector3U32::new(1, 1, 10), &[black, mid, red], &voxels);

        let diffused = QuantizeOptions {
            space: Some(ColorSpace::Srgb),
            dither: Dither::FloydSteinberg,
            ..options(2)
        };
        quantize_palette(&mut main, 0, &diffused).unwrap();
        assert_eq!(material_count(&main, palette_id), 2);
        assert_eq!(main.validate(), Ok(()));

        // On a line only +z carries, at 3/8. Tracing mid = 0.502 from zero
        // error:
        //   z=6: 0.502          -> red   (residual -0.498, carries -0.187)
        //   z=7: 0.502 - 0.187  -> black (residual  0.315, carries  0.118)
        //   z=8: 0.502 + 0.118  -> red   (residual -0.380, carries -0.142)
        //   z=9: 0.502 - 0.142  -> black
        let expected = [(6, red), (7, black), (8, red), (9, black)];
        for (z, want) in expected {
            let got = voxel_color(&main, object_id, palette_id, TyVector3U32::new(0, 0, z));
            assert_eq!(&got, want, "z = {z}");
        }
    }

    #[test]
    fn drops_the_merged_away_values() {
        let (mut main, palette_id, _) =
            main_with_colors(&["#FE0000FF", "#FF0000FF", "#0000FFFF"], &[0, 3, 0]);
        quantize_palette(&mut main, 0, &options(2)).unwrap();

        assert_eq!(value_pool_len(&main, BASE_COLOR), 2);
        assert_eq!(value_pool_len(&main, "tag"), 2);
        assert_eq!(main.validate(), Ok(()));
        assert_eq!(colors(&main, palette_id), ["#0000FFFF", "#FF0000FF"]);
    }

    #[test]
    fn keeps_values_other_palettes_hold_and_values_nothing_held() {
        // A second palette shares the tag value pool and holds tag 0, which
        // the merged-away first red also held. The pool's spare value 3 was
        // never held by anything.
        let (mut main, palette_id, _) =
            main_with_colors(&["#FE0000FF", "#FF0000FF", "#0000FFFF"], &[0, 3, 0]);
        let tag_value_pool_id = {
            let palette = main.palette(palette_id).unwrap();
            let tag_property_id = palette.property_id_by_name("tag").unwrap();
            palette.property(tag_property_id).unwrap().value_pool_id
        };
        let spare_value_pool_id =
            main.retain_value_pool(VoxValuePool::float(vec![0.0, 1.0, 2.0, 3.0]).unwrap());
        let mut other = VoxPalette::default();
        other
            .retain_property("tag".to_owned(), tag_value_pool_id, value_id(0))
            .unwrap();
        other.retain_material(vec![value_id(0)]).unwrap();
        main.retain_palette(other).unwrap();

        quantize_palette(&mut main, 0, &options(2)).unwrap();

        assert_eq!(main.value_pool(tag_value_pool_id).unwrap().len(), 3);
        assert_eq!(main.value_pool(spare_value_pool_id).unwrap().len(), 4);
        assert_eq!(main.validate(), Ok(()));
    }

    #[test]
    fn drops_materials_no_voxel_samples() {
        let (mut main, palette_id, object_id) =
            main_with_colors(&["#FF0000FF", "#00FF00FF", "#0000FFFF"], &[]);
        let object = main.object(object_id).unwrap();
        let voxel_id = object.voxel_id(TyVector3U32::new(2, 0, 0)).unwrap();
        main.release_voxel(object_id, voxel_id).unwrap();

        quantize_palette(&mut main, 0, &options(5)).unwrap();
        assert_eq!(colors(&main, palette_id), ["#00FF00FF", "#FF0000FF"]);
        assert_eq!(value_pool_len(&main, BASE_COLOR), 2);
        assert_eq!(main.validate(), Ok(()));
    }

    #[test]
    fn dithering_reduces_around_a_second_layer() {
        let (mut main, palette_id, object_id) =
            main_with_colors(&["#FF0000FF", "#00FF00FF", "#0000FFFF"], &[]);
        let strength_value_pool_id =
            main.retain_value_pool(VoxValuePool::float(vec![1.5]).unwrap());

        let mut glow_palette = VoxPalette::default();
        glow_palette
            .retain_property(
                "emissiveStrength".to_owned(),
                strength_value_pool_id,
                value_id(0),
            )
            .unwrap();
        glow_palette.retain_material(vec![value_id(0)]).unwrap();
        let glow_palette_id = main.retain_palette(glow_palette).unwrap();
        main.retain_layer(object_id, glow_palette_id, U32Id::from_u32(0))
            .unwrap();

        let dithered = QuantizeOptions {
            dither: Dither::FloydSteinberg,
            ..options(2)
        };
        quantize_palette(&mut main, 0, &dithered).unwrap();

        assert_eq!(material_count(&main, palette_id), 2);
        assert_eq!(main.validate(), Ok(()));
        assert_eq!(main.object(object_id).unwrap().layer_count(), 2);
        assert_eq!(material_count(&main, glow_palette_id), 1);
    }

    #[test]
    fn alpha_partitions_by_default_and_merges_when_ignored() {
        // A half-clear red keeps apart from the opaque colors by default, so
        // the opaque red and blue merge instead.
        let (mut main, palette_id, _) =
            main_with_colors(&["#FF0000FF", "#FF000080", "#0000FFFF"], &[1, 0, 0]);
        quantize_palette(&mut main, 0, &options(2)).unwrap();
        assert_eq!(colors(&main, palette_id), ["#FF000080", "#FF0000FF"]);

        // Ignoring alpha, the reds merge and so do the blues.
        let (mut main, palette_id, _) = main_with_colors(
            &["#FF0000FF", "#FF000080", "#0000FFFF", "#0000FEFF"],
            &[1, 0, 0, 0],
        );
        let ignored = QuantizeOptions {
            alpha: Some(AlphaMode::Ignore),
            ..options(2)
        };
        quantize_palette(&mut main, 0, &ignored).unwrap();
        assert_eq!(colors(&main, palette_id), ["#0000FFFF", "#FF0000FF"]);
    }

    #[test]
    fn errors_when_alpha_partitions_outnumber_the_cap() {
        let (mut main, _, _) = main_with_colors(&["#FF0000FF", "#FF000080", "#FF000040"], &[]);
        assert_eq!(
            error_of(&mut main, &options(2)),
            "the partitions split palette 0's sampled materials into 3 groups, more than the \
             2 material(s) allowed"
        );
    }

    #[test]
    fn alpha_distance_clusters_on_four_axes() {
        // Two opaque reds and a nearly clear red: with alpha as a distance,
        // the clear red stands apart and the opaque reds merge.
        let (mut main, palette_id, _) =
            main_with_colors(&["#FF0000FF", "#FE0000FF", "#FF000010"], &[1, 0, 0]);
        let distance = QuantizeOptions {
            alpha: Some(AlphaMode::Distance),
            ..options(2)
        };
        quantize_palette(&mut main, 0, &distance).unwrap();
        assert_eq!(colors(&main, palette_id), ["#FF000010", "#FF0000FF"]);
    }

    #[test]
    fn partition_keeps_metals_and_dielectrics_apart() {
        // Two near reds, one metal and one not, plus a blue metal. Clustering
        // alone merges the reds; partitioning on metallic keeps them apart and
        // merges the two metals instead.
        let properties = || {
            vec![
                (
                    BASE_COLOR,
                    VoxValuePool::vec_4_float(vec![
                        linear_rgba("#FF0000FF"),
                        linear_rgba("#FE0000FF"),
                        linear_rgba("#0000FFFF"),
                    ])
                    .unwrap(),
                ),
                (METALLIC, VoxValuePool::float(vec![0.0, 1.0]).unwrap()),
            ]
        };
        let rows = [vec![0, 0], vec![1, 1], vec![2, 1]];

        let (mut main, palette_id, _) = main_with_rows(properties(), &rows, &[1, 0, 0]);
        quantize_palette(&mut main, 0, &options(2)).unwrap();
        assert_eq!(colors(&main, palette_id), ["#0000FFFF", "#FF0000FF"]);

        let (mut main, palette_id, _) = main_with_rows(properties(), &rows, &[0, 0, 1]);
        let partitioned = QuantizeOptions {
            partition: PartitionProperties::Named(vec![METALLIC.to_owned()]),
            ..options(2)
        };
        quantize_palette(&mut main, 0, &partitioned).unwrap();
        assert_eq!(colors(&main, palette_id), ["#0000FFFF", "#FF0000FF"]);
        let metallic: Vec<_> = main
            .palette(palette_id)
            .unwrap()
            .iter_materials()
            .map(|material_id| {
                *material_value(
                    &main,
                    palette_id,
                    material_id,
                    METALLIC,
                    VoxValuePool::float_values,
                )
            })
            .collect();
        assert_eq!(metallic, [0.0, 1.0]);
    }

    #[test]
    fn partitioning_on_every_property_errors_past_the_cap() {
        // Each material's distinct tag makes it a partition of its own.
        let (mut main, _, _) = main_with_colors(&["#FF0000FF", "#FE0000FF", "#0000FFFF"], &[]);
        let everything = QuantizeOptions {
            partition: PartitionProperties::All,
            ..options(2)
        };
        assert_eq!(
            error_of(&mut main, &everything),
            "the partitions split palette 0's sampled materials into 3 groups, more than the \
             2 material(s) allowed"
        );
    }

    #[test]
    fn partitioned_kmeans_and_octree_give_each_partition_a_slot() {
        for method in [ReductionMethod::Kmeans, ReductionMethod::Octree] {
            let (mut main, palette_id, _) = main_with_rows(
                vec![
                    (
                        BASE_COLOR,
                        VoxValuePool::vec_4_float(vec![
                            linear_rgba("#FF0000FF"),
                            linear_rgba("#00FF00FF"),
                            linear_rgba("#0000FFFF"),
                        ])
                        .unwrap(),
                    ),
                    (METALLIC, VoxValuePool::float(vec![0.0, 1.0]).unwrap()),
                ],
                &[vec![0, 0], vec![1, 0], vec![2, 1]],
                &[],
            );
            let partitioned = QuantizeOptions {
                method,
                partition: PartitionProperties::Named(vec![METALLIC.to_owned()]),
                ..options(2)
            };
            quantize_palette(&mut main, 0, &partitioned).unwrap();
            let mut metallic: Vec<_> = main
                .palette(palette_id)
                .unwrap()
                .iter_materials()
                .map(|material_id| {
                    *material_value(
                        &main,
                        palette_id,
                        material_id,
                        METALLIC,
                        VoxValuePool::float_values,
                    )
                })
                .collect();
            metallic.dedup();
            assert_eq!(metallic.len(), 2, "method {method:?}");
        }
    }

    #[test]
    fn quantizes_a_numeric_property() {
        // Two smooth and two rough materials: cap 2 merges each pair onto its
        // more-sampled material, ties to the lower id.
        let (mut main, palette_id, _) = main_with_rows(
            vec![(
                ROUGHNESS,
                VoxValuePool::float(vec![0.1, 0.12, 0.9, 0.92]).unwrap(),
            )],
            &[vec![0], vec![1], vec![2], vec![3]],
            &[0, 2, 0, 0],
        );
        let roughness = QuantizeOptions {
            property: ROUGHNESS.to_owned(),
            ..options(2)
        };
        quantize_palette(&mut main, 0, &roughness).unwrap();
        let values: Vec<_> = main
            .palette(palette_id)
            .unwrap()
            .iter_materials()
            .map(|material_id| {
                *material_value(
                    &main,
                    palette_id,
                    material_id,
                    ROUGHNESS,
                    VoxValuePool::float_values,
                )
            })
            .collect();
        assert_eq!(values, [0.12, 0.9]);
    }

    #[test]
    fn errors_when_options_do_not_fit_the_property() {
        let (mut main, _, _) = main_with_rows(
            vec![
                (
                    BASE_COLOR,
                    VoxValuePool::vec_3_float(vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]).unwrap(),
                ),
                (ROUGHNESS, VoxValuePool::float(vec![0.1, 0.9]).unwrap()),
                ("flag", VoxValuePool::boolean(vec![false, true])),
                (
                    "tint",
                    VoxValuePool::vec_4_float(vec![[0.0; 4], [1.0; 4]]).unwrap(),
                ),
            ],
            &[vec![0, 0, 0, 0], vec![1, 1, 1, 1]],
            &[],
        );

        let cases = [
            (
                QuantizeOptions {
                    property: "missing".to_owned(),
                    ..options(1)
                },
                "palette 0 has no property `missing`",
            ),
            (
                QuantizeOptions {
                    alpha: Some(AlphaMode::Distance),
                    ..options(1)
                },
                "property `baseColor` reads as a 3-component color, which has no alpha to take \
                 part",
            ),
            (
                QuantizeOptions {
                    property: ROUGHNESS.to_owned(),
                    space: Some(ColorSpace::Lab),
                    ..options(1)
                },
                "property `roughness` reads as numeric, which a color space does not apply to",
            ),
            (
                QuantizeOptions {
                    property: ROUGHNESS.to_owned(),
                    interpret_property: PropertyInterpretation::LinearColor,
                    ..options(1)
                },
                "property `roughness` holds float values, but a color reading needs \
                 vec-3-float or vec-4-float",
            ),
            (
                QuantizeOptions {
                    property: "flag".to_owned(),
                    ..options(1)
                },
                "property `flag` holds bool values, which read as no points",
            ),
            (
                QuantizeOptions {
                    property: "tint".to_owned(),
                    method: ReductionMethod::Octree,
                    ..options(1)
                },
                "property `tint` reads as 4D points, but octree clusters 3D points",
            ),
            (
                QuantizeOptions {
                    partition: PartitionProperties::Named(vec![METALLIC.to_owned()]),
                    ..options(1)
                },
                "palette 0 has no property `metallic` to partition on",
            ),
            (
                QuantizeOptions {
                    partition: PartitionProperties::Named(vec![BASE_COLOR.to_owned()]),
                    ..options(1)
                },
                "property `baseColor` is the quantized property, so it cannot also partition",
            ),
        ];

        for (options, message) in cases {
            assert_eq!(error_of(&mut main, &options), message);
        }
    }

    #[test]
    fn errors_on_an_index_past_the_palettes() {
        let (mut main, _, _) = main_with_colors(&["#FF0000FF"], &[]);
        assert_eq!(
            quantize_palette(&mut main, 1, &options(1))
                .unwrap_err()
                .to_string(),
            "palette index 1 is out of range; the document has 1 palette(s)"
        );
    }
}
