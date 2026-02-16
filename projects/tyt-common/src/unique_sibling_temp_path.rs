use crate::temp_counter_next;
use std::{
    io::{Error as IOError, Result},
    path::{Path, PathBuf},
    process,
    time::{SystemTime, UNIX_EPOCH},
};

pub fn unique_sibling_temp_path(dst: &Path) -> Result<PathBuf> {
    let parent = dst.parent().unwrap_or_else(|| Path::new("."));
    let file_name = dst.file_name().and_then(|s| s.to_str()).unwrap_or("file");

    let now_ns = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(IOError::other)?
        .as_nanos();

    let pid = process::id();
    let n = temp_counter_next::temp_counter_next();

    let mut tmp = parent.to_path_buf();
    tmp.push(format!(".{}.tmp-{}-{}-{}", file_name, pid, now_ns, n));
    Ok(tmp)
}
