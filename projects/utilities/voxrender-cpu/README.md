# voxrender-cpu

The CPU reference renderer for `voxrender`'s render contract. It follows the
contract literally and makes the images every other renderer is tested
against. A consumer reaches it through `voxrender`'s `Render` trait.

## The walk

`RenderRayWalk` walks one grid ray per placement with an integer DDA and
yields a `RenderHit` at each surface it crosses, nearest first.
`RenderRayWalk::from_ray` quantizes a world ray into every placement's grid.
The surface is the boundary between materials. In each placement the ray
remembers the material of the cell it is inside. Entering a live cell of another material
is an entry. Leaving a material for an empty cell, the outside of the grid,
or a material that is not opaque is an exit. An exit comes before the entry
at the same face. A slab of one material shows its near and far faces. A ray
that starts inside a cell leaves that cell's material without an exit.
`cast_ray` returns the walk's first hit within a distance. A hit carries:

1. The placement
2. The cell
3. The face the ray crossed, as a unit `SurfaceSpan`
4. Where on the face the ray landed, as fractions and as a fixed point
5. The distance
6. Whether the ray left the cell through the face

A shadow ray is the same walk from a fixed point on the face toward a
light. `RenderGridRay::toward` ends it at the light.

```rust
let ray = RenderViewRays::new(&view, 1024, 1024).ray(512, 512);

for hit in RenderRayWalk::from_ray(&scene, &ray)? {
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
4. A hemisphere light mixes sky and ground by the +Y of the normal, or of
   the view's mirror direction for the transmitted share

Under `RenderOcclusion::Corner`, the corner occlusion darkens the hemisphere
light. It counts only the cells whose material passes no light, so glass
darkens nothing it encloses. An exit shades with its normal turned back into
the material it leaves. `inner_corner_occlusion` reads the exit's occlusion
on that side. A shadow is one grid ray toward the light. It starts with the
pass of the material it starts inside and transmits by the pass of each
surface it meets. A red pane throws a red shadow. `RenderShadow` casts it per
pixel, per face, or per corner, and blends the corner results across the face
per channel.

A `RenderBloom` adds a halo over the emissive term before the tonemap. The
part of each hit's emission whose brightest channel passes the threshold
blurs out to the radius, scales by the strength, and lands on every pixel.
The default strength of `0` skips the pass.

A pixel walks its ray front to back. Each hit adds its shade at its base
color's alpha and passes the rest through by the material's pass: the share
of the light behind the surface that `transmission` lets through. The pass
loses Schlick's Fresnel on the ray's cosine to the face. Glass passes less
and reflects more of the hemisphere toward grazing. Glass shows what lies
behind it and tints at each wall. Touching voxels of one glass read as one
slab.

```rust
let image = render(
    &scene,
    view_id,
    RenderOcclusion::Corner,
    RenderBloom::default(),
    1024,
    1024,
)?;
```
