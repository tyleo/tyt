use crate::{
    BMeshFile, BMeshMaterial, BMeshPrimitive, BMeshTexture, Error, MeshPrimitive, MeshProperty,
    MeshPropertyValue, MeshTextureRef, Result,
};
use branded_id::{
    U32Id,
    soa::{IdList, IdRemap},
};

/// An object, the ordered primitives that make up one piece of geometry. A
/// hierarchy node places objects.
#[derive(Debug, Default)]
pub struct MeshObject {
    /// Display name.
    name: String,

    /// Named properties outside the modeled set, names unique.
    properties: Vec<MeshProperty>,

    /// The primitives.
    primitives: IdList<BMeshPrimitive, MeshPrimitive>,
}

impl MeshObject {
    /// An object named `name` with no primitives. Then use
    /// [`retain_primitive`](Self::retain_primitive).
    pub fn new(name: String) -> Self {
        Self {
            name,
            properties: Vec::new(),
            primitives: IdList::new(),
        }
    }

    /// Compacts the primitive id pool, returning its relabeling. The
    /// primitives' material ids were already rewritten by the caller.
    pub(crate) fn gc(&mut self) -> IdRemap<BMeshPrimitive, u32> {
        self.primitives.gc()
    }

    /// The id the next retained primitive takes.
    pub(crate) fn peek_next_primitive_id(&self) -> U32Id<BMeshPrimitive> {
        self.primitives.ids().peek_next()
    }

    /// Display name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Sets the display name.
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    /// Named properties outside the modeled set.
    pub fn properties(&self) -> &[MeshProperty] {
        &self.properties
    }

    /// The property named `name`, or `None` when the object has none.
    pub fn property(&self, name: &str) -> Option<&MeshProperty> {
        self.properties
            .iter()
            .find(|property| property.name == name)
    }

    /// Replaces the properties. Their names, values, and references are
    /// checked when the object enters a [`MeshMain`](crate::MeshMain). On a
    /// retained object use
    /// [`MeshMain::set_object_properties`](crate::MeshMain::set_object_properties).
    pub fn set_properties(&mut self, properties: Vec<MeshProperty>) {
        self.properties = properties;
    }

    /// Every texture reference the properties hold, in listing order.
    pub fn iter_texture_refs(&self) -> impl Iterator<Item = MeshTextureRef> + '_ {
        self.properties
            .iter()
            .filter_map(|property| property.value.texture_ref())
    }

    /// Every file the properties point at, in listing order.
    pub fn iter_file_ids(&self) -> impl Iterator<Item = U32Id<BMeshFile>> + '_ {
        self.properties
            .iter()
            .filter_map(|property| property.value.file_id())
    }

    /// Rewrites every texture reference through `remap`, the texture id
    /// pool's relabeling from a [`MeshMain::gc`](crate::MeshMain::gc).
    /// Requires a referentially valid object, so every translation resolves.
    pub(crate) fn relabel_textures(&mut self, remap: &IdRemap<BMeshTexture, u32>) {
        for property in &mut self.properties {
            if let MeshPropertyValue::Texture(texture_ref) = &mut property.value {
                texture_ref.texture_id = remap
                    .new_id(texture_ref.texture_id)
                    .expect("an object property draws a live texture in a valid state");
            }
        }
    }

    /// Rewrites every file reference through `remap`, the file id pool's
    /// relabeling from a [`MeshMain::gc`](crate::MeshMain::gc). Requires a
    /// referentially valid object, so every translation resolves.
    pub(crate) fn relabel_files(&mut self, remap: &IdRemap<BMeshFile, u32>) {
        for property in &mut self.properties {
            if let MeshPropertyValue::File(file_id) = &mut property.value {
                *file_id = remap
                    .new_id(*file_id)
                    .expect("an object property points at a live file in a valid state");
            }
        }
    }

    /// Rewrites every primitive's material through `remap`, the material id
    /// pool's relabeling from a [`MeshMain::gc`](crate::MeshMain::gc).
    /// Requires a referentially valid object, so every translation resolves.
    pub(crate) fn relabel_materials(&mut self, remap: &IdRemap<BMeshMaterial, u32>) {
        for (_, primitive) in self.primitives.iter_mut() {
            let Some(material_id) = primitive.material_id() else {
                continue;
            };

            let new_material_id = remap
                .new_id(material_id)
                .expect("a primitive draws a live material in a valid state");

            primitive.set_material_id(Some(new_material_id));
        }
    }

    /// Retains a primitive after any existing ones and returns its id. The
    /// material id is checked against the state's materials by
    /// [`MeshMain::retain_object`](crate::MeshMain::retain_object) on insert.
    pub fn retain_primitive(&mut self, primitive: MeshPrimitive) -> U32Id<BMeshPrimitive> {
        self.primitives.retain(primitive)
    }

    /// Releases primitive `id`. The remaining primitives keep their order.
    /// Errors, changing nothing, if `id` is not one of this object's. Leaves
    /// a hole until [`MeshMain::gc`](crate::MeshMain::gc) renumbers.
    pub fn release_primitive(&mut self, id: U32Id<BMeshPrimitive>) -> Result<()> {
        let Some(_primitive) = self.primitives.release_stable(id) else {
            return Err(Error::UnknownPrimitive { primitive_id: id });
        };

        Ok(())
    }

    /// Moves primitive `id` to position `index` in the listing, shifting the
    /// primitives between its old and new positions one slot. Errors,
    /// changing nothing, if `id` is not one of this object's or `index` is
    /// at or past [`primitive_count`](Self::primitive_count).
    pub fn move_primitive(&mut self, id: U32Id<BMeshPrimitive>, index: usize) -> Result<()> {
        if !self.primitives.ids().is_retained(id) {
            return Err(Error::UnknownPrimitive { primitive_id: id });
        }

        let count = self.primitive_count();

        if index >= count {
            return Err(Error::IndexPastCount { index, count });
        }

        self.primitives.move_to(id, index);

        Ok(())
    }

    /// The primitive `id`, or `None` if not one of this object's.
    pub fn primitive(&self, id: U32Id<BMeshPrimitive>) -> Option<&MeshPrimitive> {
        self.primitives.get(id)
    }

    /// The primitive `id`, mutably, or `None` if not one of this object's.
    /// A material change through here is unchecked. Use
    /// [`MeshMain::set_primitive_material_id`](crate::MeshMain::set_primitive_material_id)
    /// on a retained object.
    pub(crate) fn primitive_mut(
        &mut self,
        id: U32Id<BMeshPrimitive>,
    ) -> Option<&mut MeshPrimitive> {
        self.primitives.get_mut(id)
    }

    /// Number of primitives.
    pub fn primitive_count(&self) -> usize {
        self.primitives.len()
    }

    /// Primitives in listing order, as `(id, primitive)`.
    pub fn iter_primitives(
        &self,
    ) -> impl Iterator<Item = (U32Id<BMeshPrimitive>, &MeshPrimitive)> + '_ {
        self.primitives.iter()
    }
}

#[cfg(test)]
mod tests {
    use crate::{Error, MeshObject, unit_triangle};
    use branded_id::U32Id;

    #[test]
    fn primitives_retain_release_and_move_in_listing_order() {
        let mut object = MeshObject::new("o".to_owned());

        let a_id = object.retain_primitive(unit_triangle());
        let b_id = object.retain_primitive(unit_triangle());
        let c_id = object.retain_primitive(unit_triangle());

        object.release_primitive(b_id).unwrap();
        assert_eq!(object.primitive_count(), 2);
        assert!(object.primitive(b_id).is_none());

        object.move_primitive(c_id, 0).unwrap();
        let order: Vec<_> = object.iter_primitives().map(|(id, _)| id).collect();
        assert_eq!(order, [c_id, a_id]);

        assert_eq!(
            object.move_primitive(c_id, 2),
            Err(Error::IndexPastCount { index: 2, count: 2 })
        );
        assert_eq!(
            object.release_primitive(U32Id::from_u32(9)),
            Err(Error::UnknownPrimitive {
                primitive_id: U32Id::from_u32(9)
            })
        );
    }

    #[test]
    fn release_primitive_errors_on_a_released_id_and_changes_nothing() {
        let mut object = MeshObject::new("o".to_owned());

        let a_id = object.retain_primitive(unit_triangle());

        let b_id = object.retain_primitive(unit_triangle());

        object.release_primitive(a_id).unwrap();

        assert_eq!(
            object.release_primitive(a_id),
            Err(Error::UnknownPrimitive { primitive_id: a_id })
        );

        let order: Vec<_> = object
            .iter_primitives()
            .map(|(primitive_id, _)| primitive_id)
            .collect();

        assert_eq!(order, [b_id]);
    }
}
