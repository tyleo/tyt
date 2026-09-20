use ty_math::TyVector3F32;

/// A run of cells on one face plane: axis `d`'s `sign` side of slice `s`,
/// spanning `u` in `[u0, u1)` and `v` in `[v0, v1)`, where `u` and `v` are
/// the axes after `d`, cyclically.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FaceSpan {
    pub(crate) d: usize,
    pub(crate) sign: i32,
    pub(crate) s: u32,
    pub(crate) u0: usize,
    pub(crate) u1: usize,
    pub(crate) v0: usize,
    pub(crate) v1: usize,
}

impl FaceSpan {
    /// The first tangent axis.
    pub(crate) fn u(&self) -> usize {
        (self.d + 1) % 3
    }

    /// The second tangent axis.
    pub(crate) fn v(&self) -> usize {
        (self.d + 2) % 3
    }

    /// The grid position of the cell at `uu` along `u` and `vv` along `v`.
    pub(crate) fn cell(&self, uu: usize, vv: usize) -> [u32; 3] {
        let mut position = [0u32; 3];
        position[self.d] = self.s;
        position[self.u()] = u32::try_from(uu).expect("a slice fits the grid");
        position[self.v()] = u32::try_from(vv).expect("a slice fits the grid");
        position
    }

    /// The grid positions of every cell the span covers, `v` outermost.
    pub(crate) fn cells(&self) -> impl Iterator<Item = [u32; 3]> + '_ {
        (self.v0..self.v1).flat_map(move |vv| (self.u0..self.u1).map(move |uu| self.cell(uu, vv)))
    }

    /// The point on the face plane at `along_u` and `along_v`. The `+` side
    /// sits one unit past the slice along `d`, the `-` side on it.
    pub(crate) fn corner(&self, along_u: f32, along_v: f32) -> TyVector3F32 {
        let mut point = [0f32; 3];
        point[self.d] = self.s as f32 + if self.sign > 0 { 1.0 } else { 0.0 };
        point[self.u()] = along_u;
        point[self.v()] = along_v;
        TyVector3F32::from_array(point)
    }

    /// The outward normal.
    pub(crate) fn normal(&self) -> TyVector3F32 {
        let mut normal = [0f32; 3];
        normal[self.d] = self.sign as f32;
        TyVector3F32::from_array(normal)
    }
}

#[cfg(test)]
mod tests {
    use crate::operations::mesh::FaceSpan;
    use ty_math::TyVector3F32;

    #[test]
    fn the_cells_run_u_fastest_and_the_corners_sit_on_the_plane() {
        let span = FaceSpan {
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
            [[0, 2, 1], [0, 2, 2], [1, 2, 1], [1, 2, 2]]
        );
        assert_eq!(span.corner(1.0, 2.0), TyVector3F32::new(2.0, 3.0, 1.0));
        assert_eq!(span.normal(), TyVector3F32::new(0.0, 1.0, 0.0));

        let negative = FaceSpan { sign: -1, ..span };
        assert_eq!(negative.corner(1.0, 2.0), TyVector3F32::new(2.0, 2.0, 1.0));
    }
}
