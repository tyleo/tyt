use std::{fs, io::Result, path::Path};

/// Copies all files from `src` into `dst`, creating `dst` if needed.
pub fn copy_dir(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        fs::copy(entry.path(), dst.join(entry.file_name()))?;
    }
    Ok(())
}
