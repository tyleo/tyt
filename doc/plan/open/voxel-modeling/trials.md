# Trials

Step S12 of the [checklist](checklist.md) runs these prompts through the
vxl-model skill. Step S14 runs them five more times, and
[round 2](trials-round-2.md), [round 3](trials-round-3.md),
[round 4](trials-round-4.md), [round 5](trials-round-5.md), and
[round 6](trials-round-6.md) log those rounds.

## Running a trial

The [trial harness](../../../../projects/vxl/trials/README.md) runs
each prompt twice, in fresh headless Claude Code sessions outside the
repository. A session inside the repository could read the builder and the docs
in place of the skill.

The prompt goes in as a user would write it, with no mention of the skill. The
trial then also tests whether the skill's description loads the skill. The
session runs passes until Claude calls the model done, and nobody steps in.

Each prompt adds a section to the [log](#log), titled with the prompt's number.
The section records the passes each run took, the failures, the operations and
report data Claude wanted and lacked, and where the model's colors read flat.

## Prompts

The prompts climb from single props to scenes and interiors. The harness reads
them and their stresses from `prompts.json`.

| #   | Directory      | Prompt                                                                                                                                                           |
| --- | -------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | `chair`        | make a voxel chair with oak and ornate jewels in the top                                                                                                         |
| 2   | `lantern`      | make a voxel lantern with a candle glowing inside                                                                                                                |
| 3   | `chest`        | make a voxel treasure chest with the lid open and gold coins inside                                                                                              |
| 4   | `sword`        | make a voxel sword with a gold hilt and a gem in the pommel                                                                                                      |
| 5   | `tree`         | make a voxel oak tree                                                                                                                                            |
| 6   | `well`         | make a voxel stone well with a wooden roof and a bucket on a rope                                                                                                |
| 7   | `robot`        | make a voxel robot whose head and arms can turn                                                                                                                  |
| 8   | `cottage`      | make a voxel mushroom cottage with a round door and windows                                                                                                      |
| 9   | `potion`       | make a voxel glass potion bottle with a cork and red potion inside                                                                                               |
| 10  | `cart`         | make a voxel wooden cart with four wheels                                                                                                                        |
| 11  | `guitar`       | make a voxel acoustic guitar with a round sound hole and six strings                                                                                             |
| 12  | `candelabra`   | make a voxel iron candelabra with a twisted square stem and five lit candles on curling arms                                                                     |
| 13  | `knight`       | make a voxel knight character for my game, 32 voxels tall, in the pico-8 palette                                                                                 |
| 14  | `pocket-watch` | make a voxel pocket watch on a chain with the gears showing                                                                                                      |
| 15  | `dragon`       | model a voxel dragon curled up asleep on a pile of gold with its wings folded                                                                                    |
| 16  | `bridge`       | make a voxel suspension bridge over a river with two stone towers and hanging cables                                                                             |
| 17  | `chess-set`    | model a voxel chess set with every piece in its starting position                                                                                                |
| 18  | `ramen-stand`  | make a cyberpunk voxel ramen stand with a neon sign that says ramen                                                                                              |
| 19  | `dungeon-kit`  | make a voxel dungeon kit for my game: floor, wall, corner, and doorway tiles on a 2 m grid with the floors tagged walkable, then a small room built from the kit |
| 20  | `crane`        | build a voxel construction crane whose cab turns and boom tilts, with a hook hanging on a cable                                                                  |
| 21  | `fish-tank`    | make a voxel fish tank with fish, seaweed, gravel, bubbles, and a little castle inside                                                                           |
| 22  | `wizard-tower` | build a voxel wizard tower with stairs spiraling up the outside and a crooked pointed roof                                                                       |
| 23  | `log-cabin`    | make a voxel log cabin buried in snow with icicles on the roof and smoke from the chimney                                                                        |
| 24  | `valley`       | model a voxel mountain valley with a river and a pine forest, under 300k voxels                                                                                  |
| 25  | `village`      | build a voxel village of thatched cottages and a stone church around a little market square                                                                      |
| 26  | `city`         | build a small voxel city with streets, tall buildings, a park, and street lights as one object for my game                                                       |
| 27  | `dollhouse`    | build a voxel two-story house with the front wall cut away to show the furnished rooms                                                                           |
| 28  | `living-room`  | make a cozy voxel living room with a fireplace, an armchair, a bookshelf full of books, and a rug                                                                |
| 29  | `spaceship`    | make a voxel spaceship bridge with a captain's chair, a glowing radar screen, and a big window out to the stars                                                  |
| 30  | `pirate-ship`  | make a voxel pirate ship interior below deck with cannons, hammocks, barrels, and a hanging lantern                                                              |

## Stresses

Prompts 11 to 30 each push on something the first ten leave alone:

11. Six strings across a 5 cm neck need voxels of about 5 mm. The guitar then
    runs about 200 voxels long against the skill's 16 to 64. The body takes 2D
    booleans and stays hollow under a one-voxel top
12. A twist reads only once the stem's section spans several cells. Bends or
    torus arcs make the arms, and five arms repeat off the orthographic axes
13. `--resolution world-y 32` meets the height with an unround voxel size that
    breaks `const v`. The 16-color pico-8 palette rules out `shades` and tests
    where the armor reads flat
14. Millimeter voxels sit below every example in the skill, and gear teeth take
    one cell each. Interlocked links share no cells, so a correct chain reports
    a piece per link. Gold renders olive under the review lighting
15. The dragon's anatomy has no sweep or skeleton to build on and comes from
    `smoothUnion`, `roundCone`, and `bend`. The renders judge it by silhouette
16. The cables hang in curves that no operation draws and take computed points.
    Hangers of different lengths need a loop, and sloped one-voxel cables can
    split into pieces. The bridge spans about 10:1 in square review frames
17. A part keeps the materials of its steps, so each piece needs a part per
    color. Five lathe profiles and a sculpted knight make 32 placements. White
    pieces stand on light squares
18. Lettering has no operation and comes from strokes a whole cell wide. The
    sign has to read the right way round in the front view, and several
    emissive colors have to stay distinct without bloom
19. The walkable tag shows in neither the report nor the renders, and the log
    checks the palette with `vxl vox-doc show`. Building the room from the kit
    pushes the session toward a library or an import. Separate tiles read as
    extra pieces on a correct kit
20. Parts nest three or four deep, and their offsets and pivots add up. The
    renders cannot show a joint turn, and the log checks each pivot in the voxj
    nodes against its hinge
21. Opaque fish and the castle inside solid water read `0 exposed` on a correct
    model and render dark behind the water. Bubbles float as pieces. The glass
    takes `#RRGGBBAA`
22. A helix has no operation. A twist on a stack of treads makes one but breaks
    the treads into fragments. A bend shapes the crooked roof
23. `coat` lays the snow. The smoke floats on purpose against the second-piece
    check. The library's snow and ice render nearly alike, and icicles can
    vanish against the snow
24. Solid terrain over about 200 m passes 300k voxels at 1 m and forces surface
    fill, a coarser grid, or a shell. Displaced terrain leaves dozens of
    one-voxel crumbs in the piece lines. The trees and the river have to meet a
    surface the model cannot query
25. A reused cottage part cannot turn to face the square. Copies repeat their
    patterns exactly unless each copy takes its own seed. The church stands as
    the one building that is no copy
26. One voxel size has to serve a 100 m scene with sub-meter windows. The front
    and right views hide the streets, and the report runs hundreds of lines.
    "One object" pushes `--flatten objects`
27. The prompt asks for the cutaway, which makes the dollhouse the control for
    28 to 30. One voxel size has to stretch from the house down to chair legs
28. Every review view looks from outside, and the session has to find a way to
    show the room. The books need variety across copies
29. The hull hides the bridge from every outside view. The stars float as
    hundreds of pieces and inflate the bounds. The radar screen takes sectors
    and arcs that have to read on a glowing panel
30. A curved hull built as a shell of a union can leave inner walls. The
    hammocks sag as one-cell sheets, and the cannons sit in carved ports. The
    lantern lights nothing under the review lighting

## Findings

Round 1's analyses recorded 685 items across the 60 runs. Each item maps to one
or more findings below. The findings rank by how many prompts they reached, and
the four that call for no action come last. The round's gallery lists each
finding's items beside the renders.

1. In 27 prompts, edits chained to the pass line met the permission check. An
   in-place `sed`, a python heredoc, a `time` wrapper, and a shell variable each
   met it even unchained
2. In 24 prompts, large faces stayed one flat color without shades, noise, or a
   pattern. Terrain cut faces read as uniform noise or stripes
3. In 18 prompts, a session built by hand what a documented operation does.
   `coat` with sides seats gems and lays snow, `repeatPolar` radiates arms,
   `bend` gnarls limbs, and a pattern's frame gives each flame one gradient
4. In 18 prompts, a session ended on a flaw it had named, claimed a fix the
   renders contradict, stopped after one review, or passed over a visible flaw
5. In 17 prompts, dark woods, iron, stone, slate, and bark read near-black and
   lost their pattern. Library walnut and mahogany read near-black beside oak. A
   custom #7A5232 still read very dark
6. In 13 prompts, cylinders, cones, rounded boxes, and octahedrons a few voxels
   across read as cubes, plus signs, or rods. The API page recommends
   `octahedron` for small cut gems. The second resolution rule says a small
   octahedron reads as a plus sign
7. In 13 prompts, a detail added before an overlapping shape lost its cells.
   Nested shapes of one feature buried each other in any order until one
   gradient step replaced them. The third resolution rule predicts each case
8. In 12 prompts, sessions judged a pass from the report alone or never opened
   the top or right view. Defects sat in the skipped views
9. In 12 prompts, runs finer than the skill's 16 to 64 voxels across read
   better. A house 160 voxels across and guitars 260 long ran in minutes. A 200
   m valley shrank to about 100 m where `--fill-mode surface` or a coarser grid
   would have kept it
10. In 11 prompts, tubes, strokes, rods, and plates near a voxel thick wrote no
    cells or split into pieces. A torus broke at a tube radius of 0.7 voxel, a
    member just over a voxel broke once tilted, and a one-voxel rise voxelized
    as a ramp. About 2 voxels across held
11. In 10 prompts, gems, trees, props, and hanging parts floated or sank because
    sessions guessed or hand-computed their heights. Hand-written terrain
    functions ignored `smoothUnion` fillets and `displace`. Both terrain runs
    asked for a surface height query
12. In 10 prompts, thin shapes and whole objects turned off the axes sampled
    into ribs, stair steps, or fragments. An armchair at 45 degrees came out
    ribbed, a tilted dish turned its rings into stripes, and a one-cell hammock
    arc broke apart
13. In 10 prompts, adjacent materials blended because their colors sat too close
14. In 9 prompts, faint glass vanished against the white background and tinted
    glass turned a red liquid mauve. Alpha water darkened what sat inside it,
    ice faded to strips, and transmissive gems read pink or gray
15. In 9 prompts, `shades` errored on saturated and near-white bases. Golds fit
    only a narrow window of hex values. The error reported a linear-light
    channel and no base or spread that would fit
16. In 9 prompts, sessions misread argument semantics. They took `arch`'s max
    for the spring line, `mirror` axes for planes, and the wrong slot for
    `repeat`'s count. They also missed that `sector` angles under an extrude on
    y turn opposite to `rotate`, that a 3D `offset` insets depth, and that a
    second `bend` moves the base
17. In 9 prompts, sessions wanted a re-aimed hero, a close-up of one part, an
    interior view, or a frame for a long subject. They dug through `--help` and
    hit view errors
18. In 8 prompts, emission strengths of 3 and up washed flames, neon, and
    windows to peach, pink, or white. Library glow and ember washed out too.
    Strengths of 1 to 1.5 kept the hue
19. In 8 prompts, a post, roof, wall, or the subject's back hid what the prompt
    is about. Interior sessions carved walls to suit the fixed angle
20. In 7 prompts, library gold read olive, brass khaki, and steel navy. The
    review rig lights a metal's voxel faces only through the rig's blue-gray
    hemisphere
21. In 7 prompts, metallic above about 0.4 pulled custom and palette colors
    toward olive, slate, or navy and darkened them. Golds read gold at metallic
    0.35 to 0.4
22. `part()` takes no rotation. In 6 prompts, sessions rotated every shape of a
    kit, a village, or a tilted boom by hand. No step flips one side alone
    because `mirror` keeps the original and `scale` rejects -1
23. In 6 prompts, scene reports ran to tens of KB. Sessions grepped the reports
    down to piece lines. Parts that only place other parts print 0-voxel lines
    that read as defects
24. In 5 prompts, `displace` left one-voxel crumbs. Sessions carved the crumbs
    at report coordinates, reseeded, or bridged them. They asked for an opt-in
    step that drops small pieces
25. In 5 prompts, helical stairs, cables, spines, scrolls, a river gorge, and a
    drooping hat tip took hand-sampled segments. The ask covers tapered profiles
    and 2D function curves too
26. In 5 prompts, books, icicles, stars, and lit windows took hand-written
    hashes. Copies that varied only by seed still read as clones
27. In 5 prompts, pale surfaces under overhangs and even lit ivory turned cool
    blue-gray. Noise on snow showed dark-blue blotches. The studio rig's blue
    sky likely casts the tint
28. In 5 prompts, sessions skipped the box block-out and wrote detailed shapes
    from the first pass
29. In 4 prompts, judging how a material renders took full passes
30. In 4 prompts, sizes drifted from real ones or the silhouette moved away from
    the prompt across revisions
31. In 3 prompts, library rope read flat olive or khaki, and grass read a
    saturated toy green
32. In 3 prompts, grain streaked across boards or read as blotches. The round's
    API page said a grain's axis runs along the wood. The code cuts slabs across
    it
33. In 3 prompts, piece lines did not say what made a crumb, and a step line
    could not locate a cut inside it. Bend errors do not say where the shape
    lands
34. In 3 prompts, a coat without a tight `within` or a broad paint box recolored
    adjacent shapes
35. In 2 prompts, sessions wanted a render with a joint turned, a hook that
    hangs plumb from a tilted boom, and a rope that spans two parts
36. In 2 prompts, sessions built a fresh part per placement or voxelized with
    the default `--frame world`. No placement shared an object. Some sessions
    claimed shared instances anyway
37. In 2 prompts, dark mortar swamped stone patterns. The smaller the stones,
    the more the mortar covered
38. In 2 prompts, a correct chain reported one piece per link and a backdrop
    behind a window floated. Sessions fused the links and sealed the backdrop to
    reach one piece
39. `vox-doc show` prints no node pivots or palette custom properties. The skill
    never mentions the command. In 2 prompts, walkable tags and joints shipped
    unchecked because python reads of the voxj met the permission check
40. In 2 prompts, a round of exactly half a side failed when the side came from
    decimal corners
41. In 2 prompts, sessions wanted segmental arches, a bend past half a turn, and
    a droop that grows toward a cone's tip
42. In 2 prompts, the robot and crane runs could not confirm a pivot without
    reading the voxj
43. In 2 prompts, nothing in the report said an emissive blew out or sat behind
    glass
44. In 2 prompts, `set()` points on cell boundaries added stray cells, an
    odd-width sign could not center, and pivots took a hand-written `snap()`
45. In 2 prompts, a mirrored side or an inset box got max before min and errored
    with `box max must be past min`
46. In 2 prompts, a flame or lantern cast no glow on nearby glass or surfaces.
    Sessions faked the glow with emissive tints or spherical paints that banded
    into rings
47. In 1 prompt, lettering took hand-written pixel-font tables. Extruded letters
    smeared in the angled hero
48. In 1 prompt, sessions wanted darker crevices and lighter edges baked into
    the colors
49. In 1 prompt, sessions could not see inside a hollow body
50. In 1 prompt, a shell or negative offset of a union left walls at buried
    faces. A smooth hollow body took a hand-sampled outline instead
51. In 1 prompt, water filling a tank put every interior step at 0 exposed and
    joined loose bubbles into one piece
52. In 1 prompt, testing one shape's arguments took a scratch model. Writes
    under /tmp met the permission check, and a model there could not find the
    builder
53. In 1 prompt, straps cut through wheels that rubbed the bed, and nothing
    flagged the overlap
54. In 1 prompt, a `const paint` material broke every paint step with
    `paint is not a function`
55. In 1 prompt, a wall that later paints recolor on purpose read 0 kept, which
    the skill's checks call a defect. The report cannot tell a full repaint from
    a burial
56. In 1 prompt, a model kept its grid constant at 5 mm while voxelize ran at
    2.5 mm. The model's whole-cell math no longer held
57. In 1 prompt, `vxl --version` errored. No binary in the workspace takes the
    flag
58. In 1 prompt, a declared material that no step applies went unflagged
59. In 1 prompt, the `cells` pattern's joints did not line up across adjacent
    tiles
60. The report gives no exposed count per material or face. In 1 prompt, a coat
    that painted unintended faces showed only in the render
61. In 27 prompts, the two runs chose near-identical plans and hit the same
    errors. The runs' quality differed by how long each kept revising against
    the renders
62. In 19 prompts, workarounds used the tools as intended and call for no change
63. In 15 prompts, sessions made mistakes that no change to vxl, its docs, or
    the skill would prevent
64. In 7 prompts, the report caught real defects and the tools held up

No run asked for patterns inside patterns, a hue shift across shades, a part's
steps on another part's grid, or a TypeScript library group.

## Log

The first round ran on 2026-10-04 with vxl 0.4.0 built from `e9d994df` and
Claude Code 2.1.287 running Claude Opus 5.5 at high effort. Every run loaded the
skill from its description and called its model done. Of the 60 runs, 42 read
good and 18 fair. The runs took 221 passes, of which 37 failed. A run took a
median of 3 minutes, and all 60 cost $62. `~/voxel-trials/rounds/2026-10-04`
holds the round, and its gallery shows every run's renders.

### 1. Chair

| Run | Verdict | Passes | Failed | Minutes | Voxels          |
| --- | ------- | ------ | ------ | ------- | --------------- |
| 1   | good    | 6      | 2      | 1.7     | 2,930 at 2.5 cm |
| 2   | fair    | 5      | 3      | 1.6     | 2,392 at 2.5 cm |

Both runs built nearly the same oak chair skeleton from the skill's example. Run
1's three render reviews grew faceted gems into an ornate five-gem crest that
reads better than run 2's flat square gems and grey diamond.

Failures:

1. Run 1, pass 1 and run 2, pass 2: float drift on a 2-cell seat failed
   `chair/seat: box round must be at most half the shortest side` with
   `0.024999999999999994, not 0.025`
2. Run 1, pass 2 and run 2, pass 3: the gems came out too small to read as
   stones at 2 to 8 cells
3. Both runs, pass 3: run 1's torus bezels read lumpy because their tubes ran
   under a voxel. Run 2's wrote 0 cells
4. Run 1, pass 4: an edit chained to the pass line was denied by permissions
5. Run 1, final pass: the session admitted its custom gold still looks brassy.
   The crest front reads mostly gold rather than oak with gold trim
6. Run 2, pass 1: `chair/crest: arch max must be at least half the width` with
   `0.15, above min, not [0.15, 0.975]`
7. Run 2, pass 4: `chair/emeralds: box max must be past min` because the gem
   helper built an inverted table box for a 2-cell gem
8. Run 2, final pass: the session stopped without fixing the half-gold rail and
   greyish one-row diamond it had reported

Lacked:

None

Missed in the skill:

1. Neither run looked at the top render. Run 2 stopped after one review despite
   the skill's loop of revising until the model matches the prompt
2. Both runs drew torus bezels with tubes under a voxel. The Resolution section
   warns that such tubes drop or break up cells
3. Run 2 set `arch` max too low because it missed that SKILL.md gives the half
   circle the rectangle's width
4. Run 1 cut emeralds and amethysts as octahedrons of 1.4 to 1.6 voxel radius.
   The Resolution section says that size reads as plus signs

Colors: The library gold and run 1's custom `#E8B84A` read olive beside a
diamond that renders flat grey in both runs. Run 2's oak grain also breaks into
blotchy patches on the posts and seat.

### 2. Lantern

| Run | Verdict | Passes | Failed | Minutes | Voxels           |
| --- | ------- | ------ | ------ | ------- | ---------------- |
| 1   | fair    | 4      | 0      | 2.1     | 5,000 at 1 cm    |
| 2   | good    | 5      | 0      | 2.0     | 2,610 at 1.25 cm |

Both runs built nearly the same iron lantern with a `boxFrame` cage, a stepped
roof, and a candle on a brass dish. Cutting the flame's emission until it read
orange gives run 2 a clearer candle than the pale peach flame run 1 stopped on.

Failures:

1. Run 1, before pass 1: the first model write held a junk drip expression
   `box(...).union?.call ? ... : ...` that had to come out
2. Both runs, pass 1: the handle torus split into 4 pieces because its 0.007 m
   or 0.009 m tube sat under the voxel
3. Run 1, pass 1 and run 2, pass 2: the wick read `0 kept` under flame shapes
   added over it
4. Both runs: python or sed edits were denied by permissions
5. Run 1, pass 3: spherical glass paints made blotchy concentric rings that hid
   the candle. The session reverted them
6. Run 1, pass 4: the session reported a peach flame that the corner post hides
   in the hero. It stopped without fixing either
7. Run 2, pass 2: the nested mid and outer flame shapes overwrote each other
8. Run 2, pass 3: emission strengths of 6 to 10 washed the flame to pale pink
   and the glass to near white
9. Run 2, pass 4: the flame core read `0 exposed` inside the outer flame until a
   gradient merged the nested shapes

Lacked:

1. Both runs wanted a render flag that picks the hero angle to clear the corner
   post hiding the flame. Both accepted the occlusion instead
2. Both runs wanted the report to flag an emissive material that washes out or
   hides behind glass. Both judged the flame by eye from the renders instead

Missed in the skill:

1. Resolution rule 1 predicts both broken handles because each torus tube sat
   under the voxel
2. Resolution rule 3 predicts both runs' buried wick and run 2's buried flame
   cores. The burials cost run 2 two passes
3. Run 1 reported its flame washing to peach at `emissiveStrength` 3 to 5. It
   never lowered the emission that the materials section exposes
4. Run 2 went from pass 2's report straight to an edit without looking at the
   renders

Colors: Both runs render the glass as one flat cream-peach wash between flat
single-shade iron plates. Run 2's flame shades from white to orange against the
wash that swallows run 1's pale peach flame.

### 3. Chest

| Run | Verdict | Passes | Failed | Minutes | Voxels           |
| --- | ------- | ------ | ------ | ------- | ---------------- |
| 1   | good    | 5      | 2      | 2.1     | 15,943 at 2.5 cm |
| 2   | fair    | 3      | 0      | 1.6     | 15,425 at 2.5 cm |

Both runs built nearly the same open chest with a 110 degree barrel lid over a
velvet-lined coin mound. Run 1's custom warm gold and coated gems read far more
like treasure than run 2's olive library gold and near-black walnut.

Failures:

1. Run 1, pass 2: a python edit of the model was denied by permissions
2. Run 1, pass 3: `shades shade 0 must be within [0, 1] in linear light` for a
   custom gold `#F5C02E`
3. Run 1, pass 4: `shades shade 2 must be within [0, 1]` for `#F2C14E`.
   `#E0AE3C` fit on the next try
4. Run 1, final pass: lid straps subtracted only to the inner radius cross the
   velvet lining as iron bars that the session never noticed
5. Run 2, passes 1 and 2: gems placed with `set()` at guessed heights floated as
   one-voxel pieces until pass 3 painted columns into the pile
6. Run 2, final pass: the session called the model done without fixing the olive
   gold that the prompt centers on

Lacked:

1. Run 1 wanted the `shades` range error to report the offending shade in the
   sRGB hex that `material` takes rather than in linear light. It guessed the
   base color twice instead

Missed in the skill:

1. Both runs built coins as cylinders of 1.5 to 1.6 voxel radius. The Resolution
   section warns that size reads as blocks or plus signs
2. Both runs first placed gems with `set()` at guessed heights that left them
   floating. `coat` with sides `+y` puts them on the pile directly
3. Run 1 guessed gold hex values against the `shades` range error instead of
   lowering the spread or count that the skill lists as the knobs of `shades`
4. Run 2 let the olive library gold and near-black walnut pass the hero check on
   whether materials read right. A custom material would have fixed the gold

Colors: Run 2's library gold coins and both runs' library brass render khaki
olive against run 1's warm yellow custom gold. Run 1's lid end panels and run
2's walnut sides go dark.

### 4. Sword

| Run | Verdict | Passes | Failed | Minutes | Voxels           |
| --- | ------- | ------ | ------ | ------- | ---------------- |
| 1   | fair    | 2      | 2      | 1.6     | 880 at 2 cm      |
| 2   | good    | 3      | 3      | 1.7     | 1,212 at 1.25 cm |

Both runs drafted nearly the same point-up sword with a plus-shaped ruby before
fighting the library gold's olive look through `shades` range errors. Run 2
reads better because its drop to metallic 0.35 turned the hilt gold where run
1's hilt stayed olive.

Failures:

1. Run 1, pass 2 and run 2, pass 5: edits chained to the pass line were denied
   by permissions
2. Run 1, pass 3 and run 2, pass 3:
   `sword/grip: shades shade 0 must be within [0, 1] in linear light` for golds
   `#E8B530` and `#D9A93A` because their blue fell below zero
3. Run 1, pass 4:
   `sword/guard center: shades shade 2 must be within [0, 1] in linear light`
   for `#E6B44A`. The run settled on `#D9A63E`
4. Run 1, final pass: the session called the gold fixed although the hero still
   reads olive
5. Run 2, passes 2 and 6:
   `sword/guard trim: shades shade 2 must be within [0, 1] in linear light` for
   `#F2C14E` and `#E6B04A`. The run settled on `#DCA848`

Lacked:

1. Both runs wanted a preview of a material's rendered color. Both judged the
   gold through full passes that cost two and three `shades` range errors

Missed in the skill:

1. Both runs built the gem as a plus of boxes although SKILL.md gives
   `octahedron` a cut-gem silhouette at small sizes
2. Neither run used `material()` to deepen the ruby that transmission turns pink
   in the front view
3. Run 1 judged its final gold fixed without the hero check's material
   comparison. That gold still matches pass 1's olive
4. Run 2's taper keeps a darker `steelMid` because the `edges` paint stops at
   0.6625 m. Extending the paint or coating the taper would keep the blade one
   tone

Colors: Both runs show a blade of blue steel rather than silver and a ruby that
reads pale pink in the front view. Run 1's hilt also reads olive in every view.

### 5. Tree

| Run | Verdict | Passes | Failed | Minutes | Voxels          |
| --- | ------- | ------ | ------ | ------- | --------------- |
| 1   | good    | 5      | 0      | 1.5     | 14,089 at 0.2 m |
| 2   | fair    | 6      | 1      | 1.4     | 50,376 at 0.1 m |

Both runs hung displaced ellipsoid leaf clusters on `roundCone` limbs before
chasing the floating leaf voxels that `displace` left. Run 1's tall dome on
kinked limbs over a root flare reads more like an oak than run 2's flat umbrella
canopy on straight struts.

Failures:

1. Run 1, passes 1 and 2, and run 2, pass 2: the crown's `displace` left 9, 14,
   and 27 pieces of floating leaf voxels
2. Run 1, passes 4 and 5: the last floating leaf voxel came out through a
   one-cell carve box hard-coded to its report coordinates. Any crown edit
   breaks the fix
3. Run 2, pass 1: `shades shade 0 must be within [0, 1] in linear light` because
   five leaf shades at spread 0.07 went past black. Spread 0.04 fit
4. Run 2, pass 5: an edit chained to the pass line was denied by permissions.
   The reseed traded one floating voxel for a six-voxel floater
5. Run 2, later passes: revisions flattened the crown into an umbrella on
   straight limbs that reads less like an oak

Lacked:

1. Both runs wanted a step or voxelize option that drops floating islands or
   keeps the largest piece. Run 1 carved the last floater by coordinates where
   run 2 bridged it with a leaf clump

Missed in the skill:

1. Both runs skipped the block-out step. Run 1 wrote detailed SDF shapes from
   its first pass
2. Both runs judged some passes from the report alone: run 1 passes 2 and 4, run
   2 passes 4 and 6
3. Run 2 built straight `roundCone` limb segments where `bend` or extra elbows
   would give gnarled oak limbs
4. Run 2 ran 77 voxels across against the skill's 16 to 64 with no stated reason

Colors: Both runs render the limbs and upper trunk near-black with no bark grain
visible. Run 1's moss also reads as flat green blocks on the roots.

### 6. Well

| Run | Verdict | Passes | Failed | Minutes | Voxels         |
| --- | ------- | ------ | ------ | ------- | -------------- |
| 1   | good    | 3      | 0      | 1.5     | 9,001 at 5 cm  |
| 2   | good    | 4      | 0      | 1.9     | 14,720 at 5 cm |

Both runs built the same well plan at 5 cm per voxel with a ring wall, a gabled
roof, a windlass, and a banded bucket on a one-voxel rope. Despite a hero that
hides the bucket under the roof, run 2's plinth, sandstone cap, barge boards,
and arched bail make a better well than run 1's plainer, darker one.

Failures:

1. Run 1, pass 2 and run 2, pass 1: edits chained to the pass line were denied
   by permissions
2. Run 1, pass 2: the enlarged bail came out as 4 pieces because its one-voxel
   arc stroke missed cells. Three boxes replaced it
3. Run 2, pass 1: the bucket broke into 5 extra pieces because its one-voxel
   cone wall and 0.03 m torus bail missed cells. The rope kept 0 of the 1 cell
   it wrote
4. Run 2, pass 3: smaller stones let the dark mortar swamp the wall. Undoing it
   took another pass

Lacked:

None

Missed in the skill:

1. Run 2 accepted a hero where the roof hides the bucket and rope. The skill's
   hero check asks whether the model reads as the prompt
2. Run 2 viewed only the hero in its last two passes. Its final message still
   says every render looks right
3. Run 1 judged its last pass on the hero, front, and top renders without
   viewing the right render after pass 1
4. Run 1's one-voxel arc bail and run 2's sub-voxel torus bail both missed
   cells. The skill gives handles a torus cut with `from` and `to` or a polyline
   sized to whole cells

Colors: Both runs share a flat dark olive rope coil and stone with a cold bluish
cast. Run 1's mortar and run 2's mahogany windlass both go nearly black.

### 7. Robot

| Run | Verdict | Passes | Failed | Minutes | Voxels          |
| --- | ------- | ------ | ------ | ------- | --------------- |
| 1   | good    | 3      | 1      | 1.3     | 4,812 at 2.5 cm |
| 2   | good    | 4      | 2      | 1.3     | 4,762 at 2.5 cm |

Both runs built a boxy robot with the head and each arm as a part on a neck or
shoulder pivot. Run 2 reads better with an elbow, forearm, claws and a backpack
against run 1's flat slab arms.

Failures:

1. Run 1, pass 1: `TypeError: paint is not a function` because a material const
   `paint` shadowed the paint step. Five Edit calls renamed it `blue`
2. Run 2, pass 1: the antenna tip failed
   `box round must be at most half the shortest side` with
   `0.02499999999999991, not 0.025`. A round of exactly half the 0.05 height
   failed by float error
3. Run 2, pass 2: `scale factor must be above zero on each axis, not [-1, 1, 1]`
   from `.scale([s, 1, 1])` flipping the right arm. A helper that swaps the x
   corners replaced it
4. Run 2, pass 3: a torso `coat` without `within` recolored the legs and feet
   red. Pass 4 fixed it after the hero showed it
5. Run 1, passes 2 and 3: a sed rename and a python3 walk of the voxj hierarchy
   were denied by permissions

Lacked:

1. Both runs: a pose flag on `vxl object render` or a rotation on `part` to show
   a joint turned. Both shipped with only the rest pose checked
2. Run 1: each part's pivot in the report beside its bounds. The session tried
   to dig the pivots out of the voxj by hand
3. Run 2: a one-sided reflection for the right arm. The session swapped the x
   corners by hand because `mirror` keeps the original beside its copy
4. Run 2: a float tolerance on the `round` check. The session dropped the
   antenna round from 0.025 to 0.02

Missed in the skill:

1. Both runs: the final pass looked only at the hero though the checks call for
   all four PNGs. Neither run opened the top view
2. Both runs: the neck cylinder at r = 2 voxels falls under the skill's warning
   for radii below about 3 voxels. Run 1's antenna bulb sphere at r = 1.6 voxels
   does too
3. Run 1: the report showed `add torso 2240 cells 0 kept` after a full-box paint
   recolored every cell. The skill asks for a fix to any 0-kept step

Colors: Both heads and run 2's torso front read as one flat color on their large
faces. Run 1's glow eyes and both antenna tips read white or pink rather than
glowing.

### 8. Cottage

| Run | Verdict | Passes | Failed | Minutes | Voxels          |
| --- | ------- | ------ | ------ | ------- | --------------- |
| 1   | good    | 3      | 0      | 2.8     | 128,331 at 5 cm |
| 2   | good    | 5      | 0      | 3.1     | 63,322 at 0.1 m |

In both runs a lathed stem carries a spotted red dome raised in revision to
clear the round door and windows. Run 1 adds finer 5 cm detail with planters, a
lantern and door straps, but run 2 reads better because its windows and cap
dormer show in the hero.

Failures:

1. Run 1, pass 2: the added lantern stood in front of the door in the hero.
   Moving it left of the path took another pass
2. Run 2, before pass 1: the first file called a nonexistent method in
   `cylinder(...).intersect?.(...) ?? intersect(...)`. The session rewrote it
   before running
3. Run 2, pass 2: the report showed 3 pieces from two floating window-frame
   voxels. Only pass 4 cleared them
4. Run 2, pass 4: the dormer's red hood covered the top half of its round
   window. A fifth pass hollowed the hood into a rim
5. Both runs, pass 2: python3 heredoc edits were denied by permissions

Lacked:

None

Missed in the skill:

1. Both runs: the first file held the full detailed model with no box block-out
2. Run 1: the final pass reviewed only the hero. The lantern overlapping the
   left window in the front view went unseen
3. Run 2: the final pass reviewed only the front and hero. The boxy dormer bump
   in the right and top views went unseen
4. Run 2: passes 4 and 5 grepped the report down to a few lines

Colors: The cap's shadow turns the upper half of run 1's stem blue-gray and run
2's flat gray. Run 1's slate doorstep reads as a flat dark slab.

### 9. Potion

| Run | Verdict | Passes | Failed | Minutes | Voxels         |
| --- | ------- | ------ | ------ | ------- | -------------- |
| 1   | fair    | 3      | 1      | 1.5     | 10,388 at 5 mm |
| 2   | good    | 3      | 1      | 1.2     | 7,056 at 5 mm  |

Run 2 reads better because its tinted flask shows the shell and neck around a
clean half fill. Run 1's faint glass leaves its cork floating over an over-full
potion.

Failures:

1. Run 1, pass 1 and run 2, pass 3:
   `shades shade 0 must be within [0, 1] in linear light, not ...` on the
   darkened reds #C8102E and #A8001C. Both swapped in three hand-picked reds
2. Run 1, pass 4: the lip paint box also recolored the cork plug and cap. Pass 5
   moved the paint steps ahead of the cork
3. Run 1, final message: the session claims the glass, potion and cork read
   clearly in the angled view. The hero shows a faint neck and a cork that looks
   detached
4. Run 2, pass 4: a `coat` with only a halfSpace `within` painted 836 cells,
   glass shoulders included. Intersecting with `inside` fixed it
5. Run 2, final message: the session calls the potion dark red, but the front
   and right renders show a desaturated mauve
6. Both runs: python heredoc edits chained to the build were denied by
   permissions

Lacked:

1. Both runs: a warning that a darker shade of a saturated color leaves the
   gamut before it reaches black. Both hand-picked three reds instead

Missed in the skill:

1. Both runs: the last review never opened the right view. Run 2 also skipped
   the top view with its gear-notched cap
2. Run 2: the final potion color went unchecked against pass 1. More opaque
   glass and darker reds had turned it mauve
3. Run 1: check 3 flags `bubbles 5 kept 0 exposed` as buried. The bubbles stayed
   sealed inside the potion
4. Run 1: an unrequested twine wrap from `mat.rope` reads as a separate floating
   khaki ring

Colors: Run 1's potion top reads flat bubblegum pink under a near-white glass
haze. Run 2's potion reads as a flat dusty mauve block with a pink top.

### 10. Cart

| Run | Verdict | Passes | Failed | Minutes | Voxels           |
| --- | ------- | ------ | ------ | ------- | ---------------- |
| 1   | good    | 2      | 0      | 1.7     | 14,996 at 2.5 cm |
| 2   | fair    | 1      | 0      | 1.1     | 14,388 at 2.5 cm |

Both runs built nearly the same oak wagon on four 8-spoke wheels with iron
tires. Run 1 reads cleaner because its wheels sit outboard with clearance.

Failures:

1. Run 1, before pass 1: the first draft gave `mirror` the planes `"xy"` and
   `"yz"` in place of an axis. Fixing it and a convoluted stake chain took five
   edits
2. Run 1, pass 1: the hub band paint reported 0 cells on `wheel.fr` and
   `wheel.br` because the band on the -x side sat offset by `-v` in place of
   `-out*v`
3. Run 2, pass 1: side straps through the wheel tops and wheels touching the
   side walls jam every wheel. The session stopped without seeing either
4. Run 2, pass 1: the axle cylinders kept 16 of 288 cells. No axle shows between
   the wheel pairs in the front view
5. Run 1, before pass 1: a python heredoc batching the draft fixes was denied by
   permissions

Lacked:

None

Missed in the skill:

1. Run 2: the review missed the straps crossing the wheels in the right view and
   the wheels under the bed in the top view
2. Run 2: the axles and T-handle are one-voxel-radius cylinders. SKILL.md says a
   cylinder under about 3 voxels reads as a plus sign or block
3. Run 1: the first draft read `mirror` as taking planes. SKILL.md's
   `leg.mirror("xz")` example gives the reflected axes
4. Run 1: the review never opened the top render. Pass 2 checked only the hero

Colors: In the hero both runs lose their dark wood and inside walls to
near-black shadow. Run 1's warped floor grain reads as blotchy bands rather than
planks.

### 11. Guitar

| Run | Verdict | Passes | Failed | Minutes | Voxels         |
| --- | ------- | ------ | ------ | ------- | -------------- |
| 1   | good    | 3      | 1      | 3.7     | 67,646 at 4 mm |
| 2   | good    | 3      | 1      | 3.7     | 52,779 at 5 mm |

Both runs drafted the whole guitar in one file. Run 2 reads more like an
acoustic with its polygon body, shaped headstock, rosette and tortoiseshell
pickguard against run 1's snowman of two circles.

Failures:

1. Run 1, pass 2: `shades shade 2 must be within [0, 1] in linear light` because
   grain's implicit shades pushed the light spruce #E6C68A past white. The
   session darkened it to #D9B477
2. Run 2, pass 1: `shades shade 0 must be within [0, 1] in linear light` because
   the dark tortoise #4A2412 with spread 0.1 ran past black. The session
   lightened it to #5A3018 with spread 0.07
3. Both runs, pass 3: grain on axis `"y"` laid bands across the guitar though
   SKILL.md says the axis runs along the wood's length. Both flipped the grain
   to axis `"x"`
4. Run 2, pass 4: thickening the walls left a dark lower side in the right view
   unchanged. The session shipped without finding the cause
5. Both runs: python and sed edits chained to the build were denied by
   permissions

Lacked:

1. Run 1: a view into the hollow body. The session could check the two-cell
   walls and coated inside back only through the sound hole
2. Run 2: a section through the body to trace the dark lower side to lighting or
   interior paint. The session shipped it unexplained

Missed in the skill:

1. Both runs: the drafts ignored the SKILL.md note on shades past white or black
   for run 1's light spruce under grain and run 2's dark tortoise with spread
   0.1
2. Run 2: the final pass cut the report to `head -3` after the wall and
   pickguard edits. The per-step checks for 0 cells, 0 kept and 0 exposed went
   unread
3. Run 2: fret dots, tuner posts and tuner shafts are cylinders of 4 to 6 mm
   radius at 5 mm voxels. SKILL.md says a cylinder under about 3 voxels reads as
   a plus sign or block
4. Run 1: the hollow and binding take `offset(-0.008)` and `shell(v)` of a 2D
   `smoothUnion` against SKILL.md's advice for an extruded polygon. No buried
   wall showed

Colors: Both runs leave the ebony fretboard and headstock face flat near-black.
Run 2's spruce top grain reads busy and high-contrast.

Stress: Run 1 went to 4 mm to keep a 5.2 cm nut with one-voxel strings and gaps
across 11 cells. Run 2 kept 5 mm by widening the fretboard about 70 percent past
real to 6.5 to 7.5 cm. The guitars ran 260 and 222 voxels long against the
skill's 16 to 64 with no performance or report trouble. Both bodies come from 2D
booleans hollowed by a negative offset, but each top runs two voxels thick in
place of one.

### 12. Candelabra

| Run | Verdict | Passes | Failed | Minutes | Voxels        |
| --- | ------- | ------ | ------ | ------- | ------------- |
| 1   | fair    | 5      | 0      | 2.6     | 6,184 at 1 cm |
| 2   | good    | 4      | 0      | 2.6     | 5,004 at 1 cm |

Both runs built the same planar design at 1 cm: a lathed foot, a 6-cell twisted
box stem with painted edges, mirrored scroll arms and gradient flames. Run 2
reads better because its twist stays legible in the hero beside bigger glowing
flames.

Failures:

1. Run 1, pass 1: the `cores` step reported 0 cells because its ellipsoids were
   narrower than one cell. The next edit dropped it
2. Run 1, pass 4: two drips floating off the candles' missing corner cells left
   3 pieces. Pass 5 brought the model back to 1 piece
3. Run 2, pass 1: one gradient over all flames from y 0.43 to 0.62 tinted each
   flame by its height. The center flame read bluish-white beside red outer ones
4. Both runs: python and sed edits chained to the pass were denied by
   permissions

Lacked:

1. Both runs: a 2D spiral or volute primitive. Both sampled spirals into
   polyline points by hand for the scrolls
2. Run 1: a guide from `emissiveStrength` to rendered color. The session cut it
   from 6 to 2 by guesswork after the flames washed out

Missed in the skill:

1. Both runs: `mirror('x')` in place of `repeatPolar` kept every candle in one
   plane
2. Both runs: the arms extrude 2D arcs two cells deep in place of torus arcs or
   `bend`. They read as flat strips from the side
3. Run 1: a custom iron material stands in for `mat.iron` with shades. The hero
   still reads near-black and flat

Colors: The iron in both runs reads as one flat black or blue-black in the hero.
Only run 2's painted stem edges break it up.

Stress: Both 6-cell stems read as spirals in the front view only because each
run painted the twisted corners a lighter iron. Run 2's 320-degree twist stays
legible in the hero where run 1's dark iron hides its 544-degree twist. In place
of bends or torus arcs, both runs extruded 2D strokes two cells deep into curls
that read flat from the side. Neither met the off-axis repeat because
`mirror('x')` set all five candles in one row along x.

### 13. Knight

| Run | Verdict | Passes | Failed | Minutes | Voxels           |
| --- | ------- | ------ | ------ | ------- | ---------------- |
| 1   | fair    | 2      | 0      | 2.2     | 2,050 at 6.25 cm |
| 2   | good    | 2      | 0      | 2.6     | 2,062 at 6.25 cm |

Both runs built a 32-voxel knight at 6.25 cm with a sword and a kite shield. Run
2's flat palette materials and forward-facing shield make a better game sprite
than run 1's metallic armor with its shield edge-on to the front view.

Failures:

1. Run 1, before pass 1: `box max must be past min` from mirrored leg boxes
   whose x corners ran in the wrong order for `s = -1`
2. Run 1, pass 1: the shield face kept 0 cells because the rim paint covered all
   61 shield cells. The shield rendered solid yellow
3. Run 1, final: the vertical breath slits read as a second pair of eyes in the
   front view. The session called them fixed
4. Run 2, final: the crossguard sits behind the lavender skirt with about three
   voxels showing in front. The session noticed it but left it
5. Run 2, final: the kite shield's bottom point stair-steps into loose grey and
   blue voxels
6. Run 2, final: a dark purple plume-tip voxel on the front of the plume reads
   as a stray

Lacked:

None

Missed in the skill:

1. Run 1's metallic 0.6 to 0.8 on the PICO-8 materials shifts every armor and
   trim color off the palette in the renders
2. Neither run broke up its flat armor with a checker or bands of palette
   colors. The skill gives a flat surface `shades` or a pattern
3. Run 1 never looked at the top view or compared front with right. That
   comparison shows the shield vanishing edge-on from the front
4. Run 2 rendered all four views on pass 2 but reviewed only hero and front. The
   plume and the side silhouette went unchecked

Colors: Run 1's metallic sheen pulls the light grey to slate blue, the yellow to
olive, and the white to mid grey. Run 2 stays true to the palette, but its
unpatterned helm sides and pauldrons read as blank slabs in the right view.

Stress: Both runs reached exactly 32 voxels by building a 2 m knight at
`--voxel-size 0.0625`. Neither used `--resolution world-y 32` or hit its unround
voxel size. Both skipped `shades` for hand-written PICO-8 hex materials. Run 1
tried to break up the flat armor with a metallic sheen that drifts off the
palette, while run 2's true-palette grey leaves the helm sides and pauldrons as
blank slabs in the side view.

### 14. Pocket watch

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | good    | 5      | 1      | 5.0     | 124,954 at 0.5 mm |
| 2   | fair    | 4      | 1      | 3.5     | 104,930 at 0.5 mm |

Run 1 built at 0.5 mm from the start with a spread-out train in mixed metals
whose gears read best in front and hero. Run 2 gives a tidier symmetric front,
but its crowded all-gold train disappears in the hero.

Failures:

1. Run 1, pass 1: the 0.5 mm hairspring broke into 2-voxel fragments among the
   report's 11 pieces. Pass 2 widened it to 0.7 mm
2. Run 1, pass 2: the report gave 19 pieces, one per interlocked link plus the
   T-bar. The session misread this correct result as floating links
3. Run 1, pass 3: a shorter pitch with each link bearing a little past touching
   merged the chain into one solid without its interlock
4. Run 1, pass 4: `shades shade 2 must be within [0, 1] in linear light` after
   the gold base brightened to `#F2C25A`. The session toned it back to `#E6B550`
5. Run 2, pass 1: `repeatPolar count must be a whole number above zero, not 0`
   from a ratchet wheel that asked for zero spokes
6. Run 2, pass 2: at 1 mm the report gave 10 pieces with a floating bow, T-bar,
   hands, and hairspring. The gear teeth came out as mushy blocks
7. Run 2, pass 3: four single-cell `set` jewels floated as pieces until small
   cylinders replaced them
8. Run 2, final: links with a 2.4 mm inner half-length on a 5.4 mm pitch overlap
   by about 0.6 mm at every joint. The chain never formed a true interlock
9. Both runs: shell edits through python or sed were denied by permissions

Lacked:

1. Both runs wanted a report that tells interlocked shapes sharing no cells from
   parts that float. Run 1 overlapped its correct chain into one piece instead,
   while run 2 got no signal that its links had fused
2. Run 1 wanted a color check for metals in the review renders. It changed the
   gold hex twice without a way to predict the rendered hue

Missed in the skill:

1. Run 1 read its per-link pieces as a fault although each piece line gave a
   link "from chain" at about 320 voxels with overlapping bounds. That signature
   marks interlocked links
2. Run 2 kept `mat.gold` for the whole run although its renders show a dark
   olive. The skill's custom `material` covers a warmer gold
3. Run 2 spent a pass on 1 mm voxels that gave its gear teeth too few cells
   before it halved the voxel size

Colors: Even with run 1's custom `#E6B550`, every gold surface in both runs
renders an olive or khaki that flattens the side and top views into one slab.
Run 2's darker `mat.gold` merges the center wheel into the case in the hero.

Stress: Run 1 went straight to 0.5 mm for teeth about 2 cells wide, while run 2
reached 0.5 mm only after its teeth turned to mush at 1 mm. Both chains end
fused in 1 piece. Run 1 overlapped its correct per-link chain at pass 3, while
run 2 overlapped its links from the start. Run 1 fought the olive gold with a
custom `#E6B550` that still reads olive, while run 2 never mentioned the color.

### 15. Dragon

| Run | Verdict | Passes | Failed | Minutes | Voxels         |
| --- | ------- | ------ | ------ | ------- | -------------- |
| 1   | fair    | 8      | 4      | 6.8     | 48,366 at 3 cm |
| 2   | good    | 7      | 2      | 5.0     | 72,817 at 3 cm |

Both runs built the dragon from a `roundCone` spine on the pile's height
function and a flat polygon wing folded down the flank. Run 2's bulky
`smoothUnion` dragon with readable wings reads more as the prompt than run 1's
thin plain-union wyrm with crest-like wings.

Failures:

1. Run 1, pass 2: `shades shade 3 must be within [0, 1] in linear light` for a
   `#F2C14E` gold at spread 0.05
2. Run 1, pass 4: the first wing design stood up off the back like a sail. The
   session rewrote the wings
3. Run 1, pass 5: `shades shade 0` failed on a negative blue channel after the
   gold changed to `#E3AE34`
4. Run 1, pass 6: the head render failed with
   `view head's transform lacks --view-frame`. The next two camera placements
   missed or poorly framed the head
5. Run 2, pass 2: `shades shade 0 must be within [0, 1]` on a negative blue
   channel for a `#E9B83A` gold
6. Run 2, pass 3: `shades shade 3` exceeded 1 for `#E6B545` at spread 0.07
7. Run 2, pass 4: the mouth paint wrote 0 cells while the sword and horn tips
   broke into loose pieces. The next pass fixed both
8. Run 2, final: the emissive `mat.ember` nostrils read as glowing open eyes in
   the hero and top views. The session never rendered a close-up to catch it
9. Both runs: shell edits through python or sed were denied by permissions

Lacked:

1. Both runs wanted SKILL.md to document a close-up render of the head. Run 1
   took three tries to frame the head with view flags from
   `vxl object render --help`, while run 2 judged the head from the four wide
   views
2. Both runs wanted the shades error to give a base color or spread that fits.
   Each guessed gold values over two failed passes

Missed in the skill:

1. Both runs pushed a saturated gold through `shades` despite the skill's
   warning about shades past black or white. Each lost two passes to it
2. Neither run used `bend`. Both curled the body from 13 hand-placed `roundCone`
   spine nodes
3. Run 1 used `smoothUnion` only on the head. The plain `union` of spine, legs,
   and paws leaves the limbs almost invisible against the torso
4. Run 2 declared a `belly` material it never used. The underside stays the same
   red as the back

Colors: Run 1's wing membrane sits close enough to the body red to merge the
folded wings into the back. Run 2's gold reads dull olive under a gravel speckle
of `cells` gems and silver.

Stress: Both runs stood in for a skeleton with radius-tagged spine nodes placed
by a pile-height function and tapered by `roundCone` chains. Neither used `bend`
because the hand-placed nodes already made the curl. Run 1 limited `smoothUnion`
to the head, while run 2 filleted the spine, legs, and head into haunches and
shoulders. From the top both show a closed curl around the head, but from the
side run 1 reads as a low red band where run 2 shows a head, a haunch hump, and
a peaked wing tent.

### 16. Bridge

| Run | Verdict | Passes | Failed | Minutes | Voxels          |
| --- | ------- | ------ | ------ | ------- | --------------- |
| 1   | good    | 3      | 1      | 2.5     | 54,367 at 0.5 m |
| 2   | good    | 3      | 0      | 2.4     | 79,482 at 0.1 m |

Both runs strung a `polyline` cable and hangers ending at `cableY(x)` between
mirrored arched stone towers. Run 2 has the smoother cable and better material
contrast, but run 1 edges ahead on more detailed towers at a believable scale.

Failures:

1. Run 1, pass 1: `bridge/windows: arch max must be at least half the width`
   with `2, above min, not [2, 12.5]`. Voxelize never ran
2. Both runs: shell edits through python or sed were denied by permissions

Lacked:

None

Missed in the skill:

1. Neither run checked its bounds against a real suspension bridge. Run 1's main
   span runs 18 m under 19 m towers, while run 2's deck sits 0.5 m over the
   water between 5 m towers
2. Each run left its largest flat surface without `shades` or a pattern. Run 1's
   road and deck read as one dark band, while run 2's `mat.water` river turns a
   pale grey-blue band
3. Run 2's report showed the saddles at 8 of 64 cells kept after the cable step
   overwrote them. The session let it pass although the checks flag lost cells

Colors: Run 1's road, deck, hangers, and cables share one dark blue-grey above
water that reads nearly black in front. Run 2's plain water turns a pale
grey-blue band in front around rocks that render as black blobs.

Stress: Both runs extruded a `polyline` through points sampled from a
parabola-plus-sag function. Hangers ending at `cableY(x)` come from nine
hand-written positions in run 1 and a `for` loop every 0.4 m in run 2. Neither
cable split into pieces because each run stroked its cable at least 1.6 voxels
wide. Both dodged the 10:1 span with an unremarked stubby bridge about 2.5:1
that reads as a toy in the square frames.

### 17. Chess set

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | good    | 5      | 2      | 4.2     | 279,176 at 2.5 mm |
| 2   | good    | 4      | 0      | 3.9     | 220,380 at 2.5 mm |

Both runs placed all 32 pieces correctly from five lathe profiles and an
extruded knight at 2.5 mm. Run 1's reshaping passes give horse-like knights and
black pieces that keep their form, while run 2's decoration passes leave slab
knights on a black side that flattens to silhouettes.

Failures:

1. Run 1, pass 1: the pass line with `echo exit $?` was denied with
   `Part of this command (a variable) cannot be checked in advance`
2. Run 1, pass 3: a close-up with `--view-angles hero 30 20` failed with
   ``view `hero`'s transform lacks --view-frame and --view-position``. The
   session fell back to the default view with `--select`
3. Run 1, final: an edit left the header comment with a broken wrap
4. Run 2, close-up: `--view-angles hero 60 20` failed with the same `hero`
   transform error
5. Run 2, close-up: `--view-orbit` with `--view-frame close subject` failed with
   ``--view-orbit sets view `close`'s transform...``. Dropping `--view-frame`
   worked
6. Both runs: python edits were denied by permissions

Lacked:

1. Both runs wanted a SKILL.md recipe for a close-up render of one part. Run 1
   settled on `--select` with the default view after one view-flag error, while
   run 2 reached `--view-orbit` alone after two

Missed in the skill:

1. Neither final file places one part per kind and color by offset as SKILL.md's
   Parts section shows. Run 1 never shared parts, while run 2 dropped its 12
   shared parts to remove the 0-voxel wrapper lines
2. Run 1 kept `const v = 0.005` as its grid constant while voxelizing at 0.0025.
   SKILL.md's Coordinates section has `v` hold the voxel size
3. Run 2 left the knight head a sharp-edged 4-voxel extrusion. The Shape3d
   `offset` and `round` options could have thickened and rounded it

Colors: Both runs' ivory reads cool grey-white rather than warm. Run 1's black
pieces read as one dark brown with little grain, while run 2's jet hides the
lathe detail in near-black silhouettes.

Stress: Colors stay right in both runs: run 1 built 32 separate square parts
from each side's material, while run 2 placed 12 kind-by-side parts by offset
before flattening them to 32. Both placed five lathed kinds and an extruded
side-profile knight in a correct R N B Q K B N R order with queens on their
color. White pieces stay distinct on light squares in both runs because the
ivory reads cool grey-white against yellow maple.

### 18. Ramen stand

| Run | Verdict | Passes | Failed | Minutes | Voxels         |
| --- | ------- | ------ | ------ | ------- | -------------- |
| 1   | good    | 3      | 0      | 12.9    | 38,622 at 5 cm |
| 2   | good    | 3      | 0      | 2.1     | 33,260 at 5 cm |

Both runs built the RAMEN sign from the same 5x7 bitmap font on a 0.05 m grid.
Run 1's katakana blade sign, rooftop gear, and posters sell cyberpunk better
despite weak reads such as grey steam and cross-shaped stools, while run 2's
sparser stall gives a crisper sign.

Failures:

1. Run 1, pass 2: the report gave 5 pieces because the first lantern-caps paint
   spanned the whole width. A mirrored corner-only box fixed it
2. Run 1, between passes 2 and 3: a handful of small edits took about ten
   minutes of the run's 12.9
3. Run 2, pass 1: the sign read RAME:N because `set` points at left -0.725
   landed on cell boundaries. Pass 3 moved the word to -0.75
4. Run 2, pass 1: the AC unit floated as a second piece while the sign braces
   kept 0 cells. The session fixed both by moving the AC and deleting the braces
5. Both runs: shell edits through python or sed were denied by permissions

Lacked:

1. Both runs wanted a text or lettering operation. Run 1 hand-wrote 5x7 Latin
   and 5x5 katakana bitmaps with a `px()` helper that emits a cell-sized box per
   lit pixel, while run 2 placed one `set()` point per lit cell
2. Both runs wanted render control over background, exposure, or bloom. Each
   lowered `emissiveStrength` instead because the neon washed pastel on the
   white review background

Missed in the skill:

1. Neither run reviewed all four views. Run 1 opened right once and top never,
   while run 2 opened only hero and front
2. Run 1 made the stool seats 0.16 m cylinders of about 3 voxels. They render as
   the plus signs the Resolution section warns of
3. Run 2's first `set()` points on cell boundaries gave the RAME:N stray cell.
   The Resolution section puts thin details on whole cells at multiples of `v`
4. Run 1's steam takes an alpha `baseColor` without transmission. It renders as
   opaque grey columns

Colors: The pink neon reads pastel in both runs. Run 1 washes its yellow border
to cream and its red antenna light to pink over near-uniform navy walls, while
run 2's walls, posts, and counter merge into one near-black mass.

Stress: Both runs built RAMEN from one-cell-wide 5x7 bitmap strokes. Both signs
read left to right in the front view, but run 1's hero smears the N toward an H
through the letters' depth faces. Run 2's four glow colors stay distinct, while
run 1's yellow and red glows collapse to cream and pink.

### 19. Dungeon kit

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | good    | 3      | 0      | 2.9     | 38,400 at 0.125 m |
| 2   | good    | 2      | 0      | 4.2     | 21,132 at 0.125 m |

Run 1 built cell-filling tiles that fit the 2 m grid literally, but the walls of
its 10x8 m room read as dark speckle. Run 2's edge-and-vertex brick kit builds
the better-looking 6x6 m room despite wall overhangs and a notched doorway
crown.

Failures:

1. Run 2, before pass 1: `vxl --version` errored with an unexpected argument
2. Both runs, pass 1: reading the arch's max corner as the spring line left a
   squat doorway that opened only 1.75 m. Pass 2 fixed it
3. Both runs, after the last pass: permissions denied every palette inspection
   but one grep in run 1. That grep found the `walkable` property name but no
   value
4. Run 1, final message: the session claims 9 distinct voxel objects for the
   room's 20 cells. Voxelized under the default `--frame world`, the room holds
   20 objects in `vox-doc show`

Lacked:

1. Placing a part rotated: `part()` takes only a pivot and an offset. Run 1
   baked rotated copies in a cached `tile(kind, rot)` factory, while run 2
   rotated every shape in `makeWall(90)` and `makeDoorway(90)`
2. A check of the walkable values: neither the report, the renders, nor any
   command in `SKILL.md` shows them. Python and grep fallbacks gave neither run
   a value

Missed in the skill:

1. Neither run voxelized the room with the `--frame local` that `SKILL.md`
   documents for sharing one object across a part's placements. Run 1 claimed
   shared objects anyway, while run 2 recommended the flag but shipped 25
   separate objects

Colors: Run 1's walls read as dark blue-black speckle with almost no masonry
between flat gray caps, plinths, and pillars. Run 2's brick and floor shades
vary well, but its coping, capitals, and pillar drums stay a flat beige-gray.

Stress: `vxl vox-doc show` lists only the `walkable` property name, but a
read-only parse of the value pools confirms both runs set it true only on the
floor shades, floor mortar, and threshold or sill. Both `room.ts` files import
`kit.ts` by relative path rather than through `lib.parts`. Both sessions read
their kit sheets' 4 pieces as separate tiles rather than floating geometry.

### 20. Crane

| Run | Verdict | Passes | Failed | Minutes | Voxels          |
| --- | ------- | ------ | ------ | ------- | --------------- |
| 1   | good    | 3      | 0      | 3.5     | 4,328 at 0.25 m |
| 2   | fair    | 2      | 0      | 2.7     | 4,578 at 0.25 m |

Both runs built a crawler crane whose upper works turn at the slewing ring and
whose boom tilts at its foot pin. Run 2 has the more distinct cab and twin
falls, but its boom stands unsupported without the pendant rope from the A-frame
to the boom head that makes run 1 the more convincing crane.

Failures:

1. Both runs, pass 1: `shades()` of the saturated yellow failed the build with
   `shades shade 0 must be within [0, 1] in linear light`. The runs replaced it
   with hand-picked hex materials (two in run 1, three in run 2)
2. Both runs, first voxelized pass: thin boom members shattered into 17 pieces
   in run 1 and 28 in run 2 once tilted 50 degrees. Run 2 also had a hook that
   missed its shank and rollers of only 2 cells
3. Both runs: a python heredoc batch edit was denied by permissions

Lacked:

1. A rope spanning two parts: the pendant runs from the upper works' A-frame to
   the boom head. Run 1 kept it in the boom part with a warning that it drifts
   off the A-frame about 0.5 m per 10 degrees of tilt, while run 2 left it out
2. A joint that stays plumb under its parent's turn: the hook inherits the boom
   tilt. Both runs told the user to counter-rotate the hook node by hand
3. Each part's pivot in the report: no line gives one. Neither run could check
   the hook pivot against the head sheave without it

Missed in the skill:

1. Both first booms shattered under the tilt because their members ran 1 to 1.2
   voxels thick. Resolution item 1 warns that shapes under a voxel miss cell
   centers
2. Neither run opened the top render or acted on the edge-on hook in the right
   view
3. The checks' hero step asks whether the model reads as the prompt. Run 2
   weighed its missing boom rigging only against the joint, never against the
   renders

Colors: Run 1's cab blends into a house and boom of the same yellow above a flat
near-black undercarriage and deck. Run 2's `#6B6F75` cable at metallic 0.8
renders as near-black as its cab roof, counterweight, and undercarriage.

Stress: Both runs nest four levels deep from the root to the hook without any
`offset`. The voxj nodes put each run's upper works pivot on the slewing ring
and its boom pivot exactly on the foot pin. Run 2's hook pivot sits exactly on
the head sheave center at fractional translations, while run 1's sits about
0.52 m from the sheave center.

### 21. Fish tank

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | fair    | 4      | 0      | 3.6     | 34,092 at 1.25 cm |
| 2   | good    | 3      | 0      | 3.0     | 20,793 at 1 cm    |

Run 1 filled its tank with a faint custom water that barely shows in the hero
yet darkens the castle in the orthographic views. Run 2's dry tank of four
brighter fish and a clearer castle reads more like the prompt.

Failures:

1. Both runs: permissions denied python heredoc edits and a run 1 sed edit
   chained to the voxelize line
2. Run 1, pass 2: the clown stripes covered all but 6 of the body's 58 cells.
   Pass 3 rewrote the stripe boxes
3. Run 2, pass 1: fins added after the bodies covered them. Pass 2 fixed the fin
   order
4. Run 2, pass 1: the flag floated above the center roof among 40 pieces. Pass 2
   fixed it
5. Run 2, passes 2 and 3: the session filtered the report down to piece lines
   and then to its first line. The final step lines went unchecked

Lacked:

1. A report that sees inside water: every interior step in run 1 reads
   `0 exposed` within a one-piece model. The session said in its final message
   that the report could not show floating or buried details

Missed in the skill:

1. Both runs' bubbles render as pale specks because their sphere radii run about
   one voxel or less. The Resolution section warns that spheres that small read
   as plus signs or blocks
2. Run 1's custom water `#4AA8E030` adds no blue to the hero. The library
   `water` or a stronger tint would have shown it
3. Run 2 left out water on the claim that it would cover the fish. The skill
   offers `#RRGGBBAA` alpha on `baseColor` and a transmissive library `water`

Colors: Neither tank interior shows a blue tint in the hero. Run 1's castle and
seaweed turn dark blue-gray behind the water in the front and right views, while
run 2's pale blue bubbles wash out against its blank white interior.

Stress: Run 1's water took its castle, gravel, seaweed, and bubble steps to
`0 exposed` while the fish parts still counted exposed cells. The same water
joined its bubbles into one piece, whereas run 2's dry tank kept its bubbles as
floating pieces that the session reported as intended. Both glasses use
`#RRGGBBAA` alpha (`#CFEFFF40` and `#CDEBFF30`), but run 2 dropped the alpha
from its bubbles.

### 22. Wizard tower

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | good    | 5      | 1      | 3.1     | 17,888 at 0.2 m   |
| 2   | good    | 7      | 1      | 4.2     | 64,150 at 0.125 m |

Run 1 has the more characterful drooping hat roof, but at 0.2 m its stairs read
as a ramp under a heavy rail. Run 2's legible stairs, balcony, upper room,
balustrade, and path at 0.125 m make it the more complete tower despite a lumpy
roof.

Failures:

1. Both runs, pass 1: thin rails split the model into 65 pieces in run 1 and 16
   in run 2 (all or nearly all `from rail`). The fix thickened the rails to 0.3
   m by 0.4 m in run 1 and 0.1 m in run 2
2. Both runs: python edits chained to the pass line were denied by permissions
3. Run 1, failed pass: a double-bend roof 7.79 long stopped the build with
   `bend shape must be at most 6.91 long along y to bend within half a turn...`
4. Run 1, a later pass: a larger bend radius sank the roof into the tower at
   y=8.6. It also left an 11-voxel loose roof sliver
5. Run 2, pass 5: a misread sector angle direction left the change to the
   ledge-rail opening with no visible effect. The `/tmp` tests that followed hit
   a blocked heredoc, a denied Write, and `Cannot find module /tmp/vt/t.ts`
6. Run 2, after pass 5: the session tested sector angles with a `scratch.ts` in
   the trial folder before deleting it
7. Both runs, final message: each claims glowing amber windows that render as
   flat pale-peach panes

Lacked:

1. A helix or a sweep along a 3D path: run 1 extruded 38 ring-sector wedges
   (`sector` minus `circle`) at their own heights. Run 2 turned 34 boxes with
   `rotate('y')` under a rail of capsules between computed posts
2. A bend that droops the roof toward its tip: `bend` gave only a lean within
   its half-turn limit. Run 1 chained `roundCone` segments along a hand-placed
   spine, while run 2 took a `smoothUnion` of four cones
3. Where a bent shape lands: run 1's `bend` error gave only the maximum length.
   The roof sinking into the tower showed only after a full pass
4. Where a cut sits inside a step: the report gives only each step's bounding
   box. Run 2 built a scratch model to locate its stairwell sector

Missed in the skill:

1. Run 1's treads voxelize as a ramp because their 0.2 m rise equals the voxel
   size. The skill's advice to size details to the grid with `v` calls for a
   rise of at least 2v
2. `SKILL.md` says `extrude` on axis y maps u,v to x,z. Run 2 missed that its
   `sector` angles then run opposite to a right-hand `rotate('y')`
3. Run 1's dark-bordered `cells` wall reads as near-black streaks in every view.
   The session never revised it despite the hero check on whether a surface
   reads right
4. Run 2's `smoothUnion` radius of 0.25 left ledges at the cone joints. A single
   bent or chained `roundCone` profile would have avoided them

Colors: Both stone walls read near-black because dark borders or mortar dominate
their cells. Both runs also have flat pale-peach window panes and a plain grass
ground.

Stress: Both runs kept every tread whole by placing each at its own angle and
height instead of twisting a stack. The predicted fragments came from the thin
rails (65 pieces in run 1, 16 in run 2). For the roof, both runs dropped
`cone(...).bend` after it gave only a shark-fin lean. Run 1's chained
`roundCone` spine droops better than run 2's `smoothUnion` of leaning cones with
its joint ledges.

### 23. Log cabin

| Run | Verdict | Passes | Failed | Minutes | Voxels           |
| --- | ------- | ------ | ------ | ------- | ---------------- |
| 1   | fair    | 3      | 0      | 3.5     | 169,347 at 0.1 m |
| 2   | good    | 2      | 0      | 4.5     | 816,895 at 5 cm  |

Run 1 built a chunky cabin at 10 cm with rod icicles and a lightened smoke that
blends into the snow. Run 2's 5 cm cabin with chinking, log gables, beams, snow
lips, tinted ice, and a gray smoke reads closer to the prompt at about five
times the voxels.

Failures:

1. Run 1, pass 1: a 1-voxel smoke fragment at [1.5, 7, -0.7] made a second
   piece. Pass 2 reshaped the puffs
2. Run 1, pass 2: lightening the smoke from `#B8B8BC` to `#D4D6DB` brought it
   close to the snow color. The session never noticed
3. Run 2, pass 1: a `time ( ... )` wrapper on the pass line was denied by
   permissions
4. Run 2, pass 1: a 1-voxel snow ground fragment at [-3.75, 0, -4] made a second
   piece. Moving the `displace` onto the drifts alone fixed it

Lacked:

None

Missed in the skill:

1. Run 1's icicle cones of radius 0.12 at `v = 0.1` came out as uniform rods.
   The Resolution section warns that a radius under about 3 voxels reads as a
   plus sign or block
2. The scenes run 92 voxels across in run 1 and 183 in run 2 against the skill's
   16 to 64. Neither session weighed a coarser size
3. Both runs wrote the detailed model without the box block-out pass that the
   Building steps call for
4. Run 2 never acted on its wall core keeping 8 of 44,044 cells. That report
   line shows the step does almost nothing

Colors: Run 1's smoke shares the snow's pale blue-white range over ground snow
blotched with large flat dark-blue noise in the top view. Run 2's ground apron
outside the drifts reads as one smooth pale slab ending in a hard ledge below
roof snow speckled dark by the stepped slope.

Stress: Both runs built the main snow from a displaced roof slab and drift
shapes rather than `coat`. Neither smoke floated because both plumes start
inside the flue. Run 2's tinted custom ice reads better than run 1's `mat.ice`
in the front and right views, but the icicles nearly vanish under the eave
shadow in both heroes.

### 24. Valley

| Run | Verdict | Passes | Failed | Minutes | Voxels         |
| --- | ------- | ------ | ------ | ------- | -------------- |
| 1   | good    | 4      | 0      | 2.3     | 161,361 at 1 m |
| 2   | good    | 3      | 0      | 3.4     | 223,660 at 1 m |

Both runs built a 1 m solid diorama of displaced cone peaks with seeded pines
set on an analytic height function. Run 1's forest buries the river below squat
peaks, while run 2's taller peaks, glades, snowfields, and clearer river read as
a mountain valley.

Failures:

1. Both runs: permissions denied the `time ( ... )` wrapper on the pass line and
   a python edit in run 2
2. Run 1, passes 1 to 3: the displaced mountains and snow left 5 one-voxel
   crumbs that gave way to a 6-voxel chip at the gorge rim
3. Run 1, pass 4: a hard-coded carve box copied from the piece line removed the
   last chip. The box breaks as soon as a peak, seed, or channel moves
4. Run 2, pass 1: the river extrusion ran 7 m past the front and back edges
   because its points ran unclipped to `S + 4`
5. Run 2, pass 2: taller peaks under `displace` amplitude 3.5 left 11 floating
   crumbs of rock and snow (45 voxels)

Lacked:

1. A query for the voxelized surface height: both runs seated their pines on a
   hand-written ground function that ignores the `smoothUnion` fillet and the
   `displace` noise. Each sank its trunks 4 m to hide the error
2. A step that drops pieces under a size: run 1 carved its last chip by the
   reported bounds. Run 2 lowered the `displace` amplitude across the whole
   range
3. Which operation made a crumb: run 1's piece lines gave the step but not
   whether `displace` or a carve rim made it. The session tried octaves before
   hand-carving the chip
4. An exposed count per material or face: run 2 could not tell from the report
   that pass 1's soil coat painted every cut side face brown. It saw that only
   in the render

Missed in the skill:

1. Neither run weighed `--fill-mode surface` or a coarser size against the
   budget before shrinking the scene to 96 m and 112 m. About 85% of each run's
   terrain cells stay buried
2. Run 2 cut the river slot from y 1 to 70 along the whole path instead of
   bounding the carve with an intersection or sloping the banks with
   `smoothSubtract`
3. Run 1 declared `const v = 1` without using it. It set the crown cones on
   half-meter centers without checking the cell-center rule for one-voxel trunks

Colors: Both rivers use one flat water material. The right face reads as one
large wall of dark gray stone noise without strata in run 1 and of uniform
stripes in run 2.

Stress: Shrinking the scene to 96 m and 112 m kept both runs under budget with
solid 1 m fill (161k and 224k voxels). Displacement left a handful of crumbs
rather than dozens (at most 11, after run 2 raised its peaks). Without a surface
query, both runs set the pine crowns at guessed heights over trunks sunk 4 m.
Run 1 cut its river as a stepped V-gorge that terraces the back slopes, while
run 2's 70 m vertical slot on a flat floor leaves canal-like walls.

### 25. Village

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | good    | 5      | 2      | 9.8     | 249,365 at 0.25 m |
| 2   | fair    | 5      | 1      | 6.1     | 162,810 at 0.25 m |

Run 1 makes the better village with a richer church and ten cottages that vary
in wall color, door color, roof form and width. Run 2's six cottages read as
copies because they share one plaster material and one hipped roof.

Failures:

1. Run 1, pass 1: `shades` on a near-white daub of `#F5F2EA` failed with
   `village/cottage.e1/walls: shades shade 2 must be within [0, 1] ...`
2. Run 1, pass 3: the extra render exited 2 with
   ``view `close`'s transform lacks --view-frame``
3. Run 1, pass 3: z counts on the x slot of three `repeat` calls for fence
   posts, vegetable rows and crenels left 4 pieces
4. Run 1, pass 3: the hay ellipsoid swallowed all but 12 of the cart bed's 60
   cells
5. Run 1, pass 4: a stray thatch voxel from the roof displace made a second
   piece. Pass 5 fixed it with a different noise scale
6. Run 2, pass 1: `shades` on a saturated pumpkin orange failed with
   `village/stall.pumpkins/goods: shades shade 0 must be within [0, 1] ...`
7. Run 2, pass 3: a z count on the x slot of the garden rows' `repeat` left 16
   floating plant pieces of 4 voxels each
8. Run 2, pass 4: a tree crown voxel stayed detached. Pass 5 fixed it by
   reseeding the tree
9. Both runs, after pass 2: a python heredoc batch edit was denied by
   permissions

Lacked:

1. Both runs: `part()` offers no turn per placement for one shared cottage part.
   Each run built a fresh part per cottage from a function that rotates every
   shape
2. Both runs: a report short enough to read whole instead of run 1's 30.6 KB for
   30 parts. Each session grepped a saved copy for empty steps and pieces

Missed in the skill:

1. Both runs: SKILL.md says a shade past black or white errors. Each run still
   lost pass 1 by running `shades` on a near-white or saturated color
2. Both runs: SKILL.md gives the per-axis count array of `repeat`. Each run
   still put z counts on the x slot
3. Run 2: per-cottage seeds on one shared plaster and one roof form left the
   cottages looking identical. Distinct copies also needed distinct materials or
   forms

Colors: The grass reads a saturated toy green in both runs. Other flat fields
cover the slate church roofs and grey cobbles in run 1 and the cream walls and
pale thatch in run 2.

Stress: Both runs avoided reusing a part by building a fresh part per cottage
from a function that rotates its shapes. Run 1 turned four of its ten cottages
outward or toward the churchyard where run 2 faced all six to the square. Both
runs seeded every cottage, but only run 1's varied colors and roof forms keep
its copies from reading as clones. In both runs a single bespoke church with its
tower toward the square stands clearly apart from the cottages.

### 26. City

| Run | Verdict | Passes | Failed | Minutes | Voxels           |
| --- | ------- | ------ | ------ | ------- | ---------------- |
| 1   | good    | 2      | 0      | 6.1     | 83,641 at 0.5 m  |
| 2   | good    | 3      | 0      | 4.2     | 120,741 at 0.5 m |

Run 1 built a four-way intersection with a fountain park and six well-detailed
buildings right on the first pass. Run 2's ten buildings with lit windows read
more like a city despite a beige lobby band and a tree that hides the fountain.

Failures:

1. Run 2, before pass 1: the session cleaned `.union?.(...) ?? union(...)`, a
   `.slice(0, 0)` tree and a `window` binding out of its first draft
2. Run 2, pass 1: `cells()` lit random half-windows that mixed lit and dark
   cells inside one opening. Per-opening boxes with a lit flag replaced them
3. Run 2, passes 1 and 2: a floating voxel of the tree 3 crown at [8, 3, 18.5]
   and then [10.5, 3, 18.5] made a second piece

Lacked:

1. Run 2: a piece 1 line that skips its roughly 150 steps when that piece holds
   the whole model. The session grepped the report down to its model, piece and
   tree lines instead

Missed in the skill:

1. Both runs: the skill's checks call for all four PNGs each pass. Run 1's pass
   2 and run 2's passes 2 and 3 read at most the hero
2. Run 2: the floating crown voxel took displace seeds 3, 6 and 9 to clear
   instead of a lower displace amplitude or a step that ties the crown to the
   trunk
3. Run 1: pass 1 read the full 98-line report in one go. Only pass 2 filtered it
   with grep

Colors: Both runs leave every roof and every glass tower in flat unshaded color.
Run 2's green tower also carries a flat pale yellow lobby band.

Stress: At 0.5 m voxels over a scene shrunk to 40 m or 48 m, both runs drew
1 x 2 m windows as a clean 2 x 4 voxels. The top and hero views carried the
streets in both runs. After first reports of 98 and 160 lines, both grepped
their later reports. With no parts in either run, the default output held one
object without `--flatten objects`.

### 27. Dollhouse

| Run | Verdict | Passes | Failed | Minutes | Voxels          |
| --- | ------- | ------ | ------ | ------- | --------------- |
| 1   | fair    | 3      | 0      | 6.9     | 94,630 at 0.1 m |
| 2   | good    | 2      | 0      | 7.1     | 756,699 at 5 cm |

Both runs left the front wall off a two-story house that stacks a bedroom and
bathroom over a living room and kitchen under an attic. Run 2's 5 cm grid,
patterned wallpaper and extra furniture read much better than run 1's 0.1 m
rooms despite a misplaced chimney.

Failures:

1. Run 1, pass 1: floating details made 9 pieces. Pass 2 brought it to 1 piece
2. Run 2, pass 1: a floating teddy and attic bulb made 8 pieces. Pass 2 brought
   it to 1 piece
3. Run 2, pass 2: moving the chimney from x 0.2 to x 1.0 left a brick column in
   the open attic with no tie to the fireplace
4. Both runs: a python3 heredoc edit chained to the build was denied by
   permissions

Lacked:

None

Missed in the skill:

1. Both runs: the skill gives a large flat surface `shades` or a pattern. Run
   1's wallpapers and both lawns still stay single flat colors
2. Run 1: the skill's checks flag the `0 kept` that the back, side and partition
   walls report in the final pass. The session grepped past it with paints
   covering those walls on purpose
3. Run 1: each chair, table and sofa takes four hand-written leg boxes where
   `mirror("xz")` from the skill builds four legs from one

Colors: The lawn reads one flat green in both runs. Other flat fields cover
run 1's dark room walls and run 2's grey stair side and tub.

Stress: Both runs give a clear control for 28 to 30 by leaving the front wall
out and trimming the cut edges white. Run 1's 0.1 m voxels make chairs a blocky
but legible 4 voxels wide. Run 2 spends 757k voxels at 5 cm to keep knobs and
candle flames visible and chairs 8 voxels wide. Both houses run past the skill's
guide of 16 to 64 voxels across without strain.

### 28. Living room

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | good    | 4      | 1      | 4.2     | 43,645 at 5 cm    |
| 2   | good    | 3      | 0      | 5.1     | 309,418 at 2.5 cm |

Both runs built a two-walled corner with the fireplace on the back wall and the
bookshelf on the left. Run 2's better fire, fuller bookshelf and fringed rug at
2.5 cm make a cozier room than run 1's chunky 5 cm one despite a ribbed
135-degree chair that hides the lower shelves.

Failures:

1. Run 1, pass 2: a short firebox arch failed with
   `living-room/firebox: arch max must be at least half the width, 0.4, ...`
   until the session raised its top to 0.85
2. Run 1, pass 3: the chair turned 45 degrees rendered as a jagged striped
   block. Squaring it to 90 degrees fixed it
3. Run 1, pass 3: the flames read salmon-pink until a later pass
4. Run 2, pass 2: emission of 3 to 5 turned the fire pinkish. Pass 3 lowered it
   to 1 to 1.5
5. Run 2, pass 3: the session saw ribbing from the chair's 135-degree turn but
   shipped it with only a note in the final message
6. Both runs: sed edits were denied by permissions

Lacked:

1. Run 2: a hero camera from another angle because the review hero shows the
   armchair's back. The session accepted the view with a note in the final
   message

Missed in the skill:

1. Run 1: SKILL.md says an arch tops its rectangle with a half circle as wide as
   the rectangle. The session still wrote a rectangle shorter than half its
   width and lost pass 2
2. Run 1: Resolution rule 2 says rounds under about 3 voxels read as blocks. The
   lamp shade's shelled cone and the firewood cylinders of 1.5-voxel radius
   still came out lumpy
3. Run 1: the Checks section gives a large flat surface `shades` or a pattern.
   The plaster walls still take one flat material
4. Run 2: 2.5 cm voxels put the room 140 voxels across against the skill's guide
   of 16 to 64. The book and fire detail paid off at a cost of 309k voxels

Colors: Run 1 leaves the upper walls flat dark green, the chair one orange and
the pillow a plain grey block. Run 2's chair velvet of two greens reads through
rotation ribbing rather than shading.

Stress: With only two walls built, the front and right views in both runs see
the fireplace and bookshelf face-on without a custom camera. Run 2's armchair
still blocks the lower shelves in the hero and right views. Both drew book
colors from a seeded RNG over one step per color. Run 2's spines read more mixed
because it also varied book width, gaps and flat stacks.

### 29. Spaceship

| Run | Verdict | Passes | Failed | Minutes | Voxels          |
| --- | ------- | ------ | ------ | ------- | --------------- |
| 1   | good    | 4      | 0      | 3.2     | 169,018 at 5 cm |
| 2   | fair    | 3      | 0      | 3.2     | 73,440 at 5 cm  |

Both runs built an open-front, open-top bridge at 5 cm with the chair facing a
radar and a big back window. Run 1 reads better because its custom bloom-lit
interior view shows radar rings and blips that run 2's tilted radar never forms.

Failures:

1. Run 1, pass 1: the starfield backdrop floated as a second piece that widened
   the bounds to 136 x 80 x 136 voxels
2. Run 1, pass 1: the radar sat inside its console with only 43 exposed cells
3. Run 1, pass 4: a ceiling beam still hides the radar in the hero. The session
   noted it without a fix in the standard views
4. Run 2, after pass 1: an Edit failed with `String to replace not found`
   because an earlier rename left `sideConsole,hullDark` without a space
5. Run 2, pass 3: the session noted unreadable radar rings and a planet hidden
   in the hero but fixed neither
6. Both runs: sed and python3 edits were denied by permissions

Lacked:

1. Both runs: a review profile with bloom and an interior camera. Run 1 placed
   its own view with `--bloom-strength 1` from `vxl object render --help`, but
   run 2 judged the radar and planet from the outside views alone

Missed in the skill:

1. Both runs: the build passed `--library materials` but wrote every emissive
   material by hand instead of taking `glow` from the Light group of `mat`
2. Run 1: a speckle over the whole star box puts stars on its outside faces. A
   paint or a `coat` with `within` confined to the cavity keeps the outside
   plain hull
3. Run 2: the radar stayed about 22 voxels across. Rings need a larger or
   flatter dish or one `arc` per ring sized to whole cells

Colors: Without bloom the screens in both runs read as flat blue slabs over wide
flat wall bands of navy or grey. Run 2's radar rings break into blotchy
mid-green patches.

Stress: Both runs left out the front wall and ceiling, but only run 2's sloped
side walls clear the hero for the radar. Run 1's star box joined to the back
wall makes one piece but pushes the bounds 1.9 m behind the hull. Run 2's slab
of about 250 stars 0.65 m behind the window held one piece in 4 x 2.8 x 4.2 m
bounds from pass 1. Of the two radars built from circle shells and painted blips
on a tilted disc, only run 1's enlarged disc reads as rings.

### 30. Pirate ship

| Run | Verdict | Passes | Failed | Minutes | Voxels         |
| --- | ------- | ------ | ------ | ------- | -------------- |
| 1   | fair    | 3      | 0      | 3.9     | 76,439 at 4 cm |
| 2   | good    | 4      | 0      | 4.5     | 56,638 at 5 cm |

Both runs built a cutaway gun deck with four cannons, two hammocks, barrels and
an emissive lantern. Run 2's flared polygon hull and curved ribs read more like
a ship below deck than run 1's box walls despite muzzles that stay inside their
ports.

Failures:

1. Run 1, pass 1: four gun barrels floating above their carriages and 0.03 m
   breeching-rope capsules breaking up made 13 pieces. Pass 2 fixed both with
   cheeks raised to 0.68 and ropes thickened to 0.045 m
2. Run 1, pass 3: the lantern sits under the center beam where the hero barely
   shows it. The session admitted it but left it
3. Run 1, pass 3: the left hull wall and the beams hide the aft barrels in the
   hero
4. Run 2, pass 1: one-cell arc hammocks breaking into diagonal fragments and
   cascabel knobs floating off all four guns made 19 pieces
5. Run 2, pass 2: hammock ropes that stopped at the beam underside (y = 2.15)
   and a floating top shot on each pile made 5 pieces
6. Run 2, pass 4: the guns never run out because their muzzles end at x = 2.05
   inside a hull whose outer face sits at 2.2
7. Run 2, pass 4: the aft deck planking hides the barrels and crate in the hero.
   The session noted it but left it
8. Both runs: a python heredoc edit chained to the pass was denied by
   permissions

Lacked:

None

Missed in the skill:

1. Both runs: SKILL.md warns that `ellipsoid(...).shell(v)` comes out thicker
   than asked. Hammocks built with it still read as rigid trays rather than a
   one-cell sling from a bent box or a subtract of two lathes
2. Run 1: SKILL.md points hollow forms to an extruded polygon. The session's
   flat box walls give no hull shape

Colors: Both runs render the cannons navy instead of black iron and the lantern
as a pale peach square. Run 1's hull walls also stay plain brown bands that go
muddy toward the far end.

Stress: Neither hull left inner walls because run 1 used flat boxes and run 2
subtracted two extruded polygons. Run 2's one-cell arcs broke into six fragments
in pass 1 before it switched to the ellipsoid-shell hammocks that read as rigid
bowls in both runs. Run 1's guns run well out through carved ports, but run 2's
muzzles stop inside the wall. The emissive lantern lights nothing under the
review lighting in either run.
