use crate::{
    BRenderView, Error, RenderBloom, RenderHit, RenderImage, RenderLight, RenderMaterial,
    RenderOcclusion, RenderRay, RenderScene, RenderShadow, RenderViewRays, Result, ShadowTarget,
    apply_bloom, cast_ray,
};
use branded_id::U32Id;
use std::f64::consts::PI;
use ty_math::{TyLinSrgbF32, TyLinSrgbF64, TyLinSrgbaF32, TyVector3F64};
use voxsurface::{SurfaceSpan, corner_occlusion};

/// The GGX alpha floor that keeps a mirror's lobe finite.
const MIN_ALPHA: f64 = 1e-3;

/// The reflectance floor of a dielectric at normal incidence.
const DIELECTRIC_F0: f64 = 0.04;

/// glTF's floor under the gap between a spot's cone cosines. It keeps a cone
/// whose angles sit a rounding apart finite.
const MIN_CONE_WIDTH: f64 = 1e-3;

/// How far a shadow ray's start sits out from its face, in grid units.
const BIAS: f64 = 1e-4;

/// The least distance from a face's edges to a sample, as a fraction of the
/// face. It keeps a corner sample in front of the hit's cell.
const INSET: f64 = 1e-3;

/// Renders the view `view_id` of `scene` into a `width` by `height` image
/// under every light of the scene. `occlusion` decides whether the corner
/// occlusion darkens the hemisphere light. A pixel a ray hits is opaque. A
/// pixel no ray hits stays transparent black until `bloom`'s halo reaches
/// it. Errors if the view is not one of the scene's, a side is zero, or a
/// bloom value is out of range.
pub fn render(
    scene: &RenderScene,
    view_id: U32Id<BRenderView>,
    occlusion: RenderOcclusion,
    bloom: RenderBloom,
    width: u32,
    height: u32,
) -> Result<RenderImage> {
    if width == 0 || height == 0 {
        return Err(Error::ImageSide { width, height });
    }

    check_bloom(bloom)?;

    let view = scene.view(view_id).ok_or(Error::UnknownView { view_id })?;

    let rays = RenderViewRays::new(view, width, height);
    let mut image = RenderImage::new(width, height);
    let mut emission = vec![TyLinSrgbF32::new(0.0, 0.0, 0.0); width as usize * height as usize];

    for y in 0..height {
        for x in 0..width {
            let ray = rays.ray(x, y);

            if let Some(hit) = cast_ray(scene, &ray, f64::INFINITY) {
                let shade = shade_hit(scene, occlusion, &ray, &hit);

                image.set_pixel(
                    x,
                    y,
                    TyLinSrgbaF32::new(
                        shade.color.red as f32,
                        shade.color.green as f32,
                        shade.color.blue as f32,
                        1.0,
                    ),
                );

                emission[y as usize * width as usize + x as usize] = TyLinSrgbF32::new(
                    shade.emission.red as f32,
                    shade.emission.green as f32,
                    shade.emission.blue as f32,
                );
            }
        }
    }

    if bloom.strength > 0.0 {
        apply_bloom(&mut image, &emission, bloom);
    }

    Ok(image)
}

/// Errors unless `bloom` has a finite strength and threshold of zero or
/// more and a finite positive radius.
fn check_bloom(bloom: RenderBloom) -> Result<()> {
    if !(bloom.strength.is_finite() && bloom.strength >= 0.0) {
        return Err(Error::BloomStrength {
            strength: bloom.strength,
        });
    }

    if !(bloom.radius.is_finite() && bloom.radius > 0.0) {
        return Err(Error::BloomRadius {
            radius: bloom.radius,
        });
    }

    if !(bloom.threshold.is_finite() && bloom.threshold >= 0.0) {
        return Err(Error::BloomThreshold {
            threshold: bloom.threshold,
        });
    }

    Ok(())
}

/// What a hit shades to.
struct HitShade {
    /// The linear color, lit by every light plus the emission.
    color: TyLinSrgbF64,

    /// The emissive term within `color`, which the bloom reads.
    emission: TyLinSrgbF64,
}

/// The shade of `hit` seen along `ray` under every light of `scene`.
/// `occlusion` decides whether the corner occlusion darkens the hemisphere
/// term.
fn shade_hit(
    scene: &RenderScene,
    occlusion: RenderOcclusion,
    ray: &RenderRay,
    hit: &RenderHit,
) -> HitShade {
    let placement = scene
        .placement(hit.placement_id)
        .expect("a hit lands on one of the scene's placements");

    let object = scene
        .object(placement.object_id)
        .expect("a placement's object is one of the scene's");

    let material = scene
        .material(
            object
                .voxel_material(hit.voxel_id)
                .expect("a hit lands on a live voxel"),
        )
        .expect("a voxel samples one of the scene's materials");

    let transform = &placement.transform;

    let normal =
        (transform.rotation * (hit.face.normal().as_dvec3() / transform.scale)).normalize();
    let view = -ray.direction;
    let point = ray.origin + ray.direction * hit.distance;

    let open = match occlusion {
        RenderOcclusion::None => 1.0,

        RenderOcclusion::Corner => {
            bilinear(&hit.face, corner_occlusion(object, &hit.face), hit.along)
        }
    };

    let emission = material.emissive_color * material.emissive_strength;
    let mut color = emission;

    for (_, light) in scene.iter_lights() {
        match *light {
            RenderLight::Directional {
                rotation,
                color: light_color,
                strength,
                shadow,
            } => {
                let toward = rotation * TyVector3F64::Z;
                let radiance = direct_radiance(material, normal, view, toward);

                if radiance == TyLinSrgbF64::new(0.0, 0.0, 0.0) {
                    continue;
                }

                let lit = shadow_factor(scene, hit, shadow, &ShadowTarget::Direction(toward));

                color += radiance * light_color * (strength * lit);
            }

            RenderLight::Point {
                position,
                color: light_color,
                strength,
                range,
                shadow,
            } => {
                let to_light = position - point;
                let distance = to_light.length();

                if distance == 0.0 {
                    continue;
                }

                let radiance = direct_radiance(material, normal, view, to_light / distance);

                if radiance == TyLinSrgbF64::new(0.0, 0.0, 0.0) {
                    continue;
                }

                let reach = point_attenuation(distance, range);
                let lit = shadow_factor(scene, hit, shadow, &ShadowTarget::Position(position));

                color += radiance * light_color * (strength * reach * lit);
            }

            RenderLight::Spot {
                position,
                rotation,
                color: light_color,
                strength,
                range,
                inner_cone,
                outer_cone,
                shadow,
            } => {
                let to_light = position - point;
                let distance = to_light.length();

                if distance == 0.0 {
                    continue;
                }

                let toward = to_light / distance;
                let radiance = direct_radiance(material, normal, view, toward);

                if radiance == TyLinSrgbF64::new(0.0, 0.0, 0.0) {
                    continue;
                }

                let axis = rotation * -TyVector3F64::Z;
                let reach = point_attenuation(distance, range)
                    * cone_attenuation(axis.dot(-toward), inner_cone, outer_cone);

                if reach == 0.0 {
                    continue;
                }

                let lit = shadow_factor(scene, hit, shadow, &ShadowTarget::Position(position));

                color += radiance * light_color * (strength * reach * lit);
            }

            RenderLight::Hemisphere {
                sky,
                ground,
                strength,
            } => {
                color += hemisphere_radiance(material, normal, sky, ground, strength, open);
            }
        }
    }

    HitShade { color, emission }
}

/// The light `material` reflects toward `view` from a unit radiance
/// arriving along `light`, scaled by the cosine of incidence. The model is
/// glTF's metallic-roughness: Lambert diffuse and GGX specular with Smith
/// visibility and Schlick Fresnel. Returns black when the light is behind
/// the surface.
///
/// # Arguments
/// * `normal` - the surface's unit normal.
/// * `view` - the unit direction toward the viewer.
/// * `light` - the unit direction toward the light.
fn direct_radiance(
    material: &RenderMaterial,
    normal: TyVector3F64,
    view: TyVector3F64,
    light: TyVector3F64,
) -> TyLinSrgbF64 {
    let n_dot_l = normal.dot(light);

    if n_dot_l <= 0.0 {
        return TyLinSrgbF64::new(0.0, 0.0, 0.0);
    }

    let n_dot_v = normal.dot(view).max(1e-4);
    let half = (light + view).normalize();
    let n_dot_h = normal.dot(half).max(0.0);
    let v_dot_h = view.dot(half).max(0.0);

    let alpha = (material.roughness * material.roughness).max(MIN_ALPHA);
    let alpha2 = alpha * alpha;

    let white = TyLinSrgbF64::new(1.0, 1.0, 1.0);
    let f0 = normal_reflectance(material);
    let fresnel = f0 + (white - f0) * (1.0 - v_dot_h).powi(5);

    let distribution = alpha2 / (PI * (n_dot_h * n_dot_h * (alpha2 - 1.0) + 1.0).powi(2));

    let visibility = 0.5
        / (n_dot_l * (n_dot_v * n_dot_v * (1.0 - alpha2) + alpha2).sqrt()
            + n_dot_v * (n_dot_l * n_dot_l * (1.0 - alpha2) + alpha2).sqrt());

    let diffuse = (white - fresnel) * material.base_color * ((1.0 - material.metallic) / PI);
    let specular = fresnel * (distribution * visibility);

    (diffuse + specular) * n_dot_l
}

/// The light `material` reflects from a hemisphere light, darkened toward
/// `occlusion` by the material's occlusion strength. The ambient light
/// reflects off the diffuse color plus the normal-incidence reflectance,
/// which gives a metal a reflection in its base color.
///
/// # Arguments
/// * `normal` - the surface's unit normal, whose world +Y mixes `sky`
///   above with `ground` below.
/// * `occlusion` - the corner occlusion, from `0` fully occluded to `1`
///   open.
fn hemisphere_radiance(
    material: &RenderMaterial,
    normal: TyVector3F64,
    sky: TyLinSrgbF64,
    ground: TyLinSrgbF64,
    strength: f64,
    occlusion: f64,
) -> TyLinSrgbF64 {
    let up = (normal.y + 1.0) / 2.0;
    let ambient = (sky * up + ground * (1.0 - up)) * strength;

    let f0 = normal_reflectance(material);
    let diffuse = material.base_color * (1.0 - material.metallic);

    let open = 1.0 - material.occlusion_strength * (1.0 - occlusion);

    ambient * (diffuse + f0) * open
}

/// The reflectance of `material` at normal incidence.
fn normal_reflectance(material: &RenderMaterial) -> TyLinSrgbF64 {
    let dielectric = TyLinSrgbF64::new(DIELECTRIC_F0, DIELECTRIC_F0, DIELECTRIC_F0);

    dielectric * (1.0 - material.metallic) + material.base_color * material.metallic
}

/// How much of a point light reaches `distance` meters out: the inverse
/// square, times glTF's smooth window that reaches zero at `range`.
fn point_attenuation(distance: f64, range: Option<f64>) -> f64 {
    let window = match range {
        None => 1.0,
        Some(range) => (1.0 - (distance / range).powi(4)).clamp(0.0, 1.0).powi(2),
    };

    window / (distance * distance)
}

/// How much of a spot light reaches a point: glTF's smooth ramp from full
/// inside `inner` to none past `outer`.
///
/// # Arguments
/// * `cosine` - the cosine of the angle between the light's -Z and the
///   direction from the light to the point.
/// * `inner`, `outer` - the cone half-angles in radians.
fn cone_attenuation(cosine: f64, inner: f64, outer: f64) -> f64 {
    let scale = 1.0 / (inner.cos() - outer.cos()).max(MIN_CONE_WIDTH);
    let offset = -outer.cos() * scale;

    (cosine * scale + offset).clamp(0.0, 1.0).powi(2)
}

/// How lit `hit` is by a light at `target`, from `0` in shadow to `1` in
/// the open, with rays cast at the `shadow` granularity.
fn shadow_factor(
    scene: &RenderScene,
    hit: &RenderHit,
    shadow: RenderShadow,
    target: &ShadowTarget,
) -> f64 {
    let placement = scene
        .placement(hit.placement_id)
        .expect("a hit lands on one of the scene's placements");

    let sample = |along: [f64; 2]| {
        let origin = placement
            .transform
            .transform_point(face_point(&hit.face, along, BIAS));

        let (direction, max_distance) = match *target {
            ShadowTarget::Direction(direction) => (direction, f64::INFINITY),

            ShadowTarget::Position(position) => {
                let to_light = position - origin;
                let distance = to_light.length();

                (to_light / distance, distance)
            }
        };

        let ray = RenderRay { origin, direction };

        if cast_ray(scene, &ray, max_distance).is_some() {
            0.0
        } else {
            1.0
        }
    };

    let inset = |along: f64| along.clamp(INSET, 1.0 - INSET);

    match shadow {
        RenderShadow::None => 1.0,

        RenderShadow::PerPixel => sample(hit.along.map(inset)),

        RenderShadow::PerFace => sample([0.5, 0.5]),

        RenderShadow::PerCorner => {
            let values = hit.face.corners().map(|[uu, vv]| {
                sample([
                    inset(if uu == hit.face.u0 { 0.0 } else { 1.0 }),
                    inset(if vv == hit.face.v0 { 0.0 } else { 1.0 }),
                ])
            });

            bilinear(&hit.face, values, hit.along)
        }
    }
}

/// The point on `face` at `along`, the fraction across its `u` and `v`,
/// pushed `bias` grid units out along its normal.
fn face_point(face: &SurfaceSpan, along: [f64; 2], bias: f64) -> TyVector3F64 {
    let mut point = [0.0; 3];

    point[face.d] =
        f64::from(face.s) + if face.sign > 0 { 1.0 } else { 0.0 } + f64::from(face.sign) * bias;
    point[face.u()] = face.u0 as f64 + along[0] * (face.u1 - face.u0) as f64;
    point[face.v()] = face.v0 as f64 + along[1] * (face.v1 - face.v0) as f64;

    TyVector3F64::from_array(point)
}

/// `values` blended bilinearly across `face` at `along`.
///
/// # Arguments
/// * `values` - one value per corner, in the order of the face's
///   [`corners`](SurfaceSpan::corners).
/// * `along` - the fraction across the face's `u` and `v`.
fn bilinear(face: &SurfaceSpan, values: [f64; 4], along: [f64; 2]) -> f64 {
    face.corners()
        .iter()
        .zip(values)
        .map(|([uu, vv], value)| {
            let weight_u = if *uu == face.u0 {
                1.0 - along[0]
            } else {
                along[0]
            };
            let weight_v = if *vv == face.v0 {
                1.0 - along[1]
            } else {
                along[1]
            };

            value * weight_u * weight_v
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use crate::{
        Error, RenderBloom, RenderLight, RenderMaterial, RenderObject, RenderOcclusion,
        RenderPlacement, RenderProjection, RenderRay, RenderScene, RenderShadow, RenderView,
        ShadowTarget, cast_ray, fit_distance, render,
        render::{
            bilinear, check_bloom, cone_attenuation, direct_radiance, hemisphere_radiance,
            point_attenuation, shade_hit, shadow_factor,
        },
        test_utilities::{
            check_goldens, cube_scene, glow_scene, l_shape_scene, room_scene, solid_object,
            spot_room_scene, two_placements_scene,
        },
    };
    use branded_id::U32Id;
    use ty_math::{
        TyLinSrgbF64, TyPoseF64, TyQuaternionExt, TyQuaternionF64, TyTransformF64, TyVector3Ext,
        TyVector3F64, TyVector3U32,
    };
    use voxsurface::SurfaceSpan;

    #[test]
    fn hits_are_opaque_and_misses_transparent() {
        let mut scene = RenderScene::default();
        let material_id = scene.retain_material(RenderMaterial::default()).unwrap();
        let mut object = RenderObject::new("c".to_owned(), TyVector3U32::ONE).unwrap();
        object
            .set_voxel_material(
                object.voxel_id(TyVector3U32::ZERO).unwrap(),
                Some(material_id),
            )
            .unwrap();
        let object_id = U32Id::from_u32(0);
        scene.retain_object(object_id, object).unwrap();
        let placement_id = scene
            .retain_placement(RenderPlacement {
                object_id,
                transform: TyTransformF64::IDENTITY,
            })
            .unwrap();
        scene
            .retain_light(RenderLight::Hemisphere {
                sky: TyLinSrgbF64::new(1.0, 1.0, 1.0),
                ground: TyLinSrgbF64::new(1.0, 1.0, 1.0),
                strength: 1.0,
            })
            .unwrap();

        let bounds = scene.subject_bounds(&[placement_id]).unwrap().unwrap();
        let direction =
            TyVector3F64::from_azimuth_elevation(45f64.to_radians(), 30f64.to_radians());
        let fov = 35f64.to_radians();
        let view_id = scene
            .retain_view(RenderView {
                pose: TyPoseF64::new(
                    bounds.center + direction * fit_distance(&bounds, fov, 8, 8),
                    TyQuaternionF64::from_look_direction(-direction, TyVector3F64::Y).unwrap(),
                ),
                projection: RenderProjection::Perspective { fov },
            })
            .unwrap();

        let image = render(
            &scene,
            view_id,
            RenderOcclusion::Corner,
            RenderBloom::default(),
            8,
            8,
        )
        .unwrap();

        let center = image.pixel(4, 4).unwrap();
        assert_eq!(center.alpha, 1.0);
        assert!(center.red > 0.0);
        assert_eq!(image.pixel(0, 0).unwrap().alpha, 0.0);
        assert_eq!(image.pixel(7, 7).unwrap().alpha, 0.0);

        assert_eq!(
            render(
                &scene,
                U32Id::from_u32(9),
                RenderOcclusion::Corner,
                RenderBloom::default(),
                8,
                8
            )
            .err(),
            Some(Error::UnknownView {
                view_id: U32Id::from_u32(9)
            })
        );
        assert_eq!(
            render(
                &scene,
                view_id,
                RenderOcclusion::Corner,
                RenderBloom::default(),
                0,
                8
            )
            .err(),
            Some(Error::ImageSide {
                width: 0,
                height: 8
            })
        );
    }

    #[test]
    fn a_bloom_value_out_of_range_errors() {
        let bloom = RenderBloom::default();

        assert_eq!(check_bloom(bloom), Ok(()));
        assert_eq!(
            check_bloom(RenderBloom {
                strength: -1.0,
                ..bloom
            }),
            Err(Error::BloomStrength { strength: -1.0 })
        );
        assert_eq!(
            check_bloom(RenderBloom {
                radius: 0.0,
                ..bloom
            }),
            Err(Error::BloomRadius { radius: 0.0 })
        );
        assert!(
            check_bloom(RenderBloom {
                radius: f64::INFINITY,
                ..bloom
            })
            .is_err()
        );
        assert!(
            check_bloom(RenderBloom {
                threshold: f64::NAN,
                ..bloom
            })
            .is_err()
        );
    }

    /// One white cube at the origin.
    fn cube() -> RenderScene {
        let mut scene = RenderScene::default();
        let material_id = scene
            .retain_material(RenderMaterial {
                metallic: 0.0,
                roughness: 0.8,
                ..RenderMaterial::default()
            })
            .unwrap();
        let mut object = RenderObject::new("c".to_owned(), TyVector3U32::ONE).unwrap();
        object
            .set_voxel_material(
                object.voxel_id(TyVector3U32::ZERO).unwrap(),
                Some(material_id),
            )
            .unwrap();
        let object_id = U32Id::from_u32(0);
        scene.retain_object(object_id, object).unwrap();
        scene
            .retain_placement(RenderPlacement {
                object_id,
                transform: TyTransformF64::IDENTITY,
            })
            .unwrap();
        scene
    }

    /// The studio rig under a view from the front-right-top.
    fn studio(scene: &mut RenderScene) {
        let view = TyQuaternionF64::from_look_direction(
            -TyVector3F64::from_azimuth_elevation(45f64.to_radians(), 30f64.to_radians()),
            TyVector3F64::Y,
        )
        .unwrap();
        let from = TyVector3F64::from_azimuth_elevation(-30f64.to_radians(), 30f64.to_radians());
        let rotation = view * TyQuaternionF64::from_look_direction(-from, TyVector3F64::Y).unwrap();

        scene
            .retain_light(RenderLight::Directional {
                rotation,
                color: TyLinSrgbF64::new(1.0, 1.0, 1.0),
                strength: 3.0,
                shadow: RenderShadow::PerPixel,
            })
            .unwrap();
        scene
            .retain_light(RenderLight::Hemisphere {
                sky: TyLinSrgbF64::new(0.4, 0.45, 0.5),
                ground: TyLinSrgbF64::new(0.15, 0.12, 0.1),
                strength: 1.0,
            })
            .unwrap();
    }

    /// The shade of the cube's face on the `+axis` side, seen head on.
    fn face_shade(scene: &RenderScene, axis: usize) -> f64 {
        let mut origin = [0.5; 3];
        origin[axis] = 4.0;
        let mut direction = [0.0; 3];
        direction[axis] = -1.0;
        let ray = RenderRay {
            origin: TyVector3F64::from_array(origin),
            direction: TyVector3F64::from_array(direction),
        };
        let hit = cast_ray(scene, &ray, f64::INFINITY).unwrap();
        let color = shade_hit(scene, RenderOcclusion::Corner, &ray, &hit).color;
        color.red + color.green + color.blue
    }

    #[test]
    fn the_studio_rig_gives_a_cube_three_shades() {
        let mut scene = cube();
        studio(&mut scene);

        let shades = [
            face_shade(&scene, 0),
            face_shade(&scene, 1),
            face_shade(&scene, 2),
        ];

        for (index, shade) in shades.iter().enumerate() {
            assert!(shade.is_finite() && *shade > 0.0);
            for other in &shades[index + 1..] {
                assert!((shade - other).abs() > 0.01, "{shades:?}");
            }
        }
    }

    #[test]
    fn emission_adds_and_a_light_behind_the_face_adds_nothing() {
        let mut scene = RenderScene::default();
        let material_id = scene
            .retain_material(RenderMaterial {
                emissive_color: TyLinSrgbF64::new(1.0, 0.5, 0.0),
                emissive_strength: 2.0,
                ..RenderMaterial::default()
            })
            .unwrap();
        let mut object = RenderObject::new("e".to_owned(), TyVector3U32::ONE).unwrap();
        object
            .set_voxel_material(
                object.voxel_id(TyVector3U32::ZERO).unwrap(),
                Some(material_id),
            )
            .unwrap();
        let object_id = U32Id::from_u32(0);
        scene.retain_object(object_id, object).unwrap();
        scene
            .retain_placement(RenderPlacement {
                object_id,
                transform: TyTransformF64::IDENTITY,
            })
            .unwrap();
        // A light shining down +Y lights the bottom, not the top.
        scene
            .retain_light(RenderLight::Directional {
                rotation: TyQuaternionF64::from_look_direction(TyVector3F64::Y, -TyVector3F64::Z)
                    .unwrap(),
                color: TyLinSrgbF64::new(1.0, 1.0, 1.0),
                strength: 1.0,
                shadow: RenderShadow::PerPixel,
            })
            .unwrap();

        let ray = RenderRay {
            origin: TyVector3F64::new(0.5, 4.0, 0.5),
            direction: -TyVector3F64::Y,
        };
        let hit = cast_ray(&scene, &ray, f64::INFINITY).unwrap();

        let shade = shade_hit(&scene, RenderOcclusion::None, &ray, &hit);
        assert_eq!(shade.color, TyLinSrgbF64::new(2.0, 1.0, 0.0));
        assert_eq!(shade.emission, shade.color);
    }

    #[test]
    fn light_behind_the_surface_is_black_and_a_metal_reflects_its_own_color() {
        let rough = RenderMaterial {
            base_color: TyLinSrgbF64::new(0.5, 0.5, 0.5),
            metallic: 0.0,
            ..RenderMaterial::default()
        };
        let normal = TyVector3F64::Y;
        let view = TyVector3F64::new(0.0, 1.0, 1.0).normalize();

        let behind = direct_radiance(&rough, normal, view, -TyVector3F64::Y);
        assert_eq!(behind, TyLinSrgbF64::new(0.0, 0.0, 0.0));

        let overhead = direct_radiance(&rough, normal, view, TyVector3F64::Y);
        let grazing = direct_radiance(
            &rough,
            normal,
            view,
            TyVector3F64::new(1.0, 0.1, 0.0).normalize(),
        );
        assert!(overhead.red > grazing.red && grazing.red > 0.0);
        assert!(overhead.red.is_finite());

        let red_metal = RenderMaterial {
            base_color: TyLinSrgbF64::new(1.0, 0.0, 0.0),
            metallic: 1.0,
            roughness: 0.3,
            ..RenderMaterial::default()
        };
        let shine = direct_radiance(&red_metal, normal, TyVector3F64::Y, TyVector3F64::Y);
        assert!(shine.red > 0.0);
        assert_eq!(shine.green, 0.0);
        assert_eq!(shine.blue, 0.0);
    }

    #[test]
    fn the_sky_lights_the_top_the_ground_the_bottom_and_occlusion_darkens() {
        let white = RenderMaterial {
            metallic: 0.0,
            ..RenderMaterial::default()
        };
        let sky = TyLinSrgbF64::new(1.0, 1.0, 1.0);
        let ground = TyLinSrgbF64::new(0.0, 0.0, 0.0);

        let top = hemisphere_radiance(&white, TyVector3F64::Y, sky, ground, 1.0, 1.0);
        let side = hemisphere_radiance(&white, TyVector3F64::X, sky, ground, 1.0, 1.0);
        let bottom = hemisphere_radiance(&white, -TyVector3F64::Y, sky, ground, 1.0, 1.0);
        assert!((top.red - 1.04).abs() < 1e-9);
        assert!((side.red - 0.52).abs() < 1e-9);
        assert_eq!(bottom.red, 0.0);

        let closed = hemisphere_radiance(&white, TyVector3F64::Y, sky, ground, 1.0, 0.0);
        assert_eq!(closed.red, 0.0);

        let half_strength = RenderMaterial {
            occlusion_strength: 0.5,
            ..white
        };
        let softened = hemisphere_radiance(&half_strength, TyVector3F64::Y, sky, ground, 1.0, 0.0);
        assert!((softened.red - 0.52).abs() < 1e-9);
    }

    #[test]
    fn the_light_falls_by_the_inverse_square_and_ends_at_the_range() {
        assert_eq!(point_attenuation(2.0, None), 0.25);
        assert_eq!(point_attenuation(4.0, Some(4.0)), 0.0);
        assert_eq!(point_attenuation(5.0, Some(4.0)), 0.0);

        let windowed = point_attenuation(2.0, Some(4.0));
        assert!(windowed > 0.0 && windowed < 0.25);
    }

    #[test]
    fn the_cone_holds_full_inside_the_inner_angle_and_fades_to_the_outer() {
        let inner = 30f64.to_radians();
        let outer = 45f64.to_radians();
        let close = |a: f64, b: f64| (a - b).abs() < 1e-9;

        assert_eq!(cone_attenuation(1.0, inner, outer), 1.0);
        assert!(close(cone_attenuation(inner.cos(), inner, outer), 1.0));
        assert!(close(cone_attenuation(outer.cos(), inner, outer), 0.0));
        assert_eq!(cone_attenuation(0.0, inner, outer), 0.0);
        assert_eq!(cone_attenuation(-1.0, inner, outer), 0.0);

        let between = cone_attenuation(37.5f64.to_radians().cos(), inner, outer);
        assert!(between > 0.0 && between < 1.0, "{between}");

        // A cone whose angles sit a rounding apart keeps a hard edge.
        let edge = 45f64.to_radians();
        assert_eq!(
            cone_attenuation(44f64.to_radians().cos(), edge - 1e-12, edge),
            1.0
        );
        assert_eq!(
            cone_attenuation(46f64.to_radians().cos(), edge - 1e-12, edge),
            0.0
        );
    }

    /// A 9 by 5 by 9 grid: a floor across `y = 0` and a ceiling across
    /// `y = 4`, lit by `light` between them.
    fn roofed(light: RenderLight) -> RenderScene {
        let mut scene = RenderScene::default();
        let material_id = scene.retain_material(RenderMaterial::default()).unwrap();
        let object_id = U32Id::from_u32(0);
        scene
            .retain_object(
                object_id,
                solid_object(
                    "roofed",
                    TyVector3U32::new(9, 5, 9),
                    material_id,
                    |position| position.y == 0 || position.y == 4,
                ),
            )
            .unwrap();
        scene
            .retain_placement(RenderPlacement {
                object_id,
                transform: TyTransformF64::IDENTITY,
            })
            .unwrap();
        scene.retain_light(light).unwrap();
        scene
    }

    #[test]
    fn a_spot_lights_a_disc_that_fades_between_the_cones_and_its_shadow_ends_at_the_light() {
        let position = TyVector3F64::new(4.5, 3.0, 4.5);

        let point = roofed(RenderLight::Point {
            position,
            color: TyLinSrgbF64::new(1.0, 1.0, 1.0),
            strength: 1.0,
            range: None,
            shadow: RenderShadow::PerPixel,
        });
        let spot = roofed(RenderLight::Spot {
            position,
            rotation: TyQuaternionF64::from_look_direction(-TyVector3F64::Y, -TyVector3F64::Z)
                .unwrap(),
            color: TyLinSrgbF64::new(1.0, 1.0, 1.0),
            strength: 1.0,
            range: None,
            inner_cone: 20f64.to_radians(),
            outer_cone: 40f64.to_radians(),
            shadow: RenderShadow::PerPixel,
        });

        // The floor's top at `x` under a ray down from the gap. A shadow ray
        // running past the light would strike the ceiling above it.
        let floor = |scene: &RenderScene, x: f64| {
            let ray = RenderRay {
                origin: TyVector3F64::new(x, 3.5, 4.5),
                direction: -TyVector3F64::Y,
            };
            let hit = cast_ray(scene, &ray, f64::INFINITY).unwrap();
            assert_eq!(hit.distance, 2.5);
            shade_hit(scene, RenderOcclusion::None, &ray, &hit)
                .color
                .red
        };

        // The light sits two units up, so the inner cone reaches 0.73 out
        // and the outer 1.68.
        let ratio = |x: f64| floor(&spot, x) / floor(&point, x);

        assert!((ratio(4.5) - 1.0).abs() < 1e-9);
        assert!((ratio(5.0) - 1.0).abs() < 1e-9);
        let between = ratio(5.7);
        assert!(between > 0.0 && between < 1.0, "{between}");
        assert_eq!(ratio(6.5), 0.0);
    }

    /// A 4 by 2 by 4 grid: a floor across `y = 0` and a wall one high along
    /// `x = 3` on top of it.
    fn walled() -> RenderScene {
        let mut scene = RenderScene::default();
        let material_id = scene.retain_material(RenderMaterial::default()).unwrap();

        let mut object = RenderObject::new("w".to_owned(), TyVector3U32::new(4, 2, 4)).unwrap();
        for x in 0..4 {
            for z in 0..4 {
                for y in 0..(if x == 3 { 2 } else { 1 }) {
                    let voxel_id = object.voxel_id(TyVector3U32::new(x, y, z)).unwrap();
                    object
                        .set_voxel_material(voxel_id, Some(material_id))
                        .unwrap();
                }
            }
        }
        let object_id = U32Id::from_u32(0);
        scene.retain_object(object_id, object).unwrap();
        scene
            .retain_placement(RenderPlacement {
                object_id,
                transform: TyTransformF64::IDENTITY,
            })
            .unwrap();

        scene
    }

    #[test]
    fn a_wall_shadows_the_floor_behind_it_at_each_granularity() {
        let scene = walled();

        // The light comes from +x at a slope of 0.7, so a ray from the
        // floor at `x` clears the wall's top at `y = 2` when `x < 1.571`.
        let target = ShadowTarget::Direction(TyVector3F64::new(3.0, 2.1, 0.0).normalize());

        let hit_at = |x: f64| {
            cast_ray(
                &scene,
                &RenderRay {
                    origin: TyVector3F64::new(x, 5.0, 1.5),
                    direction: -TyVector3F64::Y,
                },
                f64::INFINITY,
            )
            .unwrap()
        };

        let near = hit_at(1.25);
        let far = hit_at(1.75);
        assert_eq!(near.along, [0.5, 0.25]);
        assert_eq!(far.along, [0.5, 0.75]);

        // Per pixel splits the face at the shadow's edge.
        assert_eq!(
            shadow_factor(&scene, &near, RenderShadow::PerPixel, &target),
            1.0
        );
        assert_eq!(
            shadow_factor(&scene, &far, RenderShadow::PerPixel, &target),
            0.0
        );

        // Per face reads the center, which the light clears.
        assert_eq!(
            shadow_factor(&scene, &far, RenderShadow::PerFace, &target),
            1.0
        );

        // Per corner blends the lit corners at x = 1 with the shadowed
        // corners at x = 2.
        assert!(
            (shadow_factor(&scene, &near, RenderShadow::PerCorner, &target) - 0.75).abs() < 1e-9
        );
        assert!(
            (shadow_factor(&scene, &far, RenderShadow::PerCorner, &target) - 0.25).abs() < 1e-9
        );

        assert_eq!(
            shadow_factor(&scene, &far, RenderShadow::None, &target),
            1.0
        );

        // The floor well in front of the wall is fully lit.
        let open = hit_at(0.5);
        assert_eq!(
            shadow_factor(&scene, &open, RenderShadow::PerCorner, &target),
            1.0
        );

        // A point light's ray ends at the light. The wall beyond a light at
        // x = 1.75 casts nothing, and the wall blocks a light at x = 5.
        let behind = ShadowTarget::Position(TyVector3F64::new(1.75, 1.5, 1.5));
        assert_eq!(
            shadow_factor(&scene, &far, RenderShadow::PerPixel, &behind),
            1.0
        );
        let beyond = ShadowTarget::Position(TyVector3F64::new(5.0, 1.5, 1.5));
        assert_eq!(
            shadow_factor(&scene, &far, RenderShadow::PerPixel, &beyond),
            0.0
        );
    }

    #[test]
    fn the_blend_follows_the_winding_on_either_side() {
        let face = SurfaceSpan {
            d: 1,
            sign: 1,
            s: 0,
            u0: 2,
            u1: 3,
            v0: 5,
            v1: 6,
        };
        let values = [1.0, 2.0, 3.0, 4.0];

        assert_eq!(bilinear(&face, values, [0.0, 0.0]), 1.0);
        assert_eq!(bilinear(&face, values, [1.0, 0.0]), 2.0);
        assert_eq!(bilinear(&face, values, [1.0, 1.0]), 3.0);
        assert_eq!(bilinear(&face, values, [0.0, 1.0]), 4.0);
        assert_eq!(bilinear(&face, values, [0.5, 0.5]), 2.5);

        let back = SurfaceSpan { sign: -1, ..face };
        assert_eq!(bilinear(&back, values, [1.0, 0.0]), 4.0);
        assert_eq!(bilinear(&back, values, [0.0, 1.0]), 2.0);
    }

    #[test]
    fn the_cube_matches_its_goldens() {
        check_goldens(
            "cube",
            cube_scene,
            RenderBloom::default(),
            [
                include_bytes!("goldens/cube-per-pixel.png"),
                include_bytes!("goldens/cube-per-face.png"),
                include_bytes!("goldens/cube-per-corner.png"),
                include_bytes!("goldens/cube-unoccluded.png"),
            ],
        );
    }

    #[test]
    fn the_l_shape_matches_its_goldens() {
        check_goldens(
            "l-shape",
            l_shape_scene,
            RenderBloom::default(),
            [
                include_bytes!("goldens/l-shape-per-pixel.png"),
                include_bytes!("goldens/l-shape-per-face.png"),
                include_bytes!("goldens/l-shape-per-corner.png"),
                include_bytes!("goldens/l-shape-unoccluded.png"),
            ],
        );
    }

    #[test]
    fn the_two_placements_match_their_goldens() {
        check_goldens(
            "two-placements",
            two_placements_scene,
            RenderBloom::default(),
            [
                include_bytes!("goldens/two-placements-per-pixel.png"),
                include_bytes!("goldens/two-placements-per-face.png"),
                include_bytes!("goldens/two-placements-per-corner.png"),
                include_bytes!("goldens/two-placements-unoccluded.png"),
            ],
        );
    }

    #[test]
    fn the_room_matches_its_goldens() {
        check_goldens(
            "room",
            room_scene,
            RenderBloom::default(),
            [
                include_bytes!("goldens/room-per-pixel.png"),
                include_bytes!("goldens/room-per-face.png"),
                include_bytes!("goldens/room-per-corner.png"),
                include_bytes!("goldens/room-unoccluded.png"),
            ],
        );
    }

    #[test]
    fn bloom_off_or_under_the_threshold_leaves_the_image_and_the_halo_spills_past_the_silhouette() {
        let scene = glow_scene(RenderShadow::None);
        let (view_id, _) = scene.iter_views().next().unwrap();
        let draw = |bloom| render(&scene, view_id, RenderOcclusion::Corner, bloom, 64, 64).unwrap();

        // The strip's emission has a luminance of 2.48.
        let off = draw(RenderBloom::default());
        let under = draw(RenderBloom {
            strength: 1.0,
            threshold: 3.0,
            ..RenderBloom::default()
        });
        assert_eq!(off, under);

        let glow = draw(RenderBloom {
            strength: 1.0,
            ..RenderBloom::default()
        });
        assert_ne!(off, glow);

        // Down the middle column, the first hit is the strip's top edge.
        let top = (0..64)
            .find(|y| off.pixel(32, *y).unwrap().alpha == 1.0)
            .unwrap();
        assert!(top > 4, "{top}");

        // The miss above it takes the halo, orange, as a translucent pixel.
        let above = glow.pixel(32, top - 1).unwrap();
        assert_eq!(off.pixel(32, top - 1).unwrap().alpha, 0.0);
        assert!(above.alpha > 0.0 && above.alpha < 1.0, "{above:?}");
        assert!(
            above.red > above.green && above.green > above.blue,
            "{above:?}"
        );

        assert!(glow.pixel(32, top - 2).unwrap().alpha < above.alpha);
        assert_eq!(glow.pixel(0, 0).unwrap().alpha, 0.0);

        let strip = glow.pixel(32, top).unwrap();
        assert_eq!(strip.alpha, 1.0);
        assert!(strip.red > off.pixel(32, top).unwrap().red);
    }

    #[test]
    fn the_glow_matches_its_goldens() {
        check_goldens(
            "glow",
            glow_scene,
            RenderBloom {
                strength: 1.0,
                ..RenderBloom::default()
            },
            [
                include_bytes!("goldens/glow-per-pixel.png"),
                include_bytes!("goldens/glow-per-face.png"),
                include_bytes!("goldens/glow-per-corner.png"),
                include_bytes!("goldens/glow-unoccluded.png"),
            ],
        );
    }

    #[test]
    fn the_spot_room_matches_its_goldens() {
        check_goldens(
            "spot-room",
            spot_room_scene,
            RenderBloom::default(),
            [
                include_bytes!("goldens/spot-room-per-pixel.png"),
                include_bytes!("goldens/spot-room-per-face.png"),
                include_bytes!("goldens/spot-room-per-corner.png"),
                include_bytes!("goldens/spot-room-unoccluded.png"),
            ],
        );
    }
}
