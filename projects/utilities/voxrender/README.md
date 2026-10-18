# voxrender

The render contract for voxel scenes. The contract types describe an image of
a scene. Every renderer draws them through the `Render` trait.
`voxrender-cpu` follows them literally and makes the images every other
renderer is tested against. The planned `voxrender-wgpu` draws the same scene
in realtime. The `fixtures` feature adds the scenes the tests and benchmarks
share.

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
grid origin and the scene scale folded in. `subject_bounds` gives the world
bounds of the live voxels under a set of placements, which the `fit` rule
frames.

```rust
let scene = RenderScene::from_vox_main(&main, &[object_id], 0.1)?;

let placement_ids: Vec<_> = scene.iter_placements().map(|(id, _)| id).collect();
let bounds = scene.subject_bounds(&placement_ids)?;
```

## Materials

A `RenderMaterial` carries the shaded properties in linear light: the base
color with its coverage alpha, metalness, roughness, transmission, the index
of refraction, the emissive color and its strength, and the occlusion
strength. The default is glTF's: opaque white, fully metallic and rough, with
no transmission or emission. The index of refraction sets the dielectric
reflectance.

```rust
let brass = RenderMaterial {
    base_color: TyLinSrgbaF64::new(0.8, 0.6, 0.2, 1.0),
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

`RenderViewRays` gives the world ray through each pixel center of a view
over an image. A `RenderGridRay` is a ray quantized into one placement's
grid: a fixed-point origin, an integer direction, and an optional last cell.
`to_grid_rays` lowers a view's rays into a placement's grid as integer steps,
so every renderer derives the same ray for a pixel.
`RenderGridRay::from_ray` quantizes one world ray. `RenderGridRay::toward`
aims a ray at a point and ends it there.

```rust
let rays = RenderViewRays::new(&view, 1024, 1024);
let grid_rays = rays.to_grid_rays(&placement.transform, object.bounds())?;
let ray = grid_rays.ray(512, 512);
```

## Rendering

A renderer implements `Render`, which draws a view of the scene into a
`RenderImage`. A consumer takes the renderer as a dependency, so its caller
picks the render tech. `voxrender-cpu` is the reference.

```rust
let image = renderer.render(
    &scene,
    view_id,
    RenderOcclusion::Corner,
    RenderBloom::default(),
    1024,
    1024,
)?;
```

## Images

A `RenderImage` holds one `RenderPixel` per pixel, rows top to bottom: the
light that reached it in linear radiance and its transmittance, the share
of what lies behind the scene that passes through per channel. A miss has
no light and full transmittance. An opaque hit has its shade and none.
`RenderOcclusion` picks whether a render shades with the corner occlusion
`voxsurface` computes.

`RenderOutput` holds the 8-bit sRGB image a PNG stores.
`RenderOutput::from_image` takes the larger of one minus the peak
transmittance and the peak of the light, clamped to one, as each pixel's
coverage. It runs the light over that coverage through the Khronos PBR
Neutral curve in `tonemap` and scales it back by the coverage. Under a
background color it adds the background scaled by the transmittance and
writes the pixel in sRGB at full alpha. Without a background color it solves
the pixel to be exact over white for a viewer that blends sRGB values.

```rust
let output = RenderOutput::from_image(&image, None);

let bytes = output.to_bytes();
```
