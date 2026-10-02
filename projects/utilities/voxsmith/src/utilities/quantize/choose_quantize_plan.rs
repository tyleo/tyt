use crate::{
    Error, Result,
    utilities::{
        AlphaMode, ColorSpace, PartitionProperties, PropertyInterpretation, QuantizeOptions,
        QuantizePlan, QuantizePoint, ReductionMethod, kmeans, median_cut, octree,
    },
};
use branded_id::U32Id;
use std::{collections::HashMap, fmt::Debug, result::Result as StdResult};
use ty_math::{
    FromColor, TyCielabColorF64, TyColorToVector3, TyLinSrgbF64, TyOklabColorF64, TySrgbF64,
    TyVector3F64, TyVector4F64,
};
use voxcore::{
    BVoxLayer, BVoxMaterial, BVoxObject, BVoxPalette, BVoxProperty, BVoxValuePoolValue, VoxExt,
    VoxMain, VoxPalette, VoxValueColumn, VoxValuePool, VoxValuePoolValues,
    material::{BASE_COLOR, EMISSIVE_COLOR},
};

/// Chooses representatives for the materials of `palette_id` that `layers`
/// sample, each weighted by its voxel count, or `None` when they sample at most
/// `options.max_materials`.
pub fn choose_quantize_plan<T: VoxExt>(
    main: &VoxMain<T>,
    palette_id: U32Id<BVoxPalette>,
    layers: &[(U32Id<BVoxObject>, U32Id<BVoxLayer>)],
    options: &QuantizeOptions,
) -> Result<Option<QuantizePlan>> {
    let palette = main
        .palette(palette_id)
        .expect("the quantized palette is one of the main's");
    let palette_index = main
        .iter_palettes()
        .position(|(listed_palette_id, _)| listed_palette_id == palette_id)
        .expect("the quantized palette is one of the main's");

    let name = &options.property;
    let property_id = palette.property_id_by_name(name).ok_or_else(|| {
        Error::invalid(format!("palette {palette_index} has no property `{name}`"))
    })?;
    let value_pool = property_value_pool(main, palette, property_id);

    let (reading, dimensions) = Reading::resolve(options, value_pool)?;
    let partition_properties =
        partition_properties(main, palette, palette_index, property_id, options)?;

    let populations = populations(main, palette_id, layers);
    let max_materials = options.max_materials.get();
    if populations.len() <= max_materials {
        return Ok(None);
    }

    // The sampled materials in listing order, so the clustering is
    // deterministic.
    let candidates: Vec<_> = palette
        .iter_materials()
        .enumerate()
        .filter_map(|(material_index, material_id)| {
            let &population = populations.get(&material_id)?;
            Some((material_index, material_id, population))
        })
        .collect();

    let value_ids: Vec<_> = candidates
        .iter()
        .map(|&(_, material_id, _)| material_value_id(palette, material_id, property_id))
        .collect();

    let points = reading
        .points(value_pool, &value_ids)
        .map_err(|(index, value)| {
            let (material_index, _, _) = candidates[index];
            Error::invalid(format!(
                "material {material_index} of palette {palette_index} holds {value} for \
                 `{name}`, which reads as no finite point"
            ))
        })?;

    // Candidates grouped into partitions in order of first appearance.
    let mut partitions: Vec<(PartitionKey, Vec<QuantizePoint>)> = Vec::new();
    let mut partition_indices = HashMap::new();

    for ((_, material_id, population), (coords, alpha)) in candidates.into_iter().zip(points) {
        let key = PartitionKey {
            alpha,
            value_ids: partition_properties
                .iter()
                .map(|(partition_property_id, first_equal_ids)| {
                    first_equal_ids
                        [&material_value_id(palette, material_id, *partition_property_id)]
                })
                .collect(),
        };

        let point = QuantizePoint {
            material_id,
            coords,
            population,
        };

        let partition_index = match partitions.iter().position(|(other, _)| *other == key) {
            Some(partition_index) => partition_index,

            None => {
                partitions.push((key, Vec::new()));
                partitions.len() - 1
            }
        };
        partitions[partition_index].1.push(point);
        partition_indices.insert(material_id, partition_index);
    }

    if partitions.len() > max_materials {
        return Err(Error::invalid(format!(
            "the partitions split palette {palette_index}'s sampled materials into {} groups, \
             more than the {max_materials} material(s) allowed",
            partitions.len()
        )));
    }

    let partitions: Vec<Vec<QuantizePoint>> =
        partitions.into_iter().map(|(_, points)| points).collect();
    let partition_count = partitions.len();

    let clusters = match options.method {
        ReductionMethod::Kmeans => cluster_each_partition(partitions, max_materials, kmeans),
        ReductionMethod::MedianCut => median_cut(partitions, max_materials),
        ReductionMethod::Octree => cluster_each_partition(partitions, max_materials, octree),
    };

    let mut plan = QuantizePlan {
        representative_ids: HashMap::new(),
        points: HashMap::new(),
        partition_representatives: vec![Vec::new(); partition_count],
        dimensions,
    };

    for cluster in clusters {
        let representative = representative_point(&cluster);
        let partition_index = partition_indices[&representative.material_id];
        plan.partition_representatives[partition_index].push(representative);

        for point in cluster {
            plan.representative_ids
                .insert(point.material_id, representative.material_id);
            plan.points
                .insert(point.material_id, (point, partition_index));
        }
    }

    Ok(Some(plan))
}

/// How a property's values read as clustering points, resolved against its
/// value pool's kind.
#[derive(Clone, Copy)]
enum Reading {
    /// A three-component color stored in `encoding` and measured in `space`.
    Rgb {
        encoding: ColorEncoding,

        space: ColorSpace,
    },

    /// A four-component color stored in `encoding` and measured in `space`.
    /// `alpha` sets how its alpha takes part.
    Rgba {
        encoding: ColorEncoding,

        space: ColorSpace,

        alpha: AlphaMode,
    },

    /// Raw components.
    Numeric,
}

/// Whether a color reading's components are linear light or sRGB-encoded.
#[derive(Clone, Copy)]
enum ColorEncoding {
    Linear,

    Srgb,
}

/// A clustering point, plus the alpha when alpha partitions.
type Point = (TyVector4F64, Option<f64>);

impl Reading {
    /// Resolves `options`'s reading of a property drawing from `value_pool`,
    /// with the number of axes its points use.
    fn resolve(options: &QuantizeOptions, value_pool: &VoxValuePool) -> Result<(Self, usize)> {
        let name = &options.property;
        let values = value_pool.values();
        let kind_name = kind_name(values);

        let interpretation = match options.interpret_property {
            PropertyInterpretation::Auto if name == BASE_COLOR || name == EMISSIVE_COLOR => {
                PropertyInterpretation::LinearColor
            }

            PropertyInterpretation::Auto => PropertyInterpretation::Numeric,

            interpretation => interpretation,
        };

        let (reading, dimensions) = match interpretation {
            PropertyInterpretation::LinearColor | PropertyInterpretation::SrgbColor => {
                let encoding = match interpretation {
                    PropertyInterpretation::SrgbColor => ColorEncoding::Srgb,
                    _ => ColorEncoding::Linear,
                };

                let space = options.space.unwrap_or(ColorSpace::Oklab);

                match (values, options.alpha) {
                    (VoxValuePoolValues::Vec3Float(_), None) => {
                        (Reading::Rgb { encoding, space }, 3)
                    }

                    (VoxValuePoolValues::Vec3Float(_), Some(_)) => {
                        return Err(Error::invalid(format!(
                            "property `{name}` reads as a 3-component color, which has no alpha \
                             to take part"
                        )));
                    }

                    (VoxValuePoolValues::Vec4Float(_), alpha) => {
                        let alpha = alpha.unwrap_or(AlphaMode::Partition);
                        let dimensions = match alpha {
                            AlphaMode::Distance => 4,
                            AlphaMode::Ignore | AlphaMode::Partition => 3,
                        };

                        let reading = Reading::Rgba {
                            encoding,
                            space,
                            alpha,
                        };

                        (reading, dimensions)
                    }

                    _ => {
                        return Err(Error::invalid(format!(
                            "property `{name}` holds {kind_name} values, but a color reading \
                             needs vec-3-float or vec-4-float"
                        )));
                    }
                }
            }

            PropertyInterpretation::Numeric => {
                let dimensions = match values {
                    VoxValuePoolValues::Float(_) | VoxValuePoolValues::Int(_) => 1,

                    VoxValuePoolValues::Vec2Float(_) | VoxValuePoolValues::Vec2Int(_) => 2,

                    VoxValuePoolValues::Vec3Float(_) | VoxValuePoolValues::Vec3Int(_) => 3,

                    VoxValuePoolValues::Vec4Float(_) | VoxValuePoolValues::Vec4Int(_) => 4,

                    VoxValuePoolValues::Bool(_)
                    | VoxValuePoolValues::Json(_)
                    | VoxValuePoolValues::String(_) => {
                        return Err(Error::invalid(format!(
                            "property `{name}` holds {kind_name} values, which read as no points"
                        )));
                    }
                };

                if options.alpha.is_some() {
                    return Err(Error::invalid(format!(
                        "property `{name}` reads as numeric, which has no alpha to take part"
                    )));
                }

                if options.space.is_some() {
                    return Err(Error::invalid(format!(
                        "property `{name}` reads as numeric, which a color space does not apply to"
                    )));
                }

                (Reading::Numeric, dimensions)
            }

            PropertyInterpretation::Auto => unreachable!("auto resolved above"),
        };

        if options.method == ReductionMethod::Octree && dimensions == 4 {
            return Err(Error::invalid(format!(
                "property `{name}` reads as 4D points, but octree clusters 3D points"
            )));
        }

        Ok((reading, dimensions))
    }

    /// The point of each value `value_ids` draw from `value_pool`, which must be
    /// the value pool the reading resolved against. Errors with the index into
    /// `value_ids` and the value's spelling on the first point with a
    /// coordinate that is not finite.
    fn points(
        self,
        value_pool: &VoxValuePool,
        value_ids: &[U32Id<BVoxValuePoolValue>],
    ) -> StdResult<Vec<Point>, (usize, String)> {
        match (self, value_pool.values()) {
            (Reading::Rgb { encoding, space }, VoxValuePoolValues::Vec3Float(colors)) => {
                let coords = color_coords(encoding, space);
                finite_points(colors, value_ids, |&rgb| (coords(rgb).extend(0.0), None))
            }

            (
                Reading::Rgba {
                    encoding,
                    space,
                    alpha: AlphaMode::Distance,
                },
                VoxValuePoolValues::Vec4Float(colors),
            ) => {
                let coords = color_coords(encoding, space);
                let scale = alpha_scale(space);
                finite_points(colors, value_ids, |&[red, green, blue, alpha]| {
                    (coords([red, green, blue]).extend(alpha * scale), None)
                })
            }

            (
                Reading::Rgba {
                    encoding,
                    space,
                    alpha: AlphaMode::Partition,
                },
                VoxValuePoolValues::Vec4Float(colors),
            ) => {
                let coords = color_coords(encoding, space);
                finite_points(colors, value_ids, |&[red, green, blue, alpha]| {
                    (coords([red, green, blue]).extend(0.0), Some(alpha))
                })
            }

            (
                Reading::Rgba {
                    encoding,
                    space,
                    alpha: AlphaMode::Ignore,
                },
                VoxValuePoolValues::Vec4Float(colors),
            ) => {
                let coords = color_coords(encoding, space);
                finite_points(colors, value_ids, |&[red, green, blue, _]| {
                    (coords([red, green, blue]).extend(0.0), None)
                })
            }

            (Reading::Numeric, VoxValuePoolValues::Float(numbers)) => {
                finite_points(numbers, value_ids, |&number| (padded([number]), None))
            }

            (Reading::Numeric, VoxValuePoolValues::Int(numbers)) => {
                finite_points(numbers, value_ids, |&number| {
                    (padded([number as f64]), None)
                })
            }

            (Reading::Numeric, VoxValuePoolValues::Vec2Float(vectors)) => {
                finite_points(vectors, value_ids, |&vector| (padded(vector), None))
            }

            (Reading::Numeric, VoxValuePoolValues::Vec3Float(vectors)) => {
                finite_points(vectors, value_ids, |&vector| (padded(vector), None))
            }

            (Reading::Numeric, VoxValuePoolValues::Vec4Float(vectors)) => {
                finite_points(vectors, value_ids, |&vector| (padded(vector), None))
            }

            (Reading::Numeric, VoxValuePoolValues::Vec2Int(vectors)) => {
                finite_points(vectors, value_ids, |vector| {
                    (padded(vector.map(|number| number as f64)), None)
                })
            }

            (Reading::Numeric, VoxValuePoolValues::Vec3Int(vectors)) => {
                finite_points(vectors, value_ids, |vector| {
                    (padded(vector.map(|number| number as f64)), None)
                })
            }

            (Reading::Numeric, VoxValuePoolValues::Vec4Int(vectors)) => {
                finite_points(vectors, value_ids, |vector| {
                    (padded(vector.map(|number| number as f64)), None)
                })
            }

            _ => unreachable!("a reading resolves only against a kind it reads"),
        }
    }
}

/// The point `point` makes of each value `value_ids` draw from `values`.
/// Errors with the index into `value_ids` and the value's spelling on the
/// first point with a coordinate that is not finite.
fn finite_points<T: Debug>(
    values: VoxValueColumn<'_, T>,
    value_ids: &[U32Id<BVoxValuePoolValue>],
    point: impl Fn(&T) -> Point,
) -> StdResult<Vec<Point>, (usize, String)> {
    let mut points = Vec::with_capacity(value_ids.len());

    for (index, &value_id) in value_ids.iter().enumerate() {
        let value = values
            .get(value_id)
            .expect("a material's value is one of its value pool's");

        let (coords, alpha) = point(value);
        if !coords.is_finite() {
            return Err((index, format!("{value:?}")));
        }

        points.push((coords, alpha));
    }

    Ok(points)
}

/// `components` as a point with every later axis zero.
fn padded<const N: usize>(components: [f64; N]) -> TyVector4F64 {
    let mut padded = [0.0; 4];
    padded[..N].copy_from_slice(&components);
    TyVector4F64::from_array(padded)
}

/// The map from a color stored in `encoding` to a point in `space`.
fn color_coords(encoding: ColorEncoding, space: ColorSpace) -> fn([f64; 3]) -> TyVector3F64 {
    match (encoding, space) {
        (ColorEncoding::Linear, ColorSpace::Lab) => |[red, green, blue]| {
            TyCielabColorF64::from_color(TyLinSrgbF64::new(red, green, blue)).to_vector3()
        },

        (ColorEncoding::Srgb, ColorSpace::Lab) => |[red, green, blue]| {
            TyCielabColorF64::from_color(TySrgbF64::new(red, green, blue).into_linear())
                .to_vector3()
        },

        (ColorEncoding::Linear, ColorSpace::Oklab) => |[red, green, blue]| {
            TyOklabColorF64::from_color(TyLinSrgbF64::new(red, green, blue)).to_vector3()
        },

        (ColorEncoding::Srgb, ColorSpace::Oklab) => |[red, green, blue]| {
            TyOklabColorF64::from_color(TySrgbF64::new(red, green, blue).into_linear()).to_vector3()
        },

        (ColorEncoding::Linear, ColorSpace::Srgb) => |[red, green, blue]| {
            TySrgbF64::from_linear(TyLinSrgbF64::new(red, green, blue)).to_vector3()
        },

        (ColorEncoding::Srgb, ColorSpace::Srgb) => {
            |[red, green, blue]| TyVector3F64::new(red, green, blue)
        }
    }
}

/// The factor bringing alpha's `[0, 1]` to the span of `space`'s lightness
/// axis, so a distance reading weighs alpha like lightness.
fn alpha_scale(space: ColorSpace) -> f64 {
    match space {
        ColorSpace::Lab => 100.0,
        ColorSpace::Oklab | ColorSpace::Srgb => 1.0,
    }
}

/// The partition key of one material: the values it has to share with another
/// material to merge. Each value is the first value id in its value pool
/// holding an equal value.
#[derive(PartialEq)]
struct PartitionKey {
    alpha: Option<f64>,

    value_ids: Vec<U32Id<BVoxValuePoolValue>>,
}

/// The partition properties `options` picks from `palette`, each with the
/// [`FirstEqualIds`] of the value pool it draws from.
fn partition_properties<T: VoxExt>(
    main: &VoxMain<T>,
    palette: &VoxPalette,
    palette_index: usize,
    property_id: U32Id<BVoxProperty>,
    options: &QuantizeOptions,
) -> Result<Vec<(U32Id<BVoxProperty>, FirstEqualIds)>> {
    let partition_property_ids: Vec<_> = match &options.partition {
        PartitionProperties::All => palette
            .iter_properties()
            .map(|(partition_property_id, _)| partition_property_id)
            .filter(|&partition_property_id| partition_property_id != property_id)
            .collect(),

        PartitionProperties::Named(names) => names
            .iter()
            .map(|partition_name| {
                if *partition_name == options.property {
                    return Err(Error::invalid(format!(
                        "property `{partition_name}` is the quantized property, so it cannot \
                         also partition"
                    )));
                }

                palette.property_id_by_name(partition_name).ok_or_else(|| {
                    Error::invalid(format!(
                        "palette {palette_index} has no property `{partition_name}` to \
                         partition on"
                    ))
                })
            })
            .collect::<Result<_>>()?,
    };

    Ok(partition_property_ids
        .into_iter()
        .map(|partition_property_id| {
            (
                partition_property_id,
                first_equal_ids(property_value_pool(main, palette, partition_property_id)),
            )
        })
        .collect())
}

/// How many live voxels of `layers` sample each material of `palette_id`.
fn populations<T: VoxExt>(
    main: &VoxMain<T>,
    palette_id: U32Id<BVoxPalette>,
    layers: &[(U32Id<BVoxObject>, U32Id<BVoxLayer>)],
) -> HashMap<U32Id<BVoxMaterial>, u64> {
    let mut populations = HashMap::new();

    for &(object_id, layer_id) in layers {
        let object = main
            .object(object_id)
            .expect("a quantized layer's object is one of the main's");
        assert_eq!(
            object.layer_palette_id(layer_id),
            Some(palette_id),
            "a quantized layer references the quantized palette"
        );

        let samples = object
            .iter_live_samples(layer_id)
            .expect("a quantized layer is one of its object's");

        for (_, material_id) in samples {
            *populations.entry(material_id).or_insert(0) += 1;
        }
    }

    populations
}

/// Clusters each partition apart with `cluster`, into the slots
/// [`allocate_slots`] gives it.
fn cluster_each_partition(
    partitions: Vec<Vec<QuantizePoint>>,
    max_materials: usize,
    cluster: fn(Vec<QuantizePoint>, usize) -> Vec<Vec<QuantizePoint>>,
) -> Vec<Vec<QuantizePoint>> {
    let slots = allocate_slots(&partitions, max_materials);
    partitions
        .into_iter()
        .zip(slots)
        .flat_map(|(points, slot_count)| cluster(points, slot_count))
        .collect()
}

/// Hands each partition one slot, then the rest of `max_materials` one at a
/// time to the partition with the most voxels per slot, ties to the lowest
/// index. A partition takes at most one slot per point.
fn allocate_slots(partitions: &[Vec<QuantizePoint>], max_materials: usize) -> Vec<usize> {
    let populations: Vec<u128> = partitions
        .iter()
        .map(|points| points.iter().map(|point| point.population as u128).sum())
        .collect();

    let mut slots = vec![1usize; partitions.len()];

    for _ in partitions.len()..max_materials {
        let next = (0..partitions.len())
            .filter(|&index| slots[index] < partitions[index].len())
            .max_by(|&a, &b| {
                (populations[a] * slots[b] as u128)
                    .cmp(&(populations[b] * slots[a] as u128))
                    .then_with(|| b.cmp(&a))
            });

        let Some(index) = next else {
            break;
        };

        slots[index] += 1;
    }

    slots
}

/// The value pool `property_id` of `palette` draws from.
fn property_value_pool<'a, T: VoxExt>(
    main: &'a VoxMain<T>,
    palette: &VoxPalette,
    property_id: U32Id<BVoxProperty>,
) -> &'a VoxValuePool {
    let value_pool_id = palette
        .property(property_id)
        .expect("the property is one of the palette's")
        .value_pool_id;
    main.value_pool(value_pool_id)
        .expect("a property draws from a live value pool")
}

/// The value id `material_id` draws for `property_id`.
fn material_value_id(
    palette: &VoxPalette,
    material_id: U32Id<BVoxMaterial>,
    property_id: U32Id<BVoxProperty>,
) -> U32Id<BVoxValuePoolValue> {
    palette
        .value_id(material_id, property_id)
        .expect("a live material holds a value for every property")
}

/// Each value id of a value pool mapped to the first id in listing order
/// holding an equal value.
type FirstEqualIds = HashMap<U32Id<BVoxValuePoolValue>, U32Id<BVoxValuePoolValue>>;

fn first_equal_ids(value_pool: &VoxValuePool) -> FirstEqualIds {
    match value_pool.values() {
        VoxValuePoolValues::Bool(values) => first_equal_column_ids(values),
        VoxValuePoolValues::Float(values) => first_equal_column_ids(values),
        VoxValuePoolValues::Int(values) => first_equal_column_ids(values),
        VoxValuePoolValues::Json(values) => first_equal_column_ids(values),
        VoxValuePoolValues::String(values) => first_equal_column_ids(values),
        VoxValuePoolValues::Vec2Float(values) => first_equal_column_ids(values),
        VoxValuePoolValues::Vec2Int(values) => first_equal_column_ids(values),
        VoxValuePoolValues::Vec3Float(values) => first_equal_column_ids(values),
        VoxValuePoolValues::Vec3Int(values) => first_equal_column_ids(values),
        VoxValuePoolValues::Vec4Float(values) => first_equal_column_ids(values),
        VoxValuePoolValues::Vec4Int(values) => first_equal_column_ids(values),
    }
}

fn first_equal_column_ids<T: PartialEq>(values: VoxValueColumn<'_, T>) -> FirstEqualIds {
    let mut firsts: Vec<(U32Id<BVoxValuePoolValue>, &T)> = Vec::new();
    let mut ids = HashMap::new();

    for (value_id, value) in values.iter() {
        let first_id = match firsts.iter().find(|(_, first)| *first == value) {
            Some(&(first_id, _)) => first_id,

            None => {
                firsts.push((value_id, value));
                value_id
            }
        };

        ids.insert(value_id, first_id);
    }

    ids
}

/// The voxj type name of `values`'s kind.
fn kind_name(values: VoxValuePoolValues<'_>) -> &'static str {
    match values {
        VoxValuePoolValues::Bool(_) => "bool",
        VoxValuePoolValues::Float(_) => "float",
        VoxValuePoolValues::Int(_) => "int",
        VoxValuePoolValues::Json(_) => "json",
        VoxValuePoolValues::String(_) => "string",
        VoxValuePoolValues::Vec2Float(_) => "vec-2-float",
        VoxValuePoolValues::Vec2Int(_) => "vec-2-int",
        VoxValuePoolValues::Vec3Float(_) => "vec-3-float",
        VoxValuePoolValues::Vec3Int(_) => "vec-3-int",
        VoxValuePoolValues::Vec4Float(_) => "vec-4-float",
        VoxValuePoolValues::Vec4Int(_) => "vec-4-int",
    }
}

/// A cluster's representative: its most-sampled point, ties to the lowest
/// material id.
fn representative_point(cluster: &[QuantizePoint]) -> QuantizePoint {
    cluster
        .iter()
        .copied()
        .max_by(|a, b| {
            a.population
                .cmp(&b.population)
                .then_with(|| b.material_id.to_u32().cmp(&a.material_id.to_u32()))
        })
        .expect("a cluster holds at least one point")
}

#[cfg(test)]
mod tests {
    use crate::utilities::{QuantizePoint, quantize::choose_quantize_plan::allocate_slots};
    use branded_id::U32Id;
    use ty_math::TyVector4F64;

    /// A partition of one point per population.
    fn partition(populations: &[u64]) -> Vec<QuantizePoint> {
        populations
            .iter()
            .enumerate()
            .map(|(index, &population)| QuantizePoint {
                material_id: U32Id::from_u32(index as u32),
                coords: TyVector4F64::ZERO,
                population,
            })
            .collect()
    }

    #[test]
    fn slots_follow_voxel_counts_after_one_each() {
        let partitions = [partition(&[30, 30, 30]), partition(&[5, 5])];
        assert_eq!(allocate_slots(&partitions, 4), [3, 1]);
    }

    #[test]
    fn a_partition_takes_at_most_one_slot_per_point() {
        let partitions = [partition(&[100]), partition(&[1, 1, 1])];
        assert_eq!(allocate_slots(&partitions, 3), [1, 2]);
        assert_eq!(allocate_slots(&partitions, 10), [1, 3]);
    }

    #[test]
    fn ties_go_to_the_lower_partition() {
        let partitions = [partition(&[5, 5]), partition(&[5, 5])];
        assert_eq!(allocate_slots(&partitions, 3), [2, 1]);
    }
}
