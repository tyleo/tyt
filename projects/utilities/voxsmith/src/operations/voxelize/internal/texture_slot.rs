use crate::operations::voxelize::{DecodedImage, MeshInput, PlacedPrimitive};
use branded_id::{IdSlice, U32Id};
use meshdoc::{BMeshVertex, MeshTextureRef, MeshWrap};
use ty_math::TyVector2F64;

/// One texture slot of a placed primitive's material, resolved for sampling.
pub(crate) struct TextureSlot<'a> {
    /// The decoded image the slot's texture samples.
    pub image: &'a DecodedImage,

    /// The texture's wrap along `u`.
    pub wrap_s: MeshWrap,

    /// The texture's wrap along `v`.
    pub wrap_t: MeshWrap,

    /// The primitive's texture coordinates for the slot, by vertex id.
    pub uvs: &'a IdSlice<BMeshVertex, TyVector2F64>,
}

impl<'a> TextureSlot<'a> {
    /// The slot `texture_ref` binds on `primitive`, or `None` when the slot
    /// is unbound or the primitive lacks the stream it references.
    pub fn resolve(
        input: &'a MeshInput<'_>,
        primitive: &'a PlacedPrimitive<'_>,
        texture_ref: Option<MeshTextureRef>,
    ) -> Option<Self> {
        let texture_ref = texture_ref?;

        let uvs = primitive.primitive.uv_stream(texture_ref.uv_stream_id)?;

        let texture = input
            .state
            .texture(texture_ref.texture_id)
            .expect("a material draws one of the document's textures");

        let image = input
            .images
            .get(&texture.image_id)
            .expect("the flatten decoded every image a sampled slot draws");

        Some(Self {
            image,
            wrap_s: texture.wrap_s,
            wrap_t: texture.wrap_t,
            uvs,
        })
    }

    /// The texel at barycentric weights `(a, b, c)` over the triangle
    /// `vertex_ids`.
    pub fn sample(&self, vertex_ids: [U32Id<BMeshVertex>; 3], a: f64, b: f64, c: f64) -> [u8; 4] {
        let [p, q, r] = vertex_ids.map(|vertex_id| self.uvs[vertex_id.to_usize_id()]);

        let uv = TyVector2F64::new(p.x * a + q.x * b + r.x * c, p.y * a + q.y * b + r.y * c);

        self.image.sample(uv, self.wrap_s, self.wrap_t)
    }
}
