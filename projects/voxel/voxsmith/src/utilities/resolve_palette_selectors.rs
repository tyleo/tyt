use crate::{Result, utilities::IdSelector};
use branded_id::{RangeInclusiveExt, U32Id};
use voxcore::{BVoxPalette, Error as VoxError, VoxExt, VoxMain};

/// The ids of the palettes `selectors` select in `main`, in palette order and
/// each once. Errors on a named id that is not one of the main's palettes.
pub fn resolve_palette_selectors<T: VoxExt>(
    main: &VoxMain<T>,
    selectors: &[IdSelector<BVoxPalette>],
) -> Result<Vec<U32Id<BVoxPalette>>> {
    for selector in selectors {
        let Some(range) = selector.as_range() else {
            continue;
        };

        for palette_id in range.clone().into_id_range() {
            if main.palette(palette_id).is_none() {
                return Err(VoxError::UnknownPalette { palette_id }.into());
            }
        }
    }

    Ok(main
        .iter_palettes()
        .map(|(palette_id, _)| palette_id)
        .filter(|&palette_id| {
            selectors
                .iter()
                .any(|selector| selector.contains(palette_id))
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use crate::utilities::{IdSelector, resolve_palette_selectors};
    use branded_id::U32Id;
    use std::ops::RangeInclusive;
    use voxcore::{BVoxPalette, VoxMain, VoxPalette};

    /// A main holding `count` empty palettes, and their ids.
    fn main_with_palettes(count: usize) -> (VoxMain, Vec<U32Id<BVoxPalette>>) {
        let mut main = VoxMain::default();

        let palette_ids = (0..count)
            .map(|_| main.retain_palette(VoxPalette::default()).unwrap())
            .collect();

        (main, palette_ids)
    }

    fn range(range: RangeInclusive<U32Id<BVoxPalette>>) -> IdSelector<BVoxPalette> {
        IdSelector::range(range).unwrap()
    }

    #[test]
    fn all_selects_every_palette() {
        let (main, palette_ids) = main_with_palettes(3);

        assert_eq!(
            resolve_palette_selectors(&main, &[IdSelector::all()]).unwrap(),
            palette_ids
        );
    }

    #[test]
    fn selectors_union_in_palette_order_and_select_each_palette_once() {
        let (main, palette_ids) = main_with_palettes(5);

        let selectors = [
            range(palette_ids[3]..=palette_ids[4]),
            IdSelector::id(palette_ids[0]),
            IdSelector::id(palette_ids[3]),
        ];

        assert_eq!(
            resolve_palette_selectors(&main, &selectors).unwrap(),
            [palette_ids[0], palette_ids[3], palette_ids[4]]
        );
    }

    #[test]
    fn all_beside_a_range_selects_every_palette_once() {
        let (main, palette_ids) = main_with_palettes(2);

        let selectors = [range(palette_ids[1]..=palette_ids[1]), IdSelector::all()];

        assert_eq!(
            resolve_palette_selectors(&main, &selectors).unwrap(),
            palette_ids
        );
    }

    #[test]
    fn no_selector_selects_nothing() {
        let (main, _) = main_with_palettes(2);

        assert_eq!(resolve_palette_selectors(&main, &[]).unwrap(), []);
    }

    #[test]
    fn all_over_no_palettes_selects_nothing() {
        let (main, _) = main_with_palettes(0);

        assert_eq!(
            resolve_palette_selectors(&main, &[IdSelector::all()]).unwrap(),
            []
        );
    }

    #[test]
    fn errors_on_a_named_id_the_main_lacks() {
        let (mut main, palette_ids) = main_with_palettes(3);

        main.release_palette(palette_ids[2]).unwrap();

        assert_eq!(
            resolve_palette_selectors(&main, &[IdSelector::id(palette_ids[2])])
                .unwrap_err()
                .to_string(),
            "palette 2 is not one of this state's"
        );

        assert_eq!(
            resolve_palette_selectors(
                &main,
                &[IdSelector::all(), range(palette_ids[1]..=palette_ids[2])]
            )
            .unwrap_err()
            .to_string(),
            "palette 2 is not one of this state's"
        );
    }
}
