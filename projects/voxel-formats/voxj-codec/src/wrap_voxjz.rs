use crate::Deflate;
use ty_zip::wrap_deflated_member;

/// Conventional name of the single `.voxj` member inside a `.voxjz` archive.
const MEMBER_NAME: &str = "main.voxj";

/// Wraps a `.voxj` byte payload in a single-member, deflate-compressed `.voxjz`
/// zip archive through `dependencies`.
pub fn wrap_voxjz<D: Deflate>(dependencies: &D, voxj: &[u8]) -> Vec<u8> {
    wrap_deflated_member(MEMBER_NAME, voxj, &dependencies.deflate(voxj))
}
