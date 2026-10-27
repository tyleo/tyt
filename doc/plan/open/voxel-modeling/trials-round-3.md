# Trials, round 3

Step S14 of the [checklist](checklist.md) runs the [prompts](trials.md#prompts)
a third time after round 2's fixes. [Round 2](trials-round-2.md) logs the round
before, and the [trial
harness](../../../../projects/vxl/trials/README.md) ran all three.

## Changes before round 3

The owner judges the model a session makes, not the process it follows, and
asked for a shorter skill. The changes before round 3 follow from that:

1. Trial sessions run in auto mode, where Claude Code approves the commands a
   classifier judges safe
2. The skill drops its process mandates, its tool rules, and the `.vxlconfig`
   reference. Its workflow runs half as long
3. The voxelize report counts the voxels `--fill-mode surface` leaves
4. `vxl object render --to png` prints each PNG path it writes
5. The `review` profile renders on opaque white
6. Lighter `shades` keep the base's hue and chroma
7. The harness starts a new pass only after a voxelize, and the analyses judge
   the model over the process

## Findings

Round 3's analyses recorded 486 items across the 60 runs, against round 2's 672.
Every finding keeps its round 2 number, and new findings start at 85. A skipped
step now counts only where the model or the session's time suffered. Process
findings fell partly for that reason. The round's gallery lists each finding's
items beside the renders.

### What the changes did

The table counts the prompts that showed each finding the changes answered.

| #   | Finding                                       | Round 2 | Round 3 |
| --- | --------------------------------------------- | ------- | ------- |
| 8   | Renders skipped                               | 22      | 2       |
| 1   | Edits met the permission check                | 19      | 0       |
| 28  | Skipped the box block-out                     | 15      | 0       |
| 66  | Sessions guessed render file names            | 3       | 0       |
| 52  | A scratch model for one shape                 | 2       | 0       |
| 67  | Large scenes went without close-ups           | 2       | 0       |
| 68  | Lightened gold shades read beige              | 2       | 1       |
| 70  | Read-only grep met the permission check       | 2       | 0       |
| 71  | Sessions cut the report to its model line     | 2       | 5       |
| 72  | Surface fill reported the solid count         | 1       | 0       |
| 74  | Hero renders came out on black                | 1       | 1       |
| 79  | The sandbox blocked listing the trials folder | 1       | 0       |
| 82  | The harness counted a render as a pass        | 1       | 0       |

Auto mode cleared every permission finding. No session met a denial, against 34
in round 2. Sessions read the printed render paths, both valley runs set their
budgets from the surface-fill counts, and no review render came out on black.
Four findings still showed:

1. **Renders skipped.** A skipped view hid a flaw. Cottage run 2 never read the
   top view, where a stepping stone overhangs the lawn. Ramen run 1 checked only
   the hero and right views at the end and shipped round bowls as boxes
2. **Lightened gold shades read beige.** A light gold still paled. Chest run 2's
   5-shade ramp of `#F2C230` at spread 0.06 drew near-white stripes, and the
   session dropped the lightest shades. A base that light leaves little
   lightness above it
3. **Sessions cut the report to its model line.** Sessions cut the report with
   `head` or `tail` and lost lines they needed. Chess run 2, village run 2,
   dollhouse run 2, and pirate-ship run 2 each voxelized again only to read the
   cut lines. Cottage run 2 kept only the model line and shipped a stepping
   stone past the lawn's edge
4. **Hero renders came out on black.** A close-up rendered without
   `--profile review` still came out on black because every other render keeps a
   transparent background

### Still open

Nothing changed for these findings beyond the shorter skill:

| #   | Finding                                           | Round 2 | Round 3 |
| --- | ------------------------------------------------- | ------- | ------- |
| 2   | Large surfaces left flat                          | 24      | 23      |
| 4   | Ended with flaws the renders show                 | 22      | 15      |
| 3   | Documented tools left unused                      | 9       | 9       |
| 10  | Features near a voxel thick missed cells or broke | 12      | 9       |
| 6   | Small round shapes read as blocks or plus signs   | 9       | 7       |
| 9   | Finer grids read better than the guidance         | 7       | 7       |
| 17  | Views the review four cannot give                 | 4       | 7       |
| 5   | Dark materials collapse to near-black             | 8       | 6       |
| 7   | Later steps buried earlier details                | 6       | 6       |
| 13  | Neighboring materials too close in hue            | 10      | 6       |
| 32  | Grain read wrong                                  | 2       | 6       |
| 19  | The hero view hid the focal element               | 12      | 5       |
| 65  | Noise read as camouflage                          | 6       | 5       |
| 12  | Shapes at an angle alias                          | 9       | 4       |
| 44  | Placing on the cell grid took hand work           | 0       | 4       |
| 14  | Glass, water, ice, and gems read wrong            | 4       | 3       |
| 16  | Signatures misread                                | 2       | 3       |
| 18  | Emissives wash out                                | 2       | 3       |
| 20  | Library metals read off hue                       | 1       | 3       |
| 34  | Paint and coat spill onto neighbors               | 5       | 3       |
| 80  | Neither pirate ship curved its hull               | 1       | 3       |
| 81  | Sloped snow drew stair-step contour lines         | 1       | 3       |
| 36  | Placements share no object                        | 3       | 2       |
| 83  | Orthographic views showed moire                   | 1       | 2       |
| 27  | Light surfaces turn blue-gray                     | 0       | 1       |
| 37  | Mortar swamps stone walls                         | 0       | 1       |
| 54  | A variable named like a call hid the call         | 1       | 1       |
| 73  | Close-ups selected the whole sword                | 1       | 1       |
| 75  | Flame emissive dropped below the bloom threshold  | 1       | 1       |

Sessions that ended on a known flaw fell from 22 prompts to 15, and hero views
that hid the focal element fell from 12 to 5. Views the review four cannot give
rose from 4 to 7 because three runs combined `--view-orbit` with
`--view-look-at` on one view and lost the render. Findings 15, 21, 30, 31, 40,
45, 55, 56, 57, 69, 76, 77, 78, and 84 showed in no prompt.

The phase 2 candidates that round 3 showed:

| #   | Finding                                           | Round 2 | Round 3 |
| --- | ------------------------------------------------- | ------- | ------- |
| 23  | Scene reports too long to read                    | 5       | 8       |
| 24  | Floating crumbs                                   | 10      | 6       |
| 11  | Seating details on a surface                      | 12      | 5       |
| 26  | A seeded random helper                            | 5       | 4       |
| 22  | Rotation on part placement and a one-sided flip   | 3       | 3       |
| 25  | Sweeps, helices, and spirals                      | 5       | 3       |
| 35  | Posed parts, plumb joints, and ropes across parts | 2       | 2       |
| 46  | Emissives light nothing nearby                    | 2       | 2       |
| 33  | Report lines without a location                   | 0       | 1       |
| 38  | Intended separate pieces read as floating         | 1       | 1       |
| 39  | Inspecting nodes and palette values               | 2       | 1       |
| 43  | A flag for washed-out or hidden emissives         | 0       | 1       |
| 47  | Text                                              | 1       | 1       |
| 48  | Crevice and edge shading in the colors            | 2       | 1       |
| 51  | Exposure inside transmissive volumes              | 2       | 1       |
| 59  | Patterns that line up across tiles                | 0       | 1       |

Findings 29, 41, 42, 49, 50, 53, 58, and 60 showed in no prompt.

### New in round 3

85. **The skill's example matches the chair prompt.** In 1 prompt, both chair
    runs stayed close to the skill's example, an oak chair with rubies in its
    top rail. The chair prompt now measures copying more than modeling
86. **An exposed count went unread.** In 1 prompt, potion run 2's bubbles step
    reported `1 exposed` because one bubble sat outside the inner outline and
    replaced a glass-wall cell. The session never acted on the count, and the
    bubble shipped in the wall
87. **Curves came from hand-sampled points.** In 2 prompts, sessions sampled 2D
    curves by hand. Guitar run 2 fed 112 Catmull-Rom points to `polygon` for its
    body. Both candelabra runs built Bezier and spiral polylines for the arms
88. **An empty union gave an unclear error.** In 1 prompt, a loop that ran
    backwards left `union` with no shapes. The build failed with
    `union shapes[0] must be a Shape3d or a Shape2d, not undefined`. That
    message never says the union was empty
89. **A view selection that matched nothing still rendered.** In 1 prompt,
    fish-tank run 1 passed `--view-select close 'fish-tank'` on a model without
    parts. The glob matched nothing, yet the render wrote a 0.5 m shot blocked
    by a frame post. The session never read it and told the user it showed the
    whole tank
90. **Surface fill ruled out clear water.** In 1 prompt, valley run 2 made its
    water opaque because `--fill-mode surface` leaves the terrain hollow under
    it. A scene cannot combine surface fill with see-through materials over
    solid ground
91. **Market stalls repeated one layout.** In 1 prompt, village run 1 painted
    all four market stalls with one `checker`, and every stall shows the same
    goods. Its ten cottages share one thatch hue and read from above as one
    repeated block
92. **No pattern lays brick courses.** In 1 prompt, dollhouse run 1's brick
    chimney used `cells` with a plaster border and read as irregular stone

### No action

In 29 prompts the two runs chose near-identical plans, and in 26 workarounds
used the tools as intended. In 23 prompts sessions made one-off slips. In 12 the
report and the views caught real defects.

## Log

Round 3 ran on 2026-10-05 with vxl 0.5.0 built from the staged tree over
`6cef3939` and Claude Code 2.1.289 running Claude Opus 5.5 at high effort. Every
run loaded the skill and called its model done. Of the 60 runs, 48 read good and
12 fair, against round 2's 40 and 20. Different agents judged each round, so the
verdicts compare loosely. The runs took 198 passes, of which 5 failed, and 982
turns against round 2's 1,786. A run took a median of 3.0 minutes, and all 60
cost $62. `~/voxel-trials/rounds/2026-10-05-round-3` holds the round, and its
gallery shows every run's renders.

### 1. Chair

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | good    | 4      | 0      | 2.2     | 15,280 at 1.25 cm |
| 2   | good    | 3      | 0      | 2.5     | 14,954 at 1.25 cm |

Both runs built an oak chair at 1.25 cm with a gold-trimmed crest set with a
ruby, sapphires, emeralds, and diamonds. Run 2 reads better because its
dark-stained crest field sets off the gold, a cut ruby, and cut emeralds where
run 1's ornament washes into pale oak above a plain kitchen-chair frame. Both
runs still shipped 2-voxel stones that read as plus signs where round 2's run 2
grew its gems into facets.

Failures:

1. Run 1, pass 4: the session swapped the turned front legs for plain boxes
   because the 2.4-voxel lathe profile read jagged. That removed the only turned
   detail below the seat
2. Run 1, final pass: the session named the pale grey diamonds and the plus-sign
   stones and stopped without fixing either
3. Run 2, pass 1: the 2-voxel finial gems floated as two pieces until pass 2
   reattached them
4. Run 2, pass 3: the close-up render failed because it combined `--view-orbit`
   with `--view-look-at`. A position-based retry worked

Lacked:

None

Missed in the skill:

1. Both runs sized small octahedron gems at 2 to 2.4 voxels although the
   Resolution section says one under about 3 voxels reads as a plus sign. Run
   1's emeralds and diamonds and run 2's diamonds and amethysts read as crosses
   where a finer voxel would have given them facets
2. Run 1 set diamonds and gold on pale oak against the check that neighboring
   colors sit far apart in lightness. The diamonds wash out grey without a
   darker backing
3. Run 1 judged the jewels from the 1,024 px front and hero views without the
   crest close-up that Other views offers
4. Run 2 combined `--view-orbit` with `--view-look-at` although Other views
   pairs a look-at with a position. The close-up was lost to the error

Colors: Both runs' back posts and front legs read one flat tone because 4-voxel
members show a single grain ring. Run 1's splat reads flat brown. Its diamonds
read flat grey against the pale oak.

### 2. Lantern

| Run | Verdict | Passes | Failed | Minutes | Voxels         |
| --- | ------- | ------ | ------ | ------- | -------------- |
| 1   | good    | 4      | 0      | 2.2     | 4,087 at 1 cm  |
| 2   | fair    | 3      | 0      | 1.4     | 20,832 at 5 mm |

Both runs built an iron lantern with a stepped pyramid roof and a dripping
candle on a brass dish. Run 1 reads better because its faintly emissive glass
and lit floor fill the cage with warm light where run 2's crisper 5 mm flame
shows only in a side view behind a flat, unlit iron frame. Unlike round 2, where
both runs re-aimed the hero, run 2 left the corner post over the flame.

Failures:

1. Run 1, pass 1: the handle torus sat above the vent cap as a second piece
   until pass 2 moved it onto the chimney cap
2. Run 2, pass 1: the panes step ran before the cage and took 528 of its 1,344
   cells until pass 2 reordered them
3. Run 2, passes 1 to 3: the default hero kept the corner post over the flame.
   The session added a side view instead of re-aiming the hero

Lacked:

1. Both runs wanted a light source that casts the candle's glow onto nearby
   surfaces. Run 1 faked it with emissive glass and a lit floor where run 2
   painted an emissive band on the candle top

Missed in the skill:

1. Run 1 kept 1 cm voxels that left the flame 4 voxels across. It reads as a
   pale block under an orange cap where a finer voxel or the revolved `vesica`
   that SKILL.md names for flames would have shaped a teardrop
2. Run 2 left the flame hidden behind the front post in the final hero although
   SKILL.md has the prompt's detail face the hero corner with nothing between.
   Re-aiming with `--view-orbit` was the documented fix
3. Run 2 gave each iron part a single flat shade although SKILL.md says large
   faces of one flat color read unfinished. The base slab and roof read
   untextured

Colors: Run 1's glass reads as a flat, almost opaque peach that makes the
lantern look like a paper lantern. Run 2's iron is one flat dark gray on every
part. Its interior floor stays gray with no warm light.

### 3. Chest

| Run | Verdict | Passes | Failed | Minutes | Voxels          |
| --- | ------- | ------ | ------ | ------- | --------------- |
| 1   | good    | 3      | 0      | 1.9     | 100,429 at 1 cm |
| 2   | fair    | 3      | 0      | 2.2     | 90,412 at 1 cm  |

Both runs built an iron-bound chest at 1 cm with a barrel lid pivoted as a part
over a gold heap topped with gems. Run 1 reads better because it covered its
heap with about 150 overlapping round coins in three shades where run 2's
ellipsoid mound reads as a terraced gold dome with gems too small to see.

Failures:

1. Run 1, pass 1: the heap coins came out as box-like lumps that read as a block
   pile until pass 2 replaced them with round coins
2. Run 1, pass 2: gems placed at a guessed heap height sat half-buried until
   pass 3 raised them
3. Run 2, pass 1: the ring handles floated off the side walls as separate pieces
4. Run 2, pass 1: the edge coins sat buried inside the mound at `0 exposed`
5. Run 2, pass 1: the 5-shade gold ramp drew near-white stripes across the mound
6. Run 2, final pass: the mound's contour terraces still dominate the gold. The
   session never addressed them

Lacked:

None

Missed in the skill:

1. Both runs placed gems at a guessed heap height instead of using `coat` with
   `sides`. Run 1 spent pass 3 lifting half-buried gems where run 2's stayed
   tiny and partly sunk
2. Both runs sized details under the 3-voxel radius that SKILL.md says reads as
   a plus sign. Run 1's floor coins render as crosses where run 2's 8-cell
   emerald shows as one green dot
3. Run 2 passed over the pass 1 report line `edge coins 56 kept 0 exposed` until
   the renders showed the buried coins. That cost part of pass 2

Colors: Run 1's brass corner posts and lock plate read as single flat strips.
Its floor coins are flat untextured gold. Run 2's mound reads as smooth banded
yellow terraces between the coins. Its lid lining is a flat dark brown.

### 4. Sword

| Run | Verdict | Passes | Failed | Minutes | Voxels        |
| --- | ------- | ------ | ------ | ------- | ------------- |
| 1   | good    | 5      | 0      | 2.2     | 7,076 at 5 mm |
| 2   | fair    | 3      | 0      | 1.3     | 8,144 at 5 mm |

Both runs built a longsword at 5 mm with a gold crossguard and a ruby in a gold
pommel. Run 1 reads better because it finished every surface with a gold-wire
leather grip, noise on the steel and gold, polished edges, and a stepped cut
ruby where run 2 stopped with a flat steel blade, a gear-like all-gold grip, and
a pommel below the ground. Unlike round 2, both runs chose 5 mm voxels.

Failures:

1. Both runs, pass 2: the close-up render failed because it combined
   `--view-orbit` with `--view-look-at`. Both recovered with
   `--view-frame world`, `--view-position`, and `--view-look-at`
2. Run 1, pass 2: painting the fuller floor with `fuller.offset(v)` spread dark
   paint over 1,244 cells beyond the groove until pass 3 used an exact box
3. Run 1, pass 3: flattening the octahedron gem shrank it to 48 cells. Pass 4
   rebuilt it as stacked chamfered rects
4. Run 2, final pass: the pommel dips 5 mm below the ground. The session named
   the flaw and stopped

Lacked:

None

Missed in the skill:

1. Both runs combined `--view-orbit` with `--view-look-at` although Other views
   documents the orbit and position forms separately. Each lost its first
   close-up to the error
2. Run 2 left the blade and guard bar one flat material although the Checks
   section says a large face of one flat color reads unfinished
3. Run 2 stopped with the pommel below the ground although SKILL.md calls the
   model done only when no review can name a flaw

Colors: Run 1's gold reads slightly olive and mottled rather than bright. Run
2's blade is one flat `mat.steel` gray. Its guard bar, pommel, and neck are flat
`mat.gold`.

### 5. Tree

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | good    | 5      | 0      | 2.1     | 220,778 at 0.15 m |
| 2   | good    | 4      | 0      | 1.9     | 283,726 at 0.1 m  |

Run 1 built a 15 m oak at 0.15 m and run 2 a 10 m oak at 0.1 m, each with a
flared trunk under a lumpy crown lit on top and shaded below. Run 1 reads better
in its front and low views because its long gnarled limbs fan out under a crown
wider than it is tall where run 2's short hidden limbs make a generic lollipop
tree with a carved gap showing through the crown. Unlike round 2, both runs
cleared floating leaves with carve boxes at the report's coordinates rather than
retuning `displace`.

Failures:

1. Run 1, pass 3: switching the crown to a plain `smoothUnion` with 3 octaves
   still left 18 pieces
2. Run 2, pass 3: the first speck-carving boxes left 5 new single-voxel pieces
   that took another pass to clear
3. Run 2, pass 4: one carve box removed a detached voxel from the grass disc's
   `displace` noise
4. Both runs, final pass: hand-placed carve boxes at the report's coordinates
   removed the last stray leaves. Those boxes stop fitting whenever the crown's
   shape or seed changes

Lacked:

1. Both runs wanted a step that drops small disconnected pieces. They carved
   boxes at the report's coordinates instead

Missed in the skill:

None

Colors: Run 1's bark grain repeats every 0.15 m at its 0.15 m voxel. The trunk
reads as camouflage squares rather than bark streaks. Run 2's dark trunk merges
with the shaded leaf undersides where they meet. Its top view shows pale grass
through the crown's gaps.

### 6. Well

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | good    | 4      | 0      | 2.0     | 74,796 at 2.5 cm  |
| 2   | good    | 3      | 0      | 2.1     | 102,704 at 2.5 cm |

Both runs built a round stone well at 2.5 cm with oak posts, a plank gable roof,
a windlass with a rope coil and crank, and a banded bucket. Run 1 reads better
because its dark mortar, flagstone apron, moss patches, and ridge cap sell the
stonework where run 2's pale mortar leaves the wall as gray blotches. Unlike
round 2, both runs raised the roof until the bucket showed in the hero.

Failures:

1. Run 1, pass 1: the mahogany ridge cap read as a red stripe until pass 2
   swapped it for walnut
2. Run 1, pass 1: the moss coat read as a uniform green ring until pass 2 broke
   it into patches
3. Run 2, pass 1: the thin torus bail missed the bucket wall and floated as a
   second piece until pass 2 thickened and lowered it
4. Both runs, passes 1 and 2: the roof hid the bucket and rope in the hero. Both
   raised it in pass 3
5. Both runs, final pass: the eave still hides the rope coil in the hero. Run
   1's rope reads as a short stub

Lacked:

1. Run 2 wanted an operation or render setting that smooths the vertical
   striping on a stair-stepped cylinder wall

Missed in the skill:

1. Run 1 stopped with the coil and most of the rope behind the eave in the hero
   although SKILL.md has the prompt's detail face the hero corner with nothing
   between. It never rendered a lower view to check
2. Run 1's pine crank sits beside oak posts of similar lightness against the
   check that neighboring colors sit far apart. The crank reads as a stray light
   stick rather than a handle
3. Run 2's mortar `#8C8474` sits close to the stone shades against the same
   check. The wall loses its stone-and-mortar read
4. Run 2's first bail at a 0.02 m radius fell under the 2 voxels across that
   SKILL.md asks of a curved member. It detached and cost a pass

Colors: Both curved walls show vertical stair-step stripes on their flanks. Run
2's wall and cap read as blotchy gray camouflage because the mortar nearly
matches the stone. Run 1's pine crank reads too light beside the oak posts.

### 7. Robot

| Run | Verdict | Passes | Failed | Minutes | Voxels          |
| --- | ------- | ------ | ------ | ------- | --------------- |
| 1   | good    | 2      | 0      | 1.6     | 4,964 at 2.5 cm |
| 2   | good    | 2      | 0      | 1.6     | 5,772 at 2.5 cm |

Both runs built a boxy robot at 2.5 cm with the head and arms as parts on neck
and shoulder pivots. Run 1 reads better for its blooming eyes and lamps, slimmer
arms and lighter palette. Run 2 alone rendered a posed copy proving the joints,
but its oversized orange forearms, solid claws, tiny eyes and flat red bulb read
cruder.

Failures:

1. Run 1, before pass 1: the first draft named a material `paint` that shadowed
   the `paint` step function. A rename to `hull` cleared it
2. Run 1, pass 1: the eyes sat half a cell off the grid beside claws that read
   as solid blocks. Pass 2 fixed both
3. Run 2, pass 1: the antenna bulb, a sphere under 2 voxels in radius, voxelized
   into a plus sign. Pass 2 swapped in a rounded box
4. Run 1, final message: the session told the user it could not render a turned
   pose. `vxl node set rotation` plus a render does exactly that

Lacked:

1. Run 1: a documented way to render the model with its parts turned. The
   session concluded none existed and shipped unverified joints
2. Run 2: a SKILL.md note on posing parts for review. The session found
   `vxl node set rotation` through `vxl node --help`

Missed in the skill:

1. Run 2: the antenna bulb took a sphere under 2 voxels in radius though the
   Resolution rules say one under about 3 reads as a plus sign. It cost a pass

Colors: Run 1's backpack and feet read as flat unpatterned orange slabs. Run 2's
antenna bulb and LEDs at `emissiveStrength` 1 read as flat paint with no glow.
Its feet read as flat dark slabs.

### 8. Cottage

| Run | Verdict | Passes | Failed | Minutes | Voxels          |
| --- | ------- | ------ | ------ | ------- | --------------- |
| 1   | good    | 3      | 1      | 3.0     | 91,202 at 0.1 m |
| 2   | good    | 2      | 0      | 1.8     | 294,266 at 4 cm |

Both runs built a spotted red mushroom cottage on a lathed stem with a
stone-framed round plank door, amber windows, a chimney, a path and toadstools.
The runs trade strengths evenly: run 1 at 10 cm shows five well-placed windows
with flower boxes but hides the wall under the cap in the standard hero. Run 2
at 4 cm has the cleaner fully circular door, but its three windows mostly face
away from the front.

Failures:

1. Run 1, pass 1: the build failed with
   `box round must be at most half the shortest side, 0.05000000000000002, not 0.1`
   on the 0.1 m door step. Dropping the round fixed it
2. Run 1, final pass: the standard hero still hides most of the wall under the
   cap and shows one window and half the door. The session added a low orbit
   view in place of reshaping the cap or stem
3. Run 2, final pass: the last stepping stone hangs off the edge of the grass
   disc

Lacked:

None

Missed in the skill:

1. Run 2: pass 2 cut the report to its first line and the session never read the
   top view. Either check would have caught the path's max z of 2.44 past the
   ground's 2.28

Colors: Both runs render the upper wall under the cap gray. Run 1's plaster
noise on the stepped lathe wall reads as vertical streaks. Its tan gills barely
register from any view. Run 2's stem shows concentric contour rings from the
lathe steps.

### 9. Potion

| Run | Verdict | Passes | Failed | Minutes | Voxels        |
| --- | ------- | ------ | ------ | ------- | ------------- |
| 1   | good    | 2      | 0      | 1.2     | 8,724 at 5 mm |
| 2   | good    | 2      | 0      | 1.3     | 5,596 at 5 mm |

Both runs built a lathed round flask at 5 mm with a darker glass lip and base, a
two-part cork and a coated potion surface. Run 2 reads better because its dark
base with emissive, a `y` gradient and alpha glass keep the potion deep red with
depth. Run 1's potion reads flat salmon-pink. Both runs read good where round
2's heroes lost the neck glass and left the cork and collars floating.

Failures:

1. Run 1, pass 1: the surface coat's `within: halfSpace("-y", fill)` recolored a
   5-row band of 392 cells. Pass 2 cut it to one surface row with
   `halfSpace("+y", fill - v)`
2. Run 1, pass 2: the fix for washed-out red raised the potion `baseColor` to
   `#C00010` with emissive `#A00010` at strength 1. The liquid went pink rather
   than deep red
3. Run 2, pass 1: the twine ring around the neck read as floating against the
   faint glass. Pass 2 removed it rather than strengthening the neck glass
4. Run 2, final pass: a bubble at `[0.0375, 0.0225, 0.0325]` replaced a
   glass-wall cell. It stayed though the report flagged `1 exposed` from pass 1

Lacked:

None

Missed in the skill:

1. Run 1: SKILL.md says a dark `baseColor` keeps a strength-1 glow at its full
   hue. The session brightened the base instead and turned the red pink
2. Run 1: the potion takes one flat material over its whole volume against the
   skill's warning that a large one-color face reads unfinished. `shades` or
   `gradient` would have given it depth
3. Run 2: the session passed over the bubbles step's `1 exposed` though the
   skill's checks read an exposed count as where a detail sits. The count marked
   a bubble piercing the glass wall

Colors: Run 1's potion reads coral or salmon pink in one flat tone from bottom
to surface. Its glass reads milky pale blue rather than clear. Run 2's bubble
voxels read as pink speckle more than bubbles. A darker ring of uncoated cells
borders its bright surface where the shoulder overhangs it.

### 10. Cart

| Run | Verdict | Passes | Failed | Minutes | Voxels           |
| --- | ------- | ------ | ------ | ------- | ---------------- |
| 1   | good    | 1      | 0      | 1.4     | 15,108 at 2.5 cm |
| 2   | good    | 3      | 0      | 1.9     | 13,304 at 2.5 cm |

Both runs built a slatted oak cart at 2.5 cm with walnut stakes, iron caps and
four ten-spoke iron-tired wheel parts. Run 1 reads better because its planked
floor and gapped slats read clearly after one pass. Run 2 spent two more passes
on barely visible fixes and left its floor seams invisible, though its board
grain reads richer.

Failures:

1. Both runs, final pass: the one-voxel iron tires stair-step around the rim.
   They read as cog teeth in the hero
2. Run 2, pass 2: the alternating pine shade slices sat too close in tone. The
   plank seams stayed invisible and the session finished with them faint

Lacked:

None

Missed in the skill:

1. Run 2: the floor took eight plank steps with sliced shade lists in place of
   the SKILL.md recipe `bands(shades(mat.pine), { axis, period: 0.1 })`. The
   seams stayed invisible after a pass spent on them

Colors: Run 1's side and end boards read as one oak tone with sparse grain
dashes. Its bolster block reads as a flat dark-brown slab in the front view. Run
2's floor planks blend into one mottled pine surface in the top and hero views.

### 11. Guitar

| Run | Verdict | Passes | Failed | Minutes | Voxels          |
| --- | ------- | ------ | ------ | ------- | --------------- |
| 1   | fair    | 4      | 0      | 4.1     | 248,639 at 4 mm |
| 2   | good    | 1      | 0      | 4.1     | 69,975 at 4 mm  |

Both runs built a steel-string acoustic at 4 mm with six one-voxel strings, an
abalone rosette and an ebony fretboard. Run 2 reads far better because its
spline outline gives true bouts and a waist. Run 1's two-circle body reads as a
snowman or lute. Run 2 hollowed its body with `shell` where round 2 left both
bodies solid.

Failures:

1. Run 1, passes 2 to 4: the session tuned the two body circles and a reverted
   `smoothSubtract` waist cut without leaving the two-circle outline. The
   snowman silhouette survived to the end
2. Run 1, final pass: the sound hole fills the small upper bout. The fretboard
   end runs over the top of the rosette
3. Run 2, final pass: the strings stop at the nut short of the six tuner posts.
   The fretboard widens in a one-voxel step partway up the neck

Lacked:

1. Run 2: a smooth closed curve or spline 2D shape for the body outline. The
   session hand-wrote a Catmull-Rom sampler that fed 112 points to `polygon`

Missed in the skill:

1. Run 1: the body stays solid with a dark cavity painted 20 cells down the
   hole. SKILL.md documents `shell` and names an extruded polygon as the shape
   that hollows cleanly
2. Run 1: the outline stays a two-circle `smoothUnion` though `polygon` with
   points from a TS loop gives a real waist and bouts. Three passes went into
   tuning the circles
3. Run 2: the session ended on the fretboard step and the strings stopping at
   the nut though SKILL.md calls the model done only when no review can name a
   flaw. One more pass could have fixed both
4. Run 2: the session dropped the nut-to-post string legs for fear of broken
   one-voxel diagonals though SKILL.md says a tilted stroke holds at about 2
   voxels wide. Run 1's 1.5-voxel polylines held as one piece

Colors: Run 1's tortoise pickguard noise reads as high-contrast leopard
blotches. Its mahogany side grain bands into heavy horizontal stripes in the
right view. Run 2's pickguard reads as a near-flat dark brown blob. Its spruce
grain at a one-voxel period reads as speckle more than lengthwise grain up
close.

### 12. Candelabra

| Run | Verdict | Passes | Failed | Minutes | Voxels         |
| --- | ------- | ------ | ------ | ------- | -------------- |
| 1   | fair    | 3      | 0      | 2.8     | 21,490 at 4 mm |
| 2   | good    | 4      | 0      | 2.4     | 16,589 at 5 mm |

Both runs built a twisted square stem with lighter corners and scroll arms
extruded from 2D stroke curves at 4 and 5 mm. Run 2 reads better because its
five arms sit evenly off the axes with every candle on an arm under a gradient
flame. Run 1's twisted arris strips read crisper, but it puts four arms on the
axes and parks the fifth candle on the stem.

Failures:

1. Both runs, pass 1: the twist did not read on a stem about 6 cells wide. Each
   widened the bar to 8 cells and lightened its corners over the later passes
2. Run 2, pass 2: the ridge `paint` outside a 0.0195 m cylinder wrote 0 cells
   because the corner slivers were thinner than a voxel. The session caught it
   from the report
3. Run 1, final pass: one of the five candles sits on the stem top in place of
   an arm. The session reported only the front-view overlap, never the departure
   from the prompt
4. Run 1, final pass: the arms are flat bars 2 voxels deep. They look wiry in
   the hero and like hairlines from the top

Lacked:

None

Missed in the skill:

1. Run 1: the session gave up the prompt's five arms for `repeatPolar("y", 4)`
   on the axes to dodge off-axis stair steps. Run 2's `repeatPolar("y", 5)` with
   strokes 2 to 3 voxels wide held off-axis
2. Run 2: the pass 2 ridge paint was a sub-voxel sliver though the Resolution
   rules warn that a shape thinner than a voxel misses every cell center. It
   cost a pass

Colors: Run 1's stem body takes one flat iron shade and reads as a dark uniform
band between the bright arrises. Its flames stack flat yellow, orange and red
blocks like tiny cubes. Run 2's noise on every iron part breaks the light ridges
into checkered fragments in the close-up and front views. Its flames show
visible color bands.

### 13. Knight

| Run | Verdict | Passes | Failed | Minutes | Voxels           |
| --- | ------- | ------ | ------ | ------- | ---------------- |
| 1   | fair    | 5      | 1      | 3.8     | 2,103 at 6.25 cm |
| 2   | good    | 3      | 0      | 2.7     | 2,643 at 6.25 cm |

Both runs built a knight exactly 32 voxels tall at 6.25 cm in exact PICO-8
colors, with an upright sword and rigged head, arm, and leg parts. Run 2 reads
better because its T-visor great helm, forward heater shield, swept plume crest,
and cape give the stronger silhouette. Run 1's shield faces sideways and shows
edge-on in the hero, while its plume reads as a hooked block. Both runs now rig
the knight in parts, where round 2's run 2 built it as one object.

Failures:

1. Run 1, pass 1: the build failed with
   `scale factor must be above zero on each axis, not [-1, 1, 1]` because the
   session mirrored the right arm and leg with a negative scale
2. Run 1, pass 2: the greave shine coat kept 0 cells on both legs. Pass 3
   removed it
3. Run 1, pass 4: a yellow brow cross on the helm read as a beak. Pass 5
   replaced it with a lavender visor strap
4. Run 1, final pass: the heater shield is extruded along x and faces the side.
   The front view shows it as a thin red strip, while the hero sees it edge-on
   with a stair-stepped point
5. Run 1, final pass: the plume reads as an awkward hooked block
6. Run 2, pass 2: the plume shade coat on `-y` and `-z` darkened too much of the
   crest, while the cape flare ran tall. Pass 3 cut the coat to `-y` and
   shortened the flare
7. Run 2, final pass: the forward shield hides most of the torso and right leg
   in the front view

Lacked:

1. Run 1 wanted a one-sided flip for building a right-side part from left-side
   shapes, because `mirror` keeps both halves. A `Bs` helper that negates box x
   corners stood in

Missed in the skill:

1. Run 1 left the helm sides and torso flanks one flat light grey although color
   check 1 asks a large face for `shades`, `noise`, or a pattern. The session
   used `bands` on the fauld alone

Colors: Run 1's helm sides, torso flanks, and red shield field stay single flat
palette colors. Its `#FFF1E8` coats on the helm crown and pauldrons read as
cream slabs stacked on the armor rather than highlights. Run 2's blue shield
field and blue tabard stay flat under the yellow cross, but its yellow bands and
dark gray trim keep the armor from reading unfinished. Its dark gray left fist
and belt merge into one muddy block in the right view.

### 14. Pocket watch

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | good    | 2      | 0      | 3.7     | 228,617 at 0.4 mm |
| 2   | good    | 3      | 0      | 3.6     | 84,907 at 0.5 mm  |

Both runs wrote a full skeleton watch with toothed wheels, ruby jewels, a cream
chapter ring, and blue hands, then made the gears read in pass 2. Run 2 reads
better because its gear train shows clearly under two small bridges and its
chain hangs free of the bow. Run 1's finer 0.4 mm movement and denser curb chain
read richer, but its four bridges cover much of the wheels and the whole model
fuses into one piece. Both rounds end with fused chains, but round 3's sessions
took the fused count as correct rather than fusing a correct chain.

Failures:

1. Run 1, final pass: the 4.6 mm chain pitch against a 10.2 mm link makes each
   link overlap its same-orientation neighbor two places down by about 1 mm. The
   chain fuses into one rigid mass, and its first link fuses to the bow
2. Both runs, final pass: the sessions read their fused chains as correct. Run 1
   presented `1 piece` as a success, while run 2 told the user its links never
   touch although about 25 links report as 2 pieces
3. Run 1, final pass: the four silver bridges cover much of the wheels, while
   the brass center wheel shows only as a cross under the hands
4. Run 2, pass 1: two screws poked through the case side, while a full-width
   train bridge hid the gear train. Pass 2 fixed both
5. Run 2, final pass: the upright chain links read as thin sticks between flat
   rings in the front view

Lacked:

1. Both runs wanted the report to show which chain links fused and where. Its
   piece lines name steps, while every link sat in two unioned steps

Missed in the skill:

1. Neither run asked how many pieces an interlocked chain should report. Check 4
   says only that a second piece means a shape floats, and both sessions took a
   fused count as correct

Colors: Both runs' unchanged `mat.gold` reads olive-mustard in the front, right,
and top views and ochre in the hero. Neither session noticed the cast or
adjusted the gold. Run 2's gold bands barely vary its case.

### 15. Dragon

| Run | Verdict | Passes | Failed | Minutes | Voxels             |
| --- | ------- | ------ | ------ | ------- | ------------------ |
| 1   | good    | 8      | 1      | 12.0    | 576,371 at 1.25 cm |
| 2   | fair    | 4      | 0      | 6.5     | 240,045 at 2 cm    |

Both runs coiled a red dragon in a full ring on a gold mound from `roundCone`
segments along a hand-written polar spine. Run 1 reads closer to the prompt
because its maroon wing, tented over the back with `smoothUnion`, gives the
clearest folded-wing read. Run 2 has the stronger sleeping pose with its head
tucked in the tail's curl, but its wings barely break the outline and its smoke
wisp reads as a rock.

Failures:

1. Run 1, pass 1: the build failed with
   `union shapes[0] must be a Shape3d or a Shape2d, not undefined` because
   `bodyRange(-205, -45)` ran its loop backwards into an empty union
2. Run 1, pass 4, and run 2, pass 2: the close-up missed its framing. Run 1's
   first orbit missed the head, while run 2's `--view-orbit` beside
   `--view-look-at` failed with
   `--view-orbit sets view close's transform, which --view-frame, --view-node, --view-position, or a rotation flag sets already`
3. Run 1, pass 6: the wing finger bones built as tubes floated off the tented
   membrane. The session replaced them with painted strokes along rays from the
   spine
4. Run 1, final pass: the folded wing comes out as a broad uniform cowl that
   buries the torso's silhouette. The head is small and boxy for the body
5. Run 2, pass 3: the smoke wisp split into two pieces and took another pass to
   reconnect. It still reads as a grey lump
6. Run 2, final pass: the session called the model done although its own
   close-up showed the wings barely visible and the horns as slabs
7. Both runs, final pass: the front view reduces the dragon to a low red mound.
   Only run 1's horns mark the head

Lacked:

1. Both runs wanted a tapered tube along a curved path. Run 1 unioned a
   `roundCone` every 3 degrees along its spine, while run 2 chained `roundCone`s
   between spine nodes
2. Both runs wanted the height of the displaced pile surface. Run 1
   reimplemented the lathe profile as `surf(r)`, while run 2 wrote `pileY` for
   the undisplaced dome that `displace` had moved by up to 3.5 cm
3. Run 1 wanted to project a stroke onto a curved surface. Its wing bones became
   `paint` steps over fans of capsule rays from the spine
4. Both runs wanted the report to set loose one-voxel floor coins apart from
   real detached pieces. It listed about 40 coin pieces in run 1 and 36 in run 2

Missed in the skill:

1. Both runs placed gems at a hand-computed pile height although the skill sends
   gems on a pile to `coat` with `sides`. Many came out half buried, such as run
   2's `gems 0` keeping 35 of 70 cells
2. Neither run used the `bend` the skill names for horns. Both sets of horns are
   straight `roundCone` segments that voxelize as flat ivory slabs
3. Run 2 never used `smoothUnion`. Its legs, haunches, and head meet the body
   tube with hard creases that show as stepped seams at the shoulders and neck

Colors: Run 1's folded wing membrane reads as one large dark maroon field in the
hero and back views because its three close shades and faint red bone strokes do
not break it up. Run 2's wings read as a flat dark maroon stripe, while its
uniform pale grey smoke reads as stone.

### 16. Bridge

| Run | Verdict | Passes | Failed | Minutes | Voxels              |
| --- | ------- | ------ | ------ | ------- | ------------------- |
| 1   | good    | 5      | 0      | 2.9     | 955,011 at 0.25 m   |
| 2   | good    | 7      | 0      | 5.2     | 4,220,720 at 0.25 m |

Both runs built stone towers on the banks with parabolic main cables and sagging
side cables as capsule chains, looped one-voxel hangers sized to the curve, and
one piece at 0.25 m. Run 2 reads better, narrowly, because its tapered towers
with plinths and arched portals and its asphalt deck with dashes, curbs, and red
railings look more convincing. Run 1's compact 80 m diorama with a plank deck
reads clearly but plainer.

Failures:

1. Run 1, pass 1: the riverbed coat ran after the river add and recolored the
   water surface sand. The river vanished in the hero view
2. Run 1, passes 2 and 3: displaced rocks shed single-voxel fragments. Pass 4
   cleared them by sweeping the displace seeds
3. Run 2, pass 1: the grass coat ran down the terrain's cut sides. A grass voxel
   floated as a second piece
4. Run 2, passes 2 to 5: displacing the ground box and tree crowns left 107
   pieces. Three passes went to reaching 1 piece
5. Run 2, final pass: the session saw grey-white blotches on its transmissive
   river and dismissed them as reflection
6. Run 2, final pass: the round trees crowd the side spans and anchorages in the
   front view

Lacked:

1. Both runs wanted a 3D curve or sweep primitive for a cable. Each chained
   capsules between computed points on a parabola and a sagging side curve

Missed in the skill:

1. Run 2 kept blotched custom transmissive water although color check 3 warns
   that water reads faintly and the skill calls a model done only when no view
   shows a flaw a review can name. The session never tried the library water or
   an opaque blue noise

Colors: Run 1's sloped grass on the tower shelves and banks renders as
alternating light and dark green stripes from the stair-stepped coat. Its girder
face reads as one flat dark grey band in the front view. Run 2's river shows
grey-white blotches in the top and hero views, while its other colors separate
well.

### 17. Chess set

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | fair    | 3      | 0      | 3.4     | 577,295 at 2 mm   |
| 2   | good    | 3      | 0      | 2.8     | 296,184 at 2.5 mm |

Both runs placed all 32 pieces in a correct starting position on a maple and
walnut board with a brass inlay, from five lathed kinds and an extruded knight.
Run 2 reads better because it turned its knights sideways into horse heads with
ears, mane, and muzzle. Run 1's forward-facing knights show as slabs in the
front view and shark-fin wedges in the hero. Unlike both round 2 runs, run 1
built 32 separate parts instead of one shared part per kind and color.

Failures:

1. Run 1, pass 1: turned pieces came out one voxel lopsided because lathe radii
   landed exactly on cell centers
2. Run 1, pass 1: noise speckled the ivory pieces with dark dots. The session
   stripped the noise rather than tuning it
3. Run 1, passes 1 and 2: the first knight barely read as a horse. Pass 3 redrew
   it as a crude stepped wedge with one eye pit
4. Run 1, final pass: the session shipped forward-facing knights knowing they
   read as slabs from the front
5. Run 2, pass 2: the session re-ran voxelize twice on an unchanged document
   because the pass 1 `tail -60` had cut off the summary line and board steps
6. Run 2, final pass: the knight's eye box cuts through the whole head. Black
   pieces show through the hole in the front view
7. Run 2, final pass: the rook's notches sit cramped at 2.5 mm voxels

Lacked:

1. Run 1 wanted a per-part symmetry flag in the report. It grepped part
   dimensions for odd and even widths to confirm the pieces turned out symmetric

Missed in the skill:

1. Run 1 voxelized 32 separate parts under the default `--frame world` instead
   of one part per piece design under `--frame local`. Pawn copies varied from
   1,323 to 1,327 voxels until a radius nudge evened them
2. Run 1 shipped knights that read as slabs from the front although the skill
   calls a model done only when every view shows the prompt with no flaw a
   review can name. Turning the knights or thickening the head would have fixed
   it

Colors: Both runs' ebony pieces read as near-black silhouettes with little
shading. In the top view the ivory pieces blend toward the light squares in both
runs. Run 1's ivory stays one flat color after the session stripped its noise,
while its frame side grain renders as blotchy dark camouflage. Run 2's ivory
carries faint dark vertical grain streaks that read as dirt up close.

### 18. Ramen stand

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | good    | 3      | 0      | 2.5     | 236,626 at 2.5 cm |
| 2   | good    | 3      | 0      | 3.1     | 171,960 at 2.5 cm |

Both runs built a complete stall on pass 1 with a pink RAMEN sign framed in
cyan, lanterns, noren, stools, and a counter, lettering it from `polyline`
strokes 2 cells wide. Run 1 reads better because its widely spaced letters stay
crisp and its six neon colors stay distinct. Run 2 has richer surfaces and a
vertical 24H blade sign, but its RAMEN letters crowd and several flat or muddy
faces read unfinished.

Failures:

1. Run 1, passes 1 and 2: floating bowl contents left 20 pieces on pass 1 and a
   6-voxel float at the first bowl on pass 2
2. Run 1, pass 3: the session rebuilt the round bowls as rounded boxes to
   reconnect them. They read as red crates
3. Run 2, pass 1: the 1 cm puddle and AC fan well slabs fell between cell
   centers and wrote 0 cells, while the noren rod kept 4 of 84. Pass 2 fixed all
   three
4. Run 2, pass 2: the AC hub floated as a detached 8-voxel piece. Pass 3
   lengthened it into the unit
5. Run 2, final pass: the RAMEN letters sit 2 cells apart. A and M crowd in the
   hero

Lacked:

None

Missed in the skill:

1. Both runs left large faces one flat color although color check 1 asks such
   faces for `shades`, `noise`, or a pattern. Run 1's roof top stays one flat
   slate, while run 2's menu screen, steel back counter, and AC unit take single
   colors

Colors: Run 1's roof top reads as one flat slate face in the top and hero views.
Its noren crests show blank white squares, while its side billboard stacks three
flat color bands. Run 2's teal noise on the front panel reads muddy like
camouflage. Its menu screen stays one flat saturated blue and its steel back
counter a pale flat grey block. Its copper pipe reads as a bright orange stripe
at the back-right corner.

### 19. Dungeon kit

| Run | Verdict | Passes | Failed | Minutes | Voxels          |
| --- | ------- | ------ | ------ | ------- | --------------- |
| 1   | good    | 3      | 0      | 3.5     | 373,878 at 5 cm |
| 2   | fair    | 2      | 0      | 2.6     | 321,536 at 5 cm |

Both runs built a four-tile kit of floor, wall, corner, and arched doorway on 2
m cells, then assembled a one-piece room from tile factories imported out of
`kit.ts`. Run 1 reads better because its blue-stone room carries plinths, caps,
a corner pillar, lit torches, and flagstone joints that line up across every
tile. Run 2's plain beige room shows its wall pattern restarting at each tile as
panel seams. Where neither round 2 kit could serve as a tile library, run 1
exported its four tiles as named parts.

Failures:

1. Run 1, pass 2: the floors rotated with their tiles. The shifted one-voxel
   flagstone gap misaligned joints between neighbors until pass 3
2. Run 2, pass 2: a recolor of `kit.ts` went out without a rebuild of the kit.
   The shipped `kit.sdfj`, `kit.voxj`, and kit renders keep the pass 1 `#5F5850`
   floor and `#8C8679` wall

Lacked:

1. Rotating a placed part: `part` takes only a pivot and an offset. Both runs
   passed a rotation into tile factories that rotate every shape
2. Material property values in the report: both runs checked `walkable` by
   hand-parsing `room.voxj`. `vxl palette show` prints the values but `SKILL.md`
   never mentions it

Missed in the skill:

1. Both runs built a fresh part per room tile instead of one part per tile and
   rotation shared under `--frame local`. Even the same-rotation floors write
   separate objects
2. Run 2 recolored `kit.ts` without rebuilding it although the skill's pass
   builds before it voxelizes

Colors: Run 1's doorway threshold reads as one flat gray slab. Run 2's cap coat,
arch trim, and threshold each read as one flat color.

### 20. Crane

| Run | Verdict | Passes | Failed | Minutes | Voxels          |
| --- | ------- | ------ | ------ | ------- | --------------- |
| 1   | good    | 2      | 0      | 3.4     | 356,257 at 5 cm |
| 2   | good    | 2      | 0      | 3.5     | 82,433 at 0.1 m |

Both runs built a crawler crane as a four-level part tree of crane, cab, boom,
and hook with every pivot on its hinge. Run 1 reads better at the hook end the
prompt names because its 5 cm lattice, two cable falls, and large striped hook
block stay crisp. Run 2's richer cab and tracks sit under a stair-stepped 0.1 m
lattice, heavy rope bars, and a small hook block that reads almost all black.
Both round 3 runs finished in two passes with the boom hung from A-frame
pendants, where round 2's run 1 took four passes and left its boom propped on
its foot pin.

Failures:

1. Run 2, pass 1: thin ropes and a one-voxel `set` tip light broke the model
   into 93 pieces. Pass 2 thickened the ropes from 0.08 m to 0.24 m and made the
   light a sphere
2. Run 2, final pass: the thickened ropes read as heavy black bars across the
   stair-stepped lattice

Lacked:

1. A hook joint that hangs plumb: both runs told the user to counter-rotate the
   hook node by the boom angle. Run 2's hook pivot also rests on a hard-coded
   `cableX = 12.7` that breaks when `boomAngle` changes
2. A cable spanning two parts: both runs split the rigging between the cab's
   A-frame ropes and the boom's pendants. Both disclosed the gap that opens when
   the boom tilts

Missed in the skill:

1. Run 2 picked 0.1 m voxels for 0.08 m ropes and a one-voxel tip light although
   `SKILL.md` sets the voxel size by the smallest feature at 2 or more voxels.
   The miss cost pass 1 and forced the bar-like ropes

Colors: Run 1's deck, carbody, side frames, and tracks share one near-black
charcoal. Its lower works read as one dark slab in the hero under a flat yellow
cab box. Run 2's 2-voxel yellow and black hook bands read nearly all black. Its
gray deck and carbody are flat slabs.

### 21. Fish tank

| Run | Verdict | Passes | Failed | Minutes | Voxels          |
| --- | ------- | ------ | ------ | ------- | --------------- |
| 1   | good    | 4      | 0      | 3.5     | 619,404 at 5 mm |
| 2   | fair    | 2      | 0      | 3.1     | 130,877 at 5 mm |

Both runs built a 5 mm tank of gravel, a castle, seaweed, fish, and bubbles. Run
1 reads better because its rounded fish swim vivid behind alpha-tinted water,
while run 2's tank holds no water and its 3-voxel slab fish turn to sticks in
the side and top views. Run 2 wins only on bubbles because its bold white
streams read where run 1's fade into the water. Run 1's `#RRGGBBAA` glass and
water fill the tank without dulling the fish, where neither round 2 run tried
alpha and both left out a water volume.

Failures:

1. Run 1, pass 2: moving the bubble streams put the clearest column behind
   seaweed and the air tube. The final hero shows almost no bubbles
2. Run 1, pass 4: `--view-select close 'fish-tank'` matched no part and rendered
   a 0.5 m shot blocked by a frame post. The session told the user the close
   view framed the whole tank without reading it
3. Run 2, both passes: the fish are 3-voxel slabs with 1-voxel fins that read as
   flat sticks in the right and top views
4. Run 2, final pass: a stray gravel voxel ships as its own piece

Lacked:

None

Missed in the skill:

1. Run 1 raised the bubbles' `emissiveStrength` from 0.4 to 0.8 to fix their
   pale look. `SKILL.md` says a glow that reads pale takes a lower
   `emissiveStrength` or a darker `baseColor`
2. Run 2 left out water although the library names `water` and `SKILL.md` covers
   transparent materials. Its fish hang in an empty tank
3. Run 2 used `#RRGGBB` glass with transmission instead of a `#RRGGBBAA` alpha.
   The glass still grays the front view after lightening
4. Run 2 shipped its stray gravel voxel although `SKILL.md` reads a second piece
   as a floating shape

Colors: Run 1's bubbles read flat pale blue against the pale-blue water. Its top
view tints the gravel and fish one blue-gray. Run 2's glass lays a gray-blue
wash over the front view with a dark diagonal streak across the backdrop. Its
bubbles read as flat white clumps.

### 22. Wizard tower

| Run | Verdict | Passes | Failed | Minutes | Voxels           |
| --- | ------- | ------ | ------ | ------- | ---------------- |
| 1   | good    | 2      | 0      | 2.5     | 115,917 at 0.1 m |
| 2   | good    | 4      | 0      | 2.4     | 200,021 at 0.1 m |

Both runs built a one-piece stone tower at 0.1 m whose spiral stair of rotated
box treads climbs to a purple witch-hat roof shaped by `bend`. Run 1 reads
better because a tight 2.4 m bend on its roof's upper third gives an
unmistakable crook. Run 2's 72 slate treads make a crisper helix than run 1's
overlapping 1 m treads, but its two gentle bends only slump the roof. Run 1's
hook is the sharpest roof of either round, where round 2's run 1 stayed mild and
its run 2 drooped away from the hero.

Failures:

1. Run 2, pass 3: the orb sat 0.15 m past the bent roof tip and voxelized as a
   detached 86-voxel piece. Pass 4 lowered and enlarged it
2. Run 1, final pass: the landing's end pokes past the wall in the front view
   with nothing under it

Lacked:

1. A helix or a sweep along a path: both runs looped rotated boxes per step for
   the treads, posts, and rail
2. Where a point lands after a bend and a rotate: run 1 derived the roof tip by
   hand trig to seat its finial. Run 2 bent its orb with the roof instead

Missed in the skill:

None

Colors: Run 1's sandstone sills merge with the oak treads beside them. Run 2's
slate treads and landing read as one flat blue-gray. Its dark roof bands sit
close in lightness and read murky in the hero.

### 23. Log cabin

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | good    | 5      | 1      | 4.0     | 1,561,525 at 5 cm |
| 2   | good    | 3      | 0      | 6.2     | 1,104,750 at 5 cm |

Both runs built a round-log cabin at 5 cm with lit windows, a trench to the
door, thick added roof snow, blue icicles on both eaves, and a smoke plume from
a stone chimney. Run 2 reads slightly better because its hero shows a lit side
window and its icicles read sharper, while run 1's eave hides its windows in the
hero. Both cabins sit only partly buried on a sheer-sided snow plinth. Run 2's
hero keeps its lit window in view where round 2's run 2 lost the door and window
to roof shadow.

Failures:

1. Run 1, pass 1: the build failed with
   `ground snow: box round must be at most half the shortest side, 0.25, not 0.3`
2. Run 1, pass 2: ground snow added after the windows and door buried both. A
   full pass went to reordering the steps
3. Run 1, pass 2: the smoke floated as piece 2. Several icicles also floated
   because their cones started below the roof underside
4. Run 1, pass 4: the window frame's closed back face covered the glow. The
   windows stayed dark until pass 5 rebuilt the frame as an open ring
5. Run 2, pass 1: the `chinking` step kept 16 of 87,576 cells. The session
   dropped it in a rewrite
6. Run 2, passes 2 and 3: the `pine trunks` step kept 0 cells under the snow
   base. The pines stand on snow with no trunk
7. Both runs, late passes: the `close` orbit framed the cabin too tight. Run 1
   rendered a low view instead and run 2 set a world camera with
   `--view-position` and `--view-look-at`
8. Run 2, pass 3: `--profile review` with `--view-orbit close` and
   `--file-stem cabin-close` wrote five `cabin-close-*` files that the session
   deleted
9. Both runs, final pass: loose crumbs stay as extra pieces. Run 1 keeps three
   single roof-snow voxels and run 2 a 2-voxel ground-snow crumb plus a single
   voxel

Lacked:

1. Whether an emissive surface sits behind an opaque face: run 1's report
   counted 576 exposed glow cells while a frame face hid them. Finding the dark
   windows took an extra low render

Missed in the skill:

1. Run 1 added the ground snow after the window and door details although
   `SKILL.md` puts large forms first. Unburying the openings took a pass
2. Run 2 passed over the report's `0 kept` on `pine trunks`. Its pines stand on
   snow with no trunk

Colors: Both runs' trench walls and plinth sides read as flat pale planes. Both
roof snow loads read as near-uniform white.

### 24. Valley

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | good    | 3      | 0      | 2.4     | 160,042 at 0.75 m |
| 2   | good    | 6      | 0      | 13.3    | 247,440 at 0.8 m  |

Both runs built a 160 m terrain of smooth-unioned displaced cones under
`--fill-mode surface`. Each carved a river into a flat floor and seated pines by
a hand-written height function. Run 2's canopy hides most of the river in the
hero, but its eleven peaks, foothills, waterfall into a pond, 552 pines, and
strata-cut sides read far more like a mountain valley than run 1's sparse cones
and canal-straight river. Neither round 3 run hit the surface-fill miscount that
misled both round 2 runs.

Failures:

1. Both runs, pass 1: the displaced terrain and snow left crumbs. Run 1 had
   seven extra pieces with a 16-voxel snow island, while run 2 had three single
   voxels
2. Run 2, pass 3: a trial at 0.75 m came to 286,112 voxels in four pieces, all
   snow crumbs on one back peak

Lacked:

1. The terrain height at a point: both runs mirrored the cones in TypeScript
   while ignoring the displacement. Run 1 sank its trunks 4 m and run 2 5 m
2. A step that drops pieces under a size: run 1 softened the displacement and
   dropped its frost coat. Run 2 voxelized four displacement seeds and kept the
   one that came out whole
3. A rolled-up report line for repeated parts: run 2's report printed three
   lines per tree. The model and piece lines sat buried under 39 KB of output

Missed in the skill:

None

Colors: Run 1's pale meadow carries only soft noise. Its river reads one flat
blue from the top, and its side cut shows only a thin dark dirt strip under the
floor. Run 2's canopy merges into one dark green mass in the top view. Its snow
caps read as large near-white blobs.

### 25. Village

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | good    | 3      | 0      | 6.2     | 457,971 at 0.25 m |
| 2   | good    | 5      | 0      | 12.8    | 398,795 at 0.25 m |

Both runs ringed a cobbled market square with thatched cottages from a
`cottage()` function and closed its north side with a bespoke stone church. Run
2 reads better because of its gabled and hipped cottages with varied framing,
doors and chimneys around a square busy with stalls, villagers and a roofed
well. Run 1 ends in one clean piece, but its ten cottages repeat one hipped
design. Round 3's run 2 varied its cottage designs where both round 2 runs
repeated eight hipped cottages.

Failures:

1. Run 2, before pass 1: the first draft held a no-op `door knob` step and a
   stray `export default []`. The session deleted both
2. Run 1, pass 2: one floating thatch voxel on `cottage.w1` made 2 pieces. Pass
   3 cleared it by reseeding the cottage
3. Run 2, pass 2: pass 1 had piped the report through `head -150`. The session
   voxelized again only to read the rest
4. Run 2, pass 4: a floating well-roof shell, villager torsos split by a carve
   and loose leaf voxels made 20 pieces. Pass 5 fixed the well and villagers
5. Run 2, final pass: five single leaf voxels left 6 pieces. The session
   accepted them as invisible

Lacked:

1. Both runs: `part()` takes only a pivot and an offset with no turn per
   placement. Each run wrote a `cottage()` function that rotates every shape and
   builds a fresh part per call
2. Both runs: a summary-only report for large scenes. Run 1 grepped a 36 KB
   report file for piece lines, while run 2 voxelized again to read what `head`
   cut off

Missed in the skill:

1. Run 1: SKILL.md says copies repeat their pattern unless each takes its own
   seed. The four stalls share an unseeded produce checker and show identical
   goods
2. Run 2: SKILL.md treats a second piece as a floating shape to fix. The session
   shipped five loose leaf voxels
3. Run 2: the session cut pass 1's report with `head -150` instead of saving it
   once. Reading the rest cost a second voxelize

Colors: Run 1's stained-glass checkers read as pixel confetti at hero distance.
Its cottages share one thatch hue that repeats as one block from above. Run 2's
church walls read as blotchy grey camouflage cells instead of coursed stone. Its
stained glass also reads as confetti. Its thatch bands look combed rather than
layered.

### 26. City

| Run | Verdict | Passes | Failed | Minutes | Voxels              |
| --- | ------- | ------ | ------ | ------- | ------------------- |
| 1   | good    | 3      | 0      | 7.4     | 1,093,313 at 0.25 m |
| 2   | good    | 2      | 0      | 5.9     | 277,392 at 0.5 m    |

Both runs built a parts-free city with crossing streets, crosswalks, cars,
street lights, a park and a glass skyscraper. Run 2 reads better because its
nine blocks at 0.5 m hold varied towers and read as a city, though trees hide
the fountain in its cramped park. Run 1's 0.25 m detail is crisper, but its four
blocks read as one intersection. Round 3's run 2 varied its towers where round
2's nine-block run repeated one boxy slab.

Failures:

1. Run 2, before pass 1: the session dropped the park benches instead of
   resizing them for 0.5 m voxels
2. Run 1, pass 1: the frame paint covered the skyscraper glass, and the report
   showed the glass at 0 exposed. The tower rendered as a white block until pass
   2 restored the curtain wall
3. Run 1, pass 1: a water tower floated off its legs. With stray leaf voxels it
   made 8 pieces
4. Both runs, pass 1: displaced tree crowns shed floating leaf voxels. Run 2
   reseeded its one crown in pass 2, while run 1 needed until pass 3 to lower
   the crown displacement
5. Run 2, final pass: four trees hide the fountain in both the hero and the
   close-up

Lacked:

None

Missed in the skill:

1. Run 1: SKILL.md points large scenes at `--fill-mode surface`. The session
   kept solid fill at 1,093,313 voxels

Colors: Run 1's lit panels on the glass tower show as big orange patches against
busy mottled glass. Its roofs read as large flat gray tar panels in the top and
hero views. Run 2's dark navy glass tower and slate tower read as near-flat
single tones on large faces. Some of its roofs are flat gray.

### 27. Dollhouse

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | good    | 3      | 0      | 6.5     | 812,600 at 5 cm   |
| 2   | good    | 5      | 0      | 9.2     | 1,067,308 at 5 cm |

Both runs built a furnished clapboard dollhouse at 5 cm with the front wall
removed and every cut edge in a contrasting material. Run 2 reads better because
the cut runs up through the gable to a furnished attic and its re-aimed hero
looks into every room. Run 1's four tidy rooms read in the front view, but its
default hero shows mostly roof.

Failures:

1. Run 1, pass 1: a towel and towel bar floating 5 cm off the bathroom wall and
   a one-voxel bush speck made 3 pieces. Clearing them took two more passes
2. Run 2, pass 1: `tail -150` cut off the report's model and pieces lines. Pass
   2 voxelized again without a build only to read them
3. Run 2, pass 2: displaced shrub and crown specks, a floating teddy, a towel
   and a faucet made 80 pieces. A nightstand drawer paint kept 0 cells
4. Run 2, passes 3 to 5: teddy ears 4 cm in radius kept detaching at 5 cm
   voxels. They cost two extra passes
5. Run 2, final pass: thin tan lines poke through the roof near the back edge in
   the right and top views

Lacked:

None

Missed in the skill:

1. Run 1: SKILL.md says a room takes a view from Other views. The session added
   two low views but kept the default hero of mostly roof
2. Run 2: Resolution rule 2 says round details under about 3 voxels of radius
   read as blocks. The 4 cm teddy ears split off as specks for two passes
3. Run 2: the color checks flag large flat faces as unfinished. The chimney
   shipped as one flat red box

Colors: Run 1's kitchen side wall reads as one flat mustard plane. Its bathroom
upper wall reads flat teal. Its chimney's cells pattern reads as irregular stone
rather than brick. Run 2's chimney is one flat brick-red box with no shades or
pattern. Its stair side reads as a large flat brown triangle in the front view.

### 28. Living room

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | good    | 2      | 0      | 5.8     | 378,538 at 2.5 cm |
| 2   | good    | 1      | 0      | 5.0     | 374,423 at 2.5 cm |

Both runs built a cutaway corner at 2.5 cm with the fireplace on the back wall,
a stocked bookshelf on the left and an eye-level inside view. Run 2 reads better
because its squared wingback, ottoman, floor lamp, log basket and sleeping cat
fill a cozier room with no sampling ribs, though the chair faces the viewer
rather than the fire. Run 1 has the better kilim rug, but its 45-degree wingback
samples into corduroy ribs. Round 3's better run squared its chair, where round
2's better run shipped the same ribbed 45-degree chair.

Failures:

1. Run 1, pass 1: a floating mantel detail made 2 pieces. Pass 2 widened the
   candle holders and lowered the vase sprigs
2. Run 1, final pass: the armchair turned 45 degrees off the axes samples into
   stair-step ribs. The session named it a compromise rather than fixing it
3. Run 2, final pass: the armchair faces +z beside the hearth. It neither faces
   the fire nor leaves the firebox clear in the hero

Lacked:

None

Missed in the skill:

1. Run 1: SKILL.md pitfall 6 says furniture reads cleanest square to the axes.
   The session turned the armchair 35 then 45 degrees and accepted the ribs
2. Run 1: the skill's example centers boards at the origin for centered grain.
   The uncentered shelf boards cost part of pass 2

Colors: Run 1's hearth reads as one flat slate-grey slab. Its night window pane
is close to a single flat navy. Its bookshelf top carries pale grain blotches.
Run 2's night sky and rug field both read near-flat navy. Its mantel jar reads
as a brown blob.

### 29. Spaceship

| Run | Verdict | Passes | Failed | Minutes | Voxels          |
| --- | ------- | ------ | ------ | ------- | --------------- |
| 1   | good    | 4      | 0      | 3.6     | 214,530 at 5 cm |
| 2   | fair    | 4      | 1      | 3.9     | 183,229 at 5 cm |

Both runs built a bridge at 5 cm open on +x and +z with a captain's chair, a
radar, a starfield window and an inside camera behind the chair. Run 1 reads
better because it enclosed the starfield in a box behind the back wall and
finishes as one piece from every view. Run 2 drew the better flat radar table
and sky, but its starfield floats behind the room as a billboard that nearly
doubles the bounds. Both round 2 radars read as rings, where round 3's run 1
tilted its disc until the rings stair-stepped into stripes.

Failures:

1. Run 1, pass 1: the radar console's `extrude` profile under axis `"x"` took
   `(z, y)` order. The console landed under the floor at y -1.85 to -0.75 for 3
   pieces
2. Run 2, pass 1: voxelize failed with
   `bridge/stars white: set points must be distinct, not [-4.625, 3.375, -4.925] twice`
   because of hand-scattered star points
3. Run 1, pass 2: the command ran voxelize twice in a row to grep different
   report lines
4. Run 2, pass 2: the halo sphere buried the planet at 73,824 cells with 0
   exposed. It rendered as a solid blue ball until pass 3 cut it to a half dome
5. Run 1, final pass: the radar tilted 40 degrees stair-steps its arcs and
   crosshair into horizontal green stripes in the inside view
6. Run 2, final pass: the starfield backdrop stays a detached second piece. The
   session called it a limitation instead of attaching or enclosing it

Lacked:

None

Missed in the skill:

1. Run 1: SKILL.md documents that `extrude` under axis `"x"` maps a profile's u
   and v to `(y, z)`. The session wrote the profile in `(z, y)` and lost pass
   1's console
2. Run 1: SKILL.md warns that a thin form turned off the axes samples into ribs
   and stair steps. The 40-degree radar face stripes its rings and crosshair
3. Run 2: `speckle` scatters accents over a face per grid cell. Hand-placed
   `set` points for about 400 stars cost a failed pass on duplicates
4. Run 2: the checks flag a step at 0 exposed as buried. Pass 2's planet read 0
   exposed under its halo and cost a pass
5. Run 2: SKILL.md says a second piece means a shape floats. The session shipped
   the floating backdrop that shows from every outside view

Colors: Run 1's tilted radar face reads as green and dark-green horizontal
stripes rather than rings. Its side console's speckled buttons read as noise.
The outside of its sky box is a plain banded gray crate. Run 2's overhead
bulkhead is one large flat dark slab. Its chair shell is flat gray with a single
red block for a headrest. The back of its backdrop shows as a flat dark sheet in
the right view.

### 30. Pirate ship

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | fair    | 3      | 0      | 4.8     | 174,876 at 5 cm   |
| 2   | good    | 3      | 0      | 7.8     | 949,849 at 2.5 cm |

Both runs built a gun deck with red-carriage guns in carved ports, hammocks,
barrels and an emissive lantern on a chain. Run 2 reads better because its open
cutaway at 2.5 cm shows the whole deck and its richer props from the standard
hero. Run 1 curved its walls with a tumblehome profile, but at 5 cm its guns
read as blobs and all four review views hide the deck behind a closed crate.
Round 3's run 1 curved the hull where neither round 2 run did.

Failures:

1. Run 2, pass 1: the door's arch was extruded along the wrong axis. It stood
   edge-on in the bulkhead until pass 3 rotated it
2. Run 2, pass 2: `tail -80` had cut off report lines in pass 1. The session
   voxelized the unchanged model twice to read them
3. Run 1, pass 3: the session added rope rims to the hammocks that do not show
   in the eye-level view. The pass changed nothing visible
4. Run 1, final pass: the session shipped at 5 cm although cannonballs, bores
   and trunnions span 1 to 3 voxels. The cannons read as blobs and the hammock
   undersides stair-step into jagged bowls
5. Run 2, final pass: the propped port lids stick out past the hull at -60
   degrees. They read as debris in the top and right views

Lacked:

1. Both runs: a light source the lantern could cast into the room. Each lantern
   is an emissive glass box that glows without lighting anything

Missed in the skill:

1. Run 1: SKILL.md asks every named feature to span at least 2 voxels. At 5 cm
   the cannonballs and gun barrels read as lumps
2. Run 1: SKILL.md lists `bend` on a thin box for curled forms. The session
   subtracted two ellipsoids into thick, jagged hammock bowls
3. Run 2: SKILL.md builds a curved wall from one `extrude` of a polygon profile
   or a `bend`. The flat box topsides make the interior read as a rectangular
   room

Colors: Run 1's ceiling top fills the hero and top views with plain alternating
bands. Its gun barrels are one flat navy with no highlight. They merge into dark
blobs against the shot. Run 2's gun barrels are one flat near-black that loses
its shape against the black shot. Its near-black seam on every floor and deck
plank reads as heavy stripes.
