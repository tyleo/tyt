use crate::{
    FALLBACK_CONTENT_VERSION, VMaxExt, VMaxExtObjectState, place_object, synth_object_uuid,
};
use std::collections::HashSet;
use vmax::{
    VMaxBrush, VMaxBrushColor, VMaxBrushEntry, VMaxBrushState, VMaxCamera, VMaxFlag, VMaxFlagValue,
    VMaxMode, VMaxToolMode, VMaxTools, VMaxViewBox,
};
use voxcore::VoxObject;

/// The editor state of an object the document never carried, built against
/// the entries `ext` already holds so its contents UUID is fresh. It takes the
/// default session a fresh Voxel Max object has, camera framed on the object's
/// content center, because Voxel Max's object decoder rejects a sparse
/// session. The synthesizer and the retain hook both build entries here.
pub fn synthesized_object_state(ext: &VMaxExt, object: &VoxObject) -> VMaxExtObjectState {
    let uuids: HashSet<&str> = ext
        .object_states
        .values()
        .map(|entry| entry.uuid.as_str())
        .collect();
    let uuid = (0..)
        .map(synth_object_uuid)
        .find(|uuid| !uuids.contains(uuid.as_str()))
        .expect("a fresh index exists");
    let (_, placement) = place_object(object);
    VMaxExtObjectState {
        uuid,
        v: FALLBACK_CONTENT_VERSION,
        tools: Some(default_tools(placement.view_box)),
        brush: Some(default_brush()),
        cam: Some(default_camera(placement.center)),
    }
}

/// Default `tools` for a synthesized object. Voxel Max's object decoder rejects
/// a sparse `tools`, so this mirrors a fresh object's editor state. `view_box`
/// scopes the view/edit partition (`vp`) to the object's build volume.
fn default_tools(view_box: VMaxViewBox) -> VMaxTools {
    // A mode dict that sets only `mo`, or only `m`.
    let mo = |mo: &str| VMaxMode {
        mo: Some(mo.to_owned()),
        ..Default::default()
    };
    let m = |m: &str| VMaxMode {
        m: Some(m.to_owned()),
        ..Default::default()
    };
    let flag = |x: VMaxFlagValue| {
        Some(VMaxFlag {
            x,
            y: None,
            z: None,
        })
    };
    // A tool with one active surface. The closure picks the surface field.
    fn tool(set: impl FnOnce(&mut VMaxToolMode)) -> Option<VMaxToolMode> {
        let mut mode = VMaxToolMode::default();
        set(&mut mode);
        Some(mode)
    }
    VMaxTools {
        bs: 1,
        mi: 0,
        bi: 0,
        al: "1".to_owned(),
        src: None,
        stf: flag(VMaxFlagValue::Int(1)),
        mr: flag(VMaxFlagValue::Bool(false)),
        st: flag(VMaxFlagValue::Bool(false)),
        vp: Some(view_box),
        bst: Some(VMaxBrushState {
            cm: "ng".to_owned(),
            cp: "n".to_owned(),
            gm: "u".to_owned(),
            gp: "n".to_owned(),
            ocx: Some(0),
            ocn: Some(-1),
            sfaz: None,
            sfat: None,
        }),
        ct: tool(|t| t.c = Some(mo("v"))),
        ctc: tool(|t| t.c = Some(mo("v"))),
        cte: tool(|t| t.e = Some(mo("v"))),
        ctp: tool(|t| t.p = Some(mo("v"))),
        cts: tool(|t| {
            t.s = Some(VMaxMode {
                mo: Some("v".to_owned()),
                mf: Some("nw".to_owned()),
                ..Default::default()
            })
        }),
        ctm: tool(|t| t.m = Some(mo("d"))),
        pctm: tool(|t| t.m = Some(mo("d"))),
        cta: tool(|t| t.a = Some(mo("ma"))),
        dm: tool(|t| t.b = Some(m("d"))),
        dmb: tool(|t| t.b = Some(m("d"))),
        dmc: tool(|t| t.c = Some(m("e"))),
        dml: tool(|t| t.l = Some(m("d"))),
        dms: tool(|t| {
            t.s = Some(VMaxMode {
                m: Some("c8".to_owned()),
                t: Some("f".to_owned()),
                ..Default::default()
            })
        }),
    }
}

/// Default `brush` palette for a synthesized object, mirroring a fresh
/// document.
fn default_brush() -> VMaxBrush {
    use VMaxBrushEntry::{Bb, C, Ch, Db, E, Eh, Pr, Py};
    let color = |dm: [i64; 3]| VMaxBrushColor { dm: dm.to_vec() };
    VMaxBrush {
        name: "Palette #1".to_owned(),
        current: 0,
        brushes: vec![
            C(color([1, 1, 1])),
            Ch(color([5, 5, 5])),
            E(color([5, 5, 5])),
            Eh(color([5, 5, 5])),
            Bb(color([5, 5, 5])),
            Db(color([5, 5, 5])),
            Pr(color([5, 5, 5])),
            Py(color([5, 5, 5])),
        ],
    }
}

/// Default editor `cam` for a synthesized object. A single-object document
/// opens straight into this object-editor view, so the camera must orbit the
/// object's content, not the corner of its 0..256 internal grid: `target` is
/// the content center in internal-grid coordinates, the object's `e_c`,
/// matching the rig Voxel Max writes when it frames the object. The rest is a
/// neutral framed-view rig.
fn default_camera(target: [f64; 3]) -> VMaxCamera {
    VMaxCamera {
        wa: 0.0,
        ha: 0.1959133446216583,
        da: 0.0,
        lwa: 0.25,
        lha: 1.820913314819336,
        lda: 0.0,
        px: 0.0,
        py: 0.0,
        z: 512.0,
        o: target,
    }
}
