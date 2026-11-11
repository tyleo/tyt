// The box raster's fragment over front faces. Depth runs reversed, so a hit
// past its face has a lower depth. The fragment declares that bound, which
// lets the depth test run before the march.

@fragment
@early_depth_test(less_equal)
fn front(box: Box) -> Fragment {
    var fragment = first_hit(box);

    fragment.depth = min(fragment.depth, box.clip.z);

    return fragment;
}
