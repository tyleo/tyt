use crate::Deflate;
use ty_zip::wrap_deflated_member;

/// Conventional name of the single `.sdfj` member inside a `.sdfjz` archive.
const MEMBER_NAME: &str = "main.sdfj";

/// Wraps a `.sdfj` byte payload in a single-member, deflate-compressed `.sdfjz`
/// zip archive through `dependencies`.
pub fn wrap_sdfjz<D: Deflate>(dependencies: &D, sdfj: &[u8]) -> Vec<u8> {
    wrap_deflated_member(MEMBER_NAME, sdfj, &dependencies.deflate(sdfj))
}
