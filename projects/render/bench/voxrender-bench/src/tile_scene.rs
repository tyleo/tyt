use crate::{
    BTileObject, BTilePlacement, CellGrid, ObjectTiles, Result, TilePlacement, object_copies,
    to_word,
};
use branded_id::{IdVec, U32Id, UsizeId};
use std::collections::HashMap;
use ty_math::{TyLinSrgbF64, TyQuaternionF64, TyTransformF64};
use voxcore::{BVoxObject, material};
use voxrender::{RenderLight, RenderMaterial, RenderScene};

/// A scene as a [`BoxView`](crate::BoxView) reads it, with each object's
/// cells by tile. A scene takes its objects one at a time, so a caller
/// holds one object's grid while the scene keeps the tiles of the rest.
///
/// A scene can keep coarser copies of each object for a view to draw where
/// the object's cells would span under a pixel. Each copy halves the cells
/// of the one before along each axis, down to a copy that fits one tile. A
/// copy's cells sit on the lattice of its size that the scene's objects
/// share, so two objects that meet on that lattice meet at every copy.
pub struct TileScene {
    pub(crate) materials: Vec<RenderMaterial>,

    /// Whether each material passes no light.
    pub(crate) opaque: Vec<bool>,

    pub(crate) lights: Vec<RenderLight>,

    /// Each object's tiles. An object's coarser copies follow it, finest
    /// first.
    pub(crate) objects: IdVec<BTileObject, ObjectTiles>,

    /// Each placement of one of [`objects`](Self::objects). A placement of
    /// each of the object's copies follows it, finest first.
    pub(crate) placements: IdVec<BTilePlacement, TilePlacement>,

    /// The materials that pass light, in the order a glass cell's word
    /// counts them, for a scene that keeps glass.
    pub(crate) glass_materials: Vec<u32>,

    /// Whether an object keeps its surfaces between materials that pass
    /// light.
    pub(crate) glass: bool,

    /// Whether an object keeps coarser copies.
    pub(crate) coarser: bool,

    /// Whether every material counts as opaque.
    pub(crate) solid: bool,
}

impl TileScene {
    /// A scene with no light and no object yet.
    ///
    /// # Arguments
    ///
    /// 1. `glass`: whether an object keeps its surfaces between materials
    ///    that pass light, which a view draws on a device that blends two
    ///    colors a fragment
    /// 2. `coarser`: whether an object keeps coarser copies, which a view
    ///    draws where it is told how few pixels a cell can span
    pub fn new(glass: bool, coarser: bool) -> Self {
        TileScene {
            materials: Vec::new(),
            opaque: Vec::new(),
            lights: Vec::new(),
            objects: IdVec::new(),
            placements: IdVec::new(),
            glass_materials: Vec::new(),
            glass,
            coarser,
            solid: false,
        }
    }

    /// `scene` as a tile scene, with `glass` and `coarser` as
    /// [`new`](Self::new) takes them. Errors as
    /// [`push_object`](Self::push_object) does.
    pub fn of(scene: &RenderScene, glass: bool, coarser: bool) -> Result<Self> {
        let mut tiles = TileScene::new(glass, coarser);

        tiles.set_lights(scene.iter_lights().map(|(_, light)| *light).collect());
        tiles.push_scene(scene)?;

        Ok(tiles)
    }

    /// Sets the lights the scene's cells shade under.
    pub fn set_lights(&mut self, lights: Vec<RenderLight>) {
        self.lights = lights;
    }

    /// Adds `scene`'s materials, objects, and placements, each in `scene`'s
    /// order. Errors as [`push_object`](Self::push_object) does.
    pub fn push_scene(&mut self, scene: &RenderScene) -> Result<()> {
        let places: HashMap<u32, u32> = scene
            .iter_materials()
            .map(|(id, material)| (id.to_u32(), self.place_material(material)))
            .collect();

        let mut objects = HashMap::new();

        for (object_id, object) in scene.iter_objects() {
            let grid = CellGrid::new(object, &|id| places[&id]);

            let first = Self::first_cell(scene, object_id);

            objects.insert(object_id, self.push_object(&grid, first)?);
        }

        for (_, placement) in scene.iter_placements() {
            self.place_object(objects[&placement.object_id], placement.transform);
        }

        Ok(())
    }

    /// Where the first placement of `scene`'s object `object_id` sets the
    /// object's first cell, in cells, for a placement that keeps the cells
    /// whole and square to the world's axes. Zeros for any other, and for
    /// an object with no placement.
    pub(crate) fn first_cell(scene: &RenderScene, object_id: U32Id<BVoxObject>) -> [i64; 3] {
        scene
            .iter_placements()
            .find(|(_, placement)| placement.object_id == object_id)
            .and_then(|(_, placement)| {
                let transform = &placement.transform;
                let cells = transform.position / transform.scale;

                let whole = transform
                    .rotation
                    .abs_diff_eq(TyQuaternionF64::IDENTITY, 1.0e-9)
                    && cells.abs_diff_eq(cells.round(), 1.0e-6);

                whole.then(|| cells.round().to_array().map(|cell| cell as i64))
            })
            .unwrap_or([0; 3])
    }

    /// The place among the scene's materials of one equal to `material`,
    /// which joins them if none is.
    pub fn place_material(&mut self, material: &RenderMaterial) -> u32 {
        let place = self
            .materials
            .iter()
            .position(|held| held == material)
            .unwrap_or_else(|| {
                let passed = material::pass(
                    material.base_color,
                    material.metallic,
                    material.transmission,
                    material.ior,
                );

                let opaque = self.solid || passed == TyLinSrgbF64::new(0.0, 0.0, 0.0);
                let place = self.materials.len();

                self.materials.push(*material);
                self.opaque.push(opaque);

                // A glass cell's word reaches its material in 12 bits,
                // seven values a material.
                if self.glass && !opaque {
                    assert!(
                        self.glass_materials.len() < 585,
                        "a scene's glass materials number under 586"
                    );

                    self.glass_materials.push(place as u32);
                }

                place
            });

        to_word(place)
    }

    /// Adds the object of `grid`, whose cells hold places among the scene's
    /// materials, with its coarser copies if the scene keeps them. Returns
    /// the object's id. `first` says where the object's first cell sits on
    /// the lattice the scene's objects share, in cells, or holds zeros for
    /// an object off any lattice. Errors if a tile holds over 256 materials,
    /// or if a material that passes light has a place over 65534.
    pub fn push_object(
        &mut self,
        grid: &CellGrid,
        first: [i64; 3],
    ) -> Result<UsizeId<BTileObject>> {
        let glass = self.glass.then_some(self.glass_materials.as_slice());
        let copies = object_copies(grid, first, self.coarser, &self.opaque, glass)?;

        Ok(self.push_copies(copies))
    }

    /// Adds an object as [`object_copies`](object_copies()) gives its tiles,
    /// built for this scene's materials. Returns the object's id.
    pub(crate) fn push_copies(&mut self, copies: Vec<ObjectTiles>) -> UsizeId<BTileObject> {
        let object = self.objects.end();

        self.objects.as_mut_vec().extend(copies);

        object
    }

    /// Places the object [`push_object`](Self::push_object) gave `object`
    /// for by `transform`, which carries its cells to the world, and with
    /// it each of its copies.
    pub fn place_object(&mut self, object: UsizeId<BTileObject>, transform: TyTransformF64) {
        let spans = |copy: UsizeId<BTileObject>| self.objects.get(copy).map(|tiles| tiles.span);

        assert!(spans(object) == Some(1), "the object is one the scene took");

        // The grid's first cell sits ahead of the object's.
        let position = transform.transform_point(-self.objects[object].shift.as_dvec3());

        // The object's copies follow it, up to the next object.
        let copies = (1..)
            .map(|after| object.offset(after))
            .take_while(|&copy| spans(copy).is_some_and(|span| span > 1));

        for copy in [object].into_iter().chain(copies) {
            let scale = transform.scale * f64::from(self.objects[copy].span);

            self.placements.push(TilePlacement {
                object: copy,
                transform: TyTransformF64::new(position, transform.rotation, scale),
            });
        }
    }

    /// The placement that draws the copy a level coarser of what the
    /// placement at `placement` draws, if the scene keeps one.
    pub(crate) fn coarser(
        &self,
        placement: UsizeId<BTilePlacement>,
    ) -> Option<UsizeId<BTilePlacement>> {
        let span = |placement: UsizeId<BTilePlacement>| {
            Some(self.objects[self.placements.get(placement)?.object].span)
        };
        let next = placement.offset(1);

        (span(next) > span(placement)).then_some(next)
    }
}

#[cfg(test)]
mod tests {
    use crate::{TileScene, bench_scenes};

    #[test]
    fn a_scene_keeps_each_placement_in_order_with_its_objects_tiles() {
        for (name, scene) in bench_scenes() {
            let tiles = TileScene::of(&scene, true, false).unwrap();

            assert_eq!(tiles.objects.len(), scene.object_count(), "{name}");
            assert_eq!(tiles.materials.len(), tiles.opaque.len(), "{name}");
            assert_eq!(tiles.placements.len(), scene.placement_count(), "{name}");

            for (placed, (_, placement)) in tiles.placements.iter().zip(scene.iter_placements()) {
                let bounds = scene.object(placement.object_id).unwrap().bounds();

                assert_eq!(tiles.objects[placed.object].bounds, bounds, "{name}");
                assert_eq!(placed.transform, placement.transform, "{name}");
            }
        }
    }
}
