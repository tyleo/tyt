use branded_id::U32Id;
use voxcore::BVoxObject;

/// The filename suffix for an object: empty for object 0, then its numeric id.
pub(crate) fn object_file_suffix(object_id: U32Id<BVoxObject>) -> String {
    let index = object_id.to_u32();
    if index == 0 {
        String::new()
    } else {
        index.to_string()
    }
}
