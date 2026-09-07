use crate::Error;

/// The shared rejection for `computed-occlusion`, which needs the
/// not-yet-supported unwrap layout. Both the `--texture` preset and a
/// `--texture-map` channel reject through here; it fires only under the
/// palette atlas, since the `--atlas unwrap` gate runs first.
pub(crate) fn computed_occlusion_unsupported() -> Error {
    Error::usage(
        "computed-occlusion needs the unwrap atlas layout, which is not yet supported; \
         use --atlas palette",
    )
}
