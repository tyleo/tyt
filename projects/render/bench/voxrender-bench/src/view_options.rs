/// How the view of an image or a check draws its scene.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ViewOptions {
    /// The bytes of a page of the view's tiles, as
    /// [`BoxView::new`](crate::BoxView::new) takes them.
    pub page_bytes: Option<u64>,

    /// The pixels [`BoxView::set_detail`](crate::BoxView::set_detail) takes.
    pub detail: f64,
}
