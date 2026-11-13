use crate::{ObjectQuads, ObjectTiles, TileScene};
use std::collections::BTreeMap;
use ty_math::{TyVector3F64, TyVector3U32};
use voxrender::grid_point;
use voxsurface::{SurfaceGrid, SurfaceMethod, mesh_grid_keyed};

/// The cells along each side of a tile.
const TILE: u32 = 16;

/// A box of cells: its low cell and the corner past its high cell.
type CellBox = (TyVector3U32, TyVector3U32);

/// Each object's opaque quads, in the order of `tiles`' objects. A greedy
/// run merges the cells of one material alone, so a quad holds one. An
/// object's coarser copies keep none, because a ray traces the object.
///
/// With `reach`, an object keeps only the quads a mirror ray can meet.
/// `reach` gives the cells a ray from a face of a material runs, or zero
/// for a material that casts none. A ray starts on a cell of such a
/// material and ends within its reach, so a quad farther from every such
/// cell meets no ray. A scene the size of a landscape then keeps the quads
/// beside its glossy cells, where every quad would not fit a headset.
///
/// A box around each tile's cells that cast a ray grows by their longest
/// reach. An object keeps its cells inside such a box of any placement. A
/// run merges within a tile there. An object whose every cell lies in one
/// box merges across its tiles, as it does with no `reach`.
pub fn tile_quads(tiles: &TileScene, reach: Option<&dyn Fn(u32) -> f64>) -> Vec<ObjectQuads> {
    let copies = || tiles.spans.iter().map(|&span| span > 1);

    let Some(reach) = reach else {
        return (tiles.objects.iter().zip(copies()))
            .map(|(object, copy)| {
                let mut quads = ObjectQuads::default();

                if !copy {
                    push_region(&mut quads, object, (TyVector3U32::ZERO, object.bounds));
                }

                quads
            })
            .collect();
    };

    let casts: Vec<_> = (tiles.objects.iter().zip(copies()))
        .map(|(object, copy)| {
            if copy {
                Vec::new()
            } else {
                cast_boxes(object, reach)
            }
        })
        .collect();

    // The world box each placement's rays can end in, for each box of its
    // object's cells that cast one. The margin of a cell covers where a ray
    // starts off its face and the rounding of the corners.
    let mut reached: Vec<(TyVector3F64, TyVector3F64)> = Vec::new();

    for (object, transform) in &tiles.placements {
        let widest = transform.scale.abs().max_element();

        for &((low, high), cells) in &casts[*object] {
            let (least, most) = corners(low.as_dvec3(), high.as_dvec3())
                .map(|corner| transform.transform_point(corner))
                .fold(
                    (
                        TyVector3F64::splat(f64::INFINITY),
                        TyVector3F64::splat(f64::NEG_INFINITY),
                    ),
                    |(least, most), corner| (least.min(corner), most.max(corner)),
                );

            let margin = (cells + 1.0) * widest;

            reached.push((least - margin, most + margin));
        }
    }

    // Sorted along x, so a placement compares only the boxes its span
    // meets.
    reached.sort_by(|a, b| a.0.x.total_cmp(&b.0.x));

    let mut kept: Vec<Vec<CellBox>> = vec![Vec::new(); tiles.objects.len()];

    for (object, transform) in &tiles.placements {
        if tiles.spans[*object] > 1 {
            continue;
        }

        let bounds = tiles.objects[*object].bounds;

        let (least, most) = corners(TyVector3F64::ZERO, bounds.as_dvec3())
            .map(|corner| transform.transform_point(corner))
            .fold(
                (
                    TyVector3F64::splat(f64::INFINITY),
                    TyVector3F64::splat(f64::NEG_INFINITY),
                ),
                |(least, most), corner| (least.min(corner), most.max(corner)),
            );

        let end = reached.partition_point(|(low, _)| low.x < most.x);

        for &(low, high) in &reached[..end] {
            if high.cmple(least).any() || low.cmpge(most).any() {
                continue;
            }

            // The box around the reached box in the placement's cells.
            let (first, last) = corners(low, high)
                .map(|corner| grid_point(transform, corner))
                .fold(
                    (
                        TyVector3F64::splat(f64::INFINITY),
                        TyVector3F64::splat(f64::NEG_INFINITY),
                    ),
                    |(least, most), corner| (least.min(corner), most.max(corner)),
                );

            let first = first.floor().max(TyVector3F64::ZERO).as_uvec3();
            let last = last.ceil().max(TyVector3F64::ZERO).as_uvec3().min(bounds);

            if first.cmplt(last).all() {
                kept[*object].push((first, last));
            }
        }
    }

    tiles
        .objects
        .iter()
        .zip(kept)
        .map(|(object, kept)| {
            let mut quads = ObjectQuads::default();
            let whole = (TyVector3U32::ZERO, object.bounds);

            if kept.contains(&whole) {
                push_region(&mut quads, object, whole);

                return quads;
            }

            // The box around what each tile keeps, in the tiles' order.
            let mut regions: BTreeMap<usize, CellBox> = BTreeMap::new();

            for (low, high) in kept {
                let (first, last) = (low / TILE, (high - 1) / TILE);

                for x in first.x..=last.x {
                    for y in first.y..=last.y {
                        for z in first.z..=last.z {
                            let tile = TyVector3U32::new(x, y, z);

                            let Some(place) = object.tile_at(tile) else {
                                continue;
                            };

                            let part = ((tile * TILE).max(low), ((tile + 1) * TILE).min(high));

                            regions
                                .entry(place)
                                .and_modify(|(least, most)| {
                                    *least = least.min(part.0);
                                    *most = most.max(part.1);
                                })
                                .or_insert(part);
                        }
                    }
                }
            }

            for region in regions.into_values() {
                push_region(&mut quads, object, region);
            }

            quads
        })
        .collect()
}

/// The eight corners of the box from `low` to `high`.
fn corners(low: TyVector3F64, high: TyVector3F64) -> impl Iterator<Item = TyVector3F64> {
    (0..8u32).map(move |corner| {
        let pick = |bit: u32, axis: usize| {
            if corner & bit == 0 {
                low[axis]
            } else {
                high[axis]
            }
        };

        TyVector3F64::new(pick(1, 0), pick(2, 1), pick(4, 2))
    })
}

/// The box around each tile's cells whose material casts a mirror ray,
/// with the longest of those materials' reaches in cells. `reach` gives a
/// material's, or zero where it casts none.
fn cast_boxes(object: &ObjectTiles, reach: &dyn Fn(u32) -> f64) -> Vec<(CellBox, f64)> {
    let masks = &object.masks;
    let mut boxes = Vec::new();

    for (tile, &corner) in masks.corners.iter().enumerate() {
        let (mut low, mut high) = (TyVector3U32::MAX, TyVector3U32::ZERO);
        let mut longest: f64 = 0.0;

        for &index in &masks.recorded[tile] {
            let material = object
                .material_of(tile, index)
                .expect("a record cell holds a material");

            let cells = reach(material);

            if cells > 0.0 {
                let index = u32::from(index);
                let cell = corner
                    + TyVector3U32::new(index / TILE % TILE, index / (TILE * TILE), index % TILE);

                low = low.min(cell);
                high = high.max(cell + 1);
                longest = longest.max(cells);
            }
        }

        if longest > 0.0 {
            boxes.push(((low, high), longest));
        }
    }

    boxes
}

/// Appends the quads of `object`'s opaque cells from `low` to before
/// `high`. A face against an opaque cell outside the box stays hidden.
fn push_region(quads: &mut ObjectQuads, object: &ObjectTiles, (low, high): CellBox) {
    let region = Region {
        object,
        low,
        bounds: high - low,
    };

    let material = |(tile, index): (usize, u16)| {
        object
            .material_of(tile, index)
            .expect("a cell with a face has a record")
    };

    let mesh = mesh_grid_keyed(&region, SurfaceMethod::Greedy, &material, &|_| true);

    let count = |value: usize| u32::try_from(value).expect("an object's quads number under 2^30");
    let first = count(quads.positions.len());

    quads.positions.extend(
        mesh.positions
            .iter()
            .map(|&position| position + low.as_vec3()),
    );
    quads
        .indices
        .extend(mesh.indices.iter().map(|index| first + index));
    quads.faces.extend(
        mesh.spans
            .iter()
            .map(|span| count(span.d) * 2 + u32::from(span.sign > 0)),
    );
    quads
        .materials
        .extend(mesh.face_cells.iter().map(|covered| material(covered[0])));
}

/// A box of an object's cells as a mesher reads it. A cell's handle holds
/// its tile's place among the object's and its index in the tile.
struct Region<'a> {
    object: &'a ObjectTiles,

    low: TyVector3U32,

    bounds: TyVector3U32,
}

impl SurfaceGrid for Region<'_> {
    type Cell = (usize, u16);

    fn bounds(&self) -> TyVector3U32 {
        self.bounds
    }

    fn cell(&self, position: TyVector3U32) -> Option<Self::Cell> {
        self.object
            .opaque_at((self.low + position).to_array().map(i64::from))
    }

    fn is_opaque(&self, _cell: Self::Cell) -> bool {
        true
    }

    fn shares_material(&self, cell: Self::Cell, other: Self::Cell) -> bool {
        self.object.material_of(cell.0, cell.1) == self.object.material_of(other.0, other.1)
    }

    // A cell past the box's edge still hides the face against it.
    fn cell_at(&self, position: [i64; 3]) -> Option<Self::Cell> {
        let low = self.low.to_array();

        self.object
            .opaque_at([0, 1, 2].map(|axis| position[axis] + i64::from(low[axis])))
    }
}

#[cfg(test)]
mod tests {
    use crate::{TileScene, painted_object, place_object, retain_next_object, tile_quads};
    use branded_id::U32Id;
    use ty_math::{TyQuaternionF64, TyVector3F64, TyVector3U32};
    use voxrender::{RenderMaterial, RenderScene};

    /// A floor 64 cells along x and 16 along z, one cell deep, of a matte
    /// material but for the cell at x = 2 and z = 2, which is glossy.
    fn floor() -> TileScene {
        let mut scene = RenderScene::default();
        let matte = scene.retain_material(RenderMaterial::default()).unwrap();

        let glossy = scene
            .retain_material(RenderMaterial {
                roughness: 0.25,
                ..RenderMaterial::default()
            })
            .unwrap();

        let floor = retain_next_object(
            &mut scene,
            painted_object("floor", TyVector3U32::new(64, 1, 16), |cell| {
                Some(if cell.x == 2 && cell.z == 2 {
                    glossy
                } else {
                    matte
                })
            }),
        );

        place_object(
            &mut scene,
            floor,
            TyVector3F64::ZERO,
            TyQuaternionF64::IDENTITY,
            1.0,
        );

        let _: U32Id<_> = glossy;

        TileScene::of(&scene, false, false).unwrap()
    }

    #[test]
    fn a_quad_holds_one_material() {
        let tiles = floor();
        let quads = &tile_quads(&tiles, None)[0];

        assert_eq!(quads.faces.len(), quads.materials.len());
        assert_eq!(quads.indices.len(), quads.faces.len() * 6);

        // The glossy cell's top and bottom each make a quad of their own.
        assert_eq!(quads.materials.iter().filter(|&&id| id == 1).count(), 2);
    }

    #[test]
    fn an_object_keeps_the_quads_a_mirror_ray_can_meet() {
        let tiles = floor();
        let every = tile_quads(&tiles, None);

        // A reach of 4 cells with a cell's margin keeps the cells up to
        // x = 7 and z = 7, all in the first tile.
        let near = tile_quads(&tiles, Some(&|material| f64::from(material) * 4.0));
        let reach = |quads: &crate::ObjectQuads| {
            quads
                .positions
                .iter()
                .fold(0.0f32, |most, position| most.max(position.x))
        };

        assert_eq!(reach(&every[0]), 64.0);
        assert_eq!(reach(&near[0]), 8.0);

        // No material casts a ray, so no quad stays.
        assert!(tile_quads(&tiles, Some(&|_| 0.0))[0].faces.is_empty());

        // A reach past the whole floor keeps every quad.
        let far = tile_quads(&tiles, Some(&|material| f64::from(material) * 100.0));

        assert_eq!(far[0].faces.len(), every[0].faces.len());
    }
}
