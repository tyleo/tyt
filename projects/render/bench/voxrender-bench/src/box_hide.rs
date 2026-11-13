use crate::FaceShadow;
use std::ops::Range;
use ty_math::TyVector3F64;

/// The most shadows an eye has.
const SHADOWS: usize = 64;

/// The most nodes a node holds.
const FAN: usize = 8;

/// The tests from one search of a tree to its next, and the most for a tree
/// whose searches hide no box.
const SEARCH: u32 = 16;
const LONGEST: u32 = 64;

/// The part of a point's distance from the eye that the point lies past a
/// shadow's face by, which covers what a depth rounds off.
const PAST: f64 = 1.0 / 4096.0;

/// The part of a scene's reach from the world's origin that a box keeps
/// inside a shadow's planes by, which covers what f32 rounds off a world
/// position.
const LOST: f64 = 1.0 / 1024.0 / 1024.0;

/// A test that keeps a view's hidden boxes from drawing. A box that lies
/// whole in the shadow one of the scene's solid slabs casts from an eye
/// shows that eye no pixel, because each ray to it meets the slab's opaque
/// cells first. The box that draws those cells is nearer and is never
/// hidden in turn, so a frame without the hidden boxes draws the same image.
///
/// A box is hidden when one shadow of each eye holds it whole. The test
/// reads the world box around a box, which holds a turned box with room to
/// spare.
///
/// Each run of a list's boxes makes a tree: the boxes that follow one
/// another make groups of [`FAN`], and those groups make groups in turn. A
/// shadow that holds a group whole hides each box in it, and one that holds
/// none of a group is left out for each box in it.
///
/// A tree's search tries every shadow, and comes once in [`SEARCH`] tests.
/// The tests between keep a box hidden only while the shadow that last
/// held it still does. A tree whose search hides no box waits twice as long
/// for its next, up to [`LONGEST`] tests. A box that comes to hide can then
/// draw for some frames more, which costs time and no pixel.
pub struct BoxHide {
    slabs: Vec<Slab>,

    /// The slab in each of the eyes' shadows. A slab keeps its place from
    /// one aim to the next, so the shadow a box remembers stays that
    /// slab's.
    aimed: Vec<Option<usize>>,

    lists: Vec<List>,

    /// What a box keeps inside a shadow's planes by, in world meters.
    lost: f64,
}

/// A slab of opaque cells, with what an aim ranks it by.
struct Slab {
    /// The corners in the world, as [`FaceShadow::new`] takes them.
    corners: [TyVector3F64; 8],

    center: TyVector3F64,

    /// Each axis's unit direction, half the slab's length along it, and the
    /// area of a face across it.
    axes: [(TyVector3F64, f64, f64); 3],
}

impl Slab {
    fn new(corners: [TyVector3F64; 8]) -> Self {
        // A face's normal runs along the edge across it because a slab's
        // edges meet at right angles.
        let edges = [1, 2, 4].map(|far| corners[far] - corners[0]);
        let lengths = edges.map(TyVector3F64::length);

        Slab {
            corners,
            center: (corners[0] + corners[7]) / 2.0,
            axes: [0, 1, 2].map(|axis| {
                let area = lengths[(axis + 1) % 3] * lengths[(axis + 2) % 3];

                (edges[axis] / lengths[axis], lengths[axis] / 2.0, area)
            }),
        }
    }

    /// The solid angle of the face `eye` sees widest, near enough to rank
    /// slabs by. Zero where the eye faces no face from outside.
    fn wide(&self, eye: TyVector3F64) -> f64 {
        let off = eye - self.center;

        self.axes
            .iter()
            .map(|&(along, half, area)| {
                let out = along.dot(off);

                // The eye stands this far before the plane of the face it
                // can see.
                let before = out.abs() - half;
                let to = off - along * half.copysign(out);

                if before > 0.0 {
                    area * before / to.length().powi(3)
                } else {
                    0.0
                }
            })
            .fold(0.0, f64::max)
    }
}

/// One of the lists of boxes a test covers.
struct List {
    /// The trees' nodes: each tree's root, then the nodes a step from each
    /// root, and on down. A test reads the roots and little else, and they
    /// lie together.
    nodes: Vec<Node>,

    /// The tests left before each tree's next search, and the tests from
    /// its last search to that one. A tree yet to search holds zeros.
    searches: Vec<[u32; 2]>,

    /// Whether the last test hid each box.
    hidden: Vec<bool>,
}

/// A box, or the box around a group of them.
struct Node {
    center: [f32; 3],

    half: [f32; 3],

    /// The half-size's length.
    radius: f32,

    /// The first of the node's parts in its list's nodes.
    first_part: u32,

    /// The first of the boxes the node holds in its list, and their count.
    first_box: u32,
    box_count: u32,

    /// The node's parts. A box has none.
    part_count: u8,

    /// Each eye's shadow that last held the node whole.
    last: [u8; 2],

    /// Whether the last test hid a box of the node's.
    hides: bool,
}

impl Node {
    fn parts(&self) -> Range<usize> {
        let first = self.first_part as usize;

        first..first + usize::from(self.part_count)
    }

    fn boxes(&self) -> Range<usize> {
        let first = self.first_box as usize;

        first..first + self.box_count as usize
    }
}

/// A node as its tree grows: its bounds, its parts in the level below, and
/// the boxes it holds.
type Sprout = ([f32; 6], Range<usize>, Range<usize>);

/// How much of a node a shadow holds.
#[derive(Eq, PartialEq)]
enum Held {
    None,

    Part,

    Whole,
}

/// The shadows of an eye that can hold a node's parts: all of them, or
/// those of `some`'s first `count`.
#[derive(Clone, Copy)]
struct Among {
    count: usize,

    some: Option<[u8; SHADOWS]>,
}

impl Among {
    fn shadows(&self) -> impl Iterator<Item = usize> + '_ {
        (0..self.count).map(|at| self.some.map_or(at, |some| usize::from(some[at])))
    }
}

/// One eye of a test.
struct Eye<'a> {
    /// The eye's index.
    index: usize,

    position: TyVector3F64,

    /// Each shadow's planes.
    shadows: &'a [[[f64; 4]; 5]],

    /// The sine of the angle a box keeps inside a shadow's sides by.
    angle: f64,

    /// [`BoxHide`]'s `lost`.
    lost: f64,
}

impl Eye<'_> {
    /// How much of `node` the shadow `shadow` holds. No point of the node
    /// lies farther than `reach` from the eye. A node the shadow does not
    /// hold whole reads as held in part when `whole_only`.
    fn held(&self, shadow: usize, node: &Node, reach: f64, whole_only: bool) -> Held {
        let [x, y, z] = node.center.map(f64::from);
        let half = node.half.map(f64::from);
        let mut whole = true;

        for (at, [a, b, c, offset]) in self.shadows[shadow].iter().enumerate() {
            let middle = a * x + b * y + c * z + offset;
            let spread = a.abs() * half[0] + b.abs() * half[1] + c.abs() * half[2];

            if middle + spread < 0.0 {
                return Held::None;
            }

            let margin = if at == 0 { PAST } else { self.angle };

            whole &= middle - spread >= margin * reach + self.lost;

            if whole_only && !whole {
                return Held::Part;
            }
        }

        if whole { Held::Whole } else { Held::Part }
    }

    /// A distance no point of `node` lies farther than from the eye.
    fn reach(&self, node: &Node) -> f64 {
        let [x, y, z] = node.center.map(f64::from);

        (TyVector3F64::new(x, y, z) - self.position).length() + f64::from(node.radius)
    }

    /// Whether the shadow that last held `node` whole still does.
    fn keeps(&self, node: &Node) -> bool {
        let last = usize::from(node.last[self.index]);

        last < self.shadows.len() && self.held(last, node, self.reach(node), true) == Held::Whole
    }

    /// Whether a shadow of `among` holds `node` whole, and else the shadows
    /// of `among` that hold part of it. The shadow that last held the node
    /// goes first.
    fn sort(&self, among: &Among, node: &mut Node) -> (bool, Among) {
        let reach = self.reach(node);
        let leaf = node.part_count == 0;

        let mut parts = Among {
            count: 0,
            some: Some([0; SHADOWS]),
        };

        if self.keeps(node) {
            return (true, parts);
        }

        for shadow in among.shadows() {
            match self.held(shadow, node, reach, leaf) {
                Held::None => {}

                Held::Part => {
                    if let Some(some) = &mut parts.some {
                        some[parts.count] = shadow as u8;
                    }

                    parts.count += 1;
                }

                Held::Whole => {
                    node.last[self.index] = shadow as u8;

                    return (true, parts);
                }
            }
        }

        (false, parts)
    }
}

impl List {
    /// Tests the node `at` and what it holds for `eyes`, under each eye's
    /// answer `above` for the group that holds the node: whether a shadow
    /// holds that group whole, and else the shadows that hold part of it.
    /// Returns whether a box's answer changed.
    ///
    /// A test that `searches` tries each shadow. Any other asks only
    /// whether the shadow that last held a node still does, and passes
    /// over a node that hid no box.
    fn test(
        &mut self,
        eyes: &[Eye<'_>; 2],
        above: [(bool, Among); 2],
        at: usize,
        searches: bool,
    ) -> bool {
        let node = &mut self.nodes[at];

        if !searches && !node.hides {
            return false;
        }

        let leaf = node.part_count == 0;
        let mut answers = above;

        // An eye with no shadow that holds the node or a part of it sees
        // every box in it.
        let mut hides = true;

        for (eye, answer) in eyes.iter().zip(&mut answers) {
            if hides && !answer.0 {
                if searches {
                    *answer = eye.sort(&answer.1, node);
                    hides = answer.0 || (!leaf && answer.1.count > 0);
                } else {
                    answer.0 = eye.keeps(node);
                    hides = answer.0 || !leaf;
                }
            }
        }

        let whole = answers[0].0 && answers[1].0;

        if !hides || whole {
            let mut below = node.parts();
            let boxes = &mut self.hidden[node.boxes()];
            let changed = boxes.iter().any(|&hidden| hidden != whole);

            boxes.fill(whole);
            node.hides = whole;

            // Each node below now holds boxes of the one answer. A level's
            // nodes lie together, and so do their parts.
            while changed && !below.is_empty() {
                let level = &mut self.nodes[below];
                let parts = level[0].parts().start..level[level.len() - 1].parts().end;

                level.iter_mut().for_each(|node| node.hides = whole);
                below = parts;
            }

            return changed;
        }

        let mut changed = false;
        let mut hides = false;

        for part in node.parts() {
            changed |= self.test(eyes, answers, part, searches);
            hides |= self.nodes[part].hides;
        }

        self.nodes[at].hides = hides;

        changed
    }
}

impl BoxHide {
    /// Builds the test of the boxes of `lists` against the shadows of
    /// `slabs`.
    ///
    /// # Arguments
    ///
    /// 1. `slabs`: each slab's corners in the world, as
    ///    [`FaceShadow::new`] takes them
    /// 2. `lists`: for each list, the low and then the high corner of the
    ///    world box around each of its boxes, and how many boxes each of
    ///    its runs holds. The boxes of a run lie near one another, as one
    ///    placement's do, and a shadow can hold a run whole
    pub fn new(slabs: Vec<[TyVector3F64; 8]>, lists: Vec<(Vec<[f32; 6]>, Vec<usize>)>) -> Self {
        let reach = lists
            .iter()
            .flat_map(|(bounds, _)| bounds.iter().flatten())
            .fold(0.0, |reach: f64, &bound| reach.max(f64::from(bound.abs())));

        let lists = lists
            .into_iter()
            .map(|(bounds, runs)| {
                let hidden = vec![false; bounds.len()];

                // Each tree's levels from its root down.
                let mut trees: Vec<Vec<Vec<Sprout>>> = Vec::new();
                let mut first = 0;

                for run in runs.into_iter().filter(|&run| run > 0) {
                    let boxes = first..first + run;

                    first += run;

                    let mut levels: Vec<Vec<Sprout>> = vec![
                        boxes
                            .map(|index| (bounds[index], 0..0, index..index + 1))
                            .collect(),
                    ];

                    while let Some(below) = levels.last().filter(|below| below.len() > 1) {
                        let level = (0..)
                            .step_by(FAN)
                            .zip(below.chunks(FAN))
                            .map(|(first, group)| {
                                let mut around = group[0].0;

                                for (bounds, ..) in group {
                                    for axis in 0..3 {
                                        around[axis] = around[axis].min(bounds[axis]);
                                        around[axis + 3] = around[axis + 3].max(bounds[axis + 3]);
                                    }
                                }

                                let boxes = group[0].2.start..group[group.len() - 1].2.end;

                                (around, first..first + group.len(), boxes)
                            })
                            .collect();

                        levels.push(level);
                    }

                    levels.reverse();
                    trees.push(levels);
                }

                // Where each tree's levels start among the nodes.
                let depth = trees.iter().map(Vec::len).max().unwrap_or(0);
                let mut starts: Vec<Vec<usize>> = vec![Vec::new(); trees.len()];
                let mut count = 0;

                for level in 0..depth {
                    for (tree, starts) in trees.iter().zip(&mut starts) {
                        if let Some(level) = tree.get(level) {
                            starts.push(count);
                            count += level.len();
                        }
                    }
                }

                let mut nodes = Vec::with_capacity(count);

                for level in 0..depth {
                    for (tree, starts) in trees.iter().zip(&starts) {
                        for (bounds, parts, boxes) in tree.get(level).into_iter().flatten() {
                            let half =
                                [0, 1, 2].map(|axis| (bounds[axis + 3] - bounds[axis]) / 2.0);
                            let below = starts.get(level + 1).copied().unwrap_or(0);

                            let narrow = |value: usize| {
                                u32::try_from(value).expect("a scene's boxes number under 2^32")
                            };

                            nodes.push(Node {
                                center: [0, 1, 2]
                                    .map(|axis| (bounds[axis + 3] + bounds[axis]) / 2.0),
                                half,
                                radius: half.iter().map(|side| side * side).sum::<f32>().sqrt(),
                                first_part: narrow(below + parts.start),
                                first_box: narrow(boxes.start),
                                box_count: narrow(boxes.len()),
                                part_count: parts.len() as u8,
                                last: [u8::MAX; 2],
                                hides: false,
                            });
                        }
                    }
                }

                List {
                    nodes,
                    searches: vec![[0; 2]; trees.len()],
                    hidden,
                }
            })
            .collect();

        BoxHide {
            aimed: vec![None; slabs.len().min(SHADOWS)],
            slabs: slabs.into_iter().map(Slab::new).collect(),
            lists,
            lost: reach * LOST,
        }
    }

    /// Picks the slabs whose shadows the eyes have: the widest from `head`.
    pub fn aim(&mut self, head: TyVector3F64) {
        let mut wide: Vec<_> = self
            .slabs
            .iter()
            .zip(0..)
            .map(|(slab, index)| (slab.wide(head), index))
            .filter(|&(wide, _)| wide > 0.0)
            .collect();

        let places = self.aimed.len();

        if wide.len() > places {
            wide.select_nth_unstable_by(places, |a, b| b.0.total_cmp(&a.0));
            wide.truncate(places);
        }

        // A slab among the widest keeps its place, and the rest take the
        // places left.
        let mut widest: Vec<_> = wide.into_iter().map(|(_, slab)| slab).collect();

        for place in &mut self.aimed {
            *place = place.filter(|slab| {
                let kept = widest.iter().position(|widest| widest == slab);

                kept.map(|kept| widest.swap_remove(kept)).is_some()
            });
        }

        let mut rest = widest.into_iter();

        for place in self.aimed.iter_mut().filter(|place| place.is_none()) {
            *place = rest.next();
        }
    }

    /// Tests each box for `eyes`, each in the world, and returns whether a
    /// box's answer changed. A box keeps inside a shadow's sides by an angle
    /// whose sine is `angle`, which covers where the rasterizer rounds a
    /// corner to.
    pub fn test(&mut self, eyes: [TyVector3F64; 2], angle: f64) -> bool {
        // A place with no slab, or with one an eye sees no face of, holds a
        // shadow no point lies in.
        let shadows = eyes.map(|eye| {
            self.aimed
                .iter()
                .map(|&slab| {
                    slab.and_then(|slab| FaceShadow::new(&self.slabs[slab].corners, eye))
                        .map_or([[0.0, 0.0, 0.0, -1.0]; 5], |shadow| shadow.planes)
                })
                .collect::<Vec<_>>()
        });

        let eyes = [0, 1].map(|index| Eye {
            index,
            position: eyes[index],
            shadows: &shadows[index],
            angle,
            lost: self.lost,
        });

        let every = Among {
            count: self.aimed.len(),
            some: None,
        };

        let mut changed = false;

        for list in &mut self.lists {
            // A tree's root comes at the tree's index among the nodes.
            for root in 0..list.searches.len() {
                let [left, wait] = list.searches[root];
                let searches = left == 0;

                changed |= list.test(&eyes, [(false, every); 2], root, searches);

                list.searches[root] = if !searches {
                    [left - 1, wait]
                } else if wait == 0 {
                    // The trees take turns to search after their first.
                    [root as u32 % SEARCH, SEARCH]
                } else if list.nodes[root].hides {
                    [SEARCH - 1, SEARCH]
                } else {
                    let wait = (wait * 2).min(LONGEST);

                    [wait - 1, wait]
                };
            }
        }

        changed
    }

    /// Whether the last [`test`](Self::test) hid box `index` of list `list`.
    pub fn hides(&self, list: usize, index: usize) -> bool {
        self.lists[list].hidden[index]
    }
}

#[cfg(test)]
mod tests {
    use crate::{BoxHide, box_hide::Slab};
    use ty_math::TyVector3F64;

    /// The corners of a wall from (10, 0, -4) to (11, 8, 4).
    fn wall() -> [TyVector3F64; 8] {
        [0, 1, 2, 3, 4, 5, 6, 7].map(|corner: u32| {
            let far = |axis: u32, meters: f64| f64::from(corner >> axis & 1) * meters;

            TyVector3F64::new(10.0 + far(0, 1.0), far(1, 8.0), far(2, 8.0) - 4.0)
        })
    }

    /// The test of the boxes of `bounds`, in one list, against the wall.
    fn hide(bounds: Vec<[f32; 6]>) -> BoxHide {
        let run = bounds.len();

        BoxHide::new(vec![wall()], vec![(bounds, vec![run])])
    }

    /// Eyes 6 cm apart that look down +X from 4 m up.
    fn eyes() -> [TyVector3F64; 2] {
        [-0.03, 0.03].map(|z| TyVector3F64::new(0.0, 4.0, z))
    }

    /// Which of `bounds` a test from `eyes()` hides.
    fn hidden(bounds: &[[f32; 6]]) -> Vec<bool> {
        let mut hide = hide(bounds.to_vec());

        hide.aim(TyVector3F64::new(0.0, 4.0, 0.0));
        hide.test(eyes(), 1.0e-3);

        (0..bounds.len())
            .map(|index| hide.hides(0, index))
            .collect()
    }

    #[test]
    fn a_slab_ranks_by_the_face_an_eye_sees_widest() {
        let slab = Slab::new(wall());

        // The near face spans 64 m2, 10 m off. The top face is edge on.
        assert!((slab.wide(TyVector3F64::new(0.0, 4.0, 0.0)) - 0.64).abs() < 1e-9);

        // From high above, the top face's 8 m2 shows widest.
        assert!((slab.wide(TyVector3F64::new(10.5, 108.0, 0.0)) - 8.0e-4).abs() < 1e-9);

        // An eye inside the wall faces none of its faces.
        assert_eq!(slab.wide(TyVector3F64::new(10.5, 4.0, 0.0)), 0.0);
    }

    #[test]
    fn a_wall_hides_the_boxes_behind_it() {
        let boxes = [
            // Behind the wall's middle.
            [20.0, 3.0, -1.0, 21.0, 5.0, 1.0],
            // Before the wall.
            [5.0, 3.0, -1.0, 6.0, 5.0, 1.0],
            // Behind the wall, and out past its top edge.
            [20.0, 14.0, -1.0, 21.0, 18.0, 1.0],
            // Behind the wall's far face, against it.
            [11.0, 1.0, -3.0, 12.0, 2.0, -2.0],
            // Through the wall.
            [9.0, 3.0, -1.0, 12.0, 5.0, 1.0],
        ];

        assert_eq!(hidden(&boxes), [true, false, false, true, false]);
    }

    #[test]
    fn a_box_one_eye_sees_past_the_wall_shows() {
        // From the right eye the box's edge clears the wall's side. From
        // the left eye the wall covers it.
        let boxes = [
            [20.0, 3.0, 7.9, 21.0, 5.0, 7.99],
            [20.0, 3.0, 7.0, 21.0, 5.0, 7.9],
        ];

        assert_eq!(hidden(&boxes), [false, true]);
    }

    #[test]
    fn a_test_reports_a_changed_answer() {
        let mut hide = hide(vec![[20.0, 3.0, -1.0, 21.0, 5.0, 1.0]]);

        hide.aim(TyVector3F64::new(0.0, 4.0, 0.0));

        assert!(hide.test(eyes(), 1.0e-3));
        assert!(!hide.test(eyes(), 1.0e-3));

        // Eyes beside the wall see the box.
        let beside = [-0.03, 0.03].map(|z| TyVector3F64::new(15.0, 4.0, 30.0 + z));

        assert!(hide.test(beside, 1.0e-3));
        assert!(!hide.hides(0, 0));
    }

    #[test]
    fn boxes_a_group_hid_show_once_its_shadow_lets_go() {
        // 200 boxes in a row behind the wall, which one shadow holds as a
        // group.
        let boxes: Vec<_> = (0..200)
            .map(|at| {
                let z = -3.0 + at as f32 * 0.02;

                [20.0, 3.0, z, 20.5, 4.0, z + 0.01]
            })
            .collect();

        let mut hide = hide(boxes);

        hide.aim(TyVector3F64::new(0.0, 4.0, 0.0));
        hide.test(eyes(), 1.0e-3);
        hide.test(eyes(), 1.0e-3);

        assert!((0..200).all(|index| hide.hides(0, index)));

        // Eyes beside the wall see each box, in a test that searches no
        // tree.
        let beside = [-0.03, 0.03].map(|z| TyVector3F64::new(15.0, 4.0, 30.0 + z));

        assert!(hide.test(beside, 1.0e-3));
        assert!((0..200).all(|index| !hide.hides(0, index)));
    }

    #[test]
    fn a_box_that_comes_to_hide_waits_for_its_trees_search() {
        let mut hide = hide(vec![[20.0, 3.0, -1.0, 21.0, 5.0, 1.0]]);
        let beside = [-0.03, 0.03].map(|z| TyVector3F64::new(15.0, 4.0, 30.0 + z));

        hide.aim(TyVector3F64::new(0.0, 4.0, 0.0));

        // The first tree searches in its first two tests.
        hide.test(beside, 1.0e-3);
        hide.test(beside, 1.0e-3);

        // The eyes come before the wall. The tree's next search finds the
        // box hidden. A tree that hid no box waits longer for it.
        let tests = (1..=64)
            .find(|_| hide.test(eyes(), 1.0e-3))
            .expect("a search comes within 64 tests");

        assert!(tests > 1);
        assert!(hide.hides(0, 0));
    }

    #[test]
    fn many_boxes_sort_by_their_groups() {
        // 200 boxes in a row behind the wall, then 200 beside it.
        let behind = (0..200).map(|at| {
            let z = -3.0 + at as f32 * 0.02;

            [20.0, 3.0, z, 20.5, 4.0, z + 0.01]
        });

        let beside = (0..200).map(|at| {
            let z = 30.0 + at as f32;

            [20.0, 3.0, z, 20.5, 4.0, z + 0.5]
        });

        let boxes: Vec<_> = behind.chain(beside).collect();
        let hidden = hidden(&boxes);

        assert!(hidden[..200].iter().all(|&hidden| hidden));
        assert!(hidden[200..].iter().all(|&hidden| !hidden));
    }
}
