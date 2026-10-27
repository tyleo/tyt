use ty_math::{TyVector3F32, TyVector3U32};

/// A run of cells on one face plane. The tangent axes `u` and `v` follow
/// `d` cyclically.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SurfaceSpan {
    /// The axis the face's normal lies along.
    pub d: usize,

    /// The normal's direction along `d`, `1` or `-1`.
    pub sign: i32,

    /// The layer of cells along `d` whose faces the span covers.
    pub s: u32,

    /// The first cell along `u`.
    pub u0: usize,

    /// One past the last cell along `u`.
    pub u1: usize,

    /// The first cell along `v`.
    pub v0: usize,

    /// One past the last cell along `v`.
    pub v1: usize,
}

impl SurfaceSpan {
    /// The first tangent axis.
    pub fn u(&self) -> usize {
        (self.d + 1) % 3
    }

    /// The second tangent axis.
    pub fn v(&self) -> usize {
        (self.d + 2) % 3
    }

    /// The grid position of the cell at `uu` along `u` and `vv` along `v`.
    pub fn cell(&self, uu: usize, vv: usize) -> TyVector3U32 {
        let mut position = [0u32; 3];
        position[self.d] = self.s;
        position[self.u()] = u32::try_from(uu).expect("a slice fits the grid");
        position[self.v()] = u32::try_from(vv).expect("a slice fits the grid");
        TyVector3U32::from_array(position)
    }

    /// The grid positions of every cell the span covers, `v` outermost.
    pub fn cells(&self) -> impl Iterator<Item = TyVector3U32> + '_ {
        (self.v0..self.v1).flat_map(move |vv| (self.u0..self.u1).map(move |uu| self.cell(uu, vv)))
    }

    /// Each outer corner's `u` and `v` in winding order, counter-clockwise
    /// seen from outside. `u` then `v` turn counter-clockwise about `+d`, so
    /// the `+` side runs `(u0, v0)`, `(u1, v0)`, `(u1, v1)`, `(u0, v1)` and
    /// the `-` side `(u0, v0)`, `(u0, v1)`, `(u1, v1)`, `(u1, v0)`.
    pub fn corners(&self) -> [[usize; 2]; 4] {
        let (u0, u1, v0, v1) = (self.u0, self.u1, self.v0, self.v1);

        if self.sign > 0 {
            [[u0, v0], [u1, v0], [u1, v1], [u0, v1]]
        } else {
            [[u0, v0], [u0, v1], [u1, v1], [u1, v0]]
        }
    }

    /// The point on the face plane at `along_u` and `along_v`. The `+` side
    /// sits one unit past the slice along `d`, the `-` side on it.
    pub fn corner(&self, along_u: f32, along_v: f32) -> TyVector3F32 {
        let mut point = [0f32; 3];
        point[self.d] = self.s as f32 + if self.sign > 0 { 1.0 } else { 0.0 };
        point[self.u()] = along_u;
        point[self.v()] = along_v;
        TyVector3F32::from_array(point)
    }

    /// The outward normal.
    pub fn normal(&self) -> TyVector3F32 {
        let mut normal = [0f32; 3];
        normal[self.d] = self.sign as f32;
        TyVector3F32::from_array(normal)
    }
}

#[cfg(test)]
mod tests {
    use crate::SurfaceSpan;
    use ty_math::{TyVector3F32, TyVector3U32};

    #[test]
    fn the_cells_run_u_fastest_and_the_corners_sit_on_the_plane() {
        let span = SurfaceSpan {
            d: 1,
            sign: 1,
            s: 2,
            u0: 1,
            u1: 3,
            v0: 0,
            v1: 2,
        };

        // Axis 1's tangents are z then x.
        assert_eq!(
            span.cells().collect::<Vec<_>>(),
            [
                TyVector3U32::new(0, 2, 1),
                TyVector3U32::new(0, 2, 2),
                TyVector3U32::new(1, 2, 1),
                TyVector3U32::new(1, 2, 2),
            ]
        );
        assert_eq!(span.corner(1.0, 2.0), TyVector3F32::new(2.0, 3.0, 1.0));
        assert_eq!(span.normal(), TyVector3F32::new(0.0, 1.0, 0.0));
        assert_eq!(span.corners(), [[1, 0], [3, 0], [3, 2], [1, 2]]);

        let negative = SurfaceSpan { sign: -1, ..span };
        assert_eq!(negative.corner(1.0, 2.0), TyVector3F32::new(2.0, 2.0, 1.0));
        assert_eq!(negative.corners(), [[1, 0], [1, 2], [3, 2], [3, 0]]);
    }
}
