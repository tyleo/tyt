# Trials, round 5

Step S14 of the [checklist](checklist.md) runs the [prompts](trials.md#prompts)
a fifth time after round 4's fixes. [Round 4](trials-round-4.md) logs the round
before, and the [trial
harness](../../../../projects/vxl/trials/README.md) ran all five.

## Changes before round 5

Round 4's findings led to five changes in vxl and its skill:

1. A `fit` distance or orthographic scale beside `--view-look-at` errors and
   asks for meters
2. The skill's sizing example became a birdcage in place of a guitar
3. `round: 0` and `chamfer: 0` give square corners
4. The skill says a close-up alone leaves `--profile review` off
5. The skill calls a model done only when every extra piece and `0 kept` step in
   the report is intended

## Findings

Round 5's analyses recorded 480 items across the 60 runs, against round 4's 484.
Every finding keeps its round 4 number, and new findings start at 103. The
round's gallery lists each finding's items beside the renders.

### What the changes did

The table counts the prompts that showed each finding the changes answered.

| #   | Finding                                      | Round 4 | Round 5 |
| --- | -------------------------------------------- | ------- | ------- |
| 4   | Ended with flaws the renders show            | 18      | 21      |
| 24  | Floating crumbs                              | 14      | 7       |
| 17  | Views the review four cannot give            | 2       | 0       |
| 55  | 0 kept flags intended repaints               | 2       | 2       |
| 93  | The review set took a close-up's stem        | 2       | 0       |
| 85  | The skill's example matches the chair prompt | 1       | 0       |
| 100 | `round: 0` errored                           | 1       | 0       |

No session went without a view it wanted, no review set took a close-up's stem,
and no run copied an example. No session passed a zero round, so round 5 left
that change untested. The done line answered three findings in part:

1. **Ended with flaws the renders show.** Sessions still shipped flaws they had
   named or the renders show. Both lantern runs said nothing hid the flame while
   the front-right post crosses it. Knight run 1 stopped after one pass with its
   crown at `0 kept`, and cottage run 2 shipped two carves at `0 kept` without a
   word
2. **Floating crumbs.** Most sessions read their piece lines before finishing
   and fixed the floats. Dragon run 2 cut its piece lines and mistook 4 pieces
   from its spikes and wing bones for floor coins. Log-cabin run 2 waved off 17
   specks as invisible. Pocket-watch run 2 fused a chain it had interlocked to
   clear its extra pieces
3. **0 kept flags intended repaints.** Dollhouse run 1 called its
   `inner walls 0 kept` an intended repaint, and spaceship run 1 passed over its
   painted niche at `0 kept`. No session undid an intended repaint

### Still open

Nothing changed for these findings:

| #   | Finding                                           | Round 4 | Round 5 |
| --- | ------------------------------------------------- | ------- | ------- |
| 2   | Large surfaces left flat                          | 19      | 22      |
| 10  | Features near a voxel thick missed cells or broke | 11      | 11      |
| 13  | Neighboring materials too close in hue            | 12      | 11      |
| 5   | Dark materials collapse to near-black             | 9       | 8       |
| 14  | Glass, water, ice, and gems read wrong            | 5       | 8       |
| 19  | The hero view hid the focal element               | 10      | 8       |
| 65  | Noise read as camouflage                          | 9       | 7       |
| 81  | Sloped snow drew stair-step contour lines         | 3       | 6       |
| 6   | Small round shapes read as blocks or plus signs   | 5       | 5       |
| 7   | Later steps buried earlier details                | 9       | 5       |
| 9   | Finer grids read better than the guidance         | 5       | 4       |
| 12  | Shapes at an angle alias                          | 5       | 4       |
| 20  | Library metals read off hue                       | 1       | 4       |
| 44  | Placing on the cell grid took hand work           | 4       | 4       |
| 16  | Signatures misread                                | 0       | 3       |
| 18  | Emissives wash out                                | 1       | 2       |
| 27  | Light surfaces turn blue-gray                     | 1       | 2       |
| 32  | Grain read wrong                                  | 6       | 2       |
| 34  | Paint and coat spill onto neighbors               | 4       | 2       |
| 71  | Sessions cut the report to its model line         | 2       | 2       |
| 75  | Flame emissive dropped below the bloom threshold  | 2       | 2       |
| 80  | Neither pirate ship curved its hull               | 2       | 2       |
| 3   | Documented tools left unused                      | 6       | 1       |
| 30  | Proportion and silhouette drift                   | 1       | 1       |
| 31  | Library rope and grass read off                   | 0       | 1       |
| 36  | Placements share no object                        | 1       | 1       |
| 66  | Sessions guessed render file names                | 0       | 1       |
| 68  | Lightened gold shades read beige                  | 1       | 1       |
| 69  | Oak read orange and walnut read pink              | 1       | 1       |
| 83  | Orthographic views showed moire                   | 0       | 1       |
| 87  | Curves came from hand-sampled points              | 3       | 1       |
| 91  | Market stalls repeated one layout                 | 3       | 1       |
| 92  | No pattern lays brick courses                     | 2       | 1       |
| 95  | A second bend moved the first                     | 1       | 1       |
| 102 | Steam has no volume material                      | 1       | 1       |

Large flat surfaces rose from 19 prompts to 22, and glass, water, ice, and gems
that read wrong rose from 5 to 8. Library metals off hue rose from 1 to 4, and
stair-step contour lines rose from 3 to 6. Buried details fell from 9 to 5, and
grain that read wrong fell from 6 to 2. No light material washed out on white,
against 4 prompts in round 4. Findings 1, 8, 15, 21, 28, 37, 40, 45, 52, 54, 56,
57, 67, 70, 72, 73, 74, 76, 77, 78, 79, 82, 84, 86, 88, 89, 90, 94, 96, 97, 98,
99, and 101 showed in no prompt.

The phase 2 candidates that round 5 showed:

| #   | Finding                                           | Round 4 | Round 5 |
| --- | ------------------------------------------------- | ------- | ------- |
| 11  | Seating details on a surface                      | 7       | 8       |
| 25  | Sweeps, helices, and spirals                      | 3       | 4       |
| 26  | A seeded random helper                            | 5       | 3       |
| 46  | Emissives light nothing nearby                    | 2       | 3       |
| 22  | Rotation on part placement and a one-sided flip   | 4       | 2       |
| 23  | Scene reports too long to read                    | 2       | 2       |
| 35  | Posed parts, plumb joints, and ropes across parts | 2       | 2       |
| 38  | Intended separate pieces read as floating         | 1       | 2       |
| 51  | Exposure inside transmissive volumes              | 0       | 2       |
| 33  | Report lines without a location                   | 0       | 1       |
| 39  | Inspecting nodes and palette values               | 1       | 1       |
| 47  | Text                                              | 1       | 1       |
| 53  | Parts that cut through each other                 | 1       | 1       |
| 59  | Patterns that line up across tiles                | 0       | 1       |

Findings 29, 41, 42, 43, 48, 49, 50, 58, and 60 showed in no prompt.

### New in round 5

103.  **A twist on a thin stem did not read.** In 1 prompt, both candelabra runs
      twisted a square stem 8 to 10 cells across, and neither silhouette showed
      the twist at 600 or 900 degrees per meter. Both widened the stem, raised
      the rate as high as 1,400, and painted the corners before the spiral read
104.  **Millimeters went into meter arguments.** In 1 prompt, pocket-watch run 2
      wrote its shapes in millimeters. A round error and two grid-cap errors
      followed before the session built in millimeters and scaled each step by
      0.001. The grid-cap error gives a cell count without the model's bounds.
      The bounds would have shown the 1,000x scale
105.  **Small carved pits vanished.** In 1 prompt, chess run 1 carved the
      knight's eyes as pits of about 6 cells, and they vanish in every render.
      Run 2 painted its eyes in the other army's material, and they read
106.  **The report gave no count per placement.** In 1 prompt, valley run 2
      placed 170 pines under `--frame local`. The report gave no count per
      placement, and the session could not tell whether the shared trees counted
      toward the 300k budget once or per placement
107.  **Thatch read as planks.** In 1 prompt, village run 2's thatch took y-axis
      `bands` and a dark ridge roll. Both read as horizontal wooden planks, and
      the ridge strip lasted until pass 5
108.  **A posed light needed a frame flag.** In 1 prompt, pirate-ship run 1's
      inside render failed twice with `light 1's transform lacks --light-frame`,
      once for its point light and once for its directional light. The skill
      documents no light flags, and the session found them in `--help`
109.  **A part on a sibling read detached.** In 1 prompt, pirate-ship run 2's
      keg sits on barrel 1's lid as a sibling part. The model voxelized as 1
      piece, yet the keg read `detached` because the check looks only for
      contact with the parent
110.  **The skill never mentions `--flatten objects`.** In 1 prompt, the city
      asks for one object, and the skill never mentions `--flatten objects`.
      Neither city run built parts, and the gap cost nothing

### No action

In 28 prompts the two runs chose near-identical plans, and in 29 workarounds
used the tools as intended. In 21 prompts sessions made one-off slips. In 18 the
report and the views caught real defects.

## Log

Round 5 ran on 2026-10-05 with vxl 0.5.0 built from the staged tree over
`2c555db5` and Claude Code 2.1.289 running Claude Opus 5.5 at high effort. Every
run loaded the skill and called its model done. Of the 60 runs, 52 read good and
8 fair, against round 4's 45 and 15. Different agents judged each round, so the
verdicts compare loosely. The runs took 212 passes, of which 14 failed, and
1,009 turns against round 4's 973. A run took a median of 3.3 minutes, and all
60 cost $63. `~/voxel-trials/rounds/2026-10-05-round-5` holds the round, and its
gallery shows every run's renders.

### 1. Chair

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | fair    | 5      | 0      | 2.1     | 17,986 at 1 cm    |
| 2   | good    | 3      | 0      | 1.8     | 10,474 at 1.25 cm |

Both runs built an oak side chair with an arched crest, gold torus bezels around
octahedron gems, amethyst-capped posts, and square legs. Run 2 reads better
because its full-width gilt crest keeps the ruby and sapphires in separate gold
rings, where run 1 grew its bezels on a narrow 34 cm crest until they fused into
one lumpy gold mass. Unlike round 4, where growing the gems at 1 cm won, run 1's
growth at 1 cm buried its crest under gold.

Failures:

1. Both runs, pass 2: the enlarged amethysts floated above the post tops as two
   extra pieces. Run 2 seated them in pass 3 where run 1 took until pass 4
2. Run 1, pass 1: the mid stretcher floated as a second piece until pass 2
   widened it into the side stretchers
3. Run 1, final pass: the bezels, grown to a 1 cm tube around larger stones,
   fuse into each other. The session's final message names the overlap and ships
   it
4. Run 2, final pass: the four diamonds render as faint gray 2-cell cubes that
   look like glitches. The session shipped them while noting they show faintly

Lacked:

None

Missed in the skill:

1. Both runs shipped octahedron gems under the 3-voxel radius at which SKILL.md
   says an octahedron reads as a plus sign. Run 1's emeralds and topazes and run
   2's sapphires and emeralds read as crosses
2. Run 1 never rendered a close-up of the jewels the prompt centers on. One
   would have shown the fused bezels plainly
3. Run 1 finished with the overlapping settings its final message names although
   the skill's done line asks for no flaw a review can name
4. Run 2 set the diamonds as bare 2-cell clear cubes although SKILL.md warns
   that clear glass reads faintly and needs a darker rim

Colors: Both runs' square legs and posts read as nearly flat oak. Run 1's fused
bezels form one flat mustard gold mass. Run 2's gilt coat on the crest's end
faces forms flat mustard slabs in the right view.

### 2. Lantern

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | good    | 4      | 0      | 2.2     | 136,439 at 2.5 mm |
| 2   | fair    | 7      | 0      | 3.3     | 21,271 at 5 mm    |

Both runs built a dark iron lantern with a stepped pyramid roof, a hanging ring,
and a lit candle on a brass dish. Run 1 reads better because its 2.5 mm voxels
give a drippy candle and a teardrop flame with a halo, where run 2's 5 mm flame
stays blocky above a 4 cm stub that reads as a tea light. Unlike round 4, where
run 1's panes stayed blank white around the candle, run 1's halo and emissive
floor disc make the lantern glow.

Failures:

1. Both runs, pass 1: the amber glass darkened the chamber. It muddied run 1's
   interior and hid run 2's candle
2. Run 1, pass 1: the mid bars and the front-right post covered the small flame
   in the hero
3. Both runs, final pass: the front-right post crosses the flame in the hero
   although each final message says nothing hides it. It grazes run 1's flame
   after a 1.25 cm candle offset and splits run 2's down the middle after
   thinner posts and a lower candle
4. Run 1, final pass: the speckled outer floor ring that replaced a noise
   pattern in pass 4 reads as scattered rust flecks
5. Run 2, pass 2: the flame came out washed-out white and took two more passes
   of color tuning
6. Run 2, pass 4: the new brass latch went on the front-right post, directly in
   front of the candle in the hero
7. Run 2, pass 7: lowering the candle top to 7.5 cm left a 4 cm stub that reads
   as a tea light

Lacked:

1. Both runs wanted a light source that lights nearby surfaces. Run 1 painted
   emissive discs on the lantern floor where run 2 gave the glass a faint
   emissive
2. Both runs wanted the report or renders to say what hides a part from a view.
   Both judged the post over the flame by eye and called it solved

Missed in the skill:

1. Both runs left the candle on the hero diagonal behind the front-right post
   although SKILL.md says nothing sits between the prompt's detail and the hero
   corner. A larger offset, a `rotate` of the lantern, or a `--view-orbit hero`
   override would have cleared it
2. Run 2 stayed at 5 mm with 1-voxel posts and a 2-voxel wick although SKILL.md
   says every named feature spans at least 2 voxels and the smallest feature
   sets the voxel size. The flame stayed blocky

Colors: Run 1's iron is one near-black brown across base, posts, roof, and cap.
Its top view reads as a dark featureless square where the roof steps barely
separate. Both runs' glass reads as a flat panel, cream in run 1 and a uniform
peach fill in run 2. Run 2's flame is pale orange without a bright core. Noise
mottles its iron into gray camouflage blotches rather than worn metal.

### 3. Chest

| Run | Verdict | Passes | Failed | Minutes | Voxels          |
| --- | ------- | ------ | ------ | ------- | --------------- |
| 1   | good    | 5      | 0      | 3.2     | 211,894 at 1 cm |
| 2   | good    | 3      | 0      | 2.2     | 95,130 at 1 cm  |

Both runs built an iron-bound oak chest at 1 cm with a red velvet lining under a
barrel lid pivoted open 108 degrees over a domed gold hoard with three gems and
a floor spill. Run 1 reads better because its dark planks, gray iron, and bright
gold contrast sharply, where run 2's pale tan oak sits close to its mustard
gold. Unlike round 4, where run 1's rimmed coins read as round discs, run 1's
coins read as yellow flakes.

Failures:

1. Run 1, pass 3: the new floor spill left one coin detached until pass 4 moved
   it to rejoin the pile
2. Run 2, pass 1: the coins were tiny crosses until pass 2 enlarged them
3. Run 2, pass 2: shrinking the hasp left it detached from the lid until pass 3
   reattached it
4. Run 1, final pass: the coins read as yellow flakes rather than round discs.
   The floor spill reads as a few tiny blobs
5. Run 2, final pass: the spill over the rim reads as a gold lump on the lock
   plate beside a separate floor mound rather than a cascade

Lacked:

None

Missed in the skill:

1. Run 1 gave the velvet lining and all the iron one flat color each although
   the skill's color check calls for shades or noise on big single-color faces.
   The lid lining reads as a plain red slab with stair-step bands
2. Run 2 set pale tan oak beside mustard gold although the skill says
   neighboring materials need colors far apart in lightness or hue. The gold the
   prompt centers on does not pop
3. Run 2's five-step gold `shades` ramp reached near-cream and scattered
   off-color cells through the hoard and floor mound. The hero review passed
   over them

Colors: Run 1's lid lining is one flat red with stair-step bands. Its iron frame
is one flat slate gray. Run 2's pale oak sits close in hue to the gold.
Cream-white specks dot its hoard and floor spill like bone or paper. Its loose
floor coins are one flat pale yellow.

### 4. Sword

| Run | Verdict | Passes | Failed | Minutes | Voxels           |
| --- | ------- | ------ | ------ | ------- | ---------------- |
| 1   | good    | 2      | 0      | 1.3     | 6,332 at 5 mm    |
| 2   | good    | 4      | 1      | 2.1     | 53,532 at 2.5 mm |

Both runs built a point-up longsword with its flat to the front, a gold
crossguard, a banded grip, and a gold disk pommel holding a ruby. Run 2 reads
better because at 2.5 mm it gives the blade a six-sided section with a sunk
fuller, a guard sweeping up to round finials, and a faceted ruby, where run 1's
5 mm blade is a flat 2-voxel sheet with a painted fuller. Unlike round 4, where
both blades stayed flat at 5 mm and run 2's finials sank into the guard, run 2's
blade section and finials read clearly.

Failures:

1. Run 2, pass 1: the fuller carve cut a see-through slot down the blade because
   its box spanned the full thickness
2. Run 2, pass 3: the close-up render failed on the new error that asks for
   meters beside `--view-look-at`
3. Both runs, final pass: the tip climbs in coarse stair steps
4. Both runs, final pass: the ruby reads in the close-up but stays small in the
   hero. Run 1's shows as a few red pixels
5. Run 1, final pass: the guard's ball tips barely bulge past the plain straight
   bar

Lacked:

None

Missed in the skill:

1. Run 1 kept 5 mm voxels for a 1 cm blade although SKILL.md says the smallest
   feature sets the voxel size. The blade has no cross-section and its fuller is
   only paint
2. Run 1 set 3-voxel-radius ball tips on a 2.5-voxel bar although SKILL.md warns
   that small rounds read as blocks. They vanish in the hero
3. Run 2 paired `--view-look-at` with a `fit` orbit distance although SKILL.md
   says an orbit about a look-at point takes meters. The close-up render failed
   in pass 3
4. Run 2 set silver edges beside steel although SKILL.md's color checks warn the
   pair merges. The session reported the flaw instead of lightening the edge
   color

Colors: Run 1's blade reads as one flat light gray in the hero apart from the
fuller stripe. Its noise gold on the guard and pommel comes out mustard rather
than bright gold. Run 2's blade is one dull mid-gray whose silver edges merge
into it. Its guard and pommel are each one flat gold.

### 5. Tree

| Run | Verdict | Passes | Failed | Minutes | Voxels           |
| --- | ------- | ------ | ------ | ------- | ---------------- |
| 1   | good    | 4      | 1      | 1.4     | 86,451 at 0.1 m  |
| 2   | good    | 7      | 0      | 2.5     | 610,536 at 0.1 m |

Both runs built an oak at 0.1 m with root flare, limbs, and a blended crown of
displaced ellipsoids. Run 1 reads better as a finished model because distinct
sun and shade materials mottle its crown into foliage over six limbs that show
in the front and right views. Run 2 has the stronger oak form with crooked
elbowed limbs under a 16 m lobed crown, but the crown reads as nearly one light
green. Unlike round 4, where run 1's dense dome hid its limbs, run 1's limbs
spread plainly under the crown.

Failures:

1. Both runs, pass 1: the crown's `displace` split off floating leaf crumbs,
   leaving run 1 in 40 pieces and run 2 in 96. Run 1 lowered the amplitude and
   octaves until they fused where run 2 swept seeds until one gave a single
   piece
2. Run 1, pass 1: roots poked below a domed ground until a flat grass disc and a
   carve below it trimmed them
3. Run 1, pass 3: the build failed with
   `cylinder round must be at most the radius and half the length, 0.075, not 0.1`
   on the ground disc. Pass 4 lengthened the cylinder
4. Run 2, pass 1: a short trunk sat buried under a flat crown until later passes
   lengthened it and raised the clusters
5. Run 1, final pass: the crown's top looks flat and boxy like a cut hedge in
   the hero
6. Run 2, final pass: the hero looks down onto the crown and hides most of the
   trunk. The session added a low orbit view instead of moving the camera or the
   crown
7. Run 2, final pass: the tree floats with no ground under it

Lacked:

1. Both runs wanted a voxelize option that drops tiny floating pieces. Run 1
   traded away crown lumpiness for one piece where run 2 ran six voxelizes
   sweeping seeds

Missed in the skill:

1. Run 2 drew the crown's base and sunlit coat from one `shades()` set with
   spread `0.06` although SKILL.md's color checks say neighboring materials need
   colors far apart in lightness or hue. The sunlit tops barely differ and the
   crown reads flat

Colors: Run 1's grass disc reads pale and washed out beside the saturated crown.
Run 2's crown reads as one light green in the hero and top views because base,
sunlit, and shaded leaves share one tight `shades()` spread. Its grain stripes
barely show on the very dark bark.

### 6. Well

| Run | Verdict | Passes | Failed | Minutes | Voxels           |
| --- | ------- | ------ | ------ | ------- | ---------------- |
| 1   | good    | 3      | 0      | 1.8     | 71,554 at 2.5 cm |
| 2   | good    | 2      | 0      | 1.9     | 75,712 at 2.5 cm |

Both runs built a round fieldstone well at 2.5 cm with a stone cap, two posts, a
shingled gable, a windlass with a rope coil and an iron crank, and a hooped
bucket on a rope. Run 1 reads better because its gable turned to face front
frames the hanging bucket over blue water in the front view, where run 2's ridge
along `x` shows a solid roof slab over a dark shaft whose water barely reads.
Unlike round 4, where run 2's hero showed the rope dropping to the bucket over
open water, the eave hides most of the bucket in both final heroes.

Failures:

1. Run 1, pass 1: the gable ridge along `x` hid the windlass until pass 2 turned
   the gable to face front
2. Run 1, pass 1: the rope stopped short of a too-thin bucket handle and left
   the bucket and two handle fragments floating as 4 pieces. Pass 2 thickened
   the torus tube
3. Run 2, pass 1: the moss `coat` with `sides: ["+y"]` on the wall base reported
   0 cells. Pass 2 painted noise moss over the lower wall shell
4. Both runs, final pass: the eave covers most of the bucket in the hero. Run 1
   added a low orbit view where run 2's raised roof and shorter eaves still left
   only the bucket's top showing

Lacked:

None

Missed in the skill:

1. Run 2 put moss on the vertical wall base with a `coat` toward `+y` although
   SKILL.md describes `coat` as recoloring faces exposed toward a listed side.
   The moss came out at 0 cells in pass 1

Colors: Both curved walls show vertical light and dark column streaks from the
voxel staircase. Run 1's king post is one flat dark tone. Run 2's cap reads
mostly as a gray band dotted with mortar because its large stone cells span a
4-voxel ring. Its water is a dark slab nearly invisible down the shaft.

### 7. Robot

| Run | Verdict | Passes | Failed | Minutes | Voxels         |
| --- | ------- | ------ | ------ | ------- | -------------- |
| 1   | good    | 2      | 0      | 1.5     | 6,190 at 2 cm  |
| 2   | good    | 2      | 0      | 2.0     | 10,717 at 2 cm |

Both runs built a boxy toy robot at 2 cm with a visor, cyan eyes and the head
and each arm as parts on neck and shoulder pivots. Run 2 reads better because
its blue tin-toy body carries readable two-finger claws, bolted ears and a chest
screen. A posed render shows its head and arms turning cleanly. Run 1's orange
robot is tidy, but its arms read spindly and its claws merge into a block from
the front. Unlike both round 4 runs, run 1 leaves its joints unproven by any
posed render.

Failures:

1. Both runs, pass 1: the antenna stalk voxelized to 0 cells and left the tip
   floating. Pass 2 thickened or snapped it
2. Run 1, pass 1: the alert light sat at `0 exposed` behind the vents. Pass 2
   moved it into view
3. Run 2, pass 1: the eyes, teeth and vents voxelized to 0 cells because their
   faces sat at off-grid depths such as 0.13 and 0.145. The visor stayed blank
   until pass 2 snapped every detail to the grid
4. Both runs, final pass: the elbow reads as a plus sign in the right view. Run
   2's 4 cm antenna sphere also renders as a red plus sign
5. Run 1, final pass: the arms are thin 3-voxel strips against a wide torso. The
   claws merge into one solid gray block from the front
6. Run 2, final pass: the belt reaches 0.22 m in x and pokes into the forearms
   that start at 0.20 m. The arms hug the torso with no gap

Lacked:

1. Both runs: a render-time way to preview a part turned about its pivot. Run 2
   found `vxl node set rotation` through `--help` and rendered a posed copy. Run
   1 shipped without seeing a joint turned

Missed in the skill:

1. Both runs, pass 1: the skill asks that details be sized in multiples of the
   voxel constant, yet both first drafts sized or placed details off the grid.
   Run 1's antenna stalk fell under one voxel. Run 2 lost its face and chest
   details for a whole pass

Colors: Run 1's dark feet, hips and chest panel read as flat single-tone slabs.
Run 2's near-black knees, hips, claws and panel stay flat single tones. Its
orange feet and belt do too.

### 8. Cottage

| Run | Verdict | Passes | Failed | Minutes | Voxels          |
| --- | ------- | ------ | ------ | ------- | --------------- |
| 1   | good    | 3      | 0      | 1.7     | 464,904 at 5 cm |
| 2   | good    | 3      | 0      | 2.0     | 473,305 at 5 cm |

Both runs built a spotted red mushroom cottage at 5 cm with a lathed cream stem,
a round plank door, glowing round windows, a chimney and stepping stones. Run 2
reads better because its door meets the ground over a stone doorstep and its
windows sit low enough to show fully in the hero. Run 1's door floats above the
grass under a cap that looks heavy for its stem. Unlike round 4's run 1, both
runs flank the door with windows in the front view.

Failures:

1. Both runs, pass 1: the cap rim hid the windows in the hero. Run 1 raised the
   cap and stem 0.7 m, then lowered the windows 20 cm in pass 3. Run 2 lowered
   the windows from 1.7 m to 1.5 m
2. Run 2, pass 2: the toadstool caps were lathed at the origin and sat buried
   inside the house at `0 exposed`. The `set()` spots became six floating single
   voxels for 7 pieces until pass 3 translated the caps and swapped the spots
   for a speckle coat
3. Run 1, final pass: the full-circle door floats about 8 cm above the grass on
   a strip of wall. The session named it in its summary and left it
4. Both runs, final pass: the hero shows the door at a steep angle. In run 1
   only one window shows there
5. Run 2, final pass: the frames of the windows 110 degrees either side of the
   door jut past the stem's silhouette as dark slabs in the front and right
   views. The toadstools stand only about 6 voxels tall
6. Run 2, final pass: the door and window recess carves report `0 kept` because
   later frames refill them. The session left the redundant steps in

Lacked:

None

Missed in the skill:

1. Run 2: `lathe` revolves about the origin unless given its `center` option or
   a `translate`. Missing that put the pass 2 toadstool caps inside the house
   and cost a pass

Colors: Run 1's stepping stones are one flat stone gray. Its stem reads
near-uniform cream apart from lathe terracing rings. Its door and window frames
share one flat dark brown. Run 2's doorstep and path stones are one flat stone
shade, so the large half-disc doorstep reads as a plain gray slab in the hero.
Its stem is a near-uniform cream.

### 9. Potion

| Run | Verdict | Passes | Failed | Minutes | Voxels           |
| --- | ------- | ------ | ------ | ------- | ---------------- |
| 1   | good    | 3      | 0      | 1.6     | 66,552 at 2.5 mm |
| 2   | good    | 2      | 0      | 1.0     | 17,220 at 4 mm   |

Both runs built a round glass flask with a blue-gray lip, an emissive potion and
a cork. Run 2 reads better because its potion is a saturated true red under a
tapered mottled cork. Run 1 at 2.5 mm has the smoother silhouette, but its
potion reads salmon pink. Run 2's red ends the pink that both round 4 runs
showed.

Failures:

1. Both runs, pass 1: bluish glass turned the potion mauve. Pass 2 cleared the
   glass
2. Run 1, passes 1 and 2: the report showed the bubbles at `0 exposed`. Pass 2
   re-placed them and pass 3 removed them
3. Run 2, final pass: the enclosed bubbles sit at `0 exposed` and show only as
   two or three faint pink pixels
4. Run 1, final pass: the potion reads salmon pink rather than red. The
   unrequested twine ring below the lip looks more like a tan gear than wrapped
   cord
5. Run 1, final pass: the opaque blue lip collar looks thick and heavy against
   the thin glass walls
6. Run 2, final pass: at 4 mm the bulb is coarser and the liquid wall shows
   staircase rings in the front view. The opaque blue lip ring cuts across the
   middle of the cork

Lacked:

None

Missed in the skill:

1. Run 1: Checks color item 4 says a pale glow takes a darker `baseColor` or a
   lower `emissiveStrength`. The session raised `emissiveStrength` to 0.8
   instead and the potion stayed salmon pink
2. Both runs: Checks item 3 flags steps at `0 exposed`, yet both kept bubbles
   the report showed buried. Run 1 spent a third pass removing them. Run 2
   shipped them as a few faint dots

Colors: Run 1's potion side reads as uniform salmon pink with staircase contour
streaks. Run 2's potion side reads as one flat red with ring-shaped staircase
banding in the front view. In both runs the glass body is one flat pale blue in
the orthographic views.

### 10. Cart

| Run | Verdict | Passes | Failed | Minutes | Voxels           |
| --- | ------- | ------ | ------ | ------- | ---------------- |
| 1   | good    | 2      | 0      | 1.4     | 16,064 at 2.5 cm |
| 2   | good    | 2      | 0      | 1.5     | 15,944 at 2.5 cm |

Both runs built a plank wagon at 2.5 cm with stakes along the walls and four
spoked wheel parts with walnut rims. Run 1 reads better because its eight spokes
read cleanly and the hero shows the whole cart down to the T-handled draw bar.
Run 2's board-and-batten bed with iron straps carries more detail, but its
spokes clump at the hub and its pole faces away from the hero. Run 1's spokes
read cleanly where round 4's spokes fragmented or crowded into a dark mass.

Failures:

1. Run 1, pass 1: ten spokes made the hub read as a star. Pass 2 cut them to
   eight and calmed the camouflage-like rail noise
2. Run 2, pass 1: the grain ran in vertical stripes across the walls. Pass 2
   rebuilt each wall as three horizontal boards with walnut seams
3. Run 1, final pass: the diagonal spokes stair-step. The wheel tops rise well
   above the floor and cover the lower third of the bed sides
4. Run 2, final pass: the 2-voxel spokes clump around a large square oak hub and
   make the wheels look busy in the hero
5. Run 2, final pass: the hero sees the pole at `-x` only as a stub behind the
   bed. The stakes hang two voxels below the bed bottom

Lacked:

None

Missed in the skill:

1. Run 2: SKILL.md asks that the prompt's detail face `+z` or `+x` toward the
   front-right-top hero, yet the session put the pole and ring at `-x`. The hero
   hides most of the pole behind the bed

Colors: Run 1's draw bar, handle and spokes use single flat oak shades. Its
undercarriage and tongue read a little plastic next to the banded walls. Run 2's
iron corner straps and stake caps are flat uniform gray slabs. In the right view
its bolsters read as one dark brown mass under the bed.

### 11. Guitar

| Run | Verdict | Passes | Failed | Minutes | Voxels          |
| --- | ------- | ------ | ------ | ------- | --------------- |
| 1   | good    | 2      | 0      | 3.4     | 187,762 at 3 mm |
| 2   | fair    | 2      | 0      | 2.8     | 34,040 at 5 mm  |

Both runs built a hollow acoustic guitar with a rosette sound hole opening onto
a dark interior, a teardrop pickguard and six strings. Run 1 reads better
because its 3 mm grid keeps all six strings distinct in every view. Run 2 at 5
mm runs its strings to the tuners, but in the hero they merge into two ribbons.
Run 1's strings stay distinct in the hero, where both round 4 runs fused them
into one band.

Failures:

1. Run 2, pass 1: a 16th fret sat past the fretboard's end and floated as a
   second piece. Pass 2 dropped it to 15 frets
2. Run 2, final pass: at 5 mm with one-voxel gaps, the six strings merge into a
   bronze ribbon and a gray ribbon in the hero. The session saw this in the pass
   1 hero and called the model done
3. Run 2, final pass: the string runs from nut to posts come out as jagged
   staircases that clutter the headstock. The headstock tip is a crenellated V
   and the untapered 65 mm neck reads wide
4. Run 1, final pass: the strings stop at the nut instead of reaching the tuners
5. Run 1, final pass: the near-circular lower bout and sharp notched waist give
   the outline a snowman look. The fretboard taper jogs one voxel halfway up the
   neck and the heel pokes out above the body in the back view

Lacked:

None

Missed in the skill:

1. Run 1: the session stopped the strings at the nut for fear a one-voxel
   diagonal breaks apart. The Resolution section says a tilted stroke holds at
   about 2 voxels, and a 6 mm `polyline` per string at 3 mm would have reached
   the tuner posts
2. Run 2: the string tails use a 6 mm `polyline` at 5 mm voxels, under the
   2-voxel width the Resolution section asks for tilted strokes. They render as
   jagged staircases
3. Run 2: the skill's checks ask that no view show a flaw a review can name, yet
   the session accepted a hero where six strings read as two ribbons. Voxels of
   3 to 4 mm or wider gaps would have kept them apart

Colors: Run 1's tuner buttons and posts are plain gray cubes. Its heel block
reads as a flat lump on the back. Run 2's strings read as two flat untextured
bands of bronze and gray in the hero. Its sound hole interior is one flat dark
tone.

### 12. Candelabra

| Run | Verdict | Passes | Failed | Minutes | Voxels         |
| --- | ------- | ------ | ------ | ------- | -------------- |
| 1   | good    | 4      | 0      | 2.5     | 43,920 at 4 mm |
| 2   | good    | 4      | 0      | 2.1     | 35,717 at 4 mm |

Both runs built a wrought-iron candelabra at 4 mm with a twisted square stem,
painted corner ridges, scrolled arms and lit candles. Run 2 reads better by a
narrow margin because its five curling arms, spaced 72 degrees apart, carry all
five candles. Run 1 is the handsomer piece with crisp extruded volutes and
haloed flames, but it gives four arms on the axes plus a center candle. Run 2
puts the fifth candle on an arm, where both round 4 runs stood it on a center
column.

Failures:

1. Both runs, pass 1: the stem twist did not read. Run 1 widened the stem to 10
   cells and painted its corners in pass 2. Run 2 painted its corners, but the
   twist read only after pass 3 raised it from 600 to 1,400 degrees per meter
2. Run 2, pass 1: the model came out in 13 pieces because two flames floated
   above their wicks and ten single-voxel highlight scraps sat on the cup rims.
   Pass 2 thickened the wicks and closed the cup profile
3. Run 1, final pass: four arms on the x and z axes carry four candles while a
   fifth stands on a central cup. The prompt asked for five candles on curling
   arms
4. Run 1, final pass: the twist reads mostly through paint, with ragged sawtooth
   stripes over a jagged silhouette. The bulbous flames are wider than the
   candles and look like bulbs
5. Run 2, final pass: the near-black iron merges arms, scrolls and stem in the
   hero. The off-axis torus tubes look beaded and lumpy
6. Run 2, final pass: the flames are small stepped blocks with no halo at
   `emissiveStrength` 1. They look colored more than lit

Lacked:

None

Missed in the skill:

1. Run 2: pass 2 lowered the flames' `emissiveStrength` from 1.5 to 1 though
   SKILL.md says strengths of 2 to 4 add a halo. The final flames have no glow
2. Run 2: SKILL.md warns that two dark tones side by side merge, yet the final
   kept near-black `#3A3634` iron under a barely lighter coat. The arms and
   scrolls lose definition against the stem in the hero

Colors: Run 1's painted stem ridges in `#A39C90` sit far lighter than the dark
bar. The stem reads as a barber-pole stripe rather than forged highlights. Its
foot is a broad uniform gray. Run 2's near-black iron merges arms, cups and stem
in the hero. Its `+y` highlight coat speckles the cup rims and foot rings in
place of real highlights.

### 13. Knight

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | good    | 1      | 0      | 2.5     | 2,340 at 5.625 cm |
| 2   | good    | 3      | 0      | 3.8     | 2,199 at 6.25 cm  |

Both runs built a knight exactly 32 voxels tall in PICO-8 colors, with parts for
the head, arms, and legs, and ended in one piece. Run 2 reads better because
lavender lames on its pauldrons, fauld, elbows, and helm rim articulate the
plate around a carved T visor and a bent arm gripping the sword. Run 1's
front-facing blue kite shield reads well, but its flat grey armor and tacked-on
plume leave a plainer figure.

Failures:

1. Run 1, final pass: the plume reads weakly. A red brick sits on the crown with
   a striped red and purple slab hanging off the back of the helm
2. Run 1, final pass: the protruding yellow face cross covers the visor. It
   reads more as a mask than a helm detail
3. Run 2, pass 2: a light grey shield boss read as a blob under the chevron.
   Pass 3 deleted it
4. Run 2, final pass: the shield hangs on the outer side of the arm. The front
   view shows only a thin navy edge
5. Run 2, final pass: the pink-topped torus plume reads as a pink tuft
6. Run 2, final pass: the yellow buckle sits directly under the yellow cross.
   The cross appears to run through the belt

Lacked:

None

Missed in the skill:

1. Run 1 left the helm sides and greaves one flat light grey although the color
   advice says a large single-color face reads unfinished. A `bands` of two
   palette greys, as on its faulds, would have broken them up
2. Run 2 drew the shield chevron as a `polyline` stroke 1.6 voxels across
   although the Resolution section asks about 2. The stroke blobbed and cost a
   pass to redo as rects

Colors: Both runs leave the helm sides and shins as flat light grey faces. Both
top views read mostly white from the `+y` coats. Both tabards stay large flat
red panels, front and back in run 2.

### 14. Pocket watch

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | good    | 4      | 1      | 3.8     | 229,191 at 0.4 mm |
| 2   | good    | 7      | 3      | 5.8     | 303,330 at 0.4 mm |

Both runs built an open-dial gold watch facing `+z` at 0.4 mm voxels, with
alternating flat and edge torus links along a Bezier to a T-bar. Run 2 reads
better because its skeleton movement of gold, rose-gold, and steel wheels, a
ratcheted barrel, a pallet fork, striped silver bridges, and ruby jewels makes
the strongest front view. Run 1 holds four cleaner wheels and a longer chain,
but its watch fills only a third of the hero.

Failures:

1. Run 1, pass 1: the build failed with
   `center cap: cylinder round must be at most the radius and half the length, 0.0002..., not 0.0004`
2. Run 1, pass 2: the T-bar floated, while the escape wheel and minute hand left
   single stray cells for 5 pieces. Pass 3 fixed all of them
3. Run 2, passes 2 to 4: the first draft wrote millimeters into meter arguments.
   Pass 2 failed with
   `case: cylinder round must be at most the radius and half the length, 0.0042, not 2.4`,
   and passes 3 and 4 hit the grid cap
4. Run 2, pass 5: the report showed 17 pieces, 15 of them correctly interlocked
   chain links. The session read them as links not touching and respaced them
   until they fused
5. Both runs, final pass: the whole chain reports as one piece with the watch.
   Run 1's edge and flat links shared cells from its first build
6. Both runs, final pass: the chain stands rigid with its T-bar in mid-air. Run
   1's arches as an inverted U, while run 2's short chain of about seven links
   sticks straight up
7. Run 2, final pass: the bridges cover much of the gearing in the hero and
   close-up. The movement reads as busy clutter

Lacked:

1. Both runs wanted to place copies along a curve. Run 1 sampled a cubic Bezier
   by arc length, while run 2 sampled a quadratic one
2. Both runs wanted a gear profile. Each unioned `repeatPolar` teeth onto a
   circle and subtracted spoke windows
3. Run 2 wanted the piece lines to tell an interlocked link from a floating
   shape. It took 15 correctly separate links for a bug

Missed in the skill:

1. Run 2 fused a correctly interlocked chain because the skill says a second
   piece means a shape floats
2. Run 2 wrote millimeters into `round` and shape arguments although the
   Coordinates section counts meters. The mix cost three failed passes and a
   refactor
3. Both runs set blued hands beside neighbors of close hue although the color
   rules ask neighboring materials for distinct hue or lightness. Run 1's navy
   hands sit on a near-navy plate, while run 2's blend into the hairspring and
   screws. Run 2 named the problem and left it

Colors: Both runs' `mat.gold` reads olive-brown in the front, right, and top
views and mustard-yellow in the hero. Neither session remarked on the cast. Run
1's `cells` pattern on the steel plate reads as blotchy camouflage.

### 15. Dragon

| Run | Verdict | Passes | Failed | Minutes | Voxels          |
| --- | ------- | ------ | ------ | ------- | --------------- |
| 1   | good    | 6      | 1      | 6.3     | 348,222 at 1 cm |
| 2   | fair    | 6      | 1      | 8.3     | 201,466 at 2 cm |

Both runs coiled a red dragon in a ring on a gold hoard from `roundCone` chains
along a hand-sampled Catmull-Rom spline. Run 1 reads better because its hero and
top views show a sleeping dragon at once, with a tucked head, ivory horns, a
spade tail, and folded wings with finger bones beside a silver crown, goblet,
and planted sword. Run 2's folded wing reads as a maroon shell over the back.
Its head is a small blocky wedge in the hero, while its crown and goblet vanish
into the gold.

Failures:

1. Run 1, passes 3 to 6: four voxelizes chased single floating spine and
   wing-claw voxels by nudging cone tip radii and lean factors
2. Run 1, final pass: from the front the dragon is a squat red loaf with little
   neck or head separation
3. Run 1, final pass: the horns render as oversized stepped slabs in the
   close-up
4. Run 1, final pass: the pile reads as one noisy dome. Its coins show as
   lighter flecks rather than coins
5. Run 2, pass 1: the build failed with
   `dragon/horns: gradient to must be past from 0.02, not -0.18`. The session
   swapped the range
6. Run 2, final pass: four 1-voxel spike and wing-bone specks float off the
   dragon. The closing message called all 16 extra pieces scattered coin and gem
   voxels
7. Run 2, final pass: the folded wing reads as a shell or blanket over the back
   rather than a wing
8. Run 2, final pass: a tan belly patch shows as a beige blob inside the ring

Lacked:

1. Both runs wanted a sweep or tube along a curve. Each sampled a Catmull-Rom
   spline and chained `roundCone`s between the samples
2. Run 2 wanted a frame along a path to attach legs, wings, spikes, and belly
   plates. It derived tangent, up, and side vectors by hand

Missed in the skill:

1. Run 2 set a gold crown and goblet on a gold pile although the color checks
   warn that such colors merge. Both are near-invisible in every view
2. Run 2 shipped four floating dragon specks although the skill reads a second
   piece as a float and asks that every extra piece is intended

Colors: Run 1's gold pile reads olive-brown in the front and right views. Its
floor coins show as flat bright squares. Run 2's dark red wing bones sit too
close to the maroon membrane for the fingers to read in the hero. Its near-black
spikes read as holes.

### 16. Bridge

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | good    | 2      | 0      | 3.1     | 732,790 at 0.25 m |
| 2   | good    | 2      | 0      | 3.9     | 830,445 at 0.25 m |

Both runs built a stone suspension bridge on a diorama at 0.25 m voxels, with
parabolic cables of chained capsules and hangers that step down toward mid-span.
Run 2 reads better because its masonry towers stand in the river on cutwater
piers over a red lattice-girder deck with railings, trees, boulders, and strata.
Run 1 has the stronger silhouette from its tall towers and deep red sag, but its
hangers read as a picket fence and its anchorages pinch the road. Both runs
worked at 0.25 m, where round 4 took 0.1 m and 5 cm.

Failures:

1. Run 2, pass 1: displaced tree crowns shed single voxels as 13 pieces. Pass 2
   lowered the displacement
2. Run 1, final pass: the anchorage blocks overlap the road edges and pinch it.
   The session saw it and spent its one revision on the water color
3. Run 1, final pass: the hangers are 2-voxel-deep light grey bars. They read as
   a heavy picket fence, worst on the side spans
4. Run 1, final pass: the towers stand on the banks rather than in the river
5. Run 2, final pass: the towers rise only about 10 m above the deck. The sag
   reads shallower than run 1's

Lacked:

None

Missed in the skill:

1. Run 2 gave the cables, hangers, posts, and rails one near-black material
   although the skill asks neighboring materials for colors far apart. The
   hardware merges into a busy black comb in the hero

Colors: Run 1's asphalt stays one flat dark tone over the long span. Its pass 2
water turned a murky navy that reads less like water than pass 1's teal. Run 2's
cream caps on the towers, anchorage housings, and portal trim read as blank
rectangles in the top view. Its black hardware merges against the red girder.

### 17. Chess set

| Run | Verdict | Passes | Failed | Minutes | Voxels               |
| --- | ------- | ------ | ------ | ------- | -------------------- |
| 1   | good    | 4      | 0      | 4.2     | 2,284,160 at 1.25 mm |
| 2   | good    | 3      | 0      | 3.8     | 2,419,424 at 1.25 mm |

Both runs lathed five piece kinds, sculpted a knight, and placed all 32 pieces
in a correct starting position with a1 dark and each queen on her color at 1.25
mm. Run 2 reads better because its knights read as horse heads in profile, with
a muzzle, split ears, a mane ridge, and an eye and nostril painted in the
opposite color. Run 1 has the cleaner board, but its knights read as pointed
hoods or slabs from the front and hero. Unlike round 4, where run 2 stayed at
2.5 mm, both runs halved the voxel size to 1.25 mm.

Failures:

1. Run 1, pass 1: the knight was a flat extruded slab. The close-up caught it
2. Run 1, final pass: the knight reads as a pointed hood or narrow slab from the
   front and hero. The session named this in its limitations and called it
   finished
3. Run 2, pass 1: the bishop slit cut the mitre tips off as 4 floating pieces of
   76 voxels each. Pass 2 confined the slit
4. Run 2, pass 2: the offset-rounded knight still read as a blob. Pass 3 redrew
   its outline

Lacked:

None

Missed in the skill:

1. Both runs' side rails show grain that reads wrong although the skill says
   each board takes the axis it runs along. Run 1's one frame box takes grain
   along x, leaving cross-grain rings on the side rails. Run 2 built its z-axis
   rail grain 0.2 m from the origin, so the outer faces read as one broad ring
2. Run 1 carved the knight's eyes as 6-cell pits that vanish in every render. A
   contrasting paint would have shown them
3. Run 2 warped its frame grain by 0.004 m, over 3 cells. The streaks broke into
   blotches on the edge faces

Colors: Both runs' ebony pieces lose nearly all turned detail in the hero. Run
1's square noise reads as blotchy stain rather than wood grain. Run 2's frame
edges read as camouflage and its zebra-striped squares look busy. Its ivory
noise adds grey patches that look dirty in the close-up.

### 18. Ramen stand

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | good    | 3      | 0      | 3.1     | 532,652 at 2 cm   |
| 2   | good    | 2      | 0      | 4.1     | 277,919 at 2.5 cm |

Both runs built a stall under a rooftop pink RAMEN sign of `polyline` strokes in
a cyan frame, with a katakana RAMEN blade sign on the side, noren, lanterns,
three red stools, and bowls on the counter. Run 1 reads better because its
lettering is cleaner and its counter close-up shows white bowls with broth, egg,
nori, and chopsticks beside hollowed stockpots. Run 2 adds more grit with rust,
hazard-striped posts, an AC unit, cables, and a lit interior. Its bowls read as
red blobs and its steam floats as a second piece.

Failures:

1. Run 1, pass 1: an edit before the first build deleted the planned rooftop
   cable
2. Run 1, pass 1: 3 stool footrings and 4 steam puffs floated as 8 pieces. Pass
   2 added cross bars and a `smoothUnion` steam column
3. Run 1, final pass: the steam reads as grey stone columns
4. Run 2, pass 1: the antenna mast cylinder of radius 0.0125 m, half a voxel,
   missed every cell. Its crossbars and tip floated among 7 pieces until pass 2
   made it a 2-cell box
5. Run 2, final pass: the steam above the stockpot stays a detached second
   piece. The session called it intentional
6. Run 2, final pass: the dark lacquer bowls with red rims and single-cell yolks
   and scallions read as red blobs rather than ramen
7. Run 2, final pass: the katakana sign garbles in the hero. Its closing n still
   reads close to so

Lacked:

1. Both runs wanted a text or glyph operation. Each built RAMEN and the katakana
   from hand-placed `polyline` strokes about 2 cells wide

Missed in the skill:

1. Run 2 waved its floating steam through although the skill reads a second
   piece as a float and asks that every extra piece is intended. A short
   `smoothUnion` column into the pot would have joined it
2. Run 2 made its egg yolks and scallions single cells although the skill asks
   about 2 voxels for small details. The toppings do not read

Colors: Run 1's side and back walls read as near-black slabs despite banded
shades. Its puddles are a flat emissive purple with no wet look, while its noren
stay one flat red. Run 2's noren are flat red and its fascia flat black. Its
dark lacquer bowls merge with their contents into red-brown blobs, while its
roof rust lies in large flat orange-brown splotches.

### 19. Dungeon kit

| Run | Verdict | Passes | Failed | Minutes | Voxels             |
| --- | ------- | ------ | ------ | ------- | ------------------ |
| 1   | good    | 2      | 0      | 3.3     | 236,297 at 5 cm    |
| 2   | good    | 2      | 1      | 2.5     | 206,838 at 6.25 cm |

Both runs built a four-tile kit of floor, wall, corner, and arched doorway on a
2 m grid, then assembled a two-door room that imports it. Run 1 reads better
because its brick courses and flagstone joints continue across tile seams,
though its corner pillars jut 10 cm into the room. Run 2's cell tiles carry a
true L corner, but every tile repeats one stone pattern with a seam at each 2 m
joint. Neither room shares objects between repeated tiles where round 4's run 1
instanced its tiles under `--frame local`.

Failures:

1. Run 2, pass 1: the room build stopped with
   `SyntaxError: The requested module './kit.ts' does not provide an export named 'Kind'`
   because the session imported a type as a value. Pass 2 imported it as
   `type Kind`

Lacked:

1. Rotating a placed part: both runs built each rotated tile as a fresh part by
   rotating every shape
2. Material property values in the report: both runs decoded the `walkable`
   values from the raw voxj by hand

Missed in the skill:

1. Run 1 voxelized the room under the default `--frame world` although the skill
   says places of one part share an object under `--frame local`. The room
   writes 20 objects where its 6 floors, 6 walls, and 4 pillars could share one
   each
2. Run 2 built a fresh part for each room cell through `tile(kind, rot)`
   although the skill places one part at each spot of a repeated prop. Even
   `--frame local` could not share its 12 objects

Colors: Run 2's floors and walls repeat one identical pattern per tile. Seams
show at every 2 m joint. Dark bands cross the tile rows in the top view.

### 20. Crane

| Run | Verdict | Passes | Failed | Minutes | Voxels          |
| --- | ------- | ------ | ------ | ------- | --------------- |
| 1   | good    | 3      | 0      | 3.3     | 51,455 at 0.1 m |
| 2   | good    | 3      | 0      | 3.8     | 409,218 at 5 cm |

Both runs built a crawler crane in three passes as a four-level part tree of
root, upper works, boom, and hook with every pivot on its hinge. Neither run
reads clearly better. Run 1 keeps the A-frame, luffing lines, and pendants for
the more complete crane at rest, but its pendants tear off the cab bridle once
the boom tilts. Run 2's 0.05 m grid gives a crisper boom, cab, and hook, but
without rigging its 18 m boom stands unsupported.

Failures:

1. Run 1, pass 1: the session wrote the track and side-frame profiles as (z, y)
   for an x extrude. The tracks stood 6 m tall on end
2. Both runs, pass 1: parts thinner than the 0.1 m voxel broke up. Run 1's 0.08
   m luffing lines left the model in 30 pieces, while run 2's 0.04 m latch wrote
   0 cells beside a floating 2-voxel hook tip
3. Run 2, pass 1: a paint step overwrote the whole counterweight add to
   `0 kept`. Pass 2 folded the bands into the add

Lacked:

1. A rope that stays attached across two moving parts: run 1's boom pendants
   detach from the cab bridle when the boom tilts. Run 2 left out the A-frame
   and pendants
2. A render of a part turned about its pivot: run 1 reasoned out the pendant
   break without seeing it

Missed in the skill:

1. Run 1 wrote the track profile as (z, y) although `SKILL.md` says an x extrude
   maps the profile's (u, v) to (y, z). The miss cost pass 1
2. Both runs placed parts thinner than a voxel although `SKILL.md` warns such a
   shape can miss every cell center. Run 1's luffing lines and run 2's latch
   each cost a pass

Colors: Run 1's cab windows are a near-black navy that reads as holes rather
than glass. Run 2's noise-shaded engine house reads mottled from above. Its cab
glass is a flat light blue.

### 21. Fish tank

| Run | Verdict | Passes | Failed | Minutes | Voxels          |
| --- | ------- | ------ | ------ | ------- | --------------- |
| 1   | good    | 3      | 0      | 2.8     | 508,288 at 5 mm |
| 2   | fair    | 4      | 0      | 4.6     | 130,680 at 5 mm |

Both runs built a 60 cm black-framed tank at 5 mm with a gravel floor, a castle,
seaweed, bubbles, and at least five fish. Run 1 reads better because its
low-alpha water and `#CFEFFF30` glass keep a vivid read of the red-capped
sandstone castle and five recognizable fish, though its bubbles nearly vanish.
Run 2 adds a backdrop, rocks, a tetra school, and clear bubble columns, but its
`#D6F2FF` glass at transmission 0.92 washes every color blue-gray. Run 1 filled
its tank with alpha-tinted water where neither round 4 run filled the tank or
tried an alpha.

Failures:

1. Run 1, final pass: the bubbles stayed faint after the switch from alpha to
   opaque white. The session named the flaw and stopped
2. Run 2, passes 1 to 4: the glass tint muddied the fish and castle colors in
   the front and right views. No pass fixed it
3. Run 2, final pass: a single stray gravel voxel stays as piece 26. The session
   saw it and left it

Lacked:

1. Exposure past a solid water volume: every interior step in run 1 reads
   `0 exposed` because the water covers it. The check could not show which
   details were truly buried

Missed in the skill:

1. Run 1 stopped with the faint bubbles it had named although the skill counts a
   model done only when no view shows a flaw a review can name
2. Run 2 gave the glass opaque `#D6F2FF` at transmission 0.92 although
   `SKILL.md` documents `#RRGGBBAA` alpha for a `baseColor` and says tinted
   glass tints what sits behind it. The session saw the muddy fish in the front
   view and left them

Colors: Run 1's water reads as one flat pale blue with no depth toward the back
wall. Its near-white bubbles disappear against it. Run 2's front, right, and top
views read through a blue-gray wash that turns the clownfish brown and the
goldfish olive. Its top view loses almost all contrast.

### 22. Wizard tower

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | good    | 3      | 0      | 3.7     | 1,290,016 at 5 cm |
| 2   | good    | 6      | 0      | 4.0     | 863,281 at 5 cm   |

Both runs built a one-piece cobblestone tower at 0.05 m whose spiral stair of
per-step rotated treads climbs to a roof shaped by `bend`. Neither run reads
clearly better. Run 1 is the taller, finer tower with framed and barred windows,
a ground disc, and an indigo hat crooked from every view, but its stone corbels
stack into a jagged column on the silhouette. Run 2 has the stronger stair of
two dense turns behind a continuous iron rail, but its hooked roof sits squat on
the narrow wall over plain window cutouts with no ground. Run 2's hooked roof
reads as a floppy hat where round 4's run 2 read as a swept horn.

Failures:

1. Run 1, pass 1: the finial star floated as a 146-voxel second piece. Pass 2
   joined it with a gold rod
2. Run 1, pass 2: a second bend chained on the whole roof swung the eave down to
   y 11.2. Pass 3 split the roof at the kink with two `halfSpace` cuts and bent
   only the upper half
3. Run 2, pass 1: the top landing pointed the wrong way. The stair brackets ran
   below the ground to y -0.3
4. Run 2, pass 2: a 5 degree tilt on the bent roof pushed a 10-voxel sliver
   below the eave into a second piece. Pass 3 cut the roof at its base with a
   `halfSpace`
5. Run 2, pass 5: the back view showed a blank wall. Pass 6 moved and added
   windows

Lacked:

None

Missed in the skill:

1. Run 1 chained a second bend on the whole roof although `SKILL.md` says a
   second bend keeps only its pivot slice and moves the rest of the first bend's
   arc. The miss tipped the eave and cost a pass

Colors: Run 2's windows read as flat orange rectangles with no frame or darker
rim. Both runs' treads read as a uniform tan in the orthographic views.

### 23. Log cabin

| Run | Verdict | Passes | Failed | Minutes | Voxels          |
| --- | ------- | ------ | ------ | ------- | --------------- |
| 1   | good    | 5      | 0      | 3.6     | 862,364 at 5 cm |
| 2   | good    | 4      | 0      | 4.4     | 930,573 at 5 cm |

Both runs built a round-log cabin at 5 cm with glowing windows, a stone chimney,
a gray smoke plume, pale-blue icicles, and a path dug through the drifts. Run 2
reads better because its cream chinking, four-pane windows, and long chunky
icicles dominate the close-up as the prompt asks. Run 1 sits more deeply buried
under the better smoke gradient, but its 1.5-voxel icicles read as small blue
ticks at hero distance. Run 2 fixed its `0 kept` chinking where round 4's run 1
passed over the same loss.

Failures:

1. Run 1, passes 3 to 5: the session chased one- and two-voxel smoke specks for
   three passes. The last went with a carve of one hand-placed cell tied to the
   seed
2. Run 2, pass 1: the `chinking` step kept 0 cells because the logs buried it.
   Pass 3 thickened the slabs
3. Run 2, pass 2: the smoke split off as a floating 43,521-voxel piece once the
   chimney rose. The ground snow cut the sill snow to 30 kept cells
4. Run 2, final pass: 17 extra pieces of 1 to 20 voxels stay from the roof
   snow's displaced underside and the ground edge. The session waved them off as
   invisible

Lacked:

None

Missed in the skill:

1. Run 2 placed its sill snow as a hand-sized box that the ground snow buried in
   pass 2. A late `coat` with `within` would have followed the trim

Colors: Run 1's sloped roof snow aliases into dense horizontal stripes with
blue-gray noise blotches. It reads like corrugated sheet in the front and top
views. Run 2's smoke spans a narrow mid-gray range that reads flat against the
white. Both runs' ground snow shows dark stair-step speckle in the top view.

### 24. Valley

| Run | Verdict | Passes | Failed | Minutes | Voxels           |
| --- | ------- | ------ | ------ | ------- | ---------------- |
| 1   | good    | 5      | 0      | 8.7     | 186,961 at 0.5 m |
| 2   | good    | 5      | 0      | 11.6    | 237,715 at 0.3 m |

Both runs built a forested valley of snow-capped peaks and a winding river under
`--fill-mode surface`. Run 2 reads better because its 64 m tile at 0.3 m has
merged ridgelines, grass to rock to snow bands, a tarn under a headwall
waterfall, and pines in clumps that thin toward the tree line. Run 1's 100 m
tile at 0.5 m reads cleaner but plainer because its noisy gray cones carry a
thin holed snow crust over a dead flat floor. Both runs ended at one piece by
hand-carving their last crumbs where round 4's run 2 shipped 9.

Failures:

1. Run 1, pass 3: raising the peaks to 40 to 54 m broke the displaced summits
   into 17 pieces of 1 to 25 voxels
2. Run 2, pass 2: the 0.27 m trunks of the smallest pines sampled to 0 cells at
   0.4 m. Six crowns floated beside 8 terrain crumbs
3. Both runs, pass 5: each run removed its last crumb with a carve at hand-coded
   coordinates. Run 1's
   `carve("stray", box([37, 35, -45], [37.5, 35.5, -44.5]))` breaks on any
   terrain or noise edit
4. Run 2, final pass: curled snowy overhangs stay on the back-right summits. A
   thin terrain fin sticks up at the front-left corner where the bounds cut a
   peak

Lacked:

1. The terrain height at a point: both runs placed pines by an analytic height
   that ignores the noise and the smooth-union fillets. Both sank the trunks 3
   to 4 m to hide the gap
2. A step that drops pieces under a size: both runs lowered the noise and
   hand-carved their last crumb by its coordinates
3. What made a crumb: run 1's piece list showed only where each crumb sat. The
   session wanted the noise feature or step behind it
4. Per-place counts under `--frame local`: run 2 could not tell whether its 170
   tree places counted toward the 300,000-voxel budget once or per place

Missed in the skill:

1. Run 2 stopped with the curled snow overhangs it had named although the skill
   counts a model done only when no view shows a flaw a review can name
2. Run 2 gave its strata `bands` a warp of 2.5 at a 1.6 m period. The cut sides
   smear into camouflage blotches rather than the strata the skill's `bands`
   advice aims at

Colors: Run 1's lime grass reads brighter and yellower than the dark pines and
gray rock. Its mountain flanks are one mid-gray. Run 2's snow caps read as flat
white blobs. Its cut-side strata blur into marbled brown and gray camouflage.
Its dark navy river reads heavy beside the bright meadow.

### 25. Village

| Run | Verdict | Passes | Failed | Minutes | Voxels               |
| --- | ------- | ------ | ------ | ------- | -------------------- |
| 1   | good    | 5      | 1      | 8.7     | 2,835,546 at 0.125 m |
| 2   | fair    | 5      | 0      | 9.8     | 427,756 at 0.25 m    |

Both runs ringed a market square holding a roofed well and four stalls with
thatched cottages and set a bespoke stone church on it. Run 1 reads better
because its seven cottages at 0.125 m vary in size, windows, dormers, doors and
colors around lanes that leave the square through real gaps. Run 2's eight clone
cottages share one footprint and block its lanes into dead ends.

Failures:

1. Run 1, pass 1: the build failed with
   `bands materials[0] must be a Material, not a Pattern` because the tower
   quoins fed a noise pattern into `bands`
2. Run 1, passes 2 and 3: displaced tree and yew crowns shed leaf specks for 35
   pieces, then 16 at 0.125 m. Lower amplitudes and a reseed cleared them
3. Run 1, pass 2: the nave and chancel gable steps sat at `0 kept` under the
   roof prisms. A later pass dropped them
4. Run 2, pass 1: the well roof, the lych-gate roof and two crosses floated
   above their posts and joined leaf specks for 41 pieces. The specks lasted
   until the session clipped the crowns with a slightly larger ellipsoid
5. Run 2, pass 1: an extra ridge roll read as a flat plank wall along every
   cottage ridge. It stayed until pass 5
6. Run 1, final pass: the garden crops read as toy-sized cubes and the stepped
   spire as stacked slabs. The hero leaves the scene small in the frame
7. Run 2, final pass: all eight cottages share one 8 x 5 m footprint and window
   layout. Abutting pairs form terraces whose seams dead-end the east and west
   lanes
8. Run 2, final pass: the south cottages hide most of the square in the hero

Lacked:

1. Both runs: `part()` takes only a pivot and an offset with no turn per
   placement. Each run wrote a cottage function that rotates every shape inside
   a per-cottage part
2. Both runs: a piece line that stays readable. Piece 1 names every step in the
   scene on one line

Missed in the skill:

1. Run 2: SKILL.md says each copy needs its own seed. The `ridgeStraw` and
   `flowers` patterns kept one fixed seed and repeat exactly across all eight
   cottages

Colors: Run 1's timber frames, doors and shutters are single flat materials. Its
garden crops read as uniform 0.5 m colored cubes. Run 2's thatch bands read as
horizontal planks and its ridge teeth as a dark plank strip. Its flag and clock
face are flat cream. Its plaster walls look nearly identical from cottage to
cottage.

### 26. City

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | good    | 2      | 0      | 5.1     | 954,656 at 0.25 m |
| 2   | good    | 2      | 0      | 5.0     | 626,639 at 0.25 m |

Both runs built at 0.25 m in one root object with zebra crosswalks, street
lights and a park with a fountain. Run 1 reads better because its 60 m
nine-block grid rises to a 42 m glass tower over a central park that a lowered
corner tower opens to the hero. Run 2's 40 m tile has richer buildings up close
with recessed lit windows, cornices and cars, but it reads as one intersection
topping out at a 30 m stepped tower. Round 5's better run kept the nine-block
grid and showed its park after round 4 traded one for the other.

Failures:

1. Both runs, pass 1: displaced tree crowns shed 8 stray voxels for 9 pieces.
   Pass 2 lowered the amplitude in each run
2. Run 1, pass 1: a 24 m corner tower in the front-right block hid the park from
   the hero. Pass 2 cut it to 9 m
3. Both runs, final pass: tree crowns bury most of the fountain in run 1's park
   close-up and run 2's hero. Run 2's final message called this acceptable
4. Run 2, final pass: the 16 street lights are small and easy to miss at hero
   scale
5. Run 2, final pass: the final message called a blue clad wall with punched
   windows a 26 m glass tower

Lacked:

1. Both runs: a piece line that stays readable. Piece 1's `from` list names
   every step on one line and bloats the report to 128 to 166 lines

Missed in the skill:

1. Run 2: SKILL.md says the prompt's key detail faces the hero corner with
   nothing between it and that corner. The session saw a tree hiding the
   fountain and shipped it unmoved

Colors: Run 1's glass towers carry a `cells` pattern of four blues that reads as
camouflage blotches rather than glazing. Both runs' roofs share one flat tar
gray. Run 2's wall `noise` reads as grimy, blotchy plaster on the cream and gray
buildings. Its "glass" tower is opaque blue paint.

### 27. Dollhouse

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | good    | 3      | 1      | 18.4    | 1,070,225 at 5 cm |
| 2   | good    | 3      | 0      | 11.1    | 1,072,192 at 5 cm |

Both runs built a 10 x 6 m three-bay dollhouse at 5 cm with the front wall off
and six furnished spaces from the ground floor to the attic. Run 1 reads better
because of truer materials: blue clapboard, glazed windows, flower boxes, a
furnished landing and a glowing lamp in every room. Run 2 trims every cut edge
in white and shows the more legible attic, but its brick reads as red siding and
its upper stair hall stands bare. Both round 5 runs kept the attic open to the
front after round 4's run 2 hung a roof slope over it.

Failures:

1. Run 1, pass 1: the build failed with
   `step name must be unique in its list, not "attic floor" twice`
2. Both runs, first voxelized pass: run 1's `front cut` and `gable boards`
   paints and run 2's clapboard paint spilled onto the roof. The next pass fixed
   both
3. Run 1, first voxelized pass: the window frames subtracted their own glass and
   left gray filled windows. The next pass restored the panes
4. Both runs, first voxelized pass: a floating teddy and displaced bush and
   plant leaves left run 1 at 21 pieces. Detached chair backs, an outside door
   knob and displaced foliage left run 2 at 15
5. Run 2, pass 1: the bed frame buried the bed legs at `0 kept`. The session
   dropped them
6. Run 2, pass 3: enlarging the bathroom checker to 0.2 m left its moire in the
   oblique view
7. Both runs, final pass: the bathroom tub stays hidden from the review views.
   The standard hero shows mostly roof and side wall
8. Run 2, final pass: the upper stair hall is an empty gray shaft

Lacked:

None

Missed in the skill:

1. Run 1: SKILL.md says a paint recolors every live cell in its reach and
   documents `within`. The roof spill cost part of a pass
2. Run 2: SKILL.md's stone-wall recipe gives coursed blocks from `cells` with a
   `border`. The brick walls and chimney used `bands` and read as red clapboard

Colors: Both runs' bathroom checker floors moire in the oblique views. Run 1's
red and white checker reads as a tablecloth. Run 2's ground-floor brick and
chimney read as striped red siding. Its attic gable and upper stair hall are
flat gray plaster.

### 28. Living room

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | good    | 3      | 0      | 5.7     | 703,447 at 2 cm   |
| 2   | good    | 3      | 1      | 4.7     | 287,644 at 2.5 cm |

Both runs built a two-wall corner room with an arched stone fireplace, a
wingback, a bookcase of seeded varied books, a medallion rug and an eye-level
inside view. Run 1 reads better on finish because its 2 cm surfaces carry plank
seams, a cleaner fire and a richer rug. Run 2 at 2.5 cm sets the chair facing
the rug and fire but leaves salmon embers and noisy velvet, mantel and floor.
Round 5's run 2 faced its chair to the fire after both round 4 runs turned
theirs away.

Failures:

1. Run 2, pass 1: voxelize failed with
   `living-room/back cushion: box round must be at most half the shortest side, 0.0375, not 0.04`.
   Pass 2 deepened the cushion
2. Run 1, pass 1: the window pane rendered the green wall behind it. Pass 2
   carved a window opening
3. Both runs, first voxelized pass: run 1 split into 4 pieces. Run 2's floating
   lamp shade and three stray foliage voxels split it into 5
4. Run 2, pass 2: the flames rendered pale pink. Pass 3 darkened them to orange
5. Run 1, pass 2: the wainscot panels showed through the 2 cm bookshelf back in
   the right view. Pass 3 thickened the back to 6 cm
6. Run 1, final pass: the armchair sits in the window corner facing out of the
   room rather than toward the fire. The lamp table jams against the hearth
7. Run 2, final pass: the ember tops on the logs read salmon. The session
   reported them and stopped without a fix

Lacked:

None

Missed in the skill:

1. Run 2: SKILL.md says a pale glow takes a lower `emissiveStrength` or a darker
   `baseColor` and that strength 1 on a dark base keeps full hue. The fire at
   1.5 on mid-tone bases cost a pass and left the embers salmon

Colors: Run 1's walnut bookshelf reads mottled red-brown with a bullseye knot on
its top. Its left wainscot reads flat gray-beige beside the warmer back wall.
Run 2's embers read salmon instead of glowing red. Its green velvet noise reads
as camouflage. Its oak mantel shows coarse blotches. Its floor reads as noisy
stripes with no plank joints.

### 29. Spaceship

| Run | Verdict | Passes | Failed | Minutes | Voxels              |
| --- | ------- | ------ | ------ | ------- | ------------------- |
| 1   | good    | 2      | 0      | 3.4     | 979,794 at 2.5 cm   |
| 2   | fair    | 3      | 0      | 3.7     | 1,016,566 at 2.5 cm |

Both runs built a 2.5 cm bridge with a chair on a dais, a radar console before
the window and a speckled starfield niche holding a banded planet and moon. Run
1 reads better because its cutaway drops the front and right walls and the
ceiling to let every review view see the bridge. Run 2 has a strong inside view,
but its heavy roof and full side walls blank the right view and hide the planet
from the hero.

Failures:

1. Run 1, pass 1: the starfield paint covered the niche's outer faces and stuck
   a starry box out behind the room in the hero. Pass 2 lined the outside in
   dark hull
2. Run 1, pass 1: the wall screens sat flush in their bezels at 0 exposed and
   rendered black. Pass 2 moved them forward
3. Run 2, passes 1 and 2: pale glass hazed the stars and a dark `#1A2840` tint
   then blacked them out. Pass 3 dropped the glass
4. Run 1, final pass: the hero and right views see the radar nearly edge-on. The
   chair back hides the radar in the front view
5. Run 2, final pass: the right view shows only the outer hull wall and the top
   view shows mostly roof. The hero shows the planet only as a sliver at the
   window edge
6. Run 2, final pass: the moon floats as a second piece. The session accepted it
   rather than seating it on the backdrop
7. Run 2, final pass: the chair is small and plain with a ragged blob for its
   star emblem. The radar body reads as a stair-stepped black block among
   confetti console buttons

Lacked:

None

Missed in the skill:

1. Run 2: SKILL.md documents `#RRGGBBAA` `baseColor` alpha for clear glass. The
   session tried only opaque pale and dark tints and then dropped the pane
2. Run 2: SKILL.md says the prompt's details face +z or +x with nothing between
   them and the hero corner. The roof soffit and the outer box's lid hide the
   planet and half the window from the hero

Colors: Run 1's window niche reads as a flat dark slab in the hero and top
views. Its radar housing back is a flat black disc. Run 2's roof and outer side
walls are wide light-gray planes broken only by thin seams. Its radar body is an
unrelieved black block.

### 30. Pirate ship

| Run | Verdict | Passes | Failed | Minutes | Voxels          |
| --- | ------- | ------ | ------ | ------- | --------------- |
| 1   | good    | 9      | 2      | 7.6     | 532,813 at 3 cm |
| 2   | fair    | 4      | 0      | 5.3     | 114,633 at 4 cm |

Both runs built a gun deck with guns run out through framed ports, hammocks,
barrels and a lantern on a chain. Run 1 reads better because its curved 3 cm
hull with ribs, knees, four guns and a mast makes a ship's hold under the
lantern's warm light. Run 2's open 4 cm diorama reads at a glance in the hero,
but its nearly flat single wall and bright daylight make it read as a pine
cabin. Round 5's run 1 lit its hold with a point light at the lantern after
round 4 found no light the lantern could cast.

Failures:

1. Both runs, pass 1: the lantern chain stopped short and left the lantern
   loose. Run 2's 9 pieces also held hammocks floating off their lines, the
   bucket handle and the ramrod pegs
2. Run 2, pass 2: the 3 cm hammock lines still left pieces. Pass 3 thickened
   them to 4 cm
3. Run 1, pass 3: the inside render failed with
   `light 1's transform lacks --light-frame; a posed point light takes --light-frame and --light-position`.
   Pass 4 hit the same error for directional light 2 until
   `--light-frame 2 world` fixed it
4. Run 1, final pass: the review hero, top and right views show only the closed
   deck and outer hull. The mast blocks a third of the inside view
5. Run 1, final pass: the hammocks read as deep hanging bowls or scale pans.
   From below they show as a dark gray dish
6. Run 2, final pass: the keg on barrel 1 reports `detached`. The session
   filtered the report and never saw the line
7. Run 2, final pass: the hull wall is an arc of a 4.26 m circle that reads
   nearly flat. The right side and the ceiling are gone
8. Run 2, final pass: at 4 cm the hammocks read as chunky stepped boats and the
   rope lines as blocks. The lantern is a box with one flat orange pane that
   lights nothing

Lacked:

None

Missed in the skill:

1. Run 1: SKILL.md shows pointing the hero into the opening with
   `--view-orbit hero`. The session left the review hero facing the deck part
   and shipped a closed box
2. Run 2: the render command the session ran already carried `--light` with
   `point`. No render shows the lantern lighting the hold because the session
   never added a point light at it

Colors: Run 1's deck top and outer hull read as flat plank stripes in the hero
and top views. Its hammock canvas reads dark gray under the warm light. Run 2's
lantern pane is one flat orange rectangle. Its light pine floor and oak walls
wash out under daylight with little contrast between hull, bulkhead and floor.
