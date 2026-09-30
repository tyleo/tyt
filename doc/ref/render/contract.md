# The contract

_Part of the [voxel rendering plan](../../plan/open/voxel-rendering/README.md)._

The contract says what the image of a scene is. Every renderer implements it.
The CPU reference in `voxrender` follows it literally and makes the review
images and the golden images. A realtime tier turns off the features it cannot
afford, and the reference renders that tier's golden images with the same
features off. [`vxl object render`](render.md) runs the reference, and its
[profiles](profile-language.md) and flags mirror the views and lights below.

## Frame

The frame is voxj's: glTF Y-up, right-handed, +Z toward the viewer. A voxel at
`p` fills the unit cube with min corner `p`. Placement follows the node DAG,
one path making one placement. A voxel size in meters scales the whole scene,
as it scales an [`object mesh`](../mesh/mesh.md) output.

## Surface

The surface is the boundary faces between live and non-live cells, as
`voxsurface` enumerates them. Faces are axis-aligned unit squares with flat
normals. There is no smoothing, no bevel, and no sub-voxel detail.

## Materials

A voxel carries its effective palette material, in voxj's glTF vocabulary with
glTF's defaults. The render shades `baseColor`, `metallic`, `roughness`,
`emissiveColor`, `emissiveStrength`, and `occlusionStrength`. Every live voxel
is opaque. Alpha, `transmission`, and `ior` are deferred.

## Shading

Shading is glTF's metallic-roughness model in linear light: Lambert diffuse,
GGX specular with Smith visibility and Schlick Fresnel, and the emissive term.
The hemisphere term mixes sky and ground by the normal's +Y component, scaled
by the occlusion and `occlusionStrength`.

## Lights

A scene lights with a list of lights, each with a color and a strength, plus
one occlusion switch. There are three kinds:

1. A `directional` light is a rotation. It shines down its local -Z, as a
   light under glTF's `KHR_lights_punctual` does. It carries a shadow
   granularity
2. A `point` light is a position with glTF's punctual falloff: inverse
   square, an optional `range`, and glTF's smooth cutoff at that range. It
   carries a shadow granularity too. Falloff runs in meters after the voxel
   size applies, so one rig lights a large voxel size differently from a
   small one
3. A `hemisphere` light is the ambient term, a sky color above and a ground
   color below, about world +Y. It has no transform

The occlusion switch is `none` or `corner`. `corner` is the
neighbor-occupancy rule voxel art uses, one value per face corner from the
three adjacent cells. It has one implementation, in `voxsurface`: the
reference shades with it, `object mesh` bakes it, and a standalone tier stores
it with its faces. Traced occlusion is a later value.

A shadow is one grid ray toward the light. The ray runs to infinity for a
directional light and ends at a point light. A light samples it at one of
three granularities:

1. `per-pixel` casts the ray from the hit: a crisp diagonal edge across faces,
   the MagicaVoxel render and Teardown look
2. `per-face` casts it from the face center: one value per voxel face, a crisp
   staircase at voxel resolution, the look Minecraft's Vibrant Visuals snaps
   to
3. `per-corner` casts it from each corner and blends the four bilinearly
   across the face: a soft staircase, the vanilla Minecraft smooth-lighting
   look with a sun

`none` turns a light's shadow off. `per-corner` is the default look. It reads
as one look with the corner occlusion, and a standalone tier computes it
exactly where `per-pixel` needs a shadow map.

## Transforms

Every view and light resolves to one world-space pose, a position and a unit
quaternion. Object placements keep the full transform their nodes carry,
because voxels scale and cameras do not. A view reads both parts of its pose.
A directional light reads the rotation, and a point light reads the position.
The configured form is three shapes named by what they carry: a pose for a
view, a rotation for a directional light, and a position for a point light.
An entity is never handed a part it has no use for. Each shape is a tagged
union over the frame its values are read in:

1. `world` is the document's frame
2. `subject` has world axes centered on the subject's bounds
3. `camera` is the view being rendered, so a light in it follows every view
4. `orbit` is a position on a sphere about the subject's center, facing it

A view takes `world`, `subject`, or `orbit`. A directional light takes `world`
or `camera`. A point light takes all four.

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
resolves once per view. Floats stay `f64` until a GPU upload boundary narrows
them, where ids pack too.

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
distance.

## Output

The image is linear light through the Khronos PBR Neutral tonemap, then the
sRGB transfer to 8-bit RGBA with straight alpha. A pixel no ray hits is
transparent, or the background color at full alpha. PBR Neutral keeps base
colors true until highlights compress, and a voxel palette is what a reviewer
most needs to see unchanged. The default image is 1024 by 1024: square suits a
single asset, and a reviewer's model downsamples anyway.

## Determinism

Nothing in the render is random. Tests compare images per channel within a
small tolerance, never byte for byte, because float math differs across
platforms and GPUs.
