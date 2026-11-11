// `box_masks.wgsl`'s fragment over front faces. The rasterizer's point is
// where the ray enters the box. Depth runs reversed, so a hit past that
// point has a lower depth. The fragment declares that bound, which lets the
// depth test run before the march.

@fragment
@early_depth_test(less_equal)
fn front(box: Box) -> Fragment {
    return first_hit(box, box.point, direction_of(box), 0.0, true);
}
