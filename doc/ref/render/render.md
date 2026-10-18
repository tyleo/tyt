# `vxl object render`

_Part of the
[voxel rendering plan](../../plan/closed/voxel-rendering/README.md)._

```sh
vxl object render <input> [options]
```

`vxl object render` draws the selected objects into images. An image draws
one view or tiles several into a sheet, and each view follows
[the contract](contract.md). The views, images, and lights come from
[profiles](profile-language.md) and the flags that mirror them. A run that
sets no view and no image renders the built-in `hero` view, and one that sets
no light uses the built-in `studio` rig.

```sh
# the hero view under the studio rig, inline in the terminal
vxl object render turret.voxj

# turret-hero.png, turret-front.png, turret-right.png, turret-back.png, and
# turret-left.png beside the input
vxl object render turret.voxj
  --profile turnaround
  --to png

# turret.png, the review sheet of the hero view and four orthographic sides
vxl object render turret.voxj
  --profile review
  --to png

# an orthographic view from the front-left-top, its camera flags printed
vxl object render turret.voxj
  --view-orbit iso -45 30 fit
  --view-projection iso orthographic
  --print-camera

# the hero view under the studio rig, the emissive materials glowing
vxl object render turret.voxj --profile glow
```

The images show inline in the terminal one below another, each scaled to fit,
through the Kitty or iTerm2 graphics protocol or as ANSI half blocks where
neither is supported. `--to png` writes them beside the input instead: one
image as the input's stem with `.png`, and several as the stem, a hyphen, and
the image's name, as `object mesh` joins object names under `--split-files`.
`--file-stem` replaces the stem, and each PNG's path prints on its own line.
`--print-camera` prints each drawn view's resolved pose after its image as the
flags that reproduce it, ready to paste back as a `world` view or a profile
entry.

`--select` and `--select-index` choose the objects, and the default takes
every object. The selected objects render together, placed by the hierarchy
nodes reaching them, as [`object mesh`](../mesh/mesh.md) arranges them in one
mesh. A document with no objects errors. See
[Object selectors](../../plan/open/vxl-commands/reference/conventions.md#object-selectors).

## Options

1. `--from <format>`
   - Default: the input extension
   - Repeatable: no

   The source voxel format.

2. `--to <terminal | png>`
   - Default: `terminal`
   - Repeatable: no

   Where the views go.
   1. `terminal`: inline in the terminal, one view below another.
   2. `png`: a PNG per image beside the input, each path printed on its own
      line.

3. `--file-stem <file-stem>`
   - Default: the input's stem
   - Repeatable: no

   The stem the PNGs are named by under `--to png`. Under the terminal it
   errors.

4. `--width <pixels>`
   - Default: `1024`
   - Repeatable: no

   The image width. Zero errors.

5. `--height <pixels>`
   - Default: `1024`
   - Repeatable: no

   The image height. Zero errors.

6. `--background <transparent | #RRGGBB>`
   - Default: `#FFFFFF`
   - Repeatable: no

   What lies behind the scene. A color fills the pixels no ray hits at full
   alpha and shows through transparent voxels. The default stays opaque
   because an image reader that drops alpha shows a transparent background as
   black. A shell reads a bare `#` as a comment, so quote the color.

7. `--occlusion <none | corner>`
   - Default: `corner`
   - Repeatable: no

   The occlusion the render shades with; see [Lights](contract.md#lights).

8. `--scene-scale <scale>`
   - Default: `1.0`
   - Repeatable: no

   A uniform scale on the document's meters, applied to the whole scene. A point
   or spot light's falloff and `range` run in meters after it applies.

9. `--bloom-strength <strength>`
   - Default: `0`
   - Repeatable: no

   The factor scaling the bloom halo over the emissive term; see
   [Bloom](contract.md#bloom). `0` skips the pass. The built-in `glow` and
   `review` profiles set `1`. A negative strength errors.

10. `--bloom-radius <fraction>`
    - Default: `0.03`
    - Repeatable: no

    The halo's reach as a fraction of the shorter image side. Zero errors.

11. `--bloom-threshold <threshold>`
    - Default: `1`
    - Repeatable: no

    The level in linear light an emission's brightest channel must exceed to
    bloom. At `1` a material at glTF's default emissive strength stays flat. A
    negative threshold errors.

12. `--profile <profile>`
    - Repeatable: yes

    Applies a profile whole, expanding it into its flags with its `viewsFrom`
    and `lightsFrom` imports first. An explicit flag replaces the profile
    element it collides with. Repeated, the profiles stack in line order, the
    views merging by name and the lights as one rig, and an element two of
    them set errors; see the [profile language](profile-language.md#stacking).
    `vxl profile object render list` lists the profiles a run can apply.

13. `--views-from <profile>`
    - Repeatable: yes

    Applies a profile's views alone, its `viewsFrom` imports first. The
    images, the rig, and the image settings stay behind.

14. `--images-from <profile>`
    - Repeatable: yes

    Applies a profile's images alone, its `imagesFrom` imports first. The
    views, the rig, and the image settings stay behind.

15. `--lights-from <profile>`
    - Repeatable: yes

    Applies a profile's light rig alone, its `lightsFrom` imports first. The
    views, the images, and the image settings stay behind.

16. `--view-frame <view> <world | subject | node>`
    - Repeatable: yes

    The frame the named view's `--view-position` and rotation are read in.
    Every `--view-*` flag names its view, and the first mention creates the
    view. A created view takes an image of its name until the run gives an
    `--image*` flag. A posed view takes `--view-frame`, `--view-position`, and
    one rotation flag, plus `--view-node` under `node`, and a view whose
    transform neither the flags nor a profile sets errors. A flag setting an
    element set already errors.

17. `--view-node <view> <path>`
    - Repeatable: yes

    The node path the named view's `node` frame reads its position and
    rotation in: a glob over hierarchy node paths under the shared
    [glob rules](../../plan/open/vxl-commands/reference/conventions.md#glob-patterns),
    matched as `node list` matches them. It must match exactly one path.
    Zero or several matches error, listing the paths that matched. The flag
    errors under another frame, and a `node` frame without it errors.

18. `--view-position <view> <x> <y> <z>`
    - Repeatable: yes

    The named view's position in its frame, in meters.

19. `--view-quaternion <view> <x> <y> <z> <w>`
    - Repeatable: yes

    The named view's rotation as a unit quaternion, the form `--print-camera`
    prints. A quaternion off unit length errors. The four rotation flags share
    a view's one rotation, so two on one view error.

20. `--view-euler <view> <x> <y> <z>`
    - Repeatable: yes

    The named view's rotation as Euler angles in degrees about the fixed x,
    y, then z axes, as `node set rotation` takes them.

21. `--view-look-at <view> <x> <y> <z>`
    - Repeatable: yes

    Aims the named view's -Z at a point in its frame, with the frame's +Y up.
    A target at the view's position errors. Beside `--view-orbit`, the point
    sets the orbit's center in world space, and the orbit takes its distance
    in meters.

22. `--view-angles <view> <azimuth> <elevation>`
    - Repeatable: yes

    The named view's rotation as the one facing its frame's origin from the
    direction at an azimuth from +Z toward +X and an elevation toward +Y, in
    degrees.

23. `--view-orbit <view> <azimuth> <elevation> <distance | fit>`
    - Repeatable: yes

    Places the named view on a sphere about the subject's center or the
    `--view-look-at` point, facing it, at an azimuth and elevation in degrees
    and a distance in meters or `fit`, the bounding-sphere rule under
    [Views](contract.md#views). It sets the whole transform, so it errors
    beside `--view-frame`, `--view-position`, or any other rotation flag.

24. `--view-projection <view> <perspective | orthographic>`
    - Default: `perspective`
    - Repeatable: yes

    The named view's projection.

25. `--view-fov <view> <degrees>`
    - Default: `35`
    - Repeatable: yes

    The named view's vertical field of view under `perspective`. It errors
    under `orthographic`, and at 180 degrees or more.

26. `--view-scale <view> <units | fit>`
    - Default: `fit`
    - Repeatable: yes

    The world units across the shorter image side under `orthographic`. It
    errors under `perspective`.

27. `--view-select <view> <glob>`
    - Default: the rendered objects
    - Repeatable: yes

    Narrows the named view's subject, which its `subject` frame and its orbit
    are about, to the rendered objects a hierarchy-path glob matches. Repeated
    on one view, the globs union. Every placement of a matched object joins
    the subject, and a glob matching nothing errors.

28. `--image <image>`
    - Repeatable: yes

    Declares the named image. Every `--image*` flag names its image, and the
    first mention creates the image. The name suffixes the file and holds no
    path separator.

29. `--image-kind <image> <view | sheet>`
    - Default: `view`
    - Repeatable: yes

    The named image's kind. A `view` image draws one view, and a `sheet` tiles
    several row by row in a near-square grid.

30. `--image-view <image> <view>`
    - Default: the image's name
    - Repeatable: yes

    The view the named `view` image draws.

31. `--image-views <image> <view>`
    - Repeatable: yes

    Adds a view to the named sheet's tiles in order. The flags replace a
    profile's list whole.

32. `--image-border-width <image> <pixels>`
    - Default: `8`
    - Repeatable: yes

    The width of the border between the named sheet's tiles. `0` sets the
    tiles edge to edge.

33. `--image-border-color <image> <#RRGGBB>`
    - Default: `#808080`
    - Repeatable: yes

    The color of the border between the named sheet's tiles.

34. `--light <light-index> <directional | point | spot | hemisphere>`
    - Default: the profile stack's rig
    - Repeatable: yes

    Declares the indexed light's kind. Lights number from `0` with no gaps,
    so an index declared twice or skipped errors. Any `--light` replaces the
    stack's rig whole. The other `--light-*` flags fill the declared lights,
    or the stack's rig when nothing is declared, and an index the rig holds
    no light at errors. A flag whose element the light's kind never reads
    errors, as does one setting an element set already. A directional light
    takes `--light-frame` and one rotation flag. A point light takes
    `--light-frame` and `--light-position`, or `--light-orbit` for the whole
    transform. A spot light takes `--light-frame`, `--light-position`, and
    one rotation flag, or `--light-orbit`. A hemisphere light has no
    transform. A directional, point, or spot light whose transform neither
    the flags nor the rig sets errors.

35. `--light-frame <light-index> <frame>`
    - Repeatable: yes

    The frame the indexed light's transform is read in: `world`, `camera`,
    or `node` for a directional light, and `world`, `subject`, `camera`, or
    `node` for a point or spot light.

36. `--light-node <light-index> <path>`
    - Repeatable: yes

    The node path the indexed light's `node` frame reads its transform in, a
    glob as `--view-node` takes. The flag errors under another frame, and a
    `node` frame without it errors.

37. `--light-position <light-index> <x> <y> <z>`
    - Repeatable: yes

    The indexed point or spot light's position in its frame, in meters.

38. `--light-quaternion <light-index> <x> <y> <z> <w>`
    - Repeatable: yes

    The indexed directional or spot light's rotation as a unit quaternion.
    The four rotation flags share a light's one rotation, so two on one
    light error.

39. `--light-euler <light-index> <x> <y> <z>`
    - Repeatable: yes

    The indexed directional or spot light's rotation as Euler angles in
    degrees about the fixed x, y, then z axes.

40. `--light-look-at <light-index> <x> <y> <z>`
    - Repeatable: yes

    Aims the indexed directional or spot light's -Z at a point in its frame,
    a directional light from the frame's origin. A target at the light
    errors.

41. `--light-angles <light-index> <azimuth> <elevation>`
    - Repeatable: yes

    The direction the indexed directional or spot light shines from, as an
    azimuth from +Z toward +X and an elevation toward +Y, in degrees.

42. `--light-orbit <light-index> <azimuth> <elevation> <distance>`
    - Repeatable: yes

    Places the indexed point or spot light on a sphere about the subject's
    center at an azimuth and elevation in degrees and a distance in meters,
    a spot facing the center. It sets the whole transform, so it errors
    beside `--light-frame`, `--light-position`, or a rotation flag.

43. `--light-shadow <light-index> <none | per-pixel | per-face | per-corner>`
    - Default: `per-corner`
    - Repeatable: yes

    The indexed directional, point, or spot light's shadow granularity; see
    [Lights](contract.md#lights).

44. `--light-color <light-index> <#RRGGBB>`
    - Default: `#FFFFFF`
    - Repeatable: yes

    The indexed directional, point, or spot light's color, an sRGB hex.

45. `--light-strength <light-index> <strength>`
    - Default: `1`
    - Repeatable: yes

    The factor scaling the indexed light's color, zero or more. A headlight
    at pi renders a white base color as white.

46. `--light-range <light-index> <meters>`
    - Repeatable: yes

    The distance the indexed point or spot light reaches. Without it, the
    light has no cutoff.

47. `--light-cone <light-index> <inner> <outer>`
    - Default: `0 45`
    - Repeatable: yes

    The indexed spot light's cone as two half-angles in degrees. Full
    strength holds inside the inner angle and fades to nothing at the outer,
    glTF's cone falloff. The inner angle is zero or more and below the
    outer, which is at most `90`. Any other pair errors, as does the flag
    on another kind.

48. `--light-sky <light-index> <#RRGGBB>`
    - Default: `#FFFFFF`
    - Repeatable: yes

    The indexed hemisphere light's color from above.

49. `--light-ground <light-index> <#RRGGBB>`
    - Default: `#FFFFFF`
    - Repeatable: yes

    The indexed hemisphere light's color from below.

50. `--select <glob>`
    - Default: `*`, selecting every object
    - Repeatable: yes

    Chooses objects by hierarchy path, with a node path selecting its
    subtree. Unions with `--select-index`. Any explicit selector replaces the
    default, so `--select-index` alone never unions with `*`. See
    [Object selectors](../../plan/open/vxl-commands/reference/conventions.md#object-selectors).

51. `--select-index <index>`
    - Repeatable: yes

    Chooses objects by id: an integer, an `a-b` range, or `*` for every
    object. An id the document lacks errors. Unions with `--select`.

52. `--print-camera`
    - Default: off
    - Repeatable: no

    Prints each drawn view's resolved pose as its `--view-frame <view>
    world`, `--view-position`, and `--view-quaternion` flags on one line,
    after the image in the terminal.
