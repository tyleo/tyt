use crate::{
    Result,
    utilities::{Dither, QuantizePlan, QuantizePoint},
};
use branded_id::U32Id;
use std::{cmp::Ordering, collections::HashMap};
use ty_math::{TyVector3U32, TyVector4F64};
use voxcore::{BVoxLayer, BVoxMaterial, BVoxObject, VoxExt, VoxMain};

/// Snaps every live sample of `layers` onto its representative in `plan`. A
/// dithered snap instead picks the nearest representative in the material's
/// partition.
pub fn apply_quantize_plan<T: VoxExt>(
    main: &mut VoxMain<T>,
    plan: &QuantizePlan,
    layers: &[(U32Id<BVoxObject>, U32Id<BVoxLayer>)],
    dither: Dither,
) -> Result<()> {
    let spacings = representative_spacings(plan);

    for &(object_id, layer_id) in layers {
        let object = main
            .object(object_id)
            .expect("a quantized layer's object is one of the main's");
        let bounds = object.bounds();

        let layer_ids: Vec<_> = object.iter_layers().map(|(layer_id, _)| layer_id).collect();
        let slot_index = layer_ids
            .iter()
            .position(|&other_layer_id| other_layer_id == layer_id)
            .expect("a quantized layer is one of its object's");

        // Floyd-Steinberg's sparse per-voxel error; ordered needs no buffer.
        let mut errors: HashMap<TyVector3U32, TyVector4F64> = HashMap::new();

        // Every snap reads the layer as it was, so gather the whole layer's
        // rewritten rows before retaining any.
        let mut rows = Vec::new();

        let samples = object
            .iter_live_samples(layer_id)
            .expect("a quantized layer is one of its object's");

        for (voxel_id, material_id) in samples {
            let representative_id = match dither {
                Dither::None => plan.representative_ids[&material_id],

                Dither::FloydSteinberg | Dither::Ordered => {
                    let (point, partition_index) = plan.points[&material_id];
                    let position = object
                        .voxel_position(voxel_id)
                        .expect("a live voxel is within the grid");

                    let offset = match dither {
                        Dither::FloydSteinberg => {
                            errors.get(&position).copied().unwrap_or(TyVector4F64::ZERO)
                        }

                        Dither::None => unreachable!("an undithered snap takes the plan's"),

                        Dither::Ordered => ordered_offset(
                            position,
                            spacings[&plan.representative_ids[&material_id]],
                            plan.dimensions,
                        ),
                    };

                    let target = point.coords + offset;

                    let chosen = nearest_representative(
                        target,
                        &plan.partition_representatives[partition_index],
                    );

                    if dither == Dither::FloydSteinberg {
                        diffuse_error(&mut errors, bounds, position, target - chosen.coords);
                    }

                    chosen.material_id
                }
            };

            if representative_id != material_id {
                let mut row: Vec<_> = layer_ids
                    .iter()
                    .map(|&row_layer_id| {
                        object
                            .voxel_material(voxel_id, row_layer_id)
                            .expect("a live voxel samples every layer")
                    })
                    .collect();
                row[slot_index] = representative_id;
                rows.push((voxel_id, row));
            }
        }

        for (voxel_id, row) in rows {
            main.retain_voxel(object_id, voxel_id, &row)?;
        }
    }

    Ok(())
}

/// The representative nearest `coords`, ties to the lowest material id so the
/// snap is deterministic.
fn nearest_representative(
    coords: TyVector4F64,
    representatives: &[QuantizePoint],
) -> QuantizePoint {
    representatives
        .iter()
        .copied()
        .min_by(|a, b| {
            (coords - a.coords)
                .length_squared()
                .partial_cmp(&(coords - b.coords).length_squared())
                .unwrap_or(Ordering::Equal)
                .then_with(|| a.material_id.cmp(&b.material_id))
        })
        .expect("a partition keeps at least one representative")
}

/// Pushes `error` to the three raster-forward neighbors. Floyd-Steinberg has
/// no 3D kernel, so these weights are this engine's own.
fn diffuse_error(
    errors: &mut HashMap<TyVector3U32, TyVector4F64>,
    bounds: TyVector3U32,
    position: TyVector3U32,
    error: TyVector4F64,
) {
    let mut push = |neighbor: TyVector3U32, weight: f64| {
        let slot = errors.entry(neighbor).or_insert(TyVector4F64::ZERO);

        *slot += error * weight;
    };

    if position.z + 1 < bounds.z {
        push(position + TyVector3U32::Z, 3.0 / 8.0);
    }

    if position.y + 1 < bounds.y {
        push(position + TyVector3U32::Y, 3.0 / 8.0);
    }

    if position.x + 1 < bounds.x {
        push(position + TyVector3U32::X, 2.0 / 8.0);
    }
}

/// Each representative's distance to the nearest other representative of its
/// partition, zero for a lone one, which disables the ordered offset.
fn representative_spacings(plan: &QuantizePlan) -> HashMap<U32Id<BVoxMaterial>, f64> {
    let mut spacings = HashMap::new();

    for representatives in &plan.partition_representatives {
        for representative in representatives {
            let nearest = representatives
                .iter()
                .filter(|other| other.material_id != representative.material_id)
                .map(|other| (representative.coords - other.coords).length())
                .fold(f64::INFINITY, f64::min);

            let spacing = match nearest.is_finite() {
                true => nearest,
                false => 0.0,
            };
            spacings.insert(representative.material_id, spacing);
        }
    }

    spacings
}

/// A per-axis ordered-dither offset from the 3D Bayer matrix over the first
/// `dimensions` axes, each axis reading a permutation of the position so the
/// axes decorrelate. Each axis spans `spacing / sqrt(dimensions)`, so the
/// offset stays shorter than half of `spacing` and never carries a
/// representative's own voxel to another representative.
fn ordered_offset(position: TyVector3U32, spacing: f64, dimensions: usize) -> TyVector4F64 {
    let scale = spacing / (dimensions as f64).sqrt();
    let level = |raw: u32| ((raw as f64 + 0.5) / BAYER_LEVELS as f64 - 0.5) * scale;
    let (x, y, z) = (position.x, position.y, position.z);

    let mut offset = [
        level(bayer(x, y, z)),
        level(bayer(y, z, x)),
        level(bayer(z, x, y)),
        level(bayer(x, z, y)),
    ];
    for axis in &mut offset[dimensions..] {
        *axis = 0.0;
    }

    TyVector4F64::from_array(offset)
}

/// Side of the 3D Bayer matrix, a power of two for the doubling recurrence.
const BAYER_SIDE: u32 = 4;

/// Distinct threshold levels in the matrix, `BAYER_SIDE^3`.
const BAYER_LEVELS: u32 = BAYER_SIDE * BAYER_SIDE * BAYER_SIDE;

/// The Bayer threshold at `(x, y, z)` in `[0, BAYER_LEVELS)`, tiling by
/// [`BAYER_SIDE`]. One doubling of a parity-ordered 2x2x2 base (the 3D analog
/// of `[[0, 2], [3, 1]]`): `M(p) = 8 * base(p mod 2) + base(p / 2 mod 2)`.
fn bayer(x: u32, y: u32, z: u32) -> u32 {
    // The cube corners permuted 0..8, even-parity before odd so successive
    // thresholds land far apart.
    const BASE: [[[u32; 2]; 2]; 2] = [[[0, 4], [5, 1]], [[6, 2], [3, 7]]];

    let base = |x: u32, y: u32, z: u32| BASE[(x % 2) as usize][(y % 2) as usize][(z % 2) as usize];

    8 * base(x, y, z) + base(x / 2, y / 2, z / 2)
}
