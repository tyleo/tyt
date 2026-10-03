use crate::{
    BRenderLight, BRenderMaterial, BRenderPlacement, BRenderView, Error, RenderLight,
    RenderMaterial, RenderObject, RenderPlacement, RenderProjection, RenderView, Result,
};
use branded_id::{IdVec, U32Id, UsizeId, ext::IteratorExt, soa::IdList};
use std::{
    collections::{HashMap, HashSet, hash_map::Entry},
    f64::consts::{FRAC_PI_2, PI},
};
use ty_math::{
    TyBoundsF64, TyLinSrgbF64, TyQuaternionExt, TyQuaternionF64, TyTransformF64, TyVector3F64,
    TyVector3I32, UNIT_ROTATION_TOLERANCE,
};
use voxcore::{
    BVoxEffectiveProperty, BVoxHierarchyNode, BVoxObject, BVoxValuePoolValue, BVoxVoxel,
    VoxEffectivePalette, VoxExt, VoxMain, VoxValuePool,
    color::ColorValues,
    material::{
        BASE_COLOR, EMISSIVE_COLOR, EMISSIVE_STRENGTH, METALLIC, OCCLUSION_STRENGTH, ROUGHNESS,
    },
};

/// A scene flattened for drawing: objects over a table of materials,
/// placements of the objects in world meters, and the lights and views in
/// world space. Every renderer consumes it.
///
/// Objects are keyed by the id of the voxcore object each mirrors, which
/// lets an edit to the document find its object here. Materials,
/// placements, lights, and views match nothing in voxcore one to one. Each
/// of those kinds lives in an id pool, and its ids are meaningful only
/// within the scene that issued them. Every mutation checks the
/// cross-references it could break, so a scene reached through the public
/// API never places an unknown object or samples an unknown material. A
/// release leaves a hole, and the survivors keep their order.
#[derive(Default)]
pub struct RenderScene {
    materials: IdList<BRenderMaterial, RenderMaterial>,

    objects: IdVec<BVoxObject, Option<RenderObject>>,

    placements: IdList<BRenderPlacement, RenderPlacement>,

    lights: IdList<BRenderLight, RenderLight>,

    views: IdList<BRenderView, RenderView>,
}

impl RenderScene {
    /// Flattens the objects `object_ids` of `main` for drawing at
    /// `voxel_size` meters per voxel.
    ///
    /// Each object yields one material per live voxel, resolved by the
    /// layer-override rule of the effective palette and deduplicated into
    /// the material table. A shaded property the palette does not supply
    /// takes its standard default. Each root-to-object path of the hierarchy
    /// yields one placement carrying the path's world transform, with the
    /// grid origin folded in and every position scaled by the voxel size. An
    /// object no path reaches gets one placement at the identity. Errors if:
    ///
    /// 1. `voxel_size` is not finite and positive
    /// 2. an object id is not one of `main`'s or is listed twice
    /// 3. a shaded property's value pool is not the kind the contract reads
    /// 4. a shaded value is outside its range
    /// 5. voxcore refuses a read
    pub fn from_vox_main<T: VoxExt>(
        main: &VoxMain<T>,
        object_ids: &[U32Id<BVoxObject>],
        voxel_size: f64,
    ) -> Result<Self> {
        if !(voxel_size.is_finite() && voxel_size > 0.0) {
            return Err(Error::VoxelSize { voxel_size });
        }

        let mut scene = RenderScene::default();
        let mut material_ids: HashMap<MaterialKey, U32Id<BRenderMaterial>> = HashMap::new();

        for &object_id in object_ids {
            let Some(object) = main.object(object_id) else {
                return Err(Error::UnknownVoxObject { object_id });
            };

            let effective = main.effective_palette(object)?;

            let mut voxels: Vec<(U32Id<BVoxVoxel>, RenderMaterial)> = object
                .iter_live()
                .map(|voxel_id| (voxel_id, RenderMaterial::default()))
                .collect();

            fill_color(&effective, BASE_COLOR, &mut voxels, |material| {
                &mut material.base_color
            })?;

            fill_scalar(&effective, METALLIC, &mut voxels, |material| {
                &mut material.metallic
            })?;

            fill_scalar(&effective, ROUGHNESS, &mut voxels, |material| {
                &mut material.roughness
            })?;

            fill_color(&effective, EMISSIVE_COLOR, &mut voxels, |material| {
                &mut material.emissive_color
            })?;

            fill_scalar(&effective, EMISSIVE_STRENGTH, &mut voxels, |material| {
                &mut material.emissive_strength
            })?;

            fill_scalar(&effective, OCCLUSION_STRENGTH, &mut voxels, |material| {
                &mut material.occlusion_strength
            })?;

            let mut render_object = RenderObject::new(object.name().to_owned(), object.bounds())?;

            for (voxel_id, material) in voxels {
                let material_id = match material_ids.entry(material_key(&material)) {
                    Entry::Occupied(entry) => *entry.get(),
                    Entry::Vacant(entry) => *entry.insert(scene.retain_material(material)?),
                };

                render_object
                    .set_voxel_material(voxel_id, Some(material_id))
                    .expect("the render grid shares the object's bounds");
            }

            scene.retain_object(object_id, render_object)?;
        }

        let mut placed = HashSet::new();

        for &root_id in main.root_hierarchy_node_ids() {
            place(
                main,
                root_id,
                &TyTransformF64::IDENTITY,
                voxel_size,
                &mut scene,
                &mut placed,
            )?;
        }

        for &object_id in object_ids {
            if placed.contains(&object_id) {
                continue;
            }

            let origin = main
                .object(object_id)
                .expect("a flattened object is one of the main's")
                .origin();

            scene.retain_placement(RenderPlacement {
                object_id,
                transform: placement_transform(&TyTransformF64::IDENTITY, origin, voxel_size),
            })?;
        }

        Ok(scene)
    }

    /// Retains `material` at the end of the listing, returning its id.
    /// Errors, changing nothing, if a value is outside its range.
    pub fn retain_material(&mut self, material: RenderMaterial) -> Result<U32Id<BRenderMaterial>> {
        check_material(&material)?;

        Ok(self.materials.retain(material))
    }

    /// Releases material `id`. Errors, changing nothing, if `id` is not one
    /// of the scene's or a voxel still samples it.
    pub fn release_material(&mut self, id: U32Id<BRenderMaterial>) -> Result<()> {
        if !self.materials.ids().is_retained(id) {
            return Err(Error::UnknownMaterial { material_id: id });
        }

        let object_ids: Vec<_> = self
            .iter_objects()
            .filter(|(_, object)| object.iter_live().any(|(_, material_id)| material_id == id))
            .map(|(object_id, _)| object_id)
            .collect();

        if !object_ids.is_empty() {
            return Err(Error::MaterialInUse {
                material_id: id,
                object_ids,
            });
        }

        self.materials
            .release_stable(id)
            .expect("the material is one of the scene's");

        Ok(())
    }

    /// The material `id`, or `None` if not one of the scene's.
    pub fn material(&self, id: U32Id<BRenderMaterial>) -> Option<&RenderMaterial> {
        self.materials.get(id)
    }

    /// Materials in listing order, as `(id, material)`.
    pub fn iter_materials(
        &self,
    ) -> impl Iterator<Item = (U32Id<BRenderMaterial>, &RenderMaterial)> + '_ {
        self.materials.iter()
    }

    /// Number of materials.
    pub fn material_count(&self) -> usize {
        self.materials.len()
    }

    /// Retains `object` under `id`, the id of the voxcore object it mirrors.
    /// Errors, changing nothing, if the scene already holds `id` or a voxel
    /// samples a material that is not one of the scene's.
    pub fn retain_object(&mut self, id: U32Id<BVoxObject>, object: RenderObject) -> Result<()> {
        if self.object(id).is_some() {
            return Err(Error::DuplicateObject { object_id: id });
        }

        for (voxel_id, material_id) in object.iter_live() {
            if !self.materials.ids().is_retained(material_id) {
                return Err(Error::VoxelMaterialRef {
                    voxel_id,
                    material_id,
                });
            }
        }

        let len = id.to_u32() as usize + 1;

        if self.objects.len() < len {
            self.objects.resize(len, None);
        }

        self.objects[id.to_usize_id()] = Some(object);

        Ok(())
    }

    /// Releases object `id`. Errors, changing nothing, if `id` is not one of
    /// the scene's or a placement still places it.
    pub fn release_object(&mut self, id: U32Id<BVoxObject>) -> Result<()> {
        if self.object(id).is_none() {
            return Err(Error::UnknownObject { object_id: id });
        }

        let placement_ids: Vec<_> = self
            .iter_placements()
            .filter(|(_, placement)| placement.object_id == id)
            .map(|(placement_id, _)| placement_id)
            .collect();

        if !placement_ids.is_empty() {
            return Err(Error::ObjectInUse {
                object_id: id,
                placement_ids,
            });
        }

        self.objects[id.to_usize_id()] = None;

        Ok(())
    }

    /// Writes the material the cell `voxel_id` of object `object_id` holds,
    /// `None` to empty it. Errors, changing nothing, if the object or the
    /// material is not one of the scene's or the voxel is outside the grid.
    pub fn set_voxel_material(
        &mut self,
        object_id: U32Id<BVoxObject>,
        voxel_id: U32Id<BVoxVoxel>,
        material_id: Option<U32Id<BRenderMaterial>>,
    ) -> Result<()> {
        if self.object(object_id).is_none() {
            return Err(Error::UnknownObject { object_id });
        }

        if let Some(material_id) = material_id
            && !self.materials.ids().is_retained(material_id)
        {
            return Err(Error::UnknownMaterial { material_id });
        }

        let object = self.objects[object_id.to_usize_id()]
            .as_mut()
            .expect("the object is one of the scene's");

        object.set_voxel_material(voxel_id, material_id)
    }

    /// The object `id`, or `None` if not one of the scene's.
    pub fn object(&self, id: U32Id<BVoxObject>) -> Option<&RenderObject> {
        self.objects.get(id.to_usize_id()).and_then(Option::as_ref)
    }

    /// Objects in id order, as `(id, object)`.
    pub fn iter_objects(&self) -> impl Iterator<Item = (U32Id<BVoxObject>, &RenderObject)> + '_ {
        self.objects
            .iter()
            .enumerate_ids()
            .filter_map(|(id, object)| object.as_ref().map(|object| (id, object)))
    }

    /// Number of objects.
    pub fn object_count(&self) -> usize {
        self.objects.iter().flatten().count()
    }

    /// Retains `placement` at the end of the listing, returning its id.
    /// Errors, changing nothing, if its object is not one of the scene's or
    /// its transform is not finite, has a zero scale, or has a non-unit
    /// rotation.
    pub fn retain_placement(
        &mut self,
        placement: RenderPlacement,
    ) -> Result<U32Id<BRenderPlacement>> {
        if self.object(placement.object_id).is_none() {
            return Err(Error::UnknownObject {
                object_id: placement.object_id,
            });
        }

        check_transform(&placement.transform)?;

        Ok(self.placements.retain(placement))
    }

    /// Releases placement `id`. Errors, changing nothing, if `id` is not one
    /// of the scene's.
    pub fn release_placement(&mut self, id: U32Id<BRenderPlacement>) -> Result<()> {
        let Some(_) = self.placements.release_stable(id) else {
            return Err(Error::UnknownPlacement { placement_id: id });
        };

        Ok(())
    }

    /// Moves placement `id` onto `transform`. Errors, changing nothing, if
    /// `id` is not one of the scene's or `transform` fails the checks
    /// [`retain_placement`](Self::retain_placement) makes.
    pub fn set_placement_transform(
        &mut self,
        id: U32Id<BRenderPlacement>,
        transform: TyTransformF64,
    ) -> Result<()> {
        let Some(placement) = self.placements.get_mut(id) else {
            return Err(Error::UnknownPlacement { placement_id: id });
        };

        check_transform(&transform)?;

        placement.transform = transform;

        Ok(())
    }

    /// The placement `id`, or `None` if not one of the scene's.
    pub fn placement(&self, id: U32Id<BRenderPlacement>) -> Option<&RenderPlacement> {
        self.placements.get(id)
    }

    /// Placements in listing order, as `(id, placement)`.
    pub fn iter_placements(
        &self,
    ) -> impl Iterator<Item = (U32Id<BRenderPlacement>, &RenderPlacement)> + '_ {
        self.placements.iter()
    }

    /// Number of placements.
    pub fn placement_count(&self) -> usize {
        self.placements.len()
    }

    /// Retains `light` at the end of the listing, returning its id. Errors,
    /// changing nothing, if a value is not finite, a color or strength is
    /// negative, a rotation is not unit length, or a range is not positive.
    pub fn retain_light(&mut self, light: RenderLight) -> Result<U32Id<BRenderLight>> {
        check_light(&light)?;

        Ok(self.lights.retain(light))
    }

    /// Releases light `id`. Errors, changing nothing, if `id` is not one of
    /// the scene's.
    pub fn release_light(&mut self, id: U32Id<BRenderLight>) -> Result<()> {
        let Some(_) = self.lights.release_stable(id) else {
            return Err(Error::UnknownLight { light_id: id });
        };

        Ok(())
    }

    /// Replaces light `id` with `light`. Errors, changing nothing, if `id` is
    /// not one of the scene's or `light` fails the checks
    /// [`retain_light`](Self::retain_light) makes.
    pub fn set_light(&mut self, id: U32Id<BRenderLight>, light: RenderLight) -> Result<()> {
        let Some(current) = self.lights.get_mut(id) else {
            return Err(Error::UnknownLight { light_id: id });
        };

        check_light(&light)?;

        *current = light;

        Ok(())
    }

    /// The light `id`, or `None` if not one of the scene's.
    pub fn light(&self, id: U32Id<BRenderLight>) -> Option<&RenderLight> {
        self.lights.get(id)
    }

    /// Lights in listing order, as `(id, light)`.
    pub fn iter_lights(&self) -> impl Iterator<Item = (U32Id<BRenderLight>, &RenderLight)> + '_ {
        self.lights.iter()
    }

    /// Number of lights.
    pub fn light_count(&self) -> usize {
        self.lights.len()
    }

    /// Retains `view` at the end of the listing, returning its id. Errors,
    /// changing nothing, if its position is not finite, its rotation is not
    /// unit length, its field of view is not within `(0, pi)`, or its scale
    /// is not finite and positive.
    pub fn retain_view(&mut self, view: RenderView) -> Result<U32Id<BRenderView>> {
        check_view(&view)?;

        Ok(self.views.retain(view))
    }

    /// Releases view `id`. Errors, changing nothing, if `id` is not one of
    /// the scene's.
    pub fn release_view(&mut self, id: U32Id<BRenderView>) -> Result<()> {
        let Some(_) = self.views.release_stable(id) else {
            return Err(Error::UnknownView { view_id: id });
        };

        Ok(())
    }

    /// Replaces view `id` with `view`. Errors, changing nothing, if `id` is
    /// not one of the scene's or `view` fails the checks
    /// [`retain_view`](Self::retain_view) makes.
    pub fn set_view(&mut self, id: U32Id<BRenderView>, view: RenderView) -> Result<()> {
        let Some(current) = self.views.get_mut(id) else {
            return Err(Error::UnknownView { view_id: id });
        };

        check_view(&view)?;

        *current = view;

        Ok(())
    }

    /// The view `id`, or `None` if not one of the scene's.
    pub fn view(&self, id: U32Id<BRenderView>) -> Option<&RenderView> {
        self.views.get(id)
    }

    /// Views in listing order, as `(id, view)`.
    pub fn iter_views(&self) -> impl Iterator<Item = (U32Id<BRenderView>, &RenderView)> + '_ {
        self.views.iter()
    }

    /// Number of views.
    pub fn view_count(&self) -> usize {
        self.views.len()
    }

    /// The world bounds of the live voxels under `placement_ids`, or `None`
    /// where none is live. Each placement contributes the eight transformed
    /// corners of its object's live extent. Errors if a placement is not one
    /// of the scene's.
    pub fn subject_bounds(
        &self,
        placement_ids: &[U32Id<BRenderPlacement>],
    ) -> Result<Option<TyBoundsF64>> {
        let mut bounds: Option<TyBoundsF64> = None;

        for &placement_id in placement_ids {
            let Some(placement) = self.placement(placement_id) else {
                return Err(Error::UnknownPlacement { placement_id });
            };

            let object = self
                .object(placement.object_id)
                .expect("a placement's object is one of the scene's");

            let Some((min, max)) = object.live_extent() else {
                continue;
            };

            let min = min.as_dvec3();
            let max = max.as_dvec3() + TyVector3F64::ONE;

            let corners = (0..8).map(|corner| {
                let pick = |axis: usize| {
                    if corner & (1 << axis) == 0 {
                        min[axis]
                    } else {
                        max[axis]
                    }
                };

                placement
                    .transform
                    .transform_point(TyVector3F64::new(pick(0), pick(1), pick(2)))
            });

            let placed = TyBoundsF64::from_points(corners).expect("eight corners bound a box");

            bounds = Some(match bounds {
                Some(bounds) => bounds.encapsulate(&placed),
                None => placed,
            });
        }

        Ok(bounds)
    }
}

/// Whether `value` is within `0..=1`. `NaN` is not.
fn is_unit(value: f64) -> bool {
    (0.0..=1.0).contains(&value)
}

fn is_unit_color(color: TyLinSrgbF64) -> bool {
    is_unit(color.red) && is_unit(color.green) && is_unit(color.blue)
}

fn is_finite_color(color: TyLinSrgbF64) -> bool {
    color.red.is_finite() && color.green.is_finite() && color.blue.is_finite()
}

fn is_non_negative_color(color: TyLinSrgbF64) -> bool {
    color.red >= 0.0 && color.green >= 0.0 && color.blue >= 0.0
}

fn is_unit_rotation(rotation: TyQuaternionF64) -> bool {
    rotation.is_normalized_within(UNIT_ROTATION_TOLERANCE)
}

fn check_material(material: &RenderMaterial) -> Result<()> {
    let checks = [
        (BASE_COLOR, is_unit_color(material.base_color)),
        (METALLIC, is_unit(material.metallic)),
        (ROUGHNESS, is_unit(material.roughness)),
        (EMISSIVE_COLOR, is_unit_color(material.emissive_color)),
        (
            EMISSIVE_STRENGTH,
            material.emissive_strength.is_finite() && material.emissive_strength >= 0.0,
        ),
        (OCCLUSION_STRENGTH, is_unit(material.occlusion_strength)),
    ];

    for (property, holds) in checks {
        if !holds {
            return Err(Error::MaterialOutOfRange {
                property: property.to_owned(),
            });
        }
    }

    Ok(())
}

fn check_transform(transform: &TyTransformF64) -> Result<()> {
    if !transform.position.is_finite() || !transform.scale.is_finite() {
        return Err(Error::NonFinitePlacement);
    }

    let scale = transform.scale;

    if scale.x == 0.0 || scale.y == 0.0 || scale.z == 0.0 {
        return Err(Error::ZeroPlacementScale);
    }

    if !is_unit_rotation(transform.rotation) {
        return Err(Error::NonUnitPlacementRotation);
    }

    Ok(())
}

/// Errors unless a light at `position` with glTF's punctual falloff has
/// finite values, a non-negative `color` and `strength`, and a positive
/// `range` where it has one.
fn check_falloff(
    position: TyVector3F64,
    color: TyLinSrgbF64,
    strength: f64,
    range: Option<f64>,
) -> Result<()> {
    if !position.is_finite()
        || !is_finite_color(color)
        || !strength.is_finite()
        || range.is_some_and(|range| !range.is_finite())
    {
        return Err(Error::NonFiniteLight);
    }

    if !is_non_negative_color(color) || strength < 0.0 {
        return Err(Error::NegativeLight);
    }

    if let Some(range) = range
        && range <= 0.0
    {
        return Err(Error::NonPositiveLightRange { range });
    }

    Ok(())
}

fn check_light(light: &RenderLight) -> Result<()> {
    match *light {
        RenderLight::Directional {
            rotation,
            color,
            strength,
            ..
        } => {
            if !is_finite_color(color) || !strength.is_finite() {
                return Err(Error::NonFiniteLight);
            }

            if !is_unit_rotation(rotation) {
                return Err(Error::NonUnitLightRotation);
            }

            if !is_non_negative_color(color) || strength < 0.0 {
                return Err(Error::NegativeLight);
            }
        }

        RenderLight::Point {
            position,
            color,
            strength,
            range,
            ..
        } => check_falloff(position, color, strength, range)?,

        RenderLight::Spot {
            position,
            rotation,
            color,
            strength,
            range,
            inner_cone,
            outer_cone,
            ..
        } => {
            if !inner_cone.is_finite() || !outer_cone.is_finite() {
                return Err(Error::NonFiniteLight);
            }

            check_falloff(position, color, strength, range)?;

            if !is_unit_rotation(rotation) {
                return Err(Error::NonUnitLightRotation);
            }

            if !(0.0 <= inner_cone && inner_cone < outer_cone && outer_cone <= FRAC_PI_2) {
                return Err(Error::SpotCone {
                    inner_cone,
                    outer_cone,
                });
            }
        }

        RenderLight::Hemisphere {
            sky,
            ground,
            strength,
        } => {
            if !is_finite_color(sky) || !is_finite_color(ground) || !strength.is_finite() {
                return Err(Error::NonFiniteLight);
            }

            if !is_non_negative_color(sky) || !is_non_negative_color(ground) || strength < 0.0 {
                return Err(Error::NegativeLight);
            }
        }
    }

    Ok(())
}

fn check_view(view: &RenderView) -> Result<()> {
    if !view.pose.position.is_finite() {
        return Err(Error::NonFiniteView);
    }

    if !is_unit_rotation(view.pose.rotation) {
        return Err(Error::NonUnitViewRotation);
    }

    match view.projection {
        RenderProjection::Perspective { fov } => {
            if !(fov > 0.0 && fov < PI) {
                return Err(Error::FieldOfViewOutOfRange { fov });
            }
        }

        RenderProjection::Orthographic { scale } => {
            if !(scale.is_finite() && scale > 0.0) {
                return Err(Error::ViewScale { scale });
            }
        }
    }

    Ok(())
}

/// A material's six values by their bits, the key that deduplicates the
/// table.
type MaterialKey = [u64; 10];

/// Places every flattened object under `node_id`, itself under `parent`,
/// the world transform of its parent path, then walks its child nodes in
/// order.
fn place<T: VoxExt>(
    main: &VoxMain<T>,
    node_id: U32Id<BVoxHierarchyNode>,
    parent: &TyTransformF64,
    voxel_size: f64,
    scene: &mut RenderScene,
    placed: &mut HashSet<U32Id<BVoxObject>>,
) -> Result<()> {
    let node = main
        .hierarchy_node(node_id)
        .expect("a walked node is one of the main's");

    let world = parent.compose(&node.transform);

    for &object_id in &node.child_object_ids {
        if scene.object(object_id).is_none() {
            continue;
        }

        let origin = main
            .object(object_id)
            .expect("a placed object is one of the main's")
            .origin();

        scene.retain_placement(RenderPlacement {
            object_id,
            transform: placement_transform(&world, origin, voxel_size),
        })?;

        placed.insert(object_id);
    }

    for &child_id in &node.child_node_ids {
        place(main, child_id, &world, voxel_size, scene, placed)?;
    }

    Ok(())
}

/// The grid-to-world transform of an object at `origin` under the node path
/// `world`, at `voxel_size` meters per voxel. Scaling every node position by
/// the voxel size is one uniform scale of the whole path, so the path's
/// position scales and the grid units fold into the placement's scale.
fn placement_transform(
    world: &TyTransformF64,
    origin: TyVector3I32,
    voxel_size: f64,
) -> TyTransformF64 {
    let scaled = TyTransformF64 {
        position: world.position * voxel_size,
        ..*world
    };

    scaled.compose(&TyTransformF64::new(
        origin.as_dvec3() * voxel_size,
        TyQuaternionF64::IDENTITY,
        TyVector3F64::splat(voxel_size),
    ))
}

/// Fills color property `property` into each voxel's material through
/// `field`. Drops the alpha. Leaves the default where `effective` does not
/// supply it. Errors when its value pool holds no colors.
fn fill_color(
    effective: &VoxEffectivePalette<'_>,
    property: &str,
    voxels: &mut [(U32Id<BVoxVoxel>, RenderMaterial)],
    field: fn(&mut RenderMaterial) -> &mut TyLinSrgbF64,
) -> Result<()> {
    let Some(property_id) = effective.property_id_by_name(property) else {
        return Ok(());
    };

    let colors = ColorValues::of(property_value_pool(effective, property_id)).ok_or_else(|| {
        Error::MaterialPropertyKind {
            property: property.to_owned(),
        }
    })?;

    for (voxel_id, material) in voxels {
        let color = colors
            .lin_srgba_f64(voxel_value_id(effective, *voxel_id, property_id))
            .expect("a material draws one of its property's values");

        *field(material) = TyLinSrgbF64::new(color.red, color.green, color.blue);
    }

    Ok(())
}

/// Fills scalar property `property` into each voxel's material through
/// `field`. Leaves the default where `effective` does not supply it. Errors
/// when its value pool holds no floats.
fn fill_scalar(
    effective: &VoxEffectivePalette<'_>,
    property: &str,
    voxels: &mut [(U32Id<BVoxVoxel>, RenderMaterial)],
    field: fn(&mut RenderMaterial) -> &mut f64,
) -> Result<()> {
    let Some(property_id) = effective.property_id_by_name(property) else {
        return Ok(());
    };

    let values = property_value_pool(effective, property_id)
        .float_values()
        .ok_or_else(|| Error::MaterialPropertyKind {
            property: property.to_owned(),
        })?;

    for (voxel_id, material) in voxels {
        *field(material) = *values
            .get(voxel_value_id(effective, *voxel_id, property_id))
            .expect("a material draws one of its property's values");
    }

    Ok(())
}

/// The value pool the resolved property `property_id` draws from.
fn property_value_pool<'a>(
    effective: &VoxEffectivePalette<'a>,
    property_id: UsizeId<BVoxEffectiveProperty>,
) -> &'a VoxValuePool {
    effective
        .property(property_id)
        .expect("a resolved name identifies one of the effective palette's properties")
        .value_pool()
}

/// The value id the live voxel `voxel_id` reads for `property_id`.
fn voxel_value_id(
    effective: &VoxEffectivePalette<'_>,
    voxel_id: U32Id<BVoxVoxel>,
    property_id: UsizeId<BVoxEffectiveProperty>,
) -> U32Id<BVoxValuePoolValue> {
    effective
        .voxel_value_id(voxel_id, property_id)
        .expect("a live voxel samples a material holding every property of its palette")
}

fn material_key(material: &RenderMaterial) -> MaterialKey {
    [
        material.base_color.red,
        material.base_color.green,
        material.base_color.blue,
        material.metallic,
        material.roughness,
        material.emissive_color.red,
        material.emissive_color.green,
        material.emissive_color.blue,
        material.emissive_strength,
        material.occlusion_strength,
    ]
    .map(f64::to_bits)
}

#[cfg(test)]
mod tests {
    use crate::{
        BRenderMaterial, Error, RenderLight, RenderMaterial, RenderObject, RenderPlacement,
        RenderProjection, RenderScene, RenderShadow, RenderView,
    };
    use branded_id::{IdRange, U32Id};
    use std::f64::consts::{FRAC_PI_2, PI};
    use ty_math::{
        TyLinSrgbF64, TyPoseF64, TyQuaternionF64, TyTransformF64, TyVector3F64, TyVector3I32,
        TyVector3U32,
    };
    use voxcore::{
        BVoxObject, VoxHierarchyNode, VoxMain, VoxObject, VoxPalette, VoxValuePool,
        material::{BASE_COLOR, METALLIC},
    };

    fn white() -> TyLinSrgbF64 {
        TyLinSrgbF64::new(1.0, 1.0, 1.0)
    }

    /// A scene with one material and a one-voxel object sampling it.
    fn painted() -> (RenderScene, U32Id<BRenderMaterial>, U32Id<BVoxObject>) {
        let mut scene = RenderScene::default();
        let material_id = scene.retain_material(RenderMaterial::default()).unwrap();

        let mut object = RenderObject::new("o".to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();
        object
            .set_voxel_material(U32Id::from_u32(0), Some(material_id))
            .unwrap();
        let object_id = U32Id::from_u32(3);
        scene.retain_object(object_id, object).unwrap();

        (scene, material_id, object_id)
    }

    #[test]
    fn a_material_a_voxel_samples_and_an_object_a_placement_places_stay() {
        let (mut scene, material_id, object_id) = painted();

        assert_eq!(
            scene.release_material(material_id),
            Err(Error::MaterialInUse {
                material_id,
                object_ids: vec![object_id],
            })
        );

        let placement_id = scene
            .retain_placement(RenderPlacement {
                object_id,
                transform: TyTransformF64::IDENTITY,
            })
            .unwrap();

        assert_eq!(
            scene.release_object(object_id),
            Err(Error::ObjectInUse {
                object_id,
                placement_ids: vec![placement_id],
            })
        );

        scene.release_placement(placement_id).unwrap();
        scene.release_object(object_id).unwrap();
        scene.release_material(material_id).unwrap();

        assert_eq!(scene.material_count(), 0);
        assert_eq!(scene.object_count(), 0);
        assert_eq!(scene.placement_count(), 0);
        assert_eq!(
            scene.release_placement(placement_id),
            Err(Error::UnknownPlacement { placement_id })
        );
    }

    #[test]
    fn a_voxel_samples_only_a_material_of_the_scene() {
        let (mut scene, material_id, object_id) = painted();

        let mut object = RenderObject::new("p".to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();
        object
            .set_voxel_material(U32Id::from_u32(0), Some(U32Id::from_u32(9)))
            .unwrap();

        assert_eq!(
            scene.retain_object(U32Id::from_u32(4), object.clone()),
            Err(Error::VoxelMaterialRef {
                voxel_id: U32Id::from_u32(0),
                material_id: U32Id::from_u32(9),
            })
        );
        assert_eq!(
            scene.retain_object(object_id, object),
            Err(Error::DuplicateObject { object_id })
        );

        assert_eq!(
            scene.set_voxel_material(object_id, U32Id::from_u32(0), Some(U32Id::from_u32(9))),
            Err(Error::UnknownMaterial {
                material_id: U32Id::from_u32(9)
            })
        );

        scene
            .set_voxel_material(object_id, U32Id::from_u32(0), None)
            .unwrap();

        assert_eq!(scene.object(object_id).unwrap().live_count(), 0);
        scene.release_material(material_id).unwrap();
    }

    #[test]
    fn a_material_holds_to_its_ranges() {
        let mut scene = RenderScene::default();

        let too_bright = RenderMaterial {
            base_color: TyLinSrgbF64::new(1.5, 0.0, 0.0),
            ..RenderMaterial::default()
        };
        assert_eq!(
            scene.retain_material(too_bright),
            Err(Error::MaterialOutOfRange {
                property: BASE_COLOR.to_owned()
            })
        );

        let not_a_number = RenderMaterial {
            metallic: f64::NAN,
            ..RenderMaterial::default()
        };
        assert_eq!(
            scene.retain_material(not_a_number),
            Err(Error::MaterialOutOfRange {
                property: METALLIC.to_owned()
            })
        );
    }

    #[test]
    fn a_placement_needs_an_object_and_a_sound_transform() {
        let (mut scene, _, object_id) = painted();

        let placement = |transform| RenderPlacement {
            object_id,
            transform,
        };

        assert_eq!(
            scene.retain_placement(RenderPlacement {
                object_id: U32Id::from_u32(5),
                transform: TyTransformF64::IDENTITY,
            }),
            Err(Error::UnknownObject {
                object_id: U32Id::from_u32(5)
            })
        );
        assert_eq!(
            scene.retain_placement(placement(TyTransformF64 {
                scale: TyVector3F64::new(1.0, 0.0, 1.0),
                ..TyTransformF64::IDENTITY
            })),
            Err(Error::ZeroPlacementScale)
        );
        assert_eq!(
            scene.retain_placement(placement(TyTransformF64 {
                position: TyVector3F64::new(f64::INFINITY, 0.0, 0.0),
                ..TyTransformF64::IDENTITY
            })),
            Err(Error::NonFinitePlacement)
        );
        assert_eq!(
            scene.retain_placement(placement(TyTransformF64 {
                rotation: TyQuaternionF64::from_xyzw(0.0, 0.0, 0.0, 2.0),
                ..TyTransformF64::IDENTITY
            })),
            Err(Error::NonUnitPlacementRotation)
        );

        let placement_id = scene
            .retain_placement(placement(TyTransformF64::IDENTITY))
            .unwrap();
        let moved = TyTransformF64::from_translation(TyVector3F64::new(1.0, 2.0, 3.0));
        scene.set_placement_transform(placement_id, moved).unwrap();
        assert_eq!(scene.placement(placement_id).unwrap().transform, moved);
    }

    #[test]
    fn a_light_holds_to_its_checks() {
        let mut scene = RenderScene::default();

        assert_eq!(
            scene.retain_light(RenderLight::Directional {
                rotation: TyQuaternionF64::from_xyzw(0.0, 0.0, 0.0, 2.0),
                color: white(),
                strength: 1.0,
                shadow: RenderShadow::None,
            }),
            Err(Error::NonUnitLightRotation)
        );
        assert_eq!(
            scene.retain_light(RenderLight::Point {
                position: TyVector3F64::ZERO,
                color: white(),
                strength: 1.0,
                range: Some(0.0),
                shadow: RenderShadow::None,
            }),
            Err(Error::NonPositiveLightRange { range: 0.0 })
        );
        let spot = |inner_cone: f64, outer_cone: f64| RenderLight::Spot {
            position: TyVector3F64::ZERO,
            rotation: TyQuaternionF64::IDENTITY,
            color: white(),
            strength: 1.0,
            range: None,
            inner_cone,
            outer_cone,
            shadow: RenderShadow::None,
        };
        assert_eq!(
            scene.retain_light(spot(0.5, 0.5)),
            Err(Error::SpotCone {
                inner_cone: 0.5,
                outer_cone: 0.5
            })
        );
        assert_eq!(
            scene.retain_light(spot(-0.1, 0.5)),
            Err(Error::SpotCone {
                inner_cone: -0.1,
                outer_cone: 0.5
            })
        );
        assert_eq!(
            scene.retain_light(spot(0.5, 2.0)),
            Err(Error::SpotCone {
                inner_cone: 0.5,
                outer_cone: 2.0
            })
        );
        assert_eq!(
            scene.retain_light(spot(f64::NAN, 0.5)),
            Err(Error::NonFiniteLight)
        );
        let spot_id = scene.retain_light(spot(0.0, FRAC_PI_2)).unwrap();
        scene.release_light(spot_id).unwrap();
        assert_eq!(
            scene.retain_light(RenderLight::Hemisphere {
                sky: white(),
                ground: white(),
                strength: -1.0,
            }),
            Err(Error::NegativeLight)
        );
        assert_eq!(
            scene.retain_light(RenderLight::Hemisphere {
                sky: white(),
                ground: white(),
                strength: f64::NAN,
            }),
            Err(Error::NonFiniteLight)
        );

        let light_id = scene
            .retain_light(RenderLight::Hemisphere {
                sky: white(),
                ground: white(),
                strength: 1.0,
            })
            .unwrap();
        let dimmed = RenderLight::Hemisphere {
            sky: white(),
            ground: white(),
            strength: 0.5,
        };
        scene.set_light(light_id, dimmed).unwrap();
        assert_eq!(scene.light(light_id), Some(&dimmed));
        assert_eq!(scene.iter_lights().count(), 1);
        scene.release_light(light_id).unwrap();
        assert_eq!(scene.light_count(), 0);
    }

    #[test]
    fn a_view_holds_to_its_checks() {
        let mut scene = RenderScene::default();

        let view = |projection| RenderView {
            pose: TyPoseF64::IDENTITY,
            projection,
        };

        assert_eq!(
            scene.retain_view(view(RenderProjection::Perspective { fov: PI })),
            Err(Error::FieldOfViewOutOfRange { fov: PI })
        );
        assert_eq!(
            scene.retain_view(view(RenderProjection::Orthographic { scale: -1.0 })),
            Err(Error::ViewScale { scale: -1.0 })
        );
        assert_eq!(
            scene.retain_view(RenderView {
                pose: TyPoseF64::new(
                    TyVector3F64::new(f64::NAN, 0.0, 0.0),
                    TyQuaternionF64::IDENTITY
                ),
                projection: RenderProjection::Perspective { fov: 1.0 },
            }),
            Err(Error::NonFiniteView)
        );

        let view_id = scene
            .retain_view(view(RenderProjection::Perspective { fov: 1.0 }))
            .unwrap();
        let flat = view(RenderProjection::Orthographic { scale: 4.0 });
        scene.set_view(view_id, flat).unwrap();
        assert_eq!(scene.view(view_id), Some(&flat));
        scene.release_view(view_id).unwrap();
        assert_eq!(
            scene.release_view(view_id),
            Err(Error::UnknownView { view_id })
        );
    }

    #[test]
    fn a_release_keeps_the_survivors_in_order() {
        let mut scene = RenderScene::default();
        let ids: Vec<_> = (0..3)
            .map(|_| scene.retain_material(RenderMaterial::default()).unwrap())
            .collect();

        scene.release_material(ids[1]).unwrap();

        assert_eq!(
            scene.iter_materials().map(|(id, _)| id).collect::<Vec<_>>(),
            [ids[0], ids[2]]
        );
        assert_eq!(scene.material(ids[1]), None);
    }

    #[test]
    fn a_stale_id_errors_before_its_value_is_checked_and_changes_nothing() {
        let (mut scene, material_id, object_id) = painted();

        let placement_id = scene
            .retain_placement(RenderPlacement {
                object_id,
                transform: TyTransformF64::IDENTITY,
            })
            .unwrap();

        let light = RenderLight::Hemisphere {
            sky: white(),
            ground: white(),
            strength: 1.0,
        };

        let light_id = scene.retain_light(light).unwrap();

        let survivor_id = scene.retain_light(light).unwrap();

        let view = RenderView {
            pose: TyPoseF64::IDENTITY,
            projection: RenderProjection::Perspective { fov: 1.0 },
        };

        let view_id = scene.retain_view(view).unwrap();

        scene.release_placement(placement_id).unwrap();

        scene.release_light(light_id).unwrap();

        scene.release_view(view_id).unwrap();

        let flattened = TyTransformF64 {
            scale: TyVector3F64::ZERO,
            ..TyTransformF64::IDENTITY
        };

        assert_eq!(
            scene.set_placement_transform(placement_id, flattened),
            Err(Error::UnknownPlacement { placement_id })
        );

        let negative = RenderLight::Hemisphere {
            sky: white(),
            ground: white(),
            strength: -1.0,
        };

        assert_eq!(
            scene.set_light(light_id, negative),
            Err(Error::UnknownLight { light_id })
        );

        assert_eq!(
            scene.release_light(light_id),
            Err(Error::UnknownLight { light_id })
        );

        let too_wide = RenderView {
            projection: RenderProjection::Perspective { fov: PI },
            ..view
        };

        assert_eq!(
            scene.set_view(view_id, too_wide),
            Err(Error::UnknownView { view_id })
        );

        let unknown_material_id = U32Id::from_u32(9);

        assert_eq!(
            scene.release_material(unknown_material_id),
            Err(Error::UnknownMaterial {
                material_id: unknown_material_id
            })
        );

        assert_eq!(
            scene.iter_lights().collect::<Vec<_>>(),
            [(survivor_id, &light)]
        );

        assert_eq!(scene.placement_count(), 0);

        assert_eq!(scene.view_count(), 0);

        assert_eq!(
            scene.iter_materials().map(|(id, _)| id).collect::<Vec<_>>(),
            [material_id]
        );
    }

    /// A main whose palette carries a red and a blue `baseColor` over one
    /// `metallic` of `0`, a 2x1x1 bar painted red then blue at origin
    /// `(1, 0, 0)`, and a one-voxel `lone` object painted blue.
    fn painted_main() -> (VoxMain, U32Id<BVoxObject>, U32Id<BVoxObject>) {
        let mut main: VoxMain = VoxMain::default();
        let colors = main.retain_value_pool(
            VoxValuePool::vec_4_float(vec![[1.0, 0.0, 0.0, 1.0], [0.0, 0.0, 1.0, 1.0]]).unwrap(),
        );
        let metals = main.retain_value_pool(VoxValuePool::float(vec![0.0]).unwrap());

        let mut palette = VoxPalette::default();
        palette
            .retain_property(BASE_COLOR.to_owned(), colors, U32Id::from_u32(0))
            .unwrap();
        palette
            .retain_property(METALLIC.to_owned(), metals, U32Id::from_u32(0))
            .unwrap();
        for value_id in IdRange::from_len(2) {
            palette
                .retain_material(vec![value_id, U32Id::from_u32(0)])
                .unwrap();
        }
        let palette_id = main.retain_palette(palette).unwrap();

        let mut bar = VoxObject::new("bar".to_owned(), TyVector3U32::new(2, 1, 1)).unwrap();
        bar.set_origin(TyVector3I32::new(1, 0, 0));
        bar.retain_layer(palette_id, U32Id::from_u32(0));
        for x in 0..2 {
            let voxel_id = bar.voxel_id(TyVector3U32::new(x, 0, 0)).unwrap();
            bar.retain_voxel(voxel_id, &[U32Id::from_u32(x)]).unwrap();
        }
        let bar_id = main.retain_object(bar).unwrap();

        let mut lone = VoxObject::new("lone".to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();
        lone.retain_layer(palette_id, U32Id::from_u32(0));
        lone.retain_voxel(U32Id::from_u32(0), &[U32Id::from_u32(1)])
            .unwrap();
        let lone_id = main.retain_object(lone).unwrap();

        (main, bar_id, lone_id)
    }

    #[test]
    fn voxels_resolve_their_materials_and_share_the_table() {
        let (main, bar_id, lone_id) = painted_main();

        let scene = RenderScene::from_vox_main(&main, &[bar_id, lone_id], 1.0).unwrap();

        assert_eq!(scene.material_count(), 2);
        assert_eq!(scene.object_count(), 2);

        let bar = scene.object(bar_id).unwrap();
        let red_id = bar
            .voxel_material(bar.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap())
            .unwrap();
        let blue_id = bar
            .voxel_material(bar.voxel_id(TyVector3U32::new(1, 0, 0)).unwrap())
            .unwrap();
        assert_eq!(
            scene.material(red_id),
            Some(&RenderMaterial {
                base_color: TyLinSrgbF64::new(1.0, 0.0, 0.0),
                metallic: 0.0,
                ..RenderMaterial::default()
            })
        );

        let lone = scene.object(lone_id).unwrap();
        assert_eq!(lone.voxel_material(U32Id::from_u32(0)), Some(blue_id));
    }

    #[test]
    fn each_root_path_places_and_an_unplaced_object_sits_at_the_identity() {
        let (mut main, bar_id, lone_id) = painted_main();

        let leaf_id = main
            .retain_hierarchy_node(VoxHierarchyNode {
                name: "leaf".to_owned(),
                transform: TyTransformF64::from_translation(TyVector3F64::new(0.0, 2.0, 0.0)),
                child_object_ids: vec![bar_id],
                ..Default::default()
            })
            .unwrap();
        let root_transform = TyTransformF64::new(
            TyVector3F64::new(10.0, 0.0, 0.0),
            TyQuaternionF64::IDENTITY,
            TyVector3F64::splat(2.0),
        );
        let root_id = main
            .retain_hierarchy_node(VoxHierarchyNode {
                name: "root".to_owned(),
                transform: root_transform,
                child_node_ids: vec![leaf_id],
                ..Default::default()
            })
            .unwrap();
        let other_id = main
            .retain_hierarchy_node(VoxHierarchyNode {
                name: "other".to_owned(),
                child_node_ids: vec![leaf_id],
                ..Default::default()
            })
            .unwrap();
        main.set_root_hierarchy_node_ids(vec![root_id, other_id])
            .unwrap();

        let scene = RenderScene::from_vox_main(&main, &[bar_id, lone_id], 0.5).unwrap();

        let placements: Vec<_> = scene
            .iter_placements()
            .map(|(_, placement)| *placement)
            .collect();
        assert_eq!(placements.len(), 3);

        // Under root then leaf: position 0.5 * (10 + 2 * (0, 2, 0)) plus the
        // origin (1, 0, 0) at scale 2 * 0.5, then the grid at scale 2 * 0.5.
        assert_eq!(placements[0].object_id, bar_id);
        assert_eq!(
            placements[0].transform,
            TyTransformF64::new(
                TyVector3F64::new(6.0, 2.0, 0.0),
                TyQuaternionF64::IDENTITY,
                TyVector3F64::splat(1.0),
            )
        );
        assert_eq!(
            placements[0]
                .transform
                .transform_point(TyVector3F64::new(2.0, 1.0, 1.0)),
            TyVector3F64::new(8.0, 3.0, 1.0)
        );

        // Under other then leaf: the same object, a second placement.
        assert_eq!(placements[1].object_id, bar_id);
        assert_eq!(
            placements[1].transform,
            TyTransformF64::new(
                TyVector3F64::new(0.5, 1.0, 0.0),
                TyQuaternionF64::IDENTITY,
                TyVector3F64::splat(0.5),
            )
        );

        // No path reaches lone, which sits at its origin under the voxel size.
        assert_eq!(placements[2].object_id, lone_id);
        assert_eq!(
            placements[2].transform,
            TyTransformF64::new(
                TyVector3F64::ZERO,
                TyQuaternionF64::IDENTITY,
                TyVector3F64::splat(0.5),
            )
        );
    }

    #[test]
    fn a_flatten_refuses_a_bad_selection_and_a_bad_voxel_size() {
        let (main, bar_id, _) = painted_main();

        assert_eq!(
            RenderScene::from_vox_main(&main, &[U32Id::from_u32(7)], 1.0).err(),
            Some(Error::UnknownVoxObject {
                object_id: U32Id::from_u32(7)
            })
        );
        assert_eq!(
            RenderScene::from_vox_main(&main, &[bar_id, bar_id], 1.0).err(),
            Some(Error::DuplicateObject { object_id: bar_id })
        );
        assert_eq!(
            RenderScene::from_vox_main(&main, &[bar_id], 0.0).err(),
            Some(Error::VoxelSize { voxel_size: 0.0 })
        );
    }

    #[test]
    fn a_shaded_property_of_the_wrong_kind_is_refused() {
        let mut main: VoxMain = VoxMain::default();
        let ints = main.retain_value_pool(VoxValuePool::int(vec![1]).unwrap());

        let mut palette = VoxPalette::default();
        palette
            .retain_property(METALLIC.to_owned(), ints, U32Id::from_u32(0))
            .unwrap();
        palette.retain_material(vec![U32Id::from_u32(0)]).unwrap();
        let palette_id = main.retain_palette(palette).unwrap();

        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();
        object.retain_layer(palette_id, U32Id::from_u32(0));
        object
            .retain_voxel(U32Id::from_u32(0), &[U32Id::from_u32(0)])
            .unwrap();
        let object_id = main.retain_object(object).unwrap();

        assert_eq!(
            RenderScene::from_vox_main(&main, &[object_id], 1.0).err(),
            Some(Error::MaterialPropertyKind {
                property: METALLIC.to_owned()
            })
        );
    }

    #[test]
    fn the_bounds_cover_every_placement_of_the_live_voxels() {
        let mut scene = RenderScene::default();
        let material_id = scene.retain_material(RenderMaterial::default()).unwrap();

        // A 4x4x4 grid live at (1, 1, 1) and (2, 2, 2) only.
        let mut object = RenderObject::new("o".to_owned(), TyVector3U32::new(4, 4, 4)).unwrap();
        for position in [TyVector3U32::new(1, 1, 1), TyVector3U32::new(2, 2, 2)] {
            let voxel_id = object.voxel_id(position).unwrap();
            object
                .set_voxel_material(voxel_id, Some(material_id))
                .unwrap();
        }
        let object_id = U32Id::from_u32(0);
        scene.retain_object(object_id, object).unwrap();

        let placement_ids: Vec<_> = [
            TyTransformF64::IDENTITY,
            TyTransformF64::new(
                TyVector3F64::new(10.0, 0.0, 0.0),
                TyQuaternionF64::IDENTITY,
                TyVector3F64::splat(2.0),
            ),
        ]
        .into_iter()
        .map(|transform| {
            scene
                .retain_placement(RenderPlacement {
                    object_id,
                    transform,
                })
                .unwrap()
        })
        .collect();

        let bounds = scene.subject_bounds(&placement_ids).unwrap().unwrap();

        // The first placement spans 1..3 on each axis, the second 12..16 on
        // x and 2..6 on y and z.
        assert_eq!(bounds.min(), TyVector3F64::new(1.0, 1.0, 1.0));
        assert_eq!(bounds.max(), TyVector3F64::new(16.0, 6.0, 6.0));

        assert_eq!(scene.subject_bounds(&[]).unwrap(), None);
        assert!(scene.subject_bounds(&[U32Id::from_u32(9)]).is_err());
    }
}
