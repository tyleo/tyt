# The contract

_Part of the
[voxel rendering plan](../../plan/closed/voxel-rendering/README.md)._

The contract says what the image of a scene is. Every renderer implements it
through `voxrender`'s `Render` trait. The CPU reference in `voxrender-cpu`
follows it literally and makes the review images and the golden images.
[`vxl object render`](render.md) runs the reference, and its
[profiles](profile-language.md) and flags mirror the views and lights below.

## Frame

The frame is voxj's: glTF Y-up, right-handed, +Z toward the viewer. A voxel at
`p` fills the unit cube with min corner `p`. Placement follows the node DAG, one
path making one placement. The document measures meters, and the scene scale
multiplies the whole scene.

## Surface

The surface is the boundary between materials, read along a ray. In each
placement the ray remembers the material of the cell it is inside: nothing
before it enters the grid and nothing after it enters an empty cell. The ray
meets the surface two ways:

1. Entering a live cell whose material differs from the one the ray is
   inside is an entry through the face it entered by
2. Leaving a material for an empty cell, the outside of the grid, or a live
   cell of another material that is not opaque is an exit through the face
   it leaves by. Leaving for an opaque cell is no exit. The opaque cell's
   entry is the only hit on that face

An exit comes before the entry at the same face. Cells of one material have
no seam between them, so a slab shows its near and far faces. A ray that
starts inside a cell leaves that cell's material without an exit. A view from
inside a voxel sees out of it. Each placement walks on its own, and the hits
merge by distance, ties in placement order. A ray from outside the grid first
hits the boundary between live and non-live cells that `voxsurface`
enumerates. Faces are axis-aligned unit squares with flat normals. There is
no smoothing, no bevel, and no sub-voxel detail.

The walk runs in integers so every renderer makes the same choices. A ray
enters each placement's walk quantized into its grid: a fixed-point origin
with 13 fraction bits and integer direction components of at most 2^16. A
view's ray for a pixel comes from integer steps across the image carrying 12
guard bits, rounded once, halves up. A grid's sides, each ray's origin, and
the origin's mirror across the grid lie within 2^18 cells of the grid's
origin, so the walk's positions fit 32-bit integers. A render whose ray or
light lies out of that range errors. The walk steps across the boundary the
ray reaches first, judged by the sign of an integer error term between each
pair of axes. A tie steps x, then y, then z. A point on a boundary belongs to
the cell the ray moves into, and on an axis the ray does not move along, to
the cell above. A ray from outside a grid enters it as though it had walked
there from its origin. A hit's distance in meters comes from the integer
state in `f64`.

## Materials

A voxel carries its effective palette material, in voxj's glTF vocabulary with
glTF's defaults. The render shades `baseColor`, `metallic`, `roughness`,
`emissiveColor`, `emissiveStrength`, and `occlusionStrength`. `ior` sets the
dielectric reflectance at normal incidence, `((ior - 1) / (ior + 1))^2`: `0.04`
at the default `1.5` and `1` at `0`. `baseColor`'s alpha is the coverage, the
share of a pixel the voxel's surface fills. `transmission` is the share of the
diffuse light that passes through the surface.

## Shading

Shading is glTF's metallic-roughness model in linear light: Lambert diffuse,
GGX specular with Smith visibility and Schlick Fresnel, and the emissive term.
Both the direct and the hemisphere light scale the diffuse term by one minus
`transmission`. The hemisphere term mixes sky and ground by the normal's +Y
component, scaled by the occlusion and `occlusionStrength`.

What continues past a hit is the material's pass, one factor per channel.
The transmitted share loses Schlick's Fresnel `F` on the cosine between the
ray and the face normal in world space. The roughness caps the rise of `F`
toward grazing:

```
F = F0 + (max(1 - roughness, F0) - F0) * (1 - cos)^5
pass = (1 - alpha) + alpha * transmission * (1 - metallic) * (1 - F) * baseColor
```

The first term is the part of the pixel the surface does not cover. The
second is the light the covered part transmits: the dielectric share, less
what reflects, tinted by the base color. The transmitted share reflects `F`
of the hemisphere light. That reflection mixes sky and ground by the +Y of
the view's mirror direction. A ray that starts inside a material pays the
material's pass at normal incidence because the face the ray entered by is
unknown. An opaque material's pass is zero. The ray never bends.
Transmission is glTF's thin-surface model without the volume extension:
light passes straight through, tinted once at each surface it crosses. A
pane tints at its near and far walls.

An exit shades as an entry does, with its normal turned back along the ray
into the material it leaves. The lights that reach it cross that material.

A pixel walks its ray front to back with a throughput of one. At each hit
the pixel adds `throughput * alpha * shade` to its light and `throughput *
alpha * emission` to the bloom's emissive term, then multiplies the
throughput by the pass at the view ray's cosine. The walk ends at a zero
throughput or when the ray leaves every grid. What remains is the pixel's
transmittance.

## Lights

A scene lights with a list of lights, each with a color and a strength, plus
one occlusion switch. There are four kinds:

1. A `directional` light is a rotation. It shines down its local -Z, as a
   light under glTF's `KHR_lights_punctual` does. It carries a shadow
   granularity
2. A `point` light is a position with glTF's punctual falloff: inverse
   square, an optional `range`, and glTF's smooth cutoff at that range. It
   carries a shadow granularity too. Falloff runs in meters after the scene
   scale applies, so one rig lights a large scene differently from a small one
3. A `spot` light is a pose. It sits at a position and shines down its
   local -Z with a point light's falloff, times glTF's cone falloff: full
   strength inside an inner cone half-angle, none past an outer one, and a
   smooth ramp between. The angles are in degrees, `0` and `45` by default.
   The inner angle stays below the outer, which is at most `90`. It carries
   a shadow granularity too
4. A `hemisphere` light is the ambient term, a sky color above and a ground
   color below, about world +Y. It has no transform

The occlusion switch is `none` or `corner`. `corner` is the
neighbor-occupancy rule voxel art uses, one value per face corner from the
three adjacent cells, implemented once in `voxsurface` for the reference and
`object mesh` alike. A cell counts as occupied only when its material's pass
is zero, so glass darkens nothing it encloses. An exit's corners read the
cells on its material's side of the face.

A shadow is one grid ray toward the light. The ray starts on the face's plane,
which the boundary rule puts in the cell in front of a lit face. It runs to
infinity for a directional light. Toward a point or spot light its direction
is the integer offset to the light, shifted right until it fits, and it ends
in the cell it reaches at the light's coordinate on the axis it travels
farthest. Its throughput starts at the pass of each material the ray starts
inside, which the light crossed to reach the point. At each surface the ray
meets, the throughput multiplies by the pass at the shadow ray's cosine, so a
shadow is a color. A red pane throws a red shadow tinted at both its walls. A
pane two voxels thick throws the shadow a thin one throws. A floor under a
pane is lit through it. Each light scales its contribution by what remains. A
light samples the ray at one of three granularities:

1. `per-pixel` casts the ray from the hit, rounded to fixed point: a crisp
   diagonal edge across faces, the MagicaVoxel render and Teardown look
2. `per-face` casts it from the face center: one value per voxel face, a crisp
   staircase at voxel resolution, the look Minecraft's Vibrant Visuals snaps
   to
3. `per-corner` casts it from each corner and blends the four bilinearly
   across the face, per channel: a soft staircase, the vanilla Minecraft
   smooth-lighting look with a sun

A per-pixel or per-corner start sits at least 2^-10 of a voxel in from the
face's edges, which keeps it in front of the hit's cell.

`none` turns a light's shadow off. `per-corner` is the default look. It reads
as one look with the corner occlusion.

## Transforms

Every view and light resolves to one world-space pose, a position and a unit
quaternion. Object placements keep the full transform their nodes carry,
because voxels scale and cameras do not. A view and a spot light read both
parts of their poses. A directional light reads the rotation, and a point
light reads the position. The configured form is four shapes, one per entity,
named by what it carries: a pose for a view, a rotation for a directional
light, a position for a point light, and a spot pose, a pose over every frame,
for a spot light. An entity is never handed a part or a frame it has no use
for. Each shape is a tagged union over the frame its values are read in:

1. `world` is the document's frame
2. `subject` has world axes centered on the subject's bounds
3. `camera` is the view being rendered, so a light in it follows every view
4. `orbit` is a position on a sphere about the subject's center or a world
   point, facing it
5. `node` is one hierarchy node path's world transform, scale included, so a
   camera can ride a player. The entry carries a `path`, a glob over node
   paths under the shared
   [glob rules](../../plan/open/vxl-commands/reference/conventions.md#glob-patterns),
   matched as `node list` matches them. The glob must match exactly one
   path. Zero or several matches error, listing the paths that matched. The
   position and rotation are read on the node's axes and compose with the
   path's world transform, the one the flatten gives the path's placements
   with the scene scale applied

A view takes `world`, `subject`, `orbit`, or `node`. A directional light
takes `world`, `camera`, or `node`. A point light and a spot light take all
five.

A rotation is one of four forms, shared by every shape:

1. `quaternion`, as a node stores it
2. `euler`, angles about the fixed x, y, then z axes, as `node set rotation`
   takes them
3. `look-at`, toward a point in the frame
4. `angles`, the rotation of something at that spherical direction about the
   frame's origin, facing the origin. The azimuth runs from +Z toward +X and
   the elevation toward +Y. For a light this is where it comes from

`look-at` and `angles` take the frame's +Y as up. A direction along +Y or -Y
takes -Z as up instead, so a `top` view shows the front at the bottom of the
image. A look-at whose eye is its target errors. Transforms resolve in the
order subject bounds, then views, then lights, because a `camera`-frame light
resolves once per view. Floats stay `f64` until the walk quantizes them into
each placement's grid, the boundary the reference and every tier share. A
ray or light 2^32 or more cells from a placement's grid errors. Ids pack at
a GPU's upload.

## Views

A view is a named camera: a pose transform, a projection, and a vertical field
of view or an orthographic scale. The projection is perspective unless the
view says otherwise. The subject defaults to the rendered objects, and a
view's `select` narrows it with hierarchy-path globs at the object level, so
every placement of a matched object joins the subject. An orbit's distance
defaults to `fit`, the rule `tyt fbx render` uses. The subject's world-space
bounds give a center and a diagonal. The camera sits on its orbit at the
distance, or orthographic scale, that fits the bounding sphere of that
diagonal into the shorter image axis with a small margin. A sphere fits
regardless of orientation, so every fitted view of one subject sits at one
distance. An orbit about a world point takes its distance and orthographic
scale in meters.

## Bloom

Bloom is a halo over the emissive term alone, added to the linear image
before the tonemap, so a key light never makes a white face glow. A
material's `emissiveStrength` says how hard it glows. Three scalars say how
the lens responds:

1. `bloomStrength` scales the halo. `0`, the default, skips the pass
2. `bloomRadius` sets the halo's reach as a fraction of the shorter image
   side, `0.03` by default. The fraction makes a small render glow like a
   large one. The radius is the standard deviation of the widest blur
3. `bloomThreshold` sets the level in linear light an emission's brightest
   channel must exceed, `1` by default. Every hue then blooms alike. At `1` a
   material at glTF's default `emissiveStrength` stays flat and one pushed
   above it glows

The pass runs in four steps:

1. Takes the part of each hit's emission whose brightest channel passes the
   threshold, with its hue kept
2. Blurs it by a Gaussian per octave from the radius halving to one pixel
3. Averages the octaves and scales by the strength
4. Adds the halo to every pixel

The halo adds to a pixel's light and scales the pixel's transmittance by one
minus the halo's peak channel, clamped to one. A pixel no ray hit that the
halo reaches comes out with the halo's peak as its alpha and the halo over
that alpha as its color. Compositing that pixel over black gives the halo
back.

## Output

A pixel carries two colors: the light that reached it, in linear radiance,
and its transmittance, the share of what lies behind the scene that passes
through per channel. A miss carries no light and full transmittance. An
opaque hit carries its shade and none.

The output derives a coverage from the pair: the larger of one minus the
peak transmittance and the peak of the light, clamped to one. The light over
that coverage runs through the Khronos PBR Neutral tonemap. Scaling the
result by the coverage gives the pixel's layer. The peak of the light keeps a
highlight on clear glass from vanishing into a tiny coverage.

Under a background color the output adds the background scaled by the
transmittance to the layer, clamps the sum to one, and writes it at full
alpha. The background never passes through the tonemap because the tonemap
would turn a white background grey behind clear glass. Red glass over white
is red, and clear glass over white stays white.

Under `transparent` the output writes 8-bit RGBA with straight alpha. It
solves each pixel to be exact over white for a viewer that blends sRGB
values, as browsers and image editors do. Per channel, `W` is the sRGB
encoding of the layer plus the transmittance, clamped to one: the pixel over
white. The transparency `t` is the smaller of the peak transmittance and the
least channel of `W`:

```
alpha = 1 - t
color = (W - t) / alpha, in sRGB values, or 0 at a zero alpha
```

An opaque pixel has a `t` of zero and keeps the layer's color at full alpha.
A miss has a `t` of one. Colored glass keeps its tint over any backdrop. Over
a dark backdrop colored glass reads brighter than it should because one alpha
cannot be exact over two backdrops.

PBR Neutral keeps base colors true until highlights compress, and a voxel
palette is what a reviewer most needs to see unchanged. The default image is
1024 by 1024: square suits a single asset, and a reviewer's model downsamples
anyway.

## Determinism

Nothing in the render is random. The walk runs in integers, so which face
a pixel or shadow ray meets is the same on every renderer. Tests compare
images per channel within a small tolerance, never byte for byte, because
shading's float math differs across platforms and GPUs.
