use crate::{BoxView, PassTerms, TileScene, to_word};
use std::collections::HashMap;
use ty_math::TyLinSrgbF64;
use voxcore::material;

/// The floats of one material's three `vec4<f32>`s.
const VALUES: usize = 12;

/// `box_view.wgsl`'s `materials` as a view gathers them. The scene's glass
/// materials come first, seven `vec4<f32>`s each, where a glass cell's word
/// reaches them in 12 bits. Every other place counts in threes after them.
pub struct ViewMaterials<'a> {
    tiles: &'a TileScene,

    values: Vec<f32>,

    /// The place of the first of each run of materials a tile holds.
    runs: HashMap<Vec<u32>, u32>,

    /// The place of each material a quad holds.
    singles: HashMap<u32, u32>,
}

impl<'a> ViewMaterials<'a> {
    pub fn new(tiles: &'a TileScene) -> Self {
        let mut values: Vec<f32> = (tiles.glass_materials.iter())
            .flat_map(|&id| glass_values(tiles, id))
            .collect();

        values.resize(values.len().next_multiple_of(VALUES), 0.0);

        ViewMaterials {
            tiles,
            values,
            runs: HashMap::new(),
            singles: HashMap::new(),
        }
    }

    /// Material `id`'s three `vec4<f32>`s: its base color and metalness,
    /// its emission and roughness, and its reflectance at normal incidence
    /// and occlusion strength.
    pub fn values(&self, id: u32) -> [f32; VALUES] {
        let material = &self.tiles.materials[id as usize];
        let base = material.base_color.color;
        let emission = material.emissive_color * material.emissive_strength;
        let reflectance = material::normal_reflectance(base, material.metallic, material.ior);

        [
            base.red,
            base.green,
            base.blue,
            material.metallic,
            emission.red,
            emission.green,
            emission.blue,
            material.roughness,
            reflectance.red,
            reflectance.green,
            reflectance.blue,
            material.occlusion_strength,
        ]
        .map(|value| value as f32)
    }

    /// The place of the first of a tile's materials `ids`, which follow one
    /// another. A cell's place among them then leads to its material's
    /// values in one read. Tiles that hold the same materials in the same
    /// order share them.
    pub fn run(&mut self, ids: &[u32]) -> u32 {
        if let Some(&first) = self.runs.get(ids) {
            return first;
        }

        let first = to_word(self.values.len() / VALUES);

        for &id in ids {
            self.values.extend(self.values(id));
        }

        self.runs.insert(ids.to_vec(), first);

        first
    }

    /// The place of material `id` alone, as a quad's word holds it in 29
    /// bits.
    pub fn single(&mut self, id: u32) -> u32 {
        if let Some(&place) = self.singles.get(&id) {
            return place;
        }

        let place = to_word(self.values.len() / VALUES);

        assert!(place < 1 << 29, "a scene's materials number under 2^29");

        self.values.extend(self.values(id));
        self.singles.insert(id, place);

        place
    }

    /// The cells a mirror ray from a face of material `id` runs, or zero
    /// for a material whose faces cast none. A ray needs a device that
    /// has `traced` rays, and a reach past [`BoxView::MIRROR_REACH`].
    pub fn mirror_reach(&self, id: u32, traced: bool) -> f32 {
        let values = self.values(id);
        let roughness = values[7];
        let alpha = (roughness * roughness).max(1.0e-3);
        let reflectance = values[8].max(values[9]).max(values[10]);
        let far = (reflectance / BoxView::MIRROR_CUT - 1.0).max(0.0).sqrt() / alpha;

        if traced && roughness < 1.0 && far > BoxView::MIRROR_REACH {
            far
        } else {
            0.0
        }
    }

    /// Whether the brightest channel of material `id`'s emission passes
    /// `threshold`.
    pub fn emits_over(&self, id: u32, threshold: f32) -> bool {
        let values = self.values(id);

        values[4].max(values[5]).max(values[6]) > threshold
    }

    /// The floats, with room for 255 materials more. A hit in a row with no
    /// record cell can read another tile's word in `lit`, whose material
    /// lies up to 255 past the hit tile's first.
    pub fn finish(mut self) -> Vec<f32> {
        self.values.resize(self.values.len() + 255 * VALUES, 0.0);

        self.values
    }
}

/// The seven `vec4<f32>`s `box_glass.wgsl`'s `past` lists for the glass
/// material `id` of `tiles`.
fn glass_values(tiles: &TileScene, id: u32) -> [f32; 28] {
    let material = &tiles.materials[id as usize];
    let terms = PassTerms::new(material);
    let white = TyLinSrgbF64::new(1.0, 1.0, 1.0);
    let black = TyLinSrgbF64::new(0.0, 0.0, 0.0);
    let alpha = (material.roughness * material.roughness).max(1.0e-3);
    let emission = material.emissive_color * material.emissive_strength;
    let transmitted = material.transmission * (1.0 - material.metallic);
    let diffuse =
        material.base_color.color * ((1.0 - material.metallic) * (1.0 - material.transmission));
    let rise = terms.rise * transmitted;

    // A reflectance with no hue fits one channel.
    let gloss = terms.reflectance;
    let plain = gloss.red == gloss.green && gloss.red == gloss.blue;

    let follows = if !plain || emission != black {
        2.0
    } else if diffuse != black {
        1.0
    } else {
        0.0
    };

    let passed = white * terms.uncovered + terms.most * (white - gloss);

    let colors = [
        (passed, material.base_color.alpha),
        (terms.most * terms.rise, 0.5 / alpha),
        (
            TyLinSrgbF64::new(gloss.red, rise.red, alpha * alpha),
            follows,
        ),
        (diffuse, 0.0),
        (gloss, 0.0),
        (rise, 0.0),
        (emission, 0.0),
    ];

    let mut values = [0.0; 28];

    for (values, (color, last)) in values.chunks_exact_mut(4).zip(colors) {
        values
            .copy_from_slice(&[color.red, color.green, color.blue, last].map(|value| value as f32));
    }

    values
}
