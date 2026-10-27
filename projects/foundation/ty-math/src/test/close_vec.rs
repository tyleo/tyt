use crate::{TyVector3F64, close};

/// True when every component of `a` is [`close`] to the matching one of `b`.
pub fn close_vec(a: TyVector3F64, b: TyVector3F64) -> bool {
    close(a.x, b.x) && close(a.y, b.y) && close(a.z, b.z)
}
