use crate::{CellGrid, ObjectTiles, Result};
use std::collections::HashMap;
use ty_math::{TyLinSrgbF64, TyTransformF64};
use voxcore::material;
use voxrender::{RenderLight, RenderMaterial, RenderScene};

/// A scene as a [`BoxView`](crate::BoxView) reads it: its materials, its
/// lights, and its placements, with each object's cells by tile. A scene
/// takes its objects one at a time, so a caller holds one object's grid
/// while the scene keeps the tiles of the rest.
pub struct TileScene {
    pub(crate) materials: Vec<RenderMaterial>,

    /// Whether each material passes no light.
    pub(crate) opaque: Vec<bool>,

    pub(crate) lights: Vec<RenderLight>,

    pub(crate) objects: Vec<ObjectTiles>,

    /// Each placement's object among [`objects`](Self::objects) and its
    /// transform from the object's cells to the world.
    pub(crate) placements: Vec<(usize, TyTransformF64)>,

    /// The materials that pass light, in the order a glass cell's word
    /// counts them.
    pub(crate) glass_materials: Vec<u32>,

    /// Whether an object keeps its surfaces between materials that pass
    /// light.
    glass: bool,

    /// Whether every material counts as opaque.
    pub(crate) solid: bool,
}

impl TileScene {
    /// A scene with no light and no object yet. With `glass`, an object
    /// keeps its surfaces between materials that pass light, which a view
    /// draws on a device that blends two colors a fragment.
    pub fn new(glass: bool) -> Self {
        TileScene {
            materials: Vec::new(),
            opaque: Vec::new(),
            lights: Vec::new(),
            objects: Vec::new(),
            placements: Vec::new(),
            glass_materials: Vec::new(),
            glass,
            solid: false,
        }
    }

    /// The scene of `scene`'s materials, lights, objects, and placements,
    /// each in `scene`'s order. Errors as [`push_object`](Self::push_object)
    /// does.
    pub fn of(scene: &RenderScene, glass: bool) -> Result<Self> {
        let mut tiles = TileScene::new(glass);

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

            objects.insert(object_id, self.push_object(&grid)?);
        }

        for (_, placement) in scene.iter_placements() {
            self.place_object(objects[&placement.object_id], placement.transform);
        }

        Ok(())
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

                self.materials.push(*material);
                self.opaque
                    .push(self.solid || passed == TyLinSrgbF64::new(0.0, 0.0, 0.0));

                self.materials.len() - 1
            });

        u32::try_from(place).expect("a scene's materials number under 2^32")
    }

    /// Adds the object of `grid`, whose cells hold places among the
    /// scene's materials, and returns its place among the scene's objects.
    /// Errors if a tile holds over 256 materials, or if a material that
    /// passes light has a place over 65534.
    pub fn push_object(&mut self, grid: &CellGrid) -> Result<usize> {
        let opaque = &self.opaque;
        let glass_materials = &mut self.glass_materials;

        // A glass cell's word holds where its material's seven values
        // start among a view's.
        let mut place = |material: u32| {
            let place = glass_materials
                .iter()
                .position(|&placed| placed == material)
                .unwrap_or_else(|| {
                    glass_materials.push(material);

                    glass_materials.len() - 1
                });

            u32::try_from(place * 7)
                .ok()
                .filter(|&place| place < 1 << 12)
                .expect("a scene's glass materials number under 586")
        };

        let glass: Option<&mut dyn FnMut(u32) -> u32> =
            if self.glass { Some(&mut place) } else { None };

        let object = ObjectTiles::new(grid, &|material| opaque[material as usize], glass)?;

        self.objects.push(object);

        Ok(self.objects.len() - 1)
    }

    /// Places the object at `object` among the scene's objects by
    /// `transform`, which carries its cells to the world.
    pub fn place_object(&mut self, object: usize, transform: TyTransformF64) {
        assert!(object < self.objects.len(), "the object is the scene's");

        self.placements.push((object, transform));
    }
}

#[cfg(test)]
mod tests {
    use crate::{TileScene, bench_scenes};

    #[test]
    fn a_scene_keeps_each_placement_in_order_with_its_objects_tiles() {
        for (name, scene) in bench_scenes() {
            let tiles = TileScene::of(&scene, true).unwrap();

            assert_eq!(tiles.objects.len(), scene.object_count(), "{name}");
            assert_eq!(tiles.materials.len(), tiles.opaque.len(), "{name}");
            assert_eq!(tiles.placements.len(), scene.placement_count(), "{name}");

            for (&(object, transform), (_, placement)) in
                tiles.placements.iter().zip(scene.iter_placements())
            {
                let bounds = scene.object(placement.object_id).unwrap().bounds();

                assert_eq!(tiles.objects[object].bounds, bounds, "{name}");
                assert_eq!(transform, placement.transform, "{name}");
            }
        }
    }
}
