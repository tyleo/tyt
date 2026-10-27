# Trials, round 6

Step S14 of the [checklist](checklist.md) runs the [prompts](trials.md#prompts)
a sixth time after round 5's fixes. [Round 5](trials-round-5.md) logs the round
before, and the [trial
harness](../../../../projects/vxl/trials/README.md) ran all six.

## Changes before round 6

Round 5's findings led to three changes in vxl and its skill:

1. A part reads `detached` only when neither it nor a part below it shares a
   face-connected piece with its parent
2. The grid-cap error gives the grid's bounds in meters
3. The skill's done line says the final review comes from a subagent given only
   the prompt and the PNGs

The voxj format also took the meter as its scene unit, and `object mesh` and
`object render` took `--scene-scale`. No prompt exercises either change.

## Findings

Round 6's analyses recorded 551 items across the 60 runs, against round 5's 480.
Every finding keeps its round 5 number, and new findings start at 111. The
round's gallery lists each finding's items beside the renders.

### What the changes did

The table counts the prompts that showed each finding the changes answered.

| #   | Finding                               | Round 5 | Round 6 |
| --- | ------------------------------------- | ------- | ------- |
| 4   | Ended with flaws the renders show     | 21      | 27      |
| 104 | Millimeters went into meter arguments | 1       | 1       |
| 109 | A part on a sibling read detached     | 1       | 0       |

The three changes landed differently:

1. **Ended with flaws the renders show.** Every run spawned a reviewer subagent,
   and every reviewer saw only the prompt and the PNGs. The reviewers caught
   real flaws: both dragons' missing legs, both watches' overlapping gears, and
   dollhouse run 2's roof gap that a carve had carried for four passes. Sessions
   fixed most of what the reviewers found and then kept editing. Only 4 of the
   60 runs had their final model reviewed, and the shipped models still show
   flaws a reviewer or the session had named. Several sessions told the reviewer
   what to ignore, such as stair-stepping, render lighting, or an intended
   cutaway. Three reused the first reviewer through `SendMessage` in place of a
   fresh one
2. **A part on a sibling read detached.** No part joined to the model through a
   sibling read `detached`: the chess pieces on their board, pirate-ship run 1's
   keg on a barrel, and living-room run 2's cat on the rug. The flag fired on
   knight run 1's cape and living-room run 2's side table, each a voxel off its
   parent, and on the free-swimming fish in both fish tanks
3. **Millimeters went into meter arguments.** No session hit the grid-cap error,
   so round 6 left that change untested. Pocket-watch run 2 passed millimeter
   radii to `circle`, `star`, and `arc` and caught the slip before its first
   build

The reviews doubled the work. A run took a median of 6 passes against round 5's
3, and the round cost $106 against $63. Each review took 20 to 120 seconds,
about a tenth of a run's time. The rest went to the passes the reviews started.
The review left the skill after round 6 because of its cost.

### Still open

Nothing changed for these findings:

| #   | Finding                                           | Round 5 | Round 6 |
| --- | ------------------------------------------------- | ------- | ------- |
| 2   | Large surfaces left flat                          | 22      | 23      |
| 13  | Neighboring materials too close in hue            | 11      | 11      |
| 10  | Features near a voxel thick missed cells or broke | 11      | 10      |
| 24  | Floating crumbs                                   | 7       | 10      |
| 12  | Shapes at an angle alias                          | 4       | 9       |
| 65  | Noise read as camouflage                          | 7       | 9       |
| 19  | The hero view hid the focal element               | 8       | 8       |
| 5   | Dark materials collapse to near-black             | 8       | 7       |
| 6   | Small round shapes read as blocks or plus signs   | 5       | 6       |
| 34  | Paint and coat spill onto neighbors               | 2       | 6       |
| 81  | Sloped snow drew stair-step contour lines         | 6       | 6       |
| 14  | Glass, water, ice, and gems read wrong            | 8       | 5       |
| 9   | Finer grids read better than the guidance         | 4       | 4       |
| 44  | Placing on the cell grid took hand work           | 4       | 4       |
| 7   | Later steps buried earlier details                | 5       | 3       |
| 16  | Signatures misread                                | 3       | 3       |
| 87  | Curves came from hand-sampled points              | 1       | 3       |
| 102 | Steam has no volume material                      | 1       | 3       |
| 3   | Documented tools left unused                      | 1       | 2       |
| 15  | `shades` left the sRGB gamut                      | 0       | 2       |
| 17  | Views the review four cannot give                 | 0       | 2       |
| 20  | Library metals read off hue                       | 4       | 2       |
| 32  | Grain read wrong                                  | 2       | 2       |
| 75  | Flame emissive dropped below the bloom threshold  | 2       | 2       |
| 80  | Neither pirate ship curved its hull               | 2       | 2       |
| 18  | Emissives wash out                                | 2       | 1       |
| 31  | Library rope and grass read off                   | 1       | 1       |
| 36  | Placements share no object                        | 1       | 1       |
| 45  | Box corners in any order                          | 0       | 1       |
| 67  | Large scenes went without close-ups               | 0       | 1       |
| 68  | Lightened gold shades read beige                  | 1       | 1       |
| 69  | Oak read orange and walnut read pink              | 1       | 1       |
| 91  | Market stalls repeated one layout                 | 1       | 1       |
| 92  | No pattern lays brick courses                     | 1       | 1       |
| 99  | A gradient read the frame before the move         | 0       | 1       |
| 110 | The skill never mentions `--flatten objects`      | 1       | 1       |

Shapes at an angle that alias rose from 4 prompts to 9, and paint and coats that
spill rose from 2 to 6. Floating crumbs rose from 7 to 10. Glass, water, ice,
and gems that read wrong fell from 8 to 5, and library metals off hue fell from
4 to 2. Findings 1, 8, 21, 27, 28, 30, 37, 40, 52, 54, 55, 56, 57, 66, 70, 71,
72, 73, 74, 76, 77, 78, 79, 82, 83, 84, 85, 86, 88, 89, 90, 93, 94, 95, 96, 97,
98, 100, 101, 103, 105, 106, 107, and 108 showed in no prompt.

The phase 2 candidates that round 6 showed:

| #   | Finding                                           | Round 5 | Round 6 |
| --- | ------------------------------------------------- | ------- | ------- |
| 11  | Seating details on a surface                      | 8       | 7       |
| 25  | Sweeps, helices, and spirals                      | 4       | 6       |
| 46  | Emissives light nothing nearby                    | 3       | 4       |
| 22  | Rotation on part placement and a one-sided flip   | 2       | 3       |
| 23  | Scene reports too long to read                    | 2       | 2       |
| 26  | A seeded random helper                            | 3       | 2       |
| 35  | Posed parts, plumb joints, and ropes across parts | 2       | 2       |
| 38  | Intended separate pieces read as floating         | 2       | 2       |
| 33  | Report lines without a location                   | 1       | 1       |
| 39  | Inspecting nodes and palette values               | 1       | 1       |
| 42  | Part pivots in the report                         | 0       | 1       |
| 47  | Text                                              | 1       | 1       |
| 59  | Patterns that line up across tiles                | 1       | 1       |

Findings 29, 41, 43, 48, 49, 50, 51, 53, 58, and 60 showed in no prompt.

### New in round 6

111.  **One-voxel pits rendered as black specks.** In 2 prompts, chest run 1's
      heap showed one-voxel pits between its coins as near-black specks until
      its reviewer flagged them. Both tree runs shipped the pits across their
      displaced crowns. The report gives no count or place for such pits
112.  **Parts stored no turn axis.** In 1 prompt, both robot runs left the
      head's turn axis and the arms' swing in comments because a part holds a
      pivot but no axis or range
113.  **A gradient could not follow an outline.** In 1 prompt, guitar run 2
      built its sunburst from seven nested paint slabs because `gradient` runs
      only along an axis. Each band came out flat
114.  **A session wrote memory a later round could load.** In 1 prompt, knight
      run 2 wrote a memory file into its slot's Claude Code project folder. The
      slots keep their paths from round to round, and a later knight run could
      load the note
115.  **No render marked one step's cells.** In 1 prompt, dragon run 1 could not
      find its legs in the renders. The session copied the model with the legs
      in `mat.emerald` and rendered the copy, which cost a pass
116.  **Shallow carved slits read as black holes.** In 1 prompt, chess run 1
      carved the knight's mane grooves and the bishop's mitre slit a few cells
      deep in pale ivory. Both rendered as black holes, and each fix cost a pass
117.  **Profile-built knights read as slabs.** In 1 prompt, both chess runs'
      knights read as thin slabs from the front. Run 1 extruded a side profile,
      and run 2's sculpt came out half as wide as the other pieces
118.  **Gradients over a few stops banded.** In 1 prompt, both fish-tank runs
      faded the backdrop with a `gradient` over a few stops, and the fade
      rendered as hard horizontal bands
119.  **Rig shadows stained a flat backdrop.** In 1 prompt, the review lighting
      cast the posts' and plants' shadows onto the flat aquarium backdrop in
      both fish-tank runs. Both reviewers read them as stains
120.  **`gradient` rejected patterns as stops.** In 1 prompt, valley run 1
      failed with `gradient materials[0] must be a Material, not a Pattern` and
      fell back to plain-material stops
121.  **`elongate` rejected all-zero lengths.** In 1 prompt, valley run 2's
      parametric stretch passed `[0, 0, 0]` to `elongate` for no stretch, and
      the build failed. The session wrapped the call in a conditional
122.  **The report hid what a carve cut.** In 1 prompt, dollhouse run 2's roof
      carve sliced 0.25 m off every wall top for four passes. The report lists
      no steps a carve cut into, and only the final reviewer caught the gap
123.  **Openings showed the white background.** In 2 prompts, openings to the
      outside show the white render background. Both spaceship runs modeled a
      star backdrop that read from outside as a striped or speckled block, and
      both pirate-ship runs' gunports showed white
124.  **`set` rejected repeated points.** In 1 prompt, spaceship run 2's first
      build failed with `set points must be distinct` because its random points
      repeated. The skill says nothing about it
125.  **The harness missed passes run from a script.** In 1 prompt, pirate-ship
      run 1 moved its pass into a `pass.sh` script. The harness counted 3 passes
      against about 8 voxelizes because the voxelize command never appeared in
      the Bash call

### No action

In 26 prompts the two runs chose near-identical plans, and in 28 workarounds
used the tools as intended. In 25 prompts sessions made one-off slips. In 19 the
report and the views caught real defects.

## Log

Round 6 ran on 2026-10-05 with vxl 0.5.0 built from the staged tree over
`56ce75c8` and Claude Code 2.1.289 running Claude Opus 5.5 at high effort. Every
run loaded the skill and called its model done. Of the 60 runs, 50 read good and
10 fair, against round 5's 52 and 8. Different agents judged each round, so the
verdicts compare loosely. The runs took 387 passes, of which 13 failed, and
1,492 turns against round 5's 1,009. A run took a median of 6.3 minutes against
round 5's 3.3, and all 60 cost $106. `~/voxel-trials/rounds/2026-10-05-round-6`
holds the round, and its gallery shows every run's renders.

### 1. Chair

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | good    | 6      | 0      | 2.9     | 12,892 at 1.25 cm |
| 2   | good    | 6      | 0      | 3.5     | 19,582 at 1 cm    |

Both runs built an oak side chair with square legs and an arched crest set with
gems in gold rings. Run 1 gave its 1.25 cm chair a vase splat and a diamond on a
gold cup, where run 2 gave its 1 cm chair a slat back, raised bezels, gold
scrollwork, and amethyst post caps. Run 2 reads better because its bright custom
gems pop against the gold, where run 1's dark library gems leave its crest
reading as gold trim. Run 2 replaced the library gems after its reviewer called
them dark, while run 1 changed the crest after its only review and shipped the
dull gems unreviewed. Unlike round 5, where run 1's bezels fused into one gold
mass, both runs keep each gem in its own gold ring.

Failures:

1. Both runs, pass 1: the lathe legs voxelized notched until later passes
   squared them
2. Run 1, pass 4: flattening the gems with `scale([1,1,0.6])` shrank them so far
   that pass 5 had to resize them
3. Run 1, pass 5: moving the gem centers back pushed the ruby out the back of
   the crest. Pass 6 clipped it with a half-space intersect
4. Run 2, pass 3: pearls placed with `set()` left one voxel detached as a second
   piece until pass 4 nudged them down
5. Run 2, passes 3 to 6: three passes went into pearls along the crest top that
   the reviewer said read as white specks. Pass 6 removed them
6. Run 1, final pass: the small dark gems read as trim on the gold. The diamond
   reads as a pale gray blob perched on top
7. Both runs, final pass: the crest reads as a thin flat board from the side
8. Run 2, final pass: the amethyst finials read as purple flower buds rather
   than cut stones

Lacked:

1. Both runs wanted the report or skill to warn that `mat.ruby`, `mat.emerald`,
   and `mat.sapphire` render dark and desaturated in the review views. Run 1
   never learned it. Run 2 learned it from its reviewer and swapped in custom
   opaque gems that lose the library gems' transmission

Missed in the skill:

1. Run 1 shipped its last passes without the final subagent review the skill's
   done line asks for. The dull gems and gray diamond a fresh eye would likely
   have flagged went out
2. Run 1 left the diamond a pale gray blob although SKILL.md says clear
   materials read faintly and take a darker rim

Colors: Run 1's gems read dark and muddy against the gold and oak. Its diamond
is a flat pale gray. Its legs and stiles read as nearly one flat oak tone beside
the high-contrast seat grain. Run 2's gems are single flat colors with no facet
shading. Its dark oak crest backing reads muddy olive behind the gold scrolls.
Its front cross stretcher splits into a light half and a dark half.

### 2. Lantern

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | good    | 6      | 0      | 4.2     | 154,872 at 2.5 mm |
| 2   | fair    | 12     | 2      | 5.8     | 27,415 at 5 mm    |

Both runs built an iron lantern with a stepped roof over a candle on a brass
dish. Run 1 rebuilt its lantern as a 2.5 mm octagon with amber panes and a
dripping candle, where run 2 kept a square 5 mm frame with an arched bail and a
pillar candle. Run 1 reads better because every view looks through a pane at a
shaded flame, where run 2's 5-cell-wide flame reads as three stacked blocks
behind washed-out peach panes. Unlike round 5, where the front-right post
crossed the flame in both heroes, run 1's octagon clears the candle and run 2
moved its hero orbit to 25 degrees.

Failures:

1. Run 1, pass 1: a corner post blocked the candle in a square 5 mm lantern
   whose flame showed as three hard stripes. Later passes rebuilt it as a 2.5 mm
   octagon
2. Run 2, pass 1: the handle floated free. The bottom rail hid the candle under
   a pale sliver of flame
3. Run 1, passes 4 and 5, and run 2, pass 10: a stepped glow gradient on the
   glass left hard bands or crack-like seams. Both runs reverted to one even
   glow
4. Run 2, passes 2 and 7: the build failed with
   `shades shade 2 must be between black and white, not a lightness of 1.008` on
   the near-white wax. Pass 7 hit it again at `1.0012`
5. Run 2, pass 10: `paint("wax glow", halfSpace("+y", 0.115), ...)` recolored
   everything above the candle top. The report showed the roof, eave, finial,
   and handle at 0 kept until pass 11 limited it with `intersect`
6. Run 1, final pass: the flame stands nearly as tall as the stubby candle. Its
   tip goes salmon pink
7. Run 2, final pass: the flame reads as three stacked blocks of red, yellow,
   and orange in the front and right views. The dish rim and finial look like
   cogs. The orange-painted interior floor reads as an orange floor rather than
   light

Lacked:

1. Both runs wanted a pattern that blends smoothly between materials for glass
   that brightens toward the flame. Both fell back to one uniform faint emissive
   glass
2. Run 2 wanted light that the flame casts onto nearby surfaces. It painted the
   frame's inner faces and floor with a warm emissive iron

Missed in the skill:

1. Both runs tried a stepped glass gradient although SKILL.md says a pattern
   picks one material per cell and never blends. Run 1 lost passes 4 and 5 to it
   where run 2 lost pass 10
2. Run 2 called `shades` on near-white wax twice although SKILL.md says a shade
   past black or white errors. Two builds failed
3. Run 2 painted a bare half space although SKILL.md says `paint` recolors every
   live cell in its reach. The whole roof turned to wax glow
4. Run 2 stayed at 5 mm although SKILL.md says the smallest feature sets the
   voxel size. The flame and dish read as blocks where 2.5 mm would have given a
   teardrop

Colors: Run 1's panes read as flat orange slabs in the front and right views.
Its flame tip is pale salmon. Its wax looks tan through the amber glass. Its
iron noise blotches read as camouflage or stone rather than worn metal. Run 2's
flat pale peach panes wash out the candle. Its interior floor reads as flat
saturated orange paint. Its iron frame is a nearly uniform gray.

### 3. Chest

| Run | Verdict | Passes | Failed | Minutes | Voxels          |
| --- | ------- | ------ | ------ | ------- | --------------- |
| 1   | good    | 8      | 0      | 6.8     | 222,030 at 1 cm |
| 2   | good    | 8      | 0      | 4.8     | 101,382 at 1 cm |

Both runs built an open iron-banded chest at 1 cm with a lid part pivoted on its
hinge over a gold heap holding gems, with coins spilled on the floor. Run 1 adds
a stave barrel lid, drop handles, back hinges, rivets, and a brass lock plate,
where run 2 has a rounded lid lined in red velvet, two-tone gold, four large
gems, and a coin stack. Run 2 reads better because its red velvet and bright
gold give the storybook treasure read, where run 1's near-black iron and dark
brown lining keep its hero dark. Run 1's final reviewer flagged black pits
between the heap coins, which pass 8 filled by raising the mound. Unlike round
5, where run 2's pale oak sat close to its mustard gold, run 2's gold stands out
against the red lining.

Failures:

1. Both runs, pass 1: the coins rendered pale cream and washed out the gold. Run
   2's floor coins also came out square
2. Run 2, pass 1: the iron noise read as stone until later passes smoothed it
3. Run 2, pass 2: a one-voxel stray piece broke off the displaced heap until
   pass 3 changed the `displace` seed
4. Run 1, pass 6: the handle plates sat one voxel off the side wall. The handle
   torus arcs faced the wrong way until pass 7
5. Run 1, pass 7: black one-voxel pits showed between the heap coins until pass
   8 raised the mound
6. Run 2, pass 7: the enlarged heap coins poked past the chest walls until pass
   8 clipped them with an inside-chest box
7. Run 1, final pass: from above, the heap reads as a jumble of overlapping gold
   flakes rather than discs
8. Run 2, final pass: the 5-voxel floor coins read as plus-shaped crosses. The
   gems look oversized and blocky beside the coins

Lacked:

None

Missed in the skill:

None

Colors: Run 1's iron is a near-flat black on every strap and the lid rim. Its
dark brown lining barely separates from the staves. Its heap's side wall reads
as a muddy dark-gold block in the front and right views. Run 2's lid lining is
one flat crimson that reads slightly plastic. The darker gold band at its heap's
sides goes muddy in the front and right views. Its oak grain renders as bold
camouflage swirls on the sides.

### 4. Sword

| Run | Verdict | Passes | Failed | Minutes | Voxels         |
| --- | ------- | ------ | ------ | ------- | -------------- |
| 1   | fair    | 5      | 0      | 3.6     | 10,528 at 5 mm |
| 2   | good    | 6      | 0      | 3.5     | 20,380 at 4 mm |

Both runs built a longsword with a gold crossguard and a wheel pommel holding a
ruby on each face. Run 1 wrapped a leather grip in gold wire at 5 mm, where run
2 made the whole hilt gold at 4 mm. Run 2 reads better because its beveled,
ridged blade with a dark fuller ends in an even 1:2 point, where run 1's tip
steps into a tiered pagoda and its wire samples into yellow flecks. Run 1
shipped its rewritten final pass without a second review. Unlike round 5, where
both tips climbed in coarse stair steps, run 2's point steps evenly.

Failures:

1. Run 2, pass 1: the blade came out short with a stair-stepped taper until
   later passes straightened it to an even 1:2 point
2. Run 1, final pass: the offset edge band and one-voxel bevel step the tip into
   wide tiers that read blocky in the front and hero views
3. Run 1, final pass: the twisted gold wire on the grip samples into scattered
   yellow flecks instead of a spiral. The brown grip leaves the hilt reading
   half gold
4. Run 2, final pass: the grip is thin beside the wide blade. The pommel ruby
   reads as a dull maroon disk in the front view
5. Run 2, final pass: the session added a crossguard ruby the prompt did not ask
   for

Lacked:

None

Missed in the skill:

1. Run 1 shipped pass 5, a large revision made after its only review, although
   the skill's done line asks for a final review from a subagent. The tiered tip
   and flecked wire went out unexamined
2. Run 1 twisted a two-voxel wire 2,400 degrees around a 3.7-voxel-radius grip
   although SKILL.md's sampling notes warn that a part thinner than the grid
   holds on a diagonal turns into ribs and gaps. A helical band or `bands`
   pattern would have read as a wrap

Colors: Run 1's blade face reads as nearly flat mid-gray in the front and hero
views because its grain spread of `0.03` is too tight to show. Run 2's blade
face is one flat `steel[2]` with no shades or noise. Its guard bar is one flat
gold. Its pommel ruby reads dark maroon against the gold.

### 5. Tree

| Run | Verdict | Passes | Failed | Minutes | Voxels           |
| --- | ------- | ------ | ------ | ------- | ---------------- |
| 1   | fair    | 12     | 0      | 4.3     | 39,610 at 0.2 m  |
| 2   | good    | 11     | 0      | 4.7     | 130,926 at 0.1 m |

Both runs built a broadleaf tree with root flare, limbs, and a displaced
ellipsoid crown with darker undersides and lighter tops. Run 2 reads better
because its 0.1 m flared trunk and crooked forking limbs under a deep custom
green dome read as an oak, where run 1's 0.2 m limbs fan up from one point under
a flat-bottomed lime crown like an umbrella tree. Both runs shipped a last edit
no reviewer saw. Run 1's left the black pits and solid hero canopy its reviews
named, while run 2's hung a lobe nearly to the trunk top and hid the limbs.
Unlike round 5, where run 2's crown read as one light green, run 2's custom deep
green gives the crown real shade.

Failures:

1. Both runs, most passes: the crown's `displace` split off floating leaf
   specks, from 2 to 16 in run 1 and 2 to 24 in run 2. Both swept seeds and
   carved the leftovers with one-voxel boxes, a list that broke whenever the
   crown changed
2. Run 1, pass 10: lifting the crown broke the box carve and left 5 pieces. Pass
   11 dropped the carve and swept seeds until seed 7 gave one piece
3. Run 2, final pass: the session waved off the second review's four-roots and
   straight-limbs complaints as describing an earlier version. The reviewer had
   seen the current renders, where only about four roots show from the hero
   angle
4. Run 2, final pass: the unreviewed last edit hung a front-right lobe nearly to
   the trunk top in the hero. It also hid the limbs that pass 8's front view
   showed
5. Run 1, final pass: the hero crown reads as one dense broccoli blob that hides
   almost all the limbs. The roots read as a flat brown star painted on the
   grass
6. Run 2, final pass: the ground is a thin flat disc smaller than the crown. A
   few branch voxels poke through the crown's top
7. Both runs, final pass: black one-voxel pits dot the crown in the top view

Lacked:

1. Both runs wanted a step or voxelize flag that drops pieces under a voxel
   count, so displaced foliage sheds its specks. Run 2 wrote a script that
   parsed the report and appended one carve box per stray piece
2. Both runs wanted the report to count and locate one-voxel pits in a surface.
   Both reviewers flagged the black specks the report never surfaced

Missed in the skill:

1. Both runs changed the crown in a final pass with no subagent review although
   the skill's done line asks for one. Run 1 shipped the pits and solid canopy
   its reviews named. Run 2's unreviewed edit added the drooping hero lobe and
   buried the limbs

Colors: Run 1's `mat.leaf` crown reads bright, yellowish, and saturated in the
hero view, more spring lime than oak green. Run 2's ground disc reads as one
flat grass tone with sparse dots, almost a sticker. Black pits dot both canopies
in the top view.

### 6. Well

| Run | Verdict | Passes | Failed | Minutes | Voxels          |
| --- | ------- | ------ | ------ | ------- | --------------- |
| 1   | good    | 3      | 0      | 3.2     | 141,612 at 2 cm |
| 2   | fair    | 4      | 0      | 2.6     | 124,730 at 2 cm |

Both runs built a round mortared stone well at 2 cm with two oak posts, a gable
roof, a windlass with a rope coil and an iron crank, and a hooped bucket on a
rope. Run 1 reads better because its raised roof shows the rope and bucket over
blue water in the hero, with a plinth, braces, and a ridge cap finishing it. Run
2's bucket seems to float at the rim because the eave hides its rope in the
hero. Unlike round 5, where the eave covered most of the bucket in both heroes,
run 1 raised its roof until the rope and bucket showed.

Failures:

1. Run 1, pass 1: the eave sat 1.7 m up and hid the bucket and rope in the hero
   until pass 2 raised the frame 0.2 m
2. Run 2, pass 1: the bucket walls came out gappy until a later pass thickened
   them
3. Run 2, pass 2: the close-up render failed because an orbit about a point
   takes its distance in meters, not `fit`. The session reran it at 1.2 m
4. Run 2, final pass: the session raised the axle but kept the low eave, so the
   hero still hides the rope. The final message lists it as a known limitation
5. Run 1, final pass: the gable ends stay open. A faint face-like blotch remains
   on the bucket in the front view
6. Run 2, final pass: the pale mortar gives the wall and slate cap a blotchy
   camouflage look rather than laid stone. The mahogany bucket reads as a red
   clay pot

Lacked:

None

Missed in the skill:

1. Run 2 paired `--view-orbit close 30 15 fit` with `--view-look-at` although
   SKILL.md says an orbit about a look-at point takes its distance in meters.
   The close-up render failed

Colors: Both curved walls show strong vertical striping, worst on run 1's shaded
right side. Run 1's bright orange oak posts sit flat against the dark walnut
roof. Run 2's light beige mortar `#9C9384` on the slate cap and gray stones
reads as high-contrast camouflage. Its mahogany bucket reads red-orange rather
than wood.

### 7. Robot

| Run | Verdict | Passes | Failed | Minutes | Voxels          |
| --- | ------- | ------ | ------ | ------- | --------------- |
| 1   | good    | 5      | 0      | 3.0     | 9,952 at 2.5 cm |
| 2   | good    | 4      | 0      | 2.6     | 8,452 at 2.5 cm |

Both runs built a boxy blue toy robot at 2.5 cm with cyan eyes, a red antenna
lamp and the head and arms as parts on a dark neck and dark shoulder axles. Run
1 reads better because its gray head and upper arms stand apart from the blue
body and its arms sit clearly off the torso. Run 2's glowing chest core is the
nicer detail, but its head matches the body's blue and its hands read as black
slabs. A blind review gave run 2 its shoulder axles, slimmer forearms and wider
claw jaws. Unlike round 5's run 2, both runs set the arms off the torso instead
of hugging it.

Failures:

1. Run 1, pass 1: the antenna cylinder at half-voxel radius `0.0125` voxelized
   to 0 cells and left the lamp floating as a second piece
2. Run 2, pass 1: the `0.0375` m antenna-tip sphere voxelized to a 2x2x2 nub.
   Passes 2 and 3 grew it into a readable lamp
3. Run 1, pass 1: the carved grille slots read as a mustache. Pass 2 replaced
   them with a checker mouth
4. Run 1, pass 4: shoulder spheres enlarged to 0.1 m pushed black bumps through
   the orange pauldrons. Pass 5 reverted the size and added an axle
5. Both runs, final pass: the hands read as solid blocks from the front. Run 1's
   claws are plain gray boxes that open only in the side view, and run 2's palms
   and fingers are black slabs
6. Run 1, final pass: the wrist is a single dark voxel that leaves the hand
   looking barely attached
7. Run 2, final pass: the orange collar and shoulder pads crowd together. The
   dark axles between them help only a little

Lacked:

1. Both runs: a way to render the model with a part turned at its joint. Neither
   run saw the head or arms turned
2. Both runs: a turn axis on a part. A part records only its pivot, and the
   head's vertical axis and the arms' swing axis live only in comments

Missed in the skill:

1. Run 1, pass 1: SKILL.md says every feature spans at least 2 voxels, yet the
   first antenna was one voxel wide. Fixing it cost a pass

Colors: Run 1's hips are a flat black band. Its shoulder axles, elbows and
wrists run together in one near-black, and its gray claw boxes read flat from
the front. Run 2's head shares the torso's blue and stands apart only by its
black neck. Its hands, hips and knee bands are flat near-black blocks.

### 8. Cottage

| Run | Verdict | Passes | Failed | Minutes | Voxels          |
| --- | ------- | ------ | ------ | ------- | --------------- |
| 1   | good    | 5      | 0      | 4.0     | 137,277 at 5 cm |
| 2   | good    | 6      | 0      | 4.8     | 267,193 at 5 cm |

Both runs built a red-capped mushroom cottage at 5 cm with a cream stem, a round
plank door in a stone ring, round cross-barred amber windows, a chimney and
stepping stones. Run 2 reads better because its door, transom and windows sit
front and center, with one window big and clean in the hero. Run 1 has the more
charming front view with flower boxes and raised spots, but in the hero its door
sits at a glancing angle and its windows jam under the cap rim. Reviewers caught
run 1's stepped door ring and run 2's door running into its footing, and later
passes fixed both.

Failures:

1. Both runs, pass 1: the windows sat hidden under the cap until later passes
   raised the stem. Run 1's squat cap also hid the door
2. Run 1, pass 3: the reviewer found the door ring read as a stepped tube from
   the corner and the windows crowded the cap and door. Pass 4 fixed them
3. Run 2, pass 2: torus frames on the turned windows came out lumpy. Pass 6's
   clipped rings still read ragged and leave a slab poking out like a shutter
4. Run 2, pass 4: the reviewer found a spot painted onto the wall, the door
   running into the footing and overlapping toadstools. Pass 5 fixed them
5. Run 1, final pass: the cap refills the chimney because the flue carve runs
   before the cap. The top view shows red inside the flue
6. Run 1, final pass: the hero shows the door at a glancing angle half behind
   its ring. The front-left window and its flower box stay partly hidden behind
   the ring
7. Run 1, final pass: the windows jam right under the cap rim
8. Both runs, final pass: the stem carries strong vertical ribbing. Run 1's also
   shows wavy contour bands that read as stair steps rather than plaster
9. Run 2, final pass: the cap carries only a few flat painted spots

Lacked:

None

Missed in the skill:

1. Run 1: the skill says later steps win, yet the flue carve runs before the cap
   add. The cap refills the flue
2. Both runs: the skill asks for the final review after the last change. Run 1's
   pass 5 and run 2's pass 6 changed the windows without one and shipped a
   half-hidden window and a shutter-like slab

Colors: Run 1's window glass reads as flat orange discs. Its stem noise drowns
under the vertical ribbing. Run 2's cap shows large red areas broken only by
noise. Its stem noise also drowns under ribbing, and its gill band is one flat
tan.

### 9. Potion

| Run | Verdict | Passes | Failed | Minutes | Voxels           |
| --- | ------- | ------ | ------ | ------- | ---------------- |
| 1   | good    | 3      | 0      | 1.6     | 68,828 at 2.5 mm |
| 2   | good    | 6      | 0      | 2.3     | 44,864 at 2.5 mm |

Both runs built a corked round flask at 2.5 mm with a lip ring and a potion
filling just past half. Run 2 reads better because its lathed Florence flask has
a foot, a long slim neck and a tall cork over a saturated red potion. Its glass
reads heavily blue, though. Run 1's flask is squat with a stubby neck, and its
potion reads coral or salmon rather than red. Round 5 got the smooth 2.5 mm
flask and the true red from different runs, while round 6's run 2 has both.

Failures:

1. Run 1, pass 1: at 5 mm the flask came out blocky with pinkish liquid behind
   blue glass. Pass 2 halved the voxel size and lightened the glass
2. Run 2, pass 1: the liquid read pink. Passes 2 to 6 deepened it with a dark
   base and a strong emissive
3. Run 2, pass 4: the surface coat spread through 12 layers of the liquid. Pass
   5 limited it to the top layer
4. Run 1, final pass: the potion reads coral or salmon though the session called
   it deep red. The liquid sphere shows concentric stair-step rings through the
   glass in every view
5. Run 1, final pass: the cork top carries an odd chevron texture from its noise
   pattern and an octagonal outline. The short stubby neck makes the bottle look
   squat
6. Run 2, final pass: the empty upper body and neck read as a solid sky-blue
   fill in the front and right views. They could pass for a second blue liquid
7. Run 2, final pass: the cork's vertical grain stripes read as wood. The plug
   turns muddy gray behind the lip

Lacked:

None

Missed in the skill:

1. Run 1: SKILL.md says a glow reading pale takes a darker `baseColor`, yet the
   session set the base to `#C8101E` under a dim `#900010` emissive. The liquid
   stayed salmon

Colors: Run 1's potion reads coral through the glass and pink from the top. Its
stepped liquid surface adds bright ring bands in the front and right views. Run
2's empty flask and neck read as a flat mid-blue closer to tinted liquid than
clear glass. Its cork plug reads gray behind the lip.

### 10. Cart

| Run | Verdict | Passes | Failed | Minutes | Voxels           |
| --- | ------- | ------ | ------ | ------- | ---------------- |
| 1   | good    | 4      | 1      | 2.8     | 18,260 at 2 cm   |
| 2   | good    | 3      | 0      | 2.5     | 12,412 at 2.5 cm |

Both runs built a slatted plank cart with four spoked wheel parts on iron axles
and a drawbar toward the hero. Run 1 reads better because its 0.56 m walnut
wheels at 2 cm carry 10 spokes that read finer and more convincing. Run 2 at 2.5
cm has the better material contrast from a dark board seam and a walnut drawbar,
but its small 8-spoke wheels stair-step badly and its corner irons read as heavy
gray slabs. Run 2 shipped the gear-tooth tire its reviewer flagged. Run 1
carries ten spokes cleanly at 2 cm, where round 5's ten spokes read as a star at
2.5 cm.

Failures:

1. Run 1, pass 1: the build failed with
   `extrude to must be past from -0.05, not -0.08` because a negative `out` sign
   reversed the hub band's z span
2. Run 1, first good pass: the tongue sat at bed height and the spokes ran thin.
   Later passes lowered the tongue to the front bolster and widened the spokes
3. Run 2, pass 1: the sloped `polyline` drawbar sampled into loose stacked
   blocks. Pass 3 replaced it with a straight one
4. Run 2, pass 1: the gap between the wall boards showed through. A later pass
   filled it with a walnut seam set one voxel back
5. Both runs, final pass: the iron tire stair-steps into gear teeth in the hero.
   Run 2 kept it after the reviewer flagged it and called it unavoidable at 2.5
   cm
6. Run 1, final pass: the 1-voxel painted tire shows only at the top and bottom
   of each wheel in the front view. The wheel tops rise past the floor and cover
   the lower side board
7. Run 1, final pass: the small iron corner tabs look stuck on rather than
   strapping anything
8. Run 2, final pass: the wheels span only 20 voxels and their spokes read
   chunky. The iron corner plates paint over most of the walnut corner posts and
   leave small walnut nubs above the walls

Lacked:

None

Missed in the skill:

1. Run 1: SKILL.md says curved members need about 2 voxels across, yet the
   session painted the tire as a 1-voxel ring over the felloe. A separate `add`
   of a 2-voxel ring outside the felloe would have stayed continuous
2. Run 2: SKILL.md says the smallest feature sets the voxel size, yet the
   session kept 2.5 cm on a model of only 12,412 voxels. The tire and spokes
   stayed coarse
3. Run 2: the report showed the `corner iron` paint at 76 of 208 cells kept with
   32 exposed. The session left the paint swallowing the walnut corner posts

Colors: Run 1's softened pine walls read pale and nearly uniform in the hero.
Its dark iron hub caps merge with the dark walnut hubs, and its painted tire
shows as broken gray patches in the front view. Run 2's corner irons read as
large flat gray slabs. Its spokes merge into rims that share one oak tone with
the bed boards and floor.

### 11. Guitar

| Run | Verdict | Passes | Failed | Minutes | Voxels          |
| --- | ------- | ------ | ------ | ------- | --------------- |
| 1   | good    | 6      | 0      | 6.4     | 133,392 at 3 mm |
| 2   | good    | 6      | 0      | 9.8     | 132,556 at 3 mm |

Both runs built a hollow acoustic at 3 mm with a rosette sound hole onto a dark
interior, six one-voxel strings and a 3+3 headstock. Run 2 reads better because
its sunburst dreadnought has square shoulders, a straight fretboard and clean
headstock strings lying flat out to their posts. Run 1's pinched outline reads
as a parlor guitar and its headstock strings come out jagged and doubled, though
only its strings fan from bridge to nut. Both runs keep the strings distinct and
run them to the posts, where round 5's run 2 merged them at 5 mm and its run 1
stopped them at the nut.

Failures:

1. Run 1, pass 5: moving the tuner posts outboard broke the headstock string
   capsules into 9 pieces. Pass 6 thickened them from 0.6 to 0.75 voxels in
   radius
2. Run 2, pass 3: switching the headstock string runs to 8-connected cells left
   diagonal-only neighbors and 19 pieces. Pass 4 dropped the runs onto the face
   plate two cells deep
3. Run 1, final pass: the pinched figure-eight outline with a narrow waist and
   round shoulders reads as a parlor guitar, not the dreadnought the session
   claimed
4. Run 1, final pass: the fretboard edge steps out one voxel partway down the
   neck. The session had noted the step and shipped it
5. Run 1, final pass: the headstock string runs are jagged two-voxel capsules
   that bunch and overlap near the nut
6. Run 2, final pass: a dome-shaped heel sticks up behind the neck joint above
   the upper bout in the hero. The session knew of it and shipped it
7. Run 2, final pass: the strings run parallel from bridge to nut with the pins
   packed in the middle of a wide bridge. The sunburst shows as seven hard-edged
   contour rings

Lacked:

None

Missed in the skill:

1. Run 1: the skill gives detail too small for a shape to `set` at cell centers,
   yet the session drew the headstock string runs as thin capsules. They broke
   apart, then shipped jagged and two cells wide

Colors: Run 1's ebony headstock veneer and fretboard are flat near-black faces.
The `bands` paint on its sides reads as stacked horizontal stripes rather than
wood. Run 2 gives each sunburst band one flat color with no noise. The large
center of its top reads as one flat tan face, and its head plate is flat
near-black.

### 12. Candelabra

| Run | Verdict | Passes | Failed | Minutes | Voxels         |
| --- | ------- | ------ | ------ | ------- | -------------- |
| 1   | good    | 6      | 0      | 5.2     | 31,252 at 4 mm |
| 2   | good    | 7      | 0      | 4.0     | 24,842 at 4 mm |

Both runs built a wrought-iron candelabra at 4 mm with a twisted square stem,
painted corner highlights, scrolled arms and five lit candles on a center cup
and four arms. Run 2 reads better by a narrow margin because its flat fan of
extruded arcs gives the cleanest striped twist and the strongest front and hero
silhouette. Its ironwork is a 3-voxel-deep plate that the right view shows as a
lone candle on a stick. Run 1 is fully 3D with more ornament, but its twisted
stem looks ragged in the orthographic views and its cardinal arms stack the `+z`
candle in front of the center one. Both runs stand the fifth candle on a center
cup, where round 5's run 2 carried all five on arms 72 degrees apart.

Failures:

1. Run 1, pass 1: the center flame floated as a second piece and one wick wrote
   0 cells. Pass 2 fixed both
2. Run 2, pass 1: the model came out in seven pieces because all four arm flames
   and both tip scrolls floated. Pass 2 joined them
3. Run 1, pass 2: the stem-edge paint cylinder stopped at y 0.28 and left the
   stem top unhighlighted. Pass 3 extended it
4. Run 2, pass 2: the flame gradient used world heights on a translated revolve
   and never showed. Pass 3 moved it to the local frame
5. Run 2, passes 3 to 7: the flames went from vesica to `roundCone` and back.
   Wax drips came and went after the reviewer called them misaligned blocks
6. Run 1, passes 1 to 5: the 3-to-4-cell torus arm tubes read as beaded chains.
   Pass 6 thickened them
7. Both runs, final pass: four arms carry four candles while the fifth stands on
   a center cup. The prompt asked for five candles on curling arms
8. Run 1, final pass: in the front and right views the twisted corners sample
   into single-voxel stubs that make the stem look ragged rather than square.
   The cardinal arms put the `+z` candle directly in front of the center one
9. Run 1, final pass: the four inner scrolls bunch into a knot at the stem in
   the hero. The flames are small blocky cupcake shapes with an orange cap
10. Run 2, final pass: the ironwork is a 3-voxel-deep plate. The right view
    shows a lone candle on a stick and the top view a line across the foot
11. Run 2, final pass: the arcs stair-step in the hero

Lacked:

1. Both runs: a spiral or scroll stroke. Run 1 chained torus half-arcs of
   shrinking radius and run 2 extruded a sampled spiral `polyline`
2. Run 2: report data flagging that the flame gradient's range missed the shape.
   The session found it only in a close-up render

Missed in the skill:

1. Run 1: the skill notes that a thin form turned off the axes samples into
   ribs, yet the 3-to-4-cell torus tubes read as beaded chains until pass 6. The
   twisted stem's corners still leave single-voxel stubs in the orthographic
   views
2. Run 2: SKILL.md's Pattern frames section says a pattern reads the frame
   before transforms, yet the session set the flame gradient in world heights on
   a translated shape. Finding why the gradient never showed cost a pass

Colors: Run 1's candle wax reads near-flat white. Its painted stem corners and
`+y` arm highlights speckle the mid-gray iron more than they shape it. Run 2's
near-black iron leaves the arms, curls and pans as flat dark silhouettes shaded
only by stair-step edges. Its plain foot dome reads flat apart from its rim.

### 13. Knight

| Run | Verdict | Passes | Failed | Minutes | Voxels           |
| --- | ------- | ------ | ------ | ------- | ---------------- |
| 1   | fair    | 7      | 0      | 7.8     | 2,405 at 6.25 cm |
| 2   | good    | 4      | 0      | 8.3     | 2,336 at 6.25 cm |

Both runs built a knight exactly 32 voxels tall in exact PICO-8 hexes, with a
great helm, a yellow cross on the tabard, a sword, and a heater shield held
forward, and ended in one piece. Run 2 reads better because red pauldrons, a
navy tabard, dark grey rims, knee bands, and a rounded helm with a nasal guard
give the armor structure. Run 1 stripped its accents back to one light grey over
seven passes and ended a boxy, plainer figure. Unlike round 5, where run 2's
shield hung on the outer side of the arm, both runs turned the shield to face
the front.

Failures:

1. Run 1, pass 3: the session deleted the lavender cuisse lines and elbows that
   pass 2 had added
2. Run 1, pass 4: the arm rewrite left the cape a detached second piece. Pass 5
   reattached it
3. Run 1, pass 7: removing the cream colors also removed the plume tip, cape
   hem, and pauldron tops. The model ended flatter instead of recolored
4. Run 1, final pass: the single light grey armor makes the right view a
   featureless column. The cube helm reads as a box
5. Run 1, final pass: the cape is a flat red board
6. Run 2, final pass: the cape's 1-voxel red and purple bands read as pinstripes
   rather than folds from behind
7. Both runs, final pass: the shield's point stair-steps. Run 1's hangs past the
   hips

Lacked:

None

Missed in the skill:

1. Both runs left the helm, chest, and limbs as broad faces of one light grey
   although the skill flags large single-color faces. Each tried a white `+y`
   coat and removed it
2. Run 1 never rendered the back. Its cape stayed a flat red board

Colors: Run 1's armor is one `#C2C3C7` from helm to legs. Its right view reads
as a featureless grey column, while its cape is one flat red slab. Run 2's helm
and chest sides stay flat `#C2C3C7` broken only by dark grey rims.

### 14. Pocket watch

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | good    | 6      | 0      | 8.0     | 152,073 at 0.4 mm |
| 2   | fair    | 8      | 1      | 8.1     | 217,827 at 0.4 mm |

Both runs built a `+z`-facing open skeleton watch at 0.4 mm voxels, hanging from
a straight chain of alternating links to a T-bar above. Run 1 reads better
because its seven wheels, steel winding wheels, jeweled silver bridges, balance
with hairspring, and minute ticks show the gears clearly in the close-up and
front views. Run 2's chunkier chain reads better as links, but its movement is
sparse. Unlike round 5, where both chains stood rigid with the T-bar in mid-air,
both watches now hang from their chains. Run 1 shipped its chain after the final
reviewer called it beads on a wire.

Failures:

1. Run 1, pass 2: raising the chain pitch from 4.8 to 5.6 mm overlapped each
   link's end with its neighbor's. The correct 23-piece chain of pass 1 fused
   into one piece
2. Run 2, pass 2: the build failed with
   `pocket-watch/crown cap: cylinder round must be at most the radius and half the length, 0.0005000000000000004, not 0.0006`
3. Run 2, pass 3: the session read the report's 8 separate chain links as
   floating geometry. It lengthened the pitch until neighbors shared voxels and
   fused the chain
4. Both runs, final pass: the whole chain reports as one piece with the watch
5. Run 1, final pass: the edge-on links are 2 cells thin and read as rods
   between flat ovals in the front view
6. Run 1, final pass: silver winding wheels overlap a gold rim. The final
   reviewer flagged it and the session left it
7. Run 2, final pass: the balance reads as a gold ring under a hex bridge. A
   long grey pallet lever across the lower half reads as a stray arm
8. Run 2, final pass: the reviewer flagged overlapping gears and the lever as a
   broken third hand. The edits that answered them went unreviewed
9. Run 2, final pass: the hour hand's loop sits over a rose-gold wheel

Lacked:

1. Both runs wanted to place copies along a curve. Run 1 hand-rolled a
   Catmull-Rom spline with an arc-length sampler, while run 2 wrote a
   rise-arc-fall path. Both then hung the chain straight

Missed in the skill:

1. Both runs fused an interlocked chain into one piece although the Checks
   section allows intended extra pieces. Run 2 did it deliberately so neighbors
   would share voxels

Colors: Both runs' `mat.gold` case, bow, and chain read olive-khaki in the front
and right views. Run 1's gold reads mustard in the hero. Run 2 added a rose gold
to separate its side wheels but never corrected the case gold.

### 15. Dragon

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | good    | 15     | 0      | 14.9    | 229,877 at 2 cm   |
| 2   | good    | 12     | 0      | 14.1    | 299,059 at 1.5 cm |

Both runs coiled a dragon asleep around the top of a gold pile from `roundCone`
chains along a hand-computed coil path, with folded wings cut from a shelled
tube around the coil. Run 1 reads better because its thick red body, big wedge
head with swept horns, and tall lathed heap make the hero read at once. Run 2's
teal dragon has the finer lidded head and a wing that folds properly from
behind, but its thin body sits small on a low, wide mound. Unlike round 5, where
run 2 shipped four floating specks, both runs ended in one piece after twice
round 5's passes.

Failures:

1. Run 1, pass 7: nostril smoke puffs floated as 3 pieces and read as a bone.
   They cost three passes before pass 12 removed them
2. Run 1, pass 12: the third review still found no visible legs. Pass 13 rebuilt
   a recolored copy to locate them
3. Run 2, pass 3: the wing scallop subtractions left 26 single-voxel slivers.
   Clearing them took four more passes and three hard-coded carve steps
4. Both runs, final pass: the wing reads as folded only from behind. Run 1's
   reads as a dark raised hood from the front and hero, while run 2's reads as a
   striped purple blanket in the hero and top views
5. Run 1, final pass: the tan dorsal spikes read as scattered specks. The
   forelegs are nearly invisible, and the hind leg from behind is a stubby
   zigzag
6. Run 1, final pass: the closed eye reads clearly only in the face close-up
7. Run 2, final pass: the body is a thin uniform tube with little torso mass.
   The paws barely show
8. Run 2, final pass: the dragon sits small against the wide pile and plinth in
   the hero. The mound reads low and flat rather than heaped
9. Run 2, final pass: the closing message gives 0.885 m at the wing tips. The
   report's bounds top out at 0.825 m

Lacked:

1. Run 1 wanted to see where a buried step's cells sit in the renders. It
   rebuilt a copy of the model with the legs in emerald to find them

Missed in the skill:

None

Colors: Run 1's dark maroon wing merges with the dark red back into one hump in
the front view. Its pale tan spikes scatter as dots rather than a ridge. Run 2's
near-black finger bones read as blanket stripes in the hero and top views. Its
dark teal body merges with the dark back coat into one band.

### 16. Bridge

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | good    | 7      | 0      | 6.0     | 845,451 at 0.25 m |
| 2   | good    | 4      | 0      | 4.6     | 289,734 at 0.5 m  |

Both runs built a river diorama with stone towers, parabolic main cables of
chained capsules, side spans down to anchorages, and looped hangers. Run 1 reads
better because its 0.25 m voxels keep the dark cables smooth and the one-voxel
hangers crisp against red girders and a plank deck. Run 2 went to 0.5 m and
added cutwater piers, truss girders, and lane dashes, but its same-red cables
and hangers fuse into red sheets in the hero. Unlike round 5, where run 1's
hangers read as a heavy picket fence, run 1's hangers now read cleanly. Run 2's
reviewer caught red hangers merging with red girders. The session turned the
girders grey, but no review saw the hanger curtains that followed.

Failures:

1. Run 1, pass 2, and run 2, pass 3: displaced tree crowns shed 17 and 3
   one-voxel pieces. The next pass lowered the displacement amplitude
2. Run 1, pass 7: the build failed because `speckle` took a noise pattern as its
   base. The session passed `mat.dirt` instead
3. Run 2, pass 2: merlons added to the tower tops came off again in pass 3
4. Run 2, pass 2: the review found the red hangers merging with the red girders
   and the midspan cable touching the deck. Pass 3 turned the girders grey and
   raised the sag to 10.5 m
5. Run 1, final pass: the bridge barely rises because the deck sits flush with
   the bank tops about 5 m over the water
6. Run 1, final pass: the 2.5 m thick towers read as slender pillars in the
   front view
7. Run 2, final pass: the dense hangers share the cables' red and fuse into red
   curtains in the hero
8. Run 2, final pass: the 2-voxel cables show coarse stair steps
9. Run 2, final pass: the towers' pale horizontal bands read as dressed
   limestone or concrete rather than blocky stone

Lacked:

1. Both runs wanted a 3D curve or sweep through computed points because
   `polyline` is 2D only. Run 1 chained 64 capsules per cable, while run 2
   chained capsules between points sampled 1 m apart

Missed in the skill:

1. Run 2 never had its pass 3 rebuild reviewed and never rendered a close-up. A
   final review or close view would likely have caught the red hanger curtains
   and the coarse 0.5 m cables

Colors: Run 1's red girder side reads as a near-flat band in the front view
despite faint seams. Run 2's cables and hangers share one flat red, turning the
side-span hanger rows into solid red sheets in the hero. Its tower caps are one
flat cream.

### 17. Chess set

| Run | Verdict | Passes | Failed | Minutes | Voxels               |
| --- | ------- | ------ | ------ | ------- | -------------------- |
| 1   | fair    | 4      | 0      | 5.2     | 1,498,304 at 1.25 mm |
| 2   | good    | 3      | 0      | 5.5     | 573,139 at 1 mm      |

Both runs lathed five piece kinds, built a knight, and placed all 32 pieces in a
correct starting position with a1 dark and each queen on her color, on a grained
wood board with an inlay line. Run 2 reads better because its knight, sculpted
from `smoothUnion` primitives with an arched neck, tapered muzzle, ears, and
mane, reads as a horse in the hero and close-ups. Run 1's knight stays an
extruded side profile that reads as a slab with a snout. Unlike round 5, where
both runs held 1.25 mm, run 2 refined to 1 mm voxels. Run 1's reviewer flagged
near-black pieces hiding detail, and the lightened ebony shipped unreviewed as a
muddy brown.

Failures:

1. Run 1, pass 1: the checker pattern put a1 light. Pass 2 swapped the parity
2. Run 1, pass 2: carved mane grooves on the knight read as holes. Pass 3
   removed them
3. Run 1, pass 3: the reviewer flagged flat knights, near-black pieces, gold eye
   specks on the black knights, and bishop slits reading as holes. Pass 4
   addressed each, but the knight still reads as a slab
4. Run 1, final pass: the knight is a flat extruded profile with no ears or neck
   curve. It reads as a slab with a snout from most angles and as a plain
   rectangle from the front
5. Run 2, pass 1: the knights read as flat slabs in the close-up
6. Run 2, pass 2: scaling the pieces by 1.1 left each bishop's top ball floating
   as pieces 2 to 5 and a stray voxel on the white queen. Pass 3 fixed them
7. Run 2, final pass: the queen's crown points are lumpy voxel clusters with
   dark specks between them
8. Run 2, final pass: the knights still look thin from straight on
9. Both runs, final pass: the ivory pieces show stair-step speckle or vertical
   streaks on their curves

Lacked:

None

Missed in the skill:

1. Run 1 kept the knight an extruded 2D profile intersected with a taper
   although `smoothUnion`, `roundCone`, and `ellipsoid` could have sculpted a
   neck, jaw, and ears. The knight is the model's biggest flaw

Colors: Run 1's ivory pieces are one flat cream that shows their curves only as
stair-step speckle. Its lightened ebony `#3B322C` reads as a muddy dark brown
close to the walnut squares. Run 2's near-black ebony `#211A16` hides black
detail on the walnut squares.

### 18. Ramen stand

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | good    | 5      | 0      | 5.1     | 246,315 at 2.5 cm |
| 2   | good    | 6      | 0      | 9.5     | 209,392 at 2.5 cm |

Both runs built a stall under a rooftop pink RAMEN sign of `polyline` strokes in
a cyan frame, with a yellow katakana blade sign, noren, menu screens, stools,
and bowls. Run 2 reads better because a white katakana RAMEN on an indigo noren,
pink bowl icons on the menus, post neon, crates, cables, and a trash bag make a
denser cyberpunk street. Run 1 is clean but sparse. Its strength-2 neon glows
bubblegum pink rather than hot. Unlike round 5, where both noren hung flat red,
run 2 lettered its noren. Run 1's reviewer read the steam as ghostly figures and
the session dropped it, while run 2 shipped steam that still reads as grey
stacks.

Failures:

1. Run 1, passes 1 to 3: the steam over the pots rendered as grey blobs and then
   as grey X shapes from `polyline` wisps. Pass 4 deleted it
2. Run 2, pass 1: the noren and sign legs missed their supports, leaving 3
   pieces. Pass 2 joined them
3. Run 2, pass 3: new street clutter left 4 pieces. Pass 4 rejoined it
4. Run 2, final pass: the semi-transparent steam over the bowls and pots reads
   as solid grey stacks or figurines
5. Run 1, final pass: the RAMEN letters glow pale bubblegum pink rather than hot
   neon
6. Run 1, final pass: the interior and bowls are tiny and dark. The white
   squares on the menu screens read as blank paper
7. Run 2, final pass: stool shadows show as dark ovals on the counter front
8. Both runs, final pass: the blade sign's closing katakana n reads close to tsu

Lacked:

1. Both runs wanted a text or glyph operation. Each built RAMEN and the katakana
   from a hand-built stroke font of `polyline` strokes 2 cells wide

Missed in the skill:

1. Run 1 set its neon `emissiveStrength` to 2 although the skill says strengths
   of 2 to 4 pale the core. The RAMEN letters came out washed-out pink
2. Both runs changed the model after the last subagent review. Run 2 shipped
   steam that reads as grey stacks, while run 1 kept blank-paper menu icons

Colors: Run 1's RAMEN letters read pale pink from strength-2 emission. Its
puddles are flat purple ovals, while its cabinet front and interior merge into
one dark teal mass in the front view. Run 2's puddle reads as a striped pink and
cyan mat rather than wet asphalt. Its steam is flat grey, while its dark navy
roof merges with the dark purple walls in the hero.

### 19. Dungeon kit

| Run | Verdict | Passes | Failed | Minutes | Voxels             |
| --- | ------- | ------ | ------ | ------- | ------------------ |
| 1   | good    | 4      | 0      | 10.3    | 199,882 at 6.25 cm |
| 2   | fair    | 2      | 0      | 4.5     | 329,784 at 5 cm    |

Both runs built a four-tile kit of floor, wall, corner, and arched doorway on a
2 m grid, then assembled an 8 x 6 m room that imports it. Run 1 reads better
because its running bond and flagstone joints continue across tile edges, though
its room has one doorway and no props. Run 2's room adds a second doorway and
four torches, but its `cells` pattern repeats in every tile with mortar breaking
at each edge. Run 1's bonded quoin corners sit flush where round 5's run 1
pillars jutted 10 cm into the room. Run 2 shipped the seam repetition, mortar
spikes under the caps, and stepped arches its reviewer named.

Failures:

1. Run 1, pass 3: jointing the plinth into 1 m stones made it read as slabs
   standing on end. Pass 4 restored the continuous band
2. Run 2, pass 1: the 1.8 m doorway arches stood too low for a character. Pass 2
   raised them to 2.4 m
3. Run 2, final pass: the floor carries dark wedge slivers and light bands at
   tile boundaries. The reviewer's seam repetition, mortar spikes, and stepped
   arches stay unfixed

Lacked:

1. Material property values in the report: run 1 parsed `room.voxj` by hand to
   find its 10 walkable materials. Run 2 never checked which materials carry
   `walkable` true

Missed in the skill:

1. Run 2 stopped with the flaws its reviewer named although the skill counts a
   model done only when no view shows a flaw a review can name. The floor wedges
   and tile seams ship in the final model

Colors: Run 1's heavy dark mortar makes the outer wall faces busy at hero
distance. Its plinth reads as a flat gray strip in the interior views. Run 2's
posts, caps, and plinths share one dark mottled trim that reads muddy against
the pale cobble. Its floor shows flat light and dark bands at tile boundaries.

### 20. Crane

| Run | Verdict | Passes | Failed | Minutes | Voxels          |
| --- | ------- | ------ | ------ | ------- | --------------- |
| 1   | good    | 4      | 0      | 5.7     | 83,973 at 0.1 m |
| 2   | good    | 6      | 0      | 6.4     | 42,962 at 0.1 m |

Both runs built a crawler crane at 0.1 m as a four-level part tree of root, cab,
boom, and hook with every pivot on its hinge. Run 1 reads better because of its
clean four-chord truss, J hook, and red-marked slewing ring and foot pin, though
one flat yellow merges its cab into the house. Run 2 separates a white cab and a
red twin-fall hook block, but its zigzag lacing samples into chunky
stair-stepped X's. Its boom foot pin hides behind the cab. Both runs rig the
boom with pendants where round 5's run 2 left its boom unsupported. Run 1
shipped the merged yellow cab its reviewer flagged.

Failures:

1. Run 1, pass 1: a 2-voxel latch capsule split off as a second piece. Pass 2
   dropped the latch
2. Run 1, pass 1: the gantry legs ran into the operator cab, and the hazard
   stripe's intersect box was too thin. Pass 2 fixed both
3. Run 2, pass 1: the 0.08 m hoist rope at 60 degrees broke into 7 pieces. Later
   passes thickened it

Lacked:

1. A rope tied to two moving parts: both runs' pendants ride the boom. A boom
   tilt pulls their lower ends off the A-frame
2. A render of a posed joint: neither run checked the turned cab or the tilted
   boom for clearance
3. Each part's pivot in the report's part lines: neither run could confirm its
   joints except through its own code

Missed in the skill:

1. Run 1 left the cab, house, deck, and boom one flat yellow although the
   skill's color checks ask for distinct neighbors and broken-up large faces.
   The reviewer flagged the merged cab
2. Run 2 gave its tilted hoist rope a 0.08 m radius although the skill asks for
   about 2 voxels across a tilted member. The broken rope cost pass 1

Colors: Run 1's cab, house, deck, and boom share one flat yellow. The cab reads
as a bump on the house. Run 2's house noise at spread `0.02` is too faint to
see. Its near-black pendants and boom head read as heavy dark masses against the
yellow.

### 21. Fish tank

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | fair    | 13     | 0      | 11.5    | 141,975 at 5 mm   |
| 2   | good    | 9      | 0      | 6.8     | 859,145 at 2.5 mm |

Both runs built a 60 x 40 x 30 cm black-framed tank with gravel, a castle,
seaweed, a bubble column, and a one-voxel water sheet under the rim. Run 2 reads
better because its 2.5 mm grid gives six fish eyes, forked tails, and fins
beside a four-turret stone castle, though its clownfish and neons overlap the
castle in the hero. Run 1 stayed at 5 mm, where the blue tang reads as a disc
and the two neon tetras read as illegible blocks. Neither run's glass washes the
front view blue-gray as round 5's run 2 did, though both top views still read
muddy through the water sheet. Run 2's reviewer caught fish stripes jutting past
the bodies, and pass 8 clipped them.

Failures:

1. Run 1, pass 2: the bubble column merged into the clownfish. Pass 3 moved the
   column
2. Run 1, pass 9: moving the yellow tang fused it to the tank. Pass 10 moved it
   again
3. Run 1, passes 4 to 7 and 9 to 12: each pass mostly moved one fish clear of
   the castle or the surface
4. Run 2, pass 3: the castle bubbles fused with the flag into one floating
   piece. Passes 4 and 5 moved the bubbles and ran the pole into the roof
5. Run 2, pass 5: fins and grass tufts broke into pieces of 1 to 4 voxels. Pass
   6 offset the tail polygons and thickened the tuft tips
6. Run 2, pass 7: the reviewer saw fish stripes jutting past the bodies. Pass 8
   intersected the stripes with the bodies

Lacked:

1. The neighbor a fused piece touched: the report named only the fused steps
   when the yellow tang fused. Run 1 guessed and moved the fish

Missed in the skill:

1. Run 1 stayed at 5 mm although its tetra stripes, fish eyes, and flag pole
   span one voxel against the skill's 2-voxel minimum for a named feature. The
   tetras and blue tang never read as fish

Colors: Both backdrops step in hard horizontal bands. Run 1's front view carries
a dark diagonal post shadow. Run 2's shows faint seaweed and post shadow ghosts.
Both top views read as a washed-out blue-gray through the water sheet. Run 1's
castle reads dim and desaturated in the hero. Run 2's right view tints
everything a pale mint.

### 22. Wizard tower

| Run | Verdict | Passes | Failed | Minutes | Voxels           |
| --- | ------- | ------ | ------ | ------- | ---------------- |
| 1   | good    | 10     | 1      | 6.3     | 191,375 at 0.1 m |
| 2   | good    | 6      | 0      | 6.5     | 121,908 at 0.1 m |

Both runs built a cobbled tower at 0.1 m with a per-step spiral stair and a
purple roof shaped by `bend`. Run 1 reads better because its two-turn stair
shows in every view under a crisp flared hat that curls to a gold spike and orb,
though its stair sides read sawtooth and its rail reads as a crenellated fence.
Run 2 has truer steps with risers, braces, and a sloped capsule rail, but its
turn and a half reads broken in the hero beneath a lumpy, streaky twisted roof.
Run 2's first reviewer saw the stair read as two flat stacked rings, and pass 4
rebuilt it. Run 1 shipped the near-level top turn and busy overlapping turns its
second reviewer named.

Failures:

1. Both runs, pass 1: the 0.2 m rise stacked the stair into two flat rings. A
   0.3 m rise fixed it in run 2's pass 4 and run 1's pass 8
2. Run 1, pass 1: the build failed with
   `SyntaxError [ERR_INVALID_TYPESCRIPT_SYNTAX]: Expected ','` because an edit
   dropped a comma from the corbel cone call
3. Run 1, pass 4: the thin iron lantern rotated 45 degrees broke into 4
   one-voxel pieces. Pass 5 thickened it
4. Run 1, pass 6: `displace` on the ground disc left two 3-voxel grass crumbs.
   Pass 7 dropped the displace
5. Run 1, pass 8: the 0.3 m rise separated the 0.2 m per-step rail boxes into 19
   pieces. Pass 9 thickened them to 0.4 m
6. Run 1, final pass: the top turn reads almost level from the hero. The two
   turns stack busily in the orthographic views
7. Run 2, pass 4: the ground door faced away from the hero. Pass 5 rotated the
   stair start, door, and windows 30 degrees
8. Run 2, final pass: a gray sill stub juts from the wall mid-tower
9. Run 2, final pass: the lower flight reads cut off from the upper turn in the
   hero. The top landing and door show only in the right view

Lacked:

1. A helix or a sweep along a path: both runs built the stair one tread per
   step. Run 1's overlapping boxes read sawtooth-sided

Missed in the skill:

1. Run 2 put the top landing and upper door on the far side from the hero
   although `SKILL.md` faces the prompt's detail toward +z or +x. The spiral
   reads broken in the hero
2. Run 2 added `twist` and `bend` to a banded roof cone although `SKILL.md` says
   its bands read in the pre-transform frame. The shingle bands smeared into
   noisy diagonal streaks

Colors: Run 1's sandstone tread tops and banded sides sit close in hue. Treads
and risers merge into one beige band from most angles. Run 2's two tread shades
barely differ. Its cells stone streaks vertically on the shaded wall. Its
twisted shingle bands read noisy.

### 23. Log cabin

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | good    | 5      | 0      | 6.1     | 2,254,386 at 5 cm |
| 2   | good    | 11     | 1      | 8.6     | 1,317,705 at 5 cm |

Both runs built a log cabin at 5 cm with glowing windows, a stone chimney under
a puff smoke plume, pale-blue icicles, drifts, a trench to the door, and three
snowy pines. Run 2 reads better because its drifts climb halfway up the windows
and to the back eaves while long dense icicles hang along both eaves. Run 1 has
the clearer smoke that darkens to gray at the flue, but its drifts sit below the
window line and its short icicles read as faint blue specks in the hero.

Failures:

1. Run 1, pass 1: the door handle `set` kept 0 cells because later steps buried
   it. Pass 3 moved it after the icicles as two cells
2. Run 1, pass 1: the smoke's `displace` threw stray pieces of 1 to 2 voxels.
   Pass 3 removed them
3. Run 1, final pass: 5 single-voxel pine-needle specks stay as separate pieces
4. Run 2, pass 1: the build stopped with
   `shades shade 2 must be between black and white, not a lightness of 1.0065445068154075`
   because the `#F2F6FB` snow base sat too light for the spread
5. Run 2, pass 6: the hard-coded `carve("loose snow")` box read 0 cells after
   the drift edit moved the stray roof-snow voxel. Pass 7 re-aimed it
6. Run 2, pass 9: moving the drifts after the openings buried the door handle,
   sills, and sill snow to `0 kept`. Pass 10 cut the door hole from the drifts,
   and pass 11 deleted the sills
7. Run 2, final pass: a 494-voxel smoke puff floats as a second piece. The
   session kept it as a breaking wisp

Lacked:

None

Missed in the skill:

1. Run 2 hand-placed its roof, chimney, and sill snow as `add` shapes although
   `SKILL.md` lays a snow layer with `coat`. The displaced roof slab threw stray
   voxels that cost passes 5 and 7 on coordinate carves
2. Run 2 called its smoke necessarily opaque although `SKILL.md` documents
   `#RRGGBBAA` alpha and `transmission`. The session never tried either on the
   pale plume

Colors: Both runs' snow base and drifts read as a flat white wash. Voxel
stair-steps give the drifts their only definition. Run 2's smoke spans only
`#BDBBB8` to `#EBEAE9` and merges with the snowy pines in the hero.

### 24. Valley

| Run | Verdict | Passes | Failed | Minutes | Voxels           |
| --- | ------- | ------ | ------ | ------- | ---------------- |
| 1   | good    | 7      | 1      | 27.9    | 260,351 at 0.5 m |
| 2   | good    | 12     | 0      | 8.6     | 123,085 at 0.4 m |

Both runs built a forested valley of snow-capped peaks and a winding river under
`--fill-mode surface`. Run 1 reads better because its 120 m tile at 0.5 m
carries 70 m peaks that dwarf 452 clumped pines thinning toward the tree line,
though its banked channel steps into visible terraces. Run 2's 64 m tile at 0.4
m is livelier up close with river pools, boulders, and foam, but its pines stand
up to 11 m against 44 m crags beside a huge bare strata wall. Run 1 seated its
pines on true ground heights read back from a terrain-only voxelize where both
round 5 runs sank their trunks 3 to 4 m to hide an analytic guess.

Failures:

1. Run 1, pass 1: the river water extruded past the tile and hung off the front
   edge. Pass 2 clipped it
2. Run 1, pass 5: the build failed with
   `gradient materials[0] must be a Material, not a Pattern` because the session
   nested noise patterns in a gradient
3. Run 1, pass 5: the first V gorge cut too wide and erased the back-right peak.
   Another pass narrowed it
4. Run 1, passes 5 to 7: displacement left 10- and 2-voxel flecks off the back
   summits
5. Run 2, pass 2: a 482-voxel cluster of trunks and crowns floated at the cut
   edge. The fix extended the trunks 6 m down
6. Run 2, passes 2 to 4: displacement left up to 13 crumb pieces through four
   noise seeds
7. Run 2, pass 8: the build failed with
   `elongate lengths must be above zero on at least one axis, not [0, 0, 0]`.
   The chained voxelize reported the stale model without saying so
8. Both runs, final pass: the last crumbs went to carve boxes at hand-coded
   coordinates. Each carve breaks on any seed or shape edit

Lacked:

1. The terrain height at a point: run 1 rebuilt one from a 27 MB raw-json voxj
   of the terrain alone. Run 2 guessed with an analytic `heightAt` and buried
   each trunk 6 m
2. A step that drops pieces under a size: both runs carved their last crumbs by
   coordinates
3. Patterns as `gradient` stops: run 1 fell back to plain-material gradients for
   its meadow, scree, and snow transitions
4. An `elongate` that accepts all-zero lengths: run 2 wrapped a no-op stretch in
   a conditional
5. A short piece line under many parts: run 1's 452 tree parts ran the `from`
   list to thousands of characters and the report to 133 KB

Missed in the skill:

1. Run 1 passed noise patterns to `gradient` although `SKILL.md` types its stops
   as `Material[]`. The build failure cost pass 5

Colors: Run 1's meadow along the river reads flat light green with faint contour
banding. Its summit snowfields read near-uniform white from the top. Run 2's
river reads as one saturated flat blue. Its right cut wall spreads a large bare
expanse of gray strata across the right view.

### 25. Village

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | good    | 4      | 0      | 11.5    | 263,282 at 0.25 m |
| 2   | good    | 6      | 1      | 11.6    | 368,732 at 0.25 m |

Both runs ringed a flagstone square holding a stepped market cross and four
striped stalls with thatched cottages facing it and set a bespoke stone church
with a walled churchyard and a lych gate beside it. Run 1 reads better because
its nine cottages vary in hip or gable roof, timber framing, wall height and
wash color. Run 2's eight cottages share one gabled roof and one wall height and
read as near copies on a mostly empty 60 m field. Round 6's run 2 spread its
touching cottage pairs apart and dropped its dead-end paths where round 5's run
2 left terraces that dead-ended its lanes.

Failures:

1. Run 1, pass 1, and run 2, pass 2: displaced tree crowns threw off single leaf
   voxels for 21 and 5 pieces. Both sessions lowered the amplitude
2. Run 1, pass 2, and run 2, pass 5: one stray voxel on `tree.7`'s crown and 2
   pieces from the moved yew's crown lasted one more pass. A new seed cleared
   run 1's
3. Run 2, pass 2: the first cottage close-up rendered black because the camera
   sat inside geometry
4. Run 2, pass 4: the build failed with
   `village/cottage paths: box max must be apart from min [-0.75, -0.25, 3] on each axis, not [0.75, 0, 3]`
   after `pathLen` went to 0.75. The session dropped the two dead-end paths
5. Both runs, final pass: the four stalls all face +z rather than turning toward
   the cross
6. Run 1, final pass: up close the thatch reads as stacked horizontal planks.
   The chimneys stay chunky red blocks and the spire a tiered ziggurat
7. Run 1, final pass: the porch and lych-gate slate roofs still read as
   stair-stepped slats. The session re-rendered the reviewer's porch fix without
   a second review
8. Run 1, final pass: the hedge border frames the scene like a board edge
9. Run 2, final pass: the cottages differ only in plaster color, door color,
   width and chimney side. They read as copies at a glance
10. Run 2, final pass: the 60 m ground leaves wide empty grass around a compact
    village. The lych-gate roof sits like a canopy over the tower door

Lacked:

1. Both runs: `part()` takes only a pivot and an offset with no turn per
   placement. Each run wrote a cottage function that rotates every shape about y
2. Run 2: `set` takes world points. The session computed each door knob's cell
   by hand with the rotation matrix

Missed in the skill:

None

Colors: Both runs' slate roofs read as flat gray striped bands, as does run 2's
spire. Run 1's perimeter hedge reads as a uniform green frame rather than
planting. Run 2's 60 m grass field fills the hero as one mottled green expanse.
Its `cells` church stone reads as camouflage blotches rather than coursing.

### 26. City

| Run | Verdict | Passes | Failed | Minutes | Voxels              |
| --- | ------- | ------ | ------ | ------- | ------------------- |
| 1   | good    | 3      | 0      | 6.5     | 1,125,511 at 0.25 m |
| 2   | good    | 4      | 0      | 11.2    | 4,296,612 at 0.25 m |

Both runs built at 0.25 m in one root part with crosswalks, street lights,
skyscrapers and a park with a fountain. Run 1 reads better because its compact
48 m four-block tile puts the park in the hero corner and keeps its 24 hooked
street lights and lit windows legible at scene distance. Run 2's 82 m nine-block
city has richer buildings and rooftop clutter, but its park shrinks to a small
green square among the towers and its 48 street lights to specks. Round 5's
better run held a nine-block grid and a visible park together. Round 6 split
them between the runs again.

Failures:

1. Both runs, pass 1: a floating water tower and tree-crown fragments split run
   1 into 20 pieces and run 2 into 13. Pass 2 seated the towers and lowered the
   displacement amplitude
2. Run 2, pass 3: shrinking the tree crowns left one stray leaf voxel. Pass 4
   changed the displace seed
3. Run 1, pass 1: the teal and blue glass towers wore flat black slab crowns.
   Pass 2 replaced them with banded metal crowns and fins
4. Run 1, pass 2: a `cells` pattern on the window glass lit partial windows. The
   reviewer flagged it, and pass 3 lit whole windows from a seeded pick
5. Run 1, final pass: the roofs stay plain flat gray. In the front and right
   views the towers overlap into one mass
6. Run 2, final pass: the park reads as a small green square buried among the
   towers in the hero. The 48 street lights shrink to near-invisible specks
7. Run 2, final pass: the windows are uniform with none lit. The front row is
   mostly squat boxes

Lacked:

1. Run 1: a per-copy random material pick over a `repeat`. The session
   hand-rolled a seeded LCG to light about 20% of the windows as a union for a
   separate paint step

Missed in the skill:

None

Colors: Run 1's roofs share one flat gray `roof` material. Its light-stone,
cream and terracotta facades are flat colors broken only by windows. Run 2's
mottled noise roofs read dirty. Its windows are one uniform blue. Its
street-light glow is too small to register in the hero.

### 27. Dollhouse

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | good    | 7      | 1      | 12.7    | 807,351 at 5 cm   |
| 2   | good    | 6      | 0      | 12.6    | 1,037,678 at 5 cm |

Both runs built a blue dollhouse at 5 cm with the front wall and front gable
removed, a living room and kitchen below, a bedroom and bath above and an attic
under the roof. Run 2 reads better because its larger 10 x 6 m house is densely
furnished and its hero shows every room. Run 1 is tidier, but its central stair
hall cramps the kitchen and hides the bathroom fixtures in a back corner. Round
6's run 2 brought the tub into the hero after both round 5 runs hid it. Run 2's
final reviewer flagged a gap at the eave that led the session to restore 0.25 m
of wall top a pass 2 carve had sliced away.

Failures:

1. Run 1, pass 1: the fascia paint recolored nearly the whole roof off-white
2. Both runs, pass 1: run 1 split into 14 pieces and run 2 left a few props
   floating. Later moves floated run 1's new landing sconce in pass 6 and run
   2's plant in pass 4, and the next pass seated each
3. Run 2, pass 1: the attic slab poked through the roof. A partition blocked the
   view into the rooms
4. Run 2, pass 2: the `above roof` carve started at y = 5.5 instead of the eave
   and sliced 0.25 m off every wall top. It went unnoticed until the final
   reviewer's flag led to pass 6's fix
5. Run 1, pass 4: `sh.scale([-1, 1, 1])` failed with
   `scale factor must be above zero on each axis, not [-1, 1, 1]`
6. Run 2, pass 4: the `attic cut edge` paint hit `0 cells`. Pass 5 fixed it
7. Run 1, final pass: the stair hall eats a wide strip and cramps the kitchen.
   The hero sees mostly empty checker floor in the bathroom because the tub and
   vanity hide in its back corner
8. Run 1, final pass: the bed reads as a dark block with a blue stripe in the
   front view
9. Run 2, final pass: the stairs come straight up into the bathroom. The dark
   slate ridge cap lies as a heavy stripe on the red roof

Lacked:

1. Run 1: a reflection that drops the original. `mirror` keeps both halves and
   `scale` refuses a negative factor, so the session rotated the right-wall
   window 180 degrees about y
2. Run 2: report data naming the earlier steps a carve took cells from. The
   carve that sliced the wall tops looked like any other carve for four passes

Missed in the skill:

1. Run 2: the skill says a board built in place far from the origin with `grain`
   reads as one broad ring. The wardrobe, bed and wainscot were built that way
   and render as camouflage blotches instead of wood streaks

Colors: Run 1's stair-hall walls are flat terracotta. Its bathroom's upper walls
are a flat pale blue-gray. Both read as unfinished plaster beside the patterned
rooms. Run 2's `grain` wardrobe, headboard and wainscot read as mottled
camouflage. Its dark ridge cap reads near-black against the red roof.

### 28. Living room

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | good    | 7      | 1      | 9.3     | 1,031,241 at 2 cm |
| 2   | good    | 6      | 1      | 11.1    | 319,121 at 2.5 cm |

Both runs built a two-wall cutaway with a fieldstone fireplace, a wingback, a
bookshelf of seeded varied books, a patterned rug, a night window and a sleeping
cat. Run 2 reads better because it ends with fewer visible flaws and an
eye-level inside view that shows the room as a person standing in it would. Run
1's 2 cm finish gives finer spines with gilt bands and page tops, but its 45
degree armchair ribs heavily and its globe floats as a smear in the shelf. Run 2
turned its square armchair 25 degrees toward the fire in pass 3 on a reviewer's
suggestion and traded a clean chair for visible ribbing. Round 6's fires glow in
both runs where round 5's run 2 left salmon embers.

Failures:

1. Run 2, pass 1: the build failed with `box max must be apart from min` on the
   mug handle's zero-thickness cutout
2. Run 1, pass 2: the cat close-up failed with
   `view cat's transform orbits a point at a fit distance, and an orbit about a point takes its distance in meters`
3. Run 2, pass 2: the lamp table stood in front of the firebox and hid the fire.
   Pass 3 moved it beside the hearth
4. Run 2, pass 4: the side table came out `detached` one voxel above the floor
   and the globe ring at `0 cells`. Pass 5 fixed both
5. Run 1, pass 5: displaced foliage left 29 floating specks for 30 pieces. Pass
   6 lowered the amplitude
6. Both runs, final pass: the turned armchair samples into ribs. Run 1's chair
   and ottoman sit at 45 degrees and read as corduroy, and the session kept the
   angle by calling the ribs tufted channels
7. Run 1, final pass: the globe sits 0.2 m above its stand and straddles the
   shelf board at 1.66 m. It reads as a speckled smear in the shelf close-up
8. Run 2, final pass: the inside view crops the armchair at the right edge

Lacked:

1. Run 2: a light source. The emissive fire, lamp and sconce glow but cast no
   light on the room

Missed in the skill:

1. Both runs: the skill says a whole object turned off the axes samples into
   ribs and that furniture reads cleanest square to the axes. Run 1 set its
   armchair at 45 degrees from the first pass. Run 2 turned its square chair 25
   degrees in pass 3
2. Run 1: the skill says the report reads more reliably than the PNGs. The
   report's globe bounds already showed it above its stand and inside the next
   shelf board

Colors: Run 1's night window glass is one flat dark navy panel. Its flames read
flat in the front view. Run 2's lamp shade and pillow read as flat blocks. Its
plaster noise reads as camouflage blotches. Its painting canvas is noisy.

### 29. Spaceship

| Run | Verdict | Passes | Failed | Minutes | Voxels          |
| --- | ------- | ------ | ------ | ------- | --------------- |
| 1   | good    | 7      | 0      | 9.2     | 169,023 at 5 cm |
| 2   | good    | 10     | 1      | 10.6    | 248,550 at 5 cm |

Both runs built an open cutaway bridge at 5 cm with a red captain's chair, a
green radar and a window onto stars, a ringed planet and a moon held in a sealed
box behind the back wall. Run 2 reads better because its inside view frames a
channel-stitched chair, a glowing scope table and an octagonal viewport among
richer stations with screens and button grids. Run 1's upright radar disc glows
strongly, but its center-line layout lets the chair hide the radar in the front
view and the radar cover the planet from inside. Both round 6 runs worked at 5
cm where round 5's ran at 2.5 cm. Run 1's planet ring turns lumpy at that size.

Failures:

1. Both runs, first voxelized pass: the starfield stood free far behind the
   room. Run 1's slab showed from every outside view and widened the bounds to
   8.4 m, while run 2's 700 `set` stars on a 12 x 8 m backdrop inflated the
   bounds to 240 x 160 x 198 voxels and left the planet floating
2. Run 2, pass 1: the build failed with
   `bridge/desk buttons: set points must be distinct, not [-2.675, 1.025, 0.875] twice`
3. Run 1, pass 2: the cavity carve ate the window pane at `0 kept`. Pass 3
   stopped the cavity at the wall
4. Run 2, passes 2 to 4: the radar screen tilted toward the chair broke into
   stair steps and buried most of its paint at `36 exposed`. The session then
   laid it flat as a scope table
5. Run 2, pass 3: the new star box left 15 pieces. The next pass reconnected
   them
6. Run 2, pass 8: the session coated the star box's outside with star speckle.
   The hero and right views show a glittering block on the back of the room
7. Run 1, final pass: the chair hides the radar in the front view. From inside,
   the radar covers the planet
8. Run 1, final pass: the sealed hull box shows from outside as a thick gray
   block with broad z stripes
9. Run 2, final pass: the flat radar turns into a thin green rim from eye
   height. The soffit hides the planet in the hero

Lacked:

1. Run 2: SKILL.md never says `set` points must be distinct. The first build
   failed on that rule

Missed in the skill:

1. Run 1: the skill says the prompt's detail faces the hero corner with nothing
   between it and that corner. The chair, radar and window sit on one center
   line and block each other
2. Run 2: the session scattered hundreds of stars as `set` points. The skill's
   `speckle` gives the same stars in one step without the failed build and
   inflated bounds
3. Run 2: the skill says a thin form turned off the axes samples into ribs and
   stair steps. The tilted radar screen cost two passes before the session laid
   it flat

Colors: Run 1's outer hull box reads as wide gray z stripes from a `bands`
pattern rather than plating. Its large back wall faces are near-flat gray. Run
2's star-speckled sky box makes the hull read as space from outside. The dark
front face of its right counter reads flat.

### 30. Pirate ship

| Run | Verdict | Passes | Failed | Minutes | Voxels              |
| --- | ------- | ------ | ------ | ------- | ------------------- |
| 1   | good    | 3      | 0      | 11.4    | 1,253,560 at 2.5 cm |
| 2   | fair    | 6      | 1      | 8.1     | 450,984 at 2.5 cm   |

Both runs built a 2.5 cm gun deck with guns run out through ports, hammocks,
barrels and a brass lantern on a chain. Run 1 reads better because its curved
hull with tumblehome, ribs, knees and six guns makes a convincing deck from its
custom interior cameras. Its standard hero shows only a sealed planked box. Run
2's boxy cutaway shows every item in the hero but reads as a generic timber room
with two guns. Both blind reviewers read the hammocks as rowboats or bowls, and
each session's unreviewed rebuild still renders terraced. Round 5's run 1 lit
its hold with a point light at the lantern. Both round 6 holds sit evenly
daylit.

Failures:

1. Run 2, pass 1: the build failed with
   `noise seed must be a whole number from -2^31 to 2^31 - 1, not 18.5` because
   the hammock seed came from a fractional position
2. Run 1, pass 3: the first wale, tar and ochre coats used whole-side boxes and
   striped the inside walls. Per-side `within` bands fixed them
3. Run 2, pass 3: the ladder rungs repeated along z instead of y. Only one rung
   showed until the fix
4. Run 1, pass 3, and run 2, final pass: the blind reviewer read the hammocks as
   rowboats or bowls. Neither session showed its rebuilt hammocks to a reviewer
   again
5. Run 1, final pass: the hero, top and right views show only a sealed planked
   box with a grating and stripes. The hero gives no hint of the interior
6. Both runs, final pass: run 1's hammocks render as coarse terraced pods that
   dominate the top of the aft view. Run 2's read as flat trays from above and
   terraced slabs from inside
7. Run 2, final pass: the room is a box with a flat floor, a flat partition and
   one hull wall with a slight kink. A deck strip covers a third of the ceiling
8. Both runs, final pass: the lantern lights nothing. Both holds sit evenly
   daylit

Lacked:

1. Both runs: a light source that lights its surroundings. The lanterns'
   emissive flame, glass and panes glow on the lantern alone

Missed in the skill:

None

Colors: Run 1's floor and weather deck show wide alternating flat light and dark
plank stripes. Its barrel heads are flat dark discs. Run 2's broad pale oak
floor reads as a sunlit deck. Its hull and partition noise reads as stains. Its
lantern panes are a flat orange slab.
