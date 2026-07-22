use crate::TreeGridNode;

impl<V> TreeGridNode<V> {
    /// The label and annotation as printed wherever a layout labels
    /// this node.
    pub(crate) fn annotated_label(&self) -> String {
        let mut label = self.label.render();
        if let Some(annotation) = &self.annotation {
            label.push(' ');
            label.push_str(annotation);
        }
        label
    }
}
