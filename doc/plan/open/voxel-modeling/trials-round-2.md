# Trials, round 2

Step S14 of the [checklist](checklist.md) runs the [prompts](trials.md#prompts)
a second time with S13's changes. [Round 1](trials.md#findings) logs the first
run of the prompts, and the [trial
harness](../../../../projects/vxl/trials/README.md) ran both rounds.

## Findings

Round 2's analyses recorded 672 items across the 60 runs. Each item maps to one
or more findings below. A finding round 1 also found keeps its number, and new
findings start at 65. Separate agents analyzed and triaged each round, so a
count can drift by a prompt or two between rounds. The round's gallery lists
each finding's items beside the renders.

### S13's answers

S13 answered 36 of round 1's findings. A finding round 2 still showed reopens
even where its count fell. The table counts the prompts that showed each finding
in each round.

| #   | Finding                                                | Round 1 | Round 2 |
| --- | ------------------------------------------------------ | ------- | ------- |
| 2   | Large surfaces left flat                               | 24      | 24      |
| 8   | Renders skipped                                        | 12      | 22      |
| 4   | Ended with flaws the renders show                      | 18      | 22      |
| 1   | Edits met the permission check                         | 27      | 19      |
| 28  | Skipped the box block-out                              | 5       | 15      |
| 19  | The hero view hid the focal element                    | 8       | 12      |
| 10  | Features near a voxel thick missed cells or broke      | 11      | 12      |
| 13  | Neighboring materials too close in hue                 | 10      | 10      |
| 3   | Documented tools left unused                           | 18      | 9       |
| 12  | Shapes at an angle alias                               | 10      | 9       |
| 6   | Small round shapes read as blocks or plus signs        | 13      | 9       |
| 5   | Dark materials collapse to near-black                  | 17      | 8       |
| 9   | Finer grids read better than the guidance              | 12      | 7       |
| 7   | Later steps buried earlier details                     | 13      | 6       |
| 34  | Paint and coat spill onto neighbors                    | 3       | 5       |
| 14  | Glass, water, ice, and gems read wrong                 | 9       | 4       |
| 17  | Views the review four cannot give                      | 9       | 4       |
| 36  | Placements share no object                             | 2       | 3       |
| 32  | Grain read wrong                                       | 3       | 2       |
| 52  | A scratch model for one shape                          | 1       | 2       |
| 18  | Emissives wash out                                     | 8       | 2       |
| 16  | Signatures misread                                     | 9       | 2       |
| 21  | Metallic pulls custom colors off hue                   | 7       | 2       |
| 20  | Library metals read off hue                            | 7       | 1       |
| 15  | `shades` left the sRGB gamut                           | 9       | 1       |
| 54  | A variable named like a call hid the call              | 1       | 1       |
| 31  | Library rope and grass read off                        | 3       | 1       |
| 27  | Light surfaces turn blue-gray                          | 5       | 0       |
| 30  | Proportion and silhouette drift                        | 4       | 0       |
| 37  | Mortar swamps stone walls                              | 2       | 0       |
| 40  | Round limits failed on float drift                     | 2       | 0       |
| 44  | Placing on the cell grid took hand work                | 2       | 0       |
| 45  | Box corners in any order                               | 2       | 0       |
| 55  | 0 kept flags intended repaints                         | 1       | 0       |
| 56  | The model's grid constant and the voxel size disagreed | 1       | 0       |
| 57  | `--version` on the binaries                            | 1       | 0       |

The fixes that changed vxl held best. No run met a box corner error, a
float-drift round, a missing `--version`, or a blue overhang. Emissive wash fell
from 8 prompts to 2, olive metals from 7 to 1, and `shades` errors from 9 to 1.
The skill text did less. Skipped renders rose from 12 prompts to 22 and skipped
block-outs from 5 to 15, although the checks ask for all four PNGs on every pass
and the building steps open with boxes. Edits through python, `sed`, or `perl`
still met the permission check in 19 prompts.

Round 2 showed the reopened findings this way:

1. **Large surfaces left flat.** In 24 prompts against round 1's 24, large faces
   still stayed one flat color. Iron straps, a wax candle, an asphalt deck,
   armor sides, and a dark chest panel kept bare materials although the skill's
   check asks every large face for shades, noise, or a pattern. Where sessions
   did add noise, wide lightness spreads read as camouflage blotches or beige
   gold
2. **Renders skipped.** In 22 prompts against round 1's 12, sessions still
   skipped views even though the check asks for all four PNGs on every pass.
   Late passes often read only the hero or no PNG at all. The fish tank's
   all-blue top view and the spiral stair's sawtooth rail sat in views no pass
   read
3. **Ended with flaws the renders show.** In 22 prompts against round 1's 18,
   sessions still ended on flaws they had named. A roof hiding the well rope, a
   cap hiding the door, and boughs at 0 exposed went into final messages despite
   the check that a named flaw takes another pass. Sessions also explained away
   floating one-voxel pieces as stray coins, loose flakes, or too small to see
4. **Edits met the permission check.** In 19 prompts against round 1's 27,
   sessions still edited through a python heredoc, `sed -i`, or `perl` before
   the file-editing tool. Each denial cost a turn until the session redid the
   change as Edit calls. Three passes wrapped in `time ( ... )` met
   `A group in parentheses in this command can't be checked before it runs`
   although the skill names that trap
5. **Skipped the box block-out.** In 15 prompts against round 1's 5, sessions
   skipped the box block-out more often. The furnished house runs opened with
   models of 430 and 500 lines. Runs that did block out on the cart, the
   round-door cottage, and the fireplace room each beat the run that skipped
6. **The hero view hid the focal element.** In 12 prompts against round 1's 8,
   the hero view still hid what the prompt names. Roofs, a crown lobe, and a
   lychgate stood between the hero corner and a well rope, a trunk, or a church
   door. One well run turned its gable to the front, but other sessions added
   close-ups instead of re-aiming with `--view-orbit`
7. **Features near a voxel thick missed cells or broke.** In 12 prompts against
   round 1's 11, first drafts still drew features under two voxels across. A
   half-voxel antenna cylinder and a 0.02 m puddle ellipsoid wrote 0 cells.
   Where sessions applied the rule, it held. A handle tube widened to 0.025 m
   and gear teeth about 2 cells wide stayed whole
8. **Neighboring materials too close in hue.** In 10 prompts against round 1's
   10, neighboring materials still merged. Library leaf and grass greens blended
   the oak's crown into the turf. The watch's `#1E3C9A` hands vanished over its
   `#3A3E46` plate although check 4 asks neighbors to sit far apart in hue or
   lightness
9. **Documented tools left unused.** In 9 prompts against round 1's 18, sessions
   still built by hand what a documented operation does. Chest gems sat at
   hand-computed heap heights instead of riding `coat` with sides. The text
   partly held. One spaceship run scattered its stars with `speckle` where the
   other hit a duplicate `set` point
10. **Shapes at an angle alias.** In 9 prompts against round 1's 10, shapes off
    the axes still sampled into ribs and stair steps. The armchair turned 45
    degrees came out ribbed although the skill says furniture reads cleanest
    square to the axes. Other sessions dodged the angle by standing a coin on
    its edge or keeping candelabra arms on the axes
11. **Small round shapes read as blocks or plus signs.** In 9 prompts against
    round 1's 13, round shapes under about 3 voxels in radius still read as plus
    signs or blocks. Robot ears from a 2-voxel cylinder, emerald octahedrons of
    radius 2 voxels, and a 2-voxel `ngon` gem table all broke the Resolution
    rule on the first draft. Sessions that saw the plus sign swapped in boxes
    for the ears and hub cap
12. **Dark materials collapse to near-black.** In 8 prompts against round 1's
    17, fewer dark materials collapsed after the neutral fill. Ebony fretboards
    and chess pieces, cannon barrels, crane track belts, and the ramen stall's
    back walls still read as flat near-black. Walnut inlay between gold settings
    read as dirt or shadow
13. **Finer grids read better than the guidance.** In 7 prompts against round
    1's 12, sessions still picked grids too coarse for the smallest feature. A
    sword at 1 cm left its blade 2 voxels thick where the other run showed 5 mm
    fit the budget. The new text held for the guitar. Both runs took its quoted
    0.004 m and ran 260 voxels long without trouble
14. **Later steps buried earlier details.** In 6 prompts against round 1's 13,
    later steps still took earlier details' cells. The oak's boughs shipped at 0
    exposed under a crown added after them. Where a session read the report's
    counts, they steered the fix. One dragon run reordered its steps to restore
    a tail spade that kept 6 of 32 cells
15. **Paint and coat spill onto neighbors.** In 5 prompts against round 1's 3,
    broad paints and unscoped coats still recolored neighbors. A clownfish
    stripe paint bounded only in y and z laid white bands across the whole tank.
    An unscoped `coat` recolored the whole potion flask until the session added
    `within` a pass later
16. **Glass, water, ice, and gems read wrong.** In 4 prompts against round 1's
    9, tinted glass still washed what sat behind it. The worst case turned the
    fish tank's red fish mauve. Every run kept transmission instead of the
    `#RRGGBBAA` baseColor the skill documents
17. **Views the review four cannot give.** In 4 prompts against round 1's 9,
    `--view-orbit` close-ups still missed their part. An orbit close-up on the
    cottage door aimed at the model center and showed mostly cap. The crane run
    framed its hook with `--view-position` and `--view-look-at` instead
18. **Placements share no object.** In 3 prompts against round 1's 2, repeated
    props still shared no object. The valley runs unioned 230 and 120 identical
    pines into two steps instead of placing one part per spot. One tile room
    took `--frame local` and shared 9 objects against its pair's 23
19. **Grain read wrong.** In 2 prompts against round 1's 3, grain still read as
    blotches or flat tone rather than streaks. Both well runs built ridge grain
    in place far from the origin even though the skill says it reads there as
    one broad ring. The cart's walnut crossbar read pink-brown with pale
    blotches
20. **A scratch model for one shape.** In 2 prompts against round 1's 1, writes
    under `/tmp` still met the permission check. A pocket watch run aimed a
    backup copy there. A chess run aimed a `sips` crop there even though the
    skill keeps scratch work beside the model file
21. **Emissives wash out.** In 2 prompts against round 1's 8, emissives still
    paled. The ship lantern's pane at strength 1.5 stayed pale peach although
    the check asks a pale glow for a lower `emissiveStrength` or a darker
    `baseColor`. The candle flame's core bloomed near white with a small orange
    tip. S13 chose that look for strengths of 2 to 4
22. **Signatures misread.** In 2 prompts against round 1's 9, sessions still
    misread argument order. The ramen stall's blade sign rim took swapped `rect`
    coordinates and drew a stray cyan frame on the ground. A ship hammock bent
    after shelling came out with ragged rims despite the skill's warning on the
    order of `shell` and `bend`
23. **Metallic pulls custom colors off hue.** In 2 prompts against round 1's 7,
    metal colors still read off after the white sky. One dragon run's gold pile
    read as tan sand with few bright accents. The ship deck's cannon barrels and
    shot pyramid read as near-black blocks with little shading
24. **Library metals read off hue.** In 1 prompt against round 1's 7, library
    gold still read off hue. Both pocket watch runs rendered the gold case,
    bezel, and chain olive-khaki in the front and right views but mustard in the
    hero. Neither session named the olive cast
25. **`shades` left the sRGB gamut.** In 1 prompt against round 1's 9, `shades`
    still met a near-white base. A custom `#F4F8FC` snow failed with
    `shades shade 2 must be between black and white, not a lightness of 1.012`.
    The skill warns against the near-white base behind the one error. The gamut
    fix held otherwise
26. **A variable named like a call hid the call.** In 1 prompt against round 1's
    1, a material named like a call still hid the call. The fireplace room's
    `star` material cost a pass by breaking the `star` 2D shape with
    `TypeError: star is not a function`. The session recovered by renaming it to
    `starLight`
27. **Library rope and grass read off.** In 1 prompt against round 1's 3, a
    library color still read off. The well session defined custom mortar and
    shingle materials because library stone and pine read washed out

### Still open

Nothing changed for round 1's phase 2 candidates. Round 2 counted them again:

| #   | Finding                                           | Round 1 | Round 2 |
| --- | ------------------------------------------------- | ------- | ------- |
| 11  | Seating details on a surface                      | 10      | 12      |
| 24  | Floating crumbs                                   | 5       | 10      |
| 25  | Sweeps, helices, and spirals                      | 5       | 5       |
| 23  | Scene reports too long to read                    | 6       | 5       |
| 26  | A seeded random helper                            | 5       | 5       |
| 22  | Rotation on part placement and a one-sided flip   | 6       | 3       |
| 35  | Posed parts, plumb joints, and ropes across parts | 2       | 2       |
| 39  | Inspecting nodes and palette values               | 2       | 2       |
| 46  | Emissives light nothing nearby                    | 2       | 2       |
| 51  | Exposure inside transmissive volumes              | 1       | 2       |
| 48  | Crevice and edge shading in the colors            | 1       | 2       |
| 38  | Intended separate pieces read as floating         | 2       | 1       |
| 41  | A flatter arch and a bend past half a turn        | 2       | 1       |
| 47  | Text                                              | 1       | 1       |
| 42  | Part pivots in the report                         | 2       | 1       |
| 53  | Parts that cut through each other                 | 1       | 1       |
| 29  | A rendered-color check                            | 4       | 0       |
| 33  | Report lines without a location                   | 3       | 0       |
| 43  | A flag for washed-out or hidden emissives         | 2       | 0       |
| 49  | A section view of hollow forms                    | 1       | 0       |
| 50  | Hollowing a union                                 | 1       | 0       |
| 58  | Declared materials left unused                    | 1       | 0       |
| 59  | Patterns that line up across tiles                | 1       | 0       |
| 60  | Exposed counts per material                       | 1       | 0       |

### New in round 2

65. **Noise read as camouflage.** In 6 prompts, noise or grain swung so far in
    lightness that surfaces read as camouflage, stains, or stripes. Sword run 2
    left three shades at scale 0.06 on its steel blade even though its pass 4
    close-up showed the camouflage. Grass banks, a crane's yellow roof,
    dollhouse partition walls, living-room walls and velvet, and chess pieces
    broke up the same way
66. **Sessions guessed render file names.** In 3 prompts, the render command
    printed no output paths. Sessions guessed file names or listed the folder.
    Dragon run 1 and crane run 1 each read a close-up name with an extra
    `-close` suffix that `--file-stem` never wrote. In chess run 2, a bishop
    close-up overwrote the knight close-up because both `--select` renders
    shared the stem `chess-close`
67. **Large scenes went without close-ups.** In 2 prompts, sessions rendered no
    close-up with `--view-orbit` or `--view-select` of details that sit tiny in
    the square frames. On the 124 m bridge, run 2's cable sat below the rail top
    at midspan, where it merged into the railing unseen. City run 1 left
    sub-meter windows, lamp heads, and benches unchecked
68. **Lightened gold shades read beige.** In 2 prompts, gold's lightest shade
    lightened toward white until it read beige or pale tan. Sword run 1 used a
    `shades` spread of 0.06, yet the noise on its langet, guard, and pommel read
    as tan camouflage. Coins in both chest runs read pale cream or tan because
    their noise used the lighter shades
69. **Oak read orange and walnut read pink.** In 2 prompts, oak and walnut
    rendered off-hue. Oak posts read a bright, nearly uniform orange in both
    well runs. Walnut shafts and crossbars read pink-brown in both cart runs
70. **Read-only grep met the permission check.** In 2 prompts, a read-only
    `grep` to locate edit lines hit `This command requires approval`.
    Living-room run 1 switched to the Edit tool. Pirate-ship run 2 edited
    without locating the lines
71. **Sessions cut the report to its model line.** In 2 prompts, sessions piped
    the voxelize report through `head -1` and hid every step line. Dollhouse run
    1's pass 2 edits went unchecked for `0 cells` or `0 exposed`. Spaceship run
    1 lost the step lines in its pass 4 final check
72. **Surface fill reported the solid count.** In 1 prompt, the voxelize report
    under `--fill-mode surface` printed the solid count of 2,951,819 voxels for
    a file holding 265,947. A rerun of run 1's model confirms it. Run 1 found
    the real count with `vxl vox-doc show`. Run 2 believed the report and told
    the user surface fill does not reduce the count
73. **Close-ups selected the whole sword.** In 1 prompt, both sword runs aimed a
    close-up at the pommel with the glob `'sword'` in `--view-select`. The glob
    matched the whole model and framed the blade instead of the pommel. Both
    runs fell back to a positioned camera
74. **Hero renders came out on black.** In 1 prompt, both valley runs saw their
    hero render on black while front and right showed white. Every review PNG
    keeps a transparent background. The black renders were the largest files, at
    500 to 660 KB against 110 to 225 KB. Reading a large PNG probably drops its
    alpha
75. **Flame emissive dropped below the bloom threshold.** In 1 prompt, the
    lantern session lowered every flame material to emissive strength 1 to
    deepen its color. Below the bloom threshold, the flame rendered as matte
    pastel bands with no halo. SKILL.md says a strength of 2 to 4 adds a halo
    while a dark `baseColor` keeps the hue
76. **Kit tiles exported as functions named nothing.** In 1 prompt, both
    dungeon-kit runs exported tile builders as functions. SKILL.md says an
    exported function names nothing. Only run 1's `floorTile` came through as a
    named part. Neither `kit.sdfj` could serve as a tile library
77. **Painted glint read as an opaque squiggle.** In 1 prompt, the potion
    session painted a white low-transmission patch on the glass shell to suggest
    a specular highlight the renderer does not draw. Pass 6 placed the first
    glint too large and off the hero face. After pass 7 shrank it, the glint
    still read as a flat opaque white squiggle
78. **Terrain lacked a heightfield primitive.** In 1 prompt, both valley runs
    built the land from a slab plus displaced cones in a `smoothUnion` because
    vxl has no heightfield or terrain primitive
79. **The sandbox blocked listing the trials folder.** In 1 prompt, both city
    runs opened with an `ls` of the parent `~/voxel-trials` folder, and the
    sandbox blocked it. Run 1 retried inside its own folder
80. **Neither pirate ship curved its hull.** In 1 prompt, neither pirate-ship
    run attempted the curved hull. Both built a flat box wall that reads as a
    generic wooden room rather than a ship's hull
81. **Sloped snow drew stair-step contour lines.** In 1 prompt, the snow in both
    cabin runs rendered with dense dark stair-step contour lines. The lines read
    as noise across run 1's roof and drifts in the hero and top views. Run 2's
    snow ground showed the same edges
82. **The harness counted a render as a pass.** In 1 prompt, the harness counted
    4 passes for guitar run 2 though only 3 commands voxelized. `passes/03`
    holds a standalone close-up render instead of a voxelize
83. **Orthographic views showed moire.** In 1 prompt, the front and right
    orthographic views of valley run 1 showed moire across the snow and rock.
    The rock bands read as evenly spaced horizontal gray stripes rather than
    strata
84. **The gray hook washed out on white.** In 1 prompt, crane run 2's gray hook
    washed out against the white render background

### No action

The four findings that call for no action kept their shape. In 29 prompts
workarounds used the tools as intended, and in 25 the two runs chose
near-identical plans. In 21 prompts sessions made one-off slips. In 19 the
report and the views caught real defects.

## Log

Round 2 ran on 2026-10-05 with vxl 0.5.0 built from `6cef3939` and Claude Code
2.1.289 running Claude Opus 5.5 at high effort. Every run loaded the skill from
its description and called its model done. Of the 60 runs, 40 read good and 20
fair. The runs took 225 passes, of which 9 failed. A run took a median of 3.7
minutes, and all 60 cost $74. `~/voxel-trials/rounds/2026-10-05` holds the
round, and its gallery shows every run's renders.

A first attempt at the round loaded a user-level copy of the 0.4.0 skill in
place of each slot's copy. `rounds/2026-10-05-stale-skill` keeps that attempt,
and `setup.sh` now stops when a user-level skill differs from the installed
vxl's.

### 1. Chair

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | fair    | 2      | 0      | 1.6     | 13,980 at 1.25 cm |
| 2   | good    | 6      | 0      | 2.7     | 17,472 at 1.25 cm |

Both runs built an oak chair at 1.25 cm with a gilded crest rail set with a
ruby, sapphires, and emeralds. Run 2 reads better because its six passes grew a
box block-out into faceted gems in a gold medallion under diamond finials where
run 1 stopped after one fix pass with gem specks and plus-sign finials. Unlike
round 1, neither run failed a pass at the finer voxel.

Failures:

1. Run 1, pass 1: the gold scrollwork arcs broke into two floating 6-voxel
   pieces. The 3-cell `set` diamonds were too small to read
2. Run 1, final pass: the session declared the model done with emeralds,
   amethysts, and topaz still 1 to 2 voxels across
3. Run 2, pass 2: the mid stretcher and both end diamond finials floated free as
   4 pieces
4. Run 2, pass 3: a python heredoc rewrite of the crest was denied with
   `This command requires approval`. The pass rebuilt the unchanged file
5. Run 2, pass 4: the end diamonds still floated at y 1.17 above their cups
   until pass 5 lowered them to 1.16

Lacked:

None

Missed in the skill:

1. Run 1 skipped the box block-out and wrote turned legs, splat, crest, and
   jewels into its first pass
2. Run 1 gave its finial balls and pass 1 emeralds a 2-voxel radius against the
   Resolution section's 3 for round shapes. They render as plus signs and specks
3. Run 1's 1-voxel spindles and scroll tubes broke the rule that curved strokes
   need about 2 voxels across. The scrolls floated off in pass 1
4. Run 1 stopped while its review still showed gem specks and a pale, flat splat
5. Run 2 edited the model with a python heredoc although SKILL.md routes edits
   through the file-editing tool. The denial wasted a pass
6. Run 2 viewed no PNGs after passes 3 and 4 and only the hero and front after
   passes 2 and 6

Colors: Run 1's splat, spindles, and back rail read washed-out grey-tan against
the warmer oak. Gold swamps the oak on both crests. Run 2's walnut inlay shows
only as dark slivers that read as shadow.

### 2. Lantern

| Run | Verdict | Passes | Failed | Minutes | Voxels         |
| --- | ------- | ------ | ------ | ------- | -------------- |
| 1   | fair    | 4      | 0      | 1.9     | 22,160 at 5 mm |
| 2   | good    | 3      | 0      | 1.9     | 17,370 at 5 mm |

Both runs built an iron lantern at 5 mm from `boxFrame`, a pyramid roof, a torus
ring, and a gradient `roundCone` flame. Run 2 reads better because its flame
core blooms into a glow where run 1 dropped every flame strength to 1 and left
the glow to tinted glass. Unlike round 1, both runs re-aimed the hero with
`--view-orbit` to clear the corner post hiding the flame.

Failures:

1. Run 1, pass 1: 1 cm voxels left the flame at 14 cells and the wick at 1. The
   candle read unlit behind cool blue glass
2. Run 1, pass 4: dropping the flame strengths to 1 put every flame material
   below the bloom threshold. The session checked only the hero after that
   change
3. Run 2, pass 1: amber glass tinted the whole interior. The default hero put a
   corner post over the small flame
4. Run 2, pass 2: the flame overlapped the wick and left it 4 of 8 cells. The
   tip read pink until pass 3 lifted the flame

Lacked:

1. Both runs wanted a model-side light to cast candlelight on the frame and
   interior. Run 1 faked it with emissive glass where run 2 relied on flame
   bloom alone

Missed in the skill:

1. Run 1 dropped the flame to strength 1 although SKILL.md gives strengths of 2
   to 4 for a halo and a dark `baseColor` to keep the hue
2. Run 1 on passes 1, 2, and 4 and run 2 on pass 2 skipped renders that the
   skill asks for on every pass
3. Run 1 left the wax candle one flat material although every large face takes
   shades, noise, or a pattern
4. Run 1's `roundCone` flame sampled into a tiered stack of boxes at 6 voxels
   across. SKILL.md names `vesica` and `lathe` for flame silhouettes
5. Run 2's final glass reads as an opaque beige panel in the ortho views.
   SKILL.md suggests a darker rim for clear glass
6. Run 2's carved vents cut a square hole through the brass cap. Their step
   carries a no-op `rotate("y", 0)`

Colors: Both runs render the glass as one flat panel, peach in run 1 and beige
in run 2. Run 1's candle shaft is one flat cream under a matte pastel flame with
no halo. Run 2's near-white bloomed core leaves only a small orange tip.

### 3. Chest

| Run | Verdict | Passes | Failed | Minutes | Voxels         |
| --- | ------- | ------ | ------ | ------- | -------------- |
| 1   | good    | 4      | 0      | 2.5     | 91,744 at 1 cm |
| 2   | fair    | 2      | 0      | 5.4     | 96,933 at 1 cm |

Both runs built a 60 by 30 by 40 cm iron-bound chest at 1 cm with a barrel lid
pivoted as a part over an ellipsoid gold heap topped with gems. Run 1 reads
better because its rimmed coins and lumpy displaced heap deliver the gold where
run 2's richer red-lined mahogany chest holds a terraced gold dome, pale smudged
coins, and gem specks. Unlike round 1, neither run hit a `shades` range error.
Their coins lean cream or tan where round 1's library gold read olive.

Failures:

1. Run 1, pass 1: the coins rendered as cream blobs with gold patches until pass
   2 gave them a `goldDeep` rim and `goldLight` face
2. Run 1, passes 3 and 4: the session never viewed pass 3's PNGs and checked
   only the hero on pass 4
3. Run 2, pass 1: the lid end caps came out solid iron because the end bands
   filled them
4. Run 2, between passes 1 and 2: a python heredoc edit was denied with
   `This Bash command contains multiple operations`. The session redid the edits
   one at a time with Edit
5. Run 2, final pass: the session stopped without fixing the floating heap-coin
   voxel and blobby coin stacks it had named

Lacked:

1. Both runs wanted a scatter-on-surface operation for loose coins. Run 1
   hand-placed seven at heights from an ellipsoid formula that ignores the
   displacement noise
2. Run 2 wanted a seeded random operation. It wrote its own generator and
   heap-height function to place 28 coins

Missed in the skill:

1. Both runs skipped the box block-out and wrote the detailed model on pass 1
2. Both runs placed gems at a hand-computed heap height instead of using `coat`
   with sides `+y`
3. Run 1 viewed fewer than the four PNGs that the Checks section asks for on
   passes 2 through 4
4. Run 1 left the iron straps, rim, and corners one flat `mat.iron` although
   every large face takes shades, noise, or a pattern
5. Run 2 edited with a chained python heredoc although SKILL.md routes edits
   through the file-editing tool
6. Run 2 tilted its 1 cm coins with `orient` against the Resolution rules'
   warning of ribs and stair steps. The heap coins broke into pale smudges
7. Run 2 cut gems as octahedrons under 3 voxels in radius. The Resolution rules
   say that size reads as a plus sign or block
8. Run 2 stopped with a named flaw and a floating piece although a flaw named in
   review takes another pass

Colors: Run 1's iron frame, straps, and base read one flat gray. Its coin faces
lean pale cream rather than gold. Run 2's ground and stacked coins read pale tan
because their noise draws on the lighter shades. Its lid lining is one flat red.

### 4. Sword

| Run | Verdict | Passes | Failed | Minutes | Voxels        |
| --- | ------- | ------ | ------ | ------- | ------------- |
| 1   | fair    | 6      | 1      | 2.2     | 2,568 at 1 cm |
| 2   | good    | 6      | 0      | 2.3     | 9,868 at 5 mm |

Both runs built a longsword with a gold crossguard, a banded grip, and a disc
pommel set with a ruby. Run 2 reads better because its 5 mm voxels give a slim
beveled blade, an upswept guard, true gold, and a faceted ruby in a bezel where
run 1's 1 cm sword has a stair-stepped point, beige-blotched gold, and a
plus-sign gem table. Unlike round 1, neither run hit a `shades` range error. Run
2's faceted ruby replaces round 1's plus-shaped gems.

Failures:

1. Run 1, pass 3: the session edited the gem from the report's exposed count
   without viewing any PNG
2. Run 1, pass 4: a close-up that combined `--view-orbit` with `--view-look-at`
   failed because both set the view's transform. A `| tail -3` hid the failure
   from the exit code
3. Run 1, pass 5: one of two paired edits missed because an earlier edit had
   changed the gem line's scale to 1.75. The build died with
   `ReferenceError: rubyDark is not defined`
4. Run 1, final pass: the session viewed only the close, hero, and front
   renders. The beige gold blotches visible in the close-up never drew a review
   note
5. Run 2, pass 3: the octahedron gem sank behind its bezel with 8 exposed cells.
   Pass 5 fixed it with an extruded octagon intersected with an octahedron
6. Run 2, passes 3 and 4: `--view-select close 'sword'` framed mid-blade instead
   of the pommel until pass 5 switched to a positioned camera
7. Run 2, final pass: the session viewed only the hero. The close-up still crops
   the pommel's lower edge while framing mostly grip

Lacked:

None

Missed in the skill:

1. Run 1's raised ruby table has a 2-voxel `ngon` radius that the Resolution
   section says reads as a plus sign. The final close-up shows exactly that
2. Both runs skipped renders that the Checks section asks for on every pass. Run
   1 viewed none after pass 3 and run 2 viewed only the hero after pass 6
3. Run 1 combined `--view-orbit` with `--view-look-at` although the Other views
   section says `--view-orbit` excludes rotation flags
4. Both runs passed `--view-select` the glob `sword`. It matched the whole model
   rather than framing the pommel
5. Run 1 picked 1 cm voxels against the voxel-size step's rule that the smallest
   feature sets the size. The blade came out 2 voxels thick with a three-step
   point
6. Run 2 skipped the box block-out and wrote detailed shapes on pass 1
7. Run 2 finished with the camouflage noise on its blade unchanged although its
   pass 4 close-up showed it

Colors: Run 1's lightest gold shade reads beige. Its langet, guard, pommel, and
rim look like tan camouflage rather than gold. Run 1's blade is flat mid-gray
with only edge and fuller bands. Run 2's three steel shades at scale 0.06 break
the blade into gray camouflage patches that read as stone.

### 5. Tree

| Run | Verdict | Passes | Failed | Minutes | Voxels           |
| --- | ------- | ------ | ------ | ------- | ---------------- |
| 1   | fair    | 4      | 0      | 1.8     | 289,744 at 0.1 m |
| 2   | good    | 4      | 0      | 1.6     | 204,139 at 0.1 m |

Both runs built an oak at 0.1 m from a box block-out with a lathed trunk,
`roundCone` limbs, and a displaced clump crown. Run 2 reads better because its
hollowed crown makes a ragged canopy over low, level limbs in a green apart from
the meadow where run 1's even round puffs read as a cloud tree that blends into
the turf. Unlike round 1, both runs blocked out first. Both cleared the floaters
by retuning `displace` rather than carving or bridging single voxels.

Failures:

1. Both runs, pass 2: the crown's displacement and sunlit coat left run 1 with
   41 pieces and run 2 with 60 pieces of floating leaf voxels
2. Run 1, pass 2: the boughs read `0 kept` because the crown added after them
   overwrote them
3. Run 1, between passes 2 and 3: a python heredoc edit was denied with
   `This command requires approval`. The session redid it as three Edit calls
4. Run 1, pass 3: the new shadow coat left 9 floating pieces until pass 4
   lowered the displacement amplitude
5. Run 1, final pass: the boughs still read `0 exposed`. The session shipped
   them buried and said so
6. Run 2, pass 2: the library leaf and grass greens nearly matched. The custom
   meadow came out lime until pass 4 toned it down
7. Run 2, final pass: the session named a crown lobe hiding the trunk in the
   hero and stopped without fixing it

Lacked:

None

Missed in the skill:

1. Run 1 shipped its boughs buried at `0 exposed` instead of lengthening them
   past the crown or dropping them
2. Both runs skipped renders that the Checks section asks for on every pass. Run
   1 skipped some on passes 2 to 4 and run 2 viewed only the hero on its final
   pass
3. Run 1 kept the library leaf and grass greens close in hue and lightness
   against check 4. The top view shows the crown merging into the turf
4. Run 1 edited the model with a python heredoc although SKILL.md routes edits
   through the file-editing tool
5. Run 2 left a crown lobe hiding the trunk although Building step 4 keeps the
   hero corner clear. Lifting the lobe or re-aiming with `--view-orbit` would
   fix it

Colors: Run 1's crown blends into the turf from above because the leaf and turf
greens nearly match. Its puffs lack darker shading between lobes and read as
uniform cushions. Run 2's trunk reads as a near-black sliver in the crown's
shadow in the hero. Its moss coat is too small to read.

### 6. Well

| Run | Verdict | Passes | Failed | Minutes | Voxels           |
| --- | ------- | ------ | ------ | ------- | ---------------- |
| 1   | fair    | 2      | 0      | 1.4     | 65,376 at 2.5 cm |
| 2   | good    | 2      | 0      | 1.9     | 61,254 at 2.5 cm |

Both runs built a fieldstone well at 2.5 cm in two passes with oak posts, a
rope-wound windlass, a gabled roof, and a hooped bucket. Run 2 reads better
because turning its gable toward the hero shows the bucket on its rope where run
1's roof slope hides the rope. Unlike round 1, run 2 acted on the hero check by
turning the roof. Both runs worked at half round 1's 5 cm voxel.

Failures:

1. Both runs, pass 1: bucket parts floated free as separate pieces. Run 1's
   inner cut removed the bucket floor where run 2's 0.0125 m handle tube broke
2. Both runs, pass 1: the roof ran left to right with a slope facing the hero.
   Run 2 turned it in pass 2 where run 1 stopped with the roof hiding the rope
3. Run 2, final pass: the session named the hidden water and the wall striping
   in its final message and stopped without another pass

Lacked:

None

Missed in the skill:

1. Both runs skipped the box block-out and wrote the detailed model on the first
   write
2. Run 1 left the roof slope hiding the rope although Building step 4 keeps
   roofs from standing between the subject and the hero corner
3. Both runs ended on a flaw named in their final message although a named flaw
   takes another pass. Run 1 named the hidden rope and run 2 the hidden water
4. Run 1's windlass and ridge and run 2's ridge take grain built in place at y
   1.5 to 2.25. The skill says grain far from the origin reads as one broad ring
5. Run 2's axle takes bare `mat.oak` against the check that every face takes
   shades, noise, or a pattern

Colors: Both runs' oak posts read a bright uniform orange. Both curved stone
walls show vertical striping on their diagonal faces. Run 1's lightest shingle
shade reads salmon in the hero. Run 2's ridge beam reads as a flat pale tan
strip against the dark walnut roof.

### 7. Robot

| Run | Verdict | Passes | Failed | Minutes | Voxels          |
| --- | ------- | ------ | ------ | ------- | --------------- |
| 1   | good    | 2      | 0      | 1.2     | 5,702 at 2.5 cm |
| 2   | good    | 2      | 0      | 1.8     | 9,090 at 2.5 cm |

Both runs built a boxy robot at 2.5 cm with the head and arms as parts on neck
and shoulder pivots. Run 2 reads better because it rendered a posed copy that
shows the head and arms turning cleanly about their joints. In round 1 neither
run posed a part.

Failures:

1. Both runs, pass 1: a two-voxel ear cylinder rendered as a plus sign. Pass 2
   swapped in box ears
2. Run 2, pass 1: the half-voxel antenna cylinder wrote 0 cells. Its tip floated
   as a second piece until pass 2
3. Run 2, before pass 1: a perl rename of the `paint` material const chained
   with grep was denied by permissions. Two Edit calls redid it

Lacked:

1. Run 1: a review render of a part turned about its pivot. The joints went
   unchecked
2. Run 2: a SKILL.md note on posing a part for review. The session found
   `vxl node set rotation` through `--help`

Missed in the skill:

1. Both runs: the first draft broke the Resolution rules with cylinders and
   spheres under 3 voxels in radius. Run 1 fixed only the ears and left the neck
   and shoulders
2. Both runs: the chest plate and visor keep one flat color against the check
   that every large face takes shades, noise or a pattern. Run 1's gold belt
   does too
3. Run 1: the session claimed the arms also turn about z without testing it.
   From its pivot at x = 0.25 the shoulder cylinder would swing into the torso
4. Run 2: the session chained a perl rename with grep though SKILL.md routes
   edits through the file-editing tool

Colors: Both runs leave the chest plate and visor flat dark trim. Run 1's flat
gold belt reads most unfinished. Run 2's near-black shoulders, knees and claws
merge into the dark pelvis.

### 8. Cottage

| Run | Verdict | Passes | Failed | Minutes | Voxels          |
| --- | ------- | ------ | ------ | ------- | --------------- |
| 1   | good    | 3      | 0      | 2.8     | 250,706 at 5 cm |
| 2   | good    | 2      | 0      | 2.4     | 183,968 at 5 cm |

Both runs built a red domed mushroom cottage at 5 cm with a round planked door,
round glowing windows and a chimney. Run 1 reads better because its door faces
the hero camera in a stone ring under crisp painted spots. Run 2's cap overhang
half-hides its door in the hero, and its raised spots come out as lumpy warts.
Run 1 opened with the box block-out that both round 1 runs skipped.

Failures:

1. Run 1, before pass 2: the draft left a placeholder
   `halfSpace("-y", 0.1).offset(0)` on the door that would have deleted it. The
   session caught it before building
2. Run 1, pass 3: the attic window frame stood proud of the sloping cap as a
   stuck-on disc in the right view. The session shipped it
3. Run 2, pass 2: the `--view-orbit door 20 12 4.5` close-up aimed at the model
   center and showed mostly cap. The session did not retry
4. Run 2, final pass: the session named the cap overhang hiding the door in the
   hero but stopped without a fix

Lacked:

1. Both runs: a way to seat a feature on a lathe or ellipsoid surface. Each
   session computed the cap's surface points by hand to place the spots

Missed in the skill:

1. Run 2: the session skipped the box block-out of SKILL.md step 3 and wrote the
   detailed model first
2. Run 2: the door close-up framed the whole model because the session skipped
   `--view-select`
3. Run 2: the session ended after naming the hidden door and the failed
   close-up. SKILL.md says a named flaw takes another pass

Colors: Run 1's stem noise reads as vertical streaking rather than plaster. Run
2's slate doorstep and chimney cap carry no pattern. The doorstep reads as a
gray puddle in the hero.

### 9. Potion

| Run | Verdict | Passes | Failed | Minutes | Voxels         |
| --- | ------- | ------ | ------ | ------- | -------------- |
| 1   | fair    | 3      | 0      | 1.9     | 11,200 at 5 mm |
| 2   | fair    | 7      | 0      | 2.6     | 8,876 at 5 mm  |

Both runs built a corked round flask of red potion at 5 mm with an unrequested
twine collar. In both heroes the neck glass nearly vanishes and leaves the cork
and collars stacked like floating washers. Run 2 reads better because its potion
holds a truer red than run 1's pink-coral. Neither run kept the visible neck of
round 1's run 2.

Failures:

1. Run 1, every pass: the bubbles step reported 6 kept and 0 exposed. The
   bubbles stayed buried in the potion unnoticed
2. Run 1, pass 2: clearing the glass to near white removed the neck's outline.
   The hero shows the cork and collars floating
3. Run 2, pass 4: an unscoped `coat("glass skin", glass)` recolored everything
   and dropped the glass to 0 exposed. Pass 5 scoped it with `within`
4. Run 2, pass 6: the first glint sat too large and off the hero face. Pass 7
   shrank it to a still opaque white squiggle
5. Run 2, final message: the session calls the red bumps at the belly potion
   behind glass. The front view shows the potion filled to `outer.offset(-v)`
   notching through a one-voxel wall
6. Both runs, after pass 1: a python heredoc edit was denied by permissions. The
   sessions fell back on Edit and Write

Lacked:

1. Run 2: a per-step count of cells exposed through a transparent neighbor. The
   session could not tell whether the potion reached the outer surface at the
   belly

Missed in the skill:

1. Both runs: passes skipped views against the check for all four PNGs on every
   pass. Run 2's pass 4 read no renders at all
2. Both runs: the session tried a python command for an edit though SKILL.md
   routes edits through the file-editing tool
3. Run 1: check 3 flags the bubbles' 0 exposed as buried. The bubbles stayed
   invisible on every pass
4. Run 2: pass 4 wrote an unscoped `coat` though Choosing operations item 6
   warns that a coat recolors a neighbor's live cells within its reach

Colors: Run 1's potion reads pink-coral in the hero and top views. Run 2's reads
pink only from the top. Run 1's blue lip and base rim read as opaque steel-blue
plastic. Run 2's glint reads as flat painted white beside a pale twine band with
little contrast.

### 10. Cart

| Run | Verdict | Passes | Failed | Minutes | Voxels          |
| --- | ------- | ------ | ------ | ------- | --------------- |
| 1   | good    | 2      | 0      | 1.3     | 15,544 at 2 cm  |
| 2   | good    | 4      | 0      | 2.3     | 8,512 at 2.5 cm |

Run 1 built an oak hand wagon at 2 cm on four eight-spoke iron-tired wheels with
a T-handled tongue. Run 2 built a stake-sided farm cart at 2.5 cm with bolted
plank walls, twin shafts and each wheel a part turning on its axle. Run 2 reads
better for its richer construction, though run 1's wheels read cleaner and more
solid. Both runs read good where round 1's run 2 jammed its wheels against
straps and side walls.

Failures:

1. Run 2, between passes 3 and 4: a `sed -i` edit switching the wall grain
   palette was denied by permissions. The session redid it with Edit
   `replace_all`

Lacked:

None

Missed in the skill:

1. Both runs: passes skipped views against the check for all four PNGs on every
   pass. Run 1's pass 2 checked only the hero
2. Run 1: the session skipped the block-out and wrote the detailed model on pass
   1
3. Run 1: the hub cap and axle are cylinders under 2 voxels in radius. The
   Resolution rules say under about 3 reads as a rod or block
4. Run 1: all four wheels come from one mirrored step and repeat one pattern
   exactly. The skill gives a distinct look per copy its own step and seed
5. Run 2: passes 3 and 4 ran with `--report false` though the checks put the
   report first on every pass
6. Run 2: the hub band paint covers all 56 hub cells. The wood hub reads as a
   gray iron block

Colors: Both runs leave the iron flat gray and the walnut pink-brown. Run 1's
crossbar and bolster ends show pale blotches. Run 2's noise-patterned tire and
felloe read speckled rather than as clean iron and wood.

### 11. Guitar

| Run | Verdict | Passes | Failed | Minutes | Voxels          |
| --- | ------- | ------ | ------ | ------- | --------------- |
| 1   | good    | 2      | 0      | 5.0     | 228,123 at 4 mm |
| 2   | good    | 3      | 0      | 3.9     | 227,127 at 4 mm |

Both runs built a one-piece steel-string acoustic at 4 mm with six one-voxel
strings over an abalone rosette and an ebony fretboard. Run 2 reads better
because its strings fan past the nut to three tuners a side. Run 1 runs its
strings straight to one center column of posts. Round 1 hollowed both bodies
with a negative offset, but round 2 left both solid with a dark well under the
sound hole.

Failures:

1. Run 1, between passes 1 and 2: a python heredoc edit chained with grep was
   denied by permissions. Three Edit calls redid it
2. Run 2, pass 2: `bands` at a one-voxel period replaced the spruce `grain`. The
   top reads as ribbed plastic

Lacked:

None

Missed in the skill:

1. Both runs: the body stays solid with a dark-painted cylinder under the sound
   hole. `shell`, a 2D `offset` before `extrude`, or an inset carve would have
   hollowed it
2. Both runs: neither session re-aimed the hero with `--view-orbit` though it
   stacks the six strings into one strip. Only run 2 added a close-up
3. Both runs: passes skipped views against the check for all four PNGs on every
   pass. Run 2's pass 3 read only the close-up
4. Run 1: the headstock strings stay parallel because slanted one-voxel lines
   break apart. Densely sampled `set` points or a 2-voxel stroke would have
   fanned them to side posts
5. Run 1: the session chained a python heredoc edit with grep though the skill
   warns that such a command needs its own approval

Colors: Both runs leave the ebony fretboard and headstock face flat near-black.
Run 2's spruce top reads as uniform pinstripes rather than wood beside a flat
cream binding.

### 12. Candelabra

| Run | Verdict | Passes | Failed | Minutes | Voxels         |
| --- | ------- | ------ | ------ | ------- | -------------- |
| 1   | good    | 3      | 0      | 2.5     | 17,614 at 5 mm |
| 2   | good    | 4      | 0      | 3.4     | 35,965 at 5 mm |

Both runs built a lathed foot, a twisted square stem with lighter painted corner
ridges and stroked 2D scroll arms at 5 mm. Run 1 reads better against the prompt
because its five arms sit 72 degrees apart and its twist reads as broad stripes.
Run 2 draws bolder scrolls on four axis-aligned arms with the fifth candle on
the stem. Both round 1 runs set every candle in one plane with `mirror('x')`.

Failures:

1. Run 1, pass 1: a 16-voxel flame floated as a second piece. Pass 2 enlarged
   the wick to join it
2. Both runs, after pass 1: a python3 heredoc edit was denied by permissions.
   Both sessions redid it with Edit calls

Lacked:

None

Missed in the skill:

1. Both runs: passes skipped views against the check for all four PNGs on every
   pass. Run 2's passes 1, 3 and 4 each read two views
2. Run 1: the scrolls come from 2D `arc`s of 2.5-cell radius in place of the
   `bend` the skill steers curled tips to. The volutes fall under the 3-voxel
   floor and collapse into blobs
3. Run 2: four arms from `repeatPolar("y", 4)` plus a stem candle sidestep the
   prompt's five candles on arms. The off-axis sampling went untested

Colors: Both runs' iron reads uniformly dark apart from the painted stem ridges.
Run 1's tiny flames show their gradient as two or three flat blocks. Run 2's wax
drips take one shade of the candle wax and barely show.

### 13. Knight

| Run | Verdict | Passes | Failed | Minutes | Voxels           |
| --- | ------- | ------ | ------ | ------- | ---------------- |
| 1   | good    | 2      | 0      | 3.5     | 1,968 at 6.25 cm |
| 2   | fair    | 4      | 0      | 4.0     | 1,388 at 6.25 cm |

Both runs built a 32-voxel knight at 6.25 cm with a raised sword, a great helm,
and only PICO-8 colors. Run 1 reads better because its shield shares the
tabard's heraldry and its head, arms, legs, and sword ride parts with pivots.
Run 2's front view merges three yellow elements into one mass. Neither run
drifts off the palette as round 1's metallic run 1 did.

Failures:

1. Run 1, pass 2: a `grep` for hex colors in the voxj found nothing because the
   file stores colors in linear light. The session read the value pool instead
2. Run 1, final pass: the shield faces +x and shows edge-on as a grey strip in
   the front view
3. Run 2, pass 2: the skirt step kept 0 cells and the torso 40 because the later
   tabard paint covered the same box. The session left the dead step and never
   noticed the breastplate had vanished
4. Run 2, passes 2 and 3: the session added a no-op `.translate([0, 0, 0])` to
   the cross, then removed it
5. Run 2, final pass: the session piped the report through `head -1` and opened
   only the hero PNG. The yellow-on-yellow front went unreviewed

Lacked:

None

Missed in the skill:

1. Neither run patterned the chest sides, greaves, or helm sides. Check 3 asks
   every large face for noise or a pattern, and both accept palette colors
2. Run 1's one body pattern, a mail checker on the hips, sits almost wholly
   under the tabard and belt
3. Run 2's yellow cross, hem, and shield rim touch in the front view. Check 4
   asks neighboring materials for colors far apart
4. Run 2 reviewed only the hero on its last pass although the skill asks for all
   four PNGs every pass
5. Run 2 built a game character as one object although SKILL.md documents `part`
   with a pivot for limbs

Colors: The chest sides, greaves, and helm sides stay one flat `#C2C3C7` in both
runs. With the white coat on the top faces alone, the armor reads as matte grey
plastic rather than plate. Run 2's tabard front stays flat red under its yellow
trim. Its shield face is one flat `#29ADFF`.

### 14. Pocket watch

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | fair    | 5      | 0      | 3.8     | 82,261 at 0.5 mm  |
| 2   | good    | 4      | 0      | 5.4     | 112,644 at 0.5 mm |

Both runs wrote a full skeleton movement at 0.5 mm on the first pass and spent
later passes on the chain, hands, and metals. Run 2 reads better because its
gears stand apart and its fused chain still looks like separate links. Run 1's
gold and brass wheels merge into one yellow mass under crossing bridges.

Failures:

1. Run 1, pass 2, and run 2, pass 3: the report gave a correct chain of one
   piece per interlocked link, 26 and 16 pieces. Both sessions read it as a
   defect
2. Run 1, pass 3: the session cut the pitch from 0.0038 to 0.0033 to reach 1
   piece. Every link fused into its neighbors and crowds the chain in the front
   view
3. Run 1, passes 3 and 4: the session tried noise shades on the case and
   two-tone links, then reverted the case to flat `mat.gold`
4. Run 1, between passes 1 and 2: a python heredoc edit was denied with
   `This command requires approval`. The session redid it as six Edit calls
5. Run 2, after pass 1: a backup copy to `/tmp` was denied because the path sits
   outside the working folder
6. Run 2, pass 4: the session raised the pitch from 3.5 to 4 mm until each
   link's wire touched the next. The links fuse at their contact cells
7. Run 2, final pass: the dark navy hands still vanish against the dark plate

Lacked:

1. Both runs wanted the piece lines to tell a piece caught inside another from
   one that floats. Without it a correct interlocked chain looks like a failure

Missed in the skill:

1. Both runs followed check 4's "A second piece means a shape floats" and fused
   a correct chain without asking whether the pieces interlocked
2. Both runs skipped the box block-out pass and wrote the full model first
3. Both runs tried a write the skill says needs its own approval. Run 1 chained
   an edit through python, while run 2 copied a backup to `/tmp`
4. Run 1 reverted the large gold case to one flat material although check 3 asks
   every large face for shades, noise, or a pattern
5. Run 1 set the gold center wheel against brass wheels although check 4 warns
   that two metals side by side merge
6. Run 2 left `#1E3C9A` hands over a `#3A3E46` plate although check 4 asks
   neighboring materials for colors far apart
7. Run 2 saw the hero frame the whole chain and shrink the watch but never
   re-aimed it with `--view-orbit` or `--view-select`

Colors: Both runs' gold case, bezel, and chain read olive in the front and right
views. Neither session named the cast or warmed the gold with a custom material.
Run 1's gold center wheel merges with the brass beside it. Run 2's case noise
leaves pale blotches, while its navy hands merge with the dark plate.

### 15. Dragon

| Run | Verdict | Passes | Failed | Minutes | Voxels          |
| --- | ------- | ------ | ------ | ------- | --------------- |
| 1   | fair    | 7      | 0      | 8.8     | 78,209 at 3 cm  |
| 2   | fair    | 8      | 1      | 9.9     | 286,356 at 2 cm |

Both runs coiled a green dragon around a gold hoard from a hand-made polar spine
of chained `roundCone` segments. Run 2 edges ahead because its thicker coil
carries a cream belly, its gold reads as gold, and its right view shows the only
readable dragon head. Run 1 has the more legible ribbed wings and sleeping face,
but its gold reads as sand and its wings stand as a tall tent. Neither run lost
a pass to the `shades` range errors behind four of round 1's failed passes.

Failures:

1. Run 1, pass 4: after a render with `--file-stem dragon-close`, the session
   tried to read `dragon-close-close.png` and got `File does not exist`. It
   listed the folder to find the file
2. Run 1, pass 5: the tail spade kept 6 of 32 cells and the claws 2 of 14
   because the later back coat and head steps took their cells. Pass 6 reordered
   the steps
3. Run 1, passes 1 to 6: stray single voxels from the displaced pile and a wing
   slab corner floated until hardcoded carve boxes removed them
4. Run 2, pass 3: the eyelids painted 1 cell. The closed eyes stayed hidden
   until the pass 5 head rebuild
5. Run 2, pass 4: an Edit replacing the head failed with
   `String to replace not found` while the next Edit referencing `mouth`
   applied. The build failed with `ReferenceError: mouth is not defined`
6. Run 2, pass 8: the pass line ran voxelize twice to feed `head` and `grep`.
   That doubled the pass time
7. Run 2, final pass: 8 loose glint voxels stayed as extra pieces. The session
   called them stray coins

Lacked:

1. Both runs wanted a sweep or tube along a path. Run 1 interpolated a polar
   spine table linearly, while run 2 wrote a Catmull-Rom spline. Both chained
   `roundCone` segments along it
2. Both runs wanted a way to drop small detached pieces. Run 1 carved strays
   with boxes copied from the report's piece bounds, while run 2 left 8 floating
   voxels
3. Both runs wanted the render command to print its output paths. Each listed
   the folder to find its close-up

Missed in the skill:

1. Neither run used the `bend` the skill names for horns and curled tips. Horns
   and tail tips stay straight `roundCone` segments
2. Run 1 used `smoothUnion` only inside the head, while run 2 never used it. In
   both the legs and head sit on the coil as separate lumps
3. Run 1 built the wings from flat slabs set at angles. The resolution rules
   warn that such slabs sample into ribs and stair steps
4. Neither run fixed its floating pieces at the cause, although the checks say a
   second piece means a shape floats. Run 1 carved them away, while run 2 left 8
   in the final model
5. Run 2 piped the voxelize report through `head` and `grep` although the skill
   runs the pass line on its own

Colors: Run 1's gold pile reads as tan sand with few bright accents. Its dark
purple wing bones and back spikes merge into the membrane in shadow. Run 2's
rust membrane, brown bones and horns, and mahogany chest sit close in hue and
lightness. Its gold speckle at density 0.75 reads as confetti more than coins.

### 16. Bridge

| Run | Verdict | Passes | Failed | Minutes | Voxels              |
| --- | ------- | ------ | ------ | ------- | ------------------- |
| 1   | good    | 4      | 0      | 3.5     | 555,774 at 0.5 m    |
| 2   | good    | 3      | 0      | 3.7     | 1,091,873 at 0.25 m |

Both runs sampled a computed parabola every 1 m into capsule chains between
stone towers and sized looped one-voxel hangers to the curve. Run 1 reads more
correct because its revisions lifted the midspan cable off the deck and
unblocked the road. Run 2's finer 0.25 m grid adds cables splaying out to four
anchor blocks, but its midspan cable rests on the railing.

Failures:

1. Run 1, pass 1: `cableLow` 5.25 put the midspan cable down among the deck and
   railings. Pass 3 raised it to 8.25
2. Run 1, passes 1 to 3: solid anchorage blocks sat across the road. Pass 4
   carved arched gateways through them
3. Run 2, pass 2: displaced tree crowns left 17 floating fragments. Pass 3
   lowered the displace amplitude
4. Run 2, final pass: `sagY` 7 with a 0.35 m cable radius puts the cable bottom
   at 6.65 m, below the 6.75 m rail top. The session never noticed the cable
   merging into the railing at midspan
5. Run 2: a python heredoc edit was denied. The session redid it as separate
   Edit calls

Lacked:

1. Both runs wanted a 3D curve, sweep, or path primitive to draw a cable through
   computed points. Each chained 1 m capsules between samples of its curve
2. Run 2 wanted the report to give clearance or contact between steps. Nothing
   flagged the cable fusing with the rails

Missed in the skill:

1. Neither run rendered a close-up with `--view-orbit` or `--view-select`. One
   of the midspan would have shown run 2's cable resting on the railing
2. Run 2 left the large asphalt deck one flat material although check 3 asks
   every large face for shades, noise, or a pattern

Colors: Both runs render the water one flat blue in the front view. Run 1's red
girder and iron rails are single flat colors, and its grass coat draws green
stripes down the stepped left bank. Run 2's asphalt deck is one flat dark grey,
while its grass noise swings so far in lightness that the banks read as
camouflage.

### 17. Chess set

| Run | Verdict | Passes | Failed | Minutes | Voxels               |
| --- | ------- | ------ | ------ | ------- | -------------------- |
| 1   | fair    | 5      | 0      | 3.6     | 319,992 at 2.5 mm    |
| 2   | good    | 3      | 0      | 4.8     | 2,355,412 at 1.25 mm |

Both runs placed all 32 pieces from five lathed kinds and an extruded knight in
a correct opening layout with a1 dark and queens on their color. Run 2 reads
better because it halved the voxel size to 1.25 mm after one pass and tapered
the knight into a horse head. Run 1 stayed at 2.5 mm with ridged pieces and slab
knights. Unlike round 1, both runs build 12 shared parts, one per kind and
color, and place them by offset.

Failures:

1. Run 1, pass 1: the queen's `dish` carve wrote 0 cells. The queen orbs floated
   as two extra pieces
2. Run 1, passes 1 to 4: the orbs floated until pass 5 stemmed them with a box
3. Run 1, between passes 1 and 2: `sed -n 1,400p /dev/null; grep ...` was denied
   as an edit outside the working folder
4. Run 2, before pass 2: a `mkdir /tmp/chess && sips` crop of the top view was
   denied for writing outside the working folder
5. Run 2, before pass 2: two `--select` renders with `--file-stem chess-close`
   both wrote `chess-close.png`. The session rendered the knight again after the
   bishop close-up overwrote it

Lacked:

1. Run 2 wanted a zoomed crop of an existing view to confirm the a1 square's
   color. It rendered `--view-select` close-ups instead

Missed in the skill:

1. Both runs skipped the box block-out pass and wrote full lathe profiles from
   the start
2. Neither run reviewed all four views on its final pass, although the skill
   asks for all four PNGs every pass. Run 1 read hero and front, while run 2
   read hero and a knight close-up
3. Run 1's passes 2 to 4 read only close-ups of the knight, king, and queen
4. Run 1 kept 2.5 mm voxels under a 1-voxel tilted bishop slit and 3 mm queen
   points. The resolution rules ask 2 voxels for tilted strokes and warn that
   spheres under about 3 voxels read as blocks
5. Run 2 expected `chess-close-knight.png` although the skill's `--select`
   example writes `robot-close.png` alone from `--file-stem robot-close`
6. Run 2 tried to crop a PNG under `/tmp` although the skill keeps scratch work
   beside the model file

Colors: Both runs pair ivory with a pale maple, `#F2EAD3` on `#E3BE85` and
`#F0E7D3` on `#E2C08C`. White pieces separate from the light squares only
through shading and a slight hue shift. Both runs' black pieces read close to
flat dark. Run 1's ivory shows dark streaks where its noise meets the stepped
lathe surfaces, while run 2's dark-square grain draws pale vertical stripes.

### 18. Ramen stand

| Run | Verdict | Passes | Failed | Minutes | Voxels          |
| --- | ------- | ------ | ------ | ------- | --------------- |
| 1   | good    | 3      | 0      | 4.0     | 100,192 at 4 cm |
| 2   | good    | 4      | 0      | 4.4     | 468,359 at 2 cm |

Both runs put a pink RAMEN sign in a cyan border on the roof and a yellow
katakana blade sign on the side. Run 2 reads better because its 2 cm grid gives
thinner strokes and room for a chef, screens, a plank counter, and street
clutter. Run 1's chunkier 4 cm stall loses its bowls and interior in the dark.
Both runs drew the letters from strokes where round 1 hand-wrote 5x7 bitmap
fonts.

Failures:

1. Run 1, pass 1: the blade sign rim took swapped rect coordinates. It landed on
   the ground as a stray cyan frame and stretched the bounds to 96x95
2. Run 1, pass 1: both puddle paints wrote 0 cells because a 0.02 m ellipsoid
   missed every cell center. An extruded ellipse replaced it
3. Run 2, pass 1: two steam spheres and the satellite dish floated as extra
   pieces
4. Run 1, pass 2: the steam rebuild left a floating piece until pass 3 lowered
   one capsule
5. Run 1, pass 2, and run 2, pass 3: the session edited from the report without
   opening any PNG
6. Run 2, pass 3: subtracting an offset sphere from the reoriented dish left
   three floating specks of 1 to 2 voxels. Pass 4 rebuilt it as a shelled cap
7. Run 2, final pass: the session read only the hero and front PNGs before
   calling the model done
8. Both runs, final pass: the blade sign's last katakana, n, reads as so

Lacked:

1. Both runs wanted a text or glyph operation. Run 1 drew RAMEN from `rect` and
   `polyline` strokes on a letter grid, while run 2 fed per-letter stroke lists
   to `polyline`

Missed in the skill:

1. Both runs skipped the PNG review on at least one pass, although the checks
   require all four views every pass
2. Both runs wrote the full scene on pass 1 instead of the box block-out from
   workflow step 3
3. Both runs built a thin feature the resolution rules warn against. Run 1's
   puddle ellipsoid sat thinner than a voxel, while run 2's tilted dish from a
   thin sphere subtraction broke into specks over two passes
4. Both runs left large faces flat against check 3. Run 1's cabinet and back
   wall stay near-flat dark panels, while run 2's counter top and side walls
   take single colors

Colors: Both runs' steam renders as a flat opaque grey slab against near-black
back walls. Run 1's black bowls vanish against the dark counter. Run 2's steel
counter top is one flat light grey, and its three pinks on the sign, blade
border, and menu screen sit close in hue.

### 19. Dungeon kit

| Run | Verdict | Passes | Failed | Minutes | Voxels             |
| --- | ------- | ------ | ------ | ------- | ------------------ |
| 1   | good    | 2      | 0      | 3.7     | 338,276 at 5 cm    |
| 2   | good    | 2      | 0      | 4.0     | 172,576 at 6.25 cm |

Both runs built a four-tile kit of floor, wall, corner, and arched doorway, then
assembled a one-piece room from it. Run 2's 3x3 room reads better because its
tiles stay inside their cells while the brick bond, plinth, and coping run
across the seams. Run 1's 4x3 room adds a corridor tile past the door, but its
keystone pokes 0.1 m off the grid. Run 2 shares 9 objects across its room
through `--frame local`. Neither round 1 run used the flag.

Failures:

1. Both runs, pass 2: permissions denied a python3 walk of `room.voxj` for the
   walkable values. Run 1 fell back to grep, while run 2 needed a narrower grep
   after a second denial
2. Run 2, start: the sandbox blocked listing the working directory's parent

Lacked:

1. Rotating a placed part: `part` takes only a pivot and an offset. Both runs
   rotated shapes inside the tile functions into copies such as `wall.r90` and
   `doorway.r180`
2. Material properties in the report or renders: checking `walkable` took grep
   on raw `room.voxj`. Run 1 found the pool indices but not the values and
   hedged
3. Per-material values from `vxl vox-doc show`: the command lists only the
   property name `walkable`. `SKILL.md` never mentions it

Missed in the skill:

1. Run 1 voxelized the room without the `--frame local` that `SKILL.md` gives a
   kit of repeated parts. Its room holds 23 objects, 12 of them floor copies
2. Both runs exported tiles as functions. `SKILL.md` says a function export
   names nothing. Neither `kit.sdfj` can serve as a tile library because run 1's
   names only `floorTile` and run 2's names no tile

Colors: Run 1's dark mortar bed reads as a flat black band along the room's base
in the front and right views. Run 2's alternating light and dark flags read
close to a checkerboard.

### 20. Crane

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | fair    | 4      | 1      | 4.3     | 29,530 at 0.125 m |
| 2   | good    | 2      | 0      | 2.5     | 38,944 at 0.1 m   |

Both runs built a crawler crane as a four-level part tree of crane, cab, boom,
and hook with every pivot on its hinge. Run 2 reads better in half the passes
because its four-chord tapered truss hangs from an A-frame gantry by pendants.
Run 1's side-laced boom stands propped on its foot pin with nothing holding it
up. At 0.1 and 0.125 m, both round 2 booms held together where both round 1
booms shattered under the tilt.

Failures:

1. Run 1, before pass 1: the first Write held a nonsense
   `box(...).intersect ? box(...) : box(...)` ternary that an Edit removed
   before the build
2. Run 1, pass 2: permissions refused a python heredoc edit chained with the
   rebuild. The session redid the edits with the Edit tool
3. Run 1, after pass 4: the session read a `crane-hook-close.png` that did not
   exist because `--file-stem` wrote `crane-hook.png`. Its first hook close-up
   at orbit distance 6 needed a second render from a placed camera

Lacked:

1. Posing a part's node in a render or report: neither session saw the cab
   turned or the boom tilted
2. A cable spanning two parts: run 1 left out the boom pendants. Run 2 kept them
   in the boom part, where their lower ends leave the gantry on a large tilt
3. A hook joint that hangs plumb: both runs told the user to counter-rotate the
   hook node by the boom angle
4. Each part's pivot in the report: neither run could check the hook pivot
   against the sheave center
5. Clearance or detachment at a turned pose: joint behavior beyond the default
   pose went unchecked in both runs

Missed in the skill:

1. Both runs skipped renders that `SKILL.md` asks for on every pass. Run 1 read
   none after pass 3 and only the hero after pass 4, while run 2's final pass
   skipped the right and top views
2. Run 1's yellow noise at scale 0.6 leaves pale blotches on the house roof and
   cab that the review never named. `SKILL.md` says a large face takes shades or
   noise without reading as stains

Colors: Run 1's deck, cab roof, and wheels are a flat charcoal or black. Its
yellow noise reads as pale stains on the house roof and cab. Run 2's track belts
read nearly flat black under a flat gray counterweight top and cab roof. Its
gray hook washes out against the white background.

### 21. Fish tank

| Run | Verdict | Passes | Failed | Minutes | Voxels          |
| --- | ------- | ------ | ------ | ------- | --------------- |
| 1   | good    | 3      | 0      | 3.2     | 154,610 at 5 mm |
| 2   | fair    | 4      | 0      | 2.9     | 26,044 at 1 cm  |

Both runs built a tank of gravel, seaweed, a castle, fish, and bubbles without a
water volume. Run 1 reads better because its 5 mm cobblestone castle, five fish,
and shaded seaweed outclass run 2's blocky 1 cm fish muted by the glass tint.
Run 1's one-voxel water-surface sheet caps the hero and fills the top view.
Round 1 favored the dry tank, while round 2 favors run 1's 5 mm detail over a
dry 1 cm tank.

Failures:

1. Both runs, an early pass: permissions denied a python heredoc edit. Each
   session redid it with the Edit tool
2. Run 1, pass 1: solid water buried the fish, castle roofs, seaweed, and
   bubbles at `0 exposed`. The interior rendered dark blue-black
3. Run 1, pass 1: an unbounded clownfish stripe paint laid white bands down the
   glass, frame, water, and gravel
4. Run 1, pass 2: the tail polygons left 4-voxel tip fragments as separate
   pieces. Pass 3 reattached them with a 2D offset
5. Run 1, final pass: a lone gravel voxel at `[0.095, 0.06, -0.025]` has floated
   as its own piece since pass 2
6. Run 2, pass 1: fins and tail tips broke off as pieces of 2 to 22 voxels.
   Passes 3 and 4 reattached them by widening the tail root
7. Run 2, pass 2: the session read no renders and edited from the report alone
8. Run 2, final pass: the glass tint left the castle door barely legible and the
   fish desaturated. The session reported it rather than fixing it

Lacked:

1. Whether buried details sit inside a transparent material: run 1's pass 1
   report gave only `0 exposed`. The session traced the fault to the water from
   the dark render

Missed in the skill:

1. Both runs skipped renders that `SKILL.md` asks for on every pass. Run 1 never
   fixed its all-blue top view, while run 2 read no render on pass 2
2. Both runs placed unbounded paints although `SKILL.md` says a paint recolors
   any live cell in its reach. Run 1's stripes crossed the whole tank, while run
   2's shadow boxes hit only castle stone
3. Neither run tried the `#RRGGBBAA` alpha `baseColor` that `SKILL.md`
   documents. Both glasses used transmission. The skill warns that transmission
   tints what sits behind it. Run 2's red fish went mauve
4. Run 1 never chased its lone gravel piece although the checks read a second
   piece as a floating shape
5. Run 2's 2.5-voxel corner towers and small roofs came out as stubby blocks at
   1 cm. The Resolution notes say cylinders and cones under about 3 voxels read
   as rods or blocks

Colors: The glass tint washes both interiors toward pale blue in the front and
right views. Run 2's clownfish turns brown, its red fish mauve, and its yellow
fish olive. Run 1's flat light-blue water-surface sheet fills the whole top
view. Run 2's stand is one flat black.

### 22. Wizard tower

| Run | Verdict | Passes | Failed | Minutes | Voxels           |
| --- | ------- | ------ | ------ | ------- | ---------------- |
| 1   | good    | 6      | 0      | 4.6     | 245,560 at 0.1 m |
| 2   | good    | 3      | 0      | 3.2     | 134,006 at 0.1 m |

Both runs built a two-turn spiral of rotated, raised box treads around a stone
tower under a bent purple hat roof, each in one piece at 0.1 m. Run 1 reads as
the more finished tower because of its continuous capsule handrail and railed
landing, though its roof crook stays mild. Run 2 has the stronger drooping roof,
but its box rail climbs in a sawtooth and the droop points away from the hero.
Where both round 1 runs dropped `bend` for the roof, both round 2 runs kept it
by bending only the cone's upper half.

Failures:

1. Run 1, pass 2: the first lean-plus-bend roof creased on its inner side. Pass
   3 bent only the upper cone above y = 16
2. Run 1, pass 3: a two-voxel corbel fragment floated beside the top doorway.
   Pass 4 subtracted the doorway cut from the corbel ring
3. Run 1, passes 4 to 6: the lean lifted the bent tip off the skirt and left a
   ledge at the joint. The final message admits the ledge still shows in the
   right view
4. Run 2, pass 2: the star finial floated as a 52-voxel second piece. The lowest
   step brackets also reached below the ground. Pass 3 fixed both
5. Run 2, pass 3: the session declared done after checking only the hero and
   front. The sawtooth rail and sliver-thin star went unreviewed in the right
   and top views

Lacked:

None

Missed in the skill:

1. Both runs read only the hero and front on some passes although `SKILL.md`
   asks for all four PNGs every pass. Run 1 did so on passes 2 to 4, run 2 on
   passes 1 and 3
2. Run 1 skipped the box block-out the skill prescribes and wrote the full
   model, materials included, in pass 1
3. Run 1's slate treads and iron rails sit close in value to the dark stone. The
   skill advises colors far apart in lightness or hue for neighboring materials
4. Run 2 built its rail from flat boxes per step instead of sloped `capsule`
   segments between post tops
5. Run 2 bent the roof toward -x, away from the hero corner, although the skill
   says the subject faces +z or +x

Colors: Run 1's slate treads and iron rail sit close in value to the
dark-mortared stone wall. The stair reads gray on gray in the hero.

### 23. Log cabin

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | good    | 2      | 0      | 2.9     | 1,012,760 at 5 cm |
| 2   | good    | 6      | 2      | 5.4     | 1,361,336 at 5 cm |

Both runs built a round-log cabin at 5 cm with thick added roof snow, custom
light-blue icicles, and a smoke plume rising from the flue. Run 2 reads closer
to buried in snow because its berm climbs about a meter up the walls under
denser icicles and puffier smoke. Its front wall falls into deep roof shadow in
the hero, while run 1's tidier cabin sits only moderately buried. Both round 2
runs' icicles read clearly in the front views, while round 1's nearly vanished
under the eave shadow.

Failures:

1. Run 1, pass 2: a single smoke voxel stayed a second piece. The final message
   acknowledged it without a fix
2. Run 2, before pass 1: permissions denied a python heredoc edit to raise the
   chimney. The session redid it with the Edit tool
3. Run 2, pass 1: the berm failed the build with
   `box round must be at most half the shortest side, 0.75, not 0.9`
4. Run 2, pass 2: a custom `#F4F8FC` snow failed the build with
   `shades shade 2 must be between black and white, not a lightness of 1.012`
5. Run 2, pass 4: displaced pine tiers and smoke split the model into 38 pieces.
   The final pass still leaves 7 single pine-snow voxels that the session called
   loose flakes

Lacked:

None

Missed in the skill:

1. Both runs built the roof snow as an added extrusion with guessed offsets
   instead of the `coat` on +y faces that `SKILL.md` names for roof snow. The
   thick slab does sell the buried look
2. Both runs left one-voxel pieces although the checks treat any second piece as
   a defect. Run 1 kept one smoke voxel and run 2 seven pine-snow voxels
3. Run 2 fed a custom near-white snow to `shades` although `SKILL.md` warns that
   `shades` errors past white. `mat.snow` would have avoided the failed build
4. Run 2's sill snow `coat` kept 2 of 121 cells because the later log end snow
   coat overlapped it. The session never remarked on the report line

Colors: Both runs' snow shows dense dark stair-step contour lines in the hero
and top views. Run 2's front wall falls into near-black roof shadow that hides
the door and window in the hero.

### 24. Valley

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | good    | 8      | 0      | 7.3     | 265,947 at 0.25 m |
| 2   | fair    | 6      | 1      | 4.0     | 267,805 at 0.4 m  |

Both runs built a 64 m diorama of displaced cones on a slab with pines seated by
a recomputed height function, each near 266k voxels in one piece. Run 1 reads
far more like a mountain valley because its six tall snowy peaks at 0.25 m ring
a meadow and a winding river. Run 2's three small snow knobs at 0.4 m sink under
the forest. Both runs tried the `--fill-mode surface` that round 1 never
weighed, though only run 1 kept it past the overcounting report.

Failures:

1. Run 1, after pass 1: permissions denied a python heredoc rewrite of
   `valley.ts`. The session redid it as seven Edit calls
2. Run 2, pass 1: permissions refused a `time ( ... )` wrapper with
   `A group in parentheses in this command can't be checked before it runs`
3. Both runs, pass 2 onward: the report under `--fill-mode surface` printed the
   solid count before hollowing. Run 1 learned the real 65k and later 266k from
   `vox-doc show`, while run 2 concluded surface fill did nothing
4. Run 1, pass 2: raising the peaks pushed the model to 357,132 voxels in 19
   pieces of crumbs from displaced cone tips
5. Run 1, pass 4: an Edit failed with `String to replace not found` on the
   header comment
6. Run 2, pass 4: a solid voxelize printed the same count. The session read it
   as confirmation. Its final message wrongly tells the user surface fill does
   not reduce the count
7. Run 2, pass 5: the hand-carved two-cell crust thinned under displacement and
   left 14 crumbs of one to four voxels
8. Both runs, late passes: a hard-coded carve box at a crumb's reported bounds
   removed the last crumb. Run 1 dropped its `loose rock` box once a
   `smoothUnion` core fixed the neck. Run 2 kept its `fleck` box. That box
   breaks as soon as the noise or grid changes
9. Run 1, after pass 7: permissions denied a `sips -g hasAlpha` check on the
   black-background renders

Lacked:

1. The terrain height at a point: both runs recomputed cone heights in
   TypeScript while ignoring the `smoothUnion` fillets and the displacement. Run
   1 sank its trunks 3 m and run 2 1.6 m
2. A heightfield or terrain primitive: both runs built the land from a slab plus
   displaced cones in a `smoothUnion`
3. A step that drops pieces under a size: both runs carved a box at a crumb's
   reported bounds. Run 1 later reshaped its peaks over an undisplaced core cone
4. The shell's real count under `--fill-mode surface`: the report's model line
   counts the solid fill before hollowing. Run 1's read 2,951,819 against
   265,947 written
5. Why the hero renders of both hollow models come out on a black background:
   run 1's top render does too while the front and right stay white

Missed in the skill:

1. Both runs opened with a wrapper that `SKILL.md` says needs its own approval.
   Run 1 used a python heredoc and run 2 a `time` wrapper
2. Both runs unioned their pines into two steps instead of placing one part at
   each spot, as Choosing operations item 7 suggests. Run 1 has 230 pines and
   run 2 has 120
3. Run 2's snow coat used plain `mat.snow` although every large face takes
   shades, noise, or a pattern
4. Run 2 skipped the top view on passes 5 to 7 and the right view on the last
   pass although the checks ask for all four PNGs every pass

Colors: Run 1's rock bands read as evenly spaced gray stripes rather than
strata. Its front and right views moire across the snow and rock. Run 2's snow
caps are one flat white. Its strata paint leaves orange-brown specks across the
grassy slopes.

### 25. Village

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | good    | 3      | 0      | 7.7     | 404,209 at 0.25 m |
| 2   | good    | 3      | 0      | 8.6     | 642,087 at 0.25 m |

Both runs ringed a market square of stalls and a well with eight hipped thatched
cottages and set a bespoke church on its north side. Run 1 reads better because
it ends in one clean piece with thatch that reads as straw and a church door
clear of clutter. Run 2 packs in dormers, a lychgate and a hay cart but ships
blotchy thatch, flat chimneys and two floating voxels. Neither round 2 run lost
a pass, where both round 1 runs lost pass 1 to `shades` and put z counts on the
x slot of `repeat`.

Failures:

1. Run 1, before pass 1: the first draft subtracted a box scaled by `.scale(0)`
   from the churchyard wall as a placeholder. The session rewrote it as a plain
   union
2. Run 2, before pass 1: the first Write left broken placeholder code in the
   tower pinnacles. The session rewrote the whole file
3. Both runs, pass 1: a cross floating above the spire and stray tree crown
   voxels made 28 pieces in run 1 and 61 in run 2. Run 1 also floated its well
   bucket
4. Run 1, pass 2: one stray leaf voxel on `tree.4` stayed until pass 3 reseeded
   its crown
5. Run 2, final pass: one loose voxel from each yew left 2 pieces. The summary
   waved them off as too small to see

Lacked:

1. Both runs: `part()` offers no turn per placement. Each run wrote a
   `cottage()` function that rotates every shape and builds a fresh part per
   spot
2. Both runs: a summary-only report. Each pass 1 report ran to 33 KB and spilled
   to a file that the session grepped for piece and 0-cell lines

Missed in the skill:

1. Both runs: the checks ask for all four PNGs every pass. Neither run read all
   four after pass 1
2. Run 2: the checks say a second piece means a floating shape that needs a fix.
   The session still shipped two floating yew voxels

Colors: Run 2's noise thatch reads as blotchy camouflage rather than straw. Its
chimneys, caps and doors take bare `mat.brick` and door materials with no shades
or pattern. The big brick stacks read as flat red blocks in the close views. Run
1 shows only small flat patches on its clock face and doors.

### 26. City

| Run | Verdict | Passes | Failed | Minutes | Voxels              |
| --- | ------- | ------ | ------ | ------- | ------------------- |
| 1   | good    | 2      | 0      | 7.5     | 1,264,521 at 0.25 m |
| 2   | fair    | 4      | 1      | 7.7     | 623,524 at 0.25 m   |

Both runs built a parts-free city at 0.25 m with crossing streets, crosswalks,
24 lamp posts and a park. Run 1 reads better because its four blocks hold
distinctive landmark towers and a park that fills a whole quadrant. Run 2's nine
blocks read more like a skyline but repeat one boxy slab around a 7 m park that
nearly vanishes. Both round 2 runs halved round 1's 0.5 m voxel.

Failures:

1. Both runs, before pass 1: the sandbox blocked an `ls` of the parent
   `~/voxel-trials` directory
2. Run 1, pass 1: floating awnings, bench backs and stray displaced leaf voxels
   made 14 pieces. Pass 2 brought it to 1 piece
3. Run 1, before pass 2: an Edit missed its target because an earlier edit had
   rewritten the awning code
4. Run 2, pass 1: the chain wrapped in `time ( ... )` was refused with
   `A group in parentheses in this command can't be checked before it runs`
5. Run 2, pass 3: three stray leaf voxels on a displaced crown made 4 pieces.
   Pass 4 brought it to 1 piece
6. Run 2, final pass: the close-up shows a tree planted directly in front of the
   fountain. The session left it

Lacked:

1. Run 1: a summary-only report. The session grepped the roughly 100-line pass 2
   report for `roof|piece|0 kept`
2. Run 2: a built-in filter for problem lines such as `0 kept`, `0 exposed` and
   piece lines. The session grepped with a hand-written regex of step names

Missed in the skill:

1. Run 1: SKILL.md's Other views covers close-ups. The session never rendered
   one to check windows, lamp heads or benches at street level
2. Run 1: the checks flag large flat faces and send a named flaw to another
   pass. The session stopped after pass 2 with the confetti flower ring and the
   flat lobby band unchecked
3. Run 2: SKILL.md says a pass chained after a `time` wrapper needs its own
   approval. The session still wrapped pass 1 in a `time` subshell
4. Run 2: the checks ask for all four PNGs every pass. After pass 1 the session
   read none in pass 3 and only the close-up and hero in pass 4
5. Run 2: the checks on glass and neighboring colors would flag the tallest
   tower. The tower named glass came out as steel walls with slit windows that
   read as stone

Colors: Run 1's glass tower carries a flat bright yellow lobby band over a
curtain wall banded by flat tint blocks. Run 2's glass tower merges dark
blue-grey noise with its dark windows into one murky tone. Its fountain basin
reads as plain white blocks.

### 27. Dollhouse

| Run | Verdict | Passes | Failed | Minutes | Voxels          |
| --- | ------- | ------ | ------ | ------- | --------------- |
| 1   | good    | 2      | 0      | 7.8     | 874,601 at 5 cm |
| 2   | good    | 2      | 0      | 9.4     | 771,964 at 5 cm |

Both runs built an 8 m gable house at 5 cm with the +z wall cut away and white
trim on every cut edge. Run 2 reads better because its six rooms hold more
recognizable furniture and its exterior adds a side door, path and bushes. Run 1
has the better living room and attic but a sparse bathroom and flatter walls.
Both round 2 runs took the 5 cm grid that only round 1's run 2 used.

Failures:

1. Run 1, before pass 2: a python heredoc edit chained with grep was denied with
   `This Bash command contains multiple operations`. The session redid it as
   four Edit calls
2. Run 1, pass 2: piping the report through `head -1` left the step lines
   unchecked after the edits
3. Run 2, before pass 1: the first write held junk expressions such as
   `mat.chrome ?? mat.steel` that took three Edits to clean up
4. Run 2, pass 1: displaced bushes and plant leaves made 28 pieces. The
   `paint monitor screen` step kept 0 cells
5. Run 2, pass 2: removing the `eaves` paint left white specks at the front roof
   corners in the final top view. The final message still lists them as fixed
6. Both runs, pass 2: each session reviewed only two of the PNGs. Run 2 still
   declared `All four review views look right`

Lacked:

None

Missed in the skill:

1. Both runs: Building step 1 asks every named feature to span at least 2
   voxels. Each run drew 5 cm legs 1 voxel thick without weighing
   `--fill-mode surface` for a finer voxel
2. Both runs: Building step 3 asks for a block-out pass. The first pass was the
   full furnished model of 430 lines in run 1 and 500 in run 2
3. Both runs: Choosing operations item 7 places repeated props as one part per
   spot. Helper functions rebuilt each dining chair, plant and lamp instead
4. Both runs: the checks count a fix only once the next pass's PNGs show it. Run
   2 claimed the roof speck fix without looking at the top or right view
5. Run 1: the pass section routes edits through the file-editing tool. A chained
   python heredoc cost a permission denial

Colors: Run 1 leaves the kitchen walls flat mustard, the bathroom upper walls
flat teal and the ceilings flat cream. Its navy tub exterior reads as a box. Run
2's white trim and stair stringer stay flat. Its noise-toned study and kitchen
partition walls read blotchy rather than papered.

### 28. Living room

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | good    | 3      | 0      | 6.8     | 339,741 at 2.5 cm |
| 2   | fair    | 3      | 1      | 5.4     | 302,062 at 2.5 cm |

Both runs built an open-corner cutaway at 2.5 cm with the fireplace on the back
wall and the bookshelf and a night window on the left. Run 1 reads better
because of its brick bond, staggered planks and a bookshelf that varies color,
size and set-back from book to book. Run 2 falls behind on an armchair that
faces away from the fire and one gold stripe that crosses every shelf of books.
Both round 2 runs added the eye-level inside view that round 1's run 2 lacked.

Failures:

1. Both runs: permissions denied a read-only grep in run 1 before pass 3 and a
   perl in-place edit in run 2 before pass 1. Each session switched to Edit
2. Run 1, pass 2: a floating grate bar and two plant voxels made 4 pieces. Pass
   3 fixed them
3. Run 2, pass 1: the floating lamp shade made 2 pieces. The clock face kept 0
   cells until a later pass
4. Run 2, pass 2: the build failed with `TypeError: star is not a function`
   because a material named `star` shadowed the `star` 2D shape. Renaming it
   `starLight` fixed it

Lacked:

None

Missed in the skill:

1. Both runs: SKILL.md says a room keeps its walls and takes a re-aimed or
   inside view. Each session dropped the front and right walls for a cutaway
2. Run 1: SKILL.md says furniture reads cleanest square to the axes. The session
   turned the armchair 45 degrees and called the resulting ribs corduroy
3. Run 1: the checks ask for all four PNGs every pass. The session read only two
   views after pass 1 and after pass 3
4. Run 2: SKILL.md says the first pass gives each large form one box. The first
   pass was a full 15 KB model
5. Run 2: SKILL.md gives per-copy variety its own step and seed. One shelf-wide
   gold stripe intersected with all the books gave every copy the same band

Colors: Run 1's night window pane is one flat navy. Its cushions and lamp shade
read as single flat tones beside the noised chair body. Run 2's lamp shade and
picture sky read flat. Its walls, rug field and velvet carry noise that reads as
camouflage. Its floor reads as stripes rather than planks.

### 29. Spaceship

| Run | Verdict | Passes | Failed | Minutes | Voxels          |
| --- | ------- | ------ | ------ | ------- | --------------- |
| 1   | good    | 4      | 0      | 4.5     | 333,091 at 4 cm |
| 2   | good    | 5      | 1      | 4.7     | 122,678 at 5 cm |

Both runs built a cutaway bridge with a radar table between the captain's chair
and a window on -z and checked it from an eye-level inside view. Run 2 reads
better because its dark hull, black starfield bay and larger radar sell the
stars and the glow. Run 1 has the richer room and stronger chair, but its glass
pane washes the starfield to grey. Both round 2 radars read as rings where round
1's run 2 radar never formed them.

Failures:

1. Run 1, pass 2: an unclipped planet and ring stuck out through the back of the
   hull and stretched the bounds to z = -4.6. Pass 3 squashed them into the
   window bay with an ellipsoid and `halfSpace`
2. Run 1, final pass: the glass pane turns the black starfield a hazy grey. The
   session only raised its transmission slightly
3. Run 2, pass 2: voxelize failed with
   `bridge/stars white: set points must be distinct, not [1.675, 1.425, -3.325] twice`
   because the hand-rolled star RNG produced a duplicate
4. Run 2, pass 3: a solid grey sheet covered the window because the window-frame
   cutter stopped short. The session deleted the glass on a wrong guess and
   fixed the cutter only in pass 5
5. Run 2, final pass: the window has no glass because the session never restored
   the pane after finding the real cause

Lacked:

None

Missed in the skill:

1. Both runs: the checks ask for all four PNGs every pass. Run 1's passes 3 and
   4 and run 2's passes 4 and 5 skipped the right and top views
2. Run 1: the final pass piped the report through `head -1`. Every step line
   went unread
3. Run 2: `speckle` on the bay faces places stars in one step. The session wrote
   an LCG, `set` point lists and a manual dedupe set instead
4. Run 2: pass 3's report already showed the frame filling the opening with
   `window frame 6828 kept / 5656 exposed` and `carve window ... 12600 kept` of
   17,472 cells. The session removed the glass on a guess instead

Colors: Run 1's glass veils the space backdrop to a flat mid-grey. Its
side-station monitor is one flat blue rectangle. Its pale checkered walls read
clinical beside the saturated cyan strips. Run 2's steel chair shell and radar
rim read as flat pale grey slabs against the dark room. Its console screens are
plain blue stripes.

### 30. Pirate ship

| Run | Verdict | Passes | Failed | Minutes | Voxels            |
| --- | ------- | ------ | ------ | ------- | ----------------- |
| 1   | fair    | 4      | 0      | 4.5     | 325,192 at 2.5 cm |
| 2   | fair    | 8      | 0      | 8.2     | 330,940 at 2.5 cm |

Both runs built a flat-walled cutaway gun deck at 2.5 cm with two cannons on red
carriages, two hammocks, four barrels and an emissive lantern. Run 2 reads
better because its cannons run out through carved, lined ports and its barrel
stack and iron lantern read as ship fittings. Run 1's hammocks read better as
slings from the side, but both runs' rims come out jagged. Neither round 2 run
curved the hull, where round 1's run 2 flared it with an extruded polygon.

Failures:

1. Run 1, pass 1: the pass line wrapped in `time ( ... )` was denied with
   `A group in parentheses in this command can't be checked before it runs`
2. Run 1, pass 1: floating details such as the lantern core, quoin and bucket
   made 8 pieces. Pass 2 brought it to 1 piece
3. Run 1, final pass: the session never viewed the pass that deleted the unused
   `lampCore` material and `lanternCore` shape. The inside render on disk comes
   from the pass before
4. Run 2, pass 2: floating details made 3 pieces. Pass 3 joined them into 1
   piece
5. Run 2, pass 2: a grep to locate edit lines was denied with
   `This command requires approval`
6. Run 2, pass 4: the bent hammock trough of a clipped cylinder came out with
   torn-looking rims. Pass 6 replaced it
7. Run 2, pass 7: bending the ellipsoid sling pulled its ends away from the
   ropes for 3 pieces. Pass 8 moved the rope anchors

Lacked:

1. Both runs: a light-casting source or point light. Each lantern glows only
   through `emissiveColor` while the review daylight lights the room

Missed in the skill:

1. Run 1: Building step 3 asks for a block-out pass of boxes. The session wrote
   the fully detailed scene first
2. Run 1: SKILL.md says a pass chained after a `time` wrapper needs its own
   approval. The session still wrapped pass 1 in a `time` group
3. Run 1: check 6 gives a pale glow a lower `emissiveStrength` or a darker
   `baseColor`. The session left the lantern pane pale peach
4. Run 2: SKILL.md warns that distances read unreliably after `bend`. The
   session bent a shelled ellipsoid for the hammock and shipped ragged rims

Colors: Run 1's lantern pane reads pale peach instead of amber. Its cannon
barrels and shot pyramid are near-black blocks with little shading. Its very
light pine floor flattens the scene. Run 2's floor and hull planks read as
grainless laminate bands. Its beams and knees share one saturated red-brown. Its
cannon barrels stay flat black with no highlight.
