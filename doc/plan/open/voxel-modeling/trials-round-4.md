# Trials, round 4

Step S14 of the [checklist](checklist.md) runs the [prompts](trials.md#prompts)
a fourth time after round 3's fixes. [Round 3](trials-round-3.md) logs the round
before, and the [trial
harness](../../../../projects/vxl/trials/README.md) ran all four.

## Changes before round 4

Round 3's findings led to five changes in vxl and one in the harness:

1. `--view-look-at` beside `--view-orbit` sets the point the orbit circles
2. Every render draws on opaque white unless `--background transparent` asks for
   alpha
3. `--report` also writes the report beside the document as `<stem>-report.txt`
4. The skill's example became a walnut side table in place of an oak chair with
   rubies
5. An empty `union` or `intersect` says it holds no shape
6. The harness copies each pass's report beside its renders

## Findings

Round 4's analyses recorded 484 items across the 60 runs, against round 3's 486.
Every finding keeps its round 3 number, and new findings start at 93. The
round's gallery lists each finding's items beside the renders.

### The owner's read

The owner judged round 4 worse overall than round 3. The models kept their
scale. Run for run, round 4 had more steps in 35 runs and fewer in 21, and
primitive kinds, materials, and passes stayed level. The review views sat on
white in both rounds. The look could come from sampling or from the largest
prompting change. The skill's example became a plainer walnut side table in
place of an oak chair with spindles, finials, a gilt rail, and rubies.

### What the changes did

The table counts the prompts that showed each finding the changes answered.

| #   | Finding                                      | Round 3 | Round 4 |
| --- | -------------------------------------------- | ------- | ------- |
| 17  | Views the review four cannot give            | 7       | 2       |
| 71  | Sessions cut the report to its model line    | 5       | 2       |
| 74  | Hero renders came out on black               | 1       | 0       |
| 85  | The skill's example matches the chair prompt | 1       | 1       |
| 88  | An empty union gave an unclear error         | 1       | 0       |

No render came out on black, and no session hit an empty boolean. The white
background did wash out light materials in 4 prompts: lantern run 1's clear
panes, dragon run 2's silver goblet, crane run 2's white hook stripes, and
log-cabin run 1's snow in the top view. Three findings still showed:

1. **Views the review four cannot give.** Twenty-four runs combined the two
   flags in 50 renders without an error. With `fit`, the orbit frames the whole
   subject about the point. Sword run 2 and living-room run 1 got a small shot
   off center, and log-cabin run 2 accepted one
2. **Sessions cut the report to its model line.** Eight runs in the chess set,
   city, dollhouse, dungeon kit, valley, and village prompts read the file. Most
   others cut the printed report and lost nothing. Dragon run 1 grepped away its
   piece lines and shipped a floating smoke stick. Fish-tank run 1 cut the piece
   list and shipped specks of tail tips and gravel. Lantern run 1 ran the
   voxelize twice in one command to read the head and the tail
3. **The skill's example matches the chair prompt.** Neither chair run built a
   table. The guitar runs matched the skill's sizing example instead: six
   strings across a 5 cm neck at about 4 mm, about 250 voxels long. Both took 4
   mm and about 265 voxels

### Still open

Nothing changed for these findings:

| #   | Finding                                                | Round 3 | Round 4 |
| --- | ------------------------------------------------------ | ------- | ------- |
| 2   | Large surfaces left flat                               | 23      | 19      |
| 4   | Ended with flaws the renders show                      | 15      | 18      |
| 13  | Neighboring materials too close in hue                 | 6       | 12      |
| 10  | Features near a voxel thick missed cells or broke      | 9       | 11      |
| 19  | The hero view hid the focal element                    | 5       | 10      |
| 5   | Dark materials collapse to near-black                  | 6       | 9       |
| 7   | Later steps buried earlier details                     | 6       | 9       |
| 65  | Noise read as camouflage                               | 5       | 9       |
| 3   | Documented tools left unused                           | 9       | 6       |
| 32  | Grain read wrong                                       | 6       | 6       |
| 6   | Small round shapes read as blocks or plus signs        | 7       | 5       |
| 9   | Finer grids read better than the guidance              | 7       | 5       |
| 12  | Shapes at an angle alias                               | 4       | 5       |
| 14  | Glass, water, ice, and gems read wrong                 | 3       | 5       |
| 34  | Paint and coat spill onto neighbors                    | 3       | 4       |
| 44  | Placing on the cell grid took hand work                | 4       | 4       |
| 84  | The gray hook washed out on white                      | 0       | 4       |
| 81  | Sloped snow drew stair-step contour lines              | 3       | 3       |
| 87  | Curves came from hand-sampled points                   | 2       | 3       |
| 91  | Market stalls repeated one layout                      | 1       | 3       |
| 37  | Mortar swamps stone walls                              | 1       | 2       |
| 55  | 0 kept flags intended repaints                         | 0       | 2       |
| 75  | Flame emissive dropped below the bloom threshold       | 1       | 2       |
| 80  | Neither pirate ship curved its hull                    | 3       | 2       |
| 92  | No pattern lays brick courses                          | 1       | 2       |
| 8   | Renders skipped                                        | 2       | 1       |
| 18  | Emissives wash out                                     | 3       | 1       |
| 20  | Library metals read off hue                            | 3       | 1       |
| 27  | Light surfaces turn blue-gray                          | 1       | 1       |
| 30  | Proportion and silhouette drift                        | 0       | 1       |
| 36  | Placements share no object                             | 2       | 1       |
| 56  | The model's grid constant and the voxel size disagreed | 0       | 1       |
| 68  | Lightened gold shades read beige                       | 1       | 1       |
| 69  | Oak read orange and walnut read pink                   | 0       | 1       |
| 73  | Close-ups selected the whole sword                     | 1       | 1       |

Neighboring materials too close in hue rose from 6 prompts to 12, and hero views
that hid the focal element rose from 5 to 10. Sessions that ended on a known
flaw rose from 15 to 18. Findings 1, 15, 16, 21, 28, 31, 40, 45, 52, 54, 57, 66,
67, 70, 72, 76, 77, 78, 79, 82, 83, 86, 89, and 90 showed in no prompt.

The phase 2 candidates that round 4 showed:

| #   | Finding                                           | Round 3 | Round 4 |
| --- | ------------------------------------------------- | ------- | ------- |
| 24  | Floating crumbs                                   | 6       | 14      |
| 11  | Seating details on a surface                      | 5       | 7       |
| 26  | A seeded random helper                            | 4       | 5       |
| 22  | Rotation on part placement and a one-sided flip   | 3       | 4       |
| 25  | Sweeps, helices, and spirals                      | 3       | 3       |
| 23  | Scene reports too long to read                    | 8       | 2       |
| 35  | Posed parts, plumb joints, and ropes across parts | 2       | 2       |
| 46  | Emissives light nothing nearby                    | 2       | 2       |
| 38  | Intended separate pieces read as floating         | 1       | 1       |
| 39  | Inspecting nodes and palette values               | 1       | 1       |
| 43  | A flag for washed-out or hidden emissives         | 1       | 1       |
| 47  | Text                                              | 1       | 1       |
| 48  | Crevice and edge shading in the colors            | 1       | 1       |
| 53  | Parts that cut through each other                 | 0       | 1       |

Floating crumbs rose from 6 prompts to 14 as sessions displaced crowns, peaks,
steam, and smoke. Findings 29, 33, 41, 42, 49, 50, 51, 58, 59, and 60 showed in
no prompt.

### New in round 4

93. **The review set took a close-up's stem.** In 2 prompts, `--profile review`
    with an extra view and `--file-stem` wrote all five views under the stem.
    Lantern run 2 got `lantern-close-close.png` beside four review views, and
    knight run 2 got `knight-back-front.png` and its siblings. The sessions
    renamed one file and deleted the rest
94. **Ceilings rendered near-black.** In 1 prompt, dollhouse run 1's close-up at
    5 degrees elevation drew the light plaster ceilings near-black over pale
    walls. Downward faces inside closed rooms get no light
95. **A second bend moved the first.** In 1 prompt, wizard-tower run 1 bent the
    whole roof cone a second time. The second bend swung the first arc and
    dropped the eaves into the chamber. Bending only the upper half fixed it a
    pass later
96. **The harness merged back-to-back passes.** In 1 prompt, village run 2 ran 7
    voxelizes, one a loop over tree seeds, and the harness counted 6 passes. Two
    voxelizes with no render between them share a pass
97. **A lathe ring filled to a disc.** In 1 prompt, well run 1 drew its coping
    as a `lathe` profile from radius 0.475 to 0.75. `lathe` closes the outline
    along the axis, so the ring filled to a solid disc that capped the shaft.
    The session told the user the water showed
98. **A part glob took the child parts.** In 1 prompt, crane run 1 passed
    `--view-select close 'crane/cab'`. The glob took the cab's child parts. The
    close-up framed the boom and hook and showed the whole crane
99. **A gradient read the frame before the move.** In 1 prompt, candelabra run 2
    set a flame `gradient` in world heights. The pattern reads the flame's frame
    before its `translate`. The bands landed wrong, and a pass went to finding
    why
100.  **`round: 0` errored.** In 1 prompt, robot run 2's box helper passed
      `{ round: 0 }` for square boxes. The build failed with
      `box round must be above zero, not 0`, and the session lost a pass
101.  **A sunlit coat shifted hue.** In 1 prompt, tree run 2 lit its crown's
      `+y` coat with hand-picked yellow-greens. The hero and top views read lime
      because the coat moved toward yellow where only its lightness had to rise
102.  **Steam has no volume material.** In 1 prompt, ramen run 2 built steam
      from displaced transmissive ellipsoids. The steam reads as opaque gray
      blobs, and its lobes split into loose voxels that cost a pass

### No action

In 30 prompts the two runs chose near-identical plans, and in 29 workarounds
used the tools as intended. In 25 prompts sessions made one-off slips. In 10 the
report and the views caught real defects.

## Log

Round 4 ran on 2026-10-05 with vxl 0.5.0 built from the staged tree over
`630397be` and Claude Code 2.1.289 running Claude Opus 5.5 at high effort. Every
run loaded the skill and called its model done. Of the 60 runs, 45 read good and
15 fair, against round 3's 48 and 12. Different agents judged each round, so the
verdicts compare loosely. The runs took 192 passes, of which 3 failed, and 973
turns against round 3's 982. A run took a median of 3.4 minutes, and all 60 cost
$60. `~/voxel-trials/rounds/2026-10-05-round-4` holds the round, and its gallery
shows every run's renders.

### 1. Chair

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | good    | 2      | 0      | 1.7     | 11,978 at 1.25 cm |
| 2   | good    | 4      | 0      | 1.8     | 19,686 at 1 cm    |

Both runs built an oak side chair whose arched, gold-trimmed crest holds
bezel-set gems above jeweled post finials. Run 2 reads better because its 1 cm
crest carries five larger gems that read clearly from the hero and front where
run 1's 1.25 cm crest sets small emeralds beside diamond studs that read as
specks. Run 1 kept a more ornate body of vase splat, spindles, and turned legs,
but it renders ragged. Unlike round 3, where both runs shipped plus-sign stones
at 1.25 cm, run 2 moved to 1 cm and grew its gems.

Failures:

1. Both runs, pass 1: the finial gems floated above the posts as extra pieces.
   Run 1 seated its sapphires in pass 2
2. Run 1, pass 1: an edit before the first build dropped the arc filigree from
   the crest. The side filigree left behind does not show in the renders
3. Run 2, pass 1: the cross stretcher sat above the side stretchers as a
   separate piece
4. Run 2, pass 2: the session swapped the turned legs and filigree for square
   legs and box studs because they voxelized jagged at 1 cm
5. Run 2, pass 2: the new post caps, mirrored about `x = 0` from the origin,
   landed inside the crest and left the amethysts floating
6. Run 2, pass 3: the amethysts still sat one voxel above the caps until pass 4
   lowered them 1 cm

Lacked:

1. Run 2 wanted the report to give the gap between a floating piece and its
   nearest neighbor. That would have closed the amethyst gap in one pass instead
   of two

Missed in the skill:

1. Run 1 set the diamond studs as single `set` voxels and the spindles one voxel
   wide although SKILL.md says every named feature spans at least 2 voxels. The
   studs read as specks and the spindles as slivers
2. Both runs kept a voxel too coarse for the ornament although the skill has the
   smallest feature set the voxel size. Run 1's 3-voxel emeralds barely facet at
   1.25 cm where run 2 traded its jagged turned legs for square ones at 1 cm

Colors: Run 1's high-contrast oak grain turns the narrow splat into camouflage
blotches. Its back posts and legs read nearly one flat tone. Run 2's legs,
posts, and aprons read nearly flat brown. Its gilt arch reads muddy because the
gold sits close in hue to the oak patches the coat leaves.

### 2. Lantern

| Run | Verdict | Passes | Failed | Minutes | Voxels         |
| --- | ------- | ------ | ------ | ------- | -------------- |
| 1   | good    | 3      | 0      | 1.7     | 33,520 at 5 mm |
| 2   | good    | 4      | 0      | 2.1     | 4,205 at 1 cm  |

Both runs built a square iron lantern with a stepped pyramid roof, a ring
handle, and a dripping candle on a brass dish. Run 2 reads better because its
faintly emissive glass fills the panes with warm light where run 1's panes stay
blank white around a lit candle. Run 1 is the better-crafted object at 5 mm,
while run 2's 1 cm candle and flame stay square stacks behind a corner post.

Failures:

1. Run 1, pass 1: the handle torus floated as a second piece of 68 voxels until
   pass 2 attached it
2. Run 1, pass 2: a corner post hid the flame in the standard hero while the top
   bar hid its tip. Pass 3 lowered the candle and rendered the hero from azimuth
   25
3. Run 1, final pass: the mid crossbar cuts between the candle and flame in the
   front and right views
4. Run 2, pass 1: the melt carve reached 0 cells
5. Run 2, pass 1: the 4-voxel flame was too small to read behind the tinted
   glass
6. Run 2, pass 4: the close-up rendered with `--profile review` and
   `--file-stem lantern-close` wrote five PNGs including
   `lantern-close-close.png`. SKILL.md's `--file-stem robot-close` example
   promises only `robot-close.png`
7. Run 2, final pass: the standard hero keeps a corner post down the middle of
   the candle and flame

Lacked:

1. Both runs wanted a light source that casts the flame's color onto the wax,
   dish, and frame. Run 2 faked it with a `0.12` orange emissive on the glass
2. Run 1 wanted the report to flag a post or bar that hides the flame from a
   review view. It found the occlusion only in the renders

Missed in the skill:

1. Run 2 kept 1 cm voxels for a 2.5 cm candle and a 2 cm flame although SKILL.md
   says round shapes under about 3 voxels in radius read as blocks. Both stayed
   square stacks
2. Run 2 left a corner post across the candle and flame in the final hero
   although SKILL.md's `--view-orbit` hero example shows how to re-aim it
3. Run 1 left its panes blank white with no warm tint or emissive although
   SKILL.md's color rule 3 says clear glass reads faintly. The lantern looks lit
   without glowing

Colors: Run 1's panes read as flat off-white panels in the front and right
views. Its iron noise forms large light gray blotches that read as camouflage on
the cap, base, and posts. Run 2's cage, cap, plinth, and handle are each one
flat dark iron with noise only on the roof. Its candle is one flat cream.

### 3. Chest

| Run | Verdict | Passes | Failed | Minutes | Voxels          |
| --- | ------- | ------ | ------ | ------- | --------------- |
| 1   | good    | 4      | 0      | 2.0     | 100,633 at 1 cm |
| 2   | fair    | 3      | 0      | 2.0     | 88,266 at 1 cm  |

Both runs built an iron-bound chest at 1 cm with a barrel lid pivoted open as a
part over a gold heap of coins and three gems. Run 1 reads better because its
3.4-voxel coins with darker painted rims read as round discs on the heap and the
ground where run 2's 2.5-voxel coins read as flat yellow squares on a terraced
dome beside a loaf-shaped spill. The two chest bodies read equally well.

Failures:

1. Run 1, pass 1: the heap coins were 4-cell squares. Passes 2 to 4 grew them
   into 7-voxel round discs
2. Run 1, final pass: a few one-voxel holes show in the heap from above. The
   session saw them and left them
3. Run 1, final pass: the ground coin stack zigzags and reads lumpy
4. Run 2, pass 1: the inner shadow paint spanned the full body box. The report
   showed the body keeping 0 of 72,000 cells, and the outer planks took the dark
   shade until pass 2
5. Run 2, pass 1: coins at a 2-voxel radius read as plus signs. Pass 2 raised
   the radius only to 2.5 voxels and the coins shipped as squares
6. Run 2, final pass: the sapphire is a thin cross half-covered by a coin

Lacked:

None

Missed in the skill:

1. Both runs placed coins and gems at a hand-written heap height instead of the
   `coat` with `sides` that SKILL.md recommends for gems on a pile. Run 1's heap
   kept one-voxel holes where run 2's sapphire ended half-buried under a coin
2. Run 2 sized coins at 2 then 2.5 voxels although SKILL.md warns that a
   cylinder under about 3 voxels of radius reads as a plus sign or block. The
   coins, the prompt's key detail, shipped square
3. Run 2 left the heap ellipsoid undisplaced. Its grid contours read as
   concentric terraces that SKILL.md's `displace` would have broken up

Colors: Run 1's lid lining reads as one nearly flat dark brown `#3B2414` against
the heap. Run 2's heap coins are one pale yellow with the rim coat only on their
`-x` and `-z` sides. They read as flat light tiles in the hero. Its pile shading
shows contour rings rather than varied gold.

### 4. Sword

| Run | Verdict | Passes | Failed | Minutes | Voxels         |
| --- | ------- | ------ | ------ | ------- | -------------- |
| 1   | good    | 2      | 0      | 1.5     | 9,808 at 5 mm  |
| 2   | good    | 3      | 0      | 1.8     | 12,428 at 5 mm |

Both runs built a 1 m point-up longsword at 5 mm with a gold crossguard, a
banded grip, and a wheel pommel holding a bezel-set ruby. Run 2 reads better
because its two-tone steel with bright edge bevels reads as a polished blade and
its faceted octagonal ruby shows even in the hero, where run 1's grain stains
the blade with dark smears and its ruby shrinks to a few red pixels. Run 2's
guard finials sink into the bar above a spindly 3 cm grip. Unlike round 3, where
`--view-orbit` with `--view-look-at` failed both close-ups, run 1's hilt
close-up rendered with the pair.

Failures:

1. Both runs, pass 1: the blade came out too skinny with the octahedron gem
   mostly buried. Pass 2 widened the blade and replaced the gem
2. Run 1, pass 2: the first close-up passed `--select 'sword'` and
   `--view-select close 'sword'` on a model with no parts. A hand-placed orbit
   and look-at re-rendered it
3. Run 2, pass 2: the orbit close-up with fit and a look-at point framed poorly.
   The session replaced it with a hilt view placed by position, field of view,
   and look-at
4. Both runs, final pass: the tip tapers into a one-voxel needle several cells
   long
5. Run 2, final pass: the ball finials that the final message claims sit almost
   inside the guard bar. The guard reads as a plain bar with rounded ends

Lacked:

None

Missed in the skill:

1. Run 1 broke up the blade core with grain and the guard and pommel with noise
   as SKILL.md suggests for flat faces. At 5 mm both read as blotches where a
   gentle gradient or two-tone bands would have read as polished steel
2. Run 2 sized the guard finials at a 2.2 cm radius on a 4 cm bar and never
   checked the guard ends in a close-up. The finials vanish into the bar
   although the report counted 272 exposed cells

Colors: Run 1's `grain(steelShades)` reads as dark camouflage smears down the
blade. The noise on its guard bar and pommel shows as yellow blotches in the top
and hero views. Run 2's guard bar and blade core are each one flat tone. The
flat tone reads clean on the blade but plain across the 26 cm guard in the front
view.

### 5. Tree

| Run | Verdict | Passes | Failed | Minutes | Voxels           |
| --- | ------- | ------ | ------ | ------- | ---------------- |
| 1   | fair    | 3      | 0      | 1.3     | 126,254 at 0.1 m |
| 2   | good    | 4      | 0      | 1.7     | 84,289 at 0.1 m  |

Both runs built an oak at 0.1 m with roundCone limbs under a displaced crown of
ellipsoid clumps lit by a sunlit top coat. Run 2 reads better because it rebuilt
after pass 1 with crooked forking limbs under fifteen separate lobes that read
from every view, where run 1's dense dome hides its limbs everywhere but the
front. Run 2 still leans toward an umbrella silhouette with a lime crown and no
ground.

Failures:

1. Both runs, pass 1: the crown's `displace` split off floating leaf pieces, 38
   in run 1 and 16 in run 2. Run 1 cleared them in pass 2 by retuning the
   amplitude, scale, and octaves. Run 2 cut them to 5 and then 2 over passes 2
   and 3
2. Run 2, pass 4: carve boxes at coordinates copied from the report removed the
   last two tufts. The boxes stop fitting when the noise seed changes
3. Run 1, final pass: the crown hangs nearly to the trunk base and hides the
   limbs in the hero, right, and top views
4. Run 2, final pass: the model has no ground, so the tree floats on white

Lacked:

1. Both runs wanted a step that drops small disconnected pieces. Run 1 retuned
   the crown's `displace` where run 2 carved boxes at the report's coordinates

Missed in the skill:

1. Run 1's five roots read as a regular star in the hero and top views because
   one roundCone repeats under `repeatPolar`. SKILL.md notes that copies repeat
   exactly and that a distinct look takes its own step
2. Run 1 never rendered a low `--view-orbit` under the crown. The limb structure
   that the hero, right, and top views hide went unreviewed

Colors: Run 1's crown greens sit close to the grass disc's greens in the top
view, where the crown edge barely separates from the ground. Its trunk reads
near-black in the hero. Run 2's sunlit `+y` coat reads a lime and yellow-green
too bright for an oak in the hero and top views. Its `-y` shade coat turns the
crown underside into a near-black band in the front and right views.

### 6. Well

| Run | Verdict | Passes | Failed | Minutes | Voxels           |
| --- | ------- | ------ | ------ | ------- | ---------------- |
| 1   | fair    | 2      | 0      | 1.9     | 99,824 at 2.5 cm |
| 2   | good    | 4      | 0      | 1.7     | 62,302 at 2.5 cm |

Both runs built a round stone well at 2.5 cm with oak posts, a gabled roof, a
windlass with a crank and rope, and a hooped bucket. Run 2 reads better because
its rim of subtracted cylinders leaves an open shaft with blue water under a
hanging bucket where run 1's lathe coping seals the shaft. Run 1's well reads as
a capped stone drum with the bucket hovering over it. Unlike round 3, where the
eave hid the rope in both final heroes, run 2's hero shows the rope dropping to
the bucket over open water.

Failures:

1. Both runs, pass 1: the roof hid the bucket in the hero. Both raised the eaves
   in pass 2
2. Run 1, final pass: the coping lathe filled to the axis as a 10,024-cell disc
   that caps the shaft. The session's final message claims dark water inside
   that no render shows
3. Run 1, final pass: the roof hides the rope in the hero
4. Run 2, pass 1: the roof floated above the 2.05 m posts until pass 2 raised
   them to 2.4 m
5. Run 2, pass 1: the torus handle broke into two 6-voxel pieces until pass 2
   replaced it with an extruded arc
6. Run 2, final pass: the rafter ends poke below the eaves as small tabs that
   read as floating

Lacked:

None

Missed in the skill:

1. Run 1 built the coping with `lathe` although SKILL.md states that `lathe`
   closes the outline back along the axis. A subtract of two cylinders would
   have left the shaft open
2. Run 1 missed the lid in the report's 10,024-cell coping count and in the
   hero. It rechecked only the hero and top in pass 2 and never looked into the
   shaft

Colors: Both curved walls streak vertically at their sides. Run 1's water never
shows under the lid. Run 2's plain oak rafters and pale ridge beam read flat
against the banded walnut roof.

### 7. Robot

| Run | Verdict | Passes | Failed | Minutes | Voxels           |
| --- | ------- | ------ | ------ | ------- | ---------------- |
| 1   | good    | 2      | 0      | 1.6     | 9,236 at 1.25 cm |
| 2   | good    | 3      | 1      | 1.4     | 7,174 at 2.5 cm  |

Both runs built a blue toy robot with the head and arms as parts on neck and
shoulder pivots. The runs trade strengths evenly: run 1 at 1.25 cm has the
richer visor face, grille and gold-dialed chest but lumpy shoulder balls over
oversized forearm cuffs. Run 2 at 2.5 cm has cleaner, even arms under orange
shoulder caps, but its solid black claws merge with the dark legs. Unlike round
3's run 1, both runs proved the joints with a posed render.

Failures:

1. Run 2, pass 1: the build failed with `box round must be above zero, not 0`
   because the arm helper passed `{ round: 0 }` for unrounded boxes. A
   conditional fixed it
2. Run 1, pass 1: the forearm `roundCone` buried the elbow sphere at `0 kept`.
   Pass 2 turned the elbow into a gold band
3. Both runs, first built pass: the ears were cylinders 2 voxels in radius that
   sampled into plus signs. Each run spent an edit replacing them
4. Run 1, final pass: the noised shoulder sphere reads as a lumpy blob. Each arm
   reads as a thin rod ending in a forearm box wider than the shoulder
5. Run 2, final pass: the claws are solid black blocks that merge with the black
   legs in the hero. At 2.5 cm the dial and antenna tip collapse to single
   squares

Lacked:

1. Both runs: a render-time way to preview a part turned about its pivot. Both
   sessions found `vxl node set rotation` through `--help` and rendered a posed
   copy
2. Both runs: a joint clearance check in the report. Whether a turned arm or
   head clips the torso could be judged only from one hand-posed render

Missed in the skill:

1. Both runs: the ears took cylinders 2 voxels in radius though Resolution rule
   2 warns that one under about 3 voxels reads as a plus sign. The same rule
   covers run 1's lumpy 3.2-voxel shoulder sphere and run 2's square dial and
   antenna tip
2. Run 2: Color rule 2 asks neighboring materials to differ in lightness, yet
   the claws, wrists and legs share the dark joint material. The claws merge
   into the legs in the hero

Colors: Run 2's claws, wrists, legs, hips and backpack share one near-black
metal. Its lower body and hands read as one dark mass in the hero.

### 8. Cottage

| Run | Verdict | Passes | Failed | Minutes | Voxels          |
| --- | ------- | ------ | ------ | ------- | --------------- |
| 1   | good    | 2      | 0      | 2.6     | 520,282 at 5 cm |
| 2   | good    | 3      | 1      | 2.7     | 562,181 at 5 cm |

Both runs built a spotted red mushroom cottage at 5 cm on a lathed drum with a
round door, glowing round windows, a chimney and toadstools. Run 2 reads better
because its windows flank the door with a small one above to make a real facade
under a curled cap lip, beside a lantern and a flowered lawn. Run 1 sets its
windows 60 degrees off the door and leaves the door almost alone in the front
and hero views. Run 2 gives the front view the facade that both round 3 runs
lacked.

Failures:

1. Both runs, pass 1: the cap rim hid openings in the wall. Run 1 raised the
   house 0.5 m and run 2 shrank and raised the cap
2. Run 2, pass 1: the report showed 2 pieces because the lantern light floated
   inside an open `boxFrame`. Pass 2 added plates to anchor it
3. Run 2, pass 2: the build failed with
   `error: cottage/flowers b: set points must be distinct, not [-3.425, 0.025, -0.925] twice`
   from hand-scattered flower points. Pass 3 deduplicated them
4. Run 1, final pass: the front view shows the door alone with two windows
   edge-on at the silhouette. The hero shows one window tucked in the cap's
   shadow
5. Run 1, final pass: the cap's flat underside hides the gills. They show only
   as a thin tan line
6. Run 2, final pass: the side windows' extruded frames jut past the curved wall
   as dark slabs at the silhouette. The session noticed and left them

Lacked:

None

Missed in the skill:

1. Run 1: the windows went at 60-degree offsets though the skill says the
   prompt's detail faces `+z` with nothing between it and the hero corner.
   Neither the front nor the hero view shows a window face-on beside the door
2. Run 2: the session scattered lawn flowers with a hand-written LCG and three
   `set()` calls. A coat on the ground's `+y` side with
   `speckle(grass, [yellow, purple, white])` gives the same scatter in one step
   and would have saved the failed pass
3. Run 2: the window frames are flat extrusions of a planar circle and jut past
   the curved wall at their sides. Intersecting each frame with a slightly
   larger lathe of the wall would have kept it flush

Colors: Run 1's plaster noise at scale 0.4 makes large dark-edged blotches
across the wall in place of a fine texture. In the right view they look like a
face. Its gills barely show in any view. Run 2's chimney `cells` pattern with a
dirt border reads muddy brown rather than stone. Its plaster goes gray in the
upper half under the cap.

### 9. Potion

| Run | Verdict | Passes | Failed | Minutes | Voxels           |
| --- | ------- | ------ | ------ | ------- | ---------------- |
| 1   | good    | 2      | 0      | 1.2     | 19,416 at 4 mm   |
| 2   | good    | 2      | 0      | 1.0     | 66,268 at 2.5 mm |

Both runs built a lathed flask with tinted glass, a darker lip, a gradient red
potion under a top coat and a speckled cork. Run 2 reads better because its 2.5
mm grid gives a rounder bulb, a round tapered cork and a rope collar that reads.
Run 1 at 4 mm ends with a square cork cap after deleting its jagged twine. The
pink that round 3 saw over run 1's whole potion now sits on both runs' surface
coats.

Failures:

1. Run 1, pass 1: the twine torus at 5 mm came out as a jagged cross-shaped
   collar. Pass 2 deleted it in place of refining it
2. Run 2, pass 1: three bubble spheres sat fully inside the potion at
   `0 exposed` and never showed. Pass 2 removed them in place of moving them to
   the surface
3. Run 1, final pass: the cork cap is a blocky square box 8 voxels across and
   the curved potion sides stair-step through the glass in the hero. The session
   named both flaws in its final message and stopped

Lacked:

None

Missed in the skill:

1. Both runs: Color guidance item 4 says a glow that reads pale takes a lower
   `emissiveStrength` or a darker `baseColor`. Both left the potion's top coat
   reading pink in place of red
2. Run 1: the session stopped after naming the square cork though SKILL.md calls
   the model done only when no view shows a flaw a review can name. Run 2's 2.5
   mm voxel rounded its cap

Colors: Both runs' potion surface reads pink rather than red. In both, the empty
upper bulb reads as one flat pale blue in the front and right views. In run 1 it
could pass for a second liquid. Run 2's glass reads faint in the hero. A
diagonal smudge crosses its potion surface in the top and hero views.

### 10. Cart

| Run | Verdict | Passes | Failed | Minutes | Voxels           |
| --- | ------- | ------ | ------ | ------- | ---------------- |
| 1   | good    | 2      | 0      | 1.5     | 14,010 at 2.5 cm |
| 2   | good    | 2      | 0      | 1.2     | 10,280 at 2.5 cm |

Both runs built an oak cart at 2.5 cm with iron-capped stakes and four spoked
wheel parts under iron tires. Run 2 reads better because its slatted walls and
dark walnut wheels stand out against the light oak bed. Run 1's farm wagon has
solid blotchy walls and one-voxel spokes that fragment in the hero.

Failures:

1. Run 1, pass 1: the tongue, crossbar and ring sat below the frame and
   voxelized as a detached second piece. Pass 2 raised them onto the frame
2. Run 1, final pass: the spokes, one voxel thick along the axle, read as
   scattered fragments in the hero. The oak spokes and noisy felloe blend into
   the oak bed
3. Run 2, final pass: seen side-on, the spokes crowd into a dark mass around the
   hub. The front and rear wheels nearly touch across a 0.2 m gap

Lacked:

None

Missed in the skill:

1. Run 1: the session saw the blotchy wall grain in the pass 1 renders and left
   it. The skill documents the grain's `warp` and `period` for tightening boards
   into streaks

Colors: Run 1's wall grain reads as mottled camouflage rather than board
streaks. Its oak wheels nearly match the oak bed and do not stand out in the
hero. Run 2's end boards still read as blotches in the right view. Its walnut
spokes and hub merge into one dark mass from the side.

### 11. Guitar

| Run | Verdict | Passes | Failed | Minutes | Voxels          |
| --- | ------- | ------ | ------ | ------- | --------------- |
| 1   | good    | 2      | 0      | 3.9     | 70,093 at 4 mm  |
| 2   | good    | 2      | 0      | 4.2     | 243,672 at 4 mm |

Both runs built a life-size steel-string acoustic at 4 mm with an abalone
rosette and six one-voxel strings. Run 1 reads better because its body is hollow
under a two-voxel top and its outline has a defined waist. Run 2 adds finer
grain, purfling and a real heel, but its solid body fakes the sound hole with a
painted pocket. Both runs now carry the strings to the six tuner posts where
round 3's run 2 stopped them at the nut.

Failures:

1. Run 1, pass 1: four tuner buttons floated off their shafts and left 5 pieces.
   Pass 2 thickened the shafts
2. Both runs, final pass: the six strings fuse into one band over the fretboard
   in the hero. The plain silver strings merge with the silver frets into a
   ladder
3. Run 1, final pass: the vesica pickguard reads as an oversized leaf
4. Run 2, final pass: the body stays solid. The sound hole is a 22-voxel dark
   pocket whose stepped walls show in the close-up
5. Run 2, final pass: the two-circle outline gives a pear-shaped upper bout with
   a shallow waist

Lacked:

None

Missed in the skill:

1. Both runs: the skill names two metals side by side as a pitfall, yet plain
   strings and frets share `mat.silver`. In the hero the treble strings and
   frets read as one ladder
2. Run 2: the body stays solid though carving an extruded 2D offset or a `shell`
   hollows it. The sound hole became a painted pocket and the model carries 3.5
   times run 1's voxels

Colors: Run 1's top grain reads as blotchy vertical streaks more than fine
spruce. Run 2's fine spruce bands nearly vanish at full-view scale. Its top
reads close to one pale tone.

### 12. Candelabra

| Run | Verdict | Passes | Failed | Minutes | Voxels         |
| --- | ------- | ------ | ------ | ------- | -------------- |
| 1   | good    | 4      | 0      | 2.4     | 26,960 at 4 mm |
| 2   | fair    | 6      | 0      | 2.5     | 15,882 at 5 mm |

Both runs built an iron candelabra at 4 and 5 mm with a twisted square stem,
lighter painted corners and torus-arc U arms with scrolls. Run 1 reads better
because its round piece sets four arms on the diagonals under gold flames that
read as flames. Run 2's planar piece looks best head-on, but it collapses to a
post from the side and its flames read as banded candy corn. Unlike round 3's
run 2, neither run puts all five candles on arms.

Failures:

1. Both runs, final pass: four arms carry four candles and the fifth candle
   stands on a center column. The prompt asked for five candles on curling arms
2. Run 1, pass 2: a `coat` with sides `+y` within the twisted stem lit only 208
   scattered cells and did not show the twist. Pass 3 painted the twisted
   corners instead
3. Run 1, pass 2: enlarging the flames pushed the wicks to `0 kept`. Pass 3
   moved the wick step last
4. Run 2, pass 2: carving the melted candle tops cut the wicks free and left 6
   pieces. Pass 3 lowered the wicks to reattach them
5. Run 2, pass 4: the flame gradient's `from` and `to` took world heights, but
   the pattern reads the flame's pre-translate frame. The bands landed wrong
   until pass 5
6. Run 1, final pass: stray one-cell nubs on the stem silhouette make it read
   knotted rather than twisted in the hero. The top collar renders as a plain
   square block
7. Run 2, final pass: the whole piece lies in one plane. The right view
   collapses to a single post and the top view to a line
8. Run 2, final pass: the flames read as round blobs with a wide orange brim,
   closer to candy corn than flame. A tall black wick gap separates them from
   the candles

Lacked:

1. Both runs: a coat that selects a shape's edges or corners. Both sessions lit
   the stem's corners by subtracting a rotated box or a centered cylinder from
   the twisted bar

Missed in the skill:

1. Run 2: SKILL.md's Pattern frames section says a pattern reads the frame the
   shape was built in, before `translate`. The session set the flame gradient in
   world heights and spent pass 4 on the misplaced bands

Colors: Run 1's iron reads as one flat dark taupe on the arms and base despite
the noise. Its candles are flat ivory. Its flames are a uniform yellow block
with a single orange cell column in place of a graded core. Run 2's near-black
arms, stem and neck merge into one silhouette in the hero. Its flames stack
three hard flat bands of yellow, orange and red rather than a glowing core.

### 13. Knight

| Run | Verdict | Passes | Failed | Minutes | Voxels           |
| --- | ------- | ------ | ------ | ------- | ---------------- |
| 1   | good    | 3      | 0      | 3.4     | 2,700 at 6.25 cm |
| 2   | fair    | 2      | 0      | 3.5     | 1,748 at 6.25 cm |

Both runs built a 2 m knight exactly 32 voxels tall at `--voxel-size 0.0625` in
PICO-8 colors, with rigged head, arm, and leg parts. Run 1 reads better because
its raised sword, front-facing kite shield, and white, lavender, and dark grey
armor ramp give a game-ready character from every view. Run 2 stands as a stiff
mannequin with a side-facing shield and a crest that reads as a fez.

Failures:

1. Run 1, final pass: the helm runs oversized for the body. The shield's navy
   field reads as four window panes
2. Run 2, pass 1: the eye and breath slits read as a smiley face, while stray
   white stripes crossed the knees. Pass 2 merged the slits into a T visor and
   dropped the knee shine
3. Run 2, final pass: the heater shield faces `+x` off the side of the arm. The
   front view shows it only as a grey sliver
4. Run 2, final pass: the crest reads more as a fez than a plume because it
   stands as a tall pink-topped block
5. Run 2, final pass: the arms hang straight beside a body only 8 voxels deep.
   The sword dangles point down beside the leg
6. Run 2, final pass: the session stopped after one fix pass with the shield and
   the crest unaddressed

Lacked:

None

Missed in the skill:

None

Colors: Both runs' light grey armor leaves the helm and torso sides as single
flat faces. Run 2 spreads that one tone across its helm, arms, legs, and torso.
Both top views read as one white sheet from the `+y` coats. Run 1's red tabard
stays one flat panel. Its dark purple cape reads as a flat slab from behind. Run
2's blue tabard stays flat, while the red and dark purple bands on its cape read
as candy stripes rather than folds.

### 14. Pocket watch

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | fair    | 4      | 0      | 4.5     | 97,066 at 0.5 mm  |
| 2   | fair    | 5      | 0      | 4.3     | 149,341 at 0.5 mm |

Both runs built a skeleton watch at 0.5 mm voxels with a full wheel train, a
balance with a blue hairspring, and an alternating torus-link chain. Run 2 reads
better because its knurled case fills the hero and its multi-metal movement
stays legible under bright bridges. Run 1's bridges and long hands cross most of
the train, while its watch shrinks in the hero beside a chain that reads as
braided rope.

Failures:

1. Run 1, pass 2: the hands, a jewel, and a balance cock screw floated as 3
   pieces. Pass 3 moved the hand cap back and added a cock screw
2. Run 1, pass 3: a 32-voxel screw floated off the shortened train bridge. Pass
   4 moved it onto the bridge
3. Run 2, pass 1: the bow floated off the pendant, while corner-touching gear
   tooth tips left 22 pieces
4. Run 2, passes 3 to 5: three passes nudged tooth widths, gear rotations, and
   hand tips. The session dismissed the six 2-voxel specks that remain as
   invisible
5. Both runs, final pass: each link's end bar sits exactly on its neighbor's
   inner edge. Every link fuses with the watch into one piece that both sessions
   took as correct
6. Run 1, final pass: the silver bridges and long blue hands cross most of the
   train. The gears read as a busy tangle in the close view
7. Run 1, final pass: the chain reads as a braided rope that arches rigidly
   above the bow and dominates the hero. The watch takes a small corner of the
   frame
8. Run 2, final pass: the chain stands as a rigid inverted U above the watch.
   Its T-bar hangs in mid-air
9. Run 2, final pass: the diagonal hour markers render as bowties

Lacked:

1. Both runs wanted to place copies along a curve. Run 1 wrote a cubic Bezier
   sampler with an arc-length walker, while run 2 wrote a piecewise line and arc
   path to set each link's position and heading
2. Both runs wanted the report to flag interlocked links that fused. One piece
   read as success although a correct chain reports a piece per link

Missed in the skill:

1. Run 2 cut teeth about 1.2 voxels wide on its 48 and 40 tooth wheels although
   the Resolution section asks about 2 voxels across for tilted strokes. The
   teeth broke into corner-touching specks that cost three passes and still left
   six
2. Neither run asked how many pieces an interlocked chain should report. The
   skill says only that a second piece means a float

Colors: Both runs' `mat.gold` reads olive-brown on the case and chain in the
front, right, and top views and mustard in the hero. Neither session remarked on
the cast. Run 1's silver bridges and steel ratchet sit close in tone and merge
where they overlap. Run 2's custom gilt `#E8C060` barely separates from
`mat.gold`.

### 15. Dragon

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | fair    | 5      | 0      | 5.9     | 122,711 at 2.5 cm |
| 2   | good    | 6      | 0      | 8.3     | 123,002 at 2 cm   |

Both runs coiled a dragon in a ring on a gold mound from `roundCone` chains
along a parametric spine. Run 2 reads better because its head rests on tucked
forepaws beside a spade tail, while its crimson wings raise wrist knuckles that
break the outline in the right view. Run 1 has the stronger head close-up, but
its wings flattened into plum stripes that leave a low green loaf from the front
and right.

Failures:

1. Run 1, pass 2: the lid and mouth paints wrote 0 cells, while the nostrils
   wrote 1, because the boxes missed the reshaped skull. Pass 3 fixed the lids
   and mouth, but the nostrils stayed at 1 cell
2. Run 1, pass 2: rewriting the wings into split flank membranes lost pass 1's
   draped silhouette. No later pass restored it
3. Run 1, pass 3: the smoke curl missed the snout. It stayed a 90-voxel floating
   piece that reads as a bone or stick
4. Run 1, pass 4: removing the floor left its loose coins as separate 4-voxel
   pieces. The piece count rose from 10 to 37 and was never addressed
5. Run 1, final pass: the head is small and hard to pick out in the hero
6. Run 2, passes 3 to 5: spine tips cast off as 1-voxel pieces until pass 5
   swapped the cones for `roundCone`s with a 0.013 m tip
7. Run 2, pass 5: the smoke curl read as a white bone beside the head. Pass 6
   deleted it rather than reshaping it
8. Run 2, final pass: the wrist knuckles read as two red loops or ears in the
   hero and front views. The body runs as a thick uniform worm that sits small
   in the frame

Lacked:

1. Both runs wanted a sweep or tube along a sampled curve. Run 1 unioned
   `roundCone`s between points on a parametric ring, while run 2 chained 48
   `roundCone`s on a spiral with a hand-written thickness table

Missed in the skill:

1. Run 1 placed gems and coins at the analytic `pileY` height on a displaced
   heap although the skill sends gems on a pile to `coat` with `sides: ["+y"]`.
   Many float as separate pieces
2. Run 1 joined legs, neck, and haunch to the coil with plain `union` rather
   than `smoothUnion`. The haunch reads as a sphere stuck onto the ring
3. Neither run used the `bend` the skill names for horns. Run 1's straight
   `roundCone` horns render as flat ivory planks in the head close-up
4. Run 1 never acted on the 1-cell nostrils or the floating smoke although the
   Checks section flags both
5. Run 2 set wing bones of `#4E1A22` on a crimson membrane of close hue and
   lightness although the color checks warn that such colors merge. The finger
   bones barely separate
6. Run 2 built its fangs as cones under 3 voxels in radius although the
   Resolution section warns that such details vanish. They kept 2 cells and do
   not show

Colors: Run 1's plum membrane and dark green spines merge on the back. Its grey
smoke reads as bone against the white background, while the large gold heap
dominates the frame. Run 2's dark red wing bones blur into the membrane. Its
ruby, sapphire, and emerald speckle reads as confetti across the hoard, while
its silver goblet nearly matches the white background.

### 16. Bridge

| Run | Verdict | Passes | Failed | Minutes | Voxels           |
| --- | ------- | ------ | ------ | ------- | ---------------- |
| 1   | fair    | 3      | 0      | 3.4     | 801,950 at 0.1 m |
| 2   | good    | 2      | 0      | 2.5     | 783,981 at 5 cm  |

Both runs set the bridge along x over a river along z on a grass and dirt
diorama, with parabolic cables of chained capsules and hangers looped up to the
cable curve. Run 2 reads better because its cobbled towers stand in the river on
cutwater piers above an asphalt road with a dashed center line, railings, and
fine one-voxel hangers. Run 1's narrow tower faces render as stripes, while its
thick hangers read as a black picket fence. Unlike both round 3 runs, run 2
stands its towers in the water rather than on the banks.

Failures:

1. Both runs, pass 1: displaced tree crowns shed stray voxels as separate
   pieces. Pass 2 lowered the amplitudes
2. Run 1, pass 1: the riverbed coat wrote 0 cells because its `within` held only
   the carved channel. Pass 2 grew it with `offset(0.25)`
3. Run 1, pass 2: widening the towers to 1.2 m left the portal carve at its old
   depth. A stone skin blocked the deck until pass 3 deepened the carve
4. Run 1, final pass: the towers' narrow `+z` faces read as heavy dark and tan
   stripes in the front and hero views
5. Run 1, final pass: the 2-voxel square hangers come out as thick as the
   cables. They read as a dense black picket fence
6. Run 2, final pass: the cables run about 2.6 voxels across. Their slopes bead
   into a chain look in the hero

Lacked:

None

Missed in the skill:

1. Run 1 swapped pass 1's clean `cells` pattern with `border` for mortar joints
   painted on a hand-laid 0.5 m grid without checking where they met the faces.
   Whole courses on the narrow tower faces came out solid mortar

Colors: Run 1's tower `+z` faces alternate full-width dark mortar and tan bands.
Its near-black cables and hangers read as one heavy mass. Run 2's cream tower
caps and ledges stay one flat color, while its asphalt reads as a plain dark
slab broken only by the center line.

### 17. Chess set

| Run | Verdict | Passes | Failed | Minutes | Voxels               |
| --- | ------- | ------ | ------ | ------- | -------------------- |
| 1   | good    | 4      | 0      | 3.3     | 2,557,772 at 1.25 mm |
| 2   | fair    | 4      | 0      | 3.4     | 295,596 at 2.5 mm    |

Both runs lathed five piece kinds and built a knight once per color as parts,
then placed all 32 on a correct board with a1 dark and each queen on her color.
Run 1 reads better because at 1.25 mm its turned pieces hold clean profiles and
its knight reads plainly as a horse head with mouth, mane ridge, and eye. Run 2
stayed at 2.5 mm, where crown points, finials, and ears fall to one or two
voxels and its white knights read as ragged slabs. Both runs now build one part
per kind and color, where round 3's run 1 built 32 separate parts.

Failures:

1. Run 2, pass 1: each queen's top sphere floated free, leaving 3 pieces. Pass 3
   attached it with a neck cylinder
2. Run 2, pass 2: the knight's carved eyes kept 0 cells. Pass 3 painted them
   instead, at 2 cells each
3. Run 2, final pass: at about 14 voxels across, the queen's crown points, the
   finials, and the knight's ears fall to one or two voxels. Every piece reads
   lumpy
4. Run 2, final pass: the white knights read as ragged upright slabs in the
   hero. Only the black knights show a horse head with ears
5. Run 2, final pass: the close-up framed f1 and g1 with f8 and g8, leaving the
   pieces a few percent of the frame. The session signed off on a knight it
   could not judge

Lacked:

1. Both runs wanted to turn a placed part 180 degrees for the black side, but a
   part takes only an offset and a pivot. Run 1 passed a facing angle into its
   knight builder, while run 2 wrapped each shape in a `turned()` rotate

Missed in the skill:

1. Run 2 stayed at 2.5 mm with queen crown spheres 1.4 voxels in radius and
   finials and ear cones under 3 voxels, although the Resolution section says
   round details that small read as a plus sign or block

Colors: Both runs' ebony pieces read as flat near-black silhouettes with little
form in the hero. Both runs' ivory grain shows as vertical streaks rather than
turned rings. Run 2's knight mane paint `#B8AD92` shows as a flat grey-tan
stripe on the ivory side rather than a carved ridge.

### 18. Ramen stand

| Run | Verdict | Passes | Failed | Minutes | Voxels          |
| --- | ------- | ------ | ------ | ------- | --------------- |
| 1   | good    | 2      | 0      | 2.5     | 350,323 at 2 cm |
| 2   | good    | 4      | 0      | 4.2     | 425,109 at 2 cm |

Both runs built a stall facing `+z` under a rooftop pink RAMEN sign of
`polyline` strokes in a cyan border, with a hand-drawn katakana RAMEN blade sign
on the right wall. Run 2 reads better because its close-up shows real bowls with
egg, chashu, nori, and scallions, while its rust-speckled roof, red noren, and
mottled street with puddles give it more grit. Run 1's lettering runs bolder,
but its counter holds tiny flat disc bowls and its kitchen pots read as flat
grey blocks.

Failures:

1. Run 1, pass 1: the blade sign and both stockpots floated as 4 pieces. Pass 2
   hung the sign and grounded the pots
2. Run 1, final pass: the three bowls are tiny flat discs. The pots show no rim
   or broth from the front
3. Run 2, pass 1: the steam lobes split into blobs and single voxels, while the
   three torus stool rings floated, leaving 11 pieces
4. Run 2, pass 2: spokes one cell thick still left the stool rings detached at 5
   pieces. Pass 3 thickened them to 2 cells
5. Run 2, pass 3: the close-up showed the soup cylinder poking through the bowl
   walls. Pass 4 replaced it with a clipped sphere
6. Run 2, final pass: the steam reads as lumpy grey ghost shapes in the front
   view
7. Run 2, final pass: a cable pokes out diagonally past the upper-right lantern
8. Run 2, final pass: two bowls serve three stools
9. Both runs, final pass: the closing katakana n on the blade sign reads closer
   to so

Lacked:

None

Missed in the skill:

1. Run 1 stopped after one hero check without a counter close-up. The barely
   readable bowls and flat grey pots went unreviewed
2. Run 2 tied the stool rings on with spokes one cell thick although the skill
   says strokes and tubes need about 2 voxels to hold together. The rings stayed
   detached until pass 3
3. Run 2 put a two-tone ebony and dark metal `noise` behind the lettering
   although the skill asks neighboring colors for contrast. The mottling
   competes with the strokes in the front view

Colors: Run 1's counter body and kitchen merge into one dark navy mass in the
front view. Its stockpots stay flat grey slabs. Run 2's steam reads as opaque
grey blobs rather than vapor, while the dark noise on its sign panel reads as
grime under the letters. Its roof rust looks like orange sprinkles from above.

### 19. Dungeon kit

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | good    | 2      | 0      | 3.5     | 312,446 at 5 cm   |
| 2   | good    | 2      | 0      | 3.5     | 99,320 at 6.25 cm |

Both runs built a four-tile kit of floor, wall, corner, and arched doorway from
a cached `tile()` factory in `kit.ts`, then assembled a two-door room that
imports it. Run 1 reads better as a game kit because its full-cell tiles carry
their own floors, a doorway threshold, and non-walkable footing under the walls.
Run 2's warm flagstone, blue-gray ashlar, and sandstone trim keep every material
distinct, but its floorless edge tiles leave the floor under every wall walkable
and its doorway has no threshold. Run 1 instanced its repeated tiles under
`--frame local` where both round 3 runs wrote a separate object per room tile.

Failures:

1. Run 1, final pass: the final message says the 12 room cells share 8 objects
   under `--frame local`. `vox-doc show` lists 10 objects because only the north
   wall and the floor repeat
2. Run 2, final pass: the arch steps look jagged at 6.25 cm voxels. The floor
   slab's joints show as notches along the room's outer base

Lacked:

1. Rotating a placed part: `part` takes only a pivot and an offset. Both runs
   built a separate part per facing by rotating every shape
2. Material property values in the report: both runs decoded the `walkable`
   values from `room.voxj` by hand. `vox-doc show` lists the property name but
   not its per-material values

Missed in the skill:

1. Run 2 voxelized the room under the default `--frame world` although
   `SKILL.md` says places of one part share an object under `--frame local`.
   `room.voxj` writes the floor tile 9 times and every corner separately

Colors: Run 1's walls, plinth, cap, and floor share one gray hue at close
lightness. The room reads monochrome apart from the tan trim. The noise on its
cap and plinth reads as camouflage mottling rather than weathered stone. Run 2's
sandstone cap and plinth read slightly mottled.

### 20. Crane

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | good    | 2      | 0      | 2.4     | 44,771 at 0.1 m   |
| 2   | good    | 2      | 0      | 2.9     | 30,171 at 0.125 m |

Both runs built a crawler crane in two passes as a four-level part tree of root,
cab, boom, and hook with every pivot on its hinge. Neither run reads clearly
better. Run 1 carries more lower-works and house detail with a striped
counterweight, but its hook is small against the 18 m crane in the hero. Run 2
has the cleaner silhouette with a red boom head, but its upper works stay flat
yellow. Both round 4 runs came out as one piece on the first pass where round
3's run 2 lost a pass to ropes that broke into 93 pieces.

Failures:

1. Run 1, pass 1: `paint vents` and `paint cab side glass` hit 0 cells because
   both boxes sat outside the house and cab faces. Pass 2 moved them in
2. Run 1, pass 2: `--view-select close 'crane/cab'` took the cab's boom and hook
   children. `crane-close.png` shows the whole crane in place of the cab
3. Run 2, final pass: the hook block's white stripes match the white background.
   The block breaks into three loose red bars in the front view

Lacked:

1. A render of a part turned about its pivot: both runs rendered only the rest
   pose. Neither could show a joint's clearance at another angle
2. Whether a part glob in `--select` or `--view-select` takes the part's child
   parts: `SKILL.md` never says. Run 1 lost its cab close-up to it

Missed in the skill:

None

Colors: Run 1's gray counterweight and black deck read as single flat colors.
Run 2's house, deck, and cab share one flat yellow under a flat red boom head.
Its white block stripes vanish against the white background.

### 21. Fish tank

| Run | Verdict | Passes | Failed | Minutes | Voxels          |
| --- | ------- | ------ | ------ | ------- | --------------- |
| 1   | good    | 3      | 0      | 3.6     | 279,847 at 5 mm |
| 2   | good    | 3      | 0      | 3.6     | 132,581 at 5 mm |

Both runs built a 5 mm tank of a black frame, near-clear transmission glass, a
thin water-surface slab, a castle, wavy seaweed, bubbles, and fish as detached
parts. Run 1 reads slightly better because of its purple-roofed stone castle
with an arched gate and red flag and its more visible bubbles. Run 2's five fish
with eye whites are the richest, but its 1 cm bubbles nearly vanish and a
seaweed tip pokes above the water. Neither round 4 run filled the tank or tried
a `#RRGGBBAA` alpha where round 3's run 1 filled its tank with alpha-tinted
water.

Failures:

1. Run 1, pass 1: the 1 cm grid left the fish 3 to 9 voxels long with one-voxel
   eyes. Pass 2 moved to 5 mm
2. Run 1, pass 2: `tang fin` kept 0 cells and the clown fin kept 1. Pass 3
   thickened both
3. Run 1, final pass: four 2-voxel tail tips and two gravel specks float loose.
   The goldfish sits on the castle roof in the hero and close views
4. Run 1, final pass: the final message says the clownfish and goldfish merge
   into the main tank piece. The report lists both as detached pieces
5. Run 2, pass 1: the fish bodies were only 0.024 m thick. Their thin tail tips
   split off as loose pieces until pass 2 deepened them
6. Run 2, pass 1: `#CFEFF2` glass at transmission 0.92 washed the scene pale.
   Pass 2 cleared it
7. Run 2, final pass: `seaweed 3` tops out at 0.365 m above the 0.36 m water
   surface. A lime tip floats over the water in the hero

Lacked:

None

Missed in the skill:

1. Run 1 picked a 1 cm voxel size that left the fish only a few voxels long
   although the skill ties voxel size to the smallest feature. The miss cost a
   pass
2. Run 1 filtered the piece lines out of the report with `grep -v "^  piece"`.
   Its floating tail tips survived the piece check the skill sets

Colors: Both runs' top views read as a flat teal smear under the water slab.
Both runs' gravel reads as busy confetti. Run 2's bubbles fade to near-invisible
pale blue against the pale glass.

### 22. Wizard tower

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | good    | 5      | 0      | 4.1     | 1,109,938 at 5 cm |
| 2   | good    | 4      | 0      | 3.6     | 1,536,149 at 5 cm |

Both runs built a one-piece stone tower at 0.05 m whose spiral stair of per-step
rotated boxes climbs to an indigo cone roof shaped by `bend`. Run 1 reads better
because its cone, split to bend only the upper part, droops like a worn wizard
hat over a plaster chamber with timber posts. Run 2's wedge treads, iron rail,
and crenellated balcony make the cleaner stair, but its one bend plus a 6 degree
tilt reads as a swept horn rather than a crooked hat.

Failures:

1. Run 1, pass 2: a second bend chained on the whole cone moved the eaves down
   to y 9 inside the chamber. The star shrank to 12 fragmented cells until pass
   3 bent only the upper half
2. Run 1, passes 3 and 4: the star split into 3 then 2 pieces because the
   hand-computed tip after two bends missed the cone. Passes 4 and 5 went to
   nudging and enlarging it
3. Run 1, final pass: the flat extruded star reads as a thin gold bar from the
   right. The cell-patterned treads look rubbly with ragged undersides in the
   front view
4. Run 2, pass 1: the model sized for 0.05 m voxelized at 0.1 m. Its 0.12 m
   rails broke into 112 fragments
5. Run 2, passes 2 and 3: the finial star stayed detached above its sphere as
   177 then 3 voxels. Pass 4 lowered it to one piece
6. Run 2, final pass: the tilted eave ring shows as a wavy brim. The balcony
   sits cramped under it

Lacked:

1. A helix or a sweep along a path: run 1 placed 40 rotated boxes with 40 posts
   and 39 capsule rail segments. Run 2 looped 60 rotated boxes with 30 posts and
   capsule rail segments
2. Where a bent shape's tip lands: the report gives a bent shape's bounds but
   not its apex. Both runs hand-computed the tip, and seating the star cost each
   run two passes

Missed in the skill:

1. Run 1 chained two bends on the whole cone although `SKILL.md` says a second
   bend moves the rest of the first bend's arc. The miss dropped the eaves and
   cost a pass
2. Run 2 voxelized at 0.1 m although `SKILL.md` holds a model's details at the
   voxel size it was sized for. Its rails fragmented and the miss cost a pass
3. Run 2 bent its roof once and tilted it although `SKILL.md` documents chaining
   a second bend. The roof sweeps smoothly with no crooked kink

Colors: Run 1's moss paint reads as a flat saturated green band with a hard top
edge around the tower foot. Run 2's ground disc reads as one flat green. Its
balcony floor is a flat tan slab.

### 23. Log cabin

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | good    | 3      | 0      | 4.3     | 1,232,042 at 5 cm |
| 2   | fair    | 3      | 1      | 3.5     | 1,576,235 at 5 cm |

Both runs built a round-log cabin at 5 cm with lit windows, a shovelled path to
the door, a thick roof snow cap, pale-blue icicles, and a gray smoke plume from
a stone chimney. Run 1 reads better because its cabin sinks to its sills in an
organic drift mound under long icicles and a puffy plume. Run 2's flat-sided
snow slab and concrete-like trench read as a diorama tile. Its short icicles
shrink to faint specks in the hero. Run 1's mound buries its cabin where both
round 3 cabins sat partly buried on a sheer-sided snow plinth.

Failures:

1. Run 1, pass 1: the `chinking` step kept 0 of 39,488 cells because the
   touching log cylinders cover it. The final message still claims pale filler
   between the logs
2. Run 1, final pass: two single-voxel roof snow pieces stay. The session named
   them as strays without removing them
3. Run 1, final pass: the cabin sits small inside a 13 m oval mound. The hero is
   mostly white
4. Run 2, pass 2: the build stopped with
   `cabin/snow ground: box round must be at most half the shortest side, 0.44999999999999996, not 0.5`.
   The next pass dropped the round to 0.4
5. Run 2, final pass: the smoke is a single smooth sausage rather than puffs

Lacked:

None

Missed in the skill:

1. Run 1 passed over `0 kept` on `chinking` although the skill's checks read it
   as a step that lost every cell. Its summary claims chinking the model lacks
2. Run 1 coated the stair-stepped chimney shoulder with `coat` and `within`.
   Each step's top reads as a white bar across the stone in the right and hero
   views
3. Run 2 placed its cap and sill snow as hand-sized boxes at guessed heights.
   `coat` with `sides: ["+y"]` and `within` would have found the surfaces and
   caught the log ends and roof edges
4. Run 2 accepted a hero that hides the front's icicles and windows under the
   roof although the skill puts the prompt's detail where the hero can see it.
   It added a low view in place of longer icicles

Colors: Run 1's mound and roof read as one near-uniform white field. Its top
view is a featureless white oval. Run 2's slab sides and trench walls are broad
flat white faces with no strata or shading.

### 24. Valley

| Run | Verdict | Passes | Failed | Minutes | Voxels           |
| --- | ------- | ------ | ------ | ------- | ---------------- |
| 1   | good    | 5      | 0      | 4.2     | 144,137 at 1 m   |
| 2   | good    | 4      | 0      | 8.1     | 239,036 at 1.6 m |

Both runs built peaks from a smooth union of displaced cones coated by height
under `--fill-mode surface`, with a river and pines seated by an analytic
height. Run 2 reads more like a mountain valley because its 400 m horseshoe of
tall snowcapped peaks rings a forest broken into stands under a ragged treeline.
Run 1's 192 m diorama has the better river with sand banks, but its low lumpy
side hills read as a flat basin. Run 2 shipped 9 crumbs where both round 3 runs
ended whole.

Failures:

1. Both runs, pass 1: the river ran past the front and back faces. Run 1
   overshot by 8 m and run 2 by 10 m
2. Run 1, pass 1: a 293-voxel tree floated off a slope because its placement
   height ignored the displacement
3. Run 1, pass 3: displacing the floor left 4 crumb pieces of 1 to 5 voxels
4. Run 2, passes 1 and 2: the noise left 14 crumb pieces. Taller displaced peaks
   raised them to 33
5. Run 2, final pass: three displace octaves cut the crumbs only to 9 pieces of
   1 to 27 voxels. They float on the rock faces
6. Run 2, final pass: the 10 m river sits 4 m down in a gray-walled trench. It
   reads as a ditch and disappears behind trees mid-valley

Lacked:

1. A step that drops pieces under a size: run 1 carved boxes at the crumbs'
   reported bounds that break on any terrain or seed edit. Run 2 shipped its
   crumbs
2. The terrain height at a point: both runs placed trees by an analytic cone
   height that ignores `displace` and the smooth-union fillet. Each sank every
   trunk 10 m to hide the gap

Missed in the skill:

None

Colors: Run 1's snow caps are dotted with brown dirt cells from the rock bands.
Its pines share one dark green range and read as one mass. Run 2's dark blue
river reads faintly from the hero inside its gray trench. Its large gray cut
face on the right reads flat beside the detailed slopes.

### 25. Village

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | good    | 2      | 0      | 7.5     | 457,554 at 0.25 m |
| 2   | fair    | 7      | 0      | 7.2     | 367,556 at 0.25 m |

Both runs ringed a cobbled square holding a well and four stalls with thatched
cottages from a rotating cottage function and set a stone church on its north
side. Run 1 reads better because its twelve cottages vary in roof type, plaster,
timbering, thatch and doors and keep their windows clear of the eaves. Run 2 has
the richer church tower and back gardens, but its eight cottages read as stamped
copies whose low eaves bury the windows. Round 4's better run varied its
cottages in one clean piece, where round 3's varied run shipped six pieces.

Failures:

1. Both runs, pass 1: displaced tree crowns shed floating leaf voxels for 54
   pieces in run 1 and 49 in run 2. Run 1 cleared them in pass 2 by lowering the
   amplitude, while run 2 spent passes 4 to 7 cycling tree seeds to dodge 1 to 3
   voxel floaters
2. Run 1, pass 1: eaves at 2.75 m hid the window tops and buried the door
   lintels. Pass 2 raised the walls to 3.5 m
3. Run 2, pass 1: full-width framing boxes painted timber over every cottage
   wall at 0 exposed. The cottages rendered dark brown until pass 2
4. Run 2, pass 3: doubling the hip pitch made the roofs steep and low. Their
   eaves bury the window tops through the final pass
5. Run 1, final pass: the stepped thatch reads as stacked planks up close
6. Run 2, final pass: the eight cottages share one hip roof shape and one thatch
   color over three wall washes. They read as stamped copies
7. Run 2, final pass: the well's thatched roof reads as a plank bench from
   above. Stall produce hides under the awnings

Lacked:

1. Both runs: `part()` takes only a pivot and an offset with no turn per
   placement. Each run wrote a cottage function that rotates every shape inside
   a per-cottage part

Missed in the skill:

None

Colors: Run 1's stall crates read as flat pine blocks. Run 2's eight thatch
roofs share one golden palette and read as one flat tone across the village. Its
well thatch reads as plain wood.

### 26. City

| Run | Verdict | Passes | Failed | Minutes | Voxels              |
| --- | ------- | ------ | ------ | ------- | ------------------- |
| 1   | good    | 4      | 0      | 5.8     | 1,272,797 at 0.25 m |
| 2   | fair    | 3      | 0      | 6.4     | 819,105 at 0.25 m   |

Both runs scaled the city to 48 m at 0.25 m with crossing streets, crosswalks,
street lights, glass towers and a park, all in the root part. Run 1 reads better
because its park with a fountain, pond and ten trees fills the front block where
the hero shows it. Run 2 built the more convincing nine-block grid and the
better tower silhouettes, but its 9 m park sits in the center block behind a 16
m office. Round 3's nine-block run won on its grid, where round 4's lost on its
hidden park.

Failures:

1. Both runs, pass 1: displaced tree crowns shed floating leaf voxels for 18
   pieces in run 1 and 5 in run 2. Run 2 lowered the amplitude in pass 2, while
   run 1 took passes 2 and 3 to soften the amplitude and then reseed
2. Run 1, pass 4: the session changed the fountain spray color without rendering
   the close-up again. The delivered close-up shows the old dark-blue spray
3. Run 2, final pass: the front-center office hides most of the park in the
   hero. The session claimed the park stays visible
4. Run 2, final pass: the street lamp heads read as oversized boxes

Lacked:

None

Missed in the skill:

1. Run 2: SKILL.md says the detail the prompt is about faces the hero corner
   with nothing between. The park went in the center block behind a 16 m office
   and stayed there after the hero showed it buried

Colors: Both runs' roofs read as flat tar-gray fills across large areas. Run 1's
glass towers read as flat blue planes between steel bands. Run 2 gives each
glass tower one flat pane color. Its 0.25 m office banding reads as siding
rather than concrete.

### 27. Dollhouse

| Run | Verdict | Passes | Failed | Minutes | Voxels          |
| --- | ------- | ------ | ------ | ------- | --------------- |
| 1   | good    | 4      | 0      | 12.1    | 903,283 at 5 cm |
| 2   | good    | 4      | 0      | 8.2     | 844,240 at 5 cm |

Both runs built a furnished cutaway dollhouse at 5 cm with the front wall left
off and the cut edges painted white. Run 1 reads better because the cut through
its front gable opens four furnished rooms and a furnished attic to the lowered
hero. Run 2 fits five rooms, but its side-gabled roof hangs a slope over the
open front that hides the attic and fills the standard hero with roof.

Failures:

1. Run 1, before pass 1: the first draft left placeholder expressions such as
   `.intersect ? ... : ...` that needed a cleanup before building
2. Run 1, pass 1: the mantel sat inside the chimney breast. Its clock, its
   candles and the partition faces read 0 exposed in a 30-piece model
3. Run 2, pass 1: the roof eave overhung the open front. Pass 2 clipped it at
   the cut plane
4. Run 1, pass 2: the living-room plant still floated above its pot as a second
   piece. Pass 3 fixed it
5. Run 2, passes 3 and 4: the session deleted the attic furniture rather than
   reworking the roof. A 1-voxel plant leaf left 2 pieces and cost pass 4 to
   reseed
6. Run 1, final pass: the stairs arrive inside the bathroom. The 30-degree hero
   still hides the sofa and kitchen counters behind the partition
7. Run 2, final pass: the front view shows the stairs as a dark wood block

Lacked:

None

Missed in the skill:

1. Run 2: SKILL.md says the prompt's detail faces +z or +x with nothing between
   it and the hero corner. The session kept the front roof slope over the cut
   instead of turning the gable to it

Colors: Run 1's ceilings render near-black in the close-up because the slab
undersides get no light. Its butter-yellow kitchen partition reads as one large
flat panel in the hero. Its bath floor carries a noisy cyan checker. Its
wardrobe grain reads dark and noisy. Run 2's terracotta study walls and tan
kitchen partition read as large single-color panels. Its roof speckle looks busy
in the top and front views.

### 28. Living room

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | good    | 2      | 0      | 4.8     | 278,056 at 2.5 cm |
| 2   | fair    | 2      | 0      | 5.3     | 338,272 at 2.5 cm |

Both runs built a two-wall cutaway at 2.5 cm with a brick fireplace on the back
wall, a varied bookshelf on the left and an eye-level inside view. Run 1 reads
better because its rust wingback, running-bond brick and medallion rug read
cleanly with every piece square to the axes. Run 2 has the nicer moonlit window
and stone arch, but its armchair turned 30 degrees samples into heavy ribs in
every view.

Failures:

1. Both runs, pass 1: the lamp shade floated above its pole and the mantel decor
   sat buried in the chimney breast. Pass 2 fixed both in each run
2. Run 1, pass 1: a 0.0175 m curtain rod voxelized to `curtain rod 0 cells`. Its
   `cells` brick read as irregular red flagstones until pass 2 laid running-bond
   mortar
3. Run 2, pass 1: a moon rotated off the glass and a log floating above the
   basket joined the shade for 7 pieces. Pass 2 fixed them
4. Run 1, final pass: the close-up rendered with `--view-orbit fit` and a
   shifted `--view-look-at` puts the room off-center in the upper left
5. Run 1, final pass: the armchair faces the camera rather than the fire
6. Run 2, final pass: the armchair turned 30 degrees samples into vertical ribs
   that read as corduroy or wicker. It faces into the room rather than the fire
7. Run 2, final pass: the chair and lamp partly hide the corner bookshelf in the
   right view

Lacked:

1. Both runs: a coursed brick pattern. Each session painted staggered mortar
   from repeated boxes intersected with the brick

Missed in the skill:

1. Run 2: SKILL.md says furniture reads cleanest square to the axes. The
   armchair stayed at -30 degrees with its ribs
2. Both runs: SKILL.md says `grain` on a board built in place far from the
   origin reads as one broad ring. Run 1's bookcase and run 2's mantel and
   bookcase render blotchy instead of grained

Colors: Run 1's armchair front and top read as one flat rust red because the
darker velvet coat covers only `-y` and `-z`. Its rug medallion star reads as a
gold blob. Its basket log ends read as cream lumps. Both runs' walnut reads
mottled rather than grained. Run 2's reads as pink-brown camouflage.

### 29. Spaceship

| Run | Verdict | Passes | Failed | Minutes | Voxels          |
| --- | ------- | ------ | ------ | ------- | --------------- |
| 1   | good    | 2      | 0      | 3.1     | 121,358 at 5 cm |
| 2   | good    | 3      | 0      | 4.0     | 237,801 at 5 cm |

Both runs built a bridge at 5 cm with a chamfered window onto a sealed star box,
a flat radar table and a captain's chair on a dais, each in one piece. Run 1
reads better because it left off the +x wall, front and ceiling to let every
review view see the bridge. Run 2 built the more detailed room and the best
inside view of either run, but its kept +x wall blanks the right view and
half-blocks the hero. Round 4 sealed the starfield and laid the radar flat in
both runs, where round 3's run 2 floated its backdrop and run 1's tilted radar
striped.

Failures:

1. Run 1, pass 1: the near-black chair frame read as a black block in the hero.
   Pass 2 lightened it and added gold trim
2. Run 2, pass 1: the wall screens sat 1 voxel deep in their frames at 0 exposed
   and rendered dark. Pass 2 thickened them
3. Run 2, pass 1: the dais rim paint on `dais.shell` with a half space covered
   the whole platform top in orange. Pass 2 subtracted a ring instead
4. Run 2, passes 1 and 2: the planet's torus ring came out too thin and then sat
   below the window. Pass 3 moved it, but it still reads as stair-stepped rubble
5. Run 1, final pass: speckled stars cover the star box's floor and side walls.
   The hero shows a starry floor plane that gives away the box
6. Run 1, final pass: the back console's speckle at density 0.25 over five
   colors reads as confetti rather than a panel of buttons
7. Run 2, final pass: the right view shows only the outside of the +x wall. The
   hero loses the right station and part of the chair behind it

Lacked:

None

Missed in the skill:

None

Colors: Both runs' wall screens are flat unbroken rectangles. Run 1's chair back
shell reads as a plain gray slab in the front view. Its star box shows as a
featureless dark block behind the room. Run 2's chair shell and arms read as a
black block from behind. Its sky box top is a plain gray slab across the top
view.

### 30. Pirate ship

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | good    | 4      | 0      | 5.8     | 745,018 at 2.5 cm |
| 2   | fair    | 3      | 0      | 5.6     | 221,783 at 2.5 cm |

Both runs built a gun deck at 2.5 cm with three guns in carved ports, two
hammocks, barrels and a lantern with glowing panes. Run 1 reads better because
its curved elliptical hull with ribs reads as the inside of a ship. Run 2 has
the richer roped cannons and props, but its flat red box hull reads as a barn.
Round 4's better run curved its hull at 2.5 cm, where round 3's curved hull at 5
cm turned its guns into blobs.

Failures:

1. Run 1, pass 1: three muzzle spheres and two single lantern-ring voxels came
   loose for 6 pieces. Pass 2 fixed them
2. Run 2, pass 1: both hammocks floated free of their thin clew ropes for 3
   pieces. Pass 2 thickened the ropes
3. Run 2, pass 1: cask heads, hull and bulkhead seams and the door ring
   voxelized to 0 cells because they were thinner than a cell. Passes 2 and 3
   fixed them
4. Run 1, passes 3 and 4: the close-up camera landed inside a hammock twice
   before a third placement worked
5. Run 1, final pass: the deck overhead is an open grid of beams with no
   planking. The top view reads as a roofless tub
6. Run 1, final pass: the ladder rails poke past the beams into empty air. The
   hammocks read as speckled shallow boats rather than slung canvas
7. Run 2, final pass: the hammocks run only about 1 m and bunch in the aft
   corner over the door. The overhead deck covers only the port third
8. Run 2, final pass: the mast stands mid-room and hides the third gun in the
   front view

Lacked:

1. Run 1: a light source the lantern could cast into the room. Its emissive
   panes glow only on their own faces

Missed in the skill:

1. Both runs: SKILL.md says emissive strengths of 2 to 4 add a halo in review
   renders. Both lanterns stayed at `emissiveStrength` 1.6 and read dull

Colors: Run 1's hull, floor, ribs, beams and bulkhead share close mid-browns
that merge the room into one brown mass. Its cannon barrels render near-black
with almost no shading. Its speckled canvas reads as noise. Run 2's pale pine
floor and oak carriages share one beige that melts the carriages into the deck
in the hero. Its red hull bands stay flat across the whole back wall.
