use crate::{
    CliValue, Error, Result,
    commands::{LightEntry, LightKind, TransformFrame},
};
use ty_math::{TyLinSrgbF64, TyVector3F64};
use voxsmith::operations::object::{
    LightRecord, PositionTransform, RenderShadow, Rotation, RotationTransform,
};

/// The shadow granularity of a light that sets none. The three-shadow
/// renders picked it because it reads as one look with the corner
/// occlusion and the standalone tier computes it exactly.
const DEFAULT_SHADOW: RenderShadow = RenderShadow::PerCorner;

/// A light's elements as the flags set them, over the profile rig's entry
/// where the rig supplies one. A flag setting an element twice errors, as
/// does one whose element the light's kind never reads.
#[derive(Debug)]
pub struct LightElements {
    index: u32,

    kind: LightKind,

    entry: Option<LightEntry>,

    frame: Option<TransformFrame>,

    node: Option<String>,

    position: Option<TyVector3F64>,

    rotation: Option<Rotation>,

    orbit: Option<PositionTransform>,

    shadow: Option<RenderShadow>,

    color: Option<TyLinSrgbF64>,

    strength: Option<f64>,

    range: Option<f64>,

    sky: Option<TyLinSrgbF64>,

    ground: Option<TyLinSrgbF64>,
}

impl LightElements {
    /// The empty elements of light `index`, declared as `kind`.
    pub(crate) fn declared(index: u32, kind: LightKind) -> Self {
        LightElements {
            index,
            kind,
            entry: None,
            frame: None,
            node: None,
            position: None,
            rotation: None,
            orbit: None,
            shadow: None,
            color: None,
            strength: None,
            range: None,
            sky: None,
            ground: None,
        }
    }

    /// The empty elements of light `index` over the rig's `entry`.
    pub(crate) fn from_entry(index: u32, entry: LightEntry) -> Self {
        LightElements {
            entry: Some(entry.clone()),
            ..LightElements::declared(index, entry.kind())
        }
    }

    /// Sets the frame the transform is read in, which `flag` gives.
    pub(crate) fn set_frame(&mut self, flag: &str, frame: TransformFrame) -> Result<()> {
        self.check_kind(flag, &[LightKind::Directional, LightKind::Point])?;
        self.check_posed(flag)?;

        self.claim(flag, "frame", |elements| &mut elements.frame, frame)
    }

    /// Sets the node path the `node` frame reads, which `flag` gives.
    pub(crate) fn set_node(&mut self, flag: &str, path: String) -> Result<()> {
        self.check_kind(flag, &[LightKind::Directional, LightKind::Point])?;
        self.check_posed(flag)?;

        self.claim(flag, "node path", |elements| &mut elements.node, path)
    }

    /// Sets the position, which `flag` gives.
    pub(crate) fn set_position(&mut self, flag: &str, position: TyVector3F64) -> Result<()> {
        self.check_kind(flag, &[LightKind::Point])?;
        self.check_posed(flag)?;

        self.claim(
            flag,
            "position",
            |elements| &mut elements.position,
            position,
        )
    }

    /// Sets the rotation, which `flag` gives.
    pub(crate) fn set_rotation(&mut self, flag: &str, rotation: Rotation) -> Result<()> {
        self.check_kind(flag, &[LightKind::Directional])?;

        self.claim(
            flag,
            "rotation",
            |elements| &mut elements.rotation,
            rotation,
        )
    }

    /// Sets the whole transform to the orbit `flag` gives.
    pub(crate) fn set_orbit(&mut self, flag: &str, orbit: PositionTransform) -> Result<()> {
        self.check_kind(flag, &[LightKind::Point])?;

        if self.frame.is_some() || self.node.is_some() || self.position.is_some() {
            return Err(Error::usage(format!(
                "{flag} sets light {}'s transform, which --light-frame, --light-node, or \
                 --light-position sets already",
                self.index
            )));
        }

        self.claim(flag, "transform", |elements| &mut elements.orbit, orbit)
    }

    /// Sets the shadow granularity, which `flag` gives.
    pub(crate) fn set_shadow(&mut self, flag: &str, shadow: RenderShadow) -> Result<()> {
        self.check_kind(flag, &[LightKind::Directional, LightKind::Point])?;

        self.claim(flag, "shadow", |elements| &mut elements.shadow, shadow)
    }

    /// Sets the color, which `flag` gives.
    pub(crate) fn set_color(&mut self, flag: &str, color: TyLinSrgbF64) -> Result<()> {
        self.check_kind(flag, &[LightKind::Directional, LightKind::Point])?;

        self.claim(flag, "color", |elements| &mut elements.color, color)
    }

    /// Sets the strength, which `flag` gives.
    pub(crate) fn set_strength(&mut self, flag: &str, strength: f64) -> Result<()> {
        self.claim(
            flag,
            "strength",
            |elements| &mut elements.strength,
            strength,
        )
    }

    /// Sets the range in meters, which `flag` gives.
    pub(crate) fn set_range(&mut self, flag: &str, range: f64) -> Result<()> {
        self.check_kind(flag, &[LightKind::Point])?;

        self.claim(flag, "range", |elements| &mut elements.range, range)
    }

    /// Sets the sky color, which `flag` gives.
    pub(crate) fn set_sky(&mut self, flag: &str, sky: TyLinSrgbF64) -> Result<()> {
        self.check_kind(flag, &[LightKind::Hemisphere])?;

        self.claim(flag, "sky", |elements| &mut elements.sky, sky)
    }

    /// Sets the ground color, which `flag` gives.
    pub(crate) fn set_ground(&mut self, flag: &str, ground: TyLinSrgbF64) -> Result<()> {
        self.check_kind(flag, &[LightKind::Hemisphere])?;

        self.claim(flag, "ground", |elements| &mut elements.ground, ground)
    }

    /// The light record the elements lower into. A flag's element stands, the
    /// rig's entry fills the rest, and the standard defaults fill what
    /// neither sets: a white color at strength `1`, no range, and the default
    /// shadow granularity. Errors if a directional or point light has no
    /// transform, a posed transform lacks a part, a directional light's frame
    /// is `subject`, or a node path sits under another frame.
    pub(crate) fn finish(self) -> Result<LightRecord> {
        let index = self.index;
        let white = TyLinSrgbF64::new(1.0, 1.0, 1.0);

        let node = self.node;
        let has_node = node.is_some();

        let node_path = |frame| node_path(index, frame, node);

        Ok(match self.kind {
            LightKind::Directional => {
                let (entry_transform, entry_shadow, entry_color, entry_strength) = match self.entry
                {
                    Some(LightEntry::Directional {
                        transform,
                        shadow,
                        color,
                        strength,
                    }) => (transform, shadow, color, strength),

                    _ => (None, None, None, None),
                };

                let transform = match (self.frame, self.rotation) {
                    (None, None) if !has_node => entry_transform
                        .map(|transform| transform.into_transform())
                        .ok_or_else(|| {
                            Error::usage(format!(
                                "light {index} has no transform; give it --light-frame and a \
                                 rotation flag"
                            ))
                        })?,

                    (Some(frame), Some(rotation)) => match (frame, node_path(frame)?) {
                        (TransformFrame::World, _) => RotationTransform::World { rotation },

                        (TransformFrame::Camera, _) => RotationTransform::Camera { rotation },

                        (TransformFrame::Node, path) => RotationTransform::Node {
                            path: path.expect("the node frame found its path"),
                            rotation,
                        },

                        (TransformFrame::Subject, _) => {
                            return Err(Error::usage(format!(
                                "light {index}'s frame is subject, and a directional light's \
                                 frame is world, camera, or node"
                            )));
                        }
                    },

                    (frame, _) => {
                        return Err(Error::usage(format!(
                            "light {index}'s transform lacks {}; a directional light takes \
                             --light-frame and a rotation flag",
                            if frame.is_none() {
                                "--light-frame"
                            } else {
                                "a rotation flag"
                            }
                        )));
                    }
                };

                LightRecord::Directional {
                    transform,
                    shadow: self
                        .shadow
                        .or(entry_shadow.map(|shadow| shadow.0))
                        .unwrap_or(DEFAULT_SHADOW),
                    color: self
                        .color
                        .or(entry_color.map(|color| color.to_linear()))
                        .unwrap_or(white),
                    strength: self
                        .strength
                        .or(entry_strength.map(|strength| strength.0))
                        .unwrap_or(1.0),
                }
            }

            LightKind::Point => {
                let (entry_transform, entry_shadow, entry_color, entry_strength, entry_range) =
                    match self.entry {
                        Some(LightEntry::Point {
                            transform,
                            shadow,
                            color,
                            strength,
                            range,
                        }) => (transform, shadow, color, strength, range),

                        _ => (None, None, None, None, None),
                    };

                let transform = match (self.orbit, self.frame, self.position) {
                    (Some(orbit), ..) => orbit,

                    (None, None, None) if !has_node => entry_transform
                        .map(|transform| transform.into_transform())
                        .ok_or_else(|| {
                            Error::usage(format!(
                                "light {index} has no transform; give it --light-orbit, or \
                                 --light-frame with --light-position"
                            ))
                        })?,

                    (None, Some(frame), Some(position)) => match (frame, node_path(frame)?) {
                        (TransformFrame::World, _) => PositionTransform::World { position },

                        (TransformFrame::Subject, _) => PositionTransform::Subject { position },

                        (TransformFrame::Camera, _) => PositionTransform::Camera { position },

                        (TransformFrame::Node, path) => PositionTransform::Node {
                            path: path.expect("the node frame found its path"),
                            position,
                        },
                    },

                    (None, frame, _) => {
                        return Err(Error::usage(format!(
                            "light {index}'s transform lacks {}; a posed point light takes \
                             --light-frame and --light-position",
                            if frame.is_none() {
                                "--light-frame"
                            } else {
                                "--light-position"
                            }
                        )));
                    }
                };

                LightRecord::Point {
                    transform,
                    shadow: self
                        .shadow
                        .or(entry_shadow.map(|shadow| shadow.0))
                        .unwrap_or(DEFAULT_SHADOW),
                    color: self
                        .color
                        .or(entry_color.map(|color| color.to_linear()))
                        .unwrap_or(white),
                    strength: self
                        .strength
                        .or(entry_strength.map(|strength| strength.0))
                        .unwrap_or(1.0),
                    range: self.range.or(entry_range.map(|range| range.0)),
                }
            }

            LightKind::Hemisphere => {
                let (entry_sky, entry_ground, entry_strength) = match self.entry {
                    Some(LightEntry::Hemisphere {
                        sky,
                        ground,
                        strength,
                    }) => (sky, ground, strength),

                    _ => (None, None, None),
                };

                LightRecord::Hemisphere {
                    sky: self
                        .sky
                        .or(entry_sky.map(|sky| sky.to_linear()))
                        .unwrap_or(white),
                    ground: self
                        .ground
                        .or(entry_ground.map(|ground| ground.to_linear()))
                        .unwrap_or(white),
                    strength: self
                        .strength
                        .or(entry_strength.map(|strength| strength.0))
                        .unwrap_or(1.0),
                }
            }
        })
    }

    /// Errors unless the light's kind is one of `kinds`, the kinds `flag`
    /// applies to.
    fn check_kind(&self, flag: &str, kinds: &[LightKind]) -> Result<()> {
        if kinds.contains(&self.kind) {
            return Ok(());
        }

        let applies: Vec<_> = kinds.iter().map(|kind| kind.name()).collect();

        Err(Error::usage(format!(
            "{flag} applies to a {} light, and light {} is {}",
            applies.join(" or "),
            self.index,
            self.kind.name()
        )))
    }

    /// Errors when `flag` poses a light whose transform an orbit set already.
    fn check_posed(&self, flag: &str) -> Result<()> {
        if self.orbit.is_some() {
            return Err(Error::usage(format!(
                "{flag} sets light {}'s transform, which --light-orbit sets already",
                self.index
            )));
        }

        Ok(())
    }

    /// Fills the slot `slot` picks with `value`, the `element` that `flag`
    /// gives. A slot filled already errors.
    fn claim<T>(
        &mut self,
        flag: &str,
        element: &str,
        slot: impl FnOnce(&mut Self) -> &mut Option<T>,
        value: T,
    ) -> Result<()> {
        let index = self.index;
        let slot = slot(self);

        if slot.is_some() {
            return Err(Error::usage(format!(
                "{flag} sets light {index}'s {element}, which is set already"
            )));
        }

        *slot = Some(value);

        Ok(())
    }
}

/// The node path light `index` reads under `frame`: the path given under the
/// `node` frame, and none under another. Errors if a path sits under another
/// frame or the `node` frame has none.
fn node_path(index: u32, frame: TransformFrame, node: Option<String>) -> Result<Option<String>> {
    match (frame, node) {
        (TransformFrame::Node, Some(path)) => Ok(Some(path)),

        (TransformFrame::Node, None) => Err(Error::usage(format!(
            "light {index}'s frame is node, and its transform lacks --light-node"
        ))),

        (_, Some(path)) => Err(Error::usage(format!(
            "light {index} reads the node path `{path}`, which applies under the node frame, and \
             its frame is {}",
            frame.name()
        ))),

        (_, None) => Ok(None),
    }
}
