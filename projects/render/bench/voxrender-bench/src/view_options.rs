/// How the view of an image or a check draws its scene.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ViewOptions {
    /// The bytes of a page of the view's tiles, as
    /// [`BoxView::paged`](crate::BoxView::paged) takes them, in place of
    /// what one of the device's storage bindings holds.
    pub page_bytes: Option<u64>,

    /// The fewest pixels one of an object's cells spans before a coarser
    /// copy's cells draw in its place, as
    /// [`BoxView::set_detail`](crate::BoxView::set_detail) takes them. Zero
    /// draws every object's cells.
    pub detail: f64,
}
