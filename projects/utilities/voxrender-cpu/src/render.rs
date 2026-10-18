use crate::{
    RenderGrid, RenderHit, RenderRayWalk, ShadowTarget, apply_bloom, material_pass,
    normal_reflectance, placement_bounds, transmitted_reflectance,
};
use branded_id::U32Id;
use std::{
    f64::consts::PI,
    ops::{Add, Mul},
};
use ty_math::{TyLinSrgbF32, TyLinSrgbF64, TyTransformF64, TyVector3F64};
use voxrender::{
    BRenderPlacement, BRenderView, Error, GRID_FRACTION_BITS, RenderBloom, RenderGridRay,
    RenderImage, RenderLight, RenderMaterial, RenderOcclusion, RenderPixel, RenderRay, RenderScene,
    RenderShadow, RenderViewRays, Result, SHADOW_INSET_BITS, grid_point, grid_vector,
    quantize_direction, quantize_point,
};
use voxsurface::{SurfaceSpan, corner_occlusion, inner_corner_occlusion};

/// The GGX alpha floor that keeps a mirror's lobe finite.
const MIN_ALPHA: f64 = 1e-3;

/// glTF's floor under the gap between a spot's cone cosines. It keeps a cone
/// whose angles sit a rounding apart finite.
const MIN_CONE_WIDTH: f64 = 1e-3;

/// One cell's side in fixed point.
const ONE: i64 = 1 << GRID_FRACTION_BITS;

/// [`SHADOW_INSET_BITS`] in fixed point.
const INSET: i64 = ONE >> SHADOW_INSET_BITS;

const BLACK: TyLinSrgbF64 = TyLinSrgbF64::new(0.0, 0.0, 0.0);

const WHITE: TyLinSrgbF64 = TyLinSrgbF64::new(1.0, 1.0, 1.0);

/// Renders the view `view_id` of `scene` into a `width` by `height` image
/// under every light of the scene. `occlusion` decides whether the corner
/// occlusion darkens the hemisphere light. Each pixel walks its ray front
/// to back, adding each hit's shade at its coverage and passing the rest
/// through by the material's pass. A pixel no ray hits carries no light and
/// full transmittance until `bloom`'s halo reaches it. Errors if the view
/// is not one of the scene's, a side is zero, a bloom value is out of
/// range, or a ray or light lies out of a grid's
/// [range](crate::GRID_RANGE_BITS).
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

    let grid_rays = scene
        .iter_placements()
        .map(|(placement_id, placement)| {
            rays.to_grid_rays(&placement.transform, placement_bounds(scene, placement))
                .ok_or(Error::GridRange { placement_id })
        })
        .collect::<Result<Vec<_>>>()?;

    let mut image = RenderImage::new(width, height);
    let mut emission = vec![TyLinSrgbF32::new(0.0, 0.0, 0.0); width as usize * height as usize];

    for y in 0..height {
        for x in 0..width {
            let walk = RenderRayWalk::new(scene, grid_rays.iter().map(|rays| rays.ray(x, y)));
            let shade = shade_ray(scene, occlusion, &rays.ray(x, y), walk)?;

            image.set_pixel(
                x,
                y,
                RenderPixel {
                    light: shade.light.into_format(),
                    transmittance: shade.transmittance.into_format(),
                },
            );

            emission[y as usize * width as usize + x as usize] = shade.emission.into_format();
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

/// What a pixel's ray shades to.
struct RayShade {
    /// The light that reached the pixel through every hit, in linear
    /// radiance.
    light: TyLinSrgbF64,

    /// The emissive term within `light`, which the bloom reads.
    emission: TyLinSrgbF64,

    /// The share of what lies behind the scene that passes through.
    transmittance: TyLinSrgbF64,
}

/// The shade of `ray` through `scene`, summed front to back over the hits
/// of its `walk`.
fn shade_ray(
    scene: &RenderScene,
    occlusion: RenderOcclusion,
    ray: &RenderRay,
    walk: RenderRayWalk,
) -> Result<RayShade> {
    let mut light = BLACK;
    let mut emission = BLACK;
    let mut throughput = WHITE;

    for hit in walk {
        let shade = shade_hit(scene, occlusion, ray, &hit)?;
        let covered = throughput * shade.coverage;

        light += covered * shade.color;
        emission += covered * shade.emission;
        throughput *= shade.pass;

        if throughput == BLACK {
            break;
        }
    }

    Ok(RayShade {
        light,
        emission,
        transmittance: throughput,
    })
}

/// What a hit shades to.
struct HitShade {
    /// The linear color, lit by every light plus the emission.
    color: TyLinSrgbF64,

    /// The emissive term within `color`, which the bloom reads.
    emission: TyLinSrgbF64,

    /// The share of the pixel the hit fills.
    coverage: f64,

    /// The share of the light behind the hit that continues, per channel.
    pass: TyLinSrgbF64,
}

/// The shade of `hit` seen along `ray` under every light of `scene`.
/// `occlusion` decides whether the corner occlusion darkens the hemisphere
/// term.
fn shade_hit(
    scene: &RenderScene,
    occlusion: RenderOcclusion,
    ray: &RenderRay,
    hit: &RenderHit,
) -> Result<HitShade> {
    let placement = scene
        .placement(hit.placement_id)
        .expect("a hit lands on one of the scene's placements");

    let object = scene
        .object(placement.object_id)
        .expect("a placement's object is one of the scene's");

    let material = hit_material(scene, hit);
    let transform = &placement.transform;

    // An exit's face looks away from the ray, so its shade looks back into
    // the material.
    let facing = if hit.exit { -1.0 } else { 1.0 };

    let normal = world_normal(transform, &hit.face) * facing;
    let view = -ray.direction;
    let point = ray.origin + ray.direction * hit.distance;

    let open = match occlusion {
        RenderOcclusion::None => 1.0,

        RenderOcclusion::Corner => {
            let grid = RenderGrid::new(scene, object);

            let corners = if hit.exit {
                inner_corner_occlusion(&grid, &hit.face)
            } else {
                corner_occlusion(&grid, &hit.face)
            };

            bilinear(&hit.face, corners, hit.along)
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

                if radiance == BLACK {
                    continue;
                }

                let lit = shadow_factor(scene, hit, shadow, &ShadowTarget::Direction(toward))?;

                color += radiance * light_color * lit * strength;
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

                if radiance == BLACK {
                    continue;
                }

                let reach = point_attenuation(distance, range);
                let lit = shadow_factor(scene, hit, shadow, &ShadowTarget::Position(position))?;

                color += radiance * light_color * lit * (strength * reach);
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

                if radiance == BLACK {
                    continue;
                }

                let axis = rotation * -TyVector3F64::Z;
                let reach = point_attenuation(distance, range)
                    * cone_attenuation(axis.dot(-toward), inner_cone, outer_cone);

                if reach == 0.0 {
                    continue;
                }

                let lit = shadow_factor(scene, hit, shadow, &ShadowTarget::Position(position))?;

                color += radiance * light_color * lit * (strength * reach);
            }

            RenderLight::Hemisphere {
                sky,
                ground,
                strength,
            } => {
                color += hemisphere_radiance(material, normal, view, sky, ground, strength, open);
            }
        }
    }

    Ok(HitShade {
        color,
        emission,
        coverage: material.base_color.alpha,
        pass: material_pass(material, normal.dot(view)),
    })
}

fn world_normal(transform: &TyTransformF64, face: &SurfaceSpan) -> TyVector3F64 {
    (transform.rotation * (face.normal().as_dvec3() / transform.scale)).normalize()
}

fn hit_material<'a>(scene: &'a RenderScene, hit: &RenderHit) -> &'a RenderMaterial {
    let placement = scene
        .placement(hit.placement_id)
        .expect("a hit lands on one of the scene's placements");

    let object = scene
        .object(placement.object_id)
        .expect("a placement's object is one of the scene's");

    scene
        .material(
            object
                .voxel_material(hit.voxel_id)
                .expect("a hit lands on a live voxel"),
        )
        .expect("a voxel samples one of the scene's materials")
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
        return BLACK;
    }

    let n_dot_v = normal.dot(view).max(1e-4);
    let half = (light + view).normalize();
    let n_dot_h = normal.dot(half).max(0.0);
    let v_dot_h = view.dot(half).max(0.0);

    let alpha = (material.roughness * material.roughness).max(MIN_ALPHA);
    let alpha2 = alpha * alpha;

    let f0 = normal_reflectance(material);
    let fresnel = f0 + (WHITE - f0) * (1.0 - v_dot_h).powi(5);

    let distribution = alpha2 / (PI * (n_dot_h * n_dot_h * (alpha2 - 1.0) + 1.0).powi(2));

    let visibility = 0.5
        / (n_dot_l * (n_dot_v * n_dot_v * (1.0 - alpha2) + alpha2).sqrt()
            + n_dot_v * (n_dot_l * n_dot_l * (1.0 - alpha2) + alpha2).sqrt());

    let diffuse = (WHITE - fresnel)
        * material.base_color.color
        * ((1.0 - material.metallic) * (1.0 - material.transmission) / PI);
    let specular = fresnel * (distribution * visibility);

    (diffuse + specular) * n_dot_l
}

/// The light `material` reflects from a hemisphere light, darkened toward
/// `occlusion` by the material's occlusion strength. The normal-incidence
/// reflectance gives a metal a reflection in its base color.
///
/// # Arguments
/// * `normal` - the surface's unit normal.
/// * `view` - the unit direction toward the viewer.
/// * `occlusion` - the corner occlusion, from `0` fully occluded to `1`
///   open.
fn hemisphere_radiance(
    material: &RenderMaterial,
    normal: TyVector3F64,
    view: TyVector3F64,
    sky: TyLinSrgbF64,
    ground: TyLinSrgbF64,
    strength: f64,
    occlusion: f64,
) -> TyLinSrgbF64 {
    let ambient = |direction: TyVector3F64| {
        let up = (direction.y + 1.0) / 2.0;

        (sky * up + ground * (1.0 - up)) * strength
    };

    let cosine = normal.dot(view);
    let mirror = normal * (2.0 * cosine) - view;
    let transmitted = material.transmission * (1.0 - material.metallic);

    let f0 = normal_reflectance(material);
    let diffuse =
        material.base_color.color * ((1.0 - material.metallic) * (1.0 - material.transmission));

    let open = 1.0 - material.occlusion_strength * (1.0 - occlusion);

    (ambient(normal) * (diffuse + f0 * (1.0 - transmitted))
        + ambient(mirror) * transmitted_reflectance(material, cosine) * transmitted)
        * open
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

/// How lit `hit` is by a light at `target`, per channel, from black in
/// shadow to white in the open, with rays cast at the `shadow` granularity.
fn shadow_factor(
    scene: &RenderScene,
    hit: &RenderHit,
    shadow: RenderShadow,
    target: &ShadowTarget,
) -> Result<TyLinSrgbF64> {
    let placement = scene
        .placement(hit.placement_id)
        .expect("a hit lands on one of the scene's placements");

    let sample = |across: [i64; 2]| {
        let origin = face_origin(&hit.face, across);

        let world = placement
            .transform
            .transform_point(TyVector3F64::from_array(
                origin.map(|coordinate| coordinate as f64 / ONE as f64),
            ));

        let direction = match *target {
            ShadowTarget::Direction(direction) => direction,
            ShadowTarget::Position(position) => (position - world).normalize(),
        };

        let walk = RenderRayWalk::new(
            scene,
            shadow_rays(scene, hit.placement_id, origin, world, target)?,
        );
        let mut throughput = entry_pass(scene, &walk);

        if throughput == BLACK {
            return Ok(BLACK);
        }

        for hit in walk {
            let transform = &scene
                .placement(hit.placement_id)
                .expect("a hit lands on one of the scene's placements")
                .transform;
            let cosine = world_normal(transform, &hit.face).dot(direction).abs();

            throughput *= material_pass(hit_material(scene, &hit), cosine);

            if throughput == BLACK {
                break;
            }
        }

        Ok(throughput)
    };

    match shadow {
        RenderShadow::None => Ok(WHITE),

        RenderShadow::PerPixel => {
            let across = |axis: usize, start: usize| {
                (i64::from(hit.point[axis]) - start as i64 * ONE).clamp(INSET, ONE - INSET)
            };

            sample([
                across(hit.face.u(), hit.face.u0),
                across(hit.face.v(), hit.face.v0),
            ])
        }

        RenderShadow::PerFace => sample([ONE / 2; 2]),

        RenderShadow::PerCorner => {
            let edge = |at_start: bool| if at_start { INSET } else { ONE - INSET };
            let mut values = [BLACK; 4];

            for (value, [uu, vv]) in values.iter_mut().zip(hit.face.corners()) {
                *value = sample([edge(uu == hit.face.u0), edge(vv == hit.face.v0)])?;
            }

            Ok(bilinear(&hit.face, values, hit.along))
        }
    }
}

/// One ray per placement toward `target` from `origin`, a fixed point in
/// `placement_id`'s grid. Other grids get the origin from `world`, the same
/// point in world space.
fn shadow_rays(
    scene: &RenderScene,
    placement_id: U32Id<BRenderPlacement>,
    origin: [i32; 3],
    world: TyVector3F64,
    target: &ShadowTarget,
) -> Result<Vec<RenderGridRay>> {
    scene
        .iter_placements()
        .map(|(other_id, other)| {
            let bounds = placement_bounds(scene, other);

            let quantize = |point| {
                quantize_point(grid_point(&other.transform, point), bounds).ok_or(
                    Error::GridRange {
                        placement_id: other_id,
                    },
                )
            };

            let start = if other_id == placement_id {
                origin
            } else {
                quantize(world)?
            };

            Ok(match *target {
                ShadowTarget::Direction(direction) => RenderGridRay {
                    origin: start,
                    direction: quantize_direction(grid_vector(&other.transform, direction)),
                    end: None,
                },

                ShadowTarget::Position(position) => {
                    RenderGridRay::toward(start, quantize(position)?)
                }
            })
        })
        .collect()
}

/// What the light paid to enter the materials `walk` starts inside. The light
/// pays each pass at normal incidence because the faces it entered by are
/// unknown.
fn entry_pass(scene: &RenderScene, walk: &RenderRayWalk) -> TyLinSrgbF64 {
    walk.starts_inside()
        .iter()
        .fold(WHITE, |pass, &material_id| {
            let material = scene
                .material(material_id)
                .expect("a voxel samples one of the scene's materials");

            pass * material_pass(material, 1.0)
        })
}

/// The fixed point on `face`'s plane `across` units from its min corner
/// along its `u` and `v`.
fn face_origin(face: &SurfaceSpan, across: [i64; 2]) -> [i32; 3] {
    let mut point = [0; 3];

    point[face.d] = (i64::from(face.s) + i64::from(face.sign > 0)) * ONE;
    point[face.u()] = face.u0 as i64 * ONE + across[0];
    point[face.v()] = face.v0 as i64 * ONE + across[1];

    point.map(|coordinate| i32::try_from(coordinate).expect("a grid's sides are within range"))
}

/// `values` blended bilinearly across `face` at `along`.
///
/// # Arguments
/// * `values` - one value per corner, in the order of the face's
///   [`corners`](SurfaceSpan::corners).
/// * `along` - the fraction across the face's `u` and `v`.
fn bilinear<T>(face: &SurfaceSpan, values: [T; 4], along: [f64; 2]) -> T
where
    T: Add<Output = T> + Default + Mul<f64, Output = T>,
{
    face.corners()
        .iter()
        .zip(values)
        .fold(T::default(), |sum, ([uu, vv], value)| {
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

            sum + value * weight_u * weight_v
        })
}

#[cfg(test)]
mod tests {
    use crate::{
        RenderHit, RenderRayWalk, ShadowTarget, cast_ray, material_pass, render,
        render::{
            BLACK, RayShade, WHITE, bilinear, check_bloom, cone_attenuation, direct_radiance,
            hemisphere_radiance, point_attenuation, shade_hit, shade_ray, shadow_factor,
        },
        test_utilities::{bar_scene, check_goldens},
        transmitted_reflectance,
    };
    use branded_id::U32Id;
    use ty_math::{
        TyLinSrgbF32, TyLinSrgbF64, TyLinSrgbaF64, TyPoseF64, TyQuaternionExt, TyQuaternionF64,
        TyTransformF64, TyVector3Ext, TyVector3F64, TyVector3U32,
    };
    use voxrender::{
        Error, RenderBloom, RenderLight, RenderMaterial, RenderObject, RenderOcclusion,
        RenderPixel, RenderPlacement, RenderProjection, RenderRay, RenderScene, RenderShadow,
        RenderView, fit_distance,
        fixtures::{
            cube_scene, glass, glass_scene, glow_scene, l_shape_scene, matte, room_scene,
            solid_object, spot_room_scene, two_placements_scene,
        },
    };
    use voxsurface::SurfaceSpan;

    /// The shade of `ray` through `scene` without occlusion.
    fn ray_shade(scene: &RenderScene, ray: &RenderRay) -> RayShade {
        let walk = RenderRayWalk::from_ray(scene, ray).unwrap();

        shade_ray(scene, RenderOcclusion::None, ray, walk).unwrap()
    }

    #[test]
    fn hits_block_what_lies_behind_and_misses_pass_it() {
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
        assert_eq!(center.transmittance, TyLinSrgbF32::new(0.0, 0.0, 0.0));
        assert!(center.light.red > 0.0);
        assert_eq!(image.pixel(0, 0), Some(RenderPixel::default()));
        assert_eq!(image.pixel(7, 7), Some(RenderPixel::default()));

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
    #[test]
    fn a_ray_or_light_out_of_a_grids_range_errors() {
        let draw = |scene: &RenderScene| {
            let (view_id, _) = scene.iter_views().next().unwrap();

            render(
                scene,
                view_id,
                RenderOcclusion::None,
                RenderBloom::default(),
                8,
                8,
            )
        };

        let mut tiny = cube_scene(RenderShadow::PerFace);
        let (placement_id, _) = tiny.iter_placements().next().unwrap();
        tiny.set_placement_transform(
            placement_id,
            TyTransformF64::new(
                TyVector3F64::ZERO,
                TyQuaternionF64::IDENTITY,
                TyVector3F64::splat(1e-10),
            ),
        )
        .unwrap();
        assert_eq!(draw(&tiny), Err(Error::GridRange { placement_id }));

        let mut far = cube_scene(RenderShadow::PerFace);
        far.retain_light(RenderLight::Point {
            position: TyVector3F64::splat(1e12),
            color: TyLinSrgbF64::new(1.0, 1.0, 1.0),
            strength: 1.0,
            range: None,
            shadow: RenderShadow::PerFace,
        })
        .unwrap();
        assert_eq!(draw(&far), Err(Error::GridRange { placement_id }));
    }

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
        let hit = cast_ray(scene, &ray, f64::INFINITY).unwrap().unwrap();
        let color = shade_hit(scene, RenderOcclusion::Corner, &ray, &hit)
            .unwrap()
            .color;
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

    fn close(a: TyLinSrgbF64, b: TyLinSrgbF64) -> bool {
        (a.red - b.red).abs() < 1e-9
            && (a.green - b.green).abs() < 1e-9
            && (a.blue - b.blue).abs() < 1e-9
    }

    #[test]
    fn a_clear_pane_reflects_at_both_walls_and_shows_the_wall_behind_it_through_one() {
        let white = TyLinSrgbaF64::new(1.0, 1.0, 1.0, 1.0);
        let red = TyLinSrgbaF64::new(1.0, 0.0, 0.0, 1.0);

        // The white sky lights a face by its diffuse color plus the 0.04
        // reflectance. Each wall of the pane reflects the 0.04 alone and
        // passes 0.96.
        let (pane, ray) = bar_scene(&[glass(white)]);
        let alone = ray_shade(&pane, &ray);
        assert!(
            close(alone.light, WHITE * (0.04 + 0.96 * 0.04)),
            "{:?}",
            alone.light
        );
        assert!(close(alone.transmittance, WHITE * (0.96 * 0.96)));
        assert_eq!(alone.emission, BLACK);

        // The pane has no exit into the wall behind it, which shades to
        // (1.04, 0.04, 0.04).
        let (walled, _) = bar_scene(&[glass(white), matte(red)]);
        let through = ray_shade(&walled, &ray);
        assert!(
            close(
                through.light,
                TyLinSrgbF64::new(0.04 + 0.96 * 1.04, 0.04 + 0.96 * 0.04, 0.04 + 0.96 * 0.04)
            ),
            "{:?}",
            through.light
        );
        assert_eq!(through.transmittance, BLACK);
    }

    #[test]
    fn an_exit_looks_back_into_its_material() {
        let white = TyLinSrgbaF64::new(1.0, 1.0, 1.0, 1.0);

        // A glass voxel under a matte one, seen from -X under a light from
        // -X.
        let mut scene = RenderScene::default();
        let glass_id = scene.retain_material(glass(white)).unwrap();
        let matte_id = scene.retain_material(matte(white)).unwrap();

        let mut object = RenderObject::new("stack".to_owned(), TyVector3U32::new(1, 2, 1)).unwrap();
        for (y, material_id) in [(0, glass_id), (1, matte_id)] {
            let voxel_id = object.voxel_id(TyVector3U32::new(0, y, 0)).unwrap();
            object
                .set_voxel_material(voxel_id, Some(material_id))
                .unwrap();
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
            .retain_light(RenderLight::Directional {
                rotation: TyQuaternionF64::from_look_direction(TyVector3F64::X, TyVector3F64::Y)
                    .unwrap(),
                color: WHITE,
                strength: 1.0,
                shadow: RenderShadow::PerPixel,
            })
            .unwrap();

        let ray = RenderRay {
            origin: TyVector3F64::new(-1.0, 0.5, 0.5),
            direction: TyVector3F64::X,
        };

        let hits: Vec<_> = RenderRayWalk::from_ray(&scene, &ray).unwrap().collect();
        let [entry, exit] = hits.as_slice() else {
            panic!("{hits:?}");
        };
        assert!(!entry.exit && exit.exit);

        let shade = |scene: &RenderScene, hit, occlusion| {
            shade_hit(scene, occlusion, &ray, hit).unwrap().color
        };

        // The exit faces the viewer, and the light reaches it through the
        // glass.
        let lit = shade(&scene, entry, RenderOcclusion::None);
        assert!(lit.red > 0.0);
        assert!(close(
            shade(&scene, exit, RenderOcclusion::None),
            lit * 0.96
        ));

        // Under the sky, the matte voxel closes the exit's upper corners in
        // the glass. The entry looks out at nothing.
        scene
            .retain_light(RenderLight::Hemisphere {
                sky: WHITE,
                ground: WHITE,
                strength: 1.0,
            })
            .unwrap();

        assert_eq!(
            shade(&scene, entry, RenderOcclusion::Corner),
            shade(&scene, entry, RenderOcclusion::None)
        );
        assert!(
            shade(&scene, exit, RenderOcclusion::Corner).red
                < shade(&scene, exit, RenderOcclusion::None).red
        );
    }

    #[test]
    fn a_pane_two_voxels_thick_shades_as_a_thin_one() {
        let white = TyLinSrgbaF64::new(1.0, 1.0, 1.0, 1.0);
        let red = TyLinSrgbaF64::new(1.0, 0.0, 0.0, 1.0);

        let (thin, ray) = bar_scene(&[glass(white), matte(red)]);
        let (thick, _) = bar_scene(&[glass(white), glass(white), matte(red)]);

        let thin = ray_shade(&thin, &ray);
        let thick = ray_shade(&thick, &ray);
        assert_eq!(thick.light, thin.light);
        assert_eq!(thick.transmittance, thin.transmittance);
    }

    #[test]
    fn a_pane_seen_at_an_angle_passes_less_and_reflects_more() {
        let clear = glass(TyLinSrgbaF64::new(1.0, 1.0, 1.0, 1.0));
        let (pane, head_on) = bar_scene(&[clear]);

        // The ray enters by the -X face and leaves by the +Z face, each at a
        // cosine of 0.707. The white sky reflects alike in every direction.
        let oblique = RenderRay {
            origin: TyVector3F64::new(-0.5, 0.5, 0.0),
            direction: TyVector3F64::new(1.0, 0.0, 1.0).normalize(),
        };
        let cosine = 0.5f64.sqrt();
        let pass = material_pass(&clear, cosine);
        let fresnel = transmitted_reflectance(&clear, cosine);

        let shade = ray_shade(&pane, &oblique);
        assert!(close(shade.transmittance, pass * pass));
        assert!(close(shade.light, fresnel + pass * fresnel));
        assert!(shade.transmittance.red < ray_shade(&pane, &head_on).transmittance.red);
    }

    #[test]
    fn a_half_alpha_voxel_averages_its_shade_and_the_wall() {
        let blue = matte(TyLinSrgbaF64::new(0.0, 0.0, 1.0, 0.5));
        let red = matte(TyLinSrgbaF64::new(1.0, 0.0, 0.0, 1.0));

        // Half of (0.04, 0.04, 1.04) plus half of (1.04, 0.04, 0.04).
        let (walled, ray) = bar_scene(&[blue, red]);
        let shade = ray_shade(&walled, &ray);
        assert!(
            close(shade.light, TyLinSrgbF64::new(0.54, 0.04, 0.54)),
            "{:?}",
            shade.light
        );
        assert_eq!(shade.transmittance, BLACK);

        // Over nothing, the far wall covers half of the half that passes, so
        // the voxel covers three quarters in all.
        let (open, _) = bar_scene(&[blue]);
        let shade = ray_shade(&open, &ray);
        assert!(close(shade.light, TyLinSrgbF64::new(0.03, 0.03, 0.78)));
        assert_eq!(shade.transmittance, TyLinSrgbF64::new(0.25, 0.25, 0.25));
    }

    #[test]
    fn a_transmissive_emitter_adds_its_emission_at_each_wall() {
        let emitter = RenderMaterial {
            emissive_color: TyLinSrgbF64::new(1.0, 0.5, 0.0),
            emissive_strength: 2.0,
            ..glass(TyLinSrgbaF64::new(1.0, 1.0, 1.0, 0.5))
        };

        let (scene, ray) = bar_scene(&[emitter]);
        let shade = ray_shade(&scene, &ray);

        // Each wall passes 0.98: the uncovered half whole and the covered
        // half 0.96. The near wall adds half the emission plus half the
        // 0.04 reflectance, and the far wall adds as much through 0.98.
        let walls = 1.0 + 0.98;

        assert!(close(
            shade.emission,
            TyLinSrgbF64::new(1.0, 0.5, 0.0) * walls
        ));
        assert!(close(
            shade.light,
            TyLinSrgbF64::new(1.02, 0.52, 0.02) * walls
        ));
        assert!(close(shade.transmittance, WHITE * (0.98 * 0.98)));
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
        let hit = cast_ray(&scene, &ray, f64::INFINITY).unwrap().unwrap();

        let shade = shade_hit(&scene, RenderOcclusion::None, &ray, &hit).unwrap();
        assert_eq!(shade.color, TyLinSrgbF64::new(2.0, 1.0, 0.0));
        assert_eq!(shade.emission, shade.color);
    }

    #[test]
    fn light_behind_the_surface_is_black_and_a_metal_reflects_its_own_color() {
        let rough = RenderMaterial {
            base_color: TyLinSrgbaF64::new(0.5, 0.5, 0.5, 1.0),
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
            base_color: TyLinSrgbaF64::new(1.0, 0.0, 0.0, 1.0),
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
    fn glass_leaves_the_face_behind_it_open_to_the_sky() {
        let white = TyLinSrgbaF64::new(1.0, 1.0, 1.0, 1.0);

        // The face of a white voxel behind `front`, shaded with and without
        // the corner occlusion.
        let behind = |front: RenderMaterial| {
            let (scene, ray) = bar_scene(&[front, matte(white)]);
            let hit = RenderRayWalk::from_ray(&scene, &ray)
                .unwrap()
                .nth(1)
                .unwrap();
            assert_eq!(hit.distance, 2.0);

            (
                shade_hit(&scene, RenderOcclusion::Corner, &ray, &hit)
                    .unwrap()
                    .color,
                shade_hit(&scene, RenderOcclusion::None, &ray, &hit)
                    .unwrap()
                    .color,
            )
        };

        let (corner, open) = behind(glass(white));
        assert!(open.red > 0.0);
        assert_eq!(corner, open);

        let (corner, open) = behind(matte(TyLinSrgbaF64::new(1.0, 0.0, 0.0, 1.0)));
        assert!(open.red > 0.0);
        assert_eq!(corner, BLACK);
    }

    #[test]
    fn a_shadow_ray_pays_the_glass_it_starts_inside_and_nothing_in_the_open() {
        let (scene, ray) = bar_scene(&[
            glass(TyLinSrgbaF64::new(1.0, 0.0, 0.0, 1.0)),
            matte(TyLinSrgbaF64::new(1.0, 1.0, 1.0, 1.0)),
        ]);
        let away = ShadowTarget::Direction(-TyVector3F64::X);
        let mut walk = RenderRayWalk::from_ray(&scene, &ray).unwrap();

        // The pane's face stands in the open.
        let pane = walk.next().unwrap();
        assert_eq!(
            shadow_factor(&scene, &pane, RenderShadow::PerPixel, &away).unwrap(),
            WHITE
        );

        // The wall's face behind it starts inside the red glass.
        let wall = walk.next().unwrap();
        let shadow = shadow_factor(&scene, &wall, RenderShadow::PerPixel, &away).unwrap();
        assert!(
            close(shadow, TyLinSrgbF64::new(0.96, 0.0, 0.0)),
            "{shadow:?}"
        );
    }

    #[test]
    fn transmission_takes_its_share_from_the_diffuse_term() {
        let white = TyLinSrgbaF64::new(1.0, 1.0, 1.0, 1.0);
        let normal = TyVector3F64::Y;
        let radiance = |transmission| {
            direct_radiance(
                &RenderMaterial {
                    transmission,
                    ..matte(white)
                },
                normal,
                normal,
                normal,
            )
        };

        let opaque = radiance(0.0);
        let clear = radiance(1.0);
        assert!(clear.red > 0.0 && clear.red < opaque.red);
        assert!(close(radiance(0.5), (opaque + clear) * 0.5));

        // The hemisphere light leaves the reflectance alone.
        let ambient = hemisphere_radiance(
            &RenderMaterial {
                transmission: 1.0,
                ..matte(white)
            },
            normal,
            normal,
            WHITE,
            WHITE,
            1.0,
            1.0,
        );
        assert!(close(ambient, TyLinSrgbF64::new(0.04, 0.04, 0.04)));
    }

    #[test]
    fn the_sky_lights_the_top_the_ground_the_bottom_and_occlusion_darkens() {
        let white = RenderMaterial {
            metallic: 0.0,
            ..RenderMaterial::default()
        };
        let sky = TyLinSrgbF64::new(1.0, 1.0, 1.0);
        let ground = TyLinSrgbF64::new(0.0, 0.0, 0.0);

        let ambient = |material: &RenderMaterial, normal: TyVector3F64, occlusion| {
            hemisphere_radiance(material, normal, normal, sky, ground, 1.0, occlusion)
        };

        let top = ambient(&white, TyVector3F64::Y, 1.0);
        let side = ambient(&white, TyVector3F64::X, 1.0);
        let bottom = ambient(&white, -TyVector3F64::Y, 1.0);
        assert!((top.red - 1.04).abs() < 1e-9);
        assert!((side.red - 0.52).abs() < 1e-9);
        assert_eq!(bottom.red, 0.0);

        // A material without transmission mixes by its normal from every
        // view.
        let oblique = TyVector3F64::new(1.0, 1.0, 0.0).normalize();
        assert_eq!(
            hemisphere_radiance(&white, TyVector3F64::X, oblique, sky, ground, 1.0, 1.0),
            side
        );

        let closed = ambient(&white, TyVector3F64::Y, 0.0);
        assert_eq!(closed.red, 0.0);

        let half_strength = RenderMaterial {
            occlusion_strength: 0.5,
            ..white
        };
        let softened = ambient(&half_strength, TyVector3F64::Y, 0.0);
        assert!((softened.red - 0.52).abs() < 1e-9);
    }

    #[test]
    fn glass_reflects_the_hemisphere_along_the_mirror_direction() {
        let clear = glass(TyLinSrgbaF64::new(1.0, 1.0, 1.0, 1.0));
        let normal = TyVector3F64::X;
        let ambient =
            |view: TyVector3F64| hemisphere_radiance(&clear, normal, view, WHITE, BLACK, 1.0, 1.0);

        // Head on, the mirror direction is the normal, which sees half sky.
        assert!(close(ambient(normal), WHITE * 0.02));

        // Seen from below, the mirror direction looks up into the sky. Seen
        // from above, it looks down at the ground.
        let below = TyVector3F64::new(1.0, -1.0, 0.0).normalize();
        let above = TyVector3F64::new(1.0, 1.0, 0.0).normalize();
        let fresnel = transmitted_reflectance(&clear, normal.dot(below));
        let up = (1.0 + 0.5f64.sqrt()) / 2.0;

        assert!(fresnel.red > 0.04);
        assert!(close(ambient(below), fresnel * up));
        assert!(close(ambient(above), fresnel * (1.0 - up)));
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
            let hit = cast_ray(scene, &ray, f64::INFINITY).unwrap().unwrap();
            assert_eq!(hit.distance, 2.5);
            shade_hit(scene, RenderOcclusion::None, &ray, &hit)
                .unwrap()
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

    /// A 4 by 2 by 4 grid: a floor across `y = 0` and a wall of `wall` one
    /// high along `x = 3` on top of it.
    fn walled(wall: RenderMaterial) -> RenderScene {
        let mut scene = RenderScene::default();
        let floor_id = scene.retain_material(RenderMaterial::default()).unwrap();
        let wall_id = scene.retain_material(wall).unwrap();

        let mut object = RenderObject::new("w".to_owned(), TyVector3U32::new(4, 2, 4)).unwrap();
        for x in 0..4 {
            for z in 0..4 {
                for y in 0..(if x == 3 { 2 } else { 1 }) {
                    let voxel_id = object.voxel_id(TyVector3U32::new(x, y, z)).unwrap();
                    let material_id = if y == 0 { floor_id } else { wall_id };
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

    /// The light over the walled fixture: from +x at a slope of 0.7, so a
    /// ray from the floor at `x` clears the wall's top at `y = 2` when
    /// `x < 1.571`.
    fn over_the_wall() -> ShadowTarget {
        ShadowTarget::Direction(TyVector3F64::new(3.0, 2.1, 0.0).normalize())
    }

    /// The walled fixture's floor hit under a ray down at `x`.
    fn floor_at(scene: &RenderScene, x: f64) -> RenderHit {
        cast_ray(
            scene,
            &RenderRay {
                origin: TyVector3F64::new(x, 5.0, 1.5),
                direction: -TyVector3F64::Y,
            },
            f64::INFINITY,
        )
        .unwrap()
        .unwrap()
    }

    #[test]
    fn a_wall_shadows_the_floor_behind_it_at_each_granularity() {
        let scene = walled(RenderMaterial::default());
        let target = over_the_wall();

        let near = floor_at(&scene, 1.25);
        let far = floor_at(&scene, 1.75);
        assert_eq!(near.along, [0.5, 0.25]);
        assert_eq!(far.along, [0.5, 0.75]);

        // Per pixel splits the face at the shadow's edge.
        assert_eq!(
            shadow_factor(&scene, &near, RenderShadow::PerPixel, &target).unwrap(),
            WHITE
        );
        assert_eq!(
            shadow_factor(&scene, &far, RenderShadow::PerPixel, &target).unwrap(),
            BLACK
        );

        // Per face reads the center, which the light clears.
        assert_eq!(
            shadow_factor(&scene, &far, RenderShadow::PerFace, &target).unwrap(),
            WHITE
        );

        // Per corner blends the lit corners at x = 1 with the shadowed
        // corners at x = 2.
        assert!(close(
            shadow_factor(&scene, &near, RenderShadow::PerCorner, &target).unwrap(),
            WHITE * 0.75
        ));
        assert!(close(
            shadow_factor(&scene, &far, RenderShadow::PerCorner, &target).unwrap(),
            WHITE * 0.25
        ));

        assert_eq!(
            shadow_factor(&scene, &far, RenderShadow::None, &target).unwrap(),
            WHITE
        );

        // The floor well in front of the wall is fully lit.
        let open = floor_at(&scene, 0.5);
        assert_eq!(
            shadow_factor(&scene, &open, RenderShadow::PerCorner, &target).unwrap(),
            WHITE
        );

        // A point light's ray ends at the light. The wall beyond a light at
        // x = 1.75 casts nothing, and the wall blocks a light at x = 5.
        let behind = ShadowTarget::Position(TyVector3F64::new(1.75, 1.5, 1.5));
        assert_eq!(
            shadow_factor(&scene, &far, RenderShadow::PerPixel, &behind).unwrap(),
            WHITE
        );
        let beyond = ShadowTarget::Position(TyVector3F64::new(5.0, 1.5, 1.5));
        assert_eq!(
            shadow_factor(&scene, &far, RenderShadow::PerPixel, &beyond).unwrap(),
            BLACK
        );
    }

    #[test]
    fn a_red_glass_wall_throws_a_red_shadow_that_blends_per_channel() {
        let wall = glass(TyLinSrgbaF64::new(1.0, 0.0, 0.0, 1.0));
        let scene = walled(wall);
        let target = over_the_wall();

        // The ray enters the wall's side and leaves by its top. Each face
        // passes the red at the ray's cosine to it.
        let ShadowTarget::Direction(direction) = target else {
            unreachable!();
        };
        let red = material_pass(&wall, direction.x) * material_pass(&wall, direction.y);
        assert!(red.red < 0.96 * 0.96, "{red:?}");

        let near = floor_at(&scene, 1.25);
        let far = floor_at(&scene, 1.75);

        assert!(close(
            shadow_factor(&scene, &far, RenderShadow::PerPixel, &target).unwrap(),
            red
        ));
        assert_eq!(
            shadow_factor(&scene, &far, RenderShadow::PerFace, &target).unwrap(),
            WHITE
        );

        // Three lit corners and one red: the green and blue fall to the
        // lit share, the red almost holds.
        assert!(close(
            shadow_factor(&scene, &near, RenderShadow::PerCorner, &target).unwrap(),
            WHITE * 0.75 + red * 0.25
        ));
        assert!(close(
            shadow_factor(&scene, &far, RenderShadow::PerCorner, &target).unwrap(),
            WHITE * 0.25 + red * 0.75
        ));
    }

    /// A 1 by `2 + thickness` by 1 grid: a matte white floor at `y = 0` and
    /// a pane of `pane` `thickness` voxels thick from `y = 2`, under a white
    /// directional light straight down. Returns the scene and the ray down
    /// onto the floor from the gap.
    fn paned(pane: RenderMaterial, thickness: u32) -> (RenderScene, RenderRay) {
        let mut scene = RenderScene::default();
        let floor_id = scene
            .retain_material(matte(TyLinSrgbaF64::new(1.0, 1.0, 1.0, 1.0)))
            .unwrap();
        let pane_id = scene.retain_material(pane).unwrap();

        let mut object =
            RenderObject::new("paned".to_owned(), TyVector3U32::new(1, 2 + thickness, 1)).unwrap();
        object
            .set_voxel_material(object.voxel_id(TyVector3U32::ZERO).unwrap(), Some(floor_id))
            .unwrap();
        for y in 2..2 + thickness {
            let voxel_id = object.voxel_id(TyVector3U32::new(0, y, 0)).unwrap();
            object.set_voxel_material(voxel_id, Some(pane_id)).unwrap();
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
            .retain_light(RenderLight::Directional {
                rotation: TyQuaternionF64::from_look_direction(-TyVector3F64::Y, -TyVector3F64::Z)
                    .unwrap(),
                color: WHITE,
                strength: 1.0,
                shadow: RenderShadow::PerPixel,
            })
            .unwrap();

        let ray = RenderRay {
            origin: TyVector3F64::new(0.5, 1.5, 0.5),
            direction: -TyVector3F64::Y,
        };

        (scene, ray)
    }

    #[test]
    fn a_red_pane_shadows_by_both_walls_and_two_voxels_thick_throw_the_same() {
        let red = glass(TyLinSrgbaF64::new(1.0, 0.0, 0.0, 1.0));
        let up = ShadowTarget::Direction(TyVector3F64::Y);

        let (thin, ray) = paned(red, 1);
        let hit = cast_ray(&thin, &ray, f64::INFINITY).unwrap().unwrap();
        assert_eq!(hit.distance, 0.5);
        let shadow = shadow_factor(&thin, &hit, RenderShadow::PerPixel, &up).unwrap();
        assert!(
            close(shadow, TyLinSrgbF64::new(0.96 * 0.96, 0.0, 0.0)),
            "{shadow:?}"
        );

        // Every sample of the face lies under the pane.
        assert_eq!(
            shadow_factor(&thin, &hit, RenderShadow::PerFace, &up).unwrap(),
            shadow
        );
        assert!(close(
            shadow_factor(&thin, &hit, RenderShadow::PerCorner, &up).unwrap(),
            shadow
        ));

        let (thick, ray) = paned(red, 2);
        let hit = cast_ray(&thick, &ray, f64::INFINITY).unwrap().unwrap();
        assert_eq!(
            shadow_factor(&thick, &hit, RenderShadow::PerPixel, &up).unwrap(),
            shadow
        );
    }

    #[test]
    fn a_voxel_that_covers_nothing_casts_no_shadow_and_an_opaque_one_casts_black() {
        let up = ShadowTarget::Direction(TyVector3F64::Y);

        let (scene, ray) = paned(matte(TyLinSrgbaF64::new(1.0, 0.0, 0.0, 0.0)), 1);
        let hit = cast_ray(&scene, &ray, f64::INFINITY).unwrap().unwrap();
        assert_eq!(
            shadow_factor(&scene, &hit, RenderShadow::PerPixel, &up).unwrap(),
            WHITE
        );

        let (scene, ray) = paned(matte(TyLinSrgbaF64::new(1.0, 0.0, 0.0, 1.0)), 1);
        let hit = cast_ray(&scene, &ray, f64::INFINITY).unwrap().unwrap();
        assert_eq!(
            shadow_factor(&scene, &hit, RenderShadow::PerPixel, &up).unwrap(),
            BLACK
        );
    }

    #[test]
    fn a_light_through_a_red_pane_lights_the_floor_red() {
        let red = glass(TyLinSrgbaF64::new(1.0, 0.0, 0.0, 1.0));
        let (open, ray) = paned(red, 0);
        let (shaded, _) = paned(red, 1);

        let floor = |scene: &RenderScene| {
            let hit = cast_ray(scene, &ray, f64::INFINITY).unwrap().unwrap();
            shade_hit(scene, RenderOcclusion::None, &ray, &hit)
                .unwrap()
                .color
        };

        let lit = floor(&open);
        assert!(lit.red > 0.0 && lit.red == lit.green);
        let tinted = floor(&shaded);
        assert!(
            close(tinted, lit * TyLinSrgbF64::new(0.96 * 0.96, 0.0, 0.0)),
            "{tinted:?}"
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

        // The strip's emission peaks at 4 in its red channel.
        let off = draw(RenderBloom::default());
        let under = draw(RenderBloom {
            strength: 1.0,
            threshold: 4.0,
            ..RenderBloom::default()
        });
        assert_eq!(off, under);

        // Half strength keeps the halo above the strip partly transparent.
        let glow = draw(RenderBloom {
            strength: 0.5,
            ..RenderBloom::default()
        });
        assert_ne!(off, glow);

        // Down the middle column, the first hit is the strip's top edge.
        let opaque = TyLinSrgbF32::new(0.0, 0.0, 0.0);
        let top = (0..64)
            .find(|y| off.pixel(32, *y).unwrap().transmittance == opaque)
            .unwrap();
        assert!(top > 4, "{top}");

        // The miss above it takes the halo, orange, and passes the rest.
        let above = glow.pixel(32, top - 1).unwrap();
        assert_eq!(off.pixel(32, top - 1), Some(RenderPixel::default()));
        assert!(
            above.transmittance.red > 0.0 && above.transmittance.red < 1.0,
            "{above:?}"
        );
        assert!(
            above.light.red > above.light.green && above.light.green > above.light.blue,
            "{above:?}"
        );

        assert!(glow.pixel(32, top - 2).unwrap().transmittance.red > above.transmittance.red);
        assert_eq!(glow.pixel(0, 0), Some(RenderPixel::default()));

        let strip = glow.pixel(32, top).unwrap();
        assert_eq!(strip.transmittance, opaque);
        assert!(strip.light.red > off.pixel(32, top).unwrap().light.red);
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
    fn the_glass_matches_its_goldens() {
        check_goldens(
            "glass",
            glass_scene,
            RenderBloom::default(),
            [
                include_bytes!("goldens/glass-per-pixel.png"),
                include_bytes!("goldens/glass-per-face.png"),
                include_bytes!("goldens/glass-per-corner.png"),
                include_bytes!("goldens/glass-unoccluded.png"),
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
