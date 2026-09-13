use crate::{
    BMeshFile, BMeshHierarchyNode, BMeshImage, BMeshMaterial, BMeshObject, BMeshPrimitive,
    BMeshTexture, MeshGcRemap, MeshState, Result,
};
use branded_id::U32Id;

/// The ext a [`MeshMain`](crate::MeshMain) carries: the hooks the state fires
/// when an entity comes, goes, or is renumbered.
///
/// A format ext keeps an entry per entity, keyed by the entity's id. A retain
/// fires its hook after the mutation, a release fires its hook before it once
/// every check has passed, and [`gc`](crate::MeshMain::gc) fires
/// [`did_gc`](Self::did_gc) with the remap after renumbering. The `did` or
/// `will` in a hook's name says which. Each hook sees the document and can
/// build a complete entry for the entity on the spot.
///
/// A hook can refuse. A `will` hook's error leaves the state unchanged. A
/// `did` hook's error reports an ext that could not follow a mutation that
/// already happened: the document stands and the ext is out of step. Every
/// hook defaults to `Ok(())`. A type that follows nothing implements the
/// trait with an empty body, and `()` is that ext for a bare main.
///
/// How an ext persists, and how a boxed one downcasts or clones, is not part
/// of being an ext. That belongs to the crate that converts between formats.
pub trait MeshExt {
    /// Hierarchy node `node_id` was retained.
    fn hierarchy_node_did_retain(
        &mut self,
        _state: &MeshState,
        _node_id: U32Id<BMeshHierarchyNode>,
    ) -> Result<()> {
        Ok(())
    }

    /// Hierarchy node `node_id` is about to be released.
    fn hierarchy_node_will_release(
        &mut self,
        _state: &MeshState,
        _node_id: U32Id<BMeshHierarchyNode>,
    ) -> Result<()> {
        Ok(())
    }

    /// Object `object_id` was retained.
    fn object_did_retain(
        &mut self,
        _state: &MeshState,
        _object_id: U32Id<BMeshObject>,
    ) -> Result<()> {
        Ok(())
    }

    /// Object `object_id` is about to be released.
    fn object_will_release(
        &mut self,
        _state: &MeshState,
        _object_id: U32Id<BMeshObject>,
    ) -> Result<()> {
        Ok(())
    }

    /// Primitive `primitive_id` was retained to object `object_id`.
    fn primitive_did_retain(
        &mut self,
        _state: &MeshState,
        _object_id: U32Id<BMeshObject>,
        _primitive_id: U32Id<BMeshPrimitive>,
    ) -> Result<()> {
        Ok(())
    }

    /// Primitive `primitive_id` of object `object_id` is about to be
    /// released.
    fn primitive_will_release(
        &mut self,
        _state: &MeshState,
        _object_id: U32Id<BMeshObject>,
        _primitive_id: U32Id<BMeshPrimitive>,
    ) -> Result<()> {
        Ok(())
    }

    /// Material `material_id` was retained.
    fn material_did_retain(
        &mut self,
        _state: &MeshState,
        _material_id: U32Id<BMeshMaterial>,
    ) -> Result<()> {
        Ok(())
    }

    /// Material `material_id` is about to be released.
    fn material_will_release(
        &mut self,
        _state: &MeshState,
        _material_id: U32Id<BMeshMaterial>,
    ) -> Result<()> {
        Ok(())
    }

    /// Texture `texture_id` was retained.
    fn texture_did_retain(
        &mut self,
        _state: &MeshState,
        _texture_id: U32Id<BMeshTexture>,
    ) -> Result<()> {
        Ok(())
    }

    /// Texture `texture_id` is about to be released.
    fn texture_will_release(
        &mut self,
        _state: &MeshState,
        _texture_id: U32Id<BMeshTexture>,
    ) -> Result<()> {
        Ok(())
    }

    /// Image `image_id` was retained.
    fn image_did_retain(&mut self, _state: &MeshState, _image_id: U32Id<BMeshImage>) -> Result<()> {
        Ok(())
    }

    /// Image `image_id` is about to be released.
    fn image_will_release(
        &mut self,
        _state: &MeshState,
        _image_id: U32Id<BMeshImage>,
    ) -> Result<()> {
        Ok(())
    }

    /// File `file_id` was retained.
    fn file_did_retain(&mut self, _state: &MeshState, _file_id: U32Id<BMeshFile>) -> Result<()> {
        Ok(())
    }

    /// File `file_id` is about to be released.
    fn file_will_release(&mut self, _state: &MeshState, _file_id: U32Id<BMeshFile>) -> Result<()> {
        Ok(())
    }

    /// Every id pool was renumbered by `remap`. An ext keyed by id rekeys its
    /// entries here.
    fn did_gc(&mut self, _state: &MeshState, _remap: &MeshGcRemap) -> Result<()> {
        Ok(())
    }
}

/// No ext. Ignores every hook.
impl MeshExt for () {}

#[cfg(test)]
mod tests {
    use crate::{
        BMeshHierarchyNode, BMeshImage, BMeshMaterial, BMeshObject, BMeshPrimitive, BMeshTexture,
        Error, MeshExt, MeshGcRemap, MeshHierarchyNode, MeshMain, MeshMaterial, MeshObject,
        MeshState, MeshTexture, Result, png_image, unit_triangle,
    };
    use branded_id::U32Id;

    /// Records every hook the main fires, with what it could read.
    #[derive(Clone, Debug, Default, PartialEq)]
    struct Recorder(Vec<String>);

    impl MeshExt for Recorder {
        fn hierarchy_node_did_retain(
            &mut self,
            state: &MeshState,
            node_id: U32Id<BMeshHierarchyNode>,
        ) -> Result<()> {
            let count = state.hierarchy_node_count();
            self.0
                .push(format!("node retained {} of {count}", node_id.to_u32()));
            Ok(())
        }

        fn hierarchy_node_will_release(
            &mut self,
            state: &MeshState,
            node_id: U32Id<BMeshHierarchyNode>,
        ) -> Result<()> {
            let count = state.hierarchy_node_count();
            self.0
                .push(format!("node released {} of {count}", node_id.to_u32()));
            Ok(())
        }

        fn object_did_retain(
            &mut self,
            state: &MeshState,
            object_id: U32Id<BMeshObject>,
        ) -> Result<()> {
            let name = state.object(object_id).unwrap().name();
            self.0
                .push(format!("object retained {} {name}", object_id.to_u32()));
            Ok(())
        }

        fn object_will_release(
            &mut self,
            state: &MeshState,
            object_id: U32Id<BMeshObject>,
        ) -> Result<()> {
            let name = state.object(object_id).unwrap().name();
            self.0
                .push(format!("object released {} {name}", object_id.to_u32()));
            Ok(())
        }

        fn primitive_did_retain(
            &mut self,
            _state: &MeshState,
            object_id: U32Id<BMeshObject>,
            primitive_id: U32Id<BMeshPrimitive>,
        ) -> Result<()> {
            self.0.push(format!(
                "primitive retained {} {}",
                object_id.to_u32(),
                primitive_id.to_u32()
            ));
            Ok(())
        }

        fn primitive_will_release(
            &mut self,
            _state: &MeshState,
            object_id: U32Id<BMeshObject>,
            primitive_id: U32Id<BMeshPrimitive>,
        ) -> Result<()> {
            self.0.push(format!(
                "primitive released {} {}",
                object_id.to_u32(),
                primitive_id.to_u32()
            ));
            Ok(())
        }

        fn material_did_retain(
            &mut self,
            _state: &MeshState,
            material_id: U32Id<BMeshMaterial>,
        ) -> Result<()> {
            self.0
                .push(format!("material retained {}", material_id.to_u32()));
            Ok(())
        }

        fn material_will_release(
            &mut self,
            _state: &MeshState,
            material_id: U32Id<BMeshMaterial>,
        ) -> Result<()> {
            self.0
                .push(format!("material released {}", material_id.to_u32()));
            Ok(())
        }

        fn texture_did_retain(
            &mut self,
            _state: &MeshState,
            texture_id: U32Id<BMeshTexture>,
        ) -> Result<()> {
            self.0
                .push(format!("texture retained {}", texture_id.to_u32()));
            Ok(())
        }

        fn texture_will_release(
            &mut self,
            _state: &MeshState,
            texture_id: U32Id<BMeshTexture>,
        ) -> Result<()> {
            self.0
                .push(format!("texture released {}", texture_id.to_u32()));
            Ok(())
        }

        fn image_did_retain(
            &mut self,
            _state: &MeshState,
            image_id: U32Id<BMeshImage>,
        ) -> Result<()> {
            self.0.push(format!("image retained {}", image_id.to_u32()));
            Ok(())
        }

        fn image_will_release(
            &mut self,
            _state: &MeshState,
            image_id: U32Id<BMeshImage>,
        ) -> Result<()> {
            self.0.push(format!("image released {}", image_id.to_u32()));
            Ok(())
        }

        fn did_gc(&mut self, _state: &MeshState, remap: &MeshGcRemap) -> Result<()> {
            let objects: Vec<Option<u32>> = (0..remap.objects.old_len() as u32)
                .map(|old| {
                    remap
                        .objects
                        .new_id(U32Id::from_u32(old))
                        .map(|id| id.to_u32())
                })
                .collect();
            self.0.push(format!("gc objects {objects:?}"));
            Ok(())
        }
    }

    /// Refuses every release.
    #[derive(Debug, Default)]
    struct Refuser;

    impl MeshExt for Refuser {
        fn object_will_release(
            &mut self,
            _state: &MeshState,
            object_id: U32Id<BMeshObject>,
        ) -> Result<()> {
            Err(Error::Ext {
                reason: format!("object {} is pinned", object_id.to_u32()),
            })
        }
    }

    fn events(main: &MeshMain<Recorder>) -> Vec<&str> {
        main.ext().0.iter().map(String::as_str).collect()
    }

    fn named_object(name: &str) -> MeshObject {
        MeshObject::new(name.to_owned())
    }

    #[test]
    fn objects_fire_with_their_id_and_the_document() {
        let mut main: MeshMain<Recorder> = MeshMain::default();

        main.retain_object(named_object("a")).unwrap();

        let b_id = main.retain_object(named_object("b")).unwrap();

        main.retain_object(named_object("c")).unwrap();

        main.release_object(b_id).unwrap();

        // The reused id lands at the end of the listing. A move fires nothing
        // because ids do not change.
        let d_id = main.retain_object(named_object("d")).unwrap();

        main.move_object(d_id, 0).unwrap();

        assert_eq!(
            events(&main),
            [
                "object retained 0 a",
                "object retained 1 b",
                "object retained 2 c",
                "object released 1 b",
                "object retained 1 d",
            ]
        );
    }

    #[test]
    fn images_textures_and_materials_fire_with_their_id() {
        let mut main: MeshMain<Recorder> = MeshMain::default();

        let image_id = main.retain_image(png_image()).unwrap();

        let texture_id = main.retain_texture(MeshTexture::new(image_id)).unwrap();

        let material_id = main.retain_material(MeshMaterial::default()).unwrap();

        main.release_material(material_id).unwrap();

        main.release_texture(texture_id).unwrap();

        main.release_image(image_id).unwrap();

        assert_eq!(
            events(&main),
            [
                "image retained 0",
                "texture retained 0",
                "material retained 0",
                "material released 0",
                "texture released 0",
                "image released 0",
            ]
        );
    }

    #[test]
    fn hierarchy_nodes_fire_with_their_id_after_the_retain() {
        let mut main: MeshMain<Recorder> = MeshMain::default();

        main.retain_hierarchy_node(MeshHierarchyNode::default())
            .unwrap();

        let b_id = main
            .retain_hierarchy_node(MeshHierarchyNode::default())
            .unwrap();

        main.retain_hierarchy_nodes(vec![
            MeshHierarchyNode::default(),
            MeshHierarchyNode::default(),
        ])
        .unwrap();

        main.release_hierarchy_node(b_id).unwrap();

        // A batch fires after the whole batch is in. A release fires while
        // the node is still there.
        assert_eq!(
            events(&main),
            [
                "node retained 0 of 1",
                "node retained 1 of 2",
                "node retained 2 of 4",
                "node retained 3 of 4",
                "node released 1 of 4",
            ]
        );
    }

    #[test]
    fn primitives_fire_within_their_object() {
        let mut main: MeshMain<Recorder> = MeshMain::default();

        main.retain_object(named_object("a")).unwrap();

        let object_id = main.retain_object(named_object("b")).unwrap();

        let primitive_id = main.retain_primitive(object_id, unit_triangle()).unwrap();

        main.release_primitive(object_id, primitive_id).unwrap();

        assert_eq!(
            events(&main)[2..],
            ["primitive retained 1 0", "primitive released 1 0"]
        );
    }

    #[test]
    fn gc_fires_with_the_remap() {
        let mut main: MeshMain<Recorder> = MeshMain::default();

        let a_id = main.retain_object(named_object("a")).unwrap();

        main.retain_object(named_object("b")).unwrap();

        main.release_object(a_id).unwrap();

        main.gc().unwrap();

        assert_eq!(events(&main)[3..], ["gc objects [None, Some(0)]"]);
    }

    #[test]
    fn a_rejected_mutation_fires_nothing() {
        let mut main: MeshMain<Recorder> = MeshMain::default();

        assert!(main.release_object(U32Id::from_u32(9)).is_err());

        assert!(main.move_material(U32Id::from_u32(0), 0).is_err());

        assert!(events(&main).is_empty());
    }

    #[test]
    fn a_refused_release_changes_nothing() {
        let mut main: MeshMain<Refuser> = MeshMain::default();

        let object_id = main.retain_object(named_object("a")).unwrap();

        assert_eq!(
            main.release_object(object_id),
            Err(Error::Ext {
                reason: "object 0 is pinned".to_owned()
            })
        );

        assert!(main.object(object_id).is_some());
    }
}
