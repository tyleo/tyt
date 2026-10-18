# Trials, round 7

Step S19 of the [checklist](checklist.md) runs the [prompts](trials.md#prompts)
a seventh time after steps S16 to S18. [Round 6](trials-round-6.md) logs the
round before, and the
[trial harness](../../../../projects/utilities/vxl/trials/README.md) ran all
seven.

## Changes before round 7

Steps S16 to S18 changed vxl, its skill, and its library:

1. The report counts a step's face-connected pieces and lists after `lost to`
   the later steps that took the step's cells. A `paint` or `coat` line lists
   after `over` the steps it recolored, and a piece line lists the steps whose
   cells reach the piece
2. A part line gives the part's pivot. A part placed more than once lists its
   steps under its first place, and each later place reads `repeats`
3. `exposed` counts an opaque cell beside a material that lets light through
4. `review` tiles the hero, front, right, back, and top views into one sheet PNG
5. A part's `rotation` poses its node, and `--rotate` turns a part for one
   render
6. The library retuned `oak`, `walnut`, `rope`, and `grass`. `shades` defaults
   to a spread of 0.05, and a lighter shade that sRGB cannot hold at the base's
   saturation errors with the largest spread that fits
7. The done line dropped round 6's fresh-eyes reviewer and went back to round
   5's text

The harness's gallery crops the sheet back into its views.

## Cost and passes

The checklist compares round 7 with round 5 because round 6 ran the dropped
review.

| Round | Good | Fair | Passes | Median passes | Failed | Median minutes | Cost    |
| ----- | ---- | ---- | ------ | ------------- | ------ | -------------- | ------- |
| 5     | 52   | 8    | 212    | 3             | 14     | 3.3            | $62.99  |
| 6     | 50   | 10   | 387    | 6             | 13     | 6.3            | $106.00 |
| 7     | 49   | 11   | 205    | 3             | 13     | 3.4            | $65.83  |

Round 7 matches round 5's cost and passes. The owner judged round 7's models
worse than round 6's on most prompts. Three tests on the chair, chest,
candelabra, ramen stand, and pirate ship looked for the cause, each with two
fresh runs per prompt or a rerender of round 7's models:

1. **Separate views.** A session reads the sheet at about 430 px per view,
   against 1,024 px for a separate PNG. Ten sessions ran with the review render
   writing five separate PNGs and otherwise as round 7. The owner judged them
   worse, and the sheet stays
2. **Old colors.** Round 7's final models rebuilt with the vxl from before S18
   took a warmer oak and stronger grain and changed little else. The owner keeps
   the new colors as more accurate
3. **The reviewer.** Ten sessions ran with round 6's done line. They took 63
   passes against round 7's 35 on the same prompts and brought back much of
   round 6's look, at $16.76 against $10.08. The owner ruled the reviewer not
   worth its cost

`~/voxel-trials/rounds/2026-10-18-sheet-test` and
`rounds/2026-10-18-reviewer-test` hold the two session tests.

The owner also ruled that a smaller voxel size makes no better art. Round 7's
analyses credited runs that refined their grid, and the triage set those credits
aside. Finding 9, finer grids read better than the guidance, falls with that
ruling.

## Findings

Round 7's analyses recorded 447 items across the 60 runs, against round 6's 551.
Every finding keeps its round 6 number, and new findings start at 126. The
round's gallery lists each finding's items beside the renders.

### What the changes did

The table counts the prompts that showed each finding the changes answered.

| #   | Finding                                           | Round 6 | Round 7 |
| --- | ------------------------------------------------- | ------- | ------- |
| 4   | Ended with flaws the renders show                 | 27      | 22      |
| 10  | Features near a voxel thick missed cells or broke | 10      | 8       |
| 24  | Floating crumbs                                   | 10      | 7       |
| 19  | The hero view hid the focal element               | 8       | 5       |
| 34  | Paint and coat spill onto neighbors               | 6       | 4       |
| 15  | `shades` left the sRGB gamut                      | 2       | 2       |
| 65  | Noise read as camouflage                          | 9       | 1       |
| 7   | Later steps buried earlier details                | 3       | 1       |
| 17  | Views the review four cannot give                 | 2       | 1       |
| 35  | Posed parts, plumb joints, and ropes across parts | 2       | 1       |
| 8   | Renders skipped                                   | 0       | 1       |
| 23  | Scene reports too long to read                    | 2       | 0       |
| 31  | Library rope and grass read off                   | 1       | 0       |
| 33  | Report lines without a location                   | 1       | 0       |
| 39  | Inspecting nodes and palette values               | 1       | 0       |
| 42  | Part pivots in the report                         | 1       | 0       |
| 68  | Lightened gold shades read beige                  | 1       | 0       |
| 69  | Oak read orange and walnut read pink              | 1       | 0       |
| 122 | The report hid what a carve cut                   | 1       | 0       |
| 51  | Exposure inside transmissive volumes              | 0       | 0       |
| 55  | 0 kept flags intended repaints                    | 0       | 0       |

The changes landed this way:

1. **The report.** Sessions read the new fields and acted on most of them. Piece
   counts caught well run 2's handle in 9 pieces, cart run 2's floating axle,
   both guitars' broken headstock strings, and candelabra run 1's flames lifted
   off their wicks. Piece lines with their step lists located the crumbs in the
   tree, bridge, wizard-tower, valley, village, and city runs. `lost to` fixed
   cottage run 1's door, well run 2's purlins, bridge run 2's buried riverbed,
   and log-cabin run 1's door handle. Log-cabin run 2 read its chinking at
   `0 kept` in every pass and still claimed visible chinking. No session cited
   `over`. The pivots let cart run 2 find its doubled wheel pivots, and
   `repeats` kept the cart, chess, and dungeon-kit reports short. Crumbs still
   took hand cleanup in 7 prompts until S20's `prune` lands
2. **The sheet.** Every session read the sheet. Its name tripped sessions in 4
   prompts because a pass that adds a view writes `<stem>-review.png` and leaves
   the earlier `<stem>.png` beside it (finding 126). The sheet shows no left
   side and spends a tile on blank back walls (finding 127). A few late passes
   rendered the sheet and read only a close-up
3. **Rotation.** Both robots and both cranes posed their joints with `--rotate`.
   The chest lids, dungeon tiles, village cottages, and crane joints took a
   part's `rotation`. The report still prints a rotated place unrotated
   (finding 128)
4. **Colors and `shades`.** Oak read tan, walnut brown, rope tan, and grass a
   natural green. The retuned oak now sits close to `pine`, and both carts' pine
   slats and oak stakes merged (finding 13). Five sessions hit the lighter-shade
   error, and each recovered with one edit to the spread the error gave. Noise
   read as camouflage in 1 prompt against 9, on ramen-stand run 2's dark roof at
   the default spread. Iron still read near-black and gold mustard while the
   metals wait for the rendering follow-ups' reflections (findings 5 and 20)
5. **The dropped reviewer.** Flaws the renders show shipped in 22 prompts
   against round 6's 27. Most sessions stopped after 2 or 3 passes. Sessions
   shipped flaws they had named, such as dragon run 1's green head on its green
   tail and both chess sets' slab knights, and flaws they passed over, such as
   ramen-stand run 2's sign borders missing whole sides

### Still open

Nothing changed for these findings:

| #   | Finding                                          | Round 6 | Round 7 |
| --- | ------------------------------------------------ | ------- | ------- |
| 2   | Large surfaces left flat                         | 23      | 24      |
| 13  | Neighboring materials too close in hue           | 11      | 9       |
| 5   | Dark materials collapse to near-black            | 7       | 6       |
| 12  | Shapes at an angle alias                         | 9       | 5       |
| 14  | Glass, water, ice, and gems read wrong           | 5       | 5       |
| 16  | Signatures misread                               | 3       | 5       |
| 3   | Documented tools left unused                     | 2       | 4       |
| 123 | Openings showed the white background             | 2       | 4       |
| 6   | Small round shapes read as blocks or plus signs  | 6       | 3       |
| 81  | Sloped snow drew stair-step contour lines        | 6       | 3       |
| 20  | Library metals read off hue                      | 2       | 3       |
| 18  | Emissives wash out                               | 1       | 3       |
| 67  | Large scenes went without close-ups              | 1       | 2       |
| 118 | Gradients over a few stops banded                | 1       | 2       |
| 103 | A twist on a thin stem did not read              | 0       | 2       |
| 44  | Placing on the cell grid took hand work          | 4       | 1       |
| 102 | Steam has no volume material                     | 3       | 1       |
| 32  | Grain read wrong                                 | 2       | 1       |
| 75  | Flame emissive dropped below the bloom threshold | 2       | 1       |
| 80  | Neither pirate ship curved its hull              | 2       | 1       |
| 117 | Profile-built knights read as slabs              | 1       | 1       |
| 124 | `set` rejected repeated points                   | 1       | 1       |
| 30  | Proportion and silhouette drift                  | 0       | 1       |
| 90  | Surface fill ruled out clear water               | 0       | 1       |
| 94  | Ceilings rendered near-black                     | 0       | 1       |
| 107 | Thatch read as planks                            | 0       | 1       |

Openings that showed the white background rose from 2 prompts to 4, and
signatures that misread from 3 to 5. Shapes at an angle that alias fell from 9
to 5, and sloped snow's contour lines from 6 to 3. Findings 1, 21, 27, 28, 36,
40, 45, 52, 54, 56, 57, 66, 70, 71, 72, 73, 74, 76, 77, 78, 79, 82, 83, 84, 85,
86, 88, 89, 93, 95, 96, 97, 98, 99, 100, 101, 104, 105, 106, 108, 109, 110, 111,
112, 113, 114, 115, 116, 119, 120, 121, and 125 showed in no prompt.

The phase 2 candidates that round 7 showed:

| #   | Finding                                         | Round 6 | Round 7 |
| --- | ----------------------------------------------- | ------- | ------- |
| 11  | Seating details on a surface                    | 7       | 7       |
| 25  | Sweeps, helices, and spirals                    | 6       | 4       |
| 46  | Emissives light nothing nearby                  | 4       | 2       |
| 22  | Rotation on part placement and a one-sided flip | 3       | 2       |
| 87  | Curves came from hand-sampled points            | 3       | 2       |
| 26  | A seeded random helper                          | 2       | 2       |
| 38  | Intended separate pieces read as floating       | 2       | 1       |
| 91  | Market stalls repeated one layout               | 1       | 1       |
| 49  | A section view of hollow forms                  | 0       | 1       |
| 50  | Hollowing a union                               | 0       | 1       |
| 53  | Parts that cut through each other               | 0       | 1       |
| 47  | Text                                            | 1       | 0       |
| 59  | Patterns that line up across tiles              | 1       | 0       |
| 92  | No pattern lays brick courses                   | 1       | 0       |

PHASE2_Findings 1, 21, 27, 28, 36, 40, 45, 52, 54, 56, 57, 66, 70, 71, 72, 73,
74, 76, 77, 78, 79, 82, 83, 84, 85, 86, 88, 89, 93, 95, 96, 97, 98, 99, 100,
101, 104, 105, 106, 108, 109, 110, 111, 112, 113, 114, 115, 116, 119, 120, 121,
and 125 showed in no prompt.

### New in round 7

126. **Review sheets changed names between passes.** In 4 prompts, a pass that
     added a view to the review command wrote `<stem>-review.png` and left the
     earlier `<stem>.png` beside it. Sword run 1 had to tell the user to delete
     its stale sheet, and ramen-stand run 1 shipped one. Guitar run 1 judged
     pass 3 on pass 2's sheet. Chair run 2 rendered its close-up apart and then
     looked for a `chair-review.png` that never existed
127. **The sheet's fixed views missed sides and wasted the back.** In 3 prompts,
     the sheet showed no left side and spent a tile on a blank back. Ramen-stand
     run 1's poster on the -x wall never showed in any render the session read.
     Both living rooms and spaceship run 1 got the outside of a back wall
128. **Rotated parts reported unrotated bounds.** In 2 prompts, the report
     printed a rotated place in its unrotated frame. Dungeon-kit run 2's model
     line read x -4..4 against a true -3.4..3.4. Neither crane's report gave the
     tilted boom's world extent or pivot, and run 2 guessed its close-up targets
129. **`fit` failed beside `--view-look-at`.** In 2 prompts, wizard-tower run 2
     and valley run 1 passed `fit` with `--view-look-at` and hit the error that
     asks for meters. Wizard-tower run 2 lost its pass and never redrew its
     sheet after the fix
130. **No step drew a connected line of cells.** In 1 prompt, both guitars'
     diagonal one-voxel strings broke into 9 and 11 pieces. Run 1 hand-walked
     cell centers into `set`, and run 2 unioned one box per grid row
131. **Painted diagonal lines counted as dozens of pieces.** In 1 prompt, guitar
     rosette rings one voxel wide read 28 to 42 pieces and run 2's binding 252
     because diagonal steps touch only at edges. The rings read whole in the
     renders, and run 1 spent a pass widening them
132. **No step spanned a membrane between rods.** In 1 prompt, dragon run 1
     replaced a shell wing that left its fingers floating with about 50 strand
     rods. The rods render as stripes rather than a membrane
133. **Palette values printed without material names.** In 1 prompt,
     `vxl palette show --property '*' walkable` printed a bare row of values.
     Both dungeon-kit runs printed the `baseColor` row too and matched the
     columns by hand
134. **Stone patterns striped on a curved wall.** In 1 prompt, wizard-tower run
     1's round wall under a `cells` pattern rendered as dense vertical stripes
     that hide the stones. The session called it a voxel artifact and shipped it
135. **A `+y` coat whitened whole sloped crowns.** In 1 prompt, log-cabin run
     2's snow `coat` on `+y` whitened its whole pine crowns because every step
     of a cone has a `+y` face. The session swapped the coat for a speckled
     noise
136. **`gradient` rejected a high-to-low range.** In 1 prompt, dollhouse run 2's
     first build failed with `gradient to must be past from 2.5, not 1.9`. The
     session reversed its stops by hand for a painting's sky-to-ground fade
137. **Hammocks sagged into bowls.** In 1 prompt, both pirate ships built
     hammocks as one-cell ellipsoid shells that sag into deep bowls. They read
     as baskets or boats rather than slung canvas

### No action

In 19 prompts the two runs chose near-identical plans, and in 28 workarounds
used the tools as intended. In 26 prompts sessions made one-off slips. In 19 the
report and the views caught real defects.

## Log

Round 7 ran on 2026-10-18 with vxl 0.6.0 built from `c572ba00` with the
gallery's sheet change staged, and Claude Code 2.1.302 running Claude Opus 5.5
at high effort. Every run loaded the skill and called its model done. Of the 60
runs, 49 read good and 11 fair, against round 6's 50 and 10. Different agents
judged each round, so the verdicts compare loosely. The runs took 205 passes, of
which 13 failed, and 1,031 turns against round 6's 1,492. A run took a median of
3.4 minutes against round 6's 6.3, and all 60 cost $65.83.
`~/voxel-trials/rounds/2026-10-18-round-7` holds the round, and its gallery
shows every run's renders.

### 1. Chair

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | fair    | 4      | 0      | 1.9     | 12,530 at 1.25 cm |
| 2   | good    | 3      | 0      | 1.6     | 201,020 at 5 mm   |

Both runs built an oak chair with turned legs, stretchers, and an arched crest
set with a ruby, sapphires, emeralds, and diamonds. Run 1 gave its slat back a
gilt crest with a gold border and wide bezels, where run 2 gave its vase splat
an oak crest with thin gold filigree, torus bezels, and amethyst-topped gilt
finials. Run 2 reads better because its gems read as cut stones set in visible
oak, where run 1's plus-shaped gems sit on a nearly all-gold plaque. Unlike
round 6, where run 1's library gems read dark against the gold, both runs'
library gems read bright.

Failures:

1. Run 1, pass 3: switching the border from `shell` to `subtract` with `offset`
   left the model unchanged at 292 border cells
2. Run 2, pass 2: the front stretcher stopped short of the legs as a second
   piece until pass 3 lengthened it to `0.2`
3. Run 1, final pass: every gem reads as a stacked plus sign rather than a cut
   stone. The gilt coat, gold border, and wide bezels turn the crest almost
   entirely flat gold
4. Run 1, final pass: the front-leg turnings sample into offset blocks that look
   broken in the front and side views
5. Run 2, final pass: the front legs' foot turnings show small dark notches. The
   gems barely stand proud in the side view

Lacked:

None

Missed in the skill:

1. Run 1 kept emeralds about 2 voxels across and sapphires under 3 although
   SKILL.md says an octahedron under about 3 voxels reads as a plus sign. Every
   gem shipped as a plus sign
2. Run 1 laid a `+y` gilt coat over a full gold border and wide bezels although
   SKILL.md says a coat recolors every live cell in its reach. Almost no oak
   shows on the crest face

Colors: Run 1's crest face is a large field of flat yellow gold that swamps the
oak. Its diamonds read as dull translucent gray cubes.

### 2. Lantern

| Run | Verdict | Passes | Failed | Minutes | Voxels         |
| --- | ------- | ------ | ------ | ------- | -------------- |
| 1   | fair    | 4      | 0      | 2.6     | 18,206 at 5 mm |
| 2   | fair    | 3      | 0      | 1.6     | 22,376 at 5 mm |

Both runs built a square iron lantern with a stepped roof, a chimney, and a
handle over a dripping candle on a dish. Run 1 added brass corner rivets, a
noise-shaded roof, a vented chimney, and a lit melt pool, where run 2 moved its
candle 1.5 cm off center to clear the hero's front post. Run 1 reads better
because its larger teardrop flame reads as a glowing candle, where run 2's small
flame reads as three stacked color blocks. Unlike round 6, where run 1's octagon
cleared the candle, run 1's front corner post again hides the flame in the hero.

Failures:

1. Both runs, pass 1: tinted glass washed the interior orange behind a
   mid-height rail that cut a small flame. Later passes cleared the glass and
   moved the rail
2. Run 1, pass 2: the session raised the flame's `emissiveStrength` to 3.5 and
   painted an emissive lit-glass sphere over the panes. Pass 3 reverted both
3. Run 1, final pass: the front corner post sits squarely over the flame in the
   hero. The session rendered an extra view from azimuth 18 instead of moving
   the candle
4. Run 1, final pass: the flame reads as a pear with a hard orange band at its
   base and a one-voxel red cap on top
5. Run 2, final pass: the flame stays small beside the wide candle and reads as
   near-white, orange, and red blocks stacked like a traffic light. The
   off-center candle looks misplaced in the front view

Lacked:

1. Both runs wanted a light source that casts the flame's glow onto the wax,
   dish, and cage interior. Run 1 tried an emissive paint on the glass before
   settling for an emissive flame and a faintly emissive candle top. Run 2 used
   emissive flame materials alone

Missed in the skill:

1. Run 1 kept the candle centered behind the front corner post although SKILL.md
   says the prompt's detail faces the hero corner with nothing in between. The
   hero shows no flame
2. Run 1 pushed the flame strengths up to 3.5 although SKILL.md says a pale glow
   takes a lower `emissiveStrength`. Pass 3 had to bring them back down
3. Both runs started with tinted glass although SKILL.md warns that tinted glass
   tints what sits behind it. Pass 2 went to clearing it
4. Run 2 left the roof, plinth, and posts one flat iron tone although SKILL.md
   says a large face of one flat color reads unfinished
5. Run 2 split its flame gradient into three equal hard spans. The gradient's
   warp option would have softened them into a flame

Colors: Both runs' panes read as one flat daylight beige in the orthographic
views. Neither shows warmth around the candle. Run 1's plinth and foot are a
near-black slab with no variation. Run 2's roof, plinth, and posts are one flat
dark iron. Its flame bands are hard-edged.

### 3. Chest

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | good    | 4      | 0      | 2.5     | 107,346 at 1 cm   |
| 2   | good    | 3      | 1      | 2.9     | 47,634 at 1.25 cm |

Both runs built an iron-bound oak chest with a barrel lid hinged open at the
back over a gold heap with gems, with coins spilled on the ground. Run 1 adds
rimmed loose coins, a coin stack, and a brass lock plate and hasp, where run 2
lines its lid in red velvet and tops its heap with a separate ruby, sapphire,
and emerald. Run 2 reads better because its red lining and plank seams set off
the gold, where run 1's heap reads as lumpy gold rubble.

Failures:

1. Run 1, pass 2: the mound did not fill the hollow and left two one-voxel stray
   pieces inside the body until pass 3 unioned a box under the dome
2. Run 1, pass 3: the coin-face paint ran over the rubies and emeralds until
   pass 4 moved the gem steps after it
3. Run 2, pass 2: the build failed with
   `shades shade 4 must be at the base's chroma ... This base takes a spread of at most 0.057 at count 5`.
   Pass 3 lowered the spread to `0.055`
4. Run 1, final pass: the top coins sit at heights from the undisplaced mound.
   They break into 19 pieces with a third of their cells lost, and several sink
   into the heap
5. Run 1, final pass: the heap reads as lumpy gold rubble with bright square
   faces rather than distinct coins. The 3 cm gems are small crosses. The barrel
   lid's inner faces show heavy stair-stepped bands
6. Run 2, final pass: the heap coins are plus-shaped blobs. The lid straps cut
   gray stripes across the red lining. The six-coin stack reads as a small
   yellow post

Lacked:

None

Missed in the skill:

1. Run 1 placed its top coins and gems at guessed heights over a displaced mound
   although SKILL.md says gems on a pile take `coat` with `sides` to find the
   surface. The coins broke apart and sank into the heap

Colors: Run 1's iron straps and trim are a flat near-black with no metallic
sheen. Its heap's two-shade gold noise reads monotone in the hero. Run 2's
`mat.iron` renders as flat mid-gray that reads as stone. Its heap top shows
blotchy flat yellow patches in the top view.

### 4. Sword

| Run | Verdict | Passes | Failed | Minutes | Voxels        |
| --- | ------- | ------ | ------ | ------- | ------------- |
| 1   | good    | 2      | 0      | 1.3     | 6,852 at 5 mm |
| 2   | good    | 4      | 1      | 1.4     | 7,228 at 5 mm |

Both runs built a point-up longsword with a steel blade carrying a painted edge
and a dark fuller, a banded gold grip, and a gold disk pommel with a ruby on
each face. Run 1 stopped with a small square ruby and lumpy ball tips on the
guard, where run 2 grew its pommel to 45 mm around a large domed ruby in a
lighter gold bezel. Run 2 reads better because its pommel gem leads the hilt as
the prompt asks. Unlike round 6, where run 2's beveled blade stepped evenly to
its point, both blades are flat slabs with a stair-stepped taper.

Failures:

1. Run 1, pass 1: the 2,400-degree twisted bar grip read as plain gold until
   pass 2 replaced it with two-tone bands
2. Run 2, pass 1: the build failed with
   `shades shade 2 must be at the base's chroma, not 94% of it ... This base takes a spread of at most 0.114 at count 3`.
   Pass 2 dropped the spread to `0.11`
3. Run 2, pass 2: the 23 mm deep gem did not reach past the pommel's back face
   and showed nowhere in the right view until pass 3 enlarged it
4. Both runs, final pass: the blade is a flat slab 2 voxels thick with stair
   steps along the taper. It shrinks to a hairline in the right view
5. Run 1, final pass: the ruby is a 6 by 6 near-square that reads as a red tile.
   The guard's ball tips come out as clumpy plus-shaped knobs
6. Run 2, final pass: up close the ruby is a lumpy speckled blob rather than a
   faceted stone. The guard's ball ends read square

Lacked:

None

Missed in the skill:

1. Both runs made the ruby an ellipsoid although the skill's recipe sets a jewel
   with an `octahedron` for a cut-gem silhouette. Run 1's sampled into a flat
   red square. Run 2's reads as a lumpy dome
2. Run 1 kept 2.8-voxel spheres on the guard tips although SKILL.md says a
   sphere under about 3 voxels reads as a plus sign. They came out as lumpy
   knobs

Colors: Both hilts use plain `mat.gold` outside the grip and read as one flat
mustard tone with no shading or sheen. In the orthographic views the gold reads
ochre rather than bright metal.

### 5. Tree

| Run | Verdict | Passes | Failed | Minutes | Voxels           |
| --- | ------- | ------ | ------ | ------- | ---------------- |
| 1   | good    | 3      | 0      | 1.9     | 223,125 at 0.1 m |
| 2   | good    | 3      | 0      | 2.1     | 312,145 at 0.1 m |

Both runs built a broad oak with a stout flared trunk and limbs under a wide
displaced crown. Run 1 fans six straight cone limbs from one point under a
flat-bottomed crown, where run 2 sends five heavy elbowed limbs out low and wide
under five greens with a sunlit top. Run 2 reads better because its limbs and
lowered hero show an oak's structure, where run 1's hero looks down on a crown
that hides most of the trunk. Unlike round 6, where black one-voxel pits dotted
both crowns in the top view, neither top view shows them.

Failures:

1. Both runs, pass 1: the crown's `displace` split off floating leaf crumbs, 39
   in run 1 and 72 in run 2. Both carved the last ones with boxes copied from
   the report's piece bounds
2. Run 2, pass 1: the trunk stuck out below the mound as a dark stub until pass
   2 clipped the wood at the mound
3. Run 1, pass 2: two twig tips poked through the crown top while three crumbs
   still floated until pass 3
4. Run 2, pass 2: an eight-seed sweep of the crown noise found no seed with
   fewer than three stray clumps
5. Run 1, final pass: the straight limbs radiating from one point read as
   umbrella spokes. Brown limb voxels still show through the crown in the top
   view
6. Run 1, final pass: the moss coat turns the back of the root flare into a
   solid green cone that reads as a grass skirt
7. Run 2, final pass: the central sphere leaves a domed knob on the crown top
   that gives a mushroom-cap silhouette in the front and back views. The
   ellipsoid mound shows concentric terrace rings in the hero

Lacked:

1. Both runs wanted a step that drops small floating pieces after displacement.
   Run 1 carved three boxes at the report's bounds. Run 2 swept seeds first and
   then carved three boxes

Missed in the skill:

1. Both runs built limbs from straight `roundCone` segments although SKILL.md
   points limbs at `bend`. Run 1's read as stiff spokes. Run 2's bend only at a
   hard elbow

Colors: Run 1's crown reads as one mid green with a lighter top because its `-y`
shade coat barely registers. Its moss coat paints the back of the root flare a
flat green. Run 2's mound is a flat mid green whose terrace steps show as rings.

### 6. Well

| Run | Verdict | Passes | Failed | Minutes | Voxels          |
| --- | ------- | ------ | ------ | ------- | --------------- |
| 1   | good    | 5      | 0      | 2.1     | 157,552 at 2 cm |
| 2   | good    | 4      | 0      | 1.9     | 201,012 at 2 cm |

Both runs built a cobbled stone well at 2 cm with a cap ring, two oak posts, a
windlass with a rope coil and an iron crank, a walnut gable roof, and a hooped
bucket on a rope. Run 1's close-up shows a larger tapered bucket hanging over
dark water, but its gable reads as two thin parallel chevrons from the side. Run
2 turned its roof into a front-facing A-frame with a tie beam and king post, but
left its bucket small. Neither reads clearly better because each wins one key
element of the prompt.

Failures:

1. Run 1, pass 1: the moss coat with `sides` `["+y"]` inside a ring below the
   cap kept 0 cells because the wall's top faces sit under the cap. Pass 2
   replaced it with a paint
2. Run 1, pass 1: `mat.clay` as the stone border rendered bright orange mortar
   until pass 2 swapped in gray
3. Run 2, pass 1: the bucket handle torus with a `0.012` m tube broke into 9
   pieces and left the bucket a separate 608-voxel piece. Pass 2 thickened the
   tube to `0.022`
4. Run 2, pass 2: the new purlins kept only 740 of 3,552 cells while the posts
   poked through the roof. Passes 3 and 4 went to sinking them
5. Run 1, passes 4 and 5: the hoop paint broke into 32 and then 21 pieces. Iron
   flecks still show inside the bucket where the hoops paint through the thin
   wall
6. Both runs, final pass: the roof slope partly hides the bucket in the hero. In
   run 2 the front post also covers it
7. Run 1, final pass: the gable reads as two thin parallel chevrons with a gap
   between them from the side
8. Run 2, final pass: the bucket is small at about 13 voxels across. The cap's
   moss is nearly invisible. The iron crank is a thin L that reads as a bent
   wire

Lacked:

1. Run 1 had no report data flagging the vertical shading streaks on the curved
   wall. The session never addressed them

Missed in the skill:

1. Run 2 gave the bucket handle a tube 1.2 voxels in radius although SKILL.md
   says a torus tube needs about 2 voxels across. The handle broke and cost a
   pass

Colors: Both curved stone walls show vertical light and dark streaks. Run 1's
oak posts and tie beam read as near-flat tan. Run 2's cap moss is too sparse to
read.

### 7. Robot

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | good    | 2      | 0      | 1.5     | 39,692 at 1.25 cm |
| 2   | good    | 2      | 0      | 1.8     | 19,522 at 1 cm    |

Both runs built a boxy blue toy robot with the head and arms as parts pivoting
at the neck and shoulders. Run 2 reads better because its carved pincers read as
grippers from the front and its striped mouth grille, chest core, and finned
backpack give every view detail. Run 1's arms end in single orange stubs from
the front, its checker mouth merges into one dark smudge, and its back and top
are plain blue slabs. Unlike round 6, both runs rendered the head and arms
turned at their joints and saw them clear the torso.

Failures:

1. Run 2, pass 1: the `teeth` repeat of 1 cm boxes broke into stray cells. Pass
   2 replaced it with a solid grille and carved slots
2. Run 2, pass 1: the claw hands read solid from the front. Pass 2 rebuilt the
   arms with carved pincer hands
3. Run 1, final pass: the claw prongs split along z. From the front each arm
   ends in a single small orange stub
4. Run 1, final pass: the checker mouth's two near-black tones merge into one
   dark smudge
5. Run 1, final pass: the back and top are plain blue slabs with no detail
6. Run 2, final pass: the amber chest core glows pale peach
7. Run 2, final pass: the posed render turns the raised arm end-on to the
   camera. The pincers overlap the chest panel until the raise barely reads

Lacked:

None

Missed in the skill:

1. Run 1: the mouth checkers pair `joint` with `visor`, two dark tones the color
   checks say merge side by side. The grille barely reads
2. Run 1: the feet, hips, knee bands, and wrists use the flat `joint` material
   with no `shades` or `noise`. The skill flags such flat faces as unfinished
3. Run 1: pass 2 changed only the voxel size. The stub hands and featureless
   back survived although the done criterion asks a review to fix nameable flaws
4. Run 2: the chest core glows pale peach at `emissiveStrength` 1.5 on a light
   amber. The color checks say a pale glow takes a lower `emissiveStrength` or a
   darker `baseColor`

Colors: Run 1's dark joint gray on the feet, hips, and knee bands reads flat.
Its orange shoulder and ear accents are flat too. Run 2's large orange trim on
the shoulder pads and belt carries no shades. Its black chest and visor panels
read flat beside an amber core that washes out.

### 8. Cottage

| Run | Verdict | Passes | Failed | Minutes | Voxels          |
| --- | ------- | ------ | ------ | ------- | --------------- |
| 1   | good    | 2      | 0      | 2.6     | 211,432 at 5 cm |
| 2   | good    | 2      | 0      | 2.1     | 209,957 at 5 cm |

Both runs built a spotted red mushroom cottage at 5 cm with a cream stem, a
round plank door, round glowing windows, a chimney, and stepping stones. Run 1
reads better because its windows carry clean torus frames with one set into the
cap, and its cap rim stays clean. Run 2 has the taller stem and larger door, but
a tan sawtooth of gill paint frays its cap rim in every view and its clipped
window frames read as jagged half-rings. Run 1's torus frames come out clean
where round 6's run 2 shipped lumpy, ragged rings.

Failures:

1. Run 1, pass 1: the door frame extruded from a filled circle covered the door
   as a dark solid disk in every view. Pass 2 turned the frame into a ring
2. Run 2, pass 1: the `underside` coat took a `halfSpace` for its `within`
   instead of the cap and recolored the lawn, door, and windows. Pass 2 scoped
   it to the cap
3. Run 1, final pass: the door frame cuts the stem into 3 pieces in the report,
   leaving small islands around the frame
4. Run 2, final pass: the `gills` paint spills onto the cap's outer rim in 216
   pieces. A tan sawtooth rings the cap edge in every view
5. Run 1, final pass: the squat stem under a heavy cap makes the house body feel
   short. The lathed stem shows stepped contour bands that read as blotchy ovals
   in the orthographic views
6. Run 1, final pass: the single-cell flowers nearly vanish at sheet scale
7. Run 2, final pass: the window frames clipped to a wall slab render as chunky
   jagged half-rings. The flanking windows show as dark crescents from the front

Lacked:

None

Missed in the skill:

1. Run 2: SKILL.md says a paint reading more pieces than its forms spilled, yet
   `gills` read 216 pieces from 40 blades. The spill onto the rim shipped

Colors: Run 1's stem carries shades and noise, but its stepped contour bands
overpower the color variation. Run 2's brick chimney sits in nearly the cap's
red and merges with it.

### 9. Potion

| Run | Verdict | Passes | Failed | Minutes | Voxels           |
| --- | ------- | ------ | ------ | ------- | ---------------- |
| 1   | good    | 3      | 0      | 1.9     | 33,096 at 2.5 mm |
| 2   | good    | 2      | 0      | 1.4     | 16,116 at 3 mm   |

Both runs built a round-bellied corked flask with a blue lip ring, twine, and a
red potion shading darker toward the bottom. Run 2 reads better because its
liquid shades from deep crimson to a saturated red and a blue base rim outlines
the clear glass. Run 1's potion stays a muted rosy red over a ragged maroon band
that reads as sediment. Both runs cleared the glass, where round 6's run 2
shipped an empty flask that read as a solid sky-blue fill.

Failures:

1. Both runs, pass 1: blue-tinted glass over the potion muddied the red. Run 1
   read purple-gray and run 2 pinkish-violet until pass 2 cleared the glass
2. Run 1, final pass: the potion reads a muted rosy red. The warped gradient
   leaves a ragged dark-maroon lower band that reads as sediment rather than
   depth
3. Both runs, final pass: the cork cap fails to flare. Run 1's matches the lip
   ring's width and run 2's voxelizes to the plug's 6 cells
4. Run 2, final pass: the silhouette and liquid surface step visibly at 3 mm.
   The top view shows the liquid disc near-octagonal
5. Run 2, final pass: 8 bubbles sit inside the opaque potion with 2 exposed. One
   stray pixel shows in the front view

Lacked:

None

Missed in the skill:

1. Both runs: SKILL.md's checks flag a step with few exposed cells as buried,
   yet both runs placed bubbles inside a near-opaque potion. Run 1 spent pass 3
   removing them and run 2 shipped them hidden

Colors: Run 1's lower potion is a flat dark maroon behind a jagged warp
boundary. The glass also pulls its red toward salmon. Run 2's colors read true.

### 10. Cart

| Run | Verdict | Passes | Failed | Minutes | Voxels          |
| --- | ------- | ------ | ------ | ------- | --------------- |
| 1   | good    | 2      | 0      | 1.7     | 23,126 at 2 cm  |
| 2   | good    | 3      | 0      | 1.8     | 9,252 at 2.5 cm |

Both runs built a slatted plank farm cart on four spoked wheel parts with iron
tires and a tongue ending in a T handle. Run 1 reads better because its large
10-spoke wheels read crisp and its straight tongue stays clean. Run 2 has the
more detailed underframe, but its sloped tongue stair-steps along its whole
length and its 0.5 m wheels carry blobby spokes.

Failures:

1. Run 1, pass 1: the dark walnut rims and spokes ran together with the iron
   tires. Pass 2 swapped them to oak
2. Run 2, pass 1: the rear axle hung below the bolster as a second piece. Pass 2
   attached it
3. Run 2, pass 1: the tire broke into 16 pieces. Pass 2 thickened it
4. Run 2, pass 1: the session passed the pivot as an absolute position beside
   the offset and doubled every wheel pivot. Pass 2 removed it
5. Both runs, final pass: the iron tires stair-step into a cog-tooth edge in the
   hero
6. Run 1, final pass: the hitch ring is a plain box
7. Run 2, final pass: the sloped capsule tongue stair-steps along its whole
   length. The 0.5 m wheels come out with blobby spokes
8. Run 2, final pass: the wheels run only 2 voxels thick and look like thin
   plates from above

Lacked:

None

Missed in the skill:

1. Run 2: SKILL.md says `offset` moves the part's pivot too, yet pass 1 set the
   pivot equal to the offset. Fixing the doubled pivots cost a pass
2. Run 2: SKILL.md warns that a thin form turned off the axes samples into stair
   steps, yet the session kept the diagonal capsule tongue. It became the
   model's most visible flaw
3. Run 2: SKILL.md says a model faces `+Z`, yet the session ran the cart along x
   with the tongue toward `+x`. The review's front view shows the side and its
   right view the tongue end

Colors: Run 1's pine slats, oak stakes, and pine floor share one pale tan. In
the hero the bed reads as a single block with faint board bands. Run 2's pine
boards and oak stakes also merge into one tan. The white gaps between its board
rows read as missing boards rather than shadow.

### 11. Guitar

| Run | Verdict | Passes | Failed | Minutes | Voxels          |
| --- | ------- | ------ | ------ | ------- | --------------- |
| 1   | good    | 3      | 0      | 4.7     | 548,326 at 3 mm |
| 2   | good    | 2      | 0      | 4.3     | 132,819 at 3 mm |

Both runs built a steel-string acoustic at 3 mm with a ringed rosette, six
one-voxel bronze and steel strings running to six posts, frets, dot inlays, and
a tortoise pickguard. Run 2 reads better because its shelled polygon outline
gives a real dreadnought waist with black purfling around a sound hole cut into
a hollow body. Run 1's `smoothUnion` outline fills most of the waist and reads
as a pear or parlor shape, though its shaped tuner keys beat run 2's gray cubes.

Failures:

1. Both runs, pass 1: the diagonal headstock string runs broke apart, into 9
   pieces in run 1 and 11 in run 2. Pass 2 rebuilt them as face-connected cell
   steps
2. Run 1, passes 1 and 2: the one-voxel rosette rings read 30 and 42 pieces.
   Pass 3 widened them
3. Run 2, pass 1: the neck heel ellipsoid bulged above the body. Pass 2 reshaped
   it
4. Run 1, pass 3: raising the waist `smoothUnion` radius from `0.035` back to
   `0.05` undid part of the waist narrowing. The session's final message claimed
   the pear shape fixed
5. Run 1, final pass: the outline reads as a pear or parlor guitar in the front
   and back views, not a dreadnought
6. Run 2, final pass: the tuner buttons are plain gray cubes

Lacked:

1. Both runs: a face-connected 3D line of cells for diagonal one-voxel strings.
   Run 1 walked cell centers into `set` and run 2 unioned one box per grid row

Missed in the skill:

1. Run 1: the session left the body solid behind a painted 13-voxel pocket
   though SKILL.md documents `shell`. An extruded polygon outline, the hollow
   form SKILL.md names, would have shelled cleanly where the `smoothUnion`
   outline would not

Colors: Both runs' dark fretboard, headstock plate, and bridge read as one flat
near-black. In run 1's hero they merge with each other. Run 2's tuner buttons
and housings are flat gray boxes.

### 12. Candelabra

| Run | Verdict | Passes | Failed | Minutes | Voxels         |
| --- | ------- | ------ | ------ | ------- | -------------- |
| 1   | fair    | 5      | 0      | 3.1     | 23,200 at 4 mm |
| 2   | good    | 4      | 0      | 3.1     | 52,040 at 3 mm |

Both runs built a wrought-iron candelabra with a domed foot, a twisted square
stem, four curling arms, and five candles with the fifth standing on the stem.
Run 2 reads better because its stem twist reads as four clean burnished spirals
and its nested arm sweeps give the stronger front, though its flat plane of arms
shows as a thin column from the side. Run 1 spreads its arms in a cross, but its
stem highlight breaks into speckles and its back candle hides behind the center
one in the front and side views.

Failures:

1. Run 1, pass 3: lengthening the flames lifted four of them off their wicks
   into separate pieces. Pass 4 lowered them
2. Run 2, pass 1: the 8-cell stem at 1,000 degrees per meter barely showed a
   twist. Pass 2 widened it to 10 cells and raised the twist to 2,400 degrees
   per meter
3. Run 2, pass 1: the flames swallowed 36 of the 40 wick cells. Pass 2 raised
   the wicks a voxel
4. Run 2, pass 3: the burnished ridge paint split into 200 pieces. Pass 4
   tightened the cutter radius to leave 4 spirals
5. Both runs, final pass: four arms carry four candles while the fifth stands on
   the stem. The prompt asked for five candles on curling arms
6. Run 1, final pass: the ridge paint breaks into 76 pieces. The stem highlight
   speckles rather than winding in four spirals
7. Run 1, final pass: the plus-shaped layout hides the back candle behind the
   center one in the front and side views
8. Run 1, final pass: the 2-voxel-deep arm straps stair-step up close. The
   flames are tiny at about 5 voxels
9. Run 2, final pass: all the arms lie in one plane. The side view shows a thin
   column and the top view a straight line
10. Run 2, final pass: the candles and flames look blocky up close

Lacked:

None

Missed in the skill:

1. Run 1: the Checks section says a step reading more pieces than its forms
   broke on the grid, yet the session left the 76-piece ridge paint. The stem's
   twist reads noisy for it

Colors: Both runs' iron foot, stem, arms, and pans share one dark tone and merge
into a single mass. Run 1's ridge shade lifts only scattered cells. Run 2's
burnished spirals give its metal the only tonal relief.

### 13. Knight

| Run | Verdict | Passes | Failed | Minutes | Voxels           |
| --- | ------- | ------ | ------ | ------- | ---------------- |
| 1   | good    | 2      | 0      | 3.6     | 2,157 at 6.25 cm |
| 2   | good    | 2      | 0      | 3.4     | 2,551 at 6.25 cm |

Both runs built a knight exactly 32 voxels tall in exact PICO-8 hexes, with a
great helm, a red and pink crest, a red tabard with a yellow cross, a sword, and
a heater shield, and ended in one piece. Run 1 reads better because its blue
kite shield faces the front and its body, head, arms, and legs sit in parts with
pivots ready to animate. Run 2 has the better proportions, an upright sword, and
a pleated purple cape, but its shield faces `+x` and shows only edge-on from the
front. Unlike round 6, where both shields faced the front, run 2's shield turned
to the side again, while both runs kept the white `+y` coat that round 6 tried
and removed.

Failures:

1. Both runs, pass 1: the breathing holes on the helm read as a face in run 1
   and as a second visor in run 2. Pass 2 replaced them with a black T-visor in
   run 1 and dropped them in run 2
2. Run 2, final pass: the shield faces `+x`. The front view shows it as a
   stepped grey column under the left arm, and the right view shows a second red
   panel with a yellow cross that competes with the tabard
3. Run 1, final pass: the legs are only 7 of the 32 voxels. The figure reads
   squat
4. Run 1, final pass: the down-pointing sword hangs below the gauntlet like a
   dangling stick
5. Run 1, final pass: the back is a plain red slab under a plain grey helm
6. Both runs, final pass: the white `+y` coats turn the top view almost solid
   cream

Lacked:

None

Missed in the skill:

1. Both runs left the helm, greaves, and side faces one flat light grey although
   the skill offers a palette-only `checker` or `bands` over two PICO-8 greys.
   Run 2 used a checker only on the mail skirt
2. Run 2 built a game character as one flat step list although the skill
   documents `part` with pivots and shows an arm rig. The knight ships unrigged

Colors: Both runs' helm sides, greaves, and pauldron or torso sides stay one
flat `#C2C3C7`. White `+y` coats turn both top views nearly solid `#FFF1E8`. Run
1's back view is a flat red tabard slab.

### 14. Pocket watch

| Run | Verdict | Passes | Failed | Minutes | Voxels               |
| --- | ------- | ------ | ------ | ------- | -------------------- |
| 1   | good    | 2      | 0      | 4.2     | 1,383,484 at 0.25 mm |
| 2   | fair    | 2      | 0      | 6.0     | 214,095 at 0.4 mm    |

Both runs built a gold skeleton pocket watch with a cream chapter ring, colored
wheels, blued hands, and ruby jewels on a chain to a T-bar. Run 1 reads better
because its open movement shows copper, brass, and steel wheels with crisp teeth
and a blue-screwed balance. Run 2 stages the chain more naturally by standing
the watch on its rim with the chain lying on the ground, but broad silver
bridges cover much of its movement. Unlike round 6, where both watches hung from
their chains, run 1's chain arcs stiffly up from the bow to a T-bar in mid-air.

Failures:

1. Run 1, pass 1: crown slivers and hairspring fragments left seven pieces. Pass
   2 cleared them
2. Both runs, final pass: the interlocked links share cells. The whole chain
   reports as one piece with the watch, and each final message calls that one
   piece a success
3. Run 1, final pass: the chain arcs from the bow to a T-bar hanging in mid-air.
   The chain close-up shows merged links, and the session stopped without fixing
   them
4. Run 2, pass 2: turning the whole model 30 degrees about `y` skews the front,
   right, and top views. The case stair-steps

Lacked:

1. Both runs wanted the report to show contact between links. Run 1's single
   chain step gave only the total piece count, while run 2's only clue was
   `links a` reading `lost to links b`

Missed in the skill:

1. Run 2 rotated the whole model 30 degrees to face the hero camera although the
   skill warns that an object turned off the axes samples into ribs and stair
   steps

Colors: Both runs' `mat.gold` reads mustard-olive on the case back and the edge
views. Run 1's chain shares the same dull gold. Run 2's dark plate under silver
bridges flattens its wheels into one grey-gold mass.

### 15. Dragon

| Run | Verdict | Passes | Failed | Minutes | Voxels          |
| --- | ------- | ------ | ------ | ------- | --------------- |
| 1   | fair    | 6      | 1      | 6.7     | 117,751 at 3 cm |
| 2   | good    | 5      | 0      | 6.9     | 310,821 at 2 cm |

Both runs coiled a dragon on a gold heap from `roundCone` chains along a
hand-computed spine, with folded wings and straight horns. Run 2 reads better
because its red body stands off the gold, its tail wraps in front of its nose,
and its folded wings show red finger bones over a plum membrane in the hero and
top views. Run 1 reads as a coiled serpent because one green hides its head
against its tail and its wings came out as purple and green stripes along one
flank. Unlike round 6, where both runs ended in one piece and read as dragons in
the hero, run 1 ships 12 pieces and loses its head in the hero.

Failures:

1. Run 1, pass 1: the wing sheet cut as an offset shell of the torso left
   floating fingers above it. The session swapped it for about 50 strand rods
   that render as stripes
2. Run 1, pass 2: the build threw
   `TypeError: Cannot read properties of undefined (reading 'phi')` because the
   hand-rolled spine sampler indexed below 0. Pass 3 clamped the index
3. Run 2, first build: the build failed with
   `shades shade 4 must be at the base's chroma ... This base takes a spread of at most 0.057 at count 5`.
   The session lowered the spread to 0.055
4. Run 2, pass 1: `orient` rolled the head onto its side. Pass 2 replaced it
   with a pitch-then-yaw rotate
5. Run 1, final pass: 10 stray gold voxels from the heap's displacement float as
   separate pieces. The session called them loose coins
6. Both runs, final pass: the front, right, and back views flatten the dragon
   into a low hump with no raised wing or lifted head. Run 1's reads as a lumpy
   green log with ivory nubs
7. Run 1, final pass: the legs sit buried under the body
8. Run 2, final pass: the dragon fills a small part of a wide pile and floor.
   Its legs barely show, and its ivory spikes read as scattered white pellets

Lacked:

1. Both runs wanted a sweep along a curve for a tapered body. Each hand-rolled a
   spine sampler that chains `roundCone`s through sampled points

Missed in the skill:

1. Both runs built the horns as straight `roundCone`s although the skill routes
   horns and curled tips to `bend`. Run 1's horns staircase, while run 2's read
   as stakes
2. Run 1 saw the green head merging into the green tail and shipped it although
   the skill warns that neighboring materials need colors far apart

Colors: Run 1's head, body, legs, and tail share one green. Its head merges into
its tail in the hero, while its membrane strands alternate purple and dark green
in stripes. Run 2's plum membrane sits nearly black in shadow.

### 16. Bridge

| Run | Verdict | Passes | Failed | Minutes | Voxels              |
| --- | ------- | ------ | ------ | ------- | ------------------- |
| 1   | good    | 4      | 1      | 3.9     | 1,200,137 at 0.25 m |
| 2   | good    | 2      | 0      | 3.3     | 396,330 at 0.1 m    |

Both runs built a two-tower stone suspension bridge with parabolic cables of
chained capsules, looped one-voxel hangers, and a river scene. Run 1 reads
better because its towers stand on pointed piers in the river with two-tier
arched portals, and its cables run down side spans with hangers to stone
anchorages. Run 2 adds masonry detail and a layered gorge, but its short bridge
sits on the banks over a deep void. Unlike round 6, where run 2's midspan cable
touched the deck until pass 3 raised the sag, both runs now ship a cable that
sags onto the handrail at midspan.

Failures:

1. Run 1, pass 2: the build failed with
   `ReferenceError: Cannot access 'boulders' before initialization` because the
   inserted block landed after the tower function used it
2. Run 1, passes 1 to 3: the portal arches never cut because
   `extrude(..., { axis: "x" })` maps the profile to (y, z). Pass 4 turned them
   with `.rotate("y", 90)`
3. Run 1, pass 3, and run 2, pass 1: displaced tree crowns and boulders shed 37
   and 45 pieces. The next pass toned down the displacement
4. Run 2, pass 1: the riverbed read `0 kept` because grass and water took its
   cells. Pass 2 painted an offset bed
5. Both runs, final pass: the main cable sags onto the handrail at midspan. Run
   1 sags 17 m over a 36 m span, while run 2 hangs from 14 m towers over a 23 m
   span
6. Run 1, final pass: the 0.75 m cables read heavy and stair-stepped. They pass
   through the finial pyramids on the tower tops
7. Run 1, final pass: the straight trapezoid river reads as a canal
8. Run 2, final pass: two leaf voxels from `crown 23` float as separate pieces
9. Run 2, final pass: the deck spans about 27 m of a 48 m scene. The water sits
   3 m below it, and the front and back views show a large white void under the
   deck
10. Run 2, final pass: a dark tangled grass clump hangs on the left bank's
    channel wall in the hero

Lacked:

1. Both runs wanted a sweep or tube along a 3D point list. Run 1 chained
   capsules between parabola points 0.5 m apart, while run 2 chained them
   through 46 main-span and 16 backstay points

Missed in the skill:

1. Run 1 misread the skill's note that `axis: "x"` maps a profile to (y, z). The
   portals stayed uncut for three passes
2. Run 2 shipped two floating leaf voxels although the skill calls a model done
   only when every extra piece is intended
3. Neither run rendered a midspan close-up that would have caught the cable on
   the handrail. Run 2 also missed the grass clump on the channel wall

Colors: Run 1's river reads as one flat teal slab in the top view. The grey
stone band under the dirt on its terrain sides is flat. Run 2's tower and anchor
caps are one flat pale cream.

### 17. Chess set

| Run | Verdict | Passes | Failed | Minutes | Voxels          |
| --- | ------- | ------ | ------ | ------- | --------------- |
| 1   | good    | 4      | 0      | 3.1     | 568,280 at 2 mm |
| 2   | good    | 2      | 0      | 3.3     | 329,470 at 2 mm |

Both runs lathed five piece kinds, built an extruded knight, and placed all 32
pieces correctly on a maple and rosewood board with a1 dark and each queen on
her color. Run 2 reads better because its boxwood and blackwood pieces carry
faint grain, its dark pieces keep their form, and its knight has ears, eye and
nostril pits, and a banded mane. Run 1's near-black pieces read as silhouettes.
Unlike round 6, where run 2 sculpted a knight that read as a horse, both knights
stay constant-width extrudes that read as slabs from the front.

Failures:

1. Run 1, pass 1: the bishop's slit carve drew 0 cells because its box sat at
   board height instead of at the mitre. Pass 2 moved it
2. Both runs, final pass: the knights read as thin slabs from the front and
   back. Run 1's mane reads as a tan stripe
3. Run 2, final pass: the grain bands on the stair-stepped lathes give the
   pieces a ribbed, noisy look up close

Lacked:

None

Missed in the skill:

1. Both runs kept the knight head one constant-width extrude. A second extrude
   intersected along the facing axis or a `smoothUnion` of ellipsoids would have
   tapered the muzzle. Run 1 named the fix without making it

Colors: Run 1's black pieces are one flat near-black `#1C1A1A` that hides their
turnings and crowns. Its ivory pieces are unshaded, while its white knights' tan
mane reads as a stripe.

### 18. Ramen stand

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | good    | 3      | 0      | 4.3     | 225,273 at 2.5 cm |
| 2   | fair    | 4      | 1      | 3.1     | 221,609 at 2.5 cm |

Both runs built a stall under a pink RAMEN sign in a cyan neon border, with a
katakana blade sign on the `+x` wall, noren, lanterns, and bowls on the counter,
and both signs read the right way round. Run 1 reads better because its sign
frames are complete and its three red bowls show egg, nori, naruto, and
chopsticks. Run 2 drew cleaner, well-spaced letters, but its sign borders lost
whole sides and its two tiny white bowls barely read. Unlike round 6, where run
1's strength-2 neon paled to bubblegum, run 1 kept every glow at strength 1 on
dark bases and stayed saturated, while run 2 pushed its pink to 2 and paled it.

Failures:

1. Run 1, pass 1: the `tiles` step read 0 cells because its 0.01 m slab missed
   every cell center. Pass 2 thickened it to a full voxel
2. Run 2, pass 1: the build failed with
   `error: ramen-stand/puddle: box round must be at most half the shortest side, 0.0125, not 0.02`.
   Pass 2 dropped the round
3. Run 2, pass 3: the two bowls floated as separate pieces. Pass 4 widened the
   bowl feet to join them
4. Run 2, final pass: both sign borders use `.shell(0.0125)`, half a voxel
   thick. The RAMEN border lost its bottom edge, and the side sign's border
   shows only its left and top edges
5. Run 1, final pass: the RAMEN letters touch because the 0.3 m pitch leaves no
   gap past the 0.05 m strokes. R runs into A and M into E
6. Run 1, final pass: the steam over the pots reads as clear glass blobs
7. Run 1, final pass: the neon poster on the left wall shows in no view
8. Run 2, final pass: the noodles and egg in the two tiny white bowls barely
   read

Lacked:

1. Run 2 wanted the report to flag a step that lost whole edges. The
   `sign border` step's single piece and full bounds hid two sides with no cells

Missed in the skill:

1. Run 2 built half-voxel borders although the skill says a stroke needs about 2
   voxels across. No pass caught the missing sides in the renders
2. Run 2 drove the pink letters to strength 2 although the skill says strengths
   of 2 to 4 pale the core. The pink renders washed bubblegum with a halo
3. Run 1 gave the steam `transmission: 0.7` although the skill says it reads
   faintly as glass. The session named the glass look and shipped it
4. Run 1 never rendered the left wall. A hero orbit from that side would have
   shown the poster

Colors: Both runs' sign backs are flat black slabs in the back view. Run 2's
roof noise reads as camouflage blotches in the top view, while its pink neon
pales toward bubblegum.

### 19. Dungeon kit

| Run | Verdict | Passes | Failed | Minutes | Voxels          |
| --- | ------- | ------ | ------ | ------- | --------------- |
| 1   | good    | 2      | 0      | 2.9     | 275,836 at 5 cm |
| 2   | good    | 2      | 0      | 2.7     | 400,944 at 5 cm |

Both runs built a four-tile kit on a 2 m grid with walkable floor materials and
imported it into a room that voxelizes as one piece. Run 1 reads better because
it has a true L-shaped corner tile, and its two arched doorways open onto
corridor stubs with courses and floor joints lining up across the seams. Its
corner pillar stands only 2 voxels proud and reads as a thin pale strip inside
the room. Run 2's blue-gray rubble against sandstone trim gives richer surfaces,
but its corner is a 0.7 m pillar rather than a tile. Its doorways open onto
nothing, and its rubble pattern breaks at every wall seam. Run 1's doorways lead
onto corridor stubs where round 6's run 1 room had a single doorway.

Failures:

1. Run 1, pass 1: `floorJoint.rotate("y", 90)` turned the floor's cross joints
   about the origin and off the tile. Pass 2 repeated a box along z instead

Lacked:

1. A rotated part's posed bounds in the report: both reports give a rotated
   place's bounds unrotated. Run 2's model line reads an x range of -4..4 where
   the room spans -3.4..3.4
2. Custom property values beside material names: both runs confirmed `walkable`
   with `vxl palette show`, which prints bare values. Each session printed the
   palette's colors to match them to materials

Missed in the skill:

None

Colors: Run 1's plinth and caps share one dark material with no shades. They
read as flat dark bands along the base and top of every wall.

### 20. Crane

| Run | Verdict | Passes | Failed | Minutes | Voxels          |
| --- | ------- | ------ | ------ | ------- | --------------- |
| 1   | fair    | 2      | 0      | 3.4     | 51,202 at 0.1 m |
| 2   | good    | 3      | 1      | 5.2     | 76,453 at 0.1 m |

Both runs built a crawler crane at 0.1 m as a part tree of root, cab, boom, and
hook with every pivot on its hinge and the hook counter-rotated to hang plumb.
Run 2 reads better because pendants run from its 24 m lattice boom to the
A-frame and two thin falls drop to a red striped hook block. Its 30 degree
default slew points the cab at the hero camera and foreshortens the boom. Run
1's 14 m boom reads clean, but it carries no pendants, and its twin falls merge
into one black post about 0.6 m wide. Run 1 dropped the boom pendants both round
6 runs rigged because no link could follow a render-time tilt.

Failures:

1. Run 1, pass 1: because the first posed render tilted the boom without
   counter-rotating the hook, the cable swung off plumb. Pass 2's render added
   `--rotate` on the hook
2. Run 2, pass 1: the build failed with
   `crane/cab/house: shades shade 2 must be at the base's chroma ... This base takes a spread of at most 0.038 at count 3`.
   Pass 2 set the spread to 0.035
3. Run 2, pass 2: the first hook and tip close-ups aimed at guessed world
   `--view-look-at` points and missed the slewed and tilted parts. Another
   render round followed

Lacked:

1. A link stretched between two parts: run 1 left out its pendants. Run 2's
   pendants fit only the default boom angle, and a render-time `--rotate` of the
   boom leaves their ends floating off the gantry
2. A part's posed world bounds and pivot in the report: the report gives each
   part's bounds in its unrotated frame. Run 1 quoted the level boom's 20 x 14 m
   extent, while run 2 guessed where the tilted hook and tip sat

Missed in the skill:

1. Run 1 left its deck and car body one flat gray although the skill's color
   checks flag large single-color faces. Both read as flat slabs
2. Run 1 never weighed its cable against the skill's warning that neighboring
   dark tones merge. The two falls fuse into one black column in every view
3. Run 2 hand-computed world look-at points for its first close-ups although the
   skill documents `--view-select` on a part. The misses cost a render round

Colors: Run 1's gray deck and car body read as flat slabs. Its twin black falls
and dark sheave merge into one black column.

### 21. Fish tank

| Run | Verdict | Passes | Failed | Minutes | Voxels          |
| --- | ------- | ------ | ------ | ------- | --------------- |
| 1   | fair    | 4      | 0      | 4.9     | 474,548 at 5 mm |
| 2   | good    | 6      | 0      | 6.9     | 65,931 at 5 mm  |

Both runs built a black-framed tank at 5 mm with sloped gravel, a red-roofed
castle, seaweed, bubbles, and several kinds of fish. Run 2 reads better because
its colors stay saturated, with striped clownfish, blue-and-red neon tetras, a
blue tang, and a four-tower castle before a gradient blue backdrop. Its flag
crosses the water line, and its air tube rises above the rim. Run 1's solid
water washes the whole interior pale cyan. Its fish read pastel and its tetras
read as smudges. Run 1's solid water brings back the cyan wash that round 6's
one-voxel water sheets avoided.

Failures:

1. Run 1, pass 1: seaweed crossed the glass. Pass 2 intersected it with the
   inner tank box
2. Run 1, pass 2: the clown and tang eyes read `0 cells` after the eye intersect
   shrank to the bare body. Pass 3 fixed them
3. Run 1, pass 3, and run 2, pass 5: transmissive bubbles read 2 or 3 exposed
   cells and barely rendered. The next pass made them opaque
4. Run 2, pass 2: solid water washed everything cyan and turned the clownfish
   orange brown. Pass 3 switched to `--fill-mode surface`
5. Run 2, final pass: the flag spans the 0.32 m water line and pokes through the
   surface

Lacked:

1. Why a step dropped out: run 2's transmissive bubbles vanished under surface
   fill with no word from the report. The session read the cause from the
   near-zero `exposed` count and the render

Missed in the skill:

1. Run 1 never tried `--fill-mode surface` although `SKILL.md` says it keeps the
   opaque cells seen through water. The model ships under a cyan wash

Colors: Run 1's interior takes a pale cyan cast. Its gravel reads gray-blue and
the tetras lose their colors. Run 2's back view shows only the flat blue
gradient of the opaque backdrop. Its side glass reads a pale milky cyan in the
right view.

### 22. Wizard tower

| Run | Verdict | Passes | Failed | Minutes | Voxels           |
| --- | ------- | ------ | ------ | ------- | ---------------- |
| 1   | good    | 3      | 0      | 3.4     | 345,884 at 0.1 m |
| 2   | good    | 4      | 1      | 3.7     | 249,758 at 0.1 m |

Both runs built a tapered cell-stone tower at 0.1 m with a stair of looped
treads spiraling two or more turns and a cone roof curled by `bend`. Run 2 reads
better because its oak sector treads read as distinct steps in every view, and
its indigo hat carries a brim, a gold band, and a gold star on a hard-curling
tip. The right view sees its bend end-on and shows a straight cone. Run 1's 55
sandstone treads climb to a crenellated balcony, but in the hero they read as
stacked tan rings beneath a plain horn with only a tiny gold ball. Neither run
spent a pass rebuilding its stair, where both round 6 runs first stacked theirs
into flat rings.

Failures:

1. Run 1, pass 2: leaning the roof toward -45 degrees aimed the bend at the hero
   camera and hid the crook. Pass 3 turned the lean to 45 degrees
2. Run 2, pass 3: the render failed with
   ``view close's transform orbits a point at a `fit` distance, and an orbit about a point takes its distance in meters``.
   The final review sheet stayed stale from pass 2, and the joined star shows
   only in the close and back renders

Lacked:

None

Missed in the skill:

1. Run 2 passed `fit` with `--view-look-at` although `SKILL.md` gives that orbit
   a distance in meters. The failed render cost a pass and left the final sheet
   stale
2. Run 2 bent its roof toward +x although the skill faces the prompt's detail
   toward the hero corner unobstructed. The right view looks along the bend and
   shows a straight cone

Colors: Run 1's cell-stone wall shows vertical stripes in the close-up.

### 23. Log cabin

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | good    | 5      | 0      | 4.4     | 1,024,146 at 5 cm |
| 2   | good    | 4      | 0      | 5.2     | 230,226 at 5 cm   |

Both runs built a round-log cabin at 5 cm with glowing windows, roof snow slabs
that droop over the eaves, pale-blue icicles, and gray chimney smoke. Run 1
reads better because its drifts climb about 1.3 m up the walls and its icicles
up to 1.15 m long stand out against the dark logs. Its plume reads smooth and
tube-like with little billow. Run 2 sets its cabin on an earth-strata base with
a tall chimney, billowing smoke, and three snowy pines, but its snow reaches
only sill height. Its icicles form a stubby fringe of at most 0.75 m.

Failures:

1. Run 1, pass 1: the ground snow bulged below y = 0 into a saucer underside
   down to y = -1.35. Pass 2 clipped the ground
2. Both runs, pass 1: the smoke's `displace` left one-voxel crumbs as extra
   pieces. Run 1 reseeded the noise over passes 4 and 5 to clear smoke and snow
   crumbs, while run 2 reshaped the puffs in pass 2
3. Run 1, pass 2: the door handle read `0 kept` because ground snow added later
   buried it. Pass 3 moved the door steps after the path carve
4. Run 2, pass 1 onward: the chinking step kept 0 of 81,632 cells under the logs
   and snowpack. No chinking shows, though the session's final message claims it
5. Run 2, pass 3: two one-voxel roof-snow crumbs sat past the gable ends. Pass 4
   clipped them

Lacked:

None

Missed in the skill:

1. Run 2 never acted on its chinking at `0 kept` although the skill's checks
   flag every such step. The walls lack the filler the session described

Colors: Both runs' path walls are flat sheer white planes. Run 1's snow field
reads nearly uniform white in the top view. Run 2's leaf-and-snow noise on the
pines reads as random speckle rather than snow lying on the boughs.

### 24. Valley

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | good    | 7      | 0      | 9.8     | 236,650 at 1 m    |
| 2   | good    | 5      | 0      | 7.2     | 206,841 at 1.25 m |

Both runs built a surface-filled square diorama under 300,000 voxels with snowy
peaks, a banked river leaving through a V gorge, and scattered tiered pines. Run
1 reads better because its bright blue river with gravel banks winds clearly
down a 240 m tile. Its peaks are evenly spaced displaced cones that read as rows
of blobs from the top, and its valley floor sits perfectly flat. Run 2's 300 m
tile has more natural rugged peaks, open green hills, and 260 pines in groves,
but its navy river reads nearly black in the hero. Run 1 plants its pines only
on the flat floor and undisplaced foothills, where round 6's run 1 seated 452
pines on ground heights read back from the voxelized terrain.

Failures:

1. Run 1, pass 3: the gorge render failed with
   ``view gorge's transform orbits a point at a `fit` distance, and an orbit about a point takes its distance in meters``
2. Run 1, pass 3: displacing the stepped gorge walls broke the terrain into 20
   pieces. Pass 4 replaced the displace with `smoothUnion`
3. Run 2, pass 1: five trees on the displaced slopes floated as separate pieces
   because their trunks sank only 8 m. Pass 2 sank them 16 m
4. Run 2, pass 2: the first gorge, a polygon extruded along z, did not follow
   the river's bends. Pass 3 swept cones along the river path instead
5. Run 2, final pass: two carve boxes at hard-coded report coordinates delete
   the last crumbs. Any terrain edit breaks them

Lacked:

1. The terrain height at a point: run 1 re-implemented its hill heights in
   TypeScript and kept trees off the displaced cones. Run 2 mirrored the cones
   and `smoothUnion` in a TypeScript `groundAt` and sank each trunk 16 m
2. A step that drops pieces under a size: run 1 swept the displace seed from 12
   to 16 until seed 13 left one piece. Run 2 carved its last two crumbs by
   coordinates

Missed in the skill:

1. Run 1 passed `fit` with `--view-look-at` although `SKILL.md` gives that orbit
   a distance in meters. The failed gorge render cost a pass

Colors: Run 1's meadow reads a fairly uniform lime green between the trees. Its
right and back cut faces spread large expanses of gray strata, though the
banding keeps them from reading flat. Run 2's river reads as one flat dark navy,
nearly black in the hero and top views, with little contrast against the shaded
gorge.

### 25. Village

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | good    | 4      | 1      | 7.7     | 455,908 at 0.25 m |
| 2   | good    | 4      | 0      | 8.9     | 180,416 at 0.25 m |

Both runs ringed a cobbled square of four striped stalls and a roofed well with
thatched cottages beside a stone church with a clock tower, a louvred belfry and
a walled churchyard. Run 2 reads better because its seven cottages vary in
footprint, mix hipped and gabled thatch and carry gable or ridge chimneys. Run
1's eight cottages share one hipped silhouette at nearly one size and stand in
rigid rows that read from the top as a grid of identical yellow boxes. Where
round 6 had no turn per placement and left its stalls facing +z, round 7's part
`rotation` turned both runs' cottages toward the square and run 2's stalls
toward the well.

Failures:

1. Run 1, pass 1: the build failed with
   `stall.c goods 1: shades shade 2 must be at the base's chroma ... at most 0.044 at count 3`.
   A spread of 0.04 fixed it
2. Run 1, pass 2, and run 2, pass 1: stray tree-crown voxels and run 1's
   floating well roof left run 1 in 18 pieces and run 2 with 29 loose leaf
   voxels. Later passes brought both to 1 piece
3. Run 2, pass 3: the zig-zag ridge band and box ridge roll read as a loose
   slab. Pass 4 replaced them with an offset ridge cap
4. Run 1, final pass: the eight cottages share one hipped-thatch silhouette in
   two rigid rows of three plus a pair. Plaster color, door color and
   half-timbering are their only variety
5. Run 1, final pass: the well's single-pitch roof slab looks odd. The stall
   produce hides under the awnings
6. Run 2, final pass: the spire is a plain four-sided pyramid. The gabled roofs'
   ridge caps still read as a heavy stacked slab
7. Run 2, final pass: a tree at the square's corner crowds the church tower

Lacked:

None

Missed in the skill:

None

Colors: Run 1's crates read as solid timber blocks because the 0.25 m `boxFrame`
covers the whole 0.75 m crate. Run 2's gravestones take one flat shade. Its dark
thatch ridge band blends in and barely reads.

### 26. City

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | good    | 2      | 0      | 5.9     | 230,544 at 0.25 m |
| 2   | good    | 3      | 0      | 7.8     | 250,005 at 0.25 m |

Both runs built one root object at 0.25 m with crosswalks, arm street lamps,
glass towers and a park. Run 2 reads better because its 60 m nine-block grid
with streets on every side and a skyline climbing toward the back reads as a
small city in the hero. Run 1's 56 m crossroads holds the richer park with a
fountain, a hedge and flower beds, but its four blocks around one intersection
read as a single city block.

Failures:

1. Both runs, pass 1: stray crown voxels and run 1's floating bench backs left
   run 1 in 16 pieces and run 2 in 6. Pass 2 brought both to 1 piece
2. Run 2, pass 2: the storefronts read as flat beige panels at street level.
   Pass 3 darkened them and added mullions
3. Run 1, final pass: four blocks around one intersection read as one city
   block. The roofs are large flat gray slabs in the top view
4. Run 2, final pass: the lit storefronts still read as flat saturated orange
   slabs. The park fills one small block crowded by four lamps whose black caps
   over cream globes read oddly

Lacked:

None

Missed in the skill:

None

Colors: Both runs' roofs are plain single-tone gray slabs. Run 1's dark blue
window glass has no rim, but its lit windows break it up. Run 2's storefront
glow is a uniform orange slab at street level. Its lamp heads read flat pale
yellow, and its white concrete tower takes a single tone.

### 27. Dollhouse

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | good    | 4      | 0      | 9.8     | 965,592 at 5 cm   |
| 2   | good    | 6      | 2      | 12.3    | 1,181,873 at 5 cm |

Both runs built a blue clapboard dollhouse at 5 cm with the front wall open,
white-trimmed cut edges and a front-to-back ridge that opens the gable onto an
attic over six furnished rooms. Run 2 reads better because its 10.4 m house adds
a railed landing over an open stairwell and its sheet hero shows all six rooms
and the attic at once. Run 1's right roof slope hides half the upstairs and the
bathroom from its default high hero.

Failures:

1. Run 1, pass 1: the ridge ran along x. The front roof slope hid the upstairs
   rooms until pass 2 turned the ridge front to back
2. Both runs, pass 1: floating props left run 1 in 10 pieces and run 2 in 21.
   Later passes reattached every floater
3. Run 2, pass 1: the build failed with
   `gradient to must be past from 2.5, not 1.9` because two painting gradients
   ran high to low
4. Run 2, pass 1: the table runner hit 0 cells
5. Run 1, pass 2: the attic-gable paint spilled onto the roof as a white line.
   Passes 3 and 4 confined it to the gable
6. Run 2, pass 3: the renders failed with
   `4 values required for '--view-orbit <view> <azimuth> <elevation> <distance>' but 3 were provided`
   because the view name was left out
7. Run 2, pass 5: the emissive paint meant to light the black ceilings sat one
   voxel below the slabs and missed them. Pass 6 landed it
8. Run 1, final pass: the right roof slope hides half the upstairs and the
   bathroom from the sheet's default hero. The middle upstairs stair hall is a
   narrow, nearly empty slot
9. Run 2, final pass: the chimney stands apart from the left wall as a thin
   brick column that reads as a pipe. The attic's back gable is a flat brown
   panel

Lacked:

1. Run 2: an ambient or fill light. Render lighting leaves every ceiling under a
   slab black, and the session painted the ceilings with an emissive plaster
   instead

Missed in the skill:

1. Run 2: SKILL.md shows the form `--view-orbit hero 225 20 fit`. Pass 3 passed
   three values and lost a pass

Colors: Both runs' attic back gable wall and roof underside read as flat panels,
pale in run 1 and brown-tan in run 2. Run 1's plain tan stair-hall walls look
unfinished beside the papered rooms. Run 2's hall and kitchen side walls are
plain beige panels.

### 28. Living room

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | good    | 2      | 0      | 5.2     | 283,043 at 2.5 cm |
| 2   | good    | 4      | 1      | 5.7     | 259,571 at 2.5 cm |

Both runs built an open-corner room with papered walls over wainscot, a brick
fireplace with a glowing fire, a wingback, a night window and a seeded bookshelf
of varied books with gold bands and flat stacks. Run 1 reads better because its
150 books in 10 colors pack the shelves where run 2's 100 leave headroom and
gaps. Run 2 builds the cozier room with a fire on logs and a grate, but neither
run's chair faces the fire. Neither round 7 chair ribs the way both round 6
chairs did.

Failures:

1. Run 2, pass 1: the build failed with
   `living-room/rug corners: scale factor must be above zero on each axis, not [-1, -1]`
   because the session flipped the rug corner triangles with a negative scale
2. Run 1, pass 1: a vase on the shelf and the buttoned back cushion read as a
   figure and a face. Pass 2 removed them
3. Run 2, pass 2: the chimney showed a gray mortar top while the moon poked out
   the back of the wall. Later passes ran the end bricks flush and flattened the
   moon
4. Run 2, passes 2 and 3: loose plant leaves left 22 and then 10 pieces. Pass 4
   joined them with a stem
5. Run 1, final pass: the red wingback faces away from the fire toward the open
   corner and blocks part of the firebox in the hero. Up close the fire is a
   flat orange-yellow blob with no logs
6. Run 2, final pass: the books stand short for their shelves and leave headroom
   and gaps. The chair faces sideways across the room

Lacked:

None

Missed in the skill:

1. Run 2: SKILL.md documents `mirror("uv")` for 2D shapes. Flipping the rug
   corners with a negative scale cost a failed pass

Colors: Both runs' books take flat single colors that read fine at their size.
Run 1's large red rug field reads as one flat color. Run 2's green armchair and
ottoman read flat beside the speckled rug and grained wood.

### 29. Spaceship

| Run | Verdict | Passes | Failed | Minutes | Voxels              |
| --- | ------- | ------ | ------ | ------- | ------------------- |
| 1   | good    | 5      | 1      | 4.9     | 1,317,136 at 2.5 cm |
| 2   | fair    | 5      | 1      | 5.8     | 511,684 at 5 cm     |

Both runs built a bridge with a red captain's chair, a green radar and a window
onto stars and a gas giant. Run 1 reads better because its open dollhouse room
and a shallow star shadowbox behind the window let the standard sheet show the
whole bridge as one piece in 6 m bounds. Run 2's enclosed bridge shows ceiling
ribs, a door and a richer chair from its interior cameras, but its sheet shows
only a gray box beside a 13 m star plane and a floating planet. Both round 6
runs ended with an open cutaway and a sealed star box, but round 7's run 2
sealed the hull and left its star plane free.

Failures:

1. Both runs, pass 1: the star `set` failed with
   `set points must be distinct, not [0.675, 0.775, -3.425] twice` in run 1 and
   the same error in run 2. Both sessions deduped the points
2. Run 1, pass 2: the report listed `wall screens R` at `0 kept` and lost to the
   mirrored `wall screens L`. Pass 3 replaced both
3. Run 2, pass 2: the floor read `0 kept` and lost to the hull. Pass 3 reordered
   the steps
4. Run 2, pass 3: the radar tilted -50 degrees broke its rings into 52 pieces.
   Pass 4 laid it flat
5. Run 1, final pass: the room has no ceiling and a lowered right wall. The
   inside view shows white sky above the header and on the right
6. Run 1, final pass: the chair back is a plain red slab with a black spine. The
   back view shows a dark purple box
7. Run 2, final pass: the sheet shows only a noisy gray hull beside a 13 m star
   plane and a planet that floats as one of 3 pieces. Nothing the prompt names
   shows from outside
8. Run 2, final pass: the radar's one-voxel rings break into a green maze at 5
   cm. The sweep sector shrinks to a jagged streak
9. Run 2, final pass: from the inside camera the chair is a dark gray slab
   filling the foreground

Lacked:

1. Both runs: a render-time clip or section plane to see past the hull. Run 1
   carved away the ceiling, back and right wall while run 2 leaned on interior
   cameras
2. Both runs: SKILL.md never says `set` points must be distinct. Each first
   build failed on duplicate star cells

Missed in the skill:

1. Run 1: SKILL.md says a room keeps its walls and takes an inside view. Carving
   the ceiling and right wall left white sky in the inside render
2. Run 2: the skill says every named feature spans at least 2 voxels. The radar
   rings were 0.05 m shells one voxel wide at 5 cm and broke up
3. Run 2: the skill checks that the bounds match the prompt's real size. The
   star plane 5 m beyond the window and the planet pushed a 6 m room's bounds to
   13 x 8 x 12 m

Colors: Run 1's chair back reads from behind as a flat red slab split by a black
bar. Its outer gray walls read banded but bland. Run 2's chair back and frame
merge into one dark gray slab from inside. Its hull reads as undifferentiated
gray noise, and its dimmed radar rings read dull green on the dark screen.

### 30. Pirate ship

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | good    | 3      | 0      | 4.7     | 713,472 at 2.5 cm |
| 2   | fair    | 4      | 0      | 7.5     | 482,149 at 2.5 cm |

Both runs built a 2.5 cm gun deck with cannons run out through carved ports,
ellipsoid hammocks, barrels, crates and a glowing lantern. Run 1 reads better
because its curved, tumblehomed hull with ribs, beams and four cannons on both
sides reads as a ship below deck from every review view. Run 2's eye-level view
holds the finer props with roped cannons at sea-view ports, a ladder to a grated
hatch and a brass-trimmed lantern, but its flat-walled box with two cannons on
one side reads as a cabin or shed. Round 6's run 1 hero showed only a sealed
planked box, while round 7's run 1 left half the overhead off for the hero and
top views to see in.

Failures:

1. Run 2, pass 1: the build failed with
   `noise seed must be a whole number from -2^31 to 2^31 - 1, not 901.5` because
   the seed came from a fractional position
2. Run 1, pass 1: the bilge under the floor rendered as a white gap in the front
   and back views. Pass 2 filled it with stone ballast
3. Run 1, pass 1: the door ring hit 0 cells. Moving it off the door in pass 2
   split the model into 2 pieces until pass 3 moved it back
4. Run 2, pass 1: the knee polygons swapped u and v for an x-axis extrude. Six L
   brackets floated below the floor in the hero and front views
5. Both runs, final pass: the hammocks sag into deep bowls that read as baskets
   or boats. Run 2's show stair steps
6. Run 1, final pass: the 10-point hull polygon shows straight facets. The
   cannons and carriages are chunky blocks
7. Run 2, final pass: the room is a flat-walled timber box with one straight
   hull side and only two cannons. The full overhead makes the top view a plain
   deck
8. Both runs, final pass: the lantern lights nothing around it

Lacked:

1. Both runs: a light source that lights its surroundings. The emissive flames
   glow on the lanterns alone

Missed in the skill:

1. Run 1: the hull profile has only 10 vertices. More points would have curved
   the bilge and tumblehome instead of faceting them
2. Run 2: SKILL.md says an x-axis extrude maps the profile to `(y, z)`. Writing
   the knees as `(z, y)` cost a pass
3. Run 2: SKILL.md points a hollow form at an extruded polygon. The session
   never curved the hull

Colors: Run 1's ochre wale reads as one flat saturated band. Its evenly spaced
black floor seams read as a printed grid, and its stone bilge looks like gray
gravel. Run 2's sea and sky plates read as flat painted panels from the back
view. Its walls are one uniform dark brown, and its floor noise reads blotchy
rather than grained.
