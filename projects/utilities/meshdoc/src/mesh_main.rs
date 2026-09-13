use crate::{
    BMeshFile, BMeshHierarchyNode, BMeshImage, BMeshMaterial, BMeshObject, BMeshPrimitive,
    BMeshTexture, Error, MeshExt, MeshFile, MeshGcRemap, MeshHierarchyNode, MeshImage,
    MeshImageSource, MeshMaterial, MeshObject, MeshPrimitive, MeshProperty, MeshState, MeshTexture,
    Result, TakenExt, mesh_state::first_cycle_node_index,
};
use branded_id::{IdVec, U32Id, soa::IdRemap};
use std::collections::{HashMap, HashSet};

/// The in-memory state of a mesh model.
///
/// Ids are meaningful only within this state. Every mutation checks the
/// cross-references it could break, so a state reached through the public API
/// never violates a referential rule. The read API lives on [`MeshState`],
/// reached through [`state`](Self::state) and forwarded here.
///
/// `T` is the ext carried alongside the document, a [`MeshExt`]. The core
/// never reads it. A mutation that retains, releases, or renumbers tells the
/// ext through its hooks. A hook error surfaces as the mutation's error.
#[derive(Debug, Default)]
pub struct MeshMain<T = ()> {
    /// The document.
    state: MeshState,

    /// The user extension.
    ext: T,
}

impl<T: MeshExt> MeshMain<T> {
    /// Takes the ext off the state, leaving the document as a bare state. A
    /// conversion that drops or replaces a foreign ext makes the drop
    /// explicit here.
    pub fn take_ext(self) -> TakenExt<T> {
        TakenExt {
            main: MeshMain {
                state: self.state,
                ext: (),
            },
            ext: self.ext,
        }
    }

    /// The document, for reading. The hooks see the same value.
    pub fn state(&self) -> &MeshState {
        &self.state
    }

    /// Forwards [`MeshState::validate`].
    pub fn validate(&self) -> Result<()> {
        self.state.validate()
    }

    /// Forwards [`MeshState::hierarchy_node`].
    pub fn hierarchy_node(&self, id: U32Id<BMeshHierarchyNode>) -> Option<&MeshHierarchyNode> {
        self.state.hierarchy_node(id)
    }

    /// Forwards [`MeshState::hierarchy_node_count`].
    pub fn hierarchy_node_count(&self) -> usize {
        self.state.hierarchy_node_count()
    }

    /// Forwards [`MeshState::iter_hierarchy_nodes`].
    pub fn iter_hierarchy_nodes(
        &self,
    ) -> impl Iterator<Item = (U32Id<BMeshHierarchyNode>, &MeshHierarchyNode)> + '_ {
        self.state.iter_hierarchy_nodes()
    }

    /// Forwards [`MeshState::file`].
    pub fn file(&self, id: U32Id<BMeshFile>) -> Option<&MeshFile> {
        self.state.file(id)
    }

    /// Forwards [`MeshState::file_by_name`].
    pub fn file_by_name(&self, name: &str) -> Option<(U32Id<BMeshFile>, &MeshFile)> {
        self.state.file_by_name(name)
    }

    /// Forwards [`MeshState::file_count`].
    pub fn file_count(&self) -> usize {
        self.state.file_count()
    }

    /// Forwards [`MeshState::iter_files`].
    pub fn iter_files(&self) -> impl Iterator<Item = (U32Id<BMeshFile>, &MeshFile)> + '_ {
        self.state.iter_files()
    }

    /// Forwards [`MeshState::image`].
    pub fn image(&self, id: U32Id<BMeshImage>) -> Option<&MeshImage> {
        self.state.image(id)
    }

    /// Forwards [`MeshState::image_bytes`].
    pub fn image_bytes(&self, id: U32Id<BMeshImage>) -> Option<&[u8]> {
        self.state.image_bytes(id)
    }

    /// Forwards [`MeshState::image_count`].
    pub fn image_count(&self) -> usize {
        self.state.image_count()
    }

    /// Forwards [`MeshState::iter_images`].
    pub fn iter_images(&self) -> impl Iterator<Item = (U32Id<BMeshImage>, &MeshImage)> + '_ {
        self.state.iter_images()
    }

    /// Forwards [`MeshState::texture`].
    pub fn texture(&self, id: U32Id<BMeshTexture>) -> Option<&MeshTexture> {
        self.state.texture(id)
    }

    /// Forwards [`MeshState::texture_count`].
    pub fn texture_count(&self) -> usize {
        self.state.texture_count()
    }

    /// Forwards [`MeshState::iter_textures`].
    pub fn iter_textures(&self) -> impl Iterator<Item = (U32Id<BMeshTexture>, &MeshTexture)> + '_ {
        self.state.iter_textures()
    }

    /// Forwards [`MeshState::material`].
    pub fn material(&self, id: U32Id<BMeshMaterial>) -> Option<&MeshMaterial> {
        self.state.material(id)
    }

    /// Forwards [`MeshState::material_count`].
    pub fn material_count(&self) -> usize {
        self.state.material_count()
    }

    /// Forwards [`MeshState::iter_materials`].
    pub fn iter_materials(
        &self,
    ) -> impl Iterator<Item = (U32Id<BMeshMaterial>, &MeshMaterial)> + '_ {
        self.state.iter_materials()
    }

    /// Forwards [`MeshState::iter_objects`].
    pub fn iter_objects(&self) -> impl Iterator<Item = (U32Id<BMeshObject>, &MeshObject)> + '_ {
        self.state.iter_objects()
    }

    /// Forwards [`MeshState::object`].
    pub fn object(&self, id: U32Id<BMeshObject>) -> Option<&MeshObject> {
        self.state.object(id)
    }

    /// Forwards [`MeshState::object_count`].
    pub fn object_count(&self) -> usize {
        self.state.object_count()
    }

    /// Forwards [`MeshState::root_hierarchy_node_ids`].
    pub fn root_hierarchy_node_ids(&self) -> &[U32Id<BMeshHierarchyNode>] {
        self.state.root_hierarchy_node_ids()
    }

    /// The user extension.
    pub fn ext(&self) -> &T {
        &self.ext
    }

    /// The user extension, mutably.
    pub fn ext_mut(&mut self) -> &mut T {
        &mut self.ext
    }

    /// Compacts every id pool back to a contiguous `0..len` in listing order
    /// and rewrites every cross-reference to match, so a state edited by
    /// releases and moves numbers its entities the way a freshly loaded one
    /// does, keeping saves deterministic. Call it once before saving, not after
    /// each release or move.
    ///
    /// Returns the [`MeshGcRemap`] recording where each id moved, so any ids
    /// held outside the state can be translated to their compacted values.
    /// The ext sees the same remap through [`did_gc`](MeshExt::did_gc).
    /// Errors only if the ext does.
    pub fn gc(&mut self) -> Result<MeshGcRemap> {
        // Compact the files, then relabel every image source and property
        // pointing at one.
        let file_remap = self.state.file_ids.gc();
        // Safety: the file column was in sync with the pre-gc id pool, and
        // nothing has retained or released since.
        unsafe { self.state.files.gc(&file_remap) };

        for image_id in self.state.image_ids.iter().collect::<Vec<_>>() {
            // Safety: retained image ids have a value.
            let image = unsafe { self.state.images.get_mut(image_id) };
            if let MeshImageSource::File(file_id) = &mut image.source {
                *file_id = file_remap
                    .new_id(*file_id)
                    .expect("an image reads a live file in a valid state");
            }
        }

        for material_id in self.state.material_ids.iter().collect::<Vec<_>>() {
            // Safety: retained material ids have a value.
            unsafe { self.state.materials.get_mut(material_id) }.relabel_files(&file_remap);
        }

        for object_id in self.state.object_ids.iter().collect::<Vec<_>>() {
            // Safety: retained object ids have a value.
            unsafe { self.state.objects.get_mut(object_id) }.relabel_files(&file_remap);
        }

        // Compact the images, then relabel every texture's image before the
        // textures move.
        let image_remap = self.state.image_ids.gc();
        // Safety: the image column was in sync with the pre-gc id pool, and
        // nothing has retained or released since.
        unsafe { self.state.images.gc(&image_remap) };

        for texture_id in self.state.texture_ids.iter().collect::<Vec<_>>() {
            // Safety: retained texture ids have a value.
            let texture = unsafe { self.state.textures.get_mut(texture_id) };
            texture.image_id = image_remap
                .new_id(texture.image_id)
                .expect("a texture samples a live image in a valid state");
        }

        // Compact the textures, then relabel every material's and object's
        // textures.
        let texture_remap = self.state.texture_ids.gc();
        // Safety: as above, for the texture column.
        unsafe { self.state.textures.gc(&texture_remap) };

        for material_id in self.state.material_ids.iter().collect::<Vec<_>>() {
            // Safety: retained material ids have a value.
            let material = unsafe { self.state.materials.get_mut(material_id) };
            material.relabel_textures(&texture_remap);
        }

        for object_id in self.state.object_ids.iter().collect::<Vec<_>>() {
            // Safety: retained object ids have a value.
            unsafe { self.state.objects.get_mut(object_id) }.relabel_textures(&texture_remap);
        }

        // Compact the materials, then relabel every primitive's material and
        // compact each object's primitive pool. Because the primitive
        // relabelings are indexed by old object id, the column covers the
        // object id pool's whole id space.
        let material_remap = self.state.material_ids.gc();
        // Safety: as above, for the material column.
        unsafe { self.state.materials.gc(&material_remap) };

        let object_id_space = self.state.object_ids.peek_next_fresh().to_u32() as usize;
        let mut primitive_remaps =
            IdVec::from_vec((0..object_id_space).map(|_| IdRemap::default()).collect());

        for object_id in self.state.object_ids.iter().collect::<Vec<_>>() {
            // Safety: retained object ids have a value.
            let object = unsafe { self.state.objects.get_mut(object_id) };
            let primitive_ids: Vec<_> = object.iter_primitives().map(|(id, _)| id).collect();

            for primitive_id in primitive_ids {
                let primitive = object
                    .primitive_mut(primitive_id)
                    .expect("an iterated primitive is one of the object's");

                if let Some(material_id) = primitive.material_id() {
                    let new_material_id = material_remap
                        .new_id(material_id)
                        .expect("a primitive draws a live material in a valid state");
                    primitive.set_material_id(Some(new_material_id));
                }
            }

            primitive_remaps[object_id.to_usize_id()] = object.gc();
        }

        // Compact the object id pool.
        let object_remap = self.state.object_ids.gc();
        // Safety: as above, for the object column.
        unsafe { self.state.objects.gc(&object_remap) };

        // Compact the node id pool, then translate child links and roots,
        // which point at the relabeled nodes and objects.
        let node_remap = self.state.hierarchy_node_ids.gc();
        // Safety: as above, for the node column.
        unsafe { self.state.hierarchy_nodes.gc(&node_remap) };

        let node_ids: Vec<_> = self.state.hierarchy_node_ids.iter().collect();
        for node_id in node_ids {
            // Safety: retained node ids have a value.
            let node = unsafe { self.state.hierarchy_nodes.get_mut(node_id) };
            for child_id in &mut node.child_node_ids {
                *child_id = node_remap
                    .new_id(*child_id)
                    .expect("a child node is live in a valid state");
            }

            for object_id in &mut node.child_object_ids {
                *object_id = object_remap
                    .new_id(*object_id)
                    .expect("a child object is live in a valid state");
            }
        }

        for root_id in &mut self.state.root_hierarchy_node_ids {
            *root_id = node_remap
                .new_id(*root_id)
                .expect("a root is live in a valid state");
        }

        let remap = MeshGcRemap {
            files: file_remap,
            images: image_remap,
            textures: texture_remap,
            materials: material_remap,
            objects: object_remap,
            primitives: primitive_remaps,
            hierarchy_nodes: node_remap,
        };
        self.ext.did_gc(&self.state, &remap)?;
        Ok(remap)
    }

    /// Retains a hierarchy node at the end of the listing, returning its id.
    /// The node's id is fresh to every existing child list, so a node whose
    /// children are already live can never close a cycle. For a batch whose
    /// nodes reference each other, use
    /// [`retain_hierarchy_nodes`](Self::retain_hierarchy_nodes). Errors,
    /// changing nothing, if:
    ///
    /// 1. a child node or child object is not one of this state's
    /// 2. a child repeats
    /// 3. the transform is malformed
    pub fn retain_hierarchy_node(
        &mut self,
        node: MeshHierarchyNode,
    ) -> Result<U32Id<BMeshHierarchyNode>> {
        self.state.check_inserted_node(&node, 0, &HashSet::new())?;

        let node_id = self.state.hierarchy_node_ids.retain();
        self.state.hierarchy_nodes.retain(node_id, node);
        self.ext.hierarchy_node_did_retain(&self.state, node_id)?;
        Ok(node_id)
    }

    /// Retains a batch of hierarchy nodes at the end of the listing, assigning
    /// ids in listing order and returning them. A node's children may
    /// reference any already-live node or any node in the batch by the id it
    /// will take, so a listing with forward references loads in one call.
    /// Errors, changing nothing, if:
    ///
    /// 1. a child resolves to neither
    /// 2. a child repeats within a node
    /// 3. a transform is malformed
    /// 4. the batch's `child_node_ids` edges form a cycle
    pub fn retain_hierarchy_nodes(
        &mut self,
        nodes: Vec<MeshHierarchyNode>,
    ) -> Result<Vec<U32Id<BMeshHierarchyNode>>> {
        // The ids the batch will take, computed before any of it is inserted
        // so every check runs before any mutation.
        let prospective_ids: Vec<U32Id<BMeshHierarchyNode>> = (0..nodes.len())
            .map(|index| self.state.hierarchy_node_ids.peek_nth(index))
            .collect();

        let batch_ids: HashSet<U32Id<BMeshHierarchyNode>> =
            prospective_ids.iter().copied().collect();

        for (node_index, node) in nodes.iter().enumerate() {
            self.state
                .check_inserted_node(node, node_index, &batch_ids)?;
        }

        // An edge leaving the batch lands on an already-live node, whose
        // children are frozen and reference only other live nodes, so it can
        // never lead back in. Only the batch-internal edges can cycle.
        let index_of: HashMap<U32Id<BMeshHierarchyNode>, usize> = prospective_ids
            .iter()
            .enumerate()
            .map(|(node_index, &node_id)| (node_id, node_index))
            .collect();

        let children: Vec<&[U32Id<BMeshHierarchyNode>]> = nodes
            .iter()
            .map(|node| node.child_node_ids.as_slice())
            .collect();

        if let Some(node_index) = first_cycle_node_index(&children, &index_of) {
            return Err(Error::InsertedCycle { index: node_index });
        }

        let ids: Vec<U32Id<BMeshHierarchyNode>> = nodes
            .into_iter()
            .map(|node| {
                let node_id = self.state.hierarchy_node_ids.retain();
                self.state.hierarchy_nodes.retain(node_id, node);
                node_id
            })
            .collect();

        debug_assert_eq!(
            ids, prospective_ids,
            "the id pool assigned the predicted ids"
        );

        for &node_id in &ids {
            self.ext.hierarchy_node_did_retain(&self.state, node_id)?;
        }

        Ok(ids)
    }

    /// Releases hierarchy node `id`. Leaves a hole until [`gc`](Self::gc)
    /// renumbers. Errors, changing nothing, if:
    ///
    /// 1. `id` is not one of this state's nodes
    /// 2. a node still lists it as a child or the roots still list it; release
    ///    the parents first and drop it from the roots with
    ///    [`set_root_hierarchy_node_ids`](Self::set_root_hierarchy_node_ids)
    pub fn release_hierarchy_node(&mut self, id: U32Id<BMeshHierarchyNode>) -> Result<()> {
        if !self.state.hierarchy_node_ids.is_retained(id) {
            return Err(Error::UnknownHierarchyNode { node_id: id });
        }

        let parent_ids: Vec<_> = self
            .iter_hierarchy_nodes()
            .filter(|(_, node)| node.child_node_ids.contains(&id))
            .map(|(node_id, _)| node_id)
            .collect();

        let root = self.state.root_hierarchy_node_ids.contains(&id);
        if !parent_ids.is_empty() || root {
            return Err(Error::HierarchyNodeInUse {
                node_id: id,
                parent_ids,
                root,
            });
        }

        self.ext.hierarchy_node_will_release(&self.state, id)?;

        // Safety: a retained node id has a value.
        unsafe { self.state.hierarchy_nodes.release(id) };
        self.state.hierarchy_node_ids.release_stable(id);
        Ok(())
    }

    /// Replaces hierarchy node `id` with `node`, keeping its id, its listing
    /// position, its parents, and its place in the roots. Errors, changing
    /// nothing, if:
    ///
    /// 1. `id` is not one of this state's nodes
    /// 2. a child node or child object is not one of this state's
    /// 3. a child repeats
    /// 4. the transform is malformed
    /// 5. a child node reaches `id` through `child_node_ids`, closing a cycle
    ///
    /// An error reports `node` as a batch of one, at listing index `0`.
    pub fn set_hierarchy_node(
        &mut self,
        id: U32Id<BMeshHierarchyNode>,
        node: MeshHierarchyNode,
    ) -> Result<()> {
        if !self.state.hierarchy_node_ids.is_retained(id) {
            return Err(Error::UnknownHierarchyNode { node_id: id });
        }

        self.state.check_inserted_node(&node, 0, &HashSet::new())?;

        if self.state.reaches_hierarchy_node(&node.child_node_ids, id) {
            return Err(Error::InsertedCycle { index: 0 });
        }

        // Safety: a retained node id has a value.
        *unsafe { self.state.hierarchy_nodes.get_mut(id) } = node;
        Ok(())
    }

    /// Retains a file at the end of the listing, returning its id. Errors,
    /// changing nothing, if its name is empty or another file has it.
    pub fn retain_file(&mut self, file: MeshFile) -> Result<U32Id<BMeshFile>> {
        let file_id = self.state.file_ids.peek_next();

        self.state.check_inserted_file(file_id, &file)?;

        let retained_id = self.state.file_ids.retain();

        debug_assert_eq!(
            retained_id, file_id,
            "the id pool assigned the predicted id"
        );

        self.state.files.retain(file_id, file);
        self.ext.file_did_retain(&self.state, file_id)?;
        Ok(file_id)
    }

    /// Releases file `id`. Leaves a hole until [`gc`](Self::gc) renumbers.
    /// Errors, changing nothing, if:
    ///
    /// 1. `id` is not one of this state's files
    /// 2. an image still reads it, or a material or object property still
    ///    points at it; release or repoint those first
    pub fn release_file(&mut self, id: U32Id<BMeshFile>) -> Result<()> {
        if !self.state.file_ids.is_retained(id) {
            return Err(Error::UnknownFile { file_id: id });
        }

        let image_ids: Vec<_> = self
            .iter_images()
            .filter(|(_, image)| image.source == MeshImageSource::File(id))
            .map(|(image_id, _)| image_id)
            .collect();

        let material_ids: Vec<_> = self
            .iter_materials()
            .filter(|(_, material)| material.iter_file_ids().any(|file_id| file_id == id))
            .map(|(material_id, _)| material_id)
            .collect();

        let object_ids: Vec<_> = self
            .iter_objects()
            .filter(|(_, object)| object.iter_file_ids().any(|file_id| file_id == id))
            .map(|(object_id, _)| object_id)
            .collect();

        if !image_ids.is_empty() || !material_ids.is_empty() || !object_ids.is_empty() {
            return Err(Error::FileInUse {
                file_id: id,
                image_ids,
                material_ids,
                object_ids,
            });
        }

        self.ext.file_will_release(&self.state, id)?;

        // Safety: a retained file id has a value.
        unsafe { self.state.files.release(id) };
        self.state.file_ids.release_stable(id);
        Ok(())
    }

    /// Moves file `id` to position `index` in the listing, shifting the
    /// files between its old and new positions one slot. Errors, changing
    /// nothing, if `id` is not one of this state's files or `index` is at or
    /// past [`file_count`](Self::file_count).
    pub fn move_file(&mut self, id: U32Id<BMeshFile>, index: usize) -> Result<()> {
        if !self.state.file_ids.is_retained(id) {
            return Err(Error::UnknownFile { file_id: id });
        }

        let count = self.state.file_count();
        if index >= count {
            return Err(Error::IndexPastCount { index, count });
        }

        self.state.file_ids.move_to(id, index);
        Ok(())
    }

    /// Replaces file `id` with `file`, keeping its id and everything
    /// referencing it. Errors, changing nothing, if:
    ///
    /// 1. `id` is not one of this state's files
    /// 2. `file` fails a [`retain_file`](Self::retain_file) check
    /// 3. an image reading it would no longer start with its media type's
    ///    signature
    pub fn set_file(&mut self, id: U32Id<BMeshFile>, file: MeshFile) -> Result<()> {
        if !self.state.file_ids.is_retained(id) {
            return Err(Error::UnknownFile { file_id: id });
        }

        self.state.check_inserted_file(id, &file)?;

        // Safety: a retained file id has a value.
        *unsafe { self.state.files.get_mut(id) } = file;
        Ok(())
    }

    /// Retains an image at the end of the listing, returning its id. Errors,
    /// changing nothing, if it reads a file that is not one of this state's
    /// or its bytes do not start with its media type's signature.
    pub fn retain_image(&mut self, image: MeshImage) -> Result<U32Id<BMeshImage>> {
        self.state.check_inserted_image(&image)?;

        let image_id = self.state.image_ids.retain();
        self.state.images.retain(image_id, image);
        self.ext.image_did_retain(&self.state, image_id)?;
        Ok(image_id)
    }

    /// Releases image `id`. Leaves a hole until [`gc`](Self::gc) renumbers.
    /// Errors, changing nothing, if:
    ///
    /// 1. `id` is not one of this state's images
    /// 2. a texture still samples it; release those textures first
    pub fn release_image(&mut self, id: U32Id<BMeshImage>) -> Result<()> {
        if !self.state.image_ids.is_retained(id) {
            return Err(Error::UnknownImage { image_id: id });
        }

        let texture_ids: Vec<_> = self
            .iter_textures()
            .filter(|(_, texture)| texture.image_id == id)
            .map(|(texture_id, _)| texture_id)
            .collect();

        if !texture_ids.is_empty() {
            return Err(Error::ImageInUse {
                image_id: id,
                texture_ids,
            });
        }

        self.ext.image_will_release(&self.state, id)?;

        // Safety: a retained image id has a value.
        unsafe { self.state.images.release(id) };
        self.state.image_ids.release_stable(id);
        Ok(())
    }

    /// Moves image `id` to position `index` in the listing, shifting the
    /// images between its old and new positions one slot. Errors, changing
    /// nothing, if `id` is not one of this state's images or `index` is at
    /// or past [`image_count`](Self::image_count).
    pub fn move_image(&mut self, id: U32Id<BMeshImage>, index: usize) -> Result<()> {
        if !self.state.image_ids.is_retained(id) {
            return Err(Error::UnknownImage { image_id: id });
        }

        let count = self.state.image_count();
        if index >= count {
            return Err(Error::IndexPastCount { index, count });
        }

        self.state.image_ids.move_to(id, index);
        Ok(())
    }

    /// Replaces image `id` with `image`, keeping its id and every texture
    /// sampling it. Errors, changing nothing, if `id` is not one of this
    /// state's images or `image` fails a [`retain_image`](Self::retain_image)
    /// check.
    pub fn set_image(&mut self, id: U32Id<BMeshImage>, image: MeshImage) -> Result<()> {
        if !self.state.image_ids.is_retained(id) {
            return Err(Error::UnknownImage { image_id: id });
        }

        self.state.check_inserted_image(&image)?;

        // Safety: a retained image id has a value.
        *unsafe { self.state.images.get_mut(id) } = image;
        Ok(())
    }

    /// Retains a texture at the end of the listing, returning its id. Errors,
    /// changing nothing, if it samples an image that is not one of this
    /// state's.
    pub fn retain_texture(&mut self, texture: MeshTexture) -> Result<U32Id<BMeshTexture>> {
        if self.image(texture.image_id).is_none() {
            return Err(Error::TextureImageRef {
                image_id: texture.image_id,
            });
        }

        let texture_id = self.state.texture_ids.retain();
        self.state.textures.retain(texture_id, texture);
        self.ext.texture_did_retain(&self.state, texture_id)?;
        Ok(texture_id)
    }

    /// Releases texture `id`. Leaves a hole until [`gc`](Self::gc) renumbers.
    /// Errors, changing nothing, if:
    ///
    /// 1. `id` is not one of this state's textures
    /// 2. a material still draws it, or an object property still references
    ///    it; release or replace those first
    pub fn release_texture(&mut self, id: U32Id<BMeshTexture>) -> Result<()> {
        if !self.state.texture_ids.is_retained(id) {
            return Err(Error::UnknownTexture { texture_id: id });
        }

        let material_ids: Vec<_> = self
            .iter_materials()
            .filter(|(_, material)| {
                material
                    .iter_texture_refs()
                    .any(|texture_ref| texture_ref.texture_id == id)
            })
            .map(|(material_id, _)| material_id)
            .collect();

        let object_ids: Vec<_> = self
            .iter_objects()
            .filter(|(_, object)| {
                object
                    .iter_texture_refs()
                    .any(|texture_ref| texture_ref.texture_id == id)
            })
            .map(|(object_id, _)| object_id)
            .collect();

        if !material_ids.is_empty() || !object_ids.is_empty() {
            return Err(Error::TextureInUse {
                texture_id: id,
                material_ids,
                object_ids,
            });
        }

        self.ext.texture_will_release(&self.state, id)?;

        // Safety: a retained texture id has a value.
        unsafe { self.state.textures.release(id) };
        self.state.texture_ids.release_stable(id);
        Ok(())
    }

    /// Moves texture `id` to position `index` in the listing, shifting the
    /// textures between its old and new positions one slot. Errors, changing
    /// nothing, if `id` is not one of this state's textures or `index` is at
    /// or past [`texture_count`](Self::texture_count).
    pub fn move_texture(&mut self, id: U32Id<BMeshTexture>, index: usize) -> Result<()> {
        if !self.state.texture_ids.is_retained(id) {
            return Err(Error::UnknownTexture { texture_id: id });
        }

        let count = self.state.texture_count();
        if index >= count {
            return Err(Error::IndexPastCount { index, count });
        }

        self.state.texture_ids.move_to(id, index);
        Ok(())
    }

    /// Replaces texture `id` with `texture`, keeping its id and every
    /// material drawing it. Errors, changing nothing, if `id` is not one of
    /// this state's textures or `texture` samples an image that is not one
    /// of this state's.
    pub fn set_texture(&mut self, id: U32Id<BMeshTexture>, texture: MeshTexture) -> Result<()> {
        if !self.state.texture_ids.is_retained(id) {
            return Err(Error::UnknownTexture { texture_id: id });
        }

        if self.image(texture.image_id).is_none() {
            return Err(Error::TextureImageRef {
                image_id: texture.image_id,
            });
        }

        // Safety: a retained texture id has a value.
        *unsafe { self.state.textures.get_mut(id) } = texture;
        Ok(())
    }

    /// Retains a material at the end of the listing, returning its id.
    /// Errors, changing nothing, if:
    ///
    /// 1. a factor is outside the range its name fixes
    /// 2. a texture drawn is not one of this state's
    /// 3. a property name repeats, or a property holds a non-finite float
    pub fn retain_material(&mut self, material: MeshMaterial) -> Result<U32Id<BMeshMaterial>> {
        self.state.check_inserted_material(&material)?;

        let material_id = self.state.material_ids.retain();
        self.state.materials.retain(material_id, material);
        self.ext.material_did_retain(&self.state, material_id)?;
        Ok(material_id)
    }

    /// Releases material `id`. Leaves a hole until [`gc`](Self::gc)
    /// renumbers. Errors, changing nothing, if:
    ///
    /// 1. `id` is not one of this state's materials
    /// 2. a primitive still draws with it; repoint those primitives first
    ///    with [`set_primitive_material_id`](Self::set_primitive_material_id)
    pub fn release_material(&mut self, id: U32Id<BMeshMaterial>) -> Result<()> {
        if !self.state.material_ids.is_retained(id) {
            return Err(Error::UnknownMaterial { material_id: id });
        }

        let object_ids: Vec<_> = self
            .iter_objects()
            .filter(|(_, object)| {
                object
                    .iter_primitives()
                    .any(|(_, primitive)| primitive.material_id() == Some(id))
            })
            .map(|(object_id, _)| object_id)
            .collect();

        if !object_ids.is_empty() {
            return Err(Error::MaterialInUse {
                material_id: id,
                object_ids,
            });
        }

        self.ext.material_will_release(&self.state, id)?;

        // Safety: a retained material id has a value.
        unsafe { self.state.materials.release(id) };
        self.state.material_ids.release_stable(id);
        Ok(())
    }

    /// Moves material `id` to position `index` in the listing, shifting the
    /// materials between its old and new positions one slot. Errors,
    /// changing nothing, if `id` is not one of this state's materials or
    /// `index` is at or past [`material_count`](Self::material_count).
    pub fn move_material(&mut self, id: U32Id<BMeshMaterial>, index: usize) -> Result<()> {
        if !self.state.material_ids.is_retained(id) {
            return Err(Error::UnknownMaterial { material_id: id });
        }

        let count = self.state.material_count();
        if index >= count {
            return Err(Error::IndexPastCount { index, count });
        }

        self.state.material_ids.move_to(id, index);
        Ok(())
    }

    /// Replaces material `id` with `material`, keeping its id and every
    /// primitive drawing with it. Errors, changing nothing, if:
    ///
    /// 1. `id` is not one of this state's materials
    /// 2. `material` fails a [`retain_material`](Self::retain_material) check
    /// 3. a primitive drawing with it does not carry a UV stream one of
    ///    `material`'s textures samples
    pub fn set_material(&mut self, id: U32Id<BMeshMaterial>, material: MeshMaterial) -> Result<()> {
        if !self.state.material_ids.is_retained(id) {
            return Err(Error::UnknownMaterial { material_id: id });
        }

        self.state.check_inserted_material(&material)?;

        for (_, object) in self.iter_objects() {
            for (primitive_id, primitive) in object.iter_primitives() {
                if primitive.material_id() != Some(id) {
                    continue;
                }

                for texture_ref in material.iter_texture_refs() {
                    if primitive.uv_stream(texture_ref.uv_stream_id).is_none() {
                        return Err(Error::PrimitiveUvStreamRef {
                            primitive_id,
                            material_id: id,
                            uv_stream_id: texture_ref.uv_stream_id,
                        });
                    }
                }
            }
        }

        // Safety: a retained material id has a value.
        *unsafe { self.state.materials.get_mut(id) } = material;
        Ok(())
    }

    /// Retains an object at the end of the listing, returning its id. Errors,
    /// changing nothing, if:
    ///
    /// 1. its properties fail a
    ///    [`set_object_properties`](Self::set_object_properties) check
    /// 2. a primitive draws with a material that is not one of this state's
    ///    or does not carry a UV stream one of that material's textures
    ///    samples
    pub fn retain_object(&mut self, object: MeshObject) -> Result<U32Id<BMeshObject>> {
        self.state.check_properties(object.properties())?;

        for (primitive_id, primitive) in object.iter_primitives() {
            self.state.check_primitive_material(
                primitive_id,
                primitive,
                primitive.material_id(),
            )?;
        }

        let object_id = self.state.object_ids.retain();
        self.state.objects.retain(object_id, object);
        self.ext.object_did_retain(&self.state, object_id)?;
        Ok(object_id)
    }

    /// Releases object `id`. Leaves a hole until [`gc`](Self::gc) renumbers
    /// for a deterministic save. Errors, changing nothing, if:
    ///
    /// 1. `id` is not one of this state's objects
    /// 2. a hierarchy node still places it; release those nodes first
    pub fn release_object(&mut self, id: U32Id<BMeshObject>) -> Result<()> {
        if !self.state.object_ids.is_retained(id) {
            return Err(Error::UnknownObject { object_id: id });
        }

        let node_ids: Vec<_> = self
            .iter_hierarchy_nodes()
            .filter(|(_, node)| node.child_object_ids.contains(&id))
            .map(|(node_id, _)| node_id)
            .collect();

        if !node_ids.is_empty() {
            return Err(Error::ObjectInUse {
                object_id: id,
                node_ids,
            });
        }

        self.ext.object_will_release(&self.state, id)?;

        // Safety: a retained object id has a value.
        unsafe { self.state.objects.release(id) };
        self.state.object_ids.release_stable(id);
        Ok(())
    }

    /// Moves object `id` to position `index` in the listing, shifting the
    /// objects between its old and new positions one slot. Errors, changing
    /// nothing, if `id` is not one of this state's objects or `index` is at
    /// or past [`object_count`](Self::object_count).
    pub fn move_object(&mut self, id: U32Id<BMeshObject>, index: usize) -> Result<()> {
        if !self.state.object_ids.is_retained(id) {
            return Err(Error::UnknownObject { object_id: id });
        }

        let count = self.state.object_count();
        if index >= count {
            return Err(Error::IndexPastCount { index, count });
        }

        self.state.object_ids.move_to(id, index);
        Ok(())
    }

    /// Sets the name of object `object_id`. Errors, changing nothing, if
    /// `object_id` is not one of this state's.
    pub fn set_object_name(&mut self, object_id: U32Id<BMeshObject>, name: String) -> Result<()> {
        if !self.state.object_ids.is_retained(object_id) {
            return Err(Error::UnknownObject { object_id });
        }

        // Safety: the object id is retained.
        unsafe { self.state.objects.get_mut(object_id) }.set_name(name);
        Ok(())
    }

    /// Replaces the properties of object `object_id`. Errors, changing
    /// nothing, if:
    ///
    /// 1. `object_id` is not one of this state's
    /// 2. a property name repeats, or a property holds a non-finite float
    /// 3. a property references a texture or file that is not one of this
    ///    state's
    pub fn set_object_properties(
        &mut self,
        object_id: U32Id<BMeshObject>,
        properties: Vec<MeshProperty>,
    ) -> Result<()> {
        if !self.state.object_ids.is_retained(object_id) {
            return Err(Error::UnknownObject { object_id });
        }

        self.state.check_properties(&properties)?;

        // Safety: the object id is retained.
        unsafe { self.state.objects.get_mut(object_id) }.set_properties(properties);
        Ok(())
    }

    /// Retains a primitive to object `object_id`, after its existing
    /// primitives, and returns the primitive's id. Errors, changing nothing,
    /// if:
    ///
    /// 1. `object_id` is not one of this state's
    /// 2. the primitive draws with a material that is not one of this state's
    /// 3. the primitive does not carry a UV stream one of that material's
    ///    textures samples
    pub fn retain_primitive(
        &mut self,
        object_id: U32Id<BMeshObject>,
        primitive: MeshPrimitive,
    ) -> Result<U32Id<BMeshPrimitive>> {
        if !self.state.object_ids.is_retained(object_id) {
            return Err(Error::UnknownObject { object_id });
        }

        // Safety: the object id is retained.
        let object = unsafe { self.state.objects.get(object_id) };
        let primitive_id = object.peek_next_primitive_id();

        self.state
            .check_primitive_material(primitive_id, &primitive, primitive.material_id())?;

        // Safety: the object id is retained.
        let retained_id =
            unsafe { self.state.objects.get_mut(object_id) }.retain_primitive(primitive);

        debug_assert_eq!(
            retained_id, primitive_id,
            "the id pool assigned the predicted id"
        );

        self.ext
            .primitive_did_retain(&self.state, object_id, primitive_id)?;
        Ok(primitive_id)
    }

    /// Releases primitive `primitive_id` from object `object_id`. Leaves a
    /// hole until [`gc`](Self::gc) renumbers. Errors, changing nothing, if
    /// `object_id` is not one of this state's or `primitive_id` is not one of
    /// the object's.
    pub fn release_primitive(
        &mut self,
        object_id: U32Id<BMeshObject>,
        primitive_id: U32Id<BMeshPrimitive>,
    ) -> Result<()> {
        if !self.state.object_ids.is_retained(object_id) {
            return Err(Error::UnknownObject { object_id });
        }

        // Safety: the object id is retained.
        if unsafe { self.state.objects.get(object_id) }
            .primitive(primitive_id)
            .is_none()
        {
            return Err(Error::UnknownPrimitive { primitive_id });
        }

        self.ext
            .primitive_will_release(&self.state, object_id, primitive_id)?;

        // Safety: the object id is retained; the primitive was checked.
        unsafe { self.state.objects.get_mut(object_id) }.release_primitive(primitive_id)
    }

    /// Moves primitive `primitive_id` of object `object_id` to position
    /// `index` in its listing. Errors, changing nothing, if:
    ///
    /// 1. `object_id` is not one of this state's
    /// 2. `primitive_id` is not one of the object's
    /// 3. `index` is at or past its primitive count
    pub fn move_primitive(
        &mut self,
        object_id: U32Id<BMeshObject>,
        primitive_id: U32Id<BMeshPrimitive>,
        index: usize,
    ) -> Result<()> {
        if !self.state.object_ids.is_retained(object_id) {
            return Err(Error::UnknownObject { object_id });
        }

        // Safety: the object id is retained.
        unsafe { self.state.objects.get_mut(object_id) }.move_primitive(primitive_id, index)
    }

    /// Sets the material primitive `primitive_id` of object `object_id` draws
    /// with. Errors, changing nothing, if:
    ///
    /// 1. `object_id` is not one of this state's
    /// 2. `primitive_id` is not one of the object's
    /// 3. `material_id` is not one of this state's
    /// 4. the primitive does not carry a UV stream one of the material's
    ///    textures samples
    pub fn set_primitive_material_id(
        &mut self,
        object_id: U32Id<BMeshObject>,
        primitive_id: U32Id<BMeshPrimitive>,
        material_id: Option<U32Id<BMeshMaterial>>,
    ) -> Result<()> {
        if !self.state.object_ids.is_retained(object_id) {
            return Err(Error::UnknownObject { object_id });
        }

        // Safety: the object id is retained.
        let Some(primitive) = unsafe { self.state.objects.get(object_id) }.primitive(primitive_id)
        else {
            return Err(Error::UnknownPrimitive { primitive_id });
        };

        if let Some(material_id) = material_id
            && self.material(material_id).is_none()
        {
            return Err(Error::UnknownMaterial { material_id });
        }

        self.state
            .check_primitive_material(primitive_id, primitive, material_id)?;

        // Safety: the object id is retained; the primitive was checked.
        unsafe { self.state.objects.get_mut(object_id) }
            .primitive_mut(primitive_id)
            .expect("the primitive was checked")
            .set_material_id(material_id);
        Ok(())
    }

    /// Appends a root. Errors, changing nothing, if `root_id` is not one of
    /// this state's nodes or is already a root.
    pub fn push_root_hierarchy_node_id(
        &mut self,
        root_id: U32Id<BMeshHierarchyNode>,
    ) -> Result<()> {
        if self.hierarchy_node(root_id).is_none() {
            return Err(Error::Root { root_id });
        }

        if self.state.root_hierarchy_node_ids.contains(&root_id) {
            return Err(Error::DuplicateRoot { root_id });
        }

        self.state.root_hierarchy_node_ids.push(root_id);
        Ok(())
    }

    /// Replaces the document's roots. Errors, changing nothing, if a root is
    /// not one of this state's nodes or repeats.
    pub fn set_root_hierarchy_node_ids(
        &mut self,
        root_ids: Vec<U32Id<BMeshHierarchyNode>>,
    ) -> Result<()> {
        let mut seen_ids = HashSet::with_capacity(root_ids.len());
        for &root_id in &root_ids {
            if self.hierarchy_node(root_id).is_none() {
                return Err(Error::Root { root_id });
            }

            if !seen_ids.insert(root_id) {
                return Err(Error::DuplicateRoot { root_id });
            }
        }

        self.state.root_hierarchy_node_ids = root_ids;
        Ok(())
    }
}

impl MeshMain<()> {
    /// Puts `ext` on a bare state, moving the document over unchanged. This
    /// is the one way to change a state's ext type, paired with
    /// [`take_ext`] for a state that carries one.
    ///
    /// [`take_ext`]: MeshMain::take_ext
    pub fn put_ext<U>(self, ext: U) -> MeshMain<U> {
        MeshMain {
            state: self.state,
            ext,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        BMeshFile, BMeshHierarchyNode, BMeshMaterial, BMeshObject, BMeshTexture, Error, MeshExt,
        MeshFile, MeshHierarchyNode, MeshImage, MeshImageMediaType, MeshImageSource, MeshMain,
        MeshMaterial, MeshObject, MeshProperty, MeshPropertyValue, MeshTexture, MeshTextureRef,
        PNG_SIGNATURE, png_image, test_main, unit_triangle,
    };
    use branded_id::U32Id;
    use ty_math::{TyQuaternionF64, TyTransformF64, TyVector2F64, TyVector3F64};

    fn node_id(index: u32) -> U32Id<BMeshHierarchyNode> {
        U32Id::from_u32(index)
    }

    fn object_at(index: u32) -> U32Id<BMeshObject> {
        U32Id::from_u32(index)
    }

    fn material_at(index: u32) -> U32Id<BMeshMaterial> {
        U32Id::from_u32(index)
    }

    fn texture_at(index: u32) -> U32Id<BMeshTexture> {
        U32Id::from_u32(index)
    }

    /// A node referencing the given child nodes and no objects.
    fn node_with_children(child_node_ids: Vec<U32Id<BMeshHierarchyNode>>) -> MeshHierarchyNode {
        MeshHierarchyNode {
            child_node_ids,
            ..MeshHierarchyNode::default()
        }
    }

    /// A node placing the given child objects and no child nodes.
    fn node_with_objects(child_object_ids: Vec<U32Id<BMeshObject>>) -> MeshHierarchyNode {
        MeshHierarchyNode {
            child_object_ids,
            ..MeshHierarchyNode::default()
        }
    }

    /// An object of one triangle drawing `material_id`.
    fn drawing_object(material_id: Option<U32Id<BMeshMaterial>>) -> MeshObject {
        let mut primitive = unit_triangle();
        primitive.set_material_id(material_id);
        let mut object = MeshObject::new("o".to_owned());
        object.retain_primitive(primitive);
        object
    }

    #[test]
    fn the_test_main_validates_and_reads_back() {
        let (main, object_id, material_id) = test_main(());
        main.validate().unwrap();

        assert_eq!(main.image_count(), 1);
        assert_eq!(main.texture_count(), 1);
        assert_eq!(main.material_count(), 1);
        assert_eq!(main.object_count(), 1);
        assert_eq!(main.hierarchy_node_count(), 1);

        let object = main.object(object_id).unwrap();
        assert_eq!(object.name(), "body");
        let (_, primitive) = object.iter_primitives().next().unwrap();
        assert_eq!(primitive.material_id(), Some(material_id));
        assert_eq!(primitive.vertex_count(), 3);
        assert_eq!(primitive.triangle_count(), 1);
        assert_eq!(main.material(material_id).unwrap().name, "skin");
    }

    #[test]
    fn retain_image_rejects_mislabeled_bytes() {
        let mut main: MeshMain = MeshMain::default();

        assert_eq!(
            main.retain_image(MeshImage {
                name: String::new(),
                media_type: MeshImageMediaType::Jpeg,
                source: png_image().source,
            }),
            Err(Error::MalformedImage {
                media_type: MeshImageMediaType::Jpeg
            })
        );
        assert_eq!(main.image_count(), 0);
    }

    #[test]
    fn retain_texture_rejects_a_dangling_image() {
        let mut main: MeshMain = MeshMain::default();

        assert_eq!(
            main.retain_texture(MeshTexture::new(U32Id::from_u32(3))),
            Err(Error::TextureImageRef {
                image_id: U32Id::from_u32(3)
            })
        );
    }

    #[test]
    fn retain_material_rejects_a_bad_factor_a_dangling_texture_and_bad_properties() {
        let mut main: MeshMain = MeshMain::default();

        let material = MeshMaterial {
            metallic_factor: 1.5,
            ..Default::default()
        };
        assert_eq!(
            main.retain_material(material),
            Err(Error::MaterialFactor {
                name: "metallic".to_owned(),
                value: 1.5,
            })
        );

        let material = MeshMaterial {
            normal_texture: Some(MeshTextureRef {
                texture_id: texture_at(4),
                uv_stream_id: U32Id::from_u32(0),
            }),
            ..Default::default()
        };
        assert_eq!(
            main.retain_material(material),
            Err(Error::MaterialTextureRef {
                texture_id: texture_at(4)
            })
        );

        let property = |name: &str, value| MeshProperty {
            name: name.to_owned(),
            value,
        };
        let material = MeshMaterial {
            properties: vec![
                property("a", MeshPropertyValue::Int(1)),
                property("a", MeshPropertyValue::Bool(true)),
            ],
            ..Default::default()
        };
        assert_eq!(
            main.retain_material(material),
            Err(Error::DuplicatePropertyName {
                name: "a".to_owned()
            })
        );

        let material = MeshMaterial {
            properties: vec![property("a", MeshPropertyValue::Float(f64::NAN))],
            ..Default::default()
        };
        assert_eq!(
            main.retain_material(material),
            Err(Error::NonFiniteProperty {
                name: "a".to_owned()
            })
        );

        assert_eq!(main.material_count(), 0);
    }

    #[test]
    fn retain_object_rejects_a_dangling_material_and_a_missing_uv_stream() {
        let mut main: MeshMain = MeshMain::default();

        assert_eq!(
            main.retain_object(drawing_object(Some(material_at(0)))),
            Err(Error::PrimitiveMaterialRef {
                primitive_id: U32Id::from_u32(0),
                material_id: material_at(0),
            })
        );

        let image_id = main.retain_image(png_image()).unwrap();
        let texture_id = main.retain_texture(MeshTexture::new(image_id)).unwrap();
        let material_id = main
            .retain_material(MeshMaterial {
                base_color_texture: Some(MeshTextureRef {
                    texture_id,
                    uv_stream_id: U32Id::from_u32(1),
                }),
                ..Default::default()
            })
            .unwrap();

        // The material samples stream 1, which the one-stream primitive
        // lacks.
        let mut primitive = unit_triangle();
        primitive
            .push_uv_stream(vec![TyVector2F64::ZERO; 3])
            .unwrap();
        primitive.set_material_id(Some(material_id));
        let mut object = MeshObject::new("o".to_owned());
        object.retain_primitive(primitive);

        assert_eq!(
            main.retain_object(object),
            Err(Error::PrimitiveUvStreamRef {
                primitive_id: U32Id::from_u32(0),
                material_id,
                uv_stream_id: U32Id::from_u32(1),
            })
        );
        assert_eq!(main.object_count(), 0);
    }

    #[test]
    fn set_material_checks_the_primitives_drawing_it() {
        let (mut main, _, material_id) = test_main(());

        let texture_id = main.iter_textures().next().unwrap().0;

        // A second stream the one-stream primitive does not carry.
        let material = MeshMaterial {
            base_color_texture: Some(MeshTextureRef {
                texture_id,
                uv_stream_id: U32Id::from_u32(1),
            }),
            ..MeshMaterial::default()
        };

        assert!(matches!(
            main.set_material(material_id, material),
            Err(Error::PrimitiveUvStreamRef { .. })
        ));

        main.set_material(material_id, MeshMaterial::new("plain".to_owned()))
            .unwrap();
        assert_eq!(main.material(material_id).unwrap().name, "plain");
        main.validate().unwrap();
    }

    #[test]
    fn set_primitive_material_id_checks_and_repoints() {
        let (mut main, object_id, material_id) = test_main(());
        let primitive_id = U32Id::from_u32(0);

        assert_eq!(
            main.set_primitive_material_id(object_id, primitive_id, Some(material_at(7))),
            Err(Error::UnknownMaterial {
                material_id: material_at(7)
            })
        );

        main.set_primitive_material_id(object_id, primitive_id, None)
            .unwrap();
        assert_eq!(
            main.object(object_id)
                .unwrap()
                .primitive(primitive_id)
                .unwrap()
                .material_id(),
            None
        );

        main.set_primitive_material_id(object_id, primitive_id, Some(material_id))
            .unwrap();
        main.validate().unwrap();
    }

    #[test]
    fn releases_refuse_while_referenced_and_go_through_once_detached() {
        let (mut main, object_id, material_id) = test_main(());
        let image_id = main.iter_images().next().unwrap().0;
        let texture_id = main.iter_textures().next().unwrap().0;
        let root_id = main.root_hierarchy_node_ids()[0];

        assert_eq!(
            main.release_image(image_id),
            Err(Error::ImageInUse {
                image_id,
                texture_ids: vec![texture_id],
            })
        );
        assert_eq!(
            main.release_texture(texture_id),
            Err(Error::TextureInUse {
                texture_id,
                material_ids: vec![material_id],
                object_ids: Vec::new(),
            })
        );
        assert_eq!(
            main.release_material(material_id),
            Err(Error::MaterialInUse {
                material_id,
                object_ids: vec![object_id],
            })
        );
        assert_eq!(
            main.release_object(object_id),
            Err(Error::ObjectInUse {
                object_id,
                node_ids: vec![root_id],
            })
        );
        assert_eq!(
            main.release_hierarchy_node(root_id),
            Err(Error::HierarchyNodeInUse {
                node_id: root_id,
                parent_ids: Vec::new(),
                root: true,
            })
        );

        main.set_root_hierarchy_node_ids(Vec::new()).unwrap();
        main.release_hierarchy_node(root_id).unwrap();
        main.release_object(object_id).unwrap();
        main.release_material(material_id).unwrap();
        main.release_texture(texture_id).unwrap();
        main.release_image(image_id).unwrap();

        main.validate().unwrap();
        assert_eq!(main.object_count(), 0);
        assert_eq!(main.image_count(), 0);
    }

    #[test]
    fn release_then_gc_renumbers_and_resolves() {
        let mut main: MeshMain = MeshMain::default();

        // Two images, two textures, two materials; the first of each goes.
        let spare_image_id = main.retain_image(png_image()).unwrap();
        let image_id = main.retain_image(png_image()).unwrap();
        let spare_texture_id = main
            .retain_texture(MeshTexture::new(spare_image_id))
            .unwrap();
        let texture_id = main.retain_texture(MeshTexture::new(image_id)).unwrap();
        let spare_material_id = main.retain_material(MeshMaterial::default()).unwrap();
        let material_id = main
            .retain_material(MeshMaterial {
                emissive_texture: Some(MeshTextureRef {
                    texture_id,
                    uv_stream_id: U32Id::from_u32(0),
                }),
                ..Default::default()
            })
            .unwrap();

        // An object of two primitives; the first primitive goes too.
        let mut primitive = unit_triangle();
        primitive
            .push_uv_stream(vec![TyVector2F64::ZERO; 3])
            .unwrap();
        primitive.set_material_id(Some(material_id));
        let mut object = MeshObject::new("o".to_owned());
        let spare_primitive_id = object.retain_primitive(unit_triangle());
        let primitive_id = object.retain_primitive(primitive);
        let spare_object_id = main
            .retain_object(MeshObject::new("spare".to_owned()))
            .unwrap();
        let object_id = main.retain_object(object).unwrap();

        let node_ids = main
            .retain_hierarchy_nodes(vec![
                node_with_objects(vec![spare_object_id]),
                node_with_children(vec![node_id(2)]),
                node_with_objects(vec![object_id]),
            ])
            .unwrap();
        main.set_root_hierarchy_node_ids(vec![node_ids[1]]).unwrap();
        main.validate().unwrap();

        main.release_hierarchy_node(node_ids[0]).unwrap();
        main.release_object(spare_object_id).unwrap();
        main.release_primitive(object_id, spare_primitive_id)
            .unwrap();
        main.release_material(spare_material_id).unwrap();
        main.release_texture(spare_texture_id).unwrap();
        main.release_image(spare_image_id).unwrap();
        main.validate().unwrap();

        let remap = main.gc().unwrap();
        main.validate().unwrap();

        assert_eq!(remap.images.new_id(image_id), Some(U32Id::from_u32(0)));
        assert_eq!(remap.textures.new_id(texture_id), Some(U32Id::from_u32(0)));
        assert_eq!(
            remap.materials.new_id(material_id),
            Some(U32Id::from_u32(0))
        );
        assert_eq!(remap.objects.new_id(object_id), Some(U32Id::from_u32(0)));
        assert_eq!(
            remap.primitives[object_id.to_usize_id()].new_id(primitive_id),
            Some(U32Id::from_u32(0))
        );
        assert_eq!(remap.hierarchy_nodes.new_id(node_ids[2]), Some(node_id(1)));

        // Every cross-reference resolves through the compacted ids.
        assert_eq!(main.texture(texture_at(0)).unwrap().image_id.to_u32(), 0);
        let material = main.material(material_at(0)).unwrap();
        assert_eq!(material.emissive_texture.unwrap().texture_id.to_u32(), 0);
        let object = main.object(object_at(0)).unwrap();
        assert_eq!(object.primitive_count(), 1);
        assert_eq!(
            object.primitive(U32Id::from_u32(0)).unwrap().material_id(),
            Some(material_at(0))
        );
        assert_eq!(main.root_hierarchy_node_ids(), [node_id(0)]);
        assert_eq!(
            main.hierarchy_node(node_id(0)).unwrap().child_node_ids,
            [node_id(1)]
        );
        assert_eq!(
            main.hierarchy_node(node_id(1)).unwrap().child_object_ids,
            [object_at(0)]
        );
    }

    #[test]
    fn move_object_reorders_the_listing_and_validates() {
        let mut main: MeshMain = MeshMain::default();
        let a_id = main.retain_object(MeshObject::new("a".to_owned())).unwrap();
        let b_id = main.retain_object(MeshObject::new("b".to_owned())).unwrap();
        let c_id = main.retain_object(MeshObject::new("c".to_owned())).unwrap();

        main.move_object(c_id, 0).unwrap();

        let order: Vec<_> = main.iter_objects().map(|(id, _)| id).collect();
        assert_eq!(order, [c_id, a_id, b_id]);
        assert_eq!(
            main.move_object(a_id, 3),
            Err(Error::IndexPastCount { index: 3, count: 3 })
        );
        main.validate().unwrap();
    }

    #[test]
    fn retain_hierarchy_nodes_accepts_forward_references_and_rejects_a_cycle() {
        let mut main: MeshMain = MeshMain::default();

        let ids = main
            .retain_hierarchy_nodes(vec![
                node_with_children(vec![node_id(1)]),
                node_with_children(Vec::new()),
            ])
            .unwrap();
        assert_eq!(ids, [node_id(0), node_id(1)]);

        assert_eq!(
            main.retain_hierarchy_nodes(vec![
                node_with_children(vec![node_id(3)]),
                node_with_children(vec![node_id(2)]),
            ]),
            Err(Error::InsertedCycle { index: 0 })
        );
        assert_eq!(main.hierarchy_node_count(), 2);
    }

    #[test]
    fn retain_hierarchy_node_rejects_a_dangling_or_repeated_child_and_a_bad_transform() {
        let mut main: MeshMain = MeshMain::default();
        let first_id = main
            .retain_hierarchy_node(MeshHierarchyNode::default())
            .unwrap();

        assert_eq!(
            main.retain_hierarchy_node(node_with_children(vec![node_id(9)])),
            Err(Error::UnknownHierarchyNode {
                node_id: node_id(9)
            })
        );
        assert_eq!(
            main.retain_hierarchy_node(node_with_children(vec![first_id, first_id])),
            Err(Error::InsertedDuplicateChildNode {
                index: 0,
                child_id: first_id,
            })
        );
        assert_eq!(
            main.retain_hierarchy_node(node_with_objects(vec![object_at(0)])),
            Err(Error::UnknownObject {
                object_id: object_at(0)
            })
        );

        let with_transform = |transform| MeshHierarchyNode {
            transform,
            ..Default::default()
        };
        assert_eq!(
            main.retain_hierarchy_node(with_transform(TyTransformF64 {
                scale: TyVector3F64::new(1.0, 0.0, 1.0),
                ..Default::default()
            })),
            Err(Error::InsertedZeroScale { index: 0 })
        );
        assert_eq!(
            main.retain_hierarchy_node(with_transform(TyTransformF64 {
                position: TyVector3F64::new(f64::NAN, 0.0, 0.0),
                ..Default::default()
            })),
            Err(Error::InsertedNonFiniteTransform { index: 0 })
        );
        assert_eq!(
            main.retain_hierarchy_node(with_transform(TyTransformF64 {
                rotation: TyQuaternionF64::from_xyzw(0.0, 0.0, 0.0, 2.0),
                ..Default::default()
            })),
            Err(Error::InsertedNonUnitRotation { index: 0 })
        );
    }

    #[test]
    fn set_hierarchy_node_replaces_the_node_and_rejects_a_cycle() {
        let mut main: MeshMain = MeshMain::default();
        let ids = main
            .retain_hierarchy_nodes(vec![
                node_with_children(vec![node_id(1)]),
                node_with_children(Vec::new()),
            ])
            .unwrap();

        assert_eq!(
            main.set_hierarchy_node(ids[1], node_with_children(vec![ids[0]])),
            Err(Error::InsertedCycle { index: 0 })
        );

        main.set_hierarchy_node(
            ids[1],
            MeshHierarchyNode {
                name: "renamed".to_owned(),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(main.hierarchy_node(ids[1]).unwrap().name, "renamed");
        main.validate().unwrap();
    }

    #[test]
    fn root_setters_reject_a_dangling_or_duplicate_root() {
        let mut main: MeshMain = MeshMain::default();
        let id = main
            .retain_hierarchy_node(MeshHierarchyNode::default())
            .unwrap();

        assert_eq!(
            main.push_root_hierarchy_node_id(node_id(5)),
            Err(Error::Root {
                root_id: node_id(5)
            })
        );
        main.push_root_hierarchy_node_id(id).unwrap();
        assert_eq!(
            main.push_root_hierarchy_node_id(id),
            Err(Error::DuplicateRoot { root_id: id })
        );
        assert_eq!(
            main.set_root_hierarchy_node_ids(vec![id, id]),
            Err(Error::DuplicateRoot { root_id: id })
        );
    }

    /// An ext that follows nothing.
    #[derive(Debug, Default, PartialEq)]
    struct Tag(u8);

    impl MeshExt for Tag {}

    #[test]
    fn files_check_their_names_and_release_once_unreferenced() {
        let mut main: MeshMain = MeshMain::default();

        let file = |name: &str| MeshFile {
            name: name.to_owned(),
            bytes: PNG_SIGNATURE.to_vec(),
        };

        assert_eq!(main.retain_file(file("")), Err(Error::EmptyFileName));

        let file_id = main.retain_file(file("atlas.png")).unwrap();

        assert_eq!(
            main.retain_file(file("atlas.png")),
            Err(Error::FileNameTaken {
                name: "atlas.png".to_owned(),
                file_id,
            })
        );
        assert_eq!(main.file_count(), 1);
        assert_eq!(main.file_by_name("atlas.png").unwrap().0, file_id);

        // An image over the file, a material and an object pointing at it.
        let image_id = main
            .retain_image(MeshImage {
                name: "atlas".to_owned(),
                media_type: MeshImageMediaType::Png,
                source: MeshImageSource::File(file_id),
            })
            .unwrap();
        assert_eq!(main.image_bytes(image_id), Some(PNG_SIGNATURE.as_slice()));

        let material_id = main
            .retain_material(MeshMaterial {
                properties: vec![MeshProperty {
                    name: "rows".to_owned(),
                    value: MeshPropertyValue::File(file_id),
                }],
                ..MeshMaterial::default()
            })
            .unwrap();

        let mut object = MeshObject::new("body".to_owned());
        object.set_properties(vec![MeshProperty {
            name: "rows".to_owned(),
            value: MeshPropertyValue::File(file_id),
        }]);
        let object_id = main.retain_object(object).unwrap();

        main.validate().unwrap();

        assert_eq!(
            main.release_file(file_id),
            Err(Error::FileInUse {
                file_id,
                image_ids: vec![image_id],
                material_ids: vec![material_id],
                object_ids: vec![object_id],
            })
        );

        // Replacing the file's bytes must keep the image's signature.
        assert_eq!(
            main.set_file(
                file_id,
                MeshFile {
                    name: "atlas.png".to_owned(),
                    bytes: vec![0; 8],
                }
            ),
            Err(Error::MalformedImage {
                media_type: MeshImageMediaType::Png
            })
        );

        main.release_image(image_id).unwrap();
        main.release_material(material_id).unwrap();
        main.set_object_properties(object_id, Vec::new()).unwrap();
        main.release_file(file_id).unwrap();
        assert_eq!(main.file_count(), 0);
    }

    #[test]
    fn images_and_properties_reject_a_dangling_file() {
        let mut main: MeshMain = MeshMain::default();
        let file_id: U32Id<BMeshFile> = U32Id::from_u32(4);

        assert_eq!(
            main.retain_image(MeshImage {
                name: String::new(),
                media_type: MeshImageMediaType::Png,
                source: MeshImageSource::File(file_id),
            }),
            Err(Error::ImageFileRef { file_id })
        );

        let properties = vec![MeshProperty {
            name: "rows".to_owned(),
            value: MeshPropertyValue::File(file_id),
        }];

        assert_eq!(
            main.retain_material(MeshMaterial {
                properties: properties.clone(),
                ..MeshMaterial::default()
            }),
            Err(Error::PropertyFileRef {
                name: "rows".to_owned(),
                file_id,
            })
        );

        let object_id = main.retain_object(MeshObject::new(String::new())).unwrap();

        assert_eq!(
            main.set_object_properties(object_id, properties),
            Err(Error::PropertyFileRef {
                name: "rows".to_owned(),
                file_id,
            })
        );
        assert_eq!(
            main.set_object_properties(
                object_id,
                vec![MeshProperty {
                    name: "key".to_owned(),
                    value: MeshPropertyValue::Texture(MeshTextureRef {
                        texture_id: U32Id::from_u32(9),
                        uv_stream_id: U32Id::from_u32(0),
                    }),
                }]
            ),
            Err(Error::PropertyTextureRef {
                name: "key".to_owned(),
                texture_id: U32Id::from_u32(9),
            })
        );
    }

    #[test]
    fn object_properties_hold_textures_and_gc_relabels_them() {
        let (mut main, object_id, _) = test_main(());
        let spare_image_id = main.retain_image(png_image()).unwrap();
        let spare_texture_id = main
            .retain_texture(MeshTexture::new(spare_image_id))
            .unwrap();
        let file_id = main
            .retain_file(MeshFile {
                name: "values.json".to_owned(),
                bytes: b"{}".to_vec(),
            })
            .unwrap();
        let spare_file_id = main
            .retain_file(MeshFile {
                name: "spare.json".to_owned(),
                bytes: b"{}".to_vec(),
            })
            .unwrap();

        main.set_object_properties(
            object_id,
            vec![
                MeshProperty {
                    name: "key".to_owned(),
                    value: MeshPropertyValue::Texture(MeshTextureRef {
                        texture_id: spare_texture_id,
                        uv_stream_id: U32Id::from_u32(0),
                    }),
                },
                MeshProperty {
                    name: "rows".to_owned(),
                    value: MeshPropertyValue::File(spare_file_id),
                },
            ],
        )
        .unwrap();

        assert_eq!(
            main.release_texture(spare_texture_id),
            Err(Error::TextureInUse {
                texture_id: spare_texture_id,
                material_ids: Vec::new(),
                object_ids: vec![object_id],
            })
        );

        // Release the unreferenced file and the first image and texture, so
        // the referenced ones renumber.
        main.release_file(file_id).unwrap();
        let (first_texture_id, _) = main.iter_textures().next().unwrap();
        let first_image_id = main.texture(first_texture_id).unwrap().image_id;
        let (material_id, _) = main.iter_materials().next().unwrap();
        main.set_material(material_id, MeshMaterial::default())
            .unwrap();
        main.release_texture(first_texture_id).unwrap();
        main.release_image(first_image_id).unwrap();

        let remap = main.gc().unwrap();
        let object = main.object(object_id).unwrap();

        assert_eq!(
            object.property("key").unwrap().value,
            MeshPropertyValue::Texture(MeshTextureRef {
                texture_id: remap.textures.new_id(spare_texture_id).unwrap(),
                uv_stream_id: U32Id::from_u32(0),
            })
        );
        assert_eq!(
            object.property("rows").unwrap().value,
            MeshPropertyValue::File(remap.files.new_id(spare_file_id).unwrap())
        );
        assert_eq!(remap.files.new_id(spare_file_id), Some(U32Id::from_u32(0)));
        main.validate().unwrap();
    }

    #[test]
    fn take_and_put_ext_move_the_document_over() {
        let (main, object_id, _) = test_main(Tag(3));
        let taken = main.take_ext();
        assert_eq!(taken.ext, Tag(3));
        assert!(taken.main.object(object_id).is_some());

        let main = taken.main.put_ext(Tag(7));
        assert_eq!(*main.ext(), Tag(7));
        main.validate().unwrap();
    }
}
