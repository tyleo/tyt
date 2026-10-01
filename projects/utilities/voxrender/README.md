# voxrender

The render contract for voxel scenes and its CPU reference renderer. The
contract types describe an image of a scene. The CPU renderer, behind the
default `cpu` feature, follows them literally and makes the images every
other renderer is tested against. The planned `voxrender-wgpu` draws the
same scene in realtime.

## Entities

A `U32Id` over one brand addresses each kind of entity:

1. Objects: `BVoxObject`, the id of the voxcore object each mirrors
2. Voxels: `BVoxVoxel`, voxcore's id for the cell's position
3. Materials: `BRenderMaterial`
4. Placements: `BRenderPlacement`
5. Lights: `BRenderLight`
6. Views: `BRenderView`

## The scene

A `RenderScene` is a document flattened for drawing. Objects are dense
grids whose cells each hold one material of the scene's table or nothing.
Placements put an object into world meters under a `TyTransformF64`, one
per root-to-object path of the hierarchy. Lights and views sit in world
space. Every mutation checks the cross-references it could break, so a
placement never points at a released object and a voxel never samples a
released material. `from_vox_main`
flattens a voxcore document: one material per live voxel by the effective
palette, deduplicated into the table, and one placement per path with the
grid origin and the voxel size folded in. `subject_bounds` gives the world
bounds of the live voxels under a set of placements, which the `fit` rule
frames.

```rust
let scene = RenderScene::from_vox_main(&main, &[object_id], 0.1)?;

let placement_ids: Vec<_> = scene.iter_placements().map(|(id, _)| id).collect();
let bounds = scene.subject_bounds(&placement_ids)?;
```

## Materials

A `RenderMaterial` carries the shaded properties in linear light: the base
color, metalness, roughness, the emissive color and its strength, and the
occlusion strength. Every voxel is opaque. The default is glTF's: opaque
white, fully metallic and rough, with no emission.

```rust
let brass = RenderMaterial {
    base_color: TyLinSrgbF64::new(0.8, 0.6, 0.2),
    roughness: 0.4,
    ..RenderMaterial::default()
};
```

## Lights

A `RenderLight` is one of four kinds, each carrying the part of a pose it
reads. A directional light is a rotation and shines down its -Z. A point
light is a position in meters with an inverse-square falloff and an
optional range. A spot light is a position and a rotation: it shines down
its -Z with the point light's falloff, fading across a cone between an
inner and an outer half-angle in radians. A hemisphere light is a sky color
above and a ground color below, mixed by a normal's world +Y. The three
shadowing kinds carry a `RenderShadow`: none, one ray per pixel, one per
face, or one per face corner.

```rust
let sun = RenderLight::Directional {
    rotation: TyQuaternionF64::IDENTITY,
    color: TyLinSrgbF64::new(1.0, 1.0, 1.0),
    strength: 3.0,
    shadow: RenderShadow::PerCorner,
};
```

## Views

A `RenderView` is a `TyPoseF64` looking down its -Z with +Y up, and a
`RenderProjection`: perspective with a vertical field of view in radians,
or orthographic with the world units across the shorter image axis.

```rust
let view = RenderView {
    pose: TyPoseF64::new(TyVector3F64::new(0.0, 0.0, 10.0), TyQuaternionF64::IDENTITY),
    projection: RenderProjection::Perspective { fov: 35f64.to_radians() },
};
```

## Rays

The `cpu` feature adds the rays. `RenderViewRays` gives the world ray
through each pixel center of a view over an image. `cast_ray` marches a
ray through every placement's grid to the nearest voxel and returns a
`RenderHit` with:

1. The placement
2. The cell
3. The face the ray entered through, as a unit `SurfaceSpan`
4. Where on the face the ray landed
5. The distance

A shadow ray is the same cast toward a light with a distance cap.

```rust
let rays = RenderViewRays::new(&view, 1024, 1024);
let ray = rays.ray(512, 512);

if let Some(hit) = cast_ray(&scene, &ray, f64::INFINITY) {
    let object_id = scene.placement(hit.placement_id)?.object_id;
    let occlusion = corner_occlusion(scene.object(object_id)?, &hit.face);
}
```

## Rendering

`render` draws a view of the scene into a linear image under every light of
the scene. Each hit shades with glTF's metallic-roughness model: Lambert
diffuse, GGX specular with Smith visibility and Schlick Fresnel, and the
emissive term. The light kinds reach a hit differently:

1. A directional light shines down its -Z
2. A point light falls off by the inverse square, with glTF's smooth
   cutoff at its range
3. A spot light falls off as a point light does, times glTF's cone falloff
   between its inner and outer angles
4. A hemisphere light mixes sky and ground by the normal's +Y

Under `RenderOcclusion::Corner`, the corner occlusion darkens the
hemisphere light. A shadow is one grid ray toward the light. `RenderShadow`
casts it per pixel, per face, or per corner, and blends the corner results
across the face.

```rust
let image = render(&scene, view_id, RenderOcclusion::Corner, 1024, 1024)?;
```

## Images

A `RenderImage` is a linear-light RGBA `f32` buffer, rows top to bottom.
`RenderOcclusion` picks whether a render shades with the corner occlusion
`voxsurface` computes.

`RenderOutput` holds the 8-bit sRGB image a PNG stores.
`RenderOutput::from_image` runs each hit through the Khronos PBR Neutral
curve in `tonemap` and then the sRGB transfer. Each miss becomes
transparent or the background color.

```rust
let output = RenderOutput::from_image(&image, None);

let bytes = output.to_bytes();
```
