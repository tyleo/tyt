use crate::{
    BMeshFile, BMeshHierarchyNode, BMeshImage, BMeshMaterial, BMeshObject, BMeshPrimitive,
    BMeshTexture, Error, MeshFile, MeshHierarchyNode, MeshImage, MeshImageSource, MeshMaterial,
    MeshObject, MeshPrimitive, MeshProperty, MeshTexture, Result, first_cycle_node_index,
};
use branded_id::{U32Id, soa::IdList};
use std::collections::{HashMap, HashSet};
use ty_math::{TyQuaternionExt, UNIT_ROTATION_TOLERANCE};

/// The document of a mesh model: the read side of
/// [`MeshMain`](crate::MeshMain), which forwards to it.
///
/// This is the backing store. [`MeshMain`](crate::MeshMain) owns mutation logic
/// over these fields. The fields are crate-private so every edit goes through
/// the referential checks. An ext hook reads the document through this type.
#[derive(Debug, Default)]
pub struct MeshState {
    /// The files.
    pub(crate) files: IdList<BMeshFile, MeshFile>,

    /// The images.
    pub(crate) images: IdList<BMeshImage, MeshImage>,

    /// The textures.
    pub(crate) textures: IdList<BMeshTexture, MeshTexture>,

    /// The materials.
    pub(crate) materials: IdList<BMeshMaterial, MeshMaterial>,

    /// The objects.
    pub(crate) objects: IdList<BMeshObject, MeshObject>,

    /// The hierarchy nodes.
    pub(crate) hierarchy_nodes: IdList<BMeshHierarchyNode, MeshHierarchyNode>,

    /// The document's roots: hierarchy node ids.
    pub(crate) root_hierarchy_node_ids: Vec<U32Id<BMeshHierarchyNode>>,
}

impl MeshState {
    /// Audits the full rule set. Every rule here is also enforced at a
    /// mutation point, so a state reached through the public API always
    /// passes. A failure reports a meshdoc bug, never a caller error. The
    /// checks stay as the specification of what the mutations preserve:
    ///
    /// 1. every file has a name, and no two files share one
    /// 2. every image over a file reads a live one, and every image's bytes
    ///    start with its media type's signature
    /// 3. every texture samples a live image
    /// 4. per material:
    ///    1. every factor is within the range its name fixes
    ///    2. every texture drawn is live
    ///    3. its properties are well formed: no name repeats, no float is
    ///       non-finite, and every texture and file referenced is live
    /// 5. per object, its properties are well formed as a material's are,
    ///    and every primitive with a material draws a live one and carries
    ///    every UV stream the material's textures sample
    /// 6. every node child node and child object resolves, and no node lists
    ///    the same one twice
    /// 7. every root resolves, and no root repeats
    /// 8. every node transform has finite position and scale components, a
    ///    non-zero scale on each axis, and a unit-length rotation quaternion
    ///    within `1e-6`
    /// 9. the `child_node_ids` graph is acyclic
    ///
    /// A node may have several parents because the hierarchy is a DAG. That
    /// sharing is not a cycle.
    pub fn validate(&self) -> Result<()> {
        let mut file_ids_by_name = HashMap::with_capacity(self.file_count());

        for (file_id, file) in self.iter_files() {
            if file.name.is_empty() {
                return Err(Error::FileName { file_id });
            }

            if let Some(&other_file_id) = file_ids_by_name.get(file.name.as_str()) {
                return Err(Error::DuplicateFileName {
                    file_id,
                    other_file_id,
                });
            }

            file_ids_by_name.insert(file.name.as_str(), file_id);
        }

        for (image_id, image) in self.iter_images() {
            let bytes = match &image.source {
                MeshImageSource::Bytes(bytes) => bytes.as_slice(),

                MeshImageSource::File(file_id) => {
                    let file = self.file(*file_id).ok_or(Error::ImageFile {
                        image_id,
                        file_id: *file_id,
                    })?;
                    file.bytes.as_slice()
                }
            };

            if !image.media_type.matches(bytes) {
                return Err(Error::ImageBytes { image_id });
            }
        }

        for (texture_id, texture) in self.iter_textures() {
            if self.image(texture.image_id).is_none() {
                return Err(Error::TextureImage {
                    texture_id,
                    image_id: texture.image_id,
                });
            }
        }

        for (material_id, material) in self.iter_materials() {
            if let Some((name, _)) = material.first_factor_out_of_range() {
                return Err(Error::MaterialFactorRange {
                    material_id,
                    name: name.to_owned(),
                });
            }

            for texture_ref in material.iter_texture_refs() {
                if self.texture(texture_ref.texture_id).is_none() {
                    return Err(Error::MaterialTexture {
                        material_id,
                        texture_id: texture_ref.texture_id,
                    });
                }
            }

            self.check_properties(&material.properties)?;
        }

        // Object properties, then primitive material refs and their UV
        // streams. Checks are by id retention, not index range, so they hold
        // whether or not releases have left the id pools with holes.
        for (object_id, object) in self.iter_objects() {
            self.check_properties(object.properties())?;

            for (primitive_id, primitive) in object.iter_primitives() {
                let Some(material_id) = primitive.material_id() else {
                    continue;
                };

                let material = self.material(material_id).ok_or(Error::PrimitiveMaterial {
                    object_id,
                    primitive_id,
                    material_id,
                })?;

                for texture_ref in material.iter_texture_refs() {
                    if primitive.uv_stream(texture_ref.uv_stream_id).is_none() {
                        return Err(Error::PrimitiveUvStream {
                            object_id,
                            primitive_id,
                            material_id,
                            uv_stream_id: texture_ref.uv_stream_id,
                        });
                    }
                }
            }
        }

        // Node children; retention-checked before the cycle pass.
        for (node_id, node) in self.iter_hierarchy_nodes() {
            let mut seen_child_node_ids = HashSet::with_capacity(node.child_node_ids.len());
            for &child_id in &node.child_node_ids {
                if self.hierarchy_node(child_id).is_none() {
                    return Err(Error::ChildNode { node_id, child_id });
                }
                if !seen_child_node_ids.insert(child_id) {
                    return Err(Error::DuplicateChildNode { node_id, child_id });
                }
            }

            let mut seen_child_object_ids = HashSet::with_capacity(node.child_object_ids.len());
            for &object_id in &node.child_object_ids {
                if self.object(object_id).is_none() {
                    return Err(Error::ChildObject { node_id, object_id });
                }
                if !seen_child_object_ids.insert(object_id) {
                    return Err(Error::DuplicateChildObject { node_id, object_id });
                }
            }

            // The node transform must be finite and non-degenerate. The
            // rotation needs no finiteness guard of its own: a non-finite
            // component fails the unit-length check below.
            let position = node.transform.position;
            let scale = node.transform.scale;
            if !position.is_finite() || !scale.is_finite() {
                return Err(Error::NonFiniteTransform { node_id });
            }

            if scale.x == 0.0 || scale.y == 0.0 || scale.z == 0.0 {
                return Err(Error::ZeroScale { node_id });
            }

            let rotation = node.transform.rotation;
            if !rotation.is_normalized_within(UNIT_ROTATION_TOLERANCE) {
                return Err(Error::NonUnitRotation { node_id });
            }
        }

        // Roots.
        let mut seen_root_ids = HashSet::with_capacity(self.root_hierarchy_node_ids.len());

        for &root_id in &self.root_hierarchy_node_ids {
            if self.hierarchy_node(root_id).is_none() {
                return Err(Error::Root { root_id });
            }
            if !seen_root_ids.insert(root_id) {
                return Err(Error::DuplicateRoot { root_id });
            }
        }

        // Acyclicity; every child is now known live. Works over the retained
        // node ids by position, so it holds whether or not the node id pool
        // has holes.
        let node_ids: Vec<_> = self.hierarchy_nodes.ids().iter().collect();

        let index_of: HashMap<U32Id<BMeshHierarchyNode>, usize> = node_ids
            .iter()
            .enumerate()
            .map(|(node_index, &node_id)| (node_id, node_index))
            .collect();

        let children: Vec<&[U32Id<BMeshHierarchyNode>]> = self
            .iter_hierarchy_nodes()
            .map(|(_, node)| node.child_node_ids.as_slice())
            .collect();

        if let Some(node_index) = first_cycle_node_index(&children, &index_of) {
            return Err(Error::Cycle {
                node_id: node_ids[node_index],
            });
        }

        Ok(())
    }

    /// Checks a node about to be inserted at listing position `node_index` of
    /// its batch, resolving child nodes against this state and the batch's
    /// prospective `batch_ids`.
    pub(crate) fn check_inserted_node(
        &self,
        node: &MeshHierarchyNode,
        node_index: usize,
        batch_ids: &HashSet<U32Id<BMeshHierarchyNode>>,
    ) -> Result<()> {
        let mut seen_child_node_ids = HashSet::with_capacity(node.child_node_ids.len());

        for &child_id in &node.child_node_ids {
            if self.hierarchy_node(child_id).is_none() && !batch_ids.contains(&child_id) {
                return Err(Error::UnknownHierarchyNode { node_id: child_id });
            }

            if !seen_child_node_ids.insert(child_id) {
                return Err(Error::InsertedDuplicateChildNode {
                    index: node_index,
                    child_id,
                });
            }
        }

        let mut seen_child_object_ids = HashSet::with_capacity(node.child_object_ids.len());

        for &object_id in &node.child_object_ids {
            if self.object(object_id).is_none() {
                return Err(Error::UnknownObject { object_id });
            }

            if !seen_child_object_ids.insert(object_id) {
                return Err(Error::InsertedDuplicateChildObject {
                    index: node_index,
                    object_id,
                });
            }
        }

        // The rotation needs no finiteness guard of its own: a non-finite
        // component fails the unit-length check.
        let position = node.transform.position;
        let scale = node.transform.scale;
        if !position.is_finite() || !scale.is_finite() {
            return Err(Error::InsertedNonFiniteTransform { index: node_index });
        }

        if scale.x == 0.0 || scale.y == 0.0 || scale.z == 0.0 {
            return Err(Error::InsertedZeroScale { index: node_index });
        }

        if !node
            .transform
            .rotation
            .is_normalized_within(UNIT_ROTATION_TOLERANCE)
        {
            return Err(Error::InsertedNonUnitRotation { index: node_index });
        }

        Ok(())
    }

    /// Checks a material about to be inserted or replaced: its factors are
    /// within range, its textures are this state's, and its properties are
    /// well formed.
    pub(crate) fn check_inserted_material(&self, material: &MeshMaterial) -> Result<()> {
        if let Some((name, value)) = material.first_factor_out_of_range() {
            return Err(Error::MaterialFactor {
                name: name.to_owned(),
                value,
            });
        }

        for texture_ref in material.iter_texture_refs() {
            if self.texture(texture_ref.texture_id).is_none() {
                return Err(Error::MaterialTextureRef {
                    texture_id: texture_ref.texture_id,
                });
            }
        }

        self.check_properties(&material.properties)
    }

    /// Checks a property list about to enter this state: no name repeats, no
    /// float is non-finite, and every texture and file referenced is one of
    /// this state's.
    pub(crate) fn check_properties(&self, properties: &[MeshProperty]) -> Result<()> {
        if let Some(name) = first_duplicate_property_name(properties) {
            return Err(Error::DuplicatePropertyName {
                name: name.to_owned(),
            });
        }

        if let Some(name) = first_non_finite_property(properties) {
            return Err(Error::NonFiniteProperty {
                name: name.to_owned(),
            });
        }

        for property in properties {
            if let Some(texture_ref) = property.value.texture_ref()
                && self.texture(texture_ref.texture_id).is_none()
            {
                return Err(Error::PropertyTextureRef {
                    name: property.name.clone(),
                    texture_id: texture_ref.texture_id,
                });
            }

            if let Some(file_id) = property.value.file_id()
                && self.file(file_id).is_none()
            {
                return Err(Error::PropertyFileRef {
                    name: property.name.clone(),
                    file_id,
                });
            }
        }

        Ok(())
    }

    /// Checks a file about to be inserted as, or replace, `file_id`: it has
    /// a name no other file uses, and every image reading it still starts
    /// with its media type's signature.
    pub(crate) fn check_inserted_file(
        &self,
        file_id: U32Id<BMeshFile>,
        file: &MeshFile,
    ) -> Result<()> {
        if file.name.is_empty() {
            return Err(Error::EmptyFileName);
        }

        if let Some((other_file_id, _)) = self
            .iter_files()
            .find(|&(other_file_id, other)| other_file_id != file_id && other.name == file.name)
        {
            return Err(Error::FileNameTaken {
                name: file.name.clone(),
                file_id: other_file_id,
            });
        }

        for (_, image) in self.iter_images() {
            if image.source == MeshImageSource::File(file_id)
                && !image.media_type.matches(&file.bytes)
            {
                return Err(Error::MalformedImage {
                    media_type: image.media_type,
                });
            }
        }

        Ok(())
    }

    /// Checks an image about to be inserted or replaced: its bytes, own or a
    /// live file's, start with its media type's signature.
    pub(crate) fn check_inserted_image(&self, image: &MeshImage) -> Result<()> {
        let bytes = match &image.source {
            MeshImageSource::Bytes(bytes) => bytes.as_slice(),

            MeshImageSource::File(file_id) => {
                let file = self
                    .file(*file_id)
                    .ok_or(Error::ImageFileRef { file_id: *file_id })?;
                file.bytes.as_slice()
            }
        };

        if !image.media_type.matches(bytes) {
            return Err(Error::MalformedImage {
                media_type: image.media_type,
            });
        }

        Ok(())
    }

    /// Checks a primitive about to be inserted as `primitive_id`, or about to
    /// draw with `material_id` in place of its own: the material is this
    /// state's and the primitive carries every UV stream its textures sample.
    pub(crate) fn check_primitive_material(
        &self,
        primitive_id: U32Id<BMeshPrimitive>,
        primitive: &MeshPrimitive,
        material_id: Option<U32Id<BMeshMaterial>>,
    ) -> Result<()> {
        let Some(material_id) = material_id else {
            return Ok(());
        };

        let Some(material) = self.material(material_id) else {
            return Err(Error::PrimitiveMaterialRef {
                primitive_id,
                material_id,
            });
        };

        for texture_ref in material.iter_texture_refs() {
            if primitive.uv_stream(texture_ref.uv_stream_id).is_none() {
                return Err(Error::PrimitiveUvStreamRef {
                    primitive_id,
                    material_id,
                    uv_stream_id: texture_ref.uv_stream_id,
                });
            }
        }

        Ok(())
    }

    /// The hierarchy node `id`, or `None` if not one of this state's.
    pub fn hierarchy_node(&self, id: U32Id<BMeshHierarchyNode>) -> Option<&MeshHierarchyNode> {
        self.hierarchy_nodes.get(id)
    }

    /// Number of hierarchy nodes.
    pub fn hierarchy_node_count(&self) -> usize {
        self.hierarchy_nodes.len()
    }

    /// Hierarchy nodes in listing order, as `(id, node)`.
    pub fn iter_hierarchy_nodes(
        &self,
    ) -> impl Iterator<Item = (U32Id<BMeshHierarchyNode>, &MeshHierarchyNode)> + '_ {
        self.hierarchy_nodes.iter()
    }

    /// Whether `target` is one of `from` or reachable from any of them through
    /// `child_node_ids`. The walk is iterative, so a deep chain cannot
    /// overflow the stack. A shared node is visited once.
    pub(crate) fn reaches_hierarchy_node(
        &self,
        from: &[U32Id<BMeshHierarchyNode>],
        target: U32Id<BMeshHierarchyNode>,
    ) -> bool {
        let mut visited = HashSet::new();
        let mut stack: Vec<U32Id<BMeshHierarchyNode>> = from.to_vec();

        while let Some(node_id) = stack.pop() {
            if node_id == target {
                return true;
            }

            if !visited.insert(node_id) {
                continue;
            }

            if let Some(node) = self.hierarchy_node(node_id) {
                stack.extend(node.child_node_ids.iter().copied());
            }
        }

        false
    }

    /// The file `id`, or `None` if not one of this state's.
    pub fn file(&self, id: U32Id<BMeshFile>) -> Option<&MeshFile> {
        self.files.get(id)
    }

    /// The file named `name`, as `(id, file)`, or `None` if no file has the
    /// name.
    pub fn file_by_name(&self, name: &str) -> Option<(U32Id<BMeshFile>, &MeshFile)> {
        self.iter_files().find(|(_, file)| file.name == name)
    }

    /// Number of files.
    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    /// Files in listing order, as `(id, file)`.
    pub fn iter_files(&self) -> impl Iterator<Item = (U32Id<BMeshFile>, &MeshFile)> + '_ {
        self.files.iter()
    }

    /// The image `id`, or `None` if not one of this state's.
    pub fn image(&self, id: U32Id<BMeshImage>) -> Option<&MeshImage> {
        self.images.get(id)
    }

    /// The encoded bytes of image `id`, its own or its file's, or `None` if
    /// `id` is not one of this state's images.
    pub fn image_bytes(&self, id: U32Id<BMeshImage>) -> Option<&[u8]> {
        let image = self.image(id)?;

        match &image.source {
            MeshImageSource::Bytes(bytes) => Some(bytes.as_slice()),
            MeshImageSource::File(file_id) => self.file(*file_id).map(|file| file.bytes.as_slice()),
        }
    }

    /// Number of images.
    pub fn image_count(&self) -> usize {
        self.images.len()
    }

    /// Images in listing order, as `(id, image)`.
    pub fn iter_images(&self) -> impl Iterator<Item = (U32Id<BMeshImage>, &MeshImage)> + '_ {
        self.images.iter()
    }

    /// The texture `id`, or `None` if not one of this state's.
    pub fn texture(&self, id: U32Id<BMeshTexture>) -> Option<&MeshTexture> {
        self.textures.get(id)
    }

    /// Number of textures.
    pub fn texture_count(&self) -> usize {
        self.textures.len()
    }

    /// Textures in listing order, as `(id, texture)`.
    pub fn iter_textures(&self) -> impl Iterator<Item = (U32Id<BMeshTexture>, &MeshTexture)> + '_ {
        self.textures.iter()
    }

    /// The material `id`, or `None` if not one of this state's.
    pub fn material(&self, id: U32Id<BMeshMaterial>) -> Option<&MeshMaterial> {
        self.materials.get(id)
    }

    /// Number of materials.
    pub fn material_count(&self) -> usize {
        self.materials.len()
    }

    /// Materials in listing order, as `(id, material)`.
    pub fn iter_materials(
        &self,
    ) -> impl Iterator<Item = (U32Id<BMeshMaterial>, &MeshMaterial)> + '_ {
        self.materials.iter()
    }

    /// Objects in listing order, as `(id, object)`.
    pub fn iter_objects(&self) -> impl Iterator<Item = (U32Id<BMeshObject>, &MeshObject)> + '_ {
        self.objects.iter()
    }

    /// The object `id`, or `None` if not one of this state's.
    pub fn object(&self, id: U32Id<BMeshObject>) -> Option<&MeshObject> {
        self.objects.get(id)
    }

    /// Number of objects.
    pub fn object_count(&self) -> usize {
        self.objects.len()
    }

    /// The document's roots: hierarchy node ids.
    pub fn root_hierarchy_node_ids(&self) -> &[U32Id<BMeshHierarchyNode>] {
        &self.root_hierarchy_node_ids
    }
}

/// The name of the first property in `properties` that repeats an earlier
/// one.
fn first_duplicate_property_name(properties: &[MeshProperty]) -> Option<&str> {
    let mut seen = HashSet::with_capacity(properties.len());

    properties
        .iter()
        .map(|property| property.name.as_str())
        .find(|name| !seen.insert(*name))
}

/// The name of the first property in `properties` holding a non-finite
/// float.
fn first_non_finite_property(properties: &[MeshProperty]) -> Option<&str> {
    properties
        .iter()
        .find(|property| !property.value.is_finite())
        .map(|property| property.name.as_str())
}
