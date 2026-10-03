use crate::{
    Error, Result,
    operations::object::{
        LightRecord, NodeFrames, RenderElement, RenderRecord, RenderedView, resolve_light_pose,
        resolve_light_position, resolve_light_rotation, resolve_view,
    },
    utilities::select_objects,
};
use branded_id::{IdVec, U32Id, ext::IteratorExt};
use std::collections::HashSet;
use ty_math::{TyAngleUnit, TyBoundsF64, TyPoseF64};
use voxcore::{BVoxObject, Error as VoxError, VoxExt, VoxMain};
use voxrender::{
    BRenderLight, BRenderPlacement, BRenderView, RenderBloom, RenderLight, RenderOutput,
    RenderScene, render as render_image,
};

/// Renders the objects `object_ids` of `main` under `record`, one image per
/// view, placed by the hierarchy nodes reaching them. The transforms resolve
/// against the flattened scene in the order subject bounds, view, then
/// lights, so a `camera`-frame light follows each view. Errors if:
///
/// 1. `object_ids` is empty, lists an object twice, or lists one that is
///    not one of `main`'s
/// 2. an image side is zero, the voxel size is not finite and positive, or
///    a bloom value is out of range
/// 3. the record holds no view
/// 4. a view's `select` matches none of the rendered objects
/// 5. a transform cannot resolve
pub fn render<T: VoxExt>(
    main: &VoxMain<T>,
    object_ids: &[U32Id<BVoxObject>],
    record: &RenderRecord,
) -> Result<IdVec<BRenderView, RenderedView>> {
    check_objects(main, object_ids)?;

    check_record(record)?;

    let mut scene = RenderScene::from_vox_main(main, object_ids, record.voxel_size)?;

    let node_frames = NodeFrames::new(main, record.voxel_size);

    let mut outputs = IdVec::with_capacity(record.views.len());

    for view_record in record.views.iter() {
        let subject = subject_bounds(
            main,
            &scene,
            object_ids,
            &view_record.name,
            &view_record.select,
        )?;

        let view = resolve_view(
            &RenderElement::ViewTransform {
                name: view_record.name.clone(),
            },
            &view_record.transform,
            &view_record.projection,
            subject.as_ref(),
            &node_frames,
            record.width,
            record.height,
        )?;

        let view_id = scene.retain_view(view)?;

        let light_ids = record
            .lights
            .iter()
            .enumerate_ids()
            .map(|(light_id, light)| {
                let light =
                    resolve_light(light_id, light, subject.as_ref(), &node_frames, &view.pose)?;

                Ok(scene.retain_light(light)?)
            })
            .collect::<Result<Vec<_>>>()?;

        let image = render_image(
            &scene,
            view_id,
            record.occlusion,
            record.bloom,
            record.width,
            record.height,
        )?;

        for light_id in light_ids {
            scene
                .release_light(light_id)
                .expect("the light was retained for this view");
        }

        scene
            .release_view(view_id)
            .expect("the view was retained for this render");

        outputs.push(RenderedView {
            view,
            image: RenderOutput::from_image(&image, record.background),
        });
    }

    Ok(outputs)
}

/// Errors unless `object_ids` lists at least one object of `main`, each
/// once.
fn check_objects<T: VoxExt>(main: &VoxMain<T>, object_ids: &[U32Id<BVoxObject>]) -> Result<()> {
    if object_ids.is_empty() {
        return Err(Error::invalid("a run renders at least one object"));
    }

    let mut seen = HashSet::new();

    for &object_id in object_ids {
        if !seen.insert(object_id) {
            return Err(Error::invalid(format!(
                "object {} is listed twice",
                object_id.to_u32()
            )));
        }

        if main.object(object_id).is_none() {
            return Err(VoxError::UnknownObject { object_id }.into());
        }
    }

    Ok(())
}

/// Errors unless `record` has an image with two positive sides, a finite
/// positive voxel size, a bloom in range, and at least one view.
fn check_record(record: &RenderRecord) -> Result<()> {
    if record.width == 0 || record.height == 0 {
        return Err(Error::render_record(
            RenderElement::Image,
            format!(
                "is {} by {}, and each side must be greater than 0",
                record.width, record.height
            ),
        ));
    }

    if !(record.voxel_size.is_finite() && record.voxel_size > 0.0) {
        return Err(Error::render_record(
            RenderElement::VoxelSize,
            "must be finite and greater than 0",
        ));
    }

    check_bloom(&record.bloom)?;

    if record.views.is_empty() {
        return Err(Error::render_record(
            RenderElement::Views,
            "holds no view, and a run needs at least one",
        ));
    }

    Ok(())
}

/// Errors unless `bloom` has a finite strength and threshold of zero or
/// more and a finite positive radius.
fn check_bloom(bloom: &RenderBloom) -> Result<()> {
    let non_negative = |value: f64| value.is_finite() && value >= 0.0;

    if !non_negative(bloom.strength) {
        return Err(Error::render_record(
            RenderElement::Bloom,
            format!(
                "has a strength of {}, and it must be finite and 0 or more",
                bloom.strength
            ),
        ));
    }

    if !(bloom.radius.is_finite() && bloom.radius > 0.0) {
        return Err(Error::render_record(
            RenderElement::Bloom,
            format!(
                "has a radius of {}, and it must be finite and greater than 0",
                bloom.radius
            ),
        ));
    }

    if !non_negative(bloom.threshold) {
        return Err(Error::render_record(
            RenderElement::Bloom,
            format!(
                "has a threshold of {}, and it must be finite and 0 or more",
                bloom.threshold
            ),
        ));
    }

    Ok(())
}

/// The world bounds of the subject of view `name`: every placement of the
/// rendered objects, or of those `select` matches when it is not empty.
/// `None` where the subject has no live voxel. Errors if `select` matches
/// none of the rendered objects.
fn subject_bounds<T: VoxExt>(
    main: &VoxMain<T>,
    scene: &RenderScene,
    object_ids: &[U32Id<BVoxObject>],
    name: &str,
    select: &[String],
) -> Result<Option<TyBoundsF64>> {
    let rendered: HashSet<U32Id<BVoxObject>> = object_ids.iter().copied().collect();

    let subject: HashSet<U32Id<BVoxObject>> = if select.is_empty() {
        rendered
    } else {
        let matched: HashSet<_> = select_objects(main, select, &[])?
            .into_iter()
            .filter(|object_id| rendered.contains(object_id))
            .collect();

        if matched.is_empty() {
            return Err(Error::render_record(
                RenderElement::ViewSelect {
                    name: name.to_owned(),
                },
                "matches none of the rendered objects",
            ));
        }

        matched
    };

    let placement_ids: Vec<U32Id<BRenderPlacement>> = scene
        .iter_placements()
        .filter(|(_, placement)| subject.contains(&placement.object_id))
        .map(|(placement_id, _)| placement_id)
        .collect();

    Ok(scene.subject_bounds(&placement_ids)?)
}

/// The light `light_id` of the run resolved to world space for the view at
/// `view` over `subject`.
fn resolve_light(
    light_id: U32Id<BRenderLight>,
    light: &LightRecord,
    subject: Option<&TyBoundsF64>,
    node_frames: &NodeFrames,
    view: &TyPoseF64,
) -> Result<RenderLight> {
    let element = RenderElement::LightTransform { light_id };

    Ok(match light {
        LightRecord::Directional {
            transform,
            shadow,
            color,
            strength,
        } => RenderLight::Directional {
            rotation: resolve_light_rotation(&element, transform, node_frames, view)?,
            color: *color,
            strength: *strength,
            shadow: *shadow,
        },

        LightRecord::Point {
            transform,
            shadow,
            color,
            strength,
            range,
        } => RenderLight::Point {
            position: resolve_light_position(&element, transform, subject, node_frames, view)?,
            color: *color,
            strength: *strength,
            range: *range,
            shadow: *shadow,
        },

        LightRecord::Spot {
            transform,
            shadow,
            color,
            strength,
            range,
            inner_cone,
            outer_cone,
        } => {
            let pose = resolve_light_pose(&element, transform, subject, node_frames, view)?;

            RenderLight::Spot {
                position: pose.position,
                rotation: pose.rotation,
                color: *color,
                strength: *strength,
                range: *range,
                inner_cone: TyAngleUnit::Degrees.to_radians(*inner_cone),
                outer_cone: TyAngleUnit::Degrees.to_radians(*outer_cone),
                shadow: *shadow,
            }
        }

        LightRecord::Hemisphere {
            sky,
            ground,
            strength,
        } => RenderLight::Hemisphere {
            sky: *sky,
            ground: *ground,
            strength: *strength,
        },
    })
}

#[cfg(test)]
mod tests {
    use crate::{
        Error, Result,
        operations::object::{
            FitOrFixed, LightRecord, PoseTransform, RenderElement, RenderRecord, RenderedView,
            Rotation, RotationTransform, SpotTransform, ViewProjection, ViewRecord, render,
        },
        test_utilities::{HookRecorder, two_material_scene},
    };
    use branded_id::{IdVec, U32Id};
    use std::f64::consts::PI;
    use ty_math::{
        TyLinSrgbF64, TyQuaternionExt, TyQuaternionF64, TySrgbU8, TyTransformF64, TyVector3F64,
        TyVector3I32, TyVector3U32,
    };
    use voxcore::{BVoxObject, VoxHierarchyNode, VoxMain, VoxObject};
    use voxrender::{BRenderView, RenderBloom, RenderOcclusion, RenderOutput, RenderShadow};

    /// A red voxel at `x = 0` beside a blue one at `x = 1`.
    fn bar() -> VoxMain<HookRecorder> {
        two_material_scene(
            TyVector3U32::new(2, 1, 1),
            TyVector3I32::ZERO,
            &[(TyVector3U32::ZERO, 0), (TyVector3U32::X, 1)],
        )
    }

    /// The view `name` on an orbit at `azimuth` degrees, level with the
    /// subject.
    fn orbit(name: &str, azimuth: f64) -> ViewRecord {
        ViewRecord {
            name: name.to_owned(),
            transform: PoseTransform::Orbit {
                azimuth,
                elevation: 0.0,
                distance: FitOrFixed::Fit,
            },
            projection: ViewProjection::Perspective { fov: 35.0 },
            select: Vec::new(),
        }
    }

    /// A shadowless headlight that renders a white base color as white.
    fn headlight() -> LightRecord {
        LightRecord::Directional {
            transform: RotationTransform::Camera {
                rotation: Rotation::Angles {
                    azimuth: 0.0,
                    elevation: 0.0,
                },
            },
            shadow: RenderShadow::None,
            color: TyLinSrgbF64::new(1.0, 1.0, 1.0),
            strength: PI,
        }
    }

    /// An 8 by 8 render of `views` under the headlight.
    fn record(views: Vec<ViewRecord>) -> RenderRecord {
        RenderRecord {
            width: 8,
            height: 8,
            background: None,
            occlusion: RenderOcclusion::Corner,
            voxel_size: 1.0,
            bloom: RenderBloom::default(),
            views: IdVec::from(views),
            lights: IdVec::from(vec![headlight()]),
        }
    }

    /// The output of the view `index` of `outputs`.
    fn output(outputs: &IdVec<BRenderView, RenderedView>, index: u32) -> &RenderedView {
        &outputs[U32Id::<BRenderView>::from_u32(index).to_usize_id()]
    }

    /// The pixel at column `x` of row `y` of `image`.
    fn pixel(image: &RenderOutput, x: u32, y: u32) -> [u8; 4] {
        image.pixels()[(y * image.width() + x) as usize].into()
    }

    /// The leftmost opaque pixel of `image`'s middle row.
    fn leftmost_hit(image: &RenderOutput) -> [u8; 4] {
        (0..image.width())
            .map(|x| pixel(image, x, image.height() / 2))
            .find(|pixel| pixel[3] == 255)
            .expect("the bar crosses the middle row")
    }

    fn close(a: TyVector3F64, b: TyVector3F64) -> bool {
        (a - b).length() < 1e-9
    }

    #[test]
    fn each_view_yields_an_image_of_the_record_s_size() {
        let main = bar();

        let outputs = render(
            &main,
            &[U32Id::from_u32(0)],
            &record(vec![orbit("front", 0.0)]),
        )
        .unwrap();

        let [RenderedView { view, image }] = outputs.as_slice() else {
            panic!("one view, one output");
        };

        // The front view sits on +Z looking down -Z, the identity rotation.
        assert!(view.pose.position.z > 1.0);
        assert!(
            view.pose
                .rotation
                .is_approximately_equal(TyQuaternionF64::IDENTITY, 1e-9)
        );

        assert_eq!((image.width(), image.height()), (8, 8));

        // The corners miss the bar and the middle row crosses it, red on the
        // left under the headlight. The palette's metals reflect only their
        // color.
        assert_eq!(pixel(image, 0, 0)[3], 0);
        let [red, green, blue, alpha] = leftmost_hit(image);
        assert_eq!(alpha, 255);
        assert!(red > 100 && green < 20 && blue < 20, "{red} {green} {blue}");
    }

    #[test]
    fn a_camera_light_follows_each_view_and_the_outputs_keep_the_view_order() {
        let main = bar();

        let outputs = render(
            &main,
            &[U32Id::from_u32(0)],
            &record(vec![orbit("front", 0.0), orbit("back", 180.0)]),
        )
        .unwrap();

        assert_eq!(outputs.len(), 2);
        assert!(output(&outputs, 0).view.pose.position.z > 0.0);
        assert!(output(&outputs, 1).view.pose.position.z < 0.0);

        // From the back the blue voxel is on the left, and the headlight
        // turned with the view still lights it.
        let [red, _, blue, _] = leftmost_hit(&output(&outputs, 0).image);
        assert!(red > blue);
        let [red, _, blue, _] = leftmost_hit(&output(&outputs, 1).image);
        assert!(blue > 100 && red < 20, "{red} {blue}");
    }

    #[test]
    fn a_subject_spot_lights_the_bar_inside_its_cone_alone() {
        let main = bar();

        // The spot sits 3.5 past the bar's front face, which it aims at. The
        // bar's left end is 16 degrees off its axis.
        let spot = |inner_cone, outer_cone| LightRecord::Spot {
            transform: SpotTransform::Subject {
                position: TyVector3F64::new(0.0, 0.0, 4.0),
                rotation: Rotation::LookAt { target: None },
            },
            shadow: RenderShadow::None,
            color: TyLinSrgbF64::new(1.0, 1.0, 1.0),
            strength: PI * 3.5 * 3.5,
            range: None,
            inner_cone,
            outer_cone,
        };

        let mut wide = record(vec![orbit("front", 0.0)]);
        wide.lights = IdVec::from(vec![spot(30.0, 60.0)]);
        let outputs = render(&main, &[U32Id::from_u32(0)], &wide).unwrap();
        let [red, green, blue, _] = leftmost_hit(&output(&outputs, 0).image);
        assert!(red > 100 && green < 20 && blue < 20, "{red} {green} {blue}");

        let mut narrow = record(vec![orbit("front", 0.0)]);
        narrow.lights = IdVec::from(vec![spot(1.0, 2.0)]);
        let outputs = render(&main, &[U32Id::from_u32(0)], &narrow).unwrap();
        assert_eq!(leftmost_hit(&output(&outputs, 0).image), [0, 0, 0, 255]);

        let mut reversed = record(vec![orbit("front", 0.0)]);
        reversed.lights = IdVec::from(vec![spot(60.0, 30.0)]);
        assert!(render(&main, &[U32Id::from_u32(0)], &reversed).is_err());
    }

    #[test]
    fn the_background_fills_the_misses() {
        let main = bar();

        let mut record = record(vec![orbit("front", 0.0)]);
        record.background = Some(TySrgbU8::new(1, 2, 3));

        let outputs = render(&main, &[U32Id::from_u32(0)], &record).unwrap();

        assert_eq!(pixel(&output(&outputs, 0).image, 0, 0), [1, 2, 3, 255]);
    }

    #[test]
    fn a_select_narrows_the_subject_the_orbit_is_about() {
        let mut main = bar();

        let mut far = VoxObject::new("far".to_owned(), TyVector3U32::ONE).unwrap();
        far.set_origin(TyVector3I32::new(10, 0, 0));
        far.retain_voxel(far.voxel_id(TyVector3U32::ZERO).unwrap(), &[])
            .unwrap();
        let far_id = main.retain_object(far).unwrap();

        let object_ids = [U32Id::from_u32(0), far_id];

        let both = render(&main, &object_ids, &record(vec![orbit("front", 0.0)])).unwrap();
        assert!((output(&both, 0).view.pose.position.x - 5.5).abs() < 1e-9);

        let mut narrowed = orbit("front", 0.0);
        narrowed.select = vec!["far".to_owned()];

        let far_alone = render(&main, &object_ids, &record(vec![narrowed])).unwrap();
        assert!((output(&far_alone, 0).view.pose.position.x - 10.5).abs() < 1e-9);
    }

    #[test]
    fn a_node_frame_view_rides_the_node_placing_the_bar() {
        let mut main = bar();

        // The node turns its +Z to world +X, doubles, and sits at x = 10.
        let node_id = main
            .retain_hierarchy_node(VoxHierarchyNode {
                name: "player".to_owned(),
                transform: TyTransformF64::new(
                    TyVector3F64::new(10.0, 0.0, 0.0),
                    TyQuaternionF64::from_axis_angle(TyVector3F64::Y, 90f64.to_radians()),
                    TyVector3F64::splat(2.0),
                ),
                child_object_ids: vec![U32Id::from_u32(0)],
                ..Default::default()
            })
            .unwrap();
        main.push_root_hierarchy_node_id(node_id).unwrap();

        let mut riding = orbit("ride", 0.0);
        riding.transform = PoseTransform::Node {
            path: "player".to_owned(),
            position: TyVector3F64::new(0.0, 0.5, 5.0),
            rotation: Rotation::LookAt {
                target: Some(TyVector3F64::new(0.0, 0.5, 0.0)),
            },
        };

        let outputs = render(&main, &[U32Id::from_u32(0)], &record(vec![riding])).unwrap();
        let RenderedView { view, image } = output(&outputs, 0);

        // The offset doubles and turns with the node: 5 along its +Z lands 10
        // along world +X past its origin. The look back runs down -X.
        assert!(close(view.pose.position, TyVector3F64::new(20.0, 1.0, 0.0)));
        assert!(close(
            view.pose.rotation * -TyVector3F64::Z,
            -TyVector3F64::X
        ));

        // The bar rides the same node, so the view sees its red end.
        let [red, green, blue, alpha] = leftmost_hit(image);
        assert_eq!(alpha, 255);
        assert!(red > 100 && green < 20 && blue < 20, "{red} {green} {blue}");
    }

    #[test]
    fn a_bad_selection_errors() {
        let main = bar();
        let record = record(vec![orbit("front", 0.0)]);

        assert!(render(&main, &[], &record).is_err());

        let twice = U32Id::<BVoxObject>::from_u32(0);
        assert!(render(&main, &[twice, twice], &record).is_err());

        assert!(render(&main, &[U32Id::from_u32(7)], &record).is_err());
    }

    #[test]
    fn a_bad_record_errors_on_its_element() {
        let main = bar();
        let object_ids = [U32Id::from_u32(0)];

        let element = |result: Result<IdVec<BRenderView, RenderedView>>| match result {
            Err(Error::RenderRecord { element, .. }) => element,
            other => panic!("{other:?}"),
        };

        let mut zero = record(vec![orbit("front", 0.0)]);
        zero.height = 0;
        assert_eq!(
            element(render(&main, &object_ids, &zero)),
            RenderElement::Image
        );

        let mut sized = record(vec![orbit("front", 0.0)]);
        sized.voxel_size = 0.0;
        assert_eq!(
            element(render(&main, &object_ids, &sized)),
            RenderElement::VoxelSize
        );

        for bloom in [
            RenderBloom {
                strength: -1.0,
                ..RenderBloom::default()
            },
            RenderBloom {
                radius: 0.0,
                ..RenderBloom::default()
            },
            RenderBloom {
                threshold: f64::NAN,
                ..RenderBloom::default()
            },
        ] {
            let mut glowing = record(vec![orbit("front", 0.0)]);
            glowing.bloom = bloom;
            assert_eq!(
                element(render(&main, &object_ids, &glowing)),
                RenderElement::Bloom,
                "{bloom:?}"
            );
        }

        assert_eq!(
            element(render(&main, &object_ids, &record(Vec::new()))),
            RenderElement::Views
        );

        let mut lost = orbit("front", 0.0);
        lost.select = vec!["nowhere".to_owned()];
        assert_eq!(
            element(render(&main, &object_ids, &record(vec![lost]))),
            RenderElement::ViewSelect {
                name: "front".to_owned()
            }
        );

        let mut looking_inward = orbit("front", 0.0);
        looking_inward.transform = PoseTransform::World {
            position: TyVector3F64::ZERO,
            rotation: Rotation::LookAt { target: None },
        };
        assert_eq!(
            element(render(&main, &object_ids, &record(vec![looking_inward]))),
            RenderElement::ViewTransform {
                name: "front".to_owned()
            }
        );

        let mut unmounted = orbit("front", 0.0);
        unmounted.transform = PoseTransform::Node {
            path: "player".to_owned(),
            position: TyVector3F64::ZERO,
            rotation: Rotation::LookAt { target: None },
        };
        assert_eq!(
            element(render(&main, &object_ids, &record(vec![unmounted]))),
            RenderElement::ViewTransform {
                name: "front".to_owned()
            }
        );
    }
}
