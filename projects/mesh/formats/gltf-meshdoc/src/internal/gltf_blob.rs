use gltf::json::{
    Index,
    buffer::{Target, View},
    validation::{Checked, USize64},
};

/// The one binary buffer a written document's accessors read, assembled
/// region by region, and the buffer views that slice it.
#[derive(Default)]
pub struct GltfBlob {
    /// The bytes so far.
    pub bytes: Vec<u8>,

    /// The views over [`bytes`](Self::bytes), in push order.
    pub views: Vec<View>,
}

impl GltfBlob {
    /// Appends `data` as its own view, padded to a 4-byte boundary first,
    /// and returns the view's index.
    pub fn push_view(&mut self, data: &[u8], target: Option<Target>) -> Index<View> {
        while !self.bytes.len().is_multiple_of(4) {
            self.bytes.push(0);
        }

        let offset = self.bytes.len();
        self.bytes.extend_from_slice(data);

        let view = View {
            buffer: Index::new(0),
            byte_length: USize64::from(data.len()),
            byte_offset: Some(USize64::from(offset)),
            byte_stride: None,
            name: None,
            target: target.map(Checked::Valid),
            extensions: None,
            extras: Default::default(),
        };

        Index::push(&mut self.views, view)
    }
}
