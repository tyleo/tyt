use crate::{BoxRun, PlacedBox, sort_keys, squared_distance, to_word};

/// One list of a view's boxes as a sort reads them. A sort reads every
/// box's bounds and little else, so each of a box's parts lies with the
/// other boxes' like part.
#[derive(Default)]
pub struct ViewBoxes {
    /// The low and then the high corner of the world box around each box.
    pub bounds: Vec<[f32; 6]>,

    /// Each box's three `vec4<u32>`s.
    pub words: Vec<[u32; 12]>,

    /// The page that holds each box's tile.
    pub page: Vec<usize>,

    /// Whether each box's fragments can cast a mirror ray.
    pub mirror: Vec<bool>,

    /// Whether each box's fragments can meet an emission that blooms.
    pub glow: Vec<bool>,
}

impl ViewBoxes {
    pub fn push(
        &mut self,
        placed: &PlacedBox,
        words: [u32; 12],
        page: usize,
        mirror: bool,
        glow: bool,
    ) {
        let (low, high) = placed.world();
        let [low_x, low_y, low_z] = low.as_vec3().to_array();
        let [high_x, high_y, high_z] = high.as_vec3().to_array();

        self.bounds
            .push([low_x, low_y, low_z, high_x, high_y, high_z]);
        self.words.push(words);
        self.page.push(page);
        self.mirror.push(mirror);
        self.glow.push(glow);
    }

    /// The index of the box a sort's `key` holds.
    pub fn index_of(key: u64) -> usize {
        (key & 0xffff_ffff) as usize
    }

    /// Fills `keys` with a key per box, nearest to `point` first, and
    /// returns how many of the boxes lie within `reach` of `point`. A key
    /// holds the bits of its box's squared distance above the box's index.
    /// With `shown`, only the boxes at those indices take a key.
    pub fn nearest_first(
        &self,
        shown: Option<&[usize]>,
        point: [f32; 3],
        reach: f32,
        keys: &mut Vec<u64>,
        spare: &mut Vec<u64>,
    ) -> usize {
        keys.clear();

        // A square is no less than zero, so its bits order as it does.
        let key = |index: usize| {
            let square = squared_distance(&self.bounds[index], point);

            u64::from(square.to_bits()) << 32 | index as u64
        };

        match shown {
            Some(shown) => keys.extend(shown.iter().map(|&index| key(index))),
            None => keys.extend((0..self.bounds.len()).map(key)),
        }

        sort_keys(keys, spare);
        keys.partition_point(|&key| f32::from_bits((key >> 32) as u32) <= reach * reach)
    }

    /// Appends the boxes at `indices` to `sorted` a page at a time, and a
    /// run of `part` to `runs` for each of the `pages` that holds one of
    /// them. The boxes keep their order within a page.
    pub fn deal(
        &self,
        indices: impl Iterator<Item = usize> + Clone,
        part: usize,
        pages: usize,
        sorted: &mut Vec<[u32; 12]>,
        runs: &mut Vec<BoxRun>,
    ) {
        for page in 0..pages {
            let before = sorted.len();
            let held = indices.clone().filter(|&index| self.page[index] == page);

            sorted.extend(held.map(|index| self.words[index]));

            if sorted.len() > before {
                runs.push(BoxRun {
                    part,
                    page,
                    count: to_word(sorted.len() - before),
                });
            }
        }
    }

    /// Appends the boxes at `indices` to `sorted` in order, and a run of
    /// `part` to `runs` for each stretch of them that one page holds.
    pub fn deal_in_order(
        &self,
        indices: impl Iterator<Item = usize>,
        part: usize,
        sorted: &mut Vec<[u32; 12]>,
        runs: &mut Vec<BoxRun>,
    ) {
        // Whether a run of this call is the last of `runs`.
        let mut open = false;

        for index in indices {
            let page = self.page[index];

            sorted.push(self.words[index]);

            match runs.last_mut().filter(|run| open && run.page == page) {
                Some(run) => run.count += 1,
                None => runs.push(BoxRun {
                    part,
                    page,
                    count: 1,
                }),
            }

            open = true;
        }
    }
}
