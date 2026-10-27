use crate::{VMaxBrushState, VMaxFlag, VMaxFlagValue, VMaxMode, VMaxToolMode, VMaxViewBox};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A contents file's tool settings (`.vmaxb` `tools`): the work area `vp`,
/// outside which Voxel Max hides an object's voxels, and the editor's tool
/// context. Voxel Max opens no object whose `tools` lacks the context, so
/// every setting but `src` is required. An unknown key is an error.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct VMaxTools {
    /// The work area ([`VMaxViewBox`]), inclusive.
    pub vp: VMaxViewBox,

    /// Active layer token.
    pub al: String,

    /// Active brush index.
    pub bi: i64,

    /// Brush size.
    pub bs: i64,

    /// [`VMaxBrushState`].
    pub bst: VMaxBrushState,

    /// Color tool.
    pub ct: VMaxToolMode,

    /// Color-add tool.
    pub cta: VMaxToolMode,

    /// Color-copy tool.
    pub ctc: VMaxToolMode,

    /// Color-erase tool.
    pub cte: VMaxToolMode,

    /// Color-material tool.
    pub ctm: VMaxToolMode,

    /// Color-paint tool.
    pub ctp: VMaxToolMode,

    /// Color-select tool.
    pub cts: VMaxToolMode,

    /// Draw-mode brush.
    pub dm: VMaxToolMode,

    /// Draw-mode brush-b.
    pub dmb: VMaxToolMode,

    /// Draw-mode color.
    pub dmc: VMaxToolMode,

    /// Draw-mode layer.
    pub dml: VMaxToolMode,

    /// Draw-mode select.
    pub dms: VMaxToolMode,

    /// Active material index.
    pub mi: i64,

    /// Mirror [`VMaxFlag`].
    pub mr: VMaxFlag,

    /// Paint color-material tool.
    pub pctm: VMaxToolMode,

    /// Symmetry [`VMaxFlag`].
    pub st: VMaxFlag,

    /// Selection-tool [`VMaxFlag`].
    pub stf: VMaxFlag,

    /// Active-tool or source index; present on some objects.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub src: Option<i64>,
}

impl VMaxTools {
    /// The tool settings of a fresh Voxel Max object, from a new document's
    /// default object, with the work area `vp`.
    pub fn fresh(vp: VMaxViewBox) -> Self {
        let mode = |mo: &str| VMaxMode {
            mo: Some(mo.to_owned()),
            ..Default::default()
        };

        let draw = |m: &str| VMaxMode {
            m: Some(m.to_owned()),
            ..Default::default()
        };

        let off = || VMaxFlag {
            x: VMaxFlagValue::Bool(false),
            y: None,
            z: None,
        };

        VMaxTools {
            vp,
            al: "1".to_owned(),
            bi: 0,
            bs: 1,
            bst: VMaxBrushState {
                cm: "ng".to_owned(),
                cp: "n".to_owned(),
                gm: "u".to_owned(),
                gp: "n".to_owned(),
                ocx: Some(0),
                ocn: Some(-1),
                sfaz: None,
                sfat: None,
            },
            ct: VMaxToolMode {
                c: Some(mode("v")),
                ..Default::default()
            },
            cta: VMaxToolMode {
                a: Some(mode("m")),
                ..Default::default()
            },
            ctc: VMaxToolMode {
                c: Some(mode("v")),
                ..Default::default()
            },
            cte: VMaxToolMode {
                e: Some(mode("v")),
                ..Default::default()
            },
            ctm: VMaxToolMode {
                m: Some(mode("d")),
                ..Default::default()
            },
            ctp: VMaxToolMode {
                p: Some(mode("v")),
                ..Default::default()
            },
            cts: VMaxToolMode {
                s: Some(VMaxMode {
                    mf: Some("nw".to_owned()),
                    ..mode("v")
                }),
                ..Default::default()
            },
            dm: VMaxToolMode {
                b: Some(draw("d")),
                ..Default::default()
            },
            dmb: VMaxToolMode {
                b: Some(draw("d")),
                ..Default::default()
            },
            dmc: VMaxToolMode {
                c: Some(draw("e")),
                ..Default::default()
            },
            dml: VMaxToolMode {
                l: Some(draw("d")),
                ..Default::default()
            },
            dms: VMaxToolMode {
                s: Some(VMaxMode {
                    t: Some("f".to_owned()),
                    ..draw("c8")
                }),
                ..Default::default()
            },
            mi: 35,
            mr: off(),
            pctm: VMaxToolMode {
                m: Some(mode("d")),
                ..Default::default()
            },
            st: off(),
            stf: VMaxFlag {
                x: VMaxFlagValue::Int(1),
                y: Some(VMaxFlagValue::Int(0)),
                z: Some(VMaxFlagValue::Int(4)),
            },
            src: None,
        }
    }
}
