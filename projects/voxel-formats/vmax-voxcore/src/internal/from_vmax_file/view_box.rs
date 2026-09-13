use vmax::{VMaxFile, VMaxObject, VMaxViewBox};

/// The object's authored build volume (`tools.vp`) from its contents file, the
/// size the author was working in. `None` when the object has no contents or no
/// partition recorded.
pub(crate) fn view_box<'a>(serde: &'a VMaxFile, object: &VMaxObject) -> Option<&'a VMaxViewBox> {
    serde
        .contents_files
        .get(&object.data)?
        .tools
        .as_ref()?
        .vp
        .as_ref()
}
