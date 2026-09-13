use crate::{
    BMeshMaterial, BMeshTriangle, BMeshUvStream, BMeshVertex, BMeshVertexAttribute, Error,
    MeshTriangle, MeshVertexAttribute, Result,
};
use branded_id::{IdSlice, IdVec, U32Id};
use ty_math::{TyLinSrgbaF64, TyVector2F64, TyVector3F64, TyVector4F64};

/// One drawable piece of a [`MeshObject`](crate::MeshObject).
///
/// Positions are in meters, Z-up. Triangles wind counter-clockwise seen from
/// outside. Every mutation that takes a stream checks it holds one entry per
/// vertex and every value is finite, so a primitive is always consistent.
/// A vertex id is its position in the streams, a triangle id its position
/// in the triangle list, and a UV stream id its position in the stream list.
/// The material id references a [`MeshMain`](crate::MeshMain) and is
/// meaningful only within it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MeshPrimitive {
    /// Display name, empty for an unnamed primitive.
    name: String,

    /// One position per vertex.
    positions: IdVec<BMeshVertex, TyVector3F64>,

    /// One unit normal per vertex, when the primitive carries normals.
    normals: Option<IdVec<BMeshVertex, TyVector3F64>>,

    /// One tangent per vertex, `w` the bitangent sign, when the primitive
    /// carries tangents.
    tangents: Option<IdVec<BMeshVertex, TyVector4F64>>,

    /// The UV streams, each one coordinate per vertex.
    uv_streams: IdVec<BMeshUvStream, IdVec<BMeshVertex, TyVector2F64>>,

    /// One straight-alpha linear color per vertex, when the primitive
    /// carries colors.
    colors: Option<IdVec<BMeshVertex, TyLinSrgbaF64>>,

    /// Further per-vertex attributes, names unique.
    vertex_attributes: IdVec<BMeshVertexAttribute, MeshVertexAttribute>,

    /// The triangles.
    triangles: IdVec<BMeshTriangle, MeshTriangle>,

    /// The material drawn, or `None` for the renderer's default.
    material_id: Option<U32Id<BMeshMaterial>>,
}

impl MeshPrimitive {
    /// A primitive of `positions` and `triangles`, with no other streams and
    /// no material. Errors, building nothing, if:
    ///
    /// 1. either listing has more entries than a `u32` id addresses
    /// 2. a position is not finite
    /// 3. a corner references a vertex past the positions
    pub fn new(positions: Vec<TyVector3F64>, triangles: Vec<MeshTriangle>) -> Result<Self> {
        let vertex_count = listing_count(positions.len())?;

        listing_count(triangles.len())?;

        if let Some(index) = positions.iter().position(|position| !position.is_finite()) {
            return Err(Error::NonFiniteVertex {
                vertex_id: vertex_id_at(index),
            });
        }

        for (index, triangle) in triangles.iter().enumerate() {
            if let Some(&vertex_id) = triangle
                .vertex_ids
                .iter()
                .find(|vertex_id| vertex_id.to_u32() >= vertex_count)
            {
                return Err(Error::CornerVertex {
                    triangle_id: U32Id::from_u32(
                        u32::try_from(index).expect("the triangle count was checked"),
                    ),
                    vertex_id,
                });
            }
        }

        Ok(Self {
            name: String::new(),
            positions: IdVec::from_vec(positions),
            normals: None,
            tangents: None,
            uv_streams: IdVec::default(),
            colors: None,
            vertex_attributes: IdVec::default(),
            triangles: IdVec::from_vec(triangles),
            material_id: None,
        })
    }

    /// Display name, empty for an unnamed primitive.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Sets the display name.
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    /// Number of vertices.
    pub fn vertex_count(&self) -> usize {
        self.positions.len()
    }

    /// Number of triangles.
    pub fn triangle_count(&self) -> usize {
        self.triangles.len()
    }

    /// The positions, one per vertex.
    pub fn positions(&self) -> &IdSlice<BMeshVertex, TyVector3F64> {
        &self.positions
    }

    /// The triangles.
    pub fn triangles(&self) -> &IdSlice<BMeshTriangle, MeshTriangle> {
        &self.triangles
    }

    /// The normals, one per vertex, or `None` when the primitive carries none.
    pub fn normals(&self) -> Option<&IdSlice<BMeshVertex, TyVector3F64>> {
        self.normals.as_deref()
    }

    /// Sets or clears the normals. Errors, changing nothing, if `normals`
    /// does not hold one entry per vertex or an entry is not finite.
    pub fn set_normals(&mut self, normals: Option<Vec<TyVector3F64>>) -> Result<()> {
        self.normals = match normals {
            Some(normals) => Some(self.checked_stream(normals, |normal| normal.is_finite())?),
            None => None,
        };

        Ok(())
    }

    /// The tangents, one per vertex, or `None` when the primitive carries
    /// none.
    pub fn tangents(&self) -> Option<&IdSlice<BMeshVertex, TyVector4F64>> {
        self.tangents.as_deref()
    }

    /// Sets or clears the tangents. Errors, changing nothing, if `tangents`
    /// does not hold one entry per vertex or an entry is not finite.
    pub fn set_tangents(&mut self, tangents: Option<Vec<TyVector4F64>>) -> Result<()> {
        self.tangents = match tangents {
            Some(tangents) => Some(self.checked_stream(tangents, |tangent| tangent.is_finite())?),
            None => None,
        };

        Ok(())
    }

    /// Number of UV streams.
    pub fn uv_stream_count(&self) -> usize {
        self.uv_streams.len()
    }

    /// The UV stream `id`, one coordinate per vertex, or `None` if `id` is
    /// not one of this primitive's streams.
    pub fn uv_stream(
        &self,
        id: U32Id<BMeshUvStream>,
    ) -> Option<&IdSlice<BMeshVertex, TyVector2F64>> {
        self.uv_streams
            .get(id.to_usize_id())
            .map(IdVec::as_id_slice)
    }

    /// The UV streams in stream order, as `(id, stream)`.
    pub fn iter_uv_streams(
        &self,
    ) -> impl Iterator<Item = (U32Id<BMeshUvStream>, &IdSlice<BMeshVertex, TyVector2F64>)> + '_
    {
        self.uv_streams
            .iter()
            .enumerate()
            .map(|(index, stream)| (U32Id::from_u32(index as u32), stream.as_id_slice()))
    }

    /// Appends a UV stream and returns its id. Errors, changing nothing, if
    /// `uvs` does not hold one entry per vertex, an entry is not finite, or
    /// the stream count would pass what a `u32` id addresses.
    pub fn push_uv_stream(&mut self, uvs: Vec<TyVector2F64>) -> Result<U32Id<BMeshUvStream>> {
        let id = U32Id::from_u32(listing_count(self.uv_streams.len() + 1)? - 1);

        let stream = self.checked_stream(uvs, |uv| uv.is_finite())?;

        self.uv_streams.push(stream);

        Ok(id)
    }

    /// The vertex colors, one per vertex, or `None` when the primitive
    /// carries none.
    pub fn colors(&self) -> Option<&IdSlice<BMeshVertex, TyLinSrgbaF64>> {
        self.colors.as_deref()
    }

    /// Sets or clears the vertex colors. Errors, changing nothing, if
    /// `colors` does not hold one entry per vertex or a component is not
    /// finite.
    pub fn set_colors(&mut self, colors: Option<Vec<TyLinSrgbaF64>>) -> Result<()> {
        self.colors = match colors {
            Some(colors) => Some(self.checked_stream(colors, |color| {
                <[f64; 4]>::from(*color)
                    .iter()
                    .all(|component| component.is_finite())
            })?),
            None => None,
        };

        Ok(())
    }

    /// Number of further vertex attributes.
    pub fn vertex_attribute_count(&self) -> usize {
        self.vertex_attributes.len()
    }

    /// The further vertex attribute `id`, or `None` if `id` is not one of
    /// this primitive's.
    pub fn vertex_attribute(
        &self,
        id: U32Id<BMeshVertexAttribute>,
    ) -> Option<&MeshVertexAttribute> {
        self.vertex_attributes.get(id.to_usize_id())
    }

    /// The further vertex attributes in listing order, as `(id, attribute)`.
    pub fn iter_vertex_attributes(
        &self,
    ) -> impl Iterator<Item = (U32Id<BMeshVertexAttribute>, &MeshVertexAttribute)> + '_ {
        self.vertex_attributes
            .iter()
            .enumerate()
            .map(|(index, attribute)| (U32Id::from_u32(index as u32), attribute))
    }

    /// Appends a further vertex attribute and returns its id. Errors,
    /// changing nothing, if:
    ///
    /// 1. the attribute's width is `0` or its components are not `width`
    ///    per vertex
    /// 2. a component is not finite
    /// 3. the primitive already has an attribute of the name
    /// 4. the attribute count would pass what a `u32` id addresses
    pub fn push_vertex_attribute(
        &mut self,
        attribute: MeshVertexAttribute,
    ) -> Result<U32Id<BMeshVertexAttribute>> {
        let id = U32Id::from_u32(listing_count(self.vertex_attributes.len() + 1)? - 1);

        let expected = attribute.width.saturating_mul(self.vertex_count());

        if attribute.width == 0 || attribute.components.len() != expected {
            return Err(Error::AttributeArity {
                name: attribute.name,
                components: attribute.components.len(),
                expected,
            });
        }

        if let Some(index) = attribute.components.first_non_finite_index() {
            return Err(Error::NonFiniteVertex {
                vertex_id: vertex_id_at(index / attribute.width),
            });
        }

        if self
            .vertex_attributes
            .iter()
            .any(|existing| existing.name == attribute.name)
        {
            return Err(Error::DuplicateVertexAttributeName {
                name: attribute.name,
            });
        }

        self.vertex_attributes.push(attribute);

        Ok(id)
    }

    /// The material the triangles draw with, or `None` for the renderer's
    /// default.
    pub fn material_id(&self) -> Option<U32Id<BMeshMaterial>> {
        self.material_id
    }

    /// Sets the material the triangles draw with. Whether it is one of a
    /// state's materials is checked by
    /// [`MeshMain::retain_primitive`](crate::MeshMain::retain_primitive) on
    /// insert and by
    /// [`MeshMain::set_primitive_material_id`](crate::MeshMain::set_primitive_material_id)
    /// on a live primitive.
    pub fn set_material_id(&mut self, material_id: Option<U32Id<BMeshMaterial>>) {
        self.material_id = material_id;
    }

    /// `stream` as a vertex column, checked to hold one finite entry per
    /// vertex.
    fn checked_stream<T>(
        &self,
        stream: Vec<T>,
        is_finite: impl Fn(&T) -> bool,
    ) -> Result<IdVec<BMeshVertex, T>> {
        if stream.len() != self.positions.len() {
            return Err(Error::StreamArity {
                entries: stream.len(),
                vertices: self.vertex_count(),
            });
        }

        if let Some(index) = stream.iter().position(|entry| !is_finite(entry)) {
            return Err(Error::NonFiniteVertex {
                vertex_id: vertex_id_at(index),
            });
        }

        Ok(IdVec::from_vec(stream))
    }
}

/// `count` as the `u32` a listing of that many entries has, or the cap error.
fn listing_count(count: usize) -> Result<u32> {
    u32::try_from(count).map_err(|_| Error::ListingCap {
        entries: count as u64,
    })
}

/// The vertex id at listing position `index` of a checked stream.
fn vertex_id_at(index: usize) -> U32Id<BMeshVertex> {
    U32Id::from_u32(u32::try_from(index).expect("a checked stream holds at most u32::MAX entries"))
}

#[cfg(test)]
mod tests {
    use crate::{
        BMeshVertex, Error, MeshAttributeComponents, MeshPrimitive, MeshTriangle,
        MeshVertexAttribute, unit_triangle,
    };
    use branded_id::U32Id;
    use ty_math::{TyVector2F64, TyVector3F64};

    fn vertex_id(index: u32) -> U32Id<BMeshVertex> {
        U32Id::from_u32(index)
    }

    #[test]
    fn a_corner_past_the_vertices_is_rejected() {
        let error = MeshPrimitive::new(
            vec![TyVector3F64::ZERO, TyVector3F64::X],
            vec![MeshTriangle {
                vertex_ids: [vertex_id(0), vertex_id(1), vertex_id(2)],
            }],
        )
        .unwrap_err();

        assert_eq!(
            error,
            Error::CornerVertex {
                triangle_id: U32Id::from_u32(0),
                vertex_id: vertex_id(2),
            }
        );
    }

    #[test]
    fn a_non_finite_position_is_rejected() {
        let error = MeshPrimitive::new(
            vec![TyVector3F64::ZERO, TyVector3F64::NAN, TyVector3F64::X],
            Vec::new(),
        )
        .unwrap_err();

        assert_eq!(
            error,
            Error::NonFiniteVertex {
                vertex_id: vertex_id(1)
            }
        );
    }

    #[test]
    fn streams_hold_one_entry_per_vertex() {
        let mut primitive = unit_triangle();

        assert_eq!(
            primitive.set_normals(Some(vec![TyVector3F64::Z; 2])),
            Err(Error::StreamArity {
                entries: 2,
                vertices: 3,
            })
        );
        assert_eq!(primitive.normals(), None);

        primitive
            .set_normals(Some(vec![TyVector3F64::Z; 3]))
            .unwrap();
        assert_eq!(primitive.normals().unwrap().len(), 3);

        primitive.set_normals(None).unwrap();
        assert_eq!(primitive.normals(), None);

        let stream_id = primitive
            .push_uv_stream(vec![TyVector2F64::ZERO; 3])
            .unwrap();
        assert_eq!(stream_id.to_u32(), 0);
        assert_eq!(primitive.uv_stream_count(), 1);
        assert_eq!(primitive.uv_stream(U32Id::from_u32(1)), None);
    }

    #[test]
    fn attributes_check_their_width_and_names() {
        let mut primitive = unit_triangle();

        let attribute = |name: &str, width: usize, components: Vec<f64>| MeshVertexAttribute {
            name: name.to_owned(),
            width,
            components: MeshAttributeComponents::F64(components),
        };

        assert!(matches!(
            primitive.push_vertex_attribute(attribute("a", 2, vec![0.0; 5])),
            Err(Error::AttributeArity { .. })
        ));
        assert!(matches!(
            primitive.push_vertex_attribute(attribute("a", 0, Vec::new())),
            Err(Error::AttributeArity { .. })
        ));
        assert_eq!(
            primitive.push_vertex_attribute(attribute(
                "a",
                2,
                vec![0.0, 0.0, 0.0, f64::NAN, 0.0, 0.0]
            )),
            Err(Error::NonFiniteVertex {
                vertex_id: vertex_id(1)
            })
        );

        primitive
            .push_vertex_attribute(attribute("a", 2, vec![0.0; 6]))
            .unwrap();

        assert_eq!(
            primitive.push_vertex_attribute(attribute("a", 1, vec![0.0; 3])),
            Err(Error::DuplicateVertexAttributeName {
                name: "a".to_owned()
            })
        );
        assert_eq!(primitive.vertex_attribute_count(), 1);

        primitive
            .push_vertex_attribute(MeshVertexAttribute {
                name: "b".to_owned(),
                width: 1,
                components: MeshAttributeComponents::U16(vec![0, 1, 2]),
            })
            .unwrap();
        assert_eq!(primitive.vertex_attribute_count(), 2);
    }
}
