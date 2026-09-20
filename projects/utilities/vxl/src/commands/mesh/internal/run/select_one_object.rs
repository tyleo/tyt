use crate::{Error, ObjectSelection, Result};
use voxcore::{VoxExt, VoxMain, VoxObject};

/// The one object `selection` resolves to in `main`. `mesh` outputs one
/// object, so a selection resolving to several is a usage error, and so is a
/// document holding none; `resolve` already rejects a selector matching
/// nothing.
pub(crate) fn select_one_object<'a, T: VoxExt>(
    main: &'a VoxMain<T>,
    selection: &ObjectSelection,
) -> Result<&'a VoxObject> {
    let object_ids = selection.resolve(main)?;

    let object_id = match object_ids.as_slice() {
        [object_id] => *object_id,

        [] => return Err(Error::usage("the document has no objects to mesh")),

        object_ids => {
            return Err(Error::usage(format!(
                "the selection resolved to {} objects, but `mesh` outputs exactly one; narrow \
                 it with --select or --select-index",
                object_ids.len(),
            )));
        }
    };

    Ok(main
        .object(object_id)
        .expect("the selection resolved an id from the main's objects"))
}
