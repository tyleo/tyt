use crate::{Format, Result};
use std::{
    io::{Error as IOError, ErrorKind},
    path::Path,
};
use vmax_codec::from_vmax_package;
use voxcore::VoxMain;
use voxsmith::{
    from_goxl_bytes, from_mvox_bytes, from_qbcl_bytes, from_vmax_file, from_voxj_bytes,
};

/// Loads the voxel file at `input` into a [`VoxMain`], the common front half of
/// every converter. `from` names the source format, inferred from `input`'s
/// extension when `None`. Single-file formats decode their bytes through
/// voxsmith; Voxel Max reads the whole `.vmax` package directory first.
pub(crate) fn load_state(input: &Path, from: Option<Format>) -> Result<VoxMain> {
    let state = match resolve(input, from)? {
        Format::Voxj => from_voxj_bytes(&tyt_injection::read_file(input)?)?,
        Format::MVox => from_mvox_bytes(&tyt_injection::read_file(input)?)?,
        Format::Goxl => from_goxl_bytes(&tyt_injection::read_file(input)?)?,
        Format::Qbcl => from_qbcl_bytes(&tyt_injection::read_file(input)?)?,
        Format::VMax => {
            // Read the whole package into the lossless Voxel Max model, then load
            // it into voxcore. List every package-relative path, descending one
            // level into subdirectories (only `QuickLook/`) so its thumbnails
            // keep their prefix; resolve reads each path's bytes.
            let serde = from_vmax_package(
                || {
                    let mut paths = Vec::new();
                    for entry in tyt_injection::list_dir(input)? {
                        let Some(name) = entry.file_name().and_then(|n| n.to_str()) else {
                            continue;
                        };
                        if entry.is_dir() {
                            for child in tyt_injection::list_dir(&entry)? {
                                if let Some(child) = child.file_name().and_then(|n| n.to_str()) {
                                    paths.push(format!("{name}/{child}"));
                                }
                            }
                        } else {
                            paths.push(name.to_owned());
                        }
                    }
                    Ok(paths)
                },
                |name| match tyt_injection::read_file(&input.join(name)) {
                    Ok(bytes) => Ok(Some(bytes)),
                    Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
                    Err(e) => Err(e.into()),
                },
            )?;
            from_vmax_file(&serde)?
        }
    };
    Ok(state)
}

/// The source format to read `input` as: `from` when given, else inferred from
/// the path's extension. Errors when neither names a supported format.
fn resolve(input: &Path, from: Option<Format>) -> Result<Format> {
    match from {
        Some(format) => Ok(format),
        None => Format::from_path(input).ok_or_else(|| {
            IOError::new(
                ErrorKind::InvalidInput,
                format!(
                    "could not infer the input format from `{}`; pass --from",
                    input.display()
                ),
            )
            .into()
        }),
    }
}
