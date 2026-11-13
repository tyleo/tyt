use crate::{CellGrid, ObjectTiles, Result};
use ty_math::TyVector3U32;

/// The cells along each side of a tile.
const TILE: u32 = 16;

/// The tiles of `grid` and then of each of its coarser copies, finest
/// first, with the empty cells the grid takes ahead of its first cell
/// along each axis. A [`TileScene`](crate::TileScene) keeps them as one
/// object. They read nothing a scene changes as it takes other objects, so
/// a loader can build several objects' at once. Errors as
/// [`ObjectTiles::new`] does.
///
/// With `coarser`, each copy halves the one before, as
/// [`CellGrid::halved`] does, down to a copy that fits one tile. A copy's
/// cells sit on the lattice of its size, which the grid's first cell
/// reaches by the empty cells ahead of it.
///
/// # Arguments
///
/// 1. `first`: where the grid's first cell sits on the lattice its scene's
///    objects share, in cells, or zeros for an object off any lattice
/// 2. `opaque`: whether each of the scene's materials passes no light
/// 3. `glass`: for a scene that keeps the surfaces between materials that
///    pass light, those materials in the order a glass cell's word counts
///    them
pub fn object_copies(
    grid: &CellGrid,
    first: [i64; 3],
    coarser: bool,
    opaque: &[bool],
    glass: Option<&[u32]>,
) -> Result<(Vec<ObjectTiles>, TyVector3U32)> {
    let copies = if coarser {
        (0..)
            .find(|&copies| grid.bounds.max_element() >> copies <= TILE)
            .unwrap_or(0)
    } else {
        0
    };

    let shift = TyVector3U32::from_array(first.map(|cell| cell.rem_euclid(1 << copies) as u32));
    let padded;

    let mut level = if shift == TyVector3U32::ZERO {
        grid
    } else {
        padded = grid.padded(shift);

        &padded
    };

    let mut halved;
    let mut tiles = Vec::new();

    for copy in 0..=copies {
        // A glass cell's word holds where its material's seven values
        // start among a view's.
        let mut place = |material: u32| {
            let place = glass
                .and_then(|glass| glass.iter().position(|&placed| placed == material))
                .expect("the scene placed each material that passes light");

            u32::try_from(place * 7).expect("a scene's glass materials number under 586")
        };

        let place: Option<&mut dyn FnMut(u32) -> u32> = if glass.is_some() {
            Some(&mut place)
        } else {
            None
        };

        tiles.push(ObjectTiles::new(
            level,
            &|material| opaque[material as usize],
            place,
            copy == 0,
        )?);

        if copy < copies {
            halved = level.halved();
            level = &halved;
        }
    }

    Ok((tiles, shift))
}
